# PREREG — lane `w-stageoracle` (characterization lane)

    Lane:      w-stageoracle
    Branch:    wt-w-stageoracle
    Kind:      characterization lane
    Question:  Can real c2's INTERMEDIATE per-function state be observed
               reliably — deterministically, canonically, and WITHOUT changing
               the obj the sole judge compares?
    Frozen:    2026-08-20, as this lane's FIRST commit, before any probe or
               edit to c2host/, crates/ or scripts/.
    Judge:     unchanged. Real `c2.dll` under wibo, byte-exact obj compare,
               COFF TimeDateStamp (4..8) zeroed. Nothing in this lane becomes
               a second judge; nothing is admitted on snapshot equality alone.

---

## 0. What already ships, so this lane cannot re-ship it

**Board #3314's rule applies to this lane before any other**: *"unserved in
`docs/` is not unserved in the repo."* Deliverable 3 of the brief — *"one
end-to-end instrumented TU per admitted family"* — is **already half-shipped**,
and a lane that reports it as new has re-shipped board #132/#134/#136.

What exists today, verified by reading at this lane's base:

* `Toolchain::capture_listing_with(src, work, flags, cwd, qxstalls)` —
  `crates/c2-reference/src/lib.rs:970`. Drives `cl /FAsc [/QXSTALLS] /Fa<path>`
  and returns the captured reference **and** the `.cod` text.
* `crates/c2-reference/src/cod.rs` — a 301-line `.cod` reader.
* `crates/c2-harness/src/listing.rs` (636 lines) — the population scan;
  `c2rs listing` / `c2rs listing-scan` subcommands.
* Three standing tests in `crates/c2-reference/tests/listing.rs`:
  `the_listing_does_not_perturb_the_obj`,
  `the_cod_is_byte_truth_except_at_relocated_branches`,
  `qxstalls_annotates_the_listing_and_only_with_the_flag`.

So **c2 already narrates its own output**: label counter, section order, EH
layout, relocations by name, and with `/QXSTALLS` a per-instruction issue
cycle.

**What the listing seam is NOT, and what this lane is therefore for.** The
`.cod` is an **end-state** observation: after all four scheduler runs, after
COLOR, after lowering. It cannot separate COLOR's output from the scheduler's,
which is the whole content of migration step 5. And `/QXSTALLS`'s issue cycles
come from **K4** (`0x10c1ce93`), which builds *its own* whole-function DAG
read-only and tears it down (`docs/whitebox/WB_DAGCLIENTS_FINDINGS.md` §4.4) —
they are a **re-derivation**, not the schedule the scheduler produced (which is
region-bounded at `0x50` tuples and at every call). Quoting them as c2's
schedule would be a live wrong claim.

**The unbuilt half, and this lane's entire subject: observation of c2's tuple
list BETWEEN passes.**

---

## 1. Base, pinned

Every value in this section is measured at this lane's own base, in this
worktree, and none is carried from the dispatching brief.

