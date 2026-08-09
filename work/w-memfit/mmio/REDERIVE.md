# `mmio.cpp` — RE-DERIVED AT BASE `8dd1a577`, and the price MOVED

Everything here was measured this session with `c2rs census` / `c2rs gap` /
`c2rs compile` against the real `c2.dll` under wibo at the workload's own
flags. Nothing is inherited. Inherited prices have been wrong six times this
week; this is the seventh, and it moves in the **cheap** direction.

## 1. The TU, at base

```
$ c2rs census src/xdk/nuispeech/mmio.cpp --flags-file work/dc3-workload/flags.txt --cwd $C2RS_DC3
src/xdk/nuispeech/mmio.cpp -> 8/11 functions in class
```

`c2rs gap` on the same TU: `class vocab-gap`, `fn_total 11`, `fn_in_class 8`,
`fn_blockers {"expr-cmp-eq": 3}`, `fn_gate_refusals {}`.
`work/w-memfit/mmio_gap.jsonl`. **w-memcpy's 8/11 reproduces to the
function** (PREREG P15).

## 2. `?mmioGetInfo` — w-park's ladder RE-RUN, and one of its two rungs is PAID

`work/w-park/cells/lad_getinfo.cpp`, unmodified, at this base:

```
lad_getinfo.cpp -> 4/5 functions in class
  [0] ok  call-sequence-early-return      L0  the shipped shape
  [1] ok  call-sequence-early-return      L1  + a third unused formal
  [2] ok  call-sequence-early-return      L2  + a widened return type
  [3] ok  call-sequence-early-return      L3  + a three-slot call with a LITERAL
  [4] GAP expr-cmp-eq                     L4  + the callee renamed `memcpy`
```

**w-park recorded L3 as a GAP on `call-arg-lit-permuted` and priced the body
at two.** It is in class here — `w-park` shipped the `ArgSite` widening in the
same rung whose decline quoted the unpaid price, so its own decline inherited
a number its own commit had already reduced.

## 3. The `2C` conversion is free too — `work/w-memfit/mmio/lad2.cpp`

L4's arguments are `void*` formals passed to `memcpy`'s `void*` parameters, so
no conversion is minted and L4 does **not** isolate the `2C` that w-memcpy §2
read on each of `?mmioGetInfo`'s three arguments. Two more rungs:

```
lad2.cpp -> 1/2 functions in class
  [0] ok  call-sequence-early-return      L5  typed pointers, ORDINARY callee
  [1] GAP expr-cmp-eq                     L6  typed pointers, `memcpy`
```

## 4. The decisive check: L5 and L6 are BYTE-IDENTICAL to the pin

`c2rs compile lad2.cpp` at the workload's flags, `.text` per COMDAT against
`c2_core::codegen::frontier_bytes::C2_MMIOGETINFO_TEXT` — the 84 bytes real
`c2` emitted for `?mmioGetInfo`, already in the tree:

```
?L5@@YAKPAUHMMIO__@@PAUMMIOINFO_@@I@Z   84 B  BYTE-IDENTICAL   reloc 0x3c -> ?g3n@@YAXPAXPBXI@Z
?L6@@YAKPAUHMMIO__@@PAUMMIOINFO_@@I@Z   84 B  BYTE-IDENTICAL   reloc 0x3c -> memcpy
```

and the port emits L5's **whole obj** byte-exact:

```
$ c2rs gap --list l5list.txt --flags-file ../../dc3-workload/flags.txt --cwd .
  match 1  100.0%   mismatch 0   vocab-gap 0   graded by shape x verdict: seq/exact 1
```

**So `?mmioGetInfo`'s entire remaining distance is one word in the symbol
table.** The port already produces all 84 bytes, including the relocation
site, for a function that differs from it only in how the callee is spelled.

## 5. The price, re-derived — and what it is NOT

w-memcpy §6.3 priced this body at **five** independent refusals. Four of the
five are re-checked here:

| w-memcpy §6.3's item | at this base |
|---|---|
| `call-arg-lit-permuted` in front of everything | **PAID** — L3 in class (§2) |
| each pointer argument carries a `2C` | **PAID** — L5 in class (§3) |
| the expansion decision has no rule | **MEASURED** — R-MEMFIT, 724/724 over five grids (`work/w-memfit/allscore.py`); for this cell the hints are `01` and `04`, `min = 1`, `n = 72/1 = 72 > 5` ⇒ **CALL**, which is what the obj carries |
| the `40` token is not a call head; the sequence loop never consumes the statement | **UNPAID** |
| the callee has **no `.gl` token**, so `bundle::resolve` cannot produce it and the symbol must be minted *and placed* | **UNPAID** |

**Two, not five** — and the two are one piece of reader work and one piece of
emitter work. The emitter half has a precedent: `crates/c2-core/src/comdat.rs`
already mints `__savegprlr_28`/`__restgprlr_28` from the frame layout rather
than from the IL, so minting an external is not new; what is new is minting one
whose **index and ordering** in the symbol table must match c2's (the reference
obj names symbol `[19]`).

## 6. Why the TU still does not convert

A TU match is a conjunction over every emitted function. The other two bodies
are not one construct away from anything — their own ladders, re-run at this
base:

```
lad_setinfo.cpp -> 1/5 in class      rungs 1..4 blocked
lad_close.cpp   -> 3/7 in class      rungs 3..6 blocked
```

Re-derived TU price: **1 + 4 + 4 = 9 constructs**, against `w-park`'s 12
(PREREG P14 registered ±4 and this is inside it). `?mmioGetInfo` is 84 of the
TU's 380 `.text` bytes; converting it alone would move the byte fraction
64/380 → 148/380 and the TU verdict **not at all**.
