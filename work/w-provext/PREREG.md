# w-provext — PREREG

    Lane:     w-provext  (decision 16, `docs/DECISIONS_2026-08-22.md` § Decision 16)
    Kind:     instrument
    Base:     e548f01fdda3f75d14354db3c3f894dca33d5476  (master tip at dispatch)
    Board:    #3648–#3653  (mine, and only mine)
    Fences:   scripts/provenance_census.py · COMMENT-ONLY in crates/c2-il/**,
              crates/c2-obj/**, crates/c2-harness/** EXCEPT src/subsys.rs and
              src/cli/subsys.rs · work/w-provext/** · docs/rungs/2026-08-26-w-provext.md

Committed **before any classification of any constant**. See §0 for the one
thing that was measured first and why that is not a violation.

## 0. What was measured before this file, and why

The brief orders two things whose order conflicts: *"prereg first, committed
before the first count"* and *"re-measure every denominator on your own tree
— the coordinator has NOT verified them."* A denominator **is** a count.

Resolved, and stated rather than smoothed: **the base-tree census was run
before this file was written** (`work/w-provext/census_base.txt`, tree
`e548f01fd`, clean). That run is a **transcription check of the brief's own
figures**, it grades none of this lane's work, and every number in it is
reported below as a *correction to the brief*, not as an outcome. **No
constant was classified, and no marker was written, before this file was
committed.** Every prediction in §2–§4 concerns classifications and deltas
that do not exist yet.

### 0.1 The brief's denominators, re-measured — three of five are wrong

| the brief says | on my base tree `e548f01fd` | verdict |
|---|---|---|
| `crates/c2-il/src/func/**` ≈ **290** | **378** pop / **374** untagged | **WRONG, low by 84.** 290 = the `func` row's 201 untagged + `func/body`'s 89. It **omits `func/body/shapes` entirely (84 items)**. The `**` in the brief's own path says those are in scope |
| `crates/c2-obj/src` = **43** | **43** pop / **43** untagged | confirmed |
| `crates/c2-harness/**` = **214** | **221** pop / **221** untagged | **moved**: master added 6 items in `src` and 1 in `cli` after the wave-13 tip. Net of the two files fenced to peer `w-encmap` (`src/subsys.rs` 6, `src/cli/subsys.rs` 1) my writable scope is **214** — the same number by coincidence, from a different arithmetic |
| total population **1,051** | **1,058** | **moved by +7**, same cause |
| tagged **189** | **189** | confirmed; coverage is therefore **17.87 %**, not 18.0 % |

**My writable scope is 650 untagged items**: 374 (`c2-il/src/func/**`) + 19
(`c2-il/src` — in my fence, and *not named by the brief at all*) + 43
(`c2-obj/src`) + 214 (`c2-harness/**` net of the peer's two files). The
brief's own three numbers sum to 547; the fence it wrote is wider than the
numbers it quoted, in two places.

## 1. What I am building

**(A)** `provenance_census.py --since <sha>`: a two-tree diff, per module,
reporting markers added/removed **by class**, population change, and coverage
change. Decision 15's doctrine is that the tracked signal is the **CHANGE**;
the tool can only print a **LEVEL**.

**(B)** Tagging the residue: `c2-il/src/func/**` first (the brief's stated
priority and the population that tests wave 13's reading), then `c2-obj/src`,
then `c2-harness/**` minus the peer's two files.

## 2. The classification rule, pre-committed BEFORE any constant is classified

This is the load-bearing part of the prereg. **I am both the classifier and
the scorer of P7 below**, so a prediction about the resulting distribution is
worthless unless the rule that produces it is fixed first. It is fixed here:

| mark | I will use it when, and only when |
|---|---|
| `[R]` | an address in `c2.dll`'s image established this value, and I can cite it |
| `[O]` | the value was established from a real c2 artifact — an obj, a `/FAsc` listing, a `.gl`/`.ex`/IL capture — **and no other value is consistent with the cells named**. For `c2-il` this is the common case for *container/record* facts: the IL is c2's input and was read out of captures |
| `[F]` | the value is a **parameter of a rule, chosen to agree with a set of observations, and it has an off-sample failure mode those observations could not see**. DISCLOSURE's own discriminator, applied unchanged. **Every admission threshold, budget, cap, depth limit and shape bound in `c2-il` is `[F]` by this test unless a read or a boundary-pinning grid says otherwise** — that is the honest reading and I am registering it before I count |
| `[S]` | a published external standard (PE/COFF, PowerPC ISA, MSVC EH format) would give the same value if `c2.dll` had never existed |
| `[N]` | port-internal nomenclature (a census-key string, a diagnostic tag), a key-packing layout the port invented, a scratch/loop bound that reaches no emitted byte and no published verdict, a sentinel, or a value derived from another marked constant — **always with the reason in the citation** |

**The `[N]` call I expect to be argued with, registered now:** `c2-il` holds a
large population of `pub(crate) const FOO: &str = "foo-key"` census-key names
and a bit-packing layout for those keys (`mcall.rs:153-211`). Their *value*
is a label the port chose; changing it renames a histogram row and moves no
byte and no verdict's content. I will tag these **`[N]` with that reason**.
If a future reader thinks a key name is load-bearing, the marker is where the
argument happens — which is the point of tagging it rather than leaving it
silent.

## 3. Predictions — per-module splits at my tip

Stated as counts with a **bias direction** each, per the brief.

### P1 — `crates/c2-obj/src` (pop 43)

| `[R]` | `[O]` | `[F]` | `[S]` | `[N]` | untagged |
|---:|---:|---:|---:|---:|---:|
| 0 | 3 | 0 | 39 | 1 | 0 |

**Bias: I expect to be wrong toward MORE `[O]`.** `.text$` section-name
prefixes and alignment conventions are c2's choices *within* the spec, and I
will find more of those than I now think. `[R] = 0` is a **prediction of the
second zero-`[R]` calibration pole** the brief asked for, beside `#3633`'s
COFF writer.

### P2 — `crates/c2-il/**` (pop 397 = 19 + 378)

| `[R]` | `[O]` | `[F]` | `[S]` | `[N]` | untagged |
|---:|---:|---:|---:|---:|---:|
| 2 | 45 | 90 | 0 | 235 | 25 |

**Bias: I expect to be wrong toward MORE `[N]` and FEWER `[F]`** — the
`[N]` population (key strings, key packing, diagnostic ladders) is the one I
have sampled and it is larger than it looks from a name list.

### P3 — `crates/c2-harness/**` (pop 221, writable 214)

| `[R]` | `[O]` | `[F]` | `[S]` | `[N]` | untagged |
|---:|---:|---:|---:|---:|---:|
| 0 | 30 | 25 | 2 | 130 | 34 |

**Bias: wrong toward MORE `[N]`.** These are instrument definitions and
fixture names; the `[N]` clause was written for exactly this crate. The 7
untagged in the two peer-fenced files are inside the 34.

### P4 — total coverage at my tip

**Predicted 66 %** (≈ 700 of ≈ 1,058 tagged), range **58–74 %**.
**Bias: wrong LOW** — the block form makes homogeneous files nearly free.

## 4. The interesting registered prediction — P7

> **Does `[F]` become the dominant class in `c2-il`?**

**I predict NO — `[N]` will dominate `c2-il`, with `[F]` second.**

Wave 13's reading of its own P9 miss (`[R]` 100 vs `[F]` 4) was that *"`[F]`
is under-counted because the untagged 862 are untagged"* and that `c2-il`'s
decode vocabulary *"is where `[F]` should actually live"*. I am registering
that **this is half right and half wrong, and the half that is wrong is a
second instance of the same error**:

- **Right:** `[F]` will rise from **0** in `c2-il` to a substantial block —
  the admission thresholds, the shape caps, the token budgets. Wave 13 could
  not see them because it had no ledger to seed them from, exactly as it said.
- **Wrong:** it will still not dominate, because the module's *population* is
  not mostly parameters. It is mostly **names** — census keys and their
  packing. Reading "`c2-il` is where `[F]` lives" as "so `[F]` will dominate
  `c2-il`" substitutes a claim about a **subset** for a claim about a
  **denominator**, which is `#3045`'s shape one instrument over.

**Grading:** if `[F]` is the largest class in `c2-il` at my tip, P7 is a
**MISS** and I will say MISS in that word, and wave 13's reading is
vindicated over mine. If `[N]` is largest and `[F]` second, P7 is a hit. Any
other order is a miss.

## 5. Predictions about the `--since` mode

- **P5:** running `--since e548f01fd` at my own tip will report **0 items
  removed and 0 items added** in `c2-il/src/func/**`, `c2-obj/src` and the
  `c2-harness` rows — i.e. the entire delta there decomposes as
  **retagged (untagged → tagged)** and nothing else, because my edits are
  comment-only by construction. **This is a byte-level claim tested by the
  instrument itself**, and if it fails, either my edits are not comment-only
  or the differ is wrong; both are reportable defects.
- **P6:** running `--since` against a base that predates the marker
  convention (I will use a sha from before `4d48a4e77`) will report every
  module as "base carries no markers" and label the outcome, **not** print a
  coverage delta of +18 points as if it were progress. Registered because
  the brief names this as a required labelled outcome and because a triumphant
  delta against a pre-convention base is the exact failure `#3045` names.

## 6. Decline floor

- **The tagging half is `FAILED as tagging`** if fewer than **200** of the
  650 in-fence untagged items can be given a marker I can honestly cite. A
  marker I cannot cite is a citation defect the tool exits 3 on, so this
  floor is enforced by the instrument and not by my judgement.
- **The whole lane is `FAILED`** if the comment-only constraint cannot be
  held (any non-comment byte in `crates/`), or if
  `scripts/gate_identity_diff.sh` reads anything but 0 lines over 21 rows.
- **`--since` is `FAILED`** if it cannot be watched failing on a planted
  fixture — a control never seen red is decoration (`#3336`), and this repo
  shipped a `--check` that could not fail.
- **I will not mint a DISCLOSURE row.** That namespace is peer `w-disclose`'s
  this wave. If a constant needs a row that does not exist, it gets the
  strongest mark I can justify without one (`[O]`, `[F]` or `[N]`) and the
  gap is **reported**, never papered over with an invented row id.
- **I will not change `MARK_RE` or the marker grammar.** A change there
  affects all 189 existing markers and the peer's citations; the brief makes
  it a last resort and I am registering that I do not expect to need one.

## 7. What I am NOT doing

- `crates/c2-core/**` — **not in my fence at all** this wave (`codegen/**` is
  peer `w-disclose`'s comment surface; `coff/**` is already at 100 %).
- `crates/c2-harness/src/subsys.rs` and `src/cli/subsys.rs` — peer
  `w-encmap`'s.
- Any `DISCLOSURE.md` edit, any `ref/README.md` edit.
- Any non-comment byte anywhere in `crates/`.
- **Pinning a live per-module tree count in a test.** Decision 16 names this
  as a fence: I move every one of those numbers myself and a peer moves them
  too. `--since` tests pin **planted-fixture counts and algebraic
  invariants** only.
