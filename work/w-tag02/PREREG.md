# w-tag02 — PRE-REGISTRATION

**Written and committed before any probe source, capture or reader change
exists in this worktree.** Base: `dcc9214` (master), branch `w-tag02`.

The lane's subject is `.in` **element tag `02`** — *the address of another
symbol* — which `crates/c2-il/src/func/ininit.rs` names
`InInitResidue::SymbolAddress` and refuses by design. Board **#931**.

Every claim below is scored in `docs/rungs/2026-08-07-w-tag02.md` §"Prediction
versus outcome" with its outcome written next to it, whether it held or not.

---

## §0 The one thing I most expect to be wrong

The single measured witness this lane starts from is the comment in
`ininit.rs::a_pure_symbol_address_record_is_never_scanned`:

```text
  int* gp = &gi;   ->   <tok> 00 · 02 e3 09 00 04 · 07        (MEASURED)
```

and the three `??_R*` rows in `rungs/2026-08-07-w-rtti.md` §4.2, which all spell
the same five bytes `02 <tok:2> 00 04`. **Four cells, all pointer-width-4, all
offset-zero, all in one direction.** Board **#644**'s shape — *the payload may
not be one contiguous field* — is the named hazard: `00` and `04` are two
separate bytes and nothing so far separates "constant separator + width" from
"offset varint + width" from "width + flags". Every cell in §2's grid exists to
put a non-zero value in one of those two slots.

## §1 The claims

Scored **H** (held) / **R** (refuted) / **U** (unresolved — no cell separated
it). A prereg with no refutable claim is a diary, so each of P1–P8 names the
observation that kills it.

| # | claim | dies if |
|---|---|---|
| **P1** | Tag `02`'s element is exactly `02 <token-var> 00 <n>` — the token in the same 2-or-4-byte form `read_token_var` reads, then a byte, then a byte — on **every** cell of §2's grid | any cell's next element does not start where that layout predicts (checked by the parser cursor landing on `07` or on a legal tag, **not** by eyeballing the hexdump) |
| **P2** | The `00` slot is a **constant separator** and carries no offset. `int* p = &s.b;` with `b` at a non-zero offset spells the same `00` | that cell spells anything but `00` there |
| **P2′** | …and the offset appears **somewhere** — I name the three places in advance, in order of my belief: (a) in the `<n>` slot, (b) as an extra element after the `02`, (c) not at all, because c2 puts it in the `.data` raw bytes as an addend | none of (a)/(b)/(c) — i.e. the offset is in a fourth place, or the cell does not compile to a tag-02 element at all |
| **P3** | `<n>` is the **width in bytes of the pointer slot** and reads `04` on every cell of the grid, because this is a 32-bit target | any cell reads a different value there while still framing |
| **P4** | A **null** pointer initializer (`int* np = 0;`) is element tag **`01`**, not `02` — a scalar zero, no relocation, and the existing reader already handles it | it mints a tag `02` |
| **P5** | A pointer to a **string literal** (`const char* s = "abc";`) mints a tag `02` whose token is the **same token** an `00 03` literal record defines in the same `.in` | the token names nothing the literal reader knows |
| **P6** | An **array of pointers** (`int* ap[2] = {&gi,&gj};`) is **two** tag-`02` elements in **one** record, in declaration order | it is two records, or one element, or the order is reversed |
| **P7** | A **pointer-to-data-member** (`int A::*pm = &A::a;`) does **not** use tag `02` — it is a scalar offset | it mints a tag `02` |
| **P8** | The `struct A{virtual void f();int a;}; A g;` cell mints a tag `02` (the vftable) **followed by** a tag `01` (the `int`) in one record, and the obj carries **one** `IMAGE_REL_PPC_ADDR32` (`0x0002`) into `.data` with no `PAIR` | the reloc type is not `0x0002`, or there is a `PAIR`, or the element order is the other way |

### The instrument claims

| # | claim | dies if |
|---|---|---|
| **P9** | Two **independent** instruments agree on the grammar of every accepted cell: a byte-scanner that finds `02` runs in `.in` without a parser, and the production reader's own cursor. Neither is allowed to be the other's witness | they disagree on any cell's element count or byte span |
| **P10** | `InInitResidue::SymbolAddress` is **non-zero** on the 878-TU workload today, and **falls** after the widening | it is 0 today (the residue would then be a metric with no population, and item 2 of the brief would be unmeasurable as stated) |
| **P11** | The `.gl` binding invariants move on the **arity** axis, not only the totality axis: `elements` rises and `records` rises, and `conflicts` does not | `records` moves and `elements` does not, or `conflicts` moves at all |

### The decline floor — registered NOW, before any probe

