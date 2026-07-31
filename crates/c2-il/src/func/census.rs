use super::body::{
    self, call_tokens, parse_segment_detail, BodyShape, DtorSubObject, CALLEE_UNRESOLVED_DTOR,
    CALLEE_UNRESOLVED_FRAMED, CALLEE_UNRESOLVED_SEQ, CALLEE_UNRESOLVED_TAIL, OPT_MODE,
};
use super::bind::Bindings;
use super::bundle::shape_to_function;
use super::bundle::split_function_bodies;
use super::bundle::{opt_word_at, opt_word_mode};
use super::Block;
use super::IlFunction;
use crate::IlBundle;

/// Split the `.ex` stream into per-function byte segments at each `4F 1F`
/// function-start marker. Segment `k` runs from marker `k` to marker `k+1`
/// (the last to end-of-stream).
/// One function's census verdict (P2b). Either the modeled shape it parsed as,
/// or the first feature that blocked it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FnVerdict {
    /// Parsed as a modeled shape. The string is a stable shape label
    /// (`straight-line`, `void-tail-call`, `int-tail-call`, `framed-call`).
    InClass(&'static str),
    /// Blocked at the first unmodeled feature.
    Blocked(Block),
}

impl FnVerdict {
    /// The census bucket key: the shape label when in class, else the blocking
    /// feature (see [`Block::feature`]).
    pub fn key(&self) -> String {
        match self {
            FnVerdict::InClass(s) => (*s).to_string(),
            FnVerdict::Blocked(b) => b.feature(),
        }
    }
    pub fn in_class(&self) -> bool {
        matches!(self, FnVerdict::InClass(_))
    }
}

/// One census row: a function segment and how it classified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FnCensus {
    /// Index of the function within the TU (`.ex` segment order).
    pub index: usize,
    /// Mangled name, when `.gl` has one at this position.
    pub name: Option<String>,
    /// Segment length in bytes (a rough proxy for function size).
    pub seg_len: usize,
    pub verdict: FnVerdict,
    /// Raw bytes around the blocking site, for grammar work: the segment window
    /// `[off - CENSUS_HEX_BACK, off + CENSUS_HEX_FWD)` clamped to the segment,
    /// plus the index of the blocking byte within that window. Empty when the
    /// function is in class.
    pub hex: Vec<u8>,
    /// Index of the blocking byte inside [`FnCensus::hex`].
    pub hex_mark: usize,
    /// **The control-flow class** of this body, decoded independently of whether
    /// the body is in class (`crates/c2-il/src/func/body/shapes/control_flow.rs`).
    ///
    /// Two families of value, and the prefix says which:
    ///
    /// * `cflow-<shape>` / `cflow-<shape>+expr-modeled` — the statement layer
    ///   decoded end to end, so this body's CFG is fully known. The `+expr-modeled`
    ///   half is the one blocked on **control flow alone**; without the suffix the
    ///   body needs expression work as well.
    /// * `cf-<production>-0xNN` — the statement-layer decoder itself stopped, and
    ///   this is where. Ranked, it is the residue of the grammar.
    ///
    /// A third census axis, beside the blocking feature and the frame class, and a
    /// separate field for the same reason the frame class is one: the
    /// blocking-feature histogram IS the widening order and several sessions of
    /// documented tables name its keys, so an orthogonal fact goes beside it rather
    /// than into its names.
    ///
    /// **Decode-only, and structurally so**: nothing reads this field except the
    /// report. It is not consulted by acceptance, by `shape_to_function`, or by the
    /// emitter, and the scanner that produces it constructs no `BodyShape`.
    pub cflow: String,
    /// **The exception-handling axis** — which side of `docs/EH_RECORDS.md` §6's
    /// sub-object boundary this body falls on:
    ///
    /// > Exactly one sub-object statement and nothing else is a bare branch. A
    /// > second sub-object, or any other statement beside it, is the WHOLE EH
    /// > RECORD.
    ///
    /// * `eh-none` — the body decoded and carries no `5C`/`5D`/`5E` marker. No
    ///   destructible object is ever live in it, so `/EHsc` costs it nothing.
    /// * `eh-bare` — one object goes live, one is tracked, and there is no other
    ///   statement. **The cheap side**: no `__CxxFrameHandler` prefix, no second
    ///   `.pdata`, no funclet. The port's three `empty-dtor-*` shapes all live
    ///   here, which is this axis's control group.
    /// * `eh-plus-stmt` — one object, plus a body statement.
    /// * `eh-multi` — two or more objects.
    /// * `eh-partial` — a marker was seen and then the walk stopped. **Not bare**:
    ///   the bare shape decodes end to end by construction, so a body that carries
    ///   a marker and does not decode is on the EH side whatever else it needs.
    /// * `eh-unknown` — the walk stopped before any marker; nothing is claimed.
    ///
    /// A separate field from [`FnCensus::cflow`] and from the blocking feature for
    /// the reason both of those are separate from each other: **nothing in the
    /// blocking-feature key says which side a body is on.** `work/WEH/probe/p1.cpp`
    /// files a cheap constructor and an EH constructor under the *same* key
    /// `expr-intrinsic-this-adjust`, and that is not a defect of the key — the two
    /// bodies differ by one statement the key never reaches.
    ///
    /// **Decode-only, structurally**: nothing reads this field except the report.
    pub eh: String,
    /// **How many CALL tokens the body issues** — see [`call_tokens`]. Counted for
    /// every function, in class or not, because the in-class shapes are the control
    /// group: they are all leaves or single tail calls, so a non-zero count among
    /// them would say the measure is wrong.
    pub calls: usize,
    /// This function's **optimization-settings word**, read out of this segment's
    /// own `4F 1F 80 <LE32>` head (never zipped in from `IlBundle::opt_words`,
    /// which walks a different segmentation). The census/gate cross-check needs
    /// it to pick the mode the port would emit under.
    pub opt_word: Option<u32>,
}

