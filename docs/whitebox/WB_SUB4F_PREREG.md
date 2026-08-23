# WB_SUB4F — PREREG for read R9 (the `0x4F` sub-record switch)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Lane:** `w-read-r9` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered.

**Subject.** Read **R9** of the funded read-plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3 row R9, funded by the owner
2026-08-23 — `docs/DECISIONS_2026-08-22.md` decision 7, board **#3423**;
this lane's rows are **#3442**–**#3444**): the `.ex` `0x4F` sub-record
reader `FUN_10b9761e`, the 8-byte-stride descriptor table at `0x10b26268`
it indexes, and the switch at `0x10b9766c` onward. Priced **1–2 days** —
the cheapest row in the plan, which is why **completeness is the registered
target and a sample is not acceptable**: every arm, not a stratum.

**Deliverable, in the plan's words:** *"the `0x4F` width/semantic table."*
Page: `ref/P_SUB4F.md`. Grade: `WB_SUB4F_FINDINGS.md`. Instrument:
`scripts/dump_sub4f.py`.

**Why this row is unusually close to shipping consequence.** `0x4F` carries
**the one width the port transcribes rather than derives**
(`DISCLOSURE.md:89`, row `W-MID-4`, verdict `route`): *"The one width this
lane uses — `4F 01 <byte>` is three bytes — is TRANSCRIBED from the corpus
and labelled as such in the code, and every other `0x4F` sub-opcode
refuses."* So this read either **confirms** that width and supplies the
reason the port cannot currently state, or it **contradicts** it — which
would be a defect finding on shipped code. `w-read-r3` found exactly that
shape (a fitted constant, latent-wrong, green only because its control was
structurally incapable of exercising it). **If this lane finds one it
REPORTS it and does not fix it** — see § "Fence" below.

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
size 1 347 072 — **verified by this lane before any address was read**,
matching the pin at `ref/README.md:21`. The Ghidra flat export at
`~/ghidra-projects/export/c2/` is dated 2026-08-04, nineteen days before
this tip; per `READ_PLAN` §5.4 this lane **parses bytes out of the pinned
image directly** and treats the export as a cross-check only, never as the
source of a function boundary (§5.4's `0x10b7f022` trap: *"Ghidra found
4,916 functions" is a statement about Ghidra*).

---

## 0. Three corrections to the dispatch brief, made BEFORE the target was read

All three are arithmetic or quotation over **committed text**, not over the
target's bytes. They are registered here so that the findings document
cannot be accused of discovering them after the fact.

**0.1 — The brief's entry point is wrong. R5 handed R9 *arm 32*, not
"arms 48/49".** The dispatch brief and `docs/DECISIONS_2026-08-22.md:275`
both say *"**R5 supplied its entry points** (arms 48/49)"*. R5's rung says
something else. `docs/rungs/2026-08-23-w-read-r5.md:195-198`, verbatim:

> 5. **Read R9's `0x4F` sub-record** enters at arm 32 → `0x10bbe561`; **read
>    R6's `0x2f4`/`0x2f0` prologue arms** are minted by arm 48 and arm 49.
>    Both now have an entry point they did not have.

It is a compound sentence naming **two different reads**. R9's entry is
**arm 32**; arms 48 and 49 are **R6's**, and `ref/P_ILRECORD.md:245-246`
confirms they are opcodes `0x8d`/`0x8e` minting `0x2f4`/`0x2f5`/`0x2f0` —
nothing to do with `0x4F`. The mis-transcription is in the decision record,
so it would have propagated. **Corrected here; the findings document will
carry the correction to `DECISIONS_2026-08-22.md` as a row.**

R9's real handoff is `ref/P_ILRECORD.md:238`:

> | 32 | `10bc38bb` | `4f` | 0c | 23 | 1 | — | ROUTE | DEFER | → `0x10bbe561`. **The `0x4F` sub-record — read R9's target — enters here** |

**0.2 — "the one transcribed width in the port" is at least FIVE readers
carrying TWO mutually inconsistent widths.** The brief, `READ_PLAN:178`
and `WHITEBOX_LEVERAGE:90` all say *"the one transcribed width"*, singular.
Committed source says otherwise. Two sites read a **fixed one-byte**
payload:

- `crates/c2-reference/tests/middle_interfaces.rs:716-724` — `p += 3`
- `crates/c2-il/src/codec.rs:1128-1145` — `let nn = *body.get(p + 2)?; … 3`

and three read a **varint**:

- `crates/c2-il/src/func/readers.rs:409-416` — `eat_opt_stmt_marker`
- `crates/c2-il/src/func/body/shapes/control_flow.rs:534-542` — `Scan::line_markers`
- `crates/c2-il/src/func/body/expr.rs:1300-1306` — `SkipForm::Line4F`
  (and its `BranchSink::Stmt` twin at `:2203-2210`)

with a sixth candidate at `crates/c2-il/src/func/ehscope.rs:126`.
`docs/IL_STMT_GRAMMAR.md:236-240` already calls the fixed-byte read *"a
live bug for any TU whose functions live past line 127, i.e. essentially
every real one"*, and `readers.rs:391-408` records the same: *"A function
at source line 200 emits `4f 01 80 c8 00 00 00` — the escaped four-byte
form. Reading one byte therefore desynchronizes the whole token stream."*

**So the port does not have one transcribed width; it has an unreconciled
pair, and `DISCLOSURE.md:89`'s literal text matches the narrower and
already-suspected-wrong one.** Registered as the state of the world before
this read. Whether the descriptor table settles it is prediction **P2**.

**0.3 — `FUN_10b9761e` is not virgin ground, and the overlap is a
cross-check this lane owes.** `ref/P_LABEL.md:51` (read R3) marks
`0x10b97807` — *inside* `FUN_10b9761e` — as the **label seed install for IL
directive `0x16`**, `[R]`. Two independent reads of one function must
agree. Registered as prediction **P7**, and a MISS there voids nothing of
R3 automatically but is reported as a live contradiction between two pages.

**Committed inventory rows this lane starts from** (`ref/ADDR.tsv:586-590`,
`ref/FUNCS.tsv:2084`, `labels/W-IL.tsv:35`, `c2_functions.tsv:2070`,
`c2_tus.tsv:29`): `FUN_10b9761e` is **606 bytes**, TU
`e:\bt\278379\vctools\compiler\be\p2\p2pragma.c` (`in-anchor` — a fact, not
a hypothesis), **1 caller / 13 callees**, coverage `labelled`, **no page
owns it**. `0x10b26268` is `ref/ADDR.tsv:93`: `data`, **size 4**,
confidence **`unknown`** — *nobody has parsed it*, which is precisely the
state `0x10bc4152` was in before R5 corrected 189 arms to 62.

---

## 1. The grading rule

Registered **before** any byte of `FUN_10b9761e`, of `0x10b26268`, or of
any arm body was read. Tier **PREREG** by `PREREG.md`'s ladder: committed
to git as **the first commit on `wt-w-read-r9`**, before the answer existed
anywhere in this lane. **The DAG ordering is the evidence** — this commit
provably precedes the commit that teaches any tooling to parse the table at
all.

Each prediction is scored HIT / MISS / UNGRADED in `WB_SUB4F_FINDINGS.md`.
**Misses are reported as misses and are not smoothed.** A prediction too
vague to be falsifiable earns nothing and is marked UNGRADED rather than
counted. Numerators are reported with denominators and **the denominator is
named before the numerator is known**.

Confidences below are this lane's honest priors, written down so that a
lucky guess at 0.5 cannot later be presented as insight.

---

## 2. Predictions

### P1 — The "~14 arms" figure is wrong, and it is wrong in R5's direction

**Registered at 0.60 that the true distinct-arm count is not 14**, and at
**0.65 that the number of *sub-opcodes* the table admits (call it `M`)
strictly exceeds the number of *distinct arm bodies* (call it `N`)** — i.e.
this is a many-to-one structure and "~14" is a count of one level being
quoted for the other.

*Rationale, from committed text only:* `DISCLOSURE.md:89` describes exactly
two levels — *"read off an 8-byte-stride descriptor table at `0x10b26268`
**and then** a ~14-arm switch"*. A per-sub-opcode descriptor followed by a
switch is the shape in which a count of table rows and a count of switch
arms are different numbers. R5's headline error was structurally identical
(189 = the opcode domain; 62 = the arms), and `WB_MIDDLE_INTERFACES.md`
§2.2's stride-12/stride-16 trap is the same family.

**HIT/MISS.** Report `M` and `N` as measured integers with the bound
instruction that fixes `M` quoted by address. P1a HITs if `N ≠ 14`. P1b
HITs if `M > N`. If `M == N == 14` both MISS and the brief was right.

### P2 — The `4F 01` payload is a VARINT, not a fixed byte

**Registered at 0.75.** The descriptor for sub-opcode `0x01` will select a
variable-width read (the `varU`/`i16c`/`i32c` family already catalogued at
`WB_READER_FINDINGS.md:135-155`), not a fixed 1-byte field.

*Rationale:* the escaped form `4f 01 80 c8 00 00 00` is **already witnessed
in the corpus** (`readers.rs:391-408`), and three of the port's readers
already read a varint against two that do not. A read that says "fixed
byte" would have to explain the witnessed 7-byte record.

**HIT/MISS.** HIT if the arm reached for sub-opcode `0x01` performs a
variable-width read, named by the reader address and cross-referenced to
`WB_READER_FINDINGS.md`'s scalar-reader inventory. MISS if it reads exactly
one byte unconditionally.

**Consequence registered in advance, so it cannot be softened after the
fact:** if P2 HITs, then `middle_interfaces.rs:719` and `codec.rs:1131` are
**latent-wrong** and `DISCLOSURE.md:89`'s claim text is **under-general**.
This lane will file that as a defect finding with a board row and **will
not touch `crates/`.**

### P3 — The switch selector is a descriptor FIELD, not the sub-opcode

**Registered at 0.60.** The `switch` at `0x10b9766c` dispatches on a value
loaded *out of* the 8-byte descriptor entry, not on the raw sub-opcode
byte. Equivalently: the descriptor's 8 bytes contain at least one small
integer that is a **kind/width class**, and several sub-opcodes share a
class.

*Rationale:* it is the only structure under which "8-byte-stride descriptor
table **and then** a switch" is not redundant. If the switch were on the
sub-opcode, the table would be doing nothing the switch does not.

**HIT/MISS.** HIT if the value reaching the switch is provably a load from
`[…*8+0x10b26268]` or from a register loaded from it, with the byte offset
within the entry named. MISS if the switch reads `[esi+0x24]` (the raw
sub-opcode) directly. UNGRADED is not available — this is decidable from
the bytes.

### P4 — The descriptor entry's 8 bytes are two DWORDs, one of them a pointer

**Registered at 0.55** that the entry is `{u32, u32}` with **at least one
field being a code/VA pointer or a string pointer**, and at 0.70 that **at
least one field is a small integer < 0x40** usable as a width or class
code.

**HIT/MISS.** HIT on a field-by-field layout with each field's reader
instruction cited by address. Fields never read anywhere in the image are
reported as **unread, not as absent** — that distinction is the deliverable's
honesty margin.

### P5 — `M ≥ 7`, and the corpus-witnessed sub-opcodes are all admitted

**Registered at 0.85.** `docs/IL_STMT_GRAMMAR.md` §12.6 names seven `0x4F`
sub-opcodes observed in real IL: `01`, `02`, `11`, `12`, `1F`, `20`, `33`.
Every one of them must be inside the table's bound and reach a real arm,
not the default.

**HIT/MISS.** 7/7 admitted = HIT. Any witnessed sub-opcode falling to the
default/refusal arm = MISS **and is the more interesting outcome**, because
it would mean the record is consumed somewhere other than here — the `.bss`
failure mode (`ref/README.md:54-60`) applied to this read.

### P6 — The `47` in `4F 12 47 54 01 54 00` gets an explanation

**Registered at 0.45**, deliberately below even odds. `IL_STMT_GRAMMAR.md`
§12.6: *"The single byte `47` between `4F 12` and the outer scope closes is
**unexplained**; it is a fixed byte in every one of the ~5300 bodies
examined."* Prediction: sub-opcode `0x12`'s width, read out of this table,
accounts for it — i.e. `4F 12` is a **2-byte** record and `47 …` is the
next token, or `4F 12` is a **7-byte** record and `47` is inside its
payload.

**HIT/MISS.** HIT if the table + arm decide **which** of those two it is,
by width. MISS if the table is silent. This is the one prediction whose
MISS costs the lane nothing beyond the row.

### P7 — `0x10b97807` is the `0x16` arm, and R3 and R9 agree

**Registered at 0.80.** `ref/P_LABEL.md:51` places the label-seed install
for IL directive `0x16` at `0x10b97807`, inside this function, `[R]`.
Prediction: that address lies in the arm selected for **sub-opcode `0x16`**.

**HIT/MISS.** HIT if `0x10b97807` is inside the `0x16` arm's extent.
MISS if it is inside a different sub-opcode's arm — in which case
**`P_LABEL.md:51` is wrong about which directive installs the seed**, and
this lane reports the contradiction and amends beside without rewriting
R3's page (`ref/README.md:72+`: *"Corrections are amended beside, never
rewritten in place."*). **This is a registered self-test: it is a claim
made by a different lane that this read can independently falsify, and it
is the reason this read is not self-confirming.**

### P8 — `FUN_10b9761e` and `0x10bbe561` are different phases, not duplicates

**Registered at 0.70.** `FUN_10b9761e` (`p2pragma.c`, 606 B) is reached
from the **operand-format** dispatch's class-`0C` arm at `0x10b3d7d7`
(`WB_READER_FINDINGS.md:177`). `0x10bbe561` (`reader.c`, 668 B, coverage
`none`) is reached from the **record→codegen** dispatch's arm 32
(`ref/P_ILRECORD.md:238`). Prediction: the first is the **parse/width**
side and the second is the **semantic/effect** side, and neither calls the
other.

**HIT/MISS.** HIT if the call graph shows no edge between them and their
reads are disjoint in kind. MISS if one calls the other or if they are the
same code reached two ways. **Nothing in `docs/` connects them today, and
connecting them is itself part of the deliverable.**

### P9 — Table-derived widths parse whole `.ex` segments exactly

**Registered at 0.70.** This is the confirmation probe's central claim and
it is stated as a prediction so it can MISS. Using **only** widths derived
from `0x10b26268` and its arms, a walk over real captured `.ex` function
segments will consume every `0x4F` record and land exactly on the segment
end — never mid-record, never past it.

**HIT/MISS.** Denominator = the number of segments walked, named before the
numerator. HIT at 100 %. Anything below 100 % is a MISS and the failing
segments are published.

---

## 3. What would make this lane DECLINE

Registered so that a decline is a priced outcome and not a retreat.

1. **The route claim is false.** If `FUN_10b9761e` is not on the `0x4F`
   path — if `[esi+0x24]` is not the sub-opcode, or the function is
   unreachable from the class-`0C` arm — then `DISCLOSURE.md:89`'s `route`
   row is wrong. That is a **finding, not a decline**; the lane reports it
   and still owes the corrected route.
2. **The table has no static bound.** If the index into `0x10b26268` is not
   bounded by a comparison this lane can locate, `M` is unknowable
   statically and the width table cannot be published as total. Outcome:
   publish the arms that are reachable, mark the domain **open**, and say
   so in the headline. Not a full decline.
3. **Widths are data-dependent through a callee this lane cannot bound.**
   If the descriptor's field is a function pointer into a subtree deeper
   than depth 2, the lane reads to depth 2, names the boundary, and reports
   the remainder DEFER — R5's convention.
4. **Full decline** is reserved for: the switch cannot be statically
   resolved at all (computed target with no table), in which case the lane
   reports `FAILED` in that word per `docs/rungs/README.md`, not a
   compound headline.

---

## 4. The fence — what this lane will NOT do

- **Zero `crates/` bytes.** Characterization lane. `Fixtures: none`,
  `Census: +0`, predicted reach 0. The branch's whole diff against master
  must be `docs/`. This is checkable and the findings document will publish
  the check.
- **A defect in the transcribed width is REPORTED, never fixed here.**
  `w-s1bc` is the only lane in `crates/` this wave. Editing `crates/` under
  a docs-only fence is exactly what `w-read-r3` declined to do with a
  shipped defect, and it was right.
- **No `DISCLOSURE.md` row is owed** unless a disassembly-derived constant
  is adopted into `crates/` — which the fence above forbids. `DISCLOSURE.md`
  is the ledger of **adoptions**, not of readings (R1, R3 and R5 each owed
  none, and `READ_PLAN` §3's banner records that the predicted three rows
  correctly did not materialise).
- **No edit to a peer lane's page.** `w-read-r6` (final-expansion switches),
  `w-read-r7` (scheduler), `w-read-r8` (block emission order) are live in
  `docs/whitebox/`. This lane writes `WB_SUB4F_PREREG.md`,
  `WB_SUB4F_FINDINGS.md`, `ref/P_SUB4F.md`, `scripts/dump_sub4f.py`, its
  own rung, and **appends** rows `#3442`–`#3444` to `docs/BOARD.md`.
  Corrections to `DISCLOSURE.md`, `READ_PLAN`, `DECISIONS`, `P_LABEL.md`
  and `P_ILRECORD.md` are **amended beside, never rewritten in place**.
  `docs/rungs/INDEX.md` is regenerated, never hand-resolved.

---

## 5. The confirmation probe — and the failure modes it is built against

`READ_PLAN` §5.3: **`[R]` is a hypothesis.** *"`[R]` says 'the instructions
were read correctly', not 'this is what c2 does'"* — the `.bss` bump rule
was read correctly out of a clean function and was wrong about c2. So this
read ends in a probe that **can go red**.

**Two probes, because grids and corpora fail in opposite directions** and
the brief requires both. R2's probe was specified against four named
failure modes and still could not see a fifth; only a large corpus made its
control capable of failing.

### 5.1 Named failure modes the probes are built against

| # | failure mode | which probe can see it |
|---|---|---|
| **FM1** | **Origin off-by-one** — the table is indexed by `sub−1` or `sub+1`, so every published width is shifted by one sub-opcode | corpus (a shift desynchronises the walk immediately) |
| **FM2** | **Stride wrong** — 8 assumed, actually 4 or 16. `WB_MIDDLE_INTERFACES.md` §2.2 documents this trap **by name** (stride-12 vs stride-16), and R5's first probe appeared to refute its own central claim by reproducing it | grid (the `01` and `12` entries must independently predict two *different* known behaviours) |
| **FM3** | **Wrong field of the entry** — the width is read out of dword 0 when it lives in dword 1 | corpus + the arm reads themselves (the reader instruction is cited, so the field is not guessed) |
| **FM4** | **The vacuous-green trap.** Every fixture's functions live at source line < 128, so a fixed-byte read and a varint read agree on all of them and the control cannot distinguish them. **This is the mode that would let P2 pass while the port stays broken**, and board #2668 (`w-xtea2`) records a lane already paying for it: *"A body written over several source lines carries `4F 01 <line>` markers a one-line body does not — and every probe in this lane was one-line."* | grid, **and only if the grid is authored to cross the escape boundary** — which is therefore a hard requirement below |
| **FM5** | **Read-correct but off-path** — the arm exists and is read right, but a given record never reaches it because something upstream diverts it. The `.bss` bump mode | corpus (a record that never appears is reported as unwitnessed, not as confirmed) |
| **FM6** | **The default arm is invisible.** Sub-opcodes outside `M` fall to a default whose behaviour (skip? error? ICE?) decides whether an unknown record is fatal or ignorable — and no fixture in the workload produces one | **neither.** Named here as uncatchable; see §6 |

### 5.2 Probe A — the GRID (authored, adversarial, small)

Hand-authored fixtures compiled with the **real** toolchain under wibo,
capturing `.ex`, specifically constructed to **cross the varint escape
boundary** so FM4 cannot hide:

- functions and statements at source lines spanning `< 128`, `== 127`,
  `== 128`, and well past (`> 16 383` if a generated fixture makes it
  cheap), so the `4F 01` payload takes 1-byte, 2-byte and escaped forms in
  the same experiment;
- multi-statement bodies (board #2668's hazard: one-line bodies carry no
  interior line marker at all);
- at least one fixture per corpus-witnessed sub-opcode this lane can
  provoke.

**Red condition:** the byte length of an observed `4F 01 …` record differs
from the length the table-derived width predicts, on any cell.

### 5.3 Probe B — the CORPUS (broad, unauthored, capable of surprising)

An exact-consumption walk over **every** captured `.ex` function segment
available to this lane, using **only** table-derived widths for the `0x4F`
family:

- every `0x4F` record consumed by its table width;
- the walk must land **exactly** on the segment end;
- every distinct sub-opcode encountered is tallied, with its observed
  width, against the table's prediction.

**Red condition:** any segment where the walk lands off the end or
mid-record; any sub-opcode whose observed width contradicts the table.

**Would this go red if the claim were false in the most likely way?** The
most likely way for the width table to be wrong is a shifted origin (FM1)
or a wrong field (FM3). Both make the walk desynchronise, and a
desynchronised walk almost never lands exactly on a segment boundary — so
yes. What it would **not** catch is a width that is wrong only for a
sub-opcode the corpus never contains, which is FM6 and §6.

### 5.4 The registered instrument self-test

Before either probe is graded, `dump_sub4f.py` must reproduce **two facts
this lane did not derive**: the function's size (**606 bytes**, from
`ref/FUNCS.tsv:2084`) and the presence of `mov eax,[eax*8+0x10b26268]` at
**`0x10b97641`** (from `DISCLOSURE.md:89`). If either fails, the instrument
is wrong and **the lane is void** — the same gating R3 used. Registered
here, before the tool exists.

