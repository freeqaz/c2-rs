//! **W-SECT — the `.in` SCALAR-INITIALIZER record.**
//!
//! [`super::inlit`] reads the one `.in` record kind the `??__E` obj needed (the
//! string literal, element tag `03`). This module reads the one a `.data`
//! section needs: the **constant value** a statically-initialized namespace-scope
//! object is given. `.gl` carries the object's name, size, alignment and linkage
//! (`super::gl::gl_data_objects`) and does **not** carry its value; without this
//! reader a `.data` writer has a correct header and no raw bytes.
//!
//! # The record, as measured
//!
//! ```text
//!   <operand token>  00  <element>+  07
//! ```
//!
//! and an element is `<tag> …`, where the tag says what follows:
//!
//! | tag | element | this reader |
//! |---|---|---|
//! | `01` | `<type> <width> <value>` — a scalar constant | **read** |
//! | `02` | `<token> <offset> <n>` — the ADDRESS of another symbol | **read** (board #931, `work/w-tag02`) |
//! | `03` | `<len> <bytes>` — inline bytes | **read** (board #960, `work/w-inread`) |
//! | `08` | `<count>` — a run of `count` ZERO BYTES | **read** (board #960) |
//!
//! and the scalar element's **type** byte is:
//!
//! | type | means | this reader |
//! |---|---|---|
//! | `01` | signed integer | **read** |
//! | `02` | unsigned integer | **read** |
//! | `03` | a DATA pointer whose value is a plain integer | **read** (board #960) |
//! | `04` | a FUNCTION pointer whose value is a plain integer | **read** (board #960) |
//! | `05` | floating point | refused — the aux `CheckSum` exclusion |
//!
//! # What board #960 was, and what it cost
//!
//! Until `w-inread` this reader saw **1,429,596 of 1,885,700** tag-02 symbol
//! addresses (75.81 %) and **518,098 of 879,377** records over the 850-TU
//! workload — fewer on 812 TUs, more on none — because `read_elements` returned
//! `Err` for the **whole record** the moment it met one of the four kinds above.
//! `work/w-emitp2/blindspot.txt` decomposes the loss by the FIRST element each
//! record's parse refuses:
//!
//! ```text
//!   element tag 08 (zero fill)     114,865 records   315,553 symbol addresses
//!   scalar type 03                 132,528 records   132,488
//!   element tag 03 (inline bytes)  144,848 records         0
//!   scalar type 04                   4,841 records     9,675
//!   scalar type 05 (float)             134 records         0
//! ```
//!
//! **The rows are not independent and the order matters**: `??_R0` is
//! `02 <type_info vftable> · 01 03 04 00 · 03 <len> <name>`, so teaching the
//! reader scalar type `03` without also teaching it element tag `03` recovers
//! **none** of that row's 132,488 addresses — the blame merely moves one element
//! to the right. That is why this widening is four kinds and not two.
//!
//! # The value encoding, and the two places it is NOT the crate's other varints
//!
//! MEASURED across fifteen one-axis cells. The value's encoding depends on the
//! element's **width**, which is why neither [`super::readers::read_varint`]
//! (`80` + LE**32**, always) nor [`super::inlit`]'s length varint (`80` +
//! LE**16**, always) can be reused:
//!
//! ```text
//!   char  c2 = (char)200;   01 01 01 · c8            width 1: ONE RAW BYTE
//!   char  c3 = (char)128;   01 01 01 · 80            …including 0x80 itself
//!   char  c4 = 127;         01 01 01 · 7f
//!   short s6 = 127;         01 01 02 · 7f            short form, b0 < 0x80
//!   short s5 = 128;         01 01 02 · 80 8000       escape + LE16
//!   short s7 = -5;          01 01 02 · 80 fbff       NEGATIVES ALWAYS ESCAPE
//!   short s8 = -128;        01 01 02 · 80 80ff
//!   int   i5 = 127;         01 01 04 · 7f
//!   int   i2 = 200;         01 01 04 · 80 c8000000   escape + LE32
//!   int   n1 = -5;          01 01 04 · 80 fbffffff
//!   int   i7 = -1;          01 01 04 · 80 ffffffff
//!   unsigned u1 = 0xFFFFFFFF;  01 02 04 · 80 ffffffff   type 02 = unsigned
//!   bool  bl = true;        01 01 01 · 01
//!   int   a1[2] = {1,2};    01 01 04 01 · 01 01 04 02   TWO elements
//!   double f1 = 1.0;        01 05 08 · 000000000000f03f   type 05, RAW LE
//! ```
//!
//! **Width 1 is the row that makes this a separate function and not a flag on an
//! existing one.** `(char)128` spells its value `80` with no escape, so a reader
//! that treated `80` as a marker at every width would consume the record's
//! terminator as a payload byte and desynchronize. The width is known from the
//! element, so there is no ambiguity — but only if the reader uses it.
//!
//! **Type `05` (floating point) is REFUSED**, deliberately. Its value is raw
//! little-endian bytes rather than a varint, and more importantly
//! `OBJ_DATA_BSS_SHAPE.md` §4.2.1 shows a float's bytes are **omitted from the
//! section's aux `CheckSum`** — so admitting one here would need the CRC
//! exclusion, whose byte-granularity finding that document labels *not
//! pre-registered*.
//!
//! # Endianness — the value is stored LE and emitted BE
//!
//! The `.in` escape payload is **little-endian** and the `.data` section's raw
//! bytes are **big-endian**: `int i1 = 0x11223344;` spells `80 44 33 22 11` in
//! `.in` and the obj carries `11 22 33 44`. This reader returns the **obj's**
//! byte order, because that is the only one its caller can use, and the swap is
//! done in exactly one place.

use super::readers::read_token_var;

/// The byte between the operand token and the first element.
const RECORD_TAG: u8 = 0x00;

/// The element tag this reader handles: a scalar constant.
const ELEMENT_SCALAR: u8 = 0x01;

/// The element tag for **the address of another symbol** — `02 <target-token>
/// <offset> <n>` (board #931, `work/w-tag02/GRAMMAR.md`).
const ELEMENT_SYMBOL_ADDRESS: u8 = 0x02;

/// The `<n>` a tag-`02` element must carry. **OBSERVED constant, not a known
/// one**: every pointer on this 32-bit target is four bytes, so no cell in the
/// 24-cell grid can vary it. Any other value refuses.
const ADDRESS_WIDTH: u8 = 0x04;

/// The byte that closes an initializer record. Shared with [`super::inlit`].
const RECORD_END: u8 = 0x07;

/// Scalar element **type** bytes this reader admits — signed and unsigned
/// integer. `05` is floating point and is refused (see the module docs); every
/// other value is unseen and refuses with it.
const TYPE_INT_SIGNED: u8 = 0x01;
const TYPE_INT_UNSIGNED: u8 = 0x02;

/// **A DATA pointer whose value is a plain integer** — canonically a null
/// pointer *inside an aggregate*, where no relocation can be spelled because
/// there is no target.
///
/// MEASURED on `work/w-inread/grid` (board **#960**):
///
/// ```text
///   struct S{int* p; int a;} s = {0, 5};            01 03 04 · 00
///   struct S{int* p; int a;} s = {(int*)4, 5};      01 03 04 · 04
///   struct S{int* p; int a;} s = {(int*)0x11223344} 01 03 04 · 80 44 33 22 11
/// ```
///
/// and it is what `_TypeDescriptor::spare` is: `??_R0`'s record spells
/// `02 <??_7type_info@@6B@> 00 04 · 01 03 04 00 · 03 08 ".?AUA@@\0"`, which is
/// **14,703 of the workload's 14,705 type-03 elements over the first 40 TUs**.
/// A `.data` object initialized with `(int*)4` carries the four bytes and **no
/// relocation** — `z11_data_ptr_4`'s obj, checked.
const TYPE_PTR_DATA: u8 = 0x03;

/// **A FUNCTION pointer whose value is a plain integer.** The same shape as
/// [`TYPE_PTR_DATA`] and a different type byte, which is the only thing that
/// distinguishes them in the stream.
///
/// MEASURED on `work/w-inread/grid`:
///
/// ```text
///   struct S{void(*f)(); int a;} s = {0, 5};        01 04 04 · 00
///   …                            s = {(void(*)())4} 01 04 04 · 04
/// ```
///
/// and it is `_s__ThrowInfo::pForwardCompat` — `z08_throw`'s `_TI1?AUE@@` reads
/// `01 02 04 00 · 01 04 04 00 · 01 04 04 00 · 02 <_CTA1> 00 04`, two null
/// function pointers (`pmfnUnwind` is null too, because the thrown type has no
/// destructor). All **228** of the workload's type-04 elements in the sampled
/// TUs sit at element index 2 of a four-element `_TI` record.
const TYPE_PTR_FUNC: u8 = 0x04;

/// The only width [`TYPE_PTR_DATA`] and [`TYPE_PTR_FUNC`] are measured at.
///
/// Every pointer on this 32-bit target is four bytes, so no cell can vary it —
/// an OBSERVED constant like [`ADDRESS_WIDTH`], and any other width refuses
/// with [`InInitResidue::PointerWidth`] rather than being read as an integer of
/// that size. `work/w-emitp2/scalartypes.txt` finds **no** type-03 or type-04
/// element at any other width anywhere in the 850-TU workload, so the
/// restriction costs nothing and keeps the claim to what was measured.
const POINTER_WIDTH: u8 = 4;

/// Element widths this reader admits, in bytes.
const WIDTHS: [u8; 3] = [1, 2, 4];

/// **Element tag `03` — inline bytes**, `03 <len> <len bytes>`, contributing
/// its payload to the object's raw bytes verbatim.
///
/// [`super::inlit`] reads a *record* whose only element is one of these (the
/// string literal). This reads it as an **element**, which is what `??_R0`
/// needs: its third field is the type's name and its first is a symbol address,
/// so refusing the record for the blob costs a symbol address. The length field
/// is [`super::inlit::read_len`]'s — `80` + LE**16** — reused rather than
/// re-spelled, because the crate already has three different varints and a
/// fourth spelling of this one is how they drift.
const ELEMENT_INLINE_BYTES: u8 = 0x03;

