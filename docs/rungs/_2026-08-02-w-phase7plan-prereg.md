# w-phase7plan — pre-registration

Written and committed **before any measurement of the registered quantities**.
Base `9b85457` (verified `git log -1` at session start), worktree
`wt-w-phase7plan` off `master`, toolchain verified non-`SKIP` by
`scripts/setup_worktree.sh` (`w5_chain.cpp` → 4/4 in class).

**This lane writes no code.** Deliverables: `docs/PHASE7_PLAN.md` plus this
prereg, scored. Measurement is delegated to subagents; every number they return
is graded against the estimates below, which are committed first.

## Lane premise

§10.2: the port has no emit-set model, so TU match is capped at 25/871 (6
taken) until Phase 7 exists. §10.3: prediction over census features is refuted
(0.94938 per body, 1 TU vs baseline 1); widening is refuted by construction;
the supported route is **emit-set-as-binding-and-synthesis**. §8.1's stated
reason for deferring Phase 7 (the inline schedule) is REFUTED and struck.
§10.15: the 25 is `LO`-anchored against a `4F 1F`-anchored property; the
disagreement is unsigned and a sibling lane is computing the gate-side count.

## The hypothesis this lane brings, stated before testing it

The §9.18.5 predictor's feature set contained **no linkage and no reference
information**. MSVC's documented model — and c2's own diagnostic strings
(`globally unreferenced`, `unreferenced import`, `is a redirector function`) —
suggest the emit predicate is **linkage ∪ reachability**:

> a function with an `.ex` segment is emitted iff it has non-COMDAT ("strong")
> linkage, OR it is a COMDAT-linkage function referenced (transitively) from
> the TU's emitted closure (emitted functions, emitted data, vtables,
> `.CRT$XCU` initializers).

Corollary hypothesis: the `.gl` name separator already encodes the linkage
half — `00`-introduced ≈ strong (always emit), `26`-introduced ≈ COMDAT
(emit iff referenced). §9.20.2/§9.20.5 support the correlation ("`??_G`… are
`26`-separated *because* they are COMDAT-linkage"; out-of-line virtual is
`00`-separated and bound, inline is `26`-separated) but nobody has tested it
as an emit predicate.

## Declared bias

**Inflationary / optimistic, and strongly.** The linkage∪reachability
hypothesis is this lane's own synthesis; I want it true because it makes
Phase 7 plannable. Every estimate below that bears on it is therefore at risk
of being read generously. Guards:

1. E2 and E3 are registered with refutation floors that would kill the
   hypothesis outright, not soften it.
