//! c2.dll pass pipeline — **placeholder module tree**. No pass is ported yet.
//!
//! The real backend runs a fixed pipeline over the loaded IL. Documented order
//! (observed via pass-boundary tracing of the real c2 under wibo):
//!
//! ```text
//!   INIT       backend + target setup, IL bundle open
//!   IL-LOAD    parse the .ex/.gl/.sy/.in/.db bundle into the in-memory IL
//!   OPTIMIZE   35-pass optimization pipeline; COLOR (register allocation)
//!              runs at index 14
//!   CODEGEN    IL -> PPC instruction selection + Xenon scheduling
//!   CLEANUP    emit COFF, free temporaries, delete the _CL_* bundle
//! ```
//!
//! First-port targets (cleanest, self-contained, already partially RE'd) — do
//! these before anything with real function-pointer dispatch (codegen/scheduler
//! come last):
//
// TODO(T-E): IL reader   ~0x10b75000  (bounded parser; == A2 codec artifact,
//                                       double payoff)
// TODO(T-E): COLOR       ~0x10bc4be9  (register allocation; already RE'd in
//                                       COLOR_RE.md; cleanest differential-parity
//                                       first proof — bitvector scans)
// TODO(T-E): then OPTIMIZE passes outward, pass by pass
// TODO(T-E): CODEGEN + scheduler LAST (real indirect-call dispatch)
//
// Each ported pass is admitted only under the differential oracle:
// `port(IL) == c2(IL)` byte-exact (timestamp zeroed) across the whole corpus.

// (No pass implementations yet — intentionally empty.)
