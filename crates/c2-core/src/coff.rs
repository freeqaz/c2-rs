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
/// One COFF relocation record: VirtualAddress u32, SymbolTableIndex u32,
/// Type u16 (packed, not padded).
const RELOC_LEN: usize = 10;

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
    emit_obj(
        obj_name,
        &[Function {
            name: mangled_name,
            text_offset: 0,
            call: None,
        }],
        text,
    )
}

/// A relative-branch (REL24) relocation for a tail call: the callee's mangled
/// name and the `.text` byte offset of the branch instruction to patch.
pub struct Call<'a> {
    pub reloc_offset: u32,
    pub callee: &'a str,
}

/// One function placed in `.text`: its mangled name (from `.gl`), byte offset
/// within the concatenated `.text`, and — if it is a tail call — the callee
/// relocation.
pub struct Function<'a> {
    pub name: &'a str,
    pub text_offset: u32,
    pub call: Option<Call<'a>>,
}

/// IMAGE_REL_PPC_REL24 — 24-bit relative branch relocation (tail/`bl` calls).
const REL_PPC_REL24: u16 = 0x0006;
/// IMAGE_REL_PPC_ADDR32 — 32-bit VA relocation (the `.pdata` BeginAddress).
const REL_PPC_ADDR32: u16 = 0x0002;

/// `.pdata` section characteristics: CNT_INIT_DATA | ALIGN_8 | MEM_READ.
const CH_PDATA: u32 = 0x4040_0040;

/// Reflected CRC-32 (poly `0xEDB88320`, init 0, no final inversion) over a
/// section's raw bytes — the COFF aux section-def CheckSum algorithm. Used for
/// `.pdata` (whose aux carries a real checksum even though it is not a COMDAT);
/// the fixed `.XBLD$W` COMDAT checksums stay hardcoded above.
fn coff_checksum(data: &[u8]) -> u32 {
    let mut c: u32 = 0;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    c
}

/// Build the 8-byte X360 `RUNTIME_FUNCTION` for a framed `.text` of `text_len`
/// bytes: `BeginAddress` (u32 = 0, patched by the ADDR32 relocation) then the
/// packed unwind word, both **big-endian** (like `.text`, unlike COFF fields).
///
/// The packed word is `0x40000000 | (function_length_words << 8) |
/// prolog_length_words`, verified by diffing the 0x24-byte `+k` body
/// (`0x40000903`, 9 words) against the 0x28-byte `*5` body (`0x40000A03`,
/// 10 words) — incrementing the length adds `0x100`. The prologue
/// (`mflr;stw;stwu`) is 3 words for this frame class.
fn build_pdata(text_len: usize) -> Vec<u8> {
    let function_words = (text_len / 4) as u32;
    let prolog_words = 3u32;
    let unwind = 0x4000_0000u32 | (function_words << 8) | prolog_words;
    let mut b = Vec::with_capacity(8);
    b.extend_from_slice(&0u32.to_be_bytes()); // BeginAddress (reloc-patched)
    b.extend_from_slice(&unwind.to_be_bytes()); // packed unwind word
    b
}

