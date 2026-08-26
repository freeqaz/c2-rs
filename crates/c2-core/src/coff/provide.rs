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
//! Sections follow the objects slice, which the reader
//! (`IlBundle::provide_data_tu`) hands over in **declaration order** —
//! token-ascending, `DataObject::decl_index`'s measured identity. That is NOT
//! the `.gl` record order: the `/Ox` capture of the three-width fixture
//! spells its records s,c,q against a c,s,q obj, and probe `x08` (u16
//! declared before u8) follows declaration against ascending size.

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
/// PROV[S] PE/COFF spec — `IMAGE_COMDAT_SELECT_ANY` is 2. Not from c2.
const COMDAT_SELECT_ANY: u8 = 2;

/// Section characteristics bases (align nibble OR'd in at bit 20).
/// PROV[O] transcribed from real objs.
const CH_RDATA_BASE: u32 = 0x4000_1040;
// PROV[O] transcribed from real objs.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn u32obj<'a>(symbol: &'a str, bytes: &'a [u8]) -> ProvideObj<'a> {
        ProvideObj { symbol, ro: true, bytes }
    }

    /// The measured class check: every `None` is a shape with no witness.
    #[test]
    fn refuses_every_unwitnessed_shape() {
        assert!(emit_provide_data_obj("Z:\\t\\x.obj", &[]).is_none(), "empty = emit_empty_obj's");
        assert!(
            emit_provide_data_obj("Z:\\t\\x.obj", &[u32obj("?a@@3IB", &[0, 0, 0])]).is_none(),
            "size 3 has no witnessed alignment nibble"
        );
        assert!(
            emit_provide_data_obj("Z:\\t\\x.obj", &[u32obj("", &[0, 0, 0, 0])]).is_none(),
            "an empty symbol name is nothing the linker can hold"
        );
        assert!(
            emit_provide_data_obj("Z:\\t\\x.obj", &[u32obj(".rdata", &[0, 0, 0, 0])]).is_none(),
            "a section-shaped name is not an object symbol"
        );
        let a = u32obj("?a@@3IB", &[1, 2, 3, 4]);
        let b = u32obj("?a@@3IB", &[5, 6, 7, 8]);
        assert!(
            emit_provide_data_obj("Z:\\t\\x.obj", &[a, b]).is_none(),
            "two records, one linker name"
        );
    }

    /// The witnessed nibble table — the u16 cell is the one that separates it
    /// from the natural-alignment identity (`ALIGN_4`, not `ALIGN_2`).
    #[test]
    fn the_alignment_nibble_is_the_measured_table_not_natural_alignment() {
        assert_eq!(provide_nibble(1), Some(1));
        assert_eq!(provide_nibble(2), Some(3), "u16 -> ALIGN_4BYTES, measured on x06/qs");
        assert_eq!(provide_nibble(4), Some(3));
        assert_eq!(provide_nibble(8), Some(4));
        assert_eq!(provide_nibble(3), None);
        assert_eq!(provide_nibble(16), None, "no witnessed cell; refuse");
    }

    /// The shape constants, checked against `decomp_pch.cpp`'s own obj: 5
    /// sections, 14 symbols, `.rdata` chars `0x40301040`, aux CheckSum
    /// `0xdebb20e3` over `ff ff ff ff`, EXTERNAL long-name symbol in the
    /// string table.
    #[test]
    fn the_npos_shape_reproduces() {
        let name = "?npos@?$basic_string@DV?$char_traits@D@stlpmtx_std@@V?$allocator@D@2@@stlpmtx_std@@2IB";
        let obj = emit_provide_data_obj(
            "Z:\\t\\decomp_pch.obj",
            &[ProvideObj { symbol: name, ro: true, bytes: &[0xFF; 4] }],
        )
        .unwrap();
        assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 5, "section count");
        let n_syms = u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]);
        assert_eq!(n_syms, 14, "11 shell + section + aux + EXTERNAL");
        // Section 5's header (index 4): name and characteristics.
        let h = 20 + 4 * 40;
        assert_eq!(&obj[h..h + 6], b".rdata");
        let chars = u32::from_le_bytes([obj[h + 36], obj[h + 37], obj[h + 38], obj[h + 39]]);
        assert_eq!(chars, 0x4030_1040, "ro COMDAT, ALIGN_4");
        assert_eq!(coff_checksum(&[0xFF; 4]), 0xDEBB_20E3, "the measured aux CheckSum");
        // The long name rides in the string table.
        assert!(obj.windows(name.len()).any(|w| w == name.as_bytes()));
    }

    /// The rw twin (`g10`/`x07`): `.data`, chars `0xC0301040`, same symbol
    /// shape — one emitter, one branch, no second copy of the layout.
    #[test]
    fn the_rw_twin_lands_in_data() {
        let obj = emit_provide_data_obj(
            "Z:\\t\\x.obj",
            &[ProvideObj { symbol: "?sa@@3HA", ro: false, bytes: &[0, 0, 0, 3] }],
        )
        .unwrap();
        let h = 20 + 4 * 40;
        assert_eq!(&obj[h..h + 5], b".data");
        let chars = u32::from_le_bytes([obj[h + 36], obj[h + 37], obj[h + 38], obj[h + 39]]);
        assert_eq!(chars, 0xC030_1040);
        assert_eq!(coff_checksum(&[0, 0, 0, 3]), 0x9909_51BA, "g10's measured CheckSum");
    }
}
