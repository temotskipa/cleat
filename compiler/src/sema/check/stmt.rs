//! Statements (section 4.9, chapter 9) with the flow rules of chapter 12, and the
//! bodies of a class: methods, constructors and static initializers.

use super::call::{ArgIn, CallSite};
use super::expr::needs_expected;
use super::*;
use crate::ast::{ArmHead, ArmValue, ExprKind, StmtKind};
use crate::sema::decl;

// ---- the names a piece of code assigns ----

fn expr_assigns(e: &ast::Expr, out: &mut Vec<String>) {
    use ExprKind::*;
    let target_name = |t: &ast::Expr, out: &mut Vec<String>| {
        let mut t = t;
        while let Paren(x) = &t.kind {
            t = x;
        }
        if let Name(n) = &t.kind {
            out.push(n.clone());
        }
    };
    match &e.kind {
        Assign { target, value, .. } => {
            target_name(target, out);
            expr_assigns(target, out);
            expr_assigns(value, out);
        }
        IncDec { target, .. } => {
            target_name(target, out);
            expr_assigns(target, out);
        }
        Paren(x) | Field(x, _) | Unary(_, x) | Cast(_, x) | InstanceOf(x, _) | NewArray { len: x, .. } => expr_assigns(x, out),
        Call { target, args, .. } => {
            if let Some(t) = target {
                expr_assigns(t, out);
            }
            args.iter().for_each(|a| expr_assigns(a, out));
        }
        SuperCall { args, .. } | New { args, .. } => args.iter().for_each(|a| expr_assigns(a, out)),
        ArrayLit { elems, .. } => elems.iter().for_each(|a| expr_assigns(a, out)),
        Index(a, b) | Binary(_, a, b) | And(a, b) | Or(a, b) | Coalesce(a, b) => {
            expr_assigns(a, out);
            expr_assigns(b, out);
        }
        Cond(a, b, c) => {
            expr_assigns(a, out);
            expr_assigns(b, out);
            expr_assigns(c, out);
        }
        Switch { selector, arms } => {
            expr_assigns(selector, out);
            for (_, v, _) in arms {
                match v {
                    ArmValue::Expr(x) | ArmValue::Throw(x) => expr_assigns(x, out),
                }
            }
        }
        MethodRef { target: ast::RefTarget::Expr(x), .. } => expr_assigns(x, out),
        // A lambda cannot assign a local of the enclosing method.
        _ => {}
    }
}

fn block_assigns(b: &ast::Block, out: &mut Vec<String>) {
    b.stmts.iter().for_each(|s| stmt_assigns(s, out));
}

fn stmt_assigns(s: &ast::Stmt, out: &mut Vec<String>) {
    use StmtKind::*;
    match &s.kind {
        Block(b) => block_assigns(b, out),
        Local(l) => {
            if let Some(e) = &l.init {
                expr_assigns(e, out);
            }
        }
        Expr(e) | Throw(e) => expr_assigns(e, out),
        Empty | Break(_) | Continue(_) => {}
        If(c, a, b) => {
            expr_assigns(c, out);
            stmt_assigns(a, out);
            if let Some(b) = b {
                stmt_assigns(b, out);
            }
        }
        While(c, b) => {
            expr_assigns(c, out);
            stmt_assigns(b, out);
        }
        For { init, cond, update, body } => {
            init.iter().for_each(|s| stmt_assigns(s, out));
            if let Some(c) = cond {
                expr_assigns(c, out);
            }
            update.iter().for_each(|e| expr_assigns(e, out));
            stmt_assigns(body, out);
        }
        ForEach { iter, body, .. } => {
            expr_assigns(iter, out);
            stmt_assigns(body, out);
        }
        Switch { selector, arms } => {
            expr_assigns(selector, out);
            arms.iter().for_each(|(_, s, _)| stmt_assigns(s, out));
        }
        Try { block, catches, finally } => {
            block_assigns(block, out);
            catches.iter().for_each(|c| block_assigns(&c.block, out));
            if let Some(f) = finally {
                block_assigns(f, out);
            }
        }
        Using { resources, block } => {
            for r in resources {
                if let Some(e) = &r.init {
                    expr_assigns(e, out);
                }
            }
            block_assigns(block, out);
        }
        Return(e) => {
            if let Some(e) = e {
                expr_assigns(e, out);
            }
        }
        Assert(c, d) => {
            expr_assigns(c, out);
            if let Some(d) = d {
                expr_assigns(d, out);
            }
        }
        Labeled(_, s) => stmt_assigns(s, out),
        CtorCall(_, args) => args.iter().for_each(|a| expr_assigns(a, out)),
    }
}

impl<'p> Checker<'p> {
    pub fn block_stmts(&mut self, b: &ast::Block) -> Vec<TStmt> {
        self.push_scope();
        let mut out = Vec::new();
        for s in &b.stmts {
            self.stmt(s, &mut out);
        }
        self.pop_scope();
        out
    }

    fn scoped(&mut self, s: &ast::Stmt) -> Vec<TStmt> {
        self.push_scope();
        let mut out = Vec::new();
        self.stmt(s, &mut out);
        self.pop_scope();
        out
    }

    /// Before a loop: a fact about a variable the loop assigns does not hold in it, and
    /// such a variable may already have a value when the body assigns it.
    fn enter_loop(&mut self, names: &[String]) {
        let at = self.frames.len() - 1;
        for n in names {
            if let Some((fi, id)) = self.find_local(n) {
                if fi == at {
                    let f = self.frame();
                    f.flow.facts.remove(&id);
                    f.flow.maybe.insert(id);
                }
            }
        }
    }

    fn kill_facts(&mut self, names: &[String]) {
        let at = self.frames.len() - 1;
        for n in names {
            if let Some((fi, id)) = self.find_local(n) {
                if fi == at {
                    self.frame().flow.facts.remove(&id);
                }
            }
        }
    }

    /// An expression used as a statement (section 4.9).
    pub fn expr_stmt(&mut self, e: &ast::Expr) -> TExpr {
        let unit = self.unit_ty();
        match &e.kind {
            ExprKind::Assign { op, target, value } => self.assign(*op, target, value, e.pos, false),
            ExprKind::IncDec { inc, prefix, target } => self.inc_dec(*inc, *prefix, target, e.pos, false),
            ExprKind::Call { .. } | ExprKind::SuperCall { .. } => {
                let v = self.check_expr(e, None);
                if v.ty.is_error() {
                    return v;
                }
                let discardable = match &v.kind {
                    TKind::Call(c) => self.p.method(c.method).discardable,
                    TKind::StaticReq { method, .. } => self.p.method(*method).discardable,
                    _ => false,
                };
                if v.ty != unit && !discardable {
                    self.err(e.pos, "the result of this call must be used: only a call whose result is `Unit`, or whose method is `@Discardable`, stands alone as a statement");
                }
                v
            }
            _ => {
                if !needs_expected(e) {
                    self.check_expr(e, None);
                }
                self.error_expr(e.pos, "this expression is not a statement: its value would be discarded")
            }
        }
    }

