# w-mmioclose — `__declspec(noinline)` IS IN THE `.gl`, IT IS BIT 6, AND IT CLOSES A SHIPPED WRONG EMIT — while `mmio.cpp`'s first blocker turns out not to be `mmioClose` at all but ten names with no `@@`

    Tag:       w-mmioclose
    Slug:      w-mmioclose
    Date:      2026-08-09
    Fixtures:  none. The mechanism is a property of a COMPOSED body and of a
               whole-TU `.gl`, so a fixture cannot exercise it —
               `IlBundle::functions()` refuses any TU with a same-TU callee
               before composition happens. That is `w-inlfence2`'s reason
               verbatim, and it is graded the same way: 8 unit cells, plus TWO
               REAL-TOOLCHAIN integration cells that moved and are asserted
               against c2's own relocation table.
    Census:    per-function and emitted census **+0** — this lane ships no
               reader clause and no lowering. TU match **19 → 19**, mismatch
               **0 → 0**, codegen-gap **0 → 0**, vocab-gap **852 → 852**,
               capture-fail **7 → 7**. `fnbyte-exact` **35,793 → 35,793 (+0)**,
               `fnbyte-differs` **1,898 → 1,898**. **0 of 256 `gap-metric` keys
               moved**; the per-TU verdict set over all 878, by name, is
               **0 only-in-base · 0 only-in-tip · 0 changed**.
    Record:    this file; PREREG `work/w-mmioclose/PREREG.md`, committed at
               `b4f5d021` (`897499e2` pre-rebase) **before the first change to
               `crates/`**. Scored in §9. The price is
               `work/w-mmioclose/MMIO_PRICE2.md`.
    Lane:      `w-mmioclose`, branch `wt-w-mmioclose`, off master **`981efd7f`**
               (the `w-ifn` merge) and **rebased onto `c31ded7d`** (the
               `w-vsnprnc` merge) before reporting. **Master advanced mid-lane
               and it invalidated a measurement** — §2.1.
    Ships:     `c2_il::func::gl::{FN_FLAG_INLINABLE, gl_function_attrs,
               gl_noinline_names}`; `IlFunction::inlinable` and its two
               bundle-level fills (`IlBundle::functions`,
               `IlBundle::census_functions`);
               `c2_core::comdat::callee_is_one_c2_expands`' first clause;
               `c2_core::splice`'s `S7-callee-noinline`. Board rows
               **#2400**–**#2414**; **#2415**–**#2429** left explicitly
               unminted.
    Adopts:    **nothing.** No `docs/whitebox/DISCLOSURE.md` row. The bit is
               read off two IL populations this lane captured; that
               `WB_INLINE_FINDINGS` §1 independently calls it *bit 6 of
               `[sym+0x4c]`* is an agreement recorded in §3, not a source.

---

## 1. The result

> ### **THE FIELD BOARD #1039 FILED AS UNDECODED IS DECODED, AND IT IS ONE BIT.** `__declspec(noinline)` clears **`0x40`** of the attribute byte three fields past the `.gl` function record's body-start offset. Perfect separation on two independent populations: **9 of 9** manufactured cells (`plain`, `inline`, `__forceinline`, `static`, `static`+`noinline`, and two with real bodies) and **11 of 11** records of `src/xdk/nuispeech/mmio.cpp`, where exactly `mmioFlush` and `mmioSetBuffer` come back clear and the dc3 source marks exactly those two. Board **#2400**.

> ### **AND IT IS BIT 6, WHICH IS THE BIT c2's OWN LEGALITY TEST READS.** `WB_INLINE_FINDINGS.md` §1 read `0x10b5c06b` off the disassembly as *"requires **bit 6** of `[sym+0x4c]`"* before this byte was located. Two derivations, opposite sides of the seam, same bit. Board **#2401**.

> ### **A SHIPPED, PINNED WRONG EMIT IS NOW BYTE-EXACT.** `crates/c2-harness/tests/noinline_boundary.rs` cell `w10` asserted the WRONG behaviour on purpose — the splice putting `?g`'s two-word body where c2 emits one branch word, `Differs (2, 1, 0)` — with instructions that the commit which fixed it must come there. `splice_body_why` gains `S7-callee-noinline`, `Selected::Tail` picks the branch up behind it, and the cell reads **`Exact`**. **Board #1038 is closed.** Board **#2402**.

