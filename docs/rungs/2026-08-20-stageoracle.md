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

**The denominator is `armed-and-fired`, and the fix round is why** (§9, review
major 3). An obj on which the tap armed but never executed is byte-identical
for free — c2 runs no per-function phase on a TU with no function body — so it
is reported and excluded, never counted as evidence.

| population | ARMED AND FIRED | armed, never fired | `stage-tap-obj-differs` | errors | detour hits | sites armed / refused | walk refusals |
|---|---:|---:|---:|---:|---:|---|---:|
| `fixtures/cpp/*.cpp` (384 graded) | **355** | 29 | **0** | 2 | 47,979 | 2688 / 0 | **0** |
| the 26 matched workload TUs, at the workload's own flags | **20** | 6 | **0** | 0 | 1,252 | 182 / 0 | **0** |
| **campaign** | **375** | 35 | **0** | 2 | **49,231** | 2870 / 0 | **0** |

Measured with the **payload ON**, never extrapolated from a counts-only run —
the payload is the half that touches c2's own memory (plan unknown 8).
Second derivation (#3288): re-counting the log's own per-fixture verdict lines
gives `SAME 384 / DIFFERS 0 / ERR 2`, equal to the accumulated counters.
The two errors are `wmain_no_return{,_neg}.cpp` and they fail on the
**disarmed** leg — the untapped replay itself produces no obj — so they are
pre-existing and are classified as errors rather than folded into either side
of the required zero.

Logs: `work/oracle/neutrality_all_fixround.log`,
`work/oracle/neutrality_matched26_fixround.log` (the fix round's re-run, which
is what the table above reads; every per-fixture count reproduces the original
digit for digit). The originals stay at `work/oracle/neutrality_all.log` and
`work/oracle/neutrality_matched26.log` — they are the logs whose verdict line
the review's major 1 is about, and deleting them would delete the evidence.

**The matched-26 population is the same population**, and that is checked
rather than assumed: `dc3-decomp` moved **twice more** during the fix round
itself (`b25928dfb2a6` → `6f3a818e9893` → `2f35703d0241` — the **sixth and
seventh** stamp values this record knows, in a few hours), so all 26 source
blobs plus `config/373307D9/config.json` and `tools/defines_common.py` were
compared per file across the stamps — **0 of 26 moved, and neither flag input
moved**. The flags file is byte-identical to the one in the original run's
header.

The comparison is made through **one function**: `replay_tapped` with an empty
tap list *is* the disarmed leg. Each leg asserts its own state first — the
disarmed leg must print no tap line, the armed leg must report `armed_ok` —
because "identical" between two legs that were both disarmed is not a
measurement.

### G2 / G2b — DETERMINISM and CANONICALITY

`work/oracle/determinism_fixround.log`, payload on, 4 fixtures × (5 runs in ONE
fixed scratch directory + 5 runs each in a freshly minted one, with a different
`/Fo` path):

    stage-snap-runs 10 · stage-snap-distinct-max 1 · stage-snap-unstable-tus 0
    stage-snap-graded 4 · stage-snap-empty-payload 0
    add3 309 tuples · il_call_perm 329 · il_call_return 662 · mvp_add3 36

**One digest per fixture over all ten runs.** It held at the *first* schema.

> **CORRECTED IN THE FIX ROUND (review minor).** As originally published, *"5
> same-config runs"* did not run: both loops were byte-identical calls and
> every call minted its own `pid+nanos+counter` scratch dir, so there was no
> same-config leg and no contrast between the halves. The conclusion is
> unaffected — ten varied runs at `distinct-max 1` is strictly stronger than
> five plus five — but the experiment described was not the experiment that
> ran, which is exactly the thing this rung is not allowed to do. The first leg
> now pins one directory for all N runs, the numbers above are from the re-run,
> and each run additionally has to pass `armed_and_fired` before its digest is
> recorded (an unarmed run's empty stream hashes the same every time, so
> `distinct = 1` over unarmed runs is a vacuous green).
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

`work/oracle/snap_il_call_perm.log`.

**And zero walk refusals over the whole campaign** — `stage-tap-walk-refusals`
reads **0** on both fix-round neutrality logs, over 410 objs including
`jsonwriter.cpp` at 5,491 tuple rows. That matters because a truncated payload
would make every tuple count a floor rather than a measurement.

> **CORRECTED IN THE FIX ROUND (review major 2), and it was worse than the
> review found.** This zero was originally read out of the two neutrality logs
> — which **structurally could not contain the line**: `cmd_neutrality` printed
> no walk-refusal key at all, and the nine `grep` hits that looked like
> corroboration were eight fixture names containing the string *refuse* plus
> one `stage-sites-refused 0`, which counts ARMING refusals. The command now
> prints `stage-tap-walk-refusals` and one `TRUNCATED` line per occurrence, and
> the campaign was **re-run** to derive the zero from a log that could have
> carried a one.
>
> The second half is not in the review: **the payload's own fail-loud path was
> unreachable.** `REFUSE region arena-full` was appended with the same `ap`
> that had just stopped appending because the arena was full, so the
> announcement of truncation was itself dropped. Mutated to an 8 KiB arena, the
> pre-fix instrument reports `265 tuples · 0 walk refusals` **and a COLOR pair
> `DIFFERING 1 of 6`** — a phase difference manufactured by truncation, on the
> fixture whose whole finding is that the COLOR pair is identical 7 of 7. Fixed
> with a reserved tail that only `ap_reserved` may write, plus `ARENA … full=1`
> parsed as a refusal so truncation has no unwatched spelling.
> `work/oracle/fixround/mutation_arena_full.log`.

| derivation | how it is built | answer |
|---|---|---|
| the tap | patched call sites **inside c2's code**, counted in `c2host` | 7 |
| the listing | c2's own `/FAsc` writer | `7 PROC` |
| the obj | the COFF section table | `7 .text COMDAT` |

Three paths with no shared step after c2's front end. This also **re-derives
`P_DAG.md` §1's "four scheduler runs per function"** as an equality between
four separately-patched sites rather than as a reading.

The opcodes are self-consistent with the region finder's own control flow, in
the one way that is actually a cross-derivation: **opcode `0x30f` occurs at
category `0x17` and nowhere else** (re-derived at the fix round over all 329
rows — 56 of them are `17`, and every `0000030f` is one of those), and
`0x10be5d8b` — the arm reached only when the category is `0x17` — is
`cmp DWORD PTR [esi+0x4],ebx` with `ebx = 0x30f` set at `0x10be5d4c`. The code
predicts that pairing and the payload shows it.

> **CORRECTED IN THE FIX ROUND (review major 5).** This paragraph used to add
> *"and the categories observed (`0x0d 0x0f 0x12 0x15 0x17 0x19 0x1a`) are the
> set that function branches on."* **That is a category error and it is false.**
> The observed set is every category appearing in a region **body**; the
> finder's dispatch at `0x10be5d6f` (`sub edi,0x12` / `dec dec` / `sub edi,3` /
> `dec dec` / `dec dec`) branches on region **terminators**, and its set is
> `{0x12, 0x14, 0x17, 0x19, 0x1b}`. Four observed values (`0x0d 0x0f 0x15
> 0x1a`) are not branched on at all, and two branch values (`0x14`, `0x1b`)
> never appear in this fixture. The two sets have no reason to be equal, so
> their agreeing would have been the surprise. Offered as corroboration in the
> section that exists to stop an inspected green — the sharpest possible place
> to be wrong.

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

