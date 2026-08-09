# w-callprice — `expr-call-in-expr-*` priced on the **emitted** column: the body ranking and the emitted ranking disagree by **13×**, **62.5 %** of the emitted column is TU replication, w-jump #2007's own pointer (`op-0x9B`) is the **worst-yielding** key in the family, w-mcall #1963's named next step is **under**-priced 7.6×, and the one rung this lane ran to ground converts **0** because an *address-taken stack local* wears the same `26 <sym>` designator a relocation does

    Tag:       w-callprice
    Slug:      w-callprice
    Date:      2026-08-09
    Fixtures:  none — this is a PRICING lane; it ships no accepted class, no
               recognizer and no `crates/` line, and its four scratch
               instruments are reverted
    Census:    711,514 / 2,463,443 unchanged (28.88 %), **+0**; emitted
               39,200 / 178,977 unchanged, **+0**. TU match **18 → 18**,
               mismatch **0 → 0**. **This lane ships no `crates/` change at
               all** — every number below is a measurement over master
               `c5ff9953`, and the tree it leaves is byte-identical to the tree
               it found outside `docs/` and `work/w-callprice/`.
    Record:    this file; PREREG `work/w-callprice/PREREG.md` committed at
               `26511699` **before the first workload scan** and before the
               first line of the scratch instruments.
    Lane:      w-callprice, worktree branch `wt-w-callprice`. Every scan below
               ran at master **`c5ff9953`** (the w-jump merge).
    Ships:     **no code.** Four scratch instruments (§2.1–§2.4), all reverted,
               quoted as one patch at `work/w-callprice/scratch.patch`. Board
               rows **#2020**–**#2032**; **#2033**–**#2039** left explicitly
               unminted. ROADMAP **§10.26.7**, which this lane owes because its
               own PREREG P12 missed.
    Adopts:    **nothing.** No `DISCLOSURE.md` row is carried. No
               `docs/whitebox/` constant reaches `crates/`, and `crates/` is
               untouched regardless. The one external reading is **c2's own
               `/FAsc` listing** (`work/w-callprice/listing.sh`), which is the
               compiler narrating its output and not a disclosed constant.

---

## 1. The result

> ### **THE BODY RANKING AND THE EMITTED RANKING OF THIS FAMILY DISAGREE, AND THE DISAGREEMENT IS 13×.** Every published ranking of `expr-call-in-expr-*` in `docs/IL_CALL_IN_EXPR.md` — §11, §14.7, §16.7, §17.6, §18.7, §19.7, §22.8, §24.8 — is a **body** ranking. On the emitted column the order is not a permutation of it, it is a different order: body #1 (`recv-load-then-bit-and-and-branch-more`, **102,374** bodies = 24.15 % of the family) yields **41.9** emitted per 1,000 bodies; emitted #1 (`recv-object-then-call-recv-object-more`, 18,912 bodies) yields **296.5**. **Board #2020.**

> ### **AND w-jump #2007's OWN POINTER IS THE WORST-YIELDING KEY IN THE FAMILY.** #2007 sent this lane at `expr-call-in-expr-op-0x9B` — *"46,036 bodies / 1,033 emitted … an order of magnitude more than this whole family"*. Re-derived: **46,036 / 1,033 exactly**, and that is **rank 2 on the body column and rank 8 on the emitted column**, at **22.4** emitted per 1,000 bodies — the **lowest of the top thirty**. Its dominant construct, read from source, is `MEM_OVERLOAD`'s `static void operator delete(void*)` — a free call carrying **two string-literal addresses**, which is `IL_CALL_IN_EXPR.md` §17's `.rdata` pool-relative selection, already declined by name as *"a different and much larger piece of work"*. **Board #2021.**

> ### **62.5 % OF THE EMITTED COLUMN IS TU REPLICATION — AND ON THIS COLUMN THE DISCOUNT RUNS THE OTHER WAY.** 35,576 emitted symbols carry **13,329 distinct mangled names**: 2.67 emitted per construct. w-jump #2000 discounted a **body** column by replication because a body column counts segments. An **emitted** column does not discount: an emitted COMDAT in 419 TUs *is* 419 emitted symbols in the census. What replication does here is **concentrate the work** — so the ranking has to be printed three ways (raw, constructs, leverage) and the three disagree. **Board #2022.**

> ### **THE FIVE HIGHEST-LEVERAGE KEYS IN THE FAMILY ARE *ONE FUNCTION EACH*.** `??1MessageTimer@@QAA@XZ` is **419 emitted in 419 TUs and the ONLY name on its key**. `?Sym@DataArray@@QBA?AVSymbol@@H@Z` is 465 of 482 (3 names). `?SplitMs@Timer@@QAAMXZ` is 434 of 439 (5 names). `??6BinStream@@QAAAAV0@H@Z` heads 496 (8 names). `?MakeString@@YAPBDPBD@Z` is 448 of 457 (10 names). Every one was read back to its dc3 header. **Board #2023.**

> ### **`prod` × EMITTED HAS NEVER BEEN TAKEN, AND IT INVERTS w-mcall #1963.** #1963 split the sequence route's refusal on **bodies**: `call-ref` **125,458 (78.9 %)** against `call-token` **25,060 (15.8 %)**, and named the 25,060 *"the seam's own next step"*. On the emitted column: `call-ref` **5,699**, `call-token` **8,666** — the smaller body row is the **larger** emitted row, at **345.8** emitted per 1,000 bodies against 45.4, a **7.6× yield inversion**. #1963 **under**-priced its own next step. **Board #2024.**

> ### **THE ONE RUNG THIS LANE RAN TO GROUND CONVERTS ZERO, AND IT IS THE FIRST TIME AN `expr-call-in-expr` RUNG HAS BEEN GRADED BEFORE BEING PROPOSED.** R1 — admit a **named-object receiver** in a later statement of the sequence, `gObj.m();` — is thirteen lines and it was **built, run over all 878 TUs and reverted**. Function census **+0**, emitted census **+0**, per-TU verdict set **0 changed**. Of the 2,188 emitted its first-blocker key claims, the shipped locator reaches **at most 33**, and on a manufactured cell where the admission definitely fires the next blocker is `callseq-multiarg-sym` — `seq_call_arg_slots`'s blanket refusal of *any* `SymAddr` slot in a sequence call. **A first-blocker population over-stated the price by three orders of magnitude, and only running it said so.** **Board #2025.**

