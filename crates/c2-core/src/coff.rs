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
/// One COFF symbol-table record (also the aux-record stride).
const SYMBOL_LEN: usize = 18;

/// A little-endian byte sink.
struct Buf(Vec<u8>);
impl Buf {
    fn new() -> Self {
        Buf(Vec::new())
    }
    /// Pre-sized sink for the whole-obj emitters: the layout pass has already
    /// computed the symbol-table offset, so the final size is known to within
    /// the string table. Capacity is invisible in the output — this only
    /// removes the realloc-and-copy churn of growing from empty.
    fn with_capacity(n: usize) -> Self {
        Buf(Vec::with_capacity(n))
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

/// Which function a `/Gy` section belongs to. The COMDAT layout interleaves
/// `.text` and `.pdata` per function, so "section index minus the fixed prefix"
/// is **not** the function index once any function is framed — that arithmetic
/// is what this replaces.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SectionOwner {
    /// One of the four fixed sections every obj carries.
    Fixed,
    /// Function `i`'s own `.text` COMDAT.
    Text(usize),
    /// Function `i`'s `.pdata` COMDAT.
    Pdata(usize),
}

/// A section, resolved to its raw data + header metadata.
struct Section<'a> {
    name: &'static str,
    characteristics: u32,
    /// Raw section data. Borrowed for the fixed blobs (`.drectve`, the XBLD
    /// watermarks) and the caller's `.text`; owned only where it is actually
    /// built per obj (`.debug$S`, `.pdata`, the `.rdata` pools). The emitted
    /// bytes are identical either way — this only removes per-obj copies.
    raw: std::borrow::Cow<'a, [u8]>,
    /// Aux section-def CheckSum (0 for non-COMDAT).
    checksum: u32,
    /// COMDAT selection (0 = not COMDAT; 2 = SELECT_ANY; 1 = NODUPLICATES;
    /// 5 = ASSOCIATIVE).
    selection: u8,
    /// Aux section-def `Number`. Zero everywhere except a Selection=5
    /// (ASSOCIATIVE) COMDAT, where it is the **1-based section number of the
    /// section this one is tied to** — the mechanism `/Gy` uses to attach a
    /// function's `.pdata` COMDAT to its `.text` COMDAT so the linker discards
    /// both together.
    assoc: u16,
    /// `Some(n)` for an **uninitialized** section (`.bss`): the section is `n`
    /// bytes long as far as `SizeOfRawData` and the aux `Length` are concerned,
    /// but it contributes **zero bytes to the file** and its
    /// `PointerToRawData` is 0.
    ///
    /// This inversion is the doc's refuted prediction P8
    /// (`docs/OBJ_DYNINIT_SHAPE.md` §1) — the natural guess is
    /// `SizeOfRawData = 0` with the size in `VirtualSize`, and c2 does the exact
    /// opposite: `VirtualSize` is 0 in **every** section including `.bss`.
    /// Everything else in this file conflates "how many bytes the section is"
    /// with `raw.len()`, in four places (the header, the layout cursor, the raw
    /// write and the aux `Length`), which is why this is a field on `Section`
    /// and not a special case at the one call site — three of those four would
    /// still have been wrong.
    uninit_size: Option<u32>,
}

impl Section<'_> {
    /// The section's length as the container reports it — `SizeOfRawData` and
    /// the aux section-def `Length`. Equal to `raw.len()` except for `.bss`.
    fn size(&self) -> u32 {
        self.uninit_size.unwrap_or(self.raw.len() as u32)
    }
    /// How many bytes this section contributes to the obj **file**. Zero for an
    /// uninitialized section, whatever its [`Section::size`].
    fn file_len(&self) -> usize {
        if self.uninit_size.is_some() { 0 } else { self.raw.len() }
    }
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
fn shell_sections<'a>(obj_name: &str) -> Vec<Section<'a>> {
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

/// Lay the obj out after the section headers and return
/// `(PointerToRawData per section, PointerToRelocations per section, the
/// symbol-table offset)`.
///
/// Two rules, both of which this file has already been wrong about once:
///
/// * **a section's relocations immediately follow its OWN raw data**, before the
///   next section's — not all raw data then all relocations. That was invisible
///   while at most one section had relocations, and wrong from the fifth section
///   header on once two did (`docs/ROADMAP.md`; the note on
///   [`emit_comdat_obj`]).
/// * **an uninitialized section advances the cursor by nothing and gets
///   `PointerToRawData = 0`**, however large its [`Section::size`] — `.bss` sits
///   between `.rdata` and `.CRT$XCU` in the dynamic-initializer obj and those
///   two are contiguous in the file.
fn layout_sections(
    sections: &[Section],
    n_reloc_of: &[u16],
) -> (Vec<usize>, Vec<Option<usize>>, usize) {
    debug_assert_eq!(sections.len(), n_reloc_of.len());
    let mut ptrs = Vec::with_capacity(sections.len());
    let mut reloc_ptr: Vec<Option<usize>> = vec![None; sections.len()];
    let mut cursor = COFF_HEADER_LEN + sections.len() * SECTION_HEADER_LEN;
    for (i, s) in sections.iter().enumerate() {
        if s.uninit_size.is_some() {
            ptrs.push(0);
        } else {
            ptrs.push(cursor);
            cursor += s.raw.len();
        }
        if n_reloc_of[i] > 0 {
            reloc_ptr[i] = Some(cursor);
            cursor += n_reloc_of[i] as usize * RELOC_LEN;
        }
    }
    (ptrs, reloc_ptr, cursor)
}

/// The 20-byte COFF header. `TimeDateStamp` is 0 — the differential normalizes
/// it away; every other byte must genuinely match.
fn write_coff_header(b: &mut Buf, n_sections: usize, ptr_symtab: usize, n_symbols: u32) {
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp
    b.u32(ptr_symtab as u32);
    b.u32(n_symbols);
    b.u16(0); // SizeOfOptionalHeader
    b.u16(CHARACTERISTICS);
}

/// The section headers, 40 bytes each. `VirtualSize`, `VirtualAddress`,
/// `PointerToLinenumbers` and `NumberOfLinenumbers` are 0 in every section of
/// every reference obj measured — **including `.bss`**, whose size lives in
/// `SizeOfRawData` beside a null `PointerToRawData`.
fn write_section_headers(
    b: &mut Buf,
    sections: &[Section],
    ptrs: &[usize],
    reloc_ptr: &[Option<usize>],
    n_reloc_of: &[u16],
) {
    for (i, s) in sections.iter().enumerate() {
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.size()); // SizeOfRawData
        b.u32(ptrs[i] as u32); // PointerToRawData
        b.u32(reloc_ptr[i].unwrap_or(0) as u32);
        b.u32(0); // PointerToLinenumbers
        b.u16(n_reloc_of[i]);
        b.u16(0); // NumberOfLinenumbers
        b.u32(s.characteristics);
    }
}

/// Symbol records 0..=10 — the fixed prefix every obj carries, identical in all
/// 61 reference objs measured (`docs/OBJ_DYNINIT_SHAPE.md` §4.1):
/// `@comp.id`, then the four shell sections' STATIC section symbols with their
/// aux records, with the two watermark externals after their own `.XBLD$W`.
///
/// `sections` must begin with [`shell_sections`]' four.
fn emit_shell_symbols(b: &mut Buf, strtab: &mut StringTable, sections: &[Section]) {
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
const N_SHELL_SYMBOLS: u32 = 11;

/// Build the complete MVP `.obj` image bytes.
///
/// * `obj_name` — the `-Fo` output-path string exactly as the reference saw it
///   (e.g. `Z:\tmp\anat\mvp.obj`); embedded verbatim in `.debug$S` S_OBJNAME.
/// * `mangled_name` — the function's mangled symbol (from `.gl`), e.g.
///   `?add3@@YAHHHH@Z`.
/// * `text` — the `.text` bytes from codegen (12 for `add3`).
pub fn emit_mvp_obj(obj_name: &str, mangled_name: &str, text: &[u8]) -> Vec<u8> {
    // Label counter unused: a `Function::plain` has no frame, so no `$M`/`$T`.
    emit_obj(obj_name, &[Function::plain(mangled_name, 0)], text, 0)
}

/// A relative-branch (REL24) relocation for a tail call: the callee's mangled
/// name and the `.text` byte offset of the branch instruction to patch.
pub struct Call<'a> {
    pub reloc_offset: u32,
    pub callee: &'a str,
}

/// **WR1 — one reference to a NAMED DATA SYMBOL's address**: the `.text` byte
/// offset of the `lis rS,sym@ha` that opens it, plus the symbol's mangled name.
///
/// **The two halves are NOT adjacent, and that is the one place this differs from
/// [`crate::codegen::FpConstRef`].** The `lis` is hoisted to the top of the body
/// while the `addi rD,rS,sym@l` takes its own argument slot's turn in the
/// descending setup walk, so a literal slot above it lands *between* them —
/// MEASURED (`work/wr1/probes/p4.cpp`, `void a7(){ gsp(&gI, 7); }`):
/// `lis r11 · li r4,7 · addi r3,r11,0 · b`, with REFHI at the function's start
/// and REFLO **eight** bytes later, not four. Carrying one offset and adding 4
/// was a live wrong-bytes emit on exactly that body, caught by the differential
/// before it left this worktree.
///
/// Four relocation records: REFHI + PAIR at `hi_off`, REFLO + PAIR at `lo_off`,
/// both PAIRs against symbol index 0.
///
/// The symbol itself is an **undefined external DATA** symbol — `Type` 0x0000,
/// where a callee carries 0x0020 — emitted in this function's group after its
/// callee externals. MEASURED (`work/wr1/probes/p1.cpp`): `void f(){ gso(&gI); }`
/// gives `?f5@@YAXXZ`, `?gso@@YAXPAH@Z`, `?gI@@3HA`, in that order, with the
/// callee ahead of the data symbol because its `26` push precedes the argument's
/// (`docs/IL_CALL_IN_EXPR.md` §17.2 item 6).
pub struct DataRef<'a> {
    pub hi_off: u32,
    pub lo_off: u32,
    pub name: &'a str,
}

/// One function placed in `.text`: its mangled name (from `.gl`), byte offset
/// within the concatenated `.text`, and one relocation per call it makes.
pub struct Function<'a> {
    pub name: &'a str,
    pub text_offset: u32,
    /// Every REL24 site this function contributes, in ascending `.text` offset —
    /// a tail call's `b`, a framed call's `bl`, or one `bl` per call of a Class A
    /// many-call body. **A list, not an `Option`**: the shipped framed class had
    /// exactly one call site, and every "the" in this file's relocation and
    /// symbol code was that constant.
    ///
    /// Duplicates are expected and are not an error. `void f(){ g(); h(); g(); }`
    /// has three sites and **two** external symbols: c2 emits one undefined
    /// external per distinct callee and relocates every later site against that
    /// same index (measured — both `?g1` relocations in the three-call probe point
    /// at symbol 16).
    pub calls: Vec<Call<'a>>,
    /// True iff this function's body **touches floating point** in any way. The
    /// obj then carries an undefined external `_fltused`, emitted immediately
    /// after the FIRST such function's symbol group — the CRT's float-support
    /// hook. Verified: a pure FP leaf changes the obj shell by exactly this one
    /// symbol (`docs/CODEGEN_W13_FLOAT.md` §4).
    ///
    /// **"Touches FP" and "is a float leaf" are two facts**, and they shared this
    /// field until the FP store leaf pulled them apart:
    /// `void f(S* s, float v){ s->f = v; }` needs the marker and is a store leaf
    /// with a label stride of 1, not 2. The producer is
    /// [`c2_il::IlFunction::touches_floating_point`]; the stride is
    /// [`c2_il::IlFunction::label_slots`]. One field, two readers, and the
    /// mismatch it caused was 14 out of 14 objs short by one symbol.
    pub is_float: bool,
    /// W13b: this function's floating-point constant reference sites, in
    /// emission order, with `hi_off` already rebased to the whole `.text`.
    pub fp_refs: Vec<crate::codegen::FpConstRef>,
    /// **WR1**: this function's named-data-symbol address references, in emission
    /// order, with `hi_off` already rebased to the whole `.text`. At most one in
    /// the class the parser admits; a `Vec` because the relocation and symbol code
    /// below is written over a list either way and a "the" here would be the same
    /// constant [`Function::calls`]' own comment records having been.
    pub data_refs: Vec<DataRef<'a>>,
    /// `Some` iff this function establishes a stack frame, carrying the two
    /// lengths its `.pdata` record and its two `$M` labels need. `None` for a
    /// leaf — c2 emits no unwind record for one, so this field alone decides
    /// whether the obj has a `.pdata` section at all.
    pub frame: Option<Frame>,
    /// **Compiler-label counter slots this function takes BEFORE its own `$M`
    /// triple** — 0 for every class but WCR's signed two-call comparator, which
    /// takes 2.
    ///
    /// A *leading* count and not merely a bigger stride: it moves this
    /// function's own `$M`/`$M`/`$T` numbers up as well as every later
    /// function's, which is the placement `docs/CODEGEN_FRAMED_CALLS.md` §4.4
    /// records for the `__savegprlr_N`/`__restgprlr_N` pair and
    /// `docs/LABEL_COUNTER.md` §1.1 tabulates as a surcharge. Producer:
    /// [`c2_il::IlFunction::label_lead`]; the total stride it feeds is
    /// `c2_il::IlFunction::label_slots`, and the two are separate because
    /// [`plan_labels`] needs to add them at different points. Moving the same
    /// two slots to *after* the triple is 119 mismatches in
    /// `scripts/sweep.d/98-cmp-order.py`, i.e. the placement is graded and not
    /// merely the total.
    pub label_lead: u32,
}

impl<'a> Function<'a> {
    /// A function with no call, no constant pool and no frame — the common case.
    pub fn plain(name: &'a str, text_offset: u32) -> Function<'a> {
        Function {
            name,
            text_offset,
            calls: Vec::new(),
            is_float: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            frame: None,
            label_lead: 0,
        }
    }

    /// The callees this function introduces to the symbol table, in the order
    /// their symbols are **emitted**: distinct names in **reverse first-reference
    /// order**.
    ///
    /// Measured (`docs/OBJ_GY_SHAPES.md` §3.3 as extended, byte evidence in
    /// `docs/CODEGEN_FRAMED_CALLS.md` §4.1). `f(){ g1(); g2(); g3(); }` puts `?g3`
    /// at index 15, `?g2` at 16 and `?g1` at 17 — and the mirrored source
    /// `g3(); g2(); g1();` puts `?g1` at 15, which is what refutes both
    /// "alphabetical" and "declaration order". `g1(); g2(); g1();` emits two
    /// symbols, not three, and its repeat relocates against the first.
    ///
    /// This is the same LIFO the `.rdata` constant pool uses within one function
    /// (§2.3) and it has the same failure mode: a naive append emits every index
    /// swapped and **every relocation still resolves**, so the obj is wrong in a
    /// way no linker complains about.
    fn introduced_callees(&self) -> Vec<&'a str> {
        let mut first_ref: Vec<&'a str> = Vec::with_capacity(self.calls.len());
        for c in &self.calls {
            if !first_ref.contains(&c.callee) {
                first_ref.push(c.callee);
            }
        }
        first_ref.reverse();
        first_ref
    }
}

/// `.rdata` COMDAT characteristics for a pooled FP constant:
/// CNT_INITIALIZED_DATA | LNK_COMDAT | ALIGN_4/8 | MEM_READ. The alignment field
/// is the only difference between the `float` and `double` pools.
const CH_RDATA_F32: u32 = 0x4030_1040;
const CH_RDATA_F64: u32 = 0x4040_1040;

/// IMAGE_REL_PPC_REFHI / REFLO / PAIR. c2 loads a pooled FP constant through an
/// `addis`+`lfs` pair, and each half needs a PAIR record carrying the other
/// half's displacement in its `SymbolTableIndex` field. Every pooled constant
/// gets its own COMDAT section, so that displacement is always 0.
const REL_PPC_REFHI: u16 = 0x0010;
const REL_PPC_REFLO: u16 = 0x0011;
const REL_PPC_PAIR: u16 = 0x0012;

/// The mangled symbol name c2 gives a pooled FP constant: `__real@` followed by
/// the big-endian IEEE bit pattern in lowercase hex — 8 digits for a `float`,
/// 16 for a `double`.
fn real_symbol_name(bits: u64, double: bool) -> String {
    if double {
        format!("__real@{bits:016x}")
    } else {
        let v = f64::from_bits(bits) as f32;
        format!("__real@{:08x}", v.to_bits())
    }
}

/// The pooled constant's raw `.rdata` bytes: big-endian IEEE-754, narrowed to
/// binary32 for a `float`. The narrowing is exactness-checked in codegen before
/// the reference is ever recorded.
fn real_raw_bytes(bits: u64, double: bool) -> Vec<u8> {
    if double {
        bits.to_be_bytes().to_vec()
    } else {
        (f64::from_bits(bits) as f32).to_be_bytes().to_vec()
    }
}

/// The CRT float-support marker symbol.
const NAME_FLTUSED: &str = "_fltused";

/// IMAGE_REL_PPC_REL24 — 24-bit relative branch relocation (tail/`bl` calls).
const REL_PPC_REL24: u16 = 0x0006;
/// IMAGE_REL_PPC_ADDR32 — 32-bit VA relocation (the `.pdata` BeginAddress).
const REL_PPC_ADDR32: u16 = 0x0002;

/// `.pdata` section characteristics: CNT_INIT_DATA | ALIGN_8 | MEM_READ.
const CH_PDATA: u32 = 0x4040_0040;

/// Reflected CRC-32, polynomial `0xEDB88320`, **no final inversion**, over a
/// byte run — parameterised on the initial value, because c2 uses this same
/// loop twice with two different ones and getting them the wrong way round is
/// the documented way to implement this wrong (`docs/OBJ_DYNINIT_SHAPE.md`
/// §2.3, closing note):
///
/// | consumer | init | via |
/// |---|---|---|
/// | COFF aux section-def `CheckSum` | `0` | [`coff_checksum`] |
/// | the `??_C@…` string-literal name hash (JamCRC) | `0xFFFFFFFF` | [`jamcrc`] |
///
/// One loop with an argument, not two loops — two independent copies is exactly
/// how the swap happens, and it is invisible to every consistency check the port
/// has (both values are 32 bits and both look like noise).
fn crc32_reflected(init: u32, data: &[u8]) -> u32 {
    let mut c = init;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { (c >> 1) ^ 0xEDB8_8320 } else { c >> 1 };
        }
    }
    c
}

