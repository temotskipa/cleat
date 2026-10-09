//! Objects the runtime stores itself (strings, arrays, boxed machine values), the
//! bodies of `Object`'s methods, class initialization, and startup.

#![allow(non_snake_case)]

use crate::abi::*;
use crate::gc::{self, Temp};
use crate::types::{self, Td, TypeDesc};
use std::sync::atomic::Ordering;

pub const SEL_EQUALS: u32 = 0;
pub const SEL_HASH: u32 = 1;
pub const SEL_TO_STRING: u32 = 2;

/// The one `Unit` value.
#[unsafe(no_mangle)]
pub static mut cl_unit: Obj = std::ptr::null_mut();

pub unsafe fn at<T>(o: Obj, offset: usize) -> *mut T {
    unsafe { (o as *mut u8).add(offset) as *mut T }
}

pub unsafe fn class_of(o: Obj) -> &'static ClassInfo {
    unsafe { (*(*o).td).class() }
}

// ---- raising ----

/// Raises one of the prelude's exceptions with a message, unless one is already raised.
pub unsafe fn raise(ctx: *mut Ctx, kind: u32, message: &str) {
    unsafe { raise_with(ctx, kind, message, std::ptr::null_mut()) }
}

pub unsafe fn raise_with(ctx: *mut Ctx, kind: u32, message: &str, cause: Obj) {
    unsafe {
        if !(*ctx).exc.is_null() {
            return;
        }
        let _c = Temp::new(ctx, cause);
        let m = new_str(ctx, message);
        if m.is_null() {
            eprintln!("cleat: out of memory");
            std::process::exit(1);
        }
        let _t = Temp::new(ctx, m);
        let e = (gc::program().make_exception)(ctx, kind, m, cause);
        if (*ctx).exc.is_null() {
            (*ctx).exc = e;
        }
    }
}

unsafe fn checked(ctx: *mut Ctx, o: Obj) -> Obj {
    unsafe {
        if o.is_null() {
            raise(ctx, X_OUT_OF_MEMORY, "no storage for an allocation");
        }
        o
    }
}

// ---- allocation ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_new(ctx: *mut Ctx, td: Td) -> Obj {
    unsafe { checked(ctx, gc::alloc(ctx, td, (*td).class().size as usize)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_box(ctx: *mut Ctx, kind: u32, bits: u64) -> Obj {
    unsafe {
        let o = checked(ctx, gc::alloc(ctx, types::wk_type(kind), 24));
        if !o.is_null() {
            *at::<u64>(o, BODY) = bits;
        }
        o
    }
}

pub unsafe fn bits(o: Obj) -> u64 {
    unsafe { *at::<u64>(o, BODY) }
}

// ---- strings ----

pub unsafe fn chars(o: Obj) -> &'static [u32] {
    unsafe { std::slice::from_raw_parts(at::<u32>(o, ELEMS), *at::<i64>(o, BODY) as usize) }
}

pub unsafe fn new_string(ctx: *mut Ctx, text: &[u32]) -> Obj {
    unsafe {
        let o = checked(ctx, gc::alloc(ctx, types::wk_type(K_STRING), ELEMS + text.len() * 4));
        if !o.is_null() {
            *at::<i64>(o, BODY) = text.len() as i64;
            std::ptr::copy_nonoverlapping(text.as_ptr(), at::<u32>(o, ELEMS), text.len());
        }
        o
    }
}

pub unsafe fn new_str(ctx: *mut Ctx, s: &str) -> Obj {
    let text: Vec<u32> = s.chars().map(|c| c as u32).collect();
    unsafe { new_string(ctx, &text) }
}

pub unsafe fn to_rust(o: Obj) -> String {
    unsafe { chars(o).iter().map(|c| char::from_u32(*c).unwrap_or('\u{FFFD}')).collect() }
}

// ---- arrays ----

pub fn array_type(elem: Td) -> Td {
    types::intern(types::wk(K_ARRAY), vec![elem], false, Vec::new(), 0, std::ptr::null())
}

pub unsafe fn len(a: Obj) -> i64 {
    unsafe { *at::<i64>(a, BODY) }
}

/// A new array of `n` elements, each zero or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_array_new(ctx: *mut Ctx, td: Td, n: i64) -> Obj {
    unsafe {
        if n < 0 {
            raise(ctx, X_ARITHMETIC, &format!("an array length of {n}"));
            return std::ptr::null_mut();
        }
        let size = (n as usize).checked_mul(elem_size((*td).elem_kind)).and_then(|b| b.checked_add(ELEMS));
        let o = match size {
            Some(size) if size < isize::MAX as usize / 2 => gc::alloc(ctx, td, size),
            _ => std::ptr::null_mut(),
        };
        let o = checked(ctx, o);
        if !o.is_null() {
            *at::<i64>(o, BODY) = n;
        }
        o
    }
}

