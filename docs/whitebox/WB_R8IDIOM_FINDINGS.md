# WB_R8IDIOM — `mr r8,r8` is `emit 0x7d084378`, minted for pseudo-opcode `0x2e4`, and the peephole never sees it

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from. **This lane adopts nothing.**

**Lane:** `w-r8idiom` · characterization · **Fixtures:** none · **Census:** +0 ·
**reach: 0** · **prereg:** [`docs/rungs/_2026-08-24-w-r8idiom-prereg.md`](../rungs/_2026-08-24-w-r8idiom-prereg.md)
· **rung:** [`docs/rungs/2026-08-24-w-r8idiom.md`](../rungs/2026-08-24-w-r8idiom.md)
· **board:** `#3481`–`#3484`
· **image:** `c2.dll` sha256 `c80981c015166eff…a66258`, 1 347 072 B.

**Subject.** `w-tailread`'s top-ranked follow-up
([`2026-08-23-w-tailread.md`](../rungs/2026-08-23-w-tailread.md) § "Found and
not taken" 1 and 6; [`ref/P_OPATTR.md`](ref/P_OPATTR.md) §6): **3,792
`mr r8,r8` self-moves in 1,206 of 120,000 objs**, all naming `r8`,
branch-adjacent, no relocation covering them — *"real, reproducible,
unexplained"* — and the sibling handler `0x10c16d83` (peephole **arm 14**,
`mr`) that lane did not read.

---

## 0. The answer, in five lines

| # | claim | class |
|---|---|---|
| **A** | `mr r8,r8` (word `0x7d084378`) is emitted by **one arm of the final-expansion switch**, `0x10c0e194`, which builds an instruction of opcode **`0x290` = `emit`** carrying the **baked 32-bit literal** `0x7d084378`. The register is a constant in c2's code, not an allocation. | `[R]` |
| **B** | That arm is reached by exactly one opcode: the **pseudo-opcode `0x2e4`**, which is above the machine opcode space (the mnemonic table ends at `0x295`) and has no mnemonic. | `[R]` |
| **C** | The peephole `FUN_10c182b4` **refuses every opcode `>= 0x295`** — twice on the path to its arm table (`0x10c18330`, and `0x10c182df` in its pre-pass), plus a third bound at `0x10c1835a`. **Arm 14 is never reached with a `0x2e4`.** | `[R]` |
| **D** | **Arm 14 itself deletes unconditionally.** `FUN_10c16d83`, 214 B, read in full: same-register test → two guarded flag-clears → **unconditional** tail-call to the unlink `0x10c16cde`. **Zero guards**, measured, not eyeballed. | `[R]` |
| **E** | On **120,000 objs**: every one of 3,792 self-moves names `r8`; **0 bearing objs lack C++ EH**; they occur in **runs bracketing a call**; and the **run length equals the `__catch$` count on 95.19 %** of the 1,206 bearing objs. | `[O]` |

**C and D together dissolve the contradiction `P_OPATTR.md` §6 recorded.** That
page found an unconditional redundant-move eliminator *and* 3,792 surviving
self-moves, and could only say *"what is refuted is the LICENCE, not the code
read"*. Both halves were right. **The surviving self-move was never a `mr`
inside c2** — it is an `emit` of a literal word, and the eliminator dispatches
on opcode. There is nothing left to reconcile.

**What is NOT settled: what `0x2e4` IS.** §6 gives the evidence that bounds it
and refuses to name it.

---

## 1. The obj side — the population, with a denominator and a control

Tool: [`scripts/probe_r8idiom.py`](scripts/probe_r8idiom.py). It is the sibling
of `probe_selfmove.py`, which *found* these; this one characterises them.

### 1.1 `w-tailread`'s numbers reproduce exactly, and that is a trap `[O]`

`probe_selfmove.py --limit 120000`, re-run on the pinned image's corpus today:
**3,792 self-moves in 1,206 of 120,000 objs, 176,969 `.text` sections,
1,726,709 words, `r8` ×3,792, 0 covered by a relocation.** Identical to
`P_OPATTR.md` §6 in every figure, and the 6,000-obj row (29,785 non-self, 0
self) reproduces too.

