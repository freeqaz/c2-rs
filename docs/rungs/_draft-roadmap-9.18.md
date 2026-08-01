# 9.18 W-EMITSET — the emit set is not predictable, the ceiling is 111, and the inliner is not the reason (2026-08-01)

DRAFT for `docs/ROADMAP.md` §9.18 — written by lane `w-emitset`, to be landed by
the coordinator. Nothing in §1–§9.17 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-emitset-prereg.md`, committed at `3b9a4ae` before the
first measurement. Base and tip `74d0744` + this lane's commits.

**Headline: TU match 6 → 6.** Measurement and one instrument; no codegen, no
emit-set model shipped, and shipping one would have been wrong. Four findings,
in descending order of how much they change what to do next:

1. **The emit set is not predictable from anything the census can see.** A cell
   table over *every* feature the instrument has — binding, mangling, access
   code, census key, control flow, EH, frame class, dispatch, production,
   completeness — fitted on 432 TUs and graded on a disjoint 432, scores
   **94.938 % per body against a 93.700 % never-emit base**, and **1 of 432 TUs
   exactly right — the same 1 the base predictor gets, and it is a TU that emits
   nothing.** The model is worth **1.24 pp of bodies and zero TUs**.
2. **The binding, not the codegen and not the inliner, is the ceiling — and it is
   111 of 871.** For a TU to be byte-exact the port must reproduce the reference
   `.text` COMDAT set, and it can only emit a COMDAT for a body it has under a
   name the binding gives it. **760 of 871 TUs carry at least one emitted symbol
   no census row claims.** §9.16.3's 25 is a count comparison on a model-free
   port; 111 is the ceiling on any *model*.
3. **The `/O1` inline-decline schedule does NOT gate emission, and this is
   measured at workload scale.** §8.1 called it "the least-derivable model in the
   program" and made it the reason not to attempt Phase 7. **58.6 % of every
   function c2 emits is ≤ 64 bytes** — `LABEL_COUNTER.md` §6.15.3's *unbounded*
   band, the callees c1xx inlines at every site. The median emitted function is
   **40 bytes, ten instructions**. §6.5's fixture result generalizes: c2 emits
   the fully-inlined callee anyway. **The hardest thing on the board is not on
   the critical path.**
4. **What the residue actually is has a name, and it is the polymorphic class.**
   Of the 13,646 emitted symbols with no readable `.gl` body record, **70.0 % are
   virtual members** and 47.6 % are the `??_` synthesized family — and the
   control holds: the *bound* population is 42.1 % virtual, so "virtual" is not a
   fact about mangled names in general. Restricted like for like to non-`??`
   names: **bound 42.1 % virtual, unreadable-record 98.8 %.**

---

## 9.18.1 The ceiling on a MODEL is a different number from §9.16.3's, and lower

§9.16.3 measured `.ex` segment count against obj `.text` COMDAT count and got
**25 of 871**. That is the ceiling on the port *as it stands* — one COMDAT per
segment, no model. It is a comparison of two integers.

A model has to reproduce the **set**, and it is constrained further:

> `PortC2::build` can only ever emit a COMDAT for a body this bundle carries,
> under the name the `.gl` binding gives that body. An emitted symbol no row
> claims is a COMDAT the port cannot produce at any codegen quality and under
> any predictor.

That residue was already published — **17,706 symbols, 9.89 % of the 178,968
denominator** (§9.9.3) — but only ever as a *total*. Per TU it is the binding
constraint, and it had never been read that way:

| | TUs of 871 |
|---|---:|
| every emitted symbol binds to a census row — **reachable today** | **111** |
| would, if `bind.rs` lost none of the records it already finds | **116** |
| carries ≥ 1 emitted symbol with **no** `.gl` body record this reader can find | **755** |

Six of the 111 emit nothing at all, so the non-vacuous figure is **105**.

**And the arithmetic control (#144 — residue 0 is not a control, add an arity
check).** Counting TUs with a residue is not the same as counting the residue's
contents: median unbound-per-TU **10**, mean **20.3**, max **192**, sum
**17,706**; for the no-record half alone, median **9**, mean **15.7**, max
**127**, sum **13,646**. **60 TUs carry exactly one no-record symbol.** A ceiling
reported only as "760 TUs blocked" would have hidden that a twelfth of them
block on a single symbol.

## 9.18.2 The split that had to exist before the ceiling could be read

An emitted symbol no row claims had been one number. It is two things that need
opposite work, and the ceiling is stated over the second:

* it has a framed `.gl` body record — **the body is in this bundle and
  `EmitBinding` lost the row.** An instrument defect, closable in `bind.rs`.
* it has none — **a wall**: a segment-driven port must synthesize the COMDAT.

`c2_il::gl_body_record_names` reports every name owning a framed body-start
record, with the *same* framing and the *same* name-distance bound as
`EmitBinding::new`, deliberately — so a difference between the two answers is a
difference in the **binding**, never in the reader. Diagnostic only: the gate,
the census verdict and the emitter do not consult it, and every published number
is byte-identical with it armed (§9.18.7).

```
17,706 unbound emitted symbols
   4,060  have a body record   — instrument defect (bind.rs)
  13,646  have none            — the wall, as this reader sees it
