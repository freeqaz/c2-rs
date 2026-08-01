DRAFT for `docs/ROADMAP.md` §9.13 — written by lane w-adjust, to be landed by
the coordinator. Nothing in §1–§9.12 is touched. Full record:
`docs/rungs/2026-08-01-w-adjust.md`.

---

### 9.13 W-ADJUST — the largest never-measured row is worth 472, and the only clean one is worth 1,385

§8.7 named two rows and told this lane to treat them differently: **measure
`expr-intrinsic-this-adjust`, do not build it**; build
`expr-call-in-expr-recv-object-then-type-ptr-whole` only if its counterfactual
holds up. Both were measured the same way — one scratch sink at the row's own
refusal site, one warm scan, Δ`emit-in-class` against 34,674 — and the second was
built. The emitted census is **19.37 % → 20.15 %**.

| run | bodies | emitted | Δ emitted |
|---|---:|---:|---:|
| base `33d0049` | 703,875 | 34,674 | — |
| sink disabled (control) | 703,875 | 34,674 | **0** |
| #127, adjust offset 0 only | 708,193 | 35,108 | **+434** |
| #127, any adjust offset | 708,231 | 35,146 | **+472** |
| #128, named object receiver | 706,402 | 36,059 | **+1,385** |

The control is the row that matters: the *instrumented* binary with the sink
disabled reproduces the base scan on all five of its published numbers. An
instrument whose inertness is asserted rather than run is the twelfth instance of
this project's dominant failure, and it was cheap to avoid.

**#127 is 472 emitted, 5.4 % of the row, and 92.0 % of that is free.** 434 of the
472 are at adjust offset 0, where the receiver's true operand stream is
`[Load(this)]` and no new codegen is needed. The 10,469 `-whole` figure §8.7
already flagged is confirmed to belong elsewhere, and **8,790 is not the row's
worth by a factor of 18.6**.

**The row's name never said what it is.** 135,926 of its 135,941 bodies decline
at `eat_receiver_this` — the member-call production bails non-committally, the
assignment parser then runs `parse_expr`, and `parse_expr` stops on the
intrinsic. **The census key names the second reader's stop, not the first
reader's refusal.** That is also why the row has no `-whole` bit: its `Block` ctx
is `expr-intrinsic`, not `CALL_IN_EXPR`, so `whole_body_is_one_value` never runs
on it. §8.7's "largest never-measured row" is a consequence of the attribution,
and the same mechanism hides the completeness of **7,712 further clean emitted
functions** at the receiver sites.

**What it means for #131.** The two arms measured convert at **5.4 %** and
**100.4 %** — 19× apart — so no rate transfers, and #131 must be sized off stock.
Measured in emitted units for the first time (per-row dump joined to the obj's
`.text` COMDAT leaders):

| the three receiver-designator sites | emitted blocked | clean | clean ∧ complete |
|---|---:|---:|---:|
| `tail-recv-not-a-plain-b9-load` | 23,158 | 7,670 | 19 |
| `chain-recv-not-a-plain-b9-load` | 13,896 | 1,441 | 1,380 |
| `cmp-second-recv-not-a-plain-b9-load` | 6 | 0 | 0 |
| **total** | **37,060** | **9,111** | **1,399 (+3)** |

37,060 is **29.3 % of all blocked emitted** — #131 is the largest single site on
the emitted board, larger than any census key — and its honest worth is
**≈ 2,600 emitted (1.4 pp)**: 1,385 taken here, 472 from #127, and ~710 if the
remaining clean-not-whole stock converts at #127's own 15.3 %-of-clean rate.
**The raw stock overstates the site by about 14×.** The optimistic ceiling, every
clean row converting, is 9,111.

§9.11's `-whole` corruption was **verified against `:eof`/`:mid` rather than
trusted**: re-counted with the suffix, the clean-and-complete stock at these three
sites gains **3** functions (1,399 → 1,402). The corruption is real and it is
0.2 % of this table, which matters as a negative result — the 7,712 clean residue
is genuinely unmeasured, not merely mis-suffixed.

**#128 converts 100 % and its key's second half was never a blocker.** All 1,380,
plus 16 from three neighbouring `recv-object` rows, against 11 re-filing under
named codegen gates. The key reads `-then-type-ptr`, i.e. "the receiver form *and*
a pointer-typed operand", so it looked like two widenings. It is one: the `-whole`
measure's operand vocabulary is `eat_int_operands` → `eat_int_like`, while the
**shipping acceptance path** is `eat_call_args` → `parse_expr` with
`eat_int_like_or_ptr4`, which has admitted width-4 pointers since W22. **The
census measure is narrower than the emitter, and the difference is printed as a
second construct.** On the emitted board that mis-describes a further **7,983**
functions (`…-and-call-more` 5,663, `…-and-deref-load-more` 1,462,
`…-and-plumbing-more` 449, `…-and-op-more` 409). Repairing `eat_int_operands`'s
type gate is a small instrument change and belongs with board #110's `-whole{k}`
over-count and §9.11's lost suffix: **three corruptions of the same ranking
input.**

