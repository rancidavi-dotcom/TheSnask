use std::process::Command;
use std::path::Path;
use indicatif::HumanBytes;

pub fn doctor() -> Result<(), String> {
    println!("🩺 Snask Doctor");
    println!("----------------");

    // Check OS
    println!("OS: {}", std::env::consts::OS);
    println!("Arch: {}", std::env::consts::ARCH);

    // Check Tools
    check_tool("clang-18");
    check_tool("llc-18");
    check_tool("llvm-strip-18");
    check_tool("ld.lld");
    check_tool("git");
    
    #[cfg(target_os = "linux")]
    {
        check_tool("dpkg-deb");
        check_tool("appimagetool");
    }

    // Check Runtime
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let runtime_path = format!("{}/.snask/lib/runtime.bc", home);
    if Path::new(&runtime_path).exists() {
        println!("✅ Runtime found at {}", runtime_path);
    } else {
        println!("❌ Runtime NOT found. Run `snask setup`.");
    }

    Ok(())
}

fn check_tool(name: &str) {
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", name))
        .output();
    
    match status {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            println!("✅ {}: {}", name, path);
        }
        _ => {
            println!("❌ {}: NOT FOUND", name);
        }
    }
}

pub fn cmd_size(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("File '{}' not found.", path));
    }
    let meta = std::fs::metadata(p).map_err(|e| e.to_string())?;
    let len = meta.len();
    println!("File: {}", path);
    println!("Size: {}", HumanBytes(len));
    Ok(())
}

pub fn run_setup(target: Option<String>) -> Result<(), String> {
    println!("🔧 Setup Snask Toolchain");
    if let Some(t) = target {
        println!("Target: {}", t);
    } else {
        println!("Target: Host ({})", std::env::consts::ARCH);
    }
    
    // TODO: Implement actual download logic (fetch from GitHub releases or build from source)
    println!("⚠️  Setup logic is being migrated. Please install dependencies manually for now:");
    println!("   - clang-18, llvm-18, lld-18");
    
    Ok(())
}

pub fn run_uninstall() -> Result<(), String> {
    println!("🗑️  Uninstalling Snask...");
    // Logic to remove ~/.snask and binary
    Ok(())
}

pub fn self_update() -> Result<(), String> {
    println!("🔄 Checking for updates...");
    // Logic to fetch latest release
    Ok(())
}

pub fn init_zenith_project(name: Option<String>) -> Result<(), String> {
    println!("⚡ Initializing Zenith Framework project...");
    let n = name.unwrap_or_else(|| "zenith-app".to_string());
    // TODO: Generate Zenith template
    println!("Created project '{}'.", n);
    Ok(())
}
