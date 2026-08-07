# w-relo — PREREG

    Lane:   w-relo
    Base:   master `cda124c`
    Date:   2026-08-08
    Ships:  **instrument widening only.** New reader in `crates/c2-obj`, one
            lifted plan function in `crates/c2-core/src/comdat.rs` that the
            existing `/Gy` writer is re-pointed at, and the compare in
            `crates/c2-harness/src/gap/fnbytes.rs`. **No codegen change and no
            emitted byte moves.**

Committed **before the reloc compare exists** and before any reloc has been
decoded off a workload obj.

## 0.0 What was read before this file was written — disclosed, not implied

Source only: `docs/FUNCTION_BYTE_MATCH.md`, `crates/c2-harness/src/gap/fnbytes.rs`,
`crates/c2-obj/src/{lib.rs,reloc.rs}`, `crates/c2-core/src/comdat.rs`,
`crates/c2-core/src/coff/{writer.rs,function.rs,reloc.rs}`, the w-seq and
w-fnbyte rungs, board rows #322/#882/#884/#918.

One measurement was taken before this file: the **baseline scan**
(`work/w-relo/base_scan.txt`), to confirm the brief's numbers reproduce. It
reads `fnbyte-exact 35982 · differs 3195 · partial 0 · refused 130573 ·
unbound 9225 · denominator 178975 · exact-relocated 4664`, `FBM 0.20106`,
`match 10 · mismatch 0`. **No relocation record has been decoded.**

---

## 1. The question

`fnbyte-exact` credits a function on its `.text` COMDAT's **raw bytes**. A
`.text` section's raw bytes do not contain its relocations, so two bodies that
branch to two *different* functions are byte-identical here and different in the
obj. `fnbyte-exact-relocated` counts the exposure — **4,664** credited functions
carry at least one relocation the instrument does not check — and lane w-seq
handed over a **compiled reproducer**: GRID-S cell `s12`, where c2 emits
`b ?ext` and the port emits `b ?g`, both encoding `48000000`.

