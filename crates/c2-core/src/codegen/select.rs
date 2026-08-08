//! Dispatch, and the two helpers every module shares.
//!
//! `select_function` is the ordered match a rung adds one line to, and the
//! order is **load-bearing and documented per adjacency** — framed before tail
//! before leaves, compare before float before load before identity before addr.
//! `docs/ARCHITECTURE_SEAMS.md` §3.1 states why this is a match and not a
//! registration table: a table hides the order behind data, and the order is
//! the thing a reviewer has to read. Keep every arm's one-line "why this
//! adjacency" comment attached to the arm it orders.
//!
//! `function_gate` runs `select_function` **itself**, never a copy — the
//! census/gate cross-check (`crates/c2-harness/tests/census_gate.rs`) is only
//! meaningful because there is one decision procedure here, not two.

use c2_il::IlFunction;
use crate::BackendError;
use crate::codegen::calls;
use crate::codegen::calls::{call_seq_parts, int_tail_call_text, permute_args_text};
use crate::codegen::cond_tail::{cond_pair_parts, CondPairParts};
use crate::codegen::encode::encode_blr;
use crate::codegen::leaf::addr::addr_leaf_text;
use crate::codegen::leaf::compare::{cmp_shift_or_text, compare_leaf_text};
use crate::codegen::leaf::float::{
    FpConstRef, float_leaf_text, fp_permute_args_text, fp_tail_call_text,
};
use crate::codegen::leaf::load::indirect_load_text;
use crate::codegen::div_mod_leaf::div_mod_leaf_text;
use crate::codegen::ptr_walk_chain_loop::ptr_walk_chain_loop_text;
use crate::codegen::alloc_init_or_fail::alloc_init_or_fail_text;
use crate::codegen::osf_handle_guard::osf_handle_guard_text;
use crate::codegen::guard_chain_shared_tail::guard_chain_shared_tail_text;
use crate::codegen::if_call_join::if_call_join_text;
use crate::codegen::ptr_walk_loop::ptr_walk_loop_text;
use crate::codegen::leaf::store::store_leaf_text;
use crate::codegen::straightline::select_text;

/// True iff `k` fits PPC's 16-bit signed immediate field (`addi`/`subf` imm).
pub(crate) fn fits_i16(k: i32) -> bool {
    (-0x8000..=0x7FFF).contains(&k)
}

pub(crate) fn out_of_class(msg: &str) -> BackendError {
    BackendError::NotImplemented(msg.to_string())
}

/// Integer argument registers, left-to-right (Xbox 360 PPC / MSVC ABI).
pub(crate) const ARG_REGS: [u8; 8] = [3, 4, 5, 6, 7, 8, 9, 10];

/// Integer return register.
pub(crate) const RET_REG: u8 = 3;

/// First allocatable volatile scratch (r12 is reserved; COLOR picks r11 next).
pub(crate) const SCRATCH_REG: u8 = 11;

/// Which optimization mode's codegen to emit. Read from `.ex`'s per-function
/// optimization word (`c2_il::IlBundle::opt_words`), never guessed from argv.
///
/// Inside a **straight-line chain** the two differ in exactly one rule,
/// established over all 108 three- and four-operator integer chains and all 27
/// depth-2 trees: a chain intermediate whose predecessor is already dead goes to
/// a fresh descending register under [`OptMode::Ox`] and to r11 under
/// [`OptMode::O1`]. No different opcode, no different operand order — only a
/// register field.
///
/// **That statement used to be made without the qualifier, and as a general
/// claim it is REFUTED.** `docs/OPT_MODE.md` says the modes *"differ in exactly
/// one rule … never a different opcode, never a different operand order — only a
/// register field"*, and this doc said the same. Once a body has more than one
/// block they differ in **block structure**:
///
/// * W10 measured it on an `else` arm — `void e(int a){ if(a) v0(); else v1();
///   v2(); }` is **52 B with an intra-section `48000008`** at `/O1` and **68 B
///   with no `b` at all** at `/Ox` and `/O2`, the join's `bl` and all four
///   epilogue words appearing twice. It declined the shape, because the
///   duplication has a size threshold that is a c2 cost model.
/// * W11 **implements** the same split for a guarded early return, where there
///   is no threshold to fit: the duplicated block is the epilogue, whose length
///   is a constant of the frame class, and `/Ox` copies it in every measured
///   cell. [`crate::codegen::calls::call_seq_text`] is the one place that reads
///   this enum for anything other than a register field.
///
/// Board row **X-b**. Anyone quoting the register-field rule outside a
/// straight-line chain is quoting a refuted claim.
///
/// `/Ox` and `/O2` share a word *and* emit identical bytes (verified per function
/// across eight fixtures once the tail branch's displacement, which is section
/// layout rather than codegen, is masked). So one variant covers both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptMode {
    /// `/Ox` and `/O2` — optimize, favour speed. Every lowering here was
    /// originally established against this mode.
    Ox,
    /// `/O1` (and `#pragma optimize("s", on)`) — optimize, favour size. What the
    /// dc3 workload compiles with.
    O1,
}

