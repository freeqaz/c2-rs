
### 10.26.7 w-callprice — `expr-call-in-expr-*` priced on the EMITTED column, and the pointer §10.26.6 left is the family's worst row (2026-08-09)

§10.26.6 closed with an instruction: *"the only real lever is the
**call-in-loop-body** seam … It must be priced from `expr-call-in-expr-*` —
whose single largest key already carries 46,036 bodies / 1,033 emitted — and
**not** from this family's 248, or a lane will re-run exactly the mistake this
one was commissioned to correct."* It is priced
([`rungs/2026-08-09-w-callprice.md`](rungs/2026-08-09-w-callprice.md), board
**#2020**–**#2032**). **The instruction was half right: pricing it from the
family was correct, and pricing it from that key was the same mistake one column
over.**

**The family, re-derived at `c5ff9953`: 423,905 bodies / 35,576 emitted —
27.25 % of the whole blocked emitted column.** Six keys cover half of it.

**Four results, and each one changes what a follow-on should do.**

1. **The body ranking and the emitted ranking disagree by 13×, and every
   published ranking of this family is the body one.** `IL_CALL_IN_EXPR.md`
   §11, §14.7, §16.7, §17.6, §18.7, §19.7, §22.8 and §24.8 all rank bodies.
   Body #1 (`recv-load-then-bit-and-and-branch-more`, 102,374 bodies, 24 % of
   the family) yields **41.9** emitted per 1,000 bodies; emitted #1
   (`recv-object-then-call-recv-object-more`, 18,912 bodies) yields **296.5**.
   This is an **eighth** instance of the ranking-instruments lesson and a
   **third mechanism**: not a key shattered by an id, not a key inflated by TU
   replication, but a family ranked on the column the metric does not move.
   Board **#2020**.
2. **`op-0x9B` — the key §10.26.6 named — is rank 8 on the emitted column at the
   family's lowest yield (22.4 per 1,000 bodies), and its content was declined
   by name three sections earlier.** Read from source, its dominant construct is
   `MEM_OVERLOAD`'s `static void operator delete(void *v) { MemFree(v, __FILE__,
   line_num, #class_name); }` — **two string-literal addresses in one call**,
   which is `IL_CALL_IN_EXPR.md` §17 (D5)'s `.rdata` pool-relative selection,
   already priced there as *"a different and much larger piece of work"*. Board
   **#2021**.
3. **62.5 % of the emitted column is TU replication — and the discount runs the
   OTHER WAY from #2000's.** 35,576 emitted symbols over **13,329 distinct
   mangled names**. A body column counts segments, so replication inflates it; an
   **emitted** column counts symbols, so replication does not discount the
   metric — it **concentrates the work**. The five highest-leverage keys in the
   family are **one function each**, every one read back to its dc3 header:
   `??1MessageTimer@@QAA@XZ` is **419 emitted in 419 TUs and the only name on its
   key**. Boards **#2022**, **#2023**.
4. **`prod` × emitted had never been taken, and it inverts w-mcall #1963.** That
   row split the sequence route on **bodies** — `call-ref` 125,458 (78.9 %) to
   `call-token` 25,060 (15.8 %) — and named the 25,060 the seam's next step. On
   the emitted column: **`call-ref` 5,699 (39.0 %), `call-token` 8,666
   (59.3 %)**, a **7.6× yield inversion**. #1963 named the right row and
   **under**-priced it, which is the rarer direction. The largest `prod` tag on
   the emitted column is neither: it is the member call's own **argument operand
   vocabulary**, **8,909 emitted over 4,088 distinct functions**. Boards
   **#2024**, **#2030**.

**And one rung was built, run over all 878 TUs, and thrown away.** R1 — admit a
**named-object receiver** in a later statement of a statement-call sequence — is
thirteen lines behind an env gate. Function census **+0**, emitted census **+0**,
per-TU verdict set **0 changed**. Its first-blocker key claims **2,188 emitted**;
the shipped locator reaches **at most 33**; it converts **0**. **A first-blocker
population over-stated a price by three orders of magnitude, and the second- and
third-order instruments built specifically to prevent that still over-stated by
66×.** Board **#2025**. Behind it sits a wrong-bytes hazard nobody had a key for:
an **address-taken stack local wears the same `26 <sym>` designator a relocation
does**, read off c2's own `/FAsc` listing — `addi r3,r1,fs$` against `lis`+`addi`
on a relocation — over a receiver form that is **28.5 % of the family's emitted
column**. Board **#2026**.

**The re-ordering this section licenses.** §10.26.5 declared the list out of code
seams and §10.26.6 declined the loop family; both stand. What changes is the
successor ordering *inside* item 2's family, which is now priced on the column
that ranks:

| | emitted | constructs | what it is, in port terms |
|---|--:|--:|---|
| the **argument** operand vocabulary | **8,909** | 4,088 | a reader seam, but **not one rung** — its first sub-row (`-then-intrinsic-call`, 2,865 / 1,158) is an **argument slot form** for a base adjust; its second is w-value's operand-position class and needs a lowering |
| the **chained** sequence receiver | **5,638** | 1,169 | a reader route that exists at the tail (`mcall_chain`) — **and R1's sibling through the same arm, so price it by building it** |
| the **float value tail** | **544** | 9 | `CallSeq` lowers the statement half already; what is new is the **member** value tail and `CallRet::discarded`'s `_fltused` obligation on the returned side |

**In that order by size and the reverse by confidence.** The recommendation is
the third — **544 emitted over 9 constructs, `-whole` on the census's own grammar
walk, hand-checked on `float Timer::SplitMs() { Split(); return Ms(); }`
(434 emitted in 434 TUs) and on c2's listing.** That is **78×** w-mcall's
realized 7, which is what a real rung on this board looks like; it is also a
one-function class with a 434× multiplier, which is the most brittle. Board
**#2032**.

**No `IlOp::Call` variant is proposed anywhere in it** — w-mcall #1961's decline
was inherited as this lane's PREREG clause D1 before its first scan, and the two
populations that would want one are named as **lowering** work with their own
cost rather than smuggled in as admissions.

**A note on method, because this lane's own PREREG got it wrong.** All four
predictions registered as *pessimistic* — that the family would be shattered,
that its named next step would be small, that no rung was worth a lane — missed,
all four in the optimistic direction, and the PREREG says why it registered them:
*"Seven blocked-key size rankings in a row have turned out to be artifacts …
This lane assumes it is the eighth."* Board #770's streak is a record of
optimistic predictions missing; this is its mirror. **A prior calibrated on seven
instances of one mechanism misfires on the eighth.** Board **#2031**.

**This lane ships no `crates/` change**: four scratch instruments, all reverted,
recorded at `work/w-callprice/scratch.patch`, and `git diff master -- crates/` is
empty at its tip.

[`rungs/2026-08-09-w-callprice.md`](rungs/2026-08-09-w-callprice.md).
