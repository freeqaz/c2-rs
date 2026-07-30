# The per-function optimization word, and which mode the port actually targets

`.ex` carries a **per-function optimization-settings word** immediately after each
`4F 1F` function-start marker, and the port has never read it. Everything below is
from live captures.

The consequence is the headline, so it goes first: **the port's byte-exactness is
a claim about `/Ox`, and the entire real workload is `/O1`.** Those two modes emit
different code for the same source, including for the core MVP class. Seven of
nine sampled fixtures that the port matches byte-exact at `/Ox` produce different
reference bytes at `/O1`.

## 1. The encoding

```
4F 1F 80 <LE32 word>
```

Seven bytes, at the start of every function segment. `split_functions` anchors on
the `4F 1F` and the port then skips straight past the word.

## 2. Observed values

One source (`int f(int a) { return a + 1; }`), varying only the compile flags:

| flags | word |
|---|---|
| `/Ox` | `00a00005` |
| `/O2` | `00a00005` |
| `/O1` | `00200005` |
| `/Od` | `00800005` |
| `/Ot` alone | `00800005` |
| `/Ob0` alone | `00800005` |
| `/Oy-` alone | `00800005` |

And varying only the source, at `/Ox`:

| source | word |
|---|---|
| baseline | `00a00005` |
| `#pragma optimize("", off)` | `00800004` |
| `#pragma optimize("", on)` | `00a00005` |
| `#pragma optimize("s", on)` | `00200005` |
| `#pragma optimize("t", on)` | `00a00005` |

Nothing else moved it: `static`, `__declspec(noinline)`, `__forceinline`,
`__declspec(dllexport)`, `extern "C"`, `void` return, `float` return, parameter
count, and a tail call all leave it at `00a00005`.

### 2.1 A reading of the bits — hypothesis, not established

Two bits move, and the flag semantics line up if `0x00200000` is *optimizations
enabled* and `0x00800000` is *favor speed*:

* `/Ox` and `/O2` set both — optimize, for speed.
* `/O1` sets only `0x00200000` — optimize, for size. `#pragma optimize("s", on)`
  under `/Ox` produces the identical word, which is the cross-check: two very
  different ways of saying "optimize for size" agree on the encoding.
* `/Od` sets only `0x00800000` — not optimizing; the speed/size preference is
  still at its default. `/Ot`, `/Ob0` and `/Oy-` *alone* land here too, correctly:
  none of them implies an `/O` level.

The low nibble is `5` everywhere except `#pragma optimize("", off)`, which gives
`4`. Not explained. Treat the whole word as opaque and compare it whole.

## 3. `/Ox` and `/O1` emit different code

Reference objs for the same fixture, differing only in the `/O` flag:

| fixture | `/Ox` `.text` | `/O1` `.text` | |
|---|---|---|---|
| `w5_chain` | 68 B | 68 B | differs |
| `w5_tree2` | 64 B | 64 B | identical |
| `w5_tree3` | 112 B | 112 B | differs |
| `il_accum4` | 144 B | 136 B | differs |
| `il_reassoc` | 224 B | 176 B | differs |
| `w6_rel_k` | 464 B | 428 B | differs |
| `w13b_fconst` | 16 B | 16 B | identical |
| `il_call_perm` | 108 B | 96 B | differs |
| `il_deep_chain` | 76 B | 72 B | differs |

Three mechanisms are visible so far.

### 3.1 The descending-register rule does not exist at `/O1`

This is the big one, because it is the rule the port spent the most effort on.
`w5_chain`'s `c4_mul` (`a * b * c * d`):

```
/Ox   mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6      descending r11, r10
/O1   mullw r11,r3,r4 ; mullw r11,r11,r5 ; mullw r3,r11,r6      r11 reused
```

and `c4_sub` (`a - b - c - d`), which has no addition and is therefore the case
`il_accum4.cpp` documents as *always* descending:

```
/Ox   subf r11,r4,r3 ; subf r10,r5,r11 ; subf r3,r6,r10
/O1   subf r11,r5,r11 … r11 throughout
```

