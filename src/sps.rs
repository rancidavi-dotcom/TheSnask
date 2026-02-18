use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::snif_parser::{parse_snif, SnifValue};

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
            return Err("SPS: required field is empty: package.name".to_string());
        }
        if self.package.version.trim().is_empty() {
            return Err("SPS: required field is empty: package.version".to_string());
        }
        if self.package.entry.trim().is_empty() {
            return Err("SPS: required field is empty: package.entry".to_string());
        }
        if self.build.opt_level > 3 {
            return Err("SPS: build.opt_level must be between 0 and 3".to_string());
        }
        if self.profile.release.opt_level > 3 || self.profile.dev.opt_level > 3 {
            return Err("SPS: profile.*.opt_level must be between 0 and 3".to_string());
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
    let snif = start_dir.join("snask.snif");
    if snif.exists() {
        return Some(snif);
    }
    let toml = start_dir.join("snask.toml");
    if toml.exists() {
        return Some(toml);
    }
    None
}

pub fn load_manifest_from(dir: &Path) -> Result<(SpsManifest, PathBuf), String> {
    let manifest_path = find_manifest(dir).ok_or_else(|| "SPS: snask.snif not found in the current directory".to_string())?;
    let src = fs::read_to_string(&manifest_path).map_err(|e| format!("SPS: failed to read {}: {}", manifest_path.display(), e))?;
    let m: SpsManifest = if manifest_path.extension().and_then(|s| s.to_str()) == Some("snif") {
        manifest_from_snif(&src).map_err(|e| format!("SPS: failed to parse snask.snif:\n{}", e))?
    } else {
        toml::from_str(&src).map_err(|e| {
            format!(
                "SPS: failed to parse snask.toml: {}\n\nNote: snask.toml is deprecated. Migrate to snask.snif.\n",
                e
            )
        })?
    };
    m.validate()?;
    Ok((m, manifest_path))
}

fn render_snif_error(source: &str, line: usize, col: usize) -> String {
    let mut out = String::new();
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || line > lines.len() {
        return out;
    }
    let l = lines[line - 1];
    out.push_str(&format!("  --> {}:{}\n", line, col));
    out.push_str("   |\n");
    out.push_str(&format!("{:>3} | {}\n", line, l));
    let caret_pos = col.saturating_sub(1);
    let spaces = " ".repeat(caret_pos.min(l.len()));
    out.push_str(&format!("   | {}^\n", spaces));
    out
}

pub fn write_manifest(path: &Path, manifest: &SpsManifest) -> Result<(), String> {
    if path.extension().and_then(|s| s.to_str()) == Some("snif") {
        if path.exists() {
            if let Ok(src) = fs::read_to_string(path) {
                if let Ok(updated) = update_snif_dependencies_in_place(&src, &manifest.dependencies) {
                    fs::write(path, updated).map_err(|e| e.to_string())?;
                    return Ok(());
                }
            }
        }
        // Fallback: rewrite the whole file (still valid SNIF).
        let s = manifest_to_snif(manifest);
        fs::write(path, s).map_err(|e| e.to_string())?;
        return Ok(());
    }
    let s = toml::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(path, s).map_err(|e| e.to_string())?;
    Ok(())
}

