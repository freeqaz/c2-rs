# WSL — the store whose value is a LOAD, which is every copy assignment (+11,860)

    Tag:       WSL
    Slug:      store-load
    Date:      2026-07-31
    Fixtures:  wsl_store_load.cpp wsl_store_load_neg.cpp
    Census:    639,387 → 651,247 (25.96 % → 26.45 %), +11,860
    Record:    this document

W38 left three residues and ranked this one sixth of six, unsized, with a
warning attached: *the first instrument built for it conflated 165,258
functions, because a store's destination and a load's address are the same
bytes.*

The warning was about the wrong instrument. The row did not need a
first-blocker histogram at all — it needed **one counterfactual scan of the
production itself**, because W38's own `parse_store_stmt` already owned the
destination, the statement list and the tail, and the value position was the
only thing left. The ceiling came back **11,872** and the rung shipped
**11,860** of it, which is **99.90 %**.

## The framing correction, made before the estimate

W38's §8 recorded the sizing instrument's two readings — `54,433` complete with
the value a formal or a literal, and `65,746` complete with the value widened to
"anything `parse_expr` finishes, **plus** an indirect load". Their difference,
**11,313**, is a measured ceiling on the whole value widening, of which this row
is a share. That number was in the record and nobody had read it as a bound.

So the estimate was written against a **counterfactual-successor** bound — the
1.45× kind — and explicitly not against the 67× / 67.8× / 13.4× first-blocker
prior. `work/wsl/ESTIMATE.md` says so in those words, before any scan.

## What it admits

```text
  <dest designator> [2C]                  exactly W38's statement, unchanged
  <SOURCE designator>                     the same two spellings, the same
  ( 33 <int> k 27 <PTR> | 33 <int> k 28 00 00 )*   shared offset-add walk
  30 <TYPE>                               THE LOAD
  [ 2C <same kind, same width> 00 ]       a cv STRIP, which emits nothing
  32 <TYPE>                               the store, restating the post-strip type
  4B
```

**Two instructions, one scratch register, no frame.** MEASURED before any of it
was written (`work/wsl/probe/p1.cpp`, `p2.cpp`, `p4.cpp`, every word read off
the reference obj):

```text
  void c1 (S* d, Q* s) { d->a = s->qb; }   81640004 91630000  lwz r11,4(r4) ; stw r11,0(r3)
  void c1s(S* d)       { d->a = d->b;  }   81630004 91630000  ONE base register
  void c1d(int* d,int* s){ *d = *s;    }   81640000 91630000  the bare deref
  void w_c(W* d, W* s) { d->c = s->c;  }   89640000 99630000  lbz ; stb
  void w_h(W* d, W* s) { d->h = s->h;  }   a1640002 b1630002  lhz ; sth
  void w_q(W* d, W* s) { d->q = s->q;  }   e9640008 f9630008  ld  ; std   (both DS-form)
  void w_f(W* d, W* s) { d->f = s->f;  }   c0040010 d0030010  lfs f0 ; stfs f0
  void w_g(W* d, W* s) { d->g = s->g;  }   c8040018 d8030018  lfd f0 ; stfd f0
  T& operator=(const T& r){a=r.a;b=r.b;return *this;}
                                           81640000 91630000 81640004 91630004  blr
```

### The cv strip is the rung

The first version of this production was byte-complete, graded green on its own
hand-written probe, and **refused every copy assignment and every copy
constructor in the corpus**. A copy assignment takes `const T&`, so the loaded
member is `const int` where the member it lands in is plain `int`, and c1xx
spells the difference as an explicit `2C` between the `30` and the `32`:

```text
  d->a = s->a   const T* s     30 a6 41 86 20  2c 86 41 74 00  32 86 41 74
  d->a = s->a   volatile T* s  30 96 41 8a 20  2c 86 41 74 00  32 86 41 74
  d->c = s->c   const T* s     30 a2 11 8c 20  2c 82 11 70 00  32 82 11 70
  d->g = s->g   const T* s     30 a8 85 8e 20  2c 88 85 41 00  32 88 85 41
```