So `il_accum4.cpp`'s rule — "c2 decides accumulator-versus-descending once for
the whole chain; a chain with no addition gives each intermediate its own
descending register" — is a **`/Ox` rule**. At `/O1` there is no descending case
at all: r11 is reused unconditionally. Getting that rule right at `/Ox` cost 270
mis-emits found by a generated sweep. None of that work transfers.

### 3.2 Strength reduction is `/Ox`-only

```
a * 9   /Ox   rlwinm r11,r3,3,… ; add r3,r3,r11     (a<<3) + a
        /O1   mulli  r3,r3,9                        one instruction
```

Favor-size keeps the multiply. This is the same class of behaviour as the `a + a`
→ `slwi` folding already documented as c2's (not c1xx's) — but it is conditional
on the mode, which the existing notes do not say.

### 3.3 `/Ox` pads between functions, `/O1` does not

`il_reassoc` at `/Ox` has a `00000000` filler after each odd-length function,
aligning the next to 8 bytes; at `/O1` the functions are packed. That alone
accounts for most of the 224 → 176 size drop.

## 4. What this means for the roadmap

The census and the gap scan run on the real workload, whose flags are
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I…` — **`/O1`**. Every fixture is
captured with the default `/Ox /GS- /c`. So the two halves of the project have
been measuring different targets:

* the **numerator** (fixtures, sweeps, the byte-exactness claim) is `/Ox`;
* the **denominator** (110,277 in-class functions of 2,462,571; the blocker
  histogram; the 878-TU scan) is `/O1`.

An in-class function counted by the census is not a function the port can emit —
it is a function the port can *decode*. If it were emitted today it would be
emitted with `/Ox` register allocation against an `/O1` reference.

This has been invisible because no non-trivial real TU ever reached codegen. Of
the six TUs the last scan reported as `match`, **five have `fn_total = 0`** — they
are empty modules, the four-section shell with no `.text`, where the mode cannot
matter. The sixth (`src/system/utl/Spew.cpp`, two functions) is a real match, and
its two bodies are shapes where the modes agree, like `w5_tree2` above. So the
`match` column has never yet exercised mode-dependent codegen.

`codegen-gap` being 0 while everything else is `vocab-gap` is the same fact from
the other side: the decode gate refuses first, so the codegen has not yet been
asked a question it would answer wrongly.

### 4.1 Order of work

1. **Gate on the word.** Refuse any function whose optimization word is not the
   value the port was verified against. Cheap, fails closed, and covers every
   mode variation including the ones not enumerated here. Until this lands, the
   port is one decode widening away from a wrong-bytes emit on real input.
2. **Re-target to `/O1`.** This is the mode parity on the workload requires. It
   looks *simpler* than `/Ox`, not harder — no descending-register rule, no
   inter-function padding, no strength reduction to reproduce — so this is
   plausibly a net reduction in the codegen the port needs. It is not a
   translation of the existing `/Ox` codegen; it is a second target, and the
   `/Ox` work stays valid for `/Ox`.
3. **Re-run the sweeps under `/O1`.** `scripts/expr_sweep.sh` compiles with the
   default `/Ox`, so its 2589 green cases say nothing about `/O1`. The sweep
   needs a mode parameter, and the `/O1` lane should be the one that gates.

## 5. How this was found

Not by a fixture, and not by the census. `#pragma optimize("", off)` was one entry
in a batch of probes aimed at the *obj shell* — sections and symbols, the axis the
fixture corpus had almost no coverage of. It came back as a mismatch at obj offset
8, which looked like the `.drectve` class (`il_drectve_pragma.cpp`); it was not.
Diffing that TU's IL against the same source without the pragma isolated four
changed bytes, and two of them were the word documented above.

The generalizable part: the port skips bytes it does not model, and a skipped byte
is indistinguishable from a byte that is always the same. Every fixed-width field
the port steps over is a candidate for this — the same shape as the source-line
marker that turned out to carry a varint payload, and the aggregate TYPE that
`read_type` still mis-reads (`IL_TYPE_TAGS.md` §1).
