# PREREG — lane `w-mixkind`

Frozen **before** the first probe obj and **before** the first line under
`crates/`. Base master `503f8937`; lane branch `wt-w-mixkind`.

---

## 0. What this lane owns, and what it therefore cannot convert

The brief names `src/xdk/nuispeech/xboxheap.cpp`'s ladder as **two rungs** and
gives this lane the first: the reader key
`store-run-bind-mixed-kind-alloc` (`crates/c2-il/src/func/body/mod.rs:1031`) and
`crates/c2-core/src/codegen/alloc.rs`.

**Registered before any measurement: this lane will not lift the reader key,
whatever this grid says**, and the reason is structural rather than a judgement
about the rule.

* `w-mrslot` §5 lifted the clause behind an uncommitted env hatch and measured
  what happens: the reader accepts, `PortC2` still refuses, and the scan prints
  **`census/gate disagreement: 1`**. That is the invariant
  `codegen/select.rs::function_gate` exists to hold, and the whole reason
  `w-seam2` §6 moved two gates *out* of the emitter.
* What is needed underneath is `parse_simple_gpr_run` admitting a **bound
  VALUE** (`w-mrslot` §5.1 — `value_bound` is a backstop with no reachable
  input, not the wall). `crates/c2-core/src/codegen/leaf/store.rs` is lane
  **`w-midrun`**'s file this week.

So this lane's deliverable is a **measurement of the allocation question**, and
its ship-or-decline is about `alloc.rs` alone. That is registered here so a
reader of the rung cannot mistake "did not convert" for "ran out of time".

**And a second fact that bounds the blast radius, read off the code rather than
hoped for:** `alloc::allocate`'s mixed-kind arm is **unreachable from today's
emitter** — `codegen/leaf/store.rs` builds every `Producer` with
`ProducerKind::Constant`, hard-coded (`alloc.rs` module doc, last paragraph of
the H-2X section). Any change this lane makes to `allocate` moves **zero
bytes**. That makes the grid the entire content of the rung, and it also means a
0-wrong grid buys no warrant it would not have bought as prose.

---

## 1. The question

