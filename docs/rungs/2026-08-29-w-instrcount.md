# w-instrcount — the count is the front end's, c2 never touches it, and F7 measured a dead zone

    Tag:       w-instrcount
    Slug:      w-instrcount
    Date:      2026-08-29
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization: what `[fn+0x50]` IS — when it is
               written, by what, in what unit, and whether anything in the port
               could carry it
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Reach:     PREDICTED 0, REALIZED 0 — registered in prereg §1 before the
               image was opened, and this lane changes no compiled file, so the
               byte delta is 0 BY CONSTRUCTION and is the floor, not the grade
    Record:    `docs/whitebox/WB_INSTRCOUNT_FINDINGS.md` (the read); prereg
               `work/w-instrcount/PREREG.md`, committed at `71224feee` BEFORE
               the image was opened; the write census
               `work/w-instrcount/census_p50.py` + `classify_p50.py` with their
               outputs; the two confirmation probes
               `work/w-instrcount/f7_units.py` and `ceiling_units.py` with raw
               `jsonl` beside them

Charter: `docs/WAVE20_BRIEF_2026-08-29.md` §2 L2. Dispatched at master
`c5bfe89d9`. Board **#3824**–**#3830**, all seven spent.

---

## What it admits, and what it refuses

**Admits.** `[fn+0x50]` is `WORD [[fn]+0x50]` — a 16-bit field of the **`.gl`
symbol record**, reached through one indirection every clause text in this tree
omits (`0x10b626f5` loads `[fn+0x00]`, the symbol, before `0x10b626f7` loads the
field). It is the `.gl` function record's **`SIZE`** field, arriving verbatim
from the front end. **Its sole writer in the whole image is `0x10b9bf6c`**, the
`.gl` reader in `FUN_10b9b8e9` (`p2symtab.c`) — subject to one named, unclosed
blind spot (§ "Found and not taken" item 4). The unit is a **count of the front
end's instructions**: c2 sums the field across the compiland into **64-bit**
totals (`0x10b72eca`, `0x10b72f0f`), compares it against a ceiling built as
`0x10 << k`, and charges it against a growth budget in its own units — an
additive per-function magnitude. **No string in the image is printed from it**;
the two that say `instrs` belong to c2's *other* two counts, and this lane
reached for both before checking, which is recorded rather than tidied away.
Caller and callee read one field of one struct type.

**Refuses.** Three things this lane could have claimed and does not:

1. **That the port can carry it.** Adoption is a later wave's, off
   `w-clausegen`'s repaired screen. The strongest word available here is
   *derivable*, and it is used only where every field of a counterpart carries
   a `PROV[R]` address.
2. **A clean universal negative on the writer.** 28 `rep movsd` sites were
   cleared by reading each `ecx`; **119 `memcpy`/`memset` call sites were
   not**, and the page says so rather than rounding an unsearched class to
   zero. `#3505` is what happens otherwise.
3. **That the budget explains F6's site-count effect.** It is the nearest
   mechanism and the shape matches; the arithmetic does not close against the
   ceiling brackets (§6 of the findings), so it is ranked as the next read and
   deliberately not banked.

**And it corrects its own charter.** `WAVE20_BRIEF` §1 and `#3816` describe
`no-instr-count` as *"ONE missing link, four rows"*. It is one missing link and
the read closes it, but it was the **binding** blocker on **two** of the four —
C4 and C20 want the driver, which is the absence C5/C6 already name under
`no-instr-stream`. That correction is the deliverable the next wave is
dispatched off, and stating it at 2-of-4 rather than 4-of-4 is the whole point
of the brief's warning that overstating this list costs a wave.

### The clause verdicts, explicitly

