use clap::{Parser as ClapParser, Subcommand};
use std::process::Command;

use snask::compiler::{build_file, resolve_entry_file, BuildOptions};
use snask::dist;
use snask::om_scan::{run_scan, ScanOptions};
use snask::packages;
use snask::sps;
use snask::tools;

#[derive(ClapParser)]
#[command(name = "snask")]
#[command(about = "The Snask Programming Language Compiler & Toolchain", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Snask project (SPS)
    Init {
        name: Option<String>,
        #[arg(long)]
        zenith: bool,
    },
    /// Build a Snask program or project
    Build {
        file: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        lto: bool,
        #[arg(long)]
        release_size: bool,
        #[arg(long)]
        min_runtime: bool,
        #[arg(long)]
        tiny: bool,
        #[arg(long)]
        extreme: bool,
    },
    /// Distribute/package the application
    Dist {
        file: Option<String>,
        #[arg(long)]
        targets: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        deb: bool,
        #[arg(long)]
        appimage: bool,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(long)]
        linux_user: bool,
        #[arg(long, default_value = "dist")]
        out_dir: String,
    },
    /// Run a Snask program or project script
    Run { file: Option<String> },
    /// Add a dependency to the project
    Add {
        name: String,
        version: Option<String>,
    },
    /// Remove a dependency from the project
    Remove { name: String },
    /// Setup the Snask toolchain for a target
    Setup {
        #[arg(long)]
        target: Option<String>,
    },
    /// Install a package globally
    Install { name: String },
    /// Uninstall a package or Snask itself
    Uninstall { name: Option<String> },
    /// Update packages or Snask itself
    Update { name: Option<String> },
    /// Manage Snask libraries
    Lib {
        #[command(subcommand)]
        cmd: LibCommands,
    },
    /// Manage SNIF files
    Snif {
        #[command(subcommand)]
        cmd: SnifCommands,
        file: Option<String>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        stdout: bool,
        #[arg(long)]
        check: bool,
        #[arg(long)]
        strict: bool,
    },
    /// OM-Snask-System tooling
    Om {
        #[command(subcommand)]
        cmd: OmCommands,
    },
    /// Explain a Snask diagnostic code
    Explain { code: String },
    /// System health check
    Doctor,
    /// Measure binary size
    Size { path: String },
    /// Show Snask system information
    Fetch,
}

#[derive(Subcommand)]
enum LibCommands {
    Init {
        name: String,
        #[arg(long)]
        version: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    Publish {
        name: String,
        version: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(short, long)]
        message: Option<String>,
        #[arg(long)]
        push: bool,
        #[arg(long)]
        pr: bool,
        #[arg(long)]
        fork: bool,
        #[arg(long)]
        branch: Option<String>,
    },
}

#[derive(Subcommand)]
enum SnifCommands {
    Schema {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        md: bool,
    },
    Canon,
    Hash,
    Check,
    Fmt,
}

#[derive(Subcommand)]
enum OmCommands {
    /// Scan a C header and generate a temporary .om.snif contract
    Scan {
        header: String,
        #[arg(long)]
        lib: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short, long)]
        cflags: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Init { name, zenith } => {
            if *zenith {
                tools::init_zenith_project(name.clone())
            } else {
                sps::init_project(name.clone())
            }
        }
        Commands::Build {
            file,
            output,
            target,
            lto,
            release_size,
            min_runtime,
            tiny,
            extreme,
        } => run_build(
            file,
            output,
            target,
            *lto,
            *release_size,
            *min_runtime,
            *tiny,
            *extreme,
        ),
        Commands::Run { file } => run_program(file),
        Commands::Add { name, version } => sps::add_dependency(name, version.clone()),
        Commands::Remove { name } => sps::remove_dependency(name),
        Commands::Setup { target } => tools::run_setup(target.clone()),
        Commands::Install { name } => packages::install_package(name),
        Commands::Uninstall { name } => {
            if let Some(pkg) = name {
                packages::uninstall_package(pkg)
            } else {
                tools::run_uninstall()
            }
        }
        Commands::Update { name } => {
            if let Some(pkg) = name {
                packages::install_package(pkg)
            } else {
                tools::self_update()
            }
        }
        Commands::Dist {
            file,
            targets,
            all,
            deb,
            appimage,
            name,
            linux_user,
            out_dir,
        } => dist::run_dist(
            file.clone(),
            targets.clone(),
            *all,
            *deb,
            *appimage,
            name.clone(),
            *linux_user,
            out_dir.clone(),
        ),
        Commands::Doctor => tools::doctor(),
        Commands::Size { path } => tools::cmd_size(path),
        Commands::Fetch => {
            snask::fetch::run_fetch();
            Ok(())
        }
        Commands::Om { cmd } => match cmd {
            OmCommands::Scan {
                header,
                lib,
                output,
                cflags,
            } => run_scan(ScanOptions {
                header: header.clone(),
                lib: lib.clone(),
                output: output.clone(),
                extra_cflags: cflags.clone(),
            }),
        },
        Commands::Explain { code } => snask::explain::run_explain(code),
        _ => Err("Command not implemented yet in this refactor.".to_string()),
    };

    if let Err(e) = result {
        if e.starts_with("error[") || e.starts_with("warning[") {
            eprintln!("{}", e);
        } else {
            eprintln!("error: {}", e);
        }
        std::process::exit(1);
    }
}

