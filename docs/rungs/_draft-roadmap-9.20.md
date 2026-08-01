# 9.20 The `.gl` binding was one wrong byte-set, and it was worth +213 TUs of ceiling

**Board #151, and it is not the repair the board describes.** §9.18.4 priced
*"read the virtual member's `.gl` record shape"* at **+88 TUs** and named the
defect from a 16-byte transcription: *a virtual member's record carries extra
material that breaks the framing **and** the 32-byte name-distance bound.* Taken
to the byte on the same translation unit, both halves of that reading are wrong,
the population is not the virtual one, and the actual defect is a **name
separator the crate had already measured and this reader had never been taught**.

```
                                    base 9bf25a0      tip
emit-set MODEL ceiling, today            111          324      +213
                        repaired         116          420      +304
                        wall             755          451      −304
unbound emitted symbols with NO record 13,646        4,591    −9,055
```

**All four numbers are ceilings.** TU match is **6** at base and **6** at tip;
this lane shipped no codegen and converted nothing. §9.16.1 records what happens
when a board's payoff field and its outcome field are the same field.

---

## 9.20.1 The defect, at the byte

`.gl` introduces a record's name with `00` **or** `26`. That is not new — it is
`gl.rs`'s own `NAME_SEPARATORS = [0x00, 0x26]`, measured over eight real TUs and
33,059 names, and `gl_symbol_index` reads it. **`gl_symbol_runs` never did**: it
opens a run only after a `00`, so a `26`-introduced name is not mis-framed, it is
**never seen at all**.

The cost is not a missing name. It is a wrong *distance*:

```text
?_Copy_str@exception@std@@AAAXPBD@Z 00 <its record> 0e ae 15
  26 ??_Gexception@std@@UAAPAXI@Z 00 <the record the reader could not name>
```

The second record's "nearest preceding run" becomes `?_Copy_str…`, **85 bytes
back**, and `EMIT_MAX_NAME_TO_OFFSET = 32` then correctly refuses it. The record
lands in `records_nameless`, its symbol binds to nothing, and the emit-set
instrument counts it as **a symbol c2 emitted with no body in this bundle** — a
synthesis wall — when the body is right there and the offset points at a `4F 1F`
function start.

On `src/system/obj/TextFile.cpp`, **70 of 674** framed records. Workload-wide,
`records_nameless` **152,941 → 420**, a 99.7 % reduction, with the framing
untouched.

**A run must also TERMINATE at `26`, not merely open there**, and that half
repairs a second defect nobody was looking for. Terminating only at NUL lets the
run opened at the *previous* NUL swallow the `26`, so the scan resumes past the
name and it is still lost — and when the two record bytes before the separator
happen to be printable ASCII, the scanner was emitting the glue as a symbol:

```text
before   "H=&??_7FixedSizeAlloc@@6B@"        (`H=` is 0x48 0x3D — record bytes)
after    "??_7FixedSizeAlloc@@6B@"
```

Fourteen such names on `TextFile.cpp` alone. A name wrong in its first two bytes
is worse than a missing one: it is a plausible symbol no obj carries.

## 9.20.2 Why it looked virtual, and why that mattered

§9.18.3's control was real and its arithmetic was right — the no-record
population **is** 98.8 % virtual on non-`??` names against a 42.1 % bound
control. Virtualness is a **correlate of where the function is defined**, not a
property of the record:

> an **out-of-line** virtual (`??1String@@UAA@XZ`) is `00`-separated and bound
> already; an **inline** one is `26`-separated and vanished.

`NAME_SEPARATORS`' own doc says so in a sentence written weeks ago and never
connected to this: every `26` witness is *"`??_G`/`??_E` deleting destructors,
`??_7` vftables, the `??_R*` RTTI records, `_CT`/`_TI` EH descriptors, **and
header-inline member functions**"*. A header-inline member of a polymorphic class
is virtual; that is the whole of the 98.8 %.

**And the record-width story does not survive contact with the bytes either.**
Measured under the repaired scanner across the held-out grid, `TextFile.cpp` and
`App.cpp` — 3,256 virtual records:

| kind | n | name-NUL → body-offset-field distance |
|---|---:|---|
| free | 234 | **15** only |
| static | 457 | **15** only |
| member | 537 | 15, 17 |
| virtual | 3,256 | 15 (47 %), 17, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37 |

**Forty-seven per cent of virtual records are exactly as wide as a non-virtual
member's**, and the width takes twenty distinct values. "A virtual member's
record carries extra material" is true of *some* virtual records and of no
virtual record in particular.