> ### **AND THE REASON IS A DESIGNATOR AMBIGUITY THAT NO CENSUS KEY DRAWS — READ OFF c2's OWN LISTING.** `CallForm::RecvObject` is documented as *"a named data symbol"*. It is not only that: an **address-taken stack local** wears the identical `26 <sym>` push. `cl /FAsc` on a hand-written `MakeString` reproduction emits **`addi r3,r1,fs$`** — a frame offset, three times — where the same TU's string literal emits **`lis r10,??_C@…` + `addi r3,r10,…`**, a relocation pair. `IlOp::SymAddr` is the second spelling. **`recv-object-*` is 10,144 emitted, 28.5 % of the family by receiver form, and any rung that admits it through `SymAddr` emits a relocation where c2 emits a frame offset — wrong bytes, not a refusal.** **Board #2026.**

> ### **THE RECOMMENDATION IS ONE LANE, AND IT IS NOT THE BIGGEST ROW.** §7 prices four candidates. The recommended one is **R2, the float value tail of a statement-position member-call sequence**: **544 emitted over 9 constructs**, `-whole` on the census's own grammar walk, hand-checked on `Timer::SplitMs`. That is **78× w-mcall's realized 7**, which is why PREREG **P12 missed** and why this lane owes ROADMAP **§10.26.7**. **Board #2032.**

