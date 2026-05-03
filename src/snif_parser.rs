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
    b: &'a [u8],
    i: usize,
    line: usize,
    col: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser {
            s,
            b: s.as_bytes(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn eof(&self) -> bool {
        self.i >= self.b.len()
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.i += 1;
        if c == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn err<T>(&self, message: impl Into<String>) -> Result<T, SnifParseError> {
        Err(SnifParseError {
            message: message.into(),
            line: self.line,
            col: self.col,
        })
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while let Some(c) = self.peek() {
                match c {
                    b' ' | b'\t' | b'\r' | b'\n' => {
                        self.bump();
                    }
                    _ => break,
                }
            }

            // // comment
            if self.peek() == Some(b'/') && self.b.get(self.i + 1) == Some(&b'/') {
                self.bump();
                self.bump();
                while let Some(c) = self.bump() {
                    if c == b'\n' {
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn consume(&mut self, ch: u8) -> bool {
        self.skip_ws_and_comments();
        if self.peek() == Some(ch) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, ch: u8, what: &str) -> Result<(), SnifParseError> {
        if self.consume(ch) {
            Ok(())
        } else {
            self.err(format!("Expected '{}' {what}.", ch as char))
        }
    }

    fn parse_string(&mut self) -> Result<String, SnifParseError> {
        self.skip_ws_and_comments();
        let q = match self.peek() {
            Some(b'"') => b'"',
            Some(b'\'') => b'\'',
            _ => return self.err("Expected string literal."),
        };
        self.bump();

        // Fast path: scan bytes, only decode UTF-8 when needed.
        let mut out: Vec<u8> = Vec::new();
        while let Some(c) = self.bump() {
            if c == q {
                return String::from_utf8(out).map_err(|_| SnifParseError {
                    message: "Invalid UTF-8 in string literal.".to_string(),
                    line: self.line,
                    col: self.col,
                });
            }
            if c == b'\\' {
                let Some(e) = self.bump() else {
                    return self.err("Unterminated escape sequence in string.");
                };
                match e {
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'\\' => out.push(b'\\'),
                    b'"' => out.push(b'"'),
                    b'\'' => out.push(b'\''),
                    other => out.push(other),
                }
                continue;
            }
            if c == b'\n' || c == b'\r' {
                return self.err("Unterminated string literal (newline).");
            }
            out.push(c);
        }
        self.err("Unterminated string literal (end of file).")
    }

    fn is_ident_start(c: u8) -> bool {
        (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_' || c == b'$'
    }

    fn is_ident_char(c: u8) -> bool {
        Self::is_ident_start(c) || (c >= b'0' && c <= b'9') || c == b'-'
    }

    fn parse_identifier(&mut self) -> Result<String, SnifParseError> {
        self.skip_ws_and_comments();
        let Some(c0) = self.peek() else {
            return self.err("Unexpected end of input.");
        };
        if !Self::is_ident_start(c0) {
            return self.err("Expected identifier.");
        }
        let start = self.i;
        self.bump();
        while let Some(c) = self.peek() {
            if !Self::is_ident_char(c) {
                break;
            }
            self.bump();
        }
        Ok(self.s[start..self.i].to_string())
    }

    fn parse_key(&mut self) -> Result<String, SnifParseError> {
        self.skip_ws_and_comments();
        match self.peek() {
            Some(b'"') | Some(b'\'') => self.parse_string(),
            _ => self.parse_identifier(),
        }
    }

    fn parse_number(&mut self) -> Result<SnifValue, SnifParseError> {
        self.skip_ws_and_comments();
        let start = self.i;
        if matches!(self.peek(), Some(b'-') | Some(b'+')) {
            self.bump();
        }

        let mut any = false;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            any = true;
            self.bump();
        }
        if self.peek() == Some(b'.') {
            self.bump();
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                any = true;
                self.bump();
            }
        }
        if !any {
            return self.err("Invalid number literal.");
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            let save = self.i;
            self.bump();
            if matches!(self.peek(), Some(b'-') | Some(b'+')) {
                self.bump();
            }
            let mut exp_any = false;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                exp_any = true;
                self.bump();
            }
            if !exp_any {
                self.i = save;
            }
        }
        let raw = &self.s[start..self.i];
        let n: f64 = raw.parse().map_err(|_| SnifParseError {
            message: "Invalid number literal.".to_string(),
            line: self.line,
            col: self.col,
        })?;
        Ok(SnifValue::Number(n))
    }

    fn parse_array(&mut self) -> Result<SnifValue, SnifParseError> {
        self.expect(b'[', "to start array")?;
        let mut items = Vec::new();
        self.skip_ws_and_comments();
        if self.consume(b']') {
            return Ok(SnifValue::Array(items));
        }
        loop {
            let v = self.parse_value()?;
            items.push(v);
            self.skip_ws_and_comments();
            if self.consume(b']') {
                return Ok(SnifValue::Array(items));
            }
            if self.consume(b',') {
                self.skip_ws_and_comments();
                if self.consume(b']') {
                    return Ok(SnifValue::Array(items));
                }
                continue;
            }
            return self.err("Expected ',' or ']' in array.");
        }
    }

    fn parse_object(&mut self) -> Result<SnifValue, SnifParseError> {
        self.expect(b'{', "to start object")?;
        let mut map = BTreeMap::new();
        self.skip_ws_and_comments();
        if self.consume(b'}') {
            return Ok(SnifValue::Object(map));
        }
        loop {
            let key = self.parse_key()?;
            self.skip_ws_and_comments();
            if !self.consume(b':') {
                return self.err("Expected ':' after key.");
            }
            let value = self.parse_value()?;
            if map.contains_key(&key) {
                return self.err("Duplicate key in object.");
            }
            map.insert(key, value);
            self.skip_ws_and_comments();
            if self.consume(b'}') {
                return Ok(SnifValue::Object(map));
            }
            if self.consume(b',') {
                self.skip_ws_and_comments();
                if self.consume(b'}') {
                    return Ok(SnifValue::Object(map));
                }
                continue;
            }
            return self.err("Expected ',' or '}' in object.");
        }
    }

    fn parse_typed_literal(&mut self) -> Result<SnifValue, SnifParseError> {
        self.expect(b'@', "to start typed literal")?;
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
        let Some(c) = self.peek() else {
            return self.err("Unexpected end of input.");
        };
        match c {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' | b'\'' => Ok(SnifValue::String(self.parse_string()?)),
            b'@' => self.parse_typed_literal(),
            b'&' => self.parse_reference_define(),
            b'*' => self.parse_reference_use(),
            b'-' | b'+' | b'0'..=b'9' => self.parse_number(),
            _ => {
                if Self::is_ident_start(c) {
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
        return Err(SnifParseError {
            message: "Unexpected trailing characters.".to_string(),
            line: p.line,
            col: p.col,
        });
    }
    Ok(v)
}
