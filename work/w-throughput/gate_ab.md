# The `--jobs` A/B: the verdict counts, shown identical

## Method, and why it is this method

`mode_cross.sh` keys the capture cache on the source path and therefore keeps a
**stable** case directory under `<repo_root>/work/mode-cross/cases`. A fresh
worktree starts that directory empty, so the FIRST cross in a worktree is cold —
the script prices it at 5 min 45 s cold vs 13.8 s warm at 8 jobs over 61,539
cells, and this gate's cross is 90,812 cells. **A base-vs-tip gate comparison out
of a fresh worktree measures the cache, not the concurrency.**

The base tree still contributes two things that no cache state can distort: the
sweep leg (which never touches the cache — it drives `c2rs diff`, board #282),
and the identity of the binary under test:

    base run (jobs 4):   sha fbe465a1ffb1   tree f49fe5e1
    tip runs:            sha fbe465a1ffb1   tree f49fe5e1-dirty
    sha256sum target/release/c2rs -> fbe465a1ffb1...

**The pinned harness is byte-identical at both ends.** This lane changes one test
file, two shell scripts and docs — nothing the binary is built from. That is also
why the 878-TU scan cannot move, and it is checked rather than assumed.

## THE A/B — one tree, warm cache, back to back, 2026-08-08 03:20-03:32

| leg | `--jobs 4` | `--jobs 16` (the new default) |
|---|---:|---:|
| lanes (18) | 3 s | 2 s |
| generated sweep (19,556 cases) | 252 s | **88 s** |
| mode cross (90,812 cells) | 350 s | **21 s** |
| **total wall** | **605 s** | **112 s** |
| `/tmp` free-inode draw (start − low water) | 39,535 | 20,039 |

**5.4× on the whole gate.** Load average was 4-20 across the pair, one other
lane's gate intermittently present; the two runs are ten minutes apart on the
same tree with the same warm cache, which is the closest to a controlled pair
this box allowed.

## THE EQUIVALENCE — identical, digit for digit

Both runs, and the third (cold-cross) run at `--jobs 16` before them:

    lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
    graded: 5184 fixture-verdicts across all lanes
    sweep:  checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0
    cross:  checked=90812 mismatches=0 graded=90424 ungraded=388 unknown=0
    GATE: PASS — 18/18 lanes ran and every one of them graded a corpus,
      the sweep graded 19460 of 19556 generated cases and the cross graded
      90424 of 90812 case-lane cells, with 0 mismatches anywhere

And the base tree's own `--jobs 4` sweep leg, before any edit:

    checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0   (262 s)

**Nothing about what is graded moves with the worker count.** The corpus is
generated before the split; `awk -v j="$jobs" '{ print > (d "/chunk." ((NR-1) % j)) }'`
puts every case in exactly one worker; the workers' own counts are summed and
reconciled against the selected count afterwards.

## INODE HEADROOM AT THE NEW DEFAULT

    --jobs 4    733,526 free at start -> 693,991 low water   (draw 39,535)
    --jobs 16   733,426 free at start -> 713,387 low water   (draw 20,039)

Against the floor of **150,000** (`C2RS_GATE_MIN_INODES`, 3× a measured in-flight
peak) and 733k free, the headroom at the new default is **4.8×** on the low
water. The concurrency does not scale the draw — the proposal measured 19,810 at
4 and 19,885 at 16, and the 20,039 above lands on that. The larger figure is the
`--jobs 4` run's, because a slower cross holds its transient tree for longer;
raising the concurrency made the draw *smaller*, not larger.

## THE ONE TIMING THAT SAYS THE BOX MATTERS MORE THAN THE FLAG

Earlier in the same session, with **four `gate.sh` processes running at once**
(`ps -e -o comm= | grep -c gate.sh` → 4) and load average 67-100 on 32 logical
cores:

    sweep, --jobs 4  (base tree, load 7-11)     262 s
    sweep, --jobs 16 (tip tree,  load 67-100)   309 s

At that load a 16-worker gate is queueing, not parallelising, and the flag reads
backwards. The counts were identical there too — which is the point of separating
them: **the equivalence is a property of the split and the timing is a property
of the day.**
