//! A recursive-descent parser for the grammar of chapter 11.

use crate::ast::*;
use crate::lex::{self, Pos, Tok, Token, KEYWORDS, RESERVED, WORD_LITERALS};
use crate::Diagnostic;
use std::path::Path;

pub fn parse_unit(file: &Path, text: &str) -> Result<Unit, Diagnostic> {
    let toks = lex::lex(file, text)?;
    let mut p = Parser { file, text, toks, i: 0 };
    p.unit()
}

struct Parser<'a> {
    file: &'a Path,
    text: &'a str,
    toks: Vec<Token>,
    i: usize,
}

type R<T> = Result<T, Diagnostic>;

impl<'a> Parser<'a> {
    // ---- token access ----

    fn tok(&self) -> &Tok {
        &self.toks[self.i].tok
    }

    fn tok_at(&self, n: usize) -> &Tok {
        let k = (self.i + n).min(self.toks.len() - 1);
        &self.toks[k].tok
    }

    fn pos(&self) -> Pos {
        self.toks[self.i].pos
    }

    fn err<T>(&self, message: impl Into<String>) -> R<T> {
        Err(lex::diag(self.file, self.text, self.pos(), message))
    }

    fn err_at<T>(&self, pos: Pos, message: impl Into<String>) -> R<T> {
        Err(lex::diag(self.file, self.text, pos, message))
    }

    fn describe(&self) -> String {
        match self.tok() {
            Tok::Ident(s) => format!("`{s}`"),
            Tok::Int(s, _) | Tok::Dec(s) => format!("`{s}`"),
            Tok::Char(_) => "a character literal".into(),
            Tok::Str(_) => "a string literal".into(),
            Tok::Punct(p) => format!("`{p}`"),
            Tok::Eof => "the end of the file".into(),
        }
    }

    fn is_punct(&self, p: &str) -> bool {
        matches!(self.tok(), Tok::Punct(q) if *q == p)
    }

    fn is_punct_at(&self, n: usize, p: &str) -> bool {
        matches!(self.tok_at(n), Tok::Punct(q) if *q == p)
    }

    fn is_word(&self, w: &str) -> bool {
        matches!(self.tok(), Tok::Ident(s) if s == w)
    }

    fn is_word_at(&self, n: usize, w: &str) -> bool {
        matches!(self.tok_at(n), Tok::Ident(s) if s == w)
    }

