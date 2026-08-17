# PREREG — lane w-npos

Frozen as this lane's FIRST commit, before any probe, any scan, any obj read,
and any `crates/` change by this lane.

## 0. Identity, base, and what precedes this freeze

* Lane: **w-npos**. Branch `wt-w-npos`, worktree
  `.claude/worktrees/w-npos`, base master **`3835469c`**.
* **Measurements taken by this lane before this freeze: NONE.** No scan has
  been run, no obj has been read, no probe has been compiled. What this lane
  has done before the freeze is read committed source (`coff/shell.rs`,
  `coff/container.rs`, `coff/data.rs` heads, `c2-il` `bundle.rs`/`diag.rs`,
  `gap/mod.rs`/`factors.rs`/`scan.rs` excerpts) and the landed rung
  `docs/rungs/2026-08-16-three.md`. This is the fix `w-three` §10 names: its
  own prereg was frozen *after* its base scan and scored 19/19 for it; this
  one is frozen before.
* Handed-down figures, quoted at the commit they were measured at
  (`3835469c` unless said otherwise): match 25 · mismatch 0 · codegen-gap 0 ·
  vocab-gap 845 · capture-fail 8 · frontier 2 · fnbyte-exact 35,734 ·
  fnbyte-denominator 162,049 · fnbyte-refused-parse 113,612 ·
  fnbyte-refused-codegen 949 · anchored `gap-metric` keys 394 ·
  `cargo test --workspace --release` 1,648 / 0 / 42. From `w-three` (measured
  at `202bfc3f`, brief base `55933035`): decomp_pch.cpp reference obj
  **905 B**, 5 sections, 0 `.text`, 0 relocations, 0 undefined externals, one
  `sel=2` `.rdata` COMDAT
  `?npos@?$basic_string@DV?$char_traits@D@stlpmtx_std@@V?$allocator@D@2@@stlpmtx_std@@2IB`
  = `ff ff ff ff`; the three reader-clear TUs stop at `gl-stop-26-introduced`
  with `gate_causes` `[gl-stop-26-introduced, body-out-of-class]`; prices
  5 (decomp_pch) · 6 (vec) · 8 (NetworkSocket).

## 1. Mission and route

Convert `src/system/decomp_pch.cpp` (match 25 → 26) by:

1. a **whole-TU recognizer** in `c2-il` (front-of-gate acceptance — the route
   `w-three` §7.1 names as skipping mechanisms 1–3): a decidable pre-emission
   predicate proving, from the IL container alone, that c2's emit set for the
   TU is the four-section shell **plus exactly the COMDAT const-data records
   the recognizer licences and nothing else** — in particular zero functions;
2. a **general `sel=2` `.rdata` COMDAT emitter** in `crates/c2-core/src/coff/`
   (new file), N objects, section symbol + aux CheckSum + EXTERNAL object
   symbol, contents from `.in`, refusing (returning `None`) on anything the
   probe grid has not graded;
3. wiring: `PortC2::build` tries the recognizer **after `functions()` refuses**
   (zero cost on every accepted TU); `IlBundle::decodes()` gains the third…
   fourth path; the recognizer is **registered in
   `gap::WHOLE_TU_RECOGNIZERS` in the same commit** (the D∨E control
   otherwise goes red on the converted match, by design).

Sole judge: real `c2.dll` under wibo, byte-exact obj compare with
TimeDateStamp (4..8) zeroed. **A wrong emit scores strictly below the refusal
it replaces**: if byte-exactness is not reached, the lane lands the honest
refusal and declines, with the price refined.

## 2. Predictions — probability form; ceilings carry no discount factor

