# w-memcpy — PRE-REGISTRATION

Committed **before the first probe cell is generated**. Lane `w-memcpy`,
worktree branch `wt-w-memcpy` off master `85e180d4`.

The rung: the **two reader clauses** board #1444 names as standing between
`?mmioGetInfo` and a byte-exact obj, now that lane `w-mmio` has shipped the
entry-block park —

    callseq-multiarg-lit        (its `li r5,72`)
    expr-intrinsic-memcpy       (its `bl memcpy`)

---

## 0. What was re-derived from the reference obj BEFORE writing this

Per the standing instruction (#1401, and `w-loop`'s four-of-five-absent
inventory), nothing below is carried from the record; every item was read this
session off `work/w-memcpy/ref/mmio.obj` and the IL bundle beside it, both
produced by the real `c2.dll` under wibo at the workload's own flags and cwd.

* `?mmioGetInfo` is **84 B**, `.text` section #5, one REL24 at `0x3c` naming
  symbol **[19] `memcpy`**, `sc=EXTERNAL sec=0 type=0x0020`.
* The source is `MMRESULT mmioGetInfo(HMMIO, LPMMIOINFO, UINT)` — **three**
  formals, the third (`fuInfo`) **unused** — and the call is
  `memcpy(pmmioinfo, hmmio, 0x48)`, i.e. `memcpy(formal1, formal0, 72)`.
* So the call's slot list is `[Formal(1), Formal(0), Lit(72)]`: a **2-cycle of
  formals with a literal beside it**. The park (`mr r11,r3 ; mr r3,r4`) takes
  the ascending half of the cycle into the entry block; `li r5,72 ; mr r4,r11`
  is the descending remainder at the call.
* **NEW, and on no board row I could find**: `memcpy` **does not occur in the
  `.gl` name stream at all** (checked: the stream's 60 printable names are the
  eleven `mmio*`, `?FreeHandle@@YAXPAX@Z`, `.XBLD$W`, `__C1_11886`, the
  `/include:` directive and typedef names). c2 **mints** the name. Every other
  callee this port emits comes from a `.gl` token through `bundle::resolve`.
* **NEW, second item**: the IL's memcpy arguments each carry a `2C` conversion
  (`b9 <tok> <TYPE> 2C <TYPE> 00 55 <TYPE>`), and the argument region carries
  **two alignment hints** (`01` and `04`) ahead of the size literal, neither of
  which has a counterpart in the source or in the emitted call.
* The IL body is fn #0 of the bundle, `.ex` `[3203, 3489)`, 286 bytes, and the
  census stops at the **first `==`** (`expr-cmp-eq`) — board #1416's
  fall-through key, naming none of the three refusals.

---

## 1. The registered predictions

