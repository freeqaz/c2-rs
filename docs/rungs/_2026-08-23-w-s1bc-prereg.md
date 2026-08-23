# PREREG — `w-s1bc`, the rest of Phase 0 slice **S1**: the `Selected` collapse (S1b) and the native op stream (S1c)

**Frozen 2026-08-23, as the first commit on branch `wt-w-s1bc`, before any
measurement was taken on this tree.** Base `7aa91ff3d`.

Spec: `docs/rungs/2026-08-22-w-s1.md` §6 (the lane that shipped S1a priced S1b
and S1c there; that section is this lane's specification **and** its price) and
its §7 (the stop condition, inherited verbatim). Funded by
`docs/DECISIONS_2026-08-22.md` **decision 7**; the stop condition's re-anchoring
is **decision 5**. Row: `ROADMAP_SLICING_2026-08-21.md` §5 row **S1** with its
AMENDED block, under §6's seven standing rules.

Grading conditions: board **#3346** (required-zero byte delta *on the live
dispatcher* — and there are **two**, see §2.1), board **#3336** (a construct
rung must name an axis it can fail on even when every byte is identical),
board **#3048** (a gate whose tree moves under it is evidence about neither
tree), `docs/rungs/README.md` § Lane kinds.

Lane kind: **construct rung**. `Fixtures: none`. `Census: +0`. Expected
`Outcome: built`.

---

## 0. What the tree already says, before this lane measures anything

Recorded so that a later disagreement is visible as a disagreement rather than
silently overwritten. **None of these is trusted; §3 re-measures every one of
them on this tree at this base.**

| quantity | filed value | source |
|---|---|---|
| `fnbyte-exact` | **35,891** | `docs/rungs/2026-08-22-w-s1.md` §7, at base `9b9530791`, **workload stamp `e5aef017d456`** |
| `fnbyte-exact` | **35,894** | `ROADMAP_SLICING_2026-08-21.md` §5 — **known stale**, `DECISIONS` decision 5 |
| `fnbyte-exact` | **35,886** | `docs/STATUS.md` generated block, tree `977827d78` |
| `fnbyte-differs` | 1,963 | w-s1 §7, same stamp |
| `fnbyte-denominator` | 162,161 | w-s1 §7, same stamp |
| workload `match` / `mismatch` | 26 / 0 | w-s1 §5 |
| `gap-metric` key count | 485 | w-s1 §2.1 |
| gate lanes | 18 | `scripts/lanes.txt` |
| gate fixture-verdicts | 6,948 | w-s1 §5 |
| `#[test]` count at this base | **1,840** | `git grep -c '#\[test\]' -- 'crates/*.rs'`, summed, measured here before any edit |

### 0.1 **THE WORKLOAD TREE HAS ALREADY MOVED, and this lane found it before measuring**

`w-s1` measured at workload stamp **`e5aef017d456`**. The workload tree
(`../dc3-decomp`, a live repo other agents merge into) reads
**`6f5fa9ccfcd7`** at the moment this prereg is frozen — `e5aef017d` is an
ancestor, eleven-plus commits back.

**So this lane's base `fnbyte-exact` is expected to differ from 35,891, and a
difference is NOT the stop condition.** The stop condition compares **base
against tip at one stamp**, never a tip against a filed figure. This is exactly
the §7.1 hazard one workload-generation on, registered *before* the number is
known so that it cannot be retrofitted:

* This lane re-derives its own base triple on its own tree with the stamp
  printed beside it, and scores **only** against that.
* **The stamp is recorded on both ends and compared.** If the workload tree
  moves between the base scan and the tip scan, the comparison is void by
  #3048's reasoning one level out — a diff between two corpora is not
  zero-or-nonzero, it is meaningless — and the scan is re-run, with the void
  run recorded rather than deleted.
* If the base reads something other than 35,891, **both numbers are reported
  and neither is reconciled silently.**

---

## 1. What is being built

`select_function` returns `Selected`. Three of its variants —
`Plain(Vec<u8>)`, `Tail(Vec<u8>)`, `MemcpyTail(Vec<u8>)` — differ **only in
what the caller must append after the body**:

| variant | what the caller appends |
|---|---|
| `Plain` | nothing |
| `Tail` | `b <f.tail_call()>` at `text.len()`, with a `REL24` there |
| `MemcpyTail` | the same branch word, to a **minted** `memcpy` (no `.gl` record, so `f.tail_call()` is `None`) |

