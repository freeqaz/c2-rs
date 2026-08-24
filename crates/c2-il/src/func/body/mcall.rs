//! **D2 — the `26`-in-expression decode, without acceptance.**
//!
//! `parse_expr` used to refuse every `0x26` it met with one census key,
//! `expr-call-in-expr`, which at 286,240 functions (12.9 % of the blocked dc3
//! workload) was the #1 blocker and named exactly 0.2 % of its own contents
//! (`docs/IL_CALL_IN_EXPR.md` §0). This module walks the production far enough to
//! say **which construct** the `26` opened, so the one bucket becomes a set of
//! named sub-buckets. It **accepts nothing**: every entry point returns a
//! [`Block`], the gate is byte-for-byte unchanged, and only the census key moves —
//! the same honest move as the intrinsic-selector decode (`docs/IL_INTRINSIC_CALL.md`).
//!
//! Two instrument failures recorded in `GAPS.md` §6 are what the design is against:
//!
//! * **Sharded keys.** Nothing per-TU may enter a key. The walk reads tokens
//!   (per-TU ids), inline TYPEs (per-TU ids), function-type ids (per-TU) and the
//!   `66` class-pair descriptor (per-TU type refs), and *none* of them reaches
//!   [`CallForm`]. The only payload that does is an **intrinsic selector**, which
//!   is a fixed c1xx-internal enum shared by every TU (`intrinsic_name`), and a
//!   raw **opcode byte** in the residue. So the bucket count is bounded by the
//!   grammar, not by the corpus.
//! * **Mis-attribution.** The histogram must file a function by the *construct*,
//!   not by where the parse stopped. So the key is **not** the byte the walk
//!   ended on: it is the form of the value that the decisive token consumed. A
//!   member call is filed by its **receiver designator**, wherever in the
//!   statement it appeared — probe `r_load` (`x = p->Get();`, an assignment
//!   right-hand side) and probe `r_arg` (`x = g1(p->Get());`, a call-argument
//!   region) are the same construct and land in the same bucket, though the
//!   enclosing `parse_expr` differs. `docs/IL_CALL_IN_EXPR.md` §9.2 is the reason
//!   that matters: statement position, not construct, decides which *bucket the
//!   whole function* lands in, and a decomposition that repeated that mistake
//!   inside the bucket would measure nothing.
//!
//! ## Why a backward classification over a forward walk
//!
//! The member-call spine is
//! `26 <method>… <receiver> 99 <T> 00 BD <ret> <conv> <id> (<arg> 55 <T>)* 4C`
//! (`docs/IL_CALL_IN_EXPR.md` §3) and the method symbols stack **LIFO**: a chain
//! `p->Next()->Val()` pushes *two* method symbols before one receiver, so the
//! run of `26 <tok>` pushes at the head of the production cannot be split into
//! "methods" and "the receiver" by looking forward — `26 <A> 26 <B> 2C … 99` has
//! `B` as the receiver while `26 <A> 26 <B> B9 … 99` has `B` as a second method.
//!
//! This walker therefore does not try. It tokenizes forward with the
//! width-complete readers (so every boundary is exact), remembers only the **last
//! value-producing token**, and classifies at the first decisive token. The
//! receiver of a `99` bind is by definition the value on top of the operand stack,
//! which is the last value-producing token — so the classification needs no
//! method/receiver split at all, and the ambiguity above never has to be resolved.
//! The stacked-method *count* is recovered separately, and only to separate the
//! chained case (§4), which needs a frame and several `bl`s however its innermost
//! receiver is spelled.
//!
//! ## The completeness bit
//!
//! `docs/IL_CALL_IN_EXPR.md` §13.3 is the lesson this rung is built around:
//! census yield tracks **whole-body completeness**, not production coverage. D1
//! moved 17,864 functions into class and `expr-call-in-expr` fell by *exactly*
//! 17,864 with no other bucket moving, because its grammar accepts an entire
//! segment or nothing. Earlier rungs cleared 547,082 first blockers for +17,286.
//! A histogram of first blockers therefore cannot rank these sub-buckets, and
//! reporting one would repeat the mistake that mis-ranked intrinsic 2117 by a
//! factor of 4,600.
//!
//! So every sub-bucket is reported twice: once bare, and once with a `-whole`
//! suffix for the functions whose **entire segment** would parse if that one form
//! were admitted ([`whole_body_is_one_value`]). The two keys are disjoint and sum
//! to the sub-bucket, so `-whole` / total is the fraction that is worth anything.

use super::expr::{
    eat_fn_tail, eat_return_plumbing, eat_scopes, intrinsic_name, intrinsic_selector,
    off_add_admitted,
};
use super::{Block, Complete, BODY_SCOPE_DEPTH};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_int_like_or_ptr4, eat_operand_type, eat_opt_stmt_marker,
    eat_reinterpret_type, eat_value_type, read_token_var, read_type, read_varint, value_class,
};
use crate::func::readers::ValueClass;

/// The `ctx` every block from this module carries. [`Block::feature`] keys on it
/// and formats the sub-bucket name out of [`Block::aux`]; nothing else uses it.
pub(crate) const CALL_IN_EXPR: &str = "expr-call-in-expr";

/// What a `0x26` met inside `parse_expr` turned out to open.
///
/// One variant per **construct**, and a construct is named only where a capture
/// established it — the witness for each is in `docs/IL_CALL_IN_EXPR.md` §14.1.
/// Anything the walk cannot tokenize is [`CallForm::Op`], an honest hex bucket,
/// for the reason [`Block::feature`]'s own comment gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CallForm {
    /// Member call, receiver is a `B9` LOAD of a pointer formal or local.
    RecvLoad,
    /// Member call, receiver was read from memory (`… 30 <T>`).
    RecvDeref,
    /// Member call, receiver is a sub-object *address* at a **nonzero** byte
    /// offset (`… 33 <k≠0> 27 <T>` / `28`, no load) — the address costs an
    /// `addi r3,r3,k` before the branch.
    RecvField,
    /// The same, at **offset 0**: the address arithmetic emits nothing, so the
    /// receiver is already in the argument register. Kept apart from
    /// [`CallForm::RecvField`] because that is exactly the distinction D1 turned
    /// on — `docs/IL_CALL_IN_EXPR.md` §5 required the `this`-adjust offset to be 0
    /// because a nonzero one "costs a real `addi r3,r3,k` before the branch" — and
    /// a bucket that merges them cannot say how much of it is decode-only.
    RecvFieldZero,
    /// Member call, receiver is a named data symbol (`26 <sym>`, ± a `2C` decay).
    RecvObject,
    /// Member call, receiver is the result of an ordinary call (`… BD … 4C`).
    RecvCall,
    /// Member call, receiver is the result of a class-layout intrinsic
    /// (`… 40 … 4C`), carrying its selector.
    RecvIntrinsic(i32),
    /// Member call whose receiver the walk reached but could not name.
    RecvOther,
    /// **Two or more** stacked method symbols: a member-call chain
    /// (`p->Next()->Val()`, `G().Val()->M()`).
    Chained,
    /// A plain function call used as a value (`26 <fn> BD …`) — the production the
    /// bucket's *name* describes, and §7.3 measured at 0.2 % of it.
    NestedCall,
    /// A data symbol's **address** pushed as a value (string literal, array decay,
    /// `&global`, `&gA[k]`).
    DataAddr,
    /// A data symbol **read** (`… 30 <T>`): a global or static object's member.
    DataRead,
    /// An intrinsic result consumed as a value with no member bind.
    Intrinsic(i32),
    /// A decisive token was reached but the value feeding it was not named.
    Other,
    /// The walk met a byte it cannot tokenize. `docs/IL_CALL_IN_EXPR.md` §14.2
    /// ranks these; a name would be a guess.
    Op(u8),
    /// The walk ran off the end of the segment.
    Eof,
}

// --- `Block::aux` packing ---------------------------------------------------
//
// `Block` carries one `u64` of context and `ctx` is a `&'static str`, so the
// selector id and the residue opcode cannot go in the name. They go here:
//   bits  0..5    the receiver form's discriminant
//   bits  6..22   its payload (an intrinsic selector, or a residue opcode byte)
//   bit   23      the whole-body-completeness bit  (the form ALONE finishes the body)
//   bits 24..29   the SECOND blocker's discriminant                       (D4, §16)
//   bits 30..53   its payload (a nested form's own (disc, payload), a type
//                 class, an opcode byte, a structural sub-kind)
//   bits 54..55   the pair state: UNMEASURED / both⇒whole / both⇒more
// Nothing per-TU is representable in that layout, which is the sharding gate
// stated as an invariant rather than as a promise. The low 24 bits are
// bit-for-bit what D2 packed, so every §14/§15 key still renders identically.

const FORM_BITS: u32 = 6;
const FORM_MASK: u64 = (1 << FORM_BITS) - 1;
const PAYLOAD_BITS: u32 = 17;
const PAYLOAD_MASK: u64 = (1 << PAYLOAD_BITS) - 1;
const WHOLE_BIT: u64 = 1 << (FORM_BITS + PAYLOAD_BITS);

/// Where the second blocker's discriminant starts.
const BLK_SHIFT: u32 = 24;
const BLK_BITS: u32 = 6;
const BLK_MASK: u64 = (1 << BLK_BITS) - 1;
/// Where its payload starts, wide enough for a nested `(disc, payload)` pair.
const BLK_PAYLOAD_SHIFT: u32 = BLK_SHIFT + BLK_BITS;
const BLK_PAYLOAD_BITS: u32 = FORM_BITS + PAYLOAD_BITS;
const BLK_PAYLOAD_MASK: u64 = (1 << BLK_PAYLOAD_BITS) - 1;
/// Where the 3-bit **grant count** starts: how many extra constructs it took to
/// finish the segment, or one of the two sentinels below.
const NEED_SHIFT: u32 = BLK_PAYLOAD_SHIFT + BLK_PAYLOAD_BITS;
const NEED_MASK: u64 = 0x7;
/// The chain's completeness is **UNMEASURED**: no production exists for the second
/// blocker, so "would granting it finish the body" has no answer. A key with no
/// `-whole…`/`-more` suffix says exactly that, and `blocker_is_measured` is what
/// gates it — the same discipline `form_is_measured` applies one level up.
const NEED_UNMEASURED: u64 = 0;
/// MEASURED: granting up to [`MAX_ADMIT`] constructs was still not enough, or the
/// chain ran into something unmodelable partway.
const NEED_MORE: u64 = 7;
/// Where the **third** construct's coarse kind starts (5 bits), present only when
/// the greedy chain needed two or more grants.
///
/// A coarse kind rather than a whole [`Blocker`] because that is what fits, and it
/// is what the ranking needs: the k = 2 class is 25,588 functions — the largest
/// need class after `-more` — and 20,579 of them are one row (`data-addr` ×
/// `plain-call`). Whether their third construct is a pointer operand or a branch
/// decides whether that row is the best rung available or unreachable, and a single
/// hand-read witness could not settle it. The type classes are spelled out
/// individually for exactly that reason; the inner detail of a nested *call* is
/// dropped, which is stated in [`Blocker::kind_name`].
const KIND_SHIFT: u32 = NEED_SHIFT + 3;
const KIND_BITS: u32 = 5;
const KIND_MASK: u64 = (1 << KIND_BITS) - 1;
/// Where the **data-symbol count class** starts (2 bits), set only on a body the
/// matcher actually finished (`-whole…`) and only for the two data designators.
///
/// D5's measurement, and the reason it is in the key rather than in a document:
/// §16.3 ranked `data-addr × plain-call × type-ptr` first at 21,642 bodies and
/// described it as *"a global's or string literal's address passed to an ordinary
/// call"* — singular. It is not. Every one of the 2,730 symbol-carrying plain calls
/// in a 40-TU sample passes **two** string addresses, and c2 lowers the second as
/// `addi rD, rAnchor, <difference of their .rdata pool offsets>` rather than through
/// its own relocation pair. That makes instruction selection depend on a whole-TU
/// pool layout, which is a different and much larger piece of work than the one the
/// row was ranked for — so the count has to be *visible in the census*, not inferred
/// from a sample, or the row gets re-ranked wrong again next session
/// (`docs/IL_CALL_IN_EXPR.md` §17).
const SYMS_SHIFT: u32 = KIND_SHIFT + KIND_BITS;
const SYMS_MASK: u64 = 0x3;
/// Not applicable / not measured: the matcher never finished this body, or its form
/// is not a data designator so "how many data symbols" is not the operative number.
const SYMS_UNSET: u64 = 0;

impl CallForm {
    /// `(discriminant, payload)`.
    fn code(self) -> (u64, u64) {
        match self {
            CallForm::RecvLoad => (1, 0),
            CallForm::RecvDeref => (2, 0),
            CallForm::RecvField => (3, 0),
            CallForm::RecvFieldZero => (17, 0),
            CallForm::RecvObject => (4, 0),
            CallForm::RecvCall => (5, 0),
            CallForm::RecvIntrinsic(sel) => (6, sel as u64 & PAYLOAD_MASK),
            CallForm::RecvOther => (7, 0),
            CallForm::Chained => (8, 0),
            CallForm::NestedCall => (9, 0),
            CallForm::DataAddr => (10, 0),
            CallForm::DataRead => (11, 0),
            CallForm::Intrinsic(sel) => (13, sel as u64 & PAYLOAD_MASK),
            CallForm::Other => (14, 0),
            CallForm::Op(b) => (15, b as u64),
            CallForm::Eof => (16, 0),
        }
    }

    fn from_code(disc: u64, payload: u64) -> Option<CallForm> {
        Some(match disc {
            1 => CallForm::RecvLoad,
            2 => CallForm::RecvDeref,
            3 => CallForm::RecvField,
            17 => CallForm::RecvFieldZero,
            4 => CallForm::RecvObject,
            5 => CallForm::RecvCall,
            6 => CallForm::RecvIntrinsic(payload as i32),
            7 => CallForm::RecvOther,
            8 => CallForm::Chained,
            9 => CallForm::NestedCall,
            10 => CallForm::DataAddr,
            11 => CallForm::DataRead,
            13 => CallForm::Intrinsic(payload as i32),
            14 => CallForm::Other,
            15 => CallForm::Op(payload as u8),
            16 => CallForm::Eof,
            _ => return None,
        })
    }

    /// The census sub-bucket name, without the `expr-call-in-expr-` prefix.
    fn name(self) -> String {
        match self {
            CallForm::RecvLoad => "recv-load".into(),
            CallForm::RecvDeref => "recv-deref".into(),
            CallForm::RecvField => "recv-field".into(),
            CallForm::RecvFieldZero => "recv-field-off0".into(),
            CallForm::RecvObject => "recv-object".into(),
            CallForm::RecvCall => "recv-call".into(),
            CallForm::RecvIntrinsic(sel) => format!("recv-intrinsic-{}", intrinsic_name(sel)),
            CallForm::RecvOther => "recv-other".into(),
            CallForm::Chained => "chained".into(),
            CallForm::NestedCall => "nested-call".into(),
            CallForm::DataAddr => "data-addr".into(),
            CallForm::DataRead => "data-read".into(),
            CallForm::Intrinsic(sel) => format!("intrinsic-{}", intrinsic_name(sel)),
            CallForm::Other => "other".into(),
            CallForm::Op(b) => format!("op-0x{b:02X}"),
            CallForm::Eof => "eof".into(),
        }
    }
}

/// **D4 — what blocks the body once the receiver form is granted.**
///
/// §14.6 stated the limit D2 left open: *"`recv-object`'s 0.0 % says only that the
/// grammar doesn't finish those bodies; WHAT ELSE blocks them is uncharacterized"*
/// — and the three forms at 0.0 % hold 172,615 functions, 64 % of the bucket. This
/// enum is the answer's vocabulary: one variant per **construct**, so a
/// second-blocker histogram cannot repeat the by-position mis-attribution
/// `GAPS.md` §6 records (`docs/IL_CALL_IN_EXPR.md` §16.2 has the witness table).
///
/// The pair `(receiver form, Blocker)` is what a rung actually has to implement,
/// which is why both go in one census key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Blocker {
    /// Nothing: the form alone finishes the body (the `-whole` case).
    None,
    /// **Another `26`-opened production**, named by its own [`CallForm`]. This is
    /// the variant that makes the histogram actionable: `recv-object` bodies that
    /// block on `call-recv-load` say the rung is "both receiver forms at once",
    /// and a pair count is what such a rung would be estimated from.
    Call(CallForm),
    /// A `30 <TYPE>` **indirect load** used as a value — reading a member or
    /// through a pointer, which the modeled operand vocabulary has no token for.
    DerefLoad,
    /// A `9B <TYPE> <tok>` **by-value temporary bind** — the same construct the
    /// `op-0x9B` sub-bucket is, met here *after* a receiver instead of before one.
    TempBind,
    /// A **conditional branch** `<opcode> <label-tok>` — control flow, and the
    /// construct that ends the straight-line assumption every modeled shape rests
    /// on. MEASURED as a branch and not guessed: in `src/system/hamobj/Ham.cpp` the
    /// token a `39` carries is *defined* later in the same segment by a `29 <same
    /// token>` label, twice over (`39 30 67` … `29 30 67`, `39 38 67` … `29 38 67`),
    /// and the second witness `b9 <x> 86 42 75 33 86 41 74 01 0b 39 2b 67` is
    /// `if (x & 1)` — a bit-and feeding it.
    ///
    /// The **polarity** of `38` versus `39` was recorded here as UNDETERMINED, on
    /// the strength of those two wild witnesses, which cannot separate the senses:
    /// a branch to a later label is consistent with either reading. It is
    /// determined now — `fixtures/cpp/wcf_shapes.cpp` holds a controlled pair
    /// differing only by a `!`, and the name comes from the ONE table
    /// [`super::cflow_opcode_name`], shared with the `expr-*` keys so the two
    /// producers cannot disagree about what a byte is called.
    ///
    /// Deliberately has **no production**: admitting a branch as a value token would
    /// report grammar-completeness for bodies that need basic blocks, a register
    /// allocator across them and a `/Gy` layout — every one of which is a phase, not
    /// a rung. The pair is reported UNMEASURED, which is the honest answer.
    Branch(u8),
    /// A **chain link** `99 <T> 00 <call>` applied to the value already on the
    /// stack: the outer bind of `p->Get()->Foo()`. Met in a value position because
    /// the *inner* member call was consumed by the form's own production and the
    /// second bind has nothing to attach to in D2's grammar.
    ChainBind,
    /// A **byte-offset add** `27 <T>` / `28 00 00` in a value position: a sub-object
    /// address computed outside a receiver designator (`docs/IL_EXPR_LAYER.md` §4).
    OffAdd,
    /// A bare **`BD` CALL token** in a value position: an ordinary call whose callee
    /// push the form's own production already consumed. `uc("hi")` is the canonical
    /// case — `26 <uc>` is a legal `data-addr` designator, so the greedy value
    /// sequence takes it and the construct that is actually missing is *the call*.
    /// Naming the byte `op-0xBD` instead would file 56,633 string-literal argument
    /// pushes under an opcode, which is precisely the mis-attribution `GAPS.md` §6
    /// records.
    PlainCall,
    /// A `67` **virtual dispatch**.
    Virtual,
    /// A modeled token whose **TYPE** is outside the int4/pointer class the leaves
    /// lower. The payload is the type's class, never its per-TU id.
    Type(TypeClass),
    /// The body's **return plumbing / function tail** did not match, with the byte
    /// the tail opens on. Structural, not a value construct.
    Plumbing(u8),
    /// A structural refusal that is not a token at all — see [`Structural`].
    Structure(Structural),
    /// A byte with no production. `docs/IL_CALL_IN_EXPR.md` §16 ranks these; a name
    /// would be a guess, and a flat tail of these would mean a payload is being
    /// read as vocabulary (§14.2's fingerprint), not that the vocabulary is large.
    Op(u8),
    /// The matcher ran off the end of the segment.
    Eof,
}

/// The class nibble of a TYPE the modeled leaves do not lower
/// (`docs/IL_LOAD_TYPES.md` §1: 1 signed int · 2 unsigned · 3 data pointer ·
/// 4 function pointer · 5 real · 6 aggregate · 7 void · A real literal). Named by
/// **class and slot width**, both of which are fixed vocabulary — nothing per-TU
/// enters the key, so `expr-load-type-864383`-style sharding cannot happen here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TypeClass {
    /// An integer of a width other than 4 (`char`, `short`, `long long`), with
    /// that width. `int`/`unsigned` at width 4 are already modeled.
    IntWidth(u8),
    /// Class 3 — a **data pointer**. Reached as a blocker only in an *argument*
    /// region, where D2's operand grammar takes `int` and nothing else: passing a
    /// pointer to a function is the construct, and it is the single most common one
    /// in this whole decomposition.
    Ptr,
    /// Class 4 — a function/code pointer.
    CodePtr,
    /// Class 5 — `float` / `double`.
    Real,
    /// Class 6 — a struct/class value.
    Aggregate,
    /// Class 7 — `void`.
    Void,
    /// Class A — a real *literal* (its payload is 8 IEEE bytes + a size, not a
    /// varint; `docs/IL_CAST_CONVERT.md` §3.1).
    RealLit,
    /// Any other class nibble, reported as itself.
    Class(u8),
    /// Not a TYPE at all at that position — the tag's bit 7 was clear, or the
    /// aggregate size ladder refused. A *desync* signal, so it is named as one
    /// rather than folded into a class.
    NotAType,
}

impl TypeClass {
    fn at(seg: &[u8], p: usize) -> TypeClass {
        let Some((tag, kind, _, _)) = read_type(seg, p) else {
            return TypeClass::NotAType;
        };
        let width = match tag & 0x0E {
            0x2 => 1,
            0x4 => 2,
            0x6 => 4,
            0x8 => 8,
            _ => 0,
        };
        match kind & 0x0F {
            0x1 | 0x2 => TypeClass::IntWidth(width),
            0x3 => TypeClass::Ptr,
            0x4 => TypeClass::CodePtr,
            0x5 => TypeClass::Real,
            0x6 => TypeClass::Aggregate,
            0x7 => TypeClass::Void,
            0xA => TypeClass::RealLit,
            c => TypeClass::Class(c),
        }
    }

    fn code(self) -> u64 {
        match self {
            TypeClass::IntWidth(w) => 0x10 | w as u64,
            TypeClass::Ptr => 0x24,
            TypeClass::CodePtr => 0x25,
            TypeClass::Real => 0x20,
            TypeClass::Aggregate => 0x21,
            TypeClass::Void => 0x22,
            TypeClass::RealLit => 0x23,
            TypeClass::Class(c) => 0x30 | c as u64,
            TypeClass::NotAType => 0x40,
        }
    }

    fn from_code(c: u64) -> Option<TypeClass> {
        Some(match c {
            0x10..=0x1F => TypeClass::IntWidth((c & 0xF) as u8),
            0x20 => TypeClass::Real,
            0x21 => TypeClass::Aggregate,
            0x22 => TypeClass::Void,
            0x23 => TypeClass::RealLit,
            0x24 => TypeClass::Ptr,
            0x25 => TypeClass::CodePtr,
            0x30..=0x3F => TypeClass::Class((c & 0xF) as u8),
            0x40 => TypeClass::NotAType,
            _ => return None,
        })
    }

    fn name(self) -> String {
        match self {
            TypeClass::IntWidth(w) => format!("int{w}"),
            TypeClass::Ptr => "ptr".into(),
            TypeClass::CodePtr => "code-ptr".into(),
            TypeClass::Real => "real".into(),
            TypeClass::Aggregate => "aggregate".into(),
            TypeClass::Void => "void".into(),
            TypeClass::RealLit => "real-lit".into(),
            TypeClass::Class(c) => format!("class-0x{c:X}"),
            TypeClass::NotAType => "not-a-type".into(),
        }
    }

    /// Whether a TYPE at `p` is of this class — the "admitted" test for the
    /// both-handled measure, which widens [`eat_scalar_type`] by exactly one class.
    fn matches(self, seg: &[u8], p: usize) -> bool {
        TypeClass::at(seg, p) == self
    }
}

/// A refusal with no token at it: the body's frame, not its contents.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Structural {
    /// The `53` body marker was not where the segment said it would be.
    BodyMarker,
    /// A scope open/close run (`54 <d>`) the matcher could not walk.
    Scopes,
    /// More than [`MAX_STMTS`] statements. A body this long is not a rung.
    StmtLimit,
    /// **Arithmetic on a pointer value inside a call argument.** `parse_expr`
    /// refuses it (`p + 1` on an `int*` is `addi r3,r3,4`, so a modeled chain
    /// that added 1 would be wrong bytes rather than a gap) and #139 gave the
    /// measure the same rule.
    ///
    /// It is a NAMED construct rather than the byte the run stopped in front
    /// of, and that is not cosmetic: the first cut of #139's repair filed this
    /// refusal as `…-then-op-0x55`, i.e. the `55` call-end marker, which names
    /// the position instead of the construct — the precise disease #139 exists
    /// to cure, reintroduced by its own fix. `fixtures/cpp/wrr_arg_vocab_neg.cpp`
    /// caught it.
    PtrArith,
    /// The one-byte-unsigned twin of [`Structural::PtrArith`]: the class is free
    /// to be *moved*, and neither computed on nor mixed with a width-4 value.
    Int1uMisuse,
}

impl Structural {
    fn code(self) -> u64 {
        match self {
            Structural::BodyMarker => 1,
            Structural::Scopes => 2,
            Structural::StmtLimit => 3,
            Structural::PtrArith => 4,
            Structural::Int1uMisuse => 5,
        }
    }
    fn from_code(c: u64) -> Option<Structural> {
        Some(match c {
            1 => Structural::BodyMarker,
            2 => Structural::Scopes,
            3 => Structural::StmtLimit,
            4 => Structural::PtrArith,
            5 => Structural::Int1uMisuse,
            _ => return None,
        })
    }
    fn name(self) -> &'static str {
        match self {
            Structural::BodyMarker => "struct-body-marker",
            Structural::Scopes => "struct-scopes",
            Structural::StmtLimit => "struct-stmt-limit",
            Structural::PtrArith => "ptr-arith",
            Structural::Int1uMisuse => "int1u-misuse",
        }
    }
}

impl Blocker {
    /// `(discriminant, payload)`. `Call`'s payload is the nested form's own
    /// `(disc, payload)` pair, which is why the field is 23 bits wide: an intrinsic
    /// selector must survive nesting or two buckets would merge.
    fn code(self) -> (u64, u64) {
        match self {
            Blocker::None => (0, 0),
            Blocker::Call(f) => {
                let (d, p) = f.code();
                (1, d | (p << FORM_BITS))
            }
            Blocker::DerefLoad => (2, 0),
            Blocker::TempBind => (3, 0),
            Blocker::PlainCall => (10, 0),
            Blocker::Branch(b) => (11, b as u64),
            Blocker::ChainBind => (12, 0),
            Blocker::OffAdd => (13, 0),
            Blocker::Virtual => (4, 0),
            Blocker::Type(c) => (5, c.code()),
            Blocker::Plumbing(b) => (6, b as u64),
            Blocker::Structure(s) => (7, s.code()),
            Blocker::Op(b) => (8, b as u64),
            Blocker::Eof => (9, 0),
        }
    }

    fn from_code(disc: u64, payload: u64) -> Option<Blocker> {
        Some(match disc {
            0 => Blocker::None,
            1 => Blocker::Call(CallForm::from_code(
                payload & FORM_MASK,
                (payload >> FORM_BITS) & PAYLOAD_MASK,
            )?),
            2 => Blocker::DerefLoad,
            3 => Blocker::TempBind,
            10 => Blocker::PlainCall,
            11 => Blocker::Branch(payload as u8),
            12 => Blocker::ChainBind,
            13 => Blocker::OffAdd,
            4 => Blocker::Virtual,
            5 => Blocker::Type(TypeClass::from_code(payload)?),
            6 => Blocker::Plumbing(payload as u8),
            7 => Blocker::Structure(Structural::from_code(payload)?),
            8 => Blocker::Op(payload as u8),
            9 => Blocker::Eof,
            _ => return None,
        })
    }

    /// A **coarse kind**, 5 bits, for the third construct of a greedy chain. Closed
    /// and small by design: nothing per-TU, nothing payload-shaped, and the type
    /// classes that matter to a ranking are individually named.
    fn kind_code(self) -> u64 {
        match self {
            Blocker::None => 0,
            Blocker::Call(_) => 1,
            Blocker::DerefLoad => 2,
            Blocker::TempBind => 3,
            Blocker::Virtual => 4,
            Blocker::Type(TypeClass::Ptr) => 5,
            Blocker::Type(TypeClass::CodePtr) => 6,
            Blocker::Type(TypeClass::IntWidth(1)) => 7,
            Blocker::Type(TypeClass::IntWidth(2)) => 8,
            Blocker::Type(TypeClass::IntWidth(8)) => 9,
            Blocker::Type(TypeClass::Real) => 10,
            Blocker::Type(TypeClass::RealLit) => 11,
            Blocker::Type(TypeClass::Aggregate) => 12,
            Blocker::Type(_) => 13,
            Blocker::Plumbing(_) => 14,
            Blocker::Structure(_) => 15,
            Blocker::Op(_) => 16,
            Blocker::Eof => 17,
            Blocker::PlainCall => 18,
            Blocker::Branch(_) => 19,
            Blocker::ChainBind => 20,
            Blocker::OffAdd => 21,
        }
    }

    /// The name of a coarse kind. `call`, `type-other`, `op` and `branch` are
    /// deliberately detail-free — the byte, the selector and the inner receiver form
    /// do not fit in five bits, and a name that pretended otherwise would merge
    /// buckets rather than lose a suffix.
    fn kind_name(code: u64) -> &'static str {
        match code {
            1 => "call",
            2 => "deref-load",
            3 => "temp-bind",
            4 => "virtual",
            5 => "type-ptr",
            6 => "type-code-ptr",
            7 => "type-int1",
            8 => "type-int2",
            9 => "type-int8",
            10 => "type-real",
            11 => "type-real-lit",
            12 => "type-aggregate",
            13 => "type-other",
            14 => "plumbing",
            15 => "struct",
            16 => "op",
            17 => "eof",
            18 => "plain-call",
            19 => "branch",
            20 => "chain-bind",
            21 => "off-add",
            _ => "none",
        }
    }

    /// The `-then-…` half of the census key.
    fn name(self) -> String {
        match self {
            Blocker::None => String::new(),
            Blocker::Call(f) => format!("call-{}", f.name()),
            Blocker::DerefLoad => "deref-load".into(),
            Blocker::TempBind => "temp-bind".into(),
            Blocker::PlainCall => "plain-call".into(),
            Blocker::Branch(b) => match super::cflow_opcode_name(b) {
                Some(n) => format!("branch-{n}"),
                None => format!("branch-0x{b:02X}"),
            },
            Blocker::ChainBind => "chain-bind".into(),
            Blocker::OffAdd => "off-add".into(),
            Blocker::Virtual => "virtual".into(),
            Blocker::Type(c) => format!("type-{}", c.name()),
            Blocker::Plumbing(b) => format!("plumbing-0x{b:02X}"),
            Blocker::Structure(s) => s.name().into(),
            // The capture-verified names, shared with the `expr-*` keys
            // ([`super::expr_opcode_name`]) so the two families cannot disagree
            // about what a byte is called. Anything unnamed stays hex: a hex bucket
            // is a result, a guessed name is not.
            Blocker::Op(b) => match super::expr_opcode_name(b) {
                Some(n) => n.into(),
                None => format!("op-0x{b:02X}"),
            },
            Blocker::Eof => "eof".into(),
        }
    }
}

/// The census key for a [`Block`] this module raised.
///
/// Four disjoint shapes, and the suffix is load-bearing:
///
/// | key | meaning |
/// |---|---|
/// | `…-<form>-whole` | the receiver form **alone** finishes the segment (D2) |
/// | `…-<form>-then-<blk>-whole` | MEASURED: form **and** `blk` together finish it |
/// | `…-<form>-then-<blk>-more` | MEASURED: both together are still not enough |
/// | `…-<form>-then-<blk>` | **UNMEASURED**: no production exists for `blk` |
/// | `…-<form>` | UNMEASURED: no production exists for `form` either (D2 residue) |
///
/// The three `-then-` families and `-whole` partition each D2 sub-bucket exactly,
/// so §14.1's counts are recoverable by summing — which is the acceptance check
/// this rung is graded on.
/// The [`Complete`] reading this module's `aux` encodes — the **same** bits
/// [`feature`] renders as `-whole` / `-whole{k}` / `-more` / no suffix, read
/// once here so a consumer never has to recover them from the string.
///
/// Held to `feature` by `tests::the_completeness_axis_agrees_with_the_rendered_key`
/// over the whole enumerated key space, so the two cannot drift.
pub(crate) fn completeness(aux: u64) -> Complete {
    let disc = aux & FORM_MASK;
    let payload = (aux >> FORM_BITS) & PAYLOAD_MASK;
    if CallForm::from_code(disc, payload).is_none() {
        return Complete::NoSignal;
    }
    if aux & WHOLE_BIT != 0 {
        return Complete::WholeGrammar;
    }
    let blk = Blocker::from_code(
        (aux >> BLK_SHIFT) & BLK_MASK,
        (aux >> BLK_PAYLOAD_SHIFT) & BLK_PAYLOAD_MASK,
    );
    match blk {
        // No second blocker recorded at all: the form itself has no production
        // ([`form_is_measured`]), so nothing about completeness was measured.
        None | Some(Blocker::None) => Complete::UnmeasuredGrammar,
        Some(_) => match (aux >> NEED_SHIFT) & NEED_MASK {
            NEED_UNMEASURED => Complete::UnmeasuredGrammar,
            NEED_MORE => Complete::MoreGrammar,
            _ => Complete::WholeGrammar,
        },
    }
}

