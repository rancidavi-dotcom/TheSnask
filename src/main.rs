pub mod ast;
pub mod value;
pub mod symbol_table;
pub mod semantic_analyzer;
pub mod types;
pub mod parser;
pub mod modules;
pub mod stdlib;
pub mod span;
pub mod diagnostics;
pub mod packages;
pub mod lib_tool;
pub mod llvm_generator;
pub mod sps;
pub mod snif_parser;

use std::fs;
use std::process::Command;
use clap::{Parser as ClapParser, Subcommand};
use crate::parser::Parser;
use crate::semantic_analyzer::{SemanticAnalyzer, SemanticError};
use crate::llvm_generator::LLVMGenerator;
use inkwell::context::Context;
use crate::ast::{Program, StmtKind};
use crate::modules::load_module;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

const EMBEDDED_RUNTIME_C: &str = include_str!("runtime.c");
const EMBEDDED_RUNTIME_OLD: &str = include_str!("runtime/runtime_old.c");

#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { #[arg(short, long)] name: Option<String> },
    Build { file: Option<String>, #[arg(short, long)] output: Option<String>, #[arg(long)] target: Option<String> },
    Run { file: Option<String> },
    /// Generates artifacts in `dist/` (binaries and, on Linux, .deb/.AppImage when possible)
    Dist {
        /// .snask file (if omitted, SPS uses `snask.snif` when present)
        file: Option<String>,
        /// Lista de targets LLVM (mtriple). Ex: x86_64-unknown-linux-gnu
        #[arg(long)]
        targets: Option<String>,
        /// Atalho para targets comuns (linux + windows + macos)
        #[arg(long)]
        all: bool,
        /// Gera .deb (somente Linux)
        #[arg(long)]
        deb: bool,
        /// Gera .AppImage (somente Linux, requer appimagetool)
        #[arg(long)]
        appimage: bool,
        /// Nome do binário (default: nome do arquivo/manifest)
        #[arg(short, long)]
        name: Option<String>,
        /// Diretório de saída (default: dist)
        #[arg(long, default_value = "dist")]
        out_dir: String,
    },
    Add { name: String, version: Option<String> },
    Remove { name: String },
    Setup { #[arg(long)] target: Option<String> },
    Install { name: String },
    Uninstall { name: Option<String> },
    Update { name: Option<String> },
    List,
    Search { query: String },
    /// Ferramentas para criar/publicar bibliotecas Snask
    Lib {
        #[command(subcommand)]
        cmd: LibCommands,
    },
}

#[derive(Subcommand)]
enum LibCommands {
    /// Cria um template de biblioteca no diretório atual
    Init {
        name: String,
        #[arg(long, default_value = "0.1.0")]
        version: String,
        #[arg(long, default_value = "Minha biblioteca Snask.")]
        description: String,
    },
    /// Publica a biblioteca atual no registry (SnaskPackages via ~/.snask/registry)
    Publish {
        name: String,
        /// Se omitido, usa o package.json
        #[arg(long)]
        version: Option<String>,
        /// Se omitido, usa o package.json
        #[arg(long)]
        description: Option<String>,
        /// Mensagem do commit
        #[arg(long)]
        message: Option<String>,
        /// Faz git push automaticamente
        #[arg(long)]
        push: bool,
        /// Publica via fork + Pull Request (não precisa permissão no repo)
        #[arg(long)]
        pr: bool,
        /// URL do seu fork (ex: https://github.com/rancidavi-dotcom/SnaskPackages)
        #[arg(long)]
        fork: Option<String>,
        /// Nome da branch (default: pkg/<nome>-v<versao>)
        #[arg(long)]
        branch: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init { name } => {
            if let Err(e) = sps_init(name.clone()) {
                eprintln!("Project init error: {}", e);
            }
        }
        Commands::Build { file, output, target } => {
            match build_entry(file.clone(), output.clone(), target.clone()) {
                Ok(_) => println!("LLVM compilation finished successfully."),
                Err(e) => eprintln!("Compilation error: {}", e),
            }
        },
        Commands::Dist { file, targets, all, deb, appimage, name, out_dir } => {
            if let Err(e) = dist_entry(file.clone(), targets.clone(), *all, *deb, *appimage, name.clone(), out_dir.clone()) {
                eprintln!("Error: {}", e);
            }
        }
        Commands::Run { file } => {
            // scripts: `snask run dev`
            if let Some(arg) = file.clone() {
                if !arg.ends_with(".snask") {
                    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
                        if let Some(cmdline) = m.scripts.get(&arg) {
                            let status = Command::new("sh").arg("-lc").arg(cmdline).status();
                            if let Err(e) = status {
                                eprintln!("Failed to run script '{}': {}", arg, e);
                            }
                            return;
                        }
                    }
                }
            }

            let cwd = match std::env::current_dir() {
                Ok(d) => d,
                Err(e) => { eprintln!("Error: {}", e); return; }
            };
            let file_path = match crate::sps::load_manifest_from(&cwd) {
                Ok((m, _p)) => {
                    if let Err(e) = sps_pin_from_lock(&cwd, &m) {
                        eprintln!("Compilation error: {}", e);
                        return;
                    }
                    if let Err(e) = sps_resolve_deps_and_lock(&cwd, &m) {
                        eprintln!("Compilation error: {}", e);
                        return;
                    }
                    file.clone().unwrap_or_else(|| m.package.entry.clone())
                }
                Err(_) => match resolve_entry_file(file.clone()) {
                    Ok(p) => p,
                    Err(e) => { eprintln!("Compilation error: {}", e); return; }
                }
            };

            // compila (com opt do SPS se existir)
            let build_res = if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
                let opt = m.opt_level_for(true);
                build_file_with_opt(&file_path, None, opt, None)
            } else {
                build_file(&file_path, None, None)
            };

            match build_res {
                Ok(_) => {
                    let binary = file_path.replace(".snask", "");
                    let binary_path = if binary.starts_with("/") || binary.starts_with("./") { binary } else { format!("./{}", binary) };
                    let status = Command::new(&binary_path).status();
                    if let Err(e) = status {
                        eprintln!("Failed to run binary: {}", e);
                    }
                },
                Err(e) => eprintln!("Compilation error: {}", e),
            }
        },
        Commands::Add { name, version } => {
            if let Err(e) = sps_add_dependency(name, version.clone()) {
                eprintln!("Failed to add dependency: {}", e);
            }
        }
        Commands::Remove { name } => {
            if let Err(e) = sps_remove_dependency(name) {
                eprintln!("Failed to remove dependency: {}", e);
            }
        }
        Commands::Setup { target } => {
            if let Err(e) = run_setup(target.clone()) {
                eprintln!("Setup error: {}", e);
            }
        },
        Commands::Install { name } => {
            if let Err(e) = packages::install_package(name) {
                eprintln!("Package install error: {}", e);
            }
        },
        Commands::Uninstall { name } => {
            if let Some(pkg_name) = name {
                if let Err(e) = packages::uninstall_package(&pkg_name) {
                    eprintln!("Package uninstall error: {}", e);
                }
            } else {
                println!("⚠️  Warning: you are about to uninstall Snask globally.");
                if let Err(e) = run_uninstall() {
                    eprintln!("Uninstall error: {}", e);
                }
            }
        },
        Commands::Update { name } => {
            if let Some(pkg_name) = name {
                println!("🔄 Atualizando pacote '{}'...", pkg_name);
                if let Err(e) = packages::install_package(pkg_name) {
                    eprintln!("Package update error: {}", e);
                }
            } else {
                println!("🚀 Starting Snask self-update...");
                if let Err(e) = self_update() {
                    eprintln!("Snask update error: {}", e);
                }
            }
        },
        Commands::List => {
            if let Err(e) = packages::list_packages() {
                eprintln!("Failed to list packages: {}", e);
            }
        },
        Commands::Search { query } => {
            if let Err(e) = packages::search_packages(query) {
                eprintln!("Failed to search packages: {}", e);
            }
        },
        Commands::Lib { cmd } => {
            match cmd {
                LibCommands::Init { name, version, description } => {
                    let res = crate::lib_tool::lib_init(crate::lib_tool::NewLibOpts {
                        name: name.clone(),
                        description: description.clone(),
                        version: version.clone(),
                    });
                    if let Err(e) = res {
                        eprintln!("Error: {}", e);
                    }
                }
                LibCommands::Publish { name, version, description, message, push, pr, fork, branch } => {
                    let res = crate::lib_tool::lib_publish(crate::lib_tool::PublishOpts {
                        name: name.clone(),
                        version: version.clone(),
                        description: description.clone(),
                        message: message.clone(),
                        push: *push,
                        pr: *pr,
                        fork: fork.clone(),
                        branch: branch.clone(),
                    });
                    if let Err(e) = res {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }
    }
}

fn resolve_entry_file(cli_file: Option<String>) -> Result<String, String> {
    if let Some(f) = cli_file {
        return Ok(f);
    }

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
        return Ok(m.package.entry);
    }
    Err("SPS: no input file provided and `snask.snif` was not found in the current directory.\n\nHow to fix:\n- Build a file directly: `snask build main.snask`\n- Or create an SPS project: `snask init` and then `snask build`\n".to_string())
}

fn build_entry(cli_file: Option<String>, output_name: Option<String>, target: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    spinner.enable_steady_tick(std::time::Duration::from_millis(90));

    // Se estiver em um projeto SPS, resolve deps e gera lock antes de build.
    if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
        spinner.set_message(format!("SPS: snask.snif (entry: {})", m.package.entry));
        // pin pelo lock (se existir) antes de resolver
        spinner.set_message("SPS: verificando snask.lock".to_string());
        sps_pin_from_lock(&cwd, &m)?;
        spinner.set_message("SPS: resolving dependencies".to_string());
        sps_resolve_deps_and_lock(&cwd, &m)?;
        let opt = m.opt_level_for(true);
        let file_path = cli_file.unwrap_or_else(|| m.package.entry.clone());
        let out = if output_name.is_some() { output_name } else { Some(m.package.name.clone()) };
        spinner.set_message(format!("Compiling {} -> {} (O{})", file_path, out.clone().unwrap_or_default(), opt));
        let res = build_file_with_opt(&file_path, out, opt, target.clone());
        match &res {
            Ok(_) => spinner.finish_with_message("Build finished"),
            Err(_) => spinner.finish_and_clear(),
        }
        return res;
    }

    let file_path = resolve_entry_file(cli_file)?;
    spinner.set_message(format!("Compiling {}", file_path));
    let res = build_file_with_opt(&file_path, output_name, 2, target);
    match &res {
        Ok(_) => spinner.finish_with_message("Build finished"),
        Err(_) => spinner.finish_and_clear(),
    }
    res
}

