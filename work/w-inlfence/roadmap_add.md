
## 10.29 W-INLFENCE — §10.26 item 3 is a CORRECTNESS PREREQUISITE, not an optimisation; the fence is real and its whole reach is one function (2026-08-09)

**`WB_INLINE_FINDINGS.md`'s inline decision has been priced by every lane since
it landed as "converts nothing by construction"** — its own §6.3 says so of both
remedies, and §10.26's item 3 inherited that verdict. **This lane reclassifies
it.** The inline predicate is not an optional optimisation the port may buy when
it is cheap enough; **its decline side is the precondition for the port emitting
a call at all**, because a call whose callee c2 inlines is a wrong body and not a
gap. That is a different kind of item from *"lower `lower_expr`"* — it converts
nothing and it is not allowed to be deferred, in the same way `docs/GAPS.md` §6's
fail-closed rule is not allowed to be deferred.

**What was on disk before.** `IlBundle::functions` has refused *"a callee that is
also DEFINED here"* since the MVP, so `mismatch` was 0 and all 434 of w-fltret's
`Timer` TUs were `vocab-gap`. What was missing is that this was **one `any()` at
the bottom of a whole-TU gate**, and `WB_INLINE_FINDINGS.md` §7 proposes
narrowing exactly that gate (*"varargs ⇒ never inlined … narrows
`IlBundle::functions()`' wholesale refusal"*). A class whose safety is an
accident of TU-level granularity is board **#232**'s shape waiting for a
widening lane.

**What ships.** One predicate — `bind::callee_defined_here` over
`IlFunction::callees()`, so every call carrier is covered — asked by the gate
(behaviour unchanged on all 878 TUs), by the census as a post-parse gate
(`callee-defined-in-tu`) and by `diag.rs`'s re-ask. Two fixtures, one integration
target, three unit tests. Board **#2220**.

**And the fence's whole reach on the workload is ONE FUNCTION**, because the port
can enumerate a TU's own defined names on **25 of 871** captured TUs: **845 have
an empty defined-name set**, **76** names are readable across the entire
workload, and **212,114 of the 212,125** in-class rows carrying a callee
(99.995 %) are fail-open on the inline question. Census **−1**, emitted **−1**.
Board **#2221**.

**The one function is `?supershuffle@@YAXPAD@Z`** — `src/keygen_xbox.cpp`, port
21 words against the reference's 26 — which is `WB_INLINE_FINDINGS.md` §6's own
anchor, reached from the census instead of from the disassembly.
`gap-metric frontier-codegen-wrong` goes **1 → 0**: across nine frontier TUs and
51 emitted functions that was the only positively-measured codegen error, and it
is now a refusal. **#1477 should read: `?supershuffle` is not a codegen target,
it is a refusal.** Board **#2222**.

**The over-broadness test is the oracle's own and it passes at 100 %.**
`fnbyte-exact` 36,228 → 36,228, `fnbyte-elided` 1,877/1,877, `fnbyte-spliced`
723/723 — all unchanged — while the single row taken back is `fnbyte-differs` at
base. Board **#2223**.

**Two findings that are not about this lane's own class.** A standing test
(`dead_temp_elision.rs` m02) had pinned a **wrong emit** as its expected outcome
since `w-inl0`, and passed every gate since (**#2224**); and a naive fence is
over-broad because the port **already has two graded inline models**, mechanism E
(`elide`, 1,877/1,877) and mechanism I (`splice`, 723/723), whose populations are
opposite — E's callees are rows the parser refused, I's are rows it accepted.
Three drafts of the exemption were refuted in order by six peer-lane cells
(**#2225**).

**Effect on the §10.27/§10.28 ordering.** Items 1 and 2 are unchanged. Item 3 is
**re-listed as a prerequisite rather than an optimisation**, and #2082's fourth
item — *"the inliner is the binding constraint on 444 already-in-class emitted
functions"* — gains a precondition: those 444 cannot be emitted correctly until
either the inliner is modelled **or** this fence can see their TUs, and today it
can see neither. The thing to check when a binding lane closes `vocab-gap` TUs is
not the census gain but whether `callee-defined-in-tu` appears with it: **88,228
emitted-name-carrying call rows are behind that door** (**#2226**).

[`rungs/2026-08-09-w-inlfence.md`](rungs/2026-08-09-w-inlfence.md).
