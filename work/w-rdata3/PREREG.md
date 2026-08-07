# w-rdata3 — PREREG

    Lane:    w-rdata3 (`wt-w-rdata3`), branched at master `e60f8902`
    Written: BEFORE any probe was run. No `c2rs census`, no `c2rs diff`, no
             `c2rs gap`, no capture, no `cl.exe` had been invoked on this tree at
             the time this file was committed. What HAD been read is source and
             docs: `docs/rungs/2026-08-07-w-rtti.md`, `docs/OBJ_RDATA_R_SHAPE.md`,
             `docs/rungs/2026-08-04-w-rdata.md` §4, `docs/STATUS.md`,
             `docs/BOARD.md` rows #300/#301/#302/#360/#362/#926/#927/#931/#936/#941/#1029,
             and the four source files the seven refusals name
             (`coff/function.rs`, `coff/data.rs`, `c2-core/src/lib.rs`
             `data_refs_of`, `c2-il/src/func/gl.rs` `gl_data_objects_ordered`).
    Task:    re-derive the `.rdata$r` price on TODAY's tree; ship the writer only
             if ALL SEVEN refusals are paid; otherwise decline and publish the
             remaining price with owning lanes.

---

## §1 The incumbent — what a landing must beat

The incumbent is **two standing declines**, not a threshold:

* **`w-rdata` (2026-08-04, board #300)** — priced the minimal `.rdata$r` obj at
  **seven** independent unpaid refusals, registered *two* and was refuted at
  seven, in the direction that made the work look cheaper.
* **`w-rtti` (2026-08-07, board #926)** — briefed to ship it anyway, re-derived
  at master `9827bcf`, found **all seven still unpaid**, and shipped the guard
  (#927/#301) instead.

**To land a writer this lane must beat that incumbent on its own terms**: every
one of the seven paid *by measurement on this tree*, plus at least one real obj
graded **byte-exact against `c2.dll` under wibo**, plus `factor-c` re-read
**from a scan** with `writer-sections` beside it. Anything less is the third
decline, and a third decline with a sharper price is the registered success
mode.

**The decline floor** (fires exactly as `w-rtti` P9 did): the writer ships only
if a `.rdata$r` `Section` literal exists **and** has a caller reachable from
`PortC2::build` **and** the differential grades ≥ 1 real obj byte-exact. Failing
any of the three, `factor-c` stays 169 and the deliverable is the price.

**The direction I expect to lose on.** I expect to be **wrong in the direction
of "more is paid than I predict"** — a great deal of `c2-il` landed since
`9827bcf` (tag 02 at 100 % of 1,885,700 symbol addresses; mechanism E and its
fixpoint; the destroy-loop; the no-effect reader; the `.in` scalar/zero-fill
readers) and `coff/data.rs` visibly grew a whole relocation class check citing
board #931 — which is *this* rung's board row. If I am wrong the other way (a
refusal I score PAID turns out unpaid), that is the dangerous direction, because
it is the one that ships a wrong emit; every PAID verdict below therefore
requires a **positive instrument reading**, never the absence of a negative one.

**I expect at least one of P1–P7 to lose.** A re-pricing that reproduces the
previous lane's verdict on all seven items is weak evidence that anything was
re-measured rather than inherited — `w-rtti` registered the same caution and
lost two of its four spec predictions.

---

## §2 The seven, predicted one at a time

The list is `w-rdata` §4 / `OBJ_RDATA_R_SHAPE.md` §8, verbatim in numbering.

| # | refusal | crate | **prediction** | instrument I will use | confidence |
|---:|---|---|---|---|---|
| 1 | the vfptr-store leaf body class (`expr-op-0x27`) | `c2-il` | **UNPAID** | `c2rs census` on the §2 minimal source; blocking key printed | 0.75 |
| 2 | a reader for the `??_R*` record graph — the `.gl` data records of an RTTI TU | `c2-il` | **PARTLY PAID — the `.in` half; the `.gl` half UNPAID** | a `gl_data_objects_ordered` spike with a `.data` positive control, exactly `w-rtti` §4.2's table, re-run on this tree | 0.6 |
| 3 | codegen for a `DataRef` whose low half feeds a **store** | `c2-core` | **UNPAID** | `data_refs_of` source + a `c2rs diff` on the minimal TU | 0.85 |
| 4 | the `.rdata$r` / `.data`-COMDAT `Section` emitter and its `ADDR32` relocations | `c2-core` | **UNPAID** | `PORT_WRITER_SECTIONS` (10 names) + `coff/data.rs`'s own COMDAT refusal | 0.95 |
| 5 | the DFS emission order over sections **and** undefined externals | `c2-core` | **UNPAID** | grep the crate for any DFS/relocation-graph walk | 0.9 |
| 6 | the vftable `.rdata` COMDAT — Selection 6, symbol `Value` 4 | `c2-core` | **UNPAID** | grep `coff/` for a Selection-6 / non-zero-`Value` emitter | 0.9 |
| 7 | the `??_7type_info@@6B@` undefined external | `c2-core` | **UNPAID** | `coff/data.rs`'s explicit refusal of a non-local relocation target | 0.9 |

**Registered aggregate: I predict the count re-derives at SEVEN, with item 2's
*character* narrowed** (its `.in` half paid by `w-tag02`, its `.gl` half —
COMDAT records and the section-name attribute — still unpaid). Under this
project's no-discount rule (board #269, `w-conv`) a half-paid fact is still a
fact, so the count does not fall below seven unless a whole item reads paid.

**What would make me report FEWER than seven**: item 1 reading `1/1 functions in
class`, or item 2's spike returning ≥ 1 `??_R*` record. Either is a genuine
change of the ladder head's price and I will report it as such even though it
does not, on its own, license the build.

**What would make me report MORE than seven**: `w-rdata` §4 states its seven is
a **lower bound** priced on the cheapest case only. I will not inflate the count
with the destructor shape's extra facts — the comparison to the incumbent has to
be like-for-like on the minimal 11-section obj.

---

## §3 Alarms — registered as MUST-NOT-MOVE

Baseline on master `e60f8902`, from the brief and to be re-read here:

* `mismatch` **0** — may not move under any circumstance.
* `fnbyte-exact` **36,209** — may not shrink.
* `differs` **2,111** — may not grow.
* `match-tu-differs` / `match-tu-reloc-differs` **0 / 0** — may not move.
* tests **1,116 / 36 targets / 0 failed** — a *shrunken* `targets=` reads
  exactly like a deleted test, so the target count is quoted, not just the pass
  count.
* gate **18/18 PASS, 0 mismatch**, run with `--require-graded`; **graded counts
  quoted, not exit codes**.
* scan **match 10 · mismatch 0 · vocab-gap 861 · capture-fail 7**;
  factors **A 28 · B 338 · C 169 · D 10 · E 2**, `B∧C` 151, `A∧B∧C` 27,
  FRONTIER 17, `writer-sections` 10.

`factor-c` **may** rise if and only if the writer ships, and then it is reported
**from a scan** with `writer-sections` beside it — never asserted. Board #301
exists because a `Section` literal in dead code inflated it once, and #927's
guard is the thing that now says no.

---

## §4 Predictions on the outcome itself

| | registered |
|---|---|
| **P8** `factor-c` reads **169** before, from a scan | 169 |
| **P9** `factor-c` reads **169** after — because P1–P7 predict a decline | 169 |
| **P10** TU match **10 → 10**. Even a *shipped* writer is `+0` TU match: #360/#362, `A∧B` = 27 all inside C, 0 of the 676 `.rdata$r` TUs in `D∨E`. **Quoted, not re-derived** | 10 |
| **P11** the port refuses rather than mis-emits, everywhere outside what a cell proves | invariant |
| **P12** `peerkeys.py` at both ends of the lane reports **no key moved** by me — the peer lane `w-front2` owns `codegen/`, I own `coff/` and the `.gl` data reader, and the four erasures this week were all *shared semantics with no textual conflict* | no delta attributable to this lane |
| **P13** the docs/board search finds ≥ 1 already-written artefact this lane would otherwise have rebuilt | ≥ 1 |

**P13 is registered because it has fired four times this week**, most recently
on a defect filed five days earlier with the fix prescribed, invisible to a
docs-only grep because it was a board row. `docs/BOARD.md` is searched by topic
here, not only by number.

---

## §5 What this lane will NOT do

* **Not** add `.rdata$r` to `PORT_WRITER_SECTIONS` without an emitter that has a
  caller reachable from `PortC2::build` — that is BREAK 1 and BREAK 2 of
  `w-rtti`'s counterfactual, and both are test failures now.
* **Not** re-derive `OBJ_RDATA_R_SHAPE.md`. It is consumed, including its two
  corrected claims (§3.1 `??_8` ≠ `??_7` and Selection 6 is `/GR`-conditional;
  §6.1 the group is cut at the vftable run, not the `.text`).
* **Not** repeat the two errors the brief names: `.rdata$r` is **RTTI, not EH**
  (Phase 5 moves C by zero), and the ladder's 590 is **reachability, not a
  writer figure**.
* **Not** touch `crates/c2-core/src/codegen/` — `w-front2`'s seam this wave.