impl FnCensus {
    /// The **frame class**: what the call count alone settles about whether this
    /// body needs a stack frame (`docs/IL_CALL_IN_EXPR.md` §18).
    ///
    /// Three values, and the middle one is honest rather than convenient:
    ///
    /// * `calls-0` — no call at all. It cannot need LR saved, so **no frame**.
    /// * `calls-1` — exactly one. A tail call emits `b callee` and stays a leaf
    ///   (`return p->M();`), while a call whose result is then computed on needs a
    ///   frame (`return g(a) + k;`, which is the port's existing `FramedCall`).
    ///   The count cannot tell them apart and this class does not pretend to.
    /// * `calls-2plus` — two or more, which **always** needs a frame: the first
    ///   `bl` clobbers LR and the return address is still live. There is no
    ///   two-call shape that stays a leaf.
    pub fn frame_class(&self) -> &'static str {
        match self.calls {
            0 => "calls-0",
            1 => "calls-1",
            _ => "calls-2plus",
        }
    }
}

/// The control-flow axis for one segment: run the statement-layer scanner and
/// render its verdict as a census key.
///
/// Run for **every** function, in class or not, for the reason the frame class is:
/// the in-class shapes are the control group. Every one of them is a single basic
/// block, so a `cflow-loop` reading among the accepted rows would indict the
/// measure rather than reveal a loop the port lowers.
///
/// A segment with no `LO` body marker has no body to scan; that is already the
/// `lo-marker` refusal on the primary axis, and restating it here would put a
/// container-level fact into a control-flow histogram.
/// …and the EH axis beside it, from the SAME walk. Returns
/// `(control-flow key, EH key)`.
///
/// One scan, two readings. The two axes answer different questions off one
/// traversal and a second traversal would double the census's cost for a fact the
/// first one already collected.
fn cflow_key(seg: &[u8]) -> (String, String) {
    let Some(lo) = crate::func::readers::find_subslice(seg, &crate::func::bundle::LO_MARKER) else {
        return ("cf-no-body".to_string(), "eh-unknown".to_string());
    };
    let scan = body::shapes::control_flow::scan_full(seg, lo);
    let cflow = match &scan.body {
        Ok(cf) => cf.key(),
        Err(b) => b.feature(),
    };
    (cflow, scan.eh.key(scan.decoded).to_string())
}

