# w-paramfill — GATE 1 is 0 here, and it does NOT follow that the record is zero: there is a second, ungated copier, and reading a correct zero as an absence is how this lane nearly published the opposite

    Tag:       w-paramfill
    Slug:      w-paramfill
    Date:      2026-08-29
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization lane: it reads c2.dll and measures the toolchain's own command lines, and writes zero crates/ bytes
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Record:    `docs/whitebox/WB_PARAMFILL_FINDINGS.md`; page amendments `docs/whitebox/ref/P_INLINE.md` §6.9 (new), §6.1 (re-synced), §6.6.3 (correction block); prereg `work/w-paramfill/PREREG.md`, committed at `959281309` BEFORE the image was opened

Charter: `docs/ADOPTION_BRIEF_2026-08-29.md` §L3 — `w-inlswitch`'s own named
first-thing-next. Dispatched at master `12d3c0558`. Board **#3802**–**#3807**.

> **Predicted reach 0, delivered 0.** `git diff master..HEAD -- crates/` is
> empty. **No `DISCLOSURE.md` row** (that ledger records adoptions and this
> lane makes none), **no `scripts/gate.sh` row** (`#3691`), **no clause row
> added, removed or renumbered** — the reachable denominator is still 21 of 24
> (`#3505`), and `128` is neither adopted nor restated as the settled inline
> ceiling (`#3732`, `#3734`).

---

## What it admits, and what it refuses

**Admits:** `DAT_10c462c4` read end to end — its section, load-time value, both
writers, all 112 readers with their distribution, the switch that sets it, the
kind-`0x26` option arm that stores it, the call chain from c2's three entry
points to the fill, what the gate costs on this workload, and what it does not
cost. Plus an independent re-derivation of `w-inlswitch`'s two default sweeps,
and a re-sync of `P_INLINE` §6.1 to `CLAUSES.tsv`.

**Refuses:** every adoption, and one obj claim it could not earn. The
standalone `-Fl` probe's **baseline failed** (§"the control that failed"), so
no `[O]` row here is an obj comparison — all of them are `cl /Bd`
toolchain-invocation measurements. The sentence *"the live record holds table
A on every compilation"* is `[R]` and is marked `[R]`.

## Estimate vs outcome — the prereg's six predictions, seven graded halves

Registered in `work/w-paramfill/PREREG.md` §1 before the image was opened,
each with the observation that would refute it named in advance.

| # | prediction | conf | outcome |
|---|---|---:|---|
| **P1** | the gate is **non-zero** here, so the fill runs and §3 survives unchanged | 0.7 | **REFUTED.** `-Fl` is passed by 0 of 27 modes, and the unconditional writer's own path skips the fill |
| **P2** | the gate bounds §3's `live` column only, **not** `defA`/`defB`; and `DAT_10c46318` is computed before the gate | 0.8 | **HELD, both halves.** Both fillers and the `16 << k` ceiling are on the near side of the `cmp` |
| **P3** | **≥ 3** write instructions | 0.55 | **REFUTED — exactly 2**, agreed to the address by objdump, Ghidra and a decode-independent byte scan. Registered *because* the base rate was bad (10→19 last wave); the base rate did not repeat |
| **P4(a)** | it is the **`-Og`** optimisations-on flag | 0.45 | **REFUTED** — `-Fl#`, kind `0x2601`, a 200-entry file list |
| **P4(b)** | it is **global, not inline-specific** | 0.85 | **HELD** — 112 reads in 78 owner functions, **4** of 112 in the inliner band |
| **P5** | 37 / 33 / 33 / 46 re-derive exactly | 0.85 | **HELD on 37, 33, 33, 46 — and it found a 34th store.** Table A has 34 store instructions over 33 fields, and the duplicate changes a published default |
| **P6** | of §3's statements: 0 false, ≥1 needs a condition | 0.6 | **REFUTED on the headline half — 4 are false**, and P6's own "≥1 needs a condition" held |

