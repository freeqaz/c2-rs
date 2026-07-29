# GAPS — the measured distance from here to real-TU coverage

Status: living worklist (written 2026-07-29; all numbers re-measured that day
with `c2rs gap` at HEAD — nothing below is quoted from memory). Companion to
[`ROADMAP.md`](ROADMAP.md): the roadmap says *what order*; this doc says *what
is blocking, how much of the real corpus each blocker holds hostage, what each
rung unlocks, and the exact commands that decide whether a rung is done*.

The goal restated: `c2rs gap` over the real dc3-decomp workload (878 TUs,
real `/O1 /Oi /EHsc` flags) reports a nonzero — then growing, then dominant —
**match** bucket, with zero mismatches, at port speed. Today that bucket is 0.

---

## 1. State of the world (the regression baseline)

What is proven today, stated precisely enough that a regression is visible.
Any run that degrades a number in this table is a regression, not noise.

| Claim | Number (2026-07-29) | Command that re-proves it |
|---|---|---|
| Standalone-c2 replay is byte-exact **including the COFF timestamp** on the whole capturable real workload | 871/871, 0 diverged (2026-07-20 full pass; 36/36 spot-check on the 2026-07-29 rescan) | `c2rs gap … --replay-every 1` |
| Standalone-c1 (front-end) replay is byte-exact | 25/25 fixtures | `c2rs replay-c1` |
| The port is byte-exact on its accepted class, fail-closed outside it | 13/25 perf fixtures Match, 12 NotImplemented, **0 mismatch** — and 0 mismatch across all 878 real TUs | `c2rs diff`, `c2rs perf`, `c2rs gap` |
| Port speed where it works | geomean ~1524× per obj (2.6–3.4 µs vs ~4.3 ms); ~897k objs/s at 32 threads vs ~3.1k for real c2 | `c2rs perf`, `c2rs perf-scale` |
| Test suite | 146/146 green with toolchain present | `cargo test --workspace --release` |
| IL codec round-trip | `encode(parse(b)) == b` on the full fixture spread, fail-closed | `il_roundtrip.rs` (in the suite) |

The replay-soundness row is the foundation: the *reference* side of every
differential is real c2 on real code, so every other number in this doc is
measured against truth, not against an approximation of it.

## 2. Where every real TU dies today (the funnel)

`c2rs gap`, 878 dc3 TUs, real flags, 38.4 s at `--jobs 16` (2026-07-29 —
identical to the 2026-07-20 baseline):

| Bucket | TUs | % | Meaning |
|---|---|---|---|
| match | 0 | 0.0 | byte-exact vs real c2 |
| mismatch | 0 | 0.0 | port emitted wrong bytes (correctness bug — must stay 0) |
| codegen-gap | 0 | 0.0 | IL decoded, `PortC2` refused |
| **vocab-gap** | **871** | **99.2** | `c2_il` cannot decode the bundle's functions |
| capture-fail | 7 | 0.8 | reference pipeline itself can't compile the TU here |

Scale of what sits behind the vocab-gap wall, measured from the scan JSONL:

- **902,730 functions** (by `.gl` mangled-name count) across the 871 TUs;
  median **705** functions per TU, max 4,137. Ten TUs have **0** functions
  (fully preprocessed-away bodies); 40 TUs have ≤10; 79 have ≤100;
  359 have ≤500.
- **664.5 MB of `.ex`** bytes total; roughly **94.5% of bundle bytes are
  opaque** to the codec (typed coverage ~5.5%, `IL_BUNDLE_MVP.md` §K2a).
- Decode is **all-or-nothing per TU** (`functions()` returns `None` if *any*
  function segment is outside the modeled grammar — or if the module has zero
  segments). With a median of 705 functions per TU, a TU-level `match`
  requires essentially *every* function class in that TU to be both decodable
  and codegen-complete. This has two consequences that shape everything below:
  1. The TU-grained scan **cannot rank** the W5–W14 ladder — 871 × "il
     function decode failed" is one undifferentiated bucket.
  2. The headline metric must move to **functions in-class** (a census
     numerator over 902,730) long before the TU-level match bucket can move.

