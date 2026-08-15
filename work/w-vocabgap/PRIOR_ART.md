# w-vocabgap — the pre-brief grep, and the honest timing of it

**Timing, stated first because it matters.** `PREREG.md` was authored **before**
this sweep returned. Two of its predictions (**P-F**, **P-H**) are about an
instrument this sweep shows was **already refuted at frontier scale** — board
**#421**. They are scored as written; the prereg is not edited to fit.

The brief warned that *"a lane's given grep terms describe its FRAMING, not the
phenomenon"* — the fifth pre-brief-grep failure, and the first where the prior
art was a table of the exact phenomenon (`IL_DECODE_REACH.md` §5). So the sweep
was run on the **phenomenon** in several vocabularies (per-TU union /
cardinality / arity / breadth / set cover / max coverage / converts-alone /
singleton blocker set / "one key away"), over `docs/*.md`, `docs/rungs/*.md`,
`docs/BOARD.md` **rows separately**, `scripts/`, and `crates/c2-harness/src/gap/`,
**oldest hit read last**.

---

## 1. The instrument exists, once, at frontier scale — and it is REFUTED

`work/w-dclass/rerank.py` (2026-08-05), docstring line 2: *"the FRONTIER
re-ranking instrument, keyed on **BLOCKER SETS**."*

```python
def convertible(rows, closed):
    return [s for s, b in rows.items() if set(b) <= closed]
```

That is this lane's `S(t) ⊆ G`, written eleven days ago. It carries `solo`
(*"TUs whose ENTIRE blocker set is this one key (a **CONJUNCTION**)"*) against
`appears` (*"a **MARGINAL** — not a conversion"*), a greedy max-coverage ladder,
and a control for *"frontier TUs with ZERO blocker keys (must be 0)"* — which is
this lane's **P-J**, already written down.

`FRONTIER` is a **hard-coded 19-entry list**. The scan said **16** by #1404 and
says **2** today.

### 1.1 Board **#421** — the refutation, and it is of the method, not the run

> `rerank.py`'s greedy ladder **OVER-COUNTS**. `set(b) <= closed` assumes closing
> a key *removes* it; **a blocker key is the label on the FIRST refusal, so
> closing it SUBSTITUTES a successor.** Ladder-credited **5 TU, measured 0 TU**.
> 9 of 19 frontier TUs substituted a relational key for a branch key.
> Correction: `work/w-cmp/substitute.py`. **Workflow: sink first, re-rank
> second, build third.**

`PREREG.md` §2.1 item 1 registers exactly this bound, from `#3095` rather than
from `#421`, and independently: *"`S(t)` is a FIRST-blocker set, so key-covering
`t` is NECESSARY and NOT SUFFICIENT."* **The lane's own three-scan ladder is
`#421`'s "sink first" in its published form.**

### 1.2 The four rankings of this shape that are already refuted

| board | ranking | outcome |
|---|---|---|
| **#421** | `rerank.py`'s greedy blocker-set cover, 19 TUs | credited 5 TU, **measured 0** |
| **#441** | the greedy blocker-key ladder generally | *"the head never converts — the head is a **FIXED POINT** of the sink operation"*; head migrated `expr-cmp-eq` 3 → `expr-brfalse` 5 → `expr-op-0x53` 4 (a **scope-open bracket**) → `expr-op-0x27` 2 |
| **#681** (`w-build`) | the **UNION over chain inventories** as a coverage ranking | *"the **NINTH** refuted ranking"*; the operator `w-tu1` actually built (`op:09`) ranks **30 of 31**, coverage 1 |
| **#3131** (`w-read2`) | greedy head-mass generally | 19 rungs, reach **0** at every one; *"this invalidates the **INSTRUMENT**, not one result"* |

`STRATEGY_REVIEW_2026-08-13.md`: **11 refuted selectors, 12 refuted placement
rules, a 12-deep allocation-key graveyard.** A per-TU set-cover ranking
published as a plan would be **selector #12**. `PREREG.md` **N1** declines it,
and that decline was registered before this sweep.

---

## 2. The per-TU cardinality readings that DO exist — and they are all frontier-sized

