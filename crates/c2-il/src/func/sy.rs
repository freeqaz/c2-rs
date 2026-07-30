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
//! blocks, one per `.ex` function segment and in the same order, followed by a run
//! of bare `06` bytes that ends the file:
//!
//! ```text
//!   ( 03 <k!=01> <tok> <2 B | 00 <name> 00> <b> <b> )*   labels and scope markers
//!   ( 1A <b> <tok> <type prefix> <type extent> )*        an undecoded declaration
//!   03 01 <tok> 1F 00 01 01                              block open
//!   ( 0D <depth> <record>* )+                            one group per lexical scope
//!   06                                                   block close
//!   …
//!   06 06 06 …                                           file trailer, to EOF
//! ```
//!
//! Depth 1 is the formals, 2 the function body, 3 and up each nested brace; the
//! first two groups are always present and either may be empty. A record is
//!
//! ```text
//!   01 <depth> <tok> <b> <name> 00 <type>                a plain variable
//!   02 <depth> <tok> <b> <name> 00 <type> <elemsize>     an array
//!   07 <depth> <tok> <mangled> 00 <static type>          a function-scope static
//!   0B <depth> <tok> <name> 00 00 <tid>                  a typedef
//!   type := <prefix> 00 <cls> 04 <extent>
//!   prefix := <tag> <kind> | <tag|40> 81 <kind>
//!   extent := <size varint> <b> <flags16 LE> [80 00 if flags bit 7] <tid>
//! ```
//!
//! where `<cls>` is `03` for a formal and `01` for an automatic, `<flags>` bit 0
//! is *referenced* and bit 5 *address-taken*, and `<tid>` is an `.ex` type-table
//! id — one byte below `0x80`, else `80` and a 32-bit id, with qualified, pointer
//! and array types living above `0x1000`.
//!
//! # This layer had never bound on a real translation unit
//!
//! MEASURED, and it is the fact that ranked this work: of 200 workload translation
//! units, **3** parsed. The other 197 all stopped in the first few kilobytes, and
//! every stop was a width — a record whose end this reader computed one, two or four
//! bytes off, after which nothing downstream can be trusted and the whole file's
//! binding is (correctly) withheld. So the `param-width-undetermined` census key was
//! not measuring a rare construct at 567,549 functions; it was measuring this module
//! failing to parse, on essentially every input that was not a probe fixture.
//!
//! Six widths were wrong. Each is documented at the function that reads it, with its
//! witnesses; what they have in common is worth stating once, because it is a pattern
//! and not six accidents: **every one of them is a field whose two candidate
//! encodings agree on small, simple, hand-written declarations.** A varint size and a
//! `u16` size are the same bytes below 128. A wide type prefix and a narrow one are
//! the same width if you never meet a class with a vtable. A named label record and
//! an unnamed one have the same shape until a function contains a `goto`. The probe
//! corpus this module was written from could not distinguish any of them, and a green
//! probe suite therefore said nothing at all about real input. That is
//! `docs/GAPS.md` §6 again, and the countermeasure that actually worked was not more
//! probes: it was running the reader over a few hundred real `.sy` files and
//! requiring each to parse **to EOF**.
//!
//! # What is measured, and what is only constant
//!
//! MEASURED, each against a neighbour that would look identical under a plausible
//! wrong rule (probe sources in `fixtures/cpp/il_sy_locals*.cpp`):
//!
//! * `<kind> = 01` with `<size> = 4` and `<tid> = 74` is plain `int`. `const`
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
//!   one as a header refuses every function with control flow. A label that carries
//!   its source name has a *longer* record — see [`skip_inter_block`].
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
//! field this module claims to understand: the `04` between `<cls>` and the size, the
//! `81` of a wide type prefix, the `80 00` of the flags-bit-7 extra field, and the
//! `00` before a typedef's id.
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
///
/// # This count check is now the binding blocker, and it must not be loosened
///
/// MEASURED, once [`sy_blocks`] began parsing real translation units to EOF: a large
/// `.sy` has slightly MORE blocks than `.ex` has function segments — 9629 against
/// 9602, 1541 against 1540 — so the count check refuses most real files and
/// `param-width-undetermined` remains the top census blocker at 554,056 functions.
/// The surplus blocks are presumably declarations `.ex` did not emit a body for.
///
/// The obvious loosening — take the first `n_segments` blocks and bind positionally —
/// was tried and is **WRONG**, which is why this reads `==` and not `>=`. Under it,
/// the per-formal token lookup in [`SyView::formals_are_one_register_each`] fails to
/// find the `.ex` formal in the block it was handed for **343,315** functions of the
/// 554,056 it reaches. So the surplus is not a tail that can be truncated; it is
/// interspersed, and position is not the key. (Census would have risen by 2,981 and
/// the workload would still have shown 0 mismatch — because the token lookup catches
/// the misalignment and turns it into a refusal. A green scan would have hidden a
/// binding that is wrong 62% of the time.)
///
/// What the next rung needs is a **key**, not an offset: bind a block to a segment by
/// identity and validate it, rather than by ordinal. The block header's own `<tok>`
/// is the obvious candidate — it coincides with the function's exit-label token in
/// every probe — and the cheap positive check already exists in outline, since
/// `formals_are_one_register_each` must find every `.ex` formal token in the block it
/// is given. Requiring that agreement for *every* segment before binding *any* would
/// replace this ordinal check with a content check. Until then the honest state is
/// over-refusal, per this module's fail-closed asymmetry.
pub(crate) struct SyLocals {
    blocks: Option<Vec<SyBlock>>,
}

impl SyLocals {
    pub(crate) fn new(sy: Option<&[u8]>, n_segments: usize) -> Self {
        let blocks = sy.and_then(sy_blocks).filter(|b| b.len() == n_segments);
        SyLocals { blocks }
    }

    /// Everything the `.sy` layer contributes about the `i`-th function segment.
    ///
    /// `formals: None` means **undetermined**, and a body whose formals matter
    /// must refuse — never "no formals" and never "assume one register each".
    pub(crate) fn view(&self, i: usize) -> SyView<'_> {
        match &self.blocks {
            Some(b) => match b.get(i) {
                Some(blk) => SyView {
                    locals: &blk.int_locals,
                    formals: Formals::Declared(&blk.formals),
                },
                None => SyView::UNKNOWN,
            },
            None => SyView::UNKNOWN,
        }
    }
}

/// What one function segment's `.sy` block says, as the body parser needs it.
///
/// Two facts travel together because a body needs both and neither has a safe
/// default: the locals it may fold, and the **widths of its formals**. A `.sy`
/// that did not bind supplies neither.
#[derive(Clone, Copy)]
pub(crate) struct SyView<'a> {
    pub(crate) locals: &'a [u32],
    pub(crate) formals: Formals<'a>,
}

