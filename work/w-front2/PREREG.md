# w-front2 — pre-registration

Lane **w-front2**, 2026-08-08, branched at master `e60f8902` (branch
`wt-w-front2`).

**Committed before the first probe obj is built and before any file under
`crates/` is read for the purpose of changing it.** The only measurement taken
before this file was written is a *baseline reproduction* — one `c2rs gap` run
over the 878-TU workload, which is the lane's obligation to reproduce the
coordinator's numbers and is not a probe of any candidate TU.

---

## 0. Baseline, reproduced in this worktree

`./target/release/c2rs gap --list work/dc3-workload/files.txt --flags-file
work/dc3-workload/flags.txt --cwd <dc3> --jobs 16`
(`work/w-front2/scan_base.txt`):

| key | value | coordinator's figure | |
|---|---:|---:|---|
| match · mismatch · codegen-gap | 10 · 0 · 0 | 10 · 0 · — | ✔ |
| vocab-gap · capture-fail | 861 · 7 | 861 · 7 | ✔ |
| `factor-a` / `-a-lo` | 28 / 27 | 28 (LO 27) | ✔ |
| `factor-b` · `-c` · `-d` · `-e` | 338 · 169 · 10 · 2 | 338 · 169 · 10 · 2 | ✔ |
| `b-and-c` · `a-and-b-and-c` | 151 · 27 | 151 · 27 | ✔ |
| **`frontier`** | **17** | **17** | ✔ |
| `frontier-if-a` | 139 | 139 | ✔ |

Every digit the brief hands over reproduces.

## 0.1 The frontier, by name, off *this* scan (never a stale list)

Seventeen, with the three rankings the scan already prints beside each
(`blocked`, `.text` bytes, CFG reachability, label channel):

| TU | blocked/emitted | `.text` bytes | CFG verdict | in w-conv's 17? |
|---|---:|---:|---|---|
| `src/Main.cpp` | 1/1 | 124 | REACHABLE, labels 9 | yes |
| **`src/system/math/Primes.cpp`** | 1/1 | **64** | `cflow-loop`, **label-free** | **NO — new** |
| `src/xdk/LIBCMT/osfinfo.cpp` | 1/1 | 152 | `cflow-if-n` | yes |
| `src/xdk/LIBCMT/undname.cpp` | 1/1 | 140 | `cflow-if-n` | yes |
| `src/xdk/LIBCMT/vswprnc.cpp` | 1/1 | 156 | `cflow-if-n` | yes |
| `src/xdk/nuispeech/xboxheap.cpp` | 1/1 | 80 | REACHABLE, label-free | yes |
| `src/xdk/xjson/jsonwriter.cpp` | 1/1 | 304 | `cflow-loop` | yes |
| `src/xdk/xlrc/xlrcimpl.cpp` | 1/1 | 152 | `cflow-if-n` | yes |
| `src/system/negate_test.cpp` | 2/2 | 160 | `cflow-if-n` | yes |
| `src/system/synth_xbox/Biquad.cpp` | 2/2 | 176 | REACHABLE, labels 3 | yes |
| `src/xdk/LIBCMT/vsnprnc.cpp` | 2/2 | 164 | `cflow-if-n` | yes |
| `src/system/rndobj/wordwrap.cpp` | 3/3 | 816 | `cflow-if-n`+`cflow-loop` | yes |
| `src/system/utl/Pool.cpp` | 3/3 | 132 | `cflow-loop` | yes |
| `src/xdk/nuispeech/mmio.cpp` | 3/11 | 380 | `cflow-if-2`+`cflow-if-n` | yes |
| `src/system/synth_xbox/IPP_basicmath_xbox.cpp` | 4/4 | 184 | `cflow-loop` | yes |
| `src/system/utl/EncryptXTEA.cpp` | 4/5 | 272 | `cflow-loop` | yes |
| **`src/keygen_xbox.cpp`** | 18/20 | **1440** | `cflow-if-n`+`cflow-loop`, labels 15 | **NO — new** |

**Fifteen of the seventeen are w-conv's own rows.** `xboxmem.cpp` and `Sort.cpp`
converted out; `Primes.cpp` and `keygen_xbox.cpp` came in. So the "UNVERIFIED on
the newest members" caveat in `STATUS.md` reduces to exactly **two** rows that
have never been priced by anybody, and fifteen that were priced at a different
master.

---

## 1. The incumbent this lane must beat — named, not a bare threshold

The decline clause is *"a frontier TU at ≥ 4 independent refusals is not a
target"* (board #269). A bare `4` is not the bar, because the clause is a guide
and the brief says so. The **incumbent** is:

> **`src/Main.cpp` at SIX independent refusals** — w-conv's joint-cheapest row
> and, since `xboxmem.cpp` (also 6) converted, the cheapest *surviving* frontier
> TU on the published price. Its six: the two-word
> `__CxxFrameHandler`/`__ehfuncinfo$main` prefix inside `.text` with ADDR32
> relocations; two `.pdata` records; a 64-byte EH `.rdata` group with five
> relocations; a funclet with its own prologue; the `addi r31,r1,-112`
> frame-pointer form; (and the body itself).

**A conversion in this lane must price strictly below 6**, i.e. at **≤ 5**, or
the lane has found nothing the incumbent did not already offer. At **≤ 3** the
brief instructs me to take it and I will. At **4–5** I will state the case both
ways and decide on the record; the argument that has to be made there is not
"the number is small" but *"these N refusals are each a bounded transcription
with an oracle witness, in the sense `w-hash`'s twenty words were"*.

