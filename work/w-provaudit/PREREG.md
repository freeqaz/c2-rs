# PREREG — lane `w-provaudit` (decision 17, board `#3667`–`#3672`)

Committed **before the first run of the instrument**. Never edited after.
Graded honestly in the rung; a MISS is said in that word.

Base: master `0dcfca959`. Worktree branch `wt-w-provaudit`.

---

## 0. What predates this prereg, stated first because it caps the score

`w-disclose` scored 11 of 11 HIT and called it a **weak** result because ten of
its measurements predated its prereg. The same discipline applies here, and
three things are already known or already measured at the moment this file is
written:

1. **`#3645`'s two halves are known findings**, handed to this lane in the
   brief: `middle_interfaces.rs:634` cites `DISCLOSURE W-EXT-1` (not a row) and
   the `EX_CLASS_TABLE` marker still says *"NO DISCLOSURE ROW EXISTS FOR THIS
   ADDRESS"*. Any P-number below that "predicts" these is **not a prediction**
   and is excluded from the score.
2. **`#3643` is a known finding** in `crates/c2-core/src/codegen/mop.rs`, and it
   is a peer's fence.
3. **MEASURED BEFORE THIS FILE WAS WRITTEN, and therefore NOT SCORED**: while
   reading `DISCLOSURE.md` for the charter I saw the sentence *"seventeen rows,
   exhaustively"* (line 300) and then counted the table: it has **21** rows
   (`grep -c '^| \*\*W-'`). That is a false count inside a provenance document,
   inside this lane's own fence, and it is a **repair this lane will make** —
   but it is an observation, not a prediction, and it is scored as neither HIT
   nor MISS.
4. **Denominators re-measured on this lane's own tree** (`0dcfca959`, clean),
   also before this file: `provenance_census.py` prints population **997**,
   tagged **777**, untagged **220**, rule marks **6**. The brief's 777 and 997
   both **CONFIRM**. Recorded as confirmations of a peer's figures, not as
   predictions of this lane's.

Everything numbered `P*` below is registered against something this lane does
**not** already know.

---

## 1. The instrument — the checkable-shape list, fixed in advance

`scripts/prose_audit.py`. Six checks that can go RED, plus one inventory that
exists to publish what the tool cannot see. This list is registered now so that
a shape added later to fit a finding is visible as such.

| id | shape | what it can say |
|---|---|---|
| **C1** | **ROW-REF** — a `W-<NAME>-<N>` token cited anywhere in the tree must name a row in `DISCLOSURE.md`'s adopted-findings table | a citation names a row that does not exist |
| **C2** | **ABSENCE** — prose asserting that *no ledger row exists* for an address, checked against the ledger's own address column | an absence claim that the ledger falsifies |
| **C3** | **SELF-COUNT** — a document stating the size of a table it itself contains | the doc's count of its own rows is wrong |
| **C4** | **BOUND-COUNT** — a prose number bound by an explicit `COUNT[<recipe>]` annotation to a population the tool can recount | the stated number and the recounted population disagree |
| **C5** | **MENTION-RISK** — a counted mark token (`PROV[X]`, or `[R]`/`[O]`/`[I]` on the `P_*.md` pages `subsys.rs::count_marks` reads) sitting in discussion context rather than annotating anything | `#3641`'s class, as a detector |
| **C6** | **ADOPTED-PATH** — every path in a ledger row's `Adopted into` column resolves in the tree | a row points at a file that no longer exists |
| **I7** | **UNBOUND inventory** — every numeric claim on a provenance surface that **no check above can reach**, counted and printed on every run | the tool's own blind spot, as a number |

### What the instrument explicitly CANNOT see — registered before it runs

* **Any claim of fact about c2's behaviour.** *"the port emits N distinct
  opcodes"* is reachable only through C4's binding; *"c2 reads the field
  through the VI32 reader"* is not reachable at all. Only the byte judge and
  the image can grade that class.
* **Whether a cited address contains the instruction claimed.** C1/C6 grade
  *citation targets*; `addrcheck.py`'s question (is address A inside the
  function the page names) is a different check and is not folded in here.