**But the disjoint slice does not.** `--skip 120000 --limit 120000` — objs
120,001…240,000 of the same cache, a set with no member in common — reports
**13,307 self-moves in 2,044 of 120,000 objs**: **3.5× the count and 1.7× the
prevalence.** The cache is written by ~80 lanes and `os.walk` order is
whatever the filesystem says, so *"the first 120,000"* is a sampling decision,
not a population.

> **`3,792` is a fact about which fixtures happen to come first in a directory
> walk. It is not a fact about c2.** Quote the invariants below; do not quote
> the count. This is the same defect shape `w-tailread` named in its own
> instrument (`767` was a walk's domain, not a measured set) — one lane later,
> in a number that lane published.

Every **structural** claim in §1.2–§1.5 was re-checked on the disjoint slice and
holds there.

### 1.2 C++ EH is NECESSARY and is not sufficient `[O]`

| slice | bearing objs | **bearing objs with no `__ehfuncinfo$`/`__CxxFrameHandler`** | EH objs carrying none |
|---|---:|---:|---:|
| objs 1…120,000 | 1,206 | **0** | 803 of 2,009 (39.97 %) |
| objs 120,001…240,000 | 2,044 | **0** | 3,964 of 6,008 (65.98 %) |

**0 counterexamples in 3,250 bearing objs across 240,000 objs.** The control is
printed beside it precisely because "necessary" is worth much less without it:
**most EH-bearing objs carry no self-move at all.**

### 1.3 They come in runs, and the runs bracket a call `[O]`

Per-instruction neighbour histograms cannot see this — inside a run of three,
two of the three neighbours are self-moves and the bracketing instruction is
invisible. That blind spot is exactly `P_OPATTR.md` §6's `self|self ×634` row.
`probe_r8idiom.py` measures **maximal runs** instead:

```
`bl` then run                  1068
run then `bl`                   651
not adjacent to a `bl`          540      <- 24 % ; NOT a universal
```

Run lengths are **1, 2, 3, 4 and nothing else** (1,360 / 465 / 234 / 200 runs).
Where a call is bracketed on both sides, the two run lengths are **EQUAL on 581
of 613 (94.78 %)**.

### 1.4 The run length is the number of catch handlers `[O]`

Longest run in an obj vs its `__catch$` symbol count, over the 1,206 bearing
objs:

| (max run, `__catch$`) | objs |
|---|---:|
| (1, 1) | 591 |
| (1, 2) | 58 |
| (2, 2) | 287 |
| (3, 3) | 150 |
| (4, 4) | 120 |

**`max_run == n_catch` on 1,148 of 1,206 (95.19 %)**, and the only exception
class is `(1, 2)`.

### 1.5 What the sources are, and the honest limit `[O]`

**100 % of bearing objs are generator-produced**, and all 1,206 come from a
**single family, `67-eh-try-throw`** — 82 distinct sources out of that family's
148. The 66 that never bear are the family's non-`try` members
(`if(a) throw 1;`, `int z(int a){return a+1;}`); the 82 that bear are exactly
the shapes with **a call inside a `try` block**.

> **This is my registered failure mode (prereg P1.6) and it FIRED.** The
> capture cache is a fixture corpus. Everything in §1 is a statement about
> `c2` **on the shapes this one generated family contains**, and no sentence
> here generalises to C++ at large. What rescues the finding is that §2–§4
> answer the same question by reading the image, where the corpus cannot
> mislead.

### 1.6 A published claim that is wrong `[O]`

`P_OPATTR.md` §6 says *"The unit was compiled `/Ox`, so 'the peephole was
disabled' does not explain them."* The **unit** was; the **population** is not.
Bearing objs span **`/Od`, `/O1`, `/O2`, `/Ox`**, with and without `/EHsc` and
`/GR` — ten distinct flag sets in the top ten alone, with `/Od /GS- /c` and
`/Od /EHsc /GS- /c` among them. The conclusion (`/Ox` rules out a disabled
peephole) survives; the premise as a statement about the 3,792 does not.
`P_OPATTR.md` is amended beside, not rewritten.

---

## 2. c2 calls it `mr r8,r8` itself `[O]`

The `/FAsc` listing seam (`Toolchain::capture_listing_with`; board `#132`) makes
c2 narrate its own output. Driving `cl /Bd /Ox /GS- /c /FAsc` under wibo on the
corpus's own fixture reproduces the captured obj **and** prints:

```
.endprolog
$M2580:
$M2574:
  00024  7d084378   mr           r8,r8
  00028  7d084378   mr           r8,r8
  0002c  7d084378   mr           r8,r8
  00030  48000001   bl           ?h@@YAHXZ
  00034  7d084378   mr           r8,r8
  00038  7d084378   mr           r8,r8
  0003c  7d084378   mr           r8,r8
  00040  48000018   b            $LN8@f
```

— for `int f(int a){ try{ return h(); } catch(int e){return 3;}catch(char e)
{return 7;}catch(long e){return 8;} }`, three catch clauses, three words each
side. The one-catch source in the same family gives one each side.

So the word is **not a decode artifact and not data mistaken for code**: c2's
own disassembler prints it as an instruction, at a place its own listing marks
with EH state labels `$M####`.

*(The obj-side claim that they carry `$M####` labels is weaker than it looks:
1,342 of 3,792 sites have one, 2,940 have no symbol at all. Only the **first**
word of a run is labelled.)*

---

## 3. It is NOT one of c2's nops — the whole family, excluded `[R]`

`dump_movearms.py --nops`. c2's mnemonic table `0x10b1b260` carries **nine**
pseudo-nops. Each one's emitted word is read from its encoder arm (encoder form
**37**, arm `0x10bfa1ad`, 9-entry jump table `0x10bfafe9` covering opcodes
`0x277`…`0x27f`) rather than taken from the base-word table, because every arm
ORs its own register fields in:

