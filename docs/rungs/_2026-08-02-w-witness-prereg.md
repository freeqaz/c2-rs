# w-witness — pre-registration

Written and committed **before any measurement of the registered quantities**.
Base `a091e37` (`git log -1` checked in the worktree: the base is master's tip,
not a stale ref — the failure mode four lanes hit this week).

Lane premise: `docs/ROADMAP.md` §10.13 minted **#159**
(`emit-unbound-no-record|ordinary`, 6,271 symbols / 341 TUs / 65 TUs where it is
the only blocker) with *"nobody has read what an `ordinary` no-record symbol is"*
as step one. §10.14 is the record of that step failing: a standalone COFF reader
keyed on a **different predicate** (no `.gl` *run*) than the instrument
(`EmitBinding`: an emitted symbol with no framed `.gl` **body record**), and it
missed the known answer on the first witness TU (`DetectFrame.cpp`, harness 1 vs
reader 0).

So this lane does not read bytes. **It makes the harness print the names it
already classifies**, and then reads those.

## Declared bias, and its direction

**I want #159 to collapse into an existing phase, and I am primed for EH.**
§10.14 mentions that its failed reader returned `__unwind$NNNNN` symbols on two
of the three probe TUs, and I read that before estimating. The workload is 100 %
`/EHsc`. MSVC's funclet mangling (`?catch$0@?0??…`, `?dtor$0@?0??…`,
`?filt$0@?0??…`) is **single-`?`**, so it lands in `mangling_class` →
`"ordinary"` — which makes "the `ordinary` bucket is EH funclets" a tidy story I
would like to be true. It is registered as E1/E2 with an explicit refutation
condition precisely because I would otherwise find it in noise.

**Second bias: toward declining.** The brief says a priced decline is a
deliverable, and §10.13 already deflated #152 from +69 to +4. I am therefore
biased to under-price #159. E7 registers the verdict in advance so that
"declined" cannot be a conclusion I reach by preferring it.

## What is being built

A witness list **inside `crates/c2-harness/src/gap.rs`** — the file that already
computes the classification. No second implementation of any rule.

Seam note: the lane's seam is `gap.rs` and nothing else in `crates/`, so the knob
is an **environment variable, not a CLI flag** (a `--witness` flag would have to
be parsed in `main.rs`). That follows the file's own precedent: `wall_dump`
(`C2RS_WALL_DUMP`) and `row_dump` are already env-gated scratch instruments in
this file. Off by default; when off, the added code allocates nothing.

## Registered estimates

| # | claim | point | interval | what would refute it |
|---|---|---:|---|---|
| **E1** | the largest single **name family** in `emit-unbound-no-record\|ordinary` (6,271 symbols) is **MSVC EH funclets** (`?catch$` / `?dtor$` / `?filt$`), by share of symbols | **60 %** | [10 %, 95 %] | **no funclet-shaped name in the top 20 ranked names** ⇒ E1 is dead, whatever the top family turns out to be |
| **E2** | is it one family or many? | **ONE** — top-3 families ≥ **70 %** of the bucket's symbols | [40 %, 100 %] | a flat tail (top-3 < 40 %) ⇒ #159 is not a construct, it is a long tail of ordinary user functions and its 65 TUs are 65 separate problems |
| **E3** | distinct names ÷ symbols in the `ordinary` bucket (do the same names recur across TUs?) | **0.35** | [0.05, 0.9] | ≈1.0 ⇒ every symbol is its own name; no shared-header family exists to attack |
| **E4** | share of `emit-unbound-no-record\|special-generated` (947 symbols) that is `??_G`/`??_E`/`??_D`/`??__E`/`??__F` — i.e. genuinely synthesized, no `.ex` body, which is what **#152** is written about | **≥ 85 %** | [50 %, 100 %] | `??_7` (vftable) / `??_R*` (RTTI) / `??_C` (string literal) ≥ 20 % ⇒ #152 is partly a **data**-in-`.text` question and its +69/+4 price is about a different population than its title says |
| **E5** | share of `ordinary` no-record names that **do** appear as a mangled run somewhere in `.gl` — predicate `c2_il::mangled_names`, which is **neither** of the two predicates §10.14 confused | **30 %** | [0 %, 95 %] | ≈100 % ⇒ the name is present and only the *framed body record* is missing, which makes the bucket adjacent to `emit-unbound-has-record` (a binding defect) rather than a synthesis wall; ≈0 % ⇒ c2 invents these symbols with nothing in the container behind them |
| **E6** | **the control.** Every pre-existing `emit-*` key, plus TU match / mismatch / `emit-set-ceiling-*`, identical base → tip | **identical** | exact | **any** move ⇒ reported before anything else; the change did something a read-only witness list must not do |
| **E7** | the verdict I will reach on **#159** | **DECLINE or re-file as a dependency of an existing phase** (p≈0.65) | — | if the bucket is one cheap family the port could bind, #159 stays OPEN with a *raised* price and I say so |
| **E8** | `emit-unbound-has-record` (4,684 symbols) is dominated by **ordinary** `?`-names too, ≥ 50 % | **YES** | — | a different class dominating re-ranks what the "instrument defect" half of the wall actually is |