| clause | verdict | why |
|---|---|---|
| **C2** | **UNBLOCKED** | the producing field is the `.gl` `SIZE` the port already decodes and discards (`gl.rs:GL_SIZE_ESCAPE_PAYLOAD`, `DISCLOSURE` W-GLATTRS-1); load `0x10b626f5`/`0x10b626f7`, store `0x10b62703`, producer `0x10b9bf6c` |
| **C16** | **UNBLOCKED** | seed (C2) + the `add` at `0x10b625c1` — which, unlike the budget subtract one instruction above it, is **not** gated by the 40 test — + an immediate at `0x10b60a63`; and measured 6.1× slack on this corpus, so an adoption is byte-neutral by construction like C15 |
| **C17** | **blocker removed, NOT adoptable** | both operands derivable, but `[ebp+0x10]` is the budget threaded through the driver's recursion and there is no driver |
| **C4** | **NOT unblocked** | the budget *argument* is derivable now; the driver, site collector and per-site loop are not, and that is `no-instr-stream`'s absence |
| **C20** | **NOT unblocked** | what stands between `fitted` and `R-derived` is the driver, exactly as for C4 |

## Estimate vs outcome

Prereg §4 priced the read at *"well under a lane"*: one function body, one seed
site, one image-wide write census. Realized: the read and the census took the
first two thirds of the lane; the part that was **not** priced and paid for
itself twice over was the pair of **confirmation** probes (`f7_units.py`,
`ceiling_units.py`, 20 captures total). They are what turned F7 from an
argument into a measurement and what produced `#3828`, which nobody asked for.

The bias worth recording for the next lane: **read-before-probe did not mean
no probe.** It meant the probe could be designed — prereg §4 registered that
the F7 grid *cannot* be designed without the unit first, and that is exactly
how it went. The probe cost ~20 captures because the read told it where to
put them; a black-box sweep for the same brackets is the 264-cell GRID-I that
already ran and could not resolve them, because it was measuring in emitted
bytes.

