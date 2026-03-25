use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use serde_json::Value as JsonValue;

fn snask_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".snask"))
        .ok_or_else(|| "Could not find the home directory.".to_string())
}

fn registry_dir() -> Result<PathBuf, String> {
    Ok(snask_home_dir()?.join("registry"))
}

fn run_git(args: &[&str], cwd: &Path) -> Result<(), String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to run git {:?}: {}", args, e))?;
    if !out.status.success() {
        return Err(format!(
            "git {:?} failed.\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

fn ensure_registry_repo() -> Result<PathBuf, String> {
    let repo = registry_dir()?;
    let git_url = "https://github.com/rancidavi-dotcom/SnaskPackages";

    if !repo.exists() {
        // Auto-clone: necessário para qualquer dev conseguir publicar via fork/PR.
        fs::create_dir_all(repo.parent().unwrap_or_else(|| Path::new(".")))
            .map_err(|e| format!("Failed to create registry directory: {}", e))?;
        let out = Command::new("git")
            .args(["clone", "--depth", "1", git_url, repo.to_string_lossy().as_ref()])
            .output()
            .map_err(|e| format!("Failed to run git clone: {}", e))?;
        if !out.status.success() {
            return Err(format!(
                "Failed to clone registry.\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&out.stdout).trim(),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
    }

    if !repo.join(".git").exists() {
        return Err(format!(
            "Directory '{}' exists, but it is not a git repo (no .git). Delete it and try again.",
            repo.display()
        ));
    }

    // Mantém atualizado antes de publicar
    let _ = run_git(&["fetch", "--all", "--prune"], &repo);
    run_git(&["pull", "--ff-only"], &repo)
        .map_err(|e| format!("Failed to update registry via git: {}", e))?;
    Ok(repo)
}

#[derive(Debug, Clone)]
pub struct NewLibOpts {
    pub name: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    version: String,
    description: String,
}

pub fn lib_init(opts: NewLibOpts) -> Result<(), String> {
    let name = opts.name.trim();
    if name.is_empty() {
        return Err("Invalid name.".to_string());
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' ) {
        return Err("Invalid name: use only a-z, 0-9 and '_' (lowercase).".to_string());
    }

    let snask_file = format!("{}.snask", name);
    let json_file = "package.json";
    let md_file = "README.md";
    if Path::new(&snask_file).exists() || Path::new(json_file).exists() || Path::new(md_file).exists() {
        return Err(format!(
            "One of these files already exists in the current directory: '{}', '{}' or '{}'.",
            snask_file, json_file, md_file
        ));
    }

    let content = format!(
        "// Library: {name}\n// Version: {version}\n// Description: {desc}\n//\n// Tip: keep the public API as top-level functions.\n\nfun version()\n    return \"{version}\";\n\nfun about()\n    return \"{desc}\";\n\n// Example public function\nfun hello(name)\n    return \"Hello, \" + name;\n",
        name = name,
        version = opts.version,
        desc = opts.description.replace('\"', "\\\"")
    );
    fs::write(&snask_file, content).map_err(|e| format!("Failed to create '{}': {}", snask_file, e))?;

    let pkg_json = format!(
        "{{\n  \"name\": \"{name}\",\n  \"version\": \"{version}\",\n  \"description\": \"{desc}\"\n}}\n",
        name = name,
        version = opts.version,
        desc = opts.description.replace('\"', "\\\"")
    );
    fs::write(json_file, pkg_json).map_err(|e| format!("Failed to create '{}': {}", json_file, e))?;

    let readme = format!(
        "# {name}\n\n{desc}\n\n## Instalação\n\n```bash\nsnask install {name}\n```\n\n## Uso\n\n```snask\nimport \"{name}\"\n\nclass main\n    fun start()\n        print({name}::hello(\"dev\"));\n```\n",
        name = name,
        desc = opts.description
    );
    fs::write(md_file, readme).map_err(|e| format!("Failed to create '{}': {}", md_file, e))?;

    println!("✅ Created: {}, {}, {}", snask_file, json_file, md_file);
    println!("📦 Next step: `snask lib publish {}`.", name);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct PublishOpts {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub message: Option<String>,
    pub push: bool,
    pub pr: bool,
    pub fork: Option<String>,
    pub branch: Option<String>,
}

fn read_required_package_files(lib_name: &str) -> Result<(PackageJson, String, PathBuf), String> {
    let snask_path = PathBuf::from(format!("{}.snask", lib_name));
    if !snask_path.exists() {
        return Err(format!("Arquivo obrigatório não encontrado: '{}'.", snask_path.display()));
    }
    let json_bytes = fs::read("package.json").map_err(|e| format!("Arquivo obrigatório não encontrado: package.json ({})", e))?;
    let pkg: PackageJson = serde_json::from_slice(&json_bytes).map_err(|e| format!("package.json inválido: {}", e))?;
    if pkg.name.trim() != lib_name {
        return Err(format!("package.json name='{}' não bate com a lib '{}'.", pkg.name, lib_name));
    }
    let md = fs::read_to_string("README.md").map_err(|e| format!("Arquivo obrigatório não encontrado: README.md ({})", e))?;
    if md.trim().is_empty() {
        return Err("README.md está vazio.".to_string());
    }
    Ok((pkg, md, snask_path))
}

pub fn lib_publish(opts: PublishOpts) -> Result<(), String> {
    let name = opts.name.trim();
    if name.is_empty() {
        return Err("Nome inválido.".to_string());
    }
    let (pkg, _readme_md, local_file) = read_required_package_files(name)?;
    let version = opts.version.clone().unwrap_or_else(|| pkg.version.clone());
    let description = opts.description.clone().unwrap_or_else(|| pkg.description.clone());

    let repo = ensure_registry_repo()?;

    // Evita bagunçar o repo do registry caso esteja "sujo"
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo)
        .output()
        .map_err(|e| format!("Failed to check git status: {}", e))?;
    if !out.status.success() {
        return Err("Failed to check git status in the registry repo.".to_string());
    }
    if !String::from_utf8_lossy(&out.stdout).trim().is_empty() {
        return Err(format!(
            "Seu registry em '{}' tem mudanças pendentes. Commit/reverta antes de publicar.",
            repo.display()
        ));
    }

    // Se for PR, cria uma branch e envia para o fork
    let target_branch = opts.branch.clone().unwrap_or_else(|| format!("pkg/{}-v{}", name, version));
    if opts.pr {
        // Garante que estamos na main antes de criar branch
        run_git(&["checkout", "main"], &repo)?;
        run_git(&["checkout", "-B", &target_branch], &repo)?;
    }

    let packages_dir = repo.join("packages");
    let packages_src = repo.join("packages_src").join(name).join(&version);
    let index_dir = repo.join("index").join(name.chars().next().unwrap().to_ascii_lowercase().to_string());
    fs::create_dir_all(&packages_dir).map_err(|e| format!("Failed to create {}: {}", packages_dir.display(), e))?;
    fs::create_dir_all(&packages_src).map_err(|e| format!("Failed to create {}: {}", packages_src.display(), e))?;
    fs::create_dir_all(&index_dir).map_err(|e| format!("Failed to create {}: {}", index_dir.display(), e))?;

    // Copia o arquivo da lib para o repo do registry
    let dest_pkg = packages_dir.join(format!("{}.snask", name));
    fs::copy(&local_file, &dest_pkg).map_err(|e| format!("Failed to copy to {}: {}", dest_pkg.display(), e))?;

    // Fonte versionada (com package.json e README.md obrigatórios)
    fs::copy(&local_file, packages_src.join(format!("{}.snask", name)))
        .map_err(|e| format!("Failed to copy sources: {}", e))?;
    fs::copy("package.json", packages_src.join("package.json"))
        .map_err(|e| format!("Failed to copy package.json: {}", e))?;
    fs::copy("README.md", packages_src.join("README.md"))
        .map_err(|e| format!("Failed to copy README.md: {}", e))?;

    // Escreve metadados do índice (formato simples v1)
    let desc = description.replace('\"', "\\\"");
    let url = format!("{}.snask", name);
    let index_path = index_dir.join(format!("{}.json", name));
    let index_json = format!(
        "{{\n  \"version\": \"{version}\",\n  \"url\": \"{url}\",\n  \"description\": \"{desc}\"\n}}\n",
        version = version,
        url = url,
        desc = desc
    );
    fs::write(&index_path, index_json).map_err(|e| format!("Failed to write {}: {}", index_path.display(), e))?;

    // Compatibilidade: também atualiza registry.json legado, para ferramentas antigas que ainda leem um arquivo único.
    let legacy_registry_path = repo.join("registry.json");
    let mut legacy_obj: JsonValue = if legacy_registry_path.exists() {
        let bytes = fs::read(&legacy_registry_path)
            .map_err(|e| format!("Failed to read {}: {}", legacy_registry_path.display(), e))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| format!("registry.json inválido (legacy): {}", e))?
    } else {
        serde_json::json!({ "packages": {} })
    };

    if !legacy_obj.get("packages").map(|v| v.is_object()).unwrap_or(false) {
        legacy_obj["packages"] = serde_json::json!({});
    }
    legacy_obj["packages"][name] = serde_json::json!({
        "version": version,
        "url": format!("{name}.snask"),
        "description": description,
    });
    let legacy_pretty = serde_json::to_string_pretty(&legacy_obj).map_err(|e| e.to_string())? + "\n";
    fs::write(&legacy_registry_path, legacy_pretty)
        .map_err(|e| format!("Failed to write {}: {}", legacy_registry_path.display(), e))?;

    // Stage + commit
    run_git(
        &[
            "add",
            dest_pkg.to_string_lossy().as_ref(),
            index_path.to_string_lossy().as_ref(),
            packages_src.to_string_lossy().as_ref(),
            legacy_registry_path.to_string_lossy().as_ref(),
        ],
        &repo,
    )?;
    let msg = opts.message.unwrap_or_else(|| format!("pkg: publish {} v{}", name, version));
    run_git(&["commit", "-m", &msg], &repo).map_err(|e| {
        // se nada mudou, commit falha; dá uma msg melhor
        if e.contains("nothing to commit") {
            "Nada para commitar (pacote/index já estavam iguais).".to_string()
        } else {
            e
        }
    })?;

    if opts.pr {
        // Usa remote "fork" para push
        let fork_url = opts
            .fork
            .clone()
            .ok_or_else(|| "Modo PR exige `--fork <URL-do-seu-fork>`.".to_string())?;

        // cria/atualiza remote fork
        let _ = run_git(&["remote", "remove", "fork"], &repo);
        run_git(&["remote", "add", "fork", &fork_url], &repo)?;
        run_git(&["push", "-u", "fork", &target_branch], &repo)?;

        // volta para main para não confundir o usuário
        let _ = run_git(&["checkout", "main"], &repo);

        println!("✅ Branch enviada para seu fork (remote 'fork'): {}", target_branch);
        println!("📌 Abra um Pull Request no GitHub do seu fork → base: main, compare: {}.", target_branch);
        println!("ℹ️  Fork usado: {}", fork_url);
        return Ok(());
    }

    if opts.push {
        run_git(&["push", "origin", "main"], &repo)?;
        println!("✅ Publicado e enviado para o GitHub via git push.");
    } else {
        println!("✅ Publicado localmente no registry em '{}'.", repo.display());
        println!("📌 Para enviar: `cd {} && git push origin main`", repo.display());
    }
    Ok(())
}
