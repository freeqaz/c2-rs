# wb-reader — ROUND-2 PREREG (post-hoc refinement, registered before the run)

Round 1's four `NOOBJ` predictions (A2, A3, B2, C2) all came back `DIFF`.
**c2 does not ICE on a desynchronised `.ex` operand stream** in these positions —
it decodes whatever the shifted bytes happen to say and emits an obj. So
`{IDENT, DIFF, NOOBJ}` cannot separate a same-class substitution from a
cross-class one, and round 1's grid is scored as-is (four misses, all optimistic,
all in the same direction) rather than reinterpreted.

Round 2 replaces the outcome code with one that **can** separate them. It is a
post-hoc design and is labelled as such; it is registered here, in a committed
file, before the first replay of the round.

## The metric

For a mutant obj against the baseline replay obj, over `.text` COMDAT leaders:

* **`Δleaders`** — size of the symmetric difference of the leader-name sets.
* **`Δbodies`** — number of leaders present in **both** whose section raw bytes
  differ.

## The claim being tested

The `.ex` operand grammar is a **per-opcode class table** (`DAT_10b25e48`,
`0x10b3d626`), 29 classes (`0x10b3d631`), dispatched through the jump table at
`0x10b3d954`. A substitution between two opcodes **of the same class** leaves
the token stream aligned; one **across classes** shifts every following byte of
that function's segment.

A `.ex` segment is per-function, and the label-token stream feeds the TU-wide
`$M`/`$T` counter (`docs/LABEL_COUNTER.md`), so a desync is expected to reach
past the edited function.

## Frozen cell predictions

| cell | edit | class(before → after) | prediction |
|---|---|---|---|
| R2-A0 | `1F` → `1F` | 0 → 0 | `Δleaders = 0`, `Δbodies = 0` |
| R2-A1 | `1F` → `20` | 0 → 0 | `Δleaders = 0`, `Δbodies = 1` |
| R2-A1b | `1F` → `23` | 0 → 0 | `Δleaders = 0`, `Δbodies = 1` |
| R2-A2 | `1F` → `27` | 0 → **1** | `Δleaders > 0` **or** `Δbodies > 1` |
| R2-A3 | `1F` → `26` | 0 → **2** | `Δleaders > 0` **or** `Δbodies > 1` |
| R2-B1 | jump token → another label token of the same body | width-preserving | `Δleaders = 0`, `Δbodies = 1` |
| R2-B2 | jump token bytes swapped (`lo hi` → `hi lo`, `lo ≥ 0x80`) | sets the `varU` bit-15 continuation → **+2 bytes** | `Δleaders > 0` **or** `Δbodies > 1` |
| R2-C1 | `27`'s TYPE class nibble `43` → `41` | width-preserving | `Δleaders = 0`, `Δbodies = 0` (already `IDENT` in round 1; carried as the positive control for "a type nibble a `27` never classifies") |
| R2-C3 | `27`'s TYPE tag `A6` → `26` (clear bit 7) | 2-byte type word → **1-byte** short form, then the debug skip re-reads → **−2 bytes** | `Δleaders > 0` **or** `Δbodies > 1` |

**Decline clause.** If R2-A1 and R2-A1b (the same-class cells) do **not** come
back `Δleaders = 0, Δbodies = 1`, the metric itself is uninformative — a
one-token change would be perturbing the whole obj anyway — and the whole round
is reported as *no discrimination available*, not as evidence either way.
