# WB_MERGER4 — PREREG (R1, frozen before the first grep of the export)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address named here is an absolute
> VA in the image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0,
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> — **verified on `compilers/X360/16.00.11886.00/c2.dll` at the top of this
> lane** before any address was read. Navigation only; this lane adopts nothing
> into `crates/` and owes no `DISCLOSURE.md` row.

**Frozen state at the moment of writing.** The only bytes of `c2.dll` this lane
has looked at are the sha256 above. Every address below is quoted **from board
`#3103` / `WB_DAGCLIENTS_FINDINGS.md` §2 and §6 item 4**, not from any grep of
`~/ghidra-projects/export/c2/` by this lane. The first such grep happens
**after this file is committed**, and per `#3101`'s found-and-not-taken #2 the
first thing read will be **`0x10b35f88`**.

---

## 0. The question (board `#3103`, `w-dagclients`' own stated relocation)

`w-dagclients` established that `0x10b3b167` (K1, tail merge / cross jump) and
`0x10b3b41b` (K2, head merge / hoist) are a dependence-DAG **block merger**
running at `0x10b7ded5`, before the final schedule at `0x10b7df57`. It also
recorded, and did not chase:

> With `0x10b3b167`, `0x10b3b41b` **and** `0x10b3b5fd` all patched to
> `return 0` (`A123`), `dk_join3` — three predecessors of one join, each arm
> ending in the same `dc_c = 9` store — still emits **2** copies, not 3.
> `dk_loop_join` merges under the same ablation.

**Where does the residual collapse come from?** The named candidates are
`0x10b3c2cc`'s other call sites: `0x10b36805`, `0x10b38cd4`, `0x10b388eb`,
`0x10b3a253`.

**This lane's framing is deliberately wider than that list**, because `#3103`'s
own candidate set is a guess and the seam's record (`#1823`, `#3071`, `#3103`)
is that each level of this question has relocated rather than closed. The
question is stated as a disjunction over **four** causes, and the lane's job is
to discriminate between them, not to confirm the first:

| cause | statement |
|---|---|
| **C-A** | one of the four named candidates under `0x10b3c2cc` |
| **C-B** | something else inside the merger driver `0x10b3c6e5` / `0x10b7ded5` that the four-candidate list does not name |
| **C-C** | a `c2` pass **outside** `0x10b3c6e5` entirely (a later peephole, the branch/layout optimizer, the emitter) |
| **C-D** | **not `c2` at all** — the IL `c1xx` hands to `c2` already carries fewer than three copies, so nothing in `c2` ever had three to merge |

**C-D is registered first on purpose.** It is the cheapest to test, it is the
one cause under which "a fourth block merger exists" is simply **false**, and
no document in this seam has ever checked it. `w-dagclients` counted copies in
`c2`'s **output** and inferred a count in `c2`'s **input**; that inference is
exactly the shape of `#1823` and it has never been measured.

## 1. The positive question, in writing — what goes red

> **If a fourth merger exists inside `c2`, which cell of mine changes, and how
> do I know it is not the scheduler?**

The answer registered here, before any probe:

**The cell.** A function whose join has **three or more** predecessors ending in
the same store, compiled against **`A123B*`** — K1, K2, K3 *and* every
identified additional client ablated. The fourth merger is named iff there is an
image in which the copy count rises to the **source** count (3, or 4 on the
4-arm cell) while `A123` alone leaves it below. The motion is provably not the
scheduler's by `#3069` (15/15: no tuple crosses a branch or a call) and by
`dk_call_join`, whose common store is separated from the join by a **call**.

**The null the instrument must measure, not assume.** Reused from `#3100`: at
least one ablation image must patch a byte in a function that provably does not
run on the grid and come out **byte-identical to `A0`**. Without that cell the
positive deltas are not attributable.

**The loud-failure counter.** The final tables print the count of
**discriminating** cells — cells where at least two images disagree. A run
reporting **0 discriminating cells is a FAILED lane**, not a clean negative.
`w-dagclients` §6 item 3 (`dt_dep`) is the precedent: a cell whose control and
treatment agree has no discriminating power and is scored grey-zone.

## 2. Registered claims (R1)

Probability form. Scored in `WB_MERGER4_FINDINGS.md` §7.