2. E6 (the conservative sub-model's worth) is the number most likely to be
   inflated by my bias; its interval is deliberately wide and its **point is
   below its midpoint**.
3. The incumbent is registered wherever one exists: the never-emit baseline
   (1 TU) for any predictor claim; §9.18.5's 0.94938 for any per-body claim;
   the 324/420/451 ceilings for any ceiling claim. A proposal that does not
   beat its incumbent is a decline.
4. "A count is only evidence about the predicate that produced it" — every
   count a subagent returns must name its predicate (which splitter, which
   scanner, which flags). Numbers at `/Ox` do not transfer to the `/O1`
   workload (coordinator note, 2026-08-02: `/O1` implies `/GF`, `/Ox` does
   not, and the obj shapes differ structurally).
5. The listing is name-level truth only. PROC/PUBLIC sets are usable
   (#136: 0.0000 pp error on the denominator); instruction bytes,
   displacements, and **section order** are not (lane w-objshape measured the
   listing's section order disagreeing with the `/O1` obj's). Any rule read
   off a listing requires an obj cross-check before it enters the plan.

## Registered estimates

| # | claim | point | interval | refuted by |
|---|---|---:|---|---|
| **E1** | Emission of an unreferenced header-inline (COMDAT) function is **reference-sensitive**: across a probe grid of minimal pairs (identical TU ± one reference to the candidate), the emitted set flips with the reference | flips in **≥ 9/10** pairs | [7/10, 10/10] | ≤ 5/10 ⇒ the reachability half is wrong and the plan's spine with it |
| **E2** | Strong-linkage functions are emitted unconditionally: fraction of `00`-introduced framed `.gl` body-record names that appear in their TU's emitted COMDAT set, over sampled workload TUs | **95 %** | [80 %, 100 %] | < 60 % ⇒ the separator does not encode the linkage half |
| **E3** | The converse leak is small: fraction of emitted `.text` COMDATs that are `26`-introduced or record-less (i.e. NOT explained by "strong ⇒ emit") — the population the reachability half must earn | 55 % | [30 %, 80 %] | — (a shape measurement; no refutation floor, but > 80 % means the strong rule alone is nearly worthless) |
| **E4** | "Emit iff a framed `.gl` body record exists" (record-existence alone, no separator split) is **rejected as a model**: its per-TU exact-set score on sampled TUs | **≈ 0 TUs** beyond the empty-emit ones | [0, 5 %] | a high score here would be wonderful and is not expected: 1.5 M framed records vs 179 k emitted |
| **E5** | TUs of 871 where the emitted COMDAT set **exactly equals** the `00`-introduced framed-body-record name set (the zero-reachability conservative sub-model, "strong-only TUs") | **150** | [40, 400] | < 20 ⇒ no cheap conservative sub-model exists on this axis and the plan must say so |
| **E6** | c2 can be made to *name* its skip decisions through a black-box channel (a warning level, a flag, or a listing annotation naming unreferenced-removed functions, e.g. C4505-shaped) | **yes, at least one channel** | — | all channels dry ⇒ the predicate must be fitted purely from emitted-set diffs, which raises the probe-grid cost in the plan by ~3× |
| **E7** | The `.gl` record *contents* (bytes between name and body offset) carry a per-function field that separates emitted from not-emitted within one TU — i.e. c1xx already told c2 | **no such single field** | — | finding one would be the cheapest possible Phase 7 and would restructure the plan; registered as *not expected* so finding it is a genuine surprise, not confirmation |
| **E8** | The gate-splitter (`4F 1F`) recount of the 25 (sibling lane owns it): the recomputed reachable count lands | 27 | [20, 40] | outside ⇒ the ceiling is even less stable than §10.15 says; the plan must carry both counts |

## Method constraints, fixed in advance

* All probes at the workload's flags `/O1 /Oi /EHsc` (`/GS-` where the
  harness default says so). `/Ox`-only evidence is marked non-transferable.
* Ground truth for "emitted" is the **obj** `.text` COMDAT set; the listing's
  PROC set may be used as a cross-check and name source, never as sole truth.
* Structural axes crossed before values varied: linkage (inline / static /
  extern / virtual / template), reference kind (call / address-taken / vtable
  / initializer / none), reference position (same function / other function /
  dead function), class polymorphism, and TU-level (single vs multi object).
* Any fitted predicate is graded **out of sample**: predictions committed as
  a git object before the held-out set is compiled, decline floor stated in
  advance.
* Subagents write nothing outside `work/` scratch; no captured IL, objs, or
  absolute paths are committed; no `pgrep -f` self-matching watchers; every
  wait bounded.
* Corpus analysis reads the existing capture cache and cached reference objs;
  where a reader must re-derive a rule the harness owns (§10.14's trap), its
  first output is a known-answer check against the harness on a named TU.

## What this lane will NOT claim

* Not that any TU converts. Every number here is about *reachability* and
  *model ceilings*; §8.1's precedent (census 6.4×, TU match flat) is the
  standing reason those are different quantities.
* Not a realized ceiling. 324/420/451 are ceilings on a model that does not
  exist; this lane plans the model and does not build it.
* Not that the emit predicate, once fitted on probes, holds on the workload —
  that is the out-of-sample gate the plan itself must carry.