| opcode | mnemonic | emits | = |
|---|---|---|---|
| `0x276` | `nop` | `0x60000000` | `ori r0,r0,0` |
| `0x277` | `nopmthigh` | `0x7c631b78` | `or r3,r3,r3` |
| `0x278` | `nopmtmed` | `0x7c421378` | `or r2,r2,r2` |
| `0x279` | `nopmtlow` | `0x7c210b78` | `or r1,r1,r1` |
| `0x27a` | `nopstall` | *dynamic* | `or r28..r31` — see below |
| `0x27b` | `nopalign` | `0x7c000378` | `or r0,r0,r0` (arm is the join; base word verbatim) |
| `0x27c` | `nopvmxperm` | `0x181b021a` | VMX form |
| `0x27d` | `nopvmxsimp` | `0x11ef7c84` | `vor v15,v15,v15` |
| `0x27e` | `nopcapenter` | `0x7dad6b78` | `or r13,r13,r13` |
| `0x27f` | `nopcapexit` | `0x7dce7378` | `or r14,r14,r14` |

**`nopstall`'s register is a table lookup** — arm `0x10bfa1db` reads
`operand[0x18]`, caps it at `0xf` (default `0x1f`), and indexes the byte table
**`0x10c37dcc`**, then splices the value into all three register fields
(`x<<5|x`, `<<5|x`, `<<11`). The table is
`28 ×10, 29, 29, 30, 30, 31, 31` — i.e. the Xenon delay nops, `or r28,r28,r28`
for 0–9 requested cycles through `or r31,r31,r31` for 14+.

**`0x10c37dcc` is a new address for this record.** So is the decode of
`0x277`–`0x27f`.

> **None of the nine is `or r8,r8,r8`.** A `mr r8,r8` in an obj is therefore
> not one of c2's nop pseudo-ops. That is a whole family excluded by reading,
> and it is what made the next step worth taking.

---

## 4. Where it actually comes from `[R]`