| id | claim | p |
|---|---|---|
| **N1** | **C-D is the cause**: the IL `c1xx` emits for `dk_join3` at `/O1 /Oi` contains **fewer than three** copies of the `dc_c = 9` store, so `c2` never sees three | **0.35** |
| **N2** | **C-A is the cause**: ablating the four named candidates (individually or together), on top of `A123`, restores the source copy count on `dk_join3` | **0.30** |
| **N3** | at least one of `0x10b36805`, `0x10b38cd4`, `0x10b388eb`, `0x10b3a253` is **entered** (`ud2` at entry traps) on an optimized cell of this lane's grid | **0.85** |
| **N4** | at least one of the four named candidates **rewrites tuple links** — it calls `0x10bd38b0` or `0x10bd3892` (the splices), directly or through one level | **0.50** |
| **N5** | the fourth cause, whatever it is, is **favor-size-independent**: the residual collapse is present at both `/O1` and `/O1 /Ot` | **0.60** |
| **N6** | a cell exists where `A0` and `A123` **agree** and the fuller ablation **disagrees** — i.e. the fourth cause has its own red cell, distinguishable from K1/K2's | **0.55** |
| **N7** | reading `0x10b35f88` yields a **nameable structural condition** for K3's second block — a shape statable in C, not just "a search" | **0.70** |
| **N8** | K3 (`0x10b3b5fd`) is made to **fire** by at least one cell of this lane's grid (`FK3` traps), closing `w-dagclients` §6 item 1 | **0.30** |
| **N9** | the number of duplicate arms matters: the count of copies surviving full ablation is **not** a constant across the 2 / 3 / 4-arm cells | **0.65** |
| **N10** | `dk_join3` at `/Od` emits the full source copy count (3) — i.e. the collapse is optimizer-gated wherever it lives | **0.80** |
| **N11** | **at the end of this lane `#3103` is CLOSED** — the residual collapse is attributed to a named address or to C-D, with a positive measurement and not an absence | **0.55** |
| **N12** | the merger set as a whole is **closed** — no *fifth* unattributed collapse survives this lane's fullest ablation | **0.35** |

`N11` and `N12` are the lane's deliverable and are deliberately registered
apart: this seam has relocated its blind spot at every level so far, and
**a fifth relocation is a legitimate result** that must be reportable without
having been quietly bet against.

## 3. Structural axes to cross (not values to vary)

`#3102`'s lesson — the `on`/`off` cell of a 2×2 is what catches a misconfigured
probe — applies. The axes, crossed; values vary **inside** cells:

1. **number of duplicate copies**: 2, 3, 4 arms (the count is the axis; which
   store is duplicated is the value)
2. **which arm holds the extra work** (first / middle / last)
3. **presence of a call** between the duplicated statement and the join
   (`#3069` makes a call an absolute scheduler barrier)
4. **favor-size vs favor-speed** (`DAT_10c2e310`; `/O1` vs `/O1 /Ot`)
5. **`/Og` on vs off** (`/O1` vs `/Od`) — the optimizer bit `DAT_10c2e2fc`
6. **common code at the head vs the tail** of the arms (hoist vs sink)

Known gates, inherited and not re-derived: `mode == 2`, `/Og`, **not
`/LTCG:PGI`** (`DAT_10c3de20 != 1`).

## 4. Method commitments

* **Ablation of patched copies**, `#3100`'s instrument. The pinned image is
  never modified and its sha256 is re-verified at the lane tip. Patched images
  are **controls, never oracles**: every behavioural claim is read off `A0`.
* **Every patch site's original bytes asserted** against the export's
  disassembly before writing, `w-dagclients`' `patch.py` convention.
* **Fixed `/Fo` and `/Fa` paths across every variant** — `#3100`'s trap: `cl`
  embeds the path strings in the obj at file offset `0x44a`, so per-variant
  filenames fake a delta.
* **`TimeDateStamp` (file offset 4..8) zeroed** before any hash compare.
* **Grid frozen by content hash before the first `cl.exe`**, in an R2 file —
  `w-keygen`'s rule that a hold-out frozen by *name* is not frozen.
* **Absences are labelled, never banked.** A `ud2` that does not trap is
  "not entered **on the cells tried**". Any conclusion resting on an absence
  goes in a grey-zone section with no board claim, per `#3101`.

## 5. What would make this lane FAILED

Stated now so it cannot be renegotiated later:

* 0 discriminating cells; or
* the residual collapse neither attributed to an address nor to C-D, **and**
  no positive measurement (entered / past-gates / fired) narrowing where it is;
  or
* a conclusion resting only on "the ablation changed nothing".

A lane that lands "the merger set is still not closed, and here is the fifth
relocation, positively measured" is **`built`**, not FAILED. A lane that lands
"we could not tell" is FAILED, in that word.
