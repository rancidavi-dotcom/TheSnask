use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum SnifValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<SnifValue>),
    Object(BTreeMap<String, SnifValue>),
}

#[derive(Debug)]
pub struct SnifParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser { s, i: 0, line: 1, col: 1 }
    }

    fn eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn peek(&self) -> Option<char> {
        self.s[self.i..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.i += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn err<T>(&self, message: impl Into<String>) -> Result<T, SnifParseError> {
        Err(SnifParseError { message: message.into(), line: self.line, col: self.col })
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while let Some(c) = self.peek() {
                if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
                    self.bump();
                } else {
                    break;
                }
            }

            if self.peek() == Some('/') {
                let mut it = self.s[self.i..].chars();
                let a = it.next();
                let b = it.next();
                if a == Some('/') && b == Some('/') {
                    self.bump();
                    self.bump();
                    while let Some(c) = self.peek() {
                        self.bump();
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                }
            }

            break;
        }
    }

    fn consume(&mut self, ch: char) -> bool {
        self.skip_ws_and_comments();
        if self.peek() == Some(ch) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, ch: char, what: &str) -> Result<(), SnifParseError> {
        if self.consume(ch) {
            Ok(())
        } else {
            self.err(format!("Expected '{ch}' {what}."))
        }
    }

    fn parse_string(&mut self) -> Result<String, SnifParseError> {
        self.skip_ws_and_comments();
        let q = match self.peek() {
            Some('"') => '"',
            Some('\'') => '\'',
            _ => return self.err("Expected string literal."),
        };
        self.bump();
        let mut out = String::new();
        while let Some(c) = self.bump() {
            if c == q {
                return Ok(out);
            }
            if c == '\\' {
                let Some(e) = self.bump() else { return self.err("Unterminated escape sequence in string."); };
                match e {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    '\'' => out.push('\''),
                    _ => out.push(e),
                }
                continue;
            }
            if c == '\n' || c == '\r' {
                return self.err("Unterminated string literal (newline).");
            }
            out.push(c);
        }
        self.err("Unterminated string literal (end of file).")
    }

    fn parse_identifier(&mut self) -> Result<String, SnifParseError> {
        self.skip_ws_and_comments();
        let Some(c0) = self.peek() else { return self.err("Unexpected end of input."); };
        let is_start = c0.is_ascii_alphabetic() || c0 == '_' || c0 == '$';
        if !is_start {
            return self.err("Expected identifier.");
        }
        let start = self.i;
        self.bump();
        while let Some(c) = self.peek() {
            let ok = c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '-';
            if !ok {
                break;
            }
            self.bump();
        }
        Ok(self.s[start..self.i].to_string())
    }

    fn parse_key(&mut self) -> Result<String, SnifParseError> {
        self.skip_ws_and_comments();
        match self.peek() {
            Some('"') | Some('\'') => self.parse_string(),
            _ => self.parse_identifier(),
        }
    }

    fn parse_number(&mut self) -> Result<SnifValue, SnifParseError> {
        self.skip_ws_and_comments();
        let start = self.i;
        if matches!(self.peek(), Some('-') | Some('+')) {
            self.bump();
        }
        let mut any = false;
        while matches!(self.peek(), Some('0'..='9')) {
            any = true;
            self.bump();
        }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some('0'..='9')) {
                any = true;
                self.bump();
            }
        }
        if !any {
            return self.err("Invalid number literal.");
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            let save = self.i;
            self.bump();
            if matches!(self.peek(), Some('-') | Some('+')) {
                self.bump();
            }
            let mut exp_any = false;
            while matches!(self.peek(), Some('0'..='9')) {
                exp_any = true;
                self.bump();
            }
            if !exp_any {
                self.i = save;
            }
        }
        let raw = &self.s[start..self.i];
        let n: f64 = raw.parse().map_err(|_| SnifParseError { message: "Invalid number literal.".to_string(), line: self.line, col: self.col })?;
        Ok(SnifValue::Number(n))
    }

    fn parse_array(&mut self) -> Result<SnifValue, SnifParseError> {
        self.expect('[', "to start array")?;
        let mut items = Vec::new();
        self.skip_ws_and_comments();
        if self.consume(']') {
            return Ok(SnifValue::Array(items));
        }
        loop {
            let v = self.parse_value()?;
            items.push(v);
            self.skip_ws_and_comments();
            if self.consume(']') {
                return Ok(SnifValue::Array(items));
            }
            if self.consume(',') {
                self.skip_ws_and_comments();
                if self.consume(']') {
                    return Ok(SnifValue::Array(items));
                }
                continue;
            }
            return self.err("Expected ',' or ']' in array.");
        }
    }

    fn parse_object(&mut self) -> Result<SnifValue, SnifParseError> {
        self.expect('{', "to start object")?;
        let mut map = BTreeMap::new();
        self.skip_ws_and_comments();
        if self.consume('}') {
            return Ok(SnifValue::Object(map));
        }
        loop {
            let key = self.parse_key()?;
            self.skip_ws_and_comments();
            if !self.consume(':') {
                return self.err("Expected ':' after key.");
            }
            let value = self.parse_value()?;
            if map.contains_key(&key) {
                return self.err("Duplicate key in object.");
            }
            map.insert(key, value);
            self.skip_ws_and_comments();
            if self.consume('}') {
                return Ok(SnifValue::Object(map));
            }
            if self.consume(',') {
                self.skip_ws_and_comments();
                if self.consume('}') {
                    return Ok(SnifValue::Object(map));
                }
                continue;
            }
            return self.err("Expected ',' or '}' in object.");
        }
    }

    fn parse_typed_literal(&mut self) -> Result<SnifValue, SnifParseError> {
        self.expect('@', "to start typed literal")?;
        let t = self.parse_identifier()?;
        let payload = self.parse_string()?;
        let mut o = BTreeMap::new();
        match t.as_str() {
            "date" => {
                o.insert("$date".to_string(), SnifValue::String(payload));
            }
            "dec" => {
                o.insert("$dec".to_string(), SnifValue::String(payload));
            }
            "bin" => {
                o.insert("$bin".to_string(), SnifValue::String(payload));
            }
            "enum" => {
                o.insert("$enum".to_string(), SnifValue::String(payload));
            }
            _ => {
                o.insert("$type".to_string(), SnifValue::String(t));
                o.insert("value".to_string(), SnifValue::String(payload));
            }
        }
        Ok(SnifValue::Object(o))
    }

    fn parse_reference_define(&mut self) -> Result<SnifValue, SnifParseError> {
        self.err("References are not supported in SPS manifests.")
    }

    fn parse_reference_use(&mut self) -> Result<SnifValue, SnifParseError> {
        self.err("References are not supported in SPS manifests.")
    }

    fn parse_value(&mut self) -> Result<SnifValue, SnifParseError> {
        self.skip_ws_and_comments();
        let Some(c) = self.peek() else { return self.err("Unexpected end of input."); };
        match c {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' | '\'' => Ok(SnifValue::String(self.parse_string()?)),
            '@' => self.parse_typed_literal(),
            '&' => self.parse_reference_define(),
            '*' => self.parse_reference_use(),
            '-' | '+' | '0'..='9' => self.parse_number(),
            _ => {
                if c.is_ascii_alphabetic() || c == '_' || c == '$' {
                    let id = self.parse_identifier()?;
                    return match id.as_str() {
                        "true" => Ok(SnifValue::Bool(true)),
                        "false" => Ok(SnifValue::Bool(false)),
                        "null" => Ok(SnifValue::Null),
                        _ => Err(SnifParseError {
                            message: "Barewords are not allowed. Use a quoted string (\"...\") or a typed literal like @enum\"NAME\".".to_string(),
                            line: self.line,
                            col: self.col,
                        }),
                    };
                }
                self.err("Unexpected token while parsing value.")
            }
        }
    }
}

pub fn parse_snif(src: &str) -> Result<SnifValue, SnifParseError> {
    let mut p = Parser::new(src);
    let v = p.parse_value()?;
    p.skip_ws_and_comments();
    if !p.eof() {
        return Err(SnifParseError { message: "Unexpected trailing characters.".to_string(), line: p.line, col: p.col });
    }
    Ok(v)
}

