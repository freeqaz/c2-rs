# w-ilx — PREREG

    Lane:    w-ilx
    Branch:  w-ilx, worktree off master `8521606`
    Date:    2026-08-06
    Board:   rows reserved #906–#915

**Committed before any probe in this lane exists.** Nothing under
`work/w-ilx/` other than this file is present at this commit; `git show --stat`
on this commit is the proof.

---

## 0. The mission, stated so it can lose

`codegen::alloc`'s foundation is *"the distinction is read off the IL, never off
the answer"*. Five lanes have now fitted an allocation key on the **answer**
(the emitted register) and every one has died on fresh cells:

| key | died on |
|---|---|
| w-next's `uses + (register-derived ? 1 : 0)` | 7 of 56 (w-alloc2) |
| `H-self` | 11 of 72 (w-refbind) |
| clause-1-strict | 12 of 36 (w-seam) |
| RULE W | 7 of 388 (w-spell, no compile) |
| RULE W2 | 14 of 106 (w-spell holdout) |

This lane does **not** fit a sixth. It asks the prior question: **is the thing
that decides #891 and GRID S's `self`/`cross` row even present in the IL c2
reads?** The IL is c2's only input, so *something* must differ — that half is a
tautology (w-refbind §5 says so explicitly) and **is not registered as a claim**.
What is registered is *where*, *how much*, and *whether the difference is
decodable as one named fact*.

---

## 1. The instrument, and the failure modes it is built against

1. **Names are IN the IL.** `.sy`/`.gl` carry struct, function and local names
   verbatim, so two cells that differ in a generated tag differ in their IL for
   a reason that has nothing to do with the question. **Every compared pair in
   this lane uses the SAME struct name, the SAME function name, the SAME local
   names and the SAME formals**; only the body text differs. This is
   `work/w-refbind/ilcmp.py`'s stated confound and it is inherited verbatim.
2. **The objs must be compiled from the SAME source text as the IL capture.**
   A byte diff of `.ex` between two sources whose objs were produced by a
   different program is not evidence about those objs. This lane compiles the
   obj and captures the IL from **one** file per cell.
3. **#843** — c2 prints `sub` not `subf`, `slwi` not `rlwinm`, `clrlwi` not
   `andi.`. No grader here matches a per-spelling mnemonic regex; the producer's
   register is read off **its own store's displacement**, and the defining
   mnemonic is recorded as printed.
4. **w-refbind's OOR bug** — no regex anchors a source register absolutely.
   Register numbers are derived per cell from that cell's own stores.
5. **STATUS trap 5** — `selected / reached / graded / out-of-regime /
   compile-failed` are five printed counters, never one status. **STATUS trap
   4** — a total is not a control; every grid below carries at least one row
   that can go red in the most likely failure direction.
6. **#644** — a register defined more than once is OUT OF REGIME, never a hit
   and never a miss.

---

## 2. GRID I — the byte-level minimal pairs

`work/w-ilx/ilx.py`. Each cell is one `.cpp`; the obj is built at the
workload's own `/O1 /Oi /EHsc /GR` through `work/w-frame/refobj.sh`, and the IL
bundle is captured from the same file through `c2rs capture --keep-il`. The
pairs, and what each is for:

| pair | cells | why |
|---|---|---|
| **S-11** | `self` vs `cross`, 1base, (ru 1, cu 1) | the GRID S cell that disagrees. `P` vs `c` |
| **S-21** | `self` vs `cross`, 1base, (2, 1) | **the control** — same spelling difference, and the objs AGREE (`P`,`P`) |
| **X-35** | GRID X `A` (`&s->inner`) vs `B` (`&q`), (3, 5) | #891's deciding cell. `prod` vs `const` |
| **X-11** | GRID X `A` vs `B`, (1, 1) | **the control** — same spelling difference, objs AGREE (`prod`,`prod`) |
| **X-AE** | GRID X `A` vs `E` (`&s->inner`, no bind), (3, 5) | `prod` vs `const` with the same *value* spelling |

### Registered claims

