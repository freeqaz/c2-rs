# PREREG — `w-price4a`: RE-PRICE row 4a(i) / I1 against inputs now known wrong

    Tag:       w-price4a
    Date:      2026-08-26
    Kind:      characterization lane — docs-only, ZERO `crates/` bytes
    Base:      f202268f6 (c2-rs master, clean)
    Rows:      #3603–#3608 (reserved by decision 14; minted in the commit that uses them)
    Fixtures:  none — characterization lane: it re-prices a planning row and
               reads landed measurements; it claims no fixture and builds nothing
    Census:    +0 (no `crates/` file is written; no acceptance predicate exists to move)
    Reach:     +0 emitted functions, +0 TUs — required, and it is not this lane's grade
    Funded by: `docs/DECISIONS_2026-08-22.md` decision 14 (owner, 2026-08-26)
    Fence:     WRITES only the top-level pricing docs — `ARCHITECTURE_PROPOSAL_2026-08-20.md`,
               `STEP5_PRICING_2026-08-21.md`, `ROADMAP_SLICING_2026-08-21.md`,
               `WHITEBOX_LEVERAGE_2026-08-21.md`, `ROADMAP.md` — plus `BOARD.md`,
               this file and its rung. **MAY NOT WRITE `docs/whitebox/`** (that is
               `w-opclass`'s fence this wave). `w-opclass` is in flight and may
               sharpen limb 2 of the arm map: this lane prices off what has
               **LANDED at f202268f6** and does not wait.
    Status:    REGISTERED BEFORE ANY ARITHMETIC WAS DONE, BEFORE ANY SCRIPT WAS RUN,
               AND BEFORE THE ENUMERATION SWEEP REPORTED

---

## 0. The question, stated so it can come back "the price does not move"

Row **4a(i)** / **I1** — *a general op-level IL decode* — is the critical path
(`ARCHITECTURE_PROPOSAL_2026-08-20.md` §5 row 4a, §8 decision 0;
`STEP5_PRICING_2026-08-21.md` §2/§4). It carries **three** published cost
figures which are three different things and are routinely quoted as one:

| figure | what it prices | where |
|---|---:|---|
| **15–25 days** | the **READ** `R5` that would *spec* I1 | `WHITEBOX_LEVERAGE_2026-08-21.md` §3 row R5 |
| **1.5–4.5 eng-months** | I1's **raw build** estimate | `STEP5_PRICING_2026-08-21.md` §2 |
| **7.5–22.5 eng-months** | that raw figure × `CEILING.md` §5's ~5:1 | same |

Decision 14 states that **three of its four inputs are now known to be wrong**.
The lane's job is to say **which direction each moves the price, whether they
cancel, and what the amended price is** — in the units the goal is written in,
declining a point estimate if the honest answer is a range or "unresolved",
and naming what would resolve it and at what cost if so.

**This lane may NOT decide Phase 1** (decision 11's hold is the owner's;
decisions 13 and 14 each explicitly declined to rule on it). It may not
re-price 4a(ii) / I2, 4b / IR3 or step 5 except to state how they inherit.

---

## 1. THE INPUTS AS BELIEVED NOW — read out of the tree at `f202268f6`, not remembered

Every row below is what the lane will attempt to **reproduce independently**
before relying on it. The whole reason this lane exists is that a published
figure was wrong three ways, so inheritance is banned (`CLAUDE.md`; #3567).

| # | input | as the price was written | as wave 11 landed it | source of the correction |
|---|---|---|---|---|
| 1 | dispatch size | **189 arms** | **61 real arms serving 95 opcodes, + 1 refusal arm serving 94**; 62 targets, **62 distinct** | `whitebox/WB_ILARMS_MAP.md` §1 (`w-ilarms`), decision 13 |
| 2 | off-model share | **83.5 %** | **88.61 %** — 83.5 refuted by its own table's arithmetic | `ROADMAP_SLICING_2026-08-21.md` §3.0 (`w-decodereach`) |
| 3 | decode reach | **98.2 %** | that is **FRAME** reach (98.25 %); **MODEL** reach is **11.39 %** of bodies / **5.47 %** by byte | `IL_DECODE_REACH.md` banner, §3.0 |
| 4 | port-side mapping | (unstated) | 41 of 61 arms have ≥1 port reader, 68 of 95 opcodes read — **but that is cursor advance, not decode**; **no port site anywhere mints an IR node**; `P_ILRECORD.md` §8.1's 76 tree builders are **0 of 76 read** | `WB_ILARMS_MAP.md` §0, §3 |
| 5 | layer count | 1 fused layer (decode ∧ admission) | **3** — `w-unfuse` split decode from admission; `w-decodereach` found symbol binding under it, 4,001 bodies at `shape_to_function` | decision 14, `w-unfuse` / `w-decodereach` rungs |

**Landed inputs the lane will check are NOT already counted** (they predate
wave 11 and may or may not have reached the pricing docs): `S0`'s
`blind-exact 15 / differs 373 / reach 0.342 %` (#3392–#3393 — which
**declined** to re-price 4a in either direction, by a prereg-frozen rule),
`#3509`/`#3529`'s Phase-1 TU reach 0, `w-joint3`'s 97.2 % construct floor,
and `w-c1`'s measured ~0.2 engineer-days against a 2–4-week raw price.

---

## 2. REGISTERED PREDICTIONS — frozen before the arithmetic

Scored honestly in the rung's §1. A prediction that misses is written **MISS**
in those words.

| # | prediction | p |
|---|---|---:|
| **P1** | The corrected inputs **do NOT cancel**. Net direction on I1's *build* price is **UP** | **0.80** |
| **P2** | Inputs 2 and 3 are **the same measurement counted twice** — `83.5 %`'s complement and MODEL reach are one quantity, so the number of *independent* corrections is **3, not 4** | **0.70** |
| **P3** | The `189 → 61` correction lands almost entirely on the **READ** (R5, 15–25 d) and moves I1's **build** price by **no measurable amount**, because the arm count is not I1's denominator — the unread tree builders and the off-model body mass are | **0.60** |
| **P4** | I will **decline to publish a replacement point estimate or range** for `7.5–22.5 eng-mo`, and publish instead a direction + a named, priced resolving measurement | **0.55** |
| **P5** | `CEILING` §5's ~5:1 was fitted on **lane/rung-shaped forward cost** (frontier depth, refusal counts, rung counts) and **not** on engineer-months, so applying it to I1 is **outside its fitted domain** — a units caveat this tree has stated for *reads* (`STEP5_PRICING` §4) and never for eng-months | **0.50** |
| **P6** | The enumeration sweep finds **≥ 2 LIVE pricing surfaces** carrying 4a(i)'s number that a topic grep on `15–45` / `189` would not reach | **0.40** |
| **P7** | At least one landed pre-wave-11 measurement (S0, #3509, `w-joint3`, `w-c1`) bears on 4a(i)'s price and appears in **none** of the five pricing docs | **0.55** |
| **P8** | The re-price **bears on Phase 1** — and the lane will say so and stop, per its fence | **0.45** |

**Registered self-grade failures** (any one and the lane reports `FAILED`):

* Publishing a single number where the inputs do not support one, or hiding an
  answerable question behind the word "unresolved" without naming what would
  resolve it **and what that costs**.
* Amending by rewrite or deletion instead of `~~struck~~ **correction**` in
  place (`DOC_CONVENTIONS.md` §2), or touching a **dated record** —
  `ROADMAP.md`'s historical sections and any existing rung. `ROADMAP.md` §11
  is the live framing and MAY be amended.
* Any non-zero `git diff --numstat f202268f6..HEAD -- crates/ scripts/ fixtures/`.
* Writing any file under `docs/whitebox/`.
* Sweeping the consumers by grep alone. The sweep is by **enumeration of the
  file universe first**, then by token, then classified — because `w-ilarms`
  found a banner-named consumer list short by two, and `w-decodereach` found
  that every *other* `98.2 %` in the tree is a different 98.2 % and a grep
  would have swept four unrelated sites.
* Deciding Phase 1, or re-pricing 4a(ii)/4b/step 5.

---

## 3. METHOD — what will be reproduced, and how

1. **Input 1 re-derived from the image**, not from `WB_ILARMS_MAP.md`:
   `python3 docs/whitebox/scripts/dump_ilarms.py <c2.dll> --verify`, sha256 of
   the DLL quoted. Expected: 189 opcodes, 62 targets, 62 distinct, 61 real,
   94 routed to the refusal arm.
2. **Input 2 re-derived in one line of arithmetic** from
   `ROADMAP_SLICING` §3's own table: the ten constructs sum to `S`; the table
   states cumulative 97.3 %; so its denominator is `S / 0.973`. Compare against
   `0.835 × 2,404,438`. If the latter is smaller than `S`, 83.5 % is refuted by
   the table it heads.
3. **Input 3** — `decode-reach-*` are standing `gap-metric` keys. Quote them
   from a scan if one is affordable in this lane; otherwise quote them from the
   landed instrument **with the workload stamp** and say plainly that this lane
   did not re-run the 878-TU scan. Never quote a reach figure without its
   strength (FRAME / MODEL / GRAMMAR) and its denominator.
4. **Input 4** re-derived: `scan_port_opcodes.py --coverage`, plus an
   independent grep of all five crates for the node-opcode space (`≥ 0x2af`)
   to confirm **zero non-comment hits**.
5. **Input 5** read from the landed `AdmissionPolicy` seam and the
   `grammar-not-admitted` key.
6. **CEILING §5's calibration**: read what it was fitted on **before**
   applying it (the lane brief's instruction, and P5).
7. **The consumer sweep by enumeration** (see §2's self-grade failure).

---

## 4. WHAT THIS LANE DOES NOT DO, stated so absence is not read as coverage

* **No conversion, no census move, no `crates/` byte.** Predicted reach 0.
* **No decision on Phase 1**, on 4a(ii)/I2, on 4b/IR3, or on step 5.
* **No re-run of the 878-TU workload scan** is promised; if one is not run, every
  reach figure is quoted with its source stamp and that fact is stated.
* **No new whitebox reading.** `w-opclass` owns `docs/whitebox/`; anything this
  lane would have written there is **reported, not written**.
