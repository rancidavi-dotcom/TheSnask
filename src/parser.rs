use crate::ast::{
    Program, Stmt, StmtKind, Expr, ExprKind, VarDecl, MutDecl, ConstDecl, LiteralValue, 
    BinaryOp, UnaryOp, ConditionalStmt, IfBlock, LoopStmt, FuncDecl, Location
};
use crate::span::{Position, Span};
use crate::types::Type;
use std::iter::Peekable;
use std::str::FromStr;
use std::str::Chars;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
    pub notes: Vec<String>,
}

impl ParseError {
    pub fn new(code: &'static str, message: String, span: Span) -> Self {
        ParseError {
            code,
            message,
            span,
            help: None,
            notes: Vec::new(),
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
}

type ParseResult<T> = Result<T, ParseError>;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}



#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Let(Location),
    Mut(Location),
    Const(Location),
    Print(Location),
    Input(Location),
    Fun(Location),
    Class(Location),
    SelfKw(Location),
    Return(Location),
    If(Location),
    Elif(Location),
    Else(Location),
    While(Location),
    For(Location),
    In(Location),
    List(Location),
    Dict(Location),
    Import(Location),
    From(Location),
    True(Location),
    False(Location),
    Nil(Location),
    And(Location),
    Or(Location),
    Not(Location),

    // Identation
    Indent(Location),
    Dedent(Location),
    Newline(Location),

    // Literals
    Identifier(String, Location),
    Number(f64, Location),
    String(String, Location),

    // Operators
    Plus(Location),
    Minus(Location),
    Star(Location),
    Slash(Location),
    DoubleSlash(Location),
    PlusEqual(Location),
    MinusEqual(Location),
    StarEqual(Location),
    SlashEqual(Location),
    Equal(Location),
    EqualEqual(Location),
    TripleEqual(Location),
    BangEqual(Location),

    Less(Location),
    LessEqual(Location),
    Greater(Location),
    GreaterEqual(Location),

    // Delimiters
    LeftParen(Location),
    RightParen(Location),
    LeftBrace(Location),
    RightBrace(Location),
    LeftBracket(Location),
    RightBracket(Location),
    Comma(Location),
    Dot(Location),
    Colon(Location),
    DoubleColon(Location),
    Semicolon(Location),

    // End of File
    Eof(Location),
}

impl Token {
    pub fn get_location(&self) -> &Location {
        match self {
            Token::Let(loc) |
            Token::Mut(loc) |
            Token::Const(loc) |
            Token::Print(loc) |
            Token::Input(loc) |
            Token::Fun(loc) |
            Token::Class(loc) |
            Token::SelfKw(loc) |
            Token::Return(loc) |
            Token::If(loc) |
            Token::Elif(loc) |
            Token::Else(loc) |
            Token::While(loc) |
            Token::For(loc) |
            Token::In(loc) |
            Token::List(loc) |
            Token::Dict(loc) |
            Token::Import(loc) |
            Token::From(loc) |
            Token::True(loc) |
            Token::False(loc) |
            Token::Nil(loc) |
            Token::And(loc) |
            Token::Or(loc) |
            Token::Not(loc) |
            Token::Indent(loc) |
            Token::Dedent(loc) |
            Token::Newline(loc) |
            Token::Identifier(_, loc) |
            Token::Number(_, loc) |
            Token::String(_, loc) |
            Token::Plus(loc) |
            Token::Minus(loc) |
            Token::Star(loc) |
            Token::Slash(loc) |
            Token::DoubleSlash(loc) |
            Token::PlusEqual(loc) |
            Token::MinusEqual(loc) |
            Token::StarEqual(loc) |
            Token::SlashEqual(loc) |
            Token::Equal(loc) |
            Token::EqualEqual(loc) |
            Token::TripleEqual(loc) |
            Token::BangEqual(loc) |
            Token::Less(loc) |
            Token::LessEqual(loc) |
            Token::Greater(loc) |
            Token::GreaterEqual(loc) |
            Token::LeftParen(loc) |
            Token::RightParen(loc) |
            Token::LeftBrace(loc) |
            Token::RightBrace(loc) |
            Token::LeftBracket(loc) |
            Token::RightBracket(loc) |
            Token::Comma(loc) |
            Token::Dot(loc) |
            Token::Colon(loc) |
            Token::DoubleColon(loc) |
            Token::Semicolon(loc) |
            Token::Eof(loc) => loc,
        }
    }

    pub fn friendly_name(&self) -> String {
        match self {
            Token::Let(_) => "'let'".to_string(),
            Token::Mut(_) => "'mut'".to_string(),
            Token::Const(_) => "'const'".to_string(),
            Token::Print(_) => "'print'".to_string(),
            Token::Input(_) => "'input'".to_string(),
            Token::Fun(_) => "'fun'".to_string(),
            Token::Class(_) => "'class'".to_string(),
            Token::SelfKw(_) => "'self'".to_string(),
            Token::Return(_) => "'return'".to_string(),
            Token::If(_) => "'if'".to_string(),
            Token::Elif(_) => "'elif'".to_string(),
            Token::Else(_) => "'else'".to_string(),
            Token::While(_) => "'while'".to_string(),
            Token::For(_) => "'for'".to_string(),
            Token::In(_) => "'in'".to_string(),
            Token::List(_) => "'list'".to_string(),
            Token::Dict(_) => "'dict'".to_string(),
            Token::Import(_) => "'import'".to_string(),
            Token::From(_) => "'from'".to_string(),
            Token::True(_) => "'true'".to_string(),
            Token::False(_) => "'false'".to_string(),
            Token::Nil(_) => "'nil'".to_string(),
            Token::And(_) => "'and'".to_string(),
            Token::Or(_) => "'or'".to_string(),
            Token::Not(_) => "'not'".to_string(),
            Token::Indent(_) => "indent".to_string(),
            Token::Dedent(_) => "dedent".to_string(),
            Token::Newline(_) => "newline".to_string(),
            Token::Identifier(name, _) => format!("identifier '{}'", name),
            Token::Number(n, _) => format!("number '{}'", n),
            Token::String(s, _) => format!("string \"{}\"", s),
            Token::Plus(_) => "'+'".to_string(),
            Token::Minus(_) => "'-'".to_string(),
            Token::Star(_) => "'*'".to_string(),
            Token::Slash(_) => "'/'".to_string(),
            Token::DoubleSlash(_) => "'//'".to_string(),
            Token::PlusEqual(_) => "'+='".to_string(),
            Token::MinusEqual(_) => "'-='".to_string(),
            Token::StarEqual(_) => "'*='".to_string(),
            Token::SlashEqual(_) => "'/='".to_string(),
            Token::Equal(_) => "'='".to_string(),
            Token::EqualEqual(_) => "'=='".to_string(),
            Token::TripleEqual(_) => "'==='".to_string(),
            Token::BangEqual(_) => "'!='".to_string(),
            Token::Less(_) => "'<'".to_string(),
            Token::LessEqual(_) => "'<='".to_string(),
            Token::Greater(_) => "'>'".to_string(),
            Token::GreaterEqual(_) => "'>='".to_string(),
            Token::LeftParen(_) => "'('".to_string(),
            Token::RightParen(_) => "')'".to_string(),
            Token::LeftBrace(_) => "'{'".to_string(),
            Token::RightBrace(_) => "'}'".to_string(),
            Token::LeftBracket(_) => "'['".to_string(),
            Token::RightBracket(_) => "']'".to_string(),
            Token::Comma(_) => "','".to_string(),
            Token::Dot(_) => "'.'".to_string(),
            Token::Colon(_) => "':'".to_string(),
            Token::DoubleColon(_) => "'::'".to_string(),
            Token::Semicolon(_) => "';'".to_string(),
            Token::Eof(_) => "end of file".to_string(),
        }
    }
}

pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    indent_unit: Option<usize>,
    pending_tokens: Vec<Token>,
    at_start_of_line: bool,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
            line: 1,
            column: 1,
            indent_stack: vec![0],
            indent_unit: None,
            pending_tokens: Vec::new(),
            at_start_of_line: true,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next();
        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_start_of_line = true;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    
    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current_location(&self) -> Location {
        Location {
            line: self.line,
            column: self.column,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        if !self.pending_tokens.is_empty() {
            return Ok(self.pending_tokens.remove(0));
        }

        if self.at_start_of_line {
            self.at_start_of_line = false;
            let mut indent = 0;
            let loc = self.current_location();
            
            while let Some(&c) = self.peek() {
                if c == ' ' {
                    indent += 1;
                    self.advance();
                } else if c == '\t' {
                    indent += 4;
                    self.advance();
                } else if c == '\n' || c == '\r' {
                    // Linha vazia, ignora identação e reinicia
                    self.advance();
                    self.at_start_of_line = true;
                    return self.next_token();
                } else if c == '/' {
                    // Possível comentário no começo da linha: "// ..."
                    let mut it = self.chars.clone();
                    it.next();
                    if it.peek() == Some(&'/') {
                        // consome "//" e o resto da linha
                        self.advance();
                        self.advance();
                        while self.peek() != Some(&'\n') && self.peek().is_some() {
                            self.advance();
                        }
                        self.at_start_of_line = true;
                        return self.next_token();
                    }
                    break;
                } else {
                    break;
                }
            }

            if self.peek().is_none() {
                return self.handle_eof();
            }

            let last_indent = *self.indent_stack.last().unwrap();
            if indent > last_indent {
                let delta = indent - last_indent;
                if delta == 0 {
                    // impossível, mas evita divisão por zero e mensagens estranhas
                    self.indent_stack.push(indent);
                    return Ok(Token::Indent(loc));
                }
                if let Some(unit) = self.indent_unit {
                    // Permite indentação com 2, 4, etc, desde que consistente no arquivo.
                    if unit != 0 && (delta % unit != 0) {
                        return Err(format!(
                            "Inconsistent indentation at line {}, column {} (hint: keep the same indentation size per level; e.g. always +{} spaces).",
                            loc.line,
                            loc.column,
                            unit
                        ));
                    }
                } else {
                    // Primeiro bloco define a unidade de indentação do arquivo
                    self.indent_unit = Some(delta);
                }
                self.indent_stack.push(indent);
                return Ok(Token::Indent(loc));
            } else if indent < last_indent {
                while indent < *self.indent_stack.last().unwrap() {
                    self.indent_stack.pop();
                    self.pending_tokens.push(Token::Dedent(loc.clone()));
                }
                if indent != *self.indent_stack.last().unwrap() {
                    return Err(format!(
                        "Inconsistent indentation at line {}, column {} (hint: keep indentation consistent; if you use tabs, avoid mixing them with spaces).",
                        loc.line,
                        loc.column
                    ));
                }
                if !self.pending_tokens.is_empty() {
                    return Ok(self.pending_tokens.remove(0));
                }
            }
        }

        self.skip_inline_whitespace();

        let loc = self.current_location();
        let ch = match self.advance() {
            Some(c) => c,
            None => return self.handle_eof(),
        };

        if ch == '\n' || ch == '\r' {
            self.at_start_of_line = true;
            return Ok(Token::Newline(loc));
        }

        let token = if ch.is_alphabetic() || ch == '_' {
            self.read_identifier_or_keyword(ch, loc)
        } else if ch.is_digit(10) {
            self.read_number(ch, loc)
        } else {
            match ch {
                '(' => Token::LeftParen(loc),
                ')' => Token::RightParen(loc),
                '{' => Token::LeftBrace(loc),
                '}' => Token::RightBrace(loc),
                '[' => Token::LeftBracket(loc),
                ']' => Token::RightBracket(loc),
                ',' => Token::Comma(loc),
                '.' => Token::Dot(loc),
                ':' => {
                    if let Some(':') = self.peek() {
                        self.advance();
                        Token::DoubleColon(loc)
                    } else {
                        Token::Colon(loc)
                    }
                },
                ';' => Token::Semicolon(loc),
                '+' => {
                    if self.match_char('=') { Token::PlusEqual(loc) } else { Token::Plus(loc) }
                },
                '-' => {
                    if self.match_char('=') { Token::MinusEqual(loc) } else { Token::Minus(loc) }
                },
                '*' => {
                    if self.match_char('=') { Token::StarEqual(loc) } else { Token::Star(loc) }
                },
                '/' => {
                    if let Some('/') = self.peek() {
                        self.advance();
                        Token::DoubleSlash(loc)
                    } else if self.match_char('=') {
                        Token::SlashEqual(loc)
                    } else {
                        Token::Slash(loc)
                    }
                },
                '=' => {
                    if self.match_char('=') {
                        if self.match_char('=') {
                            Token::TripleEqual(loc)
                        } else {
                            Token::EqualEqual(loc)
                        }
                    } else {
                        Token::Equal(loc)
                    }
                }
                '!' => {
                    if self.match_char('=') {
                        Token::BangEqual(loc)
                    } else {
                        return Err(format!("Unexpected character: {} at line {}, column {}", ch, loc.line, loc.column));
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        Token::LessEqual(loc)
                    } else {
                        Token::Less(loc)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        Token::GreaterEqual(loc)
                    } else {
                        Token::Greater(loc)
                    }
                }
                '"' => self.read_string(loc)?,
                _ => return Err(format!(
                    "Unexpected character: {} at line {}, column {}",
                    ch, loc.line, loc.column
                )),
            }
        };
        Ok(token)
    }

    fn handle_eof(&mut self) -> Result<Token, String> {
        let loc = self.current_location();
        if self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            return Ok(Token::Dedent(loc));
        }
        Ok(Token::Eof(loc))
    }

    fn skip_inline_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier_or_keyword(&mut self, first_char: char, loc: Location) -> Token {
        let mut ident = String::new();
        ident.push(first_char);
        while let Some(&c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        match ident.as_str() {
            "let" => Token::Let(loc),
            "mut" => Token::Mut(loc),
            "const" => Token::Const(loc),
            "print" => Token::Print(loc),
            "input" => Token::Input(loc),
            "fun" => Token::Fun(loc),
            "class" => Token::Class(loc),
            "self" => Token::SelfKw(loc),
            "return" => Token::Return(loc),
            "if" => Token::If(loc),
            "elif" => Token::Elif(loc),
            "else" => Token::Else(loc),
            "while" => Token::While(loc),
            "for" => Token::For(loc),
            "in" => Token::In(loc),
            "list" => Token::List(loc),
            "dict" => Token::Dict(loc),
            "import" => Token::Import(loc),
            "from" => Token::From(loc),
            "true" => Token::True(loc),
            "false" => Token::False(loc),
            "nil" => Token::Nil(loc),
            "and" => Token::And(loc),
            "or" => Token::Or(loc),
            "not" => Token::Not(loc),
            _ => Token::Identifier(ident, loc),
        }
    }

    fn read_number(&mut self, first_char: char, loc: Location) -> Token {
        let mut number = String::new();
        number.push(first_char);
        while let Some(&c) = self.peek() {
            if c.is_digit(10) || c == '.' {
                number.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Token::Number(number.parse().unwrap(), loc)
    }

    fn read_string(&mut self, loc: Location) -> Result<Token, String> {
        let mut s = String::new();
        while let Some(c) = self.peek().copied() {
            if c == '"' {
                break;
            }
            if c == '\n' || c == '\r' {
                return Err(format!(
                    "Unterminated string (newline) at line {}, column {} (hint: close it with '\"' or use \\n)",
                    loc.line,
                    loc.column
                ));
            }
            if c == '\\' {
                self.advance();
                let esc = self.advance().unwrap_or('\0');
                match esc {
                    '"' => s.push('"'),
                    '\\' => s.push('\\'),
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    'b' => s.push('\x08'),
                    'f' => s.push('\x0c'),
                    'u' => {
                        let mut hex = String::new();
                        for _ in 0..4 {
                            if let Some(h) = self.advance() {
                                hex.push(h);
                            } else {
                                return Err(format!(
                                    "Incomplete unicode escape (\\uXXXX) at line {}, column {}",
                                    loc.line,
                                    loc.column
                                ));
                            }
                        }
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                s.push(ch);
                            } else {
                                return Err(format!(
                                    "Invalid unicode escape (\\u{}) at line {}, column {}",
                                    hex,
                                    loc.line,
                                    loc.column
                                ));
                            }
                        } else {
                            return Err(format!(
                                "Invalid unicode escape (\\u{}) at line {}, column {}",
                                hex,
                                loc.line,
                                loc.column
                            ));
                        }
                    }
                    '\0' => {
                        return Err(format!(
                            "Unterminated string (end of file) at line {}, column {}",
                            loc.line,
                            loc.column
                        ));
                    }
                    other => s.push(other),
                }
            } else {
                s.push(self.advance().unwrap());
            }
        }
        if self.peek().is_none() {
            return Err(format!(
                "Unterminated string (end of file) at line {}, column {}",
                loc.line,
                loc.column
            ));
        }
        self.advance();
        Ok(Token::String(s, loc))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Precedence {
    None,
    Assignment,  // =
    Or,          // or
    And,         // and
    Equality,    // == !=
    Comparison,  // < > <= >= 
    Term,        // + -
    Factor,      // * /
    Unary,       // - not
    Call,        // . ()
    Index,       // []
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
    peek_token: Token, // Adicionado para permitir o lookahead
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> ParseResult<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next_token().map_err(|e| {
            ParseError::new("SNASK-PARSE-TOKENIZE", e, Span::single(Position::start()))
        })?;
        let peek_token = tokenizer.next_token().map_err(|e| {
            ParseError::new("SNASK-PARSE-TOKENIZE", e, Span::single(Position::start()))
        })?; // Initialize peek_token
        Ok(Parser {
            tokenizer,
            current_token,
            peek_token,
        })
    }

    pub fn debug_tokens(input: &'a str) {
        let mut tokenizer = Tokenizer::new(input);
        loop {
            match tokenizer.next_token() {
                Ok(Token::Eof(_)) => break,
                Ok(token) => println!("{:?}", token),
                Err(e) => println!("Token Error: {}", e),
            }
        }
    }

    fn span_len(loc: &Location, len: usize) -> Span {
        let start = Position::from_line_col(loc.line, loc.column);
        let end = start.advance_cols(len.max(1));
        Span::new(start, end)
    }

    fn span1(loc: &Location) -> Span {
        Self::span_len(loc, 1)
    }

    fn token_len(tok: &Token) -> usize {
        match tok {
            Token::Identifier(s, _) => s.len().max(1),
            Token::Number(n, _) => format!("{}", n).len().max(1),
            Token::String(s, _) => (s.len() + 2).max(1),
            Token::Let(_) => 3,
            Token::Mut(_) => 3,
            Token::Const(_) => 5,
            Token::Print(_) => 5,
            Token::Input(_) => 5,
            Token::Fun(_) => 3,
            Token::Class(_) => 5,
            Token::SelfKw(_) => 4,
            Token::Return(_) => 6,
            Token::If(_) => 2,
            Token::Elif(_) => 4,
            Token::Else(_) => 4,
            Token::While(_) => 5,
            Token::For(_) => 3,
            Token::In(_) => 2,
            Token::List(_) => 4,
            Token::Dict(_) => 4,
            Token::Import(_) => 6,
            Token::From(_) => 4,
            Token::True(_) => 4,
            Token::False(_) => 5,
            Token::Nil(_) => 3,
            Token::And(_) => 3,
            Token::Or(_) => 2,
            Token::Not(_) => 3,
            Token::DoubleSlash(_) => 2,
            Token::PlusEqual(_) | Token::MinusEqual(_) | Token::StarEqual(_) | Token::SlashEqual(_) => 2,
            Token::EqualEqual(_) | Token::BangEqual(_) | Token::LessEqual(_) | Token::GreaterEqual(_) | Token::DoubleColon(_) => 2,
            Token::TripleEqual(_) => 3,
            _ => 1,
        }
    }

    fn token_span(tok: &Token) -> Span {
        let loc = tok.get_location().clone();
        Self::span_len(&loc, Self::token_len(tok))
    }

    fn consume_token(&mut self, expected_variant: &Token) -> ParseResult<Token> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(expected_variant) {
            let consumed_token = self.current_token.clone();
            self.current_token = self.peek_token.clone();
            self.peek_token = self.tokenizer.next_token().map_err(|e| {
                ParseError::new(
                    "SNASK-PARSE-TOKENIZE",
                    e,
                    Span::single(Position::start()),
                )
            })?;
            Ok(consumed_token)
        } else {
            let found = self.current_token.clone();
            let span = Self::token_span(&found);
            let msg = format!(
                "Expected {}, but found {}.",
                expected_variant.friendly_name(),
                found.friendly_name()
            );

            let mut err = if matches!(expected_variant, Token::Semicolon(_)) {
                ParseError::new("SNASK-PARSE-SEMICOLON", msg, span).with_help(
                    "You probably missed a ';' at the end of the line.".to_string(),
                )
            } else if matches!(expected_variant, Token::RightParen(_))
                && matches!(found, Token::Newline(_) | Token::Dedent(_) | Token::Eof(_))
            {
                ParseError::new("SNASK-PARSE-MISSING-RPAREN", msg, span).with_help(
                    "You probably missed a closing ')'.".to_string(),
                )
            } else if matches!(expected_variant, Token::RightBracket(_))
                && matches!(found, Token::Newline(_) | Token::Dedent(_) | Token::Eof(_))
            {
                ParseError::new("SNASK-PARSE-MISSING-RBRACKET", msg, span).with_help(
                    "You probably missed a closing ']'.".to_string(),
                )
            } else if matches!(expected_variant, Token::RightBrace(_))
                && matches!(found, Token::Newline(_) | Token::Dedent(_) | Token::Eof(_))
            {
                ParseError::new("SNASK-PARSE-MISSING-RBRACE", msg, span).with_help(
                    "You probably missed a closing '}'.".to_string(),
                )
            } else if matches!(expected_variant, Token::Indent(_)) {
                ParseError::new("SNASK-PARSE-INDENT", msg, span).with_help(
                    "Check block indentation; keep it consistent throughout the file.".to_string(),
                )
            } else {
                ParseError::new("SNASK-PARSE-EXPECTED", msg, span)
            };

            err = err.with_note(format!("expected: {}", expected_variant.friendly_name()));
            Err(err)
        }
    }
    
    fn consume_identifier(&mut self) -> ParseResult<(String, Location)> {
        let (name, loc) = match self.current_token.clone() {
            Token::Identifier(s, loc) => (s, loc),
            _ => {
                let found = self.current_token.clone();
                let span = Self::token_span(&found);
                return Err(ParseError::new(
                    "SNASK-PARSE-IDENT",
                    format!("Expected identifier, but found {}.", found.friendly_name()),
                    span,
                ));
            }
        };
        // This was the bug: it was consuming directly from tokenizer,
        // bypassing the peek_token mechanism.
        // It should update current_token from peek_token, and peek_token from tokenizer.
        self.current_token = self.peek_token.clone();
        self.peek_token = self.tokenizer.next_token().map_err(|e| {
            ParseError::new(
                "SNASK-PARSE-TOKENIZE",
                e,
                Span::single(Position::start()),
            )
        })?;
        Ok((name, loc))
    }
    
    fn at_end(&self) -> bool {
        matches!(self.current_token, Token::Eof(_))
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut program = Vec::new();
        while !self.at_end() {
            // Ignora novas linhas no topo do arquivo
            if let Token::Newline(_) = self.current_token {
                self.consume_token(&Token::Newline(Location{line:0, column:0}))?;
                continue;
            }
            program.push(self.parse_statement()?);
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        if let Token::Identifier(_, loc) = self.current_token.clone() {
            let op_tok = match self.peek_token {
                Token::Equal(_) |
                Token::PlusEqual(_) |
                Token::MinusEqual(_) |
                Token::StarEqual(_) |
                Token::SlashEqual(_) => Some(self.peek_token.clone()),
                _ => None,
            };
            if let Some(op_tok) = op_tok {
                // This is an assignment statement.
                let (name, _) = self.consume_identifier()?;
                // consume operator token
                self.consume_token(&op_tok)?;
                let rhs = self.parse_expression(Precedence::Assignment)?;
                let value = match op_tok {
                    Token::Equal(_) => rhs,
                    Token::PlusEqual(_) => Expr { kind: ExprKind::Binary { op: BinaryOp::Add, left: Box::new(Expr { kind: ExprKind::Variable(name.clone()), loc: loc.clone(), span: loc.to_span() }), right: Box::new(rhs) }, loc: loc.clone(), span: loc.to_span() },
                    Token::MinusEqual(_) => Expr { kind: ExprKind::Binary { op: BinaryOp::Subtract, left: Box::new(Expr { kind: ExprKind::Variable(name.clone()), loc: loc.clone(), span: loc.to_span() }), right: Box::new(rhs) }, loc: loc.clone(), span: loc.to_span() },
                    Token::StarEqual(_) => Expr { kind: ExprKind::Binary { op: BinaryOp::Multiply, left: Box::new(Expr { kind: ExprKind::Variable(name.clone()), loc: loc.clone(), span: loc.to_span() }), right: Box::new(rhs) }, loc: loc.clone(), span: loc.to_span() },
                    Token::SlashEqual(_) => Expr { kind: ExprKind::Binary { op: BinaryOp::Divide, left: Box::new(Expr { kind: ExprKind::Variable(name.clone()), loc: loc.clone(), span: loc.to_span() }), right: Box::new(rhs) }, loc: loc.clone(), span: loc.to_span() },
                    _ => rhs,
                };
                self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
                let kind = StmtKind::VarAssignment(crate::ast::VarSet { name, value });
                return Ok(Stmt { kind, loc: loc.clone(), span: loc.to_span() });
            }
        }

        match self.current_token {
            Token::Newline(_) => {
                self.consume_token(&Token::Newline(Location{line:0, column:0}))?;
                self.parse_statement()
            }
            Token::Let(_) => self.parse_var_declaration(),
            Token::Mut(_) => self.parse_mut_declaration(),
            Token::Const(_) => self.parse_const_declaration(),
            Token::Print(_) => self.parse_print_statement(),
            Token::Input(_) => self.parse_input_statement(),
            Token::If(_) => self.parse_if_statement(),
            Token::While(_) => self.parse_while_statement(),
            Token::For(_) => self.parse_for_statement(),
            Token::Fun(_) => self.parse_fun_declaration(),
            Token::Class(_) => self.parse_class_declaration(),
            Token::Return(_) => self.parse_return_statement(),
            Token::Import(_) => self.parse_import_statement(),
            Token::From(_) => self.parse_from_import_statement(),
            _ => {
                let loc = self.current_token.get_location().clone();
                let expr = self.parse_expression(Precedence::Assignment)?;
                
                let kind = match expr.kind {
                    ExprKind::FunctionCall { .. } => StmtKind::FuncCall(expr),
                    _ => StmtKind::Expression(expr),
                };
                self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
                Ok(Stmt { kind, loc: loc.clone(), span: loc.to_span() })
            }
        }
    }

    fn parse_input_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Input(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?
            .ok_or_else(|| ParseError::new(
                "SNASK-PARSE-TYPE-ANNOT",
                "Expected a type annotation (e.g. ': str') after the variable name in the 'input' statement.".to_string(),
                Self::span1(&loc),
            ))?;
        
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let span = Self::span1(&loc).merge(&Self::span1(&end_loc));
        Ok(Stmt {
            kind: StmtKind::Input { name, var_type },
            loc: loc.clone(),
            span,
        })
    }

    fn parse_import_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Import(Location{line:0, column:0}))?.get_location().clone();
        let path = match self.current_token.clone() {
            Token::String(s, _) => {
                self.consume_token(&Token::String("".to_string(), Location{line:0, column:0}))?;
                s
            },
            _ => {
                let found = self.current_token.clone();
                return Err(ParseError::new(
                    "SNASK-PARSE-IMPORT",
                    format!("Expected string literal after 'import', found {}.", found.friendly_name()),
                    Self::token_span(&found),
                ));
            }
        };
        
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let span = Self::span1(&loc).merge(&Self::span1(&end_loc));
        Ok(Stmt {
            kind: StmtKind::Import(path),
            loc: loc.clone(),
            span,
        })
    }

    fn parse_from_import_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::From(Location{line:0, column:0}))?.get_location().clone();

        // from / import module;
        // from dir/subdir import module;
        let mut is_current_dir = false;
        let mut from_parts: Vec<String> = Vec::new();

        match self.current_token.clone() {
            Token::Slash(_) => {
                self.consume_token(&Token::Slash(Location { line: 0, column: 0 }))?;
                is_current_dir = true;
            }
            Token::Identifier(_, _) => {
                // Parse identifiers separated by '/'
                loop {
                    let (seg, _) = self.consume_identifier()?;
                    from_parts.push(seg);
                    if matches!(self.current_token, Token::Slash(_)) {
                        self.consume_token(&Token::Slash(Location { line: 0, column: 0 }))?;
                        continue;
                    }
                    break;
                }
            }
            _ => {
                let found = self.current_token.clone();
                return Err(ParseError::new(
                    "SNASK-PARSE-FROM",
                    format!(
                        "Expected '/' or a directory name after 'from', found {}.",
                        found.friendly_name()
                    ),
                    Self::token_span(&found),
                ));
            }
        }

        self.consume_token(&Token::Import(Location { line: 0, column: 0 }))?;

        let (module, _) = self.consume_identifier()?;

        let end_loc = self
            .consume_token(&Token::Semicolon(Location { line: 0, column: 0 }))?
            .get_location()
            .clone();
        let span = Self::span1(&loc).merge(&Self::span1(&end_loc));
        Ok(Stmt::with_span(
            StmtKind::FromImport {
                from: from_parts,
                is_current_dir,
                module,
            },
            loc,
            span,
        ))
    }
    
    fn parse_block(&mut self) -> ParseResult<Vec<Stmt>> {
        // Se houver uma nova linha antes do bloco, consome.
        if let Token::Newline(_) = self.current_token {
            self.consume_token(&Token::Newline(Location{line:0, column:0}))?;
        }
        
        self.consume_token(&Token::Indent(Location{line:0, column:0}))?;
        let mut stmts = Vec::new();
        while !matches!(self.current_token, Token::Dedent(_)) && !self.at_end() {
            // Ignora novas linhas vazias dentro do bloco
            if let Token::Newline(_) = self.current_token {
                self.consume_token(&Token::Newline(Location{line:0, column:0}))?;
                continue;
            }
            stmts.push(self.parse_statement()?);
        }
        
        if !self.at_end() {
            self.consume_token(&Token::Dedent(Location{line:0, column:0}))?;
        }
        Ok(stmts)
    }

    fn parse_class_declaration(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Class(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        
        if let Token::Newline(_) = self.current_token {
            self.consume_token(&Token::Newline(Location{line:0, column:0}))?;
        }
        self.consume_token(&Token::Indent(Location{line:0, column:0}))?;
        
        let mut properties = Vec::new();
        let mut methods = Vec::new();
        
        while !matches!(self.current_token, Token::Dedent(_)) && !self.at_end() {
             match self.current_token {
                Token::Newline(_) => { self.consume_token(&Token::Newline(Location{line:0, column:0}))?; }
                Token::Let(_) => {
                    let stmt = self.parse_var_declaration()?;
                    if let StmtKind::VarDeclaration(d) = stmt.kind { properties.push(d); }
                }
                Token::Mut(_) => {
                    let stmt = self.parse_mut_declaration()?;
                    if let StmtKind::MutDeclaration(d) = stmt.kind { properties.push(d.to_var_decl()); }
                }
                Token::Fun(_) => {
                    let stmt = self.parse_fun_declaration()?;
                    if let StmtKind::FuncDeclaration(d) = stmt.kind { methods.push(d); }
                }
                _ => {
                    let found = self.current_token.clone();
                    return Err(ParseError::new(
                        "SNASK-PARSE-CLASS",
                        format!("Unexpected token in class: {}.", found.friendly_name()),
                        Self::token_span(&found),
                    ));
                }
            }
        }
        
        if !self.at_end() {
            self.consume_token(&Token::Dedent(Location{line:0, column:0}))?;
        }

        let mut span = Self::span_len(&loc, 5);
        if let Some(_last) = methods.last() {
            // Best-effort: methods are declarations; span points to class keyword..last method
            // (method spans are already on their statements in the body parsing above)
            span = span.merge(&Self::span1(&loc));
            // Can't access stmt spans here; fallback to class keyword.
        }
        Ok(Stmt::with_span(
            StmtKind::ClassDeclaration(crate::ast::ClassDecl { name, properties, methods }),
            loc,
            span,
        ))
    }

    fn parse_if_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::If(Location{line:0, column:0}))?.get_location().clone();
        let condition = self.parse_expression(Precedence::Assignment)?;
        let body = self.parse_block()?;

        let mut span = Self::span_len(&loc, 2).merge(&condition.span);
        if let Some(last) = body.last() {
            span = span.merge(&last.span);
        }
        
        let if_block = IfBlock { condition, body };
        let mut elif_blocks = Vec::new();
        
        while let Token::Elif(_) = self.current_token {
            self.consume_token(&Token::Elif(Location{line:0, column:0}))?;
            let condition = self.parse_expression(Precedence::Assignment)?;
            let body = self.parse_block()?;
            span = span.merge(&condition.span);
            if let Some(last) = body.last() {
                span = span.merge(&last.span);
            }
            elif_blocks.push(IfBlock { condition, body });
        }
        
        let else_block = if let Token::Else(_) = self.current_token {
            self.consume_token(&Token::Else(Location{line:0, column:0}))?;
            let b = self.parse_block()?;
            if let Some(last) = b.last() {
                span = span.merge(&last.span);
            }
            Some(b)
        } else {
            None
        };

        Ok(Stmt::with_span(
            StmtKind::Conditional(ConditionalStmt {
                if_block,
                elif_blocks,
                else_block,
            }),
            loc,
            span,
        ))
    }

    fn parse_while_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::While(Location{line:0, column:0}))?.get_location().clone();
        let condition = self.parse_expression(Precedence::Assignment)?;
        let body = self.parse_block()?;
        let mut span = Self::span_len(&loc, 5).merge(&condition.span);
        if let Some(last) = body.last() {
            span = span.merge(&last.span);
        }
        Ok(Stmt::with_span(
            StmtKind::Loop(LoopStmt::While { condition, body }),
            loc,
            span,
        ))
    }

    fn parse_for_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::For(Location{line:0, column:0}))?.get_location().clone();
        let (iterator, _) = self.consume_identifier()?;
        self.consume_token(&Token::In(Location{line:0, column:0}))?;
        let iterable = self.parse_expression(Precedence::Assignment)?;
        let body = self.parse_block()?;
        let mut span = Self::span_len(&loc, 3).merge(&iterable.span);
        if let Some(last) = body.last() {
            span = span.merge(&last.span);
        }
        Ok(Stmt::with_span(
            StmtKind::Loop(LoopStmt::For {
                iterator,
                iterable,
                body,
            }),
            loc,
            span,
        ))
    }

    fn parse_fun_declaration(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Fun(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        self.consume_token(&Token::LeftParen(Location{line:0, column:0}))?;
        
        let mut params = Vec::new();
        if !matches!(self.current_token, Token::RightParen(_)) {
            loop {
                let (param_name, _) = self.consume_identifier()?;
                let param_type = self.parse_type_annotation()?;
                let param_type_resolved = match param_type {
                    Some(t) => t,
                    None => Type::Any,
                };
                params.push((param_name, param_type_resolved)); // Default to Type::Any if not specified
                if !matches!(self.current_token, Token::Comma(_)) {
                    break;
                }
                self.consume_token(&Token::Comma(Location{line:0, column:0}))?;
            }
        }
        self.consume_token(&Token::RightParen(Location{line:0, column:0}))?;
        
        let return_type: Option<Type> = self.parse_type_annotation()?;
        let body = self.parse_block()?;

        let mut span = Self::span_len(&loc, 3);
        if let Some(last) = body.last() {
            span = span.merge(&last.span);
        }
        Ok(Stmt::with_span(
            StmtKind::FuncDeclaration(FuncDecl {
                name,
                params,
                return_type,
                body,
            }),
            loc,
            span,
        ))
    }

    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Return(Location { line: 0, column: 0 }))?.get_location().clone();
        let value = self.parse_expression(Precedence::Assignment)?;
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let span = Self::span1(&loc).merge(&value.span).merge(&Self::span1(&end_loc));
        Ok(Stmt::with_span(StmtKind::Return(value), loc, span))
    }

    fn parse_type_annotation(&mut self) -> ParseResult<Option<Type>> {
        if matches!(self.current_token, Token::Colon(_)) {
            self.consume_token(&Token::Colon(Location{line:0, column:0}))?;
            let (type_name, _) = self.consume_identifier()?;
            let var_type = Type::from_str(&type_name).map_err(|_| {
                ParseError::new(
                    "SNASK-PARSE-TYPE",
                    format!("Unknown type: {}", type_name),
                    Self::span_len(&self.current_token.get_location().clone(), type_name.len().max(1)),
                )
            })?;
            Ok(Some(var_type))
        } else {
            Ok(None)
        }
    }

    fn parse_var_declaration(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Let(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?;
        self.consume_token(&Token::Equal(Location{line:0, column:0}))?;
        let value = self.parse_expression(Precedence::Assignment)?;
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let span = Self::span1(&loc).merge(&value.span).merge(&Self::span1(&end_loc));
        Ok(Stmt::with_span(
            StmtKind::VarDeclaration(VarDecl {
                name,
                var_type,
                value,
            }),
            loc,
            span,
        ))
    }

    fn parse_mut_declaration(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Mut(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?;
        self.consume_token(&Token::Equal(Location{line:0, column:0}))?;
        let value = self.parse_expression(Precedence::Assignment)?;
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let span = Self::span1(&loc).merge(&value.span).merge(&Self::span1(&end_loc));
        Ok(Stmt::with_span(
            StmtKind::MutDeclaration(MutDecl {
                name,
                var_type,
                value,
            }),
            loc,
            span,
        ))
    }

    fn parse_const_declaration(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Const(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?;
        self.consume_token(&Token::Equal(Location{line:0, column:0}))?;
        let value = self.parse_expression(Precedence::Assignment)?;
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let span = Self::span1(&loc).merge(&value.span).merge(&Self::span1(&end_loc));

        Ok(Stmt::with_span(
            StmtKind::ConstDeclaration(ConstDecl {
                name,
                var_type,
                value,
            }),
            loc,
            span,
        ))
    }

    fn parse_print_statement(&mut self) -> ParseResult<Stmt> {
        let loc = self.consume_token(&Token::Print(Location{line:0, column:0}))?.get_location().clone();
        self.consume_token(&Token::LeftParen(Location{line:0, column:0}))?;
        let mut expressions = Vec::new();
        if !matches!(self.current_token, Token::RightParen(_)) {
            loop {
                expressions.push(self.parse_expression(Precedence::Assignment)?);
                if !matches!(self.current_token, Token::Comma(_)) {
                    break;
                }
                self.consume_token(&Token::Comma(Location{line:0, column:0}))?;
            }
        }
        self.consume_token(&Token::RightParen(Location{line:0, column:0}))?;
        let end_loc = self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?.get_location().clone();
        let mut span = Self::span1(&loc).merge(&Self::span1(&end_loc));
        for e in &expressions {
            span = span.merge(&e.span);
        }
        Ok(Stmt::with_span(StmtKind::Print(expressions), loc, span))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expr> {
        let mut expr = self.parse_prefix()?;

        while precedence <= self.get_precedence(&self.current_token) {
            expr = self.parse_infix(expr)?;
        }

        Ok(expr)
    }

    fn get_precedence(&self, token: &Token) -> Precedence {
        match token {
            Token::Equal(_) => Precedence::Assignment,
            Token::Or(_) => Precedence::Or,
            Token::And(_) => Precedence::And,
            Token::EqualEqual(_) | Token::TripleEqual(_) | Token::BangEqual(_) => Precedence::Equality,
            Token::Less(_) | Token::LessEqual(_) | Token::Greater(_) | Token::GreaterEqual(_) => Precedence::Comparison,
            Token::Plus(_) | Token::Minus(_) => Precedence::Term,
            Token::Star(_) | Token::Slash(_) | Token::DoubleSlash(_) => Precedence::Factor,
            Token::LeftParen(_) => Precedence::Call,
            Token::LeftBracket(_) => Precedence::Index,
            Token::Dot(_) | Token::DoubleColon(_) => Precedence::Call, // Set Dot and DoubleColon precedence
            _ => Precedence::None,
        }
    }

    fn binary_op_from_token(&self, token: &Token) -> ParseResult<BinaryOp> {
        match token {
            Token::Plus(_) => Ok(BinaryOp::Add),
            Token::Minus(_) => Ok(BinaryOp::Subtract),
            Token::Star(_) => Ok(BinaryOp::Multiply),
            Token::Slash(_) => Ok(BinaryOp::Divide),
            Token::DoubleSlash(_) => Ok(BinaryOp::IntDivide),
            Token::And(_) => Ok(BinaryOp::And),
            Token::Or(_) => Ok(BinaryOp::Or),
            Token::EqualEqual(_) => Ok(BinaryOp::Equals),
            Token::TripleEqual(_) => Ok(BinaryOp::StrictEquals),
            Token::BangEqual(_) => Ok(BinaryOp::NotEquals),
            Token::Less(_) => Ok(BinaryOp::LessThan),
            Token::LessEqual(_) => Ok(BinaryOp::LessThanOrEquals),
            Token::Greater(_) => Ok(BinaryOp::GreaterThan),
            Token::GreaterEqual(_) => Ok(BinaryOp::GreaterThanOrEquals),
            _ => Err(ParseError::new(
                "SNASK-PARSE-OP",
                "Invalid binary operator.".to_string(),
                Self::token_span(token),
            )),
        }
    }

    fn parse_prefix(&mut self) -> ParseResult<Expr> {
        let loc = self.current_token.get_location().clone();
        match self.current_token.clone() {
            Token::Number(n, _) => {
                self.consume_token(&Token::Number(0.0, loc.clone()))?;
                let span = Self::span_len(&loc, format!("{}", n).len());
                Ok(Expr::with_span(
                    ExprKind::Literal(LiteralValue::Number(n)),
                    loc,
                    span,
                ))
            }
            Token::String(s, _) => {
                self.consume_token(&Token::String("".to_string(), loc.clone()))?;
                let span = Self::span_len(&loc, s.len().saturating_add(2));
                Ok(Expr::with_span(
                    ExprKind::Literal(LiteralValue::String(s)),
                    loc,
                    span,
                ))
            }
            Token::True(_) => {
                self.consume_token(&Token::True(loc.clone()))?;
                Ok(Expr::with_span(
                    ExprKind::Literal(LiteralValue::Boolean(true)),
                    loc.clone(),
                    Self::span_len(&loc, 4),
                ))
            }
            Token::False(_) => {
                self.consume_token(&Token::False(loc.clone()))?;
                Ok(Expr::with_span(
                    ExprKind::Literal(LiteralValue::Boolean(false)),
                    loc.clone(),
                    Self::span_len(&loc, 5),
                ))
            }
            Token::Nil(_) => {
                self.consume_token(&Token::Nil(loc.clone()))?;
                Ok(Expr::with_span(
                    ExprKind::Literal(LiteralValue::Nil),
                    loc.clone(),
                    Self::span_len(&loc, 3),
                ))
            }
            Token::Identifier(s, _) => {
                let len = s.len();
                self.consume_identifier()?;
                Ok(Expr::with_span(
                    ExprKind::Variable(s),
                    loc.clone(),
                    Self::span_len(&loc, len),
                ))
            }
            Token::SelfKw(_) => {
                self.consume_token(&Token::SelfKw(loc.clone()))?;
                Ok(Expr::with_span(
                    ExprKind::Variable("self".to_string()),
                    loc.clone(),
                    Self::span_len(&loc, 4),
                ))
            }
            Token::Minus(_) => {
                self.consume_token(&Token::Minus(loc.clone()))?;
                let expr = self.parse_expression(Precedence::Unary)?;
                let span = Self::span1(&loc).merge(&expr.span);
                Ok(Expr::with_span(
                    ExprKind::Unary {
                        op: UnaryOp::Negative,
                        expr: Box::new(expr),
                    },
                    loc,
                    span,
                ))
            }
            Token::Not(_) => {
                self.consume_token(&Token::Not(loc.clone()))?;
                let expr = self.parse_expression(Precedence::Unary)?;
                let span = Self::span_len(&loc, 3).merge(&expr.span);
                Ok(Expr::with_span(
                    ExprKind::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(expr),
                    },
                    loc,
                    span,
                ))
            }
            Token::LeftParen(_) => {
                self.consume_token(&Token::LeftParen(loc))?;
                let expr = self.parse_expression(Precedence::Assignment)?;
                self.consume_token(&Token::RightParen(Location{line:0, column:0}))?;
                Ok(expr)
            }
            Token::LeftBracket(_) => self.parse_list_literal(),
            Token::LeftBrace(_) => self.parse_dict_literal(),
            _ => {
                let found = self.current_token.clone();
                Err(ParseError::new(
                    "SNASK-PARSE-EXPR",
                    format!("Expected expression, but found {}.", found.friendly_name()),
                    Self::token_span(&found),
                ))
            }
        }
    }

    fn parse_infix(&mut self, left: Expr) -> ParseResult<Expr> {
        let loc = self.current_token.get_location().clone();
        match self.current_token.clone() {
            Token::Plus(_) | Token::Minus(_) | Token::Star(_) | Token::Slash(_) | Token::DoubleSlash(_) |
            Token::And(_) | Token::Or(_) |
            Token::EqualEqual(_) | Token::TripleEqual(_) | Token::BangEqual(_) | Token::Less(_) |
            Token::LessEqual(_) | Token::Greater(_) | Token::GreaterEqual(_) => {
                let op = self.binary_op_from_token(&self.current_token)?;
                let precedence = self.get_precedence(&self.current_token);
                self.consume_token(&self.current_token.clone())?;
                let right = self.parse_expression(precedence)?;
                let span = left.span.merge(&right.span);
                Ok(Expr::with_span(
                    ExprKind::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    loc,
                    span,
                ))
            }
            Token::LeftParen(_) => self.parse_call_expression(left),
            Token::LeftBracket(_) => self.parse_index_access(left),
            Token::Dot(_) | Token::DoubleColon(_) => {
                let is_double_colon = matches!(self.current_token, Token::DoubleColon(_));
                let dot_loc = if let Token::Dot(_) = self.current_token {
                    self.consume_token(&Token::Dot(Location{line:0, column:0}))?.get_location().clone()
                } else {
                    self.consume_token(&Token::DoubleColon(Location{line:0, column:0}))?.get_location().clone()
                };
                let (property_name, _) = self.consume_identifier()?;

                if is_double_colon {
                    // Trata como Namespace (combina os nomes)
                    if let ExprKind::Variable(base_name) = left.kind {
                        let combined_name = format!("{}::{}", base_name, property_name);
                        let span = Self::span_len(&dot_loc, combined_name.len());
                        let combined_expr = Expr::with_span(ExprKind::Variable(combined_name), dot_loc, span);
                        if matches!(self.current_token, Token::LeftParen(_)) {
                            return self.parse_call_expression(combined_expr);
                        } else {
                            return Ok(combined_expr);
                        }
                    }
                }

                if matches!(self.current_token, Token::LeftParen(_)) {
                    // It's a method call
                    let span = left.span.merge(&Self::span_len(&dot_loc, property_name.len()));
                    let callee = Expr::with_span(
                        ExprKind::PropertyAccess {
                            target: Box::new(left),
                            property: property_name,
                        },
                        dot_loc,
                        span,
                    );
                    self.parse_call_expression(callee)
                } else {
                    // It's a property access
                    let span = left.span.merge(&Self::span_len(&dot_loc, property_name.len()));
                    Ok(Expr::with_span(
                        ExprKind::PropertyAccess {
                            target: Box::new(left),
                            property: property_name,
                        },
                        dot_loc,
                        span,
                    ))
                }
            }
            _ => {
                let found = self.current_token.clone();
                Err(ParseError::new(
                    "SNASK-PARSE-EXPR",
                    format!("Unexpected token in expression: {}.", found.friendly_name()),
                    Self::token_span(&found),
                ))
            }
        }
    }
    
    fn parse_call_expression(&mut self, callee: Expr) -> ParseResult<Expr> {
        let loc = self.consume_token(&Token::LeftParen(Location{line:0, column:0}))?.get_location().clone();
        let mut args = Vec::new();
        if !matches!(self.current_token, Token::RightParen(_)) {
            loop {
                args.push(self.parse_expression(Precedence::Assignment)?); // This is where arguments are parsed
                if !matches!(self.current_token, Token::Comma(_)) {
                    break;
                }
                self.consume_token(&Token::Comma(Location{line:0, column:0}))?;
            }
        }
        let end_loc = self.consume_token(&Token::RightParen(Location{line:0, column:0}))?.get_location().clone();
        let mut span = callee.span.merge(&Self::span1(&end_loc));
        for a in &args {
            span = span.merge(&a.span);
        }
        
        Ok(Expr::with_span(
            ExprKind::FunctionCall {
                callee: Box::new(callee),
                args,
            },
            loc,
            span,
        ))
    }

    fn parse_list_literal(&mut self) -> ParseResult<Expr> {
        let loc = self.consume_token(&Token::LeftBracket(Location{line:0, column:0}))?.get_location().clone();
        let mut elements = Vec::new();
        if !matches!(self.current_token, Token::RightBracket(_)) {
            loop {
                elements.push(self.parse_expression(Precedence::Assignment)?);
                if !matches!(self.current_token, Token::Comma(_)) {
                    break;
                }
                self.consume_token(&Token::Comma(Location{line:0, column:0}))?;
            }
        }
        let end_loc = self.consume_token(&Token::RightBracket(Location{line:0, column:0}))?.get_location().clone();
        let mut span = Self::span1(&loc).merge(&Self::span1(&end_loc));
        for e in &elements {
            span = span.merge(&e.span);
        }
        Ok(Expr::with_span(
            ExprKind::Literal(LiteralValue::List(elements)),
            loc,
            span,
        ))
    }

    fn parse_dict_literal(&mut self) -> ParseResult<Expr> {
        let loc = self.consume_token(&Token::LeftBrace(Location{line:0, column:0}))?.get_location().clone();
        let mut pairs = Vec::new();
        if !matches!(self.current_token, Token::RightBrace(_)) {
            loop {
                let key = self.parse_expression(Precedence::Assignment)?;
                self.consume_token(&Token::Colon(Location{line:0, column:0}))?;
                let value = self.parse_expression(Precedence::Assignment)?;
                pairs.push((key, value));
                if !matches!(self.current_token, Token::Comma(_)) {
                    break;
                }
                self.consume_token(&Token::Comma(Location{line:0, column:0}))?;
            }
        }
        let end_loc = self.consume_token(&Token::RightBrace(Location{line:0, column:0}))?.get_location().clone();
        let mut span = Self::span1(&loc).merge(&Self::span1(&end_loc));
        for (k, v) in &pairs {
            span = span.merge(&k.span).merge(&v.span);
        }
        Ok(Expr::with_span(
            ExprKind::Literal(LiteralValue::Dict(pairs)),
            loc,
            span,
        ))
    }

    fn parse_index_access(&mut self, target: Expr) -> ParseResult<Expr> {
        let loc = self.consume_token(&Token::LeftBracket(Location{line:0, column:0}))?.get_location().clone();
        let index = self.parse_expression(Precedence::Assignment)?;
        let end_loc = self.consume_token(&Token::RightBracket(Location{line:0, column:0}))?.get_location().clone();
        let span = target.span.merge(&index.span).merge(&Self::span1(&end_loc));
        Ok(Expr::with_span(
            ExprKind::IndexAccess {
                target: Box::new(target),
                index: Box::new(index),
            },
            loc,
            span,
        ))
    }
}

pub fn parse_program(source: &str) -> ParseResult<Program> {
    let mut parser = Parser::new(source)?;
    parser.parse_program()
}

// Used by the LSP for semantic tokens / lightweight tooling.
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokenizer = Tokenizer::new(source);
    let mut out: Vec<Token> = Vec::new();
    loop {
        let t = tokenizer.next_token()?;
        let is_eof = matches!(t, Token::Eof(_));
        out.push(t);
        if is_eof {
            break;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod parse_error_tests {
    use super::*;

    #[test]
    fn parse_error_has_span_for_missing_semicolon() {
        let src = "class main\n    fun start()\n        let x = 1\n";
        let mut p = Parser::new(src).unwrap();
        let e = p.parse_program().unwrap_err();
        assert_eq!(e.code, "SNASK-PARSE-SEMICOLON");
        assert!(e.span.start.line >= 1);
        assert!(e.span.start.column >= 1);
    }
}
