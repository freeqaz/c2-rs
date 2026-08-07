# w-mixed — PREREG

    Lane:    w-mixed, worktree branch `wt-w-mixed`, base master `844c3cfd`.
    Rung:    board #868/#836 — `codegen::alloc`'s mixed-kind refusal, the
             refusal the brief calls the LAST one on `src/xdk/nuispeech/xboxheap.cpp`.
    Written: before the first probe obj of this lane existed. Everything cited
             below is read out of files already committed on master.

---

## 0. A correction to the brief, registered before probing

The brief states that #868/#836 is **the only refusal left** on `xboxheap.cpp`
and that its price today is **one**. Read off master, that appears to be
**false**, and the check is four lines of already-committed code:

`crates/c2-il/src/func/body/shapes/leaf_store.rs::bind_run_ops` refuses in this
order —

```text
  1. group shape                     STORE_RUN_BIND_GROUP_SHAPE
  2. if addr_producer { … }          STORE_RUN_BIND_ADDR_PRODUCER / _MIXED_KIND
  3. if has_call { … }               STORE_RUN_BIND_CALL_TAIL
  4. lits.len() > 1                  STORE_RUN_BIND_MULTI_PRODUCER
  5. pool floor
  6. symbol crossings                STORE_RUN_BIND_SYMBOL_CROSSINGS
```

and clause 2's own comment says it is placed **before** clause 3 *"so
`xboxheap.cpp` keeps the key that sizes #836/#868"*. `xboxheap`'s ctor ends in
`AllocatePageBlock(initSize)`, so `has_call` is true and clause 3 is armed
behind clause 2.

> **P0 (the headline, registered as this lane's expected direction of loss for
> the brief rather than for me): lifting the mixed-kind refusal converts
> NOTHING. `k_target` moves from `store-run-bind-mixed-kind-alloc:eof` to
> `store-run-bind-call-tail-mr-slot:eof`, and the price of `xboxheap.cpp` is
> at least 2 and probably 3.**

The third is on the emitter side and is not the allocator:
`crates/c2-core/src/codegen/leaf/store.rs:274` refuses **any** run with
`value_bound`, and below it the producer list is built only from `s.lit`, so an
interior-address producer is invisible to both `order::schedule` and
`alloc::allocate`. Solving the allocation question does not by itself give the
emitter a producer to allocate.

**This is registered as a claim to be settled by a compiled cell, not by
reading.** Board #1175: a gate keyed on the wrong predicate reads as working.

---

## 1. The decline floor

The incumbent control is **today's shipped refusal**, which is wrong on **0** of
every cell any lane has ever graded (0 of 81 in #836, 0 of 77 in w-ilx, 0 of 71
in w-alloc3, 0 of 45 in w-spell's GRID H). The port's correctness rule forbids
wrong emits and tolerates incompleteness.

**F-1. If the frozen holdout shows ANY wrong emit, the answer is the refusal
that already ships, and this lane writes the eighth graveyard entry.** No
narrowing around the failing cells, no successor fitted on them (w-ilx's own
standing instruction, taken as binding here).

**F-2. A rule that survives on a holdout too small to decide ships as nothing**
and is specified for a larger grid this lane names.

**F-3. This lane does not ship a `crates/` widening whose gate cannot be shown
to fire on a compiled cell.**

---

## 2. The hypothesis — H-MIX

> **The producer takes `POOL_TOP` iff `cu <= u + 1`, where `cu` is the number of
> stores consuming the constant and `u` is the number of times the producer's
> register is READ — its value uses plus one more if that same register also
> serves as a store BASE.**
>
> Domain: a run with exactly two distinct producers, one of them an **interior
> address** (`&s->inner`, one `addi rD,rBase,off`) and the other a single-word
> `li` literal. Everything else refuses, as today.

Written as the reader could compute it: `u = ru + b`, where `b = 1` when the
address-valued stores' base token is a **bound reference distinct from the
literal stores' base token** (w-spell's `2base`; w-refbind R8's displacement-0
bind is not one) and `b = 0` otherwise (`1base`).

### 2.1 What is new here, stated so it can be attacked

