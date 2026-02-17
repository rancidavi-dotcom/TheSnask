use crate::value::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Cria e retorna o objeto do módulo `io` com todas as suas funções.
pub fn create_module() -> Value {
    let mut module = HashMap::new();

    module.insert("read_file".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.read_file espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                match fs::read_to_string(path) {
                    Ok(content) => Ok(Value::String(content)),
                    Err(e) => Err(format!("Erro ao ler arquivo: {}", e)),
                }
            },
            _ => Err("io.read_file espera uma string (caminho do arquivo)".to_string()),
        }
    }));

    module.insert("write_file".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("io.write_file espera 2 argumentos".to_string()); }
        
        match (&args[0], &args[1]) {
            (Value::String(path), Value::String(content)) => {
                match fs::write(path, content) {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("Erro ao escrever arquivo: {}", e)),
                }
            },
            _ => Err("io.write_file espera duas strings (caminho e conteúdo)".to_string()),
        }
    }));

    module.insert("append_file".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("io.append_file espera 2 argumentos".to_string()); }
        
        match (&args[0], &args[1]) {
            (Value::String(path), Value::String(content)) => {
                use std::fs::OpenOptions;
                use std::io::Write;

                match OpenOptions::new().create(true).append(true).open(path) {
                    Ok(mut file) => {
                        match file.write_all(content.as_bytes()) {
                            Ok(_) => Ok(Value::Boolean(true)),
                            Err(e) => Err(format!("Erro ao adicionar ao arquivo: {}", e)),
                        }
                    },
                    Err(e) => Err(format!("Erro ao abrir arquivo: {}", e)),
                }
            },
            _ => Err("io.append_file espera duas strings (caminho e conteúdo)".to_string()),
        }
    }));

    module.insert("exists".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.exists espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                Ok(Value::Boolean(Path::new(path).exists()))
            },
            _ => Err("io.exists espera uma string (caminho)".to_string()),
        }
    }));

    module.insert("delete".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.delete espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                let path_obj = Path::new(path);
                
                let result = if path_obj.is_dir() {
                    fs::remove_dir_all(path)
                } else {
                    fs::remove_file(path)
                };

                match result {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("Erro ao deletar: {}", e)),
                }
            },
            _ => Err("io.delete espera uma string (caminho)".to_string()),
        }
    }));

    module.insert("read_dir".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.read_dir espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                match fs::read_dir(path) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        for entry in entries {
                            if let Ok(entry) = entry {
                                if let Some(name) = entry.path().file_name() {
                                    if let Some(name_str) = name.to_str() {
                                        files.push(Value::String(name_str.to_string()));
                                    }
                                }
                            }
                        }
                        Ok(Value::List(files))
                    },
                    Err(e) => Err(format!("Erro ao ler diretório: {}", e)),
                }
            },
            _ => Err("io.read_dir espera uma string (caminho do diretório)".to_string()),
        }
    }));

    module.insert("is_file".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.is_file espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                Ok(Value::Boolean(Path::new(path).is_file()))
            },
            _ => Err("io.is_file espera uma string (caminho)".to_string()),
        }
    }));

    module.insert("is_dir".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.is_dir espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                Ok(Value::Boolean(Path::new(path).is_dir()))
            },
            _ => Err("io.is_dir espera uma string (caminho)".to_string()),
        }
    }));

    module.insert("create_dir".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("io.create_dir espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(path) => {
                match fs::create_dir_all(path) {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("Erro ao criar diretório: {}", e)),
                }
            },
            _ => Err("io.create_dir espera uma string (caminho)".to_string()),
        }
    }));
    
    let dict_map = module.into_iter().map(|(k, v)| (Value::String(k), v)).collect();
    Value::Dict(dict_map)
}
