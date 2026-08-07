# w-root — PRE-REGISTRATION

    Lane:        w-root (`wt-w-root`), branched at master `ddab417c`
    Partition:   committed `045d6895`, BEFORE any modelling
                 heldout200.txt  200 TUs  sha256 44b2b5c302eafd5e868e97c6b66835fd30267f171ad47f38ea47153783a7f8a2
                 fit650.txt      650 TUs  sha256 ec9efe1a40fc9509b8b9854cd6a73738ac8abc107e8c195494a741105e210911
                 corpus (850)             sha256 43b9d76d992944fd142819b9c4484eb29d62763a923bd37ec95b1c3be51c3681
    Model:       MODEL-CLOSURE-SHA256 10532d33a26805c4154c134b328541f49967066bfece24f01982cd59998535ed
                 over 15 files — work/w-root/freeze.txt
    Incumbent:   `JFP_ALIAS` = `rootmodel.model(st, roots={})`, KA'd NAME FOR NAME
                 against `work/w-quar/predict.py` on **650 / 650, 0 differ**
                 (`work/w-root/ka_fit650.txt`)

---

## §1 The rung

`w-quar` found that the emit-set model **has the edges and no root rule**: a
defined file-scope data object that nothing references is never entered by the
fixpoint, so its initializer's pointees are never predicted. Seven of its twelve
held-out misses were one 38-name set, 33 of them owned by
`?gEaseFuncs@@3PAP6AMMMM@ZA`, and in sample the same family was **16 358 of
18 212** false negatives on **431 of 850** TUs.

This lane adds a **truth-free root predicate** over the `.in` initializer owners:

    JFP_ALIAS_R(pi) = fixpoint(Seed | { d in W : pi(d) }, merged edges, U, W, skip) & U

and registers **`M3A`** as the rule:

> ### `pi(d)` — **d's mangled name is a file-scope VARIABLE and it is non-const**: `?<name>@@3<type><cv>` with `cv == 'A'`, `d` not a `??_7`/`??_R`/`??_G`/`??_E` name and with no `$` in its qualified part. In words: **a defined, non-const, file-scope data object is a root by virtue of being defined, whether or not anything references it.**

It reads the `.gl` name and the `.in` owner list and nothing else. It does not
read the reference obj, `D`, `E`, or any quantity derived from them, so it is in
the same class as `JFP_ALIAS` and **not** in `ALIAS_IN`'s class (which conditions
on `D` and is a ceiling, never a model).

`M3` — the same rule with the cv-modifier clause dropped — is registered as a
**co-primary**, because in sample the two are set-identical on all 650 TUs and
this corpus therefore **cannot** separate the const axis. Both are scored in the
one held-out pass; that is two frozen models on one gate, not a refit.

---

## §2 The held-out set — why 200 and not 21

`w-quar` spent a 21-TU set and reported that its 95 % Clopper-Pearson interval
was **17x wider** than the in-sample one: *"a one-shot 21-sample can refute a
rate; it cannot estimate one."* The three sizing constraints, registered:

| | |
|---|---|
| estimate the rate | CP width at `p ~ 0.37`, `n = 200` is `+/- 0.067` against the in-sample `+/- 0.037` — **1.8x**, not 17x |
| **see TU reach at all** | `B^C` is 151 of 871. The partition puts **31** of them in the held-out set and 114 in the fit set. A 21-sample carries ~3.6 and **cannot measure reach**; this is the number board #345 says is the only one that matters |
| leave a usable fit set | 650 remain, and the fit chooses among **23 named binary predicates** — negligible capacity |

Selection rule, computable by anyone from the committed corpus listing and
nothing else: **sort the 850 source paths by `sha256(path)` and take the first
200.** `carve.py` opens no cache entry, obj, IL blob, truth file or prediction.
Disjoint from `w-quar`'s spent 21 on both sides (**0** and **0**).

`mkidx.py` takes `heldout200.txt` as a **forbidden list** and exits non-zero if a
fit-side script is ever pointed at a held-out TU, so contamination is a crash and
not a convention. Verified: it refuses (exit 2) when handed the held-out list.

---

## §3 The candidates, and how each is expected to die

Enumerated in `rules.py` as one closed set, all graded as whole models on the fit
650 (`sweep_fit650.txt`). The registered reading of the fit:

| channel | rule | fit-side verdict |
|---|---|---|
| **the definition alone** | `ALLW` — every `.in` owner is a root | **DEAD**: exact 238 -> **27**, precision 0.998 -> **0.186**. 507 206 of 537 216 owners are idle; 94.5 % are `??_7`/`??_R` |
| **linkage** | `UNDEC` — `extern "C"` data | **DEAD**: 652 owners, **0** wanted, exact unchanged at 238 |
| **linkage** | `SC<k>` — the storage-class byte | **DEAD BY DEGENERACY**: `sc == 0x00` on **all 537 216** owners. The channel carries no information in this corpus |
| **record structure** | `TAG01`/`TAG02`/`TAG04`/`TAG0E` | **DEAD**: 04 and 0E are empty; 01 and 02 lose 210 / 211 TUs |
| **initializer property** | `NPTR_GE5`, `NPTR_GE17` — a big initializer | **DEAD**: lose 199 TUs each; precision 0.33 and 0.23 |
| **flag word** | `F20_400/1000/2000/4000` — w-db's `M11` bits | **DEAD**: lose 200-211 TUs. Reproduces `M11`'s finding on a new denominator |
| **flag word, NEW** | `F20_20000`, `F20_40000` — **bits 17 and 18, which `w-db/joint.py`'s twelve-rule enumeration never tried** | **REAL BUT DOMINATED**: `F20_40000` is +91 / **-6**. Registered as a control, not the rule |
| **name class** | **`M3A` / `M3`** | **SURVIVES**: exact 238 -> **460**, gained 222, **lost 0**; precision 0.99827 -> **0.99840** (up), recall 0.89574 -> **0.98984** |
| const axis | `M3B` — const file-scope variables | **NO EFFECT**: 14 owners, 0 wanted, exact unchanged. The corpus cannot test the axis `PHASE7_PLAN.md` clause (5) turns on |

