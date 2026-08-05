# w-brfalse — PREREG. Registered BEFORE any `crates/` edit.

    Tag:       w-brfalse-prereg
    Slug:      w-brfalse-prereg
    Date:      2026-08-05
    Fixtures:  none — a prereg admits no shape and moves no accept/refuse
               boundary. It is a record of predictions made before the
               measurement that scores them.
    Census:    706555/2463393 (28.68%) at registration — the baseline this lane
               was handed, re-measured (§0) and not copied.
    Record:    this file. Scored in `docs/rungs/_2026-08-05-w-brfalse.md`.
    Lane:      w-brfalse, worktree `wt-w-brfalse` off master **`cf86b09`**.

---

## 0. The baseline, taken BEFORE this file was written and BEFORE any edit

`c2rs gap` over the 878-TU dc3 workload at `cf86b09`, binary built in-worktree,
`C2RS_COMPILERS` / `C2RS_WIBO` exported explicitly:

| metric | read | master's block |
|---|---:|---:|
| match / mismatch / codegen-gap / vocab-gap / capture-fail | 8 / 0 / 0 / 863 / 7 | same ✔ |
| `factor-a` / `-b` / `-c` / `-d` / `-e` | 28 (LO 27) / 338 / 169 / 8 / 2 | same ✔ |
| `b-and-c` / `a-and-b-and-c` / `…-and-d-or-e` | 151 / 27 / 8 | same ✔ |
| `frontier` / `frontier-if-a` | 19 / 141 | same ✔ |
| census / emitted census | 706555/2463393 · 38458/178975 | same ✔ |

`work/w-dclass/rerank.py` over this lane's own scan reproduces w-dclass's ladder
head for head (`expr-cmp-eq` +3, `assign-store-type-8643` +2, `expr-cmp-ne` +2),
and the five TUs w-cmp's brief names carry exactly the blocker sets it published:

```
IPP_basicmath_xbox.cpp  {expr-cmp-eq: 4}      osfinfo.cpp  {expr-cmp-ge: 1}
undname.cpp             {expr-cmp-ne: 1}      mmio.cpp     {expr-cmp-eq: 3}
jsonwriter.cpp          {expr-brfalse: 1}
```

`df -i /tmp` = 195,825 / 1,048,576 (19 %) at lane start — checked first so a red
instrument could not be misread as my code.

---

## 1. What I am about to measure, and why it is TWO levels and not one

w-cmp's workflow is binding: **sink first, re-rank second, build third.** The
sink this lane owes is for `expr-brfalse`, the corrected head at 5 TUs.

**Where the key comes from decides what a sink can honestly consume.**
`expr-brfalse` is `Block { ctx: "expr", byte: 0x38 }` — the fall-through arm of
`parse_expr`. The only production that reaches it on these bodies is
`body/mod.rs:1593`, `parse_expr_classed(seg, &mut p, 0x41)`: the **return-value
expression** of the straight-line leaf class, reached after every non-committal
shape recognizer above it has declined. So the key does not say *"this body
needs a branch instruction"*. It says **"the dispatcher fell through to the
straight-line class and the body turned out to have control flow in it"**.

That is a materially different claim from `expr-cmp-eq`'s, and it is the reason
one sink is not enough. So:

* **Level B1 — `C2RS_SINK_BRANCH=expr`**, consuming `38 <tok>` / `39 <tok>` and
  nothing else. Answers exactly the question the key's name asks: *is
  `expr-brfalse` a fall-through key in the literal sense — does the census
  simply report the next byte?*
* **Level B2 — `C2RS_SINK_BRANCH=cflow`**, additionally consuming `29 <tok>`
  (label), `3A <tok>` (jump) and `4B` (statement end): the whole intra-body
  control-flow token set. Answers the question the *rung* asks: *if the port had
  a general conditional-CFG body class, would these five TUs convert?* B1 alone
  cannot answer that, and reporting B1's answer as if it did would be the
  marginal-read-as-joint error one level in.

Both levels are **measurement-only by construction**, on w-cmp's own pattern:
they push **no** `IlOp`, and a walk that reaches the end having consumed one
refuses under `expr-branch-sink-poison`. **Decoding is not accepting.** The
poison count is itself the deliverable — it is the number of emitted functions
for which the consumed family was *the last thing in the way*.

Both are run **with `C2RS_SINK_REL=expr` also ON**, because four of the five
named TUs only acquire `expr-brfalse` under w-cmp's relational sink. Measuring
B1/B2 with REL off would be measuring a frontier that does not exist.

Four arms, one binary, environment variables apart:

