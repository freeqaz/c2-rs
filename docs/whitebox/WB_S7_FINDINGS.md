# WB_S7 — stage S7 read whole: the splicer nobody priced is **unreached on this workload and one `__try` from live**, and the partition that found it counted 4 of 11 order authors

**Lane `w-s7`, 2026-08-28.** Characterization lane under
[`DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) § Decision 21 §2.
Prereg frozen at `92e199e7f` in [`WB_S7_PREREG.md`](WB_S7_PREREG.md), **before
the image was opened**; graded in §7 (**4 HIT · 2 MISS · 1 PARTIAL · 1 NOT ESTABLISHED**, after §4.3
corrected this document's own P3; the misses are still two of the lane's best
results, and the **corrected** prediction is the best one).

Instrument: [`scripts/f0_pipeline.py`](scripts/f0_pipeline.py) — reused, with
two subcommands added to it rather than a second enumerator built beside it —
plus [`scripts/dump_tuple_splice.py`](scripts/dump_tuple_splice.py), extended.
`--verify` first. Pinned image `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` —
**checked, MATCH**, so every address below is quotable. Flat export dated
2026-08-04.

Board **#3737**–**#3742**.

---

## 0. The one-paragraph answer

**S7 is two stages wearing one address.** `FUN_10b7e032` @ `0x10b7e032` opens
with `test DWORD PTR [eax+0x20],0x1000` at **`0x10b7e03a`**, over the **symbol**
record reached through `func+0`, and that one bit selects between two complete
unwind-lowering routes. **Four of S7's ten passes — 1,462 B of 2,489, 59 % —
are behind it, including BOTH of the stage's tuple splicers.**

**The splicer `0x10b35c78` is code this project has never executed — and it is
one `__try` away from executing.** Its only two callers are `0x10b7e032` and
`0x10c21b03`, and `0x10c21b03`'s only caller is `0x10b7e032` — *both inside the
gate*. The bit is measured **clear on 2,946 of 2,946 functions across 384
fixtures** at the workload's own mode, by a measurement this repo already owns:
`w-restim`'s `sched0` site `0x10b7e00c` sits inside the block that `0x10b7dfea`
skips when the bit is **set**, and `sched0` fired on every function.

> ⛔ **§4.3 IS A CORRECTION TO THIS DOCUMENT'S FIRST DRAFT AND YOU SHOULD READ IT
> BEFORE QUOTING ANY NUMBER IN THIS SECTION.** The draft said the settling cell
> could not be compiled *"because no `wibo` is present on this box"*. **That was
> false — `command -v` is a probe over `PATH`, not over the toolchain, and the
> sibling build `../wibo/build/release/wibo` is what `Toolchain::locate()`
> actually resolves and what this lane's own gate run used.** The cell was
> compiled: **the bit is set by `__try`, it is per-function, and C++ `try/catch`
> at `/EHsc` does not set it.** So *"unreached by the measured corpus"* stands
> and *"unreachable in the configurations this project compiles"* is **refuted**
> — by this lane, against itself. §4.4 states exactly which claims survive.

> **So `w-f0price`'s *"2 of the 4 confirmed splicers sit in stages F0 prices at
> 1 lane and 0 lanes"* is right about the counting and INCOMPLETE about the
> conclusion for one of the two: on the dc3 workload as written today
> `0x10b35c78` is an unreached branch, and on the first TU that uses the
> `SEH_TRY` macro dc3 already ships it is an unpriced cost.** Which raises the
> harder question the enumeration could not ask — *which passes does this
> project's corpus actually run, and what turns the rest on* — and this lane
> answers both for S7: **five of ten, 1,027 B of 2,489, and the switch is
> `__try`.**

**And the partition that produced "4 confirmed splicers" is the larger error.**
Read whole from the image, the splice band is `0x10bd3815`..`0x10bd3901` and
holds **eight** primitives; two more splice **inline**, `0x10bd5516` with **401
direct call sites**. `--splice`'s reach test names **4 of 11** authors and
cannot see function-pointer edges at all — and the pointer form is the *dominant*
one for the two biggest primitives (`0x10bd3824`: 207 calls, **506**
address-takes). Re-partitioned: **A = 14, bracket 14–31 of 34**, against
`--splice`'s 4–22. **Nine of its twelve *"cannot reorder"* rows can.**

---

## 1. What S7 does — `FUN_10b7e032` @ `0x10b7e032`, 225 B, read whole

Called once per function from `0x10b7e701` in the orchestrator `0x10b7e6af`
(that site is already adopted as the `after0` tap —
[`DISCLOSURE.md`](DISCLOSURE.md) **W-STAGETAP-3**). `0x10b7e032 + 225 =
0x10b7e113`, the next function entry: the size checks.

```
10b7e032: 56                push esi
10b7e033: 8b f1             mov  esi,ecx              ; the FUNCTION record
10b7e035: 8b 06             mov  eax,[esi]            ; -> the SYMBOL record
10b7e038: 57                push edi
10b7e039: 33 ff             xor  edi,edi              ; edi == 0 for the whole body
10b7e03a: f7 40 20 00 10 00 00   test [eax+0x20],0x1000   ; <-- THE GATE
10b7e041: 74 3d             je   0x10b7e080           ; bit CLEAR -> skip four passes
10b7e043: e8 ..             call 0x10bec297           ; abort poll
10b7e04a: 89 3d ec e2 c2 10 mov  ds:0x10c2e2ec,edi    ; the beacon, cleared
10b7e050: e8 ..             call 0x10c21b03           ; SEH driver
10b7e057: e8 ..             call 0x10be46f0           ; ehexcept walk
10b7e062: 39 3d 20 de c3 10 cmp  ds:0x10c3de20,edi    ; POGO mode
10b7e068: 75 09             jne  0x10b7e073
10b7e06e: e8 ..             call 0x10b3c6e5           ; the merger, edx=0 -> mode 0
10b7e075: e8 ..             call 0x10b35c78           ; THE SPLICER
10b7e080: 83 3d 20 de c3 10 02  cmp ds:0x10c3de20,2
10b7e08b: e8 ..             call 0x10b9d6be           ; POGO instruction count
10b7e09d: e8 ..             call 0x10b36169
10b7e0b5: e8 ..             call 0x10c12099
10b7e0c0: 39 3d 08 e3 c2 10 cmp  ds:0x10c2e308,edi
10b7e0d5: e8 ..             call 0x10b821c3
10b7e0ed: e8 ..             call 0x10c275a7
10b7e105: e8 ..             call 0x10b3421b           ; the emit driver
10b7e112: c3                ret
```

**Two structural notes the decompiler's output hides and the bytes do not.**

1. **`_DAT_10c2e2ec = 0` appears fourteen times and is fourteen identical
   stores.** Ghidra warns *"Globals starting with '_' overlap smaller symbols at
   the same address"*, which invites reading them as differently-typed writes of
   distinct phase ids. They are all `mov DWORD PTR ds:0x10c2e2ec,edi` with
   `edi` zeroed at `0x10b7e039` and never reloaded. `0x10c2e2ec` is the
   **source-line beacon** — `FUN_10b36169` and `FUN_10c21b03` both *set* it from
   `tuple+0x14` while they walk — so S7 clears it around every pass so an ICE
   raised between passes carries no stale line.
2. **The abort poll `0x10bec297` is called six times**, once before each
   unconditional pass. It is not a pass (143 sites image-wide) and
   `f0_pipeline.py` excludes it from every count; noted so the ten do not become
   eleven.

**S7 is not `/Og`-gated at any level** — the tail runs at `/Od` too, which is
what makes it the last thing that touches the tuple list on every path.

### 1.1 The ten passes, with the gate each runs under

`f0_pipeline.py --s7` prints this table.

| # | pass | B | TU | gate | what it is |
|---:|---|---:|---|---|---|
| 1 | `0x10c21b03` | 752 | `vlines.c` | **`sym+0x20 & 0x1000` SET** | the SEH driver — `__try`/`__except` scope stack, `__C_specific_handler`; **calls `0x10b35c78` itself** |
| 2 | `0x10be46f0` | 302 | `ehexcept.c` | **SET** | the walk over the EH-state pseudo-ops `0x2e1/0x2e9/0x2f0/0x2f1/0x2f4/0x2f6/0x308` |
| 3 | `0x10b3c6e5` | 260 | `fg.c` | **SET** and `DAT_10c3de20 == 0` | the block merger, **mode 0** |
| 4 | **`0x10b35c78`** | 86 | `factor.c` | **SET** | **the splicer** (§2) |
| 5 | `0x10b9d6be` | 62 | `p2symtab.c` | `DAT_10c3de20 == 2` (POGO) | `CNT:\tDynamic instruction count: %I64d\n` |
| 6 | `0x10b36169` | 149 | `fg.c` | unconditional | every `0x2e8` tuple → `0x10c1dc58`, the **late jump-table expansion** |
| 7 | `0x10c12099` | 252 | `mdlist.c` | unconditional; body needs `/Og` **and the bit CLEAR** | mints a `0x284` and **inserts it via the `0x10bd3824` pointer**, then deletes the tuple it replaced |
| 8 | `0x10b821c3` | 77 | `misc.c` | `DAT_10c2e308` | records the emitted range on the symbol |
| 9 | `0x10c275a7` | 167 | `lowersmd.c` | unconditional | mints a `0x2eb` **via the `0x10bd3815` pointer** at the first `0x1b` |
| 10 | `0x10b3421b` | 382 | `dag.c` | unconditional | **the emit driver** → `0x10b338f5`, the emit walk |

> **Every one of the ten is named by `FUN_` name in
> [`ref/P_BLOCKORDER.md`](ref/P_BLOCKORDER.md) §3, written 2026-08-23**, and
> seven of the ten carry `cover=none` in `FUNCS.tsv`. See §6.

### 1.2 The gate is a route selector, and both routes end in `.pdata`

The bit does not merely *add* work. `FUN_10b3421b`, the emit driver, branches on
the same test:

```c
if ((*(uint *)(*param_1 + 0x20) & 0x1000) == 0) {          /* 0x10b3424e */
    if ((param_1[0x25] & 0x1500000U) == 0)  FUN_10c222ac(param_1);
    else                                    FUN_10be31b7(param_1);
} else {
    FUN_10be46f0((int)param_1);
}
```

and the unwind-record emitter `FUN_10c217fd` has exactly two callers:
`0x10c21b03` (the **set** route, inside S7) and `0x10c21fd2`, which is reached
only from `0x10c222ac` / `0x10c222c4` (the **clear** route, from
`0x10b3421b`). So:

| `sym+0x20 & 0x1000` | who lowers the function's unwind data |
|---|---|
| **clear** | `0x10b3421b` → `0x10c222ac` → `0x10c21fd2` → `0x10c217fd` |
| **set** | S7's gated block: `0x10c21b03` → `0x10c217fd`, **plus** `0x10b3421b` → `0x10be46f0` |

This reconciles `WB_EH_FINDINGS.md` §96 / [`ref/P_EH.md`](ref/P_EH.md), which
labels `0x10c21b03` *"the SEH `.pdata` driver … and the path a non-EH function
takes"* — the *"non-EH"* half of that label is the `puVar9 == 0` arm **within**
the set route, not the route an ordinary function takes. Nothing was wrong;
the two pages simply never met.

### 1.3 What the bit is — and the honest limit

**`func+0` is the function's SYMBOL record**, not a second function record.
Corroboration, three ways: `FUN_10b82338` (the per-function option-word decoder,
`0x10b82338`) writes `*(uint *)(DAT_10c40214 + 0x20) |= 0x2000` at
**`0x10b823ff`**, i.e. the same `+0x20` flag word on the global
`DAT_10c40214`; `0x10b7d413` and `0x10b9e9a2` both reach the same word as
`ds:0x10c2e2f4` → `[·]` → `[·+0x20]`, and `0x10c2e2f4` is the current-function
global; and `FUN_10b9c655` allocates records of this shape whose `+0x4` is a
name (`0x10b663c5` stores `"DummyGlobal"`).

**What the bit MEANS is NOT established by this lane, and the reason is worth
recording: `c2` never sets it.** There is no `or`/`and` writing `0x1000` to any
`+0x20` anywhere in the image — 40+ sites *test* the bit and zero sites *set*
it. It arrives from the IL. Locating the read site would have cost more than the
prereg's §3 budget of 12 bodies allowed, and the reachability question (§4) is
answerable without it. **Named, not resolved** — and named as this lane's
largest open item.

> ⚠ **Do not carry forward `WB_F0PRICE_FINDINGS.md` §4.1's annotation
> `/* the EH gate — /EHsc */`.** It is not `/EHsc`: the workload compiles at
> `/O1 /EHsc /GR` and the bit is clear on 2,946 of 2,946 of its functions
> (§4). It is per-function and it comes from the IL. Registered as **P1** and
> scored **HIT**.

---

## 2. `FUN_10b35c78` @ `0x10b35c78` — 86 B, `factor.c`, read whole

`P_BLOCKORDER.md` §6 open #2 and `WB_BLOCKORDER_FINDINGS.md` §6 both leave this
function unread and standing as R8's candidate for whether a decision tree's arm
order is a block **move** or a leaf **materialization**. It is:

```c
void __fastcall FUN_10b35c78(int param_1)
{
  end   = *(int **)((*(int **)(param_1 + 8))[1] + 0x20);        /* list end */
  t     = (int *)**(undefined4 **)(**(int **)(param_1 + 8) + 0x1c);
  while (cur = t, cur != end) {
    t = (int *)*cur;                       /* next = tuple+0 */
    if ((char)cur[2] == '\x1b') {          /* tuple+8 kind == 0x1b: a LABEL */
      p = (int *)cur[0xb];                 /* tuple+0x2c: a pending chain */
      while (p != 0) {
        nxt = (int *)*p;
        FUN_10bd3852(p);                   /* UNLINK  */
        FUN_10bd3815(cur, p);              /* INSERT AFTER the label */
        p = nxt;
      }
      cur[0xb] = 0;
    }
  }
}
```

**It is a genuine move, not a materialization: unlink then insert-after, on
tuples that are already in the list.** One linear pass over the whole tuple
list; for every kind-`0x1b` label tuple it drains the chain hanging off
`tuple+0x2c` and re-inserts each element immediately after the label, in chain
order, then clears the head.

That answers `P_BLOCKORDER.md` §6 open #2 **as a read** — and it does **not**
answer open #1, because §4 shows the function does not run on the population
open #1 is about. Registered as **P2**: predicted *unlink only, cannot author a
new position*. **MISS** — it does both, and the miss is the useful direction.

`0x10c21b03` calls it a second time, from its own prologue
(`FUN_10c21075(param_1); FUN_10b35c78(...)`), so **inside the gated block the
splicer runs twice per function.** `0x10c21b03` is itself an order author: at
`0x10c21d1f`-ish it walks a label run with
`FUN_10bd38b0(piVar3, piVar12)` — unlink + **insert before** — moving `0x1b`
tuples whose symbol kind char at `sym+0x31` is not `' '`.

---

## 3. The order-author set — the partition `w-f0price` §4.2 rests on names 4 of 11

### 3.1 The band is eight, and the two published sets disagree with each other

Read whole from the pinned image, `0x10bd3815`..`0x10bd3901` is contiguous and
holds eight functions. Sizes are entry-to-entry and each body was read.

| VA | B | shape | direct call sites | address-takes | `--splice`? | `dump_tuple_splice.py`? |
|---|---:|---|---:|---:|:--:|:--:|
| `0x10bd3815` | 15 | INSERT AFTER `(at,new)` | 131 | **201** | — | ✔ |
| `0x10bd3824` | 17 | INSERT BEFORE `(at,new)` | 207 | **506** | — | ✔ |
| `0x10bd3835` | 29 | SPLICE CHAIN AFTER | 90 | **77** | — | ✔ |
| `0x10bd3852` | 31 | UNLINK `(t)` | 88 | 0 | ✔ | ✔ |
| **`0x10bd3871`** | **33** | **UNLINK RANGE `(a,b)`** | 1 | 0 | **—** | **—** |
| `0x10bd3892` | 30 | MOVE AFTER = unlink + ins-after | 18 | 0 | ✔ | — |
| `0x10bd38b0` | 32 | MOVE BEFORE = unlink + ins-before | 35 | 0 | ✔ | — |
| `0x10bd38d0` | 50 | MOVE RANGE `(a,b,c)` | 8 | 0 | — | ✔ |

*(call-site and address-take counts are `dump_tuple_splice.py`'s, parsed from
the image — `E8 rel32` **sites**, not distinct callers. Ghidra's distinct-caller
figures are lower and both are correct on their own population; the 401 in §3.2
is a site count and matches board **#3463** exactly.)*

Plus the scheduler's bulk relink `0x10be626c` (278 B, `P_DAG.md`), which
`--splice` counts and R8's script does not, for eleven authors in total once
§3.2's pair is added.

> **`--splice` names 4 of the 11; `dump_tuple_splice.py` names 5; the union of
> the two published sets is 8, and `0x10bd3871`, `0x10bd5516`, `0x10bd5577`
> are named by neither.** Both scripts are in this repository, both are correct
> about what they enumerate, and **neither says what it omits.** This is
> `#3151`'s disease in the instruments rather than the pages.

### 3.2 The premise `--splice` flags as unverified is FALSE

`f0_pipeline.py`'s own docstring, verbatim: *"group C's 'reaches none' is sound
only under the premise that no pass rewires `tuple+0` / `tuple+0x10` inline.
This script does not verify that premise."* It is refuted at the byte level by
**`FUN_10bd5516` @ `0x10bd5516`, 67 B, `tuple.c` — 401 direct call sites**:

```
10bd5545: 8b 41 10    mov eax,[ecx+0x10]     ; prev
10bd5548: 8b 11       mov edx,[ecx]          ; next
10bd554a: 89 10       mov [eax],edx          ; prev->next = next
10bd554c: 8b 01       mov eax,[ecx]
10bd554e: 8b 51 10    mov edx,[ecx+0x10]
10bd5551: 89 50 10    mov [eax+0x10],edx     ; next->prev = prev
10bd5554: e9 cf fc ff ff   jmp 0x10bd5228    ; tail-jump to the freelist
```

**It never calls `0x10bd3852`.** It is a leaner unlink than the primitive — no
null checks, no zeroing of the removed tuple's links — because it is the
delete-from-a-sentinelled-list form. `FUN_10bd55fa`, `FUN_10bd5577` and
`FUN_10bd5611` all funnel into it.

And `FUN_10bd5577` @ `0x10bd5577` (131 B) writes an **inline insert-before**,
the same four-store algebra as `0x10bd3824`, without calling it:

```c
new->prev = at->prev;  at->prev = new;  new->next = at;  new->prev->next = new;
```

> ⛔ **`0x10bd5516` was already named in this tree.** Board **#3463**
> (`w-tailread`, 2026-08-23) calls it *"the unlink, the exact inverse of
> `P_EXPAND.md` §2's mint primitive `0x10bd3824` — and **401 direct callers**
> against the mint's 207"*, and [`ref/P_EXPAND.md`](ref/P_EXPAND.md):339 repeats
> it. **`f0_pipeline.py` was written four days later, flagged exactly this
> premise as unverified, and the refutation was one board row away.**
> Sixth instance of the family the brief names, and this time the missing fact
> was not merely *findable* — it was **numbered**.

### 3.3 Re-partitioned — 14–31 of 34, and nine rows change verdict

`f0_pipeline.py --authors`, same 34 passes, same 12-hop reach, author set
extended and function-pointer edges from `xrefs.tsv` admitted:

| | A (authors directly) | B (transitive) | C (neither) | bracket |
|---|---:|---:|---:|---|
| `--splice` (`w-f0price` §4.2) | 4 · 1,674 B | 18 · 4,068 B | 12 · 4,972 B | **4–22 of 34** |
| **`--authors` (this lane)** | **14 · 6,630 B** | **17 · 2,914 B** | **3 · 1,170 B** | **14–31 of 34** |

**Nine of `--splice`'s twelve *"cannot reorder"* rows can**: `0x10b85f52`
(2,037 B, `misc.c` — calls `0x10bd5516`), `0x10be46f0`, `0x10be460f`,
`0x10c04d6d`, `0x10c12099` move to **A**; `0x10c182b4` (the peephole),
`0x10b39e59`, `0x10bb3256`, `0x10c2226b` move to **B**.

**The sensitivity table is printed with the result** so the jump is not one
undifferentiated correction:

```
      A =  4 of 34   --splice's 4 (calls only)
      A =  5 of 34   the 8-primitive band, calls only
      A =  6 of 34   + the scheduler relink
      A = 10 of 34   + function-pointer edges
      A = 14 of 34   + the two INLINE splicers
```

Two of the three corrections are each worth about four rows; the band's extra
primitives are worth one. **The largest single term is the one no call graph
can see**: `0x10bd3824` and `0x10bd3815` are handed as *pointers* to shared
tuple builders 506 and 201 times, against 207 and 131 direct calls — R8 read
this and wrote it down (*"the DIRECTION OF A SPLICE is a runtime parameter"*),
and the partition built next door did not consume it.

> **Group C is still not a proof, and this lane will not claim it is.** The
> three survivors (`0x10c113f3`, `0x10bd1068`, `0x10b821c3`) are "reaches no
> *known* author". This lane found its two inline splicers by **reading**
> `0x10be46f0`'s callees, not by a pattern scan over the image, so the
> population of inline splicers is **open** — narrower than `--splice`'s
> premise, and still an assumption.

---

## 4. Reachability — the brief's third deliverable

**`0x10b35c78` is reachable only through the `0x1000` gate.** Its callers are
`0x10b7e032` and `0x10c21b03`; `0x10c21b03`'s sole caller is `0x10b7e032`; both
call sites are inside the block `0x10b7e041` skips when the bit is clear. There
is no third route — `f0_pipeline.py --s7` prints the caller sets.

**The bit is measured clear on the whole of this project's largest corpus**, and
the measurement is one this repository already owned:

* `sched0`'s tap site is `0x10b7e00c`. The gate at `0x10b7dfe3`
  (`test [eax+0x20],0x1000`) branches `jne 0x10b7e017` at `0x10b7dfea` —
  **`0x10b7e00c < 0x10b7e017`, so the site is inside the skipped range.**
  `ref/P_DAG.md`:64 already records the test and *"taken ⇒ skip"*.
* `rungs/2026-08-21-restim.md` §3.5 reports **2,946 `sched0`→`after0` pairs over
  384 fixtures**, and `sched3`→`sched0` at **2,946 (100 %)**. `after0` is the
  unconditional call site of S7 itself, so `after0` counts every function
  processed. **`sched0` fired on all of them ⇒ the bit was clear on all of
  them.**

> **`0x10b35c78` executed zero times in 2,946 function compilations.** So did
> `0x10c21b03`, `0x10be46f0`, and S7's mode-0 merger run.

| | passes | B | share of S7's 2,489 B |
|---|---:|---:|---:|
| **reached** on `/O1 /EHsc /GR`, no POGO | **5** | **1,027** | **41 %** |
| unreached (4 behind the bit, 1 behind POGO) | 5 | 1,462 | 59 % |

**This is a statement about the corpus, not a proof about the image.** 384
fixtures is a large but bounded population, and no fixture in it uses
`__try`/`__except`. The honest claim is *"unreached by everything this project
has ever compiled and measured"*. The cheap way to break it is one
`__try`-bearing cell — **and §4.3 compiles it.**

### 4.3 THE CELL — the bit IS settable at the workload's own profile, and it is `__try`

> ⛔ **THIS SECTION CORRECTS THIS LANE'S OWN FIRST DRAFT.** That draft said
> *"which this lane did not compile, because no `wibo` is present on this box
> (`command -v wibo` → not found; the compilers are)"*. **The premise was
> false and the conclusion drawn from it was the wrong kind of claim.**
> `CLAUDE.md` § "Project context" states the resolution order — *"wibo from
> `PATH` or a sibling `../wibo` build"* — and `Toolchain::locate()`
> (`crates/c2-reference/src/lib.rs`:303) implements exactly that: `C2RS_WIBO`,
> else `<root>/../wibo/build/release/wibo`, else `PATH`. Measured in this
> worktree:
>
> ```
> command -v wibo            -> (not on PATH)
> readlink -f ../wibo        -> <milohax>/wibo   (the sibling checkout)
> ls -la ../wibo/build/release/wibo  -> -rwxr-xr-x 5204048 Aug  5 01:18
> ```
>
> `.claude/worktrees/wibo` is a **symlink to the sibling checkout**, placed so
> every worktree's `../wibo` resolves. **`command -v` is a probe over `PATH`;
> it is not a probe over "is the toolchain available", and this lane
> substituted the first for the second while writing §6, which corrects
> `w-f0price` for the identical substitution.** Two further signals were in
> this lane's own output and were printed without being read: the gate reported
> **7,038 fixture-verdicts, sweep 19,460/19,556, cross 90,424/90,812**, none of
> which exists without the seam; and `grep -c "SKIP: toolchain absent"` over
> the test log returned **0**. Seventh instance of the family, and the only one
> where the lane's own transcript already contained the refutation.

Three cells, all through the existing `c2rs stage counts` tap at **`/O1 /Oi
/EHsc /GS- /c` — the workload's own profile** (`docs/whitebox/grids/w-s7/`, tracked). The
discriminator is registered by the mechanism and not chosen after the fact:
`after0` is S7's unconditional call site, `sched0` sits inside the range
`0x10b7dfea` skips when the bit is **set**, so **`after0 − sched0` is the count
of functions with the bit set.**

| cell | functions | `sched0` | `after0` | bit set on |
|---|---:|---:|---:|---:|
| `s7_ctl.cpp` — three ordinary functions, **control** | 3 | **3** | 3 | **0** |
| `s7_seh.cpp` — `ctl_a` verbatim + `__try/__except` + `__try/__finally` | 3 | **1** | 3 | **2** |
| `s7_cxx.cpp` — `ctl_a` verbatim + C++ `try/catch` at `/EHsc` | 2 | **2** | 2 | **0** |

**Three results, and the second and third are the ones that matter.**

1. **`sym+0x20` bit 12 is set by `__try`.** Two `__try` functions, two bits set.
2. **It is per-function, not per-compiland.** `ctl_a` is byte-identical source in
   the control and in the SEH cell, and it still reaches `sched0` in a TU where
   two other functions do not. This is what the control was for.
3. **It is NOT C++ EH and NOT `/EHsc`.** The `try/catch` cell compiles at
   `/EHsc`, lowers real C++ EH, and leaves the bit **clear on both functions**.
   So the struck annotation in `WB_F0PRICE_FINDINGS.md` §4.1 was wrong about
   more than the flag name: the construct is **SEH**, a different feature.

**Obj-side, closing the loop from the read to the object:** `s7_seh.obj`
contains `__C_specific_handler`, `.pdata` and `.xdata`; `s7_ctl.obj` contains
**none of the three**. That is precisely what `0x10c21b03`'s body predicts
(`FUN_10c05869("__C_specific_handler")`, symbol kind `'S'` = `0x53`), so the
gated block is not merely entered — its output reaches the obj.

> **So the brief's question — *"is `0x10b35c78` reachable in the configurations
> this project compiles"* — is answered YES, and this lane's first draft
> answered it NO on an unchecked premise.** `0x10b35c78`, `0x10c21b03`,
> `0x10be46f0` and S7's mode-0 merger run all execute on any TU containing a
> `__try`, at the workload's own flags.

### 4.4 What survives, and what it is now a statement about

The corpus claim is unchanged and was independently re-measured: **`sched0` ==
`after0` on the control**, and `w-restim`'s 2,946/2,946 stands. What narrows is
the *scope* of the conclusion.

| claim | status after §4.3 |
|---|---|
| the bit is clear on all 2,946 measured functions | **stands** |
| S7 is 5 of 10 passes / 1,027 B reached **on that corpus** | **stands** |
| `0x10b35c78` executed zero times in those 2,946 compilations | **stands** |
| *"unreached in the configurations this project compiles"* | **REFUTED by §4.3** — it is reached by any `__try`, at the workload's flags |
| *"41 % live"* as a property of the stage | **REFUTED as a property**; it is a property of a corpus with no SEH in it |

**And the dc3 workload is one `#include` away from the other side.** Measured:

* `dc3-decomp/src/macros.h`:10 defines **`#define SEH_TRY __try`** (with
  `SEH_EXCEPT`, `SEH_FINALLY`), so the workload's own header already names the
  construct — and `grep -rlw SEH_TRY` over `src/` and `native/` returns
  **only the definition**: zero users today.
* Of the **4,213** `.obj` files in `dc3-decomp/build`, exactly **one** contains
  `__C_specific_handler` — `xdk/XBOXKRNL/xboxkrnl.obj`, an XDK import object
  (machine `0x01f2`, archive long-name members), **not** one of the 878 c2-compiled
  TUs and not in the workload list.

> **So the correct statement is narrow and it is contingent: S7's second half is
> dead on the dc3 workload *as it is written today*, and the first TU that uses
> the `SEH_TRY` macro dc3 already ships turns 1,462 B of S7 — including both of
> its tuple splicers — live.** That is a materially different fact from
> "unreached", and it is the one a port has to plan for.

### 4.1 What still authors order in the reached half

Not nothing, and two of the three are rows `--splice` calls C or B:

* **`0x10c12099`** (252 B, `mdlist.c`, `cover=none`, **`--splice` group C**) —
  runs when `/Og` and the bit is **clear**, i.e. exactly this project's mode.
  For every `0x1f`/`0x21` real-instruction tuple whose label chain reaches a
  `0x30f`-anchored `0x284`, it either rewrites the opcode `0x21 → 0x27` or
  **mints a `0x284` through `FUN_10bd75ff(0x284, 0x8000, 0, 0, FUN_10bd3824,
  piVar3)`** — the inserter passed *by pointer* — and then **deletes** the
  original with `FUN_10bd5516`. It is a return/tail-merge transform, and it
  both inserts and removes.
* **`0x10c275a7`** (167 B, `lowersmd.c`, `cover=none`, `--splice` group B) —
  mints a `0x2eb` via `FUN_10bd7652(…, FUN_10bd3815, iVar5)` at the function's
  first `0x1b` label, then stamps `-1` into `+0x24` of every operand in the
  range. Unconditional.
* **`0x10b36169`** (149 B, `fg.c`) — walks for `0x2e8` tuples and calls
  `0x10c1dc58`, the **late jump-table expansion**, which itself calls
  `0x10bd3824` *and* `0x10bd3852` *and* takes `0x10bd3824`'s address. This is
  the pass that turns `P_BLOCKORDER.md` §4's `0x2e8` into a table.

### 4.2 The freeze point is `0x10b338f5`, not `0x10b3421b`

Registered as **P7** (*"order is frozen at `FUN_10b3421b`'s entry"*) and
**REFUTED.** `0x10b3421b` itself splices nothing, but four of its depth-1
callees do, all before `FUN_10b338f5` runs:

| callee | authors |
|---|---|
| `0x10b33dd8` | `0x10bd3824` insert-before, `0x10bd5516` delete |
| `0x10b33647` | `0x10bd3815` insert-after, `0x10bd5516` delete |
| `0x10b33f96` (gated `func+0x94 & 0x20000000`) | `0x10bd5516` |
| `0x10c1da6f` (the deferred table emit) | `0x10bd5516` |

**`FUN_10b338f5` @ `0x10b338f5` reaches no author at all** — consistent with
`P_BLOCKORDER.md` §1's independent read that the walk *"follows `tuple+0` and
does nothing else"*. So the emitted order is fixed at the **emit walk's** entry,
one level below where P7 put it, and the last four things that can change it are
named above.

---

## 5. Bearing on F0 — reported, not re-priced

`w-f0price`'s **F0 ≥ 10 raw sub-lanes + 2 UNPRICED terms** stands and this lane
does not restate or replace it. Three bearings, in the direction each actually
points:

1. **One of F0's two "unpriced splicers in unpriced stages" is not a cost at
   all on this project's configurations.** `0x10b35c78` is 86 B of unreached
   code. Pricing a port against it would be paying for a branch the corpus
   never takes. *(Direction: down, by a small and specific amount. The other
   splicer, `0x10b3668d` in S3, is untouched here.)*
2. **The `4–22 of 34` bracket it published is superseded by `14–31 of 34`**, and
   the floor more than tripled. F0 is denominated in order; the set of passes
   that can change order is the denominator; **the denominator was measured with
   an author set naming 4 of 11.** *(Direction: up, and this term is much larger
   than term 1.)*
3. **The stage-level question F0's enumeration cannot express is "reached by
   what".** S7 is 41 % live on this corpus. Nothing in `--stages`, `--splice`,
   `WB_ITEMF` §6.1 or `STEP5_PRICING` §3 distinguishes a pass from a pass this
   project has never executed, and the first two of those are this repo's own
   instruments.

**Net: this lane does not publish a new F0 number and explicitly declines to.**
The two directions do not net to an arithmetic and pretending otherwise would
repeat the `8`-vs-`4` mistake `w-f0price` spent its whole §1 on.

---

## 6. `cover=none` again measured the index, not the image

`WB_F0PRICE_FINDINGS.md` §4 and §6.2 gloss the column as *"27 of 34 are
`cover=none` — **no document in this repository mentions them at all**"*.

**All ten of S7's passes are named, by `FUN_` name, in `ref/P_BLOCKORDER.md`
§3**, written 2026-08-23 — four days before `w-f0price` ran, and `w-f0price`
§4.1 **cites that very section** for the ten-callee list. Seven of the ten carry
`cover=none`.

Measured over the whole 34:

| | n |
|---|---:|
| `cover=none` | 27 |
| of those, mentioned in some repo `.md` (excluding this lane's prereg) | **17** |
| of those, mentioned in a page **other than** `WB_F0PRICE_FINDINGS.md` itself | **8** |

`build_funcs.py` computes `cover` as `paged > labelled > cited > none`, where
`cited` needs an `ADDR.tsv` row and `ADDR.tsv` is built by scanning for bare hex
addresses. `P_BLOCKORDER.md` §3 writes `FUN_10be46f0`, not `0x10be46f0`. **The
column is a statement about a scanner's regex.** Registered as **P8**, **HIT**,
and it is the family's sixth instance.

*(The count that survives unchanged is the raw one: 27 of 34 are not on a page
and not hand-labelled. That is a real coverage gap and `w-f0price` was right to
raise it. Only the gloss is wrong.)*

---

## 7. The prereg, scored — 3 HIT · 3 MISS · 1 PARTIAL · 1 NOT ESTABLISHED

| # | prediction | p | result |
|---|---|---:|---|
| **P1** | the `0x1000` bit is not a `/EHsc` mirror but a per-function/per-compiland flag c2 writes | 0.65 | **PARTIAL, and §4.3 settles all three clauses.** *Not `/EHsc`* — **HIT** twice: clear on 2,946/2,946 at `/EHsc`, and a C++ `try/catch` cell compiled at `/EHsc` leaves it clear. *Per-function* — **HIT**, proven by the `ctl_a` control surviving in the SEH TU. *A flag c2 writes* — **MISS**: no site in the image sets bit `0x1000` at any `+0x20`; it arrives from the IL, and the source construct is **`__try`** |
| **P2** | `0x10b35c78`'s splice is an unlink, so it cannot author a new position | 0.50 | **MISS.** Unlink **and** insert-after — a genuine move, and the answer to `P_BLOCKORDER` §6 open #2 |
| **P3** | the splicer **is** reachable on this project's configurations | 0.70 | **HIT — and this document's first draft scored it MISS on an unchecked premise.** Reachable only behind the gate, and the gate is clear on 2,946 of 2,946 *measured* functions; but §4.3 compiles a `__try` cell at the workload's own flags and the gate opens. **The corpus claim stands; the configuration claim does not.** The prereg's §1.1 registered that a dead splicer was the outcome the lane wanted *less* — which is exactly why this is written up as a self-refutation and not quietly amended: the draft reached the answer it had pre-committed to disliking and then **stopped checking** |
| **P4** | ≥ 3 of the ten are dead in this project's configurations | 0.55 | **HIT — 5 of 10, 59 % of the bytes.** (The first draft of `--s7` scored this by substring-matching `0x1000` on the gate text and marked `0x10c12099` dead, whose gate needs the bit **clear**; caught and replaced with a per-row flag before the number was quoted) |
| **P5** | ≥ 2 distinct live order-authors in S7 at `/O1 /EHsc /GR` | 0.70 | **HIT**, and **not in the form registered** — flagged low-information in prereg §1.2 because it leaned on the mode-0 merger, and the merger turns out unreached. It survives on three passes the prereg never named: `0x10c12099`, `0x10c275a7`, `0x10b36169` (§4.1) |
| **P6** | `w-f0price`'s A/B/C is wrong on ≥ 1 S7 row | 0.40 | **HIT, nine rows over the 34 and one inside S7** — `0x10c12099` is group C and hands the insert-before primitive to a builder by pointer |
| **P7** | order is frozen at `0x10b3421b`'s entry | 0.75 | **MISS.** Four of its callees splice first; the freeze point is `0x10b338f5` (§4.2) |
| **P8** | ≥ 3 `cover=none` rows have a prior repo mention | 0.60 | **HIT — all seven of S7's, and 8 of the 34 excluding `WB_F0PRICE`'s own tables** (§6) |

**P2's miss is the read `P_BLOCKORDER` §6 asked for; P7's miss moves the freeze
point one level and names the four passes between. P3 is the one worth reading
twice**: it turned the brief's question from *"what does this unpriced splicer
cost"* into *"this project has never run it"* — and then, once the cell was
actually compiled, into the sharper *"this project has never run it, and one
`__try` turns it on"*. **A prediction graded MISS on an unchecked premise is
worse than either a HIT or an honest MISS**, because it ships as settled.

### 7.1 The `#3505` check this lane owes

**No conclusion here rests on a ranking.** §1.1's table is in `0x10b7e032`'s own
call order; §3.1's is in address order; §3.3's A/B/C is a partition by a *read
predicate*, and its sensitivity table exists precisely so the reader can see
which predicate term does the work rather than trusting a single number.

**Where an instrument measured itself is §7's P4 footnote** — this lane's own
`--s7` classified liveness by substring-matching the gate *text* it had just
written, which made `0x10c12099` dead by a `0x1000` that meant the opposite.
Sixth of the family, and the second time in two waves that the lane's own tool
supplied the artifact.

### 7.2 Controls, watched failing (`#3336`)

| control | watched fail |
|---|---|
| `--verify` on a non-pinned image | `C2RS_IMAGE=/etc/hostname … --verify` → `*** MISMATCH — addresses NOT quotable ***`, exit 1 |
| every subcommand with the export absent | `C2RS_EXPORT=/nonexistent … --s7` → `SKIP: /nonexistent/calls.tsv absent`, exit 2 |
| `--s7`'s liveness classifier | it **did** fail, on `0x10c12099`, before the number was published (§7 P4) |

---

## 8. What this lane did NOT establish

1. **Which IL field carries `sym+0x20` bit 12.** c2 never sets it (§1.3), and
   §4.3 identifies the *source construct* (`__try`) without locating the *IL
   field* or c2's read of it. The consequence is bounded: the reachability
   question is answered, and what is missing is the mechanism.
2. ~~**Whether any `/O1 /EHsc` source construct sets it.** No `__try`-bearing
   cell was compiled — **`wibo` is not installed on this box**, so the reference
   seam cannot run here at all.~~ — **STRUCK: the premise was false and the cell
   is compiled in §4.3.** `wibo` **is** installed, as the sibling build
   `Toolchain::locate()` resolves; `command -v` probes `PATH`, and `PATH` is not
   the resolution order `CLAUDE.md` documents. **The construct is `__try`**, it
   is per-function, and C++ `try/catch` at `/EHsc` does not set the bit. What
   remains open is narrower: **which IL field `c1xx` writes it from**, which is
   a front-end read and not this lane's.
3. **Whether group C is closed.** Narrower than `--splice`'s premise and still
   an assumption (§3.3's box). Closing it needs a pattern scan for inline
   `tuple+0`/`tuple+0x10` rewiring, which this lane did not build.
4. **The bodies of `0x10c21b03`'s callees, `0x10be46f0`'s callees, or
   `0x10b3421b`'s twenty.** Only the four that splice were resolved (§4.2).
5. **Any wall-clock or lane conversion.** `w-f0price` §6.3 declines it and this
   lane declines it for the same reason.
6. **Nothing in `crates/`.** `git diff master..HEAD -- crates fixtures` is
   empty; **no `DISCLOSURE.md` row is owed**, because this lane adopts no
   constant into the port. Every address here is analysis, not adoption.

## 9. Reproduce

```sh
sha256sum compilers/X360/16.00.11886.00/c2.dll   # must equal the pinned digest

python3 docs/whitebox/scripts/f0_pipeline.py --verify
python3 docs/whitebox/scripts/f0_pipeline.py --s7        # section 1.1, section 4
python3 docs/whitebox/scripts/f0_pipeline.py --authors   # section 3.3
python3 docs/whitebox/scripts/f0_pipeline.py --splice    # w-f0price section 4.2, unchanged

python3 docs/whitebox/scripts/dump_tuple_splice.py \
        compilers/X360/16.00.11886.00/c2.dll             # section 3.1, section 3.2

# section 4.3 — the cell, at the workload's own profile
./target/release/c2rs stage counts --fixtures docs/whitebox/grids/w-s7/s7_ctl.cpp   # sched0=3 after0=3
./target/release/c2rs stage counts --fixtures docs/whitebox/grids/w-s7/s7_seh.cpp   # sched0=1 after0=3
./target/release/c2rs stage counts --fixtures docs/whitebox/grids/w-s7/s7_cxx.cpp   # sched0=2 after0=2
./target/release/c2rs compile docs/whitebox/grids/w-s7/s7_seh.cpp --keep-obj /tmp/seh.obj
strings -a /tmp/seh.obj | grep -E '__C_specific_handler|[.]pdata|[.]xdata'

# the toolchain resolution, since this lane got it wrong once
command -v wibo ; readlink -f ../wibo ; ls -la ../wibo/build/release/wibo

# the controls, watched failing
C2RS_IMAGE=/etc/hostname  python3 docs/whitebox/scripts/f0_pipeline.py --verify ; echo $?
C2RS_EXPORT=/nonexistent  python3 docs/whitebox/scripts/f0_pipeline.py --s7     ; echo $?
```