---

## 6. What these controls are STRUCTURALLY INCAPABLE of catching

Written before any result, and it is the part of this file that matters
most.

1. **Every arm the corpus never exercises stays `[R]` forever, and the
   corpus is narrow.** Board **#3096** (`w-readphase`, measured over 870
   TUs) records that **no function body in the whole workload reaches a
   `0x4F` sub-opcode other than `01` and `12`**. Even counting header and
   module positions, `IL_STMT_GRAMMAR` §12.6 names only seven. **If the
   table admits `M` sub-opcodes, then `M − 7` of them are gradeable by no
   probe this lane can build**, and their widths will be published as `[R]`
   with that denominator stated in the headline. A width wrong on an
   unwitnessed arm is exactly `w-read-r3`'s finding — a constant green only
   because its control could not exercise it — and this lane can *name* the
   exposure but not *close* it.
2. **The IL this project can generate comes from one front end at one
   version.** `c1xx.dll` 16.00.11886.00 emits the sub-opcodes it emits. A
   record class produced only by another front end, another `/O` mode this
   lane does not vary, a `#pragma` this lane does not write, or LTCG —
   which `p2pragma.c` being the **containing TU** makes a live possibility
   — is invisible to both probes. The probes measure c2's reader against
   c1xx's writer, and they agree by construction wherever c1xx never goes.
