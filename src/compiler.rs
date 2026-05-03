use indicatif::{ProgressBar, ProgressStyle};
use inkwell::context::Context;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ast::{
    ConstDecl, DictDecl, DictSet, Expr, ExprKind, IndexAssignment, ListDecl, ListPush,
    LiteralValue, Location, LoopStmt, MutDecl, Program, PropertyAssignment, Stmt, StmtKind,
    VarDecl, VarSet,
};
use crate::diagnostics::{humane_code, Annotation, Diagnostic, DiagnosticBag};
use crate::llvm_generator::LLVMGenerator;
use crate::modules::is_native_module;
use crate::om_contract::{load_builtin_om_contract, load_om_contract, OmContract};
use crate::om_scan::{scan_header, ScanOptions};
use crate::parser::{ParseError, Parser};
use crate::semantic_analyzer::{SemanticAnalyzer, SemanticError};
use crate::sps::SnifFeatureValue;
use crate::tools::{get_pkg_cflags, get_pkg_libs, has_pkg};

/// Options for the compiler build process.
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub output_name: Option<String>,
    pub target: Option<String>,
    pub profile: BuildProfile,
    pub opt_level: u8,
    pub lto: bool,
    pub release_size: bool,
    pub min_runtime: bool,
    pub tiny: bool,
    pub extreme: bool,
    pub strip: bool,
    pub opt_override: Option<String>,
    pub features: BTreeMap<String, SnifFeatureValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Humane,
    Systems,
    Baremetal,
}

impl Default for BuildProfile {
    fn default() -> Self {
        Self::Humane
    }
}

impl BuildProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "humane" | "default" | "dev" | "release" | "release-size" | "tiny" | "extreme" => {
                Some(Self::Humane)
            }
            "systems" => Some(Self::Systems),
            "baremetal" => Some(Self::Baremetal),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BuildProfile::Humane => "humane",
            BuildProfile::Systems => "systems",
            BuildProfile::Baremetal => "baremetal",
        }
    }
}

pub fn resolve_entry_file(cli_file: Option<String>) -> Result<String, String> {
    if let Some(f) = cli_file {
        return Ok(f);
    }

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if let Some(_manifest_path) = crate::sps::find_manifest(&cwd) {
        let (m, _p) = crate::sps::load_manifest_from(&cwd)?;
        return Ok(m.package.entry);
    }
    Err("SPS: no input file provided and `snask.snif` was not found in the current directory.\n\nHow to fix:\n- Build a file directly: `snask build main.snask`\n- Or create an SPS project: `snask init` and then `snask build`\n".to_string())
}

pub fn build_file(file_path: &str, options: BuildOptions) -> Result<(), String> {
    let pb = ProgressBar::new(7);
    pb.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    pb.set_message("Reading file");
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    pb.inc(1);

    pb.set_message("Parser (tokens/AST)");
    let mut parser = Parser::new(&source).map_err(|e| {
        pb.finish_and_clear();
        render_parser_diagnostic(file_path, &source, &e)
    })?;

    let (program_opt, parse_errors) = parser.parse_program_recovering(10);
    if !parse_errors.is_empty() {
        pb.finish_and_clear();
        return Err(render_parser_diagnostics(file_path, &source, &parse_errors));
    }
    let mut program = program_opt.unwrap_or_default();

    // Inject features as constants
    inject_features(&mut program, options.features.clone());
    pb.inc(1);

    // Validate entrypoint
    validate_entrypoint(&program)?;

    pb.set_message("Resolving imports");
    let mut resolved_program = Vec::new();
    let mut resolved_modules = HashSet::new();
    resolved_modules.insert(file_path.to_string());
    let entry_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
    resolve_imports(
        &mut program,
        entry_dir,
        &mut resolved_program,
        &mut resolved_modules,
    )?;
    pb.inc(1);

    pb.set_message("Expanding inheritance");
    expand_inheritance(&mut resolved_program)?;
    pb.inc(1);

    if options.profile == BuildProfile::Baremetal {
        let restrictions = find_baremetal_restrictions(&resolved_program);
        if !restrictions.is_empty() {
            pb.finish_and_clear();
            return Err(render_baremetal_restrictions(
                file_path,
                &source,
                &restrictions,
            ));
        }

        pb.finish_and_clear();
        return Err("error[S8002]: `baremetal` profile is recognized, but the freestanding backend is not implemented yet\n\nhelp: use `--profile systems` while no_std/no_runtime, custom entrypoints and linker scripts are being implemented.".to_string());
    }

    // Determine runtime linking strategy
    let needs_full_runtime = uses_full_runtime(&resolved_program);
    if options.min_runtime && needs_full_runtime {
        pb.finish_and_clear();
        return Err(
            "`--min-runtime` cannot be used with GUI/SQLite/Skia/Web imports.\n".to_string(),
        );
    }
    let link_tiny_runtime = options.tiny || (options.min_runtime && !needs_full_runtime);

    pb.set_message("Semantic analysis");
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.set_tiny_mode(options.tiny);
    analyzer.analyze(&resolved_program);
    if !analyzer.errors.is_empty() {
        pb.finish_and_clear();
        return Err(render_semantic_diagnostics(
            file_path,
            &source,
            &analyzer.errors,
        ));
    }
    pb.inc(1);

    pb.set_message("Generating LLVM IR");
    let context = Context::create();
    let mut generator = LLVMGenerator::new(&context, file_path);
    generator.set_om_contracts(load_om_contracts(&resolved_program)?);
    let ir = generator.generate(resolved_program.clone())?;
    pb.inc(1);

    let extra_pkgs = get_imported_pkgs(&resolved_program);
    link_binary(
        file_path,
        ir.into_bytes(),
        options,
        link_tiny_runtime,
        &pb,
        extra_pkgs,
    )?;

    pb.finish_with_message("OK");
    Ok(())
}

