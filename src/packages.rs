use std::path::PathBuf;
use std::fs;
use serde::Deserialize;
use std::collections::HashMap;
use sha2::{Digest, Sha256};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// Opção B (git): o registry é um repositório git local em ~/.snask/registry (clone/pull do SnaskPackages).
// Isso permite evoluir de um único registry.json para um índice por pacote (index/**/<pkg>.json) sem depender de um servidor.
const REGISTRY_GIT_URL: &str = "https://github.com/rancidavi-dotcom/SnaskPackages";
const REGISTRY_HTTP_FALLBACK_URL: &str = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/registry.json";
const BASE_PKG_URL: &str = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/packages/";

#[derive(Deserialize, Debug)]
pub struct Package {
    version: String,
    #[serde(default)]
    url: String, // can be relative to BASE_PKG_URL or absolute; default: "<name>.snask"
    #[serde(default)]
    description: String,
}

impl Package {
    pub fn version(&self) -> &str { &self.version }
    pub fn url(&self) -> &str { &self.url }
    pub fn description(&self) -> &str { &self.description }
}

#[derive(Deserialize, Debug)]
pub struct Registry {
    pub packages: HashMap<String, Package>,
}

fn snask_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".snask"))
        .ok_or_else(|| "Não foi possível encontrar o diretório home.".to_string())
}

pub fn get_packages_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Não foi possível encontrar o diretório home.");
    let dir = home.join(".snask").join("packages");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Falha ao criar diretório de pacotes.");
    }
    dir
}

fn registry_repo_dir() -> Result<PathBuf, String> {
    Ok(snask_home_dir()?.join("registry"))
}

fn run_git(args: &[&str], cwd: Option<&PathBuf>) -> Result<(), String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let out = cmd.output().map_err(|e| format!("Falha ao executar git {:?}: {}", args, e))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Err(format!("git {:?} falhou.\nstdout: {}\nstderr: {}", args, stdout.trim(), stderr.trim()));
    }
    Ok(())
}

fn ensure_registry_repo() -> Result<PathBuf, String> {
    let home = snask_home_dir()?;
    fs::create_dir_all(&home).map_err(|e| format!("Falha ao criar {}: {}", home.display(), e))?;

    let repo = registry_repo_dir()?;
    if repo.join(".git").exists() {
        // Atualiza o registry via git pull.
        // (Sem rebase para evitar conflitos caso o usuário tenha mexido localmente.)
        let _ = run_git(&["fetch", "--all", "--prune"], Some(&repo));
        run_git(&["pull", "--ff-only"], Some(&repo)).map_err(|e| {
            format!("Falha ao atualizar o registry via git. Dica: apague '{}' e rode novamente.\n{}", repo.display(), e)
        })?;
        return Ok(repo);
    }

    // Primeiro clone
    run_git(&["clone", "--depth", "1", REGISTRY_GIT_URL, repo.to_string_lossy().as_ref()], None)?;
    Ok(repo)
}

fn read_registry_from_repo(repo: &PathBuf) -> Result<Registry, String> {
    // Preferência: índice por pacote em index/**/<name>.json
    let index_dir = repo.join("index");
    if index_dir.exists() {
        let mut packages: HashMap<String, Package> = HashMap::new();
        let mut stack = vec![index_dir];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).map_err(|e| format!("Falha ao ler índice {}: {}", dir.display(), e))? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let bytes = fs::read(&path).map_err(|e| format!("Falha ao ler {}: {}", path.display(), e))?;
                let pkg: Package = serde_json::from_slice(&bytes)
                    .map_err(|e| format!("JSON inválido em {}: {}", path.display(), e))?;
                // O nome do pacote vem do filename (sem .json), para evitar inconsistências.
                let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                if !name.is_empty() {
                    packages.insert(name, pkg);
                }
            }
        }
        return Ok(Registry { packages });
    }

    // Compatibilidade: registry.json na raiz do repo
    let legacy = repo.join("registry.json");
    if legacy.exists() {
        let bytes = fs::read(&legacy).map_err(|e| format!("Falha ao ler {}: {}", legacy.display(), e))?;
        return serde_json::from_slice(&bytes).map_err(|e| format!("Erro ao processar registry.json (git): {}", e));
    }

    Err(format!("Registry inválido: não achei 'index/' nem 'registry.json' em '{}'.", repo.display()))
}

pub fn fetch_registry() -> Result<Registry, String> {
    // Primeiro tenta o modo git (opção B). Se falhar, cai no HTTP como fallback.
    match ensure_registry_repo().and_then(|repo| read_registry_from_repo(&repo)) {
        Ok(r) => Ok(r),
        Err(git_err) => {
            eprintln!("⚠️  Registry via git falhou, usando fallback HTTP. ({})", git_err);
            let response = reqwest::blocking::get(REGISTRY_HTTP_FALLBACK_URL)
                .map_err(|e| format!("Falha ao acessar o registry (HTTP): {}", e))?;
            if !response.status().is_success() {
                return Err(format!("Erro ao acessar registry (HTTP): HTTP {}", response.status()));
            }
            response
                .json()
                .map_err(|e| format!("Erro ao processar registry.json (HTTP): {}", e))
        }
    }
}

pub fn is_package_installed(name: &str) -> bool {
    let packages_dir = get_packages_dir();
    packages_dir.join(format!("{}.snask", name)).exists()
}