fn build_file(file_path: &str, output_name: Option<String>, target: Option<String>) -> Result<(), String> {
    build_file_with_opt(file_path, output_name, 2, target)
}

fn build_file_with_opt(file_path: &str, output_name: Option<String>, opt_level: u8, target: Option<String>) -> Result<(), String> {
    let pb = ProgressBar::new(6);
    pb.set_style(
        ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message("Lendo arquivo");
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    pb.inc(1);

    pb.set_message("Parser (tokens/AST)");
    let mut program = Parser::new(&source)?.parse_program().map_err(|e| format!("Parser error: {}", e))?;
    pb.inc(1);

    // Validação de Class Main Obrigatória apenas no programa principal
    let has_main = program.iter().any(|stmt| {
        if let StmtKind::ClassDeclaration(class) = &stmt.kind {
            class.name == "main"
        } else {
            false
        }
    });

    if !has_main {
        pb.finish_and_clear();
        return Err("Error: every Snask program must contain a 'class main'.".to_string());
    }

    pb.set_message("Resolvendo imports");
    let mut resolved_program = Vec::new();
    let mut resolved_modules = std::collections::HashSet::new();
    resolved_modules.insert(file_path.to_string());
    resolve_imports(&mut program, &mut resolved_program, &mut resolved_modules)?;
    pb.inc(1);

    pb.set_message("Análise semântica");
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&resolved_program);
    if !analyzer.errors.is_empty() {
        pb.finish_and_clear();
        let mut out = String::from("Análise semântica encontrou erros:\n");
        for e in &analyzer.errors {
            out.push_str(&format!("- {}\n", format_semantic_error(e)));
        }
        return Err(out);
    }
    pb.inc(1);

    pb.set_message("Gerando LLVM IR");
    let context = Context::create();
    let mut generator = LLVMGenerator::new(&context, file_path);
    let ir = generator.generate(resolved_program)?;
    pb.inc(1);

    let ir_file = "temp_snask.ll";
    let obj_file = "temp_snask.o";
    fs::write(ir_file, ir).map_err(|e| e.to_string())?;

    pb.set_message(format!("Compilando (llc-18 -O{})", opt_level));
    let opt_flag = format!("-O{}", opt_level);
    let mut llc = Command::new("llc-18");
    llc
        .arg(opt_flag)
        .arg("-relocation-model=pic")
        .arg("-filetype=obj")
        ;
    if let Some(t) = &target {
        llc.arg(format!("-mtriple={}", t));
    }
    llc
        .arg(ir_file)
        .arg("-o")
        .arg(obj_file)
        .status()
        .map_err(|e| e.to_string())?;

    pb.set_message("Linkando (clang-18)");
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let runtime_path = if let Some(t) = &target {
        format!("{}/.snask/lib/{}/runtime.o", home, t)
    } else {
        format!("{}/.snask/lib/runtime.o", home)
    };
    if !std::path::Path::new(&runtime_path).exists() {
        return Err(format!("Runtime não encontrado em '{}'. Rode `snask setup{}`.", runtime_path, target.as_ref().map(|t| format!(" --target {}", t)).unwrap_or_default()));
    }
    let final_output = output_name.unwrap_or_else(|| file_path.replace(".snask", ""));

    let mut clang = Command::new("clang-18");
    if let Some(t) = &target {
        clang.arg(format!("--target={}", t));
    }
    let status = clang
        .arg(obj_file)
        .arg(runtime_path)
        .arg("-o")
        .arg(&final_output)
        .arg("-lm")
        // Necessário para blaze (dlsym handlers) e para expor símbolos do binário
        .arg("-ldl")
        .arg("-Wl,--export-dynamic")
        // Alguns toolchains precisam de -rdynamic para dlsym() enxergar símbolos do executável
        .arg("-rdynamic")
        // Link args requeridos pelo runtime (ex.: gtk/sqlite se habilitados no setup)
        .args(get_runtime_linkargs(target.as_deref()))
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() { return Err("Final link step failed.".to_string()); }
    fs::remove_file(ir_file).ok(); fs::remove_file(obj_file).ok();
    pb.inc(1);
    pb.finish_with_message("OK");
    Ok(())
}

