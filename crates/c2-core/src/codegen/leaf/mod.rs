//! One file per leaf shape. Each is a self-contained pattern match over
//! an exact op stream, independent of the others, sharing only the
//! encoders and the two helpers in [`super::select`]. A new leaf rung is
//! a new file here plus one arm in `select_function` — and nothing else.

pub mod addr;
pub mod compare;
pub mod float;
pub mod load;
pub mod store;

pub use addr::*;
pub use compare::*;
pub use float::*;
pub use load::*;
pub use store::*;
