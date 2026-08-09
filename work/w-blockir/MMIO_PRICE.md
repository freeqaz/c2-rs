# W-BLOCKIR — `src/xdk/nuispeech/mmio.cpp`, the DECLINE and its price

PREREG clause **D3**: this TU is **not attempted**, and its price is re-derived
at base rather than inherited. Every mechanism below is read off
`work/w-blockir/ref/mmio.dis.txt` — the real obj `cl.exe` 16.00.11886.00
produced under wibo at the workload's own flags, captured by this lane.

## Why it was the other half of the commission, and why it is not the half taken

`w-band` (#2240–#2247) and `w-readpx` (#2280–#2293) independently found that
`expr-cmp-eq` is the only key owning two frontier TUs — this one and
`IPP_basicmath_xbox.cpp` — and that both are `ABC--`, so function bytes are the
whole remaining distance. The commission licensed taking one and pricing the
other if they need different things. **They need different things**, and the
difference is not marginal:

| | `IPP_basicmath_xbox.cpp` | `mmio.cpp` |
|---|---|---|
| blocked / emitted | 4 / 4 | 3 / 11 |
| `.text` bytes remaining | 184 of 184 | **316** of 380 |
| frame | none | all three blocked bodies are framed, 96 B |
| `.pdata` | none | three records, three different flag words |
| relocations | **zero** | five, including one indirect and one intra-TU |
| labels | **label-free** | **9** |
| calls | none | `memcpy` ×2, `mmioFlush`, an indirect `bctrl`, `FreeHandle`, and one **elided** |

## The price at base: ELEVEN distinct unbuilt mechanisms over three bodies

Counted as *distinct mechanisms the port does not have*, which is #1418's unit
(`≥ 8 distinct clauses`) and **not** #483/#506/#827's per-body independent-refusal
unit (17) nor #2136's re-derivation (9). Board #1418 states the two are not
comparable: *"`mmio`'s >= 8 is NOT a re-price of #483/#506/#827's 17 and does not
refute it — those count independent refusals per body, this counts distinct
clauses the port names, and the units are not comparable."* This re-derivation is
in #1418's unit.

| # | mechanism | where it is visible | bodies |
|---:|---|---|---:|
| 1 | a framed prologue/epilogue at a 96-byte frame with `mflr`/`stw`/`stwu` and the matching teardown | `+0x00`, `+0x44` | 3 |
| 2 | **the materialised common epilogue** — two or three early-return sites (`li r3,5`, `li r3,11`) each branching forward into one shared block. `Selected::Framed` emits prologue+body+epilogue as one straight-line run with **no representation for a join** | `b .+36`, `b .+20` | 3 |
| 3 | forward conditional branches on `cr6` (`bf 26,.+12`) — a three-way CFG inside a framed body | `+0x18`, `+0x24` | 3 |
| 4 | the `.pdata` flag word, which must be **computed** from prologue length and saved-register count: `40 00 15 03`, `40 00 1b 04`, `40 00 1f 04` are three different words for three bodies | the `.pdata` sections | 3 |
| 5 | the coalesced two-register park (`mr r11,r3` + `mr r3,r4`; `mr r31,r3` + `mr r5,r4`) | `+0x0c`, `+0x10` | 3 |
| 6 | **`memcpy`'s expansion cost model** — board #1925: *"`expr-intrinsic-memcpy` IS A MEASURED NON-RULE, NOT AN UNBUILT ONE … four separately frozen thresholds all miss"* | `bl memcpy` at `+0x3c` | 2 |
| 7 | a **callee-saved GPR across a call** (`std r31,-16(1)` / `ld r31,-16(1)`) — `docs/CFG_SHAPE.md` §6.2 item **F**, values live across a block boundary, the one item of the block-IR spec with no measurement behind it | `+0x08`, `+0x64` | 2 |
| 8 | a second relational regime: `cmplw cr6,r10,r11` on two **loaded** values with `bf 24` (GT, not EQ) and a conditional store | `mmioSetInfo +0x48` | 1 |
| 9 | an **indirect call** through a loaded member: `lwz r11,8(31)` · `mtctr r11` · `bctrl` — no encoder, no relocation, no representation | `mmioClose +0x3c` | 1 |
| 10 | **`cr0` compares beside `cr6` ones in one body** — `cmplwi r3,0` into crf 0 after each call result, with `bf 2` | `mmioClose +0x34`, `+0x54` | 1 |
| 11 | an **ELIDED CALL** — the source calls `mmioSetBuffer(hmmio,0,0,0)` and the obj carries **no branch for it**, though `mmioSetBuffer` is `__declspec(noinline)` and its body is `li r3,0 ; blr`. c2 dropped a call whose result is unused. Nothing in the port models this, and `crates/c2-core/src/elide.rs`'s mechanism E is a *tail* call to an **empty** body, which this is not | `mmioClose`, between `+0x58` and `+0x5c` | 1 |

Item **11** is the one worth carrying off this page. It is the inlined-callee
hazard `w-readpx` §5.2 measured — five call-bearing classes at **0.000 over 1,106
emitted functions**, *"because c2 inlines callees the port keeps as calls"* — in
its most concrete form: here the callee is not inlined, it is **deleted**, and
the port would emit a `bl` c2 does not.

## The inlined-callee check on this lane's own class, for contrast

The class this lane ships admits **no call-bearing body at all**: shapes A, B and
C are leaf loops with zero call edges and zero relocations, and
`IPP_basicmath_xbox.cpp`'s obj has **0 relocations** in all four sections. So the
bimodality that makes `framed-call` 0-for-123 and `call-sequence-cmp-eq`
0-for-542 cannot reach it, and the fence `w-inlfence` (#2220–#2227) exists to
apply has nothing to apply to. That is not an argument that the class is safe —
it is the statement that the *named* hazard is structurally absent, checked
against the obj's own relocation count rather than asserted.

## What this lane did NOT do to mmio

It did not compile a single mmio cell beyond the one reference obj above, did not
re-derive #2136's 9, did not attempt the memcpy threshold, and proposes no rung.
The eight lanes that priced this TU before are cited, not re-run.
