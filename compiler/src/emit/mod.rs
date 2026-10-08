//! Turns a checked program into LLVM IR text. `compiler/DESIGN.md` describes the shape
//! of the generated code, and `rt/src/abi.rs` the data both sides share.

mod func;
pub mod link;

use crate::ast::{TypeKind, Variance};
use crate::sema::decl::{self, ImplOf};
use crate::sema::program::*;
use crate::sema::types::*;
use func::Func;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write;

/// How a value of a static type is held in compiled code.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Repr {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Char,
    /// A `Pointer`: an address the collector does not follow.
    Ptr,
    /// A pointer to an object.
    Ref,
    /// No value: the result of a `void` method.
    Unit,
}

impl Repr {
    pub fn ll(self) -> &'static str {
        use Repr::*;
        match self {
            I8 | U8 | Bool => "i8",
            I16 | U16 => "i16",
            I32 | U32 | Char => "i32",
            I64 | U64 => "i64",
            F32 => "float",
            F64 => "double",
            Ptr | Ref => "ptr",
            Unit => "void",
        }
    }

    /// The runtime's number for a machine kind; 0 for a pointer to an object.
    pub fn kind(self) -> u32 {
        use Repr::*;
        match self {
            I8 => 1,
            I16 => 2,
            I32 => 3,
            I64 => 4,
            U8 => 5,
            U16 => 6,
            U32 => 7,
            U64 => 8,
            F32 => 9,
            F64 => 10,
            Bool => 11,
            Char => 12,
            Ptr => 13,
            Ref | Unit => 0,
        }
    }

    pub fn zero(self) -> &'static str {
        match self {
            Repr::F32 | Repr::F64 => "0.0",
            Repr::Ptr | Repr::Ref => "null",
            Repr::Unit => "",
            _ => "0",
        }
    }

    pub fn is_machine(self) -> bool {
        !matches!(self, Repr::Ref | Repr::Unit)
    }
}

/// The machine signature of a method or a constructor.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Sig {
    pub recv: Option<Repr>,
    pub params: Vec<Repr>,
    pub ret: Repr,
    /// Whether a pointer to the method's type arguments follows the context.
    pub generic: bool,
    /// A foreign method: no context, and the C calling convention's own types.
    pub foreign: bool,
}

impl Sig {
    pub fn ll_params(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if !self.foreign {
            parts.push("ptr".into());
        }
        if self.generic {
            parts.push("ptr".into());
        }
        if let Some(r) = self.recv {
            parts.push(r.ll().into());
        }
        parts.extend(self.params.iter().map(|r| r.ll().to_string()));
        parts.join(", ")
    }
}

pub struct Layout {
    pub ty: String,
    /// Every instance field, the superclass's first, with its position in the struct.
    pub fields: Vec<(FieldRef, Repr, u32)>,
}

/// The well-known classes' kinds, as `rt/src/abi.rs` numbers them.
fn special_kind(p: &Program, id: ClassId) -> u32 {
    let w = &p.wk;
    let c = p.class(id);
    let named = |n: &str| c.package == "cleat" && c.name == n;
    if id == w.string {
        20
    } else if id == w.array {
        21
    } else if id == w.rational {
        22
    } else if id == w.class {
        23
    } else if id == w.unit {
        24
    } else if id == w.null {
        25
    } else if named("Lock") {
        26
    } else if named("Condition") {
        27
    } else if named("Thread") {
        28
    } else if id == w.object {
        29
    } else {
        0
    }
}

pub struct Emitter<'p> {
    pub p: &'p Program,
    /// Type and global definitions, then functions.
    globals: String,
    funcs: String,
    decls: BTreeSet<String>,
    class_repr: Vec<Repr>,
    layouts: Vec<Option<Layout>>,
    trivial: Vec<bool>,
    roots: HashMap<MethodRef, MethodRef>,
    selectors: HashMap<MethodRef, u32>,
    req_selectors: HashMap<MethodRef, u32>,
    /// For each class with instances, the body for each method it can be sent.
    vtables: Vec<Vec<(MethodRef, MethodRef)>>,
    reqs: Vec<Vec<(MethodRef, MethodRef)>>,
    bridges: BTreeMap<String, (MethodRef, MethodRef, bool)>,
    strings: HashMap<String, usize>,
    bytes: HashMap<String, usize>,
    texprs: HashMap<String, usize>,
    cached: BTreeMap<usize, ()>,
    rationals: HashMap<String, usize>,
    static_roots: Vec<String>,
    thunks: u32,
    pub errors: Vec<String>,
}

fn esc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b' ' | b'.' | b'_' | b'-' | b'$' | b'/' | b'<' | b'>' | b',' | b'#' | b'@' | b'(' | b')' | b':') {
            out.push(b as char);
        } else {
            let _ = write!(out, "\\{b:02X}");
        }
    }
    out
}

/// A name LLVM accepts between quotes.
fn sym(s: &str) -> String {
    s.chars().map(|c| if c == '"' || c == '\\' { '_' } else { c }).collect()
}

impl<'p> Emitter<'p> {
    pub fn repr(&self, t: &Type) -> Repr {
        if t.nullable {
            return Repr::Ref;
        }
        match &t.ty {
            Ty::Class(id, _) => self.class_repr[*id as usize],
            _ => Repr::Ref,
        }
    }

