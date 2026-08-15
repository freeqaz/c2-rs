# w-labeltable — the table was RIGHT the whole time: 17 of 17 rows hold against the oracle, the contested row settles the OTHER way, and the third overstated number is in `work/w-bdnz` and is quoted into shipped code

    Tag:       w-labeltable
    Slug:      labeltable
    Date:      2026-08-15
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization: is `docs/LABEL_COUNTER.md` §4.2.1 right? Every row re-measured against real `c2.dll`, as a series. This lane lifts nothing and claims no prefix; a fixture would move the graded tree, which its own prereg registers as unchanged
    Census:    unchanged → unchanged, +0 — nothing is admitted, no `crates/` byte moves
    Record:    this file; prereg `work/w-labeltable/PREREG.md`, committed at `c0999646` **before the first `cl.exe`**; instrument `work/w-labeltable/table.py`; output `rows_o1.txt`, `rows_ox.txt`, `ladder_o1.txt`, `framed_o1.txt`, `bdnz_o1work.txt`, `bdnz_oxwork.txt`

**`Outcome: instrument`.** The deliverable is a corrected measurement of a
published table, and the correction is that **the table needed none**.

> ## The headline, and it is the opposite of the brief's premise
>
> This lane was dispatched on *"an overstated price is self-preserving, because
> nobody re-measures a fence that already looks too expensive to lift"*, with
> §4.2.1's `leaf-ptrwalk` row named as **"known one high since #3091"** and
> *"one command from settled"*.
>
> **It is settled, and it was not one high. Nor was any other row.**
>
> ```text
>   §4.2.1, 17 rows, /O1 :  17 AGREE   0 disagree
>   §4,      6 framed rows:   6 AGREE   0 disagree   (with §4's OWN minted correction)
>   both instruments      :   0 disagreements of 18 at /O1, 0 of 17 readable at /Ox
>   series                :  16 of 18 discriminating; every row c = 0, residual 0
>   mutants               :  50 red, 0 green; 18 of 18 separating controls green
>                            at EXACTLY 0 label symbols
> ```
>
> **The direction of this document's errors is now measured and it is zero.**
> Three published label numbers *have* been overstated in the fence-preserving
> direction — `#3091`, `#3148`, and `LABEL_LEAD.md`'s own finding 1 — and **none
> of them was this table**. Every one was a number *quoted out of* it into a
> class it does not describe, or a lead **differenced across two TUs**.
>
> **So the pattern `#3148` named is real, and this lane relocates it.** It is not
> "the label counter's published tables are wrong". It is **"a number stops being
> graded the moment it is copied out of the instrument that produced it"** — and
> the copies are in fixture headers, in `work/` write-ups and, in the instance
> this lane found still live, in a **shipped doc comment**.

---

## 1. The five results, in one table

| | |
|---|---|
| **§4.2.1, all 17 rows** | **HOLD.** 17 of 17 reproduce exactly at `/O1`, on two independent seed-free instruments, as a series over `n = 1, 2, 3` |
| **the contested `leaf-ptrwalk` row** | **SETTLED, and it is not one high.** It charges **3**; `?HashString` charges **2**; both measured in the same run by the same instruments. They are **different shapes**, and a 5-cell ladder puts the whole difference on **one token** |
| **`work/w-bdnz/LABEL_LEAD.md`** | **A CROSS-TU ARTIFACT.** Its eight `$M` values reproduce to the digit; every published lead is that lead **plus the cell's own `.gl` counter gap**. `+7` is **2**; `+8` is **2**; the `+2` two-locals control is **0** |
| **the sentence in shipped code** | `IlFunction::label_slots`' `counted_accum_loop` arm says *"§4.2.1's `for` row … a lead of `+1`, where the obj says **+7**"*. **Both halves wrong, in opposite directions.** Corrected: **+2 and +2 — they agree.** NOT edited here; filed |
| **`/Ox`** | a whole column §4.2.1 never had, both instruments agreeing on 17 of 17 readable rows, one marked **CONFOUNDED** rather than quoted |

**Nothing is lifted and nothing under `crates/` is opened.** §7 is the two-sided
statement of what the corrections do and do not buy.

