# W-BLOCKIR — the probe grid, and what it settled

Two rounds, 4 + 14 + 10 = 28 cells, every one compiled by real `cl.exe`
16.00.11886.00 under wibo at the **workload's own flags**
(`/nologo /c /GR /O1 /Oi /EHsc`) and disassembled with `scripts/gt_dump.py`.
Sources `probe/ipp.cpp`, `probe/walk.cpp`, `probe/bound.cpp`; objs and dumps
regenerate with `probe/cc.sh`.

Round 1 (`ipp.cpp`) is the target TU rewritten with **no `#include`**, so it can
become a fixture. It reproduces `src/system/synth_xbox/IPP_basicmath_xbox.cpp`'s
four bodies **word for word** — 48 / 36 / 48 / 52 B, every instruction identical
to `work/w-blockir/ref/ipp.dis.txt`. That is the fixture's licence.

---

## 1. The three sub-shapes, read off the objs

| shape | statement | walker `r11` | park position | body |
|---|---|---|---|---|
| **A** | `dst[i] OP= src[i]` | **`dst`** | `mr` **BEFORE** the guard | `lfsx f0,rD,r11 · lfs f13,0(r11) · fOPs f0,f0,f13 · stfs f0,0(r11) · addi r11,r11,4` |
| **B** | `dst[i] OP= s` (`s` an FPR formal) | **`dst`**, pre-biased | `addi r11,dst,-4` **AFTER** the guard | `lfs f0,4(r11) · fOPs f0,f0,f1 · stfsu f0,4(r11)` |
| **C** | `dst[i] = a[i] OP b[i]` | **`b`** — the later-DECLARED right-hand array | `mr` **AFTER** the guard | `lfsx f0,rDa,r11 · lfs f13,0(r11) · fOPs f0,f0,f13 · stfsx f0,rDdst,r11 · addi r11,r11,4` |