    pub fn declare(&mut self, line: String) {
        self.decls.insert(line);
    }

    pub fn class_info(&self, id: ClassId) -> String {
        format!("@\"ci.{}\"", sym(&self.p.class(id).qname))
    }

    pub fn layout(&self, id: ClassId) -> &Layout {
        self.layouts[id as usize].as_ref().expect("a class with a layout")
    }

    pub fn field_place(&self, owner: ClassId, f: FieldRef) -> (String, u32, Repr) {
        // The receiver's class may be a subclass; the field has one place in all of them.
        let l = self.layout(owner);
        let (_, repr, index) = *l.fields.iter().find(|(r, _, _)| *r == f).expect("a field of the class");
        (l.ty.clone(), index, repr)
    }

    pub fn static_sym(&self, f: FieldRef) -> String {
        format!("@\"s.{}.{}\"", sym(&self.p.class(f.class).qname), sym(&self.p.field(f).name))
    }

    pub fn init_rec(&self, id: ClassId) -> String {
        format!("@\"init.{}\"", sym(&self.p.class(id).qname))
    }

    pub fn is_trivial(&self, id: ClassId) -> bool {
        self.trivial[id as usize]
    }

    pub fn root_of(&self, m: MethodRef) -> MethodRef {
        self.roots.get(&m).copied().unwrap_or(m)
    }

    pub fn selector(&self, root: MethodRef) -> u32 {
        *self.selectors.get(&root).expect("a selector for a virtual method")
    }

    pub fn req_selector(&self, req: MethodRef) -> u32 {
        *self.req_selectors.get(&req).expect("a selector for a static requirement")
    }

    fn erasure(&self, t: &Type) -> String {
        match &t.ty {
            Ty::Class(id, args) if *id == self.p.wk.array => match args.first() {
                Some(Arg::Ty(e)) => format!("{}Array", self.erasure(e)),
                _ => "ObjectArray".into(),
            },
            Ty::Class(id, _) => self.p.class(*id).name.clone(),
            _ => "Object".into(),
        }
    }

    pub fn sig(&self, m: MethodRef) -> Sig {
        let md = self.p.method(m);
        let recv = if md.is_static {
            None
        } else {
            let mut t = Type::simple(m.class);
            t.nullable = md.recv_nullable;
            Some(self.repr(&t))
        };
        Sig {
            recv: if md.foreign { None } else { recv },
            params: md.params.iter().map(|p| if md.foreign && self.p.array_elem(&p.ty).is_some() { Repr::Ptr } else { self.repr(&p.ty) }).collect(),
            ret: self.repr(&md.ret),
            generic: !md.tparams.is_empty(),
            foreign: md.foreign,
        }
    }

    pub fn ctor_sig(&self, class: ClassId, index: u32) -> Sig {
        let c = self.p.class(class);
        let k = &c.ctors[index as usize];
        let mut params = Vec::new();
        if c.is_enum() {
            params.push(Repr::Ref);
            params.push(Repr::I64);
        }
        params.extend(k.params.iter().map(|p| self.repr(&p.ty)));
        Sig { recv: Some(Repr::Ref), params, ret: Repr::Unit, generic: false, foreign: false }
    }

    /// The symbol of a method's code, declared when the runtime or C supplies it.
    pub fn fn_symbol(&mut self, m: MethodRef) -> String {
        let md = self.p.method(m);
        let c = self.p.class(m.class);
        let s = self.sig(m);
        if md.foreign {
            let name = md.symbol.clone().unwrap_or_else(|| md.name.clone());
            self.declare(format!("declare {} @\"{}\"({})", s.ret.ll(), sym(&name), s.ll_params()));
            return format!("@\"{}\"", sym(&name));
        }
        if md.intrinsic {
            let mut name = format!("cl_{}_{}", c.name, md.name);
            for prm in &md.params {
                name.push('_');
                name.push_str(&self.erasure(&prm.ty));
            }
            self.declare(format!("declare {} @{name}({})", s.ret.ll(), s.ll_params()));
            return format!("@{name}");
        }
        format!("@\"{}.{}/{}\"", sym(&c.qname), sym(&md.name), m.index)
    }

    pub fn ctor_symbol(&self, class: ClassId, index: u32) -> String {
        format!("@\"{}.new/{}\"", sym(&self.p.class(class).qname), index)
    }

    /// A string literal as a static object.
    pub fn string_lit(&mut self, s: &str) -> String {
        let n = self.strings.len();
        let id = *self.strings.entry(s.to_string()).or_insert(n);
        if id == n {
            let chars: Vec<String> = s.chars().map(|c| format!("i32 {}", c as u32)).collect();
            let len = chars.len();
            let _ = writeln!(
                self.globals,
                "@str.{id} = internal global {{ ptr, i64, i64, [{len} x i32] }} {{ ptr null, i64 2, i64 {len}, [{len} x i32] [{}] }}",
                chars.join(", ")
            );
        }
        format!("@str.{id}")
    }

    /// Constant bytes: a name the runtime reads.
    pub fn byte_string(&mut self, s: &str) -> (String, usize) {
        let n = self.bytes.len();
        let id = *self.bytes.entry(s.to_string()).or_insert(n);
        if id == n {
            let _ = writeln!(self.globals, "@b.{id} = private constant [{} x i8] c\"{}\"", s.len().max(1), if s.is_empty() { "\\00".to_string() } else { esc(s) });
        }
        (format!("@b.{id}"), s.len())
    }

