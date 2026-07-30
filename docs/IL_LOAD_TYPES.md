# IL load types — the type word, the `expr-load-type-*` buckets, and what each load costs

Characterization of the `.ex` TYPE word as it appears under the `B9` LOAD /
`30` indirect-load / `33` literal productions, written against the
`expr-load-type-XXXXXX` census family — collectively the largest group of
blocking features on the real dc3 workload. Extends `IL_TYPE_TAGS.md` (the
scalar table) with: the full field grammar including aggregates, the meaning of
the bucket names, the measured PPC lowering of a load per type, and a ranked
order of work with the estimation basis stated.

Method: controlled probe TUs (scratch sources `p1_scalars.cpp` …
`p11_kinds.cpp`, not tracked), captured with `c2rs census --keep-il` and
compiled with the real toolchain (`cl.exe /Ox /GS- /c` under wibo, plus the
real workload flags where stated); `.text` decoded by hand. Every claim is
labelled MEASURED (a capture shows it) or HYPOTHESIS (inferred, would need a
discriminating capture). All corpus counts are from the 2026-07-30 scan JSONL
(878 TUs / 2,462,571 functions) and a 44-TU IL sample described in §6.

---

## 1. The type-word grammar (MEASURED, with one open bit)

```
TYPE       := <tag> <kind> <payload>
tag        := 0x80 | cv | slot-width | agg-size-bit4
              cv:          +0x20 const, +0x10 volatile
              slot-width:  low 3 bits (after bit0) = 2*(log2(w)+1):
                           0x2→1 B, 0x4→2 B, 0x6→4 B, 0x8→8 B
              bit0:        bit 4 of an aggregate's size field (0 otherwise)
kind       := <width nibble> <class nibble>
              width: the value's own byte width 1/2/4/8 — or, for aggregates,
                     the low 4 bits of the size
              class: 1 signed int   2 unsigned int   3 data pointer
                     4 function/code pointer          5 real (FP)
                     6 aggregate    7 void            A real *literal*
payload    := LEB128 id                                  (scalars, pointers,
                                                          aggregates ≤ 31 B)
            | <stmt-varint size> <LEB128 id>             (aggregates whose
                                                          5-bit size field is 0,
                                                          i.e. size ≥ 32 B)
```

Witnesses for each field, keyed to the probe that pins it:

* **tag cv bits** — `A6 41 84 20` const int / `96 41 86 20` volatile int
  (already in `readers.rs`); `A6 43 …` is the type of `this` in *both* const
  and non-const member functions (probe `p2_member.cpp`: `C0::gc` const and
  `C0::gn` non-const each block at an `A6 43` load). A tag `C6` (bit 0x40) is
  reported in `readers.rs` as occurring; none of these probes produced it, so
  that bit is **not determined** here.
* **tag slot-width is positional, not the type's** — unchanged from
  `IL_TYPE_TAGS.md`; e.g. the `27` member-offset type carries the *pointee's*
  width (`27 82 43 b0 08` for a `bool` member, probe `p10_getters.cpp`).
* **kind = width·class, one rule everywhere** — MEASURED across the whole
  table: `char 82 11`, `short 84 21`, `int 86 41`, `long long 88 81`,
  `unsigned … 12/22/42/82`, pointer `86 43`, `float 86 45`, `double 88 85`,
  void `82 07`; aggregates below. Note `char` and `signed char` share kind
  `11` and differ only in id (`70` vs `10`) — probe `p1_scalars.cpp` separates
  the triple `char`/`signed char`/`unsigned char` as `82 11 70` / `82 11 10` /
  `82 12 20`.
* **kind class 4 = function/code pointer** — probe `p11_kinds.cpp`:
  `int (*)()` literal 0 encodes `33 86 44 8d 20 00`, and a pointer-to-member-
  function result is `41 86 44 84 20`. The recurring real-TU getter
  `30 a6 44 … 2c 86 44 …` (44-TU sample, one shared inline function) is a load
  of a const function-pointer member.
