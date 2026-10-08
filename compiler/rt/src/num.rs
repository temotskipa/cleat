//! The bodies of the numeric classes' `@Intrinsic` methods (chapter 6). Each is exported
//! under the name the compiler derives: `cl_<Class>_<method>[_<ParamClass>...]`.

#![allow(non_snake_case)]

use crate::abi::*;
use crate::object::{new_str, raise, to_rust};

// ---- one value of any numeric class ----

pub enum Num {
    I(i128),
    F32(f32),
    F64(f64),
}

unsafe fn fail(ctx: *mut Ctx, n: &Num, class: &str) {
    let shown = match n {
        Num::I(v) => v.to_string(),
        Num::F32(f) => crate::object::show_float(*f as f64),
        Num::F64(f) => crate::object::show_float(*f),
    };
    unsafe { raise(ctx, X_ARITHMETIC, &format!("{shown} is not a value of {class}")) }
}

/// The integer a number equals, when it equals one.
fn exact_int(n: &Num) -> Option<i128> {
    match n {
        Num::I(v) => Some(*v),
        Num::F32(f) => exact_int(&Num::F64(*f as f64)),
        Num::F64(f) => {
            if f.is_finite() && f.fract() == 0.0 && f.abs() < 1.7e38 {
                Some(*f as i128)
            } else {
                None
            }
        }
    }
}

fn exact_f64(n: &Num) -> Option<f64> {
    match n {
        Num::I(v) => {
            let f = *v as f64;
            if f as i128 == *v && f.abs() < 1.7e38 { Some(f) } else { None }
        }
        Num::F32(f) => Some(*f as f64),
        Num::F64(f) => Some(*f),
    }
}

fn exact_f32(n: &Num) -> Option<f32> {
    if let Num::F32(f) = n {
        return Some(*f);
    }
    let d = exact_f64(n)?;
    let f = d as f32;
    if (f as f64 == d) || d.is_nan() { Some(f) } else { None }
}

fn nearest_f64(n: &Num) -> f64 {
    match n {
        Num::I(v) => *v as f64,
        Num::F32(f) => *f as f64,
        Num::F64(f) => *f,
    }
}

// Reads a parameter of each numeric class as a `Num`.
trait Load {
    unsafe fn load(self) -> Num;
}
macro_rules! load_int {
    ($($t:ty),*) => { $( impl Load for $t { unsafe fn load(self) -> Num { Num::I(self as i128) } } )* };
}
load_int!(i8, i16, i32, i64, u8, u16, u32, u64);
impl Load for f32 {
    unsafe fn load(self) -> Num {
        Num::F32(self)
    }
}
impl Load for f64 {
    unsafe fn load(self) -> Num {
        Num::F64(self)
    }
}

// Makes a value of each numeric class from a `Num`, exactly or not at all.
trait Exact: Sized {
    unsafe fn exact(ctx: *mut Ctx, n: Num) -> Self;
}
macro_rules! exact_int_impl {
    ($(($t:ty, $name:literal)),*) => { $(
        impl Exact for $t {
            unsafe fn exact(ctx: *mut Ctx, n: Num) -> $t {
                match exact_int(&n).and_then(|v| <$t>::try_from(v).ok()) {
                    Some(v) => v,
                    None => {
                        unsafe { fail(ctx, &n, $name) };
                        0
                    }
                }
            }
        }
    )* };
}
exact_int_impl!((i8, "Int8"), (i16, "Int16"), (i32, "Int32"), (i64, "Int"), (u8, "UInt8"), (u16, "UInt16"), (u32, "UInt32"), (u64, "UInt64"));
impl Exact for f32 {
    unsafe fn exact(ctx: *mut Ctx, n: Num) -> f32 {
        match exact_f32(&n) {
            Some(v) => v,
            None => {
                unsafe { fail(ctx, &n, "Float32") };
                0.0
            }
        }
    }
}
impl Exact for f64 {
    unsafe fn exact(ctx: *mut Ctx, n: Num) -> f64 {
        match exact_f64(&n) {
            Some(v) => v,
            None => {
                unsafe { fail(ctx, &n, "Float64") };
                0.0
            }
        }
    }
}

