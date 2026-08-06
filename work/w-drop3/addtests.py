import io
p = 'crates/c2-obj/src/lib.rs'
s = open(p).read()
anchor = '''    #[test]
    fn the_emitted_set_is_one_leader_per_text_comdat_section() {'''
new = r'''    /// [`coff`] plus a relocation table for one section: `(section index
    /// 0-based, records)` where each record is `(va, symbol slot, packed type)`.
    /// The records are appended past the string table and the section header's
    /// `PointerToRelocations` / `NumberOfRelocations` are patched to point at
    /// them, which is where a real obj puts them relative to the parts [`coff`]
    /// already writes.
    fn coff_with_relocs(
        sections: &[(&str, bool)],
        symbols: &[(&str, i16, u8, u8)],
        sec: usize,
        records: &[(u32, u32, u16)],
    ) -> Vec<u8> {
        let mut out = coff(sections, symbols);
        let ptr = out.len() as u32;
        for (va, sym, ty) in records {
            out.extend_from_slice(&va.to_le_bytes());
            out.extend_from_slice(&sym.to_le_bytes());
            out.extend_from_slice(&ty.to_le_bytes());
        }
        let o = COFF_HEADER_LEN + sec * SECTION_HEADER_LEN;
        out[o + 24..o + 28].copy_from_slice(&ptr.to_le_bytes());
        out[o + 32..o + 34].copy_from_slice(&(records.len() as u16).to_le_bytes());
        out
    }

    /// Symbols for the call-target tests. A relocation's `SymbolTableIndex`
    /// names a **slot**, and an aux record occupies one, so the indices are
    /// written out here rather than counted at each call site:
    /// `.text` 0 (+1 aux) · `?caller` 2 · `?callee` 3 · `?other` 4 ·
    /// `?g` 5 (+1 aux, so slot 6 is the aux) · `.data` 7.
    const CALL_SYMS: &[(&str, i16, u8, u8)] = &[
        (".text", 1, IMAGE_SYM_CLASS_STATIC, 1),
        ("?caller@@YAXXZ", 1, 2, 0),
        ("?callee@@YAXXZ", 0, 2, 0),
        ("?other@@YAXXZ", 0, 2, 0),
        ("?g@@YAXXZ", 0, 2, 1),
        (".data", 2, IMAGE_SYM_CLASS_STATIC, 1),
    ];

    /// **The whole reason this reader exists** (board #984): under `/Gy` a call
    /// out of a COMDAT is emitted with the placeholder displacement
    /// `-(offset of the word)` whatever the callee is, so two branch words that
    /// call two different functions are byte-identical and only the relocation
    /// table can tell them apart.
    #[test]
    fn a_call_target_is_named_by_its_relocation_and_not_by_its_bytes() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(0x0, 3, crate::reloc::IMAGE_REL_PPC_REL24)],
        ));
        assert_eq!(
            obj.text_comdat_call_targets(),
            Some(vec![(
                "?caller@@YAXXZ".to_string(),
                vec![(0x0, "?callee@@YAXXZ".to_string())]
            )]),
            "the REL24 names slot 3, which is ?callee"
        );
    }

    /// Only `REL24`. A `REFHI`/`REFLO` pair is a *data* reference, and a `PAIR`
    /// record's `sym` field is a displacement rather than an index (rev 6.0), so
    /// naming one would be reading a number as a symbol. Both `PAIR`s here carry
    /// a displacement that is also a valid slot, which is the case a
    /// base-type-blind reader gets wrong.
    #[test]
    fn only_rel24_records_are_call_targets() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[
                (0x0, 4, crate::reloc::IMAGE_REL_PPC_REFHI),
                (0x0, 2, crate::reloc::IMAGE_REL_PPC_PAIR),
                (0x8, 4, crate::reloc::IMAGE_REL_PPC_REFLO),
                (0x8, 2, crate::reloc::IMAGE_REL_PPC_PAIR),
                (0xc, 5, crate::reloc::IMAGE_REL_PPC_REL24),
            ],
        ));
        assert_eq!(
            obj.text_comdat_call_targets(),
            Some(vec![(
                "?caller@@YAXXZ".to_string(),
                vec![(0xc, "?g@@YAXXZ".to_string())]
            )]),
            "four data-reference records must contribute no call target"
        );
    }

    /// A flag bit on the packed type word must not hide the record: `REL24 |
    /// BRTAKEN` is `0x0206` and is still a call.
    #[test]
    fn a_modifier_bit_does_not_hide_a_call_target() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(
                0x4,
                3,
                crate::reloc::IMAGE_REL_PPC_REL24 | crate::reloc::IMAGE_REL_PPC_BRTAKEN,
            )],
        ));
        assert_eq!(
            obj.text_comdat_call_targets().map(|v| v[0].1.len()),
            Some(1),
            "a whole-word type compare is the defect Reloc::base() exists to prevent"
        );
    }

    /// Fail-closed: a `SymbolTableIndex` past the table is `None` for the whole
    /// walk, never a short list. A short list reads as "this body calls fewer
    /// things than it does", which is absence-as-success.
    #[test]
    fn a_symbol_index_past_the_table_returns_no_answer_at_all() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(0x0, 9999, crate::reloc::IMAGE_REL_PPC_REL24)],
        ));
        assert_eq!(obj.text_comdat_call_targets(), None);
    }

    /// …and so is a relocation naming an AUX slot, which carries no name.
    #[test]
    fn a_relocation_naming_an_aux_slot_returns_no_answer_at_all() {
        let obj = ObjImage::new(coff_with_relocs(
            &[(".text", true), (".data", false)],
            CALL_SYMS,
            0,
            &[(0x0, 6, crate::reloc::IMAGE_REL_PPC_REL24)],
        ));
        assert_eq!(obj.text_comdat_call_targets(), None);
    }

    /// A COMDAT with no relocations at all is an EMPTY target list, not an
    /// absent row: "calls nothing" and "did not decode" are different answers,
    /// and the caller distinguishes them.
    #[test]
    fn a_comdat_with_no_relocations_is_an_empty_list_and_not_an_absent_row() {
        let obj = ObjImage::new(coff(&[(".text", true), (".data", false)], CALL_SYMS));
        assert_eq!(
            obj.text_comdat_call_targets(),
            Some(vec![("?caller@@YAXXZ".to_string(), Vec::new())])
        );
    }

'''
assert anchor in s
s = s.replace(anchor, new + anchor, 1)
open(p, 'w').write(s)
print("ok")