```

**The two tests are on the cases that discriminate, per #145.** A row two
records collide on binds nothing and must still report *both* names — otherwise
the ceiling reads a binding collision as "c2 emitted a symbol with no body" when
the body is right there. And a symbol with no record must **not** be invented —
otherwise `emit-unbound-no-record` is 0 by construction, which is the
absence-read-as-success shape exactly.

## 9.18.3 The wall is the polymorphic class, and the control could have failed

**Key names lie, so this went to the byte.** `mangling_class` reports 47.7 %
`special-generated`, which is *every* `??_…` — and `??_` is `??_G`/`??_E`/`??_D`
(real synthesized functions) as well as `??_7` (vftable), `??_R0`…`??_R4` (RTTI)
and `??_C` (string literals), which are data. A decomposition that never prints
a name cannot tell those apart, and the whole reading rests on which it is. The
names, by exact prefix:

| | count | share of 13,646 |
|---|---:|---:|
| `??_G` scalar deleting dtor (synthesized) | 4,862 | 35.6 % |
| `??1` destructor | 3,370 | 24.7 % |
| `?` ordinary member/free function | 2,755 | 20.2 % |
| `??0` constructor | 794 | 5.8 % |
| `??_D` vbase dtor iterator (synthesized) | 587 | 4.3 % |
| `??_E` vector deleting dtor (synthesized) | 582 | 4.3 % |
| `??__F` dynamic atexit dtor (synthesized) | 379 | 2.8 % |
| `??` operator | 189 | 1.4 % |
| `??_H`, `??_F`, `??__E`, other | 99 | 0.7 % |
| undecorated | 1 | 0.0 % |

No RTTI, no vftables, no string literals — the `.text`-COMDAT-function filter
already excluded them, and printing the names is what established that rather
than assuming it. **6,508 (47.6 %) are genuinely synthesized**; the other 7,138
are real user functions with real bodies.

**And what those user functions have in common is virtualness.** MSVC's access
code sits immediately after the `@@` closing the qualified name; virtual is
`{E,F,M,N,U,V}`. The **bound** population is the control, and it could have
refuted this outright:

| population | n | virtual | non-virtual member | static | free |
|---|---:|---:|---:|---:|---:|
| bound (control) | 89,700 | **42.1 %** | 29.5 % | 4.9 % | 22.3 % |
| unbound, has record | 3,459 | **2.4 %** | 52.6 % | 0.3 % | 35.2 % |
| unbound, **no record** | 2,756 | **98.8 %** | 0.8 % | 0.0 % | 0.3 % |

(non-`??` names only, so the comparison is like for like.) Over the whole
no-record class, **9,553 of 13,646 = 70.0 % are virtual**. The bound control at
42.1 % says this is not a property of mangled names; the has-record column at
2.4 % says it is not a property of being unbound either. It is a property of
**this reader**, and the byte-level reason is visible in
`src/system/obj/TextFile.cpp`: a virtual member's `.gl` record carries extra
material between the name and the offset field —
`?Print@TextFile@@UAAXPBD@Z\0 82 07 05 00 00 20 01 04 02 93 45 dd 20 80 a3 22`
against a non-virtual `??0DataNode@@QAA@H@Z\0 86 03 05 04 20 00 02 01 00 80 …` —
so the `80 <LE32> 00 00` framing and the 32-byte name-distance bound lose it.

## 9.18.4 The ceiling ladder — what each repair is worth, in TUs

Stated as **ceilings under named repairs, not as achieved results.** §9.16.1
records what happens when a board's payoff field and its outcome field are the
same field; everything below the first row is a counterfactual and must never be
written back as a status.

| | TUs of 871 | delta |
|---|---:|---:|
| **measured today** — every emitted symbol binds | **111** | — |
| + repair the ROW binding in `bind.rs` (the 4,060 has-record symbols) | 116 | +5 |
| + read the virtual member's `.gl` record shape | 204 | +88 |
| + synthesize the `??_` family (no `.ex` body exists) | 238 | +27 |
| + **both** of the last two | **436** | +325 |
| after both, still blocked | 435 TUs, 1,797 symbols | |

**The +5 is the surprise and it is the useful one.** Repairing the row binding —
the residue this project has been reporting for weeks — buys **five TUs**. The
work that matters is the *record reader* (+88) and COMDAT synthesis (+27), and
they are worth **+325 together**, far more than the sum of their parts, because
most blocked TUs carry both kinds.

**What is left after both, by name**, and it is a third compiler-generated
population that no `??_` prefix marks: `??1?$_STLP_alloc_proxy@…@@QAA@XZ` (389),
`??1?$ObjDirItr@…@@QAA@XZ` (161), `??0bad_alloc@std@@QAA@ABV01@@Z` and
`??0logic_error@…@@QAA@ABV01@@Z`-shaped copy constructors (66 each). These are
**implicitly-declared special members** — an implicit copy constructor or
destructor is mangled exactly like a user-written one, so no prefix separates
them. That is the honest open end of this decomposition and the next lane's
first question.

## 9.18.5 The predictor, fitted and graded — and the arithmetic that dooms it

Split 864 TUs (the ones that emit anything) into two disjoint halves **by TU**,
432 fit and 432 grade; every threshold and every cell fitted on the fit half
only. The model is a **cell table**: partition rows by a feature cross, take each
cell's majority label on the fit half, apply to the grade half. That is the most
favourable non-parametric predictor over those features — if it cannot separate,
no simpler rule over the same features can.

| model | cells | held-out per-body | TP | FP | FN | TUs all rows right |
|---|---:|---:|---:|---:|---:|---:|
| **P0 never emit (BASE)** | 1 | **0.93700** | 0 | 0 | 80,479 | **1** |
| has-name | 2 | 0.93700 | 0 | 0 | 80,479 | 1 |
| + mangling + access code | 28 | 0.93701 | 13 | 2 | 80,466 | 1 |
| + census key | 1,527 | 0.94651 | 14,808 | 2,661 | 65,671 | 1 |
| + cflow / EH / frame | 3,319 | 0.94900 | 18,808 | 3,482 | 61,671 | 1 |
| + dispatch / production / completeness | 3,674 | **0.94938** | 18,823 | 3,013 | 61,656 | **1** |

**Per-TU exact set — predicted emitted names == reference emitted names — is 1
of 432 for every model, including the base.** That TU is `src/system/decomp_pch.cpp`
and it is the **only** held-out TU with zero emitted rows, verified by name: the
best model gets **zero TUs that emit anything**. The best model recovers 18,823
of 80,479 emitted functions (23.4 % recall) and invents 3,013.

**And the per-TU figure is scored generously, deliberately.** The reference set
it is compared against is the *bindable* emitted set — the 17,706 symbols of
§9.18.1 are not in it, because no row carries them. So the grading pretends the
ceiling problem away and the model still scores 1 of 432. Scored against the
real `.text` COMDAT set it would score 1 of 432 as well, but for two independent
reasons instead of one.

**C-leak, registered and it bit.** 1,134,139 rows (46.1 %) carry no bound name
at all, so they are `not-emitted` **by binding failure, not by c2's decision**,
and every model banks them for free. Restricted to the 1,328,432 named rows the
positive rate is 12.14 %, not 6.55 %.

**And a correction to a published headline that follows from the same fact.** §8.1's
denominator is 178,968 emitted against 2,462,571 bodies = 7.27 %. The rate a
segment-driven model can actually *see* is **161,262 / 2,462,571 = 6.55 %** —
the other 17,706 are not reachable from any segment. Any statement of the form
"the port need only decide 7.23 % correctly" is 0.72 pp optimistic, and the
missing 0.72 pp is exactly the population §9.18.1 shows is the ceiling.

**E10's arithmetic, which was registered before any of this and is the reason
per-body accuracy is not a headline.** Median 2,136 rows per held-out TU:

| per-body accuracy | expected TUs all-right, of 432 |
|---|---:|
| 0.99 | 0.00 |
| 0.999 | 51.0 |
| 0.9999 | 348.9 |
| 0.99999 | 422.9 |

At the measured **0.94938** the expectation is `432 × 0.94938^2136 ≈ 10⁻⁴⁶`.
**Three nines is the entry price and the instrument is two orders of magnitude
away.** Reporting "94.9 % accurate" without this table would be the single most
misleading sentence this lane could have written.

## 9.18.6 The inliner is not the reason — measured, with a control that could have failed

§8.1 declined to pre-decide *lower everything* versus *model the emit set*, and
the stated reason was that the emit set's "inliner half is the least-derivable
model in the program (`LABEL_COUNTER.md` §6.15.3: the `/O1` inline-decline
schedule is measured exactly and *generated by no formula*)."

**The premise is false, and the test is one line of arithmetic on the objs.**
The schedule's axis is `s`, the callee's own emitted `.text` size, and its top
band is `s ≤ 64` bytes (≤ 16 instructions) = *inlined at every site, unbounded*.
§6.5 claims on fixtures that c2 emits the callee's COMDAT anyway. If emission
were inline-gated, the `s ≤ 64` band would be the **rarest** among emitted
functions — every member of it was inlined everywhere and needs no out-of-line
copy. If §6.5 generalizes it should be one of the commonest.

Every `.text` COMDAT of 25 TUs, taken as every 36th line of the workload list so
it is a spread and not a hand-pick — **4,490 emitted functions**:

| `s` band | schedule | emitted | share |
|---|---|---:|---:|
| **≤ 64** | **inlined at EVERY site** | **2,632** | **58.6 %** |
| 68–72 | 9 sites | 145 | 3.2 % |
| 76 | 7 sites | 136 | 3.0 % |
| 80 | 5 sites | 57 | 1.3 % |
| 84–88 | 4 sites | 215 | 4.8 % |
| 92–100 | 3 sites | 165 | 3.7 % |
| 104–140 | 2 sites | 304 | 6.8 % |
| 144–256 | 1 site | 587 | 13.1 % |
| ≥ 260 | never inlined | 249 | 5.5 % |

Median emitted function **40 bytes = 10 instructions**; the commonest sizes are
**8 (578×), 4 (455×), 12 (355×), 16 (220×)** — one- to four-instruction getters,
every one of them a callee c1xx inlines at every site. Sanity control: **0 of
4,490 sizes is not a multiple of 4.**

**So the inline decision does not enter the emit predicate.** It is also
structurally impossible that it could: §6.15.1 measured that the *front end*
decides inlining, once, per (caller, callee) pair, so by the time c2 sees the IL
the inlining is done; and the schedule's axis `s` is the callee's **emitted**
size, which does not exist for a body that is not emitted — the axis is
undefined on exactly the population whose emission is in question.

**This is the lane's best news and it should be read as a re-plan, not a
reassurance.** §8.1's stated reason for leaving Phase 7 undecided is retired.
The reason to be cautious about Phase 7 is now §9.18.1's 111 and §9.18.5's
1-of-432 — both of which are about the *binding* and about *synthesis*, and
neither of which is undecidable.

## 9.18.7 What the residue does NOT decompose into, and why that is a result

The brief asked for the residue decomposed by c2's own named disjuncts —
`globally unreferenced`, `has linear flow`, `is a redirector function`,
`won't be inlined (too big)`, `inlining prohibited`, `unreferenced import`,
`InlBadCandidate`. **This lane could not do it and no lane can with the
instruments that exist.** Those strings live in `c2.dll`'s string table; §9.5
already established with a positive control that there is no switch that dumps
them (~25 candidate flags return `C1007 unrecognized flag … in 'p2'`), and the
`.cod` listing prints assembly, not inline decisions. **There is no per-body
disjunct label anywhere in this project's reach.**

