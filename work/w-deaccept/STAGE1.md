# Stage 1 — the poisoned sink, decomposed token by token

Base `a238180b`, no `crates/` change, 878 TUs, `--jobs 16`, ~7.4 s per scan.
Every row is one full 878-TU `c2rs gap` run with exactly one env var set.

| sunk token | `match` | `fnbyte-exact` | `fnbyte-refused-parse` | `mismatch` | de-accepts? |
|---|---:|---:|---:|---:|---|
| — (base) | **25** | **35,734** | 113,612 | 0 | — |
| `op:41` | **24** | **33,040** | 116,306 | 0 | **YES −1 / −2,694** |
| `op:55` | **19** | **32,479** | 117,762 | 0 | **YES −6 / −3,255** |
| `op:32` | 25 | 35,734 | 113,612 | 0 | no |
| `op:9B` | 25 | 35,734 | 113,612 | 0 | no |
| `op:30` | 25 | 35,734 | 113,612 | 0 | no |
| `op:4F` | 25 | 35,734 | 113,612 | 0 | no |
| `op:53` | 25 | 35,734 | 113,612 | 0 | no |
| `op:54` | 25 | 35,734 | 113,612 | 0 | no |
| `op:4B` | 25 | 35,734 | 113,612 | 0 | no |
| `op:29` | 25 | 35,734 | 113,612 | 0 | no |
| `op:38` | 25 | 35,734 | 113,612 | 0 | no |
| `op:39` | 25 | 35,734 | 113,612 | 0 | no |
| `op:3A` | 25 | 35,734 | 113,612 | 0 | no |
| **`op:41,op:55`** | **18** | **29,785** | 120,456 | 0 | −7 / −5,949 |
| **the whole 22-token ladder tip** | **18** | **29,785** | 120,456 | 0 | −7 / −5,949 |

## The two things this says

1. **PERFECT ADDITIVITY, and the last two rows are IDENTICAL.** `op:41` alone
   costs −1 / −2,694; `op:55` alone costs −6 / −3,255; together they cost
   −7 / −5,949 — which is, to the unit, **the entire published cost of
   w-readphase's 22-token ladder tip**. The other **twenty** tokens contribute
   exactly **zero**.

2. **THE MECHANISM IS NOT THE POISON.** `keydiff.py base -> s41`: all **+2,694**
   land on **`expr-jump`** (298 → 3,731) and **ZERO** land on
   `expr-chain-sink-poison`. `0x41` and `0x55` are two of `parse_expr`'s three
   `stop` bytes (`mod.rs:2815` and `assign.rs:214` use `0x41`, `calls.rs:1255`
   uses `0x55`, `assign.rs:150` uses `0x32`), and `chain_sink()` is consulted
   **before** the `b == stop` check at `expr.rs:1568` — deliberately, board #663,
   so that a chain walk runs past the first `return`. Sinking a stop byte
   therefore makes every accepted walk run off the end of its own expression.

   `op:32` is the control that keeps this from being a tautology: it **is** a
   stop byte (`assign.rs:150`) and it de-accepts **0**, so the rule is not
   "stop byte" but "**a stop byte an accepted production actually reaches**".