Ten allocation keys have died in this seam (#836, #857, #868, #892, #912,
#1067, #1134, #1217, #1227, #1243). `w-self2b` decoded **why** they died:

> the fact is a **relation between two `B9` roots plus one bit about one of
> them**, and `Producer` carries `uses` / `kind` / `first`, which are facts
> about *one* producer.

`w-prod` then closed that representation gap — `alloc::Root`,
`alloc::ProducerRoots`, `Producer::roots` — and, in doing so, produced the one
result that says the *widened* carrier is still not wide enough:

> **Board #1244.** `P6-r2k4` (TWOBIND) and `P7-r2k4` (CHAINBIND) decode
> identically — `lvalue tok 0x150a BIND [0]`, `value tok 0x140a BIND []`, every
> field of `w-self2b`'s named carrier equal on both sides — and real `c2` gives
> them **different registers**. The bind table separates them:
> `P6: 0x150a -> base 0x0f0a [76,0]` (the formal's path);
> `P7: 0x150a -> base 0x140a []` (the OTHER BIND).

`Root::base` now carries that. **The question this lane asks is whether the fact
is statable over it.**

---

## 2. THE RULE UNDER TEST — `H-CHAIN`

```text
  H-CHAIN   the address producer takes POOL_TOP (r11)  iff  cu <= ru + 1 + d

    ru = the number of stores consuming the address
    cu = the number of stores consuming the literal

    d = 1 when   the STORE designator's root is a BIND HEAD
           AND   ru >= 2
           AND   the VALUE expression's root token does not appear on the
                 store root's BIND CHAIN — the store root's own token, and
                 each successive `Root::base` that is ITSELF a bind head,
                 walked to the first non-bind base.

  DOMAIN: two producers, one an address that is a PREFIX of (or equal to) every
  address it is stored into, the other one `li`.
```

At chain depth ≤ 1 `H-CHAIN` **coincides with `H-2Z`** everywhere except
`CHAINBIND`, where `H-2Z` is wrong on all three of its GRID P cells. Its claim is
therefore exactly two things:

1. `H-2Z`'s three misses are *precisely* the chain-membership cases; and
2. the correction **generalises to deeper chains** — a claim no cell anywhere
   tests, which is why GRID X carries depth-3 families.

### 2.1 What `H-CHAIN` is NOT

It is **not** `lvalue.base == value.tok`. Board #1244 forbids that reading by
name — *"one witness, read after the grade, which is exactly how the ten keys
above were written"* — and this prereg agrees with the prohibition rather than
routing around it. `H-STEP` below **is** that reading, is scored as a **rival**,
and GRID X's `M6` is the cell that separates the two. If `M6` comes back `prod`,
the one-step reading is right and `H-CHAIN` is over-general; that outcome is
registered in §5 as a loss.

It is also **not** narrowed around any failing cell. `w-mixed`'s standing
prohibition (*"a successor may not narrow around `SELF-2B`"*) applies, and the
chain walk is a **generalisation** of `H-2Z` — it makes `d` smaller, never
larger, on the classes `H-2Z` has seen, and it makes a prediction on five
classes nobody has compiled.

---

## 3. THE RIVALS — every one separated by at least one family

All are functions of the **cell spec alone** and are written into the frozen
column by `--freeze`, which compiles nothing.

| rule | statement | separated at |
|---|---|---|
| **refusal** | the SHIPPED answer. Never wrong, never right. **THE INCUMBENT.** | — |
| **H-STEP** | #1244's forbidden one-step reading: `d=1` iff store root is a bind ∧ `ru>=2` ∧ *not* (the store root's own `base` is a bind head equal to the value root) | **M6** |
| **H-DERIV** | `d=1` iff store root is a bind ∧ `ru>=2` ∧ the value root is neither a bind-chain **ancestor NOR a descendant** of the store root (the SYMMETRIC closure) | **M9** |
| **H-DEPTH** | `d=1` iff store root is a bind ∧ roots differ ∧ `ru>=2` ∧ the store root's own base is **not** a bind (chain depth exactly 1) | **M8** |
| **H-2Z** | board #1243, key ten. `d=1` iff store root is a bind ∧ roots differ ∧ `ru>=2`. 3 wrong of 81 on GRID P | **M5, M6, M7** |
| **H-2X** | board #1227, key nine. `d=1` iff the roots differ. Symmetric | M9, M1… |
| `cu<=ru+1` | board #892/#1219. Best score anywhere 60 of 62 | M3, M4, … |
| `cu<=ru+2` | board #1221's clause. Refuted at `ru=1` (#1229) | M1, M2, … |
| `always-prod` | `w-heap` §4.1.1's reading. 44 wrong of 62 on GRID M | — |
| `clause-1` | the shipped ALLOC clause 1 alone, use count descending | — |

`--freeze` **asserts** that no scored rival is indistinguishable from `H-CHAIN`
on the frozen column, and writes nothing if one is. There are no declared twins
this time; if the assertion fires, the grid is wrong and is not graded.

---

## 4. THE DECLINE FLOOR — named against the incumbent

**The incumbent is not a threshold, it is a refusal, and it is wrong on 0.**

| population | the refusal | the best rule ever measured on it |
|---|---:|---|
| #836's 81 mixed cells | **0 wrong** | w-next's key, 20 wrong |
| GRID M, 62 in domain | **0 wrong** | `cu<=ru+1`, 2 wrong |
| GRID Z, 72 in domain | **0 wrong** | H-2X, 12 wrong |
| GRID P, 81 in domain | **0 wrong** | H-2Z, 3 wrong |

So the floor is stated as a **count against a rule that is never wrong**, not as
an accuracy:

* **F-1 — DECLINE.** If `H-CHAIN` is **≥ 1 wrong** on any in-domain cell of
  GRID X, it is the **eleventh death**. It is written into `alloc.rs`'s
  graveyard with its count and its separating cell, a board row is minted, and
  `allocate` is **not touched**. A rule that is 95 % right is *worse* than a
  refusal that is never wrong: `mismatch 0` is this project's only correctness
  criterion and a wrong emit is an alarm outranking all other work.
* **F-2 — 0 WRONG IS STILL NOT A SHIP.** If `H-CHAIN` is **0 wrong** in domain,
  it is published in `alloc.rs` **under a header saying it has no standing**, and
  `allocate` **still refuses**. `RULE W2` was 388 of 388; `RULE BIND` 33 of 33;
  `H-2X` fit 97 distinct cells across three grids. One grid is not a warrant, and
  — per §0 — a shipped decision would move zero bytes anyway while creating
  exactly the pattern that killed ten keys. What a 0-wrong result buys is a
  **named successor experiment**, not an emit.
* **F-3 — THE READER KEY IS NOT LIFTED**, under any outcome (§0).
* **F-4 — `allocate_ignores_the_roots_carrier` stays green.** `w-prod` shipped
  that test to hold the carrier/decision separation mechanically. This lane does
  not weaken it. If it must change, that is a ship and F-2 forbids it.

---

## 5. THE DIRECTION I EXPECT TO LOSE IN — registered before the grid exists

**Primary: `M9` (REVERSE).** `F& k = h->blk.s0; F& m = k;` — store through the
**shallower** bind `k`, value `(int)&m`. `H-CHAIN` walks only the *store root's*
chain, and `k`'s chain is `{k}` with a non-bind base, so `m` is not on it and
`H-CHAIN` says the bonus fires. `H-DERIV` says it does not, because `m` is
derived from `k`. **If `c2` answers `const` at `M9-r2k4`, `H-CHAIN` dies exactly
the way `H-2X` died on `MIRROR`** — by being wrong about the *direction* of an
asymmetric relation — and the surviving statement is symmetric.

This is the honest reading of the record: the one thing five lanes have
established about this seam is that its asymmetries do not point the way anyone
guesses. `H-2X` was symmetric and `MIRROR` killed it; `H-2Z` made it asymmetric
and `CHAINBIND` killed it; `H-CHAIN` keeps the asymmetry and adds a walk.

**Secondary: `M6` (DEEP-GP) at `(2,4)`.** If `c2` answers `prod`, `H-STEP` —
the one-step reading #1244 forbids anyone from *calling a rule* — is what the
bytes support, and `H-CHAIN`'s walk is over-general. That is a loss for
`H-CHAIN` and it is **not** a licence to ship `H-STEP`: it would then be a rule
with one conjunct read off a witness pair and one grid behind it, which is F-2's
case verbatim.

**Tertiary: the whole depth axis may not exist in the IL.** §6.3.

---

## 6. THE GRID — GRID X

Structural axes first, crossed; values vary inside a cell. **Values feel
thorough and discriminate least** — every one of the ten dead keys had value
variation and no structural variation on the axis that killed it.

### 6.1 Axis A — the STORE root's shape

| level | spelling |
|---|---|
| depth 0 | not a bind — a formal path |
| depth 1 | a bind whose base is a formal path |
| depth 1′ | a **second, independent** bind on the same formal path |
| depth 2 | a bind whose base is a bind |
| depth 3 | a bind whose base is a bind whose base is a bind |

### 6.2 Axis B — where the VALUE's root sits relative to that chain

`self` · `parent` · `grandparent` · `formal` (the ultimate non-bind base) ·
`sibling` (an off-chain bind aliasing the same object) · `child` (a bind
**below** the store root — the REVERSE direction).

### 6.3 The eleven realisable families

`A × B` is not full — `parent` needs depth ≥ 2 and `grandparent` depth ≥ 3.

| fam | class | binds | stores through | value | depth | rel |
|---|---|---|---|---|---:|---|
| `M1` | `SELF-1B` | — | `h->blk.s0.nX` | `&h->blk.s0` | 0 | — |
| `M2` | `LOAD` | `k=blk.s0` | `k.nX` | `&k` | 1 | self |
| `M3` | `SELF-2B` | `k=blk.s0` | `k.nX` | `&h->blk.s0` | 1 | formal |
| `M4` | `TWOBIND` | `k`,`m` both `=blk.s0` | `m.nX` | `&k` | 1′ | sibling |
| `M5` | `CHAINBIND` | `k=blk.s0; m=k` | `m.nX` | `&k` | 2 | parent |
| `M6` | `DEEP-GP` | `k; m=k; p=m` | `p.nX` | `&k` | 3 | grandparent |
| `M7` | `DEEP-PARENT` | `k; m=k; p=m` | `p.nX` | `&m` | 3 | parent |
| `M8` | `CHAIN-PATH` | `k=blk.s0; m=k` | `m.nX` | `&h->blk.s0` | 2 | formal |
| `M9` | `REVERSE` | `k=blk.s0; m=k` | `k.nX` | `&m` | 1 | child |
| `M10` | `DEEP-SELF` | `k; m=k; p=m` | `p.nX` | `&p` | 3 | self |
| `M11` | `CHAIN-SIB` | `k=blk.s0; m=k; j=blk.s0` | `j.nX` | `&m` | 1′ | off-chain |

`M1`–`M5` exist on record (GRID Z / GRID P). **`M6`–`M11` are six classes no
lane has compiled**, and they are the experiment: `w-self2b`'s closing lesson is
that every key here survived every recorded refutation and died on the first
cell of a class nobody had built.

**`M6`, `M7` and `M10` carry a DECLARED assumption about the IL** — that
`F& p = m;` where `F& m = k;` produces a **depth-3 bind chain** in the `.ex`
rather than being flattened by the front end. `--freeze` cannot check it (it
compiles nothing). `--grade` decodes one representative per family through
`w-ilx`'s `exdec.py` and **publishes the chain it actually finds**, and if the
chain is flat those three families are reported as **out of regime, not as
evidence for anything**. That is registered as the tertiary loss direction.

### 6.4 The `(ru, cu)` points

| point | why |
|---|---|
| `(1,3)` | the `ru = 1` collapse — #1229's *"the bonus vanishes at `ru = 1`"*, re-tested on fresh names in six fresh classes |
| `(2,3)` | `cu = ru+1` — every rule on record says `prod`. The control that says the cell compiled to the regime at all |
| `(2,4)` | `cu = ru+2` — **THE deciding point.** Every rival differs from `H-CHAIN` here or nowhere |
| `(2,5)` | `cu = ru+3` — every rule says `const`. The other control |
| `(3,5)` | the deciding band at `ru = 3` — the bonus must not be an artefact of `ru = 2` |

11 families × 5 points = **55 in-domain cells**, plus **3 out-of-domain
CONTROLS** × 2 points = **6**. **61 cells.**

### 6.5 The out-of-domain controls

Declared out of domain at freeze because the value is **not a prefix** of what
it is stored into: `X1` `CROSS-path`, `X2` `CROSS-chain` (a depth-2 store root,
which no lane's cross control has), `X3` `OTHEROBJ`.

### 6.6 Freshness

No struct name, member name, offset, formal name or literal of `w-spell`'s GRID
S/H, `w-ilx`'s V/X, `w-mixed`'s M, `w-self2b`'s Z or `w-prod`'s P survives.
GRID X uses `N`/`R`/`T` · `y`/`z`/`e` · `v0..vb` · `blk` · `s0`/`s1` ·
`n0..n5` · offsets `0`/`80`/`104`/`128`, literal `5`.

### 6.7 The instrument

Taken **verbatim** from `w-prod/gridp.py` because it has already survived one
grade and one OOR bug hunt: the producer's register is read off **its own
store's displacement**, no regex ever names a source register (`w-refbind`'s OOR
bug), and `observe` returns a **counter** rather than a verdict when it matched
nothing (`w-ilx`'s grader came back `OOR prod regs 0` on all 45 cells of its
first run, and that is the only reason that run was not published as a result).

Every cell compiles at **one shared path** so no directory or file name lands in
the obj (#1045); artefacts are copied out per cell afterwards. Flags are the
workload's own, read from `work/dc3-workload/flags.txt` through
`work/w-frame/refobj.sh` rather than transcribed (#1112).

---

## 7. Grids and corpora fail in opposite directions

Both are run, at **both ends**:

* GRID X (this file);
* `scripts/sweep.d/88-store-run-call.py` and `89-store-run-live-arg.py`, with
  the per-case `Match` / `NotImplemented` split (`tally.sh` — board #1205: a
  lane that tallies only at its tip books conversions it did not cause);
* the 878-TU workload scan, the full 139-line `gap-metric` block `diff`ed, and
  `blockers.py`'s `fn_blockers` / `emit_blockers` row diff;
* `work/w-splice/peerkeys.py` at both ends.

**Owned surfaces to re-check at merge even with no textual conflict** (three
lanes collided through semantics with no git conflict in one wave this week):
`ProducerKind`, `Producer`, `alloc::Root`, `alloc::ProducerRoots`, `allocate`,
`all_in`.

---

## 8. Prereg scorecard — to be filled at §8 of the rung, not here

| id | registered before the grid | outcome |
|---|---|---|
| P1 | `H-CHAIN` ≥ 1 wrong in domain → DECLINE (F-1) | |
| P2 | `M9` (REVERSE) is where I expect to lose | |
| P3 | `M6` (DEEP-GP) separates `H-STEP`; a `prod` there refutes the walk | |
| P4 | the depth-3 chain may be flattened by the front end (§6.3) | |
| P5 | 0 wrong still does not ship (F-2) | |
| P6 | the reader key is not lifted (F-3) | |