/// The COFF aux section-def CheckSum algorithm — [`crc32_reflected`] with init
/// `0`. Used for `.pdata` (whose aux carries a real checksum even though it is
/// not a COMDAT) and for a string-literal `.rdata`; the fixed `.XBLD$W` COMDAT
/// checksums stay hardcoded above.
///
/// **Scope, corrected out-of-sample** (`docs/OBJ_DYNINIT_SHAPE.md` §2.3, held-out
/// prediction H9 refuted): the field is `0` for `.text$y?`, for `.text`, for
/// `.bss`, for `.CRT$XCU`, for `.drectve`/`.debug$S`, and — the refutation — for
/// an **FP-constant** `.rdata` COMDAT. It carries the real CRC for the two
/// `.XBLD$W`, for `.pdata`, and for a **string** `.rdata`, COMDAT or not.
fn coff_checksum(data: &[u8]) -> u32 {
    crc32_reflected(0, data)
}

/// JamCRC — [`crc32_reflected`] with init `0xFFFFFFFF`, no final XOR
/// (equivalently `!crc32(data)`). The hash inside a `??_C@…` string-literal
/// COMDAT name, over the literal's bytes **including the NUL**.
fn jamcrc(data: &[u8]) -> u32 {
    crc32_reflected(0xFFFF_FFFF, data)
}

/// The unwind facts one framed function contributes: the two lengths that go
/// into its `.pdata` record and, as it happens, the values of its two `$M`
/// labels. Both in **bytes**; both must be word multiples.
///
/// A **leaf** contributes nothing — c2 emits no `.pdata` record for a function
/// that establishes no frame, and "establishes a frame" is exactly what the
/// emitter knows (it wrote the prologue). Measured: a leaf with a 400-byte local
/// array addresses it below `r1` in the red zone (`addi r10,r1,-400`) and gets no
/// record; make the array 70,000 bytes so the prologue has to move `r1` and the
/// record appears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Prologue length in bytes — the offset one past the last prologue
    /// instruction, i.e. the value of the `$M(n)` label.
    pub prolog_len: u32,
    /// Function length in bytes, **excluding** any inter-function padding —
    /// the value of the `$M(n+1)` label.
    pub func_len: u32,
}

/// `IMAGE_COMDAT_SELECT_ASSOCIATIVE` — the selection a per-function `.pdata`
/// COMDAT carries under `/Gy`, tying it to its `.text` COMDAT.
const COMDAT_SELECT_ASSOCIATIVE: u8 = 5;

/// `.pdata` COMDAT characteristics under `/Gy`: [`CH_PDATA`] plus
/// `IMAGE_SCN_LNK_COMDAT` (0x1000).
const CH_PDATA_COMDAT: u32 = 0x4040_1040;

/// Build the 8-byte X360 `RUNTIME_FUNCTION` for one framed function:
/// `BeginAddress` (patched by an ADDR32 relocation against the function's own
/// symbol, so the raw value is the addend — 0 for every record the port emits)
/// followed by the packed unwind word, both **big-endian** (like `.text`,
/// unlike every COFF header field).
///
/// The unwind word is a bitfield, established from c2's own output rather than
/// from any x64 `.pdata` documentation — the Xbox 360 form has no `.xdata` and
/// no unwind-code array at all, the whole record is these 8 bytes:
///
/// ```text
///   bits  7..0   PrologLen   prologue length in INSTRUCTIONS
///   bits 29..8   FuncLen     function length in INSTRUCTIONS
///   bit  30      ThirtyTwoBit  1 in every record c2 emitted across the probes
///   bit  31      ExceptionFlag 1 iff the function has EH data
/// ```
///
/// Witnesses, each read straight out of a reference obj (source in
/// `docs/OBJ_FORMAT_MVP.md` §7):
///
/// ```text
///   0x40000903  9 words / prolog 3   return g(a)+1        .text 0x24, $M @ 0x0c
///   0x40001205 18 words / prolog 5   two calls, r30/r31   .text 0x48, $M @ 0x14
///   0x40001607 22 words / prolog 7   100 KB local + calls .text 0x58, $M @ 0x1c
///   0x40002203 34 words / prolog 3   6 args via __savegprlr_25
///   0x40000f06 15 words / prolog 6   leaf with a 70 KB frame (still framed)
///   0xc0001306 19 words / prolog 6   a body with a destructor, /EHsc
/// ```
///
/// so `FuncLen` and `PrologLen` are the only fields that move, they are exactly
/// the two `$M` label values divided by four, and bit 31 is the one thing that
/// takes the record outside the class this port emits (EH also splits a function
/// into **several** records — a `try`/`catch` body produced two, the catch
/// funclet's first, with a non-zero `BeginAddress` addend).
pub fn pdata_record(begin_addend: u32, frame: &Frame) -> [u8; 8] {
    debug_assert_eq!(frame.func_len % 4, 0, "function length is a word multiple");
    debug_assert_eq!(frame.prolog_len % 4, 0, "prologue length is a word multiple");
    let unwind = UNWIND_THIRTY_TWO_BIT | ((frame.func_len / 4) << 8) | (frame.prolog_len / 4);
    let mut r = [0u8; 8];
    r[..4].copy_from_slice(&begin_addend.to_be_bytes());
    r[4..].copy_from_slice(&unwind.to_be_bytes());
    r
}

/// Bit 30 of the unwind word — set in every record c2 emitted across every
/// probe. Named rather than folded into a magic constant because bit 31 beside
/// it is the EH flag, and the port refuses that case.
const UNWIND_THIRTY_TWO_BIT: u32 = 0x4000_0000;

/// The `.pdata` raw section for a run of framed functions, records concatenated
/// in `.text` order. Under `/Gy` this is called once per function (one record);
/// packed, once for the whole TU.
fn build_pdata(frames: &[&Frame]) -> Vec<u8> {
    let mut b = Vec::with_capacity(frames.len() * 8);
    for f in frames {
        b.extend_from_slice(&pdata_record(0, f));
    }
    b
}

/// How far past the `.gl` label counter ([`c2_il::label_counter`]) the first
/// compiler label of a TU sits.
pub const LABEL_SEED_GAP: u32 = 9;

/// The `$M`/`$T` label numbers c2 gives each function, or `None` for a function
/// that is not framed (it consumes counter slots but emits no label).
///
/// The allocator, measured against real objs over 25 TUs — see
/// `docs/OBJ_GY_SHAPES.md` §3.4/§3.5:
///
/// * the first label of a TU is `.gl` counter + [`LABEL_SEED_GAP`];
/// * under `/Gy` a flat surcharge of **3 per function in the TU** is paid
///   up front, before any function's own labels — even for functions that emit
///   no label at all;
/// * then, in `.text` order, each function consumes **1** if it is a leaf and
///   **4** (packed) / **5** (`/Gy`) if it is framed, of which the framed
///   function emits the first three as `$M(n)` (prologue end), `$M(n+1)`
///   (function end) and `$T(n+2)` (its `.pdata` record).
///
/// The "1 per leaf" holds for every function class this port emits and **not**
/// for every function class: a signed-relational comparison leaf (`a < b`)
/// consumes 3, and each **newly pooled** FP constant a further 2. Those are
/// refused upstream ([`crate::PortC2::build`]) rather than modeled, because a
/// wrong stride is a wrong `$M` number and a wrong `$M` number is a wrong-bytes
/// obj — the whole point of the counter.
///
/// **A constant-free floating-point leaf is 1, not 2**, and this comment used to
/// say 2. The 2 is a whole-TU reading of a leaf that is itself the TU's first FP
/// function — `_fltused`'s slot, which the `+1` below already charges once per
/// TU. `docs/LABEL_COUNTER.md` §1: `leaf-float` = 2, `leaf-float-led` = 1,
/// `leaf-double-led` = 1. Charging it twice was what kept every (FP leaf, framed
/// function) pair out of class.
pub fn plan_labels(counter: u32, funcs: &[Function], comdat: bool) -> Vec<Option<[u32; 3]>> {
    let mut cur = counter + LABEL_SEED_GAP;
    if comdat {
        // Measured exactly, on 11 TUs of 2 to 5 functions: the `/Gy` pre-pass is
        // three slots per function, whatever kind, and it is **not** affected by
        // floating point. Every row below is `packed + 3 * funcs.len()`.
        cur += 3 * funcs.len() as u32;
    }
    // **One extra slot for the TU's first FP-touching function** — the `_fltused`
    // external's slot, and the same field decides where that symbol goes
    // (`Function::is_float`), so the two are one fact and cannot drift.
    //
    // This corrects a rule that was wrong from two FP functions on. It read
    // "anything that touches floating point consumes 2", which fits one FP
    // function and predicts 4 slots for two where c2 gives 3, and 6 for three
    // where c2 gives 4. Measured seed-free as the *difference* between two framed
    // functions' labels in one TU, so nothing depends on the `.gl` seed; the
    // table is on `c2_il::IlFunction::label_slots`.
    //
    // This `+1` was once explained as "one slot per TU-level external", the same
    // rule as `docs/CODEGEN_FRAMED_CALLS.md` §4.4's `__savegprlr_N`/
    // `__restgprlr_N` pair consuming two slots for its two externals.
    // **The explanation is refuted** (`docs/LABEL_COUNTER.md` §2.1): a pooled FP
    // constant costs +2 and mints no external, a string literal costs 0 and
    // mints one. The `+1` and the `+2` are both still exact — see §1.1 for the
    // surcharge table that actually fits — but no new class may be added here on
    // the strength of counting its externals.
    let mut fltused_slot_taken = !funcs.iter().any(|f| f.is_float);
    funcs
        .iter()
        .map(|f| {
            if f.is_float && !fltused_slot_taken {
                fltused_slot_taken = true;
                cur += 1;
            }
            // **The leading surcharge is taken before the function's own triple**,
            // so it moves this function's `$M` numbers as well as every later
            // one's. Measured seed-free and in-TU (`scripts/gt_cmp_rr.py
            // --stride`, with the in-TU anchor control holding on every row):
            // a signed `>`/`<` two-call comparator is stride 7 / lead 2 under
            // `/Gy` and 6 / 2 packed, against 5 / 0 and 4 / 0 for its `==`,
            // unsigned and arithmetic-tailed siblings. Same shape as the
            // `__savegprlr_N` pair's, from `docs/LABEL_COUNTER.md` §1.1's
            // surcharge table and not from counting anything's externals — the
            // rule that once explained the `+1` above is refuted.
            cur += f.label_lead;
            match f.frame {
                Some(_) => {
                    let n = cur;
                    cur += if comdat { 5 } else { 4 };
                    Some([n, n + 1, n + 2])
                }
                None => {
                    cur += 1;
                    None
                }
            }
        })
        .collect()
}

/// Render a compiler label name (`$M2545`, `$T2547`). Kept as one function so
/// the 8-byte short-name limit is checked in one place: the numbers observed run
/// to four digits, and a five-digit counter would still fit (`$M12345`).
fn label_name(prefix: char, n: u32) -> String {
    format!("${prefix}{n}")
}

// `emit_framed_obj` used to live here: a second whole-obj emitter for the one
// single-function framed TU, with a hardcoded 20-symbol table and the label
// names `$M2545/$M2546/$T2547` written out literally. It is gone. A framed
// function is now a `Function` with a `frame`, and the same two emitters
// (`emit_obj` packed, `emit_comdat_obj` under `/Gy`) build every obj — because
// this file already carries two bugs whose whole cause was one rule
// implemented in two emitters and fixed in one.

/// Emit the `$T…` label that sits on a `.pdata` record. Same shape as
/// [`emit_label_symbol`] but storage class **3 (STATIC)**, not 6 (LABEL) — a
/// one-byte difference between two symbols emitted four slots apart, and the
/// reason this is its own function rather than a boolean argument.
fn emit_pdata_label_symbol(b: &mut Buf, name: &str, value: u32, sec_num: i16) {
    b.name8(name);
    b.u32(value);
    b.i16(sec_num);
    b.u16(0x0000); // Type
    b.u8(3); // IMAGE_SYM_CLASS_STATIC
    b.u8(0); // no aux
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

#[cfg(test)]
mod comdat_tests {
    use super::*;

    /// An undefined external callee is emitted once per distinct *name*, not once
    /// per call site, and every later site relocates against that first index.
    ///
    /// Invisible until a TU has two functions calling the same callee, which no
    /// fixture did before `il_call_perm.cpp` — there the five functions after
    /// `pass3` all call `g3` and the reference has exactly one `?g3@@YAHHHH@Z`.
    /// Emitting per call site inflates `NumberOfSymbols` and shifts every symbol
    /// index after the duplicate, so it is a whole-obj mismatch, not a local one.
    #[test]
    fn callee_symbols_are_emitted_once_per_distinct_name() {
        let text = vec![0u8; 12];
        let mk = |name: &'static str, off: u32, callee: &'static str| Function {
            name,
            text_offset: off,
            calls: vec![Call { reloc_offset: off, callee }],
            is_float: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            frame: None,
            label_lead: 0,
        };
        // Three functions, two of them calling the same callee.
        let funcs = [mk("?a@@YAHXZ", 0, "?g@@YAHXZ"), mk("?b@@YAHXZ", 4, "?h@@YAHXZ"), mk("?c@@YAHXZ", 8, "?g@@YAHXZ")];
        let obj = emit_obj("Z:\\t.obj", &funcs, &text, 0);
        let n_symbols = u32::from_le_bytes(obj[12..16].try_into().unwrap());
        // 13 fixed + 3 defined + 2 distinct callees, NOT 3.
        assert_eq!(n_symbols, 18, "expected one symbol per distinct callee");

        // All three relocations are present, and the first and third share a
        // symbol index while the second differs.
        let n_reloc = u16::from_le_bytes(
            obj[COFF_HEADER_LEN + 4 * SECTION_HEADER_LEN + 32..][..2].try_into().unwrap(),
        );
        assert_eq!(n_reloc, 3);
        let prel = u32::from_le_bytes(
            obj[COFF_HEADER_LEN + 4 * SECTION_HEADER_LEN + 24..][..4].try_into().unwrap(),
        ) as usize;
        let sym_of = |i: usize| {
            u32::from_le_bytes(obj[prel + i * RELOC_LEN + 4..][..4].try_into().unwrap())
        };
        assert_eq!(sym_of(0), sym_of(2), "both `?g` call sites relocate to one symbol");
        assert_ne!(sym_of(0), sym_of(1));
    }

    /// The COMDAT emitter's two layout bugs, both found only when
    /// `scripts/mode_lane.sh` first compiled the call fixtures with `/Gy`:
    /// a callee symbol per *call site* rather than per distinct name, and all
    /// relocations batched after all raw data rather than each following its own
    /// section's.
    #[test]
    fn comdat_dedups_callees_and_places_relocs_with_their_section() {
        let blr = crate::codegen::encode_blr().to_vec();
        let mk = |name: &'static str, callee: &'static str| Function {
            name,
            text_offset: 0,
            calls: vec![Call { reloc_offset: 0, callee }],
            is_float: false,
            fp_refs: Vec::new(),
            data_refs: Vec::new(),
            frame: None,
            label_lead: 0,
        };
        // Three functions, two calling the same callee — the shape `il_call_perm.cpp`
        // has six of, where the port came out five symbols long.
        let funcs = [
            mk("?a@@YAHXZ", "?g@@YAHXZ"),
            mk("?b@@YAHXZ", "?h@@YAHXZ"),
            mk("?c@@YAHXZ", "?g@@YAHXZ"),
        ];
        let texts = vec![blr.clone(), blr.clone(), blr];
        let obj = emit_comdat_obj("Z:\\t.obj", &funcs, &texts, 0);

        // 11 fixed + per function (section symbol + aux + defined symbol) = 9,
        // + 2 distinct callees, NOT 3.
        let n_symbols = u32::from_le_bytes(obj[12..16].try_into().unwrap());
        assert_eq!(n_symbols, 22, "expected one symbol per distinct callee");

        // Each `.text` section's relocation sits immediately after its own raw
            // 4 fixed sections precede the per-function `.text` run.
        // data, so `PointerToRelocations` == `PointerToRawData` + raw length.
        for i in 0..funcs.len() {
            let h = COFF_HEADER_LEN + (4 + i) * SECTION_HEADER_LEN;
            let size = u32::from_le_bytes(obj[h + 16..][..4].try_into().unwrap()) as usize;
            let raw = u32::from_le_bytes(obj[h + 20..][..4].try_into().unwrap()) as usize;
            let prel = u32::from_le_bytes(obj[h + 24..][..4].try_into().unwrap()) as usize;
            assert_eq!(
                prel,
                raw + size,
                "section {i}: relocations must follow their own raw data"
            );
        }
        // And the two `?g` sites share one symbol index while `?h` differs.
        let sym_at = |i: usize| {
            let h = COFF_HEADER_LEN + (4 + i) * SECTION_HEADER_LEN;
            let prel = u32::from_le_bytes(obj[h + 24..][..4].try_into().unwrap()) as usize;
            u32::from_le_bytes(obj[prel + 4..][..4].try_into().unwrap())
        };
        assert_eq!(sym_at(0), sym_at(2), "both `?g` call sites relocate to one symbol");
        assert_ne!(sym_at(0), sym_at(1));
    }

    /// The COMDAT shape, pinned against `system/utl/Spew.cpp` compiled with the
    /// dc3 workload's real flags (two empty functions, so two 4-byte `.text`
    /// sections each holding a single `blr`).
    #[test]
    fn comdat_obj_has_one_text_section_per_function() {
        let blr = crate::codegen::encode_blr().to_vec();
        let funcs = [
            Function::plain("?SpewInit@@YAXXZ", 0),
            Function::plain("?SpewTerminate@@YAXXZ", 0),
        ];
        let obj = emit_comdat_obj("Z:\\x.obj", &funcs, &[blr.clone(), blr], 0);

        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| {
            u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]])
        };
        // 4 fixed sections + one per function; 11 fixed symbols + 3 per function.
        assert_eq!(u16at(2), 6, "section count");
        assert_eq!(u32at(12), 17, "symbol count");

        // Both `.text` sections are 4 bytes and carry the COMDAT bit.
        for i in 4..6 {
            let o = COFF_HEADER_LEN + i * SECTION_HEADER_LEN;
            assert_eq!(&obj[o..o + 5], b".text");
            assert_eq!(u32at(o + 16), 4, "section {i} size");
            assert_eq!(u32at(o + 36), CH_TEXT_COMDAT, "section {i} characteristics");
        }
        // Contiguous raw data — no inter-function padding, unlike the packed
        // layout's 8-byte function alignment.
        let raw0 = u32at(COFF_HEADER_LEN + 4 * SECTION_HEADER_LEN + 20);
        let raw1 = u32at(COFF_HEADER_LEN + 5 * SECTION_HEADER_LEN + 20);
        assert_eq!(raw1, raw0 + 4, "second .text follows the first with no padding");

        // Each function symbol sits at Value 0 in its OWN section, and each
        // section symbol's aux selects NODUPLICATES.
        let symtab = u32at(8) as usize;
        for (k, sec_num) in [(0usize, 5i16), (1, 6)] {
            let secsym = symtab + (11 + k * 3) * 18;
            assert_eq!(obj[secsym + 17], 1, "section symbol has one aux");
            let aux = secsym + 18;
            assert_eq!(obj[aux + 14], COMDAT_SELECT_NODUPLICATES, "aux selection");
            let fnsym = secsym + 36;
            assert_eq!(u32at(fnsym + 8), 0, "function Value is 0 in its own section");
            assert_eq!(
                i16::from_le_bytes([obj[fnsym + 12], obj[fnsym + 13]]),
                sec_num
            );
        }
    }
}

