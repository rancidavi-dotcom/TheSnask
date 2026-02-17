use crate::value::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cria e retorna o objeto do módulo `sys` com todas as suas funções.
pub fn create_module() -> Value {
    let mut module = HashMap::new();

    module.insert("time".to_string(), Value::NativeFunction(|_args| {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Ok(Value::Number(since_the_epoch.as_secs_f64()))
    }));

    module.insert("sleep".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("sys.sleep espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::Number(ms) => {
                let duration = std::time::Duration::from_millis(*ms as u64);
                std::thread::sleep(duration);
                Ok(Value::Nil)
            },
            _ => Err("sys.sleep espera um número (milissegundos)".to_string()),
        }
    }));

    module.insert("exit".to_string(), Value::NativeFunction(|args, _interpreter| {
        let code = if args.is_empty() {
            0
        } else {
            match &args[0] {
                Value::Number(n) => *n as i32,
                _ => 0,
            }
        };
        
        std::process::exit(code);
    }));

    module.insert("args".to_string(), Value::NativeFunction(|_args| {
        let args: Vec<Value> = std::env::args()
            .skip(1) // Pula o nome do executável
            .map(|arg| Value::String(arg))
            .collect();
        Ok(Value::List(args))
    }));

    module.insert("env".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("sys.env espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(key) => {
                match std::env::var(key) {
                    Ok(value) => Ok(Value::String(value)),
                    Err(_) => Ok(Value::Nil),
                }
            },
            _ => Err("sys.env espera uma string (nome da variável)".to_string()),
        }
    }));

    module.insert("set_env".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("sys.set_env espera 2 argumentos".to_string()); }
        
        match (&args[0], &args[1]) {
            (Value::String(key), Value::String(value)) => {
                std::env::set_var(key, value);
                Ok(Value::Nil)
            },
            _ => Err("sys.set_env espera duas strings".to_string()),
        }
    }));

    module.insert("cwd".to_string(), Value::NativeFunction(|_args| {
        match std::env::current_dir() {
            Ok(path) => {
                if let Some(path_str) = path.to_str() {
                    Ok(Value::String(path_str.to_string()))
                } else {
                    Err("Erro ao converter caminho".to_string())
                }
            },
            Err(e) => Err(format!("Erro ao obter diretório atual: {}", e)),
        }
    }));

    module.insert("platform".to_string(), Value::NativeFunction(|_args| {
        Ok(Value::String(std::env::consts::OS.to_string()))
    }));

    module.insert("arch".to_string(), Value::NativeFunction(|_args| {
        Ok(Value::String(std::env::consts::ARCH.to_string()))
    }));
    
    let dict_map = module.into_iter().map(|(k, v)| (Value::String(k), v)).collect();
    Value::Dict(dict_map)
}
