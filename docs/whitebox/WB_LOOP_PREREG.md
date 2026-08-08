# WB-H `wb-loop` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing here is
> adopted into `crates/`.

Registered **before the first `cl.exe` of this lane** and before any
substantive read of `~/ghidra-projects/export/c2/`, per board #770's standing
rule (running streak ~10 optimistic / 2 pessimistic / 1 hit). Scored in
[`WB_LOOP_FINDINGS.md`](WB_LOOP_FINDINGS.md) §8.

Lane: `wb-loop` / branch `worktree-agent-adaf1b7962a181419`, branched at master
`cfd972c`. Image sha256 verified at the top of this lane:
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
(`~/ghidra-projects/bin/c2dll`, matches method doc §0).

**What was read before freezing.** In-repo only:
`CAMPAIGN_2026-08-08.md` (all), `CAMPAIGN_2026-08-08_GENERATORS.md` (§WB-H),
`C2_MAP_METHOD.md` §0–§4, `WB_REGALLOC_FINDINGS.md` §1, §4, §5, §6, §7.1–§7.5,
§9, §10, `grids/wb-regalloc/regorder_grid.cpp`, `WB_REGALLOC_PREREG.md` (for
format), `scripts/gt_capture.sh`, `scripts/gt_dump.py` header.

**One export grep was run before freezing and is disclosed here**, because it
was needed to know whether the lane's entry point exists at all: a
case-insensitive grep of `strings.tsv` for `lur.c|globlopt.c|cgintrin.c|
loopopt|unroll`. It returned exactly four rows and nothing else was read:

| VA | string | xrefs |
|---|---|---|
| `10b02210` | `…\be\p2\globlopt.c` | `10b45a5f,10b47885,10b48c3f,10b4a139,10b4aab6` |
| `10b13628` | `…\be\p2\lur.c` | `10b75e5b,10b766d3,10b77625,10b7acb4` |
| `10b1410c` | `-loopopt` | `10c29860` |
| `10b19698` | `…\be\p2\ppc\cgintrin.c` | 8 xrefs in `10bf0…10bf9…` |

No disassembly, no decompilation, no `functions.tsv` and no `xrefs.tsv` had
been opened when the predictions below were written.

**Zero `cl.exe` invocations of this lane's own authorship had been run.**
(`gt_capture.sh /dev/null` was run once as a liveness check of the toolchain
and printed the compiler banner; it compiled no cell and produced no obj.)

---

## P0 — the success floor

| # | prediction | direction if wrong |
|---|---|---|
| P0.1 | This lane clears its floor: the `mtctr`/`bdnz`-vs-compare-branch choice is stated as a predicate and **survives a frozen grid** (≥8 of the cells that bear on it). | optimistic |
| P0.2 | The choice is **decidable from the source shape alone** — no whole-program or profile input is needed to predict it. | optimistic |
| P0.3 | The *zero-trip guard*'s presence/absence turns out to be **easier** to predict than the `mtctr` choice itself. | — |

## P1 — where the loop lowering lives

| # | prediction | direction if wrong |
|---|---|---|
| P1.1 | `lur.c`'s functions occupy a roughly contiguous VA band, and it is the band its four assert-xrefs bracket: **`0x10b75000`–`0x10b7b000`**. | — |
| P1.2 | **The `mtctr`/`bdnz` conversion is NOT in `lur.c`.** `lur.c` is the machine-independent loop optimizer (unroll / rotate / IV strength-reduction); the count-register form is a **machine-dependent** decision and lives in a `p2\ppc\*.c` TU (`cgintrin.c`, `lowersmd.c`, or a sibling). | — |
| P1.3 | The `lwzu`/`stwu` **update-form** selection is likewise machine-dependent (`p2\ppc\…`), and is a *peephole over an already-strength-reduced pointer*, not something `lur.c` emits. | — |
| P1.4 | There is a **named, greppable** artifact for the ctr loop — a mnemonic string (`bdnz`/`mtctr`/`ctr`) or an option/diagnostic — reachable from the conversion site. | optimistic |
| P1.5 | `-loopopt` (`0x10b1410c`, one xref `0x10c29860`) is a **live** switch in this build (its variable has ≥1 reader), unlike `-schdat#` which WB-D found dead. | optimistic |

## P2 — WHEN the `mtctr`/`bdnz` form is chosen

Stated cold, as a predicate over the source loop.

