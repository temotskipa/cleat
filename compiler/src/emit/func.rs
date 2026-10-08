//! One function's code: statements and expressions of the typed tree as LLVM IR.

use super::*;
use crate::ast::BinOp;
use crate::sema::tir::*;

/// A computed value. A pointer to an object is kept in a frame slot from the moment it
/// exists, so the collector finds it whatever runs next.
#[derive(Clone, Debug)]
pub enum Val {
    Unit,
    Imm(Repr, String),
    Slot(u32),
}

#[derive(Clone, Debug)]
enum Store {
    Slot(u32),
    Alloca(String, Repr),
    None,
}

#[derive(Clone, Debug)]
enum Target {
    Break(u32),
    Continue(u32),
    Return,
}

/// A `finally` block or a `using` close that every way out of a region runs through.
struct Scope {
    label: String,
    sel: String,
    targets: Vec<Target>,
}

struct Jump {
    brk: String,
    cont: Option<String>,
    depth: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum EnvSrc {
    None,
    /// The type arguments of `this`, seen at the class.
    This(ClassId),
    /// The method's own type arguments.
    Targs,
    /// The class's, then the method's.
    Both(ClassId, usize, usize),
}

pub struct Func<'a, 'p> {
    pub e: &'a mut Emitter<'p>,
    body: String,
    allocas: String,
    ntmp: u32,
    nlabel: u32,
    open: bool,
    locals: Vec<Store>,
    slots_next: u32,
    slots_max: u32,
    this: Store,
    env: Vec<Tv>,
    env_src: EnvSrc,
    uses_env: bool,
    handlers: Vec<String>,
    scopes: Vec<Scope>,
    jumps: HashMap<u32, Jump>,
    ret: Repr,
    ret_store: Store,
}

impl<'a, 'p> Func<'a, 'p> {
    pub fn bare(e: &'a mut Emitter<'p>) -> Func<'a, 'p> {
        Func {
            e,
            body: String::new(),
            allocas: String::new(),
            ntmp: 0,
            nlabel: 0,
            open: true,
            locals: Vec::new(),
            slots_next: 0,
            slots_max: 0,
            this: Store::None,
            env: Vec::new(),
            env_src: EnvSrc::None,
            uses_env: false,
            handlers: Vec::new(),
            scopes: Vec::new(),
            jumps: HashMap::new(),
            ret: Repr::Unit,
            ret_store: Store::None,
        }
    }

    pub fn take_body(&mut self) -> String {
        std::mem::take(&mut self.body)
    }

    // ---- instructions and blocks ----

    pub fn tmp(&mut self) -> String {
        self.ntmp += 1;
        format!("%t{}", self.ntmp)
    }

    fn new_label(&mut self, hint: &str) -> String {
        self.nlabel += 1;
        format!("{hint}{}", self.nlabel)
    }

    pub fn raw(&mut self, line: &str) {
        self.body.push_str(line);
        self.body.push('\n');
    }

    pub fn ins(&mut self, line: &str) {
        if !self.open {
            // Code after a jump: it cannot run, and LLVM still wants it in a block.
            let l = self.new_label("dead");
            self.body.push_str(&format!("{l}:\n"));
            self.open = true;
        }
        self.body.push_str("  ");
        self.body.push_str(line);
        self.body.push('\n');
    }

    fn label(&mut self, l: &str) {
        if self.open {
            self.ins(&format!("br label %{l}"));
        }
        self.body.push_str(&format!("{l}:\n"));
        self.open = true;
    }

    fn br(&mut self, l: &str) {
        if self.open {
            self.ins(&format!("br label %{l}"));
            self.open = false;
        }
    }

    fn cond_br(&mut self, c: &str, a: &str, b: &str) {
        self.ins(&format!("br i1 {c}, label %{a}, label %{b}"));
        self.open = false;
    }

    /// An `i1` from a `Boolean`.
    fn truth(&mut self, v: &str) -> String {
        let t = self.tmp();
        self.ins(&format!("{t} = icmp ne i8 {v}, 0"));
        t
    }

    fn bool_of(&mut self, i1: &str) -> String {
        let t = self.tmp();
        self.ins(&format!("{t} = zext i1 {i1} to i8"));
        t
    }

    fn alloca(&mut self, r: Repr) -> String {
        self.ntmp += 1;
        let n = format!("%v{}", self.ntmp);
        self.allocas.push_str(&format!("  {n} = alloca {}\n", r.ll()));
        n
    }

    // ---- frame slots ----

    fn new_slot(&mut self) -> u32 {
        let k = self.slots_next;
        self.slots_next += 1;
        self.slots_max = self.slots_max.max(self.slots_next);
        k
    }

    fn slot_addr(&mut self, k: u32) -> String {
        let t = self.tmp();
        self.ins(&format!("{t} = getelementptr inbounds ptr, ptr %slots, i64 {k}"));
        t
    }

    fn load_slot(&mut self, k: u32) -> String {
        let a = self.slot_addr(k);
        let t = self.tmp();
        self.ins(&format!("{t} = load ptr, ptr {a}"));
        t
    }

    fn store_slot(&mut self, k: u32, v: &str) {
        let a = self.slot_addr(k);
        self.ins(&format!("store ptr {v}, ptr {a}"));
    }

    fn root(&mut self, v: &str) -> Val {
        let k = self.new_slot();
        self.store_slot(k, v);
        Val::Slot(k)
    }

    fn val(&mut self, v: &Val) -> String {
        match v {
            Val::Unit => "undef".into(),
            Val::Imm(_, s) => s.clone(),
            Val::Slot(k) => self.load_slot(*k),
        }
    }

    fn read(&mut self, s: &Store) -> Val {
        match s {
            Store::None => Val::Unit,
            Store::Slot(k) => {
                let v = self.load_slot(*k);
                self.root(&v)
            }
            Store::Alloca(a, r) => {
                let t = self.tmp();
                self.ins(&format!("{t} = load {}, ptr {a}", r.ll()));
                Val::Imm(*r, t)
            }
        }
    }

    fn write(&mut self, s: &Store, v: &Val) {
        match s {
            Store::None => {}
            Store::Slot(k) => {
                let x = self.val(v);
                self.store_slot(*k, &x);
            }
            Store::Alloca(a, r) => {
                let x = self.val(v);
                self.ins(&format!("store {} {x}, ptr {a}", r.ll()));
            }
        }
    }

    fn new_store(&mut self, r: Repr) -> Store {
        match r {
            Repr::Unit => Store::None,
            Repr::Ref => Store::Slot(self.new_slot()),
            m => Store::Alloca(self.alloca(m), m),
        }
    }

    // ---- exceptions ----

    fn handler(&self) -> String {
        self.handlers.last().cloned().unwrap_or_else(|| "unwind".to_string())
    }

    /// After a call: goes to the handler when an exception is raised.
    fn check(&mut self) {
        let e = self.tmp();
        self.ins(&format!("{e} = load ptr, ptr %ctx"));
        let c = self.tmp();
        self.ins(&format!("{c} = icmp eq ptr {e}, null"));
        let ok = self.new_label("ok");
        let h = self.handler();
        self.cond_br(&c, &ok, &h);
        self.label(&ok);
    }

    fn jump(&mut self, t: Target) {
        let depth = match &t {
            Target::Return => 0,
            Target::Break(l) | Target::Continue(l) => self.jumps.get(l).map(|j| j.depth).unwrap_or(0),
        };
        if self.scopes.len() > depth {
            // Through the innermost `finally` first: it continues the jump afterwards.
            let s = self.scopes.last_mut().unwrap();
            let k = s.targets.len() as u32 + 2;
            s.targets.push(t);
            let (sel, label) = (s.sel.clone(), s.label.clone());
            self.ins(&format!("store i32 {k}, ptr {sel}"));
            self.br(&label);
            return;
        }
        let to = match &t {
            Target::Return => "ret".to_string(),
            Target::Break(l) => self.jumps.get(l).map(|j| j.brk.clone()).unwrap_or_else(|| "ret".into()),
            Target::Continue(l) => self.jumps.get(l).and_then(|j| j.cont.clone()).unwrap_or_else(|| "ret".into()),
        };
        self.br(&to);
    }

    // ---- representations ----

    pub fn unit_object(&mut self) -> String {
        let t = self.tmp();
        self.ins(&format!("{t} = load ptr, ptr @cl_unit"));
        t
    }

    fn to_bits(&mut self, v: &str, m: Repr) -> String {
        let t = self.tmp();
        match m {
            Repr::I8 | Repr::I16 | Repr::I32 => self.ins(&format!("{t} = sext {} {v} to i64", m.ll())),
            Repr::U8 | Repr::U16 | Repr::U32 | Repr::Bool | Repr::Char => self.ins(&format!("{t} = zext {} {v} to i64", m.ll())),
            Repr::F64 => self.ins(&format!("{t} = bitcast double {v} to i64")),
            Repr::F32 => {
                let b = self.tmp();
                self.ins(&format!("{b} = bitcast float {v} to i32"));
                self.ins(&format!("{t} = zext i32 {b} to i64"));
            }
            Repr::Ptr => self.ins(&format!("{t} = ptrtoint ptr {v} to i64")),
            _ => return v.to_string(),
        }
        t
    }

    /// Changes how a value is held: boxing, unboxing, or nothing.
    pub fn convert_raw(&mut self, v: &str, from: Repr, to: Repr) -> String {
        if from == to {
            return v.to_string();
        }
        match (from, to) {
            (Repr::Unit, Repr::Ref) => self.unit_object(),
            (_, Repr::Unit) => String::new(),
            (m, Repr::Ref) if m.is_machine() => {
                let bits = self.to_bits(v, m);
                let t = self.tmp();
                self.ins(&format!("{t} = call ptr @cl_box(ptr %ctx, i32 {}, i64 {bits})", m.kind()));
                t
            }
            (Repr::Ref, m) if m.is_machine() => {
                let p = self.tmp();
                self.ins(&format!("{p} = getelementptr inbounds i8, ptr {v}, i64 16"));
                let t = self.tmp();
                self.ins(&format!("{t} = load {}, ptr {p}", m.ll()));
                t
            }
            _ => v.to_string(),
        }
    }

    fn convert(&mut self, v: Val, from: Repr, to: Repr) -> Val {
        if from == to {
            return v;
        }
        if to == Repr::Unit {
            return Val::Unit;
        }
        if from == Repr::Unit {
            let u = self.unit_object();
            return Val::Imm(Repr::Ref, u);
        }
        let s = self.val(&v);
        let r = self.convert_raw(&s, from, to);
        if to == Repr::Ref {
            self.check();
            self.root(&r)
        } else {
            Val::Imm(to, r)
        }
    }

    // ---- types at run time ----