    fn find_label(&mut self, name: &Option<String>, want_loop: bool, pos: Pos, word: &str) -> Option<usize> {
        let f = self.frame_ref();
        let found = match name {
            Some(n) => f.labels.iter().rposition(|l| l.name.as_deref() == Some(n.as_str())),
            None => f.labels.iter().rposition(|l| if want_loop { l.is_loop } else { l.breakable }),
        };
        let Some(i) = found else {
            match name {
                Some(n) => self.err(pos, format!("no statement labeled `{n}` encloses this `{word}`")),
                None if want_loop => self.err(pos, format!("`{word}` is not inside a loop")),
                None => self.err(pos, format!("`{word}` is not inside a loop or a switch statement")),
            }
            return None;
        };
        if want_loop && !self.frame_ref().labels[i].is_loop {
            self.err(pos, format!("`{word}` with a label continues a loop, and this label is not on a loop"));
            return None;
        }
        if let Some(mark) = self.frame_ref().in_finally.last() {
            if i < *mark {
                self.err(pos, format!("a `{word}` does not leave a `finally` block"));
                return None;
            }
        }
        Some(i)
    }

    fn push_label(&mut self, name: Option<String>, is_loop: bool, breakable: bool, pos: Pos) -> u32 {
        if let Some(n) = &name {
            if self.frame_ref().labels.iter().any(|l| l.name.as_deref() == Some(n.as_str())) {
                self.err(pos, format!("the label `{n}` is already on an enclosing statement"));
            }
        }
        let id = self.new_label();
        self.frame().labels.push(Label { name, id, is_loop, breakable, break_flows: Vec::new(), continue_flows: Vec::new() });
        id
    }

    fn local_decl(&mut self, l: &ast::Local, value: Option<TExpr>, fixed: Option<Type>, is_final: bool) -> (LocalId, Option<TExpr>) {
        // `value` and `fixed` serve the forms that bind a name without an initializer of
        // their own: the variable of a `for` over a collection, for one.
        let m = &l.mods;
        if m.audience.is_some() || m.is_static || m.is_open || m.is_abstract || m.is_sealed || m.is_foreign {
            self.err(l.pos, "a local accepts `final` and qualifiers only");
        }
        let (ty, init) = match (&l.ty, &l.init, value) {
            (Some(tr), init, value) => {
                let cx = self.cx.clone();
                let (ty, rest) = decl::resolve_type_anns(self.p, &cx, tr, &m.annotations, true);
                if let Some(a) = rest.first() {
                    self.err(a.pos, "a declaration annotation is not written on a local");
                }
                let v = match (init, value) {
                    (_, Some(v)) => Some(self.coerce(v, &ty, l.pos)),
                    (Some(e), None) => {
                        let v = self.check_expr(e, Some(&ty));
                        Some(self.coerce(v, &ty, e.pos))
                    }
                    (None, None) => None,
                };
                (ty, v)
            }
            (None, _, Some(v)) => (fixed.unwrap_or_else(|| v.ty.clone()), Some(v)),
            (None, Some(e), None) => {
                let v = self.check_expr(e, None);
                if v.ty.is_null_literal() {
                    self.err(e.pos, "`var` takes the type of its initializer, and `null` alone gives no type; write the type");
                    (Type::error(), None)
                } else {
                    let mut ty = self.project(&v.ty);
                    let mut dummy = ty.clone();
                    let cx = self.cx.clone();
                    decl::apply_annotations(self.p, &cx, &mut dummy, &m.annotations, false);
                    if dummy != ty {
                        let v2 = self.coerce(v, &dummy, e.pos);
                        ty = dummy;
                        (ty, Some(v2))
                    } else {
                        (ty, Some(v))
                    }
                }
            }
            (None, None, None) => {
                self.err(l.pos, "`var` needs an initializer");
                (Type::error(), None)
            }
        };
        let id = self.declare(&l.name, ty, m.is_final || is_final, l.pos, false);
        if init.is_some() || l.init.is_some() {
            let f = self.frame();
            f.flow.assigned.insert(id);
            f.flow.maybe.insert(id);
        }
        (id, init)
    }