// ---- The per-function selector: ONE dispatch, two emitters -----------------

/// What [`select_function`] produced for one function.
///
/// The variants differ only in what the *caller* still has to do — append a
/// branch, pool a constant, take a different obj shape. Everything that decides
/// **whether a function is in class at all** happens inside
/// [`select_function`], which is the point: before it existed, the packed
/// emitter and the COMDAT emitter each had their own copy of the dispatch, and
/// a diagnostic that wanted to ask "would the port accept this function?" had to
/// grow a third (`docs/GAPS.md` §6, "one fact, one locator").
pub enum Selected {
    /// A complete body. No relocation, no pooled constants.
    Plain(Vec<u8>),
    /// A tail call. The bytes are everything *before* the `b <callee>`; the
    /// caller appends the branch at `text_offset + len` and registers the REL24
    /// there, because the branch encodes its own `.text` offset.
    Tail(Vec<u8>),
    /// A floating-point leaf plus one [`FpConstRef`] per constant reference
    /// site, at offsets relative to the start of this text.
    Float {
        text: Vec<u8>,
        consts: Vec<FpConstRef>,
    },
    /// A framed non-leaf call. It owns its whole obj shape (`.pdata` plus the
    /// compiler label symbols), so the selector hands back only the argument
    /// setup — the bytes between the prologue and the `bl` — and the caller,
    /// which knows the function's `.text` offset, finishes the body. Empty
    /// whenever the call's argument is already the formal in r3, one
    /// `or r3,rN,rN` otherwise.
    Framed { setup: Vec<u8> },
    /// A **Class A many-call body**. Like [`Selected::Framed`] it owns its whole
    /// obj shape and every branch word encodes its own `.text` offset, so the
    /// selector hands back the per-call argument setups and the post-call tail and
    /// the caller — which knows where the function lands — finishes the body
    /// through [`call_seq_text`].
    Seq {
        setups: Vec<Vec<u8>>,
        tail: Vec<u8>,
        /// **Board #275 — the entry-block park.** Empty for every body without
        /// one; when it is not, it carries both the words that go between the
        /// prologue and the first guard AND where each formal has landed, so
        /// the two writers resolve the guards' compare registers out of the
        /// same place the moves came from.
        park: calls::SeqPark,
    },
    /// **W-CFG1 — the two-armed `if`/`else` whose arms are calls.** Like
    /// [`Selected::Framed`] and [`Selected::Seq`] it owns its whole obj shape —
    /// a `.pdata` record, a `$M`/`$M`/`$T` triple and two REL24 sites — and like
    /// them its two `bl` words encode their own `.text` offsets, so the selector
    /// hands back nothing but the shape and the caller, which knows where the
    /// function lands, builds the body through
    /// [`crate::codegen::if_call_join::if_call_join_text`].
    ///
    /// A unit variant rather than one carrying bytes: the body is a pure
    /// function of `f.if_call_join` and `base_off`, and carrying a half-built
    /// text would give the two writers two chances to disagree about the block
    /// plan — the defect [`Selected::Seq`]'s `guard`/`early` resolvers were
    /// centralised to prevent.
    IfCallJoin,
    /// **W-EXTDATA — the sunk-`||`-guard body with a shared error tail.** The
    /// same contract [`Selected::IfCallJoin`] has, one block plan over: a unit
    /// variant, because the body is a pure function of
    /// `f.guard_chain_shared_tail` and `base_off` and its FOUR `bl` words each
    /// encode their own `.text` offset. Built through
    /// [`crate::codegen::guard_chain_shared_tail::guard_chain_shared_tail_text`].
    GuardChainSharedTail,
    /// **W-UNDNAME — the guarded allocation with a shared error store.** The
    /// body is a pure function of `f.alloc_init_or_fail` and `base_off`, and its
    /// ONE `bl` word encodes its own `.text` offset, so — like every framed
    /// whole-body shape here — the bytes are built by
    /// [`crate::codegen::alloc_init_or_fail::alloc_init_or_fail_text`] at the
    /// emission site and this variant carries no payload.
    AllocInitOrFail,
    /// **W-OSFINFO — the range-and-flag guarded table lookup.** The same
    /// contract [`Selected::AllocInitOrFail`] has, one block plan over: a unit
    /// variant, because the body is a pure function of `f.osf_handle_guard` and
    /// `base_off` and its TWO `bl` words each encode their own `.text` offset.
    /// Built through
    /// [`crate::codegen::osf_handle_guard::osf_handle_guard_text`].
    OsfHandleGuard,
    /// **W8 — a two-arm conditional tail call.** The body with a zero word at
    /// each of its two tail branches, which the caller fills for the same reason
    /// [`Selected::Tail`] carries an incomplete text: a `b` to an external
    /// encodes its own `.text` offset. The **conditional** branch is already
    /// finished — its displacement is self-relative and offset-independent, and
    /// it takes no relocation at all (`docs/CFG_SHAPE.md` §3.3, board #191).
    CondPair(CondPairParts),
}