/// What is known about a segment's formal-parameter widths. Three states, because
/// "not declared" and "declared as scalars" are different facts and collapsing
/// them into a bool is how the register-vs-index confusion got in.
#[derive(Clone, Copy)]
pub(crate) enum Formals<'a> {
    /// No `.sy` block bound to this segment: the widths are **unknown**, and a
    /// body that needs them must refuse.
    Undetermined,
    /// Declared by `.sy`, with each formal's byte size.
    Declared(&'a [SyFormal]),
    /// **Unit tests only.** A synthetic pinned segment whose parameters are all
    /// scalars by construction — the fixture author wrote the bytes, so the fact
    /// is known without a `.sy` companion to state it. Cannot exist in a release
    /// build, so no production path can reach the assumption it encodes.
    #[cfg(test)]
    AllOneRegisterByConstruction,
}

impl SyView<'_> {
    /// No `.sy` binding: no locals, and formal widths undetermined.
    pub(crate) const UNKNOWN: SyView<'static> =
        SyView { locals: &[], formals: Formals::Undetermined };

    /// The largest parameter width that provably occupies exactly one GPR.
    ///
    /// MEASURED with a size ladder (`fixtures/cpp/il_param_aggr_neg.cpp`): a
    /// by-value struct of 1 or 2 `int`s (4 and 8 bytes) leaves the next parameter
    /// in the next register, while 3, 4 and 5 `int`s (12, 16, 20 bytes) each push
    /// it further along. Floating-point scalars reserve exactly one GPR apiece
    /// and are also fine (`float`, two `float`s and three `double`s all match).
    ///
    /// The multi-register rule itself is **not** implemented, and `ceil(size/8)`
    /// is **contradicted** rather than merely unproven: it holds for POD
    /// aggregates (16 B → r5, 24 B → r6) and fails everywhere else. A 12-byte
    /// polymorphic class and a 16-byte class with a copy constructor each take
    /// exactly ONE register, because they are passed by hidden reference — `.sy`
    /// records the latter as a 4-byte *pointer*, kind 03 — while a 300-byte
    /// struct takes NONE and is stack-homed (`lwz r11,324(r1)`). The footprint
    /// depends on how a type is passed, which depends on its triviality as well
    /// as its size, and that convention is not captured. Anything wider than
    /// [`Self::ONE_GPR_MAX`] refuses; see `fixtures/cpp/il_param_aggr_neg.cpp`.
    ///
    /// Compared against a `u32` because the width is a **varint** and genuinely
    /// exceeds 16 bits — a `char[65540]` member is a real declaration, and
    /// reading the field as a `u16` is what made this comparison meaningless for
    /// anything ≥ 128 bytes.
    const ONE_GPR_MAX: u32 = 8;

    /// Whether every one of `formals` (tokens from `.ex`, in any order) is
    /// declared by `.sy` at a width that occupies exactly one argument register.
    ///
    /// This is the precondition that makes a formal's **index** equal its
    /// **argument-register number**, which every shape in `super::body` relies on
    /// when it does `params.iter().position(...)`.
    ///
    /// Both failures refuse, and neither is more admissible than the other, but
    /// they are returned as *different* census keys because they are different
    /// facts and rank differently: `param-width-undetermined` is a gap in this
    /// reader (a `.sy` record it cannot parse, so the whole file's binding is
    /// withheld), while `param-multi-reg` is a construct the port genuinely does
    /// not lower. Collapsing them into one bucket would put a reader bug and a
    /// missing feature under one name and rank the pair by their sum — the
    /// conflation `docs/GAPS.md` §6 records as a measurement failure in its own
    /// right.
    pub(crate) fn formals_are_one_register_each(&self, formals: &[u32]) -> Result<(), &'static str> {
        // Zero or one explicit formal needs no `.sy` at all, and saying so is not
        // a shortcut but the same proof stated where it is cheap: displacement is
        // caused by a parameter *preceding* another one, so a lone parameter is
        // always in the register its index names. `this`, when present, precedes
        // it and is a pointer — exactly one GPR, whatever it points at. This case
        // is worth keeping separate because the shapes admitted so far are
        // getters and identities, which take no argument or one.
        if formals.len() <= 1 {
            return Ok(());
        }
        let declared = match self.formals {
            Formals::Declared(d) => d,
            Formals::Undetermined => return Err("param-width-undetermined"),
            #[cfg(test)]
            Formals::AllOneRegisterByConstruction => return Ok(()),
        };
        for tok in formals {
            match declared.iter().find(|f| f.tok == *tok) {
                // `.sy` bound for this file, yet does not declare this formal:
                // the two layers disagree about what the parameters are, which is
                // a fact missing rather than a construct refused.
                None => return Err("param-width-undetermined"),
                Some(f) if f.size == 0 || f.size > Self::ONE_GPR_MAX => {
                    return Err("param-multi-reg")
                }
                Some(_) => {}
            }
        }
        Ok(())
    }
}

/// One formal parameter, as `.sy` declares it: its token and its **byte size**.
///
/// The size is the whole reason this layer is consulted for formals at all. A
/// parameter's argument-*register* number is not its declaration *index*: a
/// by-value aggregate wider than 8 bytes occupies more than one GPR and shifts
/// every later parameter up. `.ex`'s formals region (`46 (2D <tok>)*`) carries
/// tokens and no types, so the width has to come from here or from nowhere —
/// and "nowhere" is what produced the mis-emit
/// `fixtures/cpp/il_param_aggr_neg.cpp` pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SyFormal {
    pub(crate) tok: u32,
    /// The declared byte size. A `u32` because the field is a varint whose observed
    /// range reaches 4,116 — a `u16` truncation here would silently turn a
    /// 65,540-byte object into a 4-byte one, and 4 bytes is the one value that
    /// passes the one-register gate.
    pub(crate) size: u32,
}