The 1,380 are **four distinct mangled names**, three of which are 1,379 of them —
`??6DebugFailer@@QAAXPBD@Z` (759), `??6DebugNotifier` (604),
`??6DebugWarner` (16) — one header-inline `TheDebug << s;` forwarder emitted once
per TU across 803 TUs. The emitted census counts COMDATs and 1,385 is 1,385, but
the *differential coverage* behind the rung is one source shape, which is why the
generated axis carries more weight here than the fixture.

The rung **reconciles to the unit**, which is the control that matters for a
change that re-routes bodies between productions. `chain-recv-not-a-plain-b9-load`
falls 94,948 → 30,183 and the 64,765 re-routed bodies account for themselves
exactly (2,537 accepted + 24,874 `tail-object-receiver-is-not-a-tail-call` +
28,300 "does not end at the call" + 9,046 argument-vocabulary + 8); the 2,539 that
changed dispatch arm resolve as 2,527 in class + 10 refused one layer later by the
`.gl` linkage gate + 2 committed refusals. **One in-class shape label moved and
nothing shrank** — `multiarg-tail-call` 27,868 → 30,395 — so no previously
accepted body changed production, changed shape, or fell out of class. Stated
positively over 2.46 M bodies rather than as the absence of a complaint.

#### 9.13.1 ALARM — WR1's ordering rule was wrong from two setup words up, and it was live on mainline

The new fixture mismatched at first build. Bisected to a body with **no receiver
in it at all**:

```cpp
extern int gI;  void gs3(int*, int, int);
void b3() { gs3(&gI, 3, 4); }      // pure WR1: a data symbol as a call ARGUMENT
```

At `33d0049`, with none of this lane's code present, that body is **1/1 in class
and the port emits it wrong** (`Port=Mismatch @ offset 545`) — verified by
checking out `33d0049 -- crates`, rebuilding and diffing. WR1's rule was *"the
address `addi` is emitted LAST"*. c2's own `.cod` listing says it goes **SECOND**,
after exactly one word of the descending non-address walk:

```
    3d600000  lis  r11,?gI@@3HA
    38a00004  li   r5,4        <- one word of the descending walk
    386b0000  addi r3,r11,?gI  <- the address, SECOND
    38800003  li   r4,3        <- …and the rest of the walk follows
    48000000  b    ?gs3@@YAXPAHHH@Z
```

**At one setup word the two readings are the same sequence.** Eleven cells now pin
the rule — walks of length 0 to 4, the address at slot 0 and at a middle slot,
literals and in-place formals in the walk, free and member callers — and it
*subsumes* WR1's rule rather than contradicting it: address-last is the n ≤ 1
case.

Three consequences, each larger than the fix:

1. **§9.10's standing rule is attached to the wrong thing.** It says a rung that
   touches `coff.rs` must add a portable assertion for each ordering rule it
   establishes. This rule lives in `codegen/calls.rs`, had no unit test either,
   and failed the same way. **The rule belongs to the ordering rule, not to the
   file.** `the_data_address_addi_is_emitted_second_not_last` is now that
   assertion and it runs with no toolchain.
2. **The generated sweep was green over a wrong rule because the axis did not
   exist.** `53-data-symbol-addr.py`'s WR1 block emitted 70+ cases varying the
   address's slot, its destination register, the literal's value, the object's
   type and the mangled name's length — and never the **count**. Every case had
   ≤ 1 literal. Generated axes find what hand fixtures structurally cannot *only
   where the generator has the axis*; an axis a fragment does not vary is exactly
   as invisible as a fixture that does not arrange the case.
3. **A fixture's own blind spot is worth writing into the fixture.**
   `wadjust_obj_recv.cpp` states in its header that it cannot discriminate any
   slot-dependent rule, because the receiver is argument zero by construction —
   the sentence WR1's ALARM had to be discovered to produce.

#### 9.13.2 Pre-registration score — 4 of 8, and three of the misses are the findings

