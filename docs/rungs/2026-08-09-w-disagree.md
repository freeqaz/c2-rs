# w-disagree — the agreement check ran over a population that could not contain the failure; it runs over 19,556 generated cases now and found three live census over-claims on the first try

    Tag:       w-disagree
    Slug:      w-disagree
    Date:      2026-08-09
    Fixtures:  none — this rung repairs an instrument and narrows one reader clause
    Census:    711,485 / 2,463,443 unchanged (28.88 %), **+0** on the 878-TU
               workload — the narrowed clause has zero population there. TU match
               **10 → 10**, mismatch **0 → 0**, all 139 `gap-metric` lines
               byte-identical. The narrowing DOES bite on the generated corpus:
               in-class **14,299 → 14,275** over 19,467 captured cases.
    Record:    this file; prereg `work/w-disagree/PREREG.md`, committed before the
               first line of `crates/`; mutation battery
               `work/w-disagree/mutate.sh`; evidence under `work/w-disagree/out/`.
    Lane:      w-disagree, worktree branch `wt-w-disagree`, built off master
               **`04727f37`** (the merge of peer lane `w-midrun`). Every number
               below is measured at both ends in this tree, never carried.
    Ships:     one narrowed reader clause, one repaired standing test with a
               second population and three positive checks, one mutation battery.
               Board rows **#1304**–**#1311**.

---

## 1. The result

> ### **A test whose population cannot contain the failure is not a test.** `census_gate.rs` read `1` disagreement on master while **42 of `w-midrun`'s 94 grid cells were live** (board #1275), two of them cells `w-carrier` had committed to the repo days earlier. It was green because its population was the **286 hand-written fixtures** and not one of them spells `h->m.f = &h->m;`.

> ### **The instrument now runs over the GENERATED SWEEP CORPUS as a second population — 19,556 cases, 19,467 captured — and prints a count of DISCRIMINATING CELLS.** On its first run against an unmodified port it found **124 disagreements in the packed lane and 127 under `/Gy`, in THREE families**, where the fixtures hold one. All three were live on master. **None is a mis-emit**: `mismatch` is 0 at both ends and none of these bodies has ever reached an obj — what is wrong is the published numerator.

> ### **Board #1283's residue of four is 2 closed and 2 NAMED.** The `twop` pair closes by a reader clause that restates two of the emitter's three address clauses. The `mix` pair is board **#1306** and is deliberately left in class: refusing it would leave the mixed-kind allocation rule with no reachable input from **any** spelling, which is board #1291's shape.

| | base `04727f37` | tip |
|---|---|---|
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 10 / 0 / 0 / 861 / 7 | **10 / 0 / 0 / 861 / 7** |
| `gap-metric` block, 139 lines, `diff`ed | — | **EMPTY** |
| census/gate disagreement, 878-TU scan | 0 | **0** |
| census/gate disagreement, **fixtures** (packed / `/Gy`) | 1 / 12 | **1 / 12** |
| census/gate disagreement, **generated** (packed / `/Gy`) | **148 / 151** | **124 / 127** |
| discriminating cells, fixtures / generated | 1,692 / 14,299 | 1,692 / **14,275** |
| GRID M (94 cells) | 81 match / 0 mismatch, 4 disagree | **81 match / 0 mismatch, 2 disagree** |
| workspace tests | 1,153 passed / 0 failed / 36 targets | **1,153 passed / 0 failed / 39 targets** |
| `git grep -c '#[test]'` over `crates/` | 1,154 | **1,155** |
| `peerkeys.py` families that vanished | — | **0** |

---

## 2. The residue of four

### 2.1 Two close — the direct spelling's interior-address DOMAIN

All four cells reproduce at base at the workload's own `/GR /O1 /Oi /EHsc`, each
`1/1 functions in class` with `census/gate DISAGREEMENT: 1`, and all four report
**the same** emitter refusal. They are **two families under one message**:

| cells | class | what it is |
|---|---|---|
| `t_dl`, `t_dc` | **twop** | two *distinct* interior addresses, `&mBlk` and `&mAlt`. Single-kind |
| `x_dl`, `x_dc` | **mix** | one interior address beside a literal — the mixed-kind allocation rule |

