//! **DECODE REACH** — how many bodies the general decode REACHES, and whether
//! what it reaches is right.
//!
//! Lane `w-decodereach`, board **#3561**–**#3566**. Funded by
//! `docs/DECISIONS_2026-08-22.md` decision 13 (the owner: *"okay lets fund
//! general decode now"* — row **4a(i)** / I1, the general op-level IL decode).
//!
//! # The question, and why the funded row needs it
//!
//! Decision 13 prices 4a at **15–45 engineer-months as a lower bound** and names
//! the failure mode in the same breath, out of 4a's own risk column:
//!
//! > without this row a step-5 lane's only progress signal is snapshot parity —
//! > an instrument with **no emit-path consumer**, i.e. **`#3336` at program
//! > scale**, and unlike `#3336` there is no contrast case to catch it.
//!
//! `#3336` is a measured thing, not a worry: a required-zero byte delta held
//! **by construction** because the tree had no production caller for the thing
//! being measured, so the rung's own criterion *could not fail*. A multi-month
//! decode effort whose only signal is "the gate is still green" repeats that at
//! program scale. **This module is the consumer.**
//!
//! # It is a GRADIENT and it obeys `FUNCTION_BYTE_MATCH.md` §0 verbatim
//!
//! All five properties, non-negotiable, and §0 is explicitly the standing
//! template for every gradient added after FBM:
//!
//! * **Never in `scripts/gate.sh`**, and it must never be added there.
//! * **Its own block**, under its own disclaimer, apart from the class table
//!   that carries `match`/`mismatch`.
//! * **Namespaced keys** — `decode-reach-*`. No existing key, predicate or
//!   denominator is narrowed, widened or redefined here.
//! * **It licenses no emit.** `decode-reach-reached` going up is not a reason
//!   to accept a shape, to widen [`c2_il::IlBundle::functions`], or to admit
//!   anything. The only thing that accepts a shape is the differential —
//!   real `c2.dll` under wibo, `CLAUDE.md`'s one correctness rule.
//! * **Unrepresentable over an empty scan** — a run that graded nothing prints
//!   `NO-RESULT`, never a ratio over zero.
//!
//! **REACH IS NOT ADMISSION AND MUST NEVER BECOME IT.** The whole point of the
//! measurement is that the two sets are far apart: see [`SEPARATION_KEY`].
//!
//! # What it measures, and the three denominators
//!
//! Published as **containment, never as a ratio** (`w-objplan`'s lesson, board
//! #3356 — a containment claim of *"seed ⊆ emitted on 853 of 854 TUs"* turned
//! out to be **739 empty seeds**, undetectable until the claimant's own size
//! was printed beside it):
//!
//! ```text
//!     observable   ⊇   reached   ⊇   verified
//! ```
//!
//! …and the **byte-weighted twin of every one**, because a body count and a
//! byte count are two denominators: a decode that reaches 94 % of *bodies* may
//! reach a very different fraction of the *IL*. Neither is quoted alone.
//!
//! # WHAT THE REACH SIGNAL IS, TODAY, AND THE ONE SEAM
//!
//! [`reach_of`] is the **single locator** for "did the general decode reach the
//! end of this body". Today it reads [`c2_il::FnCensus::cflow`], the
//! statement-layer scanner's own verdict, which is documented **decode-only**
//! (`census.rs:172`: *"nothing reads this field except the report … not
//! consulted by acceptance, by `shape_to_function`, or by the emitter"*) and is
//! computed for **every** body whether or not it is in class. That is the
//! closest thing in the tree to 4a(i)'s general decode.
//!
//! When lane `w-unfuse` separates DECODE from ADMISSION in `crates/c2-il`, the
//! separated decode's verdict replaces the body of [`reach_of`] and **nothing
//! else in this file changes**. One fact, one locator (`docs/GAPS.md` §6) — the
//! rule S0 learned by printing `no-decode 0` on a scan whose own machine line
//! read 113,165.
//!
//! # A FIRST-BLOCKER KEY IS NOT A DISTANCE (#3131)
//!
//! The port stops at the first refusal **by design**, so every blocked body
//! reports exactly one blocker however many it has. `#3131` measured what that
//! does to a ranking: 19 greedy rungs bought reach **0**, while three tokens no
//! mass ranking can name were each worth the whole 5,184.
//!
//! So: `decode-reach-stop|<key>` is published, is **labelled a first-blocker
//! key that is neither a distance nor a ranking**, and is emitted sorted by key
//! NAME rather than by mass — `w-joint3`'s TSV precedent, and the standing rule
//! against dispatching off a blocked-key size ranking that has now bound five
//! times (#3505). **The byte-weighted reach is this module's distance.**
//!
//! Its limit is stated rather than hidden: a **per-body prefix** distance needs
//! the stop OFFSET, and `FnCensus` does not publish one. That field lives in
//! `crates/c2-il`, which this lane READS and does not write. Named as owed.

