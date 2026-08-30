# PREREG — lane `w-sizetest`, 2026-08-29

**Kind:** characterization. **Outcome will be exactly one of** `built` / `FAILED`.
**Predicted reach: 0. Census: +0. Required byte delta: 0.** This lane writes no
`crates/` code and changes no compiled file.

Board rows reserved to this lane: **#3870**–**#3876**. Base commit `1d52f8902`.
Image: `compilers/X360/16.00.11886.00/c2.dll`,
sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

**Frozen before the image was opened.** The only material read before this file
was committed is prose already in the tree: `CLAUDE.md`,
`docs/WAVE21_BRIEF_2026-08-29.md`, `docs/whitebox/WB_INSTRCOUNT_FINDINGS.md`,
`docs/BOARD.md`'s `#3830` row and the two wave ledgers,
`docs/WHITEBOX_LEVERAGE_2026-08-21.md`, `docs/whitebox/C2_MAP_METHOD.md` §0–§4,
`work/w-instrcount/PREREG.md`, and **`docs/whitebox/ref/P_INLINE.md` §2, §5 and
§6.5** (read-only; this lane may not edit that file). No disassembler has been
run, and no file under `~/ghidra-projects/export/` has been opened.

**This last item is load-bearing and is why the prereg says it out loud.**
§6.5 is read *before* the predictions below are written, and it materially
changes them: it appears to already answer the brief's primary question. The
predictions are registered against that reading, not against the brief's.

---

## 1. The question, as the brief states it

`#3830` (`WB_INSTRCOUNT_FINDINGS` §2.5): at `0x10b5fc90` the `jl` is not accept
and over-ceiling is not refuse. Both paths meet `0x10b5fcb9`, which needs
`DAT_10c2e2fc != 0` or `ATTR & 0x2080`; the over-ceiling path arrives via

```
10b5fc92:  8b 46 4c    mov   eax,DWORD PTR [esi+0x4c]
10b5fc95:  85 c7       test  edi,eax
10b5fc97:  75 20       jne   0x10b5fcb9
```

and `#3830` calls `edi` **"a caller-supplied `ATTR` mask, i.e. a decision point
c2 itself exposes as a parameter"**, *"one of `FUN_10b5fb5f`'s five
parameters"*, **"named nowhere in this repo"**. The brief (`WAVE21` §2 L4)
carries that forward verbatim and commissions this lane to name it.

## 2. The tension this lane is actually walking into — registered before any read

**`P_INLINE.md` §6.5 already names it, and names it as the opposite kind of
thing.** Lane `w-inlmetric`, 2026-08-27, board `#3717`–`#3722`, from a full
377-byte read of `FUN_10b5fb5f` whose listing is in the tree at
`work/w-inlmetric/FUN_10b5fb5f.asm`:

```
10b5fc31:  bf 00 20 00 00     mov    edi,0x2000        <- THE MASK, materialised
10b5fc36:  85 df              test   edi,ebx           <- ebx = [sym+0x4c]
...
10b5fc95:  85 c7              test   edi,eax           <- edi still 0x2000
```

with the explicit argument *"`edi` is callee-saved and nothing between the two
writes it"*, under the heading **"§2.1's `0x2000` `__forceinline` mask DOES
carry. It is `edi`."**

So `#3830` and `P_INLINE` §6.5 **cannot both be right**, and the brief inherits
`#3830`'s side. Exactly one of these is true:

* **(A)** `edi` is `0x2000`, set inside the function at `0x10b5fc31`, on every
  path that reaches `0x10b5fc95`. Then the mask is `__forceinline`, it is **not**
  caller-supplied, it is **not** a decision point c2 exposes as a parameter, and
  it **is** named in this repo — `#3830`, the brief and `WB_INSTRCOUNT` §2.5/§8
  are all wrong on this point.
