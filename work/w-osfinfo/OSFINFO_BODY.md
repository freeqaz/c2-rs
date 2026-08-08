# `_free_osfhnd` — the IL body and the 31 words, decoded

Lane `w-osfinfo`, at master `b96a3f19`, workload flags
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`). The IL itself is never
committed; this is the reading. Decoded against `docs/IL_STMT_GRAMMAR.md`,
`docs/IL_EXPR_LAYER.md` and `crates/c2-il/src/func/body/mod.rs`'s own
`cflow_opcode_name` / `expr_opcode_name` (`0x20` `!=`, `0x22` `<`, `0x23` `>=`,
`0x0A` `>>`, `0x0B` `&`, `0x04` `*`, `0x26` designator push, `0x27` member
offset, `0x28` the **subscript add** — two bytes whose meaning is unknown,
`docs/IL_EXPR_LAYER.md` §4 — `0x2C` convert, `0x30` indirect read, `0x32` store,
`0x33` literal, `0x38` brfalse, `0x3A` jump, `0x41` the RESULT-type/return
operator, `0xB9` a value read, `0xBD` a call).

**381 bytes from the `4C 4F 11` anchor to the `4D`, ONE linear token stream.**
No back-reference and no value merge at a join, so PREREG **D2 does not fire** —
a measurement on a built decode, not a guess.

## 1. The source, reconstructed from the stream

```c
int _free_osfhnd(int fh) {
    if (fh >= 0 && (unsigned)fh < (unsigned)_nhandle) {
        int i = fh >> 5;
        ioinfo *pio = (ioinfo *)((char *)__pioinfo[i] + (fh & 31) * 72);
        if ((pio->osfile & 1) != 0) {          /* byte field at +4 */
            if (pio->osfhnd != -1) {           /* word field at +0 */
                pio->osfhnd = -1;
                return 0;
            }
        }
    }
    *_errno()     = 9;                          /* EBADF */
    *__doserrno() = 0;
    return -1;
}
```

Tokens (this capture): `f7 09` `fh` · `f3 09` `_nhandle` · `f4 09` `__pioinfo` ·
`e5 09` `_errno` · `f6 09` `__doserrno` · `fa 09` `i` · `fb 09` `pio` ·
labels `f9 09` (epilogue) `fe 09` (the range-failure arm) `00 0a` (the
field-failure arm).

## 2. The stream

Offsets are from the `4C` of the `4C 4F 11` anchor.

```text
  0000  4c 4f 11 · 53 · 4f 01 1a · 4f 01 1b · 4f 01 1d · 53   body, fn scope, if scope

  000e  b9 f709 <u4> · 2c <i4> 00 · 33 <i4> 00 · 23 · 38 fe09   fh >= 0   -> BF Lfe09
  0022  b9 f709 <u4> · b9 f309 <i4> · 2c <u4> 00 · 22 · 38 fe09 fh < _nhandle -> BF Lfe09
  0037  53 53 · 4f 01 1e

  003c  26 fa09 · b9 f709 <u4> · 2c <i4> 00 · 33 <i4> 05 · 0a
        32 <i4> · 4b                                   i = fh >> 5

  0055  4f 01 1f
        26 fb09 · 26 f409                              dest pio, base __pioinfo
        b9 fa09 <i4> · 33 <T2> 04 · 04 · 28 00 00      __pioinfo + i*4
        30 <p4>                                        …dereferenced   (the lwzx)
        b9 f709 <u4> · 33 <i4> 1f · 0b                 fh & 31
        33 <T2> 48 · 04 · 28 00 00                     …*72, added     (the add)
        32 <p4> · 4b                                   -> pio

  008d  4f 01 20 · 53
        b9 fb09 <p4> · 33 <i4> 04 · 27 <T82a>          pio->4
        30 <T82b> · 2c <i4> 00                         a BYTE read, widened
        33 <i4> 01 · 0b · 33 <i4> 00 · 20 · 38 000a    (…&1) != 0 -> BF L000a
        b9 fb09 <p4> · 33 <i4> 00 · 27 <T3>
        30 <T4> · 33 <T4> ff · 20 · 38 000a            pio->0 != -1 -> BF L000a
  00d8  53 53 · 4f 01 21
        b9 fb09 <p4> · 33 <i4> 00 · 27 <T3>
        33 <T4> ff · 32 <T4> · 4b                      pio->0 = -1
  00f8  4f 01 22 · 33 <T8> 00 · 41 <T8> · 3a f909      return 0
  0107  4f 01 23 · 54 08 · 4f 01 24 · 54 07
  0111  29 000a · 54 06 · 54 05                        L000a:
  0118  4f 01 26 · 54 04
  011d  29 fe09 · 54 03                                Lfe09:
  0122  26 e509 · bd <T5> 00 80 07 10 00 00 · 4c
        33 <i4> 09 · 32 <i4> · 4b                      *_errno() = 9
  013b  4f 01 27
        26 f609 · bd <T5> 00 80 07 10 00 00 · 4c
        33 <i4> 00 · 32 <i4> · 4b                      *__doserrno() = 0
  0157  4f 01 28 · 33 <T8> ff · 41 <T8> · 3a f909      return -1
  0166  4f 01 29 · 54 02 · 29 f909                     Lf909:  the epilogue
  016e  4f 12 47 · 54 01 · 54 00 · 4f 02 20 00 · 4f 01 2a · 4d
