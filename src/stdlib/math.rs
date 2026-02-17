use crate::value::Value;
use crate::symbol_table::SymbolTable;

/// Registra funções matemáticas na stdlib
pub fn register(globals: &mut SymbolTable) {
    // Funções básicas
    globals.define_native_function("abs", |args, _interpreter| {
        if args.len() != 1 { return Err("abs espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.abs())),
            _ => Err("abs espera um número".to_string()),
        }
    });

    globals.define_native_function("floor", |args, _interpreter| {
        if args.len() != 1 { return Err("floor espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.floor())),
            _ => Err("floor espera um número".to_string()),
        }
    });

    globals.define_native_function("ceil", |args, _interpreter| {
        if args.len() != 1 { return Err("ceil espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.ceil())),
            _ => Err("ceil espera um número".to_string()),
        }
    });

    globals.define_native_function("round", |args, _interpreter| {
        if args.len() != 1 { return Err("round espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.round())),
            _ => Err("round espera um número".to_string()),
        }
    });

    globals.define_native_function("pow", |args, _interpreter| {
        if args.len() != 2 { return Err("pow espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(base), Value::Number(exp)) => Ok(Value::Number(base.powf(*exp))),
            _ => Err("pow espera dois números".to_string()),
        }
    });

    globals.define_native_function("sqrt", |args, _interpreter| {
        if args.len() != 1 { return Err("sqrt espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                if *n < 0.0 {
                    Err("sqrt não aceita números negativos".to_string())
                } else {
                    Ok(Value::Number(n.sqrt()))
                }
            },
            _ => Err("sqrt espera um número".to_string()),
        }
    });

    globals.define_native_function("log", |args, _interpreter| {
        if args.len() != 1 { return Err("log espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                if *n <= 0.0 {
                    Err("log não aceita números não-positivos".to_string())
                } else {
                    Ok(Value::Number(n.ln()))
                }
            },
            _ => Err("log espera um número".to_string()),
        }
    });

    globals.define_native_function("log10", |args, _interpreter| {
        if args.len() != 1 { return Err("log10 espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                if *n <= 0.0 {
                    Err("log10 não aceita números não-positivos".to_string())
                } else {
                    Ok(Value::Number(n.log10()))
                }
            },
            _ => Err("log10 espera um número".to_string()),
        }
    });

    globals.define_native_function("exp", |args, _interpreter| {
        if args.len() != 1 { return Err("exp espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.exp())),
            _ => Err("exp espera um número".to_string()),
        }
    });

    globals.define_native_function("min", |args, _interpreter| {
        if args.is_empty() { return Err("min espera pelo menos 1 argumento".to_string()); }
        
        let mut min_val = match &args[0] {
            Value::Number(n) => n,
            _ => return Err("min espera números".to_string()),
        };

        for arg in &args[1..] {
            match arg {
                Value::Number(n) => {
                    if n < min_val {
                        min_val = n;
                    }
                },
                _ => return Err("min espera números".to_string()),
            }
        }

        Ok(Value::Number(*min_val))
    });

    globals.define_native_function("max", |args, _interpreter| {
        if args.is_empty() { return Err("max espera pelo menos 1 argumento".to_string()); }
        
        let mut max_val = match &args[0] {
            Value::Number(n) => n,
            _ => return Err("max espera números".to_string()),
        };

        for arg in &args[1..] {
            match arg {
                Value::Number(n) => {
                    if n > max_val {
                        max_val = n;
                    }
                },
                _ => return Err("max espera números".to_string()),
            }
        }

        Ok(Value::Number(*max_val))
    });

    // Trigonometria
    globals.define_native_function("sin", |args, _interpreter| {
        if args.len() != 1 { return Err("sin espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.sin())),
            _ => Err("sin espera um número".to_string()),
        }
    });

    globals.define_native_function("cos", |args, _interpreter| {
        if args.len() != 1 { return Err("cos espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.cos())),
            _ => Err("cos espera um número".to_string()),
        }
    });

    globals.define_native_function("tan", |args, _interpreter| {
        if args.len() != 1 { return Err("tan espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.tan())),
            _ => Err("tan espera um número".to_string()),
        }
    });

    globals.define_native_function("asin", |args, _interpreter| {
        if args.len() != 1 { return Err("asin espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                if *n < -1.0 || *n > 1.0 {
                    Err("asin espera um número entre -1 e 1".to_string())
                } else {
                    Ok(Value::Number(n.asin()))
                }
            },
            _ => Err("asin espera um número".to_string()),
        }
    });

    globals.define_native_function("acos", |args, _interpreter| {
        if args.len() != 1 { return Err("acos espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                if *n < -1.0 || *n > 1.0 {
                    Err("acos espera um número entre -1 e 1".to_string())
                } else {
                    Ok(Value::Number(n.acos()))
                }
            },
            _ => Err("acos espera um número".to_string()),
        }
    });

    globals.define_native_function("atan", |args, _interpreter| {
        if args.len() != 1 { return Err("atan espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.atan())),
            _ => Err("atan espera um número".to_string()),
        }
    });

    globals.define_native_function("atan2", |args, _interpreter| {
        if args.len() != 2 { return Err("atan2 espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(y), Value::Number(x)) => Ok(Value::Number(y.atan2(*x))),
            _ => Err("atan2 espera dois números".to_string()),
        }
    });

    // Novas funções úteis
    globals.define_native_function("mod", |args, _interpreter| {
        if args.len() != 2 { return Err("mod espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err("mod: divisão por zero".to_string())
                } else {
                    Ok(Value::Number(a % b))
                }
            },
            _ => Err("mod espera dois números".to_string()),
        }
    });

    globals.define_native_function("random", |args, _interpreter| {
        if !args.is_empty() { return Err("random não espera argumentos".to_string()); }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(Value::Number(rng.gen::<f64>()))
    });

    globals.define_native_function("random_range", |args, _interpreter| {
        if args.len() != 2 { return Err("random_range espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(min), Value::Number(max)) => {
                if min >= max {
                    return Err("random_range: min deve ser menor que max".to_string());
                }
                use rand::Rng;
                let mut rng = rand::thread_rng();
                Ok(Value::Number(rng.gen_range(*min..*max)))
            },
            _ => Err("random_range espera dois números".to_string()),
        }
    });

    globals.define_native_function("clamp", |args, _interpreter| {
        if args.len() != 3 { return Err("clamp espera 3 argumentos".to_string()); }
        match (&args[0], &args[1], &args[2]) {
            (Value::Number(value), Value::Number(min), Value::Number(max)) => {
                if min > max {
                    return Err("clamp: min deve ser menor ou igual a max".to_string());
                }
                Ok(Value::Number(value.clamp(*min, *max)))
            },
            _ => Err("clamp espera três números".to_string()),
        }
    });

    globals.define_native_function("sign", |args, _interpreter| {
        if args.len() != 1 { return Err("sign espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                let result = if *n > 0.0 { 1.0 } else if *n < 0.0 { -1.0 } else { 0.0 };
                Ok(Value::Number(result))
            },
            _ => Err("sign espera um número".to_string()),
        }
    });

    globals.define_native_function("deg_to_rad", |args, _interpreter| {
        if args.len() != 1 { return Err("deg_to_rad espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(deg) => Ok(Value::Number(deg.to_radians())),
            _ => Err("deg_to_rad espera um número".to_string()),
        }
    });

    globals.define_native_function("rad_to_deg", |args, _interpreter| {
        if args.len() != 1 { return Err("rad_to_deg espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(rad) => Ok(Value::Number(rad.to_degrees())),
            _ => Err("rad_to_deg espera um número".to_string()),
        }
    });

    // Constantes
    globals.define("PI".to_string(), Value::Number(std::f64::consts::PI), false, false);
    globals.define("E".to_string(), Value::Number(std::f64::consts::E), false, false);
    globals.define("TAU".to_string(), Value::Number(std::f64::consts::TAU), false, false);
}

/// Cria e retorna o módulo math como um objeto Value
/// Esta função é usada pelo novo sistema de gerenciamento de pacotes
/// Cria e retorna o módulo math como um objeto Value
/// Esta função é usada pelo novo sistema de gerenciamento de pacotes
pub fn create_module() -> Value {
    use std::collections::HashMap;
    
    let mut module = HashMap::new();
    
    // Funções básicas
    module.insert("abs".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("abs espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.abs())),
            _ => Err("abs espera um número".to_string()),
        }
    }));
    
    module.insert("floor".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("floor espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.floor())),
            _ => Err("floor espera um número".to_string()),
        }
    }));
    
    module.insert("ceil".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("ceil espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.ceil())),
            _ => Err("ceil espera um número".to_string()),
        }
    }));
    
    module.insert("round".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("round espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.round())),
            _ => Err("round espera um número".to_string()),
        }
    }));
    
    module.insert("pow".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("pow espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(base), Value::Number(exp)) => Ok(Value::Number(base.powf(*exp))),
            _ => Err("pow espera dois números".to_string()),
        }
    }));
    
    module.insert("sqrt".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("sqrt espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                if *n < 0.0 {
                    Err("sqrt não aceita números negativos".to_string())
                } else {
                    Ok(Value::Number(n.sqrt()))
                }
            },
            _ => Err("sqrt espera um número".to_string()),
        }
    }));
    
    module.insert("sin".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("sin espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.sin())),
            _ => Err("sin espera um número".to_string()),
        }
    }));
    
    module.insert("cos".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("cos espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.cos())),
            _ => Err("cos espera um número".to_string()),
        }
    }));
    
    module.insert("tan".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("tan espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => Ok(Value::Number(n.tan())),
            _ => Err("tan espera um número".to_string()),
        }
    }));
    
    // Novas funções úteis
    module.insert("mod".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("mod espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err("mod: divisão por zero".to_string())
                } else {
                    Ok(Value::Number(a % b))
                }
            },
            _ => Err("mod espera dois números".to_string()),
        }
    }));

    module.insert("random".to_string(), Value::NativeFunction(|args, _interpreter| {
        if !args.is_empty() { return Err("random não espera argumentos".to_string()); }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(Value::Number(rng.gen::<f64>()))
    }));

    module.insert("random_range".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("random_range espera 2 argumentos".to_string()); }
        match (&args[0], &args[1]) {
            (Value::Number(min), Value::Number(max)) => {
                if min >= max {
                    return Err("random_range: min deve ser menor que max".to_string());
                }
                use rand::Rng;
                let mut rng = rand::thread_rng();
                Ok(Value::Number(rng.gen_range(*min..*max)))
            },
            _ => Err("random_range espera dois números".to_string()),
        }
    }));

    module.insert("clamp".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 3 { return Err("clamp espera 3 argumentos".to_string()); }
        match (&args[0], &args[1], &args[2]) {
            (Value::Number(value), Value::Number(min), Value::Number(max)) => {
                if min > max {
                    return Err("clamp: min deve ser menor ou igual a max".to_string());
                }
                Ok(Value::Number(value.clamp(*min, *max)))
            },
            _ => Err("clamp espera três números".to_string()),
        }
    }));

    module.insert("sign".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("sign espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(n) => {
                let result = if *n > 0.0 { 1.0 } else if *n < 0.0 { -1.0 } else { 0.0 };
                Ok(Value::Number(result))
            },
            _ => Err("sign espera um número".to_string()),
        }
    }));

    module.insert("deg_to_rad".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("deg_to_rad espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(deg) => Ok(Value::Number(deg.to_radians())),
            _ => Err("deg_to_rad espera um número".to_string()),
        }
    }));

    module.insert("rad_to_deg".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("rad_to_deg espera 1 argumento".to_string()); }
        match &args[0] {
            Value::Number(rad) => Ok(Value::Number(rad.to_degrees())),
            _ => Err("rad_to_deg espera um número".to_string()),
        }
    }));
    
    // Constantes
    module.insert("PI".to_string(), Value::Number(std::f64::consts::PI));
    module.insert("E".to_string(), Value::Number(std::f64::consts::E));
    module.insert("TAU".to_string(), Value::Number(std::f64::consts::TAU));
    
    let dict_map = module.into_iter().map(|(k, v)| (Value::String(k), v)).collect();
    Value::Dict(dict_map)
}