    pub fn stmt(&mut self, s: &ast::Stmt, out: &mut Vec<TStmt>) {
        let pos = s.pos;
        if !self.frame_ref().flow.reachable && !matches!(s.kind, StmtKind::Empty) {
            self.err(pos, "this statement is unreachable");
            self.frame().flow.reachable = true;
        }
        let boolean = self.boolean();
        match &s.kind {
            StmtKind::Empty => {}
            StmtKind::Block(b) => {
                let stmts = self.block_stmts(b);
                out.push(TStmt::Block(stmts));
            }
            StmtKind::Local(l) => {
                let (id, init) = self.local_decl(l, None, None, false);
                out.push(TStmt::Local(id, init));
            }
            StmtKind::Expr(e) => {
                let v = self.expr_stmt(e);
                out.push(TStmt::Expr(v));
            }
            StmtKind::If(c, a, b) => {
                let cond = self.check_cond(c);
                let ce = self.coerce(cond.e, &boolean, c.pos);
                let base = self.frame().flow.clone();
                self.apply_facts(&cond.t);
                let then = self.scoped(a);
                let after_then = std::mem::replace(&mut self.frame().flow, base);
                self.apply_facts(&cond.f);
                let els = match b {
                    Some(b) => self.scoped(b),
                    None => Vec::new(),
                };
                let after_else = std::mem::take(&mut self.frame().flow);
                self.frame().flow = Flow::join(after_then, after_else);
                out.push(TStmt::If(ce, then, els));
            }
            StmtKind::While(c, body) => {
                let name = self.pending_label.take();
                let mut names = Vec::new();
                expr_assigns(c, &mut names);
                stmt_assigns(body, &mut names);
                self.enter_loop(&names);
                let cond = self.check_cond(c);
                let ce = self.coerce(cond.e, &boolean, c.pos);
                let constant = match ce.kind {
                    TKind::Bool(b) => Some(b),
                    _ => None,
                };
                if constant == Some(false) {
                    self.err(c.pos, "a loop whose condition is the constant `false` is rejected: its body could never run");
                }
                let mut exit = self.frame().flow.clone();
                self.apply_facts(&cond.t);
                let label = self.push_label(name, true, true, pos);
                let stmts = self.scoped(body);
                let lab = self.frame().labels.pop().unwrap();
                let end = std::mem::take(&mut self.frame().flow);
                for (id, ty) in &cond.f {
                    exit.facts.insert(*id, ty.clone());
                }
                exit.reachable = constant != Some(true);
                exit.maybe.extend(end.maybe);
                exit.maybe_fields.extend(end.maybe_fields);
                for bf in lab.break_flows {
                    exit = Flow::join(exit, bf);
                }
                self.frame().flow = exit;
                out.push(TStmt::Loop { label, cond: Some(ce), update: Vec::new(), body: stmts });
            }
            StmtKind::For { init, cond, update, body } => {
                let name = self.pending_label.take();
                self.push_scope();
                let mut block = Vec::new();
                for s in init {
                    self.stmt(s, &mut block);
                }
                let mut names = Vec::new();
                if let Some(c) = cond {
                    expr_assigns(c, &mut names);
                }
                update.iter().for_each(|e| expr_assigns(e, &mut names));
                stmt_assigns(body, &mut names);
                self.enter_loop(&names);
                let (ce, facts_t, facts_f, constant) = match cond {
                    Some(c) => {
                        let k = self.check_cond(c);
                        let ce = self.coerce(k.e, &boolean, c.pos);
                        let constant = match ce.kind {
                            TKind::Bool(b) => Some(b),
                            _ => None,
                        };
                        if constant == Some(false) {
                            self.err(c.pos, "a loop whose condition is the constant `false` is rejected: its body could never run");
                        }
                        (Some(ce), k.t, k.f, constant)
                    }
                    None => (None, Vec::new(), Vec::new(), Some(true)),
                };
                let mut exit = self.frame().flow.clone();
                self.apply_facts(&facts_t);
                let label = self.push_label(name, true, true, pos);
                let stmts = self.scoped(body);
                let lab = self.frame().labels.pop().unwrap();
                // The update runs after the body and after a `continue`.
                let mut at_update = std::mem::take(&mut self.frame().flow);
                for cf in lab.continue_flows {
                    at_update = Flow::join(at_update, cf);
                }
                at_update.reachable = true;
                self.frame().flow = at_update;
                let mut ups = Vec::new();
                for u in update {
                    ups.push(self.expr_stmt(u));
                }
                let end = std::mem::take(&mut self.frame().flow);
                for (id, ty) in &facts_f {
                    exit.facts.insert(*id, ty.clone());
                }
                exit.reachable = constant != Some(true);
                exit.maybe.extend(end.maybe);
                exit.maybe_fields.extend(end.maybe_fields);
                for bf in lab.break_flows {
                    exit = Flow::join(exit, bf);
                }
                self.frame().flow = exit;
                self.pop_scope();
                block.push(TStmt::Loop { label, cond: ce, update: ups, body: stmts });
                out.push(TStmt::Block(block));
            }
            StmtKind::ForEach { local, iter, body } => {
                let name = self.pending_label.take();
                let it = self.check_expr(iter, None);
                let iterable = self.p.wk.iterable;
                let cap = self.capture(&it.ty);
                let elem = if it.ty.is_error() {
                    Type::error()
                } else if self.p.may_be_null(&it.ty) {
                    let s = self.show(&it.ty);
                    self.err(iter.pos, format!("a `for` statement visits an `Iterable`, and a `{s}` may be `null`"));
                    Type::error()
                } else {
                    match self.p.supertype_at(&cap, iterable) {
                        Some(up) => {
                            let e = up.args()[0].as_type().cloned().unwrap_or_else(Type::error);
                            self.project(&e)
                        }
                        None => {
                            let s = self.show(&it.ty);
                            self.err(iter.pos, format!("a `for` statement visits an `Iterable`, and `{s}` is not one"));
                            Type::error()
                        }
                    }
                };
                self.push_scope();
                let mut block = Vec::new();
                let (cond, next) = if elem.is_error() {
                    (TExpr { kind: TKind::Bool(true), ty: Type::error() }, TExpr { kind: TKind::Null, ty: Type::error() })
                } else {
                    let iterator = self.call_method(it, "iterator", CallSite::new(Vec::new(), iter.pos));
                    let tmp = self.temp(iterator.ty.clone());
                    let read = TExpr { kind: TKind::Local(tmp), ty: iterator.ty.clone() };
                    block.push(TStmt::Local(tmp, Some(iterator)));
                    let cond = self.call_method(read.clone(), "hasNext", CallSite::new(Vec::new(), iter.pos));
                    let next = self.call_method(read, "next", CallSite::new(Vec::new(), iter.pos));
                    (cond, next)
                };
                let mut names = Vec::new();
                stmt_assigns(body, &mut names);
                self.enter_loop(&names);
                let mut exit = self.frame().flow.clone();
                let label = self.push_label(name, true, true, pos);
                self.push_scope();
                let (id, init) = self.local_decl(local, Some(next), Some(elem), true);
                let mut stmts = vec![TStmt::Local(id, init)];
                self.stmt(body, &mut stmts);
                self.pop_scope();
                let lab = self.frame().labels.pop().unwrap();
                let end = std::mem::take(&mut self.frame().flow);
                exit.maybe.extend(end.maybe);
                exit.maybe_fields.extend(end.maybe_fields);
                for bf in lab.break_flows {
                    exit = Flow::join(exit, bf);
                }
                exit.reachable = true;
                self.frame().flow = exit;
                self.pop_scope();
                block.push(TStmt::Loop { label, cond: Some(cond), update: Vec::new(), body: stmts });
                out.push(TStmt::Block(block));
            }
            StmtKind::Switch { selector, arms } => self.switch_stmt(selector, arms, pos, out),
            StmtKind::Try { block, catches, finally } => self.try_stmt(block, catches, finally, pos, out),
            StmtKind::Using { resources, block } => {
                self.push_scope();
                let stmts = self.using(resources, block);
                self.pop_scope();
                out.extend(stmts);
            }
            StmtKind::Return(e) => {
                self.return_stmt(e, pos, out);
                self.frame().flow.reachable = false;
            }
            StmtKind::Break(name) => {
                if let Some(i) = self.find_label(name, false, pos, "break") {
                    let flow = self.frame().flow.clone();
                    let l = &mut self.frame().labels[i];
                    l.break_flows.push(flow);
                    let id = l.id;
                    out.push(TStmt::Break(id));
                }
                self.frame().flow.reachable = false;
            }
            StmtKind::Continue(name) => {
                if let Some(i) = self.find_label(name, true, pos, "continue") {
                    let flow = self.frame().flow.clone();
                    let l = &mut self.frame().labels[i];
                    l.continue_flows.push(flow);
                    let id = l.id;
                    out.push(TStmt::Continue(id));
                }
                self.frame().flow.reachable = false;
            }
            StmtKind::Throw(e) => {
                let v = self.check_expr(e, None);
                let throwable = Type::simple(self.p.wk.throwable);
                if !v.ty.is_error() && !self.p.is_subtype(&v.ty, &throwable) {
                    let s = self.show(&v.ty);
                    self.err(e.pos, format!("`throw` raises a `Throwable`, and this is a `{s}`"));
                }
                out.push(TStmt::Throw(v));
                self.frame().flow.reachable = false;
            }
            StmtKind::Assert(c, detail) => {
                let cond = self.check_cond(c);
                let ce = self.coerce(cond.e, &boolean, c.pos);
                let base = self.frame().flow.clone();
                let mut args = Vec::new();
                if let Some(d) = detail {
                    let v = self.check_expr(d, None);
                    if !v.ty.is_error() {
                        args.push(ArgIn::Done(self.call_method(v, "toString", CallSite::new(Vec::new(), d.pos))));
                    }
                }
                let ex = Type::simple(self.p.wk.assertion_exception);
                let raise = self.construct(ex, CallSite::new(args, pos));
                self.frame().flow = base;
                self.apply_facts(&cond.t);
                let not = TExpr { kind: TKind::Not(Box::new(ce)), ty: boolean };
                out.push(TStmt::If(not, vec![TStmt::Throw(raise)], Vec::new()));
            }
            StmtKind::Labeled(name, inner) => {
                if matches!(inner.kind, StmtKind::While(..) | StmtKind::For { .. } | StmtKind::ForEach { .. }) {
                    if self.frame_ref().labels.iter().any(|l| l.name.as_deref() == Some(name.as_str())) {
                        self.err(pos, format!("the label `{name}` is already on an enclosing statement"));
                    }
                    self.pending_label = Some(name.clone());
                    self.stmt(inner, out);
                    self.pending_label = None;
                } else {
                    let label = self.push_label(Some(name.clone()), false, false, pos);
                    let body = self.scoped(inner);
                    let lab = self.frame().labels.pop().unwrap();
                    let mut flow = std::mem::take(&mut self.frame().flow);
                    for bf in lab.break_flows {
                        flow = Flow::join(flow, bf);
                    }
                    self.frame().flow = flow;
                    out.push(TStmt::Labeled { label, body });
                }
            }
            StmtKind::CtorCall(is_super, args) => {
                let word = if *is_super { "super" } else { "this" };
                self.err(pos, format!("`{word}(...)` is written as a statement of a constructor's body itself, not inside another statement"));
                for a in args {
                    if !needs_expected(a) {
                        self.check_expr(a, None);
                    }
                }
            }
        }
    }

