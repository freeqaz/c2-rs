# W37 — the `&` operator declined on measurement, and the row that could not be ranked

    Tag:       W37
    Slug:      bit-and-declined
    Date:      2026-07-31
    Fixtures:  w37_bit_and_neg.cpp
    Census:    602,703 unchanged — the rung is DECLINED; its measured worth is 0
    Record:    this document, scripts/sweep.d/46-source-line-collisions.py

`expr-call-in-expr-recv-load-then-bit-and` was **102,382 functions, 5.5 % of
everything blocked** and the largest single key on the board after W36's
de-conflation. It was handed over as UNMEASURED with an explicit instruction to
get a completeness figure before committing to a rung shape.

It is **real**, it is **one shape**, and it converts **0 functions**. So does its
sibling `expr-bit-and` (32,381). Together the `&` operator blocks **134,763
functions, 7.2 % of everything blocked**, and admitting the token over the whole
878-TU workload moves the census numerator by **exactly zero**.

What this rung ships instead is the reason nobody could have known that from the
scan: the row carried no completeness bit **by construction**, and now it does.

## The three questions, and the answers

`GAPS.md` §6 says to ask, in this order: *is my row a limit? is it misfiled? is
it real but small?*

* **A limit?** No, and this was settled by reading rather than by building.
  `0x0B` has no production anywhere in the accepting parser — `parse_expr` names
  it in its own header as a byte that rejects the whole function, `IlOp` has
  `Add`/`Sub`/`Mul`/`Div` and no bitwise variant, and no `shapes/*.rs` mentions
  it. There is no older, narrower copy of a rule its siblings already have, which
  is what W35's row turned out to be.
* **Misfiled?** Not in W36's sense — `0B` is a real operator token and the key
  names a construct. But the row was **unmeasurable**, which is the *instrument's*
  version of the same failure and is dealt with below.
* **Real but small?** Real and **zero**. Not small: zero.

## The measurement, in the order it was taken, cheapest first

### 1. The frame and control-flow axes refuted it for free, before any build

Both cross-tables were already in the baseline scan and neither needed a
counterfactual:

| | of 102,382 |
|---|---:|
| `calls-2plus` | **102,379** (99.997 %) |
| `calls-1` | 3 |
| `calls-0` | **0** |
| `cflow-if-1` | **102,370** (99.99 %) |
| `cflow-straight` | 4 |

`GAPS.md` §6 already states the rule this exercises — *"the frame axis refutes
candidates for free, and completeness does not imply it… two independent axes;
check the cheap one first"* — and it names `expr-op-0x99` as the row that should
be refuted this way. That row became this one. §18 proves `calls-2plus` needs a
frame; `cflow-if-1` needs basic blocks. **The population reachable without either
is at most 4**, which is the noise floor.

### 2. The counterfactual: 0 whole, and the successor named

Granting `Blocker::Op(0x0B)` a production in the completeness matcher — the
diagnostic path only, no acceptance change — de-conflates the row 1:1:

| | functions |
|---|---:|
| `…-then-bit-and-and-branch-more` | **102,374** |
| `…-then-bit-and-and-type-int8-more` · `-and-op-more` · `-and-call-more` · `-and-type-real-more` | 8 |
| **`…-then-bit-and-whole`** | **0** |
| | 102,382 exactly |

**Not one function in 102,382 completes on the operator.** 99.99 % of them reach
a conditional branch on the very next token.

### 3. The sibling row, measured in the parser rather than the matcher