## 3. Gap taxonomy

Every distinct blocker between here and real-TU coverage. Ordering within
this section is by dependency, not payoff; the ranked worklist is §4.

### GAP-0 — Measurement grain: no function-level census (P2b)

- **What**: the scan buckets TUs; it cannot say *which* IL feature blocks
  *how many* of the 902,730 real functions. The decoder fails closed at the
  first unknown byte but does not report which production/byte it died in.
- **Frequency**: blocks the *ranking* of everything in GAP-1/GAP-2 — 100% of
  prioritization decisions are currently intuition (the W5–W14 order in
  `ROADMAP.md` §G1 is an educated guess awaiting this histogram).
- **Unlocks**: the per-feature histogram over real functions = the widening
  order; the "N functions in-class today" headline numerator; per-rung
  measurable acceptance criteria for every W-step.
- **Depends on**: nothing. This is the first rung.
- **Difficulty**: small. The positive parser already tokenizes far enough to
  *reject honestly*; recording (production, offending byte, offset) at the
  rejection point and aggregating per function is instrumentation, not new
  grammar. Landmine: unknown opcodes must census as honest hex buckets
  (`expr-op-0xNN`), never as guessed names — the census *is* the measurement
  of the unknown vocabulary.

### GAP-1 — IL decode vocabulary (ROADMAP G2): the 99.2% wall

- **What**: `c2_il::func::parse_segment` accepts three body shapes (int
  add/sub/mul chains, void/int tail calls, one framed-call form —
  `IL_BUNDLE_MVP.md` has the full grammar). Everything else in `.ex` is
  undecoded: comparisons (`24`), shifts (`09`), bitwise (`0B`), ternary
  (`43 42`), branch/label tokens (`38`, `54 03/04`), casts (`2C`), memory
  (`30`/`32`), switch (`3B–3D`), float ops beyond the typed `Box::Volume`
  leaf vocabulary. Also undecoded: the `.ex` header/index region
  (`0x00–0x0A54`, the single largest opaque chunk), the FnHeader interior,
  most of `.gl`, and **all of `.sy` / `.in` / `.db`**.
- **Special case, measured**: ten real TUs have *zero* functions, and
  `functions()` rejects an empty module structurally (`segs.is_empty()` →
  `None`). These TUs need no new opcode at all — only an "empty module"
  acceptance plus empty-TU obj emission.
- **Frequency**: **871/878 TUs (99.2%)**, i.e. 100% of the 902,730 real
  functions — nothing reaches codegen. ~94.5% of bundle bytes opaque.
- **Unlocks**: decode alone moves TUs from `vocab-gap` to `codegen-gap` —
  which is the census becoming *exact* (the port's own NotImplemented reasons
  become the histogram) — and is a hard prerequisite for every match.
- **Depends on**: GAP-0 for ordering *within* this gap.
- **Difficulty**: the main body of work, but incremental by construction —
  the codec's typed-islands-over-opaque-spans model means each new token
  class lands round-trip-gated without destabilizing the rest. Landmines:
  never weaken the round-trip gate to land a class; token width must be
  detected structurally, not by a size heuristic (a past bug,
  `IL_BUNDLE_MVP.md`); `.sy` becomes load-bearing around W12–W14 when types
  stop being inferable from `.ex` alone.

### GAP-2 — Codegen classes (ROADMAP G1): the W-ladder proper

- **What**: `PortC2` lowers exactly the MVP class. The missing classes, with
  mechanisms per class, are the W5–W14 table in `ROADMAP.md` §G1: W5
  multi-scratch expressions, W6 compare→bool, W7 shifts/bitwise, W8 control
  flow, W9 div/mod, W10 general frames+locals, W11 generalized calls, W12
  memory/struct access, W13 float codegen, W14 data sections/globals — plus
  a census-driven long tail (switch tables, 64-bit carry chains, virtual
  calls, intrinsics).
