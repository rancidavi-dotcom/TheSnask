use crate::om_contract::{OmConstantContract, OmContract, OmFunctionContract, OmResourceContract};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub header: String,
    pub lib: String,
    pub output: Option<String>,
    pub extra_cflags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Safety {
    Safe,
    CopyOnly,
    Blocked,
}

impl Safety {
    pub fn as_str(self) -> &'static str {
        match self {
            Safety::Safe => "SAFE",
            Safety::CopyOnly => "COPY_ONLY",
            Safety::Blocked => "BLOCKED",
        }
    }
}

#[derive(Debug, Clone)]
struct CParam {
    name: String,
    ty: String,
}

#[derive(Debug, Clone)]
struct CFunction {
    name: String,
    signature: String,
    return_ty: String,
    params: Vec<CParam>,
}

#[derive(Debug, Clone)]
struct CConstant {
    name: String,
    value: i64,
}

#[derive(Debug)]
pub struct GeneratedResource {
    pub name: String,
    pub c_type: String,
    pub constructor: String,
    pub destructor: String,
    pub surface_type: String,
    pub safety: Safety,
    pub reason: String,
}

#[derive(Debug)]
pub struct GeneratedFunction {
    pub name: String,
    pub c_function: String,
    pub surface: String,
    pub input: String,
    pub output: String,
    pub c_return_type: String,
    pub c_param_types: Vec<String>,
    pub safety: Safety,
    pub reason: String,
}