| id | prediction | p |
|---|---|---:|
| **P1** | The conversion lands: tip 878-TU scan reads `match 26`, `mismatch 0`, `codegen-gap 0`, decomp_pch row `class=match`. | 0.55 |
| **P2** | The base-end scan reproduces `w-three`'s profile on all three TUs: `vocab-gap`, `gate_cause=gl-stop-26-introduced`, `gate_causes=[gl-stop-26-introduced, body-out-of-class]`; and every handed-down §0 figure reproduces at `3835469c`. | 0.95 |
| **P3** | decomp_pch's regenerated reference obj at the workload's own flags is 905 B, 5 sections, 0 `.text`, 0 relocations, 0 undefined externals, one `sel=2` `.rdata` COMDAT of 4 B = `ff ff ff ff` under the `?npos@…2IB` name. | 0.90 |
| **P4** | decomp_pch's `.gl` carries **exactly one** COMDAT+initialized data record whose emission this class needs — the `?npos@…` one, size 4, external, `.in` value `ff ff ff ff`, zero initializer references — and every other data record it carries is licensed away by an already-measured rule (internal+uninitialized+unreferenced drop) or is absent. | 0.70 |
| **P5** | Grid g03: obj = shell + one `sel=2` `.rdata` COMDAT, 4 B `ff ff ff ff`, zero `.text` COMDATs — the decomp_pch shape reproduces in miniature. | 0.80 |
| **P6** | A decidable discriminator separates the const (`.rdata`) COMDAT static from the non-const (`.data`) one (g03 vs g04/g10). Two registered hypotheses, either scores a HIT: (a) a `.gl` record attribute bit differs; (b) the **mangled name's own cv-code** — `…IB` (const) vs `…IA` — carries it. If neither discriminates, mechanism "5" in its general form is refused fail-closed and the lane declines. | 0.65 |
| **P7** | Zero-roots soundness: g02 and g13 emit the bare shell. **Danger cell g12 (explicit instantiation), registered two-sided**: p(g12 emits `.text` with zero ordinary roots) = 0.60; if it does, p(the IL container distinguishes g12's TU from g03's by a decidable property) = 0.50. If g12 emits and is not distinguishable, the predicate gains a clause that refuses whatever cheap over-approximation is needed (e.g. refusing any TU whose `.ex` carries a segment for a name of g12's linkage shape) or the lane **declines**. | — |
| **P8** | `vec.cpp` and `NetworkSocket.cpp`: the recognizer refuses **both** at tip (vec via its non-COMDAT `.data`/reference clauses, NetworkSocket via roots or its `.text` needs); neither moves class. Measured and reported, not chased. | 0.85 |
| **P9** | Anchored `gap-metric` keys: **394 → 395**, the one new key being `emit-whole-tu\|<recognizer-name>`. | 0.70 |
| **P10** | Census +0 both axes; `fnbyte-*` all +0; factors A–D counts, `b-and-c`, `a-and-b-and-c`, binding invariants, section vocabulary: all +0. | 0.90 |
| **P11** | `scripts/gate.sh --jobs 4 --require-graded` PASS at both ends; `cargo test --workspace --release` 0 fail at both ends (tip count > 1,648 by exactly this lane's new tests). | 0.85 |
| **P12** | Anti-inflation: at least one of the briefed five mechanisms is found already-paid or skipped whole by the front-of-gate route (w-three §7.1 predicts 1–3 are skipped by exactly this route). | 0.90 |

**Ceilings (absolute, no discount):**

* `match` at tip ≤ **26**. This lane names no TU other than decomp_pch.cpp as
  convertible by it.
* `mismatch` = **0** in every scan, every gate lane, every fixture population,
  at both ends. A single mismatch anywhere is an alarm: the acceptance is
  reverted (recognizer tightened or the arm removed), never the judge.
* `codegen-gap` at tip = **0**: the recognizer's accept-set must equal the
  emitter's emit-set on the workload — a TU accepted and then refused by the
  emitter is a new `codegen-gap` row and counts as a broken ceiling.
* `capture-fail` 8, `port-error` 0, `frontier` 2 — unchanged.

**Enumerated movable keys** (identity diff over anchored keys; anything not
on this list moving is an alarm, not a result):

* `match` (+1), `vocab-gap` (−1), `factor-e` (+1),
  `a-and-b-and-c-and-d-or-e` (+1), `emit-whole-tu-any` (+1),
  `emit-whole-tu|<name>` (NEW), and the fence family exactly as decomp_pch's
  row leaves it: `fence-held-tus` (−1), `fence-cause-firings` (−2),
  `fence-match-tus-checked` (+1), and the
  `fence-blocks-{sole,first,exact,exact-bodies}:gl-stop-26-introduced` /
  `:body-out-of-class` rows by the loss of that one row's contribution.
* Every one of these moves must be exactly attributable to decomp_pch's row;
  the per-TU identity diff must show **877 of 878 rows identical**.
* If the conversion is declined: **394 → 394, zero keys move, 878/878 rows
  identical** (a docs-and-`work/`-only tree, w-three §12's pattern).

## 3. Mutants — colours registered before any run

Controls mutate the **input** (a probe copy, a stream copy, the port's own
output artifact), never the oracle. Runner `work/w-npos/mutate.sh` exits
non-zero on any off-prereg colour.

| id | mutant | registered |
|---|---|---|
| M1 | comment-only edit to a copy of g03, recompiled through real c2 | **GREEN** — obj byte-identical |
| M2 | g06 vs g03 (value `0x12345678` vs `~0u`) | **FIRES** — `.rdata` content and aux CheckSum both move, and the differential sees it |
| M3 | g03 + an ordinary root function appended (input mutation) | **FIRES** — the recognizer refuses (zero-roots clause) |
| M4 | decomp_pch's `.gl` truncated at half length, fed to the recognizer | **REFUSE** — `None`, no panic |
| M5 | an emitter input object carrying one initializer reference | **REFUSE** — emitter returns `None` |
| M6 | one byte flipped in the port's emitted obj before compare | **FIRES** — the byte compare fails; the judge can see one byte |

## 4. The grid, frozen by content hash

Committed in this same commit under `work/w-npos/probe/`; compiled only at the
workload profile (`work/dc3-workload/flags.txt`; `w-section` measured `/Ox`
disagreeing on 7 of 8 fields, so no probe is quoted at any other profile).

```
d9c390915d69629b6609239162f45a02951dd5034cd95dc969a18b4b117f4d02  g01_root.cpp
6d6f4bb70be12fc84081a96452eecdf784205cfb981f4535bfcf2ab07b66b730  g02_inline_only.cpp
5e01a5d37bb87bff2e6a04210c4e10960ef2355094683fbc15bdff9864e0c373  g03_npos_mini.cpp
d8ca8df625856b32533083c5425ef7ce3ed3ab688e5373aaaa50d932cf4b1c68  g04_nonconst.cpp
7c39ea74fa35f308c0c1a28b411c99f8b9d207235aacb91480a6117a7ef69be7  g05_two_sizes.cpp
6a66df55299a8af42fcc3bbe4c1e34657f447fb0953dc65ce4f4433c26f46861  g06_value.cpp
f19de86b0493add8433e83313afb16de2e48801f9d570369af296eb9461e5f11  g07_fptr_root.cpp
014db0aca368927d26ee81ba59bdba8371d36bdd4adfb4c0fc81da1b1d77147d  g09_dead_static.cpp
b44ce422e65e244ba9f0f552dbb9bc710073eb06166a725547c1d91b5005774f  g10_selectany.cpp
37c62d3c3057f693bd38f13f1d85d34f684f5dd1a3829775e17b6ce21d49816a  g12_explicit_inst.cpp
c8957cf1db538b341a23ed392a7c4bda8d8af3683d6db0118c989f95c27415e9  g13_inline_chain.cpp
```

(g08 and g11 were drafted and cut before the freeze — placement-beside-`.text`
and internal-linkage `.rdata` are outside this conversion's class and would
have been cells without a consumer. The gap in the numbering is deliberate and
recorded rather than renumbered.)

## 5. Verification owed (both ends)

* 878-TU scan at base and tip; anchored-key diff (§2's enumerated movers
  only); per-TU row diff (877/878 identical on conversion, 878/878 on
  decline).
* `scripts/gate.sh --jobs 4 --require-graded` PASS; since this lane lands
  tests/fixtures, the checks are the **gate-count identity diff over
  unaffected lanes + release-binary sha256** (#3215), not "graded tree
  identical".
* `cargo test --workspace --release --no-fail-fast` with counts, from stdout
  alone; `scripts/debug_lane.sh`; `scripts/board_audit.sh`; `rung_registry`.
* Rung doc from `_TEMPLATE.md`, `Kind: fixture-claim`, one-word `Outcome:`;
  `scripts/gen_rung_index.sh`; board rows drafted UNNUMBERED unless the
  pointer verifies row-by-row (pointer read: **#3218 at `3835469c`**, two
  peers in flight).
* Disassembly is not planned; if any Ghidra-derived fact is adopted, a
  `docs/whitebox/DISCLOSURE.md` row lands in the same commit. Preference is
  black-box probe objs at the workload profile throughout.
