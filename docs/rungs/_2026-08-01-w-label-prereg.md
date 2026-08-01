# w-label — pre-registration (written before any code was read for #135, and
# before any test was written for #137)

Lane `w-label`, 2026-08-01. Board **#137** (portable pins for WR1's two ordering
rules) then **#135** (model the label counter from `.cod` allocation order).

Committed **before** the mutations of §A were run and **before** the `.cod`
family of §B was captured, so the scores in `docs/ROADMAP.md` §9.12 can be graded
rather than retrofitted.

Base: `33d0049`. `cargo test --workspace` at base — **571 passed, 0 failed,
1 ignored** with the toolchain resolving, and **571 / 0 / 1** on the portable lane
(`C2RS_WIBO=/nope C2RS_CL_EXE=/nope C2RS_C2_DLL=/nope`). The two totals being
equal is itself the §9.10 observation: the toolchain-gated tests report `SKIP` and
still count as `ok`, so **no count can distinguish the two lanes** and only a
mutation can.

`git grep -c '#\[test\]'` summed over the workspace at base: **571**.

---

## A. #137 — the portable pins

### A0. The premise, stated as a falsifiable claim (this is the control)

**P0 — nothing in the workspace currently pins either rule portably.** Under each
of the three mutations below, the **portable** lane stays **571 passed / 0
failed** at base. *Refuted by* any existing test going red.

This is the claim §9.10 makes and the one worth being wrong about: if some
existing unit test already catches a mutation, #137 is smaller than the board
says and I must report that instead of adding a test on top of it. It is run
**before** any new test exists, so it cannot be contaminated by them.

Its companion, which decides that the mutations are real and not no-ops:

**P0′ — each mutation is caught by the *toolchain* lane.** With the toolchain
resolving, each of the three goes red. *Refuted by* a mutation that nothing
anywhere catches — which would mean I had mutated dead code and the whole
experiment is vacuous. (This is the "would it have gone red if the claim were
false in the most likely way" question, asked of the mutation itself rather than
of the test.)

The three mutations, each a one-site edit that implements the rule WR1 got wrong
on its first differential:

* **M1 — descending, not address-last.** In `codegen::calls::sym_slots_text`,
  emit the `addi rD,r11,sym@l` at its own slot's turn in the descending walk
  instead of after every other slot.
* **M2 — the quad's halves are adjacent, in `coff.rs`.** Write the REFLO/PAIR
  records at `hi_off + 4` instead of at `lo_off`.
* **M3 — the quad's halves are adjacent, in the derivation.** In
  `PortC2`'s `data_refs_of`, set `lo_off = base + 4` instead of searching the
  body for the low-half `addi`.

### A1. What the new tests assert

* **P1 — the address `addi` is emitted LAST.** `sym_slots_text` over
  `[Formal(0), Lit(7), SymAddr]` — the symbol at slot **2** and a literal at the
  **lower** slot 1 — emits `lis r11,0 · li r4,7 · addi r5,r11,0`, i.e. the lower
  slot's `li` **before** the higher slot's `addi`. *Refuted by* the `addi`
  preceding the `li`. Predicted: passes as written, red under M1.
* **P1′ — the agreeing arrangement stays green under M1.** The same assertion
  over `[SymAddr, Lit(7)]` (symbol at slot 0) must **pass under M1**, because
  descending and address-last agree there. *Refuted by* it going red — which
  would mean M1 is not the descending rule I think it is, and P1's red tells me
  nothing about *which* arrangement discriminates. This is the half WR1's hand
  fixture had three copies of.
* **P2 — REFLO is not at `hi_off + 4` in the derivation.** `data_refs_of` on the
  `p4` body (`lis r11 · li r4,7 · addi r3,r11,0 · b`) returns
  `lo_off == base + 8`. *Refuted by* `base + 4`. Red under M3.
* **P3 — REFLO is not at `hi_off + 4` in the obj.** An obj emitted with
  `DataRef { hi_off: 0, lo_off: 8 }` carries exactly four `.text` relocation
  records for the quad, in the order REFHI · PAIR · REFLO · PAIR, at virtual
  addresses **0 · 0 · 8 · 8**. *Refuted by* a REFLO at VA 4. Red under M2.