#[derive(Debug, Default)]
struct ScanModel {
    functions: Vec<CFunction>,
    constants: Vec<CConstant>,
    typedefs: BTreeSet<String>,
    records: BTreeSet<String>,
    enums: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct AstNode {
    kind: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    ty: Option<AstType>,
    value: Option<serde_json::Value>,
    range: Option<AstRange>,
    inner: Option<Vec<AstNode>>,
}

#[derive(Debug, Deserialize)]
struct AstType {
    #[serde(rename = "qualType")]
    qual_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AstRange {
    begin: Option<AstLoc>,
}

#[derive(Debug, Deserialize)]
struct AstLoc {
    file: Option<String>,
    #[serde(rename = "includedFrom")]
    included_from: Option<AstIncludedFrom>,
    #[serde(rename = "expansionLoc")]
    expansion_loc: Option<AstExpansionLoc>,
}

#[derive(Debug, Deserialize)]
struct AstExpansionLoc {
    file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AstIncludedFrom {
    file: Option<String>,
}

pub fn scan_header(options: ScanOptions) -> Result<OmContract, String> {
    let (clang_header, header_path) = resolve_header_path(&options.header, &options.extra_cflags)?;
    let ast = clang_ast_json(&clang_header, &options.extra_cflags)?;
    let macros = clang_macro_defines(&clang_header, &options.extra_cflags).unwrap_or_default();
    let root: AstNode = serde_json::from_slice(&ast)
        .map_err(|e| format!("OM scan: failed to parse clang AST JSON: {e}"))?;

    let mut model = ScanModel::default();
    collect_ast(&root, &mut model, header_path.as_deref(), &options.lib);
    collect_macro_constants(&mut model, macros, &options.lib);
    dedupe_functions(&mut model.functions);
    dedupe_constants(&mut model.constants);

    let (mut resources, mut functions) = classify(&options.lib, &model.functions);
    dedupe_generated_resources(&mut resources);
    dedupe_generated_functions(&mut functions);

    Ok(convert_to_om_contract(
        &options.lib,
        resources,
        functions,
        model.constants,
    ))
}

fn convert_to_om_contract(
    lib: &str,
    resources: Vec<GeneratedResource>,
    functions: Vec<GeneratedFunction>,
    constants: Vec<CConstant>,
) -> OmContract {
    let mut contract = OmContract {
        library: lib.to_string(),
        resources: Vec::new(),
        functions: Vec::new(),
        constants: constants
            .into_iter()
            .map(|constant| {
                let name = constant_surface_name_for_lib(lib, &constant.name);
                OmConstantContract {
                    name: name.clone(),
                    surface: format!("{lib}.{name}"),
                    value: constant.value,
                }
            })
            .collect(),
    };

    for r in resources {
        if r.safety == Safety::Safe {
            contract.resources.push(OmResourceContract {
                name: r.name,
                c_type: r.c_type,
                constructor: r.constructor,
                destructor: r.destructor,
                surface_type: r.surface_type,
                depends_on: None,
                safety: Some(r.safety.as_str().to_string()),
                reason: Some(r.reason),
            });
        }
    }

    for f in functions {
        if f.safety != Safety::Blocked {
            contract.functions.push(OmFunctionContract {
                name: f.name,
                c_function: f.c_function,
                surface: f.surface,
                input: f.input,
                output: f.output,
                c_return_type: Some(f.c_return_type),
                c_param_types: f.c_param_types,
                safety: Some(f.safety.as_str().to_string()),
                reason: Some(f.reason),
            });
        }
    }

    contract
}

pub fn run_scan(options: ScanOptions) -> Result<(), String> {
    let (clang_header, header_path) = resolve_header_path(&options.header, &options.extra_cflags)?;
    let ast = clang_ast_json(&clang_header, &options.extra_cflags)?;
    let macros = clang_macro_defines(&clang_header, &options.extra_cflags).unwrap_or_default();
    let root: AstNode = serde_json::from_slice(&ast)
        .map_err(|e| format!("OM scan: failed to parse clang AST JSON: {e}"))?;

    let mut model = ScanModel::default();
    collect_ast(&root, &mut model, header_path.as_deref(), &options.lib);
    collect_macro_constants(&mut model, macros, &options.lib);
    dedupe_functions(&mut model.functions);
    dedupe_constants(&mut model.constants);

    let (mut resources, mut functions) = classify(&options.lib, &model.functions);
    dedupe_generated_resources(&mut resources);
    dedupe_generated_functions(&mut functions);

    let contract_text = render_contract(&options.lib, &resources, &functions, &model.constants);
    let report = render_report(&options, &model, &resources, &functions);

    let out_path = options
        .output
        .clone()
        .unwrap_or_else(|| format!("{}.generated.om.snif", options.lib));
    fs::write(&out_path, contract_text)
        .map_err(|e| format!("OM scan: failed to write {out_path}: {e}"))?;

    let report_path = report_path_for(&out_path);
    fs::write(&report_path, &report)
        .map_err(|e| format!("OM scan: failed to write {report_path}: {e}"))?;

    println!("{report}");
    println!("Generated contract: {out_path}");
    println!("Generated report:   {report_path}");
    Ok(())
}

fn clang_ast_json(header: &str, extra_cflags: &[String]) -> Result<Vec<u8>, String> {
    let mut args = vec![
        "-x".to_string(),
        "c-header".to_string(),
        "-Xclang".to_string(),
        "-ast-dump=json".to_string(),
        "-fsyntax-only".to_string(),
    ];

    for flag in extra_cflags {
        args.push(flag.clone());
    }

    args.push(header.to_string());

    let output = Command::new("clang-18")
        .args(&args)
        .output()
        .map_err(|e| format!("OM scan: failed to run clang-18: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "OM scan: clang failed for `{}`:\n{}",
            header,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output.stdout)
}

fn clang_macro_defines(header: &str, extra_cflags: &[String]) -> Result<Vec<String>, String> {
    let mut args = vec![
        "-dM".to_string(),
        "-E".to_string(),
        "-x".to_string(),
        "c-header".to_string(),
    ];

    for flag in extra_cflags {
        args.push(flag.clone());
    }

    args.push(header.to_string());

    let output = Command::new("clang-18")
        .args(&args)
        .output()
        .map_err(|e| format!("OM scan: failed to run clang-18 for macros: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "OM scan: clang macro scan failed for `{}`:\n{}",
            header,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(ToString::to_string)
        .collect())
}

fn resolve_header_path(
    header: &str,
    extra_cflags: &[String],
) -> Result<(String, Option<String>), String> {
    let mut candidates = vec![
        PathBuf::from(header),
        PathBuf::from("/usr/include").join(header),
        PathBuf::from("/usr/local/include").join(header),
    ];

    for flag in extra_cflags {
        if let Some(dir) = flag.strip_prefix("-I") {
            if !dir.is_empty() {
                candidates.push(PathBuf::from(dir).join(header));
            }
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            let path = fs::canonicalize(&candidate).map_err(|e| {
                format!(
                    "OM scan: failed to resolve header `{}`: {e}",
                    candidate.display()
                )
            })?;
            let path = path.to_string_lossy().to_string();
            return Ok((path.clone(), Some(path)));
        }
    }

    Ok((header.to_string(), None))
}

fn collect_ast(node: &AstNode, model: &mut ScanModel, header_path: Option<&str>, lib: &str) {
    match node.kind.as_deref() {
        Some("FunctionDecl") => {
            if node_is_from_header(node, header_path) || node_name_matches_library(node, lib) {
                if let Some(function) = function_from_node(node) {
                    model.functions.push(function);
                }
            }
        }
        Some("EnumConstantDecl") => {
            if node_is_from_header(node, header_path) || node_name_matches_library(node, lib) {
                if let Some(constant) = constant_from_node(node, lib) {
                    model.constants.push(constant);
                }
            }
        }
        Some("TypedefDecl") => {
            if node_is_from_header(node, header_path) {
                if let Some(name) = node.name.as_ref() {
                    model.typedefs.insert(name.clone());
                }
            }
        }
        Some("RecordDecl") => {
            if node_is_from_header(node, header_path) {
                if let Some(name) = node.name.as_ref() {
                    if !name.is_empty() {
                        model.records.insert(name.clone());
                    }
                }
            }
        }
        Some("EnumDecl") => {
            if node_is_from_header(node, header_path) {
                if let Some(name) = node.name.as_ref() {
                    if !name.is_empty() {
                        model.enums.insert(name.clone());
                    }
                }
            }
        }
        _ => {}
    }

    if let Some(inner) = node.inner.as_ref() {
        for child in inner {
            collect_ast(child, model, header_path, lib);
        }
    }
}

fn constant_from_node(node: &AstNode, lib: &str) -> Option<CConstant> {
    let name = node.name.clone()?;
    if !constant_name_matches_library(&name, lib) {
        return None;
    }
    let value = node
        .inner
        .as_ref()
        .and_then(|inner| {
            inner
                .iter()
                .find(|child| child.kind.as_deref() == Some("ConstantExpr"))
        })
        .and_then(|child| value_as_integer(child.value.as_ref()))?;
    Some(CConstant { name, value })
}

fn value_as_integer(value: Option<&serde_json::Value>) -> Option<i64> {
    match value? {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::String(s) => parse_integer_literal(s),
        _ => None,
    }
}

fn collect_macro_constants(model: &mut ScanModel, macros: Vec<String>, lib: &str) {
    for line in macros {
        let Some(rest) = line.strip_prefix("#define ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        if name.contains('(') || !constant_name_matches_library(name, lib) {
            continue;
        }
        let Some(raw_value) = parts.next() else {
            continue;
        };
        if let Some(value) = parse_integer_literal(raw_value) {
            model.constants.push(CConstant {
                name: name.to_string(),
                value,
            });
        }
    }
}

fn dedupe_constants(constants: &mut Vec<CConstant>) {
    let mut by_name = BTreeMap::new();
    for constant in constants.drain(..) {
        by_name.entry(constant.name.clone()).or_insert(constant);
    }
    constants.extend(by_name.into_values());
}

fn node_name_matches_library(node: &AstNode, lib: &str) -> bool {
    let Some(name) = node.name.as_deref() else {
        return false;
    };
    symbol_name_matches_library(name, lib)
}

fn symbol_name_matches_library(name: &str, lib: &str) -> bool {
    match lib {
        "sdl2" | "sdl" => name.starts_with("SDL_"),
        _ => false,
    }
}

fn constant_name_matches_library(name: &str, lib: &str) -> bool {
    symbol_name_matches_library(name, lib)
}

fn node_is_from_header(node: &AstNode, header_path: Option<&str>) -> bool {
    let Some(header_path) = header_path else {
        return true;
    };
    let header_dir = PathBuf::from(header_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string());
    let Some(range) = node.range.as_ref() else {
        return false;
    };
    let Some(begin) = range.begin.as_ref() else {
        return false;
    };
    let file = begin
        .expansion_loc
        .as_ref()
        .and_then(|loc| loc.file.as_ref())
        .or(begin.file.as_ref());
    if let Some(file) = file {
        return file_matches_header_scope(file, header_path, header_dir.as_deref());
    }
    if let Some(included_from) = begin
        .included_from
        .as_ref()
        .and_then(|inc| inc.file.as_ref())
    {
        return file_matches_header_scope(included_from, header_path, header_dir.as_deref());
    }
    true
}

fn file_matches_header_scope(file: &str, header_path: &str, header_dir: Option<&str>) -> bool {
    if file == header_path {
        return true;
    }
    if let Some(header_dir) = header_dir {
        return file.starts_with(header_dir);
    }
    false
}

fn function_from_node(node: &AstNode) -> Option<CFunction> {
    let name = node.name.clone()?;
    let signature = node.ty.as_ref()?.qual_type.clone()?;
    let return_ty = signature
        .split_once('(')
        .map(|(ret, _)| ret.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let mut params = Vec::new();
    for child in node.inner.as_ref().into_iter().flatten() {
        if child.kind.as_deref() != Some("ParmVarDecl") {
            continue;
        }
        let ty = child.ty.as_ref()?.qual_type.clone()?;
        params.push(CParam {
            name: child.name.clone().unwrap_or_default(),
            ty,
        });
    }
    Some(CFunction {
        name,
        signature,
        return_ty,
        params,
    })
}

fn dedupe_functions(functions: &mut Vec<CFunction>) {
    let mut seen = BTreeSet::new();
    functions.retain(|f| seen.insert(format!("{}:{}", f.name, f.signature)));
}

fn dedupe_generated_resources(resources: &mut Vec<GeneratedResource>) {
    let mut by_surface: BTreeMap<String, GeneratedResource> = BTreeMap::new();
    for resource in resources.drain(..) {
        match by_surface.get(&resource.surface_type) {
            Some(existing) if resource.constructor.len() >= existing.constructor.len() => {}
            _ => {
                by_surface.insert(resource.surface_type.clone(), resource);
            }
        }
    }
    resources.extend(by_surface.into_values());
}

fn dedupe_generated_functions(functions: &mut Vec<GeneratedFunction>) {
    let mut by_surface: BTreeMap<String, GeneratedFunction> = BTreeMap::new();
    for function in functions.drain(..) {
        match by_surface.get(&function.surface) {
            Some(existing) if function_rank(existing) >= function_rank(&function) => {}
            _ => {
                by_surface.insert(function.surface.clone(), function);
            }
        }
    }
    functions.extend(by_surface.into_values());
}

fn function_rank(function: &GeneratedFunction) -> u8 {
    match function.c_function.as_str() {
        "compress2" => 4,
        "uncompress" => 4,
        "compress" => 3,
        "uncompress2" => 2,
        _ if function.safety == Safety::CopyOnly => 1,
        _ => 0,
    }
}

fn classify(
    lib: &str,
    functions: &[CFunction],
) -> (Vec<GeneratedResource>, Vec<GeneratedFunction>) {
    let destructors = destructor_map(functions);
    let mut resources = Vec::new();
    let mut generated_functions = Vec::new();

    for function in functions {
        if let Some(resource) = classify_constructor(lib, function, &destructors) {
            generated_functions.push(GeneratedFunction {
                name: surface_name(&function.name),
                c_function: function.name.clone(),
                surface: format!("{}.{}", lib, surface_name_for_lib(lib, &function.name)),
                input: infer_input(function),
                output: "resource".to_string(),
                c_return_type: normalize_type(&function.return_ty),
                c_param_types: normalized_param_types(function),
                safety: Safety::Safe,
                reason: format!(
                    "constructor `{}` is registered as OM resource `{}`",
                    function.name, resource.surface_type
                ),
            });
            resources.push(resource);
        }
    }

    let resource_types: Vec<String> = resources.iter().map(|r| r.c_type.clone()).collect();

    for function in functions {
        // If it's a constructor we already added, skip it for general functions
        if resources.iter().any(|r| r.constructor == function.name) {
            continue;
        }
        generated_functions.push(classify_function(lib, function, &resource_types));
    }

    (resources, generated_functions)
}

fn destructor_map(functions: &[CFunction]) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    for function in functions {
        if !is_destructor_name(&function.name) || function.params.len() != 1 {
            continue;
        }
        let ty = normalize_type(&function.params[0].ty);
        if is_single_pointer(&ty) && normalize_type(&function.return_ty) == "void" {
            match out.get(&ty) {
                Some(existing)
                    if destructor_rank(existing, &ty) >= destructor_rank(&function.name, &ty) => {}
                _ => {
                    out.insert(ty, function.name.clone());
                }
            }
        }
    }
    out
}

fn destructor_rank(name: &str, c_type: &str) -> u8 {
    let n = name.to_ascii_lowercase();
    let base = c_type
        .trim_end_matches('*')
        .trim()
        .trim_start_matches("SDL_")
        .trim_start_matches("SDL")
        .to_ascii_lowercase();
    let exact_destroy = format!("destroy_{base}");
    let exact_free = format!("free_{base}");
    if n.ends_with(&exact_destroy) || n.ends_with(&exact_free) {
        4
    } else if n.contains(&base) {
        2
    } else {
        1
    }
}

fn classify_constructor(
    lib: &str,
    function: &CFunction,
    destructors: &HashMap<String, String>,
) -> Option<GeneratedResource> {
    if !is_constructor_name(&function.name) {
        return None;
    }

    // Pattern 1: out-param Type**
    if let Some(out_param) = function
        .params
        .iter()
        .find(|p| is_double_pointer(&normalize_type(&p.ty)))
    {
        let c_type = normalize_type(&out_param.ty)
            .trim_end_matches('*')
            .trim()
            .to_string()
            + "*";
        if let Some(destructor) = destructors.get(&c_type) {
            let resource_name = resource_name_from_c_type(lib, &c_type);
            return Some(GeneratedResource {
                name: resource_name.clone(),
                c_type: c_type.clone(),
                constructor: function.name.clone(),
                destructor: destructor.clone(),
                surface_type: format!("{lib}.{resource_name}"),
                safety: Safety::Safe,
                reason: format!(
                    "constructor `{}` writes `{}` and paired destructor `{}` accepts `{}`",
                    function.name, out_param.ty, destructor, c_type
                ),
            });
        }
    }

    // Pattern 2: returns Type*
    let ret_ty = normalize_type(&function.return_ty);
    if is_single_pointer(&ret_ty) && !is_const_char_pointer(&ret_ty) {
        if let Some(destructor) = destructors.get(&ret_ty) {
            let resource_name = resource_name_from_c_type(lib, &ret_ty);
            return Some(GeneratedResource {
                name: resource_name.clone(),
                c_type: ret_ty.clone(),
                constructor: function.name.clone(),
                destructor: destructor.clone(),
                surface_type: format!("{lib}.{resource_name}"),
                safety: Safety::Safe,
                reason: format!(
                    "constructor `{}` returns `{}` and paired destructor `{}` accepts `{}`",
                    function.name, ret_ty, destructor, ret_ty
                ),
            });
        }
    }

    None
}

fn classify_function(
    lib: &str,
    function: &CFunction,
    resource_types: &[String],
) -> GeneratedFunction {
    if matches!(lib, "sdl2" | "sdl") && function.name == "SDL_PollEvent" {
        return GeneratedFunction {
            name: "poll_event".to_string(),
            c_function: function.name.clone(),
            surface: format!("{lib}.poll_event"),
            input: "unit".to_string(),
            output: "value".to_string(),
            c_return_type: normalize_type(&function.return_ty),
            c_param_types: Vec::new(),
            safety: Safety::Safe,
            reason:
                "SDL_Event is allocated on the stack by the OM-Snask-System; Snask receives only the event type"
                    .to_string(),
        };
    }

    if let Some(copy) = classify_zlib_buffer_function(lib, function) {
        return copy;
    }
    if is_const_char_pointer(&function.return_ty)
        && !function.params.iter().any(|p| p.ty.contains('*'))
    {
        return GeneratedFunction {
            name: surface_name(&function.name),
            c_function: function.name.clone(),
            surface: format!("{}.{}", lib, surface_name_for_lib(lib, &function.name)),
            input: infer_input(function),
            output: "str".to_string(),
            c_return_type: normalize_type(&function.return_ty),
            c_param_types: normalized_param_types(function),
            safety: Safety::CopyOnly,
            reason: "returns const char*; scanner exposes only an OM string copy".to_string(),
        };
    }

    // Pattern 3: Opaque resource method
    // First parameter is a known resource pointer, and no other raw pointers
    if !function.params.is_empty() {
        let first_ty = normalize_type(&function.params[0].ty);
        if resource_types.contains(&first_ty) {
            let other_pointers = function.params[1..]
                .iter()
                .any(|p| is_pointer(&normalize_type(&p.ty)));
            let ret_ptr = is_pointer(&normalize_type(&function.return_ty))
                && !is_const_char_pointer(&normalize_type(&function.return_ty));

            if !other_pointers && !ret_ptr {
                return GeneratedFunction {
                    name: surface_name(&function.name),
                    c_function: function.name.clone(),
                    surface: format!("{}.{}", lib, surface_name_for_lib(lib, &function.name)),
                    input: infer_resource_method_input(function),
                    output: infer_output(function),
                    c_return_type: normalize_type(&function.return_ty),
                    c_param_types: normalized_param_types(function),
                    safety: Safety::Safe,
                    reason: format!(
                        "method `{}` takes recognized resource `{}` as first argument",
                        function.name, first_ty
                    ),
                };
            }
        }
    }

    let (safety, reason) = if is_destructor_name(&function.name) {
        (
            Safety::Blocked,
            "cleanup/destructor functions are hidden behind OM zone cleanup".to_string(),
        )
    } else if infer_input(function) != "blocked" {
        let ret_ty = normalize_type(&function.return_ty);
        if is_c_numeric_type(&ret_ty) || ret_ty == "void" || is_const_char_pointer(&ret_ty) {
            (
                Safety::Safe,
                "all parameters and return type are OM-safe".to_string(),
            )
        } else {
            (
                Safety::Blocked,
                format!("unsupported return type `{}`", ret_ty),
            )
        }
    } else if returns_or_accepts_raw_pointer(function) {
        (
            Safety::Blocked,
            "raw pointer ownership is ambiguous; scanner refuses to expose it".to_string(),
        )
    } else if function.return_ty == "int" && function.params.iter().any(|p| p.ty.contains('*')) {
        (
            Safety::Blocked,
            "int status with pointer out-param needs a proven OM copy/ownership rule".to_string(),
        )
    } else {
        (
            Safety::Blocked,
            "no safe OM mapping recognized by MVP heuristics".to_string(),
        )
    };

    GeneratedFunction {
        name: surface_name(&function.name),
        c_function: function.name.clone(),
        surface: format!("{}.{}", lib, surface_name_for_lib(lib, &function.name)),
        input: infer_input(function),
        output: infer_output(function),
        c_return_type: normalize_type(&function.return_ty),
        c_param_types: normalized_param_types(function),
        safety,
        reason: reason.to_string(),
    }
}

fn classify_zlib_buffer_function(lib: &str, function: &CFunction) -> Option<GeneratedFunction> {
    let names: Vec<_> = function.params.iter().map(|p| p.name.as_str()).collect();
    let has_dest = names.iter().any(|n| *n == "dest");
    let has_dest_len = names.iter().any(|n| *n == "destLen" || *n == "dest_len");
    let has_source = names.iter().any(|n| *n == "source" || *n == "src");
    let has_source_len = names
        .iter()
        .any(|n| *n == "sourceLen" || *n == "source_len");
    if function.return_ty != "int" || !(has_dest && has_dest_len && has_source && has_source_len) {
        return None;
    }

    let (surface, input, output, public_name) = match function.name.as_str() {
        "compress2" | "compress" => ("compress", "str", "bytes", "compress"),
        "uncompress" | "uncompress2" => ("decompress", "bytes", "str", "decompress"),
        _ => return None,
    };

    Some(GeneratedFunction {
        name: public_name.to_string(),
        c_function: function.name.clone(),
        surface: format!("{lib}.{surface}"),
        input: input.to_string(),
        output: output.to_string(),
        c_return_type: normalize_type(&function.return_ty),
        c_param_types: normalized_param_types(function),
        safety: Safety::CopyOnly,
        reason: "recognized bounded source/dest buffer API; scanner exposes only OM-owned copies"
            .to_string(),
    })
}

fn render_contract(
    lib: &str,
    resources: &[GeneratedResource],
    functions: &[GeneratedFunction],
    constants: &[CConstant],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("library {lib}\n"));
    out.push_str("# generated_by: snask om scan\n");
    out.push_str("# policy: expose only SAFE and COPY_ONLY entries\n\n");

    for constant in constants {
        out.push_str(&format!(
            "constant {}: {}\n",
            constant_surface_name_for_lib(lib, &constant.name),
            constant.value
        ));
    }
    if !constants.is_empty() {
        out.push('\n');
    }

    for resource in resources {
        out.push_str(&format!("resource {}:\n", resource.name));
        out.push_str(&format!("    c_type: {}\n", resource.c_type));
        out.push_str(&format!("    constructor: {}\n", resource.constructor));
        out.push_str(&format!("    destructor: {}\n", resource.destructor));
        out.push_str(&format!("    surface_type: {}\n", resource.surface_type));
        out.push_str(&format!("    safety: {}\n", resource.safety.as_str()));
        out.push_str(&format!("    reason: {}\n\n", resource.reason));
    }

    for function in functions {
        out.push_str(&format!("function {}:\n", function.name));
        out.push_str(&format!("    c_function: {}\n", function.c_function));
        out.push_str(&format!("    surface: {}\n", function.surface));
        out.push_str(&format!("    input: {}\n", function.input));
        out.push_str(&format!("    output: {}\n", function.output));
        out.push_str(&format!("    c_return_type: {}\n", function.c_return_type));
        if !function.c_param_types.is_empty() {
            out.push_str(&format!(
                "    c_param_types: {}\n",
                function.c_param_types.join(", ")
            ));
        }
        out.push_str(&format!("    safety: {}\n", function.safety.as_str()));
        out.push_str(&format!("    reason: {}\n\n", function.reason));
    }

    out
}

fn render_report(
    options: &ScanOptions,
    model: &ScanModel,
    resources: &[GeneratedResource],
    functions: &[GeneratedFunction],
) -> String {
    let mut counts = BTreeMap::new();
    for resource in resources {
        *counts.entry(resource.safety).or_insert(0usize) += 1;
    }
    for function in functions {
        *counts.entry(function.safety).or_insert(0usize) += 1;
    }

    let mut out = String::new();
    out.push_str(&format!(
        "OM scan report for `{}` as `{}`\n",
        options.header, options.lib
    ));
    out.push_str(&format!(
        "AST: {} functions, {} typedefs, {} structs/unions, {} enums\n\n",
        model.functions.len(),
        model.typedefs.len(),
        model.records.len(),
        model.enums.len()
    ));
    out.push_str(&format!(
        "SAFE: {}\nCOPY_ONLY: {}\nBLOCKED: {}\n\n",
        counts.get(&Safety::Safe).copied().unwrap_or(0),
        counts.get(&Safety::CopyOnly).copied().unwrap_or(0),
        counts.get(&Safety::Blocked).copied().unwrap_or(0)
    ));

    out.push_str("Exposed resources:\n");
    for r in resources.iter().filter(|r| r.safety == Safety::Safe) {
        out.push_str(&format!(
            "- {} via {} / {}; {}\n",
            r.surface_type, r.constructor, r.destructor, r.reason
        ));
    }
    if !resources.iter().any(|r| r.safety == Safety::Safe) {
        out.push_str("- none\n");
    }

    out.push_str("\nExposed functions:\n");
    for f in functions
        .iter()
        .filter(|f| matches!(f.safety, Safety::Safe | Safety::CopyOnly))
    {
        out.push_str(&format!(
            "- {} => {} ({}, {} -> {}); {}\n",
            f.surface,
            f.c_function,
            f.safety.as_str(),
            f.input,
            f.output,
            f.reason
        ));
    }

    out.push_str("\nBlocked examples:\n");
    for f in functions
        .iter()
        .filter(|f| f.safety == Safety::Blocked)
        .take(25)
    {
        out.push_str(&format!("- {}: {}\n", f.c_function, f.reason));
    }
    out
}

fn report_path_for(contract_path: &str) -> String {
    let path = PathBuf::from(contract_path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("om_scan");
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    parent
        .join(format!("{stem}.report.txt"))
        .to_string_lossy()
        .to_string()
}

fn normalize_type(ty: &str) -> String {
    ty.replace(" *", "*")
        .replace("* ", "*")
        .replace("const ", "")
        .replace("struct ", "")
        .trim()
        .to_string()
}

fn is_single_pointer(ty: &str) -> bool {
    ty.ends_with('*') && !ty.ends_with("**")
}

fn is_double_pointer(ty: &str) -> bool {
    ty.ends_with("**")
}

fn is_pointer(ty: &str) -> bool {
    ty.contains('*')
}

fn is_const_char_pointer(ty: &str) -> bool {
    let ty = ty.trim();
    ty.contains("const char") || ty.contains("z_const char")
}

fn infer_output(function: &CFunction) -> String {
    let ty = normalize_type(&function.return_ty);
    if ty == "void" {
        "unit".to_string()
    } else if is_c_numeric_type(&ty) {
        "value".to_string()
    } else if is_const_char_pointer(&ty) {
        "str".to_string()
    } else {
        "nil".to_string()
    }
}

fn returns_or_accepts_raw_pointer(function: &CFunction) -> bool {
    function.return_ty.contains('*') || function.params.iter().any(|p| p.ty.contains('*'))
}

fn is_constructor_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("open") || n.contains("create") || n.contains("new") || n.contains("init")
}

fn is_destructor_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("close") || n.contains("free") || n.contains("destroy") || n.contains("finalize")
}

fn resource_name_from_c_type(lib: &str, c_type: &str) -> String {
    let base = c_type
        .trim_end_matches('*')
        .trim()
        .trim_start_matches(lib)
        .trim_start_matches('_');
    if lib == "sqlite" && base == "3" {
        return "Database".to_string();
    }
    if lib == "sqlite" && base == "3_stmt" {
        return "Statement".to_string();
    }
    if lib == "sqlite" && base == "3_blob" {
        return "Blob".to_string();
    }
    let base = if base.is_empty() {
        c_type.trim_end_matches('*').trim()
    } else {
        base
    };
    base.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn surface_name(c_name: &str) -> String {
    camel_to_snake(c_name.trim_start_matches('_'))
}

fn surface_name_for_lib(lib: &str, c_name: &str) -> String {
    let trimmed = c_name.trim_start_matches('_');
    let without_prefix = match lib {
        "sdl2" | "sdl" => trimmed.strip_prefix("SDL_").unwrap_or(trimmed),
        _ => trimmed,
    };
    surface_name(without_prefix)
}

fn constant_surface_name_for_lib(lib: &str, c_name: &str) -> String {
    let trimmed = c_name.trim_start_matches('_');
    match lib {
        "sdl2" | "sdl" => trimmed
            .strip_prefix("SDL_")
            .unwrap_or(trimmed)
            .to_ascii_uppercase(),
        _ => trimmed.to_string(),
    }
}

fn camel_to_snake(name: &str) -> String {
    let mut out = String::new();
    let mut prev_was_lower_or_digit = false;
    for ch in name.chars() {
        if ch == '_' || ch == '-' || ch == ' ' {
            if !out.ends_with('_') {
                out.push('_');
            }
            prev_was_lower_or_digit = false;
        } else if ch.is_ascii_uppercase() {
            if prev_was_lower_or_digit && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = false;
        } else {
            out.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }
    out.trim_matches('_').to_string()
}

fn is_c_numeric_type(ty: &str) -> bool {
    matches!(
        ty,
        "int"
            | "unsigned int"
            | "signed int"
            | "long"
            | "unsigned long"
            | "long long"
            | "unsigned long long"
            | "short"
            | "unsigned short"
            | "char"
            | "unsigned char"
            | "size_t"
            | "float"
            | "double"
            | "Uint8"
            | "Uint16"
            | "Uint32"
            | "Uint64"
            | "Sint8"
            | "Sint16"
            | "Sint32"
            | "Sint64"
            | "SDL_bool"
    )
}

fn normalized_param_types(function: &CFunction) -> Vec<String> {
    function
        .params
        .iter()
        .map(|param| normalize_type(&param.ty))
        .collect()
}

fn parse_integer_literal(raw: &str) -> Option<i64> {
    let cleaned = raw
        .trim()
        .trim_matches('(')
        .trim_matches(')')
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

fn infer_input(function: &CFunction) -> String {
    if function.params.is_empty() {
        return "unit".to_string();
    }

    let mut inputs = Vec::new();
    for p in &function.params {
        let raw_ty = p.ty.trim();
        let ty = normalize_type(&p.ty);
        if is_const_char_pointer(raw_ty) {
            inputs.push("str");
        } else if !ty.contains('*') && is_c_numeric_type(&ty) {
            inputs.push("value");
        } else if is_single_pointer(&ty) || is_double_pointer(&ty) {
            return "blocked".to_string();
        } else if ty.contains('(') && ty.contains(')') {
            return "blocked".to_string();
        } else {
            return "blocked".to_string();
        }
    }

    if inputs.len() == 1 {
        inputs[0].to_string()
    } else {
        "value".to_string()
    }
}

fn infer_resource_method_input(function: &CFunction) -> String {
    if function.params.is_empty() {
        return "unit".to_string();
    }

    let mut inputs = vec!["resource"];
    for p in function.params.iter().skip(1) {
        let raw_ty = p.ty.trim();
        let ty = normalize_type(&p.ty);
        if is_const_char_pointer(raw_ty) {
            inputs.push("str");
        } else if !ty.contains('*') && is_c_numeric_type(&ty) {
            inputs.push("value");
        } else if ty.contains('(') && ty.contains(')') {
            return "blocked".to_string();
        } else {
            return "blocked".to_string();
        }
    }

    if inputs.len() == 1 {
        inputs[0].to_string()
    } else {
        "value".to_string()
    }
}