---

## 2. The method, and the one thing that made the answer trustworthy

### 2.1 The series, applied because `#3147` said to and not because it paid

`w-slots` followed *"read the charge out of the fixture's own obj"* and the objs
read **3** where the charge is **2**; the extra slot was the TU's `_fltused`.
Its carried lesson is that **only a series separates a per-function charge from a
per-TU constant**, and that a lane reporting one cell *looks exactly like* a lane
that measured a rule.

Every row here is `n = 1, 2, 3` copies of the probe in one TU, fitted
`L(n) = k·n + c` **on the two end points, so the middle point is a residual and
not an input**.

```text
  leaf-for       2  4  6      k = 2   c = 0   resid 0,0,0
  leaf-forever   3  6  9      k = 3   c = 0   resid 0,0,0
  leaf-for2      4  8 12      k = 4   c = 0   resid 0,0,0
```

**Every row came back `c = 0`, residual 0.** So on this table, reading one cell
*would* have been right — and that is a **result of having varied `n`**, not a
reason not to have. The two facts a single cell could not have separated are
stated because they were checked: no slot in this table belongs to the TU, and
no row is non-linear in `n` up to 3.

**16 of 18 series are discriminating** (`L(n)` varies with `n`). The two that are
not — `leaf-none` and `leaf-if`, both published `+0` — are **structurally unable
to disagree** on the per-TU/per-function split, and the instrument prints them in
a separate loud line rather than counting them as passes. That is the defect a
lane here once shipped twice: two "0 disagree" results no cell could have
disagreed with.

### 2.2 The bridge cell, which is why any of this is evidence

`?HashString` is the **one** leaf loop in this repo whose charge the **oracle**
has settled: `w-fenceb` §3.3 measured **2**, installed it, and turned three
mutants — including the published `3` — into live `mismatch` against real
`c2.dll` with a separating control green under all of them.

It was registered as prereg **B1** at P = 0.90 with a falsifier attached: *if
this lane's instrument cannot reproduce that 2, every row it prints is suspect
and the lane says `FAILED` rather than publish a table.*

```text
  hashstring   counter 2546   base $M2562   real $M2564   LEAD 2
```

**It reproduces `w-fenceb` §3.1's arithmetic digit for digit** — 2546 / 2562 /
2564 — from a source this lane generated rather than from that lane's file. **F3
did not fire.** And it does so **twice more independently**: through §4.2.1's own
`a0·P·a1·a2` instrument (`stride 3`, surcharge **+2**), and as `w-bdnz`'s
`lab_goto` cell at the workload's flags (**2**).

### 2.3 Two seed-free instruments, and they agree

| | LEAD | STRIDE |
|---|---|---|
| TU shape | `[P × n, z9]`, z9 framed and last | `a0 · P · a1 · a2`, anchors framed |
| readout | `real $M(z9) − (counter + 9 + 3·segs + nleaf)` | `first(a1) − first(a0) − base`, `base` measured in-obj |
| seed | each TU's **own** `.gl` counter subtracted | a difference **inside one obj** |
| used by | `w-fenceb`, `w-slots` (`#3091`, `#3148`) | §4.2.1 itself |

`plan_labels` charges a leaf `label_lead + 1`, so `stride = lead + 1` and
**§4.2.1's `surcharge` column and a `label_lead` are the same quantity**. That
identity is what makes the two comparable at all, and it is also the identity
`counted_accum_loop`'s comment got wrong (§5).

**`instrument disagreements: 0 of 18` at `/O1` and `0 of 17` readable at `/Ox`.**
PREREG **F4** did not fire.

**Do the cells share a `.gl` counter?** Registered per row, because `#3148` is
what happens when nobody asks. **They do not, and they do not need to**: the
`n = 1, 2, 3` cells of a row are three different source texts with three
different counters, and each one's counter is subtracted **inside its own TU**
before any difference is taken. The `a0·P·a1·a2` cells never leave one obj at
all. **No number in this lane is a difference across two counters** — which is
exactly the property `LABEL_LEAD.md` and `work/w-bdnz/LABEL_LEAD.md` lack.