> ### **AND THE COMPOSITION FENCE'S MEASURED REACH COST GOES BACK TO ZERO.** `w-inlfence2` recorded cell `w04a`'s caller as the fence's cost, *"and it is one function"*, in the direction that cannot be a wrong emit. `comdat::callee_is_one_c2_expands` now stops predicting an expansion c2 does not perform, and `w04a` reads **`Exact`** again. Board **#2403**.

> ### **AT THE WORKLOAD'S OWN FLAGS c2 EXPANDS EVERY PLAIN SAME-TU CALLEE WHOSE RESULT IS USED — SEVEN CELLS OF EIGHT — AND `noinline` IS THE ONLY ONE THAT KEEPS THE CALL.** `work/w-mmioclose/probe/inl.cpp`, `/nologo /c /GR /O1 /Oi /EHsc`: a callee defined BELOW its caller, a `static` one, an `inline` one, a `__forceinline` one and one with a real arithmetic body are all expanded to `li r3,7 ; blr`. **Size does not separate them from `mmioFlush`, which is eight bytes and keeps its `bl`.** Board **#2404**.

> ### **`mmio.cpp`'s FIRST BLOCKER IS NOT `mmioClose`. IT IS TEN NAMES WITH NO `@@`.** The TU's own scan row reads **`1 .gl names`** against **11** `.ex` segments: ten of its functions are `extern "C"` and undecorated, `looks_mangled` is `contains("@@")`, and `Bindings::per_record` binds nothing. **Even with all six of `w-ifn`'s mechanisms paid and `mmioClose` byte-exact, this TU does not convert.** `docs/CEILING.md` §11's NC-4, fifth instance. Board **#2405**, **#2406**.

> ### **`w-ifn`'s SIXTH MECHANISM — *"there is nowhere in the port to ask a sibling-body question"* — IS REFUTED, IN CODE.** `IlBundle::functions()` is bundle-level and already reasons across siblings **four** ways on master, `callee_defined_here` among them. The conflation is between the *body* parser (one `.ex` segment) and the *acceptance* seam (the whole bundle); board #139 constrains the second and says nothing about the first. §4. Board **#2407**.

> ### **THE `fnbyte-exact` DELTA IS ZERO, AND THAT WAS THE MODAL REGISTERED OUTCOME.** The workload exercises neither shape — `noinline_boundary.rs` said so in its own words, *"a demonstrated defect that this corpus does not exercise"*, and `w-inlfence2` #2155 said the exposure is LATENT. The gain is in the cells. **A lane whose corpus number is 0 and whose cell number is +2 has to say which one is the claim**, and this one is the cells. Board **#2408**.

| | base `c31ded7d` | tip |
|---|---:|---:|
| **TU match** | 19 | **19** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 | **0 · 0 · 0** |
| vocab-gap · capture-fail | 852 · 7 | **852 · 7** |
| **`fnbyte-exact`** | **35,793** | **35,793 (+0)** |
| `fnbyte-differs` | 1,898 | **1,898** |
| `fnbyte-denominator` | 162,092 | **162,092** |
| `fnbyte-refused` | 114,649 | **114,649** |
| `fnbyte-decline-inlined-callee` | 1,003 | **1,003** |
| `fnbyte-census-disagree` / `-expressible` | 1,003 / **0** | 1,003 / **0** |
| `fnbyte-tus-full` | 16 | **16** |
| per-function · emitted census | — | **+0 · +0** |
| **`gap-metric` keys** | 256 | **256 — 0 vanished, 0 appeared, 0 changed** |
| **first-blocker keys** | 634 body · 613 emitted | **0 moved on either** |
| **per-TU verdict SET (878, BY NAME)** | — | **0 · 0 · 0** |
| fixtures at `/O1` (327) | 162 · 0 · 9 · 156 | **162 · 0 · 9 · 156 — 0 moved** |
| fixtures at `/Ox` (327) | 144 · 0 · 17 · 166 | **144 · 0 · 17 · 166 — 0 moved** |
| `noinline_boundary.rs` `w04a` `?f` | **`Refused`** | **`Exact`** |
| `noinline_boundary.rs` `w10` `?f` | **`Differs (2,1,0)`** | **`Exact`** |
| workspace tests | 1,406 / 38 targets | **1,414 / 38 (+8)** |

