# WB-H `wb-loop` — how c2 lowers a counted loop

> **PROVENANCE — DISASSEMBLY-DERIVED.** Obtained by statically disassembling
> Microsoft's `c2.dll` — the exact image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0, sha256 verified at the top of this
> lane as `c80981…6258`. It is **navigation only** until a row is added to
> [`DISCLOSURE.md`](DISCLOSURE.md). **The obj is the sole judge**: §7 grades
> every reading here against real `c2.dll` under wibo, and §7.6/§8 record what
> the objs refuted.

Lane `wb-loop` (WB-H), campaign 2026-08-08. PREREG:
[`WB_LOOP_PREREG.md`](WB_LOOP_PREREG.md), frozen and committed before the first
probe. Grid: [`grids/wb-loop/loop_grid.cpp`](grids/wb-loop/loop_grid.cpp) +
[`frozen.tsv`](grids/wb-loop/frozen.tsv), frozen and committed before the first
`cl.exe` of the grid. Calibration:
[`grids/wb-loop/calib.cpp`](grids/wb-loop/calib.cpp), unscored.

---

## 0. The answer in one screen

c2 lowers a counted loop with **three independent passes**, and the whole
lane's value is that they are independent — each has its own `-QX` disable
switch, and flipping one leaves the other two's output byte-identical (§7.7).

| what | where | disable switch | flag |
|---|---|---|---|
| **rotation + the zero-trip guard** | `p2\lur.c` (upstream; not isolated to a single VA by this lane) | `-NoLUR` | `DAT_10c2ec9c` |
| **the `mtctr`/`bdnz` conversion** | **`p2\ppc\lower.c`**, driver `FUN_10c0f81e` @ **`0x10c0f81e`**, per-loop `FUN_10c0f7f9` @ **`0x10c0f7f9`** | `-QXnobdnz` | `DAT_10c2ecf8` |
| **the `lwzu`/`stwu` update form** | **`p2\misc.c`**, driver `FUN_10b84869` @ **`0x10b84869`**, per-loop `FUN_10b84844` @ **`0x10b84844`**; plus the machine-level peephole `FUN_10c16569` @ **`0x10c16569`** | `-QXnopreinc` | `DAT_10c2ecfc` |

The normal form WB-D saw —

```
    cmpwi   cr6, n, 0        ; the guard: lur.c's rotated pre-test
    bf      25, .Lafter
    addi    p, base, -4      ; the preheader bias
    mtctr   n                ; the trip count
.Lbody:
    lwzu    v, 4(p)          ; the update form
    add     s, v, s
    bdnz    .Lbody
```

— is **three separate rules stacked**, not one lowering. A port can implement
any one of them without the others, and the objs prove it: `/d2QXnopreinc`
removes all 28 update forms and leaves all 29 `bdnz` untouched; `/d2QXnobdnz`
removes all 29 `bdnz` and leaves the guards and the update forms untouched
(§7.7).

---

## 1. Locating the passes (deliverable 1)

`c2_tus.tsv` bands the backend. Four TUs matter here, and **three of the four
are not where the campaign brief expected them**:

| TU | path | band |
|---|---|---|
| `lur.c` | `…\be\p2\lur.c` (string `0x10b13628`) | `0x10b75e1e`–`0x10b7abd5` |
| `misc.c` | `…\be\p2\misc.c` (string `0x10b13828`) | `0x10b7f3f4`–`0x10b86b6c` |
| `globlopt.c` | `…\be\p2\globlopt.c` (string `0x10b02210`) | `0x10b4565a`–`0x10b4a726` |
| `lower.c` | **`…\be\p2\ppc\lower.c`** (string `0x10b1fe38`) | anchor `0x10c053e7` |

**The entry point was an option string, not a function.** The image carries its
own switch list; four of its entries name this lane's subject:

| option | string VA | flag variable | readers |
|---|---|---|---|
| `-QXnobdnz` | `0x10b13be4` | `0x10c2ecf8` | `0x10c0d3fe`, `0x10c0f853` |
| `-QXnopreinc` | `0x10b13bcc` | `0x10c2ecfc` | `0x10b84869`, `0x10c16705`, `0x10c16719`, `0x10c16739` |
| `-QXnoloopreduction` | `0x10b13b5c` | (option entry at `0x10c2a177`) | — |
| `-NoLUR` | `0x10b13d58` | `0x10c2ec9c` | `0x10b84cea` (and one write at `0x10b849c6`) |

and one diagnostic names the mnemonic outright:

> `"Backend doesn't support encoding of CTR changes, use bdnz\n"` @ **`0x10b1e7d0`**,
> single xref **`0x10c03112`** — which is inside `p2\ppc\inlnasm.c`
> (`0x10c01d50`…), i.e. it is the **inline-asm** diagnostic, not the lowering.
> Recorded so the string is not mistaken for the conversion site.

**`-loopopt` (`0x10b1410c`, option entry `0x10c29860`) writes `0x10c2eaf0`, and
`0x10c2eaf0` has ZERO readers in the image.** It is a dead switch in this
build, exactly like WB-D's finding for `-schdat#` (`0x10c2eb40`). PREREG P1.5
predicted it live; scored a miss.

### 1.1 The phase order

