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
