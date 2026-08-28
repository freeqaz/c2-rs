# w-doctrine — the refusal-domain requirement is a `cargo test` now, and the control that proves it is not C6

    Tag:       w-doctrine
    Slug:      w-doctrine
    Date:      2026-08-28
    Kind:      instrument (consolidation lane, decision 21 §2)
    Outcome:   instrument
    Fixtures:  none — instrument lane: `crates/c2-core/src/surface.rs`, the
               decision-surface registry that makes board #3723's refusal-domain
               requirement checkable instead of advisory
    Census:    +0 (no acceptance predicate moved, no emit widened, no `.text`
               byte changed — §5)
    Wave:      17 (`docs/DECISIONS_2026-08-22.md` § Decision 21 §2)
    Board:     #3743–#3748
    Base:      master `8213c7b77` (decision 21's own commit)
    Prereg:    `work/w-doctrine/PREREG.md`, the FIRST commit on `wt-w-doctrine`
               (`edbee1f8f`), before the first edit to `crates/`
    Fail axis: **the control set.** This lane changes no byte by construction —
               it is purely additive to `crates/` — so the byte delta cannot
               fail here and grades nothing. Every part must be watched RED on a
               planted defect, and **E1 must be watched red on C6
               specifically**; a part never observed failing does not ship.
               Registered in PREREG §4 before the first edit. Nine planted,
               nine red — §4, and two of the four surfaces' coverage claims were
               FALSE until the control set executed them — §4.2
    Byte delta: REQUIRED-ZERO, measured — §5

---

## 1. What was asked, and the one-line answer to each

| ask | answer |
|---|---|
| make `#3723` **enforceable rather than advisory** | **Built**, as four `cargo test` assertions over a committed artifact plus one over the rung docs. No new script, no new gate row, nothing that needs remembering to run |
| *what is the checkable form?* — deliberately not answered for me | **A registry of decision surfaces whose domains run past what the corpus exercises, rendered to a committed baseline.** §2 |
| **watch it FAIL first** (`#3336`) | **Nine planted defects, nine watched red**, §4. And the control set did more than confirm: it **refuted two of the lane's own coverage claims** — §4.2. Transcript at `work/w-doctrine/controls_red.txt` |
| **reconstruct C6 and show the check going RED** | **Done, one line for one line.** `THE DECISION-SURFACE DOMAIN MOVED — 127 line(s) of 1102 differ`, §4.1 |
| …while the byte delta and the identity diff stay green | **Measured**, §5.2 |
| do not change `gate.sh`'s verdict, do not add a count-bearing row | **None added.** 21 rows at both ends, enumerated by the diff script |
| licenses no emit; never in `gate.sh`'s verdict | **Stated in the module header and in `rungs/README.md`'s new clause.** `FUNCTION_BYTE_MATCH.md` §0 |
| do not weaken the byte delta | **Not weakened.** The clause says in its own words that it stays *necessary* and is now known not to be *sufficient* |
| mind the blast radius, five worktrees live | **Scoped the way `#3690` did**, and declared in the prereg rather than discovered — §6.2 |
| if the checkable form does not exist at acceptable cost, say `declined` | It exists. One of the three candidate forms **was** declined, with its price — `#3747`, §6.1 |

---

## 2. The finding, before the machinery

**The sharp control is not C6.**

C6 — `w-regsel`'s planted widening of the caller's allowed set to `r0..r31` — is
the defect `#3723` is filed about, and reconstructing it was the lane's assigned
demonstration. It works: E1 goes red on it. But C6 also reddens **three other
tests in `crates/c2-core`**, and all three were written by `w-regsel`, the one
lane that happened to build a refusal grid because its prereg told it to.

So C6 alone cannot distinguish *"a registry catches this"* from *"a diligent
lane catches this"*. The control that can is **CS**:

```text
  FRAME_MAX_SAVED_NO_SPILL: u8 = 17   ->   31
  test result: FAILED. 670 passed; 1 failed
  the one failure: surface::tests::the_decision_surface_domain_matches_the_committed_baseline
```

`FRAME_MAX_SAVED_NO_SPILL` is the allocator-spill lock, and `frame.rs`'s own
comment beside it reads *"unreachable behind the two helper thresholds today,
and kept as the second lock because the sizing rule stops being exact here and a
wrong `stwu` immediate is one silent byte."* **Unreachable-behind-another-guard
is exactly the state in which a widening is invisible** — to every fixture, to
every gate row, to every byte, and to every test 671 of them long. Nobody was
dispatched to write a grid for it, and nobody would be, because it is behind
another guard.

That is the difference a registry makes and it is board `#3745`. `#3723` could
have been answered by telling every future lane to write a grid; the lanes that
follow the instruction produce C6's three witnesses, and the surfaces nobody was
sent at produce CS's zero.

---

## 3. What was built

### 3.1 `c2_core::surface` — the registry

Four surfaces, deliberately from **three unrelated families**, because a
registry covering only `w-regsel`'s own grid would be that lane's test moved to
a shared file:

| surface | site | the boundary | cells | refusals |
|---|---|---|---:|---:|
| `alloc.allocate` | `codegen/alloc.rs` | which registers a store run's producers get, and where the allocation refuses | 256 | 190 |
| `regalloc.select` | `codegen/regalloc.rs` | c2's minimum-cost selector over an ordered list, across **all four** entries of `ORDERS` | 224 | 50 |
| `frame.out_of_class` | `codegen/frame.rs` | which frame layouts each of the **three** prologue emitters admits, and the named reason for the rest | 504 | 453 |
| `reach.branch` | `codegen/reach.rs` | direct / expanded / refused, per branch form and displacement | 75 | 36 |

**The domains run past what the corpus reaches, and that is the whole
mechanism.** `pool_floor` to 31 where no fixture exceeds a narrow band;
producers to one past `MAX_MODELLED_PRODUCERS` so the cap is inside the
enumeration; `saved_gprs` to 18 so the spill lock behind the helper thresholds
is reachable; every displacement a boundary value or one word past one. A domain
that stops where the corpus stops reproduces the defect exactly.

Three details worth not re-deriving:

* **The row generators live in the modules they characterize, not in
  `surface.rs`.** `codegen::regalloc`'s cost fence scans every *other* file in
  the crate for `regalloc::select` call sites and for the non-default order
  names; enumerating `ORDERS` from `surface.rs` would trip it, **correctly**,
  because from there it would look like a new consumer.
* **`surface.rs` is inside its own marker scan and needs no exclusion.** The
  marker token is assembled at run time from two halves, so the file does not
  contain it. `w-regsel`'s cost fence had to exclude itself because it greps for
  a token it must contain to do the grepping; an exclusion is a hole, and this
  one is closed by construction rather than by a list.
* **`regalloc.select` enumerates the three non-default orders**, which no obj
  cell exercises and no gate row can reach. A transcription error in
  `0x10c37e50` or `0x10c37eb8` is now a moved line instead of a silence.

### 3.2 The four assertions, and what each is for

```text
  E1  live domain == surface/DOMAIN.txt        #3723 itself
  E2  markers <-> registry is a bijection      #3641 — a rename emptying the population
  E3  per-surface cell and refusal floors      #3470 — a check over zero cells is green
  E4  every boundary const covered or listed   the registry's own completeness (#3689's ratchet)
```

**The mechanism is the bless, not the block.** E1 does not decide whether a
widening is right and could not. The only way past a red E1 is to re-bless the
baseline — which puts the widening in the diff **as text somebody reads**. The
instrument makes a widening impossible to make *by accident*, which is the whole
of what `#3723` asks for.

### 3.3 `Fail axis:`, and the honest size of it

`rung_registry.rs` asserts a non-empty `Fail axis:` header field on every
construct rung dated **2026-08-28 or later**. Earlier records stay exactly as
written, which is `#3689`'s grandfathering and its reason.

**It grades zero docs today** — 330 rung docs examined, 7 declaring a construct
rung, **0** in the population — so without a control it would be decoration by
construction. It has a two-directional self-test on synthetic headers (fires on
missing, fires on empty, quiet on named, quiet on hyphenated, quiet on
non-construct, quiet before the cutoff) and a planted dated rung that reddens the
tree run by name.

**Presence is not measurement, and `#3744` says so in those words.** The field
check cannot tell a named axis from a measured one. It ships for the one thing
E1–E4 cannot do: bind a future lane over a surface the registry does not hold.

---

## 4. Estimate vs outcome, graded against the frozen prereg

| # | prediction | conf. | outcome |
|---|---|---:|---|
| **P1** | E1 goes red on C6 reconstructed, on a tree where the identity diff still reads 0 lines over 21 rows and the gate still reads `PASS` | 0.85 | **CONFIRMED** — §4.1, §5.2 |
| **P2** | C6's signature is a collapse of `alloc.allocate`'s refusal count, with the **emitted value set unchanged** | 0.75 | **CONFIRMED.** Refusals 190 → 64; `values={r10,r11,r9}` at both ends. The tail is never *selected* because the order's head is `r11` whatever the allowed set is — what moves is the **refusal**, which is why a refusal domain and not an emitted-value set is the right instrument |
| **P3** | at least one surface outside the regalloc family fires on a planted one-token widening | 0.90 | **CONFIRMED, and stronger than predicted** — the frame surface produced CS, the control where E1 is the *only* red in 671 tests (§2) |
| **P4** | required-zero byte delta at my own tip | 0.95 | **CONFIRMED** — §5 |
| **P5** | the boundary-const screen is **noisy**: at least one named const is not a decision boundary | 0.80 | **CONFIRMED** — two of the twelve. `R_BOUND` is a **register number**; `TOP` is a loop **byte offset**. Both are listed as false positives at the site rather than quietly dropped |
| **P6** | the `Fail axis:` check grades zero docs today and needs its own control | 0.90 | **CONFIRMED** — 0 of 330, §3.3 |
| **P7** | no new count-bearing gate row; 21 rows at both ends | 0.97 | **CONFIRMED** — §5 |

**Calibration: seven for seven, and the one that was nearly wrong is P2.** It
predicted the *mechanism* of C6's signature correctly and would have been
refuted by an emitted-value change; the refutation would have been the more
interesting result. Registering it at 0.75 was right and it survived. The two
things the prereg did **not** predict are in §6.

### 4.1 The controls — seven planted, seven watched red

`work/w-doctrine/controls_red.txt`, harness committed
(`work/w-doctrine/control.sh`), tree verified clean after each. The harness
records **both** halves on purpose: `#3723`'s claim is that a defect can be red
here while every emitted-byte test stays green, so the green half is evidence.

| C | planted defect | c2-core red | part |
|---|---|---:|---|
| **C6′** | **`w-regsel`'s C6 reconstructed** — the caller's allowed set widened to `r0..r31` | 6 of 671 | **E1 + E3** |
| CF | `needs_gpr_helper` `>= 3` → `>= 4` | 2 of 671 | E1 |
| **CS** | **`FRAME_MAX_SAVED_NO_SPILL` `17` → `31`** | **1 of 671** | **E1 alone** |
| CR | `BC_MAX_DISP` `32764` → `65532` | 5 of 671 | E1 |
| CN | a surface's source marker renamed | 1 of 671 | E2 |
| **CZ** | **the reach domain collapsed to one cell AND the baseline re-blessed** | **1 of 671** | **E3 alone, with E1 GREEN** |
| CU | a new unregistered boundary-named const | 1 of 671 | E4 |
| CD | a dated construct rung with no `Fail axis:` | (harness) | E5 |
| CM | `FRAME_MIN_OUT_SLOTS` `8` → `16` | 28 of 671 | E1 (after §4.2's repair) |
| CS-2 | CS re-run after §4.2's repair — the sharp control must stay sharp | **1 of 671** | **E1 alone** |

**C6′ verbatim, quoted from the run:**

```text
THE DECISION-SURFACE DOMAIN MOVED — 127 line(s) of 1102 differ from
crates/c2-core/src/surface/DOMAIN.txt.
  - alloc.allocate  kind=const n=1 floor=12  REFUSE
  + alloc.allocate  kind=const n=1 floor=12  r11
  - alloc.allocate  kind=const n=1 floor=13  REFUSE
  + alloc.allocate  kind=const n=1 floor=13  r11
  …
```

126 domain cells and one summary line. Every moved line is a `pool_floor` no
fixture reaches, which is precisely why the byte delta cannot see them.

**CZ is the control that matters for the instrument's own honesty**, and it is
board `#3748`. The domain was collapsed **and the baseline re-blessed in that
state**, so E1 compares equal and is green — a baseline check grades a domain
against itself and cannot see that the domain stopped grading anything. E3's
floors are the only thing left and they fire. That is not a contrived failure: a
lane that shrinks a domain and re-blesses in the same motion has performed CZ,
with no dishonesty required.

### 4.2 **Two of the four surfaces' guard claims were FALSE, and the control set found them**

`Surface::guards` is a **claim** — *this surface's domain exercises this
boundary constant* — and E4 reads it as the coverage numerator. The field's own
doc says *"an entry here that the domain does not reach is a false coverage
claim and the control set is what keeps it honest."* Executed against all seven
guards, two were exactly that:

```text
  POOL_TOP             re-spelled as a literal `9`   ->  0 lines moved
  FRAME_MIN_OUT_SLOTS  widened 8 -> 16               ->  0 lines moved
```

**They are the two different ways a coverage claim goes wrong and they need
different repairs**, which is why they are worth separating:

* **`POOL_TOP` is not an independent boundary at all.** Since `w-regsel` it is
  `GPR_DEFAULT.regs[0]` — a derived alias with **no production use outside tests
  and comments**. There is nothing for a domain to reach. It moves to
  `UNCOVERED` with the measurement as its reason, ratchet 12 → 13, and the thing
  it aliases is covered by `regalloc.select`.
* **`FRAME_MIN_OUT_SLOTS` is a real boundary the DOMAIN was too narrow to
  see.** At the domain's original `locals = 20_000` the floor shifts the frame
  by 64 bytes and lands on the same side of every threshold. Repaired by
  choosing `locals = 20_360`, which straddles the `_RtlCheckStack12` threshold
  *through* the floor — 20,448 admitted at a floor of 8, 20,512 refused at 16.
  Control **CM** then moves **7 lines**, and control **CS-2** confirms the
  repair did not cost the sharp control: `670 passed; 1 failed`, E1 alone.

**This is the lane's instrument catching the lane's own false claim, and it is
the reason a coverage list has to be executed rather than written.** A registry
that reports coverage it does not have is `#3470` with one extra step: the
number goes **up** and the grading does not. It is folded into `#3746` rather
than given a row of its own, because it is that row's own subject measured on
the instrument that states it.

---

## 5. Gate evidence

### 5.1 This lane's own tree

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **60 targets, 1,982 tests passed, 0 failed**, `TEST_EXIT=0`, **0 `SKIP: toolchain absent`** — `work/w-doctrine/test_release.out` |
| `scripts/gate.sh --jobs 16 --require-graded` (BASE `8213c7b77`) | `GATE: PASS`, 18/18 lanes graded, sweep 19,460/19,556, cross 90,424/90,812, **0 mismatches anywhere**, `GATE_EXIT=0` — `work/w-doctrine/gate_base.out` |
| `scripts/gate.sh --jobs 8 --require-graded` (TIP, re-run after §4.2's domain repair) | `GATE: PASS`, same counts, **0 mismatches anywhere**, `GATE_EXIT=0` — `work/w-doctrine/gate_tip.out` |
| `scripts/gate_identity_diff.sh base tip` | **`IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS`**, `21 base, 21 tip (enumerated, not asserted)` — `work/w-doctrine/identity_tip.txt` |
| `scripts/gate_identity_diff.sh --self-test` | `SELF-TEST PASS` — enumeration 21, control silent, `#3515`'s signature found exactly (14 lines / 7 rows) and nonzero, truncation refused |
| `scripts/board_audit.sh` | 0 uncited, 0 unresolved anchors, 0 raw line anchors, 0 duplicates |
| `scripts/prose_audit.py` | `VERDICT: CLEAN over 661 checked claims` |
| `scripts/tracked_artifact_audit.sh` | `examined 9871 tracked files across 5 classes; 0 violation(s)` |
| `scripts/provenance_census.py --since 8213c7b77` | **+6 new constants, 6 already tagged**; `→untag 0`, `reclass 0`, decomposition identity holds 6 of 6 |
| fixtures, `c2rs census` | unchanged — `Fixtures: none`, `Census: +0` |

**`HATCH-RED REFUSED` at both ends, identically**, for the pre-existing
`HATCH-STALE` reason (board `#1389`) present on the untouched base. That row is
one of the two `n/a`-mismatch rows `gate_identity_diff.sh` excludes by name, so
it is outside the 21 and outside the claim.

### 5.2 **The demonstration — one tree, three verdicts**

`bc658b2e0` is `w-regsel`'s C6 planted on this lane's tip, one line for one line
(`crates/c2-core/src/codegen/alloc.rs | 2 +-`). It is a **committed** probe and
not a working-tree edit, because `gate.sh` refuses a dirty tree in those words —
*"a gate run is evidence about the tree it graded"* — and it is **reverted in
the commit immediately after**, with the reason in that commit's message so
nobody rebuilds on it.

On that one tree:

```text
  scripts/gate.sh --require-graded
      GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them
      graded a corpus, 90424 of 90812 case-lane cells, 0 mismatches anywhere
      GATE_EXIT=0

  scripts/gate_identity_diff.sh gate_base.out gate_c6.out
      count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
      IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS

  cargo test -p c2-core --lib surface::tests::the_decision_surface_domain
      FAILED
      THE DECISION-SURFACE DOMAIN MOVED — 127 line(s) of 1102 differ from
      crates/c2-core/src/surface/DOMAIN.txt.
```

**That is `#3723`, closed.** The criterion decision 20 grades construct rungs on
says `PASS` and `0 lines`; the instrument this lane built says the port's
refusal boundary moved on 126 cells the corpus never reaches, and names them.
Transcripts: `work/w-doctrine/gate_c6.out`, `work/w-doctrine/identity_c6.txt`,
`work/w-doctrine/c6_e1_red.txt`.

---

## 6. What contradicts the brief, and what this lane refused

### 6.1 The third candidate form is declined, not deferred — `#3747`

The brief offered three candidate forms and endorsed none. The third — *a
predicate over the diff that decides whether a change could widen an allowed
set, so the requirement fires only when it applies* — **is not built, and the
reason is that E1 makes it redundant rather than that it is hard.**

The baseline comparison is **unconditional and costs 20 ms**. A change to a
registered surface is caught whether or not any classifier judged the diff
relevant, so a predicate in front of it could only ever *lose* cases — a
widening reached through a path the screen did not classify. And the residue a
predicate would have been for, an **unregistered** surface, it would not have
covered either: it would flag the diff and have nothing to compare it against.

A conditional check is weaker than an unconditional one whenever the
unconditional one is cheap. It is recorded so the candidate is not re-taken as
an obvious gap.

### 6.2 The blast radius was never a property of "a check", it is a property of WHICH check

`#3684` left two tree audits unwired for a wave because *"a tree audit under
`cargo test` makes every lane's `cargo test` depend on every doc in the tree"*,
and `#3690` closed it by observing that this was true of some checks and false
of others. The same split applies here and was designed in rather than
discovered:

* **E1, E2, E3** grade a population that is **opt-in by construction** — a
  surface is graded because somebody registered it, which *is* the request to
  have it watched. They can only redden the tree that changed a registered
  surface. A peer lane that touches none of the four is unaffected, and four
  peer lanes are live.
* **E4** is a ratchet with a one-line raise, visible in the diff.
* **E5** grades rung docs, which live only in their own lane's worktree until
  they merge; the merge funnel (`#3687`) runs `cargo test --workspace`, so a
  doc that fails it cannot reach master and cannot redden a peer's tree.

### 6.3 What this lane refused

* **A new count-bearing `gate.sh` row.** `#3691`: a 22nd row makes
  `gate_identity_diff.sh` **exit 2, refusing to diff at all**, for every live
  lane holding a 21-row base, on a tree they did not touch.
* **Any weakening of the byte delta.** The new clause says it stays necessary.
* **Any claim of registry completeness.** `#3746`: 12 of 19 boundary-named
  consts are uncovered, four of them named as real refusal boundaries with no
  enumerated domain. The screen is a **name** screen and is therefore both
  incomplete and noisy; both halves are printed rather than hidden.
* **A new `scripts/` entry.** `#3679` is a check that existed, was green, had a
  self-test, and nothing ran it. Everything here is a `cargo test`, which the
  merge funnel already runs.
* **Editing another lane's board block or the reservation ledger line.**
