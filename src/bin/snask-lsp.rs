use std::collections::HashMap;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as JsonResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use snask::parser::Parser;
use snask::semantic_analyzer::SemanticAnalyzer;
use snask::span as snask_span;
use snask::ast::{Program, StmtKind};
use snask::parser::Token as SnaskToken;
use snask::snif_parser::parse_snif;
use snask::snif_schema::validate_snask_manifest;
use snask::snif_fmt::format_snif;

#[derive(Default, Clone)]
struct Document {
    text: String,
    version: i32,
}

#[derive(Debug, Clone)]
enum SymbolKind {
    Function,
    Variable,
    Constant,
    Class,
    Parameter,
    Import,
    Module,
}

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    kind: SymbolKind,
    span: snask_span::Span,
    uri: Url,
}

#[derive(Default, Debug, Clone)]
struct FileSymbols {
    symbols: Vec<Symbol>,
}

#[derive(Default)]
struct State {
    docs: HashMap<Url, Document>,
    symbols: HashMap<Url, FileSymbols>,
    workspace_roots: Vec<PathBuf>,
}

struct Backend {
    client: Client,
    state: Arc<RwLock<State>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(State::default())),
        }
    }

    async fn set_doc(&self, uri: Url, text: String, version: i32) {
        let mut st = self.state.write().await;
        st.docs.insert(uri, Document { text, version });
    }

    async fn set_symbols(&self, uri: Url, symbols: FileSymbols) {
        let mut st = self.state.write().await;
        st.symbols.insert(uri, symbols);
    }

    async fn get_doc(&self, uri: &Url) -> Option<Document> {
        let st = self.state.read().await;
        st.docs.get(uri).cloned()
    }

    async fn find_symbol(&self, name: &str, preferred_uri: &Url) -> Option<Symbol> {
        let st = self.state.read().await;
        if let Some(fs) = st.symbols.get(preferred_uri) {
            if let Some(sym) = fs.symbols.iter().find(|s| s.name == name) {
                return Some(sym.clone());
            }
        }
        for (_uri, fs) in &st.symbols {
            if let Some(sym) = fs.symbols.iter().find(|s| s.name == name) {
                return Some(sym.clone());
            }
        }
        None
    }

    fn should_skip_dir(name: &str) -> bool {
        matches!(name, ".git" | "target" | "dist" | "build" | "node_modules" | ".snask")
    }

    fn scan_snask_files(root: &Path) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = Vec::new();
        let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let entries = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        if Self::should_skip_dir(name) {
                            continue;
                        }
                    }
                    stack.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("snask") {
                    out.push(path);
                }
            }
        }
        out
    }

    async fn index_workspace_static(state: Arc<RwLock<State>>, roots: Vec<PathBuf>) {
        {
            let mut st = state.write().await;
            st.workspace_roots = roots.clone();
        }

        for root in roots {
            for file in Self::scan_snask_files(&root) {
                let Ok(text) = fs::read_to_string(&file) else { continue };
                let Ok(uri) = Url::from_file_path(&file) else { continue };
                if let Ok(program) = Parser::new(&text).and_then(|mut p| p.parse_program()) {
                    let symbols = Self::collect_symbols(&uri, &program);
                    let mut st = state.write().await;
                    st.symbols.insert(uri, symbols);
                }
            }
        }
    }

    fn identifier_at(text: &str, pos: Position) -> Option<String> {
        let line = text.lines().nth(pos.line as usize)?;
        let bytes = line.as_bytes();
        let mut i = pos.character as usize;
        if i > bytes.len() {
            i = bytes.len();
        }

        let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b':';
        if i < bytes.len() && !is_ident(bytes[i]) && i > 0 && is_ident(bytes[i - 1]) {
            i -= 1;
        }
        if i >= bytes.len() || !is_ident(bytes[i]) {
            return None;
        }

        let mut start = i;
        while start > 0 && is_ident(bytes[start - 1]) {
            start -= 1;
        }
        let mut end = i;
        while end < bytes.len() && is_ident(bytes[end]) {
            end += 1;
        }
        Some(line[start..end].to_string())
    }

    fn span_to_range(span: &snask_span::Span) -> Range {
        let start_line = span.start.line.saturating_sub(1) as u32;
        let start_col = span.start.column.saturating_sub(1) as u32;
        let end_line = span.end.line.saturating_sub(1) as u32;
        let mut end_col = span.end.column.saturating_sub(1) as u32;

        if start_line == end_line && end_col <= start_col {
            end_col = start_col.saturating_add(1);
        }

        Range {
            start: Position {
                line: start_line,
                character: start_col,
            },
            end: Position {
                line: end_line,
                character: end_col,
            },
        }
    }

    fn collect_symbols(uri: &Url, program: &Program) -> FileSymbols {
        let mut out = FileSymbols::default();
        for stmt in program {
            match &stmt.kind {
                StmtKind::VarDeclaration(d) => out.symbols.push(Symbol {
                    name: d.name.clone(),
                    kind: SymbolKind::Variable,
                    span: stmt.span.clone(),
                    uri: uri.clone(),
                }),
                StmtKind::MutDeclaration(d) => out.symbols.push(Symbol {
                    name: d.name.clone(),
                    kind: SymbolKind::Variable,
                    span: stmt.span.clone(),
                    uri: uri.clone(),
                }),
                StmtKind::ConstDeclaration(d) => out.symbols.push(Symbol {
                    name: d.name.clone(),
                    kind: SymbolKind::Constant,
                    span: stmt.span.clone(),
                    uri: uri.clone(),
                }),
                StmtKind::FuncDeclaration(f) => {
                    out.symbols.push(Symbol {
                        name: f.name.clone(),
                        kind: SymbolKind::Function,
                        span: stmt.span.clone(),
                        uri: uri.clone(),
                    });
                    for (param, _ty) in &f.params {
                        out.symbols.push(Symbol {
                            name: param.clone(),
                            kind: SymbolKind::Parameter,
                            span: stmt.span.clone(),
                            uri: uri.clone(),
                        });
                    }
                }
                StmtKind::ClassDeclaration(c) => out.symbols.push(Symbol {
                    name: c.name.clone(),
                    kind: SymbolKind::Class,
                    span: stmt.span.clone(),
                    uri: uri.clone(),
                }),
                StmtKind::Import(path) => out.symbols.push(Symbol {
                    name: path.clone(),
                    kind: SymbolKind::Import,
                    span: stmt.span.clone(),
                    uri: uri.clone(),
                }),
                StmtKind::FromImport { module, .. } => out.symbols.push(Symbol {
                    name: module.clone(),
                    kind: SymbolKind::Module,
                    span: stmt.span.clone(),
                    uri: uri.clone(),
                }),
                _ => {}
            }
        }
        out
    }

    fn span_contains(span: &snask_span::Span, pos: Position) -> bool {
        let line = (pos.line as usize).saturating_add(1);
        let col = (pos.character as usize).saturating_add(1);
        let after_start = (line, col) >= (span.start.line, span.start.column);
        let before_end = (line, col) <= (span.end.line, span.end.column);
        after_start && before_end
    }

    fn collect_locals_in_stmts(stmts: &[snask::ast::Stmt], out: &mut Vec<(String, SymbolKind)>) {
        for s in stmts {
            match &s.kind {
                StmtKind::VarDeclaration(d) => out.push((d.name.clone(), SymbolKind::Variable)),
                StmtKind::MutDeclaration(d) => out.push((d.name.clone(), SymbolKind::Variable)),
                StmtKind::ConstDeclaration(d) => {
                    out.push((d.name.clone(), SymbolKind::Constant));
                }
                StmtKind::Conditional(c) => {
                    Self::collect_locals_in_stmts(&c.if_block.body, out);
                    for b in &c.elif_blocks {
                        Self::collect_locals_in_stmts(&b.body, out);
                    }
                    if let Some(b) = &c.else_block {
                        Self::collect_locals_in_stmts(b, out);
                    }
                }
                StmtKind::Loop(l) => match l {
                    snask::ast::LoopStmt::While { body, .. } => Self::collect_locals_in_stmts(body, out),
                    snask::ast::LoopStmt::For { iterator, body, .. } => {
                        out.push((iterator.clone(), SymbolKind::Variable));
                        Self::collect_locals_in_stmts(body, out)
                    }
                },
                _ => {}
            }
        }
    }

    fn locals_for_position(program: &Program, pos: Position) -> Vec<(String, SymbolKind)> {
        for stmt in program {
            if let StmtKind::FuncDeclaration(f) = &stmt.kind {
                if Self::span_contains(&stmt.span, pos) {
                    let mut out: Vec<(String, SymbolKind)> = Vec::new();
                    for (p, _ty) in &f.params {
                        out.push((p.clone(), SymbolKind::Parameter));
                    }
                    Self::collect_locals_in_stmts(&f.body, &mut out);
                    return out;
                }
            }
        }
        Vec::new()
    }

    fn line_indent(s: &str) -> String {
        s.chars().take_while(|c| *c == ' ' || *c == '\t').collect()
    }

    fn mk_workspace_edit(uri: Url, edits: Vec<TextEdit>) -> WorkspaceEdit {
        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        changes.insert(uri, edits);
        WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }
    }

    async fn publish_diagnostics(&self, uri: Url, text: String, version: i32) {
        let mut diags: Vec<Diagnostic> = Vec::new();

        let is_snif = uri.path().ends_with(".snif");
        if is_snif {
            self.set_symbols(uri.clone(), FileSymbols::default()).await;
            match parse_snif(&text) {
                Ok(v) => {
                    if uri.path().ends_with("/snask.snif") || uri.path().ends_with("snask.snif") {
                        let errs = validate_snask_manifest(&v);
                        for e in errs {
                            diags.push(Diagnostic {
                                range: Range {
                                    start: Position { line: 0, character: 0 },
                                    end: Position { line: 0, character: 1 },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: None,
                                code_description: None,
                                source: Some("snif".to_string()),
                                message: format!("{}: {}", e.path, e.message),
                                related_information: None,
                                tags: None,
                                data: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    let line0 = (e.line as u32).saturating_sub(1);
                    let col0 = (e.col as u32).saturating_sub(1);
                    diags.push(Diagnostic {
                        range: Range {
                            start: Position { line: line0, character: col0 },
                            end: Position { line: line0, character: col0.saturating_add(1) },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("snif".to_string()),
                        message: format!("SNIF parse error: {}", e.message),
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
        } else {
        match Parser::new(&text).and_then(|mut p| p.parse_program()) {
            Ok(program) => {
                let symbols = Self::collect_symbols(&uri, &program);
                self.set_symbols(uri.clone(), symbols).await;

                let mut analyzer = SemanticAnalyzer::new();
                analyzer.analyze(&program);
                for err in analyzer.errors {
                    let mut message = err.message();
                    for n in &err.notes {
                        message.push_str(&format!("\n\nnote: {}", n));
                    }
                    if let Some(h) = &err.help {
                        message.push_str(&format!("\n\nhelp: {}", h));
                    }
                    diags.push(Diagnostic {
                        range: Self::span_to_range(&err.span),
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("SNASK-SEM".to_string())),
                        code_description: None,
                        source: Some("snask".to_string()),
                        message,
                        related_information: None,
                        tags: None,
                        data: None,
                    });
                }
            }
            Err(err) => {
                self.set_symbols(uri.clone(), FileSymbols::default()).await;
                let range = Self::span_to_range(&err.span);
                let mut message = err.message.clone();
                if let Some(h) = &err.help {
                    message.push_str(&format!("\n\nhelp: {}", h));
                }
                diags.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(err.code.to_string())),
                    code_description: None,
                    source: Some("snask".to_string()),
                    message,
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }
        }

        self.client
            .publish_diagnostics(uri, diags, Some(version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> JsonResult<InitializeResult> {
        let legend = SemanticTokensLegend {
            token_types: vec![
                SemanticTokenType::KEYWORD,
                SemanticTokenType::FUNCTION,
                SemanticTokenType::VARIABLE,
                SemanticTokenType::TYPE,
                SemanticTokenType::STRING,
                SemanticTokenType::NUMBER,
                SemanticTokenType::OPERATOR,
                SemanticTokenType::NAMESPACE,
            ],
            token_modifiers: vec![],
        };

        // Kick off workspace indexing (best-effort).
        let roots: Vec<PathBuf> = if let Some(folders) = &params.workspace_folders {
            folders
                .iter()
                .filter_map(|f| f.uri.to_file_path().ok())
                .collect()
        } else if let Some(root_uri) = &params.root_uri {
            root_uri.to_file_path().ok().map(|p| vec![p]).unwrap_or_default()
        } else {
            vec![]
        };
        let state = self.state.clone();
        tokio::spawn(async move {
            Backend::index_workspace_static(state, roots).await;
        });

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![":".to_string(), ".".to_string(), "\"".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                        legend,
                        range: Some(false),
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "snask-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "snask-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> JsonResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.set_doc(doc.uri.clone(), doc.text.clone(), doc.version)
            .await;
        self.publish_diagnostics(doc.uri, doc.text, doc.version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        let mut text = if let Some(d) = self.get_doc(&uri).await {
            d.text
        } else {
            String::new()
        };

        for change in params.content_changes {
            if let Some(range) = change.range {
                // Minimal incremental apply (UTF-16 positions are complex; this is best-effort).
                // If it fails, fallback to full text.
                if range.start.line == 0 && range.start.character == 0 && range.end.line == 0 && range.end.character == 0 {
                    text = change.text;
                } else {
                    // Fallback: treat as full text (VS Code usually sends full text if configured).
                    text = change.text;
                }
            } else {
                text = change.text;
            }
        }

        self.set_doc(uri.clone(), text.clone(), version).await;
        self.publish_diagnostics(uri, text, version).await;
    }

    async fn hover(&self, params: HoverParams) -> JsonResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let doc = match self.get_doc(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let ident = Self::identifier_at(&doc.text, pos);
        let msg = if let Some(name) = ident {
            if let Some(sym) = self.find_symbol(&name, &uri).await {
                let kind = match sym.kind {
                    SymbolKind::Function => "function",
                    SymbolKind::Variable => "variable",
                    SymbolKind::Constant => "constant",
                    SymbolKind::Class => "class",
                    SymbolKind::Parameter => "parameter",
                    SymbolKind::Import => "import",
                    SymbolKind::Module => "module",
                };
                let line = sym.span.start.line;
                let col = sym.span.start.column;
                format!("**{}** `{}`\n\nDefined at {}:{}", kind, sym.name, line, col)
            } else {
                format!("`{}`", name)
            }
        } else {
            let line = doc.text.lines().nth(pos.line as usize).unwrap_or("");
            format!("Snask (v{})\n\n`{}`", env!("CARGO_PKG_VERSION"), line.trim())
        };

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: msg,
            }),
            range: None,
        }))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> JsonResult<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let doc = match self.get_doc(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };
        let ident = match Self::identifier_at(&doc.text, pos) {
            Some(s) => s,
            None => return Ok(None),
        };
        let sym = match self.find_symbol(&ident, &uri).await {
            Some(s) => s,
            None => return Ok(None),
        };
        Ok(Some(GotoDefinitionResponse::Scalar(Location {
            uri: sym.uri,
            range: Self::span_to_range(&sym.span),
        })))
    }

    async fn completion(&self, params: CompletionParams) -> JsonResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let doc = match self.get_doc(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let mut items: Vec<CompletionItem> = Vec::new();

        // Keywords/snippets
        items.extend([
            ("class main", "class main\n    fun start()\n        ", CompletionItemKind::KEYWORD),
            ("fun", "fun ", CompletionItemKind::KEYWORD),
            ("let", "let ", CompletionItemKind::KEYWORD),
            ("mut", "mut ", CompletionItemKind::KEYWORD),
            ("const", "const ", CompletionItemKind::KEYWORD),
            ("if", "if ", CompletionItemKind::KEYWORD),
            ("elif", "elif ", CompletionItemKind::KEYWORD),
            ("else", "else", CompletionItemKind::KEYWORD),
            ("while", "while ", CompletionItemKind::KEYWORD),
            ("for", "for ", CompletionItemKind::KEYWORD),
            ("import", "import \"\";\n", CompletionItemKind::KEYWORD),
            ("from / import", "from / import ", CompletionItemKind::KEYWORD),
        ].into_iter().map(|(label, insert, kind)| CompletionItem {
            label: label.to_string(),
            kind: Some(kind),
            insert_text: Some(insert.to_string()),
            ..Default::default()
        }));

        // Parse current doc to get locals at position (best-effort).
        if let Ok(program) = Parser::new(&doc.text).and_then(|mut p| p.parse_program()) {
            let locals = Self::locals_for_position(&program, pos);
            for (name, kind) in locals {
                let k = match kind {
                    SymbolKind::Function => CompletionItemKind::FUNCTION,
                    SymbolKind::Variable => CompletionItemKind::VARIABLE,
                    SymbolKind::Constant => CompletionItemKind::CONSTANT,
                    SymbolKind::Class => CompletionItemKind::CLASS,
                    SymbolKind::Parameter => CompletionItemKind::VARIABLE,
                    SymbolKind::Import => CompletionItemKind::MODULE,
                    SymbolKind::Module => CompletionItemKind::MODULE,
                };
                items.push(CompletionItem {
                    label: name,
                    kind: Some(k),
                    ..Default::default()
                });
            }
        }

        // Workspace-open files symbols
        let st = self.state.read().await;
        for (_u, fs) in &st.symbols {
            for s in &fs.symbols {
                let k = match s.kind {
                    SymbolKind::Function => CompletionItemKind::FUNCTION,
                    SymbolKind::Variable => CompletionItemKind::VARIABLE,
                    SymbolKind::Constant => CompletionItemKind::CONSTANT,
                    SymbolKind::Class => CompletionItemKind::CLASS,
                    SymbolKind::Parameter => CompletionItemKind::VARIABLE,
                    SymbolKind::Import => CompletionItemKind::MODULE,
                    SymbolKind::Module => CompletionItemKind::MODULE,
                };
                items.push(CompletionItem {
                    label: s.name.clone(),
                    kind: Some(k),
                    ..Default::default()
                });
            }
        }

        // De-dup by label
        items.sort_by(|a, b| a.label.cmp(&b.label));
        items.dedup_by(|a, b| a.label == b.label);

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn semantic_tokens_full(&self, params: SemanticTokensParams) -> JsonResult<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;
        let doc = match self.get_doc(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let is_snif = uri.path().ends_with(".snif");
        if is_snif {
            // Minimal SNIF lexer for semantic tokens (best-effort).
            let mut data: Vec<SemanticToken> = Vec::new();
            let mut prev_line: u32 = 0;
            let mut prev_col: u32 = 0;

            let bytes = doc.text.as_bytes();
            let mut i: usize = 0;
            let mut line: u32 = 0;
            let mut col: u32 = 0;
            let push_tok = |data: &mut Vec<SemanticToken>,
                            prev_line: &mut u32,
                            prev_col: &mut u32,
                            line: u32,
                            col: u32,
                            len: u32,
                            tok_type: u32| {
                let delta_line = line.saturating_sub(*prev_line);
                let delta_start = if delta_line == 0 {
                    col.saturating_sub(*prev_col)
                } else {
                    col
                };
                data.push(SemanticToken {
                    delta_line,
                    delta_start,
                    length: len,
                    token_type: tok_type,
                    token_modifiers_bitset: 0,
                });
                *prev_line = line;
                *prev_col = col;
            };

            while i < bytes.len() {
                let b = bytes[i];

                // Newline
                if b == b'\n' {
                    line += 1;
                    col = 0;
                    i += 1;
                    continue;
                }

                // Comments //
                if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    while i < bytes.len() && bytes[i] != b'\n' {
                        i += 1;
                    }
                    continue;
                }

                // Strings
                if b == b'"' || b == b'\'' {
                    let q = b;
                    let start_col = col;
                    i += 1;
                    col += 1;
                    while i < bytes.len() {
                        let c = bytes[i];
                        if c == b'\n' {
                            break;
                        }
                        if c == b'\\' {
                            // escape
                            i += 2;
                            col += 2;
                            continue;
                        }
                        i += 1;
                        col += 1;
                        if c == q {
                            break;
                        }
                    }
                    let len = col.saturating_sub(start_col).max(1);
                    push_tok(&mut data, &mut prev_line, &mut prev_col, line, start_col, len, 4);
                    continue;
                }

                // Typed literal: @type"payload"
                if b == b'@' {
                    push_tok(&mut data, &mut prev_line, &mut prev_col, line, col, 1, 6);
                    i += 1;
                    col += 1;
                    let start = i;
                    let start_col = col;
                    while i < bytes.len() {
                        let c = bytes[i];
                        let ok = (c as char).is_ascii_alphanumeric() || c == b'_' || c == b'$' || c == b'-';
                        if !ok {
                            break;
                        }
                        i += 1;
                        col += 1;
                    }
                    if i > start {
                        let len = (i - start) as u32;
                        push_tok(&mut data, &mut prev_line, &mut prev_col, line, start_col, len, 3);
                    }
                    continue;
                }

                // Numbers
                if (b as char).is_ascii_digit() || b == b'-' {
                    let start_col = col;
                    let start_i = i;
                    i += 1;
                    col += 1;
                    while i < bytes.len() {
                        let c = bytes[i];
                        let ok = (c as char).is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'+' || c == b'-';
                        if !ok {
                            break;
                        }
                        i += 1;
                        col += 1;
                    }
                    let len = (i - start_i) as u32;
                    push_tok(&mut data, &mut prev_line, &mut prev_col, line, start_col, len.max(1), 5);
                    continue;
                }

                // Identifiers (keys)
                if (b as char).is_ascii_alphabetic() || b == b'_' || b == b'$' {
                    let start_col = col;
                    let start_i = i;
                    i += 1;
                    col += 1;
                    while i < bytes.len() {
                        let c = bytes[i];
                        let ok = (c as char).is_ascii_alphanumeric() || c == b'_' || c == b'$' || c == b'-';
                        if !ok {
                            break;
                        }
                        i += 1;
                        col += 1;
                    }
                    let len = (i - start_i) as u32;
                    // If next non-ws is ':', treat as object key (namespace-ish).
                    let mut j = i;
                    while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\r') {
                        j += 1;
                    }
                    let tok_type = if j < bytes.len() && bytes[j] == b':' { 7 } else { 2 };
                    push_tok(&mut data, &mut prev_line, &mut prev_col, line, start_col, len.max(1), tok_type);
                    continue;
                }

                // Operators / punctuation
                if b"{}[]:,.".contains(&b) {
                    push_tok(&mut data, &mut prev_line, &mut prev_col, line, col, 1, 6);
                }

                i += 1;
                col += 1;
            }

            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data,
            })));
        }

        let tokens = match snask::parser::tokenize(&doc.text) {
            Ok(t) => t,
            Err(_) => return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens { result_id: None, data: vec![] }))),
        };

        let mut data: Vec<SemanticToken> = Vec::new();
        let mut prev_line: u32 = 0;
        let mut prev_col: u32 = 0;

        let mut expect_ident_as: Option<u32> = None; // token_type index

        for t in tokens {
            let loc = t.get_location().clone();
            let line = loc.line.saturating_sub(1) as u32;
            let col = loc.column.saturating_sub(1) as u32;

            let (len, tok_type): (u32, Option<u32>) = match t.clone() {
                // Keywords
                SnaskToken::Let(_) => (3, Some(0)),
                SnaskToken::Mut(_) => (3, Some(0)),
                SnaskToken::Const(_) => (5, Some(0)),
                SnaskToken::Print(_) => (5, Some(0)),
                SnaskToken::Input(_) => (5, Some(0)),
                SnaskToken::Fun(_) => (3, Some(0)),
                SnaskToken::Class(_) => (5, Some(0)),
                SnaskToken::SelfKw(_) => (4, Some(0)),
                SnaskToken::Return(_) => (6, Some(0)),
                SnaskToken::If(_) => (2, Some(0)),
                SnaskToken::Elif(_) => (4, Some(0)),
                SnaskToken::Else(_) => (4, Some(0)),
                SnaskToken::While(_) => (5, Some(0)),
                SnaskToken::For(_) => (3, Some(0)),
                SnaskToken::In(_) => (2, Some(0)),
                SnaskToken::Import(_) => (6, Some(0)),
                SnaskToken::From(_) => (4, Some(0)),
                SnaskToken::True(_) => (4, Some(0)),
                SnaskToken::False(_) => (5, Some(0)),
                SnaskToken::Nil(_) => (3, Some(0)),
                SnaskToken::And(_) => (3, Some(0)),
                SnaskToken::Or(_) => (2, Some(0)),
                SnaskToken::Not(_) => (3, Some(0)),

                // Identifiers / literals
                SnaskToken::Identifier(name, _) => {
                    let ty = expect_ident_as.take().unwrap_or(2); // default variable
                    (name.len().max(1) as u32, Some(ty))
                }
                SnaskToken::Number(n, _) => (format!("{}", n).len().max(1) as u32, Some(5)),
                SnaskToken::String(s, _) => ((s.len() + 2).max(1) as u32, Some(4)),

                // Operators/delims
                SnaskToken::Plus(_) | SnaskToken::Minus(_) | SnaskToken::Star(_) | SnaskToken::Slash(_)
                | SnaskToken::Equal(_) | SnaskToken::Less(_) | SnaskToken::Greater(_)
                | SnaskToken::Comma(_) | SnaskToken::Dot(_) | SnaskToken::Colon(_)
                | SnaskToken::Semicolon(_) => (1, Some(6)),
                SnaskToken::DoubleSlash(_) | SnaskToken::PlusEqual(_) | SnaskToken::MinusEqual(_)
                | SnaskToken::StarEqual(_) | SnaskToken::SlashEqual(_) | SnaskToken::EqualEqual(_)
                | SnaskToken::BangEqual(_) | SnaskToken::LessEqual(_) | SnaskToken::GreaterEqual(_)
                | SnaskToken::DoubleColon(_) => (2, Some(6)),
                SnaskToken::TripleEqual(_) => (3, Some(6)),

                // Brackets/parens
                SnaskToken::LeftParen(_) | SnaskToken::RightParen(_)
                | SnaskToken::LeftBrace(_) | SnaskToken::RightBrace(_)
                | SnaskToken::LeftBracket(_) | SnaskToken::RightBracket(_) => (1, Some(6)),

                // Whitespace / structural
                SnaskToken::Indent(_) | SnaskToken::Dedent(_) | SnaskToken::Newline(_) | SnaskToken::Eof(_) => (0, None),
                SnaskToken::List(_) | SnaskToken::Dict(_) => (4, Some(0)),
            };

            // Update "expect next identifier" state based on current token
            match t {
                SnaskToken::Fun(_) => expect_ident_as = Some(1),       // function
                SnaskToken::Class(_) => expect_ident_as = Some(3),     // type
                SnaskToken::Let(_) | SnaskToken::Mut(_) => expect_ident_as = Some(2), // variable
                SnaskToken::Const(_) => expect_ident_as = Some(2),
                SnaskToken::From(_) | SnaskToken::Import(_) => expect_ident_as = Some(7), // namespace/module-ish
                _ => {}
            }

            let Some(tok_type) = tok_type else { continue };
            if len == 0 {
                continue;
            }

            let delta_line = line.saturating_sub(prev_line);
            let delta_start = if delta_line == 0 {
                col.saturating_sub(prev_col)
            } else {
                col
            };

            data.push(SemanticToken {
                delta_line,
                delta_start,
                length: len,
                token_type: tok_type,
                token_modifiers_bitset: 0,
            });

            prev_line = line;
            prev_col = col;
        }

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }

    async fn code_action(&self, params: CodeActionParams) -> JsonResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let doc = match self.get_doc(&uri).await {
            Some(d) => d,
            None => return Ok(None),
        };

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        let is_snif = uri.path().ends_with(".snif");
        if is_snif {
            // Format SNIF (canonical) — rewrite full document.
            if let Ok(v) = parse_snif(&doc.text) {
                let formatted = format_snif(&v);
                let v_clone = v.clone();
                let last_line = doc.text.lines().count().saturating_sub(1) as u32;
                let last_col = doc
                    .text
                    .lines()
                    .last()
                    .map(|l| l.len() as u32)
                    .unwrap_or(0);
                let edit = TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: last_line, character: last_col },
                    },
                    new_text: formatted,
                };
                let ca = CodeAction {
                    title: "Format SNIF (canonical)".to_string(),
                    kind: Some(CodeActionKind::SOURCE),
                    diagnostics: None,
                    edit: Some(Self::mk_workspace_edit(uri.clone(), vec![edit])),
                    is_preferred: Some(true),
                    ..Default::default()
                };
                actions.push(CodeActionOrCommand::CodeAction(ca));

                // Fix common schema issue: missing package.entry in snask.snif
                if uri.path().ends_with("/snask.snif") || uri.path().ends_with("snask.snif") {
                    let errs = validate_snask_manifest(&v_clone);
                    let missing_entry = errs.iter().any(|e| e.path == "$.package.entry");
                    if missing_entry {
                        if let snask::snif_parser::SnifValue::Object(mut root) = v_clone {
                            if let Some(snask::snif_parser::SnifValue::Object(mut pkg)) = root.remove("package") {
                                if !pkg.contains_key("entry") {
                                    pkg.insert(
                                        "entry".to_string(),
                                        snask::snif_parser::SnifValue::String("main.snask".to_string()),
                                    );
                                    root.insert("package".to_string(), snask::snif_parser::SnifValue::Object(pkg));
                                    let fixed = format_snif(&snask::snif_parser::SnifValue::Object(root));
                                    let edit = TextEdit {
                                        range: Range {
                                            start: Position { line: 0, character: 0 },
                                            end: Position { line: last_line, character: last_col },
                                        },
                                        new_text: fixed,
                                    };
                                    let ca = CodeAction {
                                        title: "Add package.entry = \"main.snask\"".to_string(),
                                        kind: Some(CodeActionKind::QUICKFIX),
                                        diagnostics: None,
                                        edit: Some(Self::mk_workspace_edit(uri.clone(), vec![edit])),
                                        is_preferred: Some(false),
                                        ..Default::default()
                                    };
                                    actions.push(CodeActionOrCommand::CodeAction(ca));
                                }
                            }
                        }
                    }
                }
            }
        }

        for diag in params.context.diagnostics {
            let msg = diag.message.clone();

            // Quickfix: missing semicolon (parser hint)
            let is_missing_semicolon = match &diag.code {
                Some(NumberOrString::String(s)) => s == "SNASK-PARSE-SEMICOLON",
                _ => false,
            };
            if is_missing_semicolon || msg.contains("missed a ';'") {
                let line_idx = diag.range.start.line as usize;
                if let Some(line) = doc.text.lines().nth(line_idx) {
                    if !line.trim_end().ends_with(';') {
                        let insert_col = line.len() as u32;
                        let edit = TextEdit {
                            range: Range {
                                start: Position {
                                    line: diag.range.start.line,
                                    character: insert_col,
                                },
                                end: Position {
                                    line: diag.range.start.line,
                                    character: insert_col,
                                },
                            },
                            new_text: ";".to_string(),
                        };
                        let ca = CodeAction {
                            title: "Insert ';'".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diag.clone()]),
                            edit: Some(Self::mk_workspace_edit(uri.clone(), vec![edit])),
                            is_preferred: Some(true),
                            ..Default::default()
                        };
                        actions.push(CodeActionOrCommand::CodeAction(ca));
                    }
                }
            }

            // Quickfix: missing closing delimiters.
            let missing_closer = match &diag.code {
                Some(NumberOrString::String(s)) if s == "SNASK-PARSE-MISSING-RPAREN" => Some(")"),
                Some(NumberOrString::String(s)) if s == "SNASK-PARSE-MISSING-RBRACKET" => Some("]"),
                Some(NumberOrString::String(s)) if s == "SNASK-PARSE-MISSING-RBRACE" => Some("}"),
                _ => None,
            };
            if let Some(ch) = missing_closer {
                // Insert at end of the line where the error is reported (best-effort).
                let line_idx = diag.range.start.line as usize;
                if let Some(line) = doc.text.lines().nth(line_idx) {
                    let insert_col = line.len() as u32;
                    let edit = TextEdit {
                        range: Range {
                            start: Position {
                                line: diag.range.start.line,
                                character: insert_col,
                            },
                            end: Position {
                                line: diag.range.start.line,
                                character: insert_col,
                            },
                        },
                        new_text: ch.to_string(),
                    };
                    let ca = CodeAction {
                        title: format!("Insert '{ch}'"),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diag.clone()]),
                        edit: Some(Self::mk_workspace_edit(uri.clone(), vec![edit])),
                        is_preferred: Some(true),
                        ..Default::default()
                    };
                    actions.push(CodeActionOrCommand::CodeAction(ca));
                }
            }

            // Quickfix: create variable when "Variable 'x' not found."
            if let Some(rest) = msg.strip_prefix("Variable '") {
                if let Some((name, _)) = rest.split_once("' not found.") {
                    let line_idx = diag.range.start.line as usize;
                    let line = doc.text.lines().nth(line_idx).unwrap_or("");
                    let indent = Self::line_indent(line);
                    let insert_pos = Position {
                        line: diag.range.start.line,
                        character: 0,
                    };
                    let new_text = format!("{indent}let {name} = nil;\n");
                    let edit = TextEdit {
                        range: Range {
                            start: insert_pos,
                            end: insert_pos,
                        },
                        new_text,
                    };
                    let ca = CodeAction {
                        title: format!("Create variable '{name}'"),
                        kind: Some(CodeActionKind::QUICKFIX),
                        diagnostics: Some(vec![diag.clone()]),
                        edit: Some(Self::mk_workspace_edit(uri.clone(), vec![edit])),
                        is_preferred: Some(false),
                        ..Default::default()
                    };
                    actions.push(CodeActionOrCommand::CodeAction(ca));
                }
            }

            // Quickfix: insert import for restricted native help
            if msg.starts_with("Direct use of native '") && msg.contains("Import the library: import \"") {
                if let Some(idx) = msg.find("import \"") {
                    let after = &msg[idx + "import \"".len()..];
                    if let Some((lib, _)) = after.split_once("\"") {
                        let edit = TextEdit {
                            range: Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: 0 },
                            },
                            new_text: format!("import \"{lib}\";\n"),
                        };
                        let ca = CodeAction {
                            title: format!("Add import \"{lib}\""),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diag.clone()]),
                            edit: Some(Self::mk_workspace_edit(uri.clone(), vec![edit])),
                            is_preferred: Some(true),
                            ..Default::default()
                        };
                        actions.push(CodeActionOrCommand::CodeAction(ca));
                    }
                }
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
