# w-ildecode — the opaque middle's two edges, documented; and one of them turns out to be short

    Tag:       w-ildecode
    Slug:      w-ildecode
    Date:      2026-08-21
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization: what shape does data have at the two
               seams of the opaque middle, and by what code does c2 transform
               it?
    Census:    +0 — no crates/ emit rule, no refusal predicate, no fixture
    Record:    `docs/whitebox/WB_MIDDLE_INTERFACES.md`; prereg at
               `docs/whitebox/WB_MIDDLE_PREREG.md`, frozen as the lane's first
               commit; scratch under `work/w-ildecode/` (gitignored)
    Board:     #3357–#3360 (reserved at `72207b86f`, all four spent)

> **`coff::Function` field this lane's work would eventually write: NONE.**
> Arch review finding 3's prophylactic, answered honestly. This lane ports no
> pass and writes no field; what it produces is a record and five graded tests.
> The prophylactic's value is that a lane which cannot name a field has to say
> so out loud.

---

## 0. Verdict, one line each

| deliverable | outcome |
|---|---|
| **1. `WB_MIDDLE_INTERFACES.md`** | **SHIPPED** — both interfaces, addressed, cross-linked, READ vs MEASURED per claim, worked example traced through `mvp_add3` end to end |
| **2a. interface-1 proof code** | **SHIPPED** — a subset decoder reproduces the live tap's rows **row for row on 3 functions**, additive chains of 2/3/4 leaves |
| **2b. interface-2 proof code** | **SHIPPED, and larger than the brief allowed for** — **9 `.text` words, 32 bits of 32**, not the masked check the lane planned. `w-restim`'s operand walk landed mid-lane and supplied the register fields |
| **2c. relocation/label half of the emit seam** | **FAILED.** Zero cells. Both fixtures have zero `.text` relocations and no other fixture was graded |
| **3. DISCLOSURE rows** | **SHIPPED** — `W-MID-1`…`W-MID-4`, in the commit that adopts them |
| **4. the cost note** | **SHIPPED** — `WB_MIDDLE_INTERFACES.md` §8, as a lower bound, and it moves ONE of the two halves |

