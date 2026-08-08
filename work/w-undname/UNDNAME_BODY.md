# `?append@DName@@QAAXPAVDNameNode@@@Z` — the IL body and the 24 words, decoded

Lane `w-undname`, at master `5dd89969`, workload flags
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`). The IL itself is never
committed; this is the reading. Decoded against `docs/IL_STMT_GRAMMAR.md`'s
opcode tables and `crates/c2-il/src/func/body/mod.rs`'s own `opcode_name`
(`0x1F` `==`, `0x20` `!=`, `0x38` brfalse, `0x3A` jump, `0x27` member offset,
`0x30` indirect read, `0x2C` decay, `0x32` store, `0x33` literal, `0x99` the
`this` push, `0xB9` a value read, `0x26` a designator push).

**455 bytes from the `4C 4F 11` anchor to the `4D`, ONE linear token stream.** No
back-reference and no value merge at a join, so PREREG **D2 does not fire** — a
measurement on a built decode, not a guess.

## 1. The source (`src/xdk/LIBCMT/undname.cpp`, reconstructed from the stream)

```cpp
void DName::append(DNameNode *node) {
    if (node != 0) {
        pairNode *p = (pairNode *) gHeapManager.getMemory(16, 0);
        if (p != 0) {
            p->right    = node;
            p->refcount = -1;
            p->vtable   = pairNode_vtable;
            p->left     = this->node;
        }
        this->node = p;
        if (p == 0) goto error;
        goto done;
    }
error:
    this->status = 3;
done:
    ;
}
```

Tokens (this capture): `08 0a` `this` · `06 0a` `node` · `0b 0a` `p` ·
`05 0a` `?gHeapManager@@3V_HeapManager@@A` · `fe 09`
`?getMemory@_HeapManager@@QAAPAXHH@Z` · `e6 09` `?pairNode_vtable@@3PAXA` ·
labels `09 0a` (epilogue) `0a 0a` `0d 0a` `0f 0a` `10 0a` `11 0a` `12 0a`.

## 2. The stream

```text
  4c 4f 11 · 53 · 4f 01 2a · 53                    body, fn scope, line 42, if scope

  b9 060a <ptr> · 33 <ptr> 00 · 20 · 38 0a0a       node != 0  -> BF  L0a0a
  53 53 · 4f 01 2b

  26 0b0a                                          dest: p
  26 fe09                                          callee: ?getMemory
  26 050a · 2c <ptr> 00 · 99 <ptr> 00              the OBJECT ?gHeapManager,
                                                   decayed and pushed as `this`
  bd <call>
    33 <int> 00 · 55 <int>                           arg  0        <- REVERSE
    33 <int> 10 · 55 <int>                           arg 16           source order
  4c · 2c <ptr> 00 · 32 <ptr> · 4b                 the cast, then `p = …`

  4f 01 2c · 53
  b9 0b0a <ptr> · 33 <ptr> 00 · 20 · 38 0d0a       p != 0  -> BF  L0d0a
  53 53
    4f 01 2d  b9 0b0a · 33 <int> 08 · 27 · b9 060a · 32 · 4b     p->8  = node
    4f 01 2e  b9 0b0a · 33 <int> 0c · 27 · 33 <int> ff · 32 · 4b p->12 = -1
    4f 01 2f  b9 0b0a · 33 <int> 00 · 27 · 26 e609 · 32 · 4b     p->0  = vtable
    4f 01 30  b9 0b0a · 33 <int> 04 · 27 ·
              b9 080a · 33 <int> 00 · 27 · 30 · 2c · 32 · 4b     p->4  = this->0
  4f 01 31 · 54 08 · 4f 01 32 · 54 07
  29 0d0a · 54 06                                  L0d0a:

  b9 080a · 33 <int> 00 · 27 · b9 0b0a · 2c · 32 · 4b            this->0 = p

  4f 01 33 · 53
  b9 0b0a <ptr> · 33 <ptr> 00 · 1f · 38 0f0a       p == 0  -> BF  L0f0a
  53 53 · 4f 01 34
    3a 110a                                        goto L110a  (the `goto error`)
    3a 100a                                        (unreachable, emitted anyway)
  4f 01 35 · 54 08 · 4f 01 36 · 54 07
  29 0f0a · 54 06 · 54 05 · 54 04                  L0f0a:
  3a 120a                                          goto L120a  (skip the error)

  29 0a0a · 53 53 · 4f 01 37                       L0a0a:  the `node == 0` arm
  3a 100a                                          goto L100a
  29 110a · 3a 100a                                L110a:  goto L100a
  29 100a · 4f 01 38                               L100a:  error:
  b9 080a · 33 <int> 04 · 27 · 33 <byte> 03 · 32 <byte> · 4b     this->4 = 3
  4f 01 39 · 54 05 · 54 04
  29 120a · 54 03                                  L120a:
  4f 01 3a · 3a 090a · 54 02 · 29 090a             return -> the epilogue label
  4f 12 47 · 54 01 · 54 00 · 4f 02 20 00 · 4f 01 3b · 4d
```

**Three transfers reach the error block and it carries two labels** — `L110a`,
whose only statement is `3a 100a`, and `L100a` itself. That is the `goto` the
price named, and it costs the recognizer one clause rather than a block IR: the
walk is forward-only and each label is checked against the transfer that named
it.

## 3. The 24 words, and the pairing

`work/w-extdata/ref/undname/dis.txt` is the ground truth. PREREG §1.5 tabulates
the words. The one thing that must be read here rather than there:

```text
  +0x24  lis  r11,0      REFHI ?gHeapManager      ┐ pair 0 — low half is an
  +0x2c  addi r3,r11,0   REFLO ?gHeapManager      ┘ ARG_REG, 2 words below
  +0x40  lis  r11,0      REFHI ?pairNode_vtable   ┐ pair 1 — low half is the
  +0x4c  addi r11,r11,0  REFLO ?pairNode_vtable   ┘ SCRATCH, 3 words below
```

Two different hoist distances in one body, and one low half writing the scratch
register the high half lives in. Any derivation by a *fixed* distance, or by a
search over `addi <ARG_REG>,r11,0`, gets one of the two wrong — so the pairing is
positional: each `addis rT,0,0` opens a pair and the first `addi rD,rT,0` after
it closes it, `rD` unconstrained.

## 4. The symbol table, which is GRID A's rule

```text
  [15] ?pairNode_vtable@@3PAXA               sec=0  type=0x0000   first ref +0x40
  [16] ?getMemory@_HeapManager@@QAAPAXHH@Z   sec=0  type=0x0020   first ref +0x34
  [17] ?gHeapManager@@3V_HeapManager@@A      sec=0  type=0x0000   first ref +0x24
```

Strictly descending index against ascending first-reference offset, **for both
kinds alike**. `data · callee · data`, which no ordering of a callee loop and a
data loop can produce.