    fn return_stmt(&mut self, e: &Option<ast::Expr>, pos: Pos, out: &mut Vec<TStmt>) {
        let unit = self.unit_ty();
        if !self.frame_ref().in_finally.is_empty() {
            self.err(pos, "`return` is rejected in a `finally` block");
        }
        let f = self.frame_ref();
        if f.in_static_init {
            self.err(pos, "`return` is not written in an initializer");
            return;
        }
        if f.is_ctor {
            if e.is_some() {
                self.err(pos, "`return` with a value is rejected in a constructor");
            } else if !(f.ctor == CtorPhase::After && f.explicit_ctor_call) && !f.value_ctor {
                self.err(pos, "in a constructor, `return;` is legal only after the `super(...)` or `this(...)` call that the body writes");
            }
            out.push(TStmt::Return(None));
            return;
        }
        if f.inferring {
            let v = e.as_ref().map(|e| self.check_expr(e, None));
            let ty = v.as_ref().map(|v| v.ty.clone()).unwrap_or(unit);
            self.frame().returned.push(ty);
            out.push(TStmt::Return(v));
            return;
        }
        let ret = f.ret.clone().unwrap_or_else(Type::error);
        match e {
            None => {
                if ret != unit && !ret.is_error() {
                    let s = self.show(&ret);
                    self.err(pos, format!("this `return` needs a value of `{s}`"));
                }
                out.push(TStmt::Return(None));
            }
            Some(e) => {
                if ret == unit {
                    self.err(pos, "`return` with an operand is rejected in a method whose result is `Unit`");
                    if !needs_expected(e) {
                        self.check_expr(e, None);
                    }
                    out.push(TStmt::Return(None));
                    return;
                }
                let v = self.check_expr(e, Some(&ret));
                let v = self.coerce(v, &ret, e.pos);
                out.push(TStmt::Return(Some(v)));
            }
        }
    }

    // ---- switch ----

    fn covers(&mut self, class: ClassId, arms: &[Type], depth: u32) -> bool {
        for a in arms {
            if let Some(ac) = a.class_id() {
                if self.p.is_subclass(class, ac) {
                    return true;
                }
            }
        }
        let c = self.p.class(class);
        if depth < 16 && c.is_sealed && (c.is_interface() || c.is_abstract) {
            if let Some(list) = c.permits.clone() {
                return !list.is_empty() && list.iter().all(|p| self.covers(*p, arms, depth + 1));
            }
        }
        false
    }

