//! The Cleat runtime library: storage, types at run time, and the bodies of the
//! prelude's `@Intrinsic` methods. A compiled program is linked with it.
//!
//! # Safety
//!
//! Every `unsafe` function here has the same contract, so it is written once. Its
//! callers are compiled code and the runtime itself. A `ctx` is the context of the
//! thread that makes the call. An `Obj` is null only where the parameter's type in the
//! prelude is `@Nullable`, and otherwise points to a live object of the class the
//! function's name gives. Every other pointer is to data laid out as `abi` describes.

#![allow(clippy::missing_safety_doc)]

pub mod abi;
pub mod gc;
pub mod host;
pub mod num;
pub mod object;
pub mod types;