/// **Element tag `08` — a zero fill of `<count>` BYTES.**
///
/// MEASURED on fourteen fill lengths in `work/w-inread/grid`, every one checked
/// against the `.data` bytes and `SizeOfRawData` of real `c2`'s obj:
///
/// ```text
///   struct S{int a,b,c;} s = {1};        01 01 04 01 · 08 08          .data 12 B
///   int arr[4] = {1};                    01 01 04 01 · 08 0c          .data 16 B
///   char cs[8] = {97};                   01 01 01 61 · 08 07          .data  8 B
///   char cs[4] = {97};                   01 01 01 61 · 08 03          .data  4 B
///   short ss[6] = {7};                   01 01 02 07 · 08 0a          .data 12 B
///   int arr[32] = {1};                   01 01 04 01 · 08 7c          .data 128 B
///   int arr[33] = {1};                   01 01 04 01 · 08 80 80000000 .data 132 B
///   int arr[300] = {1};                  01 01 04 01 · 08 80 ac040000 .data 1200 B
/// ```
///
/// **`char cs[8] = {97}` is the row that makes the unit BYTES and not
/// elements**: a fill of 7 cannot be a count of 4-byte anythings, and the obj's
/// `.data` is exactly `61 00 00 00 00 00 00 00`.
///
/// **The fill is what the initializer list OMITS, not a zero that was written.**
/// `struct S{int* p; int a[3];} s = {&gi, {0,0,0}};` spells three explicit
/// `01 01 04 00` elements and **no** tag `08` (`z26_fill_only`), while
/// `struct S{bool b; int a[2];} s = {true};` spells `01 01 01 01 · 08 03 · 08
/// 08` — the first fill is the struct's own **padding** after the `bool`.
///
/// The count's varint is [`read_offset`]'s shape (`00..7F`, else `80` + LE32),
/// separated at 124 / 128 by `z05` and `z04`, and **not** [`super::inlit`]'s
/// LE16.
const ELEMENT_ZERO_FILL: u8 = 0x08;

/// Why a record that framed as an initializer did not yield bytes.
///
/// **The residue is named rather than counted**, because a totality check whose
/// residue is a single integer cannot distinguish *"this reader does not model
/// that record"* from *"this reader has a bug"*. Every variant below is the
/// former, by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InInitResidue {
    /// Element tag `02` — the initializer is the address of another symbol and
    /// needs a `.data` relocation.
    SymbolAddress,
    /// Element type `05` — floating point (see the module docs on the CheckSum).
    FloatingPoint,
    /// An element type byte nothing measured.
    UnknownType,
    /// A width outside [`WIDTHS`].
    UnknownWidth,
    /// The value did not frame — a short form at width > 1 whose first byte is
    /// neither `< 0x80` nor exactly `0x80`.
    ValueDidNotFrame,
    /// A [`TYPE_PTR_DATA`] or [`TYPE_PTR_FUNC`] element at a width other than
    /// [`POINTER_WIDTH`]. Unmeasured — no cell can vary a pointer's width on
    /// this target and the workload contains none — so it refuses rather than
    /// being read as an integer of that size.
    PointerWidth,
    /// An [`ELEMENT_ZERO_FILL`] count that did not frame: a high-bit short form
    /// (a desync, by the same rule [`read_offset`] applies), a negative count,
    /// or a run that would take the object past this reader's size bound.
    ZeroFill,
    /// An [`ELEMENT_INLINE_BYTES`] length that did not frame, or a payload that
    /// would take the object past this reader's size bound.
    InlineBytes,
    /// The record ran off the end of the stream.
    Truncated,
}

impl InInitResidue {
    /// A stable key for a scan to aggregate on. **Stable across the reader's
    /// widenings on purpose** — a residue reason that stops occurring must show
    /// as a `0`, not as a key that vanished, because `docs/STATUS.md` trap 5 is
    /// that absence reads as success.
    pub fn key(self) -> &'static str {
        match self {
            Self::SymbolAddress => "symbol-address",
            Self::FloatingPoint => "floating-point",
            Self::UnknownType => "unknown-type",
            Self::UnknownWidth => "unknown-width",
            Self::ValueDidNotFrame => "value-did-not-frame",
            Self::PointerWidth => "pointer-width",
            Self::ZeroFill => "zero-fill",
            Self::InlineBytes => "inline-bytes",
            Self::Truncated => "truncated",
        }
    }

    /// Every variant, so a report can print a `0` for the ones that did not
    /// occur. The array's length is asserted in the tests, so adding a variant
    /// without adding it here is a compile-adjacent failure rather than a silent
    /// hole in the report.
    pub const ALL: [InInitResidue; 9] = [
        Self::SymbolAddress,
        Self::FloatingPoint,
        Self::UnknownType,
        Self::UnknownWidth,
        Self::ValueDidNotFrame,
        Self::PointerWidth,
        Self::ZeroFill,
        Self::InlineBytes,
        Self::Truncated,
    ];
}

/// The `.in` initializer reader's own self-report, for a scan to print.
///
/// **This exists so the widening of a reader can be measured on the workload by
/// the same instrument before and after.** `DataTu::in_census` is only produced
/// for a TU that `data_tu` accepts whole — a few hundred of 878 — so it cannot
/// answer *"how many records does this reader refuse across the workload"*.
/// [`crate::IlBundle::in_init_report`] answers it for every TU that has an `.in`
/// at all.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InInitReport {
    /// Records that framed — the arity denominator.
    pub records: usize,
    /// Records that framed **and decoded** — the totality numerator.
    ///
    /// **A count of RECORDS, where [`InInitReport::values`] is a count of
    /// TOKENS**, and reading one for the other is what turned this report's own
    /// accounting control red. See [`InInitCensus::accepted`].
    pub accepted: usize,
    /// Accepted records whose token another accepted record had already bound to
    /// **the same bytes**. Not an error, and not zero.
    pub duplicate_records: usize,
    /// Elements decoded across every accepted record (**arity**, trap 4).
    pub elements: usize,
    /// Tokens bound to bytes.
    pub values: usize,
    /// Tokens two records disagreed about and which were dropped.
    pub conflicts: usize,
    /// Records that framed and did not decode.
    pub residue: usize,
    /// `(reason, count)` for **every** reason in [`InInitResidue::ALL`], in that
    /// order, including the zeroes.
    pub residue_by_reason: Vec<(&'static str, usize)>,
    /// Tag-`02` symbol-address elements **read** (0 until the reader models
    /// them), and the records carrying at least one.
    pub sym_refs: usize,
    pub records_with_sym_refs: usize,
    /// **THE DENOMINATOR THIS REPORT WAS SILENT ABOUT** — board **#961**.
    ///
    /// Records this reader can *see the start of* and does **not** anchor,
    /// because their first element is an [`ELEMENT_INLINE_BYTES`] blob or an
    /// [`ELEMENT_ZERO_FILL`] run and the scan's two anchors are `00 01` and
    /// `00 02`. Each one is counted only when it frames **all the way to its
    /// `07`** under the same fail-closed rule the `00 02` arm applies, so this
    /// is a count and not an estimate.
    ///
    /// **It is published beside [`InInitReport::records`] and is deliberately
    /// NOT folded into it.** The totality identity `records == accepted +
    /// residue` is correct and is a statement about the population the anchor
    /// scan reaches; `docs/STATUS.md` trap 0 is a control whose denominator is
    /// chosen by the same predicate that decides its numerator, and this is the
    /// number that makes that visible without changing what the reader accepts.
    /// A sequential parse of the same 850 workload streams frames **879,377**
    /// records where the anchor scan counted **518,098**
    /// (`work/w-emitp2/two_readers.txt`).
    pub unanchored: usize,
    /// `00 02` candidates the **fail-closed arm** dropped — *"not a record:
    /// count nothing, resync by one"*. The second half of the silent
    /// population, and the larger one: **239,279** records over the same 850
    /// TUs before this lane's widening.
    pub fail_closed: usize,
    /// `00 01` / `00 02` candidates whose preceding bytes do not read back as a
    /// token of exactly the right width, so no record is started at all. The
    /// third silent population, and the one nothing had ever counted.
    pub no_token: usize,
}

impl InInitCensus {
    /// Fold this census into the shape a scan prints.
    pub(crate) fn report(&self) -> InInitReport {
        let residue_by_reason = InInitResidue::ALL
            .iter()
            .map(|r| (r.key(), self.residue.iter().filter(|(_, w)| w == r).count()))
            .collect();
        InInitReport {
            records: self.records,
            accepted: self.accepted,
            duplicate_records: self.duplicate_records,
            elements: self.elements,
            values: self.values.len(),
            conflicts: self.conflicts,
            residue: self.residue.len(),
            residue_by_reason,
            sym_refs: self.refs.values().map(|v| v.len()).sum(),
            records_with_sym_refs: self.refs.values().filter(|v| !v.is_empty()).count(),
            unanchored: self.unanchored,
            fail_closed: self.fail_closed,
            no_token: self.no_token,
        }
    }
}

/// One **element tag `02`** — the address of another symbol — as read.
///
/// See `work/w-tag02/GRAMMAR.md` for the 24 frozen cells this is measured on.
/// The element contributes `addend` as a big-endian i32 to the object's raw
/// bytes at `at`, and the obj carries one `IMAGE_REL_PPC_ADDR32` there naming
/// `target`'s COFF symbol. **The bytes alone are not the object**: emitting them
/// without the relocation is a wrong obj, which is why this rides in its own
/// channel and not inside [`InInitCensus::values`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InSymbolRef {
    /// Byte offset of the pointer slot **within the initialized object**.
    pub at: u32,
    /// The target's `.gl` operand token — a token, never a name (#918: the
    /// per-record binding is the only one that can be trusted).
    pub target: u32,
    /// The addend, as the signed value the `.in` varint spells. Already present
    /// in [`InInitCensus::values`] as four big-endian bytes at `at`.
    pub addend: i32,
}

