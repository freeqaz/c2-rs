
## 10.30 W-READPX — §10.27's item 2 is PRICED, and the reader is not a lever either: the frontier's column is 41 not 48, seven of the eight departures are reach-1 transcriptions, and no reader rung converts a TU (2026-08-09)

§10.27 closed with an ordering whose item 2 was **"reader admission at the
frontier's 48, which is now the binding constraint on everything above"**, and
§10.27.1's replication left it there. Nobody had priced it at this tip. It is
priced ([`rungs/2026-08-09-w-readpx.md`](rungs/2026-08-09-w-readpx.md), board
**#2280**–**#2293**), and the answer is a decline — the fourth in a row on the
same question, after `w-jump` (#2007), `w-callprice` (#2025) and `w-band`
(#2242).

**The 48 is 41, and the delta is not −7.** `WB_READER_FINDINGS.md` §1 measured
48 reader-refused frontier functions at `c34c388c` this morning over 16 TUs.
At this tip `gap-metric frontier-codegen-reader` reads **41** over **9** TUs,
and the movement decomposes as **−8 recovered, +1 arrived**:
`w-inlfence`'s fence files `?supershuffle@@YAXPAD@Z` as a **parse** decline, so
`frontier-codegen-wrong 1 → 0` and `reader 40 → 41` are one event. **The
column is not monotone under correctness work** (#2280).

**And the eight that were recovered were taken by SEVEN ONE-FUNCTION CLASSES.**
Resolved by name: `?NextHashPrime` → `static-scan-loop`,
`CXLrcImpl_CreateClientWithTransport` → `xlrc-create-guard`, `_free_osfhnd` →
`osf-handle-guard`, `?append@DName` → `alloc-init-or-fail`,
`?GetBuffer@JsonWriter` → `json-utf8-copy`, `?FindNodeA` → `if-call-join`.
Whole-workload reach of those classes, measured: **1 · 1 · 1 · 1 · 1 · 2 = 7
emitted functions, 7 `fnbyte-exact`, 7 TU conversions.** Not one is a reader
*widening*; every one is a transcription of a single function, graded against
the reference obj before it shipped (#2281).

**Four results, and each changes what a follow-on should do.**

1. **No reader rung converts a TU, and the instrument that says "1" is
   fail-open.** Per-TU counterfactual over all 9 frontier TUs with the CFG
   screen applied: 3 are single-key, 2 are CFG-reachable, and the intersection
   is `src/Main.cpp` — whose single census key is a *first* blocker and whose
   chain `WB_EH_FINDINGS.md` §6 already enumerates at **fifteen** refusals,
   *"eleven of which are in seams that do not exist"*. The other two single-key
   TUs, `IPP_basicmath_xbox.cpp` (4× `expr-cmp-eq`) and `mmio.cpp` (3×), are
   `cflow-loop`/`cflow-if-n`/`cflow-if-2` and need **block IR**. That is
   **§10.29-era `w-band` #2242 reproduced on a different population by a
   different instrument** — w-band from the `≤10` distance band and the
   completeness axis, this lane from the frontier and CFG reachability. Same
   two TUs, same seven bodies, same answer. Board **#2282**.
2. **The byte judge separates the two mechanisms 11-for-11 against 0-for-1,106,
   and that ledger is now the lookup table a reader price needs.** Over all 34
   census-admitted classes: the **ten one-function classes are P(exact) =
   1.000** (11 functions, 11 exact), and **five classes are P(exact) = 0.000
   over 1,106 emitted** — `call-sequence-cmp-eq` 542, `call-sequence-value-fp`
   434, `framed-call` 123, `fp-tail-call` 5, `call-sequence-cmp-order` 2. Every
   one of the five is a call-bearing class whose callee c2 inlines, which is
   §10.28's mechanism reached from the ledger rather than from a rung.
   **`framed-call` is one of the three classes `CLAUDE.md` names as the port's
   byte-exact MVP**, and on this workload it is 123 emitted, **0** byte-exact,
   122 of them `stlpmtx_std::vector::back`. `w-inlfence2` (§10.29.1) took 625
   of the 1,106 into honest refusals hours earlier; this is that population seen
   from the byte column. Boards **#2283**, **#2284**.
3. **A ninth ranking artifact, and it is `w-callprice`'s own emitted-column
   #1.** With a demangled-STEM column — #2243's test, applied to the whole
   workload rather than to a 69-TU band —
   `expr-call-in-expr-recv-object-then-call-recv-object-more` is **5,608 emitted
   over 747 TUs, 1,139 distinct mangled names, and ONE stem: `MakeString`**
   (`src/system/utl/MakeString.h:60`). `dname` and `emitted == dTU` both pass.
   §10.26.7 named that key as the family's highest-yield row (296.5 emitted per
   1,000 bodies) **in the lane commissioned to correct the eighth artifact**.
   Six more of the top 25 collapse the same way, including
   `expr-intrinsic-memset` — §10.26.8's own recommended successor, whose
   construct count it recorded as UNMEASURED and which is **36**. Across seven
   rows the test reaches **20,795 emitted functions, 16 % of the blocked
   emitted column**; the other two replication tests reach one row between
   them. Boards **#2285**, **#2286**.
4. **#2095's requirement is unmeetable for a reader candidate, by construction.**
   Over 178,977 emitted rows (`= in-class 39,643 + blocked 130,117 + unbound
   9,217`, asserted to sum), the byte verdict over the 130,117 blocked rows is
   `fnbyte-refused` **130,117 · exact 0 · differs 0`. The census's blocked
   column and the byte judge's refused column are the **same rows**, so no
   reader candidate's `fnbyte-exact` delta can be crossed with the oracle at
   all — which is why `w-fltret` could not have been priced in advance, and why
   any future reader price must be a **prior with its confound named**. Board
   **#2290**.

**The residues, sized at this tip.** `expr-op-0x27` is **22,412 emitted over
801 TUs with 4,001 stems** — the largest key on the column, not an artifact,
and **not a reader rung**: `WB_READER_FINDINGS.md` §3.3 already establishes its
grammar cost as none, so the whole 22,412 is an acceptance gate in front of a
designator lowering (§10.26.4's lowering side). `expr-op-0x28` is **28 emitted
in 25 TUs** and all 28 witnesses read the literal `28 00 00`, so §3.4's width
disagreement is latent in every one. wb-eh's **R1 re-derives to the unit at 682
emitted** over 19 TUs and 166 stems. The walker's `9B` is **5,985 across 8
keys** (`expr-op-0x9B` alone re-derives #1943's 1,590 to within 3) and `64` is
**1,576 across 10** — 2.6× and 2.9× the published framings, which counted a
different denominator. Boards **#2287**, **#2288**, **#2289**.

**The ranked answer, and it is the deliverable.** Rank 1 is **bespoke
transcription of a frontier body on a port CFG class** — **16 of the 41**
qualify, 25 do not (`cflow-loop` 19, `cflow-if-n` 5, `cflow-if-2` 1) — with a
predicted **`fnbyte-exact` delta of +10** from the size prior applied per body,
and **0 TUs**, because the sixteen are spread over eight TUs each of which also
carries a body on a CFG class the emitter lacks. Ranks 2–5 (`param-width` 682 ·
`expr-op-0x27` 22,412 · `memset` 3,749 · the `9B` family 5,985) are all
**UNKNOWABLE** on the byte judge and three of the four are lowerings rather than
admissions. The calibration: **7 bespoke transcriptions = +7 `fnbyte-exact`,
+7 TUs; `w-fltret`'s 444-wide admission = +0, +0. 63× the admissions, zero the
bytes.** Boards **#2292**, **#2293**.

**Effect on the §10.27/§10.28/§10.29 ordering.** Item 1 (`lower_expr`) is
unchanged and untouched by this lane. **Item 2 — reader admission at the
frontier's 48 — is priced and closes**: its reach-1 form is the only form that
has ever moved `fnbyte-exact` on this board, and its wide form has never been
shown to. Item 3 (the inline fence) is unchanged and gains a ledger: #2283's
five zero-exact classes are the population it exists for. What is left at the
frontier after this lane is **25 of 41 functions behind a CFG class that does
not exist**, `src/Main.cpp`'s fifteen (`w-main`'s, cited not re-derived), and
sixteen one-function transcriptions worth about ten byte-exact functions
between them.

**This lane ships no `crates/` change**: one 62-line scratch print in
`gap/fnbytes.rs`, reverted before the gate, diff at
`work/w-readpx/scratch.patch`, and `git diff master -- crates/` is empty at its
tip.

[`rungs/2026-08-09-w-readpx.md`](rungs/2026-08-09-w-readpx.md).
