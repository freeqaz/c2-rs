# `w-grammarscreen` — deviations and corrections

Every departure from the frozen prereg
(`docs/rungs/_2026-08-18-w-grammarscreen-prereg.md`), and every defect found in
this lane's own instruments, recorded as it happened.

---

## D1 — the enumerator's first version returned ZERO sites, and it returned it silently

`enumerate.py`'s tokenizer captured its token text **after** advancing the
cursor (`adv(k - i)` then `text[i:k]`), so every `ident`, `num`, `char` and
lifetime token came back as the **empty string**. The parse then matched
nothing and the script printed a clean

    files scanned: 65
    sites parsed:  0

with exit 0. **That is #3288's failure mode reproduced inside the instrument
built to answer #3288** — an enumerator's wrong count is silent and, in this
case, maximally flattering to a lane that wanted to go home early. It was
caught by the reconciliation against the raw grep, which is the check the
prereg registered for exactly this and the only reason the defect did not
survive. Fixed by capturing each token's text before `adv`.

**Consequence for the record: none.** No number was published from the broken
version; the reconciliation is run on every invocation.

## D2 — `Location::column()` and the qualified-call column

`#[track_caller]` reports the column of the **call expression's span**. For
`blk(..)` that is the `blk` ident, which is what `enumerate.py` records; for
`Block::refuse(..)` the span starts at `Block`, seven columns earlier. Rather
than guess, `annotate.py` records **both** columns (`col`, `col_alt`) and
`rederive.py` admits either, then prints every hit it could not place as an
**out-of-frame hit**. Which column rustc actually emits is therefore READ off
the probe log rather than assumed, and a wrong guess would show up as 106
out-of-frame hits rather than as 106 silently-quiet sites.

## D3 — `w-mutcensus`' drop-table figure for `dyninit_tu` does not reproduce at its own base

Registered as a re-derivation, and the result is a delta, so per **#3269** the
measurement comes before the cause: `IlBundle::dyninit_tu` has **11**
`return None` clauses at **`3835469c`** — the census's own base, read with
`git show` — and **11** at `666fe6eb7`. The published figure is **12**. Since
the count is identical at both bases, **head drift is excluded** and the
published figure is wrong at the base it was measured on. `data_tu`'s **14**
reproduces exactly at both.

## D4 — the drop table is missing a whole function of the same class

`IlBundle::provide_data_tu` carries **19** `return None` clauses of exactly the
class `w-mutcensus` §2.1 dropped as "`IlBundle::data_tu` `return None`
clauses". It **did not exist at `3835469c`** (`fn_body` finds no such function
in `git show 3835469c:crates/c2-il/src/func/bundle.rs`), so this is shelf life
rather than an enumeration defect — and it is the third instance of
`w-mutcensus` F7's *"the enumeration went stale"*, this time **after**
publication rather than during the campaign.

## D5 — `OptWordMode`: 19 shape files, not 18

Nineteen shape files carry exactly one `opt_word_mode(opt_word_at(seg)) !=
Some(OptWordMode::O1)` gate each, against the drop table's **18**. Measured,
not explained; the class is out of this lane's probe frame either way.

## D7 — a `touch crates/c2-il/src/lib.rs` was issued while `P1` was running

To check whether rustc's own `dead_code` lint has anything to say about this
class (it is the only unreachability proof that scales to ~1,000 sites, and it
is already run on every build), a forced rebuild of `c2-il` was issued **while
the `P1` stage runner was live**. Cargo serialises on the target-directory
lock, so the two did not interleave, and the rebuild's input bytes were
**identical** — `touch` moves an mtime, not a byte. The cost is wall clock in
later `P1` stages, not correctness, and the registered validity check for
exactly this (**H3**: the instrumented run reproduces `N0`'s counts) is the
thing that decides whether it mattered. Recorded because a reader comparing
stage durations across `N0` and `P1` will otherwise find an unexplained one.

Result of the check itself: **`cargo build --release -p c2-il` is
warning-clean**, so no function in this crate is uncalled.

## D6 — the `--allow-dirty-crates` cost, inherited

`scripts/gate.sh` refuses a dirty `crates/`, so every probe run passes
`--allow-dirty-crates`, which **refuses the `hatch-red` row**. `w-deadsites`
§1.1 established that master's own clean-tree gate refuses that same row for
`HATCH-STALE` (#1389), and **this lane's `N0`, on a byte-clean tree, reproduces
`PASS (HATCH-RED REFUSED)`** — so the probe runs lose nothing the base run has.

## D8 — `C1b` reads 1,668 / 3, and the third failing test is this lane's own bookkeeping

The named control at the tip is **RED 1,668 / 3**, not 1,669 / 2. The failing
set is the registered pair by name —
`the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` and
`the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` — **plus
`rung_index_is_generated_and_current`**, because this lane added its rung doc
before regenerating `docs/rungs/INDEX.md`. It is not a property of the control:
the index was regenerated with `scripts/gen_rung_index.sh` immediately
afterwards and `T1` reads clean. Named here rather than filtered out of the
table, because a results table that silently drops a row it finds inconvenient
is not derived from its logs. (`w-deadsites` §6.4 hit the identical thing.)

## D9 — the probe frame necessarily misses anything a peer lands mid-lane

Peers `w-glattrs` (`crates/c2-il`'s `gl.rs` region) and `w-witness7`
(`crates/c2-harness/tests/`) are in flight. The frozen enumeration is the
1,336 sites present at **`666fe6eb7`**; a site landed after that is one this
frame **necessarily misses**, and it is not absorbed — absorbing it would
unfreeze the prereg. `w-mutcensus`' enumeration went stale **twice inside one
lane's wall-clock**, and §7 of the rung records a third instance
(`provide_data_tu`, 19 sites, landed after that frame froze). The shelf life of
a site enumeration in this repo is measured in landed peers.
