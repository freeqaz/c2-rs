# w-data — the grids, scored

Outcomes for `GRID.md`'s two frozen grids. Every prediction there was written
before the cell existed as an obj.

---

## GRID B — the fixture fence

### Positive cells — **3 of 3 as predicted**

`fixtures/cpp/wdata_static_scan.cpp` is **`match`** at `/O1`, byte-exact against
real `c2.dll` under wibo, as one obj containing all three cells.

| cell | predicted `.data` | measured | predicted `Characteristics` | measured |
|---|---:|---:|---|---|
| p0 | 32 | **32** | `0xC0301040` | **`0xC0301040`** |
| p1 | 256 | **256** | `0xC0401040` | **`0xC0401040`** |
| p2 | 32 | **32** | `0xC0301040` | **`0xC0301040`** |

* **p1, the separating cell, separated.** It is the only cell crossing
  `placement_align`'s 64-byte promotion and it reads **ALIGN_8** where its
  4-aligned `int` elements would give ALIGN_4. So the section nibble is the
  **size-promoted** alignment and not the `.gl` TYPE tag, and `Primes.cpp`'s
  `0xC0401040` on a 4-aligned `int[62]` is that rule and not a coincidence.
* **p2's `.text` is byte-identical to p0's**, as is p1's. Three cells differing
  in every function name, every array name, every array value and (for p1) the
  array's size emit the **same sixty-four bytes**. That is *"the class has zero
  free immediate fields"* as a measurement rather than a claim.
* Four distinct aux CheckSums were read off c2 across the lane
  (`0x25B5A181` Primes, `0xFC84F8C5` p0, `0x52892C86` p1, `0x2AFF742F` p2), each
  the plain `coff_checksum` of that object's own bytes. PREREG **P8** — the
  lane's least-confident prediction at 0.6 — is **RIGHT on four payloads**.

### Negative cells — **6 of 6, six distinct clauses**

`c2rs census`: **0/6 functions in class**, `Port` never `Mismatch`.

The census reports only the *fall-through* blocker, so it cannot show which
clause each cell tripped — w-cfgclass §6.2's confound, and this grid hit it. The
recognizer's own decline context was printed by a scratch patch
(`work/w-data/decline_probe.patch`, applied and reverted):

| cell | predicted clause | measured |
|---|---|---|
| n0 namespace-scope `static` | `resolve_data_def`, `!comdat` | **`static-scan-loop-object-out-of-class`** |
| n1 uninitialized | `resolve_data_def`, `!initialized` | **`static-scan-loop-object-out-of-class`** |
| n2 `short` elements | `scan-test-subscript` | **`scan-test-subscript-0x04`** |
| n3 `>` not `>=` | `scan-guard-not-ge` | **`scan-guard-not-ge-0x24`** |
| n4 two formals | `scan-formals-not-1` | **`scan-formals-not-1-0x26`** |
| n5 index starts at 1 | `scan-index-init-not-zero` | **`scan-index-init-not-zero-0x32`** |

**And the grid found a defect in the instrument, which is the useful part.** n0
and n1 originally read **`callee-unresolved-tail-call:eof`** — a refusal about a
symbol those bodies do not have. Their bodies are grammar-complete and this
parser accepts them; what refuses is the *object*. The census's fall-back key
was hiding a whole population under a name that names the wrong construct, which
is board #844/#1199's lesson one class over. `STATIC_SCAN_LOOP_OBJECT` now files
it, so the residue is sizeable.

n0 and n1 are shapes **c2 emits perfectly well** and this port refuses — a fence
narrower than the class, in the safe direction. Recorded as cells rather than
left to be discovered.

---

## GRID C — the multi-object obj: **R1 REFUTED, and the writer was wrong**

| | frozen prediction | measured |
|---|---|---|
| section order | **R1, GROUPED** | **R2, INTERLEAVED** |
| symbol order | groups after every function's | **interleaved, following section order** |
| symbol count | 29 | **29** |
| p0 / p1 / p2 nibbles | ALIGN_4 / ALIGN_8 / ALIGN_4 | **all three as predicted** |

```text
   1 .drectve   2 .debug$S   3 .XBLD$W   4 .XBLD$W
   5 .text(p0)  6 .data(p0)  7 .text(p1)  8 .data(p1)  9 .text(p2) 10 .data(p2)

  11/12 .text 13 ?p0@@YAHH@Z 14/15 .data 16 ?a@?1??p0@@YAHH@Z@4PAHA
  17/18 .text 19 ?p1@@YAHH@Z 20/21 .data 22 ?b@?1??p1@@YAHH@Z@4PAHA
  23/24 .text 25 ?p2@@YAHH@Z 26/27 .data 28 ?table@?1??p2@@YAHH@Z@4PAHA
```

**The symbol COUNT is 29 under both readings**, which is why the cell had to be
read by *order*. A lane that had checked `nsym` and moved on would have banked
the wrong rule.

### Three things this cell is worth, stated separately

1. **The refusal did its job.** `emit_comdat_obj` shipped refusing more than one
   defined object precisely because every rule in it came off one obj, and the
   grouped reading is the one that would have shipped. At n = 1 the two readings
   are the same obj; the refusal is the only thing that made the difference
   visible before a file was written.
2. **R2 was not a straw man.** The same writer already interleaves `.pdata`
   after its own `.text`, and the packed writer's own comment records
   `.rdata`/`.pdata` interleaving in `.text` order over 240 objs. What argued
   for R1 was that a COMDAT `.data` is `SELECT_ANY` and not `ASSOCIATIVE` — a
   real difference, and not the one that decides placement.
3. **The declined size was named before the cell ran** and did not have to be
   paid: had the answer been R2-with-a-wrong-writer *and* unfixable, the fixture
   would have split into three one-function files. It is one function call's
   worth of change instead.

### What still refuses after GRID C, and why each has no cell

* **more than one object per FUNCTION** — `undname.cpp` / `osfinfo.cpp`'s shape.
  Two objects in one *body* is a different question from two objects in one obj
  and GRID C says nothing about it.
* **an object on a FRAMED function** — where its `.data` sits relative to that
  function's associative `.pdata`, and where its symbol group sits among the
  `$M`/`$M`/`$T` triple, are both unmeasured.
* **an object on a FLOAT function** — `_fltused` goes after the first float
  function's *complete* group and nothing says whether the data group is inside
  that word.
