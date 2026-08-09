
### 10.26.5 w-bdnz — item 4 SHIPS, and the ordering it leaves behind has no third code seam in it (2026-08-09)

The `loop_guard+bdnz` class of §10.26.1 is landed
([`rungs/2026-08-09-w-bdnz.md`](rungs/2026-08-09-w-bdnz.md), board
**#1980**–**#1988**): `wb-loop`'s passes **1 and 2** — the rotated pre-test
guard and the `mtctr`/`bdnz` conversion — as a recognizer in the parser and
eight words in the emitter, byte-exact against real `c2.dll` on eleven
manufactured cells at `/O1` **and** `/Ox`, fenced by twenty-three negative cells
over twenty-two distinct clause keys. Pass 3, the update form, is **declined by
name**, and the class is drawn to contain no memory reference at all so that the
decline is structural rather than a promise.

**Reach on the workload is 0, exactly as §10.26.1 and #1829 registered**, and the
neutrality is total rather than nearly total: 878 TUs by name **0 changed**, all
251 `gap-metric` keys **byte-identical**, 635 body and 614 emitted first-blocker
keys **0 moved**, 309 pre-existing fixtures **0 moved** at both modes, both
censuses **+0**. The arithmetic ceiling was registered in advance — `expr-jump`
at **2,286 bodies / 302 emitted**, the key every cell of this class blocks at —
and it moved by zero, the sixth consecutive lane to find that a first-blocker
key's size is not its class's size.

**Three things this lane found that change what a follow-on should do.**

1. **The label charge for a back-edge class is MODE-DEPENDENT.** Measured
   against the obj in w-json's counterfactual form: +7 at `/O1` and +8 at `/Ox`
   over `leaf-none`, where `LABEL_COUNTER.md` §4.2.1's `for` row read literally
   predicts +1. `IlFunction::label_slots` has no mode parameter, so `None` is
   not conservatism for this class — it is the only value that can be right.
   And the seam has **two** layers: a correct `Some(k)` also needs
   `plan_labels` to advance the same `k`, which `IlBundle::functions`' gate
   (`label_slots(false)? != label_lead() + 1`) is what actually asks. Board
   **#1983**.
2. **`.sy` was the binding constraint on half the class, and board #764's
   finding repeats one class over.** The unsigned counter is byte-exact against
   real `c2` and was blocked entirely by `.sy`'s `plain_int` predicate. The
   repair is a **fourth positive list** (`uint_locals`), additive by
   construction — an `unsigned` local used to leave `read_record` as `Stepped`
   — and deliberately *not* a widening of `int_locals`, because the two are the
   same storage and a different `cmp`. Board **#1984**.
3. **A fence built only around what c2 refuses is a fence around nothing.**
   Sixteen of the twenty-three negative cells are loops c2 **does** convert, and
   three of them have reference text byte-identical to an accepted cell's. Board
   **#1982** — and the count was *six* in the fixture header until a script
   counted it, which is the week's paraphrase failure caught before commit.

**What the ordering looks like now.** §10.26's re-ordering had item 2 spent
(§10.26.3, corrected by §10.26.4), item 4 was this, and item 5 is a close.
That leaves **item 3, the inline decline side**, as the only unstarted code seam
on the list — and it is a decline-side rung, so it converts nothing by
construction either. **The list is out of levers, and saying so is the result.**
Every remaining step on this seam (board #1988 prices five) widens
infrastructure; two of the five are blocked on whitebox readings nobody has
made (`wb-loop` §9 item 4's trip-count selector, §4.4's unelected update-form
rival). What would make the loop family a lever is a **reader** rung on
`expr-jump` itself — 2,286 bodies / 302 emitted, and **nobody has decomposed
that key**. That is the measurement a next lane should take before it takes a
lowering.

[`rungs/2026-08-09-w-bdnz.md`](rungs/2026-08-09-w-bdnz.md).
