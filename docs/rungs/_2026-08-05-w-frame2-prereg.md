# w-frame2 — PREREG

    Tag:       w-frame2-prereg
    Slug:      w-frame2-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg. It admits no shape and changes no file
               under `crates/`. There is nothing a fixture could grade.
    Census:    unchanged, +0 — no `crates/` file is touched by this commit.
    Record:    this file. Committed **before any grid of this lane exists**.
    Lane:      w-frame2, worktree `wt-w-frame2` off master **`44aa8ce`**.
    Ships:     this file.

---

## 0. Why this file exists before anything else

Five consecutive lanes here have found their answer **outside** the class they
searched, and every one that ran the exhaustive negative first had the
residual's shape name the mechanism. This file registers, before a single cell
is compiled: what I was told, what I re-derived, what I predict, what class I
will kill first, and which cells the fitter is forbidden to open.

---

## 1. The brief's premise, RE-DERIVED — and REFUTED

I was told `xboxheap.cpp` stands **three facts** from converting, and that
landing them takes TU match to 10. **That is not what the tree says.** The
re-derivation below is entirely code- and oracle-anchored; none of it is a
model.

### 1.1 The three named facts all stand, and they are facts 4, 5 and 6

| # | fact as briefed | re-derivation at `44aa8ce` | verdict |
|---|---|---|---|
| 1 | **#602** — the LAYOUT does not generalise; `schedule()` still refuses | `codegen/order.rs:283` `schedule()` calls `rank_order()`, which returns `None` at `order.rs:181` whenever `!single_symbol(stmts)`. The refusal is exactly where the brief says. | **STANDS** |
| 2 | the post-call `mr r3,r31`; `framed_call_text`'s post-op vocabulary is `addi r3,r3,k` only | `codegen/calls.rs:202` `framed_call_text` emits `encode_addi(RET_REG, RET_REG, k)` **unconditionally**, and the IL type it reads is `c2_il::FramedCall { callee: String, add_k: i32 }` (`c2-il/src/func/mod.rs:255`) — a single `i32` is the entire post-op representation. | **STANDS** |
| 3 | a framed body with a store run BETWEEN the prologue and the call; `Selected::Framed` has no representation for it | `Selected::Framed { setup: Vec<u8> }` (`select.rs:120`) is a byte vector, so the *emitter* type could carry one. The binding refusal is one layer down: `select_function` fills it from `select_text(func, mode)` with the trailing `blr` truncated (`select.rs:163-168`), a **leaf** selector over `func.ops`, and `func.ops` for this shape is `arg_ops` — the bare LOAD of one formal (`bundle.rs:875-886`). | **STANDS, relocated** — the missing representation is in `crates/c2-il`, not in `Selected::Framed`. |

**R0 (registered).** Fact 3's locator as briefed (`select.rs`) is not where the
refusal lives; the binding one is `c2_il::FramedCall`, which has no field for a
store run. I register this now so it cannot be claimed after the fact.

### 1.2 The premise is refuted: there are SIX facts, and the binding three are PARSE facts

`w-parse` priced this TU at **6** — three parse facts (F1 a literal-valued store
mixed into a store run, F2 a member's address as a stored value, F3 a call after
a store run) plus the three above. The brief carries only the last three. **The
parse facts are still live, measured on this tree with the real toolchain:**

```
c2rs gap --list <xboxheap alone> --flags-file work/dc3-workload/flags.txt
  1 |  0/1  | src/xdk/nuispeech/xboxheap.cpp [vocab-gap]
  blocking features (the widening order):  1 (100.0%)  expr-op-0x27
```

