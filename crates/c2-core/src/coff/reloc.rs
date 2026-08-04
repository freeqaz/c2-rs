//! Relocation record types and the fixed record width.
//!
//! Every REFHI and every REFLO takes a trailing `PAIR` at the same
//! `VirtualAddress`; `REL24` and `ADDR32` take none
//! (`docs/OBJ_DYNINIT_SHAPE.md` §3.2).


/// One COFF relocation record: VirtualAddress u32, SymbolTableIndex u32,
/// Type u16 (packed, not padded).
pub(crate) const RELOC_LEN: usize = 10;

/// IMAGE_REL_PPC_REFHI / REFLO / PAIR. c2 loads a pooled FP constant through an
/// `addis`+`lfs` pair, and each half needs a PAIR record carrying the other
/// half's displacement in its `SymbolTableIndex` field. Every pooled constant
/// gets its own COMDAT section, so that displacement is always 0.
pub(crate) const REL_PPC_REFHI: u16 = 0x0010;
pub(crate) const REL_PPC_REFLO: u16 = 0x0011;
pub(crate) const REL_PPC_PAIR: u16 = 0x0012;


/// IMAGE_REL_PPC_REL24 — 24-bit relative branch relocation (tail/`bl` calls).
pub(crate) const REL_PPC_REL24: u16 = 0x0006;
/// IMAGE_REL_PPC_ADDR32 — 32-bit VA relocation (the `.pdata` BeginAddress).
pub(crate) const REL_PPC_ADDR32: u16 = 0x0002;