**And a second incumbent, because the first one is about price and not about
outcome:** the standing *conversion* incumbent is `w-hash`/`Sort.cpp` — a TU
that **re-priced at 11 against a handed-over 8 and converted anyway**, because
the class was drawn narrowly enough that the allocation became a constant. So
"expensive" has already been shown not to imply "undoable"; what it implies is
that the conversion must be a transcription of one function class, not a
lowering.

---

## 2. Predictions

Each carries the rival reading that would refute it.

| # | prediction | rival that would refute it |
|---|---|---|
| **P1** | **The fifteen carry-over rows re-price at their w-conv figure ± 1, and NOT lower.** Nothing this session shipped touches the PowerPC repertoire: mechanism E and its fixpoint are *elisions* (a body that emits nothing), `w-seed`/`w-memset`/`w-inl0` widen the **no-effect / destroy-loop** reader, `SPLICE-0-PORT`, `w-tag02` and `w-inread` are `.in` **data-initializer** readers. None of them emits an instruction the port could not emit at `caff20d`. | the session's readers moved a *body* refusal — e.g. an `.in`/`.gl` widening turned out to be what was blocking a frontier body's data operand, so a row drops by 2+ |
| **P2** | **`Primes.cpp` is the cheapest of the seventeen and prices at 5.** 64 bytes, one function, label-free, no frame, one loop. Named: (1) a counted/searched loop with a back edge that is **not** `ptr_walk_loop`'s shape; (2) an **early return from inside the loop** — two exits, so two branch targets; (3) an `int`-array element load with a **stride-4** induction (`lwz`/`lwzu`), where `ptr_walk_loop` admits `lbzu` stride 1; (4) a **REFHI/REFLO** address pair for the function-local `static` array; (5) a signed `>=` compare feeding a branch. | it prices at ≥ 6 — most likely because the loop is **rotated + peeled** the way `Sort.cpp`'s was (w-hash #9/#10), which would add two refusals nothing in the IL names |
| **P3** | **`keygen_xbox.cpp` is the most expensive row of the seventeen** and prices ≥ 12 — 1,440 `.text` bytes, 18 blocked functions, 15 label slots, both missing CFG classes. | it is a repetition of one cheap class 18 times over, so the *independent* count is small even though the byte count is huge — the exact collapse rule this lane is obliged to apply |
| **P4** | **The minimum over the seventeen is ≥ 5**, i.e. the frontier is still expensive and w-conv's "no cheap TU left in it" survives with its number moved by at most 1. | some row comes in at ≤ 3 and the lane converts it |
| **P5** | **`xboxheap.cpp` stays unpriceable-by-count.** w-conv recorded it as diverging at instruction 0 on *schedule*, with every instruction in vocabulary and `gap 0`; six scheduling rules died on it at `w-pair`. A count is the wrong instrument for it and I expect to say so again rather than to produce a number. | a schedule rule falls out of the two extra frontier members' objs and prices it |
| **P6** | **No TU converts in this lane.** The re-priced minimum is ≥ 5, above the take-it floor, and the honest table is the deliverable. | a row comes in at ≤ 3, or a 4–5 row turns out to be a `w-hash`-style transcription with every free axis graded |

### 2.1 The direction I expect to lose on, registered in advance

**I expect to lose P1/P2's optimism and I expect the loss to arrive as
`Primes.cpp` coming back DEARER than 5.** The recorded prior is explicit and
runs 5 for 5 against me: board **#770** — *"the fifth consecutive cross-check of
a frontier TU to come back dearer than the list it was handed"* (`negate_test`
10 v 4, `xboxmem` 15 v 4, `mmio` 17 v 5 **twice**, `Sort` 11 v 8). Every one of
those five was a lane that expected to find a bargain and found a bill. **P2 is
therefore written against the prior, deliberately**, so that if it loses the
sixth consecutive instance is on the page with its own prediction next to it,
and if it wins the prior is broken by a case that was called in advance.

The **second** direction I expect to lose on is **P6 vs the brief's hope**: the
brief wants a conversion and I am registering that there will not be one. If P6
loses, it loses in the direction of the project's goal, which is the good
outcome — and it is registered as a loss so that it cannot be reported as a
plan.

### 2.2 What I explicitly refuse to count as a refusal

The collapse rule, stated before it can be convenient (`STATUS.md` records an
over-prediction of 5 against a realized 2 caused by counting one variable eight
times):

* **one quantity at several thresholds is ONE refusal** — e.g. three compares
  against three different immediates is one "compare against an immediate";
* **N repetitions of one production is ONE refusal** — 18 blocked functions of
  the same class is one, not eighteen (this is what P3 is exposed to);
* **a register field is not a refusal**; a *block plan* is (w-hash §3.2);
* **an instruction the port already encodes is not a refusal** even if no
  current recognizer reaches it — I will check the encoder set, not the
  recognizer set, and say which of the two I used on every row.

---

## 3. Alarm semantics, restated as this lane's obligations

`mismatch` stays **0** and outranks everything. `fnbyte-exact` must not shrink,
`differs` must not grow, `match-tu-differs` and `match-tu-reloc-differs` stay 0.
TU match may rise; if it does I name the TU and show the byte-exact compare
against real `c2.dll` under wibo at the workload's own flags, never against a
cached or `/Ox` obj.

`scripts/gate.sh --require-graded` at both ends, quoted as **graded counts** and
not as exit codes. `work/w-splice/peerkeys.py` at both ends, reported, because
the concurrent lane `w-rdata3` owns `crates/c2-core/src/coff/` and this lane
owns `crates/c2-core/src/codegen/`.
