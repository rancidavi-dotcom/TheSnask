use std::path::PathBuf;
use std::fs;
use serde::Deserialize;
use std::collections::HashMap;
use sha2::{Digest, Sha256};

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/registry.json";
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

pub fn get_packages_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Não foi possível encontrar o diretório home.");
    let dir = home.join(".snask").join("packages");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Falha ao criar diretório de pacotes.");
    }
    dir
}

pub fn fetch_registry() -> Result<Registry, String> {
    let response = reqwest::blocking::get(REGISTRY_URL)
        .map_err(|e| format!("Falha ao acessar o registry: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("Erro ao acessar registry: HTTP {}", response.status()));
    }
    response.json().map_err(|e| format!("Erro ao processar registry.json: {}", e))
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
    let download_url = if url.starts_with("http") { url.clone() } else { format!("{}{}", BASE_PKG_URL, url) };

    let pkg_response = reqwest::blocking::get(&download_url)
        .map_err(|e| format!("Falha ao baixar pacote: {}", e))?;
    if !pkg_response.status().is_success() {
        return Err(format!("Erro ao baixar arquivo do pacote: HTTP {}", pkg_response.status()));
    }

    let content = pkg_response.bytes().map_err(|e| format!("Falha ao ler bytes do pacote: {}", e))?;
    let hash = Sha256::digest(&content);
    let sha256 = format!("{:x}", hash);

    let packages_dir = get_packages_dir();
    let file_name = if url.ends_with(".snask") {
        url.split('/').last().unwrap().to_string()
    } else {
        format!("{}.snask", name)
    };
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
    
    let response = reqwest::blocking::get(REGISTRY_URL)
        .map_err(|e| format!("Falha ao acessar o registry: {}", e))?;
    
    let registry: Registry = response.json()
        .map_err(|e| format!("Erro ao processar registry.json: {}", e))?;

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
