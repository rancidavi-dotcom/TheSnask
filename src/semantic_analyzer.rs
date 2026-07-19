use crate::ast::{
    BinaryOp, ConditionalStmt, Expr, ExprKind, LiteralValue, LoopStmt,
    Program, Stmt, StmtKind, UnaryOp, VarDecl,
};
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

fn display_type(ty: &Type) -> String {
    match ty {
        Type::F32 => "f32".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Void => "void".to_string(),
        Type::I8 => "i8".to_string(),
        Type::I16 => "i16".to_string(),
        Type::U8 => "u8".to_string(),
        Type::U16 => "u16".to_string(),
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::Usize => "usize".to_string(),
        Type::Isize => "isize".to_string(),
        Type::Ptr => "ptr".to_string(),
        Type::User(name) => name.clone(),
        Type::Function(params, ret) => {
            let params = params
                .iter()
                .map(display_type)
                .collect::<Vec<_>>()
                .join(", ");
            format!("fun({}) -> {}", params, display_type(ret))
        }
        Type::Struct(name) => format!("struct {}", name),
        Type::Array(inner, count) => format!("[{}; {}]", display_type(inner), count),
        Type::Volatile(inner) => format!("volatile {}", display_type(inner)),
    }
}

include!(concat!(env!("OUT_DIR"), "/semantic_kind.rs"));

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: SemanticErrorKind,
    pub span: Span,
    pub help: Option<String>,
    pub notes: Vec<String>,
}

