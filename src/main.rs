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
        /// URL do seu fork (ex: https://github.com/SEUUSER/SnaskPackages)
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
                eprintln!("Erro ao iniciar projeto: {}", e);
            }
        }
        Commands::Build { file, output, target } => {
            match build_entry(file.clone(), output.clone(), target.clone()) {
                Ok(_) => println!("Compilação LLVM concluída com sucesso."),
                Err(e) => eprintln!("Erro durante a compilação: {}", e),
            }
        },
        Commands::Run { file } => {
            // scripts: `snask run dev`
            if let Some(arg) = file.clone() {
                if !arg.ends_with(".snask") {
                    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                    if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
                        if let Some(cmdline) = m.scripts.get(&arg) {
                            let status = Command::new("sh").arg("-lc").arg(cmdline).status();
                            if let Err(e) = status {
                                eprintln!("Erro ao executar script '{}': {}", arg, e);
                            }
                            return;
                        }
                    }
                }
            }

            let cwd = match std::env::current_dir() {
                Ok(d) => d,
                Err(e) => { eprintln!("Erro: {}", e); return; }
            };
            let file_path = match crate::sps::load_manifest_from(&cwd) {
                Ok((m, _p)) => {
                    if let Err(e) = sps_pin_from_lock(&cwd, &m) {
                        eprintln!("Erro durante a compilação: {}", e);
                        return;
                    }
                    if let Err(e) = sps_resolve_deps_and_lock(&cwd, &m) {
                        eprintln!("Erro durante a compilação: {}", e);
                        return;
                    }
                    file.clone().unwrap_or_else(|| m.package.entry.clone())
                }
                Err(_) => match resolve_entry_file(file.clone()) {
                    Ok(p) => p,
                    Err(e) => { eprintln!("Erro durante a compilação: {}", e); return; }
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
                        eprintln!("Erro ao executar o binário: {}", e);
                    }
                },
                Err(e) => eprintln!("Erro durante a compilação: {}", e),
            }
        },
        Commands::Add { name, version } => {
            if let Err(e) = sps_add_dependency(name, version.clone()) {
                eprintln!("Erro ao adicionar dependência: {}", e);
            }
        }
        Commands::Remove { name } => {
            if let Err(e) = sps_remove_dependency(name) {
                eprintln!("Erro ao remover dependência: {}", e);
            }
        }
        Commands::Setup { target } => {
            if let Err(e) = run_setup(target.clone()) {
                eprintln!("Erro durante o setup: {}", e);
            }
        },
        Commands::Install { name } => {
            if let Err(e) = packages::install_package(name) {
                eprintln!("Erro ao instalar pacote: {}", e);
            }
        },
        Commands::Uninstall { name } => {
            if let Some(pkg_name) = name {
                if let Err(e) = packages::uninstall_package(&pkg_name) {
                    eprintln!("Erro ao desinstalar pacote: {}", e);
                }
            } else {
                println!("⚠️  Atenção: Você está prestes a desinstalar o SNask globalmente.");
                if let Err(e) = run_uninstall() {
                    eprintln!("Erro durante a desinstalação: {}", e);
                }
            }
        },
        Commands::Update { name } => {
            if let Some(pkg_name) = name {
                println!("🔄 Atualizando pacote '{}'...", pkg_name);
                if let Err(e) = packages::install_package(pkg_name) {
                    eprintln!("Erro ao atualizar pacote: {}", e);
                }
            } else {
                println!("🚀 Iniciando auto-update do SNask...");
                if let Err(e) = self_update() {
                    eprintln!("Erro ao atualizar o SNask: {}", e);
                }
            }
        },
        Commands::List => {
            if let Err(e) = packages::list_packages() {
                eprintln!("Erro ao listar pacotes: {}", e);
            }
        },
        Commands::Search { query } => {
            if let Err(e) = packages::search_packages(query) {
                eprintln!("Erro ao pesquisar pacotes: {}", e);
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
                        eprintln!("Erro: {}", e);
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
                        eprintln!("Erro: {}", e);
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
    Err("SPS: nenhum arquivo informado e não encontrei `snask.toml` no diretório atual.\n\nComo resolver:\n- Compile um arquivo direto: `snask build main.snask`\n- Ou crie um projeto SPS: `snask init` e depois `snask build`\n".to_string())
}

fn build_entry(cli_file: Option<String>, output_name: Option<String>, target: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    spinner.enable_steady_tick(std::time::Duration::from_millis(90));

    // Se estiver em um projeto SPS, resolve deps e gera lock antes de build.
    if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
        spinner.set_message(format!("SPS: snask.toml (entry: {})", m.package.entry));
        // pin pelo lock (se existir) antes de resolver
        spinner.set_message("SPS: verificando snask.lock".to_string());
        sps_pin_from_lock(&cwd, &m)?;
        spinner.set_message("SPS: resolvendo dependências".to_string());
        sps_resolve_deps_and_lock(&cwd, &m)?;
        let opt = m.opt_level_for(true);
        let file_path = cli_file.unwrap_or_else(|| m.package.entry.clone());
        spinner.set_message(format!("Compilando {} (O{})", file_path, opt));
        let res = build_file_with_opt(&file_path, output_name, opt, target.clone());
        match &res {
            Ok(_) => spinner.finish_with_message("Build finalizado"),
            Err(_) => spinner.finish_and_clear(),
        }
        return res;
    }

    let file_path = resolve_entry_file(cli_file)?;
    spinner.set_message(format!("Compilando {}", file_path));
    let res = build_file_with_opt(&file_path, output_name, 2, target);
    match &res {
        Ok(_) => spinner.finish_with_message("Build finalizado"),
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
    let mut program = Parser::new(&source)?.parse_program().map_err(|e| format!("Erro no parser: {}", e))?;
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
        return Err("Erro: Todo programa SNask deve conter uma 'class main'.".to_string());
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

    if !status.success() { return Err("Falha na linkagem final.".to_string()); }
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
    let manifest_path = cwd.join("snask.toml");
    if manifest_path.exists() {
        return Err("snask.toml já existe neste diretório.".to_string());
    }

    let entry = "main.snask";
    let main_path = cwd.join(entry);
    if main_path.exists() {
        return Err(format!("Arquivo '{}' já existe neste diretório.", entry));
    }

    let manifest = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nentry = \"{}\"\n\n[dependencies]\n\n[build]\nopt_level = 2\n",
        project_name.replace('"', ""),
        entry
    );
    fs::write(&manifest_path, manifest).map_err(|e| e.to_string())?;

    let main_src = "class main\n    fun start()\n        print(\"hello snask\");\n";
    fs::write(&main_path, main_src).map_err(|e| e.to_string())?;

    println!("✅ SPS: criado snask.toml e {}.", entry);
    Ok(())
}

