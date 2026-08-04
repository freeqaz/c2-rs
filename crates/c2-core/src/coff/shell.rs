//! The fixed four-section shell every obj begins with.
//!
//! `.drectve`, `.debug$S` and the two `.XBLD$W` watermark COMDATs, plus the 11
//! symbol records that describe them. Checked mechanically identical across 61
//! reference objs (`docs/OBJ_DYNINIT_SHAPE.md` §4.1) — only `.debug$S` moves,
//! and only with the embedded `-Fo` path length.

use super::*;

/// Fixed `.drectve` directive string (132 bytes, no NUL). 100% constant.
pub(crate) const DRECTVE: &[u8] =
    b"   /include:__C1_11886 /DEFAULTLIB:\"OLDNAMES\" /DEFAULTLIB:\"LIBCMT\" \
      /DEFAULTLIB:\"XAPILIB\" /DEFAULTLIB:\"XBOXKRNL\" /include:__C2_11886 ";

/// `.debug$S` record 2 — S_COMPILE2 (type 0x1116), 100% constant, 57 bytes
/// incl. its own u16 length. Byte-identical across all fixtures.
pub(crate) const S_COMPILE2: [u8; 57] = [
    0x37, 0x00, 0x16, 0x11, 0x01, 0x02, 0x00, 0x00, 0x42, 0x00, 0x10, 0x00, 0x00, 0x00, 0x6E, 0x2E,
    0x10, 0x00, 0x00, 0x00, 0x6E, 0x2E, 0x4D, 0x69, 0x63, 0x72, 0x6F, 0x73, 0x6F, 0x66, 0x74, 0x20,
    0x28, 0x52, 0x29, 0x20, 0x4F, 0x70, 0x74, 0x69, 0x6D, 0x69, 0x7A, 0x69, 0x6E, 0x67, 0x20, 0x43,
    0x6F, 0x6D, 0x70, 0x69, 0x6C, 0x65, 0x72, 0x00, 0x00,
];

/// `.XBLD$W` C2 watermark payload (16 bytes). Constant.
pub(crate) const XBLD_C2: [u8; 16] = [
    0x43, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x2E, 0x6E, 0x44, 0x00,
];
/// `.XBLD$W` C1 watermark payload (16 bytes). Differs from C2 only in byte 1.
pub(crate) const XBLD_C1: [u8; 16] = [
    0x43, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x2E, 0x6E, 0x44, 0x00,
];

/// Aux section-def CheckSum for the two COMDAT watermark sections (constant —
/// fixed watermark content).
pub(crate) const XBLD_C2_CHECKSUM: u32 = 0x92F8_7AA0;
pub(crate) const XBLD_C1_CHECKSUM: u32 = 0x8385_10D9;

/// `@comp.id` value — the cl 16.00.11886 toolchain-version stamp. Constant.
pub(crate) const COMP_ID_VALUE: u32 = 0x00AB_2E6E;

/// External watermark symbol names (constant toolchain build-number watermarks).
pub(crate) const NAME_C2: &str = "__C2_11886";
pub(crate) const NAME_C1: &str = "__C1_11886";


// Section characteristics (verified constant across fixtures).
pub(crate) const CH_DRECTVE: u32 = 0x0010_0A00;
pub(crate) const CH_DEBUGS: u32 = 0x4210_0040;
pub(crate) const CH_XBLD_C2: u32 = 0xC040_1040;
pub(crate) const CH_XBLD_C1: u32 = 0xC230_1040;

/// Build the `.debug$S` raw section: CV signature + one `0xF1` SYMBOLS
/// subsection (S_OBJNAME with the `-Fo` path, then the constant S_COMPILE2),
/// padded to a 4-byte multiple.
pub(crate) fn build_debug_s(obj_name: &str) -> Vec<u8> {
    let name = obj_name.as_bytes();
    // S_OBJNAME record: reclen(u16) | rectyp=0x1101(u16) | signature=0(u32) |
    //                   name bytes | NUL. reclen counts bytes AFTER itself.
    let reclen1 = (2 + 4 + name.len() + 1) as u16;

    let mut sub = Buf::new();
    // record 1
    sub.u16(reclen1);
    sub.u16(0x1101);
    sub.u32(0x0000_0000);
    sub.bytes(name);
    sub.u8(0);
    // record 2 (constant)
    sub.bytes(&S_COMPILE2);
    let subsec_content = sub.0;

    let mut b = Buf::new();
    b.u32(0x0000_0004); // CV_SIGNATURE_C13
    b.u32(0x0000_00F1); // DEBUG_S_SYMBOLS
    b.u32(subsec_content.len() as u32);
    b.bytes(&subsec_content);
    // pad the whole raw section to a 4-byte multiple.
    while b.0.len() % 4 != 0 {
        b.u8(0);
    }
    b.0
}