    fn type_desc(&mut self, t: &Type) -> String {
        let env = self.env.clone();
        let missing = std::cell::Cell::new(false);
        let (sym, open) = self.e.texpr(t, &|tv| match env.iter().position(|e| *e == tv) {
            Some(i) => Some(i as u32),
            None => {
                missing.set(true);
                None
            }
        });
        if missing.get() {
            self.e.errors.push("this program uses the unknown type of a wildcard where the compiler cannot yet name it at run time".into());
        }
        let x = self.tmp();
        if open {
            self.uses_env = true;
            self.ins(&format!("{x} = call ptr @cl_type_eval(ptr {sym}, ptr %env)"));
        } else {
            let slot = self.e.cached_type(&sym);
            self.ins(&format!("{x} = load ptr, ptr {slot}"));
        }
        x
    }

    /// Runs a class's initialization unless it has finished.
    fn guard(&mut self, class: ClassId) {
        if self.e.is_trivial(class) {
            return;
        }
        let rec = self.e.init_rec(class);
        let s = self.tmp();
        self.ins(&format!("{s} = load i32, ptr {rec}"));
        let c = self.tmp();
        self.ins(&format!("{c} = icmp eq i32 {s}, 2"));
        let (done, slow) = (self.new_label("init.done"), self.new_label("init"));
        self.cond_br(&c, &done, &slow);
        self.label(&slow);
        self.ins(&format!("call void @cl_class_init(ptr %ctx, ptr {rec})"));
        self.check();
        self.label(&done);
    }

    // ---- expressions ----

    fn repr(&self, t: &Type) -> Repr {
        self.e.repr(t)
    }

    fn float_lit(&self, f: f64, r: Repr) -> String {
        let f = if r == Repr::F32 { (f as f32) as f64 } else { f };
        format!("0x{:016X}", f.to_bits())
    }

    fn expr(&mut self, e: &TExpr) -> Val {
        let r = self.repr(&e.ty);
        match &e.kind {
            TKind::Int(v) => Val::Imm(r, v.to_string()),
            TKind::Float(f) => Val::Imm(r, self.float_lit(*f, r)),
            TKind::Rational(text) => {
                let (slot, bytes, len) = self.e.rational_const(text);
                let t = self.tmp();
                self.ins(&format!("{t} = call ptr @cl_rational_const(ptr %ctx, ptr {slot}, ptr {bytes}, i64 {len})"));
                Val::Imm(Repr::Ref, t)
            }
            TKind::Bool(b) => Val::Imm(Repr::Bool, if *b { "1".into() } else { "0".into() }),
            TKind::Char(c) => Val::Imm(Repr::Char, (*c as u32).to_string()),
            TKind::Str(s) => Val::Imm(Repr::Ref, self.e.string_lit(s)),
            TKind::Null => Val::Imm(Repr::Ref, "null".into()),
            TKind::Unit => Val::Unit,
            TKind::Local(id) => {
                let s = self.locals[*id as usize].clone();
                self.read(&s)
            }
            TKind::This => match self.this.clone() {
                Store::Slot(k) => Val::Slot(k),
                other => self.read(&other),
            },
            TKind::Field(recv, f) => {
                let owner = recv.ty.class_id().unwrap_or(f.class);
                let o = self.expr(recv);
                let o = self.val(&o);
                self.load_field(&o, owner, *f, r)
            }
            TKind::StaticField(f) => {
                self.guard(f.class);
                let decl = self.repr(&self.e.p.field(*f).ty);
                if decl == Repr::Unit {
                    return Val::Unit;
                }
                if !self.e.is_trivial(f.class) {
                    self.assigned_check(*f);
                }
                let s = self.e.static_sym(*f);
                let t = self.tmp();
                self.ins(&format!("{t} = load {}, ptr {s}", decl.ll()));
                let v = if decl == Repr::Ref { self.root(&t) } else { Val::Imm(decl, t) };
                self.convert(v, decl, r)
            }
            TKind::Call(c) => self.call(c, &e.ty),
            TKind::StaticReq { on, method, args } => {
                let sig = self.e.sig(*method);
                let mut vals = Vec::new();
                for (a, pr) in args.iter().zip(sig.params.iter()) {
                    let v = self.expr(a);
                    let v = self.convert(v, self.repr(&a.ty), *pr);
                    vals.push((v, *pr));
                }
                let td = self.type_desc(on);
                let f = self.tmp();
                self.ins(&format!("{f} = call ptr @cl_static_req(ptr {td}, i32 {})", self.e.req_selector(*method)));
                let mut all = vec!["ptr %ctx".to_string()];
                for (v, pr) in &vals {
                    let x = self.val(v);
                    all.push(format!("{} {x}", pr.ll()));
                }
                self.finish_call(&f, &sig, all, r)
            }
            TKind::New { ctor, args } => {
                if r == Repr::Unit {
                    return Val::Unit;
                }
                let sig = self.e.ctor_sig(ctor.class, ctor.index);
                let mut vals = Vec::new();
                for (a, pr) in args.iter().zip(sig.params.iter()) {
                    let v = self.expr(a);
                    let v = self.convert(v, self.repr(&a.ty), *pr);
                    vals.push((v, *pr));
                }
                let td = self.type_desc(&e.ty.bare());
                let o = self.tmp();
                self.ins(&format!("{o} = call ptr @cl_new(ptr %ctx, ptr {td})"));
                self.check();
                let obj = self.root(&o);
                let this = self.val(&obj);
                let mut all = vec!["ptr %ctx".to_string(), format!("ptr {this}")];
                for (v, pr) in &vals {
                    let x = self.val(v);
                    all.push(format!("{} {x}", pr.ll()));
                }
                let f = self.e.ctor_symbol(ctor.class, ctor.index);
                self.ins(&format!("call void {f}({})", all.join(", ")));
                self.check();
                obj
            }
            TKind::NewArray { elem, len } => {
                let n = self.expr(len);
                let n = self.val(&n);
                let td = self.type_desc(&e.ty.bare());
                let a = self.tmp();
                self.ins(&format!("{a} = call ptr @cl_array_new(ptr %ctx, ptr {td}, i64 {n})"));
                self.check();
                let arr = self.root(&a);
                if self.repr(elem) == Repr::Ref && !elem.nullable && matches!(elem.ty, Ty::Class(..)) {
                    // Every element is the default value of the class.
                    if let Some(d) = self.default_value(elem, 0) {
                        let (a, d) = (self.val(&arr), self.val(&d));
                        self.ins(&format!("call void @cl_array_fill(ptr {a}, ptr {d})"));
                    }
                }
                arr
            }
            TKind::ArrayLit { elem, elems } => {
                let td = self.type_desc(&e.ty.bare());
                let a = self.tmp();
                self.ins(&format!("{a} = call ptr @cl_array_new(ptr %ctx, ptr {td}, i64 {})", elems.len()));
                self.check();
                let arr = self.root(&a);
                let er = self.repr(elem);
                let erased = !elem.nullable && matches!(elem.ty, Ty::Var(_));
                for (i, x) in elems.iter().enumerate() {
                    let v = self.expr(x);
                    let v = self.convert(v, self.repr(&x.ty), er);
                    let (a, v) = (self.val(&arr), self.val(&v));
                    if erased {
                        self.ins(&format!("call void @cl_Array_set_Int_Object(ptr %ctx, ptr {a}, i64 {i}, ptr {v})"));
                        self.check();
                    } else if er != Repr::Unit {
                        let p = self.elem_addr(&a, &i.to_string(), er);
                        self.ins(&format!("store {} {v}, ptr {p}", er.ll()));
                    }
                }
                arr
            }
            TKind::ClassLit(t) => {
                let td = self.type_desc(t);
                let c = self.tmp();
                self.ins(&format!("{c} = call ptr @cl_class_of(ptr {td})"));
                Val::Imm(Repr::Ref, c)
            }
            TKind::Lambda(l) => {
                let c = self.e.p.class(l.class);
                let fields: Vec<Type> = c.fields.iter().map(|f| f.ty.clone()).collect();
                let env = c.lambda.as_ref().map(|x| x.env.clone()).unwrap_or_default();
                let mut vals = Vec::new();
                for (cap, fty) in l.captures.iter().zip(fields.iter()) {
                    let v = self.expr(cap);
                    let fr = self.repr(fty);
                    let v = self.convert(v, self.repr(&cap.ty), fr);
                    vals.push((v, fr));
                }
                let ty = Type::class(l.class, env.iter().map(|tv| Arg::Ty(Type::var(*tv))).collect());
                let td = self.lambda_type(l.class, &ty, &env);
                let o = self.tmp();
                self.ins(&format!("{o} = call ptr @cl_new(ptr %ctx, ptr {td})"));
                self.check();
                let obj = self.root(&o);
                for (i, (v, fr)) in vals.iter().enumerate() {
                    if *fr == Repr::Unit {
                        continue;
                    }
                    let (o, x) = (self.val(&obj), self.val(v));
                    let (lty, index, _) = self.e.field_place(l.class, FieldRef { class: l.class, index: i as u32 });
                    let p = self.tmp();
                    self.ins(&format!("{p} = getelementptr inbounds {lty}, ptr {o}, i32 0, i32 {index}"));
                    self.ins(&format!("store {} {x}, ptr {p}", fr.ll()));
                }
                obj
            }
            TKind::And(a, b) | TKind::Or(a, b) => {
                let is_and = matches!(e.kind, TKind::And(..));
                let out = self.alloca(Repr::Bool);
                let x = self.expr(a);
                let x = self.val(&x);
                self.ins(&format!("store i8 {x}, ptr {out}"));
                let c = self.truth(&x);
                let (more, done) = (self.new_label("sc.rhs"), self.new_label("sc.end"));
                if is_and {
                    self.cond_br(&c, &more, &done);
                } else {
                    self.cond_br(&c, &done, &more);
                }
                self.label(&more);
                let y = self.expr(b);
                let y = self.val(&y);
                self.ins(&format!("store i8 {y}, ptr {out}"));
                self.label(&done);
                let t = self.tmp();
                self.ins(&format!("{t} = load i8, ptr {out}"));
                Val::Imm(Repr::Bool, t)
            }
            TKind::Not(a) => {
                let x = self.expr(a);
                let x = self.val(&x);
                let t = self.tmp();
                self.ins(&format!("{t} = xor i8 {x}, 1"));
                Val::Imm(Repr::Bool, t)
            }
            TKind::Cond(c, a, b) => {
                let out = self.new_store(r);
                let k = self.expr(c);
                let k = self.val(&k);
                let k = self.truth(&k);
                let (yes, no, done) = (self.new_label("then"), self.new_label("else"), self.new_label("cond.end"));
                self.cond_br(&k, &yes, &no);
                self.label(&yes);
                let x = self.expr(a);
                let x = self.convert(x, self.repr(&a.ty), r);
                self.write(&out, &x);
                self.br(&done);
                self.label(&no);
                let y = self.expr(b);
                let y = self.convert(y, self.repr(&b.ty), r);
                self.write(&out, &y);
                self.label(&done);
                self.read(&out)
            }
            TKind::Coalesce(a, b) => {
                let out = self.new_store(r);
                let x = self.expr(a);
                let xv = self.val(&x);
                let c = self.tmp();
                self.ins(&format!("{c} = icmp ne ptr {xv}, null"));
                let (have, none, done) = (self.new_label("have"), self.new_label("none"), self.new_label("coalesce.end"));
                self.cond_br(&c, &have, &none);
                self.label(&have);
                let x = self.convert(x, Repr::Ref, r);
                self.write(&out, &x);
                self.br(&done);
                self.label(&none);
                let y = self.expr(b);
                let y = self.convert(y, self.repr(&b.ty), r);
                self.write(&out, &y);
                self.label(&done);
                self.read(&out)
            }
            TKind::AssignLocal(id, v) => {
                let x = self.expr(v);
                let s = self.locals[*id as usize].clone();
                let lr = match &s {
                    Store::Slot(_) => Repr::Ref,
                    Store::Alloca(_, m) => *m,
                    Store::None => Repr::Unit,
                };
                let x = self.convert(x, self.repr(&v.ty), lr);
                self.write(&s, &x);
                self.convert(x, lr, r)
            }
            TKind::AssignField(recv, f, v) => {
                let owner = recv.ty.class_id().unwrap_or(f.class);
                let o = self.expr(recv);
                let x = self.expr(v);
                let (lty, index, fr) = self.e.field_place(owner, *f);
                let x = self.convert(x, self.repr(&v.ty), fr);
                let (o, xv) = (self.val(&o), self.val(&x));
                let p = self.tmp();
                self.ins(&format!("{p} = getelementptr inbounds {lty}, ptr {o}, i32 0, i32 {index}"));
                self.ins(&format!("store {} {xv}, ptr {p}", fr.ll()));
                self.convert(x, fr, r)
            }
            TKind::AssignStatic(f, v) => {
                let x = self.expr(v);
                self.guard(f.class);
                let decl = self.repr(&self.e.p.field(*f).ty);
                let x = self.convert(x, self.repr(&v.ty), decl);
                if decl != Repr::Unit {
                    let xv = self.val(&x);
                    let s = self.e.static_sym(*f);
                    self.ins(&format!("store {} {xv}, ptr {s}", decl.ll()));
                    let c = self.e.p.class(f.class);
                    self.ins(&format!("store i8 1, ptr @\"set.{}.{}\"", sym(&c.qname), sym(&c.fields[f.index as usize].name)));
                }
                self.convert(x, decl, r)
            }
            TKind::Cast(inner) => {
                let from = self.repr(&inner.ty);
                let v = self.expr(inner);
                if from != Repr::Ref {
                    return self.convert(v, from, r);
                }
                let td = self.type_desc(&e.ty);
                let o = self.val(&v);
                let t = self.tmp();
                self.ins(&format!("{t} = call ptr @cl_cast(ptr %ctx, ptr {o}, ptr {td})"));
                self.check();
                self.convert(v, Repr::Ref, r)
            }
            TKind::InstanceOf(inner, ty) => {
                let from = self.repr(&inner.ty);
                let v = self.expr(inner);
                let v = self.convert(v, from, Repr::Ref);
                let td = self.type_desc(ty);
                let o = self.val(&v);
                let t = self.tmp();
                self.ins(&format!("{t} = call i8 @cl_instance_of(ptr {o}, ptr {td})"));
                Val::Imm(Repr::Bool, t)
            }
            TKind::IsNull(inner, is_eq) => {
                let from = self.repr(&inner.ty);
                let v = self.expr(inner);
                if from != Repr::Ref {
                    return Val::Imm(Repr::Bool, if *is_eq { "0".into() } else { "1".into() });
                }
                let o = self.val(&v);
                let c = self.tmp();
                self.ins(&format!("{c} = icmp {} ptr {o}, null", if *is_eq { "eq" } else { "ne" }));
                Val::Imm(Repr::Bool, self.bool_of(&c))
            }
            TKind::Coerce(inner) => {
                let v = self.expr(inner);
                self.convert(v, self.repr(&inner.ty), r)
            }
            TKind::Prim(op, a, b) => {
                let m = self.repr(&a.ty);
                let x = self.expr(a);
                let y = self.expr(b);
                let (x, y) = (self.val(&x), self.val(&y));
                let eq = self.tmp();
                if matches!(m, Repr::F32 | Repr::F64) {
                    // Equal as `equals` has it: NaN equals NaN, and the two zeros are equal.
                    let (o, na, nb, both) = (self.tmp(), self.tmp(), self.tmp(), self.tmp());
                    self.ins(&format!("{o} = fcmp oeq {} {x}, {y}", m.ll()));
                    self.ins(&format!("{na} = fcmp uno {} {x}, {x}", m.ll()));
                    self.ins(&format!("{nb} = fcmp uno {} {y}, {y}", m.ll()));
                    self.ins(&format!("{both} = and i1 {na}, {nb}"));
                    self.ins(&format!("{eq} = or i1 {o}, {both}"));
                } else {
                    self.ins(&format!("{eq} = icmp eq {} {x}, {y}", m.ll()));
                }
                let c = if *op == BinOp::Ne {
                    let n = self.tmp();
                    self.ins(&format!("{n} = xor i1 {eq}, true"));
                    n
                } else {
                    eq
                };
                Val::Imm(Repr::Bool, self.bool_of(&c))
            }
            TKind::Let(id, init, body) => {
                let v = self.expr(init);
                let s = self.locals[*id as usize].clone();
                self.write(&s, &v);
                self.expr(body)
            }
            TKind::Seq(list, last) => {
                for x in list {
                    self.expr(x);
                }
                self.expr(last)
            }
            TKind::Switch { selector, temp, arms } => {
                let out = self.new_store(r);
                let end = self.new_label("switch.end");
                let sel = self.expr(selector);
                let s = self.locals[*temp as usize].clone();
                self.write(&s, &sel);
                let sty = selector.ty.clone();
                for (test, value) in arms {
                    let next = self.new_label("arm.next");
                    self.arm_test(test, &s, &sty, &next);
                    let v = self.expr(value);
                    let v = self.convert(v, self.repr(&value.ty), r);
                    self.write(&out, &v);
                    self.br(&end);
                    self.label(&next);
                }
                // The checker has shown that some arm matches.
                self.ins("unreachable");
                self.open = false;
                self.label(&end);
                self.read(&out)
            }
            TKind::Throw(x) => {
                let v = self.expr(x);
                let v = self.val(&v);
                self.ins(&format!("store ptr {v}, ptr %ctx"));
                let h = self.handler();
                self.br(&h);
                if r == Repr::Unit {
                    Val::Unit
                } else {
                    Val::Imm(r, r.zero().to_string())
                }
            }
        }
    }