| | value |
|---|---:|
| family at `c5ff9953`, **re-derived** | **423,905 bodies / 35,576 emitted** |
| …share of the blocked **emitted** column (130,560) | **27.25 %** |
| …share of the blocked **body** column (1,751,929) | 24.20 % |
| distinct keys | 476 on bodies, **462 on emitted** |
| keys to cover 50 % of the **emitted** column | **6** |
| distinct mangled **names** over the emitted column | **13,329** (2.67 emitted per construct) |
| **the emitted column that is TU replication** | **62.5 %** |
| `op-0x9B` (#2007's pointer): body rank / emitted rank / yield | **2 / 8 / 22.4 per 1k** — lowest of the top 30 |
| `call-token` on **bodies** / on **emitted** | 25,060 (#3 of the split) / **8,666 (#1 of the split)** |
| **R1, built and run over 878 TUs** | **+0 functions, +0 emitted, 0 TUs changed** |
| `crates/` lines changed by this lane | **0** |
| workspace tests | **1,347 passed, 0 failed, 36 targets** — w-jump's tip, digit for digit |

---

## 2. The instruments — four scratches, all reverted, none committed

`git diff master -- crates/` at the tip is **empty**; the whole patch is recorded
at `work/w-callprice/scratch.patch` (path-scrubbed; `scrub.py` **asserts** no
`/home/` survives) and is in no commit. `work/w-callprice/scratch.diffstat`:
three files, 149 insertions.

### 2.1 The compound key (`crates/c2-harness/src/gap/scan.rs`)

w-jump §2.1's pattern, re-aimed and with **`prod`** added — the axis that
separates *"a construct the port has no production for"* from *"a private limit
inside a production that already ships"* (`census.rs`'s own doc comment), i.e.
ROADMAP §10.26.4's admission-vs-lowering distinction, and the axis that had
never been crossed with the emitted column.

```diff
+// ***** w-callprice SCRATCH INSTRUMENT — REVERTED, NEVER COMMITTED *****
+fn wcp_key(f: &c2_il::func::FnCensus) -> String {
+    let k = f.verdict.key();
+    if !k.starts_with("expr-call-in-expr") {
+        return k;
+    }
+    let hex: String = f.hex.iter().map(|b| format!("{b:02X}")).collect();
+    format!(
+        "{k}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
+        f.cflow, f.dispatch, f.prod, f.calls, f.seg_len, f.hex_mark, hex, f.index,
+        f.name.as_deref().or(f.emit_name.as_deref()).unwrap_or("-")
+    )
+}
```

applied at the two sites that count blockers, exactly as w-jump did.

### 2.2 The sequence route's own refusal (`mcall_tail.rs`)

w-mcall's #1963 split was measured by a scratch and **not shipped**: at
`c5ff9953` the whole 159,068 still reports the generic
`tail-void-body-does-not-end-at-the-call`, because the route re-arms that tag
after a failed attempt (deliberately — `work/w-mcall/PREREG.md` §2.2). This
carries the route's own `Block::ctx` past the re-arm:

```diff
-            if let Ok(shape) = super::calls::parse_call_sequence_from(…) {
-                return Ok(shape);
-            }
+            match super::calls::parse_call_sequence_from(…) {
+                Ok(shape) => return Ok(shape),
+                Err(b) => wcp_seq = Some(…b.ctx…),
+            }
             prod_tag("tail-void-body-does-not-end-at-the-call");
…
         eat_return_plumbing(seg, &mut p, false, depth)
-            .map_err(|_| prod_tag("tail-void-body-does-not-end-at-the-call"))?;
+            .map_err(|_| prod_tag(wcp_seq.unwrap_or("tail-void-body-does-not-end-at-the-call")))?;
```

### 2.3 The second and third order (`calls.rs`)

`call-token` is a *first* blocker of a *first* blocker, which is the defect
w-jump §5.3 was caught by. So every decline inside `eat_member_stmt_call` names
itself (a thread-local on `prod_tag`'s own last-write-wins discipline, **reset
per attempt** — a stale tag is the one failure mode that makes such an
instrument report fiction), and the `0x26` arm is decided by **`mcall`'s own
walk** rather than by a second tokenizer:

```diff
+                Some(&0x26) => {
+                    let f = super::super::mcall::feature(
+                        super::super::mcall::classify(seg, q).aux,
+                    );
+                    if f.contains("-chained") { "msc-recv-0x26-CHAIN" }
+                    else if f.contains("-recv-object") { "msc-recv-0x26-NAMED-OBJECT" }
+                    …
+                }
```

### 2.4 The R1 COUNTERFACTUAL — env-gated, and it is the rung itself

Thirteen lines, behind `C2RS_WCP_R1`, using **`mcall_tail::eat_receiver_object`**
— the one locator the shipped tail form already uses for this designator, so the
counterfactual cannot differ from the rung by a second reading:

```diff
+        if seg.get(q) == Some(&0x26) && std::env::var_os("C2RS_WCP_R1").is_some() {
+            let mut r = q;
+            match super::mcall_tail::eat_receiver_object(seg, &mut r) {
+                None => wcp_msc("R1-not-a-named-object-by-the-tail-locator"),
+                Some(tok) => … {
+                    args.push(vec![IlOp::SymAddr(tok)]);
+                    q = r;
+                    return Some((callee_tok, args));
+                } …
+            }
+            return None;
+        }
```

---

## 3. THE DECOMPOSITION TABLE

`work/w-callprice/rank.py` over `work/w-callprice/scan_inst.jsonl`. **Both
columns sum exactly to the family total, and the script asserts it against the
un-instrumented base scan rather than assuming it** (PREREG P2, §4).

```
family expr-call-in-expr: bodies 423905, emitted 35576
BASE scan (un-instrumented):  bodies 423905, emitted 35576
  ASSERTED: the compound key preserves both columns exactly.
```

### 3.1 By the EMITTED column, with the replication columns

`cons` = distinct mangled names; `lev` = emitted per construct; `em/1k` = emitted
per 1,000 bodies.

| # | emitted | % | cons | lev | TUs | bodies | em/1k | key |
|--:|--:|--:|--:|--:|--:|--:|--:|---|
| 1 | **5,608** | 15.8 | 1,139 | 4.9 | 747 | 18,912 | **296.5** | `recv-object-then-call-recv-object-more` |
| 2 | 4,290 | 12.1 | 1,340 | 3.2 | 711 | **102,374** | 41.9 | `recv-load-then-bit-and-and-branch-more` |
| 3 | 2,865 | 8.1 | 1,158 | 2.5 | 375 | 20,155 | 142.1 | `recv-load-then-intrinsic-call` |
| 4 | 2,183 | 6.1 | 974 | 2.2 | 434 | 19,240 | 113.5 | `recv-load-then-call-recv-load-and-deref-load-more` |
| 5 | 1,498 | 4.2 | 403 | 3.7 | 464 | 18,290 | 81.9 | `recv-load-whole` |
| 6 | 1,463 | 4.1 | **129** | **11.3** | 583 | 4,385 | 333.6 | `recv-object-then-deref-load-more` |
| 7 | 1,040 | 2.9 | 267 | 3.9 | 375 | 22,564 | 46.1 | `recv-load-then-off-add-more` |
| **8** | **1,033** | 2.9 | 647 | 1.6 | 319 | **46,036** | **22.4** | **`op-0x9B`** ← #2007's pointer |
| 12 | 757 | 2.1 | **62** | **12.2** | 297 | 4,912 | 154.1 | `recv-object-then-call-nested-call-and-call-whole2` |
| 16 | 496 | 1.4 | **8** | **62.0** | 290 | 9,708 | 51.1 | `recv-load-then-call-data-addr-1sym-whole` |
| 18 | 482 | 1.4 | **3** | **160.7** | 466 | 2,205 | 218.6 | `chained-then-op-0x64` |
| 19 | 457 | 1.3 | **10** | **45.7** | 453 | 1,477 | 309.4 | `recv-object-then-plumbing-0x3A` |
| 21 | 439 | 1.2 | **5** | **87.8** | 436 | 933 | 470.5 | `recv-load-then-type-real-whole` |
| 24 | 419 | 1.2 | **1** | **419.0** | 419 | 699 | 599.4 | `recv-field-off0-then-call-nested-call-and-type-real-more` |
| | **35,576** | 100 | **13,329** | 2.67 | | **423,905** | 83.9 | **TOTAL** |

**Six keys cover 50 % of the emitted column and twenty-three cover 80 %** — the
emitted column is *far* less shattered than the body column, which is the
opposite of what this lane's PREREG registered (P5, P6, both pessimistic
misses).

### 3.2 The same keys ranked by CONSTRUCTS, and every one of the top five moves

| by constructs | cons | emitted | raw rank | key |
|--:|--:|--:|--:|---|
| 1 | 1,340 | 4,290 | **2** | `recv-load-then-bit-and-and-branch-more` |
| 2 | 1,158 | 2,865 | **3** | `recv-load-then-intrinsic-call` |
| 3 | 1,139 | 5,608 | **1** | `recv-object-then-call-recv-object-more` |
| 4 | 974 | 2,183 | 4 | `recv-load-then-call-recv-load-and-deref-load-more` |
| 5 | 647 | 1,033 | **8** | `op-0x9B` |
| 8 | 403 | 1,498 | **5** | `recv-load-whole` |

PREREG **P16** registered *"at least one key in the emitted top five moves
position"* at p = 0.50. **All five move**, and the two that move furthest are
`op-0x9B` (8 → 5) and `recv-load-whole` (5 → 8) — the discount and the raw
number rank them in opposite orders.

### 3.3 By receiver FORM, which is where `recv-object`'s size shows

| form | emitted | % | bodies | em/1k |
|---|--:|--:|--:|--:|
| `recv-load` (+ `-whole`) | 19,276 | 54.2 | 260,336 | 74.0 |
| **`recv-object`** | **10,144** | **28.5** | 63,049 | **160.9** |
| `recv-intrinsic-this-adjust` (+ `-whole`) | 2,079 | 5.8 | 19,379 | 107.3 |
| `recv-field` / `recv-field-off0` (+ `-whole`) | 1,993 | 5.6 | 18,540 | 107.5 |
| `op-0x9B` | 1,033 | 2.9 | 46,036 | 22.4 |
| `chained` | 795 | 2.2 | 8,949 | 88.8 |
| eleven further forms | 256 | 0.7 | 7,616 | 33.6 |

`recv-object` is **28.5 % of the emitted column at 14.9 % of the bodies** — the
best-yielding large form in the family, and §4.1/§6 are why that matters more
than its size.

---

## 4. What the top keys MEAN — read, not counted

Every key below was located by the instrument, sampled in the **emitted**
column, listed by mangled name, and then read back to its source in the dc3
tree. Three of them were additionally **reproduced from hand-written source**,
which turns "this key probably names that construct" into a measurement.
`work/w-callprice/samples.txt` is the whole set;
`work/w-callprice/probe/keys.cpp` is the reproduction.

### 4.1 Emitted #1 — `recv-object-then-call-recv-object-more`, 5,608 / 1,139 names

Six sampled bodies, six `??$MakeString@…`; the top name is **528 in 528 TUs**.
`src/system/utl/MakeString.h:67`:

```cpp
template <class T1, class T2>
const char *MakeString(const char *c, const T1 &t1, const T2 &t2) {
    FormatString fs(c);
    fs << t1 << t2;
    return fs.Str();
}
```

Windows (`src/Memory_Xbox.cpp` #603 and five others differing only in tokens):

```text
  … 4C 4F 11 53 4F 01 4B 26 38 3A >26< 71 5A 2C A6 43 98 2E 00 99 86 43 B8 2E 00 BD …
```

`26 <method> · 26 <sym> · 2C <TYPE> 00 · 99 <TYPE> 00 · BD` — a member call whose
receiver designates through a **symbol push**. **`work/w-callprice/probe/keys.cpp`
reproduces that skeleton byte-shape from hand-written source** (`FS fs(c); fs <<
t1 << t2; return fs.Str();` → `expr-call-in-expr-recv-object-then-type-ptr-and-op-more`,
same form, its own second blocker), and §6 reads what c2 emits for it.

### 4.2 Emitted #2 — `recv-load-then-bit-and-and-branch-more`, 4,290 / 1,340 names

**The family's largest body row — 102,374 bodies, 24.15 % of it — is a
COMPILER-GENERATED function.** The eight largest names on the key are all
`??_G…`, the scalar deleting destructor:

| emitted | TUs | name |
|--:|--:|---|
| 531 | 531 | `??_GDataArray@@AAAPAXI@Z` |
| 265 | 265 | `??_GObjRef@@UAAPAXI@Z` |
| 169 | 169 | `??_GMessage@@UAAPAXI@Z` |
| 134 | 134 | `??_GObjRefOwner@@UAAPAXI@Z` |
| 75 | 75 | `??_GFilePath@@UAAPAXI@Z` |
| 72 | 72 | `??_G?$ObjPtrVec@VRndTransformable@@VObjectDir@@@@UAAPAXI@Z` |
| 67 | 67 | `??_Gexception@std@@UAAPAXI@Z` |
| 66 | 66 | `??_Glogic_error@stlpmtx_std@@UAAPAXI@Z` |

MSVC's `` `scalar deleting destructor' `` is `this->~T(); if (flags & 1) operator
delete(this); return this;` — **a member call, then a bit test on a flag, then a
branch**, which is the key spelled out. Reproduced exactly from source:

```cpp
void wcp_k2b(O *o) { if (o->GetFlags() & 4) o->Poll(); }
   → expr-call-in-expr-recv-load-then-bit-and-and-branch-more
```

### 4.3 Emitted #3 — `recv-load-then-intrinsic-call`, 2,865 / 1,158 names

STLport `basic_string` and `list` members — `c_str`, `_M_terminate_string`,
`_M_force_construct_null`, `begin`, `end`, `insert` — at 62–66 apiece.
Reproduced exactly:

```cpp
struct Base { virtual ~Base(); };
struct Derived : Base { void Take(Base *); };
void wcp_k3(Derived *d) { d->Take(d); }
   → expr-call-in-expr-recv-load-then-intrinsic-call
```

The second blocker is a **class-layout intrinsic in an ARGUMENT slot**: passing a
`Derived*` where a `Base*` is wanted is `addi rD,rS,k`, the same arithmetic
`RecvField` already carries on the receiver side and which the port has **no
argument slot form for**.

### 4.4 The five one-function keys, each read to its header

| key | emitted / cons | the function, and its source |
|---|--:|---|
| `recv-field-off0-then-call-nested-call-and-type-real-more` | **419 / 1** | `~MessageTimer() { AddTime(mObject, mMessage, mTimer.SplitMs()); }` — `src/system/obj/MessageTimer.h:93`. **419 emitted in 419 TUs and no other name on the key.** A nested member call whose **float** result is an argument: w-value's operand-position class exactly |
| `chained-then-op-0x64` | 482 / 3 | `Symbol DataArray::Sym(int i) const { return Node(i).Sym(this); }` — `src/system/obj/Data.h:393`, ×465. The other two are `ForceSym` (14) and `LiteralSym` (3), the same body |
| `recv-load-then-type-real-whole` | 439 / 5 | `float Timer::SplitMs() { Split(); return Ms(); }` — `src/system/os/Timer.h:137`, ×434. **`-whole`: the census's own grammar walk says granting the real type finishes the body** |
| `recv-load-then-call-data-addr-1sym-whole` | 496 / 8 | `BinStream &operator<<(T rhs) { WriteEndian(&rhs, sizeof(T)); return *this; }` — the `BS_WRITE_OP` macro, `src/system/utl/BinStream.h:93`. All eight names are one macro at eight types |
| `recv-object-then-plumbing-0x3A` | 457 / 10 | `inline const char *MakeString(const char *c) { FormatString fs(c); return fs.Str(); }` — `src/system/utl/MakeString.h:54`, ×448 |

### 4.5 `op-0x9B` — #2007's pointer, read

1,033 emitted over **647** names, top `??3RndAnimatable@@SAXPAX@Z` ×39.
`src/system/utl/MemMgr.h:122`:

```cpp
#define MEM_OVERLOAD(class_name, line_num) …                                    \
    static void operator delete(void *v) { MemFree(v, __FILE__, line_num, #class_name); }
```

**Two string-literal addresses in one call.** `IL_CALL_IN_EXPR.md` §17 (D5)
already measured that shape — *"every one of the 2,730 symbol-carrying plain
calls in a 40-TU sample passes **two** string addresses, and c2 lowers the second
as `addi rD, rAnchor, <difference of their .rdata pool offsets>` … a different
and much larger piece of work than the row was ranked for"* — and the census
carries a `-2sym`/`-3sym+` suffix precisely so this cannot be re-ranked wrong.
**#2007 pointed the next lane at a key whose content was declined by name three
sections earlier**, and the emitted column would have said so on its own: 22.4
per 1,000 bodies, the lowest in the top thirty.

---

## 5. THE `prod` AXIS × THE EMITTED COLUMN — never taken before

`work/w-callprice/prod.py`. The axis **partitions both columns exactly** and the
script asserts it.

| prod tag | emitted | % | cons | em/name | bodies | em/1k |
|---|--:|--:|--:|--:|--:|--:|
| **`tail-argument-not-in-the-operand-vocabulary`** | **8,909** | **25.0** | **4,088** | 2.2 | 91,503 | 97.4 |
| `tail-void-body-does-not-end-at-the-call` (§5.1 splits it) | 14,602 | 41.0 | — | — | 159,068 | 91.8 |
| `prod-not-entered` | 4,093 | 11.5 | 1,828 | 2.2 | 80,998 | 50.5 |
| `framed-result-not-consumed-by-a-literal-post-op` | 2,755 | 7.7 | 1,330 | 2.1 | 32,444 | 84.9 |
| `chain-link-does-not-bind-the-previous-result` | 1,482 | 4.2 | 935 | 1.6 | 5,808 | 255.2 |
| `chain-recv-not-a-plain-b9-load/no-b9-this-adjust` | 847 | 2.4 | 465 | 1.8 | 4,517 | 187.5 |
| `tail-object-receiver-is-not-a-tail-call` | 714 | 2.0 | 371 | 1.9 | **24,862** | **28.7** |
| `chain-link-argument-not-in-the-operand-vocabulary` | 503 | 1.4 | **24** | **21.0** | 2,226 | 226.0 |
| 30 further tags | 1,671 | 4.7 | | | 22,479 | |

**The largest thing on the family's emitted column is the member call's own
ARGUMENT operand vocabulary** — 8,909 emitted over **4,088 distinct functions**,
the *least* replicated large row in the family (2.2 emitted per construct, against
the family's 2.67). It is a **reader** seam; §7 is why it is still not one rung.

### 5.1 w-mcall #1963's split, on the EMITTED column, and it inverts

Of `tail-void-body-does-not-end-at-the-call`'s **159,068 bodies / 14,602
emitted**:

| the sequence route refused with | bodies | share | **emitted** | share | em/1k |
|---|--:|--:|--:|--:|--:|
| `call-ref` — the next statement is not a call at all | **125,458** | **78.9 %** | 5,699 | 39.0 % | **45.4** |
| `call-token` — a receiver the member reader declines | 25,060 | 15.8 % | **8,666** | **59.3 %** | **345.8** |
| `this-undetermined` · `expr` · `result-type` · `formals-marker` · tail | 8,550 | 5.4 % | 237 | 1.6 % | 27.7 |

w-mcall #1963 read *"the seam's own next step is the 25,060; the 125,458 is board
#844's composition seam and not this one."* **The direction was right and the
size was wrong by 7.6× in the conservative direction** — the row it filed at
15.8 % of the bodies is **59.3 %** of the tag on the column that ranks, and the
row it filed at 78.9 % is 39.0 %.

### 5.2 `call-token` at second and third order

Because a first blocker inside a first blocker is exactly what w-jump §5.3 was
caught by, `call-token`'s 8,666 is decomposed rather than quoted:

| clause inside `eat_member_stmt_call` | emitted | cons | bodies | em/1k |
|---|--:|--:|--:|--:|
| receiver is a `26` — **CHAIN** (`mcall`'s own walk) | **5,638** | 1,169 | 19,628 | 287.2 |
| receiver is a `26` — **NAMED OBJECT** | **2,188** | **215** | 4,094 | **534.4** |
| the result is **not discarded** — the value tail | 447 | **13** | 727 | **614.9** |
| receiver is `67` — **virtual dispatch** | 296 | 84 | 296 | 1000.0 |
| six further clauses (`26` in another form 33, `this` not a plain `B9` 24, a `33` field address 19, the argument vocabulary 11, a nested call 9, unnamed 1) | 97 | 95 | 315 | 308.0 |

Both columns sum **exactly** to `call-token`'s 8,666 / 25,060
(`work/w-callprice/tags.py`, `tags_seq3.txt`).

**The value-tail row is one function.** 434 of its 447 emitted are
`?SplitMs@Timer@@QAAMXZ`; the other thirteen names are singletons and two
`vector::at`s. A whole named seam of the sequence route is, on this workload,
**one header inline in 434 TUs**.

---

## 6. R1, BUILT AND RUN — and the designator ambiguity it exposed

### 6.1 The grade

`C2RS_WCP_R1=1`, one 878-TU scan against the base scan, compared as a **map** of
keys and a **set** of per-TU verdicts by name (`work/w-callprice/verdict.py`):

| | base | R1 |
|---|--:|--:|
| TU verdicts | 18 match · 853 vocab-gap · 7 capture-fail | **identical** |
| function census | 711,514 | **711,514 (+0)** |
| emitted in class | 39,200 | **39,200 (+0)** |
| per-TU verdict SET, by name | — | **0 only-in-base, 0 only-in-tip, 0 changed** |
| emitted-blocker key map | 614 keys | **0 appeared, 0 vanished, 6 changed**, every one by −5 to +1 |

**R1 converts zero functions**, measured three independent ways. Its own clause
split says why (`work/w-callprice/tags_r1b.txt`):

| R1's locator said | emitted | cons | bodies | em/1k |
|---|--:|--:|--:|--:|
| not a named object at all — it is a **chain** | 6,541 | 1,319 | 21,319 | 306.8 |
| the argument is not in the operand vocabulary | 809 | 64 | 1,570 | 515.3 |
| the result is not discarded — the value tail | 493 | 17 | 850 | 580.0 |
| **admitted** (the residual of the `0x26` arm's 7,869) | **26** | | 26 | |

**So of the 2,188 emitted that `mcall`'s own walk files as a named-object
receiver, the locator the shipped tail form uses admits at most 33** — the
`msc-recv-0x26-NAMED-OBJECT` row moves **2,188 → 2,155** with R1 on — **and not
one of them converts.** The six key changes are `call-token-0x26` −5,
`call-ref-0x53` +1, `call-token-0xB9` +1, `op-0x9B` +1, `expr-intrinsic-0xDF` +1,
`callseq-multiarg-sym:eof` +1. That is the whole effect of the rung on the
workload.

**On a manufactured cell, where the admission definitely fires, the next blocker
is named** — `work/w-callprice/probe/r1.cpp`, four bodies, `0/4` in class with
R1 off and `0/4` with it on, and the `prod` axis moves
`msc-recv-0x26-NAMED-OBJECT ×4` → **`callseq-multiarg-sym ×4`**. That is not a
reader refusal at all:

```rust
// WR1: a data symbol's address inside a **framed** sequence call. The
// `lis`/`addi` pair would have to be scheduled against the callee-saved copies
// of a frame, and every capture behind `sym_addr_tail_call` is a leaf tail call.
SlotArg::SymAddr(_) => return Err(Block::refuse(seg, off, "callseq-multiarg-sym")),
```

### 6.2 And the emitter's blanket refusal is LOAD-BEARING — c2's own listing

`work/w-callprice/listing.sh` runs `cl /FAsc` on `probe/keys.cpp` — the listing
seam, the compiler narrating its own output. For the hand-written `MakeString`
reproduction, whose census key is `expr-call-in-expr-recv-object-…`:

```text
; 27   : const char *wcp_MakeString(const char *c, const T1 &t1, const T2 &t2) {
  0001c  38610050   addi   r3,r1,fs$
  00024  48000001   bl     ??0FS@@QAA@PBD@Z
  0002c  38610050   addi   r3,r1,fs$
  00030  48000001   bl     ??6FS@@QAAAAV0@PBD@Z
  0003c  38610050   addi   r3,r1,fs$
  00040  48000001   bl     ?Str@FS@@QAAPBDXZ
```

against the same TU's genuine data-symbol address:

```text
; 34   :     return wcp_MakeString("%s%d", a, b);
  00004  3d400000   lis    r10,??_C@_04HGGBINEM@?$CFs?$CFd?$AA@
  0000c  386a0000   addi   r3,r10,??_C@_04HGGBINEM@?$CFs?$CFd?$AA@
```

**The same `26 <sym>` designator lowers two different ways** — `addi rD,r1,<frame
slot>` for an address-taken local, `lis`+`addi` with a relocation pair for a data
symbol — and **`mcall`'s `RecvObject` / `DataAddr` names do not draw that line**.
`IlOp::SymAddr` is the second spelling only. `recv-object-*` is **10,144 emitted,
28.5 % of the family**, and the dominant witnesses of it (`MakeString`'s
`FormatString fs`, `BinStream::operator<<`'s `&rhs`) are **stack objects**. Any
rung that admits `recv-object` through `SymAddr` without first drawing that line
emits a relocation where c2 emits a frame offset: **wrong bytes, not a refusal**
— `docs/GAPS.md` §6's two-facts-one-field, on the largest-yielding form in the
family. `seq_call_arg_slots`'s blanket refusal is what has been holding this,
and it should be **kept** until the split has a census key.

---

## 7. THE PRICED, RANKED LIST

Every population below is an **emitted** count with its construct count beside
it (PREREG D2), and every one carries the calibration §6 supplies: **a
first-blocker population is not a price.** R1's was 2,188 emitted and its price
was 0.

### R1 — a NAMED-OBJECT receiver in a later sequence statement. **DECLINED, 0.**

Population as a first blocker **2,188 emitted / 215 constructs**; reached by the
shipped locator **at most 33**; **converted 0** (§6.1). What it actually needs is
not a reader rung: it is the `SymAddr`-in-a-framed-sequence lowering, and before
that the frame-offset/relocation split of §6.2, which has **no census key and no
capture**. **Do not take it. Do not re-price it off the 2,188.**

### R2 — the FLOAT VALUE TAIL of a statement-position member-call sequence. **RECOMMENDED.**

* **Population 544 emitted over 9 constructs**: `recv-load-then-type-real-whole`
  **439 / 5** and `chained-then-type-real-whole` **105 / 4**. On the `prod` axis
  the same population is `msc-result-not-discarded-value-tail` **447 / 13**.
* **The census's own grammar walk calls it `-whole`** — granting the second
  blocker finishes the body. That is a materially stronger signal than R1's
  first-blocker count, and it is the reason this is the recommendation rather
  than the biggest row.
* **Hand-checked**: `float Timer::SplitMs() { Split(); return Ms(); }`
  (`src/system/os/Timer.h:137`), **434 emitted in 434 TUs**. Read off c2's own
  listing for the reproduction `float wcp_value_tail(O *o) { o->Poll(); return
  o->Level(); }` — `bl Poll · bl Level · addi r1,r1,96`, with `EXTRN _fltused`
  declared: **two `bl`s and an FP return, and nothing else in the body**.
* **In port terms: a READER ADMISSION plus one named gate.** `BodyShape::CallSeq`
  already lowers the statement half (w-mcall, `crates/c2-core` unmoved);
  `SeqTail::CallValue` already exists for the free-function spelling. What is
  new is the **member** value tail and `CallRet::discarded`'s `_fltused`
  obligation (`docs/GAPS.md` §6 instance #14) on the *returned* rather than the
  discarded side. No `IlOp` variant, so PREREG **D1** is not touched.
* **Risk, stated**: the class is nine constructs, so a single unmodelled detail
  in `Timer::SplitMs` takes 434 of the 544 with it. It is a **one-function
  class with a 434× multiplier**, which is the best and the most brittle shape
  on this board at once.

### R3 — the member call's ARGUMENT OPERAND VOCABULARY. **LARGEST, AND NOT ONE RUNG.**

* **Population 8,909 emitted over 4,088 constructs** (`prod
  tail-argument-not-in-the-operand-vocabulary`), 25.0 % of the family's emitted
  column and the **least replicated** large row in it — this one really is four
  thousand distinct functions.
* It decomposes by the key's own second blocker, and **three constructs carry
  68 % of it**: `-then-intrinsic-call` **2,865 / 1,158** (a class-layout base
  adjust in an argument slot — hand-checked, §4.3), `-then-call-recv-load-and-deref-load-more`
  **2,183 / 974** (a nested member call plus a memory read), `-then-off-add-more`
  **1,040 / 267** (a field offset).
* **In port terms, split**: the first is the cheapest — `addi rD,rS,k` is
  arithmetic `RecvField` already carries, and it needs an **argument slot form**,
  not a new `IlOp` call variant. The second is w-value's operand-position class
  and is squarely **D1** — a call as an operand, which the emitter has no
  representation for and which this lane refuses to propose.
* **Take the first sub-row or none of it.** Quoting 8,909 as one rung's
  population would be this lane's own headline defect one column over.

### R4 — the CHAINED receiver in a later sequence statement. **NOT NOW.**

**5,638 emitted / 1,169 constructs / 19,628 bodies** (§5.2) — the second-largest
single seam on the emitted column and a reader route that already exists at the
tail (`mcall_chain`). It is placed below R2 and R3 for one measured reason: it is
**R1's sibling**, reached through the same `eat_member_stmt_call` arm, and R1 is
the datum that says this arm's first-blocker counts do not survive contact with
the emitter. **Price it by building it, as R1 was, before proposing it.**

### 7.1 The recommendation

> **One lane, and it is R2 — 544 emitted over 9 constructs, `-whole`,
> hand-checked on `Timer::SplitMs`.** That is **78× w-mcall's realized 7** and
> **0.42 %** of the whole blocked emitted column, which is what a real rung on
> this board looks like.
>
> **What #2007 should say instead**: the lever is not `op-0x9B` — that key is
> rank 8 on the column that ranks, at the family's lowest yield, and its content
> is the two-string-address pool-relative selection §17 already declined. The
> family's levers are the **argument operand vocabulary** (8,909 / 4,088), the
> **chained sequence receiver** (5,638 / 1,169) and the **FP value tail**
> (544 / 9), in that order by size and the reverse by confidence.

---

## 8. Neutrality

No `crates/` change is shipped, so neutrality is **by construction**. Measured
anyway, master `c5ff9953` against the tip:

* family **423,905 / 35,576** at both (the base scan and the instrumented scan
  agree, asserted).
* Function census **711,514 (+0)**, emitted **39,200 (+0)**, TU match **18 → 18**,
  mismatch **0 → 0**, vocab-gap **853**, capture-fail **7**.
* `cargo test --workspace --release` **1,347 passed, 0 failed, 36 targets** —
  w-jump's published tip, digit for digit.
* `git diff master -- crates/` is **empty**.

---

## 9. Gate

| lane | result |
|---|---|
| `cargo test --workspace --release` | **1,347 passed, 0 failed, 36 targets**, and `git diff master -- crates/` is empty |
| `scripts/board_audit.sh` | 0 cited-but-rowless, 0 unresolved anchors, 0 duplicates, 0 rows-behind-the-prose |
| `cargo test -p c2-harness --release --test rung_registry` | 2 passed |
| 878-TU workload scan | match 18 · mismatch 0 · census 711,514 · family 423,905 / 35,576 |
| the R1 counterfactual, over 878 TUs | census **+0**, emitted **+0**, per-TU verdict set **0 changed** |
| fixtures | **none authored** |

`scripts/gate.sh` is not re-run: this lane changes no code, no fixture and no
registry row, so every lane's corpus and verdict is byte-identical to the one
w-jump's merge gated at `c5ff9953`. Stating that is the honest alternative to
re-running a 5,616-verdict gate to prove a doc commit changed nothing.

---

## 10. PREREG scored

| # | prediction | p | outcome |
|---|---|--:|---|
| **P1** | family re-derives to **exactly** 423,905 / 35,576 | 0.75 | **HIT**, to the unit — w-value's 423,925 / 35,583 less w-mcall's −20 / −7, and eight intervening lanes moved it by zero |
| **P2** | both columns sum exactly, **asserted** | 0.95 | **HIT** — asserted against the un-instrumented base scan, not against itself |
| **P3** | the largest emitted key is `recv-load-whole` at 1,498 ± 5 | 0.70 | **HALF** — the number is **1,498 exactly** and the rank is **5, not 1**. The quantity was inherited and right; the claim about it was wrong |
| **P4** | `op-0x9B` is second at 1,033 ± 20 | 0.55 | **HALF** — **1,033 exactly**, rank **8**, and its yield is the lowest in the top thirty. §1's second headline |
| **P5** | the top three emitted keys are **< 15 %** | 0.60 | **MISS** — **35.9 %**. Registered pessimistic |
| **P6** | **≥ 40** keys to cover 50 % of the emitted column | 0.50 | **MISS** — **six**. Registered pessimistic |
| **P7** | ≥ 1 top-3 key has names < 60 % of its emitted count | 0.55 | **HIT** — all three: 20.3 %, 31.2 %, 40.4 % |
| **P8** | no single name is ≥ 5 % of the family's emitted column | 0.60 | **HIT** — largest is `??_GDataArray@@AAAPAXI@Z` at **1.49 %** |
| **P9** | `prod-call-ref` is the largest prod tag on emitted | 0.70 | **MISS** — third. `tail-argument-not-in-the-operand-vocabulary` 8,909 and `call-token` 8,666 are both larger |
| **P10** | `call-token` is **< 500** emitted; the body column over-prices it ≥ 10× | 0.60 | **MISS, and in the opposite direction** — **8,666 emitted**, and the body column **under**-prices it 7.6× on yield. Registered pessimistic; §5.1 is worth more than the prediction was |
| **P11** | ≥ 2 of the top-3 need a lowering, not an admission | 0.55 | **HIT** — all three (§4.1–§4.3) |
| **P12** | the recommendation is a DECLINE; no rung converts ≥ 100 emitted at reader-admission cost | 0.60 | **MISS** — R2 is **544 emitted over 9 constructs** and is a reader route plus one named gate. Registered pessimistic. **This is what owes §10.26.7**, exactly as the PREREG said it would |
| **P13** | ≥ 1 top key is different constructs wearing one key | 0.75 | **HIT, twice** — the family's largest body row is a **compiler-generated** `??_G` destructor, and `recv-object` wears **both** a relocation and a frame offset (§6.2), which is the sharper one and was proven off c2's own listing |
| **P14** | no `crates/` change committed; tests digit-for-digit | 0.90 | **HIT** — 1,347 / 0 / 36 |
| **P15** | ≥ 1 unnamed refusal, pre-armed on KEY CARDINALITY or EMITTED ATTRIBUTION | 0.70 | **HALF** — neither pre-armed place fired (187 MB and **5.0 s** per scan; every emitted sample's window was anchored on the member call and every one read). §10.1 |
| **P16** | the discount moves ≥ 1 key in the emitted top five | 0.50 | **HIT** — **all five move**, and the two extremes swap: `op-0x9B` 8 → 5, `recv-load-whole` 5 → 8 |

**8 hits, 3 halves, 5 misses** — and the direction is the finding.

### 10.1 Every registered-PESSIMISTIC prediction missed, and that is the lane's own lesson

`work/w-callprice/PREREG.md` §2.1 registered **P5, P6, P10 and P12** as
pessimistic and said why: *"Seven blocked-key size rankings in a row have turned
out to be artifacts … This lane assumes it is the eighth until the measurement
says otherwise."* **All four missed, all in the optimistic direction.**

Board **#770**'s streak is a record of *optimistic* predictions missing. This is
its mirror, and it is the first time on this board: **the "assume the key is an
artifact" prior is now itself a source of error.** `expr-call-in-expr-*` is not
an artifact on the column that matters — it is 27.25 % of the blocked emitted
column, six keys cover half of it, and its named next step is 7.6× *larger* than
the record said. What was an artifact was the **column** everything had been
ranked on, which is a different failure and needed a different instrument.

### 10.2 The unnamed refusal, and it was not where it was aimed

P15 pre-armed on the compound key blowing up the scan and on the emitted
column's window being mis-anchored. Neither fired.

**What fired is §6.1.** R1's population as a first blocker is **2,188 emitted**;
the shipped locator reaches **at most 33**; the conversion is **0**. Three orders of
magnitude, and *nothing short of building it* would have said so — not the key,
not the `prod` tag, not the second-order clause split this lane built
specifically to avoid that error. w-jump §5.3 named "a first blocker inside a
first blocker"; this is the same lesson at the **third** order, where the
instrument that was supposed to fix it still over-stated by 66×. Reported as a
miss of the budget rather than absorbed — **w-park's streak goes to 10/14.**

---

## 11. What this lane deliberately did NOT do

* **It shipped no code.** Four instruments, all scratch, all reverted; one of
  them (R1) is a complete rung that was built purely to be graded and thrown
  away.
* **It did not take R3's 8,909 as one rung.** That number is a `prod` tag, not a
  construct; §7's R3 says take its first sub-row or none of it.
* **It did not propose an `IlOp::Call` variant.** PREREG **D1**, inherited from
  w-mcall, and §4.4/§7-R3 name the two populations that would want one
  (`MessageTimer::~MessageTimer`'s nested-call argument, and
  `-then-call-recv-load-and-deref-load-more`) as **lowering** work with their own
  cost rather than smuggling them in.
* **It did not correct `IL_CALL_IN_EXPR.md` §3.1 in place.** §6.2's designator
  finding is stated here and as board **#2026**; that document's §3.1 is a dated
  record of what D1 measured, and #1127's lesson is that a rung's claims about
  its own artifacts get checked, not that older records get rewritten.
* **It did not re-run `gate.sh`.** §9 says why.
* **It did not quote a single body count as a rung's population.** PREREG D2.

---

## 12. Reproduction

```sh
<mainrepo>/scripts/configure_existing_worktree.sh .
cargo build --release -p c2-harness

# the base, un-instrumented — the family total and the EMITTED ranking
C2RS_DC3=<sib>/dc3-decomp sh work/w-callprice/scan.sh scan_base
python3 work/w-callprice/fam.py work/w-callprice/scan_base.jsonl --top 30

# the instruments (§2), applied, scanned, REVERTED
git apply work/w-callprice/scratch.patch        # then `git checkout -- crates/`
sh work/w-callprice/scan.sh scan_inst
python3 work/w-callprice/decomp.py work/w-callprice/scan_inst.jsonl \
        --base work/w-callprice/scan_base.jsonl
python3 work/w-callprice/rank.py   work/w-callprice/scan_inst.jsonl
python3 work/w-callprice/prod.py   work/w-callprice/scan_inst.jsonl

# §5.1 and §5.2 — the sequence route's own refusal, at three orders
sh work/w-callprice/scan.sh scan_seq3
python3 work/w-callprice/prod.py work/w-callprice/scan_seq3.jsonl
python3 work/w-callprice/prod.py work/w-callprice/scan_seq3.jsonl \
        --tag msc-result-not-discarded-value-tail

# §6 — R1 BUILT AND GRADED
C2RS_WCP_R1=1 sh work/w-callprice/scan.sh scan_r1b
python3 work/w-callprice/verdict.py work/w-callprice/scan_base.jsonl \
        work/w-callprice/scan_r1b.jsonl
C2RS_WCP_R1=1 ./target/release/c2rs census work/w-callprice/probe/r1.cpp

# §4 and §6.2 — the bodies, read rather than counted
sh work/w-callprice/samples.sh
./target/release/c2rs census work/w-callprice/probe/keys.cpp
sh work/w-callprice/listing.sh work/w-callprice/probe/keys.cpp   # cl /FAsc
```

**Committed**: `PREREG.md`, every `.py` and `.sh`, `scratch.patch`, the two
probes' `.cpp`, and the path-scrubbed `.txt` analyses (`scrub.py` **asserts** no
`/home/` survives). **NOT committed**: the six `--jsonl` scans (65–187 MB each),
the probe `.obj`/`.cod`, and the scratch patch as a `crates/` change.