fn get_runtime_linkargs(target: Option<&str>) -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let p = if let Some(t) = target {
        format!("{}/.snask/lib/{}/runtime.linkargs", home, t)
    } else {
        format!("{}/.snask/lib/runtime.linkargs", home)
    };
    let Ok(s) = std::fs::read_to_string(&p) else { return Vec::new(); };
    s.split_whitespace().map(|x| x.to_string()).collect()
}

fn sps_init(name: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let project_name = name.unwrap_or_else(|| cwd.file_name().unwrap_or_default().to_string_lossy().to_string());
    let manifest_path = cwd.join("snask.snif");
    if manifest_path.exists() {
        return Err("snask.snif already exists in this directory.".to_string());
    }

    let entry = "main.snask";
    let main_path = cwd.join(entry);
    if main_path.exists() {
        return Err(format!("File '{}' already exists in this directory.", entry));
    }

    let manifest = format!(
        "{{\n  package: {{ name: \"{}\", version: \"0.1.0\", entry: \"{}\", }},\n  dependencies: {{}},\n  build: {{ opt_level: 2, }},\n}}\n",
        project_name.replace('\"', ""),
        entry
    );
    fs::write(&manifest_path, manifest).map_err(|e| e.to_string())?;

    let main_src = "class main\n    fun start()\n        print(\"hello snask\");\n";
    fs::write(&main_path, main_src).map_err(|e| e.to_string())?;

    println!("✅ SPS: created snask.snif and {}.", entry);
    Ok(())
}

fn sps_add_dependency(name: &str, version: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (mut m, manifest_path) = crate::sps::load_manifest_from(&cwd)?;
    m.dependencies.insert(name.to_string(), version.unwrap_or_else(|| "*".to_string()));
    crate::sps::write_manifest(&manifest_path, &m)?;

    // instala imediatamente
    let registry = crate::packages::fetch_registry()?;
    let _ = crate::packages::install_package_with_registry(name, &registry)?;
    // lock determinístico
    sps_resolve_deps_and_lock(&cwd, &m)?;

    println!("✅ SPS: dependency '{}' added.", name);
    Ok(())
}

fn sps_remove_dependency(name: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (mut m, manifest_path) = crate::sps::load_manifest_from(&cwd)?;
    m.dependencies.remove(name);
    crate::sps::write_manifest(&manifest_path, &m)?;

    // não desinstala global por padrão (pode ser compartilhado por outros projetos)
    sps_resolve_deps_and_lock(&cwd, &m)?;
    println!("✅ SPS: dependency '{}' removed from snask.snif.", name);
    Ok(())
}

