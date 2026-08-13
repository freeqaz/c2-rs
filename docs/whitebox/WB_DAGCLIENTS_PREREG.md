# WB_DAGCLIENTS — PREREG (R1, frozen before the first grep of the export)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address named here is an absolute
> VA in the image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0,
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> — **verified on `compilers/X360/16.00.11886.00/c2.dll` at the top of this
> lane** before any address was read. Navigation only; this lane adopts nothing
> into `crates/` and owes no `DISCLOSURE.md` row.

**Frozen state at the moment of writing:** the only c2.dll bytes this lane has
looked at are the sha256 above. The four subject addresses and the two parent
addresses below are quoted **from board `#3071` / `WB_DAGORDER_FINDINGS.md` §8
item 4b**, not from any grep of `~/ghidra-projects/export/c2/` performed by
this lane. The first such grep happens **after** this file is committed.

---

## 0. The question (board `#3071`, `wb-dagorder`'s own stated blind spot)

`FUN_10b328da` @ `0x10b328da` — the dependence-DAG builder — has five callers.
One is the list-scheduler driver `0x10be6382`, which is constrained to
`FUN_10be5d4b` @ `0x10be5d4b`'s regions (≤ `0x50` tuples, ended by
branch / call / label / body-end). The other four are **unread**:

| # | client | parentage per `#3071` |
|---|---|---|
| K1 | `0x10b3b167` | under `0x10b3c2cc` / `0x10b3c065` |
| K2 | `0x10b3b41b` | under `0x10b3c2cc` / `0x10b3c065` |
| K3 | `0x10b3b5fd` | under `0x10b3c2cc` / `0x10b3c065` |
| K4 | `0x10c1ce93` | under the `/QXSTALLS` listing writer `0x10b71d8f`, which also calls the graph teardown `0x10b32536` |

**Do any of the four reorder tuples, and under what conditions?** If one does,
`wb-dagorder`'s ordering model has a second author and is incomplete; and by
`#3071`'s own admission **no cell of that lane's 15-cell grid could detect it**,
because every cell observes only the final tuple order on straight-line integer
code where a second author would plausibly agree.

## 1. The positive question, in writing — what goes red

> **If one of K1–K4 DID reorder tuples, which cell of mine changes?**

A cell of this lane is red iff **both**:

* **(a) run-proof** — a positive, on-content witness that the client actually
  executed in that configuration (a listing artifact only that client writes, a
  diagnostic only it emits, or a code path whose gate I can show is satisfied
  by the exact `cl.exe` command line used); **and**
* **(b) order-delta** — the emitted `.text` tuple order differs between the
  configuration where the client runs and the one where it does not, with
  the COFF `TimeDateStamp` (offset 4..8) zeroed, on a body whose *scheduler*
  configuration is otherwise identical.

**A cell that cannot supply (a) is not a control.** "I toggled a flag and the
obj did not change" is the exact absence-read-as-success shape that produced
`#1823` and that `#3067` caught inside a published conclusion. Therefore the
grid is invalid unless at least one cell demonstrates (a) *in the affirmative*
— i.e. I must show the client ran at least once, in at least one cell, from
content the client itself produced or from a gate I can positively evaluate.

**Escape hatch, declared in advance:** if a client turns out to be
**unreachable from any `cl.exe` command line** (its gate has no writer, or the
only writer is a code path cl never takes), then requirement (a) cannot be met
and no black-box cell can go red. In that case the lane's claim shrinks to a
**read** claim, and it must be positive on content: the *gate variable's
complete writer set* named by address, and the tuple-mutation question answered
by reading the client's body for writes to the tuple links (`tuple+0` next /
`tuple+0x10` prev, per `WB_DAGORDER_FINDINGS.md` §2's emission row) and for
calls to the relinker `0x10be626c`. Reporting "I found no relink call" is
**not** admissible on its own; the claim must name what the client *does* write.

## 2. Structural axes (crossed first; values vary inside cells)

