#!/usr/bin/env python3
"""Insert w-gate's board rows (#1077-#1086) at their numeric positions.

Tooling, outside the std-only workspace. It FAILS HARD rather than skipping:
board ranges have collided repeatedly on this project, so this refuses if any
number it is about to mint already exists, refuses if the allotted range is
exceeded, and refuses if an anchor row is not found exactly once.
"""
import re
import sys
import pathlib

ALLOTTED = range(1077, 1087)          # #1077-#1086 inclusive
board = pathlib.Path(__file__).resolve().parents[2] / "docs" / "BOARD.md"
text = board.read_text()

DONE_ANCHOR = "| **1066**<sub>w-root</sub> |"
OPEN_ANCHOR = "| **299** |"

# Done table: | # | item | number | where settled |
DONE_ROWS = [
    (1077,
     "**THE MERGE GATE COULD REPORT A PASS OVER A RUN THAT GRADED NOTHING, AND FOUR LANES IN FIVE DAYS HIT IT**",
     "`gate.sh` printed `GATE: SKIPPED — NOTHING WAS GRADED`, in capitals, over an explicit *\"this exits 0 by design and is NOT a green gate\"* — and **exited 0**. A shell caller's only cheap machine-readable output is `$?`, and `$?` said 0. `w-mod` and `w-bc` (2026-08-04, board **#299**), `w-subclass` (08-05, rung §10.1), `w-root` (08-08)",
     "**DONE.** The thirteenth recorded instance of *absence read as success* by `gate.sh`'s own counter. **Both prior payments bought LEGIBILITY, and legibility was not the binding constraint** — the run was already loud; nobody read it. **#299** filed the defect AND the fix direction (*\"not to make absence fail … but to make the two cases DISTINGUISHABLE\"*) on day one and stood open five days, which is the same shape `w-root` itself recorded one rung earlier. rungs/2026-08-08-w-gate.md §2, §3.1"),
    (1078,
     "**`--require-graded` — the CALLER states its expectation, and the check is a COUNT summed over the whole gate**",
     "`--require-graded` / `C2RS_GATE_REQUIRE_GRADED=1`. Quantity = **lane fixture-verdicts + sweep cases graded + cross cells graded**; `== 0` under the demand is `GATE: FAIL (NOTHING GRADED)`, **exit 1**. The banner also prints *lanes that graded a corpus, k of n*, because the first number is a sum one busy lane can carry",
     "**DONE.** `graded`, never `checked` — w-modes found 96 of 14,635 sweep cases sitting in that gap. **`SKIPPED` still exits 0 by default and the documented exit-code contract does not move**: the portable lane has no toolchain by design and CLAUDE.md requires it to degrade cleanly, so making absence hard-fail would trade one silent failure for a noisy one in the only lane entitled to be empty. rungs/2026-08-08-w-gate.md §4; `scripts/gate.sh` header + `decide`"),
    (1079,
     "**THE CHECK SITS AT THE LAST POINT WHERE EVERY REMAINING OUTCOME EXITS 0 — that placement IS the non-enumeration**",
     "One site, not four. Above it: completeness, the generated instruments' FAIL and NO-RESULT, the lanes' NO-RESULT, the lanes' FAIL — all `return 1`. Below it: `SKIPPED`, `PASS (LANES FILTERED)`, `PASS (SAMPLED)`, `PASS` — all `return 0`",
     "**DONE, and this is the generalizing part.** The standing rule is `scripts/mode_lane.sh`'s vacuity guard, quoted verbatim into the header: *\"never as an enumeration of the ways a run can come back empty, because the next empty run will be empty in a way nobody enumerated.\"* A list of today's empty outcomes is blind to tomorrow's; **one count at the last zero-exit point covers a zero-exit outcome that does not exist yet.** The comment at the site says a `return 1` added below it invalidates the comment — move the block, do not copy it. rungs/2026-08-08-w-gate.md §4"),
    (1080,
     "**SAMPLED AND `--lane`-FILTERED RUNS SATISFY THE DEMAND, AND A PARTIAL SKIP IS NOT COVERED — both deliberate**",
     "A strided 400-case sample graded 400 things. The demand is `graded > 0`, not `graded == corpus`; both outcomes already refuse an unqualified PASS and name what they did not establish. A partial skip is **already** a FAIL and is already behind the check",
     "**DONE — a DECISION, asserted rather than assumed.** Conflating *less than everything* with *nothing at all* would make one flag mean two things and leave a lane no way to iterate under the demand. Covering the partial skip again would be a second implementation of a rule that has one — the *one rule, two implementations* shape GAPS §6 keeps recording, and the shape that produced this bug's own ancestor (`mode_lane.sh` shipped without the guard `sweep_mode.sh` had already grown). **A mismatch keeps its alarm under the demand and is never relabelled**; mutation M22 hoists the demand past the lane-FAIL check and reddens exactly those two assertions. rungs/2026-08-08-w-gate.md §5"),
    (1081,
     "**`--reap-only` UNDER THE DEMAND IS A CONTRADICTION AND EXITS 2; the inspection modes SAY the demand does not bind them**",
     "`--reap-only` grades nothing by construction and its documented exit 0 means *the disk is clear*, never *the port is right* — it is the one non-run mode a caller could substitute for a gate and read `$?` of. Refused **before** the run tree is created. `--list`/`--check`/`--selftest` print a note on stderr and carry on",
     "**DONE.** The env form is what needs this: a lane exporting `C2RS_GATE_REQUIRE_GRADED=1` in its `env.sh` would otherwise believe every command it then runs is under the demand. **A demand quietly ignored is this file's own bug class**, so it is never silent — it binds, refuses, or says it does not apply. rungs/2026-08-08-w-gate.md §5"),
    (1082,
     "**A SKIP NOW NAMES THE PATH THAT DID NOT RESOLVE, THE OVERRIDE THAT GOVERNS IT, AND THE ONE COMMAND THAT FIXES IT**",
     "Under every `GATE: SKIPPED` and under the demand banner: the effective `compilers` root, `cl.exe`, `c2.dll` and `wibo` with `found`/`MISSING`, each with `C2RS_COMPILERS` / `C2RS_CL_EXE` / `C2RS_C2_DLL` / `C2RS_WIBO` and whether it is SET; then `scripts/configure_existing_worktree.sh <root>`, then `scripts/fetch_compilers.sh`",
     "**DONE.** **`C2RS_DC3` is deliberately NOT named**, although the w-subclass rung lists it beside the other two: it is `status.sh`'s dc3 SOURCE tree and **no lane in this gate reads it**; naming it sends a reader to set a variable that cannot fix this, and a selftest case keeps it out. The block **decides nothing** — `Toolchain::locate()` is the authority, the version dir is `sed`-ed out of `crates/c2-reference/src/lib.rs` at run time rather than copied, and a selftest case extracts every `C2RS_*` the hint prints and requires the resolver to still read it. **And it refuses to lie when everything is present**: four `found` lines under a heading saying the toolchain did not resolve would be a signpost contradicting the run it explains. House style from `scripts/status.sh`'s `NO-RESULT (dc3 tree absent at $dc3 — set C2RS_DC3)`. rungs/2026-08-08-w-gate.md §6"),
    (1083,
     "**THREE OF THIS LANE'S OWN NEW CHECKS COULD NOT FAIL, AND ONLY THE MUTATIONS SHOWED IT**",
     "(a) with `C2RS_COMPILERS` set, the hint printed the **default** `cl.exe`/`c2.dll` and reported **`found`** for two files `Toolchain::locate` never opens — an override is taken verbatim and does not fall back. (b) `hint-names-every-override` grepped the **whole block** and stayed **green** under a mutation that deleted the override from the path it belongs to, because the name was still in a prose sentence lower down. (c) the anti-drift arm compared a list **repeated in the selftest** against `crates/c2-reference/src/lib.rs`, so a hint that *invented* a name was invisible — the fixed list went on agreeing with itself",
     "**DONE — all three fixed in-lane.** *A signpost aimed at the wrong road is worse than no signpost*, and *a name in prose is not a name attached to the path that failed*. (a) follows `compilers_root`'s documented precedence, transcribed for **reporting only** and labelled as such; (b) requires `override: <NAME>` on the line under the path; (c) extracts the names from the hint's own output, so M13 renames one override and reddens (b) and (c) together. **None was found by reading the code; all three were found by trying to break it** — the same lesson `parse_registry` taught this gate on its first day: **a component built to make absences visible will still have absences of its own.** rungs/2026-08-08-w-gate.md §7"),
    (1084,
     "**22 MUTATIONS, 32 OF 32 NEW ASSERTIONS SEEN RED INDIVIDUALLY, AND A CONTROL THAT STAYS GREEN**",
     "`work/w-gate/mutate.sh` — one property per mutation, each against its **own** fake repo root, aborting if a `sed` matches nothing (a mutation that did not apply proves nothing). `work/w-gate/coverage.sh` asserts every new assertion has been in the RED column: **32 checked, 0 never seen red**",
     "**DONE.** Every demand case is shadowed by a control built from **byte-identical fabricated logs**, differing only in the flag. **M1 reddens the demand case and leaves the control green; M3 does the exact opposite** — that independence is what makes the pair a control rather than a duplicate, and *a test that goes red everywhere identifies nothing*. **The case-count floor is checked AFTER every case has run**, and the comment now says why: `crates/c2-harness/tests/lane_registry.rs` has a floor that trips FIRST and GAPS records the consequence — every mutation failed on the count and the assertions behind it never executed. Each assertion carries a **distinct** message so a red names the arm that broke. Selftest **83 → 102 cases**, floor **65 → 84**. rungs/2026-08-08-w-gate.md §8"),
    (1085,
     "**`num()` IS LOAD-BEARING: without it an absent count does not fail the gate, it KILLS it**",
     "`$(( 0 + + ))` is a bash expansion error and a non-interactive shell **exits** on one. Mutation **M9** removes the guard and the selftest dies mid-run with **no verdict line at all** — the only one of the 22 mutations that produces no report",
     "**DONE, and worth its own row because the failure is worse than the one being guarded.** An absent count without `num()` is not a demand that fails; it is **a gate that stops having opinions**, silently, at the exact moment it was asked to have one. `num()` maps absent-or-unparseable to **0**, which is safe *here and only here* because the comparison is `> 0` — an unreadable count can only make the demand FAIL, the exact inverse of the defect this project keeps recording. Generalizes: **anywhere a shell reads a number out of another instrument's output and then does arithmetic on it, the read needs a floor, and the floor's DIRECTION has to be argued.** rungs/2026-08-08-w-gate.md §4"),
]

