//! **ALLOC** — which register c2 gives each value-producer of a store run.
//!
//! [`schedule`](super::schedule) settles the ORDER of a store run and says so
//! explicitly: *"the allocation is a SECOND INPUT and it is open"*. This module
//! is that second input. `docs/ALLOC.md` is the write-up; the grid is
//! `work/w-alloc/` (gitignored — the generators are committed, the `.cod` and
//! `.obj` are not).
//!
//! Before this module, `crates/c2-il/src/func/body/shapes/leaf_store.rs`
//! carried **four fitted allocation rules, each refuted by one of the others**
//! (use count by `A1`, live-range length by `A2`, last-use by `B6`, first-use
//! by `B4`/`B7`). All four killer cells are derived consequences of the single
//! rule below and are reproduced in the tests.
//!
//! # The rule
//!
//! Enumerate the run's distinct producers. Order them by
//!
//! 1. **use count, descending** — the number of stores that consume the value;
//! 2. on a tie, **register-derived** producers before **constant** ones;
//! 3. on a tie within the register-derived, **source order**;
//! 4. on a tie within the constants, **REVERSE source order**.
//!
//! and hand out the pool registers **descending** — `r11`, `r10`, `r9`, `r8`,
//! … — in that order. The pool is the free volatile registers taken
//! highest-first, minus those holding live-in formals. **`r12` is never used**
//! (board #543 — recorded, not explained).
//!
//! Clauses 3 and 4 carry **opposite signs inside one sort**, which is why the
//! rule is not a priority function. A preregistered exhaustive search over
//! **52,416 priority-function allocators** — 4 scan directions × 3 assignment
//! points × 2 pool walks × 2,184 lexicographic keys over 7 base features —
//! tops out at **179 of 236** fit cells with its residual **exactly** the tie
//! tier, 0 misses at every non-tie count. That negative is the same shape as
//! `w-sched`'s 13,104-configuration result and it is the reason this rule is
//! believable rather than merely fitted.
//!
//! # What this module refuses, and why the refusals are not conservatism
//!
//! * **A multiply producer.** `mulli` is not held live beside another
//!   producer at all: it is materialised one at a time, in `r11`, immediately
//!   before the stores that consume it. `{a=u*3; b=u*5;}` is
//!   `mulli r11 ; stw ; mulli r11 ; stw` — a different regime, measured, not a
//!   counterexample.
//! * **More than [`MAX_MODELLED_PRODUCERS`] producers.** Past three, c2 starts
//!   REUSING a freed register in preference to taking a fresh one, and the two
//!   probed four-producer runs with identical statement structure **disagree**
//!   (`li`-valued reuses `r11`; `addi`-valued takes a fresh `r8`). That is
//!   board #541 and it is open.
//! * **A run mixing constant and register-derived producers.** This refusal is
//!   **load-bearing, and clause 2 is REFUTED on the mixed run** — see
//!   "Clause 2 is refuted" below. It used to read *"clause 2 is measured only
//!   on the supplementary probe, never on the held-out partition, so it is not
//!   shipped"*, which understated it: the clause is not merely untested, it is
//!   wrong.
//! * **A pool too small to serve the run.** Once the volatiles run out c2
//!   descends into registers freed by already-emitted stores — including `r4`
//!   and even `r3`, the base pointer itself — and then into `r30`/`r31` with a
//!   save/restore pair. Open.
//!
//! # Evidence
//!
//! | population | cells | in domain | exact | wrong |
//! |---|---:|---:|---:|---:|
//! | fit | 242 | 236 | **236** | 0 |
//! | **holdout** | 284 | 257 | **250** | **0** (7 refused) |
//! | killer cells | 6 | 6 | **6** | 0 |
//!
//! The holdout partition was declared in
//! `docs/rungs/_2026-08-05-w-alloc-prereg.md` §6 **before** the grid was
//! generated, written by the generator into a file the fitter refuses to open,
//! and scored only after this rule was frozen at commit `8973ffc`.
//!
//! # Clause 2 is REFUTED, and three of the four clauses are unreachable
//!
//! **Board #836.** Lane `w-next` measured 24 mixed-kind cells and fitted a
//! single key — *`uses + (register-derived ? 1 : 0)`, descending* — with 0
//! misses, and deliberately left it unshipped. Lane `w-alloc2` took it to a
//! **fresh** holdout (`work/w-alloc2/freshgrid.py`, 60 cells / 56 graded) and
//! **refuted it on 7**. All 24 fitted cells spell the register-derived producer
//! the same way, `(int)&q`, and the bonus is a property of that spelling rather
//! than of the kind:
//!
//! ```text
//!   addi rX,3,K   (&s->inner, stored INTO s->inner)   1 use   BEATS   li 1 use
//!   add  rX,4,5   (u + v)                             1 use   loses to li 1 use
//!   addi rX,4,5   (u + 5)                             1 use   loses to li 1 use
//!   slwi rX,4,3   (u << 3)                            1 use   loses to li 1 use
//! ```
//!
//! The emitted instruction ORDER is identical in the deciding pair, so this is
//! allocation and not [`super::schedule`]. So **clause 2 as written above — "on
//! a tie, register-derived producers before constant ones" — is false**:
//! `B-notself-1v1` is a register-derived producer at 1 use losing a tie to a
//! constant at 1 use.
//!
//! Over 81 mixed cells graded against real `c2.dll`
//! (`work/w-alloc2/mutate.py`): clause 1 alone is wrong on **29**, clause 2
//! alone on **35**, w-next's key on **20**, and **this module's refusal on 0**,
//! because a refusal is never wrong. That is the whole argument for the refusal
//! staying.
//!
//! **`H-self` is REFUTED too** (board #857, lane w-refbind) — the bonus worth
//! ~1.5 uses attaching to a producer whose value is stored *into the object it
//! points at* scored 1 wrong of 81 on the cells that produced it, and then
//! **11 wrong of 72** on a frozen never-fitted holdout
//! (`work/w-refbind/holdout_pred.tsv`, predictions committed before any cell
//! was compiled). It dies on its *negative* side — `extsh` and a `lwz` load
//! take the bonus register at 1-vs-1 where H-self forbids it everywhere — so
//! it dies independently of the reference-binding axis it was suspected of
//! mismodelling. No allocation key on record survives off its own cells.
//!
//! **The NARROW lift is refused too, and it was measured rather than
//! assumed.** Board **#868**, lane `w-seam`. The remaining way to open
//! `xboxheap.cpp` was to lift the refusal only where **clause 1 decides with no
//! tie** — the register-derived producer at *strictly* more uses than the
//! constant, so no tie-break, no kind bonus and neither refuted key is
//! consulted. 36 cells at the workload's own flags, **36 graded, 0 out of
//! regime, 12 MISS**, every miss a `slwi` cell and the row losing at a
//! use-count advantage of **three** as flatly as at one:
//!
//! ```text
//!   addi-interior  12 / 0 / 0      add  12 / 0 / 0      slwi  0 / 12 / 0
//! ```
//!
//! So there is no threshold to narrow around, and the separating axis is the
//! **spelling** — which [`ProducerKind`] cannot represent.
//! `the_strict_use_count_subcase_is_refused_too` pins the six gaps.
//!
//! # RULE BIND is REFUTED — the seventh, and the first one that was not a key
//!
//! **Board #1067**, lane `w-alloc3`. Every entry above answers *which of two
//! live producers gets `r11`*. This one asked a different question and died
//! anyway, which is why it is worth its own paragraph rather than a row.
//!
//! `w-seq` (#969) dissected 503 splice failures and found every one is a
//! **field** perturbation with no reordering anywhere: 286 source renames
//! `r3 → r4`, 123 destination renames `r3 → r11`, ~92 displacement folds.
//! **RULE BIND** was the obvious reading of that:
//!
//! ```text
//!   BIND  every SOURCE register field still holding a callee formal is
//!         rewritten to the register the caller's actual already lives in.
//!   TEMP  the destination of the instruction producing the callee's return
//!         value stays r3 iff that value is the caller's returned value, and
//!         otherwise becomes POOL_TOP = r11.
//! ```
//!
//! It reproduces, from published bytes and with no toolchain, **all five**
//! recorded witnesses — `w-seq`'s 123 (`?back@?$vector` against `?end@`), its
//! 286 (`?Release@Object@Hmx@@` against `?Release@ObjRef@@`), and its hand
//! cells `s01`, `s03` and `s11` — and it is **33 of 33** on a fit grid.
//!
//! On a **frozen, never-fitted** holdout of 46 cells it is **5 WRONG of 38**
//! in domain (`work/w-alloc3/gridH.tsv`; sources and their `sha256` committed
//! at `5832dd14` before a cell was compiled, the rule frozen at `245945c2`).
//! **The shipped refusal is wrong on 0 of the same 71 cells.**
//!
//! **It dies because c2 does not rename a body — it RECOMPILES an
//! expression.** Four of the five misses are one three-formal callee at the
//! permutations that put its commutative pair the other way round:
//!
//! ```text
//!   int g(int a, int b, int c) { return a - b + c; }   ; sub r11,r3,r4
//!                                                     ; add r3,r11,r5
//!   int f(int x0,int x1,int x2){ return g(x2,x1,x0); } ; H-perm-210
//!      RULE BIND   sub r11,r5,r4 ; add r3,r11,r3      (a renaming)
//!      c2          sub r11,r3,r4 ; add r3,r11,r5      (the callee's own bytes)
//! ```
//!
//! and the fifth is sharper still — `int g(int a){return -a;}` at a site
//! `g(x1) + 4` becomes **`subfic r3,r4,4`**, one word, an opcode that appears
//! nowhere in the callee:
//!
//! ```text
//!      RULE BIND   neg r11,r4 ; addi r3,r11,4         7d6400d0 386b0004
//!      c2          subfic r3,r4,4                     20640004
//! ```
//!
//! So `w-seq` §10.2's caution is now measured rather than argued: **the field
//! diff says WHAT changed and not WHAT DECIDES IT**, and a rule stated as a
//! field edit is a description of the output. The two clauses are not equally
//! dead, and the split is the useful part:
//!
//! * **TEMP survived everything this lane could throw at it.** The result of
//!   an inlined callee lands in `POOL_TOP` = `r11` and nowhere else, at caller
//!   formal counts **1 through 8** (16 of 16 on the holdout's `H-wide`), at
//!   every bound position, with the caller's `r3` provably dead, and even when
//!   the callee already holds a temp in `r11` (4 of 4 on `H-temp`). The rival
//!   *"the temp is the lowest free volatile"* is refuted: at five caller
//!   formals the lowest free volatile is `r8` and c2 emits `lwz r11,4(r7)`.
//!   That is a **direct measurement of `POOL_TOP` in a regime this module has
//!   never been exercised in**, and it agrees with #543/#605.
//! * **BIND is what died**, and only where the caller's expression admits a
//!   different-but-equal encoding.
//!
//! A successor may not restate BIND as a field edit. What it owes first is a
//! decision procedure for c2's **operand canonicalisation** — every one of the
//! five misses is one — and that is a fresh frozen grid, not another pass over
//! these cells (#912's standing lesson).
//!
//! # H-MIX is REFUTED — the eighth, and the first killed by a pair of objs that agree on everything except a SPELLING
//!
//! **Board #1217**, lane `w-mixed`. Every entry above was refuted by a *count*
//! of misses. This one has a two-cell witness that is worth more than its
//! count, because it constrains what any successor may be stated in.
//!
//! The seven above all ask *which of `uses`, `kind` and `first` decides*.
//! H-MIX asked whether the answer is a **threshold in the two use counts**,
//! taking board #892's `cu <= ru + 1` — the best-scoring rule on record, 67 of
//! 77, and the one board **#912** has been asking for a frozen grid for since
//! 2026-08-07 — and adding one term for the spelling `w-spell` calls `2base`:
//!
//! ```text
//!   H-MIX   the producer takes POOL_TOP  iff  cu <= ru + 1 + b
//!             b = 1 when the address-valued stores go through a bound
//!                 reference distinct from the literal stores' base
//!           DOMAIN: two producers, one an interior address that is a PREFIX of
//!           every address it is stored into, one `li`.
//! ```
//!
//! It is **41 of 41** over every in-domain cell of three lanes' committed
//! tables, 15 of them on frozen holdouts, and it repairs every miss those
//! tables record inside its domain — `RULE W2`'s two `self` misses (#891) and
//! `KEY ILX`'s `SELF-2B` miss. On **GRID M**, 70 cells frozen with their
//! `sha256` at `efdcf6e6` before one was compiled, it is **12 WRONG of 62 in
//! domain**. The shipped refusal is wrong on **0 of the same 62.**
//!
//! **It dies because the allocation is decided by the SOURCE SPELLING OF THE
//! VALUE, and not by the value.** These two cells bind the same reference,
//! compute the same address, emit the same instruction, and take different
//! registers — `&q == &t->mid.lo == &t->mid == t+40`, because `lo` is `Q`'s
//! first member:
//!
//! ```text
//!   P& q = t->mid.lo;  q.b0 = (int)&q;        li 11,7 ; addi 10,3,40 ; …
//!   P& q = t->mid.lo;  q.b0 = (int)&t->mid;   li 10,7 ; addi 11,3,40 ; …
//! ```
//!
//! Their **objs differ in eight bytes and every one is a register field**
//! (`work/w-mixed/objdiff.out`, `TimeDateStamp` zeroed — the project's own
//! compare). So no rule stated in [`Producer`]'s fields can separate them:
//! `uses`, `kind` and `first` are equal across the pair, and so is the emitted
//! `addi`. This is board #868's lesson (*"the separating axis is the spelling"*)
//! reproduced **inside** the address class, between two spellings of one
//! address — which #868 could not see, because it varied `addi`/`add`/`slwi`.
//!
//! **The difference IS in the IL**, which is the one piece of good news for a
//! successor (`work/w-mixed/ildiff.out`): the `&q` spelling is a bare
//! `B9 <tok> <TYPE>` with no offset-adds, the `&t->mid` spelling carries
//! `33 <int> <varint 40> 27 <PTR>`. So the fact is readable; it is
//! [`ProducerKind`] that cannot hold it.
//!
//! ## What GRID M measured that is worth keeping
//!
//! | class (w-ilx #909's names) | cells | `cu <= ru+1` |
//! |---|---:|---|
//! | `SELF-1B` — path-spelled value, one base | 31 | **31/31** |
//! | `LOAD` — bind-name-spelled value, no offset-adds | 29 | **29/29** |
//! | `SELF-2B` — path-spelled value, bind base | 2 | **0/2** |
//! | `CROSS` — control, declared out of domain at freeze | 8 | 8/8 |
//!
//! * **`LOAD` and `SELF-1B` are ONE class.** 60 cells, an identical prod/const
//!   frontier at every one of the 21 `(ru, cu)` points, and `cu <= ru + 1`
//!   exact on all 60. `KEY ILX`'s clause 1 (`LOAD` wins iff `cu <= 1`) is
//!   **22 of 29** here, and board **#910**'s *"the `LOAD` class is not a class
//!   either"* is now measured rather than inferred.
//! * **Board #892 is REFUTED and #912 is discharged.** `cu <= ru + 1` is
//!   **60 of 62** on a frozen never-fitted grid — its best score anywhere, and
//!   still a loss to the refusal. It is wrong on exactly the `SELF-2B` pair.
//!   #912 asked for the population `cu` 6–8 at `ru` 2–3; GRID M carries all
//!   five of those points and **every rule on record agrees with the obj
//!   there**, so the population #912 named is not where it dies.
//! * **`always-prod` — `w-heap` §4.1.1's *"the interior address takes the top
//!   of the pool, whatever the use counts are"*, which is what a lane reading
//!   only `xboxheap` would ship — is **44 wrong of 62**.
//! * **The discrimination is not body length.** Every class's `prod` and
//!   `const` cells overlap in store count (5–12 against 7–14); the
//!   `/QXSTALLS` failure cannot be what this is.
//!
//! **A successor may not narrow around `SELF-2B`.** That is a gate drawn around
//! the failing cells, it is how seven of the eight above were written, and
//! `w-mixed`'s own prereg forbade it before the grade. What the residual owes
//! first is a grid of `SELF-2B` **at scale** — the whole world's supply is 15
//! cells across four lanes — varying the bind's displacement, the depth of the
//! value's path, and whether the path's tail agrees with the store's.
//!
//! **And lifting this refusal converts nothing.** `w-mixed`'s P0 ladder
//! (`work/w-mixed/p0/probe.txt`) lifts the reader's mixed-kind clause on
//! `w-carrier`'s own copy of `xboxheap`'s ctor and the body moves to
//! `store-run-bind-call-tail-mr-slot`, then with that lifted too to
//! `store-run-bind-no-emitter-carrier` — and below both sits
//! `super::super::leaf::store`'s `value_bound` refusal, which no reader lift
//! reaches. Board **#1218**: `xboxheap.cpp` prices at **three named reader keys
//! plus one emitter refusal**, not at one.
//!
//! # H-2X is REFUTED — the ninth, and the first to die on the direction its own prereg registered
//!
//! **Board #1227**, lane `w-self2b`. The eight above are each a function of one
//! producer's own fields, or of one expression's own shape. H-2X is the first
//! stated as a **relation between two expressions**, which is what w-ilx §6.1
//! said the carrier would have to be:
//!
//! ```text
//!   H-2X   the address producer takes POOL_TOP  iff  cu <= ru + 1 + d
//!            d = 1 when the ROOT SYMBOL TOKEN of the value expression differs
//!                from the root token of the designator its own stores are
//!                written through
//!          DOMAIN: two producers, one an address that is a PREFIX of (or equal
//!          to) every address it is stored into, the other one `li`.
//! ```
//!
//! It fits **62 of 62** of GRID M in domain, **20 of 20** of GRID V and **all
//! 22** `SELF-2B` cells on record — and `work/w-self2b/PREREG.md` §2.2 says in
//! advance that none of that is evidence, because the *magnitude* +1 is read
//! straight off those cells. What was new was the **predicate**, and it makes a
//! prediction on two classes that did not exist anywhere.
//!
//! On **GRID Z** — 81 cells, `sha256` and every rule's prediction committed at
//! `95839549` **before one cell was compiled**, 81 reached, 81 graded, 0 OOR, 0
//! compile-failed — it is **12 WRONG of 72 in domain**. **The shipped refusal is
//! wrong on 0 of the same 72.**
//!
//! **It dies asymmetrically, and prereg P3 named the cell.** GRID Z completes
//! the 2×2 of (value root, store root) that four lanes had populated in three
//! quadrants:
//!
//! ```text
//!   Z1  path -> path            roots same    store root a formal   cu <= ru+1
//!   Z2  bind -> bind            roots same    store root a BIND     cu <= ru+1
//!   Z3  path -> bind            roots DIFFER  store root a BIND     cu <= ru+2*
//!   Z4  shallower path -> bind  roots DIFFER  store root a BIND     cu <= ru+2*
//!   Z5  bind -> path  (MIRROR)  roots DIFFER  store root a formal   cu <= ru+1
//!   Z6  bind -> 2nd bind        roots DIFFER  store root a BIND     cu <= ru+2*
//! ```
//!
//! `Z5` has differing roots and behaves exactly like `Z1`/`Z2`, so **the
//! relation is not symmetric in the two tokens** and H-2X is wrong on all six of
//! its cells. The other six misses are `*`: see the magnitude, below.
//!
//! **`Z6` costs the successor its fallback.** Two references bound to the *same*
//! object, the value spelled as one bind's name, the stores written through the
//! other — and c2 swaps two registers. The witness is **tighter than #1217's**:
//! `Z2-r2k4` and `Z6-r2k4` emit the *same instruction sequence* and their objs
//! differ in **eight bytes, every one a register field** (`TimeDateStamp`
//! zeroed), while the whole source difference is a **second name** for an object
//! that already had one and that is never used to compute anything:
//!
//! ```text
//!   W& k = d->core.u0;                       k.m0 = (int)&k;   li 11,7  addi 10,3,48
//!   W& k = d->core.u0;  W& j = d->core.u0;   j.m0 = (int)&k;   li 10,7  addi 11,3,48
//! ```
//!
//! So the reading that would fit `SELF-2B` by name — *"the bonus attaches when
//! the value is path-spelled and the stores go through a bind"* — is refuted at
//! **8 wrong of 72**. There is nothing to narrow into.
//!
//! **And `cu <= ru + 2` is REFUTED on fresh `SELF-2B` cells.** Board #1221's
//! clause fits all 22 on record only because no lane's `SELF-2B` cells reach
//! `ru = 1, cu = 3`. GRID Z does, in all three `SELF-2B`-like families, near and
//! far, and it is **`const`** there: **6 wrong of its own 36**, 24 of 72
//! overall. The bonus **vanishes at `ru = 1`**, and that is the other half of
//! H-2X's miss count.
//!
//! **The IL fact, decoded rather than diffed** (`work/w-self2b/roots.out`,
//! through w-ilx's `exdec.py`, which is ported from this crate's own readers).
//! Every designator base is `B9 <tok> <TYPE>`; a bind head is `26 <tok>`:
//!
//! ```text
//!   cell   STORE designator root      VALUE expr root          obj
//!   Z2     tok 0x130a BIND   [0]      tok 0x130a BIND   []     const
//!   Z6     tok 0x140a BIND   [0]      tok 0x130a BIND   []     prod
//!   Z5     tok 0x0e0a formal [48,0,0] tok 0x130a BIND   []     const
//! ```
//!
//! `prod` appears exactly where the store designator's root is a **bind** *and*
//! differs from the value's root. That is a relation between **two** `B9` roots
//! plus one bit about one of them — and [`Producer`] carries `uses`, `kind` and
//! `first`, while `c2-il`'s `eat_offset_adds` returns the **sum** of the
//! offset-add literals rather than the list (#908). **Neither can hold it.** The
//! minimal honest carrier is w-ilx §6.1's: per producer, the `(root token, is a
//! bind, literal list)` of **both** the value and the lvalue.
//!
//! **What a successor may NOT do** is add the asymmetry and the `ru >= 2` guard
//! and call it a rule. That combination is 0 wrong on GRID Z and it has **three
//! conjuncts, two of them read off this grid** — `RULE W2` was 388 of 388 and
//! `RULE BIND` 33 of 33. It is scored in `work/w-self2b/rivals.out`, labelled as
//! having no standing, and what it owes first is the population GRID Z cannot
//! reach: `ru = 1` at `cu = 2`, and `ru` 4–5 at `cu = ru+2` and `cu = 2·ru`,
//! which separate a `ru >= 2` guard from a `cu <= 2·ru` cap from a requirement
//! that the address be live across two of its own stores. No lane has one of
//! those cells.
//!
//! **The bind's own displacement is FREE** — the first axis `w-mixed` §6 said
//! the residual was owed. Moving the bound object from offset 48 to 304 changes
//! no answer, 3 of 3 in every family. So is the value's path **depth** and
//! whether its **tail agrees** with the store's: `Z3` and `Z4` agree at all 9
//! points.
//!
//! **And clauses 2, 3 and 4-for-register-derived are unreachable from the
//! emitter today**, which is why none of this moves a byte:
//! `super::super::leaf::store` builds every [`Producer`] with
//! [`ProducerKind::Constant`], hard-coded, because a store's value there is
//! either a literal or a formal already live in a register. Only clause 1 and
//! clause 4 ever execute. A lane that widens the parser to admit an interior
//! address as a store value makes the mixed run reachable and inherits every
//! paragraph above.
//!
//! # H-2Z is REFUTED — the TENTH, and the first to take the DECODED FACT with it
//!
//! **Board #1243**, lane `w-prod`. `w-self2b` published a rule that is **0
//! wrong on GRID Z's 72**, under a header saying it has no standing, and did
//! not propose it:
//!
//! ```text
//!   H-2Z   the address producer takes POOL_TOP  iff  cu <= ru + 1 + d
//!            d = 1 when  the STORE designator's root token is a BIND
//!                  AND   it differs from the VALUE expression's root token
//!                  AND   ru >= 2
//! ```
//!
//! On **GRID P** — 90 cells, `sha256` and every rival's predictions committed
//! at `b5a20490` **before one cell was compiled**, 90 reached, 90 graded, **0
//! OOR, 0 compile-failed** — it is **3 WRONG of 81 in domain**, and its two
//! declared twins (`cu <= min(ru+2, 2·ru)`, and *"the address must be live
//! across two of its own stores"*) are wrong on the **same three cells**. **The
//! shipped refusal is wrong on 0 of the same 81.**
//!
//! All three misses are `CHAINBIND` — `F& m = k;`, a bind whose base is another
//! bind — which is one of three classes no lane had compiled. `w-prod`'s prereg
//! **P2** registered that direction before the grid was frozen and it landed:
//! `CHAINBIND` agrees with `LOAD` at **9 of 9** points.
//!
//! ## And it takes board #1231's predicate with it
//!
//! This is the part worth more than the count. `w-prod` decoded the `.ex` of
//! one representative per family (`work/w-prod/roots.out`, through `w-ilx`'s
//! `exdec.py`) and **`P6` (`TWOBIND`) and `P7` (`CHAINBIND`) decode
//! identically**:
//!
//! ```text
//!   P6  F& k = h->blk.s0;  F& m = h->blk.s0;  m.n0 = (int)&k;   prod
//!   P7  F& k = h->blk.s0;  F& m = k;          m.n0 = (int)&k;   const
//!
//!   both:  lvalue tok 0x150a BIND [0]     value tok 0x140a BIND []
//! ```
//!
//! **Every field of the carrier `w-self2b` named is equal on both sides, and
//! real `c2` gives them different registers.** So #1231's predicate is refuted
//! on a *decode* and not merely on a source spelling, and **no rule statable
//! over `(root token, is-a-bind, literal list)` of the two sides can separate
//! the pair.** The difference is one level down, in the bind table
//! (`work/w-prod/witness.out`):
//!
//! ```text
//!   P6:  0x150a -> base 0x0f0a [76, 0]     bound to the FORMAL's path
//!   P7:  0x150a -> base 0x140a []          bound to the OTHER BIND
//! ```
//!
//! That is why [`Root::base`] exists. A successor may **not** read
//! `lvalue.base == value.tok` off that pair and call it a rule — it is one
//! witness, read after the grade, which is exactly how the ten keys above were
//! written.
//!
//! **Two further results, and one non-result.** `TWOBIND-swapped` agrees with
//! `TWOBIND` at 9 of 9, so **declaration order does not enter the answer**;
//! `PTRBIND` (a `const` pointer, not a reference) agrees with `SELF-2B` at 9 of
//! 9 while `work/w-prod/bindbit.out` **cannot show its root is a `26` bind
//! head** — reported as a decoder limit and not as a second refutation. And the
//! `ru = 4` / `ru = 5` bands, reached for the first time by any lane, **simply
//! extend** the two fitted frontiers, which is a registered prediction this
//! lane got **wrong**.
//!
//! # The CARRIER exists — and it still ships no rule
//!
//! **Board #1231**, lane `w-prod`. Every one of the nine deaths above is a rule
//! stated in [`Producer`]'s own fields, and `w-self2b` decoded why they all
//! died: the fact is a **relation between two `B9` roots plus one bit about one
//! of them**, and `uses` / `kind` / `first` are facts about *one producer*.
//! **Nine rules were trying to state a relation in a structure that only holds
//! per-producer facts.**
//!
//! [`ProducerRoots`] is that relation — per producer, the
//! `(root token, is-a-bind, offset-add literal list)` of **both** the value and
//! the lvalue — and [`Producer::roots`] carries it. Four statements that could
//! not be written before now can be:
//!
//! ```text
//!   r.roots_differ()                   H-2X's predicate. SYMMETRIC, and wrong
//!                                      on MIRROR — that is why it died.
//!   r.store_root_is_bind()             the SCHEDULE bit of #1235.
//!   r.store_root_is_distinct_bind()    the ALLOCATION bit. A DIFFERENT bit;
//!                                      `Z2` separates them, in both directions.
//!   r.value_offsets_prefix_lvalue()    #908 — `[96]` inside `[96, 4]`, which
//!                                      the sums 96 and 100 cannot state.
//!                                      `None` where only a sum was carried.
//! ```
//!
//! **[`allocate`] does not read any of it, and
//! `allocate_ignores_the_roots_carrier` checks that mechanically.** A carrier is
//! a representation, not a decision: the shipped answer is still the refusal,
//! which is wrong on 0 of every holdout on record while ten fitted keys are
//! wrong on 5 to 42 each. `the_carrier_states_the_decoded_grid_z_table` holds
//! `work/w-self2b/roots.out`'s six graded rows, and
//! `super::leaf::store`'s `the_carrier_decodes_both_roots_of_a_bind_valued_store`
//! reads the same relation off an `IlOp` stream at the emitter's own seam —
//! where **both roots have been live since #1199** and the value's was thrown
//! away by a `..` pattern for want of a field.
//!
//! **One half is honestly missing.** [`Root::offsets`] arrives `None` from
//! today's emitter: `c2_il`'s `eat_offset_adds_list` returns the list, but the
//! seam that would carry it this far is `IlOp::BoundAddr`, whose `off` is a sum.
//! That is **one named field**, not an unmeasurable absence, and a one-element
//! list holding the sum is exactly the lie #908 warns about, so it is not
//! written.

