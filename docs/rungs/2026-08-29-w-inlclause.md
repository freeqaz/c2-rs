# w-inlclause — the `absent` column had two states, and thirteen of fifteen are the one nobody was fixing

    Tag:       w-inlclause
    Slug:      w-inlclause
    Date:      2026-08-29
    Kind:      construct rung (adoption + instrument)
    Outcome:   built
    Fixtures:  none — construct rung: it adds a `read` axis to the inliner's
               conformance table, adopts C15 into `splice.rs` under a
               required-zero byte delta, and re-expresses the already-adopted
               depth predicate through c2's full arm set
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Fail axis: FOUR, and the byte delta is again not one of them — it cannot
               fail here by construction (see §3), so it is the floor and not
               the grade. (1) THE CITATION — a `read` cell is a claim about a
               document, so `read_state.py`'s CITE check resolves every anchor
               INSIDE the file it names; it went RED on two of this lane's own
               rows on its first run. (2) THE PRECEDENCE — `0x10b60a28`'s
               bypass lands BETWEEN c2's two depth arms, so a model that
               flattens them is wrong in a way no byte can see; the arm order
               is asserted directly. (3) CONSTANT REACHABILITY — `BUDGET_C2`'s
               `max_level` IS `INLINE_MAXLEVEL_UNBOUNDED`, so a domain over the
               named models moves ZERO lines when the constant is mutated and
               the coverage claim would be false (`#3746`); the sweep is graded
               by mutating the constant and requiring the domain to move.
               (4) THE `R3` POPULATION — the EVIDENCE check grades zero rows
               today and must SAY so, never print a green over an empty
               population (`#3470`).
    Record:    this file; prereg `work/w-inlclause/PREREG.md`, committed at
               `2e4040dae` BEFORE the image was opened; the read at
               `work/w-inlclause/IMAGE_READ.md`; the scan at
               `work/w-inlclause/read_scan.out`; controls at
               `work/w-inlclause/controls_red.txt`

Charter: `docs/ADOPTION_BRIEF_2026-08-29.md` §L2. Dispatched at master
`12d3c0558`. Board **#3796**–**#3801**.

> **The brief asked which of the 15 `absent` clauses have an existing read
> behind them and which are absent because nothing has been read. The answer is
> 13 to 2** — and the 13 are not stuck for want of reading. **Four of them are
> stuck on one missing link**, the port having no pre-codegen instruction count.
> Two were not stuck at all: they had counterparts and the table could not see
> them.
>
> Split **`absent 15 · R-derived 4`** → **`absent 12 · R-derived 7`**. Byte
> delta **zero**, identity diff **0 lines over 21 rows**, `gate.sh` still **21**
> count-bearing rows (`#3691`), **128 not adopted** (`#3732`), `surface.rs` and
> `P_INLINE.md` **untouched**.

---

## 1. The question, answered mechanically

`work/w-inlclause/read_scan.py`, on the dispatch tree:

```
PIN-SCAN: 13 of 15 rows have at least one clause-pinning address cited in the
          frozen corpus as dispatched
PIN-SCAN: 15 of 15 rows once this lane's own read is included
```

Two things make that number mean something.

**It scans for the address that PINS the clause, not the address the row
cites.** For **eight of the fifteen** those differ: five rows cite a function
**entry** (C1, C5, C6, C20, C21 — `asm` cell `push ebp`) and three cite a
**block head** (C11, C12, C13 — `mov eax,[ecx+0x20]`, which is none of their
six masks). Both classes are named by `w-clausefix` itself. Scanning the cited
address would have counted a read wherever anyone had typed the function's
entry.

**A raw `.asm` dump is not corpus.** `work/w-inlbudget/FUN_10b600e6.asm`
contains every address in C5 and C6 and tells a reader nothing about which of
them matter; counting it would make every clause in a dumped function free of
`R3` for nothing. Only `.md` prose is scanned, and the module doc says so.

### 1.1 The new columns, and what they are for

`work/w-inlmetric/CLAUSES.tsv` gains `read` / `readcite` / `blocker`, graded by
`work/w-inlclause/read_state.py`. `state` says whether the **port** has a
counterpart; `read` says whether **this project** has read the clause well
enough to build one, and the two are orthogonal.

| | meaning | rows |
|---|---|---:|
| **`R1`** | read, and a port counterpart is derivable today with every field carrying a `PROV[R]` address | **15** |
| **`R2`** | read, and a **named** link is missing | **9** |
| **`R3`** | unread | **0** |

