# w-ladders — PREREGISTRATION

Written at base `851938df`, in `.claude/worktrees/w-ladders`, **before any ladder
was climbed on this tree**. The base 878-TU scan had been run (to learn who the
sixteen ARE — the frontier's membership is not a prediction, it is an input), and
nothing else. Everything below is a claim about what the two instruments will
report, registered so a hit and a miss are distinguishable afterwards.

## 0. The population, which is an input and not a prediction

`work/w-ladders/scan_base.log` line 930 — **FRONTIER 16**:

    src/Main.cpp                              src/system/rndobj/wordwrap.cpp
    src/system/math/Primes.cpp                src/system/utl/Pool.cpp
    src/xdk/LIBCMT/osfinfo.cpp                src/xdk/nuispeech/mmio.cpp
    src/xdk/LIBCMT/undname.cpp                src/system/synth_xbox/IPP_basicmath_xbox.cpp
    src/xdk/LIBCMT/vswprnc.cpp                src/system/utl/EncryptXTEA.cpp
    src/xdk/xjson/jsonwriter.cpp              src/keygen_xbox.cpp
    src/xdk/xlrc/xlrcimpl.cpp                 src/system/negate_test.cpp
    src/system/synth_xbox/Biquad.cpp          src/xdk/LIBCMT/vsnprnc.cpp

**This is `w-5c`'s seventeen less `src/xdk/nuispeech/xboxheap.cpp`.** Registered
here because two published ladder tables (`w-4c`'s "16", `w-5c`'s "17") are over
*different* sixteens and quoting either at this master is an error of membership
before it is an error of number.

## 1. How many of the sixteen are CLIMBABLE, per instrument

"Climbable" is defined here as **reaching a terminal** — `READER-CLEAR`, or the
committed poison/`TAIL` terminal — as opposed to `EXIT`/`STUCK`/`SCANFAIL`, where
the instrument ran out of lift and the row is a lower bound.

| | predicted complete | predicted LIMIT |
|---|---:|---:|
| **U** — unhatched (`C2RS_SINK_CHAIN` only) | **5 of 16** | 11 |
| **H** — hatched (`+ W_FRONT3_LIFT`) | **8 of 16** | 8 |

Named, so a partial hit is readable:

* **U complete**: `Primes`, `undname`, `vswprnc`, `vsnprnc`, `mmio`.
* **H complete**: those five **+ `osfinfo`, `jsonwriter`, `negate_test`**.
* **`keygen_xbox` is predicted to PANIC or SCANFAIL under H** and to be an
  `EXIT:noform-0x13` under U — `w-front3`'s own run died at `calls.rs:71` on it
  and `w-5c` reached 20 only with `call-arg-outer-formal` skip-listed.

Derived from `w-5c`'s `ladder{,h}_tip.txt` minus `xboxheap`, i.e. **inherited**,
which is exactly the sin this lane exists to correct — so it is registered as a
prediction and will be re-measured rather than copied into the deliverable.

## 2. The 29-rung gap (unhatched 154 → hatched 183)

**Predicted decomposition: 8 hatch rungs + 21 sink rungs that lie BELOW a
hatch-withheld exit.** Per TU (from `w-5c`'s two files, arithmetic done here):

    Main.cpp    +3 = 1 sink + 2 hatch      negate_test +12 = 11 sink + 1 hatch
    osfinfo     +9 = 8 sink + 1 hatch      vsnprnc      +1 =  0 sink + 1 hatch
    jsonwriter  +2 = 1 sink + 1 hatch      keygen_xbox  +2 =  0 sink + 2 hatch
                                           ------------------------------------
                                           total +29 = 21 sink + 8 hatch

**Predicted verdict: REAL REACH, not double-counting and not a scaffold** — the
unhatched run *exits* at the withheld clause and structurally never sees the 21,
and `net` already subtracts `SCAFFOLD`.

**Registered direction of error: I expect to be WRONG in the direction of the 29
being an OVER-count.** The named reason, registered before measuring:
`ladder.py`'s **first pass grants a sink token without checking the blocker set
moved.** Only the *second* (tail-opcode) pass runs a trial scan and discards an
inert candidate into `tail_inert`. So a first-pass grant that changes nothing is
counted as a rung whenever some *other* key in the same round advances — which is
board **#1285**'s defect (`Pool.cpp`: "round 5 measured NOTHING") in a place
nobody has looked. **Predicted inert first-pass grants across the 32 rows: 0–4.**
If it is 0, the 29 stands as reach; every one found comes off it.

## 3. Can `ladder.py` see the CODEGEN column at all?

**Predicted: NO, on 15 of 16 rows, and the mechanism is not the one board #1417
names.** #1417 says the four READER-CLEAR TUs' 47 rungs are all poisoned sink
rungs. I predict the stronger statement:

* the driver **does record** `emit_blockers` and `fn_gate_refusals` every round
  (`rounds[*].emit` / `.gate`, surfaced as `final_emit` / `final_gate`), so it is
  not blind by omission;
* but **any row carrying ≥ 1 sink rung is poisoned**, so its codegen column is
  empty-or-meaningless *by construction*, and **every one of the sixteen carries
  ≥ 1 sink rung except `xlrcimpl` and `Main.cpp`** (0 sink under U);
* **predicted rows with a non-empty, trustworthy `final_emit`: 0 or 1.**

If that holds, the plain sentence the brief asks for is: **`ladder.py` cannot
price the codegen column on ANY frontier TU that needs a sink rung to get there,
which is 14 of 16 — and the four READER-CLEARs are a special case of that, not
the whole of it.**

## 4. The null-lift control, and its discriminating-cell count

A control that cannot go red is not a control. The grid: for every one of the 16
TUs, scan three ways — **no lift**, **a NULL sink lift** (a token for an opcode
this tree has pinned but which cannot appear here), **a NULL hatch lift** (a
clause name no `front3_lift` call site tests) — and compare `fn_blockers`.

* A cell **discriminates** iff the REAL round-1 lift moves the blocker set on
  that TU **and** both null lifts leave it byte-identical.
* **Predicted discriminating cells: 12–16 of 16.** `w-one`'s grid was
  near-vacuous at ≤ 1 and said so; this one is over TUs that all have a live
  first blocker, so it should not be. **If it comes out ≤ 4 I will print the
  count and decline to bank the control**, not adjust the grid.

## 5. Unreachable rungs

`w-mrslot` found one input nothing in the crate can produce; `w-one` screened 218
keys, witnessed 34, and **printed "184 UNREPORTED is SHADOWING, not
unreachability"** rather than promoting them.

* The candidates here are the **`noform-0xNN` terminals** that bound 12 of 17 in
  #1289 — `0xBD`, `0x00`, `0x4C`, `0x1C`, `0x11`, `0x10`, `0x13`, `0x5C`.
* **Predicted genuinely unreachable: 0 of them.** Every one is a byte the
  *instrument's* width table has not pinned, which is a statement about
  `chain_skip_form`, **not** about the stream — #1289 says so in its own words
  and `w-4c` then pinned `0x4C` and extended six ladders. An unpinned width is a
  MISSING ENTRY, and a missing entry is not an unreachable rung.
* **Predicted comment-claims of unreachability that the scan refutes: ≥ 1.**
  `w-one` found one witnessed 4,973 times over 829 TUs; `w-clear` found a doc
  comment asserting a refusal that was not there. Registered expectation: the
  base rate of that defect in this file set is not zero.

## 6. What I expect to SHIP

**Nothing in `crates/`.** Repairs, if any, land in `work/w-front3/ladder.py`
(the unverified first-pass grant, §2) and in `work/w-hatch/hatch_red.py` (an arm
for whatever new behaviour that adds). **Declining with a measured table is the
expected outcome and is registered as such** — a conversion here would be a
surprise, and if one appears I owe an explanation of why sixteen priced rows
missed it.
