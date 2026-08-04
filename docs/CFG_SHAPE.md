# The CFG step — byte-level characterization

Lane **w-cfg**. This is the specification the control-flow step is meant to be
built from: how the `.ex` encodes control flow, **what `c2` emits for each
shape**, the minimal instance to build first, and what a block/instruction IR
must carry to serve it. Every byte below is transcribed from an obj produced by
the real `cl.exe` 16.00.11886.00 / `c2.dll` under wibo, or from a `.ex` captured
at the same flags. Read-only lane: **no file under `crates/` was touched.**

The IL half is largely a **confirmation** of `docs/IL_STMT_GRAMMAR.md` §7–§9 and
of the decode-only scanner in
`crates/c2-il/src/func/body/shapes/control_flow.rs`; §2 marks every claim as
confirmed or new. The **emission half (§3–§4) is new** — nothing in `docs/`
states it today.

Control for this lane is
[`rungs/_2026-08-04-w-cfg-prereg.md`](rungs/_2026-08-04-w-cfg-prereg.md),
committed at `eefc229` **before the first capture**, with 21 predictions each
carrying a named rival reading. It is scored verbatim in §1 and the wrong ones
stay on the page.

Companion docs: `docs/IL_STMT_GRAMMAR.md` (the statement layer),
`docs/CODEGEN_W6_COMPARE.md` (the comparison *value* spines, which this document
shows are a **different** family from a comparison feeding a branch),
`docs/OBJ_DYNINIT_SHAPE.md` (the obj shell and the external-branch encoding this
document contrasts intra-function branches against), `docs/LABEL_COUNTER.md`
(the `$M`/`$T` counter, which §3.6 shows does **not** move with block count).

---

## 0. The headline, before the tables

1. **An `if` in the IL does not reliably become a branch in the obj.** c2 folds a
   large fraction of `cflow-if-1` bodies into *branchless arithmetic* or into a
   **conditional return** (`bclr`), emitting no label and no displacement at all.
   Of the seven `cflow-if-1` leaf probes in `pa.cpp`, **six** emit no forward
   branch; of the two real `cflow-if-1` functions in the frontier TU
   `src/system/utl/Pool.cpp`, **both** fold to `beqlr cr6`. An implementer who
   builds a branch lowering and grades it on `Pool.cpp` will grade nothing.
   §3.5 gives the measured fold table and says plainly that the *decision* is a
   c2 cost model this lane did not crack.

2. **The cell that does branch, and is the one to build first, is real and
   small.** `?MemFree@NUISPEECH@@YAXPAX0K@Z` in
   `src/xdk/nuispeech/xboxmem.cpp` — a frontier TU — is 0x24 bytes, nine
   instructions, one `bc`, two `REL24`, no frame, no `.pdata`. §4 specifies it
   byte for byte.

3. **The branch target is a plain self-relative displacement and carries no
   relocation.** `bc` stores `(target − addr)` in bits 16..31; an
   **intra-section** `b` stores the true relative displacement
   (`d_cold`: `48000008` at 0x38 → 0x40). This is the exact opposite of the
   **external** `b`, which stores a section-start-relative word and takes a
   `REL24` (`docs/OBJ_DYNINIT_SHAPE.md` §3.3). Same opcode, two encodings,
   discriminated by whether the target is inside this section. §3.3.

4. **Condition registers are two-valued, not one.** An explicit `cmpw`/`cmpwi`/
   `cmplwi` feeding a branch always writes **cr6** — reused, never allocated,
   confirmed across three sequential compares in one body. But a **record-form**
   instruction (`addic.`) writes **cr0**, and c2 branches on cr0 there. A
   lowering that hard-codes cr6 emits wrong bytes for every decrement-and-test
   loop. §3.2.

5. **Block order is the IL's statement order — in 10 of 11 measured cells, and
   refuted in the eleventh.** `d_join` (`if(a) r=gi(1); else r=gi(2);`) is
   tail-merged into a single `bl` with the argument selected by the branch, and
   the layout inverts. §3.4 states the rule and the refutation together.

6. **Loops are rotated, and leaf counted loops become CTR loops.** Every back
   edge measured is a **conditional** branch — never the IL's unconditional
   `3A TOP` — and a leaf loop with a compile-time trip count is lowered to
   `mtctr` + `bdnz` (`BO=16, BI=0`), which is a different instruction family
   from anything in the port today. §3.7.

7. **The long-branch expansion is measured, not assumed.** At a displacement of
   +32628 bytes c2 emits a direct `bne`; at +34148 it emits `beq cr6,+8` over an
   unconditional `b`. The switch is at the architectural 14-bit `BD` limit
   (±32764), with no slack. §3.3.1.

8. **Flag provenance, measured as a control.** `c2rs capture` hardcodes
   `/Ox /GS- /c` and silently ignores flags
   (`crates/c2-reference/src/lib.rs:465`). Every `.ex` this document quotes was
   re-captured through `c2rs census --flags-file`, and the control in §10.1
   shows the on-disk bundle reproduces byte-for-byte from that path while the
   `/Ox` capture's `.ex` **differs** — in exactly 7 bytes, one per function, the
   per-function optimization word (`0x00a00005` → `0x00200005`). **This bounds
   the exposure for this lane's measurements only. It clears no other lane's
   captures.**

9. **What this document does not contain: an optimizer.** The port is
   I/O-behavioral. §3.5's fold table and §3.4's tail-merge refutation are
   recorded as *required emission rules for the accepted class* and as *reasons
   to keep the class narrow* — never as passes to reproduce in general. §8 is
   the list of what is still unbuildable, and it is long on purpose.

## 1. Pre-registration, scored

## 2. How the IL encodes control flow

## 3. What c2 emits

## 4. The minimal instance — `cflow-if-1`, in full

## 5. The widening order, ranked by TUs

## 6. What the block/instruction IR must carry

## 7. The `/FAsc` listing as a decode aid

## 8. What an implementer still cannot build from this document

## 9. Proposed board rows

## 10. Reproducing this