Every comparison is a key→value map or a set by name, never a `diff`:
`work/w-mmioclose/{metricdiff,keydiff,verdicts}.py` with their outputs beside
them.

---

## 2. §0 — the base, re-derived

`work/w-mmioclose/base2.out`, this lane's own scan of the 878 at the rebased
master, and **the workload stamp is registered because #2360's trap is that a
level means nothing beside the wrong tree**:

    c2-rs     c31ded7d (base) / this branch (tip)
    WORKLOAD  dc3-decomp 1e0215e753c2 — `src/` CLEAN (`git status --short -- src/`
              is empty; the tree reports DIRTY for `tools/` and an untracked
              `work/`, neither of which the scan reads)
    binary    pinned per run; both scans quote it
    wibo 1.2.0-c2rs.1 · cl.exe/c2.dll/c1xx.dll 16.00.11886.00

`fnbyte-denominator` reads **162,092** at both ends, so board **#2392**'s hazard
— the workload advancing under a lane — did not fire.

### 2.1 THE OTHER ONE DID, AND IT PRODUCED A FALSE REGRESSION THIS LANE ALMOST BELIEVED

> **Board #2409.** Master advanced from `981efd7f` to `c31ded7d` (the
> `w-vsnprnc` merge) **while this lane was running**, and the lane built its
> "base" counterfactual binary with `git checkout master -- crates/` — which
> silently fetched **the new master**, not the branch's base.

The fixture scan then reported **two `_neg` fixtures losing four byte-exact
functions** and one census row dropping from `guard-chain-shared-tail` in class
to `expr-cmp-eq` blocked. PREREG **F5** (a byte-exact function LOST) is
registered at p = 0.04 and **D2** requires the revert; the lane began one.

Three bisections cleared the lane's own change, one clause at a time — the
census assignment disabled, both consumers disabled, each still reading the
"regression" — and only then did `git diff master HEAD --stat` show **1,351
deletions across five files this lane never touched**. `w-vsnprnc` had widened
`guard_chain_shared_tail` on four axes; the extra in-class function was **its**
work, present in the "base" binary and absent from the tip.

**Two rules come out of it, and the second is the one worth carrying:**

1. `git checkout master -- crates/` is **not** a counterfactual. The
   counterfactual is `git checkout <merge-base> -- crates/`, or a rebase first.
   Every number in §1 above is taken **after** the rebase, at
   `git merge-base master HEAD == c31ded7d`.
2. **A bisection that clears every one of your own clauses is telling you the
   baseline moved, not that the cause is subtle.** This lane spent three builds
   looking for a mechanism by which adding a struct field could change a parse
   verdict. `git diff master HEAD --stat` was one command and would have been
   first if the possibility had been on the list.

The commission warned that peer sessions had modified a lane's worktree twice.
This is the third shape of the same hazard and it did not touch the worktree at
all — it moved the *ref the lane compared against*.

---

## 3. The bit

### 3.1 The record

`gl_defined_names_framed`'s published table stops at the body-start offset.
Three fields follow it:

```text
  00 <name> 00  <TYPE>  80 01 10 00 00 00 00  80 <LE32 offset>  <SRCPOS>  <SIZE>  <ATTR>
                        \___ the framing ___/  \_ gl_offset_framed ____/
```

`SRCPOS` is a byte under `0x80` or the escape `80 <LE32>` — **the same
`80`-plus-fixed-width-`u32` shape as the offset field itself**, which is what
makes it a record encoding rather than a displacement guess. `SIZE` is a byte
under `0x80`.

### 3.2 The grid, and the two axes that are NOT `noinline`