    /// Checks the heads of a switch. Returns the selector, its temporary, the test of
    /// each arm and whether the switch is exhaustive. `each` checks one arm's body with
    /// the arm's name in scope.
    fn switch_heads<B>(
        &mut self,
        selector: &ast::Expr,
        heads: &[(&ArmHead, Pos)],
        mut each: impl FnMut(&mut Self, usize) -> B,
    ) -> (TExpr, LocalId, Vec<(ArmTest, B)>, bool, Vec<Flow>) {
        let sel = self.check_expr(selector, None);
        let sty = sel.ty.clone();
        if !sty.is_error() && self.p.may_be_null(&sty) {
            let s = self.show(&sty);
            self.err(selector.pos, format!("the selector of a switch is not `@Nullable`, and this is a `{s}`"));
        }
        let tmp = self.temp(sty.clone());
        let base = self.frame().flow.clone();
        let has_const = heads.iter().any(|(h, _)| matches!(h, ArmHead::Constants(_)));
        let has_type = heads.iter().any(|(h, _)| matches!(h, ArmHead::Type(..)));
        if has_const && has_type {
            self.err(selector.pos, "one switch uses constant arms or type arms, not both");
        }
        if has_const && !sty.is_error() {
            let ok = match sty.class_id() {
                Some(id) => {
                    let w = &self.p.wk;
                    matches!(self.p.num_kind(id), Some(NumKind::Signed(_) | NumKind::Unsigned(_)))
                        || id == w.char_
                        || id == w.boolean
                        || id == w.string
                        || self.p.class(id).is_enum()
                }
                None => false,
            };
            if !ok {
                let s = self.show(&sty);
                self.err(selector.pos, format!("constant arms need a selector that is an integer class, `Char`, `Boolean`, `String` or an enum, and this is a `{s}`"));
            }
        }
        let mut seen: Vec<Const> = Vec::new();
        let mut types: Vec<Type> = Vec::new();
        let mut has_default = false;
        let mut arms = Vec::new();
        let mut flows = Vec::new();
        for (i, (head, hpos)) in heads.iter().enumerate() {
            self.frame().flow = base.clone();
            self.push_scope();
            if has_default {
                self.err(*hpos, "`default` is the last arm of a switch");
            }
            let test = match head {
                ArmHead::Constants(cs) => {
                    let mut vals = Vec::new();
                    for c in cs {
                        let v = self.check_expr(c, Some(&sty));
                        let v = self.coerce(v, &sty, c.pos);
                        if v.ty.is_error() {
                            continue;
                        }
                        match self.as_const(&v) {
                            Some(k) => {
                                if seen.contains(&k) {
                                    self.err(c.pos, "this constant is written twice in the switch");
                                }
                                seen.push(k);
                                vals.push(v);
                            }
                            None => self.err(c.pos, "the constant of a `case` is a constant expression"),
                        }
                    }
                    ArmTest::Constants(vals)
                }
                ArmHead::Type(tr, name) => {
                    let ty = self.resolve(tr);
                    if ty.nullable || !ty.quals.is_empty() {
                        self.err(tr.pos, "a type arm tests the class of the selector, and names no qualifier");
                    }
                    let ty = ty.bare();
                    if !ty.is_error() && !sty.is_error() {
                        if self.disjoint(&sty, &ty) {
                            let (a, b) = (self.show(&sty), self.show(&ty));
                            self.err(tr.pos, format!("`{a}` and `{b}` are disjoint, so this arm could never match"));
                        }
                        let mut dead = false;
                        for prev in &types {
                            if self.p.is_subtype(&ty, prev) {
                                dead = true;
                            }
                        }
                        if dead {
                            self.err(tr.pos, "this arm cannot match: an earlier arm names its type or a supertype of it");
                        }
                        types.push(ty.clone());
                    }
                    let id = self.declare(name, ty.clone(), true, *hpos, false);
                    let f = self.frame();
                    f.flow.assigned.insert(id);
                    f.flow.maybe.insert(id);
                    ArmTest::Type(ty, id)
                }
                ArmHead::Default => {
                    has_default = true;
                    ArmTest::Default
                }
            };
            let body = each(self, i);
            self.pop_scope();
            flows.push(std::mem::take(&mut self.frame().flow));
            arms.push((test, body));
        }
        let exhaustive = has_default
            || sty.is_error()
            || match sty.class_id() {
                Some(id) if has_type => self.covers(id, &types, 0),
                Some(id) if id == self.p.wk.boolean => seen.contains(&Const::Bool(true)) && seen.contains(&Const::Bool(false)),
                Some(id) if self.p.class(id).is_enum() => {
                    let c = self.p.class(id);
                    let all: Vec<FieldRef> = c
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| f.enum_ordinal.is_some())
                        .map(|(i, _)| FieldRef { class: id, index: i as u32 })
                        .collect();
                    all.iter().all(|f| seen.contains(&Const::Enum(*f)))
                }
                _ => false,
            };
        self.frame().flow = base;
        (sel, tmp, arms, exhaustive, flows)
    }

    fn switch_stmt(&mut self, selector: &ast::Expr, arms: &[(ArmHead, ast::Stmt, Pos)], pos: Pos, out: &mut Vec<TStmt>) {
        let label = self.push_label(None, false, true, pos);
        let heads: Vec<(&ArmHead, Pos)> = arms.iter().map(|(h, _, p)| (h, *p)).collect();
        let (sel, temp, tarms, exhaustive, flows) = self.switch_heads(selector, &heads, |ck, i| {
            let mut body = Vec::new();
            ck.stmt(&arms[i].1, &mut body);
            body
        });
        let lab = self.frame().labels.pop().unwrap();
        let mut flow = std::mem::take(&mut self.frame().flow);
        // With no arm matching, the switch completes normally.
        flow.reachable = !exhaustive;
        for f in flows {
            flow = Flow::join(flow, f);
        }
        for bf in lab.break_flows {
            flow = Flow::join(flow, bf);
        }
        self.frame().flow = flow;
        out.push(TStmt::Switch { label, selector: sel, temp, arms: tarms });
    }

    pub fn switch_expr(&mut self, selector: &ast::Expr, arms: &[(ArmHead, ArmValue, Pos)], expected: Option<&Type>, pos: Pos) -> TExpr {
        let heads: Vec<(&ArmHead, Pos)> = arms.iter().map(|(h, _, p)| (h, *p)).collect();
        let (sel, temp, mut tarms, exhaustive, flows) = self.switch_heads(selector, &heads, |ck, i| match &arms[i].1 {
            ArmValue::Expr(e) => ck.check_expr(e, expected),
            ArmValue::Throw(e) => {
                let v = ck.check_expr(e, None);
                let throwable = Type::simple(ck.p.wk.throwable);
                if !v.ty.is_error() && !ck.p.is_subtype(&v.ty, &throwable) {
                    let s = ck.show(&v.ty);
                    ck.err(e.pos, format!("`throw` raises a `Throwable`, and this is a `{s}`"));
                }
                ck.frame().flow.reachable = false;
                TExpr { kind: TKind::Throw(Box::new(v)), ty: Type::error() }
            }
        });
        if !exhaustive {
            self.err(pos, "a switch expression is exhaustive: add the missing arms or a `default` arm");
        }
        let mut flow = Flow { reachable: false, ..Flow::default() };
        for f in flows {
            flow = Flow::join(flow, f);
        }
        if !flow.reachable {
            // Every arm raises. The expression still stands where a value is expected.
            flow.reachable = true;
        }
        self.frame().flow = flow;
        // The type: the expected type, or the type of one arm that the others fit.
        let is_throw = |e: &TExpr| matches!(e.kind, TKind::Throw(_));
        let mut failed = false;
        let ty = match expected {
            Some(t) if !t.is_error() => t.clone(),
            _ => {
                let cands: Vec<Type> = tarms.iter().filter(|(_, e)| !is_throw(e) && !e.ty.is_error()).map(|(_, e)| e.ty.clone()).collect();
                failed = tarms.iter().any(|(_, e)| !is_throw(e) && e.ty.is_error());
                let mut pick = None;
                for c in &cands {
                    if c.is_null_literal() {
                        continue;
                    }
                    let mut all = true;
                    for o in &cands {
                        if !self.assignable(o, c, true) && !(o.is_null_literal()) {
                            all = false;
                            break;
                        }
                    }
                    if all {
                        let nullable = cands.iter().any(|o| o.is_null_literal());
                        pick = Some(if nullable { c.clone().nullable() } else { c.clone() });
                        break;
                    }
                }
                match pick {
                    Some(t) => t,
                    None => {
                        if !failed && !cands.is_empty() {
                            self.err(pos, "the arms of this switch have types none of which the others are assignable to; write the type where the switch is used");
                        }
                        Type::error()
                    }
                }
            }
        };
        let _ = failed;
        for (i, (_, e)) in tarms.iter_mut().enumerate() {
            let v = std::mem::replace(e, TExpr { kind: TKind::Null, ty: Type::error() });
            *e = if is_throw(&v) {
                TExpr { kind: v.kind, ty: ty.clone() }
            } else {
                let apos = match &arms[i].1 {
                    ArmValue::Expr(x) | ArmValue::Throw(x) => x.pos,
                };
                self.coerce(v, &ty, apos)
            };
        }
        TExpr { kind: TKind::Switch { selector: Box::new(sel), temp, arms: tarms }, ty }
    }

    // ---- try and using ----

    fn try_stmt(&mut self, block: &ast::Block, catches: &[ast::Catch], finally: &Option<ast::Block>, pos: Pos, out: &mut Vec<TStmt>) {
        if catches.is_empty() && finally.is_none() {
            self.err(pos, "a `try` statement has at least one `catch` clause or a `finally` block");
        }
        let base = self.frame().flow.clone();
        let mut assigned = Vec::new();
        block_assigns(block, &mut assigned);
        let body = self.block_stmts(block);
        let after_try = std::mem::take(&mut self.frame().flow);
        let throwable = Type::simple(self.p.wk.throwable);
        // A handler starts with what was assigned before the statement.
        let mut handler_start = base.clone();
        handler_start.maybe.extend(after_try.maybe.iter().copied());
        handler_start.maybe_fields.extend(after_try.maybe_fields.iter().copied());
        let mut after = after_try;
        let mut tcatches = Vec::new();
        let mut earlier: Vec<Type> = Vec::new();
        for c in catches {
            self.frame().flow = handler_start.clone();
            self.kill_facts(&assigned);
            let ty = self.resolve(&c.ty);
            if !ty.is_error() {
                if ty.nullable || !self.p.is_subtype(&ty, &throwable) {
                    let s = self.show(&ty);
                    self.err(c.ty.pos, format!("a `catch` clause names `Throwable` or a subclass of it, and this is `{s}`"));
                }
                let mut dead = false;
                for e in &earlier {
                    if self.p.is_subtype(&ty, e) {
                        dead = true;
                    }
                }
                if dead {
                    self.err(c.pos, "this `catch` clause can never run: an earlier clause names its type or a supertype of it");
                }
                earlier.push(ty.clone());
            }
            self.push_scope();
            let local = self.declare(&c.name, ty.clone(), false, c.pos, false);
            let f = self.frame();
            f.flow.assigned.insert(local);
            f.flow.maybe.insert(local);
            let stmts = self.block_stmts(&c.block);
            self.pop_scope();
            let end = std::mem::take(&mut self.frame().flow);
            after = Flow::join(after, end);
            tcatches.push(TCatch { ty, local, body: stmts });
        }
        let tfinally = match finally {
            Some(fb) => {
                let mut start = handler_start;
                start.maybe.extend(after.maybe.iter().copied());
                self.frame().flow = start;
                self.kill_facts(&assigned);
                let mut in_catches = Vec::new();
                catches.iter().for_each(|c| block_assigns(&c.block, &mut in_catches));
                self.kill_facts(&in_catches);
                let mark = self.frame_ref().labels.len();
                self.frame().in_finally.push(mark);
                let stmts = self.block_stmts(fb);
                self.frame().in_finally.pop();
                let end = std::mem::take(&mut self.frame().flow);
                // What the finally block assigns is assigned after the statement.
                let mut in_finally = Vec::new();
                block_assigns(fb, &mut in_finally);
                after.assigned.extend(end.assigned.iter().copied());
                after.fields.extend(end.fields.iter().copied());
                after.maybe.extend(end.maybe.iter().copied());
                after.maybe_fields.extend(end.maybe_fields.iter().copied());
                after.reachable = after.reachable && end.reachable;
                self.frame().flow = after;
                self.kill_facts(&in_finally);
                Some(stmts)
            }
            None => {
                self.frame().flow = after;
                None
            }
        };
        out.push(TStmt::Try { body, catches: tcatches, finally: tfinally });
    }

    fn using(&mut self, resources: &[ast::Local], block: &ast::Block) -> Vec<TStmt> {
        let Some(r) = resources.first() else {
            return self.block_stmts(block);
        };
        let unit = self.unit_ty();
        if r.init.is_none() {
            self.err(r.pos, "a `using` binding has an initializer");
        }
        let (local, init) = self.local_decl(r, None, None, true);
        let ty = self.frame_ref().locals[local as usize].ty.clone();
        let mut close = TExpr { kind: TKind::Null, ty: Type::error() };
        if !ty.is_error() {
            if self.p.may_be_null(&ty) {
                let s = self.show(&ty);
                self.err(r.pos, format!("a `using` binding is not `@Nullable`, and this is a `{s}`"));
            } else {
                let read = TExpr { kind: TKind::Local(local), ty: ty.clone() };
                close = self.call_method(read, "close", CallSite::new(Vec::new(), r.pos));
                if !close.ty.is_error() && close.ty != unit {
                    self.err(r.pos, "`using` calls `close()`, whose result is `Unit`");
                }
            }
        }
        let body = self.using(&resources[1..], block);
        match init {
            Some(init) => vec![TStmt::Using { local, init, close, body }],
            None => body,
        }
    }
}

