# W-DAGPRICE — the `[dag]` band is three claims and only the untested one is wrong; the headline `[O]` 7/50 never depended on it; and the read plan prices at ~10.5 agent-lane hours with a conversion term of zero

    Tag:       w-dagprice
    Slug:      w-dagprice
    Date:      2026-08-29
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization lane (wave 20 brief §2 L4): price the
               `[dag]` long pole by read-before-probe. Settle or refute the band
               attribution, enumerate the read targets with addresses in
               READ_PLAN §3's R1–R9 style, and price each in the unit this
               project has actually observed. Builds no scheduler and writes no
               `crates/` code (decision 20 §2).
    Census:    unchanged → unchanged, +0
    Record:    work/w-dagprice/PREREG.md — committed at `2429a6e61`, the FIRST
               commit on this branch, **before the image was opened**;
               docs/whitebox/WB_DAGPRICE_FINDINGS.md — the record;
               docs/whitebox/ref/P_DAG.md — three amendment boxes, amended
               beside and never rewritten (`ref/README.md` §2.1);
               work/w-dagprice/{band_probe,band_probe2,tu_signal,tu_signal2,
               tu_signal3,rank_artifact}.py, {span2,span3}.sh — every figure
               below is re-runnable from these;
               docs/BOARD.md rows #3838–#3844.
    Reach:     0 — predicted 0, realized 0. No fixture named, no census key
               moved, no `crates/` file touched.
    Byte delta: 0 — required, and shown below.

## What it admits, and what it refuses

**Admits three findings and one price.**

1. **The band claim is three claims** (registered as such in prereg §1 *before*
   the image was opened, so agreeing with it later is not hindsight):
   **FUNCTIONAL** — *these functions are the scheduler* — **survives**, and it
   never rested on `c2_tus.tsv`, which is a file-name partition that has no
   opinion about what a function does. **EXTENT** — *the band's members are the
   scheduler's functions* — is **refuted at both edges**, and had never been
   tested by anyone. **TRANSLATION UNIT** now has **positive evidence** for the
   first time, from an exclusive contiguous `.data` block with a watched
   negative control.
2. **`[dag]`'s headline `[O]` 7 of 50 is `count_marks()` over `P_DAG.md`'s
   prose** and is invariant under any re-attribution of the band. The brief's
   *"the load-bearing assumption under every `[dag]` number this repo
   publishes"* is **false for the headline number** and true for two
   denominators, one of which is wrong for an unrelated reason.
3. **`edge+0x19` bit 1 is cleared at edge creation and has one setter**,
   `FUN_10c1bc78` @ `0x10c1bc78`; `5` is the default and `2` the override; and
   its fourth condition is a caller-supplied record nothing in this repo names.
   Read in-lane in ~15 minutes, which is what makes the plan's row-1 price a
   measurement rather than a guess.

**Refuses**, and every refusal was registered in prereg §4 before it bound:

* **To turn an absence into a positive in either direction.** "No ICE site" is a
  property of the instrument. The TU claim is carried by a positive test with a
  control; the compiland's **name** is reported as unrecoverable, not guessed.
* **To claim any read unblocks F0 or F5.** `P_REGALLOC.md` §7 as amended prices
  F0 at ≥ 10 raw sub-lanes plus two UNPRICED terms and says both published
  figures are floors. That page is outside this lane's seam and is untouched.
* **To restate `STEP5_PRICING_2026-08-21.md` §2.1's figures** (`#3370`). Cited,
  never copied.
* **To grade a scheduler model on this corpus** (`#3435`, `#3728`).
* **To add a `gate.sh` row** (`#3691`).

**And it carries a DECLINE, in prose rather than in the outcome word** (the
brief's own instruction: `declined` is reserved for a lane that declined to
convert a fixture). **This lane declines to recommend funding RD1–RD8 as a
wave.** Ten and a half hours of reading buys eight more `[R]` rows on a page
that already has 43, on a subsystem where the standing measurement is that no
instrument this repo owns can promote one of them. The alternative is named,
cited and — deliberately — **not priced here**: `WB_SCHEDCONF` §8.1's ≈1 d
population that reorders, and `WB_SCHEDCHK` §7.4's DAG walker, which that page
explicitly refuses to price.

## Estimate vs outcome

| | predicted, before building | realized |
|---|---|---|
| band attribution | P1: functional survives, TU does not become a fact, EXTENT is the movable one | **HIT on all three.** Extent refuted at both edges |
| band function count | P2: 13 | **HIT on the digit, and the digit is misleading** — 13 only because the band's end `0x10be663f` catches `FUN_10be663e` by 2 of its 1,197 bytes. Scheduler count is **12** |
| call closure | P3: ≤ 3 of the band have an external caller, `0x10be6382` among them | **BEAT** — exactly **1** of the twelve, and it is `0x10be6382` |
| `[O]` 7/50 invariance | P4: invariant under re-attribution | **HIT**, from `subsys.rs:857-875`. And more than predicted: the metric is **writable by prose**, and this lane moved it **down** |
| edges witnessed | P5: neither edge tightly witnessed | **SPLIT.** The bottom edge is 129 B from `except.c`'s last ICE site (`0x10be5c4d`); the **top** edge is 3,075 B and 30 functions from `emit.cpp`'s first (`0x10be7240`) |
| price direction | *"calibrated multiplier < 1 — the published read prices are systematically pessimistic"* | **HIT**, and quantified: 70×–877× at 24 h/d over the four measurable rows |

