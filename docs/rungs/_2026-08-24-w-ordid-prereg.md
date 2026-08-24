# PREREG — `w-ordid`: carry a FUNCTION IDENTITY through the funcwalk tap, so a consumer can VERIFY its ordinal-to-function pairing

**Frozen 2026-08-24, as the first commit on branch `wt-w-ordid`, before any
measurement was taken on this tree.** Base `e85253cda`.

Row: board **#3459**, raised by `docs/rungs/2026-08-23-w-pwords.md` §5 — *"the
ordinal hazard, which no document named"*. Funded by
`docs/DECISIONS_2026-08-22.md` decision 9 (board **#3466**). Reserved rows
**#3477**–**#3480**; next free after the seventh wave is **#3485**.

Lane kind: **construct rung** — shared instrument machinery, re-expressed
through already-graded consumers. `Fixtures: none`. `Census: +0`.
**Required-zero byte delta**: the port's emit path is not touched.

---

## 0. THE FENCE, STATED FIRST

This lane changes the **oracle/instrument** side only. It **licenses no emit**,
it is **never a gate**, and it never stands in for the byte judge (`CLAUDE.md`
§ "The one correctness rule").

**Edit fence** (from the brief): `crates/c2-reference` and `crates/c2-harness`
are this lane's. `crates/c2-core` and `crates/c2-il` are peer lanes'
(`w-s1c3`, `w-c1`) and are **not touched**. `fixtures/` is not touched.
`c2host/stagetap.c` is the tap itself and is in scope — it is not a `crates/`
crate and no peer lane in wave 7 names it.

**One exception is claimed and named here rather than discovered later:**
`docs/whitebox/DISCLOSURE.md` gets **one appended row**. `CLAUDE.md` requires
a row naming the address in the same commit that adopts a disassembly-derived
field, and `W-STAGETAP-5` is the standing precedent for `c2host/stagetap.c`
reads. An append is the smallest possible collision surface; it is flagged to
the coordinator in the final report.

---

## 1. THE HAZARD, RESTATED — and what is and is not yet verified

`w-pwords` §5, verbatim in substance:

> This instrument pairs funcwalk `func == i+1` with the i-th `.text` function
> in address order. **Nothing in the funcwalk payload carries a name to check
> that with.** … This is a live limitation of every funcwalk-based instrument
> in the tree.

**Read on this tree before the prereg was written** (citations checked, per the
standing disclaimer that the brief's citations name enumerations, not facts):

| claim | checked | result |
|---|---|---|
| the tap's ordinal is a `sched1`-entry counter | `c2host/stagetap.c` `tap_enter`, `if (strcmp(g_phase,"sched1")==0) g_fn++;` | **HOLDS** — `g_fn` is incremented at exactly one of the eight sites, so a function whose `sched1` is skipped inherits the PREVIOUS function's ordinal at `after0` |
| the payload carries no identity | `TapReport::parse`'s `FN` arm reads `<phase>` and `<n>` and nothing else; `FuncWalk` has three fields | **HOLDS** |
| the consumer assumes address order | `pwords_bijection.rs:543-551` — `n_walks == funcs.len()` then `find(func == fi+1)` over `.text` sorted by symbol `Value` | **HOLDS** |
| 19 functions quarantined `OrdinalUnverified` | not yet re-measured on this tree | **§4 P0** |

**NOT yet verified and registered as this lane's own risk:** that the 19 are
reproducible at this base, and that the `sched1`-skip mechanism above is what
produces them. `w-pwords` names a different proximate trigger (a TU where the
funcwalk COUNT and the `.text` count disagree). Both are consistent with a
permuted pairing; this lane does not need to choose between them, but it must
not assert one.

---

## 2. THE DELIVERABLE

### 2.1 The identity, and where it is read from

`c2.dll` `sha256 c80981c0…a66258`. Three hops, all read from the
disassembly **before** this prereg was frozen:

| hop | witness |
|---|---|
| `func + 0x00` → the `.gl` **symbol record** | `FUN_10bc4715` (the function-record constructor): `pbVar3 = alloc(0xac); *(int *)pbVar3 = param_1;` where `param_1` is the symbol the work queue dequeued in `FUN_10b7f1ff` (`DAT_10c40214`, `+0x4c` emit bits, `+0x78` next) |
| `sym + 0x04` → `char *`, NUL-terminated | **`0x10b9acd0`** `mov eax,DWORD PTR [ecx+0x4]` in `FUN_10b9acc4`, c2's own symbol-name getter, with `ecx` = the symbol; and **`0x10b97f38`** `mov esi,DWORD PTR [ecx+0x4]` in `FUN_10b97f37`, which NUL-tests the same field |
| `sym + 0x30` / `sym + 0x31` — kind / sub-kind | `P_SYMBOL.md` §1 (`0x10b28ba3`, `0x10b2823b`); re-read here at `0x10b97f6b` `movzx eax,BYTE PTR [ecx+0x30]` |

**The decoration caveat, read and NOT ported.** `FUN_10b9acc4` returns
`[sym+4]` verbatim only when `FUN_10b97f37` says so — kind 4, kind 9, or a
kind-1 `?`-name. For **kind 3** it goes through `FUN_10b99dfe`, which
`strcpy_s`es `[sym+4]` into a buffer and **returns it unchanged the moment it
starts with `'?'`** (`if (DAT_10c6a020 == '?') goto LAB_10b9a093;`), and
otherwise applies per-sub-kind decoration (`"__unwind$"` for `'T'`,
`"__catch$"` for `'V'`, `"$M"` for `'W'`, a trailing `"$"` for `'\0'`).

**This lane does not reimplement that decoration.** It emits the RAW
`[sym+4]` string plus the two kind bytes, and the consumer matches by **exact
string equality** against the obj's `.text` function symbol names. A name that
does not match is **`IdentityUnmatched`, counted, and never silently demoted to
address order.** If the unmatched rate turns out to be large, that is a
measurement this lane publishes — not a licence to add a guessed rule.

### 2.2 The payload change

`c2host/stagetap.c`, funcwalk header line only:

```
  before:  FN <phase> fn <n>
  after:   FN <phase> fn <n> sk <kk> <ss> nm <name>
```

`<kk>`/`<ss>` are two hex digits each. `<name>` is the rest of the line.
**Three distinguishable spellings, never a silent zero** (the rule `rd32`
already follows): the name; `<unread>` for a pointer the plausibility filter
refused or a NULL; `<nonascii>` for a string containing a byte outside
`0x21..=0x7e` (space included in the refusal, because the name is the tail of a
whitespace-delimited line). Length bounded at 512.

The read is **bounded and fail-closed** exactly like `tap_walk_function`'s:
`plausible()` on every hop, stop at the first NUL, stop at the first
non-printable byte, stop at 512.

### 2.3 The Rust seam

`crates/c2-reference/src/stage.rs`:

* `FuncWalk` gains `sym: String` (empty when the payload carried none) and
  `sym_kind: Option<(u8, u8)>`.
* `TapReport::canonical_bytes` includes the identity, and **`SCHEMA` goes
  1 → 2**. Nothing in the tree persists a canonical stream or pins a digest
  (`grep` for `digest()`/`canonical_bytes` finds only `stage snap`'s
  within-process rerun comparison), so the bump costs nothing and lying about
  the schema would.
* A new `TapReport::verify_ordinals(phase, &[expected names in .text address
  order]) -> OrdinalVerdict`, so the check is written **once** and every
  consumer gets the same one. Verdicts: `Verified{n}`, `NoIdentity{n}` (the
  payload carried no names — the pre-#3459 state, reported as such),
  `CountMismatch{walks, text}`, `Unmatched{ordinal, tap, text}`.

### 2.4 The consumers — enumerated, every one, counted and never floored

Enumeration is `grep -rn '\.funcs' crates/` at base `e85253cda`, plus the tap
display path. **Five**, and each gets a verdict in the rung:

| # | consumer | pairs an ordinal to what | plan |
|---|---|---|---|
| 1 | `crates/c2-harness/tests/pwords_bijection.rs` | `.text` address order | **CONVERT** — the brief's named candidate |
| 2 | `crates/c2-reference/tests/middle_interfaces.rs` `the_final_tuple_order_reproduces_the_text_words` | `.text` address order, hardcoded `(fixture, func, ord)` cells | **CONVERT** |
| 3 | `crates/c2-harness/tests/stage_region_trace.rs` PROBE C | the **port's** emitted function list, address order | **CONVERT** |
| 4 | `crates/c2-harness/src/cli/stage.rs` `stage snap` | display; its `FW-XDERIV` compares funcwalk against the REGION walk at the same ordinal — both tap-side | **display only**: print the identity. The cross-derivation is tap-internal and is **unaffected** by this hazard |
| 5 | `crates/c2-reference/tests/stage.rs` | `rep.blocks` (region walk) only — never `rep.funcs` | **UNAFFECTED** |

---

## 3. THE FENCE, WATCHED REFUSING

*A fence never seen refusing is not a fence* (`CLAUDE.md`'s formatter rule,
generalized; `w-pwords` §6.1 is the priced example of registering a check that
the result then made vacuous).

**F1 — portable, no toolchain.** Unit tests on `verify_ordinals` in
`crates/c2-reference/src/stage.rs`, over a synthetic payload:
`(a) correct list → Verified; (b) list with two names SWAPPED → Unmatched;
(c) list one shorter → CountMismatch; (d) payload with no identity → NoIdentity`.
(b) is the load-bearing one: a permuted pairing is exactly what `w-pwords`
diagnosed, and it must be caught, not absorbed.

**F2 — live, on real payloads, at no extra capture cost.** Inside
`pwords_bijection.rs`'s per-fixture measurement, run the verifier a **second**
time against a ROTATED name list. On every fixture with ≥2 distinct `.text`
names the rotated verdict MUST NOT be `Verified`. The count of fixtures where
this fence fired is asserted `> 0` and reported.

**F2 is registered as possibly-vacuous, and the escape is registered with it**
(this is precisely `w-pwords` §6.1's lesson): if it turns out that most
fixtures have a single `.text` function, the rotation is the identity map and
the check reads nothing. The published number is therefore *"the fence fired on
N of M fixtures"*, with N asserted `>= 5`, and if N is smaller the lane says the
fence is weak rather than quoting it as passed.

**Not a fence and named so:** `assert!(sym != "")` — an always-non-empty field
proves the C side wrote something, not that it wrote the right thing.

---

## 4. PREDICTIONS, registered before measuring

| # | prediction | how it is scored |
|---|---|---|
| **P0** | the `OrdinalUnverified` stratum reproduces at **19** functions on the base tree at `C2RS_PWORDS_LIMIT=0` | base evidence run |
| **P1** | the identity read resolves (neither `<unread>` nor `<nonascii>`) on **≥ 95 %** of `after0` funcwalk rows | tip run, published as a rate |
| **P2** | of the resolved names, **≥ 90 %** match a `.text` function symbol name by exact equality | tip run |
| **P3** | `OrdinalUnverified` **19 → 0**, and the 19 functions land in `Leaf`/`FramedClean`/`FramedUnbounded` | tip run |
| **P4** | **H0 (`T == W`) stays 100.00 %** on every verified stratum, INCLUDING the 19 recovered rows. This is the falsifiable form of `w-pwords`'s diagnosis: if the six `wkg_splice_pos.cpp` "failures" really were a permuted pairing, naming the pairing must make them pass | tip run |
| **P5** | for every fixture that produced **zero** `OrdinalUnverified` rows at base, the per-function rows `(T, W, I, P, stratum)` are **IDENTICAL** at tip | base-vs-tip row diff |
| **P6** | the total graded count moves from 2,922 by **at most ±40**, and every moved row is in a fixture named in the base run's quarantine | base-vs-tip diff |
| **P7** | required-zero: `scripts/gate.sh --jobs 4 --require-graded` is **line-identical** at base and tip over the count-bearing rows and the 498 gap-metric keys | two gate runs |

**P4 is the one this lane can lose on**, and losing it is a result: it would
mean the H0 failures were a real compiler behaviour that the count-mismatch
quarantine hid by coincidence, and `w-pwords` §5's diagnosis would need
amending rather than closing.

**Registered as the most likely way this lane misreports itself:** an identity
that is always present and always matching would make `verify_ordinals` a check
that cannot fail on this corpus, and `Verified` would then be reporting the
fixtures' shape, not the mechanism. §3's F1(b) and F2 exist for exactly that,
and the rung must publish the fence-firing count beside the pass rate.

---

## 5. FAILED, defined in advance

The lane reports **FAILED** — in that word — if any of:

* the identity cannot be read at all (P1 under 50 %), and no second field
  reachable from `func+0x00` supplies one;
* the fence (F1(b)) cannot be made to refuse;
* the gate's count-bearing rows or the 498 gap-metric keys move (P7 red) — an
  instrument lane that moved the port's numbers is not an instrument lane;
* the armed suite is red at tip for a reason this lane introduced.

`declined` is available and is a real outcome: if the read lands but the match
rate is so low that no consumer can be converted honestly, the lane lands the
payload field + the fence and declines the conversions, saying so.

---

## 6. Protocol

* Evidence runs: `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release
  --no-fail-fast` (`timeout 5400`) and `sh scripts/gate.sh --jobs 4
  --require-graded` (`timeout 3600`), both to `work/w-ordid/` with `EXIT=`
  markers and ONE bounded waiter each.
* `crates/` and `docs/rungs/` are not edited while an evidence run is live.
* The test-parallelism race `w-pwords` fixed (work dirs keyed on PID alone)
  stays fixed: any new work root this lane adds is keyed on PID **and** a
  per-call counter.
* No push, no merge.