Log, committed at the fix round: **`work/oracle/snap_il_call_perm.log`**
(`c2rs stage snap --fixtures il_call_perm.cpp --raw 128`, 377 lines, carrying
all 329 `TU` rows, the 21 phase-pair verdicts above and the raw-window result).
The lane's own results-table rule — *"every published number is DERIVED FROM A
LOG committed under `work/oracle/`"* (PREREG, #3231 F2) — was met by every G1,
G2 and G4 number and **not** by G5's, which was the review's minor; the rule
does not bend for numbers that happen to reproduce, and they do: the review
re-derived them independently and so did this re-run.

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
be misreported (P16). **They are the reason G1's denominator moved in the fix
round**: with nothing firing, armed-vs-disarmed byte-identity on those objs is
free (§9).

The two dyninit TUs that *do* have bodies — `TomCryptLicense`, `ZlibLicense` —
read `hits=14 regions=8`, which is **one** function, not two: with all seven
sites armed, `total_hits = 6 × functions + regions`, so `(14 − 8) / 6 = 1`.
Re-derived directly at the fix round (`c2rs stage counts` on both TUs reads
`sched1=1 … sched0=1 region=8`) and corroborated by the repo's own record —
board **#2241**, *"their one emitted function is `??__EsLicense@@YAXXZ`"*. The
original *"= 2 functions × 7 sites"* arithmetic was wrong twice over (it also
counted `region`, which is per-region) and propagated verbatim into the lane's
hand-off summary; review minor, fixed here and in `work/merge-oracle.txt`.

