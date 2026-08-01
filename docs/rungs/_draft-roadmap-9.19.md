DRAFT for `docs/ROADMAP.md` §9.19 — written by lane `w-slotarg`, to be landed by
the coordinator. Nothing in §1–§9.18 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-slotarg-prereg.md`, committed at `dbd104b` before the
first capture; the out-of-sample registration is
`docs/rungs/_2026-08-01-w-slotarg-grid3-prereg.md`, committed at `6caeddc`
**before grid 3 was compiled**.

---

### 9.19 W-SLOTARG — the +356 is real, the naive lowering is wrong, and the ordering rule survives 360 cells and dies on the 361st (2026-08-01)

Board **#149** (the off-add ARGUMENT slot, 356 emitted, `crates/c2-core`) and
**#150** (`expr-op-0x27` is worth 6). **The conversion is DECLINED.** What is
delivered instead is the diagnosis §9.17 asked for, the ordering rule measured
over three capture grids, and — the result — **the rule's refutation on the grid
it did not see.**

* **`Mismatch @ offset 8` is a LENGTH telescope, not a header defect.** One
  missing instruction word, surfacing at the first size-dependent field in the
  file.
* **WR1's address-last rule mis-emits 654 of the 728 captured cells that have a
  walk (89.8 %).** It is not a safe default for this construct.
* **A rule fitted to 360 in-sample cells mispredicts 98 of 394 out-of-sample
  cells.** Every miss is an r11 pre-save, on the one axis the first two grids
  could not vary. Had it shipped, the 878-TU differential would have read
  **6 match / 0 mismatch** over it — §9.17.7's blind spot, for the fourteenth
  time.

---

#### 9.19.1 Why offset 8 — and it is not where the bytes are wrong

§9.17.9 recorded `pn.cpp` → `Port=Mismatch @ offset 8` under the `ceiling` sink
and left it undiagnosed. Offset 8 is early enough to look like a header,
section-count or relocation consequence. It is none of those.

Reproduced at this base (`74d0744`) on a three-line probe, then both objs parsed:

| | c2 | port under `ceiling` |
|---|---|---|
| `NumberOfSections` | 5 | 5 |
| `NumberOfSymbols` | 15 | 15 |
| `.text` relocations | 1, sym 14, type `0x0006` | 1, sym 14, type `0x0006` |
| the four sections before `.text` | 132 / 152 / 16 / 16 B at 220 / 352 / 504 / 520 | **identical** |
| **`.text SizeOfRawData`** | **8** | **4** |
| `.text` words | `38840008` `4bfffffc` | `48000000` |
| **`PointerToSymbolTable`** | **554** | **550** |
| total | 891 B | 887 B |

The sink drops the offset, so codegen is handed `[Load(t)]`; `t` is formal 1,
argument slot 1 is r4, `t` is *already* in r4, and the port emits **nothing** for
the argument where c2 emits `addi r4,r4,8`. `.text` is one word short.

`.text` is the last section, so its 4-byte shortfall lands in
`PointerToSymbolTable` — and that field lives at **file offset 8..12**, ahead of
every byte of section payload (COFF header 0..20, five section headers 20..220).
**Offset 8 is simply the earliest byte in the file that can show that a function
got shorter.** The branch word also differs (`4bfffffc` vs `48000000`), but only
because it now sits at `.text` offset 0 instead of 4 — a consequence of the same
missing word, not a second defect.

Taken to the byte rather than asserted: `0..4` and `12..20` are identical, the
**316 bytes of the four sections that precede `.text` are identical**, and inside
the five section headers exactly **three** bytes differ —

| byte | field | ref → port | |
|---:|---|---|---|
| 196 | `.text SizeOfRawData` | 8 → 4 | the missing word |
| 204 | `.text PointerToRelocations` | 544 → 540 | the same 4 bytes, downstream |
| 217 | `.text Characteristics` | `0x60400020` → `0x60401020` | **not the sink** — see below |

— and the third is the `prefilter` `/Gy` confound of the note below, absent from
the differential's own emission, which is why `c2rs diff` reports offset 8 and
not 217.

The control that says the probe can also pass: the same probe with the member at
offset 0 (`pz.cpp`) reads **`Port=Match`** under the `zero` sink at this base.

**A methodological note worth keeping.** The first extraction of the two objs was
done with `c2rs prefilter --emit-obj`, and it reported a divergence at byte 217
on a body the differential grades `Port=Match`. `prefilter` derives
function-level linking from the flags (`/O1` implies `/Gy`) while `differential`
does not, so the port emitted a COMDAT `.text` (`0x60401020`) against the
reference's packed one (`0x60400020`). **`prefilter` is not a valid instrument
for byte forensics against an obj captured by another path**, and the reference's
own `S_OBJNAME` must be read out of its `.debug$S` and passed back as
`--obj-name` or the comparison measures the output path.

#### 9.19.2 The 356 does not age, and the workload still cannot grade it

Board quantities age (§9.17's A1 aged by 11,406), so it was re-measured before
anything was registered against it. 878-TU dc3 workload, at `74d0744`:

| | bodies | emitted | Δ emitted |
|---|---:|---:|---:|
| base | 706,402 / 2,462,571 (28.69 %) | 36,059 / 178,968 (**20.15 %**) | — |
| `C2RS_SINK_OFF_ADD_ARG=ceiling` | 707,873 (28.75 %) | 36,415 (**20.35 %**) | **+356** |

Identical to §9.17.5 to the function. Both runs read **6 match, 0 mismatch,
census/gate disagreement 0** — the ceiling sink provably mis-emits (§9.19.1) and
the differential is silent, which is why nothing in this section is graded on it.

#### 9.19.3 The capture grid, and the two constructs that are not the same construct

Three grids of c2's own `.cod` listing (`c2rs listing`, non-perturbing),
**754 in-domain cells**, crossed rather than sampled:
`scripts/slotarg_grid{1,2,3}.py`, read by `scripts/slotarg_read.py`, rule in
`scripts/slotarg_rule.py`.

Grid 1 (240 cells) — designator steps 1/2 × offset 0/8/0x8000/0x10000 × arity
1–5 × address slot × free/member caller. Grid 2 (120) — the base formal parked
**above** every slot the call writes. Grid 3 (402, 394 in domain) — the base in a
**middle** register, offsets straddling the 16-bit boundary from both sides,
arities 6–8, and a two-step designator straddling the boundary.

Four facts hold across all 754:

1. **Two designator steps are one `addi`.** `&t->s.k` at (0x7ffc, 8) emits a
   single add of 0x8004; the steps=1 and steps=2 cells are byte-identical
   wherever the sums agree. §9.17.5's "`-more` is an arity artefact" confirmed
   from the emitter side.
2. **A wide offset splits.** `k ≥ 0x8000` is `addis`+`addi`, and a zero low half
   collapses to a bare `addis` — the "wide literal with a zero low half" hazard,
   present and load-bearing.
3. **Offset 0 with the base already in the destination emits nothing** — #127's
   arm, reproduced here.
4. **The address is NOT emitted last.** WR1's rule agrees with c2 on **74 of the
   728 cells that have a walk (10.2 %)**; it mis-emits **89.8 %**.

And the fact that matters most for whoever takes #149:

**A computed address is not scheduled like a data-symbol address.** For the same
arrangement — the address at slot 0 under a two-word walk — c2 puts a *symbol*
address at walk index 1 and a *computed* address at index 2:

```text
  gs3(&gI, 3, 4)        li r5,4  · addi r3,r11,0 · li r4,3    <- SECOND  (§9.13.1)
  f3_0(&t->k, 11, 12)   li r5,12 · li r4,11 · addi r3,r3,8    <- its descending slot