    fn load_field(&mut self, o: &str, owner: ClassId, f: FieldRef, want: Repr) -> Val {
        let decl = self.repr(&self.e.p.field(f).ty);
        if decl == Repr::Unit {
            return Val::Unit;
        }
        let (lty, index, fr) = self.e.field_place(owner, f);
        let p = self.tmp();
        self.ins(&format!("{p} = getelementptr inbounds {lty}, ptr {o}, i32 0, i32 {index}"));
        let t = self.tmp();
        self.ins(&format!("{t} = load {}, ptr {p}", fr.ll()));
        let v = if fr == Repr::Ref { self.root(&t) } else { Val::Imm(fr, t) };
        self.convert(v, fr, want)
    }

    fn assigned_check(&mut self, f: FieldRef) {
        let c = self.e.p.class(f.class);
        let (q, name) = (sym(&c.qname), c.fields[f.index as usize].name.clone());
        let rec = self.e.init_rec(f.class);
        let s = self.tmp();
        self.ins(&format!("{s} = load i32, ptr {rec}"));
        let d = self.tmp();
        self.ins(&format!("{d} = icmp eq i32 {s}, 2"));
        let (fast, chk, bad) = (self.new_label("st.ok"), self.new_label("st.chk"), self.new_label("st.bad"));
        self.cond_br(&d, &fast, &chk);
        self.label(&chk);
        let b = self.tmp();
        self.ins(&format!("{b} = load i8, ptr @\"set.{q}.{}\"", sym(&name)));
        let z = self.truth(&b);
        self.cond_br(&z, &fast, &bad);
        self.label(&bad);
        let (bytes, len) = self.e.byte_string(&format!("{}.{name}", c.qname));
        self.ins(&format!("call void @cl_static_unassigned(ptr %ctx, ptr {bytes}, i64 {len})"));
        let h = self.handler();
        self.br(&h);
        self.label(&fast);
    }

    /// The default value `new T[n]` gives an element whose class is not `@Nullable`
    /// and is held as a pointer: `Rational` zero, or a value class built from defaults.
    fn default_value(&mut self, t: &Type, depth: u32) -> Option<Val> {
        let Ty::Class(id, args) = &t.ty else { return None };
        if *id == self.e.p.wk.rational {
            let f = "@cl_Rational_zero".to_string();
            self.e.declare("declare ptr @cl_Rational_zero(ptr)".into());
            let v = self.tmp();
            self.ins(&format!("{v} = call ptr {f}(ptr %ctx)"));
            self.check();
            return Some(self.root(&v));
        }
        let c = self.e.p.class(*id).clone();
        if c.kind != TypeKind::ValueClass || depth > 8 {
            return None;
        }
        let td = self.type_desc(&t.bare());
        let o = self.tmp();
        self.ins(&format!("{o} = call ptr @cl_new(ptr %ctx, ptr {td})"));
        self.check();
        let obj = self.root(&o);
        let s = Subst::for_class(*id, args);
        for (i, f) in c.fields.iter().enumerate() {
            if f.is_static || f.ty.nullable || self.repr(&f.ty) != Repr::Ref {
                continue;
            }
            let ft = s.apply(&f.ty);
            if let Some(d) = self.default_value(&ft, depth + 1) {
                let (o, d) = (self.val(&obj), self.val(&d));
                let (lty, index, _) = self.e.field_place(*id, FieldRef { class: *id, index: i as u32 });
                let p = self.tmp();
                self.ins(&format!("{p} = getelementptr inbounds {lty}, ptr {o}, i32 0, i32 {index}"));
                self.ins(&format!("store ptr {d}, ptr {p}"));
            }
        }
        Some(obj)
    }

    /// The descriptor of a lambda's class: the class applied to the type variables in
    /// scope where the lambda is written.
    fn lambda_type(&mut self, class: ClassId, ty: &Type, lambda_env: &[Tv]) -> String {
        if lambda_env.is_empty() {
            return self.type_desc(&Type::simple(class));
        }
        self.type_desc(ty)
    }

