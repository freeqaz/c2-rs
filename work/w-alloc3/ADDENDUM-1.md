# w-alloc3 — PREREG ADDENDUM 1

    Written:   after `gen_grids.py` wrote both grids and after
               `work/w-alloc3/gridH.sha256` was committed at `5832dd14`, and
               **before the first `cl.exe` ran on either grid**. No obj of
               either grid existed when this was written.
    Why:       §2 of `PREREG.md` lists eight domain clauses. Writing the grader
               made a NINTH necessary, and it is necessary for a reason that is
               the whole point of the lane, so it is registered rather than
               patched in afterwards.

---

## A1.1 D9 — the substitution must be a valid RENAMING, or RULE BIND refuses

BIND rewrites the callee's **source** register fields and leaves its
**destination** fields alone. On a body with one live value that is always
sound. On a permuted binding it need not be:

```text
    int g(int a, int b, int c) { return a - b + c; }
    int f(int x0, int x1, int x2) { return g(x1, x2, x0); }   // H-perm-120
```

`g`'s own body writes its result into `r3` at the first instruction. In `f`,
`r3` holds `x0`, which is `g`'s **third** formal and is read by the *second*
instruction. Substituting sources and keeping the destination would emit a body
that destroys a value it later reads. c2 cannot emit that, so c2 must **choose a
different register** — and choosing a register under pressure is exactly the
question the six dead keys died on.

> **D9 (CLOBBER).** For every instruction of the callee's body, if its
> destination register still holds a caller formal value that a later
> substituted source field reads, the cell is **OUT OF DOMAIN** with clause
> `D9-clobber`. RULE BIND predicts nothing there.

This is decided mechanically from the callee's own bytes and from `β`, before
any prediction is compared, and it is decided identically on GRID-A and GRID-H.

**This is a narrowing, and it is registered as one.** It removes from RULE
BIND's reach precisely the cells where a real allocator has to run. A rule that
answered them would be a seventh candidate for the graveyard; a rule that
refuses them is the incumbent's own discipline applied one level up. The cells
D9 removes are **printed with their count**, not dropped, so the shrinkage is
visible and the rung can be read as "RULE BIND is narrow" rather than as
"RULE BIND is general".

## A1.2 Registered consequence for the holdout

At least some of GRID-H's six `H-perm` cells and both `H-perm4` cells are
expected to fall out under D9. **The holdout's in-domain size is therefore not
known when this is written**, and if it falls below **20 in-domain graded
cells** the lane takes outcome (2) — 0 wrong on a population too small to
decide, shipping nothing and naming the grid that would decide it. That
threshold is registered here, before the number is known.

## A1.3 Registered consequence for the decoder

The grader decodes the callee's words with a **fail-closed** table: any primary
opcode or extended opcode not in the table makes the cell out of domain with
`decode-unknown:<hex>`, rather than guessing which fields are registers. A
decoder that guessed would manufacture predictions, which is board **#889**'s
shape — a population *mapping* rather than a matcher manufacturing six
refutations.

## A1.4 What is NOT changed

The prediction rule of `PREREG.md` §3, the six rivals of §1.2, the nine
registered predictions of §5 and the decline floor of §6 are untouched.
