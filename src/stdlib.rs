use crate::symbol_table::SymbolTable;

/// Registra todas as funções da biblioteca padrão (Placeholder para o compilador)
pub fn register_stdlib(_globals: &mut SymbolTable) {
    // No modo compilado, a stdlib é tratada pelo SemanticAnalyzer
    // e linkada via runtime.o ou módulos .snask
}
