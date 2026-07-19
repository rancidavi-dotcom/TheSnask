use indicatif::{ProgressBar, ProgressStyle};
use inkwell::context::Context;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ast::{
    ConditionalStmt, ConstDecl, Expr, ExprKind, FuncDecl, IfBlock, IndexAssignment,
    LiteralValue, Location, LoopStmt, MutDecl, Program, Stmt, StmtKind, VarDecl, VarSet,
};
use crate::diagnostics::{humane_code, Annotation, Diagnostic, DiagnosticBag};
use crate::llvm_generator::LLVMGenerator;
use crate::parser::{ParseError, Parser};
use crate::semantic_analyzer::{SemanticAnalyzer, SemanticError};
use crate::sps::SnifFeatureValue;
use crate::toolchain;
use crate::types::Type;

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
    validate_entrypoint(&program, &options)?;

    let resolved_program = program;
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
    }

    let link_tiny_runtime = options.tiny || options.min_runtime;

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
    let mut generator = LLVMGenerator::new(&context, file_path, options.profile == BuildProfile::Baremetal);
    let ir = generator.generate(resolved_program.clone())?;
    pb.inc(1);

    link_binary(
        file_path,
        ir.into_bytes(),
        options,
        link_tiny_runtime,
        &pb,
    )?;

    pb.finish_with_message("OK");
    Ok(())
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