/// The `.in` scalar initializers, plus the census a caller needs to believe them.
pub(crate) struct InInitCensus {
    /// Token → the initializer's bytes **in the obj's (big-endian) order**,
    /// exactly `sum(width)` long.
    pub(crate) values: std::collections::BTreeMap<u32, Vec<u8>>,
    /// Token → the symbol addresses its initializer carries, in element order.
    ///
    /// **A separate channel from `values` on purpose.** A caller that consumes
    /// `values` and ignores this map emits a `.data` with the right bytes and no
    /// relocation — a wrong obj produced out of what used to be an honest
    /// refusal, which is board **#232**'s exact shape. Every consumer must
    /// either place these or refuse the object.
    pub(crate) refs: std::collections::BTreeMap<u32, Vec<InSymbolRef>>,
    /// Records that framed at all — the arity denominator.
    pub(crate) records: usize,
    /// Records that framed **and decoded** — the totality numerator.
    ///
    /// **This counts RECORDS where [`InInitCensus::values`] counts TOKENS, and
    /// the two are not the same number.** Two records can carry the same token
    /// and the same bytes: one entry in `values`, no conflict, two records. So
    /// `values + residue + conflicts == records` holds only while every accepted
    /// token is named by exactly one record — true of the scalar-only
    /// population, and **false at 826 of 878 workload TUs** the moment tag `02`
    /// was read, because the accepted population grew by an order of magnitude.
    ///
    /// The identity is stated over this field instead, and the duplicate
    /// population is published rather than absorbed. That is the repair the
    /// scan's own `in-init-accounting-broken` control forced; the control going
    /// red is what it is for.
    pub(crate) accepted: usize,
    /// Accepted records whose token an earlier accepted record had already bound
    /// to **the same bytes**. Not an error, and measurably not zero.
    pub(crate) duplicate_records: usize,
    /// Elements decoded across every accepted record. **Arity, not totality**:
    /// `records` counts entities and `elements` counts their contents, and a
    /// reader that lost an element inside a record it still accepted would leave
    /// `records` and the residue untouched. `docs/STATUS.md` trap 4 is this
    /// distinction; the project has one recorded case of totality staying silent
    /// at residue 0 while arity went red.
    pub(crate) elements: usize,
    /// Records that framed and did not decode, with the reason. Never empty on a
    /// real capture — every TU carries a constant pool.
    pub(crate) residue: Vec<(u32, InInitResidue)>,
    /// Tokens two records disagreed about, dropped rather than resolved to the
    /// first. **Injectivity**: a token that survives names exactly one byte
    /// string.
    pub(crate) conflicts: usize,
    /// See [`InInitReport::unanchored`] — board **#961**, the denominator the
    /// totality identity is silent about.
    pub(crate) unanchored: usize,
    /// See [`InInitReport::fail_closed`].
    pub(crate) fail_closed: usize,
    /// See [`InInitReport::no_token`].
    pub(crate) no_token: usize,
}

/// Read one element's value, `width` bytes wide, returning it **big-endian**.
///
/// See the module docs for why the width is a parameter and not an assumption.
fn read_value(inb: &[u8], p: &mut usize, width: u8) -> Result<Vec<u8>, InInitResidue> {
    let b0 = *inb.get(*p).ok_or(InInitResidue::Truncated)?;
    if width == 1 {
        // ONE RAW BYTE, including `0x80` — `(char)128` spells `80` and does not
        // escape. There is no ambiguity because the width already said 1.
        *p += 1;
        return Ok(vec![b0]);
    }
    if b0 < 0x80 {
        // Short form: a non-negative value below 128, zero-extended to `width`.
        // Every measured negative escapes instead, so a byte in `81..=FF` here is
        // not a sign-extended short form and is refused below.
        *p += 1;
        let mut v = vec![0u8; width as usize];
        v[width as usize - 1] = b0;
        return Ok(v);
    }
    if b0 != 0x80 {
        return Err(InInitResidue::ValueDidNotFrame);
    }
    let n = width as usize;
    let lo = *p + 1;
    let hi = lo.checked_add(n).ok_or(InInitResidue::Truncated)?;
    if hi > inb.len() {
        return Err(InInitResidue::Truncated);
    }
    // `.in` stores the escape payload little-endian; the obj wants big-endian.
    let mut v = inb[lo..hi].to_vec();
    v.reverse();
    *p = hi;
    Ok(v)
}

/// Read the tag-`02` **offset** field: a byte below `0x80`, else `0x80` + a
/// little-endian i32.
///
/// **This is the third place the crate has to spell a varint and it is not
/// interchangeable with either neighbour.** [`super::inlit`]'s length escape is
/// `80` + LE**16**; [`read_value`]'s escape width comes from the element's own
/// width byte — and here the width byte comes *after* the value, so it cannot
/// be consulted. Measured at `0`, `4`, `8`, `0x80`, `0xA0`, `0x4B0`, `0x10000`
/// and **`-4`**; the escape is LE32 in every one of the five that escape.
///
/// The short form is restricted to `00..7F`, deliberately, even though
/// [`super::readers::read_varint`]'s short form is a **signed** byte: every
/// measured negative offset escapes (`-4` is `80 fc ff ff ff`, not `fc`), so a
/// high-bit byte here is a desync and not a sign-extended offset, and reading it
/// as `-5` would put four wrong bytes in a `.data` **and** claim the wrong
/// relocation addend. `ininit.rs` already applies exactly this rule to scalar
/// values; the two agree on purpose.
fn read_offset(inb: &[u8], p: &mut usize) -> Result<i32, InInitResidue> {
    let b0 = *inb.get(*p).ok_or(InInitResidue::Truncated)?;
    if b0 < 0x80 {
        *p += 1;
        return Ok(b0 as i32);
    }
    if b0 != 0x80 {
        return Err(InInitResidue::ValueDidNotFrame);
    }
    let lo = *p + 1;
    let hi = lo.checked_add(4).ok_or(InInitResidue::Truncated)?;
    if hi > inb.len() {
        return Err(InInitResidue::Truncated);
    }
    *p = hi;
    Ok(i32::from_le_bytes([inb[lo], inb[lo + 1], inb[lo + 2], inb[lo + 3]]))
}

/// Read an [`ELEMENT_ZERO_FILL`] **count**: the same varint [`read_offset`]
/// spells — a byte below `0x80`, else `0x80` + a little-endian i32.
///
/// **Separated from [`super::inlit`]'s LE16 length by two cells that differ by
/// one array element**: `int arr[32] = {1};` spells its 124-byte fill `08 7c`
/// and `int arr[33] = {1};` spells its 128-byte fill `08 80 80 00 00 00`, with
/// `int arr[300] = {1};` at `08 80 ac 04 00 00` = 1,196. An LE16 reading would
/// have taken `80 80 00` as 128 and then read `00 00` as two more elements.
///
/// A negative count is refused rather than clamped: nothing measured spells one
/// and a negative fill is not a length.
fn read_fill_count(inb: &[u8], p: &mut usize) -> Result<u32, InInitResidue> {
    let b0 = *inb.get(*p).ok_or(InInitResidue::Truncated)?;
    if b0 < 0x80 {
        *p += 1;
        return Ok(b0 as u32);
    }
    if b0 != 0x80 {
        // A high-bit short form is a desync by the same rule `read_offset`
        // applies, not a sign-extended count.
        return Err(InInitResidue::ZeroFill);
    }
    let lo = *p + 1;
    let hi = lo.checked_add(4).ok_or(InInitResidue::Truncated)?;
    if hi > inb.len() {
        return Err(InInitResidue::Truncated);
    }
    let v = i32::from_le_bytes([inb[lo], inb[lo + 1], inb[lo + 2], inb[lo + 3]]);
    if v < 0 {
        return Err(InInitResidue::ZeroFill);
    }
    *p = hi;
    Ok(v as u32)
}

/// A record longer than any object this class admits is a desync, not a large
/// initializer. The bound predates this lane; what is new is that the two
/// length-carrying elements check it **before** allocating, so a corrupt
/// `08 80 ff ff ff 7f` cannot ask for two gigabytes on the way to being
/// refused.
const MAX_OBJECT_BYTES: usize = 1 << 16;