`dump_movearms.py --word 0x7d084378` finds the literal in `.text` **four
times**, and each is a real `push` immediate (alignment and owning function
both checked — `dump_tailclass.py` was bitten by counting table bytes
disassembled as code):

| VA | owner | TU |
|---|---|---|
| `0x10bf12d9` | `0x10bf1233` | `cgintrin.c` |
| `0x10bf1a62` | `0x10bf19da` | `cgintrin.c` |
| `0x10bf80d8` | `0x10bf7c59` | `cgintrin.c` |
| **`0x10c0e1a1`** | **`0x10c0d57e`** | **`lower.c` — the final-expansion switch** |

The last one is the one that matters. `FUN_10c0d57e` is the switch
`dump_expansion.py` reads; its opcode tree sends **`0x2e4`, and only `0x2e4`**,
to arm **`0x10c0e194`**.

**That is checked at the byte level and not taken from the tool.** `w-tailread`
showed that this tool's *domain* claims can be artifacts of its walk, so the
edge is verified directly: **exactly one branch in the entire image targets
`0x10c0e194`**, and the subtract chain that guards it is unambiguous —

```
10c0e146  mov ecx,eax                  ; ecx = the opcode
10c0e148  sub ecx,0x2ba / je 0x10c0e1cb        -> 0x2ba
10c0e150  sub ecx,0x27  / je 0x10c0e1bf        -> 0x2ba+0x27 = 0x2e1
10c0e155  sub ecx,0x3   / je 0x10c0e194        -> 0x2e1+0x03 = 0x2e4   <== HERE
10c0e15a  dec ecx       / jne 0x10c0e30b       -> 0x2e5 falls through
```

The arm is eleven instructions:

```
0x10c0e194  push esi
0x10c0e195  push 0x10bd3824          ; the MINT primitive, as a callback
0x10c0e19a  push 0x0
0x10c0e19c  mov  esi,0x2004
0x10c0e1a1  push 0x7d084378          ; <== THE WORD
0x10c0e1a6  mov  ecx,esi
0x10c0e1a8  call 0x10bd575d          ; build the literal operand node
0x10c0e1ad  push eax
0x10c0e1ae  mov  edx,esi
0x10c0e1b0  mov  ecx,0x290           ; opcode 0x290 -- the table calls it `emit`
0x10c0e1b5  call 0x10bd726d          ; build the instruction
```

**`0x290` is `emit`** in c2's own mnemonic table (form 18, base word `0`) — the
raw-word emitter. So the final instruction is not a move at all: it is *"place
this 32-bit constant"*.

**`P_EXPAND.md` §3 already scored this arm `1..1` and already named its opcode
`0x2e4`; `dump_tailclass.py:496` already used it as a CONTROL that mints.**
What no document said is **which word**. That is the whole gap, and it is one
line long.

### 4.1 The other three sites are a SECOND idiom, and it is not this one `[R]`

Read, because "four sites, one matters" is the kind of sentence that hides a
finding. All three `cgintrin.c` sites build the same `emit` (`0x290`) of the
same literal, and **two of them precede it with two `nop`s**:

```
0x10bf12b4  (fn 0x10bf1233)   mov ecx,0x276 / call 0x10bd59aa   <- nop
                              mov ecx,0x276 / call 0x10bd59aa   <- nop
                              push 0x7d084378 … mov ecx,0x290    <- mr r8,r8
0x10bf1a43  (fn 0x10bf19da)   the same, then `call 0x10bd5516`   <- and DELETES
                                                                    the original
0x10bf80cc  (fn 0x10bf7c59)   push 0x10bd3815  (a DIFFERENT callback from the
                              0x10bd3824 the other three pass) … mr r8,r8
```

So c2 has a **three-word `nop / nop / mr r8,r8` sequence** in its intrinsics
code generator, distinct from the EH-side single word, and at `0x10bf1a43` it
*replaces* an instruction with it. `0x10bf1a71` is worth one line on its own:
the opcode is computed as `lea ecx,[ebp+0x1a]` where `ebp` is still `0x276`
(`nop`) — i.e. **`0x276 + 0x1a = 0x290`**, `emit`, reached by arithmetic on the
`nop` opcode rather than by a literal.