fn sps_add_dependency(name: &str, version: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (mut m, manifest_path) = crate::sps::load_manifest_from(&cwd)?;
    m.dependencies.insert(name.to_string(), version.unwrap_or_else(|| "*".to_string()));
    let s = toml::to_string_pretty(&m).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, s).map_err(|e| e.to_string())?;

    // instala imediatamente
    let registry = crate::packages::fetch_registry()?;
    let _ = crate::packages::install_package_with_registry(name, &registry)?;
    // lock determinístico
    sps_resolve_deps_and_lock(&cwd, &m)?;

    println!("✅ SPS: dependência '{}' adicionada.", name);
    Ok(())
}

fn sps_remove_dependency(name: &str) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let (mut m, manifest_path) = crate::sps::load_manifest_from(&cwd)?;
    m.dependencies.remove(name);
    let s = toml::to_string_pretty(&m).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, s).map_err(|e| e.to_string())?;

    // não desinstala global por padrão (pode ser compartilhado por outros projetos)
    sps_resolve_deps_and_lock(&cwd, &m)?;
    println!("✅ SPS: dependência '{}' removida do snask.toml.", name);
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
                    "SPS: lockfile pede {name}@{want_ver} sha256={want_sha}, mas o download resultou em {got_ver} sha256={got_sha}.\n\nCausas comuns:\n- O pacote mudou no registry (novo release/arquivo alterado)\n- Seu lock está desatualizado\n\nComo resolver:\n- Se você quer pegar o novo pacote: `snask update {name}` e depois `snask build` (regenera snask.lock)\n- Se você quer manter o lock atual: verifique se o registry voltou a ter o mesmo sha256.\n",
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
        VariableAlreadyDeclared(name) => format!("Variável '{}' já foi declarada.", name),
        VariableNotFound(name) => format!("Variável '{}' não encontrada.", name),
        FunctionAlreadyDeclared(name) => format!("Função '{}' já foi declarada.", name),
        FunctionNotFound(name) => format!("Função '{}' não encontrada.", name),
        TypeMismatch { expected, found } => format!("Tipo incompatível: esperado {:?}, encontrado {:?}.", expected, found),
        InvalidOperation { op, type1, type2 } => {
            if let Some(t2) = type2 {
                format!("Operação inválida: '{}' entre {:?} e {:?}.", op, type1, t2)
            } else {
                format!("Operação inválida: '{}' em {:?}.", op, type1)
            }
        }
        ImmutableAssignment(name) => format!("'{}' é imutável. Dica: declare com 'mut {} = ...;'.", name, name),
        ReturnOutsideFunction => "Uso de 'return' fora de uma função.".to_string(),
        WrongNumberOfArguments { expected, found } => format!("Número errado de argumentos: esperado {}, encontrado {}.", expected, found),
        IndexAccessOnNonIndexable(t) => format!("Acesso por índice em tipo não indexável: {:?}.", t),
        InvalidIndexType(t) => format!("Tipo de índice inválido: {:?} (esperado num).", t),
        PropertyNotFound(p) => format!("Propriedade '{}' não encontrada.", p),
        NotCallable(t) => format!("Tentativa de chamada em valor não chamável: {:?}.", t),
        RestrictedNativeFunction { name, help } => format!("Uso direto de nativa '{}' não é permitido.\n{}", name, help),
    }
}

