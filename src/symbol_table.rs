use std::collections::HashMap;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub value: Value,
    pub is_mutable: bool,
    pub is_reassignable: bool, // `false` for const, `true` for `let` and `mut`
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    // A stack of scopes. The top of the stack is the current scope.
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = SymbolTable { scopes: Vec::new() };
        // Push the global scope
        table.enter_scope();
        table
    }

    // Enter a new scope (e.g., when entering a function or a block)
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    // Exit the current scope
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 { // Do not pop the global scope
            self.scopes.pop();
        }
    }

    // Define a new symbol in the current scope
    // Returns true if the symbol was successfully defined, false if it already exists in the current scope.
    pub fn define(&mut self, name: String, value: Value, is_mutable: bool, is_reassignable: bool) -> bool {
        let current_scope = self.scopes.last_mut().unwrap();
        if current_scope.contains_key(&name) {
            return false; // Symbol already exists in the current scope
        }
        current_scope.insert(name.clone(), Symbol { name, value, is_mutable, is_reassignable });
        true
    }

    // Look up a symbol, starting from the current scope and going outwards
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    // Look up a mutable symbol, starting from the current scope and going outwards
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn define_native_function(&mut self, name: &str, func: fn(Vec<Value>, &mut crate::interpreter::Interpreter) -> Result<Value, String>) {
        self.define(name.to_string(), Value::NativeFunction(func), false, false);
    }
}