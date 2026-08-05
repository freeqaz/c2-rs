# w-next — PREREG

Lane `w-next`, worktree `agent-a1a5f35ed31ef558f` off master **`8fd79b6`**.
Board range **#826–#835**.

**Honesty note on ordering.** w-hash committed its prereg before any probe
script existed in its lane directory. This lane did not: the TU **selection**
required re-deriving prices off real objs, and the brief mandates that
re-derivation ("re-derive the price yourself from the obj and the IL before
building"). So §0 below records measurements **already in hand** at the moment
this file was written, named precisely, and §1 registers predictions about
things **not yet measured**. Every §1 row is scored later against a run that did
not exist when this was committed. The scripts that existed at commit time are
`getobjs.sh`, `one.sh`, `chainwalk.sh` — all read-only with respect to
`crates/`, none of them a probe of anything in §1.

---

## 0. What was already measured, before any prediction below

### 0.1 The baseline scan (`work/w-next/baseline_scan.txt`, tree `8fd79b6`)

`match 10 · mismatch 0 · codegen-gap 0 · vocab-gap 861 · capture-fail 7`;
A 28 (LO 27) / B 338 / C 169 / D 10 / E 2; `B∧C` 151 · `A∧B∧C` 27 ·
**FRONTIER 17**; FBM 0.16654, `fnbyte-exact` 29,802, **`fnbyte-differs` 0**,
`fnbyte-partial` 9,375; census/gate disagreement 0; every control 0.

### 0.2 The selector, read off that scan

FRONTIER BY `.text` BYTE FRACTION (board #500), head of the ranking:

| rank | frac | accepted/total | remain | src |
|---:|---:|---|---:|---|
| 1 | 16.8% | 64/380 | 316 | `src/xdk/nuispeech/mmio.cpp` |
| 2 | 7.8% | 112/1440 | 1328 | `src/keygen_xbox.cpp` |
| 3 | 5.9% | 16/272 | 256 | `src/system/utl/EncryptXTEA.cpp` |
| 4–17 | **0.0%** | 0/N | — | the other **fourteen** |

### 0.3 The prices re-derived from the objs, by hand, before choosing

Reference objs captured at the workload's own flags via `work/w-frame/refobj.sh`
and disassembled with `scripts/gt_dump.py`. What each one actually contains:

* **`mmio.cpp`** (selector's head, and the instrument's own text records it
  **DECLINED**): 11 emitted, 8 of them the 8-byte `li 3,0; blr` leaf. The 3
  blocked ones are `mmioGetInfo` (84 B), `mmioSetInfo` (108 B) and `mmioClose`
  (124 B) — multi-arm `cmplwi`/`bf`/`b` if-chains with merge points, `r31`
  live across calls, a **`bctrl` indirect call through a vtable-ish slot**, and
  a conditional store. Re-derives **dear**, consistent with the two prior
  derivations at **17**. Not chosen: it is the selector's head *and* already
  declined at a price nobody has disputed.
* **`keygen_xbox.cpp`**: 1440 B, 18 blocked of 20, needs **two** CFG classes the
  port lacks (`cflow-if-n`, `cflow-loop`), labels 15. The largest TU on the
  frontier by every unit. The selector ranks it **2nd** — which is the clearest
  single demonstration that a *fraction* ignores absolute size.
* **`EncryptXTEA.cpp`**: 64-bit `std`/`ld`/`rldicl`/`rldimi`/`stdx`/`stdu`, an
  `mtctr`/`bdnz` counted loop, `__savegprlr_26`/`__restgprlr_26` register-save
  **helper calls**, and a 2×-unrolled Feistel pipeline with the delta constant
  split `addis`+`addi`. Dear.
* **`Main.cpp`** (0%, and one of only **two** TUs the CFG screen calls
  REACHABLE): its `.text` COMDAT **opens with two ADDR32 words** —
  `__CxxFrameHandler` and `__ehfuncinfo$main` — and carries a **second entry
  point at 0x54**, the unwind funclet (`addi 31,12,-112`, then `bl ??1App`).
  EH machinery, 3 calls, a stack object at `r31+80`. Much dearer than 124 bytes
  suggests.
* **`xboxheap.cpp`** (0%, the other REACHABLE one): **ONE** emitted function of
  **80 bytes**, `??0CXboxHeap@NUISPEECH@@QAA@II@Z`, `cflow-straight`,
  `calls-1`, **one** census blocker key. The cheapest on the frontier by
  re-derivation. **CHOSEN.**

### 0.4 `xboxheap`'s 80 bytes, and its source

```cpp
NUISPEECH::CXboxHeap::CXboxHeap(unsigned int initSize, unsigned int size) {
    mSize = size;  mFreeHead = this;  mCount = 0;  mUsedHead = this;
    auto& listHead = mListHead;
    listHead.mNext = &listHead;  listHead.mPrev = &listHead;
    AllocatePageBlock(initSize);
}
```

```text
  idx                                        what
  --   0000  mflr 12          \
  --   0004  stw 12,-8(1)      |  framed prologue, ONE callee-saved GPR,
  --   0008  std 31,-16(1)     |  frame 96 = align16(80 + 8 + 8)
  --   000c  stwu 1,-96(1)    /
   0   0010  li 10,0            P1 — producer for S3
   1   0014  stw 5,16(3)        S1  mSize    = size      (formal r5)
   2   0018  addi 11,3,8        P2 — producer for S5,S6: the INTERIOR pointer
   3   001c  stw 3,0(3)         S2  mFreeHead = this     (this INTO its own field)
   4   0020  stw 10,20(3)       S3  mCount   = 0
   5   0024  mr 31,3            P3 — `this` into a callee-saved reg for the return
   6   0028  stw 3,4(3)         S4  mUsedHead = this
   7   002c  stw 11,8(3)        S5  listHead.mNext = &listHead
   8   0030  stw 11,12(3)       S6  listHead.mPrev = &listHead
   9   0034  bl AllocatePageBlock   REL24; NO argument setup at all (r3,r4 already right)
  10   0038  mr 3,31            the constructor returns `this`
  --   003c  addi 1,1,96      \
  --   0040  lwz 12,-8(1)      |  epilogue
  --   0044  mtlr 12           |
  --   0048  ld 31,-16(1)      |
  --   004c  blr              /
```

**The six stores are in exact SOURCE order** (16, 0, 20, 4, 8, 12). That much is
free. What is **not** free is where the three producers land: P1 at idx 0, P2 at
idx 2, P3 at idx 5.

### 0.5 The IL chain, walked with the instrument built for it

`C2RS_SINK_CHAIN` (boards #660/#661) decodes one opcode at a time and
**poisons** — it pushes no `IlOp` and refuses at the end, so it cannot move an
obj byte. `work/w-next/chainwalk.sh`:

```text
step 0  spec=[<none>]                     -> expr-op-0x27
step 1  spec=[op:27]                      -> expr-op-0x32
step 2  spec=[op:27,op:32]                -> expr-op-0x4B
step 3  spec=[op:27,op:32,op:4B]          -> expr-op-0x4F
step 4  spec=[op:27,op:32,op:4B,op:4F]    -> expr-call-in-expr-data-addr-then-off-add-and-chain-bind-more
```

**Four opcode steps and then a structural refusal**, against a census that
reports **one** blocker key. This reproduces board **#622**'s `0x27 → 0x32` and
extends it by three more steps. It is the direct measurement of the standing
warning that *closing a blocker key is set substitution, not removal*.

---

## 1. REGISTERED — predictions, none of them yet measured

### R1 — the baseline reproduces
Every digit in §0.1 reproduces on a re-run at this tree. **Expect HIT.**

### R2 — the selector is structurally blind on single-function TUs
`byte_fraction` is `accepted_bytes / total_bytes` over a TU's emitted
functions. For a TU with **exactly one** emitted function the port either
accepts it (and the TU is at 100 %, i.e. very likely a match) or refuses it (and
the TU is at exactly 0 %). So the fraction carries **no information** on those,
and the ranking over them is alphabetical, not ordinal.

**Registered numerically:** of the 17 frontier TUs, **8** have
`emit-emitted == 1`, and **all 8** read exactly 0.0 %. Both TUs the CFG screen
calls REACHABLE are inside that blind block. **This can lose** — if fewer than 8
single-function TUs exist, or if any of them reads non-zero, the claim is wrong
as stated.

### R3 — `xboxheap` re-prices at **14**, against a census that shows 1
Predicted mechanism count, enumerated now so it can be scored:

| # | mechanism | side |
|---|---|---|
| 1 | `expr-op-0x27` | IL |
| 2 | `expr-op-0x32` | IL |
| 3 | `expr-op-0x4B` | IL |
| 4 | `expr-op-0x4F` | IL |
| 5 | `expr-call-in-expr-data-addr-then-off-add-and-chain-bind-more` (composite; may itself split) | IL |
| 6 | binding the C++ **reference local** `auto& listHead` through `.sy`/`.gl` | IL |
| 7 | the six-field store sequence to `this`, in source order | emit |
| 8 | `stw 3,0(3)` — storing `this` into its **own** field | emit |
| 9 | `addi 11,3,8` — an interior pointer as a **shared** producer for two stores | emit |
| 10 | `li 10,0` — a literal producer | emit |
| 11 | **the producer hoist schedule** — P1 at 0, P2 at 2, P3 at 5 | emit |
| 12 | `mr 31,3` — `this` into a callee-saved GPR across the call, and *which* one | emit |
| 13 | the call with **zero** argument setup (r3, r4 pass through untouched) | emit |
| 14 | `mr 3,31` — the constructor returns `this` | emit |

**Not counted, and predicted already free:** the framed prologue/epilogue and
the 96-byte frame with one saved GPR. `codegen::frame` already models
callee-saved GPRs at `r31 → −16(r1)` and `align16(80 + 8 + saved)`.
**That prediction can lose too** — if the frame does not come out at 96 with
`std`/`ld` in those slots, row 15 exists and the price is 15.

**Expect: the true count is ≥ 10.** A count of ≤ 5 refutes R3.

### R4 — the producer hoist distance is NOT a constant, and I will not fit it
Measured inside this one body: P1 → S3 is 4 slots, P2 → S5 is 5 slots, P3 → its
consumer is 5. **A fixed-distance rule is already dead in the only cell anyone
has looked at.** I predict a grid over constructor shapes will show the
placement varying with something this port cannot read, and that **no validated
schedule rule ships**. If a rule *does* validate on held-out cells, R4 loses and
that is the better outcome.

**Registered as the discipline, not just the prediction:** if I catch myself
ranking or fitting a placement rule, the answer is a **transcription** — draw the
class narrowly enough that the schedule is a constant, then grade the axes that
remain free. That is what `w-tu1` and `w-hash` both did, and it is 2-for-2.

### R5 — the conversion does NOT land, and the IL chain is what stops it
Registered **against my own goal**, as w-hash registered its R8. I predict TU
match reads **10** at the end of this lane, and that the stopper is §0.5's chain
— specifically the terminal
`expr-call-in-expr-data-addr-then-off-add-and-chain-bind-more`, which is a
*composite* key and therefore a chain of unknown further length behind a name
that looks like one fact. **I want this to lose.**

### R6 — the warranty does not move
`fnbyte-differs` is **0** at both ends. `gate.sh` grows by exactly 18
verdicts per fixture added and by nothing else; the sweep's and the cross's
ungraded counts hold byte-identically. `mismatch` stays 0 everywhere.

### R7 — additive refusal
Whatever ships, every new accept is a positive guard: `Some(false)` is the only
reading acted on, and every must-fail mutation is **run**, not described. I
register the bar w-varloop set: mutations that produce **wrong bytes** rather
than refusals are the ones that count, because a transcription breaks by
declining while a lowering breaks by getting the arithmetic wrong.

---

## 2. What would make this lane worth landing even if R5 wins

The brief says naming the exact stopper is the more common outcome and is what
makes the next lane's estimate honest. Three things here are worth more than
the conversion and none of them depends on it:

1. **R2** — a measured domain limit of the project's only 2-for-2 selector. Not
   a rival ranking; a statement about where the sanctioned one is silent.
2. **§0.5** — the first chain-walk of a frontier TU published as a *depth*,
   extending board #622 from one step to four-plus.
3. **§0.3** — re-derived prices for five frontier TUs off their own bytes, which
   is the pricing the FRONTIER's own caveat says is UNVERIFIED on its newer
   members.
