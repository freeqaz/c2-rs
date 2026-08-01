# DRAFT for `docs/ROADMAP.md` §9.12 — paste verbatim, then delete this file.

Kept out of `ROADMAP.md` on purpose: that file is the recorded add/add conflict
site for concurrent lanes (`docs/rungs/README.md`), the coordinator lands §9.12
serially, and this lane was told not to touch §1–§9.11. Everything below is the
section text.

---

### 9.12 W-LABEL — the pin §9.10 asked for was smaller than §9.10 thought, and the label counter is an ORDINAL rule (2026-08-01)

Lane `w-label`, board **#137** then **#135**. Pre-registration in
`docs/rungs/_2026-08-01-w-label-prereg.md`, committed at `6e3e9d3` before the
first mutation ran and before the first `.cod` was captured.

#### 9.12.1 #137 — `cargo test` pinned WR1's ordering rules in NEITHER lane

§9.10 stated the gap as *portable*: the fixtures are toolchain-gated, so on the
portable lane nothing pins the two rules. **The gap is one column wider than
that.** Three mutations, each a one-site edit implementing the rule WR1 got
wrong on its first differential, run against the **base tree before any new test
existed**:

| mutation | portable `cargo test --workspace` | toolchain `cargo test --workspace` | `c2rs diff wr1_sym_addr.cpp` |
|---|---|---|---|
| **M1** the address `addi` at its slot's turn in the descending walk | 571 / 0 | **571 / 0** | Mismatch @ obj 821 |
| **M2** REFLO written at `hi_off + 4` (both emitters) | 571 / 0 | **571 / 0** | Mismatch @ obj 1552 |
| **M3** `lo_off` derived as `base + 4` instead of searched | 571 / 0 | **571 / 0** | Mismatch @ obj 1552 |

The toolchain column is the surprise, and it refutes the lane's own registered
control (P0′, which predicted the toolchain lane would catch each one).
`crates/c2-harness/tests/differential.rs` runs the port against the reference on
**three named fixtures** — `add3.cpp`, `il_bool_materialization.cpp`,
`il_call_return.cpp` — and `wr1_sym_addr.cpp` is not among them. So
`cargo test --workspace` **never grades that fixture at all**, with or without a
toolchain, and the two lanes' totals are equal *because the integration tests
report `SKIP` and still count as `ok`* — no count distinguishes them and only a
mutation can. The single judge that went red is `scripts/gate.sh`: under M3,
**GATE: FAIL, 10 of 12 lanes MISMATCH**, 2,472 fixture-verdicts.

Restate §9.10's standing rule with the correction: a rung that touches `coff.rs`
must add a **portable** assertion for each ordering rule it establishes, because
the differential that catches it is `scripts/gate.sh`, **not** `cargo test`, and
a contributor who runs the workspace suite sees green either way.

**Eight tests, all portable, no toolchain**, in the three files where the two
rules actually live:

* `codegen/calls.rs` — the address `addi` is emitted LAST with a literal at a
  **strictly lower** slot (`s->m3(7, &gI)`, symbol at slot 2), plus the
  **symbol-at-slot-0 control** (`gsp(&gI, 7)`) which must stay **green** under
  M1. WR1's hand fixture had three copies of the control and none of the
  discriminator.
* `lib.rs` — `data_refs_of` **searches** the body for the low-half `addi`
  instead of assuming `hi_off + 4`, rebases both halves by the function's
  `.text` offset, and refuses four bodies it cannot read.
* `coff.rs` — the emitted quad's REFLO lands at `lo_off` in **both** emitters,
  the records are ascending-VA with REFHI ahead of its PAIR, the pooled-FP quad
  **is** adjacent (the negative that says the two quads are genuinely
  different), and the label triple's three slots bind `$M(n)`→prologue length,
  `$M(n+1)`→function length, `$T(n+2)`→`.pdata`, with the two `$M` written to
  the symbol table in the **opposite** order and the callee external between
  them.

**Mutation evidence — seven mutations, seven distinct messages**, each red on
the portable lane:

| mutation | site | assertion that fired |
|---|---|---|
| M1 descending walk | `sym_slots_text` | (c) `addi` must come LAST — and the slot-0 control stayed **green** |
| M2 REFLO at `hi_off+4` | `coff.rs`, both emitters | (d) packed, (h) COMDAT |
| M3 `lo_off = base + 4` | `data_refs_of` | (d) derivation |
| M4 PAIR emitted before REFHI | `coff.rs` | (f) record order |
| M5 the low PAIR dropped | `coff.rs` | (b) record count |
| M6 the two `$M` swap meaning (packed) | `coff.rs` | (n) `$M(n)` is the prologue length |
| M7 the two `$M` emitted in numeric order | `coff.rs` | (o) `$M(n+1)` is written first |