## 4. Estimate vs outcome — every registered prediction scored

| # | prediction | p | outcome |
|---|---|---:|---|
| P1 | slide 0 on ≥ 9/10 runs | 0.93 | **HIT** — slide 0 on every run, two derivations agreeing |
| P2 | `VirtualProtect(PAGE_EXECUTE_READWRITE)` succeeds under wibo | 0.85 | **HIT** — 7/7 sites, every run |
| P3 | the detour at `0x10b7dc9f` fires ≥ 1× at `/O1` | 0.88 | **HIT** — 7× on `il_call_perm.cpp` |
| P4 | G1 over the fixtures + the matched TUs | 0.80 | **HIT** — 0 of 375 armed-and-fired (of 410 graded; the denominator is corrected in §3/§9) |
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
| **G4** | *(a registered gate, not a P-number: every pre-existing `gap-metric` key byte-identical between base and tip)* | required | **MEASURED AND HELD, at the fix round — and it had no row here at all, which was the review's minor.** As first run it read **105 of 395 keys moved** and was confounded: the two scans were at *different workload stamps* (`3df8fd5412c2` vs `b25928dfb2a6`), so nothing separated the lane's code from `dc3-decomp`'s merge. Re-run properly — **master's code and this branch's code, both at one stamp `2f35703d0241`** — the two 878-TU scans agree on **all 395 keys, every value, byte for byte** (`diff` of the key lines is empty). The lane moves nothing. Logs `work/oracle/fixround/g4_{master,tip}_code.log`; the binaries hash `8cf63d0d1c9c6458` and `066b5b722b755e9d`, which is the evidence that two builds really ran, because both trees resolve their `c2-rs` provenance line through this worktree's `.git`. **Bound:** a gap scan runs at `replay-every=0`, so it never executes `c2host` — G4's identity is about the harness's analysis code and says nothing about the tap binary. The control for *that* is below |

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

### 6.1 Open questions handed forward, each with the deciding probe named

These are **not** loose ends the lane forgot; they are bounded and each has a
next measurement, so nobody has to rediscover the question.

1. **Where does COLOR's output live?** Not in the tuple's first 128 bytes
   (§3). The deciding probe is a raw window on the **operand records the tuple
   points to** (the tuple's pointer fields are inside the window this lane
   already dumps, so the next lane can follow them), and on the allocator's own
   candidate records around `P_REGALLOC.md` §5's `cand+0x28` / `+0x3c`. Until
   that lands, **a ported COLOR has no per-stage grade** and step 5's pricing
   for the allocator is unchanged from black box.
2. **Does the region tap see every mutation of the tuple order?** Unmeasured,
   and `P_DAG.md` §6 is the standing reason to doubt it: **tuple order has a
   second author**, the `factor.c` block merger (`0x10b3baa8` → `0x10b3a790`),
   which is *not* a DAG client. A snapshot taken only at scheduler boundaries
   can therefore miss a mutation entirely and look stable. The deciding probe
   is cheap and named: check whether the `sched0` region walk equals the code
   the `.cod` finally prints, and **name the residue rather than absorbing
   it**. `crates/c2-reference/src/cod.rs` already reads the `.cod`.