/// How a producer's value is materialised. The distinction is read off the IL,
/// never off the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerKind {
    /// Reads no register: `li`, `lis`+`ori`.
    Constant,
    /// Reads a register: `addi`, `rlwinm`, `add`, …
    RegisterDerived,
    /// A multiply. Its own regime — see the module docs.
    Multiply,
}

/// The root of one `B9 <tok> <TYPE>` designator, decoded from the IL.
///
/// **Board #1231.** Every designator base in the `.ex` is `B9 <tok> <TYPE>`
/// (board #909), optionally followed by a run of offset adds. This is that base
/// and that run, kept apart — which is the whole point, because the fact five
/// lanes have measured is a relation between two of these and not a property of
/// either.
///
/// It carries no verdict and no rule. See [`ProducerRoots`] for what it makes
/// sayable and for the standing prohibition on saying it in [`allocate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    /// The root symbol token.
    pub tok: u32,
    /// Whether that root is a temp **bind head** (`26 <tok>`) rather than a
    /// formal or a local.
    ///
    /// **Board #1128** — *a bind IS a second base symbol* — so a bound
    /// reference's root token is the BOUND LOCAL's own token and never the
    /// formal it hangs off. `crate::codegen::leaf::store` and
    /// `c2_il::IlOp::BoundAddr`'s `tok` are two derivations of that same rule
    /// and cannot disagree.
    pub is_bind: bool,
    /// **What this root is ITSELF rooted at** — the base token of the bind, or
    /// `None` for a root that is not a bind (a formal is rooted at nothing) or
    /// where the reader did not carry it.
    ///
    /// **Board #1244, measured by this lane's own GRID P and not assumed.**
    /// `w-self2b` named the carrier as `(root token, is-a-bind, literal list)`
    /// of both sides, and GRID P contains a pair — `P6-r2k4` / `P7-r2k4`,
    /// `work/w-prod/witness.out` — that is **identical in all six of those
    /// fields on both sides** and that real `c2` gives **different registers**:
    ///
    /// ```text
    ///   P6  F& k = h->blk.s0;  F& m = h->blk.s0;  m.n0 = (int)&k;   prod
    ///   P7  F& k = h->blk.s0;  F& m = k;          m.n0 = (int)&k;   const
    ///
    ///   both:  lvalue tok 0x150a BIND [0]   value tok 0x140a BIND []
    ///   bind table   P6:  0x150a -> base 0x0f0a [76, 0]   (the FORMAL's path)
    ///                P7:  0x150a -> base 0x140a []        (the OTHER BIND)
    /// ```
    ///
    /// So the difference is one level down, in what the store's root is bound
    /// **to**, and no rule stated over the other three fields can separate the
    /// pair. That is board #908's lesson a second time — not one contiguous
    /// field, and not one number either — and it is why this field exists.
    ///
    /// It carries no rule. [`allocate`] does not read it.
    pub base: Option<u32>,
    /// The offset-add literal **LIST**, or `None` where the reader that built
    /// this carried only the list's SUM.
    ///
    /// **`None` is an honest refusal and never a one-element list holding the
    /// sum.** Board **#908**: `c2_il`'s `eat_offset_adds` returns the sum, and
    /// the fact `w-ilx`'s GRID I found — one chain being a byte-exact PREFIX of
    /// another, `[96]` inside `[96, 4]` — is not a function of it.
    /// `c2_il`'s `eat_offset_adds_list` returns the list; the seam that would
    /// carry it this far is `IlOp::BoundAddr`, whose `off` is still a sum, so
    /// today's emitter fills this `None`. That is **one named gap**, not an
    /// unmeasurable one.
    pub offsets: Option<Vec<i32>>,
}

