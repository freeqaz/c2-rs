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
};
use super::{Block, BODY_SCOPE_DEPTH};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, read_token_var, read_type, read_varint,
};

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
// `Block` carries one `u32` of context and `ctx` is a `&'static str`, so the
// selector id and the residue opcode cannot go in the name. They go here:
//   bits  0..5   the form's discriminant
//   bits  6..22  its payload (an intrinsic selector, or a residue opcode byte)
//   bit   23     the whole-body-completeness bit
// Nothing per-TU is representable in that layout, which is the sharding gate
// stated as an invariant rather than as a promise.

const FORM_BITS: u32 = 6;
const FORM_MASK: u32 = (1 << FORM_BITS) - 1;
const PAYLOAD_BITS: u32 = 17;
const PAYLOAD_MASK: u32 = (1 << PAYLOAD_BITS) - 1;
const WHOLE_BIT: u32 = 1 << (FORM_BITS + PAYLOAD_BITS);

impl CallForm {
    /// `(discriminant, payload)`.
    fn code(self) -> (u32, u32) {
        match self {
            CallForm::RecvLoad => (1, 0),
            CallForm::RecvDeref => (2, 0),
            CallForm::RecvField => (3, 0),
            CallForm::RecvFieldZero => (17, 0),
            CallForm::RecvObject => (4, 0),
            CallForm::RecvCall => (5, 0),
            CallForm::RecvIntrinsic(sel) => (6, sel as u32 & PAYLOAD_MASK),
            CallForm::RecvOther => (7, 0),
            CallForm::Chained => (8, 0),
            CallForm::NestedCall => (9, 0),
            CallForm::DataAddr => (10, 0),
            CallForm::DataRead => (11, 0),
            CallForm::Intrinsic(sel) => (13, sel as u32 & PAYLOAD_MASK),
            CallForm::Other => (14, 0),
            CallForm::Op(b) => (15, b as u32),
            CallForm::Eof => (16, 0),
        }
    }

