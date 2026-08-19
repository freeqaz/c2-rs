# PREREG — lane `w-c2map3`, structural breadth over `c2.dll`

**Frozen 2026-08-19 at base `e82c9ede6`, before the first probe.** Amendments go
in a dated box below; nothing above a box is rewritten.

Image: `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, size
1 347 072. **Verified by this lane at base** (`sha256sum`, 2026-08-19).

Kind: **characterization lane**. `Fixtures: none`. `Census: +0`. Predicted
reach **0** on every gap metric, on `match`, and on `fnbyte-exact`. This lane
lands documentation and generated indices; it adopts nothing into `crates/`
and adds no `DISCLOSURE.md` row unless a constant is copied, in which case the
row lands in the same commit.

---

## 0. What was already done before this prereg was frozen, and why it is not a probe

Written down because the ordering matters and hiding it would be the defect:

* `sha256sum` of the image (a precondition check, not a measurement of c2);
* `wc -l` / `head` over `docs/whitebox/**` and `$C2RS_GHIDRA_EXPORT/*.tsv`
  (measurements of **the record and the export**, not of `c2`);
* `grep -ril` over `docs/whitebox/` for the strings `weak.?extern`, `comdat.?synth`,
  `cflow-` (a measurement of **what the record says**, which is exactly what
  H2 below predicts about — so **H2's prediction is registered as
  already-informed** and carries no credit for being right).

**No byte of `c2.dll`, no obj, and no line of `decomp_all.c` had been read
when this was frozen.** Everything under §2–§4 is downstream of that line.

---

## 1. The question

> The existing whitebox corpus is **18 deep one-off BEHAVIOUR campaigns over a
> quarter of the binary**. What Option A needs is **structural breadth** —
> subsystem → function → what it decides — over the other three quarters, so
> that a future lane arriving with a question finds the answer, or the exact
> address to go read, instead of paying for a black-box probe grid.

The repo's own price for the alternative: *a single alignment nibble cost a
lane; `dag.c`'s lowering order took two.*

---

## 2. Hypotheses, each with the measurement that would refute it

| # | hypothesis | refuted if |
|---|---|---|
| **H1** | `ADDR.tsv`'s coverage is **bounded by prose**, not by the binary: it has a row only for an address someone already wrote about, so an arriving lane holding an *uncited* address gets nothing at all — not even "this is in `globopt.c`'s gap, 3 callers, references string X". | a row exists in `ADDR.tsv` for an address that is neither cited in `docs/` nor hand-labelled. (Read from `build_ref.py`'s own `addrs = set(cites) \| set(labels)`; the check is the generated file, not the source.) |
| **H2** | Of `CEILING.md` §6.1's seven phases, **≥ 3 have no whitebox reference page and no findings document**. The coordinator's guess is phases **5** (weak externals), **6** (COMDAT synthesis) and **1** (emitter CFG classes). | fewer than 3 are unserved, **or** the unserved set differs from {1,5,6}. *(Registered as already-informed — see §0.)* |
| **H3** | The **whole-image denominator is reachable mechanically**: every one of the 4,919 Ghidra functions can be given a TU attribution, a degree, a size, and at least one *navigational hook* (a referenced string literal, a diagnostic number, an import, or a bounded call-graph distance to an already-labelled function) from data **already in the repo plus the flat export**, with no new disassembly reading. | any of the four columns cannot be filled for ≥ 5 % of the 4,919 by mechanism alone. |
| **H4** | **Navigational hooks are not uniform**: the fraction of functions with a *strong* hook (a referenced string literal or an import) is well under half, so a mechanical index is a **triage instrument**, not a substitute for reading. | ≥ 50 % of the 4,919 carry a referenced string or import. |
| **H5** (the trap, from `W-GLATTRS-1`) | Disassembly's role in this repo is **hypothesis generation**, and the black-box grid is what carries a decode. Concretely: of the 4 rules in `DISCLOSURE.md`, **the majority have a non-whitebox source that is independently sufficient**, and the whitebox reading supplied the *search space* rather than the value. | 2 or more of the 4 adopted rules have **no** independent black-box confirmation recorded, i.e. the disassembly is load-bearing alone. |

## 3. Targets, frozen

| # | quantity | denominator | target |
|---|---|---|---|
| **T1** | `ADDR.tsv` rows | — | **≥ 1 400** (from 1 209 at `dd127956` / 1 209 at this base), and the increase must be **≥ 60 % from new hand labels**, not from the self-referential prose drift `ref/README.md` §4 warns about |
| **T2** | functions with a row in the new whole-image index | **4 919** (Ghidra functions + verified Ghidra-missed entries) | **100 %** — an index whose denominator is not the image is the thing being replaced |
| **T3** | `CEILING.md` §6.1 phases with an addressed entry in `SUBSYS.md` or a `ref/` page | **7** | **≥ 6 of 7**, up from the 4 this lane expects to measure |
| **T4** | distinct functions named by at least one `ADDR.tsv` row (`ref/README.md` C2, measured 631 = 12.8 %) | **4 919** | **≥ 750 = 15.2 %** |
| **T5** | the strategic answer | — | a **priced** answer, or an explicit "cannot be priced from what exists, and here is the instrument that would price it". Both count as HIT; a hedge with neither counts as MISS |

## 4. Invalidation rules

1. **A `[R]` mark is not a finding.** Every label this lane writes is `[R]`
   unless a grid cell or an obj is named. `ref/README.md` §2 prices this: the
   `.bss` bump rule was read correctly out of a small clean function and was
   **wrong about c2**.
2. **Mechanical labels are marked as mechanical.** A row derived by joining
   `strings.tsv`/`calls.tsv` without a human reading the body carries
   confidence `mech`, never `high`. Conflating the two would make the whole
   index unreadable, and it is the exact defect `C2_MAP_METHOD.md` §7 prices.
3. **Any ranking this lane produces is suspect by default** — the repo is four
   for four on *"ranking instruments measure themselves"*. If a ranking is used
   to choose what to read, its top entries are checked against an independent
   signal before any claim rests on the ordering.
4. **The count drifts because the index is self-referential** (`ref/README.md`
   §4). Every count is quoted with the tip it was taken at, and `build_ref.py`
   is re-run after the last prose is written.
5. **`c2_tus.tsv` cannot see a TU with no ICE site.** No claim of the form "the
   image has no *X*" may rest on the TU partition — that is board `#1823`
   verbatim, and it stood for months.
6. **Never open the Ghidra project.** Only the flat export at
   `$C2RS_GHIDRA_EXPORT` is read.

## 5. Predicted reach — registered

`match` 26 → 26. `fnbyte-exact` 35 886 → 35 886. All 394 `gap-metric` keys
unchanged. Suite 1 681/0/48 plus whatever tests this lane adds. Gate PASS.
**A zero delta here is evidence about reach and is not evidence of
correctness** (`rungs/README.md`, `w-sizebracket`).
