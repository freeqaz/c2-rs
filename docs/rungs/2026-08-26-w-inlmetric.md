# w-inlmetric — the inliner's scoreboard: 17 of 24 clauses have NO port counterpart, and the absence is now VERIFIED rather than inferred

    Tag:       w-inlmetric
    Slug:      w-inlmetric
    Date:      2026-08-26
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization + instrument lane: it grades an existing predicate and reads the binary, and writes zero crates/ bytes
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Record:    this file; prereg `work/w-inlmetric/PREREG.md`, committed at `f50f5112c` before the first measurement

Charter: `docs/DECISIONS_2026-08-22.md` decision 15, the `w-inlmetric` row —
the owner's named exemplar, *"inliner is extremely valuable to understanding how
that logic works in the compiler."* Dispatched at master `6c753ead0`.

> **Everything this lane publishes is a PROGRESS INSTRUMENT and never a gate**
> (`docs/FUNCTION_BYTE_MATCH.md` §0). It licenses no emit, moves no admitted
> set, and **wrote zero `crates/` bytes** — `git diff master..HEAD -- crates/`
> is empty.

---

## 1. The result

**The inliner's conformance table is 24 clauses. Four have any counterpart in
the port at all: two `[R]`-derived, two fitted. Seventeen are absent. Three are
unexercisable.** Published at `docs/whitebox/ref/P_INLINE.md` §6.1, machine
source `work/w-inlmetric/CLAUSES.tsv`.