**What intrinsic this is, is not read**, and the corpus cannot help: no fixture
in it uses one. Ranked as follow-up 3 in the rung.

**Who mints `0x2e4`** (`mov ecx,imm32` sites only — a lower bound, since other
registers carry the opcode elsewhere):

| function | TU | sites |
|---|---|---:|
| **`0x10be3e4c`** | **`ehexcept.c`** | **4** |
| `0x10be4f28` | `except.c` | 1 |
| `0x10b372ea`, `0x10b39937` | `fg.c` | 1 each |
| `0x10b6e99b` | `inline.c` | 1 |
| `0x10b9f04e`, `0x10b9fb3f` | `p2symtab.c` | 1 each |
| `0x10c0d57e` | `lower.c` | 1 |

`FUN_10be3e4c` (`ehexcept.c`) mints them **inside a list walk**
(`edi = edi->next` at `0x10be3fe9`, looping back to `0x10be3fcd`) — one
`0x2e4` per element — which is the shape §1.4's *"run length == number of
catch handlers"* has on the other side of the compiler.

---

## 5. Arm 14, read in full — and it has no guard `[R]`

`FUN_10c16d83`, **214 B**, `mdmisc.c`, `0x10c16d83`…`0x10c16e58`. Reached by
exactly one `call` in the image, from the 12-byte thunk `0x10c18373`
(`mov ecx,esi / call 0x10c16d83 / jmp 0x10c18448`).

```
src = instr[+0x28] ; dst = instr[+0x2c]
if (dst->[0x1c] == src->[0x1c])                       <- SAME register descriptor
      if (dst->[8] == 1 && dst->[0x18]->[7] & 0x40)  dst->[0x18]->[7] &= ~0x40
      if (src->[8] == 1 && src->[0x18]->[7] & 0x40)  src->[0x18]->[7] &= ~0x40
      tail-call 0x10c16cde                            <- UNCONDITIONAL unlink
else  0x10c16a46 -> {class != 0xf, opcode not in {0x12f,0x130,0x131,0x132},
                     def->[0x2c]->[0x1c] == the same register}
      -> 0x10c16bda / 0x10c16c7d / 0x10c16ba5 vetoes -> 0x10bd7b09  (copy propagation)
```

**The measurement, not an impression.** `--arms` counts the conditional
branches strictly between the same-register compare and the tail-call, and
splits them into those that can **skip past** the unlink (a guard that could
refuse the delete) and those that rejoin before it:

| arm | opcode | handler | bytes | **guards** | inner | verdict |
|---|---|---|---:|---:|---:|---|
| 6 | `fmr` | `0x10c16fbd` | 191 | **0** | 4 | unconditional |
| **14** | **`mr`** | **`0x10c16d83`** | **214** | **0** | 4 | **unconditional** |
| 15 | `mr.` | `0x10c1707c` | 202 | — | — | **no path to the unlink at all** |
| 16 | `vmr` | `0x10c16e59` | 356 | **0** | 4 | unconditional |

Three new facts here beyond the brief:

* **Arm 15 (`mr.`) cannot delete.** No path from `FUN_10c1707c` reaches
  `0x10c16cde`. `mr.` writes `CR0`, so a same-register `mr.` is not inert —
  the eliminator correctly does not exist for it. (The corpus cannot corroborate
  this: it contains 150 `mr.` and **zero** self-`mr.`, so there is nothing to
  see either way. Recorded as `[R]` with the gap stated.)
* **The four class-1 thunks are consecutive**, 12 bytes apart:
  `0x10c18373` (`mr`), `0x10c1837f` (`vmr`), `0x10c1838b` (`fmr`),
  `0x10c18397` (`mr.`). `P_OPATTR.md` §6.1 gives handlers, not thunks; two of
  the four thunk addresses this lane first guessed were wrong and `--arms`
  said so.
