# What actually exhausts /tmp — measured, 2026-08-05, lane w-ledger

    box uptime at measurement       2 weeks (boot 2026-07-21) — /tmp was NOT drained by a reboot
    /tmp                            tmpfs, 47 GB, 1,048,576 inodes (hard ceiling)

## One gate.sh run tree

    /tmp/c2rs-gate-1742827          112 MB   16,660 inodes
    /tmp/c2rs-gate-2289810          112 MB   16,541 inodes
                                    -> call it 16.6k inodes, 112 MB per run

## Which resource binds first

    inode ceiling / 16.6k   =  ~63 run trees exhaust INODES
    space ceiling  / 112 MB =  ~430 run trees exhaust SPACE

    INODES BIND ~7x BEFORE SPACE DOES.

That is exactly what lane w-alias observed and what the coordinator's first
message reported: `df -h /tmp` showed ~19 G free (space fine) while `df -i /tmp`
showed 1048576/1048576 (inodes gone).  **A free-SPACE check alone, which is what
this lane's brief specified, would have PASSED on that run and the gate would
still have gone red.**  Both resources are checked, separately, and the verdict
names which one.

## Accumulation or peak concurrency?  ACCUMULATION.

    peak concurrent gates observed on this box   3-6 lanes
    6 concurrent run trees                       ~100k inodes = 9.5% of the ceiling
    -> CONCURRENCY ALONE CANNOT EXHAUST INODES.

    w-reach recorded ~190 leftover trees the night it hit ENOSPC
    190 x 16.6k                                  = 3.15M inodes against a 1.05M ceiling
    -> ACCUMULATION ALONE IS SUFFICIENT, and overshoots the ceiling by 3x.

So reaping is the right mechanism and the diagnosis is not a guess.  Two
qualifications, stated rather than left implicit:

  * `/tmp` is SHARED.  162,189 inodes were in use at measurement time with only
    two gate trees present; `/tmp/vt` alone is 25,971 and `/tmp/claude` 18,743,
    neither of them this project's.  So the gate does not need to exhaust /tmp
    by itself to be the straw, and the floor check must be an absolute floor on
    what is free, never a budget over what the gate believes it owns.
  * The ~190 trees DRAINED between w-reach's run and this measurement, with no
    reboot.  Nothing in this repo reaped them, so something outside it did.  An
    unowned cleanup that happens sometimes is not a mechanism; it is why the red
    does not reproduce, and why the check has to be in the gate.

---

## CORRECTION, same day: the first arithmetic above UNDER-MEASURED the peak by 3x

The section above measured a run tree AFTER its gate finished.  A run tree while
its gate is RUNNING is much larger -- the 18 lanes' scratch, the sweep's corpus
and the mode cross's corpus all exist at once, and only the summary survives.

`gate.sh`'s own new low-water instrument is what caught this, on the first real
green run after it was added:

    /tmp free inodes at the run's start      853,187
    /tmp free inodes low-water in the run    802,937
    ------------------------------------------------
    ONE RUN'S PEAK DRAW                       50,250 inodes   (3.0x the 16,660
                                                               it leaves behind)
    ONE RUN'S PEAK DRAW                      ~307 MB          (2.7x the 112 MB)

Two consequences, and the first is the more important:

  * **The default inode floor was 50,000, which is EXACTLY ONE RUN'S PEAK.**  The
    preflight would have passed a filesystem with 50,000 free inodes and the run
    would then have exhausted the filesystem it had just certified -- a check
    that licenses the very failure it exists to prevent.  Raised to **150,000**,
    3x the measured peak.  This is the second defect the instrumentation found in
    its own fix, after the btrfs `0 0 0` one.

  * **The concurrency hypothesis is NOT refuted after all, and my first answer
    was too confident.**  At 50,250 inodes in flight, ~21 concurrent runs exhaust
    the inode table with no accumulation whatsoever, and 6 concurrent runs are
    ~300k = **29 %** of the ceiling, not the 9.5 % computed above.  On a
    twenty-lane night the transient term is the same order as the accumulated
    one.

**Revised diagnosis: BOTH mechanisms are real and they compose.**  With K runs in
flight and N finished trees left lying about,

    50,250 K  +  16,660 N  >=  1,048,576

so K=6 concurrent leaves room for only ~45 accumulated trees, and K=20 exhausts
it alone.  The fix needs both halves and has both: the **reaper** answers the
accumulated N, and the **preflight** answers the transient K -- which is the term
that produces the red that does not reproduce, because K falls back to zero
before anybody investigates.

The coordinator's second message proposed exactly this and asked that it be
determined rather than assumed.  Determined: the first measurement was taken on
the wrong object (a corpse, not a live run), and it is corrected here rather than
in place, because the wrong number and how it was caught is the useful part.
