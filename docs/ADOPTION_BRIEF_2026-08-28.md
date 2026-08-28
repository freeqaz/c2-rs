# ADOPTION BRIEF — 2026-08-28 (wave 18)

**Charter:** the owner, 2026-08-28 — *"lets keep chipping away and making
measurable progress by analyzing msvc to reproduce the behavior we expect"*,
with *"dont worry about the prices"* struck through the pricing track.

Recorded as `docs/DECISIONS_2026-08-22.md` § Decision 22. Board **#3755**.

Every number below was re-derived on tree `f91d7671b` by running the command
printed beside it — not quoted from a page or a board row
(`board-rows-decay-re-derive-before-relaying`, the memory this repo cost
itself on 2026-08-27).

---

## 1. The observation this wave is built on

**Five consecutive lanes have delivered reach 0.** `w-f0price`, `w-inlfit`,
`w-regcells` (wave 16), `w-sched`, `w-lowerband`, `w-s7` (wave 17) each wrote
**zero `crates/` bytes** by design and by prereg. That was correct for each of
them individually — a characterization lane's deliverable is address-cited
findings — and the reads they produced are the best this project has.

But the aggregate has a shape, and the scoreboard prints it:

```
$ cargo run -q --release -p c2-harness --bin c2rs -- subsys | grep ported
  coff-ported       RESIDUE          globregs-ported   RESIDUE
  dag-ported        RESIDUE          inline-ported     RESIDUE
  eh-ported         RESIDUE          label-ported      RESIDUE
  encode-ported     27 / 79          regalloc-ported   RESIDUE
  section-ported    1 / 15           symbol-ported     RESIDUE
```

**Eight of ten `ported` cells are RESIDUE, and exactly one subsystem has a
numerator that can move by writing code.** Decision 21 §4 forbids inventing the
missing ones, and that prohibition stands unchanged here — `#3505` is five for
five and this brief does not propose a sixth.

So the wave's question is not *"how do we make more cells numeric"*. It is:
**where can a read that already exists be turned into port code that the byte
judge can grade?** Three places, below, and two reads that make a fourth
possible.

---

## 2. What is adoptable today, with the evidence that says so

### L1 — `w-encarms`: the encoder's 52 unported arms (**the measurable one**)

`[encode]` is the only subsystem whose four strengths are all numbers, and the
map from c2's arms to the port's fields already exists:

```
$ head -1 work/w-encmap/armmap.txt
MAPPED 27 of 79 arms
```

The unmapped 52 are **enumerated with their opcode counts** in
`docs/whitebox/ref/ENCODE_ARMS.txt`, and the largest is not small:

| c2 arm | forms | c2 opcodes | mapped? |
|---|---|---:|---|
| `10bf9f91` | 78 | **104** | **no** — the single biggest arm in the table |
| `10bfa9f0` | 92, 94 | 38 | no |
| `10bfa81d` | 8,9,10,11,13,48,60 | 19 | no |
| `10bfa082` | 80 | 14 | no |
| `10bfa1ad` | 37 | 13 | no |
| `10bfa9bd` | 91 | 12 | no |

**The lane's first job is to find out what those arms are**, because "104
opcodes, one form" is as consistent with a table-driven general encoder as with
a refusal path, and the difference decides whether any of it is adoptable. The
second job is to adopt the ones that are, under the byte judge.

**The hazard, named up front.** Adopting an encoder arm converts a
`NotImplemented` into an emit. A wrong emit scores strictly below the refusal it
replaces — that rule is not suspended by this wave. And if the corpus does not
exercise the new arm, the required-zero byte delta is **green for the reason
`#3723` names**, not because the arm is right. So: every arm this lane adopts
gets a row in `c2_core::surface`'s registry (wave 17's `w-doctrine` machinery,
`crates/c2-core/src/surface.rs`) whose domain runs past what the corpus reaches.
That is what the registry was built for, and this is its first outside consumer.

### L2 — `w-inlbudget`: adopt C20's budget model (construct rung)

