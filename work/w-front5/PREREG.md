# PREREG — lane `w-front5`, frozen 2026-08-09

**The freeze point, stated honestly.** Frozen after the **read-only**
re-derivation recorded in §1 below — the 878-TU base scan, the five frontier
IL captures, the two reference-obj dumps, the hand-transcribed `.gl` binding
walk and the `gate_causes` histogram — all of which ran without touching
`crates/`, and **before the first `crates/` edit** of any kind, including the
counterfactual build P1–P5 are written about. Everything in §1 is a **PRIOR**
and is marked as such; nothing in §2 is answerable from §1 or from anything on
disk. `w-pool` scored 30/31 and correctly called that a calibration failure
because its claims restated the tree; §2 is written to be losable.

Base: master `42871e7704de488721237be405b991f5f83896f0` (the `w-pool2`
STATUS regeneration). Worktree branch `worktree-agent-aea6fcdaf817720a7`.
Workload stamp: dc3-decomp `d7a3c1aa9d5d57a1176790c0e15a723edd2e03a0`,
878 lines in `work/dc3-workload/files.txt` (`wc -l`-checked), flags from
`config/373307D9/config.json` via `scripts/gen_dc3_workload.sh`.
Toolchain `compilers/X360/16.00.11886.00`, wibo from
`../wibo/build/release/wibo`. 341 fixtures at freeze.
Base binary `work/w-front5/c2rs-base`, md5
`53e70e8fa9cfb8a482a2072927b615aa`, built at the base commit and KEPT
(#2409 — never `git checkout master -- crates/`).

---

## 1. PRIORS — what §1's read-only pass already established (UNSCORED)

These are measurements, not predictions. They are listed so that §2 cannot be
read as having predicted them.

* The FRONTIER is **5** and its members are, from this lane's own scan:
  `src/Main.cpp`, `src/xdk/nuispeech/mmio.cpp`,
  `src/system/rndobj/wordwrap.cpp`, `src/system/utl/EncryptXTEA.cpp`,
  `src/keygen_xbox.cpp`. `wordwrap.cpp` is the member no inherited price named.
* `src/Main.cpp`'s FIRST gate cause is **`gl-stop-name-not-mangled`**. Its `.gl`
  carries exactly one framed defined record, at body-start **2713**, which is
  exactly the single `.ex` segment's start; its name is **`main`**, 4 bytes,
  and `INLINE_NAME_MAX` is 8. `gl_defined_names` therefore returns empty,
  `Bindings::per_record` returns `None`, and `IlBundle::functions()` is `None`
  before `main`'s body is looked at.
* `src/xdk/nuispeech/mmio.cpp` binds **4** records — `mmioGetInfo`,
  `mmioSetInfo`, `mmioStringToFOURCCW`, `mmioFlush` — and stops at
  **`mmioSeek`**, 8 bytes, exactly at the boundary. 4 ≠ 11 segments, so
  `per_record` is `None`.
* `wordwrap.cpp` (3:3), `EncryptXTEA.cpp` (5:5) and `keygen_xbox.cpp` (20:20)
  all BIND; their only gate cause set is `body-out-of-class` (+ accounting).
* **15** TUs carry `gl-stop-name-not-mangled` as first cause and **all 15** also
  carry `body-out-of-class`; **0** carry it alone.
* Per-function `.text` sizes off this lane's own reference objs:
  wordwrap 12 / 164 / 640 B; EncryptXTEA 16 (exact) / 12 / 32 / 116 / 96 B,
  with `$M2756`, `$M2757`, `$T2758` and `__savegprlr_26` / `__restgprlr_26`
  on `?Encrypt@`.

## 2. PREDICTIONS — scored in the rung

### 2.1 The counterfactual: relax the bound-record-name clause (P1–P5, P13–P14)

A scratch build in which `gl_defined_names_framed`'s
`runs[k].2.len() <= INLINE_NAME_MAX` refusal is removed, everything else
identical; the 878-TU scan re-run under it; the patch then reverted. This is
what turns "necessary but not sufficient" from an inference about
`decode_causes`' independence into a measurement.

| # | p | prediction |
|---|---|---|
| **P1** | 0.85 | The counterfactual leaves `match` at **22** and `mismatch` at **0**. |
| **P2a** | 0.90 | `src/Main.cpp` BINDS under it, and its first cause becomes `body-out-of-class`. |
| **P2b** | 0.50 | `src/xdk/nuispeech/mmio.cpp` still does **not** bind under it — so mmio owes a SECOND binding repair beyond the name clause. |
| **P3** | 0.60 | `fnbyte-exact` under the counterfactual is **unmoved** (35,798). |
| **P5** | 0.75 | **0** of 878 TUs change `class` under it (0 only-in-base, 0 only-in-tip, 0 changed, by name). |
| **P13** | 0.40 | **≤ 3** of the 15 `gl-stop-name-not-mangled` TUs bind under it. |
| **P14** | 0.30 | ≥ 1 of the 13 non-frontier members of that 15 gets a first cause that is neither `body-out-of-class` nor a `gl-stop-*` clause. |

### 2.2 What this lane ships (P6–P9, P15)

| # | p | prediction |
|---|---|---|
| **P6** | 0.90 | **No `crates/`, `fixtures/` or `scripts/` change ships.** Base binary md5 == tip binary md5, and all three neutrality levels are 0 **by construction** rather than by comparison. |
| **P7** | 0.95 | Reproduced at THIS base tree: `scripts/gate.sh` **18/18 PASS, 0 mismatch anywhere**; `cargo test --workspace --release --no-fail-fast` **1453 passed / 0 failed / 41 targets**; `c2rs selftest` **341 PASS / 0 ERROR**. (`hatch-red` refuses on pre-existing failures; if any of these is red at base it is recorded as pre-existing, not attributed.) |
| **P8** | **0.97** | **`fnbyte-exact` delta = 0** (35,798 → 35,798). **Registered as the scored metric per CEILING §10** — a census-only prediction is unscored. |
| **P9** | 0.95 | `match` **22 → 22**, FRONTIER **5 → 5**, factors A/B/C/D/E unmoved. |
| **P15** | 0.92 | The both-mode fixture scan grades **341 at `/O1` AND 341 at `/Ox`**, list regenerated after the last fixture and `wc -l`-checked, with **0 verdicts moved by name** at either mode. |

### 2.3 The conversion call — mutually exclusive (P10–P12)

| # | p | call |
|---|---|---|
| **P10** | **0.04** | A frontier TU **converts**: match 22 → 23. |
| **P11** | **0.93** | **Five priced declines**, each with a named, sized chain, and **two inherited prices corrected** (Main.cpp's route and mmio.cpp's stop record). |
| **P12** | 0.03 | Neither: the lane cannot re-derive the frontier and ships only a survey. |

### 2.4 The prices themselves (P16–P18)

| # | p | prediction |
|---|---|---|
| **P16** | 0.80 | **No frontier TU is a one-body conversion.** Every one of the five needs either ≥ 3 blocked bodies or a whole-obj binding repair in front of its bodies; the minimum blocked-body count over the three gate-binding TUs is **3**. |
| **P17** | 0.70 | The cheapest gate-binding TU by REMAINING BYTES (`EncryptXTEA`, 256 B) and the cheapest by BLOCKED-BODY COUNT (`wordwrap`, 3) are **different TUs**, so the two published rankings disagree at the head of this frontier — [[ranking-instruments-measure-themselves]], seventh instance. |
| **P18** | 0.60 | `EncryptXTEA.cpp`'s re-derived price is **≥ 27** — i.e. `w-pool`'s standing figure survives re-derivation rather than moving in either direction. |
