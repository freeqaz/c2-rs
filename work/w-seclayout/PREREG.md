# w-seclayout — PREREG

**Frozen before the first `crates/` change.** §1 is `CEILING.md` §11.4 worked
off this lane's own capture; §2 is what the lane already *knows* from reading
code and the board; §3 is prediction, each row with the antecedent the claim
actually needs. Scored in the rung's §9.

    Lane:      w-seclayout, worktree branch `worktree-agent-a611741296081ab81`
    Base:      master `5127a20e` ("docs: ROADMAP §10.35 — session close").
               `git merge-base HEAD master` = `5127a20ee2c4c93dfdc01166768a52aca1a3d0a7`.
               Tree CLEAN at freeze except this lane's own `work/w-seclayout/`.
    Workload:  dc3 `104e7df9c10acfe56ee3a87d75f0a9c85740df11`, tracked tree
               CLEAN (one untracked dir, `work/`). **Unchanged from
               `w-selbind` and `w-frame783`** — third lane running at the same
               stamp (#2392).
               `work/dc3-workload/files.txt` sha256
               `4996839bf89780a2dea9ed005450d8953961355a9eb2292cc1bc22572a6853b6`
               (878 lines), `flags.txt` sha256
               `fa8ba48aa21229773116bf0decff3b7e9e5e7f7ee356c3e347c506038ffbcb48`.
               **Used as they stand and never regenerated** (#2700).
    Toolchain: `compilers/X360/16.00.11886.00`, wibo `1.2.0-c2rs.1`.
    Base bin:  sha256 `ac193eab2651b63b0c28f6b59e6dda49abd0b15cea603fbd94e23c9c10c902e3`
               (md5 `5d459cd879ce7c540242affdbd86ecd6`), copied to
               `work/w-seclayout/c2rs-base` before the first edit (#2409) and
               **KEPT**. Every base figure below is that binary's own scan
               (`work/w-seclayout/base.log`, `base.jsonl`).
    Base test count, re-derived at THIS merge-base from `STATUS.md`'s generated
               block (never from a rung): **1497 passed, 0 failed, 41 targets**;
               selftest **369 PASS / 0 FAIL**; fixture gate **150 Match /
               0 mismatch / 219 not-implemented of 369**;
               `fnbyte-exact` **35810**.

---

## 1. `CEILING.md` §11.4, WORKED FIRST, OFF THIS LANE'S OWN CAPTURE

Targets of record: the ≥ 5 TUs read in §5 of the rung, drawn from this lane's
own `target380.txt`. The pass below is stated for the **class**, with the
per-TU cells in the rung.

| # | item | answer at this tree, this lane's scan |
|---:|---|---|
| **1** | ask the BYTE judge, not the census | per-TU `fnbyte` columns read from the base scan for each read TU; the class figure is that **all 380 are `vocab-gap`**, so no obj exists to grade |
| **2** | if every body is exact the blocker is not codegen | **T1 cannot fire on any of the 380** — `body-out-of-class` is in the `gate_causes` SET of **380 of 380**, so codegen is owed on every one of them regardless of what the walk does |
| **3** | read the reference obj's SYMBOL TABLE, not just `.text` | this is the lane's *subject*: §5 reads the section table, the characteristics word, the aux `Selection` byte and the symbol table of each read TU's real obj with `scripts/gt_dump.py` |
| **4** | is the refusal LIST MEMBERSHIP? | **no** — `gl-stop-26-introduced` keys on a separator BYTE (`NAME_SEPARATORS[1]`) preceding a defined record's name, not on a positive list |
| **5** | do not trust the reported key's LAYER; grep the WORKLOAD before pricing the class | done at the class level: `gl-stop-26-introduced` is the FIRST cause on **819** of the 848 vocab-gap TUs and in the SET of **831**. The 380 is the sub-population whose *emit set* is entirely named |
| **6** | check factor A before pricing reader or emitter work | `factor-a 28`, `a-and-b-and-c 27`, `frontier 4` at the base scan. **Factor A is exactly what `Bindings::selective` clause 4 needs and does not have** |
| **7** | check the board | **#232** (re-derived in §2), #259, #2783, #2820–#2827, #2860–#2867, #2530–#2545 (`w-biquad`), #2590–#2599 (`w-pool2`), #2470–#2482 (`w-fence2`), #1721. Nothing here re-enters a ranking that already measured zero (#2243/#2246 tally) |
| **8** | quote the GATE's number — **`gate_cause`**, and nothing else | over `target380.txt`: `gate_cause` is `gl-stop-26-introduced` on **379** and `drectve-not-boilerplate` on **1**; `gate_causes` carries `gl-stop-26-introduced` on **380** and `body-out-of-class` on **380**. Reproduces #2864 digit for digit off this lane's own scan |
| **8b** | an instrument's population is bounded by the reader | **THE TRAP OF THIS LANE, FOUND BEFORE IT WAS QUOTED — see §1.1** |
| **9** | if T1 fires, read the FENCES before the obligations | T1 does not fire on any of the 380 (item 2), so the three fences — `comdat::fenced_inlined_callee`, `elide`'s E, `splice`'s S7 — are *behind* an unwritten body on all 380, and are part of the price in the sense of §11.4 item 9's second instance |

### 1.1 Item 8b, and the field this lane refuses to quote

The obvious way to price *"would the 380 bind if the walk were repaired"* is
the `--jsonl` field `selective_bind` — `(records, segments, unclaimed_mangled,
unclaimed_inline_fit)` from `IlBundle::selective_bind_coverage`. Read naively
it says **records < segments on 380 of 380**, which reads as *"clause 3/4
refuses them all"*.

**It is an artifact and it is this lane's own selection that makes it one.**
`records` comes from `gl::gl_bound_names`, which is

```rust
gl_defined_names_framed(gl, true, GATE_BIND_FRAME, NameFit::InlineOrStringTable)
    .unwrap_or_default()
```

— so a TU whose walk **stops** reads `records = 0`. The 380 were *selected for
stopping*. Measured: `records == 0` on **380 of 380**
(`work/w-seclayout/records0.py`). The field says nothing whatever about a
repaired walk, and quoting it would have been the tenth ranking artifact of the
session.

The honest instrument is a **counterfactual walk** — the six stop clauses with
`Name26Introduced` removed, run over each TU's own `.gl`, off the accept path
(`work/w-seclayout/glwalk26.py`, a transcription in the manner of
`work/w-front5/glwalk.py`). §3's rows are scored against that, not against
`selective_bind`.

---

## 2. #232 RE-DERIVED, and what this lane will and will not build

Re-derived from the board row, `w-cross`'s and `w-order`'s rungs and the clause
in `crates/c2-il/src/func/gl.rs` — not paraphrased from a forward doc.

**The defect was NOT the relaxation and NOT the absence of a fence.** `d0d8a98`
(W-ADOPT, #151) taught the `.gl` scanner the `26` name separator. Its own commit
message named this exact risk — *"the one place the widening could have produced
wrong bytes instead of a refusal"* — and it shipped a fixture,
`fixtures/cpp/il_gl_sep26.cpp`, asserting `NotImplemented` at base and tip. **The
fence was written, was real, and passed.** What it guarded was a run that *ends*
at `26`; the shape that broke is a run that *begins* at one, which is a different
shape, NUL-terminated like any other, whose every downstream field arithmetic is
correct. **Nothing was wrong with the name.** What was wrong is the **obj shape
the name implies**: a `26`-introduced defined name is COMDAT-style linkage, and
the port's packed writer has one `.text` for the whole TU. So the wrong emit is
**a writer that cannot express the layout the input asks for, reached through a
gate that was widened without asking the writer**.

The three candidate mechanisms, decided:

| candidate | verdict |
|---|---|
| the relaxation itself | **no** — the widened *scanner* is still shipped today; `w-cross`'s fix widened nothing back |
| the absence of a fence | **no** — a fence existed, was reasoned about in its own commit message, and covered a neighbouring shape |
| a writer that emits the wrong layout | **YES** — 6 sections against 7, both symbols packed into one `.text`, in the opposite order; `Port=Mismatch @ offset 2`, `NumberOfSections` |

And the reason it lived 255 commits (`d0d8a98..be86f9d`) is **process, and it is
separable from the mechanism**: `scripts/gate.sh` did not run `expr_sweep.sh`, so
the merge gate structurally could not see the class. Closed by `w-book3`.

**The generalization this lane takes from it** — and it is the one clause of
w-cross's `X-c` that took no board number: *seeing a name* and *being able to
emit a body under it* are different claims, and W-ADOPT's test conflated them.

**So this lane's rule, frozen here:** it will not remove, weaken or condition
`GlBindStop::Name26Introduced` unless the writer can already express the
resulting layout, demonstrated on a real obj **before** the clause is touched.
A survey that ends in a priced decline is the expected outcome and is a success.

---

## 3. PREDICTIONS

Registered as a **conjunction**: an antecedent that only makes the registered
clause true is not the antecedent the claim needs — each row below also carries
*"and no earlier clause fires"*.

| # | claim | p | antecedent the claim actually needs | falsifier |
|---:|---|---:|---|---|
| **P1** | **This lane ships no change to `crates/` and declines with a price.** | **0.85** | that the survey finds ≥ 2 independent unpaid mechanisms *and* that none of them is a pure routing change already implemented | a `/Gy`-style COMDAT path that already emits the exact section table, characteristics word and aux `Selection` of a read TU's real obj, needing only a route |
| **P2** | **`fnbyte-exact` delta is exactly 0** (35810 → 35810). | **0.93** | P1 holds **and** no instrument-facing key is touched. Conditional on P1 | any `crates/` change lands; or a scan reads ≠ 35810 for a reason this lane did not cause (a peer merge — re-derive at the rebased base before scoring) |
| **P3** | **Of the 380, the number that would BIND under a counterfactual walk with `Name26Introduced` removed and nothing else changed is 0.** | **0.80** | that `Bindings::selective`'s clause 3 or clause 4 fires on every one of them — i.e. the counterfactual walk yields `records < segments` (clause 4, `EmitSetUnknown`) or leaves an unclaimed mangled / inline-fit run (clause 3, `Unaccounted`) — **and** that no *earlier* clause (`OffsetNotASplitPoint`, `RecordsDoNotAdvance`, `NoRecords`) is what actually fires, since an earlier stop is a different repair address | any TU of the 380 whose counterfactual walk yields records **1:1** with the `.ex` segments with no unclaimed run — that TU binds |
| **P4** | **Of the 380, the number that would CONVERT (`class == match`) under that same counterfactual is 0.** | **0.97** | P3's antecedent **or**, independently, that `body-out-of-class` is in the `gate_causes` set of 380 of 380 — so codegen is owed even where the binding is repaired. Two independent sufficient conditions; the row needs only one | a TU of the 380 reaching `class == match` on a scan |
| **P5** | **The existing `/Gy` COMDAT writer does NOT already cover the read TUs: it is not a routing question.** | **0.88** | that c2's real obj for at least one read TU is **MIXED** — at least one COMDAT `.text` and at least one non-COMDAT `.text` in the same obj — which neither `emit_obj` (all packed) nor `emit_comdat_obj` (all COMDAT) can produce, `PortC2::build` selecting between them on the single TU-level `fn_level_linking` flag | every read TU's obj being uniformly COMDAT (then it is a routing question) or uniformly packed (then `26` does not imply a COMDAT at all and #232's premise is wrong) |
| **P6** | **The aux `Selection` byte the read TUs' COMDAT `.text` carries is `IMAGE_COMDAT_SELECT_ANY` (2), not the `NODUPLICATES` (1) `writer::COMDAT_SELECT_NODUPLICATES` hard-codes.** | **0.70** | that #232's *"chars `0x60401020`, `SELECT_ANY`"* is a property of the `26`/implicit-special-member class and not of that one 4-line reproducer. Registered because it is the row that could actually go wrong and it is checkable in one `gt_dump.py` | a read TU's COMDAT `.text` aux reading `Selection = 1` |
| **P7** | **The 380 are not 380 independent problems: under the `dstem` collapse (#2243), the `26`-introduced names on the read TUs are dominated by implicit special members and template instantiations of a small number of stems.** | **0.75** | that the read sample's `26`-introduced name multiset collapses to strictly fewer stems than names, by a factor ≥ 2 | a read TU whose `26`-introduced names are ≥ 90 % distinct stems |

**P5 is the deciding row.** It is the one the commission asks for by name
(*"the cheapest possible outcome and it must be checked before anything is
designed"*), and it is the only row whose falsification would turn this lane
from a decline into a ship.

**An unlosable row, and its falsifiers written down** (the standing lesson: one
lane lost at 0.96 by registering an unlosable row; the next wrote its falsifiers
down and was vindicated). **P4 at 0.97 is the unlosable row.** It is falsified
by a single TU of the 380 reaching `class == match`, which requires the walk
repaired, clause 4 discharged, `body-out-of-class` retired for that TU, AND the
section-layout writer built — four things this lane will not build. If any peer
lane lands a factor-A model mid-session, P4 must be re-scored at the rebased
base, not at freeze.

---

## 4. WHAT THIS LANE DECLINES TO RE-PRICE

* the seven / six / nine mechanisms on `vec.cpp` and `decomp_pch.cpp`
  (`w-phase7b` §4–§5, `w-selbind` #2827) — read, not re-derived;
* factor C's greedy ladder (#2.4) — declined twice already by `w-rdata` and
  `w-rtti`, and C is necessary-not-sufficient (§2.5);
* the emit-order rule (#259, `w-order`) — it is the *ordering* half of the same
  model and this lane does not touch it.