/// **THE CARRIER.** Both sides of the relation, per producer: the root of the
/// value expression, and the root of the designator *this producer's own stores*
/// are written through.
///
/// # Why this type exists
///
/// [`Producer`]'s other fields — `uses`, `kind`, `first` — are facts about **one
/// producer**, and the module docs above record **nine** allocation keys that
/// died trying to state a *relation* in that structure. `w-self2b` decoded the
/// IL rather than diffing objs and found (board **#1231**,
/// `work/w-self2b/roots.out`):
///
/// ```text
///   cell   class          STORE designator root         VALUE expr root      obj
///   Z1     SELF-1B        tok 0x0e0a formal [48, 0, 0]  tok 0x0e0a formal    const
///   Z2     LOAD           tok 0x130a BIND   [0]         tok 0x130a BIND      const
///   Z3     SELF-2B agree  tok 0x130a BIND   [0]         tok 0x0e0a formal    prod
///   Z4     SELF-2B differ tok 0x130a BIND   [0]         tok 0x0e0a formal    prod
///   Z5     MIRROR         tok 0x0e0a formal [48, 0, 0]  tok 0x130a BIND      const
///   Z6     TWOBIND        tok 0x140a BIND   [0]         tok 0x130a BIND      prod
/// ```
///
/// `prod` appears exactly where [`Self::store_root_is_distinct_bind`] holds.
/// `Z5` has differing roots and is `const`, so **the relation is not symmetric
/// in the two tokens** — that is what killed `H-2X`. `Z2` has a bind store root
/// and is `const`, so *"the stores go through a bind"* is not enough either —
/// that is what killed `H-MIX`.
///
/// # What this type is NOT
///
/// **It is not a rule, and [`allocate`] does not read it.** The shipped
/// allocation statement is a refusal and it is wrong on 0 of every holdout on
/// record; ten keys have now been fitted over this fact and every one of them
/// died on fresh cells. `allocate_ignores_the_roots_carrier` pins the
/// separation mechanically rather than by assertion, because "by construction"
/// is the reasoning that let board #232 run 255 commits.
///
/// The carrier's job is to make the fact **expressible and measurable**. A
/// successor that wants to state a rule over it owes a frozen, never-fitted grid
/// containing a class its hypothesis has never seen — which is the one lesson
/// all ten deaths agree on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducerRoots {
    /// The root of the expression that produces the value.
    pub value: Root,
    /// The root of the designator this producer's own stores are written
    /// through. **Not the run's base** — a run may store through several.
    pub lvalue: Root,
}

