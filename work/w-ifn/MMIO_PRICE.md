# W-IFN — `src/xdk/nuispeech/mmio.cpp`: the eleven RE-DERIVED, two of three bodies TAKEN, and `mmioClose` DECLINED at six

`w-blockir` priced this TU at **eleven distinct unbuilt mechanisms** in board
#1418's unit ("distinct clauses the port names"), read off the reference obj.
That count was taken against the **obj**; this one is taken against the
**port's own source**, which is a different question and gives a different
answer. It does not refute `w-blockir` — the two units are not comparable, and
that rung says so about its own predecessors for exactly this reason.

Everything below is either a `grep`/script result in this tree at `42fe7cb1` or
a cell this lane compiled and graded against real `c2.dll` under wibo. Nothing
is inherited from a rung, a board row or the commission.

---

## 1. The eleven, re-derived — SIX were already paid

| # | `w-blockir`'s mechanism | re-derivation | where |
|---:|---|---|---|
| 1 | a framed prologue/epilogue at a 96-byte frame | **PAID** — `FrameLayout{locals:0,out_slots:3,saved_gprs:0\|1}` emits `mflr r12 · stw r12,-8(r1) · [std r31,-16(r1)] · stwu r1,-96(r1)` and the matching teardown, byte for byte | `codegen/frame.rs:488`, `:523` |
| 2 | the materialised common epilogue (a forward join) | **PAID AS A PRIMITIVE, UNPAID AS A PRODUCTION.** `Selected::Framed` really has no join; `guard_chain_shared_tail`, `osf_handle_guard`, `alloc_init_or_fail` and `if_call_join` all emit one. What was missing was a *production*, and this lane wrote it | `codegen/guard_ret_chain.rs` |
| 3 | forward conditional branches on `cr6` in a framed body | **PAID** — `encode_bc(BO_FALSE, cr_bi(6, CR_BIT_EQ), d)` | `codegen/encode.rs:944` |
| 4 | the `.pdata` flag word, **computed** | **PAID** — `pdata_record` derives it from `Frame{prolog_len, func_len}`, and it reproduces all three of this TU's words (`40001503`, `40001b04`, `40001f04`) arithmetically. Pinned by a test in the new emitter | `coff/pdata.rs:71` |
| 5 | the coalesced two-register park | **PAID BY THIS LANE** — sub-shape G's swap through r11 | `codegen/guard_ret_chain.rs` |
| 6 | `memcpy`'s expansion cost model | **RE-DERIVED AND MUCH CHEAPER THAN ITS NAME.** At `/O1 /Oi` the boundary is a **step**: `n ≤ 5` expands, every `n ≥ 6` is a call. 25 cells. What was unpaid was a *reader* for the `40` selector-172 intrinsic, not a cost model | `work/w-ifn/probe/mcpy.cpp` |
| 7 | a callee-saved GPR live across a call | **PAID** — `guard_chain_shared_tail` parks `params[0]` in r31 across two `bl`s at `saved_gprs: 1`; sub-shape S does the same | `codegen/guard_chain_shared_tail.rs:PARK_REG` |
| 8 | a second relational regime (`cmplw` on two loaded values, `bf 24`) | **PAID AT THE ENCODER, PAID AT THE READER BY THIS LANE** | `encode_cmplw`, `CR_BIT_LT`; `guard_ret_chain`'s clamp |
| 9 | an indirect call (`lwz`/`mtctr`/`bctrl`) | **HALF PAID, STILL UNBUILT.** `encode_mtctr` exists; **`bctrl` has no encoder** — script-counted, §3 — and no `Selected` shape reaches an indirect call | `encode.rs:209` has the first, nothing has the second |
| 10 | `cr0` compares beside `cr6` ones in one body | **PAID** — `guard_chain_shared_tail` reads `r < 0` on cr0 (`2c030000`) and `r != S` on cr6 (`2f03fffe`) in one body | `codegen/guard_chain_shared_tail.rs` |
| 11 | an **ELIDED CALL** | **EXPLAINED (§2), STILL UNBUILT** | — |

**Two mechanisms `w-blockir` did not count**, both found by this lane's cells:

| # | mechanism | status |
|---:|---|---|
| **12** | the `mmioClose` park of a formal into **r5, a VOLATILE, across a `bl`** | **UNBUILT, and it is an interprocedural clobber fact.** §2.2 |
| **13** | the compiler-label charge | **PAID, and it is not free — it is one slot per TU.** §2.3 |

**So of `w-blockir`'s eleven: six were paid before this lane (1, 3, 4, 7, 8's
encoder half, 10), two are paid BY this lane (2's production, 5, and 6's
reader), and three remain (9, 11, 12).** All three are `mmioClose`'s and none is
`mmioGetInfo`'s or `mmioSetInfo`'s, which is why two of the three bodies ship.

