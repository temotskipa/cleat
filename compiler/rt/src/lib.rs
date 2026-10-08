//! The Cleat runtime library: storage, types at run time, and the bodies of the
//! prelude's `@Intrinsic` methods. A compiled program is linked with it.

pub mod abi;
pub mod gc;
pub mod host;
pub mod num;
pub mod object;
pub mod types;
