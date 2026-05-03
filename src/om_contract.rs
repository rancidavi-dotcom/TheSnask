use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmContract {
    pub library: String,
    pub resources: Vec<OmResourceContract>,
    pub functions: Vec<OmFunctionContract>,
    pub constants: Vec<OmConstantContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmConstantContract {
    pub name: String,
    pub surface: String,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmResourceContract {
    pub name: String,
    pub c_type: String,
    pub constructor: String,
    pub destructor: String,
    pub surface_type: String,
    pub depends_on: Option<String>,
    pub safety: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmFunctionContract {
    pub name: String,
    pub c_function: String,
    pub surface: String,
    pub input: String,
    pub output: String,
    pub c_return_type: Option<String>,
    pub c_param_types: Vec<String>,
    pub safety: Option<String>,
    pub reason: Option<String>,
}

impl OmContract {
    pub fn resource_by_surface_type(&self, surface_type: &str) -> Option<&OmResourceContract> {
        self.resources
            .iter()
            .find(|r| r.surface_type == surface_type)
    }

    pub fn function_by_surface(&self, surface: &str) -> Option<&OmFunctionContract> {
        self.functions.iter().find(|f| f.surface == surface)
    }

    pub fn constant_by_surface(&self, surface: &str) -> Option<&OmConstantContract> {
        self.constants.iter().find(|c| c.surface == surface)
    }
}

#[derive(Debug, Default)]
struct ResourceBuilder {
    name: String,
    c_type: Option<String>,
    constructor: Option<String>,
    destructor: Option<String>,
    surface_type: Option<String>,
    depends_on: Option<String>,
    safety: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Default)]
struct FunctionBuilder {
    name: String,
    c_function: Option<String>,
    surface: Option<String>,
    input: Option<String>,
    output: Option<String>,
    c_return_type: Option<String>,
    c_param_types: Vec<String>,
    safety: Option<String>,
    reason: Option<String>,
}

impl FunctionBuilder {
    fn finish(self, line: usize) -> Result<OmFunctionContract, String> {
        Ok(OmFunctionContract {
            name: self.name,
            c_function: self.c_function.ok_or_else(|| {
                format!("OM contract: function missing `c_function` before line {line}")
            })?,
            surface: self.surface.ok_or_else(|| {
                format!("OM contract: function missing `surface` before line {line}")
            })?,
            input: self.input.ok_or_else(|| {
                format!("OM contract: function missing `input` before line {line}")
            })?,
            output: self.output.ok_or_else(|| {
                format!("OM contract: function missing `output` before line {line}")
            })?,
            c_return_type: self.c_return_type,
            c_param_types: self.c_param_types,
            safety: self.safety,
            reason: self.reason,
        })
    }
}

enum SectionBuilder {
    Resource(ResourceBuilder),
    Function(FunctionBuilder),
}

impl SectionBuilder {
    fn finish(
        self,
        line: usize,
        resources: &mut Vec<OmResourceContract>,
        functions: &mut Vec<OmFunctionContract>,
    ) -> Result<(), String> {
        match self {
            SectionBuilder::Resource(builder) => resources.push(builder.finish(line)?),
            SectionBuilder::Function(builder) => functions.push(builder.finish(line)?),
        }
        Ok(())
    }
}

impl ResourceBuilder {
    fn finish(self, line: usize) -> Result<OmResourceContract, String> {
        Ok(OmResourceContract {
            name: self.name,
            c_type: self.c_type.ok_or_else(|| {
                format!("OM contract: resource missing `c_type` before line {line}")
            })?,
            constructor: self.constructor.ok_or_else(|| {
                format!("OM contract: resource missing `constructor` before line {line}")
            })?,
            destructor: self.destructor.ok_or_else(|| {
                format!("OM contract: resource missing `destructor` before line {line}")
            })?,
            surface_type: self.surface_type.ok_or_else(|| {
                format!("OM contract: resource missing `surface_type` before line {line}")
            })?,
            depends_on: self.depends_on,
            safety: self.safety,
            reason: self.reason,
        })
    }
}

pub fn load_om_contract(path: &Path) -> Result<OmContract, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("OM contract: failed to read {}: {e}", path.display()))?;
    parse_om_contract(&src)
}

