use crate::ast::{Program, Stmt, StmtKind, Expr, ExprKind, VarDecl, BinaryOp, UnaryOp, LiteralValue, ConditionalStmt, LoopStmt};
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
        }
    }

    pub fn code(&self) -> &'static str {
        use SemanticErrorKind::*;
        match &self.kind {
            VariableAlreadyDeclared(_) => "SNASK-SEM-VAR-REDECL",
            VariableNotFound(_) => "SNASK-SEM-VAR-NOT-FOUND",
            FunctionAlreadyDeclared(_) => "SNASK-SEM-FUN-REDECL",
            FunctionNotFound(_) => "SNASK-SEM-FUN-NOT-FOUND",
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
        }
    }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SemanticSymbolTable,
    current_function_return_type: Option<Type>,
    pub errors: Vec<SemanticError>,
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
            errors: Vec::new(),
        };
        analyzer.register_stdlib();
        analyzer
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
        for statement in program {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Stmt) {
        match &statement.kind {
            StmtKind::VarDeclaration(decl) => self.analyze_var_decl(decl, SemanticSymbolKind::Immutable, statement.span.clone()),
            StmtKind::MutDeclaration(decl) => self.analyze_var_decl(&decl.to_var_decl(), SemanticSymbolKind::Mutable, statement.span.clone()),
            StmtKind::ConstDeclaration(decl) => self.analyze_var_decl(&decl.to_var_decl(), SemanticSymbolKind::Constant, statement.span.clone()),
            StmtKind::Input { name, var_type } => {
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
            StmtKind::FuncDeclaration(func_decl) => {
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
                    if symbol.symbol_type != Type::List {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::InvalidOperation {
                                op: "list_push".to_string(),
                                type1: symbol.symbol_type.clone(),
                                type2: None,
                            },
                            statement.span.clone(),
                        ));
                    }
                    let _ = self.type_check_expression(&push.value);
                } else {
                    self.errors.push(self.mk_variable_not_found(push.name.clone(), statement.span.clone()));
                }
            }
            StmtKind::DictSet(set) => {
                if let Some(symbol) = self.symbol_table.lookup(&set.name) {
                    if symbol.symbol_type != Type::Dict {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::InvalidOperation {
                                op: "dict_set".to_string(),
                                type1: symbol.symbol_type.clone(),
                                type2: None,
                            },
                            statement.span.clone(),
                        ));
                    }
                    let _ = self.type_check_expression(&set.key);
                    let _ = self.type_check_expression(&set.value);
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
                for method in &class.methods {
                    self.analyze_statement(&Stmt::with_span(StmtKind::FuncDeclaration(method.clone()), statement.loc.clone(), statement.span.clone()));
                }
                self.symbol_table.exit_scope();
            }
            _ => {}
        }
    }

    fn analyze_var_decl(&mut self, decl: &VarDecl, kind: SemanticSymbolKind, span: Span) {
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
                    Type::List => Type::Any,
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
                LiteralValue::List(_) => Ok(Type::List),
                LiteralValue::Dict(_) => Ok(Type::Dict),
                LiteralValue::Nil => Ok(Type::Any),
            },
            ExprKind::Binary { left, op, right } => {
                let left_type = self.type_check_expression(left)?;
                let right_type = self.type_check_expression(right)?;

                match op {
                    BinaryOp::Add => {
                        if left_type == Type::Any || right_type == Type::Any {
                            if left_type == Type::String || right_type == Type::String { Ok(Type::String) } else { Ok(Type::Any) }
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type == Type::Float || right_type == Type::Float { Ok(Type::Float) } else { Ok(Type::Int) }
                        } else if left_type == Type::String && right_type == Type::String {
                            Ok(Type::String)
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
                if let ExprKind::Variable(name) = &callee.kind {
                    if !name.starts_with("__") && is_library_native(name) {
                        return Err(SemanticError::new(SemanticErrorKind::RestrictedNativeFunction { name: name.clone(), help: library_native_help(name) }, expression.span.clone()));
                    }
                }

                let callee_type = self.type_check_expression(callee)?;
                if let Type::Function(param_types, return_type) = callee_type {
                    if args.len() != param_types.len() {
                        return Err(SemanticError::new(SemanticErrorKind::WrongNumberOfArguments { expected: param_types.len(), found: args.len() }, expression.span.clone()));
                    }
                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.type_check_expression(arg)?;
                        if !self.is_compatible(&param_types[i], &arg_type) {
                            return Err(SemanticError::new(SemanticErrorKind::TypeMismatch { expected: param_types[i].clone(), found: arg_type }, arg.span.clone()));
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
                    _ => Err(SemanticError::new(SemanticErrorKind::PropertyNotFound(property.clone()), expression.span.clone())),
                }
            }
            ExprKind::IndexAccess { target, index } => {
                let _ = self.type_check_expression(target)?;
                let _ = self.type_check_expression(index)?;
                Ok(Type::Any)
            }
        }
    }
}
