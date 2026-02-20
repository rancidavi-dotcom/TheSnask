use std::path::{Path, PathBuf};
use std::fs;
use crate::ast::Program;
use crate::parser::parse_program;
use crate::packages::get_packages_dir;

pub fn load_module(path: &str) -> Result<Program, String> {
    load_module_from(Path::new("."), path)
}

pub fn load_module_from(base_dir: &Path, path: &str) -> Result<Program, String> {
    // 1. Try local project dir first (relative to the importing file)
    let local_path = if path.ends_with(".snask") {
        base_dir.join(path)
    } else {
        base_dir.join(format!("{}.snask", path))
    };
    if local_path.exists() {
        return read_and_parse_module(&local_path);
    }

    // 2. Try global packages dir (~/.snask/packages/)
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
        "Module '{}' not found in '{}' nor in ~/.snask/packages/",
        path,
        base_dir.display()
    ))
}

pub fn load_from_import(
    base_dir: &Path,
    from: &[String],
    is_current_dir: bool,
    module: &str,
) -> Result<(Program, PathBuf), String> {
    let dir = if is_current_dir {
        base_dir.to_path_buf()
    } else {
        from.iter().fold(base_dir.to_path_buf(), |acc, s| acc.join(s))
    };
    let file_path = dir.join(format!("{}.snask", module));
    if !file_path.exists() {
        return Err(format!("Module '{}' not found at {}", module, file_path.display()));
    }
    let program = read_and_parse_module(&file_path)?;
    Ok((program, file_path))
}

fn read_and_parse_module(path: &Path) -> Result<Program, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read module {}: {}", path.display(), e))?;
    
    parse_program(&source)
        .map_err(|e| format!("Syntax error in module {}: {}", path.display(), e))
}