impl ProducerRoots {
    /// The two root tokens differ. **Symmetric, and therefore NOT the fact** —
    /// `Z5` (`MIRROR`) satisfies this and is `const`. Exposed because `H-2X` was
    /// exactly this predicate and the tenth reader should be able to say so.
    pub fn roots_differ(&self) -> bool {
        self.value.tok != self.lvalue.tok
    }

    /// **The decoded fact of board #1231**: the store designator's root is a
    /// temp bind head **and** differs from the value expression's root.
    ///
    /// Asymmetric on purpose. Swapping `value` and `lvalue` changes the answer
    /// at `Z5`, and that asymmetry is the measured content of the whole rung.
    pub fn store_root_is_distinct_bind(&self) -> bool {
        self.lvalue.is_bind && self.roots_differ()
    }

    /// The **schedule** bit of board **#1235**, which is a *different bit* from
    /// [`Self::store_root_is_distinct_bind`].
    ///
    /// `docs/SYMBOL.md`'s pin (two stores through different base symbols are
    /// never reordered past each other) plus #1128 (a bind is a second base
    /// symbol) decide the ORDER; the allocation needs that **and** the two roots
    /// differing. **`Z2` is the cell that separates them**: it has the bind
    /// schedule and takes the same registers as the interleaving families. A
    /// lane that reads one and infers the other is wrong on `Z2`, in whichever
    /// direction it inferred.
    pub fn store_root_is_bind(&self) -> bool {
        self.lvalue.is_bind
    }

    /// Whether the value's offset-add chain is a byte-exact **PREFIX** of the
    /// lvalue's — `Some(true)` / `Some(false)` — or `None` when either side
    /// carries only a sum.
    ///
    /// **Board #908**, and `None` is the whole reason this returns an `Option`:
    /// `[96]` is a prefix of `[96, 4]` and the sums `96` and `100` cannot say
    /// so, so a carrier that had silently substituted the sum would answer
    /// confidently and wrongly. It refuses instead.
    pub fn value_offsets_prefix_lvalue(&self) -> Option<bool> {
        match (&self.value.offsets, &self.lvalue.offsets) {
            (Some(v), Some(l)) => Some(l.starts_with(v)),
            _ => None,
        }
    }
}

/// One distinct value-producer of a store run.
///
/// **Not `Copy`.** [`Root::offsets`] is a list and #908 is the reason it is a
/// list; `Copy` was worth less than the fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Producer {
    /// Identity. Two statements sharing an `id` share one producer, because c2
    /// CSEs equal constants and equal address binds.
    pub id: u32,
    pub kind: ProducerKind,
    /// How many stores consume this value. **Clause 1 sorts on this**, and it
    /// is the field every one of the four refuted rules got wrong.
    pub uses: usize,
    /// Source index of the first statement naming this value — clauses 3 and 4.
    pub first: usize,
    /// **THE CARRIER** — see [`ProducerRoots`]. `None` where the producer
    /// reached here from a path that decodes no designator, which is every path
    /// in today's emitter that does not go through a store run.
    ///
    /// [`allocate`] does not read this field and a test pins that it does not.
    pub roots: Option<ProducerRoots>,
}

