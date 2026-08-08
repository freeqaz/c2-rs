# `?NextHashPrime@@YAHH@Z` — the IL body, decoded token by token

The thing the next lane on `Primes.cpp` needs first, and which no document
carries. Decoded by hand from the capture at `5d101061`, workload flags
(`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc …`), against
`docs/IL_STMT_GRAMMAR.md`'s opcode tables and two probes of this lane's own
(§3). The IL itself is never committed; this is the reading.

**294 bytes, ~60 tokens, ONE linear sequence.** No nesting a pattern matcher
cannot walk, no value merge at a join — so `PREREG.md`'s decline clause **D2**
(*"if the reader production needs a general basic-block IR, decline"*) **does not
fire**, and that is a measurement on a built decode rather than a guess.

## 1. The source

```c
int NextHashPrime(int i) {
    static int primes[62] = { 0x1D, 0x25, …, 0 };
    for (int i2 = 0; primes[i2] != 0; i2++)
        if (primes[i2] >= i)
            return primes[i2];
    return i;
}
```

Tokens: `e6 09` = the formal `i` · `ea 09` = `primes` · `eb 09` = `i2` ·
`ec 09` `ed 09` `ee 09` `ef 09` `e8 09` = the five labels.

## 2. The stream

```text
  4c 4f 11                     the body marker
  53                           the function scope opens
  4f 01 0e                     line

  26 eb 09                     designator   i2
  33 86 41 74 00               literal int 0
  32 86 41 74 4b               store  int              ---- i2 = 0

  3a ec 09                     JUMP  Ltest             ---- THE ROTATION: over
  29 ed 09                     LABEL Lincr                  the increment, into
  26 eb 09                     designator   i2              the bottom test
  33 86 41 74 01               literal int 1
  35 86 41 74 4b               compound +=  int        ---- i2++
  29 ec 09                     LABEL Ltest

  26 ea 09                     designator   primes     ---- primes[i2], the
  b9 eb 09 86 41 74            load int     i2              SUBSCRIPT IDIOM,
  33 86 41 12 04               literal      4                three tokens plus
  04                           MULTIPLY                      a `28 00 00`
  28 00 00                     subscript (token 0)
  30 86 41 74                  indirection  int
  33 86 41 74 00               literal int 0
  20                           cmp-ne
  38 ee 09                     BRFALSE Lexit           ---- primes[i2] != 0

  53  4f 01 0f  53             the loop body scope opens
  26 ea 09 … 28 00 00 30 …     primes[i2]              ---- the SAME idiom again
  b9 e6 09 86 41 74            load int     i (formal)
  23                           cmp-ge
  38 ef 09                     BRFALSE Lcont           ---- primes[i2] >= i

  53  4f 01 10
  26 ea 09 … 28 00 00 30 …     primes[i2]              ---- the idiom a THIRD time
  41 86 41 74                  RETURN VALUE int
  3a e8 09                     JUMP Lret

  4f 01 11  54 06
  29 ef 09                     LABEL Lcont
  54 05  54 04
  3a ed 09                     JUMP Lincr              ---- THE BACK EDGE

  29 ee 09                     LABEL Lexit
  4f 01 13
  b9 e6 09 86 41 74            load int     i
  41 86 41 74                  RETURN VALUE int
  3a e8 09                     JUMP Lret

  54 03  54 02  4f 01 14
  29 e8 09                     LABEL Lret
  4f 12 47  54 01  54 00  4f 02 20 00  4f 01 15
  4d                           end of stream
```

## 3. Two opcodes this lane pinned, with the probes that pin them

Neither is in `body::expr_opcode_name`, so both were a hex bucket before.
`work/w-cfg2/op/*.cpp`, two lines each, captured at the same flags.

* **`0x04` is `*` (multiply).** `int f(int a,int b){ return a*b; }` captures
  `b9 <a> 86 41 74 · b9 <b> 86 41 74 · 04 · 41 86 41 74 · 3a <ret>` — the byte
  sits exactly where a binary operator sits and the body has no other operation.
* **The SUBSCRIPT IDIOM is `26 <base> · b9 <index> <TYPE> · 33 <86 41 12>
  <scale> · 04 · 28 00 00 · 30 <TYPE>`.**
  `int g[8]; int f(int i){ return g[i]; }` captures it byte-for-byte identically
  to all three of `Primes`' occurrences, including the `86 41 12` literal type
  and the `28 00 00` (a `varU` of 0 — board **#1593**'s reading of opcode `0x28`
  as class `02`, confirmed here from the other side).
  `41 <TYPE>` is the **return-value** op and is always followed by `3a <Lret>`.

`idx.cpp` is therefore a two-line reproducer for the one sub-expression
`Primes.cpp` uses three times, and a next lane can grade the subscript
production without touching the loop at all.

## 4. What this says about the recognizer

The production is `ptr_walk_loop`'s in structure — same `3A Ltest · 29 Lincr ·
<incr> · 29 Ltest · <test>` rotation, same `26 <local> · 33 <lit> · 32 <TYPE>
4B` opening statement — and differs in what the loop *carries*: a subscript of a
**defined static array** instead of a pointer deref, an early `41`/`3a` return
out of the body instead of an accumulate, and a second `41`/`3a` after the exit
label. `crates/c2-il/src/func/body/shapes/ptr_walk_loop.rs` is the file to copy,
**and its accumulator clause is the trap** — board **#1636** records
`ptr_walk_loop`'s `locals`/`ptr_locals` conjunction being copied verbatim into a
class whose accumulator is a different `.sy` kind, where it is vacuously false
and refuses every instance by construction. `i2` here is a plain `int` local, so
`locals` is the right list and `ptr_locals` is the wrong one.