| # | claim | how it loses |
|---|---|---|
| **I1** | the `.sy` streams of **every** pair above are byte-identical | any pair differing in `.sy` |
| **I2** | `S-11`'s `.ex` diff is confined to **one** varint literal — the member offset — and nothing else | more than one differing byte-run, or a differing type tag / token |
| **I3** | `X-35`'s `.ex` diff is **NOT** a single byte: it is a token-sequence difference (a pushed token changes, and an offset-add production is present on one side and absent on the other) | a single-byte diff, as in #856 |
| **I4** | **the control.** `S-21` carries the *same* decoded `.ex` difference as `S-11` — same production, same field — while its objs **agree**. Likewise `X-11` against `X-35` | the controls' `.ex` diffs differing in kind from their deciding pairs' |
| **I5** | **BYTE-IDENTICAL ANYWHERE IS A FIRST-CLASS FINDING.** If any pair whose objs differ has byte-identical `.ex` **and** `.sy` **and** `.gl`, this is reported as the lane's headline and nothing is fitted | — (registered so the outcome cannot be explained away) |

**I4 is the row that decides whether an IL key is even possible.** If the
controls carry the same difference and answer differently, then no predicate
over the IL bytes *alone* can be the rule: the decision is a function of the IL
fact **and** the use counts, and any key must say so. That is registered here
rather than discovered later.

---

## 3. Decoding — GRID D

`work/w-ilx/decode.py`. The separating bytes are decoded against
`docs/IL_STMT_GRAMMAR.md`'s grammar and the constants already in
`crates/c2-il`. **#644 applies to IL**: a fact is not required to be one
contiguous field, and the decode may name a *pair* of positions.

| # | claim | how it loses |
|---|---|---|
| **D1** | every separating byte lands inside a production `crates/c2-il` already names (`B9 <tok> <TYPE>` pointer load, `33 <lit> 27 <PTR>` offset-add, `26 <tok>` symbol push, `32 <TYPE>` assign, `4F 01 <varint>` line marker) | a separating byte in an unnamed region |
| **D2** | the fact is **NOT** one contiguous field — it takes at least two positions to state | one field suffices |
| **D3** | **the negative that makes the grid worth publishing.** No predicate over the producer's value expression *alone* separates GRID S's four groups. Registered as the claim expected to WIN | a single expression-level predicate reproducing G0/G1/G2/G3 |

---

## 4. GRID V — the frozen validation

`work/w-ilx/holdout.py --freeze` writes predictions and a sha256 of every
source **before any cell is compiled**, and the freeze is committed. `--grade`
re-checks every sha256 and reads the frozen column; a moved source is a hard
error, not a re-freeze.

* **at least 20 never-fitted cells graded**, at the workload's own flags;
* the predictions come **from the IL fact alone** — the key is evaluated on the
  captured `.ex`, not on the C++ spelling and not on the obj;
* **the decline floor and the incumbent are registered**, so a win is measured
  against something:

| baseline | prediction | wrong |
|---|---|---|
| **the shipped refusal** (`alloc::allocate` refuses every mixed run) | refuse | **0**, by construction — a refusal is never wrong |
| **the incumbent** | the same refusal; today's emitter cannot even reach it (#840) | 0 |

| # | claim | how it loses |
|---|---|---|
| **V1** | ≥ 20 frozen cells graded, 0 sha256 moved | fewer than 20, or any source moved |
| **V2** | **the IL key misses at least one cell.** Registered as the EXPECTED outcome, given five prior deaths | 0 misses — in which case §5 applies |
| **V3** | the control — ≥ 3 frozen cells are `(1,1)` configurations where every prior grid says `prod`, and the key predicts them | the control silently absent |

**If V2 loses** (the key is 0-miss on ≥ 20 fresh cells) the lane still **ships
nothing to `crates/`**: it writes the spec for reading the fact into
`ProducerKind` / the run model and stops. That is registered here so a good
result cannot become an unplanned emitter change.

---

## 5. What this lane will NOT do

* **Will not ship a `crates/` change.** `git diff <base>..HEAD -- crates/` must
  be **empty** at the end; tests and docs only if anything.
* **Will not fit a successor to RULE W2** on w-spell's miss families. The
  standing instruction after a refutation is binding.
* **Will not promote #865's rival** (#893) — out of scope.
* **Will not read the answer into the key.** If the only way to state the
  separating fact is with reference to which register c2 chose, the lane
  reports that and stops.

## 6. The FBM partition, both ends

