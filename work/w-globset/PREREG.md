# PREREG — lane `w-globset`, wave 20 L3

**Committed before the first line of `crates/` code was written.** Base
`c5bfe89d9`, branch `worktree-agent-a379304d70d843b85`. Brief
`docs/WAVE20_BRIEF_2026-08-29.md` §2 L3. Board block **#3831**–**#3837**.

**Kind:** construct rung. **Outcome word, one of:** `built` or `FAILED`.
`Fixtures: none`. `Census: +0`. **Required-zero byte delta.**

---

## 1. What this lane builds

`[globregs]`'s **order** half is settled and obj-confirmed — definition order,
`[O]` on 42/42 cells with seven rivals refuted (`P_GLOBREGS` §7.1, board
`#3774`). The **candidate SET** half is read (`w-globarms`, `#3808`–`#3810`)
and is expressed nowhere in the port. `WB_GLOBARMS_FINDINGS.md` §7 states the
handoff in one sentence and this lane takes it literally:

> **Anyone porting the candidate set: the parameter to expose is linkage
> class, not variable kind. Kinds 4 and 5 are linkage 1 and 3, the
> no-COFF-record classes; everything with a COFF record is 7/8/9. The escape
> flag `sym+0x05 & 2` is the second parameter and it is per-symbol, not
> per-function.**

So: a new module `crates/c2-core/src/codegen/globset.rs` — c2's candidate-set
policy (`FUN_10b550e5` gate A, `FUN_10bd7d24` gate B, and the kind map
`FUN_10bd2913` that feeds both) written as executable code with **five named,
settable parameters whose defaults reproduce the read**:

| # | parameter | default (= the read) | the rival(s) it makes runnable |
|---|---|---|---|
| P1 | `KindMap::table` — the 8-entry jump table at `0x10bd2a9f`, indexed by `([gl+0x37]>>0x15)&7` | `NullSlot, Kind(4), StorageBits, Kind(5), StorageKind, AliasBit, StorageBits, StorageKind` | a table read at the wrong stride; entry 0 treated as reachable |
| P2 | `KindMap::gl_kind` arms — the `dec`-chain at `0x10bd2926` over `[gl+0x30]` | 1→table, 2→4, 3→`0xb`, 4→`0xa`, else→`0xa` | — |
| P3 | `GateA::bounds` — the six kind comparisons `0x10b5511a`…`0x10b5514e` | `0x10`, `3`, `3`, `5`, `6`, `8`, `!=0xa` | A6's bound moved 5→7 (the mutant `grade_globarms.py --selftest` itself plants) |
| P4 | `AliasingPolicy` — what sets `sym+0x05 & 2` | `EscapesToOpaqueCallee` | `AddressTaken`, `Never`, `Always` |
| P5 | `TypeClassPolicy` — the 30-byte gate-B table at `0x10b18b28` | not-promotable `{0x00,0x12,0x13,0x18,0x1d}` | `AllPromotable`, `NonePromotable`, a one-byte stride shift |

**P4 is the second half of the brief's "A6 needs TWO parameters, not one"**,
and it is a property of a symbol **group** (the leader's `+0x0c` chain, sole
setter `FUN_10bd2db7`), not of a symbol — the API takes the group and the
type carries the distinction in its name.

**Not "address taken".** `gb_addr_local` (`int *q = &x` with no escape) is
PROMOTED, so `AddressTaken` is a *refuted rival* and ships as one.

## 2. What this lane does NOT build, registered in advance

* **No `ported` numerator** — decision 21 §4, `#3809`, `#3505`. The twelve arms
  look like a denominator and are not one (6/12 are `CONSTR` for one shared
  structural reason; A6/A8 cover every symbol a compiland declares while A1
  covers one record per compilation). §5's power table is a **separating-power**
  measurement over an existing obj population, published including its zeros;
  it is **not** a coverage ratio and no percentage of c2 is claimed from it.
* **No production caller.** `PortC2::build` does not reach this module and must
  not. The port has no symbol arena, no `.gl` records and no tuple list; the
  policy consumes a supplied descriptor exactly as `FUN_10b550e5` consumes a
  supplied slot. So the byte delta is **required zero by construction**, which
  is precisely why §4's fail axis and §3's refusal-domain check carry the grade.
* **No register allocator** (decision 20 §2, `P_REGALLOC` §7: F5 is not
  separable from F0).
* **No new count-bearing `scripts/gate.sh` row** (`#3691` — a 22nd makes
  `gate_identity_diff.sh` exit 2 for every lane). Enforcement is `cargo test`.
* **No edit to `P_INLINE.md`, `CLAUSES.tsv`, `P_DAG.md`, `clause_table.rs`,
  `STATUS.md`, `rungs/INDEX.md`, or any peer's board block.**
* **No new read.** Every address here is `w-globarms`'/`w-globobj`'s and is
  cited, never re-taken.

## 3. THE REFUSAL-DOMAIN CHECK — `#3723`, registered before the build

`#3723` measured that a required-zero byte delta **passes a real emit
widening**: `w-regsel`'s control C6 opened the allowed register set to
`r0..r31` and 471/475 tests still passed, gate green, identity diff 0 lines.
A candidate SET is exactly the class that defect lives in, so:

