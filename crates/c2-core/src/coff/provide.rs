//! **W-NPOS — the provide-always COMDAT data obj**: the four-section shell
//! plus one `Selection=2` (`IMAGE_COMDAT_SELECT_ANY`) data section per
//! provide-always object — `.rdata` for a read-only one, `.data` otherwise —
//! with a real aux CheckSum and an EXTERNAL object symbol. No `.text`, no
//! relocations, no undefined externals.
//!
//! This is the section shape `w-three` measured as the shared terminal of the
//! three reader-clear TUs (board #3200) and `w-bind16` reached from the
//! binding side (#3196): a 4-byte `sel=2` `.rdata` COMDAT that none of this
//! directory's other four `.rdata` emission sites writes (they are a string
//! literal, an EH associative, and two FP constant pools — all with different
//! symbol shapes, checksum rules or selection values).
//!
//! # The measured cells (all at the workload's own flags, `work/w-npos/`)
//!
//! | cell | objects | section | chars | aux CheckSum | symbols |
//! |---|---|---|---|---|---|
//! | `decomp_pch.cpp` | `?npos@…@2IB` u32 `ff ff ff ff` | `.rdata` | `0x40301040` | `0xdebb20e3` | 11 shell + section/aux + EXTERNAL |
//! | `x02`, `x04` | one const u32 | `.rdata` | `0x40301040` | CRC(content) | same |
//! | `x05` | TWO const u32s | two `.rdata` | `0x40301040` ×2 | two CRCs | 17 |
//! | `x06` | u8 / u16 / u64 | three `.rdata` | nibbles **1 / 3 / 4** | three CRCs | 20 |
//! | `g10`, `x07` | one NON-const int | `.data` | `0xC0301040` | CRC(content) | 14 |
//!
//! Verified 8/8: the aux CheckSum is [`coff_checksum`] (reflected CRC-32,
//! init 0, no final XOR) over the section's raw content — **not** zero, which
//! is what the FP-constant `.rdata` COMDAT carries
//! (`docs/OBJ_DYNINIT_SHAPE.md` §2.3 H9); the two shapes genuinely differ.
//!
//! # The alignment nibble is NOT the natural-alignment identity
//!
//! Witnessed: size 1 → nibble 1 (`ALIGN_1BYTES`), size 2 → **3**
//! (`ALIGN_4BYTES`, though the type's natural alignment is 2), size 4 → 3,
//! size 8 → 4. The u16 cell is the one that separates "nibble encodes natural
//! alignment" from this table, and it refutes it. Sizes outside the table
//! refuse — an aggregate (size ≠ width) has no witnessed nibble at all.
//!
//! # Order
//!
//! Sections follow the objects slice, which the reader hands over in `.gl`
//! record order. The two-object cell (`x05`) cannot separate record order
//! from declaration order — the permutations coincide on every witness — so
//! this is recorded as unseparated, the same status `data.rs` gives its
//! relocation sort.

use super::*;

/// One provide-always COMDAT data object (see the module doc).
pub struct ProvideObj<'a> {
    /// COFF symbol name — EXTERNAL, so StorageClass 2 always.
    pub symbol: &'a str,
    /// `true` → `.rdata` (`0x40001040 | nibble`); `false` → `.data`
    /// (`0xC0001040 | nibble`).
    pub ro: bool,
    /// Raw content, already big-endian as `.in` carries it. `len()` must be a
    /// witnessed width (1, 2, 4, 8).
    pub bytes: &'a [u8],
}

/// COMDAT `IMAGE_COMDAT_SELECT_ANY`.
const COMDAT_SELECT_ANY: u8 = 2;

/// Section characteristics bases (align nibble OR'd in at bit 20).
const CH_RDATA_BASE: u32 = 0x4000_1040;
const CH_DATA_COMDAT_BASE: u32 = 0xC000_1040;

/// The witnessed size → alignment-nibble table (module doc). `None` refuses.
fn provide_nibble(size: usize) -> Option<u32> {
    match size {
        1 => Some(1),
        2 | 4 => Some(3),
        8 => Some(4),
        _ => None,
    }
}

/// Build the obj, or `None` for any cell outside the measured class. Every
/// `None` is a shape with no witness, not an error path.
pub fn emit_provide_data_obj(obj_name: &str, objects: &[ProvideObj<'_>]) -> Option<Vec<u8>> {
    if objects.is_empty() {
        return None; // the bare shell is `emit_empty_obj`'s obj, not this one
    }
    for (i, o) in objects.iter().enumerate() {
        if o.symbol.is_empty() || o.symbol.starts_with('.') {
            return None;
        }
        provide_nibble(o.bytes.len())?;
        if objects[..i].iter().any(|q| q.symbol == o.symbol) {
            return None; // two records, one linker name
        }
    }

    // ---- sections: the contiguous shell, then one COMDAT per object ----
    let mut sections = shell_sections(obj_name);
    for o in objects {
        let nibble = provide_nibble(o.bytes.len())?;
        sections.push(Section {
            name: if o.ro { ".rdata" } else { ".data" },
            characteristics: if o.ro { CH_RDATA_BASE } else { CH_DATA_COMDAT_BASE }
                | (nibble << 20),
            checksum: coff_checksum(o.bytes),
            selection: COMDAT_SELECT_ANY,
            assoc: 0,
            raw: std::borrow::Cow::Borrowed(o.bytes),
            uninit_size: None,
        });
    }

    let n_sections = sections.len();
    let n_reloc_of = vec![0u16; n_sections];
    let (ptrs, _reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);

    // 11 shell symbols, then per object: section symbol + aux + EXTERNAL.
    let n_symbols = N_SHELL_SYMBOLS + 3 * objects.len() as u32;

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 256);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &vec![None; n_sections], &n_reloc_of);
    for (i, s) in sections.iter().enumerate() {
        debug_assert_eq!(b.0.len(), ptrs[i]);
        b.bytes(&s.raw);
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table: the shell's eleven, then each object's group ----
    // The shell sections are contiguous here (nothing is spliced into them),
    // so the shared helper applies — the `.bss` splice that forbids it in
    // `data.rs` cannot happen in this shape.
    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections);
    for (k, o) in objects.iter().enumerate() {
        let si = 4 + k; // section index (0-based) after the four shell sections
        emit_section_symbol(&mut b, &sections[si], (si + 1) as i16, 0);
        emit_external_symbol(&mut b, &mut strtab, o.symbol, (si + 1) as i16, 0x0000);
    }
    debug_assert_eq!((b.0.len() - ptr_symtab) / SYMBOL_LEN, n_symbols as usize);

    b.bytes(&strtab.finish());
    Some(b.0)
}
