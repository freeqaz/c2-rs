# w-tag02 — element tag `02`, as MEASURED

Every row below is read off a real capture of a frozen, `sha256`'d cell at the
**workload's own** profile (`work/w-tag02/flags_probe.txt` — `/nologo /wd4355
/wd4164 /c /GR /O1 /Oi /EHsc`), through real `cl.exe` + `c2.dll` under wibo.
Nothing here is constructed from a rule. `work/w-tag02/grid.sha256` pins the
sources; `work/w-tag02/scan.py` is instrument 1 and reads no Rust.

---

## 1. The element

```text
  element := 02 <target-token> <offset> <n>

    <target-token>   read_token_var — 2 bytes when the SECOND stream byte's high
                     bit is clear, else 4.  BOTH FORMS MEASURED (t24).
    <offset>         a varint: one byte below 0x80, else `80` + a 4-byte LE i32.
                     MEASURED at 0, 4, 8, 128, 160, 1200, 65536 and **-4**.
    <n>              04 on every one of the 31 tag-02 elements in the grid.
                     OBSERVED constant, not a known one — see §4.
```

The element's contribution to the object's raw bytes is **`<offset>` as a
big-endian i32**, and the obj carries an `IMAGE_REL_PPC_ADDR32` (`0x0002`,
**no PAIR**) at that byte position naming the target's COFF symbol.

That is the same transform `ininit.rs::read_value` already performs at width 4:
the `.in` escape payload is little-endian and the obj's bytes are big-endian.

## 2. The cells, byte for byte

| cell | `.in` element | `.data` raw | relocation |
|---|---|---|---|
| `t01_ptr_to_global` | `02 e3 09 · 00 · 04` | `00 00 00 00` | ADDR32 → `?gi@@3HA` (`.bss`, this TU) |
| `t03_ptr_to_extern` | `02 e3 09 · 00 · 04` | `00 00 00 00` | ADDR32 → `?ge@@3HA` **undefined**, symbol index **5** |
| `t04_ptr_to_static` | `02 e3 09 · 00 · 04` | `00 00 00 01 00 00 00 00` | ADDR32 @4 → `si` (STATIC, undecorated) |
| `t05_ptr_to_func` | `02 e3 09 · 00 · 04` | `00 00 00 00` | ADDR32 → `?f@@YAXXZ` undefined, **`type=0x0020`** |
| `t06_ptr_to_literal` | `02 e4 09 · 00 · 04` | `00 00 00 00` | ADDR32 → `??_C@_03FIKCJHKP@abc?$AA@`, a `.rdata` **COMDAT** |
| `t08_ptr_array` | `02 e3 09 00 04` `02 e4 09 00 04` | `00 00 00 00 00 00 00 00` | two ADDR32, @0 and @4 |
| `t09_struct_offset` | `02 ec 09 · 04 · 04` | `00 00 00 04` | ADDR32 @0 → `?s@@3US@@A` |
| `t10_array_offset` | `02 e3 09 · 08 · 04` | `00 00 00 08` | ADDR32 @0 → `?arr@@3PAHA` |
| `t13_mixed_struct` | `01 01 04 07` then `02 e3 09 00 04` | `00 00 00 07 00 00 00 00` | ADDR32 @**4** |
| `t15_two_ptrs` | two records, one element each | `00 00 00 00 00 00 00 00` | two ADDR32 |
| `t16_ptr_to_self` | `02 eb 09 · 00 · 04` | `00 00 00 00` | ADDR32 → `?n@@3UN@@A` — **the record's own object** |
| `t18_offset_128` | `02 ec 09 · 80 80 00 00 00 · 04` | `00 00 00 80` | ADDR32 @0 |
| `t19_offset_160` | `02 e3 09 · 80 a0 00 00 00 · 04` | `00 00 00 a0` | ADDR32 @0 |
| `t20_offset_1200` | `02 e3 09 · 80 b0 04 00 00 · 04` | `00 00 04 b0` | ADDR32 @0 |
| `t21_offset_negative` | `02 e3 09 · 80 fc ff ff ff · 04` | `ff ff ff fc` | ADDR32 @0 — **a negative addend** |
| `t22_offset_65540` | `02 ec 09 · 80 00 00 01 00 · 04` | `00 01 00 00` | ADDR32 @0 |
| `t23_wide_token` | record token 4-byte, target 2-byte | `00 00 00 00` | ADDR32 @0 |
| `t24_wide_target_token` | `02 fb 82 01 00 · 00 · 04` | `00 00 00 00` | ADDR32 @0 — **the 4-byte target form** |

### The `t11_vfptr` cell — #931's own

```text
  ?g          ee 09 00 · 02 0d 0a 00 04 · 02 e4 09 00 04 · 07
  ??_R0       0f 0a 00 · 02 13 0a 00 04 · 01 03 04 00 · 03 08 ".?AUA@@\0" · 07
  ??_R1       12 0a 00 · 02 0f 0a 00 04 · 01 02 04 00 · 01 01 04 00
                       · 01 01 04 80 ffffffff · 01 01 04 00 · 01 02 04 40
                       · 02 10 0a 00 04 · 07
  ??_R3       10 0a 00 · 01 02 04 00 · 01 02 04 00 · 01 02 04 01 · 02 11 0a 00 04 · 07
  ??_R2/vftbl 0d 0a 00 · 01 02 04 00 ×3 · 02 0f 0a 00 04 · 02 10 0a 00 04 · 07
