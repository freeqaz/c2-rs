# GAPS — the measured distance from here to real-TU coverage

Status: living worklist (written 2026-07-29, revised the same day for the P2b
function-level census and the variable-token-width finding; all numbers
re-measured with `c2rs gap` / `c2rs census` at HEAD — nothing below is quoted
from memory). Companion to
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
| Standalone-c2 replay is byte-exact **including the COFF timestamp** on the whole capturable real workload | 871/871, 0 diverged — **re-proven at full strength 2026-07-29** on the post-token-fix code (43.4 s at `--jobs 16`), matching the 2026-07-20 full pass | `c2rs gap … --replay-every 1` |
| Standalone-c1 (front-end) replay is byte-exact | 25/25 fixtures | `c2rs replay-c1` |
| The port is byte-exact on its accepted class, fail-closed outside it | 13/25 perf fixtures Match, 12 NotImplemented, **0 mismatch** — and 0 mismatch across all 878 real TUs | `c2rs diff`, `c2rs perf`, `c2rs gap` |
| **Real-corpus coverage, per function** (the headline numerator, P2b) | **7,114 / 2,462,571 functions in class (0.29%)** | `c2rs gap …` (FUNCTION CENSUS block), `c2rs census <cpp>` |
| Port speed where it works | geomean ~1524× per obj (2.6–3.4 µs vs ~4.3 ms); ~897k objs/s at 32 threads vs ~3.1k for real c2 | `c2rs perf`, `c2rs perf-scale` |
| Test suite | 146/146 green with toolchain present | `cargo test --workspace --release` |
| IL codec round-trip | `encode(parse(b)) == b` on the full fixture spread, fail-closed | `il_roundtrip.rs` (in the suite) |

The replay-soundness row is the foundation: the *reference* side of every
differential is real c2 on real code, so every other number in this doc is
measured against truth, not against an approximation of it.

## 2. Where every real TU dies today (the funnel)

`c2rs gap`, 878 dc3 TUs, real flags, 37.3 s at `--jobs 16` (2026-07-29 — TU
buckets identical to the 2026-07-20 baseline; the function census below is
new):

| Bucket | TUs | % | Meaning |
|---|---|---|---|
| match | 0 | 0.0 | byte-exact vs real c2 |
| mismatch | 0 | 0.0 | port emitted wrong bytes (correctness bug — must stay 0) |
| codegen-gap | 0 | 0.0 | IL decoded, `PortC2` refused |
| **vocab-gap** | **871** | **99.2** | `c2_il` cannot decode the bundle's functions |
| capture-fail | 7 | 0.8 | reference pipeline itself can't compile the TU here |

Scale of what sits behind the vocab-gap wall, measured from the scan JSONL
and the P2b census:

- **2,462,571 functions** across the 871 TUs, of which **7,114 (0.29%) are in
  class today**. Ten TUs have **0** functions (fully preprocessed-away
  bodies); 40 TUs have ≤10; 79 have ≤100; 359 have ≤500 (`.gl`-name-derived
  per-TU distribution, retained for its *shape*; see the denominator warning
  below before quoting its absolute numbers).
- **664.5 MB of `.ex`** bytes total; roughly **94.5% of bundle bytes are
  opaque** to the codec (typed coverage ~5.5%, `IL_BUNDLE_MVP.md` §K2a).
- Decode is **all-or-nothing per TU** (`functions()` returns `None` if *any*
  function segment is outside the modeled grammar — or if the module has zero
  segments). A TU-level `match` therefore requires essentially *every*
  function class in that TU to be both decodable and codegen-complete. Two
  consequences that shape everything below:
  1. The TU-grained scan **cannot rank** the W5–W14 ladder — 871 × "il
     function decode failed" is one undifferentiated bucket. This is what the
     P2b function census (GAP-0, now landed) exists to fix.
  2. The headline metric is **functions in-class** (7,114 / 2,462,571) and
     will stay so long before the TU-level match bucket can move.

> ### The denominator is 2,462,571 — never 902,730
>
> An earlier revision of this document put the corpus at **902,730 functions**
> by counting `.gl` mangled names. **That is not a function count** and must
> not be re-derived: `mangled_names` accepts only `?…@@…` forms, and `.gl`
> also lists externals, so it both under- and over-counts relative to bodies.
> Measured on one real TU (`system/world/Dir.cpp`, 1.5 MB `.ex`):
>
> | Instrument | Count |
> |---|---:|
> | `.gl` mangled names | 2,153 |
> | `4F 1F` fn-start markers | 5,340 |
> | **`LO` body markers (`4C 4F 11`)** | **5,239** |
> | function tails (`4F 12 47 54 01 54 00`) | 5,243 |
>
> The last two agree to 0.08%; the two-byte `4F 1F` scan is ~2% high because
> that pair also occurs inside token and varint payloads. The census therefore
> anchors on the `LO` body marker (`func::split_function_bodies`, which starts
> each segment at the `4F 1F` immediately preceding its `LO` so the formals
> region stays in-segment, and never reuses a start — a collision blocks the
> later body honestly at `formals-marker` rather than silently merging two
> functions).