fn sps_resolve_deps_and_lock(dir: &std::path::Path, manifest: &crate::sps::SpsManifest) -> Result<(), String> {
    let registry = crate::packages::fetch_registry()?;
    let mut locked = std::collections::BTreeMap::new();
    for (name, _req) in &manifest.dependencies {
        // constraints v1: "*" ou versão exata
        if let Some(pkg) = registry.packages.get(name) {
            let req = manifest.dependencies.get(name).map(|s| s.as_str()).unwrap_or("*");
            if req != "*" && req != pkg.version() {
                return Err(format!(
                    "SPS: constraint de versão não satisfeita para '{}': pedido '{}', registry '{}'",
                    name,
                    req,
                    pkg.version()
                ));
            }
        }

        let url = registry.packages.get(name).map(|p| {
            let u = p.url().trim();
            if u.is_empty() { None } else { Some(u.to_string()) }
        }).flatten();

        if !crate::packages::is_package_installed(name) {
            let (ver, sha, _path) = crate::packages::install_package_with_registry(name, &registry)?;
            locked.insert(name.clone(), crate::sps::LockedDep { version: ver, sha256: sha, url });
        } else {
            let sha = crate::packages::read_installed_package_sha256(name)?;
            let ver = crate::packages::read_installed_package_version_from_registry(name, &registry).unwrap_or_else(|| "unknown".to_string());
            locked.insert(name.clone(), crate::sps::LockedDep { version: ver, sha256: sha, url });
        }
    }
    crate::sps::write_lockfile(dir, manifest, locked)?;
    Ok(())
}

fn dist_entry(
    cli_file: Option<String>,
    targets_csv: Option<String>,
    all: bool,
    deb: bool,
    appimage: bool,
    name: Option<String>,
    out_dir: String,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // resolve entry file
    let file_path = if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
        // pin + resolve deps para garantir build determinístico (e lock)
        sps_pin_from_lock(&cwd, &m)?;
        sps_resolve_deps_and_lock(&cwd, &m)?;
        cli_file.unwrap_or_else(|| m.package.entry.clone())
    } else {
        resolve_entry_file(cli_file)?
    };

    // resolve base binary name
    let base_name = name.unwrap_or_else(|| {
        if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
            m.package.name.clone()
        } else {
            std::path::Path::new(&file_path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        }
    });

    let out_dir = cwd.join(out_dir);
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let mut targets: Vec<Option<String>> = Vec::new();
    if all {
        // targets “comuns”. Requer `snask setup --target <triple>` para cada alvo.
        targets.push(None); // host
        targets.push(Some("x86_64-pc-windows-gnu".to_string()));
        targets.push(Some("x86_64-apple-darwin".to_string()));
    } else if let Some(csv) = targets_csv {
        for t in csv.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            targets.push(Some(t.to_string()));
        }
        if targets.is_empty() {
            targets.push(None);
        }
    } else {
        targets.push(None);
    }

    println!("📦 dist: entry = {}", file_path);
    println!("📁 dist: out_dir = {}", out_dir.display());

    for t in &targets {
        let is_host = t.is_none();
        let triple = t.clone().unwrap_or_else(|| "host".to_string());

        let mut out_path = out_dir.join(&base_name);
        if !is_host {
            out_path = out_dir.join(format!("{}-{}", base_name, triple));
        }
        if t.as_deref() == Some("x86_64-pc-windows-gnu") {
            out_path.set_extension("exe");
        }

        // opt level: do SPS se existir, senão O2
        let opt = if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
            m.opt_level_for(true)
        } else {
            2
        };

        println!("🔧 build: {} -> {}", triple, out_path.display());
        build_file_with_opt(&file_path, Some(out_path.to_string_lossy().to_string()), opt, t.clone())?;
    }

    // Linux packaging (best-effort)
    #[cfg(target_os = "linux")]
    {
        if deb {
            let bin_path = out_dir.join(&base_name);
            if !bin_path.exists() {
                return Err(format!("Para gerar .deb, preciso do binário Linux nativo em '{}'. Rode `snask dist --deb` sem targets de cross ou inclua o host.", bin_path.display()));
            }
            let deb_path = make_deb(&out_dir, &base_name, &bin_path)?;
            println!("✅ .deb: {}", deb_path.display());
        }

        if appimage {
            let bin_path = out_dir.join(&base_name);
            if !bin_path.exists() {
                return Err(format!("Para gerar .AppImage, preciso do binário Linux nativo em '{}'.", bin_path.display()));
            }
            let app = make_appimage(&out_dir, &base_name, &bin_path)?;
            println!("✅ .AppImage: {}", app.display());
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (deb, appimage);
    }

    println!("✅ dist finalizado.");
    Ok(())
}

#[cfg(target_os = "linux")]
fn make_deb(out_dir: &std::path::Path, name: &str, bin_path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    // Layout mínimo: package_root/usr/bin/<name> + DEBIAN/control
    let root = out_dir.join(format!("{}_debroot", name));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("usr/bin")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(root.join("DEBIAN")).map_err(|e| e.to_string())?;

    let dest_bin = root.join("usr/bin").join(name);
    std::fs::copy(bin_path, &dest_bin).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest_bin).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_bin, perms).map_err(|e| e.to_string())?;
    }

    let control = format!(
        "Package: {name}\nVersion: 0.1.0\nSection: utils\nPriority: optional\nArchitecture: amd64\nMaintainer: Snask\nDescription: Snask app packaged by snask dist\n",
    );
    std::fs::write(root.join("DEBIAN/control"), control).map_err(|e| e.to_string())?;

    // dpkg-deb
    let deb_name = format!("{name}_0.1.0_amd64.deb");
    let deb_path = out_dir.join(deb_name);
    let status = Command::new("dpkg-deb")
        .arg("--build")
        .arg(&root)
        .arg(&deb_path)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("Failed to build .deb (dpkg-deb). Install `dpkg-deb` (dpkg package) and try again.".to_string());
    }
    Ok(deb_path)
}

