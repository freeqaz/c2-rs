# w-inline — PREREGISTRATION

    Lane:    w-inline
    Board:   #896–#905 (reserved for this lane)
    Base:    master `8521606`, branch `w-inline`, worktree
             `.claude/worktrees/agent-a4d31921a588baa2a`
    Written: 2026-08-06, BEFORE any probe, any grid, any compile and any scan of
             this tree. Nothing under `work/w-inline/` exists but this file.

---

## 0. The thing being tested, and the first correction to the brief

The lane was commissioned to **map c2's `/O1` same-TU inlining predicate** from
scratch, "in the mold of w-magic's division map", on the grounds that lane
`w-fnbyte` found 4,711 emitted functions whose port bytes are wrong and traced
the largest family to *"c2 inlines a callee defined in the same TU and the
port's IL-level call recognizers do not model that decision"*.

> **The brief's step 1 is largely PRIOR ART and this file says so before a
> single probe is compiled.** `docs/LABEL_COUNTER.md` **§6.15 – §6.20** is five
> rounds of exactly this question, run against real `c2` under wibo with
> `scripts/gt_inline_decline.py`, whose module doc names the same motivation
> verbatim: *"`crates/c2-il/src/func/bundle.rs` refuses any TU where a callee is
> also defined, because c2 may inline it; the first rung that relaxes that gate
> has to know WHICH expansion tree it is counting labels for, and today nothing
> can tell it."* Between them those rounds report **449 + 344 + n rungs and
> thousands of objects**, and they have already established every axis the brief
> lists as a candidate — callee body size, callee shape, call-site count,
> linkage, argument count, tail position — plus three the brief does not
> (`inline`, leafness, the call site's CFG position).
>
> Re-deriving that grid would be a **duplicate**, which is the one thing this
> project's rung log punishes hardest. So this lane **registers the prior art as
> the INCUMBENT**, states it as one computable function, and spends its budget
> on the two things §6.15 – §6.20 explicitly could not do:
>
> 1. **grade it on the WORKLOAD** — which is out-of-distribution on every axis
>    those ladders held fixed (see §2), and
> 2. **connect it to the 4,711**.
>
> If that reading of the prior art is wrong — if the incumbent turns out not to
> be stated well enough to compute, or not to apply at `/O1 /Oi /EHsc /GR` —
> this file is the record that the lane bet on it, and the rung will say so.

## 1. The INCUMBENT, stated as one function — `INLINE-P`

Transcribed from `docs/LABEL_COUNTER.md` §6.15.3 (SCHEDULE D), §6.17.3/§6.17.4
(linkage), §6.17.5 (`inline` = 8 bytes), §6.17.6 (the parameter correction),
§6.18.5 (varargs), §6.18.6/§6.18.7 (the leaf term = 48 bytes), §6.18.9 (LAW Dc),
§6.19.6/§6.19.7 (what counts as a call), §6.19.9 (the site's CFG). **Nothing
below is this lane's; every constant has a section number.**

Inputs, per callee `G`, all read from the *reference obj* at the workload's own
flags:

* `s` — `G`'s own emitted `.text` COMDAT size in bytes at `/O1`. (§6.5: c2
  emits the callee's COMDAT whether or not it inlined it, so `s` is free.)
* `linkage(G)` ∈ {STATIC, EXTERNAL} — the COFF storage class of `G`'s symbol.
* `inline(G)` — proposed derivation: the COMDAT selection of `G`'s section
  (`SELECT_ANY` for an `inline`/template/in-class member, `SELECT_NODUPLICATES`
  for an ordinary `/Gy` function). **This derivation is UNVERIFIED and is the
  first thing §3's grid checks.**
* `nparams(G)` — from the mangled name's argument list; `this` counts (§6.17.6).
* `leaf(G)` — `G`'s emitted body contains no call, where REL24s to
  `__savegprlr_*` / `__restgprlr_*` / `__savefpr_*` / `__restfpr_*` are **not**
  calls (§6.19.6) and indirect transfers **are**, with the LK bit ignored so
  `bctr` counts as well as `bctrl` (§6.19.7).
* `varargs(G)` — from the mangled name (`ZZ` terminator).

```
index(G) =  s                                       if linkage == STATIC
            s - 4*(nparams - 1) - 8*[inline]        if linkage == EXTERNAL
         -  48*[leaf]                               both classes (§6.18.6)

N_max(G) =  0                                       if varargs(G)          (§6.18.5)
            EXTERNAL:  UNBOUNDED if index <= 64 else 0                     (§6.17.4)
            STATIC:    i = index/4
                       0                            if i >= 65
                       UNBOUNDED                    if i <= 16
                       min(9, 1 + floor(19/(i-16))) otherwise              (§6.18.9)

c2 inlines EVERY site of G iff  n_sites(G) <= N_max(G).
The decision is ALL-OR-NOTHING per (caller, callee) pair (§6.15.1) and is a
property of the CALLEE alone (§6.15.3a, §6.19.5) — except at an indirect site
(§6.18.4) and except for the 1 -> 0 ceiling, which a conditional site moves from
(256,260] to (160,164] (§6.19.9).
```

**For the workload this collapses.** Nearly every function in `differ_taxonomy`'s
top signatures is an STL template or an in-class member — EXTERNAL, implicitly
`inline` — so the operative rule is a single **binary, per-callee** test:

> **`s - 4*(nparams-1) - 8 - 48*[leaf] <= 64` → c2 inlines it at every site;
> otherwise c2 inlines it at none.**

That is exactly the shape a port recognizer can consume, which is why it is
worth grading rather than re-deriving.

## 2. Why the workload is a real hold-out and not a re-run

§6.19.10 and §6.18.10 enumerate what the ladders held fixed. The workload
violates all of it:

| held fixed in §6.15–§6.20 | the workload |
|---|---|
| every callee is `int f(int)`-shaped, `int`/`double`/small struct | templates, `sret` returns, references, `bool`, class types |
| the caller is always `int P(int a){ int s=gs(a)+a; … }` | 601 TUs of real C++ |
| **recursion never appears on either side** (§6.19.5, §6.19.10) | present |
| every call *inside* a callee is to an **undefined external** | callees calling other same-TU functions, i.e. two-level trees that are themselves inline candidates |
| `inline` is a keyword the probe writes | implicit, via templates and in-class definitions — the derivation in §1 is a **guess** |
| one callee per caller, or two unrelated ones | many |
| indices mostly 48–104 (§6.20's own warning) | the whole range |

So a workload grade is a hold-out in the strong sense, and it is also the only
population that matters for the port.

## 3. The grids, frozen before compiling

Both grids are written to `work/w-inline/`, `sha256`-stamped and **committed
before the first `cl.exe` invocation**. The stamps go in the rung.

* **GRID-1 (fit/repair)** — hand probes at the **workload's own flags**
  (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`, read from
  `work/dc3-workload/flags.txt`, never transcribed), crossing the axes §1's
  derivations are *unverified* on: how `inline`ness appears in the obj, what a
  template instantiation's linkage and selection are, `this`/sret in
  `nparams`, a callee that calls another same-TU function, and recursion.
  GRID-1 exists to make `INLINE-P`'s inputs **computable from an obj**; it is
  not allowed to move any constant in §1.
* **GRID-2 (hold-out)** — frozen and committed *before* GRID-1 is graded,
  varying axes GRID-1 does not: virtual/devirtualised sites, varargs, a
  conditional call site, an address-taken callee (vftable), a callee at the
  `s ≈ 64` and `s ≈ 112` boundaries in both leaf and non-leaf spellings, and a
  multi-level tree.

Grading is from **obj bytes**, never the listing (#843): a call c2 declined
leaves exactly one REL24 against the callee's symbol; an inlined one leaves
none (§6.15's own instrument, `scripts/gt_inline_decline.py`).

## 4. Registered predictions

Scored in the rung, hit or miss, from printed counts.

| # | claim | how it loses |
|---|---|---|
| **P1** | **THE CLAIM I MOST EXPECT TO LOSE.** `INLINE-P` survives the workload's one-sided falsifier: **no** function `G` defined in a sampled workload TU with `index(G) ≤ 64` is the target of a surviving REL24 from another `.text` COMDAT in the same obj. (A surviving REL24 is a decline; the rule says `G` is inlined at every site; the two cannot both hold.) | one such `G` exists. I expect several: recursion, `??_G` thunks and address-taken callees are all in the workload and in none of the ladders. **Self-references are counted and reported separately** and do not decide P1 — a self-recursive `G` is a pair the incumbent has never been graded on. |
| **P2** | The `inline`-ness derivation in §1 is right: a template instantiation and an in-class member both come out `SELECT_ANY` + EXTERNAL, and an ordinary out-of-class free function at `/Gy` comes out `SELECT_NODUPLICATES` + EXTERNAL, and grading `INLINE-P` with the derived bit reproduces the §6.17.5 `s−8` shift on GRID-1. | the selection does not separate them, or the shift does not reproduce — in which case `inline`ness is **not obj-readable** and §1's collapsed rule has an input the port cannot compute. That is a finding, not a failure, and would be reported as one. |
| **P3** | **Family A retrodicts.** For the sampled `tail` differs whose reference body is a bare `blr` (1,886 pairs, taxonomy family A), `INLINE-P` says "inlined" on ≥ 90 % of the (caller, callee) pairs it can resolve. | < 90 %. |
| **P4** | **The workload retrodiction, both directions**, over the resolvable (caller, callee) pairs of the sampled TUs: `INLINE-P` agrees with the obj's own REL24 evidence on ≥ 90 % of pairs. | < 90 %. |
| **P5** | **CONTROL — the leaf term is load-bearing on the workload.** Dropping the 48-byte leaf term makes the P4 number strictly *worse*. A term measured on `int` ladders that buys nothing on real C++ is a term that did not transfer, and the rung would say so. | the number does not get worse (term inert here), or gets better (term wrong here). |
| **P6** | **CONTROL — a positive control that can go red.** A hand probe whose callee is deliberately **just over** the boundary (`index = 68`) keeps its REL24, and the byte-identical one at `index = 64` loses it, at the workload's own flags. If both sides come out the same the grader is inert and every other number in this lane is void. | the two cells agree. |
| **P7** | **CONTROL — nothing ships.** `git diff master..HEAD -- crates/` is empty or touches only `tests/`. The FBM partition is unchanged end to end: `exact` **34,466** does not shrink, `differs` **4,711** does not grow, `fnbyte-match-tu-differs` **0**, scan `mismatch` **0**, TU match **10**. | any of those moves. |
| **P8** | **GRID-2 hold-out.** `INLINE-P`, unmodified by GRID-1, is exact on ≥ 90 % of GRID-2's discriminating cells. | < 90 %, in which case the DECLINE FLOOR below fires. |

## 5. The DECLINE FLOOR, registered before any measurement

**If `INLINE-P` scores below 0.90 on GRID-2's discriminating cells (P8) or below
0.90 on the workload retrodiction (P4), this lane publishes the population table
and STOPS.** No repair of the constants, no new term fitted to the misses, no
second holdout. Five allocation keys have died this way and the lanes that
stopped are the ones whose results survived.

**And a second floor, on the other side:** if `INLINE-P` scores *above* 0.90 the
lane still ships **no port change**. The output is a spec
(`docs/INLINE_PREDICATE.md`) and board rows. Narrowing `IlBundle::functions()`
on a cost model is board #269/#844's standing hazard and w-fnbyte §8.1 already
declined it by name.

## 6. What this lane will NOT do

* **It will not re-derive SCHEDULE D.** The incumbent is prior art with section
  numbers; this lane grades it and cites it.
* **It will not touch `IlBundle::functions()`, `select_function`, or any
  emitter.** `git diff` on `crates/` is empty or tests-only, and P7 checks it.
* **It will not fit a new term to a workload miss.** A miss is published as a
  population, per §5.
* **It will not quote a listing where an obj byte is available** (#843), will
  not read a producer positionally (#644), and will not assert a frame-word
  count (#869) — frame words are printed, never asserted.
* **It will not glob `work/capture-cache` or `.claude/worktrees`.**
