# w-throughput — running notes

Box condition through the session: **contended throughout.** `uptime` load
average sampled at each measurement, 32 logical cores:

    02:32  4.56   worktree setup
    02:34  3.76   baseline `cargo test --workspace --release` START
    02:37  6.06   baseline `cargo test` END (206 s)
    02:38  5.32   baseline `gate.sh --require-graded` START
    02:42  7.31   baseline sweep leg END (262 s at 4 jobs)
    02:51 37.57   baseline cross leg, cold capture cache, still running
    02:56 71.19   ditto — another lane saturating the box

Two other lanes were gating concurrently for the whole session. Every timing in
the rung doc says which of them was taken under this and which was re-taken.

## Why the baseline gate's cross leg is not comparable to the tip's

`mode_cross.sh` keys the capture cache on the SOURCE PATH and therefore keeps a
**stable** case directory, `<repo_root>/work/mode-cross/cases`. `repo_root` here
is the worktree, and `scripts/setup_worktree.sh` copies only
`work/dc3-workload/` — so the first cross run in a new worktree is **cold**
(the script's own comment prices that at 5 min 45 s vs 13.8 s warm, 25x, at 8
jobs over 61,539 cells; this gate's cross is 90,812 cells at 4 jobs on a box at
load 40-70).

Consequence for the method: the base-vs-tip gate comparison is **not** a valid
A/B for the cross leg. The A/B that IS valid is `--jobs 4` vs `--jobs 16` **on
one tree with the cache already warm**, and that is the pair the rung quotes.
The cold baseline is reported for what it is — the number a fresh worktree
actually pays — and the sweep leg, which does not use the cache at all
(`c2rs diff`, board #282), is directly comparable at both.