#[cfg(target_os = "linux")]
fn make_appimage(out_dir: &std::path::Path, name: &str, bin_path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    // Layout mínimo AppDir + appimagetool.
    let tool = which("appimagetool")?;
    let appdir = out_dir.join(format!("{}.AppDir", name));
    let _ = std::fs::remove_dir_all(&appdir);
    std::fs::create_dir_all(appdir.join("usr/bin")).map_err(|e| e.to_string())?;

    let dest_bin = appdir.join("usr/bin").join(name);
    std::fs::copy(bin_path, &dest_bin).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest_bin).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_bin, perms).map_err(|e| e.to_string())?;
    }

    // AppRun simples: executa o binário
    let apprun = format!("#!/bin/sh\nHERE=\"$(dirname \"$(readlink -f \"$0\")\")\"\nexec \"$HERE/usr/bin/{name}\" \"$@\"\n");
    let apprun_path = appdir.join("AppRun");
    std::fs::write(&apprun_path, apprun).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&apprun_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&apprun_path, perms).map_err(|e| e.to_string())?;
    }

    // .desktop mínimo (sem ícone)
    let desktop = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nExec={name}\nCategories=Utility;\nTerminal=true\n",
    );
    std::fs::write(appdir.join(format!("{}.desktop", name)), desktop).map_err(|e| e.to_string())?;

    let out_path = out_dir.join(format!("{}.AppImage", name));
    let status = Command::new(tool)
        .arg(&appdir)
        .arg(&out_path)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("Failed to build AppImage (appimagetool).".to_string());
    }
    Ok(out_path)
}

fn which(cmd: &str) -> Result<String, String> {
    let out = Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {}", cmd))
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!("Comando '{}' não encontrado no PATH.", cmd));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn sps_pin_from_lock(dir: &std::path::Path, manifest: &crate::sps::SpsManifest) -> Result<(), String> {
    // Se existir snask.lock, garante que os pacotes instalados batem com sha/version do lock.
    // Se divergir: reinstala do registry (MVP).
    let lock_path = crate::sps::lockfile_path(dir);
    if !lock_path.exists() {
        return Ok(());
    }
    let lock = crate::sps::read_lockfile(dir)?;
    let registry = crate::packages::fetch_registry()?;

    for (name, dep) in &lock.dependencies {
        // se manifest não contém mais, ignora (lock pode estar velho)
        if !manifest.dependencies.contains_key(name) {
            continue;
        }

        // checa constraint antes (manifest manda)
        if let Some(pkg) = registry.packages.get(name) {
            let req = manifest.dependencies.get(name).map(|s| s.as_str()).unwrap_or("*");
            if req != "*" && req != pkg.version() {
                return Err(format!(
                    "SPS: constraint de versão não satisfeita para '{}': pedido '{}', registry '{}'",
                    name,
                    req,
                    pkg.version()
                ));
            }
        }

        let need_install = if !crate::packages::is_package_installed(name) {
            true
        } else {
            let sha = crate::packages::read_installed_package_sha256(name)?;
            sha != dep.sha256
        };
        if need_install {
            let (ver, sha, _path) = crate::packages::install_package_with_registry(name, &registry)?;
            if ver != dep.version || sha != dep.sha256 {
                return Err(format!(
                    "SPS: lockfile expects {name}@{want_ver} sha256={want_sha}, but the download produced {got_ver} sha256={got_sha}.\n\nCommon causes:\n- The package changed in the registry (new release / file modified)\n- Your lockfile is out of date\n\nHow to fix:\n- If you want the new package: `snask update {name}` and then `snask build` (regenerates snask.lock)\n- If you want to keep the lockfile: make sure the registry still has the same sha256.\n",
                    want_ver = dep.version,
                    want_sha = dep.sha256,
                    got_ver = ver,
                    got_sha = sha
                ));
            }
        }
    }
    Ok(())
}

fn format_semantic_error(e: &SemanticError) -> String {
    use crate::semantic_analyzer::SemanticError::*;
    match e {
        VariableAlreadyDeclared(name) => format!("Variable '{}' is already declared.", name),
        VariableNotFound(name) => format!("Variable '{}' not found.", name),
        FunctionAlreadyDeclared(name) => format!("Function '{}' is already declared.", name),
        FunctionNotFound(name) => format!("Function '{}' not found.", name),
        TypeMismatch { expected, found } => format!("Type mismatch: expected {:?}, found {:?}.", expected, found),
        InvalidOperation { op, type1, type2 } => {
            if let Some(t2) = type2 {
                format!("Invalid operation: '{}' between {:?} and {:?}.", op, type1, t2)
            } else {
                format!("Invalid operation: '{}' on {:?}.", op, type1)
            }
        }
        ImmutableAssignment(name) => format!("'{}' is immutable. Tip: declare it as 'mut {} = ...;'.", name, name),
        ReturnOutsideFunction => "Using 'return' outside a function.".to_string(),
        WrongNumberOfArguments { expected, found } => format!("Wrong number of arguments: expected {}, found {}.", expected, found),
        IndexAccessOnNonIndexable(t) => format!("Index access on non-indexable type: {:?}.", t),
        InvalidIndexType(t) => format!("Invalid index type: {:?} (expected num).", t),
        PropertyNotFound(p) => format!("Property '{}' not found.", p),
        NotCallable(t) => format!("Attempted to call a non-callable value: {:?}.", t),
        RestrictedNativeFunction { name, help } => format!("Direct use of native '{}' is not allowed.\n{}", name, help),
    }
}

fn self_update() -> Result<(), String> {
    println!("📦 Fetching the latest Snask updates (git pull)...");
    let status = Command::new("git").arg("pull").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Git pull failed.".to_string()); }

    println!("⚙️  Rebuilding the compiler (cargo build --release)...");
    let status = Command::new("cargo").arg("build").arg("--release").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Build failed.".to_string()); }

    println!("✅ Snask updated successfully to v0.3.0!");
    Ok(())
}

