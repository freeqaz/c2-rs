# Lane w-cross — the framed × branching cell

`work/w-frame/RANKING.md` §4 measured the port's one un-witnessed cross-product
cell: **105 functions emitted byte-exact, 28 framed, 2 branching, ZERO both**,
with 10 of the 17 FRONTIER TUs needing the product. This lane builds it.

| file | what it is |
|---|---|
| `PREREG.md` | the pre-registered estimate, committed before any `crates/` or `fixtures/` edit |
| `p/mk.sh` | compile a probe `.cpp` at the WORKLOAD's own flags (never `c2rs compile` — board #195) |
| `p/probe1.cpp` | is the framed × branching cell real, and how small is its minimum? |
| `p/probe2.cpp` | where the guarded call's setup goes; the intra-section `b`; the r11 local; the band-2 and tail-merge neighbours |
| `p/probe3.cpp` | the label-counter stride vs branch count (5 cells, shape held fixed); the park register; saved GPRs beside a branch |

Objs and IL captures are **not** committed (`CLAUDE.md`: no `_CL_*`, no `*.obj`).
Reproduce with `p/mk.sh <probe>.cpp` and `scripts/gt_dump.py`.