| arm | `C2RS_SINK_REL` | `C2RS_SINK_BRANCH` |
|---|---|---|
| **OFF** (control) | — | — |
| **REL** (w-cmp's arm, reproduced) | `expr` | — |
| **B1** | `expr` | `expr` |
| **B2** | `expr` | `cflow` |

---

## 2. Registered predictions

Scored in the findings doc. **Misses stay on the page.**

| # | prediction |
|---|---|
| **R1** | The baseline above is exact against master's block, every digit including `cargo test --workspace --release` = **809 passed / 0 failed / 27 targets**. *(§0 already scores the scan half; the test half is scored in the findings.)* |
| **R2** | The **REL** arm reproduces w-cmp's published numbers: `expr-cmp-eq` 2208→0, `expr-brfalse` 3097→5484, `expr-brtrue` 126→659, `expr-rel-sink-poison` 5, emitted census +0, TU match 8 — **every digit**. If it does not, my instrument is wrong and nothing below is admissible. |
| **R3** | **`expr-brfalse` is a FALL-THROUGH key at level B1: TUs converted = 0.** Mechanism registered in advance: the key is raised by a straight-line-return expression walk, so consuming the branch token cannot make a body with control flow in it emittable — the walk simply reaches the next control-flow token or the `then`-block's own first unmodeled byte. |
| **R4** | **TUs converted at level B2 ∈ [0, 1].** B2 consumes the entire intra-body control-flow vocabulary, so what remains blocking is whatever the *arms* of these conditionals contain. My expectation is calls, stores and typed loads that the expression class already refuses on their own; but a genuinely tiny `if (x) return a; return b;` is a real possibility, hence the interval rather than a point. |
| **R5** | At level B1 the successor key that absorbs `expr-brfalse` in the five named TUs is a **control-flow** key (`expr-cflow-label`, `expr-cflow-jump` or the sibling branch) in **≥ 3 of 5**. |
| **R6** | **Mass is conserved across every arm**: closed mass == absorbed mass, net converted 0 blocked-emitted function sites, at both B1 and B2. This is w-cmp's `3298 / 3298 / 0` shape and I register that it repeats. |
| **R7** | **TU match at lane end = 8.** Not an interval — I register the point. |
| **R8** | **`labels.rs`'s BACKWARD refusal is NOT the binding constraint on any of the five TUs.** This directly contests my brief, which says widening `labels.rs` "is the substance of this lane". My reading is that the backward-label refusal sits *behind* several other refusals and cannot be reached by any of these five; if the measurement disagrees, R8 is a miss and the brief is right. |
| **R9** | Board **#269**'s standing decline clause (a frontier TU at ≥ 4 independent refusals is not a target) **fires on all five**. |
| **R10** | Any census gain is a driver, not the result. Registered expectation: emitted census **+0** on every arm, because every sink poisons. |
| **R11** | **At least one premise in my own brief or in this prereg is refuted by my own measurement.** Registered because the last three lanes each did this and it is the outcome worth optimising for. |

### 2.1 The decline clause I bind myself to now

**If B1 and B2 both convert 0 TUs, I decline the build and ship the
measurement.** Two consecutive lanes have done exactly this and both were right
to. I am registering it before I know the answer so that a 0 cannot be
re-narrated into "close enough to build anyway".

### 2.2 The counterexample clause I bind myself to now

**If I do build anything that emits bytes, I build the grid first.** w-cmp's
36-cell relational grid found a three-way interaction of (relation, signedness,
literal) firing at exactly `k = 0` with both neighbours normal — the third
recorded instance of that shape (the 63-burner bound, w-dclass's 32768 bound).
The analogous grid for a conditional-body lowering is
(branch sense × arm shape × `then`/`else` presence × early-return vs join),
compiled at the **workload's own flags** and graded against **real c2**, before
one line of the lowering is written. A green gate is necessary and not
sufficient; every live wrong-emit family closed this week was found by a
constructed counterexample.

---

## 3. Why `expr-brfalse` might be `expr-op-0x27` again — the argument my brief demanded

My brief requires an explicit, numerical argument for why `expr-brfalse` is not
the `expr-op-0x27` case (the #1 census key at 23,090 emitted, worth **six**
emitted functions and **zero** TUs — seven confirmations of board #150).

**I cannot make that argument, and I am registering that I cannot, in advance.**

The three facts I have before measuring all point the *wrong* way:

1. **w-cmp's R8 removed mass as a screen.** `expr-cmp-eq` was the #12 key at
   2,208 and was a fall-through key anyway. `expr-brfalse` is 3,097 at baseline
   and **5,484 under the REL sink** — larger than `expr-cmp-eq`, and mass is
   known not to discriminate.
2. **`expr-brfalse` is raised by the *dispatcher's last resort*.** `expr-op-0x27`
   is a fall-through key because it is where bodies land after everything above
   declines. `parse_expr_classed(…, 0x41)` at `body/mod.rs:1593` is structurally
   the same position: it is reached only after `try_parse_empty_dtor_delegation`,
   `try_parse_store_leaf`, `try_parse_store_run` and every recognizer above them
   have declined. A key raised at the bottom of a dispatcher is a key whose mass
   is the mass of everything that fell through to it — which is board #150's
   definition, restated.
3. **The key gained +2,387 sites when a *different* family was sunk.** 88.5 % of
   the relational family's 3,298 blocked emitted functions went straight to
   `brfalse`/`brtrue` (w-cmp §2.2). A key that *absorbs* other keys' mass under a
   counterfactual is behaving exactly like a sink of last resort.

So the honest pre-registration is: **I expect `expr-brfalse` to be the
`expr-op-0x27` case, and R3 registers that expectation as a testable 0.** The
ladder credits it with 5 TUs; I predict the measurement credits it with 0. If
the measurement says 5, R3 is a miss, the ladder was right, and the build is on.

---

## 4. Bookkeeping registered in advance

* Board rows taken: **#440** (the `C2RS_SINK_BRANCH` counterfactual and its
  result), **#441** (whatever the measurement leaves open). Highest before this
  lane was #424.
* Scratch in `work/w-brfalse/` — never `/tmp` or `~/tmp`.
* Every gate run is watched to completion in the **foreground**. No `nohup`, no
  backgrounded wrapper: w-dclass §9.1 recorded reading a wrapper's exit 0 as a
  PASS while the gate was still running, and the cheapest mitigation is not to
  create the ambiguity.
* Every number names the population it is over. Unless it says otherwise, a
  count here is over **BLOCKED EMITTED FUNCTIONS** — not the larger "blocked
  functions" population. Fourth lane in a row to state this.
