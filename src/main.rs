use std::fs;
use std::process::Command;
use clap::{Parser as ClapParser, Subcommand};
use snask::parser::Parser;
use snask::semantic_analyzer::{SemanticAnalyzer, SemanticError};
use snask::llvm_generator::LLVMGenerator;
use snask::packages;
use inkwell::context::Context;
use snask::ast::{Program, StmtKind};
use snask::diagnostics::{Annotation, Diagnostic, DiagnosticBag};
// spans are carried by diagnostics/errors; main doesn't need direct span types here.
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
        /// Installs into ~/.local (Linux only): binary + desktop entry + icon when available
        #[arg(long)]
        linux_user: bool,
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
    /// Installs optional components (e.g. `skia` backend)
    InstallOptional { name: String },
    List,
    Search { query: String },
    /// Checks your Snask installation and environment (toolchain, runtime, registry).
    Doctor,
    /// Tools to create/publish Snask libraries
    Lib {
        #[command(subcommand)]
        cmd: LibCommands,
    },
    /// SNIF tooling (formatter, checker, schema, hashing)
    Snif {
        #[command(subcommand)]
        cmd: SnifCommands,
        /// Optional file path (default: snask.snif when present)
        file: Option<String>,
        /// Write changes (only for `fmt`)
        #[arg(long)]
        write: bool,
        /// Print formatted output to stdout
        #[arg(long)]
        stdout: bool,
        /// Exit non-zero if formatting would change
        #[arg(long)]
        check: bool,
        /// Strict mode (reserved)
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Subcommand)]
enum LibCommands {
    /// Creates a library template in the current directory
    Init {
        name: String,
        #[arg(long, default_value = "0.1.0")]
        version: String,
        #[arg(long, default_value = "My Snask library.")]
        description: String,
    },
    /// Publishes the current library to the registry (SnaskPackages via ~/.snask/registry)
    Publish {
        name: String,
        /// If omitted, uses package.json
        #[arg(long)]
        version: Option<String>,
        /// If omitted, uses package.json
        #[arg(long)]
        description: Option<String>,
        /// Commit message
        #[arg(long)]
        message: Option<String>,
        /// Runs git push automatically
        #[arg(long)]
        push: bool,
        /// Publishes via fork + Pull Request (does not require write access)
        #[arg(long)]
        pr: bool,
        /// Your fork URL (e.g. https://github.com/you/SnaskPackages)
        #[arg(long)]
        fork: Option<String>,
        /// Branch name (default: pkg/<name>-v<version>)
        #[arg(long)]
        branch: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
enum SnifCommands {
    /// Canonical formatter
    Fmt,
    /// Validate SNIF syntax and (for snask.snif) schema
    Check,
    /// Print the snask.snif schema
    Schema {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        md: bool,
    },
    /// Output canonical SNIF to stdout
    Canon,
    /// Print sha256 of canonical SNIF bytes
    Hash,
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
        Commands::Dist { file, targets, all, deb, appimage, name, linux_user, out_dir } => {
            if let Err(e) = dist_entry(file.clone(), targets.clone(), *all, *deb, *appimage, name.clone(), *linux_user, out_dir.clone()) {
                eprintln!("Error: {}", e);
            }
        }
        Commands::Run { file } => {
            // scripts: `snask run dev`
            if let Some(arg) = file.clone() {
                if !arg.ends_with(".snask") {
                    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    if let Ok((m, _p)) = snask::sps::load_manifest_from(&cwd) {
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
            let file_path = match snask::sps::load_manifest_from(&cwd) {
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
            let build_res = if let Ok((m, _p)) = snask::sps::load_manifest_from(&cwd) {
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
        Commands::InstallOptional { name } => {
            if let Err(e) = install_optional(name) {
                eprintln!("Error: {}", e);
            }
        }
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
        Commands::Doctor => {
            if let Err(e) = doctor() {
                eprintln!("Doctor error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Lib { cmd } => {
            match cmd {
                LibCommands::Init { name, version, description } => {
                    let res = snask::lib_tool::lib_init(snask::lib_tool::NewLibOpts {
                        name: name.clone(),
                        description: description.clone(),
                        version: version.clone(),
                    });
                    if let Err(e) = res {
                        eprintln!("Error: {}", e);
                    }
                }
                LibCommands::Publish { name, version, description, message, push, pr, fork, branch } => {
                    let res = snask::lib_tool::lib_publish(snask::lib_tool::PublishOpts {
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
        Commands::Snif { cmd, file, write, stdout, check, strict: _ } => {
            match run_snif_cmd(cmd.clone(), file.clone(), *write, *stdout, *check) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("{}", e.message);
                    std::process::exit(e.code);
                }
            }
        }
    }
}

struct SnifCmdError {
    code: i32,
    message: String,
}

impl SnifCmdError {
    fn parse(message: String) -> Self {
        SnifCmdError { code: 1, message }
    }
    fn schema(message: String) -> Self {
        SnifCmdError { code: 2, message }
    }
}

fn run_snif_cmd(cmd: SnifCommands, file: Option<String>, write: bool, stdout: bool, check: bool) -> Result<(), SnifCmdError> {
    use snask::snif_tooling;
    use snask::snif_fmt::format_snif;
    use snask::snif_parser::parse_snif;
    use snask::snif_schema::{snask_manifest_schema_md, validate_snask_manifest};

    let cwd = std::env::current_dir().map_err(|e| SnifCmdError::parse(e.to_string()))?;
    let path = if let Some(f) = file {
        std::path::PathBuf::from(f)
    } else {
        snif_tooling::default_snask_snif_path(&cwd).ok_or_else(|| {
            SnifCmdError::parse(
                "SNIF: no file provided and `snask.snif` was not found in the current directory.\n\nHow to fix:\n- Run: `snask snif fmt snask.snif --write`\n- Or pass a file path: `snask snif check path/to/file.snif`\n".to_string(),
            )
        })?
    };
    let src = snif_tooling::read_snif_file(&path).map_err(SnifCmdError::parse)?;

    let is_manifest = path.file_name().and_then(|s| s.to_str()) == Some("snask.snif");

    match cmd {
        SnifCommands::Schema { json, md } => {
            if json {
                // keep it simple for now: no JSON schema object; emit markdown wrapped.
                let s = snask_manifest_schema_md();
                println!("{}", serde_json::json!({ "format": "markdown", "schema": s }).to_string());
                return Ok(());
            }
            let _ = md; // md is default
            println!("{}", snask_manifest_schema_md());
            Ok(())
        }
        SnifCommands::Canon => {
            let v = parse_snif(&src).map_err(|e| {
                SnifCmdError::parse(snif_tooling::render_snif_parse_diagnostic(
                    path.to_string_lossy().as_ref(),
                    &src,
                    &e,
                ))
            })?;
            print!("{}", format_snif(&v));
            Ok(())
        }
        SnifCommands::Hash => {
            let (canon, sha) = snif_tooling::snif_canon_and_hash(&src)
                .map_err(|e| {
                    SnifCmdError::parse(snif_tooling::render_snif_parse_diagnostic(
                        path.to_string_lossy().as_ref(),
                        &src,
                        &e,
                    ))
                })?;
            if stdout {
                print!("{}", canon);
            }
            println!("{}", sha);
            Ok(())
        }
        SnifCommands::Check => {
            let v = parse_snif(&src).map_err(|e| {
                SnifCmdError::parse(snif_tooling::render_snif_parse_diagnostic(
                    path.to_string_lossy().as_ref(),
                    &src,
                    &e,
                ))
            })?;
            if is_manifest {
                let errs = validate_snask_manifest(&v);
                if !errs.is_empty() {
                    return Err(SnifCmdError::schema(snif_tooling::render_snif_schema_diagnostic(
                        path.to_string_lossy().as_ref(),
                        &src,
                        &errs,
                    )));
                }
            }
            Ok(())
        }
        SnifCommands::Fmt => {
            let v = parse_snif(&src).map_err(|e| {
                SnifCmdError::parse(snif_tooling::render_snif_parse_diagnostic(
                    path.to_string_lossy().as_ref(),
                    &src,
                    &e,
                ))
            })?;
            if is_manifest {
                let errs = validate_snask_manifest(&v);
                if !errs.is_empty() {
                    return Err(SnifCmdError::schema(snif_tooling::render_snif_schema_diagnostic(
                        path.to_string_lossy().as_ref(),
                        &src,
                        &errs,
                    )));
                }
            }
            let formatted = format_snif(&v);

            let default_write = is_manifest;
            let do_write = if stdout || check { false } else if write { true } else { default_write };

            if check && formatted != src {
                return Err(SnifCmdError::schema(
                    "SNIF: file is not formatted (run `snask snif fmt --write`).".to_string(),
                ));
            }

            if stdout || !do_write {
                print!("{}", formatted);
                return Ok(());
            }

            std::fs::write(&path, formatted)
                .map_err(|e| SnifCmdError::parse(format!("SNIF: failed to write {}: {}", path.display(), e)))?;
            Ok(())
        }
    }
}

fn doctor() -> Result<(), String> {
    println!("Snask Doctor (v0.3.1)\n");

    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found.".to_string())?;
    let snask_dir = std::path::PathBuf::from(format!("{}/.snask", home));
    let snask_lib = snask_dir.join("lib");
    let snask_bin = snask_dir.join("bin");
    let snask_registry = snask_dir.join("registry");
    let snask_packages = snask_dir.join("packages");

    println!("[paths]");
    println!("- home: {}", home);
    println!("- snask: {}", snask_dir.display());
    println!("- lib: {}", snask_lib.display());
    println!("- bin: {}", snask_bin.display());
    println!("- registry: {}", snask_registry.display());
    println!("- packages: {}", snask_packages.display());
    println!();

    fn check_cmd(cmd: &str, args: &[&str]) -> (bool, String) {
        let out = Command::new(cmd).args(args).output();
        match out {
            Err(e) => (false, format!("missing ({})", e)),
            Ok(o) => {
                if !o.status.success() {
                    return (false, "present but failed".to_string());
                }
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                (true, if s.is_empty() { "ok".to_string() } else { s })
            }
        }
    }

    println!("[toolchain]");
    let (ok_clang, clang_v) = check_cmd("clang-18", &["--version"]);
    println!(
        "- clang-18: {}{}",
        if ok_clang { "OK" } else { "FAIL" },
        if ok_clang {
            format!(" ({})", clang_v.lines().next().unwrap_or(""))
        } else {
            format!(" ({})", clang_v)
        }
    );
    let (ok_llc, llc_v) = check_cmd("llc-18", &["--version"]);
    println!(
        "- llc-18: {}{}",
        if ok_llc { "OK" } else { "FAIL" },
        if ok_llc {
            format!(" ({})", llc_v.lines().next().unwrap_or(""))
        } else {
            format!(" ({})", llc_v)
        }
    );
    let (ok_gcc, gcc_v) = check_cmd("gcc", &["--version"]);
    println!(
        "- gcc: {}{}",
        if ok_gcc { "OK" } else { "FAIL" },
        if ok_gcc {
            format!(" ({})", gcc_v.lines().next().unwrap_or(""))
        } else {
            format!(" ({})", gcc_v)
        }
    );
    let (ok_git, git_v) = check_cmd("git", &["--version"]);
    println!(
        "- git: {}{}",
        if ok_git { "OK" } else { "FAIL" },
        format!(" ({})", git_v)
    );
    let (ok_pkgcfg, pkgcfg_v) = check_cmd("pkg-config", &["--version"]);
    println!(
        "- pkg-config: {}{}",
        if ok_pkgcfg { "OK" } else { "FAIL" },
        if ok_pkgcfg { format!(" ({})", pkgcfg_v) } else { format!(" ({})", pkgcfg_v) }
    );
    println!();

    println!("[native deps]");
    if ok_pkgcfg {
        let (ok_gtk, _) = check_cmd("pkg-config", &["--exists", "gtk+-3.0"]);
        println!(
            "- gtk+-3.0 (GUI): {}{}",
            if ok_gtk { "OK" } else { "MISSING" },
            if ok_gtk {
                "".to_string()
            } else {
                " (install: sudo apt install -y libgtk-3-dev gir1.2-gtk-3.0)".to_string()
            }
        );
        let (ok_sqlite, _) = check_cmd("pkg-config", &["--exists", "sqlite3"]);
        println!(
            "- sqlite3: {}{}",
            if ok_sqlite { "OK" } else { "MISSING" },
            if ok_sqlite { "".to_string() } else { " (install: sudo apt install -y libsqlite3-dev)".to_string() }
        );
    } else {
        println!("- pkg-config not available; cannot verify gtk/sqlite headers.");
    }
    println!();

    println!("[runtime]");
    let runtime_o = snask_lib.join("runtime.o");
    let linkargs = snask_lib.join("runtime.linkargs");
    println!("- runtime.o: {}", if runtime_o.exists() { "OK" } else { "MISSING (run: snask setup)" });
    println!("- runtime.linkargs: {}", if linkargs.exists() { "OK" } else { "MISSING (run: snask setup)" });
    println!();

    println!("[registry]");
    if snask_registry.exists() {
        let git_dir = snask_registry.join(".git");
        if git_dir.exists() {
            let out = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&snask_registry)
                .output();
            match out {
                Ok(o) if o.status.success() => {
                    let rev = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    println!("- repo: OK (rev {})", rev);
                }
                _ => println!("- repo: OK (git rev unknown)"),
            }
        } else {
            println!("- repo: OK (not a git repo?)");
        }
    } else {
        println!("- repo: MISSING (will be created on first registry fetch)");
    }
    let registry_json = snask_registry.join("registry.json");
    println!(
        "- registry.json: {}",
        if registry_json.exists() {
            "OK"
        } else {
            "MISSING (run: snask update / snask install <pkg>)"
        }
    );
    println!();

    println!("[dist tooling]");
    let (ok_appimagetool, _) = check_cmd("appimagetool", &["--version"]);
    println!(
        "- appimagetool (.AppImage): {}{}",
        if ok_appimagetool { "OK" } else { "MISSING" },
        if ok_appimagetool { "".to_string() } else { " (optional; needed for `snask dist --appimage`)".to_string() }
    );
    let (ok_fpm, _) = check_cmd("fpm", &["--version"]);
    println!(
        "- fpm (.deb): {}{}",
        if ok_fpm { "OK" } else { "MISSING" },
        if ok_fpm { "".to_string() } else { " (optional; needed for `snask dist --deb`)".to_string() }
    );
    println!();

    println!("[packages]");
    if snask_packages.exists() {
        let count = std::fs::read_dir(&snask_packages).map(|rd| rd.filter(|e| e.is_ok()).count()).unwrap_or(0);
        println!("- installed packages dir: OK (entries {})", count);
    } else {
        println!("- installed packages dir: MISSING");
    }
    println!();

    println!("[summary]");
    let mut critical_fail = false;
    if !ok_clang || !ok_llc {
        critical_fail = true;
        println!("- FAIL: LLVM toolchain missing. Install clang-18 + llc-18.");
    }
    if !runtime_o.exists() {
        critical_fail = true;
        println!("- FAIL: runtime missing. Run: snask setup");
    }
    if critical_fail {
        println!("- STATUS: FAIL");
        return Err("Doctor found critical issues.".to_string());
    }
    println!("- STATUS: OK");
    println!("- OK: core toolchain + runtime look good.");
    Ok(())
}

fn install_optional(name: &str) -> Result<(), String> {
    match name {
        "skia" => install_optional_skia(),
        _ => Err(format!("Unknown optional component '{}'. Supported: skia", name)),
    }
}

fn install_optional_skia() -> Result<(), String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found.".to_string())?;
    let base = std::path::PathBuf::from(format!("{}/.snask/optional", home));
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;

    let depot_tools_dir = base.join("depot_tools");
    let skia_dir = base.join("skia");

    println!("[optional] installing skia (this can take a while)");
    println!("- base: {}", base.display());

    if !depot_tools_dir.exists() {
        println!("[optional] cloning depot_tools...");
        let st = Command::new("git")
            .args(["clone", "--depth=1", "https://chromium.googlesource.com/chromium/tools/depot_tools.git"])
            .arg(&depot_tools_dir)
            .status()
            .map_err(|e| e.to_string())?;
        if !st.success() {
            return Err("Failed to clone depot_tools (git).".to_string());
        }
    } else {
        println!("[optional] depot_tools already present.");
    }

    // Prepare PATH for depot_tools commands (fetch, gclient, gn, ninja, etc.)
    let old_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", depot_tools_dir.display(), old_path);

    if !skia_dir.exists() {
        println!("[optional] fetching skia sources...");
        let st = Command::new("fetch")
            .arg("skia")
            .current_dir(&base)
            .env("PATH", &new_path)
            .status()
            .map_err(|e| e.to_string())?;
        if !st.success() {
            return Err("Failed to fetch skia (requires depot_tools).".to_string());
        }
    } else {
        println!("[optional] skia sources already present.");
    }

    println!("[optional] syncing deps (gclient sync)...");
    let st = Command::new("gclient")
        .arg("sync")
        .current_dir(&skia_dir)
        .env("PATH", &new_path)
        .status()
        .map_err(|e| e.to_string())?;
    if !st.success() {
        return Err("Failed to run gclient sync for skia.".to_string());
    }

    println!("[optional] building skia (gn + ninja)...");
    let st = Command::new("gn")
        .args(["gen", "out/snask"])
        .arg("--args=is_official_build=true is_component_build=false")
        .current_dir(&skia_dir)
        .env("PATH", &new_path)
        .status()
        .map_err(|e| e.to_string())?;
    if !st.success() {
        return Err("Failed to run gn gen for skia.".to_string());
    }

    let st = Command::new("ninja")
        .args(["-C", "out/snask", "skia", "skia_encode"])
        .current_dir(&skia_dir)
        .env("PATH", &new_path)
        .status()
        .map_err(|e| e.to_string())?;
    if !st.success() {
        return Err("Failed to build skia (ninja).".to_string());
    }

    // Generate a pkg-config file so `snask setup` can detect Skia.
    let pc_dir = base.join("pkgconfig");
    fs::create_dir_all(&pc_dir).map_err(|e| e.to_string())?;
    let pc_path = pc_dir.join("skia.pc");

    let include_dir = skia_dir.join("include");
    let lib_dir = skia_dir.join("out/snask");
    let pc = format!(
        "prefix={}\nexec_prefix=${{prefix}}\nlibdir={}\nincludedir={}\n\nName: skia\nDescription: Skia 2D graphics library (Snask optional)\nVersion: 0.0.0\nCflags: -I${{includedir}}\nLibs: -L${{libdir}} -lskia\n",
        skia_dir.display(),
        lib_dir.display(),
        include_dir.display()
    );
    fs::write(&pc_path, pc).map_err(|e| e.to_string())?;

    println!("[optional] done.");
    println!("- pkg-config file: {}", pc_path.display());
    println!();
    println!("Next:");
    println!("- Export PKG_CONFIG_PATH so snask can find Skia:");
    println!("  export PKG_CONFIG_PATH=\"{}:$PKG_CONFIG_PATH\"", pc_dir.display());
    println!("- Then run: snask setup");
    Ok(())
}

fn resolve_entry_file(cli_file: Option<String>) -> Result<String, String> {
    if let Some(f) = cli_file {
        return Ok(f);
    }

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if snask::sps::find_manifest(&cwd).is_some() {
        let (m, _p) = snask::sps::load_manifest_from(&cwd)?;
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
    if snask::sps::find_manifest(&cwd).is_some() {
        let (m, _p) = snask::sps::load_manifest_from(&cwd)?;
        spinner.set_message(format!("SPS: snask.snif (entry: {})", m.package.entry));
        // pin pelo lock (se existir) antes de resolver
        spinner.set_message("SPS(lock): checking snask.lock".to_string());
        sps_pin_from_lock(&cwd, &m)?;
        spinner.set_message("SPS(deps): resolving dependencies".to_string());
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
    pb.set_message("Reading file");
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    pb.inc(1);

    pb.set_message("Parser (tokens/AST)");
    let mut program = match Parser::new(&source).and_then(|mut p| p.parse_program()) {
        Ok(p) => p,
        Err(e) => {
            pb.finish_and_clear();
            return Err(render_parser_diagnostic(file_path, &source, &e));
        }
    };
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
        return Err(
            "Error: every Snask program must contain a `class main` with a `fun start()` entrypoint.\n\nHow to fix:\n- Add:\n  class main\n      fun start()\n          print(\"hello\");\n"
                .to_string(),
        );
    }

    pb.set_message("Resolving imports");
    let mut resolved_program = Vec::new();
    let mut resolved_modules = std::collections::HashSet::new();
    resolved_modules.insert(file_path.to_string());
    let entry_dir = std::path::Path::new(file_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    resolve_imports(&mut program, entry_dir, &mut resolved_program, &mut resolved_modules)?;
    pb.inc(1);

    pb.set_message("Semantic analysis");
    let mut analyzer = SemanticAnalyzer::new();
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

    let ir_file = "temp_snask.ll";
    let obj_file = "temp_snask.o";
    fs::write(ir_file, ir).map_err(|e| e.to_string())?;

    pb.set_message(format!("Compiling (llc-18 -O{})", opt_level));
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

    pb.set_message("Linking (clang-18)");
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let runtime_path = if let Some(t) = &target {
        format!("{}/.snask/lib/{}/runtime.o", home, t)
    } else {
        format!("{}/.snask/lib/runtime.o", home)
    };
    if !std::path::Path::new(&runtime_path).exists() {
        return Err(format!(
            "Runtime not found at '{}'.\n\nHow to fix:\n- Run: `snask setup{}`\n",
            runtime_path,
            target
                .as_ref()
                .map(|t| format!(" --target {}", t))
                .unwrap_or_default()
        ));
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
    let (mut m, manifest_path) = snask::sps::load_manifest_from(&cwd)?;
    m.dependencies.insert(name.to_string(), version.unwrap_or_else(|| "*".to_string()));
    snask::sps::write_manifest(&manifest_path, &m)?;

    // instala imediatamente
    let registry = snask::packages::fetch_registry()?;
    let _ = snask::packages::install_package_with_registry(name, &registry)?;
    // lock determinístico
    sps_resolve_deps_and_lock(&cwd, &m)?;

    println!("✅ SPS: dependency '{}' added.", name);
    Ok(())
}

fn sps_remove_dependency(name: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (mut m, manifest_path) = snask::sps::load_manifest_from(&cwd)?;
    m.dependencies.remove(name);
    snask::sps::write_manifest(&manifest_path, &m)?;

    // não desinstala global por padrão (pode ser compartilhado por outros projetos)
    sps_resolve_deps_and_lock(&cwd, &m)?;
    println!("✅ SPS: dependency '{}' removed from snask.snif.", name);
    Ok(())
}

fn sps_resolve_deps_and_lock(dir: &std::path::Path, manifest: &snask::sps::SpsManifest) -> Result<(), String> {
    let registry = snask::packages::fetch_registry()?;
    let mut locked = std::collections::BTreeMap::new();
    for (name, _req) in &manifest.dependencies {
        // constraints v1: "*" ou versão exata
        if let Some(pkg) = registry.packages.get(name) {
            let req = manifest.dependencies.get(name).map(|s| s.as_str()).unwrap_or("*");
            if req != "*" && req != pkg.version() {
                return Err(format!(
                    "SPS: version constraint not satisfied for '{name}': requested '{req}', registry provides '{got}'.\n\nHow to fix:\n- Change the version in `snask.snif` (dependencies.{name})\n- Or run `snask update {name}` and then `snask build`\n",
                    name = name,
                    req = req,
                    got = pkg.version()
                ));
            }
        }

        let url = registry.packages.get(name).map(|p| {
            let u = p.url().trim();
            if u.is_empty() { None } else { Some(u.to_string()) }
        }).flatten();

        if !snask::packages::is_package_installed(name) {
            let (ver, sha, _path) = snask::packages::install_package_with_registry(name, &registry)?;
            locked.insert(name.clone(), snask::sps::LockedDep { version: ver, sha256: sha, url });
        } else {
            let sha = snask::packages::read_installed_package_sha256(name)?;
            let ver = snask::packages::read_installed_package_version_from_registry(name, &registry).unwrap_or_else(|| "unknown".to_string());
            locked.insert(name.clone(), snask::sps::LockedDep { version: ver, sha256: sha, url });
        }
    }
    snask::sps::write_lockfile(dir, manifest, locked)?;
    Ok(())
}

fn dist_entry(
    cli_file: Option<String>,
    targets_csv: Option<String>,
    all: bool,
    deb: bool,
    appimage: bool,
    name: Option<String>,
    linux_user: bool,
    out_dir: String,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // resolve entry file + manifest (optional)
    let (manifest, file_path) = if let Ok((m, _p)) = snask::sps::load_manifest_from(&cwd) {
        // pin + resolve deps para garantir build determinístico (e lock)
        sps_pin_from_lock(&cwd, &m)?;
        sps_resolve_deps_and_lock(&cwd, &m)?;
        (Some(m.clone()), cli_file.unwrap_or_else(|| m.package.entry.clone()))
    } else {
        (None, resolve_entry_file(cli_file)?)
    };

    // resolve base binary name
    let base_name = name.unwrap_or_else(|| {
        if let Ok((m, _p)) = snask::sps::load_manifest_from(&cwd) {
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
        let opt = if let Ok((m, _p)) = snask::sps::load_manifest_from(&cwd) {
            m.opt_level_for(true)
        } else {
            2
        };

        println!("🔧 build: {} -> {}", triple, out_path.display());
        build_file_with_opt(&file_path, Some(out_path.to_string_lossy().to_string()), opt, t.clone())?;
    }

    // Linux user install (best-effort)
    #[cfg(target_os = "linux")]
    {
        if linux_user {
            let bin_path = out_dir.join(&base_name);
            if !bin_path.exists() {
                return Err(format!("For --linux-user, need native Linux binary at '{}'. Run `snask dist` without cross targets first.", bin_path.display()));
            }
            install_linux_user(&cwd, manifest.as_ref(), &base_name, &bin_path)?;
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = linux_user;
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
fn install_linux_user(
    project_dir: &std::path::Path,
    manifest: Option<&snask::sps::SpsManifest>,
    base_name: &str,
    bin_path: &std::path::Path,
) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found.".to_string())?;
    let local_bin = std::path::Path::new(&home).join(".local/bin");
    let apps = std::path::Path::new(&home).join(".local/share/applications");
    let icons = std::path::Path::new(&home).join(".local/share/icons/hicolor/scalable/apps");
    std::fs::create_dir_all(&local_bin).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&apps).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&icons).map_err(|e| e.to_string())?;

    let app = manifest.and_then(|m| m.app.clone());
    let app_id = app.as_ref().map(|a| a.id.as_str()).unwrap_or(base_name);
    let app_name = app.as_ref().map(|a| a.name.as_str()).unwrap_or(base_name);
    let comment = app.as_ref().map(|a| a.comment.as_str()).unwrap_or("");
    let categories = app.as_ref().map(|a| a.categories.as_str()).unwrap_or("Utility;");
    let terminal = app.as_ref().map(|a| a.terminal).unwrap_or(false);
    let icon_field = app.as_ref().map(|a| a.icon.as_str()).unwrap_or("");

    let dest_bin = local_bin.join(base_name);
    std::fs::copy(bin_path, &dest_bin).map_err(|e| e.to_string())?;
    let mut perms = std::fs::metadata(&dest_bin).map_err(|e| e.to_string())?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&dest_bin, perms).map_err(|e| e.to_string())?;

    // icon: if icon is a path inside project, copy it; otherwise treat as icon name.
    let icon_name = if !icon_field.trim().is_empty() {
        let p = project_dir.join(icon_field);
        if p.exists() {
            let dest = icons.join(format!("{}.svg", app_id));
            std::fs::copy(&p, &dest).map_err(|e| e.to_string())?;
            app_id.to_string()
        } else {
            icon_field.to_string()
        }
    } else {
        app_id.to_string()
    };

    let desktop_path = apps.join(format!("{}.desktop", app_id));
    let desktop = format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment={}\nExec={}\nIcon={}\nTerminal={}\nCategories={}\n",
        app_name,
        comment,
        base_name,
        icon_name,
        if terminal { "true" } else { "false" },
        categories
    );
    std::fs::write(&desktop_path, desktop).map_err(|e| e.to_string())?;

    if which("update-desktop-database").is_ok() {
        let _ = Command::new("update-desktop-database").arg(&apps).status();
    }

    println!("✅ linux-user installed:");
    println!("- binary: {}", dest_bin.display());
    println!("- desktop: {}", desktop_path.display());
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

fn sps_pin_from_lock(dir: &std::path::Path, manifest: &snask::sps::SpsManifest) -> Result<(), String> {
    // Se existir snask.lock, garante que os pacotes instalados batem com sha/version do lock.
    // Se divergir: reinstala do registry (MVP).
    let lock_path = snask::sps::lockfile_path(dir);
    if !lock_path.exists() {
        return Ok(());
    }
    let lock = snask::sps::read_lockfile(dir)?;
    let registry = snask::packages::fetch_registry()?;

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
                    "SPS: version constraint not satisfied for '{name}': requested '{req}', registry provides '{got}'.\n\nHow to fix:\n- Change the version in `snask.snif` (dependencies.{name})\n- Or run `snask update {name}` and then `snask build`\n",
                    name = name,
                    req = req,
                    got = pkg.version()
                ));
            }
        }

        let need_install = if !snask::packages::is_package_installed(name) {
            true
        } else {
            let sha = snask::packages::read_installed_package_sha256(name)?;
            sha != dep.sha256
        };
        if need_install {
            let (ver, sha, _path) = snask::packages::install_package_with_registry(name, &registry)?;
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

fn render_parser_diagnostic(filename: &str, source: &str, err: &snask::parser::ParseError) -> String {
    use snask::hds::{Cause, DiagnosticId, FixIt, FixItApply, FixItKind, HyperDiagnostic, MAYBE_THRESHOLD};

    let mut hd = HyperDiagnostic::error(DiagnosticId(err.code), err.message.clone(), err.span);
    for n in &err.notes {
        hd = hd.with_note(n.clone());
    }
    if let Some(h) = &err.help {
        hd = hd.with_help(h.clone());
    }

    // High-confidence fixits (safe-mode only).
    match err.code {
        "SNASK-PARSE-SEMICOLON" => {
            hd = hd
                .with_cause(Cause {
                    title: "Statement is missing a semicolon terminator".to_string(),
                    detail: None,
                    confidence: 95,
                })
                .with_fixit(FixIt {
                    title: "Insert ';' at the end of the statement".to_string(),
                    confidence: 95,
                    kind: FixItKind::QuickFix,
                    apply: Some(FixItApply::WorkspaceEditHint(
                        "Editor quickfix can insert the missing ';'.".to_string(),
                    )),
                });
        }
        "SNASK-PARSE-MISSING-RPAREN" => {
            hd = hd.with_fixit(FixIt {
                title: "Add missing ')'".to_string(),
                confidence: 90,
                kind: FixItKind::QuickFix,
                apply: Some(FixItApply::WorkspaceEditHint(
                    "Editor quickfix can insert a missing ')' near the error.".to_string(),
                )),
            });
        }
        "SNASK-PARSE-MISSING-RBRACKET" => {
            hd = hd.with_fixit(FixIt {
                title: "Add missing ']'".to_string(),
                confidence: 90,
                kind: FixItKind::QuickFix,
                apply: Some(FixItApply::WorkspaceEditHint(
                    "Editor quickfix can insert a missing ']' near the error.".to_string(),
                )),
            });
        }
        "SNASK-PARSE-MISSING-RBRACE" => {
            hd = hd.with_fixit(FixIt {
                title: "Add missing '}'".to_string(),
                confidence: 90,
                kind: FixItKind::QuickFix,
                apply: Some(FixItApply::WorkspaceEditHint(
                    "Editor quickfix can insert a missing '}' near the error.".to_string(),
                )),
            });
        }
        "SNASK-PARSE-INDENT" => {
            hd = hd.with_cause(Cause {
                title: "Indentation does not match the expected block structure".to_string(),
                detail: Some("Snask uses indentation as syntax; mixed tabs/spaces or a wrong indent level will break parsing.".to_string()),
                confidence: MAYBE_THRESHOLD,
            });
        }
        "SNASK-PARSE-EXPR" => {
            let snippet = if err.span.start.offset > 5 {
                &source[err.span.start.offset.saturating_sub(10)..err.span.start.offset]
            } else {
                &source[0..err.span.start.offset]
            };

            if snippet.trim_end().ends_with('=') {
                hd = hd
                    .with_cause(Cause {
                        title: "Variable assignment is missing a value".to_string(),
                        detail: Some("After the '=' operator, Snask expects an expression (a number, string, variable, or function call).".to_string()),
                        confidence: 90,
                    })
                    .with_help("Try adding a value: `let x = 10;` or `let name = \"Davi\";`".to_string());
            } else if snippet.trim_end().ends_with('(') {
                hd = hd
                    .with_cause(Cause {
                        title: "Function call or grouping is missing an expression".to_string(),
                        detail: Some("Inside parentheses, an expression is required.".to_string()),
                        confidence: 85,
                    })
                    .with_help("Provide an argument or value: `print(\"hello\")` or `(1 + 2)`".to_string());
            } else if err.message.contains("end of file") {
                hd = hd
                    .with_note("The compiler reached the end of the file while expecting more code to complete the current statement.".to_string())
                    .with_help("Check if you forgot to finish a declaration or if there's a missing closing brace/parenthesis.".to_string());
            }
        }
        _ => {}
    }

    if let Some(explanation) = snask::explain::get_explanation(err.code) {
        hd = hd.with_note(format!("Why is this an error? \n   {}", explanation.replace('\n', "\n   ")));
    }

    // Local-only trace (opt-in): SNASK_HDS_TRACE=1
    if snask::hds::should_trace() {
        let ext = std::path::Path::new(filename)
            .extension()
            .map(|s| s.to_string_lossy().to_string());
        let ctx = snask::hds::trace_context_hash(err.code, source, err.span);
        let trace = snask::hds::Trace {
            code: err.code.to_string(),
            confidence_max: hd.max_confidence(),
            file_ext: ext,
            context_hash: ctx,
        };
        let _ = snask::hds::write_trace(&trace);
    }

    let d = hd.to_renderable();
    let mut bag = DiagnosticBag::new();
    bag.add(d);
    bag.render_all(filename, source)
}

fn render_semantic_diagnostics(filename: &str, source: &str, errors: &[SemanticError]) -> String {
    use snask::hds::{DiagnosticId, HyperDiagnostic};
    let mut bag = DiagnosticBag::new();
    for e in errors {
        let mut hd = HyperDiagnostic::error(DiagnosticId(e.code()), e.message(), e.span);
        
        for n in &e.notes {
            hd = hd.with_note(n.clone());
        }
        if let Some(h) = &e.help {
            hd = hd.with_help(h.clone());
        }

        if snask::hds::should_trace() {
            let ext = std::path::Path::new(filename)
                .extension()
                .map(|s| s.to_string_lossy().to_string());
            let ctx = snask::hds::trace_context_hash(e.code(), source, e.span);
            let trace = snask::hds::Trace {
                code: e.code().to_string(),
                confidence_max: 0,
                file_ext: ext,
                context_hash: ctx,
            };
            let _ = snask::hds::write_trace(&trace);
        }
        
        bag.add(hd.to_renderable());
    }
    bag.render_all(filename, source)
}

fn self_update() -> Result<(), String> {
    println!("📦 Fetching the latest Snask updates (git pull)...");
    let status = Command::new("git").arg("pull").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Git pull failed.".to_string()); }

    println!("⚙️  Rebuilding the compiler (cargo build --release)...");
    let status = Command::new("cargo").arg("build").arg("--release").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Build failed.".to_string()); }

    println!("✅ Snask updated successfully to v0.3.1!");
    Ok(())
}

fn run_setup(target: Option<String>) -> Result<(), String> {
    println!("🚀 Starting Snask setup v0.3.1...");
    
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    let snask_lib = format!("{}/lib", snask_dir);
    let snask_bin = format!("{}/bin", snask_dir);
    let snask_tmp = format!("{}/tmp", snask_dir);
    let snask_optional_pkgconfig = format!("{}/optional/pkgconfig", snask_dir);

    // Optional components can drop .pc files here (e.g. `snask install-optional skia`).
    // We auto-include it for pkg-config calls during setup.
    let mut optional_pkg_config_path = std::env::var("PKG_CONFIG_PATH").unwrap_or_default();
    if std::path::Path::new(&snask_optional_pkgconfig).exists() {
        if optional_pkg_config_path.is_empty() {
            optional_pkg_config_path = snask_optional_pkgconfig.clone();
        } else {
            optional_pkg_config_path = format!("{}:{}", snask_optional_pkgconfig, optional_pkg_config_path);
        }
    }
    
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

    // Link args required by the runtime (persisted for `snask build`)
    let mut runtime_linkargs: Vec<String> = vec!["-pthread".to_string()];
    let mut want_skia_cpp_bridge = false;

    // Compiler: native uses gcc; cross uses clang-18 --target.
    // Note: when Skia is enabled we also compile a C++ bridge and then "ld -r" to merge objects.
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
        // Optional GUI (GTK3)
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

        // Optional Skia backend (preferred) for snask_skia
        // If available, the runtime will compile with SNASK_SKIA and link Skia libs.
        // This is intentionally opt-in: it requires a local Skia SDK installation.
        let skia_cflags = Command::new("pkg-config")
            .arg("--cflags")
            .arg("skia")
            .env("PKG_CONFIG_PATH", &optional_pkg_config_path)
            .output();
        if let Ok(out) = skia_cflags {
            if out.status.success() {
                let cflags = String::from_utf8_lossy(&out.stdout);
                for f in cflags.split_whitespace() {
                    cc.arg(f);
                }
                cc.arg("-DSNASK_SKIA");
                println!("🖌️  Skia: enabled (runtime).");
                want_skia_cpp_bridge = true;

                if let Ok(libs) = Command::new("pkg-config").arg("--libs").arg("skia").env("PKG_CONFIG_PATH", &optional_pkg_config_path).output() {
                    if libs.status.success() {
                        runtime_linkargs.extend(String::from_utf8_lossy(&libs.stdout).split_whitespace().map(|s| s.to_string()));
                        // Skia is typically built as C++ libs.
                        runtime_linkargs.push("-lstdc++".to_string());
                    }
                }
            } else {
                println!("ℹ️  Skia: not found via pkg-config (snask_skia will use Cairo fallback if GTK is enabled).");
            }
        } else {
            println!("ℹ️  Skia: pkg-config not found (snask_skia will use Cairo fallback if GTK is enabled).");
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

    // Build C runtime object first.
    let status = cc.status().map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(if target.is_some() {
            "Failed to compile runtime.c (cross). Check your target toolchain/headers and whether clang-18 supports this --target.".to_string()
        } else {
            "Failed to compile runtime.c. Make sure gcc is installed.".to_string()
        });
    }

    // If Skia is enabled, compile the C++ bridge and merge into the runtime object (ld -r).
    // This keeps `snask build` simple (still links a single `runtime.o`).
    if target.is_none() && want_skia_cpp_bridge && std::env::var("SNASK_FORCE_NO_SKIA").ok().as_deref() != Some("1") {
            let skia_cpp = tmp_dir.join("runtime").join("skia_bridge.cpp");
            let skia_h = tmp_dir.join("runtime").join("skia_bridge.h");
            fs::write(&skia_cpp, include_str!("runtime/skia_bridge.cpp")).map_err(|e| e.to_string())?;
            fs::write(&skia_h, include_str!("runtime/skia_bridge.h")).map_err(|e| e.to_string())?;

            let skia_obj = tmp_dir.join("skia_bridge.o");
            let mut cxx = Command::new("clang++-18");
            cxx.arg("-c")
                .arg(&skia_cpp)
                .arg("-std=c++17")
                .arg("-fPIC")
                .arg("-o")
                .arg(&skia_obj);

            // Add Skia cflags via pkg-config (same check used earlier).
            if let Ok(out) = Command::new("pkg-config").arg("--cflags").arg("skia").env("PKG_CONFIG_PATH", &optional_pkg_config_path).output() {
                if out.status.success() {
                    let cflags = String::from_utf8_lossy(&out.stdout);
                    for f in cflags.split_whitespace() {
                        cxx.arg(f);
                    }
                }
            }

            let st = cxx.status().map_err(|e| e.to_string())?;
            if !st.success() {
                return Err("Failed to compile skia_bridge.cpp. Make sure clang++-18 and Skia headers are installed (pkg-config skia).".to_string());
            }

            let merged = tmp_dir.join("runtime_merged.o");
            let st = Command::new("ld")
                .arg("-r")
                .arg(&runtime_out)
                .arg(&skia_obj)
                .arg("-o")
                .arg(&merged)
                .status()
                .map_err(|e| e.to_string())?;
            if !st.success() {
                return Err("Failed to merge runtime objects (ld -r).".to_string());
            }
            fs::rename(&merged, &runtime_out).map_err(|e| e.to_string())?;
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

    println!("✅ Snask v0.3.1 setup complete!");
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
    println!("🗑️  Uninstalling Snask v0.3.1...");
    
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not found.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    
    if std::path::Path::new(&snask_dir).exists() {
        fs::remove_dir_all(&snask_dir).map_err(|e| e.to_string())?;
        println!("✅ Removed directory {}.", snask_dir);
    }

    println!("✅ Snask uninstalled successfully!");
    println!("Note: to fully remove PATH changes, edit your .bashrc/.zshrc and remove the Snask lines manually.");
    
    Ok(())
}

fn resolve_imports(
    program: &mut Program,
    current_dir: &std::path::Path,
    resolved_program: &mut Program,
    resolved_modules: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    fn rewrite_expr_native_alias(e: &mut snask::ast::Expr) {
        use snask::ast::ExprKind;
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
                // Skia (experimental)
                "skia_version"|
                "skia_use_real"|
                "skia_surface"|"skia_surface_width"|"skia_surface_height"|
                "skia_surface_clear"|"skia_surface_set_color"|
                "skia_draw_rect"|"skia_draw_circle"|"skia_draw_line"|"skia_draw_text"|
                "skia_save_png"|
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

    fn rewrite_expr_module_calls(e: &mut snask::ast::Expr, module_name: &str, local_fns: &std::collections::HashSet<String>) {
        use snask::ast::ExprKind;
        match &mut e.kind {
            ExprKind::Variable(_) => {}
            ExprKind::Unary { expr, .. } => rewrite_expr_module_calls(expr, module_name, local_fns),
            ExprKind::Binary { left, right, .. } => {
                rewrite_expr_module_calls(left, module_name, local_fns);
                rewrite_expr_module_calls(right, module_name, local_fns);
            }
            ExprKind::FunctionCall { callee, args } => {
                // Only rewrite local *function calls* (callee position).
                // Do NOT rewrite plain variable usage, otherwise parameters/locals can be incorrectly namespaced
                // when they share a name with a local function.
                if let ExprKind::Variable(name) = &mut callee.kind {
                    if !name.contains("::") && local_fns.contains(name.as_str()) {
                        *name = format!("{}::{}", module_name, name);
                    }
                } else {
                    rewrite_expr_module_calls(callee, module_name, local_fns);
                }
                for a in args {
                    rewrite_expr_module_calls(a, module_name, local_fns);
                }
            }
            ExprKind::PropertyAccess { target, .. } => rewrite_expr_module_calls(target, module_name, local_fns),
            ExprKind::IndexAccess { target, index } => {
                rewrite_expr_module_calls(target, module_name, local_fns);
                rewrite_expr_module_calls(index, module_name, local_fns);
            }
            ExprKind::Literal(_) => {}
        }
    }

    fn rewrite_stmt_module_calls(s: &mut snask::ast::Stmt, module_name: &str, local_fns: &std::collections::HashSet<String>) {
        use snask::ast::{StmtKind, LoopStmt};
        match &mut s.kind {
            StmtKind::Expression(e) | StmtKind::FuncCall(e) | StmtKind::Return(e) => rewrite_expr_module_calls(e, module_name, local_fns),
            StmtKind::VarDeclaration(v) => rewrite_expr_module_calls(&mut v.value, module_name, local_fns),
            StmtKind::MutDeclaration(v) => rewrite_expr_module_calls(&mut v.value, module_name, local_fns),
            StmtKind::ConstDeclaration(v) => rewrite_expr_module_calls(&mut v.value, module_name, local_fns),
            StmtKind::VarAssignment(v) => rewrite_expr_module_calls(&mut v.value, module_name, local_fns),
            StmtKind::Print(es) => { for e in es { rewrite_expr_module_calls(e, module_name, local_fns); } }
            StmtKind::Conditional(c) => {
                rewrite_expr_module_calls(&mut c.if_block.condition, module_name, local_fns);
                for st in &mut c.if_block.body { rewrite_stmt_module_calls(st, module_name, local_fns); }
                for b in &mut c.elif_blocks {
                    rewrite_expr_module_calls(&mut b.condition, module_name, local_fns);
                    for st in &mut b.body { rewrite_stmt_module_calls(st, module_name, local_fns); }
                }
                if let Some(else_b) = &mut c.else_block {
                    for st in else_b { rewrite_stmt_module_calls(st, module_name, local_fns); }
                }
            }
            StmtKind::Loop(l) => match l {
                LoopStmt::While { condition, body } => {
                    rewrite_expr_module_calls(condition, module_name, local_fns);
                    for st in body { rewrite_stmt_module_calls(st, module_name, local_fns); }
                }
                LoopStmt::For { iterable, body, .. } => {
                    rewrite_expr_module_calls(iterable, module_name, local_fns);
                    for st in body { rewrite_stmt_module_calls(st, module_name, local_fns); }
                }
            },
            StmtKind::ListDeclaration(d) => rewrite_expr_module_calls(&mut d.value, module_name, local_fns),
            StmtKind::ListPush(p) => rewrite_expr_module_calls(&mut p.value, module_name, local_fns),
            StmtKind::DictDeclaration(d) => rewrite_expr_module_calls(&mut d.value, module_name, local_fns),
            StmtKind::DictSet(d) => { rewrite_expr_module_calls(&mut d.key, module_name, local_fns); rewrite_expr_module_calls(&mut d.value, module_name, local_fns); }
            StmtKind::FuncDeclaration(f) => { for st in &mut f.body { rewrite_stmt_module_calls(st, module_name, local_fns); } }
            StmtKind::ClassDeclaration(c) => {
                for p in &mut c.properties { rewrite_expr_module_calls(&mut p.value, module_name, local_fns); }
                for m in &mut c.methods { for st in &mut m.body { rewrite_stmt_module_calls(st, module_name, local_fns); } }
            }
            StmtKind::Input { .. } => {}
            StmtKind::Import(_) => {}
            StmtKind::FromImport { .. } => {}
        }
    }

    fn namespace_module_functions(module_ast: &mut Program, module_name: &str) {
        use snask::ast::StmtKind;
        let mut local_fns: std::collections::HashSet<String> = std::collections::HashSet::new();
        for st in module_ast.iter() {
            if let StmtKind::FuncDeclaration(f) = &st.kind {
                local_fns.insert(f.name.clone());
            }
        }

        for st in module_ast.iter_mut() {
            if let StmtKind::FuncDeclaration(f) = &mut st.kind {
                f.name = format!("{}::{}", module_name, f.name);
            }
        }

        for st in module_ast.iter_mut() {
            rewrite_stmt_module_calls(st, module_name, &local_fns);
        }
    }

    fn rewrite_stmt_native_alias(s: &mut snask::ast::Stmt) {
        use snask::ast::{StmtKind, LoopStmt};
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
            StmtKind::FromImport { .. } => {}
        }
    }

    for stmt in program.drain(..) {
        match &stmt.kind {
            StmtKind::Import(path) => {
                if !resolved_modules.contains(path) {
                    resolved_modules.insert(path.clone());

                    let mut module_ast = snask::modules::load_module_from(current_dir, path)?;
                    for st in &mut module_ast {
                        rewrite_stmt_native_alias(st);
                    }

                    let module_name = std::path::Path::new(path)
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default();

                    if module_name != "prelude" {
                        for m_stmt in &mut module_ast {
                            if let StmtKind::FuncDeclaration(f) = &mut m_stmt.kind {
                                f.name = format!("{}::{}", module_name, f.name);
                            }
                        }
                    }

                    resolve_imports(&mut module_ast, current_dir, resolved_program, resolved_modules)?;
                }
            }
            StmtKind::FromImport { from, is_current_dir, module } => {
                let (mut module_ast, module_path) =
                    snask::modules::load_from_import(current_dir, from, *is_current_dir, module)?;
                let module_key = format!(
                    "from:{}",
                    module_path.canonicalize().unwrap_or(module_path.clone()).display()
                );

                if !resolved_modules.contains(&module_key) {
                    resolved_modules.insert(module_key);

                    for st in &mut module_ast {
                        rewrite_stmt_native_alias(st);
                    }

                    let module_name = std::path::Path::new(module)
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or(module);

                    if module_name != "prelude" { namespace_module_functions(&mut module_ast, module_name); }

                    let next_dir = module_path.parent().unwrap_or(current_dir);
                    resolve_imports(&mut module_ast, next_dir, resolved_program, resolved_modules)?;
                }
            }
            _ => resolved_program.push(stmt),
        }
    }
    Ok(())
}
