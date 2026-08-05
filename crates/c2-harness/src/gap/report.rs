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
            match_tu_differs: self.fn_byte_match_tu_differs(),
            value: (exact + whole_tu) as f64 / denominator as f64,
        })
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
