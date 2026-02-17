use crate::ast::{
    Program, Stmt, StmtKind, Expr, ExprKind, VarDecl, MutDecl, ConstDecl, LiteralValue, 
    BinaryOp, UnaryOp, ConditionalStmt, IfBlock, LoopStmt, FuncDecl, Location
};
use crate::types::Type;
use std::iter::Peekable;
use std::str::FromStr;
use std::str::Chars;



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
    True(Location),
    False(Location),
    Nil(Location),

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
    Equal(Location),
    EqualEqual(Location),
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
            Token::True(loc) |
            Token::False(loc) |
            Token::Nil(loc) |
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
            Token::Equal(loc) |
            Token::EqualEqual(loc) |
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
            Token::True(_) => "'true'".to_string(),
            Token::False(_) => "'false'".to_string(),
            Token::Nil(_) => "'nil'".to_string(),
            Token::Indent(_) => "indentação".to_string(),
            Token::Dedent(_) => "redução de indentação".to_string(),
            Token::Newline(_) => "nova linha".to_string(),
            Token::Identifier(name, _) => format!("identificador '{}'", name),
            Token::Number(n, _) => format!("número '{}'", n),
            Token::String(s, _) => format!("string \"{}\"", s),
            Token::Plus(_) => "'+'".to_string(),
            Token::Minus(_) => "'-'".to_string(),
            Token::Star(_) => "'*'".to_string(),
            Token::Slash(_) => "'/'".to_string(),
            Token::Equal(_) => "'='".to_string(),
            Token::EqualEqual(_) => "'=='".to_string(),
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
            Token::Eof(_) => "fim de arquivo".to_string(),
        }
    }
}