**M6 is the one worth reading.** The first draft of that test called only
`emit_comdat_obj`; swapping the two `$M` inside `emit_obj` under it left
`cargo test` at **85 passed / 0 failed**. One rule in two emitters, pinned in
one, is exactly how this file's `.pdata`-ordering bug survived. The shipped test
asserts both emitters.

Two smaller instrument facts, recorded because both are the
absence-read-as-success shape:

* `c2rs bench` is the **oracle self-test**, not the port differential. It prints
  `206 pass, 0 fail` under M1. `scripts/configure_existing_worktree.sh`
  advertises it as "every fixture, the correctness gate", which is how the lane
  nearly read a green bench as evidence that M1 was harmless.
* Writing the FP-adjacency test against `emit_comdat_obj` read **0 relocation
  records**: the COMDAT emitter carries no constant-pool code, because
  `PortC2::build` refuses a pooled constant under `/Gy`. It was moved to
  `emit_obj` rather than shipped as a control run where the effect cannot appear.

#### 9.12.2 #135 — the allocation order, widened from one body to 80 listings

`scripts/gt_label_cod.py`: 20 shapes × 4 flag sets (`/O1 /Oi /EHsc`, `/O1`,
`/O2 /EHsc`, `/Ox`), **80 of 80 listings captured**. Five shapes are the
**fitted** set; the other fifteen were held out and not looked at until the rule
was written.

**The rule.** In allocation order (ascending label number) the counter is
consumed **per function, in `.text` order**, and within one function:

1. one **funclet-entry** label per funclet the function needs (`__catch$k` /
   `__unwind$k`), **first**, before any of that function's `$M`/`$T`;
2. the function's **EH state-transition `$M`** block, ascending;
3. the **state table's own `$T`**, in `.rdata`;
4. then **one triple per emitted body** — the main body first, then each funclet
   in emission order — each triple exactly `$M(n)` prologue end · `$M(n+1)` body
   end · `$T(n+2)` `.pdata` record, consecutive, and the triples of one function
   consecutive **with each other, stride 3**.

Steps 1–3 are empty for a function with no EH, which collapses the rule to the
single triple `coff::plan_labels` already ships.

Eleven ordinal predicates, graded per (probe, mode):

```
                                      FITTED (5)   HELD OUT (15)
P1  every .pdata $T closes a triple      16/16         40/40
P2  $M(n) prologue < $M(n+1) end         16/16         40/40
P3  one triple per emitted body          16/16         40/40
P4  funclet allocated first                6/6         26/26
P5  …and emitted last                      6/6         26/26
P6  state table below the triples          6/6         26/26
P7  the $M block splits (EH)               6/6         26/26
P7b …and does not, without EH            10/10         20/20
P8  a function's triples stride 3          6/6         26/26
P9  functions in .text order                 —         16/16
P10 the main body's triple first           6/6         26/26
TOTAL                                    94/94       312/312  = 100.0 %
```

**Held-out accuracy: 312 of 312, 100.0 %,** on shapes the rule was not fitted on
(loop, switch, nested try, two catches, EH beside plain, two EH functions in one
TU, ctor/dtor, virtual, FP leaf, relational comparator, five leaves,
leaf-then-framed, many locals).

**The control that decides whether that is news.** A predicate that restated the
shipped model would go green on the whole in-class population by construction:

```
`coff::plan_labels` accounts for EVERY label in the TU
  non-EH rows   24/24   100.0 %
  EH rows        0/32     0.0 %
```

The shipped model is **complete on every non-EH row and complete on no EH row**.
The gap is entirely EH, the new rule closes all of it, and — stated honestly —
**the new rule adds nothing on non-EH bodies**: P4–P8 and P10 are `n/a` there
and what remains is the shipped triple.

**Falsification, because 312/312 is the shape this project reads as success when
it is really absence.** Seven mutations of the parsed allocation, each of which
must turn its predicate red:

| mutation | went red |
|---|---|
| every `.pdata` `$T` one higher | P1 (56) |
| funclet allocated last | P4 (32) |
| `$M(n)` / `$M(n+1)` offsets exchanged | P2 (56) |
| state table allocated above the triples | P6 (32) |
| the funclet's triple ahead of the main body's | P10 (32), P5 (32), P6 (32), P8 (32), P4 (20), P1 (2), P3 (2) |
| functions allocated in reverse `.text` order | P9 (16) |
| triples spaced 4 apart instead of 3 | P1 (56) |