What *is* measurable is a structural partition of the 2,301,309 not-emitted
bodies, and its shape is the answer to the same question:

| | bodies | share |
|---|---:|---:|
| segment with **no bound name** — the binding cannot say what it would be called | 1,134,139 | 49.3 % |
| named row c2 did not emit | 1,167,170 | 50.7 % |

**and the second half does not decompose further.** The cell table of §9.18.5
partitions it 3,674 ways on every axis the census owns and moves accuracy
**1.24 pp**. That is the registered refuter for E5 firing: *a flat decomposition
⇒ no cheap clause exists.* There is no 90 %-and-cheap clause. There is no 40 %
one either.

## 9.18.8 Fail-closed wiring — how a model would ship without ever guessing a COMDAT

Nothing here shipped. This is the design the next lane inherits, and the point of
writing it now is that **the shape is forced**, not chosen.

1. **The decision is per TU, not per body.** `PortC2::build` already emits a
   whole obj or nothing. The model must be **total**: every `.ex` segment gets
   `Emit(name)`, `Skip`, or `Unknown`, and **one `Unknown` refuses the TU**.
   There is no partial credit — a TU with one wrong COMDAT is a mismatch, and a
   mismatch outranks every other outcome.
2. **`Emit` requires a positively bound name from the GATE binding**
   (`Bindings::per_record`), never from `EmitBinding`, which is the diagnostic
   one and is deliberately looser. A body emitted under the wrong name is a
   relocation against the wrong symbol — a mis-emit, not a gap. Today
   `emitset-unnamed-segment` alone would refuse essentially every TU (46.1 % of
   segments are unnamed), and **that is correct behaviour**: it is the model
   telling the truth about §9.18.1's ceiling instead of guessing past it.