    fn from_code(disc: u32, payload: u32) -> Option<CallForm> {
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

/// The census key for a [`Block`] this module raised: the sub-bucket, plus
/// `-whole` when the rigid whole-body matcher accounted for the entire segment.
pub(crate) fn feature(aux: u32) -> String {
    let disc = aux & FORM_MASK;
    let payload = (aux >> FORM_BITS) & PAYLOAD_MASK;
    let name = match CallForm::from_code(disc, payload) {
        Some(f) => f.name(),
        // Unreachable by construction; a bucket rather than a panic, since this
        // is a diagnostic path and a census must never take the process down.
        None => format!("aux-{aux:X}"),
    };
    if aux & WHOLE_BIT != 0 {
        format!("{CALL_IN_EXPR}-{name}-whole")
    } else {
        format!("{CALL_IN_EXPR}-{name}")
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
        aux: disc | (payload << FORM_BITS),
    }
}

/// Set the whole-body-completeness bit on a block this module raised, when the
/// **entire** segment would parse with that one form admitted.
///
/// Called from [`super::parse_segment_detail`], which is the only place that has
/// both the block and the `LO` offset. Diagnostic only: the `Err` stays an `Err`.
pub(crate) fn mark_whole(seg: &[u8], lo: usize, b: Block) -> Block {
    let disc = b.aux & FORM_MASK;
    let payload = (b.aux >> FORM_BITS) & PAYLOAD_MASK;
    let Some(form) = CallForm::from_code(disc, payload) else {
        return b;
    };
    if whole_body_is_one_value(seg, lo, form) {
        Block { aux: b.aux | WHOLE_BIT, ..b }
    } else {
        b
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
            return classify_at(Stop::Eof, last, head_syms);
        };
        // Decisive bytes first, at the top level only.
        if depth == 0 {
            match b {
                0x99 => return classify_at(Stop::Bind, last, head_syms),
                0x32 => return classify_at(Stop::Store, last, head_syms),
                0x55 => return classify_at(Stop::ArgEnd, last, head_syms),
                0x41 | 0x4B => return classify_at(Stop::StmtEnd, last, head_syms),
                _ => {}
            }
        }
        let mut consumed_head_sym = false;
        match b {
            0xB9 => {
                p += 1;
                let Some((_, w)) = read_token_var(seg, p) else {
                    return CallForm::Eof;
                };
                p += w;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return CallForm::Eof;
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
                        return CallForm::Eof;
                    }
                    // the `40 <TYPE>` intrinsic call token — no trailing field
                    // (`docs/IL_INTRINSIC_CALL.md` §1).
                    p += 1;
                    let Some((_, _, _, tw)) = read_type(seg, p) else {
                        return CallForm::Eof;
                    };
                    p += tw;
                    depth += 1;
                    open.push((true, sel));
                    last = None;
                } else {
                    p += 1;
                    let Some((_, _, _, tw)) = read_type(seg, p) else {
                        return CallForm::Eof;
                    };
                    lit_zero = seg.get(p + tw) == Some(&0x00);
                    if !eat_literal(seg, &mut p) {
                        return CallForm::Eof;
                    }
                    last = Some(Tk::Lit);
                }
            }
            0x26 => {
                let mut q = p + 1;
                let Some((_, w)) = read_token_var(seg, q) else {
                    return CallForm::Eof;
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
                    return CallForm::Eof;
                };
                p += tw;
                // the calling-convention byte, then the per-TU function-type id
                p += 1;
                if read_varint(seg, &mut p).is_none() {
                    return CallForm::Eof;
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
                    _ => return CallForm::Op(0x4C),
                }
            }
            0x55 => {
                // depth > 0 (the depth-0 case returned above): an argument
                // terminator, `55 <TYPE>`.
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return CallForm::Eof;
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
                    return CallForm::Eof;
                };
                p += tw + 1;
            }
            0x27 => {
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return CallForm::Eof;
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
                    return CallForm::Op(0x28);
                }
                last = Some(Tk::OffAdd(lit_zero));
            }
            0x30 => {
                p += 1;
                let Some((_, _, _, tw)) = read_type(seg, p) else {
                    return CallForm::Eof;
                };
                p += tw;
                last = Some(Tk::Deref);
            }
            0x66 => {
                // The class-pair descriptor of the 2113–2119 family. Not a value.
                if !eat_class_descriptor(seg, &mut p) {
                    return CallForm::Op(0x66);
                }
            }
            0x02 | 0x03 | 0x04 => {
                p += 1;
                last = Some(Tk::Op);
            }
            other => return classify_at(Stop::Op(other), last, head_syms),
        }
        if counting_head {
            if consumed_head_sym {
                head_syms += 1;
            } else {
                counting_head = false;
            }
        }
    }
    CallForm::Eof
}

