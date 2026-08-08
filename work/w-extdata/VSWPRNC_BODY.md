# `_vswprintf_s_l` — the IL body and the 39 words, decoded

Lane `w-extdata`, at master `3168b4e9`, workload flags
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`). The IL itself is never
committed; this is the reading. Decoded against `docs/IL_STMT_GRAMMAR.md`'s
opcode tables and `crates/c2-il/src/func/body/mod.rs`'s own `opcode_name`
(`0x1F` `==`, `0x20` `!=`, `0x22` `<`, `0x38` brfalse, `0x39` brtrue,
`0x3A` jump).

**429 bytes from the `4C 4F 11` anchor to the `4D`, ~40 statements, ONE linear
token stream.** No nesting a pattern matcher cannot walk and no value merge at a
join, so PREREG **D2 does not fire** — a measurement on a built decode, not a
guess.

## 1. The source (`src/xdk/LIBCMT/vswprnc.cpp`, verbatim body)

```c
int _vswprintf_s_l(wchar_t *buffer, size_t sizeInWords, size_t count,
                   const wchar_t *format, void *locale, va_list arglist) {
    int result;
    if (count == 0 || buffer == 0 || sizeInWords == 0) {
        *_errno() = 0x16;  _invalid_parameter_noinfo();  return -1;
    }
    result = _vswprintf_helper(_woutput_s_l, buffer, sizeInWords, count,
                               format, locale, arglist);
    if (result < 0)  *buffer = 0;
    if (result != -2) return result;
    *_errno() = 0x22;  _invalid_parameter_noinfo();  return -1;
}
```

Tokens (this capture): `3d 0b` `buffer` · `3e 0b` `sizeInWords` · `3f 0b` `count`
· `40 0b` `format` · `41 0b` `locale` · `42 0b` `arglist` · `45 0b` `result` ·
`22 0b` `_errno` · `3c 0b` `_invalid_parameter_noinfo` · `33 0b`
`_vswprintf_helper` · `3a 0b` `_woutput_s_l` · labels `44 0b` (epilogue)
`46 0b` `47 0b` `48 0b` `49 0b`.

## 2. The stream

```text
  4c 4f 11 · 53 · 4f 01 0e · 4f 01 10 · 53      body, fn scope, lines, if scope

  b9 3f0b <uint> · 33 <uint> 00 · 1f · 39 470b  count == 0   -> BT  L47
  b9 3d0b <ptr>  · 33 <ptr>  00 · 1f · 39 470b  buffer == 0  -> BT  L47
  b9 3e0b <uint> · 33 <uint> 00 · 1f · 38 460b  sizeInWords == 0 -> BF L46
  29 470b · 53 53                               L47:  the || target
    26 220b · bd <call> · 4c · 33 <int> 16 · 32 <int> 4b     *_errno() = 0x16
    26 3c0b · bd <call> · 4c · 4b                            _invalid_parameter_noinfo()
    33 <int> ff · 41 <int> · 3a 440b                         return -1  -> L44
  54 05 · 54 04 · 29 460b · 54 03               L46:  the fallthrough

  26 450b · 26 330b · bd <call>                 result = _vswprintf_helper(
    b9 420b <va_list>  55 …                       arglist,        <- REVERSE
    b9 410b <void*>    55 …                       locale             source
    b9 400b <wchar*>   55 …                       format             order
    b9 3f0b <uint>     55 …                       count
    b9 3e0b <uint>     55 …                       sizeInWords
    b9 3d0b <ptr>      55 …                       buffer
    26 3a0b · 2c <fnptr> 00 · 55 <fnptr>          _woutput_s_l  <- DESIGNATOR,
  4c · 32 <int> 4b                                 not B9: an address, and the
                                                   `2c` is the fn -> fnptr decay

  53 · b9 450b <int> · 33 <int> 00 · 22 · 38 480b   result < 0 -> BF L48
  53 53 · b9 3d0b <ptr> · 33 <wchar> 00 · 32 <wchar> 4b   *buffer = 0
  54 05 · 54 04 · 29 480b · 54 03               L48:

  53 · b9 450b <int> · 33 <int> fe · 20 · 38 490b   result != -2 -> BF L49
  53 53 · b9 450b <int> · 41 <int> · 3a 440b        return result -> L44
  54 05 · 54 04 · 29 490b · 54 03               L49:

  26 220b · bd <call> · 4c · 33 <int> 22 · 32 <int> 4b     *_errno() = 0x22
  26 3c0b · bd <call> · 4c · 4b                            _invalid_parameter_noinfo()
  33 <int> ff · 41 <int> · 3a 440b                         return -1  -> L44
  54 02 · 29 440b · 4f 12 47 · 54 01 · 54 00 · 4f 02 20 00 · 4f 01 26 · 4d
```

**The one token that is not a formal read.** Argument 7 of the helper call is
`26 3a0b · 2c <fnptr> 00` — a **designator** (`0x26`) and a conversion, where
every other argument is a `B9` read. That is the whole of the REFHI/REFLO in the
`.text`, and it is the byte the recognizer keys the address-taken function on.

## 3. The 39 words

```text
  0000  7d8802a6  mflr  r12                 FrameLayout{saved_gprs:1}.prologue()
  0004  9181fff8  stw   r12,-8(r1)          — byte for byte, verified §1.3
  0008  fbe1fff0  std   r31,-16(r1)
  000c  9421ffa0  stwu  r1,-96(r1)
$M(n):
  0010  7c7f1b78  mr    r31,r3         (1)  PARK buffer: it is read at 0x5c,
                                            after two calls have clobbered r3
  0014  7d094378  mr    r9,r8          (2)  arglist HOISTED to its final
                                            argument register before the rotate
  0018  2b050000  cmplwi cr6,r5,0      (3)  count == 0        ┐ ONE target for
  001c  419a0058  bt    26,+0x58 ->74       (EQ)              │ all three: the
  0020  2b030000  cmplwi cr6,r3,0      (4)  buffer == 0       │ `||` chain is
  0024  419a0050  bt    26,+0x50 ->74       (EQ)              │ THREE branches,
  0028  2b040000  cmplwi cr6,r4,0      (5)  sizeInWords == 0  │ not a computed
  002c  419a0048  bt    26,+0x48 ->74       (EQ)              ┘ boolean
  0030  7ce83b78  mr    r8,r7          (6)  ┐ the 5-deep ROTATE, descending so
  0034  7cc73378  mr    r7,r6          (7)  │ no source is clobbered before it
  0038  3d600000  lis   r11,0          (8)  │ is read — and the REFHI is
                                            │ INTERLEAVED into it, at word 14
  003c  7ca62b78  mr    r6,r5          (9)  │ of the body
  0040  7c852378  mr    r5,r4         (10)  │
  0044  7c641b78  mr    r4,r3         (11)  ┘
  0048  386b0000  addi  r3,r11,0      (12)  the REFLO — r3 gets &_woutput_s_l
  004c  4bffffb5  bl    _vswprintf_helper   REL24
  0050  2c030000  cmpwi cr0,r3,0      (13)  result < 0, on cr0 — NOT cr6, and
  0054  4080000c  bf    0,+0x0c ->60        the two compares below use cr6
  0058  39600000  li    r11,0
  005c  b17f0000  sth   r11,0(r31)    (14)  *buffer = 0 — a HALFWORD, wchar_t
  0060  2f03fffe  cmpwi cr6,r3,-2     (15)  result != -2
  0064  409a0024  bf    26,+0x24 ->88       (EQ) -> straight to the epilogue,
                                            with `result` already in r3
  0068  4bffff99  bl    _errno              REL24        ┐ the ERANGE arm
  006c  39600022  li    r11,34        (16)  0x22         │
  0070  4800000c  b     +0x0c ->7c    (17)  the TAIL     ┘
  0074  4bffff8d  bl    _errno              REL24        ┐ the EINVAL arm — and
  0078  39600016  li    r11,22        (18)  0x16         ┘ the `||` target
  007c  91630000  stw   r11,0(r3)     (19)  ┐
  0080  4bffff81  bl    _invalid_parameter_noinfo  REL24 │ the MERGED TAIL both
  0084  3860ffff  li    r3,-1         (20)  ┘ arms share
  0088  38210060  addi  r1,r1,96            FrameLayout.epilogue()
  008c  8181fff8  lwz   r12,-8(r1)
  0090  7d8803a6  mtlr  r12
  0094  ebe1fff0  ld    r31,-16(r1)
  0098  4e800020  blr
$M(n+1):
```

## 4. The five things about them

1. **The `||` chain is three branches to one block, not a computed boolean.**
   Every guard is `cmplwi cr6,rX,0 ; bt EQ`, and all three name `0x74`. A lowering
   that materialised the disjunction would emit `or` and one branch: the right
   program, and every displacement after word 7 wrong.
2. **c2 SINKS the `||` block to the end of the function.** In the IL it is L47,
   textually *before* the helper call; in `.text` it is at `0x74`, after
   everything. The recognizer therefore cannot assume IL order is block order.
3. **The two error arms are TAIL-MERGED on four words.** `stw r11,0(r3) ; bl
   _invalid ; li r3,-1 ; epilogue` is emitted **once** and reached from both, the
   ERANGE arm via the `b` at `0x70`. The two arms differ only in one `li`
   immediate. This is the counterpart of board **#1400**'s warning pointing the
   other way: here the sharing *is* the class, and a lowering that duplicated the
   tail would be four words long and still link.
4. **The REFHI is interleaved into the rotate, at word 14.** WR1's
   *"the `lis` is the body's FIRST word"* is false here by 13 words, which is
   PREREG §1.5 row 3 and the reason `data_refs_of` has to derive the site.
5. **Two compares, two condition registers.** `result < 0` is read on **cr0**
   (`cmpwi cr0` — `2c030000`) and `result != -2` on **cr6** (`2f03fffe`). Nothing
   in the source distinguishes them; a class that used one CR for both emits the
   right program and the wrong `bf` operand.

**Free immediate fields: 4** — `0x16`, `0x22`, `-2`, `-1`. Every other word is
fixed, which is PREREG D1's registered expectation and the reason the fence has
to live in the READER (board #1706: anything the emitter cannot vary must be
refused by the reader).