`try_parse_store_run` admitted an `AddrOf`-valued statement into a run on an
argument its own comment states: *"`c2_core` sees a four-op group and declines"*.
**`w-midrun` retired that argument** — `parse_simple_gpr_run` reads exactly that
group now and `scheduled_gpr_run` emits one `addi rD,rBase,off` for it, inside a
domain narrower than the reader's. So the reader restates two of the emitter's
three address clauses, keyed on `(value base token, displacement)` — the identity
`Prod::Addr` uses, so `&r` and `&h->mBlk` are one address and a run c2 emits is
not refused for spelling it twice:

* **two or more distinct addresses** — `scheduled_gpr_run`'s `kinds.len() != 1`,
  and `bind_run_ops`' `addrs.len() != 1` in the bind spelling;
* **displacement 0** (c2 materialises nothing; the value *is* the base register)
  or **past 16 bits** (`addis`+`addi`, two words where a producer is one slot).

```text
  t_dl / t_dc   codegen-gap disagree=1  ->  vocab-gap disagree=0  (expr-op-0x27)
  x_dl / x_dc   unchanged: in class, refused by the emitter, and now COUNTED
```

**Nothing in domain moved.** GRID M is 81 match / 0 mismatch at both ends and
**exactly two of its 94 rows differ**, both of them the `twop` pair. Fixtures
286 pass / 0 fail. On the workload the clause has **zero population** — 139
`gap-metric` lines byte-identical.

**On the generated corpus it is worth 24.** The instrument in §3 measures the
same clause over 19,467 cases: in-class `14,299 → 14,275`, and the
interior-address disagreement family `134 → 110`. Two GRID M cells and
**twenty-four generated ones** — which is the whole argument of this rung
restated as a number: the same clause, measured over two populations, is worth 2
or 24 depending only on what you point it at.

### 2.2 Two are NAMED and left open — board #1306

`x_dl` / `x_dc` could be refused in one more line. They are not, and the reason
is measured rather than argued:

* `bind_run_ops` **already** refuses the bind spelling as
  `STORE_RUN_BIND_MIXED_KIND`;
