# w-vgl — pre-registration

Written and committed **before any measurement of the registered quantities**.

Base **`9bf25a0`**, verified. The worktree this lane was handed was created on a
stale ref — `4ea415a`, **2026-07-19, 700+ commits behind master** — which is the
exact failure mode the brief names ("five lanes this week started 541–609 commits
behind"). Caught by `git log -1` as the first command of the session and reset
onto `9bf25a0` before anything else was read. Recorded here because a
pre-registration written against the wrong tree grades nothing.

Lane premise: `docs/ROADMAP.md` §9.18 — the emit-set **model** is not the gate (a
cell table over every census feature is worth ~0 TUs held out); **the `.gl`
binding is**, on 760 of 871 TUs. §9.18.4's ladder prices *reading the virtual
member's `.gl` record shape* (board #151) at **+88 TUs of ceiling**, the largest
single item on the board. §9.18.3 transcribed the byte witness:

```
virtual      ?Print@TextFile@@UAAXPBD@Z\0 82 07 05 00 00 20 01 04 02 93 45 dd 20 80 a3 22
non-virtual  ??0DataNode@@QAA@H@Z\0       86 03 05 04 20 00 02 01 00 80 …
```

— "a virtual member's `.gl` record carries extra material that breaks the framing
*and* the 32-byte name-distance bound."

**Nothing in `crates/c2-core` is touched and no codegen ships.** This is reader
work in `crates/c2-il`. A wrong *reader* does not produce refusals — it produces
a body emitted under another symbol's name, which is a mis-emit, and a mismatch
outranks every other outcome (`docs/GAPS.md` §6). So every repair below is
fail-closed in the same direction the existing reader is: a record whose shape is
not positively recognised yields **no name**, never a borrowed one.

## Declared bias

**Borrowed, and in one specific place.** I read §9.18.3's two-record transcription
before estimating, and its parse ("`82 07`/`86 03` return type, `05` linkage,
`00`/`04` return size, then `20` …") is the *only* structural prior I have about
what the extra material is. That transcription is 16 bytes of one record on one
TU, truncated on both witnesses (`80 …`, `80 a3 22`), and it was made by a lane
that did not own this crate. **Two prior warnings apply to it directly**: "key
names lie — go to the byte" (five times this week), and "a board item's quantity
ages". I am therefore registering E1/E2 as *re-measurements* rather than as
inherited facts, and E11 predicts against the most convenient reading of that
transcription.

**Second bias, and it is the one I expect to cost me: optimism about a
single-constant repair.** The cheapest possible outcome is that
`EMIT_MAX_NAME_TO_OFFSET = 32` is the whole defect and raising it recovers the
virtual records. §9.18.3 explicitly says the *framing* is lost as well, so I am
registering E4 low on purpose — but the pull toward "it is one constant" is real
and is named here so a hit on E4 is not read as insight.

## Registered estimates

| # | claim | point | interval | what would refute it |
|---|---|---:|---|---|
| **E1** | **#121 re-priced.** `codec::gl_offset_framed` framed-record count on `src/App.cpp` (§9.15 said 38; `bind.rs:84`'s comment says 34) | **38** | [34, 60] | 34 ⇒ §9.15's re-measurement was wrong and the comment is right |
| **E2** | `Bindings::per_record` binds **0** names on `src/App.cpp` | **YES** | — | any nonzero binding ⇒ the gate is not refusing App.cpp for the stated reason |
| **E3** | median name-NUL → body-offset-field distance for a **virtual** member's record | **40 B** | [33, 80] | > 200 or unbounded ⇒ a distance bound cannot be the repair at all |
| **E4** | share of the 13,646 no-record emitted symbols recovered by **widening `EMIT_MAX_NAME_TO_OFFSET` alone**, framing unchanged | **12 %** | [0 %, 60 %] | ≈100 % ⇒ the framing is *not* lost and §9.18.3's byte reading is wrong |
| **E5a** | **HEADLINE.** `emit-set-ceiling-today`, of 871, at this lane's tip (base to be re-measured; §9.18 published 111) | **150** | [111, 210] | ≤ 116 ⇒ the repair is worth no more than the row binding alone — DECLINE (see floor) |
| **E5b** | `emit-set-ceiling-repaired`, of 871, at tip (base 116) | **200** | [116, 260] | — |
| **E6** | **out-of-sample.** Record-recovery accuracy of the frozen shape rule on a structural grid designed *and named* before the rule is compiled | **92 %** | [50 %, 100 %] | < 70 % ⇒ the shape is fitted, not derived — DECLINE |
| **E7** | agreement with the **158 listing-adjudicated** records (§9.15: `.cod` settles 158 of 6,069 = 2.6 %) | **154** | [120, 158] | < 120 ⇒ the rule disagrees with the one independent ground truth available |
| **E8** | **arity (#144).** Adding a per-TU *arity* invariant (framed-record count and the (name, body-offset) multiset, not just a residue count) goes **green at base** | **YES** | — | red at base ⇒ the existing accounting was already losing record *contents*, which is a finding in its own right and outranks E5 |
| **E9** | the 6 byte-exact TUs stay byte-exact; workload mismatch stays **0** | **YES** | — | **any** mismatch — this outranks the entire lane |
| **E10** | TUs converted to byte-exact **by this lane** | **0** | [0, 0] | nonzero ⇒ codegen shipped, which the brief forbids |
| **E11** | the virtual record's extra material is **variable-width** (a function of the class's inheritance/vtable structure), not a constant insertion | **VARIABLE** | — | constant width across every crossed axis ⇒ the repair is a second constant and E4 should have been high |

**The decline floor, set in advance and to be honoured.** If E5a lands at **≤ 116**
*or* E6 lands **< 70 %**, this lane reports **DECLINE** on #151 — states the shape
as far as it was derived, names the residue, and prices what the decline costs in
TUs. Three lanes today wrote no port code and two produced the session's best
findings; a well-argued decline is a first-class result and will not be dressed
up as a partial win.

**And the ceiling is a CEILING.** Every number in E5 is "TUs whose emitted set
*could* be reproduced", never TU match. §9.16.1 records what happens when a
board's payoff field and its outcome field are the same field. TU match at base
is **6** and E10 registers that it stays 6.

## Controls — each could go red

* **C-arity** (#144, established today) — *totality residue 0 is not a control*.
  Removing a `DUP` expansion left totality silent at residue 0 while an arity
  check went 22 red. So the binding is graded on **record contents** — the
  multiset of `(name, body_offset)` pairs and the per-TU framed-record count —
  not only on `records == bound + residue`. Registered as E8 so it is scored even
  when it passes.
* **C-mutate** (#145, a validator that cannot see the defect it exists for is
  worse than none) — every new negative control holds the guard's quantity
  **fixed** and mutates exactly one thing, so an early guard cannot make the
  assertion under test unreachable. Specifically: a widened distance bound must
  be shown to change *binding*, not *record count*, and vice versa.
* **C-cross** (the generated-axis rule, which has bitten three times this week
  — arity, register position, structural counts) — the fixture grid **crosses
  structural axes first** and varies values inside each cell: virtual ×
  {single, multiple, virtual} inheritance × {covariant return, pure virtual,
  template instantiation, nested class} × record position in the `.gl` stream.
  More *names* at one structure is not an axis.
* **C-holdout** (§9.19's rule — 360/360 in sample, 296/394 out) — the shape rule
  is **frozen and committed** before the held-out grid is compiled, and the
  held-out grid is designed to vary the axis the fitting grid could not.
* **C-coverage** (#149) — "the scan is green" is not evidence. The 878-TU scan
  reads 0 mismatch today because 865 TUs refuse before the emitter, so it cannot
  see a binding defect. The binding is therefore graded on its own invariants
  (injectivity, totality, arity, agreement where independently known) and the
  scan is reported only as a non-regression.
* **C-inert** — every reader change is proved inert on the published numbers by
  *running* them: census, emitted census, match/mismatch, both distance ladders
  and the §9.16.3 ceiling, base vs tip.

## Instrument hygiene registered in advance

* `C2RS_GAP_CACHE=<main-repo>/work/capture-cache` **verbatim** — the main
  checkout's path, never the worktree's (#145: the cache is non-portable by
  *path length* and `--validate-cache` cannot detect it).
* #156: `prefilter` is not a valid byte-forensics instrument against an obj
  captured by another path. Both sides of any comparison are captured the same
  way, through the same `c2rs capture`.
* Disk is at 86 % — scratch captures are cleaned as the lane goes.
* `scripts/gate.sh` is the evidence, not `cargo test`. **Target count is recorded
  beside test count** at base and tip (§9.16.8).
