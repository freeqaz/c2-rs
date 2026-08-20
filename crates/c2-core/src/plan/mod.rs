//! **IR2 — the port's PREDICTED object plan, computed from IL without
//! emitting.** `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` §3.1/§3.2's `plan/`
//! slot, instrument half.
//!
//! # The tautology this module exists to avoid
//!
//! The lane that commissioned this was briefed with a control that reads:
//! *"the manifest must be identical on the TUs we already reproduce
//! byte-exactly"*. Taken literally that is **vacuous**. On a `match` TU the
//! port's obj **is** the reference's bytes (`ObjImage::diff`, timestamp
//! normalized), so `manifest(port_obj) == manifest(ref_obj)` follows from
//! `port_bytes == ref_bytes` for *any* pure function `manifest`. And on the
//! other 844 TUs the port emits no obj at all, so the port side would be
//! **undefined** and there would be no curve. The naive reading delivers
//! neither the control nor the curve.
//!
//! So the manifest has two independent producers:
//!
//! * [`c2_obj::ObjPlan::observe`] — ground truth, read off the **reference**
//!   obj. Defined wherever a reference obj decodes.
//! * [`predict`] — the port's plan, computed **from the IL bundle**, never by
//!   emitting. Defined on TUs the reader refuses, which is what makes a curve
//!   possible at all.
//!
//! Required-exact on the matched TUs is then a real constraint on [`predict`],
//! and the refused TUs get a graded verdict because [`predict`] is defined on
//! them.
//!
//! # The fence: this module may not reach the emitter, and may not ask
//! `IlBundle::functions()`
//!
//! `IlBundle::functions()` is the port's **admission gate**. It returns `None`
//! on 844 of the 870 graded TUs. A predictor that asked it for the emit set
//! would publish `known ≈ 30 of 870` and read `Unknown` on every one of the
//! 844 — which is not a curve, it is *the reader's refusal mass wearing new
//! keys*, and it is board **#3237** exactly: an instrument that returns 0
//! because it did not look is indistinguishable from one that returns 0 because
//! there was nothing to find.
//!
//! The ban is enforced by a **source-level test**
//! ([`tests::the_plan_module_does_not_reach_the_emitter_or_the_admission_gate`]),
//! in the same style as `coff::function`'s
//! `every_production_emitter_has_a_lib_rs_caller`. It is a source fence and not
//! a type fence, and that is stated plainly: a reviewer should read this
//! module's `use` list rather than trust the test's name.
//!
//! # NOT A GATE
//!
//! Nothing here may appear in an accept path, a refusal predicate, or
//! `scripts/gate.sh`. **A predicted plan that matches is NECESSARY but NOT
//! SUFFICIENT for a byte-exact obj** — a TU can be plan-exact and mismatch on
//! every instruction. The byte judge is unchanged and remains the sole judge.

use std::collections::{BTreeMap, BTreeSet};

use c2_il::IlBundle;

/// **A predicted component, or a NAMED reason the port did not look.**
///
/// `Unknown` is never `Known(empty)`, and the distinction is the entire
/// mitigation for #3237: an empty set that means *"c2 emits nothing here"* and
/// an empty set that means *"this stage has no answer"* are different facts,
/// and folding them is absence-read-as-success. The reason string is the
/// **census key** the component publishes, and it names the stage that owes the
/// work — the claims-ledger shape of the architecture proposal §3.3.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Predicted<T> {
    Known(T),
    Unknown(&'static str),
}

impl<T> Predicted<T> {
    pub fn known(&self) -> Option<&T> {
        match self {
            Predicted::Known(t) => Some(t),
            Predicted::Unknown(_) => None,
        }
    }
    /// The census key when this component did not look, `None` when it did.
    pub fn reason(&self) -> Option<&'static str> {
        match self {
            Predicted::Known(_) => None,
            Predicted::Unknown(r) => Some(r),
        }
    }
}

