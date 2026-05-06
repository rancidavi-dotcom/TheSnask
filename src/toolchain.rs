use std::path::PathBuf;
use std::process::Command;

pub fn clang() -> PathBuf {
    resolve_llvm18_tool(
        "SNASK_CLANG",
        &[
            "clang-18",
            "/usr/bin/clang-18",
            "/usr/lib/llvm18/bin/clang",
            "/usr/lib/llvm-18/bin/clang",
        ],
        "clang",
    )
    .unwrap_or_else(|| PathBuf::from("clang-18"))
}

pub fn llc() -> PathBuf {
    resolve_llvm18_tool(
        "SNASK_LLC",
        &[
            "llc-18",
            "/usr/bin/llc-18",
            "/usr/lib/llvm18/bin/llc",
            "/usr/lib/llvm-18/bin/llc",
        ],
        "llc",
    )
    .unwrap_or_else(|| PathBuf::from("llc-18"))
}

pub fn llvm_strip() -> Option<PathBuf> {
    resolve_llvm18_tool(
        "SNASK_LLVM_STRIP",
        &[
            "llvm-strip-18",
            "/usr/bin/llvm-strip-18",
            "/usr/lib/llvm18/bin/llvm-strip",
            "/usr/lib/llvm-18/bin/llvm-strip",
        ],
        "llvm-strip",
    )
}

pub fn ld_lld() -> Option<PathBuf> {
    resolve_tool(
        "SNASK_LD_LLD",
        &[
            "ld.lld",
            "ld.lld-18",
            "/usr/bin/ld.lld",
            "/usr/bin/ld.lld-18",
            "/usr/lib/llvm18/bin/ld.lld",
            "/usr/lib/llvm-18/bin/ld.lld",
        ],
    )
}

pub fn tool_display(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

pub fn command_exists(tool: &PathBuf) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn resolve_llvm18_tool(env_var: &str, versioned: &[&str], generic: &str) -> Option<PathBuf> {
    if let Some(path) = resolve_env_tool(env_var) {
        return Some(path);
    }

    for candidate in versioned {
        let path = PathBuf::from(candidate);
        if command_exists(&path) {
            return Some(path);
        }
    }

    let generic_path = PathBuf::from(generic);
    if command_major_version(&generic_path) == Some(18) {
        return Some(generic_path);
    }

    None
}

fn resolve_tool(env_var: &str, candidates: &[&str]) -> Option<PathBuf> {
    if let Some(path) = resolve_env_tool(env_var) {
        return Some(path);
    }

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if command_exists(&path) {
            return Some(path);
        }
    }

    None
}

fn resolve_env_tool(env_var: &str) -> Option<PathBuf> {
    let value = std::env::var(env_var).ok()?;
    let path = PathBuf::from(value);
    if command_exists(&path) {
        Some(path)
    } else {
        None
    }
}

fn command_major_version(tool: &PathBuf) -> Option<u32> {
    let output = Command::new(tool).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for token in text.split(|c: char| !(c.is_ascii_digit() || c == '.')) {
        if token.is_empty() {
            continue;
        }
        if let Some(first) = token.split('.').next() {
            if let Ok(major) = first.parse::<u32>() {
                return Some(major);
            }
        }
    }

    None
}