Every prior key counted a producer's `uses` as **stores that consume its
value**. H-MIX counts **reads of its register**, and in the `2base` spelling the
address register is read once more than that, because it is also the base of the
stores that consume it. The `+1` that six lanes have modelled as a *kind bonus*
is, under H-MIX, not a bonus at all — it is a read.

### 2.2 Standing on the record — 41 cells, 0 wrong, and it is NOT evidence

Scored post hoc against three lanes' committed tables, with no compile:

| population | cells | H-MIX wrong |
|---|---:|---:|
| w-ilx GRID S `self` (`fit.out`) | 10 | 0 |
| w-ilx GRID X `A` + `E` (`fit.out`) | 6 | 0 |
| w-ilx GRID V `V1`,`V2`,`V7` (`holdout_grade.out`, frozen) | 15 | 0 |
| w-spell GRID H `H2-self-2base`, `H3-self-1base` (`holdout_grade.out`, frozen) | 10 | 0 |

It repairs, with no free parameter beyond `b`, **every** miss those tables
record inside its own domain: `RULE W2`'s two `self` misses at `(2,4)` and
`(3,5)` (board #891), and `KEY ILX`'s single `SELF-2B` miss at `(2,5)`.

**This is exactly the standing `RULE W2` had at 388 of 388 and `RULE BIND` had
at 33 of 33, which is none** (#912). It is registered here as a *prior*, not as
a result, and the `b` term was read off those same cells, so **not one of the 41
counts as evidence for it**. The lane therefore **fits nothing**: H-MIX is
frozen as stated above and graded **once**, on a grid no lane has compiled.

### 2.3 Why H-MIX is not a restatement of any of the seven dead keys

| dead key | died | why H-MIX is not it |
|---|---|---|
| `w-next` (#836) — `uses + (regderived?1:0)` desc | 7 wrong / 56 | Its 7 killers are `add` / `addi`-arithmetic / `slwi` cells — **computed values, not addresses** — and every one is outside H-MIX's domain, which the reader's own vocabulary (`IlOp::AddrOf` / `BoundAddr` only) enforces. It also differs *inside* the domain: w-next's `+1` is a kind bonus paid to every register-derived producer at `1base` and `2base` alike; H-MIX pays it only where a second register read exists. |
| `H-self` (#857) — a ~1.5-use bonus for a producer stored into the object it points at | 11 / 72 | H-self died on its **negative** side (`extsh`, `lwz` take the bonus register at 1-vs-1 where H-self forbids it) — both spellings unrepresentable here. And `b` is not self-ness: `1base` cells *are* self and get `b = 0`. |
| `clause-1-strict` (#868) — clause 1 where it decides with no tie | 12 / 36 | H-MIX abandons clause 1 entirely on the mix. Clause 1 is refuted on this exact axis by `j1_lit2` (#1134, address 1 use vs literal 2, address still takes `r11`); H-MIX predicts `j1_lit2` correctly (`2 <= 1+0+1`). |
| `RULE W` (#886) — spelling classes | 7 / 388 | H-MIX reads no spelling taxonomy. Its one structural predicate, `b`, is a **base-token identity** the reader already computes; the spelling axis is closed by the domain gate rather than modelled. |
| `RULE W2` (#887) — RULE W + `2ru+3 > 2cu` | 14 / 106 | `2ru+3 > 2cu` is, over the integers, exactly `cu <= ru+1` — so RULE W2 **contains H-MIX's `b = 0` clause**, and this must be said plainly. It died on three families: `add addi srawi` at high `cu` (outside the domain), `subfic` misclassified (outside), and **`self` at `(2,4)`/`(3,5)`, which are inside and are precisely the two cells `b = 1` repairs.** H-MIX is therefore RULE W2's surviving clause plus one term, on one third of its domain. Registered as the sharpest reason this lane may be about to die. |
| `KEY ILX` (#909) — IL-field classes | 14 / 45 | Its `LOAD` (5 misses) and `CROSS` (8 misses) classes are **both outside H-MIX's domain**; its `SELF-1B` clause is `cu <= ru+1` and went 10/10; its `SELF-2B` clause is *"always wins"* and is the single remaining miss. H-MIX replaces "always wins" with `b = 1`. So H-MIX is a **narrowing of the sixth dead key plus one repair**, fitted on its refutation cell — the thing w-ilx's own standing instruction forbids as a *shipping* route, which is why §2.2 buys it nothing and only a frozen grid can. |
| `RULE BIND` (#1067) — a field edit | 5 / 38 | Different question entirely (renaming an inlined callee's operands). H-MIX reads no callee. |

---

## 3. What the grid holds constant, and what it varies

`w-carrier`'s own 53-cell frozen grid was green through four wrong emits the
sweep caught; `w-seam2` found #866 false in general. So:

**Varied (structural):** base mode `1base`/`2base`; `(ru, cu)` over the whole
boundary band `cu ∈ {ru−1 … ru+3}` **and** board #912's named killing
population `cu ∈ {6,7,8}` at `ru ∈ {2,3}`; the address's target (self-prefix,
sibling sub-object, and a second bind); store width; displacement magnitude
(in-range and past the `addi` simple form); source position of the address
stores relative to the literal stores; formal count; and **which literal value**
(so equal-`k` CSE cannot be the whole signal).

**Held constant and named as such:** there is no trailing call in any cell (the
allocation is being measured, not #1189's liveness schedule), and every cell has
exactly one literal producer (two would be three producers, past
`MAX_MODELLED_PRODUCERS`).

**Stratified:** every row prints its **store count** and its **emitted
instruction count** beside its verdict, so a discrimination that is really body
length is visible (the `/QXSTALLS` lesson, +76.25 pp that was entirely length).

A **directory per cell** (#1045). Every cell at the workload's own
**`/GR /O1 /Oi /EHsc`** (#1112), never the harness `/Ox`. The register is read
off the producer's **own store's displacement**, never off a source-register
regex (w-refbind's OOR bug).

The manifest `GRIDM.sha256` and the full prediction table `pred.tsv` are
committed **before the first cell is compiled**, and the grader re-checks every
hash and fails hard on a moved one.

---

## 4. Registered predictions

| # | prediction | scored on |
|---|---|---|
| **P0** | **Lifting the mixed-kind refusal converts nothing** — `k_target` lands on `store-run-bind-call-tail-mr-slot:eof` and `xboxheap` prices at ≥ 2. Registered as the headline and as a correction to the brief | a compiled cell |
| **P1** | **H-MIX misses at least one cell of GRID M.** Registered as the *expected* outcome — seven for seven | GRID M |
| **P2** | If it misses, the misses concentrate at **`cu = u+2`**, the first cell past the boundary, and **not** at #912's high-`cu` population, where every rival on record already agrees on `const` | GRID M |
| **P3** | The `b` term survives at `1base` and dies at `2base`, because `2base` is the only place `b` does any work and its three supporting cells are the three it was fitted on | GRID M |
| **P4** | `cu <= ru+1` **alone** (board #892, the best-scoring rule on record and the one #912 asks for a grid for) is **wrong on GRID M**, at `2base` in the band `cu ∈ {ru+2}`. This lane discharges #912 as a by-product whichever way H-MIX goes | GRID M |
| **P5** | `always-prod` — w-heap §4.1.1's *"the interior address takes the top of the pool, whatever the use counts are"*, the reading a lane that looked only at `xboxheap` would ship — is **wrong on GRID M**, and is already wrong on the record at `H2-self-2base-r2k5` | GRID M + the record |
| **P6** | `xboxheap.cpp` does **not** convert in this lane and TU match stays **10**. Registered so a null result is a scored prediction and not a disappointment | the 878-TU scan |
| **P7** | Both sweep fragments and the whole gate are unmoved at the tip, because this lane's `crates/` diff is tests only | `88`/`89`, `gate.sh --require-graded` |

**The direction I expect to lose on:** P3. If `b` is a coincidence of three
cells, H-MIX collapses to `cu <= ru+1`, which is board #892 and already has 10
wrong of 77 on record — and the lane's answer is the shipped refusal.

---

## 5. Ownership

`crates/c2-core/` is this lane's. `work/w-splice/peerkeys.py` is run at both
ends regardless.
