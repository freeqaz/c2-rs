# PHASE7_PLAN — the emit set, the obj shape, and the route from 6 to 871

    Lane:      w-phase7plan, 2026-08-02
    Prereg:    rungs/_2026-08-02-w-phase7plan-prereg.md — committed at
               `5f5b156`, before the first measurement; scored in §9 below.
    Evidence:  four sub-lanes (probe grid / diagnostic channels / .gl corpus /
               gap-shape+sections), all black-box, all measured this session.
               Scratch under `work/probes/`, `work/wB/`, `work/lane-c/`,
               `work/phase7plan-d/` in worktree `wt-w-phase7plan` (gitignored;
               scripts and raw cells preserved there for re-derivation).
    Status:    a PLAN. Nothing here is built. Every number is labelled
               measured / ceiling / estimate, with the predicate that
               produced it. TU match is 6 at both ends of this lane.

**One-line statement:** *Phase 7 is not one wall but four factors that
multiply — emit set (A), name binding (B), section shape (C), codegen breadth
(D) — their intersection reproduces the 6 matches exactly, the emit predicate
is now fitted black-box with zero violations on 172 designed cells, and the
route runs R1 (#158, +2 TUs) → sections + fail-closed emit model (ceiling 82)
→ binding/synthesis (ceiling 420+) → the codegen second half.*

---

## 1. What Phase 7 turns out to be — four factors, and they multiply

Measured this session (lane D; full census of 871 cached reference objs; COFF
reader validated 871/871 against the harness's own `emit-emitted`):

| factor | predicate (over 871 graded TUs) | TUs |
|---|---|---:|
| **A** | emit-set cardinality equal (`fn_total == emit-emitted`, `LO`-anchored) | 25 |
| **B** | `emit-set-ceiling-today` — every emitted symbol binds | 324 |
| **C** | obj section set ⊆ the port's widest current writer (`.drectve/.debug$S/.XBLD$W/.text/.pdata/.rdata`) | **84** |
| **D** | every emitted COMDAT inside the port's codegen class | **8** |

**A∧B∧C∧D = 6 — exactly the observed match set**, from four
independently-derived predicates. This factorization is the planning model, and
it kills two assumptions the project was carrying:

1. **The emit-set model is not the binding constraint. Section shape is.**
   C = 84 is 4× tighter than B = 324. A perfect emit-set model plus a perfect
   binding lifts TU match to at most **B∧C = 82** (83 repaired) until the port
   can write `.data`, `.bss`, `.rdata$r`, `.text$yc/$yd`, `.CRT$XCU`,
   `.xdata$x` — the **entire workload section vocabulary is 13 names**
   (measured, full census), so C is finite and enumerable.
2. **The walls are anti-correlated, which sets the order.** 82 of the 84
   section-reachable TUs are already `emit-set-ceiling-today`; only 1 is on
   the wall. Shell generalization and emit-set modelling attack nearly
   disjoint populations and can run as parallel lanes.
3. **D = 8 is the last cliff and the largest.** Only 8 TUs have every emitted
   function in the port's class, and 6 of those emit nothing. Codegen breadth
   (the old Phases 1–6) is what converts reachable TUs into matches — Phase 7
   makes them reachable and cannot do more.

Two supporting shape facts (lane D, both labelled `LO`-anchored):

* **The spurious bucket is a cliff, not a gradient**: over the 842
  `segments > COMDATs` TUs, delta median **1,982**, p10 = 490; delta ≤ 5 on
  **3** TUs. There is no "shave a few spurious COMDATs" tail to farm; the
  emit set must be modelled, not patched.
* **EH is on the TU-assembly critical path, not a late phase**: `.rdata$r`
  (per-function EH records) appears in **676 of 871** objs, `.xdata$x` in 67.
  The commonest beyond-reach extra-set is `{.bss,.data,.rdata$r}` (352 TUs).
  Phase 5 groundwork feeds C directly.

## 2. The emit predicate — fitted, black-box, zero violations

Lane A: 172 designed cells at `/O1 /Oi /EHsc /GS- /c`, crossing linkage class
(8+ values) × reference kind (7+ values) × TU context, graded by **both** the
`.cod` PROC set and the real obj's `.text` COMDAT leaders (agreement 172/172
after the reader survived two red known-answer checks).

> **Emission is a least-fixpoint reachability from roots, computed over
> *kept* definitions only, at ODR-use granularity, pre-optimization.**
>
> **Roots:** (1) every definition with external non-COMDAT linkage — plain
> extern, `extern "C"`, *any out-of-line definition* (member, static member,
> virtual), and anonymous-namespace functions not declared `static`;
> (2) explicit instantiation definitions, including never-referenced members;
> (3) `__declspec(dllexport)` closure incl. implicit special members;
> (4) dynamic-initializer thunks (`??__E`); (5) kept data definitions —
> external-linkage data and non-const internal data (internal *const* data is
> dropped when unreferenced and its references then do not count).
>
> **Propagation:** F is added if an already-kept definition ODR-uses it — a
> call anywhere in the pre-optimization body (including statically dead
> branches and `catch` handlers), an address-take, or a data initializer.
> `sizeof` does not count. References from removed (never-kept) definitions
> never count — the fixpoint is over kept code only; cycles do not sustain
> themselves.
>
> **Vtable rule:** a kept constructor of C keeps C's vtable, whose slots
> force **every** virtual of C plus the synthesized scalar-deleting
> destructor, called or not.

Refuted along the way, each by cells that would have gone red otherwise:
closure-over-all-bodies (13 dead-referrer cells); internal-linkage-never-emits;
anon-namespace-equals-static; size/inlinability gating (independent
re-refutation of §9.18.6); post-optimization reference sets; redirector
folding; and "c1xx strips unemitted bodies from the IL" — a TU that emits
*nothing* still ships its bodies in `.ex`, byte-count-confirmed.

Three corroborations from the other lanes:

* **The front end's removal pass is observable by name** (lane B): `/W4`
  C4505 + `/Wall` C4514 name removed functions with **precision 1.00, recall
  0.928** over 97 probe bodies; adding a one-step closure over the removed
  set makes it **exact (69/69 skips)**. The misses are one class — cascade
  tails of a non-iterated front-end pass, which c2 then drops via its own
  `globally unreferenced` disjunct.
* **The `.gl` name table is a necessary condition** (lanes A+B): every
  emitted function's name is present (28/28, 0 exceptions); unreferenced
  statics and unreferenced COMDATs are absent. It is a fail-closed upper
  bound, not the predicate (presence ≈ "external linkage or referenced by
  something").
* **The obj's COMDAT Selection byte encodes the linkage split** (lane A):
  Selection 1 (NODUPLICATES) for strong-linkage and kept statics, 2 (ANY)
  for COMDAT-linkage, over 331 emitted COMDATs with zero anomalies — the
  port must reproduce this byte anyway, and the model's linkage axis is
  exactly what determines it.

**What is refuted on the container side (lane C, full 871-TU census, readers
known-answer-gated on App.cpp 38/158 and TextFile.cpp 674/70/32-30):**

* The `.gl` name separator does **not** encode the linkage half: only
  **12.1 %** of `00`-introduced framed-body-record names are emitted
  (registered ≥ 80 %, refutation floor 60 % — refuted hard), vs 6.0 % for
  `26`. It is a name-class marker, ~2× rate difference, not a predicate.
* No field in the `.gl` record decides emission: byte-identical inter-field
  windows carry opposite fates for 70–79 % of records in large TUs.
  **c1xx did not already tell c2**; c2 computes the decision.
* What the separator repair *does* buy: **`00∪26` record names cover
  97.43 % of everything c2 emits** — the fail-closed name universe for
  `Emit(name)` — and `MeterEffect.cpp`'s 10-vs-13 anomaly is exactly its
  three `26`-named records (10 + 3 = 13), moving it from "synthesis" to
  "decode + binding".

**Standing caveat, and it is the plan's biggest unknown:** the predicate is
fitted on 172 *synthetic* cells. Real headers (STLport, templates over
templates, `??_9` adjustor thunks, multiple inheritance) are out of the grid.
It ships only through the out-of-sample gate in R3 below.

## 3. The route

Order, with each step's payoff labelled. "Ceiling" = necessary-not-sufficient;
conversions happen only when all four factors close over a TU.

| rung | what | payoff | basis |
|---|---|---|---|
| **R0** | instruments + probes (parallel, cheap) | 0 TUs; de-risks everything | measured needs, §5 |
| **R1** | **#158 both halves** — bare-`4C` thunk decode + the 8-section `??__E` obj | **+2 TUs** (TomCrypt/Zlib licenses) | obj shape byte-determined (§10.16); decode characterized (§10.12) |
| **R2** | **section vocabulary** — `.data`, `.bss`, `.text$yd/$yc`, `.CRT$XCU`, then `.rdata$r`/`.xdata$x` with Phase 5 | lifts C from 84 toward 871; joint ceiling B∧C 82→83 first, then tracks EH progress | 13-name census (lane D); exact per-section joint counts are one query on `work/phase7plan-d/sections.pkl` |
| **R3** | **the emit-set model, fail-closed** (§9.18.8 shape) — the fitted reachability predicate as `Emit(name)/Skip/Unknown` per segment, `Unknown` ⇒ refuse TU | makes the 842 reachable in principle; near-term joint ceiling **82** with R2's first slice | predicate §2; out-of-sample gate §5-D1 before it ships |
| **R4** | **binding/framing repairs** — #159 family (`ordinary` no-record = header virtuals; `?CanSelect@…` 50-bind/3-no-record probe first) | **+9 today / +65 wall** (measured §10.17), 0 conversions alone | ceilings, not payoffs |
| **R5** | **synthesis** — #152 `??_G/??__F/??_E/??_D` bodies (in neither `.gl` nor source; must be generated like c1xx+c2 do) | **+4 today / +69 wall**, 0 conversions alone | §10.13; blocked less than believed — the *names* are computable/known, the bodies are new codegen |
| **R6** | **emission order + label counter** — callees-before-callers, siblings in source order, vtable virtuals in slot order after ctor (lane A, probe-scale); `.cod` allocation order for labels (§9.3) | 0 TUs alone; a set right + order wrong is still a mismatch | probe-measured; must be re-verified on workload objs before trust |
| **R7** | **the second half: codegen breadth** — Phase 6 + expression layer *jointly* (§10.15: constructs pay only together; `if-n` first at +6), EH (which also feeds R2's `.rdata$r`), frames, member calls | this is where conversions actually land | the 14 named first targets below |
| R8 | wall multi-category residue (305 TUs need ≥2 of R4/R5/…), `.bss` ≥3-object permutation, long tail | the last 400-odd TUs | known holes, §5 |

**First conversion targets after R1** (lane D Q4, measured: delta ≤ 10 ∧
`ceiling-today` ∧ no EH sections ∧ section-in-reach, minus the 6 matched — 14
TUs, named): `mmio.cpp` (11 emitted, **8 already in class** — the nearest
non-degenerate TU to a match in the workload), `EncryptXTEA`,
`IPP_basicmath_xbox`, `xboxmem`, `negate_test`, `vsnprnc`, `Sort`, `osfinfo`,
`undname`, `vswprnc`, `xboxheap`, `jsonwriter`, `xlrcimpl`, `JsonMemory`.
These overlap §9.16.4's near band almost exactly, and §10.15 stands: each
needs ≥2 constructs at once; none falls to a single rung.

**The terminal arithmetic, honestly:** 871 requires all four factors at ~1.0 —
full section vocabulary (C: 13 names, EH-gated for 676 TUs), the emit model +
binding + synthesis (A·B: 324 → 420 repaired → the 451-TU wall, of which 305
need ≥2 items at once), order (R6), and the whole codegen program (D). Phase 7
as scoped here (A, B, C, order) is the *reachability* half; no step in this
plan converts a TU except R1's +2, and every widening estimate downstream
inherits the 6.5×–142× clean-to-realized spread (§10.6). Anyone quoting this
plan's ceilings as a schedule is repeating §9.16.1.

## 4. The first rung, concrete — R1, #158, day one

**What:** convert `TomCryptLicense.cpp` and `ZlibLicense.cpp` from
`vocab-gap` to `match`. Both halves are specified:

* **Decode** (`crates/c2-il`): split `ExToken::Lo` (`4C 4F 11`) into `4C` +
  optional `4F 11` record (§10.12's grammar), so the bare-`4C` `??__E` body
  parses; decode the 145-byte thunk segment (its tail already reads as
  `Return` under `codec.rs`'s own decoder; target assembly transcribed in
  §10.10). **Coordination**: `codec.rs` and `bundle.rs` are held by live
  lanes this session — this rung starts when they free, or lands via the
  owning lane; the K1 round-trip gate (byte-for-byte over 212 fixtures) is
  the non-negotiable control on the re-tokenization.
* **Obj shape** (`crates/c2-core` COFF writer): the 8-section obj
  (`OBJ_DYNINIT_SHAPE.md` — section set/order, all 24 symbol records,
  9+1 relocations incl. REFHI/REFLO/PAIR blocks, Selection 2 + associative
  `.pdata`, `.CRT$XCU` entry, `.bss` linkage variant, and the **JamCRC**
  string-COMDAT name, all determined or computable). Fixture
  `fixtures/cpp/il_dyninit_static.cpp` already reproduces the payload
  byte-identically.

**Controls:** (1) byte-exact against both TUs' cached reference objs **at
`/O1`** — never `/Ox`, which is a structurally different obj for this class
(§10.16); (2) the K1 round-trip over 212 fixtures; (3) `scripts/gate.sh`
12/12 with its fixture-verdict count quoted; (4) the workload scan: match
6 → 8, mismatch 0, and `emit-set` keys byte-identical elsewhere.
**Refuters:** any K1 byte moving; any existing fixture verdict changing;
either license TU landing `mismatch` (a wrong synthesis is worse than the
current refusal). **Boundary:** one object per TU, `??__E/??__F` only —
§10.12 measured `??_G` behaving differently on the decode side, and the
≥3-object `.bss` permutation is unsolved (§10.16); widening past either
boundary without new measurement is refuted-in-advance.

## 5. Decision points, each with its cheapest resolver

* **D1 — does the fitted predicate survive real TUs?** *The biggest unknown
  in the plan.* Resolver, in order: (a) one `/Wall` pass over the workload
  recording C4505/C4514 per TU (lane B's channel; zero extra instrument —
  it rides the existing capture invocation) and comparing warned∪closure
  against each TU's `.cod` PROC complement; (b) a Python prototype of §2's
  fixpoint over ~20 held-out real TUs' IL, its predicted PROC sets
  **committed as a git object before** the comparison compile, decline floor
  registered in advance. If (b) misses, the misses name the missing axis —
  iterate on new *probe* cells, never on the held-out TUs.
* **D2 — where does linkage-of-definition live in the bundle?** The model
  needs "out-of-line vs in-class" per definition and the IL carrier is
  unknown (the separator is refuted as that carrier). Resolver: minimal-pair
  IL diffs over lane A's existing cells (`v12`, out-of-line variants) — the
  captures are already on disk under `work/probes/`.
* **D3 — is the 451-TU wall a framing defect or genuinely absent records?**
  #151's precedent says one byte can be worth +213 ceiling. Resolver:
  §10.17's named probe — `?CanSelect@UIListProvider@@UBA_NH@Z` binds in 50
  TUs and is no-record in 3; byte-diff its `.gl` neighborhood across the
  boundary. One afternoon, and it re-prices R4 before R4 is scheduled.
* **D4 — the gate-anchored ceiling.** §10.15's recompute was taken and
  **declined** (§10.17): known on 6 TUs only, near-vacuously agreeing; the
  segment-count accessor on `IlBundle` (bundle.rs, owned elsewhere) is still
  owed. Until it lands, "25" and "at most 19" stay labelled `LO`-anchored.
* **D5 — how do explicit instantiations and dllexport reach the IL?** Roots
  the model cannot see are TUs it must refuse. Resolver: same minimal-pair
  IL-diff method as D2, cells already designed (lane A "not run" list).
* **D6 — the vtable rule's edge**: `dynamic_cast`/`typeid` forcing a vtable
  with no kept constructor — the one designed cell that could break the
  "ctor ⇒ vtable" formulation. One probe cell.
* **D7 — emission order at workload scale**: lane A's three order rules are
  probe-scale; verify against real objs' COMDAT sequence (the reader in
  `work/phase7plan-d/secscan.py` already parses the tables) before R6 trusts
  them.

## 6. What this plan does NOT propose, with reasons

* **No predictor over census features.** Refuted terminally (§9.18.5:
  best-case cell table = 1 TU, same as never-emit).
* **No separator-as-linkage model** (refuted here, 12.1 % vs ≥80 %
  registered) and **no "emit iff record exists"** (4.1 % of TUs, all
  trivial) and **no "strong-only" conservative sub-model** (exact on 35 TUs
  carrying 0.06 % of emitted functions — technically above my registered
  floor of 20, honestly worthless; declining my own registered fallback).
* **No spurious-COMDAT shaving.** The delta distribution is a cliff (median
  1,982); there is no incremental tail between "no model" and "model".
* **No raising of the 32-byte name-distance bound** (worth exactly zero,
  measured, and corrupts the binding past 96 — §9.20.3).
* **No disassembly of c2.dll.** The predicate yielded to black-box probing;
  the one prior candidate (the inline threshold) is off the critical path
  (§9.18.6). The clean-room claim stays blanket; no per-finding disclosure
  is incurred (§7).
* **No trusting the `.cod` beyond names.** Third strike recorded (§10.16):
  instruction bytes, displacements, and section order all diverge from the
  `/O1` obj. PROC/PUBLIC name sets only.
* **No scheduling Phase 6 as a standalone payoff phase** (§10.15: converts
  0 alone; it ships interleaved with the expression layer in R7), and **no
  "neutrality"/behavior-preserving classifier gates** anywhere — the obj
  byte-compare stays the sole judge.

## 7. Clean-room ledger

Every finding this plan adopts is **black-box** — observable outputs of the
toolchain under the README's existing blessing (§9.8): the 172-cell probe
grid (compile-and-observe), the C4505/C4514 warning channel, the `/FAsc`
listing PROC sets, `strings` over `c2.dll`/`clui.dll` (the diagnostic-string
category §9.8 already names; lane B's 128-entry flag-table harvest and the
resulting 31-accepted/0-C1007 sweep are the same category — string-table
text plus black-box flag probing, no instruction ever read), `.gl`/`.obj`
byte analysis of our own captures, the JamCRC fit (§10.16, output-fitted),
and the COMDAT Selection-byte observation. **Disassembly-derived constants
adopted: none.** The blanket clean-room claim stands unweakened. If a future
rung does adopt one, §9.8's rule applies: per-finding disclosure naming the
site, in the relevant docs file.

## 8. Instrument escalations (for the coordinator; measured, not built)

1. **Worktree binaries go stale silently.** The shared `target/release/c2rs`
   in this worktree predated HEAD's `emit-gate-segments*` keys — a lane
   reading them gets *silence, not zero* (§9.18.8's absence trap, in the
   instrument). Lane D caught it by rebuilding and diffing scans.
2. **The capture cache misses when `../dc3-decomp` moves.** Its HEAD moved
   `173eb73b → 13b583df` mid-session, forcing two cold 878-TU scans; the
   tree identity is in the cache key. Pin the dc3 rev for a session, or
   cross-lane byte-comparability is luck. (Control that passed: 0 of 871
   TUs differed across the two scans.)
3. **`strings` over `.gl` manufactures false negatives** — the `00|26`
   separator makes adjacent names concatenate. Anyone grepping `.gl` needs a
   separator-aware extractor (one exists at `work/wB/glnames.py`;
   known-answer-gated readers at `work/lane-c/readers.py`).

## 9. Pre-registration, scored — 4 hits, 2 misses, 1 refuted, 1 ungradeable

| # | registered | outcome | score |
|---|---|---|---|
| E1 | reference flips emission in ≥9/10 minimal pairs | **20/20** | **HIT** |
| E2 | ≥95 % of `00`-names emitted (floor 60 %) | **12.1 %** | **REFUTED** — the linkage-via-separator half of my hypothesis is dead; the declared inflationary bias cost exactly here |
| E3 | 55 % [30, 80] of emitted not explained by the `00`-rule | **7.7 %** | MISS low, favourable direction — recall was never the problem; precision (12.1 %) is |
| E4 | record-existence model ≈ 0 [0, 5 %] | 4.1 %, all trivial | **HIT** |
| E5 | strong-only TUs 150 [40, 400], floor 20 | **35 TUs / 0.06 % of functions** | MISS below interval; floor met on a technicality and declined in §6 |
| E6 | a skip-naming channel exists | C4505/C4514, precision 1.00 / recall 0.928, exact with closure | **HIT** |
| E7 | no single `.gl` field separates emitted from not | confirmed (collision test) | **HIT** |
| E8 | gate-anchored recount lands 27 [20, 40] | measurement **declined** by the owning lane (§10.17): known on 6 TUs, near-vacuous | **UNGRADEABLE** — carried as D4 |

The refutation is the lane's most useful output: it killed the cheap version
of Phase 7 (read linkage off the separator) before anyone built it, and the
fitted reachability predicate — which survived every cell designed to break
it — replaced it in the same session. The two misses both moved in the
deflationary direction against a declared inflationary bias, which is the
protocol working.
