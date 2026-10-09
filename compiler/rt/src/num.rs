//! The bodies of the numeric classes' `@Intrinsic` methods (chapter 6): the machine's
//! arithmetic and bit operations on `Int` and `UInt64`, IEEE 754 on the float classes,
//! and the conversions between machine numbers. Each is exported under the name the
//! compiler derives: `cl_<Class>_<method>[_<ParamClass>...]`. The other methods of the
//! numeric classes, `Rational` whole, are written in the prelude.

#![allow(non_snake_case)]

use crate::abi::*;
use crate::object::raise;

// ---- one value of any machine number class ----

pub enum Num {
    I(i128),
    F32(f32),
    F64(f64),
}

unsafe fn fail(ctx: *mut Ctx, n: &Num, class: &str) {
    unsafe {
        let shown = match n {
            Num::I(v) => Some(v.to_string()),
            Num::F32(f) => crate::object::machine_text(ctx, K_F32, f.to_bits() as u64),
            Num::F64(f) => crate::object::machine_text(ctx, K_F64, f.to_bits()),
        };
        // Without a text, printing the float raised, and that exception stands.
        if let Some(shown) = shown {
            raise(ctx, X_ARITHMETIC, &format!("{shown} is not a value of {class}"));
        }
    }
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

fn nearest_f32(n: &Num) -> f32 {
    match n {
        Num::F32(f) => *f,
        other => nearest_f64(other) as f32,
    }
}

// Reads a parameter of each machine number class as a `Num`.
trait Load {
    fn load(self) -> Num;
}
macro_rules! load_int {
    ($($t:ty),*) => { $( impl Load for $t { fn load(self) -> Num { Num::I(self as i128) } } )* };
}
load_int!(i8, i16, i32, i64, u8, u16, u32, u64);
impl Load for f32 {
    fn load(self) -> Num {
        Num::F32(self)
    }
}
impl Load for f64 {
    fn load(self) -> Num {
        Num::F64(self)
    }
}

// Makes a value of each machine number class from a `Num`, exactly or not at all.
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

/// `C.from(S)` for each listed source class, which converts exactly or raises.
macro_rules! from {
    ($C:ident, $ct:ty, $m:ident; $(($S:ident, $st:ty)),*) => {
        pub mod $m {
            use super::*;
            $(
                #[unsafe(export_name = concat!("cl_", stringify!($C), "_from_", stringify!($S)))]
                pub unsafe extern "C" fn $S(ctx: *mut Ctx, v: $st) -> $ct {
                    unsafe { <$ct as Exact>::exact(ctx, Load::load(v)) }
                }
            )*
        }
    };
}

/// `C.nearest(S)` for each listed source class, which rounds.
macro_rules! nearest {
    ($C:ident, $ct:ty, $m:ident, $near:ident; $(($S:ident, $st:ty)),*) => {
        pub mod $m {
            use super::*;
            $(
                #[unsafe(export_name = concat!("cl_", stringify!($C), "_nearest_", stringify!($S)))]
                pub unsafe extern "C" fn $S(_ctx: *mut Ctx, v: $st) -> $ct {
                    $near(&Load::load(v))
                }
            )*
        }
    };
}

// `Int` takes every narrower integer as it is, and checks the rest. `UInt64` and the
// narrower classes check an `Int`. The float classes convert an `Int` and a `UInt64`
// in one step, because two steps would round twice.
from!(Int, i64, int_from; (Int8, i8), (Int16, i16), (Int32, i32), (UInt8, u8), (UInt16, u16), (UInt32, u32), (UInt64, u64), (Float64, f64));
from!(UInt64, u64, uint64_from; (Int, i64), (Float64, f64));
from!(Int8, i8, int8_from; (Int, i64));
from!(Int16, i16, int16_from; (Int, i64));
from!(Int32, i32, int32_from; (Int, i64));
from!(UInt8, u8, uint8_from; (Int, i64));
from!(UInt16, u16, uint16_from; (Int, i64));
from!(UInt32, u32, uint32_from; (Int, i64));
from!(Float64, f64, float64_from; (Int, i64), (UInt64, u64), (Float32, f32));
from!(Float32, f32, float32_from; (Int, i64), (UInt64, u64), (Float64, f64));
nearest!(Float64, f64, float64_nearest, nearest_f64; (Int, i64), (UInt64, u64), (Float32, f32));
nearest!(Float32, f32, float32_nearest, nearest_f32; (Int, i64), (UInt64, u64), (Float64, f64));

// ---- the integer classes ----

unsafe fn overflow<T: Default>(ctx: *mut Ctx, what: &str, class: &str) -> T {
    unsafe { raise(ctx, X_ARITHMETIC, &format!("{what} is not a value of {class}")) };
    T::default()
}

/// The value of a narrower class with the low bits of an `Int`. The narrow classes
/// compute in `Int` and come back through this or through `from`.
macro_rules! wrapping {
    ($(($C:ident, $t:ty, $m:ident)),*) => { $(
        #[unsafe(export_name = concat!("cl_", stringify!($C), "_wrapping_Int"))]
        pub unsafe extern "C" fn $m(_ctx: *mut Ctx, v: i64) -> $t {
            v as $t
        }
    )* };
}
wrapping!((Int8, i8, int8_wrapping), (Int16, i16, int16_wrapping), (Int32, i32, int32_wrapping), (UInt8, u8, uint8_wrapping), (UInt16, u16, uint16_wrapping), (UInt32, u32, uint32_wrapping));

/// The machine operations of `Int` and `UInt64`.
macro_rules! int_class {
    ($C:ident, $t:ty, $m:ident) => {
        pub mod $m {
            use super::*;
            const NAME: &str = stringify!($C);
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
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_lessThan_", stringify!($C)))]
            pub unsafe extern "C" fn less_than(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a < b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_truncatingDiv_", stringify!($C)))]
            pub unsafe extern "C" fn truncating_div(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                if b == 0 {
                    unsafe { raise(ctx, X_ARITHMETIC, "division by zero") };
                    return 0;
                }
                match a.checked_div(b) {
                    Some(q) => q,
                    None => unsafe { overflow(ctx, &format!("{a} divided by {b}"), NAME) },
                }
            }
            // The minimum divided by negative one has no quotient, and a zero remainder.
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_truncatingRem_", stringify!($C)))]
            pub unsafe extern "C" fn truncating_rem(ctx: *mut Ctx, a: $t, b: $t) -> $t {
                if b == 0 {
                    unsafe { raise(ctx, X_ARITHMETIC, "division by zero") };
                    return 0;
                }
                a.checked_rem(b).unwrap_or(0)
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
        }
    };
}

int_class!(Int, i64, int64);
int_class!(UInt64, u64, uint64);

// ---- the float classes ----

macro_rules! float_class {
    ($C:ident, $t:ty, $m:ident) => {
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
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_lessThan_", stringify!($C)))]
            pub unsafe extern "C" fn less_than(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a < b) as u8
            }
            #[unsafe(export_name = concat!("cl_", stringify!($C), "_atMost_", stringify!($C)))]
            pub unsafe extern "C" fn at_most(_ctx: *mut Ctx, a: $t, b: $t) -> u8 {
                (a <= b) as u8
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
        }
    };
}

float_class!(Float32, f32, float32);
float_class!(Float64, f64, float64);