| | registered | measured | |
|---|---|---|---|
| E1 | #127 bodies 14,000, [5,000 , 30,000] | **4,356** | **MISS**, below the floor |
| E2 | #127 emitted 1,000, [300 , 3,000] | **472** | HIT (2.1× high) |
| E3 | ≥ 60 % of E2 at adjust offset 0 | **92.0 %** | HIT |
| E4 | control: gate disagreement goes non-zero | **0** | **MISS** |
| E5 | #131 ≤ 4× #127's realized | **5.5×** | **MISS** |
| E6 | #128 emitted 1,380, [600 , 1,380] | **1,385** | HIT on the point, ceiling wrong |
| E7 | #128 is ≤ 10 distinct names | **4** | HIT |
| E8 | receiver alone converts < 100 | **1,385** | **MISS**, 14× |

* **E1/E2 are the WR1 lesson repeating**: both estimates came from the same
  body-column anchor and both were high, 3.2× and 2.1×. The emitted number landed
  inside its interval only because the interval had been widened on WR1's
  precedent. A body-column anchor is not a source of an emitted estimate *even
  when it is transparently discounted*.
* **E4 registered the wrong control**, which is §9.9.2 again. The sink hands
  codegen `[Load(obj)]` where the true stream at `k != 0` is `[Load, Lit, Add]`,
  so it *does* over-claim — and `census/gate disagreement` cannot see it, because
  the port **accepts** the wrong stream and would emit wrong bytes rather than
  refuse. A gate-agreement counter separates "census accepted, port refuses"; it
  is silent on "census accepted, port would get it wrong". The control that works
  is the one run for #128 and not for #127: build it and put it in front of the
  differential — which is exactly how §9.13.1 was found.
* **E6's hit hides a wrong assumption**: the interval's ceiling was the row's own
  emitted count, on the reasoning that a row cannot convert more than itself. It
  took 16 functions from three neighbours. **A census row is not a unit of work.**

#### 9.13.3 Gate evidence

At `be797bf`, worktree configured against the shared toolchain:

* `cargo test --workspace` — **576 passed, 0 failed, 1 ignored** (pre-existing).
  **`#[test]` count 573 at the merge-base `33d0049` → 578 at tip**: five new
  portable tests, two pinning the ordering rule and its refusals, two pinning
  both directions of the `26`/`26` receiver-vs-chain discriminator.
* `c2rs selftest` — **208/208 PASS**, 0 fail, 0 skip.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes ran, 0 FAIL / 0 SKIP /
  0 NO-RESULT, **2,496 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — GATE_CROSS.
* 878-TU workload scan — 6 match, **0 mismatch**, 865 vocab-gap, 7 capture-fail;
  bodies 706,402/2,462,571 (28.69 %); **emitted 36,059/178,968 (20.15 %)**;
  census/gate disagreement **0**.
* Fixtures — `wadjust_obj_recv.cpp` 21/21 in class and `Port=Match`;
  `wadjust_obj_recv_neg.cpp` 0/11 and `Port=NotImplemented`;
  `wr1_sym_addr.cpp` 27/27 and `Port=Match` with its six new arity cells.

#### 9.13.4 New board items

* **#138 — repair `eat_int_operands`'s type gate to match the emitter's.**
  **Re-attributes** 7,983 emitted functions whose keys name a `-then-type-ptr`
  second construct the emitter does not refuse. Most of them carry `-more` and so
  would not convert on that widening alone — the claim is about where the ranking
  says the work is, not about free functions. Instrument, not a rung; goes ahead
  of any ranking taken off those rows. Sits with #110 and §9.11.
* **#139 — `expr-intrinsic-this-adjust` at adjust offset 0, 434 emitted.**
  Measured end to end here; the sink is 30 lines and is in `db812f7`. Needs the
  receiver designator to return an operand form richer than a token, which is the
  refactor #131 needs as a whole. **Schedule it at 434, not at 8,790.**
* **#140 — `call-arg-sym-permuted`: the data address beside a formal that has to
  move.** Refused by WR1 on one probe; it blocks every free-function caller of
  the #128 shape and is the largest single refusal inside the row this lane took.
  c2 pre-saves into r11 and moves the `lis` to r10 at two shifting formals — a
  designed capture grid over (formals moving) × (walk length), for which this
  lane's `q1`/`q3` listing probes are the template.
* **#141 — the other clean-not-whole receiver arms** (7,712 emitted, none with a
  completeness bit): `expr-op-0x27` 5,629 at this site, `expr-brfalse` 1,484,
  `assign-store-type-0x86` 1,138, `expr-intrinsic-dynamic-cast` 1,003. Each needs
  its own counterfactual; the two run here differ by 19×, so **no rate may be
  borrowed between arms**.
