# w-seed — PREREG

Board **#1053**. Registered **before** one line of reader code, before GRID-N was
written, and before this lane ran `cl.exe` once. Lane `w-seed`, worktree off
master `29dab722`.

Everything below was decided from records that already exist: `w-memset` §5 (the
stop, and its price), `w-inl0` §3 (the `Reduction` seam), `w-fix`'s GRID-3 (the
fixpoint's stops), `INLINE_PREDICATE.md` §1.2–§1.4, and **one orientation read**
of the workload that involved no new capture beyond the standing 878-TU scan:
`c2rs census --fn __destroy_aux` on `src/lazer/meta_ham/CharacterProvider.cpp`,
which prints the census hexdump the harness already computes.

---

## 0. The rung, and the line it crosses

`c2_core::elide::Reduction`'s doc pins an asymmetry:

> a refused body contributes a **link and never a seed**

`w-memset` stopped at exactly that line. Level 5 of board #980's chain —
`??$__destroy_aux@…`, STLport's `p->~T()` on a class with a trivial destructor —
is refused as `expr-lit-type-8207` and has **no call in it at all**, so
`NoEffectCall(&str)` cannot express it and no chain through it can close.

**This lane changes E's rule: a refused body may SEED, when its grammar proves it
emits nothing *unconditionally*.**

### 0.1 The seeding predicate, as it will be shipped

A new **decode-only** reader `c2_il::…::no_effect::no_effect_nothing(seg) ->
bool`, alongside `no_effect_call` and `no_effect_loop` and sharing their
discipline: `parse_segment` byte-for-byte unchanged, the row still
`FnVerdict::Blocked`, still `fnbyte-refused`, `IlBundle::functions` still refusing
its whole TU.

It accepts a segment whose **whole content**, walked TOTALLY from the body marker
to `eat_return_plumbing`'s fail-closed terminal, is:

```text
  53                          the body scope, and NO scope deeper than it
  <line marker>
  33 <INT_TYPE> <varint>      an int literal      — value UNCONSTRAINED
  33 82 07 <id> <varint>      a VOID literal      — value and id UNCONSTRAINED
  44                          the bind
  4B                          the discard
  <line marker>
  <return plumbing, to the segment end>
```

and **nothing else**. The vocabulary is closed: no `26` (symbol push — which is
also the data-symbol push), no `B9` (formal load), no `BD`/`40`/`4C` (call,
intrinsic, apply), no `67`, no `9B`, no `29`/`38`/`3A` label outside the return
plumbing.

**Why the two literal TYPES are pinned and the two VALUES are not.** A literal is
pure whatever its value, and the statement is discarded, so the value cannot
change what is emitted — constraining it would be #644's "one producer, one
contiguous field" mistake again. The **type** is different and is a soundness
constraint: `CallRet::discarded`'s reason, already cited in `no_effect.rs`'s own
module doc, is that a `float`/`double` drags `_fltused` into the TU and the obj
grows a symbol. `int` and `void` are the two the capture carries and the only two
this reader will admit.

### 0.2 The `elide.rs` change

One variant and one arm:

```rust
/// A body the parser REFUSED, whose grammar proves it emits nothing AT ALL —
/// no call, no data symbol, no bytes. SEEDS. Has no link.
NoEffectNothing,
```

```rust
Reduction::NoEffectNothing => (true, None),
```

The `Reduction` doc's "link and never a seed" sentence is **rewritten in the same
commit**, because it is a licence the next agent will cite.

### 0.3 THE CYCLE ARGUMENT MUST BE RE-DERIVED, and here it is in advance

`w-fix` #950: *"A cycle is never **seeded**, so it is never admitted"*. That
sentence was true because the only seed was `empty_body`. It is now registered as
a claim with a proof rather than inherited:

1. **Termination is untouched.** The iteration admits a name only on a
   `false → true` transition, so a productive round admits at least one of
   finitely many names. That argument reads the *step*, not the seed set, so
   widening the seeds cannot affect it. The round ceiling stays exactly as
   written.
2. **A seeded name has NO outgoing link.** `NoEffectNothing` sets
   `link = None` by construction, and it can do so honestly because the reader's
   walk is **total** and its vocabulary contains **no call token** — a body it
   accepts names no callee at all.
3. **Admission propagates only backwards along links from a seed.** A name is
   admitted iff it seeds, or `link[i]` names an admitted name. Following links
   out of a cycle stays inside the cycle forever, so a cycle member is admitted
   only if some cycle member seeds.
4. **A cycle member cannot seed.** Membership in the link graph's cycle requires
   an outgoing link; by (2) a `NoEffectNothing` seed has none. (An `empty_body`
   seed likewise has no `tail_call` to step to — unchanged.)

∴ no cycle member is admitted. **Cell n06 is the compiled test of this, and
`a_cycle_is_not_elided_and_terminates` / `the_round_ceiling_cannot_fire` must
still pass untouched.** If (2) is ever weakened — a seeding reader that also
returns a callee — this argument collapses and the round ceiling becomes the only
thing between the fixpoint and a hang.

