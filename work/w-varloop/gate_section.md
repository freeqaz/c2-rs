### 8.1 The numbers registered BEFORE the run

Baseline from w-sched2 §10.1, itself carried from w-rotate §10.1: **871
workspace tests / 27 targets**, **18/18 lanes**, **4,680 fixture-verdicts**,
sweep **16,710 selected / 16,614 graded / 96 ungraded**, cross **81,517 of
81,905 / 388 ungraded**, `status.sh --check` PASS, `board_audit.sh` 0/0/0.

**Two of those numbers are EXPECTED to move here and the rest are not**, which
is the distinction that makes the others worth checking: this lane adds four
fixtures and twenty-two tests, so the fixture-verdict count and the test count
must move by exactly that much and everything else must not move at all. A
changed number anywhere else is a failure rather than a curiosity.

### 8.2 The run, read from its log

| lane | result |
|---|---|
| `cargo test --workspace --release` | **893 passed, 0 failed, 27 targets** — 871 + 22, and the target count unchanged |
| `scripts/gate.sh --jobs 6` | **GATE: PASS — 18 in the registry, 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT** |
| fixture-verdicts | **4,770** across all lanes — 265 fixtures × 18, and 265 = 261 + this lane's 4 |
| generated sweep | `checked=16710 mismatches=0 graded=16614 ungraded=96` — **every digit the baseline's**, the 96 held |
| mode cross | `81517 of 81905 graded, 0 mismatch` — **every digit the baseline's**, the 388 held |
| `scripts/status.sh --check` | **PASS — 23 metrics registered, parsers pinned, absence renders NO-RESULT** |
| `scripts/board_audit.sh` | **0 / 0 / 0** — `CITED BUT NOT ON THE BOARD: 0`, unresolved anchors 0, raw line-number anchors 0, rows-behind-the-prose 0 |
| `work/w-varloop/vargrid.py` | **66 reached / 66 graded / 0 capture failures / 0 controls failed / 0 mismatches** |
| `work/w-varloop/mutate.py` | **6 of 6 mutations turn red, 0 survived**; tree restored, verified by `git diff --quiet` |

**The per-lane match counts, and what they say about the refusal:**

```text
  O1 / O1-EHsc / O1-Oi / O1-Oi-EHsc / O1-Oi-GR / O1-Oi-EHsc-GR   128 match
  Ox / Ox-EHsc / Ox-GR / Ox-EHsc-GR / O2 / O2-EHsc               124
  Ox-Gy / Ox-Gy-EHsc                                             122
  Od / Od-EHsc / Od-GR / Od-EHsc-GR                               10
```

The six `/O1` lanes are **+3** on the three fixtures this class matches
(`wvl_chain3`, `wvl_chain6_same`, `wvl_chain_two_lengths`); the twelve
non-`/O1` lanes are **+0**, because the shape refuses outside `/O1` and
`wvl_chain_neg` refuses everywhere. **That split is the mode refusal being
graded rather than asserted**, across twelve lanes that would each have caught
it emitting the `/O1` body somewhere it does not belong.

**No number outside the two that were supposed to move, moved.**
