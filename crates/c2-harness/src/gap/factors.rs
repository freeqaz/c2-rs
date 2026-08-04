//! **The Phase 7 factor model** (`docs/ROADMAP.md` §10.19/§10.21, boards #160
//! and #179): the A/B/C/D/E predicates, the joints and known-answer control
//! taken over them, the section vocabulary and its greedy ladder, and the
//! machine-readable `GAP-METRICS` block. Split out of `gap.rs` unchanged; see
//! [`super`] for the module docs.

use super::{GapReport, TuClass, TuResult, PORT_WRITER_SECTIONS, WHOLE_TU_RECOGNIZERS};

impl GapReport {
    /// **The five Phase 7 factors for one TU** (`docs/ROADMAP.md` §10.19 and
    /// §10.21, boards #160 and #179), in `[A, B, C, D, E]` order:
    ///
    /// | | predicate | key |
    /// |---|---|---|
    /// | **A** | `.ex` segments == obj `.text` COMDATs, on the anchor the port consumes | `emit-set-ceiling-gate` |
    /// | **B** | every emitted symbol binds | `emit-set-ceiling-today` |
    /// | **C** | obj section set ⊆ [`PORT_WRITER_SECTIONS`] | `emit-sec-reachable` |
    /// | **D** | every emitted COMDAT is in the port's **per-function** codegen class | `emit-class-complete` |
    /// | **E** | a registered **whole-TU** recognizer accepts this bundle | `emit-whole-tu-any` |
    ///
    /// Every one reads a key some *other* code path wrote, so this function
    /// re-derives no rule — it is a join, and that is the whole point (§10.14).
    ///
    /// # What the factorization is a factorization OF, and where D went wrong
    ///
    /// §10.19's four predicates are four questions the port must answer yes to
    /// before its output can be the reference's bytes: **A** do the port and the
    /// reference agree on *what set of things is emitted*; **B** can the port
    /// *name* everything in that set; **C** can the writer *write the containers*
    /// the obj needs; **D** does the port have an *accepted route to the
    /// contents*.
    ///
    /// A/B/C are properties of the obj and the binding. **D is the odd one out**:
    /// it is not a property of the obj at all but of the port's acceptance
    /// machinery — `emit-class-complete` is the *per-function* census's verdict,
    /// i.e. "`PortC2`'s per-function path takes every COMDAT here". §10.19 was
    /// measured when `PortC2::build` had exactly **one** acceptance path, so
    /// "the port has a route to the contents" and "the per-function path accepts
    /// every COMDAT" were the same sentence. They are not any more:
    /// `PortC2::build` tries `IlBundle::dyninit_tu()` *before* `functions()`.
    ///
    /// So D was never the general form of question 4 — it was the only reading of
    /// it that existed. **E is the whole-TU reading**, and the general form is the
    /// disjunction [`Self::emit_path`]:
    ///
    /// > A byte-exact obj requires **A ∧ B ∧ C ∧ (D ∨ E)**.
    ///
    /// Measured (2026-08-04, 871 graded TUs): the conjunction `A∧B∧C∧D` is 6
    /// while the differential grades 8, so **D alone is not necessary** and the
    /// old known-answer control was right to print `D 2`. E alone is not
    /// necessary either — it is false on all six per-function matches. The
    /// *disjunction* is what is claimed necessary, and it is what the control is
    /// taken over.
    ///
    /// # This is a disjunct on D, not a widening of D
    ///
    /// D's definition is byte-for-byte what it was: `emit-emitted ==
    /// emit-in-class`, from the per-function census. Nothing in `c2-il`'s
    /// `census.rs` is touched, so the scan's `census/gate disagreement: 0` line
    /// still tracks the symmetry w-r1c declined to break — teaching the
    /// per-function census a whole-TU fact is what a widening would have meant,
    /// and it is not what happened. D's own violation count is still printed, as
    /// a number, so §10.19's refutation stays a visible finding rather than an
    /// absorbed one.
    ///
    /// E is also deliberately **not** "the port emitted it" and **not** the class
    /// field: either would be circular and would make the model unfalsifiable. E
    /// is a class-membership predicate of the same *kind* as D — evaluated
    /// without running the emitter — just at whole-TU granularity. The accepted
    /// consequence is that, exactly like D, **E is not sufficient**:
    /// `PortC2::build_dyninit` carries the `/GF` fence, which lives in `c2-core`
    /// and not in the recognizer, so an E-true TU can still fail to emit. That
    /// would show as an over-prediction in the printed set-identity line, which
    /// is where it belongs.
    ///
    /// **A is gate-anchored** (`4F 1F`, what `PortC2::build` consumes) rather
    /// than `LO`-anchored: §10.18 settled that the two splitters disagree on 634
    /// of 871 TUs and that the port's anchor is the one its emitter has to
    /// satisfy. [`Self::factor_a_lo`] is the other reading, published beside it.
    pub fn factors(r: &TuResult) -> [bool; 5] {
        let has = |k: &str| r.emit.contains_key(k);
        [
            has("emit-set-ceiling-gate"),
            has("emit-set-ceiling-today"),
            has("emit-sec-reachable"),
            has("emit-class-complete"),
            has("emit-whole-tu-any"),
        ]
    }

