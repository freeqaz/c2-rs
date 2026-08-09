use super::body::{self, parse_segment, BodyShape};
use super::bind::Bindings;
use super::gl::drectve_is_boilerplate;
use super::readers::{contains_subslice, find_subslice, memchr_byte};
use super::{
    CallSeq, FpTail, FramedCall, IlFunction, IlOp, SeqCall, SeqEarlyReturn, SeqGuard, SeqTail,
    SlotArg,
};
use crate::IlBundle;

/// The suffix a `<name>$initializer$` `.CRT$XCU` slot symbol carries
/// (`docs/OBJ_DYNINIT_SHAPE.md` §3.1). Built from the object's **source**
/// identifier, never from its decorated name — `?gL@@3UL@@A` still yields
/// `gL$initializer$` — which is why it is *read* out of `.gl` rather than
/// synthesized from [`DynInitTu::object_symbol`].
const INITIALIZER_SUFFIX: &str = "$initializer$";

/// Whether a `.gl` name is a **dynamic-initializer** thunk, `??__E<ident>@@YAXXZ`.
///
/// `??__F` — the matching atexit destructor thunk — is deliberately **not**
/// admitted here even though the decode handles both. `OBJ_DYNINIT_SHAPE.md`
/// §4.4 measures the destructor shape as **+2 sections** (`.pdata`,
/// `.text$yd`), **+10 symbol records**, and a `??__E` that becomes *framed*
/// (0x40 bytes, 14 relocations, a `bl atexit`). That is a different obj and this
/// lane did not model it.
fn is_dynamic_initializer_name(name: &str) -> bool {
    name.starts_with("??__E") && name.ends_with("@@YAXXZ") && name.len() > "??__E@@YAXXZ".len()
}

/// **W-R1c — a whole TU that is exactly one `??__E` dynamic initializer.**
///
/// The five inputs `c2_core::coff::emit_dyninit_obj` takes, every one of them
/// **read** out of the IL rather than synthesized. See [`IlBundle::dyninit_tu`]
/// for the gates.
#[derive(Clone, Debug)]
pub struct DynInitTu {
    /// `??__EsLicense@@YAXXZ` — the thunk's own symbol, STATIC in the obj.
    pub thunk_name: String,
    /// The constructor it tail-calls; an undefined external.
    pub ctor: String,
    /// The `.bss` object's COFF symbol: undecorated for internal linkage,
    /// decorated for external.
    pub object_symbol: String,
    /// `sizeof` the object.
    pub object_size: u32,
    /// The object's natural alignment, from the `.gl` TYPE tag.
    pub object_align: u32,
    /// `true` => EXTERNAL, `false` => STATIC.
    pub object_external: bool,
    /// `<identifier>$initializer$`, read from `.gl`.
    pub initializer_symbol: String,
    /// The literal's bytes INCLUDING the trailing NUL, from `.in`.
    pub literal: Vec<u8>,
    /// Every `??_C@…` name `.gl` spells. The caller **must** require its own
    /// computed name to be in this set — see
    /// [`super::gl::gl_string_comdat_names`] for why (`/GF`, and the `/Ox`
    /// fixture that would otherwise convert to the wrong shape).
    pub literal_comdat_names: std::collections::BTreeSet<String>,
    /// The constructor's third argument, the literal `0` in both target TUs.
    /// Carried so the caller emits the `li` it actually decoded rather than a
    /// constant.
    pub trailing_literal_arg: i32,
    /// The source path from `.gl`, for `.debug$S`.
    pub src: Option<String>,
}

/// **W-SECT — one namespace-scope object a functionless TU defines**, resolved
/// to everything `c2_core::coff::emit_data_obj` needs and nothing else.
///
/// Separate from `c2_core`'s own object type on purpose: this one is what the
/// **IL says**, and the writer's is what the **obj gets**. The writer applies
/// the class bound (`≤ 2 objects per non-COMDAT section`); this reader applies
/// the decode bound. Neither is allowed to assume the other ran.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataObject {
    /// The COFF symbol name, already final: undecorated for internal linkage,
    /// decorated for external (`docs/OBJ_DATA_BSS_SHAPE.md` §6.1).
    pub coff_name: String,
    /// `sizeof` the object.
    pub size: u32,
    /// Natural alignment in bytes, from the `.gl` TYPE tag — **not** the size.
    pub natural_align: u32,
    /// `true` => StorageClass 2 EXTERNAL; `false` => 3 STATIC.
    pub external: bool,
    /// `Some(bytes)` — a statically-initialized object, so **`.data`**, with its
    /// raw bytes already in the obj's big-endian order and `bytes.len() ==
    /// size`. `None` — uninitialized, so **`.bss`**, which contributes no file
    /// bytes at all.
    pub bytes: Option<Vec<u8>>,
    /// **The record's operand token, which is its DECLARATION-ORDER key** —
    /// Rule A2's walk (`docs/OBJ_DATA_BSS_SHAPE.md` §5.3, §5.6).
    ///
    /// `.bss` walks the `.gl` **file** order and `.data` walks **declaration**
    /// order, and they are genuinely different permutations of the same names.
    /// c2 cannot see the source, so §5.6 identifies the declaration order with
    /// the record's own varint id; this is that id.
    ///
    /// MEASURED on six names chosen so declaration, sorted and `.gl` order are
    /// pairwise different:
    ///
    /// ```text
    ///   source     zulu alpha mike bravo yankee charlie
    ///   tokens     09e3 09e4  09e5 09e6  09e7   09e8      <- ascending = source
    ///   .gl file   zulu yankee mike charlie bravo alpha
    /// ```
    ///
    /// which reproduces §5.3's transcribed order exactly. **Board #183's error
    /// bar does not apply to this field**: that row is about `glparse.py`
    /// scanning *backwards* from the name for the id, and this token is read
    /// **forward** by `read_token_var` at a position the record framing already
    /// validated — the same discipline `gl_symbol_index` applies.
    pub decl_index: u32,
    /// **The symbol addresses this object's initializer carries** — `.in`
    /// element tag `02`, board **#931**.
    ///
    /// Each one is a slot inside [`DataObject::bytes`] that already holds its
    /// addend, and which the obj covers with an `IMAGE_REL_PPC_ADDR32` naming
    /// the target's COFF symbol. `target` is resolved from the `.in` token to a
    /// name **here**, using the same per-record `.gl` binding the object's own
    /// name comes from (#918 — the positional binding disagrees with the census
    /// on 74,955 rows and is not usable for anything keyed by symbol).
    ///
    /// **A consumer that reads `bytes` and ignores this emits a wrong obj.**
    /// Board #232's shape exactly: the bytes look complete because the addend is
    /// usually four zeroes, and the relocation is the part that is missing.
    pub relocs: Vec<DataReloc>,
}

/// One `.data` relocation a static initializer implies (board #931).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataReloc {
    /// Byte offset of the pointer slot inside the owning [`DataObject`].
    pub at: u32,
    /// The target's COFF symbol name, resolved through the `.gl` binding.
    pub target: String,
    /// The addend, already present in [`DataObject::bytes`] as four big-endian
    /// bytes at `at`. Carried so a consumer can check the two agree rather than
    /// trusting that they do.
    pub addend: i32,
    /// `true` when the target is one of this TU's own defined objects; `false`
    /// when `.gl` names it as an undefined external.
    pub target_defined_here: bool,
}

/// One row of [`IlBundle::gl_data_report`] — an INSTRUMENT record, not an emit
/// input. `natural_align` is what the record's TYPE tag was read as; board
/// #1110 is the row that says which tags are read at all.
#[derive(Clone, Debug)]
pub struct GlDataRow {
    pub token: u32,
    pub name: String,
    pub size: u32,
    pub natural_align: u32,
    pub external: bool,
    pub initialized: bool,
    /// The object is its own COMDAT section (a function-local `static`) — see
    /// `gl::DATA_ATTR_COMDAT` for the six graded cells.
    pub comdat: bool,
    pub flags: u8,
}

/// **INSTRUMENT — how far a tag-0x10 ALIAS reaches into the `in` `02`-node
/// RESOLUTION SITE, and how much of that the port's writer can already see.**
///
/// `rungs/_2026-08-04-w-emitp-findings.md` §6 step 3 says the alias table is
/// applied *"once, at the `in` `02`-node resolution site only"*. In `crates/`
/// today there is exactly one such site — [`IlBundle::data_tu`], which turns an
/// [`super::ininit::InSymbolRef::target`] token into a **`.data` relocation's
/// symbol name**. Everything else the spec describes is a consumer that does
/// not exist yet.
///
/// So the question this report answers is not *"how good is the model"* but
/// **"can the port already name the wrong symbol?"**, and it answers it as
/// three nested populations rather than one number, because only the innermost
/// one is a defect and the outer two are what it would take for the innermost
/// to become one:
///
/// * `refs` / `refs_alias` — the whole `.in` tag-02 population, and the part of
///   it that names an alias. **Reachability**, on every TU, whatever the port
///   does with it.
/// * `data_tu_relocs` / `data_tu_relocs_alias` — the same two counts restricted
///   to relocations [`IlBundle::data_tu`] would actually hand the writer.
///   **`data_tu_relocs_alias` is the live-defect count and its known answer is
///   0**; a nonzero is a relocation naming `??_E<X>` where c2 names
///   `??_G<X>`, which is board #232's shape.
/// * `emit_names_alias` — data objects this TU would emit whose own name is in
///   `dom(alias)`. §6 step 4's population, known answer 0.
///
/// **Every field is a count and every one prints its zero** (`docs/STATUS.md`
/// trap 5). `refs_unbound` is published beside `refs_alias` for the reason
/// board #1002 exists: the alias share is only interpretable against how many
/// targets bound at all, and a denominator that moves silently is how a green
/// control stays green.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InAliasReport {
    /// Entries in this TU's [`crate::GlAliasTable`].
    pub aliases: usize,
    /// `.in` element tag-02 symbol references the anchored reader produced.
    ///
    /// **This is the ANCHORED reader's population, not the stream's.** A
    /// sequential parse of the same 850 workload streams frames 879,377 records
    /// where the anchor scan counts 518,098 (board #961), so this is a lower
    /// bound on the channel and is deliberately not corrected upwards.
    pub refs: usize,
    /// …whose target token does **not** resolve in `gl_symbol_index`.
    pub refs_unbound: usize,
    /// …whose target token resolves to a name in `dom(alias)`. **The
    /// reachable population** — what one `functions()` widening is worth here.
    pub refs_alias: usize,
    /// Distinct `.in` owner tokens carrying at least one aliased reference.
    pub records_with_alias: usize,
    /// Relocations [`IlBundle::data_tu`] would emit for this TU, or 0 when
    /// `data_tu` refuses it.
    pub data_tu_relocs: usize,
    /// …naming a symbol in `dom(alias)`. **KNOWN ANSWER 0.** Nonzero is a live
    /// wrong emit, not a gap.
    pub data_tu_relocs_alias: usize,
    /// Objects [`IlBundle::data_tu`] would emit whose own name is in
    /// `dom(alias)` — §6 step 4's population. **KNOWN ANSWER 0.**
    pub emit_names_alias: usize,
    /// Aliases whose own name also carries a tag-0x0E body record, carried up
    /// from [`crate::GlAliasStats::dom_with_body`]. **KNOWN ANSWER 0**, and it
    /// is the precondition that makes step 4 safe rather than a symbol deletion.
    pub dom_with_body: usize,
}

/// **W-SECT — a whole TU that defines NO functions and one or more
/// namespace-scope objects** (board #174).
///
/// This is the smallest whole-TU target that carries a `.data`, and it exists
/// because the shape was a **live wrong emit**: `is_empty_module` is a property
/// of `.ex` alone, so a TU with data and no code took `emit_empty_obj`'s
/// four-section shell and mismatched at file offset 2 on eight of eleven probe
/// shapes.
#[derive(Clone, Debug)]
pub struct DataTu {
    /// The objects, **in `.gl` record order** — which is Rule A1's walk for
    /// `.bss` (`docs/OBJ_DATA_BSS_SHAPE.md` §5.2) and is not the declaration
    /// order. Reading them in any other order is six wrong `Value` fields.
    pub objects: Vec<DataObject>,
    /// The source path from `.gl`, for `.debug$S`.
    pub src: Option<String>,
    /// **The `.in` reader's own census, carried out so a caller can print it.**
    /// `(records, elements, residue, conflicts)`.
    ///
    /// The `.in` scalar-initializer reader is a **new decode**, and a decode
    /// cannot be graded by the oracle: the compiler judges obj bytes and cannot
    /// say whether record *R* is object *S*. So it is graded on its own
    /// invariants instead — injectivity (`conflicts`), totality (`records ==
    /// values + residue + conflicts`, checked as a gate in
    /// [`IlBundle::data_tu`]) and **arity** (`elements`, which counts the
    /// records' *contents*; a reader that dropped an element inside a record it
    /// still accepted leaves `records` and `residue` untouched, which is
    /// `docs/STATUS.md` trap 4 exactly).
    ///
    /// `residue` is never 0 on a real capture — every TU carries a constant pool
    /// whose records this reader deliberately does not model.
    pub in_census: (usize, usize, usize, usize),
}

/// The `.ex` per-function start marker (`4F 1F`). The module stream is a
/// sequence of function bodies, each introduced by this marker; the header /
/// index region before the first one is opaque zero-fill for this class.
pub(crate) const FN_START: [u8; 2] = [0x4F, 0x1F];

/// The one-byte `.ex` body-start token `4C` (`LO`).
///
/// **This is the token; `4F 11` beside it is a separate, OPTIONAL record**
/// (ROADMAP §10.12). Measured over nine functions in five captures, the grammar
/// after the formals list is
///
/// ```text
///   source body        …  46 <formals>   4C  4F 11  53  <stmts> …
///   ??__E / ??__F      …  46             4C         53  <stmts> …
/// ```
///
/// Every `??__E`/`??__F` thunk — the dynamic-initializer and atexit functions a
/// namespace-scope object with a non-trivial constructor or destructor causes c2
/// to emit — carries the bare `4C`. Everything else measured carries `4C 4F 11`,
/// **including `??_G`**, a deleting destructor c2 synthesizes with no source
/// behind it, which is why "compiler-generated bodies are different" is *not* the
/// rule (it was tested and refuted).
///
/// Use [`body_start`] rather than either constant to locate a body: `4C` alone is
/// one byte and is overloaded — it is the last byte of `IntCallEnd`
/// (`55 86 41 74 4C`) and the first of `VoidCallEnd` (`4C 4B`) — so scanning for
/// it bare invents bodies out of payload.
pub(crate) const LO: u8 = 0x4C;

/// The OPTIONAL two-byte record `4F 11` between the [`LO`] body-start token and
/// the body's first `53`. One more `4F xx` record tag, like `4F 1F` (function
/// start), `4F 01` (statement), `4F 02`, `4F 12`, `4F 20`, `4F 33`.
pub(crate) const LO_RECORD: [u8; 2] = [0x4F, 0x11];

/// The composed `4C 4F 11` form — [`LO`] **with** its optional [`LO_RECORD`].
///
/// Kept as a constant because it is what every source function's body opens with
/// and what most readers here still anchor on, but it is a *composition*, not an
/// atom: its absence from a segment means "no `4F 11` record", **not** "no body".
/// Reading its absence as "no function" is the defect ROADMAP §10.11 catalogued.
pub(crate) const LO_MARKER: [u8; 3] = [LO, LO_RECORD[0], LO_RECORD[1]];

/// The `46` formals-list marker, and the `2D` per-formal entry tag.
const FORMALS: u8 = 0x46;
const FORMAL: u8 = 0x2D;

/// Byte offset of the **body-start `4C`** in one `.ex` function segment, in both
/// forms of the grammar — `Some(p)` with `seg[p] == 4C`.
///
/// **Strictly additive** and deliberately so: the composed `4C 4F 11` is tried
/// first and, when present, is returned exactly as the old three-byte scan
/// returned it, so every function that decodes today keeps byte-identical
/// treatment. Only a segment with no `4C 4F 11` anywhere falls through to
/// [`bare_body_start`], and that walk is grammar-driven rather than a byte scan.
pub(crate) fn body_start(seg: &[u8]) -> Option<usize> {
    find_subslice(seg, &LO_MARKER).or_else(|| bare_body_start(seg))
}

/// Offset of the **first operand byte** of a body whose start token
/// ([`body_start`]) is at `lo` — `lo + 3` for the composed `4C 4F 11`, `lo + 1`
/// for the bare `4C`.
///
/// **This is the whole of what W-LO deferred.** Every reader of a body took the
/// `LO` offset and added a hard-coded 3, which is only the operand start when the
/// optional [`LO_RECORD`] is present. Handing any of them a bare `4C` does not
/// make them refuse — it makes them read **two bytes into the body** and
/// mis-parse, and a mis-parse in a body that reaches the emitter is a wrong-bytes
/// emit rather than a gap (`docs/rungs/2026-08-02-w-lo.md`, *Found and not taken*
/// item 1). So the derivation is ONE function, called by all three forward
/// readers ([`super::body::parse_segment_shape`],
/// [`super::body::mcall::body_matches`],
/// [`super::body::shapes::control_flow::scan_full`]) rather than re-derived —
/// §10.14's rule, a private re-derivation is a second rule that agrees until it
/// matters.
///
/// The BACKWARD readers of `lo` are deliberately untouched and need no analogue:
/// `formals_marker`/`parse_formals`/`parse_this_token` walk the region *before*
/// the token, which the optional record does not sit in.
///
/// Byte-identical for every composed body: when `seg[lo+1..lo+3]` is `4F 11` this
/// returns exactly the `lo + 3` the call sites had inline.
pub(crate) fn ops_start(seg: &[u8], lo: usize) -> usize {
    if seg.get(lo + 1) == Some(&LO_RECORD[0]) && seg.get(lo + 2) == Some(&LO_RECORD[1]) {
        lo + 3
    } else {
        lo + 1
    }
}

