//! Lambdas and method references (section 4.8).

use super::expr::Target;
use super::*;
use crate::ast::{ExprKind, LambdaBody, RefTarget};

fn unparen(e: &ast::Expr) -> &ast::Expr {
    match &e.kind {
        ExprKind::Paren(x) => unparen(x),
        _ => e,
    }
}

impl<'p> Checker<'p> {
    /// The value class that one lambda expression declares.
    fn new_lambda_class(&mut self, target: Option<&Type>, pos: Pos) -> ClassId {
        let id = self.p.classes.len() as ClassId;
        self.next_lambda += 1;
        let host = self.p.class(self.class);
        let name = format!("{}$lambda{}", host.name, id);
        let qname = format!("{}$lambda{}", host.qname, id);
        let (unit, package) = (host.unit, host.package.clone());
        let env: Vec<Tv> = self.cx.tvars.iter().map(|(_, tv)| *tv).collect();
        let object = self.p.object();
        self.p.classes.push(Class {
            id,
            unit,
            name,
            package,
            qname,
            kind: ast::TypeKind::ValueClass,
            aud: Aud::Private,
            is_open: false,
            is_sealed: false,
            is_abstract: false,
            tparams: Vec::new(),
            superclass: Some(object),
            interfaces: target.into_iter().cloned().collect(),
            permits: None,
            fields: Vec::new(),
            methods: Vec::new(),
            ctors: Vec::new(),
            static_inits: Vec::new(),
            static_init_checked: None,
            elements: Vec::new(),
            qualifier: Qualifier::No,
            refines_above: Vec::new(),
            targets: None,
            inherited: false,
            ann_uses: Vec::new(),
            anns: Vec::new(),
            deprecated: None,
            lambda: Some(LambdaClass { host: self.class, env, captures_this: None }),
            pos,
            decl_index: usize::MAX,
        });
        id
    }

    /// Checks the body of a lambda as the method of `class`. With no result type given,
    /// the result type is worked out from the body and returned.
    fn lambda_body(
        &mut self,
        class: ClassId,
        params: &[ast::LambdaParam],
        ptys: &[Type],
        body: &LambdaBody,
        ret: Option<&Type>,
    ) -> (Body, Vec<Capture>, Type) {
        let unit = self.unit_ty();
        let this_ty = Type::simple(class);
        self.push_frame(ret.cloned(), Some(this_ty.clone()));
        self.frame().lambda = Some(LambdaFrame { class, captures: Vec::new() });
        self.frame().inferring = ret.is_none();
        for (lp, t) in params.iter().zip(ptys.iter()) {
            self.declare(&lp.name, t.clone(), false, lp.pos, true);
        }
        let mut stmts = Vec::new();
        let mut result = unit.clone();
        match body {
            LambdaBody::Expr(x) => {
                let statement_form = matches!(
                    unparen(x).kind,
                    ExprKind::Assign { .. } | ExprKind::IncDec { .. } | ExprKind::Call { .. } | ExprKind::SuperCall { .. }
                );
                match ret {
                    Some(r) if *r == unit && statement_form => {
                        // The value is discarded, as a statement's would be.
                        let e = self.expr_stmt(unparen(x));
                        stmts.push(TStmt::Expr(e));
                        stmts.push(TStmt::Return(None));
                    }
                    Some(r) => {
                        let v = self.check_expr(x, Some(r));
                        let v = self.coerce(v, r, x.pos);
                        stmts.push(TStmt::Return(Some(v)));
                    }
                    None => {
                        let v = self.check_expr(x, None);
                        result = v.ty.clone();
                        stmts.push(TStmt::Return(Some(v)));
                    }
                }
            }
            LambdaBody::Block(b) => {
                stmts = self.block_stmts(b);
                let falls_off = self.frame().flow.reachable;
                match ret {
                    Some(r) if *r != unit && falls_off && !r.is_error() => {
                        self.err(b.end, "this lambda can reach the end of its body, and every path must end in a `return` with a value or in a `throw`");
                    }
                    Some(_) => {}
                    None => {
                        let returned = self.frame().returned.clone();
                        result = match returned.first() {
                            None => unit.clone(),
                            Some(first) => {
                                let mut pick = first.clone();
                                for cand in &returned {
                                    let mut all = true;
                                    for other in &returned {
                                        if !self.p.is_subtype(other, cand) {
                                            all = false;
                                            break;
                                        }
                                    }
                                    if all {
                                        pick = cand.clone();
                                        break;
                                    }
                                }
                                pick
                            }
                        };
                    }
                }
            }
        }
        let captures = self.frame().lambda.as_ref().unwrap().captures.clone();
        // The body starts by reading what the instance holds.
        let mut all = Vec::new();
        for c in &captures {
            let this = TExpr { kind: TKind::This, ty: this_ty.clone() };
            let read = TExpr { kind: TKind::Field(Box::new(this), FieldRef { class, index: c.field }), ty: c.ty.clone() };
            all.push(TStmt::Local(c.local, Some(read)));
        }
        all.extend(stmts);
        let body = self.pop_frame(all);
        (body, captures, result)
    }