/// `C.from(S)` for every pair of numeric classes, and `nearest` for the float classes.
macro_rules! conversions {
    ($C:ident, $ct:ty, $m:ident, $nearest:expr) => {
        pub mod $m {
            use super::*;
            conversions!(@from $C, $ct, $nearest; (Int8, i8), (Int16, i16), (Int32, i32), (Int, i64), (UInt8, u8), (UInt16, u16),
                (UInt32, u32), (UInt64, u64), (Float32, f32), (Float64, f64));
        }
    };
    (@from $C:ident, $ct:ty, $nearest:expr; $(($S:ident, $st:ty)),*) => { $(
        pub mod $S {
            use super::*;
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_from_", stringify!($S)))]
            pub unsafe extern "C" fn from(ctx: *mut Ctx, v: $st) -> $ct {
                unsafe { <$ct as Exact>::exact(ctx, Load::load(v)) }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_nearest_", stringify!($S)))]
            pub unsafe extern "C" fn nearest(_ctx: *mut Ctx, v: $st) -> $ct {
                let near: fn(&Num) -> $ct = $nearest;
                unsafe { near(&Load::load(v)) }
            }
        }
    )* };
}

conversions!(Int8, i8, conv_i8, |_| 0);
conversions!(Int16, i16, conv_i16, |_| 0);
conversions!(Int32, i32, conv_i32, |_| 0);
conversions!(Int, i64, conv_i64, |_| 0);
conversions!(UInt8, u8, conv_u8, |_| 0);
conversions!(UInt16, u16, conv_u16, |_| 0);
conversions!(UInt32, u32, conv_u32, |_| 0);
conversions!(UInt64, u64, conv_u64, |_| 0);
conversions!(Float32, f32, conv_f32, |n| match n {
    Num::F32(f) => *f,
    other => nearest_f64(other) as f32,
});
conversions!(Float64, f64, conv_f64, nearest_f64);

// ---- the integer classes ----

unsafe fn overflow<T: Default>(ctx: *mut Ctx, what: &str, class: &str) -> T {
    unsafe { raise(ctx, X_ARITHMETIC, &format!("{what} is not a value of {class}")) };
    T::default()
}

macro_rules! int_class {
    ($C:ident, $t:ty, $m:ident, $kind:expr, $signed:expr) => {
        pub mod $m {
            use super::*;
            const NAME: &str = stringify!($C);
            // The value of the class with the low bits of an Int. The narrow classes
            // compute in Int and come back through this or through `from`.
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_wrapping_Int"))]
            pub unsafe extern "C" fn wrapping(_ctx: *mut Ctx, v: i64) -> $t {
                v as $t
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_plus_", stringify!($C)))]
            pub unsafe extern "C" fn plus(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                match a.checked_add(b) {
                    Some(v) => v,
                    None => unsafe { overflow(ctx, &format!("{a} + {b}"), NAME) },
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_minus_", stringify!($C)))]
            pub unsafe extern "C" fn minus(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                match a.checked_sub(b) {
                    Some(v) => v,
                    None => unsafe { overflow(ctx, &format!("{a} - {b}"), NAME) },
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_times_", stringify!($C)))]
            pub unsafe extern "C" fn times(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                match a.checked_mul(b) {
                    Some(v) => v,
                    None => unsafe { overflow(ctx, &format!("{a} * {b}"), NAME) },
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_negate"))]
            pub unsafe extern "C" fn negate(ctx: *mut Ctx, a: $t) -> $t {
                match (0 as $t).checked_sub(a) {
                    Some(v) => v,
                    None => unsafe { overflow(ctx, &format!("-({a})"), NAME) },
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_zero"))]
            pub unsafe extern "C" fn zero(_ctx: *mut Ctx) -> $t {
                0
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_one"))]
            pub unsafe extern "C" fn one(_ctx: *mut Ctx) -> $t {
                1
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_lessThan_", stringify!($C)))]
            pub unsafe extern "C" fn less_than(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a < b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_atMost_", stringify!($C)))]
            pub unsafe extern "C" fn at_most(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a <= b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_greaterThan_", stringify!($C)))]
            pub unsafe extern "C" fn greater_than(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a > b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_atLeast_", stringify!($C)))]
            pub unsafe extern "C" fn at_least(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a >= b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_compare_", stringify!($C)))]
            pub unsafe extern "C" fn compare(_ctx: *mut Ctx, a: $t, b: $t) -> i64 {
                (a > b) as i64 - (a < b) as i64
            }
            #[allow(unused_comparisons)]
            unsafe fn divide(ctx: *mut Ctx, a: $t, b: $t, floor: bool) -> Option<($t, $t)> {
                if b == 0 {
                    unsafe { raise(ctx, X_ARITHMETIC, "division by zero") };
                    return None;
                }
                let (mut q, mut r) = match (a.checked_div(b), a.checked_rem(b)) {
                    (Some(q), Some(r)) => (q, r),
                    // The minimum divided by negative one: no quotient, and a zero remainder.
                    _ => return Some((0, 0)),
                };
                if floor && r != 0 && ((r < 0) != (b < 0)) {
                    q = q.wrapping_sub(1);
                    r = r.wrapping_add(b);
                }
                Some((q, r))
            }
            unsafe fn quotient(ctx: *mut Ctx, a: $t, b: $t, floor: bool) -> $t {
                unsafe {
                    if b != 0 && a.checked_div(b).is_none() {
                        return overflow(ctx, &format!("{a} divided by {b}"), NAME);
                    }
                    divide(ctx, a, b, floor).map(|x| x.0).unwrap_or(0)
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_floorDiv_", stringify!($C)))]
            pub unsafe extern "C" fn floor_div(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                unsafe { quotient(ctx, a, b, true) }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_mod_", stringify!($C)))]
            pub unsafe extern "C" fn modulo(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                unsafe { divide(ctx, a, b, true).map(|x| x.1).unwrap_or(0) }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_truncatingDiv_", stringify!($C)))]
            pub unsafe extern "C" fn truncating_div(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                unsafe { quotient(ctx, a, b, false) }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_truncatingRem_", stringify!($C)))]
            pub unsafe extern "C" fn truncating_rem(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                unsafe { divide(ctx, a, b, false).map(|x| x.1).unwrap_or(0) }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_wrappingPlus_", stringify!($C)))]
            pub unsafe extern "C" fn wrapping_plus(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a.wrapping_add(b)
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_wrappingMinus_", stringify!($C)))]
            pub unsafe extern "C" fn wrapping_minus(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a.wrapping_sub(b)
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_wrappingTimes_", stringify!($C)))]
            pub unsafe extern "C" fn wrapping_times(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a.wrapping_mul(b)
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_and_", stringify!($C)))]
            pub unsafe extern "C" fn and(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a & b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_or_", stringify!($C)))]
            pub unsafe extern "C" fn or(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a | b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_xor_", stringify!($C)))]
            pub unsafe extern "C" fn xor(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a ^ b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_complement"))]
            pub unsafe extern "C" fn complement(_ctx: *mut Ctx, a: $t) -> $t {
                !a
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_shiftLeft_Int"))]
            pub unsafe extern "C" fn shift_left(ctx: *mut Ctx, a: $t, count: i64) -> $t {
                if count < 0 || count >= <$t>::BITS as i64 {
                    return unsafe { overflow(ctx, &format!("a shift count of {count}"), "a shift of this class") };
                }
                a << count
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_shiftRight_Int"))]
            pub unsafe extern "C" fn shift_right(ctx: *mut Ctx, a: $t, count: i64) -> $t {
                if count < 0 || count >= <$t>::BITS as i64 {
                    return unsafe { overflow(ctx, &format!("a shift count of {count}"), "a shift of this class") };
                }
                a >> count
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_parse_String"))]
            pub unsafe extern "C" fn parse(ctx: *mut Ctx, s: Obj) -> Obj {
                unsafe {
                    let text = to_rust(s);
                    let digits = text.strip_prefix('-').or_else(|| text.strip_prefix('+')).unwrap_or(&text);
                    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
                        return std::ptr::null_mut();
                    }
                    match text.parse::<$t>() {
                        Ok(v) => crate::object::cl_box(ctx, $kind, v as u64),
                        Err(_) => std::ptr::null_mut(),
                    }
                }
            }
            #[allow(dead_code)]
            const SIGNED: bool = $signed;
        }
    };
}

int_class!(Int8, i8, int8, K_I8, true);
int_class!(Int16, i16, int16, K_I16, true);
int_class!(Int32, i32, int32, K_I32, true);
int_class!(Int, i64, int64, K_I64, true);
int_class!(UInt8, u8, uint8, K_U8, false);
int_class!(UInt16, u16, uint16, K_U16, false);
int_class!(UInt32, u32, uint32, K_U32, false);
int_class!(UInt64, u64, uint64, K_U64, false);

// ---- the float classes ----

macro_rules! float_class {
    ($C:ident, $t:ty, $m:ident, $kind:expr) => {
        pub mod $m {
            use super::*;
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_plus_", stringify!($C)))]
            pub unsafe extern "C" fn plus(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a + b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_minus_", stringify!($C)))]
            pub unsafe extern "C" fn minus(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a - b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_times_", stringify!($C)))]
            pub unsafe extern "C" fn times(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a * b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_div_", stringify!($C)))]
            pub unsafe extern "C" fn div(_ctx: *mut Ctx, a: $t, b: $t) -> $t {
                a / b
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_negate"))]
            pub unsafe extern "C" fn negate(_ctx: *mut Ctx, a: $t) -> $t {
                -a
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_zero"))]
            pub unsafe extern "C" fn zero(_ctx: *mut Ctx) -> $t {
                0.0
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_one"))]
            pub unsafe extern "C" fn one(_ctx: *mut Ctx) -> $t {
                1.0
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_lessThan_", stringify!($C)))]
            pub unsafe extern "C" fn less_than(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a < b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_atMost_", stringify!($C)))]
            pub unsafe extern "C" fn at_most(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a <= b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_greaterThan_", stringify!($C)))]
            pub unsafe extern "C" fn greater_than(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a > b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_atLeast_", stringify!($C)))]
            pub unsafe extern "C" fn at_least(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a >= b) as u8
            }
            // A total order in which NaN is greatest and the two zeros are equal.
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_compare_", stringify!($C)))]
            pub unsafe extern "C" fn compare(_ctx: *mut Ctx, a: $t, b: $t) -> i64 {
                match (a.is_nan(), b.is_nan()) {
                    (true, true) => 0,
                    (true, false) => 1,
                    (false, true) => -1,
                    _ => (a > b) as i64 - (a < b) as i64,
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_totalOrder_", stringify!($C)))]
            pub unsafe extern "C" fn total_order(_ctx: *mut Ctx, a: $t, b: $t) -> i64 {
                match a.total_cmp(&b) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_floor"))]
            pub unsafe extern "C" fn floor(_ctx: *mut Ctx, a: $t) -> $t {
                a.floor()
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_ceil"))]
            pub unsafe extern "C" fn ceil(_ctx: *mut Ctx, a: $t) -> $t {
                a.ceil()
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_truncate"))]
            pub unsafe extern "C" fn truncate(_ctx: *mut Ctx, a: $t) -> $t {
                a.trunc()
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_round"))]
            pub unsafe extern "C" fn round(_ctx: *mut Ctx, a: $t) -> $t {
                a.round_ties_even()
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_sqrt"))]
            pub unsafe extern "C" fn sqrt(_ctx: *mut Ctx, a: $t) -> $t {
                a.sqrt()
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_abs"))]
            pub unsafe extern "C" fn abs(_ctx: *mut Ctx, a: $t) -> $t {
                a.abs()
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_isNaN"))]
            pub unsafe extern "C" fn is_nan(_ctx: *mut Ctx, a: $t) -> u8 {
                a.is_nan() as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_isInfinite"))]
            pub unsafe extern "C" fn is_infinite(_ctx: *mut Ctx, a: $t) -> u8 {
                a.is_infinite() as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_toFixed_Int"))]
            pub unsafe extern "C" fn to_fixed(ctx: *mut Ctx, a: $t, places: i64) -> Obj {
                unsafe {
                    if !(0..=1000).contains(&places) {
                        raise(ctx, X_ILLEGAL_ARGUMENT, &format!("{places} digits after the point"));
                        return std::ptr::null_mut();
                    }
                    new_str(ctx, &format!("{:.*}", places as usize, a))
                }
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_parse_String"))]
            pub unsafe extern "C" fn parse(ctx: *mut Ctx, s: Obj) -> Obj {
                unsafe {
                    let text = to_rust(s);
                    let ok = !text.is_empty() && text.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'));
                    match text.parse::<$t>() {
                        Ok(v) if ok => crate::object::cl_box(ctx, $kind, v.to_bits() as u64),
                        _ => std::ptr::null_mut(),
                    }
                }
            }
        }
    };
}

float_class!(Float32, f32, float32, K_F32);
float_class!(Float64, f64, float64, K_F64);