fn run_build(
    file: &Option<String>,
    output: &Option<String>,
    target: &Option<String>,
    lto: bool,
    release_size: bool,
    min_runtime: bool,
    tiny: bool,
    extreme: bool,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (file_path, options) = if let Ok((m, _)) = sps::load_manifest_from(&cwd) {
        sps::pin_from_lock(&cwd, &m)?;
        sps::resolve_deps_and_lock(&cwd, &m)?;
        let entry = file.clone().unwrap_or_else(|| m.package.entry.clone());
        let profile_name = m.build.profile.as_deref().unwrap_or("default");

        let is_extreme = extreme || profile_name == "extreme";
        let is_tiny = tiny || profile_name == "tiny" || is_extreme;
        let is_release_size = release_size || profile_name == "release-size";

        let opt = BuildOptions {
            output_name: output.clone().or_else(|| Some(m.package.name.clone())),
            target: target.clone(),
            opt_level: m.build.opt_level,
            lto: lto || m.build.lto.as_deref() == Some("thin") || is_release_size || is_tiny,
            release_size: is_release_size,
            min_runtime,
            tiny: is_tiny,
            extreme: is_extreme,
            strip: m.build.strip.unwrap_or(is_release_size || is_tiny),
            opt_override: m.build.opt.clone(),
            features: m.build.features.clone(),
        };
        (entry, opt)
    } else {
        let entry = resolve_entry_file(file.clone())?;
        let opt = BuildOptions {
            output_name: output.clone(),
            target: target.clone(),
            lto,
            release_size,
            min_runtime,
            tiny,
            extreme,
            ..Default::default()
        };
        (entry, opt)
    };

    build_file(&file_path, options)
}

fn run_program(file: &Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // Handle scripts
    if let Some(arg) = file {
        if !arg.ends_with(".snask") {
            if let Ok((m, _)) = sps::load_manifest_from(&cwd) {
                if let Some(cmdline) = m.scripts.get(arg) {
                    let status = Command::new("sh")
                        .arg("-lc")
                        .arg(cmdline)
                        .status()
                        .map_err(|e| e.to_string())?;
                    if !status.success() {
                        return Err(format!("Script '{}' failed.", arg));
                    }
                    return Ok(());
                }
            }
        }
    }

    // Resolve entry file
    let (file_path, options) = if let Ok((m, _)) = sps::load_manifest_from(&cwd) {
        sps::pin_from_lock(&cwd, &m)?;
        sps::resolve_deps_and_lock(&cwd, &m)?;
        let entry = file.clone().unwrap_or_else(|| m.package.entry.clone());
        let opt = BuildOptions {
            opt_level: m.opt_level_for(true),
            features: m.build.features.clone(),
            ..Default::default()
        };
        (entry, opt)
    } else {
        (resolve_entry_file(file.clone())?, BuildOptions::default())
    };

    // Build
    build_file(&file_path, options)?;

    // Run
    let binary = file_path.replace(".snask", "");
    let binary_path = if binary.starts_with('/') || binary.starts_with("./") {
        binary
    } else {
        format!("./{}", binary)
    };
    let status = Command::new(&binary_path)
        .status()
        .map_err(|e| format!("Failed to run binary: {}", e))?;
    if !status.success() {
        return Err("Binary execution failed.".to_string());
    }
    Ok(())
}
