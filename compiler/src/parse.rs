use crate::ast::{
    Audience, BinOp, Block, Expr, ExprKind, Field, Import, Method, Param, ResultType, Stmt,
    TypeDecl, Unit,
};
use crate::lex::{Lexer, Token, TokenKind};
use std::path::Path;

pub struct ParseError {
    pub span: usize,
    pub message: String,
}

pub fn parse_unit(path: &Path, source: &str) -> Result<Unit, crate::check::Diagnostic> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        match lexer.next() {
            Ok(tok) => {
                let done = tok.kind == TokenKind::Eof;
                tokens.push(tok);
                if done {
                    break;
                }
            }
            Err(message) => {
                let (line, column) = line_col(source, lexer_index(&tokens, source));
                return Err(crate::check::Diagnostic {
                    file: path.to_path_buf(),
                    line,
                    column,
                    message,
                });
            }
        }
    }
    let mut parser = Parser {
        tokens: &tokens,
        i: 0,
    };
    match parser.unit() {
        Ok(mut unit) => {
            unit.file = path.to_path_buf();
            unit.source = source.to_string();
            Ok(unit)
        }
        Err(err) => {
            let (line, column) = line_col(source, err.span);
            Err(crate::check::Diagnostic {
                file: path.to_path_buf(),
                line,
                column,
                message: err.message,
            })
        }
    }
}

fn lexer_index(tokens: &[Token], source: &str) -> usize {
    tokens.last().map(|t| t.span).unwrap_or(source.len())
}