```

`sym_slots_text` is therefore **not reusable** for the off-add, and
`a_computed_address_is_not_scheduled_like_a_data_symbol_address` is the portable
assertion that says so. Verified discriminating: mutating `sym_slots_text` back
to address-last reds it and one other test and leaves **87 green** — §9.12's pin,
a mutation that reddens everything identifies nothing.

#### 9.19.4 The rule agreed on 360 of 360, and that was worth nothing

A rule was refined against grids 1 and 2 until it reproduced all 360 cells
exactly — mnemonic, both registers and the immediate. Four refinements:
descending merge → "never the first setup word" for the wide form → "the low half
closes early when the base is clobbered" → the `mr`/`addi` vs `addis`
asymmetry. It looked finished.

**It is fitted.** Grid 3's predictions were generated and committed at `6caeddc`
*before* grid 3 was compiled, and scored **296 / 394 (75.1 %)** — below the
registered floor of 300.

**All 98 misses are the same miss.** Every one is an **r11 pre-save that the rule
did not expect**, and the rule predicted a pre-save correctly exactly once:

```text
  w_32764_3a_0_m1     (the base formal in r4 rather than r3)
    predicted   addi r3,r4,32764 · li r5,12 · li r4,11
    c2 emits    mr r11,r4 · li r5,12 · li r4,11 · addi r3,r11,32764