**The blockers are the finding, not the trichotomy.** Over the 24:

| blocker | rows | what it is |
|---|---:|---|
| `none` | 7 | the `R-derived` set |
| `emit-change` | 5 | C7, C9, C10, C11, C12 — derivable, and adopting changes emitted bytes. **Named and stopped**, per the brief |
| **`no-instr-count`** | **5** | **C2, C4, C16, C17, C20 — one missing link, five clauses** |
| `no-instr-stream` | 2 | C5, C6 — the port has no c2 instruction/opcode stream |
| `unit-gap` | 1 | C8, and `P_INLINE` §6.6.1 already says it in those words |
| `n-a` | 3 | the `unexercisable` rows |

> **`no-instr-count` is the highest-leverage row on the page.** c2's
> `WORD [sym+0x50]` is a pre-codegen instruction count; §2.1b measured it as an
> *upper bound* on the tested quantity and not the quantity (`arith_012` and
> `mix_008`, identical `SIZE` 115, opposite verdicts), and §6.7 refuted the
> reduction that would have explained the gap. Until something closes it,
> **five clauses cannot be derived no matter how much more is read**, and
> "read C16" is wasted money. That is the sentence the `read` column exists to
> make sayable.

## 2. Two of the three conversions are STALENESS (**#3797**)

C14 and C18 moved to `R-derived` with **zero new `crates/` logic**, because the
port had carried counterparts since `w-inlbudget` landed the day before:

| row | the port's counterpart, on the dispatch tree | why the row still said `absent` |
|---|---|---|
| **C14** | `INLINE_LEVEL_DEPTH_CAP = 16` and `BudgetModel::declines_at_depth`, `PROV[R]` at **`0x10b60a1c`** — C14's own repaired address. `w-inlbudget`'s rung: *"`S6-budget-depth-cap` (C14, `level − base > 16`)"* | the row cited a **different spelling** |
| **C18** | `BudgetModel::charge`'s `callee_instrs > charge_exempt_at_or_below`, and `INLINE_CHARGE_EXEMPT_MAX`'s `PROV[R]` cites **`0x10b625b6`** — C18's own `cmp eax,0x28` | the row cited a **different spelling** |

**The ABSENCE screen is a NAME screen in the direction nobody had declared.**
`#3641` and `token_in_crates`'s own docstring declare the false-**positive**
half — a mention read as a counterpart — and `#3788` measured and repaired it
the same morning. The other half is a counterpart adopted under a name the
table does not cite, read as absence. It is **silent**, it is counted by
nothing, and it is why the column looked stuck.

> **C3 and C19 converted at the wave-18 merge only because the adopting lane
> happened to choose tokens that collided with the table's.** That is a
> coincidence of spelling, not a mechanism — and on the same tree two more rows
> were already converted and unmarked, while three rungs quoted the split.

`crates/c2-harness/tests/clause_table.rs`'s `SPLIT` constant carries the
reasoning inline, which is what that constant is for (`#3748`).

## 3. C15 adopted — and c2 has a third depth arm nobody had named (**#3798**, **#3799**)

Reading eight instructions past a row that was GREEN on all five checks:

```
10b60a0b:  mov eax,ds:0x10c3f50c     <- the base
10b60a10:  mov edx,DWORD PTR [ebp+0xc]   <- maxlevel, the 2nd stack parameter
10b60a13:  cmp eax,ebx / je 0x10b60a25   <- base == 0: skip BOTH arms
10b60a1a:  sub ecx,eax               <- level - base
10b60a1c:  cmp ecx,0x10 / jg         <- C14
10b60a21:  cmp ecx,edx  / jg         <- **NOT ANY OF THE 24 CLAUSES**
```

`0x10b60a21` declines when `level − base > maxlevel`. It is **upstream of the
`__forceinline` bypass** (`0x10b60a28`) and **not guarded by the `!= 0xff`
sentinel** (`0x10b60a2f`), so it is neither C14 nor C15. `declines_at_depth`
modelled only the first arm and therefore **admitted where c2 declines**.

**Novelty checked rather than assumed** — `0x10b60a21` appears in the frozen
corpus only as an unannotated listing line inside `REPAIRS.md`'s C14 context
window.

