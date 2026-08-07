# w-splice — HANDOFF, mid-rebase onto master `91a47df5`

**Status: a `git rebase` is IN PROGRESS and PAUSED. Do not `git rebase --abort`
— 4 of 27 commits are already applied and two non-trivial conflict resolutions
are in them.**

Worktree: `<worktree>`
Branch: `wt-w-splice` · onto `91a47df5` · stopped at commit **5/27**,
`664e114a` *"read S3 off the IL for a Seq…"*.

---

## 1. What this lane shipped (done, verified twice, not in dispute)

**SPLICE-0-PORT** — `crates/c2-core/src/splice.rs`. When the port's whole
emitted body for `F` is one call to a same-TU callee the port lowers, `F`'s
`/Gy` COMDAT **is** that callee's — text, relocations, data references — and `F`
acquires **no REL24 against its callee**. Nine clauses S1–S9, none of which
reads the reference obj. Full write-up: `docs/rungs/2026-08-08-w-splice.md`.

Measured twice (pre-rebase vs `cda124c`, and after the w-bytes rebase vs
`22816a5`), identical both times:

```
fnbyte-differs  −723      fnbyte-exact  +723
per (TU, FnCensus::emit_name): 723 converted, 0 regressed, 0 TUs one-ended
the rule fired 723 times, 723 of 723 byte-exact
723 of 723 relocation sets verified against the reference obj, 0 disagreements
fnbyte-exact-relocated 4664 at both ends (this lane adds nothing to #882)
gate 18/18 PASS, 0 mismatches anywhere
```

---

## 2. THE REBASE — exact state

```
onto      91a47df5   (master, after w-inl0 merge 560a494b)
applied   4 of 27    last applied: 664e114a is IN CONFLICT (not applied)
HEAD      ed3a9963   (detached, mid-rebase)
```

Applied so far: the prereg, the GRID-T freeze, `cd5cab2e` (ship SPLICE-0-PORT —
**contains the w-inl0 conflict resolution, §3**), `0ebffcde` (name the clause
that refused).

**23 commits remain in `git-rebase-todo`**, starting with `6c082160` *"close the
splice chain…"*.

### 2.1 The conflict on the table RIGHT NOW (`664e114a`)

Four blocks. **Three of them are this lane's own later commit trying to
re-apply changes the `cd5cab2e` resolution already made — HEAD is correct.**

| file | line | resolution |
|---|---|---|
| `crates/c2-core/src/splice.rs` | ~150 | **KEEP BOTH doc paragraphs.** HEAD's explains why refused rows stay (for `mentions`); theirs explains the refusal-census distinction. Complementary, not competing. |
| `crates/c2-core/src/splice.rs` | ~196 | **KEEP HEAD.** HEAD is `of_named` delegating to the new `of_rows`; theirs is the superseded inline body that calls `TuEmptyCallees::of_named` and would **drop board #980's edges**. |
| `crates/c2-core/src/splice.rs` | ~313 | **TAKE THEIRS — `Some(i)`.** This block is inside `unique_row`, and HEAD wrongly carries `definition`'s return (`Some((self.rows[i].1?, self.rows[i].2))`) there. `definition` at ~286 is already correct and must keep its `.1?`. |
| `crates/c2-harness/src/gap/fnbytes.rs` | ~336 | **KEEP HEAD** — the `of_rows` call with the three-valued `Option<Reduction>`. Theirs is `of_named(… g.as_ref().ok() …)`, which drops #980. |

After resolving: `cargo build --release --workspace`, then
`git add crates/c2-core/src/splice.rs crates/c2-harness/src/gap/fnbytes.rs`,
then `GIT_EDITOR=true git rebase --continue`.

### 2.2 Expect more of the same