| # | floor | |
|---|---|---|
| **P12** | **The reader widening ships only with a differential cell where OLD and NEW disagree and NEW is byte-exact against real `c2`.** A widening that only changes a residue count and reaches no obj is **not** shippable as a widening — it is shippable only as a *reader* whose output nothing consumes, and the rung must say so in those words | |
| **P13** | **The #232 hazard, named in advance and in its exact mechanism here.** `IlBundle::data_tu` accepts an initialized object iff `.in` yields **exactly `size` bytes** for it. If the widened reader returns 4 zero bytes for a tag-`02` element, `data_tu` will accept an object it used to refuse and `emit_data_obj` will emit a `.data` **with no relocation** — a wrong obj, from a refusal, which is the one direction the correctness rule forbids. **Mitigation registered before the code**: the widened reader returns the symbol reference in a *separate* channel, and `data_tu` refuses any object carrying one unless the emit path can place its relocation. A test asserts the refusal *before* the writer learns relocations | |
| **P14** | **Scan controls at both ends, printed, not summarized.** `exact 35839 · differs 3338 · partition-broken 0 · match-tu-differs 0 · census-disagree 0 · mismatch 0 · TU match 10`. Any movement in TU match is a conversion only after a **full** scan at the tip, reported by TU **name** | |

### The predictions I expect to be boring

| # | claim | |
|---|---|---|
| **P15** | TU match stays **10**. The near-miss data TUs are checked by name (brief item 3) and I predict **zero** of them convert, because `data_tu`'s clause 6 (exhaustive accounting) refuses on the *undefined external* the pointer target becomes, independently of tag `02` | |
| **P16** | `factor-c` stays **169** and the writer's section vocabulary stays **10**. Nothing here adds a section name | |

---

## §2 The grid, registered before it is written

One axis per cell, namespace scope, and — where possible — **no function
defined**, so `emit_data_obj`'s class (`data_tu` clause 1) can reach it.

| cell | source | axis |
|---|---|---|
| `t01_ptr_to_global` | `int gi; int* gp = &gi;` | the baseline witness, re-measured |
| `t02_null_ptr` | `int* np = 0;` | P4 — the tag-01 boundary |
| `t03_ptr_to_extern` | `extern int ge; int* gp = &ge;` | target is an **undefined** external |
| `t04_ptr_to_static` | `static int si = 1; int* sp = &si;` | internal linkage target |
| `t05_ptr_to_func` | `void f(); void (*fp)() = &f;` | P8 — a `.text` target |
| `t06_ptr_to_literal` | `const char* s = "abc";` | P5 — the tag-03 boundary |
| `t07_char_array` | `char s[4] = "abc";` | the control: tag 03 **inline**, no reloc |
| `t08_ptr_array` | `int gi,gj; int* ap[2] = {&gi,&gj};` | P6 — arity |
| `t09_struct_offset` | `struct S{int a; int b;} s; int* p = &s.b;` | **P2/P2′ — the offset cell** |
| `t10_array_offset` | `int arr[4]; int* p = &arr[2];` | P2′ again, a second route to a non-zero offset |
| `t11_vfptr` | `struct A{virtual void f();int a;}; A g;` | **#931's own cell** |
| `t12_ptr_to_member` | `struct A{int a;int b;}; int A::*pm=&A::b;` | P7 |
| `t13_mixed_struct` | `int gi; struct S{int a; int* p;} s = {7,&gi};` | the mixed aggregate `ininit.rs` already has a unit test for |
| `t14_const_ptr` | `int gi; int* const cp = &gi;` | `const` → does it move to `.rdata`? (a factor-C boundary, not a tag one) |
| `t15_two_ptrs` | `int gi,gj; int* p1=&gi; int* p2=&gj;` | two records, two relocations, one `.data` |
| `t16_ptr_to_self` | `struct N{N* next;}; N n = {&n};` | a self-reference — the token names the record's **own** object |
| `t17_wide_ptr_deep` | 300 objects before the pointer | forces a **4-byte** token form, so P1's `read_token_var` clause is exercised at both widths |

**Frozen and `sha256`'d and committed BEFORE the first compile**, per the
standing constraint. Any cell added later goes in a **dated addendum** below,
never by editing the table above.

## §3 Method

* Flags: the **workload's own** profile (`/O1 /Oi /EHsc /GR …`) minus include
  paths, exactly as `work/w-rtti/flags_probe.txt` does. `/Ox` is not the
  workload's mode and a capture at it is not comparable to a workload obj.
* One directory for every byte-diffed capture (the `w-ilx` rule).
* The oracle is real `c2.dll` under wibo. Nothing here is graded by a listing;
  `#843` — obj bytes over listing spellings.
* `#918`: anything keyed by a symbol uses the per-record binding, never
  `IlFunction::mangled_name`.

## §4 Board rows

This lane may use **#936–#945** and no others.

---

## Addenda

*(dated, appended, never a rewrite of the above)*