- **Frequency**: **unmeasurable until GAP-0 lands** (today: 0 TUs in
  `codegen-gap`, because decode fails first). Staged fixture evidence exists
  for W6/W7/W8/W13 (`il_bool_materialization.cpp`, `add3.cpp`'s
  `select_max`/`shift_mask`, `il_call_return.cpp`, `mvp_fmul3.cpp`).
- **Unlocks**: this is the gap whose closure moves *functions in-class*, and
  eventually TUs into `match`.
- **Depends on**: GAP-1 per class (decode first), GAP-0 for order; W10/W11
  additionally on the W-UNW-1 label-counter model; W13b/W14 on new COFF
  section/reloc emission.
- **Difficulty + landmines** (all from this repo's own probes):
  - **Non-commutative hazard list** (`CODEGEN_PPC_MVP.md`): `subf` computes
    rB−rA (operands *reversed*); shifts have fixed order and signedness
    picks `sraw` vs `srw`; `cmpw`/`cmplw` direction is not swappable. A swap
    is a silent corruption differential testing exists to catch — every such
    encoder stays exact-pattern until probed.
  - **W-UNW-1**: `.pdata` label counters (`$M2545/…`) are a fixed seed for
    the first function but shift as preceding functions consume slots —
    resolved for single-function TUs, must be modeled per-function before
    W10/W11 touch multi-function TUs (median real TU: 705 functions).
  - `.pdata` carries a real reflected-CRC-32 checksum; new sections mean new
    CONST/DERIVED byte classification work per `OBJ_FORMAT_MVP.md`.
  - **Flag regime**: every codegen byte fact so far was characterized under
    `/Ox /GS-`; the real workload compiles `/O1 /Oi /EHsc`. Divergences
    (e.g. different inlining/EH scaffolding for otherwise in-class bodies)
    are unprobed — expect the first real-flag classes to need fresh
    CONST/DERIVED passes even where fixtures already match.

### GAP-3 — Workload manifest: the 7 capture-fails

- **What**: 3× C1083, 2× C1189, 2× C2084 — all `synth_xbox`/`soundtouch`
  files the real 360 build excludes (x86-only `#error` guards) or builds
  with per-target flags. A harness/manifest refinement, not a port gap.
- **Frequency**: 7/878 TUs (0.8%).
- **Unlocks**: an honest denominator (878 → 871 measurable, or per-TU flag
  overrides in `gen_dc3_workload.sh`).
- **Depends on**: nothing. **Difficulty**: trivial; do it whenever the noise
  annoys.

### GAP-4 — Architecture: shape-matcher → real lowering pipeline (ROADMAP G4)

- **What**: codegen is a positive shape-matcher with an intentionally empty
  `passes/` tree. W8 (first CFG) and W10 (frames) force a block/instruction
  IR; COLOR register-order modeling becomes real at W5/W10.
- **Frequency**: not corpus-measurable — it is a scaling blocker for GAP-2,
  not a corpus bucket.
- **Depends on / unlocks**: restructure *at* W8, not before (per ROADMAP §G4
  — keep widening the matcher until the CFG step forces the IR, keeping
  every differential gate green through the restructure).
- **Difficulty**: the one genuinely architectural step on the ladder; the
  risk is not the IR but keeping 0-mismatch through the rewrite — land it as
  a refactor gated by the full fixture + gap-scan suite, no widening in the
  same change.

### GAP-5 — Front-end port (ROADMAP G3): not on the critical path to `match`

- **What**: `c1-core` (source→bundle) so that composition (`P3 compose`,
  source→obj fully in-process) needs no Microsoft binary. Replay proof
  (P-F0.1) landed; characterization (P-F0.2) and the crate (P-F1) have not.
- **Frequency**: blocks **0** of the gap-scan match bucket — the scan
  captures IL with the real front end. It gates only the composition
  milestone and the >2.4× downstream-speedup regime (§5).
- **Depends on**: backend class definitions (it widens in lockstep).
- **Difficulty**: medium, with one named risk: `.db` line-record semantics
  (smallest, least understood file); any preprocessor use leaves the class —
  the recognizer must fail closed.

## 4. The ladder — ordered worklist with per-rung acceptance

Rungs below the W-numbering are the instruments; W-rungs are the port. Every
rung ends with the same three gates (spelled out in §6): **fixture gate**
(byte-exact positives, NotImplemented negatives, suite green, perf
re-confirmed), **census gate** (the function-level in-class numerator rises
by the rung's measured population), **scan gate** (`c2rs gap` re-run, JSONL
diffed against the previous baseline — buckets move only in the good
direction). A rung whose fixture gate passes but whose census/scan gate
doesn't move is **not done** — it modeled a shape the real corpus doesn't
contain, which is a finding, not progress.

| Rung | Work | Passes when (measurable) |
|---|---|---|
| **R0 = P2b** | Function-level census: record (production, byte, offset) at each decode rejection; aggregate per-feature histogram over the 871-TU workload | The scan (or a census subcommand) prints a per-feature histogram whose counts sum to ~902,730; the W5–W14 order below is re-ranked from it and this doc updated with the measured populations |
| **R1** | Empty-module acceptance + empty-TU obj emission (the ten 0-function TUs) | Gap-scan **match ≥ 10/878** — the first nonzero match bucket, and the downstream tripwire (§5) trips |
| **W5** | Multi-scratch expressions (COLOR order past r11) | Fixture `(a+b)*(c+d)` exact, depth-5 negative rejected; census: multi-scratch bucket count moves to in-class |
| **W6** | Compare→bool materialization | `il_bool_materialization.cpp` shapes exact; census compare-bucket (opcode `24`) moves |
| **W7** | Shifts + bitwise + strength reduction | `shift_mask` exact; hazard-listed encoders land exact-pattern, opt-in; census `09`/`0B` buckets move |
| **W8** | Control flow (first CFG; GAP-4 restructure lands here) | `select_max` + conditional shapes exact; restructure merged with **0 mismatch** on fixtures *and* the full 878-TU scan; census branch-token bucket moves |
| **W9** | Div/mod (incl. const-divisor multiply-high) | census div bucket moves |
| **W10** | General frames + locals + per-function `.pdata` counters (W-UNW-1) | multi-function TUs with frames decode+emit; first TUs where *every* function is in-class flip to `match` — target population: the 40 TUs with ≤10 functions |
| **W11** | Calls generalized (args r3–r10, stack spill, multiple calls) | census call buckets move; match bucket starts climbing the ≤100-fn TU population (79 TUs) |
| **W12** | Memory / struct access (`.sy` becomes load-bearing) | census memory buckets (`30`/`32`) move |
| **W13** | Float codegen (13a params, 13b constants→`.rdata`) | `mvp_fmul3.cpp` exact; census float buckets move |
| **W14** | Data sections / globals (`ADDR32` relocs) | census global buckets move; match bucket now tracks whole subsystems |
| **P-F0.2→P-F2, P3** | Front-end track + compose (parallel, off the match-bucket critical path) | Grade 1 `PortC1 == captured` per file; Grade 2 `PortC2(PortC1(src)) == pipeline obj`; `compose` timed |

Expect R0 to re-rank W5–W14 — the order above is the roadmap's estimate, and
the whole point of R0 is to replace it with measured populations. Re-rank
freely; the acceptance-gate structure is what's fixed, not the order.

Session discipline: one rung (or one census-bucket slice of a rung) per
session; re-run the scan at session end; keep every JSONL
(`work/dc3-workload/scan-YYYYMMDD.jsonl`, gitignored) so coverage is a
monotone, diffable series; update the baseline table in `ROADMAP.md` §G5 and
the populations here when they move.

## 5. The payoff contract — what downstream integration exists and when this
work starts paying

The consuming project (decomp-synth, the guided-search decompilation engine
this port was built to accelerate) assessed c2-rs for its frontier scoring
loop on 2026-07-29. Condensed verdict, so the payoff line is visible from
inside this repo:

- **Today: NO-GO for every frontier-scoring use.** The port covers 0% of the
  real corpus (§2), and every speed path through it is bounded by the c1xx
  front-end cost the scoring loop must still pay per *source* candidate
  (compiles are ~245 ms on PCH units, c1xx ≈ 45 ms of it; even a 100%-
  coverage backend caps the funnel at ≲2.4× without the front-end port).
- **The only doctrinally-legal integration shape** (recorded so the target
  is stable): a **reject-only, fail-closed pre-filter**. The consumer treats
  the port's three-way verdict as: `NotImplemented` → full real compile (no
  saving, no risk); port-emitted **match** → full real compile *anyway*
  (every accepted result is still witnessed by the real toolchain — the port
  never mints a solve); only a port-emitted **mismatch** may skip a real
  compile, and skips are continuously audited by real-compiling 1-in-N of
  them. The port is never the judge; it is a fast way to spend less time
  proving negatives.
