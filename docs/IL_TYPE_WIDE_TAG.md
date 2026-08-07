# The type width that was two bytes short, and the 376 blocker rows it invented

**WVB, 2026-07-31.** Census delta **0** — this is a measurement, not a rung, on
the model of `docs/IL_DECODE_REACH.md` and `docs/EH_RECORDS.md` §7. It admits
nothing, it lowers nothing, and every body it newly understands still returns
`NotImplemented`. It belongs in `docs/` proper for exactly that reason.

Corpus: the 878-TU `dc3-decomp` workload, 2,462,571 IL functions, at the
workload's own flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`). Scans
`work/WVB/scan-probe.jsonl` (baseline, `48c1aea`) and `work/WVB/scan-final.jsonl`.

> **Bodies decoded end to end: 2,318,605 (94.2 %) → 2,394,338 (97.2 %), +75,733.**
> **The undecoded residue's distinct keys: 384 → 8.**
> Census numerator **685,882, +0**; census/gate disagreement **0**; `fn_blockers`
> **719 keys → 719, every delta zero**, and `fn_frames` **936 → 936, likewise** —
> so there is nothing to rename and no recorded comparison is invalidated.

---

## 1. What this started as, and why the answer is not where it was looked for

`IL_DECODE_REACH.md` §11.4 ranked one item as *"the only row in this work that
says a width might be incomplete rather than merely unimplemented"*: the 129
bodies filed under **`cf-vbind-type-cflow-jump`**, a `3A` standing exactly where
`9A`'s TYPE should be. If `9A` had a second form, the 69,246 bodies its reading
decodes would be split across two readings and every ranking taken off them would
be suspect.

**`9A` has no second form, and there is no `9A` in those 129 bodies at all.**
The `9A` byte the walk stopped on is the *fourth byte of the preceding type*.

```text
  … BD <A*> 00 <tok>  4C   30 c6 81 46 9a 3a           what the bytes are
  … BD <A*> 00 <tok>  4C   30 c6 81 46 | 9a 3a         where the walk resumed
                              ↑ three bytes read       ↑ `9A` is the vtable bind,
                                where five are there     so this refuses at its
                                                         TYPE, on the `3A`
```

That is the failure mode this project names in `readers.rs` and had never caught
in the act: *a stream read at the wrong alignment lands on a valid opcode by
chance and the census attributes the block to whatever byte it happened to land
on.* The row was **named after the construct standing at the resync point**, and
it contains none of it.

## 2. `<tag with bit 6> <mark> <kind> <LEB id>` — the WIDE type

The `.sy` reader has enforced this since the `.sy` layer first bound on a real TU
(`sy.rs::read_type_prefix`, witnesses `C6 81 06`, `C6 81 03`, `CA 81 0D`; getting
it wrong was measured there as the single largest cause of `.sy` never binding —
197 of 200 TUs). **The `.ex` inline reader did not have it.** One rule, two
locators, and only one of them had it.

### 2.1 Reproduced from hand-written source, with one variable

`work/WVB/probe/p3.cpp`, four lines, at the workload's own flags:

```cpp
struct P { virtual void V(); int q; };   // 8 bytes, polymorphic
struct N { int a, b; N(); };             // 8 bytes, NOT polymorphic
struct D : P, N { D(); };
D::D() {}
```

`D`'s constructor builds both bases in two adjacent statements of the *same*
production, so everything but the keyword is held fixed — including the kind byte
`86` (aggregate, size 8) and the closing `4B`:

```text
  26 <P::P> 33 int 2113 40 <T> 66 02 <D> <P> … BD … 4C   30 c6 81 86 82 20  4B
  26 <N::N> 33 int 2113 40 <T> 66 02 <D> <N> … BD … 4C   30 86    86 93 20  4B
```

**One `virtual` in the source; one byte in the type.** A class with a vtable
spells its type with tag bit 6 set and one extra byte before the kind. Both loads
are closed by the statement's `4B`, and only these two readings land on it — the
width is pinned by the grammar, not by the reader's arithmetic.

The wild bodies bracket it a second way, at the EH-live marker, where
`5C <TYPE> <varint state>` is closed by a `4B`:

```text
  5c c6 81 46 9a 3a 01 4b     TYPE, state 1, end of statement — exact
  5c c6 81 46 | 9a …          state = `9a`, which is not a legal varint at all
```

### 2.2 The mark is a BIT TEST, and the literal `81` is wrong

`.sy`'s reader requires the literal `81`, because all three of its witnesses have
it. `.ex` has a second value. `C6 84 43 <id>` occurs 106 times on the workload,
bracketed twice over in one statement:

```text
  2C c6 84 43 bf 82 01 00   55 c6 84 43 bf 82 01
     └── CONVERT to T ──┘        └── push T ──┘      the same six-byte type
```