    pub fn rational_const(&mut self, text: &str) -> (String, String, usize) {
        let n = self.rationals.len();
        let id = *self.rationals.entry(text.to_string()).or_insert(n);
        if id == n {
            let _ = writeln!(self.globals, "@rat.{id} = internal global ptr null");
            self.static_roots.len();
        }
        let (b, len) = self.byte_string(text);
        (format!("@rat.{id}"), b, len)
    }

    /// A type as constant data. The result is the symbol and whether the type mentions
    /// a type variable.
    pub fn texpr(&mut self, t: &Type, tv: &dyn Fn(Tv) -> Option<u32>) -> (String, bool) {
        let mut open = false;
        let body = self.texpr_body(t, tv, &mut open);
        let n = self.texprs.len();
        let id = *self.texprs.entry(body.clone()).or_insert(n);
        if id == n {
            let _ = writeln!(self.globals, "@te.{id} = private constant %TypeExpr {body}");
        }
        (format!("@te.{id}"), open)
    }

    fn ptr_array(&mut self, items: &[String]) -> String {
        if items.is_empty() {
            return "null".into();
        }
        let key = format!("[{} x ptr] [{}]", items.len(), items.iter().map(|i| format!("ptr {i}")).collect::<Vec<_>>().join(", "));
        let n = self.bytes.len();
        let id = *self.bytes.entry(format!("\u{1}{key}")).or_insert(n);
        if id == n {
            let _ = writeln!(self.globals, "@pa.{id} = private constant {key}");
        }
        format!("@pa.{id}")
    }

    fn texpr_body(&mut self, t: &Type, tv: &dyn Fn(Tv) -> Option<u32>, open: &mut bool) -> String {
        let quals: Vec<String> = t.quals.iter().map(|q| self.class_info(*q)).collect();
        let nquals = quals.len();
        let quals = self.ptr_array(&quals);
        let nullable = t.nullable as u32;
        match &t.ty {
            Ty::Var(v) => {
                *open = true;
                let index = tv(*v).unwrap_or(0);
                format!("{{ i32 1, i32 {nullable}, ptr null, i32 {index}, i32 0, ptr null, i32 {nquals}, i32 0, ptr {quals}, ptr null }}")
            }
            Ty::Class(id, args) => {
                let mut items = Vec::new();
                for a in args {
                    items.push(match a {
                        Arg::Ty(at) => self.texpr(at, tv),
                        Arg::Wild(e, s) => {
                            let (bound, lower) = match (e, s) {
                                (_, Some(l)) => (Some(self.texpr(l, tv)), 1),
                                (Some(u), None) => (Some(self.texpr(u, tv)), 0),
                                (None, None) => (None, 0),
                            };
                            let (b, o) = bound.unwrap_or(("null".into(), false));
                            let body = format!("{{ i32 2, i32 0, ptr null, i32 0, i32 0, ptr null, i32 0, i32 {lower}, ptr null, ptr {b} }}");
                            let n = self.texprs.len();
                            let wid = *self.texprs.entry(body.clone()).or_insert(n);
                            if wid == n {
                                let _ = writeln!(self.globals, "@te.{wid} = private constant %TypeExpr {body}");
                            }
                            (format!("@te.{wid}"), o)
                        }
                    });
                }
                for (_, o) in &items {
                    *open |= *o;
                }
                let names: Vec<String> = items.into_iter().map(|(s, _)| s).collect();
                let nargs = names.len();
                let args = self.ptr_array(&names);
                format!("{{ i32 0, i32 {nullable}, ptr {}, i32 0, i32 {nargs}, ptr {args}, i32 {nquals}, i32 0, ptr {quals}, ptr null }}", self.class_info(*id))
            }
            // A type that was already reported; no code is run from it.
            _ => format!("{{ i32 0, i32 {nullable}, ptr {}, i32 0, i32 0, ptr null, i32 0, i32 0, ptr null, ptr null }}", self.class_info(self.p.wk.object)),
        }
    }

    /// The slot that holds the descriptor of a type with no type variable.
    pub fn cached_type(&mut self, texpr: &str) -> String {
        let id: usize = texpr.trim_start_matches("@te.").parse().unwrap_or(0);
        if self.cached.insert(id, ()).is_none() {
            let _ = writeln!(self.globals, "@tc.{id} = internal global ptr null");
        }
        format!("@tc.{id}")
    }

    pub fn next_thunk(&mut self) -> u32 {
        self.thunks += 1;
        self.thunks
    }

    pub fn add_function(&mut self, text: &str) {
        self.funcs.push_str(text);
        self.funcs.push('\n');
    }

    // ---- bridges ----

    /// The code a class's table holds for `root` when `imp` is the body: `imp` itself,
    /// or a bridge when the two have different machine signatures.
    fn table_entry(&mut self, imp: MethodRef, root: MethodRef, is_req: bool) -> String {
        let (a, b) = (self.sig(imp), self.sig(root));
        let target = self.fn_symbol(imp);
        if a == b {
            return target;
        }
        let name = format!("bridge.{}.{}/{}.as.{}/{}", self.p.class(imp.class).qname, self.p.method(imp).name, imp.index, self.p.class(root.class).qname, root.index);
        self.bridges.insert(name.clone(), (imp, root, is_req));
        format!("@\"{}\"", sym(&name))
    }