/// **The port's per-function instruction selection**, in one place.
///
/// The dispatch order is load-bearing and is the union of the two orders the
/// packed and COMDAT emitters used to carry separately:
///
/// 1. `framed_call` — its own obj shape;
/// 2. `tail_call` — checked **ahead of** the leaf recognizers, so a tail call
///    can never lose its branch to a leaf pattern that happens to match its
///    argument-setup op stream;
/// 3. `empty_body` — a bare `blr`;
/// 4. the FP leaf — its op vocabulary (`Load`/`Lit`/`FpLit` + `+ - * /`) is
///    disjoint from the indirect-load and address leaves' (`LoadInd`/`AddrOf`),
///    so its position relative to them is free; it keeps the packed emitter's;
/// 5. the indirect-load leaf, then the address leaf — exact two-op streams;
/// 6. the comparison leaf — its own branchless spine;
/// 7. otherwise the ordinary arithmetic selector, which refuses whatever it
///    cannot lower.
///
/// `mode` is the per-function optimization mode read from `.ex`; the caller has
/// already refused a TU that mixes modes or carries one this port was not
/// verified against.
pub fn select_function(func: &IlFunction, mode: OptMode) -> Result<Selected, BackendError> {
    // **Board #844's invariant, asked before the dispatch and not inside it.**
    //
    // The whole defect #844 names is that `ops` and the call fields are
    // *alternatives* this function tries in a fixed order, so a body carrying
    // both is half-emitted — a store run without its `bl`, or a `bl` without its
    // run. Both are complete, plausible, wrong bodies, and board #232 is what
    // that costs: 255 commits live on master while the workload scan read
    // `mismatch 0`.
    //
    // The carrier (`c2_il::CallSeq::store_run`) makes the composition
    // unspellable twice by construction — `shape_to_function` leaves `ops` empty
    // for it — so this is a backstop. It is a REFUSAL rather than a priority
    // rule on purpose: the alternative to refusing is picking a winner between
    // the two fields, and picking a winner is the defect itself.
    super::store_run_call::gate_carrier(func)?;
    if func.framed_call.is_some() {
        // The argument setup, through the same selector the integer tail call
        // uses: `[Load(first formal)]` selects to a bare `blr` (an empty setup,
        // the value is already in r3) and `[Load(other formal)]` to
        // `mr r3,rN ; blr`. Dropping the `blr` leaves exactly the words that go
        // between the prologue and the `bl`.
        let mut setup = select_text(func, mode)?;
        let blr = encode_blr();
        debug_assert!(setup.ends_with(&blr), "select_text always terminates in blr");
        setup.truncate(setup.len() - blr.len());
        return Ok(Selected::Framed { setup });
    }
    if let Some(seq) = &func.call_seq {
        let (setups, tail, park) = call_seq_parts(&func.params, seq, mode)?;
        return Ok(Selected::Seq { setups, tail, park });
    }
    // **W8 — the two-arm conditional tail call.** Asked here, beside the other
    // shapes that own their whole branch layout and ahead of every leaf: its
    // body ends in two `b`s to two externals, so it is a *tail-call* shape in
    // every respect the obj can see, and the leaf recognizers below all
    // pattern-match operand streams this shape does not have. It cannot take a
    // body from any of them — `func.cond_pair` is set by exactly one parser
    // production and by nothing else.
    if let Some(pair) = &func.cond_pair {
        return Ok(Selected::CondPair(cond_pair_parts(func, pair)?));
    }
    if func.tail_call.is_some() {
        // A single-argument **floating-point** tail call: the argument is in the
        // other register file, so its setup is at most one `fmr`/`frsp` rather
        // than an operand stream. Asked before `arg_sources`/`ops`, both of which
        // are empty for this shape and would otherwise select the bare branch —
        // i.e. drop the move. See `fp_tail_call_text`.
        if let Some(fp) = &func.fp_tail {
            return Ok(Selected::Tail(fp_tail_call_text(&func.params, fp)?));
        }
        // A multi-argument **floating-point** tail call: a permutation of the FP
        // argument file, then the branch. Asked beside `fp_tail` and before
        // `arg_sources` for the same reason — the two `*_sources` fields index
        // different register files and are never both set.
        if let Some(sources) = &func.fp_arg_sources {
            return Ok(Selected::Tail(fp_permute_args_text(sources)?));
        }
        // Multi-argument: a register permutation, then the branch.
        if let Some(sources) = &func.arg_sources {
            return Ok(Selected::Tail(permute_args_text(sources)?));
        }
        // A VOID tail call (`void f(){ g(); }`, and the generated empty
        // destructor): no argument to compute, so the setup is empty.
        if func.ops.is_empty() {
            return Ok(Selected::Tail(Vec::new()));
        }
        // An integer tail call: the argument computed into r3. `int_tail_call_text`
        // appends the branch itself, so the setup is its text minus the last word.
        let (mut text, _) = int_tail_call_text(func, 0, mode)?;
        text.truncate(text.len() - 4);
        return Ok(Selected::Tail(text));
    }
    // **The pointer-walk accumulate loop**, asked here — after every shape that
    // owns its own obj layout and before every leaf recognizer.
    //
    // Its position is free rather than load-bearing, and saying which is the
    // point of this comment: `func.ptr_walk_loop` is set by exactly one parser
    // production, no other shape sets it, and this body's operand stream
    // (`ops` is empty, `params` is two formals) matches none of the leaf
    // pattern-matchers below. It sits above them so that a reader meets the one
    // shape with a back edge before the straight-line ones, which is a
    // readability claim and not a correctness one.
    // **W-CFG1 — the `if`/`else`-with-a-join.** Asked here, beside the other
    // whole-body shapes and ahead of every leaf recognizer, for the reason the
    // loops below are: `func.if_call_join` is set by exactly one parser
    // production, `func.ops` is empty for it, and no leaf pattern-matcher below
    // can take its body. Its position relative to the two loops is free — the
    // three fields are mutually exclusive by construction — and it is first
    // because it is the only one of the three that is FRAMED, which is the
    // property the arms above this point share.
    // **W-EXTDATA — the sunk-`||`-guard body.** Same placement argument as
    // `if_call_join` immediately below and the same freedom: the field is set by
    // exactly one parser production, `func.ops` is empty for it, and no leaf
    // pattern-matcher can take its body. It is asked first among the framed
    // whole-body shapes only because it is the newest; nothing depends on that.
    if func.guard_chain_shared_tail.is_some() {
        // The mode gate is asked in the emitter as well as in the parser (board
        // #1638), and calling the emitter here is what makes `function_gate` and
        // both writers ask it in exactly one place.
        guard_chain_shared_tail_text(
            func.guard_chain_shared_tail.as_ref().unwrap(),
            0,
            mode,
        )?;
        return Ok(Selected::GuardChainSharedTail);
    }
    // **W-UNDNAME — the guarded allocation with a shared error store.** Same
    // placement argument as its two neighbours and the same freedom: the field
    // is set by exactly one parser production, `func.ops` is empty for it, and
    // no leaf pattern-matcher can take its body.
    if func.alloc_init_or_fail.is_some() {
        // The mode gate is asked in the emitter as well as in the parser (board
        // #1638), and calling the emitter here is what makes `function_gate` and
        // both writers ask it in exactly one place.
        alloc_init_or_fail_text(func.alloc_init_or_fail.as_ref().unwrap(), 0, mode)?;
        return Ok(Selected::AllocInitOrFail);
    }
    // **W-OSFINFO — the range-and-flag guarded table lookup.** Same placement
    // argument as its three neighbours and the same freedom: the field is set by
    // exactly one parser production, `func.ops` is empty for it, and no leaf
    // pattern-matcher can take its body.
    if func.osf_handle_guard.is_some() {
        // The mode gate is asked in the emitter as well as in the parser (board
        // #1638), and calling the emitter here is what makes `function_gate` and
        // both writers ask it in exactly one place.
        osf_handle_guard_text(func.osf_handle_guard.as_ref().unwrap(), 0, mode)?;
        return Ok(Selected::OsfHandleGuard);
    }
    if func.if_call_join.is_some() {
        // The mode gate lives in the emitter, not here, so that `function_gate`
        // and both writers ask it in exactly one place.
        if_call_join_text(func.if_call_join.as_ref().unwrap(), 0, mode)?;
        return Ok(Selected::IfCallJoin);
    }
    if let Some(l) = &func.ptr_walk_loop {
        return Ok(Selected::Plain(ptr_walk_loop_text(l, mode)?));
    }
    // **W-DATA — the static-array scan loop.** Same placement argument as the
    // two loops around it and the same freedom: `func.static_scan_loop` is set
    // by exactly one parser production, `func.ops` is empty for it, and no leaf
    // pattern-matcher below can take its body.
    //
    // It is the only arm in this function whose obj carries a section the
    // *function* did not produce — a COMDAT `.data` for the object it
    // references. That section is **not** decided here: `Selected` has no
    // variant for it, deliberately, because the section belongs to the obj and
    // not to the instruction selection, and `coff::emit_comdat_obj` reads it off
    // `Function::data_defs`. Board #844's invariant is unaffected — this shape
    // sets one field and no other.
    if func.static_scan_loop.is_some() {
        return Ok(Selected::Plain(crate::codegen::static_scan_loop::static_scan_loop_emit(
            func, mode,
        )?));
    }
    // **The body-parameterized pointer-walk loop.** Same placement argument as
    // the shape above and the same freedom: `func.ptr_walk_chain_loop` is set by
    // exactly one parser production, `func.ops` is empty for it, and no leaf
    // pattern-matcher below can take its body. Unlike every shape before it the
    // text it returns has **no fixed length** — the emitter computes it from the
    // accumulate's operation list — which is why the caller must keep taking the
    // length from the returned bytes and never from a constant.
    if let Some(l) = &func.ptr_walk_chain_loop {
        return Ok(Selected::Plain(ptr_walk_chain_loop_text(l, mode)?));
    }
    // The integer divide/modulo leaf. Ahead of the straight-line chain for the
    // same reason the loop is: it is a whole-body shape, and the chain below
    // refuses its operator outright (`straightline.rs`'s `IlOp::Div` arm), so
    // neither can take a body from the other.
    if let Some(d) = &func.div_mod_leaf {
        return Ok(Selected::Plain(div_mod_leaf_text(d, mode)?));
    }
    if func.empty_body {
        return Ok(Selected::Plain(encode_blr().to_vec()));
    }
    if let Some(double) = func.float_leaf {
        let (text, consts) = float_leaf_text(func, double)?;
        return Ok(Selected::Float { text, consts });
    }
    if let Some(t) = indirect_load_text(func) {
        return Ok(Selected::Plain(t?));
    }
    if let Some(t) = addr_leaf_text(func) {
        return Ok(Selected::Plain(t?));
    }
    if let Some(t) = store_leaf_text(func, mode) {
        return Ok(Selected::Plain(t?));
    }
    if let Some(cso) = &func.cmp_shift_or {
        return Ok(Selected::Plain(cmp_shift_or_text(cso, mode)?));
    }
    if let Some(cmp) = &func.compare {
        return Ok(Selected::Plain(compare_leaf_text(cmp, mode)?));
    }
    Ok(Selected::Plain(select_text(func, mode)?))
}