**Four of seven registered halves refuted by this lane's own measurements.**
The prereg's most useful line was **P3**, registered *against* this lane's own
prior and lost: the writer count really was 2, and the reason to publish that
is that "the last lane's count was wrong so this one's will be too" is not a
method.

## The results, in one table

| target | result | where |
|---|---|---|
| **`DAT_10c462c4`** | a **one-way latch**: BSS-zero, 2 writers both storing 1, **no writer of 0 in the image**. 114 refs / 112 reads / 78 owner functions, 4 in the inliner band | `WB_PARAMFILL_FINDINGS` §2, `P_INLINE` §6.9.0 |
| **what sets it** | **`-Fl<file>`**, `0x10c45fa0`, kind **`0x2601`** — the only such row of 148. A 200-entry repeatable file list whose array is **write-only in `c2.dll`** | §3, §6.9.1 |
| **the chain** | `FUN_10bec3d3` sets the gate to 1 **and** `DAT_10c2eb38` to 1, and `FUN_10b7f3b6` uses `DAT_10c2eb38` to jump *over* the fill. So on the path that reaches the fill, only `-Fl` can open the gate | §4, §6.9.2 |
| **the measurement** | **`-Fl`: 0 hits over 27 `cl /Bd` mode rows.** `-optref` 0. `-ltcg` **4** — every `/GL` row. `/FAsc` passes `-FAasc -Fa`, **not** `-Fl` | §5, §6.9.3 |
| **what the gate costs** | **one thing: `FUN_10b5b9de`**, the module-size trim. That **refutes** `WB_INLSWITCH_FINDINGS` §9 item 2 — `-inlT#` is exactly **104**, not a range 80–136 | §6, §6.9.5 |
| **what it does NOT cost** | the live record. **`FUN_10b5b86d` at `0x10b5b88a` is a second copier, ungated, once per code-generated function** | §7, §6.9.4 |
| **`-inlfcsa#`** | table A stores `+0x40` **twice** in a straight-line sweep — 20 then 5 — and the first wins. §3's `defA = 5` is **false**; "13 differ / 8 switch-fed" is **14 and 9** | §9, §6.9.6 |

### The one that is the lane's actual result

**A per-field reference census over all 46 live fields returns `0 WRITEs`, and
that is correct, and it means nothing.** A `rep movsd` writes through `EDI`;
the destination appears once, as an immediate, one instruction earlier. Read as
an absence it gives the confident headline *"GATE 1 is 0, therefore the record
stays all-zero, therefore the 24 switches are inert for a second reason"* —
which was drafted, and is wrong.

What refuted it was **an index of the same fact already in this repo**:
`P_INLINE` §6.1's **C23** row names `0x10b5b86d`, and `dump_inlswitch.py`'s own
docstring describes the copy. Neither had reached §6.8.2's prose. `#3505` goes
to **six for six** — and this instance is the sharpest, because the instrument
was not broken: it answered a different question correctly.

## Gate evidence

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 16 --require-graded` | **`GATE: PASS`** — unqualified, with `hatch-red` live: `#3786` re-anchored the needle and this tree is clean, so `#1406`'s caveat on a `(HATCH-RED REFUSED)` run does not apply here. **18/18 lanes PASS**, 0 FAIL, 0 SKIP, 0 NO-RESULT, **7,038 fixture-verdicts**; sweep **19,460 of 19,556** graded, **0 mismatch**; cross **90,424 of 90,812** cells, **0 mismatch**; `hatch-red` **live and PASS, 14/14 arms, 11 red, 3 green controls**. Transcript `work/w-paramfill/gate_tip.out`, run at `e5fcf6d6a`; the only commits after it touch `docs/rungs/` and `work/w-paramfill/`, and `crates/` and `fixtures/` are byte-identical to `master` at both ends, which is what the gate hashes |
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,992 passed / 2 failed / 2 ignored** over 62 result blocks. **Both failures are named, and one of them was this lane's own defect.** Transcript `work/w-paramfill/cargo_test_tip.out`, which carries them in its header rather than being replaced by a clean re-run |
| `git diff master..HEAD -- crates/` | **empty** — required, and this lane's whole grading criterion |
| `python3 work/w-front3/hatch.py check` | `0 of 8 present, 8 pending, 0 undecidable`, `crates/ diff: EMPTY`, `CLEAN` |
| `python3 work/w-inlmetric/check_table.py` | `CONFORMANCE-CHECK: GREEN (0 failures over 24 rows)` — **read only**; `CLAUSES.tsv` was not touched (`w-inlclause` owns it this wave) |