## 9.20.3 The repair the reading invites is worth exactly zero, measured

§9.18.3's sentence points at one obvious fix: raise the 32-byte bound. It was
measured rather than argued — NUL-only scanner, framing unchanged, only the
constant moving, on `TextFile.cpp`:

| bound | records named | nameless | **emitted symbols covered** | names claimed by >1 record |
|---:|---:|---:|---:|---:|
| **32** (today) | 604 | 70 | **30 of 32** | 0 |
| 48 | 604 | 70 | **30 of 32** | 0 |
| 96 | 625 | 49 | **30 of 32** | **21** |
| 200 | 656 | 18 | **30 of 32** | **40** |
| 400 | 670 | 4 | **30 of 32** | **40** |

**Not one emitted symbol is recovered at any bound**, and past 96 the binding
starts handing one name to two records — which is the mis-emit this bound exists
to prevent. The named residue shrinks the whole way, so a lane grading itself on
`records_nameless` would have reported steady progress while covering nothing and
corrupting the binding. That is #144's shape again: **the residue moved and the
thing the residue is a proxy for did not.**

## 9.20.4 #121, settled — and the two `bind.rs` corrections

`codec::gl_offset_framed` on `src/App.cpp` finds **38** records, not the 34
`bind.rs:84` claimed. §9.15's re-measurement was right, and the 4 are exactly the
reader's 32-byte bound firing after the framing:

```
?_Copy_str@exception@std@@AAAXPBD@Z          dist 85
?what@bad_exception@std@@UBAPBDXZ            dist 96
??1bad_alloc@std@@UAA@XZ                     dist 97
?_Ret@?$_BothPtrType@…@@SA?AU__true_type@2@XZ  dist 81
```

**38 is the framing; 34 is the reader.** Both corrections are in the tree.

The second correction is the more misleading one. `bind.rs:84` reported 34 as
what the gate *finds* on a 9,033-body TU, which reads as a partial binding. It is
not: `gl_defined_names` returns empty the moment one framed record's nearest run
is out of bounds, so `Bindings::per_record` refuses `App.cpp` **whole**. Measured
at this HEAD, not inferred — the scan's own per-TU detail:

```
src/App.cpp   .ex 2552214 B, 3752 .gl names — c2_il::functions() = None
```

**The gate binds 0 of 9,033 bodies and 0 of 158 emitted functions on App.cpp.**
Both figures now sit in the doc comment.

## 9.20.5 The ladder, re-priced — and most of #152 was never synthesis

