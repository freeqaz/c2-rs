# W-ADOPT — pre-registration

**Lane**: teach the GATE (`gl::gl_defined_names`, and therefore
`bind::Bindings::per_record` and `IlBundle::functions`) the `26` name separator
that `gl::gl_symbol_runs_all_separators` already reads, behind the full
differential.

**Base**: `a03b8c1` (`= 47318dd` + the §9.20 doc landing). Verified with
`git log -1` as the first command of the session; the handed worktree was on
`4ea415a`, **635 commits behind master**, and was reset to `master` before
anything else.

**Committed before the first measurement.** Nothing in this file was written
after looking at a base number that is not already published in
`docs/ROADMAP.md`.

## Declared bias

* **Borrowed.** §9.20 was read in full before estimating, including its own
  recommendation #1 ("teach the GATE this reader"). Its ceiling numbers
  (324 / 420) are instrument-side and I expect them **unmoved** by this lane —
  that is a prediction, not an assumption, and E3 scores it.
* **Pessimistic on the direction of the swap.** Widening what a gate *sees*
  widens both what it can bind and what it must account for, and the accounting
  rule in `IlBundle::functions` refuses a TU for a single unaccounted `.gl`
  name. I expect the accepted class to move **down** before it moves up.
* **Structural.** I read `gl_defined_names`, `Bindings::per_record` and
  `IlBundle::functions` before estimating, so E5/E6 are informed by the code
  path rather than by a number.

## The control is the INCUMBENT, named

Not a threshold. The thing this change must beat or tie:

> **The NUL-only gate reader, as it stands at `a03b8c1`: TU match = 6,
> mismatch = 0, `scripts/gate.sh` 12/12 PASS, and whatever accepted class it
> has.**

**Decline floor, registered in advance and binding**: if the tip's TU match is
**< 6**, or the tip's mismatch count is **> 0**, or `gate.sh` loses a lane, the
change is **DECLINED and not merged**, regardless of what it does to any
ceiling, residue or accounting number. A ceiling gain does not buy a match.
There is no interval in which losing one of the 6 is a pass.

Second binding clause, because a residue can move while the thing it proxies
does not (#144, §9.20.3): **`records` and `record_offsets` from
`EmitBinding` must be byte-identical at base and tip.** This lane touches the
gate reader only; if the framing count moves, the change did something it was
not supposed to and is declined pending explanation.

## Registered estimates

| # | claim | est | interval |
|---|---|---|---|
| E1 | **the 6 byte-exact TUs, by name, all still match at tip** | YES, 6 of 6 | — (no interval; this is the floor) |
| E2 | realized **TU match** at tip, of 878 | 6 | [6, 9] |
| E3 | emit-set MODEL ceiling re-measured **at my base** (today / repaired) | 324 / 420 | [300, 340] / [400, 440] |
| E3b | the ceiling **moves** when the gate adopts the reader | NO — it is measured on `EmitBinding`, not on the gate | — |
| E4 | TUs whose `IlBundle::functions()` goes `None → Some` at tip | 5 | [0, 200] |
| E5 | TUs whose `IlBundle::functions()` goes `Some → None` at tip (accepted class LOST) | 30 | [0, 400] |
| E6 | the dominant cause of E5 is the **unclaimed-name accounting rule** in `IlBundle::functions`, not the name-distance bound | YES | — |
| E7 | a record binds to a **different** name at tip on at least one workload TU | YES | — |
| E8 | `EmitBinding` arity: `records` / `record_offsets` identical base→tip | 1,515,160 / 1,515,160, 0 breaks | — |
| E9 | agreement, the 158 listing-adjudicated records | 154 at base, 154 at tip | [150, 158] |
| E10 | a **dedicated probe** fixture carrying a `26`-separated name is required, because the 878-TU scan cannot see the defect (#149) | YES — and the base gate's verdict on it is a **refusal**, not a mismatch | — |
| E11 | the probe shows the tip **fails closed** (`NotImplemented`) rather than emitting a wrong COMDAT set | YES | — |
| E12 | `gate.sh --jobs 6` at tip | 12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, 0 mismatch | — |
| E13 | fixture-verdicts graded by `gate.sh` at tip | 2,520 | [2,400, 2,700] |
| E14 | `cargo test --workspace --release` **target count** at base and tip | 24 / 24 | — |
| E15 | TUs **converted** by this lane (match gained) | 0 | [0, 3] |
| E16 | at least one run visible only under the `26` scanner terminates on a byte that is **not** `00`, so the `linkage_needs_a_directive` read at `name_nul + 3` would be reading a field that is not the linkage byte | YES | — |

## Refuters, stated in advance

* **E1 refuter**: any of the 6 named TUs leaving `match` at tip refutes the lane
  and the change is reverted. Reported by name, individually, before anything
  else.
* **E2 refuter**: TU match at tip below 6.
* **E8 refuter**: `records` moving at all.
* **E11 refuter**: the probe producing `mismatch` at tip. A refusal is a pass; a
  wrong COMDAT set is the one outcome that outranks every gain in this brief.
* **E10 refuter (the anti-refuter, per #145)**: if the probe reads green at base
  *and* at tip *and* under a deliberately broken emit set, it cannot see the
  defect it exists for and does not count as evidence. The probe must be shown
  to go red under a mutation.

## What is out of scope

* `crates/c2-harness/src/gap.rs` — lane **w-reach** is live in it. Reachability
  numbers are taken through the existing `Report::emit_set_reachable_tus()` /
  `emit_total()` API, never by editing the file.
* `docs/ROADMAP.md` — this lane's section goes in
  `docs/rungs/_draft-roadmap-9.21.md`.
* The 32-byte bound (§9.20.9, +11 TUs), the varint framing (§9.20.8) and the
  `??_` synthesis re-measurement (#152) are named as not-taken, not attempted.
