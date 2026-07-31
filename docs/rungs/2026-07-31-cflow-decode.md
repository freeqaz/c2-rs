# WCF — the control-flow grammar, decoded and censused; nothing lowered

    Tag:       WCF
    Slug:      cflow-decode
    Date:      2026-07-31
    Fixtures:  wcf_shapes.cpp wcf_neighbours.cpp
    Census:    491013 → 491013 (19.94% → 19.94%), +0
    Record:    this file; the grammar itself is docs/IL_STMT_GRAMMAR.md

**The census moved by zero, and that is the intended outcome.** This rung is
decode-only by construction: it adds no arm to `parse_segment_shape`'s ladder, it
constructs no `BodyShape`, and every body it newly understands still returns
`NotImplemented`. What it produces instead is the measurement the block-IR
restructure has never had — how many bodies have control flow, which shapes, and
**what each would be worth if it were lowered**.

The headline of that measurement, stated plainly because it is the number every
plan for this work has been guessing at:

> Of 2,462,571 functions, **269,121 (10.9 %) have a decoded control-flow shape**
> beyond a single basic block. Of those, **718 are blocked on control flow
> alone**. The other 268,403 need expression-layer work as well, and would not
> move if a complete block IR landed tomorrow.

---

## What it admits, and what it refuses

**Admits: nothing.** `c2rs bench` 159/159, four mode lanes 74/72/72/72, and the
census numerator is byte-for-byte the value it was at `0fa82f1`. The control
group proves the "nothing" positively rather than by absence: over the whole
878-TU workload, **all 455,049 in-class bodies that the scanner reads report
`cflow-straight`** — every shape the port accepts is a single basic block, so a
`cflow-loop` among them would have indicted the measure.