/// **Diagnostic: would the port accept this one function?** Runs
/// [`select_function`] — the same dispatch the emitters run, not a copy of it —
/// plus the two gates that only `/Gy` raises.
///
/// This exists to size the **census/gate disagreement** (roadmap #44): the IL
/// parser is where acceptance is supposed to live, so that
/// [`c2_il::IlBundle::function_census`] and `PortC2` cannot disagree about what
/// is in class. Where a refusal has leaked into codegen instead, the census
/// over-claims, and a numerator with an unmeasured error term is not a
/// benchmark. `c2rs gap` runs this over every function the census calls in class
/// and reports the disagreement in the same block as the census.
///
/// Diagnostic only — nothing in the emitter consults it.
pub fn function_gate(
    func: &IlFunction,
    mode: OptMode,
    fn_level_linking: bool,
) -> Result<(), BackendError> {
    match select_function(func, mode)? {
        // A framed non-leaf call under `/Gy` used to refuse here, because its
        // `.pdata` was not modeled per COMDAT. It is now (W-UNW-1): each framed
        // function gets its own `.pdata` COMDAT tied to its `.text` by
        // SELECT_ASSOCIATIVE, so this arm is gone. Leaving it would have made
        // the diagnostic report a refusal for every framed function the emitter
        // actually emits — the disagreement counter wrong in the *under*-claiming
        // direction, which no test would have caught.
        Selected::Float { consts, .. } if fn_level_linking && !consts.is_empty() => {
            Err(out_of_class(
                "pooled floating-point constant under function-level linking (/Gy)",
            ))
        }
        _ => Ok(()),
    }
}

