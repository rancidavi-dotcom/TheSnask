use crate::span::{Span, Position};
use std::fmt;

/// Nível de severidade do diagnóstico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "erro"),
            Severity::Warning => write!(f, "aviso"),
            Severity::Info => write!(f, "info"),
            Severity::Hint => write!(f, "dica"),
        }
    }
}

impl Severity {
    /// Retorna a cor ANSI para o nível de severidade
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Error => "\x1b[31m",   // Vermelho
            Severity::Warning => "\x1b[33m", // Amarelo
            Severity::Info => "\x1b[36m",    // Ciano
            Severity::Hint => "\x1b[32m",    // Verde
        }
    }

    pub fn bold_color(&self) -> &'static str {
        match self {
            Severity::Error => "\x1b[1;31m",   // Vermelho bold
            Severity::Warning => "\x1b[1;33m", // Amarelo bold
            Severity::Info => "\x1b[1;36m",    // Ciano bold
            Severity::Hint => "\x1b[1;32m",    // Verde bold
        }
    }
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

/// Representa uma anotação em um span específico
#[derive(Debug, Clone)]
pub struct Annotation {
    pub span: Span,
    pub message: Option<String>,
    pub severity: Severity,
}

impl Annotation {
    pub fn new(span: Span, message: Option<String>, severity: Severity) -> Self {
        Annotation { span, message, severity }
    }

    pub fn primary(span: Span, message: String) -> Self {
        Annotation::new(span, Some(message), Severity::Error)
    }

    pub fn secondary(span: Span, message: String) -> Self {
        Annotation::new(span, Some(message), Severity::Info)
    }
}

/// Representa um diagnóstico completo (erro, warning, etc)
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub annotations: Vec<Annotation>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: String) -> Self {
        Diagnostic {
            severity,
            code: None,
            message,
            annotations: Vec::new(),
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn error(message: String) -> Self {
        Diagnostic::new(Severity::Error, message)
    }

    pub fn warning(message: String) -> Self {
        Diagnostic::new(Severity::Warning, message)
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
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

    /// Renderiza o diagnóstico com código fonte
    pub fn render(&self, filename: &str, source: &str) -> String {
        let mut output = String::new();

        // Cabeçalho: erro[E0001]: mensagem
        output.push_str(&format!(
            "{}{}{}: {}",
            self.severity.bold_color(),
            self.severity,
            if let Some(ref code) = self.code {
                format!("[{}]", code)
            } else {
                String::new()
            },
            RESET
        ));
        output.push_str(&format!("{}{}{}\n", BOLD, self.message, RESET));

        // Localização: --> arquivo.snask:linha:coluna
        if let Some(first_annotation) = self.annotations.first() {
            output.push_str(&format!(
                "  {}--> {}{}:{}:{}\n",
                DIM,
                RESET,
                filename,
                first_annotation.span.start.line,
                first_annotation.span.start.column
            ));
        }

        // Renderizar anotações com código fonte
        for annotation in &self.annotations {
            output.push_str(&self.render_annotation(annotation, source));
        }

        // Notas adicionais
        for note in &self.notes {
            output.push_str(&format!(
                "  {}{} = nota:{} {}\n",
                DIM,
                BOLD,
                RESET,
                note
            ));
        }

        // Ajuda/sugestão
        if let Some(ref help) = self.help {
            output.push_str(&format!(
                "  {}{} = ajuda:{} {}\n",
                DIM,
                BOLD,
                RESET,
                help
            ));
        }

        output
    }

    fn render_annotation(&self, annotation: &Annotation, source: &str) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = source.lines().collect();

        let start_line = annotation.span.start.line;
        let end_line = annotation.span.end.line;

        // Calcular largura do número da linha
        let line_num_width = end_line.to_string().len();

        // Linha vazia antes
        output.push_str(&format!("  {}{:width$} |{}\n", DIM, "", RESET, width = line_num_width));

        // Renderizar linhas afetadas
        for line_num in start_line..=end_line {
            if line_num > lines.len() {
                break;
            }

            let line = lines[line_num - 1];
            
            // Número da linha e código
            output.push_str(&format!(
                "  {}{:width$} |{} {}\n",
                DIM,
                line_num,
                RESET,
                line,
                width = line_num_width
            ));

            // Indicador de erro (^^^)
            if line_num == start_line {
                let start_col = annotation.span.start.column.saturating_sub(1);
                let end_col = if start_line == end_line {
                    annotation.span.end.column.saturating_sub(1)
                } else {
                    line.len()
                };

                let indicator_len = (end_col - start_col).max(1);
                let spaces = " ".repeat(start_col);
                let indicators = "^".repeat(indicator_len);

                output.push_str(&format!(
                    "  {}{:width$} |{} {}{}{}{}",
                    DIM,
                    "",
                    RESET,
                    spaces,
                    annotation.severity.color(),
                    indicators,
                    RESET,
                    width = line_num_width
                ));

                // Mensagem da anotação
                if let Some(ref msg) = annotation.message {
                    output.push_str(&format!(" {}{}{}", annotation.severity.color(), msg, RESET));
                }

                output.push('\n');
            }
        }

        output
    }
}

/// Coletor de diagnósticos
#[derive(Debug, Default)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        DiagnosticBag {
            diagnostics: Vec::new(),
        }
    }

    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Renderiza todos os diagnósticos
    pub fn render_all(&self, filename: &str, source: &str) -> String {
        let mut output = String::new();

        for diagnostic in &self.diagnostics {
            output.push_str(&diagnostic.render(filename, source));
            output.push('\n');
        }

        // Resumo final
        if !self.diagnostics.is_empty() {
            let errors = self.error_count();
            let warnings = self.warning_count();

            if errors > 0 || warnings > 0 {
                output.push_str(&format!(
                    "{}erro{}: {} gerado{}, {} aviso{} emitido{}\n",
                    BOLD,
                    RESET,
                    errors,
                    if errors == 1 { "" } else { "s" },
                    warnings,
                    if warnings == 1 { "" } else { "s" },
                    if warnings == 1 { "" } else { "s" }
                ));
            }
        }

        output
    }
}