(The 35,964 in-class bodies the scanner does *not* read are exactly the generated
empty destructors — the `dtor-callee-1` count, to the function. Their sub-object
trailer is opaque by design in `try_parse_empty_dtor_delegation`, and the two
numbers agreeing exactly is a second, independent check that the scanner's
segmentation is the census's.)

**Refuses, by name.** Six control-flow opcodes had been reaching the census as
hex. They are capture-verified now and rendered from one shared table
(`body::cflow_opcode_name`), read by both the `expr-*` keys and `mcall`'s
second-blocker keys so the two cannot drift:

| was | is | functions |
|---|---|---:|
| `body-0x29` | `body-cflow-label` | 48,102 |
| `expr-op-0x38` | `expr-brfalse` | 26,501 |
| `expr-call-in-expr-recv-object-then-branch-0x39` | `…-then-branch-brtrue` | 23,633 |
| `return-scope-close-0x29` | `return-scope-close-cflow-label` | 13,184 |
| `call-ref-0x3A` | `call-ref-cflow-jump` | 5,335 |
| `expr-op-0x39` | `expr-brtrue` | 3,097 |
| `expr-op-0x3A` | `expr-jump` | 923 |
| `expr-op-0x3B` | `expr-switch-dispatch` | 36 |

Every one is a **1:1 rename** — no bucket merges, no recorded comparison is
invalidated, and the totals above are unchanged from the run before.

**The polarity was undetermined and is now determined.** `mcall`'s
`Blocker::Branch` recorded `38` vs `39` as UNKNOWN because its two wild witnesses
could not separate the senses (a branch to a later label fits either reading).
`wcf_shapes.cpp` settles it with a controlled pair in one TU, differing only by a
`!`:

```
if (a)  return 1; return 2;   b9 <a> 86 41 74  38 <L>  53 …then… 54 04  29 <L>
if (!a) return 1; return 2;   b9 <a> 86 41 74  39 <L>  53 …then… 54 04  29 <L>
```

Both load `a` itself — the `!` never becomes an opcode — and both define `<L>`
*after* the then-clause, so `<L>` is "skip the then" and the branch to it is
taken when the condition is false. Negating the condition swaps `38` for `39`.
`&&` / `||` corroborate independently in the same file: `a && b` emits `38`
twice (short-circuit on false), `a || b` emits `39` then `38`.

---

## The grammar, as decoded

`crates/c2-il/src/func/body/shapes/control_flow.rs`. The statement layer is a
**flat token stream**, not a tree, so the scanner is one loop over `item`:

```text
item := 4F 01 <varint>     source LINE number, any number of them, everywhere
      | 53                 open a scope
      | 54 <k>             close it;  k == the depth remaining AFTER the pop
      | 29 <tok>           define label <tok>
      | 38 <tok>           branch if the popped value is FALSE
      | 39 <tok>           branch if the popped value is TRUE
      | 3A <tok>           unconditional branch — ALSO break / continue / goto /
      |                    return, which have no opcodes of their own
      | 3B <tok> | 3C <TYPE> <tok> | 3D <tok>     switch dispatch/table/case
      | 4B                 end of an expression statement; discards the value
      | <one operand token, stepped over by WIDTH only>
```

A body counts as decoded only when **both** hold, which is what makes the claim
falsifiable rather than merely consistent:

1. the walk lands **exactly** on the 7-byte function tail `4F 12 47 54 01 54 00`;
2. every `54 <k>` satisfies `k == the depth remaining after the pop`.

Measured on the workload, varying one thing at a time:

| variant | bodies decoded end to end |
|---|---:|
| as implemented | **1,864,128 / 2,462,571 (75.7 %)** |
| TYPE read as a fixed 3 bytes | 350,991 (14.3 %) |
| depth invariant (2) disabled | 1,864,132 — **+4** |

The first row of the falsification is the one that matters: the widths are
load-bearing, not decorative. The third says the depth invariant catches 4 real
desyncs across 2.46 M bodies — small, and worth keeping anyway, because those 4
are bodies that would otherwise land on a *wrong* function tail, which is the
over-acceptance mode this grammar has (`IL_STMT_GRAMMAR.md` §13 measured 34 such
landings under a fixed-width TYPE).

### The shapes, and what each is worth if lowered

`cflow-<shape>` is the population the block IR must serve.
`cflow-<shape>+expr-modeled` is the subset whose operand stream is **already
inside the class the port has been byte-graded on** — i.e. what lowering that
shape, and nothing else, would actually gain. The membership test is the same one
the accepting parser applies at the same positions, so the second column cannot
over-claim.

| shape | bodies | of which blocked on control flow ALONE | top co-blocker |
|---|---:|---:|---|
| `cflow-straight` (control) | 1,595,007 | 276,271 | — |
| `cflow-if-1` | 171,775 | **713** | `expr-op-0x99` 103,343 |
| `cflow-loop` | 63,212 | **0** | `body-cflow-label` 24,413 |
| `cflow-if-2` | 24,918 | **0** | `expr-op-0x27` 17,450 |
| `cflow-if-n` | 9,097 | **0** | `expr-cmp-eq` 2,987 |
| `cflow-switch` | 119 | **5** | `expr-shr` 33 |
| `cflow-multi-exit` | **0** | 0 | — |
| undecoded (`cf-*`) | 598,443 | — | `cf-expr-0x5C` 309,804 |

Four things in that table are worth more than their row.

1. **The block IR alone is worth ~718 functions, 0.03 % of the workload.** Not
   269,121, and not the 48,102 that `body-0x29`'s size suggests. Every branching
   body but 718 is *also* waiting on an expression production. This is the
   demand-driven-widening rule landing again, and it is the reason this rung
   deliberately did not attempt the lowering.
2. **`cflow-multi-exit` is zero.** Several returns with no conditional anywhere
   does not occur in 2.46 M real functions; it takes a bare `goto`, which
   `wcf_neighbours.cpp::edge_goto_only` is the only witness of. The shape keeps
   its name because a lowering has to answer for it, but it is not a rung.
3. **A loop is not "an `if` with more branches."** It is a branch whose target
   label is defined *earlier in the byte stream*, and `3A` carries no direction —
   forward and backward are the same opcode, decided only by where the `29`
   happens to sit. That is why the scanner records site *positions*, and it is
   why `Loop` is a shape rather than a `Forward` with a bigger number: a back
   edge needs register allocation across it, which is the frame/liveness spine's
   work, not the block IR's.
4. **The conditional expression is invisible to this axis.** `a ? b : c` is
   `43 42 <2 bytes>` in the *operand* stream, so a body containing one reads
   `cflow-straight`. c2 lowers it as two exits and a `bclr` — control flow
   wearing an expression's clothes. `wcf_neighbours.cpp::edge_ternary` pins the
   limit so a reader of the histogram knows it is not counted.

---

## Two live instrument defects found and fixed

Neither is a mis-emit; both are the census lying about its own contents, which
this repo treats as the failure a measuring instrument cannot survive.

**1. `call-multiarg-postop:eof` — 13,425 functions filed under "end of segment"
at a position nowhere near one.** The refusal is about a byte ("the token after a
multi-argument call's `4C` is not the `41` result annotation") and it was
constructed with `byte: None`, so `Block::feature` printed the no-byte spelling
and the one distinguishing byte was discarded. The brief for this session
recorded the bucket's composition as *unsampled*; it is now sampled, because the
key carries the byte:

| key | functions | what it is |
|---|---:|---|
| `call-multiarg-postop-0x1A` | 7,621 | `!g(a,b)` — logical not on the result |
| `call-multiarg-postop-0x2C` | 3,514 | a conversion of the result |
| `call-multiarg-postop-0x33` | 1,602 | a literal, i.e. an arithmetic post-op |
| `call-multiarg-postop-0x30` | 688 | an indirect load through the result |

The four sum to 13,425 exactly. The largest half is a *unary not*, which no
reading of the old name would have suggested.

**2. My own `44 <TYPE>`, caught by the instrument in the same session.**
`IL_CALL_GRAMMAR.md` §7's superseded reading gives `0x44` a TYPE;
`IL_EXPR_LAYER.md` §7 corrects it to payload-free. The first cut of this scanner
used the old reading, and the workload reported 112,389 bodies blocked at the
byte `55` — a byte that cannot be a TYPE tag, i.e. a desync — under a key
(`cf-op-type`) shared by seven opcodes, so *which* one had desynchronized was
unrecoverable. The keys are now one per opcode, and fixing `44` moved the decode
reach 75.7 % → 81.1 % before the conservatism below took it back.

---

## What is deliberately NOT decoded, and what that costs

An opcode whose payload no capture has established refuses at itself rather than
being stepped over at a guessed width. A guessed width that *fails* is visible; a
guessed width that *succeeds* silently desynchronizes the rest of the body, and
there is no counter for those. So the following are honest `cf-expr-0xNN` rows,
and their sizes are the measurement of what establishing them would buy:

| opcode | bodies it stops | note |
|---|---:|---|
| `5C` | 309,804 | trailing byte after `<TYPE>` is 1/2/3 — looks like `<TYPE> <varint>`, unwitnessed |
| `64` | 107,919 | `IL_CALL_GRAMMAR.md` §7 lists it unidentified |
| `67` | 44,859 | the first cut read `67 <TYPE>` and desynced 29,687 bodies |
| `05` | 27,285 | presumably `/`, but §5's table stops at `%`=`06` and does not say |
| `82` | 22,899 | in §13's residue list |
| `59` | 15,913 | appears between two FP arithmetic ops |

Establishing `5C` alone would move the decode reach by ~12 pp. It is expression
layer, not this rung.

---

## Estimate vs outcome

Recorded before building, with the bias direction and its named cause.

| quantity | estimate | outcome | bias |
|---|---|---|---|
| census numerator | 491,013, **+0** | 491,013, **+0** | none — not an estimate; no acceptance path is touched |
| statement-layer decode reach | 52–54 % (Dir.cpp 53.3 % and my Python pass over Ham.cpp 52.2 %) | **75.7 %** | estimate **low**, cause named in advance: both priors came from scanners with a varint `9B` and a fixed-4-byte `43`, and the Rust one uses the repo's corrected readings |
| shape split among decoded bodies | straight 85 %, if 13 %, loop 1.6 %, switch <0.5 % | straight 85.6 %, if 11.0 %, loop 3.4 %, switch 0.006 % | straight **called high** in advance ("Ham.cpp is template-heavy header code and unusually flat"); it was not, and loop was underestimated 2× |
| new census keys | +5 to +15, biased low | 576 → 579 (+3) | estimate **high**; the `call-multiarg-postop` split found four bytes, not a dozen |

The estimate that mattered was not on the list, and that is the honest criticism
of it: nothing predicted **718**. The counterfactual had to be built to get it,
and the brief was right that the measurement is the rung's most valuable output.

---

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | **428 pass, 0 fail** (416 → 428; +12: 11 in `control_flow`, 1 control-group assertion in `census`) |
| `c2rs bench` | **159 pass, 0 fail, 0 error** (157 → 159; the two new fixtures) |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **74 / 72 / 72 / 72**, mismatch **0** in all four |
| `scripts/expr_sweep.sh` | **6,062 checked, 0 mismatches** (5,922 → 6,062; the new fragment emits 140) |
| 878-TU workload scan | match 6 · **mismatch 0** · census **491,013 / 2,462,571 (19.94 %)** · **disagreement 0** · binding violations 0 |
| fixtures, `c2rs census` | `wcf_shapes.cpp` **0/14** (all refuse, all 14 shapes decoded) · `wcf_neighbours.cpp` **6/12** (6 control-group in class and all `cflow-straight`, 6 edge cases refusing at named keys) |

Binding counters unchanged: `dtor-callee-1` 35,964, `gl-token-ambiguous-dropped`
1,734, `gl-token-conflict-mangled` 7.

---

## Found and not taken

Ranked by what a rung would be worth, with the frame axis applied where it
changes the answer.

1. **`5C <TYPE> <varint>` — 309,804 bodies of decode reach, 0 functions of
   census.** The single largest thing standing between the statement-layer
   scanner and the corpus. A probe that varies the construct until the trailing
   byte moves off `01` establishes it or refutes it in one capture. Decode-only
   again: it buys ranking accuracy, not coverage.
2. **`expr-op-0x99` under `cflow-if-1` — 103,343 bodies, the largest single
   (shape × blocker) cell in the whole cross-tab.** `99 <TYPE> <varint>` is the
   member bind; these are member functions with one `if`. They need *both* the
   member-bind production and the block IR, and neither alone moves one of them.
   That pairing is the argument for sequencing the expression layer first.
3. **`expr-op-0x27` across the branching shapes — 40,470** (17,394 under `if-1`,
   17,450 under `if-2`, 4,535 under `loop`, 1,091 under `if-n`). The byte-offset
   add is already the #1 census row at 461,786 overall; this says ~9 % of its
   population is *also* behind control flow, so a `27` rung's own estimate should
   be discounted by that much rather than claiming the whole row.
4. **The block IR itself — 718 functions.** Worth stating as a rung so it can be
   ranked and declined: it is a serial restructure (`ARCHITECTURE_SEAMS.md` §7)
   with the frame/liveness spine in front of it, for 0.03 % of the workload
   *today*. Its value is entirely latent and entirely conditional on the
   expression layer landing first — at which point the 269,121 becomes reachable.
   The right time to build it is when `cflow-if-1+expr-modeled` has grown, and
   this axis is now the instrument that will say when.
5. **`cflow-loop` is 63,212 and worth 0.** Not a rung at any point soon: a back
   edge needs the register allocator to work across it, so it is behind the spine
   *and* behind the block IR *and* behind the expression layer.
6. **`cflow-switch` is 119 bodies in 2.46 M.** Recorded so nobody sizes it from
   the language's prominence. Its grammar is decoded and refused precisely; that
   is the whole of what it deserves.
7. **The `43` sub-opcode space.** `42` (conditional) and `37` (bitfield) are
   witnessed; everything else refuses as `cf-escape-43`. `IL_EXPR_LAYER.md` §8
   asks for the census key `expr-op-0x43-NN` rather than the current
   `expr-ternary`, which is a generalization from one sub-opcode. Not done here —
   it needs `aux` plumbing in the `expr` arm and belongs with an expression rung.
8. **Roadmap #13 is untouched and still open.** `split_function_bodies`
   under-counts ~1,972 bodies. This rung's scanner shares that segmentation
   exactly (by construction — it runs inside the census), so its percentages
   carry the same ~0.08 % error and no new one.
