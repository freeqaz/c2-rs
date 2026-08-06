# w-inline — PREREG addendum 1, 2026-08-06

Written after `work/w-inline/objs/Utl.obj` (ONE TU, the top row of
`work/w-fnbyte/differ_taxonomy.txt`) was graded and before any other workload
obj was compiled. `work/w-inline/scan_obj.py` and its `--selftest` are committed
in the same commit as this file.

## 1. The workload sample is frozen, and it is SPLIT

`PREREG.md` §2 registers "the workload" as the hold-out without saying which
TUs. Naming them after looking at one of them would let the rest be chosen. So:

| file | n | sha256 | role |
|---|---:|---|---|
| `sample_a.txt` | 20 | `8120a79ae84600c197642d8841d9c14f21736556f63efb3170cc83f02df719e6` | **diagnose / repair.** The 15 top-differ TUs of `differ_taxonomy.txt` plus 5 drawn with `random.Random(20260806)` |
| `sample_b.txt` | 100 | `c2eeba0cb9689266449bedf553ff69e76812930cb71ece949d6f9c317699904e` | **HOLD-OUT.** The next 100 of the same shuffle. **Not compiled, not read, and not counted until SAMPLE-A is scored and any repair is frozen.** |

The generator is inline in this file's commit message and reproduces both
hashes from `work/dc3-workload/files.txt` alone.

## 2. Two findings from the one TU, registered before they are measured at scale

Both change what `INLINE-P`'s inputs *are*; neither changes a constant.

**(a) `/EHsc` adds a call site the ladders had no way to contain.** `Utl.obj`
falsifies `INLINE-P` 73 times, and **40 of the 73 have no caller except
`__unwind$NNNNN` funclets** — the EH unwind bodies `/EHsc` generates. All of
§6.15–§6.20 was captured at `/O1 /GS- /c`, where no funclet exists, so this is a
site class the incumbent has never been graded on.

> **REGISTERED — R1.** A call site inside a `__unwind$` funclet is **never**
> inlined, at any callee index. It is scored as a *site-side categorical
> refusal*, in the family of §6.18.4 (indirect site) and §6.19.9 (the site's
> CFG). **It loses** if any `__unwind$` funclet in SAMPLE-B contains no REL24 to
> a same-TU function it must have called — which is not directly observable, so
> the *falsifiable* form is used instead: **it loses if excluding funclet
> callers does not reduce the falsification count**, and it is **not** scored as
> a hit merely because funclets keep their calls.

**(b) `leaf` is NOT obj-readable on real C++, and the incumbent says so.**
§6.19.6 measures that the 48-byte term keys on *a call the SOURCE contains*, and
concludes the index is *"a post-allocation byte count, minus 48 for a predicate
that is false in the emitted code and true only upstream of it."* On the
ladders that distinction never bit, because no callee's own calls were ever
inlinable. On the workload they are: `??$__uninitialized_fill_n@…` calls a copy
constructor that c2 inlines away, so the emitted body is call-free and
`is_leaf()` reads TRUE where the decider read FALSE.

> **REGISTERED — R2.** Grading `INLINE-P` with the leaf term **dropped**
> (`--drop-leaf-term`) scores *better* on the workload than grading it with the
> obj-derived leaf bit. **This is the opposite of `PREREG.md` P5**, which
> registered the term as load-bearing; P5 is now expected to MISS and is left
> in the table unmodified so the swing is on the record. R2 loses if the
> obj-derived leaf bit scores better or equal.

## 3. What is NOT changed

No constant of `INLINE-P` moves. §1's function is graded exactly as written; R1
is a *site* filter applied to the observation, and R2 is a choice between two
readings of an input the incumbent itself calls unmeasurable. Both are decided
on SAMPLE-A and then **frozen** before SAMPLE-B is compiled.
