# w-mrslot — PREREGISTRATION

    Lane:   w-mrslot, worktree branch `wt-w-mrslot` off master `6dce97a1`.
    Rung:   `store-run-bind-call-tail-mr-slot` — take the correction lane
            `w-carrier` named and deliberately did not take (board #1212).
    Written BEFORE the lane's first probe obj exists, before the first line of
    `crates/`, and before GRID R's generator exists.

Baseline, measured in THIS tree with a binary built from `6dce97a1`
(`work/w-mrslot/c2rs.base`, scan `work/w-mrslot/scan_mrslot_base.log`):

    TU match 10 · mismatch 0 · codegen-gap 0 · vocab-gap 861 · capture-fail 7
    fnbyte-exact 36,212 · differs 2,111 · reloc-differs 861 · partial 0
    fnbyte-match-tu-differs 0 · fnbyte-match-tu-reloc-differs 0
    factors A 28 · B 338 · C 169 · D 10 · E 2 · frontier 17
    census 711,485 / 2,463,443

---

## §0. WHAT #584's LEADING RUN IS FOR A MULTI-SYMBOL RUN — my reading, stated first

Board #584 corrected `ORDER`'s layout constant `u`. The two readings are:

* **COUNT** — `#{stores whose value materialises nothing}`, capped at 2. This is
  `order::head_slots` and it is `ScheduledRun::nsw`, and it is what
  `codegen::store_run_call::save_slot` is fed today.
* **LEADING RUN (#584)** — walk the **FINAL store order** from its front and
  count stores whose value materialises nothing, **stopping at the first store
  that consumes a producer**, capped at `HEAD_SLOTS_MAX = 2`. This is
  `order::lead_slots`.

**On a single-symbol run the two are equal.** `store_order` forbids a store whose
producer has rank `j` from occupying a position below `u + j`, so with one symbol
the unproduced stores are pinned into the head and the leading run reaches the
count (capped). `order::tests::the_two_readings_of_u_agree_on_every_single_symbol_run`
enumerates 5,460 of them.

**On a MULTI-SYMBOL run they are NOT equal, and the leading run is the smaller
one.** The store order is additionally pinned by symbol grouping: stores against
the same base symbol are kept together, and that pin can place a **produced**
store ahead of an **unproduced** one. The moment that happens the leading run
stops early — while the count keeps counting the stranded unproduced store. So

    u_lead  =  |leading run of unproduced stores in the FINAL order|  <=  min(count, 2)

with strict inequality exactly when a produced store precedes an unproduced one.

**A reference bind IS a second base symbol** (board #1128), which is why board
#1199's carrier is what opened this region: before it, the only multi-symbol
admission was the all-one-literal run, where the count is 0 and both readings
agree trivially.

### §0.1 THE CORRECTION, stated as the thing that will be graded

    H-LEAD:   stores_before_mr  =  nprod - 1 + min(u_lead, 2)
              with u_lead = #584's LEADING RUN, not the count.
              REFUSED when nprod == 0 and u_lead < 2 (board #867's domain
              boundary, `REFUSED_EMPTY_POOL`, unchanged).

Two facts I register now because they are checkable without any probe:

1. **The domain refusal is unchanged by the swap.** At `nprod == 0` every store
   is unproduced, so no produced store can precede one, so `u_lead == min(count,
   2)`. `REFUSED_EMPTY_POOL` therefore refuses exactly the same bodies under
   both readings. If this turns out false the swap is wider than I said and the
   lane declines.
2. **The swap is INERT on every single-symbol run**, by §0 above. So every body
   the port emits today must emit the same bytes after the swap. This is the
   lane's own no-regression invariant and it is measurable: sweep `88` at
   **97/1,479** and `89` at **302/968** must not lose a `Port=Match`.

### §0.2 It is read off c2's OWN bytes, not off the port's model

`u_lead` is defined over the **final store order**, and real `c2.dll`'s
disassembly *shows* the final store order. GRID R is therefore scored by reading
`u_lead`, `count`, `nprod` and the observed `mr` slot **out of c2's emitted
words**, not out of `order::store_order`. The port's model never enters the
scoring, so a cell where the port's store order is itself wrong shows up as a
byte mismatch rather than as a silently re-labelled class.

---

## §1. WHAT THIS LANE INTENDS TO SHIP

1. `order`: expose #584's leading run as a function of the statements
   (`lead_slots` already computes it; it is private and recomputed nowhere).
2. `codegen::leaf::store::ScheduledRun`: carry **`u_lead`** instead of / beside
   `nsw`, derived from the schedule the run already asked for — not a second
   scheduler (`GAPS.md` §6).
3. `codegen::store_run_call::save_slot`: fed `u_lead`. `BIND_IN_A_COMPOSITION`
   deleted; its doc becomes the record of the correction.
4. `c2_il::bind_run_ops`: `STORE_RUN_BIND_CALL_TAIL` lifted, **and the
   live-argument-BASE clause board #1215 deleted as dead is restored**, because
   the clause that made it dead is the clause I am lifting. It must fire on a
   named witness with a printed count (board #1175).
5. `c2_il::bundle`'s `StoreRunBind` arm builds `CallSeq::store_run` for the call
   tail, mirroring `StoreRunCall`.
6. GRID R, frozen by sha256 before the first `cl.exe`, one directory per cell
   (#1045), at the workload's own `/GR /O1 /Oi /EHsc` (#1112).

---

## §2. REGISTERED PREDICTIONS

| # | prediction | scored |
|---|---|---|
| **P0** | **H-LEAD is right on every graded GRID R cell in domain, and the incumbent COUNT reading is wrong on at least 3.** If H-LEAD is wrong on even one cell, **nothing ships** and the lane declines. | |
| **P1** | **TU match 10 → 10.** `xboxheap.cpp` does NOT convert. Its first-refusal key is `store-run-bind-mixed-kind-alloc:eof`, which is checked *before* my clause and is peer lane `w-prod`'s. My lift moves the key it would report *behind* that one, and a key behind a live clause moves no byte. | |
| **P2** | **`xboxheap` advances exactly one rung, measured not inferred**: with the mixed-kind clause lifted in a scratch tree it reports `store-run-bind-call-tail-mr-slot:eof` today and **`store-run-bind-no-emitter-carrier:eof`** after my change, leaving **two** refusals below me (`no-emitter-carrier`, then `leaf/store.rs:274`'s `value_bound`). | |
| **P3** | **`k_call` returns `vocab-gap → match`** — `w-carrier` gave it up to buy the refusal and taking it back is the signal. | |
| **P4** | The four `88-store-run-call` sweep cases `w-carrier` bisected (`s1425`–`s1428`) go `Port=NotImplemented → Port=Match`, and sweep `88`'s port split **grows** from 97/1,479. Sweep `89` (**302/968**) does **not** shrink. | |
| **P5** | `mismatch` stays 0; `fnbyte-exact` does not shrink below 36,212; `differs` does not grow above 2,111; `reloc-differs` does not grow above 861; `match-tu-differs`/`match-tu-reloc-differs` stay 0. | |

### §2.1 THE DIRECTION I EXPECT TO LOSE ON

**P-LOSS-A (most likely).** *H-LEAD is right on the four cells it was read off
and wrong on a fresh one.* Board #1212's correction was named from `s1425`–`s1428`
— **four cells, all `nprod == 1`, all `u_lead == 0`**. That is one structural
point, and "a second clause fitted on the cells that produced it" is how all six
refuted allocation keys got written (`w-heap` §4.1.1). GRID R therefore holds
cells at `nprod == 2` and at `u_lead == 1` **with the symbol crossing present**,
which nothing in the record has ever graded. If H-LEAD misses there, P0 fails and
the lane declines — and that is the outcome I am registering as most likely to
be the one that happens.

**P-LOSS-B.** *The mr slot is not the only thing that moves.* Board #1169
refuted #866 in general: the leaf schedule does **not** transfer into a framed
body when the run stores a value the call keeps alive. #866's 96 cells and
`w-seam2`'s 34 are all **single-symbol**. A multi-symbol framed run may differ
from its leaf twin in the **store order** as well as in the copy's slot, in which
case no `save_slot` argument can be right and the whole composition tail stays
refused. GRID R pairs **every** accept cell with a leaf control carrying the
identical run, exactly as `w-seam2` did, so this is separable from P-LOSS-A
rather than confounded with it.

**P-LOSS-C.** *The restored live-argument-BASE clause refuses nothing* (board
#1175), or refuses something c2 emits correctly. I register in advance that if
it has no graded witness I will say so with a count of **0** rather than count it
as measured.

**P-LOSS-D.** Board #1189 — the schedule is **not monotone in liveness** (a run
mismatches at callee arity 1, matches at 2, mismatches at 3). Any conclusion of
mine that reads "hoist the live store" or "more live arguments is strictly
harder" is dead on arrival, and GRID R varies callee arity so I find out rather
than assume.

---

## §3. THE DECLINE FLOOR, REGISTERED AGAINST THE INCUMBENT

The incumbent is **today's refusal**: `store-run-bind-call-tail-mr-slot` in the
reader plus `BIND_IN_A_COMPOSITION` in the emitter. Its score on what it refuses
is **right 100 % of the time** — it converts 0 bodies and mis-emits 0 bodies. A
refusal cannot be beaten on precision; it can only be beaten on recall at equal
precision.

**So the floor is:**

* **0 wrong on every graded GRID R cell**, and **0 wrong on both sweep
  fragments and the cross product**. One `Port=Mismatch` anywhere and the change
  is reverted and the lane declines. Not "investigated" — reverted.
* **strictly positive recall**: at least the 4 bisected sweep cases plus `k_call`
  must convert. A change that is byte-safe and converts nothing is strictly worse
  than the incumbent (it is the same behaviour with more code) and is **also** a
  decline.
* the frozen grid must contain **≥ 3 cells that separate the two readings** and
  **≥ 1 cell in every named class**. A grid that cannot tell the rivals apart
  grades nothing, which is `w-carrier`'s own §5.2 and board #1175 one layer out.

**A registered decline is a successful lane.** If P-LOSS-A or P-LOSS-B fires I
ship the refusal's documentation, the counterexample and the grid, and `k_call`
stays `vocab-gap`.

---

## §4. WHAT WILL BE MEASURED, AND WITH WHAT

* **SOLE JUDGE**: real `c2.dll` under wibo + byte-exact obj compare with the
  COFF `TimeDateStamp` (offset 4..8) zeroed — `c2rs gap`, at the workload's own
  `/GR /O1 /Oi /EHsc` (#1112), one directory per cell (#1045).
* **class verdict / first-refusal key**: `c2rs census` at the same flags.
* **the emitted words**: `scripts/gt_dump.py` over the reference obj, which is
  where `u_lead` and the observed `mr` slot are read from.
* **the corpus**: BOTH sweep fragments (`88-store-run-call`,
  `89-store-run-live-arg`) tallied at BOTH ends with `work/w-carrier/tally.sh`
  against `work/w-mrslot/c2rs.base` and the tip binary (board #1205 — a lane
  that tallies only at its tip books conversions it did not cause), plus
  `scripts/gate.sh --require-graded` and the cross product.
* **the workload**: the 878-TU scan at both ends, and
  `work/w-carrier/blockers.py` over the two JSONLs, because `codegen-gap`
  partitions per TU and cannot register this payment (board #1164).
* **the ladder**: `xboxheap`'s first-refusal key *after* my change, measured by
  lifting the mixed-kind clause in a scratch tree with an uncommitted env hatch,
  on **both** `w-carrier`'s replica cell and the real dc3 TU.
* **peers**: `work/w-splice/peerkeys.py` at both ends.
* **integrity**: no artefact is rewritten in place; a damaged one is repaired by
  a clean re-run (#1135/#1236). NUL checks are byte counts via `tr -d '\000'` or
  Python, never `grep -c $'\0'` (#1236).

---

## §5. OWNERSHIP

I own the store-run call-tail emitter path: `codegen/store_run_call.rs`,
`codegen/leaf/store.rs`, and the bind reader's call-tail clause. Peer lane
**w-prod** owns `codegen/alloc.rs`, the `Producer` type and possibly
`eat_offset_adds`. **I do not touch `alloc.rs`.** Board #1235 — schedule and
allocation are different bits, and nine keys have died in the allocation seam;
no allocation question is answered in this rung.