| # | prediction | direction if wrong |
|---|---|---|
| P2.1 | The form requires the trip count to be a **loop-invariant integer expression evaluable in the preheader**. Non-constant is fine (WB-D's L1 has `n` a parameter). | — |
| P2.2 | The form requires the induction variable to be **dead after the loop** and used inside only in affine subscripts / the exit test — i.e. IV elimination must succeed. If the loop's `i` is used after the loop (returned, stored), the form is **NOT** taken and a compare-and-branch loop is emitted. | optimistic |
| P2.3 | A **`break`** (a second exit edge) **kills** the `mtctr` form: c2 falls back to compare-and-branch. | pessimistic |
| P2.4 | A **call in the body does NOT kill** the form — WB-D's L3 is reported as sharing the normal form. (Registered even though it is architecturally surprising: `ctr` is volatile in the PPC ABI, so a `bdnz` around a `bl` is only safe if c2 assumes callees preserve `ctr`. If the obj shows L3 is actually compare-and-branch, WB-D §9.3's "identical across three bodies" is **wrong** and this lane retracts it for them.) | — |
| P2.5 | A **small constant trip count** does not produce a `bdnz` loop at all — it is **fully unrolled** (or const-folded away). Predicted unroll ceiling at `/O1`: **≤ 4 iterations** fully unrolled; ≥ 8 keeps a loop. | optimistic |
| P2.6 | A **down-counting** loop (`for (i = n-1; i >= 0; --i)`) gets the **same** `mtctr` form, with the pointer walked downward (`lwzu rX,-4(ptr)`, preheader `addi ptr,base,+N`). | optimistic |
| P2.7 | A `while` loop with the same counted shape gets the **identical** form — the choice is on the normalized CFG, not on the C keyword. | — |
| P2.8 | Nested loops: the **inner** loop gets `ctr`; the outer loop gets compare-and-branch, because there is only one `ctr` and it is not saved/restored. | — |

## P3 — the zero-trip guard

| # | prediction | direction if wrong |
|---|---|---|
| P3.1 | The guard is `cmpwi cr6, n, 0` + a forward branch **over** the whole loop, and it tests **`n > 0`** (`bf 25` = branch-if-not-GT on `cr6`, bit 24+1). | — |
| P3.2 | The guard exists because the `mtctr`/`bdnz` form is a **do-while**: `bdnz` decrements then tests, so `ctr == 0` on entry executes 2^32 iterations. It is therefore emitted **whenever the trip count is not provably ≥ 1**. | — |
| P3.3 | The guard is **omitted** when the trip count is a compile-time constant ≥ 1. | — |
| P3.4 | The guard is **omitted** when the loop is a `do { } while` in the source (the C semantics already guarantee ≥1 trip). | optimistic |
| P3.5 | The guard is **NOT** omitted merely because the count is an `unsigned` parameter (unsigned `n` can still be 0), and its compare becomes `cmplwi`. | — |
| P3.6 | The guard's compare is **always `cr6`** (WB-D §7.5's retraction generalizes: `cr6` is the lowering constant for explicit integer compares). | — |

## P4 — the induction rewrite / update-form selection

| # | prediction | direction if wrong |
|---|---|---|
| P4.1 | The preheader `addi ptr, base, -N` has **N = the byte stride of the access** (elemsize × index stride), because PPC's `lwzu rD,d(rA)` updates `rA` **before** the access. | — |
| P4.2 | With **two arrays** in the body, **both** get their own strength-reduced pointer and their own update form (two `addi ...,-4`, two `lwzu`). Rival: only one array gets the update form and the rest are `lwzx`-indexed off a shared offset. | optimistic |
| P4.3 | With **four or more** arrays the update form is still used per array (no register-pressure-driven fallback at `/O1`). | optimistic |
| P4.4 | **Non-unit stride** (`a[2*i]`) keeps the update form with `d = 8`. | optimistic |
| P4.5 | Element type selects the mnemonic family: `char`→`lbzu`, `short`→`lhzu`, `int`→`lwzu`, `float`→`lfsu`, `double`→`lfdu`. Stores likewise `stbu`/`sthu`/`stwu`/`stfsu`/`stfdu`. | — |
| P4.6 | A **write-only** loop (`for(i…) a[i] = k;`) uses `stwu` with the same `addi ptr,-4` preheader. | — |
| P4.7 | When the same array is both read and written at the same index (`a[i] = a[i] + 1`), c2 uses **one** strength-reduced pointer with **one** `lwzu` and a **non-update** `stw rX,0(ptr)` — not two pointers. | optimistic |
| P4.8 | A **stride that is not a compile-time constant** (`a[i*k]`, `k` a parameter) defeats the update form; c2 falls back to an indexed `lwzx` with a `add`-updated offset, but **keeps** `mtctr`/`bdnz`. | — |

## P5 — the class boundary, in port terms

