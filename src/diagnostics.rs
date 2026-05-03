use crate::span::Span;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize)]
pub struct Annotation {
    pub span: Span,
    pub message: String,
    pub is_primary: bool,
}

impl Annotation {
    pub fn primary(span: Span, message: String) -> Self {
        Self {
            span,
            message,
            is_primary: true,
        }
    }

    pub fn secondary(span: Span, message: String) -> Self {
        Self {
            span,
            message,
            is_primary: false,
        }
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
        Self {
            diagnostics: vec![],
        }
    }

    pub fn add(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

    pub fn render_all(&self, filename: &str, source: &str) -> String {
        let mut out = String::new();
        for (idx, d) in self.diagnostics.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }
            let sev_str = match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Note => "note",
                Severity::Info => "info",
                Severity::Hint => "hint",
            };

            out.push_str(&format!("{}[{}]: {}\n", sev_str, d.code, d.message));

            for ann in &d.annotations {
                out.push_str(&format!(
                    " --> {}:{}:{}\n",
                    filename, ann.span.start.line, ann.span.start.column
                ));
                render_source_annotation(&mut out, source, ann);
            }

            for note in &d.notes {
                out.push_str(&format!("note: {}\n", note));
            }

            if let Some(help) = &d.help {
                out.push_str(&format!("help: {}\n", help));
            }
        }
        out
    }
}

fn render_source_annotation(out: &mut String, source: &str, ann: &Annotation) {
    let Some(line) = source.lines().nth(ann.span.start.line.saturating_sub(1)) else {
        if !ann.message.is_empty() {
            out.push_str(&format!("  | {}\n", ann.message));
        }
        return;
    };

    let line_no = ann.span.start.line;
    let gutter_width = line_no.to_string().len().max(1);
    out.push_str(&format!("{:>width$} |\n", "", width = gutter_width));
    out.push_str(&format!(
        "{:>width$} | {}\n",
        line_no,
        line,
        width = gutter_width
    ));

    let start_col = ann.span.start.column.max(1);
    let mut caret_len = if ann.span.start.line == ann.span.end.line {
        ann.span.end.column.saturating_sub(ann.span.start.column)
    } else {
        1
    };
    caret_len = caret_len.max(1);

    let marker = if ann.is_primary { "^" } else { "-" }.repeat(caret_len);
    let padding = " ".repeat(start_col.saturating_sub(1));
    if ann.message.is_empty() {
        out.push_str(&format!(
            "{:>width$} | {}{}\n",
            "",
            padding,
            marker,
            width = gutter_width
        ));
    } else {
        out.push_str(&format!(
            "{:>width$} | {}{} {}\n",
            "",
            padding,
            marker,
            ann.message,
            width = gutter_width
        ));
    }
}

pub fn humane_code(code: &str) -> &'static str {
    match code {
        "SNASK-PARSE-MISSING-RPAREN" => "S1002",
        "SNASK-PARSE-MISSING-RBRACKET" => "S1003",
        "SNASK-PARSE-MISSING-RBRACE" => "S1004",
        "SNASK-PARSE-INDENT" => "S1005",
        "SNASK-PARSE-SEMICOLON" => "S1006",
        "SNASK-PARSE-EXPR" => "S1010",
        "SNASK-PARSE-EXPECTED" => "S1011",
        "SNASK-PARSE-TOKENIZE" => "S1090",
        "SNASK-SEM-VAR-REDECL" => "S2001",
        "SNASK-SEM-VAR-NOT-FOUND" => "S2002",
        "SNASK-SEM-FUN-REDECL" => "S2003",
        "SNASK-SEM-FUN-NOT-FOUND" => "S2004",
        "SNASK-SEM-UNKNOWN-TYPE" => "S2005",
        "SNASK-SEM-MISSING-RETURN" => "S2006",
        "SNASK-SEM-TYPE-MISMATCH" => "S2010",
        "SNASK-SEM-INVALID-OP" => "S2011",
        "SNASK-SEM-IMMUTABLE-ASSIGN" => "S2012",
        "SNASK-SEM-RETURN-OUTSIDE" => "S2013",
        "SNASK-SEM-ARG-COUNT" => "S2020",
        "SNASK-SEM-NOT-INDEXABLE" => "S2030",
        "SNASK-SEM-INDEX-TYPE" => "S2031",
        "SNASK-SEM-PROP-NOT-FOUND" => "S2040",
        "SNASK-SEM-NOT-CALLABLE" => "S2050",
        "SNASK-SEM-RESTRICTED-NATIVE" => "S2060",
        "SNASK-BUILD-STANDARD-RUNTIME" => "S8001",
        "SNASK-BUILD-BAREMETAL-BACKEND" => "S8002",
        "SNASK-TINY-DISALLOWED-LIB" => "S9001",
        _ => "S0000",
    }
}

pub struct DiagnosticReporter {
    pub has_errors: bool,
}

impl DiagnosticReporter {
    pub fn new() -> Self {
        Self { has_errors: false }
    }

    pub fn emit(&self, d: Diagnostic) {
        let bag = DiagnosticBag {
            diagnostics: vec![d],
        };
        eprint!("{}", bag.render_all("source", ""));
    }
}

#[cfg(test)]
mod tests {
    use super::{humane_code, Annotation, Diagnostic, DiagnosticBag};
    use crate::span::{Position, Span};

    #[test]
    fn render_all_shows_source_snippet_and_caret() {
        let source = "class main\n    fun start()\n        print(\"Hello\"\n";
        let span = Span::new(Position::new(3, 22, 0), Position::new(3, 22, 0));
        let diagnostic = Diagnostic::error("missing closing `)`".to_string())
            .with_code("S1002".to_string())
            .with_annotation(Annotation::primary(span, "expected `)` here".to_string()))
            .with_help("close the function call".to_string());
        let mut bag = DiagnosticBag::new();
        bag.add(diagnostic);

        let rendered = bag.render_all("main.snask", source);

        assert!(rendered.contains("error[S1002]: missing closing `)`"));
        assert!(rendered.contains("3 |         print(\"Hello\""));
        assert!(rendered.contains("^ expected `)` here"));
        assert!(rendered.contains("help: close the function call"));
    }

    #[test]
    fn humane_code_maps_internal_codes_to_short_codes() {
        assert_eq!(humane_code("SNASK-PARSE-MISSING-RPAREN"), "S1002");
        assert_eq!(humane_code("SNASK-SEM-VAR-NOT-FOUND"), "S2002");
        assert_eq!(humane_code("UNKNOWN"), "S0000");
    }
}