    /// The type a lambda's body gives its result, when its parameters have these types.
    /// Nothing the check does is kept.
    pub fn lambda_result(&mut self, e: &ast::Expr, ptys: &[Type]) -> Option<Type> {
        let ExprKind::Lambda { params, body } = &unparen(e).kind else { return None };
        let snap = self.snapshot();
        let class = self.new_lambda_class(None, e.pos);
        let (_, _, result) = self.lambda_body(class, params, ptys, body, None);
        let failed = self.failed_since(&snap);
        self.restore(snap);
        if failed || result.is_error() || result.is_null_literal() {
            None
        } else {
            Some(result)
        }
    }

    /// A lambda where the functional interface `target` is expected.
    pub fn lambda(&mut self, e: &ast::Expr, target: &Type) -> TExpr {
        let pos = e.pos;
        let ExprKind::Lambda { params, body } = &unparen(e).kind else { unreachable!() };
        let iface = target.bare();
        let Some((fm, fs)) = self.functional_method(&iface) else {
            let s = self.show(target);
            return self.error_expr(pos, format!("a lambda stands where a functional interface is expected, and `{s}` is not one"));
        };
        let fmd = self.p.method(fm);
        let (name, discardable) = (fmd.name.clone(), fmd.discardable);
        let fparams: Vec<Type> = fmd.params.iter().map(|p| fs.apply(&p.ty)).collect();
        let ret = fs.apply(&fmd.ret);
        let mut overrides = fmd.overrides.clone();
        overrides.push(fm);
        if !fmd.tparams.is_empty() {
            return self.error_expr(pos, "a lambda does not implement a generic method");
        }
        if fparams.len() != params.len() {
            return self.error_expr(pos, format!("the lambda has {} parameters, and `{name}` of the interface has {}", params.len(), fparams.len()));
        }
        for (lp, ft) in params.iter().zip(fparams.iter()) {
            if let Some(w) = &lp.ty {
                let wt = self.resolve(w);
                if !wt.is_error() && !self.p.same_type(&wt, ft) {
                    let (a, b) = (self.show(&wt), self.show(ft));
                    self.err(lp.pos, format!("the parameter is written `{a}`, and the interface's method takes `{b}`"));
                }
            }
        }
        let class = self.new_lambda_class(Some(&iface), pos);
        let (checked, captures, _) = self.lambda_body(class, params, &fparams, body, Some(&ret));
        let mparams: Vec<Param> = params
            .iter()
            .zip(fparams.iter())
            .map(|(lp, t)| Param { name: lp.name.clone(), ty: t.clone(), varargs: false, is_final: false, ann_uses: Vec::new(), anns: Vec::new(), pos: lp.pos })
            .collect();
        self.p.classes[class as usize].methods.push(Method {
            name,
            tparams: Vec::new(),
            params: mparams,
            ret,
            recv_nullable: false,
            recv_quals: Vec::new(),
            is_static: false,
            is_open: false,
            is_abstract: false,
            is_final_written: true,
            foreign: false,
            intrinsic: false,
            discardable,
            has_override: true,
            implicit: false,
            narrows: None,
            symbol: None,
            deprecated: None,
            aud: Aud::Public,
            only: None,
            body: None,
            ann_uses: Vec::new(),
            anns: Vec::new(),
            overrides,
            static_requirement: false,
            pos,
            checked: Some(checked),
        });
        let mut values = Vec::new();
        for c in &captures {
            if c.source == CapSource::This {
                if let Some(l) = self.p.classes[class as usize].lambda.as_mut() {
                    l.captures_this = Some(c.ty.clone());
                }
            }
            values.push(self.capture_value(c));
        }
        TExpr { kind: TKind::Lambda(Box::new(Lambda { class, captures: values })), ty: iface }
    }

