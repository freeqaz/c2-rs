//! The `.sy` **local-symbol** layer — the positive local signal `.ex` does not
//! have.
//!
//! `.ex` pushes an assignment destination as `26 <tok>` whether the destination
//! is a parameter, an automatic local, a file-scope `static` or a global. That
//! conflation is why locals were out of class: a store to a memory object is a
//! real write with a relocation, and folding it away as a register copy silently
//! drops it. The previous attempt asked whether `.gl` *named* the destination and
//! refused if so — an absence test, and absence proves nothing: a file-scope
//! `static int sv` appears there as `$sv`, whose leading `$` `gl_symbol_index`
//! does not accept as an identifier, so the token looked local and the store
//! vanished. See `docs/GAPS.md` §6.
//!
//! `.sy` answers the question positively. It is a flat sequence of per-function
//! blocks, one per `.ex` function segment and in the same order:
//!
//! ```text
//!   03 01 <tok16> 1F 00 01 01      block open
//!   0D 01  <record>*               the formals section  (always present, may be empty)
//!   0D 02  <record>*               the locals section   (always present, may be empty)
//!   06                             block close
//! ```
//!
//! and an automatic-variable record is
//!
//! ```text
//!   01 <kind> <tok16> 00 <name> 00 86 <T> 00 <n> 04 04 00 <F> 00 <enc>
//! ```
//!
//! where `<kind>` repeats the enclosing section's (`01` formal, `02` local),
//! `<tok16>` is the token `.ex` uses for the variable, `<T>` is a type reference
//! and `<enc>` an inline re-encoding of the same type, and `<F>` carries
//! **address-taken**.
//!
//! # What is measured, and what is only constant
//!
//! MEASURED, each against a neighbour that would look identical under a
//! plausible wrong rule (probe sources in `fixtures/cpp/il_sy_locals.cpp`):
//!
//! * `<T> = 01`/`<enc> = 74` is plain `int`; `unsigned` is `86 02 … 75` and
//!   `int*` is `86 03 … 80 74 04 00 00`. So the type is not assumed from
//!   context — it is read, twice, and both must say `int`.
//! * `<F>` is `01` normally and `21` when the variable's address is taken. The
//!   discriminating probe is one function holding two `int` locals of the same
//!   name shape where only one has `&x` applied: the address-taken one reads
//!   `21`, the other `01`. Without that neighbour "always 01" and "01 here by
//!   coincidence" are the same observation.
//! * `const` and `volatile` do **not** change `<T>`; they change `<enc>` to
//!   `80 01 10 00 00` / `80 00 10 00 00`. A gate on `<T>` alone would admit a
//!   `volatile int` local and fold away a store that must not be folded, so
//!   `<enc>` is what this reader actually requires.
//! * A function-scope `static` is **not** an `01` record. It is a `07` record
//!   carrying the fully mangled name (`?x@?1??f@@YAHH@Z@4HA`) and a different
//!   field layout. This is the `$sv` hazard again, and the record tag is what
//!   separates them.
//! * File-scope `static`, plain globals and `extern` declarations appear in `.sy`
//!   **not at all** — only formals, automatics and function-scope statics do.
//!   That is what makes a *membership* test here sound where the `.gl` absence
//!   test was not.
//!
//! CONSTANT ACROSS EVERY WITNESS, and therefore not interpreted — required
//! literally so a deviation fails the file closed rather than being read as a
//! field this module claims to understand: the `1F 00 01 01` block-header tail,
//! the `04 04 00` run inside a record, and `<n>` (`03` in every formal, `01` in
//! every local — indistinguishable from a per-section constant).
//!
//! NOT DERIVABLE from what has been captured: the block-header `<tok16>`, which
//! is neither the function's `.gl` token nor its first formal's; and the record
//! order, which is reverse declaration order for formals but came out `y, x` in
//! one probe and `p, q, r` in a structurally identical other, so locals are
//! treated as an unordered set. Nothing here depends on either.
//!
//! # Fail-closed shape
//!
//! [`sy_blocks`] returns `None` for the **whole file** the moment it meets
//! anything it has not measured — including a `07` static-local record, whose
//! internal layout is not characterized well enough to skip without risking a
//! desync that would silently rebind later tokens. The cost is over-refusal at
//! translation-unit granularity (a TU with one function-scope static admits no
//! locals anywhere); the alternative is resynchronizing on a guess, and a wrong
//! guess here mis-binds a token and mis-emits. Widening this needs the `07`
//! layout characterized first.