3. **Refuse on the presence of a construct, never on the absence of a symbol.**
   The port cannot check "c2 emitted a `??_G` I do not have" — it would have to
   know the name already, and it only knows the reference's names at *grade*
   time, which is the oracle. The port-side gate has to be positive: *this TU
   declares a class with a virtual destructor* / *a namespace-scope object with a
   dynamic initializer* → refuse until a synthesis phase exists. Refusing on an
   absence is the failure mode this document records twelve times.
4. **The byte compare DOES grade the emit set, and that is the strongest reason
   to build it behind the existing gate rather than behind a new classifier.** A
   wrongly skipped body shortens the obj; a wrongly emitted one lengthens it;
   either diverges at COFF offset 2 (`NumberOfSections`) or 8
   (`PointerToSymbolTable`) long before any instruction byte matters. **But
   #149/§9.17's coverage bound applies with full force**: today's 878-TU scan
   reads 0 mismatch because 865 TUs refuse before reaching the emitter, so the
   scan cannot see an emit-set defect at all. A dedicated probe is owed —
   fixtures carrying a header inline that **is** emitted beside one that is not,
   on a TU the port compiles whole.
5. **The set is not enough; the model owes an ORDER.** COMDAT section order,
   symbol-table order, and the per-TU label-counter surcharge (`LABEL_COUNTER.md`
   §1.1) are all functions of the emitted sequence. A model that gets the set
   right and the order wrong is still a mismatch. The `.cod` listing seam
   (§9.1–§9.4) is where the order is readable symbolically, and §9.3 already
   records that the label counter's phase order is **not** text order.

