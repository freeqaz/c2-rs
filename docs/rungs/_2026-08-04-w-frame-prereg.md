# Pre-registration — lane w-frame

    Slug:      w-frame
    Date:      2026-08-04
    Lane:      w-frame (`wt-w-frame`, base master `73e5831`)
    Committed: BEFORE the first measurement of this lane. Nothing under
               `work/w-frame/` exists yet; no obj has been captured; no ranking
               has been computed.

This file is frozen. It is scored verbatim in the rung doc and the wrong
predictions stay on the page.

---

## 0. Why this lane exists, in one paragraph

TU match is 8/878. `A∧B∧C` = 25, so **17 graded TUs are behind codegen alone**
(the FRONTIER). Three lanes — `w-front`, `w-pair`, `w-cfgimpl` — each picked a
frontier target and each converted **zero**, and each independently reported the
same cause: **the frontier's published ranking is by blocked-function count and
that key is wrong** (`BOARD.md` #198, the fourth instance of #150's shape).
Nobody has yet ranked the frontier by *how much unmodeled machinery a TU needs*.
That ranking is this lane's first deliverable and is worth having even if this
lane converts nothing.

## 1. The bias, stated before the scan

**I want the answer to be: at least one frontier TU is one or two constructs
away from the port's present vocabulary, and this lane converts it.** That is
the outcome that makes the lane look successful, and it is exactly the
proposition the last three lanes each believed at their start and each had
refuted by their own bytes.

The direction of the bias is therefore **optimistic on cheapness**, and the
correction the project's own history prescribes is:

> **When a row's blocker is a class whose emitter already exists, the ceiling IS
> the estimate.** Count the *independent* refusals between the ceiling and the
> emitter and apply **no discount**. Five confirmations; eight consecutive
> estimate misses before the rule; every discount ever applied here has been
> wrong.

"Independent" is load-bearing: if one quantity governs several boundaries, that
is **one** refusal, not several.

So every number I register below is a **ceiling taken neat**, and where I am
tempted to write "but probably fewer", I write the ceiling instead.

## 2. The instrument — the feature-union ranking

**The claim I am replacing.** The frontier's published order is by *blocked
function count*. A TU with 40 blocked functions that all need one construct is
cheaper than a TU with 2 blocked functions needing five. The published key
cannot tell those apart; three lanes lost to exactly that.

**The measurement.** For each of the 17 FRONTIER TUs:

1. Compile the TU with the real `cl.exe` 16.00.11886.00 / `c2.dll` under wibo at
   the **workload's own flags** (`work/dc3-workload/flags.txt` — *not* `capture`'s
   `/Ox` default; `BOARD.md` #194).
2. Read the real obj: section table, symbol table, relocations, and a
   big-endian PowerPC disassembly of every `.text` COMDAT
   (`scripts/gt_dump.py`'s route — `llvm-mc -disassemble -triple=powerpc`).
3. Classify each emitted function into a **feature set** over four axes:
   * **frame class** — leaf / `stwu`-allocated frame / `__savegprlr_N` register
     save / `.pdata` COMDAT present;
   * **instruction families** — by mnemonic, bucketed (int-arith, rotate-mask,
     load, store, indexed load/store, compare, conditional branch, `bclr`, CTR
     loop, indirect call `mtctr`/`bcctrl`, multiply, divide, float, byte-reverse,
     …);
   * **relocation kinds reaching that function** — `REL24`, `REFHI`/`REFLO`
     pairs, `ADDR32`, `SECREL`, …;
   * **TU-level shell** — the obj's section set beyond `.text`.
4. **`union(TU)`** = the union of those feature sets over **every function the
   obj emits**, because a TU matches only if the port emits the *whole* obj.
5. **`port_vocab`** = the same classifier run over the objs the **port itself
   emits today**, measured — every fixture on which `c2rs perf` reports
   `Port=Match`. Not an asserted list; a measured one.
6. **`gap(TU) = |union(TU) \ port_vocab|`** — the count of *distinct unmodeled
   constructs*. **This is the ranking key.** Rank ascending; the minimum is the
   candidate.

**What the key is not.** `gap(TU)` is a **ceiling on cheapness, not a price**.
Two named ways it can mislead, registered here so they cannot be discovered
later and presented as insight:

* **It under-counts.** Two constructs in the same bucket can be independent
  facts (`w-cfgimpl` §4.1: one `if`-fold is *four* independent unmeasured facts —
  bool spine, constant materialization, mask derivation, destination
  allocation). A bucket is one unit here and may be several refusals.
* **It cannot see scheduling.** `w-pair` killed six store-scheduling rules and
  found `xboxheap.cpp` diverges at **instruction 0** on *order*, with every
  individual instruction already in the port's vocabulary. A TU can score
  `gap = 0` and still be unemittable. **`gap = 0` is therefore not a conversion
  claim** and I register in advance that I will not treat it as one.

Both of those push in the direction of my bias (they make TUs look cheaper than
they are), which is the reason for writing them down before the numbers exist.

## 3. Predictions — the instrument (A)

Scored against the ranking, before any implementation.

| # | prediction | rival reading if wrong |
|---|---|---|
| **A1** | The feature-union key and the blocked-function key **disagree materially**: Spearman ρ between them over the 17 TUs is **< 0.5**. | R-A1: the two keys agree (ρ ≥ 0.5) and the three lanes' complaint was about *which* function, not about the ranking. If R-A1 holds, this lane's headline deliverable is worth much less and I say so. |
| **A2** | The **minimum** `gap(TU)` over the 17 is **≥ 3**. | R-A2: some TU is 1 or 2 constructs out. This is my bias's own prediction; registering the pessimistic side means R-A2 winning is a **good** surprise I did not buy. |
| **A3** | `xboxheap.cpp` — once called "the cheapest conversion in the project", then found scheduler-blocked — is **not** the minimum by this key. | R-A3: it is the minimum, and the key inherits the same blindness that made it look cheap (§2's second caveat firing on the very TU that motivated it). |
| **A4** | **Every** one of the 17 TUs requires at least one of {`stwu` frame, `__savegprlr_N`, `.pdata` COMDAT} — i.e. **no** frontier TU is leaf-only end to end. | R-A4: at least one is leaf-only, in which case the "general framed-function class is the wall" reading (w-cfgimpl) is too strong. |
| **A5** | The 17 TUs' feature unions are **not** nested into a chain: there is no single ordering where each TU's union contains the previous one's. Concretely, ≥ 3 constructs appear in exactly **one** TU each. | R-A5: they nest, meaning one widening ladder sweeps the frontier in order and the ranking is the ladder. |
| **A6** | `port_vocab` measured over the matched fixtures has **fewer than 25** distinct feature tokens. | R-A6: the port's measured vocabulary is broad and the gaps are combinational rather than lexical. |

## 4. Predictions — the outcome (B)

| # | prediction |
|---|---|
| **B1** | **Final TU match is 8.** Registered as the *ceiling-neat* reading of §1: the port's present accepted class does not contain any whole frontier TU, and no lane has yet moved this number by codegen. A value of 9 refutes B1 and is the outcome I want. |
| **B2** | **mismatch is 0** everywhere — the 878-TU scan and every `scripts/gate.sh` lane. This is an alarm, not a metric; a nonzero value aborts the lane. |
| **B3** | `capture-fail` stays **7**. A different number means the scan is not comparable and the lane's numbers are void until it is explained. |
| **B4** | The FRONTIER stays **17** unless this lane converts a TU, in which case it is 16. It cannot *grow* from anything this lane does. |
| **B5** | Census delta is **≤ +50 functions** if I implement anything, and exactly **0** if I do not. Registered against the pull to quote a large bucket as a win: `w-cfgimpl` moved 3 functions with a rung that was byte-exact on its first differential run. |
| **B6** | If I implement, `cargo test --workspace --release` gains **≥ 1 portable test per ordering rule** the rung introduces, and the `#[test]` count diff between merge-base and tip is **> 0**. A large rung with a zero test diff is a finding I will report as one. |

## 5. Decline clauses — registered in advance

I will **stop and write up the measurement instead of building** if any of these
fires:

1. The minimum `gap(TU)` is **≥ 4**. At four independent unmodeled constructs the
   ceiling-neat rule says the cost is four rungs, not one, and this lane's
   deliverable is the ranking.
2. The minimum-`gap` TU's constructs include a **schedule** or an **allocation**
   decision with fewer than three witnesses (w-pair's wall, w-cfgimpl §4.1's
   `lis` slot, `CODEGEN_W6_COMPARE.md` §6's uncharacterized register allocator).
   Fitting one of those from one cell is the single most expensive recurring
   mistake in this project's log.
3. The candidate requires a **fold cost model** — `CFG_SHAPE.md` §3.5, board
   #187. That table is fitted by all cells and tested by none and this lane will
   not build on it, exactly as `w-cfgimpl` did not.
4. Any existing fixture verdict changes, any `gate.sh` lane is lost, or
   `mismatch` goes nonzero.

**Declining on measurement is a good outcome.** The ranking is the deliverable;
a conversion is a bonus.

## 6. Controls

* **Provenance bracketing.** `../dc3-decomp` HEAD is recorded **before and
  after** every scan. wibo version is recorded (`wibo 1.0.1-23-g4a9dd6f`).
* **Known-answer control on the classifier.** The classifier is run over
  `src/system/synth/tomcrypt/TomCryptLicense.cpp` and
  `src/system/zlib/ZlibLicense.cpp` — two TUs that **are** matches today. Their
  `gap` must be **0**, because the port demonstrably emits them byte-exact. A
  nonzero `gap` on a matched TU means the classifier's `port_vocab` is
  under-measured and every other number it prints is void. This control is
  chosen because it can only fail in the direction that would flatter the lane.
* **Second control, the other direction.** `xboxheap.cpp` must score **low**
  (its instructions are all in vocabulary) and is **known** to be unemittable
  (w-pair: diverges at instruction 0 on schedule). If it does not score low, the
  classifier is not measuring what §2 says it measures.
* **Compare FAILED counts, never passed counts** — a failing target aborts the
  run and a lower passed count reads as green.
* **`coff.rs`** is owned by this lane for its duration; any ordering rule that
  touches it ships with a portable assertion, because `differential.rs` names a
  fixed list of three fixtures and `cargo test` alone does not catch an ordering
  bug there.

## 7. What this lane will not do

* No fold cost model (#187).
* No block/instruction IR restructure (`CFG_SHAPE.md` §6) unless a shape with a
  *variable* block order is admitted, which §5's clauses make unlikely.
* No loop, no `switch`, no CTR family, unless the ranking puts one at the
  minimum **and** it survives §5.
* No edits to `BOARD.md`, `ROADMAP.md` or `STATUS.md` beyond what a landed rung
  requires; `rungs/INDEX.md` is regenerated by script only.
