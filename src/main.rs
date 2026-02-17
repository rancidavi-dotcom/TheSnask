pub mod ast;
pub mod value;
pub mod symbol_table;
pub mod semantic_analyzer;
pub mod types;
pub mod parser;
pub mod interpreter;
pub mod modules;
pub mod stdlib;
pub mod span;
pub mod diagnostics;
pub mod packages;
pub mod llvm_generator;

use std::fs;
use std::process::Command;
use clap::{Parser as ClapParser, Subcommand};
use crate::parser::Parser;
use crate::semantic_analyzer::SemanticAnalyzer;
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
    Build { file: String, #[arg(short, long)] output: Option<String> },
    Install { name: String },
    Uninstall { name: String },
    Update { name: String },
    List,
    Search { query: String },
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Build { file, output } => {
            match build_file(file, output.clone()) {
                Ok(_) => println!("Compilação LLVM concluída com sucesso."),
                Err(e) => eprintln!("Erro durante a compilação: {}", e),
            }
        },
        Commands::Install { name } => {
            if let Err(e) = packages::install_package(name) {
                eprintln!("Erro ao instalar pacote: {}", e);
            }
        },
        Commands::Uninstall { name } => {
            if let Err(e) = packages::uninstall_package(name) {
                eprintln!("Erro ao desinstalar pacote: {}", e);
            }
        },
        Commands::Update { name } => {
            println!("🔄 Atualizando pacote '{}'...", name);
            if let Err(e) = packages::install_package(name) {
                eprintln!("Erro ao atualizar pacote: {}", e);
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

fn build_file(file_path: &str, output_name: Option<String>) -> Result<(), String> {
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut program = Parser::new(&source)?.parse_program().map_err(|e| format!("Erro no parser: {}", e))?;

    let mut resolved_program = Vec::new();
    let mut resolved_modules = std::collections::HashSet::new();
    resolved_modules.insert(file_path.to_string());
    resolve_imports(&mut program, &mut resolved_program, &mut resolved_modules)?;

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&resolved_program);
    if !analyzer.errors.is_empty() { return Err(format!("Análise semântica encontrou erros: {:?}", analyzer.errors)); }

    let context = Context::create();
    let mut generator = LLVMGenerator::new(&context, file_path);
    let ir = generator.generate(resolved_program)?;

    let ir_file = "temp_snask.ll";
    let obj_file = "temp_snask.o";
    fs::write(ir_file, ir).map_err(|e| e.to_string())?;

    Command::new("llc-18").arg("-relocation-model=pic").arg("-filetype=obj").arg(ir_file).arg("-o").arg(obj_file).status().map_err(|e| e.to_string())?;

    let runtime_path = format!("{}/.snask/lib/runtime.o", std::env::var("HOME").unwrap());
    let final_output = output_name.unwrap_or_else(|| file_path.replace(".snask", ""));

    let status = Command::new("clang-18").arg(obj_file).arg(runtime_path).arg("-o").arg(&final_output).arg("-lm").status().map_err(|e| e.to_string())?;

    if !status.success() { return Err("Falha na linkagem final.".to_string()); }
    fs::remove_file(ir_file).ok(); fs::remove_file(obj_file).ok();
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

                // Renomeia funções do módulo para incluir o namespace
                for m_stmt in &mut module_ast {
                    if let StmtKind::FuncDeclaration(f) = &mut m_stmt.kind {
                        f.name = format!("{}::{}", module_name, f.name);
                    }
                }

                resolve_imports(&mut module_ast, resolved_program, resolved_modules)?;
            }
        } else { resolved_program.push(stmt); }
    }
    Ok(())
}