3. **Do the 57 `call 0x10bec297` beacon sites map to the 35 pass names?** P11,
   registered 0.15 against, not measured. The deciding probe is to arm them all
   count-only on one fixture and compare the shape to the name array at
   `0x10c2e9e4` — which has **zero code xrefs in the flat export**, so
   *"COLOR is index 14"* is a data fact while *"the pipeline dispatches through
   that table"* is a hypothesis.
4. **How many armed sites does the mechanism tolerate?** Measured only to 7.
   The one-thunk design has no compile-time cap and the arming loop is linear,
   but the per-call overhead at 60+ sites and whether a heavier payload
   perturbs c2's own memory behaviour enough to threaten G1 are **not**
   measured. G1 must be re-run at whatever table a later lane arms — this
   lane's zero is a statement about seven sites.

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

> **THE NUMBERS BELOW ARE THE LANE'S ORIGINAL RUN, AT `098c6b28e`, AND THEY ARE
> SUPERSEDED BY §9.5** — the fix round rebased twice (master moved under the
> lane a second time, taking `w-refrev`) and re-ran both. Kept because a rung
> that overwrites its own measurements cannot be graded.


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

**Gate**, `scripts/gate.sh --jobs 16 --require-graded`, at the same tip:

    GATE: PASS (HATCH-RED REFUSED)
    lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
    graded: 6948 fixture-verdicts across all lanes
    sweep:  19556 of 19556 reached, 19460 GRADED, 0 mismatch
    cross:  90424 of 90812 cells graded, 0 mismatch
    debug:  18 of 18 lanes through a DEBUG-profile c2rs,
            6948 verdicts, match 2423, 0 mismatch, 0 PANIC
    graded tree: 90df6918eff3 (750 files under crates fixtures scripts)

Every lane reads `386/386`. The sweep and cross figures equal the ones this
lane was dispatched against, digit for digit.