- **The byte-identity bar the consumer holds this repo to**: replay
  raw-identical **including the timestamp**, re-proven per-TU before any
  IL-derived result is used; the port byte-exact (timestamp-zeroed) per
  accepted class, with the mismatch bucket at a hard 0.
- **The tripwire that reopens downstream use**: the gap-scan **match bucket
  going nonzero on real TUs** (R1 trips it). That is the exact, mechanical
  signal that the downstream assessment re-opens — the owner's work here
  starts paying the moment §4-R1's gate passes, and pays more with every TU
  the match bucket gains.
- **Already-live secondary payoff**: c2-only replay as a scorer backend for
  any IL-space search lane is a GO-when-relevant (~days of wiring) — the
  871/871 replay-soundness proof is the asset, and it is done. Keeping it
  green (the `--replay-every` lane) protects banked value.

## 6. How to verify honestly

The commands that constitute "this rung is done". Run from the repo root;
toolchain via `scripts/fetch_compilers.sh`, wibo on `PATH` or sibling
`../wibo`; the workload inputs via `scripts/gen_dc3_workload.sh` (needs a
sibling `../dc3-decomp` checkout).

```sh
# 1. Fixture parity + fail-closed boundary + suite + perf (the fixture gate)
cargo test --workspace --release
cargo run --release -p c2-harness --bin c2rs -- diff
cargo run --release -p c2-harness --bin c2rs -- perf

# 2. The real-workload gap scan (the census + scan gates; ~40 s at -j16)
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --jsonl work/dc3-workload/scan-$(date +%Y%m%d).jsonl \
  --replay-every 25 --jobs 16

# 3. Replay soundness at full strength (periodically, and before trusting
#    any IL-derived artifact): every TU, byte-exact including timestamp
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --replay-every 1 --jobs 16
```

The rules that keep the numbers honest:

- **Byte-exact means byte-exact.** Replay: raw-identical including the COFF
  timestamp. Port: identical with only the 4-byte timestamp zeroed. No
  fuzzy thresholds anywhere; real c2 under wibo is the sole judge.
- **`mismatch` is an alarm, not a gap.** Any nonzero mismatch bucket — on
  fixtures or the workload — is a correctness bug that outranks all widening
  work. The port's value downstream depends on this bucket staying 0.
- **A fixture pass without a real-TU improvement is not progress** — the
  census/scan gates exist precisely because fixtures sample the grammar we
  *guessed* matters; only the 878-TU scan measures the grammar that *does*.
- **Diff scans, don't overwrite them.** Coverage must be monotone
  scan-over-scan; the dated JSONLs are the longitudinal record (per-TU
  `class`/`reason` diffing catches a rung that fixes one bucket by breaking
  another).
- **Measure committed code.** Build from a clean tree before a scan you
  intend to record — a binary carrying uncommitted WIP produces numbers no
  future scan can be diffed against.
