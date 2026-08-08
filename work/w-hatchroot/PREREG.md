# w-hatchroot — PREREGISTRATION

Written and committed **before a line of either item was implemented**, at base
`85e180d4`. Two items, both apparatus, neither touching `crates/`.

The house rule this file exists for: a prediction registered after the
measurement is not a prediction. Each row below says what I expect **and the
direction I expect to be wrong in**, because the direction is the part that is
falsifiable when the number lands somewhere else.

---

## ITEM 1 — `hatch.py` resolves `ROOT` from its own file path (board #1460)

`ROOT = dirname³(os.path.abspath(__file__))`, so `python3
../../../work/w-front3/hatch.py apply` from a worktree edits the **main**
repository. It fired this session; `w-5c2` reverted it and the tell was
`sha256sum` (the "hatched" binary was byte-identical to the unhatched one).

### What I expect

| # | prediction | direction I expect to be wrong in |
|---|---|---|
| 1.1 | The fix is a **cwd-derived** root plus a **containment check**, and the containment check is what actually has teeth: the cwd's `git rev-parse --show-toplevel` must equal the *script directory's* `git rev-parse --show-toplevel`. | **Too weak.** My first instinct was a lexical `commonpath(root, script)` test, and that is wrong in one direction I can name in advance: this repo's worktrees live at `<main>/.claude/worktrees/…`, i.e. **inside the main repo's path**, so a worktree's `hatch.py` invoked from the main repo passes a lexical containment test and edits main with the worktree's `EDITS`. Two `git rev-parse` calls do not have that hole. If I am wrong it will be because a *third* arrangement (a submodule, a bare/`--separate-git-dir` checkout) satisfies both `rev-parse`s and still crosses trees. |
| 1.2 | Two new refusal words, both leading their own line: `HATCH-FOREIGN-ROOT` (cwd is a checkout, the script belongs to a different one) and `HATCH-NOREPO` (cwd is not in a checkout at all). Exit 6 and 7. | **Ordering.** `HATCH-NOREPO` is checked first, so an arm that fabricates "a script somewhere else" by pointing at a non-repo directory will fire `NOREPO` and its `FOREIGN-ROOT` assertion will pass on the wrong refusal — trap A, in the file that documents trap A. I expect to have to build a **real second git repository** for the `FOREIGN-ROOT` arm rather than a bare temp dir. |
| 1.3 | The positive announcement (`hatch: TARGET REPOSITORY …`) costs nothing and breaks nothing, because `gate.sh`'s classifier keys on `^ALL … ARMS PASS`, `^FAILED: `, `^REFUSING to run on a dirty tree` and `SETUP FAILED`, none of which a line beginning `hatch:` can collide with. | **A collision I have not enumerated.** The classifier greps `SETUP FAILED` **unanchored**, so any new text containing that string anywhere would turn a green run into `REFUSED HATCH-STALE`. I will keep the announcement free of those four shapes and check the real log rather than assume. |
| 1.4 | `hatch_red.py` gains **3 arms** — 2 red (`HATCH-FOREIGN-ROOT`, `HATCH-NOREPO`) and 1 green (root resolves from a **subdirectory** of the repo, which is the ordinary invocation and the thing a containment check is most likely to break). 11 arms → **14**; 9 red → **11**; 2 green → **3**; 8 distinct words → **10**. | **The green control is the one that will fail first**, not the reds. A guard written to refuse cross-tree invocation that also refuses `cd crates && python3 ../work/w-front3/hatch.py check` is a guard that will be deleted by the next lane, and that failure would be *mine*, not the tool's. |
| 1.5 | `hatch_red.py` has **the same defect** — its `restore()` is `git checkout -- crates/` in a `__file__`-derived `ROOT`, i.e. #1380's destruction with #1460's aim — and it gets the same resolution with its **own** word (`HATCHRED-FOREIGN-ROOT`), kept out of `ALL_WORDS` so it can never satisfy an arm's expectation. | **Scope.** I expect to be told this is one file too many. I am taking it because the red harness is *strictly more* destructive than the thing it tests: `hatch.py revert` refuses on a dirty tree, `hatch_red.py restore()` does not. |
| 1.6 | `work/w-front3/ladder.py` and `work/w-ladders/ladder_red.py` carry the same `__file__`-derived `ROOT`. Both only **read** (`EXPR_RS`), so the failure is a wrong width table rather than a destroyed tree. **I expect to decline these** and file them rather than fix them. | **Wrong to decline.** A cross-tree read of `expr.rs` produces a *plausible wrong answer* — the exact failure class this lane exists to close — and "it only reads" may not be much of a defence. If I decline it, the residue is named in the rung and gets a board row rather than being left silent. |

### What would make me abandon ITEM 1

A `git rev-parse --show-toplevel` that is wrong or slow enough to matter inside
`hatch_red.py`'s 14 module loads (28 `git` invocations). I expect this to be
noise against a run that already shells out to `git` per arm; if it is not, the
resolution caches.

---

## ITEM 2 — `ladder_red.py` is not a gate row (board #1406, second half)

`work/w-ladders/ladder_red.py`: 5 arms, 3 red 2 green, **5 of 5 fail** against
the pre-lane `ladder.py`. Run by hand; nothing in CI touches it.

### What I expect

