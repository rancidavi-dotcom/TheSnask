use crate::ast::{Program, Stmt, StmtKind, Expr, ExprKind, LiteralValue, BinaryOp, UnaryOp, VarDecl, MutDecl, ConstDecl, VarSet, ConditionalStmt, LoopStmt, FuncDecl};
use crate::symbol_table::{SymbolTable, Symbol};
use crate::types::Type;
use crate::value::Value;
use std::collections::HashMap;
use std::io;

pub enum InterpretResult {
    Ok,
    RuntimeError(String),
}

// Internal control flow for the interpreter
enum ControlFlow {
    Continue,
    Return(Value),
    Error(String),
}

#[derive(Clone)]
pub struct Interpreter {
    globals: SymbolTable,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Interpreter {
            globals: SymbolTable::new(),
        };
        crate::stdlib::register_stdlib(&mut interpreter.globals);
        interpreter
    }

    pub fn get_globals_mut(&mut self) -> &mut SymbolTable {
        &mut self.globals
    }

    /// Chama uma função diretamente por Value, útil para chamadas de runtime
    pub fn call_function_by_value(&mut self, func_val: Value, args: Vec<Value>) -> Result<Value, String> {
        match func_val {
            Value::Function(func_decl) => {
                if args.len() != func_decl.params.len() {
                    return Err(format!("Número incorreto de argumentos para a função '{}'. Esperado {}, encontrado {}.", func_decl.name, func_decl.params.len(), args.len()));
                }

                self.globals.enter_scope();
                for (i, (param_name, _param_type)) in func_decl.params.iter().enumerate() {
                    self.globals.define(param_name.clone(), args[i].clone(), false, false);
                }
                let result = self.execute_block(func_decl.body.clone());
                self.globals.exit_scope();

                match result {
                    ControlFlow::Return(val) => Ok(val),
                    ControlFlow::Error(e) => Err(e),
                    ControlFlow::Continue => Ok(Value::Nil), // Function finished without return
                }
            },
            Value::NativeFunction(func) => {
                func(args, self)
            },
            _ => Err(format!("Tentativa de chamar um valor não-invocável: {:?}", func_val))
        }
    }

    pub fn interpret(&mut self, program: Program) -> InterpretResult {
        for statement in program {
            match self.execute_statement(statement) {
                ControlFlow::Continue => continue,
                ControlFlow::Return(_) => return InterpretResult::RuntimeError("Unexpected return statement at top level.".to_string()),
                ControlFlow::Error(msg) => return InterpretResult::RuntimeError(msg),
            }
        }
        InterpretResult::Ok
    }

    fn execute_statement(&mut self, statement: Stmt) -> ControlFlow {
        match statement.kind {
            StmtKind::VarDeclaration(var_decl) => self.execute_var_declaration(var_decl),
            StmtKind::MutDeclaration(mut_decl) => self.execute_mut_declaration(mut_decl),
            StmtKind::ConstDeclaration(const_decl) => self.execute_const_declaration(const_decl),
            StmtKind::VarAssignment(var_set) => self.execute_var_assignment(var_set),
            StmtKind::Print(expressions) => self.execute_print_statement(expressions),
            StmtKind::Input { name, var_type } => self.execute_input_statement(name, var_type),
            StmtKind::Conditional(conditional) => self.execute_conditional_statement(conditional),
            StmtKind::Loop(loop_stmt) => self.execute_loop_statement(loop_stmt),
            StmtKind::FuncDeclaration(func_decl) => self.execute_func_declaration(func_decl),
            StmtKind::Return(expr) => self.execute_return_statement(expr),
            StmtKind::FuncCall(expr) => {
                match self.evaluate_expression(expr) {
                    Ok(_) => ControlFlow::Continue,
                    Err(e) => ControlFlow::Error(e),
                }
            },
            StmtKind::Import(path) => {
                if path == "collections" { // Handle standard library module
                    ControlFlow::Continue
                } else { // Handle .snask files
                    match crate::modules::load_module(&path) {
                        Ok(module_program) => self.execute_block(module_program),
                        Err(e) => ControlFlow::Error(e),
                    }
                }
            },
            _ => ControlFlow::Error(format!("Statement not yet implemented: {:?}", statement.kind)),
        }
    }

    fn execute_input_statement(&mut self, name: String, var_type: Type) -> ControlFlow {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return ControlFlow::Error("Não foi possível ler a entrada do console.".to_string());
        }
        let trimmed_input = input.trim();

        let value = match var_type {
            Type::String => Value::String(trimmed_input.to_string()),
            Type::Float => match trimmed_input.parse::<f64>() {
                Ok(n) => Value::Number(n),
                Err(_) => return ControlFlow::Error(format!("Entrada inválida. Esperado um número float, mas recebeu '{}'.", trimmed_input)),
            },
            Type::Int => match trimmed_input.parse::<i64>() {
                Ok(n) => Value::Number(n as f64),
                Err(_) => return ControlFlow::Error(format!("Entrada inválida. Esperado um número inteiro, mas recebeu '{}'.", trimmed_input)),
            },
            Type::Bool => match trimmed_input.parse::<bool>() {
                Ok(b) => Value::Boolean(b),
                Err(_) => return ControlFlow::Error(format!("Entrada inválida. Esperado 'true' ou 'false', mas recebeu '{}'.", trimmed_input)),
            },
            _ => return ControlFlow::Error(format!("Tipo de 'input' não suportado: {:?}", var_type)),
        };

        self.globals.define(name, value, true, true);
        ControlFlow::Continue
    }

    fn evaluate_expression(&mut self, expression: Expr) -> Result<Value, String> {
        match expression.kind {
            ExprKind::Literal(literal) => Ok(self.evaluate_literal(literal)),
            ExprKind::Variable(name) => self.evaluate_variable(name),
            ExprKind::Unary { op, expr } => self.evaluate_unary_expression(op, *expr),
            ExprKind::Binary { op, left, right } => self.evaluate_binary_expression(op, *left, *right),
            ExprKind::FunctionCall { callee, args } => self.evaluate_function_call(*callee, args),
            ExprKind::IndexAccess { target, index } => self.evaluate_index_access(*target, *index),
            ExprKind::PropertyAccess { target, property } => {
                let obj_val = self.evaluate_expression(*target)?;
                match obj_val {
                    Value::Dict(dict) => {
                        let prop_key = Value::String(property.clone());
                        dict.get(&prop_key)
                            .cloned()
                            .ok_or_else(|| format!("Propriedade '{}' não encontrada no objeto.", property))
                    },
                    _ => Err(format!("Tentativa de acessar propriedade '{}' em tipo não-objeto: {:?}", property, obj_val)),
                }
            }
        }
    }

    fn evaluate_index_access(&mut self, target: Expr, index: Expr) -> Result<Value, String> {
        let target_val = self.evaluate_expression(target)?;
        let index_val = self.evaluate_expression(index)?;

        match target_val {
            Value::List(list) => {
                if let Value::Number(idx) = index_val {
                    let idx = idx as usize;
                    if idx < list.len() {
                        Ok(list[idx].clone())
                    } else {
                        Err(format!("Erro de tempo de execução: Índice fora dos limites da lista. Tamanho: {}, Índice: {}", list.len(), idx))
                    }
                } else {
                    Err(format!("Erro de tempo de execução: Índice de lista não numérico: {:?}", index_val))
                }
            },
            Value::Dict(dict) => {
                if dict.contains_key(&index_val) {
                    Ok(dict[&index_val].clone())
                } else {
                    Err(format!("Erro de tempo de execução: Chave de dicionário não encontrada: {:?}", index_val))
                }
            },
            _ => Err(format!("Erro de tempo de execução: Tentativa de indexar valor não indexável: {:?}", target_val)),
        }
    }

    fn evaluate_literal(&mut self, literal: LiteralValue) -> Value {
        match literal {
            LiteralValue::Number(n) => Value::Number(n),
            LiteralValue::String(s) => Value::String(s),
            LiteralValue::Boolean(b) => Value::Boolean(b),
            LiteralValue::Nil => Value::Nil,
            LiteralValue::List(expr_list) => {
                let mut list = Vec::new();
                for expr in expr_list {
                    if let Ok(val) = self.evaluate_expression(expr) {
                        list.push(val);
                    } else {
                        list.push(Value::Nil);
                    }
                }
                Value::List(list)
            },
            LiteralValue::Dict(expr_dict) => {
                let mut dict = HashMap::new();
                for (key_expr, val_expr) in expr_dict {
                    if let (Ok(key_val), Ok(actual_val)) = (self.evaluate_expression(key_expr), self.evaluate_expression(val_expr)) {
                        dict.insert(key_val, actual_val);
                    }
                }
                Value::Dict(dict)
            },
        }
    }

    fn evaluate_variable(&mut self, name: String) -> Result<Value, String> {
        match self.globals.get(&name) {
            Some(Symbol { value, .. }) => Ok(value.clone()),
            None => Err(format!("Variável '{}' não encontrada.", name)),
        }
    }

    fn execute_var_declaration(&mut self, var_decl: VarDecl) -> ControlFlow {
        match self.evaluate_expression(var_decl.value) {
            Ok(value) => {
                self.globals.define(var_decl.name, value, false, false);
                ControlFlow::Continue
            },
            Err(e) => ControlFlow::Error(e),
        }
    }

    fn execute_mut_declaration(&mut self, mut_decl: MutDecl) -> ControlFlow {
        match self.evaluate_expression(mut_decl.value) {
            Ok(value) => {
                self.globals.define(mut_decl.name, value, true, true);
                ControlFlow::Continue
            },
            Err(e) => ControlFlow::Error(e),
        }
    }

    fn execute_const_declaration(&mut self, const_decl: ConstDecl) -> ControlFlow {
        match self.evaluate_expression(const_decl.value) {
            Ok(value) => {
                self.globals.define(const_decl.name, value, false, false);
                ControlFlow::Continue
            },
            Err(e) => ControlFlow::Error(e),
        }
    }

    fn execute_var_assignment(&mut self, var_set: VarSet) -> ControlFlow {
        let value = match self.evaluate_expression(var_set.value) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };

        match self.globals.get_mut(&var_set.name) {
            Some(symbol) => {
                if !symbol.is_reassignable {
                    return ControlFlow::Error(format!("Variável '{}' não pode ser reatribuída (é constante).", var_set.name));
                }
                symbol.value = value;
                ControlFlow::Continue
            },
            None => ControlFlow::Error(format!("Variável '{}' não encontrada para atribuição.", var_set.name)),
        }
    }

    fn execute_print_statement(&mut self, expressions: Vec<Expr>) -> ControlFlow {
        let mut output = String::new();
        for (i, expr) in expressions.iter().enumerate() {
            match self.evaluate_expression(expr.clone()) {
                Ok(value) => {
                    output.push_str(&format!("{}", value));
                    if i < expressions.len() - 1 {
                        output.push_str(" ");
                    }
                },
                Err(e) => return ControlFlow::Error(e),
            }
        }
        println!("{}", output);
        ControlFlow::Continue
    }

    fn evaluate_unary_expression(&mut self, op: UnaryOp, expr: Expr) -> Result<Value, String> {
        let right = self.evaluate_expression(expr)?;
        match op {
            UnaryOp::Negative => {
                if let Value::Number(n) = right {
                    Ok(Value::Number(-n))
                } else {
                    Err(format!("Operador unário '-' aplicado a tipo não numérico: {:?}", right))
                }
            }
        }
    }

    fn evaluate_binary_expression(&mut self, op: BinaryOp, left: Expr, right: Expr) -> Result<Value, String> {
        let left_val = self.evaluate_expression(left)?;
        let right_val = self.evaluate_expression(right)?;

        match op {
            BinaryOp::Add => self.add_values(left_val, right_val),
            BinaryOp::Subtract => self.subtract_values(left_val, right_val),
            BinaryOp::Multiply => self.multiply_values(left_val, right_val),
            BinaryOp::Divide => self.divide_values(left_val, right_val),
            BinaryOp::Equals => Ok(Value::Boolean(left_val == right_val)),
            BinaryOp::NotEquals => Ok(Value::Boolean(left_val != right_val)),
            BinaryOp::GreaterThan => self.compare_values(left_val, right_val, |a, b| a > b),
            BinaryOp::LessThan => self.compare_values(left_val, right_val, |a, b| a < b),
            BinaryOp::GreaterThanOrEquals => self.compare_values(left_val, right_val, |a, b| a >= b),
            BinaryOp::LessThanOrEquals => self.compare_values(left_val, right_val, |a, b| a <= b),
        }
    }

    fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (Value::String(a), b) => Ok(Value::String(a + &b.to_string())),
            (a, Value::String(b)) => Ok(Value::String(a.to_string() + &b)),
            (l, r) => Err(format!("Operador '+' não suportado para tipos {:?} e {:?}", l, r)),
        }
    }

    fn subtract_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (l, r) => Err(format!("Operador '-' não suportado para tipos {:?} e {:?}", l, r)),
        }
    }

    fn multiply_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (l, r) => Err(format!("Operador '*' não suportado para tipos {:?} e {:?}", l, r)),
        }
    }

    fn divide_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    Err(String::from("Erro de tempo de execução: Divisão por zero."))
                } else {
                    Ok(Value::Number(a / b))
                }
            },
            (l, r) => Err(format!("Operador '/' não suportado para tipos {:?} e {:?}", l, r)),
        }
    }

    fn compare_values<F>(&self, left: Value, right: Value, comparator: F) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(comparator(a, b))),
            (l, r) => Err(format!("Operadores de comparação não suportados para tipos {:?} e {:?}", l, r)),
        }
    }

    fn execute_conditional_statement(&mut self, conditional: ConditionalStmt) -> ControlFlow {
        match self.evaluate_expression(conditional.if_block.condition) {
            Ok(Value::Boolean(true)) => return self.execute_block(conditional.if_block.body),
            Ok(Value::Boolean(false)) => {},
            Ok(_) => return ControlFlow::Error("Condição do 'if' deve ser booleana.".to_string()),
            Err(e) => return ControlFlow::Error(e),
        }

        for block in conditional.elif_blocks {
            match self.evaluate_expression(block.condition) {
                Ok(Value::Boolean(true)) => return self.execute_block(block.body),
                Ok(Value::Boolean(false)) => {},
                Ok(_) => return ControlFlow::Error("Condição do 'elif' deve ser booleana.".to_string()),
                Err(e) => return ControlFlow::Error(e),
            }
        }

        if let Some(else_body) = conditional.else_block {
            return self.execute_block(else_body);
        }

        ControlFlow::Continue
    }

    fn execute_loop_statement(&mut self, loop_stmt: LoopStmt) -> ControlFlow {
        match loop_stmt {
            LoopStmt::While { condition, body } => {
                loop {
                    match self.evaluate_expression(condition.clone()) {
                        Ok(Value::Boolean(true)) => {
                            self.globals.enter_scope();
                            let result = self.execute_block(body.clone());
                            self.globals.exit_scope();
                            match result {
                                ControlFlow::Continue => continue,
                                ControlFlow::Return(val) => return ControlFlow::Return(val),
                                ControlFlow::Error(e) => return ControlFlow::Error(e),
                            }
                        },
                        Ok(Value::Boolean(false)) => break,
                        Ok(_) => return ControlFlow::Error("Condição do 'while' deve ser booleana.".to_string()),
                        Err(e) => return ControlFlow::Error(e),
                    }
                }
                ControlFlow::Continue
            },
            LoopStmt::For { iterator, iterable, body } => {
                let iterable_val = match self.evaluate_expression(iterable) {
                    Ok(val) => val,
                    Err(e) => return ControlFlow::Error(e),
                };

                match iterable_val {
                    Value::List(items) => {
                        for item in items {
                            self.globals.enter_scope();
                            self.globals.define(iterator.clone(), item, true, true);
                            let result = self.execute_block(body.clone());
                            self.globals.exit_scope();
                            match result {
                                ControlFlow::Continue => continue,
                                ControlFlow::Return(val) => return ControlFlow::Return(val),
                                ControlFlow::Error(e) => return ControlFlow::Error(e),
                            }
                        }
                    },
                    Value::String(s) => {
                        for c in s.chars() {
                            self.globals.enter_scope();
                            self.globals.define(iterator.clone(), Value::String(c.to_string()), true, true);
                            let result = self.execute_block(body.clone());
                            self.globals.exit_scope();
                            match result {
                                ControlFlow::Continue => continue,
                                ControlFlow::Return(val) => return ControlFlow::Return(val),
                                ControlFlow::Error(e) => return ControlFlow::Error(e),
                            }
                        }
                    },
                    _ => return ControlFlow::Error(format!("Tentativa de iterar sobre valor não-iterável: {:?}", iterable_val)),
                }
                ControlFlow::Continue
            }
        }
    }

    fn execute_block(&mut self, statements: Program) -> ControlFlow {
        for statement in statements {
            match self.execute_statement(statement) {
                ControlFlow::Continue => continue,
                ControlFlow::Return(val) => return ControlFlow::Return(val),
                ControlFlow::Error(e) => return ControlFlow::Error(e),
            }
        }
        ControlFlow::Continue
    }

    fn execute_func_declaration(&mut self, func_decl: FuncDecl) -> ControlFlow {
        self.globals.define(func_decl.name.clone(), Value::Function(func_decl), false, false);
        ControlFlow::Continue
    }

    fn evaluate_function_call(&mut self, callee: Expr, args: Vec<Expr>) -> Result<Value, String> {
        let func_val = self.evaluate_expression(callee)?;
        match func_val {
            Value::Function(func_decl) => {
                if args.len() != func_decl.params.len() {
                    return Err(format!("Número incorreto de argumentos para a função '{}'. Esperado {}, encontrado {}.", func_decl.name, func_decl.params.len(), args.len()));
                }

                self.globals.enter_scope();
                for (i, (param_name, _param_type)) in func_decl.params.iter().enumerate() {
                    let arg_value = self.evaluate_expression(args[i].clone())?;
                    self.globals.define(param_name.clone(), arg_value, false, false);
                }
                let result = self.execute_block(func_decl.body.clone());
                self.globals.exit_scope();

                match result {
                    ControlFlow::Return(val) => Ok(val),
                    ControlFlow::Error(e) => Err(e),
                    ControlFlow::Continue => Ok(Value::Nil), // Function finished without return
                }
            },
            Value::NativeFunction(func) => {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                func(evaluated_args, self)
            },
            _ => Err(format!("Tentativa de chamar um valor não-invocável: {:?}", func_val))
        }
    }

    fn execute_return_statement(&mut self, expr: Expr) -> ControlFlow {
        match self.evaluate_expression(expr) {
            Ok(value) => ControlFlow::Return(value),
            Err(e) => ControlFlow::Error(e),
        }
    }
}