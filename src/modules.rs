use std::path::{Path, PathBuf};
use std::fs;
use crate::ast::Program;
use crate::parser::parse_program;
use crate::packages::get_packages_dir;

pub fn load_module(path: &str) -> Result<Program, String> {
    // 1. Tenta carregar do diretório local (ex: import "utils.snask")
    let local_path = if path.ends_with(".snask") {
        PathBuf::from(path)
    } else {
        PathBuf::from(format!("{}.snask", path))
    };

    if local_path.exists() {
        return read_and_parse_module(&local_path);
    }

    // 2. Tenta carregar do diretório global de pacotes (~/.snask/packages/)
    let global_packages_dir = get_packages_dir();
    let global_path = if path.ends_with(".snask") {
        global_packages_dir.join(path)
    } else {
        global_packages_dir.join(format!("{}.snask", path))
    };

    if global_path.exists() {
        return read_and_parse_module(&global_path);
    }

    Err(format!(
        "Módulo '{}' não encontrado localmente nem em ~/.snask/packages/",
        path
    ))
}

fn read_and_parse_module(path: &Path) -> Result<Program, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Falha ao ler módulo {}: {}", path.display(), e))?;
    
    parse_program(&source)
        .map_err(|e| format!("Erro de sintaxe no módulo {}: {}", path.display(), e))
}
