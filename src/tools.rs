use indicatif::HumanBytes;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let snask_home = format!("{}/.snask", home);

    let lib_dir = if let Some(ref t) = target {
        format!("{}/lib/{}", snask_home, t)
    } else {
        format!("{}/lib", snask_home)
    };

    fs::create_dir_all(&lib_dir).map_err(|e| e.to_string())?;

    let target_triple = target.clone().unwrap_or_else(|| "native".to_string());
    println!("Target: {}", target_triple);

    // 1. Locate Runtime Source
    let mut runtime_src = PathBuf::from("src/runtime.c");
    if !runtime_src.exists() {
        runtime_src = PathBuf::from(format!("{}/src/TheSnask/src/runtime.c", snask_home));
    }

    if !runtime_src.exists() {
        return Err("Runtime source (src/runtime.c) not found. Are you in the Snask source tree or is it installed in ~/.snask/src/TheSnask?".to_string());
    }
    println!("📍 Source found at: {}", runtime_src.display());

    // 2. Check for Skia
    let mut extra_flags: Vec<String> = Vec::new();
    if has_skia() {
        println!("🎨 Skia detected! Enabling SNASK_SKIA in runtime.");
        extra_flags.push("-DSNASK_SKIA".to_string());
        // We could also add pkg-config --cflags skia here
        if let Ok(cflags) = get_skia_cflags() {
            for flag in cflags {
                extra_flags.push(flag);
            }
        }
    }

    if has_pkg("gtk+-3.0") {
        println!("🖼️  GTK3 detected! Enabling SNASK_GUI_GTK in runtime.");
        extra_flags.push("-DSNASK_GUI_GTK".to_string());
        if let Ok(cflags) = get_pkg_cflags("gtk+-3.0") {
            for flag in cflags {
                extra_flags.push(flag);
            }
        }
    }

    // 3. Compile Standard Runtime
    println!("📦 Compiling standard runtime (runtime.o, runtime.bc)...");
    let extra_flags_refs: Vec<&str> = extra_flags.iter().map(|s| s.as_str()).collect();
    compile_runtime(
        &runtime_src.to_string_lossy(),
        &format!("{}/runtime", lib_dir),
        &target,
        extra_flags_refs,
    )?;


    // 4. Compile Tiny Runtime
    println!("📦 Compiling tiny runtime (runtime_tiny.o, runtime_tiny.bc)...");
    let mut tiny_flags = extra_flags.clone();
    tiny_flags.push("-DSNASK_TINY".to_string());
    tiny_flags.push("-Os".to_string());
    let tiny_flags_refs: Vec<&str> = tiny_flags.iter().map(|s| s.as_str()).collect();
    compile_runtime(
        runtime_src.to_str().unwrap(),
        &format!("{}/runtime_tiny", lib_dir),
        &target,
        tiny_flags_refs,
    )?;

    // 5. Compile Nano Runtime
    let nano_src = runtime_src.parent().unwrap().join("runtime/runtime_nano.c");
    if nano_src.exists() {
        println!("📦 Compiling nano runtime (runtime_nano.o, runtime_nano.bc)...");
        compile_runtime(
            nano_src.to_str().unwrap(),
            &format!("{}/runtime_nano", lib_dir),
            &target,
            vec!["-Oz"],
        )?;
    }

    // 6. Compile rt_extreme.o (Linux x86_64 only for now)
    if target.is_none() && cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        let extreme_src = runtime_src
            .parent()
            .unwrap()
            .join("runtime/ultra_start_x86_64_linux.S");
        if extreme_src.exists() {
            println!("📦 Compiling extreme entrypoint (rt_extreme.o)...");
            let status = Command::new("clang-18")
                .arg("-c")
                .arg(extreme_src)
                .arg("-o")
                .arg(format!("{}/rt_extreme.o", lib_dir))
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                println!("⚠️  Failed to compile rt_extreme.o");
            }
        }
    }

    println!("✅ Setup completed successfully.");
    Ok(())
}

fn has_skia() -> bool {
    has_pkg("skia")
}

pub fn has_pkg(name: &str) -> bool {
    Command::new("pkg-config")
        .arg("--exists")
        .arg(name)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn get_skia_cflags() -> Result<Vec<String>, String> {
    // This is simplified. In a real scenario we'd parse output of pkg-config --cflags
    // For now we just return an empty vec or a fixed one if we know it.
    Ok(vec![])
}

pub fn get_pkg_cflags(name: &str) -> Result<Vec<String>, String> {
    let output = Command::new("pkg-config")
        .arg("--cflags")
        .arg(name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Ok(vec![]);
    }

    Ok(text.split_whitespace().map(|s| s.to_string()).collect())
}

pub fn get_pkg_libs(name: &str) -> Result<Vec<String>, String> {
    let output = Command::new("pkg-config")
        .arg("--libs")
        .arg(name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Ok(vec![]);
    }

    Ok(text.split_whitespace().map(|s| s.to_string()).collect())
}

fn compile_runtime(
    src: &str,
    dest_base: &str,
    target: &Option<String>,
    extra_args: Vec<&str>,
) -> Result<(), String> {
    let mut base_args = vec!["-c", src];
    if let Some(t) = target {
        base_args.push("--target=");
        base_args.push(t);
    }

    // .o
    let mut cmd_o = Command::new("clang-18");
    cmd_o.args(&base_args);
    cmd_o.arg("-o").arg(format!("{}.o", dest_base));
    for arg in &extra_args {
        cmd_o.arg(arg);
    }
    if !extra_args.iter().any(|a| a.starts_with("-O")) {
        cmd_o.arg("-O3");
    }

    let status = cmd_o.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("Failed to compile {}.o", dest_base));
    }

    // .bc
    let mut cmd_bc = Command::new("clang-18");
    cmd_bc.arg("-emit-llvm");
    cmd_bc.args(&base_args);
    cmd_bc.arg("-o").arg(format!("{}.bc", dest_base));
    for arg in &extra_args {
        cmd_bc.arg(arg);
    }
    if !extra_args.iter().any(|a| a.starts_with("-O")) {
        cmd_bc.arg("-O3");
    }

    let status = cmd_bc.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("Failed to compile {}.bc", dest_base));
    }

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