* **Whether a bound count is bound to the RIGHT population.** C4 checks the
  arithmetic of a binding a human wrote. A recipe pointed at the wrong array
  is green and wrong.
* **`file.md:NNN` line-citation staleness** — `doc_cite_audit.sh`'s stated
  LIMIT, unchanged and not duplicated here.
* **An unbound number.** This is the big one and it is why **I7 exists**: the
  default state of a prose number in this tree is *unreachable*, and a run
  that printed only C1–C6 would read as total coverage. I7 prints the residue.

---

## 2. Predictions

Bias direction is registered with every count prediction, because the charter
asks for it and because it is the part that can be wrong in an interesting way.

### The headline bias prediction

**P1 — FALSE COUNTS IN THIS TREE UNDERSTATE THEIR POPULATION.** Where a prose
count and its recounted population disagree, the prose number will be **lower**
than the truth in **at least 3 of every 4** disagreements found. Mechanism:
populations grow (85 opcodes were 71; 21 ledger rows were 17) and prose does
not get re-counted, so drift is one-directional. **HIT** if ≥ 75 % of
C3+C4 disagreements are understatements, on ≥ 4 disagreements. If fewer than 4
disagreements exist tree-wide, this is UNSCORABLE and says so.

### Counts

* **P2 — C1 (dead row-refs) finds `W-EXT-1` and AT MOST TWO others** tree-wide,
  after the pre-draft suppression class is applied. Registered range **1–3**.
* **P3 — C2 (false absence claims) finds the `EX_CLASS_TABLE` one and AT MOST
  ONE other.** Registered range **1–2**.
* **P4 — C3 (self-count) finds AT LEAST ONE further false self-count beyond the
  `seventeen`/21 already observed**, across `DISCLOSURE.md` and
  `docs/whitebox/ref/README.md`. Registered range **1–4** further.
* **P5 — C6 (adopted-path) finds ZERO dead paths.** `#3631` graded this
  direction at 13 of 13 and `w-disclose` reproduced 17 of 17 / 20 of 20; the
  prediction is that a third measurement on a third tree still finds none.
  A non-zero result is a MISS **and a better outcome than the HIT**.
* **P6 — I7's unbound residue is LARGER THAN THE SUM OF EVERYTHING C1–C6 CAN
  REACH, by at least 10×** on the provenance surfaces scanned. If this is
  false the tool is more complete than I believe and P6 is a happy MISS.

### Split between fences

* **P7 — MOST OF WHAT THE AUDIT FINDS IS OUTSIDE THIS LANE'S FENCE.** Of the
  C1+C2+C3+C4 findings, **strictly more than half** will sit in files this lane
  may not write — `crates/c2-core/src/codegen/**` (`w-mopfold`),
  `docs/whitebox/*_FINDINGS.md`, `docs/whitebox/ref/P_*.md`, `docs/BOARD.md`,
  `docs/ROADMAP.md`. Mechanism: this lane owns four files and the tree has
  hundreds.

### The `#3641` half

* **P8 — C5 finds AT LEAST ONE mention-risk site among the 777 counted
  `PROV[X]` markers or the 6 rule marks in `crates/`.** Registered range 0–4;
  a zero is a legitimate outcome and would say the crates-side surface is
  currently clean.
* **P9 — THE DISAMBIGUATION CONVENTION COSTS ZERO OF THE 777.** The convention
  registered in advance is: **a mark written inside backticks is a MENTION and
  is not counted; a bare token is a MARK.** Prediction: **zero** of the 777
  currently-counted `PROV[X]` markers are backticked, so adopting the rule in
  `MARK_RE` would leave all 777 counting identically. Proof obligation, and it
  is registered as the *condition on shipping the change at all*: if the count
  moves by even one, the `MARK_RE` change is **not shipped** and the convention
  is reported instead.
  **Registered separately: this lane will MEASURE, and NOT EDIT, the same
  question for `subsys.rs::count_marks`** — that file is `w-secported`'s fence
  this wave.

