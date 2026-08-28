# w-inlbudget — c2's inline budget model is in the port, and the port refuses where c2 divides

    Tag:       w-inlbudget
    Slug:      w-inlbudget
    Date:      2026-08-28
    Kind:      construct rung (adoption)
    Outcome:   built
    Fixtures:  none — construct rung: it adopts P_INLINE §6.6.2's budget model into
               splice.rs and registers it as a decision surface, re-expressing an
               already-byte-exact class through it
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Fail axis: THREE, and the byte delta is not one of them — it cannot fail here by
               construction (the divisor is 1 on the admitted set), so it is the
               floor and not the grade. (1) THE REFUSAL DOMAIN — `splice.budget`
               enumerates `n = 1..6` and the port must REFUSE every `n ≥ 2` cell;
               a model that admits one moves `surface/DOMAIN.txt` and the baseline
               test is red. That is the axis `#3723` proved a byte delta cannot
               see. (2) PRECEDENCE — the model is asked inside the chain walk, so
               a body that used to refuse for a named reason refusing for a
               different one is a failure visible in the refusal-reason census and
               in no byte. (3) REGISTRY COMPLETENESS — a `guards` entry whose
               domain cannot reach its const is a FALSE COVERAGE CLAIM
               (`#3746`: 2 of 7 were), and each of this lane's five is graded by
               widening the const and requiring the domain to move.
    Record:    this file; prereg `work/w-inlbudget/PREREG.md`, committed at
               `fa4e059cf` BEFORE the image was opened; the read at
               `work/w-inlbudget/IMAGE_READ.md`; controls at
               `work/w-inlbudget/controls_red.txt`

Charter: `docs/DECISIONS_2026-08-22.md` § Decision 22 §2, the `w-inlbudget` row,
and `docs/ADOPTION_BRIEF_2026-08-28.md` §L2. Dispatched at master `4b79bf46a`.
Board **#3762**–**#3767**.