The mission is to make the instrument see that class and to report what it
finds, in the knowledge that **`fnbyte-exact` may shrink**. That is the
instrument-widening exception to the alarm (`FUNCTION_BYTE_MATCH.md` trap 2, as
amended by w-fnbyte's 0 → 4,711) and it is declared here, in advance, rather
than explained afterwards.

---

## 2. RELOC-EQ — the comparison rule, registered before it is implemented

> For one function whose **bytes are already exact**, let
>
> * `R_ref` be its reference `.text` COMDAT's relocation records **in disk
>   order**, each decoded as `(va, ty, target)`, and
> * `R_port` be the port's own relocation plan for the same function **in the
>   order `PortC2::build` writes it**.
>
> The function's relocations **MATCH** iff the two sequences are equal
> element-wise: same length, same `va`, same **full packed 16-bit `ty`**, same
> `target`.

Five decisions, each of which can be wrong and each of which is therefore
written down:

1. **Sequence, not multiset, and no sort on either side.** The reference side is
   c2's own disk order; the port side is the emitter's own order (its stable
   sort by `VirtualAddress` lives inside the plan function, so the plan *is* the
   order the obj gets). Two sets that agree as multisets and disagree in order
   produce different obj bytes, so an order disagreement is a real disagreement
   and is reported as one.
2. **The `ty` word is compared WHOLE, never masked.** `c2-obj::reloc`'s module
   doc is explicit that the high byte carries `NEG`/`BRTAKEN`/`BRNTAKEN`/
   `TOCDEFN` and that comparing a masked base against a constant is the defect
   the `Reloc` type is shaped to prevent. `REL24|BRTAKEN` (`0x0206`) is a
   different relocation from `REL24` (`0x0006`) and this compare says so.
3. **Target identity is by SYMBOL NAME through the symbol table, never by
   index** (#918's shape one level along: indices differ across objs
   legitimately, and the port has no obj here at all — it has names). Where a
   census binding is involved the key is `FnCensus::emit_name`, never
   `IlFunction::mangled_name`; this walk inherits that binding from
   `fnbytes::measure`, which already uses it to decide which row IS which COMDAT.
4. **Three target kinds, as a typed enum and not as a string with a prefix**, so
   a mangled name can never accidentally collide with a rendered class:
   * `Symbol(name)` — an ordinary symbol-table entry, compared by name;
   * `Section(name)` — a **section-definition symbol** (`IMAGE_SYM_CLASS_STATIC`
     carrying an aux record). Its "name" is a *section* name, which is not
     unique in a `/Gy` obj, so it is a distinct variant that can never equal a
     `Symbol`. The port emits no such target, so any reference record carrying
     one is a disagreement — counted and named in its own family rather than
     silently folded into "target differs". This is the registered answer to
     "how are associative / section-relative targets compared": **they are
     compared as an inequality with a printed family**, and the `.pdata`
     association is out of scope entirely because `.pdata` is not a `.text`
     COMDAT and FBM's denominator is `.text` COMDAT leaders.
   * `PairDisplacement(n)` — a `PAIR` record (base type `0x12`), whose
     `SymbolTableIndex` field is **a displacement and not an index** (PE/COFF
     rev 6.0, quoted in `c2-obj::reloc`). Compared as a number. Every PAIR the
     port emits carries 0.
5. **Fail closed, and the failure is a bucket and not a credit.** If the
   reference obj's relocation table does not decode — the `NRELOC_OVFL`
   sentinel, a table running off the end, a `SymbolTableIndex` outside the
   symbol table, an index landing on an aux slot — the read returns `None` for
   the **whole obj** and every byte-exact function in it lands in
   `fnbyte-reloc-unknown`, **which is not credited**. Crediting an ungraded body
   is exactly the blind-instrument defect this lane exists to close.

### 2.1 ONE LOCATOR — the port's plan is lifted, never re-implemented

`crates/c2-core/src/coff/writer.rs`'s `/Gy` branch builds the `.text`
relocation records from `Function::{calls, data_refs}`. A second copy in the
harness could drift from the emitter, and **an alarm that is green about
relocations the port does not emit is worse than the blind one it replaced** —
verbatim the argument board #880 settled for `comdat_function_body`. So the list
construction moves to `c2_core::comdat::text_reloc_plan` and the writer calls
it. The gate is what proves the lift changed no byte.

---

## 3. THE NEW PARTITION — published with distinct keys

| bucket | meaning | credited |
|---|---|---|
| `fnbyte-exact` | bytes identical **and** RELOC-EQ | **yes** |
| **`fnbyte-reloc-differs`** | **NEW** — bytes identical, relocations differ | **no** |
| **`fnbyte-reloc-unknown`** | **NEW** — bytes identical, the reference relocation table did not decode | **no** |
| `fnbyte-differs` | bytes differ (**not** re-graded on relocations) | no |
| `fnbyte-partial` / `-refused` / `-unbound` / `-nobytes` | unchanged | no |

* **The old counts stay derivable.** `fnbyte-exact-bytes` is published as
  `exact + reloc-differs + reloc-unknown` and must read **35,982** — the
  baseline `fnbyte-exact` to the digit. A widening that cannot reproduce the
  number it replaced is not auditable.
* **`fnbyte-exact-relocated` is retired into a graded number**: it stays
  printed, and it is now the denominator of a verdict rather than a caveat with
  a count.
* **The partition identity keeps closing.** Buckets sum to
  `fnbyte-denominator` = 178,975, checked positively on every scan
  (`fnbyte-partition-broken`, known answer 0) — the new buckets are inside the
  same `accounted` walk, so a bucket that stopped being written shrinks the sum
  and the control fires.
* **FBM falls.** `FBM = (exact + whole_tu)/denominator`, so the ratio drops by
  `(reloc-differs + reloc-unknown)/178,975`. **Declared, not hidden.**
* `fnbyte-reloc-differs` is its OWN bucket and is **not** merged into `differs`,
  so the before/after is auditable and the two failure modes (wrong bytes, wrong
  target) never share a work queue.

### 3.1 Trap 0 — the population this control can reach

A green control is a statement about the population it ran over. This one can
reach **exactly the byte-exact functions whose reference obj's relocation table
decodes**. Everything else is printed as a counted residue, never as silence:

| key | what it counts |
|---|---|
| `fnbyte-reloc-graded` | byte-exact functions that got a RELOC-EQ verdict |
| `fnbyte-reloc-unknown` | byte-exact functions whose reference table did not decode |
| `fnbyte-reloc-table-unreadable` | TUs (not functions) whose whole reloc read failed |
| `fnbyte-exact-relocated` | of the credited, how many carry ≥1 reference relocation |

`fnbyte-reloc-graded + fnbyte-reloc-unknown = fnbyte-exact-bytes` is a second
positive identity and gets its own broken-counter.

---

## 4. THE CONTROLS

**C1 — the known-answer test (s12).** The widened instrument **MUST** move
`s12`'s `?f@@YAXXZ` from `exact` to `fnbyte-reloc-differs`, with the witness
naming REL24 at `va 0` and the two targets `?g@@YAXXZ` (port) against
`?ext@@YAXXZ` (reference). A widening that leaves s12 exact has not widened.

**C2 — the inverse control, on the SAME obj.** `?g@@YAXXZ` and `?anchor@@YAXXZ`
in `s12` are byte-exact **and** relocation-exact (each emits one REL24 against
the external it actually calls). They must stay `fnbyte-exact`. A rule that
turns every relocated function red is not an instrument, and running the
positive and the negative through one compiled obj is what separates the two.

**C3 — the five-alarm.** `fnbyte-match-tu-reloc-differs` must be **0**. On a TU
the differential graded `match`, the whole obj is byte-identical to c2's, so
every relocation record in it is c2's own. A nonzero here is not a bucket entry:
it is a live disagreement between `select_function` and the COFF emitter on a
body the oracle has certified, and it gets surfaced immediately.

**C4 — the partition.** `fnbyte-partition-broken 0` and the new
`fnbyte-reloc-partition-broken 0`; buckets sum to 178,975.

**C5 — unit tests without a toolchain.** `compare_relocs` is pure and is tested
directly on: equal sequences; one differing target name; one differing type
word; a `REL24` against `REL24|BRTAKEN`; differing counts in both directions; a
differing offset; an order swap at equal offset; a `Section` target against a
`Symbol` of the same string; a `PairDisplacement` mismatch.

---

## 5. THE PREDICTIONS — registered so they can lose

| # | claim | how it loses |
|---|---|---|
| **P1** | `s12`'s `?f` moves `exact → reloc-differs`; the cell reads `exact 2 · reloc-differs 1` where it read `exact 3 · differs 0` | it stays exact, or another cell function moves |
| **P2** | `?g` and `?anchor` in `s12` stay `exact` | either turns red |
| **P3** | `fnbyte-reloc-unknown` = **0** and `fnbyte-reloc-table-unreadable` = **0** — every graded TU's `.text` relocation table decodes | any positive count |
| **P4** | `fnbyte-reloc-differs` lands in **30 … 900**, point estimate **300** | outside the interval |
| **P5** | Every reloc-differ is a **target-name** disagreement at an existing REL24 site — **not** a count, offset or type disagreement. i.e. `reloc-differs-count` = `reloc-differs-offset` = `reloc-differs-type` = 0 | any of the three is positive |
| **P6** | `fnbyte-exact-bytes` = **35,982** exactly | any other value |
| **P7** | `fnbyte-match-tu-reloc-differs` = 0 (C3) | any positive count |
| **P8** | TU match **10 → 10**, `mismatch` **0 → 0**, `fnbyte-differs` **3,195 → 3,195**, `fnbyte-refused`/`-unbound`/`-denominator` unchanged | any move |
| **P9** | Every `Section`-target family count is **0** — no `.text` COMDAT relocation in this workload targets a section-definition symbol | positive, in which case it is named |
| **P10** | The gate stays **18/18 PASS, 0 mismatch** and the workspace tests stay green at 961 + this lane's | any red |

**P4's information content is one bit and it is stated as such.** The reasoning:
the 4,664 relocated-and-credited are `tail` + `seq` + `cond-pair`, and a
target disagreement needs c2 to have inlined a same-TU callee whose own body is
a single branch word — s12's exact shape. Forwarding wrappers are common in this
corpus, and mechanism I already accounts for 2,801 of the 3,195 *byte* differs,
so the class is certainly not empty; how big it is, this lane does not know.

---

## 6. WHAT THIS LANE WILL NOT DO

* **No codegen change.** A reloc-differ is a finding, not a repair; the repair is
  a later rung, priced from the families this one names.
* **No relaxation of a control to make a number.** If C3 fires, it is reported
  as a five-alarm and the lane stops to surface it.
* **No new gate.** FBM and every `fnbyte-` key stay out of `scripts/gate.sh`
  (`FUNCTION_BYTE_MATCH.md` §0). This licenses no emit.
* **No re-grading of the `differs` bucket on relocations.** A body whose bytes
  are already wrong scores zero and there is no second way to score zero.
</content>
</invoke>