* **id space** — MEASURED: builtins have fixed small ids (`int` 0x74,
  `char` 0x70, `void` 0x03, `float` 0x40, `double` 0x41, …); a **pointer to a
  builtin T is the fixed id `0x400 + id(T)`** (probes `p1`/`p4`: `int*` =
  `f4 08` = 0x474, `char*` `f0 08`, `void*` `83 08` = 0x403, `bool*` `b0 08`,
  `float*` `c0 08`, `double*` `c1 08`, `short*` `91 08`, `long long*` `93 08`
  — eight independent confirmations of the rule); **every other derived type
  (class types, pointers to them, cv-qualified variants, references,
  pointer-to-pointer) is allocated sequentially from 0x1000 in first-use
  order** (probe `p1`: `void**` got 0x1000, `const char*` 0x1002; the fixture
  TU in `IL_TYPE_TAGS.md` gave `const char*` 0x1001 — same mechanism,
  different first-use order). This is what makes the id **TU-dependent** for
  everything interesting, and §3 turns on it.
* **references are pointer-kind types with an explicit deref** — probe
  `p2_member.cpp`: `int& r` loads as `b9 <tok> 86 43 92 20` (a 0x1000-range
  pointer id) followed by `30 86 41 74`; returning a reference (`p11`,
  `k_ref`) is the getter shape *without* the `30` — the address itself is the
  result (`41 86 43 f4 08`).
* **enums are int-kind with a derived id** — probe `p6_misc.cpp`: `Color`
  loads as `86 41 86 20` (kind 41, id 0x1006) and blocks as
  `expr-load-type-864186`. An enum is *not* distinguishable from `int` by kind,
  only by id ≥ 0x1000.
* **`long double` is `double`** — `p6`'s `ldbl_add` parses (and matches) as
  the existing double leaf, `88 85 41`.

### 1a. Aggregates, and the `read_type` defect — CONFIRMED, and now decoded

