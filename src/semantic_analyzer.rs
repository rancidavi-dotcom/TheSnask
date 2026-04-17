use crate::ast::{Program, Stmt, StmtKind, Expr, ExprKind, VarDecl, BinaryOp, UnaryOp, LiteralValue, ConditionalStmt, LoopStmt, ClassDecl, FuncDecl};
use crate::span::Span;
use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticSymbolKind {
    Immutable,
    Mutable,
    Constant,
    Function,
    Parameter,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSymbol {
    pub name: String,
    pub symbol_type: Type,
    pub kind: SemanticSymbolKind,
    pub is_variadic: bool,
}

#[derive(Debug, Clone)]
pub struct SemanticSymbolTable {
    scopes: Vec<HashMap<String, SemanticSymbol>>,
}

impl SemanticSymbolTable {
    pub fn new() -> Self {
        let mut table = SemanticSymbolTable { scopes: Vec::new() };
        table.enter_scope();
        table
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, symbol: SemanticSymbol) -> bool {
        let is_global = self.scopes.len() == 1;
        let current_scope = self.scopes.last_mut().expect("No scope available");
        
        if is_global {
             current_scope.insert(symbol.name.clone(), symbol);
             return true;
        }
        
        if current_scope.contains_key(&symbol.name) {
            return false;
        }
        current_scope.insert(symbol.name.clone(), symbol);
        true
    }

    pub fn lookup(&self, name: &str) -> Option<&SemanticSymbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn visible_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        for scope in self.scopes.iter().rev() {
            for k in scope.keys() {
                out.push(k.clone());
            }
        }
        out
    }
}


#[derive(Debug, Clone)]
pub enum SemanticErrorKind {
    VariableAlreadyDeclared(String),
    VariableNotFound(String),
    FunctionAlreadyDeclared(String),
    FunctionNotFound(String),
    UnknownType(String),
    MissingReturn { function: String, expected: Type },
    TypeMismatch { expected: Type, found: Type },
    InvalidOperation { op: String, type1: Type, type2: Option<Type> },
    ImmutableAssignment(String),
    ReturnOutsideFunction,
    WrongNumberOfArguments { expected: usize, found: usize },
    IndexAccessOnNonIndexable(Type),
    InvalidIndexType(Type),
    PropertyNotFound(String),
    NotCallable(Type),
    RestrictedNativeFunction { name: String, help: String },
    TinyDisallowedLib { lib: String },
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: SemanticErrorKind,
    pub span: Span,
    pub help: Option<String>,
    pub notes: Vec<String>,
}

impl SemanticError {
    pub fn new(kind: SemanticErrorKind, span: Span) -> Self {
        SemanticError {
            kind,
            span,
            help: None,
            notes: Vec::new(),
        }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn message(&self) -> String {
        use SemanticErrorKind::*;
        match &self.kind {
            VariableAlreadyDeclared(name) => format!("Variable '{}' is already declared.", name),
            VariableNotFound(name) => format!("Variable '{}' not found.", name),
            FunctionAlreadyDeclared(name) => format!("Function '{}' is already declared.", name),
            FunctionNotFound(name) => format!("Function '{}' not found.", name),
            UnknownType(name) => format!("Type '{}' is not defined.", name),
            MissingReturn { function, expected } => format!(
                "Function '{}' declares return type {:?} but does not return on every path.",
                function, expected
            ),
            TypeMismatch { expected, found } => {
                format!("Type mismatch: expected {:?}, found {:?}.", expected, found)
            }
            InvalidOperation { op, type1, type2 } => {
                if let Some(t2) = type2 {
                    format!("Invalid operation: '{}' between {:?} and {:?}.", op, type1, t2)
                } else {
                    format!("Invalid operation: '{}' on {:?}.", op, type1)
                }
            }
            ImmutableAssignment(name) => format!(
                "'{}' is immutable. Tip: declare it as 'mut {} = ...;'.",
                name, name
            ),
            ReturnOutsideFunction => "Using 'return' outside a function.".to_string(),
            WrongNumberOfArguments { expected, found } => format!(
                "Wrong number of arguments: expected {}, found {}.",
                expected, found
            ),
            IndexAccessOnNonIndexable(t) => format!("Type {:?} is not indexable.", t),
            InvalidIndexType(t) => format!("Index must be a number, found {:?}.", t),
            PropertyNotFound(name) => format!("Property '{}' not found.", name),
            NotCallable(t) => format!("Type {:?} is not callable.", t),
            RestrictedNativeFunction { name, .. } => format!("Native function '{}' is restricted.", name),
            TinyDisallowedLib { lib } => format!("Library '{}' is not allowed in --tiny builds.", lib),
        }
    }

    pub fn code(&self) -> &'static str {
        use SemanticErrorKind::*;
        match &self.kind {
            VariableAlreadyDeclared(_) => "SNASK-SEM-VAR-REDECL",
            VariableNotFound(_) => "SNASK-SEM-VAR-NOT-FOUND",
            FunctionAlreadyDeclared(_) => "SNASK-SEM-FUN-REDECL",
            FunctionNotFound(_) => "SNASK-SEM-FUN-NOT-FOUND",
            UnknownType(_) => "SNASK-SEM-UNKNOWN-TYPE",
            MissingReturn { .. } => "SNASK-SEM-MISSING-RETURN",
            TypeMismatch { .. } => "SNASK-SEM-TYPE-MISMATCH",
            InvalidOperation { .. } => "SNASK-SEM-INVALID-OP",
            ImmutableAssignment(_) => "SNASK-SEM-IMMUTABLE-ASSIGN",
            ReturnOutsideFunction => "SNASK-SEM-RETURN-OUTSIDE",
            WrongNumberOfArguments { .. } => "SNASK-SEM-ARG-COUNT",
            IndexAccessOnNonIndexable(_) => "SNASK-SEM-NOT-INDEXABLE",
            InvalidIndexType(_) => "SNASK-SEM-INDEX-TYPE",
            PropertyNotFound(_) => "SNASK-SEM-PROP-NOT-FOUND",
            NotCallable(_) => "SNASK-SEM-NOT-CALLABLE",
            RestrictedNativeFunction { .. } => "SNASK-SEM-RESTRICTED-NATIVE",
            TinyDisallowedLib { .. } => "SNASK-TINY-DISALLOWED-LIB",
        }
    }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SemanticSymbolTable,
    current_function_return_type: Option<Type>,
    current_class: Option<String>,
    classes: HashMap<String, ClassDecl>,
    pub errors: Vec<SemanticError>,
    tiny_mode: bool,
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1)
                .min(cur[j] + 1)
                .min(prev[j] + cost);
        }
        prev.clone_from(&cur);
    }
    prev[b.len()]
}

fn suggest_name(target: &str, candidates: &[String]) -> Option<(String, u8)> {
    let mut scored: Vec<(String, usize)> = candidates
        .iter()
        .map(|c| (c.clone(), levenshtein(target, c)))
        .collect();
    scored.sort_by_key(|(_, d)| *d);
    let (best, best_d) = scored.first()?.clone();
    let second_d = scored.get(1).map(|(_, d)| *d).unwrap_or(usize::MAX);
    if best_d <= 2 && best_d + 1 < second_d {
        let confidence = match best_d {
            0 => 100,
            1 => 95,
            2 => 90,
            _ => 0,
        };
        return Some((best, confidence));
    }
    None
}