    /// A method reference, checked as the lambda it is short for.
    pub fn method_ref(&mut self, e: &ast::Expr, expected: Option<&Type>) -> TExpr {
        let pos = e.pos;
        let ExprKind::MethodRef { target, name } = &unparen(e).kind else { unreachable!() };
        let Some(t) = expected.filter(|t| !t.is_error()) else {
            return self.error_expr(pos, "a method reference has no type of its own; it stands where a functional interface is expected");
        };
        let Some((fm, _)) = self.functional_method(&t.bare()) else {
            let s = self.show(t);
            return self.error_expr(pos, format!("a method reference stands where a functional interface is expected, and `{s}` is not one"));
        };
        let n = self.p.method(fm).params.len();
        let names: Vec<String> = (0..n).map(|i| format!("$a{i}")).collect();
        let arg = |i: usize| ast::Expr { kind: ExprKind::Name(names[i].clone()), pos };
        let lam = |body: ast::Expr| ast::Expr {
            kind: ExprKind::Lambda {
                params: names.iter().map(|n| ast::LambdaParam { ty: None, name: n.clone(), pos }).collect(),
                body: LambdaBody::Expr(Box::new(body)),
            },
            pos,
        };
        let call = |target: ast::Expr, args: Vec<ast::Expr>| ast::Expr {
            kind: ExprKind::Call { target: Some(Box::new(target)), type_args: Vec::new(), name: name.clone(), args },
            pos,
        };
        // What stands before `::` as an expression, when it is a type.
        let type_expr: Option<ast::Expr> = match target {
            RefTarget::Type(tr) => {
                if name == "new" {
                    let body = ast::Expr { kind: ExprKind::New { ty: tr.clone(), args: (0..n).map(arg).collect() }, pos };
                    return self.lambda(&lam(body), t);
                }
                let mut x = ast::Expr { kind: ExprKind::Name(tr.name[0].clone()), pos };
                for part in &tr.name[1..] {
                    x = ast::Expr { kind: ExprKind::Field(Box::new(x), part.clone()), pos };
                }
                Some(x)
            }
            RefTarget::Expr(x) => {
                let snap = self.snapshot();
                let is_type = matches!(self.target(x), Target::Type(_));
                self.restore(snap);
                if is_type {
                    Some((**x).clone())
                } else {
                    None
                }
            }
        };
        match type_expr {
            Some(tx) => {
                // A static method first; then an instance method of the first argument.
                let snap = self.snapshot();
                let r = self.lambda(&lam(call(tx.clone(), (0..n).map(arg).collect())), t);
                if !self.failed_since(&snap) || n == 0 {
                    return r;
                }
                self.restore(snap);
                self.lambda(&lam(call(arg(0), (1..n).map(arg).collect())), t)
            }
            None => {
                let RefTarget::Expr(x) = target else { unreachable!() };
                // The receiver is evaluated once, when the reference is.
                let v = self.check_expr(x, None);
                if v.ty.is_error() {
                    return v;
                }
                let hidden = format!("$mr{}", self.frame().locals.len());
                let id = self.declare(&hidden, v.ty.clone(), true, pos, false);
                let f = self.frame();
                f.flow.assigned.insert(id);
                f.flow.maybe.insert(id);
                let recv = ast::Expr { kind: ExprKind::Name(hidden), pos };
                let l = self.lambda(&lam(call(recv, (0..n).map(arg).collect())), t);
                let ty = l.ty.clone();
                TExpr { kind: TKind::Let(id, Box::new(v), Box::new(l)), ty }
            }
        }
    }
}
