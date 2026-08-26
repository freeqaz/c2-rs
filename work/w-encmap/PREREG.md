# w-encmap — PREREG

Lane `w-encmap`, wave 14, `docs/DECISIONS_2026-08-22.md` decision 16.
Board rows **#3636**–**#3641**. Base: master `e548f01fd`, branch `wt-w-encmap`.

**Committed before the first measurement.** Nothing below is edited afterwards;
§6 is the grading section and it is appended, never a rewrite.

---

## 0. What this lane is asked for

1. Adjudicate whether `crates/c2-core/src/codegen/encode.rs` and
   `crates/c2-core/src/codegen/mop.rs` are **two independent readers of one
   fact** (this repo's most-recorded silent merge failure) or **two layers of
   one**.
2. Adjudicate **`#3634`** — its "the black-box re-derivation was retired"
   is a census **of named constants**; `encode.rs`'s module doc and
   `P_ENCODE.md` §8.1 both still describe the **helper functions** as
   black-box-derived. Decide which reading is right, correct whichever side
   is wrong.
3. Build the **arm → port-function map** over `P_ENCODE.md`'s 79 arms.
4. Publish `ported` on the `encode` row of `c2rs subsys`, with its
   denominator named.

## 1. Declared before measuring — what orientation ALREADY established

Honesty rule: these are **observations already made during orientation**, not
predictions, and they are recorded as observations so that §6 cannot claim
credit for them as hits.

* `encode.rs` lines 1..2006 (the non-`cfg(test)` half) contain **zero**
  `to_be_bytes` and every `<<` is inside a doc comment. Word composition in
  the live half therefore goes through `MachineOp::word()` only.
* `mop.rs::plan()` already cites, **per form, the address of the c2 arm it was
  read from**. The map is half-built in code; this lane extracts and grades it
  rather than deriving it afresh.
* `encode.rs` carries `#[cfg(test)] mod incumbent` (line 2381) and
  `#[cfg(test)] mod cross_check` (line 2920).
* `mop.rs`'s `OPCODES` doc says *"71 of c2's 660 rows"*. The table looked
  longer than 71 rows on a first read. **Not yet counted.**

## 2. PREDICTIONS — never edited after this commit

### P1 — the duplicate-reader verdict

**Predicted: NOT a duplicate reader. Two layers of one fact.** `encode.rs` is
the *selection/naming* layer (which c2 opcode number, which operand plays
which role); `mop.rs` is the *composition* layer (base word + field
placement), and it is the only one that makes a word.

* **P1.a** — exactly **one** live composition path exists on the emit path.
  Confidence 0.90.
* **P1.b** — the only second producer of a word is `#[cfg(test)] mod
  incumbent`, which is **deliberate**, cannot be called from non-test code,
  and has an **armed** detector (`mod cross_check`). Confidence 0.85.
* **P1.c** — they therefore **cannot disagree silently**; drift is caught by
  `cross_check` at `cargo test`. Confidence 0.75. *Registered risk:* if
  `cross_check` covers materially fewer than all incumbent encoders, the
  detector is partial and P1.c downgrades to a qualified yes.
* **P1.d** — **no word is produced twice by different rules on the emit
  path.** Confidence 0.90.

**Decline floor for this deliverable:** if a live second composition path
exists, the lane **reports it and files a board row and does not fix it** —
a fix is an emit change and needs its own two-sided price (charter, hard
constraints).

### P2 — the `#3634` adjudication

**Predicted: BOTH READINGS ARE TRUE OF DIFFERENT OBJECTS, and the real defect
is a third thing neither names** — `encode.rs`'s module doc contains the
pre-read paragraph (*"This file is a black-box re-derivation … nothing here
changes"*) **and** the `w-s1` retirement notice roughly 25 lines below it,
with the first never struck. Both `#3634` and the wave-14 brief are quoting
the same doc comment and reaching opposite conclusions **because the doc
comment contradicts itself**.

* **P2.a** — `#3634`'s census (9 `[S]`, 2 `[O]`, 0 `[F]`) is right *about
  named constants*. Confidence 0.80.
* **P2.b** — the module doc's opening paragraph is **stale** and is the thing
  that should be struck. Confidence 0.75.
* **P2.c** — `P_ENCODE.md` §8.1's *"accumulated 89 PPC words one captured obj
  at a time, never looking at `c2.dll`"* is a **true statement about how the
  port's opcode/role choices were originally obtained** and stays true even
  after `w-s1`; what changed is where the *bits* come from, not where the
  *choice of opcode* came from. So §8.1 is **not** wrong and must not be
  struck — it needs a scope sentence, not a correction. Confidence 0.60.
* **P2.d** — **the fix site is in a file this lane may not write**
  (`crates/c2-core/src/codegen/**` is `w-disclose`'s comment-only fence), so
  the outcome is **STOP-and-report** for the encode.rs half, and an
  amend-beside in `P_ENCODE.md` (owned) for the §8.1 half. Confidence 0.85.

### P3 — the arm → port map

**The denominator this lane will publish `ported` against is the 79 arms**,
declared here before counting, with the reason: `read` on the encode row is
already `79 distinct encode arms`, so `read ⊇ ported` is only well formed in
the arm unit. The band's `14` is a **different unit** (Ghidra function
entries) and `SUBSYS.md`'s `14 / 14` cell is not a coverage fraction —
`w-submetric` already recorded the 5.6× trap on this exact row.

**The mapping rule, fixed before measuring:** an arm is `ported` iff the port
has a `FieldPlan` (i.e. `mop::plan(form)` is `Some`, or the form composes in
code inside `encode_op`) for **at least one** of the forms that arm serves.

* **P3.a** — predicted **26 of 79 arms** map (32.9 %). Interval 20–34.
* **P3.b** — **stated bias direction: this rule OVER-counts.** Granting an
  arm on one of its forms credits an arm the port serves only partially. The
  lane will therefore publish the **strict** variant (every form of the arm
  planned) beside it, and predicts strict < lenient. Confidence 0.80.
* **P3.c** — the **unmapped** half is the load-bearing half and is predicted
  to be dominated by **VMX/VMX128** forms plus the default arm `10bf9f91`
  (104 opcodes, the single largest arm by opcode count). Confidence 0.70.
* **P3.d** — predicted **≥ 60 of the port's `OPCODES` rows** land on an arm
  that is in the mapped set (an opcode-unit cross-check of the arm-unit map).
  Confidence 0.70.
* **P3.e** — `mop.rs`'s *"71 of c2's 660 rows"* comment is predicted **STALE
  and low** — the table has grown since. Confidence 0.70. *(If it is stale it
  is in a file this lane may not write → STOP-and-report.)*
* **P3.f** — `P_ENCODE.md` §8.1's `89` and the file's actual
  `pub fn encode_*` count are predicted **NOT equal** today. Confidence 0.55.

### P4 — the instrument

* **P4.a** — `ported` on the encode row will be published as a **live
  recount**, not a carried constant: `subsys.rs` will ask `c2-core`'s own
  `mop::plan()` for each arm's forms, exactly as the band denominators are
  recounted from `FUNCS.tsv`. A fabricated constant then reddens by
  construction. Confidence 0.85.
* **P4.b** — a **fifth positive control** is added and **watched going red**
  before its green is quoted (`#3336`).
* **P4.c** — the other nine rows keep their named residues; **no bare zero is
  printed for any strength** (existing pinned invariant).
* **P4.d** — `cargo test --workspace` target count at tip is predicted
  **unchanged** from base, and test count predicted **+1 or more** (the new
  control). Predicted byte delta **ZERO** — this lane writes no `crates/`
  emit-path byte.

### P5 — outcome word

Predicted `Outcome: instrument`. A `declined` is legitimate and is the
registered alternative if the map cannot be built without inventing a
population.

## 3. What this lane will NOT do

* Not re-read the 79 arms. `docs/whitebox/READ_PLAN_2026-08-21.md:100` puts
  them in its **already-read** half (lane `w-read-r2`, board `#3376`), and
  `P_ENCODE.md` §5 holds the reading. No read campaign is funded here.
* Not rebuild `P_ENCODE.md` §8.1's 89-vs-82 inverse, nor re-take §8.2's
  99.38 % / 634,457-word grade.
* Not change what the port emits. Not mint a `DISCLOSURE.md` row
  (`w-disclose`'s namespace). Not add a second provenance or census reader.
* Not edit `encode.rs` or `mop.rs`, in code **or in comments**.

## 4. Denominators to re-measure on this tree (none carried on faith)

The coordinator has **not** verified any of these. Each is re-measured here
with the workload/tree stamp beside it:

| figure | as quoted | source to re-measure from |
|---|---|---|
| 111 jump-table entries | brief, `P_ENCODE.md` §4 | `ref/ENCODE_ARMS.txt` + `ENCODE_OPCODES.txt` |
| 79 distinct arm targets | brief, `#3376` | `ref/ENCODE_ARMS.txt` row count |
| 89 port encoders | `P_ENCODE.md` §8.1 | `grep -c` on `encode.rs` |
| 82 of 89 identical | `P_ENCODE.md` §8.1, `#3379` | **cited, not re-taken** |
| 71 `OPCODES` rows | `mop.rs` doc comment | count the table |
| 14 band entries | `SUBSYS.md` §1 | `FUNCS.tsv` recount (already automated) |

## 5. Gate and identity

* Base-to-tip **required-zero byte delta**, graded by
  `scripts/gate_identity_diff.sh`; the result is quoted in the rung.
* `scripts/gate.sh` run **detached** at tip, bounded wait, `GATE:` verdict
  line quoted, never the exit code. A killed run is **NO-RESULT**.
* `cargo test --workspace` **test count and target count** at base and tip.

## 6. Grading

*(appended at close; predictions above are frozen)*
