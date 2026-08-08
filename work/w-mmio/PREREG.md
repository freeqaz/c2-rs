# w-mmio — PRE-REGISTRATION

Committed **before the first probe `.cpp` is generated and before the first
`cl.exe` invocation**. Lane `w-mmio`, worktree branch `wt-w-mmio` off master
`851938df`.

The brief: `?mmioGetInfo` (84 B, `src/xdk/nuispeech/mmio.cpp`) is the cheapest
codegen rung on the board and needs **zero** new encoders (`w-clear` §1). Its
first refusal is board **#275**, the entry-block park. `w-clear` characterised
the park's rule off a 54-cell grid and **deliberately did not implement it**,
because **exactly one cell** (`[2,0,1]`) discriminates it — board #260's
warning about fitting a rule to the point that produced it.

**The central task, as briefed:** *find a population that discriminates the
park's rule at more than one point, or decline.*

---

## 0. What I read before writing this, and the one correction I am carrying

`docs/rungs/2026-08-08-w-clear.md` (whole), `2026-08-08-w-loop.md` §1–3,
`docs/BOARD.md` rows **#275**, **#260**, **#1413**, **#1414**, **#1415**,
`crates/c2-core/src/codegen/frontier_bytes.rs`,
`crates/c2-core/src/codegen/calls.rs` (`permute_args_parts`, `call_seq_parts`),
`crates/c2-il/src/func/body/shapes/calls.rs` (the three refusals),
`scripts/gt_argperm.py` (`predict_pure` — the **unguarded** model, already
complete and already scored against complete n=2..5 grids).

**A correction, registered before any measurement.** Board #1414 states the
park's rule as *"hoist the maximal prefix whose destination register is
strictly increasing, **excluding the cycle-closing move**; defer the rest"*. As
literally written that rule gives the **wrong** answer on its own discriminating
cell: `[2,0,1]`'s chain excluding the closer is `r3<-r5 · r5<-r4`, whose
destinations `(r3, r5)` **are** strictly increasing, so it predicts 3 hoisted +
1 deferred where `w-clear` measured **2 + 2**. The rule that fits all five cells
is the same idea with the closer **included** in the scan:

> **R-INC** — write the chain in dependency order `m<-s(m), s(m)<-s²(m), …,
> s^{k-1}(m)<-r11`, where `m` is the **lowest** argument register in the cycle.
> Hoist the park plus every move `c_j` such that `dest(c_i) < dest(c_i+1)` for
> all `i <= j`. Everything from the first descent on, inclusive, stays at the
> call.

I register R-INC as the rule under test, and #1414's literal text as **an error
in the record I expect to confirm** (P7).

---

## 1. Does it convert?

| | registered |
|---|---|
| **P1** | **`mmio.cpp` does NOT convert.** TU match 11 -> 11, mismatch 0 -> 0. The TU is 316 of 380 B away across **three** refused functions; `?mmioGetInfo` is one of them. This is a **rung, not a conversion**, and I will say so in the rung doc's §1 in those words. |
| **P2** | **`?mmioGetInfo` does NOT emit byte-exact this lane.** It needs three clauses — the park, `callseq-multiarg-lit`, `expr-intrinsic-memcpy` — and the second and third live in `crates/c2-il/src/func/body/` (`shapes/calls.rs::seq_call_arg_sources`, `expr.rs`), which is a **concurrent lane's** file tree. I register the expected deliverable as **the park alone**: 30 refused cells becoming byte-exact, `fnbyte-exact` on `mmio.cpp` unchanged at **8/11**. |
| **P3** | `mmio.cpp` remaining byte distance **316 B at both ends**; `?mmioGetInfo` **84 B at both ends**. |

---

## 2. The discriminating population — the registered answer

| | registered |
|---|---|
| **P4** | **The population EXISTS and is large.** The descent clause of R-INC has a witness in **every 3-cycle whose second destination exceeds its third** — one rotation of every 3-subset of the argument slots. At arity <= 5 there are **10** such 3-subsets, so I register **>= 8 discriminating cells over >= 6 distinct register triples and >= 2 arities**, against `w-clear`'s one. |
| **P5** | **And the shippable class is settled by ENUMERATION, not by discrimination.** The port refuses cycles longer than three (`call-arg-long-cycle`, `MAX_VERIFIED_PERM_CYCLE`) because past three c2 abandons the single-temp walk. A cycle of length <= 3 has a chain of <= 3 destinations, so R-INC admits **exactly two shapes**: `park+1 \| closer` at k=2, and at k=3 either `park+2 \| closer` (ascending) or `park+1 \| 2` (descent). I register that the **complete** k<=2,3 population at arity 2..5 is 50 permutations and that I will grade **all** of them, so the rule is not fitted to a sample. |
| **P6** | **The rival rules are extensionally IDENTICAL inside the shipped class.** R-INC and the "ascending-destination ready scan" (**R-SCAN**: emit the parallel copy in ascending destination order, stopping at the move sourced from `r11`) agree on every cycle of length <= 3 and first differ at length **4**, at destination sequence `(r3,r5,r6,r4)` — R-INC 3 hoisted, R-SCAN 2. The generator must therefore **assert that it cannot separate them in class** and separate them only in the out-of-class control, or the grid is claiming a discrimination it does not have. |

