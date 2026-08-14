# w-backedge — PREREG

Lane **w-backedge**. Committed **before the first `cl.exe` invocation of either
grid**. Base: master `6f2c7c41`.

The brief is board **#3082**, left OPEN by lane `w-ir-e`: *the block IR cannot
hold this port's loops, and what stops it is the LABEL COUNTER, not the IR.*
`codegen::labels::LabelMap::resolve` **invariant 4** refuses every backward
intra-section branch on the ground that c2 charges the compiler-label counter
**+1..+4** for a body with one while `coff::plan_labels` charges **0**
(`LABEL_COUNTER.md` §4.2, 17 seed-free cells).

**This lane does not build a loop lowering and does not widen invariant 4.** Its
deliverable is the answer to the question #3082 says a loop lane inherits and
must price first:

1. **Is the charge derivable from the IL the port can see?**
2. **What is the +1..+4 range's structure** — what varies it?
3. **What would it cost to lift invariant 4 safely, priced two-sided?**

---

## 0. The frozen grids — BY CONTENT HASH, before the first probe

```text
  sha256(work/w-backedge/grid1.tsv) = 3dd6e18f2b857875a9b11ee873137a6c1d0c5f9cd6a3cce1dfbf7e52120a62cd   28 cells   FITTING
  sha256(work/w-backedge/grid2.tsv) = e1e2a5a2623479b472ba10a80eb8a6deb8deeb4daaae11e004de3059a96d1e54   13 cells   HELD OUT
```

`w-keygen` froze a hold-out by **name** and its population moved −10.8 %
underneath it while the file stayed byte-identical. The hashes above are what
make grid2 a hold-out. **grid2 is not compiled, not read and not quoted until
the rule fitted on grid1 is written into §3 with its coefficients**, and the
run that scores it re-checks both hashes and refuses if either moved.

Axes are **structural and crossed**, and the axes vary **counts** — number of
sequential loops (1,2,3), nesting depth (1,2,3), number of forward-only
branches (1..4) — not just the values inside them. Three lanes here were bitten
by grids that held arity fixed.

---

## 1. What is already known, and what is therefore NOT the question