fn run_setup(target: Option<String>) -> Result<(), String> {
    println!("🚀 Starting Snask setup v0.3.0...");
    
    let home = std::env::var("HOME").map_err(|_| "Variável HOME não encontrada.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    let snask_lib = format!("{}/lib", snask_dir);
    let snask_bin = format!("{}/bin", snask_dir);
    let snask_tmp = format!("{}/tmp", snask_dir);
    
    println!("📁 Creating directories in {}...", snask_dir);
    fs::create_dir_all(&snask_lib).map_err(|e| e.to_string())?;
    fs::create_dir_all(&snask_bin).map_err(|e| e.to_string())?;
    fs::create_dir_all(&snask_tmp).map_err(|e| e.to_string())?;

    println!("⚙️  Building native runtime (C)...");
    let (_runtime_dir, runtime_out, linkargs_path) = if let Some(t) = &target {
        let dir = format!("{}/{}", snask_lib, t);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        (dir.clone(), format!("{}/runtime.o", dir), format!("{}/runtime.linkargs", dir))
    } else {
        (snask_lib.clone(), format!("{}/runtime.o", snask_lib), format!("{}/runtime.linkargs", snask_lib))
    };

    // Sempre usa o runtime embutido (setup funciona em qualquer diretório, mesmo fora do repo).
    // `runtime.c` inclui `runtime/runtime_old.c`, então gravamos ambos em ~/.snask/tmp.
    let tmp_dir = std::path::PathBuf::from(&snask_tmp);
    let runtime_dir = tmp_dir.join("runtime");
    fs::create_dir_all(&runtime_dir).map_err(|e| e.to_string())?;

    let runtime_c_path = tmp_dir.join("runtime.c");
    fs::write(&runtime_c_path, EMBEDDED_RUNTIME_C).map_err(|e| e.to_string())?;
    fs::write(runtime_dir.join("runtime_old.c"), EMBEDDED_RUNTIME_OLD).map_err(|e| e.to_string())?;

    // Link args requeridos pelo runtime (persistidos para `snask build`)
    let mut runtime_linkargs: Vec<String> = vec!["-pthread".to_string()];

    // Compilador: nativo usa gcc; cross usa clang-18 --target.
    let mut cc = if target.is_some() { Command::new("clang-18") } else { Command::new("gcc") };
    cc.arg("-c")
        .arg(&runtime_c_path)
        .arg("-ffunction-sections")
        .arg("-fdata-sections")
        .arg("-pthread")
        .arg("-o")
        .arg(&runtime_out);

        if let Some(t) = &target {
            // Para cross-compilation, não tentamos ativar GTK/SQLite via pkg-config (seria do host).
            cc.arg(format!("--target={}", t));
            println!("🎯 Cross: runtime target = {}", t);
        // Persistimos apenas pthread por padrão.
    } else {
        // GUI opcional (GTK3)
        let gtk_cflags = Command::new("pkg-config")
            .arg("--cflags")
            .arg("gtk+-3.0")
            .output();
        if let Ok(out) = gtk_cflags {
            if out.status.success() {
                let cflags = String::from_utf8_lossy(&out.stdout);
                for f in cflags.split_whitespace() {
                    cc.arg(f);
                }
                cc.arg("-DSNASK_GUI_GTK");
                println!("🖼️  GUI: GTK3 enabled (runtime).");

                if let Ok(libs) = Command::new("pkg-config").arg("--libs").arg("gtk+-3.0").output() {
                    if libs.status.success() {
                        runtime_linkargs.extend(String::from_utf8_lossy(&libs.stdout).split_whitespace().map(|s| s.to_string()));
                    }
                }
            } else {
                println!("ℹ️  GUI: GTK3 not found via pkg-config (runtime without GUI).");
            }
        } else {
            println!("ℹ️  GUI: pkg-config not found (runtime without GUI).");
        }

        // SQLite opcional
        let sqlite_cflags = Command::new("pkg-config")
            .arg("--cflags")
            .arg("sqlite3")
            .output();
        if let Ok(out) = sqlite_cflags {
            if out.status.success() {
                let cflags = String::from_utf8_lossy(&out.stdout);
                for f in cflags.split_whitespace() {
                    cc.arg(f);
                }
                cc.arg("-DSNASK_SQLITE");
                println!("🗄️  SQLite: sqlite3 enabled (runtime).");

                if let Ok(libs) = Command::new("pkg-config").arg("--libs").arg("sqlite3").output() {
                    if libs.status.success() {
                        runtime_linkargs.extend(String::from_utf8_lossy(&libs.stdout).split_whitespace().map(|s| s.to_string()));
                    }
                }
            } else {
                println!("ℹ️  SQLite: sqlite3 not found via pkg-config (runtime without SQLite).");
            }
        } else {
            println!("ℹ️  SQLite: pkg-config not found (runtime without SQLite).");
        }
    }

    let status = cc.status().map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(if target.is_some() {
            "Failed to compile runtime.c (cross). Check your target toolchain/headers and whether clang-18 supports this --target.".to_string()
        } else {
            "Failed to compile runtime.c. Make sure gcc is installed.".to_string()
        });
    }

    let linkargs = runtime_linkargs.join(" ");
    fs::write(&linkargs_path, linkargs).map_err(|e| e.to_string())?;

    println!("🚚 Installing binary into {}...", snask_bin);
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dest_path = std::path::PathBuf::from(&snask_bin).join("snask");
    install_self_binary(&current_exe, &dest_path)?;

    println!("🌐 Configuring PATH...");
    let shell_configs = vec![format!("{}/.bashrc", home), format!("{}/.zshrc", home)];
    let path_line = format!("\n# SNask Language\nexport PATH=\"{}:$PATH\"\n", snask_bin);
    
    for config_path in shell_configs {
        if std::path::Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            if !content.contains("SNask Language") {
                use std::io::Write;
                let mut file = fs::OpenOptions::new().append(true).open(&config_path).map_err(|e| e.to_string())?;
                file.write_all(path_line.as_bytes()).map_err(|e| e.to_string())?;
                println!("✅ PATH added to {}", config_path);
            }
        }
    }

    println!("✅ Snask v0.3.0 setup complete!");
    println!("Tip: restart your terminal or run 'source ~/.bashrc' to use the 'snask' command anywhere.");
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn kill_processes_using_exe_path(dest_path: &std::path::Path) -> Result<usize, String> {
    let mut killed = 0usize;
    let me = std::process::id() as i32;
    let dest = dest_path.to_path_buf();

    let proc = std::path::Path::new("/proc");
    if !proc.exists() {
        return Ok(0);
    }

    for entry in std::fs::read_dir(proc).map_err(|e| e.to_string())? {
        let Ok(entry) = entry else { continue };
        let fname = entry.file_name();
        let Ok(pid_str) = fname.to_string_lossy().parse::<u32>() else { continue };
        let pid = pid_str as i32;
        if pid == me { continue; }

        let exe_link = entry.path().join("exe");
        let Ok(exe) = std::fs::read_link(&exe_link) else { continue };
        if exe != dest { continue; }

        unsafe {
            libc::kill(pid, libc::SIGTERM);
        }
        killed += 1;
    }

    if killed == 0 {
        return Ok(0);
    }

    let deadline = Instant::now() + Duration::from_millis(800);
    while Instant::now() < deadline {
        let mut any_alive = false;
        for entry in std::fs::read_dir(proc).map_err(|e| e.to_string())? {
            let Ok(entry) = entry else { continue };
            let fname = entry.file_name();
            let Ok(pid_str) = fname.to_string_lossy().parse::<u32>() else { continue };
            let pid = pid_str as i32;
            if pid == me { continue; }
            let exe_link = entry.path().join("exe");
            let Ok(exe) = std::fs::read_link(&exe_link) else { continue };
            if exe == dest { any_alive = true; break; }
        }
        if !any_alive { break; }
        std::thread::sleep(Duration::from_millis(50));
    }

    for entry in std::fs::read_dir(proc).map_err(|e| e.to_string())? {
        let Ok(entry) = entry else { continue };
        let fname = entry.file_name();
        let Ok(pid_str) = fname.to_string_lossy().parse::<u32>() else { continue };
        let pid = pid_str as i32;
        if pid == me { continue; }
        let exe_link = entry.path().join("exe");
        let Ok(exe) = std::fs::read_link(&exe_link) else { continue };
        if exe != dest { continue; }
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }

    Ok(killed)
}