`splice.rs` now carries both arms plus C15's absolute one, with c2's precedence
between them: `declines_at_maxlevel` takes `forceinline` **as a parameter**,
because the bypass lands *between* the two, and `port_enter_site` passes
`false` — the port cannot read `[sym+0x4c] & 0x2000`, so it takes the test
rather than assuming the bypass. Flattening that into "forceinline bypasses
everything" is wrong in a way no byte delta can see.

### 3.1 Byte-neutral by construction, twice, and neither reason is a measurement

| | why the clause cannot fire on the port's admitted set |
|---|---|
| the **absolute** arm | `max_level` defaults to c2's own sentinel `INLINE_MAXLEVEL_UNBOUNDED = 255`; the `cmp edx,0xff` / `je` guard makes it identically false. `#pragma inline_depth` is the only thing that moves it, and it is in **0 of the 100** hold-out TUs |
| the **relative** arm | it needs `level_base != 0`, and `Expansion::at_pass_entry` seeds the base to 0 (`mov ds:0x10c3f50c,ebp` at `0x10b6274c`) and `enter_site` preserves it, so the whole branch is unreachable from any admitted chain |

The prereg (§4.2) required exactly this shape: *"a byte-neutrality that has to
be measured rather than argued is not admissible here, because `#3723` proved
the byte judge is blind to this class."*

### 3.2 `#3746`'s trap has a second shape: a const that is its own comparand

`declines_at_maxlevel` compares `self.max_level` against
`INLINE_MAXLEVEL_UNBOUNDED` — **and `BUDGET_C2.max_level` IS that constant.**
Mutating the constant moves both sides of the guard together, so a domain
rendered over the four named models moves **zero lines** and the coverage claim
would be false. `#3746`'s rule is *measure the coverage claim, never argue it*,
and this is a shape it had not met: not an unreachable const, a **self-tracking**
one.

Closed by a **sweep** over `max_level` values the constant does not follow
(`0, 2, 254, INLINE_MAXLEVEL_UNBOUNDED, 256`), so the sentinel is separated
from its neighbours by rows that move when it moves. `splice.rs`'s
`the_maxlevel_sentinel_is_reachable_in_the_domain_and_not_merely_named` asserts
the separation; §6's control measures it.

**No `guards` entry was added**, because `crates/c2-core/src/surface.rs` is
`w-fmadd`'s this wave. `INLINE_MAXLEVEL_UNBOUNDED` is not caught by the
boundary-name screen either (its words are `INLINE`/`MAXLEVEL`/`UNBOUNDED`,
none of the nine), so nothing forced the question. **Offered as a paste-ready
block** for whoever owns that file next — the domain already reaches it, so
this is a naming of coverage that exists, not a claim of coverage that does not:

```rust
// crates/c2-core/src/surface.rs, the `splice.budget` Surface, `guards`:
        guards: &[
            "INLINE_BUDGET_FLOOR",
            "INLINE_BUDGET_CEILING",
            "INLINE_LEVEL_DEPTH_CAP",
            "INLINE_CHARGE_EXEMPT_MAX",
            "INLINE_MAXLEVEL_UNBOUNDED",   // w-inlclause, DISCLOSURE W-INLCLAUSE-1
        ],
```

## 4. Two more reads, and one correction offered rather than made

C5 and C6 were the only two `R3` rows. Both are closed
(`work/w-inlclause/IMAGE_READ.md` §3–§4), and one of them corrects the corpus.

* **C5** — `cmp al,0xf` at **`0x10b6020b`**, on `BYTE [instr+0x8]`. The clause
  is correct and now has an address.
* **C6** — the opcode dispatch is a **19-entry dense switch** over
  `0x2ee..0x300` (`lea ecx,[eax-0x2ee]` / `cmp ecx,0x12` at `0x10b603ef`),
  through a byte index table at `0x10b60522` and a jump table at `0x10b6050e`,
  into **five arms** maintaining **three** independent nesting counters.
  Decoded from the image by `work/w-inlclause/jumptable.py` (std only).

