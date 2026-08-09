# `src/xdk/nuispeech/mmio.cpp` — the chain RE-DERIVED at base `8dd1a577`

`w-park` (2026-08-08) priced this TU at **twelve** and recorded three bodies with
three different first refusals. Inherited prices have been wrong six times this
week, so every line below was re-measured this session. **Two of w-park's
entries have moved, in opposite directions.**

## 1. The base facts, from this lane's own scan

`work/w-memfit/scan_base.out`, 878 TUs, workload flags:

```text
  FRONTIER BY `.text` BYTE FRACTION (board #500)
       accepted/total bytes    frac   exact  remain | src
            64/380      bytes   16.8%     64     316 | src/xdk/nuispeech/mmio.cpp
  FRONTIER BY CODEGEN (board #1474)
       den exact wrong cg-ref reader ungrade | src
        11     8     0      0      3       0 | src/xdk/nuispeech/mmio.cpp
```

Unchanged from `w-park`'s base to the byte: **64 / 380, 316 remaining, 8 of 11
emitted functions byte-exact, 3 behind the reader.** Still the frontier's top
byte-fraction row. (PREREG C3.)

## 2. The three blocked bodies, and the first refusal each reaches TODAY

`c2rs census` reports only the fall-through key — all three read `expr-cmp-eq`,
which says nothing. The real clause was read with the **decline probe**
(`w-park`'s method, `work/w-park/decline_probe.md`): the `Err` thrown away at
`early_return.rs`'s call to `parse_call_sequence_from` was printed instead of
dropped, the census was run, and **the patch was reverted before anything was
committed** — `git status` clean, `git diff` empty, the binary rebuilt from the
reverted tree.

| body | IL | `w-park` recorded | **re-derived at `8dd1a577`** |
|---|---:|---|---|
| fn 0 — `mmioGetInfo`, 84 B `.text` | 286 B | `callseq-tail-lit` | **`callseq-tail-lit:mid`** — same |
| fn 1 — 108 B `.text` | 419 B | `call-token-0xB9` | **`callseq-tail-lit:mid` — MOVED** |
| fn 7 — 124 B `.text` | 512 B | `call-token-0x26` | **`call-token-0x26`** — same |

**Two of the three now stop on the same clause**, and it is the memcpy
intrinsic reaching the tail parse with the statement unread. `w-park`'s
`call-token-0xB9` for the middle body is superseded: the boundary moved under it
when that lane shipped its own `ArgSite::Sequence` change.

The body sizes and the identification are the **reference obj's own**, dumped
this session (`work/w-memfit/mmio_ref.obj`, `scripts/gt_dump.py`) rather than
inherited:

```text
  .text  #5   84 B  mmioGetInfo      one REL24 -> [19] memcpy
  .text  #7  108 B  mmioSetInfo      one REL24 -> [19] memcpy  (at 0x3c)
  .text #14  124 B  mmioClose
  eight further .text sections at 8 B each                      = 64 B
  64 + 84 + 108 + 124 = 380
```

**`mmioSetInfo` calls `memcpy` too** — that is the mechanism behind the moved
clause, and it is why two bodies now stop at the same place rather than at two.
`w-park` recorded `memcpy` as `mmioGetInfo`'s remaining distance; on the obj it
is **two of the three bodies' first refusal and 192 of the 316 remaining
bytes**.

## 3. The construct ladder, re-run — L3 is PAID

`c2rs census work/w-park/cells/lad_getinfo.cpp --flags-file work/w-park/o1.txt`:

```text
  4/5 functions in class
  [0] ok  call-sequence-early-return  cflow-if-2      L0  the shipped shape
  [1] ok  call-sequence-early-return  cflow-if-2      L1  + an unused third formal
  [2] ok  call-sequence-early-return  cflow-if-2      L2  + return type widened
  [3] ok  call-sequence-early-return  cflow-if-2      L3  + a 3-slot call with a LITERAL
  [4] GAP expr-cmp-eq                 cflow-if-2      L4  + the callee named `memcpy`
```

**L3 was a `GAP — call-arg-lit-permuted` when `w-park` ran this ladder and is in
class now.** Better still, `work/w-park/cells/l3.cpp` as a whole TU is a
**`match`** at `/O1` — graded by the differential this session, not asserted. So
`?mmioGetInfo`'s exact instruction stream *is* something the port emits
byte-exact today; the whole distance on that body is the callee's name.

That confirms `w-park`'s own claim from the other side, and it moves the price:
**`call-arg-lit-permuted` is off `mmioGetInfo`'s chain.**

## 4. What `?mmioGetInfo`'s 84 bytes still cost — FOUR, re-derived

Not twelve; twelve is the **TU**. For this one body:

| # | refusal | status | evidence, this session |
|---|---|---|---|
| N1 | the `40` intrinsic token is not a call head, so the sequence loop reaches the tail parse unread | **UNPAID** | the decline probe: `callseq-tail-lit:mid` on `l4.cpp` and on mmio fn 0 |
| N2 | which way the expansion goes — `bl` or an inline copy | **PAID by this lane** | 624/624; and mmio's own case is unambiguous either way (the IL carries hints `01` and `04`, and `72/1 = 72` and `72/4 = 18` are both `> 5`, so it is a **call** — which is what the reference obj's REL24 to `memcpy` says) |
| N3 | the callee has **no `.gl` token**: the symbol must be minted and placed | **UNPAID** | the `.gl` of a one-`memcpy` TU carries `?f@@…`, `.XBLD$W`, `__C1_11886`, `/include:` and no `memcpy`; the obj carries `[14] memcpy sc=EXTERNAL sec=0 type=0x0020` |
| N4 | five IL argument operands reduce to three emitted slots (two alignment hints and a size that is not an argument register) | **UNPAID** | `work/w-memfit/probeH` — the two hint bytes at `.ex` 2733/2742, the size beside them |
| N5 | each pointer argument carries a `2C` conversion | **UNPAID** | `w-memcpy` §2, re-read on this lane's own captures |
| — | `call-arg-lit-permuted` — the `[Formal(1), Formal(0), Lit(72)]` slot list | **PAID by `w-park`** | §3, and `l3.cpp` is a whole-TU `match` |

