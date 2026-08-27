# w-metric evidence — the load-bearing lines of every scan the lane quotes

Each block below is grepped verbatim from the named scan output (full outputs
are worktree scratch; JSONLs alongside). P is computed from each point's JSONL
by `analyze.py`, whose formula the shipped `GapReport::progress_mass` mirrors;
at the tip the two agree to all printed digits.

    point        tree      match  C    f-numerator   P
    pre-w-r1c    68bdbf8   6      84   38455         0.18288
    post-w-r1c   3b00093   8      114  38455         0.19149
    post-w-sect  a4a6ad8   8      169  38458         0.20728
    lane tip     ae692fa   8      169  38458         0.20728   (wibo 1.0.1, warm)
    lane tip     ae692fa+  8      169  38458         0.20728   (wibo 1.2.0, cold re-capture)
    rebased tip  85ede65   8      169  38458         0.20728   (on master d8aa8e3)

The wibo 1.0.1 -> 1.2.0 resync (22:16:37) re-keys the capture cache; the rows
that straddle it are deliberately kept as the robustness control described in
docs/PROGRESS_METRIC.md §5.

== gap-68bdbf8 ==
  c2-rs      68bdbf835e4b (clean)  <repo>/.claude/worktrees/wt-w-metric-backfill
  binary     4289a8d25daedb58  <repo>/.claude/worktrees/wt-w-metric-backfill/target/release/c2rs
  workload   fe1b5b393411 (clean)  <home>/code/milohax/dc3-decomp
  wibo       wibo 1.2.0-27-geab90f0 (Linux x86_64)  <home>/code/milohax/wibo/build/release/wibo
GAP REPORT (878 TUs in 110.1s)
  match             6    0.7%
  mismatch          0    0.0%
  capture-fail      7    0.8%
  EMITTED CENSUS (§8): 38455/178975 emitted functions in class (21.49%)
  A  emit set reachable   `.ex` segments == obj `.text` COMDATs      28  (gate-anchored `4F 1F`; 27 on the census's `4C 4F 11` anchor)
  B  binding complete     every emitted symbol binds                338
  C  section shape        obj sections subset of the writer's 6      84

== gap-3b00093 ==
  c2-rs      3b000936a6ad (clean)  <repo>/.claude/worktrees/wt-w-metric-backfill
  binary     3c2293639e2f6f08  <repo>/.claude/worktrees/wt-w-metric-backfill/target/release/c2rs
  workload   fe1b5b393411 (clean)  <home>/code/milohax/dc3-decomp
  wibo       wibo 1.2.0-27-geab90f0 (Linux x86_64)  <home>/code/milohax/wibo/build/release/wibo
GAP REPORT (878 TUs in 5.0s)
  match             8    0.9%
  mismatch          0    0.0%
  capture-fail      7    0.8%
  EMITTED CENSUS (§8): 38455/178975 emitted functions in class (21.49%)
  A  emit set reachable   `.ex` segments == obj `.text` COMDATs      28  (gate-anchored `4F 1F`; 27 on the census's `4C 4F 11` anchor)
  B  binding complete     every emitted symbol binds                338
  C  section shape        obj sections subset of the writer's 9     114

== gap-a4a6ad8 ==
  c2-rs      a4a6ad8737b4 (clean)  <repo>/.claude/worktrees/wt-w-metric-backfill
  binary     a45a1b11515e4265  <repo>/.claude/worktrees/wt-w-metric-backfill/target/release/c2rs
  workload   fe1b5b393411 (clean)  <home>/code/milohax/dc3-decomp
  wibo       wibo 1.2.0-27-geab90f0 (Linux x86_64)  <home>/code/milohax/wibo/build/release/wibo
GAP REPORT (878 TUs in 115.2s)
  match             8    0.9%
  mismatch          0    0.0%
  capture-fail      7    0.8%
  EMITTED CENSUS (§8): 38458/178975 emitted functions in class (21.49%)
  A  emit set reachable   `.ex` segments == obj `.text` COMDATs      28  (gate-anchored `4F 1F`; 27 on the census's `4C 4F 11` anchor)
  B  binding complete     every emitted symbol binds                338
  C  section shape        obj sections subset of the writer's 10     169

== gap-tip ==
  c2-rs      ae692fa839c5 (clean)  <repo>/.claude/worktrees/wt-w-metric
  binary     650b092b2559ba17  <repo>/.claude/worktrees/wt-w-metric/target/release/c2rs
  workload   fe1b5b393411 (clean)  <home>/code/milohax/dc3-decomp
  wibo       wibo 1.0.1-23-g4a9dd6f (Linux x86_64)  <home>/code/milohax/wibo/build/release/wibo
GAP REPORT (878 TUs in 5.2s)
  match             8    0.9%
  mismatch          0    0.0%
  capture-fail      7    0.8%
  EMITTED CENSUS (§8): 38458/178975 emitted functions in class (21.49%)
  A  emit set reachable   `.ex` segments == obj `.text` COMDATs      28  (gate-anchored `4F 1F`; 27 on the census's `4C 4F 11` anchor)
  B  binding complete     every emitted symbol binds                338
  C  section shape        obj sections subset of the writer's 10     169

== gap-tip2 ==
  c2-rs      ae692fa839c5 (DIRTY)  <repo>/.claude/worktrees/wt-w-metric
  binary     ebc3d1d557ba5ee8  <repo>/.claude/worktrees/wt-w-metric/target/release/c2rs
  workload   fe1b5b393411 (clean)  <home>/code/milohax/dc3-decomp
  wibo       wibo 1.2.0-27-geab90f0 (Linux x86_64)  <home>/code/milohax/wibo/build/release/wibo
GAP REPORT (878 TUs in 4.3s)
  match             8    0.9%
  mismatch          0    0.0%
  capture-fail      7    0.8%
  EMITTED CENSUS (§8): 38458/178975 emitted functions in class (21.49%)
  A  emit set reachable   `.ex` segments == obj `.text` COMDATs      28  (gate-anchored `4F 1F`; 27 on the census's `4C 4F 11` anchor)
  B  binding complete     every emitted symbol binds                338
  C  section shape        obj sections subset of the writer's 10     169
    gap-metric progress-mass 0.20728

== gap-rebased ==
  c2-rs      85ede65eefc4 (clean)  <repo>/.claude/worktrees/wt-w-metric
  binary     c6e8642ae95b3486  <repo>/.claude/worktrees/wt-w-metric/target/release/c2rs
  workload   fe1b5b393411 (clean)  <home>/code/milohax/dc3-decomp
  wibo       wibo 1.2.0-27-geab90f0 (Linux x86_64)  <home>/code/milohax/wibo/build/release/wibo
GAP REPORT (878 TUs in 6.2s)
  match             8    0.9%
  mismatch          0    0.0%
  capture-fail      7    0.8%
  EMITTED CENSUS (§8): 38458/178975 emitted functions in class (21.49%)
  A  emit set reachable   `.ex` segments == obj `.text` COMDATs      28  (gate-anchored `4F 1F`; 27 on the census's `4C 4F 11` anchor)
  B  binding complete     every emitted symbol binds                338
  C  section shape        obj sections subset of the writer's 10     169
    gap-metric progress-mass 0.20728

