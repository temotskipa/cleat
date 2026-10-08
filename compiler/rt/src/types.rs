//! Types at run time (section 7.7): a class with its type arguments, interned, so two
//! types are the same exactly when their descriptors are the same pointer.

use crate::abi::*;
use std::collections::HashMap;
use std::sync::atomic::AtomicPtr;
use std::sync::{Mutex, OnceLock};

pub struct TypeDesc {
    /// Null for a wildcard.
    pub class: *const ClassInfo,
    pub args: Vec<*const TypeDesc>,
    pub nullable: bool,
    /// The qualifiers other than `@Nullable`, sorted by address.
    pub quals: Vec<*const ClassInfo>,
    /// 0 for a type, 1 for `?` and `? extends`, 2 for `? super`.
    pub wild: u8,
    /// The bound of a wildcard, or null for `?`.
    pub bound: *const TypeDesc,
    /// For an array type, the kind of its elements: 0 for pointers.
    pub elem_kind: u32,
    /// Every supertype, with its arguments, found on first use.
    pub supers: OnceLock<Vec<*const TypeDesc>>,
    pub class_obj: AtomicPtr<Header>,
    pub name: OnceLock<String>,
}

unsafe impl Send for TypeDesc {}
unsafe impl Sync for TypeDesc {}

#[derive(PartialEq, Eq, Hash)]
struct Key {
    class: usize,
    args: Vec<usize>,
    nullable: bool,
    quals: Vec<usize>,
    wild: u8,
    bound: usize,
}

static TABLE: Mutex<Option<HashMap<Key, usize>>> = Mutex::new(None);

pub type Td = *const TypeDesc;

pub fn intern(class: *const ClassInfo, args: Vec<Td>, nullable: bool, mut quals: Vec<*const ClassInfo>, wild: u8, bound: Td) -> Td {
    quals.sort();
    quals.dedup();
    let key = Key {
        class: class as usize,
        args: args.iter().map(|a| *a as usize).collect(),
        nullable,
        quals: quals.iter().map(|q| *q as usize).collect(),
        wild,
        bound: bound as usize,
    };
    let mut guard = TABLE.lock().unwrap_or_else(|e| e.into_inner());
    let table = guard.get_or_insert_with(HashMap::new);
    if let Some(p) = table.get(&key) {
        return *p as Td;
    }
    let elem_kind = unsafe {
        if !class.is_null() && (*class).kind == K_ARRAY && args.len() == 1 {
            let e = &*args[0];
            if !e.nullable && e.wild == 0 && !e.class.is_null() && is_machine((*e.class).kind) {
                (*e.class).kind
            } else {
                0
            }
        } else {
            0
        }
    };
    let td = Box::into_raw(Box::new(TypeDesc {
        class,
        args,
        nullable,
        quals,
        wild,
        bound,
        elem_kind,
        supers: OnceLock::new(),
        class_obj: AtomicPtr::new(std::ptr::null_mut()),
        name: OnceLock::new(),
    }));
    table.insert(key, td as usize);
    td
}

/// The type of a class that has no type parameters.
pub fn simple(class: *const ClassInfo) -> Td {
    intern(class, Vec::new(), false, Vec::new(), 0, std::ptr::null())
}

/// The prelude class of a kind.
pub fn wk(kind: u32) -> *const ClassInfo {
    unsafe { *crate::gc::program().wk.add(kind as usize) }
}

pub fn wk_type(kind: u32) -> Td {
    static CACHE: [OnceLock<usize>; K_COUNT] = [const { OnceLock::new() }; K_COUNT];
    *CACHE[kind as usize].get_or_init(|| simple(wk(kind)) as usize) as Td
}

pub unsafe fn eval(e: *const TypeExpr, env: *const Td) -> Td {
    unsafe {
        let e = &*e;
        let quals: Vec<*const ClassInfo> = slice(e.quals, e.nquals).to_vec();
        match e.tag {
            TE_VAR => {
                let base = &**env.add(e.index as usize);
                if e.nullable == 0 && quals.is_empty() {
                    return base;
                }
                // The qualifiers written on the use join those of the argument.
                let mut all = base.quals.clone();
                all.extend(quals);
                intern(base.class, base.args.clone(), base.nullable || e.nullable != 0, all, base.wild, base.bound)
            }
            TE_WILD => {
                let bound = if e.bound.is_null() { std::ptr::null() } else { eval(e.bound, env) };
                intern(std::ptr::null(), Vec::new(), false, Vec::new(), if e.lower != 0 { 2 } else { 1 }, bound)
            }
            _ => {
                let args: Vec<Td> = slice(e.args, e.nargs).iter().map(|a| eval(*a, env)).collect();
                intern(e.class, args, e.nullable != 0, quals, 0, std::ptr::null())
            }
        }
    }
}