| §9.18.4's ladder | predicted | measured here |
|---|---:|---:|
| today | 111 | **111** (re-measured at `9bf25a0`, unmoved) |
| + row binding | 116 | **116** |
| + the `.gl` record shape (#151) | 204 | **324 today / 420 repaired** |
| + `??_` synthesis (#152) | 238 | — |
| both | 436 | — |

One reader repair, with **no synthesis phase at all**, lands within 16 TUs of the
ladder's *both-repairs* row. The reason is the finding rather than a bonus:

> **The wall fell from 13,646 symbols to 4,591. Two thirds of the population
> #152 was scoped to synthesize turned out to have a body record all along** —
> the reader could not see its name, so the instrument reported it as a symbol
> with no body.

`??_G`/`??_E`/`??__F` are `26`-separated *because* they are COMDAT-linkage, which
is exactly the class this scanner was blind to. **#152 must be re-measured before
it is worked**, against 4,591 and not 13,646, and the `??_` share of it re-derived
— the emitted residue's `special-generated` class fell from 90 to **6**.

## 9.20.6 How the binding was graded, since the oracle cannot grade it

| invariant | base | tip |
|---|---|---|
| **injectivity** — names claiming two rows / rows claiming two records | 233 / 33,552, both dropped | 712 / 39,371, both dropped |
| **totality** — `records == bound + residue` | 0 accounting breaks | **0** |
| **ARITY (#144)** — framed records' body offsets | 1,515,160 records | **1,515,160 records, 1,515,160 offsets, 0 arity breaks** |
| **agreement, 6 byte-exact TUs** | residue 0 | **residue 0** |
| **agreement, the 158 listing-adjudicated records** | 147 of 158 bound | **154 of 158** |

**The arity axis is the one that had to be built, and it is the one that says
what kind of change this was.** Totality cannot distinguish "we found a record"
from "we found a name": moving a record from `bound` to `records_nameless`
satisfies `records == bound + residue` exactly. So `EmitBinding::record_offsets`
publishes the framing's *contents*, and `c2rs gap` prints it beside the residue
on every scan. **Records were 1,515,160 before and after** — byte-identical
across a change that moved 152,521 records out of the nameless bucket. That is
the control passing, and it is the evidence that the framing was not touched.

The unit test is built to the same rule: two inputs differing in **exactly one
byte** — the separator — asserted to leave `records` and `record_offsets`
identical while the binding moves.

**The injectivity residue went up and that is reported, not buried.**
Row-conflicts +5,819 and name-conflicts +479: more records now carry a name, so
more of them can collide, and every collision still drops *both* claimants. It is
the honest cost of the repair and it is where a wrong binding would hide.

**#149's coverage bound applies at full strength.** The 878-TU scan reads
`mismatch 0` at base and tip, and that is **not** evidence the binding is right —
865 TUs refuse before the emitter and the scan cannot see a binding defect at
all. The invariants above are the grading; the scan is a non-regression.

## 9.20.7 The rule that was frozen, and refuted out of sample

A **forward** record parser was derived on `TextFile.cpp`, committed at
`0400e2d` before the held-out grid was designed, and scored on a grid crossing
the structural axes the fitting TU could not vary — non-virtual, inline, single /
multiple / virtual inheritance, covariant return, pure virtual, template
instantiation, nested class, operators, >32 vtable slots, and record position in
the `.gl` stream.

| variant | held-out (14 cells, 114 emitted) | in sample (`TextFile.cpp`) | off a `4F 1F` | injectivity |
|---|---:|---:|---:|---:|
| today (incumbent) | 101/114 **88.6 %** | 30/32 | 0 | 0 |
| **`26` separator only — SHIPPED** | 112/114 **98.2 %** | 31/32 | 0 | 0 |
| `26` + varint framing (backward) | 114/114 100 % | 32/32 | **3** | 0 |
| **FROZEN forward rule** | 94/114 **82.5 %** | 32/32 | 0 | 0 |
| forward, relaxed | 114/114 100 % | 32/32 | **1** (name-distance 171) | 0 |

**The frozen rule scored 82.5 % out of sample against an incumbent at 88.6 % —
worse than the reader it was written to replace**, having been perfect in sample.
It is sound (zero false records, perfect injectivity) and it **over-refuses**:
step 3 requires the first `0x80` after the name to be the type-id field, and the
`k_wide` cell — 40 virtual slots — puts a varint-escaped slot index `80 <LE32>`
in front of it. **The grid caught it on precisely the axis the fitting TU could
not vary**, which is the whole argument for crossing structural axes before
varying values inside them. §9.19 lost 360/360 → 296/394 to the same shape; this
lane lost 100 % → 82.5 %.

The shipped repair is the smaller one, and it was **not** the frozen rule.

## 9.20.8 The second defect, found and declined

`emit_offset_framed` pins `gl[o-2] == 0 && gl[o-1] == 0`. Those two bytes are
**varint fields** — `readers.rs::read_varint`'s encoding, `0x80` + LE32 or one
signed byte — whose value is merely *usually* zero. `?Print@TextFile@@UAAXPBD@Z`
carries `2c 00`, value 44, and its record is not framed at all:

```text
?Print@TextFile@@UAAXPBD@Z 00  82 07 05 00  00 20 01 04 02 93 45 dd 20
  80 a3 22 00 00   2c 00   80 1a e4 03 00   80 48 06 01 00
  \_ 80 <LE32 tid> _/  \varints/  \_ 80 <LE32 body> _/
```

**This is board #121's defect one field later** — `gl[o-5] == 0x10` pins a byte
of the type-id's value, `gl[o-2] == 0` pins a varint's value — and it is
**DECLINED**, with the price stated:

* **worth**: 1 emitted symbol on `TextFile.cpp`, 2 of 114 on the held-out grid.
* **cost**: the backward relaxation admits **3 records whose body offset is not a
  `4F 1F` function start** on the fitting TU alone, with offsets `0x6E3F007D` and
  `0x3F260001` — 1.8 G and 1.06 G against a `.ex` of 334,576 bytes. Those do not
  fall out harmlessly: `EmitBinding::new` binds a record to the segment
  *containing* its offset, and `partition_point` puts an offset past the end on
  the **last row**, colliding with that row's real record and dropping both.
* the forward form that admits none of them is the rule §9.20.7 refuted.

A body bound under another symbol's name is a mis-emit, and a mis-emit outranks
the gap it closes. **Two guards a follow-up should have first**: the body offset
must be inside `.ex`, and the name must survive a plausibility check (one of the
three false positives is `X?DataFunc`, record bytes read as ASCII).

## 9.20.9 Priced and not taken — the 32-byte bound

Under the repaired scanner the name→offset distance still exceeds 32 on **420 of
1,515,160** records (0.028 %). They are real: on `App.cpp` there are exactly
three, at distances 33, 33 and 35, and all three point at a `4F 1F` —
`??_GSfxSeq@@UAAPAXI@Z`, `??_ESfxSeq@@WCM@AAPAXI@Z` (a `W` adjustor thunk) and
`??1SfxSeq@@UAA@XZ`.

Raising the bound to 48 was **measured on the full workload, not estimated**:

| | tip (32) | bound 48 |
|---|---:|---:|
| ceiling today | 324 | **335** |
| ceiling repaired | 420 | **435** |
| wall | 451 | **436** |
| records nameless | 420 | **1** |
| row-conflicts | 39,371 | **39,529** |

**+11 TUs, at 158 more record-conflicts.** Not taken: 48 is a constant fitted on
`App.cpp` with no out-of-sample test, and this lane has already been shown once
today what that is worth. The out-of-sample test it needs is named — a grid whose
cells carry adjustor thunks (`W`/`X` access codes) under multiple inheritance,
which is where the 33–37 distances live.

## 9.20.10 Scope — what was deliberately not moved

The **gate** keeps the NUL-only scanner. `gl_defined_names`, and therefore
`Bindings::per_record` and `IlBundle::functions`, are untouched, exactly as
`bind::emit_offset_framed` is already kept separate from
`codec::gl_offset_framed`. Widening what the gate *accepts* moves the emitted
class and could cost the 6 byte-exact TUs; widening what the **instrument** can
see is what the ceiling is measured on. §9.20.4 is the price of that separation
stated plainly: the gate still binds 0 on `App.cpp`, and **realising** any of
§9.20's ceiling needs the gate to adopt this reader — a separately-gated decision
with the differential re-run behind it, and the first item this lane hands on.

## 9.20.11 Pre-registration, scored — 8 of 12, and the misses are the lane

Registered in `docs/rungs/_2026-08-01-w-vgl-prereg.md`, committed at `e12ee81`
before the first measurement; the shape rule frozen separately at `0400e2d`
before the held-out grid existed. Declared bias: **borrowed** (§9.18.3's
transcription was read first) and **optimistic that the defect was one constant**.

| # | claim | est | interval | actual | score |
|---|---|---|---|---|---|
| E1 | `gl_offset_framed` records on `App.cpp` | 38 | [34, 60] | **38** | HIT |
| E2 | `per_record` binds 0 on `App.cpp` | YES | — | **YES** — `functions() = None` | HIT |
| E3 | median name→offset distance, virtual record | 40 B | [33, 80] | **17** | **MISS** — below the floor |
| E4 | share of the 13,646 recovered by widening the bound alone | 12 % | [0, 60] | **0 %** | HIT, at the floor |
| E5a | `emit-set-ceiling-today` at tip | 150 | [111, 210] | **324** | **MISS** — above the ceiling |
| E5b | `emit-set-ceiling-repaired` at tip | 200 | [116, 260] | **420** | **MISS** — above |
| E6 | out-of-sample accuracy of the frozen rule | 92 % | [50, 100] | **82.5 %** | HIT on the letter, **MISS in substance** |
| E7 | agreement, the 158 listing-adjudicated records | 154 | [120, 158] | **154** | HIT |
| E8 | an arity invariant is green at base | YES | — | **YES** — 0 breaks, records identical | HIT |
| E9 | 6 byte-exact TUs hold, mismatch 0 | YES | — | **YES** — gate 12/12, 2,520 verdicts | HIT |
| E10 | TUs converted by this lane | 0 | [0, 0] | **0** | HIT |
| E11 | the extra material is variable-width | VARIABLE | — | **VARIABLE — 20 widths, 15–37** | HIT |

**The four misses are worth more than the eight hits.**

* **E3 is the borrowed prior failing exactly where it was declared.** I registered
  40 bytes because §9.18.3 said virtual records are longer. The median is **17**,
  and 47 % of virtual records are 15 — the same as a non-virtual member's. Had I
  gone to the byte before estimating, as this document tells five other lanes to,
  the number was one histogram away.
* **E5a/E5b are misses in the useful direction and they re-price a different
  board.** I registered 150 against §9.18.4's 204 counterfactual and got 324. The
  gap is not codegen; it is that the same defect that hides the virtual
  population also hides the `??_` one, so **#152's wall was two-thirds a reader
  defect** (§9.20.5). A miss above the ceiling is still a miss and is scored as
  one.
* **E6 is the registration defect, and it is the same one §9.18.9 scored on E2.**
  I registered an *absolute* accuracy with a decline floor at 70 % and **never
  registered the incumbent's accuracy as the control**. The frozen rule scored
  82.5 % — inside my interval, above my floor, and **below the 88.6 % reader it
  was meant to replace**. A registered interval that passes a change which is
  worse than doing nothing is not a test. The decline floor did not fire; the
  baseline comparison, which I failed to register, did.
* **E4 is a hit whose value is entirely in the direction.** It sits at the very
  floor of its interval — the repair §9.18.3's wording invites recovers **zero**.

## 9.20.12 Gate evidence

| lane | base `9bf25a0` | tip |
|---|---|---|
| `cargo test --workspace --release` | **600 passed, 0 failed, 1 ignored, 24 targets** | **604 passed, 0 failed, 1 ignored, 24 targets** |
| `#[test]` grep over `crates/` | **601** | **605** (+4, all new) |
| `scripts/gate.sh --jobs 6` | — | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **2,520 fixture-verdicts**, 0 mismatch in every lane |
| `c2rs selftest` | 210 PASS, 0 FAIL | **210 PASS, 0 FAIL** |
| 878-TU workload scan | match 6, mismatch 0, codegen-gap 0, vocab-gap 865, capture-fail 7 | **identical** |
| census | 706,402 / 2,462,571 (28.69 %) | **identical** |
| emitted census | 36,059 / 178,968 (20.15 %) | **38,456 / 178,968 (21.49 %)** |
| emitted residue | 17,706 (9.89 %) | **9,275 (5.18 %)** |
| census/gate disagreement | 0 | **0** |
| distance (bodies) | ≤0: 1, ≤1: 10, ≤10: 25, ≤100: 32, ≤1000: 210 | **identical** |
| distance (emitted) | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 399, ≤1000: 857 | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: **403**, ≤1000: **858** |
| emit-set ceiling (§9.16.3) | 25 of 871, violations 0 | **25 of 871, violations 0** |
| emit-set MODEL ceiling | 111 / 116 / 755 | **324 / 420 / 451** |
| binding arity (#144) | 1,515,160 records | **1,515,160 records, 1,515,160 offsets, 0 breaks** |

**Target count recorded beside test count**, per §9.16.8: **24 at base and 24 at
tip**. `cross_sweep` not run — **no codegen was touched**. The diff is
`c2-il/src/func/gl.rs` (one `pub(crate) fn`, one shared helper, one test),
`c2-il/src/func/bind.rs` (the arity field and accessor, the instrument's run
scanner, three tests, the two #121 doc corrections) and `c2-harness` (one gap
accounting row, one report line). `PortC2`, `codegen` and every recognizer are
untouched; `codec::gl_offset_framed`, `gl_defined_names` and `Bindings::per_record`
are untouched **on purpose**.

**The base was verified before anything else.** The worktree this lane was handed
was created on `4ea415a` — **2026-07-19, 700+ commits behind master** — the
failure mode §9.18's pre-registration also records and the brief names for five
lanes this week. Caught by `git log -1` as the first command of the session.

## 9.20.13 Found and not taken, ranked

1. **Teach the GATE this reader.** The whole of §9.20's ceiling is unrealisable
   until `gl_defined_names` sees `26`-separated names — today it refuses `App.cpp`
   and every TU like it, whole, for want of four names. It moves the accepted
   class, so it needs the full differential behind it. **This is the lane's own
   first recommendation and the largest item it did not take.**
2. **Re-measure #152 against 4,591, not 13,646.** Two thirds of the synthesis
   wall was this defect. The `??_` decomposition of §9.18.3 needs re-deriving
   before any synthesis phase is scoped; `special-generated` in the emitted
   residue is already down from 90 to 6.
3. **The 32-byte bound, +11 TUs, priced in §9.20.9** — needs a grid carrying
   `W`/`X` adjustor thunks under multiple inheritance, which is where the 33–37
   distances live.
4. **The varint framing (§9.20.8), declined** — needs an in-`.ex` bound on the
   body offset and a name-plausibility check before it can be safe.
5. **The row/name conflict residue, now 39,371 + 712 records.** It grew because
   the repair gave more records names. It is the largest remaining component of
   the 96-TU gap between the `today` (324) and `repaired` (420) ceilings, and
   nobody has looked at what a collision actually *is* in `.gl`.
