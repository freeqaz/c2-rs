# WB_CANDID — PREREG for read **R1**: the scope of `DAT_10c400d4`

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md)
> and [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md). Every address here is an absolute
> VA in the pinned image
> `compilers/X360/16.00.11886.00/c2.dll`,
> sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> — **verified by this lane before any address was read** (`sha256sum` against
> both the repo copy and `~/ghidra-projects/bin/c2dll`; the two agree and both
> match `C2_MAP_METHOD.md` §0).

Lane `w-read-r1`, 2026-08-22. Funded by
[`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) decision 1
("option 4" — R1→R2→R3 before the decision-0 branch choice). Spec:
[`READ_PLAN_2026-08-21.md`](READ_PLAN_2026-08-21.md) §3 row R1, §4, §5.2.

Findings land in [`WB_CANDID_FINDINGS.md`](WB_CANDID_FINDINGS.md).

---

## 1. The question, and why it is load-bearing

`DAT_10c400d4` is the counter that stamps `cand+0x1c`, the id of a
register-allocation candidate. That id is the **hash key** (`id & 0x3ff` into
the 1024-bucket table `DAT_10c43b80`), and `0x10b316b1` builds the allocator's
priority worklist by walking that table **bucket by bucket**. So the id
ordering is the tie tier of the priority comparator `0x10b2b82d`.

The repo asserts both answers for the same counter:

| claim | site | says |
|---|---|---|
| **function-scoped** | [`WB_LIVE_FINDINGS.md:258-260`](WB_LIVE_FINDINGS.md) | ids are *"dense and function-scoped"*, on the evidence that the **hash table clear** `0x10b2c1f1` is per-function |
| **compilation-global** | [`ref/P_REGALLOC.md:62,86,188`](ref/P_REGALLOC.md) | *"compilation-global monotonic"*, three times, including the field table |

`READ_PLAN_2026-08-21.md:229-238` (§5.2) already registered — **committed to
git 2026-08-21, before this lane existed** — that the two are *not* trivially
reconcilable ("a global counter with a per-function hash clear makes ids
sparse-within-function, contradicting *dense*"), and what turns on each side.
**That is the pre-registration of the hypothesis pair and its consequences, at
full PREREG tier**, and this lane claims no credit for restating it.

## 2. Registration tiers — stated before anything else, because they are unequal

`PREREG.md` defines three tiers and forbids pooling them. This document
contains all three and labels each item:

| item | tier | why |
|---|---|---|
| the hypothesis pair + what each side implies (§3 below) | **PREREG** | committed in `READ_PLAN_2026-08-21.md` §5.2 on 2026-08-21, before dispatch |
| the **read protocol** in §4 | **POST-HOC** | ⚠️ **this lane ran the xref enumeration before writing this file.** The protocol is written down after the fact and **earns no evidential weight as a prediction.** It is recorded so a future reader can re-run it, not to claim ordering. |
| the **refutation conditions** in §5 | **POST-HOC** | same reason |
| the **behavioural control C1** in §6 | **PREREG** | ✅ committed here **before the control is compiled or run**, with its prediction, its red condition and its positive control all fixed in this commit |

**The honest summary of the ordering**: the read came first, this file second,
the control third. Only C1 is a prediction. Recording that plainly is cheaper
than an inflated hit rate, which is the failure `PREREG.md` exists to prevent.

## 3. The two hypotheses and what each decides — PREREG (from `READ_PLAN` §5.2)

**H-FN (function-scoped).** The counter is reset at each function's
back-end run. Ids are dense from a fixed base within a function.

* `WB_LIVE_FINDINGS.md:258-260` is right; `P_REGALLOC.md:62,86,188` is wrong
  and must be corrected in three places.
* **`P_REGALLOC.md:160-166` consequence 3 loses its force.** Its claim is that
  the tie order is a bucket walk over a *compilation-global* counter and is
  therefore "not a source property at all" — *"the most direct available
  explanation for why source-level fitted sorts keep being refuted"*. Under
  H-FN the walk is over a *per-function dense* counter, which is a **much
  weaker** disclaimer: a per-function dense id sequence is exactly the kind of
  thing that *can* correlate with source order, so it stops being an
  explanation for the refutations by itself and hands the question to the
  **mint order** — `FUN_10b55732`, read **R4**, unread.
* **The ten refuted allocation keys** at `crates/c2-core/src/codegen/alloc.rs:103-539`
  are then back to **unexplained** by this mechanism, which is the outcome
  `READ_PLAN` §3 row R1 named as the thing R1 decides.
* Board **#3242** and **#3056** must be reconciled: #3242 says "compilation-global
  counter", #3056 says "function-scoped".
* `select_function`'s no-TU-context signature is **sound** on this axis.

**H-GLOBAL (compilation-global).** The counter is written once per
compilation. Ids of a function's candidates depend on how many candidates every
preceding function minted.

* `P_REGALLOC.md` is right; `WB_LIVE_FINDINGS.md:258-260`'s "dense" is wrong.
* Consequence 3 stands, and the ten refuted keys have a standing explanation.
* `select_function`'s no-TU-context signature is **unsound** on this axis, and
  per-function composition acquires a mechanism it currently lacks.

**Neither hypothesis is the incumbent.** Both are in the tree, in equal voice,
which is the whole reason R1 was funded.

## 4. The read protocol — POST-HOC, recorded for reproducibility

1. Verify the image sha256 against `C2_MAP_METHOD.md` §0. *(Done first; both
   copies match.)*
2. **Enumerate every reference**, not the three cited sites — the brief's own
   instruction. `awk -F'\t' '$2=="10c400d4"' xrefs.tsv` over the flat export,
   cross-checked against `grep '10c400d4' objdump_intel.asm` so that no claim
   rests on Ghidra alone (`C2_MAP_METHOD.md` §1's independent-disassembly rule).
3. Classify each reference READ / WRITE / READ_WRITE and disassemble its
   containing basic block.
4. For the **write**, walk the call graph *upward* to a driver whose iteration
   granularity is observable, recording at each hop whether the call is
   unconditional and what guards it.
5. For the **increment**, enumerate every call site of the minting routine and
   check each is dynamically inside the write's driver — i.e. that no mint can
   occur without a preceding reset.
6. Do the same for the three *sibling* globals the write touches in the same
   basic block (the hash table `0x10c43b80`, the free list `0x10c2e3e0`, the
   per-class sets `0x10c400d8`), because a scope boundary that resets some but
   not all of them is a different answer.

## 5. What would refute the reading — POST-HOC

The read is **falsified**, not merely weakened, by any one of:

* a fifth reference to `0x10c400d4` that the export missed (checked two ways);
* a **second** writer, or a writer reachable at a different granularity than
  the one claimed;
* a call site of the mint `0x10b54d32` that is **not** dominated by the write —
  a mint without a preceding reset means ids carry across the boundary;
* the driver holding the write turning out to iterate something other than
  functions;
* a path that resets the hash table **without** the counter, or the counter
  **without** the hash table, in a way that makes ids sparse within a function.

The last one is exactly `READ_PLAN` §5.2's reconciliation trap and is checked
explicitly rather than assumed away.

## 6. Control **C1** — the behavioural confirmation. PREREG: written and committed before it is run

`READ_PLAN` §5.3 is the reason this section exists: `[R]` means *"the
instructions were read correctly"*, not *"this is what c2 does"*, and the
`.bss` bump rule was read correctly out of a clean function and was **wrong
about c2**. The obj is the sole judge.

### 6.1 What C1 does

Compile two TUs with the real `cl.exe` under wibo at the workload's `/O1 /GS- /c`
(`scripts/gt_capture.sh`):

* **`solo.cpp`** — a tie-sensitive probe function `P` alone.
* **`after.cpp`** — `N` filler functions that mint candidates, then a
  **character-identical** copy of `P` last.

Extract `P`'s emitted bytes from each obj and compare.

### 6.2 The prediction, fixed now

* **Under H-FN**: `P`'s bytes are **identical** in `solo.obj` and `after.obj`
  for every `N`, because the counter, the hash table and the free list are all
  reset before `P` is allocated — `P` cannot see any predecessor.
* **Under H-GLOBAL**: identical for *small* `N` too — because shifting a short
  dense id run `1..n` by a constant `D` preserves its ascending bucket order —
  but once the cumulative candidate count carries `P`'s ids **across the 1024
  boundary**, the bucket walk `0…1023` splits `P`'s candidates into two runs
  and the tie tier permutes. So C1 is only discriminating at large `N`, and it
  is run with `N` scaled deliberately past that point.

### 6.3 Red condition

**C1 is RED if `P`'s bytes differ between `solo.obj` and `after.obj` at any
`N`.** A red refutes the static read outright and is publishable as such.

### 6.4 The control's own control — mandatory, and it is why C1 is not vacuous

A byte comparison that returns "identical" is worthless if the extractor
returns a constant. So C1 ships with a **positive control C1-pos**: the same
extractor run on a `P` whose body is perturbed by one statement **must report
DIFFERENT**. If C1-pos does not go red, C1's green is discarded and reported as
an instrument failure, not as evidence. (`STATUS.md`'s standing trap: `mismatch
0` is not evidence of correctness; absence must never read as success.)

### 6.5 The limit of a green, stated before the result is known

**A green C1 corroborates H-FN; it does not prove it.** Identical bytes are
also consistent with "`P` had no ties to break", or with a shift that happened
not to permute anything. This is precisely the ambiguity board **#3363** left
behind — its 4 probe shapes × 8 TU contexts came back 32/32 byte-identical and
the row read that as *"`DAT_10c400d4`'s tie tier does not reach the emitted
bytes on these shapes"*, which is agnostic between H-FN and H-GLOBAL. C1
narrows that only if `N` is large enough to wrap 1024, and even then a green is
corroboration. **The load-bearing evidence for R1 is the static reference
closure in §4, and C1 is the check that could have overturned it.**

## 7. Scope of this lane — declared up front

Characterization lane (`docs/rungs/README.md` § "Lane kinds", kind 3).
**`Fixtures: none`. `Census: +0`. Predicted reach 0. No `crates/` change.**
Board rows **#3372–#3375** only. Deliverable is an address-cited answer plus
the reconciliation of whichever standing doc the read contradicts, and — if a
disassembly-derived constant is adopted into `crates/` by a later lane — a
`DISCLOSURE.md` row. This lane adopts nothing, so it adds a `DISCLOSURE.md`
row for the **finding**, not for an adoption.
