# w-rdata3 — the `.rdata$r` price, re-derived on master `e60f8902`

Every row below was measured on **this** tree. Nothing is inherited from
`w-rdata` (2026-08-04) or `w-rtti` (2026-08-07) except the *list* of seven,
which is quoted so the comparison is like-for-like.

The cells are the same three `w-rtti` §4.2 used, so a disagreement separates
"the tree moved" from "my probe is broken":

| cell | source | profile |
|---|---|---|
| `min_rtti_gr` | `struct A { A(); virtual void f(); int a; }; A::A(){}` | `/GR /O1 /Oi /EHsc` — **11 sections**, reproduced exactly (`p01/min_gr.obj`) |
| `ctl_data_two` | `fixtures/cpp/wsect_data_two.cpp` — **positive control** | `/Ox /GS- /c` |
| `gobj_gr` | `struct A{virtual void f();int a;}; A g;` | `/GR /O1 /Oi /EHsc` — 12 sections |

---

## The seven

| # | refusal | crate | **verdict** | instrument |
|---:|---|---|---|---|
| 1 | the vfptr-store leaf body class (`expr-op-0x27`) | `c2-il` | **UNPAID** | `c2rs census` → `0/1 functions in class`, key `expr-op-0x27`, blocking byte printed. Second instrument at the workload's OWN flags: `c2rs prefilter --flags-file` → `il-decode-failed … c2_il::functions() = None` |
| 2 | a reader for the `??_R*` record graph | `c2-il` | **HALF PAID — `.in` PAID, `.gl` UNPAID** | see §2 |
| 3 | a `DataRef` whose low half feeds a **store** | `c2-core` | **UNPAID** | see §3 |
| 4 | the `.rdata$r` / `.data`-COMDAT `Section` emitter + its `ADDR32` relocations | `c2-core` | **UNPAID** | scan `gap-metric writer-sections 10`; `PORT_WRITER_SECTIONS` carries no `.rdata$r`; `emit_data_obj` writes `selection: 0` at both of its `Section` literals |
| 5 | the DFS emission order over sections **and** undefined externals | `c2-core` | **UNPAID** | see §5 |
| 6 | the vftable `.rdata` COMDAT — Selection 6, symbol `Value` 4 | `c2-core` | **UNPAID** | `grep -rn "selection: 6" crates/` = **0 hits**. The three COMDAT constants in the tree are `NODUPLICATES 1`, `ANY 2`, `ASSOCIATIVE 5`; the emitted `selection:` values are `0` and `2` only |
| 7 | the `??_7type_info@@6B@` undefined external | `c2-core` | **UNPAID — and the mechanism was never the point**, see §7 |

**Seven items. Zero fully paid. One half paid.** The verdict is a third
**DECLINE**, on the same list, at a new master.

---

## §2 Item 2 has INVERTED, and that is the finding

`w-rtti` §4.2 and board **#931** located item 2's binding half in **`.in`**:
*"the one missing tag is `02` — the address of another symbol"*, with the `.gl`
half described as the reader being *"blind to (a) every COMDAT record and (b)
every initializer with a symbol reference"*, both attributed to that tag.

**Tag `02` has since been paid in full** (`w-tag02`, board #936; `w-inread` took
the symbol-address residue 913,136 → 0). Measured here on the RTTI cells with
the **standing** instrument `crates/c2-il/tests/in_init_probe.rs`
(`C2RS_IN_PROBE`), not a new one:

```text
ctl_data_two  records=39 elements=46 values=39 residue=0 symrefs=0
min_rtti_gr   records=43 elements=67 values=43 residue=0 symrefs=9  records_with_symrefs=6
gobj_gr       records=44 elements=68 values=44 residue=0 symrefs=10 records_with_symrefs=7
```

**`.in` reads 43 of 43 records of the minimal RTTI TU, residue 0, including all
nine symbol references across six records.** Every `[symbol-address … ]` residue
bucket is `0`. The contents of the RTTI record graph are fully readable today.

And `.gl` still returns **nothing**. Re-measured with a throwaway spike over
`gl_data_objects_ordered` (reverted; `git status --porcelain crates/` = 0 after):

| cell | records the `.gl` data reader returns |
|---|---:|
| `ctl_data_two` — the positive control | **2 of 2** (`?d1@@3HA`, `?d2@@3HA`) |
| `min_rtti_gr` — 11 sections, 6 of them data | **0** |
| `gobj_gr` | **1 of 12** — `g$initializer$` only |

Identical to `w-rtti` §4.2 to the digit. **So the two halves of item 2 have
swapped places**: the contents are readable and the *record headers* are not.

