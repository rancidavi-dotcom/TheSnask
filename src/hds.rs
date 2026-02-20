use crate::diagnostics::{Annotation, Diagnostic, Severity};
use crate::span::Span;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::io::Write;

pub const QUICKFIX_THRESHOLD: u8 = 90;
pub const MAYBE_THRESHOLD: u8 = 70;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticId(pub &'static str);

#[derive(Debug, Clone, Serialize)]
pub struct Cause {
    pub title: String,
    pub detail: Option<String>,
    pub confidence: u8,
}

#[derive(Debug, Clone, Serialize)]
pub enum FixItKind {
    QuickFix,
    Refactor,
    Format,
}

#[derive(Debug, Clone, Serialize)]
pub enum FixItApply {
    /// Human-readable CLI steps (never auto-applied by compiler).
    CliSteps(Vec<String>),
    /// Placeholder for editor-driven edits (LSP). The compiler only renders this as text.
    WorkspaceEditHint(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct FixIt {
    pub title: String,
    pub confidence: u8,
    pub kind: FixItKind,
    pub apply: Option<FixItApply>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Trace {
    pub code: String,
    pub confidence_max: u8,
    pub file_ext: Option<String>,
    pub context_hash: String,
}

#[derive(Debug, Clone)]
pub struct HyperDiagnostic {
    pub id: DiagnosticId,
    pub message: String,
    pub span: Span,
    pub severity: Severity,
    pub help: Option<String>,
    pub notes: Vec<String>,
    pub causes: Vec<Cause>,
    pub fixits: Vec<FixIt>,
    pub trace: Option<Trace>,
}

impl HyperDiagnostic {
    pub fn error(id: DiagnosticId, message: String, span: Span) -> Self {
        HyperDiagnostic {
            id,
            message,
            span,
            severity: Severity::Error,
            help: None,
            notes: Vec::new(),
            causes: Vec::new(),
            fixits: Vec::new(),
            trace: None,
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

    pub fn with_cause(mut self, cause: Cause) -> Self {
        self.causes.push(cause);
        self
    }

    pub fn with_fixit(mut self, fixit: FixIt) -> Self {
        self.fixits.push(fixit);
        self
    }

    pub fn with_trace(mut self, trace: Trace) -> Self {
        self.trace = Some(trace);
        self
    }

    pub fn max_confidence(&self) -> u8 {
        self.fixits
            .iter()
            .map(|f| f.confidence)
            .max()
            .unwrap_or(0)
            .max(self.causes.iter().map(|c| c.confidence).max().unwrap_or(0))
    }

    pub fn to_renderable(&self) -> Diagnostic {
        let mut d = match self.severity {
            Severity::Error => Diagnostic::error(self.message.clone()),
            Severity::Warning => Diagnostic::warning(self.message.clone()),
            Severity::Info => Diagnostic::new(Severity::Info, self.message.clone()),
            Severity::Hint => Diagnostic::new(Severity::Hint, self.message.clone()),
        };

        d = d.with_code(self.id.0.to_string());
        d = d.with_annotation(Annotation::primary(self.span, self.message.clone()));

        for note in &self.notes {
            d = d.with_note(note.clone());
        }

        // ELI5: Why is this an error?
        if let Some(explanation) = crate::explain::get_explanation(self.id.0) {
            d = d.with_note(format!("Why is this an error? \n   {}", explanation.replace('\n', "\n   ")));
        }

        // Causes (never claim certainty).
        if !self.causes.is_empty() {
            let mut causes = self.causes.clone();
            causes.sort_by_key(|c| std::cmp::Reverse(c.confidence));
            for (i, c) in causes.iter().take(3).enumerate() {
                let mut line = format!("likely cause {} ({}%): {}", i + 1, c.confidence, c.title);
                if let Some(detail) = &c.detail {
                    line.push_str(&format!(" — {detail}"));
                }
                d = d.with_note(line);
            }
        }

        // FixIts: only show as "help" when we're very confident; otherwise as notes.
        if !self.fixits.is_empty() {
            let mut fixes = self.fixits.clone();
            fixes.sort_by_key(|f| std::cmp::Reverse(f.confidence));
            let top = &fixes[0];
            if top.confidence >= QUICKFIX_THRESHOLD {
                let mut help = format!("safe fix ({}%): {}", top.confidence, top.title);
                if let Some(apply) = &top.apply {
                    match apply {
                        FixItApply::CliSteps(steps) => {
                            for s in steps {
                                help.push_str(&format!("\n  - {s}"));
                            }
                        }
                        FixItApply::WorkspaceEditHint(h) => {
                            help.push_str(&format!("\n  - {h}"));
                        }
                    }
                }
                if let Some(existing) = &self.help {
                    help.push_str(&format!("\n\n{existing}"));
                }
                d = d.with_help(help);
            } else if top.confidence >= MAYBE_THRESHOLD {
                d = d.with_note(format!(
                    "possible fix ({}%): {}",
                    top.confidence, top.title
                ));
                if let Some(existing) = &self.help {
                    d = d.with_help(existing.clone());
                }
            } else if let Some(existing) = &self.help {
                d = d.with_help(existing.clone());
            }
        } else if let Some(existing) = &self.help {
            d = d.with_help(existing.clone());
        }

        d
    }
}

pub fn should_trace() -> bool {
    std::env::var("SNASK_HDS_TRACE").ok().as_deref() == Some("1")
}

pub fn trace_context_hash(code: &str, source: &str, span: Span) -> String {
    // Local-only, no source is stored. We hash a small window around the span to create
    // a stable-but-private context fingerprint.
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hasher.update(b"\n");
    hasher.update(span.start.line.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(span.start.column.to_string().as_bytes());
    hasher.update(b"\n");

    let lines: Vec<&str> = source.lines().collect();
    let idx = span.start.line.saturating_sub(1);
    let window_start = idx.saturating_sub(2);
    let window_end = (idx + 2).min(lines.len().saturating_sub(1));
    for i in window_start..=window_end {
        hasher.update(lines.get(i).unwrap_or(&"").as_bytes());
        hasher.update(b"\n");
    }
    let bytes = hasher.finalize();
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

pub fn write_trace(trace: &Trace) -> std::io::Result<()> {
    let base = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".snask")
        .join("diagnostics")
        .join("traces");
    fs::create_dir_all(&base)?;
    let day = chrono::Local::now().format("%Y-%m-%d").to_string();
    let path = base.join(format!("{day}.jsonl"));
    let line = serde_json::to_string(trace).unwrap_or_else(|_| "{}".to_string());
    let mut f = fs::OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(f, "{line}")?;
    Ok(())
}
