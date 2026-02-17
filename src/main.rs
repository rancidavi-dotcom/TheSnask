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

#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { #[arg(short, long)] name: Option<String> },
    Build { file: Option<String>, #[arg(short, long)] output: Option<String> },
    Run { file: Option<String> },
    Add { name: String, version: Option<String> },
    Remove { name: String },
    Setup,
    Install { name: String },
    Uninstall { name: Option<String> },
    Update { name: Option<String> },
    List,
    Search { query: String },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init { name } => {
            if let Err(e) = sps_init(name.clone()) {
                eprintln!("Erro ao iniciar projeto: {}", e);
            }
        }
        Commands::Build { file, output } => {
            match build_entry(file.clone(), output.clone()) {
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
                build_file_with_opt(&file_path, None, opt)
            } else {
                build_file(&file_path, None)
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
        Commands::Setup => {
            if let Err(e) = run_setup() {
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
    Err("SPS: nenhum arquivo informado. Use `snask build arquivo.snask` ou crie um projeto com `snask init` (snask.toml) e rode `snask build`.".to_string())
}

fn build_entry(cli_file: Option<String>, output_name: Option<String>) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // Se estiver em um projeto SPS, resolve deps e gera lock antes de build.
    if let Ok((m, _p)) = crate::sps::load_manifest_from(&cwd) {
        sps_resolve_deps_and_lock(&cwd, &m)?;
        let opt = m.opt_level_for(true);
        let file_path = cli_file.unwrap_or_else(|| m.package.entry.clone());
        return build_file_with_opt(&file_path, output_name, opt);
    }

    let file_path = resolve_entry_file(cli_file)?;
    build_file(&file_path, output_name)
}

fn build_file(file_path: &str, output_name: Option<String>) -> Result<(), String> {
    build_file_with_opt(file_path, output_name, 2)
}

fn build_file_with_opt(file_path: &str, output_name: Option<String>, opt_level: u8) -> Result<(), String> {
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut program = Parser::new(&source)?.parse_program().map_err(|e| format!("Erro no parser: {}", e))?;

    // Validação de Class Main Obrigatória apenas no programa principal
    let has_main = program.iter().any(|stmt| {
        if let StmtKind::ClassDeclaration(class) = &stmt.kind {
            class.name == "main"
        } else {
            false
        }
    });

    if !has_main {
        return Err("Erro: Todo programa SNask deve conter uma 'class main'.".to_string());
    }

    let mut resolved_program = Vec::new();
    let mut resolved_modules = std::collections::HashSet::new();
    resolved_modules.insert(file_path.to_string());
    resolve_imports(&mut program, &mut resolved_program, &mut resolved_modules)?;

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&resolved_program);
    if !analyzer.errors.is_empty() {
        let mut out = String::from("Análise semântica encontrou erros:\n");
        for e in &analyzer.errors {
            out.push_str(&format!("- {}\n", format_semantic_error(e)));
        }
        return Err(out);
    }

    let context = Context::create();
    let mut generator = LLVMGenerator::new(&context, file_path);
    let ir = generator.generate(resolved_program)?;

    let ir_file = "temp_snask.ll";
    let obj_file = "temp_snask.o";
    fs::write(ir_file, ir).map_err(|e| e.to_string())?;

    let opt_flag = format!("-O{}", opt_level);
    Command::new("llc-18")
        .arg(opt_flag)
        .arg("-relocation-model=pic")
        .arg("-filetype=obj")
        .arg(ir_file)
        .arg("-o")
        .arg(obj_file)
        .status()
        .map_err(|e| e.to_string())?;

    let runtime_path = format!("{}/.snask/lib/runtime.o", std::env::var("HOME").unwrap());
    let final_output = output_name.unwrap_or_else(|| file_path.replace(".snask", ""));

    let status = Command::new("clang-18")
        .arg(obj_file)
        .arg(runtime_path)
        .arg("-o")
        .arg(&final_output)
        .arg("-lm")
        // Necessário para blaze (dlsym handlers) e para expor símbolos do binário
        .arg("-ldl")
        .arg("-Wl,--export-dynamic")
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() { return Err("Falha na linkagem final.".to_string()); }
    fs::remove_file(ir_file).ok(); fs::remove_file(obj_file).ok();
    Ok(())
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
        if !crate::packages::is_package_installed(name) {
            let (ver, sha, _path) = crate::packages::install_package_with_registry(name, &registry)?;
            locked.insert(name.clone(), crate::sps::LockedDep { version: ver, sha256: sha });
        } else {
            let sha = crate::packages::read_installed_package_sha256(name)?;
            let ver = crate::packages::read_installed_package_version_from_registry(name, &registry).unwrap_or_else(|| "unknown".to_string());
            locked.insert(name.clone(), crate::sps::LockedDep { version: ver, sha256: sha });
        }
    }
    crate::sps::write_lockfile(dir, manifest, locked)?;
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
    }
}

fn self_update() -> Result<(), String> {
    println!("📦 Baixando as últimas novidades do SNask (git pull)...");
    let status = Command::new("git").arg("pull").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Falha ao puxar do Git.".to_string()); }

    println!("⚙️  Recompilando o compilador (cargo build --release)...");
    let status = Command::new("cargo").arg("build").arg("--release").status().map_err(|e| e.to_string())?;
    if !status.success() { return Err("Falha ao compilar.".to_string()); }

    println!("✅ SNask atualizado com sucesso para a versão 0.2.2!");
    Ok(())
}

fn run_setup() -> Result<(), String> {
    println!("🚀 Iniciando configuração do SNask v0.2.2...");
    
    let home = std::env::var("HOME").map_err(|_| "Variável HOME não encontrada.".to_string())?;
    let snask_dir = format!("{}/.snask", home);
    let snask_lib = format!("{}/lib", snask_dir);
    let snask_bin = format!("{}/bin", snask_dir);
    
    println!("📁 Criando diretórios em {}...", snask_dir);
    fs::create_dir_all(&snask_lib).map_err(|e| e.to_string())?;
    fs::create_dir_all(&snask_bin).map_err(|e| e.to_string())?;

    println!("⚙️  Compilando o Runtime Nativo (C)...");
    let status = Command::new("gcc")
        .arg("-c")
        .arg("src/runtime.c")
        .arg("-o")
        .arg(format!("{}/runtime.o", snask_lib))
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("Falha ao compilar o runtime.c. Verifique se o gcc está instalado.".to_string());
    }

    println!("🚚 Instalando binário em {}...", snask_bin);
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    fs::copy(current_exe, format!("{}/snask", snask_bin)).map_err(|e| e.to_string())?;

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

    println!("✅ SNask v0.2.2 configurado com sucesso!");
    println!("Dica: Reinicie seu terminal ou rode 'source ~/.bashrc' para começar a usar o comando 'snask' de qualquer lugar.");
    
    Ok(())
}

fn run_uninstall() -> Result<(), String> {
    println!("🗑️  Desinstalando SNask v0.2.2...");
    
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
    for stmt in program.drain(..) {
        if let StmtKind::Import(path) = &stmt.kind {
            if !resolved_modules.contains(path) {
                resolved_modules.insert(path.clone());
                let mut module_ast = load_module(path)?;
                
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