It emits nothing (`f_const` and `f_plain` are the identical two words). The
formal-valued path requires the store's type to restate the value's **byte for
byte**, and carrying that rule across cost the whole row. The gate that
replaces it is on the **kind byte plus the width**, not on the tag: a `2C` at
this position can also be a real widening (`d->i = s->c` is
`lbz r11 ; extsb r11,r11 ; stw`), and the kind is the type's class and
signedness, which a cv strip does not move and a widening does.

**This was found by the fixture, not by the probe** — the probe used
`S*` sources throughout, and `const` is not an operator, a shape, or an opcode.
It is the axis `scripts/sweep.d/83-store-load.py` §4 now stands on.

### The scratch register is the `/O1` / `/Ox` split, and it was already known

A loaded value has to sit in a register between its load and its store, and
**that register is mode-dependent** — the first time anything in the store
family has been. MEASURED over runs of 1..8 crossed with 2..6 pointer
parameters, in both modes (`work/wsl/probe/p6.cpp` — 40 GPR runs and 10 FP
runs, each compiled twice):

```text
  /O1   every statement    r11         f0
  /Ox   statement i        r(11 - i)   f0, then f(14 - j) for j >= 1
```

with the two register files counted **independently** (`fx3` is `r11 ; f0 ; r10`,
and `MX` is `f0 ; f13 ; f12 ; f11` across both FP widths). This is not a new
fact: `docs/OPT_MODE.md` §3.1 already records the identical allocator for
arithmetic chains — *`/O1` reuses r11 because each intermediate's predecessor is
dead, `/Ox` gives every value its own descending register* — and
`select_function` already threads `OptMode` to every emitter that needs it. The
lowering reuses it rather than restating it.

The descent is a **plain** descent only while it stays above every register a
parameter could hold. Past that, c2 starts skipping and wrapping
(`work/wsl/probe/p7.cpp`):

```text
  L7 (S* d, S* s)          r11 r10 r9 r8 r7 r6 r5              a plain descent
  L8 (S* d, S* s)          r11 … r5, r4      <- r4 is `s`, dead after its own last load
  L9 (S* d, S* s)          r11 … r5, r11, r10                  <- WRAPS instead
  P8 (int,int,S* d,S* s)   r11 … r7, r4, r3, r11   <- SKIPS the two live pointer
                                                     registers, uses the two DEAD int
                                                     ones, then wraps
```

Reconstructing that needs a liveness model of the parameter registers, and
fitting one from those four rows is `GAPS.md` §6 instance #10's mistake. **The
gate is drawn where the descent is still plain** — `n <= 9 - nparams` GPR
statements — which is exactly the region every witness covers, and the parser
states it so census and gate cannot disagree. **Cost of that bound on the
workload: 12 functions**, measured as the difference between the unbounded
ceiling scan and the shipped one.

## Refused, with the measured cost of each refusal