* **(B)** There is a path from entry to `0x10b5fc95` that does **not** pass
  `0x10b5fc31`, so on that path `edi` is whatever the caller left in it. Then
  `#3830` is right about *that path*, §6.5's *"nothing between the two writes
  it"* is an argument about the wrong interval (it needs domination, not
  absence of an intervening write), and both pages are half right.
* **(C)** Something else entirely — e.g. `edi` is reloaded from memory on a
  third path, or the byte at `0x10b5fc31` is not what §6.5 prints.

**This lane's first act after the prereg is a control-flow read that decides
between (A), (B) and (C), by dominance rather than by proximity.** Not a text
grep, not a decompiler signature: every branch target in `0x10b5fb5f`–
`0x10b5fcd7` enumerated, and the predecessors of `0x10b5fc92` computed.

## 3. Registered predictions

Each is falsifiable and I report the verdict of each, including misses.

**P1 — (A) is the answer.** `mov edi,0x2000` at `0x10b5fc31` **dominates**
`0x10b5fc95` inside `FUN_10b5fb5f`, no other instruction in the function writes
`edi`, and therefore the mask tested at `0x10b5fc95` is the constant `0x2000` =
`__forceinline`. **`#3830`'s "caller-supplied mask parameter" is FALSE, and the
brief's primary commission dissolves into a correction.**
*Falsified if:* any predecessor path of `0x10b5fc92` bypasses `0x10b5fc31`, or
any instruction in the body writes `edi` a second time, or `edi` is restored
from the stack between them.

**P2 — Ghidra's "five parameters" is a decompiler artifact, not a convention.**
The image's 32-bit MSVC code uses `__thiscall`/`__fastcall` (`ecx`, `edx`) and
`__stdcall`/`__cdecl` (stack); `edi` is callee-saved in all of them and is not a
parameter register in any. I predict `FUN_10b5fb5f`'s signature in
`functions.tsv`/`decomp_all.c` shows a custom/unknown convention with `edi`
listed because Ghidra saw a read on a path it could not prove unreachable —
i.e. the same class of artifact as `#3505`'s census.
*Falsified if:* any of `FUN_10b5fb5f`'s call sites writes `edi` before the
`call`, which would make it a genuine (if unconventional) inbound register.
**I will check the call sites regardless of P1's verdict**, because that check
is what turns P2 from an assertion into evidence — an unwritten `edi` at every
call site is the positive result even if P1 already made it moot.

**P3 — `DAT_10c2e2fc` is an option/mode global, not a computed one.** I predict
≤ 3 writers, all reachable from c2's switch/option handling rather than from
per-function analysis, and that when it is non-zero `0x10b5fcb9` returns 1
unconditionally — i.e. it is a *"candidacy is disabled as a filter"* master
switch. I further predict it is **zero on this project's workload profile**, so
the second gate is decided by `ATTR & 0x2080` alone in every artifact this repo
has ever measured.
*Falsified if:* it has a per-function writer, or it is set by any flag set this
repo compiles with.

**P4 — the second gate is `__forceinline`-or-bit-7, and bit 7 is the open
half.** `0x2080` = `0x2000 | 0x80`. §2.5 already killed bit 7 as the explanation
of §2.1b's pair (`ATTR = 0x68` in both). I predict bit 7 of the `.gl` `ATTR`
word has **no other reader in the image** — i.e. `0x10b5fcc1` is where it is
consumed and the front end is what sets it — and that this lane can say what it
means only if `w-glattrs`' captured corpus contains a record with it set.
*Falsified if:* another reader exists, or the corpus has one and it does not
behave as predicted.

**P5 — the brackets are consistent with the size test NOT being the moving
predicate at either ladder.** §6.5 established there is **no linkage arm inside
the candidacy function**, so nothing in `FUN_10b5fb5f` can produce a
STATIC/EXTERNAL split. I therefore predict:

* no value of `DAT_10c46318` reproduces both brackets **and no value has to** —
  the brackets are measured *verdict* boundaries and the verdict is a
  conjunction, so attributing them to the ceiling was never licensed;
