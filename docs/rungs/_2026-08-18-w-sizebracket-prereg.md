# PREREG — `w-sizebracket`: c2's inline SIZE axis, measured against the oracle so the constant is DERIVED and not FITTED

    Lane:      w-sizebracket
    Kind:      characterization lane (docs/rungs/README.md § "Lane kinds", kind 3)
    Base:      master `1744ced1`
    Frozen:    2026-08-18, as the branch's FIRST commit, before any probe
    Board:     #3270–#3275, allocated by the coordinator (do NOT read the
               next-free pointer — it is routinely wrong at the tip that prints it)

---

## 1. The question, in one sentence

**`w-dataseam` found a two-term rule, owns one term, and swept the grader to
guess the second; this lane asks REAL `c2.dll` what the second term's UNIT and
VALUE are, so that the constant is derived from the oracle rather than fitted to
`fnbyte-exact`.**

`w-dataseam` (`docs/rungs/2026-08-18-dataseam.md` §6, §6.1, §6.3) established:

* the lifted `data_syms`-scoped rule — *callee defined here by the emit binding
  ∧ `∉ tu_modelled_callees` ∧ `cflow_key ≠ eh-state1` ⇒ REFUSE* — refuses a
  population **64.6 %** wrong against a **6.5 %** base rate and removes **91.7 %**
  of every remaining wrong body in the port;
* shipping it un-sized costs **992** `fnbyte-exact` bodies;
* a bracket of **`[180, 231]`** *IL segment bytes* makes the cost exactly **0**
  while still removing **1,189** wrong bodies;
* and that bracket is **fitted** — its position was found by sweeping the grader,
  and its unit (IL segment bytes) is not the unit the repo's other two inline
  constants are stated in (`INLINE_DECLINE_BYTES` 128, `INLINE_DECLINE_LOOP_BYTES`
  80 — both tops of measured obj-grid brackets in **emitted** bytes).

**The defect to avoid, named in advance:** a constant in the wrong unit wearing
the same clothes as the measured ones.

## 2. The DECISION RULE, frozen — what "derived" means here

This is the clause that exists so this lane cannot repeat `w-dataseam`'s
goalpost move in the other direction.

