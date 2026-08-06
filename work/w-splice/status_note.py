#!/usr/bin/env python3
"""status_note.py — append lane w-splice's paragraph to STATUS.md's retraction
chain, in the nesting the chain already uses.

Lane w-splice scratch. `docs/STATUS.md`'s generated metric block is regenerated
by `scripts/status.sh --write` and is NEVER hand-edited; this touches only the
prose chain that records what each instrument widening and each mechanism
retracted.
"""

ANCHOR = "> > > [`rungs/2026-08-07-w-fix.md`](rungs/2026-08-07-w-fix.md).\n"

NOTE = """> > >
> > > > **2026-08-08 — 723 MORE ARE CLOSED, and this time it is the OTHER
> > > > mechanism.** Everything above is mechanism **E**, the call c2 drops
> > > > because its callee does nothing. Lane `w-splice` shipped mechanism
> > > > **I** — c2 *expanded* the callee — as `crates/c2-core/src/splice.rs`:
> > > > when the port's whole emitted body for a function is one call to a
> > > > same-TU callee the port lowers, the function's `/Gy` COMDAT **is that
> > > > callee's body**, relocations included, with no branch and no REL24
> > > > against the callee. `fnbyte-differs` **3,195 → 2,472**, `fnbyte-exact`
> > > > **35,982 → 36,705**, **0 functions moved the other way** per
> > > > `(TU, emit_name)`, and the rule fired **723** times with **723 of 723**
> > > > byte-exact. `mismatch` 0, `functions()` untouched, 72 of 80
> > > > `gap-metric` lines byte-identical.
> > > >
> > > > **Three things to carry off it.** (1) **The FBM partition was not the
> > > > alarm that mattered.** This mechanism replaces a caller's relocations
> > > > with its callee's, and FBM compares a `.text` COMDAT's raw bytes, which
> > > > do not contain relocations (**#882**, 4,664 credited functions). A
> > > > per-symbol relocation check against the reference obj found **150**
> > > > wrong targets in the first shipped version, **77** in the second and
> > > > **1** in the third — every one of them scored `exact` by FBM — and each
> > > > round changed the rule. The tip is **723 of 723 verified, 0
> > > > disagreements**, and `fnbyte-exact-relocated` reads 4,664 at both ends.
> > > > (2) **Mechanism I is a FIXPOINT too** (#989): c2's body for a caller two
> > > > links above a lowerable callee is the *end's* body. (3) **#925's caution
> > > > again** — 245 distinct symbols across 284 TUs, but **three** template
> > > > roots and **87 %** of them `??0?$_List_iterator`. Boards **#986**–**#995**;
> > > > [`rungs/2026-08-08-w-splice.md`](rungs/2026-08-08-w-splice.md).
"""

p = "docs/STATUS.md"
s = open(p).read()
assert ANCHOR in s, "the w-fix block's last line moved"
assert "w-splice" not in s, "already appended"
open(p, "w").write(s.replace(ANCHOR, ANCHOR + NOTE, 1))
print("appended")
