# Lane w-frame — the FRONTIER's feature-union ranking

Measurement tooling and results. Nothing here is linked into the port; it sits
outside the std-only Rust workspace on the same footing as
`scripts/plot_perf.py` and `scripts/gt_dump.py`.

| file | what |
|---|---|
| [`RANKING.md`](RANKING.md) | **the deliverable** — the 17 FRONTIER TUs ranked by distinct unmodeled constructs, the method, the controls, and the frame × branch finding |
| `featmap.py` | the classifier and the ranking driver (`featmap.py rank`) |
| `analyse.py` | key agreement (Spearman), the frame × branch product test, A4/A5 scoring |
| `modectl.py` | the `/Ox` vs workload-profile control on `port_vocab` |
| `refobj.sh` | one real reference obj at the workload's own flags |
| `rank.json` | the ranking's raw output |
| `match_fixtures.txt` | the 102 fixtures `c2rs perf` grades `Port=Match` |
| `dc3_head_before.txt` | workload provenance, recorded before the first scan |

**Not committed** (regenerable, and one carries an absolute path): `gap_base.txt`
(a `c2rs gap` run — its provenance line prints the resolved dc3 path) and
`obj/` (real `.obj` output, gitignored).

Regenerate:

```sh
c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt \
         --cwd ../dc3-decomp --jobs 16 > work/w-frame/gap_base.txt
c2rs perf | awk '$NF=="Match"{print $1}'  > work/w-frame/match_fixtures.txt
work/w-frame/featmap.py rank work/w-frame/gap_base.txt > work/w-frame/rank.json
work/w-frame/analyse.py
work/w-frame/modectl.py
```

## The sweep — which of the port's productions has the oracle never seen?

Added at the funnel's request after the `bt`/`cmpwi` finding. `sweep.py` builds
two coverage profiles and subtracts them; see its module docstring for the
method and `docs/rungs/2026-08-04-w-frame.md` §4.5 for the result.

| file | what |
|---|---|
| `sweep.py` | the two-profile coverage differ over `crates/c2-core/src/codegen/` |
| `SWEEP.txt` | its output at the branch tip |
| `cov/`, `cov2/` | the GRADED and REACHED profiles (`.profraw`/`.profdata`/`export.json`, all gitignored — regenerate) |

```sh
RUSTFLAGS="-C instrument-coverage" cargo build --release -p c2-harness --bin c2rs \
    --target-dir target-cov
# GRADED: only 100%-match runs — perf over the Port=Match fixtures, each gate
# lane restricted to ITS OWN match list, and the 8 matching workload TUs
# REACHED: c2rs gap over every fixture at every lane
work/w-frame/sweep.py
```

**The profiles must not be built by hand from a remembered list of runs.** Both
of this instrument's own errors were exactly that: the first GRADED profile
omitted the workload TUs and falsely accused `dyninit_thunk_text`; the second
omitted the gate's per-lane runs and falsely accused 24 `/O1` register-allocation
regions. Each correction shrank the band, and each is a measurement of the
instrument rather than of the port.
