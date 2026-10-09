//! The layouts that compiled code and the runtime share. Every struct here is emitted
//! by the compiler as constant data or read by the code it generates, so a change here
//! is a change in `compiler/src/emit`.

use crate::types::TypeDesc;
use std::sync::atomic::AtomicU32;

/// The start of every object.
#[repr(C)]
pub struct Header {
    pub td: *const TypeDesc,
    pub gc: u64,
}

pub type Obj = *mut Header;

pub const GC_MARK: u64 = 1;
/// An object that is never collected: a literal, a class object, the `Unit` value.
pub const GC_STATIC: u64 = 2;

/// The offset of the first field, of a box's payload, and of an array's length.
pub const BODY: usize = 16;
/// The offset of the first element of an array and of the first scalar of a string.
pub const ELEMS: usize = 24;

/// One compiled function's pointers, linked from the context.
#[repr(C)]
pub struct Frame {
    pub prev: *mut Frame,
    pub n: u64,
    // `n` slots follow.
}

/// What every compiled function receives first: the state of the running thread.
#[repr(C)]
pub struct Ctx {
    /// The exception being propagated, or null.
    pub exc: Obj,
    pub top: *mut Frame,
    /// Nonzero when the thread must stop for the collector at its next poll.
    pub poll: AtomicU32,
    pub _pad: u32,
    /// The `Thread` object of the running thread, or null before `Thread.current`.
    pub thread: Obj,
    /// The lowest address a call's frame may have. Below it, the stack is full.
    pub stack_limit: usize,
    // The rest is the runtime's own.
    pub temps: Vec<Obj>,
    pub allocated: Vec<Obj>,
    pub bytes: usize,
    pub safe: bool,
    pub id: u64,
}

#[repr(C)]
pub struct MethodEntry {
    pub selector: u32,
    pub _pad: u32,
    pub f: *const u8,
}

#[repr(C)]
pub struct FieldInfo {
    pub name: *const u8,
    pub name_len: u32,
    pub offset: u32,
    /// 0 for a pointer, or the machine kind of the field.
    pub kind: u32,
    pub _pad: u32,
}

#[repr(C)]
pub struct AnnInfo {
    pub class: *const ClassInfo,
    /// Builds a new value of the annotation as it was written.
    pub make: extern "C" fn(*mut Ctx) -> Obj,
}

#[repr(C)]
pub struct ParamInfo {
    pub name: *const u8,
    pub name_len: u32,
    pub nanns: u32,
    pub anns: *const AnnInfo,
}

pub const M_STATIC: u32 = 1;

/// A field or a method that carries an annotation.
#[repr(C)]
pub struct MemberInfo {
    pub name: *const u8,
    pub name_len: u32,
    pub flags: u32,
    pub nanns: u32,
    pub nparams: u32,
    pub anns: *const AnnInfo,
    pub params: *const ParamInfo,
    pub get: Option<extern "C" fn(*mut Ctx, Obj) -> Obj>,
    pub set: Option<extern "C" fn(*mut Ctx, Obj, Obj)>,
    pub invoke: Option<extern "C" fn(*mut Ctx, Obj, Obj) -> Obj>,
}

pub const F_INTERFACE: u32 = 1;
pub const F_VALUE: u32 = 2;
pub const F_ABSTRACT: u32 = 4;
pub const F_ENUM: u32 = 8;
pub const F_ANNOTATION: u32 = 16;
pub const F_LAMBDA: u32 = 32;
/// The first supertype is the superclass.
pub const F_SUPERCLASS: u32 = 64;
/// An annotation declared `@Inherited`.
pub const F_INHERITED: u32 = 128;
pub const F_REFINES: u32 = 256;
pub const F_WIDENS: u32 = 512;

// The kinds of a class whose instances the runtime stores itself.
pub const K_I8: u32 = 1;
pub const K_I16: u32 = 2;
pub const K_I32: u32 = 3;
pub const K_I64: u32 = 4;
pub const K_U8: u32 = 5;
pub const K_U16: u32 = 6;
pub const K_U32: u32 = 7;
pub const K_U64: u32 = 8;
pub const K_F32: u32 = 9;
pub const K_F64: u32 = 10;
pub const K_BOOL: u32 = 11;
pub const K_CHAR: u32 = 12;
pub const K_POINTER: u32 = 13;
pub const K_STRING: u32 = 20;
pub const K_ARRAY: u32 = 21;
pub const K_CLASS: u32 = 23;
pub const K_UNIT: u32 = 24;
pub const K_NULL: u32 = 25;
pub const K_LOCK: u32 = 26;
pub const K_CONDITION: u32 = 27;
pub const K_THREAD: u32 = 28;
pub const K_OBJECT: u32 = 29;
pub const K_COUNT: usize = 32;

pub fn is_machine(kind: u32) -> bool {
    (K_I8..=K_POINTER).contains(&kind)
}

