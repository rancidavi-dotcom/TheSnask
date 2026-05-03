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

fn write_escaped_string(out: &mut String, s: &str) {
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
}

fn write_key(out: &mut String, k: &str) {
    if is_ident(k) {
        out.push_str(k);
    } else {
        out.push('"');
        write_escaped_string(out, k);
        out.push('"');
    }
}

fn write_num(out: &mut String, n: f64) {
    if n.is_finite() && (n.fract() == 0.0) {
        // Render integers without ".0"
        out.push_str(&(n as i64).to_string());
    } else {
        out.push_str(&format!("{}", n));
    }
}

fn as_typed_literal(v: &SnifValue) -> Option<(&'static str, &str)> {
    let SnifValue::Object(o) = v else { return None };
    if o.len() == 1 {
        let (k, v) = o.iter().next().unwrap();
        let SnifValue::String(payload) = v else {
            return None;
        };
        let t = match k.as_str() {
            "$date" => "date",
            "$dec" => "dec",
            "$bin" => "bin",
            "$enum" => "enum",
            _ => return None,
        };
        return Some((t, payload));
    }
    None
}

fn is_scalar(v: &SnifValue) -> bool {
    matches!(
        v,
        SnifValue::Null | SnifValue::Bool(_) | SnifValue::Number(_) | SnifValue::String(_)
    ) || matches!(as_typed_literal(v), Some(_))
}

fn inline_len(v: &SnifValue) -> usize {
    if let Some((t, p)) = as_typed_literal(v) {
        return 1 + t.len() + 2 + escaped_len(p) + 1;
    }
    match v {
        SnifValue::Null => 4,
        SnifValue::Bool(true) => 4,
        SnifValue::Bool(false) => 5,
        SnifValue::Number(n) => {
            if n.is_finite() && (n.fract() == 0.0) {
                // rough, but ok for inline decisions
                (n.abs() as i64).to_string().len() + if *n < 0.0 { 1 } else { 0 }
            } else {
                format!("{}", n).len()
            }
        }
        SnifValue::String(s) => 2 + escaped_len(s),
        SnifValue::Array(a) => inline_array_len(a),
        SnifValue::Object(o) => inline_object_len(o),
    }
}

fn inline_array_len(a: &[SnifValue]) -> usize {
    if a.is_empty() {
        return 2;
    }
    let mut len = 1; // [
    for (idx, item) in a.iter().enumerate() {
        if idx > 0 {
            len += 2; // ", "
        }
        len += inline_len(item);
    }
    len + 1 // ]
}

fn inline_object_len(o: &BTreeMap<String, SnifValue>) -> usize {
    if o.is_empty() {
        return 2;
    }
    let mut len = 1; // {
    let mut first = true;
    for (k, v) in o.iter() {
        if !first {
            len += 2; // ", "
        }
        first = false;
        len += key_len(k);
        len += 2; // ": "
        len += inline_len(v);
    }
    len + 1 // }
}

fn escaped_len(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.chars() {
        n += match c {
            '\\' | '"' => 2,
            '\n' | '\r' | '\t' => 2,
            _ => c.len_utf8(),
        };
    }
    n
}

fn key_len(k: &str) -> usize {
    if is_ident(k) {
        k.len()
    } else {
        2 + escaped_len(k)
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
    inline_array_len(a) <= 60
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
    inline_object_len(o) <= 60
}

fn fmt_value(out: &mut String, v: &SnifValue, indent: usize) {
    if let Some((t, p)) = as_typed_literal(v) {
        out.push('@');
        out.push_str(t);
        out.push('"');
        write_escaped_string(out, p);
        out.push('"');
        return;
    }

    match v {
        SnifValue::Null => out.push_str("null"),
        SnifValue::Bool(true) => out.push_str("true"),
        SnifValue::Bool(false) => out.push_str("false"),
        SnifValue::Number(n) => write_num(out, *n),
        SnifValue::String(s) => {
            out.push('"');
            write_escaped_string(out, s);
            out.push('"');
        }
        SnifValue::Array(a) => {
            if should_inline_array(a) {
                out.push('[');
                for (idx, item) in a.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    fmt_value(out, item, indent);
                }
                out.push(']');
                return;
            }
            out.push_str("[\n");
            let child = indent + 4;
            for item in a.iter() {
                out.push_str(&" ".repeat(child));
                fmt_value(out, item, child);
                out.push_str(",\n");
            }
            out.push_str(&" ".repeat(indent));
            out.push(']');
        }
        SnifValue::Object(o) => {
            if should_inline_object(o) {
                out.push('{');
                let mut first = true;
                for (k, v) in o.iter() {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    write_key(out, k);
                    out.push_str(": ");
                    fmt_value(out, v, indent);
                }
                out.push('}');
                return;
            }
            out.push_str("{\n");
            let child = indent + 4;
            for (k, v) in o.iter() {
                out.push_str(&" ".repeat(child));
                write_key(out, k);
                out.push_str(": ");
                fmt_value(out, v, child);
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
    fmt_value(&mut out, v, 0);
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
