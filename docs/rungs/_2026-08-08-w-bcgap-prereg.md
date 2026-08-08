# w-bcgap — PREREG

    Lane:   w-bcgap, worktree `wt-w-bcgap`, base master `b027eaad`
    Date:   2026-08-08, written and committed BEFORE the first probe
    Brief:  make `|{TU : some per-TU model is exact} ∩ B∧C|` COMPUTABLE.
            `_2026-08-04-w-emitp-findings.md` §5/§8.1 names it as the
            number that turns per-TU exactness into TU reach and declines
            to extrapolate it.

---

## 0. What I have read, and the one thing I already know that the brief does not

I have read `CLAUDE.md`, `docs/STATUS.md`, `_2026-08-04-w-emitp-findings.md`
§5/§7/§8/§12, and `crates/c2-harness/src/gap/factors.rs`. **No measurement has
been run.** Reading source is not a probe; the first probe is §4 below.

**The brief's premise is three days stale and I am registering that before I
measure anything.** It says "there is no per-TU `B∧C` listing". There is:
`GapReport::factor_membership` / `factor_tsv`, shipped as
`c2rs gap --factors-tsv PATH` at commit `cca119ad` by lane `w-reach`, board
**#352** — filed *because* w-emitp declined the multiplication. So the listing
half of my brief is **already done**, and if I "build" it again I will have
shipped a duplicate and reported a success.

What is still missing is the half nobody built: **the intersection itself**, and
specifically its failure mode. A lane holding a candidate set of TU names has to
join it against that TSV by hand, and the join is on a *source path string*. If
the two spellings disagree — `src/App.cpp` vs `src__App.cpp` vs an absolute
path — the join returns **0**, and `|model ∩ B∧C| = 0` reads as "this model buys
no reach" instead of "your key is wrong". That is precisely the class of defect
this project has hit nine times this week (a column zero by construction; 130,575
refusals under a codegen label; `noform-` keys where `None` was overloaded).

So this lane ships an **intersection engine with a loud positive check on the
join**, not a second listing.

## 1. What I will build

1. **One set-algebra module** that defines every published set — `A B C D E`,
   `B∧C`, `A∧B∧C`, `A∧B∧C∧D`, `A∧B∧C∧(D∨E)`, `MATCH`, `FRONTIER`,
   `frontier-if-A`, `projection-divergence` — over a single row type, and is
   used by **both** the live `GapReport` and a parser of the `--factors-tsv`
   file. One definition, two producers; a unit test asserts the two producers
   agree row-for-row, so the offline tool is a *view* of the scan's measurement
   and not a re-implementation of it.
2. **`c2rs factors`** — an offline command over a `--factors-tsv` file:
   * recomputes and prints every count above (the known-answer control), and
   * `--set NAME=PATH` (repeatable): reads a candidate TU-name set and prints
     `|cand ∩ S|` for every `S`, plus the two pricing lines that generalize
     board #213: `|cand ∩ (B∧C \ A∧B∧C)|` (the reach a *partial* emit model
     buys, of which #213's **+124** is the `cand = everything` case) and the
     frontier analogue of **+122**.
3. **The join check, printed first and unconditionally**: candidate names that
   matched a graded row, candidate names that matched **nothing** (with
   examples), and graded rows absent from the candidate file. **A candidate set
   that resolves zero rows is an error exit, not a table of zeros.**

**Cost/gating decision, registered now**: the listing stays exactly where it is
— opt-in, a file, `--factors-tsv` — because `gap` is the engine under
`mode_lane.sh`/`mode_cross.sh` and one line per graded TU is tens of thousands
of lines there (w-reach's reasoning, #352, unchanged). The new command is
**offline over that file**: no toolchain, no capture, milliseconds. So it is
neither "every scan" nor "a flag on the scan" — it is **zero marginal cost on
every scan**, which is the third option and the right one here.

**I will not change any published count.** `git diff` on `GAP-METRICS` keys must
be empty; that is the control, stated in §3.

## 2. The candidate set I will actually intersect

`w-emitp`'s `ALIAS_IN` per-TU exact set — **472 of 850** — regenerated from
`work/w-emitp/scan.py` per that rung's §12. Its `scan.jsonl` was scratch and is
gone; the inputs (`work/w-db/cacheidx.tsv`, `work/w-emit/truth`,
`work/w-joint/truth_data.py`) are present. If regeneration fails I fall back to
a set I can compute in-tree and **say so** rather than reporting an intersection
against a set I could not reproduce.

## 3. Registered expectations — point, then interval

| # | quantity | point | interval |
|---|---|---:|---|
| **K1** | `\|B∧C\|` re-derived from the TSV rows | **151** | exactly 151 |
| **K2** | `\|A∧B∧C\|` re-derived | **27** | exactly 27 |
| **K3** | `\|FRONTIER\|` re-derived | **16** | exactly 16 |
| **K4** | graded rows in the TSV | **871** | exactly 871 |
| **K5** | every other `GAP-METRICS` key, base vs tip | **byte-identical** | 0 differing keys |
| **N1** | candidate names (850) that resolve to a graded row | **850** | [820, 850] |
| **N2** | graded rows absent from the candidate set | **21** | [21, 51] |
| **I1** | `\|ALIAS_IN-exact ∩ B∧C\|` | **100** | [60, 145] |
| **I2** | `\|ALIAS_IN-exact ∩ (B∧C \ A∧B∧C)\|` — the reach bought | **78** | [40, 120] |
| **I3** | `\|ALIAS_IN-exact ∩ FRONTIER\|` | **6** | [0, 16] |
| **I4** | `\|ALIAS_IN-exact ∩ MATCH\|` | **9** | [4, 11] |
| **T1** | TU match at tip | **11** | exactly 11 (instrument lane) |
| **T2** | new unit tests added | **12** | [6, 25] |

**Why I1's point is above the naive product.** `151 × 0.555 = 84` is the
extrapolation w-emitp refused. I expect the true value **above** it, because
factor B (every emitted symbol binds) and "the emit-set model is exact on this
TU" are both easier on small, template-light TUs — positively correlated, so the
joint should exceed the product. Registering the direction means I cannot later
present either outcome as confirmation. **This is my declared bias**: if I1 lands
*below* 84 the correlation runs the other way and I will say the prediction was
backwards rather than reframing it.

**Why I3 is allowed to be 0.** FRONTIER is 16 TUs and requires **A** already, so
a model that replaces A cannot add to it; the intersection is a *check*, not a
gain. If it is 16, the model is exact on every frontier TU and buys none of them.

## 4. Decline clauses

1. **If `|B∧C|` does not fall out at 151, I stop and report the discrepancy
   before touching anything.** I will not tune the set definitions until they
   reproduce. If I ever do adjust one, the adjustment and the number before it
   go in the rung.
2. **If the candidate join resolves fewer than 820 of 850 names**, I do not
   publish I1–I4. A partly-resolved join is the trap, not a result.
3. **If `w-emitp`'s scan cannot be regenerated**, I publish the engine, the
   known-answer control, and I1–I4 as **NOT COMPUTED, and why** — I do not
   substitute a set of my own invention and call it the answer.
4. **Nothing in `crates/c2-il` or `crates/c2-core` is touched** (lane `w-phase7`
   owns them). If my change needs either, I stop and report instead.
5. **`scripts/gate.sh --require-graded` must PASS before I report.** A gate that
   grades 0 is a failure, not a pass.
6. **TU match will stay 11 and that is the expected outcome**, not a
   disappointment to be spun. If it moves I have shipped something outside my
   brief and will say so.