| # | prediction | direction if wrong |
|---|---|---|
| P5.1 | The boundary is statable as a **conjunctive predicate over ~6 clauses** (single back edge, single exit, loop-invariant trip count, affine constant-stride subscripts, IV dead on exit, body free of anything that forces a second exit). | optimistic |
| P5.2 | The predicate is **checkable on the port's own IR** without any of c2's numeric tables — i.e. a `loop_counted` class needs **no** DISCLOSURE row for its *predicate*, only (possibly) for a threshold. | optimistic |
| P5.3 | **Predicted reach over the 124-TU reach-pool: `0`.** Following WB-D §9.3 / P5.4 — a loop class is a capability, not a conversion, until the reader accepts the constructs. Registered pessimistically on purpose. | pessimistic |
| P5.4 | What a `loop_counted` class needs **beyond** WB-D's register rule is at least: (a) the pattern set for the body's operators, (b) the guard/preheader/latch **block order and the label numbering** they consume, (c) the update-form peephole. Registers are the cheapest of the four. | — |

## P6 — block order (WB-D's open question, §9.2 item 3)

| # | prediction | direction if wrong |
|---|---|---|
| P6.1 | For a guarded counted loop the emitted block order is **preheader → body → exit**, with the guard branching **forward** over the body. No block is emitted after the function's return. | — |
| P6.2 | An `if`/`else` **inside** a loop body emits the **then**-arm first (source order) — i.e. loops behave like WB-D's M1, not its M2. | — |
| P6.3 | Two sequential loops in one function emit in **source order**. | — |
| P6.4 | A loop containing a `continue` puts the continue-target (the latch) **at the end** of the body, not as a separate earlier block. | — |
| P6.5 | The **general** rule "block order = flow-graph construction order = source order except where a *selection* algorithm (switch tree, `factor.c` tail merge) built the blocks in another order" survives every loop cell in this grid. | optimistic |

## P7 — the grid itself (deliverable 3)

| # | prediction | direction if wrong |
|---|---|---|
| P7.1 | ≥ 20 cells are graded. | — |
| P7.2 | **A calibration pass is necessary and will move at least one cell** — as in wb-inline, whose v1 grid was refuted by its own cells because the compiler folded the ladder. Concretely: at least one constant-trip-count cell will not survive to be a loop. | optimistic |
| P7.3 | At least one *rival* is fully refuted by ≥3 cells. | — |
| P7.4 | At least one frozen prediction in P2–P4 is a **scratch** (obj shows something no rival named). | — |
| P7.5 | The lane ends with **≥1 retraction** of a prior campaign claim (its own or WB-D's). | — |

---

## Rivals for the `mtctr` choice, named before the run

| id | rival |
|---|---|
| **RC0** | **The predicted rule**: `mtctr`/`bdnz` iff (single back edge) ∧ (single exit) ∧ (trip count is a preheader-computable loop-invariant) ∧ (IV dead after the loop). |
| **RC1** | `mtctr` iff the loop is *innermost*, regardless of exits — i.e. exits are handled by an early `bctr`-less branch out and the form survives `break`. |
| **RC2** | `mtctr` only when the trip count is a **compile-time constant**. |
| **RC3** | `mtctr` always, for every `for`-shaped loop c2 recognizes, with the guard doing all the work. |
| **RC4** | No rule — the form is a peephole on the *already-emitted* decrement-and-compare pair, taken whenever the counter is otherwise dead (so it is decided after selection, not before). |

## Rivals for the update form

| id | rival |
|---|---|
| **RU0** | **The predicted rule**: one strength-reduced pointer per *distinct array base*, each with `addi -stride` in the preheader and an update-form access at the point of first use in the body. |
| **RU1** | Update form only for the **first** array; others `lwzx`-indexed off one shared index register. |
| **RU2** | Update form only when the loop has exactly **one** memory reference. |
| **RU3** | Never an update form — the shape WB-D saw is an artifact of a one-array loop and `lwz rX,0(ptr)` + `addi ptr,4` is the general case. |

## Minimum-separation assertion, to be checked BEFORE the run

The grid is only worth compiling if the rivals disagree on enough cells.
Asserted in advance, and re-asserted mechanically in §6 of the findings doc:

* **≥ 3 cells** must separate **RC0** from each of RC1, RC2, RC3 individually.
* **≥ 2 cells** must separate **RU0** from each of RU1, RU2, RU3.
* **≥ 1 cell** must exist where RC0 predicts *no* `mtctr` (otherwise RC3 is
  unfalsifiable by this grid and the floor is not cleared).
* Every cell's per-rival prediction is written down **before** the first
  `cl.exe`, in `grids/wb-loop/frozen.tsv`, with a sha256 recorded in the
  findings doc.
