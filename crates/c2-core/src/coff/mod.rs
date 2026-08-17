//! COFF `.obj` emitter for the MVP `add3` shape — builds the exact 5-section
//! Xbox 360 PPC object `c2.dll` produces for a single leaf int function with no
//! relocations. See the `SECTIONS` + `SYMBOLS` specs for the full byte-map.
//!
//! All COFF struct fields are **little-endian** (even though the `.text` PPC
//! payload and some watermark bytes are big-endian). The only input that varies
//! per compile is the `-Fo` output-path string (embedded in `.debug$S`
//! S_OBJNAME) and the `.text` bytes from codegen; everything else is a fixed
//! toolchain constant verified byte-identical across fixtures.
//!
//! `TimeDateStamp` (offset 4..8) is written as 0 — the differential normalizes
//! it away. Every other byte must genuinely match.


// The writers are split by concern; every module re-exports flat into `coff`,
// so `use super::*` inside any one of them sees the whole surface and the
// crate-facing API (`coff::emit_obj`, `coff::Frame`, …) is exactly what it was
// before the split.
mod buf;
mod checksum;
mod container;
mod data;
mod dyninit;
mod ehscope;
mod function;
mod label;
mod mangle;
mod order;
mod pdata;
mod provide;
mod reloc;
mod shell;
mod symbol;
mod writer;
#[cfg(test)]
mod tests;

pub(crate) use buf::*;
pub(crate) use checksum::*;
pub(crate) use container::*;
pub use data::*;
pub use dyninit::*;
pub use ehscope::*;
pub use function::*;
pub use label::*;
pub use mangle::*;
pub use order::*;
pub use pdata::*;
pub use provide::*;
pub(crate) use reloc::*;
pub use shell::*;
pub(crate) use symbol::*;
pub use writer::*;