### 2b. Where every real *function* dies (the P2b census)

The instrument: `c2rs census <cpp> [--flags-file F --cwd D] [--keep-il DIR]`
for one TU, and the `FUNCTION CENSUS` + `blocking features` block the
`c2rs gap` report now prints scan-wide. Each function segment goes through the
*same* positive parser as the port and keeps its **first** blocking
`(production, byte, offset)`.

**7,114 / 2,462,571 functions in class (0.29%)**; 1,237 distinct blocking
features. (Same instrument before the variable-token-width fix in GAP-1:
4,154 / 2,462,571 = 0.17%.) Top 20, percentages of *blocked* functions:

| Functions | % | Feature |
|---:|---:|---|
| 363,684 | 14.8 | `call-token-0xB9` |
| 235,886 | 9.6 | `call-anchor-0x00` |
| 166,483 | 6.8 | `expr-op-0x40` |
| 144,276 | 5.9 | `call-token-0x33` |
| 107,253 | 4.4 | `body-0x3A` |
| 80,284 | 3.3 | `call-token-0x26` |
| 70,078 | 2.9 | `body-0x53` |
| 67,012 | 2.7 | `expr-call-in-expr` |
| 43,269 | 1.8 | `call-anchor-0x08` |
| 41,878 | 1.7 | `expr-load-type-864383` |
| 34,573 | 1.4 | `expr-load-type-864275` |
| 28,487 | 1.2 | `body-0x9B` |
| 26,666 | 1.1 | `body-0x4F` |
| 24,600 | 1.0 | `call-anchor-0x20` |
| 22,947 | 0.9 | `expr-lit-type-821230` |
| 15,782 | 0.6 | `body-0xB3` |
| 15,480 | 0.6 | `body-0xAD` |
| 14,594 | 0.6 | `body-0x29` |
| 13,248 | 0.5 | `body-0x67` |
| 12,405 | 0.5 | `body-0x9A` |

…and **1,217 more distinct features** in the tail.

Reading the bucket names (`func::Block::feature`) — `<production>-0xNN` means
the parse was inside that grammar production and could not consume byte `NN`:

- `call-token-*` — the byte where the fixed 10-byte CALL token
  (`BD <3-byte ret type> 00 80 01 10 00 00`) was expected after a `26 <tok>`
  reference.
- `call-anchor-*` — the 6-byte anchor `00 80 01 10 00 00` did not match.
- `body-*` — the byte opening the function body, where only a call ref (`26`),
  LOAD (`B9`) or literal (`33`) is modeled.
- `expr-*` — inside the operand stream. `expr-*-type-NNNNNN` reports the whole
  3-byte inline operand type, because the triple *is* the feature (int vs
  unsigned vs float vs pointer); a bare byte would bucket them all together.

**What this implies for the ladder — INFERENCE, not measurement.** Grouping
the top-20 rows by production: call-shaped blocks (`call-token-*` 24.0% +
`call-anchor-*` 12.4% + `expr-call-in-expr` 2.7%) are **~39% of blocked
functions**, and statement/body-shaped blocks (`body-*`, led by `0x3A` — a
body opening with ASSIGN — and `0x53`, a further statement marker) are
**~12%**. Both are lower bounds: further `call-*` and `body-*` buckets sit in
the 1,217-row tail. Comparisons and shifts — the currently-scheduled **W6 and
W7** — do **not** appear in the top 20 at all. So the measured demand points
at **W11** (generalized calls) and **W10** (frames/locals/statements), and the
ROADMAP's W5→W6→W7→W8 order is not demand-driven.

Two caveats keep this provisional rather than a decision:

1. The exact meaning of several top buckets is **not characterized**. Most
   importantly `call-token-0xB9` (the largest single bucket) means a LOAD
   stands where the CALL token was expected — so `26 <tok>` is evidently
   **not** always a call prefix, and a large share of the "call-shaped" group
   may not be calls at all.
2. A blocking feature is the *first* thing that stopped the parse, not the
   only thing missing in that function. Fixing the top bucket moves those
   functions to their *next* blocker, not necessarily into class.

Characterize the `26`/`B9` grammar first; then commit the re-rank.

