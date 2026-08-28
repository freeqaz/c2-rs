# PREREG — `w-doctrine`, making `#3723` enforceable

    Lane:      w-doctrine
    Kind:      instrument (consolidation lane, decision 21 §2)
    Date:      2026-08-28
    Base:      master `8213c7b77` (decision 21's own commit)
    Board:     #3743–#3748 (mine, and only these)
    Wave:      17 — `docs/DECISIONS_2026-08-22.md` § Decision 21
    Brief:     decision 21 §2, consolidation row `w-doctrine`

**Frozen before the first edit to `crates/` and before the first measurement.**
This file is the first commit on `wt-w-doctrine`.

---

## 1. The defect, restated so the deliverable can be graded against it

Decision 20 grades a construct rung on *"required-zero byte delta, identity
diff on the 21 gate rows"* (board `#290`'s pattern). **That criterion passes a
real emit widening.** `w-regsel`'s control **C6** widened the caller's allowed
set from the volatiles to `r0..r31` — c2's callee-saved tail becomes reachable
from a production path — and:

* 471 of 475 crate tests still passed;
* no encoder row, no `store_run_call` row, no `leaf::store` row moved;
* `GATE: PASS` at both ends, **identity diff 0 lines over 21 rows**.

The widening would have shipped. It was caught only by that lane's own prereg
fail axis — a refusal-domain grid **decision 20 never asked for and
`w-regprio` was never told to build**.

The criterion can only see emissions the corpus **exercises**. A widening whose
new emissions are unexercised is invisible to it: `#1236`'s shape, a guard green
precisely because the offender is out of scope.

## 2. The deliverable, and the word that is the whole lane

**Enforceable, not advisory.** `#3679` (a check that existed, was green, had a
self-test, and nothing ran it) and `#3689` (a number printed on every run that
drifted 16→18 inside one wave with nobody reading it) are this repo's standing
finding: **an unenforced rule is a paragraph.** Another paragraph in
`docs/rungs/README.md` is the failure mode, not the fix.

The checkable form this lane will build, in four mechanical parts plus one
doc-side part that is honestly the weakest:

| # | part | what makes it enforcement rather than prose |
|---|---|---|
| **E1** | **`c2-core::surface`** — a registry of the port's decision surfaces (allowed set / candidate set / refusal boundary), each rendering a canonical row per point of an **enumerated domain that extends past what the corpus exercises**, plus a committed baseline `DOMAIN.txt` | a `cargo test` compares live against the baseline. A behavioural change to a registered surface **cannot land without the diff showing it in text** |
| **E2** | a **marker ↔ registry bijection** — every `SURFACE[<name>]` marker in `crates/` names a registered surface and vice versa | `#3641`'s shape: a rename silently emptying the population. A check over zero cells is green and says nothing |
| **E3** | **nonzero denominators** — every registered surface asserts a minimum cell count and a minimum refusal count | the same reason, per surface: a surface that refuses nothing is not a refusal boundary |
| **E4** | **`scripts/surface_audit.sh`** — the coverage ratchet. Every boundary-named `const` in `c2-core` is either COVERED by a registered surface or listed UNCOVERED with a reason, and the UNCOVERED count is a **ceiling** that cannot rise silently (`#3689`'s shape) | closes the only hole E1 cannot: the registry's own completeness. It **cannot be complete** and says so; what it can do is make the hole unable to grow quietly |
| **E5** | `rung_registry.rs`: a construct rung **dated on or after this lane** must carry a non-empty `Fail axis:` header field | grandfathered by date (`#3689`'s precedent — the population is dated records that stay as written). **Presence is not measurement and this part is the weak one**; it is shipped because it is what makes the doctrine bind on a *future* lane over an *unregistered* surface, which E1–E4 cannot reach |

**Blast radius, declared in advance.** E1–E3 fail only on the tree that changed
a registered surface — that is the intent, and the population is opt-in by
construction in `#3690`'s sense: a surface is graded because somebody registered
it, and registering it *is* the request to have it watched. E4's ceiling is a
one-line raise that is visible in the diff. E5 grades a doc population that is
**empty today** and can only be entered by a lane writing a new construct rung
in its own worktree; the merge funnel (`#3687`) runs `cargo test --workspace`,
so a red doc cannot reach master and therefore cannot redden a peer's tree.

## 3. Predictions, with confidence, frozen

| # | prediction | conf. | discriminator |
|---|---|---:|---|
| **P1** | **E1 goes RED on C6 reconstructed**, on a tree where `gate_identity_diff.sh` against base still reads **0 lines over 21 rows** and `GATE:` still reads `PASS` | 0.85 | plant C6 (`alloc.rs`'s `RegSet::range_inclusive(pool_floor, VOLATILE_GPR_TOP)` → the full file), run the surface test, run the full gate, run the identity diff. **If the gate moves, the demonstration is void and I say so** |
| **P2** | C6's signature in `DOMAIN.txt` is a collapse of the **refusal count** for `alloc.allocate` to **0**, with the **set of emitted registers unchanged** — because the order's head is `r11` whatever the allowed set is, so the tail never gets selected at `n <= MAX_MODELLED_PRODUCERS` | 0.75 | the rendered diff. If the emitted set moves too, P2 is refuted and the finding is *better* than predicted, not worse |
| **P3** | At least **one** registered surface outside the regalloc family also fires on a planted one-token widening — i.e. the registry is general and not `w-regsel`'s grid moved to a shared file | 0.90 | plant `needs_gpr_helper: >= 3` → `>= 4` (frame) and `BC_MAX_DISP` widened (reach); both must move `DOMAIN.txt` |
| **P4** | **Required-zero byte delta at my own tip**: identity diff 0 lines over 21 rows, `mismatch 0` at both ends. This lane adds no production code path | 0.95 | `scripts/gate_identity_diff.sh base tip` |
| **P5** | The boundary-constant screen (E4) is **noisy**: at least one constant it names is not a decision boundary at all | 0.80 | the UNCOVERED table's reasons. A screen whose false positives I hide is a screen I have not measured |
| **P6** | E5 grades **zero** existing rung docs — every construct rung in the tree today predates the cutoff — so it needs its own planted control or it is decoration | 0.90 | run it; then plant a dated construct rung with no `Fail axis:` and watch it red |
| **P7** | No new count-bearing `gate.sh` row; `gate_identity_diff.sh` still enumerates **21** rows at both ends | 0.97 | the diff script's own output |

## 4. The fail axis of THIS lane — `#3336`, named before starting

A required-zero byte delta is silent about everything that is not a byte, and
this lane changes no byte by construction (it is purely additive to `crates/`
plus tests and docs). **So the byte delta cannot fail here and does not grade
anything.** The axis on which this lane **can** fail:

> **THE CONTROL SET.** Every part E1–E5 must be watched RED on a planted
> defect, and **E1 must be watched red on C6 specifically** — the exact defect
> that motivated `#3723`. A part that has never been observed failing does not
> ship, and if I cannot make E1 fire on C6 I have not closed `#3723` and the
> lane says `FAILED` in that word.

Second axis: **the cost of being wrong about blast radius.** If E1–E5 redden a
peer lane's `cargo test` on a tree that peer did not touch, the instrument is a
net negative regardless of what it catches (`#3691` measured that exact cost for
a 22nd gate row). Measured by: which populations each part grades, stated per
part in §2, and by running the full workspace suite at my own tip.

## 5. Controls — planted, watched red, recorded, reverted

| C | planted defect | must go red in |
|---|---|---|
| **C6′** | **`w-regsel`'s C6 reconstructed** — `alloc.rs`'s allowed set widened past the volatiles | **E1** (`DOMAIN.txt` mismatch), **while the gate and the identity diff stay green** |
| **CF** | `FrameLayout::needs_gpr_helper` `>= 3` → `>= 4` | E1 |
| **CR** | `BC_MAX_DISP` widened by one field bit | E1 |
| **CN** | a registered surface's `SURFACE[...]` marker renamed in the source | E2 |
| **CZ** | a registered surface's domain emptied | E3 |
| **CU** | a new boundary-named `const` added to `c2-core` and not registered | E4 (ceiling exceeded) |
| **CD** | a dated construct rung with no `Fail axis:` field | E5 |

`#3336`: *a control you have never watched FAIL is decoration.* Each row is
recorded with its failure text in `work/w-doctrine/controls_red.txt` and the
tree verified clean after each.

## 6. What this lane will NOT do

* **Not change `gate.sh`'s verdict and not add a count-bearing row.**
  `gate_identity_diff.sh` selects rows by SHAPE and excludes by HARD-CODED
  NAME; a 22nd row makes it **exit 2, refusing to diff at all**, for every live
  lane holding a 21-row base (`#3691`, four peer lanes live).
* **Not license an emit.** `docs/FUNCTION_BYTE_MATCH.md` §0's separation binds:
  this is a progress/method instrument and it is never in `gate.sh`'s verdict.
  The sole judge stays real `c2.dll` under wibo plus a byte-exact obj compare.
* **Not weaken the byte delta.** It is necessary and not sufficient; this lane
  adds, it does not replace.
* **Not add a crate dependency.** std only under `crates/`.
* **Not touch another lane's board block or the reservation ledger line.**
* **Not claim registry completeness.** E4 is a ratchet on a hole, not a proof
  that the hole is closed, and the rung will say so in those words.

## 7. What would make this lane say `FAILED`

Any one of:

* **E1 does not fire on C6** — the motivating defect passes the instrument
  built to catch it;
* the gate or the identity diff **moves** under C6, which would mean the
  demonstration was never about an invisible widening;
* a part ships without having been watched red;
* the blast radius turns out to be a peer-tree failure and cannot be scoped
  away — in which case the honest outcome is `declined` with the price, per the
  brief.
