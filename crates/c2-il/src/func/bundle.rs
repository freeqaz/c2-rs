use super::body::{self, parse_segment, BodyShape};
use super::bind::Bindings;
use super::gl::drectve_is_boilerplate;
use super::readers::{find_subslice, memchr_byte};
use super::{CallSeq, FramedCall, IlFunction, IlOp, SeqCall, SeqTail};
use crate::IlBundle;

/// The `.ex` per-function start marker (`4F 1F`). The module stream is a
/// sequence of function bodies, each introduced by this marker; the header /
/// index region before the first one is opaque zero-fill for this class.
pub(crate) const FN_START: [u8; 2] = [0x4F, 0x1F];

/// The `.ex` body marker `4C 4F 11` (`LO`) that opens every function body.
pub(crate) const LO_MARKER: [u8; 3] = [0x4C, 0x4F, 0x11];

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
/// Each segment starts at the `4F 1F` immediately preceding its `LO` (so the
/// formals region stays inside the segment, where [`parse_formals`] looks for
/// it) and runs to the next segment's start. Two bodies sharing one preceding
/// `4F 1F` would collide; the later one then starts at its own `LO`, which
/// simply blocks it at `formals-marker` — an honest miss, never a merge that
/// would silently drop a function from the denominator.
pub(crate) fn split_function_bodies(ex: &[u8]) -> Vec<&[u8]> {
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
    if los.is_empty() {
        return Vec::new();
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
    (0..segs_start.len())
        .map(|k| {
            let start = segs_start[k];
            let end = segs_start.get(k + 1).copied().unwrap_or(ex.len());
            &ex[start..end.max(start)]
        })
        .collect()
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
fn split_functions_at(ex: &[u8]) -> (Vec<usize>, Vec<&[u8]>) {
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
) -> Option<IlFunction> {
    match shape {
            // An indirect-load leaf reaches the ordinary integer selector,
            // which pattern-matches its exact two-op stream; `params` carries
            // a member function's `this` at index 0 so the base register comes
            // out right.
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
            BodyShape::EmptyDtorDelegation { callee_tok, this_tok, adjust, .. } => {
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
            // A multi-argument tail call is still a tail call — same resolved
            // callee, same `b <callee>` — but its argument setup is a register
            // permutation rather than an operand stream, so `ops` stays empty
            // and `arg_sources` carries the mapping.
            BodyShape::MultiArgTailCall { params, arg_sources, callee_tok } => {
                Some(IlFunction {
                    params,
                    tail_call: Some(resolve(callee_tok)?),
                    arg_sources: Some(arg_sources),
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
            // Class A many-calls. Every callee is resolved by token through the
            // `.gl` symbol index, exactly as the tail and framed calls are, and a
            // single unresolvable one refuses the whole function — a relocation
            // against a guessed symbol is a mis-emit, not a gap.
            BodyShape::CallSeq { params, calls, tail, saved } => {
                let mut resolved = Vec::with_capacity(calls.len());
                for c in calls {
                    resolved.push(SeqCall {
                        callee: resolve(c.callee_tok)?,
                        arg_ops: c.arg_ops,
                        arg_sources: c.arg_sources,
                    });
                }
                Some(IlFunction {
                    params,
                    call_seq: Some(CallSeq {
                        calls: resolved,
                        saved,
                        tail: match tail {
                            body::SeqTail::Void => SeqTail::Void,
                            body::SeqTail::CallValue { add_k } => SeqTail::CallValue { add_k },
                            body::SeqTail::Lit(k) => SeqTail::Lit(k),
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
        let bind = Bindings::per_record(gl, self.get("sy"), &segs, &starts)?;
        let names = bind.names();
        let src = bind.src.clone();
        let resolve = |tok: u32| -> Option<String> { bind.resolve(tok) };
        let n_defined = segs.len();

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
            let f = shape_to_function(
                parse_segment(seg, bind.locals(i))?,
                name,
                &src,
                &resolve,
            )?;
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
                if f.label_slots(false)? != 1 {
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
        }
        if bind
            .unclaimed
            .iter()
            .any(|n| !accounted.contains(&n.as_str()))
        {
            return None;
        }
        // A callee that is also DEFINED here is out of class: c2 may inline it,
        // and the port cannot. `int f(int); int use(int a){return f(a);}
        // int f(int a){return a+1;}` gets a `.text` of *two* copies of
        // `addi r3,r3,1 ; blr` and **no relocations** — c2 cloned `f` into `use`
        // rather than branching to it. The port emitted `b ?f` against an
        // undefined external and mismatched at file offset 8.
        //
        // Refused wholesale rather than by callee size, because what makes c2
        // inline (and what it does to the symbol table and `.pdata` when it does)
        // is uncharacterized. Calls to true externals are unaffected — those are
        // the tail calls the class was built on.
        if funcs
            .iter()
            .any(|f| f.callees().any(|c| names.iter().any(|n| n == c)))
        {
            return None;
        }
        Some(funcs)
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

    #[test]
    fn opt_words_is_empty_for_a_module_with_no_segments() {
        // R1: an empty module has no `4F 1F` at all, and its obj is
        // mode-independent — which is why `PortC2` checks the words *after* the
        // empty-module case.
        assert_eq!(ex_bundle(vec![0u8; 64]).opt_words(), Some(Vec::new()));
    }
}