    fn emit_bridges(&mut self) {
        let list: Vec<(String, (MethodRef, MethodRef, bool))> = self.bridges.iter().map(|(k, v)| (k.clone(), *v)).collect();
        for (name, (imp, root, _)) in list {
            let (a, b) = (self.sig(imp), self.sig(root));
            let target = self.fn_symbol(imp);
            let mut f = Func::bare(self);
            let mut params = vec!["ptr %ctx".to_string()];
            if b.generic {
                params.push("ptr %targs".into());
            }
            let mut args: Vec<String> = Vec::new();
            if let (Some(have), Some(want)) = (b.recv, a.recv) {
                params.push(format!("{} %this", have.ll()));
                let v = f.convert_raw("%this", have, want);
                args.push(format!("{} {v}", want.ll()));
            }
            for (i, (have, want)) in b.params.iter().zip(a.params.iter()).enumerate() {
                params.push(format!("{} %a{i}", have.ll()));
                let v = f.convert_raw(&format!("%a{i}"), *have, *want);
                args.push(format!("{} {v}", want.ll()));
            }
            let mut all = vec!["ptr %ctx".to_string()];
            if a.generic {
                all.push("ptr %targs".into());
            }
            all.extend(args);
            let result = if a.ret == Repr::Unit {
                f.ins(&format!("call void {target}({})", all.join(", ")));
                String::new()
            } else {
                let t = f.tmp();
                f.ins(&format!("{t} = call {} {target}({})", a.ret.ll(), all.join(", ")));
                t
            };
            // An exception leaves the result unused: return without boxing it.
            let e = f.tmp();
            f.ins(&format!("{e} = load ptr, ptr %ctx"));
            let c = f.tmp();
            f.ins(&format!("{c} = icmp eq ptr {e}, null"));
            f.ins(&format!("br i1 {c}, label %ok, label %raised"));
            f.raw("raised:");
            if b.ret == Repr::Unit {
                f.raw("  ret void");
            } else {
                f.raw(&format!("  ret {} {}", b.ret.ll(), b.ret.zero()));
            }
            f.raw("ok:");
            if b.ret == Repr::Unit {
                f.raw("  ret void");
            } else {
                let v = if a.ret == Repr::Unit { f.unit_object() } else { f.convert_raw(&result, a.ret, b.ret) };
                f.raw(&format!("  ret {} {v}", b.ret.ll()));
            }
            let body = f.take_body();
            let text = format!("define internal {} @\"{}\"({}) {{\nentry:\n{body}}}\n", b.ret.ll(), sym(&name), params.join(", "));
            self.add_function(&text);
        }
    }

    // ---- classes ----

    fn method_table(&mut self, entries: &[(MethodRef, MethodRef)], is_req: bool, name: &str) -> (String, usize) {
        if entries.is_empty() {
            return ("null".into(), 0);
        }
        let mut rows: Vec<(u32, String)> = Vec::new();
        for (root, imp) in entries {
            let sel = if is_req { self.req_selector(*root) } else { self.selector(*root) };
            let f = self.table_entry(*imp, *root, is_req);
            rows.push((sel, f));
        }
        rows.sort();
        rows.dedup_by_key(|r| r.0);
        let body: Vec<String> = rows.iter().map(|(s, f)| format!("%MethodEntry {{ i32 {s}, i32 0, ptr {f} }}")).collect();
        let _ = writeln!(self.globals, "{name} = private constant [{} x %MethodEntry] [{}]", rows.len(), body.join(", "));
        (name.to_string(), rows.len())
    }

    fn u32_array(&mut self, items: &[String], name: &str) -> String {
        if items.is_empty() {
            return "null".into();
        }
        let body: Vec<String> = items.iter().map(|i| format!("i32 {i}")).collect();
        let _ = writeln!(self.globals, "{name} = private constant [{} x i32] [{}]", items.len(), body.join(", "));
        name.to_string()
    }

