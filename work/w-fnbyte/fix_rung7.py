P = "docs/rungs/2026-08-06-w-fnbyte.md"
s = open(P, encoding="utf-8").read()

old = """| lane | result |
|---|---|
| `cargo test --workspace --release` | **targets=28 passed=916 failed=0** |
| `scripts/gate.sh --jobs 6` | **18/18 PASS**, 0 mismatch — counts in §7.1 |"""
new = """| lane | result |
|---|---|
| `cargo test --workspace --release` | **targets=28 passed=916 failed=0**, measured at the tip. The recorded baseline is `w-seam`'s **911 / 0 / 27**; this lane adds four unit tests and one integration target, and `911 + 5 = 916`, `27 + 1 = 28` closes. |
| `scripts/gate.sh --jobs 6` | **GATE: PASS — 18/18 lanes ran and every one graded a corpus**, `0 FAIL, 0 SKIP, 0 NO-RESULT`, **4,770 fixture-verdicts**, sweep **16,614 of 16,710 graded / 0 mismatch**, cross **81,517 of 81,905 cells graded / 0 mismatch**, **0 mismatches anywhere**. Run twice — once at `840ab02` and again at the tip `f61db31` — with identical counts (`work/w-fnbyte/gate_tip.scrubbed.txt`) |
| `c2rs selftest` · `c2rs perf` | **265 PASS / 0 FAIL** · **124 port Match, 0 mismatch, 141 not-implemented of 265** |"""
assert s.count(old) == 1, "gate table anchor"
s = s.replace(old, new, 1)

old2 = """| FBM partition, final | `exact 34466 · whole-TU 2 · differs 4711 · partial 0 · refused 130573 · unbound 9225 · no-bytes 0` of **178,975** |"""
new2 = old2 + """
| `gap-metric` diff, tip vs the final scan | **all 72 lines IDENTICAL** — the two cosmetic commits after the graded tree move no number |"""
assert s.count(old2) == 1
s = s.replace(old2, new2, 1)

open(P, "w", encoding="utf-8").write(s)
print("rung §7 updated with the measured gate and test counts")