/// The size of an array element of a kind. Kind 0 is a pointer.
pub fn elem_size(kind: u32) -> usize {
    match kind {
        K_I8 | K_U8 | K_BOOL => 1,
        K_I16 | K_U16 => 2,
        K_I32 | K_U32 | K_F32 | K_CHAR => 4,
        _ => 8,
    }
}

#[repr(C)]
pub struct ClassInfo {
    pub name: *const u8,
    pub name_len: u32,
    pub flags: u32,
    pub kind: u32,
    pub nparams: u32,
    /// For each type parameter: 0 invariant, 1 `out`, 2 `in`.
    pub variances: *const u8,
    /// For each type parameter its first bound, or null.
    pub bounds: *const *const TypeExpr,
    pub nsupers: u32,
    pub size: u32,
    /// The direct supertypes over the class's own parameters. For a refinement, the
    /// refinements directly above it.
    pub supers: *const *const TypeExpr,
    pub nrefs: u32,
    pub nmethods: u32,
    /// The offsets of the pointer fields.
    pub refs: *const u32,
    /// The body for each selector, sorted by selector.
    pub methods: *const MethodEntry,
    pub nreqs: u32,
    pub nfields: u32,
    /// The static methods that meet the requirements of interfaces, by selector.
    pub reqs: *const MethodEntry,
    /// The instance fields of a value class, in order.
    pub fields: *const FieldInfo,
    pub nanns: u32,
    pub nmfields: u32,
    pub anns: *const AnnInfo,
    pub mfields: *const MemberInfo,
    pub nmmethods: u32,
    pub _pad: u32,
    pub mmethods: *const MemberInfo,
}

impl ClassInfo {
    pub fn name(&self) -> &'static str {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.name, self.name_len as usize)) }
    }

    pub fn simple_name(&self) -> &'static str {
        let n = self.name();
        n.rsplit('.').next().unwrap_or(n)
    }

    pub fn is(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    pub fn supers(&self) -> &'static [*const TypeExpr] {
        unsafe { slice(self.supers, self.nsupers) }
    }
}

pub unsafe fn slice<T>(p: *const T, n: u32) -> &'static [T] {
    if p.is_null() || n == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(p, n as usize) }
    }
}

pub const TE_CLASS: u32 = 0;
pub const TE_VAR: u32 = 1;
pub const TE_WILD: u32 = 2;

/// A type as the program wrote it, over the type parameters in scope.
#[repr(C)]
pub struct TypeExpr {
    pub tag: u32,
    pub nullable: u32,
    pub class: *const ClassInfo,
    /// For a variable, its index among the type arguments in scope.
    pub index: u32,
    pub nargs: u32,
    pub args: *const *const TypeExpr,
    pub nquals: u32,
    /// For a wildcard: 1 when its bound is a lower bound.
    pub lower: u32,
    pub quals: *const *const ClassInfo,
    /// For a wildcard, its bound, or null for `?`.
    pub bound: *const TypeExpr,
}

#[repr(C)]
pub struct CachedType {
    pub expr: *const TypeExpr,
    pub slot: *mut *const TypeDesc,
}

/// The record a class's initialization runs under.
#[repr(C)]
pub struct InitRec {
    /// 0 not started, 1 running, 2 done, 3 failed.
    pub state: AtomicU32,
    pub _pad: u32,
    pub run: extern "C" fn(*mut Ctx),
    pub failure: Obj,
    pub name: *const u8,
    pub name_len: u64,
    pub owner: u64,
}

// The exceptions the runtime raises, as the compiler's `cleat_make_exception` numbers them.
pub const X_ARITHMETIC: u32 = 0;
pub const X_CLASS_CAST: u32 = 1;
pub const X_INDEX: u32 = 2;
pub const X_ILLEGAL_ARGUMENT: u32 = 3;
pub const X_ILLEGAL_STATE: u32 = 4;
pub const X_ILLEGAL_ACCESS: u32 = 5;
pub const X_CLASS_INIT: u32 = 6;
pub const X_OUT_OF_MEMORY: u32 = 7;
pub const X_CANCELLATION: u32 = 8;
pub const X_IO: u32 = 9;

/// What the compiler tells the runtime about one program.
#[repr(C)]
pub struct Program {
    pub nclasses: u32,
    pub nstrings: u32,
    pub classes: *const *const ClassInfo,
    /// The string literals, whose headers the runtime completes at startup.
    pub strings: *const Obj,
    pub ncached: u32,
    pub nroots: u32,
    /// The types with no type variable, each with the slot its descriptor goes in.
    pub cached: *const CachedType,
    /// The addresses of the static fields that hold pointers.
    pub roots: *const *mut Obj,
    /// The prelude classes the runtime knows, indexed by kind.
    pub wk: *const *const ClassInfo,
    pub make_exception: extern "C" fn(*mut Ctx, u32, Obj, Obj) -> Obj,
    pub thread_run: extern "C" fn(*mut Ctx, Obj),
    pub suppress: extern "C" fn(*mut Ctx, Obj, Obj),
}
