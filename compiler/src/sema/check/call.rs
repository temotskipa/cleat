//! Calls: member lookup, overload resolution (section 4.4) and inference (section 7.5).

use super::expr::{is_literal_expr, needs_expected};
use super::*;
use crate::ast::{ExprKind, Variance};
use crate::sema::decl::{self, ImplOf};

/// An argument of a call: source still to be checked, or a value already checked.
pub enum ArgIn<'a> {
    Ast(&'a ast::Expr),
    Done(TExpr),
}

pub struct CallSite<'a> {
    pub args: Vec<ArgIn<'a>>,
    pub targs: Option<Vec<Type>>,
    pub expected: Option<Type>,
    pub pos: Pos,
    /// The numeric class a literal argument takes when its parameter names none
    /// (section 6.3, rule 2).
    pub lit_hint: Option<ClassId>,
    /// The operator that spells this call, for messages.
    pub operator: Option<&'static str>,
}

impl<'a> CallSite<'a> {
    pub fn new(args: Vec<ArgIn<'a>>, pos: Pos) -> CallSite<'a> {
        CallSite { args, targs: None, expected: None, pos, lit_hint: None, operator: None }
    }
}

/// An argument before the method is chosen.
enum PreArg {
    Typed(TExpr),
    /// It is checked against the parameter's type once that is known.
    Deferred,
}

#[derive(Clone)]
struct Cand {
    callee: MethodRef,
    is_ctor: bool,
    class_subst: Subst,
    params: Vec<Type>,
    varargs: bool,
    ret: Type,
    ntparams: usize,
}

struct Applied {
    cand: usize,
    targs: Vec<Type>,
    /// The type each argument is given for.
    ptypes: Vec<Type>,
    /// The first argument that goes into the varargs array, when the call builds one,
    /// and the array's element type.
    spread_from: Option<(usize, Type)>,
    ret: Type,
}

enum Why {
    Arity,
    Infer,
    Arg,
}

#[derive(Clone, Default)]
struct Bounds {
    fixed: Vec<Type>,
    lower: Vec<Type>,
    upper: Vec<Type>,
    /// The upper bound the expected type gave, when the result type is the parameter.
    expected: Option<Type>,
}

#[derive(Clone, Copy, PartialEq)]
enum Dir {
    Co,
    Contra,
    Inv,
}

fn compose(d: Dir, v: Variance) -> Dir {
    match (d, v) {
        (Dir::Inv, _) | (_, Variance::Invariant) => Dir::Inv,
        (Dir::Co, Variance::Out) | (Dir::Contra, Variance::In) => Dir::Co,
        (Dir::Co, Variance::In) | (Dir::Contra, Variance::Out) => Dir::Contra,
    }
}

fn as_lambda(e: &ast::Expr) -> Option<(&Vec<ast::LambdaParam>, &ast::LambdaBody)> {
    match &e.kind {
        ExprKind::Lambda { params, body } => Some((params, body)),
        ExprKind::Paren(x) => as_lambda(x),
        _ => None,
    }
}

fn is_method_ref(e: &ast::Expr) -> bool {
    match &e.kind {
        ExprKind::MethodRef { .. } => true,
        ExprKind::Paren(x) => is_method_ref(x),
        _ => false,
    }
}

impl<'p> Checker<'p> {
    // ---- candidates ----

    /// The methods named `name` that a type declares or inherits, each with the
    /// substitution that reads its signature through that type.
    pub fn method_candidates(&mut self, t: &Type, name: &str) -> Vec<(MethodRef, Subst)> {
        let mut out = Vec::new();
        self.gather(t, name, &mut out, 0);
        out
    }

    fn gather(&mut self, t: &Type, name: &str, out: &mut Vec<(MethodRef, Subst)>, depth: u32) {
        match &t.ty {
            Ty::Class(id, args) => {
                let s = Subst::for_class(*id, args);
                let mut types = vec![Type::class(*id, args.clone())];
                for sup in decl::all_supertypes(self.p, *id) {
                    types.push(s.apply(&sup));
                }
                for ty in types {
                    let Ty::Class(cid, cargs) = &ty.ty else { continue };
                    let c = self.p.class(*cid);
                    for (i, m) in c.methods.iter().enumerate() {
                        if m.name != name {
                            continue;
                        }
                        let mref = MethodRef { class: *cid, index: i as u32 };
                        if !out.iter().any(|(o, _)| *o == mref) {
                            out.push((mref, Subst::for_class(*cid, cargs)));
                        }
                    }
                }
            }
            Ty::Var(tv) if depth < 8 => {
                for b in self.p.bounds_of(*tv) {
                    self.gather(&b, name, out, depth + 1);
                }
            }
            _ => {}
        }
    }

    fn drop_overridden(&self, cands: &mut Vec<(MethodRef, Subst)>) {
        let all: Vec<MethodRef> = cands.iter().map(|(m, _)| *m).collect();
        cands.retain(|(f, _)| !all.iter().any(|g| g != f && self.p.method(*g).overrides.contains(f)));
    }

    fn method_cand(&self, m: MethodRef, s: &Subst) -> Cand {
        let md = self.p.method(m);
        Cand {
            callee: m,
            is_ctor: false,
            class_subst: s.clone(),
            params: md.params.iter().map(|p| s.apply(&p.ty)).collect(),
            varargs: md.params.last().map(|p| p.varargs).unwrap_or(false),
            ret: s.apply(&md.ret),
            ntparams: md.tparams.len(),
        }
    }