---

## 2. The three that remain, and what each actually is

### 2.1 Mechanism 11 — the elided call, ANSWERED

**The call IS in the IL.** `work/w-ifn/probe/mmio_ex.txt` decodes `mmioClose`'s
`.ex` segment token by token; the statement at source line 60 is
`26 <mmioSetBuffer> BD 86 41 12 00 80 12 10 00 00 <four args> 4C 4B` — a bare
call statement with the result unused. So **c2 deleted it, not c1xx**, and
`elide.rs`'s opening claim that mechanism E happens behind the IL seam holds
here too.

Ten cells at `/O1` and again at `/Ob0` (`work/w-ifn/probe/elide.cpp`):

| cell | the callee | result | verdict |
|---|---|---|---|
| `e1` | same TU, `__declspec(noinline)`, `return 0` | unused | **ELIDED** |
| `e5` | same TU, **not** `noinline`, `return 0` | unused | **ELIDED** |
| `e6` | same TU, `return a + 1` — emits real bytes | unused | **ELIDED** |
| `e8` | same TU, defined **below** the caller | unused | **ELIDED** |
| `e9` | same TU, `void`, empty body, **non-tail** position | n/a | **ELIDED** |
| `e2` | same TU, stores to a TU-static | unused | kept |
| `e7` | same TU, calls an external | unused | kept |
| `e3` | **external** | unused | kept |
| `e4` | same TU, `return 0` | **used** | kept |

> **The rule: a call whose RESULT IS UNUSED and whose callee is defined in this
> TU with a body that has NO SIDE EFFECT is deleted.** Not `noinline`, not
> "constant body", not declaration order, not tail position. **Every verdict is
> identical at `/Ob0`**, so this is `elide.rs` mechanism **E**'s family and not
> the inliner.

It is strictly **wider** than shipped E, which requires the callee to *reduce
to nothing*: `e6`'s callee emits `addi r3,r3,1 ; blr` and the caller still drops
the call. `w-fix`'s GRID-3 row *"mechanism I mid-chain — `int m(int a){return
a;}` … `I` at both edges"* is not a counter-example: there the result is USED.

### 2.2 Mechanism 12 — the volatile park, and it is interprocedural

`mmioClose` puts `fuClose` in **r5** at `+0x14`, calls `mmioFlush` at `+0x30`,
and reads r5 at the `bctrl` at `+0x50`. r5 is volatile. That is only correct
because c2 knows `mmioFlush` — defined in this TU as `li r3,0 ; blr` — does not
clobber it. `work/w-ifn/probe/park.cpp`, five cells, all at the workload's own
flags:

| cell | the first callee | park | frame |
|---|---|---|---|
| `p1` | same TU, `return 0` | **r5** | 96, 1 saved GPR |
| `p3` | same TU, `return f + 1` | **r5** | 96, 1 saved GPR |
| `p2` | **external** | **r30** | **112, 2 saved GPRs** |
| `p4` | same TU, tail-calls an external | **r30** | **112, 2 saved GPRs** |
| `p5` | mmioGetInfo's park, isolated, external callee | r11 (the swap) | 96, 0 saved |

`p1` reproduces `mmioClose`'s 124 bytes exactly and `p2` is 136. **The register
plan of that body rests on a same-TU clobber analysis**, and `p5` is the
control that says `mmioGetInfo` and `mmioSetInfo` need no such fact — which is
why they ship.

### 2.3 Mechanism 13 — the label charge, PAID and NOT free

`work/w-ifn/LABEL_LEAD.md` measured the framed **stride** at the framed
constant (5 under `/Gy`, 4 packed) four ways and concluded the class needed no
label arm. **That conclusion was wrong**, and the differential caught it: the
TU's first `memcpy`-minting function takes **one extra slot before its own `$M`
triple**, once per TU.

```
  [framed, sub(memcpy)]                          stride 6
  [framed, sub1(memcpy), sub2(memcpy), framed]   strides 6, 5, 5
  [sub(memcpy), framed]                          stride 5      <- INVISIBLE
