# w-depth — PREREG: parse-chain DEPTH as a frontier selector

    Tag:       w-depth
    Slug:      w-depth-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg. It admits no shape, ships no lowering,
               and the instrument it registers is measurement-only by
               construction (no `IlOp`, poisoned at the end of the walk), so
               there is nothing an obj-graded fixture could grade.
    Census:    not measured yet by design. The baseline this lane registers is
               in §1 and was collected before a line of the instrument existed.
    Lane:      w-depth, worktree `wt-w-depth` off master **`dad6257`**.
    Record:    this file. The findings record is
               `docs/rungs/_2026-08-05-w-depth.md`.

---

## 0. What this lane is for

Seven lanes have established that no existing instrument predicts which factor-D
work converts a TU. The standing explanation, board **#622** (w-frame2), is that
**a blocker key is the label on the FIRST refusal and the real cost is the depth
of the chain behind it** — promoting the only experimental sink moves
`xboxheap`'s blocker `expr-op-0x27` to `expr-op-0x32`, a token nothing models.
Nobody has measured that depth.

This lane builds the instrument that measures it, ranks the 18 frontier TUs by
it, and scores the ranking against the one conversion the project has.

---

## 1. Baseline, collected at `dad6257` BEFORE the instrument existed

`./target/release/c2rs gap --list work/dc3-workload/files.txt --flags-file
work/dc3-workload/flags.txt --cwd ../../../../dc3-decomp --jobs 8`, cache
`work/capture-cache` of the main repo:

```
match 9 · mismatch 0 · codegen-gap 0 · vocab-gap 862 · capture-fail 7
A 28 (LO 27) · B 338 · C 169 · D 9 · E 2   of 871 graded
A∧B∧C 27 · A∧B∧C∧D 7 · FRONTIER 18 · frontier-if-A 140
byte fraction: 15 of 18 frontier TUs at EXACTLY 0.0 %
```

Every digit matches the brief's registered baseline. Recorded in
`work/w-depth/gap-base.txt`, jsonl in `work/w-depth/scan-base.jsonl`.

---

## 2. A PREMISE OF THE BRIEF IS ALREADY REFUTED, and it is registered here rather than discovered later

The brief says of the three existing sinks:

> They are **measurement-only** — they push no `IlOp` and poison any walk that
> reaches the end, so **acceptance cannot move.**

**That is true of two of the three and FALSE of the third.**
`crates/c2-il/src/func/body/expr.rs`, the `0x27` arm under
`off_add_sink_enabled()`:

```rust
0x27 if off_add_sink_enabled() => {
    *p += 1;
    match read_type(seg, *p) { Some((_, _, _, w)) => *p += w, None => … }
    ops.push(IlOp::Add);          // <-- pushes an IlOp
}
```

There is no `saw_off_add_sink` flag and no poison arm for it. It is a **real
widening behind an environment variable**, which is exactly why board **#403**
records that turning it on takes `cargo test --workspace --release` to
**16 targets / 754 passed / 2 failed**. `C2RS_SINK_REL` and `C2RS_SINK_BRANCH`
are the measurement-only ones; `C2RS_SINK_OFF_ADD_ARG` is not.

**Consequence for this lane, registered now:** `C2RS_SINK_OFF_ADD_ARG` cannot be
used as a chain step, because its successor key is measured under a parser that
also *accepts* differently. The instrument this lane builds must model `0x27`
itself, under the poison discipline, and **the `0x27` successor must be
re-derived rather than inherited from #622.**

---

## 3. What the instrument is, and what its number is OVER

`C2RS_SINK_CHAIN=<comma-separated sink tokens>` — a data-driven,
**poisoning** sink in `parse_expr`. Each token names one refusal class to close.
Rules, all of which are the properties `C2RS_SINK_REL` and `C2RS_SINK_BRANCH`
already have and which this lane preserves:

1. **No `IlOp` is ever pushed** by a chain-sink arm.
2. **Any walk that reaches the end of the expression having used one refuses**,
   under `expr-chain-sink-poison`. Decoding is not accepting.
3. An opcode listed with **no declared skip form refuses under
   `expr-chain-sink-noform-0xNN`** rather than guessing a payload width. A
   guessed width desynchronises the stream and manufactures a fictitious
   successor, which is the one way this instrument could lie.
4. OFF and free on every gate lane and every default scan. The gate is re-run
   with it OFF and must reproduce §1 exactly.

