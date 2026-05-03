use crate::compiler::{self, BuildOptions};
use crate::sps;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_dist(
    file: Option<String>,
    targets_csv: Option<String>,
    all: bool,
    deb: bool,
    appimage: bool,
    name: Option<String>,
    linux_user: bool,
    out_dir_str: String,
) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // resolve entry file + manifest (optional)
    let (manifest, file_path, features) = if let Ok((m, _p)) = sps::load_manifest_from(&cwd) {
        // pin + resolve deps para garantir build determinístico (e lock)
        sps::pin_from_lock(&cwd, &m)?;
        sps::resolve_deps_and_lock(&cwd, &m)?;
        (
            Some(m.clone()),
            file.unwrap_or_else(|| m.package.entry.clone()),
            m.build.features.clone(),
        )
    } else {
        (None, compiler::resolve_entry_file(file)?, BTreeMap::new())
    };

    // resolve base binary name
    let base_name = name.unwrap_or_else(|| {
        if let Some(m) = &manifest {
            m.package.name.clone()
        } else {
            Path::new(&file_path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        }
    });

    let out_dir = cwd.join(out_dir_str);
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
        let opt_level = manifest
            .as_ref()
            .map(|m| m.opt_level_for(true))
            .unwrap_or(2);

        // dist is production-oriented: default to release-size unless the manifest explicitly chooses a profile.
        let (release_size, tiny, do_strip, opt_override) = if let Some(m) = &manifest {
            let p = m
                .build
                .profile
                .clone()
                .unwrap_or_else(|| "release-size".to_string());
            let tiny = p == "tiny";
            let release_size = p == "release-size" || (!tiny && p == "release");
            let do_strip = m.build.strip.unwrap_or(release_size || tiny);
            let opt_override = m.build.opt.clone().or_else(|| {
                if tiny {
                    Some("Oz".to_string())
                } else {
                    Some("Os".to_string())
                }
            });
            (release_size, tiny, do_strip, opt_override)
        } else {
            (true, false, true, Some("Os".to_string()))
        };

        println!(
            "🔧 build: {} -> {} (profile={})",
            triple,
            out_path.display(),
            if tiny {
                "tiny"
            } else if release_size {
                "release-size"
            } else {
                "release"
            }
        );

        let build_opts = BuildOptions {
            output_name: Some(out_path.to_string_lossy().to_string()),
            target: t.clone(),
            opt_level,
            lto: true, // Always LTO for dist
            release_size,
            min_runtime: false,
            tiny,
            extreme: false,
            strip: do_strip,
            opt_override,
            features: features.clone(),
        };

        compiler::build_file(&file_path, build_opts)?;
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
                return Err(format!(
                    "Para gerar .AppImage, preciso do binário Linux nativo em '{}'.",
                    bin_path.display()
                ));
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
    project_dir: &Path,
    manifest: Option<&sps::SpsManifest>,
    base_name: &str,
    bin_path: &Path,
) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not found.".to_string())?;
    let local_bin = Path::new(&home).join(".local/bin");
    let apps = Path::new(&home).join(".local/share/applications");
    let icons = Path::new(&home).join(".local/share/icons/hicolor/scalable/apps");
    std::fs::create_dir_all(&local_bin).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&apps).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&icons).map_err(|e| e.to_string())?;

    let app = manifest.and_then(|m| m.app.clone());
    let app_id = app.as_ref().map(|a| a.id.as_str()).unwrap_or(base_name);
    let app_name = app.as_ref().map(|a| a.name.as_str()).unwrap_or(base_name);
    let comment = app.as_ref().map(|a| a.comment.as_str()).unwrap_or("");
    let categories = app
        .as_ref()
        .map(|a| a.categories.as_str())
        .unwrap_or("Utility;");
    let terminal = app.as_ref().map(|a| a.terminal).unwrap_or(false);
    let icon_field = app.as_ref().map(|a| a.icon.as_str()).unwrap_or("");

    let dest_bin = local_bin.join(base_name);
    std::fs::copy(bin_path, &dest_bin).map_err(|e| e.to_string())?;
    let mut perms = std::fs::metadata(&dest_bin)
        .map_err(|e| e.to_string())?
        .permissions();
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
fn make_deb(out_dir: &Path, name: &str, bin_path: &Path) -> Result<PathBuf, String> {
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
        let mut perms = std::fs::metadata(&dest_bin)
            .map_err(|e| e.to_string())?
            .permissions();
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
        return Err(
            "Failed to build .deb (dpkg-deb). Install `dpkg-deb` (dpkg package) and try again."
                .to_string(),
        );
    }
    Ok(deb_path)
}

#[cfg(target_os = "linux")]
fn make_appimage(out_dir: &Path, name: &str, bin_path: &Path) -> Result<PathBuf, String> {
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
        let mut perms = std::fs::metadata(&dest_bin)
            .map_err(|e| e.to_string())?
            .permissions();
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
        let mut perms = std::fs::metadata(&apprun_path)
            .map_err(|e| e.to_string())?
            .permissions();
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
