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
//!   ( 03 03 <tok> <u16 LE> 01 <b> )*   label declarations, for the block below
//!   03 01 <tok> 1F 00 01 01            block open
//!   ( 0D <depth> <record>* )+          one group per lexical scope, preorder
//!   06                                 block close
//! ```
//!
//! Depth 1 is the formals, 2 the function body, 3 and up each nested brace; the
//! first two groups are always present and either may be empty. A record is
//!
//! ```text
//!   01 <depth> <tok> 00 <name> 00 <type>              a plain variable
//!   02 <depth> <tok> 00 <name> 00 <type> <elemsize>   an array
//!   07 <depth> <tok> <mangled> 00 …                   a function-scope static
//!   type := 86 <kind> 00 <cls> 04 <size16 LE> <flags16 LE> <tid>
//! ```
//!
//! where `<cls>` is `03` for a formal and `01` for an automatic, `<flags>` bit 0
//! is *referenced* and bit 5 *address-taken*, and `<tid>` is an `.ex` type-table
//! id — one byte below `0x80`, else `80` and a 32-bit id, with qualified, pointer
//! and array types living above `0x1000`.
//!
//! # What is measured, and what is only constant
//!
//! MEASURED, each against a neighbour that would look identical under a plausible
//! wrong rule (probe sources in `fixtures/cpp/il_sy_locals*.cpp`):
//!
//! * `<kind> = 01` with `<size16> = 4` and `<tid> = 74` is plain `int`. `const`
//!   and `volatile` leave the kind at `01` and move only the id, to `0x1001` and
//!   `0x1000` — so a gate on the kind admits a `volatile int` local and folds away
//!   a store that must not be folded. The id is what this reader requires.
//! * `<flags>` is `0001` normally and `0021` when the address is taken. The
//!   discriminating probe is one function with two `int` locals where only one has
//!   `&x` applied; without that neighbour "always 0001" and "0001 by coincidence"
//!   are the same observation.
//! * The byte after a record tag is the **scope depth**, not a formal/local kind.
//!   A local declared inside a brace reads `01 03`, so testing that byte for `02`
//!   silently drops every local in a loop or nested block — which is most of them
//!   in real code. Locals are records at depth ≥ 2, and the depth is cross-checked
//!   against `<cls>`.
//! * `03 03 …` label declarations precede the block that uses them and have the
//!   same width as a block header, told apart only by the byte after `03`. Reading
//!   one as a header refuses every function with control flow.
//! * A function-scope `static` is a `07` record: a memory object, carrying its
//!   fully mangled name, no NUL between token and name, and a second token — the
//!   one the body actually loads. This is the `$sv` hazard a second time, and the
//!   record tag is what separates them.
//! * File-scope `static`, plain globals and `extern` declarations appear in `.sy`
//!   **not at all**. That is what makes a *membership* test here sound where the
//!   `.gl` absence test was not.
//!
//! CONSTANT ACROSS EVERY WITNESS, and therefore not interpreted — required
//! literally so a deviation fails the file closed rather than being read as a
//! field this module claims to understand: the `04` between `<cls>` and
//! `<size16>`.
//!
//! NOT constant, though it looked it: the block header's four-byte tail reads
//! `1F 00 01 01` in every fixture and `1F 00 02 01` in a real translation unit, so
//! it is stepped over by width and never checked. It is the module's own instance
//! of the rule in `docs/GAPS.md` §6 — a field that never varies is
//! indistinguishable from a constant — and it was caught only by running the
//! reader against a 649 KB workload capture instead of the probes it was written
//! from.
//!
//! NOT DERIVABLE from what has been captured: the block's own `<tok>`, which is
//! not the function's `.gl` symbol token (it coincides with the function's
//! exit-label token in every probe, which is an observation, not a use); and the
//! record order within a scope, which is neither declaration order nor its reverse
//! — `y, x` in one probe and `p, q, r` in a structurally identical other — so
//! locals are treated as an unordered set. Nothing here depends on either.
//!
//! # Fail-closed shape
//!
//! [`sy_blocks`] returns `None` for the **whole file** the moment it meets
//! anything it has not measured. Over-refusal at translation-unit granularity is
//! the intended cost; the alternative is resynchronizing on a guess, and a wrong
//! guess rebinds a token and mis-emits. That is also why an array and a
//! function-scope static are *located and refused* rather than scanned past: their
//! widths are measured, so a record after one of them still binds correctly.

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
/// A record between function blocks, sharing the block header's `03` lead and its
/// width. Kind `03` is a label declaration, `06` another form seen only in real
/// translation units; only kind `01` opens a block. Skipping these is what lets a
/// function with any control flow reach its own local sections at all — and
/// skipping one that *did* open a block cannot go unnoticed, because the block
/// count then disagrees with the `.ex` segment count and nothing is bound.
const REC_INTER_BLOCK: u8 = 0x03;
/// The four bytes after a block's or label's token. **Not interpreted**, only
/// stepped over: they read `1F 00 01 01` in every fixture and `1F 00 02 01` in a
/// real translation unit, so requiring any of them literally refuses real input —
/// the trap of reading a never-varied field as a constant, caught only by running
/// this against a 649 KB workload `.sy` rather than the probes it was written from.
/// A block header and a label declaration share this tail, and so share a width;
/// the byte after `03` is the only thing that tells them apart.
const HEADER_TAIL_LEN: usize = 4;
const BLOCK_CLOSE: u8 = 0x06;
/// Opens a lexical scope's variable group: `0D <depth>`, preorder. Depth 1 is the
/// formals, 2 the function body, 3+ each nested brace.
const SECTION: u8 = 0x0D;
const DEPTH_FORMALS: u8 = 0x01;
/// A plain automatic variable.
const REC_PLAIN: u8 = 0x01;
/// An array. Same fields plus a trailing element size; never admitted, only
/// stepped over.
const REC_ARRAY: u8 = 0x02;
/// A function-scope `static` — a memory object with a relocation, carrying its
/// fully mangled name and a second token. Never admitted, only stepped over.
const REC_STATIC: u8 = 0x07;
/// The type tag of the 4-byte scalar family. **Not** a constant across the file —
/// an 8-byte type reads `88`, so the tag is read as part of the type and only
/// *admission* requires this value; the region's width does not depend on it.
const TYPE_TAG: u8 = 0x86;
const TYPE_KIND_INT: u8 = 0x01;
/// Storage class: `01` automatic, `03` formal. Redundant with the section depth in
/// every witness, and required to agree with it.
const CLS_AUTOMATIC: u8 = 0x01;
const CLS_FORMAL: u8 = 0x03;
/// Constant across every witness, between the storage class and the size.
const SIZE_LEAD: u8 = 0x04;
const SIZEOF_INT: u16 = 4;
/// `.ex` type-table id of plain `int`.
const TID_INT: u32 = 0x74;
/// Flags bit 0 is *referenced* and bit 5 is *address-taken*, so `0x0001` is an
/// ordinary variable and `0x0021` one whose address escapes. `0x0000` — seen on an
/// unreferenced formal — is accepted for locals too: an unread variable cannot
/// change what the return expression evaluates to. Every other bit pattern is
/// refused, because a bit this module cannot name might mean escape as well.
const FLAGS_REFERENCED: u16 = 0x0001;
const FLAGS_NONE: u16 = 0x0000;
/// A `.sy` name is an identifier or a mangled name; the bound keeps a corrupt
/// stream from scanning the rest of the file for a NUL.
const MAX_NAME: usize = 4096;
/// Refuse absurd files rather than allocating against a length read from data.
const MAX_BLOCKS: usize = 65536;
/// A lexical nesting depth past this is not a real function.
const MAX_DEPTH: u8 = 64;

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
        // Inter-block declarations belong to the block that follows. Their tokens are
        // read past, never recorded: a label is not a value, so it can never be an
        // assignment destination.
        while *sy.get(p)? == REC_INTER_BLOCK && *sy.get(p + 1)? != BLOCK_OPEN[1] {
            let (_tok, w) = read_token_var(sy, p + 2)?;
            // `03 03 <tok> <tail:4>` — the same width as a block header, which is
            // why the byte after `03` is the only thing separating them.
            p += 2 + w + HEADER_TAIL_LEN;
        }
        if sy.get(p..p + 2)? != BLOCK_OPEN {
            return None;
        }
        p += 2;
        // The block token is read past, not interpreted. It is not the function's
        // `.gl` symbol token; it coincides with the function's exit-label token in
        // every probe, which is an observation and not something anything needs.
        let (_hdr_tok, w) = read_token_var(sy, p)?;
        p += w;
        // The tail is stepped over, not checked. What validates the block is what
        // must follow it: a `0D <depth>` group whose records all parse, and a `06`
        // that closes it.
        sy.get(p..p + HEADER_TAIL_LEN)?;
        p += HEADER_TAIL_LEN;

        let mut block = SyBlock::default();
        // At least the formals and body sections are always present, either may be
        // empty, and a nested brace adds one more. Depths arrive in preorder, so
        // they are not required to increase — sibling scopes reuse a depth — only
        // to be a plausible nesting level.
        let mut seen_sections = 0usize;
        while *sy.get(p)? == SECTION {
            let depth = *sy.get(p + 1)?;
            if depth == 0 || depth > MAX_DEPTH {
                return None;
            }
            p += 2;
            seen_sections += 1;
            loop {
                let (rec, next) = match *sy.get(p)? {
                    REC_PLAIN | REC_ARRAY | REC_STATIC => read_record(sy, p, depth)?,
                    _ => break,
                };
                p = next;
                match rec {
                    // A formal is recorded whatever its type: `parse_formals`
                    // already establishes those from `.ex`, and this list only
                    // cross-checks. Only locals gate on the type.
                    Some(tok) if depth == DEPTH_FORMALS => block.formals.push(tok),
                    Some(tok) => block.int_locals.push(tok),
                    None => {}
                }
            }
        }
        if seen_sections < 2 {
            return None;
        }
        if *sy.get(p)? != BLOCK_CLOSE {
            return None;
        }
        p += 1;
        out.push(block);
    }
    Some(out)
}

