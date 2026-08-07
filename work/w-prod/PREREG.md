# w-prod — PREREGISTRATION

    Lane:   w-prod, worktree branch `wt-w-prod` off master `6dce97a1`.
    Filed:  2026-08-09, **before this lane's first grid cell existed** and
            before the first line of `crates/` was written.

This file is committed **before any probe**. Nothing below may be edited after
the first cell of GRID P is compiled; the scorecard is appended to the rung, not
to this file.

---

## 0. What this lane is, and what it is NOT

This is a **representation rung**. The deliverable is that a fact which has been
measured five times becomes **expressible and measurable inside `crates/`**. It
is not a rule.

`crates/c2-core/src/codegen/alloc.rs` records **nine** dead allocation keys —
`w-next` 7/56, `H-self` 11/72, `clause-1-strict` 12/36, `RULE W` 7/388,
`RULE W2` 14/106, `KEY ILX` 14/45, `RULE BIND` 5/38, `H-MIX` 12/62, `H-2X`
12/72 — every one killed on a frozen, never-fitted holdout. **The only
allocation statement in `crates/` is a refusal, and it is 0 wrong.**

`w-self2b` (board #1231) found why they died, by decoding the IL rather than
diffing objs:

> `prod` appears exactly where the **store designator's root token is a temp
> bind head AND differs from the value expression's root token**.

That is a **relation between two `B9` roots plus one bit about one of them**,
and neither carrier in the port can hold it: `alloc::Producer` carries `uses` /
`kind` / `first` — facts about *one* producer — and `c2-il`'s `eat_offset_adds`
returns the **sum** of the offset-add literal list rather than the list
(board #908). **Nine rules were trying to state a relation in a structure that
only holds per-producer facts.**

**A representation that makes the fact measurable and ships no rule is the
success condition of this lane.**

---

## 1. The carrier — its shape, registered before it is written

Per **producer**, both sides of the relation:

```text
    Root  { tok: u32, is_bind: bool, offsets: Option<Vec<i32>> }

    ProducerRoots { value: Root, lvalue: Root }

    Producer { id, kind, uses, first, roots: Option<ProducerRoots> }
```

* `tok` — the root symbol token of the `B9 <tok> <TYPE>` this expression is
  rooted at.
* `is_bind` — whether that root is a temp **bind head** (`26 <tok>`), i.e.
  board #1128's *"a bind IS a second base symbol"*.
* `offsets` — the offset-add literal **LIST**, `None` where the reader carried
  only its sum. `Some` is the #908 repair; `None` is an honest refusal, never a
  one-element list holding the sum.
* `roots: None` — the producer reached `alloc` from a path that decodes no
  designator. `allocate` must behave identically either way.

### 1.1 What becomes expressible that was not — registered as the claim

Four statements, none of which can be written today:

1. **`p.roots.value.tok != p.roots.lvalue.tok`** — the relation. Today the two
   tokens are *both present at one site* in `codegen/leaf/store.rs`
   (`parse_simple_gpr_run`, the `base_tok` binding and the value arm's
   `IlOp::BoundAddr { .. }`) and **the value's token is discarded by a `..`
   pattern**, because there is nowhere to put it.
2. **`p.roots.lvalue.is_bind`** — the schedule bit of #1235, separately from
   the allocation bit, which is (1) **and** (2).
3. **`offsets_a` is a byte-exact PREFIX of `offsets_b`** — #908's own example,
   `[96]` inside `[96, 4]`, which the sums 96 and 100 cannot state.
4. The six GRID Z rows of `work/w-self2b/roots.out` as a **table in the port's
   own types**, so the decoded fact is pinned by a compiled test rather than by
   a committed `.out` file.

### 1.2 `eat_offset_adds` — registered in advance

`eat_offset_adds`'s **signature and behaviour do not change**. A peer lane reads
it. The list is exposed by a sibling that shares the *same walk* — the module's
own stated purpose is "one fact, one locator", and a second copy of a summing
loop is a second place for the overflow check, the `28` payload rule and the
stop condition to drift.

If this turns out to require changing `eat_offset_adds` itself, the rung says so
in its headline.

### 1.3 What this lane will NOT do

* Not change `IlOp::BoundAddr { tok, base, off }`. Its `off` is a **sum**, and
  the seam that would carry the list is that variant — which is matched and
  constructed in `codegen/leaf/store.rs` and `codegen/store_run_call.rs`, the
  **call-tail emitter path concurrent lane `w-mrslot` owns**. So `offsets`
  arrives `None` from today's emitter, and that becomes **one named, countable
  gap** rather than an unmeasurable one. Registered here so it cannot be
  presented afterwards as a completed carrier.
* Not add any allocation statement. `allocate` will not read `roots`, and a
  test asserts it: two producer lists differing **only** in `roots` allocate
  identically.

---

## 2. The fenced rule, and why testing it is second

`w-self2b` published a rule that is **0 wrong on GRID Z's 72** —
`work/w-self2b/rivals.out`, printed under a header saying it has no standing,
**not proposed**:

```text
    H-2Z   the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d
             d = 1 when  the STORE designator's root token is a BIND
                   AND   it differs from the VALUE expression's root token
                   AND   ru >= 2
           DOMAIN: two producers, one an address that is a PREFIX of (or equal
           to) every address it is stored into, the other one `li`.
```

**That fencing is correct and this lane respects it.** A rule read off the cells
it is scored on is not evidence: `H-2X` fit **97 distinct cells across three
grids** and went 12 wrong on a fresh one; `RULE W2` was 388 of 388; `RULE BIND`
33 of 33. H-2Z has **three conjuncts, two of them read off GRID Z**.

So: the carrier is built and reported **on its own**. Only then is H-2Z tested,
**once**, on a genuinely fresh frozen grid over the population `w-self2b` named.

### 2.1 Registered in advance: what GRID P CANNOT say

`w-self2b` named three readings of the `ru = 1` collapse — a `ru >= 2` guard, a
`cu <= 2·ru` cap, and a requirement that the address be live across two of its
own stores. **GRID P does not separate them, and this lane will not claim it
does.** Worked out before the freeze:

```text
    guard   cu <= ru+1+[asym and ru>=2]   ru=1:2  2:4  3:5  4:6  5:7
    cap     cu <= min(ru+2, 2*ru) if asym  ru=1:2  2:4  3:5  4:6  5:7
```

They are **identical at every `ru >= 1`**, because `min(ru+2, 2ru) = ru+2`
exactly when `ru >= 2`. `rivals.out` proposed `ru` 4–5 at `cu = ru+2` against
`cu = 2·ru` as the separator; that arithmetic does not hold, and it is recorded
here as a **correction to #1229's separator list**, made before the grid was
frozen rather than after. The third reading coincides with the guard on every
in-domain family of this grid, where the address's uses *are* its own stores.

What GRID P **does** test is the three readings **jointly**, against the record,
on cells none of them has seen.

---

## 3. GRID P

**Fresh layout.** No struct name, member name, offset, formal name or literal of
w-spell's GRID S/H, w-ilx's GRID V/X, w-mixed's GRID M or w-self2b's GRID Z
survives.

```text
    w-spell  S/L/M   s/t/u/v  f0..fF  inner  in1/in2  a0..a7  32/40/96
    w-ilx    S/L/M   s/t      p0..p9  mid    in1/in2  a0..a7  40/96/128
    w-mixed  T/P/Q   t/r/x    c0..c9  mid    lo/hi    b0..b5  0/40/64/88
    w-self2b D/V/W   d/g/a    e0..eb  core   u0/u1    m0..m5  0/48/72/256/304
    HERE     H/G/F   h/i/b    c0..cb  blk    s0/s1    n0..n5  0/76/100/124
```

### 3.1 The classes — ENUMERATED before the freeze, and the generator ASSERTS them

This is the defect `w-mixed` self-reported: its generator **confounded `LOAD`
with `SELF-2B`**, and *"had the structural variants not been there, `H-MIX`
would have looked clean on a grid containing no cell of the class it is wrong
on."* `gridp.py --freeze` prints its classes and **fails** if any required class
is absent, and **fails** if any scored rival is indistinguishable from H-2Z on
the grid **unless that rival is declared a TWIN** (§2.1).

| fam | binds | store designator | value expr | class |
|---|---|---|---|---|
| `P1` | — | `blk.s0.n%d` | `(int)&blk.s0` | `SELF-1B` |
| `P2` | `F& k = blk.s0;` | `k.n%d` | `(int)&k` | `LOAD` |
| `P3` | `F& k = blk.s0;` | `k.n%d` | `(int)&blk.s0` | `SELF-2B-tail-agrees` |
| `P4` | `F& k = blk.s0;` | `k.n%d` | `(int)&blk` | `SELF-2B-tail-differs` |
| `P5` | `F& k = blk.s0;` | `blk.s0.n%d` | `(int)&k` | `MIRROR` |
| `P6` | `F& k = blk.s0; F& m = blk.s0;` | `m.n%d` | `(int)&k` | `TWOBIND` |
| `P7` | `F& k = blk.s0; F& m = k;` | `m.n%d` | `(int)&k` | **`CHAINBIND`** |
| `P8` | `F& k = blk.s0; F& m = blk.s0;` | `k.n%d` | `(int)&m` | **`TWOBIND-swapped`** |
| `P9` | `F* const p = &blk.s0;` | `p->n%d` | `(int)&blk.s0` | **`PTRBIND`** |
| `X1` | — | `blk.s0.n%d` | `(int)&blk.s1` | `CROSS-path` (OOD) |
| `X2` | `F& k = blk.s0;` | `k.n%d` | `(int)&blk.s1` | `CROSS-bind` (OOD) |
| `X3` | `F& k = blk.s0;` | `k.n%d` | `(int)&i->blk.s0` | `OTHEROBJ` (OOD) |

`blk` is `h->blk`. The six classes the brief requires — `SELF-1B`, `LOAD`,
`SELF-2B` tail-agrees, `SELF-2B` tail-differs, `MIRROR`, `TWOBIND` — are `P1`
through `P6`, and **three classes no lane has compiled** are `P7`, `P8`, `P9`.

**`P7`, `P8` and `P9` are the whole experiment**, exactly as `Z5`/`Z6` were
w-self2b's. `w-self2b`'s own closing lesson: *"the grid must contain a class the
hypothesis has never seen."*

### 3.2 The points — the population GRID Z could not reach

```text
    (1,2)                        the cell rivals.out NAMED. GRID Z has (1,1)
                                 and (1,3) and NOT (1,2), so the ru=1 frontier
                                 is unpinned between cu<=1 and cu<=2.
    (1,3)                        separates H-2Y from H-2Z (they agree at every
                                 other point of this grid) — and reproduces
                                 GRID Z's deciding ru=1 cell on fresh names.
    (2,4)                        the deciding band, on fresh names and for the
                                 three NEW classes' first placement.
    (4,5) (4,6) (4,7)            ru = 4  } EVERY rule on record was fitted and
    (5,6) (5,7) (5,8)            ru = 5  } graded at ru <= 3, in every lane.
```

9 in-domain families × 9 points = **81**, plus 3 controls × 3 points
(`(1,1)`, `(1,2)`, `(4,6)`) = **9**. **90 cells.**

**Deliberately NOT re-tested** — measured and free, per the brief: the bind's
own displacement (48→304, 18 of 18), the depth of the value's path, tail
agreement (`P3`/`P4` are carried because the brief requires both classes
present, and their agreement is scored as a REPRODUCTION, not as a new
measurement), and body length (ruled out).

### 3.3 The scoring assumption `P7` and `P9` force, declared now

Every prediction must be a function of the **cell spec alone** — `--freeze`
compiles nothing and captures no IL. `P7` (a bind whose base is another bind)
and `P9` (a `const` pointer rather than a reference) both require the generator
to commit, in advance, to **how the IL roots them**. It commits to board
#1128 / w-carrier's `IlOp::BoundAddr { tok }`:

> **a bind's root token is its OWN token, never the thing it hangs off.**

So the generator scores `P7` as roots-DIFFER (`m` vs `k`) and `P9` as
roots-DIFFER (`p` vs the formal `h`), both with a bind store root — i.e. the
`SELF-2B` frontier. **That assumption is itself under test**, and the lane
decodes the `.ex` of one representative per family to say which way it went.

---

## 4. Predictions

Committed with `GRIDP.sha256` and `pred.tsv`, **before one cell is compiled**.

| # | registered |
|---|---|
| **P0** | 90 cells reached, 90 graded, **0 OOR, 0 compile-failed**. `ru = 5, cu = 8` is 13 stores and past every population on record; if it leaves the regime the instrument returns an OOR **counter**, never a verdict. |
| **P1** | **THE DIRECTION OF LOSS, registered as the expected outcome: `H-2Z` MISSES at least one cell of GRID P.** The base rate is nine for nine. |
| **P2** | **The headline direction: `P7` (`CHAINBIND`) sits with `LOAD`/`SELF-1B` on the `cu <= ru+1` frontier, NOT with `TWOBIND`.** `F& m = k;` binds a name to a name; if c2 roots `m`'s designator at `k` rather than at `m`, the roots do not differ and the bonus does not attach — and the generator, which scored it roots-DIFFER by §3.3, is wrong on it. This is the cell that tests whether the predicate is about the **decoded root** or about the source spelling. |
| **P3** | `P8` (`TWOBIND-swapped`) agrees with `P6` (`TWOBIND`) at every one of the 9 points. Declaration order of the two binds does not enter the answer. A disagreement would mean **no rule on record contains the deciding term**. |
| **P4** | `P9` (`PTRBIND`) sits with `SELF-2B` — a `const` pointer bind is the same carrier as a reference bind (#1128). |
| **P5** | The `ru = 4` and `ru = 5` bands do **not** simply extend the fitted `+1`/`+2` frontiers: at least one in-domain family answers `const` at `cu = ru+1`, or `prod` at `cu = ru+3`, somewhere in those six points. Registered as the secondary direction of loss — every rule on record was fitted at `ru <= 3`. |
| **P6** | The controls `X1`/`X2`/`X3` **discriminate** at `(1,2)`: `const` where the in-domain families are `prod`. #1223's repair, taken at the new point rather than inherited. |
| **P7** | At `(1,2)` all nine in-domain families answer the same thing. If any family splits there, the `ru = 1` collapse is family-dependent and all three readings of §2.1 die together. |
| **P8** | **The carrier reproduces `work/w-self2b/roots.out`'s six rows** in the port's own types, by a compiled test, and `allocate` returns an identical assignment for two producer lists differing only in `roots`. |
| **P9** | `eat_offset_adds`'s own tests pass **unchanged**, and the new sibling reports `[96]` as a byte-exact prefix of `[96, 4]` where the sums (96, 100) say nothing. |
| **P10** | TU match 10, `mismatch` 0, `codegen-gap` 0, `fnbyte-exact` 36,212 unshrunk, `fnbyte-differs` 2,111 ungrown, `fnbyte-reloc-differs` 861 unmoved, `match-tu-differs`/`-reloc-differs` 0, both sweep fragments 97/1,479 and 302/968 **per case** at both ends, peer key families 0 vanished. |
| **P11** | **No allocation statement is added to `crates/`.** Checked mechanically against the unified diff, not asserted. |

---

## 5. Decline floors — registered AGAINST THE INCUMBENT REFUSAL

The incumbent is a refusal. **It is wrong on 0 of GRID Z's 72 and will be wrong
on 0 of GRID P's 90.** A rule that is 90 % right is strictly *worse*, because
the project's one correctness rule forbids wrong emits and tolerates
incompleteness.

* **F-1.** If the frozen holdout shows **any** wrong emit by H-2Z, the answer is
  the refusal and this lane writes the **tenth** graveyard entry in
  `codegen/alloc.rs`. That is a successful lane.
* **F-2.** **Even at 0 wrong, this lane does not ship H-2Z.** Three conjuncts,
  two read off the grid that scored it; GRID P cannot separate it from its two
  twins (§2.1); and #1175 — a rule whose gate cannot be shown to fire on a
  compiled cell does not ship, which `w-mixed` §5 already established for this
  gate (`leaf/store.rs` builds every `Producer` with `ProducerKind::Constant`
  hard-coded, so no register-derived producer reaches `allocate` at all).
  Registered now so the lane cannot retreat into "it survived" after the grade.
* **F-3.** If the carrier cannot be populated from the real emitter path without
  changing `IlOp::BoundAddr` or `eat_offset_adds`'s behaviour, the carrier ships
  with the unpopulated half **named and counted** and the rung says so in its
  headline. A half-carrier reported as a whole one is the failure mode this
  floor exists to prevent.
* **F-4.** If any alarm moves — `mismatch`, `fnbyte-exact`, `fnbyte-differs`,
  `fnbyte-reloc-differs`, `match-tu-differs` — the lane stops and reports it,
  whatever it costs the rung.

## 6. Instrument rules, inherited and restated

* One directory per cell (#1045); every cell compiled at ONE shared path so no
  directory or file name lands in the obj.
* The workload's own `/GR /O1 /Oi /EHsc` (#1112), read from `flags.txt` by
  `work/w-frame/refobj.sh`, never transcribed.
* The producer's register is read off **its own store's displacement**; no regex
  ever names a source register (w-refbind's OOR bug), and `observe` returns a
  **counter** rather than a verdict when it matches nothing (w-ilx's grader came
  back `OOR` on all 45 cells of its first run, which is the only reason that run
  was not published as a result).
* A moved `sha256` at `--grade` is a **hard error**, never a re-freeze.
* **`grep` cannot test for NUL bytes** (#1236): `grep -c $'\0'` counts *lines*
  and `LC_ALL=C grep -q -P '\x00'` does not fire at all. Byte counts only. And
  **size is not integrity** — a rewrite can punch a NUL hole without changing
  length. Any artefact still held open by a writer (`/proc/*/fd`) is **not
  patched**; the repair is a clean re-run.
* Both sweep fragments at **both** ends, tallied **per case**, on a harness
  built in this tree (#1205).

## 7. Already run before this file was committed, and it measures nothing

`work/w-prod/smoke.sh` compiled **one** cell — a `LOAD` shape at `(2,3)`, a
point GRID Z already carries — solely to prove this worktree can reach `cl.exe`
and `wibo` at all before a 90-cell grid was designed around them. It is a
reachability check, it is not part of GRID P, it is not scored, and its source
is not in the manifest. Recorded here rather than left silent.
