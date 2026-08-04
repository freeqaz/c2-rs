# Pre-registration — lane `w-repro`, is IL capture reproducible?

Committed **before** any measurement in this lane. The thing under test is not a
model of the compiler; it is the **instrument** every other lane's numbers are
joined on. So the failure mode to guard against is the comfortable one: running
until the two files agree and calling that reproducibility.

## The observation that opened the lane

`work/w-bss2/glcensus.jsonl` — the front-end-only (`/Bd /d2nop`) `.gl` census of
the 871 capturable workload TUs — was produced twice ~30 min apart and the two
files differ. Both are preserved:

| file | bytes | produced |
|---|---:|---|
| `work/w-repro/glcensus.20260804-0850.jsonl` | 1,861,636 | lane w-bss2 |
| `work/w-repro/glcensus.20260804-0920.jsonl` | 1,861,312 | `scripts/regen_census.sh --gl` |

Two differences were reported, and they are different kinds of thing:

* **(a)** `gid` shifted by a constant **+4** on the (three) records eyeballed.
* **(b)** exactly one TU's record set differs structurally —
  `src/system/rndobj/Anim.cpp`: kept-record indices `4,5` → `6,7`, one symbol
  present in one file and absent from the other, and the line got **shorter**
  while gaining a record.

## What I checked before writing this, and what it rules out

Disclosed so the predictions below are not secretly post-hoc:

* `work/w-bss2/glcensus.py` reaches the front end through `work/w-bss2/cap.py`,
  which shells out to `wibo cl.exe /Bd /d2nop …` with `TMP`/`TEMP` pointed at a
  **fresh per-call `tempfile.mkdtemp`**. It never consults `work/capture-cache`
  and never runs `c2rs`. **The capture cache is therefore not a candidate
  mechanism for this observation** — it is not on the path.
* `keep[].i` is an index into the *unfiltered* ordered list of `.gl` data-global
  records; which records are *kept* is filtered by `wanted_names()`, whose input
  is `work/w-bss/census/sections.jsonl`. If that file had changed between the two
  runs, (b) would follow with no `.gl` difference at all. It has **mtime 07:21**,
  before both runs (08:50, 09:20), and is clean against `HEAD` (`50afce0`).
  **Hypothesis H-filter is dead before the lane starts** — (b) has to be a `.gl`
  difference.

Registered as a **known-answer control** anyway, because the above is an argument
and not a measurement:

> **C0.** Re-running the whole `one()` computation over a **stored** `.gl` blob,
> with today's `sections.jsonl`, reproduces that blob's record byte-for-byte.
> Registered: **exact, 100 %**. If C0 fails, nothing else in this lane is
> interpretable and every number below is reported as "not measurable".

## The predictions

Everything is at the workload's own flags
(`work/dc3-workload/flags.txt`: `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` +
the project `/I` set), cwd `../dc3-decomp`, with `/Bd /d2nop` prepended.

### P1 — is `Anim.cpp` reproducibly irreproducible?

> **P1.** `src/system/rndobj/Anim.cpp` captured **N = 24 serially** (one at a
> time, no concurrency) yields **exactly 2** distinct `.gl` SHA-256 values, in a
> roughly balanced split.
>
> *Rival P1′ (concurrency-only):* **1** distinct hash serially. The front end is
> deterministic in isolation and the difference requires load — which relocates
> the mechanism to `glcensus.py`'s `ThreadPoolExecutor`, its `mkdtemp`s, or wibo.
> P1′ is scored by a second run of **N = 24 under 14-way concurrent load**; if
> serial gives 1 hash and loaded gives ≥2, P1′ wins outright.
>
> *Rival P1″ (drift):* **> 2** distinct hashes — not bistable, so no two-state
> story is available and the honest deliverable is a rate, not a mechanism.

### P2 — does anything else do it, and at what rate?

> **P2.** A **third** whole-census run, diffed against both preserved files with
> `gid` masked out, finds the unstable set confined to **≤ 4 of 871 TUs**
> (≤ 0.46 %), and `Anim.cpp` is in it.
>
> *Rival P2′:* the unstable set is large (> 20 TUs) and the 0850/0920 pair
> under-counted it because two samples of a many-state process mostly agree by
> luck. If P2′ holds, **`grade.py`'s and `r56.py`'s inputs are unstable in bulk**
> and that is the lane's headline, not a footnote.
>
> A **`jobs=1` control census** (serial, same 871 TUs) is run as the arm that
> separates "the front end is nondeterministic" from "our harness races". If
> `jobs=1` is stable across two runs and `jobs=14` is not, the mechanism is in
> the harness and I say so; if `jobs=1` is *also* unstable, it is the front end.

### P3 — is `gid` safe to sort by?

`work/w-bss2/grade.py:96` and `work/w-bss2/r56.py:124` both sort records by
`(gid, i)`. A uniform shift is harmless; a non-uniform one silently reorders.

> **P3.** Over **every** record of **every** TU present in both files (not the
> three eyeballed), `gid_new − gid_old` is the **same constant, +4**, for
> **100 %** of records, and the gid sort order is therefore identical
> — **871/871 TUs order-preserving**.
>
> *Rival P3′:* the shift is not constant — per-TU, per-record, or absent on some
> records. Registered consequence if P3′ wins: **`grade.py`'s and `r56.py`'s
> published results are in question and the report says exactly that**, in those
> words, rather than reporting a re-grade that happened to agree.

### P4 — do w-bss2's landed numbers survive?

`docs/OBJ_DATA_BSS_SHAPE.md` carries **`.bss` 110/117** and **`.data` 68/68**,
produced by `grade.py` against the 0850 file.

> **P4.** `grade.py` re-run against **each** capture file reproduces
> **110/117** and **68/68** on **all** of them, unchanged.
>
> *Rival P4′:* they move. Registered in advance: **if they move by even one, that
> is reported as a defect in a landed document**, with both numbers printed, and
> is not smoothed over, re-fitted, or explained away by picking the file that
> agrees. The incumbent is the landed pair, not "whatever the majority of runs
> say".

## Priced decline clause

If **P1′ wins** (serial capture is deterministic) **and** **P2 holds** (≤ 4 TUs
unstable) **and** **P4 holds** on all files, then after **at most two further
probes** — one varying only concurrency, one varying only the temp-dir
arrangement — I **decline** to isolate the mechanism further and deliver a
characterized boundary instead.

**The price, stated now so it is not discovered later:** the report will then say
the mechanism is **unidentified**, will name `gid`'s +4 base offset as
**unexplained**, and will state that any future measurement joined on `.gl`
record identity inherits an uncharacterized instability at the measured rate.
That is a worse deliverable than a mechanism and I am accepting it rather than
fitting a story to one sample.

## What this lane will not do

* Not touch `crates/` — lane `w-cfgimpl` owns `crates/c2-il/` and
  `crates/c2-core/`. If the mechanism is in the harness, it is characterized and
  a fix is **proposed**, not made.
* Not "fix" `glcensus.jsonl` by regenerating until it agrees with itself.
* Not glob or walk `work/capture-cache` — it is not on the path (above), and the
  glob is what OOM-killed this box twice.
