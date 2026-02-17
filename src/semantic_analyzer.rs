use crate::ast::{Program, Stmt, StmtKind, Expr, ExprKind, VarDecl, BinaryOp, UnaryOp, LiteralValue, ConditionalStmt, LoopStmt};
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
    pub is_variadic: bool, // Novo campo
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
}


#[derive(Debug)]
pub enum SemanticError {
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
}

pub struct SemanticAnalyzer {
    pub symbol_table: SemanticSymbolTable,
    current_function_return_type: Option<Type>,
    pub errors: Vec<SemanticError>,
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

    fn register_stdlib(&mut self) {
        // Define o módulo 'math' como um objeto de tipo 'Any'.
        // Isso é um atalho para evitar a necessidade de definir um tipo de dicionário complexo.
        // O verificador de tipos permitirá qualquer acesso a propriedades em 'math'.
        let math_symbol = SemanticSymbol {
            name: "math".to_string(),
            symbol_type: Type::Any,
            kind: SemanticSymbolKind::Immutable,
            is_variadic: false,
        };
        self.symbol_table.define(math_symbol);

        // Define outros módulos como Any
        let string_symbol = SemanticSymbol {
            name: "string".to_string(),
            symbol_type: Type::Any,
            kind: SemanticSymbolKind::Immutable,
            is_variadic: false,
        };
        self.symbol_table.define(string_symbol);

        let collections_symbol = SemanticSymbol {
            name: "collections".to_string(),
            symbol_type: Type::Any,
            kind: SemanticSymbolKind::Immutable,
            is_variadic: false,
        };
        self.symbol_table.define(collections_symbol);

        // Math Functions (top-level)
        self.define_builtin("abs", vec![Type::Float], Type::Float, false);
        self.define_builtin("floor", vec![Type::Float], Type::Float, false);
        self.define_builtin("ceil", vec![Type::Float], Type::Float, false);
        self.define_builtin("round", vec![Type::Float], Type::Float, false);
        self.define_builtin("pow", vec![Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("sqrt", vec![Type::Float], Type::Float, false);
        self.define_builtin("min", vec![], Type::Any, true); // Simplified, variadic
        self.define_builtin("max", vec![], Type::Any, true); // Simplified, variadic
        self.define_builtin("sin", vec![Type::Float], Type::Float, false);
        self.define_builtin("cos", vec![Type::Float], Type::Float, false);

        // Math Constants
        self.define_constant("PI", Type::Float);
        self.define_constant("E", Type::Float);
        self.define_constant("TAU", Type::Float);

        // String
        self.define_builtin("len", vec![Type::String], Type::Float, false);
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
        self.define_builtin("format", vec![Type::String, Type::Any, Type::Any], Type::String, true); // Variadic support // Basic support

        // Collections
        self.define_builtin("range", vec![Type::Float], Type::List, false); // Basic support
        self.define_builtin("sort", vec![Type::List], Type::List, false);
        self.define_builtin("reverse", vec![Type::List], Type::List, false);
        self.define_builtin("unique", vec![Type::List], Type::List, false);
        self.define_builtin("flatten", vec![Type::List], Type::List, false);
        // TODO: map, filter, reduce (need function type support in args)

        // Type checks
        self.define_builtin("is_nil", vec![Type::Any], Type::Bool, false);
        self.define_builtin("is_str", vec![Type::Any], Type::Bool, false);
        self.define_builtin("is_obj", vec![Type::Any], Type::Bool, false);

        // IO
        self.define_builtin("read_file", vec![Type::String], Type::String, false);
        self.define_builtin("write_file", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("append_file", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("exists", vec![Type::String], Type::Bool, false);
        self.define_builtin("delete", vec![Type::String], Type::Void, false);
        self.define_builtin("read_dir", vec![Type::String], Type::List, false);
        self.define_builtin("is_file", vec![Type::String], Type::Bool, false);
        self.define_builtin("is_dir", vec![Type::String], Type::Bool, false);
        self.define_builtin("create_dir", vec![Type::String], Type::Void, false);

        // HTTP
        self.define_builtin("http_get", vec![Type::String], Type::Dict, false);
        self.define_builtin("http_post", vec![Type::String, Type::String], Type::Void, false);

        // JSON
        self.define_builtin("json_parse", vec![Type::String], Type::Any, false);
        self.define_builtin("json_stringify", vec![Type::Any], Type::String, false);
        self.define_builtin("json_stringify_pretty", vec![Type::Any], Type::String, false);
        self.define_builtin("json_get", vec![Type::Any, Type::String], Type::Any, false);
        self.define_builtin("json_has", vec![Type::Any, Type::String], Type::Bool, false);
        self.define_builtin("json_len", vec![Type::Any], Type::Float, false);
        self.define_builtin("json_index", vec![Type::Any, Type::Float], Type::Any, false);
        self.define_builtin("json_set", vec![Type::Any, Type::String, Type::Any], Type::Bool, false);

        // Sjson
        self.define_builtin("sjson_new_object", vec![], Type::Any, false);
        self.define_builtin("sjson_new_array", vec![], Type::Any, false);
        self.define_builtin("sjson_type", vec![Type::Any], Type::String, false);
        self.define_builtin("sjson_arr_len", vec![Type::Any], Type::Float, false);
        self.define_builtin("sjson_arr_get", vec![Type::Any, Type::Float], Type::Any, false);
        self.define_builtin("sjson_arr_set", vec![Type::Any, Type::Float, Type::Any], Type::Bool, false);
        self.define_builtin("sjson_arr_push", vec![Type::Any, Type::Any], Type::Bool, false);
        self.define_builtin("sjson_path_get", vec![Type::Any, Type::String], Type::Any, false);
        self.define_builtin("json_parse_ex", vec![Type::String], Type::Any, false);

        // System
        self.define_builtin("time", vec![], Type::Float, false);
        self.define_builtin("sleep", vec![Type::Float], Type::Void, false);
        self.define_builtin("exit", vec![Type::Float], Type::Void, false);
        self.define_builtin("args", vec![], Type::List, false);
        self.define_builtin("env", vec![Type::String], Type::String, false);
        self.define_builtin("set_env", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("cwd", vec![], Type::String, false);
        self.define_builtin("platform", vec![], Type::String, false);
        self.define_builtin("arch", vec![], Type::String, false);
        self.define_builtin("os_cwd", vec![], Type::String, false);
        self.define_builtin("os_platform", vec![], Type::String, false);
        self.define_builtin("os_arch", vec![], Type::String, false);
        self.define_builtin("os_getenv", vec![Type::String], Type::String, false);
        self.define_builtin("os_setenv", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("os_random_hex", vec![Type::Float], Type::String, false);

        // Math - Novas funÃ§Ãµes
        self.define_builtin("mod", vec![Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("random", vec![], Type::Float, false);
        self.define_builtin("random_range", vec![Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("clamp", vec![Type::Float, Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("sign", vec![Type::Float], Type::Float, false);
        self.define_builtin("deg_to_rad", vec![Type::Float], Type::Float, false);
        self.define_builtin("rad_to_deg", vec![Type::Float], Type::Float, false);

        // File System (LLVM Native)
        self.define_builtin("sfs_read", vec![Type::String], Type::String, false);
        self.define_builtin("sfs_write", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("sfs_append", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("sfs_exists", vec![Type::String], Type::Bool, false);
        self.define_builtin("sfs_delete", vec![Type::String], Type::Bool, false);
        self.define_builtin("sfs_copy", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("sfs_move", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("sfs_mkdir", vec![Type::String], Type::Bool, false);
        self.define_builtin("sfs_is_file", vec![Type::String], Type::Bool, false);
        self.define_builtin("sfs_is_dir", vec![Type::String], Type::Bool, false);
        self.define_builtin("sfs_listdir", vec![Type::String], Type::Any, false);
        self.define_builtin("sfs_size", vec![Type::String], Type::Float, false);
        self.define_builtin("sfs_mtime", vec![Type::String], Type::Float, false);
        self.define_builtin("sfs_rmdir", vec![Type::String], Type::Bool, false);

        // Path helpers (LLVM Native)
        self.define_builtin("path_basename", vec![Type::String], Type::String, false);
        self.define_builtin("path_dirname", vec![Type::String], Type::String, false);
        self.define_builtin("path_extname", vec![Type::String], Type::String, false);
        self.define_builtin("path_join", vec![Type::String, Type::String], Type::String, false);

        // HTTP (LLVM Native)
        self.define_builtin("s_http_get", vec![Type::String], Type::String, false);
        self.define_builtin("s_http_post", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin("s_http_put", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin("s_http_delete", vec![Type::String], Type::String, false);
        self.define_builtin("s_http_patch", vec![Type::String, Type::String], Type::String, false);

        // Blaze Core
        self.define_builtin("blaze_run", vec![Type::Float, Type::Any], Type::Bool, false);
        self.define_builtin("blaze_qs_get", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin("blaze_cookie_get", vec![Type::String, Type::String], Type::String, false);

        // Auth natives (blaze_auth)
        self.define_builtin("auth_random_hex", vec![Type::Float], Type::String, false);
        self.define_builtin("auth_now", vec![], Type::Float, false);
        self.define_builtin("auth_const_time_eq", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("auth_hash_password", vec![Type::String], Type::String, false);
        self.define_builtin("auth_verify_password", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("auth_session_id", vec![], Type::String, false);
        self.define_builtin("auth_csrf_token", vec![], Type::String, false);
        self.define_builtin("auth_cookie_kv", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin("auth_cookie_session", vec![Type::String], Type::String, false);
        self.define_builtin("auth_cookie_delete", vec![Type::String], Type::String, false);
        self.define_builtin("auth_bearer_header", vec![Type::String], Type::String, false);
        self.define_builtin("auth_ok", vec![], Type::Bool, false);
        self.define_builtin("auth_fail", vec![], Type::Bool, false);
        self.define_builtin("auth_version", vec![], Type::String, false);

        // GUI (GTK) - MVP (handles are strings)
        self.define_builtin("gui_init", vec![], Type::Bool, false);
        self.define_builtin("gui_run", vec![], Type::Void, false);
        self.define_builtin("gui_quit", vec![], Type::Void, false);
        self.define_builtin("gui_window", vec![Type::String, Type::Float, Type::Float], Type::String, false);
        self.define_builtin("gui_set_title", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_set_resizable", vec![Type::String, Type::Bool], Type::Bool, false);
        self.define_builtin("gui_autosize", vec![Type::String], Type::Bool, false);
        self.define_builtin("gui_vbox", vec![], Type::String, false);
        self.define_builtin("gui_hbox", vec![], Type::String, false);
        self.define_builtin("gui_set_child", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_add", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_add_expand", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_label", vec![Type::String], Type::String, false);
        self.define_builtin("gui_entry", vec![], Type::String, false);
        self.define_builtin("gui_set_placeholder", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_set_editable", vec![Type::String, Type::Bool], Type::Bool, false);
        self.define_builtin("gui_button", vec![Type::String], Type::String, false);
        self.define_builtin("gui_set_enabled", vec![Type::String, Type::Bool], Type::Bool, false);
        self.define_builtin("gui_set_visible", vec![Type::String, Type::Bool], Type::Bool, false);
        self.define_builtin("gui_show_all", vec![Type::String], Type::Void, false);
        self.define_builtin("gui_set_text", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_get_text", vec![Type::String], Type::String, false);
        self.define_builtin("gui_on_click", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_on_click_ctx", vec![Type::String, Type::String, Type::String], Type::Bool, false);
        self.define_builtin("gui_separator_h", vec![], Type::String, false);
        self.define_builtin("gui_separator_v", vec![], Type::String, false);
        self.define_builtin("gui_msg_info", vec![Type::String, Type::String], Type::Void, false);
        self.define_builtin("gui_msg_error", vec![Type::String, Type::String], Type::Void, false);

        self.define_builtin("str_to_num", vec![Type::String], Type::Float, false);
        self.define_builtin("num_to_str", vec![Type::Float], Type::String, false);
        self.define_builtin("calc_eval", vec![Type::String], Type::Float, false);

        // SQLite (MVP)
        self.define_builtin("sqlite_open", vec![Type::String], Type::String, false);
        self.define_builtin("sqlite_close", vec![Type::String], Type::Bool, false);
        self.define_builtin("sqlite_exec", vec![Type::String, Type::String], Type::Bool, false);
        self.define_builtin("sqlite_query", vec![Type::String, Type::String], Type::Any, false);
        self.define_builtin("sqlite_prepare", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin("sqlite_finalize", vec![Type::String], Type::Bool, false);
        self.define_builtin("sqlite_reset", vec![Type::String], Type::Bool, false);
        self.define_builtin("sqlite_bind_text", vec![Type::String, Type::Float, Type::String], Type::Bool, false);
        self.define_builtin("sqlite_bind_num", vec![Type::String, Type::Float, Type::Float], Type::Bool, false);
        self.define_builtin("sqlite_bind_null", vec![Type::String, Type::Float], Type::Bool, false);
        self.define_builtin("sqlite_step", vec![Type::String], Type::Bool, false);
        self.define_builtin("sqlite_column", vec![Type::String, Type::Float], Type::Any, false);
        self.define_builtin("sqlite_column_count", vec![Type::String], Type::Float, false);
        self.define_builtin("sqlite_column_name", vec![Type::String, Type::Float], Type::String, false);

        // Multithreading (pthread)
        self.define_builtin("thread_spawn", vec![Type::String, Type::String], Type::String, false);
        self.define_builtin("thread_join", vec![Type::String], Type::Bool, false);
        self.define_builtin("thread_detach", vec![Type::String], Type::Bool, false);

        // Sistema Operacional / Baixo Nível
        self.define_builtin("peek", vec![Type::Ptr], Type::Any, false);
        self.define_builtin("poke", vec![Type::Ptr, Type::Any], Type::Void, false);
        self.define_builtin("addr", vec![Type::Any], Type::Ptr, false);
        self.define_builtin("malloc", vec![Type::Float], Type::Ptr, false);

        // Utils
        self.define_builtin("s_abs", vec![Type::Float], Type::Float, false);
        self.define_builtin("s_max", vec![Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("s_min", vec![Type::Float, Type::Float], Type::Float, false);
        self.define_builtin("s_len", vec![Type::String], Type::Float, false);
        self.define_builtin("s_upper", vec![Type::String], Type::String, false);
        self.define_builtin("s_time", vec![], Type::Float, false);
        self.define_builtin("s_sleep", vec![Type::Float], Type::Void, false);
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
            StmtKind::VarDeclaration(decl) => self.analyze_var_decl(decl, SemanticSymbolKind::Immutable),
            StmtKind::MutDeclaration(decl) => self.analyze_var_decl(&decl.to_var_decl(), SemanticSymbolKind::Mutable),
            StmtKind::ConstDeclaration(decl) => self.analyze_var_decl(&decl.to_var_decl(), SemanticSymbolKind::Constant),
            StmtKind::Input { name, var_type } => {
                let symbol = SemanticSymbol {
                    name: name.clone(),
                    symbol_type: var_type.clone(),
                    kind: SemanticSymbolKind::Mutable,
                    is_variadic: false,
                };
                if !self.symbol_table.define(symbol) {
                    self.errors.push(SemanticError::VariableAlreadyDeclared(name.clone()));
                }
            }
            StmtKind::VarAssignment(var_set) => {
                let expr_type = match self.type_check_expression(&var_set.value) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };

                if let Some(symbol) = self.symbol_table.lookup(&var_set.name) {
                    if symbol.kind == SemanticSymbolKind::Constant || symbol.kind == SemanticSymbolKind::Immutable {
                        self.errors.push(SemanticError::ImmutableAssignment(var_set.name.clone()));
                    }
                    if !self.is_compatible(&symbol.symbol_type, &expr_type) && expr_type != Type::Any {
                        self.errors.push(SemanticError::TypeMismatch {
                            expected: symbol.symbol_type.clone(),
                            found: expr_type,
                        });
                    }
                } else {
                    self.errors.push(SemanticError::VariableNotFound(var_set.name.clone()));
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
                    self.errors.push(SemanticError::FunctionAlreadyDeclared(func_decl.name.clone()));
                    return;
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
                            self.errors.push(SemanticError::TypeMismatch {
                                expected: expected_type.clone(),
                                found: return_type,
                            });
                        }
                    }
                    None => self.errors.push(SemanticError::ReturnOutsideFunction),
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
                self.analyze_var_decl(&var_decl, SemanticSymbolKind::Immutable);
            }
            StmtKind::DictDeclaration(decl) => {
                let var_decl = VarDecl {
                    name: decl.name.clone(),
                    var_type: decl.var_type.clone(),
                    value: decl.value.clone(),
                };
                self.analyze_var_decl(&var_decl, SemanticSymbolKind::Immutable);
            }
            StmtKind::ListPush(push) => {
                if let Some(symbol) = self.symbol_table.lookup(&push.name) {
                    if symbol.symbol_type != Type::List {
                        self.errors.push(SemanticError::InvalidOperation {
                            op: "list_push".to_string(),
                            type1: symbol.symbol_type.clone(),
                            type2: None,
                        });
                    }
                    let _ = self.type_check_expression(&push.value);
                } else {
                    self.errors.push(SemanticError::VariableNotFound(push.name.clone()));
                }
            }
            StmtKind::DictSet(set) => {
                if let Some(symbol) = self.symbol_table.lookup(&set.name) {
                    if symbol.symbol_type != Type::Dict {
                        self.errors.push(SemanticError::InvalidOperation {
                            op: "dict_set".to_string(),
                            type1: symbol.symbol_type.clone(),
                            type2: None,
                        });
                    }
                    let _ = self.type_check_expression(&set.key);
                    let _ = self.type_check_expression(&set.value);
                } else {
                    self.errors.push(SemanticError::VariableNotFound(set.name.clone()));
                }
            }
            StmtKind::Expression(expr) | StmtKind::FuncCall(expr) => {
                if let Err(e) = self.type_check_expression(expr) {
                    self.errors.push(e);
                }
            }
            StmtKind::Print(expressions) => {
                for expr in expressions {
                    if let Err(e) = self.type_check_expression(expr) {
                        self.errors.push(e);
                    }
                }
            }
            StmtKind::Import(_path) => {
                // Import statements are handled at runtime
                // No semantic analysis needed here
            }
            StmtKind::ClassDeclaration(_) => {
                // TODO: Implement class analysis
            }
        }
    }

    fn analyze_var_decl(&mut self, decl: &VarDecl, kind: SemanticSymbolKind) {
        let expr_type = match self.type_check_expression(&decl.value) {
            Ok(t) => t,
            Err(e) => {
                self.errors.push(e);
                return;
            }
        };

        let final_type = if let Some(ref expected_type) = decl.var_type {
            if !self.is_compatible(expected_type, &expr_type) {
                self.errors.push(SemanticError::TypeMismatch {
                    expected: expected_type.clone(),
                    found: expr_type,
                });
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
            self.errors.push(SemanticError::VariableAlreadyDeclared(decl.name.clone()));
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
                        self.symbol_table.exit_scope();
                        return;
                    }
                };

                let iterator_type = match iterable_type {
                    Type::List => Type::Any,
                    Type::String => Type::String,
                    Type::Any => Type::Any, // Permite iterar sobre Any
                    _ => {
                        self.errors.push(SemanticError::InvalidOperation {
                            op: "for-in".to_string(),
                            type1: iterable_type,
                            type2: None,
                        });
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
            return Err(SemanticError::TypeMismatch {
                expected: Type::Bool,
                found: expr_type,
            });
        }
        Ok(())
    }

    fn is_compatible(&self, expected: &Type, found: &Type) -> bool {
        if expected == found { return true; }
        // Compatibilidade Numérica
        if matches!(expected, Type::Float | Type::Int | Type::I64 | Type::I32 | Type::U8) &&
           matches!(found, Type::Float | Type::Int | Type::I64 | Type::I32 | Type::U8) {
               return true; // Por enquanto, permitimos cast implícito entre números
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
                    Err(SemanticError::VariableNotFound(name.clone()))
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
                            if left_type == Type::String || right_type == Type::String {
                                Ok(Type::String) // String + Any -> String
                            } else {
                                Ok(Type::Any)
                            }
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type == Type::Float || right_type == Type::Float { Ok(Type::Float) } else { Ok(Type::Int) }
                        } else if left_type == Type::String && right_type == Type::String {
                            Ok(Type::String)
                        } else {
                            Err(SemanticError::InvalidOperation { op: format!("{:?}", op), type1: left_type, type2: Some(right_type) })
                        }
                    }
                    BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                        if left_type == Type::Any || right_type == Type::Any {
                            Ok(Type::Any)
                        } else if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type == Type::Float || right_type == Type::Float { Ok(Type::Float) } else { Ok(Type::Int) }
                        } else {
                            Err(SemanticError::InvalidOperation { op: format!("{:?}", op), type1: left_type, type2: Some(right_type) })
                        }
                    }
                    BinaryOp::Equals | BinaryOp::NotEquals | BinaryOp::GreaterThan | BinaryOp::LessThan | BinaryOp::GreaterThanOrEquals | BinaryOp::LessThanOrEquals => {
                        if self.is_compatible(&left_type, &right_type) || self.is_compatible(&right_type, &left_type) { Ok(Type::Bool) } else {
                             Err(SemanticError::InvalidOperation { op: format!("{:?}", op), type1: left_type, type2: Some(right_type) })
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type == Type::Any || right_type == Type::Any {
                            Ok(Type::Bool)
                        } else if left_type == Type::Bool && right_type == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError::InvalidOperation { op: format!("{:?}", op), type1: left_type, type2: Some(right_type) })
                        }
                    }
                }
            }
            ExprKind::Unary { op, expr } => {
                let expr_type = self.type_check_expression(expr)?;
                match op {
                    UnaryOp::Negative => {
                        if expr_type.is_numeric() { Ok(expr_type) } else {
                            Err(SemanticError::InvalidOperation { op: "Negative".to_string(), type1: expr_type, type2: None })
                        }
                    }
                    UnaryOp::Not => {
                        if expr_type == Type::Any || expr_type == Type::Bool { Ok(Type::Bool) } else {
                            Err(SemanticError::InvalidOperation { op: "Not".to_string(), type1: expr_type, type2: None })
                        }
                    }
                }
            }
            ExprKind::FunctionCall { callee, args } => {
                let callee_type = self.type_check_expression(callee)?;

                if let Some(callee_name) = match &callee.kind {
                    ExprKind::Variable(name) => Some(name),
                    _ => None,
                } {
                    if let Some(symbol) = self.symbol_table.lookup(callee_name) {
                        if symbol.is_variadic {
                            // Para funções variádicas, apenas verificamos se os argumentos são compatíveis com Any (se definidos)
                            for arg in args {
                                let _ = self.type_check_expression(arg)?;
                            }
                            return Ok(Type::Any); // Ou o tipo de retorno esperado, se conhecido
                        }
                    }
                }

                if let Type::Function(param_types, return_type) = callee_type {
                    if args.len() != param_types.len() {
                        return Err(SemanticError::WrongNumberOfArguments { expected: param_types.len(), found: args.len() });
                    }

                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.type_check_expression(arg)?;
                        if !self.is_compatible(&param_types[i], &arg_type) {
                            return Err(SemanticError::TypeMismatch { expected: param_types[i].clone(), found: arg_type });
                        }
                    }
                    Ok(*return_type.clone())
                } else if callee_type == Type::Any {
                    // Se o tipo do callee é 'Any', não podemos verificar os argumentos.
                    // Apenas assumimos que a chamada é válida e retorna 'Any'.
                    for arg in args {
                        let _ = self.type_check_expression(arg)?;
                    }
                    Ok(Type::Any)
                } else {
                    Err(SemanticError::NotCallable(callee_type))
                }
            }
            ExprKind::PropertyAccess { target, property } => {
                let target_type = self.type_check_expression(target)?;

                match target_type {
                    Type::Any => Ok(Type::Any), // Permite acesso a propriedades em 'Any'
                    Type::List => {
                        if property == "push" {
                            Ok(Type::Function(vec![Type::Any], Box::new(Type::Void)))
                        } else {
                            Err(SemanticError::PropertyNotFound(property.clone()))
                        }
                    }
                    Type::Dict => {
                        if property == "set" {
                            Ok(Type::Function(vec![Type::Any, Type::Any], Box::new(Type::Void)))
                        } else {
                            Err(SemanticError::PropertyNotFound(property.clone()))
                        }
                    }
                    _ => Err(SemanticError::IndexAccessOnNonIndexable(target_type)),
                }
            }
            ExprKind::IndexAccess { target, index } => {
                let target_type = self.type_check_expression(target)?;
                let index_type = self.type_check_expression(index)?;

                match target_type {
                    Type::List => {
                        if index_type != Type::Int {
                            self.errors.push(SemanticError::InvalidIndexType(index_type));
                        }
                        Ok(Type::Any)
                    }
                    Type::Dict => {
                        if !matches!(index_type, Type::String | Type::Int | Type::Float | Type::Bool) {
                            self.errors.push(SemanticError::InvalidIndexType(index_type));
                        }
                        Ok(Type::Any)
                    }
                    Type::String => {
                        if index_type != Type::Int {
                            self.errors.push(SemanticError::InvalidIndexType(index_type));
                        }
                        Ok(Type::String)
                    }
                    _ => Err(SemanticError::IndexAccessOnNonIndexable(target_type)),
                }
            }
        }
    }
}
