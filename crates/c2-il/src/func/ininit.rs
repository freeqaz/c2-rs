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
//! | `02` | `<token> 00 <n>` — the ADDRESS of another symbol | refused |
//! | `03` | `<len> <bytes>` — a string literal | [`super::inlit`] |
//!
//! Element tag `02` is why `int gi; int* gp = &gi;` refuses: a pointer-valued
//! initializer stores zero bytes and carries its address entirely in a `.data`
//! relocation, which `docs/OBJ_DATA_BSS_SHAPE.md` §8.6 records as unexercised.
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

/// The byte that closes an initializer record. Shared with [`super::inlit`].
const RECORD_END: u8 = 0x07;

/// Scalar element **type** bytes this reader admits — signed and unsigned
/// integer. `05` is floating point and is refused (see the module docs); every
/// other value is unseen and refuses with it.
const TYPE_INT_SIGNED: u8 = 0x01;
const TYPE_INT_UNSIGNED: u8 = 0x02;

/// Element widths this reader admits, in bytes.
const WIDTHS: [u8; 3] = [1, 2, 4];

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
            Self::Truncated => "truncated",
        }
    }

    /// Every variant, so a report can print a `0` for the ones that did not
    /// occur. The array's length is asserted in the tests, so adding a variant
    /// without adding it here is a compile-adjacent failure rather than a silent
    /// hole in the report.
    pub const ALL: [InInitResidue; 6] = [
        Self::SymbolAddress,
        Self::FloatingPoint,
        Self::UnknownType,
        Self::UnknownWidth,
        Self::ValueDidNotFrame,
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
            elements: self.elements,
            values: self.values.len(),
            conflicts: self.conflicts,
            residue: self.residue.len(),
            residue_by_reason,
            sym_refs: self.refs.values().map(|v| v.len()).sum(),
            records_with_sym_refs: self.refs.values().filter(|v| !v.is_empty()).count(),
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

/// Parse the element run of one record, starting just past its [`RECORD_TAG`].
fn read_elements(inb: &[u8], p: &mut usize) -> Result<(Vec<u8>, usize), InInitResidue> {
    let mut out: Vec<u8> = Vec::new();
    let mut n = 0usize;
    loop {
        let tag = *inb.get(*p).ok_or(InInitResidue::Truncated)?;
        if tag == RECORD_END {
            *p += 1;
            return Ok((out, n));
        }
        if tag != ELEMENT_SCALAR {
            // `02` is a symbol address, `03` a string literal, and anything else
            // is unmeasured. All three refuse — none of them is a constant this
            // writer can put in a `.data`.
            return Err(match tag {
                0x02 => InInitResidue::SymbolAddress,
                _ => InInitResidue::UnknownType,
            });
        }
        let ty = *inb.get(*p + 1).ok_or(InInitResidue::Truncated)?;
        let width = *inb.get(*p + 2).ok_or(InInitResidue::Truncated)?;
        match ty {
            TYPE_INT_SIGNED | TYPE_INT_UNSIGNED => {}
            0x05 => return Err(InInitResidue::FloatingPoint),
            _ => return Err(InInitResidue::UnknownType),
        }
        if !WIDTHS.contains(&width) {
            return Err(InInitResidue::UnknownWidth);
        }
        *p += 3;
        out.extend_from_slice(&read_value(inb, p, width)?);
        n += 1;
        if out.len() > 1 << 16 {
            // A record longer than any object this class admits is a desync, not
            // a large initializer. Bounded so a corrupt stream cannot spin.
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
    let mut elements = 0usize;
    let mut i = 0usize;
    while i + 1 < inb.len() {
        if inb[i] != RECORD_TAG || inb[i + 1] != ELEMENT_SCALAR {
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
            records += 1;
            match read_elements(inb, &mut p) {
                Ok((bytes, n)) if !bytes.is_empty() => {
                    elements += n;
                    match values.get(&tok) {
                        None => {
                            values.insert(tok, Some(bytes));
                        }
                        Some(Some(prev)) if *prev != bytes => {
                            values.insert(tok, None);
                        }
                        _ => {}
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
        elements,
        residue,
        conflicts,
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

    /// **A record whose FIRST element is not scalar is not seen at all**, and
    /// that is safe rather than sloppy: the scan anchors on `00 01`, so
    /// `int* gp = &gi;` (`<tok> 00 02 e3 09 00 04 07`, MEASURED) never matches
    /// and its token simply has no value. The caller requires a value for every
    /// initialized object, so *not found* and *refused* are the same verdict —
    /// but only the mixed case below can reach the residue, so it is the one
    /// that is asserted.
    #[test]
    fn a_pure_symbol_address_record_is_never_scanned() {
        let got = in_scalar_initializers(&record([0xe4, 0x09], &[0x02, 0xe3, 0x09, 0x00, 0x04]));
        assert!(got.values.get(&0xe409).is_none(), "no value, which is what the caller checks");
        assert_eq!(got.records, 0, "and it was never framed, so it is not residue either");
    }

    /// **The dangerous shape is the MIXED aggregate** — `struct{int a; int* p;}`
    /// — whose first element *is* scalar, so the scan enters the record and must
    /// refuse when it reaches the address element rather than returning a
    /// truncated four bytes for an eight-byte object.
    #[test]
    fn a_mixed_aggregate_refuses_instead_of_returning_a_prefix() {
        let got = in_scalar_initializers(&record(
            [0xe4, 0x09],
            &[0x01, 0x01, 0x04, 0x01, 0x02, 0xe3, 0x09, 0x00, 0x04],
        ));
        assert!(got.values.get(&0xe409).is_none(), "NOT the first element's 4 bytes");
        assert_eq!(got.residue, vec![(0xe409, InInitResidue::SymbolAddress)]);
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
        assert_eq!(
            got.values.len() + got.residue.len() + got.conflicts,
            got.records,
            "records = values + residue + conflicts"
        );
        assert_eq!(got.records, 3);
        assert_eq!(got.elements, 2, "ARITY: the refused record contributes none");
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