| # | prediction | direction I expect to be wrong in |
|---|---|---|
| 2.1 | The row is a **copy of the hatch-red row's shape** — a pure classifier `ladder_red_verdict()` (log + status + declared arm count → `verdict\|pass\|total\|red\|green\|detail`), a `ladder_red_run()` that carries the interlock and the postcondition, one printed row in `decide`'s table, and a headline qualifier. The arm count is read from `ladder_red.py --list`, never written into the gate. | **The copy will be too literal.** `hatch_red.py` writes into `crates/` and `ladder_red.py` does not — it repoints `EXPR_RS` at a temp file under `work/`. So hatch-red's `crates/`-wide interlock is the wrong shape here; the right one is narrow, over the **one** tree file the arms read (`crates/c2-il/src/func/body/expr.rs`). If I copy the wide interlock I get a row that `REFUSED`s for reasons that have nothing to do with it. |
| 2.2 | `decide()` grows an **8th positional argument**. Every existing `decide` call in `--selftest` (11 of them) must gain the new tuple or "absent is a failure" reddens them all. | **This is where I expect to break something.** Eleven mechanical edits to a 3,231-line gate that two peers are running. The mitigation is that `--selftest` is a count-and-assert harness that runs in seconds, so a missed call site is a red case and not a silent pass — but a *missed* call site that happens to be inside a case whose expectation is `FAIL` would pass for the wrong reason. I will check the headline strings, not just the verdicts. |
| 2.3 | Distinct leading word per refusal, none colliding with hatch-red's eight: `LADDER-NO-LOG`, `LADDER-DIRTY`, `LADDER-MISSING`, `LADDER-NOGIT`, `LADDER-TRUNCATED`, `LADDER-VACUOUS`, `LADDER-EXIT`, `LADDER-ARMS-FAILED`, `LADDER-RESIDUE`, `LADDER-UNRECOGNIZED` — **10**. | **Two of them will turn out unreachable rather than merely unfired**, which is a different defect from a guard that does not fire: `LADDER-RESIDUE` asserts the arms left `crates/` untouched, and the arms have no code path that writes there. I will say so explicitly rather than counting it as covered — it is a postcondition against a **SIGKILL**, like hatch-red's, not against a code path. |
| 2.4 | `REFUSED` exits 0 and forfeits the unqualified headline: `GATE: PASS (LADDER-RED REFUSED)`. Both rows refusing prints **both** suffixes. | **Ugly rather than wrong.** I expect `GATE: PASS (HATCH-RED REFUSED) (LADDER-RED REFUSED)` and I am choosing that over a merged suffix, because a merged one cannot say *which* row refused. |
| 2.5 | The master-gate diff (master's `gate.sh` against this tree, same pinned binary) is **exactly one line**: the new `ladder-red` row. | **The hatch-red row's numbers move too, and that would make it two lines.** They should not: master's `hatch_red_run` reads the arm count from `hatch_red.py --list`, so master's gate run against *this* tree will print `14/14 11` for the same reason mine does. **If the diff is two lines I was wrong about where the count lives**, and that is worth more than the prediction being right. |
| 2.6 | The 18 lanes' counts, the sweep's, the cross's: **unchanged, digit for digit.** | If any of them moves, this lane caused it, because nothing in `crates/` is touched — the base and tip binaries are the same bytes. There is no benign reading. |

### What would make me abandon ITEM 2

A `ladder_red.py` run that is not deterministic against a clean tree, or one
that takes long enough to matter in front of `pin_harness`. Both are checked
before the row is wired, not after.

---

## Shared predictions

| # | prediction | direction I expect to be wrong in |
|---|---|---|
| S.1 | **`crates/` diff empty at the end.** No `.rs` file is opened by this lane. TU match / mismatch / codegen-gap / vocab-gap / capture-fail all unchanged, and the base and tip binaries are **byte-identical**, so this is one measurement quoted twice and will be labelled that way. | The only way this moves is if `hatch_red.py`'s arms crash mid-run and leave the tree hatched. That is what `hatch_red_run`'s postcondition is for; if it fires, the number moved and the lane says so. |
| S.2 | `git grep -c '#[test]'` under `crates/`: **+0**. Workspace target count: **+0**. `--selftest` case count: **rises** (new ladder-red cases), floor raised with it. | If the test count moves, something under `crates/` was edited and this lane is not what it says it is. |
| S.3 | `peerkeys.py` at both ends: **0 families vanished, 0 moved** — same binary, same scan. | A moved family here would mean the scan is not deterministic, which is a finding about the scan and not about this lane. |

---

## The thing I most expect to get wrong

**Not either item — the interaction.** `hatch_red.py` gains arms in the same
commit that `gate.sh` gains a row, and the hatch-red row reads its expected arm
count out of `hatch_red.py --list`. If I edit the `ARMS` table and forget one of
the three new `arm(...)` calls, the declared count and the run count disagree and
the row reads `FAIL TRUNCATED` — which is the guard working. **The failure I
cannot see that way is the mirror**: an `arm(...)` call I add and forget to
declare in `ARMS`, which makes the run *longer* than declared and reads as
`TRUNCATED` too, from the other side. Both are caught. The one that is **not**
caught by anything is an arm whose `want` word I typo into a string no guard
emits: it fails, loudly, but for the wrong reason, and I would fix the guard to
match the typo. So: **every new red is quoted verbatim in the rung, from the run,
and the word in the report is copy-pasted out of the terminal rather than typed.**
