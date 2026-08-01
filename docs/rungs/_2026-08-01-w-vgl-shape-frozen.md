# w-vgl — the `.gl` function-record shape, FROZEN

Committed **before the held-out structural grid was designed, compiled or
scored** (E6, and §9.19's rule: 360/360 in sample, 296/394 out). Derived on
**one** TU — `src/system/obj/TextFile.cpp`, the TU §9.18.3 transcribed its two
witnesses from. Everything below is therefore in-sample and is frozen here so the
out-of-sample score is a score and not a refit.

## The shape

```text
<sep> <name> <sep>                       sep ∈ { 0x00, 0x26 }
  <ret-type: 2 bytes> <linkage> <ret-size>
  <attribute bytes>                      3 bytes for a non-virtual member;
                                         9 for a virtual one — `20 01 <slot>
                                         02 <tag16> <vftable-token16>`, where
                                         <slot> is the vtable byte offset
  80 <LE32 type-id>                      high two bytes zero
  <varint> <varint>                      USUALLY 0, 0 — but NOT always
  80 <LE32 body-start offset into .ex>   <- the field `EmitBinding` wants
  80 <LE32 …>
```

`<varint>` is the encoding `readers.rs::read_varint` already models: `0x80`
followed by a 4-byte LE `i32`, otherwise **one signed byte**.

## The two defects, and they are not the ones §9.18.3 named

§9.18.3 read the witness as *"a virtual member's `.gl` record carries extra
material between the name and the offset field … so the `80 <LE32> 00 00` framing
**and** the 32-byte name-distance bound lose it."* **The second half is wrong**,
and the first half is right for a different reason than "virtual".

**D1 — the `0x26` name separator, and it is the big one.**
`gl::gl_symbol_runs` opens a run only after a `0x00`. A name introduced by
`0x26` is therefore **never seen at all**, and the record's "nearest preceding
run" is some unrelated symbol 85–194 bytes back — which the 32-byte bound then
correctly refuses. `gl.rs`'s own `NAME_SEPARATORS = [0x00, 0x26]` constant
records that `0x26` exists and lists what carries it (`??_G`/`??_E` deleting
destructors, `??_7` vftables, `??_R*` RTTI, `_CT`/`_TI` EH descriptors, **and
header-inline member functions**) — the constant was measured and the scanner
was never taught it.

That is also why the residue looked *virtual*: an out-of-line virtual
(`??1String@@UAA@XZ`) is `00`-separated and binds today; an **inline** one is
`26`-separated and vanishes. The 98.8 % is a fact about *where the function is
defined*, not about the vtable.

**D2 — the two varint fields are pinned to the literal bytes `00 00`.**
`bind.rs::emit_offset_framed` requires `gl[o-2] == 0 && gl[o-1] == 0`. Those are
two varint fields whose value is *usually* zero. `?Print@TextFile@@UAAXPBD@Z`
carries `2c 00` — value 44 — and the record is not framed at all. **This is
board #121's defect one field later**: `gl[o-5] == 0x10` pins a byte of the
type-id's *value*; `gl[o-2] == 0` pins a varint's *value*. Same class, same file.

## The rule, as it will be implemented and scored

Fail-closed, forward from the name — no "nearest preceding run", no distance
constant:

1. Runs open **and terminate** at `0x00` or `0x26`.
2. For each run, the record window is `[name NUL, next run's separator)`.
3. The **first** `0x80` byte in that window must be the type-id field and must
   satisfy `gl[q+3] == 0 && gl[q+4] == 0`. If the first `0x80` fails that, or
   there is none, **refuse this run** — it is not a defined-function record.
4. Skip exactly **two** varints.
5. The next byte must be `0x80`; its LE32 is the body-start offset. Otherwise
   **refuse** — a data symbol has no body and must not be given one.

## In-sample result (`TextFile.cpp`), stated so the out-of-sample score is honest

| variant | records | named | nameless | body offset on a `.ex` `4F 1F` | emitted symbols covered |
|---|---:|---:|---:|---:|---:|
| today | 674 | 604 | **70** | 604 / 604 | **30 of 32** |
| + D1 only | 674 | **674** | **0** | 674 / 674 | 31 of 32 |
| + D2 only (byte-pattern) | 686 | 613 | 73 | 606 / 613 | 31 of 32 |
| D1 + D2, **backward** byte-pattern | 678 | 677 | 1 | 675 / 677 | 32 of 32 |
| **D1 + D2, the rule above (forward)** | **676** | **676** | **0** | **676 / 676** | **32 of 32** |

Injectivity holds under the frozen rule: **0** names claiming more than one
record, **0** offsets claimed by more than one name. 292 runs are refused, which
is the correct answer for the source path, `__C1_11886`, undefined externals and
data symbols.

**The loosened *backward* predicate is rejected on purpose.** It reaches 32 of 32
too, but admits **3 records whose body offset is not a `4F 1F` function start** —
false positives, and a false record binds a body under another symbol's name,
which is a mis-emit and outranks the gap it closes. The forward rule admits
**zero**.

## Two predictions this rule makes, registered before the grid exists

* **P1 — the 32-byte name-distance bound never fires again.** Under the rule the
  measured name-NUL → body-field distance has maximum **27** on 676 records
  (modes 15, 17, 21, 23, 25). `EMIT_MAX_NAME_TO_OFFSET = 32` was **never the
  defect** and must not be widened. Refuted by any record needing > 32.
* **P2 — the virtual attribute block is 9 bytes where a non-virtual member's is
  3**, and the 6-byte difference is `20 01 <slot> 02 <tag16> <token16>` with
  `<slot>` a multiple of 4 (observed 0x04, 0x0c, 0x10, 0x14, 0x44). Refuted by a
  virtual record with a different width — which **E11 predicts will happen** under
  multiple or virtual inheritance, where a thunk/adjustor field has somewhere to
  go.

## Scope this rule is NOT allowed to move

The **gate** (`codec::gl_offset_framed`, `gl_defined_names`,
`Bindings::per_record`) is deliberately untouched, exactly as `emit_offset_framed`
is already kept separate from `codec::gl_offset_framed`. Teaching the gate this
reader would move the accepted class and could cost the 6 byte-exact TUs; that is
a separate, gated decision and is priced rather than taken here.