```

The axis is **the base formal's own register position**. Grids 1 and 2 always
parked the base at the lowest slot, so the clobbering `li` was always the *last*
walk word, and "hoist the address ahead of the walk" was never separated from
"hoist it ahead of the clobber". §9.13.1's third consequence, in a third costume:
*an axis the generator does not vary is exactly as invisible as a fixture that
does not arrange the case* — and this time the generator was written by a lane
that had just read that sentence.

It is also **the refusal `sym_slots_text` already carries**: "at two shifting
formals c2 pre-saves into r11 … which one probe does not separate". Here it fires
at **one** shifting formal, so the existing refusal's stated reason understates
its own scope.

**And it is not one refinement away — which is the part worth handing over.**
Over grid 3's 394 in-domain cells the pre-save fires 99 times, always inside the
298 cells whose base is clobbered, and one further axis nearly separates it:

| clobbered cells | pre-save | no pre-save |
|---|---:|---:|
| address destination **below** the base | **90** | 0 |
| address destination **above** the base | 9 | **199** |

Nine exceptions, not zero — and they are a coherent family: **wide offset with a
non-zero low half, destination exactly one register above the base, and the
base's own literal is the first word of the walk.**

```text
  w_32768_3a_2_m1     base r4, dest r5
    fitted rule   mr r11,r4 · li r4,11 · li r3,10 · addis r5,r11,1 · addi r5,r5,-32768
    c2 emits      mr r11,r4 · li r4,11 · addis r5,r11,1 · li r3,10 · addi r5,r5,-32768