fn self_update() -> Result<(), String> {
    println!("📦 Baixando as últimas novidades do SNask (git pull)...");
    let status = Command::new("git").arg("pull").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Falha ao puxar do Git.".to_string()); }

    println!("⚙️  Recompilando o compilador (cargo build --release)...");
    let status = Command::new("cargo").arg("build").arg("--release").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Falha ao compilar.".to_string()); }

    println!("✅ SNask atualizado com sucesso para a versão 0.3.0!");
    Ok(())
}

fn run_setup(target: Option<String>) -> Result<(), String> {
    println!("🚀 Iniciando configuração do SNask v0.3.0...");
    
    let home = std::env::var("HOME").map_err(|_| "Variável HOME não encontrada.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    let snask_lib = format!("{}/lib", snask_dir);
    let snask_bin = format!("{}/bin", snask_dir);
    
    println!("📁 Criando diretórios em {}...", snask_dir);
    fs::create_dir_all(&snask_lib).map_err(|e| e.to_string())?;
    fs::create_dir_all(&snask_bin).map_err(|e| e.to_string())?;

    println!("⚙️  Compilando o Runtime Nativo (C)...");
    let (_runtime_dir, runtime_out, linkargs_path) = if let Some(t) = &target {
        let dir = format!("{}/{}", snask_lib, t);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        (dir.clone(), format!("{}/runtime.o", dir), format!("{}/runtime.linkargs", dir))
    } else {
        (snask_lib.clone(), format!("{}/runtime.o", snask_lib), format!("{}/runtime.linkargs", snask_lib))
    };

    // Link args requeridos pelo runtime (persistidos para `snask build`)
    let mut runtime_linkargs: Vec<String> = vec!["-pthread".to_string()];

    // Compilador: nativo usa gcc; cross usa clang-18 --target.
    let mut cc = if target.is_some() { Command::new("clang-18") } else { Command::new("gcc") };
    cc.arg("-c")
        .arg("src/runtime.c")
        .arg("-ffunction-sections")
        .arg("-fdata-sections")
        .arg("-pthread")
        .arg("-o")
        .arg(&runtime_out);

    if let Some(t) = &target {
        // Para cross-compilation, não tentamos ativar GTK/SQLite via pkg-config (seria do host).
        cc.arg(format!("--target={}", t));
        println!("🎯 Cross: runtime alvo = {}", t);
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
                println!("🖼️  GUI: GTK3 habilitado (runtime).");

                if let Ok(libs) = Command::new("pkg-config").arg("--libs").arg("gtk+-3.0").output() {
                    if libs.status.success() {
                        runtime_linkargs.extend(String::from_utf8_lossy(&libs.stdout).split_whitespace().map(|s| s.to_string()));
                    }
                }
            } else {
                println!("ℹ️  GUI: GTK3 não encontrado via pkg-config (runtime sem GUI).");
            }
        } else {
            println!("ℹ️  GUI: pkg-config não encontrado (runtime sem GUI).");
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
                println!("🗄️  SQLite: sqlite3 habilitado (runtime).");

                if let Ok(libs) = Command::new("pkg-config").arg("--libs").arg("sqlite3").output() {
                    if libs.status.success() {
                        runtime_linkargs.extend(String::from_utf8_lossy(&libs.stdout).split_whitespace().map(|s| s.to_string()));
                    }
                }
            } else {
                println!("ℹ️  SQLite: sqlite3 não encontrado via pkg-config (runtime sem SQLite).");
            }
        } else {
            println!("ℹ️  SQLite: pkg-config não encontrado (runtime sem SQLite).");
        }
    }

    let status = cc.status().map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(if target.is_some() {
            "Falha ao compilar o runtime.c (cross). Verifique seu toolchain/headers do alvo e se o clang-18 suporta este --target.".to_string()
        } else {
            "Falha ao compilar o runtime.c. Verifique se o gcc está instalado.".to_string()
        });
    }

    let linkargs = runtime_linkargs.join(" ");
    fs::write(&linkargs_path, linkargs).map_err(|e| e.to_string())?;

    println!("🚚 Instalando binário em {}...", snask_bin);
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dest_path = std::path::PathBuf::from(&snask_bin).join("snask");
    install_self_binary(&current_exe, &dest_path)?;

    println!("🌐 Configurando o PATH...");
    let shell_configs = vec![format!("{}/.bashrc", home), format!("{}/.zshrc", home)];
    let path_line = format!("\n# SNask Language\nexport PATH=\"{}:$PATH\"\n", snask_bin);
    
    for config_path in shell_configs {
        if std::path::Path::new(&config_path).exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            if !content.contains("SNask Language") {
                use std::io::Write;
                let mut file = fs::OpenOptions::new().append(true).open(&config_path).map_err(|e| e.to_string())?;
                file.write_all(path_line.as_bytes()).map_err(|e| e.to_string())?;
                println!("✅ PATH adicionado ao arquivo {}", config_path);
            }
        }
    }

    println!("✅ SNask v0.3.0 configurado com sucesso!");
    println!("Dica: Reinicie seu terminal ou rode 'source ~/.bashrc' para começar a usar o comando 'snask' de qualquer lugar.");
    
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

    std::fs::copy(current_exe, &tmp_path).map_err(|e| format!("Falha ao copiar binário: {}", e))?;

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
            std::fs::rename(&tmp_path, dest_path).map_err(|e2| format!("Erro durante o setup: {} (orig: {})", e2, e))?;
            Ok(())
        }
    }
}