impl TypeDesc {
    pub fn class(&self) -> &'static ClassInfo {
        unsafe { &*self.class }
    }

    /// Every supertype of this class type, itself first.
    pub fn supers(&'static self) -> &'static [Td] {
        self.supers.get_or_init(|| unsafe {
            let mut all: Vec<Td> = vec![self as Td];
            let mut i = 0;
            while i < all.len() {
                let t = &*all[i];
                for s in t.class().supers() {
                    let up = eval(*s, t.args.as_ptr());
                    if !all.contains(&up) {
                        all.push(up);
                    }
                }
                i += 1;
            }
            all
        })
    }

    /// The supertype that instantiates `class`.
    pub fn super_at(&'static self, class: *const ClassInfo) -> Option<&'static TypeDesc> {
        if self.class == class {
            return Some(self);
        }
        // Every class but `Null` is a subclass of `Object`, an interface included.
        if unsafe { (*class).kind } == K_OBJECT {
            return if self.class().kind == K_NULL { None } else { Some(unsafe { &*wk_type(K_OBJECT) }) };
        }
        if self.class().is(F_ANNOTATION) && (self.class().is(F_REFINES) || self.class().is(F_WIDENS)) {
            return None;
        }
        self.supers().iter().map(|t| unsafe { &**t }).find(|t| t.class == class)
    }

    /// The name section 7.7 gives the type.
    pub fn name(&'static self) -> &'static str {
        self.name.get_or_init(|| unsafe {
            let mut s = String::new();
            if self.wild != 0 {
                s.push('?');
                if !self.bound.is_null() {
                    s.push_str(if self.wild == 2 { " super " } else { " extends " });
                    s.push_str((*self.bound).name());
                }
                return s;
            }
            let mut quals: Vec<&str> = self.quals.iter().map(|q| (**q).name()).collect();
            if self.nullable {
                quals.push("cleat.Nullable");
            }
            quals.sort();
            for q in quals {
                s.push('@');
                s.push_str(q);
                s.push(' ');
            }
            s.push_str(self.class().name());
            if !self.args.is_empty() {
                s.push('<');
                for (i, a) in self.args.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str((**a).name());
                }
                s.push('>');
            }
            s
        })
    }
}

/// Whether the refinement `low` is `high` or below it.
unsafe fn refines(low: *const ClassInfo, high: *const ClassInfo) -> bool {
    unsafe {
        if low == high {
            return true;
        }
        (*low).supers().iter().any(|above| refines((**above).class, high))
    }
}

/// The subtype rule of chapters 5, 7 and 8, for two types.
pub unsafe fn is_subtype(s: &'static TypeDesc, t: &'static TypeDesc) -> bool {
    unsafe {
        if std::ptr::eq(s, t) {
            return true;
        }
        if s.wild != 0 || t.wild != 0 {
            return false;
        }
        if s.nullable && !t.nullable {
            return false;
        }
        for q in &t.quals {
            if (**q).is(F_REFINES) && !s.quals.iter().any(|sq| (**sq).is(F_REFINES) && refines(*sq, *q)) {
                return false;
            }
        }
        for q in &s.quals {
            if (**q).is(F_WIDENS) && !t.quals.contains(q) {
                return false;
            }
        }
        let Some(up) = s.super_at(t.class) else { return false };
        let c = t.class();
        for i in 0..t.args.len() {
            let (sa, ta) = (&*up.args[i], &*t.args[i]);
            let variance = if c.variances.is_null() { 0 } else { *c.variances.add(i) };
            if !contains(ta, sa, variance) {
                return false;
            }
        }
        true
    }
}

/// Whether the argument `outer` of a supertype contains the argument `inner`.
unsafe fn contains(outer: &'static TypeDesc, inner: &'static TypeDesc, variance: u8) -> bool {
    unsafe {
        if std::ptr::eq(outer, inner) {
            return true;
        }
        // `?` with no bound contains every argument.
        if outer.wild == 1 && outer.bound.is_null() {
            return true;
        }
        match variance {
            1 => {
                // out: the upper limits decide.
                let o = if outer.wild == 1 { &*outer.bound } else { outer };
                if inner.wild == 1 {
                    return !inner.bound.is_null() && is_subtype(&*inner.bound, o);
                }
                inner.wild == 0 && outer.wild != 2 && is_subtype(inner, o)
            }
            2 => {
                let o = if outer.wild == 2 { &*outer.bound } else { outer };
                if inner.wild == 2 {
                    return is_subtype(o, &*inner.bound);
                }
                inner.wild == 0 && outer.wild != 1 && is_subtype(o, inner)
            }
            _ => match outer.wild {
                0 => false,
                1 => {
                    let u = &*outer.bound;
                    match inner.wild {
                        0 => is_subtype(inner, u),
                        1 => !inner.bound.is_null() && is_subtype(&*inner.bound, u),
                        _ => false,
                    }
                }
                _ => {
                    let l = &*outer.bound;
                    match inner.wild {
                        0 => is_subtype(l, inner),
                        2 => is_subtype(l, &*inner.bound),
                        _ => false,
                    }
                }
            },
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_type_eval(e: *const TypeExpr, env: *const Td) -> Td {
    unsafe { eval(e, env) }
}

/// The type arguments an object's class has for `class`, which it is or inherits from.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_env(o: Obj, class: *const ClassInfo) -> *const Td {
    unsafe {
        let td = &*(*o).td;
        match td.super_at(class) {
            Some(up) => up.args.as_ptr(),
            None => std::ptr::null(),
        }
    }
}

/// One type argument of an object's class as `class` sees it: the unknown type that a
/// wildcard stood for (section 7.4).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_type_arg(o: Obj, class: *const ClassInfo, index: u32) -> Td {
    unsafe { *cl_env(o, class).add(index as usize) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_instance_of(o: Obj, t: Td) -> u8 {
    unsafe {
        if o.is_null() {
            return 0;
        }
        let t = &*t;
        // The test is on the class: the qualifiers of the type itself are not a value's.
        let bare = if t.nullable || !t.quals.is_empty() { &*intern(t.class, t.args.clone(), false, Vec::new(), 0, std::ptr::null()) } else { t };
        is_subtype(&*(*o).td, bare) as u8
    }
}

/// The checked cast of section 4.7. A failed cast leaves the exception in the context.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_cast(ctx: *mut Ctx, o: Obj, t: Td) -> Obj {
    unsafe {
        if o.is_null() {
            if (*t).nullable {
                return o;
            }
            crate::object::raise(ctx, X_CLASS_CAST, &format!("null is not a {}", (*t).name()));
            return o;
        }
        if cl_instance_of(o, t) == 0 {
            crate::object::raise(ctx, X_CLASS_CAST, &format!("a {} is not a {}", (*(*o).td).name(), (*t).name()));
        }
        o
    }
}

/// The table entry of a class for a selector, or null.
pub unsafe fn find(entries: *const MethodEntry, n: u32, selector: u32) -> *const u8 {
    unsafe {
        let list = slice(entries, n);
        match list.binary_search_by_key(&selector, |e| e.selector) {
            Ok(i) => list[i].f,
            Err(_) => std::ptr::null(),
        }
    }
}

/// The body a virtual call runs for a receiver. A null receiver has the class `Null`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_lookup(o: Obj, selector: u32) -> *const u8 {
    unsafe {
        let c = if o.is_null() { &*wk(K_NULL) } else { (*(*o).td).class() };
        let f = find(c.methods, c.nmethods, selector);
        if f.is_null() {
            eprintln!("cleat: internal error: {} has no method for selector {selector}", c.name());
            std::process::exit(70);
        }
        f
    }
}

/// The static method a type argument's class has for a requirement (section 7.6).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_static_req(t: Td, selector: u32) -> *const u8 {
    unsafe {
        let c = (*t).class();
        let f = find(c.reqs, c.nreqs, selector);
        if f.is_null() {
            eprintln!("cleat: internal error: {} has no static method for requirement {selector}", c.name());
            std::process::exit(70);
        }
        f
    }
}