## 9.18.9 Pre-registration, scored — 6 of 10, and the misses are the lane

Registered in `docs/rungs/_2026-08-01-w-emitset-prereg.md`, committed at
`3b9a4ae` before the first measurement. Declared bias: **borrowed and
structural** (§6.5 was read first) and pessimistic on E4.

| # | claim | est | interval | actual | score |
|---|---|---|---|---|---|
| E1 | share of `.ex` segments with a bound name | 25 % | [10, 45] | **53.94 %** | **MISS** — above the ceiling |
| E2 | P0 held-out per-body accuracy | 92 % | [85, 99] | **93.700 %** | HIT — but see below |
| E3 | P0 held-out per-TU exact-set, of 871 | 2 | [0, 40] | **1** (of 432 held out) | HIT |
| E4 | best predictor's TUs made emit-set-reachable | 60 | [5, 300] | **~2** | **MISS** — below the floor |
| E5 | largest single disjunct's share of not-emitted | ≥ 80 % | [40, 99] | **not measurable**; nearest analogue 50.7 %, decomposition FLAT | **MISS** |
| E6 | cost of the inliner clause, in TUs | 0 | [0, 100] | **0** | **HIT** |
| E7 | unbound emitted share, re-measured | 9.9 % | [8, 12] | **9.89 %** | HIT |
| E8 | `.gl` linkage byte takes ≥ 2 values ≥ 1 % among bound records | YES | — | true on one TU; **never measured at workload scale** | **MISS (process)** |
| E9 | TUs converted by this lane | 0 | [0, 0] | **0** | HIT |
| E10 | per-TU far worse than per-body implies | YES | — | 0.94938 per body → 1 of 432 | HIT |