* **The dispatch is bounded three times.** `FUN_10c182b4`:
  `cmp eax,0x295 / jae skip` at `0x10c182df` (pre-pass) and `0x10c18330`
  (main), then `dec eax / cmp eax,0x292 / ja skip` at `0x10c1835a` before
  `movzx eax,BYTE PTR [eax+0x10c184a8]` and
  `jmp DWORD PTR [eax*4+0x10c18460]`.

**`0x2e4 >= 0x295`.** The peephole discards it at the first bound. Arm 14 is
not guarded, not weakened, and not wrong — **it is not reached.**

---

## 6. What is NOT settled, and is deliberately not guessed

**What `0x2e4` IS.** Three things are read and one is not:

* `[R]` it is above the machine opcode space and has **no mnemonic**;
* `[R]` it is recognised **image-wide** — `fg.c`, `factor.c`, `dag.c`,
  `inline.c`, `misc.c`, `globopt.c`, `p2symtab.c`, `pogocg.c`, `ptinl.c`,
  `regasg.c`, `sizeopt.c`, `ssa_seh.c`, `stack.c`, `tuple.c`, `lower.c`,
  `mdmisc.c`, `ehexcept.c`, `except.c` all test for it, always with `je`
  (equality), never as a range bound;
* `[R]` three already-published predicates in this record put it in a
  **branch class** with `0x21` (`bc`) and `0x22` (`bca`):
  `WB_LOOP_FINDINGS.md:149`, `WB_DAGCLIENTS_FINDINGS.md:104,367`,
  `WB_MERGER4_FINDINGS.md:67`;
* **not read:** its name, its operands, and what a pass is *supposed* to do
  with it.

A story that fits everything above — a branch-like marker carrying a CFG edge
to a handler, one per catch clause, expanded to one inert word so the edge has
an address the EH tables can name — **is a story, and it is not in this
record's claims.** `w-tailread` established that "deliberately not guessed at"
is a valid final state; R6's refusal precedent is the same rule. §7 ranks the
read that would settle it.

**Also not settled:** why `r8` in particular rather than `r0` (which `nopalign`
already uses). The register is a **baked literal** `[R]`, so the question is
"why did whoever wrote `lower.c` pick that constant", which the binary cannot
answer.

---

## 7. Addresses this lane adds to the record

| address | what | first recorded |
|---|---|---|
| `0x10c16d83` | peephole **arm 14** handler, `mr`, 214 B, `mdmisc.c` — read in full | here (named, unread, by `P_OPATTR.md` §6.2) |
| `0x10c18373` / `0x10c1837f` / `0x10c1838b` / `0x10c18397` | the four consecutive class-1 thunks (`mr` / `vmr` / `fmr` / `mr.`) | here (only `0x1838b`, `0x18373` were named) |
| `0x10c182df`, `0x10c18330`, `0x10c1835a` | the peephole's **three opcode bounds** (`0x295`, `0x295`, `0x292`) | here |
| `0x10c0e1a1` | the `push 0x7d084378` inside expansion arm `0x10c0e194` | here |
| `0x10c0e1b0` | the `mov ecx,0x290` that makes it an `emit` | here |
| `0x10bf12d9`, `0x10bf1a62`, `0x10bf80d8` | the other three sites of the literal, all `cgintrin.c` | here |
| `0x10be3e4c` | `ehexcept.c`, mints `0x2e4` four times inside a list walk | here |
| `0x10be4f28` | `except.c`, mints `0x2e4` | here |
| `0x10c37dcc` | `nopstall`'s cycles→register byte table (16 bytes) | here |
| `0x10bfa1ad`, `0x10bfafe9`, `0x10bfa1db`…`0x10bfa258` | encoder form 37 and its nine arms | here |
| `0x10bd575d`, `0x10bd726d` | the literal-operand builder and the instruction builder used by that arm | `0x10bd575d` in `P_OPATTR.md` §5; `0x10bd726d` here |
| `0x10c1707c` | arm 15 (`mr.`) handler — **no path to the unlink** | named by `P_OPATTR.md` §6.1, read here |