and, **the measurement no lane has published**, with the one experimental sink
that exists (`C2RS_SINK_OFF_ADD_ARG=expr`, board #364) promoted:

```
  blocking features (the widening order):  1 (100.0%)  expr-op-0x32
```

> ### **`xboxheap`'s parse chain is at least two operators deep.** Promoting the `0x27` sink does not decode the TU — it moves the first refusal to **`expr-op-0x32`**, a token nothing in the tree models. This is board #441's fall-through mechanism firing on the one TU somebody would pick from a blocker table, and it means **no amount of emitter work converts this TU in this lane.**

**R1 (registered, and the whole lane hangs on it).** `xboxheap.cpp` does **not**
convert here and **TU match ends at 9.** I register this before measuring
anything so that it cannot become an excuse later. If it converts, R1 is refuted
and I will say so in the same words.

**What I will do instead**, because it is the largest thing on the list that a
grid can actually settle: **close #602, the LAYOUT.**

---

## 2. What #602 is, stated so it can be wrong

`order.rs`'s layout clause (shipped inside `order::schedule`) is:

> let `u` = the number of head store slots; the first `u` producers go one
> apiece immediately before store slots `0 … u−1`, and every remaining producer
> is emitted contiguously immediately before store slot `u`.

Equivalently: **producer at emission index `i` sits immediately before store
slot `min(i, u)`.** `w-sym` §6 reports it 1866/1867 fit, 1644/1645 holdout and
**12 of 16 external**, with the four external misses one family:

```
 syms 000011   P0 · stw m0 · P1 · stw m1 · stw m2 · stw m3 · stw e4 · stw e5
 syms 001011   P0 · stw m0 · stw m1 · P1 · stw e2 · stw m3 · stw e4 · stw e5
```

Layout `[0,1]` against layout `[0,2]`, same statements, same store order, same
producer order, same registers.

**R2 (the exhaustive negative, to be run FIRST).** The class
`slot(i) = min(i, u)` for **any** scalar `u` cannot produce `[0, 2]`: at `i = 1`
the slot is `min(1, u) ≤ 1` for every `u`. So the *whole* shipped family is dead
on this cell by construction, and re-deriving `u` — which is where a lane would
naturally start — cannot possibly help. **I predict the search must leave the
`min(i, u)` shape entirely**, and I register that the first thing I will do is
enumerate the wider class of §2.1 and report its ceiling before fitting
anything.

### 2.1 The class I will enumerate first

**Class L — every "per-producer release slot" layout.** Producer at emission
index `i` is emitted immediately before store slot

    slot(i) = clamp( max over a chosen subset of FLOOR terms , 0 , n_stores )

with the floors drawn from the run's features, then made non-decreasing in `i`:

| floor term | |
|---|---|
| `i` | the emission index — the shipped rule's only term |
| `u_count`, `u_lead`, `u_walk` | `min(2,#unproduced)`; the leading unproduced run in the final store order; the `u` `store_order` actually selected |
| `fc(i)` | the slot of the producer's first consumption in the final store order |
| `fcg(i)` | the same, counted **within its own symbol group** |
| `grank(i)` | its rank among its own group's producers |
| `nsym_before(i)` | the number of symbol-group *changes* in the final store order strictly before `fc(i)` |
| `0`, `1`, `2` | constants |

crossed with a cap term (`min` against any of the same list) — a few thousand
configurations, enumerated exhaustively rather than sampled. **R3: I predict the
top of class L scores ≥ 99 % on FIT.** If it does not, the answer is outside
this class too and the residual is the finding, exactly as it was for `w-sched`
(13,104 list schedulers), `w-order2` (1,048,576 release times) and `w-sym`
(8,420 sort keys).

**R4.** I predict the winning term is `nsym_before` or `fcg` — i.e. **the layout
counts symbol-group structure, not producers** — because that is the only axis
on which the two rows of §2 differ at all. Registered as the hypothesis I most
expect to be wrong, in the same slot `w-parse` used for its R2 and got refuted.

**R5.** The single-symbol regime is unchanged by whatever wins: on one symbol
every candidate above must reduce to `min(i, u)`. I will assert this in code as
a reduction, not as a claim beside the code, the way `order.rs`'s single walk
does — **and I predict the 5,460-run enumerating test in `order.rs` passes
unchanged.**

**R6.** Kind-independence (board #603) **fails** for the layout the same way it
fails for the store order: I predict the mixed-kind holdout tier scores strictly
worse than the single-kind tiers, by ≥ 2 points.

---

## 3. The grid, and the trap it must be able to fall into

**Three lanes have now built a grid that could not contain its own
counterexample**, the third being the lane that had been warned. So, registered
before the generator is written:

1. The grid **must contain the `[0,2]` shape**. Concretely: cells whose symbol
   mask interleaves (`0,0,1,0,1,1`) and not merely partitions (`0,0,0,0,1,1`),
   because the two rows of §2 differ on exactly that. A grid of block masks
   cannot produce the counterexample and would report the shipped rule at
   ~100 %.
2. The grid **must contain single-symbol cells**, so that R5's reduction is
   scored on a population and not asserted.
3. The grid **must contain runs long enough that `slot(i)` can exceed 2** —
   `w-sym`'s tier S stops at 4 statements, and the counterexample needs 6.

**Falsification check, registered:** if my fitted rule says `slot(1) = 2` for the
interleaved mask, the grid contains block-mask cells with the same word where it
must say `slot(1) = 1`, and vice versa. Both are present by construction.

---

## 4. The HOLDOUT, declared here and written by the GENERATOR

The partition is decided by the generator from the cell's own description,
before any listing is read, and the fitter **raises** on a path containing
`holdout` — `w-sym`'s `symlib.read_rows`, whose refusal is demonstrated on four
spellings in `work/w-sym/raise_check.py`. I will demonstrate mine the same way.

Held out, **wholesale by shape** wherever possible — `w-sym`'s entire-tier
holdout came back 406/406 and that is much stronger than a random split:

1. every cell with **≥ 3 symbol groups** — the whole arity tier;
2. every cell with **mixed producer kinds** — the whole `#603` tier;
3. every cell with **≥ 3 distinct producers**;
4. every cell of **length ≥ 7**;
5. otherwise, `md5(cid)[0] ∈ "012345"`.

`xboxheap`'s own word and its symbol-mask twins are **EXTERNAL** — in neither
partition, scored last, and never fitted on.

**The rule is frozen at a committed SHA before the holdout is opened once**, and
that SHA is quoted in the findings doc.

---

## 5. What I will ship, and the property it must have

A **positive guard**: `order.rs` gains a `layout()` (or `schedule()` widening)
that ANSWERS where it returned `None`, and its only consumer refuses when the
answer disagrees with what the emitter was about to do.

**R7.** The change must be **additive-refusal by construction** — `Some(false)`
/ a disagreement is the only reading a new answer can produce, so a new answer
can *add* a refusal but can never turn a refusal into an accept. I register that
I will state this explicitly and point at the line that makes it true, and that
if I cannot, I ship nothing under `crates/`. **I am widening code that emits
bytes; #232 is a widening that turned a clean refusal into a live wrong emit for
255 commits.**

**R8.** The workload scan is **baseline-identical**: match 9, mismatch 0,
vocab-gap 862, capture-fail 7; A/B/C/D/E = 28 (LO 27)/338/169/9/2; `A∧B∧C` 27;
FRONTIER 18. Any digit that moves is a defect in my change, not a result.

---

## 6. Gate expectations, registered BEFORE the run

Per the brief, quoted back so a truncated run cannot read as healthy:

| | registered |
|---|---|
| `cargo test --workspace --release` | **843 passed / 0 FAILED / 27 targets** — plus whatever tests I add. **The target count is the number to read**: it stops at the first failing target, so a truncated run reports fewer passes AND fewer targets. |
| `scripts/gate.sh --jobs 6` | PASS, **18/18**, **4,500 verdicts** (no fixture is added, so the count must not move); sweep 16,394 / 16,298 graded / **96 ungraded**; cross 75,829 of 76,217 / **388 ungraded** / 0 mismatch |
| `scripts/board_audit.sh` | 0 / 0 / 0 |
| `df -i /tmp` at lane start | recorded in the findings doc |

---

## 7. Board numbers

Highest is **#605**; I start at **#620** as instructed and will report what I
took.

---

## 8. The one-shot gate

Unspent, held by twenty-three lanes. I do **not** intend to spend it: a
mechanical holdout written by the generator against a frozen commit *is*
out-of-sample validation. I will ask and stop only if I end with free parameters
my own grid cannot validate.