/// Parse the element run of one record, starting just past its [`RECORD_TAG`].
///
/// Returns the object's bytes in the obj's order, the element count (arity) and
/// the symbol addresses the run carries, each keyed to its byte offset in the
/// object.
fn read_elements(
    inb: &[u8],
    p: &mut usize,
) -> Result<(Vec<u8>, usize, Vec<InSymbolRef>), InInitResidue> {
    let mut out: Vec<u8> = Vec::new();
    let mut n = 0usize;
    let mut refs: Vec<InSymbolRef> = Vec::new();
    loop {
        let tag = *inb.get(*p).ok_or(InInitResidue::Truncated)?;
        if tag == RECORD_END {
            *p += 1;
            return Ok((out, n, refs));
        }
        if tag == ELEMENT_SYMBOL_ADDRESS {
            // `02 <target-token> <offset> <n>` — the address of another symbol.
            // MEASURED on 24 frozen cells; `work/w-tag02/GRAMMAR.md` is the
            // byte table and `docs/OBJ_DATA_BSS_SHAPE.md` §8.6 is the entry this
            // closes.
            let (target, tw) =
                read_token_var(inb, *p + 1).ok_or(InInitResidue::Truncated)?;
            let mut q = *p + 1 + tw;
            let addend = read_offset(inb, &mut q)?;
            // The trailing width. **`04` is the only value the grid can produce
            // — every pointer on this target is four bytes — so it is an
            // OBSERVED constant, not a known one, and anything else is refused
            // rather than believed to be a width.**
            match *inb.get(q).ok_or(InInitResidue::Truncated)? {
                ADDRESS_WIDTH => {}
                _ => return Err(InInitResidue::SymbolAddress),
            }
            q += 1;
            // The element's contribution to the raw bytes is the addend as a
            // big-endian i32 — measured on `t21_offset_negative`, whose `.data`
            // reads `ff ff ff fc`. The relocation supplies the rest, which is
            // why it rides out in its own channel.
            refs.push(InSymbolRef { at: out.len() as u32, target, addend });
            out.extend_from_slice(&addend.to_be_bytes());
            *p = q;
            n += 1;
            if out.len() > MAX_OBJECT_BYTES {
                return Err(InInitResidue::ValueDidNotFrame);
            }
            continue;
        }
        if tag == ELEMENT_ZERO_FILL {
            // `08 <count>` — `count` ZERO BYTES. MEASURED on fourteen fill
            // lengths against real `c2`'s obj; see [`ELEMENT_ZERO_FILL`].
            //
            // **The bytes it contributes are load-bearing beyond their own
            // value**: `z06_fill_then_ptr` puts an 8-byte fill between a scalar
            // and a pointer and its obj carries the `ADDR32` at **offset 12**,
            // so a fill read at the wrong length moves every relocation after
            // it. That cell is the reason this is a byte count and not an
            // element count in the reader.
            let mut q = *p + 1;
            let count = read_fill_count(inb, &mut q)? as usize;
            if out.len() + count > MAX_OBJECT_BYTES {
                return Err(InInitResidue::ZeroFill);
            }
            out.resize(out.len() + count, 0);
            *p = q;
            n += 1;
            continue;
        }
        if tag == ELEMENT_INLINE_BYTES {
            // `03 <len> <len bytes>` — the payload, verbatim. See
            // [`ELEMENT_INLINE_BYTES`]: reading this as an *element* rather than
            // as a whole record is what lets `??_R0` decode, and `??_R0` carries
            // a symbol address in its first field.
            let mut q = *p + 1;
            let len =
                super::inlit::read_len(inb, &mut q).ok_or(InInitResidue::InlineBytes)? as usize;
            if out.len() + len > MAX_OBJECT_BYTES {
                return Err(InInitResidue::InlineBytes);
            }
            let hi = q.checked_add(len).ok_or(InInitResidue::Truncated)?;
            if hi > inb.len() {
                return Err(InInitResidue::Truncated);
            }
            out.extend_from_slice(&inb[q..hi]);
            *p = hi;
            n += 1;
            continue;
        }
        if tag != ELEMENT_SCALAR {
            // Anything else is unmeasured and refuses — it is not a constant
            // this reader can put in a `.data`.
            return Err(InInitResidue::UnknownType);
        }
        let ty = *inb.get(*p + 1).ok_or(InInitResidue::Truncated)?;
        let width = *inb.get(*p + 2).ok_or(InInitResidue::Truncated)?;
        match ty {
            TYPE_INT_SIGNED | TYPE_INT_UNSIGNED => {
                if !WIDTHS.contains(&width) {
                    return Err(InInitResidue::UnknownWidth);
                }
            }
            // A pointer whose value is a plain integer. **Only
            // [`POINTER_WIDTH`] is measured**, and the workload contains no
            // other, so any other width refuses rather than being read as an
            // integer of that size.
            TYPE_PTR_DATA | TYPE_PTR_FUNC => {
                if width != POINTER_WIDTH {
                    return Err(InInitResidue::PointerWidth);
                }
            }
            0x05 => return Err(InInitResidue::FloatingPoint),
            _ => return Err(InInitResidue::UnknownType),
        }
        *p += 3;
        out.extend_from_slice(&read_value(inb, p, width)?);
        n += 1;
        if out.len() > MAX_OBJECT_BYTES {
            return Err(InInitResidue::ValueDidNotFrame);
        }
    }
}

