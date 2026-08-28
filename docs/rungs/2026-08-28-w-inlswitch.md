# w-inlswitch — c2's 24 inline knobs are the POGO tables' own overrides, the "narration switch" is `-pgo#` and it narrates nothing, and `k`'s run-time value is settled

    Tag:       w-inlswitch
    Slug:      w-inlswitch
    Date:      2026-08-28
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization lane: it reads c2.dll and measures the toolchain's own command lines, and writes zero crates/ bytes
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Record:    `docs/whitebox/WB_INLSWITCH_FINDINGS.md`; page amendment `docs/whitebox/ref/P_INLINE.md` §6.8; prereg `work/w-inlswitch/PREREG.md`, committed at `5eec1a7d5` BEFORE the image was opened

Charter: `docs/DECISIONS_2026-08-22.md` decision 22, the `w-inlswitch` row, and
`docs/ADOPTION_BRIEF_2026-08-28.md` §L4. Dispatched at master `4b79bf46a`.
Board **#3768**–**#3773**.

> **Predicted reach 0, delivered 0.** `git diff master..HEAD -- crates/` is
> empty. **No `DISCLOSURE.md` row** (a disclosure row accompanies an adoption
> and this lane makes none), **no `scripts/gate.sh` row** (`#3691`), **no
> clause row** added, removed, renumbered or restated — the reachable
> denominator is still 21 of 24 and is not restated as 24 (`#3505`).

---

## What it admits, and what it refuses

**Admits:** an address-cited map of c2's own inline decision surface — the 24
`-inl*` switches with their descriptor records, value words, destination
fields, **both** default sets, live addresses and readers; the writer walk of
`DAT_10c3de20` and its command-line origin; `FUN_10b5da2f` end to end; and the
run-time value of `k`.

**Refuses:** every adoption. All three targets are POGO-gated and **dead on
this workload**, measured, not assumed. Nothing here licenses an emit, `128` is
not adopted and is not restated as settled (decision 22 §3, `#3732`), and
`P_INLINE` §6.6.1's `fitted` verdict on C8 is untouched.

## Estimate vs outcome — the prereg's five predictions

Registered in `work/w-inlswitch/PREREG.md` §1 before the image was opened, with
the refutation of each named in advance.

| # | prediction | outcome |
|---|---|---|
| **P1** | ≥ 12 of 24 carry an initializing store at their own value word | **REFUTED — 0 of 24.** The words are BSS and stay 0; the operative default is installed at the *destination* field by a zero-guarded sweep. Locating that mechanism is what the miss bought |
| **P2** | ≤ 11 of 24 have any reader; ≤ 6 tied to a named decision | **REFUTED TWICE — 24 of 24 have a reader, and the in-band decision is named for all 24.** Not one is vestigial. Five of them have further readers outside the band that were not opened |
| **P3(a)** | no writer of `DAT_10c3de20` traces to a resolvable switch name | **REFUTED — and this is the lane's best result.** `-pgo#` / `-po#` / `-pgu#`, via `DAT_10c6f1c8` at `0x10b84b47`/`0x10b84b58` |
| **P3(b)** | it is a compilation-MODE selector, not a diagnostic; setting it to 2 would *change* the decisions, not report them; mode axis = whole-program operation | **HELD, including the registered guess at the axis.** `{0,1,2}` = `{no POGO, instrument, optimize/update}` |
| **P4** | `FUN_10b5da2f` is NOT on the inline path this workload takes | **REFUTED — it is in the band**, sole caller `0x10b5eb27` inside `FUN_10b5e9a5` |
| **P5** | `k`'s run-time value is settleable, and it is 3 | **HELD**, by the registered route: read the kind-`0x2401` setter, then confirm no invocation passes `-vol` |

**Three of five refuted by the lane's own measurements**, which is the shape
`w-lowerband` established and the reason the prereg is worth the commit.

**The count in the brief was already wrong before the image was opened**, and
the prereg says so in §0 rather than dressing it as a prediction: `#3718` and
`ADOPTION_BRIEF_2026-08-28.md` §L4 both say **21** `-inl*#` switches; the
artifact they cite names **24** over 24 contiguous dwords. Re-derived here from
the image, byte-identical to `w-inlfit`'s committed output.

