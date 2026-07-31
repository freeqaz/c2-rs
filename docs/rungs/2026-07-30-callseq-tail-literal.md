# W30 — the call-tail literal

    Tag:       W30
    Slug:      callseq-tail-literal
    Date:      2026-07-30
    Fixtures:  w30_callseq_tail_intlike.cpp w30_callseq_tail_intlike_neg.cpp
    Census:    474,103 → 481,876 (19.25 % → 19.57 %) for W30 + Class B together
    Record:    docs/ROADMAP.md §6l

One rule with three implementations, two of them narrower — the shape that
`shapes/calls.rs` now holds as the single copy (`GAPS.md` §6 instance #9). The
handoff's guess about `callseq-tail-lit` was **wrong**, which is why the rung
was sized before it was built.

Landed alongside Class B (values live across calls), whose liveness rule was
closed **by refutation**: the prediction failed first and a mis-emit followed
it, and the class is now stated by what it admits *and* refuses, by name. That
work is the serial spine's, and lives in `c2-core/src/codegen/calls.rs`.
