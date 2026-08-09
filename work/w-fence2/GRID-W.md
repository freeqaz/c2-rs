# GRID-W — did c2 KEEP the call? The decline side, read off 7,552 workload call sites

Instrument: `work/w-fence2/scratch_gridw.patch` (68 lines over
`crates/c2-harness/src/gap/fnbytes.rs`), applied to the base tree
(`acb151ed`, workload `d7a3c1aa`) and **reverted**. In no commit as a `crates/`
change. Scan: `work/w-fence2/scan_gridw.{out,jsonl}`; aggregation
`work/w-fence2/gridw.py`.

## 0. What is measured, and why it is not the port's correctness

For every IL call edge *(caller → a callee THIS TU defines)*, the instrument asks
the **reference obj** whether the caller's own `REL24` target set names the
callee.

* **`kept`** — c2 emitted the call. **c2 DECLINED to inline.**
* **`inlined`** — the IL spells the call and c2's obj has no relocation for it.
* **`unknown`** — the reference obj carries no target list for that caller. **0
  of 7,552.**

This is the decline side *directly*. `w-inlfence2` §2 crossed the callee's size
against **the port's own FBM verdict**, which conflates "c2 inlined it" with
"the port got the caller wrong for some other reason" — and its own table shows
the confusion (7 `differs` at 81–308 B, 2 `reloc-differs` above 308 B, neither of
which this instrument sees as an inline). Nothing here reads a port verdict.

## 1. By the callee's own reference `.text` COMDAT size (ground truth)

| band (B) | kept | inlined |
|---:|---:|---:|
| 0–15 | 0 | 4,517 |
| 16–31 | 0 | 423 |
| 32–47 | 0 | 168 |
| 48–63 | 0 | 773 |
| 64–79 | **9** | **503** |
| **80–95** | **137** | **67** |
| **96–111** | **317** | **0** |
| 112–127 | 162 | 0 |
| 128–143 | 372 | 0 |
| 144–159 | 16 | 0 |
| 160–175 | 5 | 0 |
| 176–191 | 44 | 0 |
| 192–511 | 30 | 0 |
| ≥ 512 | 10 | 0 |
| **TOTAL** | **1,101** | **6,451** |

> ### **THE LARGEST CALLEE c2 IS MEASURED TO INLINE ANYWHERE ON THE WORKLOAD IS 80 BYTES.** Exact sizes of the `inlined` arm, script-counted: the top three are 76 B (428 sites), 80 B (67) and 72 B (6). **Above 80 bytes there are 955 `kept` sites and ZERO `inlined` ones.**

The boundary is therefore **(80, 96]** of emitted `.text`, over 7,552 sites — and
it is *tighter* than every published bracket it has to live under:

| source | bracket | cells |
|---|---|---:|
| `WB_INLINE_FINDINGS` F2, EXTERNAL at `/O1` | `(100, 116]` | 60 |
| `WB_INLINE_FINDINGS` F9 / GRID-J, straight-line at the workload flags | `(96, 120]` | 56 |
| `WB_INLINE_FINDINGS` F9, loop-bodied | `(56, 80]` | 56 |
| `WB_INLINE_FINDINGS` F1, STATIC at `/O1` | `(300, 308]` | 120 |
| **GRID-W, this lane, the workload itself** | **`(80, 96]`** | **7,552** |

**64–95 B is a MIXED band** — 146 kept against 570 inlined — so no rule may
accept *or* decline in it. That is the region the shipped fence must refuse
rather than answer.

## 2. By the size the PORT can actually measure (the shippable input)

`port` is the port's own lowered `/Gy` body for the callee; `none` means the
port cannot lower it, which is the overwhelming majority and must bias to
refusal.

| band (B) | kept | inlined |
|---:|---:|---:|
| 0–15 | 0 | 3,163 |
| 32–47 | 0 | 2 |
| **144–159** | **1** | **0** |
| `none` | 1,100 | 3,286 |

> ### **THE PORT'S INPUT SEPARATES PERFECTLY, AND THE SEPARATING SITE IS THE SUBJECT.** Exactly ONE call site in the entire 878-TU workload has a locally-defined callee the port can lower **and** a call c2 kept:
>
> ```text
> XW-KEPT-SMALL src/xdk/LIBCMT/vsnprnc.cpp :: vsprintf_s -> _vsprintf_s_l
>               ref=Some(152) port=Some(152)
> ```
>
> The port's lowering of `_vsprintf_s_l` is **152 bytes and so is c2's**, which
> is `w-vsnprnc`'s `fnbyte-exact 2/2` seen from the relocation side.

Every other port-lowerable callee in the workload is ≤ 47 bytes and every one of
them c2 inlined. So **the constant this lane raises is worth 0 functions on this
workload today** (§3) and buys the soundness of the parser narrowing.

## 3. The threshold, and the margin it is chosen with

> **`INLINE_DECLINE_BYTES = 128`.**

* **48 bytes / 12 words above the largest inline measured here** (80 B), a
  factor of 1.6.
* **Above F2's `(100,116]` EXTERNAL first-declined point (116) and above GRID-J's
  straight-line first-declined point (120)** — so it is more conservative than
  either published `/O1` external ceiling, not a fit to them.
* **Not fitted to the subject.** `_vsprintf_s_l` is 152 B; the threshold is 128,
  a power of two, and moving it anywhere in `(95, 152]` changes nothing on this
  workload — the port-side table has no site between 47 and 152.
* **It is an EXTERNAL-linkage constant.** `WB_INLINE_FINDINGS` F1 puts the
  STATIC ceiling at `(300,308]`, three times higher, so a `static` callee of
  128–300 B is one c2 *would* inline. The parser therefore admits only callees
  whose `.gl` defined record carries the external linkage byte (GRID-K), and the
  STATIC class keeps the wholesale refusal — decline clause **D2**.
* **It is an `/O1` constant.** F1/F2 measure the favour-speed ceilings at
  `(212,252]` and `(156,164]`; 128 is *below* the second, so at `/O2`/`/Ox` this
  rule would be wrong. The mode gate is in the parser (#1638) — decline **D3**.