| source | quantity | published number |
|---|---|---|
| **#420** | per-TU distinct blocker-key count, 5 named TUs | *"those five sit in TUs carrying **10 to 90** other distinct blocker keys each"* — the only reading at this magnitude anywhere |
| `w-frame` §2/§3 | per-TU **union of missing constructs** over four axes, 17 frontier TUs | median **5**; *"the cheapest frontier TU is five"*; **22 constructs occur in exactly one TU**; ρ = **+0.295** against the blocked-function mass ranking; self-verdict *"EXCLUDES CORRECTLY AT THE TAIL AND DOES NOT RANK THE HEAD"* |
| `w-depth` **#666** | per-TU construct SET (chain inventory) | *"a correct **CONSTRUCT INVENTORY** and a wrong **COST MODEL** — its content named the conversion its cardinality mis-ranked"* |
| **#667** | frontier aggregate | **21 of the frontier's 45 blocked functions and 8 of its 18 TUs name nothing** |
| `w-front3` P-LOSS | the union framing, stated | *"a ladder round closes a key for **every** blocked function at once, so the count is the **union** of the per-function chains, not their maximum and not their sum"*; `keygen_xbox` union **15**, `mmio` union **7** |
| **#212** | converts-alone sweep, **factor-A axis** | *"There is no lever inside factor A — **every single bucket, closed alone, converts 0 TUs**"* — this lane's **P-D**, on a different axis |
| **#1346** | singleton blocker set at CFG-class granularity | *"`cflow-if-n` … **6 frontier TUs need it and nothing else**, and it still converts zero"* |

## 3. The one place the method HAS been run at full scale — a different axis

`w-factors` (2026-08-02) / `w-bc` §4: the per-TU **missing-section** set over
871 TUs, with both a distribution over sets and a greedy coverage curve.

> *"The commonest beyond-reach extra-set is `{.bss, .data, .rdata$r}` — **352
> TUs** … **20 distinct extra-sets** in all."*

Curve: `.data` 109 → `.rdata$r` 172 → `.bss` **574** → `.text$yd` 698 →
`.xdata$x` 745 → `.CRT$XCU` 745 (**+0**) → `.text$yc` **871**, with the standing
caveat *"greedy is not proven optimal … an upper bound"*.

**So the per-TU-set method is not new to this repo at 871-TU scale. What is new
is the axis**: it has never been run on `emit_blockers`.

---

## 4. NEGATIVE RESULTS, in the words they were asked for

- **Nobody has ever computed a per-TU `emit_blockers` union or a set-cover over
  the full 878-TU workload.** Every per-TU blocker-set computation in the repo
  is restricted to the frontier (19, later 18/17/16, today **2**) or to a
  hand-named 7- or 25-TU subset.
- **No per-TU blocker-cardinality histogram exists** — no "how many TUs have 1
  key / 2 keys / …" table anywhere in `docs/`. The closest is #420's
  parenthetical over five named TUs.
- **No coverage curve in the "grant the top-K globally, count the TUs with zero
  blockers left" form** on this axis.
- **The scripts whose names suggest it do not do it.** `scripts/w_tu_distance.py`
  is `fn_total − fn_in_class` **counts**, never sets. `scripts/w_tu_emitset.py`
  is `.ex` segments vs `.text` COMDATs. `scripts/rerank_board.py`'s
  `scan_totals` sums `emit_blockers` **across all TUs into one Counter**,
  destroying the per-TU set by construction.
- **`report.rs` publishes only the mass.** `emit_blocker_histogram` is
  `merge_counts` across TUs; the per-TU maps reach the JSONL and nothing
  aggregates them per TU.

## 5. Standing doctrine this lane is measuring, not proposing

- `CFG_SHAPE.md`: *"A TU converts only when **every** blocked function in it
  decodes end to end"*; *"`xboxmem.cpp` has four functions and converts only
  when **all four** do."*
- `_2026-08-02-w-phase6-prereg.md`: *"…set is the **set union over its blocked
  rows**, and the conversion question is…"* — the earliest statement of the
  framing found, 13 days before this lane.
- `PHASE6_RANKING.md`: *"**No marginal is ever multiplied by another.**"*
- `GAPS.md` §"intersection": *"The intersection is a **per-TU** bound, not a
  product of marginals"* — marginals said 30, the joint is **8**.