    /// **Question 4 in its general form: `D ∨ E`** — the port has an accepted
    /// route to this TU's contents, by *some* acceptance path.
    ///
    /// The term the model claims is necessary. Neither disjunct is necessary
    /// alone and both are measured not to be, which is the entire content of
    /// board #179: see [`Self::factors`].
    pub fn emit_path(f: &[bool; 5]) -> bool {
        f[3] || f[4]
    }

    /// Factor A on the **`LO`** anchor (`4C 4F 11`, the census's splitter) —
    /// the reading `emit_set_reachable_tus` filters on. Published beside the
    /// gate-anchored one because §10.18's whole finding is that they are two
    /// different numbers and only one is the port's.
    pub fn factor_a_lo(r: &TuResult) -> bool {
        r.fn_total == r.emit.get("emit-emitted").copied().unwrap_or(0)
    }

    /// The TUs the factorization is computed over: everything the harness
    /// graded, i.e. every TU that captured. `capture-fail` TUs have no obj and
    /// no census, so they are not "outside the factors" — they were never
    /// measured, and folding them in would make every factor look tighter.
    pub fn graded(&self) -> impl Iterator<Item = &TuResult> {
        self.results.iter().filter(|r| r.class != TuClass::CaptureFail)
    }

    /// `(|A|, |B|, |C|, |D|, |E|, |A_lo|, |B∧C|, |A∧B∧C|, |A∧B∧C∧D|,
    /// |A∧B∧C∧(D∨E)|)` over the graded TUs.
    ///
    /// `B∧C` is the plan's **near-term joint ceiling** — what a perfect emit-set
    /// model plus a perfect binding reaches while the writer's vocabulary is
    /// what it is (`PHASE7_PLAN.md` §1). It is a *joint*, measured per TU, and
    /// not a product of marginals: §8.6's standing rule, and the reason this
    /// function exists rather than a note telling readers to multiply.
    ///
    /// **`A∧B∧C∧D` is kept and reported** even though the model's joint is now
    /// `A∧B∧C∧(D∨E)`. §10.19's original conjunction is the thing board #179
    /// refutes; a refutation whose refuted quantity stops being measured is a
    /// claim nobody can re-check.
    pub fn factor_counts(&self) -> [usize; 10] {
        let mut c = [0usize; 10];
        for r in self.graded() {
            let f = Self::factors(r);
            for i in 0..5 {
                c[i] += usize::from(f[i]);
            }
            c[5] += usize::from(Self::factor_a_lo(r));
            c[6] += usize::from(f[1] && f[2]);
            let abc = f[0] && f[1] && f[2];
            c[7] += usize::from(abc);
            c[8] += usize::from(abc && f[3]);
            c[9] += usize::from(abc && Self::emit_path(&f));
        }
        c
    }