    fn emit_class(&mut self, id: ClassId) {
        let c = self.p.class(id).clone();
        let q = sym(&c.qname);
        let (name, name_len) = self.byte_string(&c.qname);
        let mut flags = 0u32;
        if c.is_interface() {
            flags |= 1;
        }
        if c.is_value() {
            flags |= 2;
        }
        if c.is_abstract {
            flags |= 4;
        }
        if c.is_enum() {
            flags |= 8;
        }
        if c.is_annotation() {
            flags |= 16;
        }
        if c.lambda.is_some() {
            flags |= 32;
        }
        if c.inherited {
            flags |= 128;
        }
        match c.qualifier {
            Qualifier::Refines => flags |= 256,
            Qualifier::Widens => flags |= 512,
            Qualifier::No => {}
        }
        let machine = self.class_repr[id as usize].kind();
        let kind = if machine != 0 { machine } else { special_kind(self.p, id) };

        // The type variables of the class's own header.
        let env: Vec<Tv> = match &c.lambda {
            Some(l) => l.env.clone(),
            None => (0..c.tparams.len()).map(|i| Tv { owner: TvOwner::Class(id), index: i as u32 }).collect(),
        };
        let tv = |v: Tv| env.iter().position(|e| *e == v).map(|i| i as u32);
        let nparams = env.len();
        let variances = if c.tparams.is_empty() {
            "null".to_string()
        } else {
            let bytes: Vec<String> = c
                .tparams
                .iter()
                .map(|t| {
                    format!(
                        "i8 {}",
                        match t.variance {
                            Variance::Invariant => 0,
                            Variance::Out => 1,
                            Variance::In => 2,
                        }
                    )
                })
                .collect();
            let _ = writeln!(self.globals, "@\"var.{q}\" = private constant [{} x i8] [{}]", bytes.len(), bytes.join(", "));
            format!("@\"var.{q}\"")
        };
        // Supertypes: the superclass first. A refinement lists the refinements above it.
        let mut supers: Vec<String> = Vec::new();
        if c.qualifier == Qualifier::Refines {
            for above in &c.refines_above {
                supers.push(self.texpr(&Type::simple(*above), &tv).0);
            }
        } else {
            if let Some(s) = &c.superclass {
                if !c.is_annotation() {
                    flags |= 64;
                    supers.push(self.texpr(s, &tv).0);
                }
            }
            for i in &c.interfaces {
                supers.push(self.texpr(i, &tv).0);
            }
        }
        let nsupers = supers.len();
        let supers = self.ptr_array(&supers);

        // Size and pointer fields.
        let (size, refs, nrefs, fields, nfields) = match &self.layouts[id as usize] {
            Some(l) => {
                let ty = l.ty.clone();
                let all = l.fields.clone();
                let size = format!("ptrtoint (ptr getelementptr ({ty}, ptr null, i32 1) to i32)");
                let offset = |i: u32| format!("ptrtoint (ptr getelementptr ({ty}, ptr null, i32 0, i32 {i}) to i32)");
                let ref_items: Vec<String> = all.iter().filter(|(_, r, _)| *r == Repr::Ref).map(|(_, _, i)| offset(*i)).collect();
                let nrefs = ref_items.len();
                let refs = self.u32_array(&ref_items, &format!("@\"refs.{q}\""));
                // A value class lists its fields, for the default `equals` and its kin.
                let mut rows = Vec::new();
                if c.is_value() {
                    for (f, r, i) in &all {
                        let (fname, flen) = self.byte_string(&self.p.field(*f).name.clone());
                        rows.push(format!("%FieldInfo {{ ptr {fname}, i32 {flen}, i32 {}, i32 {}, i32 0 }}", offset(*i), r.kind()));
                    }
                }
                let nfields = rows.len();
                let fields = if rows.is_empty() {
                    "null".to_string()
                } else {
                    let _ = writeln!(self.globals, "@\"fields.{q}\" = private constant [{nfields} x %FieldInfo] [{}]", rows.join(", "));
                    format!("@\"fields.{q}\"")
                };
                (size, refs, nrefs, fields, nfields)
            }
            None => ("16".to_string(), "null".to_string(), 0, "null".to_string(), 0),
        };

        let vt = std::mem::take(&mut self.vtables[id as usize]);
        let (methods, nmethods) = self.method_table(&vt, false, &format!("@\"vt.{q}\""));
        let rq = std::mem::take(&mut self.reqs[id as usize]);
        let (reqs, nreqs) = self.method_table(&rq, true, &format!("@\"rq.{q}\""));

        let (anns, nanns) = self.ann_table(&c.anns, &format!("@\"anns.{q}\""));
        let (mfields, nmfields, mmethods, nmmethods) = self.member_tables(id);

        let _ = writeln!(
            self.globals,
            "@\"ci.{q}\" = constant %ClassInfo {{ ptr {name}, i32 {name_len}, i32 {flags}, i32 {kind}, i32 {nparams}, ptr {variances}, ptr null, i32 {nsupers}, i32 {size}, ptr {supers}, i32 {nrefs}, i32 {nmethods}, ptr {refs}, ptr {methods}, i32 {nreqs}, i32 {nfields}, ptr {reqs}, ptr {fields}, i32 {nanns}, i32 {nmfields}, ptr {anns}, ptr {mfields}, i32 {nmmethods}, i32 0, ptr {mmethods} }}"
        );
    }

    /// The declaration annotations among some uses, as the runtime reads them.
    fn ann_table(&mut self, anns: &[AnnValue], name: &str) -> (String, usize) {
        let kept: Vec<&AnnValue> = anns.iter().filter(|a| self.p.class(a.class).qualifier == Qualifier::No).collect();
        if kept.is_empty() {
            return ("null".into(), 0);
        }
        let mut rows = Vec::new();
        for a in &kept {
            let make = Func::annotation_thunk(self, a);
            rows.push(format!("%AnnInfo {{ ptr {}, ptr {make} }}", self.class_info(a.class)));
        }
        let _ = writeln!(self.globals, "{name} = private constant [{} x %AnnInfo] [{}]", rows.len(), rows.join(", "));
        (name.to_string(), rows.len())
    }