fn is_library_native(name: &str) -> bool {
    if name.contains("::") { return false; }
    name.starts_with("sqlite_") || name.starts_with("gui_") || name.starts_with("skia_") || 
    name.starts_with("blaze_") || name.starts_with("auth_") || name.starts_with("sfs_") || 
    name.starts_with("path_") || name.starts_with("os_") || name.starts_with("s_http_") || 
    name.starts_with("thread_") || name.starts_with("json_") || name.starts_with("sjson_") || 
    name.starts_with("snif_") || name.starts_with("string_")
}

fn library_native_help(name: &str) -> String {
    let lib = if name.starts_with("sqlite_") { "sqlite" }
    else if name.starts_with("gui_") { "gui" }
    else if name.starts_with("skia_") { "snask_skia" }
    else if name.starts_with("blaze_") { "blaze" }
    else if name.starts_with("auth_") { "blaze_auth" }
    else if name.starts_with("sfs_") || name.starts_with("path_") { "sfs" }
    else if name.starts_with("os_") { "os" }
    else if name.starts_with("s_http_") { "requests" }
    else if name.starts_with("thread_") { "os" }
    else if name.starts_with("json_") { "json" }
    else if name.starts_with("sjson_") { "sjson" }
    else if name.starts_with("snif_") { "snif" }
    else if name.starts_with("string_") { "string" }
    else { "a library" };

    format!(
        "This native function is reserved for libraries.\n\nHow to fix:\n- Use `import \"{lib}\"` and call functions via the module namespace (e.g. `{lib}::...`).\n",
        lib = lib
    )
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = SemanticAnalyzer {
            symbol_table: SemanticSymbolTable::new(),
            current_function_return_type: None,
            current_class: None,
            classes: HashMap::new(),
            errors: Vec::new(),
            tiny_mode: false,
        };
        analyzer.register_stdlib();
        analyzer
    }

    pub fn set_tiny_mode(&mut self, tiny: bool) {
        self.tiny_mode = tiny;
    }

    fn mk_variable_not_found(&self, name: String, span: Span) -> SemanticError {
        let candidates = self.symbol_table.visible_names();
        let mut e = SemanticError::new(SemanticErrorKind::VariableNotFound(name.clone()), span);
        if let Some((best, conf)) = suggest_name(&name, &candidates) {
            if conf >= 90 {
                e = e.with_help(format!("Did you mean '{best}'?"));
            } else {
                e = e.with_note(format!("Possible match: '{best}' ({conf}%)"));
            }
        }
        e
    }

    fn mk_function_not_found(&self, name: String, span: Span) -> SemanticError {
        let candidates = self.symbol_table.visible_names();
        let mut e = SemanticError::new(SemanticErrorKind::FunctionNotFound(name.clone()), span);
        if let Some((best, conf)) = suggest_name(&name, &candidates) {
            if conf >= 90 {
                e = e.with_help(format!("Did you mean '{best}'?"));
            } else {
                e = e.with_note(format!("Possible match: '{best}' ({conf}%)"));
            }
        }
        e
    }

    fn mk_unknown_type(&self, name: String, span: Span) -> SemanticError {
        let candidates = self.symbol_table.visible_names();
        let mut e = SemanticError::new(SemanticErrorKind::UnknownType(name.clone()), span);
        if let Some((best, conf)) = suggest_name(&name, &candidates) {
            if conf >= 90 {
                e = e.with_help(format!("Did you mean type '{best}'?"));
            } else {
                e = e.with_note(format!("Possible match: '{best}' ({conf}%)"));
            }
        }
        e
    }

    fn register_stdlib(&mut self) {
        self.define_module_as_any("math");
        self.define_module_as_any("string");
        self.define_module_as_any("collections");

        self.define_builtin("abs", vec![Type::Float], Type::Float, false);
        self.define_builtin("floor", vec![Type::Float], Type::Float, false);
        self.define_builtin("ceil", vec![Type::Float], Type::Float, false);
        self.define_builtin("round", vec![Type::Float], Type::Float, false);
        self.define_builtin("pow", vec![Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("sqrt", vec![Type::Float], Type::Float, false);
        self.define_builtin("min", vec![], Type::Any, true);
        self.define_builtin("max", vec![], Type::Any, true);
        self.define_builtin("sin", vec![Type::Float], Type::Float, false);
        self.define_builtin("cos", vec![Type::Float], Type::Float, false);

        self.define_constant("PI", Type::Float);
        self.define_constant("E", Type::Float);
        self.define_constant("TAU", Type::Float);

        self.define_builtin("len", vec![Type::Any], Type::Float, false);
        self.define_builtin("upper", vec![Type::String], Type::String, false);
        self.define_builtin("lower", vec![Type::String], Type::String, false);
        self.define_builtin("trim", vec![Type::String], Type::String, false);
        self.define_builtin("split", vec![Type::String, Type::String], Type::List, false);
        self.define_builtin("join", vec![Type::List, Type::String], Type::String, false);
        self.define_builtin("replace", vec![Type::String, Type::String, Type::String], Type::String, false);
        self.define_builtin("contains", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("starts_with", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("ends_with", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("chars", vec![Type::String], Type::List, false);
        self.define_builtin("substring", vec![Type::String, Type::Float, Type::Float], Type::String, false);
        self.define_builtin("format", vec![Type::String], Type::String, true);

        self.define_builtin("range", vec![Type::Float], Type::List, false);
        self.define_builtin("sort", vec![Type::List], Type::List, false);
        self.define_builtin("reverse", vec![Type::List], Type::List, false);
        self.define_builtin("unique", vec![Type::List], Type::List, false);
        self.define_builtin("flatten", vec![Type::List], Type::List, false);

        self.define_builtin("is_nil", vec![Type::Any], Type::Bool, false);
        self.define_builtin("is_str", vec![Type::Any], Type::Bool, false);
        self.define_builtin("is_obj", vec![Type::Any], Type::Bool, false);

        self.define_builtin("read_file", vec![Type::String], Type::String, false);
        self.define_builtin("write_file", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("append_file", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("exists", vec![Type::String], Type::Bool, false);
        self.define_builtin("delete", vec![Type::String], Type::Void, false);
        self.define_builtin("read_dir", vec![Type::String], Type::List, false);
        self.define_builtin("is_file", vec![Type::String], Type::Bool, false);
        self.define_builtin("is_dir", vec![Type::String], Type::Bool, false);
        self.define_builtin("create_dir", vec![Type::String], Type::Void, false);

        self.define_builtin("http_get", vec![Type::String], Type::Dict, false);
        self.define_builtin("http_post", vec![Type::String, Type::String], Type::Void, false);

        self.define_builtin("time", vec![], Type::Float, false);
        self.define_builtin("sleep", vec![Type::Float], Type::Void, false);
        self.define_builtin("exit", vec![Type::Float], Type::Void, false);
        self.define_builtin("free", vec![Type::Any], Type::Void, false);
        self.define_builtin("arena_reset", vec![], Type::Void, false);
        self.define_builtin("__s_call_by_name", vec![Type::String, Type::Any, Type::Any, Type::Any], Type::Any, false);
        self.define_builtin("args", vec![], Type::List, false);
        self.define_builtin("env", vec![Type::String], Type::String, false);
        self.define_builtin("set_env", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("cwd", vec![], Type::String, false);
        self.define_builtin("platform", vec![], Type::String, false);
        self.define_builtin("arch", vec![], Type::String, false);

        self.define_builtin("str_to_num", vec![Type::String], Type::Float, false);
        self.define_builtin("num_to_str", vec![Type::Float], Type::String, false);
        self.define_builtin("calc_eval", vec![Type::String], Type::Float, false);

        // Core natives aliased
        self.define_builtin_with_alias("os_cwd", vec![], Type::String, false);
        self.define_builtin_with_alias("string_len", vec![Type::Any], Type::Float, false);

        // JSON (via `import "json"`; module calls compile to `__json_*`)
        self.define_builtin_with_alias("json_parse", vec![Type::String], Type::Any, false);
        self.define_builtin_with_alias("json_stringify", vec![Type::Any], Type::String, false);
        self.define_builtin_with_alias("json_stringify_pretty", vec![Type::Any], Type::String, false);
        self.define_builtin_with_alias("json_get", vec![Type::Any, Type::String], Type::Any, false);
        self.define_builtin_with_alias("json_has", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("json_len", vec![Type::Any], Type::Float, false);
        self.define_builtin_with_alias("json_index", vec![Type::Any, Type::Float], Type::Any, false);
        self.define_builtin_with_alias("json_set", vec![Type::Any, Type::String, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("json_keys", vec![Type::Any], Type::Any, false);
        self.define_builtin_with_alias("json_parse_ex", vec![Type::String], Type::Any, false);

        // SNIF (library surface over SNIF runtime)
        self.define_builtin_with_alias("snif_new_object", vec![], Type::Any, false);
        self.define_builtin_with_alias("snif_new_array", vec![], Type::Any, false);
        self.define_builtin_with_alias("snif_parse_ex", vec![Type::String], Type::Any, false);
        self.define_builtin_with_alias("snif_type", vec![Type::Any], Type::String, false);
        self.define_builtin_with_alias("snif_arr_len", vec![Type::Any], Type::Float, false);
        self.define_builtin_with_alias("snif_arr_get", vec![Type::Any, Type::Float], Type::Any, false);
        self.define_builtin_with_alias("snif_arr_set", vec![Type::Any, Type::Float, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("snif_arr_push", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("snif_path_get", vec![Type::Any, Type::String], Type::Any, false);

        // Auth (used by blaze_auth)
        self.define_builtin_with_alias("auth_random_hex", vec![Type::Float], Type::String, false);
        self.define_builtin_with_alias("auth_now", vec![], Type::Float, false);
        self.define_builtin_with_alias("auth_const_time_eq", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("auth_hash_password", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("auth_verify_password", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("auth_session_id", vec![], Type::String, false);
        self.define_builtin_with_alias("auth_csrf_token", vec![], Type::String, false);
        self.define_builtin_with_alias("auth_cookie_kv", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin_with_alias("auth_cookie_session", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("auth_cookie_delete", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("auth_bearer_header", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("auth_ok", vec![], Type::Bool, false);
        self.define_builtin_with_alias("auth_fail", vec![], Type::Bool, false);
        self.define_builtin_with_alias("auth_version", vec![], Type::String, false);

        // SFS (filesystem)
        self.define_builtin_with_alias("sfs_read", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("sfs_write", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_append", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_write_mb", vec![Type::String, Type::Float], Type::Float, false);
        self.define_builtin_with_alias("sfs_count_bytes", vec![Type::String], Type::Float, false);
        self.define_builtin_with_alias("sfs_delete", vec![Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_exists", vec![Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_copy", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_move", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_mkdir", vec![Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_is_file", vec![Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_is_dir", vec![Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sfs_listdir", vec![Type::String], Type::Any, false);
        self.define_builtin_with_alias("sfs_bench_create_small_files", vec![Type::String, Type::Float, Type::Float], Type::Float, false);
        self.define_builtin_with_alias("sfs_bench_count_entries", vec![Type::String], Type::Float, false);
        self.define_builtin_with_alias("sfs_bench_delete_small_files", vec![Type::String, Type::Float], Type::Float, false);
        self.define_builtin_with_alias("sfs_size", vec![Type::String], Type::Float, false);
        self.define_builtin_with_alias("sfs_mtime", vec![Type::String], Type::Float, false);
        self.define_builtin_with_alias("sfs_rmdir", vec![Type::String], Type::Bool, false);
        
        // Blaze (Web Server)
        self.define_builtin_with_alias("blaze_run", vec![Type::Float, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("blaze_qs_get", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin_with_alias("blaze_cookie_get", vec![Type::String, Type::String], Type::String, false);
        
        // SQLite
        self.define_builtin_with_alias("sqlite_open", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("sqlite_close", vec![Type::String], Type::Void, false);
        self.define_builtin_with_alias("sqlite_exec", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("sqlite_query", vec![Type::String, Type::String], Type::List, false);

        // OS
        self.define_builtin_with_alias("os_platform", vec![], Type::String, false);
        self.define_builtin_with_alias("os_arch", vec![], Type::String, false);
        self.define_builtin_with_alias("os_getenv", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("os_setenv", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("os_random_hex", vec![Type::Float], Type::String, false);

        // Misc "s_*" core helpers (runtime)
        self.define_builtin_with_alias("s_time", vec![], Type::Float, false);
        self.define_builtin_with_alias("s_sleep", vec![Type::Float], Type::Void, false);

        // Path helpers
        self.define_builtin_with_alias("path_basename", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("path_dirname", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("path_extname", vec![Type::String], Type::String, false);
        self.define_builtin_with_alias("path_join", vec![Type::String, Type::String], Type::String, false);

        // GUI (GTK runtime surface via `import "snask_gtk"`)
        self.define_builtin_with_alias("gui_init", vec![], Type::Bool, false);
        self.define_builtin_with_alias("gui_run", vec![], Type::Void, false);
        self.define_builtin_with_alias("gui_quit", vec![], Type::Void, false);
        self.define_builtin_with_alias("gui_window", vec![Type::String, Type::Float, Type::Float], Type::Any, false);
        self.define_builtin_with_alias("gui_set_title", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_set_resizable", vec![Type::Any, Type::Bool], Type::Bool, false);
        self.define_builtin_with_alias("gui_autosize", vec![Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_vbox", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_hbox", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_scrolled", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_eventbox", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_flowbox", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_flow_add", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_frame", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_set_margin", vec![Type::Any, Type::Float], Type::Bool, false);
        self.define_builtin_with_alias("gui_icon", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_listbox", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_list_add_text", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_on_select_ctx", vec![Type::Any, Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_set_child", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_add", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_add_expand", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_label", vec![Type::String], Type::Any, false);
        self.define_builtin_with_alias("gui_entry", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_textview", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_set_placeholder", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_set_editable", vec![Type::Any, Type::Bool], Type::Bool, false);
        self.define_builtin_with_alias("gui_button", vec![Type::String], Type::Any, false);
        self.define_builtin_with_alias("gui_set_enabled", vec![Type::Any, Type::Bool], Type::Bool, false);
        self.define_builtin_with_alias("gui_set_visible", vec![Type::Any, Type::Bool], Type::Bool, false);
        self.define_builtin_with_alias("gui_show_all", vec![Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_set_text", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_get_text", vec![Type::Any], Type::String, false);
        self.define_builtin_with_alias("gui_on_click", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_on_click_ctx", vec![Type::Any, Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_on_tap_ctx", vec![Type::Any, Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin_with_alias("gui_separator_h", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_separator_v", vec![], Type::Any, false);
        self.define_builtin_with_alias("gui_css", vec![Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_add_class", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_msg_info", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin_with_alias("gui_msg_error", vec![Type::String, Type::String], Type::Bool, false);
    }

    fn define_module_as_any(&mut self, name: &str) {
        self.symbol_table.define(SemanticSymbol {
            name: name.to_string(),
            symbol_type: Type::Any,
            kind: SemanticSymbolKind::Immutable,
            is_variadic: false,
        });
    }

    fn define_builtin(&mut self, name: &str, params: Vec<Type>, return_type: Type, is_variadic: bool) {
        let symbol = SemanticSymbol {
            name: name.to_string(),
            symbol_type: Type::Function(params, Box::new(return_type)),
            kind: SemanticSymbolKind::Function,
            is_variadic,
        };
        self.symbol_table.define(symbol);
    }

    fn define_builtin_with_alias(&mut self, name: &str, params: Vec<Type>, return_type: Type, is_variadic: bool) {
        self.define_builtin(name, params.clone(), return_type.clone(), is_variadic);
        self.define_builtin(&format!("__{}", name), params, return_type, is_variadic);
    }

    fn define_constant(&mut self, name: &str, const_type: Type) {
        let symbol = SemanticSymbol {
            name: name.to_string(),
            symbol_type: const_type,
            kind: SemanticSymbolKind::Constant,
            is_variadic: false,
        };
        self.symbol_table.define(symbol);
    }

    pub fn analyze(&mut self, program: &Program) {
        self.register_classes(program);
        for statement in program {
            self.analyze_statement(statement);
        }
    }

    fn register_classes(&mut self, program: &Program) {
        for statement in program {
            if let StmtKind::ClassDeclaration(class) = &statement.kind {
                self.classes.insert(class.name.clone(), class.clone());
                self.symbol_table.define(SemanticSymbol {
                    name: class.name.clone(),
                    symbol_type: Type::User(class.name.clone()),
                    kind: SemanticSymbolKind::Immutable,
                    is_variadic: false,
                });
            }
        }
    }

    fn property_decl_type(&mut self, decl: &VarDecl) -> Type {
        if let Some(var_type) = &decl.var_type {
            return var_type.clone();
        }

        self.type_check_expression(&decl.value).unwrap_or(Type::Any)
    }

    fn merge_inferred_types(left: Type, right: Type) -> Type {
        if left == right {
            return left;
        }
        if left == Type::Any {
            return right;
        }
        if right == Type::Any {
            return left;
        }
        if left.is_numeric() && right.is_numeric() {
            if left == Type::Float || right == Type::Float {
                return Type::Float;
            }
            return Type::Int;
        }
        Type::Any
    }

    fn method_type(method: &FuncDecl) -> Type {
        let params = method.params.iter().map(|(_, ty)| ty.clone()).collect();
        Type::Function(params, Box::new(method.return_type.clone().unwrap_or(Type::Any)))
    }

    fn class_member_type(&mut self, class_name: &str, member: &str) -> Option<Type> {
        self.class_member_type_inner(class_name, member, &mut Vec::new())
    }

    fn class_member_type_inner(&mut self, class_name: &str, member: &str, visited: &mut Vec<String>) -> Option<Type> {
        if visited.iter().any(|name| name == class_name) {
            return None;
        }
        visited.push(class_name.to_string());

        let class = self.classes.get(class_name).cloned()?;

        if let Some(prop) = class.properties.iter().find(|p| p.name == member) {
            return Some(self.property_decl_type(prop));
        }

        if let Some(method) = class.methods.iter().find(|m| m.name == member) {
            return Some(Self::method_type(method));
        }

        if let Some(parent_name) = &class.parent {
            return self.class_member_type_inner(parent_name, member, visited);
        }

        None
    }

    fn validate_type_exists(&mut self, ty: &Type, span: &Span) {
        match ty {
            Type::User(name) => {
                if !self.classes.contains_key(name) {
                    self.errors.push(self.mk_unknown_type(name.clone(), span.clone()));
                }
            }
            Type::ListOf(inner) => self.validate_type_exists(inner, span),
            Type::DictOf(key, value) => {
                self.validate_type_exists(key, span);
                self.validate_type_exists(value, span);
            }
            Type::Function(params, ret) => {
                for param in params {
                    self.validate_type_exists(param, span);
                }
                self.validate_type_exists(ret, span);
            }
            _ => {}
        }
    }

    fn stmt_guarantees_return(stmt: &Stmt) -> bool {
        match &stmt.kind {
            StmtKind::Return(_) => true,
            StmtKind::Conditional(cond) => {
                let if_returns = cond.if_block.body.iter().any(Self::stmt_guarantees_return);
                let elif_returns = cond.elif_blocks.iter().all(|b| b.body.iter().any(Self::stmt_guarantees_return));
                let else_returns = cond
                    .else_block
                    .as_ref()
                    .map(|body| body.iter().any(Self::stmt_guarantees_return))
                    .unwrap_or(false);
                if_returns && elif_returns && else_returns
            }
            StmtKind::Scope { body, .. } | StmtKind::Zone { body, .. } | StmtKind::UnsafeBlock(body) => {
                body.iter().any(Self::stmt_guarantees_return)
            }
            _ => false,
        }
    }

    fn body_guarantees_return(body: &[Stmt]) -> bool {
        body.iter().any(Self::stmt_guarantees_return)
    }

    fn analyze_statement(&mut self, statement: &Stmt) {
        match &statement.kind {
            StmtKind::Import(lib) => {
                if self.tiny_mode {
                    let disallowed = lib == "gui" || lib == "sqlite" || lib == "snask_skia" || lib == "skia";
                    if disallowed {
                        self.errors.push(
                            SemanticError::new(
                                SemanticErrorKind::TinyDisallowedLib { lib: lib.clone() },
                                statement.span.clone(),
                            )
                            .with_help("Recompile without `--tiny`, or remove this import.".to_string())
                            .with_note("Tiny mode is intended for small CLI/tools and uses a minimal runtime.".to_string()),
                        );
                    }
                }
            }
            StmtKind::FromImport { .. } => {
                // `from ... import module` is file-level; tiny restrictions apply to `import "lib"` libs only.
            }
            StmtKind::VarDeclaration(decl) => self.analyze_var_decl(decl, SemanticSymbolKind::Immutable, statement.span.clone()),
            StmtKind::MutDeclaration(decl) => self.analyze_var_decl(&decl.to_var_decl(), SemanticSymbolKind::Mutable, statement.span.clone()),
            StmtKind::ConstDeclaration(decl) => self.analyze_var_decl(&decl.to_var_decl(), SemanticSymbolKind::Constant, statement.span.clone()),
            StmtKind::Input { name, var_type } => {
                self.validate_type_exists(var_type, &statement.span);
                let symbol = SemanticSymbol {
                    name: name.clone(),
                    symbol_type: var_type.clone(),
                    kind: SemanticSymbolKind::Mutable,
                    is_variadic: false,
                };
                if !self.symbol_table.define(symbol) {
                    self.errors.push(SemanticError::new(
                        SemanticErrorKind::VariableAlreadyDeclared(name.clone()),
                        statement.span.clone(),
                    ));
                }
            }
            StmtKind::VarAssignment(var_set) => {
                let expr_type = match self.type_check_expression(&var_set.value) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };

                if let Some(symbol) = self.symbol_table.lookup(&var_set.name) {
                    if symbol.kind == SemanticSymbolKind::Constant || symbol.kind == SemanticSymbolKind::Immutable {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::ImmutableAssignment(var_set.name.clone()),
                            statement.span.clone(),
                        ));
                    }
                    if !self.is_compatible(&symbol.symbol_type, &expr_type) && expr_type != Type::Any {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::TypeMismatch {
                                expected: symbol.symbol_type.clone(),
                                found: expr_type,
                            },
                            statement.span.clone(),
                        ));
                    }
                } else {
                    self.errors.push(self.mk_variable_not_found(
                        var_set.name.clone(),
                        statement.span.clone(),
                    ));
                }
            }
            StmtKind::PropertyAssignment(p) => {
                let target_type = match self.type_check_expression(&p.target) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };
                let value_type = match self.type_check_expression(&p.value) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };

                match target_type {
                    Type::Any => {}
                    Type::User(class_name) => {
                        if let Some(expected_type) = self.class_member_type(&class_name, &p.property) {
                            if !self.is_compatible(&expected_type, &value_type) && value_type != Type::Any {
                                self.errors.push(SemanticError::new(
                                    SemanticErrorKind::TypeMismatch {
                                        expected: expected_type,
                                        found: value_type,
                                    },
                                    statement.span.clone(),
                                ));
                            }
                        } else {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::PropertyNotFound(p.property.clone()),
                                statement.span.clone(),
                            ));
                        }
                    }
                    other => {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::InvalidOperation {
                                op: "property assignment".to_string(),
                                type1: other,
                                type2: None,
                            },
                            statement.span.clone(),
                        ));
                    }
                }
            }
            StmtKind::IndexAssignment(i) => {
                let target_type = match self.type_check_expression(&i.target) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };
                let index_type = match self.type_check_expression(&i.index) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };
                let value_type = match self.type_check_expression(&i.value) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };

                match target_type {
                    Type::Any => {}
                    Type::List => {
                        if !index_type.is_numeric() && index_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::InvalidIndexType(index_type),
                                statement.span.clone(),
                            ));
                        }
                    }
                    Type::ListOf(element_type) => {
                        if !index_type.is_numeric() && index_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::InvalidIndexType(index_type),
                                statement.span.clone(),
                            ));
                        }
                        if !self.is_compatible(&element_type, &value_type) && value_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (*element_type).clone(),
                                    found: value_type,
                                },
                                statement.span.clone(),
                            ));
                        }
                    }
                    Type::Dict => {}
                    Type::DictOf(key_type, stored_value_type) => {
                        if !self.is_compatible(&key_type, &index_type) && index_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (*key_type).clone(),
                                    found: index_type,
                                },
                                i.index.span.clone(),
                            ));
                        }
                        if !self.is_compatible(&stored_value_type, &value_type) && value_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (*stored_value_type).clone(),
                                    found: value_type,
                                },
                                i.value.span.clone(),
                            ));
                        }
                    }
                    other => {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::IndexAccessOnNonIndexable(other),
                            statement.span.clone(),
                        ));
                    }
                }
            }
            StmtKind::FuncDeclaration(func_decl) => {
                for (_, param_type) in &func_decl.params {
                    self.validate_type_exists(param_type, &statement.span);
                }
                if let Some(return_type) = &func_decl.return_type {
                    self.validate_type_exists(return_type, &statement.span);
                }

                let params_types: Vec<Type> = func_decl.params.iter().map(|p| p.1.clone()).collect();
                let func_symbol = SemanticSymbol {
                    name: func_decl.name.clone(),
                    symbol_type: Type::Function(params_types, Box::new(func_decl.return_type.clone().unwrap_or(Type::Any))),
                    kind: SemanticSymbolKind::Function,
                    is_variadic: false,
                };
                if !self.symbol_table.define(func_symbol) {
                    self.errors.push(SemanticError::new(
                        SemanticErrorKind::FunctionAlreadyDeclared(func_decl.name.clone()),
                        statement.span.clone(),
                    ));
                }

                self.symbol_table.enter_scope();
                let prev_return_type = self.current_function_return_type.clone();
                self.current_function_return_type = Some(func_decl.return_type.clone().unwrap_or(Type::Any));

                for (param_name, param_type) in &func_decl.params {
                    let param_symbol = SemanticSymbol {
                        name: param_name.clone(),
                        symbol_type: param_type.clone(),
                        kind: SemanticSymbolKind::Parameter,
                        is_variadic: false,
                    };
                    self.symbol_table.define(param_symbol);
                }

                for stmt in &func_decl.body {
                    self.analyze_statement(stmt);
                }

                if let Some(return_type) = &func_decl.return_type {
                    if *return_type != Type::Void
                        && *return_type != Type::Any
                        && !Self::body_guarantees_return(&func_decl.body)
                    {
                        self.errors.push(
                            SemanticError::new(
                                SemanticErrorKind::MissingReturn {
                                    function: func_decl.name.clone(),
                                    expected: return_type.clone(),
                                },
                                statement.span.clone(),
                            )
                            .with_help("Add an explicit `return ...` on every control-flow path, or change the return type.".to_string()),
                        );
                    }
                }

                self.current_function_return_type = prev_return_type;
                self.symbol_table.exit_scope();
            }
            StmtKind::Return(expr) => {
                let return_type = match self.type_check_expression(expr) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };

                match &self.current_function_return_type {
                    Some(expected_type) => {
                        if !self.is_compatible(expected_type, &return_type) {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: expected_type.clone(),
                                    found: return_type,
                                },
                                statement.span.clone(),
                            ));
                        }
                    }
                    None => self.errors.push(SemanticError::new(
                        SemanticErrorKind::ReturnOutsideFunction,
                        statement.span.clone(),
                    )),
                }
            }
            StmtKind::Conditional(cond) => self.analyze_conditional(cond),
            StmtKind::Loop(loop_stmt) => self.analyze_loop(loop_stmt),
            StmtKind::ListDeclaration(decl) => {
                let var_decl = VarDecl {
                    name: decl.name.clone(),
                    var_type: decl.var_type.clone(),
                    value: decl.value.clone(),
                };
                self.analyze_var_decl(&var_decl, SemanticSymbolKind::Immutable, statement.span.clone());
            }
            StmtKind::DictDeclaration(decl) => {
                let var_decl = VarDecl {
                    name: decl.name.clone(),
                    var_type: decl.var_type.clone(),
                    value: decl.value.clone(),
                };
                self.analyze_var_decl(&var_decl, SemanticSymbolKind::Immutable, statement.span.clone());
            }
            StmtKind::ListPush(push) => {
                if let Some(symbol) = self.symbol_table.lookup(&push.name) {
                    let symbol_type = symbol.symbol_type.clone();
                    if !symbol_type.is_list_like() {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::InvalidOperation {
                                op: "list_push".to_string(),
                                type1: symbol_type.clone(),
                                type2: None,
                            },
                            statement.span.clone(),
                        ));
                    }
                    let value_type = match self.type_check_expression(&push.value) {
                        Ok(t) => t,
                        Err(e) => {
                            self.errors.push(e);
                            Type::Any
                        }
                    };
                    if let Type::ListOf(elem_type) = &symbol_type {
                        if !self.is_compatible(elem_type, &value_type) && value_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (**elem_type).clone(),
                                    found: value_type,
                                },
                                statement.span.clone(),
                            ));
                        }
                    }
                } else {
                    self.errors.push(self.mk_variable_not_found(push.name.clone(), statement.span.clone()));
                }
            }
            StmtKind::DictSet(set) => {
                if let Some(symbol) = self.symbol_table.lookup(&set.name) {
                    let symbol_type = symbol.symbol_type.clone();
                    if !symbol_type.is_dict_like() {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::InvalidOperation {
                                op: "dict_set".to_string(),
                                type1: symbol_type.clone(),
                                type2: None,
                            },
                            statement.span.clone(),
                        ));
                    }
                    let key_type = match self.type_check_expression(&set.key) {
                        Ok(t) => t,
                        Err(e) => {
                            self.errors.push(e);
                            Type::Any
                        }
                    };
                    let value_type = match self.type_check_expression(&set.value) {
                        Ok(t) => t,
                        Err(e) => {
                            self.errors.push(e);
                            Type::Any
                        }
                    };
                    if let Type::DictOf(expected_key, expected_value) = &symbol_type {
                        if !self.is_compatible(expected_key, &key_type) && key_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (**expected_key).clone(),
                                    found: key_type,
                                },
                                set.key.span.clone(),
                            ));
                        }
                        if !self.is_compatible(expected_value, &value_type) && value_type != Type::Any {
                            self.errors.push(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (**expected_value).clone(),
                                    found: value_type,
                                },
                                set.value.span.clone(),
                            ));
                        }
                    }
                } else {
                    self.errors.push(self.mk_variable_not_found(set.name.clone(), statement.span.clone()));
                }
            }
            StmtKind::Print(expressions) => {
                for expr in expressions {
                    if let Err(e) = self.type_check_expression(expr) {
                        self.errors.push(e);
                    }
                }
            }
            StmtKind::Expression(expr) | StmtKind::FuncCall(expr) => {
                if let Err(e) = self.type_check_expression(expr) {
                    self.errors.push(e);
                }
            }
            StmtKind::ClassDeclaration(class) => {
                self.symbol_table.enter_scope();
                let prev_class = self.current_class.clone();
                self.current_class = Some(class.name.clone());
                self.symbol_table.define(SemanticSymbol {
                    name: "self".to_string(),
                    symbol_type: Type::User(class.name.clone()),
                    kind: SemanticSymbolKind::Immutable,
                    is_variadic: false,
                });
                for method in &class.methods {
                    self.analyze_statement(&Stmt::with_span(StmtKind::FuncDeclaration(method.clone()), statement.loc.clone(), statement.span.clone()));
                }
                self.current_class = prev_class;
                self.symbol_table.exit_scope();
            }
            StmtKind::UnsafeBlock(body) => {
                for s in body { self.analyze_statement(s); }
            }
            StmtKind::Promote { .. } => {}
            StmtKind::Scope { body, .. } => {
                self.symbol_table.enter_scope();
                for s in body { self.analyze_statement(s); }
                self.symbol_table.exit_scope();
            }
            StmtKind::Zone { body, .. } => {
                self.symbol_table.enter_scope();
                for s in body { self.analyze_statement(s); }
                self.symbol_table.exit_scope();
            }
            StmtKind::Entangle { .. } => {}
        }
    }

    fn analyze_var_decl(&mut self, decl: &VarDecl, kind: SemanticSymbolKind, span: Span) {
        if let Some(expected_type) = &decl.var_type {
            self.validate_type_exists(expected_type, &span);
        }

        let expr_type = match self.type_check_expression(&decl.value) {
            Ok(t) => t,
            Err(e) => {
                self.errors.push(e);
                Type::Any
            }
        };

        let final_type = if let Some(ref expected_type) = decl.var_type {
            if !self.is_compatible(expected_type, &expr_type) && expr_type != Type::Any {
                self.errors.push(SemanticError::new(
                    SemanticErrorKind::TypeMismatch {
                        expected: expected_type.clone(),
                        found: expr_type,
                    },
                    span.clone(),
                ));
            }
            expected_type.clone()
        } else {
            expr_type
        };

        let symbol = SemanticSymbol {
            name: decl.name.clone(),
            symbol_type: final_type,
            kind,
            is_variadic: false,
        };

        if !self.symbol_table.define(symbol) {
            self.errors.push(SemanticError::new(
                SemanticErrorKind::VariableAlreadyDeclared(decl.name.clone()),
                span,
            ));
        }
    }

    fn analyze_conditional(&mut self, cond: &ConditionalStmt) {
        if let Err(e) = self.check_condition(&cond.if_block.condition) { self.errors.push(e); }
        self.symbol_table.enter_scope();
        for stmt in &cond.if_block.body { self.analyze_statement(stmt); }
        self.symbol_table.exit_scope();

        for elif in &cond.elif_blocks {
            if let Err(e) = self.check_condition(&elif.condition) { self.errors.push(e); }
            self.symbol_table.enter_scope();
            for stmt in &elif.body { self.analyze_statement(stmt); }
            self.symbol_table.exit_scope();
        }

        if let Some(else_body) = &cond.else_block {
            self.symbol_table.enter_scope();
            for stmt in else_body { self.analyze_statement(stmt); }
            self.symbol_table.exit_scope();
        }
    }

    fn analyze_loop(&mut self, loop_stmt: &LoopStmt) {
        self.symbol_table.enter_scope();
        match loop_stmt {
            LoopStmt::While { condition, body } => {
                if let Err(e) = self.check_condition(condition) { self.errors.push(e); }
                for stmt in body { self.analyze_statement(stmt); }
            }
            LoopStmt::For { iterator, iterable, body } => {
                let iterable_type = match self.type_check_expression(iterable) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        Type::Any
                    }
                };

                let iterator_type = match iterable_type {
                    Type::ListOf(element_type) => *element_type,
                    Type::List => Type::Any,
                    Type::DictOf(key_type, _value_type) => *key_type,
                    Type::Dict => Type::Any,
                    Type::String => Type::String,
                    Type::Any => Type::Any,
                    _ => {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::InvalidOperation {
                                op: "for-in".to_string(),
                                type1: iterable_type,
                                type2: None,
                            },
                            iterable.span.clone(),
                        ));
                        Type::Void
                    }
                };

                let symbol = SemanticSymbol {
                    name: iterator.clone(),
                    symbol_type: iterator_type,
                    kind: SemanticSymbolKind::Immutable,
                    is_variadic: false,
                };
                self.symbol_table.define(symbol);

                for stmt in body { self.analyze_statement(stmt); }
            }
        }
        self.symbol_table.exit_scope();
    }

    fn check_condition(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        let expr_type = self.type_check_expression(expr)?;
        if expr_type != Type::Bool && !expr_type.is_numeric() && expr_type != Type::Any {
            return Err(SemanticError::new(
                SemanticErrorKind::TypeMismatch {
                    expected: Type::Bool,
                    found: expr_type,
                },
                expr.span.clone(),
            ));
        }
        Ok(())
    }

    fn is_compatible(&self, expected: &Type, found: &Type) -> bool {
        if expected == found { return true; }
        if matches!(expected, Type::Float | Type::Int | Type::I64 | Type::I32 | Type::U8) &&
           matches!(found, Type::Float | Type::Int | Type::I64 | Type::I32 | Type::U8) {
               return true;
        }
        if let (Type::User(expected_name), Type::User(found_name)) = (expected, found) {
            return expected_name == found_name;
        }
        if matches!(expected, Type::List) && matches!(found, Type::ListOf(_)) {
            return true;
        }
        if matches!(expected, Type::Dict) && matches!(found, Type::DictOf(_, _)) {
            return true;
        }
        if let (Type::ListOf(expected_elem), Type::ListOf(found_elem)) = (expected, found) {
            return self.is_compatible(expected_elem, found_elem);
        }
        if let (Type::DictOf(expected_key, expected_value), Type::DictOf(found_key, found_value)) = (expected, found) {
            return self.is_compatible(expected_key, found_key) && self.is_compatible(expected_value, found_value);
        }
        if *expected == Type::Any || *found == Type::Any { return true; }
        false
    }

    fn type_check_expression(&mut self, expression: &Expr) -> Result<Type, SemanticError> {
        match &expression.kind {
            ExprKind::Variable(name) => {
                if let Some(symbol) = self.symbol_table.lookup(name) {
                    Ok(symbol.symbol_type.clone())
                } else {
                    Err(self.mk_variable_not_found(name.clone(), expression.span.clone()))
                }
            }
            ExprKind::Literal(value) => match value {
                LiteralValue::Number(n) => if n.fract() == 0.0 { Ok(Type::Int) } else { Ok(Type::Float) },
                LiteralValue::String(_) => Ok(Type::String),
                LiteralValue::Boolean(_) => Ok(Type::Bool),
                LiteralValue::List(items) => {
                    let mut item_type = Type::Any;
                    for item in items {
                        let current = self.type_check_expression(item)?;
                        item_type = Self::merge_inferred_types(item_type, current);
                    }
                    Ok(Type::ListOf(Box::new(item_type)))
                }
                LiteralValue::Dict(pairs) => {
                    let mut key_type = Type::Any;
                    let mut value_type = Type::Any;
                    for (key, value) in pairs {
                        let current_key_type = self.type_check_expression(key)?;
                        let current_value_type = self.type_check_expression(value)?;
                        key_type = Self::merge_inferred_types(key_type, current_key_type);
                        value_type = Self::merge_inferred_types(value_type, current_value_type);
                    }
                    Ok(Type::DictOf(Box::new(key_type), Box::new(value_type)))
                }
                LiteralValue::Nil => Ok(Type::Any),
            },
            ExprKind::Binary { left, op, right } => {
                let left_type = self.type_check_expression(left)?;
                let right_type = self.type_check_expression(right)?;

                match op {
                    BinaryOp::Add => {
                        if left_type == Type::String || right_type == Type::String {
                            Ok(Type::String)
                        } else if left_type == Type::Any || right_type == Type::Any {
                            Ok(Type::Any)
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type == Type::Float || right_type == Type::Float { Ok(Type::Float) } else { Ok(Type::Int) }
                        } else {
                            Err(SemanticError::new(
                                SemanticErrorKind::InvalidOperation {
                                    op: format!("{:?}", op),
                                    type1: left_type,
                                    type2: Some(right_type),
                                },
                                expression.span.clone(),
                            ))
                        }
                    }
                    BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::IntDivide => {
                        if left_type == Type::Any || right_type == Type::Any {
                            Ok(Type::Any)
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type == Type::Float || right_type == Type::Float { Ok(Type::Float) } else { Ok(Type::Int) }
                        } else {
                            Err(SemanticError::new(
                                SemanticErrorKind::InvalidOperation {
                                    op: format!("{:?}", op),
                                    type1: left_type,
                                    type2: Some(right_type),
                                },
                                expression.span.clone(),
                            ))
                        }
                    }
                    _ => {
                         if self.is_compatible(&left_type, &right_type) || self.is_compatible(&right_type, &left_type) { Ok(Type::Bool) } else {
                             Err(SemanticError::new(
                                 SemanticErrorKind::InvalidOperation {
                                     op: format!("{:?}", op),
                                     type1: left_type,
                                     type2: Some(right_type),
                                 },
                                 expression.span.clone(),
                             ))
                         }
                    }
                }
            }
            ExprKind::Unary { op, expr } => {
                let expr_type = self.type_check_expression(expr)?;
                match op {
                    UnaryOp::Negative => {
                        if expr_type.is_numeric() { Ok(expr_type) } else {
                            Err(SemanticError::new(SemanticErrorKind::InvalidOperation { op: "Negative".to_string(), type1: expr_type, type2: None }, expression.span.clone()))
                        }
                    }
                    UnaryOp::Not => Ok(Type::Bool),
                }
            }
            ExprKind::FunctionCall { callee, args } => {
                /* Desativado temporariamente para permitir que libs funcionem */
                /* if let ExprKind::Variable(name) = &callee.kind {
                    if !name.starts_with("__") && is_library_native(name) {
                        return Err(SemanticError::new(SemanticErrorKind::RestrictedNativeFunction { name: name.clone(), help: library_native_help(name) }, expression.span.clone()));
                    }
                } */

                let callee_symbol = if let ExprKind::Variable(name) = &callee.kind {
                    self.symbol_table.lookup(name).cloned()
                } else {
                    None
                };
                let callee_type = self.type_check_expression(callee)?;
                if let Type::Function(param_types, return_type) = callee_type {
                    let is_variadic = callee_symbol.as_ref().map(|s| s.is_variadic).unwrap_or(false);
                    if (!is_variadic && args.len() != param_types.len())
                        || (is_variadic && args.len() < param_types.len())
                    {
                        return Err(SemanticError::new(SemanticErrorKind::WrongNumberOfArguments { expected: param_types.len(), found: args.len() }, expression.span.clone()));
                    }
                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.type_check_expression(arg)?;
                        let expected_type = if i < param_types.len() {
                            param_types[i].clone()
                        } else if is_variadic {
                            param_types.last().cloned().unwrap_or(Type::Any)
                        } else {
                            Type::Any
                        };
                        if !self.is_compatible(&expected_type, &arg_type) {
                            return Err(SemanticError::new(SemanticErrorKind::TypeMismatch { expected: expected_type, found: arg_type }, arg.span.clone()));
                        }
                    }
                    Ok(*return_type)
                } else if callee_type == Type::Any {
                    for arg in args { let _ = self.type_check_expression(arg)?; }
                    Ok(Type::Any)
                } else {
                    Err(SemanticError::new(SemanticErrorKind::NotCallable(callee_type), expression.span.clone()))
                }
            }
            ExprKind::PropertyAccess { target, property } => {
                let target_type = self.type_check_expression(target)?;
                match target_type {
                    Type::Any => Ok(Type::Any),
                    Type::User(class_name) => self.class_member_type(&class_name, property).ok_or_else(|| {
                        SemanticError::new(SemanticErrorKind::PropertyNotFound(property.clone()), expression.span.clone())
                    }),
                    _ => Err(SemanticError::new(SemanticErrorKind::PropertyNotFound(property.clone()), expression.span.clone())),
                }
            }
            ExprKind::IndexAccess { target, index } => {
                let target_type = self.type_check_expression(target)?;
                let index_type = self.type_check_expression(index)?;

                match target_type {
                    Type::Any => Ok(Type::Any),
                    Type::String => {
                        if !index_type.is_numeric() && index_type != Type::Any {
                            Err(SemanticError::new(SemanticErrorKind::InvalidIndexType(index_type), expression.span.clone()))
                        } else {
                            Ok(Type::String)
                        }
                    }
                    Type::List => {
                        if !index_type.is_numeric() && index_type != Type::Any {
                            Err(SemanticError::new(SemanticErrorKind::InvalidIndexType(index_type), expression.span.clone()))
                        } else {
                            Ok(Type::Any)
                        }
                    }
                    Type::ListOf(element_type) => {
                        if !index_type.is_numeric() && index_type != Type::Any {
                            Err(SemanticError::new(SemanticErrorKind::InvalidIndexType(index_type), expression.span.clone()))
                        } else {
                            Ok(*element_type)
                        }
                    }
                    Type::Dict => Ok(Type::Any),
                    Type::DictOf(key_type, value_type) => {
                        if !self.is_compatible(&key_type, &index_type) && index_type != Type::Any {
                            Err(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: (*key_type).clone(),
                                    found: index_type,
                                },
                                expression.span.clone(),
                            ))
                        } else {
                            Ok(*value_type)
                        }
                    }
                    other => Err(SemanticError::new(SemanticErrorKind::IndexAccessOnNonIndexable(other), expression.span.clone())),
                }
            }
            ExprKind::New { class, args, .. } => {
                if !self.classes.contains_key(class) {
                    return Err(self.mk_unknown_type(class.clone(), expression.span.clone()));
                }
                let init_method = self
                    .classes
                    .get(class)
                    .ok_or_else(|| self.mk_unknown_type(class.clone(), expression.span.clone()))?
                    .methods
                    .iter()
                    .find(|m| m.name == "init")
                    .cloned();

                if let Some(init) = init_method {
                    let expected_args = init.params.len();
                    if args.len() != expected_args {
                        return Err(SemanticError::new(
                            SemanticErrorKind::WrongNumberOfArguments {
                                expected: expected_args,
                                found: args.len(),
                            },
                            expression.span.clone(),
                        ));
                    }

                    for (arg, (_, expected_type)) in args.iter().zip(init.params.iter()) {
                        let arg_type = self.type_check_expression(arg)?;
                        if !self.is_compatible(expected_type, &arg_type) {
                            return Err(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: expected_type.clone(),
                                    found: arg_type,
                                },
                                arg.span.clone(),
                            ));
                        }
                    }
                } else {
                    for arg in args {
                        let _ = self.type_check_expression(arg)?;
                    }
                }

                Ok(Type::User(class.clone()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SemanticAnalyzer, SemanticErrorKind};
    use crate::parser::parse_program;

    fn analyze_source(src: &str) -> SemanticAnalyzer {
        let program = parse_program(src).expect("source should parse");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&program);
        analyzer
    }

    #[test]
    fn new_returns_nominal_user_type_and_type_annotation_accepts_class_name() {
        let analyzer = analyze_source(
            r#"
class Point
    let x: int = 0
    let y: int = 0

class main
    fun start()
        let p: Point = new Point()
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn property_access_on_user_type_resolves_declared_property() {
        let analyzer = analyze_source(
            r#"
class Point
    let x: int = 0

class main
    fun start()
        let p: Point = new Point()
        let value: int = p.x
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn property_access_reports_unknown_member_on_user_type() {
        let analyzer = analyze_source(
            r#"
class Point
    let x: int = 0

class main
    fun start()
        let p: Point = new Point()
        let value = p.z
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::PropertyNotFound(ref name) if name == "z")),
            "expected PropertyNotFound for z, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn inherited_property_access_resolves_from_parent_class() {
        let analyzer = analyze_source(
            r#"
class Animal
    let age: int = 0

class Dog extends Animal
    let name: str = "rex"

class main
    fun start()
        let d: Dog = new Dog()
        let age: int = d.age
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn property_assignment_checks_declared_property_type() {
        let analyzer = analyze_source(
            r#"
class Point
    mut x: int = 0

class main
    fun start()
        let p: Point = new Point()
        p.x = "oops"
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::TypeMismatch { .. })),
            "expected type mismatch on property assignment, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn inherited_property_assignment_is_checked_against_parent_type() {
        let analyzer = analyze_source(
            r#"
class Animal
    mut age: int = 0

class Dog extends Animal
    let name: str = "rex"

class main
    fun start()
        let d: Dog = new Dog()
        d.age = "old"
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::TypeMismatch { .. })),
            "expected type mismatch on inherited property assignment, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn list_index_access_uses_inferred_element_type() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let xs = [1, 2, 3]
        let first: int = xs[0]
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn for_in_list_uses_inferred_element_type() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let xs = [1, 2, 3]
        for item in xs
            let value: int = item
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn dict_index_access_uses_inferred_value_type() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let scores = { "alice": 10, "bob": 20 }
        let score: int = scores["alice"]
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn list_indexed_value_reports_type_mismatch_when_assigned_to_wrong_type() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let xs = [1, 2, 3]
        let wrong: str = xs[0]
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::TypeMismatch { .. })),
            "expected type mismatch for indexed list value, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn dict_set_checks_inferred_key_and_value_types() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let scores = { "alice": 10, "bob": 20 }
        scores["carol"] = "high"
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::TypeMismatch { .. })),
            "expected type mismatch for dict set, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn unknown_user_type_in_annotation_is_reported() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let x: MissingType = 1
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::UnknownType(ref name) if name == "MissingType")),
            "expected unknown type error, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn unknown_class_in_new_is_reported() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let x = new MissingType()
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::UnknownType(ref name) if name == "MissingType")),
            "expected unknown type for constructor, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn variadic_builtin_accepts_more_than_declared_minimum() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let value = format("x", "y", "z")
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn typed_function_without_return_is_reported() {
        let analyzer = analyze_source(
            r#"
fun meaning() : int
    let x = 1

class main
    fun start()
        let y = meaning()
"#,
        );

        assert!(
            analyzer
                .errors
                .iter()
                .any(|e| matches!(e.kind, SemanticErrorKind::MissingReturn { ref function, .. } if function == "meaning")),
            "expected missing return error, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn typed_function_with_if_else_returns_on_all_paths_is_accepted() {
        let analyzer = analyze_source(
            r#"
fun classify(x: int) : int
    if x > 0
        return 1
    else
        return 0

class main
    fun start()
        let y: int = classify(2)
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }
}