```

The third row is why every seed-free stride measurement missed it: a slot taken
before the FIRST function's triple moves that function's labels **and every
later one's equally**. `w-blockir` board #2305 recorded the mirror — a wrong
charge on the LAST function moves nothing after it. **A stride measurement is
blind at both ends of the list**, and that is this lane's contribution to the
method rather than to the class.

---

## 3. The script count, so the decline is not a hand partition

`work/w-ifn/price.py` reads the reference disassembly, takes every distinct
instruction mnemonic in a body, and asks whether
`crates/c2-core/src/codegen/{encode,calls}.rs` defines an encoder for it. It
counts encoders; it does not judge.

```
mmioGetInfo: 21 words, 13 distinct mnemonics   MISSING ENCODERS: 0
mmioSetInfo: 27 words, 16 distinct mnemonics   MISSING ENCODERS: 0
mmioClose:   31 words, 17 distinct mnemonics   MISSING ENCODERS: 1  ['bctrl']
```

That is the whole of what a script can count, and it is deliberately not
presented as the price: `bctrl` is one afternoon. The expensive part of
`mmioClose` is not a word the port cannot write.

---

## 4. `mmioClose` — the priced decline, at SIX, and the sixth is architectural

| # | mechanism | why it is not one afternoon |
|---:|---|---|
| **C1** | the `bctrl` encoder | it is; script-counted above |
| **C2** | an **indirect call** as a `Selected` shape — a callee that is a loaded value rather than a name, with no REL24 and no external symbol | every call shape in `select_function` carries a callee NAME; this one has none, so `coff::Call` cannot represent it |
| **C3** | a **bound call statement** — `26 <dst> <call> 4C 2C <T> 0 32 <T> 4B`, the call's result converted and stored to a local, then read twice | a new reader clause; `guard_ret_chain` has no statement form that binds a call result |
| **C4** | a **braceless early return on a call result**, on `cr0` — `53 B9 <dst> 33 <T> 0 20 38 <L> 53 … 3A <Lepi> … 54 04 29 <L> 54 03`, one scope shallower than the guard form this lane shipped | a second guard grammar, not a parameter of the first |
| **C5** | the **elided call** (§2.1) and the **volatile park** (§2.2) — counted as ONE because they are the same question asked twice: *what does this same-TU callee do* | two facts, one analysis |
| **C6** | **the acceptance seam for C5.** Board **#139** puts acceptance in the PARSER, and the parser sees exactly one `.ex` segment. There is no place in the port today where a **sibling function's body** can gate parser acceptance | this is the one that makes the decline a rung and not an afternoon |

**C6 is the reason this lane stops here, and it is worth stating precisely.**
`elide.rs` faces the same shape and resolves it at *emit* time — `Selected::Tail`
plus `drops_tail_call(f, tu)` — which is sound there because both outcomes are
valid objs and the census's claim ("this is a tail call") is true either way.
Here it is not sound: without the fact the emitted bytes are **wrong** — a `bl`
c2 does not emit, and a park in the wrong register with the wrong frame size.
So the fact has to gate *acceptance*, acceptance lives in the parser, and the
parser cannot see the sibling. Putting the check in `IlBundle::functions()`
instead would make the census and the gate disagree on a body the census calls
in class, which is the quantity board #139 keeps at zero.

**Not attempted, and not guessed at.** No cell of `mmioClose` was written into
`crates/`; the reader's own module doc names all three mechanisms and refuses.

---

## 5. What the decline costs, in the unit that maps to the goal

| | base | tip |
|---|---:|---:|
| mmio `.text` bytes the port produces a body for | **64 / 380** (16.8 %) | **256 / 380** (67.4 %) |
| mmio blocked / emitted functions | 3 / 11 | **1 / 11** |
| `fnbyte-exact` over the 878-TU workload | 35,774 | **35,776 (+2)** |
| TU match | 19 | 19 |

`mmioClose`'s **124 bytes are the entire remaining distance** of the frontier's
top byte-fraction row, and `frontier-bytefrac-top-accepted` moves `64 → 256` on
this lane's own scans. The TU does not convert, and that is the registered
modal outcome (PREREG §3 outcome **(B)**, p = 0.34) rather than a surprise.

---

## 6. The inlined-callee hazard, checked against the obj

`w-readpx` §5.2 measured five call-bearing classes at **0.000 over 1,106
emitted functions**, `framed-call` 0-for-123, *"because c2 inlines callees the
port keeps as calls"*. **This class is call-bearing, so unlike `w-blockir`'s the
hazard is not structurally absent** and the fence is load-bearing.

The fence is **callee-side and total**: the only call either shipped sub-shape
makes is to `memcpy`, which is not a function of the TU at all — it is minted
from an intrinsic selector, has no `.gl` record, and cannot be inlined by c2
because c2 has no body for it. Checked against the obj rather than asserted:
`work/w-ifn/ref/mmio.dump.txt` shows **exactly one REL24 in each of the two
`.text` COMDATs**, both against `memcpy`, and the port's own relocation plan for
those two functions is one `REL24 → memcpy` each — compared by offset, packed
type and target symbol NAME by `fnbyte-calltarget-*`, which reads
`agree 35,774 → 35,776` and `disagree` unmoved.

And §2.1 is the hazard's *other* face made concrete: `mmioClose` is a body where
c2 deletes a call the port would emit. That body is declined, which is the fence
doing its job rather than the fence being untested.