## The four results, in one table

| target | result | where |
|---|---|---|
| **the 24 switches** | **24 named, 24 with a reader, the in-band decision named for all 24, both default sets recovered** — and they are the override inputs to `P_INLINE` §5's two "unquotable" POGO tables, not inputs to the inliner | `WB_INLSWITCH_FINDINGS.md` §1–§4, `P_INLINE` §6.8.0–§6.8.3 |
| **`DAT_10c3de20`** | **19 writers in 13 functions, not 10.** It is the EFFECTIVE POGO mode; `-pgo#`/`-po#`/`-pgu#` set it to 2. **The "narrates its own inline decisions" claim is FALSE** | §5, §6.8.4 |
| **`FUN_10b5da2f`** | a **budgeted statement-cost test**, budget `k · (n + 2 + …)`, in the band. Its "second reader of `k`" is a **loop reload after `neg ecx`** | §6, §6.8.5 |
| **`k` at run time** | **settled at 3**, hence `DAT_10c46318 = 128`. `#3734`'s open question is closed — and 128 is still not adopted | §7, §6.8.6 |

### The one that changes how the next inliner lane measures anything

`P_INLINE` §5 says the two 46-dword POGO parameter tables are BSS and *"none of
their values is quotable from the image."* The premise is right; the conclusion
is too strong. `FUN_10b5b88f` scatters 37 switch words into a 46-field record,
its two callers pass the two tables, and each caller then runs **33
zero-guarded default stores**. **Both default sets are recoverable by exactly
the method §5's own page uses for the descriptor table**, and the live table is
selected by `DAT_10c6f1c8` at `0x10b5e50f` — measured `0` here, so **table A is
the live one and its defaults are the operative ones**.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | see `work/w-inlswitch/cargo_test_tip.out` — counts in §"Gate transcripts" below |
| `scripts/gate.sh --jobs 16 --require-graded` | see `work/w-inlswitch/gate_tip.out` — verdict line quoted below |
| `git diff master..HEAD -- crates/` | **empty** — required, and the lane's whole grading criterion |

> A characterization lane's gate is a **required-zero** result, not a movement.
> The axis on which this lane can fail even with every byte identical is its
> **controls**: an absence claim from a broken enumerator is the failure mode
> this repo has paid for four times. §8 of the findings page records C1 GREEN,
> C2 RED, C3 RED, a cross-population Ghidra check, **and one control this lane
> failed and caught** before publishing.

### The control that failed

A probe of `/Gy` propagation reported that `cl /Ox /Gy` does not pass `-Gy` to
c2 — which would have made `scripts/lanes.txt`'s `Ox-Gy` and `Ox-Gy-EHsc` rows
byte-identical duplicates of `Ox` and `Ox-EHsc`, and was on its way to being a
board row. **It was the instrument, not cl.** The loop wrote the mode as an
unquoted `$m`, and **zsh does not word-split unquoted parameter expansions**,
so `cl.exe` received `/Ox /Gy` as a single argument and parsed only `/Ox`; the
same defect silently dropped `/EHsc` from every multi-flag row. Re-derived with
the flags as separate argv entries, `-Gy` **is** passed at every ordering.
**No lane is a duplicate and no row is owed.** The regenerated
`work/w-inlswitch/cl_argv_modes.out` uses zsh's explicit `${=m}` and carries
the defect in its own header.

> The same defect class bit this lane a second time, in process management
> rather than measurement: a `pgrep -f 'w-inlswitch/finish.sh'` reported the
> script **still alive** after it had been killed, because the pattern was
> present in the checking shell's own argv. Both are one assumption — *"the
> shell passed the command what I wrote"* — and neither is visible in an exit
> code. The lane's sequencer now uses the bracket trick and a deadline that
> reports `TIMEOUT` as an outcome distinct from success.
>
> **And a third, which is the one worth carrying forward.** The bracket-tricked
> waiter `pgrep -f '[c]argo test --workspace --release'` is correct about
> self-matching and **wrong about scope in a five-lane wave**: four peers run
> the identical command line in their own worktrees, so the predicate stayed
> true long after this lane's suite was the only thing left to wait for. It was
> caught by reading `/proc/<pid>/cwd` for every match — **the reported elapsed
> time went DOWN between two polls**, which is only possible if the poll had
> silently switched to a different process. Fixed by waiting on a PID, which is
> `CLAUDE.md`'s own FIX 1 and has no pattern to be wrong about. **In a
> worktree-per-lane protocol, a correct `pgrep -f` predicate is still a
> repo-wide one.**