**6 of 10, and three of the four misses carry the lane's value.**

* **E4 is the important miss and it is low, which is the useful direction.** I
  registered 60 TUs with a floor of 5, having read §9.16.3's 25 and expected a
  model to beat a model-free port. It does not beat it — it does not beat
  *nothing*. That is what makes §9.18.6 a re-plan rather than a ranking tweak.
* **E1 was wrong by more than the interval's whole width**, and I had the
  arithmetic to get it right before estimating: the scan already printed
  `emit-records 1,515,160`, `nameless 152,941`, `row-conflicts 33,552`, which
  subtract to 1,328,432 of 2,462,571 = 53.9 %. **I registered a guess where a
  subtraction was available.** Same class as §9.16.7's E8 — a borrowed prior
  used in place of a number already in the report.
* **E2 is a hit whose refuter fired.** I registered "below the 92.77 % never-emit
  base rate ⇒ worse than nothing". The measured base is **93.700 %**, not
  92.77 %, because 92.77 % was computed from `emit-emitted`/bodies and the
  reachable positive rate is `emit-bound`/bodies = 6.55 %. P0 did not fall below
  the base — it **tied it exactly, TP = 0**. A registered refuter stated against
  a stale constant would have passed a predictor that predicts nothing.
* **E8 is a process miss and is reported as one.** I registered an estimate and
  then never built the instrument that would have graded it, because §9.18.5 made
  the answer irrelevant. *Irrelevant is not measured*, and the honest score is a
  miss.

## 9.18.10 Gate evidence