    /// **The model's joint, `A∧B∧C∧(D∨E)`**, by source path. The claim is that
    /// this set **is** the match set, so it is returned as a list of names rather
    /// than a count: a count could agree by coincidence, and two sets that differ
    /// by a swap would read as equal.
    pub fn factor_all_tus(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| {
                let f = Self::factors(r);
                f[0] && f[1] && f[2] && Self::emit_path(&f)
            })
            .map(|r| r.src.as_str())
            .collect()
    }

    /// §10.19's **original** conjunction `A∧B∧C∧D`, by source path — the set
    /// board #179 refutes. Kept beside [`Self::factor_all_tus`] so the
    /// refutation stays checkable rather than becoming folklore: the difference
    /// between the two lists is exactly the TUs the fifth term accounts for.
    pub fn factor_abcd_tus(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| {
                let f = Self::factors(r);
                f[0] && f[1] && f[2] && f[3]
            })
            .map(|r| r.src.as_str())
            .collect()
    }

    /// Per-recognizer marginals for [`WHOLE_TU_RECOGNIZERS`]: `(name, TUs it
    /// accepts)`, in registry order.
    ///
    /// Printed per entry rather than only as the union, because a registry entry
    /// that never fires and one that was never added are the same number in
    /// `|E|` and very different facts about the model.
    pub fn whole_tu_marginals(&self) -> Vec<(&'static str, usize)> {
        WHOLE_TU_RECOGNIZERS
            .iter()
            .map(|(name, _)| (*name, self.emit_total(&format!("emit-whole-tu|{name}"))))
            .collect()
    }

    /// **The known-answer control on the factorization**: how many byte-exact
    /// TUs fail each term, and how many `match` TUs there were to check.
    /// Returns `([A, B, C, D, E, D∨E] violations, matching TUs)`.
    ///
    /// # Which of these must be zero, and why that is not a relaxation
    ///
    /// **A, B, C and `D∨E` must be 0.** Those are the model's *necessary*
    /// conditions, which is the only thing that makes them a ceiling; nonzero
    /// anywhere means the term is not necessary and any bound drawn from it is
    /// void.
    ///
    /// **D and E individually must not be**, and it would be wrong to require it.
    /// Both are measured non-necessary on the 878-TU workload: D fails on the two
    /// `??__E` TUs (whole-TU emit path), E fails on all six per-function matches.
    /// They are the two readings of one question (see [`Self::factors`]), so
    /// their columns are **diagnostics**, printed with the label that says so.
    ///
    /// The distinction matters because moving a column from "must be 0" to
    /// "diagnostic" is exactly the move that a fitted control would make to go
    /// green. What makes it legitimate here is that the *replacement* column is
    /// strictly narrower than "anything the port emits": `D∨E` is D plus a
    /// **closed, named registry** ([`WHOLE_TU_RECOGNIZERS`]) of one entry, so an
    /// emit path nobody registered still turns it red. `E := decodes()` would
    /// have been the relaxation; this is not it.
    ///
    /// For **C** this is also the control on [`PORT_WRITER_SECTIONS`] itself: a
    /// matching obj is the port's own output, so a name missing from that list
    /// shows up here rather than in an argument about whether the list is
    /// complete.
    pub fn factor_control_on_match_tus(&self) -> ([usize; 6], usize) {
        let mut bad = [0usize; 6];
        let mut n = 0;
        for r in self.results.iter().filter(|r| r.class == TuClass::Match) {
            n += 1;
            let f = Self::factors(r);
            for (i, ok) in f.iter().enumerate() {
                bad[i] += usize::from(!ok);
            }
            bad[5] += usize::from(!Self::emit_path(&f));
        }
        (bad, n)
    }

    /// **The frontier**: TUs inside `A∧B∧C` that are not yet a `match` and that
    /// **no acceptance path the port has covers** — the emit set is reachable,
    /// every emitted symbol binds, the obj's sections are all writable, neither
    /// the per-function class (D) nor any registered whole-TU recognizer (E)
    /// takes the TU, and widening the accepted *function* class is the whole
    /// remaining distance.
    ///
    /// **Board #179 narrowed this from `¬D` to `¬(D∨E)`.** A TU some whole-TU
    /// recognizer already accepts but that is not a match is *not* on the
    /// codegen-breadth frontier: its blocker is that whole-TU emitter's own fence
    /// (for `dyninit`, the `/GF` fence in `c2-core`), which is different work
    /// from widening the function class. Leaving it in would have advertised
    /// per-function codegen as the route to a TU per-function codegen cannot
    /// reach.
    ///
    /// This is the one actionable list the factorization produces. Everything
    /// else it prints is a bound; these are TUs where no model, no section work
    /// and no binding repair is needed. Sorted by distance (emitted functions
    /// not in class), nearest first.
    ///
    /// **It is not a schedule** (`ROADMAP.md` §9.16.1): a TU one blocked
    /// function away can be one blocked function away from a construct nobody
    /// has modelled.
    pub fn factor_frontier(&self) -> Vec<(&TuResult, usize)> {
        let mut v: Vec<(&TuResult, usize)> = self
            .graded()
            .filter(|r| r.class != TuClass::Match)
            .filter(|r| {
                let f = Self::factors(r);
                f[0] && f[1] && f[2] && !Self::emit_path(&f)
            })
            .map(|r| {
                let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                (r, e.saturating_sub(i))
            })
            .collect();
        v.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.src.cmp(&b.0.src)));
        v
    }

    /// **The counterfactual frontier**: what [`Self::factor_frontier`] would
    /// count if factor **A** were true on every graded TU — i.e. if a perfect
    /// emit-set model existed. Same clauses as the frontier with `f[0]` dropped.
    ///
    /// Board **#213** quotes this beside `B∧C` and both halves of that row's
    /// arithmetic went stale together: it was published as `17 → 99` when
    /// `A∧B∧C` was 25 and `B∧C` was 107. It is computed here rather than
    /// subtracted by hand because *that hand-subtraction is exactly the defect
    /// this function exists to prevent* — `99 − 17 == 107 − 25` only while
    /// every `match`-or-`D∨E` TU inside `B∧C` also satisfies A, which is a
    /// contingent fact about the corpus and not an identity.
    pub fn factor_frontier_if_a(&self) -> usize {
        self.graded()
            .filter(|r| r.class != TuClass::Match)
            .filter(|r| {
                let f = Self::factors(r);
                f[1] && f[2] && !Self::emit_path(&f)
            })
            .count()
    }

    /// **The TUs on which board #213's two arithmetics disagree**, by name.
    ///
    /// #213 states the value of a perfect emit predicate twice — as
    /// `B∧C − A∧B∧C` and as `frontier-if-A − FRONTIER` — and published one
    /// number for both (`+82`), because when it was written the two coincided.
    /// They coincide exactly when **no** TU inside `B∧C` fails A while already
    /// having an accepted route (D or E), and that is a contingent property of
    /// the corpus. These are the TUs in the difference: reachable if the emit
    /// set were modelled, but *not* additions to the codegen frontier, because
    /// the port already accepts their contents.
    ///
    /// Returned by name rather than as a count, for the reason
    /// [`Self::factor_all_tus`] gives: a count can agree by coincidence.
    pub fn factor_projection_divergence(&self) -> Vec<&str> {
        self.graded()
            .filter(|r| r.class != TuClass::Match)
            .filter(|r| {
                let f = Self::factors(r);
                !f[0] && f[1] && f[2] && Self::emit_path(&f)
            })
            .map(|r| r.src.as_str())
            .collect()
    }

    /// **The stable machine-readable metric block**, one `key value` pair per
    /// entry, for `scripts/status.sh` and any other collector.
    ///
    /// # Why this exists
    ///
    /// Every figure here was already printed by a `gap` scan, in prose, and
    /// **three of them went stale twice in one day** (2026-08-04, lane
    /// `w-book4`): factor `C`, `A∧B∧C` and the `FRONTIER` live only in
    /// hand-written `STATUS.md`/`BOARD.md` paragraphs because the collector's
    /// five `sed` recipes cover only the six `TuClass` counters. `B∧C` was
    /// worse — it was published once, at `C = 114`, and then *silently
    /// invalidated by a dependency* when the writer's section vocabulary grew.
    /// A number a script cannot re-derive is a number that goes stale, and the
    /// project navigated by a stale one for two merges.
    ///
    /// # The two rules this block follows
    ///
    /// * **Keys are stable and values are bare integers or bare tokens**, so a
    ///   `sed`-based collector can take them without a parser. The keys are
    ///   part of the interface: renaming one silently returns `NO-RESULT`,
    ///   which is trap 5 (absence read as success) with the mask on.
    /// * **Derived quantities are derived HERE.** `emit-predicate-worth` is
    ///   `B∧C − A∧B∧C`; publishing the two halves and letting a reader subtract
    ///   is precisely how `+82` survived both of its inputs moving.
    ///
    /// Pure over `results`, so the unit test below grades it with no toolchain.
    pub fn metrics(&self) -> Vec<(&'static str, String)> {
        let [a, b, c, d, e, a_lo, bc, abc, abcd, joint] = self.factor_counts();
        let graded = self.graded().count();
        let frontier = self.factor_frontier().len();
        let ladder = self.section_ladder();
        let mut m: Vec<(&'static str, String)> = vec![
            ("tu-total", self.results.len().to_string()),
            ("graded", graded.to_string()),
            ("match", self.count(TuClass::Match).to_string()),
            ("mismatch", self.count(TuClass::Mismatch).to_string()),
            ("codegen-gap", self.count(TuClass::CodegenGap).to_string()),
            ("vocab-gap", self.count(TuClass::VocabGap).to_string()),
            ("port-error", self.count(TuClass::PortError).to_string()),
            ("capture-fail", self.count(TuClass::CaptureFail).to_string()),
            ("factor-a", a.to_string()),
            ("factor-a-lo", a_lo.to_string()),
            ("factor-b", b.to_string()),
            ("factor-c", c.to_string()),
            ("factor-d", d.to_string()),
            ("factor-e", e.to_string()),
            ("b-and-c", bc.to_string()),
            ("a-and-b-and-c", abc.to_string()),
            ("a-and-b-and-c-and-d", abcd.to_string()),
            ("a-and-b-and-c-and-d-or-e", joint.to_string()),
            ("frontier", frontier.to_string()),
            ("frontier-if-a", self.factor_frontier_if_a().to_string()),
            // The headline projection, derived here so it cannot be assembled
            // from two independently-stale halves. Board #213.
            ("emit-predicate-worth", bc.saturating_sub(abc).to_string()),
            ("writer-sections", PORT_WRITER_SECTIONS.len().to_string()),
            ("workload-sections", self.section_vocabulary().len().to_string()),
            ("ladder-steps", ladder.len().to_string()),
        ];
        // The ladder head, when there is one. Emitted as two keys rather than
        // one "name C=n" string so the numeric one stays `sed`-able, and
        // omitted entirely when the vocabulary is closed — a collector that
        // reads a missing key as 0 would then claim a closed ladder reaches
        // C = 0, so absence must be absence.
        if let Some((name, reach)) = ladder.first() {
            m.push(("ladder-head", name.clone()));
            m.push(("ladder-head-c", reach.to_string()));
        }
        // PROGRESS MASS — emitted only when the scan graded something, for the
        // reason `progress_mass` returns `Option`: a collector must read an
        // empty scan as NO-RESULT, never as any number. The `f`-term inputs are
        // published beside it so the value is never quotable without its
        // denominators.
        if let Some(p) = self.progress_mass() {
            m.push(("progress-mass", format!("{:.5}", p.value)));
            m.push(("progress-emitted-in-class", p.emitted_in_class.to_string()));
            m.push(("progress-emitted-total", p.emitted_total.to_string()));
            m.push(("progress-mismatch-zeroed", p.mismatch_zeroed.to_string()));
        }
        m
    }

    /// **The section vocabulary census**: every distinct section name in the
    /// workload with the number of objs carrying it, most common first.
    ///
    /// The whole of factor C's problem, enumerated. It is a *finite* list —
    /// which is what makes C the one factor in §10.19 with a short route to
    /// closure — so the count of rows is itself the headline and is printed.
    pub fn section_vocabulary(&self) -> Vec<(String, usize)> {
        let mut v: Vec<(String, usize)> = self
            .emit_histogram()
            .into_iter()
            .filter_map(|(k, n)| Some((k.strip_prefix("emit-sec-name|")?.to_string(), n)))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v
    }

    /// Per-TU set of section names **outside** the port's writer vocabulary, for
    /// the graded TUs whose obj decoded. The ladder's input.
    fn extra_section_sets(&self) -> Vec<Vec<&str>> {
        self.graded()
            .filter(|r| r.emit.contains_key("emit-sec-readable"))
            .map(|r| {
                r.emit
                    .keys()
                    .filter_map(|k| k.strip_prefix("emit-sec-extra|"))
                    .collect()
            })
            .collect()
    }

    /// **The greedy section ladder**: which name to teach the writer next, by
    /// the TUs it brings into reach. Each row is `(name, resulting |C|)`.
    ///
    /// Greedy by immediate gain, ties broken by name ascending, and it **does
    /// not stop at a zero-gain step** — it runs until every readable obj is
    /// reachable. That matters: two names that only ever co-occur each score 0
    /// alone, so a ladder that halted on no-progress would report the vocabulary
    /// as unclosable when it is one step from closed. A zero-gain row printed
    /// beside a gain is also the honest way to say "these two are one step".
    ///
    /// Greedy is not proven optimal, and the row order is a *route*, not a
    /// schedule (`ROADMAP.md` §9.16.1). What it establishes is an upper bound on
    /// the length of the route, which is the claim §10.19 makes.
    pub fn section_ladder(&self) -> Vec<(String, usize)> {
        let sets = self.extra_section_sets();
        let mut taught: std::collections::BTreeSet<&str> = Default::default();
        let reach = |taught: &std::collections::BTreeSet<&str>| -> usize {
            sets.iter()
                .filter(|s| s.iter().all(|n| taught.contains(n)))
                .count()
        };
        let mut out = Vec::new();
        while reach(&taught) < sets.len() {
            let mut candidates: std::collections::BTreeSet<&str> = Default::default();
            for s in &sets {
                for n in s {
                    if !taught.contains(n) {
                        candidates.insert(n);
                    }
                }
            }
            let mut best: Option<(usize, &str)> = None;
            for c in candidates {
                let mut t = taught.clone();
                t.insert(c);
                let got = reach(&t);
                // Ties by name ascending: `BTreeSet` iterates sorted and the
                // comparison is strict, so the first of a tie wins and the
                // ladder is reproducible run to run.
                let better = match best {
                    None => true,
                    Some((n, _)) => got > n,
                };
                if better {
                    best = Some((got, c));
                }
            }
            let Some((got, name)) = best else { break };
            taught.insert(name);
            out.push((name.to_string(), got));
        }
        out
    }
}
