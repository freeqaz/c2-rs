# w-inline — PREREG addendum 2, 2026-08-06 — SAMPLE-A is scored and the rule is FROZEN

Written **after** SAMPLE-A (20 TUs, sha256 `8120a79a…`) was graded in both
directions and **before** SAMPLE-B (100 TUs, sha256 `c2eeba0c…`) was compiled.
No SAMPLE-B obj exists at this commit.

## 1. The two-sided grader, and why the one-sided number was not enough

Addendum 1 graded the single direction an obj can falsify. That number is
**gameable**: a rule that predicts "never inlined" everywhere has zero
falsifications. `work/w-inline/grade_pair.py` closes it with a second real
compilation — the same TU, the same workload flags, **`/Ob0` appended** — as
the SITE ENUMERATOR. With inline expansion off, every source-level call to a
same-TU function leaves exactly one REL24, so `sites(G)` is measured by the
compiler and not modelled. Three confounds are handled in the file's own doc
comment and one is worth naming here because it was not anticipated:

> **Inlining MIGRATES call sites.** 158 of `Utl.cpp`'s `/O1` edges do not exist
> at `/Ob0` — c2 inlined `H` into `F` and `H`'s own un-inlined calls came with
> it. The grade is therefore aggregated **per callee**, never per (caller,
> callee) pair.

## 2. SAMPLE-A, both variants, 12,242 graded callees over 20 TUs

| variant | accuracy | `INLINED-ALL` precision | recall | false inline | false decline |
|---|---:|---:|---:|---:|---:|
| `INLINE-P` **as written** (obj-derived leaf bit) | **0.9636** | 0.9531 | 0.9989 | 435 | 10 |
| `INLINE-P` **with the 48-byte leaf term dropped** | **0.9760** | 0.9772 | 0.9898 | 204 | 90 |

**Majority-class baseline 0.7232.** 0 callees dropped for not appearing in both
objs.

**R2 (addendum 1) HITS and `PREREG.md` P5 MISSES.** P5 registered the leaf term
as load-bearing on the workload; it is *anti*-load-bearing, and the mechanism is
the one §6.19.6 already named — the term keys on a call the SOURCE contains, and
on real C++ a callee's own calls are themselves inlinable, so the emitted body
reads leaf where the decider read non-leaf. The improvement is not the one-sided
artefact either: it trades **231 fewer false inlines for 80 more false
declines**, which a conservatism shift alone cannot do.

## 3. **THE STEP IS WHERE §6.17.4/§6.17.5 SAY IT IS** — the hold-out result

11,866 `EXTERNAL` + `SELECT_ANY` (i.e. `inline`) non-variadic workload callees,
by index, observed inline rate:

```
   index  <=24   24   28   32   36   40   44   48   52   56   60   64 | 68   72   76   80   84 .. >=112
    rate  .997 1.00 .962 1.00 .890 .899 .911 .980 .928 .910 .872 .701 |.157 .000 .061 .000 .025 .. .001
                                                                      ^
                                            the 64/68 step of §6.17.4 + §6.17.5
```

Fitted on `static int f(int)` ladders at `/O1 /GS- /c`; reproduced here on real
C++ templates, constructors, destructors and operators at
`/GR /O1 /Oi /EHsc`, with `this`, `sret`, references and up to five parameters.
Neither the flags nor the shapes were available to the round that measured it.

## 4. FROZEN, before SAMPLE-B is compiled

* **`INLINE-P` is graded with `--drop-leaf-term`**, i.e. `index = s` (STATIC) or
  `s − 4·(nparams−1) − 8·[SELECT_ANY]` (EXTERNAL), no leaf term. Every other
  constant is §6.15–§6.20's, unmodified.
* **R1's funclet exclusion stands**, for the independent reason that a
  `__unwind$N` name is a compilation-local counter and cannot be paired across
  the two compilations at all.
* **No term is fitted to SAMPLE-A's 294 misses.** They are published as a
  population in the rung: 204 false inlines (98 with an ordinary caller only,
  106 with an ordinary caller and a funclet), 90 false declines, both clustered
  within ±8 bytes of the step, and 155 of the 294 are `operator`s. `PREREG.md`
  §5 forbids repairing them and this file is the record that they were seen and
  not repaired.

## 5. What SAMPLE-B and GRID-2 now decide

`PREREG.md` P4 (≥ 0.90 two-sided on the workload) and P8 (≥ 0.90 on GRID-2's
discriminating cells). The floor fires below either.
