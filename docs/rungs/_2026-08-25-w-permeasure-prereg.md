# PREREG — `w-permeasure`: the permuter PRE-MEASUREMENT

    Tag:       w-permeasure
    Date:      2026-08-25
    Kind:      characterization
    Base:      a8593651b (c2-rs, clean)
    Corpus:    ../dc3-decomp @ 15a64d92f1975868e55a1c670d312a8e464074c3, 0 dirty files
    Rows:      #3534–#3538 (reserved at dispatch; minted in the results commit)
    Fixtures:  none — characterization lane: reads an external corpus, lands a finding
    Census:    +0
    Reach:     0 (predicted and required — this lane writes zero `crates/` bytes)
    Status:    REGISTERED BEFORE ANY MEASUREMENT OF THE DECOMP POPULATION

**Nothing in `crates/` moves in this lane, at all.** The deliverable is a
finding under `docs/`, one script under `scripts/`, and board rows. `gate.sh`
content-hashes `crates fixtures scripts`, so the script invalidates the gate
cache and the gate is re-run; no `crates/` byte is touched, so the required-zero
byte delta is trivially met and is not this lane's grade.

---

## 0. The question, stated so it can come back "no"

`docs/GOAL_DECISION_2026-08-21.md` § AMENDED names a downstream consumer of the
port: *"a better permuter to 'brute force' fixing code that is close, but wrong
because of opaque compiler internal state."*

`docs/DIFF_STRUCTURE.md` measured **the port's own** wrong-body population and
found it is one mechanism — c2 inlined a callee the port emitted a call to.
Board **#3369** records the conflation this lane exists to break:

> the owner's permuter use case is *matching pretext for hand-written decomp
> source*, a **different population** from the port's own refused bodies, and
> nothing in this tree has measured the two against each other.

**The question: is the failure population a permuter would actually face shaped
like the port's own wrong-body population, or not?**

A permuter searches a space. `DIFF_STRUCTURE` says that for the *port's*
population a search over allocation or scheduling would point at nothing —
**0 pure reorderings in 3,195 bodies, 2 field-only words in 5,189**. If the
decomp near-miss population has the same shape, the permuter to build searches
the inline decision. If it does not, it searches something else and
`splice.rs:57-60` is the wrong knob.

**This lane does not build a permuter.** It decides which one is worth building.

---

## 1. DENOMINATORS AS BELIEVED NOW — read before this file existed, stated here

Every figure below was read read-only from the pristine trees before any
measurement script existed. They are the *selectors*, not the measurement.

### 1.1 The decomp corpus — `../dc3-decomp` @ `15a64d92f`, **0 dirty files**

| | |
|---|---:|
| `objdiff.json` units | **2,224** |
| … carrying a `base_path` key at all | **980** |
| … with **both** target and base `.obj` on disk | **979** |
| … base present, target absent | 1 |
| … target present, base absent | 0 |
| … no `base_path` key (target-only, never decompiled) | **1,244** |
| `.obj` files under `build/373307D9/**` | 6,228 |

Manifest stamp (path + size over both obj dirs, sorted):
`45a06795a83cf6e770bc7a5a9d1769f8c1d7818ca84b37b077baa7b3c0032329`.
**Board #3500's lesson is taken**: a commit alone is not the stamp, because
`../dc3-decomp` has been observed to move its corpus without moving its commit.
The measurement will additionally record a **content** digest of exactly the
pairs it read, and will refuse to publish if that digest changes between the
control run and the measurement run.

Both objs parse as COFF `machine 0x01F2` (PPC big-endian) — verified on
`App.obj` (target 317 sections / 1,648 symbols; base 403 / 1,815).

### 1.2 `decomp.db` `functions` — 52,898 rows

| `current_percent` band | rows |
|---|---:|
| `typeof` = `real` (a score exists) | **34,843** |
| `typeof` = `null` (never scored) | **18,055** |
| ≥ 100 | 30,893 |
| [99, 100) | 516 |
| [90, 99) | **1,318** |
| [50, 90) | 418 |
| (0, 50) | 27 |
| = 0 | 1,671 |

`verdict`: `COMPLETE` 31,241 · `AT_LIMIT` 3,593 · empty 18,064.