    fn eat_punct(&mut self, p: &str) -> bool {
        if self.is_punct(p) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn expect_punct(&mut self, p: &str) -> R<()> {
        if self.eat_punct(p) {
            Ok(())
        } else {
            self.err(format!("expected `{p}`, found {}", self.describe()))
        }
    }

    fn expect_word(&mut self, w: &str) -> R<()> {
        if self.eat_word(w) {
            Ok(())
        } else {
            self.err(format!("expected `{w}`, found {}", self.describe()))
        }
    }

    /// Whether the token `n` ahead can be an identifier.
    fn is_ident_at(&self, n: usize) -> bool {
        match self.tok_at(n) {
            Tok::Ident(s) => {
                !KEYWORDS.contains(&s.as_str()) && !WORD_LITERALS.contains(&s.as_str())
            }
            _ => false,
        }
    }

    fn ident(&mut self) -> R<String> {
        match self.tok().clone() {
            Tok::Ident(s) => {
                if KEYWORDS.contains(&s.as_str()) || WORD_LITERALS.contains(&s.as_str()) {
                    return self.err(format!("`{s}` is a keyword and cannot be a name"));
                }
                if RESERVED.contains(&s.as_str()) {
                    return self.err(format!("`{s}` is a reserved word and cannot be a name"));
                }
                self.i += 1;
                Ok(s)
            }
            _ => self.err(format!("expected a name, found {}", self.describe())),
        }
    }

    /// Closes a type argument list.
    fn expect_gt(&mut self) -> R<()> {
        self.expect_punct(">")
    }

    /// Whether the token `n` ahead directly follows the one before it.
    fn glued(&self, n: usize) -> bool {
        let k = self.i + n;
        k > 0 && k < self.toks.len() && self.toks[k].pos == self.toks[k - 1].pos + 1
    }

    /// The operator at this point and how many tokens spell it. The lexer leaves `>`
    /// alone, so `>>`, `>=` and `>>=` are joined here.
    fn op(&self) -> Option<(&'static str, usize)> {
        match self.tok() {
            Tok::Punct(">") => {
                let gt2 = self.is_punct_at(1, ">") && self.glued(1);
                if gt2 && self.is_punct_at(2, "=") && self.glued(2) {
                    Some((">>=", 3))
                } else if gt2 {
                    Some((">>", 2))
                } else if self.is_punct_at(1, "=") && self.glued(1) {
                    Some((">=", 2))
                } else {
                    Some((">", 1))
                }
            }
            Tok::Punct(p) => Some((p, 1)),
            _ => None,
        }
    }

    /// Runs `f`, and puts the parser back where it was if `f` fails.
    fn attempt<T>(&mut self, f: impl FnOnce(&mut Self) -> R<T>) -> Option<T> {
        let saved = self.i;
        match f(self) {
            Ok(v) => Some(v),
            Err(_) => {
                self.i = saved;
                None
            }
        }
    }

    // ---- compilation unit ----

    fn unit(&mut self) -> R<Unit> {
        let mut package = Vec::new();
        if self.is_word("package") && self.is_ident_at(1) && !self.is_type_decl_ahead() {
            self.i += 1;
            package = self.qualified()?;
            self.expect_punct(";")?;
        }
        let mut imports = Vec::new();
        while self.is_word("import") {
            let pos = self.pos();
            self.i += 1;
            let mut path = vec![self.ident()?];
            let mut star = false;
            while self.eat_punct(".") {
                if self.eat_punct("*") {
                    star = true;
                    break;
                }
                path.push(self.ident()?);
            }
            self.expect_punct(";")?;
            imports.push(Import { path, star, pos });
        }
        let mut types = Vec::new();
        while !matches!(self.tok(), Tok::Eof) {
            types.push(self.type_decl()?);
        }
        Ok(Unit { file: self.file.to_path_buf(), text: self.text.to_string(), package, imports, types })
    }

    /// `package` also begins an audience, as in `package class Util`.
    fn is_type_decl_ahead(&self) -> bool {
        // After `package`, a type declaration continues with a modifier or a type keyword,
        // and a package declaration with a name followed by `.` or `;`.
        !(self.is_punct_at(2, ".") || self.is_punct_at(2, ";"))
    }

    fn qualified(&mut self) -> R<Vec<String>> {
        let mut path = vec![self.ident()?];
        while self.is_punct(".") && self.is_ident_at(1) {
            self.i += 1;
            path.push(self.ident()?);
        }
        Ok(path)
    }

    // ---- modifiers and annotations ----

    fn mods(&mut self) -> R<Mods> {
        let mut m = Mods { pos: self.pos(), ..Mods::default() };
        loop {
            if self.is_punct("@") {
                m.annotations.push(self.annotation()?);
                continue;
            }
            let Tok::Ident(w) = self.tok().clone() else { break };
            let dup = |p: &Self, set: bool| -> R<()> {
                if set {
                    p.err(format!("`{w}` is written twice"))
                } else {
                    Ok(())
                }
            };
            match w.as_str() {
                "private" | "package" | "protected" | "public" => {
                    if m.audience.is_some() {
                        return self.err("a declaration has one audience");
                    }
                    self.i += 1;
                    m.audience = Some(match w.as_str() {
                        "private" => Audience::Private,
                        "package" => Audience::Package,
                        "protected" => Audience::Protected,
                        _ => Audience::Public,
                    });
                    if self.is_word("only") && self.is_punct_at(1, "(") {
                        if w == "private" {
                            return self.err("`only` is not written after `private`");
                        }
                        self.i += 2;
                        let mut list = vec![self.type_ref()?];
                        while self.eat_punct(",") {
                            list.push(self.type_ref()?);
                        }
                        self.expect_punct(")")?;
                        m.only = Some(list);
                    }
                }
                "static" => {
                    dup(self, m.is_static)?;
                    self.i += 1;
                    m.is_static = true;
                }
                "final" => {
                    dup(self, m.is_final)?;
                    self.i += 1;
                    m.is_final = true;
                }
                "abstract" => {
                    dup(self, m.is_abstract)?;
                    self.i += 1;
                    m.is_abstract = true;
                }
                // The three contextual modifiers are modifiers only when a declaration
                // can still follow them.
                "open" | "sealed" | "foreign" if self.modifier_can_stand_here() => {
                    self.i += 1;
                    match w.as_str() {
                        "open" => {
                            dup(self, m.is_open)?;
                            m.is_open = true;
                        }
                        "sealed" => {
                            dup(self, m.is_sealed)?;
                            m.is_sealed = true;
                        }
                        _ => {
                            dup(self, m.is_foreign)?;
                            m.is_foreign = true;
                        }
                    }
                }
                _ => break,
            }
        }
        Ok(m)
    }

    /// `open`, `sealed` and `foreign` are identifiers unless another modifier, a type
    /// keyword, a type or `<` follows them.
    fn modifier_can_stand_here(&self) -> bool {
        match self.tok_at(1) {
            Tok::Ident(_) => true,
            Tok::Punct("@") | Tok::Punct("<") => true,
            _ => false,
        }
    }

    fn annotation(&mut self) -> R<AnnotationUse> {
        let pos = self.pos();
        self.expect_punct("@")?;
        let name = self.qualified()?;
        let mut args = AnnArgs::None;
        if self.eat_punct("(") {
            if self.is_punct(")") {
                return self.err("an annotation with no arguments writes no parentheses");
            }
            if self.is_ident_at(0) && self.is_punct_at(1, "=") {
                let mut named = Vec::new();
                loop {
                    let key = self.ident()?;
                    self.expect_punct("=")?;
                    named.push((key, self.ann_value()?));
                    if !self.eat_punct(",") {
                        break;
                    }
                }
                args = AnnArgs::Named(named);
            } else {
                let mut list = vec![self.ann_value()?];
                while self.eat_punct(",") {
                    list.push(self.ann_value()?);
                }
                args = AnnArgs::Positional(list);
            }
            self.expect_punct(")")?;
        }
        Ok(AnnotationUse { name, args, pos })
    }

    fn ann_value(&mut self) -> R<AnnValue> {
        if self.is_punct("@") {
            return Ok(AnnValue::Ann(self.annotation()?));
        }
        if self.eat_punct("{") {
            let mut list = Vec::new();
            while !self.is_punct("}") {
                list.push(self.ann_value()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct("}")?;
            return Ok(AnnValue::List(list));
        }
        Ok(AnnValue::Expr(self.conditional()?))
    }

    // ---- types ----

    fn type_ref(&mut self) -> R<TypeRef> {
        let pos = self.pos();
        let mut annotations = Vec::new();
        while self.is_punct("@") {
            annotations.push(self.annotation()?);
        }
        let mut t = self.named_type()?;
        t.annotations = annotations;
        t.pos = pos;
        self.dims(&mut t)?;
        Ok(t)
    }

    fn named_type(&mut self) -> R<TypeRef> {
        let pos = self.pos();
        let name = self.qualified()?;
        let args = if self.is_punct("<") { Some(self.type_args()?) } else { None };
        Ok(TypeRef { annotations: Vec::new(), name, args, dims: Vec::new(), pos })
    }

    /// Reads `{ {annotation} "[" "]" }` onto a type.
    fn dims(&mut self, t: &mut TypeRef) -> R<()> {
        loop {
            let save = self.i;
            let mut anns = Vec::new();
            while self.is_punct("@") {
                anns.push(self.annotation()?);
            }
            if self.is_punct("[") && self.is_punct_at(1, "]") {
                self.i += 2;
                t.dims.push(anns);
            } else {
                self.i = save;
                return Ok(());
            }
        }
    }

    fn type_args(&mut self) -> R<Vec<TypeArg>> {
        self.expect_punct("<")?;
        let mut args = Vec::new();
        loop {
            if self.is_punct("?") {
                let pos = self.pos();
                self.i += 1;
                let bound = if self.eat_word("extends") {
                    Some((false, self.type_ref()?))
                } else if self.eat_word("super") {
                    Some((true, self.type_ref()?))
                } else {
                    None
                };
                args.push(TypeArg::Wildcard(bound, pos));
            } else {
                args.push(TypeArg::Type(self.type_ref()?));
            }
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_gt()?;
        Ok(args)
    }

    fn type_list(&mut self) -> R<Vec<TypeRef>> {
        let mut list = vec![self.type_ref()?];
        while self.eat_punct(",") {
            list.push(self.type_ref()?);
        }
        Ok(list)
    }

    fn type_params(&mut self) -> R<Vec<TypeParam>> {
        let mut params = Vec::new();
        if !self.eat_punct("<") {
            return Ok(params);
        }
        loop {
            let pos = self.pos();
            let mut variance = Variance::Invariant;
            // `in` and `out` are keywords directly after `<` or `,`, unless they are
            // themselves the parameter's name.
            if (self.is_word("in") || self.is_word("out"))
                && (self.is_ident_at(1) || self.is_punct_at(1, "@"))
            {
                variance = if self.is_word("in") { Variance::In } else { Variance::Out };
                self.i += 1;
            }
            let mut annotations = Vec::new();
            while self.is_punct("@") {
                annotations.push(self.annotation()?);
            }
            let name = self.ident()?;
            let mut bounds = Vec::new();
            if self.eat_word("extends") {
                bounds.push(self.type_ref()?);
                while self.eat_punct("&") {
                    bounds.push(self.type_ref()?);
                }
            }
            params.push(TypeParam { variance, annotations, name, bounds, pos });
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_gt()?;
        Ok(params)
    }

    // ---- declarations ----

    fn type_decl(&mut self) -> R<TypeDecl> {
        let mods = self.mods()?;
        let pos = self.pos();
        if self.is_word("value") && self.is_word_at(1, "class") {
            self.i += 2;
            return self.class_decl(mods, TypeKind::ValueClass, pos);
        }
        if self.is_word("value") && self.is_word_at(1, "enum") {
            return self.err("`value enum` is rejected: an enum is a reference class");
        }
        if self.eat_word("class") {
            return self.class_decl(mods, TypeKind::Class, pos);
        }
        if self.eat_word("interface") {
            return self.interface_decl(mods, pos);
        }
        if self.eat_word("enum") {
            return self.enum_decl(mods, pos);
        }
        if self.is_word("annotation") && self.is_ident_at(1) {
            self.i += 1;
            return self.annotation_decl(mods, pos);
        }
        self.err(format!(
            "expected a class, an interface, an enum or an annotation, found {}",
            self.describe()
        ))
    }

    fn empty_decl(&self, kind: TypeKind, mods: Mods, name: String, pos: Pos) -> TypeDecl {
        TypeDecl {
            kind,
            mods,
            name,
            type_params: Vec::new(),
            extends: Vec::new(),
            implements: Vec::new(),
            permits: None,
            members: Vec::new(),
            constants: Vec::new(),
            elements: Vec::new(),
            pos,
        }
    }

    fn class_decl(&mut self, mods: Mods, kind: TypeKind, pos: Pos) -> R<TypeDecl> {
        let name = self.ident()?;
        let mut d = self.empty_decl(kind, mods, name, pos);
        d.type_params = self.type_params()?;
        if self.eat_word("extends") {
            d.extends.push(self.type_ref()?);
        }
        if self.eat_word("implements") {
            d.implements = self.type_list()?;
        }
        if self.is_word("permits") {
            self.i += 1;
            d.permits = Some(self.type_list()?);
        }
        self.expect_punct("{")?;
        while !self.is_punct("}") {
            let m = self.member(&d.name, false)?;
            d.members.push(m);
        }
        self.expect_punct("}")?;
        Ok(d)
    }

    fn interface_decl(&mut self, mods: Mods, pos: Pos) -> R<TypeDecl> {
        let name = self.ident()?;
        let mut d = self.empty_decl(TypeKind::Interface, mods, name, pos);
        d.type_params = self.type_params()?;
        if self.eat_word("extends") {
            d.extends = self.type_list()?;
        }
        if self.is_word("permits") {
            self.i += 1;
            d.permits = Some(self.type_list()?);
        }
        self.expect_punct("{")?;
        while !self.is_punct("}") {
            let m = self.member(&d.name, true)?;
            d.members.push(m);
        }
        self.expect_punct("}")?;
        Ok(d)
    }

    fn enum_decl(&mut self, mods: Mods, pos: Pos) -> R<TypeDecl> {
        let name = self.ident()?;
        let mut d = self.empty_decl(TypeKind::Enum, mods, name, pos);
        if self.is_punct("<") {
            return self.err("an enum declares no type parameters");
        }
        if self.eat_word("implements") {
            d.implements = self.type_list()?;
        }
        self.expect_punct("{")?;
        while !self.is_punct("}") && !self.is_punct(";") {
            let cpos = self.pos();
            let mut annotations = Vec::new();
            while self.is_punct("@") {
                annotations.push(self.annotation()?);
            }
            let cname = self.ident()?;
            let mut args = Vec::new();
            if self.eat_punct("(") {
                args = self.args()?;
            }
            if self.is_punct("{") {
                return self.err("an enum constant has no class body");
            }
            d.constants.push(EnumConstant { annotations, name: cname, args, pos: cpos });
            if !self.eat_punct(",") {
                break;
            }
        }
        if self.eat_punct(";") {
            while !self.is_punct("}") {
                let m = self.member(&d.name, false)?;
                d.members.push(m);
            }
        }
        self.expect_punct("}")?;
        Ok(d)
    }

    fn annotation_decl(&mut self, mods: Mods, pos: Pos) -> R<TypeDecl> {
        let name = self.ident()?;
        let mut d = self.empty_decl(TypeKind::Annotation, mods, name, pos);
        if self.is_punct("<") {
            return self.err("an annotation declares no type parameters");
        }
        if self.eat_punct("(") {
            loop {
                let epos = self.pos();
                let ty = self.type_ref()?;
                let ename = self.ident()?;
                let default = if self.eat_punct("=") { Some(self.ann_value()?) } else { None };
                d.elements.push(Element { ty, name: ename, default, pos: epos });
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct(")")?;
        }
        if self.is_punct("{") {
            return self.err("an annotation declaration has no body");
        }
        self.expect_punct(";")?;
        Ok(d)
    }

    fn member(&mut self, class_name: &str, _in_interface: bool) -> R<Member> {
        let mods = self.mods()?;
        let pos = self.pos();
        if mods.is_static && self.is_punct("{") {
            return Ok(Member::StaticInit(self.block()?));
        }
        if self.is_punct("{") {
            return self.err("an instance initializer block is not part of the language");
        }
        if self.is_word("class") || self.is_word("interface") || self.is_word("enum") {
            return self.err("a type is not declared inside another type");
        }
        // Constructors.
        if self.is_word(class_name) && self.is_punct_at(1, "(") {
            self.i += 2;
            let params = self.params_after_paren()?.1;
            let body = if self.eat_punct(";") { None } else { Some(self.block()?) };
            return Ok(Member::Ctor(CtorDecl { mods, params: Some(params), body, pos }));
        }
        if self.is_word(class_name) && self.is_punct_at(1, "{") {
            self.i += 1;
            let body = Some(self.block()?);
            return Ok(Member::Ctor(CtorDecl { mods, params: None, body, pos }));
        }
        let type_params = self.type_params()?;
        let result = if self.eat_word("void") { None } else { Some(self.type_ref()?) };
        let name = self.ident()?;
        if self.eat_punct("(") {
            let (receiver, params) = self.params_after_paren()?;
            let body = if self.eat_punct(";") { None } else { Some(self.block()?) };
            return Ok(Member::Method(MethodDecl {
                mods,
                type_params,
                result,
                name,
                receiver,
                params,
                body,
                pos,
            }));
        }
        if !type_params.is_empty() {
            return self.err("type parameters are written on a method, and this is a field");
        }
        let Some(ty) = result else {
            return self.err("`void` is the result of a method, and this is a field");
        };
        let init = if self.eat_punct("=") { Some(self.expr()?) } else { None };
        self.expect_punct(";")?;
        Ok(Member::Field(FieldDecl { mods, ty, name, init, pos }))
    }

    /// Parses parameters after the opening `(`, through the closing `)`.
    fn params_after_paren(&mut self) -> R<(Option<TypeRef>, Vec<Param>)> {
        let mut receiver = None;
        let mut params = Vec::new();
        let mut first = true;
        while !self.is_punct(")") {
            let pos = self.pos();
            let mods = self.mods()?;
            let mut ty = self.type_ref()?;
            if !mods.annotations.is_empty() {
                // Annotations before a parameter's type are kept on the parameter. The
                // checker moves the qualifiers among them onto the type.
            }
            if first && self.is_word("this") {
                self.i += 1;
                ty.annotations.splice(0..0, mods.annotations.iter().cloned());
                receiver = Some(ty);
            } else {
                let varargs = self.eat_punct("...");
                let name = self.ident()?;
                params.push(Param { mods, ty, varargs, name, pos });
            }
            first = false;
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_punct(")")?;
        Ok((receiver, params))
    }

    // ---- statements ----

    fn block(&mut self) -> R<Block> {
        let pos = self.pos();
        self.expect_punct("{")?;
        let mut stmts = Vec::new();
        while !self.is_punct("}") {
            if matches!(self.tok(), Tok::Eof) {
                return self.err("this block is never closed");
            }
            stmts.push(self.stmt()?);
        }
        let end = self.pos();
        self.expect_punct("}")?;
        Ok(Block { stmts, pos, end })
    }

    /// Tries `modifiers (type | "var") identifier` followed by one of `follow`.
    fn try_local(&mut self, follow: &[&str]) -> Option<Local> {
        self.attempt(|p| {
            let pos = p.pos();
            let mods = p.mods()?;
            let ty = if p.is_word("var") && p.is_ident_at(1) {
                p.i += 1;
                None
            } else {
                Some(p.type_ref()?)
            };
            let name = p.ident()?;
            if !follow.iter().any(|f| p.is_punct(f)) {
                return p.err("not a declaration");
            }
            Ok(Local { mods, ty, name, init: None, pos })
        })
    }

    fn local_rest(&mut self, mut local: Local) -> R<Local> {
        if self.eat_punct("=") {
            local.init = Some(self.expr()?);
        } else if local.ty.is_none() {
            return self.err("`var` needs an initializer");
        }
        Ok(local)
    }

    fn stmt(&mut self) -> R<Stmt> {
        let pos = self.pos();
        let kind = self.stmt_kind()?;
        Ok(Stmt { kind, pos })
    }

    fn stmt_kind(&mut self) -> R<StmtKind> {
        if self.is_punct("{") {
            return Ok(StmtKind::Block(self.block()?));
        }
        if self.eat_punct(";") {
            return Ok(StmtKind::Empty);
        }
        if let Tok::Ident(w) = self.tok().clone() {
            match w.as_str() {
                "if" => {
                    self.i += 1;
                    self.expect_punct("(")?;
                    let c = self.expr()?;
                    self.expect_punct(")")?;
                    let then = Box::new(self.stmt()?);
                    let otherwise =
                        if self.eat_word("else") { Some(Box::new(self.stmt()?)) } else { None };
                    return Ok(StmtKind::If(c, then, otherwise));
                }
                "while" => {
                    self.i += 1;
                    self.expect_punct("(")?;
                    let c = self.expr()?;
                    self.expect_punct(")")?;
                    return Ok(StmtKind::While(c, Box::new(self.stmt()?)));
                }
                "for" => return self.for_stmt(),
                "switch" => {
                    self.i += 1;
                    self.expect_punct("(")?;
                    let selector = self.expr()?;
                    self.expect_punct(")")?;
                    self.expect_punct("{")?;
                    let mut arms = Vec::new();
                    while !self.is_punct("}") {
                        let apos = self.pos();
                        let head = self.arm_head()?;
                        self.expect_punct("->")?;
                        let body = self.stmt()?;
                        arms.push((head, body, apos));
                    }
                    self.expect_punct("}")?;
                    return Ok(StmtKind::Switch { selector, arms });
                }
                "try" => {
                    self.i += 1;
                    let block = self.block()?;
                    let mut catches = Vec::new();
                    while self.is_word("catch") {
                        let cpos = self.pos();
                        self.i += 1;
                        self.expect_punct("(")?;
                        let ty = self.type_ref()?;
                        let name = self.ident()?;
                        self.expect_punct(")")?;
                        let cblock = self.block()?;
                        catches.push(Catch { ty, name, block: cblock, pos: cpos });
                    }
                    let finally = if self.eat_word("finally") { Some(self.block()?) } else { None };
                    if catches.is_empty() && finally.is_none() {
                        return self.err("a `try` has at least one `catch` clause or a `finally` block");
                    }
                    return Ok(StmtKind::Try { block, catches, finally });
                }
                "using" => {
                    self.i += 1;
                    self.expect_punct("(")?;
                    let mut resources = Vec::new();
                    loop {
                        let Some(local) = self.try_local(&["="]) else {
                            return self.err("a `using` resource is written `Type name = expression`");
                        };
                        resources.push(self.local_rest(local)?);
                        if !self.eat_punct(",") {
                            break;
                        }
                    }
                    self.expect_punct(")")?;
                    let block = self.block()?;
                    return Ok(StmtKind::Using { resources, block });
                }
                "return" => {
                    self.i += 1;
                    let e = if self.is_punct(";") { None } else { Some(self.expr()?) };
                    self.expect_punct(";")?;
                    return Ok(StmtKind::Return(e));
                }
                "break" | "continue" => {
                    self.i += 1;
                    let label = if self.is_punct(";") { None } else { Some(self.ident()?) };
                    self.expect_punct(";")?;
                    return Ok(if w == "break" {
                        StmtKind::Break(label)
                    } else {
                        StmtKind::Continue(label)
                    });
                }
                "throw" => {
                    self.i += 1;
                    let e = self.expr()?;
                    self.expect_punct(";")?;
                    return Ok(StmtKind::Throw(e));
                }
                "assert" => {
                    self.i += 1;
                    let c = self.expr()?;
                    let detail = if self.eat_punct(":") { Some(self.expr()?) } else { None };
                    self.expect_punct(";")?;
                    return Ok(StmtKind::Assert(c, detail));
                }
                "super" | "this" if self.is_punct_at(1, "(") => {
                    self.i += 2;
                    let args = self.args()?;
                    self.expect_punct(";")?;
                    return Ok(StmtKind::CtorCall(w == "super", args));
                }
                "else" => return self.err("this `else` has no `if`"),
                "case" | "default" if self.is_punct_at(1, "->") || w == "case" => {
                    return self.err("a switch arm stands directly inside a `switch`");
                }
                _ => {}
            }
            // A label.
            if self.is_ident_at(0) && self.is_punct_at(1, ":") {
                self.i += 2;
                return Ok(StmtKind::Labeled(w, Box::new(self.stmt()?)));
            }
        }
        if let Some(local) = self.try_local(&["=", ";"]) {
            let local = self.local_rest(local)?;
            self.expect_punct(";")?;
            return Ok(StmtKind::Local(local));
        }
        let e = self.expr()?;
        self.expect_punct(";")?;
        Ok(StmtKind::Expr(e))
    }

    fn for_stmt(&mut self) -> R<StmtKind> {
        self.i += 1;
        self.expect_punct("(")?;
        if let Some(local) = self.try_local(&[":"]) {
            self.i += 1;
            let iter = self.expr()?;
            self.expect_punct(")")?;
            let body = Box::new(self.stmt()?);
            return Ok(StmtKind::ForEach { local, iter, body });
        }
        let mut init = Vec::new();
        if !self.is_punct(";") {
            let pos = self.pos();
            if let Some(local) = self.try_local(&["=", ";"]) {
                let local = self.local_rest(local)?;
                init.push(Stmt { kind: StmtKind::Local(local), pos });
            } else {
                loop {
                    let pos = self.pos();
                    let e = self.expr()?;
                    init.push(Stmt { kind: StmtKind::Expr(e), pos });
                    if !self.eat_punct(",") {
                        break;
                    }
                }
            }
        }
        self.expect_punct(";")?;
        let cond = if self.is_punct(";") { None } else { Some(self.expr()?) };
        self.expect_punct(";")?;
        let mut update = Vec::new();
        if !self.is_punct(")") {
            loop {
                update.push(self.expr()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
        }
        self.expect_punct(")")?;
        let body = Box::new(self.stmt()?);
        Ok(StmtKind::For { init, cond, update, body })
    }

    fn arm_head(&mut self) -> R<ArmHead> {
        if self.is_word("default") {
            self.i += 1;
            return Ok(ArmHead::Default);
        }
        self.expect_word("case")?;
        let typed = self.attempt(|p| {
            let ty = p.type_ref()?;
            let name = p.ident()?;
            if !p.is_punct("->") {
                return p.err("not a type arm");
            }
            Ok((ty, name))
        });
        if let Some((ty, name)) = typed {
            return Ok(ArmHead::Type(ty, name));
        }
        let mut list = vec![self.expr()?];
        while self.eat_punct(",") {
            list.push(self.expr()?);
        }
        Ok(ArmHead::Constants(list))
    }

    // ---- expressions ----

    fn args(&mut self) -> R<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.is_punct(")") {
            args.push(self.expr()?);
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_punct(")")?;
        Ok(args)
    }

    pub fn expr(&mut self) -> R<Expr> {
        let pos = self.pos();
        let left = self.conditional()?;
        let (spelling, width) = self.op().unwrap_or(("", 0));
        let op = match spelling {
            "=" => Some(None),
            "+=" => Some(Some(BinOp::Add)),
            "-=" => Some(Some(BinOp::Sub)),
            "*=" => Some(Some(BinOp::Mul)),
            "/=" => Some(Some(BinOp::Div)),
            "%=" => Some(Some(BinOp::Rem)),
            "&=" => Some(Some(BinOp::BitAnd)),
            "|=" => Some(Some(BinOp::BitOr)),
            "^=" => Some(Some(BinOp::BitXor)),
            "<<=" => Some(Some(BinOp::Shl)),
            ">>=" => Some(Some(BinOp::Shr)),
            _ => None,
        };
        let Some(op) = op else { return Ok(left) };
        if !Self::is_location(&left) {
            return self.err("the left side of an assignment is a variable, a field or an indexed element");
        }
        self.i += width;
        let value = self.expr()?;
        Ok(Expr { kind: ExprKind::Assign { op, target: Box::new(left), value: Box::new(value) }, pos })
    }

    fn is_location(e: &Expr) -> bool {
        matches!(e.kind, ExprKind::Name(_) | ExprKind::Field(..) | ExprKind::Index(..))
    }

    fn conditional(&mut self) -> R<Expr> {
        let pos = self.pos();
        let c = self.coalesce()?;
        if !self.eat_punct("?") {
            return Ok(c);
        }
        let a = self.expr()?;
        self.expect_punct(":")?;
        let b = self.conditional()?;
        Ok(Expr { kind: ExprKind::Cond(Box::new(c), Box::new(a), Box::new(b)), pos })
    }

    fn coalesce(&mut self) -> R<Expr> {
        let pos = self.pos();
        let a = self.or_else()?;
        if !self.eat_punct("??") {
            return Ok(a);
        }
        let b = self.coalesce()?;
        Ok(Expr { kind: ExprKind::Coalesce(Box::new(a), Box::new(b)), pos })
    }

    fn or_else(&mut self) -> R<Expr> {
        let pos = self.pos();
        let mut l = self.and_also()?;
        while self.eat_punct("||") {
            let r = self.and_also()?;
            l = Expr { kind: ExprKind::Or(Box::new(l), Box::new(r)), pos };
        }
        Ok(l)
    }

    fn and_also(&mut self) -> R<Expr> {
        let pos = self.pos();
        let mut l = self.binary(0)?;
        while self.eat_punct("&&") {
            let r = self.binary(0)?;
            l = Expr { kind: ExprKind::And(Box::new(l), Box::new(r)), pos };
        }
        Ok(l)
    }

    /// The binary layers from `|` down to `*`, loosest first.
    fn binary(&mut self, level: usize) -> R<Expr> {
        const LEVELS: &[&[(&str, BinOp)]] = &[
            &[("|", BinOp::BitOr)],
            &[("^", BinOp::BitXor)],
            &[("&", BinOp::BitAnd)],
            &[("==", BinOp::Eq), ("!=", BinOp::Ne)],
            &[("<", BinOp::Lt), (">", BinOp::Gt), ("<=", BinOp::Le), (">=", BinOp::Ge)],
            &[("<<", BinOp::Shl), (">>", BinOp::Shr)],
            &[("+", BinOp::Add), ("-", BinOp::Sub)],
            &[("*", BinOp::Mul), ("/", BinOp::Div), ("%", BinOp::Rem)],
        ];
        if level == LEVELS.len() {
            return self.unary();
        }
        let pos = self.pos();
        let mut l = self.binary(level + 1)?;
        loop {
            if level == 4 && self.is_word("instanceof") {
                self.i += 1;
                let ty = self.type_ref()?;
                l = Expr { kind: ExprKind::InstanceOf(Box::new(l), ty), pos };
                continue;
            }
            let Some((spelling, width)) = self.op() else { break };
            if spelling == ">>" && self.is_punct_at(2, ">") && self.glued(2) {
                return self.err("`>>>` is not an operator");
            }
            let found = LEVELS[level].iter().find(|(p, _)| *p == spelling);
            let Some((_, op)) = found else { break };
            self.i += width;
            let r = self.binary(level + 1)?;
            l = Expr { kind: ExprKind::Binary(*op, Box::new(l), Box::new(r)), pos };
        }
        Ok(l)
    }

    fn unary(&mut self) -> R<Expr> {
        let pos = self.pos();
        let op = match self.tok() {
            Tok::Punct("-") => Some(UnOp::Neg),
            Tok::Punct("~") => Some(UnOp::BitNot),
            Tok::Punct("!") => Some(UnOp::Not),
            _ => None,
        };
        if let Some(op) = op {
            self.i += 1;
            let e = self.unary()?;
            return Ok(Expr { kind: ExprKind::Unary(op, Box::new(e)), pos });
        }
        if self.is_punct("+") {
            return self.err("there is no unary `+`");
        }
        if self.is_punct("++") || self.is_punct("--") {
            let inc = self.is_punct("++");
            self.i += 1;
            let e = self.unary()?;
            if !Self::is_location(&e) {
                return self.err_at(pos, "`++` and `--` apply to a variable, a field or an indexed element");
            }
            return Ok(Expr { kind: ExprKind::IncDec { inc, prefix: true, target: Box::new(e) }, pos });
        }
        if self.is_punct("(") {
            if let Some(ty) = self.attempt(|p| p.cast_head()) {
                let e = self.unary()?;
                return Ok(Expr { kind: ExprKind::Cast(ty, Box::new(e)), pos });
            }
        }
        self.postfix()
    }

    /// `"(" type ")"`, accepted only when what follows can begin a cast operand.
    fn cast_head(&mut self) -> R<TypeRef> {
        self.expect_punct("(")?;
        let ty = self.type_ref()?;
        self.expect_punct(")")?;
        let ok = match self.tok() {
            Tok::Ident(w) => !matches!(w.as_str(), "instanceof"),
            Tok::Int(..) | Tok::Dec(_) | Tok::Char(_) | Tok::Str(_) => true,
            Tok::Punct("(") | Tok::Punct("!") | Tok::Punct("~") => true,
            _ => false,
        };
        if ok {
            Ok(ty)
        } else {
            self.err("not a cast")
        }
    }

    fn postfix(&mut self) -> R<Expr> {
        let pos = self.pos();
        let mut e = self.primary()?;
        loop {
            if self.is_punct(".") {
                if self.is_word_at(1, "class") {
                    let ty = self.expr_to_type(&e)?;
                    self.i += 2;
                    e = Expr { kind: ExprKind::ClassLit(ty), pos };
                    continue;
                }
                self.i += 1;
                let type_args = if self.is_punct("<") { self.type_args()? } else { Vec::new() };
                let name = self.ident()?;
                if self.eat_punct("(") {
                    let args = self.args()?;
                    e = Expr {
                        kind: ExprKind::Call { target: Some(Box::new(e)), type_args, name, args },
                        pos,
                    };
                } else {
                    if !type_args.is_empty() {
                        return self.err("type arguments are written before the name of a method that is called");
                    }
                    e = Expr { kind: ExprKind::Field(Box::new(e), name), pos };
                }
            } else if self.is_punct("[") {
                self.i += 1;
                let index = self.expr()?;
                self.expect_punct("]")?;
                e = Expr { kind: ExprKind::Index(Box::new(e), Box::new(index)), pos };
            } else if self.is_punct("::") {
                self.i += 1;
                let name = if self.eat_word("new") { "new".to_string() } else { self.ident()? };
                e = Expr { kind: ExprKind::MethodRef { target: RefTarget::Expr(Box::new(e)), name }, pos };
            } else if self.is_punct("++") || self.is_punct("--") {
                if !Self::is_location(&e) {
                    return self.err("`++` and `--` apply to a variable, a field or an indexed element");
                }
                let inc = self.is_punct("++");
                self.i += 1;
                e = Expr { kind: ExprKind::IncDec { inc, prefix: false, target: Box::new(e) }, pos };
            } else {
                break;
            }
        }
        Ok(e)
    }

    /// Reads a name chain as the type it would be in `T.class`.
    fn expr_to_type(&self, e: &Expr) -> R<TypeRef> {
        fn path(e: &Expr, out: &mut Vec<String>) -> bool {
            match &e.kind {
                ExprKind::Name(n) => {
                    out.push(n.clone());
                    true
                }
                ExprKind::Field(t, n) => {
                    if !path(t, out) {
                        return false;
                    }
                    out.push(n.clone());
                    true
                }
                _ => false,
            }
        }
        let mut name = Vec::new();
        if !path(e, &mut name) {
            return self.err("a class literal is written `Type.class`");
        }
        Ok(TypeRef { annotations: Vec::new(), name, args: None, dims: Vec::new(), pos: e.pos })
    }

    /// A type written with arguments or brackets where an expression could stand:
    /// `List<String>.class`, `List<String>::new`, `Int[].class`.
    fn type_led(&mut self) -> R<Expr> {
        let pos = self.pos();
        let ty = self.type_ref()?;
        if ty.args.is_none() && ty.dims.is_empty() {
            return self.err("not a type");
        }
        if self.is_punct(".") && self.is_word_at(1, "class") {
            self.i += 2;
            return Ok(Expr { kind: ExprKind::ClassLit(ty), pos });
        }
        if self.eat_punct("::") {
            let name = if self.eat_word("new") { "new".to_string() } else { self.ident()? };
            return Ok(Expr { kind: ExprKind::MethodRef { target: RefTarget::Type(ty), name }, pos });
        }
        self.err("not a type")
    }

    fn primary(&mut self) -> R<Expr> {
        let pos = self.pos();
        let kind = match self.tok().clone() {
            Tok::Int(d, r) => {
                self.i += 1;
                ExprKind::Int(d, r)
            }
            Tok::Dec(d) => {
                self.i += 1;
                ExprKind::Dec(d)
            }
            Tok::Char(c) => {
                self.i += 1;
                ExprKind::Char(c)
            }
            Tok::Str(s) => {
                self.i += 1;
                ExprKind::Str(s)
            }
            Tok::Punct("(") => {
                if self.lambda_ahead() {
                    return self.lambda();
                }
                self.i += 1;
                let e = self.expr()?;
                self.expect_punct(")")?;
                ExprKind::Paren(Box::new(e))
            }
            Tok::Ident(w) => match w.as_str() {
                "true" => {
                    self.i += 1;
                    ExprKind::Bool(true)
                }
                "false" => {
                    self.i += 1;
                    ExprKind::Bool(false)
                }
                "null" => {
                    self.i += 1;
                    ExprKind::Null
                }
                "this" => {
                    self.i += 1;
                    ExprKind::This
                }
                "super" => {
                    self.i += 1;
                    self.expect_punct(".")?;
                    let name = self.ident()?;
                    self.expect_punct("(")?;
                    let args = self.args()?;
                    ExprKind::SuperCall { name, args }
                }
                "new" => {
                    self.i += 1;
                    return self.new_expr(pos);
                }
                "switch" => {
                    self.i += 1;
                    return self.switch_expr(pos);
                }
                _ => {
                    // A name followed by `<` or `[]` may begin a type.
                    if self.is_ident_at(0)
                        && (self.is_punct_at(1, "<")
                            || (self.is_punct_at(1, "[") && self.is_punct_at(2, "]"))
                            || self.is_punct_at(1, "."))
                    {
                        if let Some(e) = self.attempt(|p| p.type_led()) {
                            return Ok(e);
                        }
                    }
                    let name = self.ident()?;
                    if self.eat_punct("(") {
                        let args = self.args()?;
                        ExprKind::Call { target: None, type_args: Vec::new(), name, args }
                    } else {
                        ExprKind::Name(name)
                    }
                }
            },
            _ => return self.err(format!("expected an expression, found {}", self.describe())),
        };
        Ok(Expr { kind, pos })
    }

    /// Whether the `(` here opens the parameter list of a lambda.
    fn lambda_ahead(&self) -> bool {
        let mut depth = 0usize;
        let mut n = 0usize;
        loop {
            match self.tok_at(n) {
                Tok::Punct("(") => depth += 1,
                Tok::Punct(")") => {
                    depth -= 1;
                    if depth == 0 {
                        return self.is_punct_at(n + 1, "->");
                    }
                }
                Tok::Eof => return false,
                _ => {}
            }
            n += 1;
        }
    }

    fn lambda(&mut self) -> R<Expr> {
        let pos = self.pos();
        self.expect_punct("(")?;
        let mut params = Vec::new();
        while !self.is_punct(")") {
            let ppos = self.pos();
            let simple = self.is_ident_at(0) && (self.is_punct_at(1, ",") || self.is_punct_at(1, ")"));
            if simple {
                let name = self.ident()?;
                params.push(LambdaParam { ty: None, name, pos: ppos });
            } else {
                let mods = self.mods()?;
                let mut ty = self.type_ref()?;
                ty.annotations.splice(0..0, mods.annotations.into_iter());
                let name = self.ident()?;
                params.push(LambdaParam { ty: Some(ty), name, pos: ppos });
            }
            if !self.eat_punct(",") {
                break;
            }
        }
        self.expect_punct(")")?;
        self.expect_punct("->")?;
        let body = if self.is_punct("{") {
            LambdaBody::Block(self.block()?)
        } else {
            LambdaBody::Expr(Box::new(self.expr()?))
        };
        Ok(Expr { kind: ExprKind::Lambda { params, body }, pos })
    }

    fn new_expr(&mut self, pos: Pos) -> R<Expr> {
        let mut annotations = Vec::new();
        while self.is_punct("@") {
            annotations.push(self.annotation()?);
        }
        let mut ty = self.named_type()?;
        ty.annotations = annotations;
        if self.eat_punct("(") {
            if !ty.annotations.is_empty() {
                return self.err_at(pos, "a qualifier is not written on the class in `new`");
            }
            let args = self.args()?;
            return Ok(Expr { kind: ExprKind::New { ty, args }, pos });
        }
        // `new T[] ... { elements }`: the brackets belong to the type.
        let save = self.i;
        let mut probe = ty.clone();
        self.dims(&mut probe)?;
        if !probe.dims.is_empty() && self.is_punct("{") {
            self.i += 1;
            let mut elems = Vec::new();
            while !self.is_punct("}") {
                elems.push(self.expr()?);
                if !self.eat_punct(",") {
                    break;
                }
            }
            self.expect_punct("}")?;
            return Ok(Expr { kind: ExprKind::ArrayLit { ty: probe, elems }, pos });
        }
        self.i = save;
        // `new T[n]` and further brackets, which belong to the element type.
        self.expect_punct("[")?;
        let len = self.expr()?;
        self.expect_punct("]")?;
        self.dims(&mut ty)?;
        Ok(Expr { kind: ExprKind::NewArray { elem: ty, len: Box::new(len) }, pos })
    }

    fn switch_expr(&mut self, pos: Pos) -> R<Expr> {
        self.expect_punct("(")?;
        let selector = self.expr()?;
        self.expect_punct(")")?;
        self.expect_punct("{")?;
        let mut arms = Vec::new();
        while !self.is_punct("}") {
            let apos = self.pos();
            let head = self.arm_head()?;
            self.expect_punct("->")?;
            let value = if self.eat_word("throw") {
                ArmValue::Throw(self.expr()?)
            } else {
                ArmValue::Expr(self.expr()?)
            };
            self.expect_punct(";")?;
            arms.push((head, value, apos));
        }
        self.expect_punct("}")?;
        Ok(Expr { kind: ExprKind::Switch { selector: Box::new(selector), arms }, pos })
    }
}