/// The tokens one `.sy` function block declares, split by what codegen may do
/// with them.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SyBlock {
    /// Formal parameters with their widths, unordered.
    pub(crate) formals: Vec<SyFormal>,
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
/// A **second** inter-block record family, with its own lead byte and a payload
/// that is nothing like the `03` family's. It sits at top level, always between a
/// block's `06` close and the next `03 01` open, so it can never shift a variable's
/// attribution — only stop the reader dead, which is what it did: it is the reason
/// the 3 of 200 workload translation units whose `.sy` bound were the only ones
/// without one.
///
/// It reads
///
/// ```text
///   1A <b> <tok> <type prefix> <type extent>
/// ```
///
/// — the same two type readers a variable record uses ([`read_type_prefix`],
/// [`read_type_extent`]), and **not** the variable-record grammar itself: there is no
/// name, and no `00 <cls> 04` between the prefix and the extent. Two witnesses fix
/// that, and they fix it because the size field is where they differ:
///
/// ```text
///   1A 02 <tok> C6 81 06 | 80 0C 01 00 00 | 00 | 00 00 | 80 04 12 00 00   size 268
///   1A 05 <tok> C6 81 06 | 08          | 00 | 00 00 | 80 A1 14 00 00      size 8
/// ```
///
/// One is 20 bytes and the other 16, and in both the next byte is exactly a `03 01`
/// block open or a `03 03` label — so a fixed-width reading of this record is wrong,
/// and it was: the first version of this function required the 20-byte shape
/// literally and refused four of the 200 translation units measured. The size-8
/// witness also carries type id `0x14A1`, the same id as the 8-byte `str` local, so
/// the two fields corroborate each other.
///
/// UNVERIFIED, and stated as such: what the record *means*, and what the byte in the
/// depth position (`02` or `05`) is. Nothing reads either.
const REC_WIDE_INTER_BLOCK: u8 = 0x1A;
/// The four bytes after a block's or label's token. **Not interpreted**, only
/// stepped over: they read `1F 00 01 01` in every fixture and `1F 00 02 01` in a
/// real translation unit, so requiring any of them literally refuses real input —
/// the trap of reading a never-varied field as a constant, caught only by running
/// this against a 649 KB workload `.sy` rather than the probes it was written from.
/// A block header and a label declaration share this tail, and so share a width;
/// the byte after `03` is the only thing that tells them apart.
const HEADER_TAIL_LEN: usize = 4;
/// The last two bytes of an inter-block record's tail, which the named shape keeps
/// while replacing the first field with a string. See [`skip_inter_block`].
const INTER_BLOCK_TAIL_LEN: usize = 2;
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
/// A **typedef**: a name bound to a type, declaring no object. Never admitted, only
/// stepped over — see [`read_record`].
const REC_TYPEDEF: u8 = 0x0B;
/// The type tag of the 4-byte scalar family. **Not** a constant across the file —
/// an 8-byte type reads `88`, so the tag is read as part of the type and only
/// *admission* requires this value; the region's width does not depend on it.
const TYPE_TAG: u8 = 0x86;
/// A type tag with this bit set carries one **extra byte** before the kind (`C6 81
/// 06`, `CA 81 0D`), displacing every field after it. See [`read_type_prefix`].
const TYPE_TAG_WIDE_BIT: u8 = 0x40;
/// The wide prefix's extra byte. Constant at every witness, so required and not
/// interpreted.
const TYPE_WIDE_MARK: u8 = 0x81;
/// Flags bit 7: the type carries one extra 2-byte field before its id. See
/// [`read_type_extent`] — the single-channel rule, and why the two-channel one it
/// replaced was wrong.
const FLAGS_HAS_EXTRA: u16 = 0x0080;
/// That extra field's value at every witness. Meaning unknown, so required.
const TYPE_EXTRA_FIELD: [u8; 2] = [0x80, 0x00];
const TYPE_KIND_INT: u8 = 0x01;
/// Storage class: `01` automatic, `03` formal. Redundant with the section depth in
/// every witness, and required to agree with it.
const CLS_AUTOMATIC: u8 = 0x01;
const CLS_FORMAL: u8 = 0x03;
/// Constant across every witness, between the storage class and the size.
const SIZE_LEAD: u8 = 0x04;
const SIZEOF_INT: u32 = 4;
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
        loop {
            if sy.get(p) == Some(&REC_INTER_BLOCK) && sy.get(p + 1) != Some(&BLOCK_OPEN[1]) {
                // `03 03 <tok> <tail:4>` shares a block header's width, which is why
                // the byte after `03` is the only thing separating them.
                p = skip_inter_block(sy, p)?;
                continue;
            }
            if sy.get(p) == Some(&REC_WIDE_INTER_BLOCK) {
                p = skip_wide_inter_block(sy, p)?;
                continue;
            }
            break;
        }
        // A bare `06` at top level ends the **file**, not a block: every real
        // translation unit's `.sy` closes with a run of them (46 to 246 bytes in the
        // sample) and nothing after. It is required to be exactly that — a run
        // reaching EOF — because that is what was measured, across 130 files, with
        // an instrument that counted any top-level `06` followed by a non-`06` byte
        // and found **zero**. An interleaved one therefore fails the file closed
        // rather than being consumed as padding.
        //
        // UNVERIFIED: what they close. The lengths do not match the count of any one
        // inter-block record kind, and this reader does not need them to — nothing
        // is read out of them.
        if *sy.get(p)? == BLOCK_CLOSE {
            return sy[p..].iter().all(|b| *b == BLOCK_CLOSE).then_some(out);
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
                    REC_PLAIN | REC_ARRAY | REC_STATIC | REC_TYPEDEF => {
                        read_record(sy, p, depth)?
                    }
                    _ => break,
                };
                p = next;
                match rec {
                    // A formal is recorded whatever its type: `parse_formals`
                    // already establishes *which* tokens are formals from `.ex`.
                    // What only this layer knows is each one's **width**, which
                    // decides how many argument registers it occupies — see
                    // [`SyFormal`]. So the type is not gated on here, but the
                    // size is carried out.
                    Some(f) if depth == DEPTH_FORMALS => block.formals.push(f),
                    Some(f) => block.int_locals.push(f.tok),
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

/// Read a `.sy` type's **prefix** — `<tag> <kind>`, or `<tag> 81 <kind>` — and
/// return `(tag, kind, width)`.
///
/// A tag with bit 6 set carries one extra byte before the kind, displacing every
/// field after it. Getting this wrong was the single largest cause of `.sy` never
/// binding on real input: 197 of 200 workload translation units contain such a
/// record, each one desynced the reader by a byte, and the whole file's binding is
/// all-or-nothing — so the `.sy` layer had, measurably, **never** bound on a real
/// translation unit, only on the probe fixtures it was written from.
///
/// MEASURED, three witnesses whose tags differ while the extra byte does not:
///
/// * `C6 81 06` — `str`, an 8-byte class local whose address is taken.
/// * `C6 81 03` — `v`, a polymorphic class parameter, kind 3 being a *data
///   pointer*: MSVC passes a class with a vtable by hidden reference
///   (`fixtures/cpp/il_param_poly.cpp`).
/// * `CA 81 0D` — `vSrc`, a 16-byte value, on a tag whose low bits differ from
///   `C6`'s. This is what makes the rule a **bit test** rather than the literal
///   two-byte prefix `C6 81`: with only the first two witnesses those were
///   indistinguishable, and requiring `C6` literally refuses this record.
///
/// Each is bracketed externally — the byte the enclosing grammar demands next lands
/// exactly at the end of the record — so the width is pinned by the format and not
/// by this function's own arithmetic.
///
/// The extra byte is `81` at every witness. Its meaning is unknown, so it is
/// **required** and not stepped over by width: a field that never varies is
/// indistinguishable from a constant, and the honest encoding of that is to fail
/// the file closed on anything else.
fn read_type_prefix(sy: &[u8], at: usize) -> Option<(u8, u8, usize)> {
    let tag = *sy.get(at)?;
    let mut p = at + 1;
    if tag & TYPE_TAG_WIDE_BIT != 0 {
        if *sy.get(p)? != TYPE_WIDE_MARK {
            return None;
        }
        p += 1;
    }
    let kind = *sy.get(p)?;
    Some((tag, kind, p + 1 - at))
}

/// The offset of the NUL ending a name that starts at `at`, or `None` if there is
/// none within [`MAX_NAME`] or the name is empty.
///
/// A `.sy` name is read only to **bound** a record, never to bind anything: a
/// variable's source name has no bearing on what codegen may do with it. The empty
/// case is rejected because an immediate NUL is how a misread field looks, and the
/// bound keeps a corrupt stream from scanning the rest of the file.
fn name_end(sy: &[u8], at: usize) -> Option<usize> {
    let end = sy
        .iter()
        .enumerate()
        .skip(at)
        .take(MAX_NAME)
        .find(|&(_, &b)| b == 0x00)
        .map(|(i, _)| i)?;
    (end != at).then_some(end)
}

/// Step over one `03 <kind>` inter-block record, `kind != 01` — a label, a scope
/// marker, and a dozen other kinds this reader does not distinguish.
///
/// Its tail has two shapes, differing only in one field and ending in the same two
/// bytes:
///
/// ```text
///   03 <k> <tok> <a> 00           <c> <d>      1E 00 01 01, 47 00 01 01, 05 00 01 02
///   03 <k> <tok> 00 <name> 00     <c> <d>      "jump" 03 01, "ugh" 01 02
/// ```
///
/// The names are source-level **goto labels**, so both shapes occur at the same kinds
/// (`02`, `05`, `06`, `08` and more are witnessed in both) and the kind cannot be the
/// discriminator. The byte after the token is: it is `00` in the named shape and, at
/// every one of the tens of thousands of unnamed records in the sample, one of
/// `05 06 07 08 0B 0C 1E 1F 47` — never `00`.
///
/// That is a **negative** discriminator, and it is stated as one. Two things keep it
/// honest rather than merely convenient:
///
/// * the trailing `<c> <d>` is the *same* field in both shapes — `01 01`, `01 02`,
///   `00 01`, `02 01`, `03 01` all occur across both — so the named shape is the
///   unnamed one with a string spliced into one field, not a separate record whose
///   layout was guessed. Missing those two bytes is what made the reader read the
///   following `03 <k>` as a block open and then find zero sections in it, in 13 of
///   200 files.
/// * a corroborating co-occurrence exists and is deliberately NOT required: every
///   named record in the sample is immediately preceded by a record whose tail begins
///   `47`. Requiring it would mean reaching backwards to a field of a *different*
///   record, and would refuse a named label that happens to open a function's label
///   list.
///
/// The whole-file check is what carries the weight: the stream has to close as a block
/// sequence at EOF and the block count has to equal the `.ex` segment count, or
/// nothing binds.
fn skip_inter_block(sy: &[u8], at: usize) -> Option<usize> {
    let mut p = at + 2;
    let (_tok, w) = read_token_var(sy, p)?;
    p += w;
    if *sy.get(p)? == 0x00 {
        p = name_end(sy, p + 1)? + 1;
        sy.get(p..p + INTER_BLOCK_TAIL_LEN)?;
        return Some(p + INTER_BLOCK_TAIL_LEN);
    }
    sy.get(p..p + HEADER_TAIL_LEN)?;
    Some(p + HEADER_TAIL_LEN)
}

/// Read a `.sy` type's **extent** — `<size varint> <b> <flags16 LE> [80 00] <tid>`
/// — and return `(size, flags, width)`.
///
/// The size is a **varint** in the same 1-or-5-byte encoding [`read_tid`] reads, not
/// a little-endian `u16`, and the byte after it is a separate field this reader does
/// not name. The two readings are indistinguishable for every size below 128,
/// because a varint `<n>` followed by `00` and a `u16` little-endian `<n> 00` are the
/// same two bytes — and every probe fixture is a scalar or a small struct, so the
/// corpus the `u16` reading was written from could not tell them apart. Two
/// witnesses do:
///
/// * `fs`, a 4,116-byte class local: `86 06 00 01 04 | 80 14 10 00 00 | 00 | 21 00
///   | 80 37 17 00 00`. Under the `u16` reading the record is four bytes short and
///   the block's `06` close is consumed as part of the type id — which is how this
///   was found.
/// * `v`, the polymorphic parameter above, is where the unnamed byte **varies**: it
///   is `08` there and `00` at every other witness. So it is a field and not
///   padding, and folding it into the size is what made that parameter report a
///   width of 2052 — a decode error that would have been reported as
///   `param-multi-reg`, i.e. dressed up as a real construct.
///
/// One extra 2-byte field sits between the flags and the type id when **flags bit 7**
/// is set. That single channel is the rule, and it is the rule because a
/// two-channel version of it was refuted: the earlier reading also required the
/// kind's class nibble to be 6 (an aggregate), on the grounds that both channels
/// co-occurred at every witness then available. `vSrc` (`CA 81 0D`, class `D`, flags
/// `0081`) carries the field with a class nibble that is not 6, so the conjunction
/// refuses a record that is really there. The array-local counter-witness that
/// motivated the second channel — class 6, flags bit 7 *clear*, and genuinely no
/// extra field — is still handled correctly, because it is the flags bit that
/// separates them.
///
/// The field's VALUE is `80 00` at all nine witnesses. Its meaning is unknown, so it
/// is required literally rather than skipped by width.
fn read_type_extent(sy: &[u8], at: usize) -> Option<TypeExtent> {
    let mut p = at;
    let (size, sw) = read_tid(sy, p)?;
    p += sw;
    // The unnamed byte between the size and the flags: `00` at every witness but
    // the polymorphic parameter's `08`, so it is consumed and not interpreted.
    sy.get(p)?;
    let flags = u16::from_le_bytes([*sy.get(p + 1)?, *sy.get(p + 2)?]);
    p += 3;
    if flags & FLAGS_HAS_EXTRA != 0 {
        if sy.get(p..p + TYPE_EXTRA_FIELD.len())? != TYPE_EXTRA_FIELD {
            return None;
        }
        p += TYPE_EXTRA_FIELD.len();
    }
    let (tid, tw) = read_tid(sy, p)?;
    Some(TypeExtent { size, flags, tid, width: p + tw - at })
}

/// What [`read_type_extent`] decodes: a declared object's size in bytes, its flags,
/// its `.ex` type-table id, and the byte width of the region they were read from.
struct TypeExtent {
    size: u32,
    flags: u16,
    tid: u32,
    width: usize,
}

/// Step over one [`REC_WIDE_INTER_BLOCK`] record, returning the offset just past
/// it, or `None` if any byte deviates from the single measured shape.
///
/// Nothing is extracted: the record declares no variable this reader understands,
/// and it sits outside every block, so the only thing needed is its exact end. It
/// is *located and refused* in the module's usual sense — never scanned past, so a
/// stream that does not match it withholds the whole file's binding instead of
/// resynchronizing on a guess.
fn skip_wide_inter_block(sy: &[u8], at: usize) -> Option<usize> {
    let mut p = at + 1;
    // The byte in a record's depth position. `02` and `05` both occur, so it is not
    // a constant and is not required; nothing here reads it either, because this
    // record is outside every lexical scope.
    sy.get(p)?;
    p += 1;
    let (_tok, w) = read_token_var(sy, p)?;
    p += w;
    let (_tag, _kind, tw) = read_type_prefix(sy, p)?;
    p += tw;
    let ext = read_type_extent(sy, p)?;
    Some(p + ext.width)
}

/// Read one variable record, returning its token when the record is one a
/// value-substituting parse may fold — a plain, unqualified, 4-byte `int` whose
/// address never escapes — and `None` when the record is merely *stepped over*.
///
/// The distinction matters more than it looks: a record this reader cannot step
/// over exactly would desync and rebind every later token, so the two outcomes are
/// "admitted" and "located but refused", never "skipped by scanning".
fn read_record(sy: &[u8], at: usize, depth: u8) -> Option<(Option<SyFormal>, usize)> {
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
    // and a typedef record have none and run straight into the name. That byte is
    // **not** interpreted: it is `00` on every ordinary variable and `26` on a
    // compiler-generated formal (`__flags`, from the exception-handling regime), so
    // requiring a NUL refuses real translation units.
    if tag != REC_STATIC && tag != REC_TYPEDEF {
        sy.get(p)?;
        p += 1;
    }
    p = name_end(sy, p)? + 1;

    // A **typedef** binds a name to a type and declares no object at all, so there
    // is nothing to admit and nothing to refuse — only a width to get right:
    // `0B <depth> <tok> <name> 00 <b> <tid>`. Witnessed as STL-internal names
    // (`_SrcType`, `_LIterator`) inside template bodies, each ending exactly on the
    // next grammar byte — a `06` block close in one case, the next `01` record in
    // the other. Only two witnesses, and `<b>` is `00` in both, so it is required
    // literally.
    if tag == REC_TYPEDEF {
        if *sy.get(p)? != 0x00 {
            return None;
        }
        let (_tid, tw) = read_tid(sy, p + 1)?;
        return Some((None, p + 1 + tw));
    }

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
        // The type prefix obeys the same wide/narrow rule as a variable record's
        // ([`read_type_prefix`]): a function-scope `static Message msg` writes
        // `C6 81 06 …`, and reading its prefix as two bytes desyncs the record. 49 of
        // the 200 translation units measured contain one.
        let (_tag, _kind, tw) = read_type_prefix(sy, p)?;
        p += tw;
        if *sy.get(p)? != 0x00 {
            return None;
        }
        p += 1;
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

    // `<tag> [81] <kind> 00 <cls> 04 <size varint> <b> <flags16 LE> <tid>`.
    //
    // Two fields of this region were previously misread, and both misreadings were
    // invisible on the probe corpus because they agree with the truth on every
    // small scalar. Fifteen witnesses now parse with **zero leftover bytes** — each
    // one bracketed by the byte the enclosing grammar demands next (a `06` block
    // close, a `0D` section open, or the following record's tag), so the width is
    // pinned externally and not by this reader's own arithmetic.
    let (type_tag, kind, tw) = read_type_prefix(sy, p)?;
    p += tw;
    if *sy.get(p)? != 0x00 {
        return None;
    }
    let cls = *sy.get(p + 1)?;
    if *sy.get(p + 2)? != SIZE_LEAD {
        return None;
    }
    p += 3;
    let ext = read_type_extent(sy, p)?;
    p += ext.width;
    let TypeExtent { size, flags, tid, .. } = ext;
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
        return Some((Some(SyFormal { tok, size }), p));
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
    Some((admissible.then_some(SyFormal { tok, size }), p))
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
    /// The tokens of a formals list, for the assertions that predate `.sy`
    /// carrying widths. A test that cares about the width says so explicitly.
    fn formal_toks(f: &[SyFormal]) -> Vec<u32> {
        f.iter().map(|f| f.tok).collect()
    }

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
    /// size is `0x10` (four ints), whose type id is in the constructed range,
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
    fn local_rec(kind: u8, size: u32, flags: u16, tid: &[u8]) -> Vec<u8> {
        let mut v = vec![REC_PLAIN, 0x02, 0xe6, 0x09, 0x00, b'x', 0x00, TYPE_TAG, kind, 0x00];
        v.push(CLS_AUTOMATIC);
        v.push(SIZE_LEAD);
        v.extend_from_slice(&sy_varint(size));
        // The unnamed byte between the size and the flags.
        v.push(0x00);
        v.extend_from_slice(&flags.to_le_bytes());
        v.extend_from_slice(tid);
        v
    }

    /// A `.sy` size/id varint, in the same 1-or-5-byte encoding [`read_tid`] reads.
    fn sy_varint(v: u32) -> Vec<u8> {
        if v < 0x80 {
            return vec![v as u8];
        }
        let mut out = vec![0x80];
        out.extend_from_slice(&v.to_le_bytes());
        out
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
        assert_eq!(formal_toks(&b[0].formals), vec![0xe309]);
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
        assert_eq!(formal_toks(&b[0].formals), vec![0xe309]);
        assert_eq!(b[0].int_locals, vec![0xe609, 0xe709]);
    }

    /// Label declarations sit between blocks and would otherwise be read as a
    /// malformed block header, refusing every function with control flow.
    #[test]
    fn labels_before_a_block_are_stepped_over() {
        let b = sy_blocks(LOOPED).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(formal_toks(&b[0].formals), vec![0xe809]);
        assert_eq!(b[0].int_locals, vec![0xeb09, 0xec09]);
    }

    #[test]
    fn an_array_local_is_located_and_refused() {
        let b = sy_blocks(ARRAY).unwrap();
        assert_eq!(formal_toks(&b[0].formals), vec![0xf009]);
        assert!(b[0].int_locals.is_empty());
    }

    #[test]
    fn a_function_scope_static_is_located_and_refused() {
        let b = sy_blocks(STATIC).unwrap();
        assert_eq!(formal_toks(&b[0].formals), vec![0xf409]);
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
        assert_eq!(formal_toks(&b[0].formals), vec![0xe609]);
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
        assert_eq!(formal_toks(&b[0].formals), vec![0xcd0c]);
        assert!(b[0].int_locals.is_empty());
    }

    /// An **aggregate** parameter carries one extra 2-byte field between the flags
    /// and the type id. Verbatim capture of `int a1(A1 v, H* h)`'s `v`, a 4-byte
    /// struct (`fixtures/cpp/il_param_aggr.cpp`): kind `06`, size `04 00`, flags
    /// `80 00`, then `80 00`, then the wide id `0x1006`.
    ///
    /// Without this the record desyncs by two bytes, `sy_blocks` refuses the whole
    /// file, and every *other* function's formal widths go undetermined with it —
    /// which is what refused the 4- and 8-byte cases that c2 emits correctly.
    #[test]
    fn an_aggregate_formal_carries_one_extra_field_before_its_id() {
        let rec = &[
            0x01, 0x01, 0x27, 0x0a, 0x00, b'v', 0x00, 0x86, 0x06, 0x00, 0x03, 0x04, 0x04, 0x00,
            0x80, 0x00, 0x80, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0x270a, size: 4 }]);
    }

    /// A **union** parameter is kind `16`, not `06`: the aggregate discriminator is
    /// the kind's low nibble (class 6), exactly as in `readers::read_type`, and a
    /// reader testing the whole byte drops every union. Verbatim from
    /// `union U { int i; float f; }` passed by value.
    #[test]
    fn a_union_formal_is_an_aggregate_by_its_class_nibble() {
        let rec = &[
            0x01, 0x01, 0x21, 0x0a, 0x00, b'u', 0x00, 0x86, 0x16, 0x00, 0x03, 0x04, 0x04, 0x00,
            0x80, 0x00, 0x80, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0x210a, size: 4 }]);
    }

    /// The size is a real field, not a constant: the 20-byte struct that produced
    /// the original mis-emit reads `14 00`. It is *parsed* here and refused later,
    /// by the register gate rather than by this reader — locating a record and
    /// declining to admit it are different things.
    #[test]
    fn a_twenty_byte_aggregate_formal_reports_its_width() {
        let rec = &[
            0x01, 0x01, 0x37, 0x0a, 0x00, b'v', 0x00, 0x86, 0x06, 0x00, 0x03, 0x04, 0x14, 0x00,
            0x80, 0x00, 0x80, 0x00, 0x80, 0x12, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0x370a, size: 20 }]);
    }

    /// The extra field's value is `80 00` at every witness and its meaning is
    /// unknown, so anything else fails the file closed rather than being stepped
    /// over by width. Same record as above with one byte changed.
    #[test]
    fn an_aggregate_extra_field_that_is_not_80_00_refuses() {
        let rec = &[
            0x01, 0x01, 0x27, 0x0a, 0x00, b'v', 0x00, 0x86, 0x06, 0x00, 0x03, 0x04, 0x04, 0x00,
            0x80, 0x00, 0x81, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00,
        ];
        assert_eq!(sy_blocks(&block_with(DEPTH_FORMALS, rec)), None);
    }

    /// A **polymorphic** class parameter opens with the wide prefix `C6 81 03`, and
    /// its kind is `03` — a *data pointer*, because MSVC passes a class with a
    /// vtable by hidden reference. So its width is 4 and it occupies exactly one
    /// argument register, which the old reader could not see: it folded the unnamed
    /// byte after the size (`08` here, `00` everywhere else) into a
    /// little-endian `u16` and reported a width of 2052.
    ///
    /// This is the discriminating witness for BOTH corrections at once. Under the
    /// narrow-prefix reading the `00` check fails and the whole file refuses; under
    /// the `u16` size reading the record is one byte long and the next section's
    /// `0D` lands mid-field. Only both together end the record exactly where
    /// `block_with` puts the next grammar byte.
    ///
    /// Verbatim from `struct V { virtual void f(); int a; }` passed by value
    /// (`fixtures/cpp/il_param_poly.cpp`).
    #[test]
    fn a_polymorphic_class_formal_is_a_hidden_pointer_of_one_register_width() {
        let rec = &[
            0x01, 0x01, 0x29, 0x0a, 0x00, b'v', 0x00, 0xc6, 0x81, 0x03, 0x00, 0x03, 0x04, 0x04,
            0x08, 0x00, 0x00, 0x80, 0x1a, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0x290a, size: 4 }]);
    }

    /// The witness that separates a varint size from a little-endian `u16` one: a
    /// 4,116-byte class local, whose size takes the `80`-escaped five-byte form.
    /// Under the `u16` reading this record is four bytes short and the block's `06`
    /// close is read as part of the type id. Verbatim from a real translation unit
    /// (`fs`, a `system/rndobj` local).
    #[test]
    fn a_size_past_the_varint_escape_is_five_bytes_not_two() {
        let rec = &[
            0x01, 0x02, 0x46, 0x3a, 0x00, b'f', b's', 0x00, 0x86, 0x06, 0x00, 0x01, 0x04, 0x80,
            0x14, 0x10, 0x00, 0x00, 0x00, 0x21, 0x00, 0x80, 0x37, 0x17, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(0x02, rec)).expect("real capture must parse");
        assert!(b[0].int_locals.is_empty(), "a 4116-byte class local may not be folded");
        // And as a formal the same record reports a width that refuses, rather than
        // the 4 a truncating read would have produced from `0x1014 & 0xFFFF`… or the
        // 0x1480 a `u16` read of `80 14` would.
        let asf = &[
            0x01, 0x01, 0x46, 0x3a, 0x00, b'f', b's', 0x00, 0x86, 0x06, 0x00, 0x03, 0x04, 0x80,
            0x14, 0x10, 0x00, 0x00, 0x00, 0x21, 0x00, 0x80, 0x37, 0x17, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, asf)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0x463a, size: 4116 }]);
    }

    /// An aggregate **local** has the same extra field, and its flags differ from a
    /// parameter's in the referenced bit (`81 00` against `80 00`) — which is how
    /// the flags and the extra field are known to be two fields rather than one
    /// read twice. Still not an admissible local: only plain `int` is.
    #[test]
    fn an_aggregate_local_parses_and_is_still_not_an_admissible_local() {
        let rec = &[
            0x01, 0x02, 0x41, 0x0a, 0x00, b'l', b'o', b'c', 0x00, 0x86, 0x06, 0x00, 0x01, 0x04,
            0x0c, 0x00, 0x81, 0x00, 0x80, 0x00, 0x80, 0x30, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(2, rec)).expect("real capture must parse");
        assert!(b[0].int_locals.is_empty(), "a struct local may not be folded");
    }

    /// The byte between token and name is `26` on a compiler-generated formal, so
    /// requiring a NUL there refuses real input. Verbatim from a real TU.
    #[test]
    fn a_compiler_generated_formal_with_a_nonzero_pre_name_byte_parses() {
        let mut rec = vec![0x01, 0x01, 0xf9, 0x15, 0x26];
        rec.extend_from_slice(b"__flags\x00");
        rec.extend_from_slice(&[0x86, 0x02, 0x00, 0x03, 0x04, 0x04, 0x00, 0x01, 0x00, 0x75]);
        let b = sy_blocks(&block_with(DEPTH_FORMALS, &rec)).expect("real capture must parse");
        assert_eq!(formal_toks(&b[0].formals), vec![0xf915]);
    }

    /// An aggregate inserts `80 00` ahead of the type id, and `80 00 10 00 00` is
    /// also exactly how a wide id encodes `volatile int` — so the two are
    /// indistinguishable **to a reader that ignores the kind**. They are not
    /// indistinguishable in the stream: the kind's class nibble is 6 on the
    /// aggregate and 1 on the `volatile int`, and that byte precedes both.
    ///
    /// This record is the same verbatim capture that used to refuse the whole file,
    /// under a doc comment calling the ambiguity "the measured limit of the reader".
    /// It was a limit of the reader and not of the format, and it was expensive: one
    /// aggregate anywhere in a translation unit withheld the binding for all of it,
    /// so no function in that file could establish a formal's width — which is what
    /// refused the 4- and 8-byte struct parameters c2 emits correctly.
    ///
    /// Kept as a *local* rather than restated as a formal, because the point is that
    /// parsing the record and admitting the variable are different decisions: this
    /// one parses and is still not foldable.
    #[test]
    fn an_aggregate_typed_local_parses_once_the_kind_disambiguates_it() {
        let rec = &[
            0x01, 0x02, 0x77, 0x28, 0x00, b'v', b'p', 0x00, 0x86, 0x16, 0x00, 0x01, 0x04, 0x04,
            0x00, 0x81, 0x00, 0x80, 0x00, 0x80, 0x97, 0x13, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(0x02, rec)).expect("the kind byte tells the two apart");
        assert!(b[0].int_locals.is_empty(), "a union local may not be folded");
    }

    /// The neighbour that makes the *two-channel* requirement load-bearing rather
    /// than merely careful: an **array** local is also kind class 6, but its flags
    /// do NOT carry bit 7 and it has **no** extra field — its `80 00 10 00 00` is a
    /// genuine wide id of `0x1000`. A reader keyed on the kind alone would eat two
    /// bytes of that id and desync every record after it.
    ///
    /// So flags bit 7 is what separates a by-value aggregate from an array, and both
    /// facts are required before the extra field is consumed. Verbatim from
    /// `docs/IL_SY_LOCALS.md` §3.6's `int x[4]` row, element size `04` trailing.
    #[test]
    fn an_array_local_is_class_six_with_no_extra_field() {
        let rec = &[
            0x02, 0x02, 0x77, 0x28, 0x00, b'x', 0x00, 0x86, 0x06, 0x00, 0x01, 0x04, 0x10, 0x00,
            0x01, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0x04,
        ];
        let b = sy_blocks(&block_with(0x02, rec)).expect("an array record must parse");
        assert!(b[0].int_locals.is_empty(), "an array local may not be folded");
    }

    /// The neighbour that made the old ambiguity look real: a `volatile int` local,
    /// whose id genuinely is `80 00 10 00 00` with **no** extra field. Its class
    /// nibble is 1, so no extra field is consumed — and it is refused as a local on
    /// its id, which is the pre-existing rule and must not have moved.
    #[test]
    fn a_volatile_int_local_has_no_extra_field_and_is_still_refused() {
        let rec = &[
            0x01, 0x02, 0x77, 0x28, 0x00, b'v', 0x00, 0x86, 0x01, 0x00, 0x01, 0x04, 0x04, 0x00,
            0x01, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(0x02, rec)).expect("a volatile int record must parse");
        assert!(b[0].int_locals.is_empty(), "a volatile int local may not be folded");
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

        // A lead byte the reader has no rule for at all.
        let mut with_2a = vec![0x2a, 0x02, 0xa5, 0x28, 0xc6, 0x81, 0x06, 0x80];
        with_2a.extend_from_slice(EMPTY);
        assert_eq!(sy_blocks(&with_2a), None);
    }

    /// A **named** inter-block record — a `goto` label carrying its source name —
    /// keeps the two trailing bytes the unnamed shape ends with. Missing them made
    /// the reader take the FOLLOWING `03 <k>` for a block open and then find zero
    /// sections in it; that was 13 of the 200 translation units measured.
    ///
    /// Both witnesses are verbatim, and the pair is the point: the trailing field is
    /// `03 01` in one and `01 02` in the other, so it is a real field of the named
    /// shape and not two bytes of some fixed terminator.
    #[test]
    fn a_named_inter_block_record_still_carries_its_two_tail_bytes() {
        for tail in [[0x03, 0x01], [0x01, 0x02]] {
            let mut v = vec![0x03, 0x08, 0x52, 0xa6, 0x01, 0x00, 0x00];
            v.extend_from_slice(b"quat_done\x00");
            v.extend_from_slice(&tail);
            v.extend_from_slice(EMPTY);
            assert_eq!(sy_blocks(&v).map(|b| b.len()), Some(1), "tail {tail:02x?}");
        }
        // Two bytes short, which is how this was found: the next record's `03` is
        // read as a block open whose first byte is not `0D`.
        let mut short = vec![0x03, 0x08, 0x52, 0xa6, 0x01, 0x00, 0x00];
        short.extend_from_slice(b"quat_done\x00");
        short.extend_from_slice(EMPTY);
        assert_eq!(sy_blocks(&short), None);
    }

    /// The `1A` inter-block record is **not** fixed-width: its size field is the same
    /// varint the type extent uses, so one witness is 20 bytes and the other 16. A
    /// reader that pinned the 20-byte shape literally refused four of 200 files.
    /// Both verbatim from real translation units; the 16-byte one's type id `0x14A1`
    /// is the same id as the 8-byte `str` local, and its size is 8.
    #[test]
    fn the_wide_inter_block_record_has_a_varint_size_not_a_fixed_width() {
        let wide = &[
            0x1a, 0x02, 0x7d, 0x28, 0xc6, 0x81, 0x06, 0x80, 0x0c, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x80, 0x04, 0x12, 0x00, 0x00,
        ];
        let narrow = &[
            0x1a, 0x05, 0x2f, 0x45, 0xc6, 0x81, 0x06, 0x08, 0x00, 0x00, 0x00, 0x80, 0xa1, 0x14,
            0x00, 0x00,
        ];
        for rec in [&wide[..], &narrow[..]] {
            let mut v = rec.to_vec();
            v.extend_from_slice(EMPTY);
            assert_eq!(sy_blocks(&v).map(|b| b.len()), Some(1), "{rec:02x?}");
        }
    }

    /// A `0B` **typedef** record: a name bound to a type, declaring no object, with
    /// no byte between the token and the name. Verbatim from a real TU, where the
    /// next byte is the block's `06` close — so its width is pinned externally.
    #[test]
    fn a_typedef_record_is_located_and_declares_nothing() {
        let rec = &[
            0x0b, 0x02, 0x0e, 0xd1, 0x02, 0x00, b'_', b'S', b'r', b'c', b'T', b'y', b'p', b'e',
            0x00, 0x00, 0x80, 0x1f, 0x5b, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(0x02, rec)).expect("real capture must parse");
        assert!(b[0].int_locals.is_empty(), "a typedef declares no object to fold");
    }

    /// A real translation unit's `.sy` ends with a run of bare `06` bytes. They are
    /// accepted only as a run reaching EOF: an interleaved one fails the file closed,
    /// because a top-level `06` followed by a non-`06` byte was never observed in the
    /// 130 files that have a trailer.
    #[test]
    fn a_trailing_run_of_closes_ends_the_file_but_an_interleaved_one_refuses() {
        let mut ok = LOC2.to_vec();
        ok.extend_from_slice(&[BLOCK_CLOSE; 7]);
        assert_eq!(sy_blocks(&ok).map(|b| b.len()), Some(1));

        let mut bad = LOC2.to_vec();
        bad.extend_from_slice(&[BLOCK_CLOSE, BLOCK_CLOSE, 0x03, 0x06, 0xf1, 0x15, 0x1e, 0x00]);
        assert_eq!(sy_blocks(&bad), None);
    }

    /// The **wide type prefix** rule is a bit test, not the literal pair `C6 81`. A
    /// `CA` tag carries the same extra byte, so a reader keyed on `C6` refuses a
    /// record that is really there. Verbatim: `vSrc`, a 16-byte value from
    /// `src/App.cpp`, whose flags carry bit 7 with a class nibble of `D` — see
    /// [`a_flags_bit_seven_extra_field_does_not_require_an_aggregate_class`].
    #[test]
    fn a_wide_type_prefix_is_a_tag_bit_not_the_literal_c6_81() {
        let rec = &[
            0x01, 0x01, 0x46, 0x51, 0x00, b'v', b'S', b'r', b'c', 0x00, 0xca, 0x81, 0x0d, 0x00,
            0x03, 0x04, 0x10, 0x00, 0x81, 0x00, 0x80, 0x00, 0x80, 0x04, 0x1a, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0x4651, size: 16 }]);
    }

    /// The extra 2-byte field is keyed on **flags bit 7 alone**. The earlier rule also
    /// required the kind's class nibble to be 6, on the grounds that both channels
    /// co-occurred at every witness then available; `vSrc` above has class nibble `D`
    /// and carries the field, so the conjunction desyncs on it. Two channels are
    /// better than one only while both are true.
    ///
    /// The counter-witness that motivated the second channel is re-asserted here so a
    /// future widening cannot quietly lose it: an **array** local is class 6 with
    /// flags bit 7 CLEAR and genuinely has no extra field — its `80 00 10 00 00` is a
    /// real wide id — and the flags bit is what separates the two.
    #[test]
    fn a_flags_bit_seven_extra_field_does_not_require_an_aggregate_class() {
        // Class D, flags bit 7 set, extra field present: parses (asserted above by
        // width). Here the discriminating negative, class 6 with the bit clear.
        let array = &[
            0x02, 0x02, 0x77, 0x28, 0x00, b'x', 0x00, 0x86, 0x06, 0x00, 0x01, 0x04, 0x10, 0x00,
            0x01, 0x00, 0x80, 0x00, 0x10, 0x00, 0x00, 0x04,
        ];
        let b = sy_blocks(&block_with(0x02, array)).expect("an array record must parse");
        assert!(b[0].int_locals.is_empty());
        // And with the bit SET the same bytes are read as carrying the field, which
        // moves the id and leaves the record ending two bytes past the grammar's next
        // byte — so the file refuses. That is the whole load the bit bears.
        let mut set = array.to_vec();
        set[14] = 0x81;
        assert_eq!(sy_blocks(&block_with(0x02, &set)), None);
    }

    /// A class with a user copy constructor is passed by **hidden reference**: kind
    /// `03` (a data pointer), size 4, and the unnamed byte after the size is `08` — on
    /// a NARROW `86` tag, so the unnamed byte and the wide prefix are two separately
    /// observed corrections and not one. The `u16` reading made this 0x0804 = 2052 and
    /// reported `param-multi-reg`. Verbatim from
    /// `struct CC { int a,b,c,d; CC(const CC&); }` passed by value;
    /// `fixtures/cpp/il_sy_size_extent.cpp` grades it byte-exact.
    #[test]
    fn a_by_reference_class_parameter_is_a_four_byte_pointer_in_one_register() {
        let rec = &[
            0x01, 0x01, 0xf5, 0x09, 0x00, b'v', 0x00, 0x86, 0x03, 0x00, 0x03, 0x04, 0x04, 0x08,
            0x00, 0x00, 0x80, 0x0c, 0x10, 0x00, 0x00,
        ];
        let b = sy_blocks(&block_with(DEPTH_FORMALS, rec)).expect("real capture must parse");
        assert_eq!(b[0].formals, vec![SyFormal { tok: 0xf509, size: 4 }]);
    }

    /// The size boundary the `u16` reading crossed, as a record rather than as a
    /// source file: 127 is one varint byte and 128 is five. Reading `80 80` as a `u16`
    /// yields 32,896 and ends the record four bytes early — and because binding is
    /// all-or-nothing, one such local unbinds the whole translation unit. Graded
    /// byte-exact by `fixtures/cpp/il_sy_size_extent.cpp`.
    #[test]
    fn the_size_varint_escape_boundary_is_at_128() {
        for want in [127u32, 128, 300, 4116] {
            // A formal, so the decoded size is observable and not just the width.
            let mut rec = vec![REC_PLAIN, DEPTH_FORMALS, 0xe6, 0x09, 0x00, b'x', 0x00];
            rec.extend_from_slice(&[TYPE_TAG, 0x06, 0x00, CLS_FORMAL, SIZE_LEAD]);
            rec.extend_from_slice(&sy_varint(want));
            rec.extend_from_slice(&[0x00]);
            rec.extend_from_slice(&FLAGS_REFERENCED.to_le_bytes());
            rec.extend_from_slice(&[0x80, 0x00, 0x10, 0x00, 0x00]);
            let b = sy_blocks(&block_with(DEPTH_FORMALS, &rec)).expect("must parse");
            assert_eq!(b[0].formals, vec![SyFormal { tok: 0xe609, size: want }], "size {want}");
        }
    }

    #[test]
    fn an_empty_file_is_zero_blocks_not_a_refusal() {
        assert_eq!(sy_blocks(&[]), Some(Vec::new()));
    }
}