/// Read one variable record, returning its token when the record is one a
/// value-substituting parse may fold — a plain, unqualified, 4-byte `int` whose
/// address never escapes — and `None` when the record is merely *stepped over*.
///
/// The distinction matters more than it looks: a record this reader cannot step
/// over exactly would desync and rebind every later token, so the two outcomes are
/// "admitted" and "located but refused", never "skipped by scanning".
fn read_record(sy: &[u8], at: usize, depth: u8) -> Option<(Option<u32>, usize)> {
    let mut p = at;
    let tag = *sy.get(p)?;
    p += 1;
    // The record repeats its scope's depth. Requiring agreement costs nothing and
    // refuses a stream where the two disagree rather than silently trusting one.
    if *sy.get(p)? != depth {
        return None;
    }
    p += 1;
    let (tok, w) = read_token_var(sy, p)?;
    p += w;
    // A plain or array record has one byte between the token and the name; a static
    // record has none and runs straight into its mangled name. That byte is **not**
    // interpreted: it is `00` on every ordinary variable and `26` on a
    // compiler-generated formal (`__flags`, from the exception-handling regime), so
    // requiring a NUL refuses real translation units.
    if tag != REC_STATIC {
        sy.get(p)?;
        p += 1;
    }
    // The name is read only to bound the record — never to bind anything. A
    // variable's source name has no bearing on what codegen may do with it.
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

    // A `static` carries a different, shorter field region and a second token —
    // the one the body actually loads. It is a memory object either way, so it is
    // located and refused; only its width is needed.
    if tag == REC_STATIC {
        // `<tag> <kind> 00 <size> 04 00 <tid> <tok'> <b>`, where `<size>` is a
        // varint and not a byte: `static int k` writes `04`, and
        // `static int primes[62]` writes `80 F8 00 00 00` for 248. Reading it as a
        // byte refused `Primes.cpp` — a translation unit one function short of
        // matching — for a record this reader only ever steps over.
        //
        // `<tok'>` is a second token, the one the body actually loads, and `<b>` is
        // an element size (`04` for that array, `00` for the scalar). Neither is
        // interpreted: a `static` is a memory object with a relocation whatever its
        // type, so the only thing needed here is the width.
        sy.get(p)?;
        if *sy.get(p + 2)? != 0x00 {
            return None;
        }
        p += 3;
        let (_size, sw) = read_tid(sy, p)?;
        p += sw;
        if sy.get(p..p + 2)? != [SIZE_LEAD, 0x00] {
            return None;
        }
        p += 2;
        let (_tid, tw) = read_tid(sy, p)?;
        p += tw;
        let (_body_tok, bw) = read_token_var(sy, p)?;
        p += bw;
        sy.get(p)?;
        return Some((None, p + 1));
    }

    // `<tag> <kind> 00 <cls> 04 <size16 LE> <flags16 LE> <tid>`. Read as fields
    // rather than matched as a run: the earlier draft required `04 04 00`
    // literally, which happened to be correct only because it pinned
    // `size16 == 4` — true of `int` and of nothing wider.
    let type_tag = *sy.get(p)?;
    let kind = *sy.get(p + 1)?;
    if *sy.get(p + 2)? != 0x00 {
        return None;
    }
    let cls = *sy.get(p + 3)?;
    if *sy.get(p + 4)? != SIZE_LEAD {
        return None;
    }
    let size = u16::from_le_bytes([*sy.get(p + 5)?, *sy.get(p + 6)?]);
    let flags = u16::from_le_bytes([*sy.get(p + 7)?, *sy.get(p + 8)?]);
    p += 9;
    let (tid, tw) = read_tid(sy, p)?;
    p += tw;
    // An array's element size trails the type region.
    if tag == REC_ARRAY {
        sy.get(p)?;
        p += 1;
    }
    // The storage class must agree with the scope depth, for the same reason the
    // record's depth byte must: two channels saying the same thing are worth
    // checking against each other.
    let cls_ok = match depth {
        DEPTH_FORMALS => cls == CLS_FORMAL,
        _ => cls == CLS_AUTOMATIC,
    };
    if !cls_ok {
        return None;
    }
    if depth == DEPTH_FORMALS {
        return Some((Some(tok), p));
    }
    // `const` and `volatile` do not change `<kind>`; they move `<tid>` into the
    // constructed-type range, which is why the id is checked and not just the
    // kind. Bit 5 of the flags is address-taken.
    let admissible = tag == REC_PLAIN
        && type_tag == TYPE_TAG
        && kind == TYPE_KIND_INT
        && size == SIZEOF_INT
        && tid == TID_INT
        && (flags == FLAGS_REFERENCED || flags == FLAGS_NONE);
    Some((admissible.then_some(tok), p))
}