/// `.text` COMDAT selection: `IMAGE_COMDAT_SELECT_NODUPLICATES`.
const COMDAT_SELECT_NODUPLICATES: u8 = 1;

/// `.text` characteristics under **function-level linking** (`/Gy`): the packed
/// [`CH_TEXT`] plus `IMAGE_SCN_LNK_COMDAT` (0x1000).
const CH_TEXT_COMDAT: u32 = 0x6040_1020;

/// Build the complete `.obj` image with **one COMDAT `.text` section per
/// function** — the shape c2 emits under function-level linking (`/Gy`, which
/// `/O1` and `/O2` imply).
///
/// This is not a variant spelling of [`emit_obj`]; it is a different obj:
///
/// | | packed (`/Ox`) | COMDAT (`/Gy`) |
/// |---|---|---|
/// | `.text` sections | 1 | one per function |
/// | characteristics | `0x60400020` | `0x60401020` |
/// | aux `Selection` | 0 | 1 (NODUPLICATES) |
/// | function `Value` | its offset in `.text` | always 0 |
/// | inter-function padding | 8-byte aligned | none — each has its own section |
/// | symbol count | 13 + 1/fn (+callees) | 11 + 3/fn (+callees) |
///
/// So the same IL yields two legitimately different objs depending on an argv
/// flag the bundle does not record. Verified against `system/utl/Spew.cpp`
/// compiled with the dc3 workload's real flags: 6 sections, 17 symbols, two
/// 4-byte `.text` sections each holding a single `blr`, laid out contiguously
/// with no padding between them.
///
/// `texts[i]` is function `i`'s own `.text` bytes; each function's
/// `text_offset` is ignored (it is 0 within its own section) and any
/// `call.reloc_offset` is relative to that function's section.
///
/// A **framed** function additionally gets its own `.pdata` COMDAT, emitted
/// immediately after its `.text` COMDAT and tied to it by
/// `IMAGE_COMDAT_SELECT_ASSOCIATIVE` with the aux `Number` field naming that
/// `.text`'s section number — so the linker drops a function's unwind record
/// with the function. `label_counter` is the `.gl` seed
/// ([`c2_il::label_counter`]); it is unused when no function is framed, and a
/// caller with a framed function and no counter must refuse rather than guess.
pub fn emit_comdat_obj(
    obj_name: &str,
    funcs: &[Function],
    texts: &[Vec<u8>],
    label_counter: u32,
) -> Vec<u8> {
    assert_eq!(funcs.len(), texts.len(), "one text per function");
    let labels = plan_labels(label_counter, funcs, true);
    // Per-function `.pdata` raw, built up front so the sections can borrow it.
    let pdata_raw: Vec<Option<[u8; 8]>> =
        funcs.iter().map(|f| f.frame.as_ref().map(|fr| pdata_record(0, fr))).collect();

    let mut sections: Vec<Section> = shell_sections(obj_name);
    // Per function: its `.text` COMDAT, then — if it is framed — its `.pdata`
    // COMDAT immediately after, tied back with SELECT_ASSOCIATIVE. `sec_text[i]`
    // / `sec_pdata[i]` are 0-based indices into `sections`.
    let mut sec_text: Vec<usize> = Vec::with_capacity(funcs.len());
    let mut sec_pdata: Vec<Option<usize>> = Vec::with_capacity(funcs.len());
    // The inverse map, so the layout and relocation passes below index rather
    // than search: section -> the function it belongs to, and which of its two
    // sections it is. `SectionOwner::None` for the fixed prefix.
    let mut owner: Vec<SectionOwner> = vec![SectionOwner::Fixed; sections.len()];
    for (i, t) in texts.iter().enumerate() {
        sec_text.push(sections.len());
        owner.push(SectionOwner::Text(i));
        sections.push(Section {
            name: ".text",
            characteristics: CH_TEXT_COMDAT,
            raw: std::borrow::Cow::Borrowed(t.as_slice()),
            checksum: 0,
            selection: COMDAT_SELECT_NODUPLICATES,
            assoc: 0,
            uninit_size: None,
        });
        match &pdata_raw[i] {
            None => sec_pdata.push(None),
            Some(rec) => {
                let text_sec_num = (sec_text[i] + 1) as u16;
                sec_pdata.push(Some(sections.len()));
                owner.push(SectionOwner::Pdata(i));
                sections.push(Section {
                    name: ".pdata",
                    characteristics: CH_PDATA_COMDAT,
                    raw: std::borrow::Cow::Borrowed(&rec[..]),
                    // `.pdata` is the one COMDAT c2 gives a real CheckSum —
                    // `.text` and the `.rdata` constant pools carry 0.
                    checksum: coff_checksum(&rec[..]),
                    selection: COMDAT_SELECT_ASSOCIATIVE,
                    assoc: text_sec_num,
                    uninit_size: None,
                });
            }
        }
    }
    let n_sections = sections.len();

    // Raw data is packed contiguously after the section headers — including
    // between the per-function `.text` sections, which carry no padding —
    // **except** that a section's relocations immediately follow *its own* raw
    // data, before the next section's:
    //
    //   .text[0] raw @696 ; .text[0] reloc @700
    //   .text[1] raw @710 ; .text[1] reloc @714 ; …
    //
    // Not all raw data followed by all relocations. This emitter did the latter,
    // which is only invisible when at most one section has relocations — and under
    // `/Gy` every calling function's COMDAT `.text` has one, so the port's whole
    // section table carried wrong `PointerToRelocations` values from the fifth
    // header on (`il_call_value.cpp`, divergence at obj offset 204).
    //
    // Precisely the bug already fixed in [`emit_obj`] for the packed layout, where
    // `.text` being last hid it. Two emitters, one wrong assumption, and the second
    // one stayed wrong because no lane compiled a multi-call fixture with `/Gy`
    // until `scripts/mode_lane.sh`.
    //
    // A framed function's `.pdata` has exactly one relocation of its own (the
    // ADDR32 on `BeginAddress`), so it follows the same rule.
    let n_reloc_of: Vec<u16> = owner
        .iter()
        .map(|o| match o {
            // WR1: each data-symbol reference adds a REFHI/PAIR/REFLO/PAIR quad.
            SectionOwner::Text(k) => {
                (funcs[*k].calls.len() + 4 * funcs[*k].data_refs.len()) as u16
            }
            SectionOwner::Pdata(_) => 1,
            SectionOwner::Fixed => 0,
        })
        .collect();
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);

    // Symbols: the fixed 11-slot prefix, then per function a `.text` section
    // symbol (+aux), the defined FUNCTION symbol, and its callee if any.
    // An undefined external callee is emitted **once per distinct name**, after the
    // symbol of the function that first calls it; every later call site relocates
    // against that same index. This path emitted one per *call site* instead, which
    // is invisible until a TU has two functions calling the same callee under `/Gy`
    // — `il_call_perm.cpp` has six calling `?g3`, and the port's symbol table came
    // out five symbols long (obj offset 12, `NumberOfSymbols`). The packed emitter
    // had already been fixed for exactly this; `emit_comdat_obj` had not, and no lane
    // compiled the call fixtures with `/Gy` until `scripts/mode_lane.sh`.
    //
    // A framed function's group is longer, and the order inside it is the
    // reference's, not an obvious one — the END label comes before the callee and
    // the PROLOGUE label after it:
    //
    //   [.text sym + aux] [fn] [$M(n+1) @ function end] [callee, if new]
    //   [$M(n) @ prologue end] [.pdata sym + aux] [$T(n+2) @ 0]
    //
    // `_fltused` goes immediately after the **first** float function's complete
    // group — its section symbol + aux, its function symbol, and any callee external
    // it introduced — and before the next function's section symbol. That is the
    // same rule as the packed layout; `/Gy` does not move it (`docs/OBJ_GY_SHAPES.md`
    // §1, six orderings captured: float-first, int-first, float-int-float,
    // int-int-float, and a float function whose callee external precedes the marker).
    // Omitting it entirely is what left `mvp_fmul3.cpp` one symbol short of the
    // reference under `/Gy`.
    let fltused_after = funcs.iter().position(|f| f.is_float);
    let mut next_idx: u32 = N_SHELL_SYMBOLS;
    // The callee symbols this function emits, in emission order (reverse
    // first-reference), each with the index it lands at.
    let mut introduced: Vec<Vec<(&str, u32)>> = Vec::with_capacity(funcs.len());
    let mut fn_idx: Vec<u32> = Vec::with_capacity(funcs.len());
    let mut callee_syms: Vec<(&str, u32)> = Vec::new();
    let mut data_syms: Vec<(&str, u32)> = Vec::new();
    let mut introduced_data: Vec<Vec<(&str, u32)>> = Vec::with_capacity(funcs.len());
    for (i, f) in funcs.iter().enumerate() {
        next_idx += 2; // section symbol + aux
        fn_idx.push(next_idx);
        next_idx += 1; // the function symbol
        if labels[i].is_some() {
            next_idx += 1; // $M(n+1), the function-end label
        }
        // One undefined external per **distinct** callee this function is the
        // first to name, in reverse first-reference order.
        let mut here: Vec<(&str, u32)> = Vec::new();
        for name in f.introduced_callees() {
            if callee_syms.iter().any(|(n, _)| *n == name) {
                continue;
            }
            callee_syms.push((name, next_idx));
            here.push((name, next_idx));
            next_idx += 1;
        }
        introduced.push(here);
        // WR1: this function's new data symbols, after its callees, exactly as
        // the packed layout places them.
        let mut here_data: Vec<(&str, u32)> = Vec::new();
        for r in &f.data_refs {
            if data_syms.iter().any(|(n, _)| *n == r.name) {
                continue;
            }
            data_syms.push((r.name, next_idx));
            here_data.push((r.name, next_idx));
            next_idx += 1;
        }
        introduced_data.push(here_data);
        if labels[i].is_some() {
            next_idx += 1; // $M(n), the prologue-end label
            next_idx += 2; // .pdata section symbol + aux
            next_idx += 1; // $T(n+2)
        }
        if fltused_after == Some(i) {
            next_idx += 1;
        }
    }
    let n_symbols = next_idx;

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);

    // Interleaved to match the layout computed above: each section's raw data,
    // then its own relocations.
    for (i, s) in sections.iter().enumerate() {
        debug_assert_eq!(b.0.len(), ptrs[i]);
        b.bytes(&s.raw);
        match owner[i] {
            SectionOwner::Text(k) => {
                debug_assert!(
                    n_reloc_of[i] == 0 || b.0.len() == reloc_ptr[i].unwrap()
                );
                // One REL24 per call site (several sites may share one symbol
                // index — the same callee called twice) and, WR1, one
                // REFHI/PAIR/REFLO/PAIR quad per data-symbol address. Emitted
                // **ascending by VirtualAddress**, which is what the records in a
                // section are ordered by: the `lis` is at offset 0 and the tail
                // branch is last. The sort is stable, so each quad keeps its
                // REFHI-before-PAIR order at equal VA.
                let mut recs: Vec<(u32, u32, u16)> = Vec::new();
                for call in &funcs[k].calls {
                    let ci = callee_syms
                        .iter()
                        .find(|(n, _)| *n == call.callee)
                        .map(|(_, ix)| *ix)
                        .expect("every callee got a symbol");
                    recs.push((call.reloc_offset, ci, REL_PPC_REL24));
                }
                for r in &funcs[k].data_refs {
                    let di = data_syms
                        .iter()
                        .find(|(n, _)| *n == r.name)
                        .map(|(_, ix)| *ix)
                        .expect("every data symbol got a slot");
                    recs.push((r.hi_off, di, REL_PPC_REFHI));
                    recs.push((r.hi_off, 0, REL_PPC_PAIR));
                    recs.push((r.lo_off, di, REL_PPC_REFLO));
                    recs.push((r.lo_off, 0, REL_PPC_PAIR));
                }
                recs.sort_by_key(|&(va, _, _)| va);
                debug_assert_eq!(recs.len(), n_reloc_of[i] as usize);
                for (va, sym, ty) in recs {
                    b.u32(va);
                    b.u32(sym);
                    b.u16(ty);
                }
            }
            SectionOwner::Pdata(k) => {
                // `BeginAddress` at `.pdata` offset 0, ADDR32 against the framed
                // function's own symbol (the record's raw addend is 0).
                debug_assert_eq!(b.0.len(), reloc_ptr[i].unwrap());
                b.u32(0);
                b.u32(fn_idx[k]);
                b.u16(REL_PPC_ADDR32);
            }
            SectionOwner::Fixed => {}
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections);

    for (i, f) in funcs.iter().enumerate() {
        let sec_num = (sec_text[i] + 1) as i16;
        emit_section_symbol(
            &mut b,
            &sections[sec_text[i]],
            sec_num,
            (f.calls.len() + 4 * f.data_refs.len()) as u16,
        );
        // The function is at offset 0 of its own section.
        emit_function_symbol(&mut b, &mut strtab, f.name, sec_num, 0);
        if let (Some(m), Some(frame)) = (labels[i], f.frame.as_ref()) {
            emit_label_symbol(&mut b, &label_name('M', m[1]), frame.func_len, sec_num);
        }
        // Only the function that *introduces* a callee emits its symbol, in
        // reverse first-reference order.
        for (name, _) in &introduced[i] {
            emit_function_symbol(&mut b, &mut strtab, name, 0, 0);
        }
        // WR1: undefined external DATA symbols (`Type` 0x0000), after the callees.
        for (name, _) in &introduced_data[i] {
            emit_external_symbol(&mut b, &mut strtab, name, 0, 0x0000);
        }
        if let (Some(m), Some(frame), Some(ps)) = (labels[i], f.frame.as_ref(), sec_pdata[i]) {
            emit_label_symbol(&mut b, &label_name('M', m[0]), frame.prolog_len, sec_num);
            emit_section_symbol(&mut b, &sections[ps], (ps + 1) as i16, 1);
            emit_pdata_label_symbol(&mut b, &label_name('T', m[2]), 0, (ps + 1) as i16);
        }
        // The CRT float-support marker, once, after the first FP function's group.
        if fltused_after == Some(i) {
            emit_function_symbol(&mut b, &mut strtab, NAME_FLTUSED, 0, 0);
        }
    }

    b.bytes(&strtab.finish());
    b.0
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
pub fn emit_obj(obj_name: &str, funcs: &[Function], text: &[u8], label_counter: u32) -> Vec<u8> {
    let labels = plan_labels(label_counter, funcs, false);
    // One `.pdata` section for the whole TU, records in `.text` order — packed,
    // unlike `/Gy`, which gives each framed function its own COMDAT.
    let framed: Vec<&Frame> = funcs.iter().filter_map(|f| f.frame.as_ref()).collect();
    let pdata = build_pdata(&framed);

    // W13b: pool the floating-point constants, TU-wide, by bit pattern **and**
    // width (a `float` 1.0 and a `double` 1.0 are different symbols with
    // different section sizes). First-reference order fixes both the `.rdata`
    // section order and the symbol order.
    let mut pool: Vec<(u64, bool)> = Vec::new();
    for f in funcs {
        for r in &f.fp_refs {
            if !pool.contains(&(r.bits, r.double)) {
                pool.push((r.bits, r.double));
            }
        }
    }
    let pool_ix = |bits: u64, double: bool| -> usize {
        pool.iter().position(|&k| k == (bits, double)).expect("pooled")
    };

    // Section table, in the fixed emit order.
    let mut sections = shell_sections(obj_name);
    sections.push(Section {
        name: ".text",
        characteristics: CH_TEXT,
        raw: std::borrow::Cow::Borrowed(text),
        checksum: 0,
        selection: 0,
        assoc: 0,
        uninit_size: None,
    });
    let text_idx = sections.len() - 1;
    for &(bits, double) in &pool {
        sections.push(Section {
            name: ".rdata",
            characteristics: if double { CH_RDATA_F64 } else { CH_RDATA_F32 },
            raw: std::borrow::Cow::Owned(real_raw_bytes(bits, double)),
            checksum: 0,
            selection: 2,
            assoc: 0,
            uninit_size: None,
        });
    }
    // `.pdata` last — which is right only because the combination that would test
    // it is refused upstream, and **the rule it would need is now measured**.
    //
    // The comment here used to read "a TU with BOTH a constant pool and a framed
    // function would settle the `.rdata`/`.pdata` order, and none has been
    // captured". 240 such TUs were then captured (`/Ox /GS- /c`, every order of
    // one or two constant-pooling FP leaves against one or two framed functions),
    // and the answer is **not a fixed order at all**:
    //
    // > The packed section table lists `.rdata` and `.pdata` **interleaved, in
    // > `.text` order** — each section at the position of the FIRST function that
    // > needs it. `.pdata` stays a single section for the whole TU and sits where
    // > the first framed function does.
    //
    // Six distinct orders occur in those 240 objs — `(.pdata,.rdata)` 78,
    // `(.rdata,.pdata)` 64, `(.pdata,.rdata,.rdata)` 30, `(.pdata,)` 22,
    // `(.rdata,.rdata,.pdata)` 20, `(.rdata,.pdata,.rdata)` 20 — and this
    // function can express exactly one of those shapes. `L1(2.5f); seq2();
    // L2(3.5f);` is `.rdata .pdata .rdata`, which no amount of reordering the two
    // groups below produces.
    //
    // **One capture would have said the opposite.** A single leaf-then-framed TU
    // reads `.rdata .pdata`, i.e. exactly what this code already emits, and would
    // have licensed deleting the refusal. Widening here needs the interleave, not
    // a second constant in a list.
    let pdata_idx = if framed.is_empty() {
        None
    } else {
        debug_assert!(pool.is_empty(), "framed + pooled FP constant is refused upstream");
        sections.push(Section {
            name: ".pdata",
            characteristics: CH_PDATA,
            raw: std::borrow::Cow::Borrowed(&pdata),
            // The one non-COMDAT section c2 gives a real CheckSum.
            checksum: coff_checksum(&pdata),
            selection: 0,
            assoc: 0,
            uninit_size: None,
        });
        Some(sections.len() - 1)
    };
    let n_sections = sections.len();

    // Symbol layout: 13 fixed slots (indices 0..13), then per function a defined
    // FUNCTION symbol, each immediately followed by its callee's undefined
    // external symbol (if any), then — for each pooled constant this function is
    // the *first* to reference — that constant's `.rdata` section symbol (+ aux)
    // and its `__real@…` external. `_fltused` is emitted once, immediately after
    // the FIRST float function's symbol group.
    //
    // This runs before the relocations are written because each REFHI/REFLO
    // record needs its `__real@…` symbol index.
    let fltused_after = funcs.iter().position(|f| f.is_float);
    let mut next_idx: u32 = 13;
    // (function index, its defined symbol, the callee symbols it introduces —
    // reverse first-reference order, with their indices — constants introduced)
    let mut plan: Vec<(usize, u32, Vec<(&str, u32)>, Vec<usize>, Vec<(&str, u32)>)> =
        Vec::with_capacity(funcs.len());
    let mut real_idx: Vec<Option<u32>> = vec![None; pool.len()];
    // An undefined external callee is emitted **once per distinct name**, after the
    // symbol of the function that first calls it — every later call site relocates
    // against that same index. Emitting one per call site instead is invisible
    // until two functions in a TU call the same callee, which no fixture did before
    // `il_call_perm.cpp`; the reference puts `?g3` after `pass3` and nothing after
    // the four later functions that also call it.
    let mut callee_syms: Vec<(&str, u32)> = Vec::new();
    // **WR1** — the same rule for a named data symbol: one undefined external per
    // distinct name, emitted in the group of the function that first references
    // it, with every later site relocating against that index. MEASURED
    // (`work/wr1/probes/p1.cpp`): `?gI@@3HA` is referenced by three functions and
    // appears once, at index 21, which all three relocations name.
    let mut data_syms: Vec<(&str, u32)> = Vec::new();
    // Packed, the whole TU shares ONE `.pdata`, so its section symbol + aux are
    // emitted once — inside the group of the FIRST framed function, after that
    // function's prologue label and before its `$T`. Every later framed function
    // contributes only `$M`, `$M` and `$T`.
    let first_framed = funcs.iter().position(|f| f.frame.is_some());
    for (i, f) in funcs.iter().enumerate() {
        let def_idx = next_idx;
        next_idx += 1;
        if labels[i].is_some() {
            next_idx += 1; // $M(n+1), the function-end label
        }
        // One undefined external per **distinct** callee this function is the
        // first to name, in reverse first-reference order ([`Function::introduced_callees`]).
        let mut new_callees: Vec<(&str, u32)> = Vec::new();
        for name in f.introduced_callees() {
            if callee_syms.iter().any(|(n, _)| *n == name) {
                continue;
            }
            callee_syms.push((name, next_idx));
            new_callees.push((name, next_idx));
            next_idx += 1;
        }
        // …then this function's new data symbols, immediately after its callees
        // and before any label. The order inside the group is the reference's
        // (`docs/IL_CALL_IN_EXPR.md` §17.2 item 6): the callee's `26` push
        // precedes the argument's, and the emitted symbols follow the pushes.
        let mut new_data: Vec<(&str, u32)> = Vec::new();
        for r in &f.data_refs {
            if data_syms.iter().any(|(n, _)| *n == r.name) {
                continue;
            }
            data_syms.push((r.name, next_idx));
            new_data.push((r.name, next_idx));
            next_idx += 1;
        }
        if labels[i].is_some() {
            next_idx += 1; // $M(n), the prologue-end label
            if first_framed == Some(i) {
                next_idx += 2; // the shared .pdata section symbol + aux
            }
            next_idx += 1; // $T(n+2)
        }
        // Constants this function introduces, in first-reference order.
        let mut introduced: Vec<usize> = Vec::new();
        for r in &f.fp_refs {
            let k = pool_ix(r.bits, r.double);
            if real_idx[k].is_none() {
                next_idx += 2; // .rdata section symbol + its aux record
                real_idx[k] = Some(next_idx);
                next_idx += 1; // the __real@… external
                introduced.push(k);
            }
        }
        plan.push((i, def_idx, new_callees, introduced, new_data));
        if fltused_after == Some(i) {
            next_idx += 1;
        }
    }
    let n_symbols: u32 = next_idx;

    // The `.pdata` relocations: one ADDR32 per record, at the record's own
    // offset, against the framed function's defined symbol. In `.text` order,
    // which is also ascending VirtualAddress.
    let mut pdata_relocs: Vec<(u32, u32, u16)> = Vec::new();
    for (i, def, _new, _intro, _data) in &plan {
        if funcs[*i].frame.is_some() {
            pdata_relocs.push((pdata_relocs.len() as u32 * 8, *def, REL_PPC_ADDR32));
        }
    }

    // Relocations (`.text` only in this class) sit between the raw data and the
    // symbol table, **ascending by VirtualAddress**. A tail call contributes one
    // REL24; each FP constant reference contributes a REFHI/PAIR on the `addis`
    // and a REFLO/PAIR on the `lfs`/`lfd` four bytes later. The PAIR records
    // carry the partner half's displacement in the symbol-index field, which is
    // always 0 because every constant owns its whole COMDAT section.
    let mut text_relocs: Vec<(u32, u32, u16)> = Vec::new();
    for (i, _def, _new, _intro, _data) in &plan {
        let f = &funcs[*i];
        // One REL24 per call site; several sites may share one symbol index.
        for call in &f.calls {
            let cidx = callee_syms
                .iter()
                .find(|(n, _)| *n == call.callee)
                .map(|(_, ix)| *ix)
                .expect("every callee got a symbol");
            text_relocs.push((call.reloc_offset, cidx, REL_PPC_REL24));
        }
        for r in &f.fp_refs {
            let sym = real_idx[pool_ix(r.bits, r.double)].expect("pooled symbol");
            text_relocs.push((r.hi_off, sym, REL_PPC_REFHI));
            text_relocs.push((r.hi_off, 0, REL_PPC_PAIR));
            text_relocs.push((r.hi_off + 4, sym, REL_PPC_REFLO));
            text_relocs.push((r.hi_off + 4, 0, REL_PPC_PAIR));
        }
        // WR1: byte-for-byte the same quad, against an undefined external instead
        // of a pooled constant's `.rdata` symbol.
        for r in &f.data_refs {
            let sym = data_syms
                .iter()
                .find(|(n, _)| *n == r.name)
                .map(|(_, ix)| *ix)
                .expect("every data symbol got a slot");
            text_relocs.push((r.hi_off, sym, REL_PPC_REFHI));
            text_relocs.push((r.hi_off, 0, REL_PPC_PAIR));
            text_relocs.push((r.lo_off, sym, REL_PPC_REFLO));
            text_relocs.push((r.lo_off, 0, REL_PPC_PAIR));
        }
    }
    text_relocs.sort_by_key(|&(va, _, _)| va);
    let n_text_reloc = text_relocs.len();

    // Raw data is packed right after the section headers, and a section's
    // relocation records sit immediately after **that section's own** raw data —
    // not after every section's. With `.text` last (no constant pool) the two
    // layouts coincide, which is why this only surfaced once `.rdata` followed
    // `.text`: c2 put the four REFHI/REFLO records between `.text` and the
    // constant pool, the port put them after both.
    // Only `.text` and (when present) `.pdata` carry relocations in this class —
    // the `.rdata` constant pools are pure data.
    let mut n_reloc_of = vec![0u16; n_sections];
    n_reloc_of[text_idx] = n_text_reloc as u16;
    if let Some(pi) = pdata_idx {
        n_reloc_of[pi] = pdata_relocs.len() as u16;
    }
    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);
    let ptr_text_reloc = reloc_ptr[text_idx].unwrap_or(0);
    let ptr_pdata_reloc = pdata_idx.and_then(|pi| reloc_ptr[pi]).unwrap_or(0);

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);

    // ---- raw section data, each section followed by its own relocations ----
    // (10 bytes each: VA u32, SymIdx u32, Type u16)
    for (i, s) in sections.iter().enumerate() {
        debug_assert_eq!(b.0.len(), ptrs[i]);
        b.bytes(&s.raw);
        if i == text_idx {
            debug_assert!(n_text_reloc == 0 || b.0.len() == ptr_text_reloc);
            for &(va, sym, typ) in &text_relocs {
                b.u32(va);
                b.u32(sym);
                b.u16(typ);
            }
        }
        if Some(i) == pdata_idx {
            debug_assert_eq!(b.0.len(), ptr_pdata_reloc);
            for &(va, sym, typ) in &pdata_relocs {
                b.u32(va);
                b.u32(sym);
                b.u16(typ);
            }
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- symbol table + string table ----
    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections); // slots 0..=10
    // Section STATIC symbols each carry one aux section-def record. `.text`
    // (sec 5) carries the relocation count in its aux.
    emit_section_symbol(&mut b, &sections[4], 5, n_text_reloc as u16); // slot 11/12 .text

    // Per function: the defined FUNCTION symbol, then (if a tail call) the
    // undefined external callee symbol, then the constant pools this function
    // introduces (`.rdata` section symbol + aux, then the `__real@…` external).
    for (i, _def, new_callees, introduced, new_data) in &plan {
        let f = &funcs[*i];
        emit_function_symbol(&mut b, &mut strtab, f.name, 5, f.text_offset);
        // A framed function's `$M` labels are its prologue end and its function
        // end **relative to its own start**, so packed they are rebased onto the
        // shared `.text`; under `/Gy` the function starts at 0 of its own COMDAT
        // and the two coincide.
        if let (Some(m), Some(frame)) = (labels[*i], f.frame.as_ref()) {
            emit_label_symbol(&mut b, &label_name('M', m[1]), f.text_offset + frame.func_len, 5);
        }
        // Undefined external callees: section 0 (UNDEF), FUNCTION type. Only the
        // function that FIRST calls one emits its symbol, and the ones a single
        // function introduces go out in reverse first-reference order.
        for (name, _) in new_callees {
            emit_function_symbol(&mut b, &mut strtab, name, 0, 0);
        }
        // WR1: undefined external DATA symbols — section 0, `Type` 0x0000. The
        // type byte is the whole difference from the callee above, and it is the
        // difference between "a data address" and "a function pointer" in the
        // linker's eyes.
        for (name, _) in new_data {
            emit_external_symbol(&mut b, &mut strtab, name, 0, 0x0000);
        }
        if let (Some(m), Some(frame), Some(pi)) = (labels[*i], f.frame.as_ref(), pdata_idx) {
            emit_label_symbol(&mut b, &label_name('M', m[0]), f.text_offset + frame.prolog_len, 5);
            if first_framed == Some(*i) {
                emit_section_symbol(
                    &mut b,
                    &sections[pi],
                    (pi + 1) as i16,
                    pdata_relocs.len() as u16,
                );
            }
            // `$T` value is this record's byte offset inside the shared `.pdata`.
            let rec = funcs[..*i].iter().filter(|g| g.frame.is_some()).count() as u32 * 8;
            emit_pdata_label_symbol(&mut b, &label_name('T', m[2]), rec, (pi + 1) as i16);
        }
        for &k in introduced {
            let sec_num = (text_idx + 1 + k + 1) as i16;
            emit_section_symbol(&mut b, &sections[text_idx + 1 + k], sec_num, 0);
            let (bits, double) = pool[k];
            // A pooled constant is DATA, not a function: type 0x0000.
            emit_external_symbol(
                &mut b,
                &mut strtab,
                &real_symbol_name(bits, double),
                sec_num,
                0x0000,
            );
        }
        // The CRT float-support marker, once, after the first FP function.
        if fltused_after == Some(*i) {
            emit_function_symbol(&mut b, &mut strtab, NAME_FLTUSED, 0, 0);
        }
    }

    // ---- string table ----
    b.bytes(&strtab.finish());

    b.0
}

