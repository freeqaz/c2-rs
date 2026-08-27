# PREREG — lane `w-secported`, wave 15

**Committed before the first measurement of the deliverable.** Charter:
`docs/DECISIONS_2026-08-22.md` § Decision 17, row `w-secported`. Base
`0dcfca959`. Branch `wt-w-secported`. Board rows reserved: **#3661–#3666**.

**The deliverable**: convert `ported` from a named residue to a number on the
**`section`** row of `docs/SUBSYS_METRICS.md`, or **decline it with a price**.

---

## 0. WHAT WAS ALREADY MEASURED BEFORE THIS FILE WAS WRITTEN

Stated first, because `w-disclose`'s standing rule is that **11 of 11 HIT is a
weak result if the measurements predate the prereg**. Orientation done before
this file existed, and every prediction below that it touches is marked
**`[prior]`** and graded as weak:

1. **A grep of `crates/` for all 25 addresses in `P_SECTION.md` §1's table**
   returned hits for exactly **two**: `0x10b99dfe` (a comment in
   `crates/c2-core/src/coff/label.rs:14`) and `0x10b9bdcf` (three sites in
   `crates/c2-il/src/func/glalias.rs`). Everything else: zero.
2. **`docs/whitebox/labels/W-SECT.tsv` (45 rows) and `W-GLREC.tsv` (27 rows)
   exist and were read.** They are prior art on both named sites. Neither
   enumerates the 27-entry byte-index table's *contents*.
3. **`crates/c2-il/src/func/glalias.rs` was read.** It carries `.gl` record
   tag values as named constants — `ALIAS_TAG = 0x10`, `KIND4_TAGS = [0x04,
   0x0E, 0x10]`, `KIND1_TAGS = [0x01, 0x02]`.
4. **`crates/c2-il/src/func/gl.rs` was skimmed** (227 KB). It appears to be a
   byte-pattern scanner over name runs and TYPE tags, not a record-stream
   dispatcher. Not confirmed.
5. **I eyeball-counted `P_SECTION.md` §1's table at 25 rows** while the page's
   own coverage line says `24 entries`. Not mechanically recounted.
6. `docs/SUBSYS_METRICS.md` carries the section row's mark census as
   **`[O] 17` of `53`** and `section-sites 137`, `section-read 24`,
   `section-sites-tu-level 327`. Carried, not re-measured.