/// A `.ex` type-table id: one byte below `0x80`, or `80` and a 32-bit
/// little-endian id. Qualified, pointer and array types live in the constructed
/// range above `0x1000` and always take the wide form, so the width is measured
/// rather than guessed — `const int` is `80 01 10 00 00` and `volatile int`
/// `80 00 10 00 00`, against plain `int`'s bare `74`.
fn read_tid(sy: &[u8], at: usize) -> Option<(u32, usize)> {
    let b = *sy.get(at)?;
    if b < 0x80 {
        return Some((b as u32, 1));
    }
    if b != 0x80 {
        return None;
    }
    let w: [u8; 4] = sy.get(at + 1..at + 5)?.try_into().ok()?;
    Some((u32::from_le_bytes(w), 5))
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

    /// `int nested(int a) { int x = a + 1; { int y = x + 2; return y; } }` — the
    /// brace-scoped `y` sits in a third section at depth 3, with a record whose
    /// own depth byte is `03`. Verbatim capture.
    const NESTED: &[u8] = &[
        0x03, 0x01, 0xe5, 0x09, 0x1f, 0x00, 0x01, 0x01, 0x0d, 0x01, 0x01, 0x01, 0xe3, 0x09, 0x00,
        b'a', 0x00, 0x86, 0x01, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d, 0x02, 0x01,
        0x02, 0xe6, 0x09, 0x00, b'x', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04, 0x04, 0x00, 0x01, 0x00,
        0x74, 0x0d, 0x03, 0x01, 0x03, 0xe7, 0x09, 0x00, b'y', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04,
        0x04, 0x00, 0x01, 0x00, 0x74, 0x06,
    ];

    /// The three label declarations `int looped(int a) { int s = 0; for (int i = 0;
    /// i < a; i++) { s = s + i; } return s; }` emits ahead of its own block, then
    /// that block: `s` at depth 2, `i` at depth 3, and an empty depth-4 section for
    /// the loop body's braces. Verbatim capture.
    const LOOPED: &[u8] = &[
        0x03, 0x03, 0xef, 0x09, 0x0b, 0x00, 0x01, 0x01, 0x03, 0x03, 0xee, 0x09, 0x0c, 0x00, 0x01,
        0x02, 0x03, 0x03, 0xed, 0x09, 0x08, 0x00, 0x01, 0x01, 0x03, 0x01, 0xea, 0x09, 0x1f, 0x00,
        0x01, 0x01, 0x0d, 0x01, 0x01, 0x01, 0xe8, 0x09, 0x00, b'a', 0x00, 0x86, 0x01, 0x00, 0x03,
        0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d, 0x02, 0x01, 0x02, 0xeb, 0x09, 0x00, b's', 0x00,
        0x86, 0x01, 0x00, 0x01, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d, 0x03, 0x01, 0x03, 0xec,
        0x09, 0x00, b'i', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d,
        0x04, 0x06,
    ];

    /// `int arr(int a) { int v[4]; v[0] = a; return v[0]; }` — an `02` record whose
    /// `size16` is `0x0010` (four ints), whose type id is in the constructed range,
    /// and which carries a trailing element size. Verbatim capture.
    const ARRAY: &[u8] = &[
        0x03, 0x01, 0xf2, 0x09, 0x1f, 0x00, 0x01, 0x01, 0x0d, 0x01, 0x01, 0x01, 0xf0, 0x09, 0x00,
        b'a', 0x00, 0x86, 0x01, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d, 0x02, 0x02,
        0x02, 0xf3, 0x09, 0x00, b'v', 0x00, 0x86, 0x06, 0x00, 0x01, 0x04, 0x10, 0x00, 0x01, 0x00,
        0x80, 0x00, 0x10, 0x00, 0x00, 0x04, 0x06,
    ];

    /// `int stat(int a) { static int k; k = a; return k; }` — a `07` record with a
    /// mangled name, no NUL between token and name, and a second token (the one the
    /// body loads). Verbatim capture.
    const STATIC: &[u8] = &[
        0x03, 0x01, 0xf6, 0x09, 0x1f, 0x00, 0x01, 0x01, 0x0d, 0x01, 0x01, 0x01, 0xf4, 0x09, 0x00,
        b'a', 0x00, 0x86, 0x01, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x74, 0x0d, 0x02, 0x07,
        0x02, 0xf7, 0x09, b'?', b'k', b'@', b'?', b'1', b'?', b'?', b's', b't', b'a', b't', b'@',
        b'@', b'Y', b'A', b'H', b'H', b'@', b'Z', b'@', b'4', b'H', b'A', 0x00, 0x86, 0x01, 0x00,
        0x04, 0x04, 0x00, 0x74, 0xf8, 0x09, 0x00, 0x06,
    ];

    /// One depth-2 `int` record with the type fields parameterized, so a single
    /// field is the only difference between two probes.
    fn local_rec(kind: u8, size: u16, flags: u16, tid: &[u8]) -> Vec<u8> {
        let mut v = vec![REC_PLAIN, 0x02, 0xe6, 0x09, 0x00, b'x', 0x00, TYPE_TAG, kind, 0x00];
        v.push(CLS_AUTOMATIC);
        v.push(SIZE_LEAD);
        v.extend_from_slice(&size.to_le_bytes());
        v.extend_from_slice(&flags.to_le_bytes());
        v.extend_from_slice(tid);
        v
    }

    fn int_rec() -> Vec<u8> {
        local_rec(TYPE_KIND_INT, SIZEOF_INT, FLAGS_REFERENCED, &[TID_INT as u8])
    }

    fn one_block(locals: &[Vec<u8>]) -> Vec<u8> {
        let mut v = vec![0x03, 0x01, 0xe5, 0x09];
        v.extend_from_slice(&[0x1F, 0x00, 0x01, 0x01]);
        v.extend_from_slice(&[SECTION, DEPTH_FORMALS, SECTION, 0x02]);
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

    /// The refutation that mattered: the byte after the record tag is the scope
    /// DEPTH, not a formal/local kind. A reader that tested it for `02` drops every
    /// local declared inside a brace — here, `y`.
    #[test]
    fn a_brace_scoped_local_is_admitted_at_depth_three() {
        let b = sy_blocks(NESTED).unwrap();
        assert_eq!(b[0].formals, vec![0xe309]);
        assert_eq!(b[0].int_locals, vec![0xe609, 0xe709]);
    }

    /// Label declarations sit between blocks and would otherwise be read as a
    /// malformed block header, refusing every function with control flow.
    #[test]
    fn labels_before_a_block_are_stepped_over() {
        let b = sy_blocks(LOOPED).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].formals, vec![0xe809]);
        assert_eq!(b[0].int_locals, vec![0xeb09, 0xec09]);
    }

    #[test]
    fn an_array_local_is_located_and_refused() {
        let b = sy_blocks(ARRAY).unwrap();
        assert_eq!(b[0].formals, vec![0xf009]);
        assert!(b[0].int_locals.is_empty());
    }

    #[test]
    fn a_function_scope_static_is_located_and_refused() {
        let b = sy_blocks(STATIC).unwrap();
        assert_eq!(b[0].formals, vec![0xf409]);
        assert!(b[0].int_locals.is_empty());
    }

    /// `static int primes[62]` inside a function: the size field is a varint
    /// (`80 F8 00 00 00` = 248), not the byte the scalar case shows, and the local
    /// after it must still bind. Verbatim from `Primes.cpp`, a workload TU one
    /// function short of matching — which this record refused outright.
    #[test]
    fn a_static_array_local_is_stepped_over_and_the_next_local_still_binds() {
        let mut v = vec![0x03, 0x01, 0xe8, 0x09, 0x1f, 0x00, 0x02, 0x01];
        v.extend_from_slice(&[SECTION, DEPTH_FORMALS]);
        v.extend_from_slice(&[
            0x01, 0x01, 0xe6, 0x09, 0x00, b'i', 0x00, 0x86, 0x01, 0x00, 0x03, 0x04, 0x04, 0x00,
            0x01, 0x00, 0x74,
        ]);
        v.extend_from_slice(&[SECTION, 0x02]);
        v.extend_from_slice(&[0x07, 0x02, 0xe9, 0x09]);
        v.extend_from_slice(b"?primes@?1??NextHashPrime@@YAHH@Z@4PAHA\x00");
        v.extend_from_slice(&[
            0x86, 0x06, 0x00, 0x80, 0xf8, 0x00, 0x00, 0x00, 0x04, 0x00, 0x80, 0x00, 0x10, 0x00,
            0x00, 0xea, 0x09, 0x04,
        ]);
        v.extend_from_slice(&[SECTION, 0x03]);
        v.extend_from_slice(&[
            0x01, 0x03, 0xeb, 0x09, 0x00, b'i', b'2', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04, 0x04,
            0x00, 0x01, 0x00, 0x74,
        ]);
        v.extend_from_slice(&[SECTION, 0x04, SECTION, 0x05, SECTION, 0x06, BLOCK_CLOSE]);
        let b = sy_blocks(&v).expect("Primes.cpp must parse");
        assert_eq!(b[0].formals, vec![0xe609]);
        assert_eq!(b[0].int_locals, vec![0xeb09]);
    }

    #[test]
    fn blocks_are_counted_across_a_multi_function_file() {
        let mut all = LOC2.to_vec();
        all.extend_from_slice(EMPTY);
        all.extend_from_slice(LOOPED);
        let b = sy_blocks(&all).unwrap();
        assert_eq!(b.len(), 3);
        assert_eq!(b[2].int_locals, vec![0xeb09, 0xec09]);
    }

    /// The load-bearing discriminator: same type, same size, same name, same
    /// record — the address-taken bit is the ONLY difference.
    #[test]
    fn an_address_taken_local_is_refused_but_its_neighbour_is_not() {
        let plain = sy_blocks(&one_block(&[int_rec()])).unwrap();
        let taken = sy_blocks(&one_block(&[local_rec(
            TYPE_KIND_INT,
            SIZEOF_INT,
            0x0021,
            &[TID_INT as u8],
        )]))
        .unwrap();
        assert_eq!(plain[0].int_locals, vec![0xe609]);
        assert!(taken[0].int_locals.is_empty());
    }

    /// `<kind>` alone does not separate `int` from `volatile int` — both are
    /// `86 01`, and only the type id moves. Gating on the kind would fold away a
    /// volatile store.
    #[test]
    fn a_qualified_int_is_refused_despite_an_int_kind() {
        for tid in [
            [0x80, 0x00, 0x10, 0x00, 0x00], // volatile int
            [0x80, 0x01, 0x10, 0x00, 0x00], // const int
        ] {
            let b = one_block(&[local_rec(TYPE_KIND_INT, SIZEOF_INT, FLAGS_REFERENCED, &tid)]);
            assert!(sy_blocks(&b).unwrap()[0].int_locals.is_empty());
        }
    }

    #[test]
    fn a_wider_or_differently_kinded_local_is_refused() {
        // `unsigned` — a different kind and type id, same 4-byte size.
        let uns = one_block(&[local_rec(0x02, SIZEOF_INT, FLAGS_REFERENCED, &[0x75])]);
        // An `int`-kinded record claiming 8 bytes: the size is read, not assumed.
        let wide = one_block(&[local_rec(TYPE_KIND_INT, 8, FLAGS_REFERENCED, &[TID_INT as u8])]);
        assert!(sy_blocks(&uns).unwrap()[0].int_locals.is_empty());
        assert!(sy_blocks(&wide).unwrap()[0].int_locals.is_empty());
    }

    /// An unknown flag bit might mean escape too, so only the two measured words
    /// are accepted.
    #[test]
    fn an_unknown_flag_bit_is_refused() {
        let b = one_block(&[local_rec(TYPE_KIND_INT, SIZEOF_INT, 0x0041, &[TID_INT as u8])]);
        assert!(sy_blocks(&b).unwrap()[0].int_locals.is_empty());
    }

    /// A refused record must still be stepped over exactly, or the record after it
    /// gets mis-bound. This is the case a resync guess breaks.
    #[test]
    fn a_refused_record_does_not_desync_the_one_after_it() {
        let mut vol = local_rec(
            TYPE_KIND_INT,
            SIZEOF_INT,
            FLAGS_REFERENCED,
            &[0x80, 0x00, 0x10, 0x00, 0x00],
        );
        vol[2] = 0xf0;
        let b = sy_blocks(&one_block(&[vol, int_rec()])).unwrap();
        assert_eq!(b[0].int_locals, vec![0xe609]);
    }

    #[test]
    fn a_record_disagreeing_with_its_scope_depth_refuses() {
        let mut rec = int_rec();
        rec[1] = 0x03;
        assert_eq!(sy_blocks(&one_block(&[rec])), None);
    }

    /// The storage class is a second channel for the same fact, and must agree.
    #[test]
    fn a_storage_class_disagreeing_with_the_depth_refuses() {
        let mut rec = int_rec();
        rec[10] = CLS_FORMAL;
        assert_eq!(sy_blocks(&one_block(&[rec])), None);
    }

    #[test]
    fn a_truncated_or_unterminated_file_refuses() {
        assert_eq!(sy_blocks(&LOC2[..LOC2.len() - 1]), None);
        assert_eq!(sy_blocks(&LOC2[..12]), None);
        let mut no_close = LOC2.to_vec();
        let n = no_close.len();
        no_close[n - 1] = 0x01;
        assert_eq!(sy_blocks(&no_close), None);
    }

    #[test]
    fn a_block_with_fewer_than_two_sections_refuses() {
        let mut v = vec![0x03, 0x01, 0xe5, 0x09];
        v.extend_from_slice(&[0x1F, 0x00, 0x01, 0x01]);
        v.extend_from_slice(&[SECTION, DEPTH_FORMALS]);
        v.push(BLOCK_CLOSE);
        assert_eq!(sy_blocks(&v), None);
    }

    #[test]
    fn an_unknown_record_tag_refuses() {
        let mut rec = int_rec();
        rec[0] = 0x05;
        assert_eq!(sy_blocks(&one_block(&[rec])), None);
    }

    /// Wrap a verbatim record capture in the smallest legal block.
    fn block_with(depth: u8, rec: &[u8]) -> Vec<u8> {
        let mut v = vec![0x03, 0x01, 0xe5, 0x09, 0x1F, 0x00, 0x01, 0x01];
        v.extend_from_slice(&[SECTION, DEPTH_FORMALS, SECTION, 0x02]);
        if depth != DEPTH_FORMALS {
            v.extend_from_slice(rec);
        }
        v.push(BLOCK_CLOSE);
        if depth == DEPTH_FORMALS {
            // Put the record in the formals group instead.
            let mut w = vec![0x03, 0x01, 0xe5, 0x09, 0x1F, 0x00, 0x01, 0x01];
            w.extend_from_slice(&[SECTION, DEPTH_FORMALS]);
            w.extend_from_slice(rec);
            w.extend_from_slice(&[SECTION, 0x02, BLOCK_CLOSE]);
            return w;
        }
        v
    }

    /// An 8-byte type reads tag `88`, so the tag is not constant and the region's
    /// width must not depend on it. Verbatim from a real translation unit.
    #[test]
    fn an_eight_byte_typed_formal_parses_and_is_not_mistaken_for_int() {
        let rec = &[
            0x01, 0x01, 0xcd, 0x0c, 0x00, b'x', 0x00, 0x88, 0x01, 0x00, 0x03, 0x04, 0x08, 0x00,
            0x01, 0x00, 0x13,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![0xcd0c]);
        assert!(b[0].int_locals.is_empty());
    }

    /// The byte between token and name is `26` on a compiler-generated formal, so
    /// requiring a NUL there refuses real input. Verbatim from a real TU.
    #[test]
    fn a_compiler_generated_formal_with_a_nonzero_pre_name_byte_parses() {
        let mut rec = vec![0x01, 0x01, 0xf9, 0x15, 0x26];
        rec.extend_from_slice(b"__flags\x00");
        rec.extend_from_slice(&[0x86, 0x02, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x75]);
        let b = sy_blocks(&block_with(DEPTH_FORMALS, &rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![0xf915]);
    }

    /// An aggregate inserts `80 00` ahead of the type id — and `80 00 10 00 00` is
    /// also exactly how a wide id encodes `volatile int`, so the two cannot be told
    /// apart without knowing the kind. The file is refused rather than read on a
    /// guess; this is the measured limit of the reader. Verbatim from a real TU.
    #[test]
    fn an_aggregate_typed_local_refuses_the_file() {
        let rec = &[
            0x01, 0x02, 0x77, 0x28, 0x00, b'v', b'p', 0x00, 0x86, 0x16, 0x00, 0x01, 0x04, 0x04,
            0x00, 0x81, 0x00, 0x80, 0x00, 0x80, 0x97, 0x13, 0x00, 0x00,
        ];
        assert_eq!(sy_blocks(&block_with(0x02, rec)), None);
    }

    /// `.sy` carries inter-block record kinds beyond the block header and the label
    /// (`03 06`, and a wider `1A` form). Any `03 <kind>` other than `01` is stepped
    /// over at the shared width; an unrecognized leading byte refuses. Verbatim
    /// from a real TU.
    #[test]
    fn other_inter_block_records_are_stepped_over_and_unknown_ones_refuse() {
        let mut with_06 = vec![0x03, 0x06, 0xf1, 0x15, 0x1e, 0x00, 0x01, 0x01];
        with_06.extend_from_slice(EMPTY);
        assert_eq!(sy_blocks(&with_06).unwrap().len(), 1);

        let mut with_1a = vec![0x1a, 0x02, 0xa5, 0x28, 0xc6, 0x81, 0x06, 0x80];
        with_1a.extend_from_slice(EMPTY);
        assert_eq!(sy_blocks(&with_1a), None);
    }

    #[test]
    fn an_empty_file_is_zero_blocks_not_a_refusal() {
        assert_eq!(sy_blocks(&[]), Some(Vec::new()));
    }
}