> **`WB_INLINE_FINDINGS` §1 lists seven opcodes. There are eight with a
> non-default arm — `0x2fe` is missing**, and it shares arm 0 (`inc [ebp-0x8]`)
> with `0x2ee`, so a reader working from the list has one of the two
> region-openers and does not know it. `0x2fe` appears **nowhere** in the frozen
> corpus.
>
> **Amendment offered to `WB_INLINE_FINDINGS.md` §1**, as a quotable block
> rather than an edit (this lane owns neither that file nor `P_INLINE.md`):
> replace *"Tracks EH-region nesting through opcodes
> `0x2ee/0x2f0/0x2f1/0x2f4/0x2f6/0x2ff/0x300`"* with: *"Dispatches opcodes
> `0x2ee`–`0x300` through a 19-entry dense switch (`0x10b603ef`, tables at
> `0x10b60522`/`0x10b6050e`) into five arms over **three** nesting counters
> (`[ebp-0x8]`, `[ebp-0xc]`, `[ebp-0x10]`); eight opcodes have a non-default
> arm — `0x2ee 0x2f0 0x2f1 0x2f4 0x2f6 0x2fe 0x2ff 0x300` — and `0x2f0`/`0x2f1`
> are secondary tests inside two arms rather than top-level cases."*

Also read and offered, not adopted: `[site+0x1c]` is a **five-bit flag word**,
and bit 1 carries a `__forceinline` term at **`0x10b60317`** — a **third** site
testing `[sym+0x4c] & 0x2000` in this subsystem, after C10's `0x10b60a28` and
the charge's `0x10b625a6`/`0x10b6240f`. C10's clause names one of at least four.

## 5. What was NOT taken, each with the reason

| row | `read` | why it stopped here |
|---|---|---|
| **C7** | `R1` | fully read end to end (§6.6.1: one reader, two writers, `k = 3`, the `-vol#` switch). Adopting it **is** adopting 128, which `#3732` forbids on 8 counterexamples in each direction, and §6.6.1 says the unit is wrong and the converter is two subsystems away. `emit-change` |
| **C9** | `R1` | the favour-speed bit is read, writer and all (`0x10b8238d`). Adopting *"at `/O2` the size test is skipped"* changes emitted bytes — GRID-I moved it at `/O2` on 60 cells. `emit-change` |
| **C10** | `R1` | an **accept** clause, and the port has no accept path anywhere (§6.2 item 5). Any adoption is an emit change |
| **C11, C12** | `R1` | refuse clauses on bits the workload never isolates. Adopting turns emits into refusals, and a wrong refusal scores below the emit it replaced. Byte-neutrality here could only be *measured*, which §4.2 of the prereg forbids |
| **C1** | `R2` | the gate is read (`cmp ds:0x10c40ec4,ebp` at `0x10b6267b`); **nothing enumerates its writers**, so no port counterpart can know when the pass is off. `writer-unread`, and it is a ten-minute grep for the next lane |
| **C4** | `R2` | §6.6.2 fixes all six arguments and `Expansion::at_pass_entry` already carries two of them — but the third is `B`, and `B` needs the instruction count. **My own prereg named C4 as a conversion candidate and it is refuted**; ties break toward the weaker state |

## 6. Controls, watched RED before any verdict was quoted (`#3336`)

`work/w-inlclause/controls_red.txt`, reproducible with
`sh work/w-inlclause/controls.sh`:

```
=== read_state.py: one plant per rule ===============================
  C7=read=R3                        RED (3 FAIL) -- GRAMMAR: R3 with a live readcite
  C7=readcite=nope.md#0x1           RED (1 FAIL) -- CITE: the path does not exist
  C7=readcite=…P_INLINE.md#0xdeadbeef RED (1 FAIL) -- CITE: path exists, anchor does not
  C14=read=R2                       RED (1 FAIL) -- GRAMMAR: R-derived may not be R2
  C7=blocker=because                RED (1 FAIL) -- VALUE: a free-text blocker
  C7=blocker=none                   RED (1 FAIL) -- GRAMMAR: derivable, unblocked, unadopted
  C21=blocker=none                  RED (1 FAIL) -- GRAMMAR: unexercisable must be n-a
  CONTROL: unplanted -> READ-STATE: GREEN (0 failures over 24 rows)

=== check_table.py: w-inlmetric's own control, re-watched on this tip
  C16(PLANTED): ADDRESS 0x10b5c06b is in FUN_10b5c06b, table claims FUN_10b60930
  C16(PLANTED): DECODE  … -> CONFORMANCE-CHECK: RED (2 failures over 24 rows)
  unplanted -> CONFORMANCE-CHECK: GREEN (0 failures over 24 rows)

=== the adopted clause: mutate its constant, the domain must move ===
  INLINE_MAXLEVEL_UNBOUNDED: 255 -> 256   RED -- 10 domain line(s) moved
  BUDGET_MAXLEVEL_2 max_level: 2 -> 3     RED -- 13 domain line(s) moved
  CONTROL: the restored tree must be GREEN -> ok, byte-identical to the backup
```

