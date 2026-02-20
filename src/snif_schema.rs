use crate::snif_parser::SnifValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct SnifSchemaError {
    pub path: String,
    pub message: String,
}

impl SnifSchemaError {
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        SnifSchemaError {
            path: path.into(),
            message: message.into(),
        }
    }
}

fn as_obj<'a>(v: &'a SnifValue) -> Option<&'a BTreeMap<String, SnifValue>> {
    match v {
        SnifValue::Object(o) => Some(o),
        _ => None,
    }
}

fn get_str(o: &BTreeMap<String, SnifValue>, key: &str) -> Option<String> {
    match o.get(key) {
        Some(SnifValue::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn get_num(o: &BTreeMap<String, SnifValue>, key: &str) -> Option<f64> {
    match o.get(key) {
        Some(SnifValue::Number(n)) => Some(*n),
        _ => None,
    }
}

fn get_bool(o: &BTreeMap<String, SnifValue>, key: &str) -> Option<bool> {
    match o.get(key) {
        Some(SnifValue::Bool(b)) => Some(*b),
        _ => None,
    }
}

fn valid_pkg_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn valid_semverish(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

pub fn validate_snask_manifest(v: &SnifValue) -> Vec<SnifSchemaError> {
    let mut errs: Vec<SnifSchemaError> = Vec::new();

    let Some(root) = as_obj(v) else {
        errs.push(SnifSchemaError::new("$", "Top-level SNIF must be an object."));
        return errs;
    };

    // package
    let pkg = match root.get("package") {
        Some(v) => v,
        None => {
            errs.push(SnifSchemaError::new("$.package", "Missing required key 'package'."));
            return errs;
        }
    };
    let Some(pkg_o) = as_obj(pkg) else {
        errs.push(SnifSchemaError::new("$.package", "'package' must be an object."));
        return errs;
    };

    // name
    match get_str(pkg_o, "name") {
        Some(name) => {
            if !valid_pkg_name(&name) {
                errs.push(SnifSchemaError::new(
                    "$.package.name",
                    "Invalid package name. Allowed: [a-zA-Z0-9_-]+",
                ));
            }
        }
        None => errs.push(SnifSchemaError::new(
            "$.package.name",
            "Missing required string key 'package.name'.",
        )),
    }

    // version
    match get_str(pkg_o, "version") {
        Some(ver) => {
            if !valid_semverish(&ver) {
                errs.push(SnifSchemaError::new(
                    "$.package.version",
                    "Invalid version. Expected semver-ish 'x.y.z'.",
                ));
            }
        }
        None => errs.push(SnifSchemaError::new(
            "$.package.version",
            "Missing required string key 'package.version'.",
        )),
    }

    // entry
    match get_str(pkg_o, "entry") {
        Some(entry) => {
            if !entry.ends_with(".snask") {
                errs.push(SnifSchemaError::new(
                    "$.package.entry",
                    "Invalid entry. Expected a .snask file (e.g. 'main.snask').",
                ));
            }
        }
        None => errs.push(SnifSchemaError::new(
            "$.package.entry",
            "Missing required string key 'package.entry'.",
        )),
    }

    // dependencies
    if let Some(deps_v) = root.get("dependencies") {
        if let Some(deps) = as_obj(deps_v) {
            for (k, v) in deps {
                match v {
                    SnifValue::String(_) | SnifValue::Null => {}
                    _ => errs.push(SnifSchemaError::new(
                        format!("$.dependencies.{k}"),
                        "Dependency value must be a string (version) or null (latest).",
                    )),
                }
            }
        } else {
            errs.push(SnifSchemaError::new(
                "$.dependencies",
                "'dependencies' must be an object.",
            ));
        }
    }

    // build
    if let Some(build_v) = root.get("build") {
        if let Some(build) = as_obj(build_v) {
            if let Some(n) = get_num(build, "opt_level") {
                if !(0.0..=3.0).contains(&n) {
                    errs.push(SnifSchemaError::new(
                        "$.build.opt_level",
                        "opt_level must be in range 0..3.",
                    ));
                }
            }
            if build.contains_key("debug") && get_bool(build, "debug").is_none() {
                errs.push(SnifSchemaError::new("$.build.debug", "debug must be a boolean."));
            }
        } else {
            errs.push(SnifSchemaError::new("$.build", "'build' must be an object."));
        }
    }

    // scripts
    if let Some(scripts_v) = root.get("scripts") {
        if let Some(scripts) = as_obj(scripts_v) {
            for (k, v) in scripts {
                if !matches!(v, SnifValue::String(_)) {
                    errs.push(SnifSchemaError::new(
                        format!("$.scripts.{k}"),
                        "Script value must be a string.",
                    ));
                }
            }
        } else {
            errs.push(SnifSchemaError::new("$.scripts", "'scripts' must be an object."));
        }
    }

    errs
}

pub fn snask_manifest_schema_md() -> String {
    let md = r#"# snask.snif schema (v1)

## Top-level
- `package` *(object, required)*
- `dependencies` *(object, optional)*
- `build` *(object, optional)*
- `scripts` *(object, optional)*

## package
- `name` *(string, required)*: `[a-zA-Z0-9_-]+`
- `version` *(string, required)*: semver-ish `x.y.z`
- `entry` *(string, required)*: ends with `.snask`

## dependencies
Map: `name -> version|string | null`
- string: version constraint (e.g. `"0.3.0"`)
- null: latest (`*`)

## build
- `opt_level` *(number, optional)*: 0..3
- `debug` *(bool, optional)*

## scripts
Map: `name -> string`

"#;
    md.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snif_parser::parse_snif;

    #[test]
    fn schema_catches_missing_package_name() {
        let v = parse_snif("{package:{version:\"0.1.0\",entry:\"main.snask\"}}").unwrap();
        let errs = validate_snask_manifest(&v);
        assert!(errs.iter().any(|e| e.path == "$.package.name"));
    }

    #[test]
    fn schema_opt_level_range() {
        let v = parse_snif("{package:{name:\"x\",version:\"0.1.0\",entry:\"main.snask\"},build:{opt_level:9}}").unwrap();
        let errs = validate_snask_manifest(&v);
        assert!(errs.iter().any(|e| e.path == "$.build.opt_level"));
    }
}