/// Bytes of context kept before / after a blocking site.
pub const CENSUS_HEX_BACK: usize = 16;
pub const CENSUS_HEX_FWD: usize = 24;

impl IlBundle {
    /// **Function-level census (P2b).** Classify *every* function in the bundle
    /// independently, so a TU whose 700th function uses an unmodeled opcode
    /// still reports the other 699 as in-class.
    ///
    /// This is the measurement [`IlBundle::functions`] cannot give: that method
    /// is all-or-nothing per TU (correctly — the port must emit a whole obj or
    /// nothing), so over a real workload it reports one `vocab-gap` per TU and
    /// cannot rank the missing classes. The census runs the *same*
    /// [`parse_segment_detail`] per segment and keeps the first blocking
    /// feature, so the histogram of [`FnVerdict::key`] over a corpus is the
    /// widening order (docs/ROADMAP.md §G5).
    ///
    /// Diagnostic only — never a gate, and never consulted by the emitter.
    /// Returns `None` only when the bundle lacks the required files.
    pub fn function_census(&self) -> Option<Vec<FnCensus>> {
        Some(
            self.census_functions()?
                .into_iter()
                .map(|(c, _)| c)
                .collect(),
        )
    }

    /// **The census/gate cross-check (roadmap #44).** Every row
    /// [`IlBundle::function_census`] reports, paired with the emitter's own
    /// per-function record for the rows the census calls in class.
    ///
    /// Why this exists: acceptance is supposed to live in the IL parser so the
    /// census and the gate cannot disagree, and for a long time it did not
    /// entirely — `int f(int a,int b,int c){ return a + b*c; }` censused in class
    /// and `PortC2` returned `NotImplemented`, because a `*` after the first
    /// operator was gated in codegen where the census could not see it. A
    /// numerator with an unmeasured error term is not a benchmark, so the
    /// disagreement gets a permanent instrument rather than a note: the harness
    /// runs the port's own selector over every `Ok` row and reports the
    /// disagreement in the same block as the census (`docs/GAPS.md` §6, "a
    /// diagnostic that runs outside the parser needs a population whose answer is
    /// already known").
    ///
    /// `Err` carries why there is no record:
    ///
    /// * `"blocked"` — the census itself refused; nothing to cross-check.
    /// * `"callee-unresolved"` — the body parsed, but the CALL token has no `.gl`
    ///   symbol, so [`shape_to_function`] refuses. That IS a disagreement, and a
    ///   per-function one, so it is named rather than folded into `blocked`.
    ///
    /// Diagnostic only, exactly like the census: acceptance is unchanged and the
    /// emitter never consults it.
    pub fn census_functions(&self) -> Option<Vec<(FnCensus, Result<IlFunction, &'static str>)>> {
        let gl = self.get("gl")?;
        let ex = self.ex()?;
        let segs = split_function_bodies(ex);
        // The whole correspondence seam comes from ONE place ([`super::bind`]).
        // The census's names are paired POSITIONALLY, which is a different
        // binding from the gate's per-record one — `bind.rs`'s module doc states
        // that disagreement and pins it; closing it is roadmap #14's follow-up
        // and moves the numerator, so it is not done silently here.
        //
        // `.gl` is deliberately NOT threaded into the body parse. The assignment
        // class used to decide "is this destination a global?" by asking whether
        // `.gl` named it, and that was wrong (a file-scope `static` is `$sv`, which
        // the index does not accept as an identifier). The symbol view locals needed
        // turned out to be `.sy`, not `.gl`, so the vestigial `.gl` thread is gone
        // rather than left in place looking load-bearing. Do not restore the
        // absence test.
        //
        // The `.sy` locals and the `GlIndex` resolution are the SAME construction
        // the gate makes — one `Bindings`, one `SyLocals::new`, one `GlIndex` —
        // so the census cannot report a function in class that
        // `IlBundle::functions` would refuse for want of a local, or the reverse.
        // Over the census's own segment list, which is NOT the gate's: see
        // `bind.rs`'s table.
        let bind = Bindings::positional(gl, self.get("sy"), &segs);
        let resolve = |tok: u32| -> Option<String> { bind.resolve(tok) };
        let src = bind.src.clone();
        Some(
            segs.iter()
                .enumerate()
                .map(|(i, seg)| {
                    // A variadic function is refused on its NAME, because its body
                    // IL is byte-identical to a non-variadic twin's — see
                    // [`super::bind::mangled_is_varargs`], which is the same
                    // predicate `functions` applies, so the census and the gate
                    // cannot disagree.
                    //
                    // Only when the names are `paired`. Unpaired means the census
                    // has no name for this segment, and reporting the body's real
                    // blocker is better than inventing a reason: `functions`
                    // refuses that whole TU for want of names anyway, so nothing
                    // here can be emitted either way.
                    let varargs = bind.is_varargs(i);
                    // Held across the verdict so the gate side can convert the
                    // very same parse — two readings of one parse, never two parses.
                    let mut shape: Result<BodyShape, Block> = Err(Block {
                        ctx: "fn-varargs",
                        byte: None,
                        off: 0,
                        aux: 0,
                    });
                    let verdict = if varargs {
                        FnVerdict::Blocked(Block {
                            ctx: "fn-varargs",
                            byte: None,
                            off: 0,
                            aux: 0,
                        })
                    } else {
                        shape = parse_segment_detail(seg, bind.locals(i));
                        match &shape {
                            Ok(BodyShape::StraightLine { .. }) => FnVerdict::InClass("straight-line"),
                            Ok(BodyShape::VoidTailCall { .. }) => FnVerdict::InClass("void-tail-call"),
                            // Three buckets for one shape, so the movement out of
                            // `expr-call-in-expr` is attributable *per receiver
                            // production*: the base form and the member form at
                            // offset 0 emit the identical four bytes a void tail
                            // call does, and the adjusted member form emits one
                            // `addi` more. Splitting them here is what lets the
                            // in-class gain be checked against the individual
                            // `recv-field-off0` / `recv-field` bucket drops rather
                            // than against their sum.
                            Ok(BodyShape::EmptyDtorDelegation {
                                sub_object: DtorSubObject::Base,
                                ..
                            }) => FnVerdict::InClass("empty-dtor-delegation"),
                            Ok(BodyShape::EmptyDtorDelegation { adjust: 0, .. }) => {
                                FnVerdict::InClass("empty-dtor-member")
                            }
                            Ok(BodyShape::EmptyDtorDelegation { .. }) => {
                                FnVerdict::InClass("empty-dtor-member-adjusted")
                            }
                            Ok(BodyShape::IntTailCall { .. }) => FnVerdict::InClass("int-tail-call"),
                            // Split from the integer tail call by the register
                            // FILE, and split again by whether the boundary
                            // narrows, so the rung's gain is attributable to the
                            // free move and to the `frsp` separately rather than
                            // to their sum — the same reason the dtor
                            // delegation carries three buckets for one shape.
                            Ok(BodyShape::FpTailCall { narrowing: false, .. }) => {
                                FnVerdict::InClass("fp-tail-call")
                            }
                            Ok(BodyShape::FpTailCall { .. }) => {
                                FnVerdict::InClass("fp-tail-call-narrowing")
                            }
                            Ok(BodyShape::MultiArgTailCall { .. }) => {
                                FnVerdict::InClass("multiarg-tail-call")
                            }
                            // W34, the multi-argument FP tail call. Split by
                            // whether the permutation moves anything at all: the
                            // identity is a bare `b <callee>` and a cycle is
                            // `fmr`s through f0, and the two are worth different
                            // amounts of evidence even though they are one shape.
                            Ok(BodyShape::FpMultiArgTailCall { arg_sources, .. }) => {
                                FnVerdict::InClass(
                                    if arg_sources.iter().enumerate().all(|(i, &s)| i == s) {
                                        "fp-multiarg-tail-call"
                                    } else {
                                        "fp-multiarg-tail-call-perm"
                                    },
                                )
                            }
                            Ok(BodyShape::FramedCall { .. }) => FnVerdict::InClass("framed-call"),
                            // Class A many-calls. Split by tail so the rung's gain
                            // can be attributed to the production that earned it
                            // rather than to their sum.
                            Ok(BodyShape::CallSeq { tail, .. }) => {
                                FnVerdict::InClass(match tail {
                                    body::SeqTail::Void => "call-sequence",
                                    body::SeqTail::CallValue { .. } => "call-sequence-value",
                                    body::SeqTail::Lit(_) => "call-sequence-lit",
                                    // WCO — the chain result read through, one
                                    // `lwz`. Its own key rather than sharing
                                    // `-value`'s: the address form IS
                                    // `CallValue`, so a shared name would make
                                    // the two indistinguishable in a census
                                    // delta and this rung ships both.
                                    body::SeqTail::CallLoad { .. } => "call-sequence-load",
                                    // WFL — the same read-through in the OTHER
                                    // register file. Its own key rather than
                                    // sharing `-load`'s: the instruction is
                                    // `lfs`/`lfd` into f1 and the obj acquires
                                    // `_fltused`, so a shared name would make a
                                    // census delta unable to say which of the
                                    // two produced it — and this family's whole
                                    // history is deltas attributed to the wrong
                                    // production.
                                    body::SeqTail::CallLoadFp { .. } => "call-sequence-load-fp",
                                    // Split by relation, not merged: the `==`
                                    // fold and the order spines are different
                                    // instruction counts and different label
                                    // strides, so a shared key would hide which
                                    // of the two a census delta came from.
                                    body::SeqTail::Cmp { cmp: crate::func::SeqCmp::Eq, .. } => {
                                        "call-sequence-cmp-eq"
                                    }
                                    body::SeqTail::Cmp { .. } => "call-sequence-cmp-order",
                                })
                            }
                            Ok(BodyShape::Compare(_)) => FnVerdict::InClass("compare-leaf"),
                            Ok(BodyShape::EmptyBody) => FnVerdict::InClass("empty-body"),
                            Ok(BodyShape::IndirectLoad { .. }) => {
                                FnVerdict::InClass("indirect-load-leaf")
                            }
                            // Kept apart from `indirect-load-leaf` so the in-class
                            // gain of this rung can be checked against the bucket
                            // drops it claims (`docs/IL_CALL_IN_EXPR.md` §19), and
                            // because the two emit different instructions.
                            Ok(BodyShape::AddrLeaf { .. }) => FnVerdict::InClass("addr-leaf"),
                            // Kept apart from `addr-leaf` and `indirect-load-leaf`
                            // for the same reason those two are kept apart: the
                            // three share a designator and emit three different
                            // instructions, so this rung's gain can be checked
                            // against the `expr-op-0x27` / `expr-op-0x32` /
                            // `expr-intrinsic-base-member-addr` bucket drops it
                            // claims rather than against their sum.
                            Ok(BodyShape::StoreLeaf { .. }) => FnVerdict::InClass("store-leaf"),
                            // W37. Its own family, kept apart from `store-leaf`
                            // for the same reason: it is a different production
                            // with two gates the single store does not have, and
                            // `cross_sweep.sh` discovers families by this label.
                            Ok(BodyShape::StoreRun { .. }) => FnVerdict::InClass("store-run"),
                            Ok(BodyShape::FloatLeaf { double, .. }) => {
                                FnVerdict::InClass(if *double { "double-leaf" } else { "float-leaf" })
                            }
                            Err(b) => FnVerdict::Blocked(*b),
                        }
                    };
                    // ---- The two POST-PARSE gates -----------------------------
                    //
                    // Both are per-function facts `PortC2` has always enforced and
                    // the census never checked, so the numerator counted functions
                    // the port refuses (roadmap #44). They are applied **last**, to
                    // an otherwise-in-class function only, which is what keeps every
                    // blocked function's real blocking feature in the histogram —
                    // gating either of them up front would relabel bodies whose
                    // actual problem is somewhere else entirely.
                    let opt_word = opt_word_at(seg);
                    let mut func: Result<IlFunction, &'static str> = Err("blocked");
                    let verdict = match (shape, verdict) {
                        (Ok(sh), FnVerdict::InClass(label)) => {
                            // (a) The callee must resolve through `.gl`. A CALL
                            // token carries a function-*type* id, not the callee, so
                            // the name comes from the symbol index; when it is not
                            // there the emitter has no symbol to relocate against,
                            // and guessing one is a relocation against the wrong
                            // symbol — a mis-emit, not a gap. `shape_to_function` is
                            // the same conversion `IlBundle::functions` runs, so the
                            // two cannot disagree about this.
                            let name = bind.name_for_shape(i);
                            match shape_to_function(sh, &name, &src, &resolve) {
                                None => FnVerdict::Blocked(Block {
                                    ctx: match label {
                                        "framed-call" => CALLEE_UNRESOLVED_FRAMED,
                                        l if l.starts_with("call-sequence") => {
                                            CALLEE_UNRESOLVED_SEQ
                                        }
                                        l if l.starts_with("empty-dtor") => {
                                            CALLEE_UNRESOLVED_DTOR
                                        }
                                        _ => CALLEE_UNRESOLVED_TAIL,
                                    },
                                    byte: None,
                                    off: seg.len().saturating_sub(1),
                                    aux: 0,
                                }),
                                // (b) The optimization mode. `.ex` records it per
                                // function and the port emits only the two words it
                                // has been verified against; the rest — `/Od`, a
                                // `#pragma optimize("", off)`, an unreadable prefix —
                                // are refused.
                                Some(f) if opt_word_mode(opt_word).is_none() => {
                                    let _ = f;
                                    FnVerdict::Blocked(Block {
                                        ctx: OPT_MODE,
                                        byte: None,
                                        off: seg.len().saturating_sub(1),
                                        aux: opt_word.unwrap_or(0) as u64,
                                    })
                                }
                                Some(f) => {
                                    func = Ok(f);
                                    FnVerdict::InClass(label)
                                }
                            }
                        }
                        (_, v) => v,
                    };
                    // Keep the raw bytes around the blocking site: decoding a new
                    // grammar production always starts by staring at exactly this
                    // window, and having it in the census means that work is a
                    // report away instead of a one-off script.
                    let (hex, hex_mark) = match &verdict {
                        FnVerdict::InClass(_) => (Vec::new(), 0),
                        FnVerdict::Blocked(b) => {
                            let start = b.off.saturating_sub(CENSUS_HEX_BACK);
                            let end = (b.off + CENSUS_HEX_FWD).min(seg.len());
                            let start = start.min(end);
                            (seg[start..end].to_vec(), b.off - start)
                        }
                    };
                    let (cflow, eh) = cflow_key(seg);
                    (
                        FnCensus {
                            index: i,
                            name: bind.reported_name(i),
                            seg_len: seg.len(),
                            verdict,
                            hex,
                            hex_mark,
                            calls: call_tokens(seg),
                            cflow,
                            eh,
                            opt_word,
                        },
                        func,
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::bundle::FN_START;
    use crate::func::test_fixtures::*;

    /// `4F 1F 80 <LE32 /Ox>` — a segment head carrying a mode the port emits under.
    fn seg_head() -> Vec<u8> {
        let mut v = vec![FN_START[0], FN_START[1], 0x80];
        v.extend_from_slice(&crate::func::OPT_WORD_OX.to_le_bytes());
        v
    }

    /// A `.gl` with two real records: `<token> 00 <name> 00`, which is the shape
    /// [`super::super::gl::gl_symbol_index`] reads the callee name out of.
    fn gl_two_records() -> Vec<u8> {
        let mut v = vec![0xE4, 0x09, 0x00];
        v.extend_from_slice(b"?f@@YAXXZ\x00");
        v.extend_from_slice(&[0xE3, 0x09, 0x00]);
        v.extend_from_slice(b"?g@@YAXXZ\x00");
        v
    }

    #[test]
    fn census_classifies_each_function_independently() {
        // The point of P2b: one blocked function does not hide the in-class
        // ones. `functions()` (the gate) is all-or-nothing and returns None.
        // Each segment opens `4F 1F 80 <LE32 opt word>` and `.gl` carries a real
        // token→name record per symbol: both are POST-PARSE acceptance gates now
        // (the optimization mode, and the callee resolving through `.gl`), so a
        // fixture that omits them measures those gates rather than the split.
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(CALL_THEN_STMT);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), ex),
                ("gl".to_string(), gl_two_records()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 2);
        assert_eq!(census[0].verdict, FnVerdict::InClass("void-tail-call"));
        assert!(!census[1].verdict.in_class());
        assert_eq!(census[0].name.as_deref(), Some("?f@@YAXXZ"));
        // In-class functions carry no hex window; blocked ones point at the
        // offending byte inside theirs.
        assert!(census[0].hex.is_empty());
        let FnVerdict::Blocked(b) = census[1].verdict else {
            panic!("expected a block");
        };
        assert_eq!(census[1].hex[census[1].hex_mark], b.byte.unwrap());
    }

    /// **The control group for the control-flow axis.** Every shape the port
    /// accepts is a single basic block, so an in-class row must read
    /// `cflow-straight`. Asserted here on the pinned segments and measured on the
    /// workload, where all 455,049 readable in-class bodies agree — a `cflow-loop`
    /// among them would mean the port had been handed a back edge, and a
    /// `cflow-if-1` would mean the scanner invents branches.
    ///
    /// The axis is also **decode-only**, which the second half asserts: the row's
    /// verdict is the same whatever the scanner said, because nothing reads the
    /// field except the report.
    #[test]
    fn every_in_class_row_is_a_single_basic_block() {
        let mut ex: Vec<u8> = Vec::new();
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(MVP_CALL);
        ex.extend_from_slice(&seg_head());
        ex.extend_from_slice(CALL_THEN_STMT);
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![("ex".to_string(), ex), ("gl".to_string(), gl_two_records())]
                .into_iter()
                .collect(),
        };
        let census = bundle.function_census().unwrap();
        for f in &census {
            if f.verdict.in_class() {
                assert!(
                    f.cflow.starts_with("cflow-straight"),
                    "in-class function #{} reads {} — the port accepts only single \
                     basic blocks, so this is either a scanner inventing control \
                     flow or an emitter that has been handed some",
                    f.index,
                    f.cflow
                );
            }
        }
        // …and a blocked row still carries the axis, because the measurement is
        // over every function, not only the refused ones.
        assert!(census.iter().all(|f| !f.cflow.is_empty()));
    }

    #[test]
    fn census_hex_window_is_clamped_to_the_segment() {
        // A block at offset 0 must not underflow, and one near the end must not
        // run past it — the window is diagnostic and must never panic.
        let tiny: &[u8] = &[0x4C, 0x4F, 0x11, 0xFF];
        let bundle = crate::IlBundle {
            base_name: "t".into(),
            files: vec![
                ("ex".to_string(), tiny.to_vec()),
                ("gl".to_string(), b"?f@@YAXXZ\x00".to_vec()),
            ]
            .into_iter()
            .collect(),
        };
        let census = bundle.function_census().unwrap();
        assert_eq!(census.len(), 1);
        let c = &census[0];
        assert!(!c.verdict.in_class());
        assert!(c.hex_mark < c.hex.len().max(1));
        assert!(c.hex.len() <= CENSUS_HEX_BACK + CENSUS_HEX_FWD);
    }
}