    fn elem_addr(&mut self, arr: &str, index: &str, r: Repr) -> String {
        let base = self.tmp();
        self.ins(&format!("{base} = getelementptr inbounds i8, ptr {arr}, i64 24"));
        let p = self.tmp();
        self.ins(&format!("{p} = getelementptr inbounds {}, ptr {base}, i64 {index}", r.ll()));
        p
    }

    fn bounds_check(&mut self, arr: &str, index: &str) {
        let lp = self.tmp();
        self.ins(&format!("{lp} = getelementptr inbounds i8, ptr {arr}, i64 16"));
        let n = self.tmp();
        self.ins(&format!("{n} = load i64, ptr {lp}"));
        let c = self.tmp();
        self.ins(&format!("{c} = icmp ult i64 {index}, {n}"));
        let (ok, bad) = (self.new_label("in"), self.new_label("out"));
        self.cond_br(&c, &ok, &bad);
        self.label(&bad);
        self.ins(&format!("call void @cl_index_fail(ptr %ctx, i64 {index}, i64 {n})"));
        let h = self.handler();
        self.br(&h);
        self.label(&ok);
    }

    /// The call instruction and what follows it: the exception test and the result.
    fn finish_call(&mut self, callee: &str, sig: &Sig, args: Vec<String>, want: Repr) -> Val {
        if sig.foreign {
            self.ins("call void @cl_foreign_enter(ptr %ctx)");
        }
        let out = if sig.ret == Repr::Unit {
            self.ins(&format!("call void {callee}({})", args.join(", ")));
            Val::Unit
        } else {
            let t = self.tmp();
            self.ins(&format!("{t} = call {} {callee}({})", sig.ret.ll(), args.join(", ")));
            Val::Imm(sig.ret, t)
        };
        if sig.foreign {
            self.ins("call void @cl_foreign_leave(ptr %ctx)");
        } else {
            self.check();
        }
        let out = match out {
            Val::Imm(Repr::Ref, t) => self.root(&t),
            other => other,
        };
        self.convert(out, sig.ret, want)
    }

    fn call(&mut self, c: &Call, node_ty: &Type) -> Val {
        let want = self.repr(node_ty);
        let p = self.e.p;
        let md = p.method(c.method);
        // Arrays whose element type is known are read and written in place.
        if md.intrinsic && c.method.class == p.wk.array && c.recv.is_some() {
            let elem = c.class_args.first().and_then(|a| a.as_type()).cloned();
            let known = elem.as_ref().map(|t| t.nullable || !matches!(t.ty, Ty::Var(_))).unwrap_or(false);
            if md.name == "length" || known {
                let er = elem.as_ref().map(|t| self.repr(t)).unwrap_or(Repr::Ref);
                let a = self.expr(c.recv.as_ref().unwrap());
                match md.name.as_str() {
                    "length" => {
                        let a = self.val(&a);
                        let lp = self.tmp();
                        self.ins(&format!("{lp} = getelementptr inbounds i8, ptr {a}, i64 16"));
                        let n = self.tmp();
                        self.ins(&format!("{n} = load i64, ptr {lp}"));
                        return self.convert(Val::Imm(Repr::I64, n), Repr::I64, want);
                    }
                    "get" => {
                        let i = self.expr(&c.args[0]);
                        let (a, i) = (self.val(&a), self.val(&i));
                        self.bounds_check(&a, &i);
                        if er == Repr::Unit {
                            return Val::Unit;
                        }
                        let ep = self.elem_addr(&a, &i, er);
                        let t = self.tmp();
                        self.ins(&format!("{t} = load {}, ptr {ep}", er.ll()));
                        let v = if er == Repr::Ref { self.root(&t) } else { Val::Imm(er, t) };
                        return self.convert(v, er, want);
                    }
                    "set" => {
                        let i = self.expr(&c.args[0]);
                        let v = self.expr(&c.args[1]);
                        let v = self.convert(v, self.repr(&c.args[1].ty), er);
                        let (a, i, v) = (self.val(&a), self.val(&i), self.val(&v));
                        self.bounds_check(&a, &i);
                        if er != Repr::Unit {
                            let ep = self.elem_addr(&a, &i, er);
                            self.ins(&format!("store {} {v}, ptr {ep}", er.ll()));
                        }
                        return Val::Unit;
                    }
                    _ => {}
                }
            }
        }
        if md.foreign {
            if let Some(shim) = self.e.foreign_shim(c.method) {
                return self.shim_call(c, &shim, want);
            }
        }
        let target = if c.dispatch == Dispatch::Virtual { self.e.root_of(c.method) } else { c.method };
        let sig = self.e.sig(target);
        let recv = match (&c.recv, sig.recv) {
            (Some(r), Some(want)) => {
                let v = self.expr(r);
                Some((self.convert(v, self.repr(&r.ty), want), want))
            }
            (Some(r), None) => {
                // A static method named through a value: the value is still evaluated.
                self.expr(r);
                None
            }
            _ => None,
        };
        let mut vals = Vec::new();
        for (a, pr) in c.args.iter().zip(sig.params.iter()) {
            let v = self.expr(a);
            let from = self.repr(&a.ty);
            let v = if sig.foreign && *pr == Repr::Ptr && from == Repr::Ref {
                // A foreign function receives the address of an array's first element.
                let o = self.val(&v);
                let p = self.tmp();
                self.ins(&format!("{p} = getelementptr inbounds i8, ptr {o}, i64 24"));
                Val::Imm(Repr::Ptr, p)
            } else {
                self.convert(v, from, *pr)
            };
            vals.push((v, *pr));
        }
        let mut all: Vec<String> = Vec::new();
        if !sig.foreign {
            all.push("ptr %ctx".into());
        }
        if sig.generic {
            let n = c.targs.len();
            self.ntmp += 1;
            let arr = format!("%v{}", self.ntmp);
            self.allocas.push_str(&format!("  {arr} = alloca [{n} x ptr]\n"));
            for (i, t) in c.targs.iter().enumerate() {
                let td = match self.capture_source(t, c, &vals) {
                    Some(td) => td,
                    None => self.type_desc(t),
                };
                let p = self.tmp();
                self.ins(&format!("{p} = getelementptr inbounds ptr, ptr {arr}, i64 {i}"));
                self.ins(&format!("store ptr {td}, ptr {p}"));
            }
            all.push(format!("ptr {arr}"));
        }
        let callee = if c.dispatch == Dispatch::Virtual {
            let (rv, _) = recv.as_ref().expect("a virtual call has a receiver");
            let o = self.val(rv);
            let f = self.tmp();
            self.ins(&format!("{f} = call ptr @cl_lookup(ptr {o}, i32 {})", self.e.selector(target)));
            f
        } else {
            self.e.fn_symbol(target)
        };
        if let Some((rv, rr)) = &recv {
            let o = self.val(rv);
            all.push(format!("{} {o}", rr.ll()));
        }
        for (v, pr) in &vals {
            let x = self.val(v);
            all.push(format!("{} {x}", pr.ll()));
        }
        if c.recv.is_none() && !md.is_static {
            self.e.errors.push(format!("internal: the instance method `{}` is called without a receiver", md.name));
        }
        self.finish_call(&callee, &sig, all, want)
    }

