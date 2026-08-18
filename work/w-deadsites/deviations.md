# `w-deadsites` — deviations and additions

Appended in order. Every entry names what changed against
`docs/rungs/_2026-08-18-w-deadsites-prereg.md` and why.

## D1 — the gate needs `--allow-dirty-crates` under a probe patch, and that REFUSES one row

`scripts/gate.sh` refuses outright on a dirty `crates/` (its own board #2668 /
#2907 protection). Under `--allow-dirty-crates` it proceeds and **refuses the
`hatch-red` row instead**, because that row's arms write into `crates/`.

Consequence, stated rather than absorbed: **`hatch.py`'s open-hatch
configurations are outside the probed corpus.** That matters for exactly one
row — `CA13` (`calls.rs:772`), whose own comment names
`work/w-front3/hatch.py`'s `call-arg-outer-formal` hatch as the configuration
that makes it live. `CA13` is therefore reported as *dead in the shipped
configuration and live under an instrument's hatch*, which is a different
verdict from dead, and it is not deleted.

## D2 — `writeln!` is not atomic across processes; the probe writes one buffer

Run `P1`'s hits file contained the line `X2X2`. `writeln!` on a `File` issues a
syscall per format piece, so two processes appending concurrently interleaved
half-lines. `deadprobe::hit` builds the whole line and issues one `write_all`.
**`P1` is discarded and re-run from scratch** rather than parsed leniently —
a colour taken with a known-defective instrument is void, not provisional
(`docs/rungs/README.md` probe rule 1). Its logs are kept as `P1.*`.

## D3 — the corpus is run in three parts and the gate is backgrounded

A full gate at `--jobs 16` runs past this session's 10-minute foreground
command ceiling (the generated sweep alone is 92 s and the 90,812-cell mode
cross is the long pole). Runs are therefore launched detached and waited on.
No figure changes; only the invocation does.

## D4 — master advanced under this lane and it was NOT rebased

Peer `w-coldcross` merged (`5f42e9b27`) while this lane was running, moving
`scripts/gate.sh`, `scripts/mode_cross.sh`, `scripts/expr_sweep.sh` and deleting
`scripts/corpus_dir.sh`. **This lane was instructed to rebase only on the
coordinator's go-ahead and did not.** Consequence for reading its figures:
`git diff master..HEAD` is misleading and every diff in the rung is quoted
against the lane's own base `1744ced1`, where it is exactly three files
(two new tests + the one proven-dead deletion). Both gate runs quoted here graded
the tree at `1744ced1 + this lane`, not master's.

## D5 — X3 was a control this lane invented and its premise was wrong

The prereg carried four extra controls (`X1`–`X4`), one census-RED site per file
the probe touches, on the reasoning that a RED colour implies the site is
reached. That implication holds for `false &&`-style mutations **only if the
RED itself was sound**. `X3` (= `w-mutcensus` `B9`) is quiet under two screens
and a `panic!()`, which is incompatible with its published RED. The stop rule
(prereg H3) is **answered rather than waived** — rung §3.4 — and the
contradiction is published as board **#3281** rather than resolved.