---

## 1. The point prediction — w-memset's 227, registered as a point

`fnbyte-blr-stop3-expr-lit-type-8207` reads **227** on this lane's own baseline
run of master `29dab722` (`work/w-seed/base.txt`), confirming w-memset's number.

| | registered |
|---|---:|
| `fnbyte-differs` | **2,334 → 2,107** (−227) |
| `fnbyte-exact` | **35,986 → 36,213** (+227) |
| `fnbyte-elided` = `-elided-exact` | **1,654 → 1,881** (+227, and EQUAL) |
| converted, per `(TU, emit_name)` | **227** |
| regressed, per `(TU, emit_name)` | **0** |

### 1.1 What would make the 227 wrong, said in advance

* **Fewer.** If the `82 07` void TYPE's id varies across the workload and the
  reader over-constrains it — it does not constrain it, but if some other field
  varies the same way the miss looks identical. Also if any of the 227 carries a
  second blocker above the elision (a `data_sym`, or a `Selected::Seq` caller).
* **More.** `fnbyte-noeffect-stop-expr-lit-type-8207` is **4,256**, and those are
  *chain extensions*, not differs — they are refused rows and cannot become
  `exact` themselves. If `exact` rises by more than 227, some population this
  prereg did not price converted, and that is a finding to publish and re-grid,
  **not** a better result to bank. `w-inl0`'s M2 is the standing reason: an
  unsound widening makes every published number move the good way.
* **The direction that ends the lane** is §3.

### 1.2 The secondary predictions