`differs 4711` must not grow, `exact 34466` must not shrink,
`fnbyte-match-tu-differs 0`, scan `mismatch 0`. The whole partition line is
printed at both ends, not a summary of it.

## 7. Gate

`scripts/gate.sh --jobs 6` and `cargo test --workspace --release` aggregated to
a printed `targets=/passed=/failed=` line, measured **at this lane's base**
rather than quoted from another rung.

---

# ADDENDUM 1 — 2026-08-06, GRID V, committed before `holdout.py` exists

GRID I is measured (`work/w-ilx/ilx.out`) and KEY ILX is stated and scored on
the prior lanes' own configurations (`work/w-ilx/fit.out`, **32 of 32, and
w-spell's published tables agree on 32 of 32**). That is a **fit** population:
RULE W was 388 of 388 and RULE W2 was 388 of 388 before each died on fresh
cells. This addendum declares the holdout **before the grid file exists**.

## A1.1 KEY ILX, frozen wording

Evaluated on the captured `.ex` **alone** — no obj, no disassembly, no
register. Per producer statement `<lvalue addr> <value addr> 2C <int> 00 32
<TYPE> 4B`, with `eat_addr` = `B9 <tok> <TYPE> ( 33 <int-TYPE> <varint> 27
<PTR> )*` keeping the literal **list**:

* `V = (vtok, vadds)` — the value expression's base token and offset-add
  literals;
* `L = (ltok, ladds)` — the producer store's; if `ltok` is the target of a
  `26 <tok>` temp bind, `L` is resolved through it to `(btok, badds ++ ladds)`;
* `ctok` — the constant run's store base token;
* `ru` = producer stores, `cu` = constant stores.

Ordered clauses:

1. **LOAD** — `vadds` is empty (the value is a bare `B9` pointer load, no
   `33 … 27`). Producer wins iff `cu ≤ 1`.
2. **SELF-2B** — `V` is a *proper prefix* of `L` and `ltok ≠ ctok`. Producer
   always wins.
3. **SELF-1B** — `V` is a proper prefix of `L` and `ltok == ctok`. Producer
   wins iff `cu ≤ ru + 1`.
4. **CROSS** — otherwise. Producer wins iff `ru ≥ 2`.

## A1.2 The grid

Nine producer shapes over a **fresh struct** (`p0..p9` then a nested `M{L in1;
L in2;}` then a trailing `L`) and a **fresh signature** (`void h(S* s, S* t,
int u, int v, int w)` — every operand register moves, w-spell H3's axis), at
five use-count points **four of which no prior grid reached**:
`(1,1) (3,4) (3,5) (2,5) (4,2)`. **45 cells.**

The nine shapes are chosen to make the clauses separable and to attack the
*prefix* formulation specifically: `&s->mid.in2` against stores at
`s->mid.in1.aN` has offset literals `[40, 32]` where the store has `[40, 0,
4N]` — the two chains **share their first element and are not a prefix pair**,
so a reading on `eat_offset_adds`'s SUM, or on "the first literal agrees", gets
it wrong where KEY ILX gets it right, or vice versa. It is the cell this grid
exists for.

## A1.3 Registered

| # | claim | how it loses |
|---|---|---|
| **V1** | ≥ 20 frozen cells graded, 0 sha256 moved | fewer, or any source moved |
| **V2** | **KEY ILX misses at least one cell.** The EXPECTED outcome | 0 misses |
| **V3** | ≥ 3 `(1,1)` controls, and the control **can go red**: KEY ILX predicts `const` for the `CROSS` shapes at `(1,1)` and `prod` for the other seven, so a grid that came back all-`prod` would refute it | the controls all predicting one way |
| **V4** | the `(3,5)`/`(2,5)` `CROSS` cells are the most likely misses — board #892 says the add group's advantage is bounded in `cu` and clause 4 ignores `cu` entirely. Registered **as the predicted failure mode**, before grading | the misses landing somewhere else |
| **V5** | the sha256 covers the `.ex` streams as well as the sources, so a re-capture that produced different IL is a hard error rather than a silent re-freeze | — |

**The decline floor and the incumbent stand unchanged**: the shipped refusal is
wrong on 0 and refuses all 45; today's emitter cannot reach `alloc::allocate`
with a register-derived producer at all (#840). A key that misses **anything**
loses to them, and nothing is proposed for shipping either way.