pub(crate) fn feature(aux: u64) -> String {
    let disc = aux & FORM_MASK;
    let payload = (aux >> FORM_BITS) & PAYLOAD_MASK;
    let form = CallForm::from_code(disc, payload);
    let name = match form {
        Some(f) => f.name(),
        // Unreachable by construction; a bucket rather than a panic, since this
        // is a diagnostic path and a census must never take the process down.
        None => format!("aux-{aux:X}"),
    };
    // How many data symbols the finished body materializes — the number that
    // decides whether the row needs one relocation pair or a pool-relative
    // selection (see [`SYMS_SHIFT`]).
    //
    // **Rendered next to the construct that owns the operands**, which is the form
    // when the form is a data designator and the *second blocker* when it is not
    // (WDA). D5 rendered it only in the first position, on the stated ground that
    // the count "is a property of the form's own operands and not of the second
    // blocker" — true of where the count is *read*, false of where it is *produced*,
    // because a granted `Blocker::Call(DataAddr)` runs the same designator
    // production. Putting the suffix on the blocker keeps the key readable in both
    // cases: `data-addr-2sym-then-plain-call…` and
    // `recv-load-then-call-data-addr-2sym…` each name the construct that has to
    // materialize two addresses.
    let sym_suffix = match (aux >> SYMS_SHIFT) & SYMS_MASK {
        SYMS_UNSET => String::new(),
        3 => "-3sym+".to_string(),
        k => format!("-{k}sym"),
    };
    let form_owns_syms =
        matches!(form, Some(CallForm::DataAddr) | Some(CallForm::DataRead));
    let name = if form_owns_syms { format!("{name}{sym_suffix}") } else { name };
    if aux & WHOLE_BIT != 0 {
        return format!("{CALL_IN_EXPR}-{name}-whole");
    }
    let blk = Blocker::from_code(
        (aux >> BLK_SHIFT) & BLK_MASK,
        (aux >> BLK_PAYLOAD_SHIFT) & BLK_PAYLOAD_MASK,
    );
    match blk {
        None | Some(Blocker::None) => format!("{CALL_IN_EXPR}-{name}"),
        Some(b) => {
            let need = (aux >> NEED_SHIFT) & NEED_MASK;
            let suffix = match need {
                NEED_UNMEASURED => String::new(),
                NEED_MORE => "-more".into(),
                1 => "-whole".into(),
                k => format!("-whole{k}"),
            };
            // The third construct, named only when there was one.
            let third = match (aux >> KIND_SHIFT) & KIND_MASK {
                0 => String::new(),
                k => format!("-and-{}", Blocker::kind_name(k)),
            };
            // When the form did not own the count, the second blocker did — and if
            // neither does, `sym_suffix` is empty, because `mark_whole` only sets
            // the bits for those two cases.
            let blk = if form_owns_syms { b.name() } else { format!("{}{sym_suffix}", b.name()) };
            format!("{CALL_IN_EXPR}-{name}-then-{blk}{third}{suffix}")
        }
    }
}

/// **The D2 entry point.** Classify the `0x26` at `at` and return the refusal.
///
/// Always an `Err`-shaped [`Block`] — this decodes, it does not accept.
pub(crate) fn classify(seg: &[u8], at: usize) -> Block {
    let form = walk(seg, at);
    let (disc, payload) = form.code();
    Block {
        ctx: CALL_IN_EXPR,
        byte: Some(0x26),
        off: at,
        seg_len: seg.len(),
        aux: disc | (payload << FORM_BITS),
    }
}

/// **D6 — the statement-head re-anchor** (`docs/IL_CALL_IN_EXPR.md` §18.3).
///
/// §16.4 measured that the `chained` sub-bucket undercounts chains ~4.4×, and named
/// the cause: `mod.rs`'s body dispatch cannot tell a statement-head `26 <tok>`
/// assignment *destination* from a stacked *method* push — the two differ by one
/// byte far away — so it hands the assignment parser a body that has no
/// destination, that parser eats the outer method push, and `parse_expr` starts one
/// `26` late. `p->Next()->Get()` in a **value** position then has exactly one method
/// stacked where the walk can see it, and files as `recv-load`. The identical body
/// in an assignment (`x = p->Next()->Get()`) keeps both and files as `chained`.
///
/// This restores the anchor for exactly that case, and for nothing else. Called
/// only on the error path of the assignment parser's right-hand side, with
/// `stmt_head` = the statement's own first byte and `probe` = where the destination
/// push was consumed to; acceptance is untouched (the `Err` stays an `Err`).
///
/// **Three conditions, all required, because the head token is genuinely
/// ambiguous.**
///
/// 1. The refusal is this module's, and it is *at* `probe` — i.e. the very first
///    thing after the consumed token was the `26` that opened a member call. A
///    refusal deeper in the statement is anchored where it belongs.
/// 2. Walking from `stmt_head` classifies as [`CallForm::Chained`], and walking
///    from `probe` did not. Nothing else can move.
/// 3. **The bind count corroborates it.** A member-call production stacks one
///    method per `99` bind, so a head run of *m* methods is only real if the
///    statement contains *m* depth-0 binds. This is the condition that keeps
///    `x = p->Get()` — head run `26 <x> 26 <Get>`, two symbols, **one** bind — from
///    being promoted to a chain. Without it the fix would trade a 4.4× undercount
///    for an overcount of every single-link assignment in the corpus.
///
/// Condition 3 is measured by iterating [`walk_detail`] itself rather than by a
/// second tokenizer, so the count and the classification cannot drift apart.
pub(crate) fn reanchor_chain(seg: &[u8], stmt_head: usize, probe: usize, b: Block) -> Block {
    if b.ctx != CALL_IN_EXPR {
        return reanchor_stmt_member_call(seg, stmt_head, probe, b);
    }
    if b.off != probe {
        return b;
    }
    // (2) the probe-anchored form is whatever `parse_expr` recorded.
    let probe_form = CallForm::from_code(b.aux & FORM_MASK, (b.aux >> FORM_BITS) & PAYLOAD_MASK);
    if probe_form == Some(CallForm::Chained) {
        return b;
    }
    let (head_form, methods, _) = walk_detail(seg, stmt_head);
    if head_form != CallForm::Chained || methods < 2 {
        return b;
    }
    // (3) one bind per stacked method, or the head run was not all methods.
    if depth0_binds(seg, stmt_head, methods) < methods {
        return b;
    }
    let (disc, payload) = CallForm::Chained.code();
    Block { off: stmt_head, aux: disc | (payload << FORM_BITS), ..b }
}

/// **W36 — the statement-position member call**, and the other half of the same
/// mis-anchoring [`reanchor_chain`] repairs.
///
/// `x = p->M();` keeps its method push where `parse_expr` can see it, so it reaches
/// [`classify`] and files under this module. **`p->M();` does not.** The body
/// dispatch consumes its `26 <method>` as an assignment *destination* (the byte
/// after the token is the receiver's `B9`, not a `BD`), the assignment parser hands
/// the rest to `parse_expr`, and `parse_expr` takes the receiver as an ordinary
/// LOAD and stops dead on the `99` bind under the generic `expr` fall-through. The
/// whole production is then filed as an **opcode**, `expr-op-0x99` — 280,283
/// functions, 11.4 % of everything blocked and the largest single key on the board.
///
/// So `expr-op-0x99` was never a missing token: it is this module's own
/// `recv-*` family under a second name, reached by the one route that does not call
/// [`classify`]. That is `GAPS.md` §6's unstable-*attribution* hazard in its purest
/// form so far — the same construct filed under a call bucket or an opcode bucket
/// depending only on whether the statement had a destination — and it is a
/// **coverage-costing** instance: the row carried no `-whole` bit at all, so no
/// ranking taken from it could see what was complete behind it.
///
/// Three conditions, and the second is what makes this a measurement rather than a
/// guess about an ambiguous head token:
///
/// 1. The refusal is not this module's (a refusal that *is* goes to
///    [`reanchor_chain`]'s original arm) and it lands strictly past the consumed
///    destination token, **on a `99`**.
/// 2. The forward walk from `stmt_head` — the same [`walk_detail`] every other
///    reading in this module uses, never a second tokenizer — stops at **exactly
///    that byte**. Nothing else can produce that coincidence: the walk stops on a
///    depth-0 `99` and `parse_expr` stopped on a depth-0 `99`, so both readings
///    agree the head run and the receiver are one member-call production.
/// 3. The bind count corroborates the method run, exactly as the original arm
///    requires it to — one depth-0 `99` per stacked method.
///
/// Acceptance is untouched: the `Err` stays an `Err` and only its census key moves.
fn reanchor_stmt_member_call(seg: &[u8], stmt_head: usize, probe: usize, b: Block) -> Block {
    // (1) past the destination token, and on the bind.
    if b.off <= probe || seg.get(b.off) != Some(&0x99) {
        return b;
    }
    let (form, methods, stop) = walk_detail(seg, stmt_head);
    // (2) the two readings stopped on the same byte.
    if stop != b.off {
        return b;
    }
    // A walk that stops on a depth-0 `99` classifies through `Stop::Bind`, so the
    // form is one of the receiver family. Requiring that positively rather than
    // assuming it keeps a `CallForm::Eof` (a walk that gave up at this offset for
    // an unrelated reason) from being promoted to a receiver form.
    if !matches!(
        form,
        CallForm::RecvLoad
            | CallForm::RecvDeref
            | CallForm::RecvField
            | CallForm::RecvFieldZero
            | CallForm::RecvObject
            | CallForm::RecvCall
            | CallForm::RecvIntrinsic(_)
            | CallForm::RecvOther
            | CallForm::Chained
    ) {
        return b;
    }
    // (3) one bind per stacked method — `>= 1`, because a single-method head still
    // has to have produced the bind the walk stopped on.
    let want = methods.max(1);
    if depth0_binds(seg, stmt_head, want) < want {
        return b;
    }
    let (disc, payload) = form.code();
    Block {
        ctx: CALL_IN_EXPR,
        byte: Some(0x26),
        off: stmt_head,
        seg_len: seg.len(),
        aux: disc | (payload << FORM_BITS),
    }
}

/// How many depth-0 `99 <TYPE> 00` member binds the statement at `start` contains,
/// counting no further than `want` (the caller only ever asks "at least *m*").
///
/// Implemented by re-entering [`walk_detail`] past each bind it stops on, so there
/// is exactly one tokenizer in this module. A walk that stops on anything other
/// than a `99` ends the count — including a byte it cannot tokenize, which is the
/// common case here and is why the count is a **lower** bound and the caller's test
/// is `>=`.
fn depth0_binds(seg: &[u8], start: usize, want: usize) -> usize {
    let mut n = 0usize;
    let mut p = start;
    // Bounded by `want` plus a hard stop: `walk_detail` is O(segment) and this
    // must not become O(segment^2) over a 2.4 M-function census.
    for _ in 0..MAX_ADMIT.max(want) {
        let (_, _, stop) = walk_detail(seg, p);
        if seg.get(stop) != Some(&0x99) {
            return n;
        }
        let Some((_, _, _, tw)) = read_type(seg, stop + 1) else {
            return n;
        };
        if seg.get(stop + 1 + tw) != Some(&0x00) {
            return n;
        }
        n += 1;
        if n >= want {
            return n;
        }
        p = stop + 1 + tw + 1;
    }
    n
}

/// Set the whole-body-completeness bit on a block this module raised — and, when
/// the form alone is *not* enough, name the construct that blocks the body **next**
/// and say whether the two together would be.
///
/// Called from [`super::parse_segment_detail`], which is the only place that has
/// both the block and the `LO` offset. Diagnostic only: the `Err` stays an `Err`.
pub(crate) fn mark_whole(seg: &[u8], lo: usize, b: Block) -> Block {
    let disc = b.aux & FORM_MASK;
    let payload = (b.aux >> FORM_BITS) & PAYLOAD_MASK;
    let Some(form) = CallForm::from_code(disc, payload) else {
        return b;
    };
    // Forms with no production are UNMEASURED at *both* levels: there is nothing
    // to walk past, so there is no second blocker either. Saying so by leaving the
    // key bare is what keeps "0 of N complete" from being read as a measurement.
    if !form_is_measured(form) {
        return b;
    }
    // The data-symbol count is only the operative number where a data designator is
    // what materializes the address; elsewhere the operands are not symbols and a
    // count would be noise in the key. On the **bare** pass that can only be the
    // form itself: `eat_data_designator` has exactly two call sites, both in
    // [`eat_form_value`], so `Admit::bare(form)` reaches it only for these two forms
    // and `fail.syms` is 0 for every other form by construction.
    let form_counts_syms = matches!(form, CallForm::DataAddr | CallForm::DataRead);
    let sym_bits = |counts: bool, f: &Fail| if counts { f.sym_class() << SYMS_SHIFT } else { 0 };
    let mut adm = Admit::bare(form);
    let mut fail = Fail::new();
    if body_matches(seg, lo, adm, &mut fail) {
        return Block { aux: b.aux | WHOLE_BIT | sym_bits(form_counts_syms, &fail), ..b };
    }
    // **The greedy chain.** Grant the construct that blocks the body, retry, and
    // repeat — up to [`MAX_ADMIT`]. The *first* construct granted is the "second
    // blocker" the census key names; the number of grants it took to finish is what
    // separates "these bodies share one further blocker" from "each carries three
    // unrelated ones", which is the question the whole rung exists to answer.
    let first = fail.blocker(seg);
    // **WDA — the count is the operative number wherever the designator is, not only
    // where the form is.** `eat_one_blocker_value` routes a granted
    // `Blocker::Call(DataAddr|DataRead)` straight back into [`eat_form_value`], so
    // `fail.syms` is accumulated for a body whose symbols arrive as the SECOND
    // blocker exactly as it is for one whose form owns them — and then D5's predicate
    // threw the number away, because it asked about the form alone. That silently
    // un-measured every `-then-call-data-{addr,read}-…-whole…` row: 10,555 workload
    // functions, of which 10,540 are one key (`recv-load-then-call-data-addr-whole`),
    // and the count is precisely what separates §17.6 (3)'s takeable rung from
    // §17.6 (6)'s phase. A refusal that emits nothing and agrees with census by
    // construction is invisible to every gate this project has; the fix is one
    // disjunct, and [`feature`] renders it next to the construct that owns it.
    let counts_syms = form_counts_syms
        || matches!(first, Blocker::Call(CallForm::DataAddr) | Blocker::Call(CallForm::DataRead));
    let mut need = NEED_UNMEASURED;
    // The coarse kind of the UNMEASURED construct the greedy chain stopped on, so
    // that a `-more` row says what came *next* rather than only that something did.
    // Rendered as the `-and-<kind>` half of the key, and only when at least one
    // construct was actually granted first — at `adm.n == 0` the chain stops on the
    // key's own second blocker and naming its kind would restate the key.
    let mut broke_on: u64 = 0;
    while adm.n < MAX_ADMIT {
        let blk = fail.blocker(seg);
        // An unmodelable construct ends the chain: everything past it is unknowable,
        // and saying so is the whole point of `blocker_is_measured`.
        if !blocker_is_measured(blk) {
            need = if adm.n == 0 { NEED_UNMEASURED } else { NEED_MORE };
            if adm.n >= 1 {
                broke_on = blk.kind_code();
            }
            break;
        }
        // A construct that repeats means its production did not consume the thing the
        // classifier named — a bug, not a body — so stop rather than spin.
        if adm.holds(blk) {
            need = NEED_MORE;
            break;
        }
        adm.push(blk);
        fail = Fail::new();
        if body_matches(seg, lo, adm, &mut fail) {
            need = adm.n as u64;
            break;
        }
        need = NEED_MORE;
    }
    let (bd, bp) = first.code();
    // The third construct's coarse kind, when the chain needed one. Recorded even
    // when the chain ended in `-more`, because "what came after the second blocker"
    // is the question that separates a reachable row from an unreachable one — and
    // that question has an answer whether the third construct was *granted* (the
    // chain continued) or merely *reached* (the chain stopped on it, `broke_on`).
    // Reporting only the granted case is what left the whole `bit-and` row mute.
    let third = if adm.n >= 2 { adm.also[1].kind_code() } else { broke_on };
    // Only a body the matcher actually *finished* has a well-defined symbol count:
    // a `-more` body stopped partway, so its designators are however many the
    // abandoned prefix happened to hold, which is a property of the refusal and not
    // of the program. Left unset there rather than reported.
    let syms = if (1..=MAX_ADMIT as u64).contains(&need) { sym_bits(counts_syms, &fail) } else { 0 };
    Block {
        aux: b.aux
            | syms
            | (bd << BLK_SHIFT)
            | ((bp & BLK_PAYLOAD_MASK) << BLK_PAYLOAD_SHIFT)
            | (need << NEED_SHIFT)
            | (third << KIND_SHIFT),
        ..b
    }
}

// --- the forward walk ------------------------------------------------------

/// The last value-producing token the walk consumed. The receiver of a `99` bind
/// is the operand-stack top, which is exactly this.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tk {
    /// `B9 <tok> <TYPE>`
    Load,
    /// `33 <TYPE> <varint>`
    Lit,
    /// `26 <tok>` that was *not* immediately followed by a `BD` — a symbol push.
    Sym,
    /// `27 <TYPE>` / `28 00 00` — a byte-offset add, yielding an address. The flag
    /// is whether the offset literal was **zero**, which decides whether the
    /// address costs an instruction.
    OffAdd(bool),
    /// `30 <TYPE>` — an indirect load.
    Deref,
    /// `4C` closing an ordinary `BD` call (`false`) or an intrinsic `40` call
    /// (`true`, with its selector).
    CallEnd(bool, i32),
    /// A binary operator.
    Op,
}

/// The token the walk stopped on, which decides what the value it classified was
/// *used for* — never what the value *is*. See the module note on
/// mis-attribution.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stop {
    /// `99` — the member bind. Direct dispatch by construction (§3).
    Bind,
    /// `32` — a store.
    Store,
    /// `55` at the top level — an argument push in the *enclosing* call.
    ArgEnd,
    /// `41` result type / `4B` statement end — the value reached the enclosing
    /// statement.
    StmtEnd,
    /// An untokenizable byte.
    Op(u8),
    Eof,
}

/// Bound on the walk. A blocked function is walked once per census, so this is
/// only a runaway guard; the longest real production in the sample is a
/// four-deep member-call chain at well under 100 tokens.
const MAX_TOKENS: usize = 4096;

/// Tokenize forward from the `26` at `start` and classify what it opened.
fn walk(seg: &[u8], start: usize) -> CallForm {
    walk_detail(seg, start).0
}

/// [`walk`], plus the two facts the **statement-head re-anchor** needs
/// (`docs/IL_CALL_IN_EXPR.md` §18.3): how many *methods* the head run stacked, and
/// where the walk stopped.
///
/// Split out rather than duplicated: a second tokenizer over the same grammar is
/// the defect `GAPS.md` §6 records as "one fact, one locator" — and here it would
/// be worse than usual, because the re-anchor's whole job is to disagree with this
/// walk about one leading token.
fn walk_detail(seg: &[u8], start: usize) -> (CallForm, usize, usize) {
    let mut p = start;
    // Open call-argument regions. A `55` inside one terminates an *argument*, not
    // the value we are classifying — which is why the destructor skeleton (whose
    // 2113 intrinsic carries three `55`-terminated arguments before the receiver
    // is even complete) is not misfiled as an argument push.
    let mut depth: usize = 0;
    // The call token most recently opened at each depth, so a `4C` can say which
    // kind of call it closed. Index 0 is unused.
    let mut open: Vec<(bool, i32)> = Vec::new();
    let mut last: Option<Tk> = None;
    // Whether the most recent literal was zero, for the byte-offset add that may
    // consume it. Read from the payload's first byte: the short form is a signed
    // byte, so `00` is the only spelling of zero that can precede a `27`/`28`.
    let mut lit_zero = false;
    // The head run of symbol pushes that are not callees — the stacked methods,
    // plus (when the receiver is itself a named object) the receiver.
    let mut head_syms: usize = 0;
    let mut counting_head = true;

    for _ in 0..MAX_TOKENS {
        let Some(&b) = seg.get(p) else {
            return (classify_at(Stop::Eof, last, head_syms), methods_of(last, head_syms), p);
        };
        // Decisive bytes first, at the top level only.
        if depth == 0 {
            match b {
                0x99 => return (classify_at(Stop::Bind, last, head_syms), methods_of(last, head_syms), p),
                0x32 => return (classify_at(Stop::Store, last, head_syms), methods_of(last, head_syms), p),
                0x55 => return (classify_at(Stop::ArgEnd, last, head_syms), methods_of(last, head_syms), p),
                0x41 | 0x4B => return (classify_at(Stop::StmtEnd, last, head_syms), methods_of(last, head_syms), p),
                _ => {}
            }
        }
        let mut consumed_head_sym = false;
        match b {
            0xB9 => {
                p += 1;
                let Some((_, w)) = read_token_var(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += w;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw;
                last = Some(Tk::Load);
            }
            0x33 => {
                // Three different productions open on `33`, and they are told
                // apart by what follows the literal — not guessed from position:
                //   `33 <int> <sel> 40`      an intrinsic call's selector
                //   `33 <T> <k> 27|28`       a byte-offset add
                //   `33 <T> <k>`             a plain literal operand
                if let Some(sel) = intrinsic_selector(seg, p) {
                    p += 1;
                    if !eat_int_like(seg, &mut p) || read_varint(seg, &mut p).is_none() {
                        return (CallForm::Eof, methods_of(last, head_syms), p);
                    }
                    // the `40 <TYPE>` intrinsic call token — no trailing field
                    // (`docs/IL_INTRINSIC_CALL.md` §1).
                    p += 1;
                    let Some((_, _, _, tw)) = read_type(seg, p) else {
                        return (CallForm::Eof, methods_of(last, head_syms), p);
                    };
                    p += tw;
                    depth += 1;
                    open.push((true, sel));
                    last = None;
                } else {
                    p += 1;
                    let Some((_, _, _, tw)) = read_type(seg, p) else {
                        return (CallForm::Eof, methods_of(last, head_syms), p);
                    };
                    lit_zero = seg.get(p + tw) == Some(&0x00);
                    if !eat_literal(seg, &mut p) {
                        return (CallForm::Eof, methods_of(last, head_syms), p);
                    }
                    last = Some(Tk::Lit);
                }
            }
            0x26 => {
                let mut q = p + 1;
                let Some((_, w)) = read_token_var(seg, q) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                q += w;
                p = q;
                // A `26` immediately followed by the CALL opcode is a *callee*
                // push, not a method or an object. `G().Val()` has both in its
                // head run and only the first is a method (§4).
                if seg.get(p) == Some(&0xBD) {
                    counting_head = false;
                } else {
                    last = Some(Tk::Sym);
                    consumed_head_sym = true;
                }
            }
            0xBD => {
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw;
                // the calling-convention byte, then the per-TU function-type id
                p += 1;
                if read_varint(seg, &mut p).is_none() {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                }
                depth += 1;
                open.push((false, 0));
                last = None;
            }
            0x4C => {
                p += 1;
                match (depth.checked_sub(1), open.pop()) {
                    (Some(d), Some((intr, sel))) => {
                        depth = d;
                        last = Some(Tk::CallEnd(intr, sel));
                    }
                    // A `4C` with no open call is not this grammar.
                    _ => return (CallForm::Op(0x4C), methods_of(last, head_syms), p),
                }
            }
            0x55 => {
                // depth > 0 (the depth-0 case returned above): an argument
                // terminator, `55 <TYPE>`.
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw;
                last = None;
            }
            0x2C => {
                // A convert. Deliberately does NOT update `last`: a cv-strip or a
                // pointer→pointer decay leaves the same value on the stack, and
                // the receiver's form is the form of what it converted.
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw + 1;
            }
            0x27 => {
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw;
                last = Some(Tk::OffAdd(lit_zero));
            }
            0x28 => {
                // `28 00 00`, the untyped byte-offset add. The two trailing bytes
                // are `00 00` at every captured site and are not understood
                // (`docs/IL_EXPR_LAYER.md` §4); anything else is not this token.
                p += 1;
                if !eat(seg, &mut p, &[0x00, 0x00]) {
                    return (CallForm::Op(0x28), methods_of(last, head_syms), p);
                }
                last = Some(Tk::OffAdd(lit_zero));
            }
            0x30 => {
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw;
                last = Some(Tk::Deref);
            }
            // A member bind INSIDE an open call-argument region — `g1(p->Get())`.
            // At depth 0 a `99` is decisive and returned above, so this arm is only
            // ever the nested case. Without it the walk cannot tokenize a member
            // call in an argument list and files the whole production as
            // `op-0x99`, which is how D2's 19 `op-0x99` functions arose: the
            // construct is a plain call, and the `99` is two levels down inside it.
            // `99 <TYPE> 00` is not itself a value — the `4C` that closes the call
            // it opens is — so `last` is cleared, exactly as `BD` clears it.
            0x99 => {
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return (CallForm::Eof, methods_of(last, head_syms), p);
                };
                p += tw;
                if !eat_byte(seg, &mut p, 0x00) {
                    return (CallForm::Op(0x99), methods_of(last, head_syms), p);
                }
                last = None;
            }
            0x66 => {
                // The class-pair descriptor of the 2113–2119 family. Not a value.
                if eat_class_descriptor(seg, &mut p).is_none() {
                    return (CallForm::Op(0x66), methods_of(last, head_syms), p);
                }
            }
            0x02 | 0x03 | 0x04 => {
                p += 1;
                last = Some(Tk::Op);
            }
            other => return (classify_at(Stop::Op(other), last, head_syms), methods_of(last, head_syms), p),
        }
        if counting_head {
            if consumed_head_sym {
                head_syms += 1;
            } else {
                counting_head = false;
            }
        }
    }
    (CallForm::Eof, methods_of(last, head_syms), p)
}

/// **What a `26`-rooted call production leaves on the operand stack** — the
/// return of [`eat_call_value`], and the whole of the value model's output.
///
/// Three variants because the three are three different facts about the model's
/// own honesty, and collapsing any pair would make `cstack_ok` a claim the
/// walker cannot support (`GAPS.md` §6's "two facts sharing one field").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CallValue {
    /// The call returns a value whose TYPE is one [`ValueClass`] names, and the
    /// caller pushes exactly that class. The model followed the token.
    Value(ValueClass),
    /// The call returns **`void`** — class nibble 7, `docs/IL_LOAD_TYPES.md` §1's
    /// own table — so **nothing** is pushed. The model followed the token: the
    /// stack really is unchanged, which is not the same statement as `Opaque`.
    Void,
    /// The call returns a value whose TYPE is outside [`ValueClass`]'s
    /// vocabulary — a float, a narrow scalar, an aggregate, a `long long`. The
    /// production is consumed by width but the stack effect is **not modeled**,
    /// so the caller clears `cstack_ok` rather than guessing a class.
    Opaque,
}

/// **THE MEMBER-CALL VALUE MODEL** (`lane w-value`, board **#1940**) — consume
/// the whole `26 … BD … 4C` production at `p` and report the value it leaves.
///
/// # What this is for, and what board row it pays
///
/// `parse_expr`'s `0x26` arm was one byte — `return Err(classify(seg, *p))` —
/// so the walk stopped at the *first* member call in a body and every construct
/// behind it was invisible. Board **#1534** measures that family at **449,274
/// bodies / 36,751 emitted functions**, names it "the largest reader family on
/// the board", and records that it "has still never had a whole-production
/// counterfactual"; its own prescription is *"a sink that consumes `26 … 4C`
/// **entire** — a bracket walk in `mcall.rs`, not a width row"*. This is that
/// bracket walk, and it is **permanent rather than env-gated**, because what it
/// changes is which construct a blocked function is *filed under*, never whether
/// it is blocked.
///
/// # The acceptance theorem — why this cannot widen the class
///
/// Every path that reaches byte `0x26` inside `parse_expr` returns `Err`
/// today, unconditionally. This function pushes **no [`IlOp`]** and the caller
/// sets a poison that refuses at the end of the walk under the *same*
/// [`classify`] block the old arm produced. So the arm can only ever replace one
/// `Err` with another `Err`: **`parse_expr` cannot return `Ok` on any body it
/// refuses today**, the census cannot over-claim (board #139), and `mismatch`
/// cannot move. The three levels that could falsify that — the 878-TU per-TU
/// verdict set by name, the fixtures at `/O1` and `/Ox`, and the two census
/// counts — are the lane's evidence and not this comment.
///
/// # The grammar, and why it is narrower than [`walk_detail`]'s
///
/// ```text
///   CALLVALUE := (26 <tok>)+ <designator>* [99 <TYPE> 00]
///                BD <ret-TYPE> <conv> <fn-type>   ( <arg> 55 <TYPE> )*   4C
/// ```
///
/// The walk is a **bracket** walk: `BD` and the intrinsic call token `40` each
/// open a region, `4C` closes one, and the production ends at the `4C` that
/// closes a **`BD`-opened region at depth 0**. That last condition is what makes
/// a *chained* receiver (`26 <M> 33 <T> <sel> 40 <T> … 4C 99 … BD … 4C`, the
/// generated-destructor skeleton of `docs/IL_CALL_IN_EXPR.md` §5) fall out
/// without a special case: its first `4C` returns to depth 0 over an
/// **intrinsic** region, so the walk keeps going and ends at the second.
///
/// **A bare data-symbol address push is NOT a call and is deliberately not
/// handled.** `f("hello")`, `&global`, an array decay — ~18 % of the bucket
/// (§2) — reach no `BD`, so this returns `None` with `p` untouched and the
/// caller raises exactly today's block. That keeps the `data-addr` sub-buckets
/// measuring the population they measure now, which several board rows are
/// written against.
///
/// **Anything the walk cannot tokenize returns `None` with `p` untouched**, so
/// the fallback is byte-for-byte the current refusal. The residue that produces
/// is a *price*, published rather than guessed past: a walker that resynchronised
/// on an unknown opcode would be inventing a width, which is the one thing
/// `docs/GAPS.md` §6 says a reader may not do.
///
/// # Vocabulary, by depth
///
/// At **depth 0** the token set is exactly [`walk_detail`]'s spine and
/// designator vocabulary — `26 B9 33 2C 27 28 30 66 99 BD 4C 02 03 04`. Inside
/// an open region ([`depth`] > 0) the bare one-byte operators of
/// [`BARE_BINARY_OPS`] and the argument terminator `55 <TYPE>` are added,
/// because an argument is an expression and those eleven are the bare tokens the
/// project has capture witnesses for. **The relationals are admitted inside a
/// call argument and refused at depth 0 on purpose**: a `1F` *after* the call
/// has closed is the enclosing expression's, and letting it reach `parse_expr`
/// is exactly how the family's head is supposed to move.
pub(crate) fn eat_call_value(seg: &[u8], p: &mut usize) -> Option<CallValue> {
    if seg.get(*p) != Some(&0x26) {
        return None;
    }
    let mut q = *p;
    // One entry per open call region: `Some(v)` for a `BD`-opened one carrying
    // the value its `4C` will leave, `None` for an intrinsic `40` region (whose
    // result this model does not classify — it is a receiver, not a value the
    // enclosing expression reads).
    let mut open: Vec<Option<CallValue>> = Vec::new();

    for _ in 0..MAX_TOKENS {
        let &b = seg.get(q)?;
        let depth = open.len();
        match b {
            0x26 => {
                q += 1;
                let (_, w) = read_token_var(seg, q)?;
                q += w;
            }
            0xB9 => {
                q += 1;
                let (_, w) = read_token_var(seg, q)?;
                q += w;
                let (_, _, _, tw) = read_type(seg, q)?;
                q += tw;
            }
            0x33 => {
                // The same three-way split [`walk_detail`] makes, through the
                // same locator: an intrinsic selector opens a region, a
                // byte-offset add's literal and a plain literal do not.
                if intrinsic_selector(seg, q).is_some() {
                    q += 1;
                    if !eat_int_like(seg, &mut q) || read_varint(seg, &mut q).is_none() {
                        return None;
                    }
                    // the `40 <TYPE>` intrinsic call token
                    if seg.get(q) != Some(&0x40) {
                        return None;
                    }
                    q += 1;
                    let (_, _, _, tw) = read_type(seg, q)?;
                    q += tw;
                    open.push(None);
                } else {
                    q += 1;
                    read_type(seg, q)?;
                    if !eat_literal(seg, &mut q) {
                        return None;
                    }
                }
            }
            // A convert applied to the value on top — the named-object
            // receiver's address decay (`26 <sym> 2C <ptr-TYPE> 00`) among
            // others. `TYPE` plus one trailing byte, exactly as [`walk_detail`]
            // reads it.
            0x2C => {
                q += 1;
                let (_, _, _, tw) = read_type(seg, q)?;
                q += tw + 1;
            }
            0x27 => {
                q += 1;
                let (_, _, _, tw) = read_type(seg, q)?;
                q += tw;
            }
            0x28 => {
                q += 1;
                if !eat(seg, &mut q, &[0x00, 0x00]) {
                    return None;
                }
            }
            0x30 => {
                q += 1;
                let (_, _, _, tw) = read_type(seg, q)?;
                q += tw;
            }
            0x66 => {
                eat_class_descriptor(seg, &mut q)?;
            }
            0x99 => {
                q += 1;
                let (_, _, _, tw) = read_type(seg, q)?;
                q += tw;
                if !eat_byte(seg, &mut q, 0x00) {
                    return None;
                }
            }
            0xBD => {
                q += 1;
                let (tag, kind, _, tw) = read_type(seg, q)?;
                q += tw;
                // the calling-convention byte, then the per-TU function-type id
                q += 1;
                read_varint(seg, &mut q)?;
                open.push(Some(call_value_of(tag, kind)));
            }
            0x4C => {
                q += 1;
                match open.pop() {
                    // The `4C` that closes a `BD` region opened at depth 0 ends
                    // the production — **unless the value it just produced is
                    // itself the receiver of the next bind**, which is what a
                    // member-call CHAIN is (`p->Next()->Val()`, §4). That case
                    // is not a special case bolted on: it was found by the
                    // module's own `a_chain_is_a_chain_in_both_statement_positions`
                    // test going red on the first build of this walker, which
                    // returned at the inner `4C` and handed `parse_expr` a `99`
                    // it has no arm for.
                    Some(Some(v)) if open.is_empty() && !binds_the_result(seg, q) => {
                        *p = q;
                        return Some(v);
                    }
                    Some(_) => {}
                    // A `4C` with no open region is not this grammar.
                    None => return None,
                }
            }
            0x55 if depth > 0 => {
                q += 1;
                let (_, _, _, tw) = read_type(seg, q)?;
                q += tw;
            }
            0x02 | 0x03 | 0x04 => q += 1,
            x if depth > 0 && BARE_BINARY_OPS.contains(&x) => q += 1,
            _ => return None,
        }
    }
    None
}