Read first, so this lane does not re-derive it (`LABEL_COUNTER.md` §4.1/§4.2,
boards #285, #286, #287, #741, #742, #743, #1707, #2340, #2341):

* The charge is **not a function of the emitted obj** — closed at the *shape*
  level (`ho-ternary` vs `cf-ifelse`: one emitted shape, +2 and +1) and at the
  **byte** level (`do/while`, `for(;;)+break` and `goto`: **byte-identical
  24-byte `.text`**, charging **+1 / +3 / +1**), and a branch-free `mulli` body
  charges **+2**.
* The charge is **not in the `.gl` label seed** — `ilseed.py` read
  `u32(.gl[7..11])`: `cf-if2`/`cf-ifelse` **share a seed and differ by +1**.
* §4.1's own statement of what is left: *"a per-function `.ex` field is the
  only unexamined channel, and it is open"*.

**So the question this lane asks is the one §4.1 left open, and the channel it
opens is not `.ex` but `.sy`.** `crates/c2-il/src/func/sy.rs` documents, and the
shipped reader parses, a per-function record run

```text
  ( 03 <k != 01> <tok> <2 B | 00 <name> 00> <b> <b> )*   label declarations
  03 01 <tok> 1F 00 01 01                                block open
```

i.e. **the front end declares its label tokens per function, positively, in the
IL the port already reads**. No lane has correlated that count with the charge.
`IL_STMT_GRAMMAR.md` §12.5 says `while` **allocates three label tokens and uses
two** and `for` allocates three and uses three — an allocation count that is
*not* recoverable from `.ex` definitions alone, and both of those forms charge
**+2**.

---

## 2. Claims, in probability form

Scored in the rung. `P` is this lane's stated credence **before the first
capture**.

| # | claim | P |
|---|---|---|
| **C1** | The `.sy` label-declaration count is extractable for ≥ 24 of grid1's 28 cells, with the three anchors' counts equal on every extracted row (the per-row control) | 0.80 |
| **C2** | The charge is a **function of** the `.sy` label-declaration count alone on grid1 — no two cells with equal counts have different charges | 0.35 |
| **C3** | The exact rule `charge = sy_label_decls − 1` holds on ≥ 26 of 28 grid1 cells | 0.20 |
| **C4** | The charge is a function of *some* feature vector computable from the IL alone (any of: `.sy` decl count, `.ex` `29` definitions, `.ex` branch-target references, backward-reference count) on grid1 | 0.60 |
| **C5** | Whatever rule fits grid1 scores ≥ 11 of 13 on the **held-out** grid2 | 0.30 |
| **C6** | The same rule, unchanged in form and in coefficients, also fits at `/Ox` | 0.15 |
| **C7** | The `.sy` decl count separates the §4.2.2 **identical-bytes triple** (`do/while` +1, `for(;;)+break` +3, `goto` +1) — i.e. the counts are 2, 4, 2 or any assignment that respects the charges | 0.45 |
| **C8** | Invariant 4's refusal costs **0** TU conversions on today's 878-TU workload (i.e. every `cflow-loop` TU is blocked ahead of codegen, #1707/#1394 still true at `6f2c7c41`) | 0.80 |
| **C9** | ≥ 1 workload TU contains **both** a loop-bearing function and a framed function — the shape where a wrong charge becomes wrong bytes (#747's "the corpus cannot express it" is about the *gate corpus*, not the workload) | 0.75 |
| **C10** | #3082 ends this lane **NARROWED**, not CLOSED | 0.65 |

**Registered direction of the most likely error:** that the charge is a
function of the IL *plus* something c2 derives itself (a rotation, a tail
merge), so a rule fitted on grid1 will miss on grid2 by a **small integer,
most often 1**, and most often on the cells with the deepest nesting. That is
the error the control in §4 must be able to see.

---

## 3. The rule — to be written here BEFORE grid2 is compiled

> **EMPTY AT PREREG TIME. Filled in, with coefficients, in the commit that
> precedes the grid2 run.** A rule written after the hold-out is scored is not
> a prediction, and this section's git history is the evidence of which it was.

---

## 4. The control that can go red

Asked in writing, as the brief requires: **if my rule for the charge were wrong
in the most likely way, which cell changes?**

The most likely wrong-by-one error is invisible in a stride table — a stride is
a difference and a constant error cancels. So the control is not a stride. It
is a **constructed TU** of the shape #747 says neither standing instrument can
produce:

```text
  <loop leaf P>  then  <a framed function>       in ONE TU
```

Here the charge is not a difference: it moves the framed function's **`$M`/`$T`
numbers**, which are **six bytes of the reference obj's symbol table**. The
control is scored against those bytes:

* **M0** — the rule's predicted `$M`/`$T` for the framed function equals the
  reference obj's. Must be GREEN on every constructed cell.
* **M1** — the same prediction with the rule's charge perturbed **+1**.
* **M2** — perturbed **−1**.
* **M3** — the charge forced to **0**, which is exactly what `plan_labels`
  charges today, i.e. what the port would emit if invariant 4 were lifted with
  no model at all.

**M1, M2 and M3 must go RED, and the count of cells on which they go red is
printed.** A cell where M1 and M0 agree is a cell that could not have
disagreed — it is **vacuous** and is counted separately and loudly. A lane here
once reported two "0 disagree" results that no cell could have made disagree;
the printed discriminating-cell count is what makes that a loud failure instead
of a silent pass. **If the discriminating count is 0, this lane FAILED.**

This control reads **reference-obj bytes**, not the port's own model, which is
`w-ir-e`'s mutant-1 standard.

## 5. What would falsify the whole approach

* The `.sy` block order does not correspond to function order, or the anchors'
  counts differ from each other → **C1 fails**, the channel is not readable,
  and the honest answer to question 1 is *"not by this route"*.
* Two grid1 cells with identical **whole IL feature vectors** and different
  charges → **C4 fails**, and the charge is not derivable from the IL at all by
  any rule over these features. That is the answer that keeps invariant 4
  forever, and it must be stated in those words.

## 6. Scope fence

* **`crates/` is not modified by this lane** if it can be avoided, and the
  target is that it is not modified at all: `coff/` is single-occupancy and
  `labels` is the file behind both of the project's six-wrong-bytes defects.
  The rule is evaluated in the **instrument**, against reference-obj bytes; the
  port's `plan_labels` is not touched. If `crates/` is touched, the gate byte
  delta must be **zero** and proved by identity diff.
* **No loop lowering is built. Invariant 4 is not widened.** If the honest
  answer is that the blocker relocates again, this lane says so and names
  where it went — three blockers relocated rather than resolved today already.
