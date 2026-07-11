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

/// Fixed `.drectve` directive string (132 bytes, no NUL). 100% constant.
const DRECTVE: &[u8] =
    b"   /include:__C1_11886 /DEFAULTLIB:\"OLDNAMES\" /DEFAULTLIB:\"LIBCMT\" \
      /DEFAULTLIB:\"XAPILIB\" /DEFAULTLIB:\"XBOXKRNL\" /include:__C2_11886 ";

/// `.debug$S` record 2 — S_COMPILE2 (type 0x1116), 100% constant, 57 bytes
/// incl. its own u16 length. Byte-identical across all fixtures.
const S_COMPILE2: [u8; 57] = [
    0x37, 0x00, 0x16, 0x11, 0x01, 0x02, 0x00, 0x00, 0x42, 0x00, 0x10, 0x00, 0x00, 0x00, 0x6E, 0x2E,
    0x10, 0x00, 0x00, 0x00, 0x6E, 0x2E, 0x4D, 0x69, 0x63, 0x72, 0x6F, 0x73, 0x6F, 0x66, 0x74, 0x20,
    0x28, 0x52, 0x29, 0x20, 0x4F, 0x70, 0x74, 0x69, 0x6D, 0x69, 0x7A, 0x69, 0x6E, 0x67, 0x20, 0x43,
    0x6F, 0x6D, 0x70, 0x69, 0x6C, 0x65, 0x72, 0x00, 0x00,
];

/// `.XBLD$W` C2 watermark payload (16 bytes). Constant.
const XBLD_C2: [u8; 16] = [
    0x43, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x2E, 0x6E, 0x44, 0x00,
];
/// `.XBLD$W` C1 watermark payload (16 bytes). Differs from C2 only in byte 1.
const XBLD_C1: [u8; 16] = [
    0x43, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x2E, 0x6E, 0x44, 0x00,
];

/// Aux section-def CheckSum for the two COMDAT watermark sections (constant —
/// fixed watermark content).
const XBLD_C2_CHECKSUM: u32 = 0x92F8_7AA0;
const XBLD_C1_CHECKSUM: u32 = 0x8385_10D9;

/// `@comp.id` value — the cl 16.00.11886 toolchain-version stamp. Constant.
const COMP_ID_VALUE: u32 = 0x00AB_2E6E;

/// External watermark symbol names (constant toolchain build-number watermarks).
const NAME_C2: &str = "__C2_11886";
const NAME_C1: &str = "__C1_11886";

// COFF machine + characteristics.
const MACHINE_POWERPCBE: u16 = 0x01F2;
const CHARACTERISTICS: u16 = 0x0180;

// Section characteristics (verified constant across fixtures).
const CH_DRECTVE: u32 = 0x0010_0A00;
const CH_DEBUGS: u32 = 0x4210_0040;
const CH_XBLD_C2: u32 = 0xC040_1040;
const CH_XBLD_C1: u32 = 0xC230_1040;
const CH_TEXT: u32 = 0x6040_0020;

const SECTION_HEADER_LEN: usize = 40;
const COFF_HEADER_LEN: usize = 20;