// ---- the bodies of a class ----

fn machine_type(p: &Program, t: &Type, depth: u32) -> bool {
    if t.nullable || !t.quals.is_empty() {
        return false;
    }
    let Ty::Class(id, args) = &t.ty else { return false };
    let w = &p.wk;
    if !args.is_empty() {
        return false;
    }
    match p.num_kind(*id) {
        Some(NumKind::Rational) => false,
        Some(_) => true,
        None if *id == w.boolean || *id == w.char_ || *id == w.pointer => true,
        None => {
            let c = p.class(*id);
            depth < 8
                && c.kind == ast::TypeKind::ValueClass
                && c.package != "cleat"
                && c.fields.iter().filter(|f| !f.is_static).all(|f| machine_type(p, &f.ty, depth + 1))
        }
    }
}

fn check_foreign(p: &mut Program, id: ClassId, mi: usize) {
    let m = p.class(id).methods[mi].clone();
    let unit = p.class(id).unit;
    let unit_ty = Type::simple(p.wk.unit);
    for prm in &m.params {
        let ok = machine_type(p, &prm.ty, 0)
            || p.array_elem(&prm.ty).map(|e| !prm.ty.nullable && machine_type(p, &e, 0)).unwrap_or(false);
        if !ok {
            let s = p.show(&prm.ty);
            p.error(unit, prm.pos, format!("a parameter of a `foreign` method is a machine type or an array of one, and `{s}` is neither"));
        }
    }
    if m.ret != unit_ty && !machine_type(p, &m.ret, 0) {
        let s = p.show(&m.ret);
        p.error(unit, m.pos, format!("the result of a `foreign` method is a machine type or `void`, and `{s}` is neither"));
    }
    if !m.tparams.is_empty() {
        p.error(unit, m.pos, "a `foreign` method has no type parameters");
    }
}