pub fn read_installed_package_sha256(name: &str) -> Result<String, String> {
    let packages_dir = get_packages_dir();
    let path = packages_dir.join(format!("{}.snask", name));
    let bytes = fs::read(&path).map_err(|e| format!("Falha ao ler pacote instalado {}: {}", path.display(), e))?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{:x}", hash))
}

pub fn read_installed_package_version_from_registry(name: &str, registry: &Registry) -> Option<String> {
    registry.packages.get(name).map(|p| p.version.clone())
}

pub fn install_package_with_registry(name: &str, registry: &Registry) -> Result<(String, String, PathBuf), String> {
    // returns (version, sha256, path)
    let package = registry.packages.get(name)
        .ok_or_else(|| format!("Pacote '{}' não encontrado no registry.", name))?;

    let url = if package.url.trim().is_empty() {
        format!("{}.snask", name)
    } else {
        package.url.clone()
    };
    let file_name = if url.ends_with(".snask") {
        url.split('/').last().unwrap().to_string()
    } else {
        format!("{}.snask", name)
    };

    // Preferência: ler do registry git local (~/.snask/registry/packages/<file>).
    let content: Vec<u8> = (|| -> Option<Vec<u8>> {
        let repo = registry_repo_dir().ok()?;
        let local = repo.join("packages").join(&file_name);
        if local.exists() {
            return fs::read(&local).ok();
        }
        None
    })().unwrap_or_else(|| Vec::new());

    let content = if !content.is_empty() {
        content
    } else {
        // Fallback: baixa por HTTP
        let download_url = if url.starts_with("http") { url.clone() } else { format!("{}{}", BASE_PKG_URL, url) };
        // GitHub raw pode ficar em cache; adiciona um cache-buster simples.
        let download_url = if download_url.contains("raw.githubusercontent.com") && !download_url.contains('?') {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            format!("{}?t={}", download_url, ts)
        } else {
            download_url
        };

        let pkg_response = reqwest::blocking::get(&download_url)
            .map_err(|e| format!("Falha ao baixar pacote: {}", e))?;
        if !pkg_response.status().is_success() {
            return Err(format!("Erro ao baixar arquivo do pacote: HTTP {}", pkg_response.status()));
        }
        pkg_response.bytes().map_err(|e| format!("Falha ao ler bytes do pacote: {}", e))?.to_vec()
    };

    let hash = Sha256::digest(&content);
    let sha256 = format!("{:x}", hash);

    let packages_dir = get_packages_dir();
    let dest_path = packages_dir.join(file_name);
    fs::write(&dest_path, &content)
        .map_err(|e| format!("Falha ao salvar pacote localmente: {}", e))?;

    Ok((package.version.clone(), sha256, dest_path))
}

pub fn install_package(name: &str) -> Result<(), String> {
    println!("🔍 Buscando pacote '{}' no registry oficial...", name);
    
    // 1. Fetch registry.json
    let registry: Registry = fetch_registry()?;

    // 2. Check if package exists
    let package = registry.packages.get(name)
        .ok_or_else(|| format!("Pacote '{}' não encontrado no registry.", name))?;

    println!("📦 Pacote encontrado! Versão: {}", package.version);
    println!("📝 Descrição: {}", package.description);

    // 3/4. Download and save (and compute hash)
    let (ver, _sha, dest_path) = install_package_with_registry(name, &registry)?;

    println!("✅ Pacote '{}' instalado com sucesso em: {}", name, dest_path.display());
    println!("📌 Versão instalada: {}", ver);
    println!("💡 Use: import \"{}\" no seu código Snask.", name);

    Ok(())
}

pub fn uninstall_package(name: &str) -> Result<(), String> {
    let packages_dir = get_packages_dir();
    let file_name = format!("{}.snask", name);
    let path = packages_dir.join(file_name);

    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Falha ao remover pacote: {}", e))?;
        println!("🗑️  Pacote '{}' desinstalado com sucesso.", name);
        Ok(())
    } else {
        Err(format!("Pacote '{}' não está instalado.", name))
    }
}

pub fn list_packages() -> Result<(), String> {
    let packages_dir = get_packages_dir();
    println!("📦 Pacotes Snask instalados em {}:", packages_dir.display());
    
    let entries = fs::read_dir(packages_dir).map_err(|e| e.to_string())?;
    let mut found = false;

    for entry in entries {
        if let Ok(entry) = entry {
            let name = entry.file_name().into_string().unwrap_or_default();
            if name.ends_with(".snask") {
                println!(" - {}", name.trim_end_matches(".snask"));
                found = true;
            }
        }
    }

    if !found {
        println!(" (nenhum pacote encontrado)");
    }
    Ok(())
}

pub fn search_packages(query: &str) -> Result<(), String> {
    println!("🔍 Pesquisando por '{}' no registry...", query);
    
    let registry: Registry = fetch_registry()?;

    let mut found = false;
    for (name, package) in registry.packages {
        if name.contains(query) || package.description.contains(query) {
            println!("✨ {} (v{})", name, package.version);
            println!("   Description: {}", package.description);
            println!("   URL: {}", package.url);
            println!("");
            found = true;
        }
    }

    if !found {
        println!("Nenhum pacote encontrado para a busca: {}", query);
    }
    Ok(())
}