**`HATCH-RED REFUSED` is PRE-EXISTING and is not this lane's**, and that is
asserted rather than assumed: the two most recently merged lanes' own gate logs
(`work/merge-c2map3-gate.log:79,92`, `work/merge-witness7-gate.log:74`) carry
the identical `REFUSED 0/14 … HATCH-STALE` row and the identical
`GATE: PASS (HATCH-RED REFUSED)` headline. `hatch.py`'s needles have drifted
out of the tree (board **#1389**); the refusal is a property of the tree, not
of this branch, and `gate.sh` qualifies its own headline for exactly that
reason.

Per #3215, this lane lands tests, so it may **not** claim a graded-tree
identity — the tree hash moves by construction. What it claims is identity of
the **counts**, which is the criterion #290 actually uses, and every count
above matches.

**Both were taken at `098c6b28e`, and the lane's tip is three commits later.**
The three are `docs/`, `work/` and `docs/rungs/INDEX.md` only — nothing under
`crates fixtures scripts`, which is exactly the set `graded tree
90df6918eff3` hashes, so the gate's verdict is unchanged by them by
construction rather than by assumption. The one test that *can* see a `docs/`
edit, `c2-harness::rung_registry` (it asserts `INDEX.md` equals what
`scripts/gen_rung_index.sh` generates), was re-run at the real tip: 2 passed,
0 failed.

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

| | base scan (`3df8fd5412c2`) | **tip CODE** re-read at (`b25928dfb2a6`) |
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

> **CORRECTED IN THE FIX ROUND (review minor), and the correction changes the
> claim.** The second column above is the **tip code at the new stamp**, not
> the **base code** at the new stamp — so *"re-read the base at the current
> stamp"*, which is what the lane's own invalidation rule demands and what this
> section said it did, was **not** what ran, and the 105-key delta is
> confounded between `dc3-decomp`'s merge and the lane's own code. The lane
> then argued the confound away (*"the `crates/` diff is instrument-only"*),
> which is precisely the substitution rule 1 exists to forbid.
>
> **Measured instead.** Two 878-TU scans at ONE stamp — `2f35703d0241`, read
> **before and after each scan** and equal at all four readings — one with
> **master's** code and one with **this branch's**: all **395 keys identical,
> every value** (`diff` of the key lines is empty), `match 26 · mismatch 0 ·
> codegen-gap 0 · vocab-gap 844 · capture-fail 8` on both, and `fnbyte-exact
> 35,894` / `factor-c 170` on both. **G4 HOLDS; the lane moves nothing**, and
> the 105 was the workload, entirely. Logs
> `work/oracle/fixround/g4_master_code.log` and `.../g4_tip_code.log`; the two
> binaries hash `8cf63d0d1c9c6458` and `066b5b722b755e9d`, which is how you can
> tell two builds ran — both trees resolve their `c2-rs` provenance line
> through this worktree's `.git`, so that line is NOT the discriminator.
>
> A further **43 keys** moved between the lane's own tip scan and this one
> (`fnbyte-exact 35,893 → 35,894`), which is #3306/#3311 taking another
> instance in the hours between a lane and its fix round.

**And the re-read is what licenses §3's matched-TU row rather than voiding it.**
The 26 matched TUs are identical **by name** at both stamps, and — checked per
file, which is the part a name compare does not give — **all 26 source blobs are
byte-identical across the merge** (`git rev-parse <stamp>:<path>` per file; 79
other files changed). So `stage-tap-obj-differs 0 / graded 26` stands at both
ends. Had one of the 26 moved, that row would have been re-run, not annotated.

### 7.3 The closing control the rung did not name: `c2host.exe` CHANGED SHAPE

Review note, and it is a real gap in §6's *"no `crates/` behaviour changed"*:
the payload arena is a 4 MiB static, so `c2host.exe`'s `.bss` grew to
`0x004008f8` — and **every replay in this repo runs through that binary**. G1's
design (armed vs disarmed through the *same* exe) structurally cannot see it.

Measured at the fix round, at scale: `c2rs replay <cpp>` compares the **cl.exe
pipeline obj** (no `c2host` in the picture at all) against the **`c2host`
replay obj**, and over `fixtures/cpp` it reads **384 of 384
`normalized_identical=true`, 0 false**. The 2 non-graded are
`wmain_no_return{,_neg}.cpp`, the pre-existing C4716-promoted-to-error pair —
**asserted rather than assumed**: master's binary fails them identically, with
the same two `error C4716` lines and no obj.
`work/oracle/fixround/replay_pipeline_control.log`.

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

---

## 9. FIX ROUND — the review's five majors and fourteen findings, dispositioned

Review verdict `land-with-fixes` (`work/reviews/oracle-review.json`), read at
tip `8b338b39e`. This branch was rebased onto master `826ba1e41` **first**, so
one gate run covers the rebase and the fixes together. Outcome word is
unchanged: **`instrument`**.

**Nothing here was fixed by reading.** Every new assertion has a mutation
demonstration — the defect it exists to catch is planted, the assertion is
watched failing with a message distinct from every other failure in the file,
and the earlier guards' quantities are held fixed so the assertion is actually
reached. One of those mutations forced an API change on its own (below).

### 9.1 Disposition

| # | severity | finding | disposition |
|---|---|---|---|
| 1 | **major** | `c2rs stage neutrality` never asserted the tap armed; its required-zero could print `G1 HOLDS` over a population where no byte of c2.dll was patched | **FIXED** at `3a840fbb9`. Verdict now conditioned on a POSITIVE `TapReport::armed_and_fired` (every requested site armed, none refused, ≥1 detour executed) and prints `G1 IS VACUOUS` otherwise. **Mutation** §9.2 (a) |
| 2 | **major** | *"zero walk refusals over the whole campaign"* read from a log that structurally could not contain the line | **FIXED** at `3a840fbb9` + campaign **re-run**. `cmd_neutrality` prints `stage-tap-walk-refusals` and one `TRUNCATED` line per occurrence; the zero is now read off `work/oracle/neutrality_{all,matched26}_fixround.log`. §3 |
| 3 | **major** | G1's denominator counted 35 objs on which the tap fired zero times; board #3322's headline said *"over 410 graded objs with the payload armed"* | **FIXED** at `3a840fbb9` (code), and #3322's headline now reads **375 armed-and-fired of 410 graded**, naming the 35 as free. **Mutation** §9.2 (b) |
| 4 | **major** | *"the four scheduler runs are gated ONLY by `DAT_10c2e2fc`"* is refuted by the disassembly; a lane delivering whitebox corrections shipped a new wrong record claim | **FIXED** at `7a5a84954`, in five places: `P_DAG.md` §1 (with the bytes), `WB_DAGORDER_FINDINGS.md` §1, `OPT_GATED_SITES`' doc (which also said *"six sites"* over a four-element array), the G3 test's doc and the G5 test's comment. `DISCLOSURE` W-STAGETAP-1 gains the omitted gate addresses. `/Od` ⇒ 0 is kept and is unaffected; `hits == functions` is relabelled EMPIRICAL |
| 5 | **major** | the observed-categories cross-derivation is a category error: region **bodies** vs region **terminators** | **FIXED** at this commit — the false half is retracted in §3 with the two sets printed, the true half (`0x30f` ⇔ category `0x17`, the pair `0x10be5d8b` tests) is kept and re-derived over all 329 rows |
| 6 | minor | `TomCryptLicense`/`ZlibLicense` *"`hits=14` = 2 functions × 7 sites"* — wrong decomposition | **FIXED** in §3. `total_hits = 6×functions + regions` ⇒ **1** function, re-derived by `stage counts` (`sched1=1 … region=8`) and matching board #2241 |
| 7 | minor | G2's *"5 same-config runs"* did not exist — both loops minted fresh dirs | **FIXED** at `3a840fbb9`; the first leg pins one scratch dir. Re-run: `work/oracle/determinism_fixround.log`, `distinct-max 1`, same tuple counts |
| 8 | minor | G4 registered, measured failing, unscored; and the *"re-read the base at the current stamp"* was not done | **FIXED, AND G4 NOW HOLDS.** Two 878-TU scans at ONE stamp — master's code vs this branch's — agree on all 395 keys, every value. G4 has a row in §4 and the correction is in §7.2 |
| 9 | minor | G5's numbers backed by no committed log, against the lane's own results-table rule | **FIXED** — `work/oracle/snap_il_call_perm.log` (377 lines, all 329 `TU` rows, the 21 phase verdicts, the raw-window result) |
| 10 | minor | the two `DISCLOSURE` rows land four commits after the adoptions they cover | **DECLINED, priced** — §9.3 (i). Partially served: each row now names its adopting commit |
| 11 | note | `c2host.exe` changed shape (4 MiB `.bss`); G1 cannot see it and the rung did not name the closing control | **FIXED** — §7.3, and the control is measured at scale rather than named: 384/384 pipeline-obj vs c2host-replay-obj identical |
| 12 | note | `OPT_GATED_SITES`' doc says *"six sites"* over four, and is wrong about membership | **FIXED** at `7a5a84954` (folded into 4) |
| 13 | note | the fail-closed arming path has no standing test against a live image; the `+ slide` half never ran | **FIXED** at `f237a5822` — `a_wrong_slide_arms_nothing_and_never_moves_the_obj`. **Mutation** §9.2 (c) |
| 14 | note | `c2host/README.md`'s build command is the old single-source one | **FIXED** at `7a5a84954` |

**Found while fixing, and not in the review** — the payload's fail-loud path was
**unreachable**: `REFUSE region arena-full` was appended with the same `ap` that
had just stopped appending, so a full arena announced nothing and `ARENA …
full=1` was not parsed. **Mutation** §9.2 (d), and it is the sharpest of the
four: under it the instrument reports a COLOR pair `DIFFERING 1 of 6` on the
fixture whose entire finding is `IDENTICAL 7 of 7`.

### 9.2 The four mutations

Each ran with everything but the planted defect held fixed.

**(a) `G1 HOLDS` over a run that patched nothing.** `--force-slide 18`
displaces every site address; all seven fail-closed checks refuse. The
**pre-fix binary** (built from `c6e0fa9b7`) prints, in the same output:

    gap-metric stage-sites-refused 7
    G1 HOLDS over 1 graded fixtures

The refusal count was in its own output and the verdict ignored it. The fixed
binary prints `G1 IS VACUOUS — THE TAP DID NOT ARM AND FIRE` and names the
zeros. `work/oracle/fixround/mutation_G1_{before,after}.log`.

**(b) The free denominator.** `mvp_empty.cpp` emits no function body, so the
seven sites arm and never fire. Fixed binary: `NOFIRE`, excluded from the
denominator, and the verdict says so in one sentence.
`work/oracle/fixround/nofire_demo.log`.

**(c) The fail-closed check, disabled.** With check 2 (`target + slide`)
commented out and the opcode check left standing, a `+0x18` slide **arms five
of seven sites** — five real `e8 rel32` bytes at the displaced addresses — and
c2 SIGSEGVs. The new test fails with `A WRONG SLIDE PATCHED 5 SITE(S)`; the
other six tests in the file still pass, so the control leg still armed and
fired. `work/oracle/fixround/mutation_failclosed.log`.

> **THIS MUTATION CHANGED THE CODE, WHICH IS THE POINT OF RUNNING IT.** The
> first version of the test read the crash as `forced-slide replay failed` — a
> true sentence naming the wrong defect, and an **earlier guard tripping before
> the assertion the test exists for**. `replay_tapped_forced_slide` now returns
> `Option<ObjImage>`, so a missing obj is data and the arming assertion is
> reached. *"I saw the test fail"* was not evidence, and here is the instance.

**(d) The arena at 8 KiB.** `il_call_perm.cpp`'s 329-row payload overruns it.
Pre-fix: `265 tuples · 0 walk refusals · COLOR pair DIFFERING 1 of 6` —
silent truncation manufacturing a phase difference. Fixed: two refusals
reported, `stage snap` prints `TRUNCATED` and withholds the COLOR finding, and
`the_snapshot_is_nonempty_and_agrees_with_a_second_derivation` fails with *"the
bounded walk was TRUNCATED, so the tuple count below is a floor and not a
measurement"* — reached with the earlier guards intact (armed 7/7, 265 tuples
non-empty). `work/oracle/fixround/mutation_arena_full.log`.

