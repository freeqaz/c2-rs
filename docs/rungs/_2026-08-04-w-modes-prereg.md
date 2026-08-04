# w-modes — PREREGISTRATION

    Slug:   w-modes
    Date:   2026-08-04
    Lane:   w-modes (`wt-w-modes`), branched at master `caff20d`
    Seam:   `scripts/` only. `crates/` is NOT touched.

Written **before** any measurement was run. Predictions are recorded so that a
result which merely confirms whatever came out cannot be presented as a finding.

---

## 1. The question

Two correctness instruments exist and their **product** is ungraded:

* `scripts/gate.sh` → 12 mode lanes (`scripts/lanes.txt`) over the 228
  hand-written `fixtures/cpp/*.cpp`. Broad in flags, narrow in shapes.
* `scripts/expr_sweep.sh` → 14,635 generated cases from `scripts/sweep.d/`,
  graded by `c2rs diff`, which hardcodes **one** profile (`/Ox /GS- /c`).
  Broad in shapes, one flag profile.

Both defects found today live in the product: board #232 (implicit destructor ×
packed path) and w-order's **Y-a** (`/EHsc` `eh_bare` slot × empty-bodied
locally-defined unwind target, live at `/O1 /EHsc`, invisible at `/Ox`).

The naive cross is 14,635 × 12 = **175,620** gradings per gate run. The
deliverable that decides the shape of everything else is a measurement:

> **How many of the sweep fragments can actually differ across flag profiles,
> and by how much?**

## 2. What "can differ" means here — the soundness criterion

The correctness rule is `port(IL) == c2(IL)` byte-exact. So for one generated
case `c` and two lane profiles `p`, `q`:

* the **port's** entire input is `(IlBundle, obj_name, gy)` — it never reads the
  cl flags (`gap.rs:1534` derives `gy` from them and hands it in; the opt mode
  arrives inside the IL as `f.opt_word`). `obj_name` is derived from the source
  name and is equal at both profiles.
* the **reference's** output is measured directly.

Therefore:

> **IL(c,p) == IL(c,q)  ∧  gy(p) == gy(q)  ∧  refobj(c,p) == refobj(c,q)
> ⟹ grading `c` at `q` after `p` establishes nothing.**

That is a proof of redundancy, not an observed coincidence, and it is the only
exclusion this lane will accept. In particular **verdict-identical is not
redundant** — `scripts/lanes.txt` already records that `/O1 /EHsc` and `/O1`
differ in 0 verdict rows while producing genuinely different objs.

Direction of risk, stated up front: **a fragment excluded as invariant is a
fragment nothing will ever grade again at the excluded profiles.** So the
exclusion must be re-derivable by a script anyone can run, and it must be
re-derived when fragments change — not written down once.

## 3. Predictions (registered before running)

Instrument: per (case, lane), capture the reference obj at a **fixed** `-Fo`
path (so the `S_OBJNAME` baked into `.debug$S` is constant) and the IL bundle,
hash both, and partition the 12 lanes by `(il_hash, obj_hash, gy)`.

| # | prediction |
|---|---|
| **P1** | `/Od` lands in its own class, separate from every optimizing lane, on ≥ 95 % of fragments. |
| **P2** | `/O1` vs `/O2` differ in **IL** on nearly every fragment (the opt word is in the IL), so no fragment collapses to one class. |
| **P3** | `/Ox /Gy` vs `/Ox` have **identical IL** on **every** fragment — `/Gy` is a c2-side layout decision the front end does not encode — while their **objs differ** on most. This is the prediction that decides whether the IL hash alone is a sufficient key; if it holds, it is **not**, and the obj hash is load-bearing. |
| **P4** | `/EHsc` changes the IL only on fragments carrying an EH construct (a destructor, a base ctor, an object receiver): **≤ 12 of 48** fragments. |
| **P5** | `/Oi` over `/O1` changes nothing on **0** fragments — the generated corpus calls only user-defined externals, and nothing in it is intrinsic-able. |
| **P6** | The median fragment has **≥ 4** distinct classes over the 12 lanes, so **no** useful "provably flag-invariant" set exists and the cross cannot be shrunk to one profile. |

**P3 and P5 are the two that would change the design if they fail.**

## 4. Controls — and how each could go red

A control that cannot go red teaches nothing (the recorded instance: a lane
established c2's `/FAsc` listing was byte-faithful and verified it on a body
containing no relocated branch — the single class where a difference exists).

| control | how it goes red |
|---|---|
| **`63-emit-order` must NOT be reported flag-invariant** | Y-a is a *demonstrated* live wrong emit at `/O1 /EHsc` that does not exist at `/Ox`, on exactly this fragment's shape. An instrument reporting one class here is provably broken. |
| **`62-ctor-base-delegation` must NOT be reported flag-invariant** | it carries #232's shape. |
| **cell stability (`--verify N`)** | every Nth (case, lane) cell is captured a second time and the two hashes must be equal. If any run-varying byte (a temp path, a timestamp) leaks into either hash, every lane looks distinct and the measurement silently reports "nothing collapses" — the failure that would make the whole result vacuous while looking maximally conservative. |
| **at least one collapse must be observed** | if the instrument reports 12 classes for all 48 fragments, that is the same signature as the stability failure above and must be treated as suspect, not as a result. P3 is the specific collapse predicted. |
| **counts are positive and reconciled** | cells attempted vs cells recorded; a capture that fails is a named failure, never an absent row that reads as agreement. |

## 5. What this lane will NOT do

* No `crates/` edit of any kind. A defect found there is reported with a
  reproducer and routed by the coordinator.
* No `docs/BOARD.md`, `docs/STATUS.md`, `docs/ROADMAP.md` edit.
* No merge to master, no push.
* No committed scratch fragment reintroducing a fixed defect — the must-fail
  control is run and quoted, never landed, because landing it leaves the merge
  gate red for every lane.