### 1.1 S1b — the collapse

One live variant carrying a body plus a terminator:

```rust
pub enum Terminator { None, TailCall, MemcpyCall }
Selected::Body { text: Vec<u8>, term: Terminator }
```

with the three retired variants re-added as **`#[cfg(test)]` views**, in S1a's
`mod incumbent` pattern: the old three-way classification written out **as it
stands today**, not re-derived from the new field, so the cross-check compares
two independent derivations rather than the collapse against itself.

### 1.2 S1c — the native op stream

`encode.rs`'s 85 encoders each already **are** a `MachineOp` composed and
rendered (`MachineOp::new(op::ADD).s(rd).d0(ra).d1(rb).word()`). S1c adds a
`mop_*` sibling per encoder returning the `MachineOp` **unrendered**, redefines
`encode_X` as `mop_X(..).word()` — required-zero by construction, the same
composition — and converts producers to build a `Vec<MachineOp>` that is
rendered once at the boundary.

### 1.3 **S1c IS TWO SEPARABLE RUNGS AND THIS LANE REGISTERS WHICH ONE IT IS DOING**

`w-s1` §6 surveyed it: of the 18 `Selected::Plain` producers, **16 build bytes
by direct `encode_*` calls with no intermediate representation at all**; only
`ptr_walk_loop` and `ptr_walk_chain_loop` go through `block_ir`. So:

* **(i) "wire `Plain` through an opcode-carrying op stream"** — the `mop_*`
  layer plus per-producer conversion. **This lane attempts (i).**
* **(ii) "wire `Plain` through `block_ir`"** — a different job on a different
  IR. **NOT attempted. Priced at report.**

Registered because a rung that landed (i) and let its headline read as "S1c" is
the compound headline `docs/rungs/README.md` forbids.

**And board #3365 is wrong for this purpose, per w-s1 §6**: its *"13 of the 35
shape arms already share `block_ir`"* is right about the 13, but **only 2 are on
a `Selected::Plain` arm**. This lane does not price off that row.

### 1.4 Scope split, registered before measuring

| part | this session |
|---|---|
| **S1b** — the collapse, its `#[cfg(test)]` views, all consumer sites | **ATTEMPTED — the primary deliverable, landed as its own commit with its own gate pair** |
| **S1c (i)** — the `mop_*` layer + producers converted to `Vec<MachineOp>` | **ATTEMPTED if S1b lands green.** The producer set actually converted is **named at report with its denominator**, never as "S1c" |
| **S1c (ii)** — `Plain` through `block_ir` | **NOT attempted. Priced at report.** |

**If only S1b lands, the rung says so in those words and prices S1c.** S1c is
"the bulk of S1's 2–4 weeks" (`w-s1` §6) and this is one session.

### 1.5 The decision surface — and an honest null

`GOAL_DECISION_2026-08-21.md` § AMENDED and `ROADMAP_SLICING` §6 rule 7 require
a general layer to ship its arbitrary choices as named, settable parameters.