/// `.sy` encodes a token exactly as `.ex` does, and this module deliberately
/// reuses `.ex`'s reader rather than its own. The two token sets are *compared*,
/// so any disagreement about the encoding makes the comparison silently always
/// false: `.ex` is big-endian with a 4-byte widening form when the second byte
/// has its high bit set, and a little-endian 2-byte reader here — which the first
/// draft had, and which parses every captured `.sy` without complaint — turns
/// `e6 09` into `0x09E6` against `.ex`'s `0xE609`. Every local was then refused
/// with no error anywhere, which is the failure mode a shared reader removes.
use super::readers::read_token_var;

/// Per-translation-unit `.sy` locals, bound 1:1 to the `.ex` function segments.
///
/// The 1:1 requirement is the same discipline `gl_defined_names` uses for names:
/// if the block count and the segment count disagree, *nothing* is bound. A
/// plausible-looking off-by-one binding would attach one function's locals to
/// another and mis-emit; refusing the whole file only costs coverage.
pub(crate) struct SyLocals {
    blocks: Option<Vec<SyBlock>>,
}

impl SyLocals {
    pub(crate) fn new(sy: Option<&[u8]>, n_segments: usize) -> Self {
        let blocks = sy
            .and_then(sy_blocks)
            .filter(|b| b.len() == n_segments);
        SyLocals { blocks }
    }

    /// The admissible local tokens of the `i`-th function segment, or nothing.
    pub(crate) fn of(&self, i: usize) -> &[u32] {
        match &self.blocks {
            Some(b) => b.get(i).map(|b| b.int_locals.as_slice()).unwrap_or(&[]),
            None => &[],
        }
    }
}

/// The tokens one `.sy` function block declares, split by what codegen may do
/// with them.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SyBlock {
    /// Formal-parameter tokens, unordered.
    pub(crate) formals: Vec<u32>,
    /// Automatic locals of plain (unqualified) `int` type whose address is never
    /// taken — the only ones a value-substituting parse may treat as a named
    /// intermediate. Every other local is deliberately absent from this list.
    pub(crate) int_locals: Vec<u32>,
}

const BLOCK_OPEN: [u8; 2] = [0x03, 0x01];
/// Constant across every witness; required literally, not interpreted.
const BLOCK_HEADER_TAIL: [u8; 4] = [0x1F, 0x00, 0x01, 0x01];
const BLOCK_CLOSE: u8 = 0x06;
const SECTION: u8 = 0x0D;
const SECTION_FORMALS: u8 = 0x01;
const SECTION_LOCALS: u8 = 0x02;
/// An automatic variable. `07` is a function-scope `static` and is refused.
const REC_AUTOMATIC: u8 = 0x01;
const TYPE_REF: u8 = 0x86;
const TY_INT: u8 = 0x01;
/// Inline type re-encoding for plain `int`. `75` is `unsigned`; a leading `80`
/// is a qualified or pointer type.
const ENC_INT: u8 = 0x74;
/// Constant across every witness; required literally, not interpreted.
const REC_MID: [u8; 3] = [0x04, 0x04, 0x00];
/// `<F>` with no address-taken bit. `0x21` is address-taken.
const FLAG_PLAIN: u8 = 0x01;
/// A `.sy` name is an identifier; anything longer than this is not one, and the
/// bound keeps a corrupt stream from scanning the rest of the file for a NUL.
const MAX_NAME: usize = 256;
/// Refuse absurd files rather than allocating against a length read from data.
const MAX_BLOCKS: usize = 65536;

