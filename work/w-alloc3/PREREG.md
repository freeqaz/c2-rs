# w-alloc3 — PREREG

    Lane:      w-alloc3, worktree branch `wt-w-alloc3`, base master `f0d24e46`
    Written:   before GRID-A or GRID-H existed; before any cell of either was
               compiled. The only objs that existed when this was written are
               `work/w-alloc3/probe0/p0.obj` (the six-line smoke cell in §0.3,
               which is a pipeline check and is *also* the cell that made the
               hypothesis concrete) and the 878-TU baseline scan.
    Owns:      `crates/c2-core/src/codegen/alloc.rs`. Everything else is read.
    Board:     **#1067–#1076**, re-verified against master (`docs/BOARD.md`'s
               highest row is **1046**).

---

## 0. What is already known, and what this lane is allowed to claim as new

### 0.1 The six deaths are the bar

`crates/c2-core/src/codegen/alloc.rs`'s module doc records six allocation keys,
every one of which fitted its own cells and then died on a frozen never-fitted
holdout:

| key | fit | frozen holdout | board |
|---|---|---|---|
| `w-next`'s `uses + (register-derived ? 1 : 0)` | 24 / 24 | **7 wrong of 56** | #836 |
| `H-self` (the self-bind bonus, ~1.5 uses) | 80 / 81 | **11 wrong of 72** | #857 / #860 |
| `clause-1-strict` (the no-tie use-count lift) | — | **12 wrong of 36** | #868 |
| `RULE W` (the two-bit spelling key) | — | **7 wrong of 388** | #886 |
| `RULE W2` (RULE W + H-self's magnitude) | **388 / 388**, every recorded refutation cell | **14 wrong of 106** | #887 |
| `KEY ILX` (the key stated in IL fields) | 32 / 32 | **14 wrong of 45** | #909 |

**The incumbent is the refusal, and it is 0 wrong on every one of those
populations.** `codegen::alloc::allocate` returns `None` outside a narrow
regime and `all_in` fails closed; the port's correctness rule forbids wrong
emits, not incompleteness, so **a rule that is right 90 % of the time is
strictly worse than the refusal that is right 100 % of the time**. This lane
registers the refusal as its control and will not displace it on anything short
of 0 wrong.

`RULE W2` is the sharpest of the six and it sets this lane's method: it passed
**every previously recorded refutation cell** and then died on fresh ones.
Passing recorded refutations is not evidence. GRID-H below is built to contain
axes that no cell in GRID-A varies.

### 0.2 The evidence this lane is run on

`docs/rungs/2026-08-08-w-seq.md` §4.2 dissected **503** SPLICE-0 failures and
found every one is a **field** perturbation with **no reordering anywhere**:

* **286** source-register renames `r3 → r4` (`81630008 → 81640008`);
* **123** destination renames `r3 → r11` (`80630004 → 81630004`), every one of
  the `framed` differs;
* **~92** displacement folds; 2 a different body.

`docs/rungs/2026-08-08-w-splice.md` §5 leaves those exact populations refused:
`tail S3-tail-setup` **951** and `framed S1-framed` **123**.

### 0.3 THE ONE MEASUREMENT TAKEN BEFORE THIS DOCUMENT, DECLARED

Two things were measured before this file was written, and both are declared
here rather than presented later as if they came after:

**(a) The population census** (`work/w-alloc3/pop.py`, `pop.txt`, run on this
lane's own 878-TU baseline scan). The two target populations are far less
independent than their pair counts suggest, and **this is a finding in its own
right**:

```text
  framed   pairs 123   distinct symbols  83   distinct TUs  76   template roots  1
  tail     pairs 380   distinct symbols  44   distinct TUs 332   template roots 30
  the 286-witness signature:  1 SYMBOL, ?Release@Object@Hmx@@, in 286 TUs
```

**The 286 are one function**, and the 123 are **one accessor template**
(`?back@?$vector<T>`) at 83 instantiations. Board **#925**/**#952** is the
standing caution and it binds here: 83 instantiations of one class template are
one idiom and not 83. So the brief's *"the most constrained allocation evidence
this project has ever collected"* is, counted as idioms, **n = 2**. Any rule
fitted on them is fitted on two cells, and that is why this lane's whole
argument has to rest on a frozen holdout of its own manufacture.

**(b) A six-line smoke cell** (`work/w-alloc3/probe0/p0.cpp`) written to check
the compile-and-dump pipeline. It reproduced the 123-idiom exactly:

```text
-- .text ?endv@@YAPAHPAUV@@@Z          -- .text ?backv@@YAPAHPAUV@@@Z
   80630004  lwz  3, 4(3)                 81630004  lwz 11, 4(3)
   4e800020  blr                          386bfffc  addi 3, 11, -4
                                          4e800020  blr
```

That cell is **fit data, not holdout**, and it is named as such.

---

## 1. THE HYPOTHESIS — **RULE BIND**

> **RULE BIND.** When c2 inlines a callee `G` at the single call site of `F`, it
> emits `G`'s own body with exactly two rewritings, and nothing else:
>
> **BIND.** Every GPR operand field naming one of `G`'s formal registers
> `r(3+i)` is replaced by the register the caller's *i*-th actual already lives
> in — `β(i)`. No copy is emitted. Registers `G`'s body uses that are **not**
> `G`'s formals (its temps) are left alone.
>
> **TEMP.** The destination of the instruction producing `G`'s return value is
> left at `r3` **iff** that value is `F`'s own returned value verbatim.
> Otherwise it becomes `POOL_TOP` = **`r11`** — `codegen::alloc`'s own already
> shipped constant — and `F`'s trailing computation reads `r11` and writes `r3`.

The 286 are BIND with `β(0) = r4`. The 123 are TEMP's second branch. Both are
consequences of one statement, which is the cross-check the brief asks for.

### 1.1 Why RULE BIND is not a restatement of any of the six

Each of the six answers **one** question: *given two or more producers of a
store run that are simultaneously live, which of them gets `r11` first?* Every
one of them is a priority key — a comparator over producers — and every one of
them died on the comparator, at a tie or at a spelling.

RULE BIND does not contain a comparator. It is stated on a regime where **at
most one value is live at a time**, so no two producers are ever ranked:

| key | what it ranks | RULE BIND consults it? |
|---|---|---|
| `w-next` | use count + kind bonus | **no** — one value, no use count is read |
| `H-self` | a self-referential store bonus | **no** — there is no store run |
| `clause-1-strict` | strict use-count gap | **no** — no gap exists |
| `RULE W` / `RULE W2` | producer *spelling* (`add` vs `slwi` vs `self`) | **no** — the callee's spelling is whatever c2 already emitted for it and is copied, not classified |
| `KEY ILX` | IL-field classes `LOAD`/`SELF-2B`/`SELF-1B`/`CROSS` | **no** — no IL of the caller's value is read |

RULE BIND also introduces **no new register-choosing key at all**: its one
register choice, `r11`, is `POOL_TOP`, which the shipped module already
allocates for a one-producer run and which board **#605** measured directly.
What is new is the *claim that this regime is that regime* — that an inlined
callee's result is one producer of the caller's, taken from the top of the same
pool. That claim is what GRID-H tests, and it is falsifiable: if the temp is
ever anything other than `r11`, or if a copy is ever emitted, RULE BIND is dead.

**The one place a comparator could re-enter is registered as a REFUSAL**
(§2 clause D7): two inlined results simultaneously live. That is exactly the
regime the six died in, this lane does not model it, and cells in it are printed
as out of domain rather than scored.

### 1.2 The rivals this lane must separate, named before any cell exists

| rival | agrees with RULE BIND on | separated by |
|---|---|---|
| **R1** "the temp is `r11` because the callee's own body already used `r11`" | nothing yet — `?endv` uses no temp | any cell whose callee body has no `r11` (all of GRID-A) |
| **R2** "the temp is `r11` because `r3` is still live at the site" | `?back`, if `this` is conservatively live | a caller that provably never touches its formal again (**A-live0**) vs one that does (**H-live1**) |
| **R3** "the temp is the LOWEST free volatile" (`r(4+n)`) | every 1-formal cell, where `r11` and the lowest free differ | caller formal counts 5–8 (**GRID-H H1**) |
| **R4** "the destination is always `r11`, with a `mr r3,r11` when it is returned" | the 123 | already refuted by `w-seq`'s `s01`; re-checked on **A-ret0** |
| **R5** "BIND is really a rename of `r3` specifically, not of the formal at position *i*" | the 286 (position 0) | a callee whose *second* formal is the one bound (**A-pos1**, **H-perm**) |
| **R6** "c2 emits a copy when β is not the identity" | nothing — refuted by the 286 | every `β ≠ id` cell |

---

## 2. THE DOMAIN — decided from the SOURCE and from c2's COMDAT for `G`, never from c2's COMDAT for `F`

A cell is **in domain** iff every clause holds. Out-of-domain cells are
**printed with their clause** and are scored as neither hit nor miss.

| # | clause |
|---|---|
| **D1** | `F`'s body is exactly `return G(a…);`, `G(a…);` (void), or `return G(a…) ± K;` for one compile-time constant `K` |
| **D2** | every actual `a` is a formal of `F` **named directly** — no computation, no address-of, no field access. *This puts `w-seq`'s ~92 displacement folds explicitly OUT of this lane's scope, by construction and in advance.* |
| **D3** | `G` is defined in the same TU, is not virtual, not varargs, and c2's `.text` COMDAT for `G` is straight-line, carries **zero relocations**, and ends in exactly one `blr` |
| **D4** | every GPR `G`'s body reads or writes that is not one of `G`'s formal registers is **strictly above** `F`'s formal high-water mark (no temp/formal collision) |
| **D5** | all formals and returns of both `F` and `G` are ABI-integer or pointer; `F` has ≤ 8 formals; no float, no vector, no aggregate by value |
| **D6** | `β` is injective |
| **D7** | at most **one** inlined result is live at a time — i.e. `F` has exactly one call site. *This is the clause that keeps the six out.* |
| **D8** | c2 actually inlined: c2's COMDAT for `F` carries **zero relocations**. A cell where c2 kept the call is **out of domain and printed**, because RULE BIND says what the bytes are when c2 inlined and says nothing about whether it does (`INLINE_PREDICATE.md` §7's 2.84 % residual is not this lane's) |

## 3. THE PREDICTION, mechanically

Read `Gw` = c2's COMDAT words for `G` (last word `blr`). Then

```text
head := Gw[:-1] with every GPR operand field equal to a G-formal register
        r(3+i) replaced by β(i)                                     (BIND)

if F returns G's value verbatim, or both are void:
        P(F) := head ++ [blr]                                       (TEMP, r3 branch)
else (F returns G's value ± K):
        let j be the index of the last instruction of `head` writing r3;
        head[j].RD := 11;
        P(F) := head ++ [ addi r3, r11, K' ] ++ [blr]               (TEMP, r11 branch)
        where K' = K * sizeof(pointee) for pointer arithmetic and K otherwise,
        and `addi rD,rA,imm` is 0x38000000 | D<<21 | A<<16 | (imm & 0xffff).
```

`P(F)` is compared **byte for byte** against c2's own COMDAT for `F`. A cell is
a **HIT** iff they are equal, a **MISS** otherwise. There is no partial credit
and no "wrong in only one field".

## 4. THE FROZEN HOLDOUT — protocol

1. GRID-A (fit) and GRID-H (holdout) sources are generated **together**, by two
   generators committed before either is compiled. GRID-H's sources' `sha256`
   manifest is committed in the same commit as its generator, **before any
   GRID-A obj exists**, so the holdout cells cannot be tuned to the fit result.
2. GRID-A is compiled and graded. RULE BIND may be refined here; every
   refinement is written down in the rung with what forced it.
3. GRID-H's **per-cell prediction** is then frozen: a `.tsv` carrying, per cell,
   the predicted words as hex and the source's `sha256`, committed **before the
   first `cl.exe` runs on GRID-H**. The grader re-checks every `sha256` and
   reads the frozen column; it never recomputes a prediction.
4. GRID-H is graded **once**. If a refinement is made after step 3 the holdout
   is spent and this lane declines.

## 5. THE PREDICTIONS, registered

| # | registered |
|---|---|
| **P1** | **THE CLAIM I MOST EXPECT TO LOSE.** RULE BIND is **0 wrong** on GRID-H's in-domain cells. If it is wrong on even one, RULE BIND joins the graveyard as the **seventh** entry and nothing is shipped. |
| **P2** | The TEMP register is **`r11` and never anything else** across caller formal counts 1 through 8 — i.e. **R3 is refuted**, the temp is `POOL_TOP` and not the lowest free volatile. |
| **P3** | **R2 is refuted**: a caller that provably never reads its formal again still gets `r11`, so the temp is not forced by the formal's liveness. |
| **P4** | **R5 is refuted**: BIND is positional. A cell binding `G`'s *second* formal from `F`'s *first* renames `r4 → r3` — a rename in the direction **opposite** to the 286's, which the workload has never shown. |
| **P5** | **BODY-LENGTH STRATIFICATION — the `/QXSTALLS` control.** Hit rate on GRID-A ∪ GRID-H is reported per callee body length (1, 2, 3, 4+ words). If RULE BIND's accuracy tracks length, the result is a size effect and is reported as one. Registered expectation: **no monotone trend**. |
| **P6** | **CONTROL.** `crates/c2-il/` and `crates/c2-core/` diffs are empty unless P1 holds; TU match stays **10**, `mismatch` **0**, `fnbyte-differs` **2,334**, `fnbyte-exact` **35,986**, `match-tu-differs` / `match-tu-reloc-differs` **0**. |
| **P7** | **CONTROL.** `work/w-splice/peerkeys.py` at both ends: **0 key families vanish** and every family that moves is named with both numbers. |
| **P8** | **CONTROL / the parallel-probe rule.** Every cell is compiled into **its own directory**. Board **#1045** — four parallel tests sharing one PID-keyed temp dir fabricated a finding that would have reversed a decline — and a shared directory is a fabrication risk, not a tidiness one. |
| **P9** | At least two cells of each family are printed **word for word** from an obj this lane compiled. |

## 6. THE DECLINE FLOOR

* **Ship** (outcome 1) only if GRID-H is **0 wrong** on a population large
  enough to say so. What ships is then a **pure, inert predicate plus tests and
  a doc block in `crates/c2-core/src/codegen/alloc.rs`** — this lane owns that
  file and nothing else, and the emitter that would consume the rule lives in
  `crates/c2-core/src/splice.rs`, which belongs to another lane's clause set
  (`S1`/`S3`). **No emitted byte may move.** A rule shipped as bytes this wave
  would be a rule shipped by a lane that does not own the emitter.
* **Ship nothing but the spec** (outcome 2) if GRID-H is 0 wrong but too small
  to decide — and name the grid that would decide it.
* **DECLINE** (outcome 3) on **any** wrong emit, and write the seventh graveyard
  entry into `alloc.rs`'s module doc with its refutation cell. Two lanes
  declined this week and both were right.
* In all cases publish §0.3(a)'s re-partition of the 123 and the 286
  (outcome 4), because it stands independently of RULE BIND's fate.

## 7. What this lane will NOT do

* Not widen `crates/c2-il/` (that is `w-memset`'s this wave) and not touch
  `work/` outside `work/w-alloc3/` (`w-root` is measurement-only there).
* Not glob or recursively walk `work/capture-cache` or `.claude/worktrees`.
* Not key any per-record binding on `IlFunction::mangled_name` — **#918**,
  74,955 rows disagree; `FnCensus::emit_name` only.
* Not write a wait loop whose `pgrep` pattern matches its own argv, and not
  write an unbounded wait.
