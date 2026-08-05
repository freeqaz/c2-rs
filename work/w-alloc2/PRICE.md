# `xboxheap.cpp` re-priced — 16, and three of my five "new" rows were already paid

Lane `w-alloc2`. Prereg **R5** registered **19** and said *"a true count ≥ 15
refutes nothing below 19"*. Measured: **16**. R5 scores **PARTIAL** — right that
the TU does not convert, inside the registered band, and **wrong on the count in
the OVER-counting direction**, which is the opposite of w-next's error and is
the part worth reading.

## 0. What is being priced

`src/xdk/nuispeech/xboxheap.cpp`, one emitted function of 80 bytes,
`??0CXboxHeap@NUISPEECH@@QAA@II@Z`. Obj captured at the workload's own flags
through `work/w-frame/refobj.sh`, disassembled with `scripts/gt_dump.py`. It
reproduces w-next §3.3 instruction for instruction, and its `.pdata`, symbol
table and three label symbols are read here rather than taken on trust.

The IL chain was **re-walked, not copied** (`work/w-alloc2/chainwalk.out`), and
reproduces w-next's five steps exactly:

```text
step 0  spec=[<none>]                     -> expr-op-0x27
step 1  spec=[op:27]                      -> expr-op-0x32
step 2  spec=[op:27,op:32]                -> expr-op-0x4B
step 3  spec=[op:27,op:32,op:4B]          -> expr-op-0x4F
step 4  spec=[op:27,op:32,op:4B,op:4F]    -> expr-call-in-expr-data-addr-then-off-add-and-chain-bind-more
```

## 1. The method w-next's price was missing, and the one mine was missing

w-next's own diagnosis:

> *"my fourteen contained no register-allocation item at all. A price derived
> from an obj is systematically blind to facts the obj states as answers."*

That is right and it is **half** the rule. The other half is what this lane hit:
a fact the obj states as an answer may be one the port has **already paid for**.
`li 10,0` / `addi 11,3,8` is an unpaid answer; the `.pdata` word `40 00 14 04`,
the `$M2595`/`$M2596`/`$T2597` labels and the 96-byte frame are answers the obj
states just as loudly and that `coff::pdata`, `coff::label` and `codegen::frame`
already emit for every framed non-leaf function the fixture gate matches.

**So the step neither price took is: for each fact the obj states, go read
whether a `crates/` module already produces it.** Reading the obj alone
under-counts; reading the obj and assuming every stated fact is unpaid
over-counts. This lane made the second error in its own prereg — three of the
five rows R5 registered as new were already there.

## 2. The sixteen

### IL decode — 6 (the `vocab-gap` side; the scan reports **one** key)

| # | mechanism |
|---|---|
| 1 | `expr-op-0x27` |
| 2 | `expr-op-0x32` |
| 3 | `expr-op-0x4B` |
| 4 | `expr-op-0x4F` |
| 5 | the composite terminal — `data-addr` receiver form, `off-add` **and** `chain-bind` blockers, suffix `-more`. `mcall.rs`'s key grammar defines `-more` as *"MEASURED: both together are still not enough"*, so **1..5 is a LOWER BOUND** |
| 6 | binding the C++ reference local `auto& listHead` through `.sy`/`.gl` |

### Emit — 8, the body's shape

| # | mechanism |
|---|---|
| 7 | the six-field store sequence to `this` |
| 8 | `stw 3,0(3)` — `this` stored into its **own** field |
| 9 | `addi 11,3,8` — an interior pointer as a **shared** producer for two stores |
| 10 | `li 10,0` — a literal producer |
| 11 | the producer hoist schedule — P1 at idx 0, P2 at 2, P3 at 5 |
| 12 | `mr 31,3` — `this` into a callee-saved GPR across the call, **and which one** |
| 13 | the `bl` with **zero** argument setup (r3, r4 pass through untouched) |
| 14 | `mr 3,31` — the constructor returns `this` |

### Emit — 2 that w-next's fourteen did not contain

| # | mechanism | why it is new |
|---|---|---|
| **15** | **the store run lives in a `calls-1` framed body.** `scheduled_gpr_run_text` is the only place under `crates/` that consults `order::schedule` and `alloc::allocate`, it is reached only from `store_leaf_text`, and it terminates by appending `encode_blr()` unconditionally (`leaf/store.rs:356`). `select_function` dispatches it as a **whole-body leaf**. There is no seam that emits a scheduled store run as the *middle* of a framed body | w-next §5.2 **named** this as "the second wall" and did **not** put it in the fourteen |
| **16** | **which register each store-run producer takes** — `addi`→r11, `li`→r10 | w-next's own stated blind spot |

## 3. The three rows R5 registered as new and that are already paid

| registered | verdict |
|---|---|
| the callee-saved choice `r31` for the live-across-call `this` | **already row 12** — w-next's row said "and *which* one" |
| the register-derived producer *kind* on the IL side | **already row 9** |
| the `mr 3,31` return-`this` epilogue | **already row 14** |

And two more the obj states that I checked before counting and did **not**
charge, because a `crates/` module already emits them:

* the **`.pdata` unwind word** `40 00 14 04` and the `.pdata` COMDAT with its
  ADDR32 relocation — `coff/pdata.rs`, and `select.rs:293` records W-UNW-1
  giving each framed function its own `.pdata` COMDAT;
* the **label counter state** — `$M2595` @0x10, `$M2596` @0x50, `$T2597` on the
  `.pdata` record, which the scan counts as `emit-label-syms 3`.
  `coff/label.rs:21-24` allocates exactly `$M(n)`, `$M(n+1)`, `$T(n+2)` for a
  framed function, and `docs/LABEL_COUNTER.md` is the write-up.

The frame itself (96 = `align16(80+8+8)`, `r31 → −16(r1)`) was confirmed free by
w-next's own grid and is not re-charged.

## 4. The allocator is NOT the binding constraint for this body

w-next left `codegen::alloc`'s mixed-kind refusal as the stopper. This lane's
grids change that reading in a specific way:

`xboxheap`'s run is `addi r11,3,8` (**2** uses) beside `li r10,0` (**1** use).
**Clause 1 alone settles it** — 2 beats 1, no tie — and the bonus this lane
measured (`H-self`: the interior pointer is stored into the object it points at,
`*(lh+0)=lh` and `*(lh+4)=lh`) pushes the *same* way. So the *answer* for this
body was never in doubt; what blocks is `allocate`'s **blanket** refusal of any
mixed run, which the fresh holdout now shows is **right in general** (7
refutations) and is **not the binding constraint here**.

Behind it stands row 15, which no relaxation of a gate reaches.
