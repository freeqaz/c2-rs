# W22 — the int-like operand type by spelling

    Tag:       W22
    Slug:      int-like-spelling
    Date:      2026-07-30
    Fixtures:  w22_int_spelling.cpp w22_int_spelling_neg.cpp
    Census:    402,704 → 418,628 (16.35 % → 17.00 %), +15,924
    Record:    docs/ROADMAP.md §6d

`eat_int_like` matched an exact four-triple whitelist, so a width-4 integer
carrying a per-TU type id — an `enum`, a `typedef`, a `const`/`volatile`
qualification — refused even though `is_int4_type` admits it on the tag/kind
nibbles and c2 emits the identical instruction. It now falls through to that
predicate.

The recorded estimate was wrong by 2.8×; it was re-measured before building,
which is why the rung was taken. Mismatch 0, no TU changing class,
census/gate disagreement 0. Full record, including the gate table and the
found-and-not-taken list, in `docs/ROADMAP.md` §6d.