# Open table: | # | item | worth (measured, not estimated) | defined | notes |
OPEN_ROWS = [
    (1086,
     "**`scripts/status.sh --write` HAS THE SAME HOLE, OVER A COMMITTED ARTIFACT**",
     "**OPEN — small, and the blast radius is larger than #1077's.** It renders `NO-RESULT` for every metric it cannot produce and then **overwrites the generated block in `docs/STATUS.md` with them**; board **#424** records a worktree turning **19 of 23** metrics into `NO-RESULT`. It refuses nothing",
     "rungs/2026-08-08-w-gate.md §10; board **#424**; `scripts/status.sh` `collect_gap`",
     "A `--require-complete` there is the same shape as `--require-graded`: a positive check on a count (metrics that produced a value, of the 23 registered), opt-in, placed where the run would otherwise exit 0. **The difference is that `gate.sh` only misreports an EXIT CODE, while `status.sh --write` COMMITS the misreport** — a hand-quotable table of `NO-RESULT` replacing good numbers in the file CLAUDE.md names as *the one-page answer to \"where is this project\"*. **This is the row the next lane on this seam should read first**"),
]


def refuse(msg):
    print(f"board_rows.py: REFUSING — {msg}", file=sys.stderr)
    sys.exit(1)


minted = [r[0] for r in DONE_ROWS] + [r[0] for r in OPEN_ROWS]
if len(set(minted)) != len(minted):
    refuse(f"duplicate number in the mint list: {minted}")
