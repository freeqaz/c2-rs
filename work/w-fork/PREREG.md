# w-fork — registered predictions, written BEFORE the fork server existed

Written 2026-08-04 21:50 local, after the probe and the cost decomposition, and
**before a single line of the fork server was written**. Nothing below is
edited after the fact; corrections go in the rung doc's §"realized".

## §1 The cost decomposition this is predicated on

`work/w-fork/driver`, sequential, one host process per invocation, 300 reps,
`WIBO_FS_CACHE=1`, warm page cache, **box under concurrent lane load (load avg
34 on a 32-thread 7950X)**:

| stage | wall ms/invocation | child CPU ms |
|---|---:|---:|
| `wibo --version` — bare loader floor | 2.06 | 1.84 |
| `wibo c2load.exe c2.dll` — + LoadLibrary(c2.dll) + DLL init, no compile | 6.20 | 5.81 |
| `wibo c2host.exe … -il … -Fo …` — full replay | 7.88 | 7.21 |

Fixed cost (spawn + PE load + DLL init) is **6.20 of 7.88 ms = 79 %**.
Compile proper is **~1.7 ms** on this corpus.

Note this contradicts `docs/PRIOR_ART.md` §1.3 in one detail: strace of a replay
shows wibo opens **only `c2.dll`** — the six "dependent DLLs" (`msobj*`,
`mspdb*`, `msdis*`, `pgodb100`, `msvcr100`, `msvcp100`) are satisfied by wibo's
**built-in** stub modules and are never read from disk. The 5.8 ms figure there
is therefore not "loading 7 DLLs"; it is one DLL plus wibo's own init.

## §2 Prediction

A fork server removes the 6.20 ms fixed cost and replaces it with
fork + UNIX-socket round trip + waitpid.

* **Predicted per-obj wall: 2.0–2.5 ms** on this corpus, on this box, at this
  contention.
* **Predicted speedup: 3.2×–3.9×.**
* This is **below** the 10–20× that `docs/PRIOR_ART.md` §1.3 and §4 estimate.
  That estimate came from the 5.8 ms load figure without the compile's own share
  measured; with the compile measured at ~1.7 ms, Amdahl caps the fork server at
  **7.88 / 1.7 = 4.6×** even if fork were free. **10–20× is not reachable and
  the review's table should be corrected regardless of how this lane ends.**

## §3 Decision thresholds, registered in advance

| realized speedup | recommendation |
|---|---|
| ≥ 3× | **ADOPT** — build it out and integrate behind `c2-reference` |
| 2×–3× | **REPORT, do not adopt now** — real but not worth the wibo-fork maintenance surface against a warm capture cache |
| < 2× | **DECLINE** — the lane is a negative result |
| any byte mismatch that is not explained and fixed | **DECLINE regardless of speed** — a faster oracle that emits different bytes is not an oracle |

## §4 What would make me decline even at ≥ 3×

The honest denominator (measured separately, see the rung doc): if the fraction
of this project's real oracle work that is **cold** (a capture-cache miss) is
small, the fork server's headline speedup applies to a small slice. I am
registering in advance that I will compute that fraction from the committed
cache and the gate's own numbers, and that I will report it next to the speedup
whatever it says.

## §5 Falsifiers I am committing to run

1. The byte-identity check must compare **fork-server obj vs spawn obj at the
   identical `-Fo` path** (the path string is embedded in `.debug$S`, so two
   different paths legitimately give two different objs — comparing across paths
   would manufacture a false mismatch).
2. The count of objs compared is reported. A run that produces zero objs must
   exit non-zero; `driver` already does this in every mode.
3. The corpus is ≥ 300 real captured bundles, not the three fixtures.
