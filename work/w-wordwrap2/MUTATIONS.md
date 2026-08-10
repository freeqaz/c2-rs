# w-wordwrap2 — THE MUTATION GRID

Every cell is graded by the **differential against real `c2.dll` under wibo**, at
`/O1 /Oi /GS- /c`, over the thirteen `fixtures/cpp/wwrap_*.cpp` and
`fixtures/cpp/wwbss_*.cpp` cells this lane and `w-wordwrap` own between them.
Apparatus: `work/w-wordwrap2/mutate.py` (one conjunction deleted per mutation,
anchor uniqueness asserted), `work/w-wordwrap2/mutrun.sh`,
`work/w-wordwrap2/mut/GRID_RAW.txt` (every run, including the three that came
back green).

**The verdict that matters is `mismatch`** — a deletion that admits a `_neg`
cell emits an obj real c2 disagrees with, and that is the one word the
correctness rule is written in. One mutation grades by **PANIC** instead, and
that is louder rather than weaker.

---

## The control, RUN and not asserted

| | deletion | result |
|---|---|---|
| **M0** | restore `w-wordwrap`'s `return None` on `IlDataDef::uninitialized` | **both accepted cells fall back to `codegen-gap`** |

So `wwrap_gstore.cpp` and `wwbss_two.cpp` are accepted **by this production
and by nothing else**. Without it the lane could be crediting itself with an obj
some other path emits.

---

## The graded cells

| # | conjunction deleted | cell | base → mutated |
|---|---|---|---|
| **M1** | Rule S1′'s **linkage** clause — *the object must be EXTERNAL* — in **both** crates | `wwbss_static_neg` | refused → **`mismatch`** |
| **M2** | board **#184**'s object-count bound (`MAX_OBJECTS_PER_SECTION`) | `wwrap_gstore_widths` | refused → **`mismatch`** |
| **M3** | Rule **Y1**'s external clause — the symbol group is the **reverse** of the walk | `wwbss_two` | match → **`mismatch`** |
| **M4** | Rule **B1** — the section nibble is the **max** over the objects | `wwbss_two` | match → **`mismatch`** |
| **M5** | Rule **S1′** slot `B` — the section goes **between** the watermarks (index 3, not 4) | `wwrap_gstore` **and** `wwbss_two` | match → **`mismatch`**, both |
| **M6** | the **unconditional** dangling-def test, in both crates | `wwbss_static_neg` | refused → **3 PANICS** |

**Six graded of seven attempted.**

### M2 is the sharpest, because its two cells disagree

Deleting #184's bound turns `wwrap_gstore_widths.cpp` into a **`mismatch`** and
`wwbss_three_neg.cpp` into a **`match`**. Both are three-object TUs. That is
`OBJ_DATA_BSS_SHAPE.md` §8.1's *"38 of 62"* reproduced live in two cells: at
three objects the walk is right *sometimes*, and a bound that let the right ones
through would let the wrong ones through with them. A grid with only the
`three_neg` cell would have reported the bound as unnecessary.

---

## The three that came back GREEN, and what each one taught

Recorded beside their repairs rather than replaced by them (#2726's rule).

### M1's first run — the two crates are ONE conjunction over this cell

Deleting only the **reader's** `!o.external` came back green, because the
**writer** re-asserts it. *"Neither crate may assume the other ran"* is a real
property of the code and it is also an **over-fence** for a mutation aimed at one
half. #2665's shape exactly. The merged deletion takes both and the cell goes
`mismatch`.

### M4's first run — the mutation coincided with the rule ON THE ONLY CELL

`section_nibble(&bss_refs[..1])` — *"take the FIRST object's nibble"* — came back
green, and the reason is a property of the **cell**, not of the clause: this TU's
`.gl` record order puts the **8-byte** object first, so *first* and *max* are the
same number here. Re-aimed at the **last** object, where they are 3 and 4, and
the cell goes `mismatch`.

**The generalisation is worth more than the repair.** A must-fail mutation is
only a test of the clause if the cell can *tell the two readings apart*, and
nothing about writing the mutation reveals that it cannot. This is
`docs/STATUS.md` trap 0 — *a green control is a statement about the population it
ran over* — with the population being one object's position in one `.gl` stream.

### M6's first run — the two REPAIRS are one conjunction, same as M1

Re-gating the writer's dangling-def test alone came back green, because the
reader's separated refusal now catches the cell first. Merged, the panic returns:
**3 panics**, `every relocation target got a symbol`. That is the defect this
lane shipped for one commit and it is graded rather than described.

---

## The cell that grades NOTHING, NAMED and not counted (#2698)

**M7 — the `__declspec(thread)` clause.** `wwbss_tls_neg.cpp` is
`vocab-gap` at **`unclaimed-gl-symbol`** before and after the deletion:
`IlBundle::functions()` refuses the TU because nothing claims the `.tls$`
object's `.gl` name, and it refuses it whether or not this lane's clause is
there. The clause is **structurally ungradeable by a fixture at this tip** —
storing to a thread-local from an in-class body is itself out of class
(`shape-token-unresolved`, measured on the first draft of that file), so there is
no source shape that reaches the clause.

It is **kept**, because `w-sect` measured the record it exists for
(`__declspec(thread) int t1;` is byte-identical to a plain `.bss` object in every
field a reader had before the flag byte was found) and this lane gave that
reader its first function-bearing caller. A clause no mutation can break is not a
clause any mutation has graded, and saying so is the point.

The fixture stays as the compiled record of what c2 emits for the shape.