/// Stores one pointer in every element.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_array_fill(a: Obj, v: Obj) {
    unsafe {
        let elems = at::<Obj>(a, ELEMS);
        for i in 0..len(a) as usize {
            *elems.add(i) = v;
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_index_fail(ctx: *mut Ctx, index: i64, length: i64) {
    unsafe { raise(ctx, X_INDEX, &format!("index {index}, length {length}")) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Array_length(_ctx: *mut Ctx, a: Obj) -> i64 {
    unsafe { len(a) }
}

unsafe fn load_elem(a: Obj, kind: u32, i: usize) -> u64 {
    unsafe {
        let p = at::<u8>(a, ELEMS).add(i * elem_size(kind));
        match elem_size(kind) {
            1 => *p as u64,
            2 => *(p as *const u16) as u64,
            4 => *(p as *const u32) as u64,
            _ => *(p as *const u64),
        }
    }
}

unsafe fn store_elem(a: Obj, kind: u32, i: usize, v: u64) {
    unsafe {
        let p = at::<u8>(a, ELEMS).add(i * elem_size(kind));
        match elem_size(kind) {
            1 => *p = v as u8,
            2 => *(p as *mut u16) = v as u16,
            4 => *(p as *mut u32) = v as u32,
            _ => *(p as *mut u64) = v,
        }
    }
}

/// `get` on an array seen through a type parameter: the element as an object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Array_get_Int(ctx: *mut Ctx, a: Obj, i: i64) -> Obj {
    unsafe {
        let n = len(a);
        if i < 0 || i >= n {
            cl_index_fail(ctx, i, n);
            return std::ptr::null_mut();
        }
        let kind = (*(*a).td).elem_kind;
        if kind == 0 {
            *at::<Obj>(a, ELEMS).add(i as usize)
        } else {
            cl_box(ctx, kind, load_elem(a, kind, i as usize))
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Array_set_Int_Object(ctx: *mut Ctx, a: Obj, i: i64, v: Obj) {
    unsafe {
        let n = len(a);
        if i < 0 || i >= n {
            cl_index_fail(ctx, i, n);
            return;
        }
        let kind = (*(*a).td).elem_kind;
        if kind == 0 {
            *at::<Obj>(a, ELEMS).add(i as usize) = v;
        } else {
            store_elem(a, kind, i as usize, bits(v));
        }
    }
}

/// `Array.<T>unfilled(length)`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Array_unfilled_Int(ctx: *mut Ctx, targs: *const Td, n: i64) -> Obj {
    unsafe { cl_array_new(ctx, array_type(*targs), n) }
}

/// A copy of a value of a class the program declares, with its value fields copied too.
unsafe fn copy_value(ctx: *mut Ctx, o: Obj) -> Obj {
    unsafe {
        if o.is_null() {
            return o;
        }
        let c = class_of(o);
        if !c.is(F_VALUE) || c.kind != 0 {
            return o;
        }
        let size = c.size as usize;
        let _t = Temp::new(ctx, o);
        let n = checked(ctx, gc::alloc(ctx, (*o).td, size));
        if n.is_null() {
            return n;
        }
        std::ptr::copy_nonoverlapping(at::<u8>(o, BODY), at::<u8>(n, BODY), size - BODY);
        let _n = Temp::new(ctx, n);
        for f in slice(c.fields, c.nfields) {
            if f.kind == 0 {
                let inner = copy_value(ctx, *at::<Obj>(o, f.offset as usize));
                *at::<Obj>(n, f.offset as usize) = inner;
            }
        }
        n
    }
}

/// An array of fresh copies of the elements of an array. What a foreign function writes
/// goes to the copies, so no other holder of an element sees it (section 4.11).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_ffi_fresh(ctx: *mut Ctx, a: Obj) -> Obj {
    unsafe {
        let _a = Temp::new(ctx, a);
        let fresh = cl_array_new(ctx, (*a).td, len(a));
        if fresh.is_null() {
            return fresh;
        }
        let _f = Temp::new(ctx, fresh);
        for i in 0..len(a) as usize {
            let copy = copy_value(ctx, *at::<Obj>(a, ELEMS).add(i));
            *at::<Obj>(fresh, ELEMS).add(i) = copy;
        }
        fresh
    }
}

// ---- calling the three methods every value has ----

pub unsafe fn v_equals(ctx: *mut Ctx, a: Obj, b: Obj) -> bool {
    unsafe {
        let f: extern "C" fn(*mut Ctx, Obj, Obj) -> u8 = std::mem::transmute(types::cl_lookup(a, SEL_EQUALS));
        f(ctx, a, b) != 0
    }
}

pub unsafe fn v_hash(ctx: *mut Ctx, a: Obj) -> i64 {
    unsafe {
        let f: extern "C" fn(*mut Ctx, Obj) -> i64 = std::mem::transmute(types::cl_lookup(a, SEL_HASH));
        f(ctx, a)
    }
}

pub unsafe fn v_to_string(ctx: *mut Ctx, a: Obj) -> Obj {
    unsafe {
        let f: extern "C" fn(*mut Ctx, Obj) -> Obj = std::mem::transmute(types::cl_lookup(a, SEL_TO_STRING));
        f(ctx, a)
    }
}

// ---- Object ----

fn machine_equal(kind: u32, x: u64, y: u64) -> bool {
    match kind {
        K_F64 => {
            let (a, b) = (f64::from_bits(x), f64::from_bits(y));
            a == b || (a.is_nan() && b.is_nan())
        }
        K_F32 => {
            let (a, b) = (f32::from_bits(x as u32), f32::from_bits(y as u32));
            a == b || (a.is_nan() && b.is_nan())
        }
        _ => x == y,
    }
}

unsafe fn load_field(o: Obj, f: &FieldInfo) -> u64 {
    unsafe {
        let p = at::<u8>(o, f.offset as usize);
        match elem_size(f.kind) {
            1 => *p as u64,
            2 => *(p as *const u16) as u64,
            4 => *(p as *const u32) as u64,
            _ => *(p as *const u64),
        }
    }
}

/// Whether field `i` is the last of a value class and holds a reference. The default
/// `equals`, `hashCode` and `identical` follow that field in a loop rather than a call
/// when its value answers them the same way, so a chain of values as long as the digits
/// of a large `BigInt` does not fill the stack.
fn is_tail(fields: &[FieldInfo], i: usize) -> bool {
    i + 1 == fields.len() && fields[i].kind == 0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Object_equals_Object(ctx: *mut Ctx, a: Obj, b: Obj) -> u8 {
    unsafe {
        let (mut a, mut b) = (a, b);
        'chain: loop {
            if a.is_null() || b.is_null() {
                return (a.is_null() && b.is_null()) as u8;
            }
            if a == b {
                return 1;
            }
            if (*a).td != (*b).td {
                return 0;
            }
            let c = class_of(a);
            let same = match c.kind {
                k if is_machine(k) => machine_equal(k, bits(a), bits(b)),
                K_STRING => chars(a) == chars(b),
                _ if c.is(F_VALUE) => {
                    let fields = slice(c.fields, c.nfields);
                    let mut all = true;
                    for (i, f) in fields.iter().enumerate() {
                        let ok = if f.kind == 0 {
                            let (x, y) = (*at::<Obj>(a, f.offset as usize), *at::<Obj>(b, f.offset as usize));
                            if is_tail(fields, i) && types::cl_lookup(x, SEL_EQUALS) == cl_Object_equals_Object as *const u8 {
                                (a, b) = (x, y);
                                continue 'chain;
                            }
                            v_equals(ctx, x, y)
                        } else {
                            machine_equal(f.kind, load_field(a, f), load_field(b, f))
                        };
                        if !ok || !(*ctx).exc.is_null() {
                            all = false;
                            break;
                        }
                    }
                    all
                }
                _ => false,
            };
            return same as u8;
        }
    }
}

fn machine_hash(kind: u32, x: u64) -> i64 {
    match kind {
        K_F64 => {
            let f = f64::from_bits(x);
            if f.is_nan() {
                0x7ff8_0000_0000_0000
            } else if f == 0.0 {
                0
            } else {
                x as i64
            }
        }
        K_F32 => {
            let f = f32::from_bits(x as u32);
            if f.is_nan() {
                0x7fc0_0000
            } else if f == 0.0 {
                0
            } else {
                x as u32 as i64
            }
        }
        _ => x as i64,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Object_hashCode(ctx: *mut Ctx, a: Obj) -> i64 {
    unsafe {
        // A value's hash adds its last field's hash to the rest, so the values the loop
        // has passed leave a sum to add to the hash of the one it is on.
        let mut a = a;
        let mut passed = 0i64;
        'chain: loop {
            if a.is_null() {
                return passed;
            }
            let c = class_of(a);
            let h = match c.kind {
                k if is_machine(k) => machine_hash(k, bits(a)),
                K_STRING => chars(a).iter().fold(0i64, |h, ch| h.wrapping_mul(31).wrapping_add(*ch as i64)),
                _ if c.is(F_VALUE) => {
                    let fields = slice(c.fields, c.nfields);
                    let mut h = 17i64;
                    for (i, f) in fields.iter().enumerate() {
                        let part = if f.kind == 0 {
                            let x = *at::<Obj>(a, f.offset as usize);
                            if is_tail(fields, i) && types::cl_lookup(x, SEL_HASH) == cl_Object_hashCode as *const u8 {
                                passed = passed.wrapping_add(h.wrapping_mul(31));
                                a = x;
                                continue 'chain;
                            }
                            v_hash(ctx, x)
                        } else {
                            machine_hash(f.kind, load_field(a, f))
                        };
                        h = h.wrapping_mul(31).wrapping_add(part);
                    }
                    h
                }
                _ => (a as usize >> 4) as i64,
            };
            return passed.wrapping_add(h);
        }
    }
}

/// The text of a machine number other than a float.
pub fn show_machine(kind: u32, x: u64) -> String {
    match kind {
        K_I8 => (x as i8).to_string(),
        K_I16 => (x as i16).to_string(),
        K_I32 => (x as i32).to_string(),
        K_I64 => (x as i64).to_string(),
        K_U8 | K_U16 | K_U32 | K_U64 => x.to_string(),
        K_BOOL => if x != 0 { "true".into() } else { "false".into() },
        K_CHAR => char::from_u32(x as u32).unwrap_or('\u{FFFD}').to_string(),
        _ => format!("Pointer({x:#x})"),
    }
}

/// The `toString` of a machine value. A float's is written in the language, so a float
/// is boxed and asked. `None` when that raised.
pub unsafe fn machine_text(ctx: *mut Ctx, kind: u32, x: u64) -> Option<String> {
    unsafe {
        if kind != K_F32 && kind != K_F64 {
            return Some(show_machine(kind, x));
        }
        let boxed = cl_box(ctx, kind, x);
        if boxed.is_null() {
            return None;
        }
        let _t = Temp::new(ctx, boxed);
        let t = v_to_string(ctx, boxed);
        if !(*ctx).exc.is_null() || t.is_null() {
            return None;
        }
        Some(to_rust(t))
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Object_toString(ctx: *mut Ctx, a: Obj) -> Obj {
    unsafe {
        if a.is_null() {
            return new_str(ctx, "null");
        }
        let c = class_of(a);
        match c.kind {
            k if is_machine(k) => match machine_text(ctx, k, bits(a)) {
                Some(text) => new_str(ctx, &text),
                None => std::ptr::null_mut(),
            },
            K_STRING => a,
            K_CLASS => new_str(ctx, (**at::<Td>(a, BODY)).name()),
            _ if c.is(F_LAMBDA) => new_str(ctx, &format!("{}@{:x}", c.name(), a as usize >> 4)),
            _ if c.is(F_VALUE) => {
                let mut s = String::from(c.simple_name());
                s.push('(');
                for (i, f) in slice(c.fields, c.nfields).iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    if f.kind == 0 {
                        let t = v_to_string(ctx, *at::<Obj>(a, f.offset as usize));
                        if !(*ctx).exc.is_null() || t.is_null() {
                            return std::ptr::null_mut();
                        }
                        s.push_str(&to_rust(t));
                    } else {
                        match machine_text(ctx, f.kind, load_field(a, f)) {
                            Some(text) => s.push_str(&text),
                            None => return std::ptr::null_mut(),
                        }
                    }
                }
                s.push(')');
                new_str(ctx, &s)
            }
            _ => new_str(ctx, &format!("{}@{:x}", (*(*a).td).name(), a as usize >> 4)),
        }
    }
}

unsafe fn identical(a: Obj, b: Obj) -> bool {
    unsafe {
        let (mut a, mut b) = (a, b);
        loop {
            if a == b {
                return true;
            }
            if a.is_null() || b.is_null() || (*a).td != (*b).td {
                return false;
            }
            let c = class_of(a);
            match c.kind {
                k if is_machine(k) => return bits(a) == bits(b),
                K_STRING => return chars(a) == chars(b),
                _ if c.is(F_VALUE) => {
                    let fields = slice(c.fields, c.nfields);
                    let same = fields.iter().enumerate().all(|(i, f)| {
                        if is_tail(fields, i) {
                            true
                        } else if f.kind == 0 {
                            identical(*at::<Obj>(a, f.offset as usize), *at::<Obj>(b, f.offset as usize))
                        } else {
                            load_field(a, f) == load_field(b, f)
                        }
                    });
                    match fields.last() {
                        Some(f) if same && f.kind == 0 => {
                            (a, b) = (*at::<Obj>(a, f.offset as usize), *at::<Obj>(b, f.offset as usize));
                        }
                        _ => return same,
                    }
                }
                _ => return false,
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Object_identical_Object(_ctx: *mut Ctx, a: Obj, b: Obj) -> u8 {
    unsafe { identical(a, b) as u8 }
}

pub unsafe fn same_identity(a: Obj, b: Obj) -> bool {
    unsafe { identical(a, b) }
}

/// The class object of a type: one for each type, never collected.
pub unsafe fn class_object(td: Td) -> Obj {
    unsafe {
        let t = &*td;
        let have = t.class_obj.load(Ordering::Acquire);
        if !have.is_null() {
            return have;
        }
        let o = gc::alloc_static(types::wk_type(K_CLASS), 24);
        *at::<Td>(o, BODY) = td;
        match t.class_obj.compare_exchange(std::ptr::null_mut(), o, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => o,
            Err(first) => first,
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_class_of(td: Td) -> Obj {
    unsafe {
        let t = &*td;
        // A class literal names the class: qualifiers of the type are not part of it.
        let bare = if t.nullable || !t.quals.is_empty() { types::intern(t.class, t.args.clone(), false, Vec::new(), 0, std::ptr::null()) } else { td };
        class_object(bare)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Object_getClass(_ctx: *mut Ctx, a: Obj) -> Obj {
    unsafe { class_object((*a).td) }
}

// ---- String ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_length(_ctx: *mut Ctx, s: Obj) -> i64 {
    unsafe { len(s) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_get_Int(ctx: *mut Ctx, s: Obj, i: i64) -> u32 {
    unsafe {
        let c = chars(s);
        if i < 0 || i as usize >= c.len() {
            raise(ctx, X_INDEX, &format!("index {i}, length {}", c.len()));
            return 0;
        }
        c[i as usize]
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_substring_Int_Int(ctx: *mut Ctx, s: Obj, begin: i64, end: i64) -> Obj {
    unsafe {
        let n = len(s);
        if begin < 0 || end > n || begin > end {
            raise(ctx, X_INDEX, &format!("range {begin} to {end}, length {n}"));
            return std::ptr::null_mut();
        }
        let part: Vec<u32> = chars(s)[begin as usize..end as usize].to_vec();
        new_string(ctx, &part)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_join_String(ctx: *mut Ctx, s: Obj, t: Obj) -> Obj {
    unsafe {
        let mut all: Vec<u32> = Vec::with_capacity(chars(s).len() + chars(t).len());
        all.extend_from_slice(chars(s));
        all.extend_from_slice(chars(t));
        new_string(ctx, &all)
    }
}

pub unsafe fn byte_slice(a: Obj) -> &'static [u8] {
    unsafe { std::slice::from_raw_parts(at::<u8>(a, ELEMS), len(a) as usize) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_fromChars_CharArray_Int(ctx: *mut Ctx, a: Obj, count: i64) -> Obj {
    unsafe {
        let n = len(a);
        if count < 0 || count > n {
            raise(ctx, X_INDEX, &format!("count {count}, length {n}"));
            return std::ptr::null_mut();
        }
        let part: Vec<u32> = std::slice::from_raw_parts(at::<u32>(a, ELEMS), count as usize).to_vec();
        new_string(ctx, &part)
    }
}

unsafe fn map_chars(ctx: *mut Ctx, s: Obj, upper: bool) -> Obj {
    unsafe {
        let text = to_rust(s);
        new_str(ctx, &if upper { text.to_uppercase() } else { text.to_lowercase() })
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_toLowerCase(ctx: *mut Ctx, s: Obj) -> Obj {
    unsafe { map_chars(ctx, s, false) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_String_toUpperCase(ctx: *mut Ctx, s: Obj) -> Obj {
    unsafe { map_chars(ctx, s, true) }
}

// ---- Char, Pointer ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_from_Int(ctx: *mut Ctx, code: i64) -> u32 {
    unsafe {
        match u32::try_from(code).ok().and_then(char::from_u32) {
            Some(c) => c as u32,
            None => {
                raise(ctx, X_ARITHMETIC, &format!("{code} is not a Unicode scalar"));
                0
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_code(_ctx: *mut Ctx, c: u32) -> i64 {
    c as i64
}

fn ch(c: u32) -> char {
    char::from_u32(c).unwrap_or('\u{FFFD}')
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_isLetter(_ctx: *mut Ctx, c: u32) -> u8 {
    ch(c).is_alphabetic() as u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_isDigit(_ctx: *mut Ctx, c: u32) -> u8 {
    ch(c).is_ascii_digit() as u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_isWhitespace(_ctx: *mut Ctx, c: u32) -> u8 {
    ch(c).is_whitespace() as u8
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_toLowerCase(_ctx: *mut Ctx, c: u32) -> u32 {
    let mut it = ch(c).to_lowercase();
    match (it.next(), it.next()) {
        (Some(one), None) => one as u32,
        _ => c,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Char_toUpperCase(_ctx: *mut Ctx, c: u32) -> u32 {
    let mut it = ch(c).to_uppercase();
    match (it.next(), it.next()) {
        (Some(one), None) => one as u32,
        _ => c,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_Pointer_zero(_ctx: *mut Ctx) -> *mut u8 {
    std::ptr::null_mut()
}

// ---- class initialization (section 9.7) ----

static INIT: std::sync::Mutex<()> = std::sync::Mutex::new(());
static INIT_DONE: std::sync::Condvar = std::sync::Condvar::new();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_class_init(ctx: *mut Ctx, rec: *mut InitRec) {
    unsafe {
        let name = || std::str::from_utf8_unchecked(std::slice::from_raw_parts((*rec).name, (*rec).name_len as usize)).to_string();
        let me = (*ctx).id;
        let mut guard = INIT.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            match (*rec).state.load(Ordering::Acquire) {
                2 => return,
                3 => {
                    let cause = (*rec).failure;
                    drop(guard);
                    raise_with(ctx, X_CLASS_INIT, &format!("the initialization of {} failed", name()), cause);
                    return;
                }
                1 if (*rec).owner == me => return,
                1 => {
                    // Another thread is initializing the class: wait for it.
                    drop(guard);
                    gc::blocking(ctx, || std::thread::sleep(std::time::Duration::from_millis(1)));
                    guard = INIT.lock().unwrap_or_else(|e| e.into_inner());
                }
                _ => break,
            }
        }
        (*rec).state.store(1, Ordering::Release);
        (*rec).owner = me;
        drop(guard);
        ((*rec).run)(ctx);
        let _guard = INIT.lock().unwrap_or_else(|e| e.into_inner());
        if (*ctx).exc.is_null() {
            (*rec).state.store(2, Ordering::Release);
        } else {
            (*rec).failure = (*ctx).exc;
            gc::world().pinned.push((*ctx).exc);
            (*rec).state.store(3, Ordering::Release);
        }
        INIT_DONE.notify_all();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_static_unassigned(ctx: *mut Ctx, name: *const u8, n: u64) {
    unsafe {
        let name = std::str::from_utf8_unchecked(std::slice::from_raw_parts(name, n as usize));
        raise(ctx, X_CLASS_INIT, &format!("the static field {name} is read before it is assigned"));
    }
}

// ---- startup ----

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_start(program: *const Program) -> *mut Ctx {
    unsafe {
        gc::PROGRAM = program;
        if let Ok(v) = std::env::var("CLEAT_GC_STRESS") {
            gc::STRESS.store(v.parse().unwrap_or(1), Ordering::Relaxed);
        }
        let p = &*program;
        for c in slice(p.cached, p.ncached) {
            *c.slot = types::eval(c.expr, std::ptr::null());
        }
        let string = types::wk_type(K_STRING);
        for s in slice(p.strings, p.nstrings) {
            (**s).td = string;
            (**s).gc = GC_STATIC;
        }
        cl_unit = gc::alloc_static(types::wk_type(K_UNIT), BODY);
        gc::new_ctx_sized(gc::main_stack())
    }
}

/// A call found no room for its frame (section 9.10). This is not an exception.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_stack_overflow() {
    use std::io::Write;
    let _ = std::io::stdout().flush();
    eprintln!("cleat: a call cannot be given stack space");
    std::process::exit(1);
}

/// The arguments the host passed, without the program's own name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_args(ctx: *mut Ctx) -> Obj {
    unsafe {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let a = cl_array_new(ctx, array_type(types::wk_type(K_STRING)), args.len() as i64);
        let _t = Temp::new(ctx, a);
        for (i, s) in args.iter().enumerate() {
            let o = new_str(ctx, s);
            *at::<Obj>(a, ELEMS).add(i) = o;
        }
        a
    }
}

/// Ends the program after `main`: the status, with an uncaught exception reported.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cl_finish(ctx: *mut Ctx) -> i32 {
    unsafe {
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let e = (*ctx).exc;
        if e.is_null() {
            return 0;
        }
        (*ctx).exc = std::ptr::null_mut();
        let _t = Temp::new(ctx, e);
        let text = v_to_string(ctx, e);
        if (*ctx).exc.is_null() && !text.is_null() {
            eprintln!("{}", to_rust(text));
        } else {
            eprintln!("{}", (*(*e).td).name());
        }
        1
    }
}

pub fn type_desc_name(td: &'static TypeDesc) -> &'static str {
    td.name()
}