/// The argv-derived facts an object plan depends on that the IL bundle does not
/// record.
///
/// `/Gy` (implied by `/O1` and `/O2`) is the one that changes the section table
/// wholesale: it puts every emitted function in its own COMDAT `.text`. The
/// scan already computes it with
/// `PortC2::flags_imply_function_level_linking`, and it is passed in rather
/// than re-derived here so there is one decision and not two.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanInputs {
    pub function_level_linking: bool,
}

/// The port's predicted [`c2_obj::ObjPlan`], component by component.
///
/// Every component is a [`Predicted`], so a component the port cannot model
/// publishes a named `Unknown` rather than an empty `Known`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredictedPlan {
    /// **Which functions c2 emits**, as a set. The order-free claim.
    pub emit_set_members: Predicted<BTreeSet<String>>,
    /// The order the port would lay the emitted COMDATs out in.
    ///
    /// **`Unknown` on every TU, and that is a MEASURED result rather than an
    /// omission.** The only ordered `.gl` reader available is
    /// [`c2_il::mangled_names`], and `.gl` record order **is not COMDAT section
    /// order**: over the 870-TU workload it agreed on **18 of 854** TUs where
    /// both sides answered, and it **differed on 12 of the 26 TUs the port
    /// already reproduces byte-exactly** — the lane's own named control, which
    /// is what caught it. The extractor is not at fault (it agrees with
    /// `ObjImage::text_comdat_functions` by an integration test on real objs),
    /// so it is the *predictor* that is refuted, and under this lane's prereg a
    /// component whose control is red **ships as `Unknown`, never as
    /// `Differs`**.
    ///
    /// The refuted rule is still MEASURED and published — as
    /// `gap-metric plan-emitset-glorder-agrees`, a characterization number and
    /// not a port curve — because it is the price of board **#259**'s
    /// `coff::order::plan_text_order`: whatever builds this component has to
    /// beat 18 of 854, and that figure did not exist before.
    pub emit_set_order: Predicted<Vec<String>>,
    /// **The `.gl`-record-order hypothesis, kept as a CHARACTERIZATION value.**
    ///
    /// Not a component and never graded as one: this is the refuted rule above,
    /// retained so its agreement rate stays measurable at zero cost. A consumer
    /// that treated it as a prediction would be re-publishing a rule the named
    /// control has already killed.
    pub gl_record_order: Predicted<Vec<String>>,
    /// The ordered section-name sequence. Not modelled yet.
    pub sections: Predicted<Vec<String>>,
    /// Weak externals — `(weak, default)`. Not modelled yet.
    pub weak: Predicted<Vec<(String, String)>>,
    /// Undefined externals, in order. Not modelled yet.
    pub undef: Predicted<Vec<String>>,
}

/// **The emit bit.** `docs/whitebox/C2_MAP.md` §3E: the per-function flag word
/// lives at symbol offset `0x4c`, and c2's work-queue walk selects on
/// `test dl, 0x20` at `0x10b7f16e` (the load is `mov edx,[eax+0x4c]` at
/// `0x10b7f16b`, inside `0x10b7f022`).
///
/// [`c2_il::func::gl_function_attrs`] returns the **low byte** of that same
/// `+0x4c` word for every named `.gl` function record — §3E's own decode of a
/// tag-`0x0e` record shows `varU -> +0x4c = 0x1068` sitting exactly where that
/// reader takes its attribute byte, three fields past the framed `.ex` offset.
/// Bit 6 (`FN_FLAG_INLINABLE`, `0x40`) is the only bit that reader's existing
/// consumer looks at; `0x20` lives in the same low byte and is already decoded.
///
/// **This constant is disassembly-derived and carries a
/// `docs/whitebox/DISCLOSURE.md` row** (CLAUDE.md, binding).
///
/// **It is a SEED and not the emitted set.** §3E's cascade measurement is
/// explicit: on a bundle with a real call graph, clearing `0x20` on 17 of 20
/// functions changed nothing — each function falls only once its **caller** has
/// also been cleared. The emitted set is the `0x20`-seeded set **closed under
/// "referenced by an already-emitted function"**, and §3E's own practical
/// warning is that a port using the seed alone *"will over-delete on real
/// TUs"*. So this component is deliberately split: the seed is published now
/// and the closure is a lane of its own, with the gap between them measured
/// rather than argued.
pub const FN_FLAG_EMIT_SEED: u8 = 0x20;

