# stageoracle — c2's intermediate state CAN be observed; the register allocator's output is not in the tuple

    Tag:       stageoracle
    Slug:      stageoracle
    Date:      2026-08-20
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization: can real c2's INTERMEDIATE per-function
               state be observed reliably, and without changing the obj?
    Census:    +0 — no crates/ behaviour, no fixture, no emit rule changes
    Record:    this file; prereg at `work/oracle/PREREG.md`, frozen as the
               lane's first commit; logs under `work/oracle/`

---

## 0. What already shipped, so nothing below is re-shipped

Board **#3314**'s rule first: *"unserved in `docs/` is not unserved in the
repo."* Half of this lane's brief — *"one end-to-end instrumented TU per
admitted family"* — was **already shipping** as the `/FAsc` + `/QXSTALLS`
narration seam: `Toolchain::capture_listing_with`
(`crates/c2-reference/src/lib.rs:970`), `crates/c2-reference/src/cod.rs`,
`crates/c2-harness/src/listing.rs`, `c2rs listing` / `c2rs listing-scan`, and
three standing tests in `crates/c2-reference/tests/listing.rs` (boards
#132/#134/#136). c2 already narrates its own output: label counter, section
order, EH layout, relocations by name, per-instruction issue cycles.

**What that seam is not, and what this lane therefore built.** The `.cod` is an
**end-state** observation — after all four scheduler runs, after COLOR, after
lowering — so it cannot separate COLOR's output from the scheduler's, which is
the entire content of migration step 5. `/QXSTALLS`'s issue cycles are worse
than merely late: they come from **K4** (`0x10c1ce93`), which builds its own
whole-function DAG read-only and tears it down
(`docs/whitebox/WB_DAGCLIENTS_FINDINGS.md` §4.4), so they are a
*re-derivation*, not the schedule the region-bounded scheduler produced.

The unbuilt half is **observation of c2's tuple list BETWEEN passes**, and that
is what landed.

## 1. Verdict: GO, with one boundary named

**Real c2's intermediate per-function state CAN be observed reliably.**
Neutrally (the obj does not move), deterministically (one digest over ten
runs), canonically (across working directories and output paths), and with a
non-empty, cross-derived payload.

**And the boundary the lane found is worth more than the green:** the observable
is **blind to the register allocator**. It moves across the scheduler and
across lowering, on every function, and is byte-identical across COLOR on every
function — with a 128-byte raw window per tuple saying COLOR writes *nothing*
inside the tuple record. So a ported COLOR cannot be graded against this
snapshot, and the next lane knows where not to look.

## 2. What landed

| file | what |
|---|---|
| `c2host/stagetap.h` / `c2host/stagetap.c` | the site table, fail-closed arming, one shared thunk, the bounded tuple walk, the phase tagger, the raw window |
| `c2host/c2host.c` | two lines: `tap_arm(h, fn)` before the pass, `tap_report()` after |
| `crates/c2-reference/src/lib.rs` | `build_host_stub(src: &Path)` → `(srcs: &[&Path])`; `ensure_c2host` rebuilds when **any** source is newer |
| `crates/c2-reference/src/stage.rs` | `Toolchain::replay_tapped{,_with,_raw}`, `TapReport`, `StageBlock`, `canonical_bytes`, `digest`, `color_pair` |
| `crates/c2-reference/tests/stage.rs` | six named controls |
| `crates/c2-harness/src/cli/stage.rs` | `c2rs stage {counts,snap,determinism,neutrality}` and the `stage-*` gap-metric family |
| `docs/whitebox/DISCLOSURE.md` | `W-STAGETAP-1` (the seven call sites + the slide anchor), `W-STAGETAP-2` (the tuple layout) |

### The mechanism, in one paragraph

Every boundary wanted is already a 5-byte `e8 rel32` **call site**, and the flat
export gives each address. Patching the *site* rather than the callee needs no
instruction-length decoder, no prologue save/restore, no unpatch window, and
never touches the callee — which matters because `0x10be6382` has two callers
and only a site-level detour can tell them apart. Arming is fail-closed: a site
is written only if the byte there is still `0xE8` **and** the decoded target
equals the recorded target **plus the measured load slide**.

## 3. Results — every number derived from a log under `work/oracle/`

### G1 — NEUTRALITY (required zero; the sole judge's own criterion)

| population | graded | `stage-tap-obj-differs` | errors | sites armed / refused |
|---|---:|---:|---:|---|
| `fixtures/cpp/*.cpp` | **384** | **0** | 2 | 2688 / 0 |
| the 26 matched workload TUs, at the workload's own flags | **26** | **0** | 0 | 182 / 0 |

Measured with the **payload ON**, never extrapolated from a counts-only run —
the payload is the half that touches c2's own memory (plan unknown 8).
Second derivation (#3288): re-counting the log's own per-fixture verdict lines
gives `SAME 384 / DIFFERS 0 / ERR 2`, equal to the accumulated counters.
The two errors are `wmain_no_return{,_neg}.cpp` and they fail on the
**disarmed** leg — the untapped replay itself produces no obj — so they are
pre-existing and are classified as errors rather than folded into either side
of the required zero. Logs: `work/oracle/neutrality_all.log`,
`work/oracle/neutrality_matched26.log`.

The comparison is made through **one function**: `replay_tapped` with an empty
tap list *is* the disarmed leg. Each leg asserts its own state first — the
disarmed leg must print no tap line, the armed leg must report `armed_ok` —
because "identical" between two legs that were both disarmed is not a
measurement.

### G2 / G2b — DETERMINISM and CANONICALITY

`work/oracle/determinism.log`, payload on, 4 fixtures × (5 same-config runs +
5 runs from a fresh working directory with a different `/Fo` path):

    stage-snap-runs 10 · stage-snap-distinct-max 1 · stage-snap-unstable-tus 0
    stage-snap-graded 4 · stage-snap-empty-payload 0
    add3 309 tuples · il_call_perm 329 · il_call_return 662 · mvp_add3 36

**One digest per fixture over all ten runs.** It held at the *first* schema.
The schema forbids addresses, pointers, paths, PIDs, timestamps and allocation
counts by construction — the slide is recorded as `0`-or-`nonzero`, never as a
value, and the raw window is excluded from `canonical_bytes()` entirely.

### G3 — DISCRIMINATION (the null control)

`scheduler_taps_are_silent_at_Od_and_loud_at_O1`, on `il_call_perm.cpp`: the
four optimizer-gated sites fire **0** times at `/Od` and **> 0** at `/O1`.
Grounded in the bytes rather than hoped for: `0x10b7dc83`, `0x10b7dcc2` and
`0x10b7dd01` are each `cmp DWORD PTR ds:0x10c2e2fc,edi` with `edi == 0`. Had
the two counts come out equal, the instrument would have been measuring itself
— the fifth entry in this repo's *"ranking instruments measure themselves"*
family.

### G5 — CONTENT, cross-derived three ways

`il_call_perm.cpp`, `/O1 /Oi /EHsc /GS- /c`:

    sched1 7 · globregs 7 · sched2 7 · color 7 · sched3 7 · sched0 7
    region 56 · tuples 329 · walk refusals 0

**And zero walk refusals over the whole campaign** — no `walk-overrun`,
`walk-span`, `walk-implausible-*` or `arena-full` line appears in either
neutrality log, over 410 objs including `jsonwriter.cpp` at 5,491 tuple rows.
That matters because a truncated payload would make every tuple count a floor
rather than a measurement, and the refusal lines are kept in a list of their
own precisely so truncation can never be read as a terminus.

| derivation | how it is built | answer |
|---|---|---|
| the tap | patched call sites **inside c2's code**, counted in `c2host` | 7 |
| the listing | c2's own `/FAsc` writer | `7 PROC` |
| the obj | the COFF section table | `7 .text COMDAT` |

Three paths with no shared step after c2's front end. This also **re-derives
`P_DAG.md` §1's "four scheduler runs per function"** as an equality between
four separately-patched sites rather than as a reading.

The opcodes are self-consistent with the region finder's own control flow:
`0x30f` appears at category `0x17`, which is exactly the pair `0x10be5d8b`
tests, and the categories observed (`0x0d 0x0f 0x12 0x15 0x17 0x19 0x1a`) are
the set that function branches on.

### The finding: COLOR does not write the tuple

Phase-tagging each region block with the last per-function site entered gives a
pre/post pair at **every** boundary for free. All 7 functions of
`il_call_perm.cpp`:

| boundary | result |
|---|---|
| `sched1` → `sched2` (a scheduler run + globregs) | **DIFFERS on 7 of 7** (13,14,14,14,14,14,14 → 11,11,12,13,12,12,12 rows) |
| `sched2` → `sched3` (**COLOR**) | **IDENTICAL on 7 of 7**, 83 tuples |
| `sched3` → `sched0` (the lowering band) | **DIFFERS on 7 of 7** |
| raw window, 128 bytes/tuple, 83 aligned pairs across COLOR | **offsets COLOR writes: NONE** |

**The null is about COLOR's write set, not about the instrument**, and the two
neighbouring pairs are the control that says so: the same five fields, read by
the same walk on the same run, move at the scheduler and at lowering. An
instrument blind everywhere would be vacuous; this one is blind in exactly one
place. `the_tuple_walk_sees_the_scheduler_move_the_list` fences that, and if it
ever fails, every COLOR conclusion drawn here is void.

**What it means for migration step 5.** The assigned register is not in the
tuple record's first 128 bytes. It lives in the operand records the tuple
points to, or in the allocator's own candidate records
(`P_REGALLOC.md` §5's `cand+0x28` / `+0x3c`). That replaces the plan's *"read
`P_REGALLOC.md` and the export once the mechanism is green"* with a measurement
that rules the tuple out.

### Six of the 26 matched TUs produce a structurally EMPTY snapshot

`decomp_pch`, `GainEffect`, `HeadsetPlaybackEffect`, `PeakDetector`,
`mmx_optimized`, `sse_optimized`: `hits 0 · regions 0 · tuples 0` on all six
per-function sites. Not a tap failure — c2 runs no per-function phase because
those TUs emit no function bodies. Reported rather than averaged away, because
"empty and deterministic" is the outcome this lane registered as most likely to
be misreported (P16). The two dyninit TUs that *do* have bodies —
`TomCryptLicense`, `ZlibLicense` — read `hits=14` = 2 functions × 7 sites.

## 4. Estimate vs outcome — every registered prediction scored

| # | prediction | p | outcome |
|---|---|---:|---|
| P1 | slide 0 on ≥ 9/10 runs | 0.93 | **HIT** — slide 0 on every run, two derivations agreeing |
| P2 | `VirtualProtect(PAGE_EXECUTE_READWRITE)` succeeds under wibo | 0.85 | **HIT** — 7/7 sites, every run |
| P3 | the detour at `0x10b7dc9f` fires ≥ 1× at `/O1` | 0.88 | **HIT** — 7× on `il_call_perm.cpp` |
| P4 | G1 over the fixtures + the matched TUs | 0.80 | **HIT** — 0 of 410 |
| P5 | G2 at the first schema | 0.75 | **HIT** at the first schema; no canonicalization iteration needed |
| P6 | G2b, no path/PID/pointer leak | 0.85 | **HIT** |
| P7 | G3 discriminates | 0.90 | **HIT** |
| P8 | ≥ 3 tuple rows on `il_call_perm.cpp` | 0.70 | **HIT** — 329 |
| P9 | the pre/post-COLOR pair DIFFERS on ≥ 1 fixture | 0.55 | **REFUTED** — identical on 7 of 7, and a 128-byte window says COLOR writes nothing in the tuple. Reported as a finding, in those words |
| P10 | the function-record → tuple-list-head offset is needed | 0.20 | **NOT NEEDED** — the region tap hands over a live tuple pointer, and phase tagging gave the pre/post pair without it |
| P11 | the 57 beacon sites map 1:1 to the 35 pass names | 0.15 | **NOT MEASURED** — see §6 |
| P12 | a c2 flag dumps per-pass IR | 0.05 | **NOT MEASURED** (the flag-table enumeration was not re-run) |
| P13 | `-off#` is a per-pass ablation control | 0.15 | **REFUTED** — `/d2off0`, `/d2off1`, `/d2off2`, `/d2off14` on `il_call_perm.cpp` all give a timestamp-zeroed obj byte-identical to the flagless baseline (1717 B, sha `cb9f59c3aaa18f73`, five ways). No code-patch-free channel exists; the hook table is justified. Run **before** the site table was finalized, as registered |
| P14 | the lane ends in an honest DECLINE | 0.20 | **NO** |
| P15 | armed + deterministic + obj-neutral tuple snapshot **with a pre/post-COLOR diff** | 0.45 | **PARTIAL** — everything but the COLOR diff, and the COLOR diff is refuted rather than missing |
| P16 | counts-only, neutral and deterministic, payload empty/unverifiable | 0.25 | **NO** — 329 tuples on the control fixture, cross-derived three ways |
| P17 | cost within 1.5–3× a construct rung | 0.5 | **roughly**; not published in seconds (§7) |

**Calibration note.** Seven mechanism/determinism predictions in [0.70, 0.93]
all hit, which is consistent but is only seven draws. The two refutations
(P9, P13) were both registered *low* and both came back the way the low number
implied — P13 at 0.15 refuted, P9 at 0.55 refuted. The one place the prereg was
wrong in the *optimistic* direction is P10: it expected the head offset might be
needed 1 in 5 and the answer is that a second mechanism (phase tagging) removed
the need entirely.

## 5. PLAN DEFECTS — three, each caught by running rather than reading

1. **"The `HMODULE` LoadLibraryA returns IS the load base."** True on Windows,
   **false under wibo**, which returns `HMODULE 0x00000018` — an opaque token.
   The first armed run computed `slide=ef500018`, every site failed the
   fail-closed opcode check (`opcode=00`, expected `e8`) and **nothing was
   patched**: the invariant doing its job on its first outing. The slide now
   comes from `GetProcAddress("_InvokeCompilerPass@12") - 0x10bebffd`,
   cross-derived against `VirtualQuery`'s `AllocationBase`.
2. **Per-site `__declspec(naked)` thunks do not build.** GCC does not implement
   the `naked` attribute on i686, and `i686-w64-mingw32-gcc` is the only
   compiler `ensure_c2host` uses. Replaced by **one** top-level `__asm__` thunk
   that recovers the site from the return address the `call` already pushed
   (`retaddr == site + 5`). Smaller than the original, and it removes the
   compile-time cap on the number of sites — which is what makes the 57
   shared-callee beacon sites reachable at all.
3. **`FlushInstructionCache` is a missing import in wibo** and aborts the
   process. Dropped: x86's instruction cache is coherent, the bytes are written
   before c2 has executed them, and there is one thread.

Also changed from the plan, and it is an improvement rather than a
substitution: **the payload is buffered in a static arena and flushed after
`InvokeCompilerPass` returns**, with hand-rolled hex and no CRT call inside a
c2 frame. That *removes* the mingw-CRT-reentrancy question (plan unknown 7)
instead of measuring it.

### A fourth defect, in this lane's own first test

The neutrality test's first version added a third assertion — armed obj ==
`captured.ref_obj` — and it **failed**:
`Differs { first_offset: 8, a_len: 1725, b_len: 1721 }`. Offset 8 is
`PointerToSymbolTable`; the 4-byte delta was the embedded `/Fo` **path string**,
because the replay wrote to a scratch path while the pipeline obj was written to
its own. c2 was neutral all along and the assertion was comparing two different
commands. Both legs now replay to `captured.ref_obj_path`. **A "stronger" check
that silently changes the command is worth less than no check.**

### An environment defect, found while taking the base

`scripts/gen_dc3_workload.sh` maps only the **forward-slash** spelling of the
`e:/lazer_build_gmc1` include roots. At dc3-decomp `3df8fd5412c2`,
`tools/defines_common.py` emits them with **backslashes**, so six of eight `/I`
roots pass through unmapped and the scan reads
`match 17 · vocab-gap 10 · capture-fail 851` with **343** `gap-metric` keys —
every digit plausible, nothing saying the include roots are wrong. With the
backslash spellings mapped the same command reads the published state. The
invalid log is kept at `work/oracle/base_gap_BROKEN_FLAGS.log`. **A landable
one-line script fix, and not this lane's deliverable** — reported so the next
lane in a fresh worktree does not measure `capture-fail 851` and call it a base.

## 6. What this lane deliberately did NOT do

* **P11 — the 57 phase-beacon sites are not armed.** `0x10bec297` has 143
  occurrences in the flat export and only a subset are the per-function band.
  Arming them adds nothing to the go/no-go, and the prereg registered 0.15 that
  they map to the 35 pass names — a low-value measurement behind a green
  mechanism. The site table is built to take them (one shared thunk, no
  compile-time cap). **The record correction stands regardless:**
  `P_DAG.md` §2 calls `0x10bec297` *"the timer"* and it is **not** one — it
  reads `DAT_10c37d28` and tail-jumps `0x10bec23c` when set, i.e. the
  abort/cancellation poll the same global `_AbortCompilerPass@4` (`0x10bec2ac`)
  sets.
* **Residency (the peer proposal's M1 fork server) is NOT built**, and this is a
  disagreement with the brief's framing rather than an omission. One process per
  compile is precisely what makes snapshot determinism testable: no cross-compile
  state, no allocator reuse, no counter carry-over. Building residency "for free"
  here would be building the thing most likely to break the load-bearing
  property. **What a future product service reuses, exactly:** `tap_arm`'s
  slide derivation and fail-closed self-check, the multi-source
  `build_host_stub`, `replay_tapped`'s command construction (which is
  `build_replay_command` unchanged), and the structural fact that c2 runs inside
  a host process we own. What M1 adds is a loop and a socket — and it should be
  graded against these snapshots as its regression fence.
* **No `crates/` behaviour changed.** No emit, no refusal predicate, no census
  rule reads a `stage-*` key. `mismatch 0` remains the judge's alarm.

## 7. Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | see §7.1 |
| `scripts/gate.sh --jobs 16 --require-graded` | see §7.1 |
| 878-TU workload scan, base | `match 26 · mismatch 0 · codegen-gap 0 · vocab-gap 844 · capture-fail 8`; 395 `gap-metric` keys |
| workload stamp, both ends | see §7.1 — asserted EQUAL |
| `c2rs stage neutrality --payload` | `stage-tap-obj-differs 0` over 384 fixtures + 26 matched TUs |
| `c2rs stage determinism --payload` | `distinct-max 1 · unstable 0` over 10 runs × 4 fixtures |

### 7.1 The tip, measured — and the environment ASSERTED, not the exit code

**Suite**, `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release
--no-fail-fast`, at tip `098c6b28e`:

    49 targets · 1699 passed · 0 failed
    "SKIP: toolchain absent" occurrences: 0
    census_gate: 4 passed, finished in 145.65 s

The **environment** is what is asserted, not the exit code. `census_gate`'s
145.65 s is a provisioned run; the unprovisioned one is 0.00 s and prints the
same counts (#3219/#3231), and an unprovisioned worktree here would additionally
have made every stage measurement in §3 void rather than provisional.

Second derivation of 1699 (#3288): `grep -c '^test .* \.\.\. ok'` over the same
log reads **1699**, equal to the sum of the per-target `test result` lines.
And the delta against the base's **1,690 / 0 / 48** is accounted exactly:
`crates/c2-reference/tests/stage.rs` adds 6 integration tests and
`stage.rs`'s `mod tests` adds 3, for **+9 tests and +1 target** —
`1699 − 9 = 1690`, `49 − 1 = 48`.

**Gate**, `scripts/gate.sh --jobs 16 --require-graded`: GATE_RESULT_PLACEHOLDER

### 7.2 THE WORKLOAD STAMP MOVED MID-LANE, AND THIS LANE'S OWN INVALIDATION RULE FIRED

| end | stamp |
|---|---|
| base | `workload   3df8fd5412c2 (clean)  /home/free/code/milohax/dc3-decomp` |
| tip | `workload   b25928dfb2a6 (clean)  /home/free/code/milohax/dc3-decomp` |

`dc3-decomp` took `b25928dfb Merge fix/string-literals-20260820` while the lane
ran — a **fifth** stamp value, after the four this record already knows.
Prereg invalidation rule 1 says a stamp inequality **voids the results table**
and the response is to **re-read the base at the current stamp**, not to explain
the delta. Done:

| | base scan (`3df8fd5412c2`) | re-read at tip (`b25928dfb2a6`) |
|---|---|---|
| `match` / `mismatch` | 26 / 0 | **26 / 0** |
| `codegen-gap` / `vocab-gap` / `capture-fail` | 0 / 844 / 8 | **0 / 844 / 8** |
| `gap-metric` keys | 395 | **395**, same key names, none added or removed |
| **keys whose VALUE moved** | — | **105 of 395** |
| `fnbyte-exact` | 35,912 | **35,893** |
| `factor-c` | 169 | **170** |

**105 of 395 keys moved on a lane that changed no `crates/` behaviour** — the
sharpest instance of #3306/#3311 on record (the previous was 82 of 394), and
this time it happened *inside* one lane rather than between a dispatch and a
measurement. Logs: `work/oracle/base_gap.log`, `work/oracle/tip_gap.log`.

**And the re-read is what licenses §3's matched-TU row rather than voiding it.**
The 26 matched TUs are identical **by name** at both stamps, and — checked per
file, which is the part a name compare does not give — **all 26 source blobs are
byte-identical across the merge** (`git rev-parse <stamp>:<path>` per file; 79
other files changed). So `stage-tap-obj-differs 0 / graded 26` stands at both
ends. Had one of the 26 moved, that row would have been re-run, not annotated.

**Wall-clock seconds are not published.** The box carried an unrelated external
load of 32 → 150 on 32 cores across this lane; a timing taken there is a
measurement of the box.

## 8. The standing bound

The snapshot is a **development instrument**. It never gates an emit, never
appears in a refusal predicate, and no rule enters `crates/` on snapshot
equality alone. The obj byte compare against real `c2.dll` under wibo, with the
COFF `TimeDateStamp` zeroed, remains the sole judge. And a metric delta of zero
is evidence about **reach**, never about correctness (#3270–#3275): G1's zero
says the tap does not perturb c2 on the 410 objs it was run against, and
nothing more.