fn run_uninstall() -> Result<(), String> {
    println!("🗑️  Desinstalando SNask v0.3.0...");
    
    let home = std::env::var("HOME").map_err(|_| "Variável HOME não encontrada.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    
    if std::path::Path::new(&snask_dir).exists() {
        fs::remove_dir_all(&snask_dir).map_err(|e| e.to_string())?;
        println!("✅ Diretório {} removido.", snask_dir);
    }

    println!("✅ SNask desinstalado com sucesso!");
    println!("Nota: Para remover completamente o PATH, verifique seu .bashrc ou .zshrc e remova as linhas do SNask manualmente.");
    
    Ok(())
}

fn resolve_imports(program: &mut Program, resolved_program: &mut Program, resolved_modules: &mut std::collections::HashSet<String>) -> Result<(), String> {
    fn rewrite_expr_native_alias(e: &mut crate::ast::Expr) {
        use crate::ast::ExprKind;
        const NAMES: &[&str] = &[
            "sfs_read","sfs_write","sfs_append","sfs_delete","sfs_exists","sfs_copy","sfs_move","sfs_mkdir","sfs_is_file","sfs_is_dir","sfs_listdir","sfs_size","sfs_mtime","sfs_rmdir",
            "path_basename","path_dirname","path_extname","path_join",
            "os_cwd","os_platform","os_arch","os_getenv","os_setenv","os_random_hex",
            "s_http_get","s_http_post","s_http_put","s_http_delete","s_http_patch",
            "blaze_run","blaze_qs_get","blaze_cookie_get",
            "auth_random_hex","auth_now","auth_const_time_eq","auth_hash_password","auth_verify_password","auth_session_id","auth_csrf_token","auth_cookie_kv","auth_cookie_session","auth_cookie_delete","auth_bearer_header","auth_ok","auth_fail","auth_version",
            "gui_init","gui_run","gui_quit","gui_window","gui_set_title","gui_set_resizable","gui_autosize","gui_vbox","gui_hbox","gui_scrolled","gui_listbox","gui_list_add_text","gui_on_select_ctx","gui_set_child","gui_add","gui_add_expand","gui_label","gui_entry","gui_set_placeholder","gui_set_editable","gui_button","gui_set_enabled","gui_set_visible","gui_show_all","gui_set_text","gui_get_text","gui_on_click","gui_on_click_ctx","gui_separator_h","gui_separator_v","gui_msg_info","gui_msg_error",
            "sqlite_open","sqlite_close","sqlite_exec","sqlite_query","sqlite_prepare","sqlite_finalize","sqlite_reset","sqlite_bind_text","sqlite_bind_num","sqlite_bind_null","sqlite_step","sqlite_column","sqlite_column_count","sqlite_column_name",
            "thread_spawn","thread_join","thread_detach",
            "json_stringify","json_stringify_pretty","json_parse","json_get","json_has","json_len","json_index","json_set","json_keys","json_parse_ex",
            "sjson_new_object","sjson_new_array","sjson_type","sjson_arr_len","sjson_arr_get","sjson_arr_set","sjson_arr_push","sjson_path_get",
        ];

        match &mut e.kind {
            ExprKind::Variable(name) => {
                if NAMES.contains(&name.as_str()) {
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
