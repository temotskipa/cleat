#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub span: usize,
    pub kind: TokenKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Int(i128),
    Package,
    Import,
    Public,
    Private,
    Class,
    Static,
    Final,
    Open,
    Sealed,
    Extends,
    Permits,
    Void,
    Return,
    If,
    Else,
    While,
    New,
    True,
    False,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,
    Comma,
    Dot,
    Star,
    Plus,
    Minus,
    Slash,
    Percent,
    Eq,
    EqEq,
    BangEq,
    Lt,
    Gt,
    Le,
    Ge,
    Eof,
}

pub struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    i: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            i: 0,
        }
    }

    pub fn next(&mut self) -> Result<Token, String> {
        self.skip()?;
        let start = self.i;
        let Some(c) = self.peek() else {
            return Ok(Token {
                span: start,
                kind: TokenKind::Eof,
            });
        };
        let kind = match c {
            b'{' => {
                self.bump();
                TokenKind::LBrace
            }
            b'}' => {
                self.bump();
                TokenKind::RBrace
            }
            b'(' => {
                self.bump();
                TokenKind::LParen
            }
            b')' => {
                self.bump();
                TokenKind::RParen
            }
            b';' => {
                self.bump();
                TokenKind::Semicolon
            }
            b',' => {
                self.bump();
                TokenKind::Comma
            }
            b'.' => {
                self.bump();
                TokenKind::Dot
            }
            b'*' => {
                self.bump();
                TokenKind::Star
            }
            b'+' => {
                self.bump();
                TokenKind::Plus
            }
            b'-' => {
                self.bump();
                TokenKind::Minus
            }
            b'/' => {
                self.bump();
                TokenKind::Slash
            }
            b'%' => {
                self.bump();
                TokenKind::Percent
            }
            b'=' => {
                self.bump();
                if self.peek() == Some(b'=') {
                    self.bump();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            b'!' => {
                self.bump();
                if self.peek() == Some(b'=') {
                    self.bump();
                    TokenKind::BangEq
                } else {
                    return Err("expected != at this position".into());
                }
            }
            b'<' => {
                self.bump();
                if self.peek() == Some(b'=') {
                    self.bump();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            b'>' => {
                self.bump();
                if self.peek() == Some(b'=') {
                    self.bump();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            b'0'..=b'9' => self.number()?,
            b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'$' => self.ident(),
            _ => return Err(format!("unexpected character '{}'", c as char)),
        };
        Ok(Token { span: start, kind })
    }

    fn number(&mut self) -> Result<TokenKind, String> {
        let start = self.i;
        if self.peek() == Some(b'0') && matches!(self.peek_at(1), Some(b'x' | b'X' | b'b' | b'B')) {
            self.bump();
            let (radix, name) = if matches!(self.bump(), Some(b'x' | b'X')) {
                (16, "hexadecimal")
            } else {
                (2, "binary")
            };
            let digits = self.digits(radix)?;
            if digits.is_empty() {
                return Err(format!("expected {name} digits"));
            }
            return parse_int(&digits, radix);
        }
        if self.peek() == Some(b'0') && matches!(self.peek_at(1), Some(b'0'..=b'9')) {
            return Err("octal and leading-zero integers are rejected".into());
        }
        let digits = self.digits(10)?;
        let _ = start;
        parse_int(&digits, 10)
    }

    fn digits(&mut self, radix: u32) -> Result<String, String> {
        let mut out = String::new();
        let mut prev_underscore = false;
        while let Some(c) = self.peek() {
            if c == b'_' {
                if out.is_empty() || prev_underscore {
                    return Err("'_' must sit between digits".into());
                }
                prev_underscore = true;
                self.bump();
                continue;
            }
            let digit = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => break,
            };
            if u32::from(digit) >= radix {
                break;
            }
            prev_underscore = false;
            out.push(c as char);
            self.bump();
        }
        if prev_underscore {
            return Err("'_' must sit between digits".into());
        }
        Ok(out)
    }

    fn ident(&mut self) -> TokenKind {
        let start = self.i;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                self.bump();
            } else {
                break;
            }
        }
        match &self.src[start..self.i] {
            "package" => TokenKind::Package,
            "import" => TokenKind::Import,
            "public" => TokenKind::Public,
            "private" => TokenKind::Private,
            "class" => TokenKind::Class,
            "static" => TokenKind::Static,
            "final" => TokenKind::Final,
            "open" => TokenKind::Open,
            "sealed" => TokenKind::Sealed,
            "extends" => TokenKind::Extends,
            "permits" => TokenKind::Permits,
            "void" => TokenKind::Void,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "new" => TokenKind::New,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            other => TokenKind::Ident(other.to_string()),
        }
    }

    fn skip(&mut self) -> Result<(), String> {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
                self.bump();
            }
            if self.peek() == Some(b'/') && self.peek_at(1) == Some(b'/') {
                while let Some(c) = self.peek() {
                    self.bump();
                    if c == b'\n' {
                        break;
                    }
                }
                continue;
            }
            if self.peek() == Some(b'/') && self.peek_at(1) == Some(b'*') {
                self.bump();
                self.bump();
                loop {
                    match self.bump() {
                        Some(b'*') if self.peek() == Some(b'/') => {
                            self.bump();
                            break;
                        }
                        Some(_) => {}
                        None => return Err("unclosed comment".into()),
                    }
                }
                continue;
            }
            break;
        }
        Ok(())
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.i).copied()
    }

    fn peek_at(&self, n: usize) -> Option<u8> {
        self.bytes.get(self.i + n).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.i += 1;
        Some(c)
    }
}

fn parse_int(digits: &str, radix: u32) -> Result<TokenKind, String> {
    i128::from_str_radix(digits, radix)
        .map(TokenKind::Int)
        .map_err(|_| format!("integer literal {digits} does not fit in the compiler's literal range"))
}