**Procedure.** For a TU: scan, read the blocker keys, add a sink token for each,
re-scan, repeat, until either every blocked function reaches the poison (the
body's expression walked end to end through sunk tokens only), or a blocker
appears that is **outside `parse_expr`** (a `body-*`, `call-*`, `assign-*`,
`cflow-*`, `param-*` key), or a **declared bound of 12 rounds** is hit.

**What the number is over — stated explicitly, because the brief requires it.**

> **DEPTH(TU) = the number of distinct expression-layer refusal classes that must
> be closed before every blocked function in that TU walks its expression to the
> end.** It is a count over the TU's *blocked functions' expression streams*, not
> over "facts to implement", and it is a **LOWER BOUND on the work**: sinking a
> token costs nothing and implementing it costs a rung, so DEPTH = 3 means "at
> least three constructs", never "three easy things".

**Can closing operator *k* re-open something *k−1* had cleared?** Registered
answer, to be checked rather than asserted: **no, for the sunk set, and yes for
the reported population.** A chain-sink arm only advances `*p`; it sets no
`saw_*` flag other than the poison and pushes no op, so a prefix that parsed
before still parses. But the *set of functions reporting each key* is
substituted, not reduced — the w-cmp finding — so a key's count going to zero is
not that key being worth anything. The chain is nevertheless a **minimal set for
a single body**, because at each step the reported key is necessary for that body
to clear: this is an intersection over one body, not a marginal over many, which
is the defect in greedy re-ranking that #150 and w-cmp identified.

**What the instrument silently holds fixed — asked, per the trap.** (a) One flag
profile, the workload's own `/GR /O1 /Oi /EHsc`; a `/Ox` chain is not measured
and is not claimed. (b) Only `parse_expr`. Blockers raised in `mcall`,
`control_flow`, `assign`, `ctor_dtor` and the formals/plumbing walk are
**outside the instrument** and are reported as EXIT, not as depth. (c) The
operand **type** gate: `expr-load-type-*` / `expr-lit-type-*` are not opcodes and
need their own sink token, which is registered as `type` and is a *different kind*
of closure from an opcode.

---

## 4. Predictions — registered before a line of the instrument exists

| # | prediction |
|---|---|
| **R1** | The baseline in §1 reproduces exactly with the instrument compiled in and OFF: match 9, FRONTIER 18, A/B/C/D/E 28/338/169/9/2, `A∧B∧C` 27. |
| **R2** | **No frontier TU has DEPTH 1.** #622 measured `xboxheap` at ≥ 2; I predict the whole frontier is ≥ 2, because a TU whose expression layer was one token from clearing would already have been converted by one of the seven lanes that looked for one. |
| **R3** | **The head is `src/xdk/LIBCMT/undname.cpp` or `src/xdk/LIBCMT/vswprnc.cpp`**, at DEPTH 2 or 3 — one blocked function, one relational key, and the relational family is the one whose successor structure w-cmp already mapped (88.5 % go straight to a branch key, so the chain should be `cmp → branch → end`). |
| **R4** | **DEPTH is BOUNDED for at least 12 of the 18** — i.e. at least two thirds of the frontier clears the expression layer inside the 12-round bound rather than running away or exiting. |
| **R5** | **At least 3 of the 18 EXIT the expression layer** rather than terminating, and `src/Main.cpp` (`param-width-undetermined:mid`) and `src/xdk/xlrc/xlrcimpl.cpp` (`assign-rhs-call-0x26`) are two of them — their baseline keys are already not `expr-*`. |
| **R6** | **DEPTH RETRODICTS `xboxmem`.** Re-run on master as it stood before `w-tu1` (`6dcb3f4`), DEPTH puts `xboxmem.cpp` in the **top 3 of 19**. I do **not** predict first — the byte-fraction ranker already earned first, and a second instrument that also lands it first would more likely be measuring the same thing than a new thing. |
| **R7** | **DEPTH separates `mmio` and `xboxheap` from `xboxmem` by a clear margin**: both at DEPTH ≥ 4, i.e. at least twice `xboxmem`'s. |
| **R8** | **DEPTH DISAGREES with the byte-fraction ranking at the head.** Byte fraction says `mmio` (16.8 %) and REMAIN says `Primes.cpp` (64 B). I predict DEPTH says neither. |
| **R9** | **`xboxheap`'s successor after `0x27` is NOT `expr-op-0x32`** as board #622 reports, because #622 measured it through `C2RS_SINK_OFF_ADD_ARG`, which is a widening and not a sink (§2). I predict the successor under a poisoning `0x27` sink is the same byte, `0x32` — i.e. #622's *number* survives its *method* being wrong — but this is registered as a check, not as an assumption, and the alternative outcome is the more interesting one. |
| **R10** | **TU match ends at 9.** This lane converts nothing. It is an instrument lane and it builds no codegen. |

**Where I expect to be wrong:** R3 is the one I would bet against myself on. The
relational chain is the *best-mapped* one, which is exactly why the successor may
be long — w-cmp measured 2,920 of 3,298 relationals going straight to a branch
key, and a branch key at `Cflow` substitutes overwhelmingly to `expr-op-0x53`,
which w-brfalse had to add a whole third level for. A chain that has to pass
through punctuation may well be *deeper*, not shallower, than a chain that starts
at a real operator.

---

## 5. Bookkeeping

* Board rows taken: **#660**–**#669**.
* Scratch: `work/w-depth/`. Never `/tmp`, never `~/tmp`.
* Verification bar, to be run with the sink OFF: `cargo test --workspace
  --release` (849 / 0 / 27 targets), `scripts/gate.sh --jobs 6` (18/18,
  **4,536 verdicts** registered here before the run; sweep 96 ungraded, cross
  388 ungraded, 0 mismatch), `scripts/status.sh --check`, `board_audit.sh`.
* **This ranker is an instrument, never a gate.**