fn own_type(p: &Program, id: ClassId) -> Type {
    let n = p.class(id).tparams.len();
    Type::class(id, (0..n).map(|i| Arg::Ty(Type::var(Tv { owner: TvOwner::Class(id), index: i as u32 }))).collect())
}

pub fn check_class(p: &mut Program, id: ClassId) {
    if p.class(id).lambda.is_some() {
        return;
    }
    let unit_ty = Type::simple(p.wk.unit);
    // Methods.
    for mi in 0..p.class(id).methods.len() {
        let m = p.class(id).methods[mi].clone();
        if m.foreign {
            check_foreign(p, id, mi);
        }
        let Some(body) = &m.body else { continue };
        let mref = MethodRef { class: id, index: mi as u32 };
        let mut ck = Checker::new(p, id, m.is_static);
        ck.method = Some(mref);
        for (i, tp) in m.tparams.iter().enumerate() {
            ck.cx.tvars.push((tp.name.clone(), Tv { owner: TvOwner::Method(mref), index: i as u32 }));
        }
        let this_ty = if m.is_static {
            None
        } else {
            let mut t = own_type(ck.p, id);
            t.nullable = m.recv_nullable;
            t.quals = m.recv_quals.clone();
            Some(t)
        };
        ck.push_frame(Some(m.ret.clone()), this_ty);
        for prm in &m.params {
            ck.declare(&prm.name, prm.ty.clone(), prm.is_final, prm.pos, true);
        }
        let stmts = ck.block_stmts(body);
        if ck.frame_ref().flow.reachable && m.ret != unit_ty && !m.ret.is_error() {
            ck.err(body.end, format!("`{}` can reach the end of its body, and every path must end in a `return` with a value or in a `throw`", m.name));
        }
        let checked = ck.pop_frame(stmts);
        p.classes[id as usize].methods[mi].checked = Some(checked);
    }
    // Constructors.
    for ci in 0..p.class(id).ctors.len() {
        check_ctor(p, id, ci);
    }
    check_static_init(p, id);
}

