use crate::snif_fmt::format_snif;
use crate::snif_parser::{parse_snif, SnifParseError, SnifValue};
use crate::snif_schema::{snask_manifest_schema_md, validate_snask_manifest, SnifSchemaError};
use crate::diagnostics::{Annotation, Diagnostic, DiagnosticBag};
use crate::span::{Position, Span};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_snask_snif_path(dir: &Path) -> Option<PathBuf> {
    let p = dir.join("snask.snif");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

pub fn read_snif_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))
}

pub fn parse_snif_file(path: &Path) -> Result<SnifValue, SnifParseError> {
    let src = fs::read_to_string(path).map_err(|e| SnifParseError {
        message: format!("Failed to read {}: {}", path.display(), e),
        line: 1,
        col: 1,
    })?;
    parse_snif(&src)
}

pub fn render_snif_parse_error(source: &str, line: usize, col: usize, message: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("SNIF parse error: {message}\n"));
    let lines: Vec<&str> = source.lines().collect();
    if line == 0 || line > lines.len() {
        return out;
    }
    let l = lines[line - 1];
    out.push_str(&format!("  --> {}:{}\n", line, col));
    out.push_str("   |\n");
    out.push_str(&format!("{:>3} | {}\n", line, l));
    let caret_pos = col.saturating_sub(1);
    let spaces = " ".repeat(caret_pos.min(l.len()));
    out.push_str(&format!("   | {}^\n", spaces));
    out
}

pub fn render_schema_errors(errs: &[SnifSchemaError]) -> String {
    let mut out = String::new();
    out.push_str("SNIF schema errors:\n");
    for e in errs {
        out.push_str(&format!("- {}: {}\n", e.path, e.message));
    }
    out
}

pub fn render_snif_parse_diagnostic(filename: &str, source: &str, e: &SnifParseError) -> String {
    let start = Position::from_line_col(e.line, e.col);
    let end = start.advance_cols(1);
    let span = Span::new(start, end);
    let d = Diagnostic::error("SNIF parse error".to_string())
        .with_code("SNIF-PARSE".to_string())
        .with_annotation(Annotation::primary(span, e.message.clone()));
    let mut bag = DiagnosticBag::new();
    bag.add(d);
    bag.render_all(filename, source)
}

pub fn render_snif_schema_diagnostic(filename: &str, source: &str, errs: &[SnifSchemaError]) -> String {
    let start = Position::from_line_col(1, 1);
    let end = start.advance_cols(1);
    let span = Span::new(start, end);
    let mut bag = DiagnosticBag::new();
    for e in errs {
        let mut d = Diagnostic::error("SNIF schema error".to_string())
            .with_code("SNIF-SCHEMA".to_string())
            .with_annotation(Annotation::primary(
                span,
                format!("{}: {}", e.path, e.message),
            ));
        if e.path == "$.package.entry" && e.message.contains("Missing required") {
            d = d.with_help("Add: package.entry = \"main.snask\"".to_string());
        } else if e.path == "$.package.name" && e.message.contains("Missing required") {
            d = d.with_help("Add: package.name = \"your_app_name\"".to_string());
        } else if e.path == "$.package.version" && e.message.contains("Missing required") {
            d = d.with_help("Add: package.version = \"0.1.0\"".to_string());
        }
        bag.add(d);
    }
    bag.render_all(filename, source)
}

pub fn format_snif_source(src: &str) -> Result<String, SnifParseError> {
    let v = parse_snif(src)?;
    Ok(format_snif(&v))
}

pub fn snif_canon_and_hash(src: &str) -> Result<(String, String), SnifParseError> {
    let canon = format_snif_source(src)?;
    let mut hasher = Sha256::new();
    hasher.update(canon.as_bytes());
    let sha = format!("{:x}", hasher.finalize());
    Ok((canon, sha))
}

pub fn validate_snask_snif(src: &str) -> Result<Vec<SnifSchemaError>, SnifParseError> {
    let v = parse_snif(src)?;
    Ok(validate_snask_manifest(&v))
}

pub fn schema_md() -> String {
    snask_manifest_schema_md()
}
