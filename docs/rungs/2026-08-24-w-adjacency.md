# The cost protocol's rotation now balances ADJACENCY — and #3468's *sweep* was wrong about five of the runs it explained, which reading alone shows

    Tag:       w-adjacency
    Slug:      w-adjacency
    Date:      2026-08-24
    Kind:      construct (instrument)
    Outcome:   built
    Fixtures:  none — instrument lane: `scripts/cost_arms.py` only; `crates/` untouched
    Census:    +0 — nothing in `crates/` moved; this lane's fence forbids it
    Record:    this file; prereg `docs/rungs/_2026-08-24-w-adjacency-prereg.md`
    Base:      67f276409   Board: #3521–#3524

> **What this lane can fail on, named before it started.** It is an instrument
> lane, so its failure axes are (i) shipping a design that does not actually
> balance, (ii) shipping a guard that does not refuse, and (iii) claiming a
> validation whose control did not fire. §2 answers (i) by re-deriving the
> counts from the flat sequence rather than from the generator, §3 answers (ii)
> by watching four refusals, and §4's registered P3a answers (iii) by requiring
> the *old* rotation to reproduce the artefact on the same binaries before any
> credit is taken for the new one.

---

## 0. HEADLINE

*(filled in §4 order once the measurements are in — see §4.)*

---

## 1. THE BRIEF'S CITATIONS

All eleven verified; see the prereg §1. Two are worth lifting out.

**The brief understated the cyclic defect.** It says one 3-arm cycle gives
`base→nulldup ×2`, `nulldup→tip ×2`, `tip→base ×2` and that `nulldup` "never
precedes" `base`. Re-derived mechanically by
`cost_arms.py --show-design 3 --rotation cyclic`:

```text
  adjacency counts over one cycle (circular; rows = predecessor)
        A   B   C            position counts (rows = arm, cols = slot)
    A   0   2   1              A   1   1   1
    B   1   0   2              B   1   1   1
    C   2   1   0              C   1   1   1
```

The reverse pairs are **1**, not 0 — the boundary between rounds supplies them —
so the imbalance is **2-vs-1 per cycle**, not 2-vs-0. The position table beside
it is the point: **perfectly balanced, and it is the only thing #3468's
criterion looked at.**

**The brief's "Williams design, 6 sequences" is the right number for the wrong
reason, and the difference matters at even `n`.** A Williams design balances
adjacency *within* a sequence. Here the round boundary is **not a pause** — the
last arm of round `r` and the first of round `r+1` are two back-to-back
`c2rs perf` invocations with nothing between them — so a Williams-only fix would
leave `1/n` of all adjacencies unbalanced (12 of 36 at 12 rounds over 3 arms):
the same defect at a third of the size. The design here balances the **whole
flat sequence**. For `n = 3` that is also 6, so the brief's number holds; the
general rule is `L = 2n` and it differs from Williams at even `n` (Williams
needs `n`).

---

## 2. THE DESIGN — `L = 2n`, and the counts are forced rather than chosen

The rotation is a cycle of `L = 2n` rounds, each round a permutation of the
arms, with three exact properties over one cycle:

1. each arm holds each slot of a round exactly **2×** — #3468's criterion,
   subsumed and not dropped;
2. each of the `n(n-1)` ordered cross pairs `a→b` occurs exactly **2×**;
3. each of the `n` self pairs `a→a` occurs exactly **2×**.

**Why `2n` and why 2-and-2.** A cycle of `L` rounds has `L·n` adjacencies read
circularly, of which `L(n-1)` lie inside rounds and are necessarily cross pairs.
Balance requires `n(n-1) | cross_total` and `n | self_total`. At `L = 2n` the
within-round pairs alone already supply `2n(n-1)` cross adjacencies out of
`2n²` total, so `cross_each = 2` and `self_each = 2` is the **only** solution —
which forces **every one of the `2n` round boundaries to be a self-repeat, two
per arm**. That is balance, not a defect: each arm gets exactly two warm
restarts per cycle, so the min-over-rounds estimator draws from the same mixture
for every arm. `L = n` admits no design at all for `n = 3` or `n = 4`
(exhaustive DFS, no solution, budget not reached), which is why the cycle is
`2n`.