**Five registered predictions, one falsified, one split.** Scorecard in
`WB_INSTRCOUNT_FINDINGS.md` §9. The falsified one (P4a, "both F7 callers clamp
to the budget floor") is the more useful half of the F7 answer: the axis really
was varied — `B` measured **1,000 → 9,846** — and the null is a slack argument,
not a clamp argument.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,770 targets run, 1,770 pass, 0 fail** (see below for the exact recipe) |
| `c2rs selftest` | **392 PASS, 0 SKIP** — the worktree's toolchain resolves; `SKIP: toolchain absent` would have invalidated every row here |
| `scripts/gate.sh --jobs 16 --require-graded` | **`GATE: PASS`**, unqualified — *"18/18 lanes ran and every one of them graded a corpus, the sweep graded **19,542 of 19,638** generated cases and the cross graded **91,900 of 92,288** case-lane cells, with **0 mismatches anywhere** (96 sweep cases carried ungraded — the reference rejects the source), and 18/18 lanes ran again through a DEBUG-profile `c2rs` for **7,056** more fixture-verdicts at **0 panics**"*. Graded tree `a00a40351045`, 808 files |
| `scripts/expr_sweep.sh` | inside the gate row above |
| 878-TU workload scan | not run — no compiled file changed, so a scan would grade a byte-identical tree |
| fixtures, `c2rs census` | +0 by construction |

**Byte delta 0, and here it is not a criterion that abstained — it is
inapplicable.** `gate.sh` content-hashes `crates/ fixtures/ scripts/` and pinned
this run's graded tree at `71224feee` (the prereg commit); every later commit in
this lane touches only `docs/` and `work/`. A characterization lane's grade is
its prereg scorecard, and that is §9 of the findings page.

## Found and not taken

Ranked, with what each would unblock.

1. **`FUN_10b5fb5f`'s SECOND gate** (`#3830`) — the item this lane would take
   next if it had another day. `0x10b5fc90`'s `jl` is **not** accept and
   over-ceiling is **not** refuse: both paths meet `0x10b5fcb9`, which needs
   `DAT_10c2e2fc != 0` or `[sym+0x4c] & 0x2080`, and the over-ceiling path gets
   there through `test edi,eax` — **a caller-supplied `ATTR` mask, one of the
   function's five parameters, i.e. a decision point c2 already exposes as a
   parameter**. Two small reads: the three callers for `edi`, and
   `DAT_10c2e2fc`'s writer.

   **And the reason it is ranked first as an OPEN item rather than reported as a finding is worth reading.** This
   lane's first hypothesis was that `0x2080`'s bit 7 marks a foldable body and
   so explains `P_INLINE` §2.1b's matched pair. **Its own data killed that
   inside an hour** — `w-sizebracket`'s raw `series.jsonl` has both cells at
   `gl_attr = 0x68`, `gl_size = 115`, `caller_gl_size = 21`, same profile,
   opposite arms. That is `#3505`'s shape caught before publication rather than
   after, and it upgrades the §2.1b question: **every identified input to
   candidacy is identical across the pair**, so the separation is provably
   downstream of `0x10b5fc8a`, and this lane narrowed the search space rather
   than closing it.
2. **The linkage arm at `0x10b60a81`** — `test DWORD PTR [edi+0x37],0x400`, then
   `call 0x10b5de82`, sitting between C17 and the POGO model and **covered by no
   clause row of the 24**. It is the last unread thing between the ceiling this
   project has read (`DAT_10c46318 = min(0x10<<k, 1000)`) and the two it has
   measured, and this lane hands it a ready-made grade: **static `[261,267]`,
   external `[93,99]`, in the read unit** (`#3828`). No single `0x10 << k` fits
   either, and no single value fits both.
3. **Does the budget explain F6's site-count effect?** One caller, **≥ 8 call
   sites**, callees just under the ceiling, swept against caller size — the
   grid the first-site theorem says is necessary and 12 one-site cells could
   never be. 8–12 cells. It would either give C3 and C17 their **first `[O]`**
   (both are *READ, NOT CONFIRMED* today, and no `DISCLOSURE` row proposes
   either) or refute the budget as the mechanism behind `INLINE-P`'s fitted
   `n_sites` term. **This is the highest-value cheap thing on this page.**
4. **Close the 119 `memcpy`/`memset` call sites.** Bounded and mechanical; it
   is the only thing between `#3825`'s census and a clean universal negative.
   Worth doing *because* `P_INLINE` §2.1a already published the negative once
   from a grep.
5. **`FUN_10ba1eca`'s recount** — a second, 32-bit instruction count with its
   own 150-instruction *"won't be inlined (too big)"* gate that this inliner
   does **not** consult, read by `0x10b9e5d8` and `0x10ba3b7b`. Anyone modelling
   c2's inlining end to end will meet it, and the `%d instrs` diagnostic prints
   it, which is exactly the trap this lane fell into and recorded (P2's miss).
6. **`P_INLINE` §2.1b's matched pair.** Its headline — *"the `.gl` `SIZE` field
   is NOT the value the decision tests"* — is right; its stated mechanism,
   *"reduced by whatever runs before the inliner"*, has **no writer in the image
   to be the reducer**. Both cells are `SIZE = 115`, both below every ceiling
   bracket in `#3828`, so **candidacy admits both and the separation is
   downstream of it**. Re-explaining that pair is a real open item and is
   `w-clausegen`'s page to amend, not this lane's.

## Seam

Wrote: `docs/whitebox/WB_INSTRCOUNT_FINDINGS.md`, `work/w-instrcount/**`, this
rung, and board rows **#3824**–**#3829** only. Touched none of `crates/**`,
`docs/whitebox/ref/P_INLINE.md`, `work/w-inlmetric/**`, `P_GLOBREGS.md`,
`P_DAG.md`, `docs/STATUS.md`, `docs/rungs/INDEX.md`. **No `DISCLOSURE.md` row
is owed**: this lane adopted no disassembly-derived constant into `crates/`,
and the rule is that the row lands in the commit that adopts one.