| lane | base `74d0744` | tip |
|---|---|---|
| `cargo test --workspace --release` | **596 passed, 0 failed, 1 ignored, 24 targets** | **598 passed, 0 failed, 1 ignored, 24 targets** |
| `#[test]` grep over `crates/` | **597** | **599** (+2, both new) |
| `scripts/gate.sh --jobs 6` | — | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **2,520 fixture-verdicts**, 0 mismatch in every lane |
| `c2rs selftest` | — | **210 PASS, 0 FAIL** |
| 878-TU workload scan | match 6, mismatch 0, codegen-gap 0, vocab-gap 865, capture-fail 7 | **identical** |
| census | 706,402 / 2,462,571 (28.69 %) | **identical** |
| emitted census | 36,059 / 178,968 (20.15 %) | **identical** |
| census/gate disagreement | 0 | **0** |
| distance (bodies) | ≤0: 1, ≤1: 10, ≤10: 25, ≤100: 32, ≤1000: 210 | **identical** |
| distance (emitted) | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 399, ≤1000: 857 | **identical** |
| emit-set ceiling (§9.16.3) | 25 of 871, violations 0 | **25 of 871, violations 0** |
| emit-set MODEL ceiling | not measured | **111 today / 116 repaired / 755 wall** |

**Target count recorded beside test count**, per §9.16.8: 24 at base and 24 at
tip. `cross_sweep` not run — **no codegen was touched.** The diff is
`c2-il/src/func/bind.rs` (one `pub fn`, two tests), two re-exports, and
`c2-harness` (`gap.rs` accounting + a scratch dump, `main.rs` two report lines).
`PortC2`, `codegen` and every recognizer are untouched.

**Instrument inertness, asserted by running it rather than argued.** The full
878-TU scan was run three times — plain, with the wall dump armed, and with the
per-row dump armed — and census, emitted census, match, mismatch, disagreement,
both distance ladders and the §9.16.3 ceiling are byte-identical across all
three. The dumps' own totality checks are the second half of that: the wall dump
emits **178,968** lines, exactly the emitted denominator, and the row dump
**2,462,571**, exactly the census denominator.

**Environment caveat, inherited and restated:** this box's `wibo` is `1.0.1-7`,
older than the known-good `1.0.1-23`. Every scan here used `--replay-every 0` and
this lane quotes no replay number; census and mismatch counts are byte-identical
under both loaders, which is what the scan's own warning says.

## 9.18.11 Found and not taken, ranked

1. **The virtual `.gl` record shape** (§9.18.3) — worth **+88 TUs** of ceiling on
   its own and **+325 with COMDAT synthesis**, it is a *format* job with a
   byte-level witness already transcribed, and it is the largest single number
   this lane found. It is `bind.rs` and `gl.rs`, not codegen. **Rank 1 by a wide
   margin.**
2. **COMDAT synthesis for the `??_` family** — 6,508 symbols, 47.6 % of the wall,
   and `??_G` alone is 4,862. A scalar deleting destructor is
   `{ this->~T(); if (flags & 1) operator delete(this); }`, a shape small enough
   to be a fixture. It also has the cleanest possible first probe: §9.16.10
   already named `TomCryptLicense` / `ZlibLicense`, **zero `.ex` bodies and one
   emitted COMDAT each**.
3. **The implicitly-declared special members** (§9.18.4) — 1,797 symbols across
   435 TUs that no prefix marks and that this lane could only name, not classify.
   Whoever takes rung 1 will meet them immediately.
4. **§8.1's "least-derivable model" clause should be struck**, not softened.
   §9.18.6 refutes its premise at workload scale with a control that could have
   failed. Leaving it in place mis-ranks Phase 7 for the same reason §9.16.6
   found Phase 6 mis-ranked: the sentence is load-bearing and nobody re-measured
   it.
5. **The 7.23 % figure should be published as 6.55 %** where it is used to size
   the emit-set decision (§9.18.5). The difference is precisely the unreachable
   population, so quoting the larger number understates the very constraint it is
   quoted to describe.
