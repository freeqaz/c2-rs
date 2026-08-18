# `w-c2map2` PREREG — the address-indexed reference for `c2.dll`

    Lane:      w-c2map2   (branch wt-w-c2map2)
    Base:      master 071d2d47
    Kind:      characterization
    Fixtures:  none — characterization
    Census:    +0
    Board:     #3256–#3260 (coordinator-allocated; the next-free pointer was NOT read)
    Frozen:    2026-08-18, as this lane's FIRST commit, before any file under
               docs/whitebox/ref/ other than this one existed and before the
               generator was written.

> **Branch name.** `git branch -a | grep -i map` at base showed `wt-w-map`
> already taken (the original map lane). `w-c2map2` / `wt-w-c2map2` were free
> and are what this lane uses. Saying so because the brief predicted the
> obvious names would be gone, and one of them was.

---

## 1. Why this lane exists

`CLAUDE.md` § *"Whitebox analysis is AUTHORIZED — and it is not a legal risk"*
(landed `126dd853`, 2026-08-17) changed the policy: reading `c2.dll` is
encouraged, and **writing the reading down well is a deliverable rather than a
debt**. This is the first lane dispatched under that policy.

The engineering case is a measurable gap in the existing record, not a
sentiment. At base:

| | |
|---|---:|
| `WB_*_FINDINGS.md` documents | **19** |
| lines of prose across `docs/whitebox/*.md` | **~13 700** |
| hand-earned label rows in `docs/whitebox/labels/*.tsv` | **428** |
| non-`unknown` rows in `c2_functions.tsv` | **474** of 4 916 |
| **distinct `c2.dll` addresses cited anywhere under `docs/`** | **1 079** |
| documents that **join** those five sources | **0** |

The record is a *findings* archive: one file per investigation, chronological,
each excellent on its own question and mute on every other. There is no way to
start from an **address** and find out what is already known about it, and no
way to start from a **subsystem** and find the functions in it. That is why
lanes keep re-deriving: the alignment nibble cost a lane, `dag.c`'s lowering
order took two, and `#1823`'s "there is no instruction scheduler" stood for
months in a file that also contained the scheduler's band.

**This lane builds the missing index. It builds no new knowledge**, beyond what
the join itself exposes.

---

## 2. The layout, frozen here before any of it is generated

```
docs/whitebox/ref/
  PREREG.md          this file
  README.md          the front door: what the reference is, how to look something
                     up, the provenance legend, coverage with denominators, and
                     the three usability questions answered inline
  ADDR.tsv           GENERATED. One row per address that is either cited in docs/
                     or carries a hand label. Columns:
                       addr  kind  func  func_size  tu  tu_conf  subsys  page
                       conf  label  n_cites  cites
  SUBSYS.md          subsystem -> original TU -> page -> entry-point addresses.
                     The "which page do I want" table.
  P_COFF.md          coff.c + coffemit.c — the obj writer
  P_SECTION.md       p2symtab.c + emit.cpp — the section/symbol model
  P_REGALLOC.md      color.c (+ globregs.c, regasg.c) — priority colouring
  P_DAG.md           dag.c + the unnamed scheduler band — DAG build and schedule
  P_INLINE.md        inline.c + ptinl.c — the inliner
  P_EH.md            ehexcept.c + except.c + ssa_seh.c — EH state synthesis
  scripts/build_ref.py   the generator for ADDR.tsv (in docs/whitebox/scripts/)
```

### 2.1 Rules the pages are written under, frozen