for n in minted:
    if n not in ALLOTTED:
        refuse(f"#{n} is outside this lane's allotted range "
               f"#{ALLOTTED.start}-#{ALLOTTED.stop - 1}")
    if re.search(rf"^\| \*\*{n}\*\*", text, re.M):
        refuse(f"#{n} already exists in docs/BOARD.md — ranges have collided here before")

existing = sorted(int(m) for m in re.findall(r"^\| \*?\*?(\d{2,4})", text, re.M))
if not existing:
    refuse("could not parse a single row number out of docs/BOARD.md")
print(f"board_rows.py: highest existing row is #{existing[-1]}; "
      f"minting #{minted[0]}-#{minted[-1]}")

plan = [
    (DONE_ANCHOR, "".join(
        f"| **{n}**<sub>w-gate</sub> | {item} | {number} | {where} |\n"
        for (n, item, number, where) in DONE_ROWS)),
    (OPEN_ANCHOR, "".join(
        f"| **{n}**<sub>w-gate</sub> | {item} | {worth} | {defined} | {notes} |\n"
        for (n, item, worth, defined, notes) in OPEN_ROWS)),
]

for anchor, block in plan:
    lines = text.splitlines(keepends=True)
    hits = [i for i, ln in enumerate(lines) if ln.startswith(anchor)]
    if len(hits) != 1:
        refuse(f"anchor {anchor!r} matched {len(hits)} lines, expected exactly 1")
    lines.insert(hits[0] + 1, block)
    text = "".join(lines)

board.write_text(text)
print(f"board_rows.py: inserted {len(DONE_ROWS)} Done row(s) and {len(OPEN_ROWS)} Open row(s)")