| # | registered |
|---|---|
| **P2** | `fnbyte-nothing-rows` (the new reader's own firing count on refused workload rows) is **≥ 4,256** — the stop histogram's count of chains that end at this production is a LOWER bound on the bodies carrying it, because a body nothing points at is not in that histogram at all |
| **P3** | the reader's own KNOWN ANSWER: for every row it admits, c2's own `.text` COMDAT is one `4e800020` with zero relocations, or **absent**. `fnbyte-nothing-ref-other` is **0** and is printed rather than inferred |
| **P4** | **#953 does not bite** — the hand cell's leaf and the workload's `??$__destroy_aux@…` go through the **same** production, verified by comparing the cell's census key and marked bytes against the workload's, not by assuming it. Registered as the claim most likely to lose: `w-fix` §5.1 says a hand grid and the workload reach one source idiom through *different* productions |
| **P5** | the residue is **E and not I**: c2 emits one `4e800020` and zero relocations for the converted wrapper at the workload's flags **and** at `/Ob0`, in every graded cell. `w-fix` #954 — a mid-chain inline is a bare `blr` at every level and only `/Ob0` tells them apart |
| **P6** | **`body-0x67` is untouched.** `fnbyte-noeffect-stop-body-0x67` **5,154** at both ends and **zero** `body-0x67` rows are admitted by the new reader. w-memset #1056: that refusal is what keeps E safe from an indirect call site (#921/#232) |
| **P7** | `l09`'s `the_pseudo_destructor_leaf_is_the_residue_and_needs_a_seed` goes **RED** on the unmodified test, and is rewritten in the same commit to assert the conversion. **That is the intended signal.** The lane will demonstrate the red before rewriting it |
| **P8** | **CONTROL** — `fnbyte-refused` **130,579**, `vocab-gap` **861**, `fnbyte-unbound` **9,217**, denominator **178,977**, `mismatch` **0**, TU match **10**, `reloc-differs` **861**, factors **A 28 · B 338 · C 169 · D 10 · E 2** all unchanged. `parse_segment` byte-for-byte unchanged |
| **P9** | **MUTATIONS**, each verified to have changed its file before its run is read (#951), each with a distinct named message, each under `timeout` so a hang is its own outcome: **M1** delete the totality terminal; **M2** delete the closed-vocabulary refusal so any statement is accepted; **M3** make `NoEffectNothing` also carry a link (which is exactly what §0.3 (2) forbids). All three must go RED |
| **P10** | **TWO INSTRUMENTS** — a crate-free Python reader over the same `.ex`, deriving the production by a different route, agrees count-for-count with the shipped Rust reader on one named TU. Difference **0** |

---

## 2. GRID-N — the structural axes, frozen before the first `cl.exe`

`work/w-seed/cells/`, `sha256` in `work/w-seed/CELLS.sha256`, committed before
any of them is compiled. Every cell is graded through **`grade_one`** — the same
function the 878-TU scan runs — at the workload's flags **and again at `/Ob0`**,
**per call edge**, with **the caller's bytes and relocation count printed beside
every verdict** (`w-fix`'s template, and #950's requirement).

| cell | the structural axis | registered outcome |
|---|---|---|
| **n01** | THE POSITIVE — a nothing-body reached DIRECTLY by a tail call | E fires; the caller is `tail`/`Exact`, c2's own body one `4e800020`, 0 relocations, at `/O1` **and** `/Ob0` |
| **n02** | chain DEPTH with a nothing-body at the end — three links above it | every edge closes; every caller `Exact` |
| **n03** | THE WORKLOAD'S OWN SHAPE — the five-level `_Destroy_Range` chain through the LOOP (`l01`'s source) | the wrapper converts to `Exact`. **This is `l09` going red on purpose** |
| **n04** | a nothing-body that is refused for a **DIFFERENT** reason — a virtual call, `body-0x67` | NOT admitted; the caller stays an honest `Differs`; c2 keeps its relocation |
| **n05** | a nothing-body reached through an **INLINE** rather than an elision — `int m(int a){return a;}` mid-chain (`w-fix`'s `k12`) | does NOT propagate; the port keeps both branches. At `/Ob0` the REL24 survives, which is what says it was I |
| **n06** | THE CYCLE — two functions that each carry the nothing-statement **and** call each other | the reader refuses BOTH (its walk is total, and a call is not in its vocabulary); neither admitted; `overflowed()` false. §0.3 compiled |
| **n07** | an **EXTERNAL** nothing-body — declared here, defined elsewhere | nothing admitted; c2 keeps its REL24 at both settings |
| **n08** | a nothing-body **WITH A DATA SYMBOL** — the discarded statement reads a global | refused; the caller is an honest `Differs` |
| **n09** | a body that keeps BYTES — `p->~T()` on a class with a **non-trivial** destructor | refused; c2 emits a real call |
| **n10** | a discarded **FLOAT** literal in the leaf | refused — `_fltused`. If c2 turns out to emit nothing for it, that is a match declined on purpose and it is stated, not taken |
| **n11** | direct **self-recursion** through a nothing-body | not admitted. `void r(){r();}` takes NO relocation (#950), so the bytes are printed and the verdict is not read off the relocation count |

Each cell carries `w-empty`'s **ANCHOR** — a callee this TU does not define, whose
REL24 must survive — **prepended** for `w-inl0` §4's measured reason (these cells
define templates), and the five-level **TAIL PAD**. A cell whose anchor is not
`Exact` graded nothing and is reported as such rather than scored.

**Every parallel probe gets its own directory** (#1045: four tests shared one
PID-keyed temp dir, the captures raced, and the lane fabricated a finding that
would have reversed its conclusion).

---

## 3. THE UNCONDITIONAL STOP

The lane stops, reverts to a declared **DECLINE**, and publishes the refusal with
a count, if **any** of the following is observed at any point:

1. `fnbyte-exact` **shrinks** below 35,986, at all, for any reason.
2. `fnbyte-differs` **grows** above 2,334, at all.
3. **Any** function moves `exact → differs`, measured per `(TU, emit_name)` from
   the two `--fnbyte-diff-jsonl` files and **never** by subtracting totals
   (`w-splice`: subtraction cannot tell "disjoint" from "two lanes fighting over
   the same functions").
4. `mismatch` is non-zero, anywhere — scan, gate, sweep or cross.
5. `fnbyte-reloc-differs` is not 861, or `match-tu-differs` /
   `match-tu-reloc-differs` is not 0, or `fnbyte-partition-broken` is not 0.
6. **Any `body-0x67` row is admitted** by the new reader — #1056, and #232's
   shape. This is checked by a printed count and not by an argument.
7. `fnbyte-elided` and `fnbyte-elided-exact` are not **equal**: a body the port
   elided that c2 did not is the wrong-emit direction, and the equality is the
   only thing that says the judge agrees.
8. `fnbyte-nothing-ref-other` is non-zero — c2 emitted something other than a
   bare `blr` for a body this reader called nothing.
9. Any peer key family stops printing (`work/w-splice/peerkeys.py`, both ends).
10. A GRID-N cell's ANCHOR is not `Exact` and the cell is scored anyway.

**A registered decline is a successful lane.** Nothing below the line is a reason
to widen a condition until a number moves.

---

## 4. Ownership

This lane owns `crates/c2-core/src/elide.rs`. It also writes
`crates/c2-il/src/func/body/shapes/no_effect.rs` (the new reader),
`crates/c2-il/src/func/census.rs` (one field), `crates/c2-harness/src/gap/fnbytes.rs`
(the new keys) and `crates/c2-harness/tests/destroy_loop_elision.rs` (`l09`).
It does **not** touch `crates/c2-core/src/codegen/alloc.rs` (w-alloc3) or
`scripts/gate.sh` (w-gate).

**`Reduction` is a SHARED enum.** Its readers were enumerated before it was
extended: `c2_core::elide::TuEmptyCallees::of_rows` and
`c2_core::splice::TuContext::of_rows`. The second matches
`Some(Reduction::Parsed(f)) => Some(f), _ => None`; the wildcard would silently
absorb a new variant, so it is made **exhaustive** in the same commit — four
lanes this week erased each other through shared semantics with no textual
conflict, and a wildcard arm is that failure with the compiler's help switched
off. `work/w-splice/peerkeys.py` is run at both ends and reported.