**The generator is never trusted.** `carryover_cycle(n)` is a deterministic
depth-first search with no RNG — same design on every box, every run — and
`verify_design(cycle, n)` re-derives every count from the flat slot sequence
and is the only thing the runner believes. A disagreement between them stops
the run (`REFUSING: the balanced rotation … does NOT verify`). `--show-design N`
prints the design, both count tables, and the verdict.

| arms | cycle | `--show-design N` verdict |
|---|---|---|
| 2 | 4 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 3 | 6 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 4 | 8 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 5 | 10 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 6 | 12 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 7 | — | **`REFUSING: no balanced 14-round cycle found … within 2000000 steps / 40s`**, exit 1 |

Seven arms is a **refusal, not a degradation** — the script does not fall back
to a cyclic rotation, because a silent fallback is the exact defect #3495 filed.

**One adjacency in `rounds·n` is necessarily unbalanced, and it is the last
one.** The cycle balances as a *circle*; a run is a *line*, so the wrap from the
final arm back to the first never happens. `rounds·n − 1` adjacencies cannot
divide evenly among `n²` pair classes, so some class is short by one; this
design makes it exactly one, which is the floor and not a slack. At `--rounds 6`
over 3 arms that is 1 of 18. Registered in the prereg §2 before measuring, so it
is a stated property and not a discovery.

---

## 3. EVERY GUARD WATCHED REFUSING ON DELIBERATELY BROKEN INPUT

`CLAUDE.md`: *"Before relying on any `--check` flag, watch it fail on
deliberately broken input."* Four refusals, each **exit 1**:

| fed | result | exit |
|---|---|---|
| `--rounds 9`, 3 arms — **legal under #3468, illegal now** | *"not a positive multiple of 6 (the balanced cycle for 3 arms) … Use 12."* | **1** |
| `--rounds 8`, 3 arms — illegal under both | *"… Use 6."* | **1** |
| `--rounds 0` — degenerate | *"… Use 6."* | **1** |
| a null arm that is **not** byte-identical | *"arm nulldup is NOT byte-identical to base: it cannot be the null."* | **1** |
| `--show-design 7` — an arm count with no verified design | *"REFUSING: no balanced 14-round cycle found …"* | **1** |

The first row is the one that matters: **`--rounds 9` over 3 arms is now
refused, and it is the count three of this protocol's four prior readings were
taken at** (`w-s1c3` run 2, `w-permute` runs 1–3).

`--rotation cyclic` still accepts 9, deliberately — it exists to reproduce a
prior reading as a control and prints a warning saying its numbers are not
comparable to a balanced run's.

---

## 4. THE ACCEPTANCE TEST

*(measurements — filled below.)*

---

## 5. WHAT READING ALONE ALREADY SHOWS: #3468's *sweep* covered runs its
## explanation does not fit

This costs nothing and it was found before a single arm was timed, by reading
the three prior lanes' own protocol paragraphs for their **arm count and round
count** — which is `CLAUDE.md`'s read-before-probe doctrine applied to the
project's own record rather than to `c2.dll`.

#3468's finding is *"the ±1–1.7 % floor is not the box, it is an **incomplete
rotation**"*, and its evidence column sweeps in `w-s1bc` §4.3 and `w-s1c2`
§4.1/§4.3 — *"null arms −1.08 %, +0.47 %, +0.52 %; splits 40 %, 52 %"* — as
instances of the same defect. **Two of those three runs had a rotation that
completed.**

