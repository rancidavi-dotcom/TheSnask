use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::process::Command;
use std::path::{Path};
use indicatif::{ProgressBar, ProgressStyle};
use inkwell::context::Context;

use crate::ast::{ConstDecl, Expr, ExprKind, LiteralValue, Location, Program, Stmt, StmtKind};
use crate::llvm_generator::LLVMGenerator;
use crate::parser::{Parser, ParseError};
use crate::semantic_analyzer::{SemanticAnalyzer, SemanticError};
use crate::sps::{SnifFeatureValue};

/// Options for the compiler build process.
#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub output_name: Option<String>,
    pub target: Option<String>,
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
    resolve_imports(&mut program, entry_dir, &mut resolved_program, &mut resolved_modules)?;
    pb.inc(1);

    pb.set_message("Expanding inheritance");
    expand_inheritance(&mut resolved_program)?;
    pb.inc(1);

    // Determine runtime linking strategy
    let needs_full_runtime = uses_full_runtime(&resolved_program);
    if options.min_runtime && needs_full_runtime {
        pb.finish_and_clear();
        return Err("`--min-runtime` cannot be used with GUI/SQLite/Skia/Web imports.\n".to_string());
    }
    let link_tiny_runtime = options.tiny || (options.min_runtime && !needs_full_runtime);

    pb.set_message("Semantic analysis");
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.set_tiny_mode(options.tiny);
    analyzer.analyze(&resolved_program);
    if !analyzer.errors.is_empty() {
        pb.finish_and_clear();
        return Err(render_semantic_diagnostics(file_path, &source, &analyzer.errors));
    }
    pb.inc(1);

    pb.set_message("Generating LLVM IR");
    let context = Context::create();
    let mut generator = LLVMGenerator::new(&context, file_path);
    let ir = generator.generate(resolved_program)?;
    pb.inc(1);

    link_binary(file_path, ir, options, link_tiny_runtime, &pb)?;
    
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
            value: Expr::with_span(
                ExprKind::Literal(literal_val),
                loc.clone(),
                span.clone(),
            ),
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
    let has_main = program.iter().any(|stmt| {
        if let StmtKind::ClassDeclaration(class) = &stmt.kind {
            class.name == "main"
        } else {
            false
        }
    });

    if !has_main {
        return Err("Error: every Snask program must contain a `class main` with a `fun start()` entrypoint.".to_string());
    }
    Ok(())
}

fn expand_inheritance(program: &mut Program) -> Result<(), String> {
    let mut classes: std::collections::HashMap<String, crate::ast::ClassDecl> = std::collections::HashMap::new();
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
                    format!("Parent class '{}' not found for class '{}'.", parent_name, name)
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
        matches!(name, "gui" | "sqlite" | "snask_skia" | "blaze" | "blaze_auth" | "auth")
    }
    for st in program {
        match &st.kind {
            StmtKind::Import(m) => {
                if is_heavy_module(m) { return true; }
            }
            StmtKind::FromImport { module, .. } => {
                if is_heavy_module(module) { return true; }
            }
            _ => {}
        }
    }
    false
}