---

## 3. The rule's own content

| | registered |
|---|---|
| **P7** | #1414's literal text mis-predicts its own `[2,0,1]` cell (see §0). **I expect to confirm this**, i.e. to re-measure 2 + 2 and not 3 + 1. |
| **P8** | The **anchor** is the cycle's **lowest argument register**, and the guarded form parks that register while the unguarded form parks its *image* — the two are opposite rotations of one cycle break, not two unrelated rules. `gt_argperm.py::predict_pure`'s local-minimum model is the unguarded half of the same machinery. |
| **P9** | **The split does not depend on which formal the guard reads.** Guard-on-the-parked-formal, guard-on-a-moved-but-unparked-formal and guard-on-a-formal-outside-the-cycle all produce the same entry/call split. **This is the clause I am least sure of and it is the registered failure mode F1.** |
| **P10** | Guard **count** (1, 2, 3) does not change the split; only the number of compare/branch/`li`/`b` groups between the park and the call. |

---

## 4. Direction of error, and the shape question (#770)

Board **#770** ran eleven for eleven **optimistic** until two lanes this week
missed **pessimistic**, both because *a floor sat under many sites at once*.
The brief instructs me to ask which shape mine is.

**Mine is the INVERSE of a floor.** `w-clear` §6.1 established that the floor
here is the **guard itself** — `SeqEarlyReturn` composes with almost nothing,
and that one fact sits under every cell of the four frontier ladders. This rung
does not add a floor; it **lifts one composition out from under it**. A lane
lifting a floor is the shape that comes back *optimistic*, because every other
clause the floor was hiding is still there underneath (§3.2 of `w-clear`: twelve
of thirteen probes changed their reported blocker key the moment the guard was
removed).

**So I register OPTIMISTIC**, and specifically:

* **F1 — the guard target changes the split.** If a guard on a formal *outside*
  the cycle, or on a moved-but-unparked one, moves the split, then P9 is wrong
  and the rule needs a fourth axis. **This is the most likely way this lane
  declines.**
* **F2 — the park is not a function of the cycle at all**, but of the *guard's*
  scrutinee (c2 parks the value the guard reads, and the cycle break follows).
  `w-clear`'s five cells cannot separate these two because in all five the
  guard reads `a0` and `r3` is also the cycle minimum. **The grid MUST contain
  cells where the guard's formal and the cycle minimum are different registers**
  or it repeats `w-clear`'s confound. Registered as a required class of the
  generator.
* **F3 — an unmoved formal inside the cycle's register span changes the split**
  (e.g. a 3-cycle on `{r3,r5,r7}` with `r4`, `r6` untouched).
* **F4 — the port's existing 22 matching cells regress.** Any `mismatch`
  anywhere is an alarm, not a gap.
* **F5 — `--jobs`/cache contention produces a NUL-corrupted output file.** Every
  run in this lane gets its **own** output path and an **absolute** `--cache`.

---

## 5. The grid, frozen before the first `cl.exe`

**Structural axes first, crossed; values varied inside each cell.**

| axis | levels |
|---|---|
| **A — cycle length** | 2, 3 (**in class**) · 4, 5 (**out-of-class control**) |
| **B — cycle placement** | every k-subset of the argument slots at arity 2..5, both rotations at k=3 |
| **C — guard target** | the cycle **minimum**'s formal · a **moved-but-unparked** formal · a formal **outside** the cycle |
| **D — guard count** | 1, 2, 3 |
| **E — trailing call count** | 1, 2 |
| **F — a literal in a slot** | absent · present (the `?mmioGetInfo` shape, `li r5,72`) |

The generator asserts its own classes and **refuses to write** if a class is
empty or if R-INC and R-SCAN are indistinguishable on the whole grid (they must
be distinguishable in the out-of-class control and indistinguishable in class —
both are asserted). `sha256` of the generated set and every rival's per-cell
prediction are committed **before** the first compile.

---

## 6. Evidence I will produce either way

Both ends (`851938df` and tip): TU match / mismatch / codegen-gap / vocab-gap /
capture-fail; the `gap-metric` diff; `fn_blockers` / `emit_blockers` deltas
naming every key that moved; `?mmioGetInfo`'s and `mmio.cpp`'s remaining byte
distance; test count and target count; `git grep -c '#\[test\]'` under
`crates/`; `work/w-splice/peerkeys.py` at both ends; the full
`scripts/gate.sh --require-graded` verdict, **in the foreground**.

**A decline is a full result.** If F1 or F2 fires, the deliverable is the grid
and the refusal stays exactly where `w-clear` put it.