/// Map a function's `.ex` **optimization-settings word** to the mode this port
/// emits under, or refuse.
///
/// One locator: [`crate::PortC2::build`] applies it per function, and the
/// census/gate cross-check applies it to the word [`c2_il::FnCensus::opt_word`]
/// read off the same segment. A diagnostic that guessed `/O1` because the
/// workload compiles `/O1` would silently disagree with the emitter about every
/// `#pragma optimize` function in the corpus.
///
/// One bit of the word is NOT a mode: `0x0100` says the function is a
/// constructor or a destructor ([`c2_il::OPT_WORD_SPECIAL_MEMBER`], measured one
/// flag and one function kind at a time). It is masked off before the whole-word
/// compare, so a destructor's word reads as the mode it actually is — otherwise
/// every constructor and destructor in the corpus is a `codegen-gap` however
/// ordinary its body, which is what kept `A::~A() {}` (a bare `blr`, decoded as
/// `EmptyBody`) out of the emitter. Every other bit is still required to match a
/// word this port was verified against.
pub fn opt_mode_of_word(word: Option<u32>) -> Result<OptMode, BackendError> {
    match c2_il::opt_word_mode(word) {
        Some(c2_il::OptWordMode::Ox) => Ok(OptMode::Ox),
        Some(c2_il::OptWordMode::O1) => Ok(OptMode::O1),
        None => Err(out_of_class(&format!(
            // Reported as the RAW word, not the masked one: the census key has to
            // name what is actually in the file.
            "opt-mode {} : only {:08x} (/Ox, /O2) and {:08x} (/O1) are \
             implemented{}. See docs/OPT_MODE.md.",
            match word {
                Some(v) => format!("{v:08x}"),
                None => "unreadable".to_string(),
            },
            c2_il::OPT_WORD_OX,
            c2_il::OPT_WORD_O1,
            match word.map(|v| v & !c2_il::OPT_WORD_SPECIAL_MEMBER) {
                Some(0x0080_0005) => " — that is /Od",
                Some(0x0080_0004) => " — that is #pragma optimize(\"\", off)",
                _ => "",
            },
        ))),
    }
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    /// The `/O1` allocator, against `/Ox`, on the shape that separates them:
    /// a four-leaf chain with no addition, where `/Ox` gives every intermediate
    /// its own descending register and `/O1` reuses r11 because each
    /// intermediate's predecessor is dead.
    ///
    /// Transcribed from captures of `int f(int a,int b,int c,int d){return a*b*c*d;}`
    /// at `/Ox /GS- /c` and `/O1 /GS- /c` (`docs/OPT_MODE.md` §3.1).
    #[test]
    fn o1_reuses_r11_where_ox_descends() {
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
                IlOp::Load(0xE609),
                IlOp::Mul,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x4B, 0x29, 0xD6, // mullw r10,r11,r5   <- descends
                0x7C, 0x6A, 0x31, 0xD6, // mullw r3,r10,r6
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "/Ox takes a fresh descending register for a dead intermediate"
        );
        assert_eq!(
            select_text(&f, OptMode::O1).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x6B, 0x29, 0xD6, // mullw r11,r11,r5   <- reuses r11
                0x7C, 0x6B, 0x31, 0xD6, // mullw r3,r11,r6
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "/O1 reuses r11 once the predecessor is dead"
        );
    }

    /// A chain that *does* contain an addition already collapses to r11 under
    /// `/Ox`, so the two modes agree on it — the guard against "fixing" `/O1` by
    /// changing what `/Ox` emits.
    #[test]
    fn a_chain_with_an_addition_is_mode_independent() {
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Sub,
                IlOp::Load(0xE609),
                IlOp::Sub,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            select_text(&f, OptMode::O1).unwrap()
        );
    }

    /// The `/O1` comparison spines: same opcodes, operand order and immediates as
    /// `/Ox`, only the temporaries reallocated. Both sides transcribed from the
    /// captures in `docs/CODEGEN_W6_O1.md` (`int f(int a){return a < 5;}`).
    #[test]
    fn a_comparison_leaf_reallocates_temps_under_o1() {
        let cmp = c2_il::CompareLeaf {
            param: 0xE309,
            rel: c2_il::Rel::Lt,
            signed: true,
            k: 5,
        };
        // /Ox descends r10, r9, r8, r7 for the four temps after `li r11,k`.
        assert_eq!(
            compare_leaf_text(&cmp, OptMode::Ox).unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x05, // li     r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc  r10,r11,r3   (dead; CA is the point)
                0x7D, 0x69, 0x1A, 0x38, // eqv    r9,r11,r3
                0x55, 0x28, 0x0F, 0xFE, // rlwinm r8,r9,1,31,31
                0x7C, 0xE8, 0x01, 0x94, // addze  r7,r8
                0x54, 0xE3, 0x07, 0xFE, // rlwinm r3,r7,0,31,31
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
        // /O1 keeps the dead `subfc` fresh — r11 is still live for the `eqv` — and
        // collapses every temp from the `eqv` on, since that is r11's last use.
        assert_eq!(
            compare_leaf_text(&cmp, OptMode::O1).unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x05, // li     r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc  r10,r11,r3
                0x7D, 0x6B, 0x1A, 0x38, // eqv    r11,r11,r3
                0x55, 0x6B, 0x0F, 0xFE, // rlwinm r11,r11,1,31,31
                0x7D, 0x6B, 0x01, 0x94, // addze  r11,r11
                0x55, 0x63, 0x07, 0xFE, // rlwinm r3,r11,0,31,31
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    /// The unsigned `==`/`!=` immediate predicate, which is **not** the carry
    /// spines'. `a == 4294967295u` (stored as `k = -1`) must refuse: c2
    /// materializes the constant and subtracts, where the port used to emit
    /// `addi r11,r3,1` and come out 4 bytes short — in both modes. Meanwhile the
    /// *signed* `a == -1` and the unsigned *carry* spine at the same literal are
    /// both fine and must stay accepted.
    #[test]
    fn unsigned_eq_above_simm16_refuses_but_its_neighbours_do_not() {
        let mk = |rel, signed, k| c2_il::CompareLeaf { param: 0xE309, rel, signed, k };
        for mode in [OptMode::Ox, OptMode::O1] {
            assert!(compare_leaf_text(&mk(c2_il::Rel::Eq, false, -1), mode).is_err());
            assert!(compare_leaf_text(&mk(c2_il::Rel::Ne, false, -5), mode).is_err());
            // signed `== -1` is the ordinary difference spine.
            assert!(compare_leaf_text(&mk(c2_il::Rel::Eq, true, -1), mode).is_ok());
            // unsigned `>` rides the literal in the `subfic` immediate.
            assert!(compare_leaf_text(&mk(c2_il::Rel::Gt, false, -5), mode).is_ok());
            // and small unsigned literals still take the difference spine.
            assert!(compare_leaf_text(&mk(c2_il::Rel::Eq, false, 32767), mode).is_ok());
        }
    }

}