## 3. Gap taxonomy

Every distinct blocker between here and real-TU coverage. Ordering within
this section is by dependency, not payoff; the ranked worklist is §4.

### GAP-0 — Measurement grain: function-level census (P2b) — **CLOSED 2026-07-29**

- **What it was**: the scan bucketed TUs only; it could not say *which* IL
  feature blocks *how many* real functions. The decoder failed closed at the
  first unknown byte without reporting which production/byte it died in.
- **Closed by**: commits 63b1ad1 (`c2-il`: `Block` / `FnVerdict` / `FnCensus`
  / `IlBundle::function_census`, keyed on the first blocking
  `(production, byte, offset)`) and ec401a5 (`c2rs census` subcommand + the
  scan-wide census and histogram in the `gap` report).
- **Measured result**: **7,114 / 2,462,571 functions in class (0.29%)**,
  1,237 distinct blocking features — §2b. The denominator is anchored on the
  `LO` body marker, **not** `.gl` mangled names (see the boxed warning in §2);
  the previously published ~902,730 was a `.gl` name count and is wrong.
- **Held**: unknown opcodes census as honest hex buckets (`expr-op-0xNN`,
  `body-0xNN`, …), never guessed names — the census *is* the measurement of
  the unknown vocabulary. The census is diagnostic only: acceptance is
  unchanged and the emitter never consults it.
- **Residual**: the histogram ranks *first blockers*, which is not the same as
  ranking *rungs* — see the two caveats at the end of §2b. Characterizing the
  `26`/`B9` grammar is the follow-on that turns the histogram into a schedule.

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
- **Variable token width (measured 2026-07-29, commit 40f767d)**: IL tokens
  are **2 *or* 4 bytes, per token — not a per-file constant**. In one capture
  of `system/world/Dir.cpp` the `4F 02` module marker appears both as
  `4f 02 e3 09` and as `4f 02 a4 96 03 00`. The discriminator is **bit 7 of
  the token's second byte**: clear → 2 bytes, set → two more follow. Verified
  by applying the rule at every `B9` LOAD site: 21,443 sites land on a valid
  3-byte operand type. `func::read_token_var` implements this; the old
  whole-file `detect_token_width` is wrong for real TUs and the function
  parser no longer consults it.
  - **This fabricated census buckets.** A 2-byte read of a 4-byte token leaves
    the parse standing on the token's own tail bytes, which look like unknown
    opcodes. The `call-token-0x01…0x05` and `expr-load-type-0N00A6` families
    were misalignment, not vocabulary; both vanished, and the in-class
    numerator moved 4,154 → 7,114 on the identical instrument. Treat any
    "new opcode" found by a fixed-width reader as suspect until re-checked.
  - **Outstanding**: `crates/c2-il/src/codec.rs` still reads a fixed 2-byte
    token (`tok16`) and carries the same latent defect on real TUs. It is
    round-trip gated, so it fails **closed** (falls back to an opaque span)
    rather than mis-decoding — no correctness exposure — but it caps typed
    coverage on real bundles. Port it to the `read_token_var` rule, gate
    unchanged, before pointing the codec at the real workload.
- **Frequency**: **871/878 TUs (99.2%)**, i.e. 99.71% of the 2,462,571 real
  functions (§2b) — essentially nothing reaches codegen. ~94.5% of bundle
  bytes opaque.
- **Unlocks**: decode alone moves TUs from `vocab-gap` to `codegen-gap` —
  which is the census becoming *exact* (the port's own NotImplemented reasons
  become the histogram) — and is a hard prerequisite for every match.
- **Depends on**: GAP-0 (closed) for ordering *within* this gap; the
  histogram in §2b is that ordering.
- **Difficulty**: the main body of work, but incremental by construction —
  the codec's typed-islands-over-opaque-spans model means each new token
  class lands round-trip-gated without destabilizing the rest. Landmines:
  never weaken the round-trip gate to land a class; token width is
  **per-token**, read it structurally (bit 7 of byte 1) and never from a
  per-file heuristic — misalignment does not look like an error, it looks
  like new vocabulary; `.sy` becomes load-bearing around W12–W14 when types
  stop being inferable from `.ex` alone.

### GAP-2 — Codegen classes (ROADMAP G1): the W-ladder proper

- **What**: `PortC2` lowers exactly the MVP class. The missing classes, with
  mechanisms per class, are the W5–W14 table in `ROADMAP.md` §G1: W5
  multi-scratch expressions, W6 compare→bool, W7 shifts/bitwise, W8 control
  flow, W9 div/mod, W10 general frames+locals, W11 generalized calls, W12
  memory/struct access, W13 float codegen, W14 data sections/globals — plus
  a census-driven long tail (switch tables, 64-bit carry chains, virtual
  calls, intrinsics).