/// Whether the value a just-closed call left is **immediately bound as the
/// receiver of another member call** — i.e. whether the production continues.
///
/// Two spellings, both from `docs/IL_CALL_IN_EXPR.md` §4: the bind directly
/// (`p->Next()->Val()` is `26 <Val> 26 <Next> B9 <p> 99 … BD … 4C 99 … BD … 4C`),
/// and the bind behind one convert, which is the same decay §3.1 records on a
/// named-object receiver.
///
/// **A `2C` that is NOT followed by a bind is deliberately left alone**, and
/// that asymmetry is the lane's whole payoff: it is the enclosing expression's
/// conversion, `parse_expr`'s own `0x2C` arm handles it, and it now finds a
/// value on the stack where it used to find nothing and raise
/// `expr-convert-no-value` (board #1462, 4,973 witnesses).
fn binds_the_result(seg: &[u8], q: usize) -> bool {
    match seg.get(q) {
        Some(0x99) => true,
        Some(0x2C) => match read_type(seg, q + 1) {
            // `2C <TYPE> <one byte>` — the width `walk_detail` reads.
            Some((_, _, _, w)) => seg.get(q + 1 + w + 1) == Some(&0x99),
            None => false,
        },
        _ => false,
    }
}

/// The [`CallValue`] a `BD`'s return TYPE names.
///
/// `void` is the **class nibble 7** of `docs/IL_LOAD_TYPES.md` §1's own class
/// table — the reading the crate already carries from captures
/// (`readers.rs`'s `82 07 03` witness), not a new one — and it is a distinct
/// answer from "a class this parser does not model", which is why
/// [`CallValue`] has three variants and not two.
fn call_value_of(tag: u8, kind: u8) -> CallValue {
    if (kind & 0x0F) == 0x7 {
        return CallValue::Void;
    }
    match value_class(tag, kind) {
        Some(c) => CallValue::Value(c),
        None => CallValue::Opaque,
    }
}

/// Turn `(what stopped the walk, the value on top, the head symbol run)` into a
/// sub-bucket.
/// How many of the head symbol pushes were *methods*. When the receiver is itself
/// a named object it is the last of the run, so one of them is not a method;
/// otherwise all of them are.
///
/// One locator, shared: [`classify_at`] turns it into a form and [`reanchor_chain`]
/// compares it against the statement's bind count.
fn methods_of(last: Option<Tk>, head_syms: usize) -> usize {
    if last == Some(Tk::Sym) {
        head_syms.saturating_sub(1)
    } else {
        head_syms
    }
}

fn classify_at(stop: Stop, last: Option<Tk>, head_syms: usize) -> CallForm {
    let methods = methods_of(last, head_syms);
    match stop {
        Stop::Bind => {
            // Two or more stacked methods is a chain, whatever the innermost
            // receiver is: the lowering needs a frame and one `bl` per link, so
            // the receiver form is not the discriminator there (§4).
            if methods > 1 {
                return CallForm::Chained;
            }
            match last {
                Some(Tk::Load) => CallForm::RecvLoad,
                Some(Tk::Deref) => CallForm::RecvDeref,
                Some(Tk::OffAdd(false)) => CallForm::RecvField,
                Some(Tk::OffAdd(true)) => CallForm::RecvFieldZero,
                Some(Tk::Sym) => CallForm::RecvObject,
                Some(Tk::CallEnd(false, _)) => CallForm::RecvCall,
                Some(Tk::CallEnd(true, sel)) => CallForm::RecvIntrinsic(sel),
                _ => CallForm::RecvOther,
            }
        }
        // A `32` reached by *this* walk stores the value the walk classified — it
        // is not a store *to* the symbol the walk started on. That case cannot
        // arrive here: a statement-head `26` is consumed by the body dispatch
        // (`mod.rs`) as an assignment destination and never reaches `parse_expr`,
        // so §7.2's `26 <dst-sym> … 32` files under `expr-convert` /
        // `expr-op-0x27` instead. A store to a global that *does* reach this walk
        // (`f(gS.b = a)`) has an independent value on top and lands in `other`,
        // which is the honest answer — separating it needs a model of nested
        // assignment that this rung does not have.
        Stop::Store | Stop::ArgEnd | Stop::StmtEnd => match last {
            Some(Tk::CallEnd(false, _)) if methods == 0 => CallForm::NestedCall,
            Some(Tk::CallEnd(true, sel)) => CallForm::Intrinsic(sel),
            Some(Tk::Deref) => CallForm::DataRead,
            Some(Tk::Sym) | Some(Tk::OffAdd(_)) => CallForm::DataAddr,
            _ => CallForm::Other,
        },
        Stop::Op(b) => CallForm::Op(b),
        Stop::Eof => CallForm::Eof,
    }
}

// --- the whole-body-completeness matcher ------------------------------------

/// Which productions the completeness matcher may use: the receiver form under
/// test, and — for the **both-handled** measure — one second construct.
///
/// The pair is the unit because a rung is a pair. §14.1's `-whole` column ranked
/// single forms and ranked them well (two rungs converted 1:1 off it), but it
/// cannot see past a form: three sub-buckets holding 172,615 functions read
/// 0.0 %, and a number that is 0 for the three largest rows has no ordering
/// information left in it. Admitting two constructs at once restores it.
#[derive(Clone, Copy)]
struct Admit {
    form: CallForm,
    /// The extra constructs granted, in the order the greedy walk found them.
    also: [Blocker; MAX_ADMIT],
    n: usize,
}

/// How many extra constructs the greedy measure will grant before giving up.
///
/// Four, because the question this rung exists to answer is *"do these bodies share
/// **one** further blocker, or does each carry three unrelated ones"* — and a
/// distribution over 1…4 answers it directly, where a yes/no at 1 cannot. A body
/// still refusing after four is `-more`, and four unrelated constructs is not a rung
/// under any reading.
const MAX_ADMIT: usize = 4;

impl Admit {
    fn bare(form: CallForm) -> Admit {
        Admit { form, also: [Blocker::None; MAX_ADMIT], n: 0 }
    }
    fn granted(&self) -> &[Blocker] {
        &self.also[..self.n]
    }
    fn push(&mut self, b: Blocker) {
        self.also[self.n] = b;
        self.n += 1;
    }
    fn holds(&self, b: Blocker) -> bool {
        self.granted().contains(&b)
    }
    /// Whether some granted `Blocker::Type` names the TYPE at `p`.
    fn admits_type(&self, seg: &[u8], p: usize) -> bool {
        self.granted().iter().any(|b| matches!(b, Blocker::Type(c) if c.matches(seg, p)))
    }
}

/// The **furthest** refusal the matcher reached, and what kind it was.
///
/// Furthest-refusal, not first-refusal, and the difference is the whole
/// instrument. The matcher speculates: at every value position it tries the
/// form's production first, and that attempt walks *into* a call before finding
/// the byte it cannot take. So the deepest position reached is the one that names
/// the construct — a body whose second statement is `q->o.Get()` records the
/// refusal at the `33 <k> 27` sub-object address inside that call, not at the
/// outer `26`, and the key is `then-call-recv-field` rather than a useless
/// `then-op-0x26`.
///
/// `GAPS.md` §6's mis-attribution failure is the hazard here, and the guard is
/// that [`Fail::blocker`] names a **construct** at the position, never the
/// position itself: a `26` is resolved by re-running [`walk`], which is the same
/// backward classifier D2 uses, so a second blocker that is another member call
/// is filed by *its* receiver designator. The validation is structural rather
/// than argued: `blocker_is_measured` pairs are re-matched with both constructs
/// admitted, and a wrong name shows up as a `-more` where the construct really was
/// the only thing missing.
struct Fail {
    at: usize,
    kind: FailKind,
    /// How many data symbols the current parse has materialized **into a call
    /// argument** ([`eat_data_designator`] succeeding inside an argument region).
    ///
    /// Argument regions specifically, and not every designator, because the other
    /// two places a `26 <tok>` reaches this production are not addresses the body
    /// has to materialize: the **callee push** of the call itself (excluded in
    /// [`eat_data_designator`]) and an assignment statement's **destination** push,
    /// which the greedy value sequence swallows before `body_matches` gets to try
    /// its assignment arm. Counting those made `x = uc("hi")` — one string — report
    /// two. The argument region is the one position where the count means what the
    /// ranking needs it to mean: how many symbol addresses have to be in registers
    /// at the call.
    ///
    /// Carried on `Fail` because `Fail` is the one `&mut` already threaded through
    /// every production, and `Admit` is `Copy`. It is only meaningful on a parse
    /// that *finished*: the matcher speculates, so every cursor rewind must restore
    /// this too, or a designator consumed by an abandoned attempt is counted twice.
    /// [`Fail::mark`]/[`Fail::rewind`] are that pairing, and every `*p = save` in
    /// this module has one.
    syms: u32,
    /// Nesting depth of open **call-argument regions** ([`eat_call_and_args`]).
    /// Balanced by construction — the region is entered and left in one place — so
    /// a speculative parse that fails inside one cannot leave it open.
    args_depth: u32,
}

/// What sort of thing the matcher refused, which decides how the position is read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FailKind {
    /// A token in a **value** position had no production. `at` is that token.
    Value,
    /// A **TYPE** was outside the admitted classes. `at` is the type triple.
    Type,
    /// A `26`-opened production at `of` had a **receiver this form is not**. `at`
    /// is the receiver's first byte, for ordering; the construct is named by
    /// re-walking from `of`, which is the only way to get it right — the receiver's
    /// first byte is `2C` for a decayed string literal, `26` for a named object and
    /// `9B` for a by-value temporary, and reporting those bytes would file three
    /// different constructs as three uninformative opcodes.
    Receiver(usize),
    /// The **return plumbing / function tail** did not match. `at` is its start.
    Plumbing,
    /// A structural refusal with no token at it.
    Struct(Structural),
}

impl Fail {
    fn new() -> Fail {
        Fail { at: 0, kind: FailKind::Value, syms: 0, args_depth: 0 }
    }

    /// Snapshot the designator count, to be paired with a cursor save.
    fn mark(&self) -> u32 {
        self.syms
    }

    /// Restore it, to be paired with a cursor rewind. The furthest-refusal fields
    /// are deliberately **not** restored — the deepest position reached is the
    /// answer whether or not the attempt that reached it was kept.
    fn rewind(&mut self, m: u32) {
        self.syms = m;
    }

    /// The count as the 2-bit census class: 1, 2, or 3-and-above.
    fn sym_class(&self) -> u64 {
        match self.syms {
            0 => SYMS_UNSET,
            1 => 1,
            2 => 2,
            _ => 3,
        }
    }

    /// Record a refusal, keeping the furthest. Ties go to the **first** note,
    /// which is the innermost: leaf readers note before their callers do, so the
    /// most specific reading of a position wins.
    fn note(&mut self, at: usize, kind: FailKind) {
        if at > self.at {
            self.at = at;
            self.kind = kind;
        }
    }

    /// Record a refusal that must WIN a tie — for a refusal that is a property
    /// of a whole run rather than of one byte, and so is strictly more specific
    /// than any single-token note already sitting at the same offset.
    fn note_forcing(&mut self, at: usize, kind: FailKind) {
        if at >= self.at {
            self.at = at;
            self.kind = kind;
        }
    }

    /// Name the **construct** at the refusal. Every arm is a construct or an
    /// honest hex bucket; none of them is a position.
    fn blocker(&self, seg: &[u8]) -> Blocker {
        match self.kind {
            FailKind::Struct(s) => Blocker::Structure(s),
            FailKind::Type => Blocker::Type(TypeClass::at(seg, self.at)),
            FailKind::Plumbing => match seg.get(self.at) {
                Some(&b) => Blocker::Plumbing(b),
                None => Blocker::Eof,
            },
            FailKind::Receiver(of) => Blocker::Call(walk(seg, of)),
            FailKind::Value => match seg.get(self.at) {
                None => Blocker::Eof,
                // Another `26`-opened production: classify it the way D2 classifies
                // the first one, so the key names its receiver designator.
                Some(&0x26) => Blocker::Call(walk(seg, self.at)),
                Some(&0x30) => Blocker::DerefLoad,
                Some(&0x9B) => Blocker::TempBind,
                Some(&0xBD) => Blocker::PlainCall,
                // `67` is virtual dispatch and `9A` is its bind (§3), so either byte
                // in a value position is the same construct.
                Some(&0x67) | Some(&0x9A) => Blocker::Virtual,
                Some(&b @ (0x38 | 0x39)) => Blocker::Branch(b),
                // A `99` at depth 0 in a *value* position is not a first bind — the
                // form's own production consumed that one — it is the next link of a
                // chain.
                Some(&0x99) => Blocker::ChainBind,
                Some(&0x27) | Some(&0x28) => Blocker::OffAdd,
                Some(&b) => Blocker::Op(b),
            },
        }
    }
}

/// Whether the **pair**'s joint completeness can be measured at all: a second
/// blocker with no production has no both-handled figure, and the key says so by
/// carrying neither `-whole` nor `-more`.
///
/// `Blocker::Call` defers to [`form_is_measured`], so the honesty gate composes:
/// a pair whose second half is `op-0x5C` or a virtual dispatch is reported as
/// UNMEASURED at the pair level even though the *first* half is measured.
fn blocker_is_measured(blk: Blocker) -> bool {
    match blk {
        Blocker::Call(f) => form_is_measured(f),
        Blocker::DerefLoad | Blocker::PlainCall | Blocker::ChainBind | Blocker::OffAdd => true,
        // Admitting "a TYPE of class c" widens `eat_admitted_type` by exactly one
        // class, which is a real production. `NotAType` is not a class — it is a
        // desync signal — and admitting it would mean nothing.
        Blocker::Type(TypeClass::NotAType) => false,
        Blocker::Type(_) => true,
        Blocker::Op(b) => BARE_BINARY_OPS.contains(&b),
        _ => false,
    }
}

/// The operator bytes that are **one byte and nothing else** — no TYPE, no
/// varint, no trailing field — so that granting one widens the completeness
/// matcher's grammar by exactly one token, the same way `Blocker::OffAdd` does.
///
/// **Before W37 this set was empty**, which made every `Blocker::Op` UNMEASURED
/// and put a 102,382-function row (5.5 % of everything blocked, #4 on the board)
/// at the top of a ranking with no completeness bit to rank it by — the hazard
/// `GAPS.md` §6 records for `expr-op-0x99`, in the form where the row *is*
/// reaching the classifier and the classifier has nothing to say about it. What
/// that cost is measured in `docs/rungs/2026-07-31-bit-and-declined.md`: the row
/// is worth **0**, and finding that out took a scratch build where it should have
/// taken a warm scan.
///
/// Membership needs **two** pieces of evidence, not one, and both are in the
/// tree:
///
/// 1. **A capture witness that the token is bare.** `c2rs census
///    fixtures/cpp/w37_bit_and_neg.cpp` prints one per byte, and they are all the
///    same shape — an `unsigned` load, a literal, the operator, and then whatever
///    consumes the value with nothing in between:
///
///    ```text
///      0B  b9 <x> 86 42 75 · 33 86 41 74 01 · 0b · 38 <label>
///      09  b9 <x> 86 42 75 · 33 86 41 74 03 · 09 · 2c 86 41 74 00
///      0A  b9 <x> 86 42 75 · 33 86 41 74 03 · 0a · 2c 86 41 74 00
///      0C  b9 <x> 86 42 75 · 33 86 41 74 01 · 0c · 2c 86 41 74 00
///      0D  b9 <x> 86 42 75 · 33 86 41 74 01 · 0d · 2c 86 41 74 00
///    ```
///
///    `0B`'s is corroborated in the wild by [`Blocker::Branch`]'s own doc
///    comment (`src/system/hamobj/Ham.cpp`) and by the `expr-bit-and` hexdump in
///    `src/system/world/Dir.cpp`.
/// 2. **A 1:1 redistribution over the whole 878-TU workload.** Granting the set
///    moves `expr-call-in-expr-recv-load-then-bit-and`'s 102,382 into
///    `…-then-bit-and-and-branch-more` (102,374) plus 8 named stragglers, and the
///    deltas over *every* key sum to exactly 0 with the census numerator
///    unchanged at 602,703 and census/gate disagreement still 0. A byte that were
///    not bare would desync the matcher and scatter its row across the hex tail.
///
/// A byte with only the first is a guess about a field width; a byte with only
/// the second is a coincidence.
///
/// **The relational family `1F`–`24` is here as of W42**, and it was excluded
/// before on neither piece of evidence. The exclusion read a *different* byte —
/// `… 33 86 41 74 01 · 19 · 86 42 75 …` — and generalised across a numeric
/// neighbourhood; `19` belongs to the **compound-assign** family, which does
/// carry a TYPE, and the numeric order hides the boundary. Both pieces of
/// evidence are now in the tree (`docs/OPERATOR_GRANTS.md`):
///
/// 1. A capture with the compound-assign control **in the same TU**, so the two
///    families are read beside each other rather than from memory. In value
///    position the operator is followed immediately by `2C`, in branch position
///    by `38`, with nothing in between in twelve leaves and two branch bodies:
///
///    ```text
///      22  b9 ee 09 86 42 75 · 33 86 42 75 03 · 22 · 2c 86 41 74 00 · 41 …
///      23  b9 16 0a 86 41 74 · 33 86 41 74 03 · 23 · 38 19 0a …
///      0F  26 1a 0a · 33 86 41 74 03 · 0f · 86 41 74 · 4b …   THE CONTROL (`+=`)
///    ```
///
/// 2. A 1:1 redistribution over the 878-TU workload: every `…-then-cmp-*` row
///    empties into its own `-and-<second>-<whole|more>` children, the deltas over
///    every key sum to exactly 0, and the census numerator and census/gate
///    disagreement are unchanged.
///
/// **Signedness is not in the opcode** — unsigned `<` and signed `<` are both
/// `22`, and only the operand TYPE separates them. That matters to the *parser*
/// (`super::shapes::framed_compare`), not here: this set is about width.
///
/// What is still deliberately NOT here, so the omissions are decisions rather
/// than oversights: `1A` (`!`) is unary, not a binary operator over two stack
/// values; and `1B`/`1C` (`||`/`&&`) short-circuit to branches and no capture
/// shows the byte at all (1 and 3 functions on the workload).
const BARE_BINARY_OPS: &[u8] =
    &[0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24];

/// **The whole-body-completeness measure.** True when the **entire segment**
/// parses with `form` admitted as a value-producing operand and *no other* new
/// production:
///
/// ```text
///   body := LO 53 <scopes> stmt* return
///   stmt := <scopes> [ 26 <dst> ] vexpr [ 32 <T> ] 4B
///   return := <scopes> ( 3A … | vexpr 41 <T> … ) <plumbing to the segment end>
///   vexpr  := ( VALUE(form) | B9 <tok> <T> | 33 <T> <lit> | 2C <T> <b> | 02|03|04 )+
///   T      := an int-like or pointer-class TYPE — never float, narrow or aggregate
/// ```
///
/// **Why this exists at all.** `docs/IL_CALL_IN_EXPR.md` §13.3: a first-blocker
/// histogram cannot rank sub-buckets. D1 put +17,864 in class and dropped its
/// bucket by exactly 17,864 because its grammar accepts a whole segment or
/// nothing; the `.sy` rung cleared 547,082 first blockers for +17,286 because it
/// did not. So a sub-bucket of 100,000 bodies each carrying three further
/// blockers is worth less than one of 20,000 that are complete, and the count
/// alone cannot tell them apart. This can.
///
/// **What it is not.** An **upper bound on in-class yield**, not a promise of one.
/// It is a grammar measure: the codegen-class gates are deliberately *not*
/// applied — no `straight_line_is_out_of_class`, no formal/`.sy` membership for a
/// store destination, no register assignment for the receiver, no `/Gy` COMDAT
/// layout, and a store or a result may be pointer-typed where the emitter has
/// only ever been graded on `int`. Read a `-whole` count as "nothing but `form`
/// stands between this body and the modeled grammar", and expect the realized
/// yield to be below it.
///
/// Diagnostic only. Nothing here can accept a function; the caller's `Err` stays
/// an `Err`.
fn body_matches(seg: &[u8], lo: usize, adm: Admit, fail: &mut Fail) -> bool {
    let mut p = crate::func::ops_start(seg, lo);
    if !eat_byte(seg, &mut p, 0x53) {
        fail.note(p, FailKind::Struct(Structural::BodyMarker));
        return false;
    }
    let mut depth = BODY_SCOPE_DEPTH;
    for _ in 0..MAX_STMTS {
        if eat_scopes(seg, &mut p, &mut depth).is_err() {
            fail.note(p, FailKind::Struct(Structural::Scopes));
            return false;
        }
        // A void return opens directly on the plumbing's `3A` — no expression.
        if seg.get(p) == Some(&0x3A) {
            return eat_body_end(seg, &mut p, depth, fail);
        }
        // An expression, optionally preceded by an assignment destination push.
        // Tried in that order and on a copy of the cursor, because a statement
        // opening on `26` is ambiguous: it is a destination for `x = p->M();` and
        // a *method* push for `p->M();`, and only trying the expression settles
        // it. (`26 <dst>` is not itself a value here — a data symbol is only
        // admitted when `form` is one of the data designators.)
        let save = p;
        let msave = fail.mark();
        if !eat_value_seq(seg, &mut p, adm, fail) {
            p = save;
            fail.rewind(msave);
            if !eat_byte(seg, &mut p, 0x26) {
                fail.note(p, FailKind::Value);
                return false;
            }
            match read_token_var(seg, p) {
                Some((_, w)) => p += w,
                None => {
                    fail.note(p, FailKind::Value);
                    return false;
                }
            }
            if !eat_value_seq(seg, &mut p, adm, fail) {
                return false;
            }
        }
        // A store, when the statement has one.
        if eat_byte(seg, &mut p, 0x32) && !eat_admitted_type(seg, &mut p, adm) {
            fail.note(p, FailKind::Type);
            return false;
        }
        // The generated destructor's opaque statement trailer (`5C <int> <flag>`),
        // admitted here for the same reason D1 admits it and with the same measured
        // flag values — see [`eat_dtor_stmt_trailer`].
        eat_dtor_stmt_trailer(seg, &mut p);
        if eat_byte(seg, &mut p, 0x4B) {
            continue; // …and on to the next statement.
        }
        // Not a statement end, so this expression is the returned one. The result
        // annotation's TYPE is read here rather than by `eat_return_plumbing`,
        // which requires int-like: a member call may return a pointer, and
        // refusing that would understate every getter.
        if !eat_byte(seg, &mut p, 0x41) {
            fail.note(p, FailKind::Value);
            return false;
        }
        if !eat_admitted_type(seg, &mut p, adm) {
            fail.note(p, FailKind::Type);
            return false;
        }
        return eat_body_end(seg, &mut p, depth, fail);
    }
    fail.note(p, FailKind::Struct(Structural::StmtLimit));
    false
}

/// A statement count bound: a body this long is not one a rung can vouch for, and
/// an unbounded loop over a corrupt stream is not acceptable in an instrument that
/// runs over 2.4 M functions. Reported as `struct-stmt-limit`, not as a value
/// refusal, so hitting it can never be mistaken for a construct.
const MAX_STMTS: usize = 64;

/// The measured `(statement-trailer flag, sub-object-trailer flag)` pairs of the
/// generated destructor, copied from D1's [`super::shapes::try_parse_empty_dtor_delegation`]
/// rather than re-derived: `/EH…` clears bit `0x10` in both, the fixture profile
/// (`/Ox`, no `/EH`) gives `(0x11, 0x31)` and the dc3 workload profile
/// (`/O1 /Oi /EHsc`) gives `(0x01, 0x21)`, and the reference emits the same bytes
/// for both. A third value refuses.
const TRAILER_FLAGS: [(u8, u8); 2] = [(0x11, 0x31), (0x01, 0x21)];

/// Consume an optional `5C <int-TYPE> <flag>` statement trailer, reporting nothing
/// — a statement either has one or does not.
///
/// Admitting these two opaque trailers **outside** D1's rigid skeleton is exactly
/// the "skipped field" hazard `GAPS.md` §6 warns about, and it is deliberate and
/// bounded here: this function is only ever reached from
/// [`whole_body_is_one_value`], which cannot accept anything. Without them the
/// measure would report **zero** complete bodies for every destructor sub-shape —
/// which is most of the largest sub-buckets — and a vacuous zero is worse than a
/// labelled approximation. The flag byte is still required to be one of the two
/// measured values, so the field is gated, not skipped.
///
/// **That gate is what `expr-call-in-expr-*-op-0x5C` is** — see
/// [`TrailerSink`], which is the counterfactual instrument for it.
fn eat_dtor_stmt_trailer(seg: &[u8], p: &mut usize) -> bool {
    eat_dtor_stmt_trailer_with(trailer_sink(), seg, p)
}

/// `C2RS_SINK_MCALL_TRAILER` — **lane `w-5c2`'s board #1453 counterfactual**, and
/// the only thing that reads it.
///
/// # The row it is the instrument for
///
/// `expr-call-in-expr-*-op-0x5C` is **1,212 functions in 810 TUs** on the default
/// 878-TU scan (board **#1428**), and it is raised **here**: when
/// [`eat_dtor_stmt_trailer`] refuses, the cursor is left on the `5C`, the `4B`
/// and `41` arms in [`body_matches`] both miss, and `Fail::note` files the
/// position as [`FailKind::Value`] → [`Blocker::Op`]`(0x5C)`.
///
/// Two facts make a counterfactual worth running rather than a decline worth
/// repeating:
///
/// 1. **The gate is narrower than the tree's own reader of the same byte.**
///    `w-5c` (board #1423) anchored `5C <TYPE> <varint>` on **335,716 sites with
///    0 desyncs on two independent anchors**, and `control_flow.rs::operand()`
///    has read it at that width since 2026-07-31. This gate takes a *4-byte
///    integer* TYPE and **two** flag values. The workload's measured states
///    include `02 03 04 41 43` and a 9,645-site escape (`80 01 01 00 00`).
/// 2. **The `0` published beside the row is a RENDERING.**
///    `docs/IL_CALL_IN_EXPR.md` §16.2 files it as *"`op-0x5C` | 890 | 0 | a
///    destructor statement trailer whose flag is neither measured value"* — but
///    that `0` is the *whole-within-4* column, and [`blocker_is_measured`] is
///    `BARE_BINARY_OPS.contains(&0x5C)`, which is **false**, so [`mark_whole`]'s
///    greedy chain breaks on its first iteration and the column **cannot** hold
///    any other number. A zero the instrument is structurally unable to move is
///    not a measurement of the row.
///
/// | value | the trailer becomes | prices |
/// |---|---|---|
/// | unset (default) | `5C <int-like TYPE> <flag ∈ {0x11, 0x01}>` | the shipped gate |
/// | `flag` | `5C <int-like TYPE> <any short-form byte>` | the flag whitelist alone |
/// | `varint` | `5C <any TYPE> <varint>` — `w-5c`'s anchored width | the whole row |
///
/// The arms are **nested** (`Measured ⊆ Flag ⊆ Varint`), on `C2RS_SINK_BRANCH`'s
/// pattern (board #440), and `the_trailer_sink_arms_are_nested` checks it: a
/// wider arm that refused something a narrower one takes would understate the
/// recovery in the one direction nobody looks.
///
/// # It is NOT a proposal, and it cannot be one
///
/// Board #661's hazard — a sink that quietly *accepts* differently as well as
/// measuring — cannot arise here, and the reason is structural rather than
/// argued: [`eat_dtor_stmt_trailer`] has **exactly one** call site
/// ([`body_matches`]), which is reached only from [`mark_whole`], whose own
/// header says *"Diagnostic only: the `Err` stays an `Err`"*. No arm of this sink
/// can push an `IlOp`, move a census numerator, or change a graded byte. What it
/// moves is a census **key**, which is the whole point.
///
/// Shipping it permanently rather than reverting it is deliberate: three rows
/// were re-scheduled this session after a previous lane built a counterfactual
/// for them and then removed it, so the next reader had to rebuild the
/// experiment before it could re-rank the row. `C2RS_SINK_STORE_TYPE`,
/// `C2RS_SINK_BRANCH` and `C2RS_CFRESIDUE_ADMIT` are the three that stayed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TrailerSink {
    /// The shipped gate: an int-like TYPE and a flag in [`TRAILER_FLAGS`].
    Measured,
    /// The same TYPE gate, any single flag byte.
    Flag,
    /// `w-5c`'s anchored width: any well-formed TYPE, then a varint state.
    Varint,
}

fn trailer_sink() -> TrailerSink {
    static ON: std::sync::OnceLock<TrailerSink> = std::sync::OnceLock::new();
    *ON.get_or_init(|| match std::env::var("C2RS_SINK_MCALL_TRAILER").as_deref() {
        Ok("flag") => TrailerSink::Flag,
        Ok("varint") => TrailerSink::Varint,
        // An unrecognized spelling is the shipped gate, never a silent widening —
        // `docs/STATUS.md` trap 5.
        _ => TrailerSink::Measured,
    })
}

/// The trailer with the arm passed in rather than read from the environment, so
/// the arms can be graded **against each other**: the sink resolves through a
/// process-global `OnceLock`, and the property that matters most about a
/// counterfactual instrument is a relation between its arms.
fn eat_dtor_stmt_trailer_with(sink: TrailerSink, seg: &[u8], p: &mut usize) -> bool {
    let save = *p;
    if !eat_byte(seg, p, 0x5C) {
        return false;
    }
    let ty_ok = match sink {
        TrailerSink::Measured | TrailerSink::Flag => eat_int_like(seg, p),
        TrailerSink::Varint => match read_type(seg, *p) {
            Some((_, _, _, w)) => {
                *p += w;
                true
            }
            None => false,
        },
    };
    if !ty_ok {
        *p = save;
        return false;
    }
    let state_ok = match sink {
        TrailerSink::Measured => match seg.get(*p) {
            Some(&f) if TRAILER_FLAGS.iter().any(|&(s, _)| s == f) => {
                *p += 1;
                true
            }
            _ => false,
        },
        // A single **short-form** state byte. `< 0x80` deliberately: the escape is
        // `80 <LE32>`, so admitting `0x80` here as one byte would leave the cursor
        // four bytes inside a field instead of past it, and the arm would stop
        // being a subset of `Varint` — which is the one relation
        // `the_trailer_sink_arms_are_nested` is for.
        TrailerSink::Flag => match seg.get(*p) {
            Some(&f) if f < 0x80 => {
                *p += 1;
                true
            }
            _ => false,
        },
        TrailerSink::Varint => read_varint(seg, p).is_some(),
    };
    if !state_ok {
        *p = save;
        return false;
    }
    true
}

/// The return plumbing, with the generated destructor's `5E <n> <g> 4B`
/// sub-object trailer optionally wedged between the `29` return and the function
/// tail. `eat_return_plumbing` cannot do that (D1 hand-rolls the same split for
/// the same reason), so the branch/close/return run is walked here and the tail is
/// shared.
fn eat_body_end(seg: &[u8], p: &mut usize, depth: usize, fail: &mut Fail) -> bool {
    let start = *p;
    if eat_body_end_inner(seg, p, depth) {
        return true;
    }
    // Structural, and reported as such: the byte the tail opens on is the only
    // thing carried, so a plumbing refusal cannot masquerade as a value construct.
    fail.note(start, FailKind::Plumbing);
    false
}

fn eat_body_end_inner(seg: &[u8], p: &mut usize, depth: usize) -> bool {
    let save = *p;
    if eat_return_plumbing(seg, p, false, depth).is_ok() {
        return true;
    }
    *p = save;
    // 3A <label> · scope closes · 29 <label>
    if !eat_byte(seg, p, 0x3A) {
        return false;
    }
    match read_token_var(seg, *p) {
        Some((_, w)) => *p += w,
        None => return false,
    }
    for d in (BODY_SCOPE_DEPTH..=depth).rev() {
        eat_opt_stmt_marker(seg, p);
        if !eat(seg, p, &[0x54, d as u8]) {
            return false;
        }
    }
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x29) {
        return false;
    }
    match read_token_var(seg, *p) {
        Some((_, w)) => *p += w,
        None => return false,
    }
    // `5E <n> <g>` then the statement end.
    if !eat_byte(seg, p, 0x5E) {
        return false;
    }
    if seg.get(*p).is_none() {
        return false;
    }
    *p += 1;
    match seg.get(*p) {
        Some(&g) if TRAILER_FLAGS.iter().any(|&(_, s)| s == g) => *p += 1,
        _ => return false,
    }
    if !eat_byte(seg, p, 0x4B) {
        return false;
    }
    eat_fn_tail(seg, p).is_ok()
}