fn validate_entrypoint(_program: &Program, _options: &BuildOptions) -> Result<(), String> {
    Ok(())
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
        StmtKind::Expression(expr)
        | StmtKind::FuncCall(expr)
        | StmtKind::Return(expr)
        | StmtKind::VarDeclaration(VarDecl { value: expr, .. })
        | StmtKind::MutDeclaration(MutDecl { value: expr, .. })
        | StmtKind::ConstDeclaration(ConstDecl { value: expr, .. })
        | StmtKind::VarAssignment(VarSet { value: expr, .. }) => {
            collect_baremetal_expr_restrictions(expr, restrictions);
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
        StmtKind::FuncDeclaration(func) => {
            for child in &func.body {
                collect_baremetal_stmt_restrictions(child, restrictions);
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
        StmtKind::UnsafeBlock(body) => {
            for child in body {
                collect_baremetal_stmt_restrictions(child, restrictions);
            }
        }
        StmtKind::Asm(_) => {}
        StmtKind::WritePtr { .. } => {}
        StmtKind::Outb { .. } => {}
        StmtKind::GlobalAsm(_) => {}
        StmtKind::StructDeclaration(_) => {}
        StmtKind::Fence { .. } => {}
        StmtKind::AtomicRmw { .. } => {}
        StmtKind::VolatileStore { .. } => {}
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
        ExprKind::IndexAccess { target, index } => {
            collect_baremetal_expr_restrictions(target, restrictions);
            collect_baremetal_expr_restrictions(index, restrictions);
        }
        ExprKind::Literal(_) | ExprKind::Variable(_) => {}
        ExprKind::Deref { ptr, .. } => {
            collect_baremetal_expr_restrictions(ptr, restrictions);
        }
        ExprKind::Inb(_) => {}
        ExprKind::AddrOf(_) => {}
        ExprKind::SizeOf(_) => {}
        ExprKind::AlignOf(_) => {}
        ExprKind::OffsetOf { .. } => {}
        ExprKind::VolatileLoad { .. } => {}
        ExprKind::VolatileStore { .. } => {}
        ExprKind::IntToPtr { .. } => {}
        ExprKind::PtrToInt { .. } => {}
    }
}

fn baremetal_runtime_call_name(callee: &Expr) -> Option<String> {
    match &callee.kind {
        ExprKind::Variable(name) if baremetal_runtime_builtin(name) => Some(name.clone()),
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
        render_baremetal_restrictions, render_parser_diagnostics, render_semantic_diagnostics,
        validate_entrypoint, BuildOptions,
    };
    use crate::ast::{Location, Stmt, StmtKind};
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
    fn validate_entrypoint_always_succeeds_for_baremetal() {
        let program = Vec::new();
        validate_entrypoint(&program, &BuildOptions::default()).expect("should always succeed");
    }

    #[test]
    fn humane_parser_diagnostic_points_to_missing_paren() {
        let source = "fun start()\n    let x = (1 + 2\n";
        let mut parser = Parser::new(source).expect("source should tokenize");
        let (_program, errors) = parser.parse_program_recovering(10);
        let rendered = render_parser_diagnostics("hello.snask", source, &errors);

        assert!(rendered.contains("error[S1002]: missing closing `)`"));
        assert!(rendered.contains("^ expected `)` here"));
        assert!(!rendered.contains("ParseError"));
    }

    #[test]
    fn humane_semantic_diagnostic_uses_snippet_and_suggestion() {
        let source = "fun start()\n    let message = 42\n    let x = mesage\n";
        let span = Span::new(Position::new(3, 13, 0), Position::new(3, 19, 0));
        let error = SemanticError::new(
            SemanticErrorKind::VariableNotFound("mesage".to_string()),
            span,
        )
        .with_help("Did you mean 'message'?".to_string());

        let rendered = render_semantic_diagnostics("name.snask", source, &[error]);

        assert!(rendered.contains("error[S2002]: variable `mesage` was not found"));
        assert!(rendered.contains("^^^^^^ unknown name"));
        assert!(rendered.contains("help: Did you mean 'message'?"));
        assert!(!rendered.contains("SemanticError"));
    }

    #[test]
    fn humane_semantic_type_mismatch_uses_public_type_names() {
        let source = "fun start()\n    let age: i64 = true\n";
        let span = Span::new(Position::new(2, 22, 0), Position::new(2, 26, 0));
        let error = SemanticError::new(
            SemanticErrorKind::TypeMismatch {
                expected: crate::types::Type::I64,
                found: crate::types::Type::Bool,
            },
            span,
        );

        let rendered = render_semantic_diagnostics("types.snask", source, &[error]);

        assert!(rendered.contains("error[S2010]: expected `i64`, found `bool`"));
        assert!(rendered.contains("type mismatch here"));
    }
}

pub fn link_binary(
    file_path: &str,
    ir: Vec<u8>,
    options: BuildOptions,
    link_tiny_runtime: bool,
    pb: &ProgressBar,
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

    let lld = toolchain::ld_lld();
    let clang_path = toolchain::clang();
    let llc_path = toolchain::llc();
    let has_linker_ld = options.profile == BuildProfile::Baremetal
        && std::path::Path::new("linker.ld").exists();

    let have_lld = size_link
        && lld
            .as_ref()
            .map(|path| {
                Command::new(path)
                    .arg("--version")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            })
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
        pb.set_message(format!(
            "Linking ({} {} +LTO)",
            toolchain::tool_display(&clang_path),
            clang_opt
        ));
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

        if has_linker_ld {
            // baremetal + linker.ld: compile IR to .o with clang -c, then link with ld
            let obj_file = "temp_snask.o";
            let mut compile = Command::new(&clang_path);
            compile.arg(&clang_opt).arg("-c").arg("-flto=thin");
            if have_lld {
                if let Some(path) = &lld {
                    compile.arg(format!("-fuse-ld={}", path.to_string_lossy()));
                }
            }
            compile.arg("-ffreestanding").arg("-nostdlib").arg("-static").arg("-mno-red-zone");
            if let Some(t) = &options.target {
                compile.arg(format!("--target={}", t));
            }
            compile.arg(ir_file).arg("-o").arg(obj_file);
            let status = compile.status().map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Compilation to object file failed (LTO path).".to_string());
            }
            let ld_path = lld.as_ref().map(|p| p.as_path()).unwrap_or_else(|| std::path::Path::new("ld"));
            let mut link = Command::new(ld_path);
            let target_flag = if let Some(t) = &options.target {
                if t.contains("x86_64") { Some("-melf_x86_64") }
                else if t.contains("i386") || t.contains("i686") { Some("-melf_i386") }
                else { None }
            } else { Some("-melf_x86_64") };
            if let Some(emul) = target_flag {
                link.arg(emul);
            }
            let status = link
                .arg("-T").arg("linker.ld")
                .arg(obj_file)
                .arg("-o").arg(&final_output)
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Link step failed (LTO path).".to_string());
            }
        } else {
            let mut clang = Command::new(&clang_path);
            clang.arg(&clang_opt).arg("-flto=thin");
            if have_lld {
                if let Some(path) = &lld {
                    clang.arg(format!("-fuse-ld={}", path.to_string_lossy()));
                }
            }
            if extreme_obj.is_some() {
                clang.arg("-nostdlib").arg("-static");
            }
            if options.profile == BuildProfile::Baremetal {
                clang.arg("-ffreestanding").arg("-nostdlib").arg("-static").arg("-mno-red-zone");
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
            } else if options.profile == BuildProfile::Baremetal {
                // baremetal doesn't need runtime bc
            } else if options.tiny {
                args.push(lib_snask);
            } else {
                args.push(runtime_path);
            }

            let status = clang
                .args(&args)
                .arg("-o")
                .arg(&final_output)
                .args(if options.extreme {
                    vec![]
                } else if options.profile == BuildProfile::Baremetal {
                    vec![] // baremetal skips libc/libgcc
                } else if options.tiny {
                    vec!["-lc".to_string(), "-lgcc".to_string()]
                } else {
                    vec!["-ldl".to_string()]
                })
                .args(if link_tiny_runtime || options.profile == BuildProfile::Baremetal {
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
        }
        if std::env::var("SNASK_KEEP_TEMPS").ok().as_deref() != Some("1") {
            fs::remove_file(ir_file).ok();
        }
    } else {
        let obj_file = "temp_snask.o";
        pb.set_message(format!(
            "Compiling ({} -O{})",
            toolchain::tool_display(&llc_path),
            options.opt_level
        ));
        let mut llc = Command::new(&llc_path);
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

        let link_display = if has_linker_ld { "ld".to_string() } else { toolchain::tool_display(&clang_path) };
        pb.set_message(format!("Linking ({})", link_display));

        if has_linker_ld {
            // baremetal + linker.ld: use ld directly with the linker script
            let ld_path = lld.as_ref().map(|p| p.as_path()).unwrap_or_else(|| std::path::Path::new("ld"));
            let mut link = Command::new(ld_path);
            let target_flag = if let Some(t) = &options.target {
                if t.contains("x86_64") { Some("-melf_x86_64") }
                else if t.contains("i386") || t.contains("i686") { Some("-melf_i386") }
                else { None }
            } else { Some("-melf_x86_64") };
            if let Some(emul) = target_flag {
                link.arg(emul);
            }
            let status = link
                .arg("-T").arg("linker.ld")
                .arg(obj_file)
                .arg("-o").arg(&final_output)
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                return Err("Link step failed (linker.ld).".to_string());
            }
        } else {
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

            let mut clang = Command::new(&clang_path);
            clang.arg(&clang_opt);
            if have_lld {
                if let Some(path) = &lld {
                    clang.arg(format!("-fuse-ld={}", path.to_string_lossy()));
                }
            }
            if extreme_obj.is_some() {
                clang.arg("-nostdlib").arg("-static");
            }
            if options.profile == BuildProfile::Baremetal {
                clang.arg("-ffreestanding").arg("-nostdlib").arg("-static").arg("-mno-red-zone");
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
            } else if options.profile == BuildProfile::Baremetal {
            } else if options.tiny {
                args.push(lib_snask);
            } else {
                args.push(runtime_path);
            }

            let status = clang
                .args(&args)
            .arg("-o")
            .arg(&final_output)
            .args(if options.extreme {
                vec![]
            } else if options.profile == BuildProfile::Baremetal {
                vec![]
            } else if options.tiny {
                vec!["-lc".to_string(), "-lgcc".to_string()]
            } else {
                vec!["-ldl".to_string()]
            })
            .args(if link_tiny_runtime || options.profile == BuildProfile::Baremetal {
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
    let strip_tool = if let Some(path) = toolchain::llvm_strip() {
        Some(path)
    } else if Command::new("strip").arg("--version").output().is_ok() {
        Some("strip".into())
    } else {
        None
    };
    if let Some(strip_tool) = strip_tool {
        let _ = Command::new(strip_tool).arg(path).status();
    }
}

pub fn resolve_imports(
    program: &mut Program,
    _entry_dir: &Path,
    resolved_program: &mut Program,
    _resolved_modules: &mut HashSet<String>,
) -> Result<(), String> {
    resolved_program.extend(program.drain(..));
    Ok(())
}

fn namespace_imported_module(program: &mut Program, prefix: &str) {
    let local_symbols: HashSet<String> = program
        .iter()
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::FuncDeclaration(f) => Some(f.name.clone()),
            StmtKind::VarDeclaration(v) => Some(v.name.clone()),
            StmtKind::MutDeclaration(v) => Some(v.name.clone()),
            StmtKind::ConstDeclaration(v) => Some(v.name.clone()),
            _ => None,
        })
        .collect();

    for stmt in program {
        rewrite_stmt_for_namespace(stmt, prefix, &local_symbols);
        match &mut stmt.kind {
            StmtKind::FuncDeclaration(f) => f.name = namespaced(prefix, &f.name),
            StmtKind::VarDeclaration(v) => v.name = namespaced(prefix, &v.name),
            StmtKind::MutDeclaration(v) => v.name = namespaced(prefix, &v.name),
            StmtKind::ConstDeclaration(v) => v.name = namespaced(prefix, &v.name),
            _ => {}
        }
    }
}

fn namespaced(prefix: &str, name: &str) -> String {
    if name.contains("::") {
        name.to_string()
    } else {
        format!("{prefix}::{name}")
    }
}

fn rewrite_type_for_namespace(ty: &mut Type, prefix: &str, local_symbols: &HashSet<String>) {
    match ty {
        Type::User(name) if local_symbols.contains(name) => *name = namespaced(prefix, name),
        Type::Function(params, ret) => {
            for p in params {
                rewrite_type_for_namespace(p, prefix, local_symbols);
            }
            rewrite_type_for_namespace(ret, prefix, local_symbols);
        }
        Type::Array(inner, _) => rewrite_type_for_namespace(inner, prefix, local_symbols),
        Type::Volatile(inner) => rewrite_type_for_namespace(inner, prefix, local_symbols),
        _ => {}
    }
}

fn rewrite_opt_type_for_namespace(
    ty: &mut Option<Type>,
    prefix: &str,
    local_symbols: &HashSet<String>,
) {
    if let Some(ty) = ty {
        rewrite_type_for_namespace(ty, prefix, local_symbols);
    }
}

fn rewrite_var_decl_for_namespace(v: &mut VarDecl, prefix: &str, local_symbols: &HashSet<String>) {
    rewrite_opt_type_for_namespace(&mut v.var_type, prefix, local_symbols);
    rewrite_expr_for_namespace(&mut v.value, prefix, local_symbols);
}

fn rewrite_mut_decl_for_namespace(v: &mut MutDecl, prefix: &str, local_symbols: &HashSet<String>) {
    rewrite_opt_type_for_namespace(&mut v.var_type, prefix, local_symbols);
    rewrite_expr_for_namespace(&mut v.value, prefix, local_symbols);
}

fn rewrite_const_decl_for_namespace(
    v: &mut ConstDecl,
    prefix: &str,
    local_symbols: &HashSet<String>,
) {
    rewrite_opt_type_for_namespace(&mut v.var_type, prefix, local_symbols);
    rewrite_expr_for_namespace(&mut v.value, prefix, local_symbols);
}

fn rewrite_func_decl_for_namespace(
    f: &mut FuncDecl,
    prefix: &str,
    local_symbols: &HashSet<String>,
) {
    for (_, ty) in &mut f.params {
        rewrite_type_for_namespace(ty, prefix, local_symbols);
    }
    rewrite_opt_type_for_namespace(&mut f.return_type, prefix, local_symbols);
    let mut visible_symbols = local_symbols.clone();
    for (name, _) in &f.params {
        visible_symbols.remove(name);
    }
    let mut local_names = HashSet::new();
    collect_stmt_local_names(&f.body, &mut local_names);
    for name in local_names {
        visible_symbols.remove(&name);
    }
    rewrite_stmts_for_namespace(&mut f.body, prefix, &visible_symbols);
}

fn collect_stmt_local_names(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::VarDeclaration(v) => {
                names.insert(v.name.clone());
            }
            StmtKind::MutDeclaration(v) => {
                names.insert(v.name.clone());
            }
            StmtKind::ConstDeclaration(v) => {
                names.insert(v.name.clone());
            }
            StmtKind::Conditional(c) => {
                collect_stmt_local_names(&c.if_block.body, names);
                for block in &c.elif_blocks {
                    collect_stmt_local_names(&block.body, names);
                }
                if let Some(body) = &c.else_block {
                    collect_stmt_local_names(body, names);
                }
            }
            StmtKind::Loop(LoopStmt::While { body, .. }) => collect_stmt_local_names(body, names),
            StmtKind::Loop(LoopStmt::For { iterator, body, .. }) => {
                names.insert(iterator.clone());
                collect_stmt_local_names(body, names);
            }
            StmtKind::UnsafeBlock(body) => {
                collect_stmt_local_names(body, names);
            }
            _ => {}
        }
    }
}