/// Parse every `.sy` function block, or `None` if any byte deviates from the
/// measured grammar (see the module docs — over-refusal is the intended
/// behaviour, a resync guess is not).
pub(crate) fn sy_blocks(sy: &[u8]) -> Option<Vec<SyBlock>> {
    let mut p = 0usize;
    let mut out: Vec<SyBlock> = Vec::new();
    while p < sy.len() {
        if out.len() >= MAX_BLOCKS {
            return None;
        }
        if sy.get(p..p + 2)? != BLOCK_OPEN {
            return None;
        }
        p += 2;
        // The block-header token is read past, not interpreted: it is neither the
        // function's `.gl` token nor its first formal's, and nothing here needs it.
        let (_hdr_tok, w) = read_token_var(sy, p)?;
        p += w;
        if sy.get(p..p + 4)? != BLOCK_HEADER_TAIL {
            return None;
        }
        p += 4;

        let mut block = SyBlock::default();
        // Both sections are always present and either may be empty; requiring
        // them in order is what makes a missing one a refusal instead of a
        // silently empty local set.
        for (marker, formals) in [(SECTION_FORMALS, true), (SECTION_LOCALS, false)] {
            if *sy.get(p)? != SECTION || *sy.get(p + 1)? != marker {
                return None;
            }
            p += 2;
            while *sy.get(p)? == REC_AUTOMATIC {
                let (rec, next) = read_record(sy, p, marker)?;
                p = next;
                match (formals, rec.plain_int) {
                    (true, _) => block.formals.push(rec.tok),
                    (false, true) => block.int_locals.push(rec.tok),
                    // A local this reader can locate but must not admit: a
                    // qualified, non-`int` or address-taken variable. Recorded
                    // nowhere, so every use of its token stays out of class.
                    (false, false) => {}
                }
            }
        }
        if *sy.get(p)? != BLOCK_CLOSE {
            return None;
        }
        p += 1;
        out.push(block);
    }
    Some(out)
}

struct SyRecord {
    tok: u32,
    /// Unqualified `int`, address never taken — safe to treat as a value.
    plain_int: bool,
}

