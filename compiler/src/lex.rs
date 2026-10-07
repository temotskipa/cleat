//! Tokens of chapter 1 and section 11.1.

use crate::Diagnostic;
use std::path::Path;

pub type Pos = u32;

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    /// Every word, keyword or not. The parser decides, because several keywords
    /// are keywords in one position only.
    Ident(String),
    /// Digits with `_` removed, and the radix.
    Int(String, u32),
    /// A decimal literal as written, with `_` removed.
    Dec(String),
    Char(char),
    Str(String),
    /// A separator or an operator.
    Punct(&'static str),
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

pub const KEYWORDS: &[&str] = &[
    "abstract", "assert", "break", "case", "catch", "class", "continue", "else", "enum",
    "extends", "final", "finally", "for", "if", "implements", "import", "instanceof",
    "interface", "new", "package", "private", "protected", "public", "return", "static",
    "super", "switch", "this", "throw", "try", "using", "var", "void", "while",
];

pub const RESERVED: &[&str] = &[
    "boolean", "byte", "char", "double", "float", "inline", "int", "long", "short", "when",
];

pub const WORD_LITERALS: &[&str] = &["true", "false", "null"];

/// Longest first, so that maximal munch is a linear scan. `>` is always a token of its
/// own, because it also closes type argument lists. The parser joins adjacent `>` and
/// `=` tokens into `>>`, `>=` and `>>=`.
const PUNCTS: &[&str] = &[
    "<<=", "...", "->", "::", "++", "--", "&&", "||", "??", "==", "!=",
    "<=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", "(", ")", "{", "}",
    "[", "]", ";", ",", ".", "@", "+", "-", "*", "/", "%", "&", "|", "^", "~", "!", "<", ">",
    "?", ":", "=",
];

pub fn line_col(text: &str, pos: Pos) -> (u32, u32) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in text.char_indices() {
        if i as u32 >= pos {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn diag(file: &Path, text: &str, pos: Pos, message: impl Into<String>) -> Diagnostic {
    let (line, column) = line_col(text, pos);
    Diagnostic { file: file.to_path_buf(), line, column, message: message.into() }
}

struct Lexer<'a> {
    file: &'a Path,
    text: &'a str,
    chars: Vec<(usize, char)>,
    i: usize,
    out: Vec<Token>,
}

pub fn lex(file: &Path, text: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut lx = Lexer { file, text, chars: text.char_indices().collect(), i: 0, out: Vec::new() };
    lx.run()?;
    Ok(lx.out)
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

fn is_ident_part(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic() || c.is_numeric()
}

impl<'a> Lexer<'a> {
    fn err<T>(&self, at: usize, message: impl Into<String>) -> Result<T, Diagnostic> {
        Err(diag(self.file, self.text, self.offset(at), message))
    }

    fn offset(&self, at: usize) -> Pos {
        self.chars.get(at).map(|c| c.0).unwrap_or(self.text.len()) as Pos
    }

    fn peek(&self, n: usize) -> Option<char> {
        self.chars.get(self.i + n).map(|c| c.1)
    }

    fn starts_with(&self, s: &str) -> bool {
        s.chars().enumerate().all(|(k, c)| self.peek(k) == Some(c))
    }

    fn push(&mut self, start: usize, tok: Tok) {
        let pos = self.offset(start);
        self.out.push(Token { tok, pos });
    }

    fn run(&mut self) -> Result<(), Diagnostic> {
        while let Some(c) = self.peek(0) {
            let start = self.i;
            if matches!(c, '\t' | '\n' | '\u{000B}' | '\u{000C}' | '\r' | ' ') {
                self.i += 1;
            } else if self.starts_with("//") {
                while let Some(c) = self.peek(0) {
                    if c == '\n' || c == '\r' {
                        break;
                    }
                    self.i += 1;
                }
            } else if self.starts_with("/*") {
                self.i += 2;
                loop {
                    if self.peek(0).is_none() {
                        return self.err(start, "this comment is never closed");
                    }
                    if self.starts_with("*/") {
                        self.i += 2;
                        break;
                    }
                    self.i += 1;
                }
            } else if is_ident_start(c) {
                let mut word = String::new();
                while let Some(c) = self.peek(0) {
                    if !is_ident_part(c) {
                        break;
                    }
                    word.push(c);
                    self.i += 1;
                }
                self.push(start, Tok::Ident(word));
            } else if c.is_ascii_digit() {
                self.number(start)?;
            } else if self.starts_with("\"\"\"") {
                self.text_block(start)?;
            } else if c == '"' {
                self.i += 1;
                let mut s = String::new();
                loop {
                    match self.peek(0) {
                        None | Some('\n') | Some('\r') => {
                            return self.err(start, "this string literal is never closed")
                        }
                        Some('"') => {
                            self.i += 1;
                            break;
                        }
                        Some('\\') => s.push(self.escape()?),
                        Some(c) => {
                            s.push(c);
                            self.i += 1;
                        }
                    }
                }
                self.push(start, Tok::Str(s));
            } else if c == '\'' {
                self.i += 1;
                let ch = match self.peek(0) {
                    None | Some('\n') | Some('\r') | Some('\'') => {
                        return self.err(start, "a character literal holds exactly one scalar")
                    }
                    Some('\\') => self.escape()?,
                    Some(c) => {
                        self.i += 1;
                        c
                    }
                };
                if self.peek(0) != Some('\'') {
                    return self.err(start, "a character literal holds exactly one scalar");
                }
                self.i += 1;
                self.push(start, Tok::Char(ch));
            } else {
                let Some(p) = PUNCTS.iter().find(|p| self.starts_with(p)) else {
                    return self.err(start, format!("unexpected character '{c}'"));
                };
                self.i += p.chars().count();
                self.push(start, Tok::Punct(p));
            }
        }
        let end = self.chars.len();
        self.push(end, Tok::Eof);
        Ok(())
    }

    fn digits(&mut self, start: usize, radix: u32, out: &mut String) -> Result<(), Diagnostic> {
        let mut last_was_digit = false;
        let mut any = false;
        loop {
            match self.peek(0) {
                Some(c) if c.is_digit(radix) => {
                    out.push(c);
                    self.i += 1;
                    last_was_digit = true;
                    any = true;
                }
                Some('_') => {
                    let next_is_digit = self.peek(1).map(|c| c.is_digit(radix)).unwrap_or(false);
                    if !last_was_digit || !next_is_digit {
                        return self.err(start, "`_` in a number stands between two digits");
                    }
                    self.i += 1;
                    last_was_digit = false;
                }
                _ => break,
            }
        }
        if !any {
            return self.err(start, "a number needs at least one digit here");
        }
        Ok(())
    }

    fn number(&mut self, start: usize) -> Result<(), Diagnostic> {
        if self.starts_with("0x") || self.starts_with("0X") {
            self.i += 2;
            let mut d = String::new();
            self.digits(start, 16, &mut d)?;
            self.no_suffix(start)?;
            self.push(start, Tok::Int(d, 16));
            return Ok(());
        }
        if self.starts_with("0b") || self.starts_with("0B") {
            self.i += 2;
            let mut d = String::new();
            self.digits(start, 2, &mut d)?;
            self.no_suffix(start)?;
            self.push(start, Tok::Int(d, 2));
            return Ok(());
        }
        let mut d = String::new();
        self.digits(start, 10, &mut d)?;
        if d.len() > 1 && d.starts_with('0') {
            return self.err(start, "a decimal numeral has no leading zero");
        }
        let mut is_decimal = false;
        // A `.` begins a fraction only when a digit follows, so `1.plus(2)` and `a[1..` stay calls.
        if self.peek(0) == Some('.') && self.peek(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
            is_decimal = true;
            self.i += 1;
            d.push('.');
            self.digits(start, 10, &mut d)?;
        }
        if matches!(self.peek(0), Some('e') | Some('E')) {
            let sign = matches!(self.peek(1), Some('+') | Some('-'));
            let digit_at = if sign { 2 } else { 1 };
            if self.peek(digit_at).map(|c| c.is_ascii_digit()).unwrap_or(false) {
                is_decimal = true;
                d.push('e');
                self.i += 1;
                if sign {
                    d.push(self.peek(0).unwrap());
                    self.i += 1;
                }
                self.digits(start, 10, &mut d)?;
            }
        }
        self.no_suffix(start)?;
        self.push(start, if is_decimal { Tok::Dec(d) } else { Tok::Int(d, 10) });
        Ok(())
    }

    fn no_suffix(&self, start: usize) -> Result<(), Diagnostic> {
        match self.peek(0) {
            Some(c) if is_ident_part(c) => {
                self.err(start, "a numeric literal has no suffix and is not followed by a letter")
            }
            _ => Ok(()),
        }
    }

    /// Reads an escape that begins at the backslash.
    fn escape(&mut self) -> Result<char, Diagnostic> {
        let start = self.i;
        self.i += 1;
        let Some(c) = self.peek(0) else {
            return self.err(start, "an escape is cut off by the end of the file");
        };
        self.i += 1;
        let simple = match c {
            'n' => Some('\n'),
            't' => Some('\t'),
            'r' => Some('\r'),
            '\\' => Some('\\'),
            '"' => Some('"'),
            '\'' => Some('\''),
            _ => None,
        };
        if let Some(ch) = simple {
            return Ok(ch);
        }
        if c != 'u' {
            return self.err(start, format!("`\\{c}` is not an escape"));
        }
        let mut hex = String::new();
        if self.peek(0) == Some('{') {
            self.i += 1;
            while let Some(c) = self.peek(0) {
                if c == '}' {
                    break;
                }
                hex.push(c);
                self.i += 1;
            }
            if self.peek(0) != Some('}') || hex.is_empty() || hex.len() > 6 {
                return self.err(start, "`\\u{...}` takes one to six hexadecimal digits");
            }
            self.i += 1;
        } else {
            for _ in 0..4 {
                match self.peek(0) {
                    Some(c) if c.is_ascii_hexdigit() => {
                        hex.push(c);
                        self.i += 1;
                    }
                    _ => return self.err(start, "`\\u` takes exactly four hexadecimal digits"),
                }
            }
        }
        let Ok(n) = u32::from_str_radix(&hex, 16) else {
            return self.err(start, "`\\u{...}` takes one to six hexadecimal digits");
        };
        match char::from_u32(n) {
            Some(ch) => Ok(ch),
            None => self.err(start, "this escape names a surrogate or a number above 10FFFF"),
        }
    }

    fn text_block(&mut self, start: usize) -> Result<(), Diagnostic> {
        self.i += 3;
        while matches!(self.peek(0), Some(' ') | Some('\t')) {
            self.i += 1;
        }
        match self.peek(0) {
            Some('\r') if self.peek(1) == Some('\n') => self.i += 2,
            Some('\n') | Some('\r') => self.i += 1,
            _ => return self.err(start, "a text block opens with `\"\"\"` and the end of the line"),
        }
        // Raw lines up to the closing delimiter. Escapes are kept as written until the
        // indent has been removed.
        let mut lines: Vec<Vec<char>> = vec![Vec::new()];
        loop {
            match self.peek(0) {
                None => return self.err(start, "this text block is never closed"),
                Some('"') if self.starts_with("\"\"\"") => {
                    self.i += 3;
                    break;
                }
                Some('\r') if self.peek(1) == Some('\n') => {
                    self.i += 2;
                    lines.push(Vec::new());
                }
                Some('\n') | Some('\r') => {
                    self.i += 1;
                    lines.push(Vec::new());
                }
                Some('\\') => {
                    let line = lines.last_mut().unwrap();
                    line.push('\\');
                    self.i += 1;
                    if let Some(c) = self.peek(0) {
                        line.push(c);
                        self.i += 1;
                    }
                }
                Some(c) => {
                    lines.last_mut().unwrap().push(c);
                    self.i += 1;
                }
            }
        }
        let indent = lines.pop().unwrap();
        if indent.iter().any(|c| *c != ' ' && *c != '\t') {
            return self.err(start, "the closing `\"\"\"` of a text block stands on a line of its own");
        }
        let mut raw = String::new();
        for (n, line) in lines.iter().enumerate() {
            if line.len() < indent.len() || line[..indent.len()] != indent[..] {
                if !line.iter().all(|c| *c == ' ' || *c == '\t') {
                    return self.err(start, "every line of a text block begins with its indent");
                }
            } else {
                raw.extend(line[indent.len()..].iter());
            }
            if n + 1 < lines.len() {
                raw.push('\n');
            }
        }
        // Apply escapes with a nested lexer over the unindented content.
        let mut inner = Lexer {
            file: self.file,
            text: &raw,
            chars: raw.char_indices().collect(),
            i: 0,
            out: Vec::new(),
        };
        let mut s = String::new();
        while let Some(c) = inner.peek(0) {
            if c == '\\' {
                match inner.escape() {
                    Ok(ch) => s.push(ch),
                    Err(e) => return self.err(start, e.message),
                }
            } else {
                s.push(c);
                inner.i += 1;
            }
        }
        self.push(start, Tok::Str(s));
        Ok(())
    }
}
