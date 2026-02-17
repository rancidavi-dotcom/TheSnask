use std::path::PathBuf;
use std::fs;
use serde::Deserialize;
use std::collections::HashMap;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/registry.json";
const BASE_PKG_URL: &str = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/packages/";

#[derive(Deserialize, Debug)]
struct Package {
    version: String,
    url: String, // can be relative to BASE_PKG_URL or absolute
    description: String,
}

#[derive(Deserialize, Debug)]
struct Registry {
    packages: HashMap<String, Package>,
}

pub fn get_packages_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Não foi possível encontrar o diretório home.");
    let dir = home.join(".snask").join("packages");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Falha ao criar diretório de pacotes.");
    }
    dir
}

pub fn install_package(name: &str) -> Result<(), String> {
    println!("🔍 Buscando pacote '{}' no registry oficial...", name);
    
    // 1. Fetch registry.json
    let response = reqwest::blocking::get(REGISTRY_URL)
        .map_err(|e| format!("Falha ao acessar o registry: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Erro ao acessar registry: HTTP {}", response.status()));
    }

    let registry: Registry = response.json()
        .map_err(|e| format!("Erro ao processar registry.json: {}", e))?;

    // 2. Check if package exists
    let package = registry.packages.get(name)
        .ok_or_else(|| format!("Pacote '{}' não encontrado no registry.", name))?;

    println!("📦 Pacote encontrado! Versão: {}", package.version);
    println!("📝 Descrição: {}", package.description);

    // 3. Download the package file (.snask)
    let download_url = if package.url.starts_with("http") {
        package.url.clone()
    } else {
        format!("{}{}", BASE_PKG_URL, package.url)
    };

    println!("📥 Baixando de: {}", download_url);

    let pkg_response = reqwest::blocking::get(&download_url)
        .map_err(|e| format!("Falha ao baixar pacote: {}", e))?;

    if !pkg_response.status().is_success() {
        return Err(format!("Erro ao baixar arquivo do pacote: HTTP {}", pkg_response.status()));
    }

    let content = pkg_response.text()
        .map_err(|e| format!("Falha ao ler conteúdo do pacote: {}", e))?;

    // 4. Save to local packages directory
    let packages_dir = get_packages_dir();
    let file_name = if package.url.ends_with(".snask") {
        package.url.split('/').last().unwrap().to_string()
    } else {
        format!("{}.snask", name)
    };
    
    let dest_path = packages_dir.join(file_name);
    fs::write(&dest_path, content)
        .map_err(|e| format!("Falha ao salvar pacote localmente: {}", e))?;

    println!("✅ Pacote '{}' instalado com sucesso em: {}", name, dest_path.display());
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