**Direction and size of the bias in this lane's own estimate:** the prereg
predicted 6–9 ranked rows and produced 9 (8 reads + 1 explicitly-not-a-read);
it predicted a "not affordable, here is the number" outcome was possible and
that is what happened, but for a reason the prereg did not anticipate — **not
that the reads are expensive, but that they are cheap and buy nothing
gradable.** That is the inversion `#3603`/`#3605` describes, met head-on rather
than in hindsight.

## The ranking's own artifact test — registered, run, published

Prereg §2 registered: *"my ranking is an artifact if the rank order is predicted
by a property of the binary … `|ρ| ≥ 0.7` against size is the artifact
threshold."* `#3505` is six for six on lanes that moved a number by constructing
one.

```
  Spearman rho(rank, named-body bytes) = +0.000  over n=8
  registered artifact threshold: |rho| >= 0.700
  VERDICT: NOT FIRED
```

`work/w-dagprice/rank_artifact.py`. The ranking's **denominator is stated**:
15 candidates considered, 9 ranked, **6 rejected — five of the six because the
read is already taken** (the six negative latency tags, the region rule's
unpinned clauses, the `0x50` cap, `0x10c3afd8`, and the block merger). That
rejection list is the lane's most reusable output: **`[dag]` is not
under-read.**

## The instrument disclosure this lane owes on itself

`agreement` is a mark census of the page's prose (`subsys.rs:857-875`). Editing
`P_DAG.md` therefore moves `[dag]`'s published percentage with no new evidence.
Both values, and the recipe:

```
  BEFORE   [R] 43  [O] 7  [I] 0   total 50   ->  [O] 14.0 %
  AFTER    [R] 46  [O] 7  [I] 0   total 53   ->  [O] 13.2 %

  python3 -c "p=open('docs/whitebox/ref/P_DAG.md').read().split(chr(10));\
  i=[k for k,l in enumerate(p) if l.rstrip()=='---'][0];b=chr(10).join(p[i+1:]);\
  print(b.count('[R]'),b.count('[O]'),b.count('[I]'))"
```

**The headline agreement went DOWN 0.8 points on a lane that settled the band
attribution, refuted its extent, corrected its denominator and read a
previously-unread discriminator.** No `[O]` was lost; three `[R]` sentences were
added. Board **#3841**.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | see below — target count and pass count both recorded |
| `scripts/gate.sh --jobs 16 --require-graded` | see below — the `GATE:` **verdict line**, never the exit code |
| byte delta | **0, required** — `git diff --stat` touches no `crates/**` path; the lane's whole diff is `docs/**` + `work/w-dagprice/**` |
| reach | **0, predicted and realized** — no fixture, no census key, no `DISCLOSURE.md` row |

## Found and not taken

Ranked, with the frame axis applied. Full addresses and citations in
`WB_DAGPRICE_FINDINGS.md` §5.

1. **RD9 — not a read, and it outranks every read.** `WB_SCHEDCONF` §8.1's
   population that reorders (≈1 d, its figure) and `WB_SCHEDCHK` §7.4's DAG node
   walker at the **existing** region hook (unpriced there, unpriced here). With
   the order channel at 8 positions of 3,015, every RD1–RD8 result lands `[R]`
   and stays `[R]` without one of these.
2. **`SUBSYS.md`'s `[dag]` row carries the wrong denominator** (`61`, and its
   `exercised` cell repeats the band caveat this lane splits). **Outside this
   lane's seam** and deliberately untouched — the next lane that owns
   `ref/SUBSYS.md` should carry `#3839` and `#3838` into it.
3. **`ref/FUNCS.tsv` gives `FUN_10be5cbe` to `except.c`/`eh`/`P_EH.md`** on the
   nearest-anchor rule, against a call edge from `0x10c1bbaf`. `P_EH.md` is not
   this lane's; the finding is on the board.
4. **The 29 gap functions above the band own zero exclusive `.data`** and their
   union with `except.c` does not beat `except.c` alone — so their `except.c`
   attribution has support in neither direction. Their cluster's outside caller
   is `0x10c1f1bb`, which is the thread to pull.
5. **`P_DAG` §3's `node+0x4e` bits 2/3** now have their two predicate addresses
   (`FUN_10b32516` → bit 2, `FUN_10b324f9` → bit 3, both from `FUN_10b327cd`),
   and bit 4 is cleared unconditionally at node creation. Recorded in
   `WB_DAGPRICE_FINDINGS.md` §4; not folded into §3's box, which is R7's.
6. **`0x10c1c3f7` (2,716 B) reads `edge+0x19` bit 1** and is the largest
   unattributed consumer of the flag `#3842` reads. Folded into RD5.