`expr-bit-and` (32,381) is the free-standing mask that never reaches `mcall` at
all — 97.2 % `calls-1`, a different population, so its rate could not be borrowed
from the row above (this is W36's own lesson, applied rather than repeated).
Admitting `0x0B` in **`parse_expr`** and rescanning:

| lands in | Δ |
|---|---:|
| `expr-brtrue` | **+32,368** |
| `expr-brfalse` | +8 |
| `assign-dst-not-formal:eof` · `expr-cmp-eq` · `expr-cmp-ne` | +5 |
| `expr-bit-and` | −32,381 |
| **census numerator** | **+0** |

The gate was lifted **in the parser alone**, which is the direction this project
does not test, and the expected over-claim is the interesting part: there is
none. The census gain is 0, so `census/gate disagreement` stayed at 0 — the
parser and the emitter cannot disagree about a token that releases nothing.

### 4. The control group, which is what makes the zero a result

A completeness measure that reported 0 because it *cannot* report anything else
would be worthless. `fixtures/cpp/w37_bit_and_neg.cpp` carries the separator:
`int n_ret_call_mask(const S *p){ return p->Flags() & 7; }` censuses
**`expr-call-in-expr-recv-load-then-bit-and-whole`**. The measure can say
`-whole` for this exact row; over 102,382 real functions it said it zero times.
`GAPS.md` §6's rule about giving a diagnostic a population whose answer is
already known, applied to a *negative* answer.

## What the row actually is

```text
  4c 4f 11 53 4f 01 23 53
  26 <Flags>                       push the method
  b9 <p> 86 43 82 20               LOAD the receiver
  99 86 43 85 20 00                bind it as argument zero
  bd 86 41 74 00 80 05 10 00 00    CALL, int result
  4c                               apply
  33 86 41 74 04                   LITERAL 4
  0b                               the bit-and
  38 <label>                       ...branch if FALSE          <- the whole story
```

`if (p->Flags() & 4) { … }`. The `&` is not the construct; it is one token
inside a **condition**. Taking this row needs a frame, basic blocks, a register
allocator across them and a `/Gy` layout — every one of which is a phase, and
control flow was declined the same day at **718** realized against a 48,102 row.
The `and` instruction is the last thing it needs and the only thing the key
named.

## Estimate vs outcome

Written to `work/w37/ESTIMATE.md` **before** any scan, census or scratch build,
with the pre-filter analysis the brief required.

| | estimate | range | outcome | bias |
|---|---|---|---|---|
| A. counterfactual (`-then-bit-and-whole` with `0B` granted) | **8,000** | 1,500–30,000 | **0** | HIGH — direction **correct**, magnitude total |
| B. the shippable rung | **2,500** | 0–12,000 | **0** | HIGH — direction **correct**, outcome inside the range |

This is the first estimate in five rungs whose **direction** was right, and the
reason is worth more than the number: it did not come from a ratio. The three
prior estimates that missed were all built by scaling something — a
row-to-counterfactual prior, a sibling family's `-whole` rate — and the one that
landed here was built by enumerating **what the four C++ spellings of `x & k` do
to the next token**, in a table, before any of them was counted:

| source | what follows the `0B` | predicted key | measured |
|---|---|---|---|
| `return p->m() & k;` | result annotation | `-whole` | 0 |
| `if (p->m() & k) …` | `38`/`39` branch | `-more` | **102,374** |
| `return (p->m() & k) != 0;` | compare | `-more` | ~0 |
| `x = p->m() & k;` | store or a second statement | either | ~0 |

Three of the four rows are right and the fourth is right in kind. The estimate
said *"only the first is `-whole`, and it is the least idiomatic of the four"*
and predicted under 20 %; the true figure is 0 %. **A prediction about which
token comes next is checkable against the grammar; a prediction scaled from
another row's rate is not**, which is why the second kind has now been wrong four
times running and the first kind was not.

### Where the estimate was wrong, since that is the part worth keeping

It hedged. It named a LOW hazard — *"a single popular inline could be 30,000 of
the row on its own"* — and left it unbounded, which `GAPS.md` §6 already calls an
excuse. The lumpiness was real (102,370 of 102,382 are one shape) and it went the
**other** way: the popular inline was the branch, not the value. The bound was
available for free in the scan that had already been taken, in the `cflow-if-1`
row, and reading it first would have made the estimate "0, range 0–4" and the
whole rung a fifteen-minute decline.

**So the transferable rule is the frame/cflow one, sharpened**: when a row has a
cross-tabulated axis in the scan, read the cross-tabulation *before* writing the
estimate, not after. §6 says the frame axis refutes candidates for free. The
control-flow axis does too, it is printed in the same block, and this row is
refuted by either one alone.

## What shipped: the row can now be ranked

**No widening.** The census numerator is unchanged at 602,703 / 2,462,571.

`Blocker::Op` had no production, so `mark_whole`'s greedy chain stopped dead at
any operator and the pair was reported UNMEASURED — *by construction*, for every
operator row, forever. `GAPS.md` §6 records the twin of this for `expr-op-0x99`
(*"a row with no `-whole` bit sitting at the top of the ranking is not
'unmeasured'; it is evidence that the row is not reaching the classifier"*). This
is the other half: the row **was** reaching the classifier, and the classifier
had nothing to say. From the outside the two are indistinguishable, and both put
a six-figure row at the top of a ranking with no completeness figure attached.

[`BARE_BINARY_OPS`] is now `09 0A 0B 0C 0D`, and membership needs **two** pieces
of evidence rather than one:

1. **a capture witness that the token is bare** — `c2rs census
   fixtures/cpp/w37_bit_and_neg.cpp` prints one per byte, all the same shape
   (`b9 <x> 86 42 75 · 33 86 41 74 <k> · <op> · <consumer>`), pinned by
   `every_grantable_operator_byte_is_one_byte_in_a_capture`;
2. **a 1:1 redistribution over the whole workload** — deltas over all 219 moved
   keys sum to exactly 0, numerator unchanged, disagreement 0. A byte that were
   not bare would desync the matcher and scatter its row across the hex tail.

The relational family `1F`–`24` is deliberately **excluded**: a neighbouring
opcode in the same numeric range is observed carrying a TYPE
(`… 33 86 41 74 01 · 19 · 86 42 75 …`), so "one byte" would be an assumption
there rather than a reading. `1A` is unary; `1B`/`1C` short-circuit to branches
and have 1 and 3 functions on the workload between them.

A `-more` row also now names the construct the greedy chain **broke on**, not
only the ones it granted — guarded so it never fires at `adm.n == 0`, where it
would restate the key's own second blocker. That is what turns `…-then-bit-and`
into `…-then-bit-and-and-branch`, i.e. what turns *"something else blocks this"*
into *"basic blocks block this"*.

### What it made visible on its first run

`expr-call-in-expr-recv-load-then-shr-and-type-ptr-whole2` — **557 functions**,
previously the mute `…-then-shr`. It is the **only** `-whole` the entire
bare-operator family produces on 2.4 M bodies. All 557 are `calls-2plus`, so the
frame axis refutes it for free too, and the family's total takeable population is
**0**.

### Refused, with the measured cost of each refusal

Nothing new is refused — W37 widens nothing — so the table is of the *existing*
refusals this rung measured and chose to leave in place.

| refusal | why | measured cost |
|---|---|---|
| `&` in a value position (`return p->m() & k;`) | the row does not contain it: `-whole` is 0 of 102,382 | **0** |
| `&` feeding a branch (`if (p->m() & k)`) | needs a frame *and* basic blocks; control flow declined at 718 | 102,374, unreachable behind two phases |
| the free-standing `&` (`expr-bit-and`) | admitting the token in `parse_expr` gains **+0** and moves 32,368 onto `expr-brtrue` | **0** |
| `>>`, `<<`, `\|`, `^` as *accepted* tokens | same measurement one row out: the only completeness behind them is 557 `calls-2plus` functions | **0** |
| the relational family in `BARE_BINARY_OPS` | `19` is observed carrying a TYPE, so bare-ness is unproven for that range | leaves `expr-cmp-*` rows UNMEASURED; 4,040 in the largest |

`w37_bit_and_neg.cpp` carries one case per row and censuses **0/10**. It exists
for a reason specific to a *declined* row: the decline rests entirely on which
shapes the workload does and does not contain, so if a later rung admits `0B`
because these bodies look easy, the measurement has to fail loudly rather than be
re-derived from memory.

## Gate evidence

Corpus `dc3-decomp` at `05ca6d09`; final scan on the **clean committed tree**
`acdc084` with `--validate-cache 50` and `--replay-every 25`: 17 entries
re-captured through the real toolchain and agreed, **0 POISONED**; replay
soundness **36 checked, 0 diverged**.

| lane | baseline (master `1b5f1dc`) | W37 |
|---|---|---|
| `cargo test --workspace --release` | 457 pass / 0 fail | **459 pass / 0 fail** |
| `c2rs bench` | 169 pass / 0 fail / 0 error | **170 pass / 0 fail / 0 error** |
| `scripts/mode_lane.sh /Ox` | 79 match, 0 mismatch, 0 codegen-gap | **79 / 0 / 0** |
| `/O1` · `/O2` · `/Ox /Gy` | 77 match, 0 mismatch, 2 codegen-gap | **77 / 0 / 2** each |
| `scripts/expr_sweep.sh` | 10,359 cases, 0 mismatches | **10,630 cases (+271), 0 mismatches** |
| `scripts/cross_sweep.sh` | 11,341 × 4, 0 mismatches | **11,341 × 4, 0 mismatches**, family set unchanged |
| 878-TU scan | 602,703 / 2,462,571 (24.47 %), mismatch 0, disagreement 0 | **602,703 / 2,462,571 (24.47 %)**, mismatch 0, **disagreement 0** |
| `census fixtures/cpp/w37_bit_and_neg.cpp` | — | **0/10 in class**, `Port=NotImplemented` |

The census numerator is **identical to the function** and so is every key that
did not carry an operator second-blocker; the 219 keys that moved sum to exactly
0. W37 adds no shape family, so `cross_sweep.sh`'s configuration count is
unchanged, which is the correct outcome rather than a missed one.

## The generated axis

`scripts/sweep.d/46-source-line-collisions.py`, **271 cases**, one file per axis.
It varies the **source line**, which is the one field in the pre-body region a
*programmer* chooses and therefore the cheapest place for a byte that looks like
an opcode. This project's first live wrong-bytes emit is a member function on
line **70**, whose marker reads `4F 01 46` — the formals marker's own byte — and
what exists today (`44-member-source-lines.py`) sweeps lines 66..74 for one
shape. That is a window around the defect that was found, not a sweep of its
range, which is `GAPS.md` §6's "a rule fitted to the shapes the corpus happened
to contain" in the corpus itself.

Two unswept regions, both closed here: **twenty structurally-colliding byte
values** as line numbers (`LO`, scope open and close, RETURN, the branch, the
result annotation, the formals entry, the symbol push, LOAD, CALL, the bind, the
store, the marker's own opcode), crossed with **ten accepted shape families**
each taking two formals so that a vanished formals list cannot pass silently;
and the **varint width boundary at 127/128/255/256**, which no fixture in this
repo has ever crossed — so no accepted shape has ever been graded with a
multi-byte line marker anywhere in its segment — with and without a second
function behind it, plus one case where the boundary falls *between* two accepted
functions. It varies no operator and no shape, so every case has a known right
answer. **0 mismatches**; this axis found nothing, which is a result and not a
pass.

## Found and not taken

Ranked, with the frame axis applied first because it is free.

1. **A member call preceded by assignment statements — ~10,000, and still the
   cheapest thing on the board.** `expr-call-in-expr-recv-load-whole` is 10,494
   with **10,463 `calls-1`**; it is W36's own item #1 and nothing here has
   touched it. The dispatch only offers the member-call production the *first*
   statement, so a body whose call follows `int x = a;` goes to the assignment
   parser and has nowhere to hand the call.
2. **`expr-shr` — 3,686, and the only operator row with a real `calls-0`
   population: 2,146.** Every other operator row is `calls-1` or worse. Its
   control-flow split is `cflow-loop` 1,382 / `cflow-straight` 812 / `cflow-if-1`
   724, so the takeable population is bounded by ~812 and needs `srawi`/`slwi`
   selection with the shift-count range enumerated rather than fitted. Small, but
   it is the only place in this family where the two free axes do not both say
   zero. **The one thing this rung leaves genuinely open in its own seam.**
3. **`expr-call-in-expr-data-addr-2sym-then-plain-call-and-type-ptr-whole2` —
   18,925 `calls-1`, and it is a trap.** `GAPS.md` §6 records `data-addr`
   realizing **0** against an 11,000 estimate because c2 derives every address
   after the first from a whole-TU pool layout that no per-body grammar can
   express — and `2sym` is exactly two addresses. Do not schedule without
   re-reading that entry.
4. **`expr-call-in-expr-chained-whole` — 12,479, all `calls-2plus`.** Needs a
   frame first; Class C was declined at 0.
5. **The relational family's bare-ness — UNMEASURED, and it is the riskiest
   thing this rung leaves.** `1F`–`24` are excluded from `BARE_BINARY_OPS` on the
   strength of one neighbouring opcode (`19`) observed carrying a TYPE. That is
   an *inference from an adjacent byte*, which is the same class of reasoning
   this project has been wrong about three times (`call-anchor-*`, `expr-cast`,
   the relational names themselves). It costs coverage of the instrument, not
   correctness — the `expr-cmp-*` rows stay UNMEASURED, 4,040 in the largest —
   and the fix is one probe: a `.cpp` whose comparison result is consumed as a
   value, censused with `--keep-il`, read once. It was not taken because the
   rows behind it are small and the rung was already a decline.
6. **A second-order consequence of #5 worth naming separately.** Every row this
   rung made measurable turned out to be worth 0, so the instrument change bought
   *ranking* and not coverage. If the relational bytes are bare too, the same
   change would reach `expr-cmp-eq`/`-ne`/`-lt`/… — and those feed the comparison
   leaf the port **already emits**, which is the one operator neighbourhood where
   a `-whole` count could be non-zero. That is the reason to spend the probe.