Every assertion carries a **distinct** message, and no early guard may make a
later one unreachable: the fixture quantities each test depends on (how many
words the body has, how many relocation records the section carries) are pinned
by their own assertion first, phrased over a property of the input rather than
over the classifier under test — §9.9.1's rule.

**P4 — the `#[test]` total moves.** Predicted **571 → 577**, interval
[575, 580]. A rung that touches `coff.rs` and does not move it is the §9.10
finding repeating.

---

## B. #135 — the label counter's allocation order

### B0. The prior being tested

`coff::plan_labels` today models a framed function as **one contiguous ascending
triple** `[n, n+1, n+2]` = `$M(n)` prologue end, `$M(n+1)` function end,
`$T(n+2)` `.pdata`, allocated in `.text` order, with a per-function stride of 4
packed / 5 under `/Gy`. §9.3's single `.cod` row says that on a **framed + EH**
body the allocation is neither contiguous nor in text order.

### B1. Registered predictions

* **B1 — the non-EH triple is contiguous and in that order.** Over every `.cod`
  captured in this round with no EH, every framed function's labels are exactly
  `$M(n) · $M(n+1) · $T(n+2)` with consecutive numbers, `$M(n)` at the prologue
  end and `$M(n+1)` at the function end. Predicted **100 %**. *Refuted by* one
  framed non-EH function whose three labels are not consecutive or not in that
  order.
* **B2 — allocation order is text order for non-EH bodies.** Within a TU with no
  EH, sorting functions by first label number gives the same order as sorting by
  `.text` offset. Predicted **100 %**. *Refuted by* one inversion.
* **B3 — the funclet is allocated first.** On every EH body, the `__unwind$`
  label's number is **lower** than every `$M`/`$T` in the same function, and its
  text offset is **higher** than every `$M` in the same function. Predicted
  **100 %** of EH bodies. *Refuted by* one EH body where it is not.
* **B4 — the `$M` block splits around the `$T` tables.** On an EH body the label
  sequence in allocation order is not `$M*` then `$T*`; at least one `$T` sits
  between two `$M`s. Predicted **100 %** of EH bodies. *Refuted by* one EH body
  whose `$T`s are all after all its `$M`s.
* **B5 — the stride is the label count, and it is exact.** For consecutive
  functions in one TU, `first_label(f_{i+1}) − first_label(f_i)` equals the
  number of labels `f_i` allocates plus a per-mode constant. Predicted: a rule
  I can state and test. **Held out**: fitted on the `/O1 /Oi /EHsc` family and
  tested on `/O2`, `/Ox` and `/EHsc`-off families it was **not** fitted on.
  Registered held-out accuracy: **≥ 90 %** of held-out functions' label numbers
  predicted exactly from the TU's first label. *Refuted by* < 70 %.

### B2′. The control that decides whether any of it means anything

A predicate that says "the labels are what `plan_labels` already says" would go
green on the whole in-class population **by construction**, because that
population is chosen to be the one the port emits byte-exact. So the held-out
family must contain shapes `plan_labels` **cannot** model — EH, loops, switches,
multiple functions per TU — and the accuracy must be reported **separately** for
the shapes the port already handles and the shapes it refuses. If the two are
equal, the model has learned nothing beyond the shipped one and #135 must say so.

The registered discrimination: on shapes the port refuses, `plan_labels`'s own
prediction is expected to be **wrong** (predicted accuracy **< 50 %** there),
while the new rule is expected to be right (**≥ 90 %**). If `plan_labels` is
*already* ≥ 90 % on the refused shapes, there was no gap to close and the round
must report that.

### B3′. What would make me report a refusal instead of a rule

If the allocation order on the widened family is not generated by any predicate I
can state in one paragraph — the state `LABEL_COUNTER.md` §6.15.3 records for the
inline-decline schedule — then #135 reports the transcription and the refusal,
and does **not** ship a `plan_labels` change. A wrong stride is a wrong `$M`
number and a wrong `$M` number is a wrong-bytes obj.

---

## C. `LABEL_COUNTER.md` §6.15–§6.19

Registered before reading them: I expect the `.cod` evidence to be **largely
orthogonal** to those rounds' negatives, because §6 measures label *counts* on a
front-end inlining ladder and §9.3 measures label *allocation order* inside one
body. Predicted: **at most one** of the negatives is touched at all. *Refuted by*
the `.cod` evidence widening or refuting three or more of them.