Later commits will re-conflict in the same two files for the same reason (they
are this lane's own increments over regions the resolution already advanced).
**Default: keep HEAD; take theirs only where the incoming commit adds something
genuinely new.** Check with
`git show <sha>:crates/c2-core/src/splice.rs > /tmp/inc.rs` and diff the
function list.

Docs will also conflict — `docs/BOARD.md`, `docs/rungs/INDEX.md`,
`docs/STATUS.md`. Rules from the coordinator:
- **BOARD**: keep both sides' rows, numeric order. **Re-check for number
  collisions** (see §4 — this has already bitten twice).
- **INDEX**: never by hand — `scripts/gen_rung_index.sh`.
- **STATUS generated block**: keep **master's**; the coordinator regenerates
  after the merge. Merge the prose additions from both sides.

---

## 3. THE w-inl0 RESOLUTION — the part that needs understanding, not just replaying

Lane `w-inl0` (board **#980**) made `tu_empty_callees` feed **parse-refused**
rows into mechanism E's closure: a refused row whose `c2-il` dead-temporary
reader can read a `no_effect_callee` contributes one `Reduction::NoEffectCall`
edge. This lane had changed the same function to return a `TuContext` carrying
each row's `opt_word`.

**The resolution is NOT "take both iterators."** `TuContext` serves three
questions over three *different* row sets:

| asked by | which rows |
|---|---|
| E's closure (`TuEmptyCallees::of_rows`) | only rows qualifying under #980 — parsed, or refused **with** a readable `no_effect_callee` |
| `definition()` — the splice's S5/S6 | **parsed rows only**; a `NoEffectCall` row is a refused body, so there are no bytes to splice |
| `mentions()` | **every** row with an `emit_name`, parsed or not, qualifying or not |

**The third is the one a careless merge loses, and losing it is a wrong-bytes
emit rather than a missing count.** `S6-chain-truncated` refuses a splice when
the chain's last link still names a callee this TU carries. If a refused row
without a readable `no_effect_callee` were dropped from the context, `mentions()`
goes false, the clause stops firing, and the splice runs off the end of a chain
it cannot see — which is exactly the 72 relocation disagreements this lane
already had to close once.

So the constructor is `TuContext::of_rows((name, Option<Reduction>, opt_word))`,
three-valued on purpose:

- `Some(Reduction::Parsed(f))` — both mechanisms may use it
- `Some(Reduction::NoEffectCall(c))` — **E only** (#980)
- `None` — a refused row **neither** can use, still carried so `mentions` sees it

Scripts that applied this (committed, in `work/w-splice/`):
`resolve_inl0.py`, `add_980_tests.py`.

**Two unit tests pin the boundary** (in `splice.rs`, added at `cd5cab2e`):
`a_no_effect_call_row_feeds_e_and_never_the_splice` and
`a_refused_row_with_no_edge_is_visible_and_nothing_else`. They currently assert
via `tu.len()` because `mentions()` does not exist yet at that commit — **when
the commit that adds `mentions()` lands, strengthen both to assert
`tu.mentions("?g@@YAXXZ")` directly.** That is the only deliberate loose end in
the code.

---

## 4. BOARD NUMBER COLLISIONS — has bitten twice, check again

The brief allocated this lane **#986–#995**. Peers landed and took numbers out
of that range:

| lane | took | effect |
|---|---|---|
| `w-drop3` | #984–#989 | collided with #986–#989 |
| `w-inread` | #996–#1005 | collided with the first fix attempt |
| `w-inl0` | **unknown — CHECK** | may collide again |

Current allocation after the last fix: **#990–#995 and #1006–#1009**.
`work/w-splice/renumber.py` is the tool; it lists every site explicitly and
**fails hard** on a site it cannot find. Its header records why the first
attempt was wrong: a regex guard of "does this line name another lane" is
unsound, because a cross-reference need not name the lane it cites — it
corrupted w-drop3's `"Boards #984–#989"` in STATUS.

Verify with:
```bash
python3 - <<'PY'
import re, collections
n=[int(m.group(1)) for l in open("docs/BOARD.md")
   if (m:=re.match(r"^\| \*\*(\d+)\*\*<sub>", l))]
print(len(n), len(set(n)), sorted(x for x,c in collections.Counter(n).items() if c>1))
PY
```
Must print `<N> <N> []`.

---

## 5. ACCEPTANCE the coordinator asked for (none of it run yet on this rebase)

1. `cargo test --workspace --release` — 0 failed, **no shrunken target count**.
   Master baseline is **1,017 / 32 targets**; this lane adds ~20 (18 + the 2
   new #980 boundary tests) → expect **~1,037 / 32**. Quote the real number.
2. **One 878-TU scan.** Runner: `work/w-splice/run_scan.sh <tag> [jobs]`
   (needs `C2RS_DC3=<dc3>`).
3. **Disjointness — VERIFY, do not assume.** Expected `differs` =
   3,195 − 138 (w-inl0) − 723 (this lane) = **2,334**. If it is not 2,334 the
   two mechanisms overlap. Measure it per `(TU, emit_name)`:
   - scan the rebase base `91a47df5` from a detached checkout (this is how the
     w-bytes round attributed motion — do **not** subtract);
   - `python3 work/w-splice/partition.py <base>.jsonl <tip>.jsonl` prints
     converted / regressed / one-ended per symbol.
   - Overlap should be 0 because S9 refuses any function mechanism E claims.
     **If it is not 0, report the overlap set by name and which mechanism wins
     each** — do not net it out.
4. **`fnbyte-elided` must keep w-inl0's growth**, and
   `fnbyte-tu-empty-callees` must read its honest value. Run
   `python3 work/w-splice/peerkeys.py <base>.jsonl <tip>.jsonl` — it counts
   every peer lane's key family at both ends and prints anything that vanished.
   **This control already caught one real defect of this lane** (§6).
5. **Reloc verification 723/723**:
   `python3 work/w-splice/relocheck.py <tip>.jsonl` → expect
   `723 no-relocs, DISAGREEMENTS: 0`.
6. **Gate**: `scripts/gate.sh --jobs 6` → 18/18 PASS, 0 mismatch. Must be run
   **on the rebased tree** (the resolution changes code).

Other instruments in `work/w-splice/`: `why.py` (which clause refused each
residual differ), `spread.py` (family spread, #925/#952), `relowitness.py`
(names + targets of any reloc disagreement), `grade_cells.py` + `run_cells.sh`
(GRID-T).

---

## 6. Two traps this lane hit — do not re-learn them

1. **`scrub.sh` is destructive and must never run on a file something holds
   open.** It `sed -i`'d `gate_tip.txt` while the gate was still appending; a
   run that exited **0** left a log with no `GATE:` line, looking exactly like a
   hang. The gate had to be re-run.
2. **`TuContext` `Deref`s to `TuEmptyCallees`, and an inherent method SHADOWS
   the target's.** While `TuContext` spelled its definition count `len`, the
   scan's `fnbyte-tu-empty-callees` silently reported the wrong quantity —
   **88,894 → 1,474,755**, no compile error, no test failure, no gate red, and
   not one of the 80 `gap-metric` lines. The method is `definitions()` now.
   `peerkeys.py` is what found it. **Any new inherent method on `TuContext`
   whose name also exists on `TuEmptyCallees` is a silent override.**

---

## 7. Reporting owed to the coordinator

Tip sha · test aggregate · the scan partition line · **the disjointness verdict
(overlap count, names if any)** · one sentence per conflict block on what was
kept. **Do not push.**
