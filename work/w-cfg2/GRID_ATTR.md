# w-cfg2 — GRID A: the `.gl` DATA attribute byte, frozen before the first `cl.exe`

## Why this grid exists — decline clause **D1 FIRED**

`PREREG.md` §4 **D1** says: if `gl_data_objects_ordered` and
`in_scalar_initializers` cannot between them produce, for `Primes.cpp`'s bundle,
**both** the name and 248 bytes byte-identical to the reference obj's `.data`,
decline and publish the measurement.

Measured at `c102f7d2`, before any line under `crates/` changed:

* **`.in` DELIVERS.** `in_scalar_initializers` returns, for token `0xea09`,
  **248 bytes** that are byte-identical to the reference obj's `.data` section:
  `0000001d 00000025 … 000d744f 00000000`. `records=38 accepted=38
  elements=106 residue=0`, the totality identity closes.
* **`.gl` DOES NOT.** `gl_data_objects_ordered` returns **zero** records for this
  bundle. The record is *there* and frames perfectly —

  ```text
  .gl 0x14d:  ea 09 | 24 '$' | ?primes@?1??NextHashPrime@@YAHH@Z@4PAHA 00
  .gl 0x178:  86    06    00 02   04      80 f8 00 00 00     a0
              tag   kind  lit'l   linkage size varint (248)  ATTR
              align4             STATIC
  ```

  — and it is rejected on **one byte**: `data_object_at` enumerates
  `DATA_ATTR_UNINITIALIZED = 0x00` and `DATA_ATTR_INITIALIZED = 0x80` and fails
  closed on anything else. The observed value is **`0xa0`**.

**So D1 fires as written.** It is recorded as fired, and it is scored in the
rung as a miss on the *clause's* design rather than on the target: the clause
cannot tell "the reader has no production for this shape" from "the reader has
the production and one enumerated byte value is unwitnessed", and the measured
answer is the second. D1's own remedy — *"the honest deliverable is the
measurement"* — is this grid.

**Nothing below is a licence to guess `0xa0`.** A wrong reading of this byte is a
wrong `.bss`-vs-`.data` decision, which is a wrong section **count**, which
mismatches at file offset 2 — the failure `data_object_at`'s doc names and the
one the fail-closed arm exists to prevent. The grid decides it against real
`c2` or the lane does not widen the byte at all.

## The cells

Every cell is compiled with the workload's own flags (`/O1 /Oi /EHsc /GR …`) by
real `c2.dll` under wibo, and each is read **twice** — once as `.gl` (the attr
byte) and once as the obj (`.data`/`.bss` characteristics, COMDAT selection,
symbol StorageClass). A rule is banked only if both readings agree on all six.

| cell | source |
|---|---|
| **a1** | `int f(){ static int p[4]={1,2,3,0}; return p[0]+p[3]; }` — function-local static, initialized, aggregate. **`Primes.cpp`'s own shape.** |
| **a2** | `int f(){ static int p=7; return p; }` — function-local static, initialized, scalar |
| **a3** | `int f(int i){ static int p[4]; p[i]=i; return p[0]; }` — function-local static, **uninitialized** |
| **a4** | `static int p[4]={1,2,3,0}; int f(){ return p[0]+p[3]; }` — namespace-scope **static**, initialized, aggregate |
| **a5** | `int p[4]={1,2,3,0}; int f(){ return p[0]+p[3]; }` — namespace-scope **external**, initialized, aggregate |
| **a6** | `int gDef=3; int f(){ return gDef; }` — the shape `data_object_at`'s doc already records at attr `80`. **THE POSITIVE CONTROL**: if a6 does not read `80`, the instrument is wrong and no other cell means anything. |

`a4` and `a5` separate the attribute from the **linkage** field (`04` vs `01`),
which is the confound a grid of function-local cells alone could not exclude.
`a2` separates it from **aggregate-ness**. `a3` separates it from
**initialized-ness**, which already owns bit `0x80`.

## The rivals, and each one's per-cell prediction — FROZEN

`R-A` is the lane's registered call. Every rival must be separated by a cell
that discriminates it, and the separating cell is named.

| | **R-A** bit `0x20` = *the object is its own COMDAT* (a function-local static) | **R-B** bit `0x20` = *aggregate* (array/struct, not scalar) | **R-C** bit `0x20` = *the initializer has more elements than the reader saw* | **R-D** `0xa0` is not a bitfield at all — a third enumerated value meaning "function-local initialized" |
|---|---|---|---|---|
| **a1** | `a0` | `a0` | `a0` | `a0` |
| **a2** | **`a0`** | **`80`** | `80` | `a0` |
| **a3** | **`20`** | `00` | `00` | **`00`** — R-D has no uninitialized member, so it predicts the shipped value |
| **a4** | **`80`** | **`a0`** | `a0` | `80` |
| **a5** | `80` | `a0` | `a0` | `80` |
| **a6** | `80` | `80` | `80` | `80` |

**The separating cells, named before the run:**

* **a2** separates **R-A** from **R-B**/**R-C** (`a0` vs `80`).
* **a4** separates **R-A**/**R-D** from **R-B**/**R-C** (`80` vs `a0`).
* **a3** separates **R-A** from **R-D** (`20` vs `00`) — and it is the cell that
  matters most, because it is the one that decides whether widening the
  enumeration touches the **`.bss`** arm of `emit_data_obj`, which ships today.
* **a6** is the positive control and discriminates nothing on purpose.

If a1's attr is anything other than `0xa0` the grid is measuring the wrong byte
and everything above is void; that is checked first.

## The obj half — predicted before the run, independently of the `.gl` half

The `.gl` reading is a claim about a *stream*. What the port has to get right is
the **obj**, so each cell's obj is predicted too, and a rule is banked only if
the two halves agree.

| cell | predicted `.data`/`.bss` section | predicted symbol |
|---|---|---|
| a1 | **COMDAT** `.data`, `IMAGE_SCN_LNK_COMDAT` set, aux `Selection = 2` (ANY) | STATIC, in that section |
| a2 | COMDAT `.data`, Selection 2 | STATIC |
| a3 | COMDAT `.bss`, Selection 2 | STATIC |
| a4 | **non-COMDAT** `.data`, no `LNK_COMDAT`, aux Selection 0 | STATIC |
| a5 | non-COMDAT `.data` | **EXTERNAL** |
| a6 | non-COMDAT `.data` | EXTERNAL |

`Primes.cpp`'s own obj already reads `.data chars=0xc0401040` (`LNK_COMDAT`
set) with `aux(len=248 … sel=2)` and the symbol STATIC, which is a1's predicted
row and is the reason **R-A** is the registered call.

## What is NOT decided here

The grid decides what the byte **means**. It does not decide the **section
order** — where a `.data` group sits relative to the code groups — which is
`PREREG.md` **P5**/**D3**'s question and board **#1179**'s rule, and is graded
separately. A cell here that happens to place a section is not evidence for
that.
