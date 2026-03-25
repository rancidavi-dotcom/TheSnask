// Proxy de migração para o SemanticAnalyzer
impl<'a> SemanticAnalyzer<'a> {
    pub fn push_error(&mut self, err: SemanticError) {
        let span = crate::diagnostics::Span { 
            line: err.span.start.line, 
            col: err.span.start.column, 
            len: 1 
        };
        self.reporter.error(err.code(), "Semantic error detected")
            .with_span(span, "Error location")
            .with_note("This code is using the legacy error model")
            .report();
    }
}