pub fn line_col(source: &str, span: usize) -> (u32, u32) {
    let mut line = 1u32;
    let mut col = 1u32;
    for (idx, ch) in source.char_indices() {
        if idx >= span {
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

struct Parser<'a> {
    tokens: &'a [Token],
    i: usize,
}

enum Member {
    Method(Method),
    Field(Field),
}

impl<'a> Parser<'a> {
    fn unit(&mut self) -> Result<Unit, ParseError> {
        self.expect(&TokenKind::Package)?;
        let package = self.qualified_name()?;
        self.expect(&TokenKind::Semicolon)?;
        let mut imports = Vec::new();
        while self.at(&TokenKind::Import) {
            let span = self.peek_span();
            self.bump();
            let name = self.qualified_name()?;
            let mut star = false;
            if self.at(&TokenKind::Dot) {
                self.bump();
                self.expect(&TokenKind::Star)?;
                star = true;
            }
            self.expect(&TokenKind::Semicolon)?;
            imports.push(Import { span, name, star });
        }
        let mut types = Vec::new();
        while !self.at(&TokenKind::Eof) {
            types.push(self.type_decl()?);
        }
        Ok(Unit {
            file: Default::default(),
            source: String::new(),
            package,
            imports,
            types,
        })
    }

    fn type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        let span = self.peek_span();
        let mut audience = Audience::Private;
        let mut is_open = false;
        let mut is_sealed = false;
        let mut saw_class = false;
        loop {
            match &self.peek().kind {
                TokenKind::Public => audience = Audience::Public,
                TokenKind::Private => audience = Audience::Private,
                TokenKind::Package => audience = Audience::Package,
                TokenKind::Final => {}
                TokenKind::Open => is_open = true,
                TokenKind::Sealed => is_sealed = true,
                TokenKind::Class => {
                    saw_class = true;
                    self.bump();
                    break;
                }
                _ => break,
            }
            self.bump();
        }
        if !saw_class {
            return self.err("expected a class declaration");
        }
        let name = self.ident()?;
        let mut extends = None;
        if self.at(&TokenKind::Extends) {
            self.bump();
            extends = Some(self.ident()?);
        }
        let mut permits = Vec::new();
        if self.at(&TokenKind::Permits) {
            self.bump();
            permits.push(self.ident()?);
            while self.at(&TokenKind::Comma) {
                self.bump();
                permits.push(self.ident()?);
            }
        }
        self.expect(&TokenKind::LBrace)?;
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            match self.member()? {
                Member::Method(method) => methods.push(method),
                Member::Field(field) => fields.push(field),
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(TypeDecl {
            span,
            audience,
            is_open,
            is_sealed,
            name,
            extends,
            permits,
            fields,
            methods,
        })
    }

    fn member(&mut self) -> Result<Member, ParseError> {
        let span = self.peek_span();
        let mut audience = Audience::Private;
        let mut is_static = false;
        let mut is_open = false;
        loop {
            match &self.peek().kind {
                TokenKind::Public => audience = Audience::Public,
                TokenKind::Private => audience = Audience::Private,
                TokenKind::Package => audience = Audience::Package,
                TokenKind::Static => is_static = true,
                TokenKind::Final => {}
                TokenKind::Open => is_open = true,
                _ => break,
            }
            self.bump();
        }
        if self.at(&TokenKind::Void) {
            self.bump();
            let name = self.ident()?;
            let params = self.params()?;
            let only = self.only_clause()?;
            let body = self.block()?;
            return Ok(Member::Method(Method {
                span,
                audience,
                is_static,
                is_open,
                result: ResultType::Void,
                name,
                params,
                only,
                body,
            }));
        }
        let nullable = self.nullable_ann()?;
        let ty = self.type_name()?;
        let name = self.ident()?;
        if self.at(&TokenKind::LParen) {
            let params = self.params()?;
            let only = self.only_clause()?;
            let body = self.block()?;
            return Ok(Member::Method(Method {
                span,
                audience,
                is_static,
                is_open,
                result: ResultType::Named { name: ty, nullable },
                name,
                params,
                only,
                body,
            }));
        }
        if is_open {
            return self.err("open is a method modifier (section 4.1)");
        }
        let only = self.only_clause()?;
        let init = if self.at(&TokenKind::Eq) {
            self.bump();
            Some(self.expression()?)
        } else {
            None
        };
        self.expect(&TokenKind::Semicolon)?;
        Ok(Member::Field(Field {
            span,
            audience,
            is_static,
            nullable,
            ty,
            name,
            init,
            only,
        }))
    }

    fn params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.at(&TokenKind::RParen) {
            loop {
                let nullable = self.nullable_ann()?;
                let ty = self.type_name()?;
                let name = self.ident()?;
                params.push(Param { name, ty, nullable });
                if self.at(&TokenKind::Comma) {
                    self.bump();
                } else {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;
        Ok(params)
    }

    fn only_clause(&mut self) -> Result<Vec<String>, ParseError> {
        let mut names = Vec::new();
        if self.at(&TokenKind::Only) {
            self.bump();
            names.push(self.ident()?);
            while self.at(&TokenKind::Comma) {
                self.bump();
                names.push(self.ident()?);
            }
        }
        Ok(names)
    }

    fn nullable_ann(&mut self) -> Result<bool, ParseError> {
        if !self.at(&TokenKind::At) {
            return Ok(false);
        }
        self.bump();
        let name = self.ident()?;
        if name != "Nullable" {
            return self.err("the type-use annotation is @Nullable (section 5.2)");
        }
        Ok(true)
    }

    fn block(&mut self) -> Result<Block, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            stmts.push(self.statement()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Block { stmts })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.at(&TokenKind::If) {
            let span = self.peek_span();
            self.bump();
            self.expect(&TokenKind::LParen)?;
            let cond = self.expression()?;
            self.expect(&TokenKind::RParen)?;
            let then_body = self.block_or_stmt()?;
            let else_body = if self.at(&TokenKind::Else) {
                self.bump();
                Some(self.block_or_stmt()?)
            } else {
                None
            };
            return Ok(Stmt::If {
                span,
                cond,
                then_body,
                else_body,
            });
        }
        if self.at(&TokenKind::While) {
            let span = self.peek_span();
            self.bump();
            self.expect(&TokenKind::LParen)?;
            let cond = self.expression()?;
            self.expect(&TokenKind::RParen)?;
            let body = self.block_or_stmt()?;
            return Ok(Stmt::While { span, cond, body });
        }
        if self.at(&TokenKind::Return) {
            let span = self.peek_span();
            self.bump();
            let expr = if self.at(&TokenKind::Semicolon) {
                None
            } else {
                Some(self.expression()?)
            };
            self.expect(&TokenKind::Semicolon)?;
            return Ok(Stmt::Return { span, expr });
        }
        if self.at(&TokenKind::LBrace) {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.at(&TokenKind::At) || self.looks_like_decl() {
            let span = self.peek_span();
            let nullable = self.nullable_ann()?;
            let ty = self.type_name()?;
            let name = self.ident()?;
            let init = if self.at(&TokenKind::Eq) {
                self.bump();
                Some(self.expression()?)
            } else {
                None
            };
            self.expect(&TokenKind::Semicolon)?;
            return Ok(Stmt::Local {
                span,
                ty,
                name,
                nullable,
                init,
            });
        }
        let span = self.peek_span();
        let expr = self.expression()?;
        if self.at(&TokenKind::Eq) {
            let (name, owner) = match expr.kind {
                ExprKind::Name(name) => (name, None),
                ExprKind::Select(recv, name) => {
                    let object = *recv;
                    self.bump();
                    let value = self.expression()?;
                    self.expect(&TokenKind::Semicolon)?;
                    return Ok(Stmt::SetField {
                        span,
                        object,
                        name,
                        expr: value,
                    });
                }
                ExprKind::Index(array, index) => {
                    self.bump();
                    let value = self.expression()?;
                    self.expect(&TokenKind::Semicolon)?;
                    return Ok(Stmt::SetIndex {
                        span,
                        array: *array,
                        index: *index,
                        expr: value,
                    });
                }
                _ => return self.err("assignment target must be a local, field, or array element"),
            };
            self.bump();
            let value = self.expression()?;
            self.expect(&TokenKind::Semicolon)?;
            return Ok(Stmt::Assign {
                span,
                name,
                owner,
                expr: value,
            });
        }
        self.expect(&TokenKind::Semicolon)?;
        Ok(Stmt::Expr { span, expr })
    }

    fn block_or_stmt(&mut self) -> Result<Block, ParseError> {
        if self.at(&TokenKind::LBrace) {
            self.block()
        } else {
            let stmt = self.statement()?;
            Ok(Block { stmts: vec![stmt] })
        }
    }

    fn looks_like_decl(&self) -> bool {
        let Some(TokenKind::Ident(_)) = self.tokens.get(self.i).map(|t| &t.kind) else {
            return false;
        };
        let mut next = self.i + 1;
        if matches!(self.tokens.get(next).map(|t| &t.kind), Some(TokenKind::LBracket))
            && matches!(
                self.tokens.get(next + 1).map(|t| &t.kind),
                Some(TokenKind::RBracket)
            )
        {
            next += 2;
        }
        matches!(
            self.tokens.get(next).map(|t| &t.kind),
            Some(TokenKind::Ident(_))
        )
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.comparison()?;
        loop {
            let op = if self.at(&TokenKind::EqEq) {
                BinOp::Eq
            } else if self.at(&TokenKind::BangEq) {
                BinOp::Ne
            } else {
                break;
            };
            let span = self.peek_span();
            self.bump();
            let right = self.comparison()?;
            left = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            };
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.term()?;
        loop {
            let op = if self.at(&TokenKind::Lt) {
                BinOp::Lt
            } else if self.at(&TokenKind::Gt) {
                BinOp::Gt
            } else if self.at(&TokenKind::Le) {
                BinOp::Le
            } else if self.at(&TokenKind::Ge) {
                BinOp::Ge
            } else {
                break;
            };
            let span = self.peek_span();
            self.bump();
            let right = self.term()?;
            left = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            };
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.factor()?;
        loop {
            let op = if self.at(&TokenKind::Plus) {
                BinOp::Add
            } else if self.at(&TokenKind::Minus) {
                BinOp::Sub
            } else {
                break;
            };
            let span = self.peek_span();
            self.bump();
            let right = self.factor()?;
            left = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            };
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        loop {
            let op = if self.at(&TokenKind::Star) {
                BinOp::Mul
            } else if self.at(&TokenKind::Slash) {
                BinOp::Div
            } else if self.at(&TokenKind::Percent) {
                BinOp::Rem
            } else {
                break;
            };
            let span = self.peek_span();
            self.bump();
            let right = self.unary()?;
            left = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(left), Box::new(right)),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.at(&TokenKind::Minus) {
            let span = self.peek_span();
            self.bump();
            let expr = self.unary()?;
            return Ok(Expr {
                span,
                kind: ExprKind::UnaryNeg(Box::new(expr)),
            });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.at(&TokenKind::Dot) {
                self.bump();
                let name = self.ident()?;
                expr = Expr {
                    span: expr.span,
                    kind: ExprKind::Select(Box::new(expr), name),
                };
            } else if self.at(&TokenKind::LBracket) {
                self.bump();
                let index = self.expression()?;
                self.expect(&TokenKind::RBracket)?;
                expr = Expr {
                    span: expr.span,
                    kind: ExprKind::Index(Box::new(expr), Box::new(index)),
                };
            } else if self.at(&TokenKind::LParen) {
                self.bump();
                let mut args = Vec::new();
                if !self.at(&TokenKind::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if self.at(&TokenKind::Comma) {
                            self.bump();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RParen)?;
                expr = Expr {
                    span: expr.span,
                    kind: ExprKind::Call(Box::new(expr), args),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek_span();
        match &self.peek().kind {
            TokenKind::Int(n) => {
                let n = *n;
                self.bump();
                Ok(Expr {
                    span,
                    kind: ExprKind::Int(n),
                })
            }
            TokenKind::True => {
                self.bump();
                Ok(Expr {
                    span,
                    kind: ExprKind::Bool(true),
                })
            }
            TokenKind::False => {
                self.bump();
                Ok(Expr {
                    span,
                    kind: ExprKind::Bool(false),
                })
            }
            TokenKind::Null => {
                self.bump();
                Ok(Expr {
                    span,
                    kind: ExprKind::Null,
                })
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.bump();
                Ok(Expr {
                    span,
                    kind: ExprKind::Name(name),
                })
            }
            TokenKind::New => {
                self.bump();
                let name = self.type_name()?;
                if self.at(&TokenKind::LBracket) {
                    self.bump();
                    let len = self.expression()?;
                    self.expect(&TokenKind::RBracket)?;
                    let elem = name.trim_end_matches("[]").to_string();
                    return Ok(Expr {
                        span,
                        kind: ExprKind::NewArray {
                            elem,
                            len: Box::new(len),
                        },
                    });
                }
                self.expect(&TokenKind::LParen)?;
                self.expect(&TokenKind::RParen)?;
                if name == "Unit" {
                    return Ok(Expr {
                        span,
                        kind: ExprKind::NewUnit,
                    });
                }
                Ok(Expr {
                    span,
                    kind: ExprKind::NewClass(name),
                })
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.expression()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            _ => self.err("expected an expression"),
        }
    }

    fn qualified_name(&mut self) -> Result<String, ParseError> {
        let mut name = self.ident()?;
        while self.at(&TokenKind::Dot) && matches!(self.lookahead(1), Some(TokenKind::Ident(_))) {
            self.bump();
            name.push('.');
            name.push_str(&self.ident()?);
        }
        Ok(name)
    }

    fn type_name(&mut self) -> Result<String, ParseError> {
        let mut name = self.ident()?;
        while self.at(&TokenKind::LBracket)
            && matches!(self.lookahead(1), Some(TokenKind::RBracket))
        {
            self.bump();
            self.bump();
            name.push_str("[]");
        }
        Ok(name)
    }

    fn ident(&mut self) -> Result<String, ParseError> {
        match &self.peek().kind {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.bump();
                Ok(name)
            }
            _ => self.err("expected an identifier"),
        }
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<(), ParseError> {
        if self.same(&self.peek().kind, kind) {
            self.bump();
            Ok(())
        } else {
            self.err(&format!("expected {kind:?}"))
        }
    }

    fn same(&self, got: &TokenKind, want: &TokenKind) -> bool {
        std::mem::discriminant(got) == std::mem::discriminant(want)
    }

    fn at(&self, kind: &TokenKind) -> bool {
        self.same(&self.peek().kind, kind)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.i.min(self.tokens.len() - 1)]
    }

    fn peek_span(&self) -> usize {
        self.peek().span
    }

    fn lookahead(&self, n: usize) -> Option<&TokenKind> {
        self.tokens.get(self.i + n).map(|t| &t.kind)
    }

    fn bump(&mut self) {
        if self.i < self.tokens.len() {
            self.i += 1;
        }
    }

    fn err<T>(&self, message: &str) -> Result<T, ParseError> {
        Err(ParseError {
            span: self.peek_span(),
            message: message.to_string(),
        })
    }
}
