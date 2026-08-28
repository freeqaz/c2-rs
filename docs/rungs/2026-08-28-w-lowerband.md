# w-lowerband — the `[sym+0x50]` reduction does not exist: one writer, nine readers, and the missing link is AT the store

    Tag:       w-lowerband
    Slug:      w-lowerband
    Date:      2026-08-28
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization lane: it reads c2.dll and re-reads an
               existing measured series, writes zero crates/ bytes and admits
               nothing
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Reach:     0, as predicted
    Record:    docs/whitebox/WB_LOWERBAND_FINDINGS.md and
               docs/whitebox/ref/P_INLINE.md §6.7; prereg
               work/w-lowerband/PREREG.md, committed at `19d6c4797` BEFORE the
               image was opened

Charter: `docs/DECISIONS_2026-08-22.md` decision 21, the `w-lowerband` row —
*"the `[sym+0x50]` reduction chain between `0x10b9bf6c` and `0x10b5fc8a` … the
inliner's blocker, and it is **located but unread**"*. Dispatched at master
`8213c7b77`. Board **#3731**–**#3736**.

> **Predicted reach 0, delivered 0.** `git diff master..HEAD -- crates/` is
> empty. No `DISCLOSURE.md` row, no `gate.sh` row (`#3691`), no clause row of
> `P_INLINE` §6.1 added/removed/renumbered/restated, `INLINE-P` untouched, and
> **128 not adopted**.

---

## 1. What it admits, and what it refuses

**It admits nothing.** A characterization lane's deliverable is address-cited
findings under prereg, and this one's is a **reference set**: the complete
enumeration of what reads and what writes `WORD [sym+0x50]` on c2's `.gl`
function-symbol record.

**It refuses to name a reduction**, because there is none to name — see §2.
It also refuses the two things the brief fenced: it does not touch count→bytes
(§5), and it changes no emit.

## 2. The result, in one table

| the brief asked | the answer |
|---|---|
| *"Find what reads and reduces `[sym+0x50]`"* | **9 readers, 1 writer, 0 reducers.** The writer is `0x10b9bf6c` and it stores `il-read-varint16`'s return **verbatim**, with no instruction between the `call` and the `mov` |
| *"report the reduction as an enumerated chain of sites"* | **The reduction chain has length zero.** What is on the path is a **pointer-selection** chain of three sites — which record C8 reads, and whether C8's test runs at all |
| *"If the chain is longer or more conditional than the fixture pair suggests, that is the result"* | It is **shorter**: it is empty. And **two of the three selection sites are inside `0x10b5b86d`–`0x10b62b00`**, the band `P_INLINE` §6.6 says cannot contain the answer |

## 3. Estimate vs outcome — the prereg LOST, and that is the value

`work/w-lowerband/PREREG.md` §3, registered before the first `grep`:

| # | prediction | outcome |
|---|---|---|
| **P1** | the write set is **larger than one** (2–6); §2.1a's *"exactly ONE 16-bit store"* is an **instrument claim** | **MISS.** §2.1a is right. All four named omission forms — RMW, dword-width, computed pointer, block copy — were searched and **all four are absent** |
| **P2** | `arith_012`/`mix_008` separated by **one** site, not many | **HOLDS vacuously** — zero value-changing sites, so not ≥3 |
| **P3** | the mechanism is a **recompute**, not a decrement | **Unscoreable in consequence** — there is no mechanism to classify. Reported, not quietly dropped |
| **P4** | no site that **changes the value** is in the band | **HOLDS.** No such site exists anywhere. The in-band sites change the **operand**, not the value — a distinction the prereg's wording happens to make correctly, and this rung does not claim more credit than that |
| **P5** | the negative branch: if the answer is one writer, **say so, name the enumeration and its population, name the alternative carriers, do not fit a formula, STOP** | **THIS IS THE BRANCH THAT RAN**, and it was followed to the letter |
| **P6** | reach 0 | **HOLDS** |

**The prereg's value here is that it lost.** Registering *"§2.1a is probably an
instrument claim"* and then finding §2.1a correct is worth more than the
confirmation would have been — because P5 fixed the null result's write-up in
advance, so the lane could not dress it as a discovery. The finding is **not**
"a reduction was found elsewhere"; it is **that the reduction was never there**.

## 4. The axis on which this lane could have failed, and did not

Three instruments over three different populations, because *"nothing writes X"*
is a claim about a tool until the reference set is enumerated:

| instrument | population | result |
|---|---|---|
| `f50.py` | the **independent objdump boundary set**, **424,232** decoded instructions (= 425,871 addressed lines − 1,639 byte-continuation lines; `#3721`'s denominator is the former) | 125 operands at `+0x50`; **1** 16-bit write |
| Ghidra's decompiler — **control-flow-driven, not linear** | the whole export | **0** `ushort` assignments at `+0x50` image-wide |
| `bytescan.py` — **decode-independent** | **all 1,232,384 bytes of `.text`**, **2,136** encoding patterns | **exactly one** 16-bit-store encoding present |

The third is load-bearing: `objdump` sweeps `.text` linearly and c2 has a
~150 KB data block at its head (Ghidra's first function is `0x10b266d0`), so a
store inside a desynchronised run would be invisible to the listing — **which is
the exact shape of the four prior "no cell / no reader exists" defects this
repo has recorded.**

**Controls (`work/w-lowerband/controls.out`, `#3336`):** C1 and C3 positive
**GREEN**; C2 and C4 **watched RED** on a planted one-byte shift of the known
store — one in the listing, one in a **copy** of the image (C5 checks the tree's
own image is unmodified) — before any count from either was quoted.

## 5. Where it stopped

Decision 21 and the brief scope this lane **off count→bytes**. §3 of the
findings walks up to it — the `inlined`/`kept` split is clean in the **emitted**
`.text` and absent in `.gl SIZE` — and stops. **The boundary is `0x10b5fc8a`'s
left operand**: everything upstream of the `movzx` at `0x10b5fc86` is
enumerated; the converter is not opened, not modelled and not priced.

## 6. What contradicts the brief and `P_INLINE` §6.6 — reported because a refutation is worth more than a confirmation

1. **§6.6.1's first missing link does not exist** (`#3731`). *"Reduced by every
   pass that runs between there and `0x10b5fc8a`"* is refuted as a statement
   about the image.
2. **§6.6's stated REASON is wrong, though its conclusion holds** (`#3733`).
   *"Not replaceable by any read confined to the band"* rests on both links
   being outside it; what stands in link 1's place — S1 `0x10b5fb6e`
   (`FUN_10b5bfae`, 18 B, 13 callers, **two** resolutions), S2 `0x10b5fbf3`
   (**C8's operand replaced by `[sym+0x90]`**), S3 `0x10b624c6`/`0x10b6255a`
   (the charge **saves, overrides and restores** the favour-speed global around
   the expansion) — is **two-thirds in-band and was unread**. A lane that had
   trusted *"not in the band"* would not have looked at `0x10b5fbf3`.
3. **The brief's own framing is refuted with it.** *"`[sym+0x50]` … is reduced
   by every pass that runs between there and `0x10b5fc8a` — and nothing yet
   located reads that reduction. **That is your subject.**"* The subject turned
   out to be a null.
4. **Two corrections to `#3717`/`#3718`** (`#3734`), both about instruments:
   `k` has **three** readers not two (`0x10b5dacb` is new), and *"never stored
   by any instruction"* is exact about direct-addressed stores while `k`'s
   address sits in the `-vol#` descriptor — so `k = 3` is the **load-time**
   value and the run-time one is not settled by that enumeration.
5. **The favour-speed bit has THREE homes** (`#3735`), and this row **replaces
   a claim this lane nearly shipped.** `DAT_10c2e310`'s image value is `1` —
   non-zero means C8's size test is *skipped* — but `FUN_10b82338` writes it
   from bit 23 of a **per-function option word**, so *"the default is on,
   therefore `/O1` clears it"* does **not** follow and is not claimed. What does
   follow: on the branch where `DAT_10c3de20 == 2` the same bit goes to a
   **different global** and `DAT_10c2e310` is **never written**; and
   `0x10b82352` stores that option word into `[[…]+0x80]+0x76`, **the exact
   field S3 reads with mask `0x800000`** — so S3 restores the favour-speed bit
   to the **callee's own `/Ot`-vs-`/Os` setting** for its expansion. The same
   read also reconciles §2.1's *"`ebx = 0`"* with §6.5's *"`ebx` holds
   `[sym+0x4c]`"* — both true; `ebx` is re-zeroed at `0x10b5fc42`/`0x10b5fc67`.
6. **A trap closed before anyone walks into it** (`#3732`): §2.1b's one-sided
   `SIZE < T ⇒ inlined` holds at `T = 98` and **must not be raised to the
   image's 128**. Re-reading `w-sizebracket`'s already-committed 168 cells — no
   recompilation — gives **8 counterexamples in each direction at `/O1`**.

## 7. Named follow-ups, NOT pursued and NOT adopted

Filed rather than taken, per `w-inlfit` §6.5's convention: `FUN_10b566e9`'s
`& 0x3f` view of the same field; `FUN_10b8fb47` mixing it into an **IL hash**;
`DAT_10c3de20` (389 refs, 10 writers, three values) — **naming the switch that
sets it to `2` would make c2 narrate its own inline decisions**, which is the
direct measurement of the quantity this whole thread is about, and it is not in
the descriptor table `optmap.py` recovers; `0x10b9bf75`'s `and eax,0xfffffffb`
clearing ATTR bit 2 at load; and `FUN_10b5da2f` (573 B, unread), which reads `k`
twice.

## 8. Gate

Full `scripts/gate.sh` and `cargo test --workspace --release` at this lane's
tip; transcripts `work/w-lowerband/gate_tip.out` and
`work/w-lowerband/cargo_test_tip.out`. The `GATE:` verdict **line** is the
verdict, not the exit code:

```
lanes:  18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
        checked=19556 mismatches=0 graded=19460 ungraded=96 unknown=0   (sweep)
        checked=90812 mismatches=0 graded=90424 ungraded=388 unknown=0  (cross)
        7038 fixture-verdicts, match 2479, 0 mismatch, 0 PANIC
GATE: PASS (HATCH-RED REFUSED) — 18/18 lanes ran and every one of them graded a corpus,
  the sweep graded 19460 of 19556 generated cases and the cross graded
  90424 of 90812 case-lane cells, with 0 mismatches anywhere
```

**A required-zero byte delta is not this lane's criterion** — it is a
characterization lane, not a construct rung, and `git diff 8213c7b77..HEAD --
crates/` is empty, so the gate cannot be evidence *for* it either. The gate is
here to show the tree was not broken, and **`mismatch 0` is not evidence of
correctness** (`docs/STATUS.md`'s standing trap).

**The transcripts are sanitized of the worktree's absolute path** before
committing (`<worktree>`), per `CLAUDE.md` § "Never commit … absolute machine
paths"; the `GATE:` lines and every count are untouched.

**`cargo test` was run twice on purpose.** The first run predated this rung
file, and `rung_registry.rs` reads `docs/rungs/` — so a green from it would
have graded a tree that did not contain the thing most likely to fail. The
committed transcript is the second run, at the tip.