**Late correction, found during the final proofread and worth the top of the
page**: the interface-1 grade is over the **region walk's first block**, which
is a *proper subset* of the function's tuple list — 7 rows of 16 on `mvp_add3`
at `sched1`. The 9 rows it omits are three parameter-in pseudo-ops and **three
`stw` home-slot stores**, so **PREREG P1.2 ("the three `B9` parameter loads
become zero tuples") is REFUTED at function scope** and true only of the region
view. `#1823`'s shape for the fifth time. It is now a test
(`the_region_view_is_a_strict_subset_of_the_function`) rather than a sentence,
and `WB_MIDDLE_INTERFACES.md` §3.5 carries it.

**Outcome word: `instrument`.** No fixture claimed, no census number moved, obj
bytes required-zero and unmoved. Predicted reach was 0 TUs and the reach was 0
TUs.

---

## 1. What was actually found

Three things, in descending order of how much they change the picture.

**(a) Interface 2 is short and interface 1 is the whole compiler.** The final
tuple order becomes `.text` through `FUN_10bf9f15` — two array lookups
(`0x10c3a578` base word, `0x10c39b18` encode form) and a 111-arm switch. The
`ret`/`blr` arm is literally one instruction (`or ebx,0x2800000`). Between the
IL token stream and the machine tuple list there is, by contrast, **nothing
exposed**: by the time any tap can observe a tuple, selection has already run.
That asymmetry is the lane's headline and it is what §8's re-pricing rests on.

**(b) The opcode space is one named table, and there is a trap beside it.**
`0x10b1b260`, stride 12, 0-based, `_last` at `0x295`. Immediately after it sits
a *second* table (`0x10b1d180`, stride 16) with its own index space, which
decodes tuple opcode `0x30f` as the trap instruction `tdlngi` in a function
whose source is `return a+b+c`. Read as a continuation it is silently plausible
and completely wrong.

**(c) The instruction-carrying tuples are in bijection with the emitted words,
at one site — on a population of THREE, fenced in `WB_MIDDLE_INTERFACES.md`
§6.1.** `w-restim`'s Probe C measured 19–22 tuples against 4–5
instructions and concluded there is no common coordinate. That stands. What is
added is that most of the ratio is bookkeeping — on `mvp_add3` the region-walk
payload is 36 rows across 7 blocks for a 3-word function, but the
whole-function list at `after0` is 8 rows of which 3 carry an instruction, and
those 3 are exactly the 3 emitted words in order. The projection is undefined in
the **region** coordinate and the identity in the **instruction** coordinate.

**This result is fenced by this lane and not by a reviewer.** §6.1 states the
population completely (three leaf, frameless, call-free, branch-free,
relocation-free `int` functions) and **names in advance where it is expected to
break**: on any **framed** function, because the final expansion switch rewrites
the prologue pseudo-op in situ into many words
(`WB_REGALLOC_FINDINGS.md` §4 item 2, `0x2f4`/`0x2f0` → `0x10bff95c`). The three
graded functions are precisely the ones that cannot exhibit that. If the check
is promoted to the fixture corpus it must be promoted as a **per-function ratio
measurement**, never as the equality assertion it is today — that assertion goes
red on the first framed function and would be read as an instrument defect.

---

## 2. The graded results

`cargo test -p c2-reference --test middle_interfaces`, five tests, all green
under `C2RS_REQUIRE_TOOLCHAIN=1`:

```
interface-0: 75 real-instruction tuples (36 at sched0, all machine opcodes),
             213 structural (all above the machine space),
             6 pre-lowering pseudo-op instruction tuples
interface-1: 3 functions, row-for-row equality, chains of 2/3/4 leaves
interface-2: 9 .text words reproduced from the post-final-schedule tuple order,
             32 bits of 32
scope:       sched1 function walk 16 rows (3 stw), region block 0 7 rows (0 stw)
             — the interface-1 grade covers the region view and NOT the 9 rows
             ahead of it
the_probe_levers_never_move_the_obj_at_this_lanes_profile ... ok
```

Fixtures: `mvp_add3.cpp` (`add3`), `mvp_two.cpp` (`add2`, `add4`),
`mvp_call.cpp`, `il_stmt_seq.cpp`. The three functions carrying the interface-1
and interface-2 grades are all **`Port=Match`** — byte-exact today — so the
lane's subject matter is inside the shipped class and nothing here can move the
required-zero.

**Two positive checks the tests refuse to run without**, because both are the
shape that has produced twelve recorded instances of absence-read-as-success in
this project:

* `armed_and_fired()` before any row is read — an unarmed run agrees with itself
  on zero rows.
* interface-0's `pseudo_pre > 0` liveness clause — without it, *"no pseudo-ops
  at `sched0`"* would pass on a corpus that has no pseudo-ops anywhere.

---

## 3. PREREG scored — 12 H · 4 M · 3 U

Full table in `WB_MIDDLE_INTERFACES.md` §7. The three that matter:

* **P0.1 REFUTED.** *"Every real-instruction tuple carries a machine opcode"* is
  false before the lowering band. Caught by the test, not by re-reading.
* **P2.2's form values were wrong in the prereg.** The registered arm indices
  `0x03`/`0x1e` came from reading the jump table's arms sequentially instead of
  through `form − 1`; the real forms are `0x31` and `0x37`. Corrected in public
  rather than folded in.
* **P1.2 REFUTED at function scope.** See the correction at the top of this
  page and `WB_MIDDLE_INTERFACES.md` §3.5.
* **P1.5 DID NOT RUN**, and it was the discriminating half of P1.4. So
  `+0xa & 0x1f` is documented as *consistent with* an operand size (4 on every
  4-byte tuple, 0 on every structural one) and the condition-code reading it was
  meant to refute is **not refuted**. Said in those words in §3.3 rather than
  claimed.

Two further U rows (P1.6, P2.4) are fixture coverage this lane did not buy.

---

## 4. What FAILED, in those words

**The relocation and label half of the emit seam: FAILED.** The brief asked for
*"the relocation/label emission at that seam"* and this lane produced **zero
cells** on it. Both graded fixtures have zero `.text` relocations, so the seam
was not merely under-measured — it was not observed at all. Nothing in
`WB_MIDDLE_INTERFACES.md` §5 says anything about relocations, and §5.6 says so
explicitly so absence does not read as coverage.

**Coverage of the encoder: 2 of 111 arms.** Enough to grade 9 words and not
enough to say anything about the other 109. §8 names the cheap follow-up (dump
all 111 arms and histogram `0x10c39b18` over the workload's emitted opcodes)
and declines to extrapolate, per `#1767`.

---

## 5. The peer collision, and what was deleted

`w-restim` landed (merge `b6fd2bf48`) while this lane was mid-flight, with a
strictly richer version of the tap extension this lane had already written and
greened: an eighth site `after0`, a whole-function walk from the function
record, and an operand walk over both operand lists reaching the assigned and
physical registers. This lane's own version — one `OPD` line per tuple carrying
three registers via the encoder's path (`operand+0x1c → +0x28`), env-gated,
with its own neutrality test — was **deleted** in the rebase resolution.

`c2host/stagetap.c` and `crates/c2-reference/src/stage.rs` were taken **verbatim
from master**; this lane's net delta in both is **zero lines**. Board **#3360**
carries the rule: when a peer's instrument subsumes yours, the merge hazard is
not the textual conflict, it is the one that *doesn't* conflict — two operand
walks over the same pointers would have merged cleanly, doubled the payload, and
left two register encodings (`n` and `n − 1`) in one stream.

The two encodings were then **reconciled rather than assumed compatible**:
`sym+0x08 → +0x1c` equals `operand+0x1c → +0x28` plus one, confirmed by
construction (`0x0c` where the obj has `r11`).

---

## 6. Four defects found in this lane's own code and record, recorded

* **`IMAGE_SYMBOL.Value` is at `+8`, not `+4`.** The first `.text` slicer read
  four bytes of a mangled name as a section offset and panicked on an inverted
  range. It also could not have used
  `ObjImage::text_comdat_functions_with_bytes` at all: these fixtures' `.text`
  is `0x60400020`, with **no `LNK_COMDAT` bit**, so the COMDAT walk correctly
  returns nothing — and a test built on it would have graded **zero functions
  while printing a pass.**
* **The subset decoder's stopping rule was "the first byte I do not
  recognise".** That cannot distinguish *finished* from *gave up*, and reporting
  the second as the first is how a subset decoder manufactures a green. It now
  stops at the epilogue label token and **refuses** a body whose label it never
  reaches.
* **§4 compared a region view on one side to a function view on the other.**
  Before §3.5 existed, "the lowering band, watched" read as a one-tuple rename.
  Against the function walk on both sides it is three things: the whole
  six-tuple parameter-home-slot prologue **deleted**, one pseudo-op rewritten
  into a machine opcode, and the two `add`s untouched. Two of the three are
  deletions — the half a port cannot see from the obj at all.
* **§3.5's own correction undercounted its own omission.** The first draft said
  *"those 9 are three `0x2f8` and three `stw`"* — which is six. The other three
  are structural markers. All sixteen rows are now listed. Getting the count
  wrong in the paragraph whose entire subject is a count that was got wrong is
  worth a line here rather than a quiet fix.

---

## 7. Gate, suite, and the workload's required-zero metrics

### 7.1 The delta, stated first, because it bounds what the evidence has to do

`git diff --stat master...HEAD` over `crates/`:

```
crates/c2-reference/tests/middle_interfaces.rs | 1011 ++++++++++++++++++++++++
```

**One file, and it is a test.** No `crates/` library or binary source, no
`c2host/` source, no fixture, no `scripts/` change, no shipped rule, no refusal
predicate. `c2host/stagetap.c`, `crates/c2-reference/src/stage.rs` and
`crates/c2-harness/src/cli/stage.rs` are **byte-identical to master** — see §5.
So the required-zero metrics are unmovable by construction, and the runs below
are no-regression controls rather than the thing that makes the claim.

### 7.2 Suite

`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --no-fail-fast`:

```
TOTAL PASSED: 1770    TOTAL FAILED: 0
"SKIP: toolchain absent" occurrences: 0
```

**The delta against `w-restim`'s 1,765 is exactly +5**, and all five are this
lane's: `crates/c2-reference/tests/middle_interfaces.rs`, one target, five
tests. `1765 + 5 = 1770`.

Confirmed at the tip, over a frozen `crates/` tree, with the run's own log at
`/tmp/w-ildecode-suite2.log`. One repair was needed on the way and is worth
recording because it is a tripwire a docs-shaped lane will hit again:
`crates/c2-harness/tests/rung_registry.rs::rung_index_is_generated_and_current`
failed until `scripts/gen_rung_index.sh` was re-run. A new rung file is a code
change as far as that test is concerned.

### 7.3 The workload's required-zero metrics — UNMOVED

878-TU scan at c2-rs tip **`c6bd560b8ff1` (clean)** and workload stamp
**`2f666acc8aa2` (clean)** — the same workload stamp `w-restim` and `ir1`
published against. Log: `work/w-ildecode/tip_scan.log` (gitignored).

| metric | dispatched | this tip |
|---|---:|---:|
| `match` / `mismatch` | 26 / 0 | **26 / 0** |
| `fnbyte-exact` | 35,894 | **35,894** |
| `codegen-gap` / `vocab-gap` / `capture-fail` | 0 / 844 / 8 | **0 / 844 / 8** |
| `factor-c` | 170 | **170** |
| EMITTED CENSUS | 39,344 / 162,147 | **39,344 / 162,147** |

### 7.4 Gate — and the first run was DISCARDED by the gate's own check

`C2RS_REQUIRE_TOOLCHAIN=1 scripts/gate.sh --jobs 8`, at tree `bc4b10956`,
graded tree `5dfe54e43296` (761 files under `crates fixtures scripts`,
content-hashed) — **identical at both ends of the run, and the run's footer
carries no `THE TREE MOVED` line.**

**A first gate run exists and is NOT quoted anywhere.** It was launched while
this lane was still editing a test file, and its own footer says exactly what
that costs:

> *THE TREE MOVED UNDER THIS RUN — it began at `87c07e649920` (761 files) and
> ended at `5dfe54e43296` (761 files). The verdict above was produced partly
> from each, so it is evidence about NEITHER tree.*

That run printed `GATE: PASS` with 0 mismatches everywhere. **It is still not
evidence**, and it is recorded here rather than quietly dropped, because a green
that the instrument itself disclaims is exactly the green a tired reader lifts.
The block in §7.5 is the re-run, over a frozen tree, and nothing else.

`hatch-red` reports `REFUSED HATCH-STALE` — a property of a fresh worktree's
`work/w-hatch/` scratch and not of this lane; `gate.sh` exits 0 on `REFUSED` and
forfeits the unqualified headline, which is the designed behaviour and is why
the headline below reads `PASS (HATCH-RED REFUSED)`.

### 7.5 The verdict block, verbatim

```
lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
graded: 6948 fixture-verdicts across all lanes
sweep:  PASS — 19556 of 19556 selected cases reached, 19460 GRADED by the
        oracle (96 ungraded: no reference obj), 0 mismatch (corpus 19556)
cross:  PASS — 90424 of 90812 selected cells graded, 0 mismatch (product 90812)
debug:  PASS — 18 of 18 lanes through a DEBUG-profile c2rs,
        6948 fixture-verdicts, match 2423, 0 mismatch, 0 PANIC

GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus,
  the sweep graded 19460 of 19556 generated cases and the cross graded
  90424 of 90812 case-lane cells, with 0 mismatches anywhere
  ...
graded tree: 5dfe54e43296  (761 files: crates fixtures scripts, content-hashed)
```

### 7.6 The `c2rs` binary is bit-identical across this lane's whole span

The 878-TU scan in §7.3 ran at `c6bd560b8ff1`, and `crates/` changed once after
that (`8a3e7f3e2`, the fifth test). That delta cannot reach the scan, and the
evidence is a hash rather than an argument: the release `c2rs` is
**sha `7f81b4355dc2`** at `a9fc1cd85`, at `bc4b10956`, and rebuilt at the tip —
three builds spanning every `crates/` commit this lane made. A test file under
`crates/c2-reference/tests/` links into no binary that `c2rs gap` runs, and the
sha says so.