pub fn load_builtin_om_contract(name: &str) -> Result<OmContract, String> {
    match name {
        "sqlite" => parse_om_contract(include_str!("../contracts/sqlite.om.snif")),
        "zlib" => parse_om_contract(include_str!("../contracts/zlib.om.snif")),
        _ => Err(format!(
            "OM contract: no built-in contract for library `{name}`"
        )),
    }
}

pub fn parse_om_contract(src: &str) -> Result<OmContract, String> {
    let mut library = None;
    let mut resources = Vec::new();
    let mut functions = Vec::new();
    let mut constants = Vec::new();
    let mut current: Option<SectionBuilder> = None;

    for (idx, raw_line) in src.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("library ") {
            if library.is_some() {
                return Err(format!(
                    "OM contract: duplicate library declaration at line {line_no}"
                ));
            }
            let name = rest.trim();
            if name.is_empty() {
                return Err(format!("OM contract: empty library name at line {line_no}"));
            }
            library = Some(name.to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("resource ") {
            if let Some(builder) = current.take() {
                builder.finish(line_no, &mut resources, &mut functions)?;
            }
            let name = rest
                .strip_suffix(':')
                .ok_or_else(|| {
                    format!("OM contract: expected ':' after resource name at line {line_no}")
                })?
                .trim();
            if name.is_empty() {
                return Err(format!(
                    "OM contract: empty resource name at line {line_no}"
                ));
            }
            current = Some(SectionBuilder::Resource(ResourceBuilder {
                name: name.to_string(),
                ..ResourceBuilder::default()
            }));
            continue;
        }

        if let Some(rest) = line.strip_prefix("function ") {
            if let Some(builder) = current.take() {
                builder.finish(line_no, &mut resources, &mut functions)?;
            }
            let name = rest
                .strip_suffix(':')
                .ok_or_else(|| {
                    format!("OM contract: expected ':' after function name at line {line_no}")
                })?
                .trim();
            if name.is_empty() {
                return Err(format!(
                    "OM contract: empty function name at line {line_no}"
                ));
            }
            current = Some(SectionBuilder::Function(FunctionBuilder {
                name: name.to_string(),
                ..FunctionBuilder::default()
            }));
            continue;
        }

        if let Some(rest) = line.strip_prefix("constant ") {
            if let Some(builder) = current.take() {
                builder.finish(line_no, &mut resources, &mut functions)?;
            }
            let (name, raw_value) = rest.split_once(':').ok_or_else(|| {
                format!("OM contract: expected `constant NAME: value` at line {line_no}")
            })?;
            let name = name.trim();
            if name.is_empty() {
                return Err(format!(
                    "OM contract: empty constant name at line {line_no}"
                ));
            }
            let raw_value = raw_value.trim();
            let value = parse_integer_literal(raw_value).ok_or_else(|| {
                format!("OM contract: invalid constant value `{raw_value}` at line {line_no}")
            })?;
            let library_name = library
                .as_ref()
                .ok_or_else(|| format!("OM contract: constant before library at line {line_no}"))?;
            constants.push(OmConstantContract {
                name: name.to_string(),
                surface: format!("{library_name}.{name}"),
                value,
            });
            continue;
        }

        let Some(section) = current.as_mut() else {
            return Err(format!(
                "OM contract: field outside section at line {line_no}"
            ));
        };
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("OM contract: expected `key: value` at line {line_no}"))?;
        let value = value.trim();
        if value.is_empty() {
            return Err(format!(
                "OM contract: empty value for `{}` at line {line_no}",
                key.trim()
            ));
        }

        match section {
            SectionBuilder::Resource(builder) => match key.trim() {
                "c_type" => builder.c_type = Some(value.to_string()),
                "constructor" => builder.constructor = Some(value.to_string()),
                "destructor" => builder.destructor = Some(value.to_string()),
                "surface_type" => builder.surface_type = Some(value.to_string()),
                "depends_on" => builder.depends_on = Some(value.to_string()),
                "safety" => builder.safety = Some(value.to_string()),
                "reason" => builder.reason = Some(value.to_string()),
                other => {
                    return Err(format!(
                        "OM contract: unknown resource field `{other}` at line {line_no}"
                    ))
                }
            },
            SectionBuilder::Function(builder) => match key.trim() {
                "c_function" => builder.c_function = Some(value.to_string()),
                "surface" => builder.surface = Some(value.to_string()),
                "input" => builder.input = Some(value.to_string()),
                "output" => builder.output = Some(value.to_string()),
                "c_return_type" => builder.c_return_type = Some(value.to_string()),
                "c_param_types" => {
                    builder.c_param_types = value
                        .split(',')
                        .map(str::trim)
                        .filter(|ty| !ty.is_empty())
                        .map(ToString::to_string)
                        .collect()
                }
                "safety" => builder.safety = Some(value.to_string()),
                "reason" => builder.reason = Some(value.to_string()),
                other => {
                    return Err(format!(
                        "OM contract: unknown function field `{other}` at line {line_no}"
                    ))
                }
            },
        }
    }

    if let Some(builder) = current.take() {
        builder.finish(src.lines().count() + 1, &mut resources, &mut functions)?;
    }

    let library =
        library.ok_or_else(|| "OM contract: missing `library` declaration".to_string())?;
    if resources.is_empty() && functions.is_empty() {
        return Err("OM contract: expected at least one resource or function".to_string());
    }

    Ok(OmContract {
        library,
        resources,
        functions,
        constants,
    })
}