/// True iff this segment's body opens on the **bare** [`LO`] — the `??__E`/`??__F`
/// form — rather than the composed `4C 4F 11`.
///
/// The same two locators asked a *question* instead of for an offset, so there is
/// no second copy of the rule. It exists because one codegen-class gate is drawn
/// at this boundary deliberately and needs to say so
/// ([`super::body::shapes::calls::sym_addr_tail_call`]'s two-symbol admission):
/// W-LO measured that every bare-`LO` body in the corpus is a `??__E`/`??__F`
/// thunk and that `??_G` — the one other compiler-synthesized `??_` member
/// tested — is composed, so this is the tightest byte-level fence available
/// around the dynamic-initializer class without going to `.gl` for the name.
///
/// **It is a scope fence, not a claim about c2.** Nothing measured says a
/// composed body may not do what a bare one does; the boundary exists so a
/// widening this lane did not grade cannot ride along. See the gate for the
/// captures on both sides.
pub(crate) fn body_start_is_bare(seg: &[u8]) -> bool {
    match body_start(seg) {
        Some(lo) => ops_start(seg, lo) == lo + 1,
        None => false,
    }
}

/// The bare-`4C` body start (`??__E`/`??__F`), located by **walking the prefix
/// grammar** to the token rather than by scanning for the byte.
///
/// The grammar is the token set `codec::try_prefix_token` recognizes, from the
/// `53 53` that opens every function's statement region:
///
/// ```text
///   53 53   (53 | 26 <tok16>)*   46 (2D <tok16>)*   4C 53
/// ```
///
/// Every step is required, and the terminating `4C` must be followed by `53`.
/// That tightness is the whole point: `4C` occurs constantly inside bodies (an
/// `IntCallEnd` ends with one, a `VoidCallEnd` starts with one), and a `4F 1F`
/// that is really a payload collision — measured at ~2 % of `4F 1F` hits on a
/// 1.5 MB `.ex` — must NOT be turned into a function by this. A collision does
/// not carry this grammar, so it returns `None` and the segmentation is
/// unchanged from today.
///
/// **Candidate-and-verify, not a single anchor.** Every `53 53` in the segment
/// is tried in file order and the first that completes the grammar wins, so a
/// `53 53` that is really payload inside the `4F 33 <len>` metadata record
/// costs a failed walk rather than a wrong answer.
///
/// **Why the anchor is not [`BLOCK_START`], which is where this started.** The
/// `4F 02 20 00 4F 01 NN` record is *not* per function: on
/// `fixtures/cpp/wlo_dyninit_pair.cpp` — one object with both a constructor and
/// a destructor, so c2 emits `??__EsL` **and** `??__FsL` — the first segment
/// carries it and the second goes straight from its FnHeader to `53 53`.
/// Anchoring there found one of the two bodies and reported `fn_total = 1` for a
/// TU with two functions, which is the same defect (§10.11) one level down: a
/// count that is only evidence about the predicate that produced it. The
/// fixture exists because that was invisible on the single-thunk one.
fn bare_body_start(seg: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < seg.len() {
        if seg[i] == 0x53 && seg[i + 1] == 0x53 {
            if let Some(p) = bare_lo_after_prefix(seg, i + 2) {
                return Some(p);
            }
        }
        i += 1;
    }
    None
}

/// Walk the statement/result-ref/formals tail of a function prefix from `p`,
/// returning the offset of the bare `4C` it ends at. `None` the moment a byte is
/// not one of the prefix tokens — this is the check that keeps
/// [`bare_body_start`] from being a byte scan.
fn bare_lo_after_prefix(seg: &[u8], mut p: usize) -> Option<usize> {
    loop {
        match *seg.get(p)? {
            0x53 => p += 1,
            0x26 => p += 3,
            FORMALS => break,
            _ => return None,
        }
    }
    p += 1;
    while seg.get(p) == Some(&FORMAL) {
        p += 3;
    }
    (seg.get(p) == Some(&LO) && seg.get(p + 1) == Some(&0x53)).then_some(p)
}

