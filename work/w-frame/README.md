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