fn parse_integer_literal(raw: &str) -> Option<i64> {
    let cleaned = raw
        .trim()
        .trim_end_matches('u')
        .trim_end_matches('U')
        .trim_end_matches('l')
        .trim_end_matches('L');
    if let Some(hex) = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
    {
        i64::from_str_radix(hex, 16).ok()
    } else {
        cleaned.parse::<i64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{load_builtin_om_contract, parse_om_contract};

    #[test]
    fn loads_sqlite_builtin_contract() {
        let contract = load_builtin_om_contract("sqlite").expect("sqlite contract should load");
        assert_eq!(contract.library, "sqlite");

        let db = contract
            .resource_by_surface_type("sqlite.Database")
            .expect("Database resource should exist");
        assert_eq!(db.name, "Database");
        assert_eq!(db.c_type, "sqlite3*");
        assert_eq!(db.constructor, "sqlite3_open");
        assert_eq!(db.destructor, "sqlite3_close");
        assert_eq!(db.depends_on, None);

        let stmt = contract
            .resource_by_surface_type("sqlite.Statement")
            .expect("Statement resource should exist");
        assert_eq!(stmt.c_type, "sqlite3_stmt*");
        assert_eq!(stmt.constructor, "sqlite3_prepare_v2");
        assert_eq!(stmt.destructor, "sqlite3_finalize");
        assert_eq!(stmt.depends_on.as_deref(), Some("Database"));
    }

    #[test]
    fn loads_zlib_builtin_contract() {
        let contract = load_builtin_om_contract("zlib").expect("zlib contract should load");
        assert_eq!(contract.library, "zlib");

        let compress = contract
            .function_by_surface("zlib.compress")
            .expect("compress function should exist");
        assert_eq!(compress.name, "compress");
        assert_eq!(compress.c_function, "compress2");
        assert_eq!(compress.input, "str");
        assert_eq!(compress.output, "bytes");

        let decompress = contract
            .function_by_surface("zlib.decompress")
            .expect("decompress function should exist");
        assert_eq!(decompress.c_function, "uncompress");
        assert_eq!(decompress.input, "bytes");
        assert_eq!(decompress.output, "str");
    }

    #[test]
    fn rejects_resource_without_destructor() {
        let err = parse_om_contract(
            r#"
library bad

resource Thing:
    c_type: thing*
    constructor: thing_open
    surface_type: bad.Thing
"#,
        )
        .expect_err("missing destructor must fail");
        assert!(err.contains("destructor"), "{err}");
    }
}