Two things about the control script itself, both of which cost a run:

* **`cp`/`mv` preserves the backup's older mtime** (`#3767`), so the restore is
  followed by `touch` and the restored tree is re-verified. Inherited from
  `w-inlbudget` §5.4 rather than rediscovered.
* **The restore check must NOT be `git diff`.** This lane edits `splice.rs`, so
  a diff against `HEAD` is non-empty *by design* and the check could not fail.
  It compares against the pre-mutation **backup**. The first run printed
  `RESTORE FAILED` on a correctly restored tree, which is the same class one
  level up: a control that cannot fail, and a control that fails on everything,
  are both controls that say nothing.

> ### 6.1 The CITE check caught two of my own citations (**#3800**)
>
> On its first run `read_state.py` went RED on **C6** and **C14** — both
> `readcite` cells named a document that did **not** contain the address they
> claimed, because the prose there uses the bare-hex form. A `read` cell is a
> claim about a document, so it is checkable against the document, and this is
> `#3470`'s shape applied to a citation instead of a count. Fixed by
> re-pointing C14 at `REPAIRS.md` and by writing the missing prose for C6.

## 7. Estimate vs outcome

| # | predicted, before the classification | realized |
|---|---|---|
| **P1** | 2–4 rows have a counterpart today under a name the table does not cite | **held, at the bottom of the band** — 2 (C14, C18). The third prior, **C4, is refuted**: `at_pass_entry` carries two of six arguments and the third needs `B` |
| **P2** | **≥ 5** rows place `R3` | **REFUTED — 2 as dispatched, 0 at this tip.** The prereg named P2 as the one it most expected to lose, and gave this reason: *"every one of these 24 rows sits inside a function some lane has listed"* |
| **P3** | ≥ 3 `R2` rows share **one** blocker | **held, and by more** — 5 share `no-instr-count` |
| **P4** | 0 clauses adopted that change a byte; identity diff 0 over 21 rows; still 21 gate rows | **held** — see §8 |
| **P5** | the split moves only `absent → R-derived`, by 2–4 cells | **held** — 3 cells, no other transition, `unexercisable` unchanged at 3 |
| **P6** | at least one thing in the prereg, `P_INLINE` §6.1 or the brief is refuted; specifically a DECODE-green row whose address does not pin its clause | **held on the outcome and WRONG on the mechanism** — see §9 |

## 8. Gate evidence

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 16 --require-graded` (base, `12d3c0558`, before any `crates/` edit) | **`GATE: PASS`, unqualified** — `work/w-inlclause/gate_base.out` |
| `scripts/gate.sh --jobs 16 --require-graded` (tip) | **`GATE: PASS`, unqualified** — 18/18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, **7038** fixture-verdicts; sweep 19460 of 19556 graded, 0 mismatch; cross 90424 of 90812, 0 mismatch; debug 7038 verdicts, match 2479, 0 PANIC |
| `scripts/gate_identity_diff.sh base tip` | **`IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS`**, `21 base, 21 tip` |
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **62 targets, 1995 passed, 1 failed** — the one failure is a peer lane's, §8.1 |
| fixtures, `c2rs census` | none claimed — construct rung, `Census: +0` |

**`GATE: PASS` is unqualified at both ends.** `#3786` re-anchored the
`hatch-red` needle on 2026-08-29 and this is a lane confirming it from a fresh
worktree: neither run carries `(HATCH-RED REFUSED)`.

**All 21 count-bearing rows are digit-for-digit identical**, base to tip. This
lane added **no `gate.sh` row**, which is what keeps the diff usable for every
other live lane in the wave (`#3691`).

### 8.1 The one suite failure is another worktree's, and it is named

```
---- no_worktree_holds_an_unlocked_pinned_artifact ----
  reap guard FAILED — a worktree holds a pinned artifact and is NOT locked.
  UNLOCKED AND PINNED  <mainrepo>/.claude/worktrees/w-fmadd
    P1 unique-binary  work/w-fmadd/sweep_fp/c2rs
    P1 unique-binary  work/w-fmadd/sweep_fp2/c2rs
```

