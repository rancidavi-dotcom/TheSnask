use crate::snif_parser::SnifValue;
use std::collections::BTreeMap;

fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(c0) = chars.next() else { return false };
    let is_start = c0.is_ascii_alphabetic() || c0 == '_' || c0 == '$';
    if !is_start {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '-')
}

fn escape_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn key_to_snif(k: &str) -> String {
    if is_ident(k) {
        k.to_string()
    } else {
        format!("\"{}\"", escape_string(k))
    }
}

fn num_to_snif(n: f64) -> String {
    if n.is_finite() && (n.fract() == 0.0) {
        // Render integers without ".0"
        format!("{}", n as i64)
    } else {
        // Stable enough for tooling: Rust's default is deterministic for finite f64.
        // We avoid scientific notation normalization to keep it minimal.
        let s = format!("{}", n);
        s
    }
}

fn is_scalar(v: &SnifValue) -> bool {
    matches!(
        v,
        SnifValue::Null | SnifValue::Bool(_) | SnifValue::Number(_) | SnifValue::String(_)
    ) || as_typed_literal(v).is_some()
}

fn as_typed_literal(v: &SnifValue) -> Option<(String, String)> {
    let SnifValue::Object(o) = v else { return None };
    if o.len() != 1 {
        // generic typed literal: {"$type": "...", "value": "..."} would be len=2
        // but we represent it as object in the parser. We canonicalize known single-key types.
        if o.len() == 2 {
            let t = match o.get("$type") {
                Some(SnifValue::String(s)) => s.clone(),
                _ => return None,
            };
            let p = match o.get("value") {
                Some(SnifValue::String(s)) => s.clone(),
                _ => return None,
            };
            return Some((t, p));
        }
        return None;
    }
    let (k, v) = o.iter().next().unwrap();
    let payload = match v {
        SnifValue::String(s) => s.clone(),
        _ => return None,
    };
    let t = match k.as_str() {
        "$date" => "date",
        "$dec" => "dec",
        "$bin" => "bin",
        "$enum" => "enum",
        _ => return None,
    };
    Some((t.to_string(), payload))
}

fn format_inline(v: &SnifValue) -> String {
    if let Some((t, p)) = as_typed_literal(v) {
        return format!("@{}\"{}\"", t, escape_string(&p));
    }
    match v {
        SnifValue::Null => "null".to_string(),
        SnifValue::Bool(true) => "true".to_string(),
        SnifValue::Bool(false) => "false".to_string(),
        SnifValue::Number(n) => num_to_snif(*n),
        SnifValue::String(s) => format!("\"{}\"", escape_string(s)),
        SnifValue::Array(a) => {
            let inner: Vec<String> = a.iter().map(format_inline).collect();
            format!("[{}]", inner.join(", "))
        }
        SnifValue::Object(o) => {
            let inner: Vec<String> = o
                .iter()
                .map(|(k, v)| format!("{}: {}", key_to_snif(k), format_inline(v)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

fn should_inline_array(a: &[SnifValue]) -> bool {
    if a.is_empty() {
        return true;
    }
    if a.len() > 6 {
        return false;
    }
    if !a.iter().all(is_scalar) {
        return false;
    }
    let s = format_inline(&SnifValue::Array(a.to_vec()));
    s.len() <= 60
}

fn should_inline_object(o: &BTreeMap<String, SnifValue>) -> bool {
    if o.is_empty() {
        return true;
    }
    if o.len() > 6 {
        return false;
    }
    if !o.values().all(is_scalar) {
        return false;
    }
    let s = format_inline(&SnifValue::Object(o.clone()));
    s.len() <= 60
}

fn fmt_value(v: &SnifValue, indent: usize, out: &mut String) {
    if let Some((t, p)) = as_typed_literal(v) {
        out.push_str(&format!("@{}\"{}\"", t, escape_string(&p)));
        return;
    }
    match v {
        SnifValue::Null | SnifValue::Bool(_) | SnifValue::Number(_) | SnifValue::String(_) => {
            out.push_str(&format_inline(v));
        }
        SnifValue::Array(a) => {
            if should_inline_array(a) {
                out.push_str(&format_inline(v));
                return;
            }
            out.push_str("[\n");
            let child_indent = indent + 4;
            for item in a {
                out.push_str(&" ".repeat(child_indent));
                fmt_value(item, child_indent, out);
                out.push_str(",\n");
            }
            out.push_str(&" ".repeat(indent));
            out.push(']');
        }
        SnifValue::Object(o) => {
            if should_inline_object(o) {
                out.push_str(&format_inline(v));
                return;
            }
            out.push_str("{\n");
            let child_indent = indent + 4;
            for (k, v) in o {
                out.push_str(&" ".repeat(child_indent));
                out.push_str(&key_to_snif(k));
                out.push_str(": ");
                fmt_value(v, child_indent, out);
                out.push_str(",\n");
            }
            out.push_str(&" ".repeat(indent));
            out.push('}');
        }
    }
}

/// Canonical SNIF formatting (stable output).
pub fn format_snif(v: &SnifValue) -> String {
    let mut out = String::new();
    fmt_value(v, 0, &mut out);
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snif_parser::parse_snif;

    #[test]
    fn fmt_is_idempotent() {
        let src = "{package:{name:\"x\",version:\"0.1.0\",entry:\"main.snask\"},dependencies:{json:\"0.1.0\"}}";
        let v = parse_snif(src).unwrap();
        let a = format_snif(&v);
        let v2 = parse_snif(&a).unwrap();
        let b = format_snif(&v2);
        assert_eq!(a, b);
    }

    #[test]
    fn typed_literals_canonicalize() {
        let v = parse_snif("{x:@date\"2026-01-01\"}").unwrap();
        let s = format_snif(&v);
        assert!(s.contains("@date\"2026-01-01\""));
    }
}