| refusal | why | cost |
|---|---|---|
| a run **mixing** a loaded value with a formal or a literal one | c2 schedules: `{ d->a=s->a; d->b=u; }` is `lwz r11 ; stw r5,4(r3) ; stw r11,0(r3)` — the load hoisted and its store SUNK past the next statement, the two statements in the opposite order to the source. With a literal it additionally takes a **second** scratch register (r10) where a pure run uses only r11. The reverse order happens to survive (`{ d->a=u; d->b=s->b; }` is source order), and two orders of one pair that disagree is not a rule | **358**, measured by lifting the gate in the parser alone and rescanning (651,247 → 651,605), with `store_leaf_text` refusing the mixed groups so the over-claim is the count |
| a run longer than the plain scratch descent | the table above | **12** |
| one object both **loaded from and stored to** in one run | c2 forwards through the pair and eliminates the dead half: `{ d->a=d->b; d->b=d->a; }` is a *single* `lwz r11,4(r3) ; stw r11,0(r3)`. Gated on the TOKEN, because the elimination is a dataflow fact about one object and not about one offset; a run of ONE is unaffected | **0** |
| two statements writing **overlapping bytes** of one destination | inherited from W38 and now covering two opposite behaviours: with formal values c2 eliminates the dead store, with loaded ones it keeps **both** (the source may alias the destination, so the first store is observable). One gate, two lowerings, so it refuses both | **0** |
| a **conversion** on the loaded value | a widening pays an `extsb` (`d->i = s->c`); the narrowing twin `d->c = (char)s->i` is free, so the asymmetry is c2's own and admitting the free direction means deciding it from two type triples | **0** |
| an 8-byte element reached through a **subscript** | the `27` re-types the address to a pointer-to-ARRAY, whose tag width nibble is the POINTER's alignment (4) and not the element's size (8), so the designator contradicts the `30`. The same limit `try_parse_indirect_load_leaf` draws, at the same position, through the same shared walk | **0** |
| a `volatile` pointer **formal** as the source base | a memory object; c2 homes it in the frame. (A pointer *to* volatile is a different bit position and is **free** — both `v_src` and `v_dst` are the bare pair, and both are in the positive fixture) | **0** |
| the source base past the eighth argument | stack-homed, which needs a frame | **0** |

`wsl_store_load_neg.cpp` carries one case per row and censuses **0/11**.

**The under-claiming direction, which is the one nothing here tests.** The
descent bound is drawn for `/Ox` and applied in the parser, which has no mode —
so at `/O1`, where every statement reuses r11 and there is no bound at all, the
port refuses 12 functions it could emit. The whole dc3 workload is `/O1`, so
those 12 are a real, measured, deliberate loss. The alternative is a
mode-dependent census, which is the thing that makes census and gate disagree.

## Two things this rung refutes

