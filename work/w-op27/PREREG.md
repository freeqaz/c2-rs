# w-op27 — PREREGISTRATION

    Lane:   w-op27
    Base:   master f49fe5e1 ("docs: regenerate the STATUS block — TU match 11,
            and the FRONTIER is 16")
    Branch: wt-w-op27, worktree .claude/worktrees/w-op27
    Written BEFORE: the first probe obj, the first `cl.exe` invocation, and the
            first line changed under `crates/`. Nothing under `crates/` has been
            touched at the time of writing; `git status` in the worktree is clean
            except for this file.

This file is frozen once the first measurement runs. Corrections go in the rung
doc as scored misses, never by editing a line above the score table.

---

## 0. The rung as briefed

`expr-op-0x27` is the **largest blocking row on the emitted board**: 22,373 of
130,575 blocked emitted functions, **17.1 %**, rank 1, with nothing close
(`body-cflow-label` 14,990, `expr-intrinsic-this-adjust` 9,267). It is also the
key that four separate lanes hit this week while working on something else:

* `w-lineage`'s `CHAINBIND`, `DEEP-GP`, `REVERSE` and `SELF-2B` cells — the four
  that killed allocation keys ten, eleven and twelve and priced board #1266;
* `w-mrslot`'s three unconverted GRID R cells;
* `w-front3`'s `lv_*` cells;
* `w-mrslot`'s dead-bind template defect.

## 1. The prior art I read BEFORE writing this, oldest hit read LAST

Searched `expr-op-0x27` across `docs/` and `crates/`, then searched `BOARD.md`
by topic (`fall-through`) separately because the board's rows do not contain the
phrasing a topic grep uses. Twelve independent prior measurements exist. In the
order I read them (newest first, as the standing instruction requires):

| where | what it says |
|---|---|
| **#1288** (`w-front3`, newest) | `x3`/`x0` moved **out of** `expr-op-0x27` into `store-run-bind-address-producer` / `store-run-bind-mixed-kind-alloc` **without one byte of source changing** — "the fall-through moved when the reader got far enough to name what it was doing" |
| **#1294** (`w-lineage`) | `CHAINBIND`, `DEEP-GP`, `REVERSE`, `SELF-2B` "all report `expr-op-0x27`" — and the finding is that the four are **one predicate outside the reader's class**, not four rules |
| **#684**/**#687** (`w-build`) | the ONE-AWAY screen: `op:27` mass **402,148**, one-away **24,088** — and #687 corrects the screen's own semantics: the poison count is an **upper bound with a named leak** (poison is raised *above* the pointer-arithmetic and one-byte-unsigned guards) |
| **#466** (`w-tu1`) | the one registered **limit** on #150's generalization: a fall-through argument does **not** apply to a key whose construct the port already emits |
| **#441** (`w-brfalse`) | the greedy blocker-key ladder "has a head at ALL TIMES and the head never converts"; the corrected ladder's co-head is **`expr-op-0x27` converting 2**, "the canonical zero-TU key of board #150 itself, with **seven** prior confirmations" |
| **#400** (`w-dclass`/B) | the end-to-end counterfactual: two 878-TU `c2rs gap` runs of the **same unmodified binary**, one env var apart. `expr-op-0x27` **23,090 emitted blocked (17.6 %, rank 1) → 0, key absent**; **407,017 all-blocked (23.2 %) → 0**; emitted census **38,458 → 38,464**; **TU match 8 → 8**. The other **23,084 were RENAMED, not converted** |
| **#364** (`w-joint2`) | `0x27` is **already accepted unconditionally** in `codec.rs`, `designator.rs`, `control_flow.rs` and `ctor_dtor.rs`; only `parse_expr` gates it, behind `C2RS_SINK_OFF_ADD_ARG=expr` |
| **#622** (`w-frame2`) | promoting the sink on `xboxheap.cpp` moves the blocker `0x27 → expr-op-0x32` and converts nothing |
| **#269** (`w-conv`) | the standing decline clause: *a frontier TU at ≥ 4 independent refusals is not a target* |
| ROADMAP **§9.16.5** (oldest, read last) | the mechanism, shown rather than argued: on `xboxheap`'s L2/L3/L4 the census reports blocking byte **96** for three structurally unrelated constructs, and the **in-class control L1 contains the identical byte at the identical offset behind an identical 96-byte prefix**. So the reported byte demonstrably does not block. `expr-op-0x27` records **where the second reader stopped after the first reader declined**. "Every 'go to the byte' investigation that started from an `expr-op-0x27` window has been reading the wrong bytes" |
| ROADMAP **§9.17.6 / §9.19 / #150** | the original: "granting its named token in `parse_expr` converts **6** emitted functions"; **#150 is closed at 6**; the board should carry 6, not 22,759 |
| ROADMAP **§10.6** | the re-pricing table: `expr-op-0x27` stock **22,759 emitted**, realized **6** |

**This row has therefore already been measured end-to-end at least twice with a
full 878-TU counterfactual (§9.19 and #400) and confirmed seven more times
sideways.** It has re-entered the widening order on **size alone** for at least
the third time. That is precisely the failure mode the brief names.

## 2. Which of the five categories I expect this to be

Registered before any measurement:

1. a private limit inside a recognizer that already exists — **NO** (30 %
   residual probability; see §3, I will still grep for it because it is the
   cheapest question and it is the most common answer on this board)
2. a production misfiled under an opcode — **NO**
3. real, and far smaller than its size — **PARTIALLY**, as a consequence of 5
4. unmeasurable, because the instrument has no production for it — **NO**; the
   instrument exists and is named (`C2RS_SINK_OFF_ADD_ARG=expr`)
5. **mis-described — YES. This is my primary registration.** `expr-op-0x27` is
   not a construct row. It is the **fall-through label of the generic
   expression parse**: `Block { ctx: "expr", byte: 0x27 }` with no entry in
   `expr_opcode_name`, raised wherever `parse_expr` was reached after every
   non-committal shape recognizer above it declined. Its census mass is the mass
   of **everything that falls through to it**, so ranking by it ranks the
   ladder's floor.

**Category 5 with 3 as its consequence.** The row is not mis-scheduled as "one
construct that contains zero of it" in the usual sense — it is scheduled as a
construct and is in fact a *residue*, which is category 5's sharper form.

## 3. What I predict widening converts

**The counterfactual number, at THIS base, and I take the ceiling neat per
board #770's rule.**

* The blocker class here is **not** a class whose emitter is missing — `0x27` is
  accepted in four other parsers already (#364) and `AddrLeaf` lowers the leaf
  form to one `addi`. So #466's qualifier is live and I do **not** get to assume
  #150's argument transfers unexamined. That is why I am re-running the
  counterfactual instead of citing #400.
* **Ceiling, neat: 24,088** — #684's one-away poison count for `op:27`, which is
  an upper bound with #687's named leak, so the true ceiling is ≤ that.
* **Point estimate: +6 emitted functions, +0 TU match**, unchanged from #400,
  because #400 measured the *identical* env-var counterfactual with the same
  binary on the same workload.
* **Independent refusals between ceiling and emitter:** counted as required. On
  the canonical body (`xboxheap`) `w-dclass`/B counted **5** folded / **7**
  unfolded, and asked "what varies between these refusals?" — the answer is
  **not** "nothing, one variable at different thresholds". F1 (a literal store
  mixed into a formal store run), F2 (a member's address in *value* position),
  F3 (a call composed after a store run) and F4a/F4b (the producer schedule,
  two decisions, the second unfitted) vary in construct, in layer and in crate.
  So it is **not one refusal**, and the ceiling does not collapse to the
  emitter.

## 4. The direction I expect to be wrong in

The brief records that estimates on this project have missed in the **optimistic
direction ten consecutive times** (board #770). Registered explicitly:

* **My headline prediction (+6) is a transcription of a three-day-old
  measurement, and the base has moved.** Between `9f9e6c0` (where #400 was
  taken) and `f49fe5e1` there are many merges, several of which widened readers
  in exactly this layer (`w-front3` moved two cells out of `0x27` with no source
  change; `w-mrslot`, `w-lineage` and `w-bd` all worked the store/bind layer).
  **`expr-op-0x27` is 22,373 today and was 23,090 then — it has SHRUNK by 717,
  which means the population under the key is not the same population.**
* Therefore **the direction I most expect to be wrong in is that today's
  counterfactual converts MORE than 6** — because widened readers mean more
  bodies now reach the end of the walk once the token is granted. I am
  registering the *pessimistic* guard rather than the optimistic one: if the
  number comes back **> 6**, that is a real change and I must not report it as
  "#400 reproduces".
* The second direction I expect to be wrong in: I expect the **successor
  distribution** to have changed more than the headline. #400 saw `CritSec.cpp`'s
  8 become 1 conversion + 7 blockers under five keys. If `store-run-bind-*` keys
  now absorb a visible slice, that is #1288 generalizing and is the finding.
* Third: I may be wrong that this is category 5 *rather than* 1. If a sibling
  recognizer carries a private limit that is one byte narrower than its
  siblings, the fall-through would be *manufactured* by that limit rather than
  intrinsic. §5 registers that grep as mandatory and its result as reportable
  either way.

## 5. What I will do, in order, and the stopping rule

1. **Both-ends evidence at base** (`f49fe5e1`): `c2rs gap` TU match / mismatch /
   codegen-gap / vocab-gap / capture-fail, the `gap-metric` block, `fn_blockers`
   and `emit_blockers`, `git grep -c '#\[test\]'`.
2. **The category-1 grep, before any build.** Every sibling recognizer in
   `crates/c2-il/src/func/body/shapes/` that reads `0x27`, compared **in both
   directions**: does any copy refuse *more* than its siblings? Report the
   answer either way. Specifically: `leaf_addr.rs`'s `-0x8000..=0x7FFF` bound,
   `designator.rs`'s destination-only reading, `codec.rs`, `control_flow.rs`,
   `ctor_dtor.rs`, `leaf_store.rs`, `assign.rs`, `store_run` / `leaf/store.rs`.
3. **The overloaded-`None` check** the brief demands, by analogy with `w-bd`'s
   `chain_skip_form`: is `expr-op-0x27` one key over several *productions*? It
   is minted at exactly one site (`Block::feature`, `ctx == "expr"`, byte 0x27,
   no name in `expr_opcode_name`) — but `ctx == "expr"` is reached from many
   callers, so the **key is one label over N productions**. I will count how
   many distinct productions reach it, and report that count as the deflation/
   inflation answer. If the answer is "many", the 22,373 is a sum over
   productions and no single rung owns it.
4. **The end-to-end counterfactual at this base**: two full 878-TU `c2rs gap`
   runs of the SAME binary, `C2RS_SINK_OFF_ADD_ARG` unset vs `=expr`. Report
   TU match, emitted census, the key's own size, and the successor keys by name.
   **Registered caveat, from board #661: `C2RS_SINK_OFF_ADD_ARG` is NOT a
   measurement-only sink** — its `0x27` arm ends `ops.push(IlOp::Add)`, there is
   no poison arm, and `cargo test --workspace` is RED under it (2 failed, per
   #403 / `w-dclass`/B §5.4). So the sink-ON scan is a scan of a *different
   parser*, and I will say so beside every number it produces. It is a valid
   counterfactual for "what does granting this token convert" and is **not** a
   shippable widening.
5. **Cross-check against the four lanes that hit the layer**, using their own
   cells, that the key they report is this fall-through and not a distinct
   construct.
6. **Stopping rule, registered now:** if step 4 reproduces #400 to within the
   drift explained by step 3 — i.e. the conversion is in `[0, 24]` emitted
   functions and **0 TUs** — then **board #269's clause and #150's rule both
   fire and I DECLINE**, ship the record and the number, change nothing under
   `crates/`, and mint the board rows. I will not build.
7. If step 4 comes back at **> 100 emitted functions or ≥ 1 TU**, the row has
   genuinely changed and I re-open, freeze a grid (sha256 + every rival's
   predictions, structural axes crossed first, **arity varied inside each
   cell**), and grade against real `c2.dll` — never by disassembly reading.

## 6. What I am NOT going to do

* Not touch `crates/c2-core/src/codegen/labels.rs` (lane `w-cflowlabel`),
  `scripts/gate.sh`, `crates/c2-harness/tests/cli_flags.rs`, `scripts/status.sh`
  (lane `w-throughput`), or `crates/c2-core/src/codegen/coff.rs`
  (single-occupancy).
* Not promote `C2RS_SINK_OFF_ADD_ARG` to a default. It is red on the workspace
  suite and #400 already priced what it buys.
* Not rename `expr-op-0x27` or any published key spelling. `expr.rs:993` records
  that a dozen keys "must keep their published spellings", and a rename would
  silently invalidate every recorded comparison — the one failure a census
  instrument cannot survive.
* Not grade anything by disassembly. `w-lineage` had a change read 0 wrong of 30
  by disassembly and **11 of 30 `Port=Mismatch`** by the differential.

## 7. Board rows

Allotted **#1333–#1342**. Minted with the work; any left unused stay unminted
and are named as such in the rung doc.
