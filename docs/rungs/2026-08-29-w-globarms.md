# W-GLOBARMS — gate A's twelve arms: the `kind` byte has one writer, and it keys the whole gate on COFF linkage

    Tag:       W-GLOBARMS
    Slug:      w-globarms
    Date:      2026-08-29
    Kind:      characterization — docs/rungs/README.md "Lane kinds"
    Outcome:   instrument
    Fixtures:  none — characterization lane. Two grids live in
               docs/whitebox/grids/w-globarms/, deliberately NOT in
               fixtures/cpp/ (that would move the census; w-globobj and
               w-regcells made the same choice for the same reason)
    Census:    +0 — nothing is admitted, no crates/ file is touched
    Reach:     0, as predicted. `git diff master..HEAD -- crates/` is empty
    Record:    docs/whitebox/WB_GLOBARMS_FINDINGS.md
    Board:     #3808–#3813
    Lane:      L4 of docs/ADOPTION_BRIEF_2026-08-29.md

**Outcome word: `instrument`.** The lane read all twelve arms, built the grids
and the grader it was commissioned for, converted four arms on
`P_GLOBREGS.md`, and classified the other eight with the two-bodies rule. It
converted no TU, adopted nothing into `crates/`, and **defined no `ported`
numerator** — which is the shape the brief priced it at.

---

## 1. What it admits, and what it refuses

**Admits, each `[O]` with its witness named:**

* **A6** (`0x10b5513e`, kinds 4 and 5) **and its internal aliasing test** —
  `gb_pair_yescape` (`x → r31`, `y → stw 10, 80(1)`) against
  `gb_pair_xescape` (the exact mirror). Two `int` locals of one type in one
  body; **same kind, same arm, same TU, same profile**, differing only in which
  address escapes. `sym+0x05 & 2` is **per-symbol**.
* **A4** and **A11's accept side** — `ga_temp`: `return f1(x) + g1(y);` holds
  `f1`'s **unnamed** result in `r30` across the second call with no frame
  traffic. A kind-3 temporary is a candidate.
* **A3's consequence** — `ga_structmix` puts an `int`, a `char`, a `short`
  **and a `long long`** member all in callee-saved registers with no frame
  traffic, which is `w-globobj`'s member-wise aggregate finding given its
  mechanism.

**Refuses to claim**, in the findings' own words:

* **A8**, whose three MEMORY cells are measured and **not banked** — a symbol
  with a COFF record must be observable across an opaque call for language
  reasons, so gate A and the object model predict the same thing (**#3811**);
* **A10** and **A12**, filed `UNCOMP` with the cell that would decide each,
  not `CONSTR` — the shapes are nameable and this lane did not build them;
* any **`ported` numerator** on the twelve-arm population (decision 21 §4,
  `#3505`). §5 says what it hands the owner instead;
* the reading of `sym+0x05 & 2` as *"address taken"* — `gb_addr_local` refutes
  that; *escape* is `[I]`, the partition is `[O]`.

Ships **no `crates/` change**, adds **no `DISCLOSURE.md` row** (nothing was
adopted, so none is owed), adds **no gate row** (`#3691`), commits **no obj**.

---

## 2. Why the arms were unreadable, and it was one function away

`P_GLOBREGS` §3 has carried gate A as twelve tests over `sym+0x04` since read
R4. It never said what a value of that byte *is*, and without that no arm has a
witness: you cannot build a cell for *"kind 6 is rejected"* if nothing connects
kind 6 to a line of C++.

> **`FUN_10bd2913` (`0x10bd2913`) is c2's front-end → back-end symbol map**, and
> it is the deciding writer. The kind is a `dec`-chain on the `.gl` record's own
> kind byte and, for a *data* record, an **8-entry jump table at `0x10bd2a9f`
> indexed by the 3-bit linkage field `([gl+0x37] >> 0x15) & 7`.**

`P_SYMBOL.md` §3 **already reads that field**, at `0x10b28bb4`, and records that
linkage `∈ {1,3}` is *"a linkage class that is suppressed outright"* — no COFF
record at all. Linkage 1 → kind 4 and linkage 3 → kind 5:

> **Gate A's A6 arm is exactly the set of symbols that never reach the object
> file.** Everything that does get a COFF record arrives at A8 or A9 as kind 7,
> 8 or 9.

**The two reads sat on adjacent pages in this repo for six days and neither
cited the other.** That is the finding's actual shape, and it is the read-first
doctrine paying: the alternative was a probe grid over C++ storage classes,
which would have measured a partition and never named the field.

---

## 3. The axis on which this lane could fail, and it did — six times

`#3336`: a control never watched fail is decoration.
`work/w-globarms/CONTROLS_RED.txt`.

| control | outcome |
|---|---|
| C1 — `ga_int` PROMOTED / `ga_vol` MEMORY, both profiles | **fired both ways** |
| **five planted defects** against the final grader | **all five RED** — two abort the decode outright, three fail named assertions |
| C4 — the **cross-grader**: `grade_globobj.py`'s independent readout on the same dumps | **agrees on all 38 cell/profile verdicts** |
| `--selftest` | **16 assertions, 3 of them the grader having to REJECT** a mutated image |
| premise test | **0 of 38 cells scored `U`** |
| the pair cells' mirror | `gb_pair_xescape` is the exact reversal of `gb_pair_yescape` |

**And the lane's own instrument was wrong once, found by a control, and the
number it would have published was a table of misses:**

> Planted defect 5 removed the relocated-static arm of the frame-traffic
> readout. The three A8 cells flipped to PROMOTED and the run printed
> **`GRADE: PASS  (3 prediction misses — a RESULT, not a failure)`**.
> `--selftest` caught it; the `--arms` path did not. **A dead readout was
> publishing a table that reads like a finding.** Fixed at `eaeebd42a` — the
> four synthetic readout assertions are now a *precondition* of the cell half.

That is `w-globobj` §2.6's third defect in a different costume: a control that
reports on one path while the publishing path stays green. Second wave running.

**One assertion is worth naming on its own**, because it is what makes the
answer key the image's rather than the instrument's: patching `cmp al,5` to
`cmp al,7` at `0x10b5513f` in a fake image **moves kinds 6 and 7 into A6**, and
`--selftest` checks that it does. A constant living in `grade_globarms.py`
could not do that.

---

## 4. Estimate vs outcome — the prereg score, by tier, never pooled

| tier | commit | predictions | hits | misses | ungraded |
|---|---|---:|---:|---:|---:|
| **PREREG** — before the image was opened, before any cell | `a0e5b58a3` | 34 | **31** | **1** | 2 |
| **ADDENDUM 1** — after the read, before the compile, **committed after** | `e3835448f` (grid header) | 6 | **6** | 0 | 0 |

**Addendum 1's weaker provenance is named rather than smoothed over.** Its
predictions were written into `arm2_grid.cpp`'s header before the grid was
compiled, but the addendum file itself was committed after the dumps existed.
That is not PREREG's tier and it is not reported as if it were.

**The ceiling held.** `PREREG.md` §4 registered, before the image was opened,
*"at most 5 arms convert, at least 6 are `CONSTR`"*. Outcome: **4 converted, 6
`CONSTR`, 2 `UNCOMP`.** Registering a ceiling before the deciding cell is what
made `w-regcells`'s negative result credible and it is why **#3811**'s three
declined cells are a refusal rather than a shortfall — banking them would have
taken the count past the lane's own registered bound on a confounded mechanism.

**The one MISS is the useful one.** K3 predicted kind 10 was the *aggregate*
arm, and `w-globobj`'s member-wise finding made that look obvious. It is wrong:
kind 10 is **extern/alias**, a local aggregate is a kind-4/5 symbol on the
general path, and A10's `t+0x20 == 4` width test never sees it. `ga_structmix`
is the cell that says so — a **`long long`** member promoted alongside a `char`
one, which a 4-byte width test forbids.

**And one prediction whose refutation would have cost half the lane.**
`PREREG.md` §4.1 registered **G1** at p = 0.20: if any reader of the reject
tail's counter `DAT_10c2e454` reached an emitted artifact, A1/A3's *silent*
skip and A5/A7/A9's *charged* reject would separate in an obj and the `CONSTR`
classification would be wrong for five arms. **`DAT_10c2e454` has exactly two
references in the image, both writes, both inside `FUN_10b550e5`** — corroborated
by an objdump displacement scan and by Ghidra's xrefs independently. G1 **HIT**,
and the wall stands on a measured absence of readers.

---

## 5. What moved

`c2rs subsys`, `[globregs]` row, re-derived on this tree at both ends:

```
before   2 agreement : marks [O] 21 of 74 (28.4 %) — [R] 49 [I] 4
after    2 agreement : marks [O] 29 of 100 (29.0 %) — [R] 66 [I] 5
```

**+8 `[O]`; the denominator grew by 26 because the lane filed new `[R]` residue
rather than closing questions silently** — §3.1's kind enum, §2's two corrected
allocator rows, §6.4's sharpened recycling wrinkle, §10.4's arm classification
and the two `UNCOMP` cells. That is the same correct movement `w-globobj` and
`[inline]` showed: a denominator that grows because honest residue was filed is
not a regression, and the number to watch is the `OBS` bucket's fill rate
(`#3776`).

**Nothing else moved and nothing else should have.** `read` is unchanged (no
new site was opened against the R4 denominator), `exercised` is unchanged
(still `RESIDUE`), `ported` stays `RESIDUE` (decision 21 §4), `byte-owned`
stays **cited at `#3534`, never re-taken**.

### The population exists, and this lane defines nothing on it

The brief asked for the arms and forbade a numerator. The read makes the
temptation concrete, so it is named rather than left implicit and handed up:

> **There is a defensible site-level population for globregs: 12 gate-A arms,
> each with an address, the kind values that reach it, and a classification.
> 4 of 12 have an obj witness.**

Two facts the owner would have to settle first, both in **#3809**:
**6 of 12 are `CONSTR` for one shared structural reason** (every rejecting arm
branches to `0x10b552b8`), so a ratio would carry a 6/12 ceiling on day one —
`#3776`'s trap in a new place — and **the arms are not equally weighted**: A6
and A8 cover every symbol a C++ compiland declares while A1 covers **one record
per compilation**. Twelve equal sites measure the binary's branch structure,
not the compiler's behaviour.

---

## 6. Handoffs

* **A lane wanting A10**: build an aggregate that is an **undefined external**
  and assign it member-wise across a call, so a kind-`0xa` symbol reaches A10
  *with sub-symbols* and `t+0x20 == 4` bites. `ARMS.tsv` names it; **do not
  re-file it `CONSTR` without the two bodies** (`#3776`'s rule).
* **A lane wanting A12, or A11's reject side**: read what sets `sym+0x07 & 0x40`
  and `sym+0x14` on a kind-3 temporary. Both are `UNCOMP` because the *cause*
  is unread, so the cell cannot be written — the same state `w-globobj` filed
  `aux+0x18` in.
* **`P_SYMBOL.md`'s owner** (not this lane's seam, **not edited**): §3's linkage
  field has a **second consumer**, `FUN_10bd2913`'s jump table at `0x10bd2a9f`,
  which reads **all eight** values where §3 reads only the `{1,3}` suppression.
  **Entry 0 of that table is a null slot** — linkage 0 is unreachable by
  invariant and c2 would jump to address 0 if it ever arose.
* **`DISCLOSURE.md`'s reader**: `sym+0x04` means two different things in
  `W-STAGETAP-4` (the globregs kind byte) and `W-STAGETAP-6` (the `.gl` record's
  name pointer). `FUN_10bd2913` is the bridge. Nothing was adopted here so no
  row is owed; the ambiguity is a reading hazard.
* **Anyone porting the candidate set** (not this wave — decision 20 §2): the
  parameter to expose is **linkage class**, not variable kind, and the second
  one is the **per-symbol** escape flag.

---

## 7. Gate evidence

### ⛔ THE ONE RED, AND IT IS THE SAME SEAM CONFLICT `w-globobj` HIT — READ THIS BEFORE MERGING

`subsys::tests::the_mark_census_reproduces` **fails at this tip**, for exactly
the reason `docs/rungs/2026-08-28-w-globobj.md` §7 documents: **`P_GLOBREGS`'s
mark census is pinned as a constant inside `crates/c2-harness/src/subsys.rs`**,
this lane is assigned that page *and* barred from `crates/`, and every `[O]` it
was dispatched to add reddens the assertion.

```
crates/c2-harness/src/subsys.rs:2817  assertion `left == right` failed
  left: (66, 29, 5)      <- P_GLOBREGS.md at this tip
 right: (49, 21, 4)      <- the pinned baseline (w-globobj's re-bless)
```

**The coordinator's re-bless is one line:**

```rust
assert_eq!((gr.read, gr.obj, gr.inferred), (66, 29, 5));
```

**This lane did not make it** — `#3748`'s doctrine is that a re-bless belongs in
a diff a reviewer reads, and `reach 0` is the check the brief uses to verify the
lane stayed in its seam. The triple is re-derivable without building anything:

```
$ python3 -c "t=open('docs/whitebox/ref/P_GLOBREGS.md').read(); b=t[t.find(chr(10)+'---'+chr(10)):];
  print([b.count(m) for m in ('[R]','[O]','[I]')])"
[66, 29, 5]
```

`eh` is untouched at `(27, 14, 0)`.

**This is the second consecutive wave in which the conflict has fired**, which
is now a fact about the wave's design rather than about either lane. `w-globobj`
§5.1 already named the two exits: either the pin moves out of `crates/` — it is
a documentation baseline, not port behaviour — or a page-owning lane gets a
one-line exemption named in its brief. Neither was taken, so it fired again.

### THE OTHER TWO REDS, AND NEITHER IS A DEFECT — one is generated, one is a PEER'S WORKTREE

`cargo test --workspace --release --no-fail-fast` at this tip:
**1,991 passed, 3 failed, 2 ignored.** The three, named individually because
"three failures" is not a reportable state on its own:

| test | why | whose |
|---|---|---|
| `subsys::tests::the_mark_census_reproduces` | the pinned `P_GLOBREGS` triple — the box above | **this lane's, by design** |
| `rung_registry::rung_index_is_generated_and_current` | *"`docs/rungs/INDEX.md` is stale — it is GENERATED. Run `scripts/gen_rung_index.sh`."* This rung file is new and `INDEX.md` is regenerated **at merge**; the brief bars this lane from touching it | **this lane's, by design** |
| `wt_pin_audit::no_worktree_holds_an_unlocked_pinned_artifact` | **not this tree.** The guard scans **all nine** worktrees on the box and reports `UNLOCKED AND PINNED /…/w-fmadd`, holding `work/w-fmadd/sweep_fp/c2rs` and `work/w-fmadd/sweep_fp2/c2rs`. `w-globarms` appears nowhere in its output | **`w-fmadd`'s** — a wave-19 peer, remedy `scripts/wt_pin_audit.sh --lock` |

**The third one is worth a sentence beyond "not mine".** `wt_pin_audit` is
worktree-**global**: it fails in every concurrently-running lane's tree the
moment one lane leaves an unlocked binary, so on a five-lane wave a single
lane's omission reddens the suite for all five. That is the same
cross-worktree coupling `#3545` documents for its own WIDE counters, and a
lane reading its suite output without opening this failure would report a
red it did not cause.

### Everything else

    GATE: PASS — 18/18 lanes ran and every one of them graded a corpus,
      the sweep graded 19460 of 19556 generated cases and the cross graded
      90424 of 90812 case-lane cells, with 0 mismatches anywhere
      (96 sweep cases carried ungraded — the reference rejects the source),
      and 18/18 lanes ran again through a DEBUG-profile c2rs for
      7038 more fixture-verdicts at 0 panics.

    graded tree: bcb7e1dfff2a  (806 files: crates fixtures scripts, content-hashed)

Unqualified `GATE: PASS` (`#3786`), read off the verdict **line**, not the exit
code. Transcript `work/w-globarms/gate_tip.out`; suite transcript
`work/w-globarms/tests_tip.out` (**1,991 passed / 3 failed / 2 ignored**);
instrument transcripts `work/w-globarms/GRADE.txt` (`SELFTEST PASS`, 16
assertions; `GRADE: PASS`, 38 graded, 0 `U`, 0 misses) and
`work/w-globarms/CONTROLS_RED.txt` (five planted defects RED, C4 agreeing
38/38).