/// Emit the 6-section `.obj` for a **framed non-leaf call** `int f(int a){
/// return g(a) + k; }` (W4b2). Adds a `.pdata` unwind section and the
/// compiler-generated label symbols ($M2545/$M2546/$T2547) on top of the leaf
/// layout; the 5-section [`emit_obj`] path is untouched for leaf/tail TUs.
///
/// Scope: a **single-function TU** with one external callee. The $M/$T label
/// counters are a fixed toolchain seed (`2545/2546/2547`) only for the first
/// function of the TU (W-UNW-1 probe) — so the emitter hardcodes those names
/// and the full 20-symbol layout in the observed slot order.
///
/// * `obj_name`   — the `-Fo` path (embedded in `.debug$S` S_OBJNAME).
/// * `func_name`  — the defined function's mangled name (`?f@@YAHH@Z`).
/// * `callee_name`— the external callee's mangled name (`?g@@YAHH@Z`).
/// * `text`       — the framed `.text` from codegen (0x24 bytes).
pub fn emit_framed_obj(obj_name: &str, func_name: &str, callee_name: &str, text: &[u8]) -> Vec<u8> {
    let debug_s = build_debug_s(obj_name);
    let pdata = build_pdata(text.len());
    let pdata_checksum = coff_checksum(&pdata);

    // Six sections, fixed order. `.text` and `.pdata` each carry one relocation.
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
        Section {
            name: ".pdata",
            characteristics: CH_PDATA,
            raw: pdata,
            checksum: pdata_checksum,
            selection: 0,
        },
    ];
    let n_sections = sections.len();

    // --- file layout ---
    // Sections 0..=4 raw data is packed contiguously; then MSVC writes each
    // remaining section's raw + its relocations *interleaved* in section order:
    // `.text` raw, `.text` reloc, `.pdata` raw, `.pdata` reloc, symbol table.
    // (Verified against the reference obj: `.pdata` raw sits AFTER the `.text`
    // relocation block, not contiguous with `.text` raw.)
    let raw_base = COFF_HEADER_LEN + n_sections * SECTION_HEADER_LEN;
    let mut ptr_raw = [0usize; 6];
    let mut ptr_reloc = [0usize; 6];
    let mut cursor = raw_base;
    // sections 0..=4 raw, contiguous.
    for i in 0..5 {
        ptr_raw[i] = cursor;
        cursor += sections[i].raw.len();
    }
    // .text (idx 4) reloc immediately follows its raw.
    ptr_reloc[4] = cursor;
    cursor += RELOC_LEN;
    // .pdata (idx 5) raw, then its reloc.
    ptr_raw[5] = cursor;
    cursor += sections[5].raw.len();
    ptr_reloc[5] = cursor;
    cursor += RELOC_LEN;
    let ptr_symtab = cursor;

    // Fixed 20-symbol layout (single-function TU). Reloc symbol indices are
    // hardcoded to match the observed table: the `bl` REL24 targets `?g`
    // (idx 15); the `.pdata` ADDR32 targets `?f` (idx 13).
    const SYM_F: u32 = 13;
    const SYM_G: u32 = 15;
    let n_symbols: u32 = 20;

    // ---- COFF header ----
    let mut b = Buf::new();
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp — normalized away
    b.u32(ptr_symtab as u32);
    b.u32(n_symbols);
    b.u16(0); // SizeOfOptionalHeader
    b.u16(CHARACTERISTICS);

    // ---- section headers ----
    for (i, s) in sections.iter().enumerate() {
        let (prel, nrel) = match i {
            4 | 5 => (ptr_reloc[i] as u32, 1u16), // .text / .pdata each have 1
            _ => (0, 0),
        };
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.raw.len() as u32); // SizeOfRawData
        b.u32(ptr_raw[i] as u32); // PointerToRawData
        b.u32(prel); // PointerToRelocations
        b.u32(0); // PointerToLinenumbers
        b.u16(nrel); // NumberOfRelocations
        b.u16(0); // NumberOfLinenumbers
        b.u32(s.characteristics);
    }

    // ---- interleaved raw + relocations ----
    for i in 0..5 {
        b.bytes(&sections[i].raw);
    }
    debug_assert_eq!(b.0.len(), ptr_reloc[4]);
    // .text relocation: the `bl` REL24 at FRAMED_BL_OFFSET → callee `?g`.
    b.u32(crate::codegen::FRAMED_BL_OFFSET);
    b.u32(SYM_G);
    b.u16(REL_PPC_REL24);
    debug_assert_eq!(b.0.len(), ptr_raw[5]);
    b.bytes(&sections[5].raw);
    debug_assert_eq!(b.0.len(), ptr_reloc[5]);
    // .pdata relocation: ADDR32 at va=0 (BeginAddress) → defined function `?f`.
    b.u32(0);
    b.u32(SYM_F);
    b.u16(REL_PPC_ADDR32);
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table (fixed 20-slot order) + string table ----
    let mut strtab = StringTable::new();
    let text_len = text.len() as u32;

    // 0: @comp.id
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1);
    b.u16(0x0000);
    b.u8(3);
    b.u8(0);

    emit_section_symbol(&mut b, &sections[0], 1, 0); // 1/2  .drectve
    emit_section_symbol(&mut b, &sections[1], 2, 0); // 3/4  .debug$S
    emit_section_symbol(&mut b, &sections[2], 3, 0); // 5/6  .XBLD$W C2
    emit_external_symbol(&mut b, &mut strtab, NAME_C2, 3, 0x0000); // 7
    emit_section_symbol(&mut b, &sections[3], 4, 0); // 8/9  .XBLD$W C1
    emit_external_symbol(&mut b, &mut strtab, NAME_C1, 4, 0x0000); // 10
    emit_section_symbol(&mut b, &sections[4], 5, 1); // 11/12 .text (1 reloc)

    // 13: ?f — defined function in .text.
    emit_function_symbol(&mut b, &mut strtab, func_name, 5, 0);
    // 14: $M2546 — label at end of .text (value = text length).
    emit_label_symbol(&mut b, "$M2546", text_len, 5);
    // 15: ?g — undefined external callee (section 0, FUNCTION type).
    emit_function_symbol(&mut b, &mut strtab, callee_name, 0, 0);
    // 16: $M2545 — label at the `bl` site.
    emit_label_symbol(&mut b, "$M2545", crate::codegen::FRAMED_BL_OFFSET, 5);
    // 17/18: .pdata section symbol + aux (1 reloc, real CRC checksum).
    emit_section_symbol(&mut b, &sections[5], 6, 1);
    // 19: $T2547 — the `.pdata` label (storage class 3, not 6).
    b.name8("$T2547");
    b.u32(0); // Value
    b.i16(6); // .pdata
    b.u16(0x0000);
    b.u8(3); // STATIC
    b.u8(0);

    b.bytes(&strtab.finish());
    b.0
}