`ref/ADDR.tsv` is **generated** (`build_ref.py`, from prose citations plus a
machine-local Ghidra export). These addresses enter it at the next
regeneration on a machine that has the export; the citation above is the
source of record either way.

---

## 8. Instruments, and what running them caught

* [`scripts/probe_r8idiom.py`](scripts/probe_r8idiom.py) — obj-side census.
  **Corpus fence tied to the pinned image**: every capture-cache entry records
  `tool c2.dll <len>:<digest128>`, so the probe sha256-verifies the image,
  **recomputes that line from those bytes** (`digest128` ported from
  `crates/c2-il/src/cachefmt.rs`) and refuses to measure any obj recording a
  different one. `--selftest` checks the port against real entries.
* [`scripts/dump_movearms.py`](scripts/dump_movearms.py) — image side;
  `--arms`, `--nops`, `--word`, `--chain`.

**Every fence was watched refusing deliberately broken input** before any
number above was quoted:

| fence | input | result |
|---|---|---|
| image digest | truncated `c2.dll` (400 kB) | `REFUSE`, exit 1 — both tools |
| image digest | one flipped byte at `0x10000` | `REFUSE`, exit 1 — both tools |
| corpus | empty cache directory | `VACUOUS … NOT a pass`, exit 2 |
| corpus | an obj whose `entry.bin` records a **different c2** | `0 objs measured, 1 foreign` → `VACUOUS`, exit 2 |
| ordering | a cache whose **only** move form is a self-move | measured, **not** reported vacuous |
| mode | an unknown mode argument | `REFUSE`, exit 1 |

**Three defects were found by running these, and all three were mine:**

1. **`--arms`' first guard classifier counted the compare's own not-equal exit
   as a guard**, so all four arms scored "GUARDED" — including arm 6, which
   `P_OPATTR.md` §6.1 reads as unconditional. **Four for four is the tell**: a
   classifier that returns the same answer for every input is measuring itself.
   Fixed; the equal path begins *after* that branch.
2. **Two of the four thunk addresses were hard-coded and wrong.** They came
   from `P_OPATTR.md` §6.1, which names handlers and never claimed to give
   thunks. `--arms` printed `MISMATCH (thunk calls 0x10c1772b)` and that is the
   only reason it surfaced. Thunks are now *found*, by scanning for every
   `call rel32` landing on the handler.
3. **`probe_r8idiom.py`'s `entry.bin` parser paired every value with the next
   key**, because the newline-keyed preamble is not NUL-terminated. `--show`
   printed `src-arg: ?` on an obj that plainly has one.

And one near-miss worth recording: the first search for the literal
`0x7d084378` **found all four sites and I dismissed them**, because I converted
the file offsets to VAs by hand, got `0x10bf8fda` instead of `0x10bf12da`, and
disassembled 31 kB away — where objdump, starting mid-instruction, printed
plausible nonsense (`adc ah,al` / `test BYTE PTR [edi-0x407b31f0],bh`). The
mechanism was on screen an hour before it was found. **Print hex; do not do
base conversion in your head; and never disassemble from an address you
computed by hand without checking that it decodes.**

---

## 9. Consequences for the port

**None adopted, and none required.** For the record, if a future lane emits
this class:

* the word is a **constant**, `0x7d084378` — there is no register allocation to
  model, and modelling one would be wrong;
* it is emitted **once per `0x2e4`**, and `0x2e4` count tracks catch-clause
  count on the shapes measured — but §1.5's limit binds: that is 82 sources of
  one generated family, not a law about C++;
* it is **not** a peephole survivor, so a port that runs a redundant-move
  eliminator over its own IR will *not* accidentally reproduce it, and a port
  that models it as `mr` **will** delete it and lose the byte;
* it needs a `DISCLOSURE.md` row naming `0x10c0e1a1` before the constant enters
  `crates/`.