`FUN_10c24021` @ `0x10c24021` is the machine-dependent lowering driver
(`p2\ppc\lowersmd.c` band). Its **first** call is `FUN_10c0f81e` — the bdnz
pass — before `FUN_10c2262b`, the instruction peepholes, and
`FUN_10c04faf` (WB-D's volatile/reserved-set setter). So the ctr conversion
runs on the **tuple IR**, before machine instruction selection is finished, and
long before register allocation. The update-form pass `FUN_10b84869` runs
**earlier still**, in the machine-independent `p2\misc.c` band.

---

## 2. THE `mtctr`/`bdnz` DECISION — `FUN_10c0f7f9` @ `0x10c0f7f9`

The driver is trivial and worth quoting because it names the data structure:

```c
// FUN_10c0f81e @ 0x10c0f81e  — p2\ppc\lower.c
if (DAT_10c2e2fc != 0) {                    // optimization on
    ...                                     // FUN_10b36741 / FUN_10b37519: rebuild the flow graph
    if (DAT_10c2ecf8 == 0) {                // NOT -QXnobdnz
        for (L = func[3]; L; L = *L)        // func+0x0c is the LOOP LIST
            FUN_10c0f7f9(func, L);
    }
}
```

`FUN_10c0f7f9` is **a chain of ~20 guard clauses, every one of which
`return 0`s** (refuse), followed by one rewrite. Read in order:

**R0 — inner loops first.** The function opens by recursing over `loop+0x04`,
the child-loop list, *before* testing itself. So conversion is **innermost-out**,
and once an inner loop has taken `ctr`, the enclosing loop fails R2 below. This
is the mechanism behind "the inner loop gets `ctr`" — and §7.2's cell `a5`
shows it is genuinely a *resource* rule and not an "inner" rule.

**R1 — a per-loop veto bit.** `if ((*(byte*)(loop+0x4c) & 1) != 0) return 0;`

**R2 — CTR must be free through the whole loop.** `FUN_10c0d3fe` @ `0x10c0d3fe`
returns 1 (refuse) if `-QXnobdnz`, or `DAT_10c3de20 == 1`, or
`(*(byte*)(DAT_10c472e8 + 0xcdc) & 10) != 0`, **or if any tuple in any block of
the loop satisfies `FUN_10c09c81`** @ `0x10c09c81`.

`FUN_10c09c81` is the CTR-availability test and it has two halves:

* for an **already-machine** tuple (opcode `0` or `> 0x294`): refuse if the
  tuple references the CTR pseudo-symbol **`DAT_10c31008`** (via `FUN_10bd4874`
  use / `FUN_10bd482d` def), or if any of its operands is register **`0x54`**
  (`FUN_10b26f37(operand, 0x54)`). **`0x54` is CTR** — the rewrite at the end
  of `FUN_10c0f7f9` mints its new operand with exactly `FUN_10bd42c2(0x54, 0x2004)`.
* for a **tuple-IR** tuple: refuse on opcode `0x290`, on a kind-`0x12`
  conditional branch whose `FUN_10bd5209` is 0 (an indirect/computed branch),
  and on opcodes `0x2bd`, `0x2dc`, `0x2e8`, `0x2ed`; conditionally on `0x2be`
  and on `0x2eb` (an intrinsic-id range test against `0x96`/`0x98`/`0xac`/`0xae`).

  **A call is refused by the first half, not the second**: a call tuple carries
  CTR in its clobber set, so `FUN_10bd482d(tuple, DAT_10c31008)` hits. This is
  the clause that makes `for (…) s += f(a[i]);` a compare-and-branch loop
  (§7.2), and it is the clause the PREREG got backwards.

**R3 — the loop must be bottom-tested on a recognised compare.** The last tuple
of the loop's tail block (`*(*(loop+0x18)+0x20)+0x10`) must be kind `0x12`
(conditional branch); its opcode must be `0x2e4`, `0x21` or `0x22` (or have a
non-null `[0xd]`); the type nibble must map through table **`0x10b18990`** to
size **4 or 2**; the condition operand's kind must be in `{1,2}`; and the tuple
it comes from must have opcode **`0x2d4`** — the compare.

**R4 — the compare's other side is a symbol with zero displacement**
(`kind == 7`, `+0x18 == 0`, `+0x1c == 0`): the normalised counter.

**R5 — the latch must contain `iv = iv + 1`.** The pass walks the tuple list
back from the compare looking for opcode **`0x2af`** (assign) whose destination
matches the counter; the search must not cross a block boundary (kinds `0x1a`
/ `0x1b` → refuse). The assignment's RHS must be opcode **`0x2c6`** (add) whose
constant operand is **exactly 1** (`+0x18 == 1`, `+0x1c == 0`).

> **The `+1` is on the NORMALISED counter, not on the source stride.** Cell
> `b4` (`i += 3`) converts: `lur.c` had already rewritten it to a unit counter
> with trip count `(n-1)/3 + 1` (§7.3). So R5 is not a restriction on the
> source step.

**R6 — the counter must be referenced NOWHERE ELSE in the loop.** The pass
walks every tuple from the loop head to the terminator calling
`FUN_10bd4874(tuple, counter)`; any hit refuses. This is the induction-variable
elimination precondition: the subscripts must already have been
strength-reduced off the counter (that is `globlopt.c` / `lur.c`'s job, done
before this pass runs).

**R7 — the counter must be dead outside the loop.** `FUN_10c23151` @
`0x10c23151` finds the counter's initialising `0x2af` assignment in the
preheader; then **five** `FUN_10c2317d` @ `0x10c2317d` range checks
(use *or* def, `FUN_10bd4874` / `FUN_10bd482d`) must all come back clean over
the preheader→head, head→compare, compare→latch, latch→tail and tail→exit
ranges. Note §7.2 cell `cal_ivlive`: c2 satisfies R7 by **rematerialising** the
final counter value in the preheader (`mr r9,r4`), it does not refuse.

### 2.1 The rewrite

Only after all of the above:

```c
t = FUN_10bd786a(0x2af, type, counter_operand, …, preheader_pos);  // the mtctr assign
ctr = FUN_10bd42c2(0x54, 0x2004);                                  // the CTR operand
e   = FUN_10bd72b0(0xf8, 0x2004, ctr, …);                          // the ctr decrement/test
b   = FUN_10bd76e6(0x288, 4, <branch label>, ctr, e, …);           // the bdnz branch
FUN_10bd7108(b, ctr);
b[8] = old_branch[8];  old_branch[8] = 0;                          // move the line/label info
FUN_10bd5516(add); FUN_10bd5516(assign); FUN_10bd5516(compare);    // DELETE i=i+1 and the compare
FUN_10bd55fa(old_branch);                                          // replace the branch
```

Three tuples are **deleted** (the add, the assign, the compare) and two are
**created** (the `mtctr` assign in the preheader, the `bdnz` at the latch).
**Nothing here creates the guard and nothing here creates the update form** —
which is why both survive `-QXnobdnz` (§7.7).

### 2.2 The choice, in five sentences (deliverable 1)

> c2 takes the `mtctr`/`bdnz` form when a **single-exit, bottom-tested loop**
> whose exit compare (`0x2d4`) tests a counter that the latch increments by
> exactly one (`0x2af` of `0x2c6`+1) has that counter **referenced nowhere else
> in the body and dead outside it** — the two conditions that make the counter
> pure overhead — and when **CTR is free through the entire loop**, which any
> `bl`/`bctrl` call, any `bctr` jump table and any already-converted inner
> `bdnz` destroys (`FUN_10c0d3fe` @ `0x10c0d3fe` → `FUN_10c09c81` @
> `0x10c09c81`, refusing on the CTR pseudo-symbol `DAT_10c31008` and on
> register operand `0x54`). It is decided in `p2\ppc\lower.c`
> (`FUN_10c0f7f9` @ `0x10c0f7f9`, driven from `FUN_10c0f81e` @ `0x10c0f81e`)
> **on the tuple IR, before instruction selection completes and long before
> register allocation** — so it is upstream of everything WB-D read. The pass
> runs **innermost-first** over the loop tree, which is why the inner loop
> normally wins CTR, but the rule is a *resource* rule and not an "inner" rule:
> when the inner loop fails for another reason the **outer** loop takes CTR
> instead (grid cell `a5`, §7.2). Everything else — the zero-trip guard, the
> `lwzu` walk, the trip-count arithmetic — is somebody else's pass.

---

## 3. THE ZERO-TRIP GUARD — it is the rotated pre-test, not a `>0` test

The guard is **not** emitted by the ctr pass (§2.1 creates no compare) and it
survives `-QXnobdnz` unchanged (§7.7). It is `lur.c`'s loop **rotation**: a
top-tested `for`/`while` becomes `if (pretest) do { … } while (…);`, and the
`if` is the guard.

The consequence is the sharpest black-box prediction this lane made, and it
came out exactly right:

**The guard compares the loop's START expression against its BOUND, with the
loop's own signedness, in `cr6` — it is not a test of the trip count against
zero.**

| cell | source | guard emitted |
|---|---|---|
| `cal_base` | `for (i=0; i<n; ++i)` | `cmpwi 6,r4,0` / `bf 25` (not-GT) |
| `b1` | `for (i=3; i<n; ++i)` | **`cmpwi 6,r4,3`** / `bclr 4,25` |
| `b5` | `for (i=0; i<=n; ++i)` | `cmpwi 6,r4,0` / **`bclr 12,24`** (branch on **LT**) |
| `b6` | `for (unsigned i=1; i<n; ++i)` | **`cmplwi 6,r4,1`** / `bclr 4,25` |
| `cal_uns` | `for (unsigned i=0; i<n; ++i)` | `cmplwi 6,r4,0` / `bt 26` (**EQ**) |
| `b2` | `for (i=3; i<10; ++i)` | **omitted** — the pre-test is a compile-time truth |
| `cal_c2…c64` | constant trip counts 2…64 | **omitted** |
| `cal_down` / `c8` | `for (i=n-1; i>=0; --i)` | `addic. r,n,-1` / `bclr 12,0` (record form, **CR0**) |

Four separate branch conditions (`bf 25`, `bt 26`, `bclr 12,24`, `bclr 12,0`)
fall straight out of one rule; a "compare the trip count against 0" rule
predicts one of them.

**Realisation.** When the loop is the function's tail and the fall-through is
the epilogue, the guard becomes a **conditional return** (`bclr`) instead of a
forward branch (`b1`, `b2`… vs `b7`, which has code after the loop and gets
`bf 25, .+24`). That is a peephole on the branch target, not a different rule.

**CR field.** Every non-record-form guard compare in the grid is **`cr6`** —
WB-D §7.5's retraction generalises to loops. The record forms (`addic.`) use
CR0 implicitly.

**The guard does not need the ctr form.** Cells `a1`, `a2`, `a4`, `a9`, `a10`,
`a14` all have a guard and **no** `bdnz`. Rival RG2 ("the guard is part of the
ctr lowering") is refuted on six cells.

**c2 does not use dominating conditions.** Cell `b3` writes
`if (n<=0) return 0;` immediately above the loop; c2 emits the `n<=0` test and
then **a second, identical** `cmpwi 6,r4,0` guard. At `/O1` the rotation does
not consult the dominator's range information.

---

## 4. THE INDUCTION REWRITE AND THE UPDATE FORM (deliverable 2)

### 4.1 The pre-decrement

PPC's `lwzu rD,d(rA)` computes `EA = rA + d` and then writes `rA = EA` — the
update is **before** the access. So the walking pointer must start one stride
*behind* the first element. The preheader constant is therefore

```
    bias = (byte offset of the first accessed element) − (byte stride)
```

and the grid nails the general form rather than the `-4` special case:

| cell | first element at | stride | preheader | access |
|---|---|---|---|---|
| `cal_base` | `+0` | 4 | `addi p,base,-4` | `lwzu v,4(p)` |
| `b1` (`i` from 3) | `+12` | 4 | **`addi p,base,+8`** | `lwzu v,4(p)` |
| `b2` (`i` 3..9) | `+12` | 4 | `addi p,base,+8` | `lwzu v,4(p)` |
| `b4` (`i += 3`) | `+0` | **12** | **`addi p,base,-12`** | **`lwzu v,12(p)`** |
| `c7` (`a[i].y`, 12-byte struct) | **`+4`** | **12** | **`addi p,base,-8`** | **`lwzu v,12(p)`** |
| `c8` (descending store) | `+4(n-1)` | **−4** | `addi p,…,+4n` | **`stwu k,-4(p)`** |

`c7` is the cell that proves it is `first − stride` and not `−stride`:
`4 − 12 = −8`, and `-8` is what c2 emits.

### 4.2 Which mnemonic

| element type | emitted |
|---|---|
| `int` | `lwzu` / `stwu` |
| `short` | **`lhau`** (signed load-and-update, *not* `lhzu`) |
| `char` / `unsigned char` | **no update form at all** — `lbzx rD,idx,base` with an integer index register |
| `float` | `lfsu` |
| `double` | `lfdu` |
| `long long` | `ldu` |
| non-constant stride | **`lwzux`** (the update-**indexed** form) |

**A 1-byte stride never gets a pointer walk.** Both `cal_char` (signed) and
`c4` (unsigned) keep `i` as a real index register and use `lbzx`. That is
measured twice and is not a signedness effect.

### 4.3 Multiple arrays — the base-difference rule (the biggest scratch)

The PREREG predicted one update-form pointer per array. **That is wrong and it
is wrong in an interesting way.** When ≥2 arrays are indexed by the same affine
expression *with the same byte stride*, c2 keeps **exactly one** walking
pointer and reaches every other array by an **X-form access at a
preheader-computed base difference**:

```
    ; c1:  s += b[i] + a[i]      (a=r3, b=r4, n=r5)
    sub   r7, r3, r4         ; a − b, computed ONCE in the preheader
    lwzx  r9, r7, r11        ; a[i]  =  (a−b) + walker
    lwz   r8, 0(r11)         ; b[i]
    addi  r11, r11, 4        ; the walker — and it can no longer fold
```

`cal_four` shows it scaling: three `sub`s in the preheader, three `lwzx` in the
body, one walking pointer. **The number of live pointers is 1, not N** — which
is presumably the point.

**When the strides differ the trick is inapplicable and c2 keeps two induction
variables.** Cell `c3` (`int a[]` + `char b[]`) emits a genuine `lwzu v,4(p)`
walk for the `int` **and** an `lbzx`+index for the `char`, in one loop. This
was the frozen prediction and it is the cell that separates the address-form
rule from every "one update form per loop" rival.

**And the base-difference trick is what kills the update form**, not array
count: the walker is now the *index* operand of an `lwzx`, which has no
displacement field, so the `addi p,p,4` cannot be folded into it.

### 4.4 What actually selects the update form — REFUTED and replaced

The frozen rule (RU0'-b) was: *fold `addi p,p,S` into p's last access iff every
other use of p is a D-form access that can absorb a `+S` displacement.* Cells
`c9` (`a[i] + a[i+1]`) and `c10` (`a[i] + a[i+1] + a[i+2]`) **refute it** — two
and three D-form accesses on one walker, and c2 emits **no** update form,
keeping a separate incremented copy instead:

```
    ; c10
    addi r9, r11, 4          ; the next walker value, in a SECOND register
    lwz  r7, -4(r11)
    lwz  r6,  4(r11)
    lwz  r8,  0(r11)
    mr   r11, r9
```

RU0'-b is retracted. The rule that fits all 36 grid cells **and** all 24
calibration cells is narrower — but it is **fitted after the fact and has not
been through a frozen grid**, so it is filed as a hypothesis, not a finding:

> **RU-H (hypothesis, unfrozen).** The `addi p,p,S` is folded into a memory
> access on `p` iff (i) `|S| ≥ 2` bytes, (ii) **every** access on `p` in the
> body uses the **same** displacement — i.e. there is exactly one distinct
> address expression on `p` — (iii) `p` is not the base or index of any X-form
> access, and (iv) all of those accesses are in the **same basic block** as the
> increment.

Clause (ii) is the one `c9`/`c10` forced and (iv) is the one `cal_ifelse` and
`cal_cont` force (a body with an `if` in it loses the update form even with a
single access). `cal_rmw` (`a[i] = a[i] + 1`) is the cell that shows (ii) is
about *displacement* and not about *count*: two accesses, same address,
`lwz r,4(p)` + **`stwu r,4(p)`** — the fold lands on the **last** access and
the earlier one is pre-biased by `+S`.

Note **RU2** ("update form iff the loop has exactly one memory reference")
scores the *same* 8/10 as the frozen RU0' on this grid, with disjoint failures
(`c3` and `c4` refute RU2; `c9` and `c10` refute RU0'). **Neither rival won.**
Saying so is the result; RU-H is the next lane's frozen grid, not this lane's
finding.

---

## 5. THE CLASS BOUNDARY IN PORT TERMS (deliverable 4)

A `loop_counted` lowering class in `c2-core` should accept a loop iff **all** of
the following hold on the port's own IR. Every clause is checkable without any
number out of `c2.dll`, so **the predicate needs no DISCLOSURE row** (§10).

```
loop_counted(L) :=
    L.back_edges == 1                            // single latch
  ∧ L.exits      == 1                            // no break / return / goto out   [a1,a2,cal_break]
  ∧ L.counter is a 32-BIT INTEGER local          // not 64-bit, not a pointer      [a9,a14]
  ∧ L.counter.step is a compile-time constant    // any constant; lur.c normalises [b4,cal_inc2]
  ∧ L.bound is a loop-invariant SYMBOL, not a
      computed expression                        //                                [a10]
  ∧ L.counter used ONLY by the exit compare and
      by affine subscripts that were strength-
      reduced away                               // R6
  ∧ L.body contains NO call (direct or indirect),
      NO computed branch (jump-table switch),
      and NO nested loop that already took CTR    // R2                            [a4,a5,a6,a7,d4]
  ∧ L.body is ONE basic block                    // required for the update form,
                                                 // not for bdnz                   [d3,cal_ifelse]
```

with the emission rule

```
guard:      if the pre-test (start REL bound) is not a compile-time truth,
            emit  cmp{w,lw}i cr6, bound, start   +  branch-over
            (a conditional RETURN when the loop is the function tail)
preheader:  addi  p, base, first_offset - stride       (per walking pointer)
            one `sub` per additional same-stride array (base_i - base_walk)
            mtctr trip_count
body:       the single walking pointer's access takes the update form iff RU-H;
            every other same-stride array is `Xzx rD, diff, p`
latch:      bdnz  body
```

**What is NOT in the class**, stated so absence does not read as coverage:

* the **trip-count arithmetic** for a non-unit step. `b4` emits
  `addi n,-1` / `li 3` / `divwu` / `addi +1`; `cal_inc2` emits
  `addi n,-1` / `srwi 1` / `addi +1`. This lane did not read the code that
  picks `divwu` over a shift and cannot predict it for an arbitrary constant.
* the **walker selection** when there are ≥2 same-stride arrays. In all five
  measured cells the walker is the array whose access is emitted last, which is
  circular. `#1767`'s rule against a two-point fit applies; not claimed.
* **RU-H** itself (§4.4).

### 5.1 The block-order question WB-D left open (deliverable 4, second half)

WB-D §9.2 item 3 recorded "M1 says source order, M2 says reverse. Both from the
same compiler on the same day." **This lane can close part of it.**

| observation | cells |
|---|---|
| Straight-line and `if`/`else`/`else-if` blocks are emitted in **source order** | `d3` (then, else-if, else), `cal_ifelse`, `b3` (the `n<=0` early return precedes the loop), `d2` (the `return -1` arm precedes the loop) |
| Two sequential loops are emitted in **source order** | `cal_seq` |
| A nested loop is emitted **inside** its parent, at its source position | `a6`, `d4` |
| A block reachable only as an **exit from inside a loop body** is **SUNK past the function's normal return** | **`a1`** — `mr r3,r9 / blr / li r3,-1 / blr` |
| A `switch` lowered to a **decision tree** emits its arms in **REVERSE source order** | **`d5`** — case 2, case 1, case 0(the loop) |
| A `switch` lowered to a **JUMP TABLE** emits its arms in **SOURCE order** | **`a7`** — 12 dense cases, `lbzx` index table + `bctr`, arms `case 0 … case 11` then `default` |

So WB-D's M1/M2 contradiction is **not a contradiction**: M2 was a
six-case *decision tree*, and decision trees emit reversed. `a7` is the control
WB-D did not have — same construct, enough cases to become a table, and the
order flips back to source. **Two rules, one per switch lowering**, plus the
loop-exit sinking rule from `a1`.

This does not make block order *predictable* in general — the pivot choice
inside a decision tree is still unread — but it removes the case that made the
question look unanswerable, and it gives a port a correct rule for the
`loop_counted` class (which contains no switch): **source order, with
loop-exit-only blocks sunk after the return.**

---

## 6. THE OBJ-CHECK — FROZEN BEFORE THE FIRST `cl.exe` OF THIS GRID

Source: [`grids/wb-loop/loop_grid.cpp`](grids/wb-loop/loop_grid.cpp),
sha256 `d81778bfdb59c9b54826005daa3335a064f5508f9958a0597760ef1e662966a6`.
Predictions: [`grids/wb-loop/frozen.tsv`](grids/wb-loop/frozen.tsv),
sha256 `5a3254f80d0706b8c6856e7856cb81c7c9e314f249693f917d19624697de9984`.
Both committed in `4dd7c77` **before** the grid's first compile; the run is
`work/wb-loop/run/grid.obj` (not committed — it is an obj).

36 cells, one COMDAT each, `/nologo /c /GR /O1 /Oi /EHsc` (WB-D's workload
mode). Six `mtctr` rivals (RC0'…RC5), four update-form rivals (RU0'…RU3), four
guard rivals (RG0'…RG3n), and two counterfactual runs. The
minimum-separation assertion is in `frozen.tsv` and was checked before the run.

### 6.1 The calibration pass, and why it exists

`grids/wb-loop/calib.cpp` (24 cells) was compiled **first and is unscored**.
The brief required it because wb-inline's v1 grid was refuted by its own cells
when a folding compiler collapsed the ladder. It earned its keep immediately:

* **`/O1` does not unroll at all.** Constant trip counts 2, 3, 4, 6, 8, 16 and
  64 all produce the *identical* nine-word `li / li / addi -4 / mtctr / lwzu /
  add / bdnz / mr / blr` body. PREREG P2.5 had registered "≤4 unrolled, ≥8
  keeps a loop" — a whole block of planned constant-trip cells would have
  measured nothing. Block A of the frozen grid has **no** constant-trip cells
  as a direct result.
* **A call in the body evicts the loop from CTR** — which flipped the frozen
  prediction for the entire indirect-call / nested-call family before it cost a
  cell.
* **Two arrays share one walking pointer** — which is why `c3` (differing
  strides) exists at all, and it is the grid's best cell.

---

## 7. RESULTS

### 7.1 The `mtctr` choice — 34 of 36

| cell | emitted | RC0' | RC1 | RC2 | RC3 | RC4 | RC5 |
|---|---|---|---|---|---|---|---|
| `a1_ret` (early `return`) | compare-branch | ✅ | ✗ | ✅ | ✗ | ✗ | ✗ |
| `a2_goto` | compare-branch | ✅ | ✗ | ✅ | ✗ | ✗ | ✗ |
| `a3_inline` | `bdnz` | ✅ | ✅ | ✗ | ✅ | ✅ | ✅ |
| `a4_indcall` (`bctrl`) | compare-branch | ✅ | ✅ | ✅ | ✗ | ✗ | ✅ |
| **`a5_nest_inner_break`** | **outer `bdnz`, inner compare-branch** | ✅ | ✗ | ✗ | ✗ | ✗ | ✗ |
| `a6_nest_both` | inner `bdnz`, outer compare-branch | ✅ | ✅ | ✗ | ✗ | ✗ | ✗ |
| `a7_switch` (12 dense cases) | **jump table + `bctr`**, loop compare-branch | ✅ | ✅ | ✅ | ✗ | ✗ | ✗ |
| `a8_div` | `bdnz` | ✅ | ✅ | ✗ | ✅ | ✅ | ✅ |
| **`a9_i64`** (64-bit counter) | **compare-branch** (`cmpdi`/`cmpldi`) | **✗** | ✗ | ✅ | ✗ | ✗ | ✗ |
| **`a10_expr`** (bound `n/2+3`) | **compare-branch** | **✗** | ✗ | ✅ | ✗ | ✗ | ✗ |
| `a11_vol` (`volatile`) | `bdnz` + `lwzu` | ✅ | ✅ | ✗ | ✅ | ✅ | ✅ |
| `a12_while` | `bdnz` | ✅ | ✅ | ✗ | ✅ | ✅ | ✅ |
| `a13_fp` | `bdnz` + `lfsu` + `fmadd` | ✅ | ✅ | ✗ | ✅ | ✅ | ✅ |
| `a14_ptriv` (pointer IV) | compare-branch | ✅ | ✗ | ✅ | ✗ | ✗ | ✗ |
| `b1`…`b7` (7 cells) | all `bdnz` | 7/7 | 7/7 | 1/7 | 7/7 | 7/7 | 7/7 |
| `c1`…`c10` (10 cells) | all `bdnz` | 10/10 | 10/10 | 0/10 | 10/10 | 10/10 | 10/10 |
| `d1`,`d2`,`d3`,`d5` | all `bdnz` | 4/4 | 4/4 | 0/4 | 4/4 | 4/4 | 4/4 |
| `d4_twonest` | inner `bdnz`, outer compare-branch | ✅ | ✅ | ✗ | ✗ | ✗ | ✅ |

| rival | score | verdict |
|---|---|---|
| **RC0'** — single-exit ∧ unit-normalised integer counter ∧ CTR free ∧ integer IV | **34 / 36** | **SURVIVES**; the two misses are §7.4 |
| RC1 — exits do not matter | 31 / 36 | **REFUTED** by `a1`, `a2`, `a5`, `a14` (+ the two shared misses) |
| RC2 — constant trip count required | 9 / 36 | **REFUTED**, 27 cells |
| RC3 — always | 26 / 36 | **REFUTED**, 10 cells |
| RC4 — late peephole whenever the counter is dead | 26 / 36 | **REFUTED**, 10 cells |
| RC5 — "no call in the body" (the one-clause rule) | 28 / 36 | **REFUTED** by `a1`, `a2`, `a5`, `a6`, `a7`, `a14` |

**`a5` is the cell of the lane.** RC0' predicted, in advance and against the
obvious reading, that when the *inner* loop is disqualified the **outer** loop
takes CTR. It does:

```
    cmpwi  6, r4, 0
    bf     25, .+60
    mtctr  r4                  ; <-- the OUTER loop
    …
      lwz  r9,0(r10) / cmpwi 6,r9,0 / bt 26 (the break)   ; the INNER loop,
      addi r11,r11,1 / add / addi r10,10,4                ; compare-and-branch
      cmpw 6, r11, r5 / bt 24
    bdnz   .-48                ; <-- the OUTER latch
```

RC1 predicted the exact opposite (`in=Y out=N`); every other rival predicted
both or neither. It confirms `FUN_10c0f7f9`'s **innermost-first recursion plus
`FUN_10c0d3fe`'s CTR-availability scan** as a *resource* allocation, and refutes
"the inner loop gets `ctr`" as a rule.

### 7.2 What the calls did, and the WB-D RETRACTION

`a4`'s body is the mechanism in one obj: `mtctr r28` / `bctrl` — the indirect
call **is** the CTR user, so the loop cannot have it, and c2 falls back to
`addic. r31,r31,-1` / `bf 2, …`.

> **RETRACTION on WB-D's behalf.** `WB_REGALLOC_FINDINGS.md` §9.3 says the
> counted-loop normal form was "**identical across three different bodies** (L1,
> L2, L3 share `cmpwi cr6` guard / `addi ptr,-4` / `mtctr` / `lwzu` / `bdnz`)"
> and picks that class as the first to attempt *because* of the three-witness
> stability. **L3 does not share it.** `wbr_loop_call` has a `bl` in the body;
> recompiled here (as `cal_call`, byte-identical shape) it emits
> `mr r31,r4` / `bl` / **`addic. r31,r31,-1`** / **`bf 2`** — no `mtctr`, no
> `bdnz`. The class is stable across **two** witnesses (L1, L2), not three, and
> the third witness is on the *other* side of the boundary. WB-D's §7.2 row F5
> only checked L1, so its own grid never graded the claim.

The correction does not damage WB-D's conclusion — a two-witness shape that
this lane then reproduced on 28 further cells is still a class — but "identical
across three bodies" was the stated evidence and it is wrong.

### 7.3 The guard — 7 of 7, and RG3n refuted

| cell | predicted (frozen) | emitted | verdict |
|---|---|---|---|
| `b1` | `cmpwi cr6,n,3`; `mtctr` gets `n-3`; base `+8` | `cmpwi 6,r4,3` / `addi r10,r4,-3` / `addi r11,r11,8` | **HIT** (all three) |
| `b2` | guard **omitted**; `li r,7`; base `+8` | `li r9,7` / `addi r10,r3,8`, no compare | **HIT** |
| `b3` | guard **present** despite the dominating `n<=0` | two identical `cmpwi 6,r4,0` | **HIT** |
| `b4` | present vs 0; trip `(n-1)/3+1` | `cmpwi 6,r4,0`; `addi -1` / `li 3` / `divwu` / `addi +1` | **HIT** (arithmetic by `divwu`, not multiply-high — noted, not scored) |
| `b5` | signed vs 0, branch on **LT** (not "not GT") | `cmpwi 6,r4,0` / **`bclr 12,24`** | **HIT** |
| `b6` | **unsigned** compare against **1** | **`cmplwi 6,r4,1`** | **HIT** |
| `b7` | **forward** branch, not `bclr` | `bf 25,.+24` | **HIT** |

| rival | verdict |
|---|---|
| **RG0'** — the guard is the rotated pre-test (start REL bound, loop's own signedness, `cr6`) | **SURVIVES 7/7 plus 8 calibration cells** |
| RG1 — always emitted | **REFUTED** by `b2` and all seven `cal_c*` |
| RG2 — emitted iff `mtctr` is chosen | **REFUTED** by `a1`, `a2`, `a4`, `a9`, `a10`, `a14` (6 cells) |
| RG3n — always a `> 0` test on the trip count | **REFUTED** by `b1`, `b5`, `b6` |

### 7.4 The two MISSES, stated as misses

**`a9_i64` — a 64-bit counter defeats the conversion.** `for (long long i=0;
i<n; ++i)` emits `cmpdi 6,r4,0` for the guard and then `cmpldi 6,r10,0` /
`bt 25` for the loop — no `mtctr`, even though CTR is 64 bits wide on this
target and the loop is otherwise textbook. RC0' predicted `bdnz`. Reading
back, R3's type-nibble test through table `0x10b18990` requires size **4 or 2**
and an 8-byte counter maps to neither — so **the reading predicted this cell and
the lane did not**. That is precisely WB-D §7.3's failure mode ("R0 the rule is
not refuted; R0-as-I-applied-it is") and it is scored the same way: the cell is
a **MISS** and is not re-scored in RC0's favour.

**`a10_expr` — a computed bound defeats it.** `for (i=0; i<n/2+3; ++i)` emits
`srawi` / `addze` / `addic.` for the guard and then a `cmpw 6,r11,r9`
compare-and-branch. `b4` proves computed *trip counts* are fine, so the
boundary is on the **bound operand**: R4 requires the compare's other side to
be a symbol with zero displacement (`kind == 7`, `+0x18 == 0`), and a
temporary holding `n/2+3` is not that. Again the reading covers it and the
lane's prediction did not. **MISS.**

Both are also **scratches** in the PREREG P7.4 sense: no rival named the
64-bit-counter or computed-bound boundary, and both are now clauses of §5.

### 7.5 The update form — no rival won

| cell | predicted | emitted | RU0' | RU1 | RU2 | RU3 |
|---|---|---|---|---|---|---|
| `c1_ba` (2 arrays) | no u-form | `sub`+`lwzx`+`lwz`+`addi` | ✅ | ✗ | ✅ | ✅ |
| `c2_copy` | no u-form, **not** memcpy | `sub`+`lwzx`+`stw`+`addi` | ✅ | ✗ | ✅ | ✅ |
| **`c3_mixed`** (strides 4 and 1) | **`lwzu 4` for the int, index form for the char** | `lwzu 8,4(r9)` **and** `lbzx 7,r11,r4` | ✅ | ✅ | ✗ | ✗ |
| `c4_uchar` | no u-form (1-byte stride) | `lbzx`+index | ✅ | ✗ | ✗ | ✅ |
| `c5_float` | `lfsu 4` | `lfsu 0,4(r11)` | ✅ | ✅ | ✅ | ✗ |
| `c6_i64` | `ldu 8` | `ldu 9,8(r11)` | ✅ | ✅ | ✅ | ✗ |
| **`c7_struct`** | `lwzu 12`, base bias **−8** | `addi r11,r3,-8` / `lwzu 9,12(r11)` | ✅ | ✅ | ✅ | ✗ |
| `c8_downstore` | `stwu -4` | `stwu 5,-4(r10)` | ✅ | ✅ | ✅ | ✗ |
| **`c9_selfoff`** | one u-form + one D-form | **no u-form**, `addi r9,r11,4` / `mr r11,r9` | **✗** | ✗ | ✅ | ✅ |
| **`c10_three`** | one u-form + two D-form | **no u-form** | **✗** | ✗ | ✅ | ✅ |
| | | **totals** | **8/10** | 5/10 | **8/10** | 5/10 |

**RU0'-a (the base-difference address-form rule) survives 3/3** (`c1`, `c2`,
`c3`) and is the strongest single reading in §4. **RU0'-b (the update-fold
clause) is REFUTED** by `c9`/`c10` and retracted in §4.4. RU2 ties RU0' at 8/10
on *disjoint* cells, so this grid **elects no update-form rule** — recorded as
the result rather than resolved by picking the prettier one. RU-H (§4.4) is the
next lane's frozen grid.

### 7.6 Block order — 5 of 7

`d1`, `d2`, `d3`, `d4` and `d5` are hits, including the frozen call that a
decision-tree `switch` emits its arms in **reverse** source order (`d5`,
confirming WB-D's M2 as a rule and not an accident).

Two misses:

* **`a1` — the loop-exit block is SUNK.** Frozen: "early-return arm before the
  loop tail". Emitted: `mr r3,r9 / blr / li r3,-1 / blr` — the `return -1` is
  **after** the function's normal return. PREREG P6.1 ("no block is emitted
  after the function's return") is a **MISS**, and the rule it becomes is in
  §5.1.
* **`a7` — a jump-table `switch` emits arms in SOURCE order**, not reverse.
  Frozen said reverse (generalising WB-D's M2). The miss is what produced the
  two-rule split in §5.1, so it is the more useful of the two.

### 7.7 THE COUNTERFACTUAL RUNS — the option→pass links, obj-confirmed

The two `-QX` switches are passed to the backend through `cl.exe` as
`/d2QXnobdnz` and `/d2QXnopreinc`; the same 36-cell source, same mode.

| run | `bdnz` | `mtctr` | update forms | predicted |
|---|---|---|---|---|
| baseline | **29** | 31 | **28** | — |
| `/d2QXnobdnz` | **0** | **2** | 23 | "every `bdnz` disappears; guards survive" |
| `/d2QXnopreinc` | **29** | 31 | **0** | "every update form disappears; `bdnz` survives" |

Both predictions **HIT exactly**. The two surviving `mtctr` under `-QXnobdnz`
are `a4`'s `bctrl` and `a7`'s `bctr` — the two genuine indirect branches, which
is the control that says the count is measuring the right thing. Guards survive
both flags unchanged (`a11` keeps `cmpwi 6,r4,0` / `bf 25`; `b2` keeps no
guard).

This is the lane's cleanest result and it is **three claims at once**:

1. the `bdnz` conversion is a separable pass gated by `DAT_10c2ecf8`, exactly
   as read at `FUN_10c0d3fe` / `FUN_10c0f81e`;
2. the update form is a separable pass gated by `DAT_10c2ecfc`, exactly as read
   at `FUN_10b84869` / `FUN_10c16569`;
3. **the zero-trip guard is neither** — it survives both, so §3's attribution
   of it to `lur.c`'s rotation is confirmed by elimination.

The `-QXnobdnz` fallback shape is `addic. rN,rN,-1` / `bf 2` — **the same words
c2 emits for a loop with a call in it**. One fallback path, reached two ways.

---

## 8. PREREG SCORE

30 hits, 12 misses, 2 not scored. Seven of the twelve misses were registered
**optimistic**, which is board #770's streak continuing rather than breaking.

| # | verdict | note |
|---|---|---|
| P0.1 floor cleared | **HIT** | RC0' 34/36 |
| P0.2 decidable from source shape | **HIT** | §5's predicate needs no profile |
| P0.3 guard easier than the choice | **HIT** | 7/7 vs 34/36, and the guard's four branch conditions all fell out of one rule |
| P1.1 `lur.c` band `0x10b75000`–`0x10b7b000` | **HIT** | `0x10b75e1e`–`0x10b7abd5` |
| P1.2 the conversion is machine-dependent, not in `lur.c` | **HIT** | `p2\ppc\lower.c`, `0x10c0f7f9` |
| P1.3 the update form is machine-dependent | **MISS** | the per-loop driver `0x10b84869` is `p2\misc.c`, machine-**in**dependent |
| P1.4 a named greppable artifact exists | **HIT** | `-QXnobdnz`, and the `"use bdnz"` inline-asm diagnostic |
| P1.5 `-loopopt` is live | **MISS** (optimistic) | `0x10c2eaf0` has **zero** readers — a second dead switch beside `-schdat#` |
| P2.1 non-constant trip counts qualify | **HIT** | |
| P2.2 an IV live after the loop refuses | **MISS** (optimistic) | c2 **rematerialises** it in the preheader (`cal_ivlive`: `mr r9,r4`) |
| P2.3 `break` kills it | **HIT** | `a1`, `a2`, `a5`-inner, `cal_break` |
| P2.4 a call does **not** kill it | **MISS** | it does; and it retracts WB-D §9.3 (§7.2) |
| P2.5 ≤4 unrolled, ≥8 loops | **MISS** (optimistic) | `/O1` never unrolls: 2…64 identical |
| P2.6 down-counting loops keep the form | **HIT** | `cal_down`, `c8` |
| P2.7 `while` ≡ `for` | **HIT** | `a12` byte-identical to `cal_base` |
| P2.8 inner gets `ctr`, outer does not | **HIT**, and its stated *reason* is confirmed by `a5` | |
| P3.1 `cmpwi cr6` + `bf 25` | **HIT** for the plain case; §3 is the general form | |
| P3.2 the guard exists because `bdnz` is a do-while | **HIT** | |
| P3.3 omitted for a constant count ≥1 | **HIT** | `b2`, `cal_c*` |
| P3.4 omitted for a source `do{}while` | **MISS** (optimistic) | `cal_dowhile` has no guard **and** no `bdnz` — it is not converted at all |
| P3.5 unsigned `n` still guarded, `cmplwi` | **HIT** | `cal_uns`, `b6` |
| P3.6 the guard compare is always `cr6` | **HIT** | except the record forms (`addic.`), which are CR0 by construction |
| P4.1 preheader bias = −stride | **HIT**, and generalised to `first − stride` by `c7`'s **−8** | |
| P4.2 two arrays → two update forms | **MISS** (optimistic) | one walker + base difference (§4.3) |
| P4.3 four arrays → per-array update form | **MISS** (optimistic) | `cal_four`: three `sub`s, one walker |
| P4.4 non-unit constant stride keeps the form | **HIT** | `cal_stride2`, `b4`, `c7` |
| P4.5 element → mnemonic family | **MISS** | `lfsu`/`lfdu`/`ldu` right; `short`→`lhau` (signed) and `char`→ **no update form at all** |
| P4.6 write-only loop → `stwu` | **HIT** | `cal_store`, `c8` |
| P4.7 rmw → `lwzu` + plain `stw` | **MISS** (optimistic) | it is the mirror: `lwz 4(p)` + **`stwu 4(p)`** |
| P4.8 non-constant stride defeats the update form | **MISS** | `mtctr` kept ✅ but the form is **`lwzux`**, an update form. Scored a miss per the no-rescue rule |
| P5.1 ~6-clause conjunctive predicate | **HIT** | §5 has eight |
| P5.2 the predicate needs no DISCLOSURE row | **HIT** | §10 |
| P5.3 reach over the 124-TU pool = 0 | **not scored** | this lane adopts nothing and measured no reach; registered pessimistically and nothing here moves it |
| P5.4 what a class needs beyond registers | **judgment**, §9 | |
| P6.1 no block after the function's return | **MISS** | `a1` sinks the loop-exit arm past the `blr` |
| P6.2 then-arm first inside a body | **HIT** | `d3`, `cal_ifelse` |
| P6.3 two sequential loops in source order | **HIT** | `cal_seq` |
| P6.4 `continue` target at the end of the body | **HIT** | `cal_cont` |
| P6.5 the general order rule survives | **HIT with a refinement** | jump table = source order, decision tree = reverse (§5.1) |
| P7.1 ≥20 cells | **HIT** | 36 + 24 calibration + 2 counterfactual runs |
| P7.2 calibration necessary and moves ≥1 cell | **HIT** | it deleted a whole planned block (§6.1) |
| P7.3 a rival fully refuted by ≥3 cells | **HIT** | RC2 on 27, RC3/RC4 on 10, RC5 on 6, RG2 on 6 |
| P7.4 ≥1 scratch | **HIT** | `a9` (64-bit counter) and `a10` (computed bound) |
| P7.5 ≥1 retraction | **HIT** | WB-D §9.3's L3, and this lane's own RU0'-b |

---

## 9. THE JUDGMENT — what a `loop_counted` class needs beyond WB-D's register rule (deliverable 5)

WB-D §9.1 concluded *"yes for register assignment; no for instruction selection
and block order — and those, not registers, are what a class lowering actually
needs."* **This lane converts two of the three "no"s into "yes", and finds a
fourth requirement nobody had listed.**

**1. The loop normal form: YES, and it is three rules, not one.** WB-D §9.2
item 2 asked for "`lur.c`'s output shape" as a single thing. It is not one
thing, and the `-QX` counterfactuals prove it (§7.7): rotation-plus-guard, the
`ctr` conversion, and the update form are three passes that can be enabled
independently and whose outputs compose. **A port can ship them in that order
and be byte-correct at each step for a growing subset of loops** — which is a
much better shape than an all-or-nothing "normal form". Specifically, shipping
only rule 1 + rule 2 (guard + `bdnz`, no update form) reproduces c2's obj
exactly for every loop where the update form does not apply, and that set is
large: any body with an `if` in it, any 1-byte stride, any two same-stride
arrays, any pointer with two distinct displacements.

**2. Block order: PARTLY YES, and the M1/M2 contradiction is resolved.**
§5.1. Source order, with two named exceptions (decision-tree switch arms
reversed; loop-exit-only blocks sunk past the return). For the
`loop_counted` class specifically the rule is complete, because the class
excludes switches.

**3. The pattern set for the body's operators: STILL NO, and it is now the
only "no" left.** WB-D's `wbr_cmp_u` carry idiom is unread and this lane did
not read it either. But the loop grid *narrows* it usefully: within a converted
loop the body words in every one of the 36 cells were plain, one-to-one
lowerings (`add`, `mullw`, `xor`, `sub`, `fadds`, `fmadd`, `extsb`, `divw` with
its two `twi` traps). **The idiom library bites on comparisons producing
values, not on arithmetic**, and a `loop_counted` class whose body predicate
excludes control flow (which §5 already requires, for the update form) is
mostly outside it.

**4. The new requirement WB-D did not list: THE TRIP-COUNT ARITHMETIC.** For a
non-unit step c2 emits a small preheader computation (`addi −1`, then `srwi`
for a power of two or `divwu` for 3, then `addi +1`) and its exact word
sequence is unread (§5, "what is NOT in the class"). This is a *new* unknown
introduced by the class, not one inherited, and it is the reason the honest
first `loop_counted` class should require **step ∈ {+1, −1}**.

**5. WB-D's register rule is still the cheapest part, and it is still last.**
Every converted cell in this grid uses `r11` first, then `r10`, `r9`… exactly as
WB-D §3.4 predicts, with the loop-carried accumulator taking the next free
volatile and `bl __savegprlr_N` appearing only when a call forces it (`a4`,
`d4`). Nothing here contradicts §3.4 and nothing here needed it: **selection →
order → registers** held on all 36 cells.

### 9.1 Predicted reach, restated

**Still `0` on the first scan.** WB-D §9.3/P5.4's argument is untouched by
anything here: 48 of the frontier's 59 functions die at the port's IL reader
before any emitter question is reachable. What this lane changes is the *price*
of the capability, not its yield — and it changes it downward, because §7.7
means the class can be built and verified in three separately-gradeable
increments instead of one.

**Explicitly declined as a class**: loops over ≥2 same-stride arrays (§4.3's
walker selection is unread), loops with any `switch` in the body (the pivot
algorithm is unread), and non-unit steps (item 4 above).

---

## 10. Pre-drafted DISCLOSURE rows

Per `DISCLOSURE.md` step 5 the black-box alternative is preferred, and **for
this lane it is unusually strong**: `grids/wb-loop/loop_grid.cpp` +
`calib.cpp` re-derive §2.2's predicate, §3's guard rule and §4's forms against
real `c2.dll` with **no address at all**, and `/d2QXnobdnz` / `/d2QXnopreinc`
are documented switches of the shipped compiler, not disassembly. **A code lane
that ships the §5 predicate needs no row.**

| # | Kind | What would be adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-LOOP-1** | **route** | **The `mtctr`/`bdnz` conversion is a separate tuple-IR pass that runs innermost-first over the loop tree and refuses whenever CTR is not free through the whole loop.** | **`0x10c0f7f9`** (the per-loop converter), `0x10c0f81e` (the driver, and the `func+0x0c` loop list), `0x10c0d3fe` (the CTR-availability scan), `0x10c09c81` (the per-tuple CTR test), `0x10c24021` (the phase order), `0x10c2ecf8` (the `-QXnobdnz` flag) | *(nothing — this lane adopts no code)* | *(pending)* | **The black-box alternative is complete**: cells `a1`–`a14` + `d4` exhibit the whole predicate, and `/d2QXnobdnz` isolates the pass without any address. Carry this row only if the *opcode numbers* or the *structure offsets* below are copied. |
| **W-LOOP-2** | **route** | **The tuple opcode numbers the converter matches and mints**: `0x2d4` compare, `0x2af` assign, `0x2c6` add, `0x288` the `bdnz` branch, `0xf8` the ctr decrement, `0x54` the CTR register operand, `DAT_10c31008` the CTR pseudo-symbol; and the size table `0x10b18990` whose value must be 4 or 2. | `0x10c0f7f9` (all of them), `0x10b18990` (the size table), `0x10c31008` | *(nothing)* | *(pending)* | **No obj exposes these numbers.** They are c2's internal IR encoding and a port has its own; the row exists because §2's *reading* is defended by quoting them and because `0x10b18990`'s "4 or 2" is the only explanation this lane has for the `a9` 64-bit miss (§7.4). A port that states the clause as "the counter is a 32-bit integer" needs no row. |
| **W-LOOP-3** | **route** | **The `lwzu`/`stwu` update form is a separate, machine-INDEPENDENT per-loop pass**, and the base-difference address form for ≥2 same-stride arrays. | **`0x10b84869`** (the driver, `p2\misc.c`), `0x10b84844` (the per-loop pass), `0x10c16569` + `0x10c15d7f` + `0x10c15b87` (the machine-level peephole), `0x10c2ecfc` (the `-QXnopreinc` flag) | *(nothing)* | *(pending)* | **The black-box alternative is complete and should be used instead**: `/d2QXnopreinc` removes exactly the 28 update forms and nothing else (§7.7), and cells `c1`–`c10` exhibit the base-difference rule. **The selection RULE is NOT claimed** — §4.4 elects no rival and RU-H is unfrozen. |
| **W-LOOP-4** | **navigation, held** | **`-loopopt`'s variable `0x10c2eaf0` has zero readers** — a second dead switch beside WB-D's `-schdat#`. | `0x10b1410c` (the string), `0x10c29860` (the option entry), `0x10c2eaf0` (the variable) | *(nothing)* | — | Not adoptable, recorded so a future lane does not spend a probe on it. |

**Held, not proposed.** The zero-trip guard's *site* inside `lur.c` was not
isolated to a VA — §3 attributes it to rotation by **elimination** (it survives
both `-QX` flags) plus the `-NoLUR` flag's reader at `0x10b84cea`. That is
navigation of the weakest kind and is not offered as a row; the guard **rule**
(§3) is fully black-box and needs none.

**Not claimed.** This lane did not read `lur.c` itself, `globlopt.c`, the
strength reducer that creates the walking pointers, the trip-count arithmetic
selector, the switch pivot algorithm, or the `cgintrin.c` pattern library.
§9 item 3 and §5's "what is NOT in the class" are where a class lowering still
has unread ground.