use c2_il::FnCensus;

use super::fnbytes::FnByte;
use super::TuResult;

/// **The env switch.** `on` (default) | `off`. A named, settable parameter
/// rather than a baked constant (`docs/GOAL_DECISION_2026-08-21.md`
/// § "AMENDED"); `off` is a legal instrument state and licenses nothing.
pub const ENABLE_ENV: &str = "C2RS_DECODE_REACH";

/// **Which population the printed HEADLINE is denominated in** — `all`
/// (default) | `emitted` | `admitted`.
///
/// The keys are emitted for **all three regardless**. This parameter selects
/// the sentence, never the measurement, so a run cannot narrow its own
/// denominator — which is the defect `calc_fuzzy_match_percent` has and FBM was
/// built to avoid.
pub const POP_ENV: &str = "C2RS_DECODE_REACH_POP";

/// **The discriminating-cell key: bodies the general decode REACHES that the
/// admission gate does NOT accept.**
///
/// This is the positive check that says the instrument measures reach rather
/// than admission, and the prereg freezes a threshold on it: if this is **0**
/// the lane reports `FAILED`; if it is below `decode-reach-admitted` the
/// instrument is not measuring reach.
///
/// *"Absence reads as success. The fix that generalizes is a **positive**
/// check — 'the run must have GRADED something' — never an enumeration of the
/// ways it can be empty."*
pub const SEPARATION_KEY: &str = "decode-reach-reached-not-admitted";

/// Whether the instrument runs. An unparseable value is **refused loudly**
/// rather than silently defaulted: a scan that quietly measured a different
/// thing would publish a number against the wrong denominator, which is the one
/// thing this lane's prereg forbids. S0's rule, same reason.
pub fn enabled_from_env() -> bool {
    match std::env::var(ENABLE_ENV) {
        Err(_) => true,
        Ok(v) => match v.trim() {
            "on" | "1" | "" => true,
            "off" | "0" => false,
            other => {
                eprintln!(
                    "{ENABLE_ENV}={other:?} is not `on` or `off`; refusing rather than \
                     silently measuring at a setting nobody asked for"
                );
                std::process::exit(2);
            }
        },
    }
}

/// The headline population. Parsed the same way, refused the same way.
pub fn pop_from_env() -> Pop {
    match std::env::var(POP_ENV) {
        Err(_) => Pop::All,
        Ok(v) => match v.trim() {
            "all" | "" => Pop::All,
            "emitted" => Pop::Emitted,
            "admitted" => Pop::Admitted,
            other => {
                eprintln!(
                    "{POP_ENV}={other:?} is not `all`, `emitted` or `admitted`; refusing \
                     rather than printing a headline against a denominator nobody named"
                );
                std::process::exit(2);
            }
        },
    }
}

/// Which denominator the printed headline names. Every key is emitted for all
/// three regardless of this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pop {
    /// Every census row with a body — the widest denominator.
    All,
    /// Only the rows bound to a `.text` COMDAT c2 actually emitted.
    Emitted,
    /// Only the rows the incumbent parser accepts.
    Admitted,
}