> **A trap recorded before it can be quoted at me.** `current_percent` is
> **objdiff's fuzzy match percentage** — the exact scoring
> `docs/FUNCTION_BYTE_MATCH.md` and `DIFF_STRUCTURE.md` §1.1 refuse, because it
> pays more for a wrong emit than for an honest refusal. It is used here as a
> **selector** for "which functions is a human currently near", and **never** as
> the measurement. Every shape figure this lane publishes is computed from the
> bytes.
>
> A second reason not to trust it as a measurement: a `sum(current_percent is
> null)` aggregate over this table returned **0** while `typeof` returns
> **18,055** nulls. Something in that table's bookkeeping does not agree with
> itself. It is a selector.

### 1.3 The port side — c2-rs @ `a8593651b`

`docs/DIFF_STRUCTURE.md`'s published table is at tree **`0c8a185`**: 3,195
bodies, 5,189 substituted words. `docs/STATUS.md` line 293 says the tree now
reads `fnbyte-differs` **1,960** + `fnbyte-reloc-differs` **530**. **Neither of
those is verified by me yet.** The port side will be **rescanned, not quoted**
— the brief's instruction and #3369's own finding (the instrument
`gap/fndiff.rs` ships and prints `DIFF STRUCTURE` on every scan; it is not to be
rebuilt).

---

## 2. THE MEASUREMENT, specified before it runs

**The lens must be the same on both sides or the comparison is not one.**
`crates/c2-harness/src/gap/fndiff.rs` is the shipped lens: LCS over 4-byte
big-endian words, adjacent insert/delete runs paired into substitutions, a
field classification per substituted pair, a `same_multiset` reordering bit, and
relocation-site awareness. It cannot be pointed at two arbitrary objs without a
`crates/` edit, and this lane writes zero `crates/` bytes.

So the decomp side is measured by `scripts/permeasure.py`, a re-expression of
that lens — and **the re-expression is graded before it is used**.

### 2.1 THE CONTROL — board #2064's rule, applied to myself

> *A rescoring harness that cannot reproduce the published scores is measuring
> something else.*

`fndiff.rs::to_json` emits `port_hex` and `ref_hex` — the **full word lists** of
both bodies — alongside its own `first`, `equal`, `sub`, `ins`, `del`,
`same_multiset`, `classes`, `csig` and `sig`.

**The control:** run the shipped Rust instrument over the port population on
this tree with `--fnbyte-diff-jsonl`, feed `scripts/permeasure.py` **only** the
`port_hex`/`ref_hex` arrays from each row, and require it to re-derive every one
of those fields **exactly, row for row**.

`scripts/permeasure.py` **refuses to print a single decomp number until that
control passes.** A control that runs after the interesting number is not a
control.

Rows where `body_truncated` is true are excluded from the control and the
excluded count is published beside the pass rate — a control's denominator is
the first thing to lie.

### 2.2 The decomp population, defined by bytes and not by a score

* **P (the pairable population)** — every symbol that names a `.text` COMDAT
  body in **both** the target obj and the base obj of one of the 979 units.
  Byte-identical bodies and differing bodies both counted; both published.