// ===========================================================================
// #158 — the `??__E` dynamic-initializer obj.
//
// A TU whose only emitted function is one `??__E<name>@@YAXXZ` thunk running
// one namespace-scope object's constructor. Eight sections, 24 symbol records,
// 9 + 1 relocations. Every byte below is transcribed from an obj produced by the
// real cl 16.00.11886.00 / c2.dll under wibo; `docs/OBJ_DYNINIT_SHAPE.md` is the
// characterization and names the cell each rule was fitted on and tested
// against. Where that doc and the bytes disagree, the bytes win, and the three
// places they do are marked CORRECTION below.
//
// **Grade at `/O1`, not `/Ox`** (§7.3 caveat 1): `/Ox` does not imply `/GF`, and
// without `/GF` the literal is a non-COMDAT `$SG<n>` `.rdata` placed *before*
// `.text`, with no `??_C@…` symbol at all. That is a different obj.
// ===========================================================================

/// `.text$yc` characteristics — CNT_CODE | COMDAT | ALIGN_8 | EXECUTE | READ.
/// Numerically the same word as an ordinary `/Gy` `.text`; the **selection**
/// is what differs (2 ANY here, 1 NODUPLICATES there), which prereg P3 got
/// backwards.
const CH_TEXT_YC: u32 = 0x6040_1020;

/// `.CRT$XCU` characteristics — CNT_INIT_DATA | ALIGN_4 | READ | WRITE.
/// ALIGN_4 in every cell measured, and **not** a COMDAT.
const CH_CRT_XCU: u32 = 0xC030_0040;

/// `.rdata` (string literal, `/GF`) characteristics with the alignment nibble
/// cleared: CNT_INIT_DATA | COMDAT | READ. OR in `nibble << 20`.
const CH_RDATA_STRING_BASE: u32 = 0x4000_1040;

/// `.bss` characteristics with the alignment nibble cleared: CNT_UNINIT_DATA |
/// READ | WRITE. OR in `nibble << 20`. **Never** a COMDAT (prereg P2, refuted
/// in that direction).
const CH_BSS_BASE: u32 = 0xC000_0080;

/// `IMAGE_COMDAT_SELECT_ANY` — the selection a `??__E` thunk's `.text$yc` and
/// its string `.rdata` carry. An *ordinary* function's `.text` uses
/// [`COMDAT_SELECT_NODUPLICATES`] instead, so this is a discriminator and not a
/// constant (prereg P3).
const COMDAT_SELECT_ANY: u8 = 2;

/// The alignment nibble (bits 23:20 of `Characteristics`) for a blob of `n`
/// bytes whose natural alignment is `t`:
///
/// > `align = max(t, 1 if n < 2 else 4 if n < 64 else 8)`
///
/// One rule for both `.bss` and the string `.rdata`, measured on both sides
/// across `n = 1, 2, 3..63, 64, 65..256` (`docs/OBJ_DYNINIT_SHAPE.md` §4.2).
/// `t` moves independently: a `double` member gives ALIGN_8 at `n = 8` where a
/// `char[8]` gives ALIGN_4.
///
/// The nibble is `log2(align) + 1` — 1→1, 2→2, 4→3, 8→4. Returns `None` for an
/// alignment that is not a power of two in 1..=8, rather than emitting a nibble
/// for a case nothing measured.
fn align_nibble(n: u32, natural: u32) -> Option<u32> {
    let implied: u32 = if n < 2 {
        1
    } else if n < 64 {
        4
    } else {
        8
    };
    match natural.max(implied) {
        1 => Some(1),
        2 => Some(2),
        4 => Some(3),
        8 => Some(4),
        _ => None,
    }
}

/// One base-16 digit in MSVC's `A`..`P` alphabet (`A` = 0 … `P` = 15).
fn base16_ap_digit(nibble: u32) -> char {
    (b'A' + nibble as u8) as char
}

/// A `u32` in base 16, digits `A`..`P`, **most-significant first with leading
/// zeros suppressed**.
///
/// The suppression is the rule the 101-byte held-out literal bought
/// (`docs/OBJ_DYNINIT_SHAPE.md` §5): its JamCRC is `0x0B7B9BC4`, the obj carries
/// the **7**-digit `LHLJLME`, and a fixed-width-8 renderer would have written
/// `ALHLJLME` — right on ~15 of 16 literals and silently wrong on the rest.
///
/// `0` renders as the empty string, which no caller may emit; both callers
/// reject it explicitly rather than inventing a spelling for it.
fn base16_ap(v: u32) -> String {
    let mut out = String::new();
    let mut started = false;
    for shift in (0..8).rev() {
        let d = (v >> (shift * 4)) & 0xF;
        if d == 0 && !started {
            continue;
        }
        started = true;
        out.push(base16_ap_digit(d));
    }
    out
}