fn check_ctor(p: &mut Program, id: ClassId, ci: usize) {
    let c = p.class(id).clone();
    let k = c.ctors[ci].clone();
    if k.intrinsic || c.is_interface() {
        return;
    }
    let mut ck = Checker::new(p, id, false);
    let this_ty = own_type(ck.p, id);
    ck.push_frame(None, Some(this_ty.clone()));
    ck.frame().is_ctor = true;
    let this = TExpr { kind: TKind::This, ty: this_ty };
    let mut stmts: Vec<TStmt> = Vec::new();

    if c.is_value() {
        // The parameters are the fields, and the body runs once they are bound.
        ck.frame().value_ctor = true;
        let mut fi = 0u32;
        for (i, f) in c.fields.iter().enumerate() {
            if f.is_static {
                continue;
            }
            let prm = &k.params[fi as usize];
            let local = ck.declare(&prm.name, prm.ty.clone(), true, prm.pos, true);
            let read = TExpr { kind: TKind::Local(local), ty: prm.ty.clone() };
            let fref = FieldRef { class: id, index: i as u32 };
            stmts.push(TStmt::Expr(TExpr { kind: TKind::AssignField(Box::new(this.clone()), fref, Box::new(read)), ty: f.ty.clone() }));
            fi += 1;
        }
        ck.frame().ctor = CtorPhase::After;
        if let Some(body) = &k.body {
            stmts.extend(ck.block_stmts(body));
        }
        let checked = ck.pop_frame(stmts);
        p.classes[id as usize].ctors[ci].checked = Some(checked);
        return;
    }

    ck.frame().ctor = CtorPhase::Before;
    let hidden: Vec<TExpr> = if c.is_enum() {
        let name = ck.declare("#name", ck.string(), true, k.pos, true);
        let ordinal = ck.declare("#ordinal", ck.int(), true, k.pos, true);
        vec![TExpr { kind: TKind::Local(name), ty: ck.string() }, TExpr { kind: TKind::Local(ordinal), ty: ck.int() }]
    } else {
        Vec::new()
    };
    for prm in &k.params {
        ck.declare(&prm.name, prm.ty.clone(), prm.is_final, prm.pos, true);
    }
    let empty = ast::Block { stmts: Vec::new(), pos: k.pos, end: k.pos };
    let body = k.body.as_ref().unwrap_or(&empty);
    let calls_this = matches!(body.stmts.first().map(|s| &s.kind), Some(StmtKind::CtorCall(false, _)));
    if !calls_this {
        // Field initializers run first, in source order.
        for (i, f) in c.fields.iter().enumerate() {
            if f.is_static {
                continue;
            }
            let Some(init) = &f.init else { continue };
            let v = ck.check_expr(init, Some(&f.ty));
            let v = ck.coerce(v, &f.ty, init.pos);
            let fref = FieldRef { class: id, index: i as u32 };
            stmts.push(TStmt::Expr(TExpr { kind: TKind::AssignField(Box::new(this.clone()), fref, Box::new(v)), ty: f.ty.clone() }));
            let fl = &mut ck.frame().flow;
            fl.fields.insert(i as u32);
            fl.maybe_fields.insert(i as u32);
        }
    }
    let fields_assigned = |ck: &mut Checker, pos: Pos| {
        if !ck.frame_ref().flow.reachable {
            return;
        }
        for (i, f) in c.fields.iter().enumerate() {
            if f.is_static || f.ty.nullable || f.ty.is_error() {
                continue;
            }
            if !ck.frame_ref().flow.fields.contains(&(i as u32)) {
                ck.err(pos, format!("the field `{}` is not assigned on every path to the `super(...)` call", f.name));
            }
        }
    };
    ck.push_scope();
    let mut seen_call = false;
    for (si, s) in body.stmts.iter().enumerate() {
        let StmtKind::CtorCall(is_super, args) = &s.kind else {
            ck.stmt(s, &mut stmts);
            continue;
        };
        if !ck.frame_ref().flow.reachable {
            ck.err(s.pos, "this statement is unreachable");
            ck.frame().flow.reachable = true;
        }
        if seen_call {
            ck.err(s.pos, "a constructor body contains at most one `super(...)` or `this(...)` call");
            continue;
        }
        seen_call = true;
        if !*is_super && si != 0 {
            ck.err(s.pos, "`this(...)`, when written, is the first statement of the constructor");
        }
        if *is_super && c.is_enum() {
            ck.err(s.pos, "an enum constructor does not write `super(...)`");
            continue;
        }
        if *is_super {
            fields_assigned(&mut ck, s.pos);
        }
        let site = CallSite::new(args.iter().map(ArgIn::Ast).collect(), s.pos);
        if let Some((ctor, checked)) = ck.ctor_call(*is_super, site) {
            let mut all = if *is_super { Vec::new() } else { hidden.clone() };
            all.extend(checked);
            stmts.push(TStmt::CtorCall { ctor, args: all });
        }
        let f = ck.frame();
        f.ctor = CtorPhase::After;
        f.explicit_ctor_call = true;
    }
    ck.pop_scope();
    if !seen_call {
        // The body ends with an implicit `super()`.
        fields_assigned(&mut ck, body.end);
        let args: Vec<ArgIn> = hidden.iter().cloned().map(ArgIn::Done).collect();
        let was = ck.frame_ref().flow.reachable;
        if c.superclass.is_some() {
            if let Some((ctor, checked)) = ck.ctor_call(true, CallSite::new(args, k.pos)) {
                if was {
                    stmts.push(TStmt::CtorCall { ctor, args: checked });
                }
            }
        }
    }
    let checked = ck.pop_frame(stmts);
    p.classes[id as usize].ctors[ci].checked = Some(checked);
}

fn check_static_init(p: &mut Program, id: ClassId) {
    let c = p.class(id).clone();
    let mut ck = Checker::new(p, id, true);
    ck.push_frame(None, None);
    ck.frame().in_static_init = true;
    let mut stmts: Vec<TStmt> = Vec::new();
    let own = Type::simple(id);
    // An enum creates its constants first.
    for (i, f) in c.fields.iter().enumerate() {
        let Some(ordinal) = f.enum_ordinal else { continue };
        let site = CallSite::new(f.enum_args.iter().map(ArgIn::Ast).collect(), f.pos);
        if let Some((ctor, args)) = ck.ctor_call(false, site) {
            let mut all = vec![
                TExpr { kind: TKind::Str(f.name.clone()), ty: ck.string() },
                TExpr { kind: TKind::Int(ordinal as i128), ty: ck.int() },
            ];
            all.extend(args);
            let new = TExpr { kind: TKind::New { ctor, args: all }, ty: own.clone() };
            let fref = FieldRef { class: id, index: i as u32 };
            stmts.push(TStmt::Expr(TExpr { kind: TKind::AssignStatic(fref, Box::new(new)), ty: own.clone() }));
        }
        let fl = &mut ck.frame().flow;
        fl.fields.insert(i as u32);
        fl.maybe_fields.insert(i as u32);
    }
    // Then the field initializers and the initializer blocks, in source order.
    enum Item<'a> {
        Field(usize),
        Block(&'a ast::Block),
    }
    let mut items: Vec<(Pos, Item)> = Vec::new();
    for (i, f) in c.fields.iter().enumerate() {
        if f.is_static && f.enum_ordinal.is_none() && f.init.is_some() {
            items.push((f.pos, Item::Field(i)));
        }
    }
    for b in &c.static_inits {
        items.push((b.pos, Item::Block(b)));
    }
    items.sort_by_key(|(pos, _)| *pos);
    for (_, item) in items {
        match item {
            Item::Field(i) => {
                let f = &c.fields[i];
                let init = f.init.as_ref().unwrap();
                let v = ck.check_expr(init, Some(&f.ty));
                let v = ck.coerce(v, &f.ty, init.pos);
                let fref = FieldRef { class: id, index: i as u32 };
                stmts.push(TStmt::Expr(TExpr { kind: TKind::AssignStatic(fref, Box::new(v)), ty: f.ty.clone() }));
                let fl = &mut ck.frame().flow;
                fl.fields.insert(i as u32);
                fl.maybe_fields.insert(i as u32);
            }
            Item::Block(b) => {
                let body = ck.block_stmts(b);
                stmts.push(TStmt::Block(body));
            }
        }
    }
    if ck.frame_ref().flow.reachable {
        for (i, f) in c.fields.iter().enumerate() {
            if !f.is_static || f.ty.nullable || f.ty.is_error() {
                continue;
            }
            if !ck.frame_ref().flow.fields.contains(&(i as u32)) {
                ck.err(f.pos, format!("the static field `{}` is not assigned by its initializer or by a static initializer block", f.name));
            }
        }
    }
    let checked = ck.pop_frame(stmts);
    p.classes[id as usize].static_init_checked = Some(checked);
}
