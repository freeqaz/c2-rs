
## 10.28 W-FLTRET — §10.27's item 3 SHIPPED: the price came true at 99.3 %, and the byte judge did not move at all (2026-08-09)

§10.27's ordering put w-callprice's **R2** third — *"the float value tail, 544
emitted over 9 constructs, §10.26.7, as the largest priced conversion rung on the
board"*. It is shipped
([`rungs/2026-08-09-w-fltret.md`](rungs/2026-08-09-w-fltret.md), board
**#2080**–**#2087**), and it produced the cleanest realization *and* the
sharpest disappointment this board has recorded on one rung.

**The reader admission.** `BodyShape::CallSeq` already lowered the statement
half; what was new is the **member** call in the sequence's **value tail** —
w-mcall's decline **D3**, filed *unsized* because no census key separated it —
plus the `_fltused` obligation on the *returned* side. It is a reader change in
`crates/c2-il` with one new `SeqTail` variant; `crates/c2-core` gains **two
match arms and no instruction**, because c2's own `/FAsc` listing shows the
float body and the `int` body are the same instruction stream and the only
difference in the obj is one undefined external.

**Six results.**

1. **A first-blocker price came TRUE, for the first time on this board.**
   w-callprice §5.2 measured the clause at **447 emitted over 13 constructs**;
   it converts **444 over 10**, **99.3 %** — measured by a counterfactual on one
   binary against itself, not by differencing two builds. One lane earlier
   #2025's R1 claimed 2,188 emitted and converted **0**. Same family, same
   instrument, same day. What separated them is the census's own **`-whole`**
   bit: the grammar walk's claim that granting the blocker *finishes the body*.
   It is a claim by the census and not by the parser, so it is a signal and not
   a guarantee — but across two rungs it separated 99.3 % from 0 %. Board
   **#2080**.
2. **And `fnbyte-exact` moved by ZERO.** 36,228 before and after; `fnbyte-differs`
   **2,111 → 2,555 = +444**; FBM **0.20243 → 0.20243**. Every function the
   emitted census now claims is graded by the oracle's own per-function byte
   test and **not one is byte-exact.** This is a **ninth** instance of the
   ranking-instruments lesson and a **fourth mechanism**: w-callprice re-ranked
   this family off the body column onto the emitted column because the body
   column was wrong (#2020) and was right to — and the emitted column is *also*
   not the byte judge. The lane's own PREREG made seventeen predictions about a
   census column and **none** about `fnbyte-exact`. Board **#2081**.
3. **All 444 are one mechanism and it is c2's INLINER.**
   `?SplitMs@Timer@@QAAMXZ` is **434 of the 444 and the only name on the new
   census key** — `float Timer::SplitMs(){ Split(); return Ms(); }`,
   `src/system/os/Timer.h:137`. The reference body is **31 words where the port
   emits 13**, and the words c2 has and the port does not are `Split()`'s and
   `Ms()`'s own (`lfd`, `fcfid`, two `lis` pairs): both callees are `inline`
   members in the same header. In the fixture the callees are declared and not
   defined, c2 cannot inline them, and the TU is a **whole-TU byte-exact match
   at `/O1` and `/Ox`**. The class is byte-exact exactly where the callees are
   opaque, and on this workload they never are. `mismatch` is 0, `functions()`
   is untouched and all 434 TUs are `vocab-gap`, so no obj has ever carried one:
   what is wrong is the *census's claim*, which is STATUS.md trap 2 in its
   standing form, and the remaining distance is `splice.rs`'s. Board **#2082**.
4. **The `_fltused` obligation needed no new insertion point, and its post-op
   fence is a MISSING FIELD.** `SeqTail::CallValueFp` is the **fifth** producer
   of `touches_floating_point` and the first that emits no FP instruction at
   all. It carries no `add_k`, because `return o->F() + 1.0f` is `lfs` from the
   `.rdata` FP pool plus `fadds` and the field would have no correct value.
   Placement was derived from c2's listing, from the reference obj, and from a
   third TU whose *first* function is not the FP one — not by analogy. Board
   **#2083**.
5. **The IL draws the same-width line itself, and the fence gives up a free
   conversion.** A converted real result carries an explicit `2C <TYPE> 00`
   between the `4C` and the `41` **in both directions**, and only one costs an
   instruction (`float`←`double` is `frsp`; `double`←`float` is nothing).
   Requiring the `41` immediately after the `4C` refuses both. Said as a decline
   with the listing beside it rather than smuggled. Board **#2084**.
6. **A `_neg` fixture cannot see one of its own cells being converted.**
   `wmcall_seq_neg.cpp` graded `Port=NotImplemented` before and after, because a
   `_neg` fixture's graded property is a *whole-TU refusal* and that survives any
   one cell becoming a positive. Cell N6 — w-mcall's D3 — reads
   `ok call-sequence-value` at this tip and no gate row noticed. Re-taken per
   w-park's precedent. **Every `_neg` fixture on this board is a claim that N
   clauses refuse, graded by a property that holds if one does**, and nothing
   standing re-checks them per function. Board **#2085**.

**Two instrument findings a follow-on needs.** Three of the four refusals this
rung adds are **inert on the whole workload**, and reading "0 in the 635-key map"
would have measured *nothing* — `parse_call_sequence_from`'s `Err` is discarded
by its caller, so a clause inside that loop can never mint a first-blocker key.
A scratch that **commits** the loop's `Err` makes the zero a measurement, and
prices the loop's live clauses at the same time (`callseq-multiarg-sym:eof`
**1,425**, which is #2026's blanket refusal). Board **#2086**. Separately,
`git checkout -- crates/` to revert a scratch **also reverted this lane's
uncommitted unit-test repairs**; the discipline every recent lane uses is only
safe if every non-scratch change is committed first, and no rung says so. Board
**#2087**, reported as an unbudgeted unnamed refusal — w-park's streak goes to
**11/15**.

**Two corrections to §10.26.7's own numbers, both re-derived rather than
inherited.** R2's `-type-real-whole` population is **545 over three keys**, not
544 over two. And those keys are **two reader routes**:
`recv-load-then-type-real-whole` is 714 of its 933 bodies on `CallSeq`'s route,
while `chained-then-type-real-whole`'s **1,472 bodies / 105 emitted** are
**100 %** on `mcall_chain`'s — so *"CallSeq already lowers the statement half,
reuse it"* is a statement about the 439 and not about the 105. The 105 are
declined by name and converted **0**, as the PREREG registered.

**Effect on the §10.27 ordering.** Item 3 is done. Items 1 and 2 — `lower_expr`
and reader admission at the frontier's 48 — are unchanged, and #2082 adds a
fourth item that was not on the list: **the inliner is now the binding
constraint on 444 already-in-class emitted functions**, which is the first time
`splice.rs`'s territory has been sized off a conversion rather than off a
survey.

[`rungs/2026-08-09-w-fltret.md`](rungs/2026-08-09-w-fltret.md).