    fn member_tables(&mut self, id: ClassId) -> (String, usize, String, usize) {
        let c = self.p.class(id).clone();
        let q = sym(&c.qname);
        let mut frows = Vec::new();
        for (i, f) in c.fields.iter().enumerate() {
            if !f.anns.iter().any(|a| self.p.class(a.class).qualifier == Qualifier::No) {
                continue;
            }
            let (name, len) = self.byte_string(&f.name);
            let (anns, nanns) = self.ann_table(&f.anns, &format!("@\"fanns.{q}.{i}\""));
            let fref = FieldRef { class: id, index: i as u32 };
            let (get, set) = Func::field_thunks(self, fref);
            let flags = f.is_static as u32;
            frows.push(format!("%MemberInfo {{ ptr {name}, i32 {len}, i32 {flags}, i32 {nanns}, i32 0, ptr {anns}, ptr null, ptr {get}, ptr {set}, ptr null }}"));
        }
        let mut mrows = Vec::new();
        for (i, m) in c.methods.iter().enumerate() {
            if !m.anns.iter().any(|a| self.p.class(a.class).qualifier == Qualifier::No) {
                continue;
            }
            let (name, len) = self.byte_string(&m.name);
            let (anns, nanns) = self.ann_table(&m.anns, &format!("@\"manns.{q}.{i}\""));
            let mut prows = Vec::new();
            for (pi, prm) in m.params.iter().enumerate() {
                let (pname, plen) = self.byte_string(&prm.name);
                let (panns, pn) = self.ann_table(&prm.anns, &format!("@\"panns.{q}.{i}.{pi}\""));
                prows.push(format!("%ParamInfo {{ ptr {pname}, i32 {plen}, i32 {pn}, ptr {panns} }}"));
            }
            let params = if prows.is_empty() {
                "null".to_string()
            } else {
                let _ = writeln!(self.globals, "@\"params.{q}.{i}\" = private constant [{} x %ParamInfo] [{}]", prows.len(), prows.join(", "));
                format!("@\"params.{q}.{i}\"")
            };
            let mref = MethodRef { class: id, index: i as u32 };
            let invoke = Func::invoke_thunk(self, mref);
            let flags = m.is_static as u32;
            mrows.push(format!(
                "%MemberInfo {{ ptr {name}, i32 {len}, i32 {flags}, i32 {nanns}, i32 {}, ptr {anns}, ptr {params}, ptr null, ptr null, ptr {invoke} }}",
                m.params.len()
            ));
        }
        let table = |e: &mut Self, rows: Vec<String>, name: String| -> (String, usize) {
            if rows.is_empty() {
                return ("null".into(), 0);
            }
            let _ = writeln!(e.globals, "{name} = private constant [{} x %MemberInfo] [{}]", rows.len(), rows.join(", "));
            (name, rows.len())
        };
        let (mf, nf) = table(self, frows, format!("@\"mfields.{q}\""));
        let (mm, nm) = table(self, mrows, format!("@\"mmethods.{q}\""));
        (mf, nf, mm, nm)
    }

    fn emit_statics(&mut self, id: ClassId) {
        let c = self.p.class(id).clone();
        let q = sym(&c.qname);
        for (i, f) in c.fields.iter().enumerate() {
            if !f.is_static {
                continue;
            }
            let r = self.repr(&f.ty);
            if r == Repr::Unit {
                continue;
            }
            let s = self.static_sym(FieldRef { class: id, index: i as u32 });
            let _ = writeln!(self.globals, "{s} = internal global {} {}", r.ll(), r.zero());
            let _ = writeln!(self.globals, "@\"set.{q}.{}\" = internal global i8 0", sym(&f.name));
            if r == Repr::Ref {
                self.static_roots.push(s);
            }
        }
        if !self.is_trivial(id) {
            let (name, len) = self.byte_string(&c.qname);
            let _ = writeln!(self.globals, "@\"init.{q}\" = internal global %InitRec {{ i32 0, i32 0, ptr @\"{q}.clinit\", ptr null, ptr {name}, i64 {len}, i64 0 }}");
        }
    }

    // ---- the whole program ----

    fn finish(mut self, entry: ClassId) -> Result<String, Vec<String>> {
        let p = self.p;
        let n = p.classes.len() as ClassId;
        for id in 0..n {
            self.emit_statics(id);
        }
        for id in 0..n {
            Func::class_bodies(&mut self, id);
        }
        for id in 0..n {
            self.emit_class(id);
        }
        Func::program_functions(&mut self, entry);
        self.emit_bridges();

        // The tables the runtime starts from.
        let classes: Vec<String> = (0..n).map(|id| self.class_info(id)).collect();
        let nclasses = classes.len();
        let classes = self.ptr_array(&classes);
        let strings: Vec<String> = (0..self.strings.len()).map(|i| format!("@str.{i}")).collect();
        let nstrings = strings.len();
        let strings = self.ptr_array_global(&strings, "@cleat.strings");
        let cached: Vec<String> = self.cached.keys().map(|id| format!("%CachedType {{ ptr @te.{id}, ptr @tc.{id} }}")).collect();
        let ncached = cached.len();
        let cached_sym = if cached.is_empty() {
            "null".to_string()
        } else {
            let _ = writeln!(self.globals, "@cleat.cached = private constant [{ncached} x %CachedType] [{}]", cached.join(", "));
            "@cleat.cached".to_string()
        };
        let roots = self.static_roots.clone();
        let nroots = roots.len();
        let roots = self.ptr_array_global(&roots, "@cleat.roots");
        let mut wk: Vec<String> = vec!["null".into(); 32];
        for id in 0..n {
            let machine = self.class_repr[id as usize].kind();
            let k = if machine != 0 { machine } else { special_kind(p, id) };
            if k != 0 {
                wk[k as usize] = self.class_info(id);
            }
        }
        let wk = self.ptr_array_global(&wk, "@cleat.wk");
        let _ = writeln!(
            self.globals,
            "@cleat_program = constant %Program {{ i32 {nclasses}, i32 {nstrings}, ptr {classes}, ptr {strings}, i32 {ncached}, i32 {nroots}, ptr {cached_sym}, ptr {roots}, ptr {wk}, ptr @cleat_make_exception, ptr @cleat_thread_run, ptr @cleat_suppress }}"
        );

        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        let mut out = String::new();
        out.push_str(HEADER);
        for l in &self.layouts {
            if let Some(l) = l {
                let fields: Vec<&str> = l.fields.iter().map(|(_, r, _)| r.ll()).collect();
                let mut all = vec!["ptr", "i64"];
                all.extend(fields);
                let _ = writeln!(out, "{} = type {{ {} }}", l.ty, all.join(", "));
            }
        }
        out.push('\n');
        for d in &self.decls {
            // The header declares what every program uses.
            if !HEADER.lines().any(|l| l == d) {
                out.push_str(d);
                out.push('\n');
            }
        }
        out.push('\n');
        out.push_str(&self.globals);
        out.push('\n');
        out.push_str(&self.funcs);
        Ok(out)
    }