/// The top of the pool. `r12` is never allocated (board #543).
pub const POOL_TOP: u8 = 11;

/// Past three producers c2 begins reusing freed registers and the probed cells
/// disagree. Board #541.
pub const MAX_MODELLED_PRODUCERS: usize = 3;

/// The allocation, or `None` when the run is outside the modelled regime.
///
/// `pool_floor` is the lowest register number free for the whole run — one
/// above the highest register holding a live-in formal. For a run in a
/// function taking `this` plus `n` integer formals that is `4 + n`.
///
/// Returns `(producer id, register number)` pairs, in the order the registers
/// were handed out, so the caller can see both the assignment and the rank.
pub fn allocate(producers: &[Producer], pool_floor: u8) -> Option<Vec<(u32, u8)>> {
    if producers.is_empty() || producers.len() > MAX_MODELLED_PRODUCERS {
        return None;
    }
    if producers.iter().any(|p| p.kind == ProducerKind::Multiply) {
        return None;
    }
    // **A mixed run refuses, and clause 2 is REFUTED rather than merely
    // untested** (board #836). Over 81 mixed cells graded against real `c2`,
    // clause 1 alone is wrong on 29 and clause 2 alone on 35; this refusal is
    // wrong on 0. `the_mixed_refusal_covers_the_measured_refutations` below
    // pins the seven cells any future mixed rule has to reproduce.
    let constant = producers[0].kind == ProducerKind::Constant;
    if producers
        .iter()
        .any(|p| (p.kind == ProducerKind::Constant) != constant)
    {
        return None;
    }
    if pool_floor > POOL_TOP {
        return None;
    }
    if ((POOL_TOP - pool_floor + 1) as usize) < producers.len() {
        return None;
    }

    let mut order: Vec<&Producer> = producers.iter().collect();
    order.sort_by(|a, b| {
        // Clause 1: use count, descending.
        b.uses.cmp(&a.uses).then_with(|| {
            // Clauses 3/4. The tiebreak REVERSES only for a SHARED constant; a
            // count-1 tie runs forward whatever the kind. That sign flip,
            // inside one sort, is what puts the rule outside every
            // priority-key class.
            if constant && a.uses >= 2 {
                b.first.cmp(&a.first)
            } else {
                a.first.cmp(&b.first)
            }
        })
    });
    Some(
        order
            .iter()
            .enumerate()
            .map(|(i, p)| (p.id, POOL_TOP - i as u8))
            .collect(),
    )
}