pub struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
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
                    // Possível comentário, verifica
                    self.advance();
                    if self.match_char('/') {
                        while self.peek() != Some(&'\n') && self.peek().is_some() {
                            self.advance();
                        }
                        self.at_start_of_line = true;
                        return self.next_token();
                    } else {
                        // Não era comentário, era uma divisão. 
                        // Mas division não pode começar linha sem identação antes.
                        // Na verdade, pode se indent == 0.
                        // Vamos tratar como token normal abaixo, mas precisamos restaurar o '/'
                        // Por simplicidade, assumimos que nenhuma linha começa com '/' sem ser comentário.
                        return Err(format!("Linha não pode começar com '/' na linha {}, coluna {}", loc.line, loc.column));
                    }
                } else {
                    break;
                }
            }

            if self.peek().is_none() {
                return self.handle_eof();
            }

            let last_indent = *self.indent_stack.last().unwrap();
            if indent > last_indent {
                self.indent_stack.push(indent);
                return Ok(Token::Indent(loc));
            } else if indent < last_indent {
                while indent < *self.indent_stack.last().unwrap() {
                    self.indent_stack.pop();
                    self.pending_tokens.push(Token::Dedent(loc.clone()));
                }
                if indent != *self.indent_stack.last().unwrap() {
                    return Err(format!("Nível de identação inconsistente na linha {}, coluna {}", loc.line, loc.column));
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
                '+' => Token::Plus(loc),
                '-' => Token::Minus(loc),
                '*' => Token::Star(loc),
                '/' => Token::Slash(loc),
                '=' => {
                    if self.match_char('=') {
                        Token::EqualEqual(loc)
                    } else {
                        Token::Equal(loc)
                    }
                }
                '!' => {
                    if self.match_char('=') {
                        Token::BangEqual(loc)
                    } else {
                        return Err(format!("Caractere inesperado: {} na linha {}, coluna {}", ch, loc.line, loc.column));
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
                '"' => self.read_string(loc),
                _ => return Err(format!(
                    "Caractere inesperado: {} na linha {}, coluna {}",
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
            "true" => Token::True(loc),
            "false" => Token::False(loc),
            "nil" => Token::Nil(loc),
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

    fn read_string(&mut self, loc: Location) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek().copied() {
            if c != '"' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        self.advance();
        Token::String(s, loc)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Precedence {
    None,
    Assignment,  // =
    Equality,    // == !=
    Comparison,  // < > <= >= 
    Term,        // + -
    Factor,      // * /
    Unary,       // -
    Call,        // . ()
    Index,       // []
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
    peek_token: Token, // Adicionado para permitir o lookahead
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, String> {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next_token()?;
        let peek_token = tokenizer.next_token()?; // Initialize peek_token
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

    fn consume_token(&mut self, expected_variant: &Token) -> Result<Token, String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(expected_variant) {
            let consumed_token = self.current_token.clone();
            self.current_token = self.peek_token.clone();
            self.peek_token = self.tokenizer.next_token()?;
            Ok(consumed_token)
        } else {
            let found_loc = self.current_token.get_location().clone();
            Err(format!(
                "Esperado {}, mas encontrado {} na linha {}, coluna {}",
                expected_variant.friendly_name(), 
                self.current_token.friendly_name(), 
                found_loc.line, 
                found_loc.column
            ))
        }
    }
    
    fn consume_identifier(&mut self) -> Result<(String, Location), String> {
        let (name, loc) = match self.current_token.clone() {
            Token::Identifier(s, loc) => (s, loc),
            _ => {
                let found_loc = self.current_token.get_location().clone();
                return Err(format!("Esperado identificador, mas encontrado {} na linha {}, coluna {}", self.current_token.friendly_name(), found_loc.line, found_loc.column));
            }
        };
        // This was the bug: it was consuming directly from tokenizer,
        // bypassing the peek_token mechanism.
        // It should update current_token from peek_token, and peek_token from tokenizer.
        self.current_token = self.peek_token.clone();
        self.peek_token = self.tokenizer.next_token()?;
        Ok((name, loc))
    }
    
    fn at_end(&self) -> bool {
        matches!(self.current_token, Token::Eof(_))
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
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

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        if let Token::Identifier(_, loc) = self.current_token.clone() {
            if let Token::Equal(_) = self.peek_token {
                // This is an assignment statement.
                let (name, _) = self.consume_identifier()?;
                self.consume_token(&Token::Equal(Location{line:0,column:0}))?;
                let value = self.parse_expression(Precedence::Assignment)?;
                self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
                let kind = StmtKind::VarAssignment(crate::ast::VarSet { name, value });
                return Ok(Stmt { kind, loc });
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
            _ => {
                let loc = self.current_token.get_location().clone();
                let expr = self.parse_expression(Precedence::Assignment)?;
                
                let kind = match expr.kind {
                    ExprKind::FunctionCall { .. } => StmtKind::FuncCall(expr),
                    _ => StmtKind::Expression(expr),
                };
                self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
                Ok(Stmt { kind, loc })
            }
        }
    }

    fn parse_input_statement(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::Input(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?
            .ok_or_else(|| "Esperado anotação de tipo (ex: ': str') após nome da variável para comando 'input'.".to_string())?;
        
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
        Ok(Stmt {
            kind: StmtKind::Input { name, var_type },
            loc,
        })
    }

    fn parse_import_statement(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::Import(Location{line:0, column:0}))?.get_location().clone();
        let path = match self.current_token.clone() {
            Token::String(s, _) => {
                self.consume_token(&Token::String("".to_string(), Location{line:0, column:0}))?;
                s
            },
            _ => return Err(format!("Esperado string literal após 'import', encontrado {}", self.current_token.friendly_name())),
        };
        
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
        Ok(Stmt {
            kind: StmtKind::Import(path),
            loc,
        })
    }
    
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
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

    fn parse_class_declaration(&mut self) -> Result<Stmt, String> {
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
                _ => return Err(format!("Token inesperado em classe: {} na linha {}, coluna {}", self.current_token.friendly_name(), self.current_token.get_location().line, self.current_token.get_location().column)),
             }
        }
        
        if !self.at_end() {
            self.consume_token(&Token::Dedent(Location{line:0, column:0}))?;
        }

        Ok(Stmt {
            kind: StmtKind::ClassDeclaration(crate::ast::ClassDecl { name, properties, methods }),
            loc,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::If(Location{line:0, column:0}))?.get_location().clone();
        let condition = self.parse_expression(Precedence::Assignment)?;
        let body = self.parse_block()?;
        
        let if_block = IfBlock { condition, body };
        let mut elif_blocks = Vec::new();
        
        while let Token::Elif(_) = self.current_token {
            self.consume_token(&Token::Elif(Location{line:0, column:0}))?;
            let condition = self.parse_expression(Precedence::Assignment)?;
            let body = self.parse_block()?;
            elif_blocks.push(IfBlock { condition, body });
        }
        
        let else_block = if let Token::Else(_) = self.current_token {
            self.consume_token(&Token::Else(Location{line:0, column:0}))?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt {
            kind: StmtKind::Conditional(ConditionalStmt { if_block, elif_blocks, else_block }),
            loc,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::While(Location{line:0, column:0}))?.get_location().clone();
        let condition = self.parse_expression(Precedence::Assignment)?;
        let body = self.parse_block()?;
        Ok(Stmt {
            kind: StmtKind::Loop(LoopStmt::While { condition, body }),
            loc,
        })
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::For(Location{line:0, column:0}))?.get_location().clone();
        let (iterator, _) = self.consume_identifier()?;
        self.consume_token(&Token::In(Location{line:0, column:0}))?;
        let iterable = self.parse_expression(Precedence::Assignment)?;
        let body = self.parse_block()?;
        Ok(Stmt {
            kind: StmtKind::Loop(LoopStmt::For { iterator, iterable, body }),
            loc,
        })
    }

    fn parse_fun_declaration(&mut self) -> Result<Stmt, String> {
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

        Ok(Stmt {
            kind: StmtKind::FuncDeclaration(FuncDecl { name, params, return_type, body }),
            loc,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::Return(Location { line: 0, column: 0 }))?.get_location().clone();
        let value = self.parse_expression(Precedence::Assignment)?;
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
        Ok(Stmt {
            kind: StmtKind::Return(value),
            loc,
        })
    }

    fn parse_type_annotation(&mut self) -> Result<Option<Type>, String> {
        if matches!(self.current_token, Token::Colon(_)) {
            self.consume_token(&Token::Colon(Location{line:0, column:0}))?;
            let (type_name, _) = self.consume_identifier()?;
            let var_type = Type::from_str(&type_name).map_err(|_| format!("Tipo desconhecido: {}", type_name))?;
            Ok(Some(var_type))
        } else {
            Ok(None)
        }
    }

    fn parse_var_declaration(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::Let(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?;
        self.consume_token(&Token::Equal(Location{line:0, column:0}))?;
        let value = self.parse_expression(Precedence::Assignment)?;
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
        Ok(Stmt {
            kind: StmtKind::VarDeclaration(VarDecl { name, var_type, value }),
            loc,
        })
    }

    fn parse_mut_declaration(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::Mut(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?;
        self.consume_token(&Token::Equal(Location{line:0, column:0}))?;
        let value = self.parse_expression(Precedence::Assignment)?;
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
        Ok(Stmt {
            kind: StmtKind::MutDeclaration(MutDecl { name, var_type, value }),
            loc,
        })
    }

    fn parse_const_declaration(&mut self) -> Result<Stmt, String> {
        let loc = self.consume_token(&Token::Const(Location{line:0, column:0}))?.get_location().clone();
        let (name, _) = self.consume_identifier()?;
        let var_type = self.parse_type_annotation()?;
        self.consume_token(&Token::Equal(Location{line:0, column:0}))?;
        let value = self.parse_expression(Precedence::Assignment)?;
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;

        Ok(Stmt {
            kind: StmtKind::ConstDeclaration(ConstDecl { name, var_type, value }),
            loc,
        })
    }

    fn parse_print_statement(&mut self) -> Result<Stmt, String> {
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
        self.consume_token(&Token::Semicolon(Location{line:0, column:0}))?;
        Ok(Stmt {
            kind: StmtKind::Print(expressions),
            loc,
        })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr, String> {
        let mut expr = self.parse_prefix()?;

        while precedence <= self.get_precedence(&self.current_token) {
            expr = self.parse_infix(expr)?;
        }

        Ok(expr)
    }

    fn get_precedence(&self, token: &Token) -> Precedence {
        match token {
            Token::Equal(_) => Precedence::Assignment,
            Token::EqualEqual(_) | Token::BangEqual(_) => Precedence::Equality,
            Token::Less(_) | Token::LessEqual(_) | Token::Greater(_) | Token::GreaterEqual(_) => Precedence::Comparison,
            Token::Plus(_) | Token::Minus(_) => Precedence::Term,
            Token::Star(_) | Token::Slash(_) => Precedence::Factor,
            Token::LeftParen(_) => Precedence::Call,
            Token::LeftBracket(_) => Precedence::Index,
            Token::Dot(_) | Token::DoubleColon(_) => Precedence::Call, // Set Dot and DoubleColon precedence
            _ => Precedence::None,
        }
    }

    fn binary_op_from_token(&self, token: &Token) -> Result<BinaryOp, String> {
        match token {
            Token::Plus(_) => Ok(BinaryOp::Add),
            Token::Minus(_) => Ok(BinaryOp::Subtract),
            Token::Star(_) => Ok(BinaryOp::Multiply),
            Token::Slash(_) => Ok(BinaryOp::Divide),
            Token::EqualEqual(_) => Ok(BinaryOp::Equals),
            Token::BangEqual(_) => Ok(BinaryOp::NotEquals),
            Token::Less(_) => Ok(BinaryOp::LessThan),
            Token::LessEqual(_) => Ok(BinaryOp::LessThanOrEquals),
            Token::Greater(_) => Ok(BinaryOp::GreaterThan),
            Token::GreaterEqual(_) => Ok(BinaryOp::GreaterThanOrEquals),
            _ => Err("Operador binário inválido".to_string()),
        }
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let loc = self.current_token.get_location().clone();
        match self.current_token.clone() {
            Token::Number(n, _) => {
                self.consume_token(&Token::Number(0.0, loc.clone()))?;
                Ok(Expr {
                    kind: ExprKind::Literal(LiteralValue::Number(n)),
                    loc,
                })
            }
            Token::String(s, _) => {
                self.consume_token(&Token::String("".to_string(), loc.clone()))?;
                
                Ok(Expr {
                    kind: ExprKind::Literal(LiteralValue::String(s)),
                    loc,
                })
            }
            Token::True(_) => {
                self.consume_token(&Token::True(loc.clone()))?;
                Ok(Expr {
                    kind: ExprKind::Literal(LiteralValue::Boolean(true)),
                    loc,
                })
            }
            Token::False(_) => {
                self.consume_token(&Token::False(loc.clone()))?;
                Ok(Expr {
                    kind: ExprKind::Literal(LiteralValue::Boolean(false)),
                    loc,
                })
            }
            Token::Nil(_) => {
                self.consume_token(&Token::Nil(loc.clone()))?;
                Ok(Expr {
                    kind: ExprKind::Literal(LiteralValue::Nil),
                    loc,
                })
            }
            Token::Identifier(s, _) => {
                self.consume_identifier()?;
                Ok(Expr {
                    kind: ExprKind::Variable(s),
                    loc,
                })
            }
            Token::SelfKw(_) => {
                self.consume_token(&Token::SelfKw(loc.clone()))?;
                Ok(Expr {
                    kind: ExprKind::Variable("self".to_string()),
                    loc,
                })
            }
            Token::Minus(_) => {
                self.consume_token(&Token::Minus(loc.clone()))?;
                let expr = self.parse_expression(Precedence::Unary)?;
                Ok(Expr {
                    kind: ExprKind::Unary { op: UnaryOp::Negative, expr: Box::new(expr) },
                    loc,
                })
            }
            Token::LeftParen(_) => {
                self.consume_token(&Token::LeftParen(loc))?;
                let expr = self.parse_expression(Precedence::Assignment)?;
                self.consume_token(&Token::RightParen(Location{line:0, column:0}))?;
                Ok(expr)
            }
            Token::LeftBracket(_) => self.parse_list_literal(),
            Token::LeftBrace(_) => self.parse_dict_literal(),
            _ => Err(format!(
                "Esperada expressão, mas encontrado {} na linha {}, coluna {}",
                self.current_token.friendly_name(), loc.line, loc.column
            )),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr, String> {
        let loc = self.current_token.get_location().clone();
        match self.current_token.clone() {
            Token::Plus(_) | Token::Minus(_) | Token::Star(_) | Token::Slash(_) |
            Token::EqualEqual(_) | Token::BangEqual(_) | Token::Less(_) |
            Token::LessEqual(_) | Token::Greater(_) | Token::GreaterEqual(_) => {
                let op = self.binary_op_from_token(&self.current_token)?;
                let precedence = self.get_precedence(&self.current_token);
                self.consume_token(&self.current_token.clone())?;
                let right = self.parse_expression(precedence)?;
                Ok(Expr {
                    kind: ExprKind::Binary {
                        op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    loc,
                })
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
                        let combined_expr = Expr {
                            kind: ExprKind::Variable(combined_name),
                            loc: dot_loc,
                        };
                        if matches!(self.current_token, Token::LeftParen(_)) {
                            return self.parse_call_expression(combined_expr);
                        } else {
                            return Ok(combined_expr);
                        }
                    }
                }

                if matches!(self.current_token, Token::LeftParen(_)) {
                    // It's a method call
                    let callee = Expr {
                        kind: ExprKind::PropertyAccess {
                            target: Box::new(left),
                            property: property_name,
                        },
                        loc: dot_loc,
                    };
                    self.parse_call_expression(callee)
                } else {
                    // It's a property access
                    Ok(Expr {
                        kind: ExprKind::PropertyAccess {
                            target: Box::new(left),
                            property: property_name,
                        },
                        loc: dot_loc,
                    })
                }
            }
            _ => Err(format!("Token inesperado em expressão: {} na linha {}, coluna {}", self.current_token.friendly_name(), loc.line, loc.column)),
        }
    }
    
    fn parse_call_expression(&mut self, callee: Expr) -> Result<Expr, String> {
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
        self.consume_token(&Token::RightParen(Location{line:0, column:0}))?;
        
        Ok(Expr {
            kind: ExprKind::FunctionCall { callee: Box::new(callee), args },
            loc,
        })
    }

    fn parse_list_literal(&mut self) -> Result<Expr, String> {
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
        self.consume_token(&Token::RightBracket(Location{line:0, column:0}))?;
        Ok(Expr {
            kind: ExprKind::Literal(LiteralValue::List(elements)),
            loc,
        })
    }

    fn parse_dict_literal(&mut self) -> Result<Expr, String> {
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
        self.consume_token(&Token::RightBrace(Location{line:0, column:0}))?;
        Ok(Expr {
            kind: ExprKind::Literal(LiteralValue::Dict(pairs)),
            loc,
        })
    }

    fn parse_index_access(&mut self, target: Expr) -> Result<Expr, String> {
        let loc = self.consume_token(&Token::LeftBracket(Location{line:0, column:0}))?.get_location().clone();
        let index = self.parse_expression(Precedence::Assignment)?;
        self.consume_token(&Token::RightBracket(Location{line:0, column:0}))?;
        Ok(Expr {
            kind: ExprKind::IndexAccess { target: Box::new(target), index: Box::new(index) },
            loc,
        })
    }
}

pub fn parse_program(source: &str) -> Result<Program, String> {
    let mut parser = Parser::new(source)?;
    parser.parse_program()
}