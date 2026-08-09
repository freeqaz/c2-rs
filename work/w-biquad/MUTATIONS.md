# w-biquad — the must-fail mutations, RUN

`_neg` cells have been inert or confounded in six of the last nine lanes, and
`w-blockir` #2305 is the lesson: a fence is proved live by **running** a mutation
that should break it, not by reasoning that it would. Each row below was applied
to shipping code, graded by real `c2.dll` under wibo through
`scripts/mode_lane.sh /O1`, and reverted.

The instrument is the whole `/O1` fixture lane, so a mutation that broke
something *else* would show up too. The baseline is
`LANE-RESULT PASS … graded=333 total=333 match=163 mismatch=0`.

| # | mutation | what it says the port knows | result |
|---|---|---|---:|
| **M1** | delete `plan_labels`' `+2` per newly pooled FP constant | `LABEL_COUNTER` §1.1's fourth surcharge row, and that it is observable at all | **FAIL**, `match=162 mismatch=1` |
| **M2** | drop B′-RULE's flip — load the divisor first in **every** division | `WB_CHOOSER_FINDINGS` §4.1: the flip is on the LAST statement of the run and nowhere else | **FAIL**, `match=162 mismatch=1` |
| **M3** | B-RULE's rival reading — hoist **both** `lis` into the entry block, taking r11 then r10 | §3.3's dominator rule, and cell B1's refutation of *"the pooled `lis` is the function's first word"* | **FAIL**, `match=162 mismatch=1` |
| **M4** | park `this` in **r11** instead of r10 | §2.3's *"r11 if the value does not cross a call, r10 if it does"* | **FAIL**, `match=162 mismatch=1` |

**Four for four, and each is a `mismatch` rather than a refusal** — the port
emits bytes for `wbiquad_fp_store_diamond.cpp` and real `c2.dll` disagrees with
them. A mutation that produced `NotImplemented` would prove only that the class
had stopped accepting the body; a `mismatch` proves the fence is carrying a byte.

**M3 is the one worth reading twice.** Its rival is not a strawman: it is exactly
what `Biquad.cpp`'s own obj invites, because both readings put a `lis` at word 0
and they disagree only at word 4. `WB_CHOOSER_FINDINGS`' cell **B1** is what
separates them, and this lane depended on that cell without compiling it.

## Reproduction

```sh
# apply one mutation, then:
cargo build --release -p c2-harness
scripts/mode_lane.sh /O1
# revert:
git checkout crates/
```