> **D1 — the cut is selected from an ORACLE cross-tabulation, never from
> `fnbyte-exact`.** The selection statistic is real `c2.dll`'s own
> kept-vs-inlined verdict per call edge (GRID-W's observable: does the reference
> caller's `REL24` target set name the callee?). The `fnbyte-*` price is measured
> **afterwards, as a consequence**, and may not be consulted while choosing the
> cut. If the two disagree — if the oracle-derived cut costs `fnbyte-exact` — the
> lane reports the disagreement and **declines**; it does not slide the cut.

> **D2 — the cut must be stated in the unit c2 is measured to decide on**, with
> the measurement that establishes that unit named. If two candidate units both
> separate, the one with the cleaner separation (fewer mixed cells) wins, and the
> margin between them is published.

> **D3 — a single cell is not a rule.** Every finding is published as a SERIES
> over cells (`w-slots` #3147; `w-bind16`'s `2n+1`; `w-section`'s R-SEC n=1..4;
> `w-dagorder`'s n=1..8), with the series' `n` range stated.

> **D4 — profile scope is part of every claim.** Both `/O1` (the workload's own,
> `/nologo /c /GR /O1 /Oi /EHsc`) and `/Ox` are measured and the disagreement is
> reported. `w-section` found `/Ox` disagrees with `/O1` on 7 of 8 fields;
> `w-dagorder` found `/Ox` **inverts** the allocator order on the 6 of 20 cells
> carrying signal. A finding without a stated profile scope is not landed.

> **D5 — ship only if D1 and D2 are both satisfied.** Otherwise
> `Outcome: declined` with the measurement, which is a full deliverable
> (`w-dataseam`'s precedent). Shipping requires, two-sided: `match` ≥ 26,
> `mismatch` **0**, `fnbyte-exact` Δ ≥ 0 measured in a bracketed back-to-back
> block, and the decline's own cost counted in the same units.

## 3. ADMISSIBLE CONTAINER FACTS — registered NOW, before any probe

`w-dataseam` declined partly because segment length was not on its frozen list.
This lane registers the list **broadly and up front**, so that neither admitting
nor refusing a fact mid-lane is a goalpost move. Every fact below is decidable
from the IL container before codegen, on the port's existing readers:

| # | fact | where |
|---|---|---|
| A1 | the callee's `.ex` IL segment **byte length** (`segs[j].len()`) | `c2-il` census |
| A2 | any **instruction/element count** derivable by walking that segment | `c2-il` |
| A3 | any per-function **count field carried verbatim in the IL** (`.gl` record bytes, `.ex` header words), should one exist | `c2-il` |
| A4 | the `.gl` attribute bytes already read: `FN_FLAG_INLINABLE` (`0x40`), linkage/plain-external (`gl::plain_external_names_among`) | `c2-il::func::gl` |
| A5 | `cflow_key(seg)` — including its loop and EH-state components | `c2-il` |
| A6 | the emit binding's ground map, `tu_modelled_callees`, the modelled set | `c2-il` |
| A7 | the port's own **lowered** `/Gy` body length for the callee, where the port can lower it (`comdat_function_body(...).text.len()`) | `c2-core::comdat` |
| A8 | the per-function optimization word (`opt_word`) / profile gate | `c2-il` |

**Explicitly NOT admissible, at any point in this lane:** a name list, a
population list, a string-literal term, a TU list, and any statistic computed
from `fnbyte-exact`/`fnbyte-differs` (D1).

**A7 is registered but is expected to be unusable as the shipped input** —
GRID-W §2 measured that the port can lower the callee at only 3,165 of 7,552
edges and at exactly ONE `kept` site. It is registered so that measuring it is
not a mid-lane admission.

## 4. Ceilings — probability form, NO discount factor

Stated as the maximum the lane could realize if everything it hopes is true.
Discounting them here is the move `docs/rungs/README.md` forbids.

| ceiling | value | basis |
|---|---:|---|
| wrong bodies removable if the completed rule ships at a zero-cost cut | **1,189** (1,133 `differs` + 56 `reloc-differs`) | `w-dataseam` §6.1 |
| that as a share of `fnbyte-differs` | **57.9 %** (1,133 / 1,958) | same |
| that as a share of every wrong body the instrument grades | **47.6 %** (1,189 / 2,499) | same |
| `fnbyte-exact` gain | **0** — this lane cannot add a byte-exact body | it only refuses |
| TU `match` gain | **0** | a characterization lane converts nothing |
| census gain | **+0** | kind 3 |

**The ceiling on the goal metric is ZERO and is stated first.** Everything this
lane can win is latent-hazard removal (`w-fnbyte` #876–#879) plus a measured
rule. It cannot move `match` and it will not claim to.

## 5. Predictions — frozen, with probabilities

| id | P | prediction |
|---|---:|---|
| **P1** | 0.70 | Re-reading the base at `1744ced1` in this worktree reproduces the dispatch's `fnbyte-exact` **35,899** exactly; if not, the delta is ≤ ±2 (#3249's floor) |
| **P2** | 0.95 | `match` **26** / `mismatch` **0** / `codegen-gap` **0** / `vocab-gap` **844** / `capture-fail` **8** in **every** build this lane makes |
| **P3** | 0.90 | The anchored key count reads **394** with `grep -cE '^ *gap-metric \S+ \S+$'` |
| **P4** | 0.65 | Cross-tabulating the callee's **IL segment length** against real c2's kept/inlined verdict over GRID-W's ~7,552 workload call edges yields a **monotone** separation — a band below which almost everything is inlined and above which almost everything is kept |
| **P5** | 0.70 | That separation is **strictly dirtier** than the emitted-byte one: GRID-W's emitted-byte table has **zero** `inlined` sites above 80 B, and the IL-byte table will have a non-empty mixed region above its own first-kept point |
| **P6** | 0.50 | The IL-byte cut derived from the oracle cross-tab (D1) lands **inside** `w-dataseam`'s fitted `[180, 231]` |
| **P7** | 0.55 | The ratio (IL segment bytes) / (emitted `.text` bytes) over the workload's defined-here callees has a median in **[1.4, 2.4]**, i.e. the fitted 180 IL is the shadow of GRID-W's measured (80, 96] emitted boundary |
| **P8** | 0.35 | The IL carries a per-function **instruction count** (A3) that `c2.dll` loads into `[sym+0x50]` — the WORD `P_INLINE.md` §2.1 shows the size test reading — making the exact quantity c2 tests readable from the container |
| **P9** | 0.60 | A controlled probe series at the workload profile, varying one callee's size monotonically across the IL range `[176, 232]`, shows the inline→decline flip **within** that range |
| **P10** | 0.80 | `/Ox` **disagrees** with `/O1` on the boundary — the flip happens at a strictly larger callee at `/Ox` (`WB_INLINE_FINDINGS` F1/F2 put the favour-speed ceilings at `(212,252]` and `(156,164]` against `/O1`'s `(300,308]` and `(100,116]`) |
| **P11** | 0.45 | The lane's own most-likely outcome is **`built`** (the characterization lands, no `crates/` change ships) |
| **P12** | 0.30 | The completed two-term rule **ships**, at a derived cut, with `fnbyte-exact` Δ = 0 and `mismatch` 0 |
| **P13** | 0.85 | `scripts/gate.sh --jobs 16 --require-graded` PASSes at both ends with an identical per-lane gate-count table, range length asserted |
| **P14** | 0.75 | `w-dataseam` §6.1's lower bound holds under an oracle measurement: real c2 **keeps** the call to `?MakeString@@YAPBDPBD@Z` in `ContentMgr_Xbox.cpp`, i.e. the witness is a `kept` edge and not an artifact of the grader |
| **P15** | 0.55 | The unit c2 actually decides on is **neither** of the two candidates as stated — it is a pre-codegen instruction/element count, and both byte units are proxies for it whose fidelity is what the cross-tab measures |

## 6. Mutant colours, registered up front

Applies only if a `crates/` predicate ships (P12). If the Outcome is `declined`
or `built` with no shipped predicate, this section is reported as **NOT RUN, by
the pre-registered condition**, in those words — never omitted, and never
substituted with mutants of an unchanged tree.

| id | mutant | registered colour |
|---|---|---|
| **MS1** | flip the size comparison (`>` ⇄ `<=`) at the shipped cut | **RED** — the fenced population inverts; `fnbyte-exact` must fall |
| **MS2** | move the cut to 0 (refuse every size) | **RED** — reproduces `w-dataseam`'s `lift`, `fnbyte-exact` −992 |
| **MS3** | move the cut to `usize::MAX` (exempt every size) | **RED** — the rule refuses nothing new; `fnbyte-differs` returns to base |
| **MS4** | drop the `cflow_key ≠ eh-state1` conjunct | **RED** — the incumbent `w-fence163` witness `?ContentPath@XboxContentMgr@@UAAPBDH@Z` must move |
| **MS5** | drop the `∉ tu_modelled_callees` conjunct | **RED** — re-fences the graded elide/splice population |
| **MS6** | replace the derived cut with `w-dataseam`'s fitted 231 | **GREEN expected, and that is the point** — if the two are indistinguishable on this workload, the derivation's value is the *record*, not the number, and the rung must say so |

**MS6 is registered as a predicted GREEN on purpose.** A mutant that cannot be
told apart from the shipped value is evidence about the workload's resolution,
and registering it in advance stops it being reported as a success.

## 7. Registered biases — the ways this lane could fool itself

1. **Fitting under a new name.** Deriving a cut from an oracle cross-tab whose
   *bands* were chosen after seeing where `fnbyte-exact` is flat. Mitigation:
   bands are fixed at 16 bytes (GRID-W's own, unchanged) and the raw per-edge
   dump is committed so any other banding can be re-derived.
2. **Keeping a small residual term to make the price come out flat** —
   `w-dataseam`'s registered bias #2, inherited verbatim.
3. **Reading a null as a negative result.** A cross-tab that separates because
   the population is thin, not because c2 decides that way. Mitigation: every
   band prints its `n`, and any claim resting on a band with `n < 20` is stated
   as unresolved.
4. **Quoting a `/O1` finding without its scope** (D4).
5. **`fnbyte-*` drift read as an effect** (#3249). Mitigation: base re-read
   immediately before the tip, back to back, cache state and `dc3-decomp` head
   stated, any effect under ~10 bodies treated as unattributable.
6. **The per-symbol `fnbyte-differs` set compare** (#3237) — **VOID** for this
   lane, which changes the admitted population by construction. A name-stable
   per-TU shape multiset is the only acceptance comparator here.
7. **`fnbyte-reloc-differs` as a monotone control** — `w-dataseam` §12.2
   measured it going **up** by 67 under a change that only refuses. Not used as
   a control.
8. **The probe-soundness trap** (#3219/#3231): a worktree with no `compilers/`
   makes every capture test silently skip, and RED reads GREEN. Mitigation:
   `C2RS_REQUIRE_TOOLCHAIN=1`, a control pinned by NAME re-run in both
   configurations, executed counts and durations asserted rather than exit codes,
   and any unvalidated colour **voided rather than downgraded**.

## 8. Invalidation conditions

The lane's headline is **void**, not provisional, if any of these holds:

* the base re-read at `1744ced1` differs from the dispatch's figures by more than
  ±2 `fnbyte-exact` and the cause is not identified;
* the bracketed back-to-back block's two base readings differ in any column;
* the environment control (§7.8) fails to distinguish provisioned from
  unprovisioned in this worktree;
* `match` ≠ 26 or `mismatch` ≠ 0 in any build;
* the oracle cross-tab reports any `unknown` arm at a rate that could flip a band
  (GRID-W read 0 of 7,552; anything above 1 % is reported and the affected bands
  are marked unresolved).

## 9. Grids frozen by CONTENT HASH

Recorded here at freeze time by path; the hashes are written into the rung's §2
from the files as they are read, and the workload manifest's hash is compared
against `w-dataseam`'s to establish whether the corpus moved again.

| input | path |
|---|---|
| workload TU list | `work/dc3-workload/files.txt` |
| workload flags | `work/dc3-workload/flags.txt` |
| this lane's probe cell generator | `work/w-sizebracket/gen_cells.py` |
| this lane's probe cell corpus | `work/w-sizebracket/cells/` |

`w-dataseam` measured, at its own base:

    files.txt  4996839bf89780a2dea9ed005450d8953961355a9eb2292cc1bc22572a6853b6
    flags.txt  fa8ba48aa21229773116bf0decff3b7e9e5e7f7ee356c3e347c506038ffbcb48

**If either differs at `1744ced1`, the corpus moved again inside one day and
every `w-dataseam` figure this lane quotes is re-measured rather than carried.**

## 10. Seams

Owned: `crates/c2-il` (the `data_syms` region and the size predicate), any
`c2-core` codegen the rule needs, `docs/whitebox/` (extending `ref/`, not
starting a parallel record). **Not owned:** `crates/c2-harness/src/gap/tests.rs`,
`crates/c2-harness/tests/` (peer `w-deadsites`), `scripts/` (peer
`w-coldcross`). A harness guard, if needed, is described and routed, not landed.

Any disassembly-derived constant entering `crates/` requires a
`docs/whitebox/DISCLOSURE.md` row naming the address **in the same commit**.