| axis | levels | why structural |
|---|---|---|
| **S-CLIENT** | K1 / K2 / K3 (the `0x10b3c2cc` family) vs K4 (`/QXSTALLS`) | different parents, different gates; a result for one says nothing about the other |
| **S-OPT** | `/Od` vs `/Og`-class (`/O1`, `/O2`, `/Ox`) | the scheduler is gated on `DAT_10c2e2fc` bit 21 (`0x10b82429`); at `/Od` none of the four scheduler passes run, so `/Od` isolates any client whose gate is *not* that bit |
| **S-FLAG** | the client's own gate off / on (`/QXSTALLS` for K4; TBD for K1–K3) | the only axis on which (b) can be measured without confounding by S-OPT |
| **S-REGION** | body with no region ender / with a branch / with a call / with a label | K1–K4 bypass `0x10be5d4b`, so they can see barriers *inside* a range; if any reorders, a body containing an interior barrier is where it must differ from the scheduler |
| **S-SIZE** | < `0x50` tuples vs > `0x50` tuples | straddles the region cap: the scheduler's unit changes at the cap, a non-region client's does not |

Values that vary *inside* cells (never as a substitute for an axis): number of
statements, symbol count, int vs pointer, `/Gy`, `/EHsc`.

## 3. Registered claims (probability form; scored in the findings doc)

| id | claim | p |
|---|---|---|
| **M1** | **At least one of K1–K4 re-threads the tuple list** (writes `tuple+0`/`tuple+0x10`, or calls the relinker `0x10be626c` / an equivalent splice) — i.e. tuple order has a second author | **0.20** |
| **M2** | K4 (`0x10c1ce93`, under the `/QXSTALLS` listing writer) is **read-only w.r.t. tuple order** — it builds the DAG to *report* stalls, and tears it down (`0x10b32536`) without mutating the list | 0.85 |
| **M3** | K1–K3 share **one** gating condition, and it is **not** `DAT_10c2e2fc` bit 21 alone (i.e. `/Og` is necessary but not sufficient to reach them) | 0.55 |
| **M4** | At `/Od`, **none** of K1–K4 runs | 0.65 |
| **M5** | K1–K3's gate is reachable from a `cl.exe` command line I can issue (a documented switch, a `/d2`/`-d2` backend switch, or an implied one) — i.e. requirement (a) is satisfiable black-box for that family | 0.50 |
| **M6** | K1–K3 do **not** call `0x10be5d4b` — `#3071`'s "bypass the region finder" is confirmed by reading their bodies, not inherited | 0.80 |
| **M7** | At least one of K1–K3 is a **loop / cross-block** analysis (its DAG unit is a loop body or a whole function rather than a straight-line range) | 0.40 |
| **M8** | `/QXSTALLS` changes the emitted obj (TimeDateStamp-zeroed) on at least one grid cell | 0.30 |
| **M9** | The detecting cell defined in §1 exists **and stays green** — i.e. the honest answer is "none of the four reorders", established positively rather than by absence | 0.55 |
| **M10** | At least one of K1–K4 **shares the DAG that the scheduler already built** (is invoked on an existing graph rather than building its own from tuples) | 0.25 |

**M1 and M9 are the lane's deliverable and are mutually exclusive in outcome
but not in evidence**: M1 true ⇒ the second author must be *named and its
condition stated*; M1 false ⇒ M9 must be *demonstrated*, not asserted.

## 4. Falsifiers / abort conditions, declared now

* **The lane FAILS** if it lands neither (i) a named second author with the
  condition under which it reorders, nor (ii) a cell satisfying §1(a)+(b) that
  could have gone red and did not, nor (iii) a positive-on-content read
  establishing unreachability per §1's escape hatch. "I read the four functions
  and saw no reordering" **alone is a FAILED lane**, not a result.
* **Grey-zone rule:** any claim I cannot discriminate is filed grey-zone with
  no board `DISCLOSURE` row and no banked probability score, per `wb-live`'s
  cost-array precedent. A grey-zone claim is not a hit and not a miss.
* **Probe validation before reading any result:** every `cl.exe` cell has its
  structure verified (PROC count, listing shape, the presence of the construct
  the cell exists to test) before any byte comparison is read off it. Two
  ladders in this repo failed to test the construct they were named for.
* If a second prereg (the obj grid) is needed, it is frozen **by content
  hash** before the first `cl.exe`, per `w-keygen`'s lesson that a hold-out
  frozen by name is not frozen.

## 5. Scope

Docs-only. `git diff master..HEAD -- crates fixtures scripts` must be empty at
the lane tip. The 878-TU scan must be digit-identical to base
(`match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8`).
