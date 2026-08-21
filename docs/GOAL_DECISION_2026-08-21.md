# The goal question, answered — 2026-08-21

`STRATEGY_REVIEW_2026-08-13.md:251` recorded that the project's goal question
was *"currently owned by nobody."* It stayed unowned for eight days, during
which the seven-lens architecture review (`docs/ARCH_REVIEW_2026-08-21.md`)
found it was the hinge on which step 5's entire justification turned: step 5
is a **wrong trade** against the verifier-throughput thesis and the **right
and only trade** against full reproduction. The review's consequence 1 said
so, and gated the port program on the owner re-owning it.

**Decided by the project owner, 2026-08-21.** Quoted, because the wording
carries the ranking:

> *"the goal is: the perfect reproduction that gives us a clear understanding
> of the MSVC internals, to help us with decomp, and also to get parity so
> that we have a 100% open source implementation."*

Two ends, ranked equally: **(1) perfect reproduction as a route to
understanding MSVC's internals, in service of decomp; (2) parity — a 100%
open-source implementation.**

## What this settles

**Option A, decisively** (`STRATEGY_REVIEW_2026-08-13.md` §8.1's framing).
The `match` → 870/878 number is the scoreboard. Step 5 — porting the middle —
is therefore the right trade and, per the cost/benefit lens, the *only* one:
the middle is needed by **843 of 845** refused TUs and nothing substitutes for
it.

## What this retires

The **verifier-throughput thesis**, which `CLAUDE.md` and `README.md` both
carried as *the* thesis until this decision. It is now demoted to a property.

That retirement also **moots the entire economic case against the port**,
which was the strongest negative finding in the review. `ARCH_REVIEW` §7's
measurements stand as measurements and are worth keeping — the sole consumer
is source-space and capped at **≈2.4×** even with an infinitely fast c2; its
wall-clock bound has moved off compilation to generation; the shipped hybrid
prefilter has never been enabled; work-weighted coverage is 46 of 162,147
emitted functions — but they were an argument against a goal this project does
not hold. **No lane should be declined on those grounds again**, and the
2026-08-13 NO-GO's reopen tripwire is superseded rather than satisfied.

Note the asymmetry this creates, and do not lose it: throughput arguments can
no longer *justify* work, and they can no longer *forbid* it either.

## What this promotes, and it is the non-obvious half

**Characterization becomes a first-class deliverable.** Under the old thesis,
`docs/STEP5_PRICING_2026-08-21.md`'s headline — *"per-stage observability buys
CHARACTERIZATION, not a per-stage differential grade"* — read as a downgrade,
and the review reported it that way. Under goal (1) it is a direct hit: a
mechanised, addressed account of what c2's middle actually does **is** the
product, whether or not a ported pass can be graded against it today.

Consequences for how lanes are chosen:

- Characterization lanes stop needing a conversion story to justify
  themselves. Predicted reach 0 is not a mark against a lane whose output is
  the understanding.
- `docs/whitebox/` is product, not provenance overhead. This was already the
  standing position (`CLAUDE.md` § "Whitebox analysis is AUTHORIZED", owner,
  2026-08-17) — goal (1) now supplies the *reason*, and the two agree.
- **`w-ildecode` is directly on-goal**, not merely de-risking. It was
  dispatched to document the two opaque middle interfaces from the binary;
  under goal (1) that documentation is a deliverable in its own right.

## What does NOT change

- **The one correctness rule.** Real `c2` under wibo plus a byte-exact obj
  compare remains the sole judge. Goal (1) is about understanding the
  internals; it is **not** a licence to grade the port against c2's internal
  state, and `ARCH_REVIEW`'s probe C measured that the port→tuple projection
  is undefined anyway.
- **The port stays I/O-behavioral.** Understanding c2's internals does not
  mean reproducing its instruction bytes — that is still the wrong artifact.
- **`docs/PROGRESS_METRIC.md`.** A wrong emit still scores strictly below the
  refusal it replaced. Parity is coverage, and coverage is not bought by
  guessing.
- **Two-sided pricing of every new fence** (#1042, NC-5/#2691). Unchanged.

## Still open — the owner's, and not answered by this decision

`ARCHITECTURE_PROPOSAL_2026-08-20.md` §8 **decision 0**, added by
`w-archamend`: approve the new rows **4a** (the integration prerequisites — a
general op-level IL decode and a general lowering to `coff::Function`, priced
two-sided at **15–45 engineer-months** as a lower bound under CEILING §5's
~5:1 calibration) and **4b** (give IR3 its own step, defined in c2's
tuple/region coordinates), or declare step 5 **characterization-only**.

The goal decision constrains it but does not settle it: parity (goal 2) is
unreachable without 4a, since a ported pass with no route to `coff::Function`
cannot move a single obj byte. Goal (1), by contrast, is served by
characterization alone. So the live question is no longer *whether* the port
is worth doing — it is whether to fund 4a now or bank understanding first.
