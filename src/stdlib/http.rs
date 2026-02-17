use crate::value::Value;
use std::collections::HashMap;

/// Cria e retorna o objeto do módulo `http` com todas as suas funções.
pub fn create_module() -> Value {
    let mut module = HashMap::new();

    module.insert("get".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 1 { return Err("http.get espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(url) => {
                #[cfg(feature = "http")]
                {
                    match reqwest::blocking::get(url) {
                        Ok(response) => {
                            let mut result = HashMap::new();
                            
                            // Status code
                            result.insert(
                                Value::String("status".to_string()),
                                Value::Number(response.status().as_u16() as f64)
                            );

                            // Body
                            match response.text() {
                                Ok(body) => {
                                    result.insert(
                                        Value::String("body".to_string()),
                                        Value::String(body)
                                    );
                                },
                                Err(e) => {
                                    return Err(format!("Erro ao ler resposta: {}", e));
                                }
                            }

                            Ok(Value::Dict(result))
                        },
                        Err(e) => Err(format!("Erro na requisição HTTP: {}", e)),
                    }
                }
                
                #[cfg(not(feature = "http"))]
                {
                    Err("HTTP não está habilitado nesta build".to_string())
                }
            },
            _ => Err("http.get espera uma string (URL)".to_string()),
        }
    }));

    module.insert("post".to_string(), Value::NativeFunction(|args, _interpreter| {
        if args.len() != 2 { return Err("http.post espera 2 argumentos".to_string()); }
        
        match (&args[0], &args[1]) {
            (Value::String(url), Value::String(body)) => {
                #[cfg(feature = "http")]
                {
                    let client = reqwest::blocking::Client::new();
                    match client.post(url).body(body.clone()).send() {
                        Ok(response) => {
                            let mut result = HashMap::new();
                            
                            result.insert(
                                Value::String("status".to_string()),
                                Value::Number(response.status().as_u16() as f64)
                            );

                            match response.text() {
                                Ok(response_body) => {
                                    result.insert(
                                        Value::String("body".to_string()),
                                        Value::String(response_body)
                                    );
                                },
                                Err(e) => {
                                    return Err(format!("Erro ao ler resposta: {}", e));
                                }
                            }

                            Ok(Value::Dict(result))
                        },
                        Err(e) => Err(format!("Erro na requisição HTTP: {}", e)),
                    }
                }
                
                #[cfg(not(feature = "http"))]
                {
                    Err("HTTP não está habilitado nesta build".to_string())
                }
            },
            _ => Err("http.post espera duas strings (URL e body)".to_string()),
        }
    }));
    
    // Converte o HashMap para o formato esperado por Value::Dict
    let dict_map = module.into_iter().map(|(k, v)| (Value::String(k), v)).collect();
    Value::Dict(dict_map)
}