> A characterization lane's gate is a **required-zero** result, not a movement.
> The axis on which it can fail with every byte identical is its **controls**:
> `WB_PARAMFILL_FINDINGS` §1 records C1 GREEN, C2 RED, C3 RED and a
> cross-population Ghidra check, **and §8 records one control this lane could
> not turn green and therefore quotes nothing from.**

### The two suite failures, and the one that was real

**`tracked_artifact_audit::the_index_carries_no_artifact_claude_md_forbids` —
class 3, `VIOLATION work/w-paramfill/gate_tip.out`.** The gate transcript was
committed carrying this box's absolute paths. That is `CLAUDE.md`'s own rule and
the box convention's own rule, and **the absolute-path grep had been run** — over
`docs/BOARD.md`, both new `docs/` pages, the instrument, and every other `work/`
output. The gate transcript simply was not in the set that was remembered.
**An enumerated check beat a remembered one**, which is the same shape as
`#3689` and `#3679` and is why the check exists. Fixed by sanitizing the
transcript to `<WORKTREE>` / `<REPO>` / `<HOME>` — no count, verdict or hash
edited — and re-run green (`2 passed; 0 failed`).

**`rung_registry::rung_index_is_generated_and_current`.**
`docs/rungs/INDEX.md` is **generated** and this lane added a rung doc.
`ADOPTION_BRIEF_2026-08-29.md` §4 says INDEX.md is regenerated **at merge**, and
this lane's dispatch lists it under MUST NOT TOUCH — with two lanes adding rung
docs this wave, each regenerating it is an add/add conflict on a generated file.
**So it is left stale deliberately and the failure is reported rather than
resolved**, which is the opposite of what `w-inlswitch` did last wave (it
regenerated) and is a difference in the wave's instructions, not in the rule.
Never regenerate it by hand; `scripts/gen_rung_index.sh` is the only writer.

### The control that failed

The decisive measurement for §6 would be an obj: run standalone c2 twice on one
captured bundle, once with `-Fl` appended, byte-compare. **Its baseline
failed.** `c2rs replay` succeeds (`ref=878B replay=878B
normalized_identical=true`) and `c2rs selftest` is all `PASS`, so the toolchain
is present. But a hand-built invocation of the same `c2host.exe`, same `wibo`,
same `c2.dll`, same captured bundle, and the argv template transcribed from
`crates/c2-reference/src/lib.rs:1709` aborts inside c2 with
`wibo: call reached missing import lstrcatW from kernel32` — at both argv
orderings, from three working directories, with and without `TMP`/`TEMP`.

**No obj-level claim is made from it.** The discrepancy is filed as the
top not-reached item, because it bounds what *any* future by-hand standalone
probe can claim: until it is explained, *"the standalone replay is reproducible
by hand"* is not a claim this repo can make. The fix — teach
`Toolchain::build_replay_command` to print its argv or accept extra tokens — is
a `crates/` change this lane may not make.

> **And one instrument defect caught the same way, twice.** The default-sweep
> matcher accepted only `mov ds:F,<imm>` and reported **10** stores for table A
> where there are 34: half the sweep stores its constant through a register
> (`push 0x20 / pop edx / mov ds:F,edx` is 7 bytes against a 10-byte immediate
> form). Widened, it still reported `-inlniln#` as **0** (it is `1`, via
> `xor edx,edx / inc edx`) and missed table B's `+0x44 = -1` entirely because
> that one is an **`or ds:F,0xffffffff`**. Both were caught by **disagreeing
> with `w-inlswitch`'s published cell** — which is the only reason the third
> pass's agreement on 32 of 33 is worth quoting. A re-derivation that matches on
> the first attempt has tested less than one that does not.