/// Predict the object plan from the IL bundle, **without emitting**.
///
/// Every component that is not modelled returns `Unknown` with the census key
/// naming the stage that owes it. Nothing here reads a body, calls the
/// emitter, or asks the admission gate.
pub fn predict(bundle: &IlBundle, _inputs: &PlanInputs) -> PredictedPlan {
    let gl = bundle.get("gl");

    // ---- the emit set, via the `0x20` seed --------------------------------
    //
    // `gl_function_attrs` is whole-file fail-closed: one unrecognized
    // `SRCPOS`/`SIZE` encoding anywhere refuses the map, and "no information"
    // is required to be the status quo rather than a permission. That refusal
    // is this component's `Unknown` and its rate is P1's ceiling — measured,
    // never assumed.
    let attrs: Option<BTreeMap<String, u8>> = gl.and_then(c2_il::func::gl_function_attrs);
    let (members, order) = match (&attrs, gl) {
        (None, None) => (
            Predicted::Unknown("plan-no-gl"),
            Predicted::Unknown("plan-no-gl"),
        ),
        (None, Some(_)) => (
            Predicted::Unknown("gl-attrs-refused"),
            Predicted::Unknown("gl-attrs-refused"),
        ),
        (Some(a), Some(gl)) => {
            let seed: BTreeSet<String> = a
                .iter()
                .filter(|(_, &v)| v & FN_FLAG_EMIT_SEED != 0)
                .map(|(n, _)| n.clone())
                .collect();
            // **The ONLY ordered reader available.** `gl_body_record_names` and
            // `gl_gate_record_names` return a `BTreeSet` and have therefore
            // already discarded order; `mangled_names` walks the `.gl` symbol
            // runs in file order. Whether that order relates to COMDAT section
            // order at all is a MEASUREMENT this lane owes and does not assume
            // — if it does not, `emit_set_order` starts near zero and #259's
            // `coff::order::plan_text_order` is the whole component.
            let mut ordered: Vec<String> = Vec::new();
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for n in c2_il::mangled_names(gl) {
                // A name that occurs twice among the `.gl` symbol runs is
                // emitted once; the FIRST occurrence fixes its position.
                // Deduplicated here rather than left to the grader, because a
                // repeated name would make the ORDER component differ for a
                // reason that has nothing to do with order.
                if seed.contains(&n) && seen.insert(n.clone()) {
                    ordered.push(n);
                }
            }
            (Predicted::Known(seed), Predicted::Known(ordered))
        }
        (Some(_), None) => unreachable!("attrs are derived from gl"),
    };

    PredictedPlan {
        emit_set_members: members,
        // REFUTED BY THE NAMED CONTROL — see the field's doc. The `.gl`-order
        // hypothesis survives beside it as a characterization value, so the
        // number that prices #259's `plan_text_order` stays measurable.
        emit_set_order: Predicted::Unknown("plan-order-unmodelled"),
        gl_record_order: order,
        // Nothing below is modelled yet. Each names the stage that owes it, and
        // each is `Unknown` rather than an empty `Known` — see [`Predicted`].
        sections: Predicted::Unknown("plan-sections-unmodelled"),
        weak: Predicted::Unknown("plan-weak-unmodelled"),
        undef: Predicted::Unknown("plan-undef-unmodelled"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The fence, as a source-level test.** `crates/c2-core/src/plan/` may not
    /// name the emitter or the admission gate. If it did, the control on the
    /// matched TUs would be free — the predictor would be reaching the same
    /// code that produced the bytes the control compares against — and the
    /// curve on the refused TUs would collapse to the reader's refusal mass
    /// (#3237).
    ///
    /// Reads its own source with `include_str!`, so it needs no filesystem and
    /// no toolchain, and it cannot be defeated by moving the file.
    #[test]
    fn the_plan_module_does_not_reach_the_emitter_or_the_admission_gate() {
        let src = include_str!("mod.rs");
        // Everything before the `#[cfg(test)]` marker — the tests may name what
        // they forbid — **with comments stripped**.
        //
        // The comment strip is not tidiness and it is not a weakening. The
        // first version of this fence scanned the raw text and went red on its
        // own module doc, which explains the ban by naming `PortC2` and
        // `IlBundle::functions()`. **A fence that its own explanation trips is
        // a fence nobody can document**, and the pressure would have been to
        // delete the explanation rather than the dependency — which is the
        // wrong direction, because the explanation is what a reviewer reads.
        //
        // What is given up: a banned symbol hidden after a `//` on a code line
        // would not be seen. That is acceptable because a *use* of one is code,
        // never a trailing comment, and because — as the doc above says — this
        // is a **source fence and not a type fence**: the real check is a
        // reviewer reading this module's `use` list.
        let body: String = src
            .split("#[cfg(test)]")
            .next()
            .expect("the module body precedes its tests")
            .lines()
            .map(|l| match l.find("//") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<&str>>()
            .join("\n");
        // The mutation control: the fence must be able to FAIL. A test that
        // scans a string for absent needles passes trivially if the needle list
        // is wrong, so one needle that IS present is checked in the positive
        // direction first.
        assert!(
            body.contains("gl_function_attrs"),
            "the scanner is not reading this module's body at all"
        );
        for banned in [
            "PortC2",
            "crate::coff",
            "crate::codegen",
            "codegen::select",
            "coff::emit",
            ".functions(",
            "IlBundle::functions",
        ] {
            assert!(
                !body.contains(banned),
                "crates/c2-core/src/plan/ must not name `{banned}`: the predictor \
                 must be computable on the TUs the reader refuses, or the curve it \
                 publishes is the reader's refusal mass wearing new keys (#3237)"
            );
        }
    }

    #[test]
    fn unknown_is_never_known_empty() {
        let u: Predicted<BTreeSet<String>> = Predicted::Unknown("gl-attrs-refused");
        let k: Predicted<BTreeSet<String>> = Predicted::Known(BTreeSet::new());
        assert_ne!(u, k);
        assert_eq!(u.reason(), Some("gl-attrs-refused"));
        assert_eq!(k.reason(), None);
        assert!(u.known().is_none());
        assert!(k.known().is_some());
    }

    /// A bundle with no `.gl` publishes `plan-no-gl`, not an empty seed.
    #[test]
    fn a_bundle_with_no_gl_names_its_reason() {
        let b = IlBundle::new("_CL_test");
        let p = predict(&b, &PlanInputs { function_level_linking: true });
        assert_eq!(p.emit_set_members.reason(), Some("plan-no-gl"));
        // The ORDER component is `Unknown` unconditionally (its rule was refuted
        // by the named control), so the no-`.gl` reason surfaces on the
        // characterization value instead.
        assert_eq!(p.emit_set_order.reason(), Some("plan-order-unmodelled"));
        assert_eq!(p.gl_record_order.reason(), Some("plan-no-gl"));
    }

    /// The unmodelled components publish a NAMED reason apiece — never a
    /// shared one, because a shared reason cannot rank the stages that owe
    /// work.
    #[test]
    fn every_unmodelled_component_names_its_own_stage() {
        let b = IlBundle::new("_CL_test");
        let p = predict(&b, &PlanInputs { function_level_linking: true });
        let reasons = [
            p.sections.reason().unwrap(),
            p.weak.reason().unwrap(),
            p.undef.reason().unwrap(),
        ];
        let uniq: BTreeSet<&str> = reasons.iter().copied().collect();
        assert_eq!(uniq.len(), reasons.len(), "reasons must be distinct: {reasons:?}");
    }
}