/// Which forms [`eat_form_value`] has a production for. The rest are UNMEASURED,
/// and `docs/IL_CALL_IN_EXPR.md` §14 reports them as such rather than as 0 %:
/// a completeness figure for a grammar that was never written is not a
/// measurement.
fn form_is_measured(form: CallForm) -> bool {
    matches!(
        form,
        CallForm::RecvLoad
            | CallForm::RecvDeref
            | CallForm::RecvField
            | CallForm::RecvFieldZero
            | CallForm::RecvObject
            | CallForm::RecvCall
            | CallForm::RecvIntrinsic(_)
            | CallForm::Chained
            | CallForm::NestedCall
            | CallForm::DataAddr
            | CallForm::DataRead
    )
}

/// One or more value tokens: the `form` under test, plus the operand vocabulary
/// the modeled leaves already carry. Nothing else — a `30` load, a `9B` sret bind,
/// a comparison, a ternary, an intrinsic call all stop the sequence and therefore
/// fail the body, which is what makes a `-whole` count mean "only `form` is
/// missing".
fn eat_value_seq(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    let mut n = 0;
    loop {
        let save = *p;
        let msave = fail.mark();
        if eat_form_value(seg, p, adm.form, adm, fail) {
            n += 1;
            continue;
        }
        *p = save;
        fail.rewind(msave);
        // The second admitted construct, for the both-handled measure. Tried after
        // the form and on the same restored cursor, so neither can leave the other
        // mid-token.
        if eat_blocker_value(seg, p, adm, fail) {
            n += 1;
            continue;
        }
        *p = save;
        fail.rewind(msave);
        match seg.get(*p) {
            Some(&0xB9) => {
                *p += 1;
                match read_token_var(seg, *p) {
                    Some((_, w)) => *p += w,
                    None => {
                        fail.note(*p, FailKind::Value);
                        return false;
                    }
                }
                if !eat_admitted_type(seg, p, adm) {
                    fail.note(*p, FailKind::Type);
                    return false;
                }
            }
            Some(&0x33) => {
                let Some((tag, kind, _, _)) = read_type(seg, *p + 1) else {
                    fail.note(*p + 1, FailKind::Type);
                    return false;
                };
                *p += 1;
                if !eat_admitted_type(seg, p, adm) {
                    fail.note(*p, FailKind::Type);
                    return false;
                }
                if !eat_literal_payload(seg, p, tag, kind) {
                    fail.note(*p, FailKind::Value);
                    return false;
                }
            }
            Some(&0x2C) => {
                *p += 1;
                if !eat_admitted_type(seg, p, adm) {
                    fail.note(*p, FailKind::Type);
                    return false;
                }
                if seg.get(*p).is_none() {
                    fail.note(*p, FailKind::Value);
                    return false;
                }
                *p += 1;
            }
            Some(&0x02) | Some(&0x03) | Some(&0x04) => *p += 1,
            _ => {
                // Not a hard failure: the sequence simply ends here, and the caller
                // decides whether what follows is a statement end. The note is
                // still taken, because if the caller *does* fail this is the token
                // that stopped it.
                fail.note(*p, FailKind::Value);
                return n > 0;
            }
        }
        n += 1;
    }
}

/// Consume one value of the **second** admitted construct, for the both-handled
/// measure. An empty grant set admits nothing, which is D2's behaviour exactly.
///
/// Only the variants [`blocker_is_measured`] returns true for have a production
/// here; the rest are UNMEASURED and never reach this function, because
/// [`mark_whole`] does not run the second pass for them.
fn eat_blocker_value(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    let save = *p;
    let msave = fail.mark();
    for &b in adm.granted() {
        *p = save;
        fail.rewind(msave);
        if eat_one_blocker_value(seg, p, b, adm, fail) {
            return true;
        }
    }
    *p = save;
    fail.rewind(msave);
    false
}

fn eat_one_blocker_value(
    seg: &[u8],
    p: &mut usize,
    blk: Blocker,
    adm: Admit,
    fail: &mut Fail,
) -> bool {
    match blk {
        Blocker::Call(f) if form_is_measured(f) => eat_form_value(seg, p, f, adm, fail),
        // `30 <TYPE>` — an indirect load. The TYPE must itself be admitted, so
        // "loads are handled" does not smuggle in "and every type is too".
        Blocker::DerefLoad => eat_byte(seg, p, 0x30) && eat_admitted_type(seg, p, adm),
        // A bare CALL token over values already on the stack.
        Blocker::PlainCall => {
            seg.get(*p) == Some(&0xBD) && eat_call_and_args(seg, p, adm, fail)
        }
        Blocker::ChainBind => eat_chain_link(seg, p, adm, fail),
        // A bare one-byte binary operator over two values already on the stack —
        // the same production `eat_value_seq` gives `02`/`03`/`04` unconditionally,
        // granted rather than free because the emitter has no lowering for it. See
        // [`BARE_BINARY_OPS`] for why the set is one byte and what admits another.
        Blocker::Op(b) if BARE_BINARY_OPS.contains(&b) => eat_byte(seg, p, b),
        // The `33 <int> <k>` literal that feeds it is already in the modeled
        // vocabulary; the add itself is the missing token.
        Blocker::OffAdd => match seg.get(*p) {
            Some(&0x27) => {
                *p += 1;
                eat_type(seg, p)
            }
            Some(&0x28) => {
                *p += 1;
                eat(seg, p, &[0x00, 0x00])
            }
            _ => false,
        },
        // A `Type` blocker is admitted inside `eat_admitted_type`, not here: it
        // widens the type test rather than adding a token.
        _ => false,
    }
}

/// The **class-pair descriptor** `66 <n> <ref>×n` that every 2113–2119 intrinsic
/// call carries: `n` type references naming the classes the adjustment is
/// between.
///
/// **Each ref is a plain LEB128 id — not a fixed two bytes, and not a
/// [`read_token_var`] token.** This is the rung's most consequential measurement
/// and it was found the way GAPS.md §6 says these things get found: by a residue
/// that made no sense. The first D2 scan spread 17,757 functions over 197
/// `op-0xNN` buckets, and every witness was a *generated destructor* whose
/// descriptor read `66 02 fb 8a 01 e0 91 01` — two **three**-byte refs. Stepping
/// four bytes lands two bytes short, inside the second ref, and the walk then
/// reads a payload byte as an opcode.
///
/// Why LEB and not the other two candidates, with the witnesses that separate
/// them:
///
/// * **Fixed 2 bytes** is what the small probes show (`66 02 92 20 93 20`,
///   `66 02 ad 20 a8 20`) and it is what `shapes.rs` implements. It cannot be
///   right: `src/App.cpp` and `src/lazer/game/Game.cpp` — TUs with tens of
///   thousands of types — carry `fb 8a 01`, `e0 91 01`, `ff ff 01`, `d3 80 02`,
///   `cd a5 02`. Under a fixed-2 reading the byte after the descriptor would be a
///   type-id continuation byte, and it is not.
/// * **A `read_token_var` token** would take `fb 8a 01 …` as *four* bytes (byte 1
///   has bit 7 set), which oversteps by one and desyncs the other way. `92 20`
///   agrees with LEB and with tokens, so only the wide witnesses separate them.
/// * **LEB128** reads `92 20` as 2 bytes and `fb 8a 01` as 3, and lands exactly on
///   the following `55` argument terminator at every witness in both TU sizes.
///   That marker is what pins it, the same way the `41`/`55`/`4C 4B` markers pin
///   [`read_type`]'s width.
///
/// **`shapes.rs` still steps a fixed four bytes**, in `try_parse_base_member_load`
/// and in D1's `try_parse_empty_dtor_delegation`, and this rung does not touch it:
/// changing it would change *acceptance*, which D2 must not do. The consequence is
/// measured and reported in `docs/IL_CALL_IN_EXPR.md` §14.3 — D1 is refusing
/// textbook base-delegating destructors in every large TU for want of this one
/// step, and the `recv-intrinsic-this-adjust-whole` count is the size of that.
/// Returns the **ref count** on success, so that each caller applies its own
/// acceptance rule to it while this function owns only the *encoding*. That split
/// is why the fix could be shared: `try_parse_base_member_load` bounds `n` at 3
/// and D1 requires exactly 2, and neither constraint belongs in a decoder.
pub(super) fn eat_class_descriptor(seg: &[u8], p: &mut usize) -> Option<u8> {
    if !eat_byte(seg, p, 0x66) {
        return None;
    }
    let &n = seg.get(*p)?;
    *p += 1;
    for _ in 0..n {
        if !eat_leb(seg, p) {
            return None;
        }
    }
    Some(n)
}

/// One LEB128 id: bytes with bit 7 set continue, the first without it ends.
fn eat_leb(seg: &[u8], p: &mut usize) -> bool {
    for _ in 0..5 {
        match seg.get(*p) {
            Some(&b) => {
                *p += 1;
                if b & 0x80 == 0 {
                    return true;
                }
            }
            None => return false,
        }
    }
    false
}

/// A `33` LITERAL, type triple and payload, for **any** literal type.
///
/// This is the one place the walk needed a rule `readers::read_varint` does not
/// have, and it was not optional: `read_varint` models the 1-byte signed short
/// form and a 4-byte escape, which is right for `int` and **wrong for three
/// literal classes that occur constantly in real code**. Getting it wrong is not
/// a refusal, it is a *desync* — the walk lands mid-payload, reads a value byte as
/// an opcode, and the census files the function under whatever byte that happened
/// to be. MEASURED: the first D2 scan, with `read_varint` here, spread 17,757
/// functions over **198 distinct `op-0xNN` buckets** at 80–300 each — a flat
/// distribution over almost the whole byte range, which is the fingerprint of
/// reading payload as vocabulary. With the rules below the same scan concentrates
/// them.
///
/// The rules are `docs/IL_CAST_CONVERT.md` §3.1/§3.2, measured there against
/// `work/cast/k9.cpp` and `k11.cpp`:
///
/// * **A real literal** — kind class `0xA`, the value kind + 5 (`86 4A 40` for
///   `float`, `88 8A 41` for `double`) — carries **8 raw IEEE-754 binary64 bytes,
///   little-endian, then a u16 LE target size**. A `float` literal is stored as a
///   `double` too and differs only in the triple and that trailing size.
/// * **An integer literal** is a signed byte, unless it is the escape `0x80`, and
///   then the payload is **8 bytes for tag `0x88`** (`long long`) and 4 otherwise
///   — including for the 1- and 2-byte types, whose escapes are still 4.
fn eat_literal(seg: &[u8], p: &mut usize) -> bool {
    let Some((tag, kind, _, w)) = read_type(seg, *p) else {
        return false;
    };
    *p += w;
    eat_literal_payload(seg, p, tag, kind)
}

/// The payload half of [`eat_literal`], for callers that have already consumed
/// (and class-checked) the type triple.
fn eat_literal_payload(seg: &[u8], p: &mut usize, tag: u8, kind: u8) -> bool {
    // A real literal: 8 IEEE bytes + a u16 LE size. Not a varint at all.
    if kind & 0x0F == 0xA {
        *p += 10;
        return *p <= seg.len();
    }
    match seg.get(*p) {
        Some(&0x80) => {
            *p += 1 + if tag == 0x88 { 8 } else { 4 };
            *p <= seg.len()
        }
        Some(_) => {
            *p += 1;
            true
        }
        None => false,
    }
}

/// A TYPE naming a 4-byte integer or a pointer — the two classes the modeled
/// leaves lower (`ValueClass` in `shapes.rs`) — **plus** the one further class
/// `adm.also` admits, when the second blocker is a type class.
///
/// A float, a narrow integer or an aggregate here is a second missing production,
/// so it refuses and the body is not counted complete. That refusal is exactly
/// what a `Blocker::Type` names, and widening this function by one class is what
/// "if both were handled" means for such a pair.
fn eat_admitted_type(seg: &[u8], p: &mut usize, adm: Admit) -> bool {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) => {
            let int4 = matches!(kind & 0x0F, 0x1 | 0x2) && (kind >> 4) == 4 && (tag & 0x0F) == 0x6;
            let ptr = matches!(kind & 0x0F, 0x3 | 0x4);
            let extra = adm.admits_type(seg, *p);
            if int4 || ptr || extra {
                *p += w;
                true
            } else {
                false
            }
        }
        None => false,
    }
}

/// Consume exactly one value of `form`. `false` — cursor position unspecified,
/// the caller discards it — for anything else.
///
/// The forms with **no** production here (`RecvOther`, `Chained`, `Intrinsic`,
/// `Other`, `Op`, `Eof`) are UNMEASURED rather than measured-zero,
/// and `docs/IL_CALL_IN_EXPR.md` §14 says so in the table: reporting 0 %
/// completeness for a form whose grammar was never written would be a claim, and
/// the honest statement is that the number does not exist.
fn eat_form_value(seg: &[u8], p: &mut usize, form: CallForm, adm: Admit, fail: &mut Fail) -> bool {
    match form {
        CallForm::RecvLoad
        | CallForm::RecvDeref
        | CallForm::RecvField
        | CallForm::RecvFieldZero
        | CallForm::RecvObject
        | CallForm::RecvCall
        | CallForm::RecvIntrinsic(_) => eat_member_call(seg, p, form, adm, fail),
        CallForm::Chained => eat_chained_call(seg, p, adm, fail),
        CallForm::NestedCall => eat_plain_call(seg, p, adm, fail),
        CallForm::DataAddr => eat_data_designator(seg, p, false, fail),
        CallForm::DataRead => eat_data_designator(seg, p, true, fail),
        _ => false,
    }
}

/// `26 <method> <receiver of `form`> 99 <T> 00 BD <ret> <conv> <id> (<arg> 55 <T>)* 4C`
/// — exactly **one** method symbol, so a chain cannot slip through.
fn eat_member_call(seg: &[u8], p: &mut usize, form: CallForm, adm: Admit, fail: &mut Fail) -> bool {
    let head = *p;
    if !eat_byte(seg, p, 0x26) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    if !eat_receiver(seg, p, form, adm, fail) {
        // The `26` really did open a call-shaped production and its receiver is a
        // designator this form is not. Naming it needs the whole production, so the
        // note carries the `26` and [`Fail::blocker`] re-walks from there.
        fail.note(*p, FailKind::Receiver(head));
        return false;
    }
    // The member bind. `99` is DIRECT dispatch by construction: virtual dispatch
    // is opcode `67` with a `9A` bind (§3), which is what licenses reading this
    // as a branch to a named callee at all.
    if !eat_byte(seg, p, 0x99) {
        fail.note(*p, FailKind::Value);
        return false;
    }
    if !eat_type(seg, p) {
        fail.note(*p, FailKind::Type);
        return false;
    }
    if !eat_byte(seg, p, 0x00) {
        fail.note(*p, FailKind::Value);
        return false;
    }
    eat_call_and_args(seg, p, adm, fail)
}

/// `26 <fn> BD … (<arg> 55 <T>)* 4C`.
fn eat_plain_call(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    if !eat_byte(seg, p, 0x26) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    eat_call_and_args(seg, p, adm, fail)
}

/// The `BD` CALL token and its explicit-argument region. Each argument must be an
/// **already-modeled** int-like operand stream, so a body needing a second new
/// production is not counted complete.
fn eat_call_and_args(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    if !eat_byte(seg, p, 0xBD) {
        fail.note(*p, FailKind::Value);
        return false;
    }
    if !eat_type(seg, p) {
        fail.note(*p, FailKind::Type);
        return false;
    }
    // cdecl only: the one calling convention every captured member call carries.
    if !eat_byte(seg, p, 0x00) || read_varint(seg, p).is_none() {
        fail.note(*p, FailKind::Value);
        return false;
    }
    // Everything from here to the `4C` is an argument region. Entered and left in
    // exactly one place so [`Fail::args_depth`] is balanced on every path, including
    // the ones a speculative parse abandons.
    fail.args_depth += 1;
    let ok = eat_call_args_region(seg, p, adm, fail);
    fail.args_depth -= 1;
    ok
}

/// The `(<arg> 55 <TYPE>)* 4C` region of [`eat_call_and_args`].
fn eat_call_args_region(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    loop {
        match seg.get(*p) {
            Some(&0x4C) => {
                *p += 1;
                return true;
            }
            Some(_) => {
                // Deliberately still D2's argument grammar in the **first** pass: an
                // already-modeled int-like operand stream, nothing else. Widening it
                // unconditionally would move the `-whole` counts §14.1 and §15.4 are
                // stated in, and this rung's acceptance check is that those counts
                // are recoverable by summing.
                //
                // The **second** pass admits the second blocker here too, and it has
                // to: `gO.Set(p->Get())` blocks on a receiver form inside an argument
                // region, and a both-handled measure that refused arguments would
                // report `-more` for a body that only ever needed the two. Inert when
                // the grant set is empty, which is every first pass.
                let save = *p;
                let msave = fail.mark();
                let nested = adm.n > 0
                    && (eat_form_value(seg, p, adm.form, adm, fail) || {
                        *p = save;
                        fail.rewind(msave);
                        eat_blocker_value(seg, p, adm, fail)
                    });
                if !nested {
                    *p = save;
                    fail.rewind(msave);
                    if !eat_int_operands(seg, p, Vocab::CallArg, adm, fail) {
                        return false;
                    }
                }
                if !eat_byte(seg, p, 0x55) {
                    fail.note(*p, FailKind::Value);
                    return false;
                }
                // **The `55` call-end annotation is the FORMAL's declared type,
                // and the emitter gates it** — `shapes::calls::eat_call_args`
                // requires `eat_int_like_or_ptr4` here and refuses the call
                // otherwise, on the measured ground that widening one of the two
                // positions without the other admits no real call site at all.
                // This read `eat_type` — *any* TYPE — until #139, which made the
                // measure WIDER than its emitter at this position and so
                // manufactured completeness: a body whose argument is annotated
                // `float`, `short`, `long long` or `char *` was counted `-whole`
                // when the shipping path refuses it outright. That is the
                // direction §9.13's E4 records as invisible to every gate, and
                // it is 2,925 of 13,500 enumerated operand streams.
                if eat_int_like_or_ptr4(seg, p).is_none() {
                    fail.note(*p, FailKind::Type);
                    return false;
                }
            }
            None => {
                fail.note(*p, FailKind::Value);
                return false;
            }
        }
    }
}

/// **Which shipping position this operand stream is the measure OF.**
///
/// The rule board **#139** establishes: *when a census key names a construct,
/// the measure's acceptance vocabulary must match the emitter's.* A measure
/// narrower than its emitter does not merely under-count — the greedy walker
/// charges the difference as a granted construct, so the key prints a second
/// construct that was never a blocker and `-whole{k}` is one too high.
///
/// So the vocabulary is no longer implicit. Each variant **names the shipping
/// locator it mirrors**, and
/// `tests::a_measure_and_its_emitter_admit_the_same_types` holds the pair to it
/// over the whole enumerated TYPE space rather than over a witness list.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Vocab {
    /// A **call argument**. Mirrors `shapes::calls::eat_call_args` — which all
    /// three shipping member-call productions (`mcall_tail`, `mcall_chain`,
    /// `mcall_cmp`) route arguments through — position by position:
    ///
    /// | position | the emitter's gate | reached via |
    /// |---|---|---|
    /// | LOAD / LIT operand TYPE | [`eat_operand_type`] | `parse_expr` |
    /// | the `02`/`03`/`04` operators | the same three | `parse_expr` |
    /// | pointer / one-byte-unsigned stream rules | [`Stream::emitter_would_refuse`] | `parse_expr` |
    /// | the `55` call-end annotation | [`eat_int_like_or_ptr4`] | `eat_call_args` |
    /// | the `2C` class-preserving conversion | `eat_value_type` | `parse_expr` |
    ///
    /// **The `2C` row was very nearly left out on a number measured one scan
    /// too early**, and that is worth recording. Sized on the BASE tree the
    /// conversion looked worth nothing — 0 bodies carried a `…-then-op-0x2C`
    /// key — so it was documented as a deliberate, bounded omission. Both
    /// halves of that were wrong. The key is spelled `…-then-convert`
    /// ([`super::expr_opcode_name`] names the byte, which is the whole point of
    /// that table), and, more to the point, the number was a number about a
    /// tree that no longer existed: repairing the operand TYPE let the walk
    /// reach *past* the pointer it used to stop at, and the conversion behind it
    /// went **829 → 13,325 bodies and 26 → 1,144 emitted in one scan**.
    ///
    /// A residue sized before the repair that exposes it is not sized. The rule
    /// this whole section exists to state — a census key must not name a
    /// construct its emitter does not refuse — applies to the construct the
    /// repair *reveals* exactly as it applies to the one the repair removes.
    CallArg,
    /// An **intrinsic-call receiver**. Nothing in the intrinsic family is
    /// lowered at all (`docs/IL_INTRINSIC_CALL.md`), so there is no emitter for
    /// this position and no correspondence to hold it to. It keeps D2's
    /// original int-only vocabulary — left UNMEASURED rather than widened to
    /// match a production that does not exist, which is the same honesty gate
    /// [`form_is_measured`] applies.
    IntrinsicRecv,
}

/// What the operand run has seen so far, for the **stream-level** refusals that
/// no single operand can reach.
///
/// `parse_expr` carries three of them — a pointer may be moved but not computed
/// on, a one-byte-unsigned value may be moved but neither computed on nor mixed
/// with a width-4 one — and each is a place the measure was WIDER than its
/// emitter. That direction manufactures phantom *completeness*: a row reads
/// `-whole` and the shipping path refuses it outright. It is invisible to
/// `census/gate disagreement` for the reason §9.13's E4 gives, and it was 1,053
/// of 13,500 enumerated operand streams.
struct Stream {
    ptr: bool,
    int1u: bool,
    wide: bool,
    arith: bool,
    /// The [`ValueClass`] on top of the operand stack, or `None` before the
    /// first operand — the top of [`Stream::cstack`], kept as its own field
    /// because the `2C` arm reads it on every token and the stack is only
    /// meaningful while `cstack_ok`.
    last: Option<ValueClass>,
    /// **The operand stack's CLASSES** (`lane w-convert`, board **#701**), one
    /// for one with `parse_expr_classed`'s `cstack`. The emitter's
    /// `expr-ptr-arith` is precise where this model holds — *was an arithmetic
    /// operator applied to a value that was a pointer AT THAT MOMENT* — and a
    /// measure still asking the coarse `ptr && arith` would be NARROWER than its
    /// emitter, which #139 records as manufacturing phantom rungs.
    cstack: Vec<ValueClass>,
    cstack_ok: bool,
    ptr_arith_exact: bool,
}

impl Default for Stream {
    fn default() -> Stream {
        Stream {
            ptr: false,
            int1u: false,
            wide: false,
            arith: false,
            last: None,
            cstack: Vec::new(),
            // The model starts believing itself; every token it cannot follow
            // clears it and the coarse flags take over.
            cstack_ok: true,
            ptr_arith_exact: false,
        }
    }
}

impl Stream {
    /// Push one operand's class.
    fn push_class(&mut self, c: ValueClass) {
        self.cstack.push(c);
        self.last = Some(c);
    }

    /// Fold one binary operator, exactly as `parse_expr_classed`'s
    /// `fold_binary!` does: pop two, indict the operator if either was a
    /// pointer, push the result's class.
    fn fold_binary(&mut self) {
        match (self.cstack.pop(), self.cstack.pop()) {
            (Some(r), Some(l)) => {
                let ptr = r == ValueClass::Ptr4 || l == ValueClass::Ptr4;
                if ptr {
                    self.ptr_arith_exact = true;
                }
                let out = if ptr { ValueClass::Ptr4 } else { ValueClass::Int4 };
                self.cstack.push(out);
                self.last = Some(out);
            }
            _ => self.cstack_ok = false,
        }
    }

    /// Fold the **byte-offset add** `27 <TYPE>`, exactly as
    /// `parse_expr_classed`'s `0x27` arm does: pop the offset the preceding
    /// `33` pushed and the address under it, push the re-typed address as a
    /// [`ValueClass::Ptr4`].
    ///
    /// It deliberately does **not** set [`Stream::ptr_arith_exact`], and that
    /// asymmetry with [`Stream::fold_binary`] is the whole reason `27` and `02`
    /// are different opcodes: `27` is the *byte*-offset add and carries its
    /// scaling already, while `02` over a pointer is scaled by the pointee
    /// width and costs a real multiply. The emitter's guard refuses one and not
    /// the other (`w-c1`, `expr.rs`'s `0x27` arm), so a measure that indicted
    /// this fold would be **wider** than its emitter — the direction #139
    /// records as manufacturing phantom completeness.
    fn fold_off_add(&mut self) {
        match (self.cstack.pop(), self.cstack.pop()) {
            (Some(_off), Some(_base)) => self.push_class(ValueClass::Ptr4),
            // On a stream the postfix model could not follow, the coarse flags
            // take over — `fold_binary`'s own failure arm, one for one.
            _ => self.cstack_ok = false,
        }
    }

    /// Replace the class on top — an accepted `2C`.
    fn retop_class(&mut self, c: ValueClass) {
        if let Some(top) = self.cstack.last_mut() {
            *top = c;
        }
        self.last = Some(c);
    }
}

impl Stream {
    /// Exactly `parse_expr`'s two guards, in the same order and on the same
    /// conditions. Only meaningful where the position HAS an emitter.
    fn emitter_would_refuse(&self) -> bool {
        self.which_refusal_opt().is_some()
    }

    fn which_refusal_opt(&self) -> Option<Structural> {
        // Precise where the class stack held (#701); the coarse flag otherwise.
        // Same expression as `parse_expr_classed`'s, and it must stay that way:
        // a measure narrower than its emitter manufactures phantom rungs and a
        // wider one manufactures phantom completeness.
        let ptr_arith = if self.cstack_ok { self.ptr_arith_exact } else { self.ptr && self.arith };
        if ptr_arith {
            Some(Structural::PtrArith)
        } else if self.int1u && (self.arith || self.wide) {
            Some(Structural::Int1uMisuse)
        } else {
            None
        }
    }

    fn which_refusal(&self) -> Structural {
        self.which_refusal_opt().expect("only called when a refusal was found")
    }
}

/// One or more `B9 <tok> <TYPE>` / `33 <TYPE> <k>` / `02|03|04` tokens — the
/// operand vocabulary the position's own emitter accepts (see [`Vocab`]), and
/// nothing else.
fn eat_int_operands(seg: &[u8], p: &mut usize, v: Vocab, adm: Admit, fail: &mut Fail) -> bool {
    let mut n = 0;
    let mut st = Stream::default();
    // The run is over; report it the way the position's own emitter would.
    macro_rules! finish {
        () => {
            if n > 0 && v == Vocab::CallArg && st.emitter_would_refuse() {
                // Name the CONSTRUCT, never the byte the run stopped in front
                // of — see [`Structural::PtrArith`] for what filing this as
                // `op-0x55` cost and how it was caught.
                //
                // Forced, and at the furthest position the run reached: a
                // stream refusal is a property of the WHOLE operand run, not of
                // one offset, and [`Fail::note`] gives ties to the first note —
                // which is the `FailKind::Value` this very loop records one line
                // before returning. That tie is exactly how `op-0x55` survived
                // the first attempt at naming this.
                fail.note_forcing(fail.at.max(*p), FailKind::Struct(st.which_refusal()));
                false
            } else {
                n > 0
            }
        };
    }
    loop {
        match seg.get(*p) {
            Some(&0xB9) => {
                let save = *p;
                *p += 1;
                let Some((_, w)) = read_token_var(seg, *p) else {
                    fail.note(*p, FailKind::Value);
                    return false;
                };
                *p += w;
                if !eat_operand_or_admitted(seg, p, v, adm, &mut st) {
                    fail.note(*p, FailKind::Type);
                    *p = save;
                    return finish!();
                }
            }
            Some(&0x33) => {
                let save = *p;
                *p += 1;
                if !eat_operand_or_admitted(seg, p, v, adm, &mut st) {
                    fail.note(*p, FailKind::Type);
                    *p = save;
                    return finish!();
                }
                if read_varint(seg, p).is_none() {
                    fail.note(*p, FailKind::Value);
                    return false;
                }
            }
            Some(&0x02) | Some(&0x03) | Some(&0x04) => {
                st.arith = true;
                st.fold_binary();
                *p += 1;
            }
            // **A `2C` CLASS-PRESERVING CONVERSION**, exactly as `parse_expr`
            // admits it: the target must be the class the value already has, a
            // register-to-register identity c2 emits nothing for, and the
            // trailing varint must literally be `0`.
            //
            // This arm did not exist until the enumerated guard's own output
            // demanded it. #139's repair let the walk reach *past* the pointer
            // TYPE it used to stop at, and what it then stopped on was this —
            // `…-then-convert` went 829 -> 13,325 bodies and 26 -> 1,144
            // emitted in a single scan. Left unrepaired that is #139 all over
            // again one token later: a census key naming `convert` as the
            // second construct for bodies whose emitter does not refuse a
            // conversion at all.
            Some(&0x2C) if v == Vocab::CallArg => {
                let start = *p;
                let mut probe = *p + 1;
                let Some(cls) = st.last else {
                    fail.note(start, FailKind::Value);
                    return finish!();
                };
                if eat_value_type(seg, &mut probe, cls) {
                    // Class-preserving: the class on the stack is unchanged.
                } else if let Some(got) = eat_reinterpret_type(seg, &mut probe, cls) {
                    // **The width-4 REINTERPRET** (`lane w-convert`, board
                    // **#700**), mirrored from `parse_expr_classed`'s `2C` arm
                    // one token for one token — including the `saw_ptr` line,
                    // which is `st.ptr` here. A measure that admitted the
                    // reinterpret without indicting the value would be *wider*
                    // than the emitter, which is the direction #139 records as
                    // manufacturing phantom completeness; the enumerated guard
                    // below is what caught this arm being narrower.
                    st.retop_class(got);
                    if got == ValueClass::Ptr4 {
                        st.ptr = true;
                    }
                } else {
                    fail.note(*p + 1, FailKind::Type);
                    return finish!();
                }
                if !eat_byte(seg, &mut probe, 0x00) {
                    fail.note(probe, FailKind::Value);
                    return finish!();
                }
                *p = probe;
            }
            // **THE BYTE-OFFSET ADD** — lane `w-3475`, board **#3475**, and it
            // is #139's shape for the third time at this position.
            //
            // Lane `w-c1` (Phase 1 slice C1) promoted `parse_expr_classed`'s
            // `0x27` arm from an env-gated sink to a graded, **default-on**
            // construct. `Vocab::CallArg`'s contract is that this measure
            // mirrors `shapes::calls::eat_call_args`, whose operand streams are
            // `parse_expr`'s on the default path — `off_add_arg_sink` is OFF
            // unless `C2RS_SINK_OFF_ADD_ARG` is set and `eat_sym_addr_arg`
            // refuses an offset run by construction. The measure did not
            // follow, so from `w-c1` until here it was NARROWER than its
            // emitter at this byte, and the greedy walker charged the
            // difference as a granted `Blocker::OffAdd` — a census key naming
            // `off-add` as the second blocker for bodies whose emitter decodes
            // it. That is the *phantom rung* direction, and #139 measured what
            // it costs: a 14x estimate miss.
            //
            // Mirrored token for token, and the three things it does NOT do are
            // as load-bearing as the three it does:
            //
            // * **`0x28` is NOT admitted.** `parse_expr` has no `28` arm at all
            //   (it refuses under `expr-op-0x28`), so admitting the subscript
            //   offset add here would be the *wider* error in the same line
            //   that fixes the narrower one. `Fail::blocker` still names both
            //   bytes `off-add`, which is correct — it names the construct —
            //   and a `28` therefore still stops this run and is still granted
            //   through `eat_one_blocker_value`.
            // * **`Vocab::IntrinsicRecv` is NOT widened.** Nothing in the
            //   intrinsic family is lowered, so that position has no emitter
            //   and no correspondence to hold it to (see [`Vocab`]). Widening
            //   it would be inventing one.
            // * **`ptr_arith_exact` is NOT set** — see [`Stream::fold_off_add`].
            //
            // Gated on the same named decision point the emitter reads, so
            // `C2RS_OFF_ADD=off` restores the pre-`w-c1` parser **and** the
            // pre-`w-3475` measure together. The two must not be settable
            // apart: a configuration in which the correspondence is false is a
            // configuration in which every `-whole{k}` figure is wrong, and
            // nothing would say so.
            Some(&0x27) if v == Vocab::CallArg && off_add_admitted() => {
                let start = *p;
                let mut probe = *p + 1;
                let Some((tag, kind, _, w)) = read_type(seg, probe) else {
                    fail.note(probe, FailKind::Type);
                    return finish!();
                };
                if !super::shapes::designator::is_ptr_any(tag, kind) {
                    // The emitter refuses this one too, under its own key
                    // (`expr-off-add-ptee`) and on this same predicate. The
                    // refused CONSTRUCT is still the off-add, so it is named at
                    // the `27` and not at the TYPE behind it: a `FailKind::Type`
                    // here would file the body under a granted *type class*,
                    // which `eat_admitted_type` would then admit at every OTHER
                    // operand position while this one kept refusing — a grant
                    // that cannot complete the body it was granted for.
                    fail.note(start, FailKind::Value);
                    return finish!();
                }
                probe += w;
                st.ptr = true;
                st.fold_off_add();
                *p = probe;
            }
            _ => {
                fail.note(*p, FailKind::Value);
                return finish!();
            }
        }
        n += 1;
    }
}