/// Every scalar initializer `.in` defines, keyed by the operand token its `.gl`
/// data record carries.
///
/// **Graded on its own invariants and not on the oracle**, because the compiler
/// judges obj bytes and cannot say whether record *R* is object *S*:
///
/// * **injectivity** — a token two records disagree about is dropped, and the
///   drop is counted in [`InInitCensus::conflicts`];
/// * **totality** — every record that framed is either in `values` or named in
///   [`InInitCensus::residue`] with its reason, so `records == values.len() +
///   residue.len()` after conflicts are accounted;
/// * **arity** — [`InInitCensus::elements`] counts the *contents*, which a
///   records-only check cannot see.
pub(crate) fn in_scalar_initializers(inb: &[u8]) -> InInitCensus {
    let mut values: std::collections::BTreeMap<u32, Option<Vec<u8>>> =
        std::collections::BTreeMap::new();
    let mut refs: std::collections::BTreeMap<u32, Vec<InSymbolRef>> =
        std::collections::BTreeMap::new();
    let mut residue: Vec<(u32, InInitResidue)> = Vec::new();
    let mut records = 0usize;
    let mut accepted = 0usize;
    let mut duplicate_records = 0usize;
    let mut elements = 0usize;
    // **The three silent populations — board #961.** None of them changes what
    // this scan accepts or where it resumes; they are counted so the totality
    // identity's denominator stops being invisible. See
    // [`InInitReport::unanchored`].
    let mut unanchored = 0usize;
    let mut fail_closed = 0usize;
    let mut no_token = 0usize;
    let mut i = 0usize;
    while i + 1 < inb.len() {
        // **TWO anchors, and they are deliberately not symmetric.**
        //
        // `00 01` is the original: a record whose first element is a scalar. It
        // is left byte-for-byte as it was, because every number the workload
        // scan reports about this reader — `records`, `elements`, the residue
        // histogram — is a number about *that* scan, and a lane that widened it
        // would have no before/after to compare.
        //
        // `00 02` is new, and it is the whole reason a pure pointer initializer
        // was invisible rather than refused: `int* gp = &gi;` spells
        // `<tok> 00 02 e3 09 00 04 07`, so the old anchor never matched it and
        // its token simply had no value (the unit test named
        // `a_pure_symbol_address_record_is_never_scanned` pinned exactly that).
        // `00 02` is a much commoner byte pair than `00 01` — it occurs inside
        // every four-byte escape payload whose third byte is 2 — so this arm
        // requires the record to **frame all the way to its `07`** before it
        // counts anything at all. A candidate that does not frame is not a
        // record, contributes to neither `records` nor the residue, and the scan
        // resumes one byte on.
        let anchor_scalar = inb[i] == RECORD_TAG && inb[i + 1] == ELEMENT_SCALAR;
        let anchor_address = inb[i] == RECORD_TAG && inb[i + 1] == ELEMENT_SYMBOL_ADDRESS;
        if !anchor_scalar && !anchor_address {
            // **#961: COUNT what this scan cannot see, without changing what it
            // does.** A record whose first element is a tag-`03` blob or a
            // tag-`08` fill matches neither anchor, so it is in neither
            // `records` nor the residue — it is invisible to the totality
            // control, the arity control and the residue histogram at once.
            //
            // Widening the anchor set to `00 03` / `00 08` is DECLINED and
            // priced: over the 850-TU workload those records are **144,850**
            // and they carry **ZERO** tag-02 symbol addresses
            // (`work/w-emitp2/blindspot.txt`), while `00 03` is a byte pair that
            // occurs inside any four-byte escape payload whose third byte is 3.
            // Counting them costs nothing and admitting them would change
            // `values` for tokens `super::inlit` already binds.
            //
            // The count is fail-closed exactly as the `00 02` arm is: a
            // candidate is counted only if a token reads back at the right width
            // AND the run frames all the way to its `07`. The scan still resumes
            // one byte on, so no trajectory changes.
            if i + 1 < inb.len()
                && inb[i] == RECORD_TAG
                && (inb[i + 1] == ELEMENT_INLINE_BYTES || inb[i + 1] == ELEMENT_ZERO_FILL)
            {
                for w in [4usize, 2] {
                    if i < w {
                        continue;
                    }
                    let Some((_tok, got)) = read_token_var(inb, i - w) else { continue };
                    if got != w {
                        continue;
                    }
                    let mut p = i + 1;
                    if matches!(read_elements(inb, &mut p), Ok((b, _, _)) if !b.is_empty()) {
                        unanchored += 1;
                    }
                    break;
                }
            }
            i += 1;
            continue;
        }
        // The token ends where [`RECORD_TAG`] begins. Try the 4-byte form first
        // and require its decoded width to land exactly there — the same
        // discipline `gl_symbol_index` and `in_string_literals` apply.
        let mut matched = false;
        for w in [4usize, 2] {
            if i < w {
                continue;
            }
            let Some((tok, got)) = read_token_var(inb, i - w) else {
                continue;
            };
            if got != w {
                continue;
            }
            let mut p = i + 1;
            let parsed = read_elements(inb, &mut p);
            if anchor_address && !matches!(&parsed, Ok((b, _, _)) if !b.is_empty()) {
                // The fail-closed arm. Not a record: count nothing, resync by one.
                // **#961**: `fail_closed` is the one number this arm used to
                // leave completely silent, and it is the LARGER half of the
                // 384,129 records that were in neither `records` nor the
                // residue — 239,279 of them before this lane's widening.
                fail_closed += 1;
                break;
            }
            records += 1;
            match parsed {
                Ok((bytes, n, r)) if !bytes.is_empty() => {
                    elements += n;
                    accepted += 1;
                    match values.get(&tok) {
                        None => {
                            values.insert(tok, Some(bytes));
                            if !r.is_empty() {
                                refs.insert(tok, r);
                            }
                        }
                        Some(Some(prev)) if *prev != bytes => {
                            values.insert(tok, None);
                        }
                        // Two records, one token, the same bytes. Not a conflict
                        // — but it IS a second record, and counting it as one
                        // entry in `values` is what broke the totality identity
                        // at 826 TUs once tag `02` widened the accepted set.
                        _ => duplicate_records += 1,
                    }
                    i = p;
                }
                Ok(_) => {
                    residue.push((tok, InInitResidue::ValueDidNotFrame));
                    i = p;
                }
                Err(why) => {
                    residue.push((tok, why));
                    i += 2;
                }
            }
            matched = true;
            break;
        }
        if !matched {
            // Neither token width read back to land exactly on the anchor, so
            // no record was started. **#961**: the third silent population, and
            // the one nothing had ever counted.
            no_token += 1;
            i += 1;
        }
    }
    let conflicts = values.values().filter(|v| v.is_none()).count();
    // A poisoned token names no byte string, so it names no relocation either —
    // dropping its refs with its bytes keeps the two channels describing the
    // same set of objects, which is what lets a consumer trust their pairing.
    refs.retain(|t, _| values.get(t).map(Option::is_some).unwrap_or(false));
    InInitCensus {
        values: values.into_iter().filter_map(|(t, b)| b.map(|b| (t, b))).collect(),
        refs,
        records,
        accepted,
        duplicate_records,
        elements,
        residue,
        conflicts,
        unanchored,
        fail_closed,
        no_token,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build `<token> 00 <elements…> 07`.
    fn record(tok: [u8; 2], elems: &[u8]) -> Vec<u8> {
        let mut v = vec![tok[0], tok[1], RECORD_TAG];
        v.extend_from_slice(elems);
        v.push(RECORD_END);
        v
    }

    /// **The fifteen measured cells**, byte for byte, with the obj's byte order
    /// on the right. Every row was read off a real capture at the workload's
    /// flags; nothing here is constructed from a rule.
    #[test]
    fn the_measured_scalar_records_decode_big_endian() {
        let cases: [(&str, &[u8], &[u8]); 13] = [
            // (source, .in element bytes, expected obj bytes)
            ("int sa = 3;", &[0x01, 0x01, 0x04, 0x03], &[0, 0, 0, 3]),
            ("int i1 = 0x11223344;", &[0x01, 0x01, 0x04, 0x80, 0x44, 0x33, 0x22, 0x11],
             &[0x11, 0x22, 0x33, 0x44]),
            ("int i2 = 200;", &[0x01, 0x01, 0x04, 0x80, 0xc8, 0, 0, 0], &[0, 0, 0, 0xc8]),
            ("int i5 = 127;", &[0x01, 0x01, 0x04, 0x7f], &[0, 0, 0, 0x7f]),
            ("int n1 = -5;", &[0x01, 0x01, 0x04, 0x80, 0xfb, 0xff, 0xff, 0xff],
             &[0xff, 0xff, 0xff, 0xfb]),
            ("int i7 = -1;", &[0x01, 0x01, 0x04, 0x80, 0xff, 0xff, 0xff, 0xff],
             &[0xff, 0xff, 0xff, 0xff]),
            ("unsigned u1 = 0xFFFFFFFF;", &[0x01, 0x02, 0x04, 0x80, 0xff, 0xff, 0xff, 0xff],
             &[0xff, 0xff, 0xff, 0xff]),
            ("short s6 = 127;", &[0x01, 0x01, 0x02, 0x7f], &[0, 0x7f]),
            ("short s5 = 128;", &[0x01, 0x01, 0x02, 0x80, 0x80, 0x00], &[0x00, 0x80]),
            ("short s7 = -5;", &[0x01, 0x01, 0x02, 0x80, 0xfb, 0xff], &[0xff, 0xfb]),
            ("short sn = -300;", &[0x01, 0x01, 0x02, 0x80, 0xd4, 0xfe], &[0xfe, 0xd4]),
            ("char c2 = (char)200;", &[0x01, 0x01, 0x01, 0xc8], &[0xc8]),
            ("bool bl = true;", &[0x01, 0x01, 0x01, 0x01], &[0x01]),
        ];
        for (src, elems, want) in cases {
            let got = in_scalar_initializers(&record([0xe3, 0x09], elems));
            assert_eq!(
                got.values.get(&0xe309).map(|v| v.as_slice()),
                Some(want),
                "{src}"
            );
            assert_eq!(got.conflicts, 0, "{src}");
        }
    }

    /// **The width-1 escape boundary — the row that makes `read_value` take the
    /// width as a parameter.**
    ///
    /// `char c3 = (char)128;` spells its value `80` with NO escape, because the
    /// width already said one byte. A reader that treated `80` as a marker at
    /// every width would consume the record's `07` terminator as payload and
    /// desynchronize the rest of the stream. Both sides of the boundary are
    /// pinned, and the same byte at width 2 is asserted to mean the opposite
    /// thing.
    #[test]
    fn a_width_1_value_of_0x80_is_a_raw_byte_and_at_width_2_it_is_the_escape() {
        let got = in_scalar_initializers(&record([0xe3, 0x09], &[0x01, 0x01, 0x01, 0x80]));
        assert_eq!(got.values.get(&0xe309).map(|v| v.as_slice()), Some(&[0x80u8][..]));

        // The SAME first byte at width 2 introduces a two-byte LE payload.
        let got = in_scalar_initializers(&record([0xe3, 0x09], &[0x01, 0x01, 0x02, 0x80, 0x80, 0x00]));
        assert_eq!(got.values.get(&0xe309).map(|v| v.as_slice()), Some(&[0x00u8, 0x80][..]));
    }

    /// An aggregate is several elements in one record, and the bytes concatenate
    /// in element order. `int a1[2] = {1,2};` — MEASURED.
    #[test]
    fn an_aggregate_is_several_elements_in_one_record() {
        let got = in_scalar_initializers(&record(
            [0xe3, 0x09],
            &[0x01, 0x01, 0x04, 0x01, 0x01, 0x01, 0x04, 0x02],
        ));
        assert_eq!(
            got.values.get(&0xe309).map(|v| v.as_slice()),
            Some(&[0, 0, 0, 1, 0, 0, 0, 2][..])
        );
        assert_eq!(got.elements, 2, "ARITY: two elements, not one record's worth");
        assert_eq!(got.records, 1);
    }

    /// **A record whose first element is a SYMBOL ADDRESS is read now** — and
    /// this test is the record of what it used to say, because the widening is
    /// the lane's whole subject.
    ///
    /// Until board **#931** it was called
    /// `a_pure_symbol_address_record_is_never_scanned` and asserted
    /// `records == 0`: the scan anchored on `00 01` alone, so `int* gp = &gi;`
    /// (`<tok> 00 02 e3 09 00 04 07`, MEASURED then and re-measured on
    /// `work/w-tag02/grid/t01_ptr_to_global.cpp` now) never matched and its token
    /// simply had no value. Invisible, not refused — which is why the `.gl` data
    /// reader returned **1 of 12** records on the `struct A{virtual void f();int
    /// a;}; A g;` TU that #931 was filed from.
    #[test]
    fn a_pure_symbol_address_record_is_read_and_carries_its_reference() {
        let got = in_scalar_initializers(&record([0xe4, 0x09], &[0x02, 0xe3, 0x09, 0x00, 0x04]));
        assert_eq!(got.records, 1, "framed now — it used to be invisible");
        assert_eq!(
            got.values.get(&0xe409).map(|v| v.as_slice()),
            Some(&[0, 0, 0, 0][..]),
            "the addend, which is what the obj's raw bytes hold"
        );
        assert_eq!(
            got.refs.get(&0xe409).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 0, target: 0xe309, addend: 0 }][..]),
            "and the reference, WITHOUT which those four bytes are a wrong obj"
        );
        assert_eq!(got.elements, 1, "ARITY");
        assert!(got.residue.is_empty());
    }

    /// **The mixed aggregate** — `struct{int a; int* p;} s = {7, &gi};`, which is
    /// `work/w-tag02/grid/t13_mixed_struct.cpp` and whose real obj reads
    /// `.data = 00 00 00 07 00 00 00 00` with **one ADDR32 at offset 4**.
    ///
    /// Until #931 this asserted `residue == [(tok, SymbolAddress)]` — the record
    /// was entered through its scalar first element and then refused whole,
    /// rather than returning a truncated four bytes for an eight-byte object.
    /// The refusal was right for a reader that could not place the relocation;
    /// what makes reading it right now is that the offset **4** comes out with
    /// the bytes and is checkable against the obj.
    #[test]
    fn a_mixed_aggregate_yields_both_elements_and_the_reference_at_offset_4() {
        let got = in_scalar_initializers(&record(
            [0xe4, 0x09],
            &[0x01, 0x01, 0x04, 0x07, 0x02, 0xe3, 0x09, 0x00, 0x04],
        ));
        assert_eq!(
            got.values.get(&0xe409).map(|v| v.as_slice()),
            Some(&[0, 0, 0, 7, 0, 0, 0, 0][..]),
        );
        assert_eq!(
            got.refs.get(&0xe409).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 4, target: 0xe309, addend: 0 }][..]),
        );
        assert_eq!(got.elements, 2, "ARITY: two elements in one record");
        assert!(got.residue.is_empty());
    }

    /// **The measured tag-02 cells, byte for byte** —
    /// `work/w-tag02/GRAMMAR.md` §2. Every row was read off a real capture of a
    /// frozen `sha256`'d source at the workload's own flags; the right-hand
    /// column is the obj's `.data` bytes and its relocation, read out of the obj
    /// by `scripts/gt_dump.py` and not from this reader.
    #[test]
    fn the_measured_symbol_address_cells_decode() {
        // (cell, .in element bytes, obj bytes, addend)
        let cases: [(&str, &[u8], &[u8], i32); 7] = [
            ("t01 int* gp = &gi;", &[0x02, 0xe3, 0x09, 0x00, 0x04], &[0, 0, 0, 0], 0),
            ("t09 &s.b (offset 4)", &[0x02, 0xe3, 0x09, 0x04, 0x04], &[0, 0, 0, 4], 4),
            ("t10 &arr[2] (offset 8)", &[0x02, 0xe3, 0x09, 0x08, 0x04], &[0, 0, 0, 8], 8),
            (
                "t18 &s.b (offset 128 — the escape boundary)",
                &[0x02, 0xe3, 0x09, 0x80, 0x80, 0x00, 0x00, 0x00, 0x04],
                &[0, 0, 0, 0x80],
                128,
            ),
            (
                "t20 &arr[300] (offset 1200)",
                &[0x02, 0xe3, 0x09, 0x80, 0xb0, 0x04, 0x00, 0x00, 0x04],
                &[0, 0, 0x04, 0xb0],
                1200,
            ),
            (
                "t21 arr - 1 (offset -4 — NEGATIVE, and it escapes)",
                &[0x02, 0xe3, 0x09, 0x80, 0xfc, 0xff, 0xff, 0xff, 0x04],
                &[0xff, 0xff, 0xff, 0xfc],
                -4,
            ),
            (
                "t22 &s.b (offset 65536)",
                &[0x02, 0xe3, 0x09, 0x80, 0x00, 0x00, 0x01, 0x00, 0x04],
                &[0, 0x01, 0x00, 0x00],
                65536,
            ),
        ];
        for (cell, elem, want, addend) in cases {
            let got = in_scalar_initializers(&record([0xe4, 0x09], elem));
            assert_eq!(got.values.get(&0xe409).map(|v| v.as_slice()), Some(want), "{cell}");
            assert_eq!(
                got.refs.get(&0xe409).map(|v| v.as_slice()),
                Some(&[InSymbolRef { at: 0, target: 0xe309, addend }][..]),
                "{cell}"
            );
        }
    }

    /// **The 4-byte target-token form** — `t24_wide_target_token.cpp`, 31,000
    /// objects deep, MEASURED as `02 fb 82 01 00 · 00 · 04`. This is the only
    /// cell in the grid that exercises `read_token_var`'s escape on a tag-02
    /// *target*: `t17`'s 302 objects only reached `0x0b10`, and the 2-byte form
    /// runs until the stream's second byte gets its high bit.
    #[test]
    fn a_target_token_past_the_two_byte_form_is_read_at_four_bytes() {
        let got = in_scalar_initializers(&record(
            [0xe4, 0x09],
            &[0x02, 0xfb, 0x82, 0x01, 0x00, 0x00, 0x04],
        ));
        assert_eq!(
            got.refs.get(&0xe409).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 0, target: 0xfb82_0100, addend: 0 }][..]),
        );
        assert_eq!(got.values.get(&0xe409).map(|v| v.as_slice()), Some(&[0, 0, 0, 0][..]));
    }

    /// **An array of pointers is several tag-02 elements in ONE record, and
    /// their offsets are the walk** — `t08_ptr_array.cpp`, whose obj carries two
    /// ADDR32 at 0 and 4. `at` is the arity axis for this element kind: a reader
    /// that lost the second reference would leave `records` and the residue
    /// untouched (`docs/STATUS.md` trap 4).
    #[test]
    fn several_addresses_in_one_record_keep_their_offsets() {
        let got = in_scalar_initializers(&record(
            [0xe5, 0x09],
            &[0x02, 0xe3, 0x09, 0x00, 0x04, 0x02, 0xe4, 0x09, 0x00, 0x04],
        ));
        assert_eq!(
            got.refs.get(&0xe509).map(|v| v.as_slice()),
            Some(
                &[
                    InSymbolRef { at: 0, target: 0xe309, addend: 0 },
                    InSymbolRef { at: 4, target: 0xe409, addend: 0 },
                ][..]
            ),
        );
        assert_eq!(got.elements, 2, "ARITY");
        assert_eq!(got.values.get(&0xe509).map(|v| v.len()), Some(8));
    }

    /// **The two refusals a tag-02 element can reach**, both fail-closed.
    ///
    /// `<n>` other than `04` is refused rather than believed to be a width:
    /// nothing in the 24-cell grid can vary it, so it is an *observed* constant.
    /// A short-form offset in `81..FF` is refused rather than sign-extended:
    /// every measured negative offset **escapes** (`-4` is `80 fc ff ff ff`),
    /// so a high-bit byte there is a desync — the same rule `read_value` already
    /// applies to scalar values, and the opposite of what
    /// `super::readers::read_varint` does, whose short form IS signed.
    #[test]
    fn an_unmeasured_address_width_and_a_high_bit_offset_both_refuse() {
        let got = in_scalar_initializers(&record([0xe4, 0x09], &[0x02, 0xe3, 0x09, 0x00, 0x08]));
        assert!(got.values.get(&0xe409).is_none(), "n = 08 is unmeasured");
        assert!(got.refs.get(&0xe409).is_none());

        let got = in_scalar_initializers(&record(
            [0xe4, 0x09],
            &[0x01, 0x01, 0x04, 0x07, 0x02, 0xe3, 0x09, 0xfb, 0x04],
        ));
        assert_eq!(
            got.residue,
            vec![(0xe409, InInitResidue::ValueDidNotFrame)],
            "a high-bit short-form offset is a desync, not -5"
        );
        assert!(got.values.get(&0xe409).is_none());
    }

    /// **The `00 02` anchor is fail-closed and the `00 01` anchor is not**, and
    /// the asymmetry is deliberate. `00 02` occurs inside any four-byte escape
    /// payload whose third byte is 2, so a candidate that does not frame all the
    /// way to its `07` is not a record: it contributes to neither `records` nor
    /// the residue. Leaving the older anchor alone is what makes the workload's
    /// before/after numbers comparable at all.
    #[test]
    fn an_address_anchor_that_does_not_frame_counts_nothing() {
        // `<tok> 00 02 …` that runs off the end without a terminator.
        let v = vec![0xe4, 0x09, 0x00, 0x02, 0xe3, 0x09, 0x00, 0x04];
        let got = in_scalar_initializers(&v);
        assert_eq!(got.records, 0, "no terminator, so not a record");
        assert!(got.residue.is_empty(), "and not residue either");

        // The same bytes with the `07` present ARE a record.
        let mut ok = v.clone();
        ok.push(0x07);
        assert_eq!(in_scalar_initializers(&ok).records, 1);
    }

    /// **A poisoned token drops its references with its bytes.** Two records
    /// disagreeing about one token leaves no value, and a relocation whose slot
    /// has no bytes is a relocation into nothing.
    #[test]
    fn an_ambiguous_token_drops_its_references_too() {
        let mut v = record([0xe4, 0x09], &[0x02, 0xe3, 0x09, 0x00, 0x04]);
        v.extend_from_slice(&record([0xe4, 0x09], &[0x01, 0x01, 0x04, 0x03]));
        let got = in_scalar_initializers(&v);
        assert!(got.values.get(&0xe409).is_none());
        assert!(got.refs.get(&0xe409).is_none(), "the refs go with the bytes");
        assert_eq!(got.conflicts, 1);
    }

    /// **The two refusals a scalar record can reach**, each named in the residue
    /// rather than counted: a float needs the CheckSum exclusion, and an
    /// unmeasured width could be any number of bytes.
    #[test]
    fn the_refusals_are_named_in_the_residue() {
        // `double f1 = 1.0;` — MEASURED as `01 05 08 <8 raw LE bytes>`.
        let got = in_scalar_initializers(&record(
            [0xe3, 0x09],
            &[0x01, 0x05, 0x08, 0, 0, 0, 0, 0, 0, 0xf0, 0x3f],
        ));
        assert!(got.values.get(&0xe309).is_none());
        assert_eq!(got.residue, vec![(0xe309, InInitResidue::FloatingPoint)]);

        // An 8-byte integer width is outside the measured set.
        let got = in_scalar_initializers(&record([0xe3, 0x09], &[0x01, 0x01, 0x08, 0x01]));
        assert_eq!(got.residue, vec![(0xe309, InInitResidue::UnknownWidth)]);

        // A short form at width > 1 whose first byte is neither `< 0x80` nor
        // exactly `0x80`: every measured negative escapes, so this is a desync.
        let got = in_scalar_initializers(&record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0xfb]));
        assert_eq!(got.residue, vec![(0xe309, InInitResidue::ValueDidNotFrame)]);
    }

    /// **Injectivity.** A token two records disagree about is dropped, not
    /// resolved to the first — the same third value every other reader in this
    /// crate gives an ambiguous token. Two records that AGREE are not a conflict.
    #[test]
    fn an_ambiguous_token_is_dropped_and_agreement_is_not_a_conflict() {
        let mut v = record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]);
        v.extend_from_slice(&record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x04]));
        let got = in_scalar_initializers(&v);
        assert!(got.values.get(&0xe309).is_none());
        assert_eq!(got.conflicts, 1);

        let mut v = record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]);
        v.extend_from_slice(&record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]));
        let got = in_scalar_initializers(&v);
        assert_eq!(got.values.get(&0xe309).map(|v| v.as_slice()), Some(&[0, 0, 0, 3][..]));
        assert_eq!(got.conflicts, 0);
    }

    /// **Totality.** Every record that framed is either a value or a named
    /// residue entry; the accounting closes. This is the check that would go red
    /// if a future element tag were skipped silently instead of refused.
    #[test]
    fn every_framed_record_is_a_value_or_a_named_residue_entry() {
        let mut v = record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]); // ok
        v.extend_from_slice(&record([0xe4, 0x09], &[0x01, 0x05, 0x08, 0, 0, 0, 0, 0, 0, 0xf0, 0x3f])); // float
        v.extend_from_slice(&record([0xe5, 0x09], &[0x01, 0x01, 0x02, 0x7f])); // ok
        let got = in_scalar_initializers(&v);
        assert_eq!(got.accepted + got.residue.len(), got.records, "records = accepted + residue");
        assert_eq!(got.records, 3);
        assert_eq!(got.elements, 2, "ARITY: the refused record contributes none");
    }

    /// **`values` counts TOKENS and `records` counts RECORDS, and the identity
    /// that confused them was live for the whole life of this reader** (board
    /// **#937**).
    ///
    /// Two records, one token, the same bytes: not a conflict, and *two*
    /// records. `values + residue + conflicts == records` is therefore `1 == 2`
    /// — and it held on the workload only because the scalar-only accepted
    /// population never contained enough of the shape to notice. The moment tag
    /// `02` widened that population the scan's `in-init-accounting-broken`
    /// control fired at **826 of 878** TUs, which is exactly what it is for; the
    /// identity was repaired rather than the control adjusted.
    #[test]
    fn two_records_agreeing_on_one_token_are_two_records_and_one_value() {
        let mut v = record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]);
        v.extend_from_slice(&record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]));
        let got = in_scalar_initializers(&v);
        assert_eq!(got.records, 2);
        assert_eq!(got.accepted, 2, "both records decoded");
        assert_eq!(got.values.len(), 1, "one token");
        assert_eq!(got.duplicate_records, 1);
        assert_eq!(got.conflicts, 0, "agreement is not a conflict");
        assert_eq!(got.accepted + got.residue.len(), got.records, "the repaired identity closes");
        assert_ne!(
            got.values.len() + got.residue.len() + got.conflicts,
            got.records,
            "and the OLD one does not — this is the case it could never see"
        );
    }

    /// **THE MEASURED ZERO-FILL CELLS, byte for byte** —
    /// `work/w-inread/spell.txt`, read off real captures of `sha256`'d sources
    /// at the workload's own flags. The right-hand column is the length and
    /// content of the obj's `.data`, read out of the obj by `scripts/gt_dump.py`
    /// and not from this reader.
    ///
    /// **`char cs[8] = {97}` is the row that decides the UNIT.** Its fill is
    /// **7**, which cannot be a count of 4-byte anythings, and its obj's `.data`
    /// is exactly `61 00 00 00 00 00 00 00`. Every other cell is consistent with
    /// both an element count and a byte count; this one is not.
    #[test]
    fn the_measured_zero_fill_cells_decode_to_that_many_zero_bytes() {
        // (cell, source, .in element bytes, obj `.data`)
        let cases: [(&str, &[u8], Vec<u8>); 8] = [
            (
                "z01 struct S{int a,b,c;} s={1};  .data 12 B",
                &[0x01, 0x01, 0x04, 0x01, 0x08, 0x08],
                [vec![0, 0, 0, 1], vec![0u8; 8]].concat(),
            ),
            (
                "z02 int arr[4]={1};  .data 16 B",
                &[0x01, 0x01, 0x04, 0x01, 0x08, 0x0c],
                [vec![0, 0, 0, 1], vec![0u8; 12]].concat(),
            ),
            (
                "z18 char cs[8]={97};  .data 8 B — THE UNIT CELL",
                &[0x01, 0x01, 0x01, 0x61, 0x08, 0x07],
                [vec![0x61], vec![0u8; 7]].concat(),
            ),
            (
                "z19 char cs[4]={97};  .data 4 B",
                &[0x01, 0x01, 0x01, 0x61, 0x08, 0x03],
                [vec![0x61], vec![0u8; 3]].concat(),
            ),
            (
                "z23 short ss[6]={7};  .data 12 B",
                &[0x01, 0x01, 0x02, 0x07, 0x08, 0x0a],
                [vec![0, 0x07], vec![0u8; 10]].concat(),
            ),
            (
                "z05 int arr[32]={1};  .data 128 B — 124 in the SHORT form",
                &[0x01, 0x01, 0x04, 0x01, 0x08, 0x7c],
                [vec![0, 0, 0, 1], vec![0u8; 124]].concat(),
            ),
            (
                "z04 int arr[33]={1};  .data 132 B — 128 ESCAPES, and it is LE32",
                &[0x01, 0x01, 0x04, 0x01, 0x08, 0x80, 0x80, 0x00, 0x00, 0x00],
                [vec![0, 0, 0, 1], vec![0u8; 128]].concat(),
            ),
            (
                "z15 int arr[300]={1};  .data 1200 B",
                &[0x01, 0x01, 0x04, 0x01, 0x08, 0x80, 0xac, 0x04, 0x00, 0x00],
                [vec![0, 0, 0, 1], vec![0u8; 1196]].concat(),
            ),
        ];
        for (cell, elems, want) in cases {
            let got = in_scalar_initializers(&record([0xe3, 0x09], elems));
            assert_eq!(got.values.get(&0xe309).map(|v| v.as_slice()), Some(&want[..]), "{cell}");
            assert_eq!(got.elements, 2, "ARITY: the fill is an element too — {cell}");
            assert!(got.residue.is_empty(), "{cell}");
        }
    }

    /// **The 124/128 boundary, which is what separates the fill count's varint
    /// from [`super::inlit`]'s.**
    ///
    /// `int arr[32] = {1}` and `int arr[33] = {1}` differ by one array element
    /// and their fills straddle `0x80`. An LE**16** reading of the escape —
    /// which is what the crate's *other* `.in` length varint does — would take
    /// `80 80 00` as 128 and then read the remaining `00 00` as two more
    /// element tags, desynchronizing the record. Both sides are pinned, and the
    /// wrong reading is asserted to be wrong.
    #[test]
    fn a_fill_count_escapes_at_0x80_with_le32_and_not_le16() {
        // Both records lead with the scalar `int arr[N]`'s first element, which
        // is what makes them anchor at all — `00 08` is deliberately NOT an
        // anchor (see `an_unanchored_record_is_counted_and_changes_nothing_else`).
        let short = in_scalar_initializers(&record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x01, 0x08, 0x7c]));
        assert_eq!(short.values.get(&0xe309).map(|v| v.len()), Some(128));

        let esc = in_scalar_initializers(&record(
            [0xe3, 0x09],
            &[0x01, 0x01, 0x04, 0x01, 0x08, 0x80, 0x80, 0x00, 0x00, 0x00],
        ));
        assert_eq!(esc.values.get(&0xe309).map(|v| v.len()), Some(132));
        assert_eq!(esc.elements, 2, "ARITY: TWO elements, not four");

        // A high-bit short form is a desync, not a negative fill — the same rule
        // `read_offset` and `read_value` already apply.
        let bad = in_scalar_initializers(&record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x07, 0x08, 0xfb]));
        assert_eq!(bad.residue, vec![(0xe309, InInitResidue::ZeroFill)]);
        assert!(bad.values.get(&0xe309).is_none());
    }

    /// **The fill DISPLACES the relocations that follow it** —
    /// `z06_fill_then_ptr.cpp`, `struct S{int a[3]; int* p;} s = {{1}, &gi};`,
    /// whose real obj is a 16-byte `.data` with one `IMAGE_REL_PPC_ADDR32` at
    /// **offset 12**.
    ///
    /// This is the cell that makes the fill's length load-bearing beyond its own
    /// bytes: a fill read at the wrong length moves every symbol address after
    /// it, and `docs/STATUS.md` trap 4 is that a reader which lost that would
    /// leave `records` and the residue untouched.
    #[test]
    fn a_fill_moves_the_offset_of_every_reference_after_it() {
        let got = in_scalar_initializers(&record(
            [0xed, 0x09],
            &[0x01, 0x01, 0x04, 0x01, 0x08, 0x08, 0x02, 0xe3, 0x09, 0x00, 0x04],
        ));
        assert_eq!(
            got.values.get(&0xed09).map(|v| v.as_slice()),
            Some(&[0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0][..]),
        );
        assert_eq!(
            got.refs.get(&0xed09).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 12, target: 0xe309, addend: 0 }][..]),
            "the obj's ADDR32 is at 12, which is 4 + the 8-byte fill",
        );
        assert_eq!(got.elements, 3, "ARITY");

        // `z16_ptr_then_fill.cpp` — the pointer first, then a 16-byte fill; a
        // 20-byte `.data` with the ADDR32 at 0.
        let got = in_scalar_initializers(&record(
            [0xed, 0x09],
            &[0x02, 0xe3, 0x09, 0x00, 0x04, 0x08, 0x10],
        ));
        assert_eq!(got.values.get(&0xed09).map(|v| v.len()), Some(20));
        assert_eq!(
            got.refs.get(&0xed09).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 0, target: 0xe309, addend: 0 }][..]),
        );
    }

    /// **A fill is what the initializer list OMITS, and an explicit zero is
    /// not** — the boundary, measured on two cells that differ only in whether
    /// the zeros are written down.
    ///
    /// `struct S{int* p; int a[3];} s = {&gi, {0,0,0}};` (`z26_fill_only`)
    /// spells three `01 01 04 00` elements and **no** tag `08`;
    /// `struct S{bool b; int a[2];} s = {true};` (`z24_bool_fill`) spells
    /// `01 01 01 01 · 08 03 · 08 08`, where the FIRST fill is the struct's own
    /// padding after the `bool` and the second is the omitted array.
    #[test]
    fn an_omitted_element_is_a_fill_and_a_written_zero_is_a_scalar() {
        let explicit = in_scalar_initializers(&record(
            [0xed, 0x09],
            &[
                0x02, 0xe3, 0x09, 0x00, 0x04, // &gi
                0x01, 0x01, 0x04, 0x00, 0x01, 0x01, 0x04, 0x00, 0x01, 0x01, 0x04, 0x00,
            ],
        ));
        assert_eq!(explicit.values.get(&0xed09).map(|v| v.len()), Some(16));
        assert_eq!(explicit.elements, 4, "four elements, no fill");

        // Two fills in one record, and the first one is PADDING.
        let padded =
            in_scalar_initializers(&record([0xec, 0x09], &[0x01, 0x01, 0x01, 0x01, 0x08, 0x03, 0x08, 0x08]));
        assert_eq!(
            padded.values.get(&0xec09).map(|v| v.as_slice()),
            Some(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0][..]),
            "sizeof(struct with a bool and an int[2]) == 12",
        );
        assert_eq!(padded.elements, 3);
    }

    /// **Scalar type `03` is a DATA pointer and type `04` is a FUNCTION
    /// pointer**, both plain integers, both at width 4, neither carrying a
    /// relocation — `work/w-inread/grid`, checked against the obj's `.data` and
    /// its (empty) relocation table.
    ///
    /// Both were **UNMEASURED** before this lane: `w-tag02`'s 24-cell grid
    /// produced neither, and `work/w-emitp2/scalartypes.txt` counts 132,528
    /// type-03 and 4,844 type-04 elements in the workload that the reader was
    /// refusing on that account.
    #[test]
    fn the_measured_pointer_typed_scalars_decode() {
        // (cell, .in element bytes, obj bytes)
        let cases: [(&str, &[u8], &[u8]); 6] = [
            ("z09 struct S{int* p; int a;} s={0,5};", &[0x01, 0x03, 0x04, 0x00], &[0, 0, 0, 0]),
            ("z11 s={(int*)4,5};", &[0x01, 0x03, 0x04, 0x04], &[0, 0, 0, 4]),
            (
                "z12 s={(int*)0x11223344,5}; — the LE32 escape at type 03",
                &[0x01, 0x03, 0x04, 0x80, 0x44, 0x33, 0x22, 0x11],
                &[0x11, 0x22, 0x33, 0x44],
            ),
            ("z10 struct S{void(*f)(); int a;} s={0,5};", &[0x01, 0x04, 0x04, 0x00], &[0, 0, 0, 0]),
            ("z13 s={(void(*)())4,5};", &[0x01, 0x04, 0x04, 0x04], &[0, 0, 0, 4]),
            (
                "z22 s={(void(*)())0x11223344,5};",
                &[0x01, 0x04, 0x04, 0x80, 0x44, 0x33, 0x22, 0x11],
                &[0x11, 0x22, 0x33, 0x44],
            ),
        ];
        for (cell, elems, want) in cases {
            let got = in_scalar_initializers(&record([0xec, 0x09], elems));
            assert_eq!(got.values.get(&0xec09).map(|v| v.as_slice()), Some(want), "{cell}");
            assert!(got.refs.get(&0xec09).is_none(), "no relocation — {cell}");
            assert!(got.residue.is_empty(), "{cell}");
        }

        // A pointer at a width no cell can produce refuses rather than being
        // read as an integer of that size. `work/w-emitp2/scalartypes.txt` finds
        // none at any other width in the whole workload.
        for w in [1u8, 2, 8] {
            let got = in_scalar_initializers(&record([0xec, 0x09], &[0x01, 0x03, w, 0x00]));
            assert_eq!(got.residue, vec![(0xec09, InInitResidue::PointerWidth)], "width {w}");
        }
    }

    /// **`??_R0` — the record that proves the four kinds are not independent.**
    ///
    /// `z14_typedesc.cpp` (`struct A{virtual void f(); int a;}; A ga;`) spells
    /// its `_TypeDescriptor` as
    ///
    /// ```text
    ///   02 <??_7type_info@@6B@> 00 04 · 01 03 04 00 · 03 08 ".?AUA@@\0"
    /// ```
    ///
    /// and its `.data` is 16 bytes with one `ADDR32` at 0. **Teaching the reader
    /// scalar type `03` alone recovers none of it** — the record would then
    /// refuse on the tag-`03` blob instead, one element to the right — which is
    /// why `work/w-emitp2/blindspot.txt`'s rows are upper bounds and not
    /// addends.
    #[test]
    fn the_type_descriptor_record_needs_all_three_kinds_at_once() {
        let elems: &[u8] = &[
            0x02, 0x13, 0x0a, 0x00, 0x04, // the type_info vftable address
            0x01, 0x03, 0x04, 0x00, // spare — a null void*
            0x03, 0x08, b'.', b'?', b'A', b'U', b'A', b'@', b'@', 0x00, // the name
        ];
        let got = in_scalar_initializers(&record([0x0f, 0x0a], elems));
        assert_eq!(
            got.values.get(&0x0f0a).map(|v| v.as_slice()),
            Some(&[0, 0, 0, 0, 0, 0, 0, 0, b'.', b'?', b'A', b'U', b'A', b'@', b'@', 0][..]),
            "16 bytes, which is what the obj's .data is",
        );
        assert_eq!(
            got.refs.get(&0x0f0a).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 0, target: 0x130a, addend: 0 }][..]),
            "the symbol address this record was costing",
        );
        assert_eq!(got.elements, 3, "ARITY");
        assert!(got.residue.is_empty());
    }

    /// **`_TI` and `_CT` — the two CRT records the workload's type-04 and its
    /// tag-08 actually live in**, transcribed from `z08_throw.cpp`'s capture.
    ///
    /// `_TI1?AUE@@` is `_s__ThrowInfo`, whose `pmfnUnwind` and `pForwardCompat`
    /// are both null function pointers here; `_CT??_R0?AUE@@@84` is
    /// `_s__CatchableType`, whose `thisDisplacement.vdisp` is spelled as a
    /// **four-byte fill** while the `mdisp` and `pdisp` beside it are ordinary
    /// scalars — the omitted-versus-written boundary, inside the CRT.
    #[test]
    fn the_crt_throw_records_decode_whole() {
        // _TI1?AUE@@ — attributes, pmfnUnwind, pForwardCompat, pCatchableTypeArray
        let ti = in_scalar_initializers(&record(
            [0xee, 0x09],
            &[
                0x01, 0x02, 0x04, 0x00, //
                0x01, 0x04, 0x04, 0x00, //
                0x01, 0x04, 0x04, 0x00, //
                0x02, 0xf0, 0x09, 0x00, 0x04,
            ],
        ));
        assert_eq!(ti.values.get(&0xee09).map(|v| v.len()), Some(16));
        assert_eq!(
            ti.refs.get(&0xee09).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 12, target: 0xf009, addend: 0 }][..]),
        );

        // _CT??_R0?AUE@@@84 — properties, pType, mdisp, pdisp, vdisp(FILL),
        // sizeOrOffset, copyFunction
        let ct = in_scalar_initializers(&record(
            [0xf2, 0x09],
            &[
                0x01, 0x02, 0x04, 0x00, //
                0x02, 0xf1, 0x09, 0x00, 0x04, //
                0x01, 0x01, 0x04, 0x00, //
                0x01, 0x01, 0x04, 0x80, 0xff, 0xff, 0xff, 0xff, //
                0x08, 0x04, //
                0x01, 0x01, 0x04, 0x04, //
                0x01, 0x04, 0x04, 0x00,
            ],
        ));
        assert_eq!(
            ct.values.get(&0xf209).map(|v| v.as_slice()),
            Some(
                &[
                    0, 0, 0, 0, // properties
                    0, 0, 0, 0, // pType (relocated)
                    0, 0, 0, 0, // mdisp
                    0xff, 0xff, 0xff, 0xff, // pdisp = -1
                    0, 0, 0, 0, // vdisp — the FILL
                    0, 0, 0, 4, // sizeOrOffset
                    0, 0, 0, 0, // copyFunction — a null FUNCTION pointer
                ][..]
            ),
        );
        assert_eq!(ct.elements, 7, "ARITY: seven fields of _s__CatchableType");
        assert_eq!(
            ct.refs.get(&0xf209).map(|v| v.as_slice()),
            Some(&[InSymbolRef { at: 4, target: 0xf109, addend: 0 }][..]),
        );
    }

    /// **#961 — the denominator, counted rather than folded in.**
    ///
    /// A record whose first element is a tag-`03` blob or a tag-`08` fill
    /// matches neither anchor, so it is in **neither** `records` nor the
    /// residue. Over the 850-TU workload that population is **144,850** records
    /// and the fail-closed `00 02` arm drops another **239,279**
    /// (`work/w-emitp2/two_readers.txt`), which is 43.7 % of the stream
    /// invisible to the totality control, the arity control and the residue
    /// histogram at the same time.
    ///
    /// The counter must **not** change what the scan accepts: `records`,
    /// `accepted` and `values` are asserted to be identical with and without an
    /// unanchored record in the stream.
    #[test]
    fn an_unanchored_record_is_counted_and_changes_nothing_else() {
        let anchored = record([0xe3, 0x09], &[0x01, 0x01, 0x04, 0x03]);
        let mut both = record([0xe4, 0x09], &[0x03, 0x04, b'a', b'b', b'c', 0x00]);
        both.extend_from_slice(&anchored);

        let a = in_scalar_initializers(&anchored);
        let b = in_scalar_initializers(&both);
        assert_eq!(a.unanchored, 0);
        assert_eq!(b.unanchored, 1, "the blob record is SEEN and not anchored");
        assert_eq!(b.records, a.records, "and it is NOT in `records`");
        assert_eq!(b.accepted, a.accepted);
        assert_eq!(b.residue.len(), a.residue.len(), "nor in the residue");
        assert_eq!(b.values.len(), a.values.len());
        assert_eq!(
            b.accepted + b.residue.len(),
            b.records,
            "the totality identity still closes — over the population it reaches",
        );

        // A tag-08-first record is the same shape. It is the rarer half: 2
        // records over 850 TUs, against 144,848 blob-first ones.
        let fill = in_scalar_initializers(&record([0xe4, 0x09], &[0x08, 0x08]));
        assert_eq!(fill.unanchored, 1);
        assert_eq!(fill.records, 0);
    }

    /// **#961 — the fail-closed arm is the LARGER silent half, and it now says
    /// so.** A `00 02` candidate that does not frame all the way to its `07` is
    /// deliberately *"not a record: count nothing, resync by one"*; what was
    /// missing is that nothing counted how often that happened.
    #[test]
    fn a_fail_closed_candidate_is_counted_where_it_used_to_be_silent() {
        // `<tok> 00 02 …` with no terminator — the test
        // `an_address_anchor_that_does_not_frame_counts_nothing` pins that this
        // is neither a record nor residue; this pins that it is now VISIBLE.
        let got = in_scalar_initializers(&[0xe4, 0x09, 0x00, 0x02, 0xe3, 0x09, 0x00, 0x04]);
        assert_eq!(got.records, 0);
        assert!(got.residue.is_empty());
        assert_eq!(got.fail_closed, 1, "counted, where it used to be invisible");

        // The same bytes with the `07` present are a record and are NOT counted
        // as fail-closed.
        let mut ok = vec![0xe4, 0x09, 0x00, 0x02, 0xe3, 0x09, 0x00, 0x04];
        ok.push(0x07);
        let got = in_scalar_initializers(&ok);
        assert_eq!(got.records, 1);
        assert_eq!(got.fail_closed, 0);
    }

    /// Every [`InInitResidue`] variant is in [`InInitResidue::ALL`], so a new
    /// refusal cannot be added without also being reported. The report prints
    /// **every** reason including the zeroes (trap 5), and a variant missing
    /// from `ALL` would be a residue that exists and is never printed.
    #[test]
    fn every_residue_variant_is_reported() {
        let all = InInitResidue::ALL;
        assert_eq!(all.len(), 9, "add the new variant to ALL as well");
        let mut keys: Vec<&str> = all.iter().map(|r| r.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), all.len(), "two variants share a key");
    }

    /// A truncated or empty stream yields nothing and does not panic — the CLI
    /// must degrade cleanly.
    #[test]
    fn a_truncated_stream_yields_nothing_and_does_not_panic() {
        for s in [
            &[][..],
            &[0x00, 0x01][..],
            &[0xe3, 0x09, 0x00, 0x01][..],
            &[0xe3, 0x09, 0x00, 0x01, 0x01][..],
            &[0xe3, 0x09, 0x00, 0x01, 0x01, 0x04][..],
            &[0xe3, 0x09, 0x00, 0x01, 0x01, 0x04, 0x80, 0x01][..],
        ] {
            let got = in_scalar_initializers(s);
            assert!(got.values.is_empty(), "{s:?}");
        }
    }
}