* `DAT_10c46318` is **run-time written** (like the POGO tables of §5, which sit
  above the image's raw `.data` at `0x10c3cc00`), so a value read out of the
  image's static bytes is not the value the workload takes;
* the arithmetic that *is* consistent with both brackets, if any exists, is a
  **per-linkage adjustment applied downstream of the ceiling**, and I will state
  what the two windows jointly exclude rather than proposing a number.

**I will not name a ceiling value.** `#3732` refuted 128 with 8 counterexamples
each way and adopting 128 is a standing prohibition; adopting 256, 261, 267 or
any other fitted value in its place is the same error with a fresh number.

**P6 — §2.1a/§2.1b.** I predict I can restate the correction more sharply than
`w-instrcount` did, because if P1 holds then §2.1's *"the test at `0x10b5fc92`
is against a mask held in `edi` rather than an immediate"* is **an encoding
observation and not a semantic one**, and three pages have been building
inferences on it as if it were semantic.

## 4. Read before probe — the price registered now

Everything above is a **read**. The flat export is already on disk
(`~/ghidra-projects/export/c2/`: `objdump_intel.asm`, `decomp_all.c`,
`xrefs.tsv`, `functions.tsv`, `data.tsv`, `calls.tsv`), so the marginal cost of
each question here is a grep and a CFG walk over ≤ 400 bytes of code.

The probe that would answer the same questions is a flag-sweep grid over
`/Ob`, `/O`, `/Gy` and linkage, at enough resolution to separate three
predicates that all feed one boolean — which is precisely the shape that cost
this project `#3732`'s two-sided refutation and `w-sizebracket`'s undecidable
`/Ox` bracket. **The read is cheaper by more than an order of magnitude and it
is preferred.** If I run any cell at all it is a *confirmation* probe against a
prediction written above, per `WHITEBOX_LEVERAGE_2026-08-21.md` §1.

Specifically, I price and **decline** the following grid up front: a
linkage × size ladder at 8+ cells per arm to re-take §6's brackets. It would
re-measure numbers the tree already has and could not attribute them, because
attribution needs the dominance fact, which is a read.

## 5. What I refuse to conclude

* **I will not name a numeric inline ceiling** — not 128 (`#3732`, prohibited),
  not a value fitted to `[261,267]` or `[93,99]`, not any single value that
  "fits both". If the brackets exclude everything I can name, the finding is the
  exclusion.
* **I will not claim the size test decides candidacy** in either direction.
  `#3830`'s structural half — neither necessary nor sufficient — is right and
  this lane does not weaken it. Only its *mask* attribution is in question.
* **I will not conclude "the mask is caller-supplied" from a decompiler
  signature.** A Ghidra parameter list is a hypothesis about a convention. Only
  a call site that writes the register is evidence.
* **I will not assert a universal negative** (no other writer / no other reader)
  without publishing the query set and the classes it cannot see. `#3505` is six
  for six, and its sharpest instance — 60 refs / 0 writes, correct, because the
  write went through `rep movsd` and `EDI` — is about **this very register**.
* **If the honest answer is "the mask is `0x2000` and the size test still cannot
  be decided from an obj", I say exactly that** and the lane is complete.
* **I will not touch** `crates/**`, **`docs/whitebox/ref/P_INLINE.md`**,
  `work/w-inlmetric/**`, `work/w-emitprice/**`, `work/w-budget/**`,
  `scripts/gate.sh`, `docs/STATUS.md`, `docs/rungs/INDEX.md`, or any board row
  outside **#3870**–**#3876**.

## 6. Gate evidence owed

`scripts/gate.sh --jobs 16 --require-graded` (unqualified `GATE: PASS`, read
from the `GATE:` verdict LINE and never the exit code) and
`cargo test --workspace --release --no-fail-fast` with **both the target count
and the pass count**. Byte delta must be zero and is zero by construction.
`rung_index_is_generated_and_current` is expected RED at this lane's tip
(`INDEX.md` is regenerated at merge) and is not this lane's to fix.