| fact | value |
|---|---|
| `git log --oneline -1` | `c277d3bb0 docs: architecture proposal — the conjunction is the binding constraint, and four staged IRs dissolve it` |
| `sha256sum compilers/X360/16.00.11886.00/c2.dll` | `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` |
| expected c2.dll sha (whitebox record's image, `C2_MAP_METHOD.md` §0) | `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` |
| c2.dll sha ASSERTION | **EQUAL** — the whitebox record is written against this exact image |
| `c2rs gap` workload stamp line, verbatim | `workload   3df8fd5412c2 (clean)  /home/free/code/milohax/dc3-decomp` |
| `grep -cE '^ *gap-metric \S+ \S+$'` at this base | **395**, derived twice (line count 395, distinct key names 395 — equal) |
| `match` / `mismatch` at this base | `match 26 · mismatch 0 · codegen-gap 0 · vocab-gap 844 · capture-fail 8` |
| wibo | `/home/free/code/milohax/wibo/build/wibo` (not on `PATH`) |
| mingw | `/usr/bin/i686-w64-mingw32-gcc` |
| box load at lane start (`uptime`) | 32.3 at lane start, 150.4 twenty minutes later (32 cores, unrelated external jobs) — wall-clock seconds are NOT published by this lane |

**The workload stamp is re-read at this lane's TIP and asserted EQUAL to the
line above** (#3306/#3311 — it moved three times on 2026-08-19 and
`fnbyte-exact` moved with it). A stamp inequality VOIDS every count in this
lane's results table; the response is to re-read the base at the current stamp,
not to explain the delta.

**Results-table rule (#3231 F2):** every published number in this lane is
DERIVED FROM A LOG committed under `work/oracle/`, never accumulated as the
campaign runs. A classifier that turns out wrong must be re-appliable
retroactively.

### 1.1 An environment defect found while taking the base — `gen_dc3_workload.sh` is BROKEN at this dc3 head

Recorded here because it is a *base measurement*, and because the first scan
this lane ran was **wrong in the flattering-to-nobody direction and would have
been obvious only if someone knew the right answer**.

`scripts/gen_dc3_workload.sh` maps the original build roots onto the local tree
with the forward-slash spellings `e:/lazer_build_gmc1/system/src` and
`e:/lazer_build_gmc1/lazer/src`. At dc3-decomp `3df8fd5412c2`,
`tools/defines_common.py` emits those roots with **backslashes**
(`e:\lazer_build_gmc1\system\src`), so **six of the eight `/I` roots pass
through unmapped**. The generated `flags.txt` then points c2 at `e:\…`, and the
scan reads:

    match 17 · mismatch 0 · codegen-gap 0 · vocab-gap 10 · capture-fail 851

with **343** `gap-metric` keys. Every one of those numbers is a plausible-looking
figure; nothing in the run says "your include roots are wrong". Log kept as
`work/oracle/base_gap_BROKEN_FLAGS.log` (invalid logs are kept, not deleted —
`docs/rungs/README.md`).

With the backslash spellings mapped as well, the same command reads
`match 26 · mismatch 0 · vocab-gap 844 · capture-fail 8` and **395** keys —
i.e. the published state. `work/oracle/flags.raw.txt` is the raw generator
output; `work/dc3-workload/flags.txt` is the corrected file this lane measures
with, and it is **byte-identical to the main repo's** `work/dc3-workload/flags.txt`
(diffed, empty).

**This is a landable one-line script fix and it is NOT this lane's deliverable.**
It is reported so the next lane in a fresh worktree does not silently measure
`capture-fail 851` and call it a base. It is also the reason the key count read
343 before it read 395: **a lane that finds an unexpected delta owes a
measurement before it owes a cause** (#3269) — and the cause here was neither
a peer's merge nor the workload moving, but the lane's own generator.

**Second-derivation rule (#3288):** every published count is derived a second,
differently-built way and the two are diffed. This has caught a wrong figure in
every lane that has run it.

---

## 2. Mechanism facts READ (not measured) at this base

All seven call sites the plan names were verified byte-for-byte against the
flat export `~/ghidra-projects/export/c2/objdump_intel.asm` before this prereg
was frozen. They are `[R]` — read from disassembly, **not** obj-checked.

| site VA | bytes at site | target | what (`P_DAG.md` §1) |
|---|---|---|---|
| `0x10b7dc9f` | `e8 de 86 06 00` | `0x10be6382` | scheduler run 1 (edx=1) |
| `0x10b7dcb7` | `e8 77 99 fd ff` | `0x10b57633` | globregs |
| `0x10b7dcde` | `e8 9f 86 06 00` | `0x10be6382` | scheduler run 2 |
| `0x10b7dcf6` | `e8 9f 3f fb ff` | `0x10b31c9a` | the register allocator (COLOR band) |
| `0x10b7dd1d` | `e8 60 86 06 00` | `0x10be6382` | scheduler run 3 |
| `0x10b7e00c` | `e8 71 83 06 00` | `0x10be6382` | scheduler run 4 (mode 0) |
| `0x10be643e` | `e8 08 f9 ff ff` | `0x10be5d4b` | the region finder — sole call site |

Also read at this base, and it grounds prediction P7: the three in-band
scheduler calls at `0x10b7dc9f`/`0x10b7dcde`/`0x10b7dd1d` each sit behind
`cmp DWORD PTR ds:0x10c2e2fc,edi` (with `edi == 0`) + `test BYTE PTR
[esi+0x1c],bl`, so at `/Od` — where `DAT_10c2e2fc`'s optimizer bit is clear —
**none of the three is reached**. The discrimination control is a property of
the code, not a hope.

**Record correction, confirmed at this base:** `docs/whitebox/ref/P_DAG.md` §2
calls `0x10bec297` *"the timer"*. It is not. It reads `DAT_10c37d28` and
tail-jumps `0x10bec23c` when set — the **abort/cancellation poll**, the same
global `_AbortCompilerPass@4` (`0x10bec2ac`) sets. Confirmed by reading
`10bec297: cmp DWORD PTR ds:0x10c37d28,0x0 / je 0x10bec2a5 / jmp 0x10bec23c /
ret`. It remains a phase beacon (143 occurrences of the literal in the flat
export), which is what this lane would use it for, but the label is wrong.

---

## 3. Grading criterion — the lane is GREEN iff all five hold

Preconditions on every measurement: `C2RS_REQUIRE_TOOLCHAIN=1`; the worktree's
`compilers` symlink present (a fresh worktree has none, and without it every
capture test skips silently and a registered RED reads GREEN — #3219/#3231);
the workload stamp asserted equal at both ends.

**G1 — NEUTRALITY (required-zero; the sole judge).** Over all `fixtures/cpp/*.cpp`
and the matched workload TUs: obj bytes with taps ARMED == obj bytes with taps
DISARMED, TimeDateStamp zeroed. `stage-tap-obj-differs` **= 0**. Nonzero
anywhere → the oracle is grading a different compiler → **DECLINE**, no
negotiation.

**G2 — DETERMINISM (the load-bearing property).** Per graded TU, N=5 armed runs
in the same configuration → `distinct(digest(canonical_bytes)) == 1`.

**G2b — CANONICALITY.** N=5 more runs in **different working directories with
different `/Fo` paths** → same digest. Without this, G2 passes on a stream that
is deterministic only because the environment was.

**G3 — DISCRIMINATION (the null control).** On one NAMED fixture, `/Od` vs
`/O1`: the optimizer-gated taps fire **0** times at `/Od` and **≥1** at `/O1`.
If the two counts are equal, the instrument is measuring itself — the fifth
entry in this repo's *"ranking instruments measure themselves"* family.

**G4 — IDENTITY (#290's construct-rung pattern).** Base vs tip, line for line:
`match` / `mismatch`, gate lane counts, sweep, cross, the census numerator, the
blocker histogram key-for-key, and **every pre-existing `gap-metric` key
byte-identical** — the only additions being new `stage-*` keys. Note #3215:
this lane lands tests, so it may NOT claim a graded-tree identity over
`crates fixtures scripts`; it claims identity of the COUNTS.

**G5 — CONTENT (a positive check, never an inspected green).** At least one tap
emits a **non-empty** payload block, cross-derived a second, differently-built
way (#3288). A structurally deterministic **empty** snapshot passes G1–G4
trivially and is this project's own signature defect (absence read as success,
twelve recorded instances).

### Controls pinned BY NAME

* `taps_are_inert_unarmed_and_never_move_the_obj` — G1, on
  **`fixtures/cpp/il_call_perm.cpp`**.
* `scheduler_taps_are_silent_at_Od_and_loud_at_O1` — G3, same fixture.
* `the_snapshot_is_nonempty_and_agrees_with_a_second_derivation` — G5, same
  fixture.
* `the_tapped_run_actually_armed` — the ENVIRONMENT control. Under
  `C2RS_REQUIRE_TOOLCHAIN` it **FAILS** rather than skips when arming refused.
  Without it, "SKIP because mingw was missing" is the unprovisioned-worktree
  failure (#3219/#3231) wearing a new coat.

**Why `il_call_perm.cpp` and not `add3.cpp`.** `cod.rs`'s module doc records
`add3` as the control that *cannot detect* the property it was run against
(`mullw`/`add`/`blr`, no relocated branch) — the twelfth instance of
absence-read-as-success here. `il_call_perm.cpp` has multiple functions,
relocated branches, and **calls, which end scheduling regions** (`P_DAG.md`
§4.5). **`add3.cpp` is banned as this lane's positive control.**

---

## 4. Registered predictions (probability form, no discount factor)

**Mechanism**

* **P1** wibo loads `c2.dll` at its preferred base `0x10b00000` (slide 0) on
  ≥ 9 of 10 runs — **0.93**. The slide is *computed and printed* regardless, so
  a miss is not a failure.
* **P2** `VirtualProtect(.text page, PAGE_EXECUTE_READWRITE)` returns TRUE
  under `/home/free/code/milohax/wibo/build/wibo` — **0.85** [0.7, 0.95].
* **P3** A call-site detour at `0x10b7dc9f` fires ≥ 1 time on
  `il_call_perm.cpp` at `/O1` — **0.88**.
* **P4** **G1 NEUTRALITY: obj byte-identical armed vs disarmed on the fixture
  corpus + the matched TUs** — **0.80** [0.6, 0.92].

**Determinism**

* **P5** G2 holds (`distinct == 1`) on ≥ 95 % of graded TUs at the FIRST schema
  — **0.75**; after ONE canonicalization iteration — **0.90** [0.75, 0.97].
* **P6** G2b holds (no path/PID/pointer leak once the schema forbids them) —
  **0.85**.
* **P7** G3 discriminates (0 at `/Od`, ≥1 at `/O1`) — **0.90**.

**Content**

* **P8** The tuple walk from `0x10be643e` yields ≥ 3 tuple rows on
  `il_call_perm.cpp` — **0.70**. The offsets `+0/+4/+8/+9/+0xa` are `[R]`.
* **P9** The pre/post-COLOR snapshot pair **differs** on ≥ 1 named fixture —
  **0.55** [0.3, 0.8]. If it never differs, the walk is reading a list COLOR
  does not write, and that is a finding, not a green.
* **P10** The per-function-record → tuple-list-head offset is needed — **0.20**
  (the region tap avoids it).

**Registered because a lane would otherwise assume them**

* **P11** The `call 0x10bec297` beacon sites in `0x10b7d85e`–`0x10b7e300`
  correspond 1:1 to entries of the 35-pass name table — **0.15**, i.e. I expect
  NOT.
* **P12** A `c2` command-line flag exists that dumps per-pass IR — **0.05**.
* **P13** `-off#` (name `0x10b1437c`, target `0x10c2eccc`, kind `0x22`) is a
  per-pass ablation control — **0.15**. Cheap side probe; if it is, it is a
  second observation channel needing **no code patch at all**, worth more than
  half the hook table. **Registered to be run BEFORE the site table is
  finalized**, so the lane does not build the expensive channel when a free one
  exists.

**Outcome**

* **P14** P(the lane ends in an honest DECLINE) — **0.20**.
* **P15** P(it ships an armed, deterministic, obj-neutral tuple snapshot with a
  pre/post-COLOR diff) — **0.45**.
* **P16** P(it ships counts-only taps that are neutral and deterministic but
  whose payload is empty or unverifiable) — **0.25**. This is the outcome most
  likely to be *misreported* as success; G5 exists to force it to be reported as
  what it is.
* **P17** Lane cost within 1.5–3× a normal construct rung — **0.5**.

**Ceilings carry no discount factor.** If G1 or G2 fails, the reach of every
downstream migration step is **0**, not "reduced".

---

## 5. Invalidation rules, frozen

1. **Workload stamp inequality between the lane's two ends VOIDS the results
   table.** Re-read the base at the current stamp; keep the invalid log.
2. **A colour taken in an unvalidated environment is void, not provisional.**
   If `the_tapped_run_actually_armed` did not run and pass in the environment a
   measurement was taken in, the measurement is discarded and re-run.
3. **G1 nonzero anywhere ⇒ DECLINE**, and the decline is the deliverable. Every
   later step is void. The required sentence is that the tap **changes the
   compiler**, and the two-sided price is published.
4. **G2 irreducibly failing after two canonicalization iterations ⇒ DECLINE**,
   with the required sentence *"intermediate state cannot be observed
   reliably"* and the digest table as evidence.
5. **A deterministic EMPTY payload is reported as P16, in those words** — never
   as a green, and never as "the mechanism works".
6. **A metric delta of zero is evidence about REACH, never about correctness**
   (#3270–#3275). This lane publishes no correctness claim from a zero delta.

## 6. Standing bound on every claim this lane makes

The snapshot is a **development instrument**. It never gates an emit, never
appears in a refusal predicate, and no rule enters `crates/` on snapshot
equality alone. The obj byte compare against real `c2.dll` remains the sole
judge. Any disassembly-derived constant adopted into the repo takes a
`docs/whitebox/DISCLOSURE.md` row in the same commit that adopts it.
