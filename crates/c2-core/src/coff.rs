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
}

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
    /// True iff this function's body does floating-point arithmetic. The obj
    /// then carries an undefined external `_fltused`, emitted immediately after
    /// the FIRST such function's symbol group — the CRT's float-support hook.
    /// Verified: a pure FP leaf changes the obj shell by exactly this one
    /// symbol (`docs/CODEGEN_W13_FLOAT.md` §4).
    pub is_float: bool,
    /// W13b: this function's floating-point constant reference sites, in
    /// emission order, with `hi_off` already rebased to the whole `.text`.
    pub fp_refs: Vec<crate::codegen::FpConstRef>,
    /// `Some` iff this function establishes a stack frame, carrying the two
    /// lengths its `.pdata` record and its two `$M` labels need. `None` for a
    /// leaf — c2 emits no unwind record for one, so this field alone decides
    /// whether the obj has a `.pdata` section at all.
    pub frame: Option<Frame>,
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
            frame: None,
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
/// for every function class: a comparison leaf (`a < b`) consumes 3, a
/// floating-point leaf 2, and each pooled FP constant a further 2. Those are
/// refused upstream ([`crate::PortC2::build`]) rather than modeled, because a
/// wrong stride is a wrong `$M` number and a wrong `$M` number is a wrong-bytes
/// obj — the whole point of the counter.
pub fn plan_labels(counter: u32, funcs: &[Function], comdat: bool) -> Vec<Option<[u32; 3]>> {
    let mut cur = counter + LABEL_SEED_GAP;
    if comdat {
        cur += 3 * funcs.len() as u32;
    }
    funcs
        .iter()
        .map(|f| match f.frame {
            Some(_) => {
                let n = cur;
                cur += if comdat { 5 } else { 4 };
                Some([n, n + 1, n + 2])
            }
            None => {
                cur += 1;
                None
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
    let sections = [
        Section {
            name: ".drectve",
            characteristics: CH_DRECTVE,
            raw: std::borrow::Cow::Borrowed(DRECTVE),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
        Section {
            name: ".debug$S",
            characteristics: CH_DEBUGS,
            raw: std::borrow::Cow::Owned(build_debug_s(obj_name)),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C2,
            raw: std::borrow::Cow::Borrowed(&XBLD_C2),
            checksum: XBLD_C2_CHECKSUM,
            selection: 2,
            assoc: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C1,
            raw: std::borrow::Cow::Borrowed(&XBLD_C1),
            checksum: XBLD_C1_CHECKSUM,
            selection: 2,
            assoc: 0,
        },
    ];
    let n_sections = sections.len();

    let raw_base = COFF_HEADER_LEN + n_sections * SECTION_HEADER_LEN;
    let mut ptrs = Vec::with_capacity(n_sections);
    let mut cursor = raw_base;
    for s in &sections {
        ptrs.push(cursor);
        cursor += s.raw.len();
    }
    // No relocations, so the symbol table follows the raw data directly.
    let ptr_symtab = cursor;
    // 1 (@comp.id) + 4 section symbols x 2 (symbol + aux) + 2 watermark externs.
    const N_SYMBOLS: u32 = 11;

    let mut b = Buf::with_capacity(ptr_symtab + N_SYMBOLS as usize * SYMBOL_LEN + 512);
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp — normalized away by the compare
    b.u32(ptr_symtab as u32);
    b.u32(N_SYMBOLS);
    b.u16(0); // SizeOfOptionalHeader
    b.u16(CHARACTERISTICS);

    for (i, s) in sections.iter().enumerate() {
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.raw.len() as u32);
        b.u32(ptrs[i] as u32);
        b.u32(0); // PointerToRelocations — none
        b.u32(0); // PointerToLinenumbers
        b.u16(0); // NumberOfRelocations
        b.u16(0); // NumberOfLinenumbers
        b.u32(s.characteristics);
    }

    for s in &sections {
        b.bytes(&s.raw);
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    let mut strtab = StringTable::new();
    // slot 0: @comp.id (ABS, STATIC, no aux)
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1); // IMAGE_SYM_ABSOLUTE
    b.u16(0x0000);
    b.u8(3); // STATIC
    b.u8(0);

    emit_section_symbol(&mut b, &sections[0], 1, 0); // slot 1/2  .drectve
    emit_section_symbol(&mut b, &sections[1], 2, 0); // slot 3/4  .debug$S
    emit_section_symbol(&mut b, &sections[2], 3, 0); // slot 5/6  .XBLD$W C2
    emit_external_symbol(&mut b, &mut strtab, NAME_C2, 3, 0x0000); // slot 7
    emit_section_symbol(&mut b, &sections[3], 4, 0); // slot 8/9  .XBLD$W C1
    emit_external_symbol(&mut b, &mut strtab, NAME_C1, 4, 0x0000); // slot 10

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
            frame: None,
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
            frame: None,
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

    let mut sections: Vec<Section> = vec![
        Section {
            name: ".drectve",
            characteristics: CH_DRECTVE,
            raw: std::borrow::Cow::Borrowed(DRECTVE),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
        Section {
            name: ".debug$S",
            characteristics: CH_DEBUGS,
            raw: std::borrow::Cow::Owned(build_debug_s(obj_name)),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C2,
            raw: std::borrow::Cow::Borrowed(&XBLD_C2),
            checksum: XBLD_C2_CHECKSUM,
            selection: 2,
            assoc: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C1,
            raw: std::borrow::Cow::Borrowed(&XBLD_C1),
            checksum: XBLD_C1_CHECKSUM,
            selection: 2,
            assoc: 0,
        },
    ];
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
            SectionOwner::Text(k) => funcs[*k].calls.len() as u16,
            SectionOwner::Pdata(_) => 1,
            SectionOwner::Fixed => 0,
        })
        .collect();
    let raw_base = COFF_HEADER_LEN + n_sections * SECTION_HEADER_LEN;
    let mut ptrs = Vec::with_capacity(n_sections);
    let mut reloc_ptr: Vec<Option<usize>> = vec![None; n_sections];
    let mut cursor = raw_base;
    for (i, s) in sections.iter().enumerate() {
        ptrs.push(cursor);
        cursor += s.raw.len();
        if n_reloc_of[i] > 0 {
            reloc_ptr[i] = Some(cursor);
            cursor += n_reloc_of[i] as usize * RELOC_LEN;
        }
    }
    let ptr_symtab = cursor;

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
    let mut next_idx: u32 = 11;
    // The callee symbols this function emits, in emission order (reverse
    // first-reference), each with the index it lands at.
    let mut introduced: Vec<Vec<(&str, u32)>> = Vec::with_capacity(funcs.len());
    let mut fn_idx: Vec<u32> = Vec::with_capacity(funcs.len());
    let mut callee_syms: Vec<(&str, u32)> = Vec::new();
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
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp — normalized away
    b.u32(ptr_symtab as u32);
    b.u32(n_symbols);
    b.u16(0);
    b.u16(CHARACTERISTICS);

    for (i, s) in sections.iter().enumerate() {
        b.name8(s.name);
        b.u32(0); // VirtualSize
        b.u32(0); // VirtualAddress
        b.u32(s.raw.len() as u32);
        b.u32(ptrs[i] as u32);
        b.u32(reloc_ptr[i].unwrap_or(0) as u32);
        b.u32(0); // PointerToLinenumbers
        b.u16(n_reloc_of[i]);
        b.u16(0);
        b.u32(s.characteristics);
    }

    // Interleaved to match the layout computed above: each section's raw data,
    // then its own relocations.
    for (i, s) in sections.iter().enumerate() {
        debug_assert_eq!(b.0.len(), ptrs[i]);
        b.bytes(&s.raw);
        match owner[i] {
            SectionOwner::Text(k) => {
                debug_assert!(
                    funcs[k].calls.is_empty() || b.0.len() == reloc_ptr[i].unwrap()
                );
                // One REL24 per call site, in ascending `.text` offset. Several
                // sites may share one symbol index (the same callee called twice).
                for call in &funcs[k].calls {
                    let ci = callee_syms
                        .iter()
                        .find(|(n, _)| *n == call.callee)
                        .map(|(_, ix)| *ix)
                        .expect("every callee got a symbol");
                    b.u32(call.reloc_offset);
                    b.u32(ci);
                    b.u16(REL_PPC_REL24);
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
    b.name8("@comp.id");
    b.u32(COMP_ID_VALUE);
    b.i16(-1);
    b.u16(0x0000);
    b.u8(3);
    b.u8(0);
    emit_section_symbol(&mut b, &sections[0], 1, 0);
    emit_section_symbol(&mut b, &sections[1], 2, 0);
    emit_section_symbol(&mut b, &sections[2], 3, 0);
    emit_external_symbol(&mut b, &mut strtab, NAME_C2, 3, 0x0000);
    emit_section_symbol(&mut b, &sections[3], 4, 0);
    emit_external_symbol(&mut b, &mut strtab, NAME_C1, 4, 0x0000);

    for (i, f) in funcs.iter().enumerate() {
        let sec_num = (sec_text[i] + 1) as i16;
        emit_section_symbol(&mut b, &sections[sec_text[i]], sec_num, f.calls.len() as u16);
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
    let debug_s = build_debug_s(obj_name);
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
    let mut sections = vec![
        Section {
            name: ".drectve",
            characteristics: CH_DRECTVE,
            raw: std::borrow::Cow::Borrowed(DRECTVE),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
        Section {
            name: ".debug$S",
            characteristics: CH_DEBUGS,
            raw: std::borrow::Cow::Owned(debug_s),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C2,
            raw: std::borrow::Cow::Borrowed(&XBLD_C2),
            checksum: XBLD_C2_CHECKSUM,
            selection: 2,
            assoc: 0,
        },
        Section {
            name: ".XBLD$W",
            characteristics: CH_XBLD_C1,
            raw: std::borrow::Cow::Borrowed(&XBLD_C1),
            checksum: XBLD_C1_CHECKSUM,
            selection: 2,
            assoc: 0,
        },
        Section {
            name: ".text",
            characteristics: CH_TEXT,
            raw: std::borrow::Cow::Borrowed(text),
            checksum: 0,
            selection: 0,
            assoc: 0,
        },
    ];
    let text_idx = sections.len() - 1;
    for &(bits, double) in &pool {
        sections.push(Section {
            name: ".rdata",
            characteristics: if double { CH_RDATA_F64 } else { CH_RDATA_F32 },
            raw: std::borrow::Cow::Owned(real_raw_bytes(bits, double)),
            checksum: 0,
            selection: 2,
            assoc: 0,
        });
    }
    // `.pdata` last. A TU with BOTH a constant pool and a framed function would
    // settle the `.rdata`/`.pdata` order, and none has been captured — the
    // combination is refused upstream rather than guessed at here.
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
    let mut plan: Vec<(usize, u32, Vec<(&str, u32)>, Vec<usize>)> =
        Vec::with_capacity(funcs.len());
    let mut real_idx: Vec<Option<u32>> = vec![None; pool.len()];
    // An undefined external callee is emitted **once per distinct name**, after the
    // symbol of the function that first calls it — every later call site relocates
    // against that same index. Emitting one per call site instead is invisible
    // until two functions in a TU call the same callee, which no fixture did before
    // `il_call_perm.cpp`; the reference puts `?g3` after `pass3` and nothing after
    // the four later functions that also call it.
    let mut callee_syms: Vec<(&str, u32)> = Vec::new();
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
        plan.push((i, def_idx, new_callees, introduced));
        if fltused_after == Some(i) {
            next_idx += 1;
        }
    }
    let n_symbols: u32 = next_idx;

    // The `.pdata` relocations: one ADDR32 per record, at the record's own
    // offset, against the framed function's defined symbol. In `.text` order,
    // which is also ascending VirtualAddress.
    let mut pdata_relocs: Vec<(u32, u32, u16)> = Vec::new();
    for (i, def, _new, _intro) in &plan {
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
    for (i, _def, _new, _intro) in &plan {
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
    }
    text_relocs.sort_by_key(|&(va, _, _)| va);
    let n_text_reloc = text_relocs.len();

    // Raw data is packed right after the section headers, and a section's
    // relocation records sit immediately after **that section's own** raw data —
    // not after every section's. With `.text` last (no constant pool) the two
    // layouts coincide, which is why this only surfaced once `.rdata` followed
    // `.text`: c2 put the four REFHI/REFLO records between `.text` and the
    // constant pool, the port put them after both.
    let raw_base = COFF_HEADER_LEN + n_sections * SECTION_HEADER_LEN;
    let mut ptrs = Vec::with_capacity(n_sections);
    let mut cursor = raw_base;
    let mut ptr_text_reloc = 0usize;
    let mut ptr_pdata_reloc = 0usize;
    for (i, s) in sections.iter().enumerate() {
        ptrs.push(cursor);
        cursor += s.raw.len();
        if i == text_idx && n_text_reloc > 0 {
            ptr_text_reloc = cursor;
            cursor += n_text_reloc * RELOC_LEN;
        }
        if Some(i) == pdata_idx {
            ptr_pdata_reloc = cursor;
            cursor += pdata_relocs.len() * RELOC_LEN;
        }
    }
    let ptr_symtab = cursor; // symbol table right after the last section's data

    // ---- COFF header (20 bytes) ----
    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    b.u16(MACHINE_POWERPCBE);
    b.u16(n_sections as u16);
    b.u32(0); // TimeDateStamp — normalized away
    b.u32(ptr_symtab as u32);
    b.u32(n_symbols);
    b.u16(0); // SizeOfOptionalHeader
    b.u16(CHARACTERISTICS);

    // ---- section headers (40 bytes each) ----
    // Only `.text` carries relocations in this class (the `.rdata` constant
    // pools are pure data).
    for (i, s) in sections.iter().enumerate() {
        let (prel, nrel) = if i == text_idx && n_text_reloc > 0 {
            (ptr_text_reloc as u32, n_text_reloc as u16)
        } else if Some(i) == pdata_idx {
            (ptr_pdata_reloc as u32, pdata_relocs.len() as u16)
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
    // undefined external callee symbol, then the constant pools this function
    // introduces (`.rdata` section symbol + aux, then the `__real@…` external).
    for (i, _def, new_callees, introduced) in &plan {
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
    b.u16(s.assoc); // Number — 0 unless Selection=5 (ASSOCIATIVE)
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
}