| # | prediction |
|---|---|
| **P1** | **`?mmioGetInfo` does NOT go byte-exact in this lane, and `mmio.cpp` converts nothing.** TU match `11 → 11`. This is a **rung**, and it is registered as one before any measurement, exactly as `w-mmio` registered its own. |
| **P2** | `mmio.cpp`'s distance is **316 of 380 bytes** at both ends, `8 / 11` functions in class at both ends — unchanged **to the byte**. |
| **P3** | **`callseq-multiarg-lit` is the cheap clause and `expr-intrinsic-memcpy` is not.** I register the asymmetry rather than a pair of numbers: the literal clause is a *slot-kind* widening over machinery that already exists on the tail-call side (WLA), and the intrinsic clause needs a callee **name that is in no IL stream**. |
| **P4** | **The literal's `li` takes its place in the framed sequence call's marshalling by DESCENDING DESTINATION**, the same rule `permute_args_parts` already uses for the tail-call form and the same rule `w-mmio` §2.1 read off the `lit` cells. The rivals are R2 *literals last*, R3 *literals first*, R4 *dependency order*. |
| **P5** | **The park does not change where the `li` goes.** The park is an entry-block phenomenon and the literal is a call-site one; a formal that the park has already moved does not pull the literal with it. |
| **P6** | **The generator will find ≥ 20 cells on which R1 and at least one rival disagree**, and will refuse to write the grid otherwise. |
| **P7** | **`expr-intrinsic-memcpy` is ≥ 4 independent refusals**, not one: (a) the `40` call head is not a call head to `eat_call_head`; (b) the callee has no `.gl` token, so `bundle::resolve` has nothing to resolve; (c) the two alignment hints and the size literal are three argument slots the emitted call does not have, so the 5→3 reduction is a rule; (d) each pointer argument carries a `2C` the slot classifier does not see through. |
| **P8** | **There is a fifth refusal nobody has written down: `memcpy` is not always a `bl`.** `docs/IL_INTRINSIC_CALL.md` §1.3 records that `Dir.cpp` fn931 pushes hint `04` where the fixture pushes `01` "and the expansion changes with it", and §3 records the *call* form only. I predict c2 **inlines** the copy below some size threshold and that the threshold and the hint interact — so admitting selector 172 on the strength of the id would be a wrong emit, which is §5.1's argument one selector over. |
| **P9** | `mismatch` is **0 at both ends**, on the workload and on every probe grid. Anything else is an alarm, not a result. |

## 2. The decision rule, registered so it is not chosen after the fact

* Clause A (`callseq-multiarg-lit`) **ships only if** a grid frozen before the
  first `cl.exe` scores one rule at 100 % of its in-class cells, and only over
  the sub-class that rule was **not re-fitted on**. A clause re-fitted by the
  population that refuted its predecessor is board #260 and comes out as a gap
  (`w-mmio` §3, and it declined its own third fit on exactly this ground).
* Clause B (`expr-intrinsic-memcpy`) **ships only if** P8 comes back false —
  i.e. the expansion is a `bl` over the whole grid — **and** the callee-name
  and 5→3 questions each have a measured answer rather than a fitted one.
  Otherwise it is **priced and declined**, and the price is the deliverable.
* **Acceptance goes in the IL parser** or not at all (board #139). If the
  emitter cannot lower something the reader admits, the reader refuses it too,
  and `fn_gate_refusals` empty at the tip is the check.
* **`mismatch 0` is the sole judge.** No disassembly reading is evidence
  (`w-mmio`'s third clause read plausibly and cost 155; `w-lineage`'s read
  0-wrong-of-30 and was 11 of 30).

## 3. Direction I expect to be WRONG in

**PESSIMISTIC**, and this is the minority call — board #770 stands at
twelve-of-thirteen **optimistic**, so registering the other direction is a
prediction that can be scored against a strong base rate.

The argument for it: this lane inherits a *floor that was just lifted*. `w-mmio`
shipped the park an hour before this lane started, and everything a floor was
hiding is still underneath — which is `w-mmio`'s own §6.1, where the direction
was right and the reasoning was wrong. What I expect to be wrong about
specifically is **the size of clause A**: I have registered it as "the cheap
one" on the strength of WLA existing, and the thing WLA does not have is the
interleave with `plan_saved_gprs`' hoist/trail rule and with a previous `bl`'s
result save — which is precisely the sentence `seq_call_arg_sources`' own doc
gives as the reason for the refusal. If A is dear, it is dear there, in Class B,
and the grid must cross it or it will not find out.

Registered failure modes:

* **F1** — the descending rule is right for the tail-call form and wrong the
  moment a callee-saved copy is in the same block. This is the most likely
  decline for clause A.
* **F2** — the literal's position depends on the *guard*, the way `w-mmio`'s
  anchor turned out to. Registered as unlikely (P5) and therefore the more
  informative if it fires.
* **F3** — clause A ships, converts nothing anywhere, and the honest report is
  a rung with +0 on every workload number. Registered as the **expected**
  outcome, not as a failure.
