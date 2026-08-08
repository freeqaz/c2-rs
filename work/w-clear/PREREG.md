# w-clear — PREREGISTRATION

Committed **before the first probe**. Lane `w-clear`, worktree branch
`wt-w-clear` off master **`119af05f`**.

## 0. What this lane is asked

`w-4c` (#1383–#1390) took four frontier TUs from `EXIT:expr-chain-noform-0x4C`
to **READER-CLEAR**:

    src/xdk/LIBCMT/undname.cpp    7 -> 12 rungs
    src/xdk/LIBCMT/vswprnc.cpp    6 -> 11 rungs
    src/xdk/LIBCMT/vsnprnc.cpp    6 -> 12 rungs
    src/xdk/nuispeech/mmio.cpp    7 -> 12 rungs

Ladder all four on the **CODEGEN** side; convert one if it is genuinely short.

## 1. What I have already read (before predicting)

Board rows, oldest last, per the protocol:

* **#260** (`w-cross`) — `undname` `28030000`, `vswprnc`/`vsnprnc` `2c030000`:
  each an explicit compare of a **call result in r3** writing **cr0**, in bodies
  that later compare into **cr6**. *The discriminator is UNMEASURED.*
* **#275** (`w-conv`) — `?mmioGetInfo` is `w11_early_return.cpp::mm` plus
  `mr r11,r3 ; mr r3,r4` and a `memcpy` setup. OPEN. "closes one refusal and
  converts nothing on its own."
* **#483 / #506** (`w-tu2`, `w-tu3`) — `mmio.cpp` priced **17**, twice,
  independently; blocked on a **materialized common epilogue**, `r31` across
  calls, a **second CR-field regime**, an **intra-TU REL24**, `mtctr`/`bcctrl`.
* **#502 / #505** (`w-tu3`) — `mmio` is the frontier head by byte *fraction*
  (16.8 %) with **316 B remaining**; it ranks **15th of 18** by remainder.
* **#827** (`w-next`) — `mmio` re-derives dear a third time at 17.
* **#1346** (`w-cflowlabel`) — of 16 frontier TUs, **6 need `cflow-if-n` and
  nothing else**, and `undname`, `vsnprnc`, `vswprnc` are three of the six.
  Lifting it **still converts zero**; every one is priced ≥ 7.
* **#1353** (`w-one`) — the CODEGEN column for the one-function frontier TUs is
  read off #720's CFG instrument, **not** re-derived as an integer. 6 of 7 need
  a control-flow class the emitter cannot express.
* **#1385/#1386** (`w-4c`) — the four went READER-CLEAR under **11–12 poisoned
  sink grants**; READER-CLEAR is a *decoding* statement, never an acceptance.

## 2. THE INSTRUMENT PROBLEM, REGISTERED IN ADVANCE

Every rung of all four ladders is a **`sink`** rung (`work/w-4c/lad_tip2/ladder.json`
— 12/11/12/12 net, **0 hatch rungs each**). The chain sink is **poisoned**
(`expr-chain-sink-poison`, board #660): a body lifted through it can never reach
`select_function`. `ladder.py`'s own header says it: *"the CODEGEN column is
precisely what a poisoned lift cannot see."*

**So I predict, before probing, that `ladder.py` cannot climb the codegen side of
these four at all**, and that the honest instrument is the one `w-loop` shipped:
`codegen::frontier_bytes` — decode the reference obj's own `.text` words and ask
of each whether `codegen::encode` can build it. That is an **encoder** ladder,
which is a strict *lower* bound on the codegen ladder, and I register that
distinction now rather than after the fact.

## 3. PREDICTIONS

### 3.1 Obj-derived inventory (P-INV) — I predict my carried numbers are WRONG

Board **#1401**'s lesson. What the record hands me, which I will re-derive:

| TU | carried | mine to check |
|---|---|---|
| `undname` | 1 emitted fn; a `28030000` cr0 compare of a call result | fn count, sizes, `.data`/`.rdata`, relocs |
| `vswprnc` | 1 emitted fn; `2c030000` | same |
| `vsnprnc` | 1 emitted fn (#1353 says `fn_total = 1`) | **the source defines TWO functions** — `_vsprintf_s_l` and `vsprintf_s`. `w-4c` read `emit_blockers` = 2. I predict **2 emitted**, and that #1353's "seven TUs that are literally one function" list does not contain `vsnprnc` |
| `mmio` | 316 B remaining, 17 refusals, 11 emitted | the source defines **12** functions; `w-4c` read 11 emit_blockers. I predict **11 or 12 emitted** and a total `.text` well over 316 B |

**P-INV: I predict at least three carried items across the four are wrong or
unstated**, with `undname`'s relocation count and `mmio`'s function count the
most likely.

### 3.2 Codegen depth (P-DEPTH)

Independent codegen refusals, counted as *distinct structures the port has no
representation for*:

| TU | predicted codegen depth | converts? |
|---|---:|---|
| `undname` | **≥ 8** | no |
| `vswprnc` | **≥ 8** | no |
| `vsnprnc` | **≥ 9** (two functions, one a forwarding tail call) | no |
| `mmio` | **≥ 17** (holding #483/#506 rather than re-deriving lower) | no |

**P-CONV: none of the four converts.** The honest prior in the brief, and the
one I hold.

### 3.3 Encoder shortfall (P-ENC)

`w-loop` found `Primes.cpp` **14 of 16** words already encodable. These four are
call-heavy CRT/API glue rather than arithmetic, so the *instruction* vocabulary
should be even closer to complete. I predict:

* **≥ 90 % of all `.text` words across the four are already encodable** by
  `codegen::encode` as it stands at `119af05f`.
* The named shortfalls will be, in order of likelihood: **`bl` (REL24)**,
  **`mflr`/`mtlr`/`mtctr`/`bcctrl`**, **`bne`/`beq` off cr0**, **`stmw`/`lmw` or
  `__savegprlr` helper calls**, **`cmplw` register-register**, **`b` to a
  materialized epilogue**.
* At least one of the four will need **zero** new encoders.

### 3.4 The CR-field discriminator (P-CR)

#260 says the discriminator between `cr0` and `cr6` is unmeasured and that an
emitter hard-coding `BI = 4*6+bit` writes *a plausible-looking wrong branch*.
I predict **all four TUs contain both regimes**, so the discriminator is a rung
on all four, and that it is **not** an encoder rung — `encode_bc` takes `bi`.

### 3.5 Bytes remaining (P-BYTES)

| TU | predicted `.text` bytes |
|---|---:|
| `undname` | 80–140 |
| `vswprnc` | 120–200 |
| `vsnprnc` | 160–260 |
| `mmio` | **316** (#505's published figure — re-derived, not carried) |

## 4. DIRECTION OF ERROR — and which SHAPE this rung is

Board **#770** is **eleven for eleven on optimistic misses**; `w-4c` broke the
streak *pessimistically*, and named the cause: `0x4C` was **a closing bracket at
the end of every call**, i.e. one floor under every call site at once, not a
floor in the middle of one body.

**Is my rung bracket-shaped?** There is exactly one candidate and I name it:
**`cflow-if-n`**. #1346 measured that *six* frontier TUs need it and nothing
else, and three of my four are on that list — so if `cflow-if-n` were the only
thing missing, all three would move together and I would miss pessimistically
like `w-4c`.

**I predict it is NOT bracket-shaped, and that I will miss OPTIMISTICALLY.**
The reason: #1346 already ran the bracket experiment on this exact axis and
recorded the answer — *"it still converts zero"*, every one of the six priced at
≥ 7 independent refusals. A CFG class is a floor under the *branches*; it is not
a floor under register allocation, the label counter, the frame class, the
relocation fan-out or the CR-field regime, and those are per-body. So I register
**optimistic**, i.e. the measured depths will come back **dearer** than §3.2.

### 4.1 Registered failure modes — conditions under which I am wrong

* **F1** — if the measured depth of any of `undname`/`vswprnc`/`vsnprnc` is
  **< 6**, §3.2 was too dear and #1346's "≥ 7" is refuted on that TU.
* **F2** — if the encoder shortfall is **> 10 % of words**, §3.3 is wrong and the
  instruction vocabulary, not the structure, is the binding constraint.
* **F3** — if `mmio` re-prices **below 17**, three independent derivations
  (#483, #506, #827) are jointly wrong and this lane must say so loudly.
* **F4** — if `ladder.py` *can* climb a codegen rung on any of the four, §2 is
  wrong about the poison and the whole instrument choice was unnecessary.

## 5. Declines registered in advance

* **I will not open `crates/c2-core/src/codegen/coff.rs`.** If a rung requires
  it I stop and report that.
* **I will not fix `c2rs gap --cache <RELATIVE dir>`** (#1388) — a concurrent
  lane owns it. Every cache argument this lane passes is **absolute**.
* **I will not touch** `crates/c2-il/src/func/body/expr.rs` (lane `w-5c`),
  `crates/c2-harness/src/` cache handling or `scripts/gate.sh` (lane `w-cache`),
  or `work/w-front3/hatch.py` (lane `w-hatch` owns its repair).
* **A measured table with no conversion is the expected outcome of this lane**,
  and I register that I will not manufacture a conversion to avoid publishing
  one.