## Found and not taken

Ranked, sized, with what stopped each. Full list at
`WB_INLSWITCH_FINDINGS.md` §9.

1. **`DAT_10c462c4` gates the entire parameter fill** at `0x10b5e4f7` and is
   tested against zero in ~110 places image-wide. Two writers, one of them
   unconditional at `0x10bec3e4`. Not read. **It bounds every statement in §3
   of the findings page** — if it is ever 0, the live 46-dword record stays
   zero and the 24 switches are inert for a second, independent reason. One
   hour, and it is the first thing the next lane here should do.
2. **`FUN_10b5b9de`** trims table A's `-inlT#` and `-inlfcsw#` **by module
   size**, in six bands at 50 k / 100 k / 500 k / 1.5 M / 2.5 M
   (`+32/+24`, `+24/+16`, `+16`, `−8`, `−16`, `−24`). So `-inlT#`'s effective
   default is a **range, 80–136**, not the 104 the sweep installs. Read, not
   graded, because the field is POGO-dead here.
3. **12 of the 39 switch reads are outside the band** — `FUN_10bb7aa3` (5),
   `FUN_10ba24c4` (4), `FUN_10ba2948` (3) — and were not opened. 25 of the 39
   are in `FUN_10b5fcd8` and 2 in `FUN_10b5dc6c`.
4. **`FUN_10b5e9a5`**, `FUN_10b5da2f`'s sole caller: what it does with the
   returned 0/1 is unread, so §6's test is characterized without its consumer.
5. **The 13 dead parameter fields** `+0x84`…`+0xb4`, fed from
   `0x10c45d80`–`0x10c45db0`: no switch name, no default in either sweep, no
   reader — copied on every initialisation and used by nothing.
6. **`FUN_10c1f572`'s kinds `0x08`/`0x23`/`0x26`/`0x27`** were not read, so
   `optmap.py`'s remaining `(reg)` rows are still unresolved. This lane
   resolved four of them (`-pgi#`, `-pgo#`, `-pi#`, `-po#`) because the mode
   chain needed them, and found **two alias pairs on one word each**.
7. **`hatch-red` cannot run for ANY lane in this wave, and the cause is on
   `master`.** Not this lane's to fix and **not filed as a board row** — all six
   of `#3768`–`#3773` are spent and a lane writes rows only inside its own
   block — so it is recorded here and in the report instead.
   `work/w-front3/hatch.py apply` refuses `HATCH-DRIFT` on
   `crates/c2-il/src/func/body/shapes/calls.rs`: the `call-arg-lit-permuted`
   edit's needle is `if !in_place && !one_moved_at_two &&
   !permutation_decided_downstream {` and `master`'s line 596 reads
   `… && !permutation_decided_downstream && !lit_inserted {`. **It predates
   this lane by construction** — `git diff master..HEAD -- crates/` is empty
   and the worktree copy is clean, so the drift is `master`'s. Until the needle
   is re-anchored, every lane's gate line will read
   `GATE: PASS (HATCH-RED REFUSED)` and `#1406`'s point stands: that run does
   not establish what a full run establishes. `#3219`'s liveness counter reports
   *"1 consecutive run"*, which its own text says is a **floor**, not the age —
   this worktree's `gate_row_history.tsv` starts empty.

## What this lane did not reach

* **Nothing was graded against an obj.** Everything it read is POGO-gated and
  this project compiles no POGO, so the `[O]` rows are toolchain-invocation
  measurements (`cl /Bd`), not obj comparisons.
* **The `-inl*` abbreviations are not expanded.** `csw`, `dasw`, `casw`,
  `ocsa`, `mlsa`, `crmax`, `fcsa`, `ipfw`, `nlw` are given as arithmetic, not
  as names; expanding them would be `[I]` dressed as `[R]`.
* **`DAT_10c462c4`** — item 1 above.