**Registered in advance: S1b has no arbitrary choice to expose, and this lane
will say so rather than invent a parameter.** The terminator is not a tunable —
it is a total function of the `BodyShape` arm, and there is no configuration in
which a different value reproduces c2. What S1b does ship is the **naming**: a
public `Terminator` enum whose three values are the three post-body obligations,
where today they are three byte-vector variants that a reader must diff two
dispatchers to tell apart. A fake parameter would be worse than a stated null
(#3356's discipline, one level over).

S1c inherits S1a's real surface (`EncodeParams`) unchanged and adds none.

---

## 2. The graded-by criteria, registered with their thresholds

### 2.1 Criterion A — required-zero byte delta on **both** LIVE dispatchers (#3346)

**Threshold: exactly 0 lines of difference. Any non-zero delta = the edit is
reverted or the lane FAILS. This lane never ships a non-zero byte delta.**

#3346 says "the live dispatcher". **There are two, and the packed one is not a
superset of the COMDAT one** — the correction `w-s1`'s prereg filed and this
lane inherits:

* `crates/c2-core/src/comdat.rs::body_of` — the `/Gy` COMDAT path;
* `crates/c2-core/src/lib.rs::PortC2::build` — the packed `.text` path, which
  refuses `CtorForwardCall`, `FpStoreDiamond` and `MemcpyTail` outright.

Both must show zero delta. Unlike S1a — which sat one layer *below* `Selected`
and so covered both by construction — **S1b edits both dispatchers directly**,
so this criterion is load-bearing here in a way it was not there.

Measured as a **line-for-line identity diff of the per-lane gate counts**
(board #290's pattern):

1. `scripts/gate.sh --jobs 4` at base and at each tip; the per-lane count table
   diffed with `diff`. **Registered expectation: 0 lines.**
2. `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast`,
   recording **test count AND target count**.
3. The 878-TU workload scan: every `gap-metric` key, unmoved, at one stamp.

**`mismatch` is an alarm outranking all work.** Any run reading `mismatch > 0`
stops this lane and is reported, whatever else is in flight.

**Sequencing, registered because S1a voided a run on it (#3048).** `crates/` is
not edited while a gate is in flight. Gates run sequentially on committed
trees. `docs/` edits during a gate are permitted — the detector hashes
`crates fixtures scripts` — and this prereg is committed before the base gate
starts.

### 2.2 The toolchain-live assertion — a POSITIVE fact, never a duration cut

`w-read-r5`'s form, and `w-s1` §5.0's correction to its own criterion:
`crates/c2-harness/tests/require_toolchain.rs` makes a toolchain-less run under
`C2RS_REQUIRE_TOOLCHAIN=1` **fail** rather than skip, so **exit 0 under that
flag IS the assertion**.

**Registered against two forms this lane will not use**: a fixed duration cut
(`census_gate` read 70.62 s under load and 55.96 s quiet — a ≥60 s cut fails
the *better* run), and "0 SKIP lines" (vacuous, #3341). The gate's own graded
counts — fixture-verdicts, sweep cases, cross cells, all through real `c2.dll`
under wibo — are the corroborating positive facts.

### 2.3 Criterion B — the pre-registered COST criterion (#3336)

**The axis is port throughput**, `ir0`'s protocol reused verbatim — the same
protocol S1a used, so the two figures compose:

* Arms: `base`, `tip`, and **`nulldup`, a byte-identical `cp` of `base` verified
  with `cmp`**.
* **8 rounds**, arm order rotated every round.
* Population: the fixtures the port `Match`es. **Its size is published beside
  every figure**, measured at measurement time.
* **Estimator: per fixture and arm, the MINIMUM port median over the 8
  rounds.** Pairing per fixture. The reference column never enters.

**Registered thresholds:**

* **FLOOR:** the null arm's own reading. **No conclusion smaller than the null
  arm is a conclusion.** S1a re-derived that floor at **±0.15 %** with the null
  splitting the sign 76 of 153; this lane re-derives it again rather than
  assuming it, and publishes the null arm's sign-split.
* **PASS:** `< +5 %` port time per obj against base.
* **REPORTED, not failing:** `+5 %` to `+15 %`, stated in the rung header and
  the board row.
* **DECLINE:** `> +15 %`.

**Registered expectation, stated before measuring, so it can be scored:**

* **S1b costs nothing measurable.** It moves a discriminant from an enum tag to
  a field; no allocation is added or removed and no hot loop changes. Predicted
  **inside the null arm's CI**, i.e. the registered verdict is "below the noise
  floor", *not* "no cost".
* **S1c (i) is the one that can cost.** It puts a `Vec<MachineOp>` — 28 bytes
  per op against 4 — between the producer and the bytes. Predicted **positive
  and > the floor**; whether it clears `< +5 %` is the open question and is what
  makes this criterion capable of failing.

**S1a's criterion fired first at +10.67 % and was paid down to +3.03 % by three
no-byte changes. This lane expects to have to pay its own down too**, and the
three levers S1a found (a linear scan that should be an O(1) index; missing
`#[inline(always)]` blocking constant propagation; instrument-state tests on the
hot path) are the first places it will look.

**S1a's open residual is inherited, not re-smoothed**: `wr1_sym_addr.cpp`
**+114.73 %** and `wadjust_obj_recv.cpp` **+103.65 %** remain at about 2× and
are explained by none of S1a's three changes. This lane **reports those two
fixtures' readings explicitly on both of its arms**, whatever the mean says. A
mean quoted without them is a number with a known hole in it.

### 2.4 Criterion C — the `after0` opcode-agreement ratio (instrument, never a gate)

S1a shipped it and pinned its **shape**, never a floor:
**82 of 85** distinct c2 opcodes agree, the 3 residuals being `blr`, `bctrl`
(form 55's `or 0x2800000`) and `bdnz` (form 1's `or 0x2000000`).

**Registered: this lane predicts 82/85 UNMOVED.** S1b touches no encoder at all.
S1c (i) redefines every `encode_X` as `mop_X(..).word()` — the same composition,
so agreement cannot move. **A move in either direction is a finding against this
lane's own construction** and is reported as such. The denominator (85 distinct
opcodes, **not** #3379's 89 encoder *functions*) is published in the same
sentence as the ratio, standing rule 4.

### 2.5 The bijection — NOT touched, and the rung will say so

`w-ildecode`'s `the_final_tuple_order_reproduces_the_text_words` asserts
equality over **three functions, nine words**, all leaf/frameless/call-free.
`ROADMAP_SLICING` §5's AMENDED block and `w-s1`'s prereg §2.4 both register that
pointing it at the corpus unchanged goes **red on the first framed function**,
because the final expansion switch rewrites the prologue pseudo-op in situ
(`WB_REGALLOC_FINDINGS.md` §4 item 2) — an **instrument defect, not a property
of c2**.

**Registered: this lane does not touch it.** The mechanism that makes it go red
is exactly what peer lane `w-read-r6` is reading this wave. Touching it now
would be pricing against an answer that is in flight. If the rung reports the
bijection at all it reports it as "not touched", never as implied coverage.

---

## 3. The base re-measurement, and the STOP CONDITION

Filed snapshots are not trusted (§0 — three of them already disagree and the
corpus under all three has moved). The first thing this lane runs, **before any
`crates/` edit**, is the base triple on its own tree at `7aa91ff3d`, with the
workload stamp printed beside it.

> ### THE STOP CONDITION — watched for, reported LOUDLY, never patched
>
> **If criterion A's required-zero delta HOLDS but the workload `fnbyte-exact`
> count MOVES AT ALL between this lane's own base and its own tip at one
> workload stamp, the program's pricing basis is VOID and the program should
> stop** (`ROADMAP_SLICING_2026-08-21.md` §5; re-anchored by `DECISIONS`
> decision 5 to *the S1 lane's own base-tree measurement*, never a filed
> snapshot).
>
> The reasoning: a pure re-expression that changes no obj byte cannot change how
> many *functions* are byte-exact — unless the port's byte-exactness is a **fit**
> to the shapes' populations rather than a model of c2, which is the §4
> assumption Phase 0 exists to test.
>
> **Registered response, so it cannot be reinterpreted under pressure:** this
> lane does **not** patch, tune, or explain away such a move. It reports it as
> the primary finding, in those words, with the **per-symbol movement** —
> standing rule 3, never subtracted totals (`w-empty`'s first attempt read
> `+0/−14` where the truth was `+1,373/−0`).
>
> **And it is anchored to a stamp.** A base-vs-tip difference across two
> different workload stamps is not a stop-condition trigger, it is a void
> comparison (§0.1). This lane checks the stamp before it reads the delta.

---

## 4. What would make this lane DECLINE

Registered in advance, because a priced decline is a good outcome and a
retrofitted one is not:

1. **Any non-zero byte delta this lane cannot close.** Never ship one. An
   honest smaller re-expression that holds required-zero beats a big diff with a
   hand-waved delta — the brief's instruction, registered so it cannot be
   reinterpreted after the fact.
2. **Cost > +15 %** on §2.3's protocol against the null arm.
3. **`selected_tag`'s published strings cannot be preserved.** `"plain"`,
   `"tail"`, `"memcpy-tail"` are a **published interface** — `c2rs gap` prints
   them, `docs/FUNCTION_BYTE_MATCH.md` quotes them, and `fnbytes.rs`/`fndiff.rs`
   carry them in their own docs. If the collapse cannot keep producing them
   **byte for byte**, the collapse is declined. Narrowing or renaming a shared
   predicate is how this repo has silently erased peer findings twice, with no
   conflict marker and no red gate.
4. **A shared predicate would have to narrow to serve this lane.** In
   particular `splice.rs`'s `Selected::Tail(setup) if setup.is_empty()` clause
   is a **semantic** distinction (SPLICE-P's `port_words > 1` stratum, 0 of 953)
   riding on a variant this lane is collapsing. If the collapsed form cannot
   express it exactly, the lane declines rather than approximating it.
5. **A peer-lane collision.** Four peer lanes are live and all four write only
   `docs/whitebox/`; this lane is the only one in `crates/`. If this lane finds
   itself needing to edit `docs/whitebox/`, it stops and reports the collision
   rather than writing there.
6. **The stop condition fires.** §3.

---

## 5. WHAT THIS LANE'S CONTROLS ARE STRUCTURALLY INCAPABLE OF CATCHING

Registered because #3379's own lesson is that *a control is only capable of
failing on the population you ran it on*, and because a prereg that lists only
its strengths is an advertisement.

1. **The gate exercises the operand combinations and the `Selected` variants its
   FIXTURES HAPPEN TO CONTAIN.** `Selected` has 20 variants. If the fixture
   corpus never produces, say, a `MemcpyTail` under the packed dispatcher — and
   it cannot, because that arm is a documented `NotImplemented` and `/O1`
   implies `/Gy` — then the identity diff says **nothing whatever** about
   whether this lane rewrote that arm correctly. **The packed `MemcpyTail` arm
   is unreachable-by-construction today and its rewrite is graded by nothing.**
   Named here so the rung cannot later imply it was covered.
2. **A required-zero byte delta is silent about everything that is not a byte**
   (#3336) — which is why §2.3 exists, but §2.3 measures only *time*. Memory,
   binary size, compile time and API ergonomics are all unmeasured.
3. **The workload scan grades 870 of 878 TUs** (`capture-fail 8`) and the port
   `match`es 26. `fnbyte-exact` reaches deeper — but `fnbyte-refused-parse` was
   **113,557 of 162,161** at the last reading, so roughly **70 % of the
   function population is refused before the lowering is ever reached.** An
   identity diff over that population cannot see a defect in a shape the parser
   refuses.
4. **`hatch-red` is `REFUSED` on this tree class** (board #1389 `HATCH-STALE`).
   It refuses identically at both ends so it contributes 0 lines to the identity
   diff — and per board #1406 **neither run establishes what a full run would.**
5. **Two independent derivations that are both wrong the same way agree.** The
   `#[cfg(test)]` incumbent views are transcribed from today's code. If today's
   three-way classification is itself wrong somewhere, the cross-check
   reproduces the error and goes green. It proves *preservation*, never
   *correctness* — and the only thing that proves correctness here is real
   `c2.dll` under wibo, which is criterion A.
6. **The cost estimator's floor is ±0.15 % and it is a floor, not a bound on
   bias.** A change that costs every fixture the same 1 % is indistinguishable
   from a box that got 1 % busier, except that the null arm rides along —
   which is why the null arm is byte-identical and why its sign-split is
   published rather than just its mean.

---

## 6. Test-count discipline

A construct rung touching codegen that adds **zero** tests is a finding against
itself. Registered: `git grep -c '#\[test\]'` is diffed base → tip and the delta
published. Base measured here, before any edit: **1,840**.

**Each structural rule the collapse encodes gets a PORTABLE assertion** — a
toolchain-gated fixture pins nothing in the portable lane, and the portable lane
is the one that runs everywhere. Specifically registered:

* the three published tag strings, asserted against the `#[cfg(test)]`
  incumbent's independent derivation, exhaustively over `Terminator`;
* the terminator→post-body-obligation mapping, asserted at both dispatchers'
  seams where a portable seam exists;
* for S1c (i), a per-encoder cross-check that `mop_X(..).word() ==
  encode_X(..)` over each converted encoder's operand domain — S1a's sweep
  pattern, the **whole** `0..32`, not a sample.

**And an executed-mutation corollary** (`ir0`'s, `README` § Lane kinds): the
criterion must be shown to be *capable* of failing by executed mutations with
distinct signatures, not by argument. At least two are run against the tip and
the tree restored and re-verified green between each.

---

## 7. Bookkeeping registered in advance

* Board rows: **`#3424`–`#3428` only** (reserved for `w-s1bc` in `BOARD.md`'s
  trailing live ledger). Appended in-branch.
* Rung file under `docs/rungs/`, `Outcome:` exactly one of
  `converted | declined | instrument | built | FAILED`. **No deliverable =
  `FAILED`, in that word.** `INDEX.md` regenerated with
  `scripts/gen_rung_index.sh`, never hand-edited.
* Scratch under `work/w-s1bc/`, never `/tmp`.
* No `Co-Authored-By` or any AI/agent trailer. No IL, no `*.obj`, no absolute
  `/home/...` paths. Staged deliberately, never `git add -A`.
* **Do not merge, do not push.** The coordinator merges.