- **Frequency**: still 0 TUs in `codegen-gap` (decode fails first), so the
  per-class codegen demand is not directly measured. What *is* measured is
  the decode-side proxy in §2b: call-shaped blockers ~39% of blocked
  functions and body/statement-shaped ~12% (top-20 rows, lower bounds),
  while comparisons and shifts are absent from the top 20 — i.e. the demand
  visible today points at **W11** and **W10**, not W6/W7. Treat that as
  inference (see the §2b caveats), not as a codegen measurement. Staged
  fixture evidence exists for W6/W7/W8/W13
  (`il_bool_materialization.cpp`, `add3.cpp`'s `select_max`/`shift_mask`,
  `il_call_return.cpp`, `mvp_fmul3.cpp`) — fixtures sample the grammar we
  guessed matters; the census measures the grammar that does.
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
    W10/W11 touch multi-function TUs (a real TU averages ~2,800 functions
    over the corrected denominator — §2).
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
| ~~**R0 = P2b**~~ **DONE 2026-07-29** | Function-level census: record (production, byte, offset) at each decode rejection; aggregate per-feature histogram over the 871-TU workload | **Passed**: `c2rs gap` and `c2rs census` print a per-feature histogram whose in-class + blocked counts sum to the **2,462,571**-function denominator (`LO`-marker anchored — **not** the ~902,730 `.gl` name count an earlier revision of this row demanded; see §2). Measured: 7,114 in class (0.29%), 1,237 blocking features, §2b. The W-order below is re-ranked *provisionally* from it — pending the `26`/`B9` characterization |
| **R0b** | Characterize the `26 <tok>` / `B9` grammar behind `call-token-0xB9` (14.8%) and `call-anchor-0x00` (9.6%) — the largest buckets, and the ones whose meaning the re-rank depends on | Those buckets are attributable to a named rung (or split into ones that are), and the §2b grouping is either confirmed or replaced; the ladder order below is then committed to rather than provisional |
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

**R0 has run, and it does re-rank W5–W14 — the table order above is now known
to be the roadmap's pre-census estimate.** Measured demand (§2b) puts W11
(calls) and W10 (frames/locals/statements) at the top and leaves W6/W7 out of
the top 20 entirely. The re-ranking is *inference* from a first-blocker
histogram, so R0b (characterizing `26`/`B9`) is scheduled before the order is
rewritten. Re-rank freely once it lands; the acceptance-gate structure is what
is fixed, not the order.

Note also that the census numerator is now the per-rung yardstick in a way it
was not before: R1's ten 0-function TUs still trip the match tripwire, but
every W-rung is judged by how many of the 2,455,457 blocked functions it moves
— and by which bucket they move *to*, since a function's first blocker is
rarely its only one.

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

- **Today: NO-GO for every frontier-scoring use.** The port covers 0 TUs of
  the real corpus — 0.29% of its functions (§2, §2b, measured after P2b; the
  assessment predates the census but its input, a zero match bucket, is
  unchanged) — and every speed path through it is bounded by the c1xx
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

# 2. The real-workload gap scan (the census + scan gates; ~37 s at -j16).
#    Prints the TU buckets, the FUNCTION CENSUS numerator, and the top-20
#    blocking-feature histogram.
cargo run --release -p c2-harness --bin c2rs -- gap \
  --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
  --cwd ../dc3-decomp --jsonl work/dc3-workload/scan-$(date +%Y%m%d).jsonl \
  --replay-every 25 --jobs 16

# 2b. Single-TU census while developing a widening step: run it before and
#     after and watch named functions move from a blocking feature to a shape.
#     --keep-il drops the captured bundle in a (gitignored) scratch dir for
#     grammar work.
cargo run --release -p c2-harness --bin c2rs -- census system/world/Dir.cpp \
  --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp \
  --keep-il work/il-scratch

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
- **The denominator has one definition**: functions counted at the `LO` body
  marker (`4C 4F 11`), 2,462,571 over this workload. Never re-derive it from
  `.gl` mangled names (~902,730, wrong) or from raw `4F 1F` scans (~2% high).
  A coverage percentage is only comparable to a previous one if both used it.
- **A new census bucket may be a parser bug, not a feature.** Misaligned
  reads look exactly like unknown vocabulary — the variable-token-width fix
  (GAP-1) deleted two whole bucket families that were pure misalignment.
  Before scheduling work against a bucket, dump the bytes at the recorded
  offset (`c2rs census --keep-il`) and confirm the parse arrived there aligned.