/// Read one `01` automatic-variable record. `None` on any deviation, including a
/// `07` function-scope static, whose layout is not characterized.
fn read_record(sy: &[u8], at: usize, section_kind: u8) -> Option<(SyRecord, usize)> {
    let mut p = at;
    if *sy.get(p)? != REC_AUTOMATIC {
        return None;
    }
    p += 1;
    // The record repeats its section's kind. Both readings of the original dump
    // fit the bytes — record-carried kind, or a copy of the section marker — so
    // requiring agreement costs nothing and refuses a stream where they disagree
    // instead of silently preferring one.
    if *sy.get(p)? != section_kind {
        return None;
    }
    p += 1;
    let (tok, w) = read_token_var(sy, p)?;
    p += w;
    if *sy.get(p)? != 0x00 {
        return None;
    }
    p += 1;
    // The name is read only to bound the record — it is never used. A local's
    // source name has no bearing on what codegen may do with it, and binding on
    // it would reintroduce name-shaped guessing.
    let name_end = sy
        .iter()
        .enumerate()
        .skip(p)
        .take(MAX_NAME)
        .find(|&(_, &b)| b == 0x00)
        .map(|(i, _)| i)?;
    if name_end == p {
        return None;
    }
    p = name_end + 1;

    if *sy.get(p)? != TYPE_REF {
        return None;
    }
    let ty = *sy.get(p + 1)?;
    if *sy.get(p + 2)? != 0x00 {
        return None;
    }
    // `<n>`: `03` in every formal and `01` in every local across all witnesses,
    // so it is read past rather than checked — a constant field asserted as a
    // field is a claim this module cannot support.
    let _n = *sy.get(p + 3)?;
    if sy.get(p + 4..p + 7)? != REC_MID {
        return None;
    }
    let flags = *sy.get(p + 7)?;
    if *sy.get(p + 8)? != 0x00 {
        return None;
    }
    p += 9;

    // The inline type re-encoding. Its width is what lets the reader step over a
    // record whose type it refuses, so each accepted form is a *measured* width,
    // never `80` plus a guessed payload length: after the step the caller
    // immediately requires the next byte to be a record or a section marker, so
    // a wrong width fails the file closed rather than desyncing.
    let (enc_width, enc_int) = match *sy.get(p)? {
        ENC_INT => (1, true),
        // `unsigned`.
        0x75 => (1, false),
        // Qualified (`const`/`volatile`) or pointer: a 4-byte payload in every
        // witness.
        0x80 => (5, false),
        _ => return None,
    };
    sy.get(p..p + enc_width)?;
    p += enc_width;

    let plain_int = ty == TY_INT && enc_int && flags == FLAG_PLAIN;
    Some((SyRecord { tok, plain_int }, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `int f(int a) { int x = a + 1; int y = x + 2; return y; }`, transcribed
    /// verbatim from `c2rs census --keep-il`.
    const LOC2: &[u8] = &[
        0x03, 0x01, 0xe5, 0x09, 0x1f, 0x00, 0x01, 0x01, 0x0d, 0x01, 0x01, 0x01, 0xe3, 0x09, 0x00,
        b'a', 0x00, 0x86, 0x01, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d, 0x02, 0x01,
        0x02, 0xe7, 0x09, 0x00, b'y', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04, 0x04, 0x00, 0x01, 0x00,
        0x74, 0x01, 0x02, 0xe6, 0x09, 0x00, b'x', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04, 0x04, 0x00,
        0x01, 0x00, 0x74, 0x06,
    ];

    /// `int nothing() { return 7; }` — both sections present, both empty.
    const EMPTY: &[u8] = &[
        0x03, 0x01, 0xe7, 0x09, 0x1f, 0x00, 0x01, 0x01, 0x0d, 0x01, 0x0d, 0x02, 0x06,
    ];

    /// One `int` local record with the type/flag/encoding fields parameterized,
    /// so a single-field change is the only difference between two probes.
    fn local_rec(ty: u8, flags: u8, enc: &[u8]) -> Vec<u8> {
        let mut v = vec![0x01, 0x02, 0xe6, 0x09, 0x00, b'x', 0x00, TYPE_REF, ty, 0x00, 0x01];
        v.extend_from_slice(&REC_MID);
        v.push(flags);
        v.push(0x00);
        v.extend_from_slice(enc);
        v
    }

    fn one_block(locals: &[Vec<u8>]) -> Vec<u8> {
        let mut v = vec![0x03, 0x01, 0xe5, 0x09];
        v.extend_from_slice(&BLOCK_HEADER_TAIL);
        v.extend_from_slice(&[SECTION, SECTION_FORMALS, SECTION, SECTION_LOCALS]);
        for l in locals {
            v.extend_from_slice(l);
        }
        v.push(BLOCK_CLOSE);
        v
    }

    #[test]
    fn a_formal_and_two_int_locals_are_told_apart() {
        let b = sy_blocks(LOC2).expect("measured capture must parse");
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].formals, vec![0xe309]);
        // Unordered by contract, but pin the file order so a reordering shows up.
        assert_eq!(b[0].int_locals, vec![0xe709, 0xe609]);
    }

    #[test]
    fn both_sections_may_be_empty() {
        let b = sy_blocks(EMPTY).expect("empty sections are a real capture");
        assert_eq!(b, vec![SyBlock::default()]);
    }

    #[test]
    fn blocks_are_counted_across_a_multi_function_file() {
        let mut two = LOC2.to_vec();
        two.extend_from_slice(EMPTY);
        let b = sy_blocks(&two).unwrap();
        assert_eq!(b.len(), 2);
        assert_eq!(b[1].int_locals.len(), 0);
    }

    /// The load-bearing discriminator: same type, same name, same record —
    /// address-taken is the ONLY difference, and it must not be admitted.
    #[test]
    fn an_address_taken_local_is_refused_but_its_neighbour_is_not() {
        let plain = sy_blocks(&one_block(&[local_rec(TY_INT, 0x01, &[ENC_INT])])).unwrap();
        let taken = sy_blocks(&one_block(&[local_rec(TY_INT, 0x21, &[ENC_INT])])).unwrap();
        assert_eq!(plain[0].int_locals, vec![0xe609]);
        assert!(taken[0].int_locals.is_empty());
    }

    /// `<T>` alone does not separate `int` from `volatile int` — both are
    /// `86 01`. Gating on it would fold away a volatile store.
    #[test]
    fn a_qualified_int_is_refused_despite_an_int_type_ref() {
        let vol = one_block(&[local_rec(TY_INT, 0x01, &[0x80, 0x00, 0x10, 0x00, 0x00])]);
        let cst = one_block(&[local_rec(TY_INT, 0x01, &[0x80, 0x01, 0x10, 0x00, 0x00])]);
        assert!(sy_blocks(&vol).unwrap()[0].int_locals.is_empty());
        assert!(sy_blocks(&cst).unwrap()[0].int_locals.is_empty());
    }

    #[test]
    fn unsigned_and_pointer_locals_are_refused() {
        let uns = one_block(&[local_rec(0x02, 0x01, &[0x75])]);
        let ptr = one_block(&[local_rec(0x03, 0x01, &[0x80, 0x74, 0x04, 0x00, 0x00])]);
        assert!(sy_blocks(&uns).unwrap()[0].int_locals.is_empty());
        assert!(sy_blocks(&ptr).unwrap()[0].int_locals.is_empty());
    }

    /// A refused local must still be *stepped over* correctly, or the local after
    /// it gets mis-bound. This is the case that a resync guess would break.
    #[test]
    fn a_refused_local_does_not_desync_the_one_after_it() {
        let mut ptr = local_rec(0x03, 0x01, &[0x80, 0x74, 0x04, 0x00, 0x00]);
        ptr[2] = 0xf0;
        let ok = local_rec(TY_INT, 0x01, &[ENC_INT]);
        let b = sy_blocks(&one_block(&[ptr, ok])).unwrap();
        assert_eq!(b[0].int_locals, vec![0xe609]);
    }

    /// A function-scope `static` is a `07` record with a mangled name and an
    /// uncharacterized layout — the whole file is refused rather than skipped.
    #[test]
    fn a_static_local_record_refuses_the_whole_file() {
        let mut v = vec![0x03, 0x01, 0xe5, 0x09];
        v.extend_from_slice(&BLOCK_HEADER_TAIL);
        v.extend_from_slice(&[SECTION, SECTION_FORMALS, SECTION, SECTION_LOCALS]);
        // Verbatim from `static int x;` inside `int s_local(int a)`.
        v.extend_from_slice(&[0x07, 0x02, 0xec, 0x09]);
        v.extend_from_slice(b"?x@?1??s_local@@YAHH@Z@4HA\x00");
        v.extend_from_slice(&[0x86, 0x01, 0x00, 0x04, 0x04, 0x00, 0x74, 0xed, 0x09, 0x00, 0x06]);
        assert_eq!(sy_blocks(&v), None);
    }

    #[test]
    fn a_truncated_or_unterminated_file_refuses() {
        assert_eq!(sy_blocks(&LOC2[..LOC2.len() - 1]), None);
        assert_eq!(sy_blocks(&LOC2[..12]), None);
        // A block that never closes.
        let mut no_close = LOC2.to_vec();
        let n = no_close.len();
        no_close[n - 1] = 0x01;
        assert_eq!(sy_blocks(&no_close), None);
    }

    #[test]
    fn a_missing_section_marker_refuses() {
        let mut v = one_block(&[local_rec(TY_INT, 0x01, &[ENC_INT])]);
        // Drop the locals section marker, leaving the record where a `0D` must be.
        v.remove(10);
        v.remove(10);
        assert_eq!(sy_blocks(&v), None);
    }

    #[test]
    fn a_record_disagreeing_with_its_section_refuses() {
        let mut rec = local_rec(TY_INT, 0x01, &[ENC_INT]);
        rec[1] = SECTION_FORMALS;
        assert_eq!(sy_blocks(&one_block(&[rec])), None);
    }

    #[test]
    fn an_empty_file_is_zero_blocks_not_a_refusal() {
        assert_eq!(sy_blocks(&[]), Some(Vec::new()));
    }
}