`w-inlfit` read the inliner's recursive expansion end to end and published the
result at `P_INLINE.md` §6.6.2, board `#3719`/`#3720`:

> **The growth budget is divided evenly among the remaining call sites.** At
> site *i* of *n*, the nested pass gets `remaining_budget / (n − i + 1)`.
> `level` becomes `BYTE [site+0x18] + level`. A `__forceinline` callee is
> **charged nothing** — `0x10b6240f` skips the local budget *and* the global
> growth total.

The port has none of it. `splice.rs`'s `S6-chain` has **no level, no budget, no
site count and no division**, and `w-inlfit` was explicit about why it is
nonetheless correct today: every admitted body has `n = 1`, so c2's divisor is
the identity there. Its own words: *"a soundness argument for a fit, not a
derivation of one"* — and it named the hazard, `#1020`'s: **the moment a lane
widens `S2` to two call sites, c2's divisor stops being 1 and the port has
nothing to divide.**

**This is the cleanest adoption available in the project.** The behaviour is
read, address-cited, and its adoption is provably byte-neutral on the admitted
set — not by hope, but because the divisor is 1 there by construction.

Grade it as a construct rung: `Fixtures: none`, `Census: +0`, **required-zero
byte delta**, identity diff over the 21 gate rows. And because `#3723` says that
criterion is not sufficient, the lane **must** register the budget model as a
decision surface whose domain includes `n ≥ 2` — where the port's obligation is
to **refuse**, not to guess.

**Secondary, same seam:** `w-doctrine` left `#3723` open on registry
completeness — 13 of 19 boundary-named consts uncovered, four of them named as
real refusal boundaries, and the screen is a NAME screen with two known false
positives. This lane owns `surface.rs`; it closes as much of that as it can and
says what it could not.

### L3 — `w-globobj`: the lowest agreement cell on the board, and it is the allocator's input

```
$ … subsys | grep globregs
  globregs-marks-obj 2      globregs-marks-total 48     →  4.2 %
  globregs-read 26          globregs-sites 19
  globregs-exercised-proxy RESIDUE                globregs-ported RESIDUE
```

**2 of 48 is the weakest agreement strength of the ten subsystems**, and
`P_GLOBREGS.md`'s own header says what the page is:

> adjacent: `P_REGALLOC.md` (the *colouring*; this page is the *candidate set
> and its order*, **which is that page's missing input**)

The owner named the register allocator as one of the two most valuable
subsystems. Its colouring rule is `[O]` on 26 cells after wave 16. **Its input
is `[R]` on 46 of 48 marks.** Converting `[R]` → `[O]` here is the cheapest
evidence in the subsystem the owner asked for, and `w-regcells` has just
demonstrated the exact method on this exact band (20/20 cells, four rivals
refuted, at two optimisation profiles).

### L4 — `w-inlswitch`: c2's own inliner decision surface, entirely unread

Decision 15's instruction is that general layers expose decision points as
**named, settable parameters**. `w-inlfit` found c2 already has them, and did
not read them (`#3718`):

* **21 undocumented `-inl*#` switches**, value words at
  `0x10c45db4`–`0x10c45e10`, recovered by `work/w-inlfit/optmap.py` from the 484
  stores that *build* the BSS descriptor table. Unread.
* **`FUN_10b5da2f`** — 573 B, unread, and it reads `k` **twice**
  (`0x10b5da64` as a multiplier `(n+2) × k`, and `0x10b5dacb`, which `w-lowerband`
  found and `#3734` filed as a correction to `#3717`).
* **`DAT_10c3de20`** — 389 refs, 10 writers, three values. `w-lowerband` §7
  filed this as the follow-up it did not take, in words worth repeating:
  *"naming the switch that sets it to `2` would make c2 narrate its own inline
  decisions"* — the direct measurement of the quantity this entire thread is
  about. It is **not** in the descriptor table `optmap.py` recovers, so it needs
  the writer set walked, not a table lookup.

This is a read lane and it will adopt nothing. It is here because it is the
input to the *next* wave's adoption, and because a settable knob found in the
compiler beats one invented in the port.

### L5 — `w-clausefix`: the table that grades the inliner is wrong about a third of its own addresses

`w-inlfit` §4, board `#3721`:

> `check_table.py`'s ADDRESS check asks about containment in the owner function.
> It **cannot fail on a mid-instruction address**, and **eight of the 24 are**:
> C2, C3, C4, C14, C16, C17, C18, C19.

Two of those eight (C18, C19) are `0x11b` bytes early because they landed in an
**instruction-for-instruction duplicate of the wrong function**
(`0x10b62488`–`0x10b624be` copies `0x10b5fb85`–`0x10b5fbbb`). C10 is a third
kind: *aligned*, but decoding to `call 0x10b5e64d` rather than the `0x2000` test
the row claims.

`work/w-inlfit/addr_align.py` is the missing half of the check and was already
**watched RED on a planted one-byte shift** before its verdict was quoted. It
lives beside the checker instead of inside it, and the 24-clause table — the
instrument the inliner's whole conformance story is graded on — still carries
the eight bad rows because `w-inlfit`'s prereg correctly forbade it from editing
another lane's frozen instrument.

Repair them under a prereg that owns `CLAUSES.tsv`, fold `addr_align.py` into
`check_table.py`, and settle C10.

---

## 3. What this wave does NOT authorise

Unchanged from decisions 20 and 21, and restated because a wave that adopts is
exactly when these slip:

* **No full register allocator.** F5 is not separable from F0 and the port
  schedules nothing. L3 reads the allocator's *input*; it builds no allocator.
* **No invented `ported` numerator.** Decision 21 §4. `#3505` is five for five.
  `[encode]` moves because it already has a defined denominator, not because
  this wave defines one.
* **No new count-bearing `gate.sh` row**, from any lane (`#3691`) — a 22nd row
  makes `gate_identity_diff.sh` exit 2 and refuse to diff for every other live
  lane.
* **No re-taking `#3534`.** `byte-owned` stays cited.
* **No adoption of 128** as the inline ceiling. `w-lowerband`'s `#3732` closed
  that trap: §2.1b's one-sided rule holds at `T = 98` and re-reading
  `w-sizebracket`'s committed 168 cells gives **8 counterexamples in each
  direction** at the image's 128.

## 4. The seams, because three lanes touch the inliner

Assigned so no two lanes write the same file:

| lane | owns | must not touch |
|---|---|---|
| `w-encarms` | `crates/c2-core/src/codegen/mop.rs` + encoder, `work/w-encarms/` | `splice.rs`, `P_INLINE.md` |
| `w-inlbudget` | `crates/c2-core/src/codegen/splice.rs`, `crates/c2-core/src/surface.rs`, `surface/DOMAIN.txt` | `P_INLINE.md`, `mop.rs`, `CLAUSES.tsv` |
| `w-inlswitch` | `docs/whitebox/ref/P_INLINE.md`, `docs/whitebox/WB_INLSWITCH_*` | `crates/`, `CLAUSES.tsv` |
| `w-globobj` | `docs/whitebox/ref/P_GLOBREGS.md`, `docs/whitebox/grids/w-globobj/` | `crates/`, `P_REGALLOC.md` |
| `w-clausefix` | `work/w-inlmetric/CLAUSES.tsv`, `docs/whitebox/scripts/check_table.py` | `crates/`, `P_INLINE.md`, `surface.rs` |

`docs/BOARD.md` and `docs/rungs/` are shared by construction; every lane writes
**only** rows in its own reserved range and its own rung file, and
`docs/rungs/INDEX.md` is regenerated at merge, never hand-resolved.

**Board:** `#3755` this brief · `#3756`–`#3761` `w-encarms` ·
`#3762`–`#3767` `w-inlbudget` · `#3768`–`#3773` `w-inlswitch` ·
`#3774`–`#3779` `w-globobj` · `#3780`–`#3785` `w-clausefix`.
`#3647` remains reserved-and-unspent. Next free `#3786`.
