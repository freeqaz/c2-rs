//! The `??__E` dynamic-initializer obj (board #158).
//!
//! A TU whose emitted functions are namespace-scope object initializer thunks.
//! `docs/OBJ_DYNINIT_SHAPE.md` characterizes the shape; `docs/OBJ_DATA_BSS_SHAPE.md`
//! §5.2/§6.2 supply the `.bss` walk and symbol order for more than one object.

use super::*;

/// `.text$yc` characteristics — CNT_CODE | COMDAT | ALIGN_8 | EXECUTE | READ.
/// Numerically the same word as an ordinary `/Gy` `.text`; the **selection**
/// is what differs (2 ANY here, 1 NODUPLICATES there), which prereg P3 got
/// backwards.
pub(crate) const CH_TEXT_YC: u32 = 0x6040_1020;

/// `.CRT$XCU` characteristics — CNT_INIT_DATA | ALIGN_4 | READ | WRITE.
/// ALIGN_4 in every cell measured, and **not** a COMDAT.
pub(crate) const CH_CRT_XCU: u32 = 0xC030_0040;

/// `.rdata` (string literal, `/GF`) characteristics with the alignment nibble
/// cleared: CNT_INIT_DATA | COMDAT | READ. OR in `nibble << 20`.
pub(crate) const CH_RDATA_STRING_BASE: u32 = 0x4000_1040;

/// `.bss` characteristics with the alignment nibble cleared: CNT_UNINIT_DATA |
/// READ | WRITE. OR in `nibble << 20`. **Never** a COMDAT (prereg P2, refuted
/// in that direction).
pub(crate) const CH_BSS_BASE: u32 = 0xC000_0080;

/// `IMAGE_COMDAT_SELECT_ANY` — the selection a `??__E` thunk's `.text$yc` and
/// its string `.rdata` carry. An *ordinary* function's `.text` uses
/// [`COMDAT_SELECT_NODUPLICATES`] instead, so this is a discriminator and not a
/// constant (prereg P3).
pub(crate) const COMDAT_SELECT_ANY: u8 = 2;


/// The `.bss` object a dynamic initializer constructs.
pub struct BssObject<'a> {
    /// The COFF symbol name, already in its final form: undecorated
    /// (`sLicense`) for internal linkage, decorated
    /// (`?sLicense@@3VLicenses@@A`) for external.
    pub symbol: &'a str,
    /// `sizeof` the object. Becomes `.bss` `SizeOfRawData` and the aux `Length`
    /// — **not** `VirtualSize`, which is 0 (§1, prereg P8 refuted).
    pub size: u32,
    /// The object's natural alignment `t`, in bytes (1/2/4/8). Feeds
    /// [`align_nibble`] together with `size`.
    pub natural_align: u32,
    /// `true` => StorageClass 2 EXTERNAL; `false` => 3 STATIC. The **object's**
    /// linkage only: the `??__E` thunk stays STATIC either way, and so does
    /// `<name>$initializer$` (§4.3).
    pub external: bool,
    /// The `.CRT$XCU` slot symbol: `<source identifier>$initializer$`, built
    /// from the SOURCE identifier and never from the decorated name —
    /// `?gL@@3UL@@A` still yields `gL$initializer$` (§3.1).
    pub initializer_symbol: &'a str,
}

/// One string-literal `.rdata` COMDAT.
pub struct StringLiteral<'a> {
    /// The literal's bytes INCLUDING the trailing NUL. Becomes the `.rdata` raw
    /// data, the aux `Length`, the aux `CheckSum` (the real CRC — a string
    /// `.rdata` carries it, unlike an FP-constant one) and the COMDAT name via
    /// [`string_comdat_name`].
    pub bytes: &'a [u8],
}