### §2.1 The `.gl` refusal is ONE ATTRIBUTE BYTE, and the records frame perfectly

Located by hexdumping the `.gl` at each record's terminating NUL
(`p01/glhex.py`, read-only) and walking `data_object_at`'s own gate sequence.

For `??_R4A@@6B@`, bytes from the NUL:

```text
   00   86   06   00 02   01   14   a0   a3 …
   ^NUL ^tag ^kind ^frame ^link ^size ^ATTR
```

* `tag` **`86`** → `align_of_type_tag` = 4 ✔
* `frame` **`00 02`** — the ORDINARY-DATA frame ✔ (not the `00 04` read-only one)
* `linkage` **`01`** = `LINKAGE_DEFINED_EXTERN` ✔
* `size` varint **`14`** = **20** ✔ — and `??_R3` reads `10` = 16, `??_R2` reads
  `08` = 8, `??_R1` reads `1c` = 28, `??_R0` reads `10` = 16. **All five sizes
  match `OBJ_RDATA_R_SHAPE.md` §3 exactly**, so the frame is not merely passing,
  it is *correct*.
* `DATA_ATTR` **`a0`** → `data_object_at` models `00` (uninitialized) and `80`
  (initialized) only, documents `60`/`E0` as `__declspec(selectany)`, and
  **fails closed on everything else**. `a0` is a fourth value. **This one byte
  refuses all five records.**

The vftable `??_7A@@6B@` refuses one gate *earlier* — its frame is **`00 04`**,
the read-only form `data_object_at` refuses by name (*"a read-only
(string-literal) record … refused here rather than admitted with a different
meaning"*), which is correct: a vftable is `.rdata`, not `.data`.

### §2.2 CORRECTION — `?g@@3UA@@A` is refused by the TYPE TAG, not by `.in` tag `02`

`w-rtti` §4.2, `OBJ_RDATA_R_SHAPE.md` §8.1 and board **#931** all give the same
cause for the third row: *"not even the plain `?g@@3UA@@A`, because its
initializer carries a **relocation** — the same element tag `02`."*

**That is wrong, and it is now falsifiable rather than arguable**: tag `02` is
paid to zero residue (§2 above, `symrefs=10 residue=0` on that very cell) and
the row is **unchanged at 1 of 12**. The actual gate, from the bytes:

```text
?g@@3UA@@A   00   c6   81   06   00 02   01   08   00   61 …
             ^NUL ^tag ^wide ^kind ^frame ^link ^size ^attr ^flags
```

Everything passes — wide mark `81` ✔, frame `00 02` ✔, linkage `01` ✔, size `08`
= 8 B (vfptr + `int`) ✔, attr `00` = uninitialized ✔ — and then
**`align_of_type_tag(0xC6)` returns `None`**. `align_of_type_tag` models exactly
four tags, `82`/`84`/`86`/`88` (1/2/4/8 bytes); `C6` is the **wide aggregate**
form and has no entry. The refusal is in a different function, in a different
file section, and has nothing to do with `.in`.

This matters beyond bookkeeping: a lane briefed off #931 would pay tag `02`
again — it is already paid — and still read `1 of 12`.

---

## §3 Item 3 — the low half lands in the scratch register, not an argument register

`crates/c2-core/src/codegen/select.rs`: `ARG_REGS = [3,4,5,6,7,8,9,10]`,
`SCRATCH_REG = 11`. `data_refs_of` (`crates/c2-core/src/lib.rs`) searches the
setup words for *"the unique `addi rD,r11,0` among the setup words"* with
`d ∈ ARG_REGS`, and returns `NotImplemented("a data-symbol address with no
`addi rD,r11,0` low half")` when there is none.

The minimal TU's body, disassembled from the **real** obj at the workload's own
`/GR /O1` (`scripts/gt_dump.py work/w-rdata3/p01/min_gr.obj`):

```text
   0000  3d600000  lis  11, 0        ; REFHI -> ??_7A@@6B@ ; PAIR
   0004  396b0000  addi 11, 11, 0    ; REFLO -> ??_7A@@6B@ ; PAIR
   0008  91630000  stw  11, 0(3)
   000c  4e800020  blr
```

`rD` is **r11**, the scratch register itself — not in `ARG_REGS`. Refused.

> **A profile trap worth recording.** At the harness's *default* `/Ox /GS- /c`
> the same source emits `addi 10, 11, 0` — and **r10 IS in `ARG_REGS`**. So a
> lane that priced item 3 off the default profile would find the low-half search
> succeeding and could score it PAID; at the workload's own flags it does not.
> Item 3 is unpaid at the profile that matters, and the `/Ox` obj has no
> `.rdata$r` at all (6 sections, no RTTI), so it cannot be the pricing cell.

`c2rs diff min_rtti.cpp` → `ReferenceReplay=ByteExact (ref=1033B replay=1033B)
Port=NotImplemented`. The reference half is byte-exact and the port half is an
honest refusal.

---

## §5 Item 5 — the only ordering rule in the tree is explicitly NOT this one

`crates/c2-core/src/coff/order.rs` is the **`.text` emission order**: a
readiness loop over the IL's reference set, repeated until nothing more becomes
ready. Its own module doc rules out the shape item 5 needs, in terms:

> *"It takes as many passes as the chain is deep … (Equivalently: **it is not a
> DFS from each root either** — a DFS of `f, h, g` with `f→g` gives `g, f, h`
> and c2 gives `h, g, f`.)"*