```

Twelve sections, of which four are `.rdata$r`. The whole `??_R*` graph is
spelled in `.in` and the three "irreducible" integers of
`OBJ_RDATA_R_SHAPE.md` §4 are visible: `??_R1`'s `pdisp = -1`
(`01 01 04 80 ffffffff`) and `attributes = 0x40` (`01 02 04 40`), and
`??_R3`'s `numBaseClasses = 1` (`01 02 04 01`). Board **#931** is confirmed on
a fresh capture.

## 3. The three boundaries that are NOT tag 02

| cell | what happens | why it matters |
|---|---|---|
| `t02_null_ptr` (`int* np = 0;`) | **no `.in` initializer record at all**, and the obj puts `np` in **`.bss`** | a null pointer is not a tag-`01` zero either — it is *uninitialized*. The prereg's P4 named the right verdict for the wrong reason |
| `t07_char_array` (`char s[4] = "abc";`) | element tag **`03`**, inline bytes, `.data` raw `61 62 63 00`, **no relocation** | the tag-03 boundary is about *where the bytes come from*, not about being a `char*` |
| `t12_ptr_to_member` (`int A::*pm = &A::b;`) | element tag **`01`**, `.data` raw `00 00 00 04`, **no relocation** | a member pointer is a plain scalar offset. P7 held |

## 4. What is NOT measured, and is therefore refused

1. **`<n>` other than `04`.** Nothing in the grid can vary it — every pointer on
   this target is 4 bytes. Any other value is unmeasured and the reader refuses
   it rather than assuming it is a width.
2. **A short-form offset in `81..FF`.** Every measured negative offset
   *escapes* (`-4` is `80 fc ff ff ff`, not `fc`), exactly as `ininit.rs`
   already records for scalar values. So a high-bit short form is treated as a
   desync and refused, not sign-extended.
3. **The `.rdata`-COMDAT target** (`t06`) and the **undefined-external** target
   (`t03`, `t05`). Both are read fine; both are refused by the **writer**, for
   reasons that are about the obj's *symbol table*, not about tag `02`.
4. **`t14_const_ptr`** — `int* const cp = &gi;`. The `.in` carries the tag-02
   record and **c2 emits no `.data`, no section and no symbol for `cp`**. See
   `PREREG.md` addendum A P21: this is a live wrong-emit hazard for any widening
   that hands `data_tu` bytes for it, and it is *not* covered by the existing
   `!external && !initialized && !referenced` drop rule, because `cp` **is**
   initialized.
