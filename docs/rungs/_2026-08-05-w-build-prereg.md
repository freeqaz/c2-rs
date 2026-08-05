# w-build — PREREG: the UNION over frontier chain inventories as a coverage ranking, and a registered selection rule for what to build from it

    Tag:       w-build
    Slug:      w-build-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg. It admits no shape and ships no
               lowering, so there is nothing an obj-graded fixture could grade.
               The findings record names whatever fixtures the build earns.
    Census:    not measured yet by design. The baseline this lane registers is
               in §1 and is the brief's, to be reproduced before any change.
    Lane:      w-build, worktree `wt-w-build` off master **`4d6aa58`**.
    Record:    this file. The findings record is
               `docs/rungs/_2026-08-05-w-build.md`.

---

## 0. What this lane is for

`w-depth` (#660–#669) built a per-TU **construct inventory** — the ordered set of
expression-layer opcodes a frontier TU's parse chain meets — and refuted the
*count* over it as a selector while establishing the *content* as the thing that
retrodicted the one conversion the project has (#666).

Eight lanes have refuted eight rankings over blocker keys. The brief sends this
one to try the **ninth**: rank operators by how many frontier TUs' inventories
contain them — a **union / coverage** ranking rather than a first-refusal label —
and build the head.

**Then build something and measure it in both units.**

---

## 1. Baseline, to be reproduced before any change

```
match 9 · mismatch 0 · codegen-gap 0 · vocab-gap 862 · capture-fail 7
A/B/C/D/E = 28 (LO 27)/338/169/9/2 · A∧B∧C 27 · FRONTIER 18
FBM 0.16654 · fnbyte-exact 29,801 · fnbyte-partial 9,375 · fnbyte-differs 0
cargo test --workspace --release: 852 passed / 0 FAILED / 27 targets
scripts/gate.sh --jobs 6: PASS 18/18, 4,536 verdicts; sweep 16,394 / 16,298
  graded / 96 ungraded; cross 75,829 of 76,217 / 388 ungraded; 0 mismatch
```

**Registered before the run**, per the bar: gate verdicts **4,536**, and it grows
by exactly 18 per fixture — so a lane that lands *N* fixtures must see
`4,536 + 18N` and nothing else.

---

## 2. The INPUT, measured from `w-depth`'s published artifact and not by me

`work/w-depth/run2/chain.json`, the post-`0x35`-pin walk over the 18 frontier
TUs. For each TU I take the **`sinks`** list (the scaffold-subtracted chain) and
the **`also_at_exit`** list (operators named in that TU's round-0 blockers even
though the walk left `parse_expr` immediately). The union ranking is

    coverage(op) = |{ T in FRONTIER : op in sinks(T) or op in also_at_exit(T) }|

and its head, computed before this file was committed, is

| op | UNION | sink | exit-only | what it is |
|---|---:|---:|---:|---|
| `op:26` | **12** | 11 | 1 | call-in-expr |
| `op:38` | **10** | 9 | 1 | brfalse |
| `op:30` | **8** | 6 | 2 | indirect load |
| `type` | 7 | 6 | 1 | the operand-TYPE gate |
| `op:32` | 6 | 6 | 0 | indirect store |
| `op:27` | 6 | 6 | 0 | byte-offset add |
| `op:28` | 6 | 5 | 1 | byte-offset add, subscript form |
| `op:1F` | 6 | 5 | 1 | cmp-eq |
| `op:BD` | 5 | 5 | 0 | the CALL token |
| `convert` | 5 | 5 | 0 | the `2C` target-type gate |

This is an **input**, not a result: it is arithmetic over a file another lane
committed. Everything below it is the prediction.

---

## 3. Registered predictions

### R1 — the baseline reproduces, every digit, before any change

Named in §1. A lane that cannot reproduce the baseline has no before-number.

### R2 — the union ranking DOES NOT retrodict either

Rebuild the identical union over `work/w-depth/retrodiction-6dcb3f4.json` — the
19 TUs that were the frontier at `6dcb3f4`, master before `w-tu1`. `w-tu1` built
the **W43** class `((P != 0) << SH) | C`, whose operators are `op:20` (cmp-ne),
`op:09` (shl), `op:0C` (bit-or).

**I predict `op:09` reads coverage 1 and ties for LAST.** #666 already states
`op:09` appears in exactly one frontier chain; if that survives onto the
retrodiction tree, then the union ranking puts the operator that actually
converted a TU **at the bottom of 19**, which is a worse retrodiction than
depth's 11th-of-19 and a much worse one than byte fraction's 1st-of-19.

**If instead `op:09` lands in the union's top 5, the ranking retrodicts and I
build it.** That is the falsification condition and it is checkable in one
command.

### R3 — the union ranking is INVARIANT across the one conversion we have

Top-3 by coverage at `6dcb3f4` is **set-identical** to top-3 at `4d6aa58`. A
selector that does not move when the frontier loses the TU it was supposed to
have selected is not reading the frontier.

### R4 — every (op, TU) pair in the union is a CERTIFIED fall-through

`chain.py` sinks a key and re-scans; a step exists **only because** closing it
revealed another blocker. So I predict **0** of the union's (op, TU) pairs has
that op as the TU's *terminal* refusal — that is, no operator in the inventory
is ever the last thing standing between a frontier TU and conversion. If this
holds it is not a correlation but a **property of the instrument**, and it means
the union ranking cannot in principle name an operator whose construction
converts a frontier TU.

I register the complement honestly: the 2 CLEAR TUs (`Primes`, `jsonwriter`) do
have a last step, so the strict claim is **0 of the 16 non-CLEAR TUs**, and for
the 2 CLEAR ones the last step is a *necessary* member of a 7- and an 18-member
conjunction, not a sufficient one.

### R5 — WORKLOAD MASS of the bitwise/shift family is small

`expr-shl`/`expr-shr`/`expr-bit-and`/`expr-bit-or`/`expr-bit-xor` (`09`, `0A`,
`0B`, `0C`, `0D`) as **first-refusal** keys over all 878 TUs: I predict the five
combined are **≤ 5,000 blocked functions**, i.e. below `expr-op-0x27`'s 23,090
by more than 4×. Registered because the brief's warning is that *mass is not
even a screen* — so I want the mass on the page whichever way the build goes.

### R6 — the SELECTION RULE, registered before the histogram is read

Walk the union ranking top-down and build the first operator that clears all
three:

* **(a)** it is not a member of a family already measured at **zero TUs** —
  which excludes the relationals `1F`–`24` (#420), the branch/control-flow
  skeleton `38`/`39`/`29`/`3A` (#440), and the byte-offset add `27`/`28`
  (#364/#622);
* **(b)** it is a widening of the **expression layer** that a lane can ship —
  not an entry into a whole other subsystem;
* **(c)** its lowering is decidable by real `c2` over the **full cross product**
  of the axes it has, not one witness. (The single-cell trap has fired five
  times.)

Everything the walk refuses gets a number beside it in the findings record.

### R7 — the OUTCOME, registered in both units

**`fnbyte-exact` moves up; TU match stays at 9; `fnbyte-differs` stays 0.**

Seven of the last eight lanes correctly predicted no TU movement. I predict the
same and I predict it for R4's reason rather than by induction: whatever I build
is a member of at least one 5-to-18-member conjunction on every frontier TU that
wants it.

I register a **quantitative** version so it can miss: **`fnbyte-exact` +1 to
+800**, and `+0` scores a MISS.

### R8 — TU match ends at 9

---

## 4. Where I expect to be wrong

R5. The brief's own history says mass surprises — `expr-op-0x27` is 23,090
functions and worth **zero** TUs, and `expr-cmp-eq` was the #12 key by mass and
still a fall-through. A family I am guessing at ≤5,000 could as easily be 40,000
or 200. Registering a number I have not seen is the only way that guess costs me
anything.

And R7's lower bound. `+0` is a real possibility: five merges of codegen work
moved **zero** chain steps (#669), and a widening that the *rest* of each body's
conjunction still blocks emits nothing new at all.