/// A little-endian byte sink.
struct Buf(Vec<u8>);
impl Buf {
    fn new() -> Self {
        Buf(Vec::new())
    }
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i16(&mut self, v: i16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn bytes(&mut self, v: &[u8]) {
        self.0.extend_from_slice(v);
    }
    /// 8-byte NUL-padded short name (`len <= 8`).
    fn name8(&mut self, name: &str) {
        let b = name.as_bytes();
        assert!(b.len() <= 8, "short name > 8 bytes: {name}");
        self.0.extend_from_slice(b);
        for _ in b.len()..8 {
            self.0.push(0);
        }
    }
}

/// Build the `.debug$S` raw section: CV signature + one `0xF1` SYMBOLS
/// subsection (S_OBJNAME with the `-Fo` path, then the constant S_COMPILE2),
/// padded to a 4-byte multiple.
fn build_debug_s(obj_name: &str) -> Vec<u8> {
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

/// A section, resolved to its raw data + header metadata.
struct Section {
    name: &'static str,
    characteristics: u32,
    raw: Vec<u8>,
    /// Aux section-def CheckSum (0 for non-COMDAT).
    checksum: u32,
    /// COMDAT selection (0 = not COMDAT; 2 = SELECT_ANY).
    selection: u8,
}

/// Build the complete MVP `.obj` image bytes.
///
/// * `obj_name` — the `-Fo` output-path string exactly as the reference saw it
///   (e.g. `Z:\tmp\anat\mvp.obj`); embedded verbatim in `.debug$S` S_OBJNAME.
/// * `mangled_name` — the function's mangled symbol (from `.gl`), e.g.
///   `?add3@@YAHHHH@Z`.
/// * `text` — the `.text` bytes from codegen (12 for `add3`).
pub fn emit_mvp_obj(obj_name: &str, mangled_name: &str, text: &[u8]) -> Vec<u8> {
    emit_obj(obj_name, &[Function { name: mangled_name, text_offset: 0 }], text)
}

/// One function placed in `.text`: its mangled name (from `.gl`) and byte
/// offset within the concatenated `.text` payload.
pub struct Function<'a> {
    pub name: &'a str,
    pub text_offset: u32,
}

/// Build the complete `.obj` image for one or more straight-line functions
/// sharing a single `.text`. Generalizes [`emit_mvp_obj`]: functions are packed
/// contiguously in `.text` (no inter-function padding — c2's real layout), each
/// gets an EXTERNAL FUNCTION symbol whose `Value` is its `.text` byte offset,
/// and `NumberOfSymbols` = 13 fixed slots + one per function.
///
/// * `obj_name` — the `-Fo` path (embedded in `.debug$S` S_OBJNAME).
/// * `funcs` — functions in emit order (matches `.gl`/`.ex` order); each
///   `text_offset` is its start within `text`.
/// * `text` — the full concatenated `.text` bytes from codegen.
pub fn emit_obj(obj_name: &str, funcs: &[Function], text: &[u8]) -> Vec<u8> {
    let debug_s = build_debug_s(obj_name);

    // Section table, in the fixed emit order.
    let sections = [
        Section {
            name: ".drectve",
            characteristics: CH_DRECTVE,
            raw: DRECTVE.to_vec(),
            checksum: 0,
            selection: 0,
        },
        Section {
            name: ".debug$S",
            characteristics: CH_DEBUGS,
            raw: debug_s,
            checksum: 0,
            selection: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C2,
            raw: XBLD_C2.to_vec(),
            checksum: XBLD_C2_CHECKSUM,
            selection: 2,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C1,
            raw: XBLD_C1.to_vec(),
            checksum: XBLD_C1_CHECKSUM,
            selection: 2,
        },
        Section {
            name: ".text",
            characteristics: CH_TEXT,
            raw: text.to_vec(),
            checksum: 0,
            selection: 0,
        },
    ];
    let n_sections = sections.len();

    // Raw data is packed contiguously right after the section headers.
    let raw_base = COFF_HEADER_LEN + n_sections * SECTION_HEADER_LEN;
    let mut ptrs = Vec::with_capacity(n_sections);
    let mut cursor = raw_base;
    for s in &sections {
        ptrs.push(cursor);
        cursor += s.raw.len();
    }
    let ptr_symtab = cursor; // symbol table right after last raw section

    // 13 fixed slots (@comp.id, 4 section symbols + their aux, 2 externals) plus
    // one EXTERNAL FUNCTION symbol per function. NumberOfSymbols counts aux.
    let n_symbols: u32 = 13 + funcs.len() as u32;

    // ---- COFF header (20 bytes) ----
    let mut b = Buf::new();
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp — normalized away
    b.u32(ptr_symtab as u32);
    b.u32(n_symbols);
    b.u16(0); // SizeOfOptionalHeader
    b.u16(CHARACTERISTICS);

    // ---- section headers (40 bytes each) ----
    for (i, s) in sections.iter().enumerate() {
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.raw.len() as u32); // SizeOfRawData
        b.u32(ptrs[i] as u32); // PointerToRawData
        b.u32(0); // PointerToRelocations
        b.u32(0); // PointerToLinenumbers
        b.u16(0); // NumberOfRelocations
        b.u16(0); // NumberOfLinenumbers
        b.u32(s.characteristics);
    }

    // ---- raw section data (packed) ----
    for s in &sections {
        b.bytes(&s.raw);
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table + string table ----
    // Long-name string table, built in first-reference order.
    let mut strtab = StringTable::new();

    // slot 0: @comp.id (ABS, STATIC, no aux)
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1); // IMAGE_SYM_ABSOLUTE
    b.u16(0x0000);
    b.u8(3); // STATIC
    b.u8(0);

