# w-heap — PREREGISTRATION

    Lane:    w-heap, branch `wt-w-heap`, off master `25bd166d`.
    Target:  convert `src/xdk/nuispeech/xboxheap.cpp`; TU match 10 -> 11.
    Written: BEFORE any probe obj exists in this lane. The only objs I have
             read are the ones `docs/rungs/2026-08-08-w-front2.md` prints in
             its own text (`x6`'s, §3.2). Nothing under `work/w-heap/` exists
             yet but this file.

---

## 0. A correction to the brief, registered up front

The brief names the target `src/system/xbox/xboxheap.cpp`. **There is no such
file in the workload.** `work/dc3-workload/files.txt:875` is
`src/xdk/nuispeech/xboxheap.cpp`, which is the path board #401, #507, #1097 and
`w-front2` all use. I am working the `src/xdk/nuispeech/` one and every number
below is about it.

I also record now, before it can look like a post-hoc excuse: `w-front2` §8
claims `work/w-front2/ref/*/dis.txt` **is** committed. It is not.
`work/w-front2/ref/` holds five `.cpp` copies and no disassembly, and there is
no `xboxheap` directory under it at all. Nor is `work/w-front2/probe/x6/ref.obj`
present, though board #1106 cites it. So the "gift on the shelf" is the *source*
of nine ladder cells (which is real and which I will re-run) plus the x6
disassembly quoted in the rung's prose, and not the objs.

---

## 1. The incumbent, and the floor registered against it

**The incumbent is today's refusal.** `try_parse_store_run` returns `None` for
xboxheap's ctor, `IlBundle::functions()` drops the TU, `gap` reports
`vocab-gap`, and the port emits nothing. That refusal is **right 100 % of the
time on what it refuses** — 861 TUs, 130,579 refused functions, and it has
never produced a wrong byte.

**The decline floor.** I decline rather than ship if any of these is true at the
end:

* **F-1.** Any cell in a frozen grid emits and is not byte-exact, and I cannot
  make the reader refuse exactly that cell **by a structural predicate** — one
  expressible in `crates/c2-il` without consulting `crates/c2-core`'s models.
  "Refuse it by adding the failing offset to a list" is not a structural
  predicate and does not clear this bar.
* **F-2.** `mismatch` is anything but 0 at any gate row, or the sweep grades
  fewer cases than baseline, or `fnbyte-exact` shrinks, or `differs` grows, or
  `reloc-differs` moves off 861.
* **F-3.** The widening accepts a body whose acceptance I cannot trace to a
  named frozen cell. A production that admits a superset of what I measured is
  board #232's shape, and #232 is the reason this floor exists.
* **F-4.** The grid cannot be frozen before the first compile. If I find myself
  adding an axis after seeing a result, the lane declines and ships the grid.

**A reader that is mostly right is worse than today's refusal.** I will take
0 conversions over 1 conversion + 1 unproven accept.

---

## 2. What I think the three refusals actually are

### F2 — a member's address as a stored value (`crates/c2-il`)

`parse_store_stmt` (`leaf_store.rs:400`) admits a stored VALUE that is exactly
one of: a bare formal (`B9 <tok> <type>`), an integer literal (`33 …`), an FP
value, or an indirect load. `&this->mListHead` is none of those — it is a
formal **plus an off-add**, i.e. `[Load(this), AddrOf { off }]`, which the crate
already has a variant for (`IlOp::AddrOf`) and already parses in
`try_parse_addr_leaf`. So F2 is, at the reader layer, "let the value position
reach the address production the address *leaf* already uses".

The emitter consequence is that the run acquires a **second producer kind**: an
`addi rD,rBase,off` beside the `li`. Board #844 states that `xboxheap`'s run is
"`addi` at 2 uses beside `li` at 1, so clause 1 alone settles its allocation".

### F3 — a call after a store run (`crates/c2-il`)

`try_parse_store_run` requires the tail after the last store to be return
plumbing. A call there is refused. The crate already has a shape for a framed
body with calls and callee-saved formals — `BodyShape::CallSeq { params, calls,
tail, saved, … }` — so F3 is, structurally, "let a `CallSeq` carry a **leading
store run**", not "let a store run carry a trailing call". Which side owns it is
itself a prediction (P4 below).

### The seam (`crates/c2-core`) — board #844

`scheduled_gpr_run_text` is reached only from `store_leaf_text` and terminates
in an unconditional `encode_blr()`. #844: "there is no seam that emits a
scheduled store run as the MIDDLE of a framed body."

---

## 3. THE PREDICTION I EXPECT TO LOSE, and it is the headline

> ### **P0. The `x6` / `xboxheap` shape is board #870's BROKEN transfer, not board #866's IDENT transfer, and the price is therefore ≥ 4 rather than the 3 `w-front2` published.**

This is registered against `w-front2`'s own price and against the brief's
"the emitter plan already transfers — 96 cells, 12/12 IDENT". I think that is
reading #866 and not reading #870, which is the *next row on the same board from
the same lane*, and I get it from the x6 disassembly `w-front2` prints in its
own §3.2:

```text
  mflr 12 · stw 12,-8(1) · std 31,-16(1) · stwu 1,-96(1)
  mr 31,3
  stw 5,16(3) · mr 3,4 · stw 31,0(31) · stw 31,4(31)
  bl ?g@@YAXI@Z
  mr 3,31
```

Three things in that run are exactly the three #870 names as the transfer
boundary:

1. **The store base CHANGES MID-RUN.** `stw 5,16(3)` is based on **r3**;
   `stw 31,0(31)` and `stw 31,4(31)` are based on **r31**. `scheduled_gpr_run_text`
   emits a run against one base.
2. **The call's argument setup is INTERLEAVED INTO the run.** `mr 3,4` sits
   *between* store 1 and store 2, not after store 3. There is no seam shape in
   which the setup is a suffix.
3. **`r3` is consumed by the argument**, which is #870's stated cause
   ("`gx(u)` instead of `gx()` moves three things at once… because `r3` is
   wanted for the argument").

#866's grid held the callee **nullary** on the framed cells that read IDENT;
#870 is the 2 of 36 where an argument was added, and it DIFFERS. `x6`'s callee
is `g(initSize)` — it takes an argument. **So `x6` sits on the wrong side of
#866's own boundary**, and `w-front2` priced it at 2 by citing the side that
excludes it.

If P0 holds, "the plan is measured, not guessed" is false for this TU and the
emitter half is a **model** (where does the base switch, where does the setup
land) and not a **seam**, and the honest price is at least F2 + F3 + seam +
base-switch/interleave = 4, with the fourth being the same *"where does the
`mr r31,r3` live-range save go"* axis board #524 already recorded as
"a hypothesis with a mechanism and explicitly not a rule, n = 1".

**I would rather be wrong about this than right.** Being wrong means the lane
converts.

---

## 4. The grids, frozen before compiling

Manifest `work/w-heap/GRID.sha256` is committed **before** the first cell is
compiled, and every cell directory is its own (board #1045). Axes are
**structural**, not value permutations (`w-front2` §6, `w-hash` §4.2).

### GRID F3 — six axes, named by the brief, crossed

| axis | levels |
|---|---|
| **A. store count before the call** | 0, 1, 3, 6 |
| **B. store order / base** | one base; two bases; base written by the call's own receiver |
| **C. argument count of the call** | 0, 1, 2 |
| **D. callee kind** | free function; member on `this`; member on another object |
| **E. receiver slot** | n/a (free); slot 0 = `this`; slot 0 = a formal |
| **F. return-value use** | void; discarded int; ctor's implicit `return this`; `return <call>` |

Full cross is 4·3·3·3·3·4 with the illegal combinations removed. I will freeze
the **legal** enumeration, not the raw product, and print the removed count.
**Axis C level 0 vs ≥ 1 is the P0 decision axis** and every other axis is held
at xboxheap's own value while it varies.

### GRID F2 — three axes

| axis | levels |
|---|---|
| **G. member offset of the addressed sub-object** | 0, 8, 16 (xboxheap's `mListHead` is at 8) |
| **H. how the address is bound** | direct `&mListHead`; reference bind `BE& h = mListHead` |
| **I. where the address is stored** | into the addressed object itself (xboxheap's self-link); into a different member; into a different object |

Plus a **use-count** axis, because #844 says the allocation is settled by use
count alone: the address at 1 use, 2 uses, 3 uses.

### The orthogonality control

`x3` (F2, no call) ⊥ `x6` (call, no F2) is inherited from `w-front2` and re-run,
not rebuilt.

---

## 5. Registered predictions, scored at the end

| # | prediction |
|---|---|
| **P0** | **the one above** — the shape is #870's broken transfer, price ≥ 4, and I expect to LOSE this in the sense that I want it refuted |
| **P1** | F2 in the reader is **small** — the value position reaching `AddrOf` — and lands in `parse_store_stmt` alone, ≤ 40 lines |
| **P2** | F3 is **not** small, and it is not in `try_parse_store_run` at all: the accepting production is `CallSeq` gaining a store-run prefix, in `shapes/calls.rs` |
| **P3** | the emitter's binding fact is **the store base**, not the schedule and not the allocation. `order::schedule` and `alloc::allocate` will reproduce xboxheap's producer ranking correctly and the body will still differ |
| **P4** | `xboxheap.cpp` does **not** convert in this lane, and the lane ships the two grids and an honest ≥ 4 |
| **P5** | `IlBundle::functions()` widens by **more than one function** if F2 ships alone, because the address-as-value position is not xboxheap-specific — I will count it and prove the new accepts |
| **P6** | board #871's blocker really is discharged: a seam built today lands in `Seq`/`Framed` and `fnbyte-partial` stays 0. The #322 gate is open. **This one I expect to WIN**, and if I do, it is the finding worth more than the conversion |
| **P7** | at least one `w-front2` ladder cell does **not** reproduce on re-run, because its objs were never committed and the rung's §8 reproduction recipe references files that are absent |

**Expected loss direction, named as the brief requires:** I expect to lose on
**P0 being right** — i.e. I expect the lane to decline, and I expect the reason
to be the emitter's store-base switch, which is the half `w-front2` declared
licensed. If instead the cells show `x6` matching byte-exact with the existing
`scheduled_gpr_run_text` + `call_seq_text` composition, P0 is a MISS, P4 is a
MISS, and the lane converts.

## 6. Instruments

Two independent instruments must agree before anything ships:

1. `c2rs gap` — the whole-TU differential against real `c2.dll` under wibo. The
   sole judge.
2. `c2rs census --flags-file work/dc3-workload/flags.txt` — the class verdict
   and the first-refusal key.

Both at **the workload's own flags** (`/GR /O1 /Oi /EHsc …`), never the harness
default `/Ox` (board #1112). `census` takes `--flags-file`; `gap` takes
`--flags-file`. Neither is run without it in this lane.

Per-record binding is on `FnCensus::emit_name` (#918), never
`IlFunction::mangled_name`.

`work/w-splice/peerkeys.py` at both ends.
