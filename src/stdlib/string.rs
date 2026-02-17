use crate::value::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;

/// Cria e retorna o objeto do módulo `string` com todas as suas funções (exceto format).
pub fn create_module() -> Value {
    let mut module = HashMap::new();

    module.insert("len".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("string.len espera 1 argumento".to_string()); }
        match &args[0] {
            Value::String(s) => Ok(Value::Number(s.len() as f64)),
            Value::List(l) => Ok(Value::Number(l.len() as f64)),
            _ => Err("string.len espera uma string ou lista".to_string()),
        }
    }));

    module.insert("upper".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("string.upper espera 1 argumento".to_string()); }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err("string.upper espera uma string".to_string()),
        }
    }));

    module.insert("lower".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("string.lower espera 1 argumento".to_string()); }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err("string.lower espera uma string".to_string()),
        }
    }));

    module.insert("trim".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("string.trim espera 1 argumento".to_string()); }
        match &args[0] {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err("string.trim espera uma string".to_string()),
        }
    }));

    module.insert("split".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("string.split espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::String(s), Value::String(delimiter)) => {
                let parts: Vec<Value> = s.split(delimiter.as_str())
                    .map(|part| Value::String(part.to_string()))
                    .collect();
                Ok(Value::List(parts))
            },
            _ => Err("string.split espera duas strings".to_string()),
        }
    }));

    module.insert("join".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("string.join espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::List(list), Value::String(separator)) => {
                let strings: Result<Vec<String>, String> = list.iter().map(|v| {
                    match v {
                        Value::String(s) => Ok(s.clone()),
                        _ => Err("string.join espera uma lista de strings".to_string()),
                    }
                }).collect();

                match strings {
                    Ok(strs) => Ok(Value::String(strs.join(separator.as_str()))),
                    Err(e) => Err(e),
                }
            },
            _ => Err("string.join espera uma lista e uma string".to_string()),
        }
    }));

    module.insert("replace".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 3 { return Err("string.replace espera 3 argumentos".to_string()); }
        match (&args[0], &args[1], &args[2]) {
            (Value::String(s), Value::String(from), Value::String(to)) => {
                Ok(Value::String(s.replace(from.as_str(), to.as_str())))
            },
            _ => Err("string.replace espera três strings".to_string()),
        }
    }));

    module.insert("contains".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("string.contains espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::String(s), Value::String(substr)) => {
                Ok(Value::Boolean(s.contains(substr.as_str())))
            },
            _ => Err("string.contains espera duas strings".to_string()),
        }
    }));

    module.insert("starts_with".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("string.starts_with espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::String(s), Value::String(prefix)) => {
                Ok(Value::Boolean(s.starts_with(prefix.as_str())))
            },
            _ => Err("string.starts_with espera duas strings".to_string()),
        }
    }));

    module.insert("ends_with".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("string.ends_with espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::String(s), Value::String(suffix)) => {
                Ok(Value::Boolean(s.ends_with(suffix.as_str())))
            },
            _ => Err("string.ends_with espera duas strings".to_string()),
        }
    }));

    module.insert("chars".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("string.chars espera 1 argumento".to_string()); }
        match &args[0] {
            Value::String(s) => {
                let chars: Vec<Value> = s.chars()
                    .map(|c| Value::String(c.to_string()))
                    .collect();
                Ok(Value::List(chars))
            },
            _ => Err("string.chars espera uma string".to_string()),
        }
    }));

    module.insert("substring".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 3 { return Err("string.substring espera 3 argumentos".to_string()); }
        match (&args[0], &args[1], &args[2]) {
            (Value::String(s), Value::Number(start), Value::Number(end)) => {
                let start_idx = (*start as usize).min(s.len());
                let end_idx = (*end as usize).min(s.len());
                
                if start_idx > end_idx {
                    return Err("índice inicial maior que índice final".to_string());
                }

                let substr: String = s.chars()
                    .skip(start_idx)
                    .take(end_idx - start_idx)
                    .collect();
                
                Ok(Value::String(substr))
            },
            _ => Err("string.substring espera uma string e dois números".to_string()),
        }
    }));

    let dict_map = module.into_iter().map(|(k, v)| (Value::String(k), v)).collect();
    Value::Dict(dict_map)
}

/// Retorna a função 'format' para ser registrada globalmente.
pub fn get_global_format_function() -> Value {
    Value::NativeFunction(|args, _interpreter| {
        if args.is_empty() { return Err("format espera pelo menos 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(template) => {
                let mut result = template.clone();
                
                for (i, arg) in args[1..].iter().enumerate() {
                    let placeholder = format!("{{{}}}", i);
                    let value_str = match arg {
                        Value::Number(n) => n.to_string(),
                        Value::String(s) => s.clone(),
                        Value::Boolean(b) => b.to_string(),
                        Value::List(_) => "[lista]".to_string(),
                        Value::Dict(_) => "{dict}".to_string(),
                        Value::Nil => "nil".to_string(),
                        _ => "?".to_string(),
                    };
                    result = result.replace(&placeholder, &value_str);
                }
                
                Ok(Value::String(result))
            },
            _ => Err("format espera uma string como primeiro argumento".to_string()),
        }
    })
}