/// The position's own operand TYPE vocabulary, widened by the one type class a
/// `Blocker::Type` second blocker admits, recording the class on `st`. The
/// widening is inert in the first pass (the grant set is empty).
fn eat_operand_or_admitted(
    seg: &[u8],
    p: &mut usize,
    v: Vocab,
    adm: Admit,
    st: &mut Stream,
) -> bool {
    match v {
        Vocab::CallArg => {
            let save = *p;
            if let Some(c) = eat_operand_type(seg, p) {
                st.ptr |= c == ValueClass::Ptr4;
                st.int1u |= c == ValueClass::Int1u;
                st.wide |= c != ValueClass::Int1u;
                st.push_class(c);
                return true;
            }
            *p = save;
        }
        Vocab::IntrinsicRecv => {
            if eat_int_like(seg, p) {
                st.wide = true;
                // `IntrinsicRecv` reads int-likeness without classifying, so
                // there is no class to push; the model stops here.
                st.cstack_ok = false;
                return true;
            }
        }
    }
    if adm.admits_type(seg, *p) {
        if let Some((_, _, _, w)) = read_type(seg, *p) {
            *p += w;
            // A granted class is a HYPOTHETICAL production, so no emitter rule
            // can be derived for it. Counted as a width-4-shaped value, which is
            // what leaves the stream guards inert for it rather than inventing a
            // refusal the shipping path has never been asked about.
            st.wide = true;
            // A granted class has no [`ValueClass`], so nothing can be pushed
            // for it and the stack model stops being able to follow the stream.
            st.cstack_ok = false;
            return true;
        }
    }
    false
}

/// One **chain link**: the bind that applies a method to the value already on the
/// operand stack, and the call it opens. `99 <TYPE> 00 BD <ret> <conv> <id>
/// (<arg> 55 <T>)* 4C`.
///
/// `99` is DIRECT dispatch by construction (§3), which is what licenses reading a
/// link as one more `bl` to a named callee.
fn eat_chain_link(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    if !eat_byte(seg, p, 0x99) || !eat_type(seg, p) || !eat_byte(seg, p, 0x00) {
        return false;
    }
    eat_call_and_args(seg, p, adm, fail)
}

/// A **member-call chain**, giving [`CallForm::Chained`] the production D2 left
/// unwritten — 8,000 functions whose completeness was UNMEASURED rather than zero
/// (§14.6).
///
/// The shape is the LIFO one the module header describes: `h` method symbols push,
/// then one receiver, then one bind-and-call per method, innermost first:
/// `(26 <tok>){h} <receiver> (99 <T> 00 BD … 4C){links}`. `h` and `links` differ by
/// one exactly when the receiver is itself a named object, because then the last
/// push *is* the receiver — the same ambiguity [`walk`] refuses to resolve forward,
/// resolved here by trying the designators and falling back.
fn eat_chained_call(seg: &[u8], p: &mut usize, adm: Admit, fail: &mut Fail) -> bool {
    // The head run of symbol pushes that are not callees. A `26` immediately
    // followed by `BD` is a callee push, not a method (§4).
    let mut heads: Vec<usize> = Vec::new();
    while seg.get(*p) == Some(&0x26) {
        let save = *p;
        let mut q = *p + 1;
        match read_token_var(seg, q) {
            Some((_, w)) => q += w,
            None => break,
        }
        if seg.get(q) == Some(&0xBD) {
            break;
        }
        heads.push(save);
        *p = q;
        if heads.len() > MAX_CHAIN {
            return false;
        }
    }
    if heads.len() < 2 {
        return false;
    }
    // The innermost receiver, tried over every designator this module names. A
    // named-object base is not tried here: it was already consumed as the last
    // head push, which is what the `heads.len() - 1` fallback covers.
    const BASES: [CallForm; 5] = [
        CallForm::RecvLoad,
        CallForm::RecvDeref,
        CallForm::RecvFieldZero,
        CallForm::RecvField,
        CallForm::RecvCall,
    ];
    let after_heads = *p;
    let msave = fail.mark();
    let mut links = 0;
    for base in BASES {
        *p = after_heads;
        fail.rewind(msave);
        if eat_receiver(seg, p, base, adm, fail) {
            links = heads.len();
            break;
        }
    }
    if links == 0 {
        // The last push was the receiver: rewind to it and take it as the object.
        *p = heads[heads.len() - 1];
        fail.rewind(msave);
        if !eat_receiver(seg, p, CallForm::RecvObject, adm, fail) {
            return false;
        }
        links = heads.len() - 1;
        if links < 2 {
            return false;
        }
    }
    for _ in 0..links {
        if !eat_chain_link(seg, p, adm, fail) {
            return false;
        }
    }
    true
}

/// Bound on a chain's length. The deepest chain in the D2 sample is four links.
const MAX_CHAIN: usize = 8;

/// The receiver designator of each named form, and only that form's.
fn eat_receiver(seg: &[u8], p: &mut usize, form: CallForm, adm: Admit, fail: &mut Fail) -> bool {
    let ok = match form {
        CallForm::RecvLoad => eat_ptr_load(seg, p),
        CallForm::RecvDeref => {
            eat_ptr_load(seg, p) && eat_opt_off_add(seg, p) && eat_byte(seg, p, 0x30) && eat_type(seg, p)
        }
        CallForm::RecvField => eat_ptr_load(seg, p) && eat_off_add_of(seg, p, false),
        CallForm::RecvFieldZero => eat_ptr_load(seg, p) && eat_off_add_of(seg, p, true),
        CallForm::RecvObject => {
            eat_byte(seg, p, 0x26)
                && match read_token_var(seg, *p) {
                    Some((_, w)) => {
                        *p += w;
                        true
                    }
                    None => false,
                }
        }
        CallForm::RecvCall => eat_plain_call(seg, p, adm, fail),
        CallForm::RecvIntrinsic(sel) => eat_intrinsic_receiver(seg, p, sel, adm, fail),
        _ => false,
    };
    ok && eat_opt_convert(seg, p)
}

/// `B9 <tok> <TYPE>` where the TYPE is a **data pointer** (`kind`'s low nibble 3
/// — `docs/IL_LOAD_TYPES.md` §1). A receiver is a pointer or a reference, and
/// those are byte-identical (§3); an int-typed value here is not this production.
fn eat_ptr_load(seg: &[u8], p: &mut usize) -> bool {
    if !eat_byte(seg, p, 0xB9) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    match read_type(seg, *p) {
        Some((_, kind, _, tw)) if kind & 0x0F == 0x3 => {
            *p += tw;
            true
        }
        _ => false,
    }
}

fn eat_type(seg: &[u8], p: &mut usize) -> bool {
    match read_type(seg, *p) {
        Some((_, _, _, w)) => {
            *p += w;
            true
        }
        None => false,
    }
}

/// `2C <TYPE> <byte>`, optional. A pointer→pointer convert and a cv-strip both
/// emit nothing (`docs/IL_LOAD_TYPES.md` §3), which is why a receiver may carry
/// one and still be the same value.
fn eat_opt_convert(seg: &[u8], p: &mut usize) -> bool {
    if seg.get(*p) != Some(&0x2C) {
        return true;
    }
    let save = *p;
    *p += 1;
    if !eat_type(seg, p) || seg.get(*p).is_none() {
        *p = save;
        return false;
    }
    *p += 1;
    true
}

/// `33 <int-like> <k>` then `27 <TYPE>` or `28 00 00` — the byte-offset add, with
/// `k` required to be zero or nonzero as `want_zero` says.
fn eat_off_add_of(seg: &[u8], p: &mut usize, want_zero: bool) -> bool {
    if !eat_byte(seg, p, 0x33) || !eat_int_like(seg, p) {
        return false;
    }
    match read_varint(seg, p) {
        Some(k) if (k == 0) == want_zero => {}
        _ => return false,
    }
    match seg.get(*p) {
        Some(&0x27) => {
            *p += 1;
            eat_type(seg, p)
        }
        Some(&0x28) => {
            *p += 1;
            eat(seg, p, &[0x00, 0x00])
        }
        _ => false,
    }
}

/// The byte-offset add at any offset, optional — used by the data designators,
/// where the offset is a member position and not a codegen decision.
fn eat_opt_off_add(seg: &[u8], p: &mut usize) -> bool {
    if seg.get(*p) != Some(&0x33) {
        return true;
    }
    let save = *p;
    if eat_off_add_of(seg, p, true) {
        return true;
    }
    *p = save;
    eat_off_add_of(seg, p, false)
}

/// The class-layout intrinsic receiver:
/// `33 <int> <sel> 40 <TYPE> 66 <n> <2n> (<arg> 55 <T>)* 4C`.
///
/// The selector is required to be the one the classifier reported, and the
/// arguments are stepped as `<int-operands or a pointer load> 55 <TYPE>` — the
/// 2113 form's three arguments (`docs/IL_CAST_CONVERT.md`) being a selector
/// terminator, the adjust offset, and the object pointer.
fn eat_intrinsic_receiver(seg: &[u8], p: &mut usize, sel: i32, adm: Admit, fail: &mut Fail) -> bool {
    let Some(found) = intrinsic_selector(seg, *p) else {
        return false;
    };
    if found != sel {
        return false;
    }
    *p += 1;
    if !eat_int_like(seg, p) || read_varint(seg, p).is_none() {
        return false;
    }
    // `40 <TYPE>` — no trailing field.
    if !eat_byte(seg, p, 0x40) || !eat_type(seg, p) {
        return false;
    }
    if eat_class_descriptor(seg, p).is_none() {
        return false;
    }
    loop {
        match seg.get(*p) {
            Some(&0x4C) => {
                *p += 1;
                return true;
            }
            Some(&0x55) => {
                *p += 1;
                if !eat_type(seg, p) {
                    return false;
                }
            }
            Some(&0xB9) => {
                if !eat_ptr_load(seg, p) {
                    return false;
                }
            }
            Some(_) => {
                if !eat_int_operands(seg, p, Vocab::IntrinsicRecv, adm, fail) {
                    return false;
                }
            }
            None => return false,
        }
    }
}

/// A data symbol used as an address (`want_load` false) or read (`true`):
/// `26 <sym> [2C …] [33 <k> 27|28 …] [2C …] [30 <TYPE> [2C …]]`.
///
/// Counts itself on `fail` when it succeeds — see [`Fail::syms`] for why the count
/// lives there and why every caller that rewinds the cursor must rewind it too.
fn eat_data_designator(seg: &[u8], p: &mut usize, want_load: bool, fail: &mut Fail) -> bool {
    if !eat_byte(seg, p, 0x26) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    // A `26 <tok>` **immediately followed by `BD`** is a CALLEE push, not a value:
    // `uc("hi")` opens `26 <uc> BD …`, and this greedy designator takes that push
    // because nothing else in the grammar will (`docs/IL_CALL_IN_EXPR.md` §16.2 —
    // it is why the second blocker is named `plain-call` and not `op-0xBD`).
    // Consuming it is what makes the `-whole` counts what §14/§16 state, so that is
    // unchanged; **counting** it as a materialized data symbol is not, and would
    // report every single-string call as two. Same test `eat_chained_call` uses to
    // tell a callee push from a method push.
    let is_callee_push = seg.get(*p) == Some(&0xBD);
    if !eat_opt_convert(seg, p) || !eat_opt_off_add(seg, p) || !eat_opt_convert(seg, p) {
        return false;
    }
    let loaded = eat_byte(seg, p, 0x30) && eat_type(seg, p);
    if loaded != want_load {
        return false;
    }
    if !eat_opt_convert(seg, p) {
        return false;
    }
    if !is_callee_push && fail.args_depth > 0 {
        fail.syms += 1;
    }
    true
}

