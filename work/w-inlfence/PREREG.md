# w-inlfence — PREREG

    Lane:   w-inlfence, worktree branch `worktree-agent-aa27ac4f82249ec4b`
    Base:   master `751351b6` (the w-fltret merge)
    Frozen: BEFORE the first `crates/` change and before the first fixture line.
    Board:  #2220–#2239 allocated; the remainder will be declared unminted.

---

## 0. The commission, in one sentence

`w-fltret` admitted 444 emitted functions whose bodies c2 **inlines**
(`rungs/2026-08-09-w-fltret.md` §6, board #2082): `fnbyte-exact` moved by
**zero** and `fnbyte-differs` by **+444**. Nothing is wrong on disk today —
`mismatch` is 0 and all 434 TUs are `vocab-gap` — but that is those TUs being
blocked *elsewhere*. This lane turns "blocked elsewhere" into a **named class
fence**, measures what the fence takes back, and states plainly whether the port
can emit a wrong body in this class.

## 1. What is already on disk, re-derived at base (no `crates/` change)

`work/w-inlfence/scan_base.out`, 878 TUs:

| | base `751351b6` |
|---|--:|
| TU match | 18 |
| mismatch | **0** |
| per-function census | 712,238 (28.91 %) |
| emitted census | 39,644 (22.15 %) |
| `fnbyte-exact` | 36,228 |
| `fnbyte-differs` | 2,555 |
| `fnbyte-reloc-differs` | 861 |
| FBM | 0.20243 |
| PROGRESS MASS | 0.20893 |

**In-class rows by key, aggregated from `fn_dispatch` (`…|INCLASS|…`)** —
712,238 total, of which the **call-carrying** keys sum to **212,117** (29.8 %):
tail calls 127,566 (`fp-tail-call` 65,395, `multiarg-tail-call` 30,395,
`fp-multiarg-tail-call` 18,876, `int-tail-call` 12,060, `void-tail-call` 840),
the dtor/ctor delegations 41,826, the `call-sequence-*` family 39,156,
`framed-call` 3,558, and 19 rows of singleton classes.

**Emitted rows by FBM shape** — 29,357 `plain` (no callee, **all byte-exact**),
7,098 `tail`, 2,605 `seq`, 450 `float`, 123 `framed`, 9 singletons.

**The over-broadness control this gives me for free.** 5,172 `tail` and 1,238
`seq` emitted rows are **byte-exact against real c2**, i.e. c2 emitted the `bl`
and did *not* inline the callee. A fence that is exactly right takes back **only
wrong bodies**, so `fnbyte-exact` must not fall by one.

## 2. The fence I intend to ship

**One predicate, asked in the parser, at both callers.** A callee whose name
this TU also **defines** is a callee c2 may inline; the port may not, so the
function is refused. Where the callee is genuinely external/opaque, the port
keeps its call — the accept case `fixtures/cpp/wfltret_value_tail.cpp` already
grades byte-exact at `/O1` **and** `/Ox`.

* **The gate (`IlBundle::functions`) already carries this clause** as a
  whole-TU refusal (`bundle.rs`, *"A callee that is also DEFINED here is out of
  class: c2 may inline it"*), over a `Bindings::per_record` name list that is
  total by construction. The intent is to **factor it into one shared
  predicate** so a future narrowing of that wholesale refusal — which
  `WB_INLINE_FINDINGS.md` §7 explicitly proposes — cannot uncover the class.
* **The census does not carry it at all**, which is why the emitted census
  claims 444 bodies c2 inlines. The census gains the same predicate as a
  **post-parse gate**, keyed per function.
* **Fail-open direction, named up front**: the census's defined-name set comes
  from `gl_defined_names`, which returns an EMPTY pair when its walk stops. A
  name **in** the set is certainly defined here (refuse — sound); a name absent
  may still be defined here on a TU whose records did not frame (silent — the
  precedent is `Bindings::is_varargs`, which is silent when unpaired because the
  gate refuses that TU for want of names anyway). **That residue will be sized,
  not folded in.**

**Adoption**: none intended. `WB_INLINE_FINDINGS.md`'s two pre-drafted rows are
`route:` rows needing no constant, and this fence copies **no threshold** — it
is the categorical direction (*"c2 cannot inline what it cannot see"*), which
needs no ceiling, no favour-speed bit and no instruction count. If any
threshold constant enters `crates/`, a `DISCLOSURE.md` row lands in the same
commit.

## 3. Predictions

Scored in the rung. `p` is my credence before any measurement.

| # | prediction | p |
|---|---|--:|
| **P1** | the fence takes back ≥ 1 emitted function | 0.85 |
| **P2** | the **emitted** census delta is in `[−4000, −100]` | 0.55 |
| **P3** | the emitted census delta is more negative than **−1000** | 0.45 |
| **P4** | the **per-function** census delta is in `[−250000, −10000]` | 0.60 |
| **P5** | **`fnbyte-exact` does not fall** — 36,228 → 36,228 | 0.72 |
| **P6** | the take-back is concentrated in `fnbyte-differs` + `fnbyte-reloc-differs`: ≥ 70 % of the emitted rows taken back are graded NOT byte-exact at base | 0.70 |
| **P7** | ≥ 400 of w-fltret's **444** are taken back, checked BY NAME | 0.45 |
| **P8** | ≥ 1 of the 444 is taken back (i.e. the census can see this class at all) | 0.70 |
| **P9** | `?SplitMs@Timer@@QAAMXZ` is among the taken-back names | 0.55 |
| **P10** | `mismatch` stays 0 at every level — 878 TUs, 314 fixtures × 2 modes, 18 gate lanes, sweep, cross | 0.95 |
| **P11** | **TU verdicts move in ONE direction only** (toward refusal) and in fact move **zero**: match 18 → 18, 0 TUs change verdict by name | 0.85 |
| **P12** | `IlBundle::functions`' behaviour is **unchanged on every one of the 878 TUs** — the factoring is a refactor there, not a widening | 0.88 |
| **P13** | the fail-open residue (call-carrying census rows on TUs whose defined-name walk stopped) is **larger** than the population the fence takes back | 0.50 |
| **P14** | at least one wb-inline decline clause is declined for shipping because it converts **0 by construction** under the existing architecture, and I can name which | 0.80 |
| **P15** | `#[test]` DELTA is in `[+3, +12]` | 0.65 |
| **P16** | no census key vanishes; the new key(s) are the only ones that appear | 0.70 |
| **P17** | ≥ 1 of this lane's own `_neg` cells is confounded on first writing (three lanes running) | 0.55 |
| **P18** | ≥ 1 unnamed refusal fires at a pre-armed place (below) | 0.65 |

**Registered direction: PESSIMISTIC on the size of the take-back and
OPTIMISTIC on nothing.** The one number I most expect to be wrong is P7 — the
census's defined-name set may not be readable on the 434 `Timer` TUs at all,
in which case the fence is real, correct, and takes back **0 of the 444**, and
that is the honest report.

## 4. The unnamed-refusal budget: ONE. Pre-armed places

1. **FENCE ORDER / CLAUSE REACHABILITY** (w-park's streak 9/15). A per-function
   gate applied after `shape_to_function` can be unreachable because an earlier
   gate already refused, and "0 in the key map" would then read as *"the clause
   is inert"* when it means *"the clause never ran"*. Instrument: the new key
   must appear with a non-zero count **and** the rows it claims must be
   subtracted from a NAMED prior key, checked as a map diff.
2. **BOARD #1380 — `git checkout -- crates/` eats uncommitted work.** Every
   scratch instrument this lane applies will be preceded by a commit of all real
   work. Stated here so a violation is a scored refusal and not a footnote.
3. **A THIRD, from w-fltret §9.2's own miss**: any file list generated for a
   neutrality scan is compared with `wc -l` against `ls fixtures/cpp/*.cpp | wc -l`
   **after** the last fixture is written, never before.

## 5. Declines registered in advance

| # | declined | why, in advance |
|---|---|---|
| **D1** | **the accept side of the inline predicate** — any rule of the form *"c2 will not inline this callee, so the port may keep the call"* | `WB_INLINE_FINDINGS.md` §7: *"The accept side is not offered"*; a mis-predicted accept is a wrong obj |
| **D2** | every **size ceiling** — `(300,308]`, `(212,252]`, `(100,116]`, `(156,164]`, the loop class's `(56,80]` | they are brackets, not numbers, and the port cannot ask "how big is this callee" without lowering it first |
| **D3** | the **budget** `clamp(2×instrs, 1000, 35000)` and the 40-instruction free threshold | `WB_INLINE_FINDINGS.md` §4.1 records it READ, NOT CONFIRMED, with no DISCLOSURE row |
| **D4** | the **POGO** cost model and both 46-dword tables | unreachable on this workload; no value quotable |
| **D5** | `/Ob0` as an accept-side licence (*"at `/Od` nothing inlines, so admit the class"*) | an accept side by another name |
| **D6** | any widening of `IlBundle::functions`, `PORT_CFG_CLASSES`, `splice.rs`, or any whole-TU recognizer | this is a fence lane |
| **D7** | rewriting w-fltret's rung or board rows | dated records stay as written |

## 6. Reproduction

```sh
sh work/w-inlfence/scan.sh scan_base      # at 751351b6
sh work/w-inlfence/scan.sh scan_tip
python3 work/w-inlfence/keys.py work/w-inlfence/scan_base.jsonl \
        --diff work/w-inlfence/scan_tip.jsonl
```