    fn ptr_array_global(&mut self, items: &[String], name: &str) -> String {
        if items.is_empty() {
            return "null".into();
        }
        let body: Vec<String> = items.iter().map(|i| format!("ptr {i}")).collect();
        let _ = writeln!(self.globals, "{name} = private constant [{} x ptr] [{}]", items.len(), body.join(", "));
        name.to_string()
    }
}

const HEADER: &str = r#"; Generated by cleatc.
%Ctx = type { ptr, ptr, i32, i32, ptr }
%MethodEntry = type { i32, i32, ptr }
%FieldInfo = type { ptr, i32, i32, i32, i32 }
%AnnInfo = type { ptr, ptr }
%ParamInfo = type { ptr, i32, i32, ptr }
%MemberInfo = type { ptr, i32, i32, i32, i32, ptr, ptr, ptr, ptr, ptr }
%ClassInfo = type { ptr, i32, i32, i32, i32, ptr, ptr, i32, i32, ptr, i32, i32, ptr, ptr, i32, i32, ptr, ptr, i32, i32, ptr, ptr, i32, i32, ptr }
%TypeExpr = type { i32, i32, ptr, i32, i32, ptr, i32, i32, ptr, ptr }
%CachedType = type { ptr, ptr }
%InitRec = type { i32, i32, ptr, ptr, ptr, i64, i64 }
%Program = type { i32, i32, ptr, ptr, i32, i32, ptr, ptr, ptr, ptr, ptr, ptr }
@cl_unit = external global ptr
declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
declare ptr @cl_start(ptr)
declare ptr @cl_args(ptr)
declare i32 @cl_finish(ptr)
declare void @cl_poll(ptr)
declare ptr @cl_new(ptr, ptr)
declare ptr @cl_box(ptr, i32, i64)
declare ptr @cl_array_new(ptr, ptr, i64)
declare void @cl_array_fill(ptr, ptr)
declare void @cl_index_fail(ptr, i64, i64)
declare ptr @cl_type_eval(ptr, ptr)
declare ptr @cl_env(ptr, ptr)
declare ptr @cl_type_arg(ptr, ptr, i32)
declare i8 @cl_instance_of(ptr, ptr)
declare ptr @cl_cast(ptr, ptr, ptr)
declare ptr @cl_cast_class(ptr, ptr, ptr)
declare ptr @cl_class_of(ptr)
declare ptr @cl_lookup(ptr, i32)
declare ptr @cl_static_req(ptr, i32)
declare void @cl_class_init(ptr, ptr)
declare void @cl_static_unassigned(ptr, ptr, i64)
declare ptr @cl_rational_const(ptr, ptr, ptr, i64)
declare void @cl_mirror_fail(ptr, i32, ptr, i64)
declare void @cl_foreign_enter(ptr)
declare void @cl_foreign_leave(ptr)
declare ptr @cl_Array_get_Int(ptr, ptr, i64)
declare void @cl_Array_set_Int_Object(ptr, ptr, i64, ptr)
declare i8 @cl_Object_equals_Object(ptr, ptr, ptr)
"#;

