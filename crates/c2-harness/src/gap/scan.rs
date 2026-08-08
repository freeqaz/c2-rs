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

use crate::capture_cache::CaptureCache;
use crate::provenance::Provenance;

use super::classify::{
    cflow_needs_block_ir, clip, dtor_callee_class, gate_key, is_compiler_generated, mangling_class,
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
        fn_eh: BTreeMap::new(),
        fn_dispatch: BTreeMap::new(),
        fn_complete: BTreeMap::new(),
        fn_prod: BTreeMap::new(),
        fn_gate_refusals: BTreeMap::new(),
        bind_checks: BTreeMap::new(),
        emit: BTreeMap::new(),
        emit_blockers: BTreeMap::new(),
        emit_witness: Vec::new(),
        fndiff: Vec::new(),
    };

    // 1. Capture: real flags, real cwd, strace keeps bundle + obj. Served from
    //    the content-addressed cache when one is configured — the cache dir IS
    //    the capture dir, so the `-Fo` path c2 bakes into the obj is a function
    //    of the key and a hit is byte-identical to the capture that filled it
    //    (`crate::capture_cache`).
    let capture_result = match cache {
        Some(c) => c.capture(tc, src, &cfg.flags, cfg.cwd.as_deref(), work).0,
        None => tc.capture_reference_with(src, work, &cfg.flags, cfg.cwd.as_deref()),
    };
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
    res.fn_names = captured
        .bundle
        .get("gl")
        .map(|gl| c2_il::mangled_names(gl).len())
        .unwrap_or(0);

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
        res.detail = format!(
            ".ex {} B, {} .gl names — c2_il::functions() and dyninit_tu() both None",
            res.ex_len, res.fn_names
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

    if let Some(path) = &cfg.jsonl {
        let mut f = std::fs::File::create(path)?;
        // Record 0 is the provenance header (roadmap #46). Per-TU rows below are
        // unchanged and carry no `record` field, so two scans' rows stay
        // byte-comparable; a consumer skips this one with
        // `if r.get("record"): continue`.
        let extra: Vec<(&str, String)> = vec![
            (
                "cache_root",
                match &cfg.cache {
                    Some(p) => crate::jstr(&p.display().to_string()),
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
            writeln!(
                f,
                "{{\"src\":{},\"class\":{},\"reason\":{},\"detail\":{},\"ex_len\":{},\"fn_names\":{},\"replay_ok\":{},\"fn_total\":{},\"fn_in_class\":{},\"fn_blockers\":{{{}}},\"fn_frames\":{{{}}},\"fn_cflow\":{{{}}},\"fn_eh\":{{{}}},\"fn_dispatch\":{{{}}},\"fn_complete\":{{{}}},\"fn_prod\":{{{}}},\"fn_gate_refusals\":{{{}}},\"bind_checks\":{{{}}},\"emit\":{{{}}},\"emit_blockers\":{{{}}}}}",
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
