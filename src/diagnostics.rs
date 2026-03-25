use crate::span::Span;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity { 
    Error, 
    Warning, 
    Note, 
    Info, 
    Hint 
}

#[derive(Debug, Clone, Serialize)]
pub struct Annotation {
    pub span: Span,
    pub message: String,
    pub is_primary: bool,
}

impl Annotation {
    pub fn primary(span: Span, message: String) -> Self {
        Self { span, message, is_primary: true }
    }

    pub fn secondary(span: Span, message: String) -> Self {
        Self { span, message, is_primary: false }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub annotations: Vec<Annotation>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: String) -> Self {
        Self {
            code: "SNASK-DIAG".to_string(),
            message,
            severity,
            annotations: vec![],
            notes: vec![],
            help: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self::new(Severity::Error, message)
    }

    pub fn warning(message: String) -> Self {
        Self::new(Severity::Warning, message)
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = code;
        self
    }

    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }
}

pub struct DiagnosticBag {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self { diagnostics: vec![] }
    }

    pub fn add(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

    pub fn render_all(&self, filename: &str, _source: &str) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            let sev_str = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Note => "note",
                Severity::Info => "info",
                Severity::Hint => "hint",
            };

            out.push_str(&format!("{}[{}]: {}\n", sev_str, d.code, d.message));
            
            for ann in &d.annotations {
                out.push_str(&format!("  --> {}:{}:{}\n", filename, ann.span.start.line, ann.span.start.column));
                if !ann.message.is_empty() {
                    out.push_str(&format!("   | {}\n", ann.message));
                }
            }

            for note in &d.notes {
                out.push_str(&format!("  = note: {}\n", note));
            }

            if let Some(help) = &d.help {
                out.push_str(&format!("  = help: {}\n", help));
            }
            out.push_str("\n");
        }
        out
    }
}

pub struct DiagnosticReporter {
    pub has_errors: bool,
}

impl DiagnosticReporter {
    pub fn new() -> Self { Self { has_errors: false } }
    
    pub fn emit(&self, d: Diagnostic) {
        let bag = DiagnosticBag { diagnostics: vec![d] };
        eprint!("{}", bag.render_all("source", ""));
    }
}
