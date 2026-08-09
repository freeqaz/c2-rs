
### 10.26.8 w-memfit — the two `memcpy` lanes were BOTH RIGHT, the rule scores 624 of 624, and `mmio.cpp` declines at four (2026-08-09)

Two landed lanes had measured the same decision and published opposite
conclusions, and the board carried both:

* **`w-memcpy`** (black box, 2026-08-08) — *"no rule fits"*. Its best frozen
  rival scored **182 / 232**, `M-ALWAYSCALL` **114 / 232**, four separately
  frozen thresholds all missed, and its one unanimous sub-class was refuted by a
  second grid at **114 / 176**. `w-park` cites this as *"the rule was measured
  NOT TO EXIST"* and declines `?mmioGetInfo` on it.
* **`wb-memcpy`** (whitebox, same day) — a decision function READ out of the
  binary and graded **180 / 180** on a **new** grid of its own.

**Neither number was ever comparable to the other, and putting them on one
denominator is the whole of this rung**
([`rungs/2026-08-09-w-memfit.md`](rungs/2026-08-09-w-memfit.md), board
**#2060**–**#2071**).

**The reading explains the cells — 232 / 232 on GRID-M and 176 / 176 on GRID-M2,
the denominators on which `w-memcpy` published 114 and 114.** With GRID-W that
is **624 of 624 over three grids frozen by two different lanes on two different
days**, one of them frozen expressly to refute the other's fence. Nothing is
retracted; no address moves.

**Four results, and three of them change how a measurement should be read.**

1. **"No rule fits" was a rule-space limitation, and the missing axis is NOT
   the one everybody named.** `w-memcpy`'s grids are `/O1`-only — a real
   limitation — but at `/O1` the threshold **is** 5, and its own cells separate
   `T = 5` from `T = 10` on **76 cells, 76–0**. What none of its six rivals
   could express is a **quotient**: one keys on the intrinsic id, one on
   constancy, and four on the **size**. The quantity is `size / align`,
   truncating. (#2061 — and this lane's own PREREG named favor-speed as the
   missing axis, at p = 0.80, and was wrong about the diagnosis while right
   about the outcome.)

2. **The obj cells decide a part of the whitebox reading that the whitebox
   lane's own grid cannot see.** `size/align` and `ceil(size/align)` differ only
   on a size the alignment does not divide. GRID-W's `n` axis is elements, i.e.
   exact multiples, so **0 of its 216 cells** separate them; `w-memcpy`'s
   absolute size axis separates them on **22**, truncating **22–0**. The
   truncation is `WB_MEMCPY_FINDINGS.md` §3's own sharpest claim, and the grid
   built to grade that reading could not have graded it. **A
   disassembly-derived grid inherits the disassembly's axes.** (#2062)

3. **The grey-zone alternative was tried a second time and SUCCEEDED, so the
   disclosure row is `route:` and not `adoption`.** An exhaustive fit over four
   candidate quantities × every threshold `0..2048`, held out both directions,
   recovers both constants from obj cells with no disassembler: fitted on
   GRID-W's 72 `/O1` cells it scores 232/232 and 176/176 on grids it never saw;
   fitted on those 408 it scores 72/72 on GRID-W `/O1` and refuses `/O2`,
   `/Ox`, `/O1 /Ot` at 18/36 each. The disassembly supplied the **search
   space**. `DISCLOSURE.md` gains **W-MEMCPY-1** as `route:`; W-MEMCPY-2 and
   W-MEMCPY-4 are **not carried at all**, and W-MEMCPY-3 was re-derived black
   box. `README.md`'s per-finding wording moved in the same commit. (#2063)

4. **A rescoring harness that cannot reproduce the published scores is
   measuring something else — and this one could not, twice.** `score.py`
   refuses to print a new number until it has re-derived all eight of
   `w-memcpy`'s published scores from that lane's own files. It caught the
   committed `probeM2/measured.json` carrying the **two-valued** verdict
   `w-memcpy` §6.2 itself records as a bug (44 eliminated bodies labelled
   `inline`, which grades the rule 132/176 and publishes a refutation), and it
   caught a second, unbudgeted defect in this lane's own verdict function — the
   relocation must be consulted **before** the byte count, because a
   non-constant size at `/O1` is a four-byte **tail call** with a REL24.
   **Both produce a plausible number.** (#2064)

**The conversion DECLINES, and the re-derivation moved two of `w-park`'s twelve
in opposite directions.** `call-arg-lit-permuted` is **paid** — `l3.cpp` is a
whole-TU `match` at `/O1`, graded this session, so `?mmioGetInfo`'s exact
instruction stream is already something the port emits byte-exact (#2068) — and
`mmioSetInfo`'s first refusal is no longer `call-token-0xB9` but
`callseq-tail-lit`, because **`mmioSetInfo` calls `memcpy` too**. So `w-park`'s
*"the whole remaining distance is the word `memcpy`"* is a statement about one
body's 84 bytes; on the obj the word governs **two bodies and 192 of the 316
remaining** (#2067). The cheapest body still costs **four** — the `40` token is
not a call head, the callee has **no `.gl` token** so the symbol must be minted
and placed, five IL operands reduce to three emitted slots, and each pointer
argument carries a `2C` (#2069).

**And if the family is ever taken it is taken for `memset`, not for `mmio`.**
First-blocker populations: `expr-intrinsic-memcpy` 3,366 bodies / 99 emitted;
`expr-intrinsic-memset` 34,795 / **3,749** — 2.95 % of the whole blocked emitted
column, obeying the same rule on every cell it was crossed with. Quoted as a
**size and explicitly not as a price**: #2025, one lane over, built a
2,188-emitted key and converted zero.

**This lane ships no emitter or parser change.** One comment in
`crates/c2-il/src/func/body/expr.rs` pointing at
[`IL_INTRINSIC_CALL.md`](IL_INTRINSIC_CALL.md) §5.1.1, which is where the rule
now lives with its confident core (**552 of 552**) stated separately from its
score, and with the two arms that have **zero** cells of evidence printed as
zero. Census, match, mismatch, all 251 `gap-metric` keys, all 635 + 614 blocker
keys, the whole FRONTIER block and all 312 fixtures at `/O1` **and** `/Ox` are
unchanged; `#[test]` delta **0**.

[`rungs/2026-08-09-w-memfit.md`](rungs/2026-08-09-w-memfit.md).