/// The `<L>` field of a `??_C@_0…` name: `n`, the literal's byte length
/// **including the NUL**, as an MSVC-mangled number.
///
/// `1..=10` → the single character `'0' + (n - 1)`; anything larger →
/// [`base16_ap`] followed by `@`. Verified: 4→`3`, 10→`9`, 11→`L@`, 14→`O@`,
/// 16→`BA@`, 26→`BK@`, 31→`BP@`, 32→`CA@`, 33→`CB@`, 49→`DB@`, 101→`GF@`.
///
/// **CORRECTION to §5.** The doc's decomposition line writes the template as
/// `??_C@` `_0` `<L>` `@` `<H>` `@` `<text>` `@`, i.e. with an `@` between the
/// length and the hash. There is none: the obj carries
/// `??_C@_03FIKCJHKP@abc?$AA@`, where `3` is the whole length field and the
/// next character is the hash's first digit. The `@` visible in the long form
/// `_0BK@` is the **trailing `@` of this mangling**, present only for `n > 10`.
/// Coding the doc's line literally produces `??_C@_03@FIKCJHKP@abc?$AA@`.
/// Cross-checked three ways, on the string-table *sizes* of three reference
/// objs (which no part of this rule was fitted to): the fixture's table is 100
/// bytes, TomCrypt's 161 and Zlib's 175, and each is reproduced to the byte
/// only by the template as written here.
fn mangle_len(n: u32) -> String {
    if (1..=10).contains(&n) {
        ((b'0' + (n - 1) as u8) as char).to_string()
    } else {
        format!("{}@", base16_ap(n))
    }
}

/// Append one literal byte in its `??_C@…` escaped form, or return `false` if
/// its escape has **not been measured**.
///
/// Three classes, all measured (`docs/OBJ_DYNINIT_SHAPE.md` §5 plus this lane's
/// probes):
///
/// * `[A-Za-z0-9_$]` pass through literally — uppercase and `$` included.
/// * six single-`?` escapes: `?0`=`,` `?1`=`/` `?3`=`:` `?4`=`.` `?5`=space
///   `?9`=`-`.
/// * `?$` + two `A`..`P` nibble digits, MSB first, fixed width 2: NUL→`?$AA`,
///   `!`(0x21)→`?$CB`, `+`(0x2B)→`?$CL`.
///
/// **Everything else is refused, and the refusal is the point.** `?2`, `?6`,
/// `?7` and `?8` are single-`?` escape slots that this lane never observed a
/// character in. Some byte claims each of them, and it is *not* discoverable
/// from the three `?$XX` cells above which one — a byte that takes a single-`?`
/// escape in real c2 would be rendered here as a two-digit `?$XX` and the whole
/// COMDAT name, its length field and the obj's string table would all be wrong,
/// with nothing to flag it. Guessing the four unmeasured slots to widen coverage
/// is strictly worse than declining: a synthesized name that links is the
/// failure mode this project's one correctness rule exists to prevent. Only `/`
/// and `?$AA` are needed for the #158 target class.
fn escape_literal_byte(byte: u8, out: &mut String) -> bool {
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' => {
            out.push(byte as char);
            true
        }
        b',' => {
            out.push_str("?0");
            true
        }
        b'/' => {
            out.push_str("?1");
            true
        }
        b':' => {
            out.push_str("?3");
            true
        }
        b'.' => {
            out.push_str("?4");
            true
        }
        b' ' => {
            out.push_str("?5");
            true
        }
        b'-' => {
            out.push_str("?9");
            true
        }
        // The three `?$XX` cells that were actually captured.
        0x00 | b'!' | b'+' => {
            out.push_str("?$");
            out.push(base16_ap_digit((byte >> 4) as u32));
            out.push(base16_ap_digit((byte & 0xF) as u32));
            true
        }
        _ => false,
    }
}

/// How many of a literal's bytes the escaped-text field of a `??_C@…` name
/// renders before it is cut off.
///
/// **CORRECTION to §5.** The doc says the text is "truncated at 32 characters",
/// which reads as a limit on the *escaped output*. It is not: the limit is on
/// the **source bytes of `literal + NUL`**. Measured on this lane's probes —
/// a 31-character literal (32 bytes with its NUL) renders the `?$AA`, a
/// 32-character one (33 bytes) drops it, and a 30-character all-`/` literal
/// produces 54 escaped characters with nothing cut. Reading the limit as an
/// output-character budget truncates the second of those in the middle.
const LITERAL_TEXT_BYTE_LIMIT: usize = 32;

/// `??_C@_0<len><hash>@<escaped text>@` — the COMDAT symbol name c2 gives a
/// narrow (`char`) string literal under `/GF`.
///
/// `bytes` is the literal **including its trailing NUL**; that NUL is part of
/// the length, part of the hash and (unless cut by
/// [`LITERAL_TEXT_BYTE_LIMIT`]) part of the escaped text. Returns `None` when
/// any byte's escape is outside the measured set — see [`escape_literal_byte`].
///
/// Byte evidence, every literal this lane or the characterization measured:
///
/// | literal | n | JamCRC | `<H>` |
/// |---|---:|---|---|
/// | `abc` | 4 | `0x58A297AF` | `FIKCJHKP` |
/// | `defg` | 5 | `0x3F7194AC` | `DPHBJEKM` |
/// | *(empty)* | 1 | `0x2DFD1072` | `CNPNBAHC` |
/// | `Hello, world!` | 14 | `0x647FB1F9` | `GEHPLBPJ` |
/// | `xyzzy` | 6 | `0xFE973C8F` | `POJHDMIP` |
/// | `q`×100 | 101 | `0x0B7B9BC4` | `LHLJLME` |
/// | `system/src/synth/tomcrypt` | 26 | `0xF4BC3E1C` | `PELMDOBM` |
/// | `system/src/zlib` | 16 | `0x55C0A74D` | `FFMAKHEN` |
pub fn string_comdat_name(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() || *bytes.last().unwrap() != 0 {
        // The NUL is load-bearing in all three fields; a caller that dropped it
        // would get a name that is wrong everywhere and looks right nowhere.
        return None;
    }
    let hash = jamcrc(bytes);
    if hash == 0 {
        // `base16_ap(0)` is the empty string and no cell measured what c2 writes
        // for a literal whose JamCRC is zero. Refuse rather than pick between
        // "" and "A".
        return None;
    }
    let mut text = String::new();
    for &b in bytes.iter().take(LITERAL_TEXT_BYTE_LIMIT) {
        if !escape_literal_byte(b, &mut text) {
            return None;
        }
    }
    Some(format!(
        "??_C@_0{}{}@{}@",
        mangle_len(bytes.len() as u32),
        base16_ap(hash),
        text
    ))
}

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
    // `Length` is the section's declared size, which for `.bss` is NOT the raw
    // byte count (there is none) — see [`Section::uninit_size`].
    b.u32(s.size());
    b.u16(n_reloc); // NumberOfRelocations
    b.u16(0); // NumberOfLinenumbers
    b.u32(s.checksum);
    b.u16(s.assoc); // Number — 0 unless Selection=5 (ASSOCIATIVE)
    b.u8(s.selection);
    b.bytes(&[0, 0, 0]); // Unused
}

/// Emit one aux-less symbol record with an inline (≤ 8 byte) or string-table
/// name. The three axes that actually vary between the callers — `Type`
/// (0x0020 FUNCTION vs 0x0000 DATA), `StorageClass` (2 EXTERNAL vs 3 STATIC)
/// and `Value` — are arguments rather than three near-copies of this body,
/// because the dynamic-initializer obj needs the one combination none of the
/// older helpers spelled: a **STATIC symbol of FUNCTION type** (the
/// `??__E<name>@@YAXXZ` thunk, `docs/OBJ_DYNINIT_SHAPE.md` §3.1).
fn emit_symbol(
    b: &mut Buf,
    strtab: &mut StringTable,
    name: &str,
    value: u32,
    sec_num: i16,
    typ: u16,
    storage_class: u8,
) {
    if name.len() <= 8 {
        b.name8(name);
    } else {
        let off = strtab.intern(name);
        b.u32(0); // long-name marker
        b.u32(off);
    }
    b.u32(value);
    b.i16(sec_num);
    b.u16(typ);
    b.u8(storage_class);
    b.u8(0); // no aux
}

/// Emit an EXTERNAL symbol whose (long) name lives in the string table.
fn emit_external_symbol(b: &mut Buf, strtab: &mut StringTable, name: &str, sec_num: i16, typ: u16) {
    emit_symbol(b, strtab, name, 0, sec_num, typ, 2);
}