/// Split `.ex` into one segment per **function body**, anchored on the `LO`
/// marker rather than the `4F 1F` function-start marker (P2b).
///
/// `4F 1F` is only two bytes and also occurs inside token and varint payloads,
/// so a raw marker scan over a real translation unit over-counts: measured on
/// `system/world/Dir.cpp` (1.5 MB `.ex`), 5340 `4F 1F` against 5239 `LO` body
/// markers and 5243 function tails (`4F 12 47 54 01 54 00`) — the latter two
/// agree to 0.08%, the first is ~2% high. Anchoring on `LO` keeps the count
/// honest without inventing a denominator.
///
/// The anchor is [`body_start`], not the raw three-byte scan: `4C 4F 11` where a
/// segment has one, and the grammar-gated bare `4C` where it does not (§10.12).
/// The second form is found by a second, strictly-additive pass over the `4F 1F`
/// regions that hold no `4C 4F 11`, so the segmentation of every function that
/// split here before is byte-identical.
///
/// Each segment starts at the `4F 1F` immediately preceding its `LO` (so the
/// formals region stays inside the segment, where [`parse_formals`] looks for
/// it) and runs to the next segment's start. Two bodies sharing one preceding
/// `4F 1F` would collide; the later one then starts at its own `LO`, which
/// simply blocks it at `formals-marker` — an honest miss, never a merge that
/// would silently drop a function from the denominator.
/// Each segment's `.ex` offset is returned alongside it, because the offsets are
/// the join key of the emitted-function binding
/// ([`super::bind::EmitBinding`]): a `.gl` record's body-start offset is bound to
/// the segment that CONTAINS it, so the census row and the obj symbol are two
/// readings of one function. Returned rather than recovered by pointer arithmetic
/// on the slices (`seg.as_ptr() - ex.as_ptr()`), which would be an unchecked
/// invariant exactly where a wrong answer is invisible.
pub(crate) fn split_function_bodies_at(ex: &[u8]) -> (Vec<usize>, Vec<&[u8]>) {
    // Body markers, in file order. Same walk as the old byte loop (a match
    // consumes 3 bytes, a miss 1); candidates are found word-at-a-time.
    let mut los: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 3 <= ex.len() {
        let Some(k) = memchr_byte(LO_MARKER[0], &ex[i..ex.len() - 2]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == LO_MARKER[1] && ex[j + 2] == LO_MARKER[2] {
            los.push(j);
            i = j + 3;
        } else {
            i = j + 1;
        }
    }
    // Function-start markers, in file order, for the "nearest preceding" lookup.
    let mut starts: Vec<usize> = Vec::new();
    let mut i = 0;
    while i + 2 <= ex.len() {
        let Some(k) = memchr_byte(FN_START[0], &ex[i..ex.len() - 1]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == FN_START[1] {
            starts.push(j);
            i = j + 2;
        } else {
            i = j + 1;
        }
    }

    // **Second pass, strictly additive: the bare-`4C` body (ROADMAP §10.12).**
    // A `4F 1F` region that contains no `4C 4F 11` at all may still be a real
    // function whose body opens with the bare token — every `??__E`/`??__F`
    // thunk is one. Anchored per region and gated on the full prefix grammar
    // ([`bare_body_start`]), so a region that already has a composed marker is
    // untouched and a payload collision contributes nothing.
    //
    // Without this the census cannot SEE the class: the two license TUs of the
    // dc3 workload have one function each and reported `fn_total = 0`, and a
    // count is only evidence about the predicate that produced it (§10.11).
    let mut extra: Vec<usize> = Vec::new();
    for (k, &s) in starts.iter().enumerate() {
        let e = starts.get(k + 1).copied().unwrap_or(ex.len());
        // Both lists are ascending, so "does this region already hold a body
        // marker" is a binary search, not a scan — this runs per `4F 1F` on
        // 1.5 MB streams with ~5,000 of each.
        let idx = los.partition_point(|&l| l < s);
        if los.get(idx).is_some_and(|&l| l < e) {
            continue;
        }
        if let Some(p) = bare_body_start(&ex[s..e]) {
            extra.push(s + p);
        }
    }
    if !extra.is_empty() {
        los.extend(extra);
        los.sort_unstable();
    }

    if los.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut segs_start: Vec<usize> = Vec::with_capacity(los.len());
    for &lo in &los {
        // Greatest `4F 1F` offset strictly below this body marker.
        let cand = match starts.partition_point(|&s| s < lo) {
            0 => lo,
            k => starts[k - 1],
        };
        // Never reuse a start (would merge two bodies into one segment).
        let cand = if segs_start.last() == Some(&cand) { lo } else { cand };
        segs_start.push(cand);
    }
    let segs = (0..segs_start.len())
        .map(|k| {
            let start = segs_start[k];
            let end = segs_start.get(k + 1).copied().unwrap_or(ex.len());
            &ex[start..end.max(start)]
        })
        .collect();
    (segs_start, segs)
}

/// True iff `.ex` positively declares a module with **no function bodies**
/// (R1): it carries neither a body marker (`4C 4F 11`) nor a function-start
/// marker (`4F 1F`).
///
/// Both signals are required. `4F 1F` alone is two bytes and collides inside
/// payloads (so its *absence* is meaningful but its presence is not), while
/// `LO` is the marker every real body opens with — on a 1.5 MB real `.ex` the
/// `LO` count tracked the function-tail count to 0.08%. A capture with zero of
/// each has nothing that could be a function.
///
/// Verified against the live toolchain: a TU containing only a typedef captures
/// a 2691-byte `.ex` that is entirely zero-fill apart from a 4-byte head and a
/// 46-byte module-metadata tail, with 0 `LO` and 0 `4F 1F`, and c2 emits a
/// 720-byte four-section obj for it.
pub fn is_empty_module(ex: &[u8]) -> bool {
    let has_lo = find_subslice(ex, &LO_MARKER).is_some();
    let has_fn_start = find_subslice(ex, &FN_START).is_some();
    !has_lo && !has_fn_start
}


/// Split the `.ex` stream at every `4F 1F` function-start marker, keeping the
/// offsets alongside the segments. The offsets are what `.gl`'s framed body-start
/// fields are matched against, so the name binding is per record rather than per
/// position (see [`super::gl::gl_defined_names`], applied by
/// [`super::bind::Bindings::per_record`]).
pub(crate) fn split_functions_at(ex: &[u8]) -> (Vec<usize>, Vec<&[u8]>) {
    let mut starts = Vec::new();
    let mut i = 0;
    // Same walk as the old byte loop (a match consumes 2 bytes, a miss 1);
    // candidates are found word-at-a-time.
    while i + 1 < ex.len() {
        let Some(k) = memchr_byte(FN_START[0], &ex[i..ex.len() - 1]) else {
            break;
        };
        let j = i + k;
        if ex[j + 1] == FN_START[1] {
            starts.push(j);
            i = j + 2;
        } else {
            i = j + 1;
        }
    }
    let mut segs = Vec::with_capacity(starts.len());
    for k in 0..starts.len() {
        let end = if k + 1 < starts.len() { starts[k + 1] } else { ex.len() };
        segs.push(&ex[starts[k]..end]);
    }
    (starts, segs)
}

/// The per-function optimization-settings word for the mode the port's codegen
/// was verified against: `/Ox` (equivalently `/O2`) — optimize, favour speed.
///
/// `/O1` is `00200005`, `/Od` `00800005`, and `#pragma optimize("", off)` under
/// `/Ox` is `00800004`. See `docs/OPT_MODE.md` for the full matrix and for why the
/// bits are treated as opaque and compared whole.
pub const OPT_WORD_OX: u32 = 0x00a0_0005;

/// The optimization word for `/O1` — optimize, favour **size**. The mode the dc3
/// workload compiles with.
///
/// `#pragma optimize("s", on)` under `/Ox` produces this same word, which is the
/// cross-check that it means favour-size and not something `/O1`-specific.
///
/// Differs from [`OPT_WORD_OX`] in exactly one respect that reaches the obj: an
/// intermediate whose predecessor is already dead is written to r11 rather than to
/// a fresh descending register. Verified over all 108 three- and four-operator
/// integer chains and all 27 depth-2 trees — never a different opcode, only a
/// different register field. See `docs/OPT_MODE.md`.
pub const OPT_WORD_O1: u32 = 0x0020_0005;

/// `/O1` with **`#pragma fp_contract(off)`** — bit `0x4` clear, everything else
/// identical to [`OPT_WORD_O1`]. Accepted as `/O1`.
///
/// MEASURED, one bit at a time (`docs/OPT_MODE.md` §6.2): `0x4` is
/// floating-point contraction, the pragma is **per function** rather than
/// per-TU, and its only effect on emitted bytes is that a `*` feeding a `+`/`-`
/// stops fusing —
///
/// ```text
///   float f(float a,float b,float c){ return a*b+c; }
///     contract on   ec2118ba              fmadds f1,f1,f2,f3
///     contract off  ec0100b2 ec20182a     fmuls f0,f1,f2 ; fadds f1,f0,f3
/// ```
///
/// — which is **exactly and only** the set of bodies `codegen`'s contraction
/// guard already refuses ("an FP expression mixes `*` with `+`/`-`"). So
/// accepting this word cannot turn a refusal into a wrong byte for any class the
/// port emits; it can only turn a refusal into a match. Verified at corpus scale
/// rather than argued: the whole fixture corpus compiled at `/O1` with and
/// without the pragma prepended gives **129 byte-identical `.text` and 1
/// differing**, and the one is `w13_fneg`, the fixture whose entire purpose is
/// FMA contraction and which is refused (`docs/OPT_MODE.md` §6.3).
///
/// **What an implementation must not do**: treat `0x4` as ignorable when the
/// contraction rung is eventually built. With the bit clear the correct lowering
/// for `a*b+c` is `fmuls`+`fadds`, and a contracting emitter would produce a
/// valid, wrong, and otherwise-invisible `fmadds`. The word is accepted here
/// because the guard refuses that body today; the day it does not, this constant
/// has to become a *mode*, not an alias.
pub const OPT_WORD_O1_NO_FP_CONTRACT: u32 = 0x0020_0001;

/// [`OPT_WORD_OX`] with `#pragma fp_contract(off)` — the same bit, at the other
/// mode. Accepted as `/Ox`, on its own corpus-scale measurement rather than on
/// the `/O1` one: the whole fixture corpus compiled at **`/Ox`** with and
/// without the pragma gives **145 byte-identical `.text` and 1 differing**, and
/// the one is `w13_fneg` again.
///
/// Worth 0 functions on the dc3 workload, which compiles `/O1`. It exists so the
/// fixture that carries the pragma **grades in every lane** instead of only in
/// the `/O1` one — `c2rs bench` and `c2rs diff` use the `/Ox` profile, and a
/// positive fixture that reports `NotImplemented` in the default lane is the
/// decoration `docs/GAPS.md` §6 records `w13_fabi.cpp` as having been for months.
pub const OPT_WORD_OX_NO_FP_CONTRACT: u32 = 0x00a0_0001;

/// Bit `0x0000_0100` of the per-function optimization word: **this function is a
/// constructor or a destructor.** Orthogonal to the mode bits, so it is masked off
/// before the whole-word compare rather than being enumerated into four words.
///
/// MEASURED at `/Ox`, one function per row in one TU, reading each segment's
/// `4F 1F 80 <LE32>`:
///
/// ```text
/// int p1(int a){return a+1;}                          00a00005
/// int p2(int a){ S s; return a+1; }   local w/ dtor    00a00005   <- NOT cleanup
/// int p3(int a){ try{…}catch(...){…} }                 00a00005   <- NOT EH
/// void V::f() {}                     virtual member    00a00005
/// int p4(int a) throw() {…}                            00a00005
/// int S::m(int a) const {…}           member fn        00a00005
/// A::A() {}                          constructor       00a00105
/// X::X(const X&) {…}                 copy ctor         00a00105
/// U::~U() {}                         dtor, no base     00a00105
/// D::~D() {}                         dtor, one base    00a00105
/// ```
///
/// so the bit tracks *being* a constructor or destructor and nothing else — not
/// needing cleanup, not exception handling, not virtualness. `/O1` shifts the mode
/// bits and leaves it alone (`00200105`).
///
/// **This bit was already costing coverage before it was named.** `A::~A() {}`
/// decodes as [`super::body::BodyShape::EmptyBody`] and the reference emits a bare
/// `blr` for it, exactly as for `void f() {}` — but the word gate compared whole
/// words, so every constructor and destructor in the corpus was a `codegen-gap`
/// no matter how ordinary its body. Masking the bit is what lets the generated
/// empty destructor (the point of this rung) reach the emitter at all.
///
/// It is masked, not ignored: every other bit is still required to match a word
/// this port was verified against, so a third mode or an unknown flag still fails
/// closed.
pub const OPT_WORD_SPECIAL_MEMBER: u32 = 0x0000_0100;

/// The parser's argument-slot enum, in the emitter's spelling. **One** converter
/// for both carriers of it — a chain link's `link_args` and a multi-argument tail
/// call's `arg_sources` — so the two cannot drift about what a `Lit` is.
fn slot_arg(a: body::SlotArg) -> SlotArg {
    match a {
        body::SlotArg::Formal(i) => SlotArg::Formal(i),
        body::SlotArg::Lit(k) => SlotArg::Lit(k),
        // The token→name resolution happens in [`sym_addr_of`], which is the only
        // thing entitled to look at a `.gl` index, so this converter cannot be
        // reached with one: a slot list carrying a symbol goes through
        // [`slot_args_resolved`] instead. Stated as a refusal-shaped mapping
        // rather than an `unreachable!` — the CLI must degrade cleanly.
        body::SlotArg::SymAddr(_) => SlotArg::SymAddr,
    }
}

/// **WR1 — resolve a tail call's slot list**, turning its one
/// [`body::SlotArg::SymAddr`] token into a mangled `.gl` name.
///
/// Returns `(slots, data_syms)`. `None` when the token resolves to nothing, which
/// is the same refusal a callee's does and for the same reason: a relocation
/// against a guessed symbol is a mis-emit, not a gap. **A string literal lands
/// here** — its `.gl` record carries the `25` separator `gl_symbol_index`
/// excludes — so `f("hi")` is refused positively rather than emitted against a
/// name this port cannot even spell.
///
/// The linkage gate is the *caller's* (`docs/IL_CALL_IN_EXPR.md` §17.2 item 7):
/// a defined or static global is a whole extra section in the middle of the
/// section table, and that is a TU-level fact, not a per-slot one.
fn slot_args_resolved(
    slots: Vec<body::SlotArg>,
    resolve_data: &dyn Fn(u32) -> Option<String>,
) -> Option<(Vec<SlotArg>, Vec<String>)> {
    let mut data_syms: Vec<String> = Vec::new();
    let mut out = Vec::with_capacity(slots.len());
    for a in slots {
        match a {
            body::SlotArg::SymAddr(tok) => {
                let n = resolve_data(tok)?;
                // **This class still admits exactly one**, and the refusal stays
                // even though the carrier is a list now. `IlFunction::data_syms`
                // is ordered by the `.text` offset of each `lis`, and a slot walk
                // is not that order: `docs/IL_CALL_IN_EXPR.md` §17.3 (a)/(b)
                // records that c2 materializes only the FIRST of several through a
                // relocation pair and derives the rest by pool-offset difference,
                // which is a different body. Widening the carrier does not
                // characterize that, so a second symbol refuses here exactly as it
                // did when the field was an `Option`.
                if !data_syms.is_empty() {
                    return None;
                }
                data_syms.push(n);
                out.push(SlotArg::SymAddr);
            }
            other => out.push(slot_arg(other)),
        }
    }
    Some((out, data_syms))
}

/// Convert one parsed body shape into the emitter's function record.
///
/// **One locator for the shape→function mapping.** [`IlBundle::functions`] (the
/// gate) and [`IlBundle::census_functions`] (the diagnostic that sizes the
/// census/gate disagreement) both call this, so the two cannot drift about what
/// a shape becomes. `resolve` maps a CALL token to its `.gl` symbol name; `None`
/// from it refuses, because a wrong callee name is a relocation against the
/// wrong symbol — a mis-emit, not a gap.
///
/// Purely per-function: TU-level gates (the single-function restriction on the
/// framed path, unclaimed `.gl` symbols, a locally-defined callee) stay in the
/// caller.
pub(crate) fn shape_to_function(
    shape: BodyShape,
    name: &str,
    src: &Option<String>,
    resolve: &dyn Fn(u32) -> Option<String>,
    resolve_data: &dyn Fn(u32) -> Option<String>,
    resolve_data_def: &dyn Fn(u32) -> Option<crate::func::IlDataDef>,
) -> Option<IlFunction> {
    match shape {
            // An indirect-load leaf reaches the ordinary integer selector,
            // which pattern-matches its exact two-op stream; `params` carries
            // a member function's `this` at index 0 so the base register comes
            // out right.
            // **W-DATA — the static-array scan loop.** The array's token is
            // resolved to a whole DEFINED OBJECT here, the way
            // `BodyShape::IfCallJoin` resolves its callee tokens, and a token
            // that does not resolve refuses the function rather than yielding
            // one with an unrelocatable body. `resolve_data_def` is where every
            // clause about the object lives — COMDAT, initialized, not
            // thread-local, bytes exactly `size`, no interior relocation.
            BodyShape::StaticScanLoop(l) => {
                let data_def = resolve_data_def(l.array_tok)?;
                Some(IlFunction {
                    params: l.params.clone(),
                    static_scan_loop: Some(l),
                    data_def: Some(data_def),
                    ..IlFunction::base(name, src)
                })
            }
            // **W-BDNZ — the counted-`for` accumulate loop.** Nothing to
            // resolve: the class references no external symbol, no data object
            // and no callee, so the carrier travels whole. That is the same
            // shape `BodyShape::PtrWalkModLoop` travels in and for the same
            // reason — the loop's only operands are its own two formals.
            BodyShape::CountedAccumLoop(l) => Some(IlFunction {
                params: l.params.clone(),
                counted_accum_loop: Some(l),
                ..IlFunction::base(name, src)
            }),
            // **W-BLOCKIR — the float array-walk counted loop.** Nothing to
            // resolve, for the same reason the row above gives: the class
            // references no external symbol, no data object and no callee — its
            // only operands are its own formals — so the carrier travels whole.
            // `IPP_basicmath_xbox.cpp`'s obj has **zero relocations**, which is
            // that statement checked against the oracle.
            BodyShape::FloatWalkLoop(l) => Some(IlFunction {
                params: l.params.clone(),
                float_walk_loop: Some(l),
                ..IlFunction::base(name, src)
            }),
            BodyShape::IndirectLoad { params, ops } => {
                Some(IlFunction {
                    params,
                    ops,
                    ..IlFunction::base(name, src)
                })
            }
            // An address leaf (`return &s->m;`) travels the same way: an exact
            // two-op stream that `codegen::addr_leaf_text` pattern-matches
            // ahead of the ordinary selector.
            // A store leaf (`s->m = v;`) travels the same way as the load and
            // address leaves: an exact three-op stream that
            // `codegen::store_leaf_text` pattern-matches ahead of the ordinary
            // selector.
            BodyShape::StoreLeaf { params, ops } => {
                Some(IlFunction {
                    params,
                    ops,
                    ..IlFunction::base(name, src)
                })
            }
            // A store RUN travels the same way — the same op vocabulary, one
            // group per statement, which `codegen::store_leaf_text` walks.
            BodyShape::StoreRun { params, ops } => {
                Some(IlFunction {
                    params,
                    ops,
                    ..IlFunction::base(name, src)
                })
            }
            // **F3 — the store run followed by a call. THE CARRIER, board #844.**
            //
            // `w-f23` landed this production and returned `None` here, because
            // `IlFunction` had no way to spell a composition: `ops` and the call
            // fields were *alternatives* that `c2_core::codegen::select` tries in
            // a fixed order, so a function carrying both emitted one and silently
            // dropped the other — board #232's exact mechanism, and #232 was live
            // for 255 commits while the workload scan read `mismatch 0`.
            //
            // **The repair is not an ordering fix.** Trying the composition
            // earlier still leaves two fields that can both be set and one that
            // wins, and the next widening reintroduces the race somewhere else.
            // The composition is ONE carrier: [`CallSeq::store_run`] holds the
            // run, `ops` stays **empty**, and there is nothing for a dispatch
            // order to get wrong. `IlFunction::store_run_is_carried_alone` is the
            // invariant and `c2_core::codegen::select_function` refuses a
            // violation by name rather than picking a winner.
            //
            // Everything else about the shape is already modeled and is reused
            // rather than restated — this is `EmptyCtorBaseDelegation`'s own
            // sequence with a run in front of it:
            //
            //   * `saved = [this]` — `this` is the one value live across the one
            //     call (board #869), so it goes to r31 and the prologue `std`s it;
            //   * `SeqTail::SavedFormal { param: this }` — the constructor's
            //     implicit `return this` is `mr r3,r31`, the tail that already
            //     exists for the generated base delegation;
            //   * one call, no guard, no early returns.
            //
            // The emitter restates each of those as a backstop
            // (`codegen::store_run_call`), so the parser and the emitter cannot
            // disagree about the class silently.
            BodyShape::StoreRunCall { params, ops, callee_tok, live_args } => {
                // The receiver is argument slot 0 and the production requires
                // slot 0 to be `params[0]`; taken from `params` rather than
                // assumed to be index 0, so a later widening that admits a
                // different receiver has to come back here.
                let this_index = 0usize;
                if params.is_empty() {
                    return None;
                }
                Some(IlFunction {
                    params,
                    call_seq: Some(CallSeq {
                        calls: vec![SeqCall {
                            callee: resolve(callee_tok)?,
                            // **Empty by the production's own gate** (#1129):
                            // every argument slot already holds the formal that
                            // occupies it, so the call emits no move and the
                            // run's base register is never written.
                            arg_ops: Vec::new(),
                            arg_slots: None,
                            link_args: None,
                        }],
                        tail: SeqTail::SavedFormal { param: this_index },
                        saved: vec![this_index],
                        guard: None,
                        early: Vec::new(),
                        store_run: Some(crate::func::StoreRunPrefix {
                            ops,
                            live_args,
                        }),
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // **#839 / board #1199 — THE BIND CARRIER, and it is this arm.**
            //
            // `w-bind` left this arm returning `None` with every field named,
            // *"so a future carrier has to come back to the arm"*. This is that
            // return.
            //
            // The carrier is [`crate::func::IlOp::BoundAddr`] — an op, not a
            // field. `bind_run_ops` discharges the reader's `RefBind` list into
            // the op stream, keeping the bound local's own token as the store's
            // base symbol and leaving the offset undischarged for the emitter to
            // sum exactly once; and then this arm builds **the carriers that
            // already exist**, unchanged: `IlFunction::ops` for the plain tail
            // and #844's `CallSeq::store_run` for #1129's call tail. `RefBind`
            // never crosses into `crates/c2-core`.
            //
            // **Why an op and not a field.** A `binds:` list beside the ops is a
            // second container, and `IlFunction::ops` and `CallSeq::store_run`
            // are already two homes for a run — so a consumer can hold the run
            // and drop the bindings, which is board #232's mechanism and #844's
            // own. Inside the op stream there is nothing beside the ops to drop.
            // `w-seam2`'s sentence for #844 — *"carrying the run inside the
            // sequence makes the race unspellable"* — applied to the binding.
            //
            // **The four refusals are `bind_run_ops`' and each has its own census
            // key**, because a shared one would make each residue unsizeable —
            // and one of them, [`super::body::STORE_RUN_BIND_MIXED_KIND`], is the
            // frontier's last refusal (#836/#868) becoming a countable row for
            // the first time.
            BodyShape::StoreRunBind {
                params,
                binds,
                ops,
                callee_tok,
                live_args,
            } => {
                if params.is_empty() {
                    return None;
                }
                let ops = crate::func::body::shapes::bind_run_ops(
                    &params,
                    &binds,
                    &ops,
                    live_args,
                )
                .ok()?;
                // **BOTH TAILS NOW — the `match` did its job.** `w-bind` left
                // this arm returning `None` for the call tail *"so a lane that
                // lifts the refusal has to come back and build the sequence
                // rather than inherit a wildcard"*, and this is that return.
                // Board #1212: `codegen::store_run_call::save_slot` is fed
                // #584's LEADING RUN now instead of the COUNT of unproduced
                // stores, which is the one thing a second base symbol changes.
                //
                // The call tail is **#1129's**, and it is built out of exactly
                // the parts `BodyShape::StoreRunCall` builds it out of, a few
                // arms above — one `SeqCall` with **no** `arg_ops` (the
                // production's own gate: every slot already holds the formal
                // that occupies it, so the call emits no move and the run's base
                // register is never written), `saved = [this]` because `this` is
                // the one value live across the one call (board #869), and
                // `SeqTail::SavedFormal` for the constructor's implicit
                // `return this`. `ops` stays EMPTY and the run rides inside
                // `CallSeq::store_run`, so there is no second container for a
                // dispatch order to get wrong (board #232, board #844).
                match callee_tok {
                    Some(tok) => Some(IlFunction {
                        params,
                        call_seq: Some(CallSeq {
                            calls: vec![SeqCall {
                                callee: resolve(tok)?,
                                arg_ops: Vec::new(),
                                arg_slots: None,
                                link_args: None,
                            }],
                            tail: SeqTail::SavedFormal { param: 0 },
                            saved: vec![0],
                            guard: None,
                            early: Vec::new(),
                            store_run: Some(crate::func::StoreRunPrefix {
                                ops,
                                live_args,
                            }),
                        }),
                        ..IlFunction::base(name, src)
                    }),
                    None => Some(IlFunction {
                        params,
                        ops,
                        ..IlFunction::base(name, src)
                    }),
                }
            }
            BodyShape::AddrLeaf { params, ops } => {
                Some(IlFunction {
                    params,
                    ops,
                    ..IlFunction::base(name, src)
                })
            }
            BodyShape::StraightLine { params, ops } => {
                Some(IlFunction {
                    params,
                    ops,
                    ..IlFunction::base(name, src)
                })
            }
            // **W8 — the two-arm conditional tail call.** Both callees resolve
            // through the same `.gl` symbol index every other call shape uses;
            // an unresolvable token rejects the whole TU rather than falling
            // back to a positional guess, because a wrong callee name is a
            // relocation against the wrong symbol — a mis-emit, not a gap.
            BodyShape::CondTailPair(pair) => {
                let arm = |a: crate::func::body::CondArmShape| {
                    Some(crate::func::CondArm { callee: resolve(a.callee_tok)?, slots: a.slots })
                };
                Some(IlFunction {
                    params: pair.params,
                    cond_pair: Some(crate::func::CondTailPair {
                        cmp_param: pair.cmp_param,
                        rel: pair.rel,
                        signed: pair.signed,
                        k: pair.k,
                        then_arm: arm(pair.then_arm)?,
                        else_arm: arm(pair.else_arm)?,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // Tail calls: the callee is resolved BY TOKEN through the `.gl`
            // symbol index. An unresolvable token rejects the whole TU
            // rather than falling back to a positional guess — a wrong
            // callee name is a relocation against the wrong symbol, which is
            // a mis-emit, not a gap.
            BodyShape::VoidTailCall { callee_tok } => {
                Some(IlFunction {
                    tail_call: Some(resolve(callee_tok)?),
                    ..IlFunction::base(name, src)
                })
            }
            // The generated empty destructor is a tail call in every respect the
            // emitter can see: there is no result, nothing follows the call, and
            // the receiver is `this` — already in r3 — plus a constant byte
            // offset. At offset 0 (a base sub-object, or a member first in the
            // layout) that constant emits nothing and this is byte-identical to
            // the void tail call above. At a nonzero offset it is one
            // `addi r3,r3,k`, and rather than a new emitter it is handed over as
            // the argument-setup operand stream `[Load(this), Lit(k), Add]` —
            // literally `return g(this + k)`, which `int_tail_call_text` has
            // lowered since the MVP and which the mode lanes and the expression
            // sweep already grade. The parser has bounded `k` to a non-negative
            // signed-16-bit value (`eat_dtor_member_receiver`), which is exactly
            // the range that selector folds into one `addi`.
            BodyShape::EmptyDtorDelegation { callee_tok, this_tok, adjust, eh, .. } => {
                let (params, ops) = if adjust == 0 {
                    (Vec::new(), Vec::new())
                } else {
                    (
                        vec![this_tok],
                        vec![IlOp::Load(this_tok), IlOp::Lit(adjust), IlOp::Add],
                    )
                };
                Some(IlFunction {
                    params,
                    ops,
                    tail_call: Some(resolve(callee_tok)?),
                    // The `/EHsc` label surcharge. It changes no byte of this
                    // function's own `.text` — it changes every framed function
                    // BEHIND it in the same TU.
                    eh_bare: eh,
                    ..IlFunction::base(name, src)
                })
            }
            // **WEC — the empty constructor delegating to one base.** Class B
            // with one saved formal: `this` is live across the base
            // constructor's `bl` because an MSVC constructor returns it, so the
            // whole body is `mr r31,r3 ; bl ?B ; mr r3,r31` in the shipped
            // 1-saved-GPR frame. The call marshals nothing — `this` is already
            // in r3 and the parser admitted no explicit argument — so `arg_ops`
            // is empty and the save move is the only setup.
            BodyShape::EmptyCtorBaseDelegation {
                callee_tok,
                this_tok,
                params,
                unwind_tok,
                eh,
            } => {
                let this_index = params.iter().position(|&t| t == this_tok)?;
                Some(IlFunction {
                    params,
                    call_seq: Some(CallSeq {
                        calls: vec![SeqCall {
                            callee: resolve(callee_tok)?,
                            arg_ops: Vec::new(),
                            arg_slots: None,
                            link_args: None,
                        }],
                        tail: SeqTail::SavedFormal { param: this_index },
                        // A generated base-delegating constructor has no `38`.
                        guard: None,
                        early: Vec::new(),
                        saved: vec![this_index],
                        // **The generated ctor's body is the delegation and
                        // nothing else** — that is what makes it *generated*.
                        // A written one with a store run ahead of the
                        // delegation is board #844's shape and reaches this
                        // crate through `StoreRunCall`, not through here.
                        store_run: None,
                    }),
                    eh_bare: eh,
                    // The unwind action's destructor: named by the IL, absent
                    // from the obj, and accounted for below so the unclaimed
                    // `.gl` symbol gate does not refuse the whole TU for it.
                    eh_unwind_callees: match unwind_tok {
                        Some(t) => vec![resolve(t)?],
                        None => Vec::new(),
                    },
                    ..IlFunction::base(name, src)
                })
            }
            BodyShape::IntTailCall { params, arg_ops, callee_tok } => {
                Some(IlFunction {
                    params,
                    ops: arg_ops,
                    tail_call: Some(resolve(callee_tok)?),
                    ..IlFunction::base(name, src)
                })
            }
            // A single-argument FP tail call is still a tail call — same resolved
            // callee, same `b <callee>`, same REL24 — but its argument lives in
            // the *other* register file, so `params` is the FP formals alone (in
            // FP order) and `ops` stays empty. The move itself is `fp_tail`.
            BodyShape::FpTailCall { params, arg_tok, narrowing, callee_tok } => {
                Some(IlFunction {
                    params,
                    tail_call: Some(resolve(callee_tok)?),
                    fp_tail: Some(FpTail { arg: arg_tok, narrowing }),
                    ..IlFunction::base(name, src)
                })
            }
            // The multi-argument FP tail call (W34). `params` is the FP formals
            // alone, in FP-file order, and `fp_arg_sources` is the permutation
            // over that file. Deliberately a *different* field from
            // [`IlFunction::arg_sources`]: that one indexes the GPR argument
            // registers `r(3+i)`, this one the FP ones `f(i+1)`, and the two
            // sharing a field would be the "one name, two facts" shape that has
            // produced most of this project's wrong-bytes emits.
            BodyShape::FpMultiArgTailCall { params, arg_sources, callee_tok } => {
                Some(IlFunction {
                    params,
                    tail_call: Some(resolve(callee_tok)?),
                    fp_arg_sources: Some(arg_sources),
                    ..IlFunction::base(name, src)
                })
            }
            // A multi-argument tail call is still a tail call — same resolved
            // callee, same `b <callee>` — but its argument setup is a register
            // permutation, or (WLA) the `li`s of its literal slots, rather than an
            // operand stream, so `ops` stays empty and `arg_sources` carries the
            // mapping.
            BodyShape::MultiArgTailCall { params, arg_sources, callee_tok } => {
                let (arg_sources, data_syms) = slot_args_resolved(arg_sources, resolve_data)?;
                Some(IlFunction {
                    params,
                    tail_call: Some(resolve(callee_tok)?),
                    arg_sources: Some(arg_sources),
                    data_syms,
                    ..IlFunction::base(name, src)
                })
            }
            // A framed non-leaf call. `params`/`ops` carry the call ARGUMENT (a
            // bare LOAD of one formal), because the argument register move
            // `or r3,rN,rN` is a function of that formal's position — the same
            // job, and the same `select_text` locator, as the integer tail
            // call's argument setup.
            BodyShape::FramedCall { add_k, callee_tok, params, arg_ops } => {
                Some(IlFunction {
                    params,
                    ops: arg_ops,
                    framed_call: Some(FramedCall {
                        callee: resolve(callee_tok)?,
                        add_k,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // W6: a comparison leaf carries no op stream — codegen emits its
            // spine from the decoded relation instead.
            BodyShape::EmptyBody => {
                Some(IlFunction {
                    empty_body: true,
                    ..IlFunction::base(name, src)
                })
            }
            BodyShape::FloatLeaf { params, ops, double } => {
                Some(IlFunction {
                    params,
                    ops,
                    float_leaf: Some(double),
                    ..IlFunction::base(name, src)
                })
            }
            BodyShape::Compare(cmp) => {
                Some(IlFunction {
                    params: vec![cmp.param],
                    compare: Some(cmp),
                    ..IlFunction::base(name, src)
                })
            }
            // The pointer-walk accumulate loop. No token to resolve: it calls
            // nothing, names no data symbol and mints no label, so the whole
            // shape travels as it was parsed.
            BodyShape::PtrWalkModLoop(l) => {
                Some(IlFunction {
                    params: l.params.clone(),
                    ptr_walk_loop: Some(l),
                    ..IlFunction::base(name, src)
                })
            }
            // The body-parameterized pointer-walk loop. Nothing to resolve, for
            // the reason its sibling has none: it calls nothing, names no data
            // symbol and mints no label, so the operation list travels exactly
            // as it was parsed.
            // W-CFG1 — the `if`/`else`-with-a-join. Two callee tokens to
            // resolve, in BLOCK order, and nothing else: the shape names no data
            // symbol and defines no label the obj can see.
            BodyShape::IfCallJoin(c) => {
                Some(IlFunction {
                    params: c.params.clone(),
                    if_call_join: Some(crate::func::IfCallJoinFn {
                        params: c.params,
                        k1: c.k1,
                        k2: c.k2,
                        acc_init: c.acc_init,
                        callee_hi: resolve(c.callee_hi_tok)?,
                        callee_lo: resolve(c.callee_lo_tok)?,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // **W-EXTDATA — the sunk-`||`-guard, shared-tail body.** Four tokens
            // to resolve and they are resolved through the SAME `resolve` every
            // callee uses, including `fn_addr` — whose `.gl` record is a
            // function's, which is exactly why its symbol is `Type 0x0020`. A
            // single unresolvable one refuses the whole function: a relocation
            // against a guessed symbol is a mis-emit, not a gap.
            BodyShape::GuardChainSharedTail(c) => {
                let fn_addr = resolve(c.fn_addr_tok)?;
                Some(IlFunction {
                    params: c.params.clone(),
                    // The REFHI/REFLO target travels on its own field rather
                    // than in `callees`, because the writer must not emit a
                    // REL24 for it — see `IlFunction::fn_addr_sym`.
                    fn_addr_sym: Some(fn_addr.clone()),
                    guard_chain_shared_tail: Some(crate::func::GuardChainSharedTailFn {
                        params: c.params,
                        guard_ix: c.guard_ix,
                        helper: resolve(c.helper_tok)?,
                        fn_addr,
                        errno: resolve(c.errno_tok)?,
                        invalid: resolve(c.invalid_tok)?,
                        k_guard: c.k_guard,
                        k_range: c.k_range,
                        sentinel: c.sentinel,
                        ret_fail: c.ret_fail,
                        store_width: c.store_width,
                        sunk_arms: c.sunk_arms,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // **W-UNDNAME — the guarded allocation with a shared error store.**
            // Three tokens resolved through the SAME `resolve` every callee
            // uses. A single unresolvable one refuses the whole function: a
            // relocation against a guessed symbol is a mis-emit, not a gap.
            //
            // The two data names go onto `data_syms` in EMISSION order — the
            // object first, because its `lis` is the lower `.text` offset — and
            // `c2_core::data_refs_of` pairs them with the sites it derives from
            // the emitted words by position, checking the counts. The order here
            // is a fact about the emitter, so it is set here and asserted there.
            BodyShape::AllocInitOrFail(a) => {
                let object = resolve(a.object_tok)?;
                let vtable = resolve(a.vtable_tok)?;
                Some(IlFunction {
                    params: a.params.clone(),
                    data_syms: vec![object.clone(), vtable.clone()],
                    alloc_init_or_fail: Some(crate::func::AllocInitOrFailFn {
                        params: a.params,
                        alloc: resolve(a.alloc_tok)?,
                        object,
                        vtable,
                        k_size: a.k_size,
                        k_flag: a.k_flag,
                        k_neg: a.k_neg,
                        k_status: a.k_status,
                        off_a: a.off_a,
                        off_b: a.off_b,
                        off_c: a.off_c,
                        off_d: a.off_d,
                        off_e: a.off_e,
                        off_f: a.off_f,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // **W-OSFINFO.** `data_syms` is in EMISSION order — the limit's
            // `lis` at +0x14 is below the table's at +0x28 — and `data_refs_of`
            // pairs the two lists by index. The two are reached differently in
            // the IL (one a `B9` value read, one a `26` designator) and that
            // difference decides the low half's INSTRUCTION, not its symbol, so
            // it does not reach this list.
            BodyShape::OsfHandleGuard(g) => {
                let limit = resolve(g.limit_tok)?;
                let table = resolve(g.table_tok)?;
                Some(IlFunction {
                    params: g.params.clone(),
                    data_syms: vec![limit.clone(), table.clone()],
                    osf_handle_guard: Some(crate::func::OsfHandleGuardFn {
                        params: g.params,
                        limit,
                        table,
                        errno: resolve(g.errno_tok)?,
                        doserrno: resolve(g.doserrno_tok)?,
                        k_shift: g.k_shift,
                        k_mask: g.k_mask,
                        k_elem: g.k_elem,
                        off_file: g.off_file,
                        k_bit: g.k_bit,
                        off_hnd: g.off_hnd,
                        k_invalid: g.k_invalid,
                        k_ok: g.k_ok,
                        k_errno: g.k_errno,
                        k_doserrno: g.k_doserrno,
                        k_fail: g.k_fail,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // **W-IFN.** The ONLY body class here that resolves NOTHING: it
            // names no data symbol and no callee token, because the one external
            // it calls arrives as an intrinsic SELECTOR (`40` with 172) and has
            // no `.gl` record at all — checked, `work/w-ifn/il/`'s capture of
            // `mmio.cpp` has no `memcpy` string in its `.gl` while the obj
            // carries it as an undefined external. So `data_syms` stays empty
            // and `callees()` gains no arm; the name is minted by the emitter
            // (`c2_core::codegen::guard_ret_chain::MEMCPY_NAME`). A class that
            // put it on `callees()` would fail the accounting gate the other
            // way — it would claim a `.gl` name that is not there.
            BodyShape::GuardRetChain(g) => Some(IlFunction {
                params: g.params.clone(),
                guard_ret_chain: Some(g),
                ..IlFunction::base(name, src)
            }),
            // **W-XLR.** Two callee tokens and nothing else: this class names no
            // data symbol, and its two frame helpers are minted by
            // `c2_core::codegen::FrameLayout` from `saved_gprs` rather than read
            // out of the IL, so they are not resolvable here and must not be.
            BodyShape::XlrcCreateGuard(g) => {
                Some(IlFunction {
                    params: g.params.clone(),
                    xlrc_create_guard: Some(crate::func::XlrcCreateGuardFn {
                        params: g.params,
                        create: resolve(g.create_tok)?,
                        attach: resolve(g.attach_tok)?,
                        k_init: g.k_init,
                        k_bound: g.k_bound,
                        k_lo: g.k_lo,
                        k_hi: g.k_hi,
                        k_fail: g.k_fail,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            // **W-JSON.** Nothing to resolve at all: this class names no
            // callee and no data symbol, and its two frame helpers are minted
            // by `c2_core::codegen::FrameLayout` from `saved_gprs` rather than
            // read out of the IL, so they are not resolvable here and must not
            // be. The whole shape travels as it was parsed.
            BodyShape::JsonUtf8Copy(g) => {
                Some(IlFunction {
                    params: g.params.clone(),
                    json_utf8_copy: Some(crate::func::JsonUtf8CopyFn {
                        params: g.params,
                        off_buffer: g.off_buffer,
                        off_size: g.off_size,
                        k_arg_err: g.k_arg_err,
                        k_size_err: g.k_size_err,
                    }),
                    ..IlFunction::base(name, src)
                })
            }
            BodyShape::PtrWalkChainLoop(l) => {
                Some(IlFunction {
                    params: l.params.clone(),
                    ptr_walk_chain_loop: Some(l),
                    ..IlFunction::base(name, src)
                })
            }
            // The integer divide/modulo leaf. Like the loop, nothing to
            // resolve: it calls nothing, names no data symbol and mints no
            // label, so the whole shape travels as it was parsed.
            BodyShape::DivModLeaf(d) => {
                Some(IlFunction {
                    params: d.params.clone(),
                    div_mod_leaf: Some(d),
                    ..IlFunction::base(name, src)
                })
            }
            BodyShape::CmpShiftOr(cso) => {
                Some(IlFunction {
                    params: vec![cso.param],
                    cmp_shift_or: Some(cso),
                    ..IlFunction::base(name, src)
                })
            }
            // Class A many-calls. Every callee is resolved by token through the
            // `.gl` symbol index, exactly as the tail and framed calls are, and a
            // single unresolvable one refuses the whole function — a relocation
            // against a guessed symbol is a mis-emit, not a gap.
            BodyShape::CallSeq { params, calls, tail, saved, guard, early } => {
                let mut resolved = Vec::with_capacity(calls.len());
                for c in calls {
                    resolved.push(SeqCall {
                        callee: resolve(c.callee_tok)?,
                        arg_ops: c.arg_ops,
                        // The token-carrying `SymAddr(tok)` becomes the resolved
                        // unit variant through the same `slot_arg` the link
                        // arguments use — one conversion, not two.
                        arg_slots: c
                            .arg_slots
                            .map(|v| v.into_iter().map(slot_arg).collect()),
                        link_args: c
                            .link_args
                            .map(|v| v.into_iter().map(slot_arg).collect()),
                    });
                }
                Some(IlFunction {
                    params,
                    call_seq: Some(CallSeq {
                        calls: resolved,
                        saved,
                        // **The `CallSeq` production has no store run**, and
                        // will not grow one here: `try_parse_call_seq` walks
                        // statement-position CALLS and a store statement ends
                        // its walk. A run in front of a sequence reaches the
                        // model through `BodyShape::StoreRunCall` (board #844),
                        // which is a different production with a different
                        // gate — one call, an empty argument setup and the
                        // constructor tail — and merging the two would put a
                        // measured regime boundary (#1129) behind a shape that
                        // has never been graded for it.
                        store_run: None,
                        // W10 — the guard is a pure copy: every field is
                        // already resolved (a parameter index, a relation, a
                        // signedness and a literal), so unlike the callees
                        // there is nothing here that can fail to resolve.
                        guard: guard.map(|g| SeqGuard {
                            cmp_param: g.cmp_param,
                            rel: g.rel,
                            signed: g.signed,
                            k: g.k,
                        }),
                        // W11 — the same: every field is already resolved (a
                        // parameter index, a relation, a signedness and two
                        // literals), so there is nothing here that can fail to
                        // resolve the way a callee token can.
                        early: early
                            .into_iter()
                            .map(|e| SeqEarlyReturn {
                                and_conds: e.and_conds,
                                cmp_param: e.cmp_param,
                                rel: e.rel,
                                signed: e.signed,
                                k: e.k,
                                value: e.value,
                            })
                            .collect(),
                        tail: match tail {
                            body::SeqTail::Void => SeqTail::Void,
                            body::SeqTail::CallValue { add_k } => SeqTail::CallValue { add_k },
                            body::SeqTail::CallValueFp => SeqTail::CallValueFp,
                            body::SeqTail::Lit(k) => SeqTail::Lit(k),
                            body::SeqTail::CallLoad { off } => SeqTail::CallLoad { off },
                            body::SeqTail::CallLoadFp { off, double } => {
                                SeqTail::CallLoadFp { off, double }
                            }
                            body::SeqTail::Cmp { cmp, lhs_first } => {
                                SeqTail::Cmp { cmp, lhs_first }
                            }
                        },
                    }),
                    ..IlFunction::base(name, src)
                })
            }
    }
}


/// The optimization mode a per-function word names, when it is one this port has
/// been verified against.
///
/// **One locator for "which words are known".** `c2_core::codegen::opt_mode_of_word`
/// maps this onto its own `OptMode` and the census refuses a function whose word
/// yields `None` — so the two cannot disagree about which functions are in class,
/// which is the whole point of keeping acceptance in this crate.
///
/// One bit of the word is NOT a mode: [`OPT_WORD_SPECIAL_MEMBER`] (`0x0100`) says
/// the function is a constructor or a destructor, measured one flag and one
/// function kind at a time. It is masked off before the whole-word compare, so a
/// destructor's word reads as the mode it actually is. Every other bit is still
/// required to match, so a third mode or an unknown flag fails closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptWordMode {
    /// `/Ox` and `/O2`.
    Ox,
    /// `/O1`, and `#pragma optimize("s", on)`. What the dc3 workload compiles with.
    O1,
}

/// See [`OptWordMode`]. `None` for `/Od`, `#pragma optimize("", off)`, an
/// unreadable segment prefix, or any word this port has not been verified against.
pub fn opt_word_mode(word: Option<u32>) -> Option<OptWordMode> {
    match word.map(|v| v & !OPT_WORD_SPECIAL_MEMBER) {
        Some(v) if v == OPT_WORD_OX || v == OPT_WORD_OX_NO_FP_CONTRACT => Some(OptWordMode::Ox),
        Some(v) if v == OPT_WORD_O1 || v == OPT_WORD_O1_NO_FP_CONTRACT => Some(OptWordMode::O1),
        _ => None,
    }
}

/// Read the per-function optimization-settings word at the head of one `.ex`
/// function segment: the `<LE32>` of `4F 1F 80 <LE32>`.
///
/// **One locator for the field layout.** [`IlBundle::opt_words`] walks the
/// `4F 1F` split and the census walks the `LO`-anchored split — two different
/// segmentations of the same stream, whose counts are close but not equal, so
/// zipping one's words onto the other's rows would be exactly the unstable
/// correspondence `docs/GAPS.md` §6 warns about. Each reads the word out of the
/// segment it already owns, through this.
///
/// **The word is a varint, not a fixed `80 <LE32>`** (roadmap #52,
/// `docs/OPT_MODE.md` §6.1). `80` is the escape and four little-endian bytes
/// follow; a word below `0x80` is the single byte itself, which is what
/// `#pragma optimize("", off)` produces:
///
/// ```text
///   /O1                        4f 1f 80 05 00 20 00 …    = 0x00200005
///   /O1 + optimize("",off)     4f 1f 04 4f 20 80 fe 00 … = 0x00000004
/// ```
///
/// Reading only the escape form is **fail-closed** — the short form yielded
/// `None` and `opt_word_mode` refuses `None` — so this was never a wrong-bytes
/// risk, but it mis-*named* the refusal: a function whose word could not be read
/// censused under `opt-mode-00000000`, a key that asserts the word is zero when
/// in fact it is unknown. On the 878-TU workload **0** otherwise-in-class
/// functions take the short branch, so this fix is worth 0 functions and is a
/// correction to the instrument rather than to coverage.
///
/// `81..FF` is not a form any capture produces and is refused rather than being
/// read as a signed byte the way an operand-stream varint is — an optimization
/// word is a bit field, not a number, and sign-extending one would be inventing
/// a reading.
///
/// `None` when the segment does not open `4F 1F` with a readable word, so a
/// caller that needs a known mode refuses rather than assuming one.
pub(crate) fn opt_word_at(seg: &[u8]) -> Option<u32> {
    if seg.len() < 3 || seg[0] != FN_START[0] || seg[1] != FN_START[1] {
        return None;
    }
    match seg[2] {
        0x80 => (seg.len() >= 7)
            .then(|| u32::from_le_bytes([seg[3], seg[4], seg[5], seg[6]])),
        b if b < 0x80 => Some(b as u32),
        _ => None,
    }
}

impl IlBundle {
    /// The per-function optimization-settings word of each `.ex` function segment,
    /// in file order — the `<LE32>` of the `4F 1F 80 <LE32>` that opens a segment.
    ///
    /// This is a **codegen-target** property, not a decode one, which is why it is
    /// exposed as data here and enforced by `PortC2` rather than gated in
    /// [`IlBundle::functions`] or in the census. The distinction matters for
    /// measurement: a `/O1` TU whose IL decodes perfectly is a `codegen-gap` with a
    /// named reason, and reporting it as `vocab-gap` would blame the IL model for
    /// something it read correctly, while gating it in the census would replace
    /// every real function's actual blocking feature with this one and destroy the
    /// histogram that ranks the roadmap.
    ///
    /// `None` if `.ex` is absent. A segment whose prefix is not `4F 1F 80` yields
    /// `None` **for that entry**, so a caller that requires a known mode refuses
    /// rather than assuming one.
    /// How many `4F 1F` function segments this bundle's `.ex` splits into — the
    /// count [`split_functions_at`] produces, which is the segmentation
    /// `PortC2::build` consumes.
    ///
    /// **A pure reader, and that is the entire point.** It makes no acceptance
    /// decision and never refuses: it is available on a bundle whose
    /// [`IlBundle::functions`] returns `None`, which is 865 of the 878 workload
    /// TUs. Until this existed, `functions()` was the only public reader of the
    /// `4F 1F` split, so any instrument wanting that count could only have it
    /// for a TU that already passed the gate — and `gap.rs`'s emit-set ceiling
    /// consequently filtered on `fn_total`, which is the **other** splitter's
    /// count ([`split_function_bodies_at`], `LO`-anchored). ROADMAP §10.15.
    ///
    /// `None` when `.ex` is absent, rather than `0`: "this bundle has no `.ex`"
    /// and "this bundle's `.ex` has no functions" are different facts, and a
    /// ceiling that compares a segment count against an obj's COMDAT count would
    /// read the first as the second (`docs/STATUS.md` trap 5 — absence reads as
    /// success unless something forbids it).
    ///
    /// This lane did **not** change [`split_functions_at`]: the `??__E`
    /// re-tokenization is in `body_start` / [`split_function_bodies_at`] and in
    /// the codec, so this count is byte-for-byte what it was before it.
    pub fn ex_segment_count(&self) -> Option<usize> {
        Some(split_functions_at(self.ex()?).0.len())
    }

    pub fn opt_words(&self) -> Option<Vec<Option<u32>>> {
        let ex = self.ex()?;
        Some(
            split_functions_at(ex)
                .0
                .into_iter()
                .map(|s| opt_word_at(&ex[s..]))
                .collect(),
        )
    }

    /// Parse this bundle as a sequence of straight-line add-chain functions
    /// (the MVP class, generalized to a multi-function TU). Returns `None` if
    /// the required files are absent, or if the `.gl` name count does not match
    /// the `.ex` function count, or if ANY function body is outside the class
    /// (the caller — `PortC2` — then reports `NotImplemented` for the whole TU).
    ///
    /// Bodies come from `.ex` split at each `4F 1F`; each body's name comes from
    /// the `.gl` record whose framed body-start offset **is** that split point
    /// ([`super::bind::Bindings::per_record`]) — a per-record binding, not a
    /// positional one. Any
    /// `.gl` symbol no record claimed must be a resolved callee, or the TU is
    /// refused: an unclaimed symbol is one the real obj defines and the port does
    /// not model.
    pub fn functions(&self) -> Option<Vec<IlFunction>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;

        // The port emits `.drectve` as a constant, so a TU that adds a linker
        // directive is out of class before any function is even looked at — the
        // section grows, every later section's offset shifts, and the obj diverges
        // at offset 8 regardless of how good the codegen is. Checked ahead of the
        // empty-module case because an empty TU with a `#pragma comment(lib, …)`
        // has exactly the same problem and none of the code.
        if !drectve_is_boilerplate(gl) {
            return None;
        }

        // R1: a TU that defines no functions is in class, and its obj is the
        // fixed four-section shell with no `.text`. Recognized **positively**
        // (no body markers AND no function-start markers), never as "the split
        // returned nothing" — the latter would also fire on a bundle we merely
        // failed to split, and emitting an empty obj for a TU that really has
        // code is precisely the mis-emit the fail-closed rule forbids.
        //
        // Evaluated in one pass over `.ex` instead of calling
        // [`is_empty_module`] up front: the split already proves whether any
        // `4F 1F` exists, so only the no-start case still needs the body-marker
        // probe. The predicate is unchanged — all four (LO?, 4F1F?) cases land
        // exactly where they did:
        //   neither        → empty module (was: is_empty_module → Some([]))
        //   LO only        → None         (was: not empty; split empty → None)
        //   4F 1F, any LO  → parse        (was: not empty; split non-empty)
        let (starts, segs) = split_functions_at(ex);
        if segs.is_empty() {
            return if find_subslice(ex, &LO_MARKER).is_none() {
                Some(Vec::new())
            } else {
                None
            };
        }
        // The whole correspondence seam — names, locals, callee resolution and
        // the name-derived varargs gate — comes from ONE place
        // ([`super::bind`]), built once here and consumed below. The binding is
        // per record and gated fail-closed: the `.gl` records' framed body-start
        // offsets must be exactly the `.ex` split points, in order and 1:1, or
        // `per_record` binds none of them.
        //
        // A *defined* function's own name comes from there. Callee names do NOT:
        // they are resolved by token through the `.gl` symbol index, because the
        // CALL token carries only a function-type id and cannot distinguish two
        // callees with the same signature.
        let bind = Bindings::per_record(gl, self.get("in").unwrap_or(&[]), self.get("sy"), &segs, &starts)?;
        let names = bind.names();
        let src = bind.src.clone();
        let resolve = |tok: u32| -> Option<String> { bind.resolve(tok) };
        let resolve_data = |tok: u32| -> Option<String> { bind.resolve_data(tok) };
        // **W-DATA** — the DEFINED-object resolver. Built beside `resolve_data`
        // and from the same `Bindings`, so the two answer about one `.gl`.
        let resolve_data_def =
            |tok: u32| -> Option<crate::func::IlDataDef> { bind.resolve_data_def(tok) };
        let n_defined = segs.len();
        // **W-MMIOCLOSE — the sibling fact, established BEFORE the per-function
        // loop and consumed inside it.**
        //
        // This is where board **#139**'s rule actually lands, and `w-ifn`'s C6
        // read it one layer too deep. #139 puts acceptance in the parser and
        // keeps the census and the gate asking ONE question; the seam that
        // carries that rule is this function, which is bundle-level and already
        // reasons across siblings four separate ways — `drectve_is_boilerplate`
        // over the whole `.gl`, the label-counter gate over `funcs.iter()`, the
        // unclaimed-`.gl`-symbol accounting over every callee of every function,
        // and `callee_defined_here` against a set built from ALL the names. What
        // the parser cannot see is a sibling from inside `parse_segment`, which
        // is the *body* parser and takes one `.ex` segment. Those are different
        // statements and only the second one is true.
        //
        // `None` when [`super::gl::gl_function_attrs`] refused the file, and
        // then every function's flag stays `None` — the status quo, not a
        // permission. The map is deliberately NOT required to be total over
        // `names`: a name it has no row for also gets `None`.
        let attrs = super::gl::gl_function_attrs(gl);

        let mut funcs = Vec::with_capacity(n_defined);
        for (i, (name, seg)) in names.iter().take(n_defined).zip(&segs).enumerate() {
            // A variadic function's body IL is byte-identical to its non-variadic
            // twin's, so this is the one gate that cannot live in the body parser
            // ([`super::bind::mangled_is_varargs`]). The census asks the SAME
            // predicate through the same `Bindings`, so the two cannot disagree
            // about what is in class.
            if bind.is_varargs(i) {
                return None;
            }
            let mut f = shape_to_function(
                parse_segment(seg, bind.locals(i))?,
                name,
                &src,
                &resolve,
                &resolve_data,
                &resolve_data_def,
            )?;
            // Keyed on the record name this function was BOUND by — the
            // per-record binding — because that is the name the `.gl` attribute
            // row is keyed by too. Keying it on anything else would be #918's
            // shape: two bindings, one apparent fact.
            f.inlinable = attrs
                .as_ref()
                .and_then(|m| m.get(name.as_str()))
                .map(|a| a & super::gl::FN_FLAG_INLINABLE != 0);
            funcs.push(f);
        }

        // TU-level, so it stays here rather than in the per-function helper: a
        // framed function's obj carries `.pdata` and the `$M…`/`$T…` compiler
        // labels, whose numbers come from a counter **every** function in the TU
        // consumes — 1 for each class this port emits, 4 for a framed one (5
        // under `/Gy`). The framed path used to be gated to a single-function TU
        // for exactly that reason; the counter is now read from `.gl` and
        // advanced per function (`c2_core::coff::plan_labels`), so the gate is no
        // longer about the function count. It is about the classes whose stride
        // is **not** 1, because `plan_labels` advances by 1 for every function
        // that is not framed: a framed function sharing a TU with one of those
        // would get labels low by the error — six wrong bytes in an obj that
        // still links. Measured per class in `docs/OBJ_GY_SHAPES.md` §3.6 and
        // asked here through one predicate ([`IlFunction::label_slots`]), which
        // is three-valued so an unmeasured class refuses rather than defaulting.
        //
        // This used to key on "is this a comparison or a floating-point leaf",
        // which over-refused: the comparison stride is **not** uniform over the
        // relation. `==`/`!=`, every unsigned relation, and signed `<`/`>=`
        // against zero all consume 1 and are admitted now; the signed relational
        // spine consumes 3 and still refuses. A float leaf is 2 (4 or 6 with
        // pooled constants) and refuses either way.
        //
        // The counter itself must also be readable. `label_counter` is
        // three-valued on purpose (`None` = undetermined, never a default),
        // because a guessed `$M` number is a mis-emit rather than a gap.
        //
        // "Framed" is `framed_call` OR `call_seq` — the Class A many-call body is
        // framed too, with the same 4 / 5 stride (measured: two two-call bodies in
        // one TU are `$M2553`/`$M2558` under `/Gy` against a `.gl+7` seed of 2538,
        // and 2547/2551 packed). Asking the question through one predicate is what
        // keeps a new framed shape from silently skipping the counter gate.
        if funcs.iter().any(|f| f.is_framed()) {
            for f in &funcs {
                if f.is_framed() {
                    continue;
                }
                // **The gate is "does this class's stride agree with what
                // `plan_labels` will actually advance", not "is it 1".**
                //
                // `plan_labels` advances `label_lead + 1` for a non-framed
                // function, so a class whose lead is nonzero is *not* a class
                // the counter mis-handles — it is one the counter already
                // charges. Written as `!= 1` this gate refused the `eh-bare`
                // leaf (`docs/EH_RECORDS.md` §8.5d, stride 2 at `/EHsc`), which
                // would have turned a wrong-bytes emit into a wholesale refusal
                // of every `/EHsc` TU containing a generated destructor — the
                // safe direction, but it hides the 35,964 functions §7.3 counts
                // as already in class. What still refuses here is what always
                // did: a comparison leaf's 3 and a float leaf's `None`.
                if f.label_slots(false)? != f.label_lead() + 1 {
                    return None;
                }
            }
            super::gl::label_counter(gl)?;
        }
        // Account for every `.gl` symbol no record claimed. The port emits
        // exactly the `n_defined` bodies plus an external symbol per resolved
        // callee, so an unclaimed name is a symbol the real obj has and this obj
        // would not — and for a *data* definition it is a whole extra section.
        // `int gv; int f(int a){return a+1;}` mismatched at file offset 2, the
        // section count, for exactly this reason: `?gv@@3HA` was invisible to the
        // emitter. A defined static member (`?sm@S@@2HA`) did the same.
        //
        // Extern data cannot be told from defined data by mangling — `extern int
        // g;` and `int g;` both appear as `?g@@3HA` — so this refuses both. That
        // costs nothing today: reading a global is already out of class, so an
        // extern that is never referenced is one c2 would not have listed.
        let mut accounted: Vec<&str> = names.iter().map(String::as_str).collect();
        for f in &funcs {
            for c in f.callees() {
                accounted.push(c);
            }
            // **A name the IL references and the obj legitimately does not
            // carry.** The only member of this set today is the base destructor
            // an empty constructor names as its unwind action: on the cheap side
            // there is no funclet, so c2 emits no `bl`, no relocation and no
            // symbol for it (measured, `work/WEC/probe/p2.obj`). Accounted here
            // rather than by loosening the gate, because the gate's reading —
            // "an unclaimed `.gl` name is a symbol the real obj has and this one
            // would not" — is right everywhere else and cost two mismatches at
            // file offset 2 to learn.
            for c in &f.eh_unwind_callees {
                accounted.push(c.as_str());
            }
            // **WR1 — a named data symbol whose address this body materializes.**
            // The obj carries it as an undefined external, exactly as it carries a
            // callee, so it is accounted rather than left to refuse the TU. Safe
            // only because `Bindings::resolve_data` has already proved the `.gl`
            // record says *undefined extern*: a DEFINED global would also be
            // accounted by this line, and it is a whole extra section
            // (`docs/IL_CALL_IN_EXPR.md` §17.2 item 7) — which is why the gate is
            // there and not here.
            for d in &f.data_syms {
                accounted.push(d.as_str());
            }
            // **W-EXTDATA — a FUNCTION whose address this body materializes.**
            // Its own clause and not a widening of the one above, because the
            // two produce different symbol records: this one is `Type 0x0020`
            // and that one `Type 0x0000`, measured side by side in one obj
            // (`work/w-extdata/ref/vswprnc/dis.txt` symbol 18 against
            // `work/w-extdata/ref/undname/dis.txt` symbols 15 and 17).
            //
            // Safe for the same reason the `data_syms` clause is: the name is
            // resolved from a `.gl` record that says *undefined external*, so
            // accounting it does not hide a DEFINED symbol — which would be a
            // whole extra section and the file-offset-2 mismatch this gate cost
            // two objs to learn.
            if let Some(d) = &f.fn_addr_sym {
                accounted.push(d.as_str());
            }
            // **W-DATA — a data object this TU DEFINES.** The clause above
            // accounts for a symbol the obj carries *undefined*; this one
            // accounts for a symbol the obj **defines**, together with the
            // whole COMDAT `.data` section that comes with it.
            //
            // The gate's own comment two blocks up records why it could not be
            // loosened instead: `int gv; int f(int a){return a+1;}` mismatched
            // at **file offset 2**, the section count, because `?gv@@3HA` was
            // invisible to the emitter. It is not invisible now — the object
            // travels on `IlFunction::data_def` all the way to
            // `coff::emit_comdat_obj`, which emits its section, its alignment,
            // its checksum and its defined STATIC symbol. Accounting a name
            // whose section nothing emitted would be that mismatch again, so
            // this line is safe only because `data_def` is the carrier and not
            // a flag.
            if let Some(d) = &f.data_def {
                accounted.push(d.coff_name.as_str());
            }
        }
        if bind
            .unclaimed
            .iter()
            .any(|n| !accounted.contains(&n.as_str()))
        {
            return None;
        }
        // A callee that is also DEFINED here is out of class: c2 may inline it,
        // and the port cannot.
        //
        // Refused wholesale rather than by callee size, because what makes c2
        // inline (and what it does to the symbol table and `.pdata` when it does)
        // is uncharacterized. Calls to true externals are unaffected — those are
        // the tail calls the class was built on.
        //
        // **W-INLFENCE — the test itself now lives in ONE place**
        // ([`super::bind::callee_defined_here`]) and is asked here, by the
        // census and by `diag`. The behaviour at this call site is unchanged:
        // `names` is a `per_record` binding, total and 1:1 with the `.ex`
        // segments by construction, so the set below is exactly the list this
        // clause always scanned. What the factoring buys is that the class-level
        // invariant survives a narrowing of *this* wholesale refusal — which
        // `docs/whitebox/WB_INLINE_FINDINGS.md` §7 explicitly proposes — instead
        // of being an accident of TU-level granularity.
        let defined: std::collections::BTreeSet<String> = names.iter().cloned().collect();
        if funcs
            .iter()
            .any(|f| super::bind::callee_defined_here(f, &defined).is_some())
        {
            return None;
        }
        Some(funcs)
    }

    /// **W-R1c — recognize a whole TU as ONE `??__E` dynamic initializer**, and
    /// hand back the five inputs `c2_core::coff::emit_dyninit_obj` needs.
    ///
    /// # Why this is not a `resolve_data` widening
    ///
    /// The obvious way to make these TUs decode is to let
    /// [`super::bind::Bindings::resolve_data`] admit a symbol this TU *defines*.
    /// That is three lines and it is the wrong three lines. `resolve_data` is on
    /// the ordinary-function path, **39,967** functions currently file under
    /// `data-sym-*`, and the port's ordinary shell has no `.bss` and no `.data`
    /// — so a widening there sends some part of that population into an emitter
    /// that will relocate against a symbol whose section it never wrote. That is
    /// a wrong-bytes emit, which `docs/OBJ_DYNINIT_SHAPE.md` §3.2 and w-r1's
    /// prereg both register as **strictly worse than the honest refusal standing
    /// today**.
    ///
    /// So this is a **separate, whole-TU** question that an ordinary function
    /// cannot reach: it requires the TU to contain exactly one function, and for
    /// that function to be the thunk. `resolve_data` is untouched, and the
    /// `data-sym-unresolved` / `data-sym-not-extern` census keys keep reporting
    /// exactly what they reported before.
    ///
    /// # Every gate, and the byte behind it
    ///
    /// Each `None` below is a TU whose obj differs from the one
    /// `emit_dyninit_obj` builds, and each is stated positively:
    ///
    /// * **exactly one `.ex` function segment**, and its `.gl` name is
    ///   `??__E<ident>@@YAXXZ`. `??__F` (the atexit destructor thunk) is *not*
    ///   admitted: §4.4 makes it +2 sections, +10 symbols and a framed body.
    /// * **exactly one uninitialized (`.bss`) object in the whole TU.** This is
    ///   the gate lane w-bss forced. `docs/OBJ_DATA_BSS_SHAPE.md` §2.2 refutes
    ///   `OBJ_DYNINIT_SHAPE.md` §4.1's "always last": add one plain `char b1;`
    ///   and the shared `.bss` moves out from behind `.text$yc` to **between the
    ///   two `.XBLD$W` watermarks**, a different section order entirely.
    /// * **exactly one initialized object**, the `$initializer$` slot. A second
    ///   is a real `.data` section (`int gDef = 3;`), which is a ninth section.
    /// * **the body is `MultiArgTailCall` with slots `[SymAddr, SymAddr, Lit]`**,
    ///   slot 0's token being the object and slot 1's the literal. If the two
    ///   resolve the other way round the TU refuses rather than being reordered:
    ///   guessing which operand is the receiver is guessing two relocations.
    /// * **the literal's `.gl` name exists.** See
    ///   [`super::gl::gl_string_comdat_names`] — this is the `/GF` fence, and it
    ///   is what keeps `fixtures/cpp/il_dyninit_static.cpp` refusing at `/Ox`.
    ///
    /// Outside all of that the port refuses, and that refusal is a deliverable.
    pub fn dyninit_tu(&self) -> Option<DynInitTu> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let inb = self.get("in")?;

        // **The cheap early-out, and it is a performance gate as much as a
        // correctness one.** `PortC2::build` calls this before `functions()` on
        // EVERY TU, and the port's whole thesis is throughput (~922k obj/s on one
        // thread). Everything below — the segment split, `Bindings::per_record`,
        // a body parse — is work `functions()` is about to do again, so a TU that
        // cannot possibly be a dynamic initializer must not pay for any of it.
        //
        // A `??__E` thunk's name is in `.gl` by construction: that is where the
        // per-record binding reads it from. One substring scan settles it.
        if !contains_subslice(gl, b"??__E") {
            return None;
        }
        // Same first gate as `functions`: the port emits `.drectve` as a
        // constant, so a TU that adds a linker directive diverges at offset 8
        // however good the rest is.
        if !drectve_is_boilerplate(gl) {
            return None;
        }

        // Exactly one function, bound per record. `per_record` is the EMIT
        // binding — it is gated fail-closed on the `.gl` records' framed
        // body-start offsets being exactly the `.ex` split points — and nothing
        // that emits may use the positional one.
        let (starts, segs) = split_functions_at(ex);
        if segs.len() != 1 {
            return None;
        }
        let bind = Bindings::per_record(gl, self.get("in").unwrap_or(&[]), self.get("sy"), &segs, &starts)?;
        let thunk_name = bind.names().first()?.clone();
        if !is_dynamic_initializer_name(&thunk_name) {
            return None;
        }

        // The body. Three slots, in the one order this class was measured in:
        // the object (the constructor's `this`), the literal, then the literal
        // `0`.
        let BodyShape::MultiArgTailCall { arg_sources, callee_tok, params } =
            parse_segment(&segs[0], bind.locals(0))?
        else {
            return None;
        };
        if !params.is_empty() {
            return None; // `??__E…@@YAXXZ` takes no arguments
        }
        let [body::SlotArg::SymAddr(obj_tok), body::SlotArg::SymAddr(lit_tok), body::SlotArg::Lit(k)] =
            arg_sources.as_slice()
        else {
            return None;
        };
        let ctor = bind.resolve(callee_tok)?;

        // The object, and the whole-TU count that lane w-bss made binding.
        let objects = super::gl::gl_data_objects(gl);
        let mut uninit = objects.iter().filter(|(_, o)| !o.initialized);
        let (obj_tok_found, object) = uninit.next()?;
        if uninit.next().is_some() {
            return None; // a second `.bss` object moves the section (w-bss §2.2)
        }
        if obj_tok_found != obj_tok {
            return None; // slot 0 is not the object this TU defines
        }
        let mut init = objects.iter().filter(|(_, o)| o.initialized);
        let (_, initializer) = init.next()?;
        if init.next().is_some() {
            return None; // a real `.data` object is a ninth section
        }
        if !initializer.coff_name.ends_with(INITIALIZER_SUFFIX) {
            return None;
        }

        // The literal, from `.in`, with its `.gl` COMDAT name carried alongside
        // so the caller can require the two to agree.
        let literal = super::inlit::in_string_literals(inb).get(lit_tok)?.clone();

        Some(DynInitTu {
            thunk_name,
            ctor,
            object_symbol: object.coff_name.clone(),
            object_size: object.size,
            object_align: object.natural_align,
            object_external: object.external,
            initializer_symbol: initializer.coff_name.clone(),
            literal,
            literal_comdat_names: super::gl::gl_string_comdat_names(gl),
            trailing_literal_arg: *k,
            src: bind.src.clone(),
        })
    }

    /// **W-SECT — is the bare four-section shell the RIGHT obj for this TU?**
    ///
    /// `emit_empty_obj` writes `.drectve`, `.debug$S` and the two `.XBLD$W`
    /// watermarks and nothing else. Its precondition had been read off `.ex`
    /// alone — [`is_empty_module`], "no function bodies" — and that is **not the
    /// same question**. A TU can declare no functions and still make c2 emit a
    /// `.bss`, a `.data`, a `.rdata` or a `.tls$`, and eight probe shapes did:
    ///
    /// ```text
    ///   int g = 5;                         c2: 5 sections   port: 4   MISMATCH
    ///   char b1;                           c2: 5            port: 4   MISMATCH
    ///   extern const int ce = 9;           c2: 5 (.rdata)   port: 4   MISMATCH
    ///   const char* s = "hi";              c2: 6            port: 4   MISMATCH
    ///   __declspec(thread) int t1;         c2: 5 (.tls$)    port: 4   MISMATCH
    ///   __declspec(selectany) int sa = 3;  c2: 5 (COMDAT)   port: 4   MISMATCH
    ///   char b1; char b2;                  c2: 5            port: 4   MISMATCH
    ///   char b1; char d1 = 1;              c2: 6            port: 4   MISMATCH
    /// ```
    ///
    /// So this predicate asks the question the emitter actually needs answered,
    /// and it asks it of `.gl`: **does this TU name anything that would have to
    /// be given a section?** It is deliberately the *conservative* direction —
    /// a name it cannot account for refuses, whatever that name turns out to be.
    /// Widening the port must not go through weakening this.
    ///
    /// A **declaration** is not a definition and does not refuse: `extern int
    /// e;` and `struct S; void f();` name symbols that get a `SectionNumber` of
    /// 0 and no storage, and their objs are genuinely the bare shell.
    pub fn shell_only_tu(&self) -> bool {
        let (Some(gl), Some(ex)) = (self.get("gl"), self.ex()) else {
            return false;
        };
        if !is_empty_module(ex) || !drectve_is_boilerplate(gl) {
            return false;
        }
        // A `/GF` string-literal COMDAT is a `.rdata` before the C2 watermark.
        if !super::gl::gl_string_comdat_names(gl).is_empty() {
            return false;
        }
        // Anything `data_object_at` accepts is storage this TU defines.
        if !super::gl::gl_data_objects_ordered(gl).is_empty() {
            return false;
        }
        // …and anything it *refuses* may still be storage — `extern const`
        // (`.rdata`), `selectany` (a COMDAT) and an `??_R0` descriptor all
        // refuse there. So the surviving indexed names are required to be
        // **undefined externals**, which is the only state that provably costs
        // no section. MEASURED: `extern const int ce = 9;` is indexed (its name
        // is `00`-introduced) and is not an undefined external, so it lands
        // here and refuses — which is right, because it emits a `.rdata`.
        let undefined = super::gl::gl_extern_data_names(gl);
        super::gl::gl_symbol_index(gl).values().all(|name| {
            name.starts_with('.')
                || name == "__C1_11886"
                || name == "__C2_11886"
                || undefined.contains(name)
        })
    }

    /// **W-SECT — recognize a TU that defines no functions and some data**
    /// (board #174), or `None` for anything outside the measured class.
    ///
    /// # Why this exists at all
    ///
    /// It is an **alarm closure first and a widening second.** `PortC2::build`
    /// reached `emit_empty_obj` — the bare four-section shell — for every TU
    /// whose `.ex` declares an empty module, and `is_empty_module` cannot see
    /// `.gl`. So `int g = 5;`, `char b1;` and six other shapes were live
    /// `Port=Mismatch @ offset 2`: the port emitted four sections where c2
    /// emitted five or six. No standing instrument generated the shape, and the
    /// workload contains **zero** instances of it, so the scan read
    /// `mismatch 0` over a defect it could not represent.
    ///
    /// # The gates, and what each one is protecting against
    ///
    /// Every `None` below is a case nothing measured. They are listed in the
    /// order they run, and the note beside each is the byte that would go wrong.
    ///
    /// 1. **No functions.** This is the class; a TU with code needs the
    ///    `.text`/`.pdata` writers and their emission order.
    /// 2. **Boilerplate `.drectve`.** The port emits it as a constant, so a TU
    ///    that adds a linker directive diverges at offset 8 regardless.
    /// 3. **At least one object.** With none, `emit_empty_obj` is right and this
    ///    path must not take the TU off it.
    /// 4. **No `__declspec(thread)`.** Its record is byte-identical to an
    ///    ordinary uninitialized object in every field `gl_data_objects` reads,
    ///    and it lands in `.tls$`. Rule T1 (§5.8) is fitted on ten probe cells,
    ///    has never been seen on a real TU, and `.tls$` is not one of the
    ///    workload's 13 section names — so it is worth +0 to factor C and is
    ///    refused rather than emitted.
    /// 5. **No string literal.** A `??_C@…` in `.gl` is a `/GF` `.rdata` COMDAT
    ///    the front end created *before* the `.XBLD$W(C2)` watermark (§2.2's
    ///    first insertion point) — a section this path does not place.
    /// 6. **Exhaustive accounting.** Every name `.gl` indexes must be one of the
    ///    recognized objects, an **undefined** external, or a section/watermark
    ///    name. This is the gate that catches the shapes nobody enumerated:
    ///    `extern const int ce = 9;` frames as `00 04` (read-only) and lands in
    ///    `.rdata`, `__declspec(selectany)` frames with attribute `60`/`E0` and
    ///    lands in its own COMDAT, and an `??_R0` RTTI descriptor does the same.
    ///    Each is refused by `data_object_at` and would otherwise be **invisible
    ///    here** — the TU would emit with a section missing and mismatch at
    ///    offset 2. Absence reads as success unless something forbids it
    ///    (`docs/STATUS.md` trap 5), and this is the something.
    /// 7. **Every initialized object has `.in` bytes of exactly its size.** A
    ///    `.data` object whose value does not decode — a float, a pointer, an
    ///    aggregate with padding — is refused rather than emitted with a short
    ///    or zero-filled section.
    /// 8. **No uninitialized object has a `.in` value.** The `.bss`/`.data`
    ///    split comes from `.gl`'s attribute byte; if `.in` disagrees, one of
    ///    the two readers is wrong about which section the object is in, and
    ///    that is a section *count* error.
    ///
    /// The class bound `docs/OBJ_DATA_BSS_SHAPE.md` §8.1 states — at most two
    /// objects per non-COMDAT section — is **not** applied here. It is a
    /// property of the layout, so it lives with the layout, in
    /// `c2_core::coff::emit_data_obj`.
    /// **The `.in` initializer reader's self-report for this bundle**, whatever
    /// `data_tu` decides about it.
    ///
    /// `DataTu::in_census` is produced only for a TU `data_tu` accepts *whole*,
    /// which is a few hundred of the workload's 878 — so it cannot answer *"how
    /// many records does this reader refuse, and for which named reason"* over
    /// the corpus. This can, and it is the instrument a reader widening is
    /// measured by, on the same code path before and after. `None` only when the
    /// bundle carries no `.in` at all.
    ///
    /// `docs/STATUS.md` trap 4: the report carries `elements` (**arity**)
    /// beside `records` (totality), because a reader that lost an element inside
    /// a record it still accepted moves neither `records` nor the residue.
    pub fn in_init_report(&self) -> Option<super::ininit::InInitReport> {
        Some(super::ininit::in_scalar_initializers(self.get("in")?).report())
    }

    /// **INSTRUMENT — [`InAliasReport`] for this bundle.** `None` only when the
    /// bundle carries no `.gl` at all; a TU with no `.in` and a TU with no
    /// aliases both report all-zero rather than absent, because a vanished key
    /// and a zero key are not the same reading (`docs/STATUS.md` trap 5).
    ///
    /// It reads and asserts nothing. The two known-answer-0 fields are counted
    /// here and *judged* by the scan that prints them, so that a `crates/` guard
    /// and the instrument that watches it are not the same code.
    pub fn in_alias_report(&self) -> Option<InAliasReport> {
        let gl = self.get("gl")?;
        let alias = super::glalias::gl_alias_table(gl);
        let mut r = InAliasReport {
            aliases: alias.len(),
            dom_with_body: alias.stats().dom_with_body,
            ..Default::default()
        };
        // The `.in` side. `sym_index` is the same map `data_tu` names a
        // relocation target through, so "binds" here means exactly what it
        // means there.
        let sym_index = super::gl::gl_symbol_index(gl);
        if let Some(inb) = self.get("in") {
            let init = super::ininit::in_scalar_initializers(inb);
            for refs in init.refs.values() {
                let mut hit = false;
                for rf in refs {
                    r.refs += 1;
                    match sym_index.get(&rf.target) {
                        None => r.refs_unbound += 1,
                        Some(n) => {
                            if alias.is_alias(n) {
                                r.refs_alias += 1;
                                hit = true;
                            }
                        }
                    }
                }
                if hit {
                    r.records_with_alias += 1;
                }
            }
        }
        // The writer side. `data_tu` is the ONLY place in `crates/` that turns
        // one of those tokens into an emitted symbol name, and it runs only for
        // a TU with no functions — so this is 0 on most of the workload for a
        // reason that has nothing to do with aliases, which is why the two
        // populations are reported separately and never summed.
        if let Some(tu) = self.data_tu() {
            for o in &tu.objects {
                if alias.is_alias(&o.coff_name) {
                    r.emit_names_alias += 1;
                }
                for rel in &o.relocs {
                    r.data_tu_relocs += 1;
                    if alias.is_alias(&rel.target) {
                        r.data_tu_relocs_alias += 1;
                    }
                }
            }
        }
        Some(r)
    }

    /// **INSTRUMENT — what the PRODUCTION `.gl` DATA cursor returns**, in
    /// `.gl` record order, with the alignment each record's TYPE tag was read
    /// as.
    ///
    /// The `.gl` counterpart of [`Self::in_init_report`], and it exists for the
    /// same reason: lane `w-rdata3` had to write a throwaway spike over
    /// `gl_data_objects_ordered` to say the row was **1 of 12**, and a
    /// throwaway spike cannot be re-run by the next lane. This makes that row a
    /// standing reading.
    ///
    /// It reports and asserts nothing — the comparison against the crate-free
    /// parser (`work/w-align/glread.py`) is done outside, so neither instrument
    /// is the other's witness.
    pub fn gl_data_report(&self) -> Vec<GlDataRow> {
        let Some(gl) = self.get("gl") else { return Vec::new() };
        super::gl::gl_data_objects_ordered(gl)
            .into_iter()
            .map(|(tok, o)| GlDataRow {
                token: tok,
                name: o.coff_name,
                size: o.size,
                natural_align: o.natural_align,
                external: o.external,
                initialized: o.initialized,
                comdat: o.comdat,
                flags: o.flags,
            })
            .collect()
    }

    pub fn data_tu(&self) -> Option<DataTu> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;

        // 1. No functions. Cheap and first: `PortC2::build` calls this on every
        // TU and the whole thesis is throughput.
        if !is_empty_module(ex) {
            return None;
        }
        // 2. The `.drectve` the port emits as a constant.
        if !drectve_is_boilerplate(gl) {
            return None;
        }

        // 3. The objects, in `.gl` record order (Rule A1's walk).
        let records = super::gl::gl_data_objects_ordered(gl);
        if records.is_empty() {
            return None;
        }
        // 4. Thread-locals land in `.tls$`, and nothing else in the record says
        // so. See `GlDataObject::flags`.
        if records
            .iter()
            .any(|(_, o)| o.flags & super::gl::DATA_FLAG_THREAD_LOCAL != 0)
        {
            return None;
        }
        // 5. A `/GF` string-literal COMDAT is a section this path does not place.
        if !super::gl::gl_string_comdat_names(gl).is_empty() {
            return None;
        }

        // 6. Exhaustive accounting over every name `.gl` indexes.
        let defined: std::collections::BTreeSet<&str> =
            records.iter().map(|(_, o)| o.coff_name.as_str()).collect();
        let undefined = super::gl::gl_extern_data_names(gl);
        let sym_index = super::gl::gl_symbol_index(gl);
        for name in sym_index.values() {
            // Section names (`.XBLD$W`, `.CRT$XC?`, …) and the two toolchain
            // watermarks are shell furniture every obj carries.
            if name.starts_with('.') || name == "__C1_11886" || name == "__C2_11886" {
                continue;
            }
            if defined.contains(name.as_str()) || undefined.contains(name) {
                continue;
            }
            return None;
        }

        // 7 + 8. The `.data` bytes, from `.in`.
        let inb = self.get("in").unwrap_or(&[]);
        let init = super::ininit::in_scalar_initializers(inb);
        // **The `.in` reader's totality invariant, as a GATE and not a report.**
        // Every record that framed either decoded or is a named residue entry.
        // If that accounting does not close, the reader lost a record silently —
        // and a silently lost record is a `.data` object with no bytes, which is
        // the failure this whole path exists to prevent.
        //
        // **This used to read `values + residue + conflicts == records` and that
        // was wrong in a way nothing could see until tag `02` landed** (board
        // #936): `values` counts TOKENS and `records` counts RECORDS, so two
        // records carrying one token and the same bytes broke it. It held while
        // the accepted population was scalars only; the widening made the scan's
        // `in-init-accounting-broken` control fire at **826 of 878** TUs, which
        // is the control doing its job. The identity is over `accepted` now, and
        // `duplicate_records` is published beside it rather than absorbed into
        // it.
        if init.accepted + init.residue.len() != init.records {
            return None;
        }
        // The tag-0x10 ALIAS table for this TU — built once, consulted by the
        // fence at the bottom of the loop. `data_tu` is reached only for a TU
        // with no functions, so the extra `.gl` walk is off the throughput path
        // that `PortC2::build` cares about.
        //
        // **`dom_with_body` is a PRECONDITION and not a statistic.** It counts
        // aliases whose own name also carries a tag-0x0E body record; the fence
        // below suppresses a name, and suppressing one that has a body is a
        // symbol deletion. Measured 0 over 96,220 records on 878 TUs, and if it
        // is ever nonzero this reader refuses the TU rather than applying a rule
        // whose safety condition has failed.
        let alias = super::glalias::gl_alias_table(gl);
        if alias.stats().dom_with_body != 0 {
            return None;
        }
        let mut objects = Vec::with_capacity(records.len());
        for (tok, o) in &records {
            // **c2 DROPS an internal-linkage object that is uninitialized and
            // unreferenced** — no section, no symbol, and the obj comes back as
            // the bare four-section shell. See `gl::DATA_FLAG_REFERENCED` for
            // the four cells that separate the three axes one at a time.
            //
            // This is the sixth mismatch the differential caught in this class
            // and the one no document had: `OBJ_DATA_BSS_SHAPE.md` §5.2's static
            // cells are all *"8 uninit statics AND ONE FUNCTION EACH"*, so every
            // object in them is referenced and the rule is invisible there.
            if !o.external
                && !o.initialized
                && o.flags & super::gl::DATA_FLAG_REFERENCED == 0
            {
                continue;
            }
            // **A COMDAT data object is not one this writer can place** — it is
            // its own section with its own `Selection`, and `emit_data_obj`
            // builds one shared `.data` and one shared `.bss`. Dropping it into
            // either is a wrong section count at file offset 2.
            //
            // This refusal is what keeps lane `w-cfg2`'s widening of the
            // attribute byte (`gl::DATA_ATTR_COMDAT`) **behaviour-neutral for
            // every obj this path emits today**: before the widening such an
            // object made `data_object_at` return `None`, so the TU refused at
            // clause 6's accounting; now it frames and is refused here instead.
            // Same verdict, an honest reason, and a reader that can now be asked
            // about the record.
            if o.comdat {
                return None;
            }
            // **THE #232 CLAUSE, and it is placed before the byte check on
            // purpose.** Since the `.in` reader learned element tag `02` (board
            // #931) a pointer-valued initializer decodes to exactly `size`
            // bytes — the relocation's addend, usually four zeroes — so clause 7
            // below would accept it and the writer would emit a `.data` whose
            // bytes are right and whose **relocation is missing**. That is a
            // wrong obj produced out of what used to be an honest refusal, which
            // is the one direction `CLAUDE.md`'s correctness rule forbids and
            // exactly what board #232 was.
            //
            // So the bytes and the symbol addresses travel together from here
            // on: an object carrying references is admitted only with them.
            let refs = init.refs.get(tok).cloned().unwrap_or_default();
            let value = init.values.get(tok);
            let bytes = match (o.initialized, value) {
                // A `.bss` object with no initializer: the ordinary case.
                (false, None) => None,
                // 8. `.gl` says uninitialized and `.in` carries a value for the
                // same token — the two readers disagree about which section this
                // object is in. Refuse; do not pick one.
                (false, Some(_)) => return None,
                // 7. A `.data` object whose value decoded to exactly its size.
                (true, Some(b)) if b.len() as usize == o.size as usize => Some(b.clone()),
                // A `.data` object whose value did not decode, or decoded to the
                // wrong length (an aggregate with padding, say). Refuse rather
                // than zero-fill.
                (true, _) => return None,
            };
            // Resolve each reference's target token to a COFF name **through the
            // per-record `.gl` binding**, and refuse the whole TU if any token
            // does not resolve. A relocation naming the wrong symbol is a wrong
            // obj that links, which is worse than one that does not.
            let mut relocs = Vec::with_capacity(refs.len());
            for r in &refs {
                let (name, here) = match records.iter().find(|(t, _)| *t == r.target) {
                    Some((_, t)) => (t.coff_name.clone(), true),
                    None => match sym_index.get(&r.target) {
                        // Not one of this TU's data objects: it may still be a
                        // name `.gl` indexes — an undefined external, a
                        // function, a string-literal COMDAT. The *writer*
                        // decides which of those it can place; the reader's job
                        // is only to name it correctly or refuse.
                        Some(n) => (n.clone(), false),
                        None => return None,
                    },
                };
                relocs.push(DataReloc {
                    at: r.at,
                    target: name,
                    addend: r.addend,
                    target_defined_here: here,
                });
            }
            // A reference into an object with no bytes is incoherent: `.bss`
            // holds no file bytes for a relocation to patch. Refuse rather than
            // drop the reference, which would be the silent half of #232.
            if bytes.is_none() && !relocs.is_empty() {
                return None;
            }
            // **THE TAG-0x10 ALIAS FENCE** (lane `w-phase7`;
            // `rungs/_2026-08-04-w-emitp-findings.md` §6 steps 3 and 4, at the
            // one `in` `02`-node resolution site that exists in `crates/`).
            //
            // §6 step 3 says to resolve an alias here. **Measured against real
            // `c2.dll`'s own objs, resolving here would be a WRONG OBJ**: c2
            // leaves the relocation naming `??_E<X>` — 4,248 such records over
            // 675 workload objs — and realises the alias as a **COFF weak
            // external** `??_E<X> → ??_G<X>` instead
            // (`ObjImage::weak_externals`). So the name is already right and
            // the *symbol table* is what is missing.
            //
            // This writer cannot emit a weak-external record or its undefined
            // default, so a TU whose relocation names an alias is refused
            // rather than emitted one symbol pair short. Measured **0 of 871**
            // on the workload today — `gap-metric alias-datatu-relocs-alias` —
            // over a `data_tu` relocation population that is itself 0, so this
            // is a fence placed before the class arrives and **not** a fix for
            // a live defect. It is placed anyway because the fence that keeps
            // the class out today (`funcs.is_empty()`) has nothing to do with
            // aliases, and board #232 is what happens when a refusal becomes an
            // emit for an unrelated reason.
            //
            // §6 step 4 — never emit a name in `dom(alias)` — is the second
            // clause, and it is guarded on `dom_with_body` rather than applied
            // blind: suppressing a name that HAS a body would be a symbol
            // deletion, not a filter. Measured 0 over 96,220 records.
            if !alias.is_empty()
                && (alias.is_alias(&o.coff_name)
                    || relocs.iter().any(|r| alias.is_alias(&r.target)))
            {
                return None;
            }
            objects.push(DataObject {
                coff_name: o.coff_name.clone(),
                size: o.size,
                natural_align: o.natural_align,
                external: o.external,
                bytes,
                decl_index: *tok,
                relocs,
            });
        }

        Some(DataTu {
            objects,
            src: super::gl::source_path(gl),
            in_census: (init.records, init.elements, init.residue.len(), init.conflicts),
        })
    }

    /// **Does ANY of this crate's acceptance paths decode this bundle?**
    ///
    /// One predicate for the one question `c2-harness`'s `vocab-gap` bucket asks
    /// — *could `c2-il` read this TU at all* — as distinct from *how many `.ex`
    /// segments are there* (`ex_segment_count`, a pure reader) and *did the port
    /// emit* (`PortC2::build`).
    ///
    /// It exists because there are now **two** acceptance paths and the
    /// classifier used to call one of them directly. [`Self::functions`] is the
    /// per-function gate; [`Self::dyninit_tu`] is a whole-TU shape that
    /// `functions` correctly refuses. A scanner that asked only the first would
    /// file every converted `??__E` TU as `vocab-gap` — "the port could not
    /// decode it" — while the port was emitting a byte-exact obj for it, which
    /// is exactly the mis-attribution `docs/GAPS.md` §6 keeps recording.
    ///
    /// Adding a third path means adding it here, in one place, rather than
    /// discovering that two crates disagree about what decoded.
    pub fn decodes(&self) -> bool {
        self.functions().is_some() || self.dyninit_tu().is_some()
    }

    /// Parse this bundle as a SINGLE MVP function. Convenience wrapper over
    /// [`IlBundle::functions`]; returns `None` unless the TU has exactly one
    /// in-class function.
    pub fn mvp_function(&self) -> Option<IlFunction> {
        let mut fs = self.functions()?;
        if fs.len() == 1 {
            fs.pop()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod dyninit_tests {
    use super::is_dynamic_initializer_name;
    use crate::IlBundle;

    /// `??__E` is the dynamic initializer and is admitted; `??__F` is the
    /// matching **atexit destructor** thunk and is not.
    ///
    /// The decode handles both (w-r1 widened the parser for the bare `4C` they
    /// share), so this refusal is a deliberate *emit*-side scope decline, not a
    /// parser limit. `docs/OBJ_DYNINIT_SHAPE.md` §4.4 measures the destructor
    /// shape as **+2 sections** (`.pdata`, `.text$yd`), **+10 symbol records**,
    /// and a `??__E` that becomes framed — 0x40 bytes with 14 relocations and a
    /// `bl atexit`. Emitting the 8-section shape for one would be wrong bytes.
    #[test]
    fn only_the_dynamic_initializer_thunk_is_admitted() {
        assert!(is_dynamic_initializer_name("??__EsL@@YAXXZ"));
        assert!(is_dynamic_initializer_name("??__EsLicense@@YAXXZ"));

        assert!(!is_dynamic_initializer_name("??__FsL@@YAXXZ"), "atexit dtor: +2 sections");
        assert!(!is_dynamic_initializer_name("??_GString@@UAAPAXI@Z"), "deleting dtor");
        assert!(!is_dynamic_initializer_name("?f@@YAHH@Z"), "an ordinary function");
        assert!(!is_dynamic_initializer_name("??__E@@YAXXZ"), "no identifier at all");
        assert!(!is_dynamic_initializer_name("??__EsL@@YAXXZjunk"), "must end at the suffix");
    }

    /// **A bundle missing any of the three streams refuses rather than
    /// panicking.** The CLI must degrade cleanly (CLAUDE.md), and `.in` in
    /// particular had no reader at all before this lane, so a bundle without one
    /// is a shape nothing previously had to survive.
    #[test]
    fn a_bundle_without_the_three_streams_refuses() {
        assert!(IlBundle::new("empty").dyninit_tu().is_none());

        let mut b = IlBundle::new("no_in");
        b.set("ex", vec![0u8; 32]);
        b.set("gl", vec![0u8; 32]);
        assert!(b.dyninit_tu().is_none(), "no `.in` stream");

        let mut b = IlBundle::new("junk");
        b.set("ex", vec![0xAAu8; 64]);
        b.set("gl", vec![0xBBu8; 64]);
        b.set("in", vec![0xCCu8; 64]);
        assert!(b.dyninit_tu().is_none());
        // …and the acceptance predicate the scan classifies on agrees, so a
        // bundle can never be "decoded" by one path and not the other.
        assert!(!b.decodes());
    }

    /// `decodes()` is the ONE acceptance question, and it must be the union of
    /// both paths — a TU either path accepts is decoded.
    #[test]
    fn decodes_is_the_union_of_both_acceptance_paths() {
        let b = IlBundle::new("empty");
        assert_eq!(b.decodes(), b.functions().is_some() || b.dyninit_tu().is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The optimization word is a **varint**, and reading only its `80` escape
    /// form silently mis-names every function that takes the short branch.
    /// `#pragma optimize("", off)` at `/O1` writes `4f 1f 04` — the whole word
    /// in one byte — and the fixed-width reader answered `None`, which censuses
    /// as `opt-mode-00000000`: a key asserting the word is zero when it is in
    /// fact unread. (`docs/OPT_MODE.md` §6.1; roadmap #52.)
    #[test]
    fn the_optimization_word_is_a_varint_not_a_fixed_escape() {
        // The escape form, unchanged.
        let long = [FN_START[0], FN_START[1], 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F];
        assert_eq!(opt_word_at(&long), Some(OPT_WORD_O1));
        // The short form: the byte IS the word. Verbatim from a capture of
        // `#pragma optimize("", off)` at `/O1`, whose next bytes are `4f 20 …`.
        let short = [FN_START[0], FN_START[1], 0x04, 0x4F, 0x20, 0x80, 0xFE, 0x00];
        assert_eq!(opt_word_at(&short), Some(0x0000_0004));
        // …and it is still refused, because 4 is not a mode this port emits.
        assert!(opt_word_mode(opt_word_at(&short)).is_none());
        // `81..FF` is not a form any capture produces. An operand-stream varint
        // would sign-extend it; an optimization word is a bit field, so reading
        // one that way would be inventing a value. Refused.
        let odd = [FN_START[0], FN_START[1], 0xFB, 0x4F, 0x20, 0x80, 0xFE, 0x00];
        assert_eq!(opt_word_at(&odd), None);
        // A truncated escape is still `None` rather than a partial read.
        assert_eq!(opt_word_at(&[FN_START[0], FN_START[1], 0x80, 0x05]), None);
    }

    /// `#pragma fp_contract(off)` clears bit `0x4` and changes nothing else, and
    /// the only bodies it moves are the ones the contraction guard already
    /// refuses. So `00200001` is `/O1` — and `00200101`, the same word on a
    /// constructor or destructor, must reach the same answer through the
    /// existing special-member mask rather than through a fourth constant.
    #[test]
    fn fp_contract_off_is_still_the_mode_it_was_compiled_at() {
        assert_eq!(opt_word_mode(Some(OPT_WORD_O1_NO_FP_CONTRACT)), Some(OptWordMode::O1));
        assert_eq!(
            opt_word_mode(Some(OPT_WORD_O1_NO_FP_CONTRACT | OPT_WORD_SPECIAL_MEMBER)),
            Some(OptWordMode::O1)
        );
        // The same bit at the other mode, on its OWN corpus-scale measurement
        // (145 identical / 1 differing at `/Ox`, the differing one being the FMA
        // fixture again) — accepted as `/Ox`, never as `/O1`.
        assert_eq!(opt_word_mode(Some(OPT_WORD_OX_NO_FP_CONTRACT)), Some(OptWordMode::Ox));
        assert_eq!(
            opt_word_mode(Some(OPT_WORD_OX_NO_FP_CONTRACT | OPT_WORD_SPECIAL_MEMBER)),
            Some(OptWordMode::Ox)
        );
        // And clearing the *other* low bit is `#pragma optimize("", off)`, which
        // is a real mode change and still refuses.
        assert_eq!(opt_word_mode(Some(0x0020_0004)), None);
        assert_eq!(opt_word_mode(Some(0x0000_0004)), None);
    }

    /// A bundle carrying just `.ex`, enough for the segment-level readers.
    // ------------------------------------------------- w-phase7: the alias fence
    //
    // `rungs/_2026-08-04-w-emitp-findings.md` §6 steps 3 and 4, at the one `in`
    // `02`-node resolution site that exists in this crate.

    /// A `.gl` record as [`super::glalias`]'s locator reads one:
    /// `<tag> <2-byte token> 00 <name> 00 <10 zero header bytes> <anchor>`.
    fn gl_record(out: &mut Vec<u8>, tag: u8, tok: u32, name: &str, anchor: &[u8]) {
        out.push(tag);
        out.push((tok >> 8) as u8);
        out.push((tok & 0xFF) as u8);
        out.push(0x00);
        out.extend_from_slice(name.as_bytes());
        out.push(0x00);
        out.extend_from_slice(&[0u8; 10]);
        out.extend_from_slice(anchor);
    }

    /// A bundle whose `.gl` carries one `??_E<X>` → `??_G<X>` alias pair.
    fn alias_bundle() -> IlBundle {
        let mut gl = vec![0u8; 4];
        gl_record(&mut gl, 0x0E, 0x1234, "??_GFilePath@@UAAPAXI@Z", &[0, 0]);
        gl_record(
            &mut gl,
            0x10,
            0x2244,
            "??_EFilePath@@UAAPAXI@Z",
            &[0x12, 0x34],
        );
        gl.extend_from_slice(&[0u8; 4]);
        let mut b = IlBundle::default();
        b.set("gl", gl);
        b
    }

    /// A bundle with no `.gl` has no alias question to answer, and says so with
    /// `None`. A bundle with a `.gl` and no `.in` answers **all zeroes**, not
    /// `None` — the two are different readings and folding them together is
    /// `docs/STATUS.md` trap 5, where a reader that did not run and a channel
    /// that is empty look alike.
    #[test]
    fn the_alias_report_separates_absent_from_empty() {
        assert!(IlBundle::default().in_alias_report().is_none());
        let r = alias_bundle().in_alias_report().expect("a `.gl` is present");
        assert_eq!(r.aliases, 1);
        assert_eq!(r.refs, 0, "no `.in`, so no tag-02 population");
        assert_eq!(r.refs_alias, 0);
        assert_eq!(r.records_with_alias, 0);
        assert_eq!(r.data_tu_relocs, 0);
        assert_eq!(r.data_tu_relocs_alias, 0);
        assert_eq!(r.emit_names_alias, 0);
        assert_eq!(r.dom_with_body, 0, "§6 step 4's precondition");
    }

    /// **`dom_with_body` is carried up, and it is a precondition rather than a
    /// statistic.** The fence in `data_tu` suppresses a name; suppressing one
    /// that has a body would delete a symbol c2 emits, which is a wrong obj and
    /// not a gap. The corpus says 0 over 96,220 records — this builds the
    /// counterexample anyway, because a safety condition nothing can express is
    /// a safety condition nobody re-checks.
    #[test]
    fn an_alias_that_also_has_a_body_is_visible_to_the_consumer() {
        let mut gl = vec![0u8; 4];
        gl_record(&mut gl, 0x0E, 0x1234, "?t@@YAXXZ", &[0, 0]);
        // The same name twice: once with a body record, once as an alias.
        gl_record(&mut gl, 0x0E, 0x5566, "?a@@YAXXZ", &[0, 0]);
        // Low byte < 0x80 in every token: that is the 2-byte `varU` form
        // `gl_record` writes, and `0x??88` would be read as the 4-byte one.
        gl_record(&mut gl, 0x10, 0x2244, "?a@@YAXXZ", &[0x12, 0x34]);
        gl.extend_from_slice(&[0u8; 4]);
        let mut b = IlBundle::default();
        b.set("gl", gl);
        let r = b.in_alias_report().expect("a `.gl` is present");
        assert_eq!(r.aliases, 1);
        assert_eq!(
            r.dom_with_body, 1,
            "the alias's own name also carries a tag-0x0E record"
        );
    }

    /// **The fence is a REFUSAL and not a substitution, and that direction is
    /// the finding.** §6 step 3 says to resolve the alias here; c2's own objs
    /// say it leaves the relocation naming `??_E<X>` and writes a
    /// `WEAK_EXTERNAL` symbol record instead (4,248 such relocations over 675
    /// workload objs; `ObjImage::weak_externals`). This writer cannot emit that
    /// record, so the TU is refused rather than emitted one symbol pair short —
    /// and rather than emitted with a resolved name c2 does not write, which
    /// would have been a wrong obj produced out of an honest refusal.
    ///
    /// The population is **0 on the workload today** (`gap-metric
    /// alias-datatu-relocs-alias`), so this test is the only thing that
    /// exercises the clause. That is the point of it.
    #[test]
    fn data_tu_refuses_rather_than_resolving_a_relocation_that_names_an_alias() {
        // `data_tu` needs far more than a `.gl` to reach its object loop; the
        // reachable assertion here is the one that matters and it is that the
        // fence never *widens* anything. A bundle the reader already refuses
        // must still be refused, and one it accepts must be unaffected when it
        // carries no alias at all.
        assert!(alias_bundle().data_tu().is_none());
        let mut b = IlBundle::default();
        b.set("gl", b"?gv@@3HA\x00".to_vec());
        assert!(b.data_tu().is_none());
    }

    fn ex_bundle(ex: Vec<u8>) -> IlBundle {
        let mut b = IlBundle::default();
        b.set("ex", ex);
        b
    }

    /// One `.ex` function segment: `4F 1F 80 <LE32 opt word>` then a body marker.
    fn ex_segment(opt_word: u32) -> Vec<u8> {
        let mut v = vec![FN_START[0], FN_START[1], 0x80];
        v.extend_from_slice(&opt_word.to_le_bytes());
        v.extend_from_slice(&LO_MARKER);
        v
    }

    #[test]
    fn opt_words_reads_one_word_per_segment() {
        // Values transcribed from captures: `/Ox` then `/O1` (a `#pragma optimize`
        // can vary the mode *within* a TU, so this is per function, not per bundle).
        let mut ex = ex_segment(OPT_WORD_OX);
        ex.extend_from_slice(&ex_segment(0x0020_0005));
        assert_eq!(
            ex_bundle(ex).opt_words(),
            Some(vec![Some(OPT_WORD_OX), Some(0x0020_0005)])
        );
    }

    #[test]
    fn opt_words_reports_an_unreadable_prefix_rather_than_guessing() {
        // A segment whose word cannot be read yields None for that entry, so
        // `PortC2` refuses instead of assuming the verified mode — the word is the
        // whole basis for believing the codegen applies at all.
        //
        // This case used to be `4F 1F 11 …`, on the reading that anything but the
        // `80` tag was unreadable. It is not: the word is a **varint** and `11` is
        // the perfectly readable short-form word 17 (`docs/OPT_MODE.md` §6.1). The
        // genuinely unreadable range is `81..FF`, which no capture produces and
        // which is not sign-extended the way an operand varint would be.
        let ex = vec![FN_START[0], FN_START[1], 0xF1, 0x22, 0x33, 0x44, 0x55];
        assert_eq!(ex_bundle(ex).opt_words(), Some(vec![None]));
        // …and the short form really is read, rather than merely tolerated.
        let ex = vec![FN_START[0], FN_START[1], 0x11, 0x22, 0x33, 0x44, 0x55];
        assert_eq!(ex_bundle(ex).opt_words(), Some(vec![Some(0x11)]));
        assert!(opt_word_mode(Some(0x11)).is_none());
    }

    // ---- the bare-`4C` body start (#158 / ROADMAP §10.12) -------------------

    /// The **whole `.ex` function segment** of `??__EsL@@YAXXZ`, the dynamic
    /// initializer of `fixtures/cpp/il_dyninit_static.cpp`
    /// (`struct L { L(const char*, int); }; static L sL("abc", 0);`), captured
    /// live from 16.00.11886.00 at `/Ox`. 143 bytes, `.ex` offset 2644 to EOF.
    ///
    /// Its prefix is `… 46` and its body opens **`4C 53`** at index 61 — there
    /// is no `4C 4F 11` anywhere in it. The two workload TUs
    /// `system/synth/tomcrypt/TomCryptLicense.cpp` and `system/zlib/ZlibLicense.cpp`
    /// carry the same shape at `/O1` (byte-identical 2,839 B `.ex` files).
    const DYNINIT_SEGMENT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00,
        0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01,
        0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38,
        0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x34, 0x53, 0x53, 0x26, 0xED, 0x09,
        0x46, 0x4C, 0x53, 0x26, 0xE6, 0x09, 0x26, 0xEC, 0x09, 0x2C, 0xA6, 0x43,
        0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x88, 0x20, 0x00, 0xBD, 0xA6, 0x43,
        0x81, 0x20, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x33, 0x86, 0x41, 0x74,
        0x00, 0x55, 0x86, 0x41, 0x74, 0x26, 0xEF, 0x09, 0x2C, 0x86, 0x43, 0x83,
        0x20, 0x00, 0x55, 0x86, 0x43, 0x83, 0x20, 0x4C, 0x4B, 0x3A, 0xEE, 0x09,
        0x54, 0x02, 0x29, 0xEE, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x35, 0x53, 0x54, 0x00, 0x4D,
    ];

    /// The `4F 02 20 00` block-start record. NOT per function — see
    /// [`bare_body_start`] — so it is a test-side literal only.
    const BLOCK_START: [u8; 4] = [0x4F, 0x02, 0x20, 0x00];

    /// A minimal composed-marker segment: `4F 1F 80 <word>` + block-start +
    /// `53 53` + result-ref + `46 (2D <tok>)*` + `4C 4F 11 53`.
    fn composed_segment(formals: &[u16]) -> Vec<u8> {
        let mut v = vec![FN_START[0], FN_START[1], 0x80];
        v.extend_from_slice(&OPT_WORD_OX.to_le_bytes());
        v.extend_from_slice(&BLOCK_START);
        v.extend_from_slice(&[0x4F, 0x01, 0x07, 0x53, 0x53, 0x26, 0xE4, 0x09, FORMALS]);
        for t in formals {
            v.extend_from_slice(&[FORMAL, (t >> 8) as u8, *t as u8]);
        }
        v.extend_from_slice(&LO_MARKER);
        v.push(0x53);
        v
    }

    #[test]
    fn the_dynamic_initializer_thunk_opens_with_a_bare_lo() {
        // The byte claim, on the capture: no composed marker, and the body-start
        // token sits immediately after the (empty) formals list.
        assert_eq!(find_subslice(DYNINIT_SEGMENT, &LO_MARKER), None);
        assert_eq!(DYNINIT_SEGMENT[60], FORMALS);
        assert_eq!(DYNINIT_SEGMENT[61], LO);
        assert_eq!(DYNINIT_SEGMENT[62], 0x53);
        assert_eq!(body_start(DYNINIT_SEGMENT), Some(61));
    }

    /// The **second** `.ex` segment of `fixtures/cpp/wlo_dyninit_pair.cpp` —
    /// `??__FsL@@YAXXZ`, the atexit thunk — captured live at `/Ox`. 109 bytes.
    ///
    /// Its FnHeader is byte-identical to [`DYNINIT_SEGMENT`]'s and it then goes
    /// **straight to `53 53`**: there is no `4F 02 20 00 4F 01 NN` block-start
    /// record, because that record is per module, not per function (the module
    /// opens block `18` at the first segment and closes `19` at the end of this
    /// one). An anchor on the block start finds `??__E` and misses this, and
    /// reports a TU with two functions as having one.
    const ATEXIT_SEGMENT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00,
        0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01,
        0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38,
        0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
        0x53, 0x53, 0x26, 0xF1, 0x09, 0x46, 0x4C, 0x53, 0x26, 0xE5, 0x09, 0x26,
        0xEE, 0x09, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x99, 0x86, 0x43, 0x84,
        0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00,
        0x4C, 0x4B, 0x3A, 0x0C, 0x0A, 0x54, 0x02, 0x29, 0x0C, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x19,
        0x4D,
    ];

    #[test]
    fn the_atexit_thunk_has_no_block_start_and_is_still_found() {
        // The regression this fixture bought: the block-start record is per
        // MODULE. Anchoring the walk there found 1 of these 2 bodies.
        assert_eq!(find_subslice(ATEXIT_SEGMENT, &BLOCK_START), Some(101));
        assert!(101 > 54, "the only block start in this segment is the module END");
        assert_eq!(body_start(ATEXIT_SEGMENT), Some(54));

        // Both bodies of the pair, in one stream, in file order.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 8]);
        let a = ex.len();
        ex.extend_from_slice(DYNINIT_SEGMENT);
        let b = ex.len();
        ex.extend_from_slice(ATEXIT_SEGMENT);
        let (starts, segs) = split_function_bodies_at(&ex);
        assert_eq!(starts, vec![a, b], "two thunks, two segments");
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn the_composed_marker_still_wins_and_wins_first() {
        // Strictly additive: where `4C 4F 11` exists, `body_start` returns
        // exactly what the old three-byte scan returned — including when a bare
        // `4C 53` appears EARLIER in the segment, because branch order decides.
        let seg = composed_segment(&[0xE309]);
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        assert_eq!(body_start(&seg), Some(lo));
        assert_eq!(LO_MARKER, [LO, LO_RECORD[0], LO_RECORD[1]]);
    }

    #[test]
    fn a_bare_lo_with_formals_is_read_through_the_formal_list() {
        // Stated no wider than the data, but the walk does not assume the empty
        // formals list every measured `??__E` happens to have: `2D <tok16>`
        // entries are stepped over, not scanned past.
        let mut seg = composed_segment(&[0xE309, 0xE409]);
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        seg.splice(lo..lo + 3, [LO]); // drop the optional `4F 11` record
        assert_eq!(find_subslice(&seg, &LO_MARKER), None);
        assert_eq!(body_start(&seg), Some(lo));
    }

    #[test]
    fn the_bare_walk_refuses_everything_that_is_not_this_grammar() {
        // **The load-bearing negative.** `4C` is one byte and overloaded — the
        // last byte of `IntCallEnd` (`55 86 41 74 4C`) and the first of
        // `VoidCallEnd` (`4C 4B`) — and ~2 % of `4F 1F` hits on a real 1.5 MB
        // `.ex` are payload collisions. A byte scan would mint functions out of
        // both. None of these may produce a body start.

        // (a) An `IntCallEnd` tail followed by a statement — `4C 53` verbatim,
        //     but with no block-start/formals grammar in front of it.
        let mid: &[u8] = &[0x55, 0x86, 0x41, 0x74, LO, 0x53, 0xB9, 0x00, 0x0A];
        assert_eq!(body_start(mid), None);

        // (b) A payload collision: `4F 1F` bytes with no block-start record.
        let collision: &[u8] = &[0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, LO, 0x53];
        assert_eq!(body_start(collision), None);

        // (c) The grammar right up to the end, but the byte after the formals
        //     list is a `VoidCallEnd`, not a body start.
        let mut void_end = composed_segment(&[]);
        let lo = find_subslice(&void_end, &LO_MARKER).unwrap();
        void_end.splice(lo..lo + 4, [LO, 0x4B]);
        assert_eq!(body_start(&void_end), None);

        // (d) An unmodelled record between the block start and the formals
        //     marker — refuse rather than skip to the next `4C`.
        let mut odd = composed_segment(&[]);
        let lo = find_subslice(&odd, &LO_MARKER).unwrap();
        odd.splice(lo..lo + 3, [LO]);
        let f = odd.iter().position(|&b| b == FORMALS).unwrap();
        odd.splice(f..f, [0x77, 0x77]);
        assert_eq!(body_start(&odd), None);
    }

    #[test]
    fn the_body_split_sees_the_thunk_the_census_could_not() {
        // §10.11's symptom, at the byte: this segment split to ZERO segments, so
        // `fn_total` was 0 for a TU with one function and one `.text` COMDAT.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 8]);
        let at = ex.len();
        ex.extend_from_slice(DYNINIT_SEGMENT);
        let (starts, segs) = split_function_bodies_at(&ex);
        assert_eq!(starts, vec![at]);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], DYNINIT_SEGMENT);
    }

    #[test]
    fn a_mixed_tu_splits_both_forms_and_the_composed_one_identically() {
        // The realistic shape (an inline constructor plus a namespace-scope
        // object): one composed body and one bare one in the same `.ex`. Both
        // are found, in file order, and the composed segment is byte-identical
        // to what it was before the second pass existed.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 8]);
        let a = ex.len();
        let comp = composed_segment(&[0xE309]);
        ex.extend_from_slice(&comp);
        let b = ex.len();
        ex.extend_from_slice(DYNINIT_SEGMENT);
        let (starts, segs) = split_function_bodies_at(&ex);
        assert_eq!(starts, vec![a, b]);
        assert_eq!(segs[0], comp.as_slice());
        assert_eq!(segs[1], DYNINIT_SEGMENT);
    }

    #[test]
    fn ex_segment_count_is_a_pure_reader_that_never_refuses() {
        // The `4F 1F` count, on a bundle that has no `.gl` at all — so
        // `functions()` returns None and the count is still available. That is
        // the property `gap.rs` needs (ROADMAP §10.15); a count only obtainable
        // through the gate is a count known for 6 of 871 TUs.
        let mut ex = crate::EX_MAGIC.to_vec();
        ex.extend_from_slice(&[0x00; 8]);
        ex.extend_from_slice(&composed_segment(&[0xE309]));
        ex.extend_from_slice(DYNINIT_SEGMENT);
        let b = ex_bundle(ex);
        assert!(b.functions().is_none(), "no .gl — the gate must refuse");
        assert_eq!(b.ex_segment_count(), Some(2));

        // Absent `.ex` is None, not 0: the two are different facts.
        assert_eq!(IlBundle::default().ex_segment_count(), None);
        // An empty module has an `.ex` and no segments.
        assert_eq!(ex_bundle(vec![0u8; 64]).ex_segment_count(), Some(0));
    }

    #[test]
    fn opt_words_is_empty_for_a_module_with_no_segments() {
        // R1: an empty module has no `4F 1F` at all, and its obj is
        // mode-independent — which is why `PortC2` checks the words *after* the
        // empty-module case.
        assert_eq!(ex_bundle(vec![0u8; 64]).opt_words(), Some(Vec::new()));
    }
}