    /// A foreign call that passes a struct: through the C shim, which copies each value
    /// class into the struct C expects and back (section 4.11).
    fn shim_call(&mut self, c: &Call, shim: &Shim, want: Repr) -> Val {
        enum Piece {
            Value(&'static str, Val),
            Elements(Val),
            Length(Val),
        }
        let ret_ty = self.e.p.method(c.method).ret.clone();
        let mut pieces = Vec::new();
        for (a, kind) in c.args.iter().zip(shim.params.iter()) {
            let v = self.expr(a);
            match kind {
                ShimParam::Scalar(r) => {
                    let v = self.convert(v, self.repr(&a.ty), *r);
                    pieces.push(Piece::Value(r.ll(), v));
                }
                ShimParam::Struct => pieces.push(Piece::Value("ptr", v)),
                ShimParam::Array => pieces.push(Piece::Elements(v)),
                ShimParam::StructArray => {
                    // The function writes into fresh values, which then take the
                    // elements' places.
                    let o = self.val(&v);
                    let f = self.tmp();
                    self.ins(&format!("{f} = call ptr @cl_ffi_fresh(ptr %ctx, ptr {o})"));
                    self.check();
                    let fresh = self.root(&f);
                    pieces.push(Piece::Elements(v.clone()));
                    pieces.push(Piece::Elements(fresh));
                    pieces.push(Piece::Length(v));
                }
            }
        }
        let result = if shim.ret_struct { self.default_value(&ret_ty, 0) } else { None };
        let mut all: Vec<String> = Vec::new();
        if let Some(r) = &result {
            let o = self.val(r);
            all.push(format!("ptr {o}"));
        }
        for p in &pieces {
            match p {
                Piece::Value(ty, v) => {
                    let x = self.val(v);
                    all.push(format!("{ty} {x}"));
                }
                Piece::Elements(v) => {
                    let o = self.val(v);
                    let e = self.tmp();
                    self.ins(&format!("{e} = getelementptr inbounds i8, ptr {o}, i64 24"));
                    all.push(format!("ptr {e}"));
                }
                Piece::Length(v) => {
                    let o = self.val(v);
                    let lp = self.tmp();
                    self.ins(&format!("{lp} = getelementptr inbounds i8, ptr {o}, i64 16"));
                    let n = self.tmp();
                    self.ins(&format!("{n} = load i64, ptr {lp}"));
                    all.push(format!("i64 {n}"));
                }
            }
        }
        self.ins("call void @cl_foreign_enter(ptr %ctx)");
        let scalar = if shim.ret_struct || shim.ret == Repr::Unit {
            self.ins(&format!("call void {}({})", shim.symbol, all.join(", ")));
            None
        } else {
            let t = self.tmp();
            self.ins(&format!("{t} = call {} {}({})", shim.ret.ll(), shim.symbol, all.join(", ")));
            Some(t)
        };
        self.ins("call void @cl_foreign_leave(ptr %ctx)");
        match (result, scalar) {
            (Some(o), _) => self.convert(o, Repr::Ref, want),
            (None, Some(t)) => self.convert(Val::Imm(shim.ret, t), shim.ret, want),
            _ => Val::Unit,
        }
    }

    /// A type argument that is the unknown type of a wildcard: the type argument of the
    /// object that was passed (section 7.4).
    fn capture_source(&mut self, t: &Type, c: &Call, vals: &[(Val, Repr)]) -> Option<String> {
        let Ty::Var(tv) = &t.ty else { return None };
        if !matches!(tv.owner, TvOwner::Capture(_)) {
            return None;
        }
        for (i, a) in c.args.iter().enumerate() {
            let Ty::Class(g, args) = &a.ty.ty else { continue };
            for (j, arg) in args.iter().enumerate() {
                if matches!(arg, Arg::Ty(at) if at.ty == Ty::Var(*tv)) {
                    let o = self.val(&vals[i].0);
                    let ci = self.e.class_info(*g);
                    let x = self.tmp();
                    self.ins(&format!("{x} = call ptr @cl_type_arg(ptr {o}, ptr {ci}, i32 {j})"));
                    return Some(x);
                }
            }
        }
        None
    }

    /// Branches to `next` unless the selector meets the arm's test.
    fn arm_test(&mut self, test: &ArmTest, sel: &Store, sty: &Type, next: &str) {
        let sr = self.repr(sty);
        match test {
            ArmTest::Default => {}
            ArmTest::Constants(cs) => {
                let hit = self.new_label("arm");
                for k in cs {
                    let s = self.read(sel);
                    let kv = self.expr(k);
                    let c = if sr.is_machine() {
                        let (a, b) = (self.val(&s), self.val(&kv));
                        let t = self.tmp();
                        self.ins(&format!("{t} = icmp eq {} {a}, {b}", sr.ll()));
                        t
                    } else if sty.class_id().map(|id| self.e.p.class(id).is_enum()).unwrap_or(false) {
                        let (a, b) = (self.val(&s), self.val(&kv));
                        let t = self.tmp();
                        self.ins(&format!("{t} = icmp eq ptr {a}, {b}"));
                        t
                    } else {
                        let (a, b) = (self.val(&s), self.val(&kv));
                        let t = self.tmp();
                        self.ins(&format!("{t} = call i8 @cl_Object_equals_Object(ptr %ctx, ptr {a}, ptr {b})"));
                        self.truth(&t)
                    };
                    let more = self.new_label("arm.more");
                    self.cond_br(&c, &hit, &more);
                    self.label(&more);
                }
                self.br(next);
                self.label(&hit);
            }
            ArmTest::Type(ty, local) => {
                let s = self.read(sel);
                let s = self.convert(s, sr, Repr::Ref);
                let td = self.type_desc(ty);
                let o = self.val(&s);
                let t = self.tmp();
                self.ins(&format!("{t} = call i8 @cl_instance_of(ptr {o}, ptr {td})"));
                let c = self.truth(&t);
                let hit = self.new_label("arm");
                self.cond_br(&c, &hit, next);
                self.label(&hit);
                let v = self.convert(s, Repr::Ref, self.repr(ty));
                let st = self.locals[*local as usize].clone();
                self.write(&st, &v);
            }
        }
    }

    // ---- statements ----

    fn stmts(&mut self, list: &[TStmt]) {
        for s in list {
            let mark = self.slots_next;
            self.stmt(s);
            self.slots_next = mark;
        }
    }

    /// Stops for the collector when it has asked.
    fn poll(&mut self) {
        let p = self.tmp();
        self.ins(&format!("{p} = load i32, ptr %poll"));
        let z = self.tmp();
        self.ins(&format!("{z} = icmp eq i32 {p}, 0"));
        let (go, slow) = (self.new_label("go"), self.new_label("poll"));
        self.cond_br(&z, &go, &slow);
        self.label(&slow);
        self.ins("call void @cl_poll(ptr %ctx)");
        self.label(&go);
    }

    fn open_scope(&mut self) -> (String, Store, String) {
        self.ntmp += 1;
        let sel = format!("%v{}", self.ntmp);
        self.allocas.push_str(&format!("  {sel} = alloca i32\n"));
        let saved = Store::Slot(self.new_slot());
        self.write(&saved, &Val::Imm(Repr::Ref, "null".into()));
        let label = self.new_label("finally");
        self.scopes.push(Scope { label: label.clone(), sel: sel.clone(), targets: Vec::new() });
        (sel, saved, label)
    }

    /// The block a region's uncaught exception reaches: it keeps the exception and
    /// enters the `finally` code.
    fn scope_catch(&mut self, raised: &str, sel: &str, saved: &Store, label: &str) {
        self.label(raised);
        let e = self.tmp();
        self.ins(&format!("{e} = load ptr, ptr %ctx"));
        self.ins("store ptr null, ptr %ctx");
        self.write(saved, &Val::Imm(Repr::Ref, e));
        self.ins(&format!("store i32 1, ptr {sel}"));
        self.br(label);
    }

    /// After the `finally` code: continue in the way the region was left.
    fn scope_dispatch(&mut self, scope: Scope, saved: &Store, after: &str) {
        let s = self.tmp();
        self.ins(&format!("{s} = load i32, ptr {}", scope.sel));
        let rethrow = self.new_label("rethrow");
        let labels: Vec<String> = scope.targets.iter().map(|_| self.new_label("leave")).collect();
        let mut cases = format!("i32 1, label %{rethrow}");
        for (i, l) in labels.iter().enumerate() {
            cases.push_str(&format!(" i32 {}, label %{l}", i + 2));
        }
        self.ins(&format!("switch i32 {s}, label %{after} [ {cases} ]"));
        self.open = false;
        self.label(&rethrow);
        let x = self.read(saved);
        let x = self.val(&x);
        self.ins(&format!("store ptr {x}, ptr %ctx"));
        let h = self.handler();
        self.br(&h);
        for (t, l) in scope.targets.into_iter().zip(labels) {
            self.label(&l);
            self.jump(t);
        }
    }

    fn suppress(&mut self, on: &str, other: &str) {
        self.ins(&format!("call void @cleat_suppress(ptr %ctx, ptr {on}, ptr {other})"));
    }

    fn stmt(&mut self, s: &TStmt) {
        match s {
            TStmt::Expr(e) => {
                self.expr(e);
            }
            TStmt::Local(id, init) => {
                if let Some(e) = init {
                    let v = self.expr(e);
                    let st = self.locals[*id as usize].clone();
                    let lr = match &st {
                        Store::Slot(_) => Repr::Ref,
                        Store::Alloca(_, m) => *m,
                        Store::None => Repr::Unit,
                    };
                    let v = self.convert(v, self.repr(&e.ty), lr);
                    self.write(&st, &v);
                }
            }
            TStmt::Block(b) => self.stmts(b),
            TStmt::If(c, a, b) => {
                let k = self.expr(c);
                let k = self.val(&k);
                let k = self.truth(&k);
                let (yes, no, done) = (self.new_label("if"), self.new_label("else"), self.new_label("endif"));
                self.cond_br(&k, &yes, if b.is_empty() { &done } else { &no });
                self.label(&yes);
                self.stmts(a);
                self.br(&done);
                if !b.is_empty() {
                    self.label(&no);
                    self.stmts(b);
                }
                self.label(&done);
            }
            TStmt::Loop { label, cond, update, body } => {
                let (head, top, cont, exit) = (self.new_label("loop"), self.new_label("body"), self.new_label("next"), self.new_label("done"));
                self.jumps.insert(*label, Jump { brk: exit.clone(), cont: Some(cont.clone()), depth: self.scopes.len() });
                self.label(&head);
                self.poll();
                let mark = self.slots_next;
                if let Some(c) = cond {
                    let k = self.expr(c);
                    let k = self.val(&k);
                    let k = self.truth(&k);
                    self.cond_br(&k, &top, &exit);
                }
                self.slots_next = mark;
                self.label(&top);
                self.stmts(body);
                self.label(&cont);
                for u in update {
                    let mark = self.slots_next;
                    self.expr(u);
                    self.slots_next = mark;
                }
                self.br(&head);
                self.label(&exit);
            }
            TStmt::Labeled { label, body } => {
                let end = self.new_label("labeled.end");
                self.jumps.insert(*label, Jump { brk: end.clone(), cont: None, depth: self.scopes.len() });
                self.stmts(body);
                self.label(&end);
            }
            TStmt::Switch { label, selector, temp, arms } => {
                let end = self.new_label("switch.end");
                self.jumps.insert(*label, Jump { brk: end.clone(), cont: None, depth: self.scopes.len() });
                let sel = self.expr(selector);
                let st = self.locals[*temp as usize].clone();
                self.write(&st, &sel);
                let sty = selector.ty.clone();
                for (test, body) in arms {
                    let next = self.new_label("arm.next");
                    self.arm_test(test, &st, &sty, &next);
                    self.stmts(body);
                    self.br(&end);
                    self.label(&next);
                }
                self.label(&end);
            }
            TStmt::Try { body, catches, finally } => {
                let after = self.new_label("try.end");
                let scope = finally.as_ref().map(|_| self.open_scope());
                let raised = self.new_label("try.raised");
                let done = |f: &mut Self| match &scope {
                    Some((sel, _, label)) => {
                        f.ins(&format!("store i32 0, ptr {sel}"));
                        f.br(label);
                    }
                    None => f.br(&after),
                };
                let catch = self.new_label("catch");
                self.handlers.push(if catches.is_empty() { raised.clone() } else { catch.clone() });
                self.stmts(body);
                self.handlers.pop();
                done(self);
                if !catches.is_empty() {
                    self.label(&catch);
                    if scope.is_some() {
                        self.handlers.push(raised.clone());
                    }
                    let e = self.tmp();
                    self.ins(&format!("{e} = load ptr, ptr %ctx"));
                    self.ins("store ptr null, ptr %ctx");
                    let held = self.root(&e);
                    for c in catches {
                        let td = self.type_desc(&c.ty);
                        let o = self.val(&held);
                        let t = self.tmp();
                        self.ins(&format!("{t} = call i8 @cl_instance_of(ptr {o}, ptr {td})"));
                        let k = self.truth(&t);
                        let (hit, next) = (self.new_label("caught"), self.new_label("catch.next"));
                        self.cond_br(&k, &hit, &next);
                        self.label(&hit);
                        let st = self.locals[c.local as usize].clone();
                        self.write(&st, &held);
                        self.stmts(&c.body);
                        done(self);
                        self.label(&next);
                    }
                    // No clause accepts it: it goes on propagating.
                    let o = self.val(&held);
                    self.ins(&format!("store ptr {o}, ptr %ctx"));
                    if scope.is_some() {
                        self.handlers.pop();
                        self.br(&raised);
                    } else {
                        let h = self.handler();
                        self.br(&h);
                    }
                }
                if let (Some((sel, saved, label)), Some(fin)) = (scope, finally) {
                    self.scope_catch(&raised, &sel, &saved, &label);
                    let sc = self.scopes.pop().unwrap();
                    self.label(&label);
                    let fin_raised = self.new_label("finally.raised");
                    self.handlers.push(fin_raised.clone());
                    self.stmts(fin);
                    self.handlers.pop();
                    self.scope_dispatch(sc, &saved, &after);
                    // The `finally` block raised: an exception it interrupted is recorded on the new one.
                    self.label(&fin_raised);
                    let new = self.tmp();
                    self.ins(&format!("{new} = load ptr, ptr %ctx"));
                    let new = self.root(&new);
                    let old = self.read(&saved);
                    let o = self.val(&old);
                    let c = self.tmp();
                    self.ins(&format!("{c} = icmp ne ptr {o}, null"));
                    let (rec, go) = (self.new_label("suppress"), self.new_label("finally.out"));
                    self.cond_br(&c, &rec, &go);
                    self.label(&rec);
                    self.ins("store ptr null, ptr %ctx");
                    let (n, o) = (self.val(&new), self.val(&old));
                    self.suppress(&n, &o);
                    let n = self.val(&new);
                    self.ins(&format!("store ptr {n}, ptr %ctx"));
                    self.label(&go);
                    let h = self.handler();
                    self.br(&h);
                }
                self.label(&after);
            }
            TStmt::Using { local, init, close, body } => {
                let v = self.expr(init);
                let st = self.locals[*local as usize].clone();
                self.write(&st, &v);
                let after = self.new_label("using.end");
                let (sel, saved, label) = self.open_scope();
                let raised = self.new_label("using.raised");
                self.handlers.push(raised.clone());
                self.stmts(body);
                self.handlers.pop();
                self.ins(&format!("store i32 0, ptr {sel}"));
                self.br(&label);
                self.scope_catch(&raised, &sel, &saved, &label);
                let sc = self.scopes.pop().unwrap();
                self.label(&label);
                let close_raised = self.new_label("close.raised");
                self.handlers.push(close_raised.clone());
                self.expr(close);
                self.handlers.pop();
                self.scope_dispatch(sc, &saved, &after);
                // `close` raised. The block's own exception wins, with this one recorded on it.
                self.label(&close_raised);
                let new = self.tmp();
                self.ins(&format!("{new} = load ptr, ptr %ctx"));
                let new = self.root(&new);
                let old = self.read(&saved);
                let o = self.val(&old);
                let c = self.tmp();
                self.ins(&format!("{c} = icmp ne ptr {o}, null"));
                let (rec, go) = (self.new_label("suppress"), self.new_label("close.out"));
                self.cond_br(&c, &rec, &go);
                self.label(&rec);
                self.ins("store ptr null, ptr %ctx");
                let (n, o) = (self.val(&new), self.val(&old));
                self.suppress(&o, &n);
                let o = self.val(&old);
                self.ins(&format!("store ptr {o}, ptr %ctx"));
                self.label(&go);
                let h = self.handler();
                self.br(&h);
                self.label(&after);
            }
            TStmt::Return(v) => {
                if let Some(e) = v {
                    let x = self.expr(e);
                    let x = self.convert(x, self.repr(&e.ty), self.ret);
                    let st = self.ret_store.clone();
                    self.write(&st, &x);
                }
                self.jump(Target::Return);
            }
            TStmt::Break(l) => self.jump(Target::Break(*l)),
            TStmt::Continue(l) => self.jump(Target::Continue(*l)),
            TStmt::Throw(e) => {
                let v = self.expr(e);
                let v = self.val(&v);
                self.ins(&format!("store ptr {v}, ptr %ctx"));
                let h = self.handler();
                self.br(&h);
            }
            TStmt::CtorCall { ctor, args } => {
                let sig = self.e.ctor_sig(ctor.class, ctor.index);
                let mut vals = Vec::new();
                for (a, pr) in args.iter().zip(sig.params.iter()) {
                    let v = self.expr(a);
                    let v = self.convert(v, self.repr(&a.ty), *pr);
                    vals.push((v, *pr));
                }
                let this = self.this.clone();
                let this = self.read(&this);
                let this = self.val(&this);
                let mut all = vec!["ptr %ctx".to_string(), format!("ptr {this}")];
                for (v, pr) in &vals {
                    let x = self.val(v);
                    all.push(format!("{} {x}", pr.ll()));
                }
                let f = self.e.ctor_symbol(ctor.class, ctor.index);
                // The constructor the runtime supplies for a class with no state does nothing.
                if self.e.p.class(ctor.class).ctors[ctor.index as usize].intrinsic {
                    return;
                }
                self.ins(&format!("call void {f}({})", all.join(", ")));
                self.check();
            }
        }
    }

    // ---- whole functions ----

    /// Gives every local of a body its storage.
    fn allot(&mut self, body: &Body) {
        for l in &body.locals {
            let r = self.repr(&l.ty);
            let s = self.new_store(r);
            self.locals.push(s);
        }
    }

    /// Puts the pieces of a function together. `params` are the parameters after the
    /// context, each with the storage its value goes to.
    fn assemble(&mut self, linkage: &str, name: &str, ret: Repr, generic: bool, params: &[(Repr, Store)], init: Option<ClassId>) -> String {
        let mut text = String::new();
        let mut plist = vec!["ptr %ctx".to_string()];
        if generic {
            plist.push("ptr %targs".into());
        }
        for (i, (r, _)) in params.iter().enumerate() {
            plist.push(format!("{} %p{i}", r.ll()));
        }
        let _ = writeln!(text, "define {linkage}{} {name}({}) {{", ret.ll(), plist.join(", "));
        text.push_str("entry:\n");
        text.push_str(&self.allocas);
        let n = self.slots_max;
        if n > 0 {
            let _ = writeln!(text, "  %frame = alloca {{ ptr, i64, [{n} x ptr] }}");
        }
        if let (true, EnvSrc::Both(_, nc, nm)) = (self.uses_env, self.env_src) {
            let _ = writeln!(text, "  %env = alloca [{} x ptr]", nc + nm);
        }
        // A call that finds the stack full ends the program (section 9.10).
        text.push_str("  %sp = call ptr @llvm.frameaddress.p0(i32 0)\n");
        text.push_str("  %sp.limit.at = getelementptr inbounds i8, ptr %ctx, i64 32\n");
        text.push_str("  %sp.limit = load ptr, ptr %sp.limit.at\n");
        text.push_str("  %sp.full = icmp ult ptr %sp, %sp.limit\n");
        text.push_str("  br i1 %sp.full, label %stack.full, label %setup\n");
        text.push_str("stack.full:\n  call void @cl_stack_overflow()\n  unreachable\n");
        text.push_str("setup:\n");
        text.push_str("  %poll = getelementptr inbounds i8, ptr %ctx, i64 16\n");
        if n > 0 {
            let _ = writeln!(text, "  call void @llvm.memset.p0.i64(ptr %frame, i8 0, i64 {}, i1 false)", 16 + 8 * n as u64);
            text.push_str("  %top = getelementptr inbounds i8, ptr %ctx, i64 8\n");
            text.push_str("  %prev = load ptr, ptr %top\n");
            text.push_str("  store ptr %prev, ptr %frame\n");
            text.push_str("  %frame.n = getelementptr inbounds i8, ptr %frame, i64 8\n");
            let _ = writeln!(text, "  store i64 {n}, ptr %frame.n");
            text.push_str("  %slots = getelementptr inbounds i8, ptr %frame, i64 16\n");
            text.push_str("  store ptr %frame, ptr %top\n");
        }
        for (i, (r, s)) in params.iter().enumerate() {
            match s {
                Store::Slot(k) => {
                    let _ = writeln!(text, "  %ps{i} = getelementptr inbounds ptr, ptr %slots, i64 {k}");
                    let _ = writeln!(text, "  store ptr %p{i}, ptr %ps{i}");
                }
                Store::Alloca(a, _) => {
                    let _ = writeln!(text, "  store {} %p{i}, ptr {a}", r.ll());
                }
                Store::None => {}
            }
        }
        if self.uses_env {
            match self.env_src {
                EnvSrc::None => text.push_str("  %env = inttoptr i64 0 to ptr\n"),
                EnvSrc::Targs => text.push_str("  %env = getelementptr inbounds i8, ptr %targs, i64 0\n"),
                EnvSrc::This(c) => {
                    let _ = writeln!(text, "  %env = call ptr @cl_env(ptr %p0, ptr {})", self.e.class_info(c));
                }
                EnvSrc::Both(c, nc, nm) => {
                    let _ = writeln!(text, "  %cenv = call ptr @cl_env(ptr %p0, ptr {})", self.e.class_info(c));
                    for i in 0..nc + nm {
                        let (src, j) = if i < nc { ("%cenv", i) } else { ("%targs", i - nc) };
                        let _ = writeln!(text, "  %e.s{i} = getelementptr inbounds ptr, ptr {src}, i64 {j}");
                        let _ = writeln!(text, "  %e.v{i} = load ptr, ptr %e.s{i}");
                        let _ = writeln!(text, "  %e.d{i} = getelementptr inbounds ptr, ptr %env, i64 {i}");
                        let _ = writeln!(text, "  store ptr %e.v{i}, ptr %e.d{i}");
                    }
                }
            }
        }
        // A static method or a constructor is a first use of its class.
        if let Some(c) = init {
            if !self.e.is_trivial(c) {
                let rec = self.e.init_rec(c);
                let _ = writeln!(text, "  %init.s = load i32, ptr {rec}");
                text.push_str("  %init.c = icmp eq i32 %init.s, 2\n");
                text.push_str("  br i1 %init.c, label %body, label %init.run\n");
                text.push_str("init.run:\n");
                let _ = writeln!(text, "  call void @cl_class_init(ptr %ctx, ptr {rec})");
                text.push_str("  %init.e = load ptr, ptr %ctx\n");
                text.push_str("  %init.ok = icmp eq ptr %init.e, null\n");
                text.push_str("  br i1 %init.ok, label %body, label %unwind\n");
            } else {
                text.push_str("  br label %body\n");
            }
        } else {
            text.push_str("  br label %body\n");
        }
        text.push_str("body:\n");
        text.push_str(&self.body);
        let pop = if n > 0 { "  store ptr %prev, ptr %top\n" } else { "" };
        text.push_str("ret:\n");
        match &self.ret_store {
            Store::Slot(k) => {
                let _ = writeln!(text, "  %ret.s = getelementptr inbounds ptr, ptr %slots, i64 {k}");
                text.push_str("  %ret.v = load ptr, ptr %ret.s\n");
                text.push_str(pop);
                text.push_str("  ret ptr %ret.v\n");
            }
            Store::Alloca(a, r) => {
                let _ = writeln!(text, "  %ret.v = load {}, ptr {a}", r.ll());
                text.push_str(pop);
                let _ = writeln!(text, "  ret {} %ret.v", r.ll());
            }
            Store::None => {
                text.push_str(pop);
                text.push_str("  ret void\n");
            }
        }
        text.push_str("unwind:\n");
        text.push_str(pop);
        if ret == Repr::Unit {
            text.push_str("  ret void\n");
        } else {
            let _ = writeln!(text, "  ret {} {}", ret.ll(), ret.zero());
        }
        text.push_str("}\n");
        text
    }

    /// Ends a body: a `void` function returns, and any other cannot reach here.
    fn close(&mut self) {
        if self.open {
            if self.ret == Repr::Unit {
                self.br("ret");
            } else {
                self.ins("unreachable");
                self.open = false;
            }
        }
    }

    fn set_env(&mut self, class: ClassId, method: Option<MethodRef>, is_static: bool) {
        let c = self.e.p.class(class);
        if let Some(l) = &c.lambda {
            self.env = l.env.clone();
            self.env_src = if self.env.is_empty() { EnvSrc::None } else { EnvSrc::This(class) };
            return;
        }
        let nc = if is_static { 0 } else { c.tparams.len() };
        let nm = method.map(|m| self.e.p.method(m).tparams.len()).unwrap_or(0);
        for i in 0..nc {
            self.env.push(Tv { owner: TvOwner::Class(class), index: i as u32 });
        }
        if let Some(m) = method {
            for i in 0..nm {
                self.env.push(Tv { owner: TvOwner::Method(m), index: i as u32 });
            }
        }
        self.env_src = match (nc, nm) {
            (0, 0) => EnvSrc::None,
            (_, 0) => EnvSrc::This(class),
            (0, _) => EnvSrc::Targs,
            _ => EnvSrc::Both(class, nc, nm),
        };
    }

    /// Emits the methods, the constructors and the initializer of a class.
    pub fn class_bodies(e: &mut Emitter, id: ClassId) {
        let p = e.p;
        let c = p.class(id);
        for (i, m) in c.methods.iter().enumerate() {
            let Some(body) = &m.checked else { continue };
            let mref = MethodRef { class: id, index: i as u32 };
            let sig = e.sig(mref);
            let name = e.fn_symbol(mref);
            let mut f = Func::bare(e);
            f.set_env(id, Some(mref), m.is_static);
            f.ret = sig.ret;
            f.ret_store = f.new_store(sig.ret);
            f.allot(body);
            let mut params: Vec<(Repr, Store)> = Vec::new();
            if let Some(r) = sig.recv {
                f.this = f.new_store(r);
                params.push((r, f.this.clone()));
            }
            for (k, pr) in sig.params.iter().enumerate() {
                params.push((*pr, f.locals[body.params[k] as usize].clone()));
            }
            f.stmts(&body.stmts);
            f.close();
            let init = if m.is_static { Some(id) } else { None };
            let text = f.assemble("", &name, sig.ret, sig.generic, &params, init);
            e.add_function(&text);
        }
        for (i, k) in c.ctors.iter().enumerate() {
            let Some(body) = &k.checked else { continue };
            let sig = e.ctor_sig(id, i as u32);
            let name = e.ctor_symbol(id, i as u32);
            let mut f = Func::bare(e);
            f.set_env(id, None, false);
            f.this = f.new_store(Repr::Ref);
            f.allot(body);
            let mut params: Vec<(Repr, Store)> = vec![(Repr::Ref, f.this.clone())];
            for (k, pr) in sig.params.iter().enumerate() {
                params.push((*pr, f.locals[body.params[k] as usize].clone()));
            }
            f.stmts(&body.stmts);
            f.close();
            let text = f.assemble("", &name, Repr::Unit, false, &params, Some(id));
            e.add_function(&text);
        }
        if !e.is_trivial(id) {
            let body = c.static_init_checked.clone().unwrap_or(Body { locals: Vec::new(), params: Vec::new(), stmts: Vec::new() });
            let name = format!("@\"{}.clinit\"", sym(&c.qname));
            let mut f = Func::bare(e);
            f.allot(&body);
            // The superclass is initialized before the class.
            if let Some(s) = c.superclass.as_ref().and_then(|t| t.class_id()) {
                f.guard(s);
            }
            f.stmts(&body.stmts);
            f.close();
            let text = f.assemble("internal ", &name, Repr::Unit, false, &[], None);
            e.add_function(&text);
        }
    }

    // ---- constants and annotation values ----

    fn konst(&mut self, c: &Const, ty: &Type) -> Val {
        let r = self.repr(ty);
        match c {
            Const::Int(v, _) => Val::Imm(r, v.to_string()),
            Const::Float(f, _) => Val::Imm(r, self.float_lit(*f, r)),
            Const::Rational(q) => {
                let (slot, bytes, len) = self.e.rational_const(&format!("{}/{}", q.numer(), q.denom()));
                let t = self.tmp();
                self.ins(&format!("{t} = call ptr @cl_rational_const(ptr %ctx, ptr {slot}, ptr {bytes}, i64 {len})"));
                Val::Imm(Repr::Ref, t)
            }
            Const::Bool(b) => Val::Imm(Repr::Bool, if *b { "1".into() } else { "0".into() }),
            Const::Char(ch) => Val::Imm(Repr::Char, (*ch as u32).to_string()),
            Const::Str(s) => Val::Imm(Repr::Ref, self.e.string_lit(s)),
            Const::Enum(f) => {
                self.guard(f.class);
                let s = self.e.static_sym(*f);
                let t = self.tmp();
                self.ins(&format!("{t} = load ptr, ptr {s}"));
                self.root(&t)
            }
            Const::Class(t) => {
                let td = self.type_desc(t);
                let x = self.tmp();
                self.ins(&format!("{x} = call ptr @cl_class_of(ptr {td})"));
                Val::Imm(Repr::Ref, x)
            }
            Const::Ann(a) => self.annotation(a),
            Const::Array(items) => {
                let elem = self.e.p.array_elem(ty).unwrap_or_else(Type::error);
                let er = self.repr(&elem);
                let td = self.type_desc(&ty.bare());
                let a = self.tmp();
                self.ins(&format!("{a} = call ptr @cl_array_new(ptr %ctx, ptr {td}, i64 {})", items.len()));
                self.check();
                let arr = self.root(&a);
                for (i, it) in items.iter().enumerate() {
                    let v = self.konst(it, &elem);
                    let (a, v) = (self.val(&arr), self.val(&v));
                    let p = self.elem_addr(&a, &i.to_string(), er);
                    self.ins(&format!("store {} {v}, ptr {p}", er.ll()));
                }
                arr
            }
        }
    }

    /// A new value of an annotation as one use wrote it.
    fn annotation(&mut self, a: &AnnValue) -> Val {
        let c = self.e.p.class(a.class);
        let td = self.type_desc(&Type::simple(a.class));
        let o = self.tmp();
        self.ins(&format!("{o} = call ptr @cl_new(ptr %ctx, ptr {td})"));
        self.check();
        let obj = self.root(&o);
        for (i, (v, el)) in a.values.iter().zip(c.elements.iter()).enumerate() {
            let fr = self.repr(&el.ty);
            let x = self.konst(v, &el.ty);
            let (o, x) = (self.val(&obj), self.val(&x));
            let (lty, index, _) = self.e.field_place(a.class, FieldRef { class: a.class, index: i as u32 });
            let p = self.tmp();
            self.ins(&format!("{p} = getelementptr inbounds {lty}, ptr {o}, i32 0, i32 {index}"));
            self.ins(&format!("store {} {x}, ptr {p}", fr.ll()));
        }
        obj
    }

    pub fn annotation_thunk(e: &mut Emitter, a: &AnnValue) -> String {
        let name = format!("@ann.{}", e.next_thunk());
        let mut f = Func::bare(e);
        f.ret = Repr::Ref;
        f.ret_store = f.new_store(Repr::Ref);
        let v = f.annotation(a);
        let st = f.ret_store.clone();
        f.write(&st, &v);
        f.br("ret");
        let text = f.assemble("internal ", &name, Repr::Ref, false, &[], None);
        e.add_function(&text);
        name
    }

    fn refuse(&mut self, kind: u32, message: &str) {
        let (bytes, len) = self.e.byte_string(message);
        self.ins(&format!("call void @cl_mirror_fail(ptr %ctx, i32 {kind}, ptr {bytes}, i64 {len})"));
        self.br("unwind");
    }

    /// The code behind `Field.get` and `Field.set` for one field (section 8.9).
    pub fn field_thunks(e: &mut Emitter, fref: FieldRef) -> (String, String) {
        let p = e.p;
        let fd = p.field(fref);
        let c = p.class(fref.class);
        let n = e.next_thunk();
        let (get, set) = (format!("@fget.{n}"), format!("@fset.{n}"));
        let public = fd.aud == Aud::Public;
        let what = format!("{}.{}", c.qname, fd.name);
        let decl = e.repr(&fd.ty);
        for is_set in [false, true] {
            let mut f = Func::bare(e);
            f.set_env(fref.class, None, fd.is_static);
            let target = f.new_store(Repr::Ref);
            let value = f.new_store(Repr::Ref);
            f.this = target.clone();
            if !is_set {
                f.ret = Repr::Ref;
                f.ret_store = f.new_store(Repr::Ref);
            }
            if !public {
                f.refuse(5, &format!("the field {what} is not public"));
            } else if is_set && fd.is_final {
                f.refuse(5, &format!("the field {what} is final"));
            } else {
                if fd.is_static {
                    f.guard(fref.class);
                } else {
                    let t = f.read(&target);
                    let t = f.val(&t);
                    let x = f.tmp();
                    f.ins(&format!("{x} = call ptr @cl_cast_class(ptr %ctx, ptr {t}, ptr {})", f.e.class_info(fref.class)));
                    f.check();
                }
                let place = |f: &mut Func| -> String {
                    if fd.is_static {
                        f.e.static_sym(fref)
                    } else {
                        let t = f.read(&target);
                        let t = f.val(&t);
                        let (lty, index, _) = f.e.field_place(fref.class, fref);
                        let ptr = f.tmp();
                        f.ins(&format!("{ptr} = getelementptr inbounds {lty}, ptr {t}, i32 0, i32 {index}"));
                        ptr
                    }
                };
                if is_set {
                    let td = f.type_desc(&fd.ty);
                    let v = f.read(&value);
                    let vv = f.val(&v);
                    let x = f.tmp();
                    f.ins(&format!("{x} = call ptr @cl_cast(ptr %ctx, ptr {vv}, ptr {td})"));
                    f.check();
                    let v = f.convert(v, Repr::Ref, decl);
                    if decl != Repr::Unit {
                        let vv = f.val(&v);
                        let ptr = place(&mut f);
                        f.ins(&format!("store {} {vv}, ptr {ptr}", decl.ll()));
                    }
                    f.br("ret");
                } else {
                    let v = if decl == Repr::Unit {
                        Val::Unit
                    } else {
                        let ptr = place(&mut f);
                        let t = f.tmp();
                        f.ins(&format!("{t} = load {}, ptr {ptr}", decl.ll()));
                        if decl == Repr::Ref { f.root(&t) } else { Val::Imm(decl, t) }
                    };
                    let v = f.convert(v, decl, Repr::Ref);
                    let st = f.ret_store.clone();
                    f.write(&st, &v);
                    f.br("ret");
                }
            }
            let params: Vec<(Repr, Store)> = if is_set { vec![(Repr::Ref, target), (Repr::Ref, value)] } else { vec![(Repr::Ref, target)] };
            let ret = if is_set { Repr::Unit } else { Repr::Ref };
            let text = f.assemble("internal ", if is_set { &set } else { &get }, ret, false, &params, None);
            e.add_function(&text);
        }
        (get, set)
    }

    /// The code behind `Method.invoke` for one method (section 8.9).
    pub fn invoke_thunk(e: &mut Emitter, mref: MethodRef) -> String {
        let p = e.p;
        let md = p.method(mref);
        let c = p.class(mref.class);
        let name = format!("@invoke.{}", e.next_thunk());
        let what = format!("{}.{}", c.qname, md.name);
        let mut f = Func::bare(e);
        f.set_env(mref.class, None, md.is_static);
        f.ret = Repr::Ref;
        f.ret_store = f.new_store(Repr::Ref);
        let target = f.new_store(Repr::Ref);
        let args = f.new_store(Repr::Ref);
        f.this = target.clone();
        if md.aud != Aud::Public {
            f.refuse(5, &format!("the method {what} is not public"));
        } else if !md.tparams.is_empty() {
            f.refuse(3, &format!("the method {what} is generic, and a mirror does not call a generic method"));
        } else if md.is_abstract && c.is_interface() && false {
            f.refuse(3, "");
        } else {
            // The number of arguments is the number of parameters.
            let a = f.read(&args);
            let a = f.val(&a);
            let lp = f.tmp();
            f.ins(&format!("{lp} = getelementptr inbounds i8, ptr {a}, i64 16"));
            let n = f.tmp();
            f.ins(&format!("{n} = load i64, ptr {lp}"));
            let ok = f.tmp();
            f.ins(&format!("{ok} = icmp eq i64 {n}, {}", md.params.len()));
            let (go, bad) = (f.new_label("count.ok"), f.new_label("count.bad"));
            f.cond_br(&ok, &go, &bad);
            f.label(&bad);
            f.refuse(3, &format!("{what} takes {} arguments", md.params.len()));
            f.label(&go);
            if !md.is_static {
                let t = f.read(&target);
                let t = f.val(&t);
                let x = f.tmp();
                f.ins(&format!("{x} = call ptr @cl_cast_class(ptr %ctx, ptr {t}, ptr {})", f.e.class_info(mref.class)));
                f.check();
            }
            // Each argument is tested as a cast would test it, and then held as the
            // parameter is.
            let mut locals = Vec::new();
            for (i, prm) in md.params.iter().enumerate() {
                let a = f.read(&args);
                let a = f.val(&a);
                let ep = f.elem_addr(&a, &i.to_string(), Repr::Ref);
                let v = f.tmp();
                f.ins(&format!("{v} = load ptr, ptr {ep}"));
                let held = f.root(&v);
                let td = f.type_desc(&prm.ty);
                let hv = f.val(&held);
                let x = f.tmp();
                f.ins(&format!("{x} = call ptr @cl_cast(ptr %ctx, ptr {hv}, ptr {td})"));
                f.check();
                let pr = f.repr(&prm.ty);
                let v = f.convert(held, Repr::Ref, pr);
                let st = f.new_store(pr);
                f.write(&st, &v);
                f.locals.push(st);
                locals.push(TExpr { kind: TKind::Local(f.locals.len() as u32 - 1), ty: prm.ty.clone() });
            }
            let mut self_ty = Type::class(mref.class, (0..c.tparams.len()).map(|i| Arg::Ty(Type::var(Tv { owner: TvOwner::Class(mref.class), index: i as u32 }))).collect());
            self_ty.nullable = md.recv_nullable;
            let is_virtual = md.is_virtual(c.is_interface());
            let call = Call {
                // The target arrives as an object, whatever the receiver's own form is.
                recv: if md.is_static {
                    None
                } else {
                    let object = TExpr { kind: TKind::This, ty: Type::simple(p.wk.object).nullable() };
                    Some(TExpr { kind: TKind::Coerce(Box::new(object)), ty: self_ty })
                },
                method: mref,
                class_args: Vec::new(),
                targs: Vec::new(),
                args: locals,
                dispatch: if is_virtual { Dispatch::Virtual } else { Dispatch::Direct },
            };
            let v = f.call(&call, &md.ret);
            let v = f.convert(v, f.repr(&md.ret), Repr::Ref);
            let st = f.ret_store.clone();
            f.write(&st, &v);
            f.br("ret");
        }
        let text = f.assemble("internal ", &name, Repr::Ref, false, &[(Repr::Ref, target), (Repr::Ref, args)], None);
        e.add_function(&text);
        name
    }

    // ---- what the runtime calls, and `main` ----

    pub fn program_functions(e: &mut Emitter, entry: ClassId) {
        let p = e.p;
        let find = |name: &str| -> Option<ClassId> { p.by_name.get(&("cleat".to_string(), name.to_string())).and_then(|v| v.first().copied()) };
        let string = Type::simple(p.wk.string);

        // An exception the runtime raises, by its number in `rt/src/abi.rs`.
        let kinds = [
            "ArithmeticException",
            "ClassCastException",
            "IndexOutOfBoundsException",
            "IllegalArgumentException",
            "IllegalStateException",
            "IllegalAccessException",
            "ClassInitializationException",
            "OutOfMemoryException",
            "CancellationException",
            "IOException",
        ];
        let mut f = Func::bare(e);
        f.ret = Repr::Ref;
        f.ret_store = f.new_store(Repr::Ref);
        let kind = Store::Alloca(f.alloca(Repr::I32), Repr::I32);
        let msg = f.new_store(Repr::Ref);
        let cause = f.new_store(Repr::Ref);
        let k = f.read(&kind);
        let k = f.val(&k);
        let labels: Vec<String> = kinds.iter().map(|_| f.new_label("kind")).collect();
        let cases: Vec<String> = labels.iter().enumerate().map(|(i, l)| format!("i32 {i}, label %{l}")).collect();
        f.ins(&format!("switch i32 {k}, label %{} [ {} ]", labels[4], cases.join(" ")));
        f.open = false;
        for (name, l) in kinds.iter().zip(labels.iter()) {
            f.label(l);
            let Some(cid) = find(name) else {
                f.e.errors.push(format!("the prelude does not declare `cleat.{name}`"));
                continue;
            };
            let c = p.class(cid);
            let one = c.ctors.iter().position(|k| k.params.len() == 1 && k.params[0].ty == string);
            let two = c.ctors.iter().position(|k| k.params.len() == 2 && k.params[0].ty == string);
            let (Some(one), Some(two)) = (one, two) else {
                f.e.errors.push(format!("`cleat.{name}` needs the constructors (String) and (String, Throwable)"));
                continue;
            };
            let td = f.type_desc(&Type::simple(cid));
            let o = f.tmp();
            f.ins(&format!("{o} = call ptr @cl_new(ptr %ctx, ptr {td})"));
            f.check();
            let obj = f.root(&o);
            let st = f.ret_store.clone();
            f.write(&st, &obj);
            let cv = f.read(&cause);
            let cv = f.val(&cv);
            let has = f.tmp();
            f.ins(&format!("{has} = icmp ne ptr {cv}, null"));
            let (with, without) = (f.new_label("with.cause"), f.new_label("no.cause"));
            f.cond_br(&has, &with, &without);
            for (label, index, n) in [(&with, two, 2), (&without, one, 1)] {
                f.label(label);
                let (o, m) = (f.val(&obj), f.read(&msg));
                let m = f.val(&m);
                let mut args = format!("ptr %ctx, ptr {o}, ptr {m}");
                if n == 2 {
                    let cv = f.read(&cause);
                    let cv = f.val(&cv);
                    args.push_str(&format!(", ptr {cv}"));
                }
                f.ins(&format!("call void {}({args})", f.e.ctor_symbol(cid, index as u32)));
                f.br("ret");
            }
        }
        let text = f.assemble("", "@cleat_make_exception", Repr::Ref, false, &[(Repr::I32, kind), (Repr::Ref, msg), (Repr::Ref, cause)], None);
        e.add_function(&text);

        // The body of a started thread, and the recording of a suppressed exception.
        let method = |e: &mut Emitter, class: Option<ClassId>, name: &str| -> Option<String> {
            let cid = class?;
            let i = p.class(cid).methods.iter().position(|m| m.name == name)?;
            Some(e.fn_symbol(MethodRef { class: cid, index: i as u32 }))
        };
        match method(e, find("Thread"), "run") {
            Some(run) => e.add_function(&format!("define void @cleat_thread_run(ptr %ctx, ptr %t) {{\n  call void {run}(ptr %ctx, ptr %t)\n  ret void\n}}\n")),
            None => e.errors.push("the prelude's `Thread` declares no `run`".into()),
        }
        match method(e, Some(p.wk.throwable), "suppress") {
            Some(s) => e.add_function(&format!("define void @cleat_suppress(ptr %ctx, ptr %a, ptr %b) {{\n  call void {s}(ptr %ctx, ptr %a, ptr %b)\n  ret void\n}}\n")),
            None => e.errors.push("the prelude's `Throwable` declares no `suppress`".into()),
        }

        // `main`.
        let c = p.class(entry);
        let args_ty = p.array_of(string);
        let mains: Vec<(usize, &Method)> = c.methods.iter().enumerate().filter(|(_, m)| m.name == "main" && m.is_static).collect();
        let unit = Type::simple(p.wk.unit);
        let ok: Vec<&(usize, &Method)> = mains
            .iter()
            .filter(|(_, m)| m.aud == Aud::Public && m.ret == unit && m.tparams.is_empty() && (m.params.is_empty() || (m.params.len() == 1 && m.params[0].ty == args_ty)))
            .collect();
        if ok.len() != 1 {
            e.errors.push(format!(
                "`{}` is not an entry class: it declares exactly one of `public static void main()` and `public static void main(String[] args)`",
                c.qname
            ));
            return;
        }
        let (index, m) = ok[0];
        let f = e.fn_symbol(MethodRef { class: entry, index: *index as u32 });
        let call = if m.params.is_empty() {
            format!("  call void {f}(ptr %ctx)\n")
        } else {
            format!("  %args = call ptr @cl_args(ptr %ctx)\n  call void {f}(ptr %ctx, ptr %args)\n")
        };
        e.add_function(&format!(
            "define i32 @main(i32 %argc, ptr %argv) {{\n  %ctx = call ptr @cl_start(ptr @cleat_program)\n{call}  %code = call i32 @cl_finish(ptr %ctx)\n  ret i32 %code\n}}\n"
        ));
    }
}