`crates/c2-harness/tests/wt_pin_audit.rs` scans **every** worktree of the
primary repo, so it reports the same failure from any tree in the checkout —
confirmed by running `scripts/wt_pin_audit.sh` directly, which names `w-fmadd`
and nothing else. **`w-inlclause`'s worktree appears in no violation line.**
The remedy (`scripts/wt_pin_audit.sh --lock`) belongs to that lane; locking a
peer's live worktree from here would be the seam violation, not the fix.

Recorded rather than filtered: `w-inlbudget`'s own rung makes the point that a
lane which shows only its clean re-run has hidden the failures that were real,
and the honest version of that is naming the one failure and whose it is. The
transcript is committed with the failure in it (`cargo_test_tip.out`).

> **`crates/c2-core/src/surface/DOMAIN.txt` is a generated baseline and this
> lane re-blessed it** (2540 → 2853 lines). `w-fmadd` will re-bless the same
> file this wave for `mop.rs`'s rows, so **the merge collides in generated data
> and is resolved by re-running the blessing**, not by picking a side:
> `cargo test -p c2-core --lib surface::tests::bless -- --ignored`. Flagged
> here because a conflict in a generated file is the kind a merge resolves
> wrongly and quietly. `surface.rs` itself is untouched.

## 9. What this lane refuted, including two of its own claims (**#3801**)

**Of the brief.** It says C3 and C19 converted *"because `P_INLINE` §6.6.2 had
been read to address level and the port could be derived from it"*, and invites
the same question of the other 15 as though reading were the scarce input. It
is not: 13 of 15 were already read, and the scarce input is a **quantity the
port can compute**. The brief's framing would have sent the next four lanes to
read C16, C17, C2 and C4, which are already read and blocked on the same thing.

**Of `P_INLINE` §6.1.** P6 predicted a DECODE-green row whose address does not
pin its clause, and blamed `asm` having been transcribed *from* the address.
**The outcome held and the mechanism was wrong.** There are eight such rows —
five entry citations and three block heads — and they are not an oversight:
`w-clausefix` found them, documented every one in
`work/w-clausefix/REPAIRS.md`, and **deliberately left them**, because a clause
that spans four instructions has no single address to cite. The table header
already says `asm` *"DOES NOT verify the clause"*. What §6.1 does not carry is
that the reachable denominator **21 of 24** is 21 of the *rows* — it is not a
statement about c2, and `0x10b60a21` is an arm of the decision function that
none of the 24 covers.

**Of this lane's own writing, twice.** I read C10's bypass semantics and
C11/C12/C13's block-head citations out of the image and **drafted both as
findings** before checking the corpus. Both are in `REPAIRS.md`, at the same
addresses. `IMAGE_READ.md` §0 says so before it says anything else, and the
three `addr` cells were **not** touched. The seam that gives one lane a file at
a time also buries the prior art in a *work directory* the artifact does not
link — which the new `readcite` column incidentally fixes: **thirteen of the 24
rows now cite `REPAIRS.md` by address**, so the next reader finds it from the
table.

## 10. Found and not taken

1. **`DAT_10c40ec4`'s writers** — C1's blocker in one grep. *Which switch turns
   the inline pass off?* Nothing in the corpus says, and the row cannot leave
   `R2` until something does.
2. **The `no-instr-count` link itself** — five clauses, one gap, and §6.7
   already refuted the reduction that would have explained it. The remaining
   question is whether c2's `WORD [sym+0x50]` at *decision* time is derivable
   from the IL at all; if it is not, five rows are permanently `R2` and saying
   so is worth more than five more reads.
3. **`[site+0x1c]`'s bits 2 and 4** — read as `[ebp-0x30] != 0` and an incoming
   `edi & 1`, and neither source is traced. §3.2 of the read.
4. **The `0x2080` mask at `0x10b5fcc1`** — the candidacy function's near-miss on
   C10's `0x2000`. `0x80` is unidentified and it sits beside `__forceinline` in
   the one test that decides candidacy on a non-POGO build.
5. **A 25th clause row for `0x10b60a21`.** This lane's prereg froze the clause
   list at 24 and adding one after seeing the answer is fitting the instrument
   to its own result. The arm is in `splice.rs`, in `DISCLOSURE W-INLCLAUSE-1`
   and in `IMAGE_READ.md` §2; **the row is the next owner's**, under their own
   prereg, and the ratchet question they inherit is *how many other arms are
   there* — nothing on this page has ever asked.