/// A `??__E<name>@@YAXXZ` dynamic-initializer thunk.
pub struct DynInitThunk<'a> {
    /// e.g. `??__EsLicense@@YAXXZ`. Emitted **STATIC (3)** with `Type` 0x0020 —
    /// even when the object it initializes has external linkage. `ZlibLicense.cpp`
    /// confirms both halves at once: `?sLicense@@3VLicenses@@A` is EXTERNAL
    /// while `??__EsLicense@@YAXXZ` is STATIC (§3.1).
    pub name: &'a str,
    /// The encoded text; 0x18 bytes for the target class, and byte-identical
    /// across the fixture and both workload TUs.
    pub text: &'a [u8],
    /// REL24 sites. Exactly one for the target class, and it takes **no** PAIR.
    pub calls: Vec<Call<'a>>,
    /// REFHI/REFLO quads. Each `name` must match either the string COMDAT's
    /// mangled name (see [`string_comdat_name`]) or [`BssObject::symbol`].
    pub data_refs: Vec<DataRef<'a>>,
}

/// Emit the 8-section `??__E` dynamic-initializer obj, or `None` if the inputs
/// fall outside the class this was measured on — in which case the caller
/// reports `NotImplemented`.
///
/// Target class: exactly one thunk, exactly one `.bss` object, at most one
/// string literal, no `.pdata` (the thunk is a leaf) and no destructor. A
/// destructor is +2 sections, +10 symbol records and a framed `??__E` with 14
/// relocations (§4.4); ≥2 objects needs the `.bss` address permutation §7.1
/// explicitly declines.
///
/// Section order, and the symbol table that follows it exactly:
///
/// ```text
///   1 .drectve   2 .debug$S   3 .XBLD$W(C2)   4 .XBLD$W(C1)
///   5 .text$yc   6 .rdata     7 .bss          8 .CRT$XCU
/// ```
///
/// **The ordering rule** (§3.1), which is a rule and not a fit: the symbol table
/// follows section order; for each section, the section symbol + aux, then the
/// symbols that section defines, then any **undefined external first referenced
/// by that section**. That is why the constructor — `SectionNumber` 0, defined
/// nowhere — sits at index 14, *inside* the `.text$yc` group and *before* the
/// `.rdata` section symbol at 15. Neither [`emit_obj`] nor [`emit_comdat_obj`]
/// places an undefined external there, so their sequence is not reusable here
/// even though every primitive below it is.
pub fn emit_dyninit_obj(
    obj_name: &str,
    thunk: &DynInitThunk<'_>,
    literal: Option<&StringLiteral<'_>>,
    object: &BssObject<'_>,
) -> Option<Vec<u8>> {
    // ---- class check. Every `None` below is a case nothing measured. ----
    if thunk.calls.len() != 1 {
        return None; // one constructor call; 0 or 2+ is a different body
    }
    if thunk.text.is_empty() || thunk.text.len() % 4 != 0 {
        return None;
    }
    if object.size == 0 {
        return None;
    }
    let bss_nibble = align_nibble(object.size, object.natural_align)?;
    let string_name = match literal {
        None => None,
        Some(l) => Some(string_comdat_name(l.bytes)?),
    };
    // The data-symbol references must be exactly the string COMDAT (when there
    // is one) and the object — no more, no fewer, no repeats. A quad against
    // anything else is an operand class this shape was never measured with.
    let mut expected: Vec<&str> = Vec::new();
    if let Some(n) = &string_name {
        expected.push(n.as_str());
    }
    expected.push(object.symbol);
    if thunk.data_refs.len() != expected.len() {
        return None;
    }
    for e in &expected {
        if thunk.data_refs.iter().filter(|r| r.name == *e).count() != 1 {
            return None;
        }
    }

    // ---- sections ----
    let mut sections = shell_sections(obj_name);
    sections.push(Section {
        name: ".text$yc",
        characteristics: CH_TEXT_YC,
        raw: std::borrow::Cow::Borrowed(thunk.text),
        // `.text$y?` carries CheckSum 0 — the CRC is for `.XBLD$W`, `.pdata`
        // and a string `.rdata` only (§2.3).
        checksum: 0,
        selection: COMDAT_SELECT_ANY,
        assoc: 0,
        uninit_size: None,
    });
    let sec_text = sections.len() - 1;
    let sec_rdata = match literal {
        None => None,
        Some(l) => {
            let nibble = align_nibble(l.bytes.len() as u32, 1)?; // narrow char: t = 1
            sections.push(Section {
                name: ".rdata",
                characteristics: CH_RDATA_STRING_BASE | (nibble << 20),
                raw: std::borrow::Cow::Borrowed(l.bytes),
                // **CORRECTION to §2.3's scope.** The doc says the CheckSum is 0
                // "for every non-COMDAT section". At `/Ox` the *non*-COMDAT
                // `$SG` string `.rdata` carries the real CRC `0x8619B74C` for
                // `"abc\0"`, so the rule is about the section being a **string**
                // `.rdata`, not about it being a COMDAT. Not load-bearing here
                // (this one is a COMDAT), but the doc's version must not be the
                // rule that gets encoded.
                checksum: coff_checksum(l.bytes),
                selection: COMDAT_SELECT_ANY,
                assoc: 0,
                uninit_size: None,
            });
            Some(sections.len() - 1)
        }
    };
    sections.push(Section {
        name: ".bss",
        characteristics: CH_BSS_BASE | (bss_nibble << 20),
        raw: std::borrow::Cow::Borrowed(&[]),
        checksum: 0,
        // `.bss` is **never** a COMDAT, whatever the object's linkage (§2.2).
        selection: 0,
        assoc: 0,
        uninit_size: Some(object.size),
    });
    let sec_bss = sections.len() - 1;
    sections.push(Section {
        name: ".CRT$XCU",
        characteristics: CH_CRT_XCU,
        // All zero — the address comes entirely from the ADDR32 relocation
        // (§3.4). Its CRC-with-init-0 is also 0, so this section cannot
        // discriminate the checksum rule either way.
        raw: std::borrow::Cow::Borrowed(&[0, 0, 0, 0]),
        checksum: 0,
        selection: 0,
        assoc: 0,
        uninit_size: None,
    });
    let sec_crt = sections.len() - 1;
    let n_sections = sections.len();

    // ---- symbol indices, in section order ----
    // 11 shell, then per section: section symbol + aux, the symbols it defines,
    // then any undefined external it is the first to reference.
    let mut next = N_SHELL_SYMBOLS;
    let _idx_text_sec = next;
    next += 2;
    let idx_thunk = next;
    next += 1;
    // The constructor: SectionNumber 0, and it belongs to the `.text$yc` group
    // because that is the section that references it.
    let idx_ctor = next;
    next += 1;
    let idx_string = sec_rdata.map(|_| {
        next += 2; // .rdata section symbol + aux
        let i = next;
        next += 1;
        i
    });
    next += 2; // .bss section symbol + aux
    let idx_object = next;
    next += 1;
    next += 2; // .CRT$XCU section symbol + aux
    let idx_initializer = next;
    next += 1;
    let n_symbols = next;

    // ---- relocations ----
    // `.text$yc`: one REFHI/PAIR/REFLO/PAIR quad per data reference plus the
    // REL24, **ordered by VirtualAddress** with each primary ahead of its PAIR
    // at the equal VA. Derived by sorting, not positioned: §3.2's last row is a
    // cell where the HI block and the LO block name their symbols in *different*
    // orders (the FP constant's REFLO rides an `lfs` displacement), so "HI, HI,
    // LO, LO in data_ref order" is not a law even though it holds here.
    let sym_of = |name: &str| -> Option<u32> {
        if Some(name) == string_name.as_deref() {
            idx_string
        } else if name == object.symbol {
            Some(idx_object)
        } else {
            None
        }
    };
    let mut text_relocs: Vec<(u32, u32, u16)> = Vec::new();
    for r in &thunk.data_refs {
        let s = sym_of(r.name)?;
        text_relocs.push((r.hi_off, s, REL_PPC_REFHI));
        text_relocs.push((r.hi_off, 0, REL_PPC_PAIR));
        text_relocs.push((r.lo_off, s, REL_PPC_REFLO));
        text_relocs.push((r.lo_off, 0, REL_PPC_PAIR));
    }
    for c in &thunk.calls {
        // REL24 takes no PAIR.
        text_relocs.push((c.reloc_offset, idx_ctor, REL_PPC_REL24));
    }
    text_relocs.sort_by_key(|&(va, _, _)| va);
    // `.CRT$XCU`: one ADDR32 at offset 0 against the thunk, no PAIR (§3.4).
    let crt_relocs: Vec<(u32, u32, u16)> = vec![(0, idx_thunk, REL_PPC_ADDR32)];

    let mut relocs: Vec<Vec<(u32, u32, u16)>> = vec![Vec::new(); n_sections];
    relocs[sec_text] = text_relocs;
    relocs[sec_crt] = crt_relocs;
    let n_reloc_of: Vec<u16> = relocs.iter().map(|r| r.len() as u16).collect();

    // The pre-existing emitters both assume "only `.text` carries relocations in
    // this class"; here `.CRT$XCU` carries one too, which is why the counts come
    // off the record lists rather than off a per-section formula.
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);

    // Raw data, each section immediately followed by its own relocations.
    // `.bss` writes nothing at all — `raw` is empty and `file_len` is 0 — so
    // `.rdata` and `.CRT$XCU` end up contiguous in the file.
    for (i, s) in sections.iter().enumerate() {
        if s.uninit_size.is_none() {
            debug_assert_eq!(b.0.len(), ptrs[i]);
        }
        // An uninitialized section must carry no raw bytes, or this write and
        // the layout cursor would disagree by exactly `raw.len()`.
        debug_assert_eq!(s.file_len(), s.raw.len());
        b.bytes(&s.raw);
        if !relocs[i].is_empty() {
            debug_assert_eq!(b.0.len(), reloc_ptr[i].unwrap());
            for &(va, sym, ty) in &relocs[i] {
                b.u32(va);
                b.u32(sym);
                b.u16(ty);
            }
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table ----
    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections); // 0..=10

    // `.text$yc` group: section symbol + aux, the STATIC thunk, then the
    // undefined external constructor.
    emit_section_symbol(&mut b, &sections[sec_text], (sec_text + 1) as i16, n_reloc_of[sec_text]);
    // The one symbol shape none of the older helpers spelled: STATIC (3) with
    // FUNCTION type (0x0020). An *ordinary* function's symbol is EXTERNAL (2).
    emit_symbol(&mut b, &mut strtab, thunk.name, 0, (sec_text + 1) as i16, 0x0020, 3);
    emit_function_symbol(&mut b, &mut strtab, thunk.calls[0].callee, 0, 0);

    // `.rdata` group: section symbol + aux, then the COMDAT's defining symbol —
    // EXTERNAL (2) with `Type` 0, so the linker can fold it. (Without `/GF` the
    // corresponding `$SG<n>` symbol is STATIC instead.)
    if let (Some(si), Some(name)) = (sec_rdata, &string_name) {
        emit_section_symbol(&mut b, &sections[si], (si + 1) as i16, n_reloc_of[si]);
        emit_external_symbol(&mut b, &mut strtab, name, (si + 1) as i16, 0x0000);
    }

    // `.bss` group: section symbol + aux (Selection 0 — never a COMDAT), then
    // the object, whose storage class is the one thing its linkage moves.
    emit_section_symbol(&mut b, &sections[sec_bss], (sec_bss + 1) as i16, n_reloc_of[sec_bss]);
    emit_symbol(
        &mut b,
        &mut strtab,
        object.symbol,
        0,
        (sec_bss + 1) as i16,
        0x0000,
        if object.external { 2 } else { 3 },
    );

    // `.CRT$XCU` group: section symbol + aux, then `<name>$initializer$` —
    // STATIC, `Type` 0, `Value` 0, referenced by no relocation. It exists so the
    // linker has a name for the 4-byte slot.
    emit_section_symbol(&mut b, &sections[sec_crt], (sec_crt + 1) as i16, n_reloc_of[sec_crt]);
    emit_symbol(&mut b, &mut strtab, object.initializer_symbol, 0, (sec_crt + 1) as i16, 0x0000, 3);

    b.bytes(&strtab.finish());
    debug_assert_eq!(idx_initializer + 1, n_symbols);
    Some(b.0)
}
