#!/usr/bin/env python3
"""crateacc.py — the WIDENED crate acceptance, as a transcription, patched into
`work/w-emitp2/strictin.py` so w-emitp2's own `scan2.py` / `two_readers.py` run
against the reader that ships today with **nothing else changed**.

`strictin._crate_verdict` transcribes `crates/c2-il/src/func/ininit.rs`'s
`read_elements` acceptance.  Before `w-inread` that was:

    first element must be 01 or 02 ; every element must be 01 or 02 ;
    scalar type in (01, 02) ; scalar width in (1, 2, 4)

and after it is the same anchor with three more element kinds:

    element tag 03 (inline bytes)     — read, contributes its payload
    element tag 08 (zero fill)        — read, contributes <count> zero bytes
    scalar type 03 / 04 at width 4    — read, a pointer-valued plain integer

**The anchor set is UNCHANGED and that is deliberate** (see the `#961` comment
in `in_scalar_initializers`): a record whose FIRST element is a tag-03 blob or
a tag-08 fill is still not anchored, so it is still in neither `records` nor
the residue — it is now *counted*, in `InInitReport::unanchored`.

This file changes ONE function.  Everything else in the reproduction chain —
`U`, `Seed`, the skips, the closure operators, the alias table, both truths, and
`strictin`'s own sequential framing — is w-emitp2's by value, so any movement in
the CRATE column is the reader and nothing else.

**It is a transcription and it is graded as one.** `two_readers.py` reconciles
it count by count against `crates/c2-il/tests/in_init_probe.rs`, the shipping
reader's own cursor; neither is the other's witness.

    import crateacc     # patches strictin in place; nothing else to call

stdlib only.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT", os.path.abspath(os.path.join(HERE, "..", "..")))
sys.path.insert(0, os.path.join(MAIN, "work", "w-emitp2"))
import strictin  # noqa: E402

SYM = strictin.SYM
#: `ininit.rs`'s `TYPE_INT_SIGNED` / `TYPE_INT_UNSIGNED` / `TYPE_PTR_DATA` /
#: `TYPE_PTR_FUNC`.
SCALAR_TYPES = (0x01, 0x02, 0x03, 0x04)
#: `ininit.rs`'s `WIDTHS`.
INT_WIDTHS = (1, 2, 4)
#: `ininit.rs`'s `POINTER_WIDTH` — the only width types 03/04 are measured at.
PTR_TYPES = (0x03, 0x04)
PTR_WIDTH = 4
#: `ininit.rs`'s `ELEMENT_INLINE_BYTES` / `ELEMENT_ZERO_FILL`.
ELEMENT_KINDS = (0x01, SYM, 0x03, 0x08)


def crate_verdict_widened(elems):
    """(accepted, first_tag, why) under the WIDENED `ininit.rs` acceptance."""
    if not elems:
        return False, None, "empty"
    first = elems[0][0]
    # The anchor is still `00 01` / `00 02` only.
    if first not in (0x01, SYM):
        return False, first, "UNANCHORED"
    for k, a, w in elems:
        if k in (SYM, 0x03, 0x08):
            continue
        if k != 0x01:
            return False, first, "unknown-type"
        if a == 0x05:
            return False, first, "floating-point"
        if a not in SCALAR_TYPES:
            return False, first, "unknown-type"
        if a in PTR_TYPES:
            if w != PTR_WIDTH:
                return False, first, "pointer-width"
        elif w not in INT_WIDTHS:
            return False, first, "unknown-width"
    return True, first, None


strictin._crate_verdict = crate_verdict_widened