fn link_binary(
    file_path: &str, 
    ir: String, 
    options: BuildOptions, 
    link_tiny_runtime: bool,
    pb: &ProgressBar
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

    let final_output = options.output_name.unwrap_or_else(|| file_path.replace(".snask", ""));

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
        if have_lld { clang.arg("-fuse-ld=lld"); }
        if extreme_obj.is_some() { clang.arg("-nostdlib").arg("-static"); }
        if let Some(t) = &options.target { clang.arg(format!("--target={}", t)); }
        if let Some(p) = &extreme_obj { clang.arg(p); }
        
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
            .args(if options.extreme { vec![] } else if options.tiny { vec!["-lc".to_string(), "-lgcc".to_string()] } else { vec!["-ldl".to_string()] })
            .args(if link_tiny_runtime { vec![] } else { vec!["-lm".to_string()] })
            .args(get_link_flags(size_link, have_lld))
            .args(get_runtime_linkargs_for(options.target.as_deref(), link_tiny_runtime))
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() { return Err("Final link step failed (LTO path).".to_string()); }
        fs::remove_file(ir_file).ok();
    } else {
        let obj_file = "temp_snask.o";
        pb.set_message(format!("Compiling (llc-18 -O{})", options.opt_level));
        let mut llc = Command::new("llc-18");
        llc.arg(format!("-O{}", options.opt_level)).arg("-relocation-model=pic").arg("-filetype=obj");
        if let Some(t) = &options.target { llc.arg(format!("-mtriple={}", t)); }
        llc.arg(ir_file).arg("-o").arg(obj_file).status().map_err(|e| e.to_string())?;

        pb.set_message("Linking (clang-18)");
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        
        let mut runtime_path = if let Some(t) = &options.target {
            if options.tiny { format!("{}/.snask/lib/{}/runtime_nano.o", home, t) }
            else if link_tiny_runtime { format!("{}/.snask/lib/{}/runtime_tiny.o", home, t) }
            else { format!("{}/.snask/lib/{}/runtime.o", home, t) }
        } else if options.tiny { format!("{}/.snask/lib/runtime_nano.o", home) }
        else if link_tiny_runtime { format!("{}/.snask/lib/runtime_tiny.o", home) }
        else { format!("{}/.snask/lib/runtime.o", home) };

        // APT Support: Check for global installation paths
        let global_runtime = "/usr/lib/snask/runtime/runtime.o";
        if !std::path::Path::new(&runtime_path).exists() && std::path::Path::new(global_runtime).exists() {
             runtime_path = global_runtime.to_string();
        }

        let mut clang = Command::new("clang-18");
        clang.arg(&clang_opt);
        if have_lld { clang.arg("-fuse-ld=lld"); }
        if extreme_obj.is_some() { clang.arg("-nostdlib").arg("-static"); }
        if let Some(t) = &options.target { clang.arg(format!("--target={}", t)); }
        if let Some(p) = &extreme_obj { clang.arg(p); }
        
        let mut lib_snask = format!("{}/.snask/lib/libsnask.a", home);
        let global_lib = "/usr/lib/snask/libsnask.a";
        if !std::path::Path::new(&lib_snask).exists() && std::path::Path::new(global_lib).exists() {
             lib_snask = global_lib.to_string();
        }
        
        let mut args = vec![obj_file.to_string()];
        if options.extreme { } else if options.tiny { args.push(lib_snask); } else { args.push(runtime_path); }

        let status = clang
            .args(&args)
            .arg("-o")
            .arg(&final_output)
            .args(if options.extreme { vec![] } else if options.tiny { vec!["-lc".to_string(), "-lgcc".to_string()] } else { vec!["-ldl".to_string()] })
            .args(if link_tiny_runtime { vec![] } else { vec!["-lm".to_string()] })
            .args(get_link_flags(size_link, have_lld))
            .args(get_runtime_linkargs_for(options.target.as_deref(), link_tiny_runtime))
            .status()
            .map_err(|e| e.to_string())?;

        if !status.success() { return Err("Final link step failed.".to_string()); }
        fs::remove_file(ir_file).ok(); fs::remove_file(obj_file).ok();
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
        if have_lld { v.push("-Wl,--icf=all".to_string()); }
        v
    } else {
        vec!["-Wl,--export-dynamic".to_string(), "-rdynamic".to_string()]
    }
}

pub fn get_runtime_linkargs_for(target: Option<&str>, tiny: bool) -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let p = if let Some(t) = target {
        if tiny { format!("{}/.snask/lib/{}/runtime_tiny.linkargs", home, t) }
        else { format!("{}/.snask/lib/{}/runtime.linkargs", home, t) }
    } else {
        if tiny { format!("{}/.snask/lib/runtime_tiny.linkargs", home) }
        else { format!("{}/.snask/lib/runtime.linkargs", home) }
    };
    let Ok(s) = std::fs::read_to_string(&p) else { return Vec::new(); };
    s.split_whitespace().map(|x| x.to_string()).collect()
}

fn strip_binary(path: &str) {
    let strip_tool = if Command::new("llvm-strip-18").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
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
                let module_path = resolve_module_path(entry_dir, module_name)?;
                if !resolved_modules.contains(&module_path) {
                    resolved_modules.insert(module_path.clone());
                    let source = fs::read_to_string(&module_path).map_err(|e| format!("Failed to read module {}: {}", module_name, e))?;
                    let mut parser = Parser::new(&source).map_err(|e| render_parser_diagnostic(&module_path, &source, &e))?;
                    let (mut module_program, errors) = parser.parse_program_recovering(10);
                    if !errors.is_empty() {
                        return Err(render_parser_diagnostics(&module_path, &source, &errors));
                    }
                    if let Some(mut prog) = module_program {
                        resolve_imports(&mut prog, Path::new(&module_path).parent().unwrap(), resolved_program, resolved_modules)?;
                    }
                }
            }
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
    let pkg_direct = Path::new(&home).join(".snask/packages").join(&name_with_ext);
    if pkg_direct.exists() {
        return Ok(pkg_direct.to_string_lossy().to_string());
    }
    let pkg_nested = Path::new(&home).join(".snask/packages").join(module_name).join(&name_with_ext);
    if pkg_nested.exists() {
        return Ok(pkg_nested.to_string_lossy().to_string());
    }
    Err(format!("Module '{}' not found.", module_name))
}

pub fn render_parser_diagnostic(_filename: &str, _source: &str, err: &ParseError) -> String {
    format!("Error: {} at line {}, col {}", err.message, err.span.start.line, err.span.start.column)
}

pub fn render_parser_diagnostics(filename: &str, source: &str, errs: &[ParseError]) -> String {
    errs.iter().map(|e| render_parser_diagnostic(filename, source, e)).collect::<Vec<_>>().join("\n")
}

pub fn render_semantic_diagnostics(filename: &str, _source: &str, errors: &[SemanticError]) -> String {
    errors.iter().map(|e| format!("Semantic Error in {}: {:?}", filename, e)).collect::<Vec<_>>().join("\n")
}