**All of the above is IN SAMPLE.** It is the fit, it is reported as the fit, and
nothing in it is evidence about the held-out 200.

---

## §4 REGISTERED PREDICTIONS — held-out 200, scored ONCE

Points are the fit-side rate scaled to `n = 200`; intervals are central 95 %
binomial acceptance regions at that rate.

| # | quantity | **point** | interval |
|---|---|---:|---|
| **Q1** | `JFP_ALIAS` (the INCUMBENT, `NOROOT`) exact of 200 | **73** | [60, 87] |
| **Q2** | **`M3A` exact of 200** | **142** | [129, 154] |
| **Q3** | **`M3A` − `NOROOT`, PAIRED, gained by name** | **+68** | [+55, +82] |
| **Q4** | **TUs `NOROOT` gets and `M3A` LOSES** — the losing direction | **0** | [0, 4] |
| **Q5** | `M3` exact of 200 | **142** | [129, 154] |
| **Q6** | `M3` and `M3A` predict the SAME set on how many of 200 | **200** | [193, 200] |
| **Q7** | `M3B` exact − `NOROOT` exact | **0** | [0, 4] |
| **Q8** | `UNDEC` exact − `NOROOT` exact | **0** | [0, 4] |
| **Q9** | `ALLW` exact of 200 — the catastrophic control | **8** | [3, 14] |
| **Q10** | `F20_40000` exact of 200 — the dominated control | **99** | [86, 113] |
| **Q11** | micro precision of `M3A` — *SECONDARY* | **0.998** | [0.99, 1.000] |
| **Q12** | micro recall of `M3A` — *SECONDARY* | **0.990** | [0.96, 1.000] |
| **Q13** | micro F1 of `M3A` — *SECONDARY* | **0.994** | [0.97, 1.000] |
| **Q14** | `Ease*`-family false negatives remaining under `M3A` | **0** | [0, 60] |
| **Q15** | largest FN family under `M3A`, in TUs | **3** | [1, 15] |
| **Q16** | **`NOROOT` TU REACH** — `\|exact ∩ B^C\|` of 31 | **26** | [22, 30] |
| **Q17** | **`M3A` TU REACH of 31** | **28** | [24, 31] |
| **Q18** | **REACH GAINED, by name** | **+2** | **[0, +5]** |
| **Q19** | REACH LOST, by name | **0** | [0, 2] |
| **Q20** | `\|B^C ∩ heldout200\|` — the denominator, fixed by the partition | **31** | exact |

**Q18's interval contains 0 deliberately.** Boards #345 and #302 record that
per-TU exact and TU reach are decoupled and that `w-quar`'s pass bought **+0**
reach. A registered reach prediction that could not come in at zero would be
unfalsifiable, and a per-TU-exact gain must not be allowed to stand in for reach.

---

## §5 DECLINE CLAUSES — binding, and honoured whatever the number

1. **DECLINE if Q3 ≤ +20** — the rule buys under a third of its fit-side rate.
2. **DECLINE if Q4 ≥ 8** — the rule destroys TUs the incumbent had.
3. **DECLINE if Q11 (micro precision) < 0.95** — `w-emitpred`'s V3 fail-closed floor.
4. **ONE SHOT. If any of 1-3 fires, the rung is DECLINED and reported as
   declined.** No re-fit, no re-score, no second predicate. A refitted model has
   spent the gate and learned nothing; this project has the record of exactly
   that failure (a computed-address schedule read 360/360 in sample and 296/394
   out of sample, and only the out-of-sample number decided anything).
5. **The held-out 200 are SPENT after this run** and may never be reused as a
   held-out set, by this lane or any other.
6. **Nothing ships under `crates/`.** This is a measurement lane; `git diff
   ddab417c -- crates/ scripts/ Cargo.toml Cargo.lock fixtures/` must be 0 bytes.
7. **Per-TU exact BY NAME on both sides**, gained and lost as two name lists,
   never a net count (trap 8, board #345).
8. **TU reach is reported separately and is not implied by Q2/Q3.**

## §6 Declared bias

**INFLATIONARY** — a lane that has found a clean fit wants the gate to pass. The
guards that can catch it, all registered above and all able to fail:

* **Q4 and Q19 register the LOSING directions** at [0, 4] and [0, 2].
* **Q3 is PAIRED** — same 200 TUs, same frozen code, same run — so a gain cannot
  be manufactured by a population difference.
* **Q9 and Q10 are controls that must come in LOW and MIDDLE respectively.** If
  `ALLW` scores well the fit-side story is wrong; if `F20_40000` beats `M3A` the
  registered rule is the wrong one.
* **Q6 and Q7 register that the const axis is UNTESTED**, so a lane cannot later
  claim the corpus confirmed `PHASE7_PLAN.md` clause (5)'s const clause.
* **Q18's acceptance region contains 0**, so "reach did not move" is a HIT and
  not a face-saving reinterpretation.