**Four unpaid on the cheapest of the three bodies.** `w-memcpy` §6.3 priced this
clause at five; one of the five is paid, one is new (`w-park`'s), and the
arithmetic nets to four.

## 5. What the TU costs — the other two bodies, unpriced by this lane

fn 1 now stops at the same clause as fn 0, so N1/N3/N4/N5 recur there, **and
what is behind them is `w-park`'s M7** (the callee-saved park, `std/ld r31`, and
a post-call conditional member store that is none of `SeqTail`'s three forms).
fn 7 stops at `call-token-0x26`, a guard on a **call result**, with `w-park`'s
M8–M10 behind it: an indirect call through a loaded member, `cr0` result
compares beside `cr6` formal compares, a coalesced park, and an **elided** call
the port must not emit. Plus M11, three framed functions' `$M`/`$M`/`$T` triples
and `.pdata` with the label counter running across all eleven.

**This lane did not re-price M7–M11 body by body** and says so rather than
reporting `w-park`'s twelve as if it had. What it re-derived is that **two of
twelve moved**: M5 (`callseq-multiarg-lit`) is paid, and M4's neighbour on the
middle body is a different clause than recorded.

## 6. The CFG-reachability screen over-states this TU by one class

The scan's board-#720 line reads

```text
   3 blocked | src/xdk/nuispeech/mmio.cpp | labels 9 | needs a CFG class the
              port lacks: cflow-if-2, cflow-if-n
```

**`cflow-if-2` is not a class the port lacks.** `work/w-park/cells/l3.cpp` is a
`cflow-if-2` body and a whole-TU **`match`** at `/O1`, graded this session; the
`SeqEarlyReturn` shape has emitted that class since W11. What is missing from
`PORT_CFG_CLASSES` (`crates/c2-harness/src/gap/factors.rs`) is the `cflow-if-2`
entry, and that list's own doc already records one deliberate omission
(`cflow-loop`) without recording this one.

**The delta is zero on this workload** and the row is filed rather than fixed:
`mmio.cpp` is the only frontier TU whose "needs" list mentions `cflow-if-2`, and
it names `cflow-if-n` as well, so `reach` stays at 2 of 9 either way. Widening
an instrument is a claim and belongs to a lane that has registered it. Board
row minted; **this lane changed no instrument.**
