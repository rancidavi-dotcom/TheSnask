use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpsManifest {
    pub package: PackageSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub build: BuildSection,
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,
    #[serde(default)]
    pub profile: ProfileSection,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    #[serde(default = "default_entry")]
    pub entry: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BuildSection {
    #[serde(default = "default_opt_level")]
    pub opt_level: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ProfileSection {
    #[serde(default)]
    pub release: BuildSection,
    #[serde(default)]
    pub dev: BuildSection,
}

fn default_entry() -> String {
    "main.snask".to_string()
}

fn default_opt_level() -> u8 {
    2
}

impl SpsManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.package.name.trim().is_empty() {
            return Err("SPS: campo obrigatório vazio: [package].name".to_string());
        }
        if self.package.version.trim().is_empty() {
            return Err("SPS: campo obrigatório vazio: [package].version".to_string());
        }
        if self.package.entry.trim().is_empty() {
            return Err("SPS: campo obrigatório vazio: [package].entry".to_string());
        }
        if self.build.opt_level > 3 {
            return Err("SPS: [build].opt_level deve estar entre 0 e 3".to_string());
        }
        if self.profile.release.opt_level > 3 || self.profile.dev.opt_level > 3 {
            return Err("SPS: [profile.*].opt_level deve estar entre 0 e 3".to_string());
        }
        Ok(())
    }

    pub fn opt_level_for(&self, is_release: bool) -> u8 {
        if is_release && self.profile.release.opt_level != 0 {
            return self.profile.release.opt_level;
        }
        if !is_release && self.profile.dev.opt_level != 0 {
            return self.profile.dev.opt_level;
        }
        self.build.opt_level
    }
}

pub fn find_manifest(start_dir: &Path) -> Option<PathBuf> {
    // MVP: procura apenas no diretório atual (sem subir árvore)
    let p = start_dir.join("snask.toml");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

pub fn load_manifest_from(dir: &Path) -> Result<(SpsManifest, PathBuf), String> {
    let manifest_path = find_manifest(dir).ok_or_else(|| "SPS: snask.toml não encontrado no diretório atual".to_string())?;
    let src = fs::read_to_string(&manifest_path).map_err(|e| format!("SPS: falha ao ler {}: {}", manifest_path.display(), e))?;
    let m: SpsManifest = toml::from_str(&src).map_err(|e| format!("SPS: erro ao parsear snask.toml: {}", e))?;
    m.validate()?;
    Ok((m, manifest_path))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Lockfile {
    pub package: LockPackage,
    pub dependencies: BTreeMap<String, LockedDep>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LockedDep {
    pub version: String,
    pub sha256: String,
}

pub fn lockfile_path(dir: &Path) -> PathBuf {
    dir.join("snask.lock")
}

pub fn write_lockfile(dir: &Path, manifest: &SpsManifest, deps: BTreeMap<String, LockedDep>) -> Result<(), String> {
    let lf = Lockfile {
        package: LockPackage { name: manifest.package.name.clone(), version: manifest.package.version.clone() },
        dependencies: deps,
    };
    let s = toml::to_string_pretty(&lf).map_err(|e| e.to_string())?;
    fs::write(lockfile_path(dir), s).map_err(|e| e.to_string())?;
    Ok(())
}