```

Type shorthands, verbatim from the capture:
`<i4>` = `86 41 74` (int) · `<u4>` = `86 42 75` (unsigned) ·
`<T2>` = `86 41 12` (the scaling type the two multiplies carry, and the type of
the `osfhnd` word) · `<p4>` = `86 43 81 20` (`ioinfo *`) ·
`<T3>` = `86 43 92 08` · `<T4>` = `86 41 12` · `<T5>` = `86 43 f4 08` (the
`int *` both callees return) · `<T8>` = `88 81 13` (the function's own return
type, which the `41` operator carries) · `<T82a>` = `82 43 f0 08` and
`<T82b>` = `82 11 70` — **the `0x82` type tag family, which no prior class of
this seam has seen**; it is the `osfile` byte field and its containing struct.

**Three labels, and only two blocks.** `L000a` and `Lfe09` are two *distinct*
labels that fall through into one another — `29 000a · 54 06 · 54 05 · 4f 01 26
· 54 04 · 29 fe09 · 54 03` has no statement between them — so all four guards
reach the same error block through two label definitions and zero jumps.
That is why the emitted body has **one** `Lerr` at 0x68 with four `bc`s naming
it, and it is the fact a recognizer that collapsed the two labels would get
right by accident and a recognizer that required a jump between them would
refuse.

## 3. The 31 words, and the five things about them

`work/w-osfinfo/ref/osfinfo/dis.txt` is the ground truth. PREREG §1.3 tabulates
the words. The five a general lowering gets wrong:

1. **The two entry guards read the SAME condition register and DIFFERENT
   compare forms.** `fh >= 0` is `cmpwi cr6,r3,0` (signed, immediate) and
   `fh < _nhandle` is `cmplw cr6,r3,r11` (**unsigned, register**). The IL says
   so — the first converts `fh` to `<i4>` and the second converts `_nhandle` to
   `<u4>` — and a class that used one form for both emits the right program with
   one wrong word.
2. **Both guards branch on the LT bit, one true and one false.**
   `bt 24` then `bf 24`. `>=` becomes *branch if LT*, `<` becomes *branch if not
   LT*: the relation's polarity, not the operand order.
3. **The `& 1` test is a RECORD-form `rlwinm` and reads cr0.** `clrlwi.
   r10,r10,31` sets cr0 with no compare instruction at all, and the branch is
   `bt 2`. A class that emitted `andi.`, or a `clrlwi` plus a `cmpwi`, is one or
   two words longer and still links.
4. **The address chain uses `slwi` for ×4 and `mulli` for ×72, in one body.**
   Two multiplies, two different instructions, chosen by whether the constant is
   a power of two — the exact shape of a chooser D1 forbids, which is why the
   reader pins the scale to 4 and refuses a power-of-two element size.
5. **The final store is TAIL-MERGED across two unrelated statements.**
   `stw r10,0(r11)` at 0x84 is reached from the success path (r11 = `pio`,
   r10 = −1, i.e. `pio->osfhnd = -1`) *and* from the error path (r11 =
   `__doserrno()`, r10 = 0). One word serves both, and it is only legal because
   `off_hnd == 0`.

**Eleven free immediate fields** and **zero words chosen by a scheduler or an
allocator**, which is what PREREG **D1** registered.

## 4. The symbol table, which is board #1720's rule again

```text
  [15] __doserrno   sec=0  type=0x0020   first ref +0x74
  [16] _errno       sec=0  type=0x0020   first ref +0x68
  [17] __pioinfo    sec=0  type=0x0000   first ref +0x28
  [18] _nhandle     sec=0  type=0x0000   first ref +0x14
```

Strictly descending index against ascending first-reference offset, for both
kinds alike — `callee · callee · data · data` this time, where `undname.cpp`'s
was `data · callee · data`. The merged rule w-undname shipped predicts both;
**two loops would predict this one correctly and `undname`'s wrongly**, which is
why this TU alone could never have established the rule and why it is a second
cell rather than a first.