| | count |
|---|---:|
| `[R]`-derived | **2** — C13 (legality bit 6 of `[sym+0x4c]`), C24 (the `.gl` `SIZE` field) |
| fitted | **2** — C8 (the size test), C20 (the expansion's recursion) |
| **absent** | **17** |
| unexercisable | **3** — the POGO model, its per-site discount, the parameter tables |

**The word that matters is `verified`.** Every `absent` row names a token that
the grader confirms is **not present in `crates/`**. Before this table, an
absent clause was inferred from the port not mentioning it — which is
indistinguishable from nobody having looked. That is the whole of the w-root
mitigation pattern, applied to the subsystem the owner named.

**Three further results, each measured here:**

* **`INLINE-P` re-graded on a hold-out re-frozen by CONTENT: `0.9678` at
  `n = 8,936`** (dc3 `15a64d92f`, 0 dirty). `#3045`'s drift **did not
  continue** — 24 of the 100 listed TUs changed their own source and the
  population moved **+0.22 %**.
* **`INLINE-P` degenerates to a single threshold on this workload.** SCHEDULE
  D's graduated middle fires on **0 of 8,936** callees; the STATIC arm's
  workload population is **1**.
* **§2.1's `0x2000` `__forceinline` mask carries after all — it is `edi`**,
  materialised at `0x10b5fc31`. An explicitly-open half of `w-sizebracket`'s
  correction, discharged by one bounded read.

---

## 2. Deliverable 1 — the conformance table

`docs/whitebox/ref/P_INLINE.md` §6.1 (24 rows) · `work/w-inlmetric/CLAUSES.tsv`
(machine source) · `work/w-inlmetric/check_table.py` (grader) ·
`work/w-inlmetric/POSITIVE_CONTROL.md` (the grader watched failing).

Three checks run over every row: **ADDRESS** (the cited VA must lie inside the
function the page claims, per `FUNCS.tsv` entry+size), **WITNESS** (a
counterpart row's `path:token` must exist), **ABSENCE** (an `absent` row's
`none:<token>` must be absent from `crates/`).

**The grader was watched failing before its green was quoted** (PREREG §6).
Three verdicts planted at once, one per check, each caught by its own check and
no other:

```
  FAIL C7:  state absent but token 'INLINE_UNBOUNDED_BYTES' IS PRESENT in crates/
  FAIL C8:  ADDRESS 0x10b5fe14 is in FUN_10b5fcd8, table claims FUN_10b5fb5f
  FAIL C14: WITNESS 'INLINE_MAX_DEPTH' NOT FOUND in crates/c2-core/src/splice.rs
CONFORMANCE-CHECK: RED  (3 failure(s) over 24 rows)
```

reverted →

```
CONFORMANCE-CHECK: GREEN  (0 failure(s) over 24 rows)
```

Plant 2 is the one to keep: it re-injects `w-sizebracket`'s exact defect, and
`P_INLINE.md` §2.1's CORRECTION block is that comparison done by hand, once,
*after* four addresses had already shipped in the wrong function.

### 2.1 …and running the ADDRESS check found the SAME defect uncorrected in a second file

`work/w-inlmetric/addrcheck.py` over all **29** addresses `P_INLINE.md` §1–§2
and `WB_INLINE_FINDINGS.md` §1–§2.4 cite: **4 wrong, 25 right.** The four are
`0x10b5fdfd / 0x10b5fe0c / 0x10b5fe14 / 0x10b5fe1e` — `w-sizebracket`'s four,
reproduced mechanically.

**But `w-sizebracket` wrote its correction on `P_INLINE.md` only.**
`WB_INLINE_FINDINGS.md` §2.1 has carried the same four wrong addresses
uncorrected for eight days, **and §10's pre-drafted `W-INLINE-1` DISCLOSURE row
quotes two of them in bold** as the addresses a future code lane would carry
into the provenance ledger. Nothing in `crates/` is affected (the row is not
adopted). Amended beside, this lane, `docs/whitebox/WB_INLINE_FINDINGS.md` §2.1.

> **The general form, and it is a methodology finding rather than an inline
> one:** a correction written on the page where the error was *found* does not
> reach the page that *repeats* it. The fix is not more diligence — it is that
> the check is now a program that runs over every row of the table.

### 2.2 The five readings the table is worth more than a percentage for

1. **The two fitted rows are fitted to a different QUANTITY than c2 tests.**
   c2 compares a pre-codegen instruction count (`WORD [sym+0x50]`, the
   `"%d instrs"` string); the port's three ceilings — `INLINE_UNBOUNDED_BYTES
   = 64`, `INLINE_DECLINE_BYTES = 128`, `INLINE_DECLINE_LOOP_BYTES = 80` — are
   **lowered byte counts**, every one obj-fitted. `INLINE_DECLINE_LOOP_BYTES`
   exists *because* of that unit gap: a loop body priced in emitted bytes is
   over-credited ≈ 1.55× (F9).
2. **C24 is the sharpest row.** The port already **decodes the field c2's
   decision tests** (`GL_SIZE_ESCAPE_PAYLOAD`, `DISCLOSURE` row `W-GLATTRS-1`,
   at `mismatch 0`) **and discards the value.** This is *not* a recommendation
   to consult it — §2.1b measured `SIZE` as an **upper bound** on the tested
   quantity, with `arith_012` and `mix_008` at an identical `SIZE` of 115 and
   opposite verdicts.
3. **C13 is the only row where two independent derivations MET** — the
   disassembly's *"requires bit 6 of `[sym+0x4c]`"* and `w-mmioclose`'s
   container-side `0x40`, which closed a shipped wrong emit (#2402).
4. **`__forceinline` is the biggest asymmetry and it is directional.** c2's C10
   is an **accept** clause that bypasses every size and budget test (F4: a
   980-byte callee). The port has **no accept path anywhere**. `WB` §7's *"the
   accept side is not offered"* is not a policy the port adopted — it is the
   port's entire relationship with this subsystem.
5. **Six clauses are `not-separable` rather than untested**, all of them the
   budget family (C2/C3/C4/C17/C18/C19). F7 moved the caller 48 B → 5,640 B and
   nothing changed on 12 cells. `P_INLINE.md` §3.1 already files the budget
   `READ, NOT CONFIRMED`; the table now says the same thing in a cell rather
   than in a paragraph.

---

## 3. Deliverable 2 — `INLINE-P`'s hold-out, re-frozen by CONTENT

`work/w-inlmetric/refreeze.py` → `work/w-inlmetric/sample_b_frozen.tsv`:
100 rows, each TU's `sha256` and byte length, at a named dc3 stamp.
`sample_b.txt` is byte-identical and unchanged — **re-freezing by content is not
re-selecting.**

    dc3 15a64d92f1975868e55a1c670d312a8e464074c3  dirty=0  rows=100  missing=0
    sample_b.txt sha256   = c2eeba0cb9689266449bedf553ff69e76812930cb71ece949d6f9c317699904e
    CONTENT-FREEZE sha256 = 278c0afac96b0580293fa93dbb3d696fb42c974b63030da3c7c9faa85f43952a

Re-graded through the `work/w-inline/grade_pair.py` lineage, `built: 100 of 100`
at both arms, `dropped (not in both objs): 0`:

| | published | `#3045` (dc3 `2277bb73ef23`) | **this lane (dc3 `15a64d92f`)** |
|---|---:|---:|---:|
| **accuracy, leaf dropped** | 0.9716 | 0.9681 | **0.9678** |
| **graded callees** | 9,993 | 8,916 | **8,936** |
| source-leaf | 0.9688 | 0.9650 | **0.9646** |
| `/O1`-obj leaf | 0.9631 | 0.9586 | **0.9585** |
| majority baseline | 0.6434 | — | **0.7020** |
| precision / recall | 0.969 / 0.988 | — | **0.9678 / 0.9869** |

**The finding is the denominator, and it is the opposite of `#3045`'s.**
Between the two stamps dc3 moved **727 files**; **24 of the 100 listed TUs
changed their own source**; **159 headers** under `src/` changed; **33.0 % of
the graded callees live in a TU whose `.cpp` churned.** The population moved
**+0.22 %** and the accuracy moved **−0.0003**.

**So `#3045`'s 10.8 % was a one-time event, not ongoing drift** — and the two
readings together say something neither says alone: `INLINE-P` is considerably
more robust to corpus churn than the figure that prompted the warning
suggested. **The warning stands anyway.** A rate without its denominator is
unreadable, and the freeze is what makes the *next* drop attributable:
`work/w-inlmetric/per_tu_population.tsv` carries per-TU graded and hit counts,
so a future collapse names its TUs rather than only its size.

**Two limits of this freeze, stated so it is not over-trusted.** It hashes the
**`.cpp` only** — 159 headers moved in the same interval and a header change
moves a TU's callee set without touching its hash. And **4 of the 100 TUs
contribute zero graded callees**, so the effective corpus is **96 TUs**, and
always was.

---

## 4. Deliverable 3 — the 4-tuple

| # | strength | value | denominator |
|---|---|---|---|
| 1 | **read** | **13 / 93 = 14.0 %** functions (`16` §1 entries; `17/93` with `cited`) | `FUNCS.tsv`, band `0x10b5b86d`–`0x10b62b00`, this tree |
| 2 | **agreement** | **4 / 24** clauses have any counterpart (2 `[R]` + 2 fitted); `INLINE-P` **0.9678** | 24 clauses; **8,936** graded callees |
| 3 | **exercised** | **9 yes · 6 no · 6 not-separable · 3 unexercisable** | the hold-out's 8,936 callees, dc3 `15a64d92f` |
| 4 | **byte-owned** | **CITED, NOT RE-MEASURED — `#3534`, 2026-08-25** | see §4.2 |

### 4.1 `read` — the published coverage line mixes two units

**93 replicates exactly.** But `cover=paged` is **13**, and §1's sixteen rows
are 13 functions **plus 3 addresses interior to another row** (`0x10b626d8` and
`0x10b6276a` inside `0x10b62675`; `0x10b600c8` inside `0x10b5fcd8`) — the page
marks each *(in `0x…`)* itself. So `16 / 93` reads as 17.2 % and the function
coverage is **13 / 93 = 14.0 %**. `SUBSYS.md` §1's inliner row publishes
`16 / 93` and inherits the same mix. Nothing is wrong with the 16; what is
wrong is reading it against 93 as a rate. All three readings published together
(`P_INLINE.md` §6.0) so the next lane picks one deliberately.

### 4.2 `exercised` — and the structurally unexercisable cells, with reasons

```
   N_max == UNBOUNDED : 6,397 = 71.59 %
   N_max == 0         : 2,539 = 28.41 %
   N_max FINITE, non-zero (SCHEDULE D's graduated middle FIRING) : 0 of 8,936
   EXTERNAL 8,935  ·  STATIC 1  ·  varargs 0
```

**The whole of SCHEDULE D — `min(9, 1 + floor(19/(i−16)))`, the most elaborate
object in `INLINE_PREDICATE.md` §2 and the subject of round 31 — fires on ZERO
of 8,936 workload callees.** It is STATIC-only and the STATIC arm's workload
population is **one**: `?ModChan@@YAHH@Z` in `src/system/rndobj/ColorXfm.cpp`.
`n_sites > 1` **is** exercised (2,910 callees, up to 31 sites) but only in the
EXTERNAL class, where `N_max` is UNBOUNDED-or-0 and the site count changes no
verdict. **On this workload `INLINE-P` is a single threshold.**

**`#3066` is cited and deliberately NOT re-derived.** It reads *"the port's
largest lowered body is 152 B and c2's static-inline floor is > 308 B, so the
windows do not overlap"* — a claim about the tree at that rung, and **`#3063`'s
standing lesson is that such a control is re-derived before it is made
mandatory**. Re-deriving the **port** half needs a full port scan this lane did
not run, so that number is quoted as `#3066`'s and not as this lane's. What
**is** measured here is the same non-overlap from **c2's** side, and it is
stronger: the STATIC clause has a population of 1 in 8,936 whatever the port's
ceiling is.

C21–C23 are the only genuinely **unexercisable** rows: `/GL` and
`profile-guided` appear in **0** dc3 source files, and both 46-dword tables sit
above the image's raw `.data` — zero at load, unquotable. C9 (favour-speed) is
unexercis**ed** and not unexercis**able**: `/O1` pins the bit here, and GRID-I
moved it at `/O2` on 60 cells. C15 is `0xff` throughout — `#pragma
inline_depth` and `#pragma auto_inline` appear in **0 of the 100** TUs.

**An unexercisable cell is not a covered one, and neither is an unexercised
one. Nine of twenty-four is what this workload can grade.**

### 4.3 `byte-owned` — and the thing this lane may not quietly reverse

`#3534` measured it on 2026-08-25 and decision 15 forbids re-taking it. Stated
here because a richer inline scoreboard could be *read* as an argument to
re-open what `#3534` closed, and it is not one:

**`#3534` flipped OFF the inline-decision permuter**, on a measurement in both
directions on one tree on one day. The port's wrong bodies are **99.87 % opcode
substitutions with 0 reorderings**; the permuter's actual working set is
**2.14 % opcode, 52.50 % register, 7.90 % pure reorderings**.
`INLINE_PREDICATE.md`'s model and `splice.rs`'s S7 are **right for the port's
population and wrong for the permuter's, both by measurement.** Nothing in this
rung reverses that, and §4.2's finding — that `INLINE-P` degenerates to a single
threshold on the workload — **strengthens it rather than weakening it**: a
decision surface with two states is a poor thing to search.

---

## 5. Deliverable 4 — the read, priced, and ONE taken

Priced from `FUNCS.tsv` before anything was read
(`work/w-inlmetric/read_price.txt`):

| | |
|---|---|
| band functions | 93 |
| read (`cover=paged`) | **13**, 6,639 B |
| **unread** | **80** (76 if `cited` counts), **22,840 B**, mean 285 B/fn |
| a total sweep | **3.4× the bytes already read** |
| cheapest candidates (hop ≤ 2, ≤ 120 B) | 24 functions, 1,236 B total |

> **The brief's precondition was not met, and that is the honest report.** It
> asks for a read *"if a clause's port-state is unknowable without reading more
> of the unread band."* **No clause's port state is unknowable**: all 24 are
> determinable from `crates/` alone, mechanically, and `check_table.py` proves
> it on every row. What is unread is **c2's side**, not the port's.

**So the one read taken was not from the 80.** It was the **whole of
`FUN_10b5fb5f`** — 377 B, already inside the read 13, of which §2.1 had quoted
seven lines. Image `sha256` verified. It **discharged an explicitly-open item**:

* **§2.1's `0x2000` `__forceinline` mask DOES carry — it is `edi`**, `mov
  edi,0x2000` at `0x10b5fc31`, callee-saved and unclobbered through
  `test edi,eax` at `0x10b5fc95`. `w-sizebracket` retracted the address
  correctly and could not settle the mask. The **original** §2.1 reading was
  right in substance and wrong only in address and encoding.
* **The legality function is called FROM candidacy** (`0x10b5fc13`), so
  C11–C13 sit **inside** C8's function, not beside it.
* **There is NO linkage arm in the candidacy function** — all 377 B read, no
  storage-class field tested anywhere. §5 named *"most plausibly the linkage
  arm and the `[sym+0x50]`-vs-emitted-size gap"* as the unread piece of the
  `16 << k` puzzle. **One of the two candidates is eliminated.**

Full write-up `P_INLINE.md` §6.5; listing
`work/w-inlmetric/FUN_10b5fb5f.asm`.

---

## 6. PREREG, scored

| # | registered | outcome |
|---|---|---|
| **P1.1** | 18 clauses ±3 (accept 15–21) | **MISS.** The table is **24**. The prereg listed 24 candidates and predicted they would merge to 18; they did not, and merging them **after** seeing the split would have been fitting the instrument to its own result. Miss in the direction of *more* clauses — the decision function is richer than registered |
| **P1.2** | ≥1 clause un-citable at the brief's address, `p = 0.55` | **MISS, pessimistic.** 4 of 29 addresses fail — and all four are the four `w-sizebracket` **already** struck. **No new wrong address exists.** The gain is that the check is now mechanical, and it found the same four **uncorrected in a second file** (§2.1) |
| **P2.1** | `absent` is the plurality, ≥ 9 rows | **HIT**, and by a distance: 17 |
| **P2.2** | registered bias **OPTIMISTIC**; true `absent` ≥ 10 | **HIT.** 17 ≥ 10. The registered direction was right — I predicted more port counterparts than exist |
| **P2.3** | the single `[R]`-derived row is the legality bit, `p = 0.6` | **HALF.** Right that C13 is `[R]`-derived; **wrong that it is the only one** — C24 (the `.gl` `SIZE` field, `W-GLATTRS-1`) is a second |
| **P2.4** | nine named clauses have no counterpart | **8 of 9.** The miss is **the expansion's recursion**: the port's splice **is** a fixpoint (`S6-chain`, #1020, 150 relocation witnesses), so C20 is a real behavioural counterpart. Scored a miss |
| **P3.1** | accuracy in `[0.960, 0.975]`, point estimate 0.968 | **HIT**, 0.9678 |
| **P3.2** | the three leaf readings' ordering replicates, `p = 0.9` | **HIT**, third replication |
| **P3.3** | population moves > 2 % from 8,916 | **MISS — and the miss is the result.** +0.22 %, under 24 % source churn |
| **P3.4** | 1–8 of the 100 TUs missing or non-compiling | **MISS.** `missing: 0`, `built: 100 of 100`. The registered alternative — *"if it is 0, the drop was entirely within-file churn, and that is a sharper finding"* — is the one that happened |
| **P3.5** | the regression/improvement reading, fixed in advance | **APPLIED AS WRITTEN.** Inside the bracket, population moved ≤ 2 % ⇒ **a stable rule on a moved corpus**; neither a regression nor an improvement |
| **P4.1** | `16/93` re-measures within ±3 / ±1 | **HIT on the 93, exactly** — and it exposed that the 16 and the 93 are **different units** (§4.1), which the prediction had no room for |
| **P4.2** | ≥ 4 of the clauses structurally unexercisable, with reasons | **HIT, 3 unexercisable + 6 unexercised + 6 not-separable = 15 of 24 ungradeable here.** POGO and depth-16 were both named in advance; the STATIC-population-of-1 finding was **not** registered and is the strongest of them |
| **P4.3** | `#3534` cited, not re-measured; not reversed | **HELD.** §4.3 |

**Direction: 8 hits · 1 half · 5 misses.** Four of the five misses are
**pessimistic** (more clauses, no new address error, no population drift, no
missing TUs); one — P2.4's expansion recursion — is **optimistic** about the
port's absence and is the only place I under-credited `crates/`. That is the
opposite skew to `#770`'s standing ~11 optimistic / 2 pessimistic tally, and it
is what registering P2.2's direction explicitly was for.

---

## 7. What this lane did NOT do

* **Zero `crates/` bytes.** No decision rule, threshold or constant shipped.
  `git diff master..HEAD -- crates/` is empty.
* **No emission widening**, no admitted-set move, no gate row. Nothing here is
  in `gate.sh` and nothing licenses an emit.
* **Did not grow the clause table with §6.5's read.** At least three new
  clauses are visible in the listing (`[sym+0x4c] & 0x10` gating
  `[ebp+0xc] & 0xf00`; a second `0x200` test; the three-valued `0x10c3de20`
  selector). Adding rows discovered *after* the per-state split was predicted
  is fitting. Filed in §8 for a lane that pre-registers them.
* **Did not re-derive `#3066`'s port-side 152 B** (§4.2), and says so where it
  is quoted.
* **Did not re-measure `#3534`** (decision 15 forbids it) and does not reverse
  it (§4.3).
* **Did not enter the 80 unread band functions.** Priced at 3.4× the bytes
  already read and declined as an open-ended campaign, per the brief.
* **Did not write `docs/whitebox/READ_PLAN_2026-08-21.md`** — §8 reports
  instead, the `#3607` precedent.
* **Did not touch any peer's surface**: no `crates/c2-harness`, no
  `scripts/subsys_*`, no `docs/SUBSYS_METRICS.md`, no `DISCLOSURE.md`, no
  comment edits in the other crates.

---

## 8. Named follow-ups, deliberately not taken — FOR THE COORDINATOR

**These are reported, not filed.** `#3607`'s precedent: a lane may not write
`docs/whitebox/READ_PLAN_2026-08-21.md`.

1. **A READ_PLAN row: the three unregistered clauses in `FUN_10b5fb5f`.**
   `[sym+0x4c] & 0x10` gating `[ebp+0xc] & 0xf00` (`0x10b5fbfc`); the **second**
   `0x200` test (`0x10b5fc3a`, where §1 lists `0x200` only as a `0x10b5c06b`
   refusal bit); and `ds:0x10c3de20` as a **three-valued** selector tested at
   `0x10b5fbde` / `0x10b5fc4c` / `0x10b5fc69`, whose `== 2` arms call
   `0x10b9e796` with string `0x10b02588`. **Cost: 0 further bytes read** — the
   listing is committed at `work/w-inlmetric/FUN_10b5fb5f.asm`. What it needs is
   a lane that **pre-registers** them so the split is not fitted.

2. **A READ_PLAN row: where the linkage split actually lives.** §5's `16 << k`
   gap named two candidates; §6.5 **eliminates one** (no linkage arm in the
   candidacy function). The remaining question — what produces
   `INLINE_PREDICATE.md` §6.17.3's measured STATIC/EXTERNAL split — is now
   sharper and unowned. **Caution attached: §4.2 measures the STATIC arm's
   workload population at 1 of 8,936**, so this read buys *understanding*
   (goal 1) and provably **zero** workload reach. Price it that way or not at
   all.

3. **`SUBSYS.md` §1's inliner row publishes `16 / 93`** and inherits §4.1's unit
   mix. That file is not in this lane's fence. A one-line amendment naming the
   unit — or publishing `13 / 93` beside it — belongs to whoever owns it
   (`w-submetric` is building the per-subsystem instrument off exactly these
   denominators this wave, so it is the natural owner).

4. **`W-INLINE-1`'s pre-drafted DISCLOSURE row cites two wrong addresses in
   bold.** Not adopted, so nothing in `crates/` is affected; corrected
   beside in `WB_INLINE_FINDINGS.md` §2.1 this lane. **If it is ever carried,
   the corrected addresses must go into the ledger, not the struck ones.**

5. **Not taken, and priced as not worth taking: the `.gl` `SIZE` field as an
   inline input.** C24 shows the port already decodes what c2 tests. §2.1b
   measured `SIZE` as an **upper bound** on the tested quantity — `arith_012`
   and `mix_008` at identical `SIZE = 115`, opposite verdicts. Consuming it
   would be adopting a bound as the quantity, and a wrong emit scores below the
   refusal it replaced. **The one-sided form (`SIZE < T ⇒ inlined`) is sound and
   is a DECLINE-side rule only** — `WB` §7's standing constraint, unchanged.

---

## 9. Reproducing

```sh
python3 work/w-inlmetric/check_table.py                     # the conformance grader
python3 work/w-inlmetric/addrcheck.py --pairs <addr>:<owner> ...
python3 work/w-inlmetric/refreeze.py <dc3> work/w-inline/sample_b.txt out.tsv
work/w-inline/build_objs.sh     work/w-inline/sample_b.txt work/w-inlmetric/objA 16
work/w-inline/build_objs_ob0.sh work/w-inlmetric/objA/index.txt work/w-inlmetric/objB 16
python3 work/w-inline/grade_pair.py --a work/w-inlmetric/objA --b work/w-inlmetric/objB \
        --index work/w-inlmetric/objA/index.txt --both
```

**Stamps, per PREREG §8.** dc3 `15a64d92f1975868e55a1c670d312a8e464074c3`,
0 dirty. Flags `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` (+ `/Ob0` for the
site enumerator). Image `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
verified at the head of §6.5's read. **dc3 is a LIVE repo — a reading at another
stamp is not comparable to these.**