**Two predicates survive every mutation: P7 and P7b.** They are §9.3's headline
— "the `$M` block splits around the `$T` tables" — and they are **entailed** by
P1 + P6 + P8 + P10, not independent evidence. Once a function has more than one
triple a `$T` necessarily sits between two `$M`; the split is a consequence of
"one (M,M,T) triple per emitted body, main first, then funclets", which is the
load-bearing statement. §9.3's phrasing is true and is not the finding.

**Two corrections to §9.3's wording**, both TU-versus-function scope:

* "the funclet is allocated **first**" is a **per-function** statement, not a
  per-TU one. At TU scope it is **false on 2 of 26 EH cells** — `eh_loop_two_fn`
  at `/O1` and at `/Ox`, where a first function's labels are allocated before a
  second function's funclet. Per function it is 32/32.
* the same for "the state table sits below the triples".

**What is NOT MODELLED, and the round refuses to guess it.** The rule above is
**ordinal**. The counter also consumes slots it never emits, and those gaps are
**not constant**:

```
gap: last funclet label  →  first EH-state $M    2, 3, 4, 5, 7, 8, 9, 10, 11
gap: state table $T      →  first triple         0, 1, 2, 3
```

So the *numbers* cannot be predicted from the shape, only their **order**.
Registered prediction B5 — "≥ 90 % of held-out label numbers predicted exactly
from the TU's first label" — is **REFUTED**, and that is the round's most useful
negative: `plan_labels` needs cardinal numbers, so **#135 ships no
`plan_labels` change**. A wrong stride is a wrong `$M` number and a wrong `$M`
number is a wrong-bytes obj. What ships is the instrument, the ordinal rule, and
the portable pin on the triple's slot binding (§9.12.1), which is the half of
#135 that *is* transcribable today.

A worked transcription, `eh_two_catch` at `/O1 /Oi /EHsc`, one function, three
bodies:

```
__catch$2553  funclet entry   text 0x5c     allocated first
__catch$2554  funclet entry   text 0x84
$M2564        EH state        text 0x24
$T2565        state table     .rdata
$M2568/$M2569/$T2570   main body      0x24 / 0x54 / .pdata
$M2571/$M2572/$T2573   __catch$2553   0x64 / 0x7c / .pdata
$M2574/$M2575/$T2576   __catch$2554   0x8c / 0xa4 / .pdata
```

Three instrument defects, all found before a verdict was read, all recorded
because each printed a plausible row:

1. a body-end `$M` took the **next** function's first offset — under `/Gy` that
   restarts at 0, so every multi-function TU read "end 0 < prologue 12" and
   **15 sound cells went red**;
2. `/Ox` names its sections with a bare `.rdata` directive rather than
   `NAME SEGMENT`, so every `/Ox` `$T` was attributed to `.XBLD$W` and the
   state-table predicate scored **`n/a` on all 20 `/Ox` rows** — which prints
   exactly like a predicate that passed;
3. a `.pdata` `$T` row sits **outside any `PROC`** and names its body in its
   `DD` operand; binding by position put all 56 of them in `fn_ix = -1` and the
   triple predicate read **0 of 56**.

#### 9.12.3 `LABEL_COUNTER.md` §6.15–§6.19 — the `.cod` evidence leaves every one of them UNTOUCHED

Not "widens some, refutes none". **Untouched, all of them, and for an
instrumental reason rather than a numerical one.**

§6.15–§6.19 do not measure the label counter. They measure the **inline-decline
decision** — how many of `N` call sites survived at a given callee size `s` —
through `scripts/gt_inline_decline.py`, which reads **REL24 relocation counts
and `bcctrl` counts** and reads **zero label symbols** (`grep -c` for `$M`,
`$T` and `first(` over that script: **0**). §6.15 says so itself: *"§6's law is
graded on label strides"*, distinguishing itself from the rounds that follow.
And §9.5 already recorded that the listing leaves the `/O1` inline-decline
schedule *unmoved*.

A `.cod` allocation order is a statement about the ordering of the counter
**within** a body. It is commensurable with §1 and §6.0–§6.14 (the counter, law
L′) — that is board #135 — and with nothing in §6.15–§6.19.

**The two vacuous ones specifically, and why `.cod` is not their remedy either.**
They are already named by the document's own §6.20 audit, so this lane did not
find them:

* **§6.15.2 — "dead locals move the decline by zero."** Vacuous because a
  `deadloc` ladder moves `s` by **zero by construction** — that is the point of
  the probe — so all twenty rungs sit at one index, 16 bytes below the nearest
  band edge, every cell saturated. A listing changes the *readout* of the
  decline, not the ladder's index range. **Untouched.**