* A surface **`globregs.candidate_set`** is registered in
  `c2_core::surface::SURFACES`, site `codegen/globset.rs`, with a domain over
  **(`.gl` kind × linkage 0..7 × storage bits × storage kind × alias bit)**,
  **(kind `0x00`..`0x11` × the two conditional bits × escape)**, **(kind ×
  type class `0x00`..past `TYPE_CLASS_MAX`)** and the composed
  **(linkage × type class × escape)** verdict. The domain reaches values **no
  fixture and no gate row can reach** — the module has no production caller at
  all, so *every* point is past the corpus.
* `crates/c2-core/src/surface/DOMAIN.txt` is re-blessed in the same commit, so
  the addition is a reviewable text diff.

**The registered control, run and recorded in `work/w-globset/CONTROLS_RED.txt`
before any verdict is quoted (`#3336`):**

| # | perturbation | prediction |
|---|---|---|
| **C1** | `AliasingPolicy::Always` as the module default | byte delta **0**, gate **PASS**, identity diff **0 lines** — and `DOMAIN.txt` **moves > 100 lines** |
| **C2** | A6's kind bound `5` → `7` (the mutant `grade_globarms.py` plants in the image) | domain moves; **and §4's fail-axis test goes RED** against the image-decoded table |
| **C3** | `TYPE_CLASS_MAX` widened by one step | domain moves, which is what makes naming it a `guards` entry a **true** coverage claim (`#3746`: two of seven original guard entries were false and moved zero lines) |
| **C4** | the whole domain generator returning `Vec::new()` | E3's cell floor goes RED — a check over zero cells is green and says nothing (`#3470`) |

**Falsifier for the headline pair.** If C1 moves **0** domain lines, the
surface does not reach the parameter it claims to cover and the lane says so
rather than shipping the claim — that is `#3746`'s finding applied to itself,
and it caught a wrong **non**-coverage claim as readily as a wrong coverage one.

## 4. THE FAIL AXIS — non-empty, and it is a measurement, not a name

`rung_registry.rs` asserts the header field's **presence**; the module doc
comment records that presence is not measurement. This is the measured form.

> **Fail axis: agreement with the tables another lane's grader decoded out of
> `c2.dll` itself, parsed from `work/w-globarms/GRADE.txt` at test time rather
> than transcribed here — the 17-row kind→arm simulation, the 8-row linkage
> table, the 5-row `.gl` kind map and the gate-B not-promotable set — plus the
> measured separating power of the 38-cell + 46-cell obj populations over this
> module's non-default parameter values. Both halves can go RED while every
> byte, every gate row and every identity-diff line is unchanged.**

Why this is not circular: the two `GRADE.txt` files are **produced by a
different lane's instrument from the pinned image and the real toolchain**, and
this module did not author them. If P1/P3/P5 are transcribed wrongly the test
is red though nothing else moves. If the files are absent or parse to fewer
rows than expected the test **fails** — it does not skip (`#3470`: a screen
over zero rows is green).

**Registered predictions, before the code:**

* **F1** — the kind→arm map reproduces all **17** decoded rows (`0x00`…`0x10`).
  p = 0.90.
* **F2** — the linkage table reproduces all **8** decoded rows. p = 0.90.
* **F3** — the composed verdict agrees with all **38** `w-globarms` cell
  verdicts **on the subset whose symbol descriptor is fixed by the cell's own
  construction** (the five `gb_*` escape cells, `ga_int`, `ga_vol`,
  `ga_escape`). p = 0.80. **The A8 cells (`ga_extern`, `ga_fstatic`,
  `ga_lstatic`) are NOT scored** — `w-globarms` §1.3 refuses them as
  CONFOUNDED and this lane inherits that refusal rather than banking three free
  hits.
* **F4** — the 84 obj verdicts refute `AliasingPolicy::Never` and
  `AliasingPolicy::Always`, and **also refute `AddressTaken`** (via
  `gb_addr_local` alone). p = 0.75 on the third.
* **F5 — a registered ZERO.** The obj populations refute **none** of P1's
  linkage arms 4, 5 and 7, and **none** of P5's type-class table beyond the
  classes the two grids' C++ types reach. p = 0.85. **Publishing this zero is
  the point** (`#1236`: "my test passes" and "my test can tell two rules apart"
  are different claims).

## 5. Grading

* `scripts/gate.sh --jobs 16 --require-graded` — unqualified `GATE: PASS`,
  verdict **line** read, never the exit code.
* `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` —
  both the target count and the pass count.
* `scripts/expr_sweep.sh`.
* **Identity diff of per-lane gate counts, base vs tip — required 0 lines.**
* `DOMAIN.txt` line count before and after, and the C1 move.
* Every control watched RED **before** any verdict from it is quoted, in the
  same environment (`#3219`/`#3231`), with the restore `touch`ed and the
  rebuild verified (wave 18's `cp`-mtime trap).

## 6. What would make this lane say FAILED

* The byte delta is non-zero, or the identity diff is not 0 lines.
* C1 moves 0 domain lines (the surface does not reach its own parameter).
* F1 or F2 misses — the transcription is wrong and the module is a fiction.
* No control is watched red.
* The module acquires a production caller.