fn get_imported_pkgs(program: &[Stmt]) -> Vec<String> {
    let mut pkgs = Vec::new();
    let mut seen = HashSet::new();
    for stmt in program {
        match &stmt.kind {
            StmtKind::Import(module_name) => {
                if (module_name == "sqlite" || module_name == "zlib")
                    && seen.insert(module_name.clone())
                {
                    pkgs.push(resolve_pkg_name(module_name));
                } else if !is_native_module(module_name) {
                    let pkg = resolve_pkg_name(module_name);
                    if has_pkg(&pkg) && seen.insert(pkg.clone()) {
                        pkgs.push(pkg);
                    }
                }
            }
            StmtKind::ImportCOm { alias, .. } => {
                if seen.insert(alias.clone()) {
                    pkgs.push(alias.clone());
                }
            }
            _ => {}
        }
    }
    pkgs
}

fn load_om_contracts(program: &[Stmt]) -> Result<Vec<OmContract>, String> {
    let mut contracts = Vec::new();
    let mut seen = HashSet::new();

    for stmt in program {
        match &stmt.kind {
            StmtKind::Import(module_name) => {
                if (module_name == "sqlite" || module_name == "zlib")
                    && seen.insert(module_name.clone())
                {
                    contracts.push(load_builtin_om_contract(module_name)?);
                } else if !is_native_module(module_name) {
                    let pkg = resolve_pkg_name(module_name);
                    if has_pkg(&pkg) && seen.insert(module_name.clone()) {
                        contracts.push(load_or_generate_auto_om_contract(&pkg, module_name)?);
                    }
                }
            }
            StmtKind::ImportCOm { header, alias } => {
                if seen.insert(alias.clone()) {
                    contracts.push(load_or_generate_om_contract(header, alias)?);
                }
            }
            _ => {}
        }
    }

    Ok(contracts)
}

fn resolve_pkg_name(name: &str) -> String {
    if name == "sqlite" && has_pkg("sqlite3") && !has_pkg("sqlite") {
        return "sqlite3".to_string();
    }
    name.to_string()
}

fn find_header_for_pkg(pkg_name: &str) -> Result<String, String> {
    match pkg_name {
        "zlib" => Ok("zlib.h".to_string()),
        "sqlite3" | "sqlite" => Ok("sqlite3.h".to_string()),
        "sdl2" => Ok("SDL2/SDL.h".to_string()),
        "raylib" => Ok("raylib.h".to_string()),
        "gl" => Ok("GL/gl.h".to_string()),
        "libpng" | "png" => Ok("png.h".to_string()),
        "libuv" | "uv" => Ok("uv.h".to_string()),
        "freetype2" => Ok("ft2build.h".to_string()),
        "gtk+-3.0" | "gtk3" => Ok("gtk/gtk.h".to_string()),
        _ => {
            // Default pattern
            Ok(format!("{}.h", pkg_name))
        }
    }
}

fn load_or_generate_auto_om_contract(pkg_name: &str, alias: &str) -> Result<OmContract, String> {
    let header = find_header_for_pkg(pkg_name)?;
    let cflags = get_pkg_cflags(pkg_name).unwrap_or_default();
    let generated = scan_header(ScanOptions {
        header,
        lib: alias.to_string(),
        output: None,
        extra_cflags: cflags,
    })
    .map_err(|e| format!("OM-Snask-System scan failed for package `{pkg_name}`: {e}"))?;

    merge_om_override_if_present(generated, alias)
}

fn load_or_generate_om_contract(header: &str, alias: &str) -> Result<OmContract, String> {
    let generated = scan_header(ScanOptions {
        header: header.to_string(),
        lib: alias.to_string(),
        output: None,
        extra_cflags: get_pkg_cflags(alias).unwrap_or_default(),
    })
    .map_err(|e| format!("import_c_om failed while scanning `{header}` as `{alias}`: {e}"))?;

    merge_om_override_if_present(generated, alias)
}

fn merge_om_override_if_present(
    mut generated: OmContract,
    alias: &str,
) -> Result<OmContract, String> {
    let Some(path) = find_om_contract_override(alias) else {
        return Ok(generated);
    };

    let override_contract = load_om_contract(&path)?;

    for override_constant in override_contract.constants {
        if let Some(existing) = generated
            .constants
            .iter_mut()
            .find(|constant| constant.surface == override_constant.surface)
        {
            *existing = override_constant;
        } else {
            generated.constants.push(override_constant);
        }
    }

    for override_resource in override_contract.resources {
        if let Some(existing) = generated
            .resources
            .iter_mut()
            .find(|resource| resource.surface_type == override_resource.surface_type)
        {
            *existing = override_resource;
        } else {
            generated.resources.push(override_resource);
        }
    }

    for mut override_function in override_contract.functions {
        if let Some(existing) = generated
            .functions
            .iter_mut()
            .find(|function| function.surface == override_function.surface)
        {
            if override_function.c_return_type.is_none() {
                override_function.c_return_type = existing.c_return_type.clone();
            }
            if override_function.c_param_types.is_empty() {
                override_function.c_param_types = existing.c_param_types.clone();
            }
            *existing = override_function;
        } else {
            generated.functions.push(override_function);
        }
    }

    Ok(generated)
}