* a direct-spelling refusal beside it leaves the **mixed-kind allocation rule**
  (#836 wrong-on-0 over 81 cells, #868's narrow lift refuted 12/36, #1134's
  clause 1 refuted on this very mix, #1265's `H-DERIV`) with **no reachable input
  from any spelling**;
* that is board **#1291**'s shape exactly — a published cause that can never
  fire — and the lane that then shipped the rule would move **zero bytes** and be
  told by two source comments that it should have worked.

`w-lineage` owns `alloc.rs`, the roots carrier and `STORE_RUN_BIND_MIXED_KIND`,
and none of the three is touched here. #1283's own instruction is that this row
*"closes by ACCEPTING these four, not by refusing them"*; `w-mixkind` landed
without the rule (#1265 is published and **not proposed**), so it is re-filed
rather than forced.

**What changed is that they are no longer invisible while they wait.** §3 counts
them — and #1306 is **110 functions in the generated corpus**, not 2.

---

## 3. The instrument

### 3.1 What "discriminating" means, and what it does not

A cell is **discriminating** when both verdicts exist and are reached
independently: the census calls the function in class (so the port's answer is
compared at all) **and** the port's answer is produced by
`codegen::function_gate` running `select_function` on a real `IlFunction`. Those
are the cells in which a disagreement *can* appear, and the count of them is
printed on every run, per population and per linkage mode, together with the
number of distinct census shape keys they span.

Three **positive** demands, in an order chosen so each is reachable:

```text
  captured > 0                    POPULATION EMPTY
  discriminating > 0              NO DISCRIMINATING CELLS
  discriminating >= floor         DISCRIMINATING CELLS COLLAPSED
  distinct shape keys >= floor    DISCRIMINATING BREADTH COLLAPSED
  then, and only then, the disagreement pin
```

None enumerates a way the run can be empty. Each has a message no other one can
produce — §4 drives all five and shows the messages.

**The honest limit, stated here rather than left to be found.** On both corpora
today `discriminating == in_class` **exactly** (1,692 = 1,692 and 14,275 =
14,275), because `census_functions` derives its in-class verdict from the same
`shape_to_function` call the gate slot carries. The separate counter is a claim
about what *could* diverge — a later gate producing `InClass` with an `Err` gate
slot — not about what does. Mutation C in §4 moved both together and this rung
says so.

### 3.2 What it found

| population | captured | in class | discriminating | shape keys | disagreements packed | `/Gy` |
|---|---:|---:|---:|---:|---:|---:|
| `fixtures/cpp` | 286 | 1,692 | 1,692 | **35** | **1** | **12** |
| generated (`sweep_gen.py`) | 19,467 of 19,556 | 14,275 | 14,275 | **31** | **124** | **127** |

**Three families, all live on master, none of them a mis-emit:**

| substring | packed | fragments | owner |
|---|---:|---|---|
| a store run with an interior address BESIDE another producer | **110** | `88-store-run-call` 64, `89-store-run-live-arg` 46 | board **#1306** — this lane's own named residue, at 110 rather than 2 |
| a store-run-before-a-call whose run materialises nothing | **12** | `88` 2, `89` 10 | board **#867**'s slot rule → **#1307** |
| a bitwise or shift operand that is not a bare register | **2** | `68-shift-ops` | `cmp_shift_or`'s immediate forms → **#1308** |

Representative cases, read off the generated corpus:

```cpp
// 88-store-run-call-0793.cpp — the 110-family, and it is xboxheap.cpp's own shape
H::H(unsigned initSize, unsigned size) {
  mCount = 0;                        // a literal
  mListHead.mNext = &mListHead;      // beside an interior address
  mListHead.mPrev = &mListHead;
}
// 88-store-run-call-0025.cpp — the 12-family
H::H(unsigned initSize, unsigned size) { mSize = size; Alloc(initSize); }
// 68-shift-ops-0217.cpp — the 2-family
int f(int a){ return q(a << 1); }
```

**`mismatch` is 0 at both ends and no wide run this lane made produced one.**
Every one of the 124 is a *refusal*: the census counts the body in class and
`PortC2` declines it, so nothing was emitted and nothing is wrong in an obj. What
is wrong is the **published coverage numerator**, by 124 functions on this
corpus, and it has been wrong for as long as those fragments have existed.

### 3.3 The instrument that already existed, and why it is not this one

`scripts/sweep_mode.sh` runs the same generated corpus through `c2rs gap` at two
`/O1` profiles and ratchets `census/gate DISAGREEMENT` against
`C2RS_SWEEP_MODE_MAX_DISAGREE=3`. It is genuine prior art and it was read before
a line was written (PREREG §0). Three things it does not do:

1. **Nothing runs it.** It is not a row of `scripts/gate.sh` — #299's shape,
   still open for this script.
2. Its baseline `3` was measured **2026-08-04**, nine merges ago.
3. **Its three cases do not appear here at all, and this lane's 124 do not appear
   there** — board **#1310**. The seam is the difference: `sweep_mode.sh` asks
   `IlBundle::functions()` a **whole-TU** question (the TU-level label-counter
   stride gate is what refuses its `70-framed` trio); `census_gate.rs` asks
   `census_functions()` + `function_gate` a **per-function** one. Two instruments,
   one name, disjoint findings. Quoting either as *"the census/gate
   disagreement"* is quoting a phrase that names two quantities.

### 3.4 Cost, and the one it caught in itself

**+83 s** on `cargo test --workspace --release` (fixtures 14 s, generated 83 s at
16 capture threads; ~24 ms per `capture_il`). The prereg's decline floor was 10
minutes, so no stride was needed and the wide lane grades the **whole** corpus.

**The wide lane reported `ok` in 0.0 s on its first run, having graded nothing.**
`generate_wide` parsed the generator's *second* whitespace token instead of the
number before `cases`, the parse failed, the `None` flowed into the caller's
`let … else { return }`, and the test passed. That is **absence read as success
inside the fix for absence read as success**. The repair is structural, not a
better parser: `None` now means *and can only mean* "python3 is absent"; a corpus
that generated and could not be counted is a `panic!`.

---

## 4. THE MUTATION BATTERY — every check driven red on purpose

`work/w-disagree/mutate.sh`, five mutations, each applied with `python3 -c`, run,
and reverted by a trap. Board **#1236**: a guard nobody has seen fire is not a
guard. `docs/GAPS.md` §7: an earlier guard that fires first makes every later
assertion unreachable — the lane registry's count floor tripped and its `/EH` and
`/Oi` assertions never executed. So **each mutation holds every earlier check's
quantity at its measured value** and moves only the one under test.

| mut | what it breaks | earlier quantities | first line of the panic (verbatim) |
|---|---|---|---|
| **A** | `function_gate` refuses any function with a formal | captured 286, in-class 1,692, discriminating 1,692, keys 35 — all unchanged | `assertion left == right failed: census/gate disagreement changed with fn_level_linking=false (1692 functions in class across the fixture corpus, 1692 of them discriminating).` …and, on the wide lane, `A NEW census/gate refusal family appeared over the generated corpus (fn_level_linking=false):` |
| **B** | every `Toolchain::capture_il` fails, toolchain still resolves | — (this is the first guard) | `POPULATION EMPTY [fixtures, fn_level_linking=false]: not one source in this population produced IL, so the agreement check graded nothing and would have passed by absence. This is the instrument, not the port.` |
| **C** | `shape_to_function` refuses everything | captured 286 | `NO DISCRIMINATING CELLS [fixtures, fn_level_linking=false]: 286 sources captured and 0 functions censused in class, but codegen::function_gate reached its own verdict on ZERO of them — so no cell in this run could have produced a disagreement, and a disagreement count taken over it says only that the run was vacuous. This is the instrument, not the port.` |
| **D** | it refuses half (odd-length names) | captured 286, discriminating 903 > 0 | `DISCRIMINATING CELLS COLLAPSED [fixtures, fn_level_linking=false]: 903 of 903 in-class functions reached codegen::function_gate, below the floor of 1200. …` |
| **E** | `FnVerdict::key` returns one constant | captured 286, in-class 1,692, **discriminating 1,692** | `DISCRIMINATING BREADTH COLLAPSED [fixtures, fn_level_linking=false]: the 1692 discriminating cells span only 1 distinct census shape keys, below the floor of 25. … Keys: ["MUTATION-E-one-key"]` |

**Five mutations, six distinct failure messages, zero collisions.** Mutation E is
the one that matters most: it holds the cell count *exactly* at 1,692 and moves
nothing but the number of distinct shape keys, which is the quantity that would
have been the fixture corpus's tell in the first place.

**Mutation C's honest defect**, recorded rather than smoothed: it drove
`in_class` to 0 as well, because the census's in-class verdict comes from the
same `shape_to_function` call. So C demonstrates that the *assertion* fires and
that it fires **before** the disagreement pin; it does not demonstrate that
`discriminating` and `in_class` are separable in today's code. §3.1 says they are
not.

---

## 5. Prereg scorecard

| | registered | outcome |
|---|---|---|
| **PRED-R1** | 2 of 4 close, 2 named | **HIT** — `t_dl`/`t_dc` closed, `x_dl`/`x_dc` are #1306 |
| **PRED-R2** | closing the twop pair costs 0 on the workload | **HIT** — `match` 10 → 10, 139 `gap-metric` lines byte-identical |
| **PRED-R3** | no GRID M in-domain cell moves | **HIT** — 81 match at both ends, exactly 2 of 94 rows differ |
| **PRED-I1** | fixture discriminating cells **400–1,200**, keys **≤ 30**; expected to be *high* on keys | **MISS, both, and in the direction NOT registered** — 1,692 cells and 35 keys. The fixture corpus is bigger and broader than this lane assumed while still being blind to the construct that started the rung, which is the sharper version of the point |
| **PRED-I2** | wide lane ≥ 6,000 cells, ≥ 25 keys, **≥ 10× the fixture lane** | **HIT on the floors, MISS on the ratio** — 14,275 cells and 31 keys, but only **8.4×** the fixture lane, and **four FEWER** shape keys. What the wide corpus buys is breadth of *construct*, not bulk or key count: 8.4× the cells, **124×** the disagreements |
| **PRED-I3** — *the one registered to be wrong about* | packed lane reads **3** on the wide corpus, direction of error **UNDER** | **MISS, under by 41×, and the direction was called.** 124, in three families, **none of them the predicted trio** (§3.3). Registering "0 new disagreements" would have been wrong by the same 124 |
| **PRED-I4** | `/Gy` ≥ 150 on the wide corpus, dominated by pooled constants | **MISS, in the opposite direction** — **3**, against 11 over 286 fixtures. Board **#1311**: a hand-written corpus can be the *denser* one on any axis somebody wrote it for |
| **PRED-I5** | `mismatch` stays 0 | **HIT** — 0 at both ends, no wide run produced one |
| **the mutation** | 3 mutations, 3 distinct messages | **HIT and exceeded** — 5 mutations, 6 messages, and the registered ordering hazard was real: C would have hit the disagreement pin if the discriminating check had been placed after it |

**The weakest part of this lane** is that its two most confident predictions
about its own instrument (PRED-I1, PRED-I4) were both wrong, one in each
direction, and the prediction it registered as *likely wrong* (PRED-I3) was the
one that paid. That is the argument against sizing an instrument before pointing
it at anything, and it is written here rather than in a footnote.

---

## 6. What the next lane inherits

| | |
|---|---|
| **`census_gate.rs` grades 19,467 generated cases as well as 286 fixtures**, and prints discriminating cells + shape keys per population | §3 |
| **three live census over-claims, 124 functions**, none of them a mis-emit: #1306 (110), #1307 (12), #1308 (2) | §3.2 |
| **#1306 is 110, not 2** — whoever lands the mixed-kind allocation rule closes it by ACCEPTING, and now has a population to grade against | §2.2 |
| the two instruments called *"the census/gate disagreement"* find **disjoint** things | §3.3, #1310 |
| five positive checks, each shown to fire, with distinct messages | §4 |
| `discriminating == in_class` today; the counters are separate for a divergence that has not happened | §3.1 |
| `scripts/sweep_mode.sh` is still not a row of `scripts/gate.sh` | §3.3 |

### 6.1 Peer keys and shared surfaces

`work/w-splice/peerkeys.py` over the base and tip `--jsonl`: **FAMILIES THAT
VANISHED: 0**.

* `crates/c2-core/src/codegen/alloc.rs` — **untouched**, not one line.
* `STORE_RUN_BIND_MIXED_KIND`, `STORE_RUN_BIND_ADDR_PRODUCER`,
  `STORE_RUN_BIND_MULTI_PRODUCER` — **untouched in name and in meaning**; the new
  clause is in the *direct* spelling's production and adds a `return None` beside
  the existing ones. No existing key changes what it means, so no reader of one
  is a candidate erasure (`w-relo`'s `FnByte::Exact` hazard).
* `crates/c2-core/src/codegen/coff.rs` — **never opened**.
* `crates/c2-il/src/func/body/shapes/control_flow.rs` — untouched (`w-bd`).
* `Prod::Addr`, `scheduled_gpr_run`, `bind_run_ops`, `FnVerdict::key`,
  `codegen::function_gate`, `codegen::select_function` — **read, not changed**.
* The mutation battery edits three files and reverts them under a trap; the tree
  is verified clean by `git status --short` at the end of every run.

**The merge question, distinct from "did git report a conflict":** the reader
clause and `scheduled_gpr_run`'s clause 1 are **one fact stated twice**, by
design (the emitter restates every reader clause as a backstop). A lane that
widens either must widen both, and `census_gate.rs`'s wide lane will report the
*reader-wider-than-emitter* direction. It will **not** report the other
direction, and that asymmetry is what makes #1306 a real hazard rather than a
bookkeeping note.
