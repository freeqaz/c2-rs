# PREREG — lane `w-dbgassert`

Frozen and committed **before the first build**. Base: `da3ed0d3`.
Question: board **#3074** — the two `debug_assert_eq!`s at
`crates/c2-core/src/coff/writer.rs:658` are FALSE in a debug build, and every
standing instrument is `--release`. **Is the ASSERTION wrong, or is the WRITER
wrong?**

What I have read before freezing this (and what it does *not* settle): board
row #3074 in full, `docs/rungs/2026-08-13-ircond.md` §5/§6 `AD-c`, the two
assertion sites (`writer.rs:658` and `writer.rs:1170`), and
`coff/container.rs::layout_sections` with its doc comment. I have **not** built,
run, or asked the oracle anything. The code read is why P1 below is high rather
than even; it is not evidence, and the oracle has refuted a confident read of
this file before (`OBJ_DYNINIT_SHAPE.md` §1 P8, the `SizeOfRawData`/`VirtualSize`
inversion — the natural guess was backwards).

## P1 — assertion-wrong vs writer-wrong

| outcome | probability |
|---|---|
| **assertion wrong** — the emitted bytes are byte-exact against real c2; the assertion encodes a stale invariant that predates `.bss` in this writer | **0.88** |
| **writer wrong** — the port emits bytes that differ from c2's for the affected input; a LIVE mismatch, which stops this lane and becomes an alarm | **0.10** |
| neither cleanly (both need work, or the input is unreachable from any real IL so no oracle answer exists) | 0.02 |

The mechanism I expect: `layout_sections` pushes `ptrs[i] = 0` for a section
with `uninit_size.is_some()` — a `.bss` has `PointerToRawData = 0` by design and
that design is documented as **measured against real c2**. The assertion
`b.0.len() == ptrs[i]` does not except that case. Predicted values at the two
firing sites: `left` is the live file cursor (496 / 536), `right` is `0` because
the section is the uninitialized one. **496 and 536 should each equal the file
offset the NEXT initialized section actually gets** — i.e. the cursor is
undisturbed. I register that as the discriminator: if the cursor is undisturbed
and the following section's `ptrs` is the same number, the writer is right.

## P2 — release vs debug bytes

**Identical, P = 0.97.** `debug_assert*` is the only `cfg(debug_assertions)`
dependence I expect in this path, and it has no side effect on `b`. Registered
as a positive check, not an assumption: I will emit the affected obj from a
`--release` build and from a `--debug` build (with the assertion neutralized so
the debug build can reach the end) and compare the two files byte-for-byte.
Predicted: 0 differing bytes.

## P3 — sibling unreachable `debug_assert`s

Predicted **total** `debug_assert*` occurrences under `crates/`: **70**,
80 % interval **40–130**.

Predicted fraction reachable by any **standing instrument** (`gate.sh`,
`expr_sweep.sh`, `mode_cross.sh`, the 878-TU scan, `status.sh`, the workspace
test row): **0 of them — 100 % unreachable**, P = 0.95. Every one of those runs
`--release`. This is the policy finding and I expect it to be exact rather than
approximate.

Predicted fraction that *would* be executed by a debug-profile
`cargo test --workspace` run: **60 %**, interval 30–85 %.

Predicted number of siblings that are **false** (fire) beyond the two already
known: **1**, interval 0–4.

## P4 — deltas (the repair, whatever it turns out to be)

| quantity | base | predicted tip |
|---|---|---|
| fixtures `match` (gate) | as gate prints | **+0** |
| `mismatch` | 0 | **0** — any other value is an alarm and stops the lane |
| census | as `c2rs census` prints | **+0** |
| 878-TU workload `match` | as scanned | **+0** |
| emitted bytes, any lane | — | **required zero delta** |
| `cargo test --workspace --release` | **1,548 passed / 42 targets** | **1,550 / 42** (+2 tests, +0 targets) |
| `cargo test --workspace` (debug) at base | **RED**, exactly **2** failures | — |
| `cargo test --workspace` (debug) at tip | — | **GREEN** |

The +2 is a test asserting the `.bss` layout invariant in its *correct* form
(`ptrs[i] == 0` iff uninitialized, and the cursor unmoved), which is the
assertion's true content. If the repair turns out to be writer-side, this row
is void and the lane reports an alarm instead.

## P5 — outcome word

Predicted `Outcome:` = **`instrument`** (P = 0.80): a corrected measuring
instrument plus a proposal for the blindness. `FAILED` if I cannot reproduce
positively (P = 0.05). `built`/alarm if writer-wrong (P = 0.10). `declined`
(P = 0.05).

## P6 — the blindness proposal

I predict I will propose **one debug-profile lane in `scripts/gate.sh`**, and I
predict its price is **under 3 minutes wall-clock** added to a gate run
(interval 1–15 min), because the debug lane needs to compile the workspace once
in the default profile and run only the *unit* rows, not the toolchain rows.
I register in advance that I will **propose and price it, not impose it** — the
gate is shared and a peer lane is live.

## What would make me wrong in the words the brief asks for

If the oracle says the emitted obj for the affected input is **not** byte-exact,
then **I was wrong: the writer is wrong, not the assertion**, and I will say so
in those words at the top of the rung doc and stop for the alarm.