**1. W38's mixed-register-file scheduling rule does not generalize to loaded
values.** W38 measured that a run of three or more formal-valued stores mixing
`stw` and `stfs` is *scheduled* (wrong in 16 of 24 mixed triples) and gated on
it. Every mixed-file run of **loaded** values probed here comes back in source
order — `{f,i,q}`, `{i,q,g}`, `{f,g,i}` and the four-statement `{c,f,h,g}` — so
the gate is scoped to non-loaded runs, measured rather than inherited. The two
populations differ in exactly the property the scheduler acts on: a loaded value
is a self-contained load/store pair with no live range crossing another
statement, while a formal sits in an argument register that the run must work
around. **That narrows the open scheduling question** (§Found and not taken #1):
c2's scheduler here is acting on *register pressure between statements*, not on
the statements.

**2. The row was category 1, and the "private limit" was one line.** The
recognizer that already covered the obvious case was `parse_store_stmt`, and
what it refused on the value side was everything that is not a bare `B9` or a
`33`. W38's own correction — *take the recognizer that already covers the
obvious case and list what it refuses on both sides* — is what named this row,
and this time the list had one entry.

## Estimate vs outcome

The estimate was written to `work/wsl/ESTIMATE.md` **before any scan or
instrument**, with the pre-filter named.

**What the bucket had already been filtered by.** To reach the value position a
body must have bound `.sy`, passed `formals_are_one_register_each`, opened on
`B9`/`33`, been declined by the six sibling leaves, had its **destination
designator parse** (`is_ptr4_kind`, the shared offset-add run, the displacement
bound, a base at a register argument position), and had its **tail parse** —
W38 spent the entire statement-list and tail side, which was 81 % of its own
row. Everything cheap was already spent; what was not filtered was the value and
the scratch register.

| | estimate | outcome | bias |
|---|---|---|---|
| the parse ceiling | **≤ 11,313** quoted from W38 §8 as a hard bound | **11,872** | the quoted bound was **4.9 % LOW** — see below |
| the shippable rung | **+4,500**, range 1,000–9,000, biased **LOW** | **+11,860** | **LOW by 2.64×, outside the stated range** |

**Sixth consecutive miss, third consecutive one both in the wrong direction and
outside its own range.** The direction was called correctly for the first time
(the estimate says "expect to be LOW" and names five prior misses as the
reason); the magnitude was not.

Two distinct causes, and only one of them is new:

* **The discount was invented.** The point estimate was `ceiling × 0.65
  (indirect-load share) × 0.674 (W38's realized fraction)`. The second factor is
  **W36's error exactly** — borrowing a sibling rung's rate without checking
  what produced it. W38 realized 67 % of its ceiling because its ceiling was a
  *union* of two independent widenings and it shipped one and a half of them;
  there was no reason for that ratio to transfer, and it did not: this rung
  realized **99.90 %**. A rung that ships one gate against one measured
  counterfactual has no discount at all, and the way to know that in advance is
  to ask *how many independent refusals stand between the counterfactual and the
  emitter* — here, one, and it cost 12.
* **The 11,313 was not the bound it looked like.** W38's instrument measured
  `parse_expr`-completable **plus** indirect-load values together and this rung
  reached 11,872 with the indirect-load half alone. The two are not comparable
  because that instrument did not admit the `2C` cv strip, which is on **every**
  copy assignment in the corpus. A quoted counterfactual is only a bound on the
  production that produced it.

**The generalizable correction**, narrower than "estimate harder": when the
sizing instrument is a *counterfactual of the production you are about to
widen*, the ceiling is the estimate, and the only thing left to estimate is
**how many separate refusals stand between it and the emitter**. Count them —
here it was one (the `/Ox` descent) and it cost 12. The 0.674 discount was
borrowed from a rung that had four.

## Gate evidence

Corpus `dc3-decomp`; baseline re-taken in this worktree and reproducing master
`62ade68` exactly (639,387 / 2,462,571 = 25.96 %, mismatch 0, disagreement 0).

| lane | baseline | WSL |
|---|---|---|
| `cargo test --workspace --release` | 460 pass / 0 fail | **461 pass / 0 fail** |
| `c2rs bench` | 172 / 0 / 0 | **174 / 0 / 0** |
| `scripts/mode_lane.sh /Ox` | 80 match, 0 mismatch, 0 codegen-gap | **81 match, 0 mismatch, 0 codegen-gap** |
| `/O1` · `/O2` · `/Ox /Gy` | 78 match, 0 mismatch, 2 codegen-gap | **79 match, 0 mismatch, 2 codegen-gap** each |
| `scripts/expr_sweep.sh` | 10,996 cases, 0 mismatches | **11,390 cases (+394), 0 mismatches** |
| `scripts/cross_sweep.sh` | 11,761 × 4, 0 mismatches | **12,180 × 4, 0 mismatches** |
| 878-TU scan | 639,387 / 2,462,571 (25.96 %), mismatch 0, disagreement 0 | **651,247 / 2,462,571 (26.45 %)**, mismatch 0, **disagreement 0** |
| `census fixtures/cpp/wsl_store_load.cpp` | — | **40/40 in class**, `Port=Match` |
| `census fixtures/cpp/wsl_store_load_neg.cpp` | — | **0/11 in class**, `Port=NotImplemented` |

The census delta is **1:1** and the two keys are the two spellings of the same
body: `expr-op-0x27` −10,238 (a destination with an offset add, which blocks at
the add) and `expr-op-0x30` −1,622 (a destination without one — `*d = *s` and
`d->a = s->a` at offset 0 — which gets past the add and blocks at the source's
`30`). Their sum is exactly the gain, **no bucket rises**, and no new census key
is created. `expr-intrinsic-base-member-addr` does not move at all, which is
worth stating: W38 took 2,264 out of it and this row takes none, so the two
rungs drained different halves of the same designator family.

The new axis is `scripts/sweep.d/83-store-load.py`, **394 cases**, one file per
axis so it cannot conflict with a peer's fragment. It varies what only a *loaded
value* can vary: run length crossed with parameter count (the axis the `/Ox`
descent lives on, and cosmetic before this rung); the two register files
interleaved inside one run; the source designator crossed against the
destination's in all four spellings; cv-qualification on the source at every
width and mixed within a run; both base slots moved independently; the three
value kinds mixed at every position of a 3-run; and every (source, target)
conversion pair.

## Found and not taken

Ranked. Every figure from `work/wsl/scan-wsl.jsonl` unless stated.

1. **The FP/GPR scheduling rule, the literal-in-a-run rule and the mixed-value
   rule — 12,009 + 358, and still the riskiest thing the store family leaves.** Unchanged in size, but
   **narrowed**: this rung measured that the rule does not apply to loaded
   values at all, so what c2 is scheduling around is register pressure between
   statements and not the statements themselves. That is a better-posed question
   than W38 left, and it is still the one `GAPS.md` §6 instance #10 warns against
   fitting from the data to hand.
2. **The `/Ox` scratch descent past its plain region — 12, and it is a liveness
   model.** Small, and the reason it is listed at all is that the *same* model
   is what the mixed-value scheduling rule (#1) needs. Whoever fits one gets the
   other; nobody should fit either from four rows.
3. **W38's residue #3 — the 5,740 inherited statement gates — is STILL not
   decomposed.** This rung did not touch it: it is the *destination* statement's
   own gates (value width class, type-restates-value, displacement bound,
   argument-register position, FP register resolution) and every one of them now
   has a second consumer on the source side, so decomposing it decomposes twice
   as much as it did yesterday. It was ranked below this row on size (5,740
   against a measured 11,872) and that ranking was correct, but it is now the
   cheapest unexamined thing in the family.
4. **`expr-op-0x27` is 411,967 after this rung** (from 422,205), and 116,118 of
   it is `calls-2plus` — a quarter of the row needs a frame before it needs
   anything else. `expr-intrinsic-base-member-addr` is unchanged at 113,981 with 35,006 `calls-2plus`.
5. **The `0x19` compound assign**, still untouched and still un-sized, exactly
   as W38 left it. Two rungs have now taken 48,544 out of the same two rows
   without going near it.

## Reproduction

```sh
# the lowering, read off the reference obj rather than inferred:
scripts/gt_capture.sh work/wsl/probe/p1.cpp /O1 /GS- /c   # the pair, runs, source order
scripts/gt_capture.sh work/wsl/probe/p2.cpp /O1 /GS- /c   # widths, FP, the two designators
scripts/gt_capture.sh work/wsl/probe/p4.cpp /O1 /GS- /c   # long runs, mixing, aliasing
scripts/gt_capture.sh work/wsl/probe/p5.cpp /O1 /GS- /c   # the cv strip, all widths
scripts/gt_capture.sh work/wsl/probe/p6.cpp /Ox /Gy /GS- /c   # THE ALLOCATOR: 1..8 x 2..6 params
scripts/gt_capture.sh work/wsl/probe/p6.cpp /O1 /Gy /GS- /c   # …and the same in the other mode
scripts/gt_capture.sh work/wsl/probe/p7.cpp /Ox /Gy /GS- /c   # where the descent stops descending
python3 scripts/gt_dump.py work/wsl/probe/p6Ox.obj

# the SIZING, which needed no new instrument — the production is its own:
#   lift the value gate in `parse_store_stmt` only, refuse multi-statement runs
#   in `store_leaf_text`, and take ONE warm scan. The census delta is the parse
#   ceiling and the census/gate disagreement is the run half, in the same run:
#     census 639,387 -> 651,259   (+11,872, the ceiling)
#     disagreement 0 -> 10,890    (the runs; so a SINGLE load-valued store is 982)
#   then ship both halves and the disagreement returns to 0 at 651,247.
```
