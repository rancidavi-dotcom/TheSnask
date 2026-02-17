use crate::value::Value;
use crate::symbol_table::SymbolTable;
use std::collections::HashMap;

/// Registra funções de JSON na stdlib
pub fn register(globals: &mut SymbolTable) {
    globals.define_native_function("json_parse", |args, _interpreter| {
        if args.len() != 1 { return Err("json_parse espera 1 argumento".to_string()); }
        
        match &args[0] {
            Value::String(json_str) => {
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(json_value) => Ok(json_to_value(&json_value)),
                    Err(e) => Err(format!("Erro ao parsear JSON: {}", e)),
                }
            },
            _ => Err("json_parse espera uma string".to_string()),
        }
    });

    globals.define_native_function("json_stringify", |args| {
        if args.len() != 1 { return Err("json_stringify espera 1 argumento".to_string()); }
        
        let json_value = value_to_json(&args[0]);
        match serde_json::to_string(&json_value) {
            Ok(json_str) => Ok(Value::String(json_str)),
            Err(e) => Err(format!("Erro ao converter para JSON: {}", e)),
        }
    });

    globals.define_native_function("json_stringify_pretty", |args| {
        if args.len() != 1 { return Err("json_stringify_pretty espera 1 argumento".to_string()); }
        
        let json_value = value_to_json(&args[0]);
        match serde_json::to_string_pretty(&json_value) {
            Ok(json_str) => Ok(Value::String(json_str)),
            Err(e) => Err(format!("Erro ao converter para JSON: {}", e)),
        }
    });
}

/// Converte serde_json::Value para nosso Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Nil
            }
        },
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(json_to_value).collect();
            Value::List(values)
        },
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, value) in obj {
                map.insert(Value::String(key.clone()), json_to_value(value));
            }
            Value::Dict(map)
        },
    }
}

/// Converte nosso Value para serde_json::Value
fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Nil => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => {
            serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        },
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::List(list) => {
            let arr: Vec<serde_json::Value> = list.iter().map(value_to_json).collect();
            serde_json::Value::Array(arr)
        },
        Value::Dict(dict) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in dict {
                if let Value::String(key_str) = key {
                    obj.insert(key_str.clone(), value_to_json(val));
                }
            }
            serde_json::Value::Object(obj)
        },
        _ => serde_json::Value::Null,
    }
}