/// True when every producer of the run lands in `reg`.
///
/// The store emitters put every materialised value in `r11` today, which
/// [`allocate`] confirms is right for a run with **one** producer and wrong for
/// every run with two or more. This is the positive check the emitters call
/// before emitting, so a widening of the *parser* cannot silently turn a clean
/// refusal into a wrong register — board **#232** is the precedent, a parser
/// widening that became a live wrong emit and survived 255 commits.
pub fn all_in(producers: &[Producer], pool_floor: u8, reg: u8) -> bool {
    match allocate(producers, pool_floor) {
        Some(a) => a.iter().all(|&(_, r)| r == reg),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compact spelling: one char per statement, the char naming the value.
    /// `k` picks the kind for every producer in the run.
    fn run(spec: &str, k: ProducerKind) -> Vec<Producer> {
        let mut out: Vec<Producer> = Vec::new();
        for (i, c) in spec.chars().enumerate() {
            let id = c as u32;
            match out.iter_mut().find(|p| p.id == id) {
                Some(p) => p.uses += 1,
                None => out.push(Producer {
                    id,
                    kind: k,
                    uses: 1,
                    first: i,
                    roots: None,
                }),
            }
        }
        out
    }

    fn regs(spec: &str, k: ProducerKind) -> Vec<(char, u8)> {
        let mut a: Vec<(char, u8)> = allocate(&run(spec, k), 4)
            .unwrap()
            .iter()
            .map(|&(id, r)| (char::from_u32(id).unwrap(), r))
            .collect();
        a.sort();
        a
    }

    /// The four allocation rules `leaf_store.rs` records as refuted are all
    /// derived consequences of ALLOC. MEASURED through the real c2 at the
    /// workload's flags — `work/w-alloc/external.py` recompiles every one.
    #[test]
    fn the_four_refuted_rules_killer_cells() {
        // B4  {a=1;b=2;c=3;d=1}  refuted "first-use order"
        assert_eq!(
            regs("1231", ProducerKind::Constant),
            vec![('1', 11), ('2', 10), ('3', 9)]
        );
        // B7  {a=1;b=2;c=3;d=2;e=1}  refuted "use count by A1"
        assert_eq!(
            regs("12321", ProducerKind::Constant),
            vec![('1', 10), ('2', 11), ('3', 9)]
        );
        // A1  {a=1;b=2;c=1;d=2}
        assert_eq!(
            regs("1212", ProducerKind::Constant),
            vec![('1', 10), ('2', 11)]
        );
        // B6  {a=1;b=1;c=2;d=2;e=2}  refuted "last-use"
        assert_eq!(
            regs("11222", ProducerKind::Constant),
            vec![('1', 10), ('2', 11)]
        );
    }

    /// Clause 1 is the USE COUNT and it outranks every tiebreak. These are the
    /// unequal-count patterns that refuted the SHARED-vs-SIMPLE framing the
    /// prereg's H1 was stated in.
    #[test]
    fn clause_one_is_the_use_count() {
        for k in [ProducerKind::Constant, ProducerKind::RegisterDerived] {
            // 0 used twice, 1 used three times -> the busier value takes r11.
            assert_eq!(regs("00111", k), vec![('0', 10), ('1', 11)]);
            assert_eq!(regs("01111", k), vec![('0', 10), ('1', 11)]);
            assert_eq!(regs("011110", k), vec![('0', 10), ('1', 11)]);
        }
    }

    /// Clauses 3 and 4 carry OPPOSITE SIGNS. A count-1 tie runs forward for
    /// both kinds; a count>=2 tie reverses for constants only.
    #[test]
    fn the_tiebreak_sign_flips_on_the_count_and_on_the_kind() {
        // count-1 tie: forward, both kinds.
        assert_eq!(
            regs("012", ProducerKind::Constant),
            vec![('0', 11), ('1', 10), ('2', 9)]
        );
        assert_eq!(
            regs("012", ProducerKind::RegisterDerived),
            vec![('0', 11), ('1', 10), ('2', 9)]
        );
        // count-2 tie: constants REVERSE, register-derived do not.
        assert_eq!(
            regs("0101", ProducerKind::Constant),
            vec![('0', 10), ('1', 11)]
        );
        assert_eq!(
            regs("0101", ProducerKind::RegisterDerived),
            vec![('0', 11), ('1', 10)]
        );
        assert_eq!(
            regs("0011", ProducerKind::Constant),
            vec![('0', 10), ('1', 11)]
        );
        assert_eq!(
            regs("0011", ProducerKind::RegisterDerived),
            vec![('0', 11), ('1', 10)]
        );
    }

    /// The pool is walked highest-first and starts below the live-in formals.
    #[test]
    fn the_pool_starts_below_the_live_in_formals() {
        let r = run("012", ProducerKind::Constant);
        assert_eq!(
            allocate(&r, 4),
            Some(vec![(48, 11), (49, 10), (50, 9)]),
            "with one formal the pool is r11..r5"
        );
        // Six formals hold r4..r9, so only r11 and r10 are free: three
        // producers do not fit and the allocator REFUSES rather than guessing.
        assert_eq!(allocate(&r, 10), None);
        // Two producers do fit.
        assert!(allocate(&run("01", ProducerKind::Constant), 10).is_some());
    }

    /// The refusals, each one a measured different regime rather than caution.
    #[test]
    fn refusals_are_measured_regimes_not_caution() {
        // a multiply is never held live beside another producer
        assert_eq!(allocate(&run("01", ProducerKind::Multiply), 4), None);
        // past three producers c2 reuses a freed register (board #541)
        assert_eq!(allocate(&run("0123", ProducerKind::Constant), 4), None);
        // a mixed run: clause 2 was never held out, so it is not shipped
        let mixed = vec![
            Producer {
                id: 0,
                kind: ProducerKind::Constant,
                uses: 2,
                first: 0,
                roots: None,
            },
            Producer {
                id: 1,
                kind: ProducerKind::RegisterDerived,
                uses: 2,
                first: 1,
                roots: None,
            },
        ];
        assert_eq!(allocate(&mixed, 4), None);
        assert_eq!(allocate(&[], 4), None);
    }

    /// **The seven cells any future mixed-kind rule has to reproduce.**
    ///
    /// Board **#836**. Each row is a mixed run measured against real `c2.dll`
    /// at the workload's own flags (`work/w-alloc2/freshgrid.py`,
    /// `opgrid.py`), and each is a cell where the obvious rule emits the WRONG
    /// register rather than a refusal:
    ///
    /// ```text
    ///   cell              producer            uses  c2 gives r11 to
    ///   F4-add-r1k1       add  rX,4,5   (u+v)  1/1   the CONSTANT
    ///   F4-add-r1k2       add  rX,4,5          1/2   the CONSTANT
    ///   F4-addi-r1k1      addi rX,4,5   (u+5)  1/1   the CONSTANT
    ///   F4-addi-r1k2      addi rX,4,5          1/2   the CONSTANT
    ///   F4-shift-r1k1     slwi rX,4,3   (u<<3) 1/1   the CONSTANT
    ///   F4-shift-r1k2     slwi rX,4,3          1/2   the CONSTANT
    ///   F4-shift-r2k1     slwi rX,4,3          2/1   the CONSTANT  <- clause 1 too
    /// ```
    ///
    /// w-next's key (`uses + register-derived ? 1 : 0`) says the
    /// register-derived producer takes `r11` in all seven. **Shipping it would
    /// have produced wrong bytes, not a refusal** — which is why the refusal
    /// below is the shipped answer.
    ///
    /// This test fails the moment [`allocate`] answers a mixed run, so a lane
    /// that ships one has to come here and state what it measured.
    #[test]
    fn the_mixed_refusal_covers_the_measured_refutations() {
        // (register-derived uses, constant uses) for the seven cells above.
        for &(ru, cu) in &[(1, 1), (1, 2), (1, 1), (1, 2), (1, 1), (1, 2), (2, 1)] {
            let mixed = vec![
                Producer {
                    id: 0,
                    kind: ProducerKind::Constant,
                    uses: cu,
                    first: 0,
                    roots: None,
                },
                Producer {
                    id: 1,
                    kind: ProducerKind::RegisterDerived,
                    uses: ru,
                    first: 1,
                    roots: None,
                },
            ];
            assert_eq!(
                allocate(&mixed, 4),
                None,
                "a mixed run at (reg {ru}, const {cu}) must REFUSE: real c2 \
                 gives r11 to the constant here, and every fitted rule gives \
                 it to the register-derived producer"
            );
            // …and the guard the emitters actually call must decline too.
            assert!(!all_in(&mixed, 4, 11));
        }
    }

    /// **The NARROW lift is refused too, and this pins the grid that killed
    /// it** — lane `w-seam`, board **#868**, `work/w-seam/grida.out`.
    ///
    /// The obvious way to open `xboxheap.cpp`'s configuration is to lift the
    /// mixed refusal only for the sub-case *clause 1 decides with no tie*:
    /// two producers, one register-derived and one single-word constant, with
    /// the register-derived one at **strictly more uses**. No tie-break clause
    /// runs, no kind bonus, neither refuted key is consulted — it looks like
    /// pure clause 1 and therefore like conservatism.
    ///
    /// **It is not.** 36 cells compiled at the workload's own flags and graded
    /// against real `c2.dll` — three spellings × six use-count gaps × two body
    /// kinds (leaf, and a run before a trailing call) — **36 graded, 0 out of
    /// regime, 12 MISS**:
    ///
    /// ```text
    ///   spelling         hit / miss / out-of-regime
    ///   addi-interior    12 /  0 / 0     (int)&q   — xboxheap's own spelling
    ///   add              12 /  0 / 0     (u + v)
    ///   slwi              0 / 12 / 0     (u << 3)  — the CONSTANT takes r11
    /// ```
    ///
    /// The `slwi` row loses at a use-count advantage of **three** (reg 4 uses
    /// against const 1) exactly as flatly as at one, so there is no threshold
    /// the lift could be narrowed around, and both body kinds agree cell for
    /// cell, so a frame does not rescue it either. The separating axis is the
    /// **spelling**, which is [`ProducerKind::RegisterDerived`]'s own blind
    /// spot — the enum cannot represent the distinction the answer turns on.
    ///
    /// A lane that ships the strict-gap sub-case has to come here and say what
    /// it measured that these 36 cells did not.
    #[test]
    fn the_strict_use_count_subcase_is_refused_too() {
        // Every (reg uses, const uses) gap of `work/w-seam/grida.py`, each one
        // a cell where clause 1 alone decides and 12 of 36 graded objs
        // disagree with it.
        for &(ru, cu) in &[(2, 1), (3, 1), (3, 2), (4, 1), (4, 2), (4, 3)] {
            assert!(ru > cu, "the sub-case is a STRICT use-count advantage");
            let mixed = vec![
                Producer {
                    id: 0,
                    kind: ProducerKind::Constant,
                    uses: cu,
                    first: 0,
                    roots: None,
                },
                Producer {
                    id: 1,
                    kind: ProducerKind::RegisterDerived,
                    uses: ru,
                    first: 1,
                    roots: None,
                },
            ];
            assert_eq!(
                allocate(&mixed, 4),
                None,
                "the strict-gap mixed run at (reg {ru}, const {cu}) must \
                 REFUSE: real c2 gives r11 to the CONSTANT for a `slwi` \
                 producer at every one of these gaps (w-seam GRID A, 12 of 36)"
            );
            assert!(!all_in(&mixed, 4, 11));
        }
    }

    /// Three of the four clauses are unreachable from the emitter, and this
    /// pins the reason rather than leaving it to prose: a pure-constant run —
    /// the only kind `leaf::store` can build — never consults clause 2 or
    /// clause 3, so widening the parser is what makes them live.
    #[test]
    fn a_pure_constant_run_is_the_only_kind_the_emitter_can_build() {
        // Same use counts, both kinds. The pure runs answer; the mix refuses.
        assert!(allocate(&run("0011", ProducerKind::Constant), 4).is_some());
        assert!(allocate(&run("0011", ProducerKind::RegisterDerived), 4).is_some());
        let mixed = vec![
            Producer {
                id: 0,
                kind: ProducerKind::Constant,
                uses: 2,
                first: 0,
                roots: None,
            },
            Producer {
                id: 1,
                kind: ProducerKind::RegisterDerived,
                uses: 2,
                first: 1,
                roots: None,
            },
        ];
        assert_eq!(allocate(&mixed, 4), None);
    }

    /// **`POOL_TOP` MEASURED IN A REGIME THIS MODULE HAS NEVER BEEN EXERCISED
    /// IN** — lane `w-alloc3`, board **#1068**, the surviving half of the
    /// refuted RULE BIND.
    ///
    /// Every grid behind this module so far has been a *store run* in a leaf.
    /// `w-alloc3`'s `H-wide` family is a different shape entirely — the single
    /// value an **inlined callee** returns, consumed by one more instruction —
    /// and it lands in `r11` at **every** caller formal count from 1 to 8, at
    /// the first and last bound position, with the caller's `r3` provably
    /// dead. 16 of 16 on a frozen holdout, graded against real `c2.dll`:
    ///
    /// ```text
    ///   int* g(V* v) { return v->b; }                      lwz  r3, 4(r3)
    ///   int* f(int,int,int,int,V* x4){ return g(x4)-1; }   lwz  r11,4(r7)
    ///                                                      addi r3, r11,-4
    /// ```
    ///
    /// With five formals live the **lowest** free volatile is `r8`, so this
    /// separates *"the pool is walked highest-first"* from *"the pool is
    /// walked lowest-first"* on a population that is not a store run at all.
    /// [`allocate`] already answers that way for one producer at every legal
    /// floor, and this pins it so a future edit cannot quietly invert the walk
    /// and stay green — the walk direction has no other test that varies the
    /// floor.
    #[test]
    fn one_producer_takes_pool_top_at_every_floor() {
        for floor in 4..=POOL_TOP {
            let a = allocate(&run("0", ProducerKind::Constant), floor)
                .expect("one producer fits at every floor up to POOL_TOP");
            assert_eq!(
                a,
                vec![(48, POOL_TOP)],
                "a single value takes r11 at pool floor r{floor}, not the \
                 lowest free volatile — w-alloc3 H-wide, 16 of 16"
            );
            assert!(all_in(&run("0", ProducerKind::Constant), floor, POOL_TOP));
        }
        // …and above the top there is no pool, so it refuses rather than
        // reaching for r12 (#543).
        assert_eq!(allocate(&run("0", ProducerKind::Constant), POOL_TOP + 1), None);
    }

    /// **THE PAIR THAT KILLED H-MIX, and the reason no successor may be stated
    /// in [`Producer`]'s fields** — lane `w-mixed`, board **#1217**,
    /// `work/w-mixed/gridM/`, frozen at `efdcf6e6` before a cell was compiled.
    ///
    /// Two sources that bind the same reference, compute the **same address**
    /// (`&q == &t->mid.lo == &t->mid == t+40`, because `lo` is `Q`'s first
    /// member), store it into the same two slots, and emit the same `addi`:
    ///
    /// ```text
    ///   B-2base-r2k4          q.b0 = (int)&q;
    ///       li 11,7 ; addi 10,3,40 ; stw 11,0(3) … stw 10,40(3) ; stw 10,44(3)
    ///   C-2base-r2k4-selfup   q.b0 = (int)&t->mid;
    ///       li 10,7 ; addi 11,3,40 ; stw 10,0(3) … stw 11,40(3) ; stw 11,44(3)
    /// ```
    ///
    /// The objs differ in **8 bytes, every one a register field**, with the
    /// `TimeDateStamp` zeroed (`work/w-mixed/objdiff.out`). Across the pair
    /// [`Producer::uses`], [`Producer::kind`] and [`Producer::first`] are
    /// **equal** — so any rule this module could express gives both cells one
    /// answer, and one of them is wrong bytes.
    ///
    /// The two constructions are therefore built here as the **identical**
    /// producer list, and the assertion is that [`allocate`] refuses it. That
    /// is the refusal doing the only thing that is right on both.
    ///
    /// The distinction is visible in the IL and nowhere in this module: the
    /// `&q` spelling is a bare `B9 <tok> <TYPE>`, the `&t->mid` spelling adds
    /// `33 <int> <varint 40> 27 <PTR>` (`work/w-mixed/ildiff.out`). A successor
    /// has to be stated over *that*, and it owes a `SELF-2B` grid at scale
    /// before it is worth stating at all.
    #[test]
    fn the_same_address_spelled_two_ways_takes_two_registers_so_the_run_refuses() {
        // GRID M's headline pair. `ru = 2` address stores, `cu = 4` literal
        // stores, in both cells — this list is what BOTH compile to here.
        let pair = vec![
            Producer {
                id: 0,
                kind: ProducerKind::Constant,
                uses: 4,
                first: 0,
                roots: None,
            },
            Producer {
                id: 1,
                kind: ProducerKind::RegisterDerived,
                uses: 2,
                first: 4,
                roots: None,
            },
        ];
        assert_eq!(
            allocate(&pair, 4),
            None,
            "real c2 gives r11 to the CONSTANT for `(int)&q` and to the ADDRESS \
             for `(int)&t->mid` — the same address, the same `addi r,3,40`, the \
             same use counts, objs 8 bytes apart and every byte a register \
             field. Any answer here is wrong on one of them (board #1217)"
        );
        assert!(!all_in(&pair, 4, 11));

        // The gate FIRES rather than being satisfied by something else (#1175):
        // drop the kind mix and the very same counts are answered.
        let same_counts_one_kind = vec![
            Producer {
                id: 0,
                kind: ProducerKind::Constant,
                uses: 4,
                first: 0,
                roots: None,
            },
            Producer {
                id: 1,
                kind: ProducerKind::Constant,
                uses: 2,
                first: 4,
                roots: None,
            },
        ];
        assert!(
            allocate(&same_counts_one_kind, 4).is_some(),
            "the refusal above must be the MIXED-KIND clause and not the pool, \
             the producer count or the use counts — all three are unchanged here"
        );
    }

    /// **H-MIX's frontier, pinned so it cannot be re-derived without meeting
    /// GRID M** — board **#1217**, `work/w-mixed/grade.out`.
    ///
    /// `cu <= ru + 1` (board **#892**, and `RULE W2`'s surviving magnitude
    /// clause, since `2ru+3 > 2cu` is the same predicate over the integers) is
    /// **60 of 62** on GRID M — its best score on any population and still a
    /// loss to the refusal, which is 0 wrong of the same 62. Board **#912**
    /// asked for exactly this grid and named `cu` 6–8 at `ru` 2–3 as the
    /// population that would kill it; GRID M carries all five of those points
    /// and every rule on record agrees with the obj there. **It dies somewhere
    /// else**: at `SELF-2B`, `(2,4)` and `(3,5)`, where it says `const` and real
    /// `c2` says `prod`.
    ///
    /// The cells below are the frontier `c2` actually draws for the 60 cells of
    /// `SELF-1B` + `LOAD`, which GRID M found are **one class** and not two
    /// (board #910, now measured). They are pinned as data a successor must
    /// reproduce, and every one of them is REFUSED today.
    #[test]
    fn grid_m_frontier_is_pinned_and_every_cell_of_it_is_refused() {
        // (ru, cu, what real c2 does) — `work/w-mixed/grade.out`, the SELF-1B
        // and LOAD classes, which agree cell for cell at all 21 points.
        const FRONTIER: &[(usize, usize, bool)] = &[
            (1, 1, true),
            (1, 2, true),
            (1, 3, false),
            (1, 4, false),
            (2, 2, true),
            (2, 3, true),
            (2, 4, false),
            (2, 5, false),
            (3, 3, true),
            (3, 4, true),
            (3, 5, false),
            (3, 6, false),
            (4, 4, true),
            (4, 5, true),
            (4, 6, false),
            (4, 7, false),
            // board #912's named population — every rule on record agrees here
            (2, 6, false),
            (2, 7, false),
            (2, 8, false),
            (3, 7, false),
            (3, 8, false),
        ];
        for &(ru, cu, prod_wins) in FRONTIER {
            assert_eq!(
                prod_wins,
                cu <= ru + 1,
                "GRID M's SELF-1B and LOAD classes are exactly `cu <= ru+1`, \
                 60 of 60 — if this line ever fails the table above was edited"
            );
            let mixed = vec![
                Producer {
                    id: 0,
                    kind: ProducerKind::Constant,
                    uses: cu,
                    first: 0,
                    roots: None,
                },
                Producer {
                    id: 1,
                    kind: ProducerKind::RegisterDerived,
                    uses: ru,
                    first: cu,
                    roots: None,
                },
            ];
            assert_eq!(
                allocate(&mixed, 4),
                None,
                "GRID M cell (reg {ru}, const {cu}) must REFUSE — the rule that \
                 fits these 60 is wrong on the SELF-2B pair, and this module \
                 cannot tell the two apart (board #1217)"
            );
        }
    }

    /// GRID Z's **two** frontiers, and the refusal firing on every cell of
    /// both. Lane `w-self2b`, board #1227 — 81 cells frozen with their
    /// `sha256` before one was compiled, 72 in domain, real `c2.dll` under
    /// wibo at the workload's own `/GR /O1 /Oi /EHsc`.
    ///
    /// The table is the measurement, transcribed from
    /// `work/w-self2b/grade.out`. `A` is the frontier of `Z1` (`SELF-1B`),
    /// `Z2` (`LOAD`) and `Z5` (`MIRROR`); `B` is the frontier of `Z3`, `Z4`
    /// and `Z6` (the two `SELF-2B` spellings and `TWOBIND`). **They differ**,
    /// so no single `(ru, cu)` rule is right on both — and `B` is **not**
    /// `cu <= ru + 2`, which is the part board #1221 had wrong.
    ///
    /// This test's job is #1175: prove the gate fires. Every one of the 18
    /// cells below is `None` from [`allocate`], which is the only answer that
    /// is wrong on none of them.
    #[test]
    fn grid_z_two_frontiers_and_the_refusal_fires_on_both() {
        // (ru, cu, A wins with `prod`, B wins with `prod`)
        const FRONTIER: &[(usize, usize, bool, bool)] = &[
            (1, 1, true, true),
            (1, 3, false, false), // <- B is `const`; `cu <= ru+2` says `prod`
            (1, 4, false, false),
            (2, 3, true, true),
            (2, 4, false, true), // <- the deciding band
            (2, 5, false, false),
            (3, 4, true, true),
            (3, 5, false, true), // <- the deciding band
            (3, 6, false, false),
        ];
        let mut a_ne_b = 0;
        let mut cu_le_ru2_wrong_on_b = 0;
        for &(ru, cu, a, b) in FRONTIER {
            assert_eq!(
                a,
                cu <= ru + 1,
                "GRID Z's A frontier (SELF-1B, LOAD and MIRROR — 27 cells) is \
                 exactly `cu <= ru + 1`; if this fails the table was edited"
            );
            if a != b {
                a_ne_b += 1;
            }
            if (cu <= ru + 2) != b {
                cu_le_ru2_wrong_on_b += 1;
            }
            let mixed = vec![
                Producer {
                    id: 0,
                    kind: ProducerKind::Constant,
                    uses: cu,
                    first: 0,
                    roots: None,
                },
                Producer {
                    id: 1,
                    kind: ProducerKind::RegisterDerived,
                    uses: ru,
                    first: cu,
                    roots: None,
                },
            ];
            assert_eq!(
                allocate(&mixed, 4),
                None,
                "GRID Z cell (reg {ru}, const {cu}) must REFUSE — two spellings \
                 of ONE address with equal `uses`, `kind` and `first` take \
                 different registers here (board #1227)"
            );
        }
        assert_eq!(
            a_ne_b, 2,
            "the two frontiers must differ, or GRID Z separated nothing"
        );
        assert_eq!(
            cu_le_ru2_wrong_on_b, 1,
            "board #1221's `cu <= ru+2` must be WRONG on B at (1,3) — that is \
             the point no lane's SELF-2B cells reached"
        );
    }

    /// The guard the store emitters call. One producer is r11 — which is what
    /// the port emits today and why this is inert; two or more never are.
    #[test]
    fn all_in_r11_holds_for_one_producer_and_no_more() {
        assert!(all_in(&run("000", ProducerKind::Constant), 4, 11));
        assert!(all_in(&run("0", ProducerKind::Constant), 4, 11));
        assert!(!all_in(&run("01", ProducerKind::Constant), 4, 11));
        assert!(!all_in(&run("0101", ProducerKind::Constant), 4, 11));
        // out of the modelled regime => not provable => refuse
        assert!(!all_in(&run("0123", ProducerKind::Constant), 4, 11));
        assert!(!all_in(&run("00", ProducerKind::Multiply), 4, 11));
    }

    // ---------------------------------------------------------- THE CARRIER
    //
    // Board #1231. These tests state the decoded fact in the port's own types
    // and pin that `allocate` does not read it. They add no allocation
    // statement; the shipped answer is still the refusal.

    fn formal(tok: u32) -> Root {
        Root { tok, is_bind: false, base: None, offsets: None }
    }
    fn bind(tok: u32) -> Root {
        Root { tok, is_bind: true, base: Some(0xf0a), offsets: None }
    }

    /// **The six rows of `work/w-self2b/roots.out`, in the port's own types.**
    ///
    /// GRID Z, one representative per family at `(ru, cu) = (2, 4)`, decoded
    /// from the `.ex` alone — no obj, no disassembly, no register — and graded
    /// against real `c2.dll` under wibo at the workload's own `/GR /O1 /Oi
    /// /EHsc`. Until this carrier existed the table lived only in a committed
    /// `.out` file, because nothing in `crates/` could hold a row of it.
    ///
    /// The `prod`/`const` column is **c2's answer**, not a prediction. What is
    /// asserted is that [`ProducerRoots::store_root_is_distinct_bind`]
    /// reproduces it on all six — which is the measured content of #1231 — and
    /// that the two symmetric readings that came before it do NOT.
    #[test]
    fn the_carrier_states_the_decoded_grid_z_table() {
        // (cell, class, lvalue root, value root, c2's answer)
        let rows: [(&str, &str, Root, Root, bool); 6] = [
            ("Z1", "SELF-1B", formal(0x0e0a), formal(0x0e0a), false),
            ("Z2", "LOAD", bind(0x130a), bind(0x130a), false),
            ("Z3", "SELF-2B-tail-agrees", bind(0x130a), formal(0x0e0a), true),
            ("Z4", "SELF-2B-tail-differs", bind(0x130a), formal(0x0e0a), true),
            ("Z5", "MIRROR", formal(0x0e0a), bind(0x130a), false),
            ("Z6", "TWOBIND", bind(0x140a), bind(0x130a), true),
        ];

        let (mut sym_wrong, mut bind_only_wrong) = (0, 0);
        for (cell, klass, lvalue, value, is_prod) in rows {
            let r = ProducerRoots { value, lvalue };
            assert_eq!(
                r.store_root_is_distinct_bind(),
                is_prod,
                "{cell} ({klass}): the #1231 predicate must reproduce c2"
            );
            // `H-2X`'s predicate — symmetric in the two tokens.
            if r.roots_differ() != is_prod {
                sym_wrong += 1;
            }
            // `H-MIX`'s — "the stores go through a bind", one token only.
            if r.store_root_is_bind() != is_prod {
                bind_only_wrong += 1;
            }
        }

        // **The asymmetry, as a count.** `Z5` has differing roots and c2 says
        // `const`, so the symmetric reading is wrong on exactly it — that is
        // what refuted `H-2X` on 12 of 72, and it is why `store_root_is_distinct
        // _bind` may not be written as `roots_differ()`.
        assert_eq!(sym_wrong, 1, "the symmetric reading must miss Z5 (MIRROR)");
        // **And one bit about one root is not enough.** `Z2` is a bind store
        // root and c2 says `const` — `H-MIX`, 12 wrong of 62 on GRID M.
        assert_eq!(bind_only_wrong, 1, "the bind-only reading must miss Z2 (LOAD)");
    }

    /// **The relation is NOT symmetric**, stated directly rather than inferred
    /// from a grid row. `Z3` and `Z5` are each other with the two roots
    /// exchanged, and c2 answers differently.
    #[test]
    fn the_carrier_is_not_symmetric_in_the_two_roots() {
        let z3 = ProducerRoots { value: formal(0x0e0a), lvalue: bind(0x130a) };
        let z5 = ProducerRoots { value: bind(0x130a), lvalue: formal(0x0e0a) };
        assert!(z3.store_root_is_distinct_bind());
        assert!(!z5.store_root_is_distinct_bind());
        // both are "the roots differ", which is why that reading cannot work
        assert!(z3.roots_differ() && z5.roots_differ());
    }

    /// **Board #908 — the list, and the refusal when there is only a sum.**
    ///
    /// `[96]` is a byte-exact prefix of `[96, 4]`; the sums 96 and 100 are not
    /// in a prefix relation and nothing recovers one from them. A carrier that
    /// had quietly stored the sum as a one-element list would answer this
    /// question **confidently and wrongly**, so `offsets: None` refuses instead
    /// — and today's emitter fills `None`, which is the one named gap this rung
    /// leaves open.
    #[test]
    fn the_offset_lists_state_a_prefix_and_a_sum_only_carrier_refuses() {
        let with = |v: Vec<i32>, l: Vec<i32>| ProducerRoots {
            value: Root { tok: 1, is_bind: false, base: None, offsets: Some(v) },
            lvalue: Root { tok: 1, is_bind: true, base: Some(0xf0a), offsets: Some(l) },
        };
        assert_eq!(with(vec![96], vec![96, 4]).value_offsets_prefix_lvalue(), Some(true));
        assert_eq!(with(vec![96, 8], vec![96, 4]).value_offsets_prefix_lvalue(), Some(false));
        // equal chains are a prefix of each other — `SELF-1B`'s own shape
        assert_eq!(with(vec![96], vec![96]).value_offsets_prefix_lvalue(), Some(true));

        // sum-only on either side: REFUSED, never guessed.
        let half = ProducerRoots {
            value: Root { tok: 1, is_bind: false, base: None, offsets: Some(vec![96]) },
            lvalue: bind(1),
        };
        assert_eq!(half.value_offsets_prefix_lvalue(), None);
        assert_eq!(
            ProducerRoots { value: formal(1), lvalue: bind(1) }.value_offsets_prefix_lvalue(),
            None
        );
    }

    /// **`allocate` DOES NOT READ THE CARRIER**, checked mechanically.
    ///
    /// The shipped allocation statement is a refusal and it is wrong on 0 of
    /// every holdout on record; **ten** keys have now been fitted over this fact
    /// and every one died on fresh cells. This test exists because "by
    /// construction" is the reasoning that let board #232 run 255 commits: a
    /// successor that wires the carrier into the sort will fail here, loudly,
    /// rather than ship a tenth wrong emit.
    ///
    /// Every row of the GRID Z table is crossed with every producer shape the
    /// module models, and the assignment must be **identical** to the
    /// `roots: None` one in all of them.
    #[test]
    fn allocate_ignores_the_roots_carrier() {
        let carriers = [
            None,
            Some(ProducerRoots { value: formal(0x0e0a), lvalue: formal(0x0e0a) }),
            Some(ProducerRoots { value: bind(0x130a), lvalue: bind(0x130a) }),
            Some(ProducerRoots { value: formal(0x0e0a), lvalue: bind(0x130a) }),
            Some(ProducerRoots { value: bind(0x130a), lvalue: formal(0x0e0a) }),
            Some(ProducerRoots { value: bind(0x130a), lvalue: bind(0x140a) }),
            Some(ProducerRoots {
                value: Root { tok: 7, is_bind: true, base: Some(0xf0a), offsets: Some(vec![96]) },
                lvalue: Root { tok: 9, is_bind: true, base: Some(0xf0a), offsets: Some(vec![96, 4]) },
            }),
        ];
        let mut checked = 0;
        for spec in ["0", "01", "0011", "012", "0101", "11222", "1231"] {
            for k in [ProducerKind::Constant, ProducerKind::RegisterDerived] {
                let base = run(spec, k);
                let want = allocate(&base, 4);
                for c in &carriers {
                    let mut ps = base.clone();
                    // the carrier on EVERY producer, and on ONE of them
                    for p in ps.iter_mut() {
                        p.roots = c.clone();
                    }
                    assert_eq!(allocate(&ps, 4), want, "{spec} {k:?} — all");
                    let mut one = base.clone();
                    one[0].roots = c.clone();
                    assert_eq!(allocate(&one, 4), want, "{spec} {k:?} — first only");
                    checked += 2;
                }
            }
        }
        assert_eq!(checked, 7 * 2 * 7 * 2, "every cross was actually visited");
    }

    /// **Board #1244 — the pair the carrier's first three fields CANNOT
    /// separate, and the field that does.**
    ///
    /// GRID P's `P6-r2k4` and `P7-r2k4` (`work/w-prod/witness.out`), decoded
    /// from the `.ex` alone through `w-ilx`'s `exdec.py` and graded against
    /// real `c2.dll` under wibo at the workload's own `/GR /O1 /Oi /EHsc`:
    ///
    /// ```text
    ///   P6  F& k = h->blk.s0;  F& m = h->blk.s0;  m.n0 = (int)&k;   prod
    ///   P7  F& k = h->blk.s0;  F& m = k;          m.n0 = (int)&k;   const
    /// ```
    ///
    /// Both decode to lvalue `tok 0x150a` BIND `[0]` and value `tok 0x140a`
    /// BIND `[]`. **Every field `w-self2b` named is equal on both sides and c2
    /// takes different registers**, so board #1231's predicate — and every rule
    /// statable over that carrier — is refuted on a *decode*, not merely on a
    /// source spelling. What differs is [`Root::base`]: `m` is bound to the
    /// formal's path in `P6` and to the other bind in `P7`.
    #[test]
    fn the_witness_pair_needs_the_root_s_own_base() {
        // the carrier as `w-self2b` named it — (tok, is_bind, offsets)
        let named = |tok, is_bind| Root { tok, is_bind, base: None, offsets: None };
        let p6_named = ProducerRoots { value: named(0x140a, true), lvalue: named(0x150a, true) };
        let p7_named = p6_named.clone();
        assert_eq!(p6_named, p7_named, "the named carrier cannot tell them apart");
        // …and it answers the same on both, while c2 does not.
        assert!(p6_named.store_root_is_distinct_bind());
        assert!(p7_named.store_root_is_distinct_bind());

        // the carrier WITH the root's own base — `P6` roots `m` at the formal
        // `0x0f0a`, `P7` roots it at the other bind `0x140a`.
        let with = |tok, base| Root { tok, is_bind: true, base: Some(base), offsets: None };
        let p6 = ProducerRoots { value: with(0x140a, 0x0f0a), lvalue: with(0x150a, 0x0f0a) };
        let p7 = ProducerRoots { value: with(0x140a, 0x0f0a), lvalue: with(0x150a, 0x140a) };
        assert_ne!(p6, p7, "the base separates the pair");
        // and the one term that names the difference
        assert_eq!(p6.lvalue.base, Some(p6.lvalue.base.unwrap()));
        assert_ne!(p6.lvalue.base, p7.lvalue.base);
        assert_eq!(p7.lvalue.base, Some(p7.value.tok), "P7's store root is bound to the VALUE's root");
        assert_ne!(p6.lvalue.base, Some(p6.value.tok), "P6's is not");

        // **STILL NOT A RULE.** `lvalue.base == value.tok` is what separates
        // this pair and it is read off this pair, which is precisely how ten
        // keys were written. It is not proposed, and `allocate` does not read
        // `base` any more than it reads the rest.
        let mut a = run("0011", ProducerKind::Constant);
        let mut b = a.clone();
        a[0].roots = Some(p6);
        b[0].roots = Some(p7);
        assert_eq!(allocate(&a, 4), allocate(&b, 4));
    }
}