### 9.3 Priced declines

**(i) Finding 10 — the `DISCLOSURE` rows land four commits after the code that
adopts the addresses.** DECLINED.

* **Cost of fixing:** rewriting 15 commits of already-reviewed history to move
  a documentation hunk earlier. The branch is on the critical path (proposal §5
  step 5 sits behind it), the review's own re-derivations are pinned to those
  commits, and this box's standing rule is that intermediate history is the
  valuable part — a fix here trades a real record for a formal one.
* **Cost of the hole:** at four commits in the middle of one unmerged branch,
  the tree carries seven disassembly-derived addresses without a provenance
  row. The letter of `CLAUDE.md` scopes the rule to `crates/`, and `crates/`
  holds names only — the addresses are in `c2host/`. At the tip, and therefore
  at every commit a reader will ever `git log`, both rows are present.
* **Partial fix taken instead:** each row now names its adopting commit
  (`2bfc70caf`, `a09f33704`), which is the property a reader actually wants —
  *"which change adopted this address"* — and which the same-commit rule was a
  proxy for.

**(ii) The review's `mostLikelyRemainingDefect` — completeness of the
observable — is NOT addressed here, deliberately.** It is open question 2 in
§6.1 with its deciding probe already named (does the `sched0` region walk equal
what the `.cod` prints?), and it is a *measurement lane*, not a fix: `P_DAG.md`
§6's second author of tuple order (`factor.c`'s block merger, not a DAG client)
is a claim about c2, not about this instrument. Fixing it inside a fix round
would mean shipping an unreviewed characterization result under a review's
cover. **The standing bound already forbids the failure it leads to** (§8: no
`crates/` rule enters on snapshot equality), and the rung says in §6.1 that a
ported pass must not be graded against a stage snapshot until that probe lands.