    fn accessible_methods(&mut self, cands: Vec<(MethodRef, Subst)>, recv: Option<&Type>, name: &str, pos: Pos) -> Option<Vec<(MethodRef, Subst)>> {
        let mut out = Vec::new();
        for (m, s) in &cands {
            let md = self.p.method(*m);
            let (aud, only) = (md.aud, md.only.clone());
            if self.can_access(m.class, aud, &only, recv) {
                out.push((*m, s.clone()));
            }
        }
        if out.is_empty() && !cands.is_empty() {
            let c = self.p.class(cands[0].0.class).name.clone();
            self.err(pos, format!("the method `{name}` of `{c}` is outside its audience here"));
            return None;
        }
        Some(out)
    }

    // ---- arguments ----

    fn precheck(&mut self, site: &CallSite) -> Vec<PreArg> {
        let mut out = Vec::new();
        for a in &site.args {
            out.push(match a {
                ArgIn::Done(e) => PreArg::Typed(self.captured_arg(e.clone())),
                ArgIn::Ast(e) if needs_expected(e) => PreArg::Deferred,
                ArgIn::Ast(e) => {
                    // A call may need the parameter's type to infer its own type arguments.
                    let speculative = matches!(&e.kind, ExprKind::Call { type_args, .. } if type_args.is_empty());
                    if speculative {
                        let snap = self.snapshot();
                        let v = self.check_expr(e, None);
                        if self.failed_since(&snap) {
                            self.restore(snap);
                            PreArg::Deferred
                        } else {
                            PreArg::Typed(self.captured_arg(v))
                        }
                    } else {
                        let v = self.check_expr(e, None);
                        PreArg::Typed(self.captured_arg(v))
                    }
                }
            });
        }
        out
    }

    /// An argument of a wildcard type stands for one unknown type in this call.
    fn captured_arg(&mut self, e: TExpr) -> TExpr {
        if !e.ty.has_wildcard_arg() {
            return e;
        }
        let ty = self.capture(&e.ty);
        TExpr { kind: TKind::Coerce(Box::new(e)), ty }
    }

    // ---- applicability ----