impl SemanticError {
    pub fn new(kind: SemanticErrorKind, span: Span) -> Self {
        SemanticError { kind, span, help: None, notes: Vec::new() }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help); self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note); self
    }

    pub fn message(&self) -> String { self.kind.message() }
    pub fn code(&self) -> &'static str { self.kind.code() }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SemanticSymbolTable,
    current_function_return_type: Option<Type>,
    pub errors: Vec<SemanticError>,
    tiny_mode: bool,
    unsafe_depth: usize,
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
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

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = SemanticAnalyzer {
            symbol_table: SemanticSymbolTable::new(),
            current_function_return_type: None,
            errors: Vec::new(),
            tiny_mode: false,
            unsafe_depth: 0,
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
        self.register_systems_low_level_builtins();
    }

    fn define_builtin(
        &mut self,
        name: &str,
        params: Vec<Type>,
        return_type: Type,
        is_variadic: bool,
    ) {
        let symbol = SemanticSymbol {
            name: name.to_string(),
            symbol_type: Type::Function(params, Box::new(return_type)),
            kind: SemanticSymbolKind::Function,
            is_variadic,
        };
        self.symbol_table.define(symbol);
    }

    fn register_systems_low_level_builtins(&mut self) {
        for (name, ret) in [
            ("as_u8", Type::U8),
            ("as_u16", Type::U16),
            ("as_u32", Type::U32),
            ("as_u64", Type::U64),
            ("as_i8", Type::I8),
            ("as_i16", Type::I16),
            ("as_i32", Type::I32),
            ("as_i64", Type::I64),
            ("as_usize", Type::Usize),
            ("as_isize", Type::Isize),
            ("lo_u8", Type::U8),
            ("hi_u8", Type::U8),
            ("is_zero_u8", Type::Bool),
            ("is_negative_u8", Type::Bool),
            ("null_ptr", Type::Ptr),
        ] {
            let params = if name == "null_ptr" {
                Vec::new()
            } else {
                vec![Type::U64]
            };
            self.define_builtin(name, params, ret, false);
        }

        for name in [
            "make_u16",
            "bit_set",
            "bit_clear",
            "bit_toggle",
            "bit_write",
            "flag_set",
            "flag_clear",
            "flag_write",
            "wrapping_inc",
            "wrapping_dec",
        ] {
            self.define_builtin(name, vec![Type::U8, Type::U8], Type::U8, false);
        }

        for name in ["bit_test", "flag_has"] {
            self.define_builtin(name, vec![Type::U8, Type::U8], Type::Bool, false);
        }

        for name in [
            "carry_add_u8",
            "borrow_sub_u8",
            "overflow_add_i8",
            "overflow_sub_i8",
        ] {
            self.define_builtin(
                name,
                vec![Type::U8, Type::U8, Type::U8],
                Type::Bool,
                false,
            );
        }

        self.define_builtin("mem_alloc", vec![Type::U64], Type::Ptr, false);
        self.define_builtin("mem_alloc_zero", vec![Type::U64], Type::Ptr, false);
        self.define_builtin("mem_free", vec![Type::Ptr], Type::Void, false);
        self.define_builtin("ptr_add", vec![Type::Ptr, Type::U64], Type::Ptr, false);
        self.define_builtin("mem_read_u8", vec![Type::Ptr, Type::U64], Type::U8, false);
        self.define_builtin("mem_read_u16", vec![Type::Ptr, Type::U64], Type::U16, false);
        self.define_builtin("mem_read_u32", vec![Type::Ptr, Type::U64], Type::U32, false);
        self.define_builtin(
            "mem_write_u8",
            vec![Type::Ptr, Type::U64, Type::U8],
            Type::Void,
            false,
        );
        self.define_builtin(
            "mem_write_u16",
            vec![Type::Ptr, Type::U64, Type::U16],
            Type::Void,
            false,
        );
        self.define_builtin(
            "mem_write_u32",
            vec![Type::Ptr, Type::U64, Type::U32],
            Type::Void,
            false,
        );
        self.define_builtin(
            "mem_fill_u8",
            vec![Type::Ptr, Type::U64, Type::U8],
            Type::Void,
            false,
        );
        self.define_builtin(
            "mem_copy",
            vec![Type::Ptr, Type::Ptr, Type::U64],
            Type::Void,
            false,
        );
    }

    pub fn analyze(&mut self, program: &Program) {
        self.register_functions(program);
        for statement in program {
            self.analyze_statement(statement);
        }
    }

    fn register_functions(&mut self, program: &Program) {
        for statement in program {
            if let StmtKind::FuncDeclaration(func) = &statement.kind {
                let params_types: Vec<Type> = func.params.iter().map(|p| p.1.clone()).collect();
                self.symbol_table.define(SemanticSymbol {
                    name: func.name.clone(),
                    symbol_type: Type::Function(
                        params_types,
                        Box::new(func.return_type.clone().unwrap_or(Type::Void)),
                    ),
                    kind: SemanticSymbolKind::Function,
                    is_variadic: false,
                });
            }
        }
    }

    fn validate_type_exists(&mut self, ty: &Type, span: &Span) {
        match ty {
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
                let elif_returns = cond
                    .elif_blocks
                    .iter()
                    .all(|b| b.body.iter().any(Self::stmt_guarantees_return));
                let else_returns = cond
                    .else_block
                    .as_ref()
                    .map(|body| body.iter().any(Self::stmt_guarantees_return))
                    .unwrap_or(false);
                if_returns && elif_returns && else_returns
            }
            StmtKind::UnsafeBlock(body) => body.iter().any(Self::stmt_guarantees_return),
            _ => false,
        }
    }

    fn body_guarantees_return(body: &[Stmt]) -> bool {
        body.iter().any(Self::stmt_guarantees_return)
    }

    fn analyze_statement(&mut self, statement: &Stmt) {
        match &statement.kind {
            StmtKind::VarDeclaration(decl) => {
                self.analyze_var_decl(decl, SemanticSymbolKind::Immutable, statement.span.clone())
            }
            StmtKind::MutDeclaration(decl) => self.analyze_var_decl(
                &decl.to_var_decl(),
                SemanticSymbolKind::Mutable,
                statement.span.clone(),
            ),
            StmtKind::ConstDeclaration(decl) => self.analyze_var_decl(
                &decl.to_var_decl(),
                SemanticSymbolKind::Constant,
                statement.span.clone(),
            ),
            StmtKind::VarAssignment(var_set) => {
                let expr_type = match self.type_check_expression(&var_set.value) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };

                if let Some(symbol) = self.symbol_table.lookup(&var_set.name) {
                    if symbol.kind == SemanticSymbolKind::Constant
                        || symbol.kind == SemanticSymbolKind::Immutable
                    {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::ImmutableAssignment(var_set.name.clone()),
                            statement.span.clone(),
                        ));
                    }
                    if !self.is_compatible(&symbol.symbol_type, &expr_type) {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::TypeMismatch {
                                expected: symbol.symbol_type.clone(),
                                found: expr_type,
                            },
                            statement.span.clone(),
                        ));
                    }
                } else {
                    self.errors.push(
                        self.mk_variable_not_found(var_set.name.clone(), statement.span.clone()),
                    );
                }
            }
            StmtKind::IndexAssignment(i) => {
                let target_type = match self.type_check_expression(&i.target) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };
                let index_type = match self.type_check_expression(&i.index) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };
                let value_type = match self.type_check_expression(&i.value) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };

                if !target_type.is_numeric()
                    && !index_type.is_numeric()
                    && !self.is_compatible(&target_type, &value_type)
                {
                    self.errors.push(SemanticError::new(
                        SemanticErrorKind::IndexAccessOnNonIndexable(target_type),
                        statement.span.clone(),
                    ));
                }
            }
            StmtKind::FuncDeclaration(func_decl) => {
                for (_, param_type) in &func_decl.params {
                    self.validate_type_exists(param_type, &statement.span);
                }
                if let Some(return_type) = &func_decl.return_type {
                    self.validate_type_exists(return_type, &statement.span);
                }

                let params_types: Vec<Type> =
                    func_decl.params.iter().map(|p| p.1.clone()).collect();
                let func_symbol = SemanticSymbol {
                    name: func_decl.name.clone(),
                    symbol_type: Type::Function(
                        params_types,
                        Box::new(func_decl.return_type.clone().unwrap_or(Type::Void)),
                    ),
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
                self.current_function_return_type =
                    Some(func_decl.return_type.clone().unwrap_or(Type::Void));

                for (param_name, param_type) in &func_decl.params {
                    let param_symbol = SemanticSymbol {
                        name: param_name.clone(),
                        symbol_type: param_type.clone(),
                        kind: SemanticSymbolKind::Parameter,
                        is_variadic: false,
                    };
                    self.symbol_table.define(param_symbol);
                }

                if func_decl.is_unsafe {
                    self.unsafe_depth += 1;
                }

                for stmt in &func_decl.body {
                    self.analyze_statement(stmt);
                }

                if func_decl.is_unsafe {
                    self.unsafe_depth = self.unsafe_depth.saturating_sub(1);
                }

                if let Some(return_type) = &func_decl.return_type {
                    if *return_type != Type::Void
                        && !func_decl.is_extern
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
            StmtKind::Expression(expr) | StmtKind::FuncCall(expr) => {
                if let Err(e) = self.type_check_expression(expr) {
                    self.errors.push(e);
                }
            }
            StmtKind::UnsafeBlock(body) => {
                self.unsafe_depth += 1;
                for s in body {
                    self.analyze_statement(s);
                }
                self.unsafe_depth = self.unsafe_depth.saturating_sub(1);
            }
            StmtKind::Asm(_) => {}
            StmtKind::WritePtr { ptr, value, type_hint } => {
                let _ = self.type_check_expression(ptr);
                let val_ty = self.type_check_expression(value);
                if let Ok(ty) = val_ty {
                    if ty != *type_hint {
                        self.errors.push(SemanticError::new(
                            SemanticErrorKind::TypeMismatch {
                                expected: type_hint.clone(),
                                found: ty,
                            },
                            statement.span.clone(),
                        ));
                    }
                }
            }
            StmtKind::Outb { port, value } => {
                let _ = self.type_check_expression(port);
                let _ = self.type_check_expression(value);
            }
            StmtKind::GlobalAsm(_) => {}
            StmtKind::StructDeclaration(_) => {
                // Struct declarations are validated during parsing; nothing to analyze here
            }
            StmtKind::Fence { .. } => {}
            StmtKind::AtomicRmw { ptr, value, .. } => {
                let _ = self.type_check_expression(ptr);
                let _ = self.type_check_expression(value);
            }
            StmtKind::VolatileStore { ptr, value, type_hint: _ } => {
                let _ = self.type_check_expression(ptr);
                let _ = self.type_check_expression(value);
            }
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
                return;
            }
        };

        let final_type = if let Some(ref expected_type) = decl.var_type {
            if !self.is_compatible(expected_type, &expr_type) {
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
        if let Err(e) = self.check_condition(&cond.if_block.condition) {
            self.errors.push(e);
        }
        self.symbol_table.enter_scope();
        for stmt in &cond.if_block.body {
            self.analyze_statement(stmt);
        }
        self.symbol_table.exit_scope();

        for elif in &cond.elif_blocks {
            if let Err(e) = self.check_condition(&elif.condition) {
                self.errors.push(e);
            }
            self.symbol_table.enter_scope();
            for stmt in &elif.body {
                self.analyze_statement(stmt);
            }
            self.symbol_table.exit_scope();
        }

        if let Some(else_body) = &cond.else_block {
            self.symbol_table.enter_scope();
            for stmt in else_body {
                self.analyze_statement(stmt);
            }
            self.symbol_table.exit_scope();
        }
    }

    fn analyze_loop(&mut self, loop_stmt: &LoopStmt) {
        self.symbol_table.enter_scope();
        match loop_stmt {
            LoopStmt::While { condition, body } => {
                if let Err(e) = self.check_condition(condition) {
                    self.errors.push(e);
                }
                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
            LoopStmt::For {
                iterator,
                iterable,
                body,
            } => {
                let iterable_type = match self.type_check_expression(iterable) {
                    Ok(t) => t,
                    Err(e) => {
                        self.errors.push(e);
                        return;
                    }
                };

                let iterator_type = if iterable_type.is_numeric() || iterable_type == Type::Bool {
                    iterable_type
                } else {
                    self.errors.push(SemanticError::new(
                        SemanticErrorKind::InvalidOperation {
                            op: "for-in".to_string(),
                            type1: iterable_type,
                            type2: None,
                        },
                        iterable.span.clone(),
                    ));
                    return;
                };

                let symbol = SemanticSymbol {
                    name: iterator.clone(),
                    symbol_type: iterator_type,
                    kind: SemanticSymbolKind::Immutable,
                    is_variadic: false,
                };
                self.symbol_table.define(symbol);

                for stmt in body {
                    self.analyze_statement(stmt);
                }
            }
        }
        self.symbol_table.exit_scope();
    }

    fn check_condition(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        let expr_type = self.type_check_expression(expr)?;
        if expr_type != Type::Bool && !expr_type.is_numeric() {
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
        if expected == found {
            return true;
        }
        if matches!(
            expected,
            Type::F32
                | Type::F64
                | Type::I8
                | Type::I16
                | Type::I64
                | Type::I32
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::Isize
        ) && matches!(
            found,
            Type::F32
                | Type::F64
                | Type::I8
                | Type::I16
                | Type::I64
                | Type::I32
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::Isize
        ) {
            return true;
        }
        false
    }

    fn systems_low_level_call_type(
        &mut self,
        name: &str,
        args: &[Expr],
        span: Span,
    ) -> Result<Option<Type>, SemanticError> {
        let is_raw_memory = matches!(
            name,
            "mem_alloc"
                | "mem_alloc_zero"
                | "mem_free"
                | "ptr_add"
                | "mem_read_u8"
                | "mem_read_u16"
                | "mem_read_u32"
                | "mem_write_u8"
                | "mem_write_u16"
                | "mem_write_u32"
                | "mem_fill_u8"
                | "mem_copy"
        );
        if is_raw_memory && self.unsafe_depth == 0 {
            return Err(
                SemanticError::new(
                    SemanticErrorKind::RestrictedNativeFunction {
                        name: name.to_string(),
                        help: "raw memory access belongs inside `@unsafe`.".to_string(),
                    },
                    span,
                )
                .with_help(format!(
                    "`{name}` performs raw memory access. Wrap the smallest possible region in `@unsafe`."
                )),
            );
        }

        let mut arg_types = Vec::new();
        for arg in args {
            arg_types.push(self.type_check_expression(arg)?);
        }

        let ret = match name {
            "as_u8" => Some(Type::U8),
            "as_u16" => Some(Type::U16),
            "as_u32" => Some(Type::U32),
            "as_u64" => Some(Type::U64),
            "as_i8" => Some(Type::I8),
            "as_i16" => Some(Type::I16),
            "as_i32" => Some(Type::I32),
            "as_i64" => Some(Type::I64),
            "as_usize" => Some(Type::Usize),
            "as_isize" => Some(Type::Isize),
            "null_ptr" => Some(Type::Ptr),
            "lo_u8" | "hi_u8" => Some(Type::U8),
            "make_u16" => Some(Type::U16),
            "is_zero_u8" | "is_negative_u8" | "bit_test" | "flag_has" | "carry_add_u8"
            | "borrow_sub_u8" | "overflow_add_i8" | "overflow_sub_i8" => Some(Type::Bool),
            "bit_set" | "bit_clear" | "bit_toggle" | "bit_write" | "flag_set" | "flag_clear"
            | "flag_write" | "wrapping_inc" | "wrapping_dec" => {
                Some(arg_types.first().cloned().unwrap_or(Type::Void))
            }
            "mem_alloc" | "mem_alloc_zero" | "ptr_add" => Some(Type::Ptr),
            "mem_read_u8" => Some(Type::U8),
            "mem_read_u16" => Some(Type::U16),
            "mem_read_u32" => Some(Type::U32),
            "mem_free" | "mem_write_u8" | "mem_write_u16" | "mem_write_u32" | "mem_fill_u8"
            | "mem_copy" => Some(Type::Void),
            _ => None,
        };

        Ok(ret)
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
                LiteralValue::Number(n) => {
                    if n.fract() == 0.0 {
                        Ok(Type::I32)
                    } else {
                        Ok(Type::F64)
                    }
                }
                LiteralValue::Boolean(_) => Ok(Type::Bool),
                LiteralValue::String(_) => Ok(Type::Ptr),
                LiteralValue::Nil => Ok(Type::Ptr),
            },
            ExprKind::Binary { left, op, right } => {
                let left_type = self.type_check_expression(left)?;
                let right_type = self.type_check_expression(right)?;

                match op {
                    BinaryOp::Add => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type.is_float() || right_type.is_float() {
                                Ok(Type::F64)
                            } else if left_type == right_type {
                                Ok(left_type)
                            } else {
                                Ok(Type::I32)
                            }
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
                    BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::IntDivide
                    | BinaryOp::Modulo => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type.is_float() || right_type.is_float() {
                                Ok(Type::F64)
                            } else if left_type == right_type {
                                Ok(left_type)
                            } else {
                                Ok(Type::I32)
                            }
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
                    BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight => {
                        if left_type.is_integer() && right_type.is_integer() {
                            Ok(left_type)
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
                        if self.is_compatible(&left_type, &right_type)
                            || self.is_compatible(&right_type, &left_type)
                        {
                            Ok(Type::Bool)
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
                }
            }
            ExprKind::Unary { op, expr } => {
                let expr_type = self.type_check_expression(expr)?;
                match op {
                    UnaryOp::Negative => {
                        if expr_type.is_numeric() {
                            Ok(expr_type)
                        } else {
                            Err(SemanticError::new(
                                SemanticErrorKind::InvalidOperation {
                                    op: "Negative".to_string(),
                                    type1: expr_type,
                                    type2: None,
                                },
                                expression.span.clone(),
                            ))
                        }
                    }
                    UnaryOp::Not => Ok(Type::Bool),
                    UnaryOp::BitNot => {
                        if expr_type.is_integer() {
                            Ok(expr_type)
                        } else {
                            Err(SemanticError::new(
                                SemanticErrorKind::InvalidOperation {
                                    op: "BitNot".to_string(),
                                    type1: expr_type,
                                    type2: None,
                                },
                                expression.span.clone(),
                            ))
                        }
                    }
                }
            }
            ExprKind::FunctionCall { callee, args } => {
                if let ExprKind::Variable(name) = &callee.kind {
                    if let Some(ret) =
                        self.systems_low_level_call_type(name, args, expression.span)?
                    {
                        return Ok(ret);
                    }
                }

                let callee_symbol = if let ExprKind::Variable(name) = &callee.kind {
                    self.symbol_table.lookup(name).cloned()
                } else {
                    None
                };
                let callee_type = self.type_check_expression(callee)?;
                if let Type::Function(param_types, return_type) = callee_type {
                    let is_variadic = callee_symbol
                        .as_ref()
                        .map(|s| s.is_variadic)
                        .unwrap_or(false);
                    if (!is_variadic && args.len() != param_types.len())
                        || (is_variadic && args.len() < param_types.len())
                    {
                        return Err(SemanticError::new(
                            SemanticErrorKind::WrongNumberOfArguments {
                                expected: param_types.len(),
                                found: args.len(),
                            },
                            expression.span.clone(),
                        ));
                    }
                    for (i, arg) in args.iter().enumerate() {
                        let arg_type = self.type_check_expression(arg)?;
                        let expected_type = if i < param_types.len() {
                            param_types[i].clone()
                        } else if is_variadic {
                            param_types.last().cloned().unwrap_or(Type::Void)
                        } else {
                            return Err(SemanticError::new(
                                SemanticErrorKind::WrongNumberOfArguments {
                                    expected: param_types.len(),
                                    found: args.len(),
                                },
                                expression.span.clone(),
                            ));
                        };
                        if !self.is_compatible(&expected_type, &arg_type) {
                            return Err(SemanticError::new(
                                SemanticErrorKind::TypeMismatch {
                                    expected: expected_type,
                                    found: arg_type,
                                },
                                arg.span.clone(),
                            ));
                        }
                    }
                    Ok(*return_type)
                } else {
                    Err(SemanticError::new(
                        SemanticErrorKind::NotCallable(callee_type),
                        expression.span.clone(),
                    ))
                }
            }
            ExprKind::IndexAccess { target, index } => {
                let _target_type = self.type_check_expression(target)?;
                let index_type = self.type_check_expression(index)?;
                if index_type.is_numeric() {
                    Ok(Type::Void)
                } else {
                    Err(SemanticError::new(
                        SemanticErrorKind::InvalidIndexType(index_type),
                        expression.span.clone(),
                    ))
                }
            }
            ExprKind::Deref { ptr, type_hint } => {
                let _ = self.type_check_expression(ptr)?;
                Ok(type_hint.clone())
            }
            ExprKind::Inb(port) => {
                let _ = self.type_check_expression(port)?;
                Ok(Type::U8)
            }
            ExprKind::AddrOf(fn_name) => {
                let _ = self.symbol_table.lookup(fn_name);
                Ok(Type::Ptr)
            }
            ExprKind::SizeOf(_) => {
                Ok(Type::U64)
            }
            ExprKind::AlignOf(_) => {
                Ok(Type::U64)
            }
            ExprKind::OffsetOf { .. } => {
                Ok(Type::U64)
            }
            ExprKind::VolatileLoad { ptr, type_hint } => {
                let _ = self.type_check_expression(ptr)?;
                Ok(type_hint.clone())
            }
            ExprKind::VolatileStore { ptr, value } => {
                let _ = self.type_check_expression(ptr)?;
                let _ = self.type_check_expression(value)?;
                Ok(Type::Void)
            }
            ExprKind::IntToPtr { expr, type_hint } => {
                let _ = self.type_check_expression(expr)?;
                Ok(type_hint.clone())
            }
            ExprKind::PtrToInt { expr, type_hint } => {
                let _ = self.type_check_expression(expr)?;
                Ok(type_hint.clone())
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
    fn low_level_integer_types_accept_bitwise_and_wrapping_ops() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let a: u8 = 0xF0
        let b: u8 = 0x0F
        let c: u8 = (a & b) | 1
        let d: u8 = wrapping_add(c, 255)
        let e: u16 = 1 << 8
        let f: i32 = ~1
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected low-level integer program to type-check, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn nes_foundation_builtins_type_check_in_unsafe_region() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        @unsafe:
            let mem: ptr = mem_alloc_zero(65536)
            mem_write_u8(mem, 0xFFFC, 0x00)
            mem_write_u8(mem, 0xFFFD, 0x80)
            let reset: u16 = mem_read_u16(mem, 0xFFFC)
            let lo: u8 = lo_u8(reset)
            let hi: u8 = hi_u8(reset)
            let pc: u16 = make_u16(lo, hi)
            let flags: u8 = flag_write(0, 7, true)
            let has_n: bool = flag_has(flags, 7)
            let z: bool = is_zero_u8(as_u8(pc))
            let n: bool = is_negative_u8(flags)
            let carry: bool = carry_add_u8(0xFF, 1, 0)
            let borrow: bool = borrow_sub_u8(0, 1, 0)
            let ov1: bool = overflow_add_i8(127, 1, 0)
            let ov2: bool = overflow_sub_i8(128, 1, 0)
            mem_free(mem)
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected NES foundation helpers to type-check, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn raw_memory_builtins_require_unsafe() {
        let analyzer = analyze_source(
            r#"
class main
    fun start()
        let mem: ptr = mem_alloc(65536)
"#,
        );

        assert!(
            analyzer.errors.iter().any(|e| matches!(
                e.kind,
                SemanticErrorKind::RestrictedNativeFunction { ref name, .. } if name == "mem_alloc"
            )),
            "expected mem_alloc to require @unsafe, got: {:?}",
            analyzer.errors
        );
    }

    #[test]
    fn typed_function_without_return_is_reported() {
        let analyzer = analyze_source(
            r#"
fun meaning() : i32
    let x: i32 = 1

class main
    fun start()
        let y: i32 = meaning()
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
fun classify(x: i32) : i32
    if x > 0
        return 1
    else
        return 0

class main
    fun start()
        let y: i32 = classify(2)
"#,
        );

        assert!(
            analyzer.errors.is_empty(),
            "expected no semantic errors, got: {:?}",
            analyzer.errors
        );
    }
}