### 9.4 What the fix round did not change

The verdict. **GO** stands, on a denominator that is now 375 rather than 410,
with the same zero, the same seven armed sites, the same 49,231 detour hits,
and the COLOR null intact at 7 of 7 with `offsets COLOR writes: NONE`.

### 9.5 THE FIX ROUND'S OWN GATE AND SUITE, at the rebased tip

Rebased **twice**: onto `826ba1e41` as dispatched, then onto `5f9ae0829` when
the coordinator merged `w-refrev` mid-round. Both conflicts were
`docs/rungs/INDEX.md` and both were resolved by **regenerating** it
(`scripts/gen_rung_index.sh`), never by hand — a hand-resolved INDEX has failed
`rung_registry` before. The `docs/BOARD.md` conflict on the first rebase was
resolved by keeping master's reservation block and amending **only** the
`#3322`–`#3326` row, which is this lane's.

**Gate**, `scripts/gate.sh --jobs 16 --require-graded`:

    GATE: PASS (HATCH-RED REFUSED)
    lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
    graded: 6948 fixture-verdicts across all lanes
    sweep:  PASS — 19556 of 19556 reached, 19460 GRADED, 0 mismatch
    cross:  PASS — 90424 of 90812 cells graded, 0 mismatch
    debug:  PASS — 18 of 18 lanes through a DEBUG-profile c2rs,
            6948 verdicts, match 2423, 0 mismatch, 0 PANIC
    graded tree: b41e1158e6d7 (752 files under crates fixtures scripts)
    GATE_EXIT=0