    fn apply(&mut self, cands: &[Cand], ci: usize, pre: &[PreArg], site: &CallSite, conv: bool, strict: bool) -> Result<Applied, Why> {
        let c = &cands[ci];
        let (n, k) = (c.params.len(), pre.len());
        let mut shapes: Vec<(Vec<Type>, Option<(usize, Type)>)> = Vec::new();
        if k == n {
            shapes.push((c.params.clone(), None));
        }
        if c.varargs && k + 1 >= n {
            let elem = self.p.array_elem(&c.params[n - 1]).unwrap_or_else(Type::error);
            let mut v: Vec<Type> = c.params[..n - 1].to_vec();
            for _ in n - 1..k {
                v.push(elem.clone());
            }
            shapes.push((v, Some((n - 1, elem))));
        }
        if shapes.is_empty() {
            return Err(Why::Arity);
        }
        let mut last = Why::Arg;
        let count = shapes.len();
        for (si, (ptypes, spread)) in shapes.into_iter().enumerate() {
            let is_last = si + 1 == count;
            let targs: Vec<Type> = if c.ntparams == 0 {
                Vec::new()
            } else if let Some(t) = &site.targs {
                if t.len() != c.ntparams {
                    last = Why::Infer;
                    continue;
                }
                t.clone()
            } else {
                match self.infer(c, &ptypes, pre, site) {
                    Some(t) => t,
                    None => {
                        last = Why::Infer;
                        continue;
                    }
                }
            };
            let mut ms = Subst::new();
            for (i, t) in targs.iter().enumerate() {
                ms.bind(Tv { owner: TvOwner::Method(c.callee), index: i as u32 }, t.clone());
            }
            let ptypes: Vec<Type> = ptypes.iter().map(|t| ms.apply(t)).collect();
            let ret = ms.apply(&c.ret);
            let spread = spread.map(|(from, elem)| (from, ms.apply(&elem)));
            let mut ok = true;
            if strict || !is_last {
                for (i, a) in pre.iter().enumerate() {
                    if !self.arg_applicable(a, &site.args[i], &ptypes[i], site, conv) {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                return Ok(Applied { cand: ci, targs, ptypes, spread_from: spread, ret });
            }
            last = Why::Arg;
        }
        Err(last)
    }

    fn arg_applicable(&mut self, pre: &PreArg, arg: &ArgIn, pt: &Type, site: &CallSite, conv: bool) -> bool {
        match pre {
            PreArg::Typed(e) => e.ty.is_error() || self.assignable(&e.ty, pt, conv),
            PreArg::Deferred => {
                let ArgIn::Ast(e) = arg else { return true };
                if let Some((params, _)) = as_lambda(e) {
                    // The body of a lambda plays no part in choosing the method.
                    let Some((fm, fs)) = self.functional_method(pt) else { return false };
                    let fparams: Vec<Type> = self.p.method(fm).params.iter().map(|p| fs.apply(&p.ty)).collect();
                    if fparams.len() != params.len() {
                        return false;
                    }
                    for (lp, ft) in params.iter().zip(fparams.iter()) {
                        if let Some(w) = &lp.ty {
                            let snap = self.snapshot();
                            let wt = self.resolve(w);
                            self.restore(snap);
                            if !wt.is_error() && !self.p.same_type(&wt, ft) {
                                return false;
                            }
                        }
                    }
                    return true;
                }
                if is_method_ref(e) {
                    return self.functional_method(pt).is_some();
                }
                if is_literal_expr(e) {
                    return match self.p.numeric_class(pt) {
                        Some(c) => self.literal_fits(e, c),
                        None => {
                            let class = site.lit_hint.unwrap_or_else(|| self.default_class_of(e));
                            self.literal_fits(e, class) && self.assignable(&Type::simple(class), pt, conv)
                        }
                    };
                }
                let snap = self.snapshot();
                let v = self.check_expr(e, Some(pt));
                let ok = !self.failed_since(&snap) && self.assignable(&v.ty, pt, conv);
                self.restore(snap);
                ok
            }
        }
    }

    fn default_class_of(&self, e: &ast::Expr) -> ClassId {
        fn dec(e: &ast::Expr) -> bool {
            match &e.kind {
                ExprKind::Dec(_) => true,
                ExprKind::Paren(x) | ExprKind::Unary(_, x) => dec(x),
                ExprKind::Binary(_, a, b) => dec(a) || dec(b),
                _ => false,
            }
        }
        if dec(e) {
            self.p.wk.rational
        } else {
            self.p.wk.int
        }
    }

    // ---- inference ----

    fn collect(&mut self, owner: TvOwner, b: &mut Vec<Bounds>, pt: &Type, at: &Type, dir: Dir) {
        if at.is_error() || pt.is_error() {
            return;
        }
        match &pt.ty {
            Ty::Var(tv) if tv.owner == owner => {
                if at.is_null_literal() {
                    return;
                }
                let mut a = at.clone();
                // `@Nullable P` takes the nullability on itself.
                if pt.nullable && dir == Dir::Co {
                    a.nullable = false;
                }
                let slot = &mut b[tv.index as usize];
                match dir {
                    Dir::Co => slot.lower.push(a),
                    Dir::Contra => slot.upper.push(a),
                    Dir::Inv => slot.fixed.push(a),
                }
            }
            Ty::Class(pid, pargs) if pt.mentions(&|tv| tv.owner == owner) => {
                let (pargs, aargs, gid): (Vec<Arg>, Vec<Arg>, ClassId) = match dir {
                    Dir::Co => {
                        let Some(up) = self.p.supertype_at(at, *pid) else { return };
                        (pargs.clone(), up.args().to_vec(), *pid)
                    }
                    Dir::Contra => {
                        let Some(aid) = at.class_id() else { return };
                        let Some(up) = self.p.supertype_at(pt, aid) else { return };
                        (up.args().to_vec(), at.args().to_vec(), aid)
                    }
                    Dir::Inv => {
                        if at.class_id() != Some(*pid) {
                            return;
                        }
                        (pargs.clone(), at.args().to_vec(), *pid)
                    }
                };
                for (i, (pa, aa)) in pargs.iter().zip(aargs.iter()).enumerate() {
                    let v = self.p.class(gid).tparams[i].variance;
                    let d = compose(dir, v);
                    match (pa, aa) {
                        (Arg::Ty(p), Arg::Ty(a)) => self.collect(owner, b, p, a, d),
                        (Arg::Ty(p), Arg::Wild(e, s)) => match d {
                            Dir::Co => {
                                if let Some(e) = e {
                                    self.collect(owner, b, p, e, Dir::Co);
                                }
                            }
                            Dir::Contra => {
                                if let Some(s) = s {
                                    self.collect(owner, b, p, s, Dir::Contra);
                                }
                            }
                            // A wildcard that is not the argument's own fixes nothing.
                            Dir::Inv => {}
                        },
                        (Arg::Wild(pe, ps), a) => {
                            if let Some(pe) = pe {
                                let d = compose(dir, Variance::Out);
                                match a {
                                    Arg::Ty(a) => self.collect(owner, b, pe, a, d),
                                    Arg::Wild(Some(e), _) => self.collect(owner, b, pe, e, d),
                                    _ => {}
                                }
                            }
                            if let Some(ps) = ps {
                                let d = compose(dir, Variance::In);
                                match a {
                                    Arg::Ty(a) => self.collect(owner, b, ps, a, d),
                                    Arg::Wild(_, Some(s)) => self.collect(owner, b, ps, s, d),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// The type the constraints determine, `Ok(None)` when they determine nothing, and
    /// `Err` when they conflict.
    fn solve_one(&mut self, b: &Bounds) -> Result<Option<Type>, ()> {
        if let Some(first) = b.fixed.first() {
            for other in &b.fixed[1..] {
                if !self.p.same_type(first, other) {
                    return Err(());
                }
            }
            return Ok(Some(first.clone()));
        }
        if !b.lower.is_empty() {
            for cand in &b.lower {
                let mut all = true;
                for other in &b.lower {
                    if !self.p.is_subtype(other, cand) {
                        all = false;
                        break;
                    }
                }
                if all {
                    return Ok(Some(cand.clone()));
                }
            }
            return match &b.expected {
                Some(t) => Ok(Some(t.clone())),
                None => Err(()),
            };
        }
        if !b.upper.is_empty() {
            for cand in &b.upper {
                let mut all = true;
                for other in &b.upper {
                    if !self.p.is_subtype(cand, other) {
                        all = false;
                        break;
                    }
                }
                if all {
                    return Ok(Some(cand.clone()));
                }
            }
            return Err(());
        }
        Ok(None)
    }

    fn partial(&mut self, owner: TvOwner, b: &[Bounds]) -> Subst {
        let mut s = Subst::new();
        for (i, slot) in b.iter().enumerate() {
            if let Ok(Some(t)) = self.solve_one(slot) {
                s.bind(Tv { owner, index: i as u32 }, t);
            }
        }
        s
    }

    fn infer(&mut self, c: &Cand, ptypes: &[Type], pre: &[PreArg], site: &CallSite) -> Option<Vec<Type>> {
        let n = c.ntparams;
        self.next_infer += 1;
        let owner = TvOwner::Infer(self.next_infer);
        let mut to_inf = Subst::new();
        for i in 0..n {
            to_inf.bind(
                Tv { owner: TvOwner::Method(c.callee), index: i as u32 },
                Type::var(Tv { owner, index: i as u32 }),
            );
        }
        let pts: Vec<Type> = ptypes.iter().map(|t| to_inf.apply(t)).collect();
        let ret = to_inf.apply(&c.ret);
        let mut b = vec![Bounds::default(); n];
        let mentions = |t: &Type| t.mentions(&|tv| tv.owner == owner);

        for (i, a) in pre.iter().enumerate() {
            if let PreArg::Typed(e) = a {
                self.collect(owner, &mut b, &pts[i], &e.ty, Dir::Co);
            }
        }
        if let Some(exp) = &site.expected {
            match &ret.ty {
                Ty::Var(tv) if tv.owner == owner => {
                    let slot = &mut b[tv.index as usize];
                    slot.upper.push(exp.clone());
                    slot.expected = Some(exp.clone());
                }
                _ => self.collect(owner, &mut b, &ret, exp, Dir::Contra),
            }
        }
        // Lambdas, after the other arguments.
        for (i, a) in site.args.iter().enumerate() {
            let ArgIn::Ast(e) = a else { continue };
            let Some((lparams, _)) = as_lambda(e) else { continue };
            let pt = self.partial(owner, &b).apply(&pts[i]);
            let Some((fm, fs)) = self.functional_method(&pt) else { continue };
            let fmd = self.p.method(fm);
            let fparams: Vec<Type> = fmd.params.iter().map(|p| fs.apply(&p.ty)).collect();
            let fret = fs.apply(&fmd.ret);
            if fparams.len() != lparams.len() {
                continue;
            }
            let mut ptys = Vec::new();
            for (lp, ft) in lparams.iter().zip(fparams.iter()) {
                let mut t = ft.clone();
                if mentions(&t) {
                    if let Some(w) = &lp.ty {
                        let wt = self.resolve(w);
                        self.collect(owner, &mut b, &t, &wt, Dir::Inv);
                        t = self.partial(owner, &b).apply(&t);
                    }
                }
                if mentions(&t) {
                    return None;
                }
                ptys.push(t);
            }
            let rt = self.partial(owner, &b).apply(&fret);
            if mentions(&rt) {
                if let Some(res) = self.lambda_result(e, &ptys) {
                    self.collect(owner, &mut b, &rt, &res, Dir::Co);
                }
            }
        }
        // Numeric literals, last.
        for (i, a) in site.args.iter().enumerate() {
            let (ArgIn::Ast(e), PreArg::Deferred) = (a, &pre[i]) else { continue };
            if !is_literal_expr(e) {
                continue;
            }
            if let Ty::Var(tv) = &pts[i].ty {
                if tv.owner == owner && !matches!(self.solve_one(&b[tv.index as usize]), Ok(Some(_))) {
                    let class = site.lit_hint.unwrap_or_else(|| self.default_class_of(e));
                    b[tv.index as usize].lower.push(Type::simple(class));
                }
            }
        }
        let mut out = Vec::new();
        for slot in &b {
            match self.solve_one(slot) {
                Ok(Some(t)) => out.push(t),
                _ => return None,
            }
        }
        // Each type is within the parameter's declared bound and carries its tags.
        let mut ms = Subst::new();
        for (i, t) in out.iter().enumerate() {
            ms.bind(Tv { owner: TvOwner::Method(c.callee), index: i as u32 }, t.clone());
        }
        let tparams = self.p.method(c.callee).tparams.clone();
        for (i, tp) in tparams.iter().enumerate() {
            for bound in &tp.bounds {
                let bound = ms.apply(&c.class_subst.apply(bound));
                if !self.p.is_subtype(&out[i], &bound) {
                    return None;
                }
            }
            for tag in &tp.tags {
                if !decl::carries_tag(self.p, &out[i], *tag) {
                    return None;
                }
            }
        }
        Some(out)
    }

    // ---- choosing ----

    fn more_specific(&mut self, a: &Applied, b: &Applied, conv: bool) -> bool {
        match (&a.spread_from, &b.spread_from) {
            (None, Some(_)) => return true,
            (Some(_), None) => return false,
            _ => {}
        }
        if a.ptypes.len() != b.ptypes.len() {
            return false;
        }
        for (x, y) in a.ptypes.iter().zip(b.ptypes.iter()) {
            if !(self.p.is_subtype(x, y) || (conv && self.conversion(x, y).is_some())) {
                return false;
            }
        }
        true
    }

    fn resolve_call(&mut self, name: &str, what: &str, cands: &[Cand], site: &CallSite, pre: &[PreArg]) -> Option<Applied> {
        let pos = site.pos;
        if pre.iter().any(|a| matches!(a, PreArg::Typed(e) if e.ty.is_error())) {
            return None;
        }
        let mut apps: Vec<Applied> = Vec::new();
        let mut conv = false;
        for round in 0..2 {
            conv = round == 1;
            for ci in 0..cands.len() {
                if let Ok(a) = self.apply(cands, ci, pre, site, conv, true) {
                    apps.push(a);
                }
            }
            if !apps.is_empty() {
                break;
            }
        }
        if apps.is_empty() {
            if cands.len() == 1 {
                return match self.apply(cands, 0, pre, site, true, false) {
                    Ok(a) => Some(a),
                    Err(Why::Arity) => {
                        let n = cands[0].params.len();
                        self.err(pos, format!("{what} `{name}` takes {n} argument{}, and {} {} written", if n == 1 { "" } else { "s" }, pre.len(), if pre.len() == 1 { "is" } else { "are" }));
                        None
                    }
                    Err(_) => {
                        if site.targs.is_some() {
                            let n = cands[0].ntparams;
                            self.err(pos, format!("{what} `{name}` has {n} type parameter{}; a call writes all of its type arguments or none", if n == 1 { "" } else { "s" }));
                        } else {
                            self.err(pos, format!("the type arguments of `{name}` cannot be inferred from this call; write them, as in `.<T>{name}(...)`"));
                        }
                        None
                    }
                };
            }
            // An argument that failed on its own explains the failure better than the
            // call does: report its mistake.
            let mut reported = false;
            for (i, a) in site.args.iter().enumerate() {
                if let (ArgIn::Ast(e), PreArg::Deferred) = (a, &pre[i]) {
                    if matches!(e.kind, ExprKind::Call { .. }) {
                        let before = self.p.diags.len();
                        self.check_expr(e, None);
                        reported |= self.p.diags.len() > before;
                    }
                }
            }
            if reported {
                return None;
            }
            let mut shown = Vec::new();
            for a in pre {
                shown.push(match a {
                    PreArg::Typed(e) => self.show(&e.ty),
                    PreArg::Deferred => "_".to_string(),
                });
            }
            self.err(pos, format!("no {what} `{name}` accepts the arguments ({})", shown.join(", ")));
            return None;
        }
        if apps.len() > 1 {
            let mut best: Vec<usize> = Vec::new();
            for i in 0..apps.len() {
                let mut wins = true;
                for j in 0..apps.len() {
                    if i != j && !self.more_specific(&apps[i], &apps[j], conv) {
                        wins = false;
                        break;
                    }
                }
                if wins {
                    best.push(i);
                }
            }
            if best.len() > 1 {
                // The same signature reached two ways: a body wins over a bare declaration.
                let with_body: Vec<usize> = best
                    .iter()
                    .copied()
                    .filter(|i| cands[apps[*i].cand].is_ctor || !self.p.method(cands[apps[*i].cand].callee).is_abstract)
                    .collect();
                best = if with_body.is_empty() { vec![best[0]] } else { vec![with_body[0]] };
            }
            if best.is_empty() {
                // Section 6.3: among methods a literal fits, the one of its default class.
                let mut by_default: Vec<usize> = Vec::new();
                'apps: for (i, a) in apps.iter().enumerate() {
                    let mut any = false;
                    for (k, arg) in site.args.iter().enumerate() {
                        let (ArgIn::Ast(e), PreArg::Deferred) = (arg, &pre[k]) else { continue };
                        if !is_literal_expr(e) {
                            continue;
                        }
                        any = true;
                        if a.ptypes[k].class_id() != Some(self.default_class_of(e)) {
                            continue 'apps;
                        }
                    }
                    if any {
                        by_default.push(i);
                    }
                }
                if by_default.len() == 1 {
                    best = by_default;
                }
            }
            if best.len() != 1 {
                self.err(pos, format!("the call of `{name}` is ambiguous: {} methods accept these arguments and none is more specific than the rest; write a cast or explicit type arguments", apps.len()));
                return None;
            }
            return Some(apps.swap_remove(best[0]));
        }
        apps.pop()
    }

    /// Checks each argument against the parameter the chosen method gives it.
    fn finish_args(&mut self, a: &Applied, site: CallSite, pre: Vec<PreArg>) -> Vec<TExpr> {
        let mut out = Vec::new();
        let hint = site.lit_hint;
        for (i, (arg, p)) in site.args.into_iter().zip(pre.into_iter()).enumerate() {
            let pt = &a.ptypes[i];
            let pos = match &arg {
                ArgIn::Ast(e) => e.pos,
                ArgIn::Done(_) => site.pos,
            };
            let v = match (p, &arg) {
                (PreArg::Typed(e), _) => e,
                (PreArg::Deferred, ArgIn::Ast(e)) => {
                    if is_literal_expr(e) && self.p.numeric_class(pt).is_none() {
                        let class = hint.unwrap_or_else(|| self.default_class_of(e));
                        self.literal_at(e, class)
                    } else {
                        self.check_expr(e, Some(pt))
                    }
                }
                (PreArg::Deferred, ArgIn::Done(e)) => e.clone(),
            };
            out.push(self.coerce(v, pt, pos));
        }
        if let Some((from, elem)) = a.spread_from.clone() {
            let rest: Vec<TExpr> = out.split_off(from);
            out.push(TExpr { kind: TKind::ArrayLit { elem: elem.clone(), elems: rest }, ty: self.p.array_of(elem) });
        }
        out
    }

    fn check_explicit_targs(&mut self, c: &Cand, a: &Applied, pos: Pos) {
        let mut ms = Subst::new();
        for (i, t) in a.targs.iter().enumerate() {
            ms.bind(Tv { owner: TvOwner::Method(c.callee), index: i as u32 }, t.clone());
        }
        let tparams = self.p.method(c.callee).tparams.clone();
        for (i, tp) in tparams.iter().enumerate() {
            let Some(t) = a.targs.get(i) else { continue };
            if t.is_error() {
                continue;
            }
            for bound in &tp.bounds {
                let bound = ms.apply(&c.class_subst.apply(bound));
                if !self.p.is_subtype(t, &bound) {
                    let (x, y) = (self.show(t), self.show(&bound));
                    self.err(pos, format!("`{x}` is not within the bound `{y}` of the type parameter `{}`", tp.name));
                }
            }
            for tag in &tp.tags {
                if !decl::carries_tag(self.p, t, *tag) {
                    let (x, y) = (self.show(t), self.p.class(*tag).name.clone());
                    self.err(pos, format!("`{x}` does not carry the tag `@{y}` that the type parameter requires"));
                }
            }
        }
    }

    fn fail(&mut self, site: CallSite) -> TExpr {
        // Check what can still be checked, so mistakes inside the arguments are reported.
        for a in &site.args {
            if let ArgIn::Ast(e) = a {
                if !needs_expected(e) && !matches!(e.kind, ExprKind::Call { .. }) {
                    self.check_expr(e, None);
                }
            }
        }
        TExpr { kind: TKind::Null, ty: Type::error() }
    }

    fn class_args(&mut self, recv: &Type, class: ClassId) -> Vec<Arg> {
        match self.p.supertype_at(recv, class) {
            Some(up) => up.args().to_vec(),
            None => Vec::new(),
        }
    }

    // ---- the kinds of call ----

    /// `recv.name(args)` on a value.
    pub fn call_method(&mut self, recv: TExpr, name: &str, site: CallSite) -> TExpr {
        let pos = site.pos;
        if recv.ty.is_error() {
            return self.fail(site);
        }
        let recv_ty = if recv.ty.is_null_literal() { self.p.object().nullable() } else { self.capture(&recv.ty) };
        let recv = if recv_ty == recv.ty { recv } else { TExpr { kind: TKind::Coerce(Box::new(recv)), ty: recv_ty.clone() } };
        let mut cands = self.method_candidates(&recv_ty, name);
        cands.retain(|(m, _)| !self.p.method(*m).is_static);
        if cands.is_empty() {
            let s = self.show(&recv_ty);
            match site.operator {
                Some(op) => self.err(pos, format!("`{op}` is the method `{name}`, and `{s}` has no method `{name}`")),
                None => self.err(pos, format!("`{s}` has no method named `{name}`")),
            }
            return self.fail(site);
        }
        if self.p.may_be_null(&recv_ty) {
            cands.retain(|(m, _)| self.p.method(*m).recv_nullable);
            if cands.is_empty() {
                let s = self.show(&recv_ty);
                self.err(pos, format!("`{name}` is sent to a `{s}`, which may be `null`; only a method that declares a `@Nullable` receiver is sent to it until it is narrowed"));
                return self.fail(site);
            }
        }
        self.drop_overridden(&mut cands);
        let Some(cands) = self.accessible_methods(cands, Some(&recv_ty), name, pos) else { return self.fail(site) };
        let cs: Vec<Cand> = cands.iter().map(|(m, s)| self.method_cand(*m, s)).collect();
        let pre = self.precheck(&site);
        let Some(app) = self.resolve_call(name, "method", &cs, &site, &pre) else { return TExpr { kind: TKind::Null, ty: Type::error() } };
        let c = cs[app.cand].clone();
        self.finish_method(Some(recv), c, app, site, pre, false)
    }

    fn finish_method(&mut self, recv: Option<TExpr>, c: Cand, app: Applied, site: CallSite, pre: Vec<PreArg>, is_super: bool) -> TExpr {
        let pos = site.pos;
        if site.targs.is_some() {
            self.check_explicit_targs(&c, &app, pos);
        }
        let mut m = c.callee;
        let md = self.p.method(m);
        let (name, recv_nullable, recv_quals, deprecated, is_static) =
            (md.name.clone(), md.recv_nullable, md.recv_quals.clone(), md.deprecated.clone(), md.is_static);
        let mut is_virtual = md.is_virtual(self.p.class(m.class).is_interface());
        self.deprecation(m.class, &format!("the method `{name}`"), &deprecated, pos);
        if let Some(r) = &recv {
            // The receiver's type is a subtype of the method's receiver type (section 8.4).
            let want = Type { ty: r.ty.ty.clone(), nullable: recv_nullable || r.ty.nullable, quals: recv_quals.clone() };
            let mut have = r.ty.clone();
            if recv_nullable {
                have.nullable = want.nullable;
            }
            if !self.p.is_subtype(&have, &want) {
                let (x, y) = (self.show(&r.ty), self.show(&want));
                self.err(pos, format!("`{name}` is sent to a `{x}`, and its receiver is declared `{y}`"));
            }
        }
        let args = self.finish_args(&app, site, pre);
        let mut dispatch = Dispatch::Direct;
        let mut class_args = Vec::new();
        if let Some(r) = &recv {
            if is_virtual && !is_super {
                dispatch = Dispatch::Virtual;
                // A final class has one body for the signature, known here.
                if let (Some(rc), false) = (r.ty.class_id(), self.p.may_be_null(&r.ty)) {
                    if self.p.class(rc).is_final() {
                        if let ImplOf::Found(f) = decl::impl_of(self.p, rc, m) {
                            if !self.p.method(f).is_abstract {
                                m = f;
                                dispatch = Dispatch::Direct;
                                is_virtual = false;
                            }
                        }
                    }
                }
            }
            class_args = self.class_args(&r.ty, m.class);
        }
        let _ = (is_virtual, is_static);
        let ty = self.project(&app.ret);
        let call = Call { recv, method: m, class_args, targs: app.targs, args, dispatch };
        let e = TExpr { kind: TKind::Call(Box::new(call)), ty };
        self.fold(e, pos)
    }

    /// `Type.name(args)`.
    pub fn call_static(&mut self, class: ClassId, name: &str, site: CallSite) -> TExpr {
        let pos = site.pos;
        let cname = self.p.class(class).name.clone();
        let n = self.p.class(class).tparams.len();
        let own = Type::class(class, (0..n).map(|i| Arg::Ty(Type::var(Tv { owner: TvOwner::Class(class), index: i as u32 }))).collect());
        let all = self.method_candidates(&own, name);
        let mut cands = all.clone();
        cands.retain(|(m, _)| {
            let md = self.p.method(*m);
            md.is_static && !md.static_requirement
        });
        if cands.is_empty() {
            if all.iter().any(|(m, _)| self.p.method(*m).static_requirement) {
                self.err(pos, format!("`{name}` is a static requirement of `{cname}`; it is called through a type parameter, not on the interface"));
            } else if all.is_empty() {
                self.err(pos, format!("`{cname}` has no static method named `{name}`"));
            } else {
                self.err(pos, format!("`{name}` is an instance method of `{cname}` and is called on a value"));
            }
            return self.fail(site);
        }
        let Some(cands) = self.accessible_methods(cands, None, name, pos) else { return self.fail(site) };
        let cs: Vec<Cand> = cands.iter().map(|(m, _)| self.method_cand(*m, &Subst::new())).collect();
        let pre = self.precheck(&site);
        let Some(app) = self.resolve_call(name, "method", &cs, &site, &pre) else { return TExpr { kind: TKind::Null, ty: Type::error() } };
        let c = cs[app.cand].clone();
        self.finish_method(None, c, app, site, pre, false)
    }

    /// `T.name(args)`: a static requirement reached through a type parameter (section 7.6).
    pub fn call_static_req(&mut self, tv: Tv, name: &str, site: CallSite) -> TExpr {
        let pos = site.pos;
        let on = Type::var(tv);
        let mut cands = self.method_candidates(&on, name);
        cands.retain(|(m, _)| self.p.method(*m).static_requirement);
        if cands.is_empty() {
            let s = self.show(&on);
            self.err(pos, format!("no bound of `{s}` requires a static method named `{name}`; no other static member is reached through a type parameter"));
            return self.fail(site);
        }
        let cs: Vec<Cand> = cands.iter().map(|(m, s)| self.method_cand(*m, s)).collect();
        let pre = self.precheck(&site);
        let Some(app) = self.resolve_call(name, "method", &cs, &site, &pre) else { return TExpr { kind: TKind::Null, ty: Type::error() } };
        let method = cs[app.cand].callee;
        let ty = app.ret.clone();
        let args = self.finish_args(&app, site, pre);
        TExpr { kind: TKind::StaticReq { on, method, args }, ty }
    }

    /// `name(args)`: a method of the enclosing class.
    pub fn call_unqualified(&mut self, name: &str, site: CallSite) -> TExpr {
        let pos = site.pos;
        let own = self.self_type();
        let mut cands = self.method_candidates(&own, name);
        cands.retain(|(m, _)| !self.p.method(*m).static_requirement);
        if cands.is_empty() {
            let c = self.p.class(self.class).name.clone();
            self.err(pos, format!("`{c}` has no method named `{name}`"));
            return self.fail(site);
        }
        self.drop_overridden(&mut cands);
        let Some(cands) = self.accessible_methods(cands, Some(&own), name, pos) else { return self.fail(site) };
        let cs: Vec<Cand> = cands.iter().map(|(m, s)| self.method_cand(*m, s)).collect();
        let pre = self.precheck(&site);
        let Some(app) = self.resolve_call(name, "method", &cs, &site, &pre) else { return TExpr { kind: TKind::Null, ty: Type::error() } };
        let c = cs[app.cand].clone();
        if self.p.method(c.callee).is_static {
            return self.finish_method(None, c, app, site, pre, false);
        }
        if self.is_static {
            return self.error_expr(pos, format!("`{name}` is an instance method, and a static method has no receiver to call it on"));
        }
        if self.frames[0].ctor == CtorPhase::Before {
            return self.error_expr(pos, format!("no method is called on `this` before `super(...)` is called, and `{name}` is an instance method"));
        }
        let this = self.this_value(pos);
        if this.ty.is_error() {
            return this;
        }
        self.finish_method(Some(this), c, app, site, pre, false)
    }

    /// `super.name(args)`.
    pub fn call_super(&mut self, name: &str, site: CallSite) -> TExpr {
        let pos = site.pos;
        let Some(sup) = self.p.class(self.class).superclass.clone() else {
            return self.error_expr(pos, "this class has no superclass");
        };
        let this = self.this_value(pos);
        if this.ty.is_error() {
            return this;
        }
        let mut cands = self.method_candidates(&sup, name);
        cands.retain(|(m, _)| !self.p.method(*m).is_static);
        self.drop_overridden(&mut cands);
        if cands.is_empty() {
            let s = self.show(&sup);
            self.err(pos, format!("`{s}` has no method named `{name}`"));
            return self.fail(site);
        }
        let Some(cands) = self.accessible_methods(cands, None, name, pos) else { return self.fail(site) };
        let cs: Vec<Cand> = cands.iter().map(|(m, s)| self.method_cand(*m, s)).collect();
        let pre = self.precheck(&site);
        let Some(app) = self.resolve_call(name, "method", &cs, &site, &pre) else { return TExpr { kind: TKind::Null, ty: Type::error() } };
        let c = cs[app.cand].clone();
        if self.p.method(c.callee).is_abstract {
            return self.error_expr(pos, format!("`super.{name}` names an abstract method, which has no body to run"));
        }
        self.finish_method(Some(this), c, app, site, pre, true)
    }

    fn ctor_cands(&mut self, ty: &Type) -> Vec<Cand> {
        let Ty::Class(cid, args) = &ty.ty else { return Vec::new() };
        let s = Subst::for_class(*cid, args);
        let c = self.p.class(*cid);
        c.ctors
            .iter()
            .enumerate()
            .map(|(i, k)| Cand {
                callee: MethodRef { class: *cid, index: i as u32 },
                is_ctor: true,
                class_subst: s.clone(),
                params: k.params.iter().map(|p| s.apply(&p.ty)).collect(),
                varargs: k.params.last().map(|p| p.varargs).unwrap_or(false),
                ret: ty.clone(),
                ntparams: 0,
            })
            .collect()
    }

    fn resolve_ctor(&mut self, ty: &Type, site: CallSite, check_access: bool) -> Option<(MethodRef, Vec<TExpr>)> {
        let pos = site.pos;
        let cid = ty.class_id()?;
        let cname = self.p.class(cid).name.clone();
        let mut cs = self.ctor_cands(ty);
        if check_access {
            let before = cs.len();
            let mut kept = Vec::new();
            for c in cs {
                let k = &self.p.class(cid).ctors[c.callee.index as usize];
                let (aud, only) = (k.aud, k.only.clone());
                if self.can_access(cid, aud, &only, None) {
                    kept.push(c);
                }
            }
            cs = kept;
            if cs.is_empty() && before > 0 {
                self.err(pos, format!("the constructor of `{cname}` is outside its audience here"));
                self.fail(site);
                return None;
            }
        }
        if cs.is_empty() {
            self.err(pos, format!("`{cname}` has no constructor"));
            self.fail(site);
            return None;
        }
        let pre = self.precheck(&site);
        let app = self.resolve_call(&cname, "constructor", &cs, &site, &pre)?;
        let ctor = cs[app.cand].callee;
        let dep = self.p.class(cid).ctors[ctor.index as usize].deprecated.clone();
        self.deprecation(cid, &format!("this constructor of `{cname}`"), &dep, pos);
        let args = self.finish_args(&app, site, pre);
        Some((ctor, args))
    }

    /// `new C<..>(args)`.
    pub fn construct(&mut self, ty: Type, site: CallSite) -> TExpr {
        match self.resolve_ctor(&ty, site, true) {
            Some((ctor, args)) => TExpr { kind: TKind::New { ctor, args }, ty },
            None => TExpr { kind: TKind::Null, ty: Type::error() },
        }
    }

    /// `super(args)` or `this(args)` in a constructor.
    pub fn ctor_call(&mut self, is_super: bool, site: CallSite) -> Option<(MethodRef, Vec<TExpr>)> {
        let ty = if is_super {
            match self.p.class(self.class).superclass.clone() {
                Some(t) => t,
                None => {
                    self.err(site.pos, "this class has no superclass to construct");
                    return None;
                }
            }
        } else {
            self.self_type()
        };
        self.resolve_ctor(&ty, site, is_super)
    }

    // ---- functional interfaces ----

    /// The one abstract method of a functional interface (section 4.8), with the
    /// substitution that reads its signature through the type.
    pub fn functional_method(&mut self, t: &Type) -> Option<(MethodRef, Subst)> {
        let Ty::Class(id, args) = &t.ty else { return None };
        if !self.p.class(*id).is_interface() || t.has_wildcard_arg() {
            return None;
        }
        let s = Subst::for_class(*id, args);
        let mut types = vec![Type::class(*id, args.clone())];
        for sup in decl::all_supertypes(self.p, *id) {
            types.push(s.apply(&sup));
        }
        let mut abstracts: Vec<(MethodRef, Subst)> = Vec::new();
        let mut bodies: Vec<MethodRef> = Vec::new();
        for ty in &types {
            let Ty::Class(cid, cargs) = &ty.ty else { continue };
            if *cid == self.p.wk.object {
                continue;
            }
            for (i, m) in self.p.class(*cid).methods.iter().enumerate() {
                let mref = MethodRef { class: *cid, index: i as u32 };
                if m.static_requirement {
                    return None;
                }
                if m.is_static {
                    continue;
                }
                if m.is_abstract {
                    abstracts.push((mref, Subst::for_class(*cid, cargs)));
                } else {
                    bodies.push(mref);
                }
            }
        }
        let all: Vec<MethodRef> = abstracts.iter().map(|(m, _)| *m).chain(bodies.iter().copied()).collect();
        abstracts.retain(|(f, _)| !all.iter().any(|g| g != f && self.p.method(*g).overrides.contains(f)));
        if abstracts.len() == 1 {
            abstracts.pop()
        } else {
            None
        }
    }
}