fn rewrite_stmts_for_namespace(stmts: &mut [Stmt], prefix: &str, local_symbols: &HashSet<String>) {
    for stmt in stmts {
        rewrite_stmt_for_namespace(stmt, prefix, local_symbols);
    }
}

fn rewrite_if_block_for_namespace(
    block: &mut IfBlock,
    prefix: &str,
    local_symbols: &HashSet<String>,
) {
    rewrite_expr_for_namespace(&mut block.condition, prefix, local_symbols);
    rewrite_stmts_for_namespace(&mut block.body, prefix, local_symbols);
}

fn rewrite_conditional_for_namespace(
    c: &mut ConditionalStmt,
    prefix: &str,
    local_symbols: &HashSet<String>,
) {
    rewrite_if_block_for_namespace(&mut c.if_block, prefix, local_symbols);
    for block in &mut c.elif_blocks {
        rewrite_if_block_for_namespace(block, prefix, local_symbols);
    }
    if let Some(body) = &mut c.else_block {
        rewrite_stmts_for_namespace(body, prefix, local_symbols);
    }
}

fn rewrite_stmt_for_namespace(stmt: &mut Stmt, prefix: &str, local_symbols: &HashSet<String>) {
    match &mut stmt.kind {
        StmtKind::Expression(e) | StmtKind::FuncCall(e) | StmtKind::Return(e) => {
            rewrite_expr_for_namespace(e, prefix, local_symbols)
        }
        StmtKind::VarDeclaration(v) => rewrite_var_decl_for_namespace(v, prefix, local_symbols),
        StmtKind::MutDeclaration(v) => rewrite_mut_decl_for_namespace(v, prefix, local_symbols),
        StmtKind::ConstDeclaration(v) => rewrite_const_decl_for_namespace(v, prefix, local_symbols),
        StmtKind::VarAssignment(v) => {
            if local_symbols.contains(&v.name) {
                v.name = namespaced(prefix, &v.name);
            }
            rewrite_expr_for_namespace(&mut v.value, prefix, local_symbols)
        }
        StmtKind::IndexAssignment(a) => {
            rewrite_expr_for_namespace(&mut a.target, prefix, local_symbols);
            rewrite_expr_for_namespace(&mut a.index, prefix, local_symbols);
            rewrite_expr_for_namespace(&mut a.value, prefix, local_symbols);
        }
        StmtKind::FuncDeclaration(f) => rewrite_func_decl_for_namespace(f, prefix, local_symbols),
        StmtKind::Conditional(c) => rewrite_conditional_for_namespace(c, prefix, local_symbols),
        StmtKind::Loop(LoopStmt::While { condition, body }) => {
            rewrite_expr_for_namespace(condition, prefix, local_symbols);
            rewrite_stmts_for_namespace(body, prefix, local_symbols);
        }
        StmtKind::Loop(LoopStmt::For { iterable, body, .. }) => {
            rewrite_expr_for_namespace(iterable, prefix, local_symbols);
            rewrite_stmts_for_namespace(body, prefix, local_symbols);
        }
        StmtKind::UnsafeBlock(body) => {
            rewrite_stmts_for_namespace(body, prefix, local_symbols);
        }
        _ => {}
    }
}

