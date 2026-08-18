# WB-DAGORDER2 — PREREG R2: the grid, frozen by content hash

Committed **before the first `cl.exe` of this lane**. Round 1 is
[`WB_DAGORDER2_PREREG.md`](WB_DAGORDER2_PREREG.md).

## The freeze

    file    docs/whitebox/grids/wb-dagorder2/candorder_grid.cpp
    sha256  b06a05fc83fe0cca45a539684b87d88998b70c305e28ca29b54fb4fcefafeee6
    cells   20 graded bodies + 2 extern declarations

A hold-out frozen by *name* is not frozen. The hash above is the freeze; if the
file's hash at scoring time differs from this line, every result in this lane is
**void**, not provisional.

## The cells and what each one is for

| family | cells | the axis it moves | which readings it separates |
|---|---|---|---|
| **A** | `cnd_a1` … `cnd_a8` | **n**, the number of candidates competing | none by itself — it is the **series**, and it is what makes any rule a rule rather than a cell (#3147) |
| **X** | `cnd_x2`, `cnd_x2r`, `cnd_x3`, `cnd_x3r` | commutative operand order | H-SCHED vs the rest — **registered as possibly inert**, see below |
| **X-sub** | `cnd_s2`, `cnd_s2r` | non-commutative operand order | H-SCHED vs the rest, without the reassociation defect |
| **H** | `cnd_h2`, `cnd_h2r`, `cnd_h3`, `cnd_h3r` | **dependence height only** — formal order, declaration order and live set all held fixed | **THE DISCRIMINATOR.** H-SCHED predicts the pairs DISAGREE; H-SRC, H-REV, H-ARR and H-USE all predict they AGREE, because none of them can see height |
| **U** | `cnd_u2`, `cnd_u2r` | use count | H-USE vs the rest |
| **C** | `cnd_c0` | nothing live across the call | **the instrument control** — if this one takes a callee-saved colour, the instrument is not reading what it claims and the batch is void |

## The registered caveat, before the first compile

`cnd_x2` / `cnd_x2r` compute the **same value**. A reassociating front end may
normalize `b + a` to `a + b` and emit one tuple list for both, in which case the
X pair is **inert** — it neither confirms nor refutes H-SCHED. This is written
down now so that an inert X pair cannot be reported afterwards as a refutation.
The **H family has no such defect**: `cnd_h2` and `cnd_h2r` compute different
values by construction and differ only in which formal carries the taller
producer.

## Profile scope — both, and the disagreement is a deliverable

Every cell is compiled twice:

* **`/nologo /c /GR /O1 /Oi /EHsc`** — the workload's own profile, the one the
  878-TU scan and every conversion is graded at.
* **`/nologo /c /GR /Ox /Oi /EHsc`** — because `w-section` found `/Ox`
  disagreeing with `/O1` on **seven of eight fields** of the section emitter.

Every finding in the findings doc states its profile scope. A finding measured
at one profile is reported as holding at one profile.

## Reading rule, fixed now

The register a formal ends on is read from the **`/FAsc` listing** (the seam
that makes c2 narrate its own output) and confirmed against the **obj's
disassembled `.text` COMDAT**. A claim confirmed by only one of the two is
labelled as such. The callee-saved run is handed out `r31, r30, r29, …`
(`W-REGALLOC-1`), so *"which formal got `r31`"* is *"which candidate was
coloured first"* — and that inference is itself a dependency on `W-REGALLOC-1`,
recorded here rather than assumed.

## Environment control — pinned by NAME, re-run per batch

`fixtures/cpp/w5_chain.cpp` → **`4/4 functions in class`**. Logged at the head
of every batch. A batch whose control does not print 4/4, or whose differential
runs in 0.00 s, is **void**.