The Ghidra export at `~/ghidra-projects/export/c2/` and the pinned
`compilers/X360/16.00.11886.00/c2.dll` (sha256 `c80981c0…a66258`, matching
`WB_ILARMS_MAP.md`'s pin) are both present on this box. **No byte of either
has been read for this lane yet.**

---

## 1. THE REGISTERED QUESTION: is this a JOIN or a READ?

`w-encmap` converted the `encode` row for the cost of a **join**, and named
`section` as *"the only other subsystem whose sites are RULES rather than
ADDRESSES"* — the property it claims made the encoder cheap. **This lane's
first duty is to test that claim, not to assume it.**

My reading of *why* the encoder was cheap differs from `w-encmap`'s, and the
difference is registered here as a falsifiable prediction:

> **J1 — the encoder was cheap because BOTH SIDES CARRIED THE SAME NUMERIC
> KEY, not because its sites were "rules".** The key is the **encode-form
> number**: c2 keeps it at `0x10c39b18`, and the port carries it in
> `mop::OPCODES` because `DISCLOSURE.md` row **W-MOP-2** adopted 85 whole rows
> of c2's own table into `crates/`. The join is `ENCODE_ARMS.txt.form ==
> OPCODES.form`. **Predicted: no comparable adopted key exists on the section
> side**, so the encoder-shaped join does **not** exist here.
> **Confidence 0.70.**

| id | prediction | conf | falsifier |
|---|---|---:|---|
| **J1** | No adopted shared numeric key exists between `P_SECTION`'s sites and any `crates/` table — i.e. the encoder-shaped join does NOT exist for `section` | 0.70 | a `DISCLOSURE.md` row adopting a section-side table into `crates/`, or any `crates/` table keyed by c2's section **kind** |
| **J2** `[prior]` | A **weaker** join does exist, on the **`.gl` record TAG BYTE**: both c2's byte-index table at `0x10b9c615` and the port's `.gl` reader carry tag values, so `tag ∈ port` is mechanically answerable | 0.75 | the port names no `.gl` tag as a value; or the tag constants cannot be recounted without a hand-authored list |
| **J3** | The **section-kind** unit (`0x10b982d6`'s arms) is NOT joinable: the port never sees a c2 section kind — it reads IL and emits **names** — so there is no kind-valued quantity anywhere in `crates/` | 0.80 | any `crates/` constant holding a c2 section-kind value (`0x1D`, `0x13`, `0x14`, `0x1B`, `0x20`, …) as a kind |

---

## 2. THE DENOMINATOR — chosen out loud, before it is counted

`w-encmap` found the encoder had **three** defensible denominators up to 5.6×
apart and published the one it used with the reason. `section` has at least
**four** candidates. Registered ranking of which one I expect to publish:

| # | candidate | what it is | predicted verdict |
|---|---|---|---|
| **A** | **27** | the `.gl` record dispatcher's byte-index table at `0x10b9c615` | **PREDICTED CHOICE**, conf 0.50 |
| **B** | **24 / 25** | `P_SECTION.md` §1's read entries — the row's *existing* `read` value | conf 0.30 |
| **C** | **137** | the two bands' Ghidra function entries — the row's `sites` | conf 0.05 |
| **D** | **327** | `FUNCS.tsv`'s TU-level attribution (2.4× the band) | conf 0.05 |
| **E** | — | none: `declined` | conf 0.10 |

**The registered argument for B over A, which I expect to lose:** the section
row is the one row in the table where `sites` (137 entries) and `read` (24
entries) are **already in the same unit**, so `sites ⊇ read ⊇ ported` is
well formed in *entries* without any of the encoder's unit gymnastics
(`encode` has `read 79 > sites 14`). If `ported` can be counted in entries,
that containment is strictly better formed than the one already shipped.
**The registered argument against B, which I expect to win:** an entry-level
verdict is a **judgment** ("does the port implement the alignment chooser"),
and a hand-authored per-entry verdict list is exactly the typed number
`verify()` exists to refuse.

> **D1 — I will publish the denominator and the reason, and I will publish
> every rival I measured beside it, as `w-encmap` did.** Registered failure
> mode: picking one silently, or reporting only the one that flatters.

---

## 3. THE NUMBER

| id | prediction | conf | bias |
|---|---|---:|---|
| **N1** `[prior]` | On denominator **A (27 tags)**: `ported = 5 ± 2`, the set `{0x01, 0x02, 0x04, 0x0E, 0x10}` | 0.45 | **UNDER** — I expect the port to name fewer tags than it effectively decodes, because `gl.rs` scans byte patterns rather than dispatching on a tag |
| **N2** | On denominator **B (entries)**: `ported ≤ 8` of 24/25 under any predicate I would be willing to defend | 0.60 | UNDER |
| **N3** | The strict address-citation reading — an entry counts iff its address appears in live `crates/` code — is **2 of 25** and I will publish it as a **rival reading that measures citation discipline, not implementation**, never as the number | 0.85 `[prior]` | — |
| **N4** | The byte-index table really holds **27** entries and the jump table really holds **16**, as `W-GLREC.tsv` and `P_SECTION.md` both say | 0.80 | — |
| **N5** | `P_SECTION.md` §1's table holds **25** rows against the page's own `24 entries` coverage line — i.e. the row's shipped `read = 24` does not reproduce by counting the table | 0.60 `[prior]` | — |

---

## 4. THE RESIDUE — the load-bearing half

The charter: *"a residue you can name is worth more than a ratio you cannot
defend."*

> **R1 — I will name, entry by entry, which of `P_SECTION.md`'s rules NOTHING
> in the port implements**, and predict that **≥ 15 of the 25 §1 entries** are
> unimplemented under every reading. **Confidence 0.65.**

Named in advance as expected members of the residue, so that finding them is
not scored as discovery: the `$zz`/`$zy` PGO-ordering creators
(`0x10be794d`, `0x10be79fa`, `0x10be7552`), the DEBTYP `$$TYPES` creator
(`0x10be7643`), the `.tls$` half of `0x10be7b9e`, the kind-remap
(`0x10be7727`), the base-section resolver (`0x10be76d4`), the alignment
chooser (`0x10be77a3`), and the bump allocator (`0x10c27b56`, whose rule
§5 **retracts**).

**R2** — I predict at least one entry where the port implements the rule's
**output** while citing nothing, so that an address-grep scores it 0 and a
behavioural reading scores it 1. Confidence 0.70. Named candidate: the `.bss`
reversal (`0x10b99093`).

---

## 5. THE DECLINE FLOOR — stated before the work, not after

`declined` is a legitimate outcome and the charter says so. I decline if
**either** holds:

* **F1** — no recount can be written that (a) recomputes `ported` from the
  tree on every `cargo test`, (b) contains **no hand-authored per-entry
  verdict list**, and (c) can be shown **red** by a planted fabrication; or
* **F2** — every candidate recount measures something other than *"the port
  implements this rule"* (e.g. citation discipline) by a margin I cannot
  state.

**If I decline, the deliverable is the priced decline plus §4's residue, and
the finding is `S1`:**

> **S1 — the encoder WAS a special case, and `w-encmap` named the wrong
> property.** Not *"rules rather than addresses"* but *"a numeric key the port
> ADOPTED from c2's own table"*. Predicted **TRUE, confidence 0.65.** If S1 is
> true, the scoreboard's shape follows: `ported` is convertible exactly on the
> rows where `DISCLOSURE.md` records an adoption, and on no others.

**What is NOT a decline:** shipping a small number. `ported = 3 of 27` is a
result. Only an undefendable or unrecountable number is.

---

## 6. HARD CONSTRAINTS I AM GRADED ON

| id | prediction |
|---|---|
| **H1** | `scripts/gate_identity_diff.sh` base→tip: **0 lines over 21 rows**, plus `--self-test`. I touch no emit path. Conf 0.95 |
| **H2** | `GATE:` verdict line at tip reads the same verdict as base. Conf 0.90 |
| **H3** | `cargo test --workspace` target count **unchanged or +0/-0 targets**; test count may rise by the controls I add. Conf 0.85 |
| **H4** | **#3641 will bite.** Editing `P_SECTION.md` will move the `section` mark census (now `[O] 17` of `53`) unless every mark letter I write is respelled. I will report before/after **for every doc edit**, and predict at least one draft moves it. Conf 0.80 |
| **H5** | I add **no** second provenance or census reader, mint **no** `DISCLOSURE.md` row, and write **no** file outside my fence. Conf 0.95 |
| **H6** | A new control is planted, **watched red**, and reverted before its green is quoted (#3336). Conf 0.95 |

---

## 7. SELF-GRADE FAILURE CONDITIONS

Registered as failures of *this lane*, not of the tree:

1. Publishing a `ported` number whose denominator choice is not stated with
   its rivals and the reason.
2. Publishing a number that `verify()` cannot recount, or that a fabrication
   control cannot redden.
3. Manufacturing a number to match `w-encmap`'s shape — the charter names
   this explicitly.
4. Reporting the mark census only once, or only after the last edit.
5. Grading J1/J2/N1/N3/N5 as HITs without saying `[prior]` beside them.

*(No line in this file is edited after its commit. Grades land in
`docs/rungs/2026-08-26-w-secported.md`.)*