fn find_om_contract_override(alias: &str) -> Option<std::path::PathBuf> {
    [
        Path::new("contracts").join(format!("{alias}.om.snif")),
        Path::new(&format!("{alias}.om.snif")).to_path_buf(),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn inject_features(program: &mut Program, features: BTreeMap<String, SnifFeatureValue>) {
    let mut feature_stmts: Vec<Stmt> = Vec::new();
    for (name, value) in features {
        let literal_val = match value {
            SnifFeatureValue::Bool(b) => LiteralValue::Boolean(b),
            SnifFeatureValue::String(s) => LiteralValue::String(s.clone()),
            SnifFeatureValue::Number(n) => LiteralValue::Number(n),
        };
        let loc = Location { line: 1, column: 1 };
        let span = loc.to_span();
        let const_decl = ConstDecl {
            name,
            var_type: None,
            value: Expr::with_span(ExprKind::Literal(literal_val), loc.clone(), span.clone()),
        };
        feature_stmts.push(Stmt::with_span(
            StmtKind::ConstDeclaration(const_decl),
            loc,
            span,
        ));
    }
    program.splice(0..0, feature_stmts);
}

fn validate_entrypoint(program: &Program) -> Result<(), String> {
    for stmt in program {
        if let StmtKind::ClassDeclaration(class) = &stmt.kind {
            if class.name == "main" {
                if class.methods.is_empty() {
                    return Err(
                        "Error: `class main` must declare at least one method. `start()` is preferred, but Snask will also accept the first method as the entrypoint.".to_string(),
                    );
                }
                return Ok(());
            }
        }
    }

    Err(
        "Error: every Snask program must contain a `class main` with at least one method. `start()` is preferred, but Snask will also accept the first method as the entrypoint.".to_string(),
    )
}

fn expand_inheritance(program: &mut Program) -> Result<(), String> {
    let mut classes: std::collections::HashMap<String, crate::ast::ClassDecl> =
        std::collections::HashMap::new();
    for stmt in program.iter() {
        if let StmtKind::ClassDeclaration(class) = &stmt.kind {
            classes.insert(class.name.clone(), class.clone());
        }
    }

    let class_names: Vec<String> = classes.keys().cloned().collect();
    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 100 {
        changed = false;
        iterations += 1;
        for name in &class_names {
            let mut current = classes.get(name).unwrap().clone();
            if let Some(parent_name) = &current.parent {
                let parent = classes.get(parent_name).ok_or_else(|| {
                    format!(
                        "Parent class '{}' not found for class '{}'.",
                        parent_name, name
                    )
                })?;

                let mut modified = false;
                for p_prop in &parent.properties {
                    if !current.properties.iter().any(|p| p.name == p_prop.name) {
                        current.properties.push(p_prop.clone());
                        modified = true;
                    }
                }
                for p_meth in &parent.methods {
                    if !current.methods.iter().any(|m| m.name == p_meth.name) {
                        current.methods.push(p_meth.clone());
                        modified = true;
                    }
                }

                if modified {
                    classes.insert(name.clone(), current);
                    changed = true;
                }
            }
        }
    }

    if iterations >= 100 {
        return Err("Circular inheritance or too deep hierarchy detected (max 100).".to_string());
    }

    for stmt in program.iter_mut() {
        if let StmtKind::ClassDeclaration(class) = &mut stmt.kind {
            if let Some(expanded) = classes.get(&class.name) {
                *class = expanded.clone();
            }
        }
    }
    Ok(())
}

fn uses_full_runtime(program: &[Stmt]) -> bool {
    fn is_heavy_module(name: &str) -> bool {
        matches!(
            name,
            "gui" | "sqlite" | "zlib" | "snask_skia" | "blaze" | "blaze_auth" | "auth"
        )
    }
    for st in program {
        match &st.kind {
            StmtKind::Import(m) => {
                if is_heavy_module(m) {
                    return true;
                }
            }
            StmtKind::ImportCOm { .. } => return true,
            StmtKind::FromImport { module, .. } => {
                if is_heavy_module(module) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[derive(Debug, Clone)]
struct BaremetalRestriction {
    span: crate::span::Span,
    message: String,
    annotation: String,
    help: String,
    note: Option<String>,
}

fn find_baremetal_restrictions(program: &[Stmt]) -> Vec<BaremetalRestriction> {
    let mut restrictions = Vec::new();
    for stmt in program {
        collect_baremetal_stmt_restrictions(stmt, &mut restrictions);
    }
    restrictions
}

fn collect_baremetal_stmt_restrictions(stmt: &Stmt, restrictions: &mut Vec<BaremetalRestriction>) {
    match &stmt.kind {
        StmtKind::Print(exprs) => {
            restrictions.push(BaremetalRestriction {
                span: stmt.span,
                message: "print requires std runtime".to_string(),
                annotation: "std output is not available in baremetal".to_string(),
                help: "use a serial/VGA driver or build with `--profile humane`".to_string(),
                note: Some(
                    "`baremetal` starts without Snask std/runtime services by default.".to_string(),
                ),
            });
            for expr in exprs {
                collect_baremetal_expr_restrictions(expr, restrictions);
            }
        }
        StmtKind::Import(lib) => {
            if baremetal_runtime_module(lib) {
                restrictions.push(BaremetalRestriction {
                    span: stmt.span,
                    message: format!("import `{lib}` requires std runtime"),
                    annotation: "runtime-backed import".to_string(),
                    help: "use a baremetal driver/module, or build with `--profile humane`"
                        .to_string(),
                    note: Some(
                        "Baremetal code cannot assume OS, libc, GUI or filesystem services."
                            .to_string(),
                    ),
                });
            }
        }
        StmtKind::ImportCOm { alias, .. } => restrictions.push(BaremetalRestriction {
            span: stmt.span,
            message: format!("import_c_om `{alias}` requires std runtime"),
            annotation: "OM C interop needs the native runtime/link pipeline".to_string(),
            help: "use `--profile systems` for native C interop while baremetal support matures"
                .to_string(),
            note: None,
        }),
        StmtKind::Expression(expr)
        | StmtKind::FuncCall(expr)
        | StmtKind::Return(expr)
        | StmtKind::VarDeclaration(VarDecl { value: expr, .. })
        | StmtKind::MutDeclaration(MutDecl { value: expr, .. })
        | StmtKind::ConstDeclaration(ConstDecl { value: expr, .. })
        | StmtKind::VarAssignment(VarSet { value: expr, .. })
        | StmtKind::ListDeclaration(ListDecl { value: expr, .. })
        | StmtKind::ListPush(ListPush { value: expr, .. }) => {
            collect_baremetal_expr_restrictions(expr, restrictions);
        }
        StmtKind::PropertyAssignment(PropertyAssignment { target, value, .. }) => {
            collect_baremetal_expr_restrictions(target, restrictions);
            collect_baremetal_expr_restrictions(value, restrictions);
        }
        StmtKind::IndexAssignment(IndexAssignment {
            target,
            index,
            value,
            ..
        }) => {
            collect_baremetal_expr_restrictions(target, restrictions);
            collect_baremetal_expr_restrictions(index, restrictions);
            collect_baremetal_expr_restrictions(value, restrictions);
        }
        StmtKind::DictDeclaration(DictDecl { value, .. }) => {
            collect_baremetal_expr_restrictions(value, restrictions);
        }
        StmtKind::DictSet(DictSet { key, value, .. }) => {
            collect_baremetal_expr_restrictions(key, restrictions);
            collect_baremetal_expr_restrictions(value, restrictions);
        }
        StmtKind::FuncDeclaration(func) => {
            for child in &func.body {
                collect_baremetal_stmt_restrictions(child, restrictions);
            }
        }
        StmtKind::ClassDeclaration(class) => {
            for property in &class.properties {
                collect_baremetal_expr_restrictions(&property.value, restrictions);
            }
            for method in &class.methods {
                for child in &method.body {
                    collect_baremetal_stmt_restrictions(child, restrictions);
                }
            }
        }
        StmtKind::Conditional(cond) => {
            collect_baremetal_expr_restrictions(&cond.if_block.condition, restrictions);
            for child in &cond.if_block.body {
                collect_baremetal_stmt_restrictions(child, restrictions);
            }
            for elif in &cond.elif_blocks {
                collect_baremetal_expr_restrictions(&elif.condition, restrictions);
                for child in &elif.body {
                    collect_baremetal_stmt_restrictions(child, restrictions);
                }
            }
            if let Some(else_body) = &cond.else_block {
                for child in else_body {
                    collect_baremetal_stmt_restrictions(child, restrictions);
                }
            }
        }
        StmtKind::Loop(loop_stmt) => match loop_stmt {
            LoopStmt::While { condition, body } => {
                collect_baremetal_expr_restrictions(condition, restrictions);
                for child in body {
                    collect_baremetal_stmt_restrictions(child, restrictions);
                }
            }
            LoopStmt::For { iterable, body, .. } => {
                collect_baremetal_expr_restrictions(iterable, restrictions);
                for child in body {
                    collect_baremetal_stmt_restrictions(child, restrictions);
                }
            }
        },
        StmtKind::UnsafeBlock(body)
        | StmtKind::Scope { body, .. }
        | StmtKind::Zone { body, .. } => {
            for child in body {
                collect_baremetal_stmt_restrictions(child, restrictions);
            }
        }
        StmtKind::Input { .. } => restrictions.push(BaremetalRestriction {
            span: stmt.span,
            message: "input requires std runtime".to_string(),
            annotation: "stdin is not available in baremetal".to_string(),
            help: "read from a device driver, serial port, or build with `--profile humane`"
                .to_string(),
            note: None,
        }),
        StmtKind::FromImport { module, .. } => {
            if baremetal_runtime_module(module) {
                restrictions.push(BaremetalRestriction {
                    span: stmt.span,
                    message: format!("import `{module}` requires std runtime"),
                    annotation: "runtime-backed import".to_string(),
                    help: "use a baremetal driver/module, or build with `--profile humane`"
                        .to_string(),
                    note: None,
                });
            }
        }
        StmtKind::Promote { .. } | StmtKind::Entangle { .. } => {}
    }
}

fn collect_baremetal_expr_restrictions(expr: &Expr, restrictions: &mut Vec<BaremetalRestriction>) {
    match &expr.kind {
        ExprKind::FunctionCall { callee, args } => {
            if let Some(name) = baremetal_runtime_call_name(callee) {
                restrictions.push(BaremetalRestriction {
                    span: expr.span,
                    message: format!("{name} requires std runtime"),
                    annotation: "runtime-backed call".to_string(),
                    help: "use a baremetal intrinsic/driver or build with `--profile humane`"
                        .to_string(),
                    note: Some(
                        "`systems` keeps the normal runtime while exposing low-level features."
                            .to_string(),
                    ),
                });
            }
            collect_baremetal_expr_restrictions(callee, restrictions);
            for arg in args {
                collect_baremetal_expr_restrictions(arg, restrictions);
            }
        }
        ExprKind::Unary { expr, .. } => collect_baremetal_expr_restrictions(expr, restrictions),
        ExprKind::Binary { left, right, .. } => {
            collect_baremetal_expr_restrictions(left, restrictions);
            collect_baremetal_expr_restrictions(right, restrictions);
        }
        ExprKind::PropertyAccess { target, .. } => {
            collect_baremetal_expr_restrictions(target, restrictions);
        }
        ExprKind::IndexAccess { target, index } => {
            collect_baremetal_expr_restrictions(target, restrictions);
            collect_baremetal_expr_restrictions(index, restrictions);
        }
        ExprKind::New { args, .. } => {
            for arg in args {
                collect_baremetal_expr_restrictions(arg, restrictions);
            }
        }
        ExprKind::Literal(LiteralValue::List(items)) => {
            for item in items {
                collect_baremetal_expr_restrictions(item, restrictions);
            }
        }
        ExprKind::Literal(LiteralValue::Dict(pairs)) => {
            for (key, value) in pairs {
                collect_baremetal_expr_restrictions(key, restrictions);
                collect_baremetal_expr_restrictions(value, restrictions);
            }
        }
        ExprKind::Literal(_) | ExprKind::Variable(_) => {}
    }
}

fn baremetal_runtime_call_name(callee: &Expr) -> Option<String> {
    match &callee.kind {
        ExprKind::Variable(name) if baremetal_runtime_builtin(name) => Some(name.clone()),
        ExprKind::PropertyAccess { target, property } => {
            if let ExprKind::Variable(module) = &target.kind {
                if baremetal_runtime_module(module) {
                    return Some(format!("{module}.{property}"));
                }
            }
            None
        }
        _ => None,
    }
}

fn baremetal_runtime_builtin(name: &str) -> bool {
    matches!(
        name,
        "read_file"
            | "write_file"
            | "append_file"
            | "exists"
            | "delete"
            | "read_dir"
            | "is_file"
            | "is_dir"
            | "create_dir"
            | "http_get"
            | "http_post"
            | "time"
            | "sleep"
            | "exit"
            | "args"
            | "env"
            | "set_env"
            | "cwd"
            | "platform"
            | "arch"
            | "json_parse"
            | "json_stringify"
            | "json_stringify_pretty"
            | "json_get"
            | "json_has"
    )
}

fn baremetal_runtime_module(name: &str) -> bool {
    matches!(
        name,
        "gui" | "sqlite" | "zlib" | "snask_skia" | "skia" | "json" | "http" | "fs" | "os" | "stdio"
    )
}

fn render_baremetal_restrictions(
    filename: &str,
    source: &str,
    restrictions: &[BaremetalRestriction],
) -> String {
    let mut bag = DiagnosticBag::new();
    for restriction in restrictions.iter().take(5) {
        let mut diagnostic = Diagnostic::error(restriction.message.clone())
            .with_code("S8001".to_string())
            .with_annotation(Annotation::primary(
                restriction.span,
                restriction.annotation.clone(),
            ))
            .with_help(restriction.help.clone());
        if let Some(note) = &restriction.note {
            diagnostic = diagnostic.with_note(note.clone());
        }
        bag.add(diagnostic);
    }

    let mut rendered = bag.render_all(filename, source);
    if restrictions.len() > 5 {
        rendered.push_str(&format!(
            "\nnote: {} more baremetal restriction(s) were hidden. Fix the first error and run the compiler again.\n",
            restrictions.len() - 5
        ));
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::{
        find_baremetal_restrictions, render_baremetal_restrictions, render_parser_diagnostics,
        render_semantic_diagnostics, validate_entrypoint,
    };
    use crate::ast::{ClassDecl, Location, Stmt, StmtKind};
    use crate::parser::Parser;
    use crate::semantic_analyzer::{SemanticError, SemanticErrorKind};
    use crate::span::{Position, Span};

    fn loc() -> Location {
        Location { line: 1, column: 1 }
    }

    fn span() -> Span {
        loc().to_span()
    }

    #[test]
    fn validate_entrypoint_rejects_missing_main_class() {
        let program = Vec::new();
        let err = validate_entrypoint(&program).expect_err("missing main must fail");
        assert!(err.contains("class main"));
        assert!(err.contains("at least one method"));
    }

    #[test]
    fn validate_entrypoint_rejects_empty_main_class() {
        let program = vec![Stmt::with_span(
            StmtKind::ClassDeclaration(ClassDecl {
                name: "main".to_string(),
                parent: None,
                properties: Vec::new(),
                methods: Vec::new(),
            }),
            loc(),
            span(),
        )];

        let err = validate_entrypoint(&program).expect_err("empty main must fail");
        assert!(err.contains("at least one method"));
    }

    #[test]
    fn humane_parser_diagnostic_points_to_missing_paren() {
        let source = "class main\n    fun start()\n        print(\"Hello\"\n";
        let mut parser = Parser::new(source).expect("source should tokenize");
        let (_program, errors) = parser.parse_program_recovering(10);
        let rendered = render_parser_diagnostics("hello.snask", source, &errors);

        assert!(rendered.contains("error[S1002]: missing closing `)`"));
        assert!(rendered.contains("print(\"Hello\""));
        assert!(rendered.contains("^ expected `)` here"));
        assert!(!rendered.contains("ParseError"));
    }

    #[test]
    fn humane_semantic_diagnostic_uses_snippet_and_suggestion() {
        let source =
            "class main\n    fun start()\n        let message = \"Hello\"\n        print(mesage)\n";
        let span = Span::new(Position::new(4, 15, 0), Position::new(4, 21, 0));
        let error = SemanticError::new(
            SemanticErrorKind::VariableNotFound("mesage".to_string()),
            span,
        )
        .with_help("Did you mean 'message'?".to_string());

        let rendered = render_semantic_diagnostics("name.snask", source, &[error]);

        assert!(rendered.contains("error[S2002]: variable `mesage` was not found"));
        assert!(rendered.contains("print(mesage)"));
        assert!(rendered.contains("^^^^^^ unknown name"));
        assert!(rendered.contains("help: Did you mean 'message'?"));
        assert!(!rendered.contains("SemanticError"));
    }

    #[test]
    fn humane_semantic_type_mismatch_uses_public_type_names() {
        let source = "class main\n    fun start()\n        let age: int = \"18\"\n";
        let span = Span::new(Position::new(3, 24, 0), Position::new(3, 28, 0));
        let error = SemanticError::new(
            SemanticErrorKind::TypeMismatch {
                expected: crate::types::Type::Int,
                found: crate::types::Type::String,
            },
            span,
        );

        let rendered = render_semantic_diagnostics("types.snask", source, &[error]);

        assert!(rendered.contains("error[S2010]: expected `int`, found `str`"));
        assert!(rendered.contains("type mismatch here"));
        assert!(!rendered.contains("String"));
    }

    #[test]
    fn baremetal_diagnostic_explains_std_runtime_requirement() {
        let source = "class main\n    fun start()\n        print(\"Hello\")\n";
        let mut parser = Parser::new(source).expect("source should tokenize");
        let program = parser.parse_program().expect("source should parse");
        let restrictions = find_baremetal_restrictions(&program);
        let rendered = render_baremetal_restrictions("kernel.snask", source, &restrictions);

        assert!(rendered.contains("error[S8001]: print requires std runtime"));
        assert!(rendered.contains("print(\"Hello\")"));
        assert!(rendered.contains("std output is not available in baremetal"));
        assert!(rendered.contains("help: use a serial/VGA driver or build with `--profile humane`"));
    }
}

pub fn link_binary(
    file_path: &str,
    ir: Vec<u8>,
    options: BuildOptions,
    link_tiny_runtime: bool,
    pb: &ProgressBar,
    extra_pkgs: Vec<String>,
) -> Result<(), String> {
    let ir_file = "temp_snask.ll";
    fs::write(ir_file, ir).map_err(|e| e.to_string())?;

    let size_link = options.release_size || options.tiny || options.extreme;
    let clang_opt = if let Some(o) = options.opt_override.as_deref() {
        format!("-{}", o)
    } else if options.extreme || options.tiny {
        "-Oz".to_string()
    } else if options.release_size {
        "-Os".to_string()
    } else {
        format!("-O{}", options.opt_level)
    };

    let mut extra_libs = Vec::new();
    for pkg in extra_pkgs {
        if let Ok(libs) = get_pkg_libs(&pkg) {
            extra_libs.extend(libs);
        }
    }

    let have_lld = size_link
        && Command::new("ld.lld")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    let extreme_obj = if options.extreme && options.target.is_none() && cfg!(target_os = "linux") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Some(format!("{}/.snask/lib/rt_extreme.o", home))
    } else {
        None
    };

    let final_output = options
        .output_name
        .unwrap_or_else(|| file_path.replace(".snask", ""));

    if options.lto {
        pb.set_message(format!("Linking (clang-18 {} +LTO)", clang_opt));
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let runtime_path = if let Some(t) = &options.target {
            if options.tiny {
                format!("{}/.snask/lib/{}/runtime_nano.bc", home, t)
            } else if link_tiny_runtime {
                format!("{}/.snask/lib/{}/runtime_tiny.bc", home, t)
            } else {
                format!("{}/.snask/lib/{}/runtime.bc", home, t)
            }
        } else if options.tiny {
            format!("{}/.snask/lib/runtime_nano.bc", home)
        } else if link_tiny_runtime {
            format!("{}/.snask/lib/runtime_tiny.bc", home)
        } else {
            format!("{}/.snask/lib/runtime.bc", home)
        };

        let mut clang = Command::new("clang-18");
        clang.arg(&clang_opt).arg("-flto=thin");
        if have_lld {
            clang.arg("-fuse-ld=lld");
        }
        if extreme_obj.is_some() {
            clang.arg("-nostdlib").arg("-static");
        }
        if let Some(t) = &options.target {
            clang.arg(format!("--target={}", t));
        }
        if let Some(p) = &extreme_obj {
            clang.arg(p);
        }

        let lib_snask = format!("{}/.snask/lib/libsnask.a", home);

        let mut args = vec![ir_file.to_string()];
        if options.extreme {
            // extreme doesn't need runtime bc
        } else if options.tiny {
            args.push(lib_snask);
        } else {
            args.push(runtime_path);
        }

        let status = clang
            .args(&args)
            .arg("-o")
            .arg(&final_output)
            .args(&extra_libs)
            .args(if options.extreme {
                vec![]
            } else if options.tiny {
                vec!["-lc".to_string(), "-lgcc".to_string()]
            } else {
                vec!["-ldl".to_string()]
            })
            .args(if link_tiny_runtime {
                vec![]
            } else {
                vec!["-lm".to_string()]
            })
            .args(get_link_flags(size_link, have_lld))
            .args(get_runtime_linkargs_for(
                options.target.as_deref(),
                link_tiny_runtime,
            ))
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Final link step failed (LTO path).".to_string());
        }
        if std::env::var("SNASK_KEEP_TEMPS").ok().as_deref() != Some("1") {
            fs::remove_file(ir_file).ok();
        }
    } else {
        let obj_file = "temp_snask.o";
        pb.set_message(format!("Compiling (llc-18 -O{})", options.opt_level));
        let mut llc = Command::new("llc-18");
        llc.arg(format!("-O{}", options.opt_level))
            .arg("-relocation-model=pic")
            .arg("-filetype=obj");
        if let Some(t) = &options.target {
            llc.arg(format!("-mtriple={}", t));
        }
        llc.arg(ir_file)
            .arg("-o")
            .arg(obj_file)
            .status()
            .map_err(|e| e.to_string())?;

        pb.set_message("Linking (clang-18)");
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

        let mut runtime_path = if let Some(t) = &options.target {
            if options.tiny {
                format!("{}/.snask/lib/{}/runtime_nano.o", home, t)
            } else if link_tiny_runtime {
                format!("{}/.snask/lib/{}/runtime_tiny.o", home, t)
            } else {
                format!("{}/.snask/lib/{}/runtime.o", home, t)
            }
        } else if options.tiny {
            format!("{}/.snask/lib/runtime_nano.o", home)
        } else if link_tiny_runtime {
            format!("{}/.snask/lib/runtime_tiny.o", home)
        } else {
            format!("{}/.snask/lib/runtime.o", home)
        };

        // APT Support: Check for global installation paths
        let global_runtime = "/usr/lib/snask/runtime/runtime.o";
        if !std::path::Path::new(&runtime_path).exists()
            && std::path::Path::new(global_runtime).exists()
        {
            runtime_path = global_runtime.to_string();
        }

        let mut clang = Command::new("clang-18");
        clang.arg(&clang_opt);
        if have_lld {
            clang.arg("-fuse-ld=lld");
        }
        if extreme_obj.is_some() {
            clang.arg("-nostdlib").arg("-static");
        }
        if let Some(t) = &options.target {
            clang.arg(format!("--target={}", t));
        }
        if let Some(p) = &extreme_obj {
            clang.arg(p);
        }

        let mut lib_snask = format!("{}/.snask/lib/libsnask.a", home);
        let global_lib = "/usr/lib/snask/libsnask.a";
        if !std::path::Path::new(&lib_snask).exists() && std::path::Path::new(global_lib).exists() {
            lib_snask = global_lib.to_string();
        }

        let mut args = vec![obj_file.to_string()];
        if options.extreme {
        } else if options.tiny {
            args.push(lib_snask);
        } else {
            args.push(runtime_path);
        }

        let status = clang
            .args(&args)
            .arg("-o")
            .arg(&final_output)
            .args(&extra_libs)
            .args(if options.extreme {
                vec![]
            } else if options.tiny {
                vec!["-lc".to_string(), "-lgcc".to_string()]
            } else {
                vec!["-ldl".to_string()]
            })
            .args(if link_tiny_runtime {
                vec![]
            } else {
                vec!["-lm".to_string()]
            })
            .args(get_link_flags(size_link, have_lld))
            .args(get_runtime_linkargs_for(
                options.target.as_deref(),
                link_tiny_runtime,
            ))
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Final link step failed.".to_string());
        }
        if std::env::var("SNASK_KEEP_TEMPS").ok().as_deref() != Some("1") {
            fs::remove_file(ir_file).ok();
            fs::remove_file(obj_file).ok();
        }
    }

    if options.strip && options.target.is_none() {
        strip_binary(&final_output);
    }

    Ok(())
}

fn get_link_flags(size_link: bool, have_lld: bool) -> Vec<String> {
    if size_link {
        let mut v = vec![
            "-Wl,--gc-sections".to_string(),
            "-Wl,--as-needed".to_string(),
            "-Wl,--build-id=none".to_string(),
            "-Wl,-O1".to_string(),
        ];
        if have_lld {
            v.push("-Wl,--icf=all".to_string());
        }
        v
    } else {
        vec!["-Wl,--export-dynamic".to_string(), "-rdynamic".to_string()]
    }
}

pub fn get_runtime_linkargs_for(target: Option<&str>, tiny: bool) -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let p = if let Some(t) = target {
        if tiny {
            format!("{}/.snask/lib/{}/runtime_tiny.linkargs", home, t)
        } else {
            format!("{}/.snask/lib/{}/runtime.linkargs", home, t)
        }
    } else {
        if tiny {
            format!("{}/.snask/lib/runtime_tiny.linkargs", home)
        } else {
            format!("{}/.snask/lib/runtime.linkargs", home)
        }
    };
    if let Ok(s) = std::fs::read_to_string(&p) {
        let mut args: Vec<String> = s.split_whitespace().map(|x| x.to_string()).collect();
        for arg in fallback_runtime_linkargs() {
            if !args.contains(&arg) {
                args.push(arg);
            }
        }
        return args;
    }
    fallback_runtime_linkargs()
}

fn fallback_runtime_linkargs() -> Vec<String> {
    let mut args = Vec::new();
    for pkg in ["gtk+-3.0", "sqlite3", "zlib"] {
        let out = Command::new("pkg-config").args(["--libs", pkg]).output();
        let Ok(out) = out else { continue };
        if !out.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        for token in text.split_whitespace() {
            let token = token.to_string();
            if !args.contains(&token) {
                args.push(token);
            }
        }
    }
    if !args.iter().any(|arg| arg == "-lz") {
        args.push("-lz".to_string());
    }
    args
}

fn strip_binary(path: &str) {
    let strip_tool = if Command::new("llvm-strip-18")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "llvm-strip-18"
    } else if Command::new("strip").arg("--version").output().is_ok() {
        "strip"
    } else {
        ""
    };
    if !strip_tool.is_empty() {
        let _ = Command::new(strip_tool).arg(path).status();
    }
}

pub fn resolve_imports(
    program: &mut Program,
    entry_dir: &Path,
    resolved_program: &mut Program,
    resolved_modules: &mut HashSet<String>,
) -> Result<(), String> {
    for stmt in program.drain(..) {
        match stmt.kind {
            StmtKind::Import(ref module_name) => {
                if is_native_module(module_name) {
                    resolved_program.push(stmt);
                    continue;
                }
                match resolve_module_path(entry_dir, module_name) {
                    Ok(module_path) => {
                        if !resolved_modules.contains(&module_path) {
                            resolved_modules.insert(module_path.clone());
                            let source = fs::read_to_string(&module_path).map_err(|e| {
                                format!("Failed to read module {}: {}", module_name, e)
                            })?;
                            let mut parser = Parser::new(&source)
                                .map_err(|e| render_parser_diagnostic(&module_path, &source, &e))?;
                            let (module_program, errors) = parser.parse_program_recovering(10);
                            if !errors.is_empty() {
                                return Err(render_parser_diagnostics(
                                    &module_path,
                                    &source,
                                    &errors,
                                ));
                            }
                            if let Some(mut prog) = module_program {
                                let prefix = module_name.split('/').last().unwrap_or(module_name);
                                if prefix != "prelude" {
                                    for s in &mut prog {
                                        match &mut s.kind {
                                            StmtKind::FuncDeclaration(f) => {
                                                f.name = format!("{}::{}", prefix, f.name)
                                            }
                                            StmtKind::ClassDeclaration(c) => {
                                                c.name = format!("{}::{}", prefix, c.name)
                                            }
                                            StmtKind::VarDeclaration(v) => {
                                                v.name = format!("{}::{}", prefix, v.name)
                                            }
                                            StmtKind::MutDeclaration(v) => {
                                                v.name = format!("{}::{}", prefix, v.name)
                                            }
                                            StmtKind::ConstDeclaration(v) => {
                                                v.name = format!("{}::{}", prefix, v.name)
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                resolve_imports(
                                    &mut prog,
                                    Path::new(&module_path).parent().unwrap(),
                                    resolved_program,
                                    resolved_modules,
                                )?;
                            }
                        }
                    }
                    Err(e) => {
                        // If not found as .snask, check if it's a C package
                        let pkg = resolve_pkg_name(module_name);
                        if has_pkg(&pkg) {
                            resolved_program.push(stmt);
                            continue;
                        }
                        return Err(e);
                    }
                }
            }
            StmtKind::ImportCOm { .. } => resolved_program.push(stmt),
            _ => resolved_program.push(stmt),
        }
    }
    Ok(())
}

fn resolve_module_path(entry_dir: &Path, module_name: &str) -> Result<String, String> {
    let name_with_ext = if module_name.ends_with(".snask") {
        module_name.to_string()
    } else {
        format!("{}.snask", module_name)
    };

    // 1. Local path
    let local = entry_dir.join(&name_with_ext);
    if local.exists() {
        return Ok(local.to_string_lossy().to_string());
    }

    // Try raw module_name if it's a direct path
    let raw = entry_dir.join(module_name);
    if raw.exists() {
        return Ok(raw.to_string_lossy().to_string());
    }

    // 2. Stdlib / Packages (MVP: check both direct and nested structure)
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let pkg_direct = Path::new(&home)
        .join(".snask/packages")
        .join(&name_with_ext);
    if pkg_direct.exists() {
        return Ok(pkg_direct.to_string_lossy().to_string());
    }
    let pkg_nested = Path::new(&home)
        .join(".snask/packages")
        .join(module_name)
        .join(&name_with_ext);
    if pkg_nested.exists() {
        return Ok(pkg_nested.to_string_lossy().to_string());
    }
    Err(format!("Module '{}' not found.", module_name))
}

pub fn render_parser_diagnostic(filename: &str, source: &str, err: &ParseError) -> String {
    render_parser_diagnostics(filename, source, std::slice::from_ref(err))
}

pub fn render_parser_diagnostics(filename: &str, source: &str, errs: &[ParseError]) -> String {
    let mut bag = DiagnosticBag::new();
    let shown = errs.len().min(3);
    for err in errs.iter().take(shown) {
        let mut diagnostic = Diagnostic::error(parser_message(err))
            .with_code(humane_code(err.code).to_string())
            .with_annotation(Annotation::primary(
                err.span,
                parser_annotation(err).to_string(),
            ));
        for note in &err.notes {
            diagnostic = diagnostic.with_note(note.clone());
        }
        if let Some(help) = &err.help {
            diagnostic = diagnostic.with_help(help.clone());
        }
        bag.add(diagnostic);
    }
    let mut rendered = bag.render_all(filename, source);
    if errs.len() > shown {
        rendered.push_str(&format!(
            "\nnote: {} more parse error(s) were hidden. Fix the first error and run the compiler again.\n",
            errs.len() - shown
        ));
    }
    rendered
}

pub fn render_semantic_diagnostics(
    filename: &str,
    source: &str,
    errors: &[SemanticError],
) -> String {
    let mut bag = DiagnosticBag::new();
    let shown = errors.len().min(5);
    for error in errors.iter().take(shown) {
        let mut diagnostic = Diagnostic::error(error.message())
            .with_code(humane_code(error.code()).to_string())
            .with_annotation(Annotation::primary(
                error.span,
                semantic_annotation(error).to_string(),
            ));
        for note in &error.notes {
            diagnostic = diagnostic.with_note(note.clone());
        }
        if let Some(help) = &error.help {
            diagnostic = diagnostic.with_help(help.clone());
        }
        bag.add(diagnostic);
    }
    let mut rendered = bag.render_all(filename, source);
    if errors.len() > shown {
        rendered.push_str(&format!(
            "\nnote: {} more semantic error(s) were hidden. Fix the first error and run the compiler again.\n",
            errors.len() - shown
        ));
    }
    rendered
}

fn parser_message(err: &ParseError) -> String {
    match err.code {
        "SNASK-PARSE-MISSING-RPAREN" => "missing closing `)`".to_string(),
        "SNASK-PARSE-MISSING-RBRACKET" => "missing closing `]`".to_string(),
        "SNASK-PARSE-MISSING-RBRACE" => "missing closing `}`".to_string(),
        "SNASK-PARSE-INDENT" => "expected an indented block".to_string(),
        "SNASK-PARSE-SEMICOLON" => "missing statement terminator".to_string(),
        "SNASK-PARSE-EXPR" => "expected an expression".to_string(),
        _ => err.message.clone(),
    }
}

fn parser_annotation(err: &ParseError) -> &'static str {
    match err.code {
        "SNASK-PARSE-MISSING-RPAREN" => "expected `)` here",
        "SNASK-PARSE-MISSING-RBRACKET" => "expected `]` here",
        "SNASK-PARSE-MISSING-RBRACE" => "expected `}` here",
        "SNASK-PARSE-INDENT" => "this block needs indentation",
        "SNASK-PARSE-SEMICOLON" => "statement ends here",
        "SNASK-PARSE-EXPR" => "expression should start here",
        _ => "problem starts here",
    }
}

fn semantic_annotation(error: &SemanticError) -> &'static str {
    use crate::semantic_analyzer::SemanticErrorKind::*;
    match &error.kind {
        VariableAlreadyDeclared(_) => "already declared here",
        VariableNotFound(_) => "unknown name",
        FunctionAlreadyDeclared(_) => "already declared here",
        FunctionNotFound(_) => "unknown function",
        UnknownType(_) => "unknown type",
        MissingReturn { .. } => "function may exit here without returning",
        TypeMismatch { .. } => "type mismatch here",
        InvalidOperation { .. } => "invalid operation",
        ImmutableAssignment(_) => "cannot assign to immutable binding",
        ReturnOutsideFunction => "`return` is only valid inside a function",
        WrongNumberOfArguments { .. } => "wrong number of arguments",
        IndexAccessOnNonIndexable(_) => "cannot index this value",
        InvalidIndexType(_) => "invalid index type",
        PropertyNotFound(_) => "unknown property",
        NotCallable(_) => "this value is not callable",
        RestrictedNativeFunction { .. } => "reserved native function",
        TinyDisallowedLib { .. } => "not available in tiny mode",
    }
}

#[cfg(test)]
mod build_profile_tests {
    use super::BuildProfile;

    #[test]
    fn parses_language_profiles() {
        assert_eq!(BuildProfile::parse("humane"), Some(BuildProfile::Humane));
        assert_eq!(BuildProfile::parse("systems"), Some(BuildProfile::Systems));
        assert_eq!(
            BuildProfile::parse("baremetal"),
            Some(BuildProfile::Baremetal)
        );
    }

    #[test]
    fn keeps_legacy_build_profiles_as_humane_surface() {
        for profile in [
            "default",
            "dev",
            "release",
            "release-size",
            "tiny",
            "extreme",
        ] {
            assert_eq!(BuildProfile::parse(profile), Some(BuildProfile::Humane));
        }
        assert_eq!(BuildProfile::parse("unknown"), None);
    }
}