Common to all three: `cmplwi cr6,r3,0 · bclr 12,26` (the rotated pre-test
realised as a conditional return — `wb-loop`'s pass 1), `mtctr r3` (pass 2),
one `sub rX,other,walker` per array that is **not** the walker, `bdnz` back to
the first body word, `blr`.

**No label, no relocation, no frame, no `.pdata`** in any cell of shape A/B/C.

---

## 2. PREREG §5.1 — walker selection. **W1 HITS on the compound-assign shapes and is REFUTED on the plain-assign one.**

| cell | statement | formals (GPR) | walker | W1 predicted |
|---|---|---|---|---|
| `Add_InPlace` | `f2[i] += f1[i]` | f1=r4 f2=r5 | **f2** | f2 ✔ |
| `Mul_InPlace` | `f2[i] *= f1[i]` | f1=r4 f2=r5 | **f2** | f2 ✔ |
| `c2` | `f1[i] += f2[i]` | f1=r4 f2=r5 | **f1** | f1 ✔ |
| `c7` | `f2[i] -= f1[i]` | f1=r4 f2=r5 | **f2** | f2 ✔ |
| `c8` | `f2[i] /= f1[i]` | f1=r4 f2=r5 | **f2** | f2 ✔ |
| `d5` | `f1[i] *= f2[i]`, +trailing formal | f1=r4 f2=r5 | **f1** | f1 ✔ |
| `c3` | `f1[i] = f2[i]` (no RHS op) | f1=r4 f2=r5 | **f1** | **f2** ✘ |
| `Mul` | `f3[i] = f1[i]*f2[i]` | f1=r4 f2=r5 f3=r6 | **f2** | f2 ✔ |
| `c1` | `f3[i] = f2[i]*f1[i]` | f1=r4 f2=r5 f3=r6 | **f2** | **f1** ✘ |
| `c6` | `f3[i] = f1[i]*f2[i]`, decls reordered | f1=r4 f3=r5 f2=r6 | **f2** (=r6) | f2 ✔ |
| `c4` | `f4[i] = f1[i]+f2[i]+f3[i]` | …r7 | **f2** | **f3** ✘ |

**The rule that survives all eleven cells, stated per shape and not generalised:**

* compound assign (`OP=`) — the walker is the **left-hand array**;
* plain assign with **exactly two** right-hand arrays — the walker is the one
  **declared later**, whatever the *source* order (`c1` proves it is declaration
  order and not source order: `f2[i]*f1[i]` and `f1[i]*f2[i]` produce
  **byte-identical** 52 B bodies, both walking `f2`);
* plain assign with **no** right-hand operator — the walker is the destination
  (`c3`);
* **it does not extend to three right-hand arrays** — `c4` walks `f2`, the
  *second* of three, not the last, and c2 restructures the add tree
  (`fadds f0,f3[i],f1[i]` then `fadds f0,f0,f2[i]`) to get there.

`c4` is the reason the class carries **exactly one** right-hand operator.
`WB_LOOP_FINDINGS.md` §4.3 says of the walker *"In all five measured cells the
walker is the array whose access is emitted last, which is circular. `#1767`'s
rule against a two-point fit applies; not claimed."* This grid is not circular —
it varies declaration order (`c6`), source order (`c1`), formal count (`c5`,
`d5`) and array count (`c3`, `c4`) independently — but it is still **three
separate per-shape rules**, so PREREG **W5 holds**: the class ships them as
transcribed constants with the witness count beside each, not as a derived
allocator.

**PREREG W1 (p = 0.55): SPLIT** — right on the six compound-assign cells and on
`Mul`, wrong on `c1`, `c3` and `c4`. **W2 (0.20), W3 (0.15), W4 (0.10): MISS.**

## 3. PREREG §5.2 — the park's position. **P1 is REFUTED by two cells.**

| cell | walker's incoming reg | last GPR formal | park |
|---|---|---|---|
| `Add_InPlace` | r5 | r5 | **BEFORE** |
| `c5` (trailing unused `unsigned` formal) | r5 | **r6** | **BEFORE** |
| `d5` (trailing unused `unsigned` formal) | r4 | **r6** | **BEFORE** |
| `Mul` | r5 | r6 | AFTER |
| `c6` | **r6** | **r6** | **AFTER** |
| `c4` | r5 | r7 | AFTER |
| `MulConstant`, `c12`, `d3`, `d4` | r4 (`addi`, not `mr`) | r4 | AFTER |

P1 said *"the park floats above the guard **iff** the walker arrives in the last
GPR formal register"*. `c5` and `d5` float it while **not** being last; `c6` does
not float it while **being** last. **PREREG P1 (p = 0.35): MISS**, refuted from
both sides.

The rule that fits all seven: **a `mr` park floats above the guard iff the
preheader has exactly one `sub`** — i.e. iff the loop touches exactly two
arrays. A biased park (`addi`, shape B) never floats. That is **PREREG P2's**
outcome (p = 0.30, *"ships as a per-sub-shape constant"*) and it is what ships:
shape A parks before the guard, shapes B and C after, each a constant with 6 / 4
/ 4 witnesses. **P2: HIT.** P3 (0.20) is close but wrong as stated — the count
that matters is the `sub` count, not the preheader instruction count, and
`MulConstant` (two preheader instructions, no float) is what separates them.

## 4. The operand order is NOT a constant — it flips with commutativity

Shape A, `+=` and `*=` (`Add_InPlace`, `Mul_InPlace`, `c2`, `d5`):

    lfsx f0, rDiff, r11      ← the OTHER array, first
    lfs  f13, 0(r11)         ← the walker
    fadds/fmuls f0, f0, f13

Shape A, `-=` and `/=` (`c7`, `c8`):

    lfs  f0, 0(r11)          ← the WALKER, first
    lfsx f13, rDiff, r11
    fsubs/fdivs f0, f0, f13

Both are `fD = fA OP fB` with the operands in source order; what moves is which
**load** comes first, because the non-commutative op pins `fA` to the left-hand
value. **This is why the class ships `+=` and `*=` and refuses `-=` and `/=`**:
the two arms are a different word order, not a different immediate field, and
`-=`/`/=` are not in the target TU. They are declined **by name, with their
graded cells cited**, which is the honest form of the refusal — the arm is
*absent* rather than present-and-ungraded (board #1148's shape).

## 5. What else the grid settled

| cell | finding | disposition |
|---|---|---|
| `c9` | a **signed** counter/bound gives `cmpwi cr6,r3,0` + `bclr 4,25` — the same two-valued field `counted_accum_loop` already carries | **refused**: IPP is unsigned-only, and an unshipped arm is a wrong emit waiting |
| `c10` | **removing `if (size == 0) return;` changes NOTHING** — the body is byte-identical to `Add_InPlace`. The `for` rotation needs the zero-trip test anyway, so c2 fuses the two | the guard is **redundant in the obj and load-bearing in the IL**; the reader must consume it, and the guard-free IL form is a *different* token stream this lane does not admit |
| `c11` | `double` gives `lfdx`/`lfd`/`fadd`/`stfd` and `addi 11,11,8` | refused — a different scale and four different words |
| `c14` | an `int` array gives the identical skeleton with `lwzx`/`lwz`/`add`/`stw` | refused — the skeleton generalises and this lane does not ship the generalisation |
| `c13` | `f1[i] = f2;` (a scalar splat) becomes a **`_blkmov` tail call**, not a loop at all | a real negative cell |
| `d1`, `d2` | shape C takes `+` and `-` as pure field substitutions (`fadds`/`fsubs`) at the same word slot | refused — not in the target TU |
| `d3`, `d4` | shape B takes `+` and `-` the same way | refused — not in the target TU |
| `e1` | the induction variable used for something besides subscripting: a **second** live value, `li r3,0`/`add r3,r9,r3`, an interleaved schedule and 68 B | refusal cell |
| `e2` | the loop is not the function tail: the body continues past `bdnz` with a REFHI/REFLO pair into `__real@3f800000` | refusal cell |
| `e3` | step 2: `addi r10,r3,-1 · srwi r10,r10,1 · addi r10,r10,1` trip-count arithmetic and `addi r11,r11,8` | refusal cell |
| `e4` | bound ≠ the guard's subject: **two** guards, `cmplwi cr6,r3,0 · bclr` then `cmplwi cr6,r4,0 · bclr` | refusal cell |
| `e5` | two statements in the loop body: an extra `stfsx` inside the loop and `bdnz .-24` | refusal cell |

## 6. The mode call — PREREG M1 HITS

At `/Ox` (`probe/ipp_ox`) c2 **unrolls `Add_InPlace` four times** behind a
`cmpwi cr6,r3,4` pre-test, with a `lfsu`, a remainder loop that re-derives the
walker from `slwi r11,r9,2 · add r11,r11,r5`, and **688 bytes** in a single
`.text` section instead of four COMDATs. The class is `/O1` **only** and must
refuse `/Ox` rather than approximate it.
