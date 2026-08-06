# w-fnbyte — PREREGISTRATION

    Lane:    w-fnbyte
    Board:   #322 (FBM's blind spot) — prerequisite of #844
    Base:    master `33a1867`, branch `wt-w-fnbyte`
    Written: 2026-08-06, BEFORE any probe, any code change and any scan of this
             tree. Nothing under `work/w-fnbyte/` exists but this file.

---

## 0. The thing being tested

`fnbyte-differs 0` is the project's standing per-function alarm. Lane `w-seam`
established that it is **structurally blind** to every function whose
`codegen::select_function` verdict is one of `Selected::{Tail, Framed, Seq,
CondPair}` or a `Float` with pooled constants: `crates/c2-harness/src/gap/
fnbytes.rs::complete_body` maps all five to `FnByte::Partial(..)` and declines to
compare bytes at all. The scan prints the population —
**`partial by shape: tail 7098 · seq 2150 · framed 123 · cond-pair 4`**, 9,375
functions — and a wrong emit in any of them reads as `differs 0`.

`docs/FUNCTION_BYTE_MATCH.md` §3.1 gives the decline reason as *"the missing
words encode their own `.text` offset"* and says the reconstruction *"needs a
per-function entry point in `c2-core`, which is the crate that owns the fact"*.

## 1. The hypothesis

> **H.** The decline reason is stated for the PACKED emitter, and FBM's
> denominator is the **`/Gy` COMDAT** population. `PortC2::build`'s
> `fn_level_linking` branch already builds every one of these bodies *complete*,
> at `text_offset = 0`, because under `/Gy` each function starts at offset 0 of
> its own COMDAT. So the offset the harness "cannot know" is a **constant zero**
> for exactly the population FBM grades, and the four shapes are gradeable by
> lifting that loop body — unchanged — into a public per-function entry point in
> `c2-core` that both `PortC2::build` and the harness call.

The lift is the whole design constraint: **the harness must call the emitter's
own code, never a second copy.** A reconstruction that re-implements the
composition could diverge from the emitter and grade a fiction, which would be a
worse instrument than the blind one it replaced.

## 2. Registered predictions

Scored in the rung, hit or miss, from printed counts.

| # | claim | how it loses |
|---|---|---|
| **P1** | All four shapes become gradeable through the `/Gy` entry point. `fnbyte-partial` falls from **9,375** to **≤ 10**, and `partial by shape` loses its `tail`, `seq`, `framed` and `cond-pair` rows entirely. | any shape still needs a fact the per-function entry point does not have (a TU-wide label counter, an emission-order index, a `data_refs_of` that fails) and stays `Partial` |
| **P2** | **THE CLAIM I MOST EXPECT TO LOSE.** `fnbyte-differs` moves **off 0**, and at least one of the differs is a `tail`. 9,375 functions have never had a byte compared; the workload's shapes are far wider than the fixture set that established these lowerings. | `fnbyte-differs` reads **0** after the widening — i.e. the port is byte-exact on all four newly-graded shapes, on 9,375 workload functions. That is a **strong positive result for the port** and a clean miss for me, and it will be reported as both. |
| **P3** | `fnbyte-exact-relocated` moves **off 0**. `FUNCTION_BYTE_MATCH.md` trap 6 says *"the first accepted shape that relocates makes this bucket nonzero"* — a tail call's `b` is a REL24, so every newly-exact `tail`/`framed`/`seq`/`cond-pair` function relocates. I predict it lands within ±5 % of the number of newly-exact functions. | it stays 0 (would mean the reference COMDATs carry no relocation, refuting the tail shape's own model), or the ratio is outside ±5 % |
| **P4** | **CONTROL — no already-graded function changes verdict.** The 29,802 `fnbyte-exact` functions at the base are exact at the end too: `exact_after ≥ exact_before`, and the count of functions that were `Exact` and are no longer is **0**, measured per shape and printed. | any previously-exact function moves out of `Exact` |
| **P5** | **CONTROL — the partition identity holds at both ends**: `exact + partial + differs + refused + unbound + nobytes == denominator`, `fnbyte-partition-broken 0`, and the denominator (`178,975` scale) is **unchanged** by this lane. | any of those moves |
| **P6** | **CONTROL — the accept/refuse boundary is untouched.** `git diff` on `IlBundle::functions()` and `select_function` is empty; `fnbyte-census-disagree` and the scan's `census/gate disagreement` both stay **0**; TU match reads **10** and `mismatch` **0** at both ends. | any of those moves |
| **P7** | **MUTATIONS — one per newly-graded shape, four total, all RED.** Corrupting a single reconstructed word (the tail branch, the framed `bl`, a `seq` prologue word, a `cond-pair` arm) makes the comparison report `Differs` for that shape. A mutation that stays green means that shape is not actually being reconstructed. | fewer than 4 go red |
| **P8** | `FBM` rises by an **instrument** change, not port progress, and the rung says so in its first sentence. The rise is bounded by `9,375 / 178,975 = +0.0524`; I predict the realised rise is **within `[0.045, 0.053]`** (i.e. ≥ 86 % of the partial population turns out exact). | outside that band — in either direction, and the low side is P2 winning |

## 3. What this lane will NOT do

* **It will not narrow the reconstruction to make `differs` read 0.** If P2 wins,
  the differing functions are reported by name with their bytes and the alarm is
  left red. Board #232/#259/#263/#276 are all defects found by an instrument
  widening; a widening that retreats when it finds one is worthless.
* **It will not fix the emitter.** A codegen change graded only against the same
  instrument that flagged it is circular. Any defect found here is filed, not
  patched, unless the fix is graded against real `c2` bytes and said so loudly.
* **It will not touch `IlBundle::functions()` or `select_function`.** The
  accept/refuse boundary is another lane's; this one is instrument-only.
* **It will not add FBM to `scripts/gate.sh`.** FBM is a progress instrument and
  the separation rule in `FUNCTION_BYTE_MATCH.md` §0 is absolute.

## 4. The measurement protocol

1. Full 878-TU scan at the base, `gap-metric` block saved to
   `work/w-fnbyte/baseline_metrics.txt`.
2. The change, with unit tests and the four mutations committed **before** any
   mutation harness runs (`w-seam`'s #874 destroyed its own uncommitted tests
   with a `git checkout` restore).
3. Full 878-TU scan at the end, `work/w-fnbyte/final_metrics.txt`, and a `diff`
   of the two sorted key files — not a reading of a summary.
4. `scripts/gate.sh --jobs 6` and `cargo test --workspace --release`, both
   quoted as counts (`targets=/passed=/failed=`), never as a status.