* **N (the near-miss population)** — the members of P whose bodies differ.
* **N₉₀** — the subset of N whose `decomp.db` `current_percent` is ≥ 90, i.e.
  the band a permuter is actually run on. **Both N and N₉₀ get a full shape
  table with their own denominator printed in the same sentence** (#3356).

Relocation blindness is board **#984**'s trap and it bites hardest here: under
`/Gy` a `bl` out of a COMDAT carries the same four bytes whatever the callee, so
a byte-equal word can be a *different call*. Both sides' relocation tables are
read and every figure that could be affected is reported twice — once
byte-equal, once relocation-target-equal.

### 2.3 What is computed, per differing body

The `DIFF_STRUCTURE` lens exactly: `port`↔`base` (ours) and `ref`↔`target`
(theirs); common prefix / suffix in words; first divergence index; LCS
alignment; substitutions paired out of adjacent ins/del runs; per-substitution
field class (`opcode` / `reg` / `imm` / `disp` / mixed / `undecoded`);
`same_multiset`; `has_transfer` on each side under `DIFF_STRUCTURE` §3's
predicate (primary 16 or 18, or primary 19 with XO 16/528); relocation sites.

---

## 3. PREDICTIONS — with probabilities, never edited afterwards

The reasoning behind all of them, stated once so a hit is not read as luck: the
port's population is wrong because **the port is immature** — it emits a call
where c2 inlined. A decomp near-miss goes through the **real** compiler, so the
real compiler makes the inlining decision from the human's source; what differs
is the *source*, and source differences buy you register assignment, expression
and store order, stack-slot displacement and literal choice. I therefore expect
the two populations to be shaped **differently**, and I expect the classes
`DIFF_STRUCTURE` measured at **zero** to be the classes that are non-zero here.

| # | prediction | p |
|---|---|---:|
| **P1** | In **N₉₀**, the share of substituted words whose **opcode** differs is **< 90 %** (port: 5,173/5,189 = **99.7 %**) | **0.75** |
| **P2** | Pure reorderings (`same_multiset`) are **> 1 %** of **N** bodies (port: **0 of 3,195**) | **0.60** |
| **P3** | Bodies already wrong at **word 0** are **< 50 %** of **N** (port: 3,013/3,195 = **94.3 %**) | **0.80** |
| **P4** | Register-field-only substitutions are **≥ 5 %** of **N₉₀**'s substituted words (port: 2/5,189 = **0.04 %**) | **0.70** |
| **P5** | The population is reachable at all: **\|N\| ≥ 200** | **0.85** |
| **P6** | The §2.1 control reproduces `fndiff.rs` on **≥ 99 %** of non-truncated rows **on the first run** | **0.45** |
| **P7** | **The recommendation flips**: the permuter worth building searches **allocation / scheduling / expression order**, not `splice.rs:57-60`'s inline cost model | **0.70** |
| **P8** | Inlining is nevertheless a material minority in **N**: **≥ 10 %** of differing bodies show a transfer-count disagreement (one side calls, the other does not) | **0.30** |

**P6 is deliberately below even.** Every re-derivation control in this repo's
history has caught something on its first run (#2064 caught two label defects,
#2130 caught a superseded verdict function). Predicting it passes clean would be
predicting against the file's own record. **If P6 misses, that is the budgeted
surprise and it is reported as a result, not repaired silently.**

---

## 4. THE DECLINE FLOOR — written before the numbers, so it cannot be moved

The brief is explicit that a priced decline is a good outcome. These are the
conditions under which this lane says **declined** and prices the corpus instead
of publishing a shape:

1. **`|N| < 100`.** Fewer than 100 differing function bodies reachable with both
   sides present ⇒ **declined**. The deliverable becomes: the corpus that would
   make the measurement possible, priced, with what is missing named.
2. **The control cannot be made to pass at ≥ 99 % after ONE repair.** ⇒ the
   decomp shape is published **UNGRADED** and the recommendation is
   **declined**. I will not publish a shape number from an instrument that
   cannot re-derive the shipped one; #2064 is the whole reason that rule exists.
3. **The corpus moves under me** — the §1.1 content digest differs between the
   control run and the measurement run ⇒ the run is **VOID** and re-taken
   against a pinned snapshot (#3500's answer), or the lane declines.
4. **`|N₉₀| < 50`.** The ≥ 90 band is the permuter's actual domain; below 50
   bodies I will publish N's shape but **decline the recommendation**, because a
   recommendation about a search space needs a population the search would run
   on.

**A recommendation is not owed.** If the two populations turn out to be shaped
the *same*, that is a real answer that confirms `splice.rs` and it will be
reported as one, at whatever probability P7's miss implies.

---

## 5. What this lane will NOT do

* **Not build a permuter**, or any part of one.
* **Not touch `crates/`** — the decision surface clause (`rungs/README.md`
  § "Lane kinds" 2) is about construct rungs and this is not one.
* **Not edit `docs/DIFF_STRUCTURE.md`'s published table.** The brief says
  *rescan rather than edit*; the rescan's numbers land in this lane's rung and
  in a dated banner, and `0c8a185`'s table stays as the record of what was
  measured then. #3369's own lesson — a doc kept verbatim under a dated banner
  is worth more than a tidy page.
* **Not multiply marginals.** Every intersection (differing **and** ≥ 90,
  reordering **and** call-disagreeing) is measured jointly and never inferred
  from two rates.
* **Not report a clean result as reassuring without naming its population.** If
  N comes back small or oddly shaped, the first sentence says what it ran over.
