//! The scan proper: [`scan_one`] over one TU, and [`gap_scan`]'s worker pool
//! over the source list. Split out of `gap.rs` unchanged; see [`super`] for the
//! module docs.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use c2_core::{Backend, BackendError, PortC2};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

use crate::capture_cache::{capture_via, CaptureCache};
use crate::provenance::Provenance;

use super::classify::{
    cflow_needs_block_ir, cflow_series_bucket, clip, dtor_callee_class, gate_key,
    is_compiler_generated, mangling_class,
    normalize_cl_error,
};
use super::render::print_factorization;
use super::witness::{row_dump, wall_dump, witness_path, write_witness};
use super::{
    GapConfig, GapReport, TuClass, TuResult, WitnessRow, PORT_WRITER_SECTIONS,
    WHOLE_TU_RECOGNIZERS,
};

/// Scan one TU. `work` must be a private (per-TU) directory.
fn scan_one(
    tc: &Toolchain,
    cfg: &GapConfig,
    cache: Option<&CaptureCache>,
    src: &str,
    work: &Path,
    do_replay: bool,
) -> TuResult {
    let mut res = TuResult {
        src: src.to_string(),
        class: TuClass::CaptureFail,
        reason: String::new(),
        detail: String::new(),
        ex_len: 0,
        fn_names: 0,
        replay_ok: None,
        fn_total: 0,
        fn_in_class: 0,
        fn_blockers: BTreeMap::new(),
        fn_frames: BTreeMap::new(),
        fn_cflow: BTreeMap::new(),
        fn_cflow_off: BTreeMap::new(),
        fn_cfg_admit: BTreeMap::new(),
        fn_eh: BTreeMap::new(),
        fn_dispatch: BTreeMap::new(),
        fn_complete: BTreeMap::new(),
        fn_prod: BTreeMap::new(),
        fn_gate_refusals: BTreeMap::new(),
        bind_checks: BTreeMap::new(),
        gate_cause: None,
        gate_causes: Vec::new(),
        gl_body_starts: None,
        selective_bind: None,
        emit: BTreeMap::new(),
        emit_blockers: BTreeMap::new(),
        emit_witness: Vec::new(),
        fndiff: Vec::new(),
        plan: Default::default(),
    };

    // 1. Capture: real flags, real cwd, strace keeps bundle + obj. Served from
    //    the content-addressed cache when one is configured — the cache dir IS
    //    the capture dir, so the `-Fo` path c2 bakes into the obj is a function
    //    of the key and a hit is byte-identical to the capture that filled it
    //    (`crate::capture_cache`).
    //    The cache-or-not decision itself is `capture_cache::capture_via` — one
    //    implementation, shared with the fixture gate in `lib.rs`. The outcome
    //    word is dropped HERE, visibly, because the scan reports the cache in
    //    aggregate from `CacheStats` and deliberately keeps it out of the
    //    per-TU JSONL row (see `CacheOutcome`'s own doc).
    let capture_result =
        capture_via(cache, tc, src, &cfg.flags, cfg.cwd.as_deref(), work).0;
    let captured =
        match capture_result {
            Ok(c) => c,
            Err(e) => {
                let (key, detail) = normalize_cl_error(&e.to_string());
                res.reason = key;
                res.detail = detail;
                return res;
            }
        };
    // The obj's shape depends on argv the IL bundle does not record: /Gy (implied
    // by /O1 and /O2) puts each function in its own COMDAT .text. Two of the
    // port's per-function refusals are /Gy-only, so the cross-check below needs
    // the same flag the emitter gets.
    let gy = PortC2::flags_imply_function_level_linking(&cfg.flags);
    res.ex_len = captured.bundle.ex().map(|b| b.len()).unwrap_or(0);

    // 1a''. **IR0 / K1 — THE LOSSLESS CONTAINER CODEC, RUN ON THE WORKLOAD FOR
    //       THE FIRST TIME** (lane `ir0`, `docs/rungs/2026-08-20-ir0.md`).
    //
    //       `IlModel::parse` is a **total, fail-closed** container codec: it
    //       frames every byte of `.ex` and `.gl` into typed spans with
    //       unrecognized runs coalesced into `Span::Opaque`, re-encodes the
    //       result, and returns `CodecError::CannotRoundTrip` rather than a
    //       model it cannot serialize back byte-for-byte
    //       (`crates/c2-il/src/codec.rs`). That is exactly the invariant the
    //       architecture proposal's IR0 asks for — **and until this line it had
    //       only ever been run on the 386 fixtures**
    //       (`crates/c2-harness/tests/il_roundtrip.rs`) and on the generated
    //       corpus. Never on dc3.
    //
    //       The distinction that makes this worth a key rather than a test:
    //       `parse_ex`/`parse_gl` are total **by construction**, so a failure
    //       cannot come from an unrecognized construct — it can only come from
    //       an `ExToken::encode_into` disagreeing with the bytes it consumed,
    //       i.e. a **decoding bug**, on a stream shape the fixture spread has
    //       no instance of. So `ir0-roundtrip-broken` is not a coverage number;
    //       it is a defect count with a known answer of 0.
    //
    //       Both halves are emitted, including the zero, because a residue key
    //       that stops occurring must read `0` rather than vanish
    //       (`docs/STATUS.md` trap 5). `-tus` is their denominator, published
    //       beside them for the reason `emit_set_violations_gate` publishes
    //       its population: a green control whose denominator is unstated is
    //       indistinguishable from a control that checked nothing.
    {
        *res.emit.entry("ir0-roundtrip-tus".into()).or_insert(0) += 1;
        match c2_il::IlModel::parse(&captured.bundle) {
            Ok(model) => {
                // `parse` already verified the re-encode file by file and fails
                // closed; re-assert it here over the WHOLE bundle so the key is
                // a second, differently-built derivation of the same claim
                // (#3288) rather than a restatement of `parse`'s Ok.
                if model.encode().files == captured.bundle.files {
                    *res.emit.entry("ir0-roundtrip-ok".into()).or_insert(0) += 1;
                } else {
                    *res.emit.entry("ir0-roundtrip-broken".into()).or_insert(0) += 1;
                    *res.emit
                        .entry("ir0-roundtrip-broken-reencode".into())
                        .or_insert(0) += 1;
                }
            }
            Err(e) => {
                *res.emit.entry("ir0-roundtrip-broken".into()).or_insert(0) += 1;
                // The suffix that refused, so the residue is actionable rather
                // than a bare count. `CodecError` has one variant today.
                let c2_il::CodecError::CannotRoundTrip { suffix, .. } = &e;
                *res.emit
                    .entry(format!("ir0-roundtrip-broken-suffix|{suffix}"))
                    .or_insert(0) += 1;
            }
        }
    }
    res.fn_names = captured
        .bundle
        .get("gl")
        .map(|gl| c2_il::mangled_names(gl).len())
        .unwrap_or(0);
    // **W-PHASE7B — read for EVERY class, not only the refused ones.** The
    // matching TUs are the control: if a TU the port already emits byte-exact
    // could read `present < total`, the field would be measuring the reader and
    // not the input, which is the one thing it claims not to do.
    // **THE OBJECT PLAN** (lane `w-objplan`) — graded for EVERY TU, before the
    // class is decided and independently of it.
    //
    // Two independent producers, and the independence is the design: `observe`
    // reads the **reference** obj (ground truth, available on every TU that
    // captured) and `c2_core::plan::predict` computes the port's plan **from the
    // IL bundle without emitting**. A grade taken over `observe(port_obj)`
    // instead would be VACUOUS on the matched TUs — there the port's bytes ARE
    // the reference's — and UNDEFINED on the 844 the port refuses, which is
    // exactly the population the curve is for.
    //
    // Diagnostic only. Nothing below branches on it and no verdict anywhere
    // depends on it; `plan-*` is an instrument and the byte judge is unchanged.
    let observed_plan = captured.ref_obj.observe();
    res.plan = super::plan::grade(
        observed_plan.as_ref(),
        &c2_core::plan::predict(
            &captured.bundle,
            &c2_core::plan::PlanInputs {
                function_level_linking: gy,
            },
        ),
    );
    // **R2 AT WORKLOAD SCALE**, not on three synthetic objs. The prereg's
    // tertiary criterion says `observe` must agree with each existing `c2-obj`
    // accessor "over the whole workload, TU by TU", and what shipped was three
    // hand-written cells. This carries the emit-set half — the one whose sum is
    // published as a denominator — over every TU that captured. Known answer 0.
    super::plan::record_accessor_agreement(
        &mut res.plan,
        observed_plan.as_ref(),
        captured.ref_obj.text_comdat_functions(),
    );
    res.gl_body_starts = captured.bundle.gl_body_start_coverage();
    // **W-SELBIND — the same question one instrument tighter, and read for every
    // class for the same reason.** `gl_body_starts` asks whether a segment's
    // body-start offset is SPELLED in `.gl` and its own doc says `present` is a
    // deliberate over-count; this asks whether a RECORD NAMES it, which is what a
    // binding needs. On `src/system/math/vec.cpp` the two read 373 and **36**.
    res.selective_bind = captured.bundle.selective_bind_coverage();
    if let Some((records, segments, mangled, inline_fit)) = res.selective_bind {
        let key = |k: &str| k.to_string();
        if records > 0 && records < segments {
            *res.bind_checks.entry(key("selbind-selective-tus")).or_insert(0) += 1;
            // The totality clause, reported rather than decided: `(0, 0)` is the
            // only value at which a selective binding may stand.
            if mangled == 0 && inline_fit == 0 {
                *res.bind_checks.entry(key("selbind-total-tus")).or_insert(0) += 1;
            }
            if mangled != 0 {
                *res.bind_checks.entry(key("selbind-blocked-mangled-tus")).or_insert(0) += 1;
            }
            if inline_fit != 0 {
                *res.bind_checks
                    .entry(key("selbind-blocked-inline-fit-tus"))
                    .or_insert(0) += 1;
            }
        } else if records == segments && records > 0 {
            *res.bind_checks.entry(key("selbind-one-to-one-tus")).or_insert(0) += 1;
        }
        // **THE JOIN w-phase7b §10 item 3 left open: is `emitted ⊆ claimed`?**
        //
        // Per emitted symbol — the reference obj's own `.text` COMDAT leaders,
        // the same denominator `emit-emitted` counts — does a `.gl` record NAME
        // it? Asked under both framings, because they are a different set on a
        // real TU (#2783) and the gate only has the first: `gl_gate_record_names`
        // is `codec::gl_offset_framed`, what `Bindings::selective` reads, and
        // `gl_body_record_names` is the window-free framing the instrument runs.
        //
        // This is the measurement that says whether a selective binding has a
        // denominator at all. A TU where some emitted symbol is unclaimed can
        // never be bound selectively however good the accounting gets, because
        // the port would emit an obj missing that function.
        if let (Some(gl), Some(emitted)) = (
            captured.bundle.get("gl"),
            captured.ref_obj.text_comdat_functions(),
        ) {
            // **W-FRAME783 — FOUR readers, because the gate-vs-wide gap has TWO
            // terms and was published as if it had one.** #2824 attributed the
            // whole 34 → 414 to #2783's framing; the framing is shipped now and
            // the gate's number did not move. `narrow` and `precise` are the
            // same walk-free scan as `wide` under the two other framings, so
            // the gap decomposes instead of being attributed:
            //
            //   narrow  → precise : what the FRAMING is worth (this lane's ship)
            //   precise → wide    : the 551 framed offsets that are not `.ex`
            //                       split points, i.e. how much of 414 is noise
            //   precise → gate    : what the WALK's six stop clauses cost
            let gate = c2_il::gl_gate_record_names(gl);
            let wide = c2_il::gl_body_record_names(gl);
            let narrow = c2_il::gl_narrow_record_names(gl);
            let precise = c2_il::gl_precise_record_names(gl);
            let (mut ng, mut nw, mut nn, mut np) = (0usize, 0usize, 0usize, 0usize);
            for e in &emitted {
                if gate.contains(e) {
                    ng += 1;
                }
                if wide.contains(e) {
                    nw += 1;
                }
                if narrow.contains(e) {
                    nn += 1;
                }
                if precise.contains(e) {
                    np += 1;
                }
            }
            *res.bind_checks.entry(key("selbind-emitted")).or_insert(0) += emitted.len();
            *res.bind_checks.entry(key("selbind-emitted-named-gate")).or_insert(0) += ng;
            *res.bind_checks.entry(key("selbind-emitted-named-wide")).or_insert(0) += nw;
            *res.bind_checks
                .entry(key("selbind-emitted-named-scan-narrow"))
                .or_insert(0) += nn;
            *res.bind_checks
                .entry(key("selbind-emitted-named-scan-precise"))
                .or_insert(0) += np;
            if !emitted.is_empty() {
                *res.bind_checks.entry(key("selbind-emit-tus")).or_insert(0) += 1;
                if ng == emitted.len() {
                    *res.bind_checks
                        .entry(key("selbind-emit-subset-gate-tus"))
                        .or_insert(0) += 1;
                }
                if nw == emitted.len() {
                    *res.bind_checks
                        .entry(key("selbind-emit-subset-wide-tus"))
                        .or_insert(0) += 1;
                }
                if nn == emitted.len() {
                    *res.bind_checks
                        .entry(key("selbind-emit-subset-scan-narrow-tus"))
                        .or_insert(0) += 1;
                }
                if np == emitted.len() {
                    *res.bind_checks
                        .entry(key("selbind-emit-subset-scan-precise-tus"))
                        .or_insert(0) += 1;
                }
            }
        }
    }

    // 1b. P2b function-level census — runs regardless of the TU class below, so
    //     even a `vocab-gap` TU contributes its per-function ranking. This is
    //     the only measurement that moves before whole TUs come in class.
    if let Some(census) = captured.bundle.census_functions() {
        res.fn_total = census.len();
        for (f, gate) in &census {
            if f.verdict.in_class() {
                res.fn_in_class += 1;
                // 1c. The cross-check: run the port's own per-function selector
                //     over every function the census claims. A refusal here is a
                //     census/gate disagreement, and it is recorded under its own
                //     key rather than left as a rumour — the numerator is the
                //     public claim, so its error term has to be measured on every
                //     scan (roadmap #44, `docs/GAPS.md` §6).
                let key = match gate {
                    Err(e) => Some((*e).to_string()),
                    Ok(func) => match c2_core::codegen::opt_mode_of_word(f.opt_word) {
                        Err(_) => Some("opt-mode".to_string()),
                        Ok(mode) => c2_core::codegen::function_gate(func, mode, gy)
                            .err()
                            .map(|e| gate_key(&e.to_string())),
                    },
                };
                if let Some(k) = key {
                    *res.fn_gate_refusals.entry(k).or_insert(0) += 1;
                }
            } else {
                *res.fn_blockers.entry(f.verdict.key()).or_insert(0) += 1;
            }
            // The D6 frame axis, over *every* function: the in-class shapes are
            // the control group (all of them are leaves or single tail calls, so
            // a `calls-2plus` reading among them would indict the measure).
            *res.fn_frames
                .entry(format!("{}|{}", f.frame_class(), f.verdict.key()))
                .or_insert(0) += 1;
            // The control-flow axis, likewise over every function — the in-class
            // shapes are the control group here too, and they must all read
            // `cflow-straight`.
            *res.fn_cflow.entry(f.cflow.clone()).or_insert(0) += 1;
            // **§14.2 step 5's fail-closed boundary, scored over every body in
            // the corpus** (lane `w-stmt5`), on the same `|IN-CLASS` /
            // `|BLOCKED` cross the axes above use.
            //
            // **Its OWN map.** It was written into `fn_cflow` first, with a
            // comment arguing a seventh map was unnecessary because it is the
            // same walk's verdict over the same population. That was wrong:
            // `cflow_residue_control` sweeps every `fn_cflow` row ending
            // `|IN-CLASS` into its off-class total, and the published
            // `cflow-residue-inclass-offclass` went 517,425 -> 1,222,684. The
            // field's own doc records it; `TuResult::fn_cflow_off` had already
            // written the rule down, one field away, in those words.
            //
            // The IN-CLASS column is the control and it is the interesting one:
            // a body the port already ACCEPTS AND EMITS BYTE-EXACTLY, that this
            // predicate refuses, is a measured unit of the predicate being
            // narrower than the shipped class — the same two-sided error
            // `cflow_residue_control` publishes for the residue, asked of the
            // boundary instead.
            *res.fn_cfg_admit
                .entry(format!(
                    "{}|{}",
                    f.cfg_admit,
                    if f.verdict.in_class() { "IN-CLASS" } else { "BLOCKED" }
                ))
                .or_insert(0) += 1;
            if f.cflow.starts_with("cflow-") {
                *res.fn_cflow
                    .entry(format!("{}|{}", f.cflow, f.verdict.key()))
                    .or_insert(0) += 1;
                // …and the same class crossed with the **population**, on the
                // `|IN-CLASS` / `|BLOCKED` spelling the EH axis below already
                // uses. This is a second cross rather than a reading of the
                // first, because reading the first means asking whether a
                // census key is a blocker key, and that is a guess about a
                // namespace `FnVerdict::key` shares between the two.
                //
                // **What it is FOR — the residue predicate's own denominator.**
                // A `cflow-*+expr-modeled` row claims "this body is blocked on
                // control flow ALONE", and that claim is only as good as the
                // vocabulary `CfResidue::Modeled` tests against — a hand-written
                // mirror of the port's accepted class
                // (`c2-il .../shapes/control_flow.rs`). Nothing checked that
                // mirror against the port, so it could fall arbitrarily far
                // behind while the counterfactual it produces stayed quotable.
                // The in-class rows are the check: a body the port ACCEPTS that
                // the residue calls off-class is a measured unit of staleness.
                // Published, never folded in — trap 0, and `w-inread`'s rule
                // that a denominator is not published until it has been printed
                // on both sides of a change.
                *res.fn_cflow
                    .entry(format!(
                        "{}|{}",
                        f.cflow,
                        if f.verdict.in_class() { "IN-CLASS" } else { "BLOCKED" }
                    ))
                    .or_insert(0) += 1;
                // …and the DECOMPOSITION of the off-class side, board #1345.
                // The row above says a body is not `Modeled`; this one says
                // which of `control_flow`'s twenty-one arms decided that. It is
                // the half of the pair #1345 says a widening owes: a repair set
                // measured rather than guessed, scoreable on BOTH sides of the
                // two-sided error before anything is widened.
                //
                // Only for bodies that HAVE a reason — an empty string is
                // `+expr-modeled` or a body with no scan, and folding those in
                // would make the largest row of the table the one that says
                // nothing (`eh-bare|empty-dtor-delegation`'s lesson, six
                // paragraphs up).
                if !f.cflow_off.is_empty() {
                    *res.fn_cflow_off
                        .entry(format!(
                            "{}|{}",
                            f.cflow_off,
                            if f.verdict.in_class() { "IN-CLASS" } else { "BLOCKED" }
                        ))
                        .or_insert(0) += 1;
                }
            }
            // The EH axis, likewise over every function — and here the in-class
            // shapes are more than a control group: the `empty-dtor-*` buckets
            // ARE the cheap side of the boundary, so any of them reading
            // anything but the cheap key would say the axis is wrong.
            //
            // **The cross says which population a row is in, and it must.**
            // `FnVerdict::key` spells IN-CLASS labels and BLOCKER keys into one
            // namespace, and this cross used to be `"<eh>|<key>"` for both. On
            // one scan that made `eh-bare|empty-dtor-delegation` — 27,501 —
            // the largest row of the whole EH cross, and `empty-dtor-delegation`
            // is an ACCEPTED shape. Anyone ranking off the table ranked a control
            // group, and one nearly got scheduled as a rung. The population is
            // now in the key, and there is a per-class `|BLOCKED` subtotal so a
            // blocked stock can be sized without knowing the in-class label
            // strings by heart.
            *res.fn_eh.entry(f.eh.clone()).or_insert(0) += 1;
            let pop = if f.verdict.in_class() { "INCLASS" } else { "BLOCKED" };
            if !f.verdict.in_class() {
                *res.fn_eh.entry(format!("{}|BLOCKED", f.eh)).or_insert(0) += 1;
            }
            *res.fn_eh
                .entry(format!("{}|{pop}|{}", f.eh, f.verdict.key()))
                .or_insert(0) += 1;
            // The two DISPATCH axes, over every function — same row shapes as the
            // EH cross above and for the same reason: `FnVerdict::key` spells
            // accepted shapes and blockers into one namespace, so the population
            // has to be in the key or an accepted control group reads like a rung.
            //
            // The bare totals are emitted for EVERY function, so both axes sum to
            // the census and a body that reached no tagged site is a printed row
            // rather than a hole. That is the whole discipline here: this axis
            // exists because 30,475 functions were previously reported only as
            // "none of the three productions was entered", which is an absence,
            // and an absence cannot be ranked.
            *res.fn_dispatch.entry(f.dispatch.to_string()).or_insert(0) += 1;
            *res.fn_prod.entry(f.prod.to_string()).or_insert(0) += 1;
            let complete = f.verdict.completeness().name();
            *res.fn_complete.entry(complete.to_string()).or_insert(0) += 1;
            if !f.verdict.in_class() {
                *res.fn_complete
                    .entry(format!("{complete}|BLOCKED"))
                    .or_insert(0) += 1;
                *res.fn_complete
                    .entry(format!("{complete}|{}", f.verdict.key()))
                    .or_insert(0) += 1;
            }
            if !f.verdict.in_class() {
                *res.fn_dispatch
                    .entry(format!("{}|BLOCKED", f.dispatch))
                    .or_insert(0) += 1;
                *res.fn_prod
                    .entry(format!("{}|BLOCKED", f.prod))
                    .or_insert(0) += 1;
            }
            *res.fn_dispatch
                .entry(format!("{}|{pop}|{}", f.dispatch, f.verdict.key()))
                .or_insert(0) += 1;
            // The production cross is emitted only for the bodies that actually
            // reached a member-call production. Crossing `prod-not-entered` with a
            // census key would restate the dispatch axis under a second name, and
            // it is the dispatch axis that owns that population.
            if f.prod != "prod-not-entered" {
                *res.fn_prod
                    .entry(format!("{}|{pop}|{}", f.prod, f.verdict.key()))
                    .or_insert(0) += 1;
            }
            // …and the migration cross: the measured `maxState` axis against the
            // refuted statement-count one it replaces (`docs/EH_RECORDS.md` §9.4,
            // §10). This is what reconciles §7.3's published split with the real
            // one instead of silently replacing it.
            *res.fn_eh
                .entry(format!("eh-migrate|{}|{}", f.eh, f.eh_stmt))
                .or_insert(0) += 1;
            // 1d. The binding invariant (D14): what did the `.gl` symbol index
            //     say a generated destructor delegates to? A destructor, always —
            //     anything else is a binding the oracle would have had no chance
            //     to catch, because these bodies rarely reach an emitter.
            if f.verdict.key().starts_with("empty-dtor") {
                if let Ok(func) = gate {
                    let k = match &func.tail_call {
                        Some(c) => dtor_callee_class(c),
                        None => "none",
                    };
                    *res.bind_checks
                        .entry(format!("dtor-callee-{k}"))
                        .or_insert(0) += 1;
                }
            }
        }
        // 1e. **The emitted-function census** (`docs/GAPS.md` §8). The census
        //     above counts IL bodies; this counts the functions c2 *emitted*, and
        //     says how many of those the port's accepted class covers. It is the
        //     only reading of the numerator that a byte compare could ever have
        //     graded, because it is the only one restricted to code that appears
        //     in an obj.
        //
        //     The join is: census row --(`.gl` body-offset record)--> mangled name
        //     --(`.text` COMDAT leader)--> emitted. Both halves fail closed, and
        //     every failure lands in a printed residue row rather than adjusting
        //     a denominator downwards, which would inflate the ratio.
        match captured.ref_obj.text_comdat_functions() {
            None => {
                *res.emit.entry("emit-obj-unreadable".into()).or_insert(0) += 1;
            }
            Some(emitted) => {
                // Which rows claim which emitted symbol. A symbol two rows claim
                // is not bound to either — `EmitBinding` already drops those, so
                // this can only see a repeat when two DISTINCT sections carry one
                // name, which is itself a thing to count rather than to average.
                let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
                for (i, (f, _)) in census.iter().enumerate() {
                    if let Some(n) = f.emit_name.as_deref() {
                        claim.entry(n).or_default().push(i);
                    }
                }
                *res.emit.entry("emit-emitted".into()).or_insert(0) += emitted.len();
                // **W-EMITSET — the residue split that decides the ceiling.**
                // §9.16.3 ceilings TU match at 25/871 because the port emits one
                // COMDAT per `.ex` segment with no emit-set model. The ceiling on
                // any *model* over `.ex` segments is different and harder: the
                // port can only ever emit a COMDAT for a body it HAS, under the
                // name the binding gives it. So an emitted symbol no row claims
                // is one of two completely different things, and they had been
                // reported as one number:
                //
                // * it has a framed `.gl` body record — the body IS in this
                //   bundle and `EmitBinding` merely lost the row. **Instrument
                //   defect**, closable in `bind.rs`.
                // * it has none — no body, so a segment-driven port must
                //   SYNTHESIZE the COMDAT. **A wall**, and a different phase.
                //
                // `emit-set-ceiling-*` below turns that into a per-TU predicate.
                let body_records = captured
                    .bundle
                    .get("gl")
                    .map(c2_il::gl_body_record_names)
                    .unwrap_or_default();
                let mut unbound_with_body = 0usize;
                let mut unbound_no_body = 0usize;
                // The witness list's third predicate (board #159, `witness_path`):
                // is the name in `.gl` AT ALL? Built only when witnesses are on,
                // and deliberately NOT used by any counter above — a name present
                // as a run with no framed body record is a different fact from
                // both "binds to a row" and "has a body record", and §10.14 is the
                // record of what conflating two of the three costs.
                let (gl_runs, gl_index): (
                    std::collections::BTreeSet<String>,
                    std::collections::BTreeSet<String>,
                ) = match (witness_path(), captured.bundle.get("gl")) {
                    (Some(_), Some(gl)) => (
                        c2_il::mangled_names(gl).into_iter().collect(),
                        c2_il::gl_symbol_index(gl).into_values().collect(),
                    ),
                    _ => Default::default(),
                };
                let witness = |res: &mut TuResult, bucket: String, name: &str| {
                    if witness_path().is_some() {
                        res.emit_witness.push(WitnessRow {
                            bucket,
                            name: name.to_string(),
                            in_gl_runs: gl_runs.contains(name),
                            in_gl_index: gl_index.contains(name),
                        });
                    }
                };
                for name in &emitted {
                    if matches!(claim.get(name.as_str()).map(Vec::as_slice), Some([_])) {
                        // The CONTROL population. Whatever story the residue's
                        // names suggest has to be false of the symbols that DO
                        // bind, or it is a story about mangled names in general
                        // and not about the residue.
                        wall_dump(src, name, "bound");
                        continue;
                    }
                    if body_records.contains(name) {
                        unbound_with_body += 1;
                        *res.emit.entry("emit-unbound-has-record".into()).or_insert(0) += 1;
                        wall_dump(src, name, "has-record");
                        witness(&mut res, "emit-unbound-has-record".into(), name);
                    } else {
                        unbound_no_body += 1;
                        *res.emit.entry("emit-unbound-no-record".into()).or_insert(0) += 1;
                        let key = format!("emit-unbound-no-record|{}", mangling_class(name));
                        *res.emit.entry(key.clone()).or_insert(0) += 1;
                        wall_dump(src, name, "no-record");
                        witness(&mut res, key, name);
                    }
                }
                // The three per-TU ceilings, as counts of TUs (0 or 1 each here;
                // summed over the scan by the report). Stated as *nested* bounds
                // so the order of attack is legible:
                //
                //  * `today`     — every emitted symbol already binds to a row.
                //                  The ceiling on a model built on today's binding.
                //  * `repaired`  — every emitted symbol at least HAS a body record.
                //                  The ceiling if `bind.rs` were perfect.
                //  * `wall`      — this TU has an emitted symbol with no body at
                //                  all, so no binding repair can reach it.
                if unbound_with_body == 0 && unbound_no_body == 0 {
                    *res.emit.entry("emit-set-ceiling-today".into()).or_insert(0) += 1;
                }
                if unbound_no_body == 0 {
                    *res.emit.entry("emit-set-ceiling-repaired".into()).or_insert(0) += 1;
                } else {
                    *res.emit.entry("emit-set-ceiling-wall".into()).or_insert(0) += 1;
                }
                for name in &emitted {
                    match claim.get(name.as_str()).map(Vec::as_slice) {
                        Some([row]) => {
                            *res.emit.entry("emit-bound".into()).or_insert(0) += 1;
                            let f = &census[*row].0;
                            if f.verdict.in_class() {
                                *res.emit.entry("emit-in-class".into()).or_insert(0) += 1;
                            } else {
                                *res.emit_blockers.entry(f.verdict.key()).or_insert(0) += 1;
                                // **The control-flow counterfactual, on the
                                // column that ranks.** `emit_blockers` is the
                                // widening order and its #2 row is
                                // `body-cflow-label`; but a row's size is not
                                // its yield, and for this row the yield is
                                // "emitted functions a block IR alone would
                                // convert". That is a cross of the emitted
                                // population with the control-flow axis, and it
                                // existed nowhere — `fn_cflow` is over every
                                // body and `emit_blockers` carries no shape. So
                                // the emitted counterfactual could only ever be
                                // re-derived by hand, which is why it had not
                                // been re-derived since 2026-07-31.
                                //
                                // Three counters, nested, so the bracket is
                                // legible rather than a single number:
                                //   -branchy         the block IR must serve it
                                //   -branchy-modeled …and its operand vocabulary
                                //                    is inside what
                                //                    `CfResidue::Modeled` tests
                                //                    — the counterfactual, and a
                                //                    LOWER bound, because that
                                //                    vocabulary is the stale
                                //                    mirror the control above
                                //                    measures.
                                // Read the two together or not at all.
                                if cflow_needs_block_ir(&f.cflow) {
                                    *res.emit.entry("emit-cflow-branchy".into()).or_insert(0) += 1;
                                    if f.cflow.ends_with("+expr-modeled") {
                                        *res.emit
                                            .entry("emit-cflow-branchy-modeled".into())
                                            .or_insert(0) += 1;
                                    }
                                }
                                // **THE SAME CROSS, AS A SERIES** (lane
                                // `w-stmt5`). The two counters above collapse
                                // seven `CfShape`s into one boolean and then
                                // throw away the largest of them: the
                                // `-branchy` predicate is
                                // [`cflow_needs_block_ir`], which excludes
                                // `cflow-straight` by name. So the emitted
                                // column — the population `fnbyte-refused-parse`
                                // is counted over — has never been able to see
                                // the shape `IL_STMT_GRAMMAR.md` §14.2 step 5's
                                // `29 <tok>` production serves most often, and
                                // a lane pricing that step off `-branchy` is
                                // pricing it off a bucket its own population is
                                // not in.
                                //
                                // `w-slots` board **#3147** is the rule this
                                // obeys: a number read off one cell is right
                                // for that cell and wrong as a law, and only
                                // varying the structural count separates them.
                                // The structural count here is the CFG shape,
                                // so the shape is what varies and the series is
                                // what is published.
                                //
                                // **Bucketed, never sharded.** A body the
                                // scanner did not decode carries a `cf-*`
                                // blocker key, and those are per-TU sharded in
                                // places (`GAPS.md` §6). All of them land in one
                                // `undecoded` bucket: the series is over shapes,
                                // and "no shape was determined" is one answer,
                                // not four hundred.
                                *res.emit
                                    .entry(format!(
                                        "emit-cflow-shape|{}",
                                        cflow_series_bucket(&f.cflow)
                                    ))
                                    .or_insert(0) += 1;
                                // The partition control, counted in the SAME
                                // unit and at the SAME site as the buckets it
                                // controls. `w-tag02`'s rule is why it is
                                // incremented here and not recomputed from
                                // `emit_blockers` later: an identity whose two
                                // sides are counted in different units reads 0
                                // forever and is green for the wrong reason.
                                *res.emit
                                    .entry("emit-cflow-shape-accounted".into())
                                    .or_insert(0) += 1;
                                // **THE WIDENING ORDER, RESTRICTED TO THE
                                // POPULATION A READER STEP CAN ACTUALLY REACH.**
                                //
                                // `emit_blockers` ranks all 113,612 and is the
                                // order every reader lane has been dispatched
                                // off. But a reader step can only move a body it
                                // can MODEL, and `CfResidue::Modeled` is this
                                // tree's own name for that — so the rows that
                                // rank are not the rows that are reachable, and
                                // the two lists have never been printed side by
                                // side. This is `emit_blockers` filtered to
                                // `+expr-modeled`, and it is small enough to
                                // read whole.
                                //
                                // It is what settles a question no aggregate
                                // can: `body-cflow-label` is rank 2 of
                                // `emit_blockers` at 2,832 and is §14.2 step 5's
                                // headline row. If it does not appear here, then
                                // every one of those 2,832 is blocked on the
                                // expression layer as well and step 5 converts
                                // none of them — measured on the grading unit's
                                // own population instead of inferred from the
                                // bodies one (**#3107**'s rule, which exists
                                // because a published ceiling read off the wrong
                                // population was wrong by 117x).
                                if f.cflow.ends_with("+expr-modeled") {
                                    *res.emit
                                        .entry(format!(
                                            "emit-cflow-modeled-key|{}",
                                            f.verdict.key()
                                        ))
                                        .or_insert(0) += 1;
                                }
                            }
                        }
                        Some(_) => {
                            *res.emit.entry("emit-name-two-rows".into()).or_insert(0) += 1;
                        }
                        None if is_compiler_generated(name) => {
                            *res.emit
                                .entry("emit-residue-generated".into())
                                .or_insert(0) += 1;
                        }
                        None => {
                            *res.emit.entry("emit-residue-unbound".into()).or_insert(0) += 1;
                            // …and WHAT it is, by mangling class. A residue
                            // reported only as a number is a rumour; these rows
                            // are what a follow-up lane would attack, and they
                            // are also the check on the story: if the residue
                            // were really "c2 synthesized it", it would be
                            // concentrated in the special-member classes, and if
                            // it is spread across ordinary `?…` functions then
                            // the BINDING is losing them and the reader is what
                            // needs work.
                            *res.emit
                                .entry(format!("emit-residue-unbound|{}", mangling_class(name)))
                                .or_insert(0) += 1;
                        }
                    }
                }
                // 1e''. **W-AFAIL — the ROW side of factor A**
                //       (`docs/rungs/_2026-08-04-w-afail-findings.md`; the board
                //       row is PROPOSED, not minted — 196–205 was contended by
                //       four concurrent lanes on 2026-08-04 and BOARD.md's
                //       "#143–#146 minted twice" contradiction is what a number
                //       minted inside a worktree costs).
                //
                //       Every key above walks `emitted` — the obj's `.text`
                //       COMDAT leaders — and asks which census row claims each.
                //       That is one half of factor A. A is a count *equality*
                //       (`.ex` segments == COMDATs, step 1g), so its failure is
                //       a signed integer, and the surplus direction — an IL body
                //       with no COMDAT — is invisible to every key above,
                //       because it is not a member of `emitted`.
                //
                //       This partitions the census rows the same way, so that
                //       `n − c` decomposes into named populations rather than
                //       staying one unexplained number:
                //
                //       | key | means |
                //       |---|---|
                //       | `afail-row-emitted` | the row's `emit_name` IS a `.text` leader — the CONTROL population |
                //       | `afail-row-not-emitted` | named, and c2 **discarded** the body: a compiler fact, and Phase 7's actual subject |
                //       | `afail-row-unnamed` | the `.gl` binding gave the row no name: an **instrument limit**, not a compiler fact |
                //
                //       **The third bucket is the point of splitting it out.**
                //       Folding it into the second would report "c2 discarded
                //       this body" about rows whose names we never had, which is
                //       the shape ROADMAP §10.14 charges for. The three sum to
                //       `fn_total` by construction, and that identity is checked
                //       as a control rather than asserted here: it is an
                //       *absence* detector (§9.18.8), because a block that never
                //       runs also produces no broken identity.
                //
                //       Additive only. No existing count is read or written.
                let emitted_set: std::collections::BTreeSet<&str> =
                    emitted.iter().map(String::as_str).collect();
                for (f, _) in census.iter() {
                    match f.emit_name.as_deref() {
                        None => {
                            *res.emit.entry("afail-row-unnamed".into()).or_insert(0) += 1;
                        }
                        Some(n) if emitted_set.contains(n) => {
                            *res.emit.entry("afail-row-emitted".into()).or_insert(0) += 1;
                        }
                        Some(n) => {
                            *res.emit.entry("afail-row-not-emitted".into()).or_insert(0) += 1;
                            *res.emit
                                .entry(format!("afail-row-not-emitted|{}", mangling_class(n)))
                                .or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        // 1e''''. **FUNCTION BYTE MATCH (lane w-fuzzy, `super::fnbytes`).** The
        //         byte-exact differential at *function* granularity: for every
        //         `.text` COMDAT c2 emitted, is the port's own per-function
        //         selection byte-identical to it? Additive only — it reads the
        //         same census this block bound and writes only `fnbyte-` keys.
        //
        //         This is the term STATUS trap 2 was missing. `emit-in-class` is
        //         a parse-time claim the oracle has never graded on a TU that
        //         does not match; `fnbyte-exact` is the oracle's own predicate,
        //         applied to a unit the port can actually answer for.
        super::fnbytes::measure(&mut res, &census, &captured.ref_obj);
        // 1e'. SCRATCH INSTRUMENT (W-ADJUST, boards #127/#128) — see [`row_dump`].
        //      Off unless `C2RS_ROW_DUMP` is set; changes no count either way.
        row_dump(
            src,
            &census,
            captured.ref_obj.text_comdat_functions().as_deref(),
        );
    }

    // 1f. The emit binding's own self-report. Printed on every scan, whether or
    //     not the obj was readable, so a residue cannot disappear by the route
    //     `prod-entered-untagged` had to be dragged out of: an absence.
    if let Some(b) = captured.bundle.emit_binding() {
        for (key, n) in [
            ("emit-records", b.records),
            ("emit-record-outside", b.records_outside),
            ("emit-record-nameless", b.records_nameless),
            ("emit-row-conflict", b.dropped_row_conflict),
            ("emit-name-conflict", b.dropped_name_conflict),
        ] {
            if n > 0 {
                *res.emit.entry(key.into()).or_insert(0) += n;
            }
        }
        let (found, accounted) = b.accounting();
        if found != accounted {
            *res.emit.entry("emit-accounting-broken".into()).or_insert(0) += 1;
        }
        // **ARITY (#144, W-VGL).** `emit-records` counts records as entities and
        // the totality identity above balances them against a residue; neither
        // can see a change that keeps every record and loses something *inside*
        // one. `record_offsets` is the contents — a property of the FRAMING only
        // — so it is reported as its own row and its own broken-invariant count.
        // A naming change (W-VGL's `26` separator) must move `emit-record-nameless`
        // and leave `emit-record-offsets` alone; a framing change moves both, and
        // a report carrying only the first cannot tell them apart.
        let (records, offsets) = b.arity();
        *res.emit.entry("emit-record-offsets".into()).or_insert(0) += offsets;
        if records != offsets {
            *res.emit.entry("emit-arity-broken".into()).or_insert(0) += 1;
        }
    }

    // 1f'. **The `.in` initializer reader's own self-report** (board #936,
    //      w-tag02). Printed on every scan for every TU that has an `.in`, not
    //      only for the few hundred `data_tu` accepts whole — a reader widening
    //      has to be measurable on the workload by the *same* instrument before
    //      and after it, and `DataTu::in_census` cannot be, because a widening
    //      changes which TUs produce one.
    //
    //      **Every residue reason is emitted, including the zeroes.** A reason
    //      that stops occurring must read `0` rather than vanish from the
    //      report; `docs/STATUS.md` trap 5 is that absence reads as success, and
    //      a residue key that disappeared is exactly that shape.
    if let Some(r) = captured.bundle.in_init_report() {
        for (key, n) in [
            ("in-init-records", r.records),
            ("in-init-accepted", r.accepted),
            ("in-init-duplicate-records", r.duplicate_records),
            // ARITY, not totality (trap 4): a reader that lost an element inside
            // a record it still accepted moves neither `records` nor `residue`.
            ("in-init-elements", r.elements),
            ("in-init-values", r.values),
            ("in-init-conflicts", r.conflicts),
            ("in-init-residue", r.residue),
            ("in-init-symrefs", r.sym_refs),
            ("in-init-records-with-symrefs", r.records_with_sym_refs),
            // **THE DENOMINATOR — board #961.** `records` is a count over the
            // population the anchor scan reaches, and until this line nothing
            // printed how large the rest is. All three are counted by the
            // reader without changing what it accepts or where it resumes; see
            // `InInitReport::unanchored`. `docs/STATUS.md` trap 0 is a control
            // whose denominator is chosen by the same predicate that decides
            // its numerator, and these are the numbers that make that visible
            // one level down from #937.
            ("in-init-unanchored", r.unanchored),
            ("in-init-fail-closed", r.fail_closed),
            ("in-init-no-token", r.no_token),
        ] {
            *res.emit.entry(key.into()).or_insert(0) += n;
        }
        for (reason, n) in &r.residue_by_reason {
            *res.emit.entry(format!("in-init-residue-{reason}")).or_insert(0) += n;
        }
        // **Totality, as a printed control and not as an assertion.** Every
        // record that framed either decoded or is a named residue entry.
        //
        // This control FIRED at 826 of 878 TUs when tag `02` landed, and it was
        // right to: the identity it was written with counted `values` (TOKENS)
        // against `records` (RECORDS), which coincide only while no two accepted
        // records share a token. That was true of the scalar-only population and
        // false the moment the accepted set grew. Board **#937**.
        if r.accepted + r.residue != r.records {
            *res.emit.entry("in-init-accounting-broken".into()).or_insert(0) += 1;
        }
    }

    // 1f''. **W-PHASE7 — the tag-0x10 ALIAS channel, and the ONE resolution
    //       site that exists in `crates/` today.**
    //
    //       `rungs/_2026-08-04-w-emitp-findings.md` §6 is five steps; steps 1,
    //       2 and 5 shipped with `c2_il::gl_alias_table` (lane `w-alias`,
    //       `d2bdadc`) and had **no consumer**. Steps 3 and 4 are consumer
    //       rules, and the only place in this workspace that turns an `.in`
    //       tag-02 target token into an emitted symbol name is
    //       `IlBundle::data_tu`'s relocation naming.
    //
    //       Four things are printed and they are four different questions:
    //
    //       * `alias-*` — the decode's own invariants, measured **in Rust over
    //         this workload** rather than inherited from the 850-TU Python.
    //         Both nulls ride along: a field position quoted without its
    //         shifted read is a field position that was searched for.
    //       * `alias-dom-emitted` — `dom(alias) ∩ E` against the **real obj**.
    //         The Python measured this over its own truth dump; here it is the
    //         same join the differential makes. **KNOWN ANSWER 0.**
    //       * `alias-inref-*` — the reachable population at the resolution
    //         site, whatever any writer does with it.
    //       * `alias-datatu-relocs-alias` / `alias-emit-names` — the *live*
    //         population, i.e. what the port would name today. **KNOWN ANSWER
    //         0 for both**, and both are currently a **zero denominator**
    //         (`alias-datatu-relocs` is 0), which is printed as a zero
    //         denominator and never as a passed test.
    //
    //       …and a fifth group, `alias-weak-*` / `alias-rule-*`, which is
    //       where the channel's real obj-level observable turned out to be.
    //
    //       Every key prints its zero, and the nulls are counts and not
    //       statuses (`docs/STATUS.md` trap 5).
    if let Some(gl) = captured.bundle.get("gl") {
        let alias = c2_il::gl_alias_table(gl);
        let st = alias.stats();
        let m1 = c2_il::gl_alias_table_shifted(gl, -1);
        let p1 = c2_il::gl_alias_table_shifted(gl, 1);
        for (key, n) in [
            ("alias-runs", st.runs),
            ("alias-tag10", st.tag10),
            ("alias-head-fail", st.head_fail),
            ("alias-rt-fail", st.rt_fail),
            ("alias-unbound-target", st.unbound_target),
            ("alias-self", st.self_alias),
            ("alias-dup", st.dup),
            ("alias-bound", st.bound),
            ("alias-shape-e-to-g", st.shape_e_to_g),
            // The precondition for §6 step 4, carried on every scan rather
            // than asserted once in a test: suppressing a name that HAS a body
            // is a symbol deletion, not a filter.
            ("alias-dom-with-body", st.dom_with_body),
            // THE NULL, shipped rather than described.
            ("alias-null-m1-bound", m1.stats().bound),
            ("alias-null-m1-shape", m1.stats().shape_e_to_g),
            ("alias-null-p1-bound", p1.stats().bound),
            ("alias-null-p1-shape", p1.stats().shape_e_to_g),
        ] {
            *res.emit.entry(key.into()).or_insert(0) += n;
        }
        // `dom(alias) ∩ E` and the targets that ARE emitted, joined against the
        // reference obj's own `.text` COMDAT leaders — the same list
        // `emit-emitted` counts, so the two denominators are the same one.
        if let Some(emitted) = captured.ref_obj.text_comdat_functions() {
            let set: std::collections::BTreeSet<&str> =
                emitted.iter().map(String::as_str).collect();
            let mut dom_e = 0usize;
            let mut tgt_e = 0usize;
            for (a, t) in alias.iter_names() {
                if set.contains(a) {
                    dom_e += 1;
                }
                if set.contains(t) {
                    tgt_e += 1;
                }
            }
            *res.emit.entry("alias-dom-emitted".into()).or_insert(0) += dom_e;
            *res.emit.entry("alias-target-emitted".into()).or_insert(0) += tgt_e;
            if dom_e > 0 {
                *res.emit.entry("alias-dom-emitted-tus".into()).or_insert(0) += 1;
            }
        }
        // **THE ORACLE-SIDE QUESTION, and it is the one that decides what a
        // consumer of §6 step 3 should DO.**
        //
        // `dom(alias) ∩ E = 0` says c2 never *defines* an alias. It does not say
        // c2 never *names* one in a relocation — an undefined external is a
        // perfectly legal relocation target, and the port's `data_tu` names its
        // relocation targets out of exactly the token an alias would occupy.
        // So: read the real obj's own relocation targets, over **every**
        // section, and count how many name a `dom(alias)` symbol.
        //
        // `alias-obj-reloc-alias` is the number that decides it, and **it is
        // 4,248 over 675 of 871 objs**: c2 does NOT resolve, it leaves the
        // record naming `??_E<X>` at the vftable slot (`.rdata`, `ADDR32`) and
        // at an adjustor thunk's branch (`.text`, `REL24`). A consumer that
        // "resolved" here would write a name c2 never writes.
        //
        // `alias-obj-reloc-target` is the same count for the alias's *target*
        // and is printed beside it so that a `0` on either could never be read
        // as *"vftable relocations were not in this population"* — a zero
        // denominator is not a passed test (`w-emitp` §4's `StreamNull.cpp`
        // rule).
        if let Some(rows) = captured.ref_obj.relocs_named() {
            let tgts: std::collections::BTreeSet<&str> =
                alias.iter_names().map(|(_, t)| t).collect();
            let mut named_alias = 0usize;
            let mut named_target = 0usize;
            for (_, _, _, t) in &rows {
                if let c2_obj::RelocTarget::Symbol(n) = t {
                    if alias.is_alias(n) {
                        named_alias += 1;
                    }
                    if tgts.contains(n.as_str()) {
                        named_target += 1;
                    }
                }
            }
            *res.emit.entry("alias-obj-relocs".into()).or_insert(0) += rows.len();
            *res.emit.entry("alias-obj-reloc-alias".into()).or_insert(0) += named_alias;
            *res.emit.entry("alias-obj-reloc-target".into()).or_insert(0) += named_target;
            if named_alias > 0 {
                *res.emit.entry("alias-obj-reloc-alias-tus".into()).or_insert(0) += 1;
            }
        } else {
            *res.emit.entry("alias-obj-reloc-unreadable".into()).or_insert(0) += 1;
        }
        // **THE ANSWER, and it is a THIRD thing neither §6 step 3 nor step 4
        // describes: the alias is realised as a COFF WEAK EXTERNAL.**
        //
        // `alias-obj-reloc-alias` above is **4,248 over 675 TUs** — c2 leaves
        // its relocations naming `??_E<X>` and does *not* substitute the
        // target. What it writes instead is a symbol record
        //
        //     class WEAK_EXTERNAL  ??_E<X>  -> default ??_G<X>, SEARCH_ALIAS
        //
        // so `dom(alias) ∩ E = 0` is a statement about COMDAT **leaders**, and
        // a consumer that "resolved" the alias at a relocation would emit a
        // name c2 does not write. See `ObjImage::weak_externals`.
        //
        // That makes this the strongest grade the table has ever had: not a
        // per-TU conjunction over a closure, but a **per-record pairing against
        // c2's own symbol table**. Five keys, and the two disagreement
        // directions are separate because they mean opposite things:
        //
        // * `alias-weak-predicted` — `(name, default)` is exactly a table entry;
        // * `alias-weak-default-disagree` — the name is in `dom(alias)` and c2's
        //   default is a **different** symbol. **KNOWN ANSWER 0**, and an alarm:
        //   the decode read the wrong token;
        // * `alias-weak-unpredicted` — c2 wrote a weak external the table has
        //   no entry for. Recall's error term;
        // * `alias-unrealized` — a table entry with no weak external in this
        //   obj. Precision's error term, and NOT necessarily a defect: the
        //   emit-set model's own claim is that an alias is realised only when
        //   something reaches it.
        if let Some(weaks) = captured.ref_obj.weak_externals() {
            let mut predicted = 0usize;
            let mut disagree = 0usize;
            let mut unpredicted = 0usize;
            let mut nonalias_class = 0usize;
            let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
            for (name, default, ch) in &weaks {
                // **Measured `Characteristics` = 2 =
                // `IMAGE_WEAK_EXTERN_SEARCH_LIBRARY`**, on the HamUser.cpp
                // probe and, per this key, on all of them. Counted rather than
                // filtered on: a weak external with another characteristic is a
                // shape nothing here measured, and it must be visible rather
                // than silently pooled. **KNOWN ANSWER 0.**
                if *ch != 2 {
                    nonalias_class += 1;
                }
                if alias.is_alias(name) {
                    seen.insert(name.as_str());
                    if alias.resolve_name(name) == default {
                        predicted += 1;
                    } else {
                        disagree += 1;
                    }
                } else {
                    unpredicted += 1;
                }
            }
            *res.emit.entry("alias-weak-records".into()).or_insert(0) += weaks.len();
            *res.emit.entry("alias-weak-predicted".into()).or_insert(0) += predicted;
            *res.emit.entry("alias-weak-default-disagree".into()).or_insert(0) += disagree;
            *res.emit.entry("alias-weak-unpredicted".into()).or_insert(0) += unpredicted;
            *res.emit.entry("alias-weak-not-search-library".into()).or_insert(0) += nonalias_class;
            *res.emit.entry("alias-unrealized".into()).or_insert(0) +=
                alias.len().saturating_sub(seen.len());
            if weaks.len() == predicted && alias.len() == seen.len() && !weaks.is_empty() {
                *res.emit.entry("alias-weak-exact-tus".into()).or_insert(0) += 1;
            }
            if !weaks.is_empty() {
                *res.emit.entry("alias-weak-tus".into()).or_insert(0) += 1;
            }
            // **THE REALISATION RULE, stated and graded per record.**
            //
            // 94,250 of the 98,263 table entries produce no weak external, so
            // *which* entries are realised is a real predicate and not a
            // rounding error. The rule the totals suggest —
            //
            //     R:  c2 writes the weak external `a -> t`  IFF  `t` is a
            //         `.text` COMDAT leader of this same obj
            //
            // — is graded here with **both** error terms separate, because they
            // are different mistakes: `-miss` is the rule promising a record c2
            // did not write, `-extra` is c2 writing one the rule did not
            // promise. A rule reported as one "disagreement" count could trade
            // them off against each other and read better than it is.
            //
            // This is the *first* rule in the project stated over the alias
            // channel that a single obj can refute, which is why it is a
            // standing key and not a paragraph.
            if let Some(emitted) = captured.ref_obj.text_comdat_functions() {
                let leaders: std::collections::BTreeSet<&str> =
                    emitted.iter().map(String::as_str).collect();
                let written: std::collections::BTreeSet<&str> =
                    weaks.iter().map(|(n, _, _)| n.as_str()).collect();
                let mut miss = 0usize;
                let mut predicted_set: std::collections::BTreeSet<&str> =
                    std::collections::BTreeSet::new();
                for (a, t) in alias.iter_names() {
                    if leaders.contains(t) {
                        predicted_set.insert(a);
                        if !written.contains(a) {
                            miss += 1;
                        }
                    }
                }
                let extra = written.iter().filter(|w| !predicted_set.contains(*w)).count();
                *res.emit.entry("alias-rule-predicted".into()).or_insert(0) += predicted_set.len();
                *res.emit.entry("alias-rule-miss".into()).or_insert(0) += miss;
                *res.emit.entry("alias-rule-extra".into()).or_insert(0) += extra;
                if miss == 0 && extra == 0 {
                    *res.emit.entry("alias-rule-exact-tus".into()).or_insert(0) += 1;
                }
            }
        } else {
            *res.emit.entry("alias-weak-unreadable".into()).or_insert(0) += 1;
        }
    }
    if let Some(r) = captured.bundle.in_alias_report() {
        for (key, n) in [
            ("alias-inref-total", r.refs),
            ("alias-inref-unbound", r.refs_unbound),
            ("alias-inref-alias", r.refs_alias),
            ("alias-inref-records", r.records_with_alias),
            ("alias-datatu-relocs", r.data_tu_relocs),
            ("alias-datatu-relocs-alias", r.data_tu_relocs_alias),
            ("alias-emit-names", r.emit_names_alias),
        ] {
            *res.emit.entry(key.into()).or_insert(0) += n;
        }
        if r.refs_alias > 0 {
            *res.emit.entry("alias-inref-tus".into()).or_insert(0) += 1;
        }
    }

    if let Some(gl) = captured.bundle.get("gl") {
        let (dropped, mangled) = c2_il::gl_symbol_conflicts(gl);
        if dropped > 0 {
            *res.bind_checks
                .entry("gl-token-ambiguous-dropped".to_string())
                .or_insert(0) += dropped;
        }
        if mangled > 0 {
            *res.bind_checks
                .entry("gl-token-conflict-mangled".to_string())
                .or_insert(0) += mangled;
        }
    }

    // 1g. **THE TWO SPLITTERS DISAGREE, AND THE CEILING IS COMPUTED WITH THE
    //     WRONG ONE** (ROADMAP §10.11 / §10.12, W-PHASE6).
    //
    //     `emit_set_reachable_tus` — the "25 of 871" emit-set ceiling, and the
    //     `at most 19 TUs, ever` claim §10 builds its re-plan on — filters on
    //     `fn_total == emit-emitted`, and its doc comment asserts that
    //     `fn_total` "is exactly that segment count", meaning the count
    //     `PortC2::build` consumes. **It is not.**
    //
    //     | count | comes from | anchored on |
    //     |---|---|---|
    //     | `fn_total` | `census_functions()` / `split_function_bodies_at` | `LO_MARKER` = `4C 4F 11` |
    //     | what the port consumes | `IlBundle::functions()` / `split_functions_at` | `FN_START` = `4F 1F` |
    //
    //     §10.12 named the population that separates them: a `??__E`/`??__F`
    //     dynamic-initializer thunk carries a **bare `4C`** with no `4F 11`, so
    //     the census sees 0 segments where the gate sees 1.
    //
    //     So this counts the ceiling BOTH ways and publishes the disagreement.
    //     It does **not** replace `fn_total` or `emit-emitted` — a ceiling
    //     silently recomputed under the same name would be indistinguishable
    //     from the old one being right, and which of the two is the ceiling is
    //     not something this instrument gets to decide.
    //
    //     **This used to read `functions().map(|f| f.len())`, and that made the
    //     whole block near-vacuous**: `functions()` is an *acceptance* decision,
    //     so it returns `None` for every `vocab-gap` TU — 865 of 871 — and the
    //     gate-anchored ceiling was knowable for exactly the 6 that already
    //     match, five of which define zero functions. The lane that added this
    //     block measured that and declined to report a number, correctly.
    //     `IlBundle::ex_segment_count` is the pure reader that closes it: the
    //     `4F 1F` split with no acceptance decision attached, available on a
    //     bundle `functions()` refuses.
    //
    //     Its `None` means **no `.ex` at all**, which is not "zero functions" —
    //     so it keeps feeding `emit-gate-segments-unknown` rather than being
    //     unwrapped to 0. Every key here is NEW; none of the existing ones is
    //     touched.
    let gate_segments = captured.bundle.ex_segment_count();
    {
        let comdats = res.emit.get("emit-emitted").copied().unwrap_or(0);
        match gate_segments {
            None => {
                *res.emit
                    .entry("emit-gate-segments-unknown".into())
                    .or_insert(0) += 1;
            }
            Some(n) => {
                *res.emit.entry("emit-gate-segments-known".into()).or_insert(0) += 1;
                *res.emit.entry("emit-gate-segments".into()).or_insert(0) += n;
                if n == res.fn_total {
                    *res.emit.entry("emit-splitter-agree".into()).or_insert(0) += 1;
                } else {
                    *res.emit.entry("emit-splitter-disagree".into()).or_insert(0) += 1;
                    // Signed, in two keys rather than one absolute value: §10.12
                    // predicts the gate seeing MORE segments than the census
                    // (`??__E` with a bare `4C`), and a count that cannot tell
                    // that from the opposite is not evidence for it.
                    let key = if n > res.fn_total {
                        "emit-splitter-gate-sees-more"
                    } else {
                        "emit-splitter-census-sees-more"
                    };
                    *res.emit.entry(key.into()).or_insert(0) += 1;
                }
                // The ceiling, gate-anchored, and how it moves against the
                // `LO`-anchored one. `enter`/`leave` are the deliverable: the
                // net is not enough, because two TUs swapping sides is a
                // different fact from nothing happening.
                let (lo_reach, gate_reach) = (res.fn_total == comdats, n == comdats);
                if gate_reach {
                    *res.emit.entry("emit-set-ceiling-gate".into()).or_insert(0) += 1;
                }
                match (gate_reach, lo_reach) {
                    (true, false) => {
                        *res.emit.entry("emit-set-ceiling-gate-enter".into()).or_insert(0) += 1
                    }
                    (false, true) => {
                        *res.emit.entry("emit-set-ceiling-gate-leave".into()).or_insert(0) += 1
                    }
                    _ => {}
                }
            }
        }
    }

    // 1h. **FACTORS C AND D** (`docs/ROADMAP.md` §10.19, board #160).
    //
    //     §10.19 factored Phase 7 into four predicates over the graded TUs and
    //     found `A∧B∧C∧D` reproduces the match set exactly. **A and B were
    //     already keys here; C and D were not** — they lived in a one-off
    //     analysis, which means the project's central planning model could not
    //     regress-detect and the next reader re-derives it by hand. §10.14 is
    //     the record of what a by-hand re-derivation of a rule the harness owns
    //     costs.
    //
    //     | factor | predicate | key |
    //     |---|---|---|
    //     | A | `.ex` segments == obj `.text` COMDATs | `emit-set-ceiling-gate` (1g) |
    //     | B | every emitted symbol binds | `emit-set-ceiling-today` (1e) |
    //     | **C** | obj section set ⊆ [`PORT_WRITER_SECTIONS`] | `emit-sec-reachable` |
    //     | **D** | every emitted COMDAT in the port's codegen class | `emit-class-complete` |
    //
    //     **C reads the obj afresh and shares no variable with 1e, 1g or step
    //     3.** That is not fussiness: §10.18 is this file's own record of a
    //     variable with two consumers being changed for one of them, moving 865
    //     TUs between classes with nothing red. The question here — *what
    //     sections does this obj have* — is not the question `text_comdat_*`
    //     asks (*which COMDAT `.text` leaders are there*), and a `.data` or
    //     `.bss` is invisible to the second.
    //
    //     **D is built from the keys 1e already computed, never from a second
    //     in-class rule.** `emit-in-class` is the census's own
    //     `FnVerdict::in_class()` joined through the binding, so an emitted
    //     symbol that fails to bind is *not* counted in class — D fails closed,
    //     which is the direction that cannot flatter it.
    {
        match captured.ref_obj.section_names() {
            None => {
                // Fail closed and SAY SO. An unreadable obj contributes no
                // section vocabulary and is outside C — never "carries nothing
                // outside the writer's set", which would put it inside.
                *res.emit.entry("emit-sec-unreadable".into()).or_insert(0) += 1;
            }
            Some(names) => {
                *res.emit.entry("emit-sec-readable".into()).or_insert(0) += 1;
                *res.emit.entry("emit-sec-count".into()).or_insert(0) += names.len();
                let distinct: std::collections::BTreeSet<&str> =
                    names.iter().map(String::as_str).collect();
                *res.emit.entry("emit-sec-distinct".into()).or_insert(0) += distinct.len();
                let mut extra = 0usize;
                for n in &distinct {
                    // One per DISTINCT name per TU, so the aggregated row reads
                    // "objs carrying this section" and not "sections named this"
                    // — under `/Gy` the second would count 158 `.text`s in one
                    // obj and no reader of the table would know which it was.
                    *res.emit.entry(format!("emit-sec-name|{n}")).or_insert(0) += 1;
                    if !PORT_WRITER_SECTIONS.contains(n) {
                        extra += 1;
                        *res.emit.entry(format!("emit-sec-extra|{n}")).or_insert(0) += 1;
                    }
                }
                let key = if extra == 0 {
                    "emit-sec-reachable"
                } else {
                    "emit-sec-blocked"
                };
                *res.emit.entry(key.into()).or_insert(0) += 1;
            }
        }
        // **The compiler-label channel** (lane `w-loop`, board **#742**). Read
        // afresh off the reference obj for the same reason C is: it is a
        // *different question* from "which COMDAT leaders are there", and
        // sharing a variable with a walk that answers the other one is §10.18's
        // recorded defect.
        //
        // `emit-label-syms` is a COUNT and `emit-label-free` a per-TU flag, both
        // fail-closed: an obj that does not decode gets `emit-label-unreadable`
        // and **neither** of the other two, so "we could not read it" can never
        // be read as "it has no labels". Absence read as success is this
        // project's most-repeated defect and this is exactly its shape.
        match captured.ref_obj.compiler_label_symbols() {
            None => {
                *res.emit.entry("emit-label-unreadable".into()).or_insert(0) += 1;
            }
            Some(labels) => {
                *res.emit.entry("emit-label-readable".into()).or_insert(0) += 1;
                *res.emit.entry("emit-label-syms".into()).or_insert(0) += labels.len();
                if labels.is_empty() {
                    *res.emit.entry("emit-label-free".into()).or_insert(0) += 1;
                }
            }
        }
        // Factor D. Its population is **"1e's join actually ran"**, which the
        // presence of the `emit-emitted` key states exactly: 1e writes it
        // unconditionally once the census decoded and the obj's emitted set
        // read, including when the value is 0. A TU whose obj did not decode,
        // or that has no census at all, is therefore *outside* D rather than
        // vacuously inside it on `0 == 0` — the flattering direction, and the
        // one §9.18.8 records twelve times.
        if let Some(&emitted) = res.emit.get("emit-emitted") {
            let in_class = res.emit.get("emit-in-class").copied().unwrap_or(0);
            *res.emit.entry("emit-class-known".into()).or_insert(0) += 1;
            if emitted == in_class {
                *res.emit.entry("emit-class-complete".into()).or_insert(0) += 1;
            }
            // The vacuous half, counted separately because §10.19 says "6 of
            // those 8 emit nothing" and a factor that is mostly satisfied by
            // empty objs is a different fact from one that is not.
            if emitted == 0 {
                *res.emit.entry("emit-class-empty".into()).or_insert(0) += 1;
            }
        }
        // **FACTOR E — whole-TU acceptance** (board #179, §10.21/§10.22).
        //
        // The fifth term. D above asks "does the port's PER-FUNCTION acceptance
        // path take every COMDAT here", which was the only reading of "the port
        // has a route to the contents" when §10.19 was measured. `PortC2::build`
        // now tries a WHOLE-TU arm first, and this is the predicate for it.
        //
        // **Its population is the graded TUs, which is where it must be**: this
        // block runs after capture and before every early return that a `match`
        // TU could take, so E is written for every TU C and D are written for.
        // A capture-fail TU has no bundle and is outside all five, exactly as
        // `graded()` says.
        //
        // **It shares no variable with 1e, 1g or step 3.** §10.18 is this file's
        // own record of what a shared reader with two consumers costs — 865 TUs
        // moved class with nothing red — and this reads `captured.bundle`
        // directly through the registry's own function pointers. In particular it
        // does NOT call `bundle.decodes()`, whose whole job is a different
        // question (*did anything decode*, the `vocab-gap` bucket's predicate),
        // and which would silently absorb a future third acceptance path. See
        // [`WHOLE_TU_RECOGNIZERS`] for that argument in full.
        //
        // Cost note: each recognizer is a real decode. `dyninit_tu` early-outs on
        // one substring scan for `??__E` and only the ~126 TUs carrying one pay
        // for the rest, which is why this is affordable per TU in a scan that
        // grades 878 of them in under three seconds.
        {
            let mut any = false;
            for (name, accepts) in WHOLE_TU_RECOGNIZERS {
                if accepts(&captured.bundle) {
                    any = true;
                    *res.emit.entry(format!("emit-whole-tu|{name}")).or_insert(0) += 1;
                }
            }
            if any {
                *res.emit.entry("emit-whole-tu-any".into()).or_insert(0) += 1;
            }
            // `D ∨ E`, materialized as its own key so the disjunction is a thing
            // the JSONL carries per TU rather than a join two readers could
            // re-derive differently.
            if any || res.emit.contains_key("emit-class-complete") {
                *res.emit.entry("emit-emit-path".into()).or_insert(0) += 1;
            }
        }
    }

    // 2. Optional soundness lane: standalone-c2 replay must reproduce the
    //    pipeline obj on this real bundle.
    if do_replay {
        let ref_obj_path = captured.ref_obj_path.clone();
        res.replay_ok = Some(
            match tc.replay(&captured, &work.join("replay_il"), &ref_obj_path) {
                Ok(obj) => {
                    matches!(ObjImage::diff(&captured.ref_obj, &obj), ObjDiff::Identical)
                }
                Err(_) => false,
            },
        );
        // The replay must write to the reference's own `-Fo` path (that string
        // is inside the obj), which under a cache is the cache entry itself —
        // so restore the captured bytes afterwards. Without this a diverging
        // replay would leave its own output behind as the "cached capture",
        // i.e. the scan would poison its own cache with the thing it was
        // checking for.
        if cache.is_some() {
            let _ = std::fs::write(&ref_obj_path, captured.ref_obj.as_bytes());
        }
    }

    // 3. Vocabulary: can the IL model even decode this bundle's functions?
    //
    //    **This calls `functions()` itself, and must never be folded back into
    //    1g's `gate_segments`.** It was, briefly, on the true observation that
    //    `functions()` is pure and was being evaluated twice. Then 1g's reader
    //    changed from `functions()` to `ex_segment_count()` — correctly, because
    //    the ceiling needs the `4F 1F` split on TUs the gate refuses — and the
    //    shared variable silently carried that change into the class decision.
    //    `ex_segment_count` is `None` only when there is no `.ex` at all, so the
    //    vocab-gap test stopped firing: **`vocab-gap 865 -> 0` and
    //    `codegen-gap 0 -> 865` in one run**, with `mismatch` still 0 and
    //    `match` still 6.
    //
    //    That is the FALSE-GREEN direction and it is why this comment is here:
    //    a report reading "the port now decodes 865 TUs and merely declines to
    //    lower them" is a headline, and nothing in the run was red. The two
    //    predicates answer different questions —
    //
    //      | question | reader |
    //      |---|---|
    //      | how many `.ex` segments are there | `ex_segment_count` — pure, always answers |
    //      | will the gate ACCEPT this bundle  | `functions()` — an acceptance decision |
    //
    //    — and sharing one call between them makes the second silently inherit
    //    whatever the first is changed to.
    //
    //    `the_class_predicate_is_not_the_segment_counter` pins the *premise*
    //    (the two readers really do disagree on a bundle the gate refuses), so
    //    the paragraph above is executable rather than folklore. **It does not
    //    catch a re-fold** — that shows up only as the class counts moving, and
    //    `classify_one` needs a toolchain, so no portable test reaches it. The
    //    evidence on record is a 3-TU scan run both ways: folded gives
    //    `codegen-gap 2 / vocab-gap 0`, unfolded `codegen-gap 0 / vocab-gap 2`.
    //    Stated plainly so the next reader does not take the test for a guard it
    //    is not.
    //    **W-R1c: the acceptance question now has TWO paths and must be asked
    //    through one predicate.** `IlBundle::decodes()` is
    //    `functions().is_some() || dyninit_tu().is_some()`. Calling `functions()`
    //    alone here would file every converted `??__E` dynamic-initializer TU as
    //    `vocab-gap` — "the port could not decode it" — while the port emitted a
    //    byte-exact obj for it, which is the same mis-attribution this comment
    //    block already warns about in the other direction.
    if !captured.bundle.decodes() {
        res.class = TuClass::VocabGap;
        res.reason = "il function decode failed".to_string();
        // **W-VEC (#2500) — name the gate's OWN first refusal.**
        //
        // `decode_causes()` has answered "which of the eleven gates fired" since
        // lane `w-vocab` and had **no caller in this crate**, so every `vocab-gap`
        // row rendered two sizes and the fact that both acceptance paths said
        // `None`. That is `CEILING.md` §11.4 item 8's trap with the instrument
        // already built and never wired: a lane pricing a TU conversion had to
        // write a scratch patch to learn what `functions()` stopped on, and the
        // one that did not got `src/system/math/vec.cpp` priced four mechanisms
        // downstream of its actual first stop.
        //
        // Diagnostic only, and asked **after** the class is already decided by
        // `decodes()` — the same predicate, so this cannot move a verdict. The
        // anti-drift invariant is `c2_il`'s own: `causes.is_empty() == decodes`.
        let causes = captured.bundle.decode_causes();
        res.gate_cause = causes.first.map(str::to_string);
        res.gate_causes = causes.causes.iter().map(|c| c.to_string()).collect();
        // **W-PHASE7B — and say whether the binding this TU failed is
        // SATISFIABLE AT ALL.** `gate_cause` names the clause the walk stopped
        // on, which reads as a repair address; on a TU where `.gl` spells a
        // body-start offset for half its `.ex` segments there is no repair at
        // that address, because `Bindings::per_record`'s 1:1 requirement has no
        // solution on the input. Printed only when it bites, so the 848-row
        // bucket does not grow a column that is `n of n` on most of it.
        let cover = match res.gl_body_starts {
            Some((p, t)) if p < t => format!(
                "; .gl spells a body-start for {} of {} .ex segments — {} can bind to NO record",
                p,
                t,
                t - p
            ),
            _ => String::new(),
        };
        // **W-SELBIND — and say how many of those segments a RECORD NAMES**, which
        // is the number a selective binding is about and is not the number above.
        // Printed only when the two disagree, so the 848-row bucket does not grow
        // a column that repeats its neighbour.
        let named = match res.selective_bind {
            Some((rc, sg, m, i)) if rc < sg => format!(
                "; a .gl RECORD NAMES {rc} of {sg} — selective binding blocked by \
                 {m} unclaimed mangled + {i} unclaimed inline-fit run(s)"
            ),
            _ => String::new(),
        };
        res.detail = format!(
            ".ex {} B, {} .gl names — c2_il::functions() and dyninit_tu() both None; \
             gate stops at {} (all: {}){}",
            res.ex_len,
            res.fn_names,
            res.gate_cause.as_deref().unwrap_or("<none>"),
            if res.gate_causes.is_empty() {
                "<none>".to_string()
            } else {
                res.gate_causes.join(",")
            },
            format!("{cover}{named}")
        );
        return res;
    }

    // 4. The port, threaded with the reference's exact -Fo path (S_OBJNAME).
    let obj_name = c2_reference::to_wibo_path(&captured.ref_obj_path);
    // The obj's shape depends on argv the IL bundle does not record: /Gy
    // (implied by /O1 and /O2) puts each function in its own COMDAT .text.
    // Pass the project's real flags so the port can refuse rather than emit a
    // packed .text against a per-function-COMDAT reference.
    let port = PortC2::new(obj_name.clone()).with_function_level_linking(gy);
    match port.compile_to(&captured.bundle, &obj_name) {
        Ok(obj) => match ObjImage::diff(&captured.ref_obj, &obj) {
            ObjDiff::Identical => {
                res.class = TuClass::Match;
                res.reason = "byte-exact".to_string();
            }
            ObjDiff::Differs { first_offset, .. } => {
                res.class = TuClass::Mismatch;
                res.reason = "bytes diverge".to_string();
                res.detail = format!(
                    "first divergence at {first_offset} (ref {} B, port {} B)",
                    captured.ref_obj.len(),
                    obj.len()
                );
            }
        },
        Err(BackendError::NotImplemented(msg)) => {
            res.class = TuClass::CodegenGap;
            res.reason = clip(&msg, 80);
            res.detail = clip(&msg, 200);
        }
        Err(e) => {
            res.class = TuClass::PortError;
            res.reason = clip(&e.to_string(), 80);
            res.detail = clip(&e.to_string(), 200);
        }
    }
    res
}

/// Run the scan: worker pool over the source list, per-TU work subdirs.
/// `progress` is called per finished TU (from worker threads, serialized).
///
/// The scan also records **what produced the numbers** — see
/// [`Provenance`]: a scan whose corpus moved and whose `fn_total` matched anyway
/// is a scan that lied, and the denominator guard alone is proven insufficient.
pub fn gap_scan(
    tc: &Toolchain,
    cfg: &GapConfig,
    progress: &(dyn Fn(usize, usize, &TuResult) + Sync),
) -> std::io::Result<GapReport> {
    let provenance = Provenance::collect(tc, cfg.cwd.as_deref());
    let cache = match &cfg.cache {
        Some(root) => Some(CaptureCache::new(
            root.clone(),
            tc,
            cfg.cwd.as_deref(),
            cfg.validate_cache,
        )?),
        None => None,
    };
    let sources: Vec<&str> = cfg
        .sources
        .iter()
        .map(|s| s.as_str())
        .take(cfg.limit.unwrap_or(usize::MAX))
        .collect();
    let total = sources.len();
    std::fs::create_dir_all(&cfg.work)?;

    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results: Mutex<Vec<TuResult>> = Mutex::new(Vec::with_capacity(total));
    let jobs = cfg.jobs.max(1).min(total.max(1));

    std::thread::scope(|scope| {
        for worker in 0..jobs {
            let sources = &sources;
            let next = &next;
            let done = &done;
            let results = &results;
            let cache = cache.as_ref();
            scope.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= sources.len() {
                    break;
                }
                let src = sources[i];
                let work = cfg.work.join(format!("tu{i:05}"));
                let _ = std::fs::create_dir_all(&work);
                let do_replay = cfg.replay_every > 0 && i % cfg.replay_every == 0;
                let r = scan_one(tc, cfg, cache, src, &work, do_replay);
                // Bound scratch usage: captured bundles/objs for huge scans
                // add up; the JSONL is the durable record.
                let _ = std::fs::remove_dir_all(&work);
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                progress(n, total, &r);
                results.lock().unwrap().push(r);
                let _ = worker;
            });
        }
    });

    let mut results = results.into_inner().unwrap();
    results.sort_by(|a, b| a.src.cmp(&b.src));

    let cache_stats = cache
        .as_ref()
        .map(|c| c.stats())
        .unwrap_or_default();

    if let Some(path) = &cfg.plan_tsv {
        // The membership the `plan-*` counts are counts OF. Written from the
        // same `GapReport` the counts come from, so the offline re-derivation
        // (#3288) is a re-derivation of THIS scan and not of a lookalike.
        let report = GapReport {
            results: results.clone(),
            provenance: None,
            cache: cache_stats.clone(),
        };
        std::fs::write(path, super::plan::plan_tsv(&report.plan_rows()))?;
    }
    if let Some(path) = &cfg.jsonl {
        let mut f = std::fs::File::create(path)?;
        // Record 0 is the provenance header (roadmap #46). Per-TU rows below are
        // unchanged and carry no `record` field, so two scans' rows stay
        // byte-comparable; a consumer skips this one with
        // `if r.get("record"): continue`.
        let extra: Vec<(&str, String)> = vec![
            (
                // The RESOLVED root, not the spelling on the command line: the
                // cache absolutises it (board #1388) and it is the absolute form
                // that is in the key and in every obj's `S_OBJNAME`. Recording
                // `--cache work/x` here made two runs against one cache look
                // like two caches.
                "cache_root",
                match cache.as_ref() {
                    Some(c) => crate::jstr(&c.root().display().to_string()),
                    None => "null".to_string(),
                },
            ),
            (
                "cache_context",
                match cache.as_ref() {
                    Some(c) => crate::jstr(&c.context_digest()),
                    None => "null".to_string(),
                },
            ),
            ("cache_hits", cache_stats.hits.to_string()),
            ("cache_misses", cache_stats.misses.to_string()),
            ("cache_validated", cache_stats.validated.to_string()),
            ("cache_poisoned", cache_stats.poisoned.to_string()),
            ("tu_count", results.len().to_string()),
            ("replay_every", cfg.replay_every.to_string()),
            ("flags", crate::jstr(&cfg.flags.join(" "))),
        ];
        writeln!(f, "{}", provenance.to_json(&extra))?;
        for r in &results {
            let blockers = r
                .fn_blockers
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let frames = r
                .fn_frames
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let cfg_admit = r
                .fn_cfg_admit
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let cflow = r
                .fn_cflow
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let eh = r
                .fn_eh
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let dispatch = r
                .fn_dispatch
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let complete = r
                .fn_complete
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let prod = r
                .fn_prod
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let gate = r
                .fn_gate_refusals
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let binds = r
                .bind_checks
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let emit = r
                .emit
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            let emit_blockers = r
                .emit_blockers
                .iter()
                .map(|(k, n)| format!("{}:{}", crate::jstr(k), n))
                .collect::<Vec<_>>()
                .join(",");
            // **W-VEC (#2500)** — the gate's own refusal, per TU, in the machine
            // -readable stream. `gate_cause` is `null` on every TU that decodes.
            let gate_cause = match &r.gate_cause {
                None => "null".to_string(),
                Some(c) => crate::jstr(c),
            };
            let gate_causes = r
                .gate_causes
                .iter()
                .map(|c| crate::jstr(c))
                .collect::<Vec<_>>()
                .join(",");
            // **W-PHASE7B** — `null` when `.ex` or `.gl` is absent, which is a
            // different fact from `[0,0]` and has to stay different here.
            let gl_body_starts = match r.gl_body_starts {
                None => "null".to_string(),
                Some((p, t)) => format!("[{p},{t}]"),
            };
            // **W-SELBIND** — `[records, segments, unclaimed_mangled,
            // unclaimed_inline_fit]`, `null` on a bundle with no `.ex` or no
            // `.gl` for the same reason the row above is.
            let selective_bind = match r.selective_bind {
                None => "null".to_string(),
                Some((rc, sg, m, i)) => format!("[{rc},{sg},{m},{i}]"),
            };
            writeln!(
                f,
                "{{\"src\":{},\"class\":{},\"reason\":{},\"detail\":{},\"ex_len\":{},\"fn_names\":{},\"replay_ok\":{},\"fn_total\":{},\"fn_in_class\":{},\"gate_cause\":{gate_cause},\"gate_causes\":[{gate_causes}],\"gl_body_starts\":{gl_body_starts},\"selective_bind\":{selective_bind},\"fn_blockers\":{{{}}},\"fn_frames\":{{{}}},\"fn_cflow\":{{{}}},\"fn_cfg_admit\":{{{}}},\"fn_eh\":{{{}}},\"fn_dispatch\":{{{}}},\"fn_complete\":{{{}}},\"fn_prod\":{{{}}},\"fn_gate_refusals\":{{{}}},\"bind_checks\":{{{}}},\"emit\":{{{}}},\"emit_blockers\":{{{}}}}}",
                crate::jstr(&r.src),
                crate::jstr(r.class.label()),
                crate::jstr(&r.reason),
                crate::jstr(&r.detail),
                r.ex_len,
                r.fn_names,
                match r.replay_ok {
                    None => "null".to_string(),
                    Some(b) => b.to_string(),
                },
                r.fn_total,
                r.fn_in_class,
                blockers,
                frames,
                cflow,
                cfg_admit,
                eh,
                dispatch,
                complete,
                prod,
                gate,
                binds,
                emit,
                emit_blockers,
            )?;
        }
    }

    // **The diff-signature sink** (board #976, `super::fndiff`). One row per
    // `fnbyte-differs` FUNCTION, not per TU, so it is a separate file from the
    // scan's own JSONL rather than a field inside it: joining the two is a
    // `tu` key away, and nesting thousands of function rows inside a TU row
    // would make the per-TU record unreadable by every existing consumer.
    //
    // The rows exist on `TuResult` whether or not this path is taken, so the
    // `fndiff-*` counters printed by every scan are never conditional on a flag
    // — a census that only some invocations produce is a census that goes stale
    // without anybody noticing.
    if let Some(path) = &cfg.fndiff_jsonl {
        let mut f = std::fs::File::create(path)?;
        for r in &results {
            for row in &r.fndiff {
                writeln!(f, "{row}")?;
            }
        }
    }

    let report = GapReport {
        results,
        provenance: Some(provenance),
        cache: cache_stats,
    };
    // Board #159's step one: print the names the scan just classified. Written
    // last, from the collected results, so the ranking is over the whole scan
    // and not over whatever one worker happened to see.
    if let Some(p) = witness_path() {
        write_witness(&report, p)?;
    }
    // **The two segment counts, side by side** (step 1g). Printed here rather
    // than in the caller's report because the classification lives in this file;
    // printed ALWAYS, as counts, because a disagreement nobody prints is the
    // absence-reads-as-success shape this project has paid for repeatedly.
    // Neither number is presented as "the" ceiling: which anchor the ceiling
    // should use is a decision, and this is the measurement it needs.
    let (known, unknown, agree, disagree, gate_more, census_more, gate_ceil, enter, leave) =
        report.splitter_disagreement();
    let (viol, viol_pop) = report.emit_set_violations_gate();
    // **IR0 — the container codec's round-trip over the workload** (lane `ir0`).
    // Printed as well as keyed, and printed ALWAYS, for the reason the block
    // below is: a defect count nobody prints is the absence-reads-as-success
    // shape. `broken` has a known answer of **0**; anything else is a decoding
    // bug in `ExToken::encode_into` on a stream shape the 386-fixture spread
    // has no instance of, and the per-suffix residue names which stream.
    {
        let rt_tus = report.emit_total("ir0-roundtrip-tus");
        let rt_ok = report.emit_total("ir0-roundtrip-ok");
        let rt_bad = report.emit_total("ir0-roundtrip-broken");
        println!(
            "\nIR0 CONTAINER ROUND-TRIP (K1, `IlModel::parse`/`encode`) — LOSSLESS or it \
             refuses; run on the workload for the first time 2026-08-20\n\
             \x20 {rt_ok} of {rt_tus} captured bundles re-encode BYTE-IDENTICALLY; {rt_bad} \
             broken (known answer 0)\n\
             \x20 totality is BY CONSTRUCTION — unrecognized bytes coalesce into \
             `Span::Opaque`, so a break is never an unmodelled construct, only an \
             `ExToken::encode_into` disagreeing with the bytes it consumed"
        );
        let residue: Vec<(String, usize)> = report
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("ir0-roundtrip-broken-")?.to_string(), n)))
            .collect();
        if residue.is_empty() {
            println!("\x20 residue by reason: none");
        } else {
            for (r, n) in residue {
                println!("\x20 residue {r}: {n}");
            }
        }
    }
    println!(
        "\nSPLITTER ANCHORS (ROADMAP §10.11/§10.12) — the census splits `.ex` on `4C 4F 11`, \
         the port on `4F 1F`\n\
         \x20 gate-side segment count KNOWN for {known} of {} captured TUs; UNKNOWN for {unknown} \
         (no `.ex` at all). Read through `IlBundle::ex_segment_count`, a PURE reader of the \
         `4F 1F` split — not `functions()`, which is an acceptance decision and returns None \
         for every vocab-gap TU, leaving this knowable for only the 6 that already match.\n\
         \x20 of the {known} known: {agree} agree with `fn_total`, {disagree} disagree \
         ({gate_more} where the gate sees MORE segments, {census_more} where the census does)\n\
         \x20 emit-set ceiling, LO-anchored, over ALL graded TUs: {}\n\
         \x20 emit-set ceiling, GATE-anchored (`4F 1F`, what the port consumes), over the \
         {known} known: {gate_ceil} (+{enter} entering, -{leave} leaving vs the LO-anchored set)\n\
         \x20 gate-anchored control on matching TUs: {viol} violations over {viol_pop} matching \
         TUs whose gate count is known",
        known + unknown,
        report.emit_set_reachable_tus().len(),
    );
    print_factorization(&report);
    // **The per-TU factor membership** — the rows the joints above are counts
    // of. Written to a file rather than stdout because this same command grades
    // the generated case corpus one lane at a time; the absence of the file is
    // stated positively rather than left silent, because "the membership was
    // never asked for" and "the membership is empty" must not look alike
    // (`docs/STATUS.md` trap 5).
    match &cfg.factors_tsv {
        Some(p) => {
            std::fs::write(p, report.factor_tsv())?;
            println!(
                "\x20 GAP-FACTOR-TUS — per-TU A/B/C/D/E membership for {} graded TUs written to \
                 {}. Every joint above (B and C, A and B and C, the FRONTIER) is a COUNT and is \
                 re-derivable from these rows; a count is what a lane holding a per-TU set of \
                 its own cannot intersect with.",
                report.graded().count(),
                p.display()
            );
        }
        None => println!(
            "\x20 GAP-FACTOR-TUS: NOT REQUESTED — pass `--factors-tsv PATH` for the per-TU \
             A/B/C/D/E membership. The joints above are counts; `|{{some per-TU set}} and B and \
             C|` cannot be got from one, and multiplying a rate by a joint count is how `B and \
             C` stayed published at 107 after C moved."
        ),
    }
    Ok(report)
}