## `P_INLINE` §6.1 re-synced — two divergences, not one

`w-clausefix` §8 item 2 left ten address repairs as a paste-ready block because
it did not own the page. **This lane owns it and applied all ten, after
independent re-derivation** (`work/w-paramfill/clause_addr_recheck.out`): all
ten new addresses are decoded instruction starts over **424,232** starts,
**eight of the ten old ones are not**, and each new address decodes to the
instruction its clause names — `cmp ds:0x10c3f5cc,0x88b8` (= 35000) for C16,
`cmp eax,0x28` (= 40) for C18, `call 0x10b61ee1` for C4, and so on.

**And a second divergence nobody had filed.** C3 and C19 were converted from
`absent` to `[R]`-derived by `w-inlbudget` in wave 18; the conversion reached
`CLAUSES.tsv` and not the page, so §6.1 read `absent 17 · [R]-derived 2` while
`check_table.py` printed **GREEN** over `absent 15 · R-derived 4`. Both cells
and the split line are now synced, at master `12d3c0558`.

> **A grader that is green on the machine table proves nothing about the prose
> copy of it.** That is `#3785`/`#3679` one level up, and the durable fix —
> generate §6.1 from `CLAUSES.tsv` — is **named and not built**, because
> `check_table.py` is `w-inlmetric`'s instrument and a lane that does not own it
> should not decide its output format. **`w-inlclause` may convert further rows
> this wave and cannot edit this page**; the merge has to re-sync §6.1 again.

## Found and not taken

Ranked, sized, with what stopped each. Full list at
`WB_PARAMFILL_FINDINGS.md` §10.

1. **The hand `c2host` invocation that does not reproduce `c2rs replay`** —
   above. One hour, and it unlocks the obj-level test of every gate in
   `P_INLINE` §6.8–§6.9.
2. **`FUN_10b7f1ff`'s seven reads of the gate** select between two whole driver
   bodies. Only the `.gl`/`.sy`/`.ex`/`.in` name derivation was read;
   `call 0x10b72f0a` / `0x10b72f21` / `0x10b734f7` (the gate ≠ 0 arms) are
   unopened. **This is c2's whole-program driver and nobody has read it.**
3. **74 of the 80 owner functions of the 112 reads.** Anything of the form "c2
   behaves differently under LTCG" is in that set.
4. **`FUN_10b5b9de`'s six bands** are dead here but real; a `-Fl` compilation
   takes them.
5. **`FUN_10b73634`**, the second caller of `FUN_10bec3d3`, has no callers in
   `calls.tsv` — a vtable or callback target not resolved.
6. **`FUN_10c1f572` kinds `0x08`, `0x23`, `0x27`** are still unread; `0x26` is
   closed here because `-Fl` needed it.

## What this lane did not reach

* **Nothing was graded against an obj.** §8's control failed. Every `[O]` row
  is a `cl /Bd` toolchain-invocation measurement.
* **`-Fl`'s expansion is not named**; §6.4's whole-program reading is `[I]`.
* **§3's `readers` column was not re-derived.** This lane re-derived the
  defaults and the counts it names; the 39 reader addresses are
  `w-inlswitch`'s and are unchanged by this lane, which is stated rather than
  implied.
* **The brief's own citation was wrong and is corrected, not followed.**
  `ADOPTION_BRIEF_2026-08-29.md` §L3 and the `BOARD.md` ledger say the gate
  *"bounds every statement in `P_INLINE` §3"*. `P_INLINE` §3 is GRID-I's 264
  obj-measured cells and is downstream of nothing here. The §3 that is bounded
  is `WB_INLSWITCH_FINDINGS.md` §3, which is what `w-inlswitch`'s rung wrote.
  **Dated records stay as written**; the correction is here and in board row
  **#3802**.
