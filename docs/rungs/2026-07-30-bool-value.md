# W26 — the one-byte-unsigned value class

    Tag:       W26
    Slug:      bool-value
    Date:      2026-07-30
    Fixtures:  w26_bool_value.cpp w26_bool_value_neg.cpp
    Census:    442,273 → 464,584 (17.96 % → 18.87 %), +22,311
    Record:    docs/ROADMAP.md §6i

`bool` and `unsigned char` share the operand type `82 12`, and inside the class
a value costs no instruction: `li r3,k`, a bare `blr`, or the W18 register move.
The refusals are the conversions *out* of the class, which are a real `rlwinm`,
and the other one-byte class — `char`/`signed char` are `82 11`, and a signed
narrow value parts company from an unsigned one exactly one token later.

Mismatch 0, disagreement 0. Swept by `scripts/sweep.d/90-bool-byte.py`, whose
axes are (spelling) × (literal value) × (argument slot) crossed against those
refusals.