/// Emit a compiler-generated **label** symbol (storage class 6, no aux) with an
/// inline short name, e.g. `$M2545`/`$M2546`. `value` is its `.text` offset.
fn emit_label_symbol(b: &mut Buf, name: &str, value: u32, sec_num: i16) {
    b.name8(name);
    b.u32(value);
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(6); // IMAGE_SYM_CLASS_LABEL
    b.u8(0); // no aux
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
    // Relocations (`.text` only in this class) sit between the raw data and the
    // symbol table. Each function that is a tail call contributes one REL24.
    let n_text_reloc = funcs.iter().filter(|f| f.call.is_some()).count();
    let ptr_text_reloc = cursor; // right after the last section's raw data
    cursor += n_text_reloc * RELOC_LEN;
    let ptr_symtab = cursor; // symbol table right after the relocations

    // Symbol layout: 13 fixed slots (indices 0..13), then per function a defined
    // FUNCTION symbol, each immediately followed by its callee's undefined
    // external symbol (if any). Record each callee's symbol index for its reloc.
    let mut next_idx: u32 = 13;
    let mut plan: Vec<(usize, u32, Option<u32>)> = Vec::with_capacity(funcs.len());
    for (i, f) in funcs.iter().enumerate() {
        let def_idx = next_idx;
        next_idx += 1;
        let callee_idx = if f.call.is_some() {
            let c = next_idx;
            next_idx += 1;
            Some(c)
        } else {
            None
        };
        plan.push((i, def_idx, callee_idx));
    }
    let n_symbols: u32 = next_idx;

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
    // Only `.text` (the last section) carries relocations in this class.
    let text_idx = n_sections - 1;
    for (i, s) in sections.iter().enumerate() {
        let (prel, nrel) = if i == text_idx && n_text_reloc > 0 {
            (ptr_text_reloc as u32, n_text_reloc as u16)
        } else {
            (0, 0)
        };
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.raw.len() as u32); // SizeOfRawData
        b.u32(ptrs[i] as u32); // PointerToRawData
        b.u32(prel); // PointerToRelocations
        b.u32(0); // PointerToLinenumbers
        b.u16(nrel); // NumberOfRelocations
        b.u16(0); // NumberOfLinenumbers
        b.u32(s.characteristics);
    }

    // ---- raw section data (packed) ----
    for s in &sections {
        b.bytes(&s.raw);
    }
    debug_assert_eq!(b.0.len(), ptr_text_reloc);

    // ---- relocation records (10 bytes each: VA u32, SymIdx u32, Type u16) ----
    for (i, _def, callee_idx) in &plan {
        if let (Some(call), Some(cidx)) = (&funcs[*i].call, callee_idx) {
            b.u32(call.reloc_offset);
            b.u32(*cidx);
            b.u16(REL_PPC_REL24);
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table + string table ----
    let mut strtab = StringTable::new();

    // slot 0: @comp.id (ABS, STATIC, no aux)
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1); // IMAGE_SYM_ABSOLUTE
    b.u16(0x0000);
    b.u8(3); // STATIC
    b.u8(0);

    // Section STATIC symbols each carry one aux section-def record. `.text`
    // (sec 5) carries the relocation count in its aux.
    emit_section_symbol(&mut b, &sections[0], 1, 0); // slot 1/2 .drectve
    emit_section_symbol(&mut b, &sections[1], 2, 0); // slot 3/4 .debug$S
    emit_section_symbol(&mut b, &sections[2], 3, 0); // slot 5/6 .XBLD$W C2
    emit_external_symbol(&mut b, &mut strtab, NAME_C2, 3, 0x0000); // slot 7
    emit_section_symbol(&mut b, &sections[3], 4, 0); // slot 8/9 .XBLD$W C1
    emit_external_symbol(&mut b, &mut strtab, NAME_C1, 4, 0x0000); // slot 10
    emit_section_symbol(&mut b, &sections[4], 5, n_text_reloc as u16); // slot 11/12 .text

    // Per function: the defined FUNCTION symbol, then (if a tail call) the
    // undefined external callee symbol.
    for (i, _def, callee_idx) in &plan {
        let f = &funcs[*i];
        emit_function_symbol(&mut b, &mut strtab, f.name, 5, f.text_offset);
        if let (Some(call), Some(_)) = (&f.call, callee_idx) {
            // Undefined external callee: section 0 (UNDEF), FUNCTION type.
            emit_function_symbol(&mut b, &mut strtab, call.callee, 0, 0);
        }
    }

    // ---- string table ----
    b.bytes(&strtab.finish());

    b.0
}