> **Byte delta ZERO, domain moved 1,306 lines.** `crates/` changed and the
> emitted bytes did not: identity diff **0 lines over 21 rows**,
> `surface/DOMAIN.txt` **1102 → 2408**. This lane added **no `gate.sh` row**
> (`#3691`), adopted **no ceiling** (`#3732`), edited **no clause row**
> (`CLAUSES.tsv` is `w-clausefix`'s) and touched **no** `P_INLINE.md`
> (`w-inlswitch`'s).

---

## 1. What it admits, and what it refuses

`P_INLINE.md` §6.6.2 read c2's recursive inline expansion and published a budget
model the port had **no counterpart to at all**. `splice.rs` now carries it as
`BudgetModel` — nine fields, every one an address in `DISCLOSURE W-INLBUDGET-1`
— with four named entries in `BUDGET_MODELS`, index 0 the default and the other
three instrument states pinned out of every production path by a source scan
(`regalloc::ORDERS`' shape, decision 15's instruction).

**The one thing it does not adopt is `B`.** c2's budget is
`clamp(2 × WORD [fn+0x50], 1000, 35000)`, and §2.1b measured `WORD [fn+0x50]` as
an *upper bound* on the tested quantity rather than the quantity — `arith_012`
and `mix_008` at an identical `SIZE` of 115 with opposite verdicts. So the port
cannot evaluate `B`, and `NestedBudget` carries the **divisor** instead of a
number:

| the port's state | when | why it is safe |
|---|---|---|
| `NestedBudget::Parent` | `n = 1` | the divisor is 1, so the nested budget is the parent's **for every possible `B`** |
| `NestedBudget::Divided { k }` | `n ≥ 2` | it is `B / k`, and the port has no `B` — **`port_enter_site` REFUSES**, `S6-budget-divided` |

That is `#1020`'s hazard, which `w-inlfit` named and could not close:

> *"the moment a lane widens `S2` to two call sites, `n = 2`, c2's divisor stops
> being 1, and the port has nothing to divide."*

It now has a named refusal instead of nothing, and
`two_call_sites_refuse_by_name_and_do_not_guess` asserts it over `n = 2..6` at
every site index — including the correct **admission** of a run's *last* site,
where c2's counter has reached 1 and the division is the identity there too.
Refusing that one would be refusing on the shape of the question rather than
modelling the rule.

**The other refusals by name:** `S6-budget-depth-cap` (C14, `level − base > 16`),
`S6-budget-no-sites`, `S6-budget-site-index`.

**Byte-neutral by construction, not by hope.** The port admits only chains whose
every link has exactly one call site — `S2` requires it, `S6-chain-open`
requires the end to have none — so `n = 1` at every expansion. `w-inlfit` called
that *"a soundness argument for a fit, not a derivation of one"*; this is the
derivation, and `every_admitted_link_has_exactly_one_call_site` asserts its
premise on `t01` and on all three links of `t11` rather than leaving it as prose.

**It is wired in, not parked beside.** `splice_body_why` enters the model once
per chain edge, and the walk's termination ceiling is now expressed on
`exp.level` — c2's own units — instead of a private `seen.len()`. That is the
construct-rung pattern (re-express an already-byte-exact class through the new
machinery) and it is what stops the identity diff being a tautology over dead
code, which `rungs/README.md`'s corollary warns about by name.

## 2. Verified before adopted — V1–V7, and all seven confirm (**#3762**)

The brief said in so many words that §6.6.2 *"is a read by another lane;
re-derive it. If it is wrong, saying so is a better outcome than adopting it."*
Prereg §2.1 fixed the seven claims and §3 predicted **P1: all seven confirm**.
They do. Listings: `work/w-inlbudget/IMAGE_READ.md`, image
`sha256 c80981c0…a66258`, independent objdump disassembly.

| | claim | verdict |
|---|---|---|
| V1 | the recursion edge at `0x10b62402` | **confirmed**; the six arguments are fixed by push order |
| V2 | `FUN_10b61ee1` has exactly two callers | **confirmed** by grep over the whole 22 MB listing |
| V3 | `level' = BYTE [site+0x18] + level` | **confirmed**, and the `+ level` operand traced through three frames to the driver's `mov [ebp-0x20],esi` |
| V4 | the budget argument is `*budget / remaining_sites` | **confirmed**; the dividend is the same cell the charge writes back through |
| V5 | the divisor is the site collector's out-parameter | **confirmed in all four frames**, and both intermediate functions have exactly one caller, so the path is *the* path |
| V6 | `__forceinline` is charged nothing | **confirmed** — and there are **two** such skips, not one |
| V7 | stack 3/4 are one 64-bit quota that halves | **confirmed**; read, and deliberately not adopted |

**§6.6.2 is right.** The value of this section is not that it found an error; it
is that six published addresses and a four-frame data-flow trace are now
independently reproduced, and that the two things below fell out of doing it.

### 2.1 `BYTE [site+0x18]` is the LEVEL INCREMENT and it is `1` (**#3764**)

§6.6.2 publishes `level' = BYTE [site+0x18] + level` and leaves the field
unexplained — which means the level is **uninterpretable**: at delta 0 the depth
cap never binds, at delta 1 it is a 16-level nesting cap, and nothing in that
section decides between them. The field has exactly two writers:

```
10b602ce:  c6 40 18 01     mov    BYTE PTR [eax+0x18],0x1      <- every site, at birth
10b604fa:  88 42 18        mov    BYTE PTR [edx+0x18],al       <- the override
```

and the override is gated on `[callee+0x4c] & 0x10` (`0x10b604df`) — the bit the
driver **sets on the function it is expanding** at `0x10b61f56` and clears at
`0x10b620dc`. The byte it copies is a per-callee occurrence counter built in the
same scan (`0x10b603bc` seeds it at 1, `0x10b60398`–`0x10b603a6` increments it,
saturating at 255).

> **So the override is c2's handling of a callee already on the inline stack —
> recursion — and on any chain without it the level advances by exactly one per
> expansion.** `level` is a true nesting depth, and C14's `0x10` is a **16-level
> cap** on it. That reading is what `BUDGET_C2.site_level_delta = 1` is, and
> `BUDGET_FLAT_LEVEL` is the counterfactual: the model a reader of §6.6.2 alone
> would have had to guess at.

### 2.2 A correction to §6.6.2, offered as a patch block (**#3765**)

§6.6.2 finding 2 reads:

> *"`0x10b6240f` tests `[sym+0x4c] & 0x2000` and `jne` skips **both**
> `sub DWORD [ebx],eax` (`0x10b62418`) and `add ds:0x10c3f5cc,eax`
> (`0x10b6241a`)."*

True of the two stores it names. There is a **third** global write on that path
and the `jne` does **not** skip it:

```
10b6240a:  a3 d0 f5 c3 10   mov    ds:0x10c3f5d0,eax      <- BEFORE the test
10b6240f:  f7 41 4c 00 20 00 00  test DWORD PTR [ecx+0x4c],0x2000
```

`DAT_10c3f5d0` receives the nested pass's consumed budget unconditionally. A
reader taking *"charged nothing"* as *"leaves no trace in c2's global state"* is
wrong by exactly one datum. **This lane does not own `P_INLINE.md`**
(`w-inlswitch` does), so the correction is here as a quotable block rather than
in that file:

> **Amendment to §6.6.2, finding 2, offered by `w-inlbudget`.** Replace
> *"skips **both** …"* with: *"skips both `sub DWORD [ebx],eax` (`0x10b62418`)
> and `add ds:0x10c3f5cc,eax` (`0x10b6241a`) — but **not**
> `mov ds:0x10c3f5d0,eax` at `0x10b6240a`, which runs before the test. And the
> same exemption appears a second time, in the charge function, at
> `0x10b625a6`/`0x10b625b0`, where it skips the callee's own instruction-count
> charge: `jne 0x10b625c7` skips **both** `0x10b625bb` (local) and `0x10b625c1`
> (global), while C18's `jbe 0x10b625b9` skips **only** the local one. The two
> exemptions differ in EXTENT, not only in condition — which is the sharpest
> form of the orthogonality claim this section makes."*

### 2.3 Three more mid-instruction clause addresses, for `w-clausefix` (**#3766**)

§6.6.3 found eight of the 24 clause addresses mid-instruction and hand-verified
three. This lane needed C2, C3 and C14 for the model and re-derived them; the
other three below reproduce §6.6.3's independently. **No row is edited** —
`work/w-inlmetric/CLAUSES.tsv` is `w-clausefix`'s under decision 22.

| row | cited | what decodes there | the clause's real address |
|---|---|---|---|
| C2 | `0x10b626d8` | mid-instruction | **`0x10b626f7`** (`movzx eax,WORD [fn+0x50]`), store at `0x10b62703` |
| C3 | `0x10b626f4` | mid-instruction | **`0x10b62708`** (`add eax,eax`), clamp through `0x10b6271e` |
| C14 | `0x10b609ae` | inside `and eax,0x10` at `0x10b609ad` | **`0x10b60a0b`**–`0x10b60a1f` |
| C18 | `0x10b6249b` | mid-instruction | **`0x10b625b6`** — agrees with §6.6.3 |
| C19 | `0x10b624a2` | mid-instruction | **`0x10b625bb`** + **`0x10b625c1`** — agrees with §6.6.3 |
| C10 | `0x10b609d3` | `call 0x10b5e64d` — aligned, different instruction | unresolved; §6.6.3's finding reproduced |

## 3. `#3746`'s residue — five of thirteen closed, and the control refuted the lane (**#3763**)

Board `#3746` left the registry's completeness open: 13 `UNCOVERED` rows, four
named as *"a real refusal boundary … not enumerated yet"*, and the trap that
**a `guards` entry whose domain cannot reach its const is a false coverage
claim** — 2 of the original 7 were.

Three new surfaces close **all four** named boundaries, plus one more:

| surface | site | cells / refusals | consts closed |
|---|---|---:|---|
| `splice.budget` | `splice.rs` | 672 / 387 | *(four new consts, covered on arrival)* |
| `mangle.string_comdat` | `coff/mangle.rs` | 94 / 15 | `LITERAL_TEXT_BYTE_LIMIT` |
| `order.store_run` | `codegen/order.rs` | 120 / 48 | `HEAD_SLOTS_MAX`, `MAX_MULTISYM_PRODUCERS`, `MAX_SYMBOL_CROSSINGS` |
| `nonce.ds_form` | `codegen/nonce_add_run.rs` | 392 / 343 | `DS_MAX` |

`UNCOVERED` **13 → 8**, `UNCOVERED_RATCHET` lowered to match. A ratchet that
only ever rises is a ratchet nobody believes.

> ### The control refuted this lane's own writing, and that is the finding
>
> `HEAD_SLOTS_MAX` was first written **into** `UNCOVERED`, with an argument:
> `layout_slots` reads `u` only through `i.min(u)`, and a run with enough
> producers to see `u = 3` is already past `MAX_MODELLED_PRODUCERS`, so widening
> it cannot matter. The control disagreed — **47 domain lines move**, through
> `leading_unproduced` (rendered in its own right) and through `store_order`'s
> `for u in (0..=head_slots).rev()` search. It is a claimed guard now.
>
> `#3746`'s rule is *measure the coverage claim, never argue it*. It turns out
> to catch a wrong **non**-coverage claim exactly as readily, which is the
> direction nobody was watching: an `UNCOVERED` row with a plausible reason is
> **cheaper to write and harder to doubt** than a `guards` entry, and nothing
> before this graded one.

### 3.1 What did NOT close, and why — 8 rows

| row | why it stayed |
|---|---|
| `MAX_C2_OPCODE`, `MAX_FIELDS` | both in `codegen/mop.rs`, which lane **`w-encarms` owns this wave**. Touching it would have been a seam violation; they are the first two rows for whoever owns that file next |
| `MAX_OBJECTS_PER_SECTION` | a real `[F]` layout cap in `coff/data.rs`. Reachable — `mangle.rs`'s own doc names it as the fitted contrast to `LITERAL_TEXT_BYTE_LIMIT`'s pinned `[O]` — and simply not reached before the lane's budget ran out. **The honest next row** |
| `K_ASCII_MAX`, `K_TWO_MAX` | genuine UTF-8 encoding-length brackets, and `#3746`'s *"a value, not a decision"* is wrong about them — the reason is rewritten. A domain over them needs the class's parser, not the consts, because the consumer takes the whole run or nothing |
| `POOL_TOP` | unchanged: a derived alias with no production use, already measured at 0 lines by `w-doctrine` |
| `R_BOUND`, `TOP` | the name screen's own two false positives, unchanged |

**The screen is still a NAME screen**, so a boundary spelled without one of its
nine words is missed entirely and nothing here reaches it. What the ratchet buys
is that the hole cannot grow quietly. That was true before this lane and is
still the whole claim.

## 4. Estimate vs outcome

| # | predicted, before the image was opened | realized |
|---|---|---|
| **P1** | all seven of V1–V7 confirm | **held** — seven of seven |
| **P2** | byte delta zero; identity diff 0 lines over 21 rows | **held** — see §5 |
| **P3** | `DOMAIN.txt` grows by **≥ 200** lines | **held, and by 6.5×** — +1,306 (1102 → 2408) |
| **P4** | **≥ 4** of the 13 `UNCOVERED` rows close, ratchet ≤ 9 | **held** — 5 closed, ratchet **8** |
| **P5** | no new suite failure attributable to this lane | **held** — see §5 |

**The prereg's own hedge was wrong, and it is worth saying which way.** §3 said
*"P1 is the one I most expect to be wrong in part"* and named a refutation of a
V-row as the better outcome. P1 held in full; the refutation arrived from
somewhere the prereg did not look — §2.1's unexplained field, §2.2's third
global write, and §3's own `HEAD_SLOTS_MAX` claim. **Two of the three things
this lane refuted are its own.**

## 5. Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **see §5.2** |
| `scripts/gate.sh --jobs 16 --require-graded` | **see §5.1** |
| `scripts/gate_identity_diff.sh base tip` | **see §5.1** |
| fixtures, `c2rs census` | none claimed — construct rung, `Census: +0` |

Transcripts: `work/w-inlbudget/gate_base.out` (taken on the clean tree at
`4b79bf46a` **before** any `crates/` edit), `work/w-inlbudget/gate_tip.out`,
`work/w-inlbudget/cargo_test_tip.out`.

### 5.1 The gate and the identity diff

```
GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus
  lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
  graded: 7038 fixture-verdicts across all lanes
  sweep:  PASS — 19556 of 19556 reached, 19460 GRADED, 0 mismatch
  cross:  PASS — 90424 of 90812 cells graded, 0 mismatch
  debug:  PASS — 18 of 18 lanes, 7038 fixture-verdicts, match 2479, 0 mismatch, 0 PANIC
```

**`HATCH-RED REFUSED` is INHERITED and is in the BASE run too.** `hatch.py apply`
cannot hatch this tree, so the arms have no tree to run on (`#1389`). The base
gate at `4b79bf46a`, on a clean tree before any edit, prints the identical
qualified headline.

```
$ scripts/gate_identity_diff.sh work/w-inlbudget/gate_base.out work/w-inlbudget/gate_tip.out
count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS
exit=0
```

**All 21 rows are digit-for-digit identical**, base to tip: the six `/O1`
families at 186/187/188/189/188/189, the six `Ox` at 157, the two `O2` at 163,
the four `Od` at 21, `expr-sweep` 19460, `mode-cross` 90424, `debug-lane` 2479.
The instrument's own `--self-test` was run at this tip and passes: enumeration
21, control silent, `#3515`'s one-TU-refused signature found at exactly 14 lines
/ 7 rows and nonzero, truncation refused (exit 2).

**`gate.sh` still carries 21 count-bearing rows — this lane added none**
(`#3691`), which is what keeps the identity diff usable for every other live
lane this wave.

### 5.2 The workspace suite

```
C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast
  60 targets, 1989 passed, 2 failed   (first run, at 8b4ca972c)
```

**Both failures are this lane's own DOC BOOKKEEPING, both are the class a
zero-emit change actually breaks, and both are fixed** — `w-inlfit` recorded the
same pair one wave ago and it is worth two lines rather than a footnote:

| target | why | fix |
|---|---|---|
| `rung_registry::rung_index_is_generated_and_current` | this rung doc landed without `docs/rungs/INDEX.md` being regenerated | `scripts/gen_rung_index.sh` — the index is **generated**, and running the generator is not the hand-edit the seam table forbids |
| `provenance::prose_audit_tree_run_finds_no_false_count_binding` | `DISCLOSURE.md`'s own `COUNT[ledger-rows] = 24` and the sentence beside it, against a table that `W-INLBUDGET-1` made **25** rows long | both updated to 25 |

Re-run on the fixed tree: **`provenance` 4 passed / 0 failed, `rung_registry`
4 passed / 0 failed.** No other target moved, and nothing in the 1,989 is a
`crates/` regression: the port's own suite is `c2-core` 680 passed / 0 failed at
this tip, with 31 of them in `splice::tests` including the nine this lane added.

> **The doc-sensitive targets are the ones a zero-`crates/`-byte change can
> actually break**, which is the opposite of where attention goes on an adoption
> lane. Running them *first* next time is a cheaper order than running them last.

### 5.3 The controls, watched RED before any verdict was quoted (`#3336`)

`work/w-inlbudget/controls_red.txt`, reproducible with
`sh work/w-inlbudget/controls.sh`:

```
C1 — the model's own default, flipped:
  BUDGET_C2.divide := false: RED — 316 domain line(s) moved
  BUDGET_C2.site_level_delta := 2: RED — 166 domain line(s) moved
C2 — every const this lane claims as a surface guard:
  INLINE_BUDGET_FLOOR 1000->1001: RED — 21 domain line(s) moved
  INLINE_BUDGET_CEILING 35000->35001: RED — 5 domain line(s) moved
  INLINE_LEVEL_DEPTH_CAP 16->17: RED — 40 domain line(s) moved
  INLINE_CHARGE_EXEMPT_MAX 40->41: RED — 6 domain line(s) moved
  LITERAL_TEXT_BYTE_LIMIT 32->33: RED — 32 domain line(s) moved
  MAX_MULTISYM_PRODUCERS 2->3: RED — 17 domain line(s) moved
  MAX_SYMBOL_CROSSINGS 2->3: RED — 5 domain line(s) moved
  HEAD_SLOTS_MAX 2->3: RED — 47 domain line(s) moved
  DS_MAX 0x7FF8->0x7FFC: RED — 16 domain line(s) moved
CONTROL: the unmutated tree must be GREEN
  restored tree: GREEN
```

**C5** — the `Fail axis:` enforcement in `crates/c2-harness/tests/rung_registry.rs`
is new on 2026-08-28 and this rung is the first record it grades, so it was
watched failing on this file before its green was quoted. See §7.

> ### 5.4 The control script had a defect that made it lie in the safe direction (**#3767**)
>
> The first run reported **`RESTORED TREE IS RED`** on a tree whose sources were
> byte-identical to the committed ones. The cause is not in the surfaces:
> **`cp` then `mv` preserves the backup's mtime**, which is older than the
> artifact cargo had just built *from the mutation*, so the closing green check
> linked the **mutated** test binary.
>
> The per-mutation readings are unaffected — the `perl -0pi` rewrite always
> post-dates the artifact, so every RED above is a fresh build of its own
> mutation — but a control that cannot see itself finish is one line from a
> control that cannot see itself *start*. Fixed with a `touch` after the
> restore, and the reason is a comment in the script rather than a fact somebody
> has to rediscover. **This is the generic hazard for every mutation control in
> this repo that restores by moving a backup into place**, and there are
> several.

## 6. Found and not taken

Ranked, and each with what it would cost.

1. **`MAX_OBJECTS_PER_SECTION`** — the only `#3746` row this lane could have
   closed and did not. `coff/data.rs`, a real `[F]` layout cap, no seam
   conflict. Perhaps an hour.
2. **The 64-bit quota (V7).** Read at `0x10b6204e` and adopted nowhere: the
   `0x10b5beac` helper's arithmetic on three 64-bit values is unread, and the
   halving is gated on `[callee+0x4c] & 0x10`, a bit the port cannot see. It is
   a **second budget on a different schedule** and nothing in this project knows
   what it meters. `FUN_10b5beac` is the read.
3. **`DAT_10c3f5d0`.** §2.2's third global write. One writer found here; its
   readers are unenumerated, and *"who reads the consumed-budget global"* is a
   ten-minute grep that would say whether it is telemetry or a decision input.
4. **The `[sym+0x4c] & 0x10` bit itself.** It appears in *four* places this lane
   touched — the driver sets and clears it (`0x10b61f56`/`0x10b620dc`), the
   quota's halving is gated on it (`0x10b6203a`), and the level override reads
   it (`0x10b604df`). Reading it as *"on the inline stack"* is this lane's
   inference from those four sites and is marked `[I]`, not `[R]`. Naming it
   properly would firm up §2.1.
5. **The charge's other consumer.** `DAT_10c3f5cc` is seeded at the pass entry
   with the caller's instruction count and added to by both charge sites; C16's
   *"caller-huge decline: `35000 < DAT_10c3f5cc`"* is the reader. The port has
   the model's `charge()` and no way to call it, because it has no instruction
   count. That gap is C2/C24's, not this lane's, and it is the same gap §6.6.1
   names.
6. **`splice.budget`'s exercised fraction.** The surface renders 672 cells; the
   production path reaches exactly **one shape** of them (`n = 1`, `level` small,
   `base = 0`). Publishing that ratio the way `regalloc`'s
   `how_much_of_the_parameter_is_exercised` does would put a denominator on the
   adoption instead of leaving it to be inferred.

## 7. The `Fail axis:` check, watched failing (`#3336`, board `#3744`)

`construct_rungs_from_the_cutoff_name_an_axis_they_can_fail_on` is new and this
is the first rung dated at or after its `2026-08-28` cutoff, so a green from it
is otherwise a green from an empty population.

```
$ # C5a — delete the Fail axis: field from this file
$ cargo test -p c2-harness --release --test rung_registry
fail-axis: 336 rung docs examined, 8 declare a construct rung,
           1 dated at or after 2026-08-28 (the graded population), 1 violation(s)
2026-08-28-w-inlbudget.md declares `Kind: construct rung` and carries no `Fail axis:` header field
test result: FAILED. 3 passed; 1 failed

$ # C5b — restore
test result: ok. 4 passed; 0 failed
```

**The graded population is 1 and it is this file.** `#3744` shipped the check
saying in its own words that it *"grades ZERO docs today, which is why it
carries its own control"*; the tree run now has a real member, and it goes red
on it. Eight rung docs declare `Kind: construct rung` and seven are before the
cutoff, which is the grandfathering working as written.

**What it still cannot do, restated because it is easy to forget once it is
green:** the field is checked for **presence, not measurement**. Nothing in this
repo can tell a named axis from a measured one — §5.3's controls are what make
this lane's three axes measured, and they are not what `rung_registry` reads.