### 2.4 The mutants, and an honest statement of their strength

For every row, the neighbouring charges `k−1`, `k+1` and `0` are checked against
the reference obj's **own symbol-table bytes**: a charge `k′` predicts
`$M(base + k′)` for the framed function, and the obj says what it says.

```text
  leaf-ptrwalk    base 2560  real 2563  |  2:MISMATCH  *3:match  4:MISMATCH  0:MISMATCH | green (0 labels)
  hashstring      base 2562  real 2564  |  1:MISMATCH  *2:match  3:MISMATCH  0:MISMATCH | green (0 labels)
  leaf-for2       base 2564  real 2568  |  3:MISMATCH  *4:match  5:MISMATCH  0:MISMATCH | green (0 labels)

  50 red · 0 green · 18 of 18 separating controls green at EXACTLY 0 label symbols
```

**The separating control is the same body in a leaf-only TU.** A leaf-only TU
mints no label at all, so the counter never reaches its obj (board **#742**) and
**no charge can break it** — a mutant reddening both columns would be measuring
something else. It is positive on content: the control prints its label-symbol
count, and all 18 print **0**, which reproduces half of §4.2.3 as a by-product.

**This is a WEAKER construction than `w-fenceb`'s and `w-slots`', and the lane
says so rather than dressing it up.** Those lanes routed a wrong charge through
the shipped emitter and read `c2rs gap` against real `c2.dll`. That is available
only for a class the port **emits**, and of these 18 rows exactly one is — the
bridge cell, whose port-level battery `w-fenceb` already ran and whose reference
`$M2564` this lane independently reproduces. For the other 17 the judge is the
same six bytes one layer earlier. **Re-running the one available port-level
battery would have re-proved `#3126` and nothing else**, and it would have
required a `crates/` mutation this lane's own prereg forbids (**F5**).

---

## 3. §4.2.1, row by row — published / measured / agrees

`work/w-labeltable/rows_o1.txt`, `/O1 /GS- /c`, both instruments.

| row | published | **measured (LEAD `k`)** | STRIDE−1 | series `L(1),L(2),L(3)` | agrees |
|---|---:|---:|---:|---|---|
| `leaf-none` | +0 | **+0** | +0 | 0, 0, 0 | **yes** *(non-discriminating)* |
| `leaf-if` | +0 | **+0** | +0 | 0, 0, 0 | **yes** *(non-discriminating)* |
| `leaf-while` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-dowhile` | +1 | **+1** | +1 | 1, 2, 3 | **yes** |
| `leaf-for` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-for-k` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-for-stride` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-for-down` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-for-cont` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-for-live` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-idxload` | +2 | **+2** | +2 | 2, 4, 6 | **yes** |
| `leaf-forever` | +3 | **+3** | +3 | 3, 6, 9 | **yes** |
| `leaf-for-break` | +3 | **+3** | +3 | 3, 6, 9 | **yes** |
| **`leaf-ptrwalk`** | **+3** | **+3** | +3 | 3, 6, 9 | **yes — §4** |
| `leaf-for2` | +4 | **+4** | +4 | 4, 8, 12 | **yes** |
| `leaf-fornest` | +4 | **+4** | +4 | 4, 8, 12 | **yes** |
| `leaf-goto-back` | +1 | **+1** | +1 | 1, 2, 3 | **yes** |
| *`hashstring`* (bridge, not a §4.2.1 row) | *2, shipped* | **2** | 2 | 2, 4, 6 | **yes** |

**Where a published number and the obj disagree the obj wins, and there is no
such row.** `rows where measured != published: 0`.

### 3.1 §4's six framed rows — 6 of 6, and the re-derivation trap fires on demand

§4.2.1's rightmost column pairs six of its rows against §4's framed table. Those
six, through §4's own instrument (`work/w-labeltable/framed_o1.txt`):

```text
  row              pub stride  minted   NAIVE  corrected | verdict
  cf-if              0      5       5       0          0 | AGREES
  cf-while           2      9       7       4          2 | AGREES
  cf-dowhile         1      8       7       3          1 | AGREES
  cf-for             2      9       7       4          2 | AGREES
  cf-fornest         4     11       7       6          4 | AGREES
  cf-goto-back       1      8       7       3          1 | AGREES
```

**The `NAIVE` column reads `for` +4 and nested +6 — the exact two numbers §4's
own warning box says a fresh worktree gets and believes.** Every loop-bearing row
is `minted 7` against the `if` row's 5, so the naive reading is uniformly `+2`
high and `stride − minted` recovers §4's published value on all six. The hazard
is **a constant +2 on exactly the rows that spill**, and it is now something an
instrument prints rather than something a reader has to remember.

---

## 4. The contested row, settled — and it settles the other way

`#3091` was raised by `w-backedge`, settled by `w-fenceb` **against the oracle**
at two sites, and `w-fenceb` §6 item 3 **deliberately left the table row alone**:

> *"the table's row is `Sort.cpp`'s shape measured in a leaf-only TU, and whether
> **it** is wrong or merely a different shape is a measurement nobody has made.
> Left as a row rather than edited on inference, which is the mistake that
> produced the two corrected sites."*

**That was the right call, and the measurement now exists. It is a different
shape.** Both bodies, same run, same instruments:

```text
  leaf-ptrwalk   int P(const char* s){ … for (const char* p=s; *p; p++) r=r+*p; }    3
  ?HashString    … for (unsigned char *u=(unsigned char*)str; *u!=0; u++) …           2
```

### 4.1 The ladder — one token, one slot

Five cells walking one body into the other, `work/w-labeltable/ladder_o1.txt`,
each with its own three-point series:

```text
  pw0-signed    const char*                          3 | 6 |  9      k = 3
  pw1-unsigned  const unsigned char*                 2 | 4 |  6      k = 2   <== THE STEP
  pw2-ne0       …and `*p != 0`                       2 | 4 |  6      k = 2
  pw3-mul       …and `r = *p + r*0x7F`               2 | 4 |  6      k = 2
  pw4-mod       …and `% i`   (= ?HashString)         2 | 4 |  6      k = 2
```

**The step is at cell 1 and the other three are flat.** The whole 3-vs-2
difference is the **signedness of the loaded byte**; the multiply, the modulo,
the explicit `!= 0` and the second formal are worth **zero** between them.

The two objs are the same **40 bytes** long and differ in block structure:

```text
  pw0 (signed)      lbz · mr · mr · li · b .+12 · lbzu · add · extsb. · bf -12 · blr
  pw1 (unsigned)    lbz · mr · li · cmplwi · bclr 12,2 · lbzu · add · mr.    · bf -12 · blr
```

The signed form guards the loop with an unconditional `b` **into** the body; the
unsigned form returns early through `bclr`. **That coincidence with `w-osfinfo`'s
*"one slot per unconditional intra-section `b`"* is recorded and is NOT filed as
a rule** — `w-vsnprnc` refuted exactly that rule on two spellings emitting the
identical thirty-eight words and charging 1 and 0, and `w-xlr` refuted a
one-witness discriminator twice. It is one witness. It is named so a later lane
registers it rather than re-derives it.

### 4.2 What this does to `#3091`

`#3091` is **not** widened by this lane; it is **bounded** by it. Its two
corrected sites were right to correct — a lead of 3 for `ptr_walk_loop` is a live
wrong obj, graded. **Its third site was not a third instance.** The row is a
correct measurement of a body the class does not contain, and the defect was
never in the table: it was in a fixture header and a doc comment **quoting a row
about `const char*` at a class that walks `unsigned char*`**.

---

## 5. The third overstated number, and it is in shipped code

`work/w-bdnz/LABEL_LEAD.md` measures a lead as *"two TUs differ in exactly one
function body"* — **the construction `#3148` refuted**. Re-differenced seed-free
at that lane's own `/O1 /Oi /EHsc /GR` (`work/w-labeltable/bdnz_o1work.txt`):

```text
  cell           counter    real $M   published   SEED-FREE   gap vs lab_ctl
  lab_ctl           2540       2556          —          0      +0
  lab_forever       2542       2558        +2          0      +2
  lab_loop          2545       2563        +7          2      +5     THIS CLASS
  lab_while         2545       2563        +7          2      +5
  lab_dowhile       2545       2562        +6          1      +5
  lab_goto          2546       2564        +8          2      +6     ?HashString
  lab_op            2545       2563        +7          2      +5
  lab_uns           2545       2563        +7          2      +5
```

**All eight `$M` values reproduce to the digit, and every published lead is the
seed-free lead plus that cell's own counter gap.** `lab_goto` reads **2**, which
is the oracle-settled number again, from a third construction.

**And `lab_forever` — the separating control the file nets its locals out with —
reads 0.** Two `int` locals cost the counter nothing, so *"net of locals it is +5
against +1"* is wrong in both terms.

### 5.1 The sentence in `crates/`

`IlFunction::label_slots`' `counted_accum_loop` arm, shipped, reads:

> *"§4.2.1's `for` row records `+2` against `leaf-none = 1` — a lead of `+1`,
> where the obj says **+7**."*

**Both halves are wrong and in opposite directions.** A §4.2.1 surcharge **is** a
lead: `plan_labels` charges a leaf `label_lead + 1`, so `stride 3` is `lead 2`,
not `lead 1` — the sentence subtracts the base twice. And the obj says **2**, not
7. Corrected, it reads: ***§4.2.1's `for` row records +2, and the obj says +2.
They agree.***

**NOT edited here.** It is a `crates/` change, this is a docs lane whose prereg
registers a zero byte delta and an unchanged `graded tree` (**F5**), and the arm
belongs to a peer in flight. The replacement text is above; it is filed for the
coordinator.

### 5.2 What this does and does not do to that arm's verdict

**It does not move it, and the lane says so before anyone asks.** The `None` on
`counted_accum_loop` rests on **reading 2**, not reading 1:

> *"THE CHARGE IS MODE-DEPENDENT … and this class accepts BOTH modes, so it would
> meet the wrong one immediately."*

That reason is **untouched, and re-measured here** rather than inherited: the
same body reads **2** at `/O1` and **3** at `/Ox` (`bdnz_oxwork.txt`). The
**step** that lane measured, `+1`, is exactly right; only its base was inflated
by the counter gap. `w-slots` §5 independently confirmed the both-modes premise
in the reader (`counted_accum_loop.rs:233-235`, `O1 | Ox`).

So the correction changes **how dear the fence looks** and not **whether it
holds**. §7 prices that distinction, because it is the whole of what a corrected
number buys here.

---

## 6. §4.2.1a — the `/Ox` column, and one row that is CONFOUNDED

`work/w-labeltable/rows_ox.txt`. Both instruments agree on **17 of 17 readable
rows**.

**§4's `/Ox` row — `for` +8, nested +10 — reproduces to the digit on LEAVES**,
where §4 measured framed bodies. The `/Ox` surcharge is frame-class-independent
too.

**The mode is not a constant offset**: `leaf-if`, `leaf-dowhile`, `leaf-forever`
and `leaf-goto-back` do not move at all, and `leaf-for-k` moves **+11**. A
`label_slots` learning a mode word would need the whole table twice, not a delta
— which is `#1983`'s reason, now with a magnitude on it.

**`leaf-idxload` is CONFOUNDED at `/Ox` and is marked rather than quoted.** Its
probe **stops being a leaf** there; the TU acquires a second and third framed
group as `n` grows, so the triple the instrument reads is not the control's. This
is `w-slots` §3's `/Ox` shape-C confound on a different body, and it was caught
the same way — by **counting framed groups per cell** rather than trusting the
readout. The instrument refuses the row instead of printing a number for it.

**Consequently §4.2.3's channel claim is `/O1`-scoped**: *"a leaf-only TU mints
zero labels"* is **18 of 18** at `/O1` and **fails on `leaf-idxload` at `/Ox`**,
for the same reason. The claim is about leaves and the mode decides which bodies
are leaves.

The `/Ox` cells need one calibration, and it is measured rather than assumed:
`base = counter + 9 + 3·segs + nleaf` is a `/Gy` fact and `/Ox` is not `/Gy`, so
every function is over-charged there by a constant **3**. It lands in the
**slope**, **both** zero-controls read the same one, and the `a0·P·a1·a2`
instrument — which measures its own base in-obj — needs no calibration at all and
agrees with the calibrated leads on every readable row. That agreement is what
makes it a calibration rather than a fudge.

---

## 7. What gets cheaper, and by how much — **STATED AND NOT TAKEN**

The prereg registered this section as claims about a **fence's price**, each
followed by a full stop. `#3147`'s standard for a lift is **a closed recognizer
plus a series**, and a recognizer is a `crates/` change this lane is forbidden.

| fence | what the correction changes | what it does NOT change |
|---|---|---|
| **`counted_accum_loop`'s `None`** (`#746` fence B, third arm) | its published charge falls from **+7 to 2** — the *same* 2 that `ptr_walk_loop` and `float_walk_loop` now ship. So the arm no longer looks like an outlier three times the size of its neighbours; it looks like the same charge with a mode step | **the verdict.** Its `None` rests on mode-dependence on a reader admitting **both** modes, measured, and the mode step (`2 → 3`) is confirmed here. A lift still needs the reader narrowed or `label_slots` given a mode word, and `wb-label` §7.6 forbids the latter |
| **`ptr_walk_chain_loop`, `pool_ctor_chain`** (fence B arms B and C) | nothing. `w-slots` §4 priced both at **ZERO tracked fixtures and ZERO workload TUs** against a probe whose positive control fires, and the binding objection is **property 3** — nothing in the tree could grade a charge for either | unchanged, and this lane adds a reason not to take them: a number installed without a gradeable cell is exactly the condition that produced every artifact in §5 |
| **§4.2.1 as a source** | it is now safe to quote **for the body it names**, with 17 rows re-graded and a mutant battery behind each | it is **not** safe to quote *at a class*. That is the error `#3091` corrected twice and §4 warns about, and this lane found a third instance of it in shipped code |

**A third `for` class at 2.** `w-slots` §8 item 4 registered that `ptr_walk_loop`
and `float_walk_loop` both charging **2** is *"one witness short of a rule"*.
This lane measures a **third**: `counted_accum_loop`'s own body, seed-free, at
**2**. Plus `pw1`–`pw4` and `?HashString`, all `for`, all **2**; and `leaf-for`,
`leaf-for-k`, `leaf-for-stride`, `leaf-for-down`, `leaf-for-cont`,
`leaf-for-live`, `leaf-idxload` — **seven more `for` rows at 2** in the published
table.

**It is still not filed as a rule, and the reason is in this lane's own data.**
`pw0-signed` is a `for` and charges **3**; `leaf-forever` and `leaf-for-break`
are `for`s and charge **3**; `leaf-for2` and `leaf-fornest` charge **4**. A
kind-keyed charge is refuted by rows in the same table. What is true is narrower
and is all that is claimed: **a single, un-nested, break-free, continue-free
`for` over a non-negative-byte or integer induction variable charges 2 at `/O1`,
on twelve witnesses** — and `#3127` already says a rule fitted to a grid dies on
its hold-out, so this is registered as **the prereg a fourth lane would open**,
scored nowhere.

**Not dispatched off any ranking, and none is built.** `w-loo` measured that five
of six published rankings carry no information and that a ladder never scores
what it starts with. The rows above are named with their prices, in the order the
evidence arrived, and no lane is dispatched from them.

---

## 8. Prereg scored

| # | claim | P | outcome |
|---|---|---:|---|
| **17 per-row claims** | published §4.2.1 number holds | 0.55–0.97 | **17 HITS, 0 MISSES.** The aggregate was registered at *point estimate 1, ceiling 4 of 17 disagreeing*; realized **0** |
| **B1** | the instrument reproduces `?HashString`'s oracle-graded **2** | 0.90 | **HIT** — 2546 / 2562 / 2564, `w-fenceb` §3.1 digit for digit, and twice more independently |
| **B2** | `?HashString` through §4.2.1's own instrument reads surcharge **+2**, so the table carries no systematic offset | 0.70 | **HIT** — `stride 3` |
| **B3** | if B2 missed, the whole column is one high as a rule | 0.25 | **DID NOT ARISE** — B2 hit |
| **B4** | `leaf-ptrwalk` and `?HashString` are different shapes | — | **CONFIRMED, and the separating token is measured** (§4.1) — registered as stated-not-scored, and it became the lane's second result |
| **S1** | every row fits `L(n) = k·n`, `c = 0` | 0.70 | **HIT** — 17 of 17 |
| **S2** | at least one row has `c ≠ 0` | 0.30 | **MISS** — none does. Registered as the complement of S1 and it is the honest miss |
| **S3** | every row's `k` equals its `n = 1` lead | 0.65 | **HIT** — reading one cell would have been right *here* |
| **S4** | residual 0 on every row | 0.75 | **HIT** |
| **W1** | `w-bdnz`'s `lab_goto` re-reads as **2**, not 8 | 0.85 | **HIT** |
| **W2** | all 8 rows move, by exactly each cell's counter gap | 0.75 | **HIT** — 7 of 7 (`lab_ctl` is the reference), gap exact on every row |
| **W3** | `lab_loop`'s `+7` re-reads at **≤ 4** | 0.80 | **HIT** — **2** |
| **W4** | the table-vs-obj comparison was never like-for-like | 0.70 | **HIT, and larger than registered** — it is wrong in *both* halves, and the second half is a `stride`/`lead` unit error, not a body mismatch |
| **F1** | anchor control fails on some row | — | **did not fire** — control 5 on all 18 rows at `/O1`, 4 on all at `/Ox` |
| **F2** | the zero-controls do not read 0 | — | **did not fire** — both read exactly 0 at `/O1` |
| **F3** | the instrument cannot reproduce the oracle-graded 2 → **`FAILED`** | — | **did not fire** |
| **F4** | the two instruments disagree non-constantly | — | **did not fire** — 0 of 18 at `/O1`, 0 of 17 readable at `/Ox` |
| **F5** | any byte under `crates/` changes | — | **did not fire** — zero files under `crates/` opened |
| **F6** | fewer than 10 of 17 series discriminating → the grid is vacuous | — | **did not fire** — **16 of 18** |

**One miss on twenty-two, and it is `S2`, the complement of a claim that hit.**
The registered aggregate — *"point estimate 1, ceiling 4 rows disagreeing"* — was
**over** by 1 and by 4. The lane's prior was that a seed-free table would mostly
reproduce, and the direction of the error was that it hedged *toward* the brief's
premise rather than against it.

### The registered outcome numbers

| quantity | **population** | registered | realized |
|---|---|---|---|
| `mismatch` | every gate lane, sweep, cross, 878-TU scan | **0** | **0** |
| fixture-gate `match` | **381 fixtures × 18 mode lanes** | **+0** | **+0** |
| `c2rs perf` `Match` | **381 fixtures at the `/Ox` default** | **+0** | **+0** |
| **878-TU workload `match`** | **878 dc3 TUs** | **25 → 25** | **25 → 25** |
| `codegen-gap` / `vocab-gap` / `capture-fail` / `frontier` | 878-TU scan | **0 / 845 / 8 / 2** | **as registered** |
| `gap-metric` keys · verdict lines | — | **372 · 878** | **as registered** |
| `fnbyte-exact` | 878-TU scan | **35734** | **35734** |
| workspace tests | — | **1610 / 42, no test added** | **1610 / 42** |
| `graded tree` | `crates fixtures scripts` | **`04e3500f07b7`, 730 files**, both ends | **unchanged, both ends** |
| census · new fixtures | — | **+0 · 0** | **+0 · 0** |

**Every number was registered before the first `cl.exe` with its denominator
named in the same breath** (`#3125`). This lane publishes **no** `match`
movement of any kind, and the three figures called `match` are listed separately
so that a summary cannot collapse them.

---

## 9. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1610 passed, 0 failed, 42 targets** — identical to master's, because no test was added and no `crates/` file was opened. The target count is quoted because a dropped target means an earlier target failed |
| `scripts/gate.sh --jobs 4 --require-graded` | see `work/w-labeltable/gate_tip.txt` |
| `graded tree`, **both ends** | **`04e3500f07b7`, 730 files** under `crates fixtures scripts` — the coordinator's stated master value, unchanged at both ends and unchanged in file count. This lane created **no** file under those trees and modified none |
| **878-TU workload `match`, both ends** | **25 → 25**, `mismatch` 0, `codegen-gap` 0, `vocab-gap` 845, `capture-fail` 8, `frontier` 2, `fnbyte-exact` 35734 |
| `scripts/debug_lane.sh` | `work/w-labeltable/debug_lane.txt` |
| `scripts/board_audit.sh` | all-zero |
| `crates/c2-harness/tests/rung_registry.rs` | **2 passed, 0 failed** |

**A docs lane still runs the whole gate**, because *"no `crates/` file was
opened"* is a claim about a tree and the identity diff is what proves it. The
`graded tree` hash at both ends is the sharpest evidence in this rung: it is the
same 12 hex digits and the same 730 files, so **F5** is measured and not asserted.

---

## 10. Found and not taken

Ranked, with the frame axis applied.

1. **The wrong sentence is in `crates/c2-il/src/func/mod.rs` and this lane could
   not edit it.** `label_slots`' `counted_accum_loop` arm carries *"a lead of
   `+1`, where the obj says `+7`"*; both halves are wrong and §5.1 gives the
   replacement text. **It is one comment edit and it changes no byte of any
   obj** — but it changes the `graded tree` hash this lane registered as
   unchanged, and the arm belongs to peer `w-counted`. **The right taker is that
   peer, in the same commit as whatever it does to the reader.** Left as a filed
   correction rather than done on the side, because a docs lane silently editing
   a peer's shipped comment is how two lanes come to disagree about one number.

2. **`work/w-bdnz/LABEL_LEAD.md` is corrected by a revision box, and its
   INSTRUMENT is not.** `work/w-bdnz/label.sh` still differences across TUs, and
   it is the script a future lane will reach for on that class. It is ~20 lines
   and `work/w-labeltable/table.py --bdnz` already does the job. **Not rewritten
   here**, because rewriting another lane's instrument to produce different
   numbers than its own write-up is exactly the confusion the revision-box
   convention exists to avoid. A lane touching that class should retire it.

3. **Twelve `for` witnesses at 2, and it is still not a rule** (§7). A fourth
   lane wanting a kind-keyed charge owns the prereg stated there, and it owns
   `#3127`'s hold-out standard. What is new and cheap is the **counter-example
   set inside this lane's own table**: `pw0-signed` at 3, `leaf-forever` and
   `leaf-for-break` at 3, `leaf-for2` and `leaf-fornest` at 4. Any rule is
   graded against those before it is graded against anything else.

4. **The signed/unsigned step is one witness for a `b`-word discriminator and
   `w-vsnprnc` already refuted that rule.** §4.1. Named so it is registered
   rather than re-derived. The cheap next cell is the *same* ladder at a second
   loop kind — if `while` also steps by 1 on signedness, it is a term; if it does
   not, it is `for`-specific and the ladder has told a fourth lane where not to
   look. **One command, not taken here** because it is a new grid and not this
   lane's question.

5. **§4.2.3's `/O1` scope is now known and the section does not say so.** §6.
   The `leaf-idxload` confound means *"34 of 34 leaf-only TUs mint zero labels"*
   is a statement about `/O1` leaves. §4.2.1a records it; §4.2.3's own text is
   left as written, per the never-rewrite-around-a-wrong-number convention.

6. **`#3098` is still OPEN and this lane did not touch it.** Four env-gated sink
   instruments live only in `expr.rs` doc comments and `IL_DECODE_REACH.md` §12
   now names five; a docs section did not fix the board-topic discoverability
   half. **`table.py` is a sixth undiscoverable instrument by the same
   mechanism** — it is reachable from `LABEL_COUNTER.md` §5 and from this rung,
   and from no board topic. Named rather than fixed, because the fix is the one
   `#3098` is waiting for and minting a second half-fix is what produced the
   first.