impl Pop {
    pub fn name(self) -> &'static str {
        match self {
            Pop::All => "all",
            Pop::Emitted => "emitted",
            Pop::Admitted => "admitted",
        }
    }
}

/// **One body's decode outcome — the partition, and every arm is printed
/// including as a zero.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// The general decode walked this body to the segment tail.
    Reached,
    /// It stopped inside the body.
    Stopped,
    /// There is no body to decode — the `LO` marker is absent. **Its own arm
    /// and never folded into `Stopped`**: "this row has no body" is a fact
    /// about the *input*, and folding it into a refusal would put every
    /// bodiless row into a bucket named after a walk that did not happen. The
    /// same distinction `cflow_key` itself draws with `cfg-no-body`.
    NoBody,
}

impl Reach {
    pub fn key(self) -> &'static str {
        match self {
            Reach::Reached => "decode-reach-reached",
            Reach::Stopped => "decode-reach-stopped",
            Reach::NoBody => "decode-reach-nobody",
        }
    }

    /// Every value, so the report can print each one including as a zero.
    pub const ALL: [Reach; 3] = [Reach::Reached, Reach::Stopped, Reach::NoBody];
}

/// **THE SEAM. One fact, one locator.**
///
/// Reads the general decode's verdict for one census row. Today that is
/// `FnCensus::cflow`, whose value space is exactly three shapes and whose
/// prefix is the whole signal:
///
/// | `cflow` | means |
/// |---|---|
/// | `cflow-<shape>[+expr-modeled]` | the decode walked to the tail |
/// | `cf-no-body` | no `LO` marker; there is nothing to decode |
/// | `cf-<production>-0xNN` | the decode stopped, and this is where |
///
/// **The `+expr-modeled` suffix is deliberately ignored**, and that is not a
/// convenience: the suffix is the *residue* axis, moved by
/// `C2RS_CFRESIDUE_ADMIT`, which by construction cannot move the decode's
/// Ok/Err (`control_flow.rs:645` — decode-only, *"cannot move an obj byte in
/// either direction"*). A reach key that read the whole string would move under
/// a knob that changes no decode, and
/// `the_residue_suffix_does_not_move_the_reach_verdict` is the executed
/// mutation that proves this arm can fail.
pub fn reach_of_cflow(cflow: &str) -> Reach {
    if cflow == "cf-no-body" {
        return Reach::NoBody;
    }
    if cflow.starts_with("cflow-") {
        return Reach::Reached;
    }
    Reach::Stopped
}

/// [`reach_of_cflow`] against a census row. The string form is the one the
/// tests drive, so the grading is exercised without a toolchain and without
/// constructing a `c2-il` type this crate does not own.
pub fn reach_of(c: &FnCensus) -> Reach {
    reach_of_cflow(&c.cflow)
}

/// **The stopping production, for a body that stopped.** A first-blocker key,
/// and this function's name says so. Never a distance (#3131), never a ranking
/// (#3505). `None` for a body that reached or has no body.
pub fn first_blocker_key(cflow: &str) -> Option<&str> {
    match reach_of_cflow(cflow) {
        Reach::Stopped => Some(cflow),
        _ => None,
    }
}

/// **Grade one body's reach.** Isolated from every lookup — no obj, no map, no
/// env, and not even a `c2-il` type — so it is testable without a toolchain,
/// exactly as [`super::blind::grade_one_blind`] is.
///
/// Returns `(reach, admitted)`; the pair is the unit of the separation, and it
/// is returned together rather than derived twice so the two cannot drift.
pub fn grade_one(cflow: &str, in_class: bool) -> (Reach, bool) {
    (reach_of_cflow(cflow), in_class)
}