/// **Test hook: run the completeness measure's own argument region.**
///
/// Exists so `tests::a_measure_and_its_emitter_admit_the_same_types` can drive
/// the measure through the *same bytes* it drives the emitter through, instead
/// of asserting a property of a shared helper — a guard that reads both sides
/// through one locator cannot see the two sides drifting apart, which is the
/// only failure it is there to catch.
#[cfg(test)]
pub(crate) fn measure_admits_call_args(seg: &[u8]) -> bool {
    let mut p = 0usize;
    let mut fail = Fail::new();
    let adm = Admit::bare(CallForm::RecvLoad);
    eat_call_args_region(seg, &mut p, adm, &mut fail) && p == seg.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail};
    use crate::func::test_fixtures::{free_fn, NO_LOCALS};

    /// **THE MEASURE-vs-EMITTER GUARD** — `docs/ROADMAP.md` §9.14, board #139.
    ///
    /// > When a census key names a construct, the measure's acceptance
    /// > vocabulary must match the emitter's. A measure narrower than the
    /// > emitter manufactures phantom rungs; a measure wider than the emitter
    /// > manufactures phantom *completeness*.
    ///
    /// Neither direction is visible to any gate this project has.
    /// `census/gate disagreement` compares *the census* with *the port* on
    /// acceptance, and this measure decides neither: [`mark_whole`] is
    /// diagnostic, its `Err` stays an `Err`. So a mismatched vocabulary emits
    /// no wrong byte, disagrees with nothing, and shows up only as a ranking
    /// that sends a lane after a construct that was never a blocker — which is
    /// what #139 cost, a 14× estimate miss.
    ///
    /// **What makes this a control rather than a restatement.** Both sides are
    /// driven end to end through their own public entry points over the same
    /// bytes:
    ///
    /// * the emitter — `shapes::calls::eat_call_args`, which is what all three
    ///   shipping member-call productions (`mcall_tail`, `mcall_chain`,
    ///   `mcall_cmp`) route arguments through;
    /// * the measure — [`measure_admits_call_args`], the completeness walker's
    ///   own argument region.
    ///
    /// A test that instead asserted a property of the shared locator would pass
    /// no matter how far `parse_expr` drifted from it.
    ///
    /// **The domain is enumerated, not sampled.** Every `(tag, kind)` in
    /// `0x80..=0xFF × 0x00..=0xFF` that [`read_type`] parses at all — a witness
    /// list would have missed exactly the class that was wrong, because the
    /// class that was wrong (`Ptr4`) had witnesses on the emitter side and none
    /// on the measure side.
    ///
    /// **This test FAILS on the tree before #139** (measured: 5,312 TYPEs
    /// disagree, every one of them a pointer class the emitter admits and the
    /// measure refused), and passes after.
    #[test]
    fn a_measure_and_its_emitter_admit_the_same_types() {
        use crate::func::body::shapes::calls::eat_call_args;
        // A fixed, known-good formal type at the `55` call-end, so the ONLY
        // thing varying between cases is the operand's own TYPE. `86 41 74` is
        // plain `int`, transcribed from a live capture.
        const INT: [u8; 3] = [0x86, 0x41, 0x74];
        let mut disagreements: Vec<(u8, u8, bool, bool)> = Vec::new();
        let mut enumerated = 0usize;
        for tag in 0x80u8..=0xFF {
            for kind in 0x00u8..=0xFF {
                // Build the TYPE, then ask `read_type` how wide it really is;
                // the wide-prefix and aggregate forms are longer than three
                // bytes and are skipped rather than guessed at.
                let buf = [tag, kind, 0x74, 0x00, 0x00, 0x00];
                let Some((_, _, _, w)) = read_type(&buf, 0) else {
                    continue;
                };
                if w != 3 {
                    continue;
                }
                enumerated += 1;
                let ty = &buf[..3];
                // `B9 <tok:2> <TYPE>` · `55 <int>` · `4C` — one argument,
                // one LOAD, nothing else in the stream.
                let mut seg = vec![0xB9, 0x01, 0x02];
                seg.extend_from_slice(ty);
                seg.push(0x55);
                seg.extend_from_slice(&INT);
                seg.push(0x4C);

                let mut p = 0usize;
                let emitter = eat_call_args(&seg, &mut p).is_ok() && p == seg.len();
                let measure = super::measure_admits_call_args(&seg);
                if emitter != measure {
                    disagreements.push((tag, kind, emitter, measure));
                }
            }
        }
        assert!(
            enumerated > 1000,
            "the domain collapsed to {enumerated} TYPEs — the guard would pass vacuously"
        );
        assert!(
            disagreements.is_empty(),
            "{} of {enumerated} enumerated TYPEs are accepted by one side and refused by the \
             other. A census key that names a construct is only true if the measure and the \
             emitter admit the same operands (roadmap #139). First 12: {:02X?}",
            disagreements.len(),
            &disagreements[..disagreements.len().min(12)]
        );
    }

    /// One captured TYPE per class the argument grammar can meet. Transcribed
    /// from live captures (the spellings are quoted in [`super::super::readers`]
    /// and `docs/IL_TYPE_TAGS.md` §2), and every entry is checked to parse
    /// before it is used, so a mistyped triple fails loudly instead of silently
    /// shrinking the domain.
    const CLASS_WITNESSES: &[(&str, &[u8])] = &[
        ("int", &[0x86, 0x41, 0x74]),
        ("unsigned", &[0x86, 0x42, 0x75]),
        ("long", &[0x86, 0x41, 0x12]),
        ("unsigned long", &[0x86, 0x42, 0x22]),
        ("const int", &[0xA6, 0x41, 0x84, 0x20]),
        ("volatile int", &[0x96, 0x41, 0x80, 0x20]),
        ("int *", &[0x86, 0x43, 0x74]),
        ("int * const", &[0xA6, 0x43, 0x8F, 0x20]),
        ("int * volatile", &[0x96, 0x43, 0x80, 0x20]),
        ("char *", &[0x82, 0x43, 0xF0, 0x08]),
        ("bool / unsigned char", &[0x82, 0x12, 0x74]),
        ("short", &[0x84, 0x41, 0x74]),
        ("long long", &[0x88, 0x41, 0x74]),
        ("float", &[0x86, 0x45, 0x40]),
        ("double", &[0x88, 0x85, 0x41]),
    ];

    /// **The guard, over operand STREAMS rather than single operands.**
    ///
    /// `parse_expr` carries three refusals that no single-operand case can
    /// reach — the pointer-arithmetic guard, the one-byte-unsigned arithmetic
    /// guard, and the one-byte-unsigned mixing guard — and each of them is a
    /// place the measure could be *wider* than the emitter, which manufactures
    /// phantom completeness: a row that reads takeable and is not. That
    /// direction is the one §9.13's E4 records as invisible to
    /// `census/gate disagreement`, because the port would emit wrong bytes
    /// rather than refuse.
    ///
    /// The domain is the full cross of two operand classes × the four operator
    /// shapes the grammar admits (none, `02` add, `03` sub, `04` mul) × the
    /// formal type at the `55`. Small enough to enumerate exhaustively, wide
    /// enough that every stream-level guard is reached by construction.
    #[test]
    fn a_measure_and_its_emitter_admit_the_same_operand_streams() {
        use crate::func::body::shapes::calls::eat_call_args;
        for (name, ty) in CLASS_WITNESSES {
            let (_, _, _, w) = read_type(ty, 0)
                .unwrap_or_else(|| panic!("witness TYPE for {name} does not parse: {ty:02X?}"));
            assert_eq!(w, ty.len(), "witness TYPE for {name} has trailing bytes");
        }
        let mut disagreements: Vec<String> = Vec::new();
        let mut cases = 0usize;
        // `None` is "one operand and nothing else"; `02`/`03`/`04` append a
        // second operand and the arithmetic that reaches the stream guards;
        // `2C` appends a CONVERSION whose target is the second class, which
        // reaches the class-preserving rule and nothing else does.
        //
        // **`0xFF` is not a token — it is the CONVERT-THEN-ARITHMETIC shape**
        // (`lane w-convert`, board #701), and it is here because without it this
        // guard's domain could not separate the coarse `ptr && arith` from the
        // precise class-stack model: every arithmetic case above puts a raw LOAD
        // on both sides, where the two models agree by construction. A guard
        // whose domain cannot express the drift it is looking for is the
        // `sweep.d/10-int-chains.py` failure one layer up (#688), and this axis
        // is the fix. `(int)p + b` is the whole of the workload population the
        // precise model converts.
        for (n1, t1) in CLASS_WITNESSES {
            for op in [
                None,
                Some(0x02u8),
                Some(0x03),
                Some(0x04),
                Some(0x2C),
                Some(0xFF),
                Some(0x27),
            ] {
                for (n2, t2) in CLASS_WITNESSES {
                    for (n5, t5) in CLASS_WITNESSES {
                        cases += 1;
                        let mut seg = vec![0xB9, 0x01, 0x02];
                        seg.extend_from_slice(t1);
                        match op {
                            Some(0x2C) => {
                                // `2C <target TYPE> 00`
                                seg.push(0x2C);
                                seg.extend_from_slice(t2);
                                seg.push(0x00);
                            }
                            Some(0xFF) => {
                                // `2C <target TYPE> 00` then a second `int`
                                // operand and an ADD: the conversion decides
                                // the class the operator sees.
                                seg.push(0x2C);
                                seg.extend_from_slice(t2);
                                seg.push(0x00);
                                seg.extend_from_slice(&[0xB9, 0x01, 0x03, 0x86, 0x41, 0x74, 0x02]);
                            }
                            // **`0x27` is the BYTE-OFFSET ADD axis** (lane
                            // `w-3475`, board **#3475**), and it is the axis
                            // whose absence let the `Vocab::CallArg`
                            // correspondence break for a whole lane without a
                            // single test going red. This guard's operator axis
                            // was `02`/`03`/`04`/`2C` — **arithmetic and
                            // conversion only** — so when `w-c1` widened
                            // `parse_expr` to admit `27`, nothing here could
                            // see it. The claim the tree carried in three
                            // places (`expr.rs`'s comment, board **#364**,
                            // ROADMAP §9.17.6) was that widening `parse_expr`
                            // *"obliges `mcall::eat_int_operands` to widen in
                            // lockstep, or the correspondence guard goes red"*;
                            // `w-c1` measured that nothing went red and scored
                            // its own prereg **P10 a MISS**. This line is why
                            // it did not: **the guard everybody cited had no
                            // opcode axis for the opcode in question.**
                            //
                            // `33 <t2> 08 · 27 <t2>` — the designator step
                            // `&t->s.k` spells, with `t2` in both positions so
                            // the enumeration sweeps the accepting (pointer)
                            // and refusing (non-pointer) halves of
                            // `designator::is_ptr_any` on both sides at once.
                            //
                            // **It goes RED on the tree before `w-3475`** and
                            // green after, which is the only thing that makes
                            // it a control rather than a restatement: the whole
                            // module is diagnostic, so the byte judge cannot go
                            // red for this defect and never could.
                            Some(0x27) => {
                                seg.push(0x33);
                                seg.extend_from_slice(t2);
                                seg.push(0x08);
                                seg.push(0x27);
                                seg.extend_from_slice(t2);
                            }
                            Some(o) => {
                                seg.extend_from_slice(&[0xB9, 0x01, 0x03]);
                                seg.extend_from_slice(t2);
                                seg.push(o);
                            }
                            None => {}
                        }
                        seg.push(0x55);
                        seg.extend_from_slice(t5);
                        seg.push(0x4C);

                        let mut p = 0usize;
                        let emitter = eat_call_args(&seg, &mut p).is_ok() && p == seg.len();
                        let measure = super::measure_admits_call_args(&seg);
                        if emitter != measure {
                            disagreements.push(format!(
                                "[{n1}] {} [{n2}] :: 55 [{n5}] -> emitter={emitter} measure={measure}",
                                match op {
                                    None => "·",
                                    Some(0x02) => "+",
                                    Some(0x03) => "-",
                                    Some(0x04) => "*",
                                    Some(0xFF) => "convert-then-add-int, to",
                                    Some(0x27) => "byte-offset-add by",
                                    _ => "convert-to",
                                }
                            ));
                        }
                    }
                }
            }
        }
        assert!(cases > 3000, "domain collapsed to {cases} streams");
        assert!(
            disagreements.is_empty(),
            "{} of {cases} enumerated operand streams are accepted by one side and refused by \
             the other (roadmap #139). First 12:\n  {}",
            disagreements.len(),
            disagreements[..disagreements.len().min(12)].join("\n  ")
        );
    }

    /// The guard above is only worth its runtime if it can go red, so this
    /// states the failure it was built from **positively**: a width-4 pointer
    /// TYPE at a call-argument operand is admitted by the emitter, and before
    /// #139 the measure refused it and charged a `Blocker::Type(Ptr)` grant for
    /// the difference.
    ///
    /// Captured spelling, from `int h1(int*); int f(int* p){return h1(p);}`:
    /// `… B9 p 86 43 f4 08 · 55 86 43 f4 08 · 4C`.
    #[test]
    fn a_pointer_call_argument_was_never_a_second_construct() {
        // `86 43 74` — a 4-byte pointer, the class W22 admitted at this
        // position and the measure did not.
        let seg = vec![
            0xB9, 0x01, 0x02, 0x86, 0x43, 0x74, 0x55, 0x86, 0x43, 0x74, 0x4C,
        ];
        let mut p = 0usize;
        assert!(
            crate::func::body::shapes::calls::eat_call_args(&seg, &mut p).is_ok(),
            "the EMITTER has admitted a pointer argument since W22"
        );
        assert!(
            super::measure_admits_call_args(&seg),
            "…and so must the measure, or the census key prints `-then-type-ptr` \
             for a construct that was never a blocker"
        );
        // The negative half: a class NEITHER side admits stays refused on both,
        // so the repair widened the measure to the emitter and not past it.
        // `86 45 74` is `float`, which no argument production lowers.
        let seg = vec![
            0xB9, 0x01, 0x02, 0x86, 0x45, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C,
        ];
        let mut p = 0usize;
        assert!(crate::func::body::shapes::calls::eat_call_args(&seg, &mut p).is_err());
        assert!(!super::measure_admits_call_args(&seg));
    }

    /// The same statement for the **byte-offset add** — lane `w-3475`, board
    /// **#3475**, and the reason the guard above needed a `0x27` axis.
    ///
    /// Since lane `w-c1` the emitter has decoded `27 <ptr TYPE>` by default
    /// (`expr.rs`'s arm, `off_add_admitted`, `C2RS_OFF_ADD`). Until this lane
    /// the measure did not, so the greedy walker charged the difference as a
    /// granted `Blocker::OffAdd` and printed `…-then-off-add` as the second
    /// blocker for a body whose emitter takes that token — **#139's shape**,
    /// and it stood for the whole interval between the two lanes with nothing
    /// in the tree able to say so.
    ///
    /// The spelling is the designator step `&t->s.k` compiles to, quoted from
    /// `expr.rs`'s own arm: `B9 t · 33 <int> k · 27 <T*>`.
    ///
    /// **Three cells, and the two negatives are what keep the repair from
    /// overshooting into the phantom-completeness direction:**
    ///
    /// 1. `27` over a **pointer** TYPE — both sides admit;
    /// 2. `27` over a **non-pointer** TYPE — both sides refuse
    ///    (`designator::is_ptr_any`, the same predicate the four leaf consumers
    ///    and the emitter's arm use; the emitter names it `expr-off-add-ptee`);
    /// 3. **`28`, the subscript offset add — both sides refuse.** `parse_expr`
    ///    has no `28` arm, so a measure that admitted it would be *wider* than
    ///    its emitter in the same line that fixed the narrower error. This cell
    ///    is the one that would catch a repair reaching for the neighbouring
    ///    byte because `Fail::blocker` gives the two the same construct name.
    #[test]
    fn a_byte_offset_add_call_argument_was_never_a_second_construct() {
        // `86 43 74` — a 4-byte pointer; `86 41 74` — plain `int`.
        let arg = |off: &[u8]| {
            let mut seg = vec![0xB9, 0x01, 0x02, 0x86, 0x43, 0x74];
            seg.extend_from_slice(&[0x33, 0x86, 0x41, 0x74, 0x08]);
            seg.extend_from_slice(off);
            seg.extend_from_slice(&[0x55, 0x86, 0x43, 0x74, 0x4C]);
            seg
        };
        let both = |seg: &[u8]| {
            let mut p = 0usize;
            let emitter = crate::func::body::shapes::calls::eat_call_args(seg, &mut p).is_ok()
                && p == seg.len();
            (emitter, super::measure_admits_call_args(seg))
        };

        // 1. the pointer TYPE — admitted by both.
        assert_eq!(
            both(&arg(&[0x27, 0x86, 0x43, 0x74])),
            (true, true),
            "the EMITTER has decoded `27 <ptr>` since w-c1; the measure must too, or the \
             census key prints `-then-off-add` for a construct that was never a blocker"
        );
        // 2. a non-pointer TYPE — refused by both, so the repair widened the
        //    measure TO the emitter and not past it.
        assert_eq!(
            both(&arg(&[0x27, 0x86, 0x41, 0x74])),
            (false, false),
            "`is_ptr_any` fences both sides; the emitter calls this `expr-off-add-ptee`"
        );
        // 3. `28` — refused by both. `parse_expr` has no arm for it.
        assert_eq!(
            both(&arg(&[0x28, 0x00, 0x00])),
            (false, false),
            "the subscript offset add is NOT the byte-offset add; admitting it here would \
             make the measure wider than its emitter"
        );
    }

    /// One pinned body's census [`Block`].
    ///
    /// **`formals-marker:mid` here is a property of the EXCERPT, not a refusal.**
    /// Three of the constants below — [`DATA_ADDR`], [`DATA_ADDR_INDEX`] and
    /// [`DATA_ADDR_PTR_ARG`] — now parse past their argument region, because WR1
    /// admitted a named data symbol's address as a call argument, and they then
    /// reach `parse_params`, which these excerpts have no `46` for. Nothing had
    /// ever parsed far enough to want one. The construct they used to be the
    /// witnesses for is graded end to end instead, against a real obj, by
    /// `fixtures/cpp/wr1_sym_addr.cpp` (18/18 in class, whole obj byte-exact) and
    /// `wr1_sym_addr_neg.cpp` (0/13, `Port=NotImplemented`) — which is a stronger
    /// grading than a pinned key, not a weaker one.
    fn probe_block(seg: &[u8]) -> Block {
        let s = free_fn(seg);
        parse_segment_detail(&s, NO_LOCALS).unwrap_err()
    }

    /// The census key a body that now parses past its arguments reports in an
    /// excerpt with no formals region. Named once so the three sites that expect
    /// it read as one statement rather than three copies of a magic string.
    const PARSED_PAST_THE_ARGUMENTS: &str = "formals-marker:mid";

    // Every byte array below is transcribed verbatim from a live-toolchain
    // capture of a controlled probe (`c2rs census <probe> --keep-il <dir>`), one
    // function per constant, at the fixture profile `/Ox /GS- /c`. The probe
    // sources and the capture commands are in `docs/IL_CALL_IN_EXPR.md` §14.4;
    // none of these is hand-assembled, which is the whole point — the field
    // widths of this production were guessed wrong twice before a capture settled
    // them (`docs/IL_INTRINSIC_CALL.md`).

    /// `int r_load(Obj* p) { int x; x = p->Get(); return x; }`
    /// The receiver is a `B9` load of a pointer formal, and the whole body is one
    /// member call — so this is the `-whole` witness too.
    const RECV_LOAD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x04, 0x0A, 0x26, 0xE4, 0x09, 0xB9, 0x01, 0x0A, 0x86, 0x43,
        0x81, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05,
        0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x04, 0x0A, 0x86, 0x41, 0x74,
        0x41, 0x86, 0x41, 0x74, 0x3A, 0x03, 0x0A, 0x54, 0x02, 0x29, 0x03, 0x0A, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00,
    ];
    /// `int r_thru(Wrap* w) { int x; x = w->p->Get(); return x; }` — the receiver
    /// is read from memory (`33 0 27 <T> 30 <T>`).
    const RECV_DEREF: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x0C, 0x0A, 0x26, 0xE4, 0x09, 0xB9, 0x09, 0x0A, 0x86, 0x43,
        0x8D, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x90, 0x20, 0x30, 0x86, 0x43,
        0x81, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05,
        0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x0C, 0x0A, 0x86, 0x41, 0x74,
        0x41, 0x86, 0x41, 0x74, 0x3A, 0x0B, 0x0A, 0x54, 0x02, 0x29, 0x0B, 0x0A, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00,
    ];
    /// `int r_sub(Wrap* w) { int x; x = w->o.Get(); return x; }` — the receiver is
    /// a sub-object *address*: the same bytes as [`RECV_DEREF`] minus the `30`.
    /// The pair is what separates the two forms, and getting it wrong is the
    /// `&s->m` / `s->m` trap `try_parse_ptr_identity_leaf` already documents.
    const RECV_FIELD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x10, 0x0A, 0x26, 0xE4, 0x09, 0xB9, 0x0D, 0x0A, 0x86, 0x43,
        0x8D, 0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x90, 0x20, 0x99, 0x86, 0x43,
        0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x32,
        0x86, 0x41, 0x74, 0x4B, 0xB9, 0x10, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0x0F, 0x0A, 0x54, 0x02, 0x29, 0x0F, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int r_named() { int x; x = gO.Get(); return x; }` — `26 <sym> 2C <ptr> 00`.
    const RECV_OBJECT: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x13, 0x0A, 0x26, 0xE4, 0x09, 0x26, 0xF8, 0x09, 0x2C, 0xA6,
        0x43, 0x84, 0x20, 0x00, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00,
        0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x13, 0x0A, 0x86,
        0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x12, 0x0A, 0x54, 0x02, 0x29, 0x12, 0x0A, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int c_callrecv() { int x; x = GN()->Val(); return x; }` — the receiver is
    /// a plain call's result. **One** method is stacked: the `26 <GN>` is a callee
    /// push, which is why the head-run count excludes a `26` followed by `BD`.
    const RECV_CALL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x41, 0x0A, 0x26, 0xE5, 0x09, 0x26, 0x12, 0x0A, 0xBD, 0x86,
        0x43, 0x9A, 0x20, 0x00, 0x80, 0x20, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x9D, 0x20,
        0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x1D, 0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41,
        0x74, 0x4B, 0xB9, 0x41, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x40, 0x0A,
        0x54, 0x02, 0x29, 0x40, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int i_base(D1* d) { int x; x = d->Bm(); return x; }`, `Bm` inherited from
    /// `B1` — the receiver is intrinsic 2113 `this-adjust` at offset 0.
    const RECV_INTRINSIC: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x49, 0x0A, 0x26, 0xFA, 0x09, 0x33, 0x86, 0x41, 0x74, 0x80,
        0x41, 0x08, 0x00, 0x00, 0x40, 0xA6, 0x43, 0xA9, 0x20, 0x66, 0x02, 0xAD, 0x20, 0xA8, 0x20,
        0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0xB9, 0x46,
        0x0A, 0x86, 0x43, 0xB0, 0x20, 0x55, 0x86, 0x43, 0xB0, 0x20, 0x4C, 0x99, 0x86, 0x43, 0xAA,
        0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x2A, 0x10, 0x00, 0x00, 0x4C, 0x32, 0x86,
        0x41, 0x74, 0x4B, 0xB9, 0x49, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x48,
        0x0A, 0x54, 0x02, 0x29, 0x48, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int c_two(N* p) { int x; x = p->Next()->Val(); return x; }` — **two**
    /// stacked method symbols before one receiver. The innermost bind's receiver
    /// is the `B9` load, so a receiver-only classification would file this as
    /// `recv-load`; the head run is what separates it.
    const CHAINED: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x3A, 0x0A, 0x26, 0xE5, 0x09, 0x26, 0xE4, 0x09, 0xB9, 0x37,
        0x0A, 0x86, 0x43, 0x9A, 0x20, 0x99, 0x86, 0x43, 0x9C, 0x20, 0x00, 0xBD, 0x86, 0x43, 0x9A,
        0x20, 0x00, 0x80, 0x1C, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x9D, 0x20, 0x00, 0xBD,
        0x86, 0x41, 0x74, 0x00, 0x80, 0x1D, 0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B,
        0xB9, 0x3A, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x39, 0x0A, 0x54, 0x02,
        0x29, 0x39, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int n_call(int a) { int x; x = g1(g1(a)); return x; }` — the production the
    /// bucket is *named* after, and §7.3 measured at 0.2 % of it.
    const NESTED_CALL: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x28, 0x0A, 0x26, 0xFA, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00,
        0x80, 0x11, 0x10, 0x00, 0x00, 0x26, 0xFA, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x11,
        0x10, 0x00, 0x00, 0xB9, 0x25, 0x0A, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x55,
        0x86, 0x41, 0x74, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x28, 0x0A, 0x86, 0x41, 0x74,
        0x41, 0x86, 0x41, 0x74, 0x3A, 0x27, 0x0A, 0x54, 0x02, 0x29, 0x27, 0x0A, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00,
    ];
    /// `int a_str() { int x; x = uc("hi"); return x; }` — a string literal's
    /// address decayed into an argument. No `99` anywhere: not a call at all on
    /// this `26`, which is the ~18 % of the bucket §6 measured.
    const DATA_ADDR: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x1A, 0x0A, 0x26, 0xFC, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00,
        0x80, 0x15, 0x10, 0x00, 0x00, 0x26, 0x1B, 0x0A, 0x2C, 0x86, 0x43, 0x93, 0x20, 0x00, 0x55,
        0x86, 0x43, 0x93, 0x20, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x1A, 0x0A, 0x86, 0x41,
        0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x19, 0x0A, 0x54, 0x02, 0x29, 0x19, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int a_addr() { int x; x = ui(&gA[2]); return x; }` — the same construct
    /// through a scaled subscript (`33 <long> 8` then `28 00 00`).
    const DATA_ADDR_INDEX: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x21, 0x0A, 0x26, 0xFE, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00,
        0x80, 0x18, 0x10, 0x00, 0x00, 0x26, 0xFF, 0x09, 0x33, 0x86, 0x41, 0x12, 0x08, 0x28, 0x00,
        0x00, 0x55, 0x86, 0x43, 0xF4, 0x08, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x21, 0x0A,
        0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x20, 0x0A, 0x54, 0x02, 0x29, 0x20, 0x0A,
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `void h1() { d1("aa", "bb"); }` — **two** string-literal addresses in one
    /// call, which D5 measured to be what this whole row actually is
    /// (`docs/IL_CALL_IN_EXPR.md` §17): 2,730 of 2,730 symbol-carrying plain calls
    /// in a 40-TU workload sample pass two, and none passes one.
    ///
    /// It is the discriminating witness for the count, in two directions at once:
    /// three `26` pushes reach [`eat_data_designator`] here (the callee `d1` and the
    /// two literals) and the census must report **2**, so a rule that counted every
    /// designator would say three and a rule that counted arguments-only without the
    /// callee test would say two by luck on [`DATA_ADDR`] and three here.
    const DATA_ADDR_TWO_SYMS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x03, 0x10,
        0x00, 0x00, 0x26, 0xE8, 0x09, 0x2C, 0x86, 0x43, 0x81, 0x20, 0x00, 0x55, 0x86, 0x43, 0x81,
        0x20, 0x26, 0xE9, 0x09, 0x2C, 0x86, 0x43, 0x81, 0x20, 0x00, 0x55, 0x86, 0x43, 0x81, 0x20,
        0x4C, 0x4B, 0x3A, 0xE7, 0x09, 0x54, 0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];
    /// `void m_ps(T* p) { u3(p, "cc"); }` — one string address beside a **pointer**
    /// formal, so this is the k = 2 row (`then-plain-call-and-type-ptr-whole2`) and
    /// the matcher runs `body_matches` three times over it. The witness for the
    /// rewind rule: without restoring the count on each abandoned attempt this reads
    /// more than one symbol.
    const DATA_ADDR_PTR_ARG: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE6, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x05, 0x10,
        0x00, 0x00, 0x26, 0xEA, 0x09, 0x2C, 0x86, 0x43, 0x83, 0x20, 0x00, 0x55, 0x86, 0x43, 0x83,
        0x20, 0xB9, 0xE7, 0x09, 0x86, 0x43, 0x81, 0x20, 0x55, 0x86, 0x43, 0x81, 0x20, 0x4C, 0x4B,
        0x3A, 0xE9, 0x09, 0x54, 0x02, 0x29, 0xE9, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// **WDA's discriminating pair, half one.**
    /// `struct O { int M1(const char*); }; int m1(O* p) { int x; x = p->M1("hi"); return x; }`
    /// — a member call on a pointer formal with **one** string-literal argument. The
    /// form is [`CallForm::RecvLoad`] and the data symbol arrives as the *second*
    /// blocker, so this is the population D5's `counts_syms` predicate measured and
    /// then discarded: 10,540 workload functions in one key with no count on it.
    const RECV_LOAD_ONE_SYM: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF2, 0x09, 0x26, 0xE5, 0x09, 0xB9, 0xEF, 0x09, 0x86, 0x43,
        0x81, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x06,
        0x10, 0x00, 0x00, 0x26, 0xF3, 0x09, 0x2C, 0x86, 0x43, 0x83, 0x20, 0x00, 0x55, 0x86, 0x43,
        0x83, 0x20, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0xF2, 0x09, 0x86, 0x41, 0x74, 0x41,
        0x86, 0x41, 0x74, 0x3A, 0xF1, 0x09, 0x54, 0x02, 0x29, 0xF1, 0x09, 0x4F, 0x12, 0x47, 0x54,
        0x01, 0x54, 0x00,
    ];
    /// **WDA's discriminating pair, half two.**
    /// `int M2(const char*, const char*); … x = p->M2("aa","bb");` — the same member
    /// call with **two** string-literal arguments, so c2 must derive the second
    /// address from the first by `.rdata` pool-offset difference
    /// (`docs/IL_CALL_IN_EXPR.md` §17.3 (a)).
    ///
    /// The pair differs by exactly one `26 <tok> 2C <T> 00 55 <T>` argument group and
    /// by **nothing else** — same receiver form, same second blocker, same grant
    /// count, same `-whole` suffix. Before WDA both printed the identical key. That
    /// is the whole finding: the census could not tell a takeable rung from a phase
    /// inside its own largest `-whole` member-call row.
    const RECV_LOAD_TWO_SYMS: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF7, 0x09, 0x26, 0xE8, 0x09, 0xB9, 0xF4, 0x09, 0x86, 0x43,
        0x81, 0x20, 0x99, 0x86, 0x43, 0x88, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x08,
        0x10, 0x00, 0x00, 0x26, 0xF8, 0x09, 0x2C, 0x86, 0x43, 0x83, 0x20, 0x00, 0x55, 0x86, 0x43,
        0x83, 0x20, 0x26, 0xF9, 0x09, 0x2C, 0x86, 0x43, 0x83, 0x20, 0x00, 0x55, 0x86, 0x43, 0x83,
        0x20, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0xF7, 0x09, 0x86, 0x41, 0x74, 0x41, 0x86,
        0x41, 0x74, 0x3A, 0xF6, 0x09, 0x54, 0x02, 0x29, 0xF6, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];
    /// `int d_read() { int x; x = gO.m; return x; }` — a global object's member
    /// read, §7.1's 2.5 %.
    const DATA_READ: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x24, 0x0A, 0x26, 0xF8, 0x09, 0x33, 0x86, 0x41, 0x74, 0x00,
        0x27, 0x86, 0x43, 0xF4, 0x08, 0x30, 0x86, 0x41, 0x74, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9,
        0x24, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x23, 0x0A, 0x54, 0x02, 0x29,
        0x23, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `int r_arg(Obj* p) { int x; x = g1(p->Get()); return x; }` — the *same*
    /// construct as [`RECV_LOAD`] reached through a call-argument region instead of
    /// an assignment right-hand side. It must land in the same bucket (the
    /// mis-attribution gate) and must NOT be whole-body complete (two calls).
    const RECV_LOAD_IN_ARG: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x17, 0x0A, 0x26, 0xFA, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00,
        0x80, 0x11, 0x10, 0x00, 0x00, 0x26, 0xE4, 0x09, 0xB9, 0x14, 0x0A, 0x86, 0x43, 0x81, 0x20,
        0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05, 0x10, 0x00,
        0x00, 0x4C, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x17, 0x0A,
        0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x16, 0x0A, 0x54, 0x02, 0x29, 0x16, 0x0A,
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int f_off0(Wrap0* w) { int x; x = w->o.Get(); return x; }`, the member `o`
    /// at **offset 0** — the address arithmetic emits nothing and the reference is
    /// a bare `b ?Get@M@@QBAHXZ` (MEASURED, `work/d2/p3.obj`). The offset-4 twin
    /// [`RECV_FIELD`] emits `addi r3,r3,4` first, which is why the two are separate
    /// buckets.
    const RECV_FIELD_OFF0: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x2E, 0x0A, 0x26, 0xE4, 0x09, 0xB9, 0x2B, 0x0A, 0x86, 0x43,
        0x81, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x86, 0x20, 0x99, 0x86, 0x43,
        0x8A, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x4C, 0x32,
        0x86, 0x41, 0x74, 0x4B, 0xB9, 0x2E, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A,
        0x2D, 0x0A, 0x54, 0x02, 0x29, 0x2D, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `struct HasMem { ~HasMem(); MemA m; }; HasMem::~HasMem() {}` — the generated
    /// destructor of a class with **no base** and one destructible member at offset
    /// 0. Its whole body is one member call through a plain `27` offset add (no
    /// intrinsic at all, unlike D1's base delegation), and the reference emits
    /// **`b ??1MemA@@QAA@XZ`**, 4 bytes, one REL24 — byte-identical in form to what
    /// D1 already emits. Fixture profile, so the trailers read `5C … 11` /
    /// `5E 01 31`.
    const DTOR_MEMBER_OFF0: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xFE, 0x09, 0xB9, 0x34, 0x0A,
        0xA6, 0x43, 0x92, 0x20, 0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0xA6, 0x43, 0x9A, 0x20, 0x2C,
        0xA6, 0x43, 0x9B, 0x20, 0x00, 0x99, 0x86, 0x43, 0x9C, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x1C, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74, 0x11, 0x4B, 0x3A, 0x35,
        0x0A, 0x54, 0x02, 0x29, 0x35, 0x0A, 0x5E, 0x01, 0x31, 0x4B, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];
    /// The same with the member at offset 4 (`HasMem4 { int pad; MemA m; }`) —
    /// reference `addi r3,r3,4 ; b ??1MemA@@QAA@XZ`. One instruction of new
    /// codegen, and the only byte that differs from [`DTOR_MEMBER_OFF0`] in the
    /// designator is the offset literal.
    const DTOR_MEMBER_OFF4: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xFE, 0x09, 0xB9, 0x37, 0x0A,
        0xA6, 0x43, 0xA1, 0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0xA6, 0x43, 0x9A, 0x20, 0x2C,
        0xA6, 0x43, 0x9B, 0x20, 0x00, 0x99, 0x86, 0x43, 0x9C, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x1C, 0x10, 0x00, 0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74, 0x11, 0x4B, 0x3A, 0x38,
        0x0A, 0x54, 0x02, 0x29, 0x38, 0x0A, 0x5E, 0x01, 0x31, 0x4B, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];
    /// `int t_byval() { int x; x = GetV().Val(); return x; }` — a member call on a
    /// **by-value returned temporary**. The `9B` binds the temporary and opcode
    /// `0x44` sits between the cv strip and the bind; neither is decoded, so this
    /// files as `op-0x9B` and the name stays hex. It is the single largest residue
    /// on the real workload (39,360) and §4's "`9B` temporary receiver, 5 sites"
    /// was the sample's shadow of it.
    const BYVAL_TEMP: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x3B, 0x0A, 0x26, 0x23, 0x0A, 0x9B, 0x82, 0x16, 0xA6, 0x20,
        0x3C, 0x0A, 0x26, 0x2A, 0x0A, 0xBD, 0x82, 0x16, 0xA6, 0x20, 0x00, 0x80, 0x2B, 0x10, 0x00,
        0x00, 0x4C, 0x32, 0x82, 0x16, 0xA6, 0x20, 0x9B, 0x82, 0x16, 0xA6, 0x20, 0x3C, 0x0A, 0x2C,
        0x86, 0x43, 0xAC, 0x20, 0x00, 0x44, 0x99, 0x86, 0x43, 0xA8, 0x20, 0x00, 0xBD, 0x86, 0x41,
        0x74, 0x00, 0x80, 0x28, 0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x3B,
        0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x3A, 0x0A, 0x54, 0x02, 0x29, 0x3A,
        0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// **A wild witness, and the one that pins the class-pair descriptor.** A
    /// base-delegating generated destructor from `src/App.cpp` at the dc3 workload's
    /// own flags, transcribed from its census window: 4-byte tokens throughout and
    /// a descriptor `66 02 fb 8a 01 e0 91 01` — two **three**-byte LEB refs, where
    /// every small probe has two-byte ones.
    ///
    /// Everything else about it is D1's skeleton exactly (selector 2113 wide,
    /// adjust offset 0, `2C` strip, void `BD`, zero arguments, `5C … 01`,
    /// `5E 01 21`, the plumbing reaching the segment end) — so
    /// `try_parse_empty_dtor_delegation` would accept it but for stepping the
    /// descriptor a fixed four bytes. See [`eat_class_descriptor`].
    const WILD_DTOR_WIDE_DESCRIPTOR: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0x7D, 0xC3, 0x02, 0x00, 0x33,
        0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00, 0x40, 0x86, 0x43, 0xBF, 0x93, 0x01, 0x66,
        0x02, 0xFB, 0x8A, 0x01, 0xE0, 0x91, 0x01, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74,
        0x00, 0x55, 0x86, 0x41, 0x74, 0xB9, 0xDA, 0xC4, 0x02, 0x00, 0xA6, 0x43, 0xE8, 0x92, 0x01,
        0x55, 0xA6, 0x43, 0xE8, 0x92, 0x01, 0x4C, 0x2C, 0xA6, 0x43, 0x89, 0x92, 0x01, 0x00, 0x99,
        0x86, 0x43, 0xBB, 0x92, 0x01, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x3B, 0x49, 0x00,
        0x00, 0x4C, 0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B, 0x3A, 0xDB, 0xC4, 0x02, 0x00, 0x54, 0x02,
        0x29, 0xDB, 0xC4, 0x02, 0x00, 0x5E, 0x01, 0x21, 0x4B, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54,
        0x00,
    ];

    /// `int c_ret(O* p) { return p->Next()->Get(); }` — **the witness for the largest
    /// actionable pair in the bucket** (`recv-load` x `chain-bind`, 12,480 functions
    /// at the workload). `work/sb/probes/s1.cpp`, fixture profile.
    ///
    /// It is a *two-link chain*, byte for byte: `26 <Get> 26 <Next> B9 <p> 99 … 4C
    /// 99 … 4C`. But there is no assignment destination, so `mod.rs`'s statement
    /// dispatch takes the head `26 <Get>` for one and `parse_expr` starts at
    /// `26 <Next>` — where exactly one method is stacked. D2 therefore files a chain
    /// as **`recv-load`**, and [`PROBE_CHAIN_IN_ASSIGNMENT`] is the *same source
    /// construct* filing as `chained` because it has a destination to absorb the head
    /// push. That is `docs/IL_CALL_IN_EXPR.md` §9.2's mis-attribution one level down,
    /// and D4 is what found it: the second blocker `chain-bind` is the outer bind of
    /// the chain the form classification lost.
    const PROBE_CHAIN_IN_RETURN: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xEC, 0x09, 0x26, 0xEE, 0x09, 0xB9, 0x01, 0x0A, 0x86, 0x43,
        0x86, 0x20, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00, 0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80,
        0x07, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74,
        0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x03, 0x0A, 0x54,
        0x02, 0x29, 0x03, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int c_asg(O* p) { int x; x = p->Next()->Get(); return x; }` — the same two
    /// links as [`PROBE_CHAIN_IN_RETURN`] with an assignment destination in front,
    /// which is the *only* source difference and moves the whole function to another
    /// bucket.
    const PROBE_CHAIN_IN_ASSIGNMENT: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x07, 0x0A, 0x26, 0xEC, 0x09, 0x26, 0xEE, 0x09, 0xB9, 0x04,
        0x0A, 0x86, 0x43, 0x86, 0x20, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00, 0xBD, 0x86, 0x43, 0x86,
        0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD,
        0x86, 0x41, 0x74, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B,
        0xB9, 0x07, 0x0A, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x06, 0x0A, 0x54, 0x02,
        0x29, 0x06, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int s_asg(O* p) { int x; x = p->Get(); return x; }` — **the re-anchor's
    /// control**, and the reason [`reanchor_chain`] needs a bind count rather than a
    /// head-run length.
    ///
    /// Its head is `26 <x> 26 <Get> B9 <p> 99 …` — *two* symbol pushes before the
    /// receiver, exactly like [`PROBE_CHAIN_IN_RETURN`]'s `26 <Get> 26 <Next> B9 <p>
    /// 99 …`. The only structural difference is that this statement contains **one**
    /// depth-0 `99` bind and the chain contains two. Captured from `work/fa/probes/p6.cpp`,
    /// fixture profile.
    const PROBE_ONE_LINK_ASSIGN: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0x02, 0x0A, 0x26, 0xED, 0x09, 0xB9, 0xFF, 0x09, 0x86, 0x43,
        0x81, 0x20, 0x99, 0x86, 0x43, 0x88, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x08,
        0x10, 0x00, 0x00, 0x4C, 0x32, 0x86, 0x41, 0x74, 0x4B, 0xB9, 0x02, 0x0A, 0x86, 0x41, 0x74,
        0x41, 0x86, 0x41, 0x74, 0x3A, 0x01, 0x0A, 0x54, 0x02, 0x29, 0x01, 0x0A, 0x4F, 0x12, 0x47,
        0x54, 0x01, 0x54, 0x00,
    ];

    /// `int b_if() { if (gO.Ok()) return 1; return 0; }` — **the branch witness, and
    /// the polarity of `38` with it.**
    ///
    /// The `bool`-returning call on a named object closes with `4C`, then `38 00 0A`
    /// — and `29 00 0A` **defines that exact label** further down, immediately before
    /// the `return 0`. So `38 <label>` is a conditional branch and it is taken when
    /// the condition is **false**: the fall-through path is `33 <int> 1 … 3A`
    /// (`return 1`). The workload's much larger `39` flavour pairs with a later `29`
    /// the same way (two wild witnesses in `src/system/hamobj/Ham.cpp`, one of them
    /// `b9 <x> 86 42 75 33 86 41 74 01 0b 39 2b 67` = `if (x & 1)`) but its polarity
    /// is UNDETERMINED, which is why the byte stays in the key.
    ///
    /// 23,632 workload functions sit in this one row, and it is the reason
    /// [`Blocker::Branch`] has no production: what those bodies need is basic
    /// blocks.
    const PROBE_IF_ON_NAMED_OBJECT: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x53, 0x26, 0xED, 0x09, 0x26, 0xF7, 0x09, 0x2C, 0xA6, 0x43, 0x82,
        0x20, 0x00, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x82, 0x12, 0x30, 0x00, 0x80, 0x05,
        0x10, 0x00, 0x00, 0x4C, 0x38, 0x00, 0x0A, 0x53, 0x33, 0x86, 0x41, 0x74, 0x01, 0x41, 0x86,
        0x41, 0x74, 0x3A, 0xFF, 0x09, 0x54, 0x04, 0x29, 0x00, 0x0A, 0x54, 0x03, 0x33, 0x86, 0x41,
        0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xFF, 0x09, 0x54, 0x02, 0x29, 0xFF, 0x09, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// **A wild witness for the same shape**, from `src/system/rndobj/Mesh.cpp` at
    /// the dc3 workload's own flags: a two-link chain in a return position whose
    /// outer link takes an `int` argument and whose result is then dereferenced to a
    /// `const float`. Filed `recv-load`, second blocker `chain-bind`, and **not**
    /// whole — the trailing `30 A6 45 F3 30` is a third construct. Kept because
    /// §14.2's fourth caution applies here too: only wild witnesses show what the
    /// argument regions and result types of a real chain look like.
    const WILD_CHAIN_AS_RECV_LOAD: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF5, 0x42, 0x26, 0x6B, 0x43, 0xB9, 0x8F, 0x43, 0x86, 0x43,
        0xFE, 0x31, 0x99, 0x86, 0x43, 0xF2, 0x31, 0x00, 0xBD, 0x86, 0x43, 0xCB, 0x31, 0x00, 0x80,
        0xF2, 0x18, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0xC3, 0x31, 0x00, 0xBD, 0x86, 0x43, 0xF4,
        0x30, 0x00, 0x80, 0xC3, 0x18, 0x00, 0x00, 0xB9, 0x75, 0x43, 0x86, 0x41, 0x74, 0x55, 0x86,
        0x41, 0x74, 0x4C, 0x30, 0xA6, 0x45, 0xF3, 0x30, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x41, 0x86,
        0x45, 0x40, 0x3A, 0x90, 0x43, 0x54, 0x02, 0x29, 0x90, 0x43, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];

    /// **A wild witness for `recv-field` x `off-add`, the 8,486-function row**, from
    /// `src/system/hamobj/Ham.cpp` at the workload's flags. It is a generated
    /// destructor that destroys a member sub-object at offset 0x18 (D3m's accepted
    /// shape, §15) and *then* does the equivalent of `delete mThing;` on a pointer
    /// member at offset 0x14:
    ///
    /// ```text
    ///   … 4C 5C 86 41 74 01 4B                            the accepted statement, closed
    ///   B9 <this> 33 <int> 14 27 <T> 30 <T> 38 <label>    load the pointer, branch if null
    ///   26 <dtor> B9 <this> 33 14 27 30 99 … BD void … 4C 4B    destroy through it
    ///   B9 <this> 33 14 27 33 <ptr> 0 32 <ptr> 4B         null the member
    /// ```
    ///
    /// The *named* second blocker is the `27` off-add of the second statement —
    /// correctly, since that is the first token the grammar cannot take — but the
    /// greedy chain shows the body needs an off-add, a `30` indirect load, a
    /// **conditional branch** and a pointer store, and the branch is unmodelable.
    /// Hence `-more`. This is the shape that makes `off-add`'s 11,211 functions worth
    /// almost nothing on their own: 2 of them are one construct away.
    const WILD_DTOR_DELETES_A_MEMBER: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xE1, 0x8F, 0x03, 0x00, 0xB9,
        0xBE, 0xA5, 0x05, 0x00, 0xA6, 0x43, 0x9D, 0x9B, 0x02, 0x33, 0x86, 0x41, 0x74, 0x18, 0x27,
        0xA6, 0x43, 0xCE, 0xBB, 0x01, 0x2C, 0xA6, 0x43, 0xF6, 0xBB, 0x01, 0x00, 0x99, 0x86, 0x43,
        0xA8, 0xBC, 0x01, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x28, 0x5E, 0x00, 0x00, 0x4C,
        0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B, 0x4F, 0x01, 0x1E, 0x53, 0xB9, 0xBE, 0xA5, 0x05, 0x00,
        0xA6, 0x43, 0x9D, 0x9B, 0x02, 0x33, 0x86, 0x41, 0x74, 0x14, 0x27, 0xA6, 0x43, 0xEB, 0x32,
        0x30, 0x86, 0x43, 0xD5, 0x30, 0x38, 0xC0, 0xA5, 0x05, 0x00, 0x53, 0x53, 0x4F, 0x01, 0x1F,
        0x26, 0x7F, 0x40, 0xB9, 0xBE, 0xA5, 0x05, 0x00, 0xA6, 0x43, 0x9D, 0x9B, 0x02, 0x33, 0x86,
        0x41, 0x74, 0x14, 0x27, 0xA6, 0x43, 0xEB, 0x32, 0x30, 0x86, 0x43, 0xD5, 0x30, 0x99, 0x86,
        0x43, 0xCD, 0x31, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0xCD, 0x18, 0x00, 0x00, 0x4C,
        0x4B, 0x4F, 0x01, 0x20, 0xB9, 0xBE, 0xA5, 0x05, 0x00, 0xA6, 0x43, 0x9D, 0x9B, 0x02, 0x33,
        0x86, 0x41, 0x74, 0x14, 0x27, 0xA6, 0x43, 0xEB, 0x32, 0x33, 0x86, 0x43, 0xD5, 0x30, 0x00,
        0x32, 0x86, 0x43, 0xD5, 0x30, 0x4B, 0x4F, 0x01, 0x21, 0x54, 0x05, 0x4F, 0x01, 0x22, 0x54,
        0x04, 0x29, 0xC0, 0xA5, 0x05, 0x00, 0x54, 0x03, 0x3A, 0xBF, 0xA5, 0x05, 0x00, 0x54, 0x02,
        0x29, 0xBF, 0xA5, 0x05, 0x00, 0x5E, 0x01, 0x21, 0x4B, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54,
        0x00,
    ];

    /// The whole point of the rung: one bucket becomes a set of named ones, and
    /// each name is the *construct*, not the byte the parse stopped on.
    #[test]
    fn every_probe_reports_its_own_construct() {
        let cases: &[(&[u8], &str)] = &[
            (RECV_FIELD_OFF0, "expr-call-in-expr-recv-field-off0-whole"),
            (DTOR_MEMBER_OFF0, "expr-call-in-expr-recv-field-off0-whole"),
            (DTOR_MEMBER_OFF4, "expr-call-in-expr-recv-field-whole"),
            (BYVAL_TEMP, "expr-call-in-expr-op-0x9B"),
            (
                WILD_DTOR_WIDE_DESCRIPTOR,
                "expr-call-in-expr-recv-intrinsic-this-adjust-whole",
            ),
            (RECV_LOAD, "expr-call-in-expr-recv-load-whole"),
            (RECV_DEREF, "expr-call-in-expr-recv-deref-whole"),
            (RECV_FIELD, "expr-call-in-expr-recv-field-whole"),
            (RECV_OBJECT, "expr-call-in-expr-recv-object-whole"),
            (RECV_CALL, "expr-call-in-expr-recv-call-whole"),
            (RECV_INTRINSIC, "expr-call-in-expr-recv-intrinsic-this-adjust-whole"),
            // D4 gave `chained` a production, so its completeness is now MEASURED
            // rather than absent: `x = p->Next()->Val();` is a whole body.
            (CHAINED, "expr-call-in-expr-chained-whole"),
            // `g1(g1(a))`: the second blocker is the *same* construct, nested in the
            // argument region — and admitting both finishes the body, which is what
            // the `-whole` suffix on a `-then-` key means.
            (NESTED_CALL, "expr-call-in-expr-nested-call-then-call-nested-call-whole"),
            // `uc("hi")` and `ui(&gA[2])` used to report
            // `data-addr-1sym-then-plain-call-whole` here. **WR1 built that
            // production**, so the single-symbol form is no longer a second
            // blocker at all: the parse walks the argument region and reaches the
            // formals, which this excerpt does not carry. See [`probe_block`].
            (DATA_ADDR, PARSED_PAST_THE_ARGUMENTS),
            // `ui(&gA[2])` is UNCHANGED, and the pair is the discriminator: the
            // subscript's byte-offset run is refused, because the addend is not
            // folded into the relocation — `lis ; addi r11,r11,0 ; addi r3,r11,8`
            // is a THIRD instruction (§17.2 item 1) — so this one is still a
            // second blocker where the bare designator beside it is not.
            (DATA_ADDR_INDEX, "expr-call-in-expr-data-addr-1sym-then-plain-call-whole"),
            // TWO symbols is still a refusal and still the `2sym` class — c2
            // materializes only the first through a relocation pair and derives
            // the second by `.rdata` pool-offset difference (§17.3 (a)) — but the
            // designators themselves now parse, so in this excerpt the formals run
            // out first. On a whole function it is `call-arg-multi-sym`, which
            // `fixtures/cpp/wr1_sym_addr_neg.cpp` grades (`t1`, `t2`).
            (DATA_ADDR_TWO_SYMS, PARSED_PAST_THE_ARGUMENTS),
            // …and `x = gO.m;` carries no suffix at all: its data symbol is read at
            // statement level, never materialized into an argument register, so the
            // count is 0 and the key says nothing rather than saying "one".
            (DATA_READ, "expr-call-in-expr-data-read-whole"),
            (RECV_LOAD_IN_ARG, "expr-call-in-expr-recv-load-then-call-nested-call-whole"),
        ];
        for (seg, want) in cases {
            let s = free_fn(seg);
            let b = probe_block(seg);
            assert_eq!(b.feature(), *want);
            // …and a CALL_IN_EXPR key is still reported at the `26` it names.
            // The two WR1 admitted are reported past their arguments instead, and
            // that is the whole difference.
            if b.ctx == CALL_IN_EXPR {
                assert_eq!(s[b.off], 0x26, "{want}: reported at the `26`");
            }
        }
    }

    /// Decoding is not accepting. Every one of these still fails closed, so the
    /// census and the emission gate cannot disagree — the invariant that makes a
    /// measurement rung safe at all.
    #[test]
    fn the_decode_accepts_nothing() {
        for seg in [
            RECV_LOAD,
            RECV_DEREF,
            RECV_FIELD,
            RECV_OBJECT,
            RECV_CALL,
            RECV_INTRINSIC,
            CHAINED,
            NESTED_CALL,
            DATA_ADDR,
            DATA_ADDR_INDEX,
            DATA_READ,
            RECV_LOAD_IN_ARG,
            RECV_FIELD_OFF0,
            DTOR_MEMBER_OFF0,
            DTOR_MEMBER_OFF4,
            BYVAL_TEMP,
            WILD_DTOR_WIDE_DESCRIPTOR,
        ] {
            assert!(parse_segment(&free_fn(seg), NO_LOCALS).is_none());
        }
    }

    /// The class-pair descriptor's refs are **LEB128 ids**, and the only witnesses
    /// that can prove it are the wide ones from a real TU. Under the fixed-2-byte
    /// reading `shapes.rs` still uses, the walk lands two bytes short — inside the
    /// second ref — and files the function under whatever payload byte it finds.
    /// MEASURED: that reading spread 17,757 workload functions over 197
    /// `op-0xNN` buckets; this one leaves 127 in 7.
    #[test]
    fn the_class_pair_descriptor_refs_are_leb_ids_not_fixed_pairs() {
        // The wide witness classifies, and its whole body is accounted for.
        let seg = free_fn(WILD_DTOR_WIDE_DESCRIPTOR);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-call-in-expr-recv-intrinsic-this-adjust-whole");
        // The descriptor really is 2 + 3 + 3 bytes, and it ends exactly on the `55`
        // argument terminator — which is the marker that pins the width, the same
        // way `41`/`55`/`4C 4B` pin `read_type`'s.
        let at = seg.windows(2).position(|w| w == [0x66, 0x02]).unwrap();
        let mut p = at;
        assert!(eat_class_descriptor(&seg, &mut p).is_some());
        assert_eq!(p - at, 8);
        assert_eq!(seg[p], 0x55, "the descriptor must end on the argument push");
        // Stepping a fixed four bytes lands inside the second ref instead.
        assert_ne!(seg[at + 4], 0x55);
        // …and the narrow probe agrees with the same reader, at 2 + 2 + 2.
        let narrow = free_fn(RECV_INTRINSIC);
        let at = narrow.windows(2).position(|w| w == [0x66, 0x02]).unwrap();
        let mut p = at;
        assert!(eat_class_descriptor(&narrow, &mut p).is_some());
        assert_eq!(p - at, 6);
        assert_eq!(narrow[p], 0x55);
    }

    /// The offset literal decides whether the receiver's address costs an
    /// instruction, so it decides the bucket. MEASURED (`work/d2/p3.obj`): the
    /// offset-0 generated destructor is `b ??1MemA@@QAA@XZ` and its offset-4 twin is
    /// `addi r3,r3,4 ; b ??1MemA@@QAA@XZ`, and the two segments differ in exactly
    /// that one literal byte (plus per-TU tokens and type ids).
    #[test]
    fn a_zero_offset_receiver_is_a_different_bucket_from_a_nonzero_one() {
        let a = parse_segment_detail(&free_fn(DTOR_MEMBER_OFF0), NO_LOCALS).unwrap_err();
        let b = parse_segment_detail(&free_fn(DTOR_MEMBER_OFF4), NO_LOCALS).unwrap_err();
        assert_eq!(a.feature(), "expr-call-in-expr-recv-field-off0-whole");
        assert_eq!(b.feature(), "expr-call-in-expr-recv-field-whole");
        assert_eq!(DTOR_MEMBER_OFF0.len(), DTOR_MEMBER_OFF4.len());
    }

    /// The two probes that differ by exactly one token must not share a bucket:
    /// `w->p->Get()` loads the receiver and `w->o.Get()` takes its address. That
    /// is the same distinction the `return *p;` / `return &s->m;` pair turns on,
    /// where conflating them emits a bare `blr` for an `addi`.
    #[test]
    fn the_load_and_the_address_of_a_sub_object_are_different_buckets() {
        let deref = free_fn(RECV_DEREF);
        let field = free_fn(RECV_FIELD);
        let a = parse_segment_detail(&deref, NO_LOCALS).unwrap_err();
        let b = parse_segment_detail(&field, NO_LOCALS).unwrap_err();
        assert_ne!(a.feature(), b.feature());
        // …and the only difference in the two designators is the `30` load.
        assert_eq!(deref.len(), field.len() + 5);
    }

    /// A chain's innermost bind has an ordinary `B9` receiver, so classifying on
    /// the receiver alone would hide every chain inside `recv-load` — the exact
    /// shape of the mis-attribution failure `GAPS.md` §6 records.
    #[test]
    fn a_chain_is_not_filed_as_its_innermost_receiver() {
        let seg = free_fn(CHAINED);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-call-in-expr-chained-whole");
        // …and the walk says so directly, from the right-hand side's own `26`.
        assert_eq!(walk(&seg, find_first_26_in_rhs(&seg)), CallForm::Chained);
        // The bytes really do open with two method pushes over one `B9` load.
        let rhs = find_first_26_in_rhs(&seg);
        assert_eq!(seg[rhs], 0x26);
        assert_eq!(seg[rhs + 3], 0x26);
        assert_eq!(seg[rhs + 6], 0xB9);
    }

    /// Same construct, two statement positions, one bucket. `docs/IL_CALL_IN_EXPR.md`
    /// §9.2 is the failure this guards: statement position decides which bucket a
    /// whole function lands in, and a decomposition that repeated that inside the
    /// bucket would measure the parser rather than the corpus.
    #[test]
    fn statement_position_does_not_change_the_bucket() {
        let a = parse_segment_detail(&free_fn(RECV_LOAD), NO_LOCALS).unwrap_err();
        let b = parse_segment_detail(&free_fn(RECV_LOAD_IN_ARG), NO_LOCALS).unwrap_err();
        // Same construct…
        assert_eq!(a.aux & FORM_MASK, b.aux & FORM_MASK);
        // …but only one of them is a whole body that one form would finish: the
        // other needs a second call and a frame.
        assert_ne!(a.aux & WHOLE_BIT, b.aux & WHOLE_BIT);
        assert_eq!(a.feature(), "expr-call-in-expr-recv-load-whole");
        // …and D4 says *what* the other one needs on top: a plain call around it.
        assert_eq!(b.feature(), "expr-call-in-expr-recv-load-then-call-nested-call-whole");
    }

    /// Nothing per-TU may reach a key. Retag every per-TU field in the
    /// `recv-load` witness — the method token, the receiver token, the local's
    /// token, the inline TYPE ids and the function-type id — and the bucket must
    /// not move. This is the sharded-key failure (`GAPS.md` §6) stated as a test
    /// rather than as an intention.
    #[test]
    fn per_tu_identifiers_do_not_shard_the_bucket() {
        let base = free_fn(RECV_LOAD);
        let want = parse_segment_detail(&base, NO_LOCALS).unwrap_err().feature();
        // The function-type id `80 05 10 00 00` → `80 7F 10 00 00`.
        let mut v = base.clone();
        let at = v.windows(5).position(|w| w == [0x80, 0x05, 0x10, 0x00, 0x00]).unwrap();
        v[at + 1] = 0x7F;
        assert_eq!(parse_segment_detail(&v, NO_LOCALS).unwrap_err().feature(), want);
        // The receiver's inline TYPE id `86 43 81 20` → `86 43 FF 20` (same class,
        // different per-TU id).
        let mut v = base.clone();
        let at = v.windows(4).position(|w| w == [0x86, 0x43, 0x81, 0x20]).unwrap();
        v[at + 2] = 0xFF;
        assert_eq!(parse_segment_detail(&v, NO_LOCALS).unwrap_err().feature(), want);
    }

    /// The residue must name the byte it could not tokenize, not a guess. A
    /// virtual member call's `67` after a `26` is the real case (probe `v_virt`).
    #[test]
    fn an_untokenizable_byte_becomes_an_honest_hex_bucket() {
        let mut v = RECV_LOAD.to_vec();
        // Replace the receiver LOAD with an unmodeled opcode.
        let at = v.iter().position(|&b| b == 0xB9).unwrap();
        v[at] = 0x67;
        let seg = free_fn(&v);
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-call-in-expr-op-0x67");
    }

    /// The completeness bit is a *whole-segment* claim: truncate the return
    /// plumbing and it must clear, even though the member call itself is intact.
    #[test]
    fn completeness_requires_the_whole_segment() {
        let mut v = RECV_LOAD.to_vec();
        // Drop the final `47 54 01 54 00` function-tail terminator.
        v.truncate(v.len() - 5);
        let b = parse_segment_detail(&free_fn(&v), NO_LOCALS).unwrap_err();
        // The member call itself is intact, so D4 attributes the refusal to the
        // *plumbing* — a structural bucket, not a value construct, and UNMEASURED
        // as a pair because no production admits a truncated function tail.
        assert_eq!(b.feature(), "expr-call-in-expr-recv-load-then-plumbing-0x3A");
        // …and an extra statement after the call is not a whole body either.
        let mut v = RECV_LOAD.to_vec();
        let at = v.windows(2).position(|w| w == [0x32, 0x86]).unwrap();
        v.splice(at..at, [0x4B].iter().copied());
        let b = parse_segment_detail(&free_fn(&v), NO_LOCALS).unwrap_err();
        // Not complete on the form alone: the key must carry a `-then-` half. (A
        // `-then-…-whole` would mean "complete once a *second* construct is granted
        // too", which is a different claim and would still not be `-whole`.)
        assert!(!b.feature().ends_with("-recv-load-whole"), "{}", b.feature());
        assert!(b.feature().contains("-then-"), "{}", b.feature());
    }

    /// The `aux` layout round-trips every form, including the two that carry a
    /// payload. A silent collision here would merge buckets, which is the one
    /// failure a census instrument cannot survive.
    #[test]
    fn the_aux_packing_round_trips_every_form() {
        let forms = [
            CallForm::RecvLoad,
            CallForm::RecvDeref,
            CallForm::RecvField,
            CallForm::RecvFieldZero,
            CallForm::RecvObject,
            CallForm::RecvCall,
            CallForm::RecvIntrinsic(2113),
            CallForm::RecvIntrinsic(2119),
            CallForm::RecvOther,
            CallForm::Chained,
            CallForm::NestedCall,
            CallForm::DataAddr,
            CallForm::DataRead,
            CallForm::Intrinsic(173),
            CallForm::Other,
            CallForm::Op(0x9B),
            CallForm::Eof,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for f in forms {
            let (disc, payload) = f.code();
            assert!(disc <= FORM_MASK, "{f:?}: discriminant overflows its field");
            assert!(payload <= PAYLOAD_MASK, "{f:?}: payload overflows its field");
            assert_eq!(CallForm::from_code(disc, payload), Some(f), "{f:?}");
            let key = feature(disc | (payload << FORM_BITS));
            assert!(key.starts_with(CALL_IN_EXPR), "{key}");
            assert!(seen.insert(key.clone()), "duplicate bucket name {key}");
            // …and the `-whole` variant is a distinct, disjoint bucket.
            let whole = feature(disc | (payload << FORM_BITS) | WHOLE_BIT);
            assert_eq!(whole, format!("{key}-whole"));
            assert!(seen.insert(whole));
        }
    }

    // --- D4: the second blocker ---------------------------------------------

    /// The rung's own point: every 0 %-complete form now says what blocks it *next*,
    /// and the name is the construct.
    #[test]
    fn every_second_blocker_names_its_own_construct() {
        let cases: &[(&[u8], &str)] = &[
            // §18.3's own row — the value-position two-link chain — is NOT here
            // any more: WCH gave `chained` an acceptance production and
            // `PROBE_CHAIN_IN_RETURN` is in class, so it has no blocker key to
            // name. The form claim it carried is now made directly against
            // [`classify`], which is the decode this test is about, in
            // `a_chain_is_a_chain_in_both_statement_positions` below.
            // `WILD_CHAIN_AS_RECV_LOAD` was the third row here and is not any
            // more: **WCO** gave the chain-plus-designator an acceptance
            // production, and that segment parses to the end of the body under
            // it (`30 A6 45 F3 30` is an indirect load of a `const float`), so
            // it refused under `mcall-chain-tail-load-class` — a complete body
            // stopped by a named gate, which is strictly more informative than
            // a second-blocker name. **WFL then emitted that instruction**
            // (`lfs f1,k(r3)`), so the segment is now an ACCEPTANCE witness and
            // not a refusal one. Asserted at its site, in `shapes::mcall_chain`.
            (PROBE_CHAIN_IN_ASSIGNMENT, "expr-call-in-expr-chained-whole"),
            (PROBE_IF_ON_NAMED_OBJECT, "expr-call-in-expr-recv-object-then-branch-brfalse"),
            // …and here too: off-add, then the `30 86 43 D5 30` indirect load of the
            // pointer member, then the conditional branch that stops the chain.
            (
                WILD_DTOR_DELETES_A_MEMBER,
                "expr-call-in-expr-recv-field-then-off-add-and-deref-load-more",
            ),
        ];
        for (seg, want) in cases {
            let seg = free_fn(seg);
            let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
            assert_eq!(b.feature(), *want);
            // Still a refusal, and still reported at the `26`: D4 moved the key, not
            // the gate and not the offset.
            assert_eq!(seg[b.off], 0x26, "{want}");
            assert!(parse_segment(&seg, NO_LOCALS).is_none(), "{want}");
        }
    }

    /// **§16.4's finding, now fixed, as a test.** One source construct — a two-link
    /// chain — used to file under two different D2 forms depending on whether the
    /// statement had an assignment destination to absorb the head method push, and
    /// the `chained` bucket understated the chain population ~4.4× as a result. The
    /// statement-head re-anchor ([`reanchor_chain`], §18.3) puts the value-position
    /// form back where the assignment form already was.
    ///
    /// **The value-position half is asserted against [`classify`] rather than
    /// against the body dispatch**, because WCH accepted that half: a chain in a
    /// return position is now in class and raises no blocker at all. The claim
    /// this test makes is about the *decode*, and `classify` is the decode — going
    /// through `parse_segment_detail` for it was always one indirection more than
    /// the claim needed, and acceptance is exactly what that indirection is
    /// sensitive to.
    #[test]
    fn a_chain_is_a_chain_in_both_statement_positions() {
        let ret_seg = free_fn(PROBE_CHAIN_IN_RETURN);
        let lo = crate::func::readers::find_subslice(&ret_seg, &[0x4C, 0x4F, 0x11]).unwrap();
        let at = lo + 4; // past `LO` and the `53`
        assert_eq!(ret_seg[at], 0x26, "the chain's outermost method push");
        let ret = classify(&ret_seg, at);
        let asg = parse_segment_detail(&free_fn(PROBE_CHAIN_IN_ASSIGNMENT), NO_LOCALS).unwrap_err();
        // The same *form* now, which is the fix.
        assert_eq!(ret.aux & FORM_MASK, asg.aux & FORM_MASK);
        assert_eq!(ret.feature(), "expr-call-in-expr-chained");
        assert_eq!(asg.feature(), "expr-call-in-expr-chained-whole");
        // …and the assignment body really is the return body plus one `26 <dst>`
        // push, byte for byte, plus the store and load of the local.
        assert!(PROBE_CHAIN_IN_ASSIGNMENT.len() > PROBE_CHAIN_IN_RETURN.len());
        // The assignment half is still a refusal, still at a `26`, and still
        // whole-body accounted for: the key moved, the gate did not.
        let asg_seg = free_fn(PROBE_CHAIN_IN_ASSIGNMENT);
        assert_eq!(asg_seg[asg.off], 0x26);
        assert!(parse_segment(&asg_seg, NO_LOCALS).is_none());
    }

    /// **The guard on the re-anchor, and the reason it needs a bind count.**
    ///
    /// A *single-link* member call in an assignment has the same head shape as a
    /// two-link chain in a value position — `26 <tok> 26 <tok>` then the receiver —
    /// and differs only in how many `99` binds the statement contains. A re-anchor
    /// that trusted the head run alone would promote every one of them, trading
    /// §16.4's 4.4× undercount for an overcount of the whole corpus's assignments.
    #[test]
    fn a_single_link_call_with_a_destination_is_not_promoted_to_a_chain() {
        // `x = p->Get();` — head run of two symbols, ONE bind.
        let one = parse_segment_detail(&free_fn(PROBE_ONE_LINK_ASSIGN), NO_LOCALS).unwrap_err();
        assert_eq!(one.feature(), "expr-call-in-expr-recv-load-whole");
        assert_eq!(depth0_binds(&free_fn(PROBE_ONE_LINK_ASSIGN), one.off, 4), 1);
        // …against the two-link value form, which has two. Counted from the
        // statement's own `26` rather than from a refusal's offset: WCH accepts
        // this body, so there is no refusal here to take an offset from, and the
        // bind count was never a property of one.
        let two = free_fn(PROBE_CHAIN_IN_RETURN);
        let at = crate::func::readers::find_subslice(&two, &[0x4C, 0x4F, 0x11]).unwrap() + 4;
        assert_eq!(two[at], 0x26);
        assert_eq!(depth0_binds(&two, at, 4), 2);
    }

    /// A branch is named a branch because its label is **defined later in the same
    /// segment** by a `29 <same token>`, not because the byte looked like one. The
    /// witness carries the pair, so the test can check it directly.
    #[test]
    fn a_branch_target_is_a_label_defined_later_in_the_segment() {
        let seg = PROBE_IF_ON_NAMED_OBJECT;
        let at = seg.iter().position(|&b| b == 0x38).expect("the branch");
        let (label, w) = read_token_var(seg, at + 1).expect("its target");
        // The same token appears after a `29` further down — that is the definition.
        let mut found = false;
        let mut q = at + 1 + w;
        while q + 1 < seg.len() {
            if seg[q] == 0x29 {
                if let Some((t, _)) = read_token_var(seg, q + 1) {
                    if t == label {
                        found = true;
                        break;
                    }
                }
            }
            q += 1;
        }
        assert!(found, "the `38` target {label:#x} is defined by a later `29`");
        // …and the construct is reported as a branch, with no completeness claim:
        // `Blocker::Branch` has no production, so the pair is UNMEASURED and the key
        // carries neither `-whole…` nor `-more`.
        let f = parse_segment_detail(&free_fn(seg), NO_LOCALS).unwrap_err().feature();
        // …and it is named, not hex: the polarity of the pair is capture-verified
        // (`super::cflow_opcode_name`), and this key producer reads that one table
        // so it cannot drift from the `expr-*` keys.
        assert!(f.ends_with("-then-branch-brfalse"), "{f}");
    }

    /// An UNMEASURED pair must not be readable as a measured incompleteness. The
    /// suffix carries that distinction and nothing else does.
    #[test]
    fn an_unmodelable_second_blocker_leaves_the_pair_unmeasured() {
        for (seg, blk) in [
            (PROBE_IF_ON_NAMED_OBJECT, Blocker::Branch(0x38)),
            (BYVAL_TEMP, Blocker::TempBind),
        ] {
            assert!(!blocker_is_measured(blk), "{blk:?}");
            let f = parse_segment_detail(&free_fn(seg), NO_LOCALS).unwrap_err().feature();
            assert!(!f.ends_with("-more"), "{f}");
            assert!(!f.contains("-whole"), "{f}");
        }
        // …while a modelable one always carries one or the other.
        // (`PROBE_CHAIN_IN_RETURN` was the third row here and is now in class —
        // WCH — so it raises no block to carry either suffix.)
        // (`WILD_CHAIN_AS_RECV_LOAD` was the first of these and is now claimed
        // by WCO's acceptance gate — see the note in
        // `every_second_blocker_names_its_own_construct`.)
        for seg in [WILD_DTOR_DELETES_A_MEMBER] {
            let f = parse_segment_detail(&free_fn(seg), NO_LOCALS).unwrap_err().feature();
            assert!(f.ends_with("-more") || f.contains("-whole"), "{f}");
        }
    }

    // ---- W37: the bare binary operator, and what a `-more` row broke on --------
    //
    // All three segments are transcribed verbatim from a live capture of
    // `fixtures/cpp/w37_bit_and_neg.cpp` (`c2rs census … --keep-il`), whole from the
    // `53 53 26 <fn>` statement start through the function tail — not truncated at
    // the formals marker, because `GAPS.md` §6 records that a trimmed segment
    // witnesses nothing about the region it omits.

    /// `int n_if_call_mask(const S *p){ if (p->Flags() & 4) return gk(1); return 0; }`
    /// — the workload row, at 102,374 of its 102,382 functions. The operand stream
    /// reads `… 4C · 33 86 41 74 04 · 0B · 38 <label> …`: the member call closes,
    /// the mask literal, the bare operator, and straight into a **conditional
    /// branch**.
    const PROBE_IF_CALL_MASK: &[u8] = &[
        0x53, 0x53, 0x26, 0xF1, 0x09, 0x46, 0x2D, 0xF0, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01,
        0x23, 0x53, 0x26, 0xE5, 0x09, 0xB9, 0xF0, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43,
        0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x33,
        0x86, 0x41, 0x74, 0x04, 0x0B, 0x38, 0xF3, 0x09, 0x53, 0x53, 0x4F, 0x01, 0x24, 0x26, 0xEF,
        0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x33, 0x86, 0x41, 0x74,
        0x01, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xF2, 0x09, 0x4F, 0x01,
        0x25, 0x54, 0x05, 0x4F, 0x01, 0x26, 0x54, 0x04, 0x29, 0xF3, 0x09, 0x54, 0x03, 0x33, 0x86,
        0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xF2, 0x09, 0x4F, 0x01, 0x27, 0x54, 0x02,
        0x29, 0xF2, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int n_ret_call_mask(const S *p){ return p->Flags() & 7; }` — the same row's
    /// **value** spelling, `… 4C · 33 86 41 74 07 · 0B · 41 86 41 74 …`. This is the
    /// control group the whole W37 measurement rests on: the completeness measure
    /// CAN report `-whole` for this row, and over 102,382 real workload functions it
    /// reported it **zero** times. A measure that could never say `-whole` would
    /// make that zero worthless.
    const PROBE_RET_CALL_MASK: &[u8] = &[
        0x53, 0x53, 0x26, 0x01, 0x0A, 0x46, 0x2D, 0x00, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01,
        0x48, 0x26, 0xE5, 0x09, 0xB9, 0x00, 0x0A, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85,
        0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86,
        0x41, 0x74, 0x07, 0x0B, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x02, 0x0A, 0x4F, 0x01, 0x49, 0x54,
        0x02, 0x29, 0x02, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int n_or(unsigned x){ return int(x | 1); }` — one of the four bytes that
    /// share `0B`'s encoding, kept so the "bare, one byte, nothing else" reading of
    /// [`BARE_BINARY_OPS`] has a witness per member and not one per family.
    const PROBE_BIT_OR: &[u8] = &[
        0x53, 0x53, 0x26, 0x0D, 0x0A, 0x46, 0x2D, 0x0C, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x4F, 0x01,
        0x60, 0xB9, 0x0C, 0x0A, 0x86, 0x42, 0x75, 0x33, 0x86, 0x41, 0x74, 0x01, 0x0C, 0x2C, 0x86,
        0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x0E, 0x0A, 0x4F, 0x01, 0x61, 0x54, 0x02,
        0x29, 0x0E, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// W42's relational witnesses. `int f(unsigned x, int y){ return x < 3; }`
    /// and the branch spelling, transcribed from the capture in
    /// `docs/OPERATOR_GRANTS.md` — the operator byte and the token that consumes
    /// it, with nothing in between.
    const PROBE_REL_VALUE: &[u8] = &[
        0x33, 0x86, 0x42, 0x75, 0x03, 0x22, 0x2C, 0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74,
    ];
    const PROBE_REL_BRANCH: &[u8] = &[
        0x33, 0x86, 0x41, 0x74, 0x03, 0x23, 0x38, 0x19, 0x0A,
    ];
    /// …and the **control** from the same capture: a compound assign, which is
    /// the family `19` belongs to and which *does* carry a TYPE. The old
    /// exclusion of `1F`-`24` was this byte, read across a family boundary the
    /// numeric order hides.
    const PROBE_COMPOUND_ASSIGN: &[u8] = &[
        0x26, 0x1D, 0x0A, 0x33, 0x86, 0x41, 0x74, 0x03, 0x19, 0x86, 0x42, 0x75, 0x4B,
    ];

    /// **The row that was #4 on the board says what it is.** Before W37 both
    /// segments below censused the same bare `expr-call-in-expr-recv-load-then-bit-and`
    /// — no `-whole`, no `-more`, nothing to rank by — because `Blocker::Op` had no
    /// production and [`mark_whole`]'s greedy chain stopped dead at the operator.
    /// They are two different bodies and the key has to separate them.
    #[test]
    fn a_bare_binary_operator_is_granted_and_the_pair_then_says_what_it_is() {
        // The value spelling completes on the operator alone.
        let f = parse_segment_detail(PROBE_RET_CALL_MASK, NO_LOCALS).unwrap_err().feature();
        assert_eq!(f, "expr-call-in-expr-recv-load-then-bit-and-whole");
        // The workload's actual spelling does not: it needs basic blocks, and the
        // key now names the branch instead of stopping silently at the `&`.
        let f = parse_segment_detail(PROBE_IF_CALL_MASK, NO_LOCALS).unwrap_err().feature();
        assert_eq!(f, "expr-call-in-expr-recv-load-then-bit-and-and-branch-more");
        // Granting a construct is never accepting one. Both still refuse.
        for seg in [PROBE_RET_CALL_MASK, PROBE_IF_CALL_MASK, PROBE_BIT_OR] {
            assert!(parse_segment(seg, NO_LOCALS).is_none());
        }
    }

    /// Every member of [`BARE_BINARY_OPS`] must be *bare* — one byte, no TYPE, no
    /// varint — or granting it desyncs the matcher and the row it was meant to
    /// measure scatters instead. Checked against the capture rather than asserted:
    /// in each witness the byte is immediately followed by the token that consumes
    /// the value.
    #[test]
    fn every_grantable_operator_byte_is_one_byte_in_a_capture() {
        assert_eq!(
            BARE_BINARY_OPS,
            &[0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24]
        );
        for (seg, op, next) in [
            (PROBE_IF_CALL_MASK, 0x0Bu8, 0x38u8),  // → conditional branch
            (PROBE_RET_CALL_MASK, 0x0B, 0x41),     // → result-type annotation
            (PROBE_BIT_OR, 0x0C, 0x2C),            // → class-preserving convert
            // W42's relational witnesses, from the same capture as the
            // compound-assign CONTROL below — value position and branch position,
            // signed and unsigned.
            (PROBE_REL_VALUE, 0x22, 0x2C),         // → class-preserving convert
            (PROBE_REL_BRANCH, 0x23, 0x38),        // → conditional branch
        ] {
            // The literal that feeds it, then the operator, then the consumer — with
            // nothing in between. `33 <86 41 74 | 86 42 75> <k>` is the literal;
            // BOTH signednesses are searched, because W42 measured that the
            // relational opcode does not carry one — unsigned `<` and signed `<`
            // are the same `22` and only the operand TYPE separates them.
            let at = seg
                .windows(6)
                .position(|w| {
                    w[0] == 0x33
                        && w[1] == 0x86
                        && (w[2..4] == [0x41, 0x74] || w[2..4] == [0x42, 0x75])
                        && w[5] == op
                })
                .unwrap_or_else(|| panic!("no `33 <int-like> <k> {op:#04X}` in the capture"));
            assert_eq!(seg[at + 5], op);
            assert_eq!(seg[at + 6], next, "op {op:#04X} is not one byte wide");
            assert!(BARE_BINARY_OPS.contains(&op));
        }
        // …and the bytes that are still NOT grantable. `1A` is unary and
        // `1B`/`1C` short-circuit; the relational family left this list in W42,
        // on the capture above plus a redistribution that summed to zero.
        for b in [0x1Au8, 0x1B, 0x1C] {
            assert!(!blocker_is_measured(Blocker::Op(b)), "{b:#04X}");
        }
        // **The CONTROL, and it is the whole reason the old exclusion was wrong.**
        // `19` is a *compound-assign*, and that family DOES carry a TYPE — read
        // beside the relations rather than from memory, which is what the
        // neighbourhood inference never did. If this ever came back bare the
        // capture, not the reading, would have changed.
        let at = PROBE_COMPOUND_ASSIGN
            .windows(2)
            .position(|w| w[0] == 0x19)
            .expect("no `19` in the compound-assign capture");
        assert_eq!(
            &PROBE_COMPOUND_ASSIGN[at + 1..at + 4],
            &[0x86, 0x42, 0x75],
            "`19` is a compound assign and carries a TYPE"
        );
        assert!(!BARE_BINARY_OPS.contains(&0x19));
    }

    /// The second blocker is a **construct**, not the byte the matcher stopped on —
    /// the failure `GAPS.md` §6 records, checked at the two places it would show.
    #[test]
    fn the_second_blocker_is_not_the_byte_the_matcher_stopped_on() {
        // `ui(&gA[2])`: the greedy `data-addr` designator eats the callee push, so
        // the matcher stops on a bare `BD`. The key must say `plain-call`.
        // (`DATA_ADDR` used to be this witness; WR1 admitted its bare designator,
        // and the subscript form is the one that still refuses — see
        // [`probe_block`] and §17.2 item 1.)
        let f = probe_block(DATA_ADDR_INDEX).feature();
        assert_eq!(f, "expr-call-in-expr-data-addr-1sym-then-plain-call-whole");
        assert!(!f.contains("0xBD"), "{f}");
        // The destructor-with-delete stops on a `27`, which is a byte-offset add and
        // is named as one.
        let f = probe_block(WILD_DTOR_DELETES_A_MEMBER).feature();
        assert!(f.contains("-then-off-add"), "{f}");
        assert!(!f.contains("0x27"), "{f}");
        // A member call reached through a call-argument region is a *call*, named by
        // its own form, not `op-0x26`.
        let f = probe_block(RECV_LOAD_IN_ARG).feature();
        assert!(f.contains("-then-call-nested-call"), "{f}");
    }

    /// **D5's measurement, as a test.** The symbol count is the number of data
    /// addresses the body puts in **argument registers** — not the number of `26`
    /// pushes the designator production happens to consume.
    ///
    /// The three positions a `26 <tok>` reaches [`eat_data_designator`] from are all
    /// exercised here, and only one of them counts:
    ///
    /// | witness | `26` pushes consumed | reported |
    /// |---|---:|---:|
    /// | `x = uc("hi")` ([`DATA_ADDR`]) | 3 — destination, callee, literal | **1** |
    /// | `d1("aa","bb")` ([`DATA_ADDR_TWO_SYMS`]) | 3 — callee, literal, literal | **2** |
    /// | `x = gO.m` ([`DATA_READ`]) | 2 — destination, the global | **0** |
    ///
    /// Getting this wrong is not cosmetic: the whole point of the class is to say
    /// whether the row needs one relocation pair or a `.rdata`-pool-relative
    /// selection, and an off-by-one would report the single-symbol call — which the
    /// workload does **not** contain — as the two-symbol one, which is all it
    /// contains (`docs/IL_CALL_IN_EXPR.md` §17).
    #[test]
    fn the_symbol_count_is_the_addresses_the_call_materializes() {
        let f = |seg: &[u8]| probe_block(seg).feature();
        // `x = uc("hi")` is no longer a second blocker at all — WR1 built the
        // production, and the excerpt then runs out of formals ([`probe_block`]).
        assert_eq!(f(DATA_ADDR), PARSED_PAST_THE_ARGUMENTS);
        // `x = ui(&gA[2])` still is: the subscript's offset run is a third
        // instruction and is refused (§17.2 item 1). **One symbol**, still.
        assert_eq!(f(DATA_ADDR_INDEX), "expr-call-in-expr-data-addr-1sym-then-plain-call-whole");
        // `d1("aa","bb")`'s designators now parse too, so this excerpt runs out
        // of formals. Its refusal on a whole function is `call-arg-multi-sym` —
        // graded by `fixtures/cpp/wr1_sym_addr_neg.cpp` (`t1`, `t2`).
        assert_eq!(f(DATA_ADDR_TWO_SYMS), PARSED_PAST_THE_ARGUMENTS);
        assert_eq!(f(DATA_READ), "expr-call-in-expr-data-read-whole");
        // Nothing per-TU rides along: the two literal tokens can be retagged and the
        // key does not move. Same sharding gate as every other payload here.
        let mut retagged = DATA_ADDR_TWO_SYMS.to_vec();
        assert_eq!((retagged[18], retagged[19]), (0xE8, 0x09));
        assert_eq!((retagged[32], retagged[33]), (0xE9, 0x09));
        retagged[18] = 0x41;
        retagged[19] = 0x33;
        retagged[32] = 0x77;
        retagged[33] = 0x21;
        assert_eq!(f(&retagged), PARSED_PAST_THE_ARGUMENTS);
    }

    /// **WDA — the count belongs to the construct that materializes the address,
    /// not to whichever construct happened to open the body.**
    ///
    /// D5 gated the count on the *form*, so a body whose data symbols arrive as the
    /// **second blocker** had its count computed by the same production and then
    /// thrown away. `p->M1("hi")` and `p->M2("aa","bb")` printed one identical key,
    /// and that key — `expr-call-in-expr-recv-load-then-call-data-addr-whole`, 10,540
    /// functions over 828 TUs — is the largest `-whole` member-call row on the board.
    /// One relocation pair or a whole-TU `.rdata` pool layout is the difference
    /// between §17.6 (3) and §17.6 (6), and the census could not see it.
    ///
    /// The suffix moves to the **blocker** here rather than to the form, because
    /// `recv-load-2sym` would name the wrong construct: the receiver is a pointer
    /// formal in a register and materializes no symbol at all.
    #[test]
    fn the_count_follows_the_designator_into_the_second_blocker() {
        let f = |seg| parse_segment_detail(&free_fn(seg), NO_LOCALS).unwrap_err().feature();
        assert_eq!(
            f(RECV_LOAD_ONE_SYM),
            "expr-call-in-expr-recv-load-then-call-data-addr-1sym-whole"
        );
        assert_eq!(
            f(RECV_LOAD_TWO_SYMS),
            "expr-call-in-expr-recv-load-then-call-data-addr-2sym-whole"
        );
        // Sharding gate, same as the form-owned case: the literals' tokens are
        // per-TU and retagging them must not move the key.
        let mut retagged = RECV_LOAD_TWO_SYMS.to_vec();
        assert_eq!((retagged[34], retagged[35]), (0xF8, 0x09));
        assert_eq!((retagged[48], retagged[49]), (0xF9, 0x09));
        retagged[34] = 0x41;
        retagged[35] = 0x33;
        retagged[48] = 0x77;
        retagged[49] = 0x21;
        assert_eq!(
            f(&retagged),
            "expr-call-in-expr-recv-load-then-call-data-addr-2sym-whole"
        );
    }

    /// **The under-claiming direction of the same predicate**, which is the half
    /// nothing else tests: widening *where* the count is rendered must not start
    /// printing a count on rows that materialize no symbol, and must not move a
    /// single key that D5 already prints.
    ///
    /// The three bare-`-whole` forms below reach [`mark_whole`]'s first
    /// `body_matches` and never grant anything, so `fail.syms` is 0 by construction
    /// (`eat_data_designator` has exactly two call sites and both are in
    /// [`eat_form_value`]). `RECV_LOAD_IN_ARG` is the one that would break first if
    /// the widened predicate were keyed on "any grant" instead of "the *named*
    /// second blocker is a designator": its second blocker is another plain call,
    /// not a data symbol.
    #[test]
    fn widening_the_count_prints_none_where_there_is_no_designator() {
        let f = |seg| parse_segment_detail(&free_fn(seg), NO_LOCALS).unwrap_err().feature();
        assert_eq!(f(RECV_LOAD), "expr-call-in-expr-recv-load-whole");
        assert_eq!(f(RECV_OBJECT), "expr-call-in-expr-recv-object-whole");
        assert_eq!(f(CHAINED), "expr-call-in-expr-chained-whole");
        assert_eq!(
            f(RECV_LOAD_IN_ARG),
            "expr-call-in-expr-recv-load-then-call-nested-call-whole"
        );
        assert_eq!(
            f(NESTED_CALL),
            "expr-call-in-expr-nested-call-then-call-nested-call-whole"
        );
    }

    /// **What the `-whole` / `-whole2` / `-whole3` / `-whole4` suffix counts** — the
    /// question that decides whether a rung's unit of work is a key or a receiver
    /// form, and one the key's own name does not answer.
    ///
    /// It is `need` = **the number of DISTINCT extra constructs `mark_whole` had to
    /// grant past the form** before the body parsed to its end. Not statements, not
    /// calls, not symbols. The controlled pair is one source token apart:
    ///
    /// ```text
    ///   int  one_sym()         { int x; x = uc("hi");     return x; }
    ///        -> data-addr-1sym-then-plain-call-whole                    need 1
    ///   int  one_sym_ptr(T* p) { int x; x = u3(p, "cc");  return x; }
    ///        -> data-addr-1sym-then-plain-call-and-type-ptr-whole2      need 2
    /// ```
    ///
    /// Adding one **pointer formal** — no new statement, no new call, no new symbol —
    /// takes the suffix from `-whole` to `-whole2` and the `-and-<kind>` half names
    /// the construct that was added. And the *reverse* control matters just as much:
    /// [`DATA_ADDR`] → [`DATA_ADDR_TWO_SYMS`] adds a whole extra string argument and
    /// the suffix does **not** move, because two designators are one construct.
    ///
    /// So keys sharing a receiver form but differing in tail need **different
    /// construct sets**, and the receiver form is therefore *not* the unit of work:
    /// `recv-load-then-*` spans 46 `-whole…` keys precisely because it spans 46
    /// different construct sets. The unit of work is `{form} ∪ granted`.
    #[test]
    fn the_whole_suffix_counts_granted_constructs_not_occurrences() {
        let need = |seg: &[u8]| (probe_block(seg).aux >> NEED_SHIFT) & NEED_MASK;
        // one construct granted (`plain-call`), one symbol. `DATA_ADDR` and
        // `DATA_ADDR_PTR_ARG` were the other two witnesses and WR1 admitted both,
        // so they no longer raise a CALL_IN_EXPR key at all ([`probe_block`]);
        // `DATA_ADDR_INDEX` is the one whose offset run still refuses.
        assert_eq!(need(DATA_ADDR_INDEX), 1);
        // the bare `-whole` forms grant nothing and take the WHOLE_BIT path, which
        // never writes `need` at all.
        for seg in [RECV_LOAD, RECV_OBJECT, CHAINED] {
            assert_ne!(probe_block(seg).aux & WHOLE_BIT, 0);
        }
    }

    /// The count survives the matcher's own speculation. `mark_whole` re-runs
    /// `body_matches` once per grant, and inside one run the value sequence tries the
    /// form, then the granted blocker, then the plain operand vocabulary — rewinding
    /// the cursor each time. A designator consumed by an abandoned attempt must be
    /// un-counted with it, or a body that needs two grants reports more symbols than
    /// it has. Checked on the witness that actually takes the greedy path twice.
    #[test]
    fn a_rewound_designator_is_not_counted() {
        // `u3(p, "cc")` — one string, one pointer formal, so the chain is
        // `plain-call` then `type-ptr` and `body_matches` runs three times.
        // WR1 admitted this body's bare designator, so it now walks the whole
        // argument region and stops at the formals this excerpt does not carry
        // ([`probe_block`]). The rewind rule it was the witness for is still
        // exercised — by `DATA_ADDR_INDEX`, whose offset run keeps the greedy
        // path taken twice and refused.
        assert_eq!(probe_block(DATA_ADDR_PTR_ARG).feature(), PARSED_PAST_THE_ARGUMENTS);
        assert_eq!(
            probe_block(DATA_ADDR_INDEX).feature(),
            "expr-call-in-expr-data-addr-1sym-then-plain-call-whole"
        );
    }

    /// The greedy chain must terminate, stay inside its bound, and never report a
    /// grant count it did not reach. A construct whose production fails to consume
    /// the thing the classifier named would otherwise spin forever.
    #[test]
    fn the_greedy_chain_is_bounded_and_terminates() {
        for seg in [
            RECV_LOAD,
            RECV_LOAD_IN_ARG,
            DATA_ADDR,
            DATA_READ,
            CHAINED,
            NESTED_CALL,
            BYVAL_TEMP,
            PROBE_CHAIN_IN_RETURN,
            PROBE_CHAIN_IN_ASSIGNMENT,
            PROBE_IF_ON_NAMED_OBJECT,
            // (`WILD_CHAIN_AS_RECV_LOAD` is claimed by WCO's acceptance gate
            // since it parses to the end of the body, so it no longer carries a
            // `CALL_IN_EXPR` key at all.)
            WILD_DTOR_DELETES_A_MEMBER,
            WILD_DTOR_WIDE_DESCRIPTOR,
            DTOR_MEMBER_OFF0,
            DTOR_MEMBER_OFF4,
        ] {
            let seg = free_fn(seg);
            let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
            let need = (b.aux >> NEED_SHIFT) & NEED_MASK;
            assert!(
                need == NEED_UNMEASURED || need == NEED_MORE || need as usize <= MAX_ADMIT,
                "grant count {need} out of range for {}",
                b.feature()
            );
            // The `-whole<k>` suffix and the grant count are the same number.
            let f = b.feature();
            if let Some(k) = f.strip_suffix("-whole2").map(|_| 2).or_else(|| {
                f.strip_suffix("-whole3")
                    .map(|_| 3)
                    .or_else(|| f.strip_suffix("-whole4").map(|_| 4))
            }) {
                assert_eq!(need, k, "{f}");
            }
        }
    }

    /// The `(form, blocker)` enumeration both key-space tests walk. Hoisted to
    /// one place so the completeness axis is graded over **exactly** the space
    /// the uniqueness test grades — a correspondence checked on a subset of the
    /// keys it claims to cover is not checked.
    const ALL_FORMS: [CallForm; 9] = [
        CallForm::RecvLoad,
        CallForm::RecvField,
        CallForm::RecvFieldZero,
        CallForm::RecvObject,
        CallForm::RecvIntrinsic(2113),
        CallForm::RecvIntrinsic(2119),
        CallForm::Chained,
        CallForm::DataAddr,
        CallForm::Op(0x9B),
    ];

    const ALL_BLOCKERS: [Blocker; 23] = [
        Blocker::Call(CallForm::RecvLoad),
        Blocker::Call(CallForm::RecvIntrinsic(2113)),
        Blocker::Call(CallForm::RecvIntrinsic(2119)),
        Blocker::Call(CallForm::Op(0x9B)),
        Blocker::DerefLoad,
        Blocker::TempBind,
        Blocker::Virtual,
        Blocker::Type(TypeClass::Ptr),
        Blocker::Type(TypeClass::CodePtr),
        Blocker::Type(TypeClass::IntWidth(1)),
        Blocker::Type(TypeClass::IntWidth(8)),
        Blocker::Type(TypeClass::Real),
        Blocker::Type(TypeClass::Aggregate),
        Blocker::Type(TypeClass::NotAType),
        Blocker::Plumbing(0x3A),
        Blocker::Structure(Structural::StmtLimit),
        Blocker::Op(0x5C),
        Blocker::Branch(0x38),
        Blocker::Branch(0x39),
        Blocker::ChainBind,
        Blocker::OffAdd,
        Blocker::PlainCall,
        Blocker::Eof,
    ];

    /// Every `(form, blocker, grant count)` triple must round-trip and every rendered
    /// key must be unique. A silent collision here would merge two census buckets,
    /// which is the one failure a census instrument cannot survive — and this rung
    /// widened `Block::aux` to `u64` precisely so the pair need not be squeezed.
    #[test]
    fn the_aux_packing_round_trips_every_pair() {
        let forms = ALL_FORMS;
        let blockers = ALL_BLOCKERS;
        let mut seen = std::collections::BTreeSet::new();
        for f in forms {
            let (fd, fp) = f.code();
            assert!(fd <= FORM_MASK && fp <= PAYLOAD_MASK, "{f:?}");
            // The bare and `-whole` keys, unchanged from D2.
            let base = fd | (fp << FORM_BITS);
            assert!(seen.insert(feature(base)));
            assert!(seen.insert(feature(base | WHOLE_BIT)));
            for blk in blockers {
                let (bd, bp) = blk.code();
                assert!(bd <= BLK_MASK, "{blk:?}: discriminant overflows");
                assert!(bp <= BLK_PAYLOAD_MASK, "{blk:?}: payload overflows");
                assert_eq!(Blocker::from_code(bd, bp), Some(blk), "{blk:?}");
                let pair = base | (bd << BLK_SHIFT) | (bp << BLK_PAYLOAD_SHIFT);
                for need in [NEED_UNMEASURED, 1, 2, 3, 4, NEED_MORE] {
                    let key = feature(pair | (need << NEED_SHIFT));
                    assert!(key.starts_with(CALL_IN_EXPR), "{key}");
                    assert!(seen.insert(key.clone()), "duplicate bucket name {key}");
                }
            }
        }
        // Nothing per-TU can reach any of it: the widest field is 23 bits of a
        // nested form's own (disc, payload), and a payload is only ever an intrinsic
        // selector, a type class or an opcode byte. And the whole layout fits, which
        // is why `Block::aux` is a `u64` — squeezing it would have merged buckets.
        assert!(KIND_SHIFT + KIND_BITS <= 64);
        // Every coarse kind is representable and distinctly named.
        let mut kinds = std::collections::BTreeSet::new();
        for blk in blockers {
            let k = blk.kind_code();
            assert!(k <= KIND_MASK, "{blk:?}: kind overflows its field");
            assert_ne!(k, 0, "{blk:?}: only `None` may be kind 0");
            kinds.insert(Blocker::kind_name(k));
        }
        assert!(!kinds.contains("none"));
    }

    /// **The completeness axis is graded as a CORRESPONDENCE** — roadmap §9.11 /
    /// §9.14. It maps a census key to a construct-level claim, and the oracle
    /// cannot grade that: a byte-exact obj compare says nothing about whether a
    /// *name* is true. So it is graded the three ways a correspondence can be:
    ///
    /// 1. **Agreement where the answer is independently known.** [`feature`]
    ///    already encodes the same fact in the rendered suffix, by a completely
    ///    separate code path. Over the whole enumerated key space the two must
    ///    say the same thing — and this is the check that could fail, because
    ///    [`completeness`] re-reads the bit layout rather than the string.
    /// 2. **Totality.** Every reachable `aux` gets a reading; there is no
    ///    silent hole.
    /// 3. **Injectivity of the vocabulary.** The four grammar readings are
    ///    distinct strings, so summing them cannot double-count.
    ///
    /// A renderer that returned `WholeGrammar` for everything passes none of
    /// them: assertion 1 fails on every `-more` and every suffix-less key.
    #[test]
    fn the_completeness_axis_agrees_with_the_rendered_key() {
        let forms = ALL_FORMS;
        let blockers = ALL_BLOCKERS;
        let mut readings = std::collections::BTreeSet::new();
        let mut n = 0usize;
        let mut check = |aux: u64| {
            let key = feature(aux);
            let got = completeness(aux);
            // The rendered suffix, read the way a ranking table reads it — which
            // is the thing this axis exists to stop anyone doing.
            let want = if !key.contains("-then-") {
                if key.ends_with("-whole") {
                    Complete::WholeGrammar
                } else {
                    Complete::UnmeasuredGrammar
                }
            } else if key.ends_with("-more") {
                Complete::MoreGrammar
            } else if key.ends_with("-whole")
                || key.ends_with("-whole2")
                || key.ends_with("-whole3")
                || key.ends_with("-whole4")
            {
                Complete::WholeGrammar
            } else {
                Complete::UnmeasuredGrammar
            };
            assert_eq!(got, want, "key {key} reads {got:?} but renders as {want:?}");
            readings.insert(got.name());
            n += 1;
        };
        for f in forms {
            let (fd, fp) = f.code();
            let base = fd | (fp << FORM_BITS);
            check(base);
            check(base | WHOLE_BIT);
            for blk in blockers {
                let (bd, bp) = blk.code();
                let pair = base | (bd << BLK_SHIFT) | (bp << BLK_PAYLOAD_SHIFT);
                for need in [NEED_UNMEASURED, 1, 2, 3, 4, NEED_MORE] {
                    check(pair | (need << NEED_SHIFT));
                }
            }
        }
        // Totality: the walk above covered the whole key space and every one of
        // its keys got a reading.
        assert_eq!(n, forms.len() * (2 + blockers.len() * 6));
        // Injectivity of the vocabulary actually reached.
        assert_eq!(
            readings,
            ["complete-more:grammar", "complete-unmeasured:grammar", "complete-whole:grammar"]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>(),
            "the grammar readings reached must be exactly these three"
        );
    }

    /// D4 must not have changed what the census counts, only how it names it. The
    /// four key shapes are disjoint by construction (the suffixes are mutually
    /// exclusive) and every witness lands in exactly one.
    #[test]
    fn the_key_shapes_partition_the_bucket() {
        for seg in [
            RECV_LOAD,
            RECV_DEREF,
            RECV_FIELD,
            RECV_OBJECT,
            RECV_CALL,
            RECV_INTRINSIC,
            CHAINED,
            NESTED_CALL,
            // (`DATA_ADDR` is claimed by WR1's acceptance gate — its designator
            // is a production now — so it no longer carries a `CALL_IN_EXPR` key
            // at all. `DATA_ADDR_INDEX`, whose offset run still refuses, stands
            // in for the family here.)
            DATA_ADDR_INDEX,
            DATA_READ,
            RECV_LOAD_IN_ARG,
            RECV_FIELD_OFF0,
            DTOR_MEMBER_OFF0,
            DTOR_MEMBER_OFF4,
            BYVAL_TEMP,
            WILD_DTOR_WIDE_DESCRIPTOR,
            // (`PROBE_CHAIN_IN_RETURN` is in class since WCH and raises no key.)
            PROBE_CHAIN_IN_ASSIGNMENT,
            PROBE_IF_ON_NAMED_OBJECT,
            // (`WILD_CHAIN_AS_RECV_LOAD` is claimed by WCO's acceptance gate
            // since it parses to the end of the body, so it no longer carries a
            // `CALL_IN_EXPR` key at all.)
            WILD_DTOR_DELETES_A_MEMBER,
        ] {
            let b = probe_block(seg);
            let f = b.feature();
            assert!(f.starts_with(CALL_IN_EXPR), "{f}");
            // Exactly one of the three shapes, every time:
            //   the form alone finishes the body      -> `…-<form>-whole`
            //   a second construct is named           -> `…-<form>-then-<blk>[…]`
            //   the form itself has no production     -> `…-<form>` (D2's residue)
            let whole_alone = b.aux & WHOLE_BIT != 0;
            let has_pair = f.contains("-then-");
            let bare = !whole_alone && !has_pair;
            assert_eq!(
                usize::from(whole_alone) + usize::from(has_pair) + usize::from(bare),
                1,
                "{f}"
            );
            assert_eq!(whole_alone, f.ends_with("-whole") && !has_pair, "{f}");
            // A bare key is only legal for a form with no production at all.
            let form =
                CallForm::from_code(b.aux & FORM_MASK, (b.aux >> FORM_BITS) & PAYLOAD_MASK).unwrap();
            assert!(!bare || !form_is_measured(form), "{f}");
        }
    }

    // ---------------------------------------------------------------------
    // `C2RS_SINK_MCALL_TRAILER` — lane w-5c2's counterfactual for board #1428's
    // second `0x5C` rung. See [`super::TrailerSink`].
    // ---------------------------------------------------------------------

    /// **The counterfactual sink must be OFF in every test process**, exactly as
    /// `expr.rs`'s `C2RS_SINK_CHAIN` and `assign.rs`'s `C2RS_SINK_STORE_TYPE`
    /// tripwires require of their own.
    ///
    /// A shell that exported it for a scan and then ran `cargo test` in the same
    /// session would grade a **different classifier**, and the sink emits nothing,
    /// so no gate in this repo would notice. `docs/STATUS.md` trap 5.
    #[test]
    fn the_trailer_sink_is_off_in_the_test_process() {
        assert!(
            std::env::var("C2RS_SINK_MCALL_TRAILER").is_err(),
            "the test process must not set C2RS_SINK_MCALL_TRAILER"
        );
        assert_eq!(super::trailer_sink(), super::TrailerSink::Measured);
    }

    /// The `5C` trailer grid: every TYPE spelling × every state spelling the
    /// workload and the probes are known to carry. Built once so all three arm
    /// tests grade the **same** population.
    ///
    /// The TYPEs are the bare `int` triple `w-5c`'s `userfn()` row carries, a
    /// `const int` with a per-TU id, and **the `const`-qualified POINTER
    /// `A6 43 81 20`** — which is the TYPE `w-5c`'s `probe/eh5c.cpp`
    /// `one_local()`, `two_locals()`, `across()` and `many_locals()` rows all
    /// carry, and which `eat_int_like` refuses on its `kind & 0x0F == 3`
    /// (pointer) nibble. The states are the two shipped flags, the four other
    /// short forms `w-5c` §4 lists from the workload, and the escape.
    const TRAILER_TYPES: [&[u8]; 4] = [
        &[0x86, 0x41, 0x74],
        &[0xA6, 0x41, 0x84, 0x20],
        &[0xA6, 0x43, 0x81, 0x20],
        &[0x86, 0x43, 0x83, 0x20],
    ];
    const TRAILER_STATES: [&[u8]; 7] = [
        &[0x11],
        &[0x01],
        &[0x02],
        &[0x03],
        &[0x04],
        &[0x41],
        // `w-5c` §3.2's escape, the single byte sequence all 9,645 escaped
        // workload sites carry: state 257.
        &[0x80, 0x01, 0x01, 0x00, 0x00],
    ];

    /// `5C <ty> <state> 4B`, the statement-terminal position the trailer is read at.
    fn trailer_case(ty: &[u8], state: &[u8]) -> Vec<u8> {
        let mut v = vec![0x5C];
        v.extend_from_slice(ty);
        v.extend_from_slice(state);
        v.push(0x4B);
        v
    }

    /// **`Measured ⊆ Flag ⊆ Varint`, on acceptance AND on the final cursor.**
    ///
    /// The relation is the load-bearing property of a counterfactual instrument:
    /// the only number the lane reports is a *recovery*, and a wider arm that
    /// refused something a narrower one takes could only lower it — in the one
    /// direction nobody would look. The cursor half matters too: an arm that
    /// accepts the same bytes but stops four bytes inside the escape's `LE32`
    /// would desync the next statement and scatter the row, which is exactly the
    /// failure `docs/IL_CALL_IN_EXPR.md` §14.2's flat-hex-tail caution describes.
    #[test]
    fn the_trailer_sink_arms_are_nested() {
        use super::TrailerSink::*;
        for ty in TRAILER_TYPES {
            for st in TRAILER_STATES {
                let seg = trailer_case(ty, st);
                let mut got = Vec::new();
                for arm in [Measured, Flag, Varint] {
                    let mut p = 0usize;
                    got.push(if super::eat_dtor_stmt_trailer_with(arm, &seg, &mut p) {
                        Some(p)
                    } else {
                        assert_eq!(p, 0, "a refusing arm must rewind: {arm:?} {seg:02X?}");
                        None
                    });
                }
                for w in got.windows(2) {
                    if let Some(narrow) = w[0] {
                        assert_eq!(
                            w[1],
                            Some(narrow),
                            "arms are not nested on {seg:02X?}: {got:?}"
                        );
                    }
                }
                // Whatever an arm takes, it takes the WHOLE trailer: the cursor
                // lands on the `4B` that closes the statement.
                for g in got.into_iter().flatten() {
                    assert_eq!(seg[g], 0x4B, "arm stopped inside the trailer: {seg:02X?}");
                }
            }
        }
    }

    /// The default arm is the gate that shipped — the two measured flags, an
    /// int-like TYPE, and nothing else — and the widest arm is `w-5c`'s anchored
    /// width. Pinned over the same grid, as a table, so a regression re-prices
    /// board #1428's row silently and this is what says so.
    #[test]
    fn the_trailer_arms_take_exactly_what_their_doc_claims() {
        use super::TrailerSink::*;
        let takes = |arm, ty: &[u8], st: &[u8]| {
            let seg = trailer_case(ty, st);
            let mut p = 0usize;
            super::eat_dtor_stmt_trailer_with(arm, &seg, &mut p)
        };
        let int = TRAILER_TYPES[0];
        let cst = TRAILER_TYPES[1];
        let ptr = TRAILER_TYPES[2];
        let esc = TRAILER_STATES[6];

        // `Measured`: the two flags on an int-like TYPE, and refusal everywhere else.
        assert!(takes(Measured, int, &[0x11]) && takes(Measured, int, &[0x01]));
        assert!(takes(Measured, cst, &[0x01]), "a `const int` TYPE is int-like");
        for st in [&[0x02u8][..], &[0x03][..], &[0x04][..], &[0x41][..], esc] {
            assert!(!takes(Measured, int, st), "the shipped gate takes two flags only");
        }
        // **The finding the flag half of this row's published description misses.**
        // `IL_CALL_IN_EXPR.md` §16.2 calls `op-0x5C` *"a destructor statement
        // trailer whose flag is neither measured value"*. It is also, and more
        // often, a trailer whose **TYPE** is a pointer: `eat_int_like` requires
        // `kind & 0x0F ∈ {1, 2}` and every `A6 43 …` trailer in `w-5c`'s own
        // probe carries `3`. The flag on those rows is a perfectly ordinary `01`.
        assert!(!takes(Measured, ptr, &[0x01]), "the shipped gate's TYPE is int-like");

        // `Flag`: the whitelist gone, the TYPE gate and the escape still in place.
        assert!(takes(Flag, int, &[0x02]) && takes(Flag, int, &[0x41]));
        assert!(!takes(Flag, int, esc), "`flag` is the SHORT form only");
        assert!(!takes(Flag, ptr, &[0x01]), "`flag` keeps the int-like TYPE gate");

        // `Varint`: `w-5c`'s width — any TYPE, any state, escape included.
        assert!(takes(Varint, ptr, &[0x02]));
        assert!(takes(Varint, int, esc), "the 9,645-site workload escape");
        // Not a licence to eat anything: the token still has to be a `5C` with a
        // readable TYPE after it, which is the 100.00 % `w-5c` §2.2 measured.
        let mut p = 0;
        assert!(!super::eat_dtor_stmt_trailer_with(Varint, &[0x5C, 0x01, 0x01, 0x4B], &mut p));
        assert_eq!(p, 0);
    }

    /// **The mechanism claim, pinned on a body: `…-then-op-0x5C` IS this gate.**
    ///
    /// [`DTOR_MEMBER_OFF0`] is `expr-call-in-expr-recv-field-off0-whole` — a
    /// complete body — and the only thing separating it from the census key board
    /// #1428 measured at 1,212 functions is **one byte**, the trailer's flag.
    /// Change `11` to `02` and the same body files as
    /// `…-recv-field-off0-then-op-0x5C`.
    ///
    /// This is what makes the row a *diagnostic* finding rather than a rung: the
    /// body did not become harder to compile, and nothing about the port moved.
    /// A classifier narrower than the tree's own reader of the byte re-labelled
    /// it.
    #[test]
    fn the_trailer_flag_alone_turns_a_whole_body_into_the_op_0x5c_key() {
        let base = DTOR_MEMBER_OFF0.to_vec();
        assert_eq!(
            parse_segment_detail(&base, NO_LOCALS).unwrap_err().feature(),
            "expr-call-in-expr-recv-field-off0-whole"
        );
        let at = crate::func::readers::find_subslice(&base, &[0x5C, 0x86, 0x41, 0x74, 0x11])
            .expect("the probe carries the statement trailer");
        let mut moved = base.clone();
        moved[at + 4] = 0x02;
        assert_eq!(
            parse_segment_detail(&moved, NO_LOCALS).unwrap_err().feature(),
            "expr-call-in-expr-recv-field-off0-then-op-0x5C",
            "the flag whitelist is what raises the key"
        );
        // And the sink's widest arm is exactly what closes that gap — checked on
        // the trailer itself, since the classifier reads the environment once per
        // process and a test cannot exercise two arms in one.
        let seg = &moved[at..];
        let mut p = 0usize;
        assert!(!super::eat_dtor_stmt_trailer_with(super::TrailerSink::Measured, seg, &mut p));
        let mut p = 0usize;
        assert!(super::eat_dtor_stmt_trailer_with(super::TrailerSink::Varint, seg, &mut p));
        assert_eq!(seg[p], 0x4B);
    }

    /// Locate the `26` the census reports for the assignment-body probes: the
    /// second one, past the destination push.
    fn find_first_26_in_rhs(seg: &[u8]) -> usize {
        let lo = crate::func::readers::find_subslice(seg, &[0x4C, 0x4F, 0x11]).unwrap();
        let first = lo + 4; // past `LO` and the `53`
        assert_eq!(seg[first], 0x26);
        let (_, w) = read_token_var(seg, first + 1).unwrap();
        first + 1 + w
    }

    // ---- w-value: the member-call VALUE model ---------------------------------

    /// The walk consumes the **whole** production and lands exactly on the byte
    /// after the `4C`, and it reports the class the `BD`'s return TYPE names.
    ///
    /// The landing offset is the half that matters: a walker that reported the
    /// right class from the wrong cursor would hand `parse_expr` a stream it
    /// then mis-tokenizes, and every downstream key would be about a byte
    /// nobody chose. Checked against the token that FOLLOWS the production in
    /// each capture rather than against a hard-coded index.
    #[test]
    fn the_value_model_consumes_a_whole_member_call_and_lands_after_its_4c() {
        for (seg, want, next) in [
            // `x = p->Get();` — `BD 86 41 74` is `int`.
            (RECV_LOAD, CallValue::Value(ValueClass::Int4), 0x32),
            // a named-object receiver, `26 <sym> 2C <ptr> 00` then the bind.
            (RECV_OBJECT, CallValue::Value(ValueClass::Int4), 0x32),
            // `x = w->p->Get();` — the receiver is read from memory.
            (RECV_DEREF, CallValue::Value(ValueClass::Int4), 0x32),
        ] {
            let mut p = find_first_26_in_rhs(seg);
            assert_eq!(seg[p], 0x26);
            let got = eat_call_value(seg, &mut p);
            assert_eq!(got, Some(want), "{:?}", &seg[..8]);
            assert_eq!(seg[p - 1], 0x4C, "the cursor lands past the closing 4C");
            assert_eq!(seg[p], next, "and on the token the capture has there");
        }
    }

    /// A **chain** is one production, not two: the walk must not stop at the
    /// inner `4C`, because the value it closes is bound as the next receiver by
    /// the `99` immediately after it.
    ///
    /// This test is the one that found the rule. The first build of the walker
    /// returned at the first depth-0 `4C` and handed `parse_expr` a `99` it has
    /// no arm for; `a_chain_is_a_chain_in_both_statement_positions` went red and
    /// [`binds_the_result`] is the repair.
    #[test]
    fn the_value_model_walks_through_a_chain_rather_than_stopping_at_the_inner_4c() {
        let mut p = find_first_26_in_rhs(CHAINED);
        // There are TWO `4C`s closing `BD` regions in this body's production.
        let got = eat_call_value(CHAINED, &mut p);
        assert_eq!(got, Some(CallValue::Value(ValueClass::Int4)));
        assert_eq!(CHAINED[p - 1], 0x4C);
        assert_eq!(CHAINED[p], 0x32, "the store the assignment ends with");
        // …and the inner `4C` really is passed over: the production it closed is
        // followed by a bind, which is what [`binds_the_result`] keys on.
        let inner = CHAINED
            .iter()
            .position(|&b| b == 0x4C)
            .map(|i| i + 1 + CHAINED[i + 1..].iter().position(|&b| b == 0x4C).unwrap())
            .unwrap();
        assert!(inner < p - 1);
        assert!(binds_the_result(CHAINED, inner + 1));
    }

    /// The walk **declines** what is not a `BD`-rooted call production, and
    /// leaves the caller's cursor exactly where it found it — so the fallback is
    /// byte-for-byte the refusal this arm has always raised.
    ///
    /// Two different declines, on purpose: a global object's member **read**
    /// (`x = gO.m;`) opens on a `26` and never reaches a `BD` at all, and a `9B`
    /// temporary receiver is a token this walker has no width for. The second is
    /// **69 % of the model's whole price on the 878-TU workload** (1,590 of
    /// 2,306 emitted), which is why it has a cell in
    /// `fixtures/cpp/wvalue_call_value_neg.cpp` as well as a line here.
    ///
    /// [`DATA_ADDR_INDEX`] is deliberately **not** in this list, and finding out
    /// why cost this test one red run: `x = u2(gA[0])` is a *plain* call
    /// (`26 <u2> BD …`), so the walker consumes it correctly and the probe named
    /// after the data-address family is not a witness for declining one.
    #[test]
    fn the_value_model_declines_what_is_not_a_call_and_moves_no_cursor() {
        for seg in [DATA_READ, BYVAL_TEMP] {
            let at = find_first_26_in_rhs(seg);
            let mut p = at;
            assert_eq!(eat_call_value(seg, &mut p), None, "{:?}", &seg[..8]);
            assert_eq!(p, at, "a decline may not move the cursor");
        }
    }

    /// `void` and "a class this parser does not model" are **two answers**, and
    /// the caller does two different things with them: `Void` pushes nothing and
    /// leaves `cstack_ok` true, `Opaque` pushes a placeholder and clears it.
    ///
    /// Merging them was measured and was wrong in BOTH directions. Treating
    /// `Opaque` as "push nothing" moved `expr-convert-no-value-0x2C` — board
    /// #1462's key — from 4,973 to 5,790 bodies on the 878-TU workload, by
    /// under-reporting the model's own stack depth. Treating `Void` as `Opaque`
    /// would clear `cstack_ok` on the one token this model follows exactly.
    #[test]
    fn a_void_return_and_an_unmodeled_one_are_two_answers() {
        // `82 07 03` — class nibble 7, the `void` of `docs/IL_LOAD_TYPES.md` §1,
        // and the TYPE `NESTED_CALL`'s sibling captures carry verbatim.
        assert_eq!(call_value_of(0x82, 0x07), CallValue::Void);
        // `86 41 74` int, `86 43 81` a data pointer — the two modeled classes.
        assert_eq!(call_value_of(0x86, 0x41), CallValue::Value(ValueClass::Int4));
        assert_eq!(call_value_of(0x86, 0x43), CallValue::Value(ValueClass::Ptr4));
        // `86 45` is `float` and `88 48` an 8-byte real: consumed by width,
        // class not modeled.
        assert_eq!(call_value_of(0x86, 0x45), CallValue::Opaque);
        assert_eq!(call_value_of(0x88, 0x48), CallValue::Opaque);
    }

    /// **THE ACCEPTANCE THEOREM, as a test rather than as a comment.**
    ///
    /// Every path that reaches byte `0x26` inside `parse_expr` returned `Err`
    /// before the value model and must return `Err` after it: the arm pushes no
    /// [`IlOp`] and the end-of-walk poison re-raises [`classify`]. So every one
    /// of this module's captured probes must still be a refusal, and none of
    /// them may have become `expr-empty-*` — the guard the poison had to be
    /// placed ahead of.
    #[test]
    fn the_value_model_never_turns_a_refusal_into_an_acceptance() {
        for seg in [
            RECV_LOAD, RECV_DEREF, RECV_FIELD, RECV_OBJECT, RECV_CALL, RECV_INTRINSIC, CHAINED,
            NESTED_CALL, DATA_ADDR_INDEX, DATA_READ, RECV_LOAD_IN_ARG, RECV_FIELD_OFF0,
            DTOR_MEMBER_OFF0, DTOR_MEMBER_OFF4, BYVAL_TEMP, WILD_DTOR_WIDE_DESCRIPTOR,
            PROBE_CHAIN_IN_ASSIGNMENT, PROBE_IF_ON_NAMED_OBJECT, WILD_DTOR_DELETES_A_MEMBER,
        ] {
            let s = free_fn(seg);
            let r = parse_segment_detail(&s, NO_LOCALS);
            assert!(r.is_err(), "the value model may not accept: {:?}", &seg[..8]);
            let f = r.unwrap_err().feature();
            assert!(
                !f.starts_with("expr-empty"),
                "the poison must precede the empty-ops guard, got {f}"
            );
        }
        // …and the same statement about the port, not just the census: nothing
        // in this set may reach `parse_segment`'s accepting path either.
        for seg in [RECV_LOAD, CHAINED, NESTED_CALL, BYVAL_TEMP] {
            assert!(parse_segment(&free_fn(seg), NO_LOCALS).is_none());
        }
    }
}