Every count equals the lane's dispatched values digit for digit. `HATCH-RED
REFUSED` is pre-existing (§7.1).

**Suite**, `scripts/partest.sh --jobs 14` — which sets `C2RS_REQUIRE_TOOLCHAIN=1`
by default (board #3247, landed by `w-warranty` after this lane's first run):

    49 targets · 1707 passed · 0 failed · 1 ignored · 1708 named results
    "SKIP: toolchain absent" occurrences: 0
    SUITE_EXIT=0

**Two derivations of the target count, because `cargo test` can silently run a
fraction of the targets:** summing `^test result` lines over the aggregate log
gives `49 targets / 1707 passed / 0 failed / 1 ignored`, and summing them over
the 48 per-target logs plus the doc-test log gives **the same 49 / 1707**.
`#[test]` occurrences under `crates/`: master `5f9ae0829` **1705**, this tip
**1717** — **+12**, all of them in the two files this lane adds
(`crates/c2-reference/src/stage.rs` +5, `crates/c2-reference/tests/stage.rs`
+7). Of those twelve, **three** are the fix round's: the two new parse tests
and `a_wrong_slide_arms_nothing_and_never_moves_the_obj`.

**One integration finding from the second rebase, fixed rather than reported:**
`w-refrev` funnelled fifteen hand-rolled `SKIP:` blocks into
`toolchain_gate::toolchain_ready`, and `c2rs stage` — written on this branch in
parallel — carried a sixteenth. Merged as-was, the funnel would have had
exactly one hole, and under `C2RS_REQUIRE_TOOLCHAIN` that command would have
printed SKIP and exited 0 where every other command refuses. Converted, and
demonstrated both ways with `C2RS_MINGW` pointed at a nonexistent binary:
`work/oracle/fixround/demand_honoured.log`.

**And one gate run is kept BECAUSE it failed.** The first fix-round gate ran
while a peer lane's full suite held the box at load 38–49 and read `sweep: FAIL
— UNGRADED 100 exceeds the carried baseline 96`, with **0 mismatch anywhere**:
~40 `capture_reference produced no obj`, plus one
`ReferenceReplay=MISMATCH @ offset 855 (ref=855B replay=894B) cache=hit` — a
39-byte **length** difference on a cache HIT, i.e. the `/Fo` path-string class
(§5's fourth plan defect), out of a capture cache **shared** with the peer's
concurrent gate. The case is clean twice in isolation. Excerpt and both
isolated runs: `work/oracle/fixround/gate_LOADED_sweep_excerpt.log`.

**A gate row also caught this fix round's own worst moment**, and it is the
same lesson from the other side: `FATAL: cargo build failed — refusing to grade
with whatever binary happens to be on disk`. The tip did not compile for about
forty minutes, because a signature change was verified with
`cargo test -p c2-reference`, which does not build `c2-harness`. Every
`c2rs` invocation in that window ran a **stale binary**; every number taken in
it was re-taken against a binary built from a committed tree, and all of them
reproduced. A number is evidence about the binary that produced it.
