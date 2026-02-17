use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn snask_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".snask"))
        .ok_or_else(|| "Não foi possível encontrar o diretório home.".to_string())
}

fn registry_dir() -> Result<PathBuf, String> {
    Ok(snask_home_dir()?.join("registry"))
}

fn run_git(args: &[&str], cwd: &Path) -> Result<(), String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Falha ao executar git {:?}: {}", args, e))?;
    if !out.status.success() {
        return Err(format!(
            "git {:?} falhou.\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

fn ensure_registry_repo() -> Result<PathBuf, String> {
    let repo = registry_dir()?;
    if !repo.exists() {
        return Err(format!(
            "Registry não encontrado em '{}'. Rode um comando de pacotes (ex: `snask search json`) para clonar o registry primeiro.",
            repo.display()
        ));
    }
    if !repo.join(".git").exists() {
        return Err(format!(
            "Pasta '{}' existe, mas não é um repo git (sem .git). Apague e rode `snask search json` para recriar.",
            repo.display()
        ));
    }
    // Mantém atualizado antes de publicar
    let _ = run_git(&["fetch", "--all", "--prune"], &repo);
    run_git(&["pull", "--ff-only"], &repo)
        .map_err(|e| format!("Falha ao atualizar registry via git: {}", e))?;
    Ok(repo)
}

#[derive(Debug, Clone)]
pub struct NewLibOpts {
    pub name: String,
    pub description: String,
    pub version: String,
}

pub fn lib_init(opts: NewLibOpts) -> Result<(), String> {
    let name = opts.name.trim();
    if name.is_empty() {
        return Err("Nome inválido.".to_string());
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' ) {
        return Err("Nome inválido: use apenas a-z, 0-9 e '_' (minúsculo).".to_string());
    }

    let file = format!("{}.snask", name);
    if Path::new(&file).exists() {
        return Err(format!("Arquivo '{}' já existe no diretório atual.", file));
    }

    let content = format!(
        "// Biblioteca: {name}\n// Versão: {version}\n// Descrição: {desc}\n//\n// Dica: mantenha a API pública como funções top-level.\n\nfun version()\n    return \"{version}\";\n\nfun about()\n    return \"{desc}\";\n\n// Exemplo de função pública\nfun hello(nome)\n    return \"Olá, \" + nome;\n",
        name = name,
        version = opts.version,
        desc = opts.description.replace('\"', "\\\"")
    );
    fs::write(&file, content).map_err(|e| format!("Falha ao criar '{}': {}", file, e))?;

    let readme = format!(
        "# {name}\n\n- Versão: `{version}`\n- Descrição: {desc}\n\n## Uso\n\n```snask\nimport \"{name}\"\n\nclass main\n    fun start()\n        print({name}::hello(\"dev\"));\n```\n",
        name = name,
        version = opts.version,
        desc = opts.description
    );
    fs::write(format!("{}_README.md", name), readme)
        .map_err(|e| format!("Falha ao criar README: {}", e))?;

    println!("✅ Criado: {} e {}_README.md", file, name);
    println!("📦 Próximo passo: `snask lib publish {}` (publica no SnaskPackages).", name);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct PublishOpts {
    pub name: String,
    pub version: String,
    pub description: String,
    pub message: Option<String>,
    pub push: bool,
    pub pr: bool,
    pub fork: Option<String>,
    pub branch: Option<String>,
}

pub fn lib_publish(opts: PublishOpts) -> Result<(), String> {
    let name = opts.name.trim();
    if name.is_empty() {
        return Err("Nome inválido.".to_string());
    }
    let local_file = PathBuf::from(format!("{}.snask", name));
    if !local_file.exists() {
        return Err(format!("Não achei '{}' no diretório atual.", local_file.display()));
    }

    let repo = ensure_registry_repo()?;

    // Evita bagunçar o repo do registry caso esteja "sujo"
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo)
        .output()
        .map_err(|e| format!("Falha ao checar status do git: {}", e))?;
    if !out.status.success() {
        return Err("Falha ao checar status do git no registry.".to_string());
    }
    if !String::from_utf8_lossy(&out.stdout).trim().is_empty() {
        return Err(format!(
            "Seu registry em '{}' tem mudanças pendentes. Commit/reverta antes de publicar.",
            repo.display()
        ));
    }

    // Se for PR, cria uma branch e envia para o fork
    let target_branch = opts.branch.clone().unwrap_or_else(|| format!("pkg/{}-v{}", name, opts.version));
    if opts.pr {
        // Garante que estamos na main antes de criar branch
        run_git(&["checkout", "main"], &repo)?;
        run_git(&["checkout", "-B", &target_branch], &repo)?;
    }

    let packages_dir = repo.join("packages");
    let index_dir = repo.join("index").join(name.chars().next().unwrap().to_ascii_lowercase().to_string());
    fs::create_dir_all(&packages_dir).map_err(|e| format!("Falha ao criar {}: {}", packages_dir.display(), e))?;
    fs::create_dir_all(&index_dir).map_err(|e| format!("Falha ao criar {}: {}", index_dir.display(), e))?;

    // Copia o arquivo da lib para o repo do registry
    let dest_pkg = packages_dir.join(format!("{}.snask", name));
    fs::copy(&local_file, &dest_pkg).map_err(|e| format!("Falha ao copiar para {}: {}", dest_pkg.display(), e))?;

    // Escreve metadados do índice (formato simples v1)
    let desc = opts.description.replace('\"', "\\\"");
    let url = format!("{}.snask", name);
    let index_path = index_dir.join(format!("{}.json", name));
    let index_json = format!(
        "{{\n  \"version\": \"{version}\",\n  \"url\": \"{url}\",\n  \"description\": \"{desc}\"\n}}\n",
        version = opts.version,
        url = url,
        desc = desc
    );
    fs::write(&index_path, index_json).map_err(|e| format!("Falha ao escrever {}: {}", index_path.display(), e))?;

    // Stage + commit
    run_git(&["add", dest_pkg.to_string_lossy().as_ref(), index_path.to_string_lossy().as_ref()], &repo)?;
    let msg = opts.message.unwrap_or_else(|| format!("pkg: publish {} v{}", name, opts.version));
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