/// Compiles a checked program to LLVM IR. `entry` is the class whose `main` runs.
pub fn emit(p: &mut Program, entry: ClassId) -> Result<String, Vec<String>> {
    let n = p.classes.len();
    let w = p.wk.clone();
    // How each class's values are held.
    let mut class_repr = vec![Repr::Ref; n];
    for (id, r) in [
        (w.int8, Repr::I8),
        (w.int16, Repr::I16),
        (w.int32, Repr::I32),
        (w.int, Repr::I64),
        (w.uint8, Repr::U8),
        (w.uint16, Repr::U16),
        (w.uint32, Repr::U32),
        (w.uint64, Repr::U64),
        (w.float32, Repr::F32),
        (w.float64, Repr::F64),
        (w.boolean, Repr::Bool),
        (w.char_, Repr::Char),
        (w.pointer, Repr::Ptr),
        (w.unit, Repr::Unit),
    ] {
        class_repr[id as usize] = r;
    }

    // Selectors: one for each method that overrides nothing and can be overridden.
    let mut roots: HashMap<MethodRef, MethodRef> = HashMap::new();
    let mut selectors: HashMap<MethodRef, u32> = HashMap::new();
    let mut req_selectors: HashMap<MethodRef, u32> = HashMap::new();
    let mut next = 3u32;
    for id in 0..n as ClassId {
        let c = p.class(id);
        let iface = c.is_interface();
        for (i, m) in c.methods.iter().enumerate() {
            let mref = MethodRef { class: id, index: i as u32 };
            if m.static_requirement {
                req_selectors.insert(mref, req_selectors.len() as u32);
                continue;
            }
            if m.is_static {
                continue;
            }
            let root = m.overrides.iter().copied().find(|o| p.method(*o).overrides.is_empty() && !p.method(*o).is_static);
            match root {
                Some(r) => {
                    roots.insert(mref, r);
                }
                None => {
                    let is_virtual = m.is_virtual(iface);
                    if is_virtual {
                        let fixed = if id == w.object {
                            match m.name.as_str() {
                                "equals" => Some(0),
                                "hashCode" => Some(1),
                                "toString" => Some(2),
                                _ => None,
                            }
                        } else {
                            None
                        };
                        let s = fixed.unwrap_or_else(|| {
                            next += 1;
                            next - 1
                        });
                        selectors.insert(mref, s);
                    }
                }
            }
        }
    }

    // The table of each class that has instances.
    let mut vtables: Vec<Vec<(MethodRef, MethodRef)>> = vec![Vec::new(); n];
    let mut reqs: Vec<Vec<(MethodRef, MethodRef)>> = vec![Vec::new(); n];
    for id in 0..n as ClassId {
        let c = p.class(id).clone();
        if c.is_interface() || c.is_abstract {
            continue;
        }
        if id == w.null {
            // `Null` stands outside the hierarchy and answers the three methods of `Object`.
            for (i, m) in c.methods.iter().enumerate() {
                let root = p.class(w.object).methods.iter().position(|o| o.name == m.name && o.params.len() == m.params.len());
                if let Some(r) = root {
                    vtables[id as usize].push((MethodRef { class: w.object, index: r as u32 }, MethodRef { class: id, index: i as u32 }));
                }
            }
            continue;
        }
        let mut classes = vec![id];
        classes.extend(decl::all_supertypes(p, id).iter().filter_map(|t| t.class_id()));
        for cid in classes {
            for i in 0..p.class(cid).methods.len() as u32 {
                let r = MethodRef { class: cid, index: i };
                if selectors.contains_key(&r) {
                    if let ImplOf::Found(f) = decl::impl_of(p, id, r) {
                        if !p.method(f).is_abstract {
                            vtables[id as usize].push((r, f));
                        }
                    }
                } else if req_selectors.contains_key(&r) {
                    if let Some(f) = decl::find_static_impl(p, id, r) {
                        reqs[id as usize].push((r, f));
                    }
                }
            }
        }
    }

    // Layouts.
    let mut layouts: Vec<Option<Layout>> = Vec::new();
    for id in 0..n as ClassId {
        let c = p.class(id);
        if c.is_interface() {
            layouts.push(None);
            continue;
        }
        // The superclass chain, root first.
        let mut chain = vec![id];
        let mut cur = c.superclass.as_ref().and_then(|t| t.class_id());
        while let Some(s) = cur {
            chain.push(s);
            cur = p.class(s).superclass.as_ref().and_then(|t| t.class_id());
        }
        chain.reverse();
        let mut fields = Vec::new();
        let mut index = 2u32;
        for cid in chain {
            for (i, f) in p.class(cid).fields.iter().enumerate() {
                if f.is_static {
                    continue;
                }
                let t = &f.ty;
                let r = if t.nullable {
                    Repr::Ref
                } else {
                    match &t.ty {
                        Ty::Class(k, _) => class_repr[*k as usize],
                        _ => Repr::Ref,
                    }
                };
                if r == Repr::Unit {
                    continue;
                }
                fields.push((FieldRef { class: cid, index: i as u32 }, r, index));
                index += 1;
            }
        }
        layouts.push(Some(Layout { ty: format!("%\"L.{}\"", sym(&c.qname)), fields }));
    }

    // A class whose initialization does nothing needs no guard.
    let mut trivial = vec![true; n];
    for id in 0..n {
        let c = &p.classes[id];
        trivial[id] = c.static_init_checked.as_ref().map(|b| b.stmts.is_empty()).unwrap_or(true);
    }
    loop {
        let mut changed = false;
        for id in 0..n {
            if trivial[id] {
                if let Some(s) = p.classes[id].superclass.as_ref().and_then(|t| t.class_id()) {
                    if !trivial[s as usize] {
                        trivial[id] = false;
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    let e = Emitter {
        p,
        globals: String::new(),
        funcs: String::new(),
        decls: BTreeSet::new(),
        class_repr,
        layouts,
        trivial,
        roots,
        selectors,
        req_selectors,
        vtables,
        reqs,
        bridges: BTreeMap::new(),
        strings: HashMap::new(),
        bytes: HashMap::new(),
        texprs: HashMap::new(),
        cached: BTreeMap::new(),
        rationals: HashMap::new(),
        static_roots: Vec::new(),
        thunks: 0,
        errors: Vec::new(),
    };
    let _ = TypeKind::Class;
    e.finish(entry)
}