The task-list defect ("`86 06 20 ec 20` parses as 3 bytes where a bracketing
`41` pins it at 5") is real, and the size ladder in probes
`p7_aggr.cpp`/`p8_sizes.cpp`/`p9_edge.cpp` decodes what the extra bytes are.
Struct-copy bodies (`*d = *s;`) put the aggregate TYPE in the `30`/`32`
positions:

| struct | size | align | TYPE bytes | reading |
|---|---:|---:|---|---|
| S4  | 4  | 4 | `86 46 80 20` | kind 46 = size 4·aggr, LEB id 0x1000 |
| S8  | 8  | 4 | `86 86 86 20` | kind 86 = size 8·aggr, id 0x1006 |
| B12 | 12 | 4 | `86 C6 80 20` | kind C6 = size 12·aggr |
| S15 | 15 | 1 | `82 F6 8C 20` | tag 82 (align 1), kind F6 = size 15 |
| S16 | 16 | 4 | `87 06 93 20` | **tag bit0 set** → size 0x10, kind high 0 |
| S20 | 20 | 4 | `87 46 99 20` | size = 0x10 + 4 = 20 |
| SD16| 16 | 8 | `89 06 95 20` | tag 88·bit0 (align 8, size bit4) |
| S31 | 31 | 1 | `83 F6 80 20` | size = 0x10 + 0xF = 31 |
| S32 | 32 | 1 | `82 06 20 87 20` | size field 0 → **varint size 0x20**, id |
| S33 | 33 | 1 | `82 06 21 8e 20` | varint 0x21 — one byte moved, so it *is* a size |
| T40 | 40 | 4 | `86 06 28 a0 20` | varint 0x28 = 40, id 0x1020 |

So: **an aggregate's size is a 5-bit field spread across tag bit 0 (bit 4) and
the kind's high nibble (bits 3..0); the tag's remaining low bits carry the
alignment class, not the width; and when the size is ≥ 32 the field is 0 and a
statement-varint size is inserted between the kind and the LEB id.** The
S32/S33 pair is the discriminating capture: under the plausible wrong rule
("the trailing bytes are a class token") the byte `20` would not move when the
struct grows by one byte; it does (`20`→`21`).

Consequences for `read_type`:

* it mis-parses exactly the **≥ 32-byte aggregates** — `86 06 28 a0 20` LEB-
  reads as 3 bytes (`86 06 28`) leaving `a0 20` in the stream, the claimed
  desync. The wild witness: `system/meta/Sorting.cpp` carries
  `4c 30 86 06 80 14 10 00 00 a5 29 4b` — varint size 0x1014 (a 4,116-byte
  object) then id, aligned with the surrounding call grammar under the new
  rule and torn under the old one. The task's `86 06 20 ec 20` is a 32-byte
  aggregate with id 0x106C.
* `type_width` also returns `None` for the odd tags `83/87/89`, so ≤ 31-byte
  aggregates whose size crosses bit 4 fail closed rather than mis-parse
  (S16/S20/SD16 above). ≤ 31-byte aggregates with tag bit 0 clear happen to
  parse correctly under the current code (kind treated as an opaque byte +
  LEB id).

## 2. What the bucket names are (and are not)

`expr-load-type-XXXXXX` is the **first three bytes** of the refused TYPE
(`blk_type` packs `seg[p..p+3]`), i.e. `<tag> <kind> <first id byte>` — the id
is *truncated*. For 3-byte types the name is the whole type; for pointers and
every derived type the name keys on **id mod 128** (the first LEB byte), and
the id is TU-dependent (§1). Two measured consequences:

* **A `A643xx`/`8643xx` bucket is not a type.** The same six `std::exception`
  inline members (emitted into nearly every TU by the stlport/libcmt header
  prologue) census as `expr-load-type-A6438B` in `xdk/nuiapi/headtracker.cpp`
  (this-type id 0x100B) and as `expr-load-type-A6438A` in
  `system/meta/Sorting.cpp` (id 0x100A) — one include difference shifts the
  whole family by one bucket. MEASURED by an include-replica probe
  (`p3_dc3hdr.cpp`, headtracker's four includes + one anchor function): it
  reproduces headtracker's first 25 segments bucket-for-bucket, and its `.gl`
  names only the exception-family inlines.
* **The family is the real unit.** Summing the scan JSONL:
  `expr-load-type-A643xx` = **750,421** blocked functions across all 128
  possible id bytes (loads of a *const-qualified data pointer* — overwhelmingly
  `this`), and `expr-load-type-8643xx` = **294,810** (plain data-pointer
  loads). Together ~44% of all blocked functions sit behind a pointer-typed
  load. Individual `xx` buckets merely say which class's id landed on which
  residue in which TU.

### The table for the ten largest (2026-07-30 measurement)

| bucket | blocked fns | meaning | evidence |
|---|---:|---|---|
| `expr-load-type-864540` | 93,189 | **float** (`86 45 40`, fixed id) | MEASURED: probes `b_float`, `wp_float` land in it |
| `expr-load-type-888541` | 79,542 | **double** (`88 85 41`) | MEASURED: `b_double` |
| `expr-load-type-864383` | 53,398 | **`void*`** (`86 43 83 08`, fixed id 0x403) | MEASURED: `p3` witness `b9 … 86 43 83 08`; p1 table |
| `expr-load-type-86439E` | 30,232 | data pointer, id ≡ 0x1E (mod 128) — a **per-TU class pointer** (`86 43 9E 20` = id 0x101E etc.); no single C type | id rule §1; no builtin has id 0x1E |
| `expr-load-type-A6438B` | 27,423 | `this` (const data pointer), id ≡ 0x0B — `std::exception* const` in header-prologue TUs, other classes elsewhere (ClipPlayer.cpp has 69) | MEASURED both ways (`p3` replica; JSONL spread) |
| `expr-load-type-8643A6` | 14,837 | data pointer, id ≡ 0x26 — per-TU | id rule §1 |
| `expr-load-type-A64387` | 13,365 | `this`, id ≡ 0x07 — per-TU | id rule §1 |
| `expr-load-type-8643F0` | 10,492 | **ambiguous between `char*`** (`f0 08`, fixed 0x470 — probe `b_pchar` lands here) **and derived pointers id ≡ 0x70** (`f0 20` = 0x1070 occurs in headtracker) | MEASURED both readings occur |
| `expr-load-type-A643C9` | 7,001 | `this`, id ≡ 0x49 (witness: id 0x1249, Sorting.cpp `b9 7f 2f a6 43 c9 24`) | capture |
| `expr-load-type-A643EC` | 12,350* | `this`, id ≡ 0x6C (witness: id 0x106C, TempoMap.cpp) | capture |

\* count from the 2026-07-30 scan JSONL aggregation; the freshly-measured list
this doc was commissioned against orders the tail slightly differently — the
tail *should* wobble, since it is include-order hash noise (see above).

Nearby literal buckets that decode the same way: `expr-lit-type-821230` =
`bool` literal (23,428 — third largest type bucket overall);
`expr-lit-type-864A40` / `888A41` = float/double literals (kind class A);
`expr-lit-type-820703` = a `void`-typed literal (4,169 — shape not probed
here, **not determined**).

## 3. The PPC lowering of a load, per type (MEASURED)

Probe `p4_loads.cpp` (`T f(T* p){ return *p; }` and `int f(T* p){ return *p; }`,
`/Ox /GS-`), `.text` bytes decoded by hand; probe `p10_getters.cpp` confirms
the identical scheme for member getters (`return h->m;` folds the offset into
the displacement). IL shape in every case:

```
B9 <tok p> <T*-TYPE>  [33 <int> <off> 27 <ptr-TYPE>]  30 <T-TYPE>
[2C 86 41 74 00]      41 <result-TYPE>   <return plumbing>
```

**The widening conversion is a separate IL op (`2C <target> 00`), never folded
into the load** — `int f(char* p)` differs from `char f(char* p)` by exactly
the five bytes `2c 86 41 74 00` (captures at `.ex` 0x0a9f vs 0x0ffd of the p4
bundle). What each pair emits:

| pointee | `T f(T*)` (no 2C) | `int f(T*)` (2C to int) |
|---|---|---|
| `char` / `signed char` | `88630000` lbz r3,0(r3) | `89630000` lbz **r11**; `7d630774` extsb r3,r11 |
| `unsigned char`, `bool` | lbz r3 | lbz r3 (2C emits nothing) |
| `short` | `a0630000` lhz r3 — **not lha** | `a1630000` lhz r11; `7d630734` extsh r3,r11 |
| `unsigned short`, `wchar_t` | lhz r3 | lhz r3 (nothing) |
| `int`, `unsigned`, any data pointer | `80630000` lwz r3 | lwz r3 |
| `long long`, `unsigned long long` | `e8630000` **ld** r3 (one 64-bit GPR; DS-form, offset must be 0 mod 4) | — |
| `float` | `c0230000` lfs **f1** | `c0030000` lfs **f0**; `fda0001e` fctiwz f13,f0; `d9a1fff0` stfd f13,-16(r1); `8061fff4` lwz r3,-12(r1) |
| `double` | `c8230000` lfd f1 | same fctiwz/stfd/lwz tail |

Register rules visible in the bytes: an unextended load targets r3 directly; a
load feeding an extension targets **r11** and the `exts*` produces r3; an FP
load targets **f1** when it is the result and **f0** when a convert consumes it
(consistent with the W13a pool). The FP→int convert spills through the red
zone (negative offsets off r1, no frame). Any TU containing an FP-typed
function carries the `_fltused` undefined external (both probe objs).

Adjacent measured facts that bound the widening steps:

* **Returning a narrow type costs nothing extra; converting to int does.**
  `int f(char a){ return a; }` (probe `p6`) emits `extsb r3,r3` — c2 does
  **not** exploit the ABI's extension of narrow arguments; `unsigned char`
  masks (`clrlwi r3,24`), `float` does the full fctiwz spill. The IL is the
  parameter load + the same `2C 86 41 74 00`.
* **The same 2C is context-dependent** (the `IL_CAST_CONVERT.md` hazard,
  now measured from the load side): in `char a < char b` (probe `p1`) *both*
  operand 2Cs materialize (`extsb`×2 feeding the branchless compare), while
  `IL_TYPE_TAGS.md` §3.2's `short a+b` extends neither input. A blanket
  "2C-to-int over a parameter is free" rule is therefore wrong; "2C-to-int
  emits the §3 extension at each use demanded by the consumer" fits all
  captures but is HYPOTHESIS beyond them.
* **`2C <ptr> 00` over a pointer emits nothing** — `void* f(H* p){return p;}`
  is a bare `blr` (p10, `as_void`), and `void*`→`char*` before pointer arith
  emits nothing (p3 segment 1). Static up/down-casts that *adjust* the address
  never come through `2C` — they are intrinsics 2113/2114/2115 (existing
  captures in `IL_CAST_CONVERT.md`); nothing in these probes contradicts that,
  so admitting `2C ptr→ptr 00` as free is sound *for the decode the port can
  see* but should keep the trailing-`00` gate.
* **Compares diverge by type family** (probe `p1` objs): int compares are the
  known branchless idioms; `long long` uses `cmpd cr6` + `bltlr cr6` + two
  `li`; float/double use `fcmpu cr6` + the same conditional-return tail. So
  the W6 leaf does **not** generalize to 8-byte/FP operands by operand
  substitution — it is a different (branchy) scheme.
* **Pointer arithmetic** (`p + x`, probes `p1`/`p5`): `slwi r11,r4,log2(size)`
  + `add r3,r11,r3`; size 1 is a bare `add`; a 12-byte element is
  `slwi 1; add; slwi 2; add` (strength-reduced ×3·4); constant indices fold
  into the load displacement (`p[3]` → `lwz r3,12(r3)`); variable indices use
  `lwzx`. Pointer equality is the branchless int `subf/cntlzw` idiom.

## 4. Decode-only vs needs-codegen

"Decode-only" = the existing emitter already produces the right bytes once the
parser admits the TYPE; everything else names its exact new requirement.

| widening | verdict | what codegen needs |
|---|---|---|
| data-pointer TYPEs (kind 3, width 4, incl. `A6`/`96` cv and `void*`/`char*`) in the **indirect-load-leaf `30`/`2C`/`41` positions** and the **identity/`return this` position** | **decode-only** | none: the load is the same `lwz`/`mr`/`blr` the int class already emits (p10: `g_pi` = `lwz r3,24(r3)` byte-identical scheme to in-class `g_i`) |
| function-pointer TYPEs (kind 4) in the same positions | decode-only | same `lwz`; only the kind gate widens |
| `bool`/`unsigned char` pointee in the leaf | tiny codegen | one encoder: `lbz` (both with and without the no-op 2C) |
| `char`/`signed char`, `short`(+`ushort`/`wchar_t`) pointee | tiny codegen | `lbz`/`lhz` + `extsb`/`extsh` and the r11-then-r3 register rule; **never `lha`** |
| `long long` pointee | tiny codegen | `ld` encoder + DS-form offset%4 gate |
| `float`/`double` pointee returning same type | small codegen | `lfs`/`lfd` into f1 + the `_fltused` shell effect (machinery exists since W13a) |
| FP pointee converted to int (`2C` to int over FP) | real codegen | fctiwz + red-zone spill sequence; refuse initially |
| narrow types in **general arithmetic** positions | **must keep refusing** | extension placement is (operator × operand × result)-dependent (`IL_TYPE_TAGS.md` §3.2) and mis-emits silently |
| float/double loads in general FP arithmetic (the 864540/888541 buckets proper) | real codegen | everything W13a/b deliberately gates: mandatory `fmadds` contraction, multi-constant scheduling, converts, mixing |
| pointer loads feeding member calls / `99` binds / multi-level derefs (the bulk of `A643xx`) | blocked on W11/W12 | not a load problem at all |

## 5. Expected yield, ranked — with the basis

Basis: a rigid full-body shape matcher (token/type/varint readers plus the §1a
aggregate rule) run over the captured IL of a **44-TU stratified sample**
(every 20th line of the workload list; 128,081 function bodies = 5.20% of the
2,462,571-function corpus; scale factor ×19.2). A body counts only if the
*entire* segment matches load(+offset)(+2C)+result+return-plumbing — i.e. it
would be fully in class, not merely advanced, modulo the caveats below. Sample
counts, with already-in-class int forms excluded:

| shape | sample | corpus est. |
|---|---:|---:|
| deref/getter leaf, **pointer-valued** (`T* f(){return m_p;}`), incl. `2C ptr→ptr` strip | 3,881 | ~74,000 |
| **identity / `return this` / `return p;`**, pointer-typed | 2,293 | ~44,000 |
| getter leaf, `bool`/`u8` (`lbz`) | 1,601 | ~31,000 |
| getter leaf, `float`/`double` (`lfs`/`lfd`) | 934 | ~18,000 |
| getter leaf, `short`/`ushort` (`lhz`(+`extsh`)) | 191 | ~3,700 |
| widen-param leaves (`int f(char a)` etc., non-pointer) | ~460 | ~8,800 |
| getter leaf, `long long` (`ld`) | 1 | ~20 |

Caveats on the basis, stated rather than hidden: (a) the matcher did not
verify that the base token binds to a formal or `this` — bases that are
locals/globals will still refuse honestly, so these are upper bounds; (b) the
flat per-TU counts inside some rows (42×, one per TU) are shared header
inlines, so the corpus scale-up is *good* for them and conservative for
TU-local code; (c) `/O1`-vs-`/Ox` flag effects on the obj shell are already
handled machinery (R3) but untested for these classes.

Against that, the measured cautions from this project's own history: intrinsic
2117's 149,200-function bucket yielded **32**; W13b's float-leaf rung yielded
**+1** out of 81,478. The difference here is that the estimate is not a bucket
size — it is a count of bodies that match the *whole* accepted grammar end to
end except for the type gate. The float/double buckets proper (864540/888541,
~173k) remain W13-shaped: their leaf-like sub-population is the ~18k row
above; the rest is FP arithmetic and should be expected to behave like W13b
(near-zero) until real FP codegen exists.

## 6. Order of work

1. **Admit kind-3/kind-4 width-4 TYPEs through the indirect-load leaf and the
   identity shapes** (`30`/`41`/`2C … 00` positions plus the plain
   load-return). Zero new instructions — the emitter path is the existing
   `lwz`/`mr`/`blr`. Estimated **~118k functions fully in class**
   (74k + 44k above; the largest single step available on the board, R2-sized).
   This is also the answer to "smallest widening step with nonzero yield":
   nonzero is *guaranteed* by 6,174 sample witnesses, and the step needs no
   encoder at all.
2. **`lbz` for `bool`/`unsigned char` getters** (+`lbz`+`extsb` for signed
   char, `lhz`(+`extsh`) for 16-bit): three trivial encoders + the r11 rule;
   ~35k estimated.
3. **`lfs`/`lfd` getters returning float/double** (~18k): encoders exist
   (W13a); the work is wiring the leaf shape to the FP result register and the
   `_fltused` shell effect.
4. **`ld` for long long** (~20): do it while touching the leaf, gate offset%4.
5. **Widen-param conversion leaves** (~9k): `extsb`/`extsh`/`clrlwi`
   encoders; keep the FP→int spill form refused until characterized under a
   frame.
6. **Fix `read_type` for aggregates** (§1a) — not for yield (aggregate copies
   need `memcpy`-class codegen) but because the current reader *mis-parses*
   ≥32-byte aggregates instead of failing closed, which is the census-
   corrupting failure mode GAP-1 documents. The rule is now fully specified
   and two-witness confirmed (S32/S33 + Sorting.cpp).
7. Everything else in the family — pointer loads feeding calls/binds, general
   FP bodies, narrow arithmetic — stays refused pending W11/W12/W13 proper.
