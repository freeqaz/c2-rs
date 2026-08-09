# w-callprice — PREREG

**Frozen before the first workload scan and before the first line of the scratch
instrument.** Lane `w-callprice`, worktree branch `wt-w-callprice`, off master
`c5ff9953` (the w-jump merge).

This is a **PRICING** lane. It ships no accepted class, no recognizer and no
`crates/` line. Its output is a priced, ranked list of the next rungs on the
`expr-call-in-expr-*` family **by the emitted column, with TU replication
discounted** — or the honest statement that none is worth a lane.

---

## 0. Why this lane exists, stated before it measures anything

`docs/rungs/2026-08-09-w-jump.md` §7.2 (board **#2007**, ROADMAP §10.26.6) priced
the whole `expr-jump` loop family at zero and left **one** lever standing:

> **(R3) A call inside a loop body.** … **This is the only real lever and it is
> not a reader rung and not a loop rung.** It is `expr-call-in-expr-*`, which at
> base already carries `46,036` bodies / `1,033` emitted on its single largest
> key (`-op-0x9B`) … **A lane on it should be priced from *that* key, not from
> this one**, or it will re-run the mistake this lane was commissioned to
> correct.

So the question is fixed in advance and it is **not** "what is the biggest key".
It is: **on the emitted column, with replication discounted, is there a rung on
this family worth a lane, and is it a reader admission or a genuinely new
lowering** (ROADMAP §10.26.4's distinction).

Seven blocked-key **size** rankings in a row have turned out to be artifacts
(memory: *"Ranking instruments measure themselves"*, four; plus w-bdnz, w-mcall,
w-jump — the seventh being a NEW mechanism, a key inflated by **TU
replication**, `bodies == TUs`). This lane assumes it is the eighth until the
measurement says otherwise, and registers a **pessimistic** headline (P12) for
exactly that reason.

## 0.1 What is already known, quoted from the tree rather than re-derived

Every number in this section is read out of a committed document **before** this
lane runs anything, so that P1–P4 are predictions and not observations.

| source | number |
|---|---|
| `rungs/2026-08-08-w-value.md` (tip `c5c94058`) | family **423,925 bodies / 35,583 emitted** — a **redefinition** of the key, both sides published (#1948) |
| `rungs/2026-08-08-w-mcall.md` | `…-recv-load-whole` **18,310 → 18,290 / 1,505 → 1,498**: the family loses **−20 bodies / −7 emitted** |
| `IL_CALL_IN_EXPR.md` §28.1 | of the OLD 36,751 emitted: **33,277 (90.5 %)** nothing else in the expression · **2,306 (6.3 %)** untokenizable · **1,168 (3.2 %)** head moved |
| `rungs/2026-08-08-w-value.md` §4.3 | the walker's stopping bytes: `9B` **50,023 / 1,590**, `64` **2,999 / 546** — 93 % of the emitted price in two tokens |
| `rungs/2026-08-08-w-mcall.md` #1963 | the sequence route's refusal, **on bodies**: `call-ref` **125,458** · `call-token` **25,060** · `this-undetermined` 4,139 · `expr` 2,124 · `result-type` 1,411 · `formals-marker` 830 |
| `rungs/2026-08-09-w-jump.md` §6 | the whole blocked emitted column is **130,560** symbols over **1,751,929** bodies |
| `IL_CALL_IN_EXPR.md` §14.5, §18.4, §20.x | the key is de-sharded and exact-partitioned; `op-0x9B` is measured by capture |

**Nothing in that table is an emitted-column measurement of the `prod` axis.**
`work/w-mcall/proddiff_scratch.txt` is a **body** column. That cross —
`prod × emitted`, on this family — is what this lane's instrument is for, and it
has never been taken.

---

## 1. The instrument, described before it is written

**One scratch instrument, at the two sites that count blockers**
(`crates/c2-harness/src/gap/scan.rs`), on w-jump's §2.1 pattern: replace
`f.verdict.key()` with a **compound key** that carries every offline axis, so a
single scan answers the whole question and **both columns sum to the family
total by construction**. Restricted to keys starting `expr-call-in-expr` so the
rest of the census is byte-identical.

Axes put in the key, and why each is there:

* `cflow`, `dispatch`, `prod`, `calls`, `seg_len` — the five census axes that
  already exist per function. **`prod` is the one that matters**: it is the axis
  that separates *"a construct the port has no production for"* from *"a private
  limit inside a production that already ships"* (`census.rs`'s own doc comment),
  which is exactly ROADMAP §10.26.4's admission-vs-lowering distinction, and it
  has never been crossed with the emitted column.
* `hex` + `hex_mark` — the census window, for reading bodies back.
* `index` + `name`/`emit_name` — the TU-local identity, for the replication
  check and for locating the source.

**The replication check** (w-jump's new mechanism, board #2000): a body column
counts **segments**, not constructs. Its emitted-column analogue is the
**mangled name**: an emitted symbol that is a COMDAT header inline appears once
per TU that emits it, so `emitted == distinct TUs` for one name is the same
finding one column over. Reported per top group as **emitted / distinct names /
distinct TUs**, beside the body counts.

**Reverted before the gate.** Zero committed `crates/` changes; the diff is
quoted in the rung and stored at `work/w-callprice/scratch.patch`.

### 1.1 The budget

Three workload scans at most (base, instrumented, and one counterfactual if a
shipped env-gated sink can answer a question without a patch). `gate.sh` is not
re-run if `git diff master -- crates/` is empty, on w-jump §9's stated grounds.

---

## 2. PREDICTIONS

Scored in the rung. Each has a probability written down before the scan.

| # | p | prediction |
|---|---:|---|
| **P1** | 0.75 | The family re-derives at base to **exactly 423,905 bodies / 35,576 emitted** — w-value's 423,925 / 35,583 less w-mcall's −20 / −7, with w-bdnz, w-jump, w-front3, w-midrun, w-mixkind, w-mrslot, w-prod and w-disagree having moved it by **zero**. |
| **P2** | 0.95 | Both columns of the decomposition sum **exactly** to the family total, and the script **asserts** it rather than printing it. |
| **P3** | 0.70 | The largest key on the **emitted** column is `expr-call-in-expr-recv-load-whole`, at **1,498 ± 5**. |
| **P4** | 0.55 | `expr-call-in-expr-op-0x9B` is **second** on the emitted column, at **1,033 ± 20**. |
| **P5** | 0.60 | The top three emitted keys together are **< 15 %** of the emitted column — i.e. the emitted column of this family is **more** shattered than `expr-jump`'s body column was, not less. |
| **P6** | 0.50 | **≥ 40 distinct keys** are needed to cover 50 % of the emitted column. |
| **P7** | 0.55 | For at least one of the top-3 emitted keys, the count of **distinct mangled names** is **< 60 %** of its emitted count — i.e. replication is present in the emitted column too. |
| **P8** | 0.60 | But it is **materially weaker** than in `expr-jump`'s body column: **no single mangled name** accounts for **≥ 5 %** of the whole family's emitted column. (`__stl_hash_string` was 36 % of `expr-jump`'s bodies.) |
| **P9** | 0.70 | `prod-call-ref` is the **largest** `prod` tag on the family's emitted column. |
| **P10** | 0.60 | `prod-call-token` — w-mcall #1963's *"the seam's own next step"*, **25,060 bodies** — is **< 500 emitted**, i.e. the body column over-prices it by **≥ 10×**. Registered as a pessimistic prediction about a number this project has already published in the optimistic direction. |
| **P11** | 0.55 | Of the three top emitted keys read back to source, **≥ 2** need a **lowering** (an `IlOp`/emitter representation that does not exist), not a **reader admission**. |
| **P12** | 0.60 | **The recommendation is a DECLINE**: no rung on this family converts **≥ 100** emitted functions at reader-admission cost. Registered pessimistically and deliberately, on w-jump's P14 precedent. |
| **P13** | 0.75 | **≥ 1** top emitted-column key turns out to be **different constructs wearing one key** (`GAPS.md` §6), demonstrated by reading, not by counting. |
| **P14** | 0.90 | **No `crates/` change is committed**, and `cargo test --workspace --release` is **digit-for-digit** master's published tip. |
| **P15** | 0.70 | **≥ 1 unnamed refusal fires.** Pre-armed at two places: **(i) KEY CARDINALITY** — this family is ~185× `expr-jump`'s body column, so a hex-bearing compound key may blow the scan's memory or its JSONL size past what one pass can hold; **(ii) EMITTED ATTRIBUTION** — the emitted column binds a *name* to a *body record*, and the window this lane reads may not be anchored on the member call's own `26`, which would make a per-key hand-read of the emitted column read the wrong bytes. |
| **P16** | 0.50 | The **TU-replication discount changes the ranking**: at least one key in the emitted top five **moves position** once ranked by distinct names instead of raw emitted count. |

### 2.1 Registered directions

Stated so the miss direction is scoreable against board #770's optimistic streak
and against the seven-instance ranking lesson.

* **Pessimistic:** P5, P6, P10, P12 — the family is expected to be *more*
  shattered, and its named next step *smaller*, than the record says.
* **Optimistic:** none about size. The only optimistic-shaped prediction is
  **P16** (that the discount is load-bearing, i.e. that the lane's own
  instrument earns its keep).
* **Structural, not size:** P2, P9, P11, P13, P14 — these are about *what the
  keys mean*, and are the ones the deliverable actually rests on.

If P12 **misses** — i.e. a rung ≥ 100 emitted survives discounting — the lane
owes ROADMAP **§10.26.7**, because that would reorder §10.26.6's close. If P12
hits, §10.26.7 is **not** written and the rung says so.

---

## 3. DECLINE CLAUSES — frozen

**D1 — no `IlOp::Call` variant is proposed.** Inherited verbatim from
`work/w-mcall/PREREG.md` D1: it would be a **second representation of a call**
beside `SeqCall`, and `docs/GAPS.md` §6 instance #9 is one rule with two
implementations, paid for four times in this project. If a priced rung needs
operand-position call lowering, it is named **as a lowering rung with its own
cost**, never smuggled in as a reader admission.

**D2 — no rung is recommended on a population quoted from the body column
alone.** Every price in the deliverable is an **emitted** count with the
replication discount stated beside it. A body count may appear only as context,
labelled as such. This is the whole commission.

**D3 — nothing from `docs/whitebox/` is adopted.** `WB_READER_FINDINGS.md`'s
operand table is **navigation only**, cited in prose; no constant reaches
`crates/`, and `crates/` is untouched regardless. No `DISCLOSURE.md` row is
carried.

**D4 — the lane declines rather than ranking the largest remaining thing.** If
the top surviving population is under 100 emitted functions, the deliverable is
the negative, stated plainly, and **not** a rung on the biggest of the small
rows.

**D5 — no key is quoted from a scan that is not this lane's own.** Every number
in the deliverable is re-derived at this lane's base. Inherited numbers appear
only in §0.1 above, labelled as inherited.

---

## 4. What would falsify this lane's own method

If the instrumented scan's two columns do **not** sum to the family total
(P2 misses), every table below it is void and the rung says so rather than
publishing a table with a residue row. If the emitted-column hand-reads
(§ deliverable) cannot locate a body's source in the dc3 tree for at least three
bodies per top key, the key is reported as **unread**, not as "read by
inference".
