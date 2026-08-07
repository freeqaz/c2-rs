# w-relo — HANDOFF

**Written for whoever picks this up next. Everything below is verified state, not
plan.** The lane is **finished and committed**; what remains is a rebase and a
merge, which the coordinator owns.

    Branch:   wt-w-relo
    Tip:      c5bd8cb7
    Base:     master 22816a5  (NOT rebased onto 91a47df5 — coordinator's call)
    Worktree: .claude/worktrees/agent-a22fc764eb4b6022e
    Status:   clean, 0 modified files, nothing pushed
    Board:    #1006–#1016, plus #884 marked closed

---

## 1. What this lane did, in one paragraph

FUNCTION BYTE MATCH graded a function's `.text` COMDAT **bytes** and not its
**relocations**, so two bodies branching to two different functions were
byte-identical to it. Board #884 counted the exposure at 4,664 credited
functions. This lane grades them: **861 name the wrong function.** `fnbyte-exact`
**shrank** 35,986 → 35,125 and FBM fell 0.20108 → **0.19627**. That is the
instrument-widening motion, declared in the prereg before it was measured.
`mismatch` is 0 at both ends — `IlBundle::functions()` refuses every TU carrying
one of the 861, so no obj shipped wrong. What is wrong is the census's claim.

---

## 2. The state you can rely on

| | |
|---|---|
| gate | **18/18 PASS, 0 mismatch** at every base this lane had. `work/w-relo/gate_tip.txt` |
| tests | **1,058 passed / 33 targets / 0 failed** (master `5bef565f` was 1,038/32; +9 unit, +8 c2-obj, +3 integration) |
| scan | 878 TUs, `match 10, mismatch 0, capture-fail 7`. Both ends taken on one pinned workload snapshot `a44b1cf9406e (clean)` |
| the 723 | `w-splice`'s spliced bodies re-graded by THIS instrument: **723/723 clean, 0 RelocDiffers, 0 unreached** (`regrade723.py`) |
| peer keys | `peerkeys.py`: **0 families vanished**, every peer total identical at both ends |
| `status.sh --check` | PASS, 23 metrics |
| `board_audit.sh` | 0 duplicate rows, 0 unresolved anchors |

**Board collisions: NONE.** Re-checked against master `91a47df5` — master tops
out at #1005 (w-inread), w-inl0 took #990–#995, this lane holds **#1006–#1016**.
The rows were originally written as #996–#1005 and renumbered mid-lane when
master's `b231974` gave that range to w-inread.

---

## 3. Where everything lives

| what | where |
|---|---|
| the rung | `docs/rungs/2026-08-08-w-relo.md` — **read §4.3 first**, it is the most important part |
| the rule | `docs/FUNCTION_BYTE_MATCH.md` §2.2 (rule) and §6.3 (the reading) |
| prereg | `work/w-relo/PREREG.md`, committed at `8d031784` **before the compare existed** |
| reference reader | `crates/c2-obj/src/reloc.rs` — `ObjImage::text_comdat_relocs`, `RelocTarget`, `CodeReloc`; `symbol_targets` in `lib.rs` |
| port plan | `crates/c2-core/src/comdat.rs` — `text_reloc_plan`, `TextReloc`, `PlanTarget` |
| the compare | `crates/c2-harness/src/gap/fnbytes.rs` — `compare_relocs`, `FnByte::{RelocDiffers, RelocUnknown}`, `RelocKind`, `callee_chain` |
| known-answer tests | `crates/c2-harness/tests/reloc_identity.rs` (3 tests, GRID-S `s12`) |
| evidence | `work/w-relo/{base,tip}_{scan,metrics}.txt`, `gate_tip.txt`, `tests_tip.summary.txt` — all scrubbed of absolute paths |

---

## 4. The comparison rule, as shipped

For a function whose **bytes are already exact**, the port's plan and the
reference COMDAT's records are compared **as sequences**: same length, same
offset, same **whole packed 16-bit type word** (flags included — `REL24|BRTAKEN`
is not `REL24`), same target. Targets are **symbol names through the symbol
table, never indices**, as three typed variants so `Section(".rdata")` can never
equal `Symbol(".rdata")`, and a `PAIR`'s index field is compared as the
**displacement** it actually is (rev 6.0).

**Refuses:** a body whose bytes already differ is not re-graded. An undecodable
table, an out-of-range index, or an index landing on an **aux record** fails the
whole obj closed into `fnbyte-reloc-unknown` — ungraded, never credited.

**ONE LOCATOR.** The `/Gy` writer in `coff/writer.rs` now **calls**
`comdat::text_reloc_plan` rather than building its own record list; only the
name→symbol-index resolution stays in the writer. Board #880's rule one field
along, and the gate is what proves the lift moved no byte.

---

## 5. The numbers (workload `a44b1cf9`, master `5bef565f`, denominator 178,977)

```
exact          35,986     reloc-differs      861      reloc-unknown    0
differs         2,334     partial              0      refused    130,579
unbound         9,217                                 = 178,977

FBM 0.20589 -> 0.20108     exact-bytes 36,847 (the OLD exact, recovered exactly)
reloc-graded 36,847        reloc-graded-relocated 4,664 (= #884's exposure)
exact-relocated 4,664 -> 3,803 (retired from a blind spot into a graded number)
```

**All 861 are a TARGET disagreement** — `-count`, `-offset`, `-type` and
`-section-target` are all **0**. The port emits the right *number* of
relocations, at the right *offsets*, of the right *types*, naming the wrong
*function*.

Families: `tail·local→local·blocked` **529** · `unrelated` **169** · `chain2`
**73** · `chain1` **69** · `seq·local→extern·chain1` **16** · 5 `comdat-only`.
`blocked` priced by production: `expr-call-in-expr-recv-field-off0-then-chain-bind-whole`
**349** · `…-intrinsic-this-adjust-then-chain-bind-whole` **103** ·
`expr-ternary` **50** · two more.

Controls, all **0**: `partition-broken`, `reloc-partition-broken`,
`match-tu-differs`, `match-tu-reloc-differs` (the five-alarm),
`reloc-table-unreadable`, `reloc-index-desync`, `census-disagree`.

---

## 6. TWO THINGS THE NEXT PERSON MUST NOT LOSE

### 6.1 The 861 is independently replicated — and it is now a per-function control

Peer lane **w-drop3** reached **861** on the same corpus from a *different*
reader (board #986): `REL24` targets by name via
`ObjImage::text_comdat_call_targets`, port side from
`comdat_function_body().calls`. This lane's `compare_relocs` asks *every* record
via `ObjImage::text_comdat_relocs`, port side from `comdat::text_reloc_plan`.

Two equal totals are **not** evidence that two readers agree — they are
consistent with each finding 861 *different* functions. So it is published per
function and in a direction:

```
fnbyte-reloc-vs-calltarget-both            861
fnbyte-reloc-vs-calltarget-calltarget-only   0   <- known answer 0
fnbyte-reloc-vs-calltarget-reloc-only        0   <- measured, not predicted
```

### 6.2 The merge nearly ERASED w-drop3's finding, silently

w-drop3's walk guards on `FnByte::Exact` and buckets on it. When it was written
that meant *"the bytes are c2's"*. **This lane narrowed `FnByte::Exact` to "the
bytes AND the relocations are c2's"** — so every function that key exists to
report now lands in `RelocDiffers`, and left alone
`fnbyte-calltarget-disagree-exact` prints **0**. No test red, no conflict marker:
the two lanes never touch the same line.

Both sites read `bytes_exact()` now, which is what that walk always meant, and
w-drop3's eight `fnbyte-calltarget-*` keys are **unchanged to the digit**,
checked by `diff`.

> **The generalization, and the thing to carry forward:** *when a lane narrows
> the meaning of a shared predicate, every existing reader of that predicate is a
> candidate erasure, and `git` will not say so.* What caught it here was reading
> w-drop3's walk while resolving an unrelated conflict twenty lines away.

**If you rebase this branch onto a master that has changed `fnbytes.rs` again,
re-run that check**: grep for `FnByte::Exact` and ask of each site whether it
means *bytes* or *bytes and relocations*.

---

## 7. Prereg scorecard — 9 of 10 won

| # | claim | outcome |
|---|---|---|
| P1 | `s12`'s `?f` moves `exact → reloc-differs` | **WON** |
| P2 | `?g` and `?anchor` stay `exact` | **WON** |
| P3 | `reloc-unknown` = 0, `reloc-table-unreadable` = 0 | **WON** |
| **P4** | count in 30…900, point estimate **300** | **interval won, point LOST — 861, 2.9×** |
| P5 | every reloc-differ is a *target* disagreement | **WON** (861 · 0 · 0 · 0) |
| P6 | `fnbyte-exact-bytes` recovers the old count | **WON** |
| P7 | `match-tu-reloc-differs` = 0 | **WON** |
| P8 | TU match 10, mismatch 0, nothing else moves | **WON** |
| P9 | no section-symbol targets | **WON** |
| P10 | gate and tests green | **WON** |

P4 was low because the estimate assumed a target disagreement needs the port's
callee to be a readable single-branch forwarder; **529 of the 861 have a callee
the parser refuses outright**, a population the reasoning never considered.

---

## 8. WHAT REMAINS

**Owned by the coordinator, not by this lane:**

1. **Rebase onto current master** (`91a47df5` or later). Expect conflicts in
   `crates/c2-harness/src/gap/fnbytes.rs` (every lane touches it) and
   `docs/BOARD.md`. Board numbers are safe — see §2.
2. **Re-gate after the rebase** and re-take both scans back to back on one
   workload snapshot. The dc3 tree moved under this lane once, which is why the
   final numbers were re-taken; do the same rather than diffing across snapshots.
3. **Merge with `--no-ff`**, per the standing convention. Do not squash — the
   prereg-before-measurement commit and the "families first read merged `blocked`
   into `unrelated`" commit are the point.
4. **Do not push** without asking.

**The follow-on work, deliberately NOT done here** (board **#1013**):

The 861 are a work queue, not a repair. The prereg refuses the repair in advance:
*a reloc-differ is a finding, not a repair; the repair is a later rung, priced
from the families this one names.* Every one of the 861 is already accepted by
the **per-function** gate, so a TU-level widening that admits any TU containing
one ships an obj with a branch to the wrong function.

**The interaction that matters most:** board **#968** measured SPLICE-0 holding
on 1,967 of the 3,195 byte-`differs`, 726 with a callee the port already lowers
byte-exactly. An emitter acting on that must get the **relocation** right too —
and this instrument is now the thing that would catch it if it did not. Before
this lane, a splice emitter could have moved 726 functions from `differs` to
`exact` while pointing every one at the wrong symbol, and the scan would have
read it as progress.

---

## 9. Two smaller things recorded honestly

* **This lane's own first read of the families was wrong.** Written as "no path
  found", the walk answered `unrelated` where it could not expand a single edge,
  printing `unrelated 697` and naming nothing — w-seq §2's `refused:blocked`
  defect one lane later on a different axis. It answers `blocked` now and names
  the blocking production. Rung §4.1.
* **`vocab-gap` is 861 and `fnbyte-reloc-differs` is 861 and they are
  UNRELATED** — one counts TUs, the other functions, and `vocab-gap` read 861
  before this compare existed. (w-drop3 flagged the same collision independently
  at `eea3406`.) But `fnbyte-reloc-differs` 861 and
  `fnbyte-calltarget-disagree-exact` 861 **are** the same 861 — that one is §6.1's
  control, not a coincidence. Do not conflate the two situations.
</content>
