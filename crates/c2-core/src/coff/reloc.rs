//! The relocation types the writer **emits**, and the fixed record width.
//!
//! Every REFHI and every REFLO takes a trailing `PAIR` at the same
//! `VirtualAddress`; `REL24` and `ADDR32` take none
//! (`docs/OBJ_DYNINIT_SHAPE.md` §3.2).
//!
//! **The values are not defined here any more.** The complete
//! `IMAGE_REL_PPC_*` table — all 23 types, the `TYPEMASK` and the four modifier
//! bits — lives in `c2-obj::reloc`, which is also where the *reader* is; this
//! module is a five-row window onto it, so the emitter and the decoder can never
//! drift onto two different vocabularies. See `crates/c2-obj/src/reloc.rs` for
//! provenance and for the packed-word rule.

pub(crate) use c2_obj::RELOC_LEN;

/// IMAGE_REL_PPC_REFHI / REFLO / PAIR. c2 loads a pooled FP constant through an
/// `addis`+`lfs` pair, and each half needs a PAIR record carrying the other
/// half's displacement in its `SymbolTableIndex` field. Every pooled constant
/// gets its own COMDAT section, so that displacement is always 0.
pub(crate) use c2_obj::IMAGE_REL_PPC_PAIR as REL_PPC_PAIR;
pub(crate) use c2_obj::IMAGE_REL_PPC_REFHI as REL_PPC_REFHI;
pub(crate) use c2_obj::IMAGE_REL_PPC_REFLO as REL_PPC_REFLO;

/// IMAGE_REL_PPC_REL24 — 24-bit relative branch relocation (tail/`bl` calls).
pub(crate) use c2_obj::IMAGE_REL_PPC_REL24 as REL_PPC_REL24;
/// IMAGE_REL_PPC_ADDR32 — 32-bit VA relocation (the `.pdata` BeginAddress).
pub(crate) use c2_obj::IMAGE_REL_PPC_ADDR32 as REL_PPC_ADDR32;

#[cfg(test)]
mod tests {
    /// **The emitted bytes must not move.** This lane replaced five locally
    /// defined constants with re-exports of a ported table; had any value
    /// changed, every obj the writer produces would change with it. The
    /// literals below are the ones this file held before the swap, transcribed
    /// from `git show c303ad0:crates/c2-core/src/coff/reloc.rs` — they are
    /// deliberately written out rather than referenced, so this test cannot be
    /// satisfied by the same table it is checking.
    #[test]
    fn the_five_emitted_type_values_are_unchanged_by_the_table_port() {
        assert_eq!(super::RELOC_LEN, 10);
        assert_eq!(super::REL_PPC_ADDR32, 0x0002);
        assert_eq!(super::REL_PPC_REL24, 0x0006);
        assert_eq!(super::REL_PPC_REFHI, 0x0010);
        assert_eq!(super::REL_PPC_REFLO, 0x0011);
        assert_eq!(super::REL_PPC_PAIR, 0x0012);
    }
}
