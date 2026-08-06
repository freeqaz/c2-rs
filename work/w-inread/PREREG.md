# w-inread — PREREGISTRATION

    Lane:    w-inread, 2026-08-08
    Branch:  wt-w-inread, off master `e928ee7`
    Board:   #960 / #961 (rows to be filed under #976–#985)
    Scope:   A READER WIDENING that SHIPS under `crates/c2-il`, plus the grammar
             cells that license it and the metric #961 asks for.
    Written: BEFORE any probe exists on this branch.  No script in
             `work/w-inread/` has been run; no cell has been authored, frozen or
             compiled; no corpus-wide number has been computed here.  Everything
             cited below is quoted from a LANDED rung
             (`docs/rungs/2026-08-08-w-emitp2.md`,
             `docs/rungs/2026-08-07-w-tag02.md`,
             `work/w-emitp2/blindspot.txt`, `work/w-emitp2/two_readers.txt`,
             `work/w-tag02/GRAMMAR.md`) or read out of
             `crates/c2-il/src/func/ininit.rs` as it stands at `e928ee7`.

---

## 0. The question

`crates/c2-il`'s `.in` reader — the one an actual Phase-7 model would have to run
on — reads **1,429,596 of 1,885,700** tag-02 symbol addresses (75.81 %) and
**518,098 of 879,377** records (58.92 %) over the **850** TUs of
`work/w-db/cacheidx.tsv`; fewer on **812** TUs, equal on 38, **more on none**.
w-emitp2 decomposed the blind spot into four element kinds and priced closing it
at **+19 per-TU exact** to `JFP_ALIAS` (289 → 308) and **+213** to the
`ALIAS_IN` ceiling (259 → 472).

This lane widens that reader, on measured grammar, and reproduces the price
**forwards**.

**Two of the four element kinds have never been measured on any cell.**
w-tag02's 24-cell grid produced neither scalar type `03` nor scalar type `04`.
That makes this a grammar rung and not a widening, and it is why cells come
before code.

---

## 1. The registered claims

Every point below is stated so that it can LOSE, with the interval written
before the probe.  Scored in the rung doc's §"prereg scorecard".

### A. Element tag `08` — the zero fill (315,553 symbol addresses over 812 TUs)

