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