3. **Exact consumption is a necessary, not a sufficient, condition.** Two
   different width tables can both consume a segment exactly if the
   disagreement is compensated downstream, or if the disagreeing sub-opcode
   is absent. Probe B can confirm a table and still be blind to a wrong
   cell. It is a strong *falsifier* and a weak *verifier*, and the findings
   will not present a green as proof.
4. **The default arm's behaviour is unobservable from valid IL.** FM6.
   Deciding whether an unknown `0x4F` sub-opcode is an ICE, a skip, or
   silent corruption requires feeding c2 **invalid** IL, which is a
   different experiment with a different risk profile and is **out of scope
   for this lane**. It is registered as an open follow-up, not as covered.
5. **Reading a width table cannot prove the reader is on the path.** The
   `.bss` bump mode (`ref/README.md:54-60`) in full: a function can be
   present, correct-looking, and simply not on the path a given input
   takes. Probe B mitigates this for witnessed records only.
6. **This lane grades no obj.** The deliverable is a decode/width spec, not
   an emit rule. Nothing here is `[O] port`, and no claim on
   `ref/P_SUB4F.md` may be marked so.

---

## 7. Bookkeeping registered in advance

- **Board rows: `#3442`–`#3444` only.** Appended, no existing row edited.
  Next free number afterwards: `#3445`.
- **Rung:** `docs/rungs/2026-08-23-w-read-r9.md`, `Outcome:` exactly one of
  `converted | declined | instrument | built | FAILED`. **Expected
  `built`.** A lane that produces none of its deliverable says `FAILED` in
  that word.
- **Gates:** the armed suite
  (`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast`,
  liveness asserted from the positive fact that
  `crates/c2-harness/tests/require_toolchain.rs` makes a toolchain-less run
  FAIL, so exit 0 under the flag **is** the assertion — never a duration
  cut, never a SKIP count), `scripts/gate.sh --jobs 4` with counts quoted,
  and `scripts/board_audit.sh` graded by its before/after row-count diff,
  its exit code being advisory by its own header.
- **`coff::Function` field this lane's work would eventually write: NONE.**
  Arch review finding 3's prophylactic. This lane reads a binary and writes
  documentation.