    // Section STATIC symbols each carry one aux section-def record. Section
    // numbers are 1-based in emit order.
    // slot 1/2: .drectve (sec 1)
    emit_section_symbol(&mut b, &sections[0], 1);
    // slot 3/4: .debug$S (sec 2)
    emit_section_symbol(&mut b, &sections[1], 2);
    // slot 5/6: .XBLD$W C2 (sec 3), followed by external __C2_11886
    emit_section_symbol(&mut b, &sections[2], 3);
    emit_external_symbol(&mut b, &mut strtab, NAME_C2, 3, 0x0000);
    // slot 8/9: .XBLD$W C1 (sec 4), followed by external __C1_11886
    emit_section_symbol(&mut b, &sections[3], 4);
    emit_external_symbol(&mut b, &mut strtab, NAME_C1, 4, 0x0000);
    // slot 11/12: .text (sec 5)
    emit_section_symbol(&mut b, &sections[4], 5);
    // slot 13…: one EXTERNAL FUNCTION symbol per function (type 0x20, sec .text),
    // Value = its byte offset within .text, in emit order.
    for f in funcs {
        emit_function_symbol(&mut b, &mut strtab, f.name, 5, f.text_offset);
    }

    // ---- string table ----
    b.bytes(&strtab.finish());

    b.0
}

/// Emit a section STATIC symbol + its aux section-def record.
fn emit_section_symbol(b: &mut Buf, s: &Section, sec_num: i16) {
    b.name8(s.name);
    b.u32(0); // Value
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(3); // STATIC
    b.u8(1); // one aux record

    // Aux section-def: Length | nReloc(u16) | nLineno(u16) | CheckSum(u32) |
    //                  Number(u16) | Selection(u8) | Unused[3].
    b.u32(s.raw.len() as u32);
    b.u16(0); // NumberOfRelocations (MVP: none)
    b.u16(0); // NumberOfLinenumbers
    b.u32(s.checksum);
    b.u16(0); // Number (SELECT_ANY → 0)
    b.u8(s.selection);
    b.bytes(&[0, 0, 0]); // Unused
}

/// Emit an EXTERNAL symbol whose (long) name lives in the string table.
fn emit_external_symbol(b: &mut Buf, strtab: &mut StringTable, name: &str, sec_num: i16, typ: u16) {
    if name.len() <= 8 {
        b.name8(name);
    } else {
        let off = strtab.intern(name);
        b.u32(0); // long-name marker
        b.u32(off);
    }
    b.u32(0); // Value (fn offset in .text = 0 for the single MVP fn)
    b.i16(sec_num);
    b.u16(typ);
    b.u8(2); // EXTERNAL
    b.u8(0); // no aux
}

/// Emit an EXTERNAL FUNCTION symbol (type 0x20) whose (long) name lives in the
/// string table, with `Value` = its byte offset within `.text`.
fn emit_function_symbol(b: &mut Buf, strtab: &mut StringTable, name: &str, sec_num: i16, value: u32) {
    if name.len() <= 8 {
        b.name8(name);
    } else {
        let off = strtab.intern(name);
        b.u32(0); // long-name marker
        b.u32(off);
    }
    b.u32(value); // Value = fn offset within .text
    b.i16(sec_num);
    b.u16(0x0020); // DTYPE_FUNCTION
    b.u8(2); // EXTERNAL
    b.u8(0); // no aux
}

/// COFF string table: `Size:u32(incl self)` + NUL-terminated names in
/// first-reference order. Offsets returned are from the table base (so the
/// first name is at offset 4).
struct StringTable {
    names: Vec<(String, u32)>,
    cursor: u32,
}
impl StringTable {
    fn new() -> Self {
        StringTable {
            names: Vec::new(),
            cursor: 4, // past the 4-byte size word
        }
    }
    /// Intern a name (append if new), returning its byte offset.
    fn intern(&mut self, name: &str) -> u32 {
        if let Some((_, off)) = self.names.iter().find(|(n, _)| n == name) {
            return *off;
        }
        let off = self.cursor;
        self.cursor += name.len() as u32 + 1; // + NUL
        self.names.push((name.to_string(), off));
        off
    }
    fn finish(self) -> Vec<u8> {
        let mut out = Buf::new();
        out.u32(self.cursor); // Size includes the size word itself
        for (name, _) in &self.names {
            out.bytes(name.as_bytes());
            out.u8(0);
        }
        out.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drectve_is_132_bytes() {
        assert_eq!(DRECTVE.len(), 132, "drectve must be exactly 132 bytes");
    }

    #[test]
    fn s_compile2_is_57_bytes() {
        assert_eq!(S_COMPILE2.len(), 57);
    }

    #[test]
    fn debug_s_size_for_mvp_path() {
        // "Z:\tmp\anat\mvp.obj" = 19 chars → raw 97 → padded 100.
        let d = build_debug_s(r"Z:\tmp\anat\mvp.obj");
        assert_eq!(d.len(), 100);
    }
}