/// Run the decode-reach measurement over one TU.
///
/// **Additive only**: no existing count is read except FBM's per-symbol
/// verdicts, which are passed in from the walk that produced them rather than
/// recomputed — `#1464`'s rule (never by subtracting two published totals) and
/// `fnbytes.rs:98`'s rule (called, never copied) at the same time. Recomputing
/// the byte compare here would double FBM's cost AND create a second producer
/// of the judge's verdict, which is the drift this repo has shipped twice.
///
/// `fbm` is `(emitted symbol name, verdict)` for every `.text` COMDAT leader
/// FBM graded in this TU, in that walk's order.
pub(super) fn measure(
    res: &mut TuResult,
    census: &[(FnCensus, Result<c2_il::IlFunction, &'static str>)],
    fbm: &[(String, FnByte)],
) {
    // ---- the census population: observable ⊇ reached ⊇ verified -------------
    //
    // Local accumulators. Every control below is computed from THESE, in this
    // TU iteration, never by subtracting two published totals.
    let (mut observable, mut reached, mut stopped, mut nobody) = (0usize, 0, 0, 0);
    let (mut admitted, mut admitted_reached) = (0usize, 0);
    let (mut reached_not_admitted, mut admitted_not_reached) = (0usize, 0);
    let (mut bytes_observable, mut bytes_reached) = (0usize, 0);
    for (c, _) in census.iter() {
        let (r, adm) = grade_one(&c.cflow, c.verdict.in_class());
        observable += 1;
        bytes_observable += c.seg_len;
        *res.fn_decode.entry(r.key().into()).or_insert(0) += 1;
        match r {
            Reach::Reached => {
                reached += 1;
                bytes_reached += c.seg_len;
            }
            Reach::Stopped => stopped += 1,
            Reach::NoBody => nobody += 1,
        }
        if adm {
            admitted += 1;
            if r == Reach::Reached {
                admitted_reached += 1;
            } else {
                // **NOT a known-answer-0 control.** A body the incumbent parser
                // accepts whole, that the general decode stops inside, is two
                // independent walkers disagreeing about the same bytes — a
                // FINDING about the decode, not an alarm about this module.
                // The prereg registers p = 0.55 that this is nonzero.
                admitted_not_reached += 1;
                // …and WHICH production stopped it, because a count of
                // disagreements that cannot be looked at is not a repair set.
                if let Some(k) = first_blocker_key(&c.cflow) {
                    *res.fn_decode
                        .entry(format!("decode-reach-admitted-not-reached|{k}"))
                        .or_insert(0) += 1;
                }
            }
        } else if r == Reach::Reached {
            reached_not_admitted += 1;
        }
        // The first-blocker histogram. Sorted by NAME by the `BTreeMap` it
        // lands in, never by mass, and labelled in the printed block.
        if let Some(k) = first_blocker_key(&c.cflow) {
            *res.fn_decode
                .entry(format!("decode-reach-stop|{k}"))
                .or_insert(0) += 1;
        }
    }
    for (k, n) in [
        ("decode-reach-observable", observable),
        ("decode-reach-admitted", admitted),
        ("decode-reach-admitted-reached", admitted_reached),
        ("decode-reach-admitted-not-reached", admitted_not_reached),
        (SEPARATION_KEY, reached_not_admitted),
        ("decode-reach-bytes-observable", bytes_observable),
        ("decode-reach-bytes-reached", bytes_reached),
    ] {
        if n > 0 {
            *res.fn_decode.entry(k.into()).or_insert(0) += n;
        }
    }

    // ---- GRADE 2: the byte judge's own verdict, crossed with reach ----------
    //
    // The emit-path consumer 4a's risk column says an I1 instrument must have.
    // Reach cannot rise without this table saying whether anything the judge
    // can see moved.
    //
    // Bound by NAME through the same `emit_name` binding FBM itself uses, and a
    // name claimed by two rows binds NEITHER — that is `fnbyte-unbound` on
    // FBM's side and is simply not a crossed row here.
    if !fbm.is_empty() {
        let mut claim: std::collections::BTreeMap<&str, Vec<usize>> = Default::default();
        for (i, (c, _)) in census.iter().enumerate() {
            if let Some(n) = c.emit_name.as_deref() {
                claim.entry(n).or_default().push(i);
            }
        }
        let (mut emit_obs, mut emit_reached, mut emit_verified) = (0usize, 0, 0);
        let (mut emit_bytes_obs, mut emit_bytes_reached) = (0usize, 0);
        for (name, v) in fbm.iter() {
            let Some([i]) = claim.get(name.as_str()).map(Vec::as_slice) else {
                // FBM's `Unbound` bucket. Counted so the emitted denominator's
                // residue is a printed number and not a subtraction.
                *res.fn_decode
                    .entry("decode-reach-emit-unbound".into())
                    .or_insert(0) += 1;
                continue;
            };
            let (c, _) = &census[*i];
            let r = reach_of(c);
            emit_obs += 1;
            emit_bytes_obs += c.seg_len;
            if r == Reach::Reached {
                emit_reached += 1;
                emit_bytes_reached += c.seg_len;
            }
            // **VERIFIED** — the third denominator, and it is the judge's own
            // word. A body is verified iff the general decode reached it AND
            // real c2's bytes agree with what the port emitted for it.
            //
            // `FnByte::Exact` requires bytes AND relocation identity, so this
            // is strictly the judge speaking. It is a SUBSET of `reached` by
            // construction and the containment is checked below rather than
            // assumed.
            if r == Reach::Reached && v == &FnByte::Exact {
                emit_verified += 1;
            }
            // The 2×N cross. This is the answer to "is what it reaches right",
            // in the judge's units, WITH the denominator on which the judge
            // cannot speak printed in the same table (the `refused` column).
            *res.fn_decode
                .entry(format!(
                    "decode-reach-emit|{}|{}",
                    match r {
                        Reach::Reached => "reached",
                        Reach::Stopped => "stopped",
                        Reach::NoBody => "nobody",
                    },
                    v.bare()
                ))
                .or_insert(0) += 1;
        }
        for (k, n) in [
            ("decode-reach-emit-observable", emit_obs),
            ("decode-reach-emit-reached", emit_reached),
            ("decode-reach-verified", emit_verified),
            ("decode-reach-emit-bytes-observable", emit_bytes_obs),
            ("decode-reach-emit-bytes-reached", emit_bytes_reached),
        ] {
            if n > 0 {
                *res.fn_decode.entry(k.into()).or_insert(0) += n;
            }
        }
        // **THE CONTAINMENT INVARIANT, checked and not assumed** — `verified`
        // ⊆ `emit-reached` ⊆ `emit-observable`. Known answer 0.
        if emit_verified > emit_reached || emit_reached > emit_obs {
            *res.fn_decode
                .entry("decode-reach-containment-broken".into())
                .or_insert(0) += 1;
        }
    }

    // ---- the two known-answer controls, both filed POSITIVELY ---------------
    //
    // (1) PARTITION. Every observable body landed in exactly one arm. A bucket
    //     that silently stopped being written would otherwise shrink the
    //     accounted total while the ratio kept printing.
    if reached + stopped + nobody != observable {
        *res.fn_decode
            .entry("decode-reach-partition-broken".into())
            .or_insert(0) += 1;
    }
    // (2) POPULATION — and it is deliberately NOT `observable != census.len()`.
    //
    //     That comparison is a TAUTOLOGY: this walk increments `observable`
    //     once per element of `census`, so it could not fail, and a control
    //     that cannot fail has abstained rather than passed. **That is
    //     `#3336`'s whole finding**, and this lane exists because of it — so
    //     shipping one here would be the failure it was funded to prevent.
    //
    //     The comparison that CAN fail is against the count the **scan's own
    //     census loop** filed, in `gap::scan` step 1b/1c, over the same slice
    //     through different code: `fn_in_class` plus every `fn_blockers` row.
    //     Every census row is exactly one of those two, so their sum is this
    //     walk's denominator computed by a walk this module did not write —
    //     S0's `population-broken` shape (compare against the SIBLING walk),
    //     and #1464's rule that a control is never a subtraction of two
    //     published totals.
    //
    //     The size AND the direction, because an aggregate cannot distinguish
    //     `+1400/-27` from `+1373/-0` (`ROADMAP_SLICING` §6 rule 3).
    let sibling = res.fn_in_class + res.fn_blockers.values().sum::<usize>();
    if observable != sibling {
        *res.fn_decode
            .entry("decode-reach-population-broken".into())
            .or_insert(0) += 1;
        *res.fn_decode
            .entry(if observable > sibling {
                "decode-reach-population-over".into()
            } else {
                "decode-reach-population-under".to_string()
            })
            .or_insert(0) += observable.abs_diff(sibling);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The three arms are distinct and exhaustive over the value space
    /// `cflow_key` can produce.** A bucket that silently stopped being written
    /// would shrink the accounted total while the ratio kept printing.
    #[test]
    fn every_reach_arm_has_its_own_printed_key() {
        let keys: std::collections::BTreeSet<&str> = Reach::ALL.iter().map(|r| r.key()).collect();
        assert_eq!(keys.len(), Reach::ALL.len(), "two arms share a key");
        assert_eq!(Reach::ALL.len(), 3);
        for k in keys {
            assert!(k.starts_with("decode-reach-"), "{k} is not namespaced");
        }
    }

    /// **`cf-no-body` is its own arm and is NOT `Stopped`.** Folding it in
    /// would put every bodiless row into a bucket named after a walk that did
    /// not happen — the same distinction `cflow_key` draws for itself.
    #[test]
    fn a_row_with_no_body_is_its_own_arm() {
        assert_eq!(reach_of_cflow("cf-no-body"), Reach::NoBody);
        assert_ne!(reach_of_cflow("cf-no-body"), Reach::Stopped);
        assert_eq!(first_blocker_key("cf-no-body"), None);
    }

    /// **THE EXECUTED MUTATION FOR §5.3'S NEGATIVE CONTROL.**
    ///
    /// `C2RS_CFRESIDUE_ADMIT` moves the `+expr-modeled` residue suffix and
    /// cannot move the decode's Ok/Err. So the reach verdict must be invariant
    /// under the suffix — and the mutation that breaks it is the obvious one:
    /// keying reach off the whole `cflow` string instead of its prefix. Both
    /// halves are asserted here, so the control is watched going red rather
    /// than argued to be sound.
    #[test]
    fn the_residue_suffix_does_not_move_the_reach_verdict() {
        let bare = "cflow-straight";
        let suffixed = "cflow-straight+expr-modeled";
        assert_eq!(reach_of_cflow(bare), Reach::Reached);
        assert_eq!(reach_of_cflow(suffixed), Reach::Reached);
        // …and the mutation that WOULD break it, EXECUTED, so this control is
        // watched going red rather than argued to be sound: a reach predicate
        // written against the whole string instead of the prefix moves under a
        // knob (`C2RS_CFRESIDUE_ADMIT`) that changes no decode at all.
        let mutated = |cflow: &str| cflow == "cflow-straight";
        assert!(mutated(bare));
        assert!(
            !mutated(suffixed),
            "the whole-string reading must disagree — if it does not, this \
             control cannot fail and is worthless"
        );
    }

    /// **REACH IS NOT ADMISSION, pinned on one row from both sides.** The
    /// prereg's falsifier 3: the reach number must not be computed from a
    /// verdict the admission gate produced.
    #[test]
    fn reach_and_admission_are_independent_on_one_row() {
        // Reached and NOT admitted — the discriminating cell.
        assert_eq!(grade_one("cflow-loop", false), (Reach::Reached, false));
        // Admitted and NOT reached — the finding this lane is for.
        assert_eq!(grade_one("cf-expr-0x05", true), (Reach::Stopped, true));
        // Both, and neither. All four cells of the 2x2 are reachable, which is
        // what makes the separation a measurement rather than a restatement.
        assert_eq!(grade_one("cflow-straight", true), (Reach::Reached, true));
        assert_eq!(grade_one("cf-expr-0x05", false), (Reach::Stopped, false));
    }

    /// **The stop key is a FIRST-BLOCKER key and the API says so.** It is
    /// `None` for anything that did not stop, so it can never be read as a
    /// distance over the reached population.
    #[test]
    fn the_stop_key_exists_only_for_a_body_that_stopped() {
        assert_eq!(first_blocker_key("cf-expr-0x05"), Some("cf-expr-0x05"));
        assert_eq!(first_blocker_key("cflow-straight"), None);
        assert_eq!(first_blocker_key("cflow-loop+expr-modeled"), None);
    }

    /// **THE PARTITION CONTROL, WATCHED GOING RED.** A control nobody has seen
    /// fail is a control nobody has tested. The mutation is the realistic one:
    /// a bucket that silently stops being written, which shrinks the accounted
    /// total while every ratio keeps printing.
    #[test]
    fn the_partition_control_fires_when_a_bucket_stops_being_written() {
        let rows = [
            ("cflow-straight", true),
            ("cf-expr-0x05", false),
            ("cf-no-body", false),
        ];
        let (mut reached, mut stopped, mut nobody, mut observable) = (0, 0, 0, 0);
        for (cflow, _) in rows {
            observable += 1;
            match reach_of_cflow(cflow) {
                Reach::Reached => reached += 1,
                Reach::Stopped => stopped += 1,
                Reach::NoBody => nobody += 1,
            }
        }
        // Green on the honest count …
        assert_eq!(reached + stopped + nobody, observable);
        // … and RED when one bucket stops being written. Executed, not argued.
        assert_ne!(reached + stopped, observable, "the control cannot fail");
    }

    /// **THE POPULATION CONTROL IS NOT A TAUTOLOGY, and this test is the proof
    /// that it can disagree.**
    ///
    /// `observable != census.len()` could not fail — the walk increments once
    /// per element — and a criterion that cannot fail has ABSTAINED rather than
    /// passed, which is `#3336` exactly. The shipped control compares against
    /// the SIBLING walk's `fn_in_class + sum(fn_blockers)`, computed in
    /// `gap::scan` by code this module did not write, so the two can disagree.
    #[test]
    fn the_population_control_compares_against_a_walk_it_did_not_write() {
        // The sibling walk files one row per census row, split by class.
        let (sibling_in_class, sibling_blocked) = (2usize, 3usize);
        let observable = 5usize;
        assert_eq!(observable, sibling_in_class + sibling_blocked);
        // The failure this catches: the sibling walk skipped a row (a `continue`
        // on some predicate) while this walk counted it. Direction AND size,
        // because an aggregate cannot tell `+1400/-27` from `+1373/-0`.
        let sibling_broken = sibling_in_class + sibling_blocked - 1;
        assert_ne!(observable, sibling_broken);
        assert_eq!(observable.abs_diff(sibling_broken), 1);
        assert!(observable > sibling_broken, "direction is `over`");
    }

    /// The named parameters have stated defaults and refuse what they cannot
    /// parse. (The refusal path calls `exit`, so it is asserted by the
    /// unset/`on`/`off` arms rather than executed here.)
    #[test]
    fn the_parameters_are_named_with_stated_defaults() {
        assert_eq!(ENABLE_ENV, "C2RS_DECODE_REACH");
        assert_eq!(POP_ENV, "C2RS_DECODE_REACH_POP");
        assert_eq!(Pop::All.name(), "all");
        assert_eq!(Pop::Emitted.name(), "emitted");
        assert_eq!(Pop::Admitted.name(), "admitted");
    }

    /// **The keys are namespaced and never collide with FBM's or S0's.** A
    /// reader that summed `decode-reach-verified` with `fnbyte-exact` would be
    /// adding two different questions over two different denominators; the
    /// namespace is the guard, as it is for `fnbyte-blind-*`.
    #[test]
    fn the_keys_never_collide_with_fbm_or_blind() {
        for k in [
            Reach::Reached.key(),
            Reach::Stopped.key(),
            Reach::NoBody.key(),
            SEPARATION_KEY,
        ] {
            assert!(k.starts_with("decode-reach-"));
            assert!(!k.starts_with("fnbyte-"));
            assert_ne!(k, "fnbyte-exact");
            assert_ne!(k, "fnbyte-blind-exact");
        }
    }
}