1. **Every claim carries a provenance mark.** Three-way, and the distinction is
   the resource's main quality signal:
   * **`[R]` read** — taken from the disassembly and *not* checked against any
     obj or listing. A hypothesis, however clean the code looked
     (`C2_MAP_METHOD.md` §7 is this project's priced example).
   * **`[O] obj` / `[O] cod`** — confirmed against a real obj produced by real
     `c2.dll` under wibo, or against a `/FAsc` listing. Names the grid or cell.
   * **`[I] inferred`** — a step of interpretation on top of `[R]` or `[O]`.
   An unmarked claim is a defect in this reference.
2. **Never rewrite a dated finding in place.** Amend beside it, in a revision
   box, which is `WB_DAGORDER_FINDINGS.md`'s own rule. Corrections already in
   the record are carried forward, specifically:
   * `#1823`'s *"there is no instruction scheduler"* — **refuted**;
   * `WB_LIVE_FINDINGS.md`'s candidate field table — `+0x18` is the priority
     list's **prev** pointer and is phase-overloaded; `+0x44` is **not in the
     table at all** and is the field that decides every tie (`#3243`).
3. **This lane adopts nothing into `crates/`.** No `DISCLOSURE.md` row is added
   and none is needed. `crates/`, `fixtures/` and `scripts/` are byte-identical
   at both ends of this lane (`graded tree` identical).
4. **The Ghidra export path is machine state and is never hard-coded.** It is
   referenced as `$C2RS_GHIDRA_EXPORT`, defaulting to
   `~/ghidra-projects/export/c2`, in the generator and in prose.
5. **No page may be a dump.** A page that is more than half verbatim decompiler
   output has failed rule 5 — the deliverable is navigability, not volume.
   Full bodies stay in the (uncommitted, regenerable) export; the pages carry
   the decisive instructions and the exact retrieval recipe.

---

## 3. Coverage targets, with denominators — frozen

| # | denominator | what it is | target |
|---|---:|---|---|
| **C1** | **1 079** | distinct `c2.dll` addresses cited anywhere under `docs/` at base `071d2d47` | **≥ 95%** appear as a row in `ADDR.tsv`; **≥ 70%** resolve to a *containing function* with a size |
| **C2** | **4 916** | functions Ghidra found in the image | **no target** — this will be a small percentage and is reported honestly, not padded |
| **C3** | **6** | prioritized subsystems (COFF writer, section/symbol model, register allocator, `dag.c` + scheduler, inliner, EH) | **6 of 6** get a page with ≥ 8 addressed entries each |
| **C4** | **19** | `WB_*_FINDINGS.md` documents | **≥ 15** are back-linked from at least one `ADDR.tsv` row, so the reference points *at* the findings rather than restating them |

C2 has no target on purpose. A reference that claims broad coverage of a
1.35 MB binary in one lane would be lying, and the honest fraction is more
useful than a padded one.

**Explicitly not covered, declared in advance:** `globopt.c`, `globlopt.c`,
`lur.c`, `pogo*.c` (104 imports of a subsystem dead on this workload),
`dbg.cpp`/`.debug$S`, `ltcg.c`, `inlnasm.c`, the `.ex` opcode semantics beyond
what `WB_READER_FINDINGS.md` already has, and the instruction-selection tables
(three lanes have already read those and `WB_SELECT_RECONCILED.md` is their
join — this lane links to it and does not re-read).

---

## 4. The usability test — three questions past lanes answered the hard way

Volume is not the deliverable. The test is whether a future lane can **look
something up instead of re-deriving it**. Three questions, each of which cost
real lane time in this project's history, are frozen here. The finished
`README.md` must answer each **in one lookup** — meaning: one grep of
`ADDR.tsv`, or one named section of one page, with no reading of the 19
findings documents required to get the answer (they are cited so the answer can
be *checked*, which is different).

| # | question | why it is the test |
|---|---|---|
| **U1** | **The section-Characteristics alignment nibble.** Given an object of size *n*, what nibble does c2 put in the section's `Characteristics`, and where in the binary is that decided? The known table is `1 → ALIGN_1`, `2 → ALIGN_4`, `4 → ALIGN_4`, `8 → ALIGN_8`. | **This cost a lane.** A wrong nibble is a wrong `Characteristics` word, which is a byte mismatch. |
| **U2** | **The section emitter's `Selection` and `CheckSum` rules.** When does c2 write the aux record's `Number`/`Selection` at all, which `Selection` values ever occur, and what computes the aux `CheckSum`? | **Three lanes reached the COFF/section emitter by three independent routes** and none left a place to look this up. |
| **U3** | **The register allocator's tie-break field.** When two colouring candidates tie on priority, what decides? | `#3243`: the deciding field `+0x44` is **not in `WB_LIVE_FINDINGS.md`'s field table at all**, and at `/O1` most cells are ties. A reader of the field table would conclude the tie is unresolved. |

**Grading, frozen:** for each of U1/U2/U3 the report states **answered in one
lookup** or **not answered in one lookup**, naming the lookup. A question the
resource cannot answer is reported as a miss, not repaired by adding a section
after seeing the result — the section may be added, but the miss stands.

---

## 5. Predicted reach — **0**

Registered explicitly: this lane converts **zero** TUs and moves **zero**
fixtures. `match` stays **26**, `mismatch` **0**, `vocab-gap` **844**, anchored
keys **394**, `cargo test --workspace --release` **1 660 / 0 / 43**.

Per `#3249`, `fnbyte-*` is not a pure function of the commit and this lane
should not be moving it at all; it is not measured here and no `fnbyte` figure
is quoted as this lane's.

---

## 6. What makes this lane `FAILED` rather than `built`

Stated in advance so the outcome word is decided by the criteria and not by the
narrative:

* **F1** — fewer than **2 of 3** usability questions answered in one lookup.
* **F2** — `ADDR.tsv` covers **< 80%** of the 1 079 cited addresses (target is
  95%; below 80% the index is not an index).
* **F3** — any page is more than half verbatim decompiler output (rule 5): the
  lane produced volume instead of navigation.
* **F4** — the end state is not identical: `scripts/gate.sh --require-graded`
  not PASS, scan identity not **0 deltas over 394 keys**, `cargo test` not
  **1 660 / 0 / 43**, or `crates/`/`fixtures/`/`scripts/` not byte-identical.
* **F5** — a claim is published without a provenance mark, or a dated finding is
  rewritten in place rather than amended beside.
* **F6** — a machine path (`/home/...`) or the Ghidra export path is hard-coded
  into a tracked file.

Any one of F1–F6 makes the outcome word **FAILED**. There is no partial
outcome word; the rung header carries exactly one.

---

## 7. Registered risks

1. **The reference gets quoted as measurement.** A navigational index that
   blurs `[R]` and `[O]` will be cited as evidence within a week. Rule 2.1 is
   the mitigation and F5 is its enforcement.
2. **The join surfaces contradictions between findings documents.** Expected —
   there are already at least four filed corrections between lanes. Where two
   documents disagree the reference records **both, dated, with the later
   correction beside the earlier claim**, and does not arbitrate on its own
   authority.
3. **Stale addresses.** Every address is an absolute VA in exactly
   `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.
   The generator records the expected sha and the reference repeats it.
4. **The export may be missing something.** Per the brief, if the flat export
   at `$C2RS_GHIDRA_EXPORT` lacks something needed, this lane **names it and
   stops** rather than opening the Ghidra project or working around it.