`work/w-mmioclose/probe/glgrid.cpp`, nine functions differing only in the
declaration attribute, every body `return 0` so the `.ex` segments are identical
and no body feature can be confounded with the flag:

| cell | ATTR | `0x40` |
|---|---:|---|
| `g_plain`, `g_plain_body`, `use_all` | `0x68` | set |
| `g_inl` (`inline`), `g_finl` (`__forceinline`) | `0xC8` | set |
| `g_static` | `0x48` | set |
| **`g_noinl`, `g_noinl_body`** | **`0x28`** | **clear** |
| **`g_static_noinl`** | **`0x08`** | **clear** |

`0x80` moves with the `inline` keyword and `0x20` with internal linkage, and
`ATTR_STATIC & !0x40 == ATTR_STATIC_NOINLINE` exactly — so the axes are
independent rather than one field, which is the statement that `0x40` is the
inliner's **legality** flag and not a general "this declaration carries an
attribute" marker. `__forceinline` is `0xC8` like `inline`; what separates them
is the NEXT byte (`0x38` against `0x18`), which this lane read and did not ship.

### 3.3 The agreement, stated as an agreement

`WB_INLINE_FINDINGS.md` §1 tabulates c2's inline pass and puts legality at
`0x10b5c06b`: *"refuses on flags `0x400 / 0x1000 / 0x40 / 0x100` at `[sym+0x20]`
and `0x80000 / 0x200` at `[sym+0x4c]`; **requires bit 6 of `[sym+0x4c]`**"*.
Bit 6 is `0x40`. That reading came off the disassembly, from the other side of
the IL seam, before this byte was located — so the two are **independent
derivations of one field** and this lane carries no `DISCLOSURE.md` row, because
nothing was adopted: every value above is read off an obj-and-`.gl` pair this
lane captured.

### 3.4 The fence, and the mis-decode it caught

`gl_function_attrs` returns `None` for the **whole file** on any unrecognised
`SRCPOS`/`SIZE` encoding, on a record with no name run near enough to be its
own, or on one name carrying two different attribute bytes. That is
`Bindings::per_record`'s standard (`w-inlfence`, #2220–#2227) and the direction
is why: the consumer reads *bit clear* as *"c2 keeps this call, so the port may
emit it"*, and a record decoded at the wrong displacement would produce that
reading from an unrelated byte.

**It was not a hypothetical.** The first version of this reader took the escape
for `80 <LE16>` and returned attribute **`0x00`** — bit clear, the permissive
value — for **nine of `mmio.cpp`'s eleven** records. It was caught because the
two records whose `SRCPOS` fits in one byte decoded correctly and the other nine
all came back identical, where the grid predicts a split. **Board #2410.**

### 3.5 What the map is total OVER, stated exactly

Not "every defined function". `codec::gl_offset_framed` requires the record's
token field to read `80 XX 10 00 00 00 00`, i.e. a token whose high byte is
`0x10`. On `src/lazer/game/BustAMovePanel.cpp` — a workload TU with **three**
`__declspec(noinline)` functions — this reader returns **58 records and zero
`noinline`**, because all three carry tokens `0xb24f`, `0x6140`, `0x6135` and
are not framed at all.

**That is the safe direction and it is why the field is an `Option`.** A record
this walk never reaches contributes no entry, its function's `inlinable` stays
`None`, and `None` is required to mean *status quo*, never *permission*. The
limitation is the shipped framing predicate's and is shared with
`gl_defined_names_framed`, which binds the same set. **Board #2411.**

---

## 4. The architectural question, answered

`w-ifn`'s C6: *"Board #139 puts acceptance in the PARSER, and the parser sees
exactly one `.ex` segment. There is no place in the port today where a sibling
function's body can gate parser acceptance."*

**Refuted.** The conflation is between the *body* parser — `parse_segment`,
which does take one segment — and the *acceptance seam*,
`IlBundle::functions()`, which is bundle-level. #139's invariant is the quantity
the scan prints as `fnbyte-census-disagree-expressible`, target **0**: that
acceptance and the census ask ONE question through ONE predicate. It does not
constrain how many segments that question may look at, and four clauses of
`functions()` already look at more than one on master:

| clause | what it reads |
|---|---|
| `drectve_is_boilerplate(gl)` | the whole `.gl` |
| the label-counter gate | every function's framedness, then every non-framed function's stride |
| the unclaimed-`.gl`-symbol accounting | every callee, data symbol and EH unwind callee of **every** function |
| **`callee_defined_here(f, &defined)`** | a set built from **all** the names — `w-inlfence` factored this out (#2220–#2227) precisely so one bundle-level fact could be asked in three places |

So a bundle-level pass that establishes a sibling fact **before** the
per-function loop and consumes it inside is not a violation of #139; it is where
#139's acceptance already lives for this kind of fact. This lane does exactly
that, twice — `let attrs = gl_function_attrs(gl);` above the loop in
`functions()` and above it in `census_functions()` — and each fill is keyed on
**the name that binding bound the function by**: the per-record name in the
gate, `EmitBinding::name` in the census, which is #918's discipline rather than
a shortcut.

**What C6 got right and this lane keeps**: `elide.rs` resolves its sibling
question at *emit* time and is sound there because both of its outcomes are
valid objs. That is not true of the elision or the volatile park, so those must
gate acceptance. The answer is that they *can* — not that they are free.

`fnbyte-census-disagree-expressible` reads **0** at base and tip, which is the
invariant holding across the change rather than being argued about.

---

## 5. Why this is a NARROWING and never a widening

`WB_INLINE_FINDINGS` §7 licenses five narrowings, every one a decline rule, and
says in terms that *"the accept side is not offered"* because a mis-predicted
accept is a wrong obj. Both consumers here consult the accept side, so the
direction has to be stated exactly:

* the prediction is **`Some(false)` ⇒ c2 does NOT expand**;
* what it buys is that the port **keeps a call it was already emitting** — it
  never causes the port to emit a call it was not, and it never causes the port
  to expand anything;
* `None` (unasked) and `Some(true)` leave both consumers **byte-identical** to
  their previous behaviour, and that is asserted by a must-fail cell in each
  (`n7_only_some_false_moves_the_fence`, and the loop at the end of
  `a_noinline_callee_is_not_spliced`). A clause written `!= Some(true)` passes
  the positive and fails those.

The one direction a mis-read could hurt is a **false** `Some(false)` — a record
decoded at the wrong displacement — and §3.4 is the fence for it.

---

## 6. Neutrality, at three levels

### 6.1 The 878 TUs, by name

`work/w-mmioclose/verdicts.txt`, a set comparison over
`src → (class, fn_in_class, fn_total)`:

```
TUs base 878  tip 878
  only-in-base 0   only-in-tip 0   changed 0
```

**Nothing moved in either direction.** No TU left `match`; none arrived; none
became `mismatch` or `codegen-gap`; and no TU's in-class count moved, which is
the statement that this lane ships no reader clause.

### 6.2 Every `gap-metric` key, accounted

`work/w-mmioclose/metricdiff.txt`: **256 keys base, 256 tip, 0 vanished, 0
appeared, 0 changed, 256 identical.** The first-blocker maps
(`keydiff.txt`): **634 body keys and 613 emitted keys, 0 moved on either**,
totals identical to the unit. The dispatch axis likewise: 840 keys, 0 moved.

A lane that ships an emit-path clause and moves **no** key is making a claim
that has to be checkable, and it is: the clause fires on `Some(false)`, and
`Some(false)` occurs **nowhere** in the 878-TU workload that the fences can
reach (§3.5).

### 6.3 All 327 fixtures, at `/O1` AND `/Ox`, against a binary built at the base

The list was regenerated by the script, from the tree, **after** the last change
and `wc -l`-checked: `work/w-mmioclose/fixtures.txt`, **327 lines**, **0** of
them this lane's. Both scans were run twice — once with the tip binary and once
with `work/w-mmioclose/c2rs-base`, built from `git checkout master -- crates/`
**at the rebased tree**, so `master == merge-base` and the base verdict of every
cell is a real counterfactual (§2.1 is what happens when it is not).

| mode | base | tip | moved |
|---|---|---|---|
| `/O1 /Oi /EHsc /GR /GS- /c` | 162 match · 0 mismatch · 9 codegen-gap · 156 vocab-gap | **162 · 0 · 9 · 156** | **0** |
| `/Ox` | 144 · 0 · 17 · 166 | **144 · 0 · 17 · 166** | **0** |

Compared as SETS by name at both modes: `0 only-in-base · 0 only-in-tip · 0
changed`.

---

## 7. `mmio.cpp` — the priced decline, at NINE

Full table in `work/w-mmioclose/MMIO_PRICE2.md`. `w-ifn`'s six are re-derived:
**C1–C5 stand**, **C6 is refuted** (§4), and **three are added that no previous
pricing counted** — and the first of the three is the one that gates everything:

| # | mechanism | status |
|---:|---|---|
| C1 | the `bctrl` encoder | stands; script-counted as the body's only missing mnemonic |
| C2 | an indirect call as a `Selected` shape, with no callee NAME | stands |
| C3 | a bound call statement | stands |
| C4 | a braceless early return on a call result, on `cr0` | stands — and there are **two** in this body, not one |
| C5 | the elision and the volatile park | stands as a mechanism; **its INPUT is paid by this lane** |
| C6 | *"the acceptance seam for C5"* | **REFUTED** — §4 |
| **C7** | **the `.gl` NAME binding**: 10 of 11 names carry no `@@`, and `mmioSeek` / `mmioRead` are exactly 8 characters and hit `INLINE_NAME_MAX` too | **NEW, and it is the FIRST blocker.** Declined: board **#1721** already declined this widening with its reason, and that reason is about **this** TU — a bundle with no mangled name anywhere returns an EMPTY `unclaimed`, so the gate goes **vacuous rather than satisfied** |
| **C8** | the whole-TU inline fence — `mmioClose` calls `mmioFlush`, defined here | **NEW**; unreachable while C7 stands |
| **C9** | a REL24 against a symbol this obj **defines** (`bl mmioFlush` → symbol \[33], `sec=10`) | **NEW**; `introduced_externals` mints undefined externals and nothing mints a self-relocation |

**Not attempted and not guessed at.** No cell of `mmioClose` was written into
`crates/`.

---

## 8. Gate

| lane | result |
|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1,414 passed · 0 failed · 1 ignored · 38 targets** (base **1,406 / 38** — Δ **+8**) |
| 878-TU workload scan | **match 19 · mismatch 0 · codegen-gap 0 · vocab-gap 852 · capture-fail 7** |
| fixtures, `/O1 /Oi /EHsc /GR /GS- /c` | 162 match · **0 mismatch** of 327 |
| fixtures, `/Ox` | 144 match · **0 mismatch** of 327 |
| **`c2rs selftest`** | **327 PASS · 0 ERROR** |
| `scripts/board_audit.sh` | **0 / 0 / 0 / 0 / 0** |
| `scripts/gate.sh --require-graded --jobs 6` | see §8.1 |
| `cargo test -p c2-harness --release --test rung_registry` | **passes** |

The base test count is a **counterfactual**, taken with
`git checkout master -- crates/` at the rebased tree; this lane ships no fixture
and no rung-named fixture, so nothing had to be moved aside for it.

---

## 9. PREREG, scored

`work/w-mmioclose/PREREG.md`, frozen at `b4f5d021`.

### 9.1 The conversion call

| outcome | p | result |
|---|---:|---|
| (A) `match` 19 → 20 | 0.04 | no |
| **(B)** 19 → 19, `mmioClose` declined, first blocker shown to be the `.gl` name binding | **0.62** | **HIT** |
| (C) 19 → 19, declined for `w-ifn`'s reasons, nothing added | 0.10 | no |
| (D) 19 → 19, another TU converts as a side effect | 0.06 | no |
| (E) something else | 0.18 | no |

| # | p | call | outcome |
|---|---:|---|---|
| T1 | 0.03 | `mmioClose` ships byte-exact | **MISS** (declined) |
| T2 | 0.04 | the TU converts | **MISS** |
| **T3** | 0.88 | the first gate blocker is the `.gl` NAME binding | **HIT** — `1 .gl names` against 11 segments |
| **T4** | 0.90 | the six are re-derived and at least one more added | **HIT** — three more (C7, C8, C9) |
| **T5** | 0.75 | C6 is refuted by `functions()`' own existing clauses | **HIT** — four of them |

### 9.2 The `fnbyte-exact` delta — the calibrated metric

| # | p | call | outcome |
|---|---:|---|---|
| **F1** | **0.34** | **+0 exactly** | **HIT** — 35,793 → 35,793 |
| F2 | 0.40 | +1 … +8 | MISS (the registered modal bin) |
| F3 | 0.16 | +9 … +40 | MISS |
| F4 | 0.06 | > +40 | MISS |
| **F5** | 0.04 | negative | **MISS — and it APPEARED to fire.** §2.1: a contaminated counterfactual read −4, and the revert D2 requires was begun before the cause was found. Scored a miss because the tip lost nothing; recorded because for three builds this lane believed it had |
| **F6** | 0.80 | `fnbyte-differs` moves by 0 or downward | **HIT** — unmoved |
| **F7** | 0.95 | `mismatch` 0 everywhere | **HIT** |
| **F8** | 0.70 | both censuses move by 0 | **HIT** |
| F9 | 0.60 | `fnbyte-decline-inlined-callee` falls | **MISS** — 1,003 → 1,003; §3.5 is why |

The point estimate was **+2** and the answer is **+0**. The lane's own §1 says
which number is the claim.

### 9.3 The attribute field

| # | p | call | outcome |
|---|---:|---|---|
| **G1** | 0.90 | a single bit separates the two records | **HIT** |
| **G2** | 0.70 | it is bit 6 (`0x40`) | **HIT**, and it agrees with `WB_INLINE_FINDINGS` §1 |
| **G3** | 0.65 | `inline`/`__forceinline` move a different bit | **HIT** — `0x80`, and they differ from each other in the byte after |
| **G4** | 0.80 | it reproduces on `mmio.cpp`'s own `.gl`, 2 of 11 | **HIT** — 11 of 11 records, 2 clear |
| **G5** | 0.55 | `static` moves a third bit | **HIT** — `0x20` |

### 9.4 Neutrality

| # | p | call | outcome |
|---|---:|---|---|
| **U1** | 0.80 | ≤ 1 arrival, 0 departures, 0 into `mismatch`/`codegen-gap` | **HIT** — 0 changed |
| **U2** | 0.70 | 0 keys vanish, ≤ 2 appear | **HIT** — 0 and 0 |
| **U3** | 0.85 | no fixture moves, list regenerated after the last change | **HIT** — 327 lines, 0 moved at either mode |
| **U4** | 0.90 | `c2rs selftest` stays green | **HIT** — 327 PASS / 0 ERROR |
| **U5** | 0.60 | the first-blocker maps move on 0 keys | **HIT** |
| **U6** | 0.95 | `board_audit.sh` 0/0/0/0/0 and `rung_registry` passes | **HIT** |

### 9.5 The test-count DELTA

| # | p | call | outcome |
|---|---:|---|---|
| **N1** | **0.46** | **+1 … +10** | **HIT — +8** |
| N2 | 0.30 | +11 … +18 | MISS |
| N3 | 0.14 | +19 … +30 | MISS |
| N4 | 0.10 | outside all | MISS |

Registered point estimate **+7**, actual **+8**. **The three-lane
over-estimation streak is broken**, and the correction that broke it is
`w-ifn` §10.6's own: size the number off the emitter's tests plus the mode gate,
not off the cell count. This lane has twelve measured cells behind it (nine grid
+ three attribute axes) and eight tests.

### 9.6 The decline clauses

* **D1** (`mmioClose` is not attempted unless the gate blocker is paid first) —
  **FIRED as registered.** C7 is the gate blocker and is declined at §7.
* **D2** (the bit ships as a narrowing, never a widening; `None` must be
  byte-identical to today) — **held, and asserted rather than claimed**: the
  must-fail cell in each consumer is the `None`/`Some(true)` pair.
* **D3** (the `.gl` NAME binding is not widened) — **FIRED as registered**, with
  board #1721's own reason and this TU as its instance.
* **D4** (a mismatch ⇒ revert to the last committed known-good tree, #1380) —
  **did not fire on a mismatch; the apparent `fnbyte-exact` loss of §2.1 started
  the revert and the tree was already committed at every step**, which is what
  made the three bisections cheap.
* **D5** (no bytes without a grade) — held. This lane changes emitted bytes on
  exactly two shapes and both are graded by real `c2` in
  `noinline_boundary.rs`, at the workload's own profile.
* **D6** (`PORT_CFG_CLASSES` not widened) — held.
* **D7** (one unnamed refusal budgeted) — **SPENT, once**: the framing
  predicate's `0x10` token-high-byte restriction (§3.5). It was not foreseen and
  it is the reason the workload delta is 0.
* **D8** (fence order last; every `_neg` cell's clause key probe-verified with a
  must-fail mutation) — held. `one_unreadable_record_refuses_the_whole_file`
  proves its carrier decodes **before** applying any mutation, so a cell that
  refuses is refusing because of its own mutation; five clauses, five distinct
  refusal causes.

**Score: 20 hits · 8 misses, counted per registered line.** Six of the eight
misses are the losing bins of two mutually exclusive distributions and are
counted rather than dropped. Per *question* — one outcome per distribution — it
is **20 hits · 3 misses** over 23, and the three are `mmioClose` (T1, T2) and
`fnbyte-decline-inlined-callee` (F9).

**Everything registered about the FIELD hit, five for five, including the bit
number.** Everything registered about the field's REACH missed: F2 was the modal
bin and F9 was called at 0.60, and both rest on the assumption that a `noinline`
function in the workload would be one this reader can see. §3.5 is why it is
not, and it is D7's unnamed refusal.

---

## 10. What this lane did NOT do

* It did **not** attempt `mmioClose`, build the `bctrl` encoder, or write an
  indirect-call `Selected` shape. §7.
* It did **not** widen `looks_mangled` or `INLINE_NAME_MAX` (C7). Board #1721's
  decline stands and its stated reason is about this TU.
* It did **not** narrow the PARSER's `callee_defined_here` with the bit. The
  clause is now *available* there — `attrs` is already computed in
  `functions()` — and the population it would convert was not measured, so it is
  left as unminted board work rather than shipped on a guess.
* It did **not** widen `codec::gl_offset_framed`, which is why the workload
  delta is 0 (§3.5). Widening it moves the **gate** binding, not just this
  reader, and that is a rung.
* It did **not** ship the `__forceinline` bit, the `inline` bit or the `static`
  bit, though it measured all three. Only `0x40` has a consumer.
* It shipped **no `DISCLOSURE.md` row** and **no fixture**.

---

## 11. Reproduction

```sh
sh work/w-mmioclose/run.sh sh work/w-mmioclose/scan.sh tip          # 878 TUs
python3 work/w-mmioclose/metricdiff.py work/w-mmioclose/base2.out   work/w-mmioclose/tip.out
python3 work/w-mmioclose/verdicts.py   work/w-mmioclose/base2.jsonl work/w-mmioclose/tip.jsonl
python3 work/w-mmioclose/keydiff.py    work/w-mmioclose/base2.jsonl work/w-mmioclose/tip.jsonl
sh work/w-mmioclose/run.sh sh work/w-mmioclose/probe/cc.sh \
      work/w-mmioclose/probe/inl.cpp /tmp/x /nologo /c /GR /O1 /Oi /EHsc   # 8 expansion cells
sh work/w-mmioclose/run.sh ./target/release/c2rs capture \
      work/w-mmioclose/probe/glgrid.cpp --keep-il DIR \
      --flags-file work/w-mmioclose/o1flags.txt                     # the attribute grid
python3 work/w-mmioclose/probe/glflag.py DIR/_CL_xxxxxxxx 30 all    # the record tails
sh work/w-mmioclose/run.sh cargo test --release -p c2-harness --test noinline_boundary
sh work/w-mmioclose/run.sh sh work/w-mmioclose/tests.sh tests_tip   # --no-fail-fast, targets counted
```