and, on the input:

> *"the caller must pass the IL's reference set, **not the obj's**; a planner fed
> the relocation list alone gets this case wrong"*

Item 5 needs the exact opposite — a DFS pre-order over the **relocation graph**,
per group, cut at the vftable run (`OBJ_RDATA_R_SHAPE.md` §6.1). The only
occurrence of that phrase in `crates/` is a doc comment in
`coff/function.rs:125` **describing what does not exist**. Unpaid.

---

## §7 Item 7 — the mechanism predates the pricing by three days

`emit_obj` has emitted **undefined external DATA symbols** (`Type 0x0000`,
section 0) since **`7e09ccd9`, 2026-08-01** — *"WR1: the single-symbol data
address, as a call argument"* — which is **three days before `w-rdata` counted
item 7 as unpaid on 2026-08-04**. So item 7 was never a claim that the port
cannot emit that *kind* of symbol record. It is a claim about **placement**, and
`coff/data.rs` states the unpaid part itself:

> *"an undefined external needs a symbol record spliced in at index 5, **between
> `.debug$S`'s aux and the `.XBLD$W` C2 watermark** — MEASURED on
> `t03_ptr_to_extern` and `t05_ptr_to_func` … That is a symbol-table shape this
> writer does not model, so it refuses."*

`??_7type_info@@6B@` is reached from the `??_R0` **`.data` COMDAT**'s single
`ADDR32`, which is exactly that shape. Still refused — and recorded this way so
the next lane does not "pay" item 7 by pointing at WR1.

---

## §9 The remaining price, with the lane each piece belongs to

| piece | crate / seam | owning lane |
|---|---|---|
| the vfptr-store leaf body (`expr-op-0x27`) — item 1 | `c2-il` body decode | a **`c2-il` body-shape** lane (the `leaf_store` / `ctor_dtor` family) |
| `data_object_at`: the `DATA_ATTR` COMDAT value `0xA0` — item 2a | `c2-il` `.gl` | a **`c2-il` `.gl`-attribute** lane |
| `align_of_type_tag`: the wide aggregate TYPE tag `0xC6` — item 2b | `c2-il` `.gl` | same lane; it is 4 lines from 2a and blocks a **plain** `.data` object today |
| `data_object_at`: the `00 04` read-only frame (the vftable) — item 2c | `c2-il` `.gl` | same lane |
| `data_refs_of`: a low half in the scratch register feeding a store — item 3 | `c2-core` `codegen/` + `lib.rs` | **`w-front2`'s seam this wave** — NOT this lane's, and not touched here |
| the `.rdata$r` / COMDAT-`.data` `Section` emitter — item 4 | `c2-core` `coff/` | this lane's seam, **blocked behind items 1–3** |
| the per-group DFS over the relocation graph — item 5 | `c2-core` `coff/` | this lane's seam, blocked behind item 2 |
| the vftable COMDAT Selection 6 / `Value` 4 — item 6 | `c2-core` `coff/` | this lane's seam, blocked behind item 2c |
| the undefined-external splice at symbol index 5 — item 7 | `c2-core` `coff/` | this lane's seam, blocked behind item 4 |

**Four of the nine pieces are `c2-il`'s and every one of the five in `coff/` is
downstream of them.** That is the same shape `w-rdata` §4 reported and the same
one `w-rtti` §8 reported: the writer cannot be built first, because there is
nothing for it to write.

**And even fully paid it is worth `+0` TU match** — #360/#362, quoted and not
re-derived here: `A∧B` = 27 with `|{A∧B} \ C| = 0`, `|D∨E|` = 10, and **0 of the
676** `.rdata$r` TUs are in `D∨E`. This lane's own scan is consistent with all
three and independently confirms none: `a-and-b-and-c 27`,
`a-and-b-and-c-and-d-or-e 10`, `factor-d 10`, `factor-e 2`, `match 10`.