fn rewrite_literal_for_namespace(
    _lit: &mut LiteralValue,
    _prefix: &str,
    _local_symbols: &HashSet<String>,
) {
}

fn rewrite_expr_for_namespace(expr: &mut Expr, prefix: &str, local_symbols: &HashSet<String>) {
    match &mut expr.kind {
        ExprKind::Literal(lit) => rewrite_literal_for_namespace(lit, prefix, local_symbols),
        ExprKind::Variable(name) if local_symbols.contains(name) => {
            *name = namespaced(prefix, name)
        }
        ExprKind::Unary { expr, .. } => rewrite_expr_for_namespace(expr, prefix, local_symbols),
        ExprKind::Binary { left, right, .. } => {
            rewrite_expr_for_namespace(left, prefix, local_symbols);
            rewrite_expr_for_namespace(right, prefix, local_symbols);
        }
        ExprKind::FunctionCall { callee, args } => {
            rewrite_expr_for_namespace(callee, prefix, local_symbols);
            for arg in args {
                rewrite_expr_for_namespace(arg, prefix, local_symbols);
            }
        }
        ExprKind::IndexAccess { target, index } => {
            rewrite_expr_for_namespace(target, prefix, local_symbols);
            rewrite_expr_for_namespace(index, prefix, local_symbols);
        }
        ExprKind::Deref { ptr, .. } => rewrite_expr_for_namespace(ptr, prefix, local_symbols),
        ExprKind::SizeOf(e) | ExprKind::AlignOf(e) => {
            rewrite_expr_for_namespace(e, prefix, local_symbols);
        }
        ExprKind::OffsetOf { expr, .. } => {
            rewrite_expr_for_namespace(expr, prefix, local_symbols);
        }
        ExprKind::VolatileLoad { ptr, .. } => {
            rewrite_expr_for_namespace(ptr, prefix, local_symbols);
        }
        ExprKind::VolatileStore { ptr, value } => {
            rewrite_expr_for_namespace(ptr, prefix, local_symbols);
            rewrite_expr_for_namespace(value, prefix, local_symbols);
        }
        ExprKind::IntToPtr { expr, .. } | ExprKind::PtrToInt { expr, .. } => {
            rewrite_expr_for_namespace(expr, prefix, local_symbols);
        }
        _ => {}
    }
}

fn render_parser_diagnostic(filename: &str, source: &str, err: &ParseError) -> String {
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
        TinyDisallowedLib(_) => "not available in tiny mode",
        _ => "error",
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
