# PREREG — lane `w-inlclause`, wave 19

Registered **before** the image was opened and before any clause was
classified. Board **#3796**–**#3801**. Base: master `12d3c0558`, branch
`wt-w-inlclause`.

Charter: `docs/ADOPTION_BRIEF_2026-08-29.md` §L2.

> *"C3 and C19 converted because `P_INLINE` §6.6.2 had been read to address
> level and the port could be derived from it. Ask the same question of the
> other 15: is there an existing read the port could be derived from, or is the
> row `absent` because nothing has been read yet? Those are different states and
> the table cannot currently tell them apart."*

---

## 0. What I had already read when this file was written

Registered because it is what makes the predictions in §5 honest rather than
retrodictive. Before writing this file I read, in the working tree and **not**
in the c2 image: `CLAUSES.tsv`, `check_table.py` (including
`token_in_crates`'s docstring), `crates/c2-core/src/splice.rs` lines 1–564,
`crates/c2-harness/tests/clause_table.rs`, `P_INLINE.md` §6.0–§6.2 and
§6.6–§6.6.3, `docs/rungs/2026-08-28-w-inlbudget.md`, and commit `72caf2586`.

From that reading I already hold three priors, and they are registered here as
priors rather than presented later as findings:

* **C14** — `splice.rs` carries `INLINE_LEVEL_DEPTH_CAP = 16` with a `PROV[R]`
  address and `BudgetModel::declines_at_depth`, and the `w-inlbudget` rung
  names it *"`S6-budget-depth-cap` (C14)"* in those words. The row still reads
  `absent` because the token it cites is a **different spelling** from the one
  that was adopted.
* **C18** — `BudgetModel::charge`'s `callee_instrs > charge_exempt_at_or_below`
  is the same `cmp eax,0x28` the row's own `addr` and `asm` cells name.
* **C4** — `Expansion::at_pass_entry()`'s doc transcribes the six-argument call
  the clause describes, at the clause's own repaired address.

**If those three hold, the state that moved them is not "nothing was read".
It is that the ABSENCE screen is a NAME screen in the OTHER direction too** —
it cannot see a counterpart that was adopted under a name the table does not
cite. `#3641` and `token_in_crates`'s docstring both declare the
mention-as-false-positive half. The false-**negative** half is not declared
anywhere, and this lane will either demonstrate it or retract it.

## 1. The clause list, frozen

The **15** rows whose `state` is `absent` on `12d3c0558`:

```
C1 C2 C4 C5 C6 C7 C9 C10 C11 C12 C14 C15 C16 C17 C18
```

Denominator 15, of 24 rows, of which 21 are reachable (3 `unexercisable`).
**No row outside this list may change `state`.** Adding a row to this list
after seeing an answer is the thing this file exists to forbid.

## 2. The trichotomy — fixed now, applied later

A new `read` column on `CLAUSES.tsv`, one of exactly three values. It is
orthogonal to `state`: `state` says whether the **port** has a counterpart,
`read` says whether **this project** has read the clause well enough to build
one.

| value | means |
|---|---|
| **`R1`** | **read, and derivable.** An address-cited read exists in the §3 corpus from which a port counterpart could be written today with every field carrying a `PROV[R]` address — the level `BudgetModel::seed` was derived at. |
| **`R2`** | **read, not derivable.** An address-cited read exists, and a **named** link between c2's quantity and anything `crates/` can compute is missing. The blocker is named in the `blocker` cell and cited. |
| **`R3`** | **unread.** Nothing in the §3 corpus reads the clause beyond restating it. |

**What counts as "a read"** (fixed now, so it cannot be loosened later): an
address-cited passage in the §3 corpus that goes **beyond restating the
clause** — it names at least one address *other than* the row's own `addr`, or
it enumerates the readers/writers of the datum the clause tests. A passage that
only repeats the clause in prose is **not** a read, and the row is `R3`.

Two further columns:

* `readcite` — for `R1`/`R2`, `path#anchor`; the grader requires the path to
  exist and the anchor to be present in it. For `R3`, `-`.
* `blocker` — for `R1`: `none` (adoptable) or `emit-change` (derivable and out
  of this lane's scope). For `R2`: a token naming the missing link. For `R3`:
  `unread`. For `unexercisable` rows: `n-a`.

**`R3` IS NOT CHEAP.** A universal negative over a corpus is the easiest thing
in this file to assert and the hardest to check, so every `R3` row must carry a
recorded, reproducible search in `work/w-inlclause/UNREAD_EVIDENCE.md` — the
row's `addr`, the datum its clause names, and the hit list over the §3 corpus —
and the grader requires that section to exist. This makes `R3` cost the same as
`R1`.

**Ties break AWAY from `R1`.** `R1` is the only value that licenses an
adoption, so a row I cannot place confidently is `R2`, and a row whose read I
cannot cite is `R3`.

## 3. The corpus, frozen

A read outside this list does not count, and I may not widen it after seeing an
answer.

```
docs/whitebox/ref/P_INLINE.md
docs/whitebox/ref/FUNCS.tsv
docs/whitebox/WB_INLINE_FINDINGS.md
docs/whitebox/WB_INLSWITCH_FINDINGS.md
docs/whitebox/WB_LOWERBAND_FINDINGS.md
docs/whitebox/WB_CANDID_FINDINGS.md
docs/INLINE_PREDICATE.md
work/w-inlmetric/  work/w-inlfit/  work/w-inlbudget/
work/w-inlswitch/  work/w-clausefix/  work/w-lowerband/
```

The c2 image (`sha256 c80981c0…a66258`) and the independent objdump listing are
**not** corpus — they are the *verifier*. Any row I place `R1` is re-derived at
its address against the listing before the placement is quoted, because a
citation I only relayed is not a read (`docs/STATUS.md`'s standing rule, and
this repo is 4-for-4 on relayed rows decaying).

## 4. What each outcome licenses

1. `state` moves `absent` → `R-derived` **only** when the port carries a
   counterpart **today** whose fields cite addresses, and `check_table.py`'s
   WITNESS check passes on the new `path:token`. No new `crates/` code is
   required for such a move — the row was stale, not unconverted.
2. A new counterpart may be **written** only if its byte-neutrality is a
   **property of the port's admitted set**, argued the way `w-inlbudget`
   argued the divisor (`n = 1` at every link, so c2's division is the
   identity). A byte-neutrality that has to be *measured* rather than *argued*
   is not admissible here, because `#3723` proved the byte judge is blind to
   exactly this class.
3. A clause whose adoption changes any emitted byte is **named and stopped**
   (`blocker=emit-change`), per the brief. It does not become `R-derived`.
4. A row whose honest answer is `R3` is a **complete result** and is reported
   with the read that would be needed. `#3505` is five-for-five against lanes
   that moved a number by constructing one.
5. **`INLINE_UNBOUNDED_BYTES` and `128` are not touched** (`#3732`).
6. No new `gate.sh` row (`#3691`).

## 5. Predictions, registered before the classification

| # | prediction |
|---|---|
| **P1** | **2–4** of the 15 have a port counterpart today under a name the table does not cite, and move `absent` → `R-derived` with **zero** new `crates/` logic. §0's three are the named candidates. |
| **P2** | **≥ 5** of the 15 place `R3`. |
| **P3** | **≥ 3** of the 15 place `R2` with a blocker that is *the same* blocker — the port has no pre-codegen instruction count — making one missing link, not many, the reason the column is stuck. |
| **P4** | **0** clauses are adopted that change an emitted byte. Identity diff **0 lines over 21 rows**; `gate.sh` still 21 count-bearing rows. |
| **P5** | The split moves **only** `absent` → `R-derived`, by **2–4** cells. No row becomes `fitted`, and `unexercisable` is unchanged at 3. |
| **P6** | At least one thing in this prereg, in `P_INLINE` §6.1, or in the brief is **refuted**. Named on the day: I expect §6.1's table to have at least one row whose `addr` is DECODE-green and still does not pin its clause, because `asm` was recorded from the address rather than the address chosen from the clause. |

The prediction I most expect to be wrong is **P2**. Every one of these 24 rows
sits inside a function some lane has listed, so "unread" may turn out to be
rarer than the `absent` column's size suggests — in which case the answer to
the brief's question is *"almost none of them are absent because unread"*, and
that is a **more** useful result than a large `R3`, not a failed lane.

## 6. Controls (`#3336`)

* The `read` column's grader (`work/w-inlclause/read_state.py`) is watched
  **RED** on planted defects — one per rule — before any green from it is
  quoted, and the transcript is committed.
* `check_table.py` is re-run after every `CLAUSES.tsv` edit; its own planted
  control (`--plant C16=10b5c06b`) is re-watched RED on my tip.
* If `crates/` changes at all, `scripts/gate_identity_diff.sh` against
  `work/w-inlclause/gate_base.out`, taken on the clean tree at `12d3c0558`
  before any edit.
* Any mutation control restores with `cp`/`mv` **followed by `touch`**
  (`#3767`), and the restored tree is re-verified GREEN.

## 7. Seam

OWN: `crates/c2-core/src/splice.rs`, `work/w-inlmetric/CLAUSES.tsv`,
`work/w-inlclause/`, `DISCLOSURE.md` (append), `docs/BOARD.md` rows
`#3796`–`#3801`, `docs/rungs/2026-08-29-w-inlclause.md`. If the split moves,
`crates/c2-harness/tests/clause_table.rs`'s `SPLIT` constant moves with it —
that file is the table's compiled guard and belongs to whoever owns the table.

MUST NOT TOUCH: `crates/c2-core/src/surface.rs`,
`crates/c2-core/src/codegen/mop.rs` (`w-fmadd`),
`docs/whitebox/ref/P_INLINE.md` (`w-paramfill`),
`docs/whitebox/ref/P_GLOBREGS.md` (`w-globarms`), `docs/STATUS.md`,
`docs/rungs/INDEX.md`.

**A surface row, if one is needed, is a paste-ready block in the rung** and not
an edit to `surface.rs`.

**Never spell an `absent` row's token in prose inside `crates/`.** Clauses are
named by id here and everywhere.
