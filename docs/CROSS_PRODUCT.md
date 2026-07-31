# CROSS_PRODUCT — grading the combinations, not the axes

`scripts/cross_sweep.sh` (driver: `scripts/cross_sweep.py`). Written 2026-07-31.

## Why

`docs/GAPS.md` §6 #12. Two branches were each fully green — an FP-store rung and
a many-call framed rung — and the **merge** mis-emitted: the compiler-label
counter is a per-TU quantity and it was being read from a per-function method,
so a framed function downstream came out six bytes wrong in an obj that still
links. **Neither branch's corpus could contain the case.** The label counter has
an observable effect only when a framed function follows, and until Class A
many-calls landed there was no framed shape that could share a TU with an FP
store; the FP rung's fixtures have no framed function and the framed rung's have
no floating point. #13 then found that the *repair* was also wrong one row
further out, because a per-function quantity and a per-TU one are
indistinguishable at n = 1.

The rule those two wrote down is:

> A merge of two independently-green branches is a **new corpus**, and the shapes
> only it contains have never been graded by anyone.

Until this lane, applying that rule was manual — it depended on whoever was
merging remembering to compile the cross product. `scripts/expr_sweep.sh` grows
**additively**: each fragment varies its own parameters inside one shape family,
so 24 fragments give 24 independently-swept axes and **zero** graded
combinations beyond whatever a fragment happened to put in one file. This lane
grows the corpus **combinatorially** instead.

## How the families are enumerated (and why not by hand)

A hand-written list of families drifts the moment a rung adds one, and a lane
that silently under-enumerates reports full coverage of a subset — the exact
failure `GAPS.md` §6 keeps recording. So:

1. **The families come from the port.** They are the `FnVerdict::InClass("…")`
   labels in `crates/c2-il/src/func/census.rs`, extracted by a paren-matched
   scan of each call's whole argument. Not a line-wise `grep`: three of the
   eighteen (`call-sequence*`, and `float-leaf`/`double-leaf`) live inside a
   nested `match`/`if` and a line pattern misses them.
2. **The representatives come from compiling.** The whole `scripts/sweep.d/`
   corpus is generated and graded, and a family's representative is a *matched*
   TU whose in-class functions are all of that one family — smallest first,
   one per fragment before a second from the same fragment, `k` of them
   (`C2RS_CROSS_REPS`, default 3).
3. **A family with no representative fails the lane, by name.** That check found
   a real hole on its first run: `call-sequence`, `call-sequence-value` and
   `call-sequence-lit` — the newest accepted class, and *the* class that made
   §6 #12 reachable — had **no single-family case anywhere in the corpus**.
   Every TU that reached them carried a second function, so the class had only
   ever been graded beside something else. `scripts/sweep.d/71-call-sequence.py`
   is what closed it (+303 cases).
4. **The external-bearing predicate is measured, not assumed — and it is a
   heuristic, not a derivation.** A representative is "external-bearing" if its
   own obj carries `_fltused`, `__savegprlr`, `__restgprlr` or a `.pdata` — read
   out of the bytes, never inferred from the family's name.

   What that predicate is *for* is picking tier-C representatives likely to
   disturb the compiler-label counter, which is the mechanism behind every bug
   this lane exists for. It is **not** an instance of "one slot per TU-level
   external": that rule was **refuted** on 2026-07-31 — `docs/LABEL_COUNTER.md`
   §2.1 — in both directions, by a newly pooled FP constant that costs **+2 and
   mints no external at all**, and by a string literal that mints one and costs
   **0**. The rule that fits the measurements is a per-function **surcharge
   table**, `LABEL_COUNTER.md` §1.1: base 1 for a leaf and 4 packed / 5 `/Gy`
   framed, plus `+1` for `_fltused` on the *first* FP-touching function, `+2`
   per distinct GPR/FPR helper width first introduced, `+2` per newly pooled FP
   constant, and `0` for a callee external at any count.

   The predicate still selects well because three of its four markers *are*
   surcharge-bearing (`_fltused` +1, the helper pairs +2 each, `.pdata` marks
   the framed base). What it **misses** is stated under "what it deliberately
   does not grade": the surcharges that mint nothing.

## What it grades

Four mode lanes throughout: `/Ox` packed, `/Ox /Gy`, `/O1`, `/O2`.

| tier | what | count at k = 3 |
|---|---|---:|
| W | each representative **alone inside a namespace** — the wrapping check | 54 |
| A | every **ordered pair** of representatives, both orders, diagonal included | 2,916 |
| B | **arity**: n = 1…4 copies of a family, alone and with a framed observer before and after | 216 |
| C | **ordered triples over the TU-external families**, with and without a stride-1 separator at each position | 1,715 |

4,901 configurations × 4 lanes = **19,604 gradings**, ~70 s cold and ~25 s warm
at `--jobs 16`.

Two of those tiers need their reason stated, because neither is obvious:

* **Tier W exists so the lane cannot lie.** Every half after the first sits in a
  `namespace`, and if a namespace by itself pushed a shape out of class then
  every pair would grade a refusal and the green would mean nothing. All 54
  representatives still match wrapped. (Namespaces rather than identifier
  renaming: they cannot collide, they need no tokenizer, and the port reads
  names out of the IL so the extra mangling is not a variable. The **first**
  half is left unwrapped, so it is byte-identical to the standalone case that
  was graded.)