fn update_snif_dependencies_in_place(src: &str, deps: &BTreeMap<String, String>) -> Result<String, String> {
    // Goal: preserve formatting and comments by editing only the `dependencies: { ... }` object.
    // This is a best-effort text transform. If it fails, the caller falls back to rewriting.

    // Find the start of a `dependencies` key at top level-ish by scanning tokens while tracking strings/comments/braces.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode { Normal, Str(char), Comment }

    let bytes = src.as_bytes();
    let mut mode = Mode::Normal;
    let mut i: usize = 0;

    fn is_ident_start(c: u8) -> bool { (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' || c == b'$' }
    fn is_ident_char(c: u8) -> bool { is_ident_start(c) || (c >= b'0' && c <= b'9') || c == b'-' }

    while i < bytes.len() {
        match mode {
            Mode::Comment => {
                if bytes[i] == b'\n' { mode = Mode::Normal; }
                i += 1;
                continue;
            }
            Mode::Str(q) => {
                if bytes[i] == b'\\' { i += 2; continue; }
                if bytes[i] as char == q { mode = Mode::Normal; }
                i += 1;
                continue;
            }
            Mode::Normal => {}
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            mode = Mode::Comment;
            i += 2;
            continue;
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            mode = Mode::Str(bytes[i] as char);
            i += 1;
            continue;
        }
        if bytes[i] == b'{' { i += 1; continue; }
        if bytes[i] == b'}' { i += 1; continue; }

        // Only try to match key when not inside strings/comments.
        if is_ident_start(bytes[i]) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_char(bytes[i]) { i += 1; }
            let ident = &src[start..i];
            if ident == "dependencies" {
                // Skip whitespace/comments, then expect ':', then skip, then expect '{'
                let mut j = i;
                // skip ws + comments
                loop {
                    while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r' || bytes[j] == b'\n') { j += 1; }
                    if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'/' {
                        j += 2;
                        while j < bytes.len() && bytes[j] != b'\n' { j += 1; }
                        continue;
                    }
                    break;
                }
                if j >= bytes.len() || bytes[j] != b':' { continue; }
                j += 1;
                // skip ws/comments again
                loop {
                    while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r' || bytes[j] == b'\n') { j += 1; }
                    if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'/' {
                        j += 2;
                        while j < bytes.len() && bytes[j] != b'\n' { j += 1; }
                        continue;
                    }
                    break;
                }
                if j >= bytes.len() || bytes[j] != b'{' { continue; }

                // Find matching '}' for this dependencies object
                let obj_open = j;
                let mut k = j + 1;
                let mut local_mode = Mode::Normal;
                let mut local_depth: i32 = 1;
                while k < bytes.len() {
                    match local_mode {
                        Mode::Comment => {
                            if bytes[k] == b'\n' { local_mode = Mode::Normal; }
                            k += 1;
                            continue;
                        }
                        Mode::Str(q) => {
                            if bytes[k] == b'\\' { k += 2; continue; }
                            if bytes[k] as char == q { local_mode = Mode::Normal; }
                            k += 1;
                            continue;
                        }
                        Mode::Normal => {}
                    }
                    if k + 1 < bytes.len() && bytes[k] == b'/' && bytes[k + 1] == b'/' { local_mode = Mode::Comment; k += 2; continue; }
                    if bytes[k] == b'"' || bytes[k] == b'\'' { local_mode = Mode::Str(bytes[k] as char); k += 1; continue; }
                    if bytes[k] == b'{' { local_depth += 1; k += 1; continue; }
                    if bytes[k] == b'}' {
                        local_depth -= 1;
                        if local_depth == 0 {
                            let obj_close = k;
                            // Determine indentation for entries by looking backward from obj_open to line start.
                            let line_start = src[..obj_open].rfind('\n').map(|x| x + 1).unwrap_or(0);
                            let base_indent = &src[line_start..obj_open].chars().take_while(|c| *c == ' ' || *c == '\t').collect::<String>();
                            let entry_indent = format!("{}  ", base_indent);

                            let mut block = String::new();
                            if deps.is_empty() {
                                // keep empty with no extra lines
                                block.push_str("{");
                                block.push_str("}");
                            } else {
                                block.push_str("{\n");
                                for (name, ver) in deps {
                                    // SNIF strict: values must be quoted strings.
                                    let key = if is_ident_start(name.as_bytes()[0]) && name.as_bytes().iter().all(|&c| is_ident_char(c)) {
                                        name.clone()
                                    } else {
                                        format!("\"{}\"", name.replace('\"', ""))
                                    };
                                    block.push_str(&format!("{indent}{key}: \"{ver}\",\n", indent = entry_indent, key = key, ver = ver.replace('\"', "")));
                                }
                                block.push_str(&format!("{}}}", base_indent));
                            }

                            let mut out = String::new();
                            out.push_str(&src[..obj_open]);
                            out.push_str(&block);
                            out.push_str(&src[obj_close + 1..]);
                            return Ok(out);
                        }
                        k += 1;
                        continue;
                    }
                    k += 1;
                }
                return Err("Could not find end of dependencies object.".to_string());
            }
            continue;
        }
        i += 1;
    }

    Err("dependencies section not found".to_string())
}

fn manifest_to_snif(m: &SpsManifest) -> String {
    fn is_ident(s: &str) -> bool {
        let mut it = s.chars();
        let Some(c0) = it.next() else { return false; };
        let start_ok = c0.is_ascii_alphabetic() || c0 == '_' || c0 == '$';
        if !start_ok {
            return false;
        }
        for c in it {
            let ok = c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '-';
            if !ok {
                return false;
            }
        }
        true
    }

    fn fmt_key(k: &str) -> String {
        if is_ident(k) {
            k.to_string()
        } else {
            format!("\"{}\"", k.replace('\"', ""))
        }
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  package: { ");
    out.push_str(&format!("name: \"{}\", ", m.package.name.replace('\"', "")));
    out.push_str(&format!("version: \"{}\", ", m.package.version.replace('\"', "")));
    out.push_str(&format!("entry: \"{}\", ", m.package.entry.replace('\"', "")));
    out.push_str("},\n");

    out.push_str("  dependencies: {\n");
    for (k, v) in &m.dependencies {
        out.push_str(&format!("    {}: \"{}\",\n", fmt_key(k), v.replace('\"', "")));
    }
    out.push_str("  },\n");

    out.push_str(&format!("  build: {{ opt_level: {}, }},\n", m.build.opt_level));
    if !m.scripts.is_empty() {
        out.push_str("  scripts: {\n");
        for (k, v) in &m.scripts {
            out.push_str(&format!("    {}: \"{}\",\n", k, v.replace('\"', "")));
        }
        out.push_str("  },\n");
    }
    out.push_str("}\n");
    out
}

fn snif_get_obj<'a>(v: &'a SnifValue, path: &str) -> Result<&'a std::collections::BTreeMap<String, SnifValue>, String> {
    match v {
        SnifValue::Object(o) => Ok(o),
        _ => Err(format!("Expected object at {path}")),
    }
}

fn snif_get_str(o: &std::collections::BTreeMap<String, SnifValue>, key: &str, default: Option<String>) -> Result<String, String> {
    match o.get(key) {
        None => default.ok_or_else(|| format!("Missing required field: {key}")),
        Some(SnifValue::String(s)) => Ok(s.clone()),
        Some(_) => Err(format!("Expected string for field: {key}")),
    }
}

fn snif_get_u8(o: &std::collections::BTreeMap<String, SnifValue>, key: &str, default: u8) -> Result<u8, String> {
    match o.get(key) {
        None => Ok(default),
        Some(SnifValue::Number(n)) => Ok(*n as u8),
        Some(_) => Err(format!("Expected number for field: {key}")),
    }
}

fn snif_get_map(o: &std::collections::BTreeMap<String, SnifValue>, key: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    let Some(v) = o.get(key) else { return Ok(BTreeMap::new()); };
    let m = snif_get_obj(v, key)?;
    let mut out = BTreeMap::new();
    for (k, v) in m {
        match v {
            SnifValue::String(s) => { out.insert(k.clone(), s.clone()); }
            SnifValue::Null => { out.insert(k.clone(), "*".to_string()); }
            _ => return Err(format!("Expected string (version) in {key}.{k}")),
        }
    }
    Ok(out)
}

fn manifest_from_snif(src: &str) -> Result<SpsManifest, String> {
    let root = parse_snif(src).map_err(|e| {
        let mut msg = String::new();
        msg.push_str(&format!("{}\n", e.message));
        msg.push_str(&render_snif_error(src, e.line, e.col));
        msg.push_str("\nHow to fix:\n- Run `snask init` to generate a valid `snask.snif` template.\n- Or compare your file against the example in `docs/SPS.md`.\n");
        msg
    })?;
    let root_obj = snif_get_obj(&root, "root")?;

    let pkg_v = root_obj.get("package").ok_or_else(|| {
        "Missing required section: package\n\nHow to fix:\n- Add:\n  package: { name: \"app\", version: \"0.1.0\", entry: \"main.snask\", },\n".to_string()
    })?;
    let pkg = snif_get_obj(pkg_v, "package")?;
    let package = PackageSection {
        name: snif_get_str(pkg, "name", None)?,
        version: snif_get_str(pkg, "version", None)?,
        entry: snif_get_str(pkg, "entry", Some(default_entry()))?,
    };

    let empty_build = SnifValue::Object(BTreeMap::new());
    let build_v = root_obj.get("build").unwrap_or(&empty_build);
    let build_obj = snif_get_obj(build_v, "build")?;
    let build = BuildSection { opt_level: snif_get_u8(build_obj, "opt_level", default_opt_level())? };

    let dependencies = snif_get_map(root_obj, "dependencies")?;

    let scripts = match root_obj.get("scripts") {
        None => BTreeMap::new(),
        Some(v) => {
            let o = snif_get_obj(v, "scripts")?;
            let mut out = BTreeMap::new();
            for (k, v) in o {
                match v {
                    SnifValue::String(s) => { out.insert(k.clone(), s.clone()); }
                    _ => return Err(format!("Expected string in scripts.{k}")),
                }
            }
            out
        }
    };

    let profile = ProfileSection::default();

    Ok(SpsManifest { package, dependencies, build, scripts, profile })
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Lockfile {
    pub package: LockPackage,
    #[serde(default)]
    pub registry: Option<LockRegistry>,
    pub dependencies: BTreeMap<String, LockedDep>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LockRegistry {
    pub git_url: String,
    pub rev: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LockedDep {
    pub version: String,
    pub sha256: String,
    #[serde(default)]
    pub url: Option<String>,
}

pub fn lockfile_path(dir: &Path) -> PathBuf {
    dir.join("snask.lock")
}

pub fn read_lockfile(dir: &Path) -> Result<Lockfile, String> {
    let p = lockfile_path(dir);
    let s = fs::read_to_string(&p).map_err(|e| format!("SPS: failed to read {}: {}", p.display(), e))?;
    toml::from_str(&s).map_err(|e| {
        format!(
            "SPS: failed to parse snask.lock: {}\n\nTip: delete `snask.lock` and run `snask build` to regenerate it.",
            e
        )
    })
}

pub fn write_lockfile(dir: &Path, manifest: &SpsManifest, deps: BTreeMap<String, LockedDep>) -> Result<(), String> {
    // Best-effort: registra a revisão do registry local (git) para auditoria/debug.
    // Reprodutibilidade real é garantida por sha256 em cada dep.
    let registry = registry_head().ok().map(|rev| LockRegistry {
        git_url: "https://github.com/rancidavi-dotcom/SnaskPackages".to_string(),
        rev,
    });

    let lf = Lockfile {
        package: LockPackage { name: manifest.package.name.clone(), version: manifest.package.version.clone() },
        registry,
        dependencies: deps,
    };
    let s = toml::to_string_pretty(&lf).map_err(|e| e.to_string())?;
    fs::write(lockfile_path(dir), s).map_err(|e| e.to_string())?;
    Ok(())
}

fn registry_head() -> Result<String, String> {
    let home = dirs::home_dir().ok_or_else(|| "home directory not found".to_string())?;
    let repo = home.join(".snask").join("registry");
    if !repo.join(".git").exists() {
        return Err("registry git repo not found".to_string());
    }
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&repo)
        .output()
        .map_err(|e| format!("Failed to run git rev-parse: {}", e))?;
    if !out.status.success() {
        return Err("git rev-parse HEAD failed".to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
