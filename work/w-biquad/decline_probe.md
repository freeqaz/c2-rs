# w-biquad — the refusal-key probe, applied and REVERTED

Board **#1704**'s defect paying again: `try_parse_fp_store_diamond` returns
`Err(Block)` and the dispatch site discards it, so a declining body reports the
arm's blocker and the recognizer's own key is unreachable from outside. The
patch below was applied, run, read and reverted; it is recorded here so the
reading is reproducible and so nobody re-derives it by adding a permanent
`eprintln`.

```diff
-            if let Ok(shape) = try_parse_fp_store_diamond(seg, p, lo) {
-                disp("disp-fp-store-diamond");
-                return Ok(shape);
-            }
+            match try_parse_fp_store_diamond(seg, p, lo) {
+                Ok(shape) => { disp("disp-fp-store-diamond"); return Ok(shape); }
+                Err(e) => {
+                    if std::env::var("W_BIQUAD_PROBE").is_ok() {
+                        eprintln!("W-BIQUAD-PROBE {}", e.feature());
+                    }
+                }
+            }
```

Run as

```sh
W_BIQUAD_PROBE=1 ./target/release/c2rs gap --list work/w-biquad/one.txt \
    --flags-file work/dc3-workload/flags.txt --cwd <dc3> --no-cache
```

## What it read, and what it was worth

`W-BIQUAD-PROBE assign-0x4F` — twice, once per capture of the TU.

`assign` is `eat_return_head`'s own context and `0x4F` is a **line marker**. So
the whole 35-word body had already parsed — the guard, both arms, the join and
the two-pool shape — and the walk stopped one token short, on the `4F 01 15`
that carries the source line of the closing `}`. `eat_return_head` requires its
`3A` immediately; the marker belongs to the statement run that just ended, not
to the plumbing, so the fix is one `eat_opt_stmt_marker` at the call site rather
than anything about the grammar.

**Without the probe the reading available from outside was `expr-cmp-eq`** — the
arm's blocker, unchanged — which says nothing about whether the recognizer got
one byte in or thirty-four words in. That is the whole cost #1704 keeps
recording.