```

So the pre-save arm is wrong twice over: about *when* it fires, and about *where
the computation goes once it does* — c2 interleaves the address into the walk
rather than appending it. **The last time this lane had a residue of 0 it was
wrong on the next grid**, and a residue of 9 on an axis discovered by the grid
that broke the rule is not a basis for a fifth attempt.

#### 9.19.5 DECLINE, under the rule registered in advance

`_2026-08-01-w-slotarg-prereg.md` registered: *if the measured rule cannot be
stated as a total function over the grid, it is refused, not fitted* — and the
grid-3 registration added *if O1 lands below its floor the rule is refused, not
patched again*. It did. **A fifth refinement against a third grid is fitting with
extra steps**, and the arithmetic says what it would have bought: the fitted rule
is right on **656 / 754 (87.0 %)** of everything captured, i.e. it would mis-emit
roughly one call in eight, silently, on a shape the workload differential cannot
see.

**So #149's stock is left unconverted: 356 emitted functions, still blocked.**

Two further constraints are recorded rather than worked around:

* **`SlotArg` is declared in `crates/c2-il`** (`func/mod.rs`, plus the
  `pub(crate)` twin in `func/body/mod.rs`), and lane `w-emitset` was live there.
  So even a proven rule could not have shipped end to end from this lane; the
  Δ emitted here is **0 by construction, not by measurement**, and it was
  registered that way before any of this was run.
* The port stays **honestly unable to represent the shape**. The exhaustive
  `match` in `the_computed_address_schedule_is_not_established_and_has_no_slot_variant`
  stops compiling the moment a variant is added, which is where the next lane
  will read §9.19.4.

#### 9.19.6 Pre-registration score — 9 of 13, and the two misses are the section

| | registered | measured | |
|---|---|---|---|
| G1 | c2 emits ≥ 200 of 240 grid-1 cells | **240** | HIT |
| G2 | ≥ 2 schedule shapes; 4, [2, 8] | **36** raw sequences in grid 1 alone | **MISS**, above the ceiling |
| G3 | ≥ 1 r11 pre-save; 30, [10, 150] | grid 1 **8**, grid 2 **0**, grid 3 **99** | HIT on the phenomenon, **MISS** on grid 1's interval |
| G4 | `k ≥ 0x8000` is not one `addi` | `addis`(+`addi`), zero low half collapses | HIT |
| G5 | `k = 0`, base in place, emits nothing | **YES** | HIT |
| G6 | the address's position differs slot-0 vs last | **YES** | HIT |
| S1 | Δ emitted from this lane | **0** — a constraint, declared in advance | — |
| S2 | address-last mis-emits ≥ 50 %; 65 %, [20 %, 90 %] | **89.8 %** | HIT, at the ceiling |
| S3 | portable assertions 5, [3, 10] | **2** | **MISS**, below the floor |
| S4 | the control stays green under every mutation | **PASS** — 87 green, 2 red | HIT |
| S5 | tip workload scan identical to base | **identical** | HIT |
| S6 | gate/selftest/sweeps unchanged | **unchanged** | HIT |
| S7 | verdict: partial — rule shipped, conversion declined | **rule REFUTED**, conversion declined | **MISS**, and it is the finding |
| O1 | out-of-sample 370/402, [300, 402] | **296 / 394** | **MISS**, below the floor |
| O2 | the failing axis is the middle-clobber one | **it is, all 98 of it** | HIT |
| O3 | boundary offsets need no new arm | **YES** | HIT |
| O4 | arity 6–8 needs no new arm | YES; arity 8 spills the base to the stack (8 cells, out of domain) | HIT |

* **O1 is the section.** It was registered at 92 % by a lane that had just
  watched its rule reproduce 360 of 360, and the honest floor it set is what
  turned a shippable-looking result into a decline. **The value of the
  out-of-sample grid was entirely in its being generated before it was seen** —
  had grid 3 been captured first, the fifth refinement would have been
  irresistible and the rule would have looked finished again.
* **G2's miss is the same error as §9.17's C1**: a shape count registered as a
  small integer when the quantity was a cross-product. The 36 raw sequences are
  reproduced by a rule with **4 ordering arms**, and 4 was the point estimate —
  the registration named the wrong noun, not the wrong number, and an interval
  on "shapes" could not have been right about either.
* **S3 under-delivered on purpose.** Five assertions were registered for a rule
  that would ship; two is what a refuted rule can honestly support, and inventing
  three more would be pinning a rule this section says is wrong.
* **S7 is a miss in the good direction.** The lane expected to ship the rule and
  decline only the conversion. It is declining both, and the reason is a
  measurement that only exists because the decline rule was written down first.

#### 9.19.7 Gate evidence

At `e9a56f5`, worktree branched from `origin/master` **609 commits behind**
(`4ea415a`) and reset to `master` `74d0744` before any work — the **fifth** lane
this week to meet that, and the third to have it be the first thing it found.
Cache addressed by its canonical main-repo path.

* `cargo test --workspace` — base `74d0744` **596 passed, 0 failed, 1 ignored**
  → tip **598 passed, 0 failed, 1 ignored**. Both measured, not inferred (base
  rebuilt from `git checkout 74d0744 -- crates`). **`#[test]` grep over
  `crates/` 597 at base → 599 at tip**, reconciling with the runner at both ends
  once the one `#[ignore]`d test is added (597 = 596 + 1, 599 = 598 + 1).
  **Target count 24 at base and 24 at tip** — no target was added, so the two new
  tests are in the lane `differential.rs` does not grade, which is why
  `scripts/gate.sh` is quoted separately below.
* `c2rs selftest` — **210 PASS, 0 FAIL, 0 skip**.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes, 0 FAIL / 0 SKIP /
  0 NO-RESULT, **2,520 fixture-verdicts, 0 mismatch in every lane**.
  `--selftest` PASS, 15 cases.
* `scripts/expr_sweep.sh` — 47 fragments, **14,484 cases, mismatches=0**.
* `scripts/cross_sweep.sh` — 42,719 configurations × 12 lanes =
  **512,628 gradings, 0 mismatches**; 406 of 406 declared family pairs reached
  and emitted; refusal-frontier residue **0**.
