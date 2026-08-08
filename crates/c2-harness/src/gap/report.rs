//! [`GapReport`]'s class counters, the per-axis blocking histograms, the
//! emitted-function census read-out and **PROGRESS MASS**. Split out of
//! `gap.rs` unchanged; see [`super`] for the module docs.

use std::collections::BTreeMap;

use super::classify::merge_counts;
use super::{FnByteMatch, GapReport, ProgressMass, TuClass, TuResult};

impl GapReport {
    pub fn count(&self, class: TuClass) -> usize {
        self.results.iter().filter(|r| r.class == class).count()
    }

    /// Reasons for `class`, most frequent first, with TU counts.
    pub fn top_reasons(&self, class: TuClass) -> Vec<(String, usize)> {
        let mut map: BTreeMap<&str, usize> = BTreeMap::new();
        for r in self.results.iter().filter(|r| r.class == class) {
            *map.entry(r.reason.as_str()).or_insert(0) += 1;
        }
        let mut v: Vec<(String, usize)> =
            map.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **P2b headline**: (functions in class, functions total) across the scan.
    /// Unlike the TU classes this is monotone and fine-grained — it moves on
    /// every widening step, where TU-level `match` stays 0 until a whole TU
    /// happens to be in class.
    pub fn fn_coverage(&self) -> (usize, usize) {
        self.results
            .iter()
            .fold((0, 0), |(a, b), r| (a + r.fn_in_class, b + r.fn_total))
    }

    /// Blocking features across all scanned functions, most frequent first.
    /// **This histogram is the widening order** (docs/ROADMAP.md §G5/P2b).
    pub fn fn_blocker_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_blockers))
    }

    /// **The D6 frame measure**, aggregated: `"<calls-class>|<census key>"` counts
    /// over every scanned function, most frequent first.
    pub fn fn_frame_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_frames))
    }

    /// **The control-flow axis**, aggregated, most frequent first. Rows are either
    /// a bare class (`cflow-…` decoded, `cf-…` the decoder's own residue) or a
    /// `"<cflow class>|<census key>"` cross-tab; see [`TuResult::fn_cflow`].
    pub fn fn_cflow_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_cflow))
    }

    /// **The residue predicate's own denominator** — `(modeled, off_class)` over
    /// the bodies the port ACCEPTS.
    ///
    /// `CfResidue::Modeled` is a hand-written mirror of the port's class, and a
    /// `cflow-*+expr-modeled` count is only a counterfactual to the extent that
    /// mirror is current. These two numbers say how current it is, on the one
    /// population where the answer is known independently: an in-class body is
    /// one the per-function gate accepts, so a residue that called *every* one
    /// of them `Modeled` would be exactly as wide as the class. `off_class`
    /// is how many it is narrower by.
    ///
    /// **Not a gate and not an error.** The mirror is deliberately no wider than
    /// the accepting parser at the positions it checks (see `CfResidue`'s own
    /// doc: a looser residue would *over*-claim, which is the failure a
    /// counterfactual exists to avoid). A large `off_class` does not mean the
    /// axis is wrong. It means the counterfactual built on it is a proxy whose
    /// error is now measured instead of assumed, and it is published so that
    /// nobody quotes the counterfactual without it again.
    ///
    /// **Read it beside [`GapReport::cflow_residue_overclaim`], because the two
    /// errors point OPPOSITE ways.** `Modeled` neither contains nor is contained
    /// in the port's class: this method counts the in-class bodies it misses,
    /// that one counts the straight-line bodies it calls `Modeled` and the port
    /// still refuses. So the counterfactual is not a bound in either direction —
    /// it is an unvalidated proxy with a measured two-sided error, which is a
    /// weaker and truer thing to say about it than "lower bound".
    pub fn cflow_residue_control(&self) -> (usize, usize) {
        let h = self.fn_cflow_histogram();
        let mut modeled = 0;
        let mut off = 0;
        for (k, n) in &h {
            let Some(cls) = k.strip_suffix("|IN-CLASS") else { continue };
            if cls.ends_with("+expr-modeled") {
                modeled += *n;
            } else {
                off += *n;
            }
        }
        (modeled, off)
    }

    /// **The residue predicate's OTHER error** — straight-line bodies it calls
    /// `Modeled` that the port nonetheless refuses.
    ///
    /// Restricted to `cflow-straight` deliberately, and the restriction is the
    /// whole argument: for a straight-line body "blocked on control flow alone"
    /// is vacuous, so a body here that is `Modeled` **and** blocked is refused
    /// for a reason `Modeled` claimed was not there. It is the counterexample to
    /// reading [`GapReport::cflow_residue_control`] as "the residue is
    /// conservative" — it is not conservative, it is *different*.
    ///
    /// Why it matters for the rung this was written for: the standing price of
    /// the block-IR restructure is a count of `cflow-<branching>+expr-modeled`
    /// bodies, and that count was called a lower bound. A lower bound needs
    /// `Modeled ⊆ class`. This number is how badly that fails on the one class
    /// where it can be checked without lowering anything.
    pub fn cflow_residue_overclaim(&self) -> usize {
        self.fn_cflow_histogram()
            .iter()
            .filter(|(k, _)| k == "cflow-straight+expr-modeled|BLOCKED")
            .map(|(_, n)| *n)
            .sum()
    }

    /// **The control-flow counterfactual on the EMITTED column** —
    /// `(branchy, branchy_modeled)` over blocked *emitted* functions.
    ///
    /// `branchy` is how many emitted functions a block IR would have to serve at
    /// all; `branchy_modeled` is how many it would convert **by itself**, which
    /// is the number the `body-cflow-label` row has to be ranked from. The
    /// second is a lower bound in exactly the ratio
    /// [`GapReport::cflow_residue_control`] publishes.
    pub fn cflow_emitted_counterfactual(&self) -> (usize, usize) {
        (
            self.emit_total("emit-cflow-branchy"),
            self.emit_total("emit-cflow-branchy-modeled"),
        )
    }

    /// **The EH axis**, aggregated, most frequent first. Rows are either a bare
    /// class (`eh-…`) or an `"<eh class>|<census key>"` cross-tab; see
    /// [`TuResult::fn_eh`].
    pub fn fn_eh_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_eh))
    }

    /// How many scanned functions the statement-layer scanner decoded end to end,
    /// and how many it did not — `(decoded, undecoded)`.
    ///
    /// The ratio is the honest bound on everything the control-flow axis claims: a
    /// shape histogram over half the corpus is a shape histogram over half the
    /// corpus, and the other half's CFG is simply not known yet.
    pub fn cflow_decoded_totals(&self) -> (usize, usize) {
        let mut d = 0;
        let mut u = 0;
        for r in &self.results {
            for (k, n) in &r.fn_cflow {
                if k.contains('|') {
                    continue; // a cross-tab row, already counted in its bare class
                }
                if k.starts_with("cflow-") {
                    d += n;
                } else {
                    u += n;
                }
            }
        }
        (d, u)
    }

    /// The three frame classes' totals across the scan, in `calls-0`, `calls-1`,
    /// `calls-2plus` order.
    pub fn frame_class_totals(&self) -> [usize; 3] {
        let mut t = [0usize; 3];
        for r in &self.results {
            for (k, n) in &r.fn_frames {
                let i = match k.split('|').next() {
                    Some("calls-0") => 0,
                    Some("calls-1") => 1,
                    _ => 2,
                };
                t[i] += n;
            }
        }
        t
    }

    /// **The body-dispatch axis**, aggregated, most frequent first. See
    /// [`TuResult::fn_dispatch`] for the row shapes.
    pub fn fn_complete_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_complete))
    }

    pub fn fn_dispatch_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_dispatch))
    }

    /// **The member-call production first-blocker axis**, aggregated, most frequent
    /// first. See [`TuResult::fn_prod`].
    pub fn fn_prod_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_prod))
    }

    /// The **tag-coverage residue** of the production axis: bodies that entered a
    /// member-call production, declined non-committally, and reached no tagged
    /// bail — so their refusal is inside a shipping recognizer and is **not yet
    /// attributed to a site**.
    ///
    /// Reported as a number on every scan rather than inferred from the absence of
    /// rows. It is an upper bound on what the 37 tag sites in
    /// `body::shapes::mcall_{tail,chain,cmp}` have left to explain, and it reaches
    /// 0 when they are all placed.
    pub fn prod_untagged_residue(&self) -> usize {
        self.results
            .iter()
            .map(|r| {
                r.fn_prod
                    .get("prod-entered-untagged")
                    .copied()
                    .unwrap_or(0)
            })
            .sum()
    }

    /// How many functions each dispatch axis saw in total. Both must equal the
    /// census's own function total: every body takes exactly one arm and reaches
    /// exactly one production state, so a short count means a body slipped through
    /// untagged and the axis is under-reporting rather than the population being
    /// small.
    pub fn dispatch_axis_totals(&self) -> (usize, usize) {
        let bare = |m: &BTreeMap<String, usize>| -> usize {
            m.iter()
                .filter(|(k, _)| !k.contains('|'))
                .map(|(_, n)| *n)
                .sum()
        };
        self.results.iter().fold((0, 0), |(a, b), r| {
            (a + bare(&r.fn_dispatch), b + bare(&r.fn_prod))
        })
    }

    /// **The census/gate disagreement**, aggregated: how many censused-in-class
    /// functions `PortC2` refuses, per refusal, most frequent first.
    ///
    /// Every entry is an error term on [`GapReport::fn_coverage`]'s numerator.
    /// The target is an empty list.
    pub fn fn_gate_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.fn_gate_refusals))
    }

    /// Total censused-in-class functions the port refuses across the scan.
    pub fn fn_gate_disagreement(&self) -> usize {
        self.results
            .iter()
            .map(|r| r.fn_gate_refusals.values().sum::<usize>())
            .sum()
    }

    /// **The `.gl` binding invariants**, aggregated (see [`TuResult::bind_checks`]).
    pub fn bind_check_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.bind_checks))
    }

    /// **The emitted-function census**, aggregated (see [`TuResult::emit`]).
    pub fn emit_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.emit))
    }

    /// One aggregated emitted-census row.
    pub fn emit_total(&self, key: &str) -> usize {
        self.results
            .iter()
            .map(|r| r.emit.get(key).copied().unwrap_or(0))
            .sum()
    }

    /// **The read-out**: (in class ∩ emitted, emitted). The ratio is what
    /// `docs/ROADMAP.md` §8.2 ranks the plan by, and it is a **floor** — every
    /// emitted symbol the binding could not claim is residue, never a numerator.
    pub fn emit_coverage(&self) -> (usize, usize) {
        (self.emit_total("emit-in-class"), self.emit_total("emit-emitted"))
    }

    /// The unbound residue, split: (compiler-generated with no IL body,
    /// unexplained). The second number is the one that has to shrink; the first
    /// is a population no binding could ever claim.
    pub fn emit_residue(&self) -> (usize, usize) {
        (
            self.emit_total("emit-residue-generated"),
            self.emit_total("emit-residue-unbound") + self.emit_total("emit-name-two-rows"),
        )
    }

    /// **The emitted-only widening order**: blocking features restricted to rows
    /// that bind to a symbol c2 emitted, most frequent first.
    pub fn emit_blocker_histogram(&self) -> Vec<(String, usize)> {
        merge_counts(self.results.iter().map(|r| &r.emit_blockers))
    }

    /// **Ground truth.** On a TU the port compiles byte-exact, c2's emitted set
    /// *is* the port's, which came from the gate's own per-record binding — so
    /// the emitted census must read `in-class == emitted` with an empty residue.
    /// Returns how many emitted symbols on `match` TUs the binding failed to
    /// bind to an in-class row. **Known answer: 0.**
    ///
    /// This is the only check on the binding that is not a self-invariant. The
    /// oracle cannot grade a correspondence in general — but on a byte-exact TU
    /// it has already graded the whole symbol table, so the answer is known and
    /// the binding can be held to it.
    pub fn emit_match_tu_residue(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .map(|r| {
                let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                e.saturating_sub(i)
            })
            .sum()
    }

    /// TUs ordered by **distance to matching** — how many of their functions are
    /// blocked — keeping only those at or below `max_blocked`, nearest first.
    ///
    /// `docs/ROADMAP.md` §8.2 makes TU match the payoff metric and this
    /// distribution its leading indicator; the emitted census is what says
    /// whether a given TU's remaining distance is real work or bookkeeping.
    /// `capture-fail` TUs are excluded: they have no census at all, so a
    /// distance of 0 there means "never measured", not "nearly done".
    pub fn near_match_tus(&self, max_blocked: usize) -> Vec<&TuResult> {
        let mut v: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| r.class != TuClass::CaptureFail && r.fn_total > 0)
            .filter(|r| r.fn_total - r.fn_in_class <= max_blocked)
            .collect();
        v.sort_by_key(|r| (r.fn_total - r.fn_in_class, r.src.clone()));
        v
    }

    /// The same distribution measured on the population the **goal** is written
    /// in: blocked *emitted* functions, not blocked IL bodies.
    ///
    /// [`Self::near_match_tus`] counts `.ex` bodies, and the workload carries
    /// 2,462,571 of those against 178,968 emitted functions (`ROADMAP.md` §8.1).
    /// The two distances are not the same number and not even the same order:
    /// `src/system/math/Rand2.cpp` is 8 blocked bodies but **2** blocked emitted
    /// functions, and `src/system/math/vec.cpp` is 565 blocked bodies with
    /// **zero** blocked emitted functions. Published side by side because
    /// neither one alone is "distance to a byte-exact TU" — see
    /// [`Self::emit_set_reachable_tus`] for the third constraint that binds
    /// both.
    pub fn near_match_tus_emitted(&self, max_blocked: usize) -> Vec<&TuResult> {
        let blocked = |r: &TuResult| {
            let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
            let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
            e.saturating_sub(i)
        };
        let mut v: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| {
                r.class != TuClass::CaptureFail
                    && r.emit.get("emit-emitted").copied().unwrap_or(0) > 0
            })
            .filter(|r| blocked(r) <= max_blocked)
            .collect();
        v.sort_by_key(|r| (blocked(r), r.src.clone()));
        v
    }

    /// TUs for which the port could emit the **right set of `.text` COMDATs at
    /// all**, however good its codegen becomes — a hard ceiling on TU match that
    /// no widening can lift.
    ///
    /// `PortC2::build` takes `il.functions()`, one entry per `.ex` function
    /// segment, and under `/Gy` pushes exactly one `.text` COMDAT per entry.
    /// **There is no emit-set model anywhere in the port** (`ROADMAP.md` §8.3
    /// Phase 7 is where one would go). So when a TU's `.ex` segment count
    /// differs from its reference obj's `.text` COMDAT-leader count, the port
    /// emits the wrong number of sections and the obj diverges regardless of
    /// what any function lowers to. `emit-emitted` is exactly that leader count
    /// and `fn_total` is exactly that segment count, so the predicate is a
    /// comparison of two numbers the scan already has.
    ///
    /// This is a **necessary** condition, not a sufficient one — the bodies
    /// still have to lower byte-exact. Its value is as a ceiling: on the dc3
    /// workload it holds for 25 of 871 graded TUs, which bounds TU match at
    /// 25/878 until Phase 7 exists, against a terminal target of 871.
    pub fn emit_set_reachable_tus(&self) -> Vec<&TuResult> {
        let mut v: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| r.class != TuClass::CaptureFail)
            .filter(|r| r.fn_total == r.emit.get("emit-emitted").copied().unwrap_or(0))
            .collect();
        v.sort_by_key(|r| (r.fn_total - r.fn_in_class, r.src.clone()));
        v
    }

    /// The invariant behind [`Self::emit_set_reachable_tus`], as a count that
    /// must be **zero**: a TU that the differential graded `match` and whose
    /// `.ex` segment count nevertheless disagrees with its obj's `.text`
    /// COMDAT-leader count.
    ///
    /// A byte-exact obj cannot have a different number of `.text` COMDATs than
    /// the port emitted, so a nonzero here means `fn_total` and `emit-emitted`
    /// are not counting the things this reading says they count, and the
    /// ceiling above is void. It is the control that makes the ceiling a
    /// measurement rather than an argument: on this workload the agreement rate
    /// is 25/871 = 2.9 %, so six matching TUs agreeing by accident is ~10⁻⁹.
    pub fn emit_set_violations(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .filter(|r| r.fn_total != r.emit.get("emit-emitted").copied().unwrap_or(0))
            .count()
    }

    /// [`Self::emit_set_violations`] against the **gate-anchored** segment count
    /// (`4F 1F`) instead of the census's (`4C 4F 11`) — see step 1g in
    /// [`scan_one`].
    ///
    /// Returns `(violations, matching TUs the gate count is KNOWN for)`. The
    /// second number is not decoration: this control can only go red on a TU
    /// where `functions()` returned a count, and reporting the violation total
    /// without the population it was taken over is the shape that lets a green
    /// control mean "nothing was checked".
    pub fn emit_set_violations_gate(&self) -> (usize, usize) {
        let m: Vec<&TuResult> = self
            .results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .filter(|r| r.emit.contains_key("emit-gate-segments-known"))
            .collect();
        let bad = m
            .iter()
            .filter(|r| {
                r.emit.get("emit-gate-segments").copied().unwrap_or(0)
                    != r.emit.get("emit-emitted").copied().unwrap_or(0)
            })
            .count();
        (bad, m.len())
    }

    /// The splitter disagreement as counts (step 1g): `(TUs the gate count is
    /// known for, unknown, agree, disagree, gate sees more, census sees more,
    /// gate-anchored ceiling, entering the ceiling, leaving it)`.
    #[allow(clippy::type_complexity)]
    pub fn splitter_disagreement(&self) -> (usize, usize, usize, usize, usize, usize, usize, usize, usize) {
        let t = |k: &str| self.emit_total(k);
        (
            t("emit-gate-segments-known"),
            t("emit-gate-segments-unknown"),
            t("emit-splitter-agree"),
            t("emit-splitter-disagree"),
            t("emit-splitter-gate-sees-more"),
            t("emit-splitter-census-sees-more"),
            t("emit-set-ceiling-gate"),
            t("emit-set-ceiling-gate-enter"),
            t("emit-set-ceiling-gate-leave"),
        )
    }

    /// The binding invariant that must be **zero**: a generated destructor bound to
    /// a callee that is not a destructor. Nonzero means the `.gl` reader is naming
    /// the wrong symbol in a way no obj comparison over this corpus could have
    /// shown, because these bodies rarely reach an emitter.
    ///
    /// The ambiguity counts are deliberately **not** in here. A token two records
    /// disagree about is dropped, so it is an over-refusal with a measurable cost,
    /// not a wrong binding; the workload's residual is 7, all of them one `.gl`
    /// record form this reader does not model (`$…$initializer$` local statics), and
    /// their measured cost is 0 functions.
    pub fn bind_violations(&self) -> usize {
        self.results
            .iter()
            .map(|r| {
                r.bind_checks
                    .iter()
                    .filter(|(k, _)| {
                        k.as_str() == "dtor-callee-other" || k.as_str() == "dtor-callee-none"
                    })
                    .map(|(_, n)| *n)
                    .sum::<usize>()
            })
            .sum()
    }

    /// **PROGRESS MASS (lane w-metric, `docs/PROGRESS_METRIC.md`) — a PROGRESS
    /// metric, NEVER a correctness signal.** The byte-exact differential is the
    /// sole judge of the port; this number exists because that judge is a
    /// per-TU conjunction and therefore moves only when a TU's *last* defect
    /// closes, which left a day that moved factor C by 55 TUs reading as "no
    /// progress".
    ///
    /// `P = mean(a, b, c, f)` over the graded workload, where
    ///
    /// * `a = |A| / graded` — emit-set reachable (gate-anchored),
    /// * `b = |B| / graded` — every emitted symbol binds,
    /// * `c = |C| / graded` — obj sections within the writer's vocabulary,
    /// * `f = emitted-in-class / emitted` — the emitted census ratio.
    ///
    /// # What this is measured OVER, and what it deliberately is not
    ///
    /// Every term is a *precondition* count (obj-side facts A/B/C, measured
    /// against the reference obj) or an *acceptance* count (the emitted
    /// census). **No term rewards emitted bytes and no term is a similarity
    /// score** — an objdiff-style fuzzy match was evaluated and rejected for
    /// this workload (`docs/PROGRESS_METRIC.md` §3): the port emits an object
    /// on 8 of 871 graded TUs, so output similarity is undefined on 99.1 % of
    /// the denominator, and where a partial-credit byte score *is* defined it
    /// rewards a wrong emit over an honest refusal — the exact inversion the
    /// correctness rule forbids.
    ///
    /// # The two structural guards
    ///
    /// * **A wrong emit always scores below the refusal it replaced.** A TU
    ///   graded `mismatch` contributes 0 to every numerator while staying in
    ///   every denominator, so turning a refusal into a wrong emit strictly
    ///   decreases P. A metric without this property would pay lanes to emit
    ///   *something*, and the port's honest `NotImplemented` boundary is the
    ///   open gate, not a defect.
    /// * **`None` over an empty scan.** A scan that graded nothing has no
    ///   progress to report; returning `Some(1.0)` (objdiff's own
    ///   `calc_fuzzy_match_percent` returns 100.0 over zero code bytes) is the
    ///   absence-reads-as-success shape this project has recorded fifteen
    ///   times. `None` here, `NO-RESULT` in the print, no `gap-metric` key.
    ///
    /// # What it can still get wrong (documented, not fixed)
    ///
    /// `f`'s numerator is the census's in-class verdict — a parse-time claim
    /// that the differential has not graded for any never-emitted whole TU
    /// (STATUS trap 2). A widening that is wrong in a way no standing
    /// instrument grades (board #232's shape) raises `f` exactly as it raises
    /// the census. That is why this is a progress metric and not a gate, and
    /// why it must never appear in `scripts/gate.sh`.
    pub fn progress_mass(&self) -> Option<ProgressMass> {
        let graded = self.graded().count();
        // Denominators are taken over ALL graded TUs, mismatches included.
        let emitted_total: usize = self
            .graded()
            .map(|r| r.emit.get("emit-emitted").copied().unwrap_or(0))
            .sum();
        if graded == 0 || emitted_total == 0 {
            return None; // no progress is representable over nothing graded
        }
        // Numerators exclude every mismatch TU — the wrong-emit guard.
        let mut a = 0usize;
        let mut b = 0usize;
        let mut c = 0usize;
        let mut in_class = 0usize;
        let mut zeroed = 0usize;
        for r in self.graded() {
            if r.class == TuClass::Mismatch {
                zeroed += 1;
                continue;
            }
            let f = Self::factors(r);
            a += usize::from(f[0]);
            b += usize::from(f[1]);
            c += usize::from(f[2]);
            in_class += r.emit.get("emit-in-class").copied().unwrap_or(0);
        }
        let g = graded as f64;
        let value = (a as f64 / g
            + b as f64 / g
            + c as f64 / g
            + in_class as f64 / emitted_total as f64)
            / 4.0;
        Some(ProgressMass {
            graded,
            a,
            b,
            c,
            emitted_in_class: in_class,
            emitted_total,
            mismatch_zeroed: zeroed,
            value,
        })
    }

    /// **FUNCTION BYTE MATCH (lane w-fuzzy, `docs/FUNCTION_BYTE_MATCH.md`) — the
    /// byte-exact differential at function granularity.** A PROGRESS
    /// instrument, never a gate: the whole-obj compare against real `c2` under
    /// wibo remains the sole judge (`CLAUDE.md`).
    ///
    /// `FBM = fnbyte-exact / fnbyte-denominator` over the graded workload, where
    /// the denominator is every `.text` COMDAT leader in c2's own objs and the
    /// numerator is those the port lowers **byte-identically**.
    ///
    /// # The anti-gaming property
    ///
    /// *The denominator is a function of `c2`'s output alone; the numerator is
    /// the judge's own predicate.* There is no partial credit: a wrong body
    /// scores 0, exactly what a refusal scores, so the metric can never pay a
    /// lane to emit *something* — the inversion `docs/PROGRESS_METRIC.md` §2.2
    /// disqualified objdiff-style similarity for. Refusing more does not shrink
    /// the denominator, because the denominator is counted off the reference.
    ///
    /// # Absence
    ///
    /// `None` when nothing was graded — never `Some(1.0)`, which is what
    /// objdiff's `calc_fuzzy_match_percent` returns over zero code bytes
    /// (`objdiff-core/src/bindings/report.rs:249-250`) and is the shape this
    /// project has recorded sixteen times. A TU whose obj does not decode
    /// contributes to neither numerator nor denominator and is counted in
    /// `obj_unreadable`.
    ///
    /// # It is a FLOOR
    ///
    /// `partial` counts functions the port selected but whose body the COFF
    /// emitter finishes; the harness must not reconstruct them (board #322).
    /// FBM therefore under-reports the port and never over-reports it.
    pub fn fn_byte_match(&self) -> Option<FnByteMatch> {
        let t = |k: &str| self.emit_total(k);
        let denominator = t("fnbyte-denominator");
        if denominator == 0 {
            return None;
        }
        let exact = t("fnbyte-exact");
        // **The whole-TU override.** `super::fnbytes` grades through
        // `codegen::select_function`, the port's PER-FUNCTION route. `PortC2` has
        // one route that is not per-function — the whole-TU `??__E`
        // dynamic-initializer recognizer — and on the two TUs it compiles, the
        // per-function route reports `refused` for a body the differential has
        // already certified byte-exact.
        //
        // Discovered by the known-answer control below on its first corpus run:
        // it read 2 where it must read 0. The fix is not to relax the control.
        // On a TU the differential graded `match`, EVERY emitted function is
        // byte-identical to c2's — that is what a whole-obj byte compare means —
        // so the judge's own verdict supersedes the per-function route wherever
        // the two are both defined, and the credit is taken at the hardest
        // possible bar: the TU must already match.
        let whole_tu: usize = self
            .results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .map(|r| {
                let g = |k: &str| r.emit.get(k).copied().unwrap_or(0);
                g("fnbyte-denominator").saturating_sub(g("fnbyte-exact"))
            })
            .sum();
        Some(FnByteMatch {
            denominator,
            exact,
            whole_tu,
            differs: t("fnbyte-differs"),
            partial: t("fnbyte-partial"),
            refused: t("fnbyte-refused"),
            unbound: t("fnbyte-unbound"),
            nobytes: t("fnbyte-nobytes"),
            obj_unreadable: t("fnbyte-obj-unreadable"),
            partition_broken: t("fnbyte-partition-broken"),
            differ_words: (
                t("fnbyte-differs-port-words"),
                t("fnbyte-differs-ref-words"),
                t("fnbyte-differs-equal-words"),
            ),
            census_disagree: t("fnbyte-census-disagree"),
            exact_relocated: t("fnbyte-exact-relocated"),
            reloc_differs: t("fnbyte-reloc-differs"),
            reloc_unknown: t("fnbyte-reloc-unknown"),
            reloc_graded: t("fnbyte-reloc-graded"),
            exact_bytes: t("fnbyte-exact-bytes"),
            reloc_partition_broken: t("fnbyte-reloc-partition-broken"),
            match_tu_differs: self.fn_byte_match_tu_differs(),
            match_tu_reloc_differs: self.fn_byte_match_tu_reloc_differs(),
            value: (exact + whole_tu) as f64 / denominator as f64,
        })
    }

    /// **THE FIVE-ALARM** (lane `w-relo`). On a TU the differential graded
    /// `match`, the whole obj is byte-identical to c2's — so every relocation
    /// record in it is c2's own record, target and type and offset.
    ///
    /// **Known answer: 0.** A positive count is not a bucket entry and not a
    /// work item: it says the port's relocation plan
    /// (`c2_core::comdat::text_reloc_plan`, which the COFF writer now calls) and
    /// the obj the writer produced disagree about a function the oracle has
    /// already certified. That is one implementation contradicting itself, and
    /// it gets surfaced rather than counted.
    ///
    /// Same discipline as [`Self::fn_byte_match_tu_differs`], one field along.
    pub fn fn_byte_match_tu_reloc_differs(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .map(|r| r.emit.get("fnbyte-reloc-differs").copied().unwrap_or(0))
            .sum()
    }

    /// **Every function whose bytes are c2's and whose relocations are not**,
    /// by name, record index and the two targets — the witness list behind
    /// `fnbyte-reloc-differs`.
    ///
    /// The same argument `fn_byte_differ_witnesses` carries: a count cannot be
    /// acted on, and board #232/#259/#263/#276 were each closed from a named
    /// reproducer. Here the reproducer needs one thing more than a word index —
    /// **the two symbol names** — because the disagreement is invisible in the
    /// instruction bytes by construction.
    pub fn fn_byte_reloc_witnesses(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, _)| Some(k.strip_prefix("fnbyte-reloc-differs-fn|")?.to_string()))
            .collect();
        v.sort();
        v
    }

    /// **The relocation-disagreement FAMILIES**, most frequent first:
    /// `(shape|kind|where→where|relation, count)`.
    ///
    /// A list of mangled name pairs is not a finding. This is the axis that
    /// makes one: *what the two targets are to this TU* (`local`, `extern`,
    /// `comdat-only`) and *how they are related* — `chain1` meaning the
    /// reference names what the port's own callee calls, which is GRID-S `s12`'s
    /// class stated as a rule.
    pub fn fn_byte_reloc_families(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("fnbyte-reloc-fam|")?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **Why the family walk could not answer**, by blocking production —
    /// `(FnVerdict::key(), count)`, most frequent first.
    ///
    /// A `blocked` family row says *the port's own target is a body the parser
    /// refused, so whether it calls what c2 named is not answerable here.* This
    /// is the price of that, in the units a widening rung is written in, and it
    /// is the work list under the largest relocation family.
    pub fn fn_byte_reloc_blocked(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("fnbyte-reloc-blocked|")?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// [`Self::fn_byte_reloc_witnesses`] collapsed to signatures:
    /// `(shape|kind|counts|index|targets, distinct functions, one example)`.
    ///
    /// Same reason `fn_byte_differ_signatures` exists — 1,950 mangled STL names
    /// reporting one defect are **one** finding, and a list that prints them one
    /// per line hides that behind its own length.
    pub fn fn_byte_reloc_signatures(&self) -> Vec<(String, usize, String)> {
        let mut by_sig: std::collections::BTreeMap<String, (usize, String)> = Default::default();
        for w in self.fn_byte_reloc_witnesses() {
            // `shape|kind|counts|@index|targets|symbol` — the symbol is last and
            // mangled names contain no `|`, so the split is by count.
            let mut it = w.splitn(6, '|');
            let f: Vec<&str> = (&mut it).take(5).collect();
            let Some(name) = it.next() else { continue };
            if f.len() < 5 {
                continue;
            }
            let e = by_sig.entry(f.join("|")).or_insert((0, name.to_string()));
            e.0 += 1;
        }
        let mut v: Vec<(String, usize, String)> =
            by_sig.into_iter().map(|(k, (n, ex))| (k, n, ex)).collect();
        v.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
        v
    }

    /// **The known answer FBM is held to.** On a TU the differential graded
    /// `match`, the whole obj is byte-identical to c2's — so every one of its
    /// emitted functions was lowered by this port, correctly. The per-function
    /// route may legitimately have *no* body for such a function (the whole-TU
    /// `??__E` recognizer, `partial` shapes), but it may never produce a
    /// **different** one: `select_function` and the COFF emitter would then
    /// disagree about a body the oracle has already certified.
    ///
    /// **Known answer: 0.** A nonzero here says FBM's numerator is not counting
    /// what its documentation says it counts, and the whole-TU credit above —
    /// which is taken on the oracle's verdict — would be papering over a real
    /// disagreement inside the port.
    ///
    /// Same discipline as [`Self::emit_match_tu_residue`]: the oracle cannot
    /// grade a per-function correspondence in general, but on a byte-exact TU it
    /// has already graded the whole obj, so the answer is known and the
    /// instrument can be held to it.
    pub fn fn_byte_match_tu_differs(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.class == TuClass::Match)
            .map(|r| r.emit.get("fnbyte-differs").copied().unwrap_or(0))
            .sum()
    }

    /// The `fnbyte-partial|…` rows, most frequent first — **the size of FBM's
    /// own under-report, by [`Selected`](c2_core::codegen::Selected) variant**,
    /// which is also the work list for board #322.
    pub fn fn_byte_partial_histogram(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("fnbyte-partial|")?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **The per-shape census** (board #322): `(shape, verdict, count)` for
    /// every `Selected` variant the port produced, crossed with what the judge
    /// then said about its bytes.
    ///
    /// `fn_byte_partial_histogram` above answers *"which shapes is the alarm
    /// blind to"*. This answers the question that replaces it once the blind
    /// spot closes — ***which shapes is it now grading, and with what
    /// verdict*** — and it is the only place a per-shape `differs` shows up as
    /// a row rather than as a share of one corpus total. A shape that quietly
    /// stopped being graded would lose its `exact` row here while
    /// `fnbyte-differs` went on reading 0.
    pub fn fn_byte_shape_census(&self) -> Vec<(String, String, usize)> {
        let mut v: Vec<(String, String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| {
                let rest = k.strip_prefix("fnbyte-shape|")?;
                let (shape, verdict) = rest.split_once('|')?;
                Some((
                    shape.to_string(),
                    verdict.strip_prefix("fnbyte-").unwrap_or(verdict).to_string(),
                    n,
                ))
            })
            .collect();
        v.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| (&a.0, &a.1).cmp(&(&b.0, &b.1))));
        v
    }

    /// **Board #980 — where the dead-temporary chain STOPS**, by production.
    ///
    /// One row per blocking feature of a callee that a recognized no-effect body
    /// names and that does not itself reduce to nothing. It is the widening
    /// order for this rule and nothing else: a rung that closes the top row here
    /// converts that many more callers and no others.
    pub fn fn_byte_noeffect_stops(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("fnbyte-noeffect-stop|")?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **Board #980's residue, by production** — for every `fnbyte-differs`
    /// whose whole reference body is one `blr`, the callee's own blocking
    /// feature (`fnbyte-blr-stop|…`) and, when that callee is itself a
    /// recognized dead-temporary body, its callee's (`fnbyte-blr-stop2|…`).
    ///
    /// The prefix is the argument: `prefix` is `"fnbyte-blr-stop|"` or
    /// `"fnbyte-blr-stop2|"`, and one function serves both rather than two that
    /// can drift.
    ///
    /// **It is the GENERIC pipe-keyed histogram** and is reached under other
    /// prefixes too — board #1053's `"fnbyte-nothing-key|"` is one. Named for its
    /// first caller and kept that way so the rungs and board rows that cite it
    /// still resolve; reach for it rather than writing a second `strip_prefix`
    /// walk, which is exactly the duplication `w-relo`'s merge produced with no
    /// conflict marker to warn anyone.
    pub fn fn_byte_blr_stops(&self, prefix: &str) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix(prefix)?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// **Every differing function, by name and by word** — the witness list
    /// behind `fnbyte-differs`.
    ///
    /// Known answer: empty. A count cannot be acted on; each row here names the
    /// shape, the word counts, the first disagreeing word (port and reference
    /// hex) and the mangled symbol, which is what a lane needs to reproduce it.
    /// Board #232/#259/#263/#276 were each closed from a named reproducer.
    pub fn fn_byte_differ_witnesses(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, _)| Some(k.strip_prefix("fnbyte-differs-fn|")?.to_string()))
            .collect();
        v.sort();
        v
    }

    /// **Every function whose CALL TARGETS disagree**, port list against real
    /// c2's `REL24` targets (lane `w-drop3`, board **#985**).
    ///
    /// Restricted to the `exact` bucket and bounded per TU by
    /// `MAX_CALLTARGET_WITNESSES` — unlike
    /// [`Self::fn_byte_differ_witnesses`], whose population has known answer 0,
    /// this one has a known answer in the thousands (every mechanism-I body
    /// disagrees by count), so an unbounded list would be a transcript rather
    /// than evidence. The **counts** beside it are unbounded.
    pub fn fn_byte_call_target_witnesses(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, _)| {
                Some(k.strip_prefix("fnbyte-calltarget-witness|")?.to_string())
            })
            .collect();
        v.sort();
        v
    }

    /// [`Self::fn_byte_differ_witnesses`] **collapsed to signatures**:
    /// `(shape|words|first-disagreeing-word, distinct functions, one example
    /// symbol)`, most frequent first.
    ///
    /// The witness list is the evidence; this is the part a reader can act on.
    /// 1,950 mangled STL names all failing at word 0 with the same two words is
    /// **one** finding, and a list that prints them one per line hides that
    /// behind its own length — trap 5's shape, where the reader's own summary
    /// step is what loses the information.
    pub fn fn_byte_differ_signatures(&self) -> Vec<(String, usize, String)> {
        let mut by_sig: std::collections::BTreeMap<String, (usize, String)> = Default::default();
        for w in self.fn_byte_differ_witnesses() {
            // `shape|words|first-word|symbol` — the symbol is the last field and
            // mangled names contain `|` nowhere, but they DO contain `@` and
            // `$`, so the split is from the right and by count, not by search.
            let mut it = w.splitn(4, '|');
            let (a, b, c, name) = (it.next(), it.next(), it.next(), it.next());
            let (Some(a), Some(b), Some(c), Some(name)) = (a, b, c, name) else {
                continue;
            };
            let e = by_sig
                .entry(format!("{a}|{b}|{c}"))
                .or_insert((0, name.to_string()));
            e.0 += 1;
        }
        let mut v: Vec<(String, usize, String)> = by_sig
            .into_iter()
            .map(|(k, (n, ex))| (k, n, ex))
            .collect();
        v.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
        v
    }

    /// **THE DIFF-SIGNATURE CLUSTER CENSUS** (board #976, [`super::fndiff`]) —
    /// the `fnbyte-differs` population grouped by the *structure* of its
    /// disagreement rather than by its first wrong word.
    ///
    /// Rows whose key carries `prefix`, largest first. The prefix is stripped so
    /// the caller renders a cluster key, not a counter name.
    fn fndiff_rows(&self, prefix: &str) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix(prefix)?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// The coarse clusters: `shape|length-relation|edit-shape|field-classes`.
    pub fn fndiff_clusters(&self) -> Vec<(String, usize)> {
        self.fndiff_rows("fndiff-csig|")
    }

    /// Per-substituted-word field classes — `reg`, `imm`, `disp`, `opcode`,
    /// `branch-target`, `undecoded`, … Counted in WORDS, not in functions, so it
    /// does not sum to the cluster table.
    pub fn fndiff_classes(&self) -> Vec<(String, usize)> {
        self.fndiff_rows("fndiff-class|")
    }

    /// Where the first disagreement is, bucketed.
    pub fn fndiff_first_buckets(&self) -> Vec<(String, usize)> {
        self.fndiff_rows("fndiff-first|")
    }

    /// **Per-TU FBM**, nearest-to-done first: `(src, exact, denominator)` over
    /// every graded TU that carries at least one emitted function.
    ///
    /// This is the answer to *"we are 8/878 exact — how close is the other
    /// 870?"* stated per TU rather than as one corpus number. `capture-fail`
    /// TUs are excluded exactly as [`Self::near_match_tus`] excludes them: a
    /// ratio over zero emitted functions is "never measured", not "done".
    pub fn fn_byte_by_tu(&self) -> Vec<(&str, usize, usize)> {
        let mut v: Vec<(&str, usize, usize)> = self
            .graded()
            .map(|r| {
                let d = r.emit.get("fnbyte-denominator").copied().unwrap_or(0);
                // Same whole-TU override as `fn_byte_match`: a byte-exact obj
                // means every function in it is byte-exact, so a `match` TU is
                // 100 % whatever the per-function route could reconstruct.
                let e = if r.class == TuClass::Match {
                    d
                } else {
                    r.emit.get("fnbyte-exact").copied().unwrap_or(0)
                };
                (r.src.as_str(), e, d)
            })
            .filter(|(_, _, d)| *d > 0)
            .collect();
        v.sort_by(|a, b| {
            let ra = a.1 as f64 / a.2 as f64;
            let rb = b.1 as f64 / b.2 as f64;
            rb.partial_cmp(&ra)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(b.0))
        });
        v
    }

    /// Replay soundness: (checked, diverged).
    pub fn replay_stats(&self) -> (usize, usize) {
        let checked = self.results.iter().filter(|r| r.replay_ok.is_some()).count();
        let bad = self
            .results
            .iter()
            .filter(|r| r.replay_ok == Some(false))
            .count();
        (checked, bad)
    }
}