Requiring `81` literally refuses **36 bodies that are really there** — measured as
`cf-load-type-0xC6` (18) + `cf-convert-type-0xC6` (18) on a full scan taken
*before* the bit test existed. That is the same shape as `CA 81 0D` refuting the
literal `C6 81` prefix one container over.

Bit 7 is the discriminator and not merely a convenience. `read_type` is also
called **speculatively**, at positions that are not types (a blocker's own naming,
`mcall`'s lookahead), and a bit-6 tag met there is the middle of some other
field's LEB. Over the workload:

Instrumented at the call, over the whole workload — **every** `read_type` call
whose tag has bit 6, decode path and speculation alike:

| second byte | calls | tags carrying it |
|---|---:|---|
| `81` | 213,140 | `C6`, `C7`, `CB` — three, concentrated |
| `84` | 106 | `C6` |
| `01`…`07` | 60,819 | a flat tail across ~50 tags |
| `D9`, `BF`, `3F` | 6 | — |

The `01`…`07` group is the misaligned one and its *tag* distribution says so: a
few hundred calls each across some fifty tags, against two tags carrying 99 % of
the `81` group. **Bit 7 separates them** — which is what a LEB continuation bit
would do if the "tag + mark + kind" framing is really "tag + two-byte kind". Its
value is otherwise not interpreted.

That is a description of the call sites, not a proof, so the claim is settled by
a scan instead: the fully permissive rule (step one byte, whatever it is) decodes
**2,394,338** bodies and the bit-7 rule decodes **2,394,338** — the same number,
to the function. **The `01`…`07` population never contributed a decoded body**, so
the stricter rule costs nothing and refuses more.

## 3. The variant table — one thing changed at a time, on the whole workload

Every row is a full 878-TU scan. `mismatch 0`, census **685,882** and
census/gate disagreement **0** in all of them.

| variant | bodies decoded end to end | vs baseline |
|---|---:|---:|
| baseline `48c1aea` — no wide rule | 2,318,605 (94.2 %) | — |
| wide, mark required literally `81` | 2,394,306 | +75,701 |
| **wide, mark = bit 7 (shipped)** | **2,394,338 (97.2 %)** | **+75,733** |
| wide, mark stepped by width unchecked | 2,394,338 | +75,733 (identical) |
| shipped, but `9A` read as `<TYPE> <varint>` | 2,324,132 | −70,206 vs shipped |

The last row is the answer to the question this work was scheduled for. **The
69,246 do not re-split: they grow to 70,206.** `9A <TYPE>` is confirmed by a
*larger* margin once the bodies around it decode, and the counterfactual reading
decodes nothing the shipped one does not.

## 4. The residue: 384 rows → 8, and two of the retired ones were ranked

`IL_DECODE_REACH.md` §7's table of *"the largest remaining undecoded rows, and
what establishing each would buy"* had six entries. **Three of them do not
exist.**

| row | §7 said | now | what it actually was |
|---|---:|---:|---|
| `cf-expr-0x82` | 23,254 — *"in §13's residue list"* | **0** | the second byte of a type id |
| `cf-expr-0x80` | 14,185 | **0** | likewise |
| `cf-expr-0x05` | 32,755 | 32,872 | real; still the head row |
| `cf-expr-0x59` | 16,016 | 16,033 | real |
| `cf-expr-0x60` | 9,665 | 9,642 | real (`try`/`catch`) |
| `cf-expr-0x08` | 8,242 | 8,248 | real |

and below them, `cf-expr-0x01` (4,192), `0xBF` (2,593), `0xE1` (1,818), `0xDD`
(1,760), `0x9F` (1,757), `0xC1` (1,224), `0x84` (1,174), `0xFF` (1,119), `0xA5`
(828) and **367 more rows** all go to zero. The whole `cf-vbind-type-*` family
(180) and `cf-materialize-type-*` (3) go with them:

> **`IL_DECODE_REACH.md` §5's "the two new productions' own residue is 183 bodies
> … they are honest refusals, not desyncs" is REFUTED. All 183 were desyncs**, and
> not of `67`, `9A` or `64` — of a type width two tokens upstream.

The eight rows that remain:

| row | bodies |
|---|---:|
| `cf-expr-0x05` | 32,872 |
| `cf-expr-0x59` | 16,033 |
| `cf-expr-0x60` | 9,642 |
| `cf-expr-0x08` | 8,248 |
| `cf-expr-0xBC` | 734 |
| `cf-offadd-type-0x86` | 700 |
| `cf-offadd-type-0xA6` | 2 |
| `cf-expr-0x00` | 2 |

68,233 bodies, **2.8 %** of the workload, in eight buckets. Four opcodes are 99 %
of it.

## 5. What the newly-legible bodies are, and the one column that did not move

| EH class | before | after | delta |
|---|---:|---:|---:|
| `eh-none` | 2,009,514 | 2,044,067 | **+34,553** |
| `eh-unknown` | 137,187 | **63,858** | **−73,329** |
| `eh-plus-stmt` | 196,138 | 225,330 | +29,192 |
| `eh-multi` | 35,806 | 47,105 | +11,299 |
| `eh-bare` | 77,147 | 77,836 | +689 |
| `eh-partial` | 6,779 | 4,375 | −2,404 |

**This one goes the other way from WDR's, and the difference is the result.**
WDR's 150,885 newly legible bodies were ≥ 96.4 % `eh-none`; these 75,733 are
**54 % on the EH side** (`plus-stmt` + `multi` + the `partial` that reclassified).
The EH population grows 238,723 → **276,809, +38,086 (+16.0 %)**, against WDR's
+2.2 %. The reason is visible in the byte: a wide tag means *a class with a
vtable*, and a body handling one is a body with sub-objects that have destructors.
`docs/ROADMAP.md` §6o's phase conclusion is reinforced, not weakened — the EH
stock was under-counted by 38,086 while this width was short.

| shape | before | after | of which blocked on control flow ALONE |
|---|---:|---:|---:|
| `cflow-straight` | 1,650,903 | 1,714,090 | 276,271 → **276,271** |
| `cflow-if-1` | 234,254 | 238,766 | 713 → **713** |
| `cflow-if-2` | 28,903 | 29,187 | 0 → 0 |
| `cflow-if-n` | 43,335 | 43,658 | 0 → 0 |
| `cflow-loop` | 83,948 | 91,344 | 0 → 0 |
| `cflow-switch` | 273 | 304 | 5 → **5** |

**The block IR is still worth exactly 718 functions**, and the `+expr-modeled`
column is unchanged *to the function* in every row — the fifth time that column
has survived a decode widening untouched.

## 6. What this says about ranking, and it is the whole point

1. **A first-blocker histogram cannot tell a construct from a resync.** Both are
   "the walk stopped at byte `X`". 376 of 384 rows on this axis were the second
   kind, including the one ranked #2, and no cross with the frame class, the
   control-flow class or the EH class would have said so — a resync passes every
   axis, because every axis is computed *after* it.
2. **The detector is the one WRD wrote down, applied to a byte instead of a key:**
   reproduce it from hand-written source. `cf-expr-0x82` has no source. The
   probe that separates `virtual` from not took four lines and one capture.
3. **"Honest refusal, not desync" is a claim, and §5 of `IL_DECODE_REACH.md` made
   it on the strength of the falsification test** (land on the seven-byte tail,
   every `54 <k>` agreeing). That test is sound and it did its job — those 183
   bodies really did fail it. What the test cannot do is say *where* the desync
   started, and the residue was read as though it could.
4. **One rule, two locators, and only one of them had it.** `.sy` has required the
   wide prefix since it first bound on a real TU; `.ex` never did. Nothing
   compares the two readers, and nothing would have: they are in different
   modules, read different containers, and agree with each other by construction
   on every type that is not wide.

## 7. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | **508 pass, 0 fail** (504 → 508; four new width tests) |
| `c2rs bench` | **191 pass, 0 fail, 0 error** |
| `scripts/mode_lane.sh` `/Ox` · `/O1` · `/O2` · `/Ox /Gy` | **90 / 88 / 88 / 88** match, **mismatch 0** in all four |
| `scripts/expr_sweep.sh` | **13,601 checked, 0 mismatches** |
| `scripts/cross_sweep.sh` | **23,841 × 4, 0 mismatches** |
| 878-TU workload scan | **mismatch 0** · census **685,882 / 2,462,571 (27.85 %)** · **disagreement 0** · binding violations 0 |
| **debug-build** 878-TU scan | **0 panics**, identical to release |
| census key drift | `fn_blockers` **719 → 719, every delta 0**; `fn_frames` **936 → 936, likewise**. **No rename.** |

## 7.1 ADDENDUM 2026-08-08 — §2.1's rule is an `.ex` rule and does NOT carry over to `.gl` (board #1118)

§2.1 above is measured and stands: in `.ex`, `virtual` is the one source change
that sets tag bit 6. **In a `.gl` DATA record it is not that bit's meaning.**
Lane `w-align` (`docs/rungs/2026-08-08-w-align.md` §3) froze 23 cells by
`sha256` before compiling any of them and populated **both** off-diagonal boxes:

```text
                       WIDE                        NOT WIDE
  polymorphic     poly + int/char/char[64]/    poly + double     (88)
                  empty/array/vbase/vdtor      poly + long long  (88)
  NOT poly        __declspec(align(8))         {int,int} · derived · nested ·
                  struct A{int a;}   (C8)      array · double (88) · char (82)
```

So `__declspec(align(8)) struct A { int a; }` is wide with no virtual anything,
and `struct A { virtual void f(); double d; }` is **not** wide. The
co-occurrence that survives all 23 cells is with the type's **required
alignment**, not with polymorphism — and that is a description of 23 cells and
**not a rule**, because no probe there moves a type across the boundary while
holding alignment fixed.

**What §2.1 licensed and still licenses is the WIDTH: one extra byte before the
kind.** That is what `read_type` needs and it is unaffected. What it does not
license is any inference from "wide" to "has a vtable" in another container —
and board #1110's phrase *"the wide **aggregate** form"* is where that inference
was spelled, believed, and (by that lane's own prereg, P7 at 0.75) lost.

The `.gl` width field itself is now read: `align_of_type_tag(tag & !0x40)`,
confirmed 21 of 21 against **c2's own obj** alignment nibbles. §8 item 2's
residual risk — *"the mark byte's meaning is UNKNOWN, and so is its value SET"* —
is honoured there rather than assumed away: the `.gl` alignment reading requires
the mark to be `0x81`, the only value all ten wide cells carry.

## 7.2 ADDENDUM 2026-08-08 — `CA` is taken, `CC`/`CE` exist, and the ORTHOGONALITY rule now carries an unwitnessed arm (board #1120)

`CA` (= 16) is **no longer refused**: lane `w-align16` taught the promotion table
16 and it grades byte-exact through both `data_tu` and `dyninit_tu`. Two things
that grid found which bear directly on this page's subject:

**The width field goes higher than 16.** `__declspec(align(32))` spells **`CC`**
and `align(64)` spells **`CE`**, and c2 gives them `Characteristics` nibbles 6
and 7. The `0x80 + 2*(log2(size)+1)` encoding is therefore confirmed to 64 in
`.gl`, not merely to 16. Both are **refused** — for the grid's coverage, not for
any doubt about the encoding.

**Every 16/32/64 cell is WIDE, and that sharpens §3's co-occurrence without
turning it into a rule.** All twelve go `CA`/`CC`/`CE`; **bare `8A`, `8C` and
`8E` were never produced**, at any of the four profiles, including by a scalar
(`__declspec(align(16)) int g;`) and by a type made 16-aligned through a *member*
rather than through the attribute. A census of all 878 workload TUs finds **0**
records at any of the six tags, out of 85,895 — the workload's `.gl` vocabulary
is `82` (216), `84` (877), `86` (84,334), `88` (33), `C6` (435), and nothing
else. Note that even `C8` — `w-align`'s own `T08`/`T16` — has **zero** workload
witnesses, so the wide form appears there at exactly one width.

So the port's non-wide 16 arm is **the orthogonality rule applied to a shape
nothing has ever emitted**. It is shipped because splitting the table (both forms
at 1/2/4/8, only the wide form at 16) would be a worse hazard than the arm, and
it is labelled as an extrapolation in `align_of_type_tag`'s own doc comment
rather than counted among the confirmations.

## 8. Found and not taken

1. **`cf-expr-0x05` — 32,872 bodies, and it is now 48 % of the whole residue.**
   `IL_STMT_GRAMMAR.md` §5's operator table has `%` at `06` and is silent about
   `05`. One probe per arithmetic operator settles it. Decode-only.
2. **The mark byte's meaning is UNKNOWN, and so is its value SET.** `81` and `84`
   are the only two values this corpus puts in front of a kind, and every `84`
   witness on it is a class-3 (pointer) kind — but no probe here makes a class
   emit `84` on demand, so that is a co-occurrence and not a rule, and the same
   sentence would have been written about `81` before the 106 `84`s turned up. A
   probe that separates them is the honest next step and would also say whether
   the value set is two or merely two-so-far. **This is the residual risk of this
   work**: the width no longer depends on the value, but the fail-closed test does
   (bit 7), and a third value with bit 7 clear would be refused rather than read.
3. **`is_int4_type`'s doc says the kind's high nibble is the object's size.** The
   `.ex`/`.sy` kind correspondence this work walked into says it is a constant
   `4` in `.ex` and absent in `.sy` (`.ex` `41`/`43`/`45`/`46` against `.sy`
   `01`/`03`/`05`/`06`), with the size living in the *aggregate* branch's own
   5-bit field. The predicates are unaffected — they test the pair, and no wide
   type on this corpus passes either — but the comment is describing a
   coincidence, and a rung that widens the operand class should not read it as a
   rule.
4. **`sy.rs::read_type_prefix` still requires the literal `81`.** It is the same
   field and the `.ex` side now knows the value set is larger. Left alone
   deliberately: `.sy` binding is all-or-nothing per file and loosening it is a
   change to which translation units bind at all, which is a measurement of its
   own and not this one.