### The known-answer checks, stated in advance

These are the ones §10.14's reader failed, and they are what makes this lane's
output trustworthy rather than merely plausible:

* **KA1** — `src/system/hamobj/DetectFrame.cpp` must produce **exactly 1**
  `emit-unbound-no-record|ordinary` witness row. §10.14's reader returned 0 here.
* **KA2** — `src/lazer/game/PartyModeMgr.cpp`, `src/system/meta/Profile.cpp` and
  `src/system/meta/SongPreview.cpp` must produce **exactly 1** each.
* **KA3** — the witness rows, summed by bucket, must equal the scan's own
  `emit-unbound-no-record|<class>` and `emit-unbound-has-record` counters
  **exactly**. The list is emitted from the same loop that increments them, so a
  discrepancy is a bug in the emission, not a finding.
* **KA4** — a **positive** count on every line of the witness report. Absence
  must not read as success: an empty bucket prints its zero next to a nonzero
  total, never nothing.

## Controls — registered, not optional

* **C-incumbent (the named control).** The 878-TU scan at `a091e37`, run
  **before** the change lands, and re-run after. Registered incumbent values, to
  be confirmed by my own base run rather than quoted from the brief:

  | key | base |
  |---|---|
  | `emit-records` | 1,515,161 |
  | `emit-record-offsets` | 1,515,161 |
  | `emit-unbound-no-record` | 4,591 |
  | `emit-unbound-has-record` | 4,684 |
  | `emit-set-ceiling-today` / `repaired` / `wall` | 324 / 420 / 451 |
  | TU match / mismatch | 6 / 0 |

  A **new** key appearing at tip is an addition and is allowed; an existing key
  changing value is not.

* **C-tests.** `cargo test --workspace --release` — base **24 targets / 606
  passed / 0 failed**. The target count is recorded beside the test count
  because §9.18.8's newest instance was a runner reporting `ok` with 169 tests
  silently not run.

* **C-gate.** `scripts/gate.sh --jobs 6` — base **12/12 PASS, 2,544
  fixture-verdicts, 0 mismatch**. Quoted with its verdict count; `12/12 PASS`
  over 0 graded is a failure.

* **C-off.** A tip scan with the witness knob **unset** must reproduce every base
  number. This separates "the witness list changed a count" from "collecting
  witnesses changed a count".

## What this lane will NOT do

* No second COFF reader, no `.gl` re-parse, no re-derivation of `mangling_class`
  outside `gap.rs`. Any grouping of names into "families" for the write-up is
  **descriptive post-processing of harness-emitted strings** and will be labelled
  as mine, not as a harness predicate.
* No change to `crates/c2-il/src/codec.rs` or `crates/c2-il/src/func/bundle.rs`
  (lane `w-lo` is live in both), no change to any file in `crates/` other than
  `gap.rs`, and no edit to `docs/ROADMAP.md` or `docs/BOARD.md`.
* No emitter change. Nothing here can convert a TU, and E7 is a *reading*, not a
  build.

## The trap this lane is most likely to fall into

§10.13's own standing caveat: **every number here is an instrument estimate.**
The witness list makes the residue *legible*, and §9.20.3 is the recorded case of
a residue becoming more legible and monotonically better while covering exactly
nothing. Naming 6,271 symbols is not converting one. Whatever E1–E5 return, the
realized TU count of #159 remains **0 until a rung builds it**, and the 65 is a
ceiling on a ceiling: all 341 TUs are `vocab-gap` today.
