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
| `p/probe4.cpp` | the two IL bodies transcribed into `guarded_seq.rs`'s tests |
| `p/probe5.cpp` | **the `/Ox` tail-duplication threshold** — join length 1/2/3 against a one-armed control. The cell that took the `else` arm back out |
| `alarm/case.cpp` | board **#232**, the wrong emit: `??1M` in its own `.text` COMDAT and the port packing it |
| `alarm/explicit.cpp` | the separating cell — an EXPLICIT `M::~M()` is `00`-introduced, and its `.gl` has no `0x26` byte at all. Still mismatches, at a different offset: a **pre-existing packed-`.text` ordering defect**, proposed as board row X-d |
| `alarm/count.py` | how many reference objs carry more than one `.text` in packed mode (twelve, and the COMDAT is not always first) |
| `cov_sweep.sh` | builds the two coverage profiles `work/w-frame/sweep.py` reads |
| `SWEEP.txt` | its output on this tip, after the one production it found was closed |

Objs and IL captures are **not** committed (`CLAUDE.md`: no `_CL_*`, no `*.obj`).
Reproduce with `p/mk.sh <probe>.cpp` and `scripts/gt_dump.py`.