| lane | run | arms | rounds | cyclic rotation completes? | null | split |
|---|---|---|---|---|---|---|
| `w-s1bc` | first (loaded) | **4** | **8** | **YES** (8 % 4 = 0) | **+0.47 %** [+0.12, +0.83] | **57 %** |
| `w-s1bc` | final (quiet) | 4 | 8 | **YES** | −0.08 % [−0.34, +0.17] | 46 % |
| `w-s1c2` | 1 (loaded) | 3 | **6** | **YES** (6 % 3 = 0) | **−1.08 %** [−2.81, +0.65] | **40 %** |
| `w-s1c2` | 2 (quieter) | 3 | 8 | no | +0.52 % [−0.52, +1.57] | 52 % |
| `w-s1c3` | 1 | 3 | 8 | no | +0.57 % [+0.45, +0.68] | 76 % |
| `w-s1c3` | 2 | 3 | 9 | **YES** | +0.09 % [−0.07, +0.26] | 51 % |
| `w-permute` | 1 | 3 | 9 | **YES** | +0.29 % [+0.14, +0.44] | 62 % |
| `w-permute` | 2 | 3 | 9 | **YES** | +0.46 % [+0.32, +0.61] | 71 % |
| `w-permute` | 3 | 3 | 9, list reordered | **YES** | +0.06 % [−0.05, +0.17] | 54 % |

**Seven of the nine runs this protocol has ever produced had a complete cyclic
rotation, and their nulls range from −1.08 % to +0.46 % with splits from 40 % to
71 %.** Completeness predicts nothing. #3468's own *controlled comparison*
(8 vs 9 rounds, same box, same binaries, twenty minutes apart) stands untouched
— it is a real A/B and it found a real effect. What does not survive is the
**generalisation** to the two predecessor lanes: `w-s1bc`'s 8-over-4 and
`w-s1c2`'s 6-over-3 were already position-balanced, so whatever moved their
nulls was never the incompleteness.

And the 4-arm case is the worse one. `--show-design 4 --rotation cyclic` on
`w-s1bc`'s exact configuration:

```text
        A   B   C   D                    A   B   C   D
    A   0   3   1   0                A   1   1   1   1
    B   0   0   3   1                B   1   1   1   1
    C   1   0   0   3                C   1   1   1   1
    D   3   1   0   0                D   1   1   1   1
    adjacency per cycle              position per cycle
```

`A→B` happens **three** times per cycle and `B→A` **zero**. `w-s1bc`'s `nulldup`
sat in slot 2, immediately after `base`, on **three of every four rounds and
never before it** — a stronger version of exactly the relation `w-permute`
reversed. Its two runs are the pair with the largest disagreement in the table
(+0.47 % vs −0.08 %).

### 5.1 THIS SECTION REFUTES AND DEFERS — it establishes no cause, and must not be cited as if it did

**What §5 shows:** a complete cyclic rotation is **not sufficient** for a null
arm to read zero. That is a clean refutation and it rests on arithmetic anyone
can recheck (`8 % 4 == 0`, `6 % 3 == 0`).

**What §5 does NOT show, and is not evidence for:** that *adjacency* is what
explains the spread. Those nine runs differ in **many ways at once** — different
lanes, binaries, corpora, box loads, arm counts, dates — and **nothing in the
table varies adjacency alone**. The adjacency tables above explain how each
configuration *could* carry an order artefact; they do not measure that it did.

This is **#3483's sharpening in its exact shape**: a comparison can prove a
property is not sufficient without licensing any claim about what is. The
generalisation this lane is taking apart — #3468's sweep of two predecessor runs
into an explanation that did not fit them — was built by exactly this move, and
repeating it one level up is the failure mode, not the finding.

**The positive claim is carried by §4 and by §4 only**: runs A and B differ in
`--rotation` and in nothing else — same binaries, same box, same 157-fixture
population, same round count, minutes apart. Everything §5 offers is a reason to
have run that control, not a substitute for it.

**If run A comes back clean**, the registered UNPOWERED verdict covers this
section too: §5 would then be a **live anomaly with no established cause**, and
it is labelled that way rather than quietly retained as support.

**Filed beside #3468, never over it** (`w-r8idiom`'s convention, the same one
#3495 used). #3468 is merged and pushed; its measurement and its controlled
comparison stand untouched.

---

## 6. Gate evidence

*(filled below.)*

---

## 7. What was NOT done — priced, not silently dropped

*(filled below.)*