/// The four fixed sections **every** obj this file emits begins with, in order:
/// `.drectve`, `.debug$S`, `.XBLD$W` (C2), `.XBLD$W` (C1). Only `.debug$S`
/// varies, and only with the `-Fo` path.
///
/// Checked mechanically over 61 reference objs (`docs/OBJ_DYNINIT_SHAPE.md`
/// §4.1): the `.drectve` raw bytes, both `.XBLD$W` raw bytes and the first four
/// `Characteristics` words are identical in all 61, across `/Ox` and `/O1`
/// alike. It is one shell, so it is written once — the four emitters below had
/// four byte-identical copies of this literal.
pub(crate) fn shell_sections<'a>(obj_name: &str) -> Vec<Section<'a>> {
    vec![
        Section {
            name: ".drectve",
            characteristics: CH_DRECTVE,
            raw: std::borrow::Cow::Borrowed(DRECTVE),
            checksum: 0,
            selection: 0,
            assoc: 0,
            uninit_size: None,
        },
        Section {
            name: ".debug$S",
            characteristics: CH_DEBUGS,
            raw: std::borrow::Cow::Owned(build_debug_s(obj_name)),
            checksum: 0,
            selection: 0,
            assoc: 0,
            uninit_size: None,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C2,
            raw: std::borrow::Cow::Borrowed(&XBLD_C2),
            checksum: XBLD_C2_CHECKSUM,
            selection: 2,
            assoc: 0,
            uninit_size: None,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C1,
            raw: std::borrow::Cow::Borrowed(&XBLD_C1),
            checksum: XBLD_C1_CHECKSUM,
            selection: 2,
            assoc: 0,
            uninit_size: None,
        },
    ]
}


/// Symbol records 0..=10 — the fixed prefix every obj carries, identical in all
/// 61 reference objs measured (`docs/OBJ_DYNINIT_SHAPE.md` §4.1):
/// `@comp.id`, then the four shell sections' STATIC section symbols with their
/// aux records, with the two watermark externals after their own `.XBLD$W`.
///
/// `sections` must begin with [`shell_sections`]' four.
pub(crate) fn emit_shell_symbols(b: &mut Buf, strtab: &mut StringTable, sections: &[Section]) {
    // slot 0: @comp.id (ABS, STATIC, no aux)
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1); // IMAGE_SYM_ABSOLUTE
    b.u16(0x0000);
    b.u8(3); // STATIC
    b.u8(0);
    emit_section_symbol(b, &sections[0], 1, 0); // slot 1/2  .drectve
    emit_section_symbol(b, &sections[1], 2, 0); // slot 3/4  .debug$S
    emit_section_symbol(b, &sections[2], 3, 0); // slot 5/6  .XBLD$W C2
    emit_external_symbol(b, strtab, NAME_C2, 3, 0x0000); // slot 7
    emit_section_symbol(b, &sections[3], 4, 0); // slot 8/9  .XBLD$W C1
    emit_external_symbol(b, strtab, NAME_C1, 4, 0x0000); // slot 10
}

/// How many symbol records [`emit_shell_symbols`] writes.
pub(crate) const N_SHELL_SYMBOLS: u32 = 11;


/// Build the complete `.obj` image for a translation unit that **defines no
/// functions** (R1).
///
/// Such TUs are real and not rare — license-text files, and platform sources
/// whose entire body is `#ifdef`'d out for the 360 target; the dc3 workload has
/// seven. The front end still emits a full five-file IL bundle and c2 still
/// emits a genuine COFF obj, just one with no code in it. That makes this the
/// smallest possible *whole-TU* byte-exact target, and the only one reachable
/// without any instruction selection at all.
///
/// The image is the fixed four-section shell every obj carries, minus `.text`:
///
/// ```text
///   1 .drectve   132 B   constant directive string
///   2 .debug$S   152 B   only the S_OBJNAME path varies (the `-Fo` argument)
///   3 .XBLD$W     16 B   c2 watermark + COMDAT checksum
///   4 .XBLD$W     16 B   c1 watermark + COMDAT checksum
/// ```
///
/// with 11 symbols — the same fixed prefix [`emit_obj`] uses, stopping before
/// the `.text` section symbol (which would be slots 11/12). No relocations.
/// Verified against the live toolchain: a 720-byte obj for a TU containing only
/// a typedef.
pub fn emit_empty_obj(obj_name: &str) -> Vec<u8> {
    let sections = shell_sections(obj_name);
    let n_reloc_of = vec![0u16; sections.len()];
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);
    // 1 (@comp.id) + 4 section symbols x 2 (symbol + aux) + 2 watermark externs.
    let n_symbols = N_SHELL_SYMBOLS;

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, sections.len(), ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);
    for s in &sections {
        b.bytes(&s.raw);
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections);
    b.bytes(&strtab.finish());
    b.0
}
