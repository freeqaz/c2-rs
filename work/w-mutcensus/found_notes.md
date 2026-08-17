# w-mutcensus — found and not taken (draft for the rung's §9)

Ranked by what the next lane should read first.

## F1 — The suite row every rung quotes has NO `--require-graded`, and `gate.sh` has one *because of exactly this failure*

`scripts/gate.sh` grew `--require-graded` / `C2RS_GATE_REQUIRE_GRADED=1` for one
reason, stated in its own header: *"the thirteenth [absence-read-as-success]
being the one `--require-graded` above was written for."* Its design is quoted
because the same design applies here — *"a POSITIVE check on a COUNT, never an
enumeration of the ways a run can be empty"*, and the demand belongs to the
**caller**, so the portable lane (entitled to be empty) is unaffected.

**Nothing equivalent exists for `cargo test --workspace`.** Measured:
`grep -rn 'REQUIRE_TOOLCHAIN\|REQUIRE_GRADED' crates scripts` returns **8 hits,
all in `scripts/gate.sh`, none under `crates/`.** Yet the workspace suite row is
quoted as evidence in essentially every rung doc in `docs/rungs/`, and §7 of this
rung shows what that row is worth in an unprovisioned worktree: **1,648 / 0 / 42
with the differential at 84.17 s and 1,648 / 0 / 42 with it at 0.00 s.**

The fix is one function and needs no new dependency: a test that reads
`C2RS_REQUIRE_TOOLCHAIN` and **fails** when it is set and `Toolchain::locate()`
is `None`. Caller states its expectation; default behaviour does not move; the
portable lane still passes.

**NOT TAKEN, and the reason is structural rather than a shortage of time.** It
lands a test under `crates/`, and this lane's success criterion is a
**required-zero byte delta** on `crates fixtures scripts`. Those are the same
commit's two halves and cannot both happen — **which is precisely the conflict
`#3217` recorded one wave ago** for the missing `cflow_emitted_modeled_keys`
printer (*"a zero-delta rung and a new printed key are the same commit's two
halves"*). **That is now twice in two waves that the instrument a lane discovered
it needed could not be landed by the lane that discovered it.** The pattern, not
the item, is the finding: a characterization lane is structurally the wrong unit
for shipping the check it just proved necessary, and the repo has no unit that
is. Note this one is **cheaper than #3217's**: it adds no `gap-metric` key, so
the anchored-key count stays at 394 and only the byte-delta rule blocks it.

## F2 — The guard tests are per-KEY witness tests, so they pin ONE raise site per key and every sibling site is invisible

This is the mechanism behind the census's headline shape, and it is readable
directly in the guard that produced this lane's first RED.
`leaf_store.rs::every_bind_gate_fires_on_a_named_input` asserts **8 witnesses
over 5 distinct keys** — one input per key, each asserting
`bind_run_ops(...) == Err(THAT_KEY)`. It is a **key-reachability** test. It says
nothing about *which* of a key's raise sites produced it, so:

* `STORE_RUN_BIND_GROUP_SHAPE` has **four** raise sites —
  `leaf_store.rs:2254`, `:2257`, `:2285`, `:2456` (the last reached through the
  gate at `:2455`). The single witness (case 5, a 4-op F2 address-valued group
  where `parse_simple_gpr_run` matches exactly three) routes through **one** of
  them. Swap the key at that one and the witness fails; swap it at any sibling
  and nothing anywhere notices.

**Generalized:** a per-key witness suite guards `min(1, sites)` of each key's
raise sites, so a key with *k* raise sites contributes *k − 1* unguarded sites by
construction — no matter how carefully the witnesses were written. The families
this lane counted are exactly the multi-raise-site keys.

The fix is a helper that asserts **site** reachability rather than **key**
reachability — e.g. each raise site carries a distinct `#[cfg(test)]`-visible
discriminant, or the witness table is required to cover each site. Sizing for
one file: `leaf_store.rs`'s **9** enumerated sites sit under **5** keys.
NOT TAKEN for the same zero-delta reason as F1.

## F3 — The 1,227-site grammar class is unmeasured, and a SAMPLED census over it is a lane

Published in §2.1 with its count. The reason it is a separate lane is not only
budget (≈ 5 days serial) but that it is a **different guard class**: the key is
generated *from the blocking byte*, so a key-swap mutation does not exist, and a
removal mutation merely moves the parse to the next blocking byte. The right
instrument is a **stride sample** with a registered colour per sampled site —
`gate.sh`'s own sweep argument, *"the sample is a STRIDE across the sorted case
list, not a prefix"*, applies unchanged: a prefix over `blk(` sites would sit
entirely inside one parser.

## F4 — Nothing re-runs this census, so X/N goes stale on the next landed fence — and one already landed during the campaign

`enumerate.sh` and `mutants.py` live under `work/w-mutcensus/`. They are tracked,
so the census is *reproducible*, but nothing *re-runs* them, and §2.2 shows the
frame going stale inside this lane's own wall-clock: peer `w-fence163`'s
`d28326b4` adds a 20th fence-key constant and new deciding gates.

The cheap standing version is **not** a re-run of the campaign (56 suite runs is
not a gate row). It is a **count**: `enumerate.sh` already prints one line per
E1–E3 site, so a gate row that compares that count against a checked-in
expectation and fails when a fence lands without the census being re-scored would
turn "X/N is a fact about a commit" into a maintained invariant. **NOT TAKEN
twice over:** the byte-delta rule (F1), and a live seam — a separate lane is
wiring `debug_lane.sh` into `scripts/gate.sh` right now and this lane was
instructed not to edit either script.

## F5 — `STORE_RUN_BIND_CALL_TAIL_RETIRED` is a fence key with zero live raise sites

Enumerated and published in §2.1. Test-only since #1212's correction, so no
mutant is possible: there is nothing to mutate. Worth a row of its own because
the *inverse* of this lane's question — **a key with no fence** — is as invisible
to every instrument as a fence with no test, and this is the only one the frame
found. Whether it should be deleted or re-armed is a decision, not a measurement,
and it is not this lane's to make.