### `W-EXT-1` — the decision, stated in advance with its condition

**The condition, fixed now.** `W-EXT-1` is **promoted to a real row** if and
only if **all** of the following hold when checked against the pinned image
(sha256 `c80981c0…a66258`) and `docs/whitebox/ref/FUNCS.tsv`:

1. `0x10c1fe40` is a function entry in the image's own function table, and
2. every other address the pre-draft cites — `0x10b3d550`, `0x10b3d5a6`,
   `0x10b3d5b4`, `0x10b3d5c1`, `0x10b3d5ea` — lands **inside** a function, with
   the `0x10b3d5*` group inside one and the same function (the type reader), and
3. the bytes at `0x10b3d5c1` are an actual `shr ebx,9` / `and ebx,7` pair, i.e.
   the one clause the port's `type_len` transcription depends on is confirmed
   at the byte level, not merely cited.

**If any of the three fails, the row is NOT promoted**, the citation at
`middle_interfaces.rs:634` is repaired to point at
`WB_READER_FINDINGS.md` §3.2 / §5.3 and board `#1594` — what actually exists —
and the failure is stated as the reason. `#3626`'s precedent is the whole
point: a pre-draft carried on sight held two wrong addresses in bold for eight
days.

* **P10 — the condition WILL be satisfied and the row WILL be promoted.**
  Registered because it is the outcome I expect and it must be scorable
  against; `#1594` calls the finding obj-confirmed and `FUNCS.tsv` already
  carries `10c1fe40` at size 93.
* **P11 — the `EX_CLASS_TABLE` marker repair and the `W-EXT-1` disposition land
  in the SAME COMMIT**, per `#3645`'s *"both repairs land together or not at
  all"*.

### The required-zero identity

* **P12 — identity diff 0 lines over 21 rows, base to tip**, and
  `--self-test` green beside it. Every `crates/` edit this lane makes is
  comment-only and in `crates/c2-reference/**` only.
* **P13 — the census's 777/997 is UNMOVED by this lane's own edits**, and
  `--since 0dcfca959` shows `→tag 0`, `→untag 0`, `reclass 0`, `+new 0`,
  `-gone 0` for every module. (A comment-only edit that changed a marker would
  show up here, which is the point of using `--since` as the proof rather than
  asserting stability.)

### `#1406`

* **P14 — the census will NOT be wired under `cargo test` by this lane**, and
  the reason will be a price, not an omission. Registered as a prediction about
  my own choice so that taking it is a MISS I have to explain, and declining it
  is not a silent default.

---

## 3. Decline floor

This lane reports **FAILED** — in that word — if any of:

* the instrument cannot be watched going RED on a planted false claim **and**
  GREEN on a planted true one, both directions, with `$?` read directly and
  never through a pipe; or
* C1–C6 collectively reach **zero** real claims on the tree (a checker with no
  subject is decoration); or
* the `MARK_RE` change moves any of the 777 (in which case the convention is
  reported and the code change is not shipped — that alone is not a FAILED, but
  shipping it anyway would be); or
* the gate's `GATE:` verdict line is not a PASS at tip.

A **priced decline** of `W-EXT-1`'s promotion is a legitimate outcome and is
not a FAILED. So is a zero on P8.

---

## 4. Fences, restated so a violation is visible

**Writes:** root `README.md`; **comment-only** in `crates/c2-reference/**`;
`docs/whitebox/DISCLOSURE.md`; `docs/whitebox/ref/README.md`;
`scripts/provenance_census.py`; new `scripts/prose_*` files;
`work/w-provaudit/**`; `docs/rungs/2026-08-26-w-provaudit.md`; board rows
**#3667**–**#3672** only.

**Explicitly NOT written, findings routed instead:**
`crates/c2-core/src/codegen/**` (`w-mopfold`); `docs/whitebox/ref/P_SECTION.md`,
`crates/c2-harness/src/subsys.rs`, `crates/c2-harness/src/cli/subsys.rs`,
`docs/SUBSYS_METRICS.md`, `scripts/subsys_metrics.sh` (`w-secported`).