* **§6.17.8 — "`/Ox`: there is no linkage split at all."** Vacuous because the
  sweep was `range(0, 9)` hardcoded and stops 28 bytes short of the only `/Ox`
  threshold there is; 36 cells rested on six, and the two rung kinds the section
  names first contributed none. Again a range-design fault. **Untouched.**

The one route by which `.cod` could ever touch these rounds is as a **second,
name-carrying source for the site count** — the listing prints every surviving
call by callee name, where the relocation table prints it by index. That is the
#136 relationship (a second instrument for the same observable), not new
evidence, and it is not scheduled here.

#### 9.12.4 Pre-registration scores

Registered at `6e3e9d3`, before the first mutation and the first capture.

| | registered | measured | |
|---|---|---|---|
| **P0** nothing portable pins either rule | portable stays 571/0 under M1–M3 | 571/0, three times | HIT |
| **P0′** each mutation is caught by the toolchain lane | toolchain `cargo test` goes red | **571/0, three times** | **MISS** |
| P1 `addi` last with a literal at a lower slot | passes; red under M1 | red, message (c) | HIT |
| P1′ the slot-0 control stays green under M1 | green | green | HIT |
| P2 `lo_off` searched, not `hi_off+4` | red under M3 | red, message (d) | HIT |
| P3 REFLO at 8 in the emitted obj | red under M2 | red, (d) and (h) | HIT |
| P4 the test-block total moves 571 → 577, [575, 580] | — | **571 → 579** | HIT |
| B1 non-EH triple contiguous and in order, 100 % | 100 % | 100 % (56/56) | HIT |
| B2 allocation order is text order, non-EH, 100 % | 100 % | 100 % (16/16 held out) | HIT |
| B3 funclet allocated first / emitted last, 100 % | 100 % of EH bodies | 100 % **per function**; 2 of 26 fail at TU scope | HIT, with a scope correction |
| B4 the `$M` block splits around the `$T`, 100 % | 100 % of EH bodies | 100 % — and **entailed, not independent** | HIT, vacuously |
| B5 ≥ 90 % of held-out label NUMBERS predicted exactly | ≥ 90 %, refuted below 70 % | **not modelled at all** | **MISS** |
| B2′ control: `plan_labels` < 50 % on refused shapes, new rule ≥ 90 % | discriminates | **0.0 % vs 100 %** | HIT |
| C `.cod` touches at most one §6.15–§6.19 negative | ≤ 1 | **0** | HIT |

**13 of 15.** Both misses are worth more than the hits. **P0′** changes a
standing rule: the lane predicted the toolchain lane would catch the mutations
and it does not, because `cargo test` never grades that fixture — so §9.10's
"portable lane" framing understates the gap by one column, and the sentence a
contributor needs is "run `scripts/gate.sh`", not "run the tests". **B5** is the
one that stops a bad change landing: without it this lane would have had an
ordinal rule and an invitation to fit a stride to five samples, and the gaps say
the stride is not there.

Two registered predictions were graded and found **vacuous rather than wrong**,
and are called out rather than counted as evidence: **B4** (entailed by B1/B3
once a function has two triples — it survives every one of the seven
falsification mutations, which is the definition of measuring nothing) and
**P9's fitted column** (0 of 0 cells: no fitted shape has two labelled
functions, so that column could not have failed; the predicate is graded on the
held-out column alone, 16/16).

#### 9.12.5 Gate evidence

Test blocks: **571 at the merge base `33d0049`, 579 at the tip** — the diff
§9.10 asks for, quoted at both ends. **Eight** new blocks, every one portable.

The first tally read **580**, and the extra one was a `#[test]` literal inside a
comment this lane wrote *about* the count. `git grep -c` cannot tell a comment
from an attribute, so §9.10's own metric is one a rung can inflate by writing
about it. The comment now spells the attribute out in prose and the grep and the
runner agree at 579.

* `cargo test --workspace` — **579 passed, 0 failed, 1 ignored** (the ignored
  one is the pre-existing doc-test), and the same **579 / 0 / 1** on the
  **portable** lane (`C2RS_WIBO=/nope C2RS_CL_EXE=/nope C2RS_C2_DLL=/nope`) —
  the eight new blocks are in both numbers.
* `c2rs selftest` — **206/206 PASS**, 0 fail, 0 error.
* `scripts/gate.sh --jobs 6` — **GATE: PASS, 12/12 lanes ran and every one
  graded a corpus**, 2,472 fixture-verdicts, 206/206 in every lane, 0 mismatch.
  `scripts/gate.sh --selftest` — PASS, 15 cases.
* No port behaviour changed: this lane added tests, one tooling script and this
  section, and wrote no emitter code.