| # | claim | interval / refuter |
|---|---|---|
| **P1** | The element is exactly **`08 <count>`** — one tag byte and one varint field, nothing more. | Exact.  **Low information and registered as such**: `work/w-emitp2/strictin.py::node` already frames it as `08 <i32c>` and its sequential parse is clean on **850 of 850** streams, so a different field count would desynchronize 850 streams.  A loss here would mean the sequential parser has been wrong all along. |
| **P2** | `<count>` is a **BYTE count**, and the element contributes exactly `<count>` **zero bytes** to the initialized object's raw bytes at the running offset. | The three named alternatives that would refute it: **(a)** it is an *element* count / a repeat of the preceding element; **(b)** it is a *bit* count; **(c)** it is the object's total size rather than the length of the fill run.  Judge: real `c2`'s obj — the section's `SizeOfRawData` and its bytes, read out of the obj, not out of a listing (#843). |
| **P3** | A tag-`08` element carries **no relocation**. | Exactly **0** relocations attributable to a tag-08 element in any cell. |
| **P4** | The minting construct is a **partially-initialized aggregate**: `struct S{int a; int b; int c;} s = {1};` spells `01 01 04 01` then a tag-`08` run of **8**, rather than three scalar elements. | Ranked alternative, which I think is live: c2 spells the trailing zeros as explicit `01 01 04 00` elements and tag `08` appears only above some size threshold.  **This is the informative cell of the tag-08 set** and I do not know which way it goes. |
| **P5** | Widening `read_elements` **alone**, without adding a `00 08` anchor, recovers **≥ 300,000** of the 315,553 symbol addresses tag 08 is blamed for. | `[280000, 315553]`.  Basis: `blindspot.txt`'s *"THE LOST RECORDS BY THEIR FIRST ELEMENT"* reads `first=08` at **2** records over 850 TUs, so all but 2 of the 114,865 tag-08-blamed records are entered through an `01` or `02` first element and are already anchored. |
| **P6** | The `<count>` field's short form is restricted to `00..7F` with a `80` + LE32 escape, the same shape `read_offset` already spells, and a high-bit short form is a **desync** that refuses. | Refuted by any cell whose fill run spells a high-bit byte with no escape.  Cells will drive the run length past 127 deliberately. |

### B. Scalar type `03` at width 4 (132,488 symbol addresses over 812 TUs)

| # | claim | interval / refuter |
|---|---|---|
| **P7** | Type `03` is a **pointer-typed scalar** — a pointer whose value is a plain integer and which needs **no relocation** (canonically a null pointer *inside an aggregate*). | **The evidence that makes this predictable rather than a guess is already published**: `work/w-tag02/GRAMMAR.md` spells `t11_vfptr`'s `??_R0` record as `02 13 0a 00 04 · 01 03 04 00 · 03 08 ".?AUA@@\0"`, and `??_R0` is `struct TypeDescriptor { void* pVFTable; void* spare; char name[]; }` — so the `01 03 04 00` slot is **`spare`, a null `void*`**.  Refuted if a cell with a null pointer member spells anything other than `01 03 04 <0>`. |
| **P8** | Type `03`'s value uses the **same encoding** as types `01`/`02` at the same width — short form `< 0x80`, else `80` + LE(width) — and contributes `width` **big-endian** bytes to the obj. | Refuted by a cell with a non-zero type-03 value that spells otherwise.  A cell that can only ever produce `0` leaves this **UNSEPARATED**, and in that case the reader admits type 03 **only at value 0** and refuses the rest, rather than assuming. |
| **P9** | In the workload, **≥ 90 %** of the 132,528 type-03 elements carry the value `0`. | `[0.75, 1.00]`.  An observational claim about the corpus, graded by a corpus scan and explicitly **not** by a cell. |

### C. Scalar type `04` at width 4 (9,675 symbol addresses over 806 TUs)

| # | claim | interval / refuter |
|---|---|---|
| **P10** | **I do not know what type `04` is.**  Ranked guesses, registered so the miss is visible: (1) a **function** pointer, as distinct from `03`'s data pointer; (2) a pointer-to-member / `__based` pointer; (3) an enum with a fixed underlying type. | Whichever cell mints it wins; if none does, P10 scores a **MISS in full** and the decline floor in §2.1 fires. |
| **P11** | Type `04` at width 4 uses the same value encoding as P8. | Same refuter. |
| **P12** | Type `04` is ~**6 elements per TU** (4,844 over 806 TUs) and is therefore a *rarer, structural* slot rather than a common C++ spelling. | Descriptive; scored against whatever the minting cell turns out to be. |

### D. The uniform ±2 (w-emitp2 §4.1)

| # | claim | interval / refuter |
|---|---|---|
| **P13** | **The commissioning brief has the direction inverted.**  It reads *"the crate is uniformly short by exactly 2 records on 806 of 850 TUs"*.  w-emitp2 §4.1 measures the **transcript** short by exactly **2 symbol addresses** on 806 of 850 TUs — `T = 1,427,984`, `B = 1,429,596`, `Σ\|Δ\| = 1,612 = 806 × 2` — i.e. **the crate sees TWO MORE**, and the units are symbol addresses, not records. | Registered as a correction that can itself lose: if a fresh reconciliation puts the direction the other way, I say so. |
| **P14** | The two extra are **real** — records the crate frames and the sequential parser does not — and **not** phantoms produced by re-anchoring inside a refused record's tail. | `[1, 2]` of the 2 genuine.  **This is the claim I expect to lose**: w-emitp2 §4.1 attributes the gap to resync, which points the other way, and I am registering the opposite because it is the reading that would make the crate's number the better one. |
| **P15** | Whatever the cause, it is **uniform per TU** — max per-TU \|Δ\| stays 2 after the widening. | `[0, 4]`. |

### E. Forward reproduction — the acceptance test

| # | claim | interval |
|---|---|---|
| **P16** | After the widening, the crate's cursor reads **≥ 1,860,000** of the channel's 1,885,700 tag-02 symbol addresses on the same 850 TUs (≥ 98.6 %). | `[1800000, 1890000]` |
| **P17** | `JFP_ALIAS` per-TU exact on the widened crate reader = **308** of 850. | `[300, 308]` |
| **P18** | `ALIAS_IN` per-TU exact on the widened crate reader = **472** of 850. | `[440, 472]` |
| **P19** | `INIT` recall on the widened crate reader = **0.95991**. | `[0.94, 0.96]` |
| **P20** | TUs where the crate sees FEWER symbol addresses than the channel: **≤ 60** of 850 (from 812). | `[0, 200]` |

### F. #961 — the denominator

| # | claim | interval |
|---|---|---|
| **P21** | An `unanchored` population exists and the crate can count it **without a sequential parser**: the scan positions it skips, and the `00 02` candidates the fail-closed arm drops.  Published as printed scan metrics beside `records`. | Ships or does not. |
| **P22** | On the **878**-TU workload scan the newly-published unanchored population is **non-zero on ≥ 800 TUs**. | `[700, 878]` |
| **P23** | The totality identity `records == accepted + residue` stays **closed on all 878** — `in-init-accounting-broken` **0** — with the new counter published beside it and **not folded into it**. | Exact: 0. |

### G. The alarm — a pure reader widening should move NO emit-side number

| # | claim | interval |
|---|---|---|
| **P24** | FBM partition after: **exact 35,982 · whole-TU 2 · differs 3,195 · partial 0 · refused 130,573 · unbound 9,225** of 178,975; `fnbyte-match-tu-differs` **0**; `partition-broken` **0**. | Every digit exact. |
| **P25** | 878-TU scan: **match 10 · mismatch 0 · codegen-gap 0 · vocab-gap 861 · capture-fail 7 · port-error 0**. | Every digit exact. |
| **P26** | Factors **A 28 · B 338 · C 169 · D 10 · E 2**, `A∧B∧C` **27**. | Every digit exact. |
| **P27** | `scripts/gate.sh --jobs 6` **18/18 PASS**, 0 mismatch; `cargo test --workspace --release` **≥ 961 passed / ≥ 30 targets / 0 failed**. | Exact on the floor. |
| **P28** | `data-only-tu` accepts **0 of the 871 graded TUs** before AND after — the same measurement w-tag02 §6 published, which is *why* P24–P26 can hold through a widening that grows the accepted record population by ~70 %. | Exact: 0 / 0. |

---

## 2. Decline floors — registered in advance, so declining is not a retreat

1. **Anything not measured on a frozen cell REFUSES.**  Specifically: a scalar
   type outside the set the cells mint; a width outside `{1, 2, 4}` for any
   newly-admitted type; a tag-08 `<count>` with a high-bit short form; a tag-08
   `<count>` that would run the object past the crate's existing `1 << 16`
   bound.  Every refusal ships as a **named** `InInitResidue` variant with a
   workload count, never as a silent skip.

   1.1 **If no cell mints scalar type `04`**, the reader keeps refusing it and
   the lane publishes **9,675 symbol addresses** as residue, with the cells that
   failed to mint it listed by name.  A guessed width-4 integer would put wrong
   bytes in a `.data` and is the exact shape of board **#232**.

2. **The writer refuses what it cannot place.**  If widening `read_elements`
   hands `data_tu` an object whose bytes cannot be checked against real `c2`,
   `emit_data_obj` refuses rather than emitting.  A widening that turns an honest
   refusal into a wrong obj is a regression even when every count goes up
   (#232, w-tag02 §5).

3. **A shortfall is decomposed, not averaged.**  If the forward reproduction of
   +19 / +213 falls short, `work/w-emitp2/blindspot.py` is re-run **unmodified**
   on the widened reader and the residual is published by cause with counts.

4. **Nothing is priced by class removal** (`docs/STATUS.md` trap 8).

5. **Two independent instruments must agree before a reading ships**, and
   neither may be the other's witness (w-divsplit / w-tag02 §3).  The
   crate-free parser and the crate's own cursor are reconciled **count by
   count**, and every disagreement is printed with a count.

6. **Per-TU exact is reported BY NAME for any set that moves** (#250) — a count
   is not a set.

7. **Any per-record binding keys on `FnCensus::emit_name`** (#918), never on a
   positional name.

8. **The ±2 gets a verdict, not a shrug.**  If it is a defect in the crate it is
   fixed and the fix is graded; if it is a defect in the transcript the
   transcript is corrected and said so; if it is neither, the mechanism is
   named.

9. **A null is reported as a null.**  If the widening moves the emit-side
   metrics by zero — which P24–P26 predict — that is the headline, stated first.

10. **`work/` artifacts are scrubbed of absolute machine paths before commit**
    and no IL, obj or `_CL_*` artifact is committed.

---

## 3. What "the judge" is here

The real `c2.dll` under wibo plus a byte-exact obj compare with the COFF
`TimeDateStamp` zeroed, at the **workload's own** flags
(`work/w-tag02/flags_probe.txt` — `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi
/EHsc`).  Cells are `sha256`'d into a committed manifest **before** they are
compiled.  No listing decides anything (#843).  A corpus scan is a **driver**
that tells me which C++ to write; it is never evidence about the grammar.