#[cfg(not(target_os = "linux"))]
fn kill_processes_using_exe_path(_dest_path: &std::path::Path) -> Result<usize, String> {
    Ok(0)
}

fn install_self_binary(current_exe: &std::path::Path, dest_path: &std::path::Path) -> Result<(), String> {
    let tmp_path = dest_path.with_extension("tmp");
    let _ = std::fs::remove_file(&tmp_path);

    std::fs::copy(current_exe, &tmp_path).map_err(|e| format!("Failed to copy binary: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&tmp_path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&tmp_path, perms).map_err(|e| e.to_string())?;
    }

    match std::fs::rename(&tmp_path, dest_path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = kill_processes_using_exe_path(dest_path);
            std::fs::rename(&tmp_path, dest_path).map_err(|e2| format!("Setup error: {} (orig: {})", e2, e))?;
            Ok(())
        }
    }
}

fn run_uninstall() -> Result<(), String> {
    println!("🗑️  Uninstalling Snask v0.3.0...");
    
    let home = std::env::var("HOME").map_err(|_| "Variável HOME não encontrada.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    
    if std::path::Path::new(&snask_dir).exists() {
        fs::remove_dir_all(&snask_dir).map_err(|e| e.to_string())?;
        println!("✅ Removed directory {}.", snask_dir);
    }

    println!("✅ Snask uninstalled successfully!");
    println!("Note: to fully remove PATH changes, edit your .bashrc/.zshrc and remove the Snask lines manually.");
    
    Ok(())
}

fn resolve_imports(program: &mut Program, resolved_program: &mut Program, resolved_modules: &mut std::collections::HashSet<String>) -> Result<(), String> {
    fn rewrite_expr_native_alias(e: &mut crate::ast::Expr) {
        use crate::ast::ExprKind;
        fn should_alias(name: &str) -> bool {
            matches!(name,
                // SFS / Path / OS / HTTP
                "sfs_read"|"sfs_write"|"sfs_append"|"sfs_delete"|"sfs_exists"|"sfs_copy"|"sfs_move"|"sfs_mkdir"|"sfs_is_file"|"sfs_is_dir"|"sfs_listdir"|"sfs_size"|"sfs_mtime"|"sfs_rmdir"|
                "path_basename"|"path_dirname"|"path_extname"|"path_join"|
                "os_cwd"|"os_platform"|"os_arch"|"os_getenv"|"os_setenv"|"os_random_hex"|
                "s_http_get"|"s_http_post"|"s_http_put"|"s_http_delete"|"s_http_patch"|
                // Blaze / Auth
                "blaze_run"|"blaze_qs_get"|"blaze_cookie_get"|
                "auth_random_hex"|"auth_now"|"auth_const_time_eq"|"auth_hash_password"|"auth_verify_password"|"auth_session_id"|"auth_csrf_token"|"auth_cookie_kv"|"auth_cookie_session"|"auth_cookie_delete"|"auth_bearer_header"|"auth_ok"|"auth_fail"|"auth_version"|
                // GUI
                "gui_init"|"gui_run"|"gui_quit"|"gui_window"|"gui_set_title"|"gui_set_resizable"|"gui_autosize"|"gui_vbox"|"gui_hbox"|"gui_scrolled"|
                "gui_flowbox"|"gui_flow_add"|"gui_frame"|"gui_set_margin"|"gui_icon"|"gui_css"|"gui_add_class"|"gui_eventbox"|
                "gui_listbox"|"gui_list_add_text"|"gui_on_select_ctx"|"gui_set_child"|"gui_add"|"gui_add_expand"|"gui_label"|"gui_entry"|"gui_textview"|"gui_set_placeholder"|"gui_set_editable"|
                "gui_button"|"gui_set_enabled"|"gui_set_visible"|"gui_show_all"|"gui_set_text"|"gui_get_text"|"gui_on_click"|"gui_on_click_ctx"|"gui_on_tap_ctx"|"gui_separator_h"|"gui_separator_v"|"gui_msg_info"|"gui_msg_error"|
                // SQLite
                "sqlite_open"|"sqlite_close"|"sqlite_exec"|"sqlite_query"|"sqlite_prepare"|"sqlite_finalize"|"sqlite_reset"|"sqlite_bind_text"|"sqlite_bind_num"|"sqlite_bind_null"|"sqlite_step"|"sqlite_column"|"sqlite_column_count"|"sqlite_column_name"|
                // Threads
                "thread_spawn"|"thread_join"|"thread_detach"|
                // JSON / SJSON
                "json_stringify"|"json_stringify_pretty"|"json_parse"|"json_get"|"json_has"|"json_len"|"json_index"|"json_set"|"json_keys"|"json_parse_ex"|
                "snif_new_object"|"snif_new_array"|"snif_parse_ex"|"snif_type"|"snif_arr_len"|"snif_arr_get"|"snif_arr_set"|"snif_arr_push"|"snif_path_get"|
                // String extras
                "string_len"|"string_upper"|"string_lower"|"string_trim"|"string_split"|"string_join"|"string_replace"|"string_contains"|"string_starts_with"|"string_ends_with"|
                "string_chars"|"string_substring"|"string_format"|"string_index_of"|"string_last_index_of"|"string_repeat"|"string_is_empty"|"string_is_blank"|
                "string_pad_start"|"string_pad_end"|"string_capitalize"|"string_title"|"string_swapcase"|"string_count"|"string_is_numeric"|"string_is_alpha"|
                "string_is_alphanumeric"|"string_is_ascii"|"string_hex"|"string_from_char_code"|"string_to_char_code"|"string_reverse"
            )
        }

        match &mut e.kind {
            ExprKind::Variable(name) => {
                if should_alias(name.as_str()) {
                    *name = format!("__{}", name);
                }
            }
            ExprKind::Unary { expr, .. } => rewrite_expr_native_alias(expr),
            ExprKind::Binary { left, right, .. } => { rewrite_expr_native_alias(left); rewrite_expr_native_alias(right); }
            ExprKind::FunctionCall { callee, args } => {
                rewrite_expr_native_alias(callee);
                for a in args { rewrite_expr_native_alias(a); }
            }
            ExprKind::PropertyAccess { target, .. } => rewrite_expr_native_alias(target),
            ExprKind::IndexAccess { target, index } => { rewrite_expr_native_alias(target); rewrite_expr_native_alias(index); }
            ExprKind::Literal(_) => {}
        }
    }

    fn rewrite_stmt_native_alias(s: &mut crate::ast::Stmt) {
        use crate::ast::{StmtKind, LoopStmt};
        match &mut s.kind {
            StmtKind::Expression(e) | StmtKind::FuncCall(e) | StmtKind::Return(e) => rewrite_expr_native_alias(e),
            StmtKind::VarDeclaration(v) => rewrite_expr_native_alias(&mut v.value),
            StmtKind::MutDeclaration(v) => rewrite_expr_native_alias(&mut v.value),
            StmtKind::ConstDeclaration(v) => rewrite_expr_native_alias(&mut v.value),
            StmtKind::VarAssignment(v) => rewrite_expr_native_alias(&mut v.value),
            StmtKind::Print(es) => { for e in es { rewrite_expr_native_alias(e); } }
            StmtKind::Conditional(c) => {
                rewrite_expr_native_alias(&mut c.if_block.condition);
                for st in &mut c.if_block.body { rewrite_stmt_native_alias(st); }
                for b in &mut c.elif_blocks {
                    rewrite_expr_native_alias(&mut b.condition);
                    for st in &mut b.body { rewrite_stmt_native_alias(st); }
                }
                if let Some(else_b) = &mut c.else_block {
                    for st in else_b { rewrite_stmt_native_alias(st); }
                }
            }
            StmtKind::Loop(l) => match l {
                LoopStmt::While { condition, body } => {
                    rewrite_expr_native_alias(condition);
                    for st in body { rewrite_stmt_native_alias(st); }
                }
                LoopStmt::For { iterable, body, .. } => {
                    rewrite_expr_native_alias(iterable);
                    for st in body { rewrite_stmt_native_alias(st); }
                }
            },
            StmtKind::ListDeclaration(d) => rewrite_expr_native_alias(&mut d.value),
            StmtKind::ListPush(p) => rewrite_expr_native_alias(&mut p.value),
            StmtKind::DictDeclaration(d) => rewrite_expr_native_alias(&mut d.value),
            StmtKind::DictSet(d) => { rewrite_expr_native_alias(&mut d.key); rewrite_expr_native_alias(&mut d.value); }
            StmtKind::FuncDeclaration(f) => { for st in &mut f.body { rewrite_stmt_native_alias(st); } }
            StmtKind::ClassDeclaration(c) => {
                for p in &mut c.properties { rewrite_expr_native_alias(&mut p.value); }
                for m in &mut c.methods { for st in &mut m.body { rewrite_stmt_native_alias(st); } }
            }
            StmtKind::Input { .. } => {}
            StmtKind::Import(_) => {}
        }
    }

    for stmt in program.drain(..) {
        if let StmtKind::Import(path) = &stmt.kind {
            if !resolved_modules.contains(path) {
                resolved_modules.insert(path.clone());
                let mut module_ast = load_module(path)?;
                for st in &mut module_ast {
                    rewrite_stmt_native_alias(st);
                }
                
                // Extrai o nome do módulo (sem extensão e sem diretório)
                let module_name = std::path::Path::new(path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

                // Renomeia funções do módulo para incluir o namespace.
                // Exceção: "prelude" é pensado para ser importado e usado sem prefixos (ergonomia).
                if module_name != "prelude" {
                    for m_stmt in &mut module_ast {
                        if let StmtKind::FuncDeclaration(f) = &mut m_stmt.kind {
                            f.name = format!("{}::{}", module_name, f.name);
                        }
                    }
                }

                resolve_imports(&mut module_ast, resolved_program, resolved_modules)?;
            }
        } else { resolved_program.push(stmt); }
    }
    Ok(())
}
