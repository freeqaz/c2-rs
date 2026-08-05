# w-tu3 — pre-registration

Written **before** any measurement in this lane, before the ranker exists, and
before any frontier obj was disassembled. Committed as its own commit so the
timestamp is checkable.

Lane `w-tu3`, worktree `wt-w-tu3` off master **`9ffbad4`**.

Board rows taken: **#500**–**#505** (highest was #484; brief says start at #500).

---

## 0. What this lane is for

Three tasks, in order, from the brief:

1. Build the **byte-fraction ranker** in `crates/c2-harness/src/gap/` and print it
   for every frontier TU, ranked. It must not be gameable and a TU with zero
   emitted bytes must not score 100 %.
2. Give w-tu2's **inter-function label stride** rule a `crates/` home and a
   fixture — and *not* overclaim it into the intra-function case.
3. Convert the top TU by the corrected ranking, re-pricing it myself.

---

## 1. Registered predictions

| # | prediction | how it is scored |
|---|---|---|
| **R1** | The baseline reproduces exactly: match **9**, mismatch 0, codegen-gap 0, vocab-gap **862**, capture-fail **7**; A/B/C/D/E = 28 (LO 27)/338/169/9/2; `A∧B∧C` **27**; `A∧B∧C∧D` **7**; FRONTIER **18** | a scan run and read |
| **R2** | The ranker, computed independently in `crates/` from the reference obj's own `.text` COMDAT bytes, **reproduces w-tu2's two hand-counted cells within rounding**: `xboxmem` ≈ 54.5 % (72/132), `mmio` ≈ 16.8 % (64/380). If it does not, one of the two measurements is wrong and I must say which. | printed ranker output vs `rungs/_2026-08-05-w-tu2.md` §3.1 |
| **R3** | `xboxmem` is **not on the frontier any more** (it converted), so the ranker's frontier table will not contain it; I will have to compute it as an off-frontier control to score R2. | printed output |
| **R4** | **The ranker's top frontier TU is NOT `mmio`.** (`mmio` scored 16.8 %; if it still ranks first the byte metric has not separated anything and task 1 has failed.) | printed output |
| **R5** | **No frontier TU scores ≥ 54.5 %** — i.e. the ranker's top is *below* the one converted data point, so task 3 is a harder conversion than w-tu1's. I register this as the pessimistic reading; a hit here predicts I do **not** convert a TU. | printed output |
| **R6** | The stride rule `5 + 1·(leaf/tail) + 5·(framed)` **reproduces in `crates/` as a pure function** and holds on w-tu2's 36 cells re-encoded as unit-test data. It is **not** claimed for the intra-function charge; a test asserting the intra-function case is *open* will be written so the boundary is machine-checked. | `cargo test` |
| **R7** | My own re-price of whatever TU the ranker names will come back **dearer than w-dclass's reprice table** — the fourth consecutive such cross-check (10 vs 4, 15 vs 4, 17 vs 5). | hand count off the obj |
| **R8** | The gate is unmoved except for fixtures I add: `cargo test --workspace --release` ≥ **813 passed / 27 targets**; `gate.sh` PASS with `18 + 1·(fixtures added)` lanes and `4,482 + 18·(fixtures added)` verdicts; sweep **96 ungraded** and cross **388 ungraded** both HOLD. | runs watched to completion, read from logs |
| **R9** | **No fitted schedule/placement rule ships.** If one fits, it must hold on cells it was not fitted to or be refused in writing. | this record |

## 2. The anti-gaming specification, registered before it is written

The defect to avoid is objdiff's `calc_fuzzy_match_percent` returning `100.0`
over zero code bytes, and this project's most-repeated defect (absence read as
success, 16+ instances). The registered design:

* **The denominator is a function of `c2`'s output alone** — the sum of
  `.text` COMDAT raw-data lengths in the *reference* obj, exactly the
  population `fnbyte-denominator` counts. The port cannot shrink it.
* **A zero denominator yields NO ratio.** It is counted under its own key and
  printed as `n/a`, never as 100 %. A positive count of such TUs is printed.
* **The numerator excludes `Differs`.** A wrong emit must *lower* the score, not
  raise it, so bytes of a function whose body the judge has already said is wrong
  are credited nowhere and are printed as their own alarm count.
* **The denominator is printed beside every ratio**, per the brief.
* **It is an instrument, never a gate.** It licenses no emit and appears in no
  accept/refuse path.

## 3. What would refute the premises I was handed

* If the ranker cannot reproduce w-tu2's 54.5 % / 16.8 % from the objs, the
  corrected heuristic rests on two hand-counted cells and one outcome each, and
  that must be said plainly.
* If the frontier's byte fractions are all in a narrow band, the metric ranks
  nothing and is not an improvement on #465 — it is only a *different* wrong
  unit. I will print the full distribution, not just the top.
* Two cells and two outcomes is **n = 2**. Even a perfect reproduction does not
  make the byte metric *validated*; it makes it *consistent with the only two
  outcomes there are*. The honest ceiling on any claim here is low and I register
  that now, before seeing the numbers.