/// Emit an EXTERNAL FUNCTION symbol (type 0x20) whose (long) name lives in the
/// string table, with `Value` = its byte offset within `.text`.
fn emit_function_symbol(b: &mut Buf, strtab: &mut StringTable, name: &str, sec_num: i16, value: u32) {
    emit_symbol(b, strtab, name, value, sec_num, 0x0020, 2);
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

    /// Five representative objs from the three pre-existing emitters, reduced to
    /// `(length, CRC)` — a byte-level pin taken **before** the shared-primitive
    /// refactor that `emit_dyninit_obj` needed, so that refactor could be proved
    /// output-preserving rather than asserted to be.
    ///
    /// `emit_obj`, `emit_comdat_obj` and `emit_empty_obj` had their section
    /// layout, section-header writing and 11-symbol shell open-coded three times
    /// over; the dynamic-initializer obj needs a `.bss` section whose
    /// `SizeOfRawData` is non-zero while it contributes **no** file bytes, which
    /// touches all three of those. A fourth open-coded copy is this file's
    /// recorded defect shape (see the `emit_framed_obj` note above), so the
    /// copies were merged instead — and this test is what said the merge changed
    /// nothing. It is not a spec: if a *deliberate* change to one of those three
    /// emitters lands, re-derive these numbers from the reference obj, never from
    /// the port.
    fn obj_fingerprints() -> Vec<(&'static str, usize, u32)> {
        let mk_call = || Function {
            calls: vec![Call { reloc_offset: 0x0C, callee: "?g@@YAHH@Z" }],
            frame: Some(Frame { prolog_len: 0x0C, func_len: 0x24 }),
            ..Function::plain("?f@@YAHH@Z", 0)
        };
        let mk_data = || Function {
            calls: vec![Call { reloc_offset: 12, callee: "?gsp@@YAXPAHH@Z" }],
            data_refs: vec![DataRef { hi_off: 0, lo_off: 8, name: "?gI@@3HA" }],
            ..Function::plain("?a7@@YAXXZ", 0)
        };
        let mk_fp = || Function {
            is_float: true,
            fp_refs: vec![crate::codegen::FpConstRef {
                hi_off: 0,
                bits: 0x3FF0_0000_0000_0000,
                double: false,
            }],
            ..Function::plain("?fc@@YAMXZ", 0)
        };
        let blr = crate::codegen::encode_blr().to_vec();
        let objs: Vec<(&'static str, Vec<u8>)> = vec![
            ("empty", emit_empty_obj(r"Z:\tmp\anat\mvp.obj")),
            ("mvp", emit_mvp_obj(r"Z:\tmp\anat\mvp.obj", "?add3@@YAHHHH@Z", &[0u8; 12])),
            ("framed", emit_obj(r"Z:\t\f.obj", &[mk_call()], &[0u8; 0x24], 2536)),
            ("dataref", emit_obj(r"Z:\t\a7.obj", &[mk_data()], &[0u8; 16], 2536)),
            ("fppool", emit_obj(r"Z:\t\fc.obj", &[mk_fp()], &[0u8; 12], 2536)),
            (
                "comdat",
                emit_comdat_obj(
                    r"Z:\t\s.obj",
                    &[mk_call(), mk_data()],
                    &[vec![0u8; 0x24], vec![0u8; 16]],
                    2536,
                ),
            ),
            (
                "comdat_plain",
                emit_comdat_obj(
                    r"Z:\x.obj",
                    &[
                        Function::plain("?SpewInit@@YAXXZ", 0),
                        Function::plain("?SpewTerminate@@YAXXZ", 0),
                    ],
                    &[blr.clone(), blr],
                    0,
                ),
            ),
        ];
        objs.into_iter().map(|(k, o)| (k, o.len(), coff_checksum(&o))).collect()
    }

    #[test]
    fn the_three_pre_existing_emitters_are_byte_stable() {
        assert_eq!(
            obj_fingerprints(),
            vec![
                ("empty", 668, 0x7E17_6256u32),
                ("mvp", 790, 0x8036_8217),
                ("framed", 984, 0x529B_5631),
                ("dataref", 883, 0x187A_138D),
                ("fppool", 949, 0x1EEB_0597),
                ("comdat", 1207, 0xB4F7_683C),
                ("comdat_plain", 891, 0xC7A4_226B),
            ]
        );
    }

    #[test]
    fn drectve_is_132_bytes() {
        assert_eq!(DRECTVE.len(), 132, "drectve must be exactly 132 bytes");
    }

    #[test]
    fn s_compile2_is_57_bytes() {
        assert_eq!(S_COMPILE2.len(), 57);
    }

    /// The two lengths of the `+k` frame class, as a `Frame`.
    fn frame(func_len: u32) -> Frame {
        Frame { prolog_len: 0x0C, func_len }
    }

    #[test]
    fn pdata_unwind_word_encodes_function_and_prologue_lengths() {
        // 0x24 body (9 words, +k class) → BeginAddress 0 + big-endian
        // 0x40000903. 0x28 body (10 words, *5) → 0x40000A03 (length +1 = +0x100).
        assert_eq!(pdata_record(0, &frame(0x24)), [0, 0, 0, 0, 0x40, 0x00, 0x09, 0x03]);
        assert_eq!(pdata_record(0, &frame(0x28)), [0, 0, 0, 0, 0x40, 0x00, 0x0A, 0x03]);
        // The prologue field is the low byte and moves independently: the
        // two-call `r30`/`r31` body is 18 words with a 5-word prologue, and the
        // 100 KB-frame body 22 words with a 7-word one. Both read straight out
        // of reference objs; `build_pdata` hardcoded 3 until this landed.
        assert_eq!(
            pdata_record(0, &Frame { prolog_len: 0x14, func_len: 0x48 }),
            [0, 0, 0, 0, 0x40, 0x00, 0x12, 0x05]
        );
        assert_eq!(
            pdata_record(0, &Frame { prolog_len: 0x1C, func_len: 0x58 }),
            [0, 0, 0, 0, 0x40, 0x00, 0x16, 0x07]
        );
    }

    #[test]
    fn pdata_checksum_matches_reference_aux() {
        // The `.pdata` aux CheckSum in the reference obj (0xd3dfb2ce for the +k
        // frame) is the reflected CRC-32 of the 8 raw bytes.
        assert_eq!(coff_checksum(&build_pdata(&[&frame(0x24)])), 0xD3DF_B2CE);
        assert_eq!(coff_checksum(&build_pdata(&[&frame(0x28)])), 0xF8F2_E10D);
    }

    #[test]
    fn label_plan_matches_the_captured_counters() {
        let leaf = Function::plain("?L@@YAHH@Z", 0);
        let framed = |name| Function {
            frame: Some(frame(0x24)),
            ..Function::plain(name, 0)
        };
        // mvp_framed: one framed function, `.gl` counter 2536 → $M2545/6, $T2547.
        assert_eq!(
            plan_labels(2536, &[framed("?f@@YAHH@Z")], false),
            vec![Some([2545, 2546, 2547])]
        );
        // Under `/Gy` the same TU pays a flat 3-per-function surcharge first.
        assert_eq!(
            plan_labels(2536, &[framed("?f@@YAHH@Z")], true),
            vec![Some([2548, 2549, 2550])]
        );
        // A leading leaf consumes exactly one slot (`n1`: counter 2539 → 2549).
        assert_eq!(
            plan_labels(2539, &[leaf, framed("?F@@YAHH@Z")], false),
            vec![None, Some([2549, 2550, 2551])]
        );
        // Framed stride: 4 packed, 5 under `/Gy` (`m2`, counter 2539).
        let two = [framed("?F1@@YAHH@Z"), framed("?F2@@YAHH@Z")];
        assert_eq!(
            plan_labels(2539, &two, false),
            vec![Some([2548, 2549, 2550]), Some([2552, 2553, 2554])]
        );
        assert_eq!(
            plan_labels(2539, &two, true),
            vec![Some([2554, 2555, 2556]), Some([2559, 2560, 2561])]
        );
    }

    /// **The leading surcharge moves the function's OWN triple, not just the
    /// next one's** — which is the whole reason it is a separate field rather
    /// than a bigger stride, and the direction a "stride 7" model gets wrong.
    /// Allocating the same two slots after the triple instead of before it is
    /// **119 mismatches** in `scripts/sweep.d/98-cmp-order.py`, the same number
    /// as dropping them entirely: the total and the placement are two claims and
    /// this test is the one that pins the second.
    ///
    /// Measured: a signed `>`/`<` two-call comparator is stride 7 / lead 2 under
    /// `/Gy`, 6 / 2 packed (`scripts/gt_cmp_rr.py --stride`).
    #[test]
    fn a_leading_label_surcharge_moves_its_own_triple_and_every_later_one() {
        let cmp = |name| Function {
            frame: Some(frame(0x24)),
            label_lead: 2,
            ..Function::plain(name, 0)
        };
        let plain = |name| Function {
            frame: Some(frame(0x24)),
            ..Function::plain(name, 0)
        };
        // Packed: base 2545 (see the row above), + 2 for the lead, then the
        // following function starts 6 later rather than 4.
        assert_eq!(
            plan_labels(2536, &[cmp("?c@@YA_NPBU@Z"), plain("?f@@YAHH@Z")], false),
            vec![Some([2547, 2548, 2549]), Some([2551, 2552, 2553])]
        );
        // `/Gy`: the flat 3-per-function pre-pass, then the same +2 / stride 7.
        assert_eq!(
            plan_labels(2536, &[cmp("?c@@YA_NPBU@Z"), plain("?f@@YAHH@Z")], true),
            vec![Some([2553, 2554, 2555]), Some([2558, 2559, 2560])]
        );
        // A lead of 0 is the shipped behaviour, unchanged.
        assert_eq!(
            plan_labels(2536, &[plain("?f@@YAHH@Z")], false),
            vec![Some([2545, 2546, 2547])]
        );
    }

    #[test]
    fn framed_obj_has_six_sections_and_twenty_symbols() {
        // A framed obj built with the verified 0x24 text: 6 sections, 20 symbols.
        let text = vec![0u8; 0x24];
        let f = Function {
            calls: vec![Call { reloc_offset: 0x0C, callee: "?g@@YAHH@Z" }],
            frame: Some(frame(0x24)),
            ..Function::plain("?f@@YAHH@Z", 0)
        };
        let obj = emit_obj(r"Z:\t\f.obj", &[f], &text, 2536);
        assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 6); // NumberOfSections
        assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 20); // NumberOfSymbols
    }

    #[test]
    fn debug_s_size_for_mvp_path() {
        // "Z:\tmp\anat\mvp.obj" = 19 chars → raw 97 → padded 100.
        let d = build_debug_s(r"Z:\tmp\anat\mvp.obj");
        assert_eq!(d.len(), 100);
    }

    // -----------------------------------------------------------------------
    // #137 — the PORTABLE pins for WR1's two ordering rules.
    //
    // WR1 landed 150 lines in this file and moved the workspace test-block total
    // (the attribute is spelled out in prose on purpose: `git grep -c` for it is
    // how §9.10 counts, and a literal in a comment inflates that count by one —
    // this lane's own first tally read 580 blocks against 579 running tests)
    // by **zero** (`docs/ROADMAP.md` §9.10). Its two ordering rules were pinned
    // only by `fixtures/cpp/wr1_sym_addr.cpp`, and the mutation table in §9.12
    // shows what that was worth: with the address rule inverted, or with the
    // REFLO offset forced back to `hi_off + 4`, `cargo test --workspace` is
    // **571 passed / 0 failed in BOTH lanes** — the portable one *and* the one
    // with the toolchain resolving, because `differential.rs` names three
    // fixtures and `wr1_sym_addr.cpp` is not among them. Only `scripts/gate.sh`
    // went red (10 of 12 lanes). These tests move that pin into `cargo test`.
    // -----------------------------------------------------------------------

    /// A COFF section header field reader, used only by the tests below.
    /// Deliberately a *separate* walk of the container from the emitter's — the
    /// point of a pin is that it fails when the emitter changes, so it must not
    /// share the emitter's arithmetic.
    fn text_relocations(obj: &[u8]) -> Vec<(u32, u32, u16)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let n_sections = u16at(2) as usize;
        let mut out = Vec::new();
        for s in 0..n_sections {
            let h = COFF_HEADER_LEN + s * SECTION_HEADER_LEN;
            if &obj[h..h + 5] != b".text" {
                continue;
            }
            let ptr = u32at(h + 24) as usize;
            let n = u16at(h + 32) as usize;
            for r in 0..n {
                let o = ptr + r * 10;
                out.push((u32at(o), u32at(o + 4), u16at(o + 8)));
            }
        }
        out
    }

    /// **#137 rule 2 — the REFHI/REFLO quad's halves are NOT adjacent.**
    ///
    /// The `lis rS,sym@ha` is hoisted to the top of the body while the
    /// `addi rD,rS,sym@l` is emitted after the rest of the argument setup, so a
    /// literal slot lands *between* them and REFLO is **not** at `hi_off + 4`.
    /// MEASURED, `work/wr1/probes/p4.cpp`: `void a7(){ gsp(&gI, 7); }` is
    /// `lis r11 · li r4,7 · addi r3,r11,0 · b`, REFLO **eight** bytes past
    /// REFHI. Emitting the quad as the adjacent pair a pooled FP constant uses
    /// was a live wrong-bytes emit on exactly that body.
    ///
    /// The input here is that body's shape and nothing else: `hi_off` 0 and
    /// `lo_off` 8, four words of `.text`. Every assertion carries its own
    /// message and the two quantities the later ones rest on — how many
    /// relocation records the section has, and that `hi_off + 4` is a real
    /// offset inside the body rather than past its end — are pinned first, so a
    /// broken reader goes red on its own line instead of making the offset
    /// assertions unreachable.
    #[test]
    fn the_data_address_quad_puts_reflo_at_its_own_offset_not_beside_refhi() {
        let text = vec![0u8; 16]; // lis · li · addi · b
        let f = Function {
            calls: vec![Call { reloc_offset: 12, callee: "?gsp@@YAXPAHH@Z" }],
            data_refs: vec![DataRef { hi_off: 0, lo_off: 8, name: "?gI@@3HA" }],
            ..Function::plain("?a7@@YAXXZ", 0)
        };
        let obj = emit_obj(r"Z:\t\a7.obj", &[f], &text, 2536);
        let recs = text_relocations(&obj);

        // (a) The fixture property, pinned over the INPUT and not over the rule
        // under test: the two halves are 8 bytes apart, so `hi_off + 4` is a
        // different word of a body that actually has one there. Without this the
        // test could be satisfied by a body too short to tell the two apart.
        assert_eq!(
            (0u32, 8u32, text.len()),
            (0, 8, 16),
            "(a) the discriminating body is `lis · li · addi · b` with the halves \
             8 bytes apart and a real word at +4"
        );
        // (b) One REL24 for the branch plus the quad — and nothing else. Pinned
        // before any record is inspected by index.
        assert_eq!(
            recs.len(),
            5,
            "(b) expected 5 .text relocation records (1 REL24 + a REFHI/PAIR/\
             REFLO/PAIR quad), got {}",
            recs.len()
        );
        // (c) REFHI sits at the hoisted `lis`, offset 0.
        let refhi: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFHI).map(|r| r.0).collect();
        assert_eq!(refhi, vec![0], "(c) REFHI is not at the hoisted `lis` (offset 0): {refhi:?}");
        // (d) **The rule.** REFLO is at the `addi`'s own offset, 8 — NOT at
        // `hi_off + 4` = 4, which is where the literal's `li` is.
        let reflo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert_eq!(
            reflo,
            vec![8],
            "(d) REFLO must be at the `addi`'s own offset 8, not at hi_off+4 = 4 \
             — the two halves of the quad are NOT adjacent: {reflo:?}"
        );
        // (e) Both PAIRs shadow their own half, and against symbol index 0.
        let pairs: Vec<(u32, u32)> =
            recs.iter().filter(|r| r.2 == REL_PPC_PAIR).map(|r| (r.0, r.1)).collect();
        assert_eq!(
            pairs,
            vec![(0, 0), (8, 0)],
            "(e) each PAIR shadows its own half's offset against symbol 0: {pairs:?}"
        );
        // (f) Records are ascending by VirtualAddress and REFHI precedes its
        // PAIR at the equal VA — the order c2 writes them in.
        let order: Vec<(u32, u16)> = recs.iter().map(|r| (r.0, r.2)).collect();
        assert_eq!(
            order,
            vec![
                (0, REL_PPC_REFHI),
                (0, REL_PPC_PAIR),
                (8, REL_PPC_REFLO),
                (8, REL_PPC_PAIR),
                (12, REL_PPC_REL24),
            ],
            "(f) the .text relocation records are not in ascending-VA order with \
             REFHI ahead of its PAIR: {order:?}"
        );
    }

    /// The same rule in the **`/Gy` COMDAT** emitter, which is a second copy of
    /// the quad code — and a second copy of one fact is this file's recorded
    /// defect shape (see the `emit_framed_obj` note above). One emitter fixed
    /// and one not is exactly how the `.pdata`-ordering bug survived.
    #[test]
    fn the_comdat_emitter_places_reflo_at_its_own_offset_too() {
        let text = vec![0u8; 16];
        let f = Function {
            calls: vec![Call { reloc_offset: 12, callee: "?gsp@@YAXPAHH@Z" }],
            data_refs: vec![DataRef { hi_off: 0, lo_off: 8, name: "?gI@@3HA" }],
            ..Function::plain("?a7@@YAXXZ", 0)
        };
        let obj = emit_comdat_obj(r"Z:\t\a7.obj", &[f], &[text], 2536);
        let recs = text_relocations(&obj);
        assert_eq!(
            recs.len(),
            5,
            "(g) the COMDAT emitter wrote {} .text relocation records, expected 5",
            recs.len()
        );
        let reflo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert_eq!(
            reflo,
            vec![8],
            "(h) COMDAT emitter: REFLO must be at the `addi`'s own offset 8, not \
             at hi_off+4 = 4: {reflo:?}"
        );
    }

    /// Every COFF symbol record's `(name, Value, SectionNumber)`, in table
    /// order. A second walk of the container, like [`text_relocations`].
    fn symbols(obj: &[u8]) -> Vec<(String, u32, i16)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let ptr = u32at(8) as usize;
        let n = u32at(12) as usize;
        let strtab = ptr + n * 18;
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let r = ptr + i * 18;
            let name = if u32at(r) == 0 {
                let off = strtab + u32at(r + 4) as usize;
                let end = obj[off..].iter().position(|&c| c == 0).unwrap_or(0) + off;
                String::from_utf8_lossy(&obj[off..end]).into_owned()
            } else {
                let raw = &obj[r..r + 8];
                let end = raw.iter().position(|&c| c == 0).unwrap_or(8);
                String::from_utf8_lossy(&raw[..end]).into_owned()
            };
            out.push((name, u32at(r + 8), u16at(r + 12) as i16));
            i += 1 + obj[r + 17] as usize;
        }
        out
    }

    /// **#135/#137 — the compiler-label triple's three slots are not
    /// interchangeable, and the symbol table emits the two `$M` out of numeric
    /// order.** Asserted in BOTH emitters.
    ///
    /// `plan_labels` hands back `[n, n+1, n+2]` and the emitter binds them:
    /// `$M(n)` carries the **prologue** length, `$M(n+1)` the **function**
    /// length, `$T(n+2)` the `.pdata` record — and the two `$M` records are
    /// written `$M(n+1)` **first**, `$M(n)` second, with the callee external
    /// between them. Nothing pinned either fact portably; swapping the two
    /// `Value`s is six wrong bytes in an obj that still links, which is this
    /// file's recorded defect class (#5).
    ///
    /// **Both emitters, because there are two copies of this binding** — and
    /// the first draft of this test called only [`emit_comdat_obj`], under which
    /// swapping the two `$M` in [`emit_obj`] left `cargo test` **85 passed / 0
    /// failed**. One rule in two emitters, pinned in one, is how the `.pdata`
    /// ordering bug survived (see the `emit_framed_obj` note above).
    ///
    /// The number→meaning half is **independently confirmed by `.cod`**
    /// (`scripts/gt_label_cod.py`, `docs/ROADMAP.md` §9.12): on 56 of 56 graded
    /// bodies across 20 shapes and 4 flag sets the listing prints `$M(n)` at a
    /// **lower** text offset than `$M(n+1)` in the same body — the prologue end
    /// really is the lower number. Measured on both sides of the seam.
    #[test]
    fn the_label_triple_binds_prolog_to_n_and_function_length_to_n_plus_one() {
        let mk = || Function {
            calls: vec![Call { reloc_offset: 0x0C, callee: "?g@@YAHH@Z" }],
            frame: Some(Frame { prolog_len: 0x0C, func_len: 0x24 }),
            ..Function::plain("?f@@YAHH@Z", 0)
        };
        let text = vec![0u8; 0x24];

        // (l) The triples this obj is supposed to carry, pinned against
        // `plan_labels` itself so the assertions below name real symbols. If
        // the planner moves, this line goes red rather than the later ones
        // silently comparing `None` to `None`. Packed is 4 lower than `/Gy`:
        // 2536 + LABEL_SEED_GAP = 2545, plus the flat 3-per-function pre-pass.
        let planned = |comdat| {
            plan_labels(2536, &[mk()], comdat)[0].expect("a framed function gets a triple")
        };
        assert_eq!(
            (planned(false), planned(true)),
            ([2545, 2546, 2547], [2548, 2549, 2550]),
            "(l) the planned triple moved: packed {:?}, /Gy {:?}",
            planned(false),
            planned(true)
        );

        for (tag, obj, m) in [
            ("packed", emit_obj(r"Z:\t\f.obj", &[mk()], &text, 2536), planned(false)),
            (
                "/Gy",
                emit_comdat_obj(r"Z:\t\f.obj", &[mk()], &[text.clone()], 2536),
                planned(true),
            ),
        ] {
            let syms = symbols(&obj);
            let n0 = label_name('M', m[0]);
            let n1 = label_name('M', m[1]);
            let n2 = label_name('T', m[2]);
            let ix = |n: &str| syms.iter().position(|s| s.0 == n);
            let val = |n: &str| syms.iter().find(|s| s.0 == n).map(|s| s.1);

            // (m) All three symbols are present, under `label_name`'s spelling.
            for n in [&n0, &n1, &n2] {
                assert!(ix(n).is_some(), "(m) {tag}: the obj has no symbol named {n}");
            }

            // (n) **The binding.** `$M(n)` is the PROLOGUE length and `$M(n+1)`
            // the FUNCTION length — not the other way round.
            assert_eq!(
                (val(&n0), val(&n1)),
                (Some(0x0C), Some(0x24)),
                "(n) {tag}: $M(n)={n0} must carry the prologue length 0x0C and \
                 $M(n+1)={n1} the function length 0x24 — swapping them is six \
                 wrong bytes in an obj that still links"
            );

            // (o) **The emission order**, the opposite of the numeric order:
            // `$M(n+1)` is written BEFORE `$M(n)`, and `$T(n+2)` after both.
            let (a, b, c) = (ix(&n1).unwrap(), ix(&n0).unwrap(), ix(&n2).unwrap());
            assert!(
                a < b && b < c,
                "(o) {tag}: the symbol table must carry $M(n+1) before $M(n) \
                 before $T(n+2); got {n1} at {a}, {n0} at {b}, {n2} at {c}"
            );

            // (o2) …with the callee external BETWEEN the two `$M`.
            let callee = ix("?g@@YAHH@Z")
                .unwrap_or_else(|| panic!("(o2) {tag}: the callee symbol is missing"));
            assert!(
                a < callee && callee < b,
                "(o2) {tag}: the callee external sits between $M(n+1) and $M(n): \
                 {n1} at {a}, callee at {callee}, {n0} at {b}"
            );

            // (p) `$T(n+2)` is the `.pdata` record's own label and is the only
            // member of the triple that leaves the code section.
            let t_sec = syms.iter().find(|s| s.0 == n2).map(|s| s.2).unwrap();
            let m_sec = syms.iter().find(|s| s.0 == n0).map(|s| s.2).unwrap();
            assert_ne!(
                t_sec, m_sec,
                "(p) {tag}: $T(n+2) must live in `.pdata`, not beside the two $M \
                 in `.text` (both read section {t_sec})"
            );
        }
    }

    // =======================================================================
    // #158 — the dynamic-initializer obj.
    //
    // PORTABLE pins (prereg D2). `cargo test` has twice missed an ordering bug
    // in this file that only `scripts/gate.sh` caught — the callee-per-call-site
    // inflation and the batched-relocations layout — because the shapes that
    // discriminate them were reachable only through a fixture. Everything below
    // runs with **no toolchain**: `emit_dyninit_obj` plus a parser written here,
    // deliberately a separate walk of the container from the emitter's.
    // =======================================================================

    /// Every section header, as `(name, SizeOfRawData, PointerToRawData,
    /// PointerToRelocations, NumberOfRelocations, VirtualSize, Characteristics)`.
    fn sections_of(obj: &[u8]) -> Vec<(String, u32, u32, u32, u16, u32, u32)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        (0..u16at(2) as usize)
            .map(|s| {
                let h = COFF_HEADER_LEN + s * SECTION_HEADER_LEN;
                let end = obj[h..h + 8].iter().position(|&c| c == 0).unwrap_or(8);
                (
                    String::from_utf8_lossy(&obj[h..h + end]).into_owned(),
                    u32at(h + 16),
                    u32at(h + 20),
                    u32at(h + 24),
                    u16at(h + 32),
                    u32at(h + 8),
                    u32at(h + 36),
                )
            })
            .collect()
    }

    /// Every relocation record of the named section, in file order.
    fn relocations_of(obj: &[u8], want: &str) -> Vec<(u32, u32, u16)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let mut out = Vec::new();
        for s in sections_of(obj).iter() {
            if s.0 != want {
                continue;
            }
            for r in 0..s.4 as usize {
                let o = s.3 as usize + r * RELOC_LEN;
                out.push((u32at(o), u32at(o + 4), u16at(o + 8)));
            }
        }
        out
    }

    /// Every symbol record as `(name, Value, SectionNumber, Type, StorageClass,
    /// nAux)`, aux records skipped — plus, for a symbol that has one, its aux
    /// decoded as `(Length, nReloc, CheckSum, Number, Selection)`.
    #[allow(clippy::type_complexity)]
    fn symbols_full(
        obj: &[u8],
    ) -> Vec<((String, u32, i16, u16, u8, u8), Option<(u32, u16, u32, u16, u8)>)> {
        let u16at = |o: usize| u16::from_le_bytes([obj[o], obj[o + 1]]);
        let u32at = |o: usize| u32::from_le_bytes([obj[o], obj[o + 1], obj[o + 2], obj[o + 3]]);
        let ptr = u32at(8) as usize;
        let n = u32at(12) as usize;
        let strtab = ptr + n * SYMBOL_LEN;
        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            let r = ptr + i * SYMBOL_LEN;
            let name = if u32at(r) == 0 {
                let off = strtab + u32at(r + 4) as usize;
                let end = obj[off..].iter().position(|&c| c == 0).unwrap_or(0) + off;
                String::from_utf8_lossy(&obj[off..end]).into_owned()
            } else {
                let raw = &obj[r..r + 8];
                let end = raw.iter().position(|&c| c == 0).unwrap_or(8);
                String::from_utf8_lossy(&raw[..end]).into_owned()
            };
            let naux = obj[r + 17];
            let aux = if naux == 1 {
                let a = r + SYMBOL_LEN;
                Some((u32at(a), u16at(a + 4), u32at(a + 8), u16at(a + 12), obj[a + 14]))
            } else {
                None
            };
            out.push((
                (name, u32at(r + 8), u16at(r + 12) as i16, u16at(r + 14), obj[r + 16], naux),
                aux,
            ));
            i += 1 + naux as usize;
        }
        out
    }

    /// The `.text$yc` payload shared byte-for-byte by the fixture and both
    /// workload TUs (`docs/OBJ_DYNINIT_SHAPE.md` §3.3):
    /// `lis r11 · lis r10 · addi r4,r11 · addi r3,r10 · li r5,0 · b -0x14`.
    const DYNINIT_TEXT: [u8; 0x18] = [
        0x3d, 0x60, 0x00, 0x00, 0x3d, 0x40, 0x00, 0x00, 0x38, 0x8b, 0x00, 0x00, 0x38, 0x6a, 0x00,
        0x00, 0x38, 0xa0, 0x00, 0x00, 0x4b, 0xff, 0xff, 0xec,
    ];

    /// The reference cell: `fixtures/cpp/il_dyninit_static.cpp`,
    /// `struct L { L(const char*, int); }; static L sL("abc", 0);` at
    /// `/O1 /Oi /EHsc /GS- /c`.
    fn fixture_obj() -> Vec<u8> {
        let lit = StringLiteral { bytes: b"abc\0" };
        let name = string_comdat_name(lit.bytes).expect("the fixture literal is representable");
        let thunk = DynInitThunk {
            name: "??__EsL@@YAXXZ",
            text: &DYNINIT_TEXT,
            calls: vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }],
            data_refs: vec![
                DataRef { hi_off: 0x00, lo_off: 0x08, name: &name },
                DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL" },
            ],
        };
        let object = BssObject {
            symbol: "sL",
            size: 1,
            natural_align: 1,
            external: false,
            initializer_symbol: "sL$initializer$",
        };
        emit_dyninit_obj(r"Z:\tmp\anat\mvp.obj", &thunk, Some(&lit), &object)
            .expect("the reference cell is in class")
    }

    /// **The eight verified literals**, name for name. The hash column is
    /// `docs/OBJ_DYNINIT_SHAPE.md` §5; the full names are the ones the reference
    /// objs' symbol tables carry.
    #[test]
    fn the_string_comdat_name_matches_every_measured_literal() {
        // The 101-byte held-out cell, built rather than typed: a 7-digit hash
        // (the leading `A` suppressed) and an escaped text cut at 32 source
        // bytes. Miscounting the `q`s by one silently grades a different cell.
        let q100 = {
            let mut v = vec![b'q'; 100];
            v.push(0);
            v
        };
        assert_eq!(q100.len(), 101);
        assert_eq!(
            string_comdat_name(&q100).as_deref(),
            Some("??_C@_0GF@LHLJLME@qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq@"),
            "the 101-byte cell: 7 hash digits, and the text cut at 32 source bytes"
        );

        let cases: [(&[u8], &str); 7] = [
            (b"abc\0", "??_C@_03FIKCJHKP@abc?$AA@"),
            (b"defg\0", "??_C@_04DPHBJEKM@defg?$AA@"),
            (b"\0", "??_C@_00CNPNBAHC@?$AA@"),
            (b"Hello, world!\0", "??_C@_0O@GEHPLBPJ@Hello?0?5world?$CB?$AA@"),
            (b"xyzzy\0", "??_C@_05POJHDMIP@xyzzy?$AA@"),
            (
                b"system/src/synth/tomcrypt\0",
                "??_C@_0BK@PELMDOBM@system?1src?1synth?1tomcrypt?$AA@",
            ),
            (b"system/src/zlib\0", "??_C@_0BA@FFMAKHEN@system?1src?1zlib?$AA@"),
        ];
        for (bytes, want) in cases {
            assert_eq!(
                string_comdat_name(bytes).as_deref(),
                Some(want),
                "literal {:?}",
                String::from_utf8_lossy(&bytes[..bytes.len() - 1])
            );
        }
    }

    /// **The swapped-init trap, made a test.** §2.3 closes by naming it: the
    /// same polynomial appears twice with different initial values — section
    /// aux CheckSum init `0`, string-name hash init `0xFFFFFFFF` — and getting
    /// them the wrong way round is the obvious way to implement this wrong.
    /// Both values are 32 bits of noise, so nothing else in the port notices.
    #[test]
    fn the_two_crc_initial_values_are_not_interchangeable() {
        for lit in [&b"abc\0"[..], b"defg\0", b"xyzzy\0", b"system/src/zlib\0"] {
            assert_ne!(
                coff_checksum(lit),
                jamcrc(lit),
                "the aux checksum and the name hash must not coincide on {lit:?}"
            );
        }
        // The measured pairs, both directions pinned on one literal.
        assert_eq!(jamcrc(b"abc\0"), 0x58A2_97AF, "JamCRC uses init 0xFFFFFFFF");
        assert_eq!(coff_checksum(b"abc\0"), 0x8619_B74C, "the aux CheckSum uses init 0");
        assert_eq!(jamcrc(b"defg\0"), 0x3F71_94AC);
        assert_eq!(coff_checksum(b"defg\0"), 0x06AC_9C4E);
        assert_eq!(jamcrc(b"xyzzy\0"), 0xFE97_3C8F);
        assert_eq!(coff_checksum(b"xyzzy\0"), 0xB0AA_62D3);
        // …and the two `.XBLD$W` constants, which predate this lane, are init-0.
        assert_eq!(coff_checksum(&XBLD_C2), XBLD_C2_CHECKSUM);
        assert_eq!(coff_checksum(&XBLD_C1), XBLD_C1_CHECKSUM);
    }

    /// **The refusal, which is the deliberate part.** `?2`, `?6`, `?7` and `?8`
    /// are single-`?` escape slots this lane never observed a character in. A
    /// byte that takes one of them in real c2 would be rendered here as a
    /// two-digit `?$XX`, and the COMDAT name, the length field and the obj's
    /// whole string table would be wrong with nothing to flag it — so any byte
    /// outside the measured set refuses the name, and the caller refuses the obj.
    #[test]
    fn an_unmeasured_escape_byte_refuses_the_name_rather_than_guessing() {
        // Backslash, newline, tab, apostrophe, `<`, `%`, `#`, and a high byte:
        // all plausible occupants of ?2/?6/?7/?8 or of an unverified `?$XX`.
        for b in [b'\\', b'\n', b'\t', b'\'', b'<', b'%', b'#', 0xE9] {
            let lit = [b'a', b, 0];
            assert_eq!(
                string_comdat_name(&lit),
                None,
                "byte {b:#04x} has no measured escape and must refuse"
            );
        }
        // A missing NUL refuses too — it is part of the length, the hash and the
        // text, so a caller that dropped it gets a name wrong in three places.
        assert_eq!(string_comdat_name(b"abc"), None);
        assert_eq!(string_comdat_name(b""), None);
        // …and the whole obj declines with it.
        let lit = StringLiteral { bytes: b"a\\b\0" };
        let thunk = DynInitThunk {
            name: "??__EsL@@YAXXZ",
            text: &DYNINIT_TEXT,
            calls: vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }],
            data_refs: vec![
                DataRef { hi_off: 0x00, lo_off: 0x08, name: "unused" },
                DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL" },
            ],
        };
        let object = BssObject {
            symbol: "sL",
            size: 1,
            natural_align: 1,
            external: false,
            initializer_symbol: "sL$initializer$",
        };
        assert_eq!(
            emit_dyninit_obj(r"Z:\t\x.obj", &thunk, Some(&lit), &object).map(|o| o.len()),
            None,
            "an unrepresentable literal must decline the whole obj"
        );
    }

    /// **CORRECTION to §5's "truncated at 32 characters".** The limit is on the
    /// *source* bytes of `literal + NUL`, not on the escaped output. Three
    /// discriminating cells, none of which the doc's reading gets right.
    #[test]
    fn the_escaped_text_is_cut_at_thirty_two_source_bytes_not_output_characters() {
        // 31 source characters = 32 bytes with the NUL → the `?$AA` IS rendered.
        let n31 = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0";
        assert_eq!(n31.len(), 32);
        let s31 = string_comdat_name(n31).unwrap();
        assert!(s31.ends_with("?$AA@"), "31 chars + NUL keeps the NUL escape: {s31}");
        // 32 source characters = 33 bytes → the NUL is DROPPED.
        let n32 = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\0";
        assert_eq!(n32.len(), 33);
        let s32 = string_comdat_name(n32).unwrap();
        assert!(!s32.ends_with("?$AA@"), "32 chars + NUL drops the NUL escape: {s32}");
        assert!(s32.ends_with("aaaa@"));
        // A 30-character all-`/` literal escapes to 2 characters each — 62
        // escaped characters from 31 source bytes, and NOTHING is cut. Reading
        // the limit as an output-character budget truncates this one mid-name.
        let slashes = b"//////////////////////////////\0";
        assert_eq!(slashes.len(), 31);
        let s = string_comdat_name(slashes).unwrap();
        assert_eq!(s.matches("?1").count(), 30, "all 30 slashes must survive: {s}");
        assert!(s.ends_with("?$AA@"), "and so must the NUL: {s}");
    }

    /// **(a) Section order**, with `.bss` then `.CRT$XCU` always last (§4.1).
    /// At `/O1` `.text$yc` precedes `.rdata`; at `/Ox` it is the other way round
    /// and the obj is a different shape entirely, which is why the grading flags
    /// matter (§7.3 caveat 1). Prereg P1 got the code section's *name* wrong —
    /// it is `.text$yc`, not `.text`.
    #[test]
    fn dyninit_section_order_puts_bss_and_crt_xcu_last() {
        let obj = fixture_obj();
        let names: Vec<String> = sections_of(&obj).into_iter().map(|s| s.0).collect();
        assert_eq!(
            names,
            vec![
                ".drectve", ".debug$S", ".XBLD$W", ".XBLD$W", ".text$yc", ".rdata", ".bss",
                ".CRT$XCU"
            ],
            "(a) the eight sections, in order"
        );
        let ix = |n: &str| names.iter().rposition(|s| s == n).unwrap();
        assert!(
            ix(".text$yc") < ix(".rdata") && ix(".rdata") < ix(".bss") && ix(".bss") < ix(".CRT$XCU"),
            "(a) .text$yc < .rdata < .bss < .CRT$XCU"
        );
        assert_eq!(ix(".CRT$XCU"), names.len() - 1, "(a) .CRT$XCU is last");
        assert_eq!(ix(".bss"), names.len() - 2, "(a) .bss is second to last");
        // Characteristics, per §2.1/§4.2: ALIGN_4 `.rdata` (n=4, t=1) and
        // ALIGN_1 `.bss` (n=1, t=1).
        let ch: Vec<u32> = sections_of(&obj).into_iter().map(|s| s.6).collect();
        assert_eq!(ch[4], 0x6040_1020, "(a) .text$yc characteristics");
        assert_eq!(ch[5], 0x4030_1040, "(a) .rdata characteristics, ALIGN_4");
        assert_eq!(ch[6], 0xC010_0080, "(a) .bss characteristics, ALIGN_1");
        assert_eq!(ch[7], 0xC030_0040, "(a) .CRT$XCU characteristics");
    }

    /// **(b) The undefined external constructor sits at index 14** — inside the
    /// `.text$yc` group and *before* the `.rdata` section symbol at 15.
    ///
    /// This is the ordering rule of §3.1 (the symbol table follows section
    /// order; per section, the section symbol + aux, then what it defines, then
    /// any undefined external it is the first to reference), and it is **not**
    /// where either pre-existing emitter puts an undefined external — both put
    /// callees after the defining function with no interleaved section group to
    /// get wrong. Placing the constructor after the `.rdata` group instead
    /// shifts three symbol indices, which every relocation would still resolve
    /// against: a wrong obj no linker complains about, this file's recorded
    /// defect class.
    #[test]
    fn the_undefined_constructor_sits_inside_the_text_yc_group_at_index_fourteen() {
        let obj = fixture_obj();
        let syms = symbols_full(&obj);
        // Flatten to raw record indices so "index 14" means the COFF index.
        let mut at: Vec<String> = Vec::new();
        for (s, aux) in &syms {
            at.push(s.0.clone());
            if aux.is_some() {
                at.push(format!("<aux of {}>", s.0));
            }
        }
        assert_eq!(at.len(), 24, "(b) 24 symbol records");
        assert_eq!(at[11], ".text$yc", "(b) the .text$yc section symbol is at 11");
        assert_eq!(at[13], "??__EsL@@YAXXZ", "(b) the thunk is at 13");
        assert_eq!(
            at[14], "??0L@@QAA@PBDH@Z",
            "(b) the undefined external constructor is at 14, inside the \
             .text$yc group"
        );
        assert_eq!(at[15], ".rdata", "(b) and BEFORE the .rdata section symbol at 15");
        // The constructor really is undefined and really is a function.
        let ctor = syms.iter().find(|(s, _)| s.0 == "??0L@@QAA@PBDH@Z").unwrap();
        assert_eq!(
            (ctor.0 .2, ctor.0 .3, ctor.0 .4),
            (0, 0x0020, 2),
            "(b) the constructor is SectionNumber 0, Type 0x0020, EXTERNAL"
        );
    }

    /// **(c) The relocation record order on `.text$yc`.**
    ///
    /// Nine records: the HI block (VA 0, 4) entirely before the LO block
    /// (VA 8, 12) — the halves are **not** adjacent — a PAIR after every REFHI
    /// *and* every REFLO with `SymbolTableIndex` 0, and **no** PAIR after the
    /// REL24. Prereg P5 predicted 5, and its registered alternative "7, a PAIR
    /// after each REFHI" was wrong too.
    ///
    /// The block separation is asserted as a property, not as fixed positions:
    /// §3.2's `L(float)` row is a cell where the HI and LO blocks name their
    /// symbols in *different* orders, so the emitter derives this by sorting on
    /// offset and the test checks the sorted consequence.
    #[test]
    fn the_dyninit_relocations_pair_both_halves_and_leave_rel24_bare() {
        let obj = fixture_obj();
        let recs = relocations_of(&obj, ".text$yc");
        assert_eq!(recs.len(), 9, "(c) nine .text$yc relocation records");
        assert_eq!(
            recs,
            vec![
                (0x00, 17, REL_PPC_REFHI),
                (0x00, 0, REL_PPC_PAIR),
                (0x04, 20, REL_PPC_REFHI),
                (0x04, 0, REL_PPC_PAIR),
                (0x08, 17, REL_PPC_REFLO),
                (0x08, 0, REL_PPC_PAIR),
                (0x0c, 20, REL_PPC_REFLO),
                (0x0c, 0, REL_PPC_PAIR),
                (0x14, 14, REL_PPC_REL24),
            ],
            "(c) the nine records, transcribed from the reference obj"
        );
        // The same facts as properties, so a future cell with a different
        // symbol order inside a block still grades.
        let hi: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFHI).map(|r| r.0).collect();
        let lo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert!(
            hi.iter().max() < lo.iter().min(),
            "(c) the whole HI block precedes the whole LO block: {hi:?} then {lo:?}"
        );
        for (i, r) in recs.iter().enumerate() {
            if r.2 == REL_PPC_REFHI || r.2 == REL_PPC_REFLO {
                let p = recs[i + 1];
                assert_eq!(
                    (p.0, p.1, p.2),
                    (r.0, 0, REL_PPC_PAIR),
                    "(c) record {i} must be followed by a PAIR at its own VA against symbol 0"
                );
            }
            if r.2 == REL_PPC_REL24 {
                assert!(
                    recs.get(i + 1).map(|n| n.2) != Some(REL_PPC_PAIR),
                    "(c) REL24 takes no PAIR"
                );
            }
        }
        // `.CRT$XCU`: one ADDR32 at offset 0 against the thunk at 13 (§3.4).
        assert_eq!(
            relocations_of(&obj, ".CRT$XCU"),
            vec![(0, 13, REL_PPC_ADDR32)],
            "(c) .CRT$XCU carries one ADDR32 -> the thunk — the pre-existing \
             emitters assume only .text carries relocations, and here that is false"
        );
    }

    /// **(d) The `.bss` inversion** — prereg P8, refuted, and the single most
    /// likely wrong-bytes trap in this shape.
    ///
    /// `SizeOfRawData` carries the object's size, `VirtualSize` is 0,
    /// `PointerToRawData` is 0, the aux `Length` is the size and the aux
    /// `Selection` is 0 (never a COMDAT) — **and the section contributes zero
    /// bytes to the file**, so `.rdata` and `.CRT$XCU` are contiguous across it.
    /// The natural implementation puts the size in `VirtualSize`, and every
    /// other emitter in this file equates "the section's length" with
    /// `raw.len()` in four separate places.
    #[test]
    fn the_bss_section_declares_its_size_but_occupies_no_file_bytes() {
        let obj = fixture_obj();
        let secs = sections_of(&obj);
        let (name, size, ptr_raw, ptr_rel, n_rel, vsize, _ch) = secs[6].clone();
        assert_eq!(name, ".bss");
        assert_eq!(size, 1, "(d) SizeOfRawData carries `sizeof`");
        assert_eq!(vsize, 0, "(d) VirtualSize is 0 — the P8 inversion");
        assert_eq!(ptr_raw, 0, "(d) PointerToRawData is 0");
        assert_eq!((ptr_rel, n_rel), (0, 0), "(d) .bss has no relocations");
        // The aux record.
        let bss_aux = symbols_full(&obj)
            .into_iter()
            .find(|(s, a)| s.0 == ".bss" && a.is_some())
            .and_then(|(_, a)| a)
            .expect("(d) .bss has a section symbol with one aux");
        assert_eq!(bss_aux.0, 1, "(d) aux Length is the object size");
        assert_eq!(bss_aux.1, 0, "(d) aux nReloc");
        assert_eq!(bss_aux.2, 0, "(d) aux CheckSum is 0 for .bss");
        assert_eq!(bss_aux.4, 0, "(d) aux Selection 0 — .bss is NEVER a COMDAT");
        // **Zero file bytes.** `.CRT$XCU` starts exactly where `.rdata`'s own
        // relocations would end — here `.rdata` has none, so immediately after
        // `.rdata`'s raw data, with no gap for `.bss`.
        let text = &secs[4];
        let rdata = &secs[5];
        let crt = &secs[7];
        assert_eq!(
            text.3,
            text.2 + text.1,
            "(d) .text$yc relocations follow its own raw data"
        );
        assert_eq!(
            rdata.2,
            text.3 + 9 * RELOC_LEN as u32,
            "(d) .rdata follows .text$yc's nine relocation records"
        );
        assert_eq!(
            crt.2,
            rdata.2 + rdata.1,
            "(d) .CRT$XCU follows .rdata with NO gap — .bss contributed nothing"
        );
        // A larger object moves only the declared size, never the file layout.
        let big = {
            let lit = StringLiteral { bytes: b"abc\0" };
            let name = string_comdat_name(lit.bytes).unwrap();
            let thunk = DynInitThunk {
                name: "??__EsL@@YAXXZ",
                text: &DYNINIT_TEXT,
                calls: vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }],
                data_refs: vec![
                    DataRef { hi_off: 0x00, lo_off: 0x08, name: &name },
                    DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL" },
                ],
            };
            let object = BssObject {
                symbol: "sL",
                size: 0x1000,
                natural_align: 4,
                external: false,
                initializer_symbol: "sL$initializer$",
            };
            emit_dyninit_obj(r"Z:\tmp\anat\mvp.obj", &thunk, Some(&lit), &object).unwrap()
        };
        assert_eq!(
            big.len(),
            obj.len(),
            "(d) a 0x1000-byte object must not add a single byte to the file"
        );
        let bs = sections_of(&big);
        assert_eq!(bs[6].1, 0x1000, "(d) …only SizeOfRawData moves");
        assert_eq!(bs[6].6, 0xC040_0080, "(d) …and the alignment nibble, to ALIGN_8");
        assert_eq!(bs[7].2, crt.2, "(d) .CRT$XCU stays at the same file offset");
    }

    /// The whole reference cell, all 24 symbol records and both aux fields that
    /// vary, against `docs/OBJ_DYNINIT_SHAPE.md` §3.1's table.
    ///
    /// Storage classes are the part that is easy to get backwards and the part
    /// the workload TUs discriminate: the thunk is **STATIC** with `Type`
    /// 0x0020 even though an ordinary function is EXTERNAL; the string COMDAT's
    /// defining symbol is **EXTERNAL** with `Type` 0 so the linker can fold it;
    /// a `static` object's `.bss` symbol is STATIC and undecorated while a
    /// non-`static` one is EXTERNAL and decorated; `<name>$initializer$` is
    /// STATIC and undecorated either way.
    #[test]
    fn the_dyninit_symbol_table_is_the_reference_cells_twenty_four_records() {
        let obj = fixture_obj();
        // Header.
        assert_eq!(u16::from_le_bytes([obj[0], obj[1]]), MACHINE_POWERPCBE);
        assert_eq!(u16::from_le_bytes([obj[2], obj[3]]), 8, "8 sections");
        assert_eq!(u32::from_le_bytes([obj[4], obj[5], obj[6], obj[7]]), 0, "TimeDateStamp 0");
        assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 24, "24 symbols");
        assert_eq!(u16::from_le_bytes([obj[16], obj[17]]), 0, "SizeOfOptionalHeader");
        assert_eq!(u16::from_le_bytes([obj[18], obj[19]]), CHARACTERISTICS);

        let str_name = string_comdat_name(b"abc\0").unwrap();
        // (name, Value, Sec, Type, StorageClass, nAux)
        let want: Vec<(&str, u32, i16, u16, u8, u8)> = vec![
            ("@comp.id", COMP_ID_VALUE, -1, 0, 3, 0),
            (".drectve", 0, 1, 0, 3, 1),
            (".debug$S", 0, 2, 0, 3, 1),
            (".XBLD$W", 0, 3, 0, 3, 1),
            ("__C2_11886", 0, 3, 0, 2, 0),
            (".XBLD$W", 0, 4, 0, 3, 1),
            ("__C1_11886", 0, 4, 0, 2, 0),
            (".text$yc", 0, 5, 0, 3, 1),
            ("??__EsL@@YAXXZ", 0, 5, 0x0020, 3, 0),
            ("??0L@@QAA@PBDH@Z", 0, 0, 0x0020, 2, 0),
            (".rdata", 0, 6, 0, 3, 1),
            (&str_name, 0, 6, 0x0000, 2, 0),
            (".bss", 0, 7, 0, 3, 1),
            ("sL", 0, 7, 0x0000, 3, 0),
            (".CRT$XCU", 0, 8, 0, 3, 1),
            ("sL$initializer$", 0, 8, 0x0000, 3, 0),
        ];
        let got = symbols_full(&obj);
        let got_hdr: Vec<(&str, u32, i16, u16, u8, u8)> =
            got.iter().map(|(s, _)| (s.0.as_str(), s.1, s.2, s.3, s.4, s.5)).collect();
        assert_eq!(got_hdr, want, "the 16 non-aux symbol records");

        // The aux records that carry something other than zeros:
        // (Length, nReloc, CheckSum, Number, Selection).
        let aux = |n: &str, k: usize| {
            got.iter().filter(|(s, _)| s.0 == n).nth(k).and_then(|(_, a)| *a).unwrap()
        };
        assert_eq!(aux(".drectve", 0), (132, 0, 0, 0, 0));
        assert_eq!(aux(".XBLD$W", 0), (16, 0, XBLD_C2_CHECKSUM, 0, 2));
        assert_eq!(aux(".XBLD$W", 1), (16, 0, XBLD_C1_CHECKSUM, 0, 2));
        assert_eq!(
            aux(".text$yc", 0),
            (0x18, 9, 0, 0, 2),
            ".text$yc: 9 relocations, CheckSum 0, Selection 2 ANY (not 1 \
             NODUPLICATES — that is an ORDINARY function's .text)"
        );
        assert_eq!(
            aux(".rdata", 0),
            (4, 0, 0x8619_B74C, 0, 2),
            ".rdata: a STRING literal COMDAT carries the real CRC — an \
             FP-constant one carries 0"
        );
        assert_eq!(aux(".bss", 0), (1, 0, 0, 0, 0));
        assert_eq!(aux(".CRT$XCU", 0), (4, 1, 0, 0, 0));

        // The string table: six long names, in first-use order, 100 bytes.
        let symtab = u32::from_le_bytes([obj[8], obj[9], obj[10], obj[11]]) as usize;
        let st = symtab + 24 * SYMBOL_LEN;
        let st_size = u32::from_le_bytes([obj[st], obj[st + 1], obj[st + 2], obj[st + 3]]);
        assert_eq!(st_size as usize, obj.len() - st);
        assert_eq!(
            st_size, 100,
            "the reference cell's string table is 100 bytes: __C2_11886, \
             __C1_11886, ??__EsL@@YAXXZ, ??0L@@QAA@PBDH@Z, {str_name}, \
             sL$initializer$ — `sL` and `.text$yc` are <= 8 chars and go inline"
        );

        // The total obj size is **`-Fo`-path dependent** and must not be
        // hardcoded: `.debug$S` embeds the output path in its S_OBJNAME record
        // and measured 0x94 in the probes against 0xac in the workload TUs. So
        // the pin is the path-independent remainder, and the doc's 1,316-byte
        // reference cell is then a consequence of its 148-byte `.debug$S`.
        let debug_s_len = build_debug_s(r"Z:\tmp\anat\mvp.obj").len();
        assert_eq!(
            obj.len(),
            1168 + debug_s_len,
            "everything but `.debug$S` is 1,168 bytes for this cell"
        );
        assert_eq!(1168 + 148, 1316, "…so the reference cell's 0x94 `.debug$S` gives 1,316 B");
    }

    /// The two real workload TUs, `TomCryptLicense.cpp` and `ZlibLicense.cpp`
    /// (§7.2) — the only structural difference between them is the object
    /// symbol's linkage, and the string table size is a whole-obj consequence of
    /// the COMDAT name rule that nothing here was fitted to.
    #[test]
    fn the_two_workload_tus_differ_only_in_the_objects_linkage() {
        let cell = |lit: &'static [u8], sym: &'static str, ctor: &'static str, external: bool| {
            let name = string_comdat_name(lit).unwrap();
            let thunk = DynInitThunk {
                name: "??__EsLicense@@YAXXZ",
                text: &DYNINIT_TEXT,
                calls: vec![Call { reloc_offset: 0x14, callee: ctor }],
                data_refs: vec![
                    DataRef { hi_off: 0x00, lo_off: 0x08, name: &name },
                    DataRef { hi_off: 0x04, lo_off: 0x0c, name: sym },
                ],
            };
            let object = BssObject {
                symbol: sym,
                size: 0xc,
                natural_align: 4,
                external,
                initializer_symbol: "sLicense$initializer$",
            };
            emit_dyninit_obj(
                r"Z:\t\x.obj",
                &thunk,
                Some(&StringLiteral { bytes: lit }),
                &object,
            )
            .expect("both workload TUs are in class")
        };
        let ctor = "??0Licenses@@QAA@PBDW4Requirement@0@@Z";
        let tomcrypt = cell(b"system/src/synth/tomcrypt\0", "sLicense", ctor, false);
        let zlib = cell(b"system/src/zlib\0", "?sLicense@@3VLicenses@@A", ctor, true);

        for (tag, obj, rdata_size, class, obj_sym) in [
            ("tomcrypt", &tomcrypt, 0x1au32, 3u8, "sLicense"),
            ("zlib", &zlib, 0x10, 2, "?sLicense@@3VLicenses@@A"),
        ] {
            let secs = sections_of(obj);
            assert_eq!(secs.len(), 8, "{tag}: 8 sections");
            assert_eq!(secs[5].1, rdata_size, "{tag}: .rdata size");
            assert_eq!(secs[5].6, 0x4030_1040, "{tag}: .rdata ALIGN_4");
            assert_eq!(secs[6].1, 0xc, "{tag}: .bss size");
            assert_eq!(secs[6].6, 0xC030_0080, "{tag}: .bss ALIGN_4");
            assert_eq!(u32::from_le_bytes([obj[12], obj[13], obj[14], obj[15]]), 24);
            let syms = symbols_full(obj);
            // By exact name: `??__EsLicense@@YAXXZ` also *contains* the object's
            // spelling and sits earlier in the table, so a substring match here
            // grades the thunk instead and reads STATIC in both cells — a test
            // that passes for the wrong reason on the row that matters.
            let (o, _) = syms.iter().find(|(s, _)| s.0 == obj_sym).unwrap();
            assert_eq!(o.4, class, "{tag}: the object symbol's storage class");
            assert_eq!(o.2, 7, "{tag}: the object lives in .bss");
            // The thunk stays STATIC in BOTH — the object's linkage does not
            // move it (§4.3). ZlibLicense.cpp confirms both halves at once.
            let (t, _) = syms.iter().find(|(s, _)| s.0 == "??__EsLicense@@YAXXZ").unwrap();
            assert_eq!((t.3, t.4), (0x0020, 3), "{tag}: the thunk is STATIC of FUNCTION type");
            let (init, _) =
                syms.iter().find(|(s, _)| s.0 == "sLicense$initializer$").unwrap();
            assert_eq!((init.2, init.3, init.4), (8, 0, 3), "{tag}: $initializer$ is STATIC in .CRT$XCU");
        }
        // The string tables, whose sizes are a byte-level consequence of the
        // COMDAT-name rule and were transcribed from the reference objs.
        let st_size = |obj: &[u8]| {
            let symtab = u32::from_le_bytes([obj[8], obj[9], obj[10], obj[11]]) as usize;
            let st = symtab + 24 * SYMBOL_LEN;
            u32::from_le_bytes([obj[st], obj[st + 1], obj[st + 2], obj[st + 3]])
        };
        assert_eq!(
            st_size(&tomcrypt),
            161,
            "TomCrypt: 6 entries — `sLicense` is exactly 8 chars and goes INLINE"
        );
        assert_eq!(
            st_size(&zlib),
            175,
            "Zlib: 7 entries — the decorated ?sLicense@@3VLicenses@@A is interned \
             before sLicense$initializer$"
        );
    }

    /// The class boundary, stated as refusals rather than as a comment. Each of
    /// these is a shape `docs/OBJ_DYNINIT_SHAPE.md` measured to be *different*
    /// or never measured at all, and an honest `None` is the required answer.
    #[test]
    fn emit_dyninit_obj_declines_everything_outside_the_measured_class() {
        let lit = StringLiteral { bytes: b"abc\0" };
        let name = string_comdat_name(lit.bytes).unwrap();
        let ok_refs = || {
            vec![
                DataRef { hi_off: 0x00, lo_off: 0x08, name: &name },
                DataRef { hi_off: 0x04, lo_off: 0x0c, name: "sL" },
            ]
        };
        let object = |size: u32, align: u32| BssObject {
            symbol: "sL",
            size,
            natural_align: align,
            external: false,
            initializer_symbol: "sL$initializer$",
        };
        let go = |t: DynInitThunk, l: Option<&StringLiteral>, o: BssObject| {
            emit_dyninit_obj(r"Z:\t\x.obj", &t, l, &o).is_some()
        };
        fn base<'a>(calls: Vec<Call<'a>>, refs: Vec<DataRef<'a>>) -> DynInitThunk<'a> {
            DynInitThunk {
                name: "??__EsL@@YAXXZ",
                text: &DYNINIT_TEXT,
                calls,
                data_refs: refs,
            }
        }
        let one_call = || vec![Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" }];

        assert!(go(base(one_call(), ok_refs()), Some(&lit), object(1, 1)), "the reference cell is in class");
        // No call, or two: a different body — the destructor shape is framed
        // with 14 relocations and a `bl atexit` (§4.4).
        assert!(!go(base(vec![], ok_refs()), Some(&lit), object(1, 1)), "zero calls");
        assert!(
            !go(
                base(
                    vec![
                        Call { reloc_offset: 0x14, callee: "??0L@@QAA@PBDH@Z" },
                        Call { reloc_offset: 0x18, callee: "atexit" },
                    ],
                    ok_refs()
                ),
                Some(&lit),
                object(1, 1)
            ),
            "two calls — that is the destructor shape"
        );
        // A quad against a symbol that is neither the literal nor the object.
        assert!(
            !go(
                base(
                    one_call(),
                    vec![
                        DataRef { hi_off: 0, lo_off: 8, name: &name },
                        DataRef { hi_off: 4, lo_off: 12, name: "?other@@3HA" },
                    ]
                ),
                Some(&lit),
                object(1, 1)
            ),
            "an unrelated data symbol"
        );
        // A literal present but never referenced, or referenced twice.
        assert!(
            !go(
                base(one_call(), vec![DataRef { hi_off: 4, lo_off: 12, name: "sL" }]),
                Some(&lit),
                object(1, 1)
            ),
            "a literal with no reference to it"
        );
        // A zero-sized object, and an alignment that is not 1/2/4/8.
        assert!(!go(base(one_call(), ok_refs()), Some(&lit), object(0, 1)), "sizeof 0");
        assert!(!go(base(one_call(), ok_refs()), Some(&lit), object(1, 3)), "align 3");
        // A `.text` that is not a whole number of instructions.
        assert!(
            emit_dyninit_obj(
                r"Z:\t\x.obj",
                &DynInitThunk {
                    name: "??__EsL@@YAXXZ",
                    text: &[0, 1, 2],
                    calls: one_call(),
                    data_refs: ok_refs(),
                },
                Some(&lit),
                &object(1, 1)
            )
            .is_none(),
            "a 3-byte .text"
        );
        // The literal-free cell IS in class (§3.2's `L(int)` row: one address
        // operand, five relocations) — and it is a 7-section, 21-symbol obj, so
        // nothing here may assume 8 and 24.
        let no_lit = emit_dyninit_obj(
            r"Z:\t\x.obj",
            &base(one_call(), vec![DataRef { hi_off: 0, lo_off: 4, name: "sL" }]),
            None,
            &object(1, 1),
        )
        .expect("a constructor with no string argument is in class");
        assert_eq!(u16::from_le_bytes([no_lit[2], no_lit[3]]), 7, "no .rdata section");
        assert_eq!(
            u32::from_le_bytes([no_lit[12], no_lit[13], no_lit[14], no_lit[15]]),
            21,
            "24 minus the .rdata section symbol, its aux and the literal"
        );
        assert_eq!(
            relocations_of(&no_lit, ".text$yc").len(),
            5,
            "one quad plus the REL24"
        );
    }

    /// The alignment rule (§4.2), both sides, at every measured threshold.
    #[test]
    fn the_alignment_nibble_rule_matches_both_measured_columns() {
        // n = 1 -> ALIGN_1; 2..63 -> ALIGN_4; >= 64 -> ALIGN_8, then `max` with
        // the natural alignment.
        for (n, t, want) in [
            (1u32, 1u32, 1u32),
            (2, 1, 3),
            (3, 1, 3),
            (63, 1, 3),
            (64, 1, 4),
            (256, 1, 4),
            // `t` moves independently: a `double` member is ALIGN_8 at n = 8
            // where a `char[8]` is ALIGN_4.
            (8, 8, 4),
            (8, 1, 3),
            (1, 2, 2),
            (4, 8, 4),
        ] {
            assert_eq!(align_nibble(n, t), Some(want), "n={n}, t={t}");
        }
        assert_eq!(align_nibble(1, 3), None, "a non-power-of-two alignment is refused");
        assert_eq!(align_nibble(1, 16), None, "ALIGN_16 was never measured here");
    }

    /// The **negative half of the same rule**: a pooled FP constant's halves
    /// *are* adjacent (`addis` then `lfs`, four bytes apart), and that is why
    /// `hi_off + 4` looked right. Pinning it here is what stops a future
    /// "unify the two quad emitters" refactor from fixing one by breaking the
    /// other — the two quads are genuinely different and this says so portably.
    ///
    /// Packed, not `/Gy`: [`emit_comdat_obj`] carries no constant-pool code at
    /// all, because `PortC2::build` refuses a pooled constant under `/Gy`
    /// (`docs/OBJ_GY_SHAPES.md` §2, the reverse-append ordering) and hardcodes
    /// `fp_refs: Vec::new()` on that path. Writing this test against the COMDAT
    /// emitter read **0 relocation records** and would have been the vacuous
    /// shape — a control run where the effect cannot appear.
    #[test]
    fn the_pooled_fp_constant_quad_is_adjacent_which_is_why_the_data_one_looked_it() {
        let text = vec![0u8; 12];
        let f = Function {
            is_float: true,
            fp_refs: vec![crate::codegen::FpConstRef {
                hi_off: 0,
                bits: 0x3FF0_0000_0000_0000,
                double: false,
            }],
            ..Function::plain("?fc@@YAMXZ", 0)
        };
        let obj = emit_obj(r"Z:\t\fc.obj", &[f], &text, 2536);
        let recs = text_relocations(&obj);
        assert_eq!(
            recs.len(),
            4,
            "(i) a single pooled FP constant is one quad = 4 records, got {}",
            recs.len()
        );
        let reflo: Vec<u32> = recs.iter().filter(|r| r.2 == REL_PPC_REFLO).map(|r| r.0).collect();
        assert_eq!(
            reflo,
            vec![4],
            "(j) the FP quad's halves ARE adjacent — REFLO belongs at hi_off+4 = \
             4 here, and the data-symbol quad's does NOT: {reflo:?}"
        );
    }
}