* 878-TU workload scan at tip — **identical to base on every headline number**:
  6 match, 0 mismatch, 865 vocab-gap, 7 capture-fail; bodies 706,402 / 2,462,571
  (28.69 %); emitted **36,059 / 178,968 (20.15 %)**; census/gate disagreement 0.
* The two sink scans on the same binary, for the counterfactuals quoted above:
  `ceiling` emitted **36,415** (**+356**, and it mis-emits), `expr` emitted
  **36,065** (**+6**). Four 878-TU scans in total — base, tip, `ceiling`, `expr`.
* Probes — `work/pn.cpp` `Port=Mismatch @ offset 8` under `ceiling` and
  `Port=NotImplemented` under `zero`; `work/pz.cpp` `Port=Match` under `zero`.
* Grids — 240 + 120 + 402 cells, all emitted by c2; 8 arity-8 cells spill the
  base to the stack (`lwz r11,t$(r1)`) and are excluded from the rule's domain by
  name rather than dropped silently.

**No fixture was added and no `fixtures/cpp/` entry changed**, because nothing
shipped — §9.17.9's rule, and the same reasoning: a fixture for a shape the port
refuses would put a claim in every gate lane this lane did not earn.

**Reproduction.** The grids are generated by committed code
(`scripts/slotarg_grid{1,2,3}.py` → `c2rs listing <cpp> --out <cod>` → read with
`scripts/slotarg_read.py`, scored by `scripts/slotarg_rule.py`), so nothing here
depends on a scratch directory. The two one-off probes are three lines each and
are given in full rather than named, because §9.17.9's `pz/ph/pn.cpp` live under
a gitignored `work/` and could not be re-run from the section that cites them:

```cpp
// pn.cpp — Mismatch @ offset 8 under `ceiling`, NotImplemented under `zero`
struct S { void one(int*); };
struct T { int pad0; int pad1; struct { int k; } s; };
void a1(S* s, T* t) { s->one(&t->s.k); }

// pz.cpp — the control: Port=Match under `zero`
struct S { void one(int*); };
struct T { struct { int k; } s; };
void a2(S* s, T* t) { s->one(&t->s.k); }
```

#### 9.19.8 Board items

* **#149 stays open at 356, and its cost is now known.** The variant is trivial;
  the ordering rule is **not established**, and the next lane starts from
  `scripts/slotarg_grid{1,2,3}.py` (754 graded cells) plus the 98 witnesses that
  refuted the fitted rule. **Do not re-derive the rule from grids 1–2 alone** —
  they agree with a rule that is wrong one call in eight.
* **#155 — the r11 pre-save is a rule of its own, and it is under-scoped
  everywhere it is mentioned.** `sym_slots_text` refuses it at "two shifting
  formals"; grid 3 fires it at **one**, and 98 of 394 cells need it. It is the
  same object as board **#141** (`call-arg-sym-permuted`), which is sized off one
  probe. Both should be measured on one grid over (base register position) ×
  (walk length) × (wide/narrow offset), which grid 3 already is for the off-add
  half.
* **#156 — `prefilter` and `differential` disagree about function-level
  linking.** `prefilter` infers `/Gy` from `/O1`; `differential` does not. On the
  same source the two emit `.text` characteristics `0x60401020` and `0x60400020`,
  so a body the differential grades `Port=Match` reads `bytes-diverge at 217`
  through `prefilter`. Nothing shipped depends on it, but `prefilter` is the
  reject-only seam a caller is meant to trust, and one of the two is wrong about
  the workload's real flags.
* **#157 — a computed address whose base formal is passed on the stack.** Grid
  3's arity-8 cells lower it as `lwz r11,t$(r1)` and then compute from r11. Out
  of the modeled domain, 8 witnesses captured, named here so a later grid does
  not rediscover it as an anomaly.
* **#150 is closed at 6.** `expr-op-0x27` reproduces at this base to the
  function — **22,759 emitted, 407,016 bodies**, identical to §9.17.6 — and
  granting its named token converts **6 emitted functions**, re-measured here on
  the same binary (`C2RS_SINK_OFF_ADD_ARG=expr`: emitted **36,065** against the
  base's **36,059**). The board should carry **6**, not 22,759. The #1 blocking
  feature on the emitted board is also the least valuable thing on it, which is
  §8.7's rule about blocking-feature counts being queue positions rather than
  quantities of work.