* **Tier C is where a per-function and a per-TU counter rule come apart.** #13's
  candidate pair — "one slot per function plus one for the TU if anything
  touches floating point" versus "two slots per FP function" — agree at n = 1
  and disagree at n ≥ 2, which is why a single-FP-function probe could never
  have separated them and why the wrong one looked right. Pairs get n = 2; the
  triples get n = 3 with every ordering and with a separator, because a counter
  error an adjacent function absorbs is invisible without one. (Both of those
  candidates have since been superseded by the measured surcharge table,
  `docs/LABEL_COUNTER.md` §1.1; what tier C grades is unchanged, and *n* is
  still the axis that separates a per-function quantity from a per-TU one.)

## What it deliberately does NOT grade

Stated because a silent cap reads as "covered everything", which §6 forbids.

* **Triples of three *distinct non-external* families.** Tier C is restricted to
  the TU-external families (7 of 18); the full `R³` is not run. A three-way
  interaction among plain leaves would not be caught.
* **The intra-family parameter space, crossed.** A family is represented by 3
  TUs out of the hundreds the sweep generates. Operand order, widths, offsets,
  argument positions and source lines are swept *within* a family by the
  per-axis fragments and are **not** crossed against another family here.
  Concretely: the lane grades "an addr-leaf beside a framed call", not "every
  addr-leaf beside every framed call".
* **The label surcharges that mint no symbol.** Tier C selects its triples on an
  *external-bearing* predicate, and `docs/LABEL_COUNTER.md` §1.1 measures three
  surcharges that predicate cannot see: a **newly pooled FP constant** (+2), a
  **materialised signed relational** (+2), and a **loop** (+2 to +5, and not
  uniform). A TU built from those would disturb the counter exactly as an
  external-bearing one does, and tier C would not have selected it. Two of the
  three are not reachable today anyway — §2.1 checked each counterexample
  through `c2rs diff` and the TU-level gate refuses all of them — so this
  overlaps the refusal frontier rather than adding to it, but the overlap is
  incidental and will stop holding the moment that gate moves. When it does, the
  predicate should be re-grounded on the surcharge table rather than on symbols.
* **Flags beyond this lane's own four modes** — `/Od`, `/EHsc`, `/GS`, `/GR`,
  `/Zi`, `/Oi`, and every combination of them.

  > **This is now the last place the un-enumerated four survive, and it is worth
  > naming as such (2026-07-31).** `cross_sweep.sh` carries its own hardcoded
  > mode list — packed, `/Gy`, `/O1`, `/O2` — which is the same four that ran
  > everywhere else and compiles **no `/EH` at any invocation**. The fixture gate
  > no longer works that way: the lane list is data (`scripts/lanes.txt`), one
  > command runs all 12 (`scripts/gate.sh`), and a test fails if the registry
  > stops carrying an `/EH` lane
  > (`crates/c2-harness/tests/lane_registry.rs`). The cross-product lane has not
  > been converted, so **its `/EHsc` intersection is empty and reads exactly like
  > a lane that verified those flags** — `docs/GAPS.md` §7's defect, still open
  > here. Note the specific shape of the hazard before assuming it is small: this
  > lane's whole reason for existing is that it finds mis-emits the hand-written
  > corpus does not, and 35,964 `eh-bare` functions are in class on the workload
  > with markers that only appear under `/EHsc`.
* **Everything on the refusal frontier below.** Those are compiled and counted
  and named, but the port refuses the TU, so no bytes were compared. They are
  **unmeasured**, not green.

## Result, 2026-07-31 (master `ded71a4` merged into this branch)

**0 mismatches**, all four of *this lane's* modes (packed / `/Gy` / `/O1` /
`/O2` — not the 12-lane fixture registry, which did not exist yet and which this
lane still does not use), 19,604 gradings.

* **86 of the 171 unordered family pairs occur in no matched TU of the fixture
  corpus or the whole 6,365-case sweep corpus** — nothing had ever graded them.
* **163 of the 171 emit somewhere in this lane** and are now graded.
* **8 pairs are the TU-level refusal frontier** — they never emitted in *any*
  configuration, at any arity or mode:

  ```
    call-sequence       + float-leaf / double-leaf
    call-sequence-lit   + float-leaf / double-leaf
    call-sequence-value + float-leaf / double-leaf
    framed-call         + float-leaf / double-leaf
  ```

  That is **exactly the configuration of §6 #12**, and it is #13's outstanding
  debt in one line: *the gate that kept the wrong label rule latent is still
  there, so the lane can prove the port does not mis-emit these — only because
  it emits nothing at all.* Verified not to be an artifact of the namespace
  wrapping: `float f(float,float){…}` beside `int F(int a){return g(a)+1;}` in a
  plain flat TU is `Port=NotImplemented` too. The day that gate comes off, this
  lane is what grades what comes out.

## Running it

```sh
scripts/cross_sweep.sh                       # full, ~70 s cold
C2RS_CROSS_REPS=1 scripts/cross_sweep.sh     # 1 rep/family, ~15 s
C2RS_JOBS=32 scripts/cross_sweep.sh /tmp/x   # more parallelism, chosen workdir
```

Exit codes: `0` clean · `1` **MISMATCH** (an alarm — the port emitted bytes for
a combination and they were wrong) · `2` a declared family has no representative
(a hole in `scripts/sweep.d/`) · `3` the namespace wrapping is not
coverage-neutral (the instrument is lying). Toolchain absent → `SKIP`, exit 0.
