# The sink's de-acceptance, censused — all 49 pinned tokens, one scan each

Base `a238180b`, 878 TUs, `--jobs 16`, ~7.5 s per scan. Every row is one full
`c2rs gap` run with exactly one `C2RS_SINK_CHAIN` value set. `mismatch` is **0**
in every row of every table on this page — **63 scans, 0 mismatch**.

Base: `match` **25** · `fnbyte-exact` **35,734** · `fnbyte-refused-parse`
**113,612** · 372 `gap-metric` keys · 878 verdict lines.

## 1. The census — 5 of 49 de-accept, 44 are exactly neutral

| sunk token | `match` | `fnbyte-exact` | Δ | what `parse_expr` does with that byte TODAY |
|---|---:|---:|---:|---|
| `op:33` | **20** | **32,022** | −5 / −3,712 | **accepts** — the INT LITERAL arm |
| `op:B9` | **21** | **32,322** | −4 / −3,412 | **accepts** — the LOAD arm |
| `op:55` | **19** | **32,479** | −6 / −3,255 | **`stop`** at `calls.rs:1255` |
| `op:41` | **24** | **33,040** | −1 / −2,694 | **`stop`** at `mod.rs:2815`, `assign.rs:214` |
| `op:2C` | **24** | **35,163** | −1 / −571 | **accepts** — the CONVERT arm, on a modeled target |
| the other **44** | **25** | **35,734** | **0 / 0** | **refuses** — every one of them |

The 44 neutral ones, in full: `02 03 04 05 06 09 0A 0B 0C 0D 0F 1A 1F 20 21 22
23 24 26 27 28 29 30 32 35 38 39 3A 40 43 44 4B 4C 4F 53 54 5C 5D 5E 66 67 99 9B
BD`.

> ### The rule, and it has no exceptions in 49 tries
>
> **The poisoned sink de-accepts if and only if the sunk byte is one
> `parse_expr` already handles** — either as a production it accepts (`33`,
> `B9`, `2C`) or as one of its `stop` bytes (`41`, `55`). On every one of the
> **44** bytes `parse_expr` currently **refuses** — which is every byte a real
> widening could possibly target — the sink is **exactly, bit-for-bit neutral**.
>
> `op:32` is the control that keeps this from being a tautology about stop bytes:
> it **is** a `stop` (`assign.rs:150`) and it de-accepts **0**, because no
> accepted production reaches it.

## 2. Two mechanisms, both instrument-only, and both PROVEN rather than argued

**(a) The stop-byte override.** `chain_sink()` is consulted **before** the
`b == stop` check (`expr.rs:1627`, deliberately, board **#663**, so a chain walk
runs past the first `return`). Sinking a stop byte therefore walks every accepted
expression off its own end. `keydiff base -> op:41`: all **+2,694** land on
**`expr-jump`** (298 → 3,731) and **zero** on `expr-chain-sink-poison`.

**MEASURED, not inferred.** A scratch mutant that moves the `b == stop` check
**above** the sink (reverted; `git diff` on `crates/` is empty at this tip):

| | `op:41` | `op:55` | the other 11 tested |
|---|---:|---:|---:|
| sink **before** stop (shipping) | `match` **24**, `fnb` 33,040 | `match` **19**, `fnb` 32,479 | neutral |
| sink **after** stop (mutant X) | `match` **25**, `fnb` **35,734** | `match` **25**, `fnb` **35,734** | neutral |

The whole −7 / −5,949 attributed to those two tokens **vanishes** when the
ordering is reversed. It is an ordering decision inside the instrument.

**(b) Production replacement.** Sinking `33`/`B9`/`2C` does not widen anything —
those bytes are already accepted. It **replaces a working production with a
width-skip**, so the walk pushes no `IlOp` and is refused either by the poison
(`op:2C`: +3,525 on `expr-chain-sink-poison:mid`) or, more often, by the
`ops.is_empty()` arm that shadows it (`op:B9`: +3,439 `expr-empty-0x55`, +1,114
`expr-empty-0x32`; `op:33`: +4,470 `expr-empty-0x55`, +2,580 `expr-empty-0x41`).
**Sinking an already-accepted token is a NARROWING dressed as a widening.**

Neither mechanism is available to a real widening. A real widening targets a byte
that is refused today — and the sink is neutral on all 44 of those.

## 3. The floor, and a correction to this page's own first draft

| spec | `match` | `fnbyte-exact` |
|---|---:|---:|
| `op:41` | 24 | 33,040 |
| `op:55` | 19 | 32,479 |
| **`op:41,op:55`** | **18** | **29,785** |
| **`op:2C,op:33,op:B9`** | **18** | **29,785** |
| **all five** | **18** | **29,785** |
| w-readphase's **22-token ladder tip** | **18** | **29,785** |
| the **49-token + type/convert/intrinsic CEILING** | **18** | **29,785** |

`41` and `55` are additive on `fnbyte-exact` (2,694 + 3,255 = 5,949) and on
`match` (1 + 6 = 7) because their function sets are disjoint. **But the first
draft of this page called that "perfect additivity" and it is not: it is
SATURATION.** Mechanism (b)'s three tokens reach the *identical* floor on their
own, and so does the full ceiling. **29,785 is a floor** — the byte-exact
functions whose emission never routes through `parse_expr` at all — and
**5,949 is the whole population any sink can remove**, by either route.

So w-readphase's published *"at the 22-token ladder tip it is `match` 25 → 18 and
`fnbyte-exact` −5,949"* is not a cost that grows with the widening. It is the
instrument hitting its floor, and the twenty tokens that are not `41` or `55`
contributed **zero** to it.

## 4. Reproducing

```sh
./work/w-deaccept/scan.sh <name> [C2RS_SINK_CHAIN=<spec>]     # one 878-TU scan
python3 work/w-readphase/keydiff.py base.jsonl <name>.jsonl   # where they land
```

`work/w-deaccept/census.tsv` is the 49-row table (`token match mismatch
fnbyte-exact`). `ceiling_with.txt` / `ceiling_without.txt` are **derived from
`chain_skip_form` by parsing the tree**, never typed — `greedy.py`'s
`pinned_opcodes()` rule, which refuses rather than returning an empty set.