/// Emit a section STATIC symbol + its aux section-def record. `n_reloc` is the
/// section's relocation count (0 for all sections except `.text` when calls
/// are present) — it appears in the aux record and must match the section
/// header's `NumberOfRelocations`.
fn emit_section_symbol(b: &mut Buf, s: &Section, sec_num: i16, n_reloc: u16) {
    b.name8(s.name);
    b.u32(0); // Value
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(3); // STATIC
    b.u8(1); // one aux record

    // Aux section-def: Length | nReloc(u16) | nLineno(u16) | CheckSum(u32) |
    //                  Number(u16) | Selection(u8) | Unused[3].
    b.u32(s.raw.len() as u32);
    b.u16(n_reloc); // NumberOfRelocations
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
    fn pdata_unwind_word_encodes_function_length() {
        // 0x24 body (9 words, +k class) → BeginAddress 0 + big-endian
        // 0x40000903. 0x28 body (10 words, *5) → 0x40000A03 (length +1 = +0x100).
        assert_eq!(
            build_pdata(0x24),
            [0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x09, 0x03]
        );
        assert_eq!(
            build_pdata(0x28),
            [0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x0A, 0x03]
        );
    }

    #[test]
    fn pdata_checksum_matches_reference_aux() {
        // The `.pdata` aux CheckSum in the reference obj (0xd3dfb2ce for the +k
        // frame) is the reflected CRC-32 of the 8 raw bytes.
        assert_eq!(coff_checksum(&build_pdata(0x24)), 0xD3DF_B2CE);
        assert_eq!(coff_checksum(&build_pdata(0x28)), 0xF8F2_E10D);
    }

    #[test]
    fn framed_obj_has_six_sections_and_twenty_symbols() {
        // A framed obj built with the verified 0x24 text: 6 sections, 20 symbols.
        let text = vec![0u8; 0x24];
        let obj = emit_framed_obj(r"Z:\t\f.obj", "?f@@YAHH@Z", "?g@@YAHH@Z", &text);
        assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 6); // NumberOfSections
        assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 20); // NumberOfSymbols
    }

    #[test]
    fn debug_s_size_for_mvp_path() {
        // "Z:\tmp\anat\mvp.obj" = 19 chars → raw 97 → padded 100.
        let d = build_debug_s(r"Z:\tmp\anat\mvp.obj");
        assert_eq!(d.len(), 100);
    }
}
