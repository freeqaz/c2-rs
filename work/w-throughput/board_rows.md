# Board rows minted by w-throughput (draft; the landed copy is docs/BOARD.md)

Range assigned: **#1323–#1332**. Minted: **#1323–#1330** (eight rows).
**#1331 and #1332 are left UNMINTED** — the lane found nothing further worth a
permanent number, and a row minted to fill a range is a row nobody can retire.

Placement:

| # | section | one-line |
|---|---|---|
| 1323 | Done | `gate.sh` default `--jobs` 4 → 16; the constant was never a decision |
| 1324 | Done | the killed-worker false-green check, verified by killing a worker |
| 1325 | Done | `C2RS_JOBS` deliberately NOT raised — it multiplies, and its leg is 2 s |
| 1326 | Done | `cli_flags` split 1 → 4 tests; ~116 s serial becomes ~44 s parallel |
| 1327 | Done | the roster partition control — what splitting a test can newly break |
| 1328 | Done | `status.sh --tests-log`, gated on a derived closure and a short count |
| 1329 | Done | a must-fail suite whose expectations are not distinct does not fail |
| 1330 | Declined | the docs-only re-gate skip fires on **0 of the last 40 merges** |