/// Turn `(what stopped the walk, the value on top, the head symbol run)` into a
/// sub-bucket.
fn classify_at(stop: Stop, last: Option<Tk>, head_syms: usize) -> CallForm {
    // How many of the head symbol pushes were *methods*. When the receiver is
    // itself a named object it is the last of the run, so one of them is not a
    // method; otherwise all of them are.
    let methods = if last == Some(Tk::Sym) {
        head_syms.saturating_sub(1)
    } else {
        head_syms
    };
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
fn whole_body_is_one_value(seg: &[u8], lo: usize, form: CallForm) -> bool {
    // Forms with no production in [`eat_form_value`] are UNMEASURED, and saying so
    // by returning early keeps "0 of N complete" from being read as a measurement.
    if !form_is_measured(form) {
        return false;
    }
    let mut p = lo + 3;
    if !eat_byte(seg, &mut p, 0x53) {
        return false;
    }
    let mut depth = BODY_SCOPE_DEPTH;
    // A statement count bound: a body this long is not one this rung can vouch
    // for, and an unbounded loop over a corrupt stream is not acceptable in an
    // instrument that runs over 2.4 M functions.
    const MAX_STMTS: usize = 64;
    for _ in 0..MAX_STMTS {
        if eat_scopes(seg, &mut p, &mut depth).is_err() {
            return false;
        }
        // A void return opens directly on the plumbing's `3A` — no expression.
        if seg.get(p) == Some(&0x3A) {
            return eat_body_end(seg, &mut p, depth);
        }
        // An expression, optionally preceded by an assignment destination push.
        // Tried in that order and on a copy of the cursor, because a statement
        // opening on `26` is ambiguous: it is a destination for `x = p->M();` and
        // a *method* push for `p->M();`, and only trying the expression settles
        // it. (`26 <dst>` is not itself a value here — a data symbol is only
        // admitted when `form` is one of the data designators.)
        let save = p;
        if !eat_value_seq(seg, &mut p, form) {
            p = save;
            if !eat_byte(seg, &mut p, 0x26) {
                return false;
            }
            match read_token_var(seg, p) {
                Some((_, w)) => p += w,
                None => return false,
            }
            if !eat_value_seq(seg, &mut p, form) {
                return false;
            }
        }
        // A store, when the statement has one.
        if eat_byte(seg, &mut p, 0x32) && !eat_scalar_type(seg, &mut p) {
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
        if !eat_byte(seg, &mut p, 0x41) || !eat_scalar_type(seg, &mut p) {
            return false;
        }
        return eat_body_end(seg, &mut p, depth);
    }
    false
}

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
fn eat_dtor_stmt_trailer(seg: &[u8], p: &mut usize) -> bool {
    let save = *p;
    if !eat_byte(seg, p, 0x5C) {
        return false;
    }
    if !eat_int_like(seg, p) {
        *p = save;
        return false;
    }
    match seg.get(*p) {
        Some(&f) if TRAILER_FLAGS.iter().any(|&(s, _)| s == f) => {
            *p += 1;
            true
        }
        _ => {
            *p = save;
            false
        }
    }
}

/// The return plumbing, with the generated destructor's `5E <n> <g> 4B`
/// sub-object trailer optionally wedged between the `29` return and the function
/// tail. `eat_return_plumbing` cannot do that (D1 hand-rolls the same split for
/// the same reason), so the branch/close/return run is walked here and the tail is
/// shared.
fn eat_body_end(seg: &[u8], p: &mut usize, depth: usize) -> bool {
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
fn eat_value_seq(seg: &[u8], p: &mut usize, form: CallForm) -> bool {
    let mut n = 0;
    loop {
        let save = *p;
        if eat_form_value(seg, p, form) {
            n += 1;
            continue;
        }
        *p = save;
        match seg.get(*p) {
            Some(&0xB9) => {
                *p += 1;
                match read_token_var(seg, *p) {
                    Some((_, w)) => *p += w,
                    None => return false,
                }
                if !eat_scalar_type(seg, p) {
                    return false;
                }
            }
            Some(&0x33) => {
                let Some((tag, kind, _, _)) = read_type(seg, *p + 1) else {
                    return false;
                };
                *p += 1;
                if !eat_scalar_type(seg, p) || !eat_literal_payload(seg, p, tag, kind) {
                    return false;
                }
            }
            Some(&0x2C) => {
                *p += 1;
                if !eat_scalar_type(seg, p) || seg.get(*p).is_none() {
                    return false;
                }
                *p += 1;
            }
            Some(&0x02) | Some(&0x03) | Some(&0x04) => *p += 1,
            _ => return n > 0,
        }
        n += 1;
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
fn eat_class_descriptor(seg: &[u8], p: &mut usize) -> bool {
    if !eat_byte(seg, p, 0x66) {
        return false;
    }
    let Some(&n) = seg.get(*p) else {
        return false;
    };
    *p += 1;
    for _ in 0..n {
        if !eat_leb(seg, p) {
            return false;
        }
    }
    true
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
/// leaves lower (`ValueClass` in `shapes.rs`). A float, a narrow integer or an
/// aggregate here is a second missing production, so it refuses and the body is
/// not counted complete.
fn eat_scalar_type(seg: &[u8], p: &mut usize) -> bool {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) => {
            let int4 = matches!(kind & 0x0F, 0x1 | 0x2) && (kind >> 4) == 4 && (tag & 0x0F) == 0x6;
            let ptr = matches!(kind & 0x0F, 0x3 | 0x4);
            if int4 || ptr {
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
fn eat_form_value(seg: &[u8], p: &mut usize, form: CallForm) -> bool {
    match form {
        CallForm::RecvLoad
        | CallForm::RecvDeref
        | CallForm::RecvField
        | CallForm::RecvFieldZero
        | CallForm::RecvObject
        | CallForm::RecvCall
        | CallForm::RecvIntrinsic(_) => eat_member_call(seg, p, form),
        CallForm::NestedCall => eat_plain_call(seg, p),
        CallForm::DataAddr => eat_data_designator(seg, p, false),
        CallForm::DataRead => eat_data_designator(seg, p, true),
        _ => false,
    }
}

/// `26 <method> <receiver of `form`> 99 <T> 00 BD <ret> <conv> <id> (<arg> 55 <T>)* 4C`
/// — exactly **one** method symbol, so a chain cannot slip through.
fn eat_member_call(seg: &[u8], p: &mut usize, form: CallForm) -> bool {
    if !eat_byte(seg, p, 0x26) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    if !eat_receiver(seg, p, form) {
        return false;
    }
    // The member bind. `99` is DIRECT dispatch by construction: virtual dispatch
    // is opcode `67` with a `9A` bind (§3), which is what licenses reading this
    // as a branch to a named callee at all.
    if !eat_byte(seg, p, 0x99) || !eat_type(seg, p) || !eat_byte(seg, p, 0x00) {
        return false;
    }
    eat_call_and_args(seg, p)
}

/// `26 <fn> BD … (<arg> 55 <T>)* 4C`.
fn eat_plain_call(seg: &[u8], p: &mut usize) -> bool {
    if !eat_byte(seg, p, 0x26) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    eat_call_and_args(seg, p)
}

/// The `BD` CALL token and its explicit-argument region. Each argument must be an
/// **already-modeled** int-like operand stream, so a body needing a second new
/// production is not counted complete.
fn eat_call_and_args(seg: &[u8], p: &mut usize) -> bool {
    if !eat_byte(seg, p, 0xBD) || !eat_type(seg, p) {
        return false;
    }
    // cdecl only: the one calling convention every captured member call carries.
    if !eat_byte(seg, p, 0x00) || read_varint(seg, p).is_none() {
        return false;
    }
    loop {
        match seg.get(*p) {
            Some(&0x4C) => {
                *p += 1;
                return true;
            }
            Some(_) => {
                if !eat_int_operands(seg, p) {
                    return false;
                }
                if !eat_byte(seg, p, 0x55) || !eat_type(seg, p) {
                    return false;
                }
            }
            None => return false,
        }
    }
}

/// One or more `B9 <tok> <int-like>` / `33 <int-like> <k>` / `02|03|04` tokens —
/// the operand vocabulary `parse_expr` already accepts, and nothing else.
fn eat_int_operands(seg: &[u8], p: &mut usize) -> bool {
    let mut n = 0;
    loop {
        match seg.get(*p) {
            Some(&0xB9) => {
                let save = *p;
                *p += 1;
                let Some((_, w)) = read_token_var(seg, *p) else {
                    return false;
                };
                *p += w;
                if !eat_int_like(seg, p) {
                    *p = save;
                    return n > 0;
                }
            }
            Some(&0x33) => {
                let save = *p;
                *p += 1;
                if !eat_int_like(seg, p) {
                    *p = save;
                    return n > 0;
                }
                if read_varint(seg, p).is_none() {
                    return false;
                }
            }
            Some(&0x02) | Some(&0x03) | Some(&0x04) => *p += 1,
            _ => return n > 0,
        }
        n += 1;
    }
}

/// The receiver designator of each named form, and only that form's.
fn eat_receiver(seg: &[u8], p: &mut usize, form: CallForm) -> bool {
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
        CallForm::RecvCall => eat_plain_call(seg, p),
        CallForm::RecvIntrinsic(sel) => eat_intrinsic_receiver(seg, p, sel),
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
fn eat_intrinsic_receiver(seg: &[u8], p: &mut usize, sel: i32) -> bool {
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
    if !eat_class_descriptor(seg, p) {
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
                if !eat_int_operands(seg, p) {
                    return false;
                }
            }
            None => return false,
        }
    }
}

/// A data symbol used as an address (`want_load` false) or read (`true`):
/// `26 <sym> [2C …] [33 <k> 27|28 …] [2C …] [30 <TYPE> [2C …]]`.
fn eat_data_designator(seg: &[u8], p: &mut usize, want_load: bool) -> bool {
    if !eat_byte(seg, p, 0x26) {
        return false;
    }
    let Some((_, w)) = read_token_var(seg, *p) else {
        return false;
    };
    *p += w;
    if !eat_opt_convert(seg, p) || !eat_opt_off_add(seg, p) || !eat_opt_convert(seg, p) {
        return false;
    }
    let loaded = eat_byte(seg, p, 0x30) && eat_type(seg, p);
    if loaded != want_load {
        return false;
    }
    eat_opt_convert(seg, p)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail};
    use crate::func::test_fixtures::{free_fn, NO_LOCALS};

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
            (CHAINED, "expr-call-in-expr-chained"),
            (NESTED_CALL, "expr-call-in-expr-nested-call"),
            (DATA_ADDR, "expr-call-in-expr-data-addr"),
            (DATA_ADDR_INDEX, "expr-call-in-expr-data-addr"),
            (DATA_READ, "expr-call-in-expr-data-read-whole"),
            (RECV_LOAD_IN_ARG, "expr-call-in-expr-recv-load"),
        ];
        for (seg, want) in cases {
            let seg = free_fn(seg);
            let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
            assert_eq!(b.feature(), *want);
            assert_eq!(seg[b.off], 0x26, "{want}: reported at the `26`");
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
        assert!(eat_class_descriptor(&seg, &mut p));
        assert_eq!(p - at, 8);
        assert_eq!(seg[p], 0x55, "the descriptor must end on the argument push");
        // Stepping a fixed four bytes lands inside the second ref instead.
        assert_ne!(seg[at + 4], 0x55);
        // …and the narrow probe agrees with the same reader, at 2 + 2 + 2.
        let narrow = free_fn(RECV_INTRINSIC);
        let at = narrow.windows(2).position(|w| w == [0x66, 0x02]).unwrap();
        let mut p = at;
        assert!(eat_class_descriptor(&narrow, &mut p));
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
        assert_eq!(b.feature(), "expr-call-in-expr-chained");
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
        assert_eq!(b.feature(), "expr-call-in-expr-recv-load");
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
        assert_eq!(b.feature(), "expr-call-in-expr-recv-load");
        // …and an extra statement after the call is not a whole body either.
        let mut v = RECV_LOAD.to_vec();
        let at = v.windows(2).position(|w| w == [0x32, 0x86]).unwrap();
        v.splice(at..at, [0x4B].iter().copied());
        let b = parse_segment_detail(&free_fn(&v), NO_LOCALS).unwrap_err();
        assert!(!b.feature().ends_with("-whole"));
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

    /// Locate the `26` the census reports for the assignment-body probes: the
    /// second one, past the destination push.
    fn find_first_26_in_rhs(seg: &[u8]) -> usize {
        let lo = crate::func::readers::find_subslice(seg, &[0x4C, 0x4F, 0x11]).unwrap();
        let first = lo + 4; // past `LO` and the `53`
        assert_eq!(seg[first], 0x26);
        let (_, w) = read_token_var(seg, first + 1).unwrap();
        first + 1 + w
    }
}
