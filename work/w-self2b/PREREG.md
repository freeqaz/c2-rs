# w-self2b — PREREGISTRATION

    Lane:    w-self2b, worktree branch `wt-w-self2b`.
    Base:    master `a620ec61` (= `1659c3ec` + the STATUS regen commit).
    Written: BEFORE this lane compiled, captured or disassembled anything.
             The only measurement in it is a re-read of four lanes' already
             committed tables and of `work/w-mixed/ildiff.out`.
    Ships:   see §7. The expected answer is "nothing in `crates/`".

---

## 0. The incumbent, and the decline floor

**The incumbent is the shipped refusal in `crates/c2-core/src/codegen/alloc.rs`.**
A run mixing a constant producer and a register-derived producer is refused. On
every population any lane has built it is **wrong on 0 cells**:

| population | cells | refusal WRONG | best rival |
|---|---:|---:|---|
| `w-alloc2` mutate | 81 | **0** | w-next's key, 20 wrong |
| `w-refbind` holdout | 72 | **0** | `H-self`, 11 wrong |
| `w-seam` narrow lift | 36 | **0** | strict clause 1, 12 wrong |
| `w-ilx` GRID V | 45 | **0** | `KEY ILX`, 14 wrong |
| `w-alloc3` GRID H | 38 | **0** | `RULE BIND`, 5 wrong |
| `w-mixed` GRID M | 62 | **0** | `cu <= ru+1`, 2 wrong |

**Decline floor F-1.** If this lane's frozen holdout shows **one** wrong emit
from the rule under test, the answer is the refusal and this lane writes the
**ninth** graveyard entry in that module doc. Not "a narrowed version of the
rule"; the refusal. A rule that is right 97 % of the time loses to a refusal that
is right 100 % of the time on what it refuses — that is the port's correctness
rule, not a preference.

**Decline floor F-2.** No successor may be **narrowed around the failing cells**.
`w-mixed` forbade this in advance for `SELF-2B` and board **#1221** repeats it:
`cu <= ru+2` fits all 22 `SELF-2B` cells on record and is 17/31 and 17/29 on the
other two classes, so it is a clause, not a rule. This lane scores it and never
proposes it. §2.3 states, in advance and in the affirmative, **which cells of
this lane's grid would make its own hypothesis a narrowing**, and registers that
outcome as a REFUTATION of the hypothesis rather than as a domain to retreat
into.

**Decline floor F-3.** A rule whose gate cannot be shown to fire on a compiled
cell does not ship (#1175). `w-mixed` §5 already establishes that this gate
**cannot** fire on today's emitter: `codegen/leaf/store.rs` builds
`alloc::Producer`s only from `s.lit` and hard-codes `ProducerKind::Constant`, so
no register-derived producer reaches `alloc::allocate` at all, and `xboxheap.cpp`
— the only workload TU in the area — sits behind **three** reader keys and an
emitter refusal (#1218). **This lane therefore registers, before probing, that
the most likely honest outcome is tier 2/3 of its own brief: a measurement, an
IL fact and a named representation change, with `crates/` unchanged except for
the graveyard entry.** Shipping is not the goal; a wrong emit is the thing to
avoid.

---

## 1. The eight dead keys, and why H-2X is not a ninth restatement

| # | key | shape | died |
|---|---|---|---|
| 1 | `w-next`'s key | `uses + (register-derived ? 1 : 0)` desc | 7 wrong / 56 |
| 2 | `H-self` | a ~1.5-use bonus for "stored into what it points at" | 11 / 72 |
| 3 | `clause-1-strict` | clause 1 with **no tie** (the narrow lift) | 12 / 36 |
| 4 | `RULE W` | a use-count/first-use lexicographic key | 7 / 388 |
| 5 | `RULE W2` | `RULE W` + a magnitude clause | 14 / 106 |
| 6 | `KEY ILX` | a two-clause key over the `.ex` (`LOAD` iff `cu<=1`) | 14 / 45 |
| 7 | `RULE BIND` | a **field edit** (`BIND` + `TEMP`), not a key | 5 / 38 |
| 8 | `H-MIX` | `cu <= ru + 1 + b`, `b` = w-spell's `2base` | 12 / 62 |

**Keys 1, 2, 3, 4, 5 are functions of `Producer`'s three fields** (`uses`,
`kind`, `first`). Key 6 reads the `.ex` but as a **per-producer** property (does
the value's offset-add chain exist). Key 7 is a field edit on the emitted
instruction. Key 8 is a threshold in the two use counts plus a term read off the
**store** spelling.

**H-2X is a RELATION BETWEEN TWO EXPRESSIONS, and no dead key is one.** It is
stated over the pair (value expression, store designator) and its whole content
is whether they are rooted at the **same IL symbol token**. w-ilx §6.1 says in
so many words that such a fact "is a relation between **two** expressions, so the
carrier is not a per-producer enum at all" — and then declines to state a rule
over it. This lane states one.

The one that comes closest is **key 8**, and the difference is exactly the thing
that killed it. `H-MIX`'s `b` asks *"do the address stores go through a bind that
is not the literal stores' base?"* — a question about **where the stores are
rooted**. It answers **yes** for `w-mixed`'s `B-2base` cells (`q.b0 = (int)&q`),
which is why it is wrong on 12 of them. H-2X asks a **different** question of the
**same** cells and answers **no**, because there the value and the store are
rooted at the *same* symbol `q`. §2.2 is the arithmetic.

---

## 2. The hypothesis

### 2.1 H-2X, stated so it can be refuted

```text
  H-2X    the address-valued producer takes POOL_TOP (r11)   iff   cu <= ru + 1 + d

            ru = the number of stores that consume the address
            cu = the number of stores that consume the literal
            d  = 1 when the ROOT SYMBOL of the value expression is a DIFFERENT
                 IL symbol token from the ROOT SYMBOL of the designator its own
                 stores are written through; 0 when they are the same token.

          DOMAIN: exactly two producers, one an address that is a PREFIX of (or
          equal to) every address it is stored into, the other one `li`.
```

`d` is a **syntactic** fact about the source spelling and it is **readable in the
`.ex`**: every designator base in the IL is `B9 <tok> <TYPE>` (w-ilx §6 / board
#909), and `d` is `tok(value) != tok(store)`. It is not a fact about the address,
which is **equal** in every cell where `d = 1` in this lane's grid.

### 2.2 It fits every in-domain cell on record — and the fit is worth NOTHING

Scored by hand over five lanes' committed tables (`work/w-mixed/grade.out`,
`work/w-ilx/holdout_grade.out`, `work/w-ilx/fit.out`,
`work/w-spell/holdout_grade.out`, `work/w-spell/fit.out`), before this lane
compiled anything:

| family | value spelled | stores through | `d` | H-2X | obj |
|---|---|---|---:|---|---|
| GRID M `1base` | `&t->mid.lo` (root `t`) | `t->mid.lo.b*` (root `t`) | 0 | `cu<=ru+1` | 31/31 |
| GRID M `2base self` | `&q` (root `q`) | `q.b*` (root `q`) | 0 | `cu<=ru+1` | 29/29 |
| GRID M `2base selfup` | `&t->mid` (root `t`) | `q.b*` (root `q`) | **1** | `cu<=ru+2` | 2/2 |
| GRID V `V1/V2` | `&s->mid[.in1]` (root `s`) | `s->mid.in1.a*` (root `s`) | 0 | `cu<=ru+1` | 10/10 |
| GRID V `V6` | `&q` (root `q`) | `q.a*` (root `q`) | 0 | `cu<=ru+1` | 5/5 |
| GRID V `V7` | `&s->mid.in1` (root `s`) | `q.a*` (root `q`) | **1** | `cu<=ru+2` | 5/5 |
| w-spell `S/H2-self-1base` | root `s` | root `s` | 0 | `cu<=ru+1` | — |
| w-spell `S/H2-self-2base` | `&s->inner` (root `s`) | `q.a*` (root `q`) | **1** | `cu<=ru+2` | 12/12 |
| w-ilx `X-A` | root `s` | root `q` | **1** | `cu<=ru+2` | 3/3 |

**That is 62 of 62 on GRID M in domain, 20 of 20 on GRID V in domain, and 22 of
22 on the whole `SELF-2B` world — and NOT ONE of them is evidence.** The
*magnitude* of `d`'s bonus (+1) is read straight off those `SELF-2B` cells; it is
exactly what makes `cu <= ru+2` fit them, and `cu <= ru+2` is board #1221's
**clause, not a rule**. `RULE W2` was 388 of 388 and `RULE BIND` 33 of 33 and
both died; #912's standing lesson is that a rule which fits its own cells has no
standing at all. **The lane fits nothing and grades once.**

### 2.3 What is NEW, and the cell that decides it

The *magnitude* is fitted. The **predicate** is not, and it makes a prediction
nobody has ever compiled. Lay the record out as a 2×2 in (value root, store
root):

| value spelled as | stores written through | class | cells on record |
|---|---|---|---:|
| a path from the formal | the same path | `SELF-1B` | many |
| the bind's own name | the bind | `LOAD` | many |
| a path from the formal | the bind | `SELF-2B` | 22 |
| **the bind's own name** | **the path from the formal** | **— no name —** | **0** |
| **one bind's name** | **a second bind to the same object** | **— no name —** | **0** |

The last two rows **do not exist anywhere**. They are the only cells that
separate H-2X from the two rules that fit the record equally well:

```text
  H-VADD  d = 1 iff the VALUE is path-spelled AND the stores go through a bind.
          (= "SELF-2B", i.e. exactly board #1221's clause with a name on it.
           This is what a lane narrowing around the failing cells would ship,
           and it fits GRID M 62/62 and GRID V 20/20 just as H-2X does.)

  H-MIX   d = 1 iff the address stores go through a bind distinct from the
          literal stores' base.   (key 8 — already dead, re-tested here.)
```

| GRID Z family | value root | store root | H-2X | H-VADD | H-MIX | `cu<=ru+1` |
|---|---|---|---|---|---|---|
| `Z1` path→path | `d` | `d` | `ru+1` | `ru+1` | `ru+1` | `ru+1` |
| `Z2` bind→bind | `k` | `k` | `ru+1` | `ru+1` | **`ru+2`** | `ru+1` |
| `Z3` path→bind | `d` | `k` | **`ru+2`** | **`ru+2`** | **`ru+2`** | `ru+1` |
| `Z4` shallow-path→bind | `d` | `k` | **`ru+2`** | **`ru+2`** | **`ru+2`** | `ru+1` |
| **`Z5` bind→path (MIRROR)** | `k` | `d` | **`ru+2`** | `ru+1` | `ru+1` | `ru+1` |
| **`Z6` bind→other bind (TWOBIND)** | `k` | `j` | **`ru+2`** | `ru+1` | **`ru+2`** | `ru+1` |

**`Z5` and `Z6` are the whole experiment.** `Z1`–`Z4` re-measure the record on
fresh names, offsets and formals and are worth nothing on their own.

**And this is where F-2 is discharged in advance.** If `Z5` comes back `ru+1`,
H-2X is **REFUTED**, and the surviving reading is H-VADD — which *is* the
narrowing around `SELF-2B`, and this lane will say so and ship nothing, rather
than adopt it. Registering that now is the point: the lane cannot retreat into
the narrowing after the fact, because it has already declared it unshippable.

### 2.4 Why H-2X rather than H-VADD — the mechanism, stated so it can be wrong

`work/w-mixed/ildiff.out` shows the killing pair's `.ex` differing at exactly the
value expression, and its first differing bytes are the **token and the type**:

```text
  (int)&q         b9 11 0a 86 43 8a 20 2c …        token 0x11, then END
  (int)&t->mid    b9 0c 0a 86 43 81 20 33 …        token 0x0c, then an offset-add
```

`0x11` is the bind `q`'s token; `0x0c` is the formal `t`'s. The store designators
in both cells are written through `q` and the `.ex` is byte-identical there. So
the fact c2 has in front of it is **two root tokens, equal or not** — not "does
the value carry an offset-add", which is a property of one expression alone and
is what `KEY ILX` (key 6) already died on. A rule that reads only the value's own
shape has been tried and is dead; a relation has not.

**The registered direction of loss (P3).** If this is wrong, the likeliest shape
is that the effect is **asymmetric** — it fires only when the *stores* are rooted
at a bind, because #1128 says a bind IS a second base symbol and #1222 measured
that the `2base` spelling is a **schedule** fact. Under that reading `Z5` is
`ru+1` and `Z6` is `ru+2`, and H-2X dies on `Z5` alone.

---

## 3. GRID Z — the classes it contains, ENUMERATED

**This section exists because of the defect `w-mixed` self-reported** (rung §8.1,
board #1223 in spirit): its own generator confounded `LOAD` with `SELF-2B` in the
frozen column, and *"had the structural variants not been there, H-MIX would have
looked clean on a grid containing no cell of the class it is wrong on."* A frozen
holdout is only as good as the classes it contains, so this lane enumerates them
**before** the freeze and the generator **asserts** the count per class.

| class | GRID Z family | on record before this lane |
|---|---|---|
| `SELF-1B` (path→path) | `Z1` | yes |
| `LOAD` (bind→bind) | `Z2` | yes |
| `SELF-2B`, path tail AGREES with the store's | `Z3` | yes (w-spell, w-ilx) |
| `SELF-2B`, path tail DIFFERS (depth 1) | `Z4` | yes (w-mixed `-selfup`, 2 cells) |
| **`MIRROR` (bind→path)** | `Z5` | **NO — 0 cells anywhere** |
| **`TWOBIND` (bind→second bind)** | `Z6` | **NO — 0 cells anywhere** |
| `CROSS` / `OTHEROBJ` — declared OUT OF DOMAIN | `X1`,`X2`,`X3` | yes |

**`SELF-2B` cells are present in the frozen column, in two spellings, at 9 points
each.** That is the confound check stated positively: the class the incumbent
hypothesis is at risk on is in the grid, twice over, and so are two classes
nobody has.

### 3.1 The axes, and the four the residual was owed

`w-mixed` §6 named four axes no lane had varied. All four are here:

| axis owed | how GRID Z varies it |
|---|---|
| the bind's own **displacement** (all five families on record bind at exactly one) | `-far`: the whole target object moves from offset **48** to offset **256** |
| the **depth** of the value's path | `Z3` binds/values at depth 2 (`d->core.u0`), `Z4` values at depth 1 (`d->core`) — the same address, `u0` being `V`'s first member |
| whether the path's **tail agrees** with the store's | `Z3` agrees, `Z4` does not |
| `cu = ru+2` **and** `cu = ru+3` in one family | every family carries both, at two `ru` each |

### 3.2 The points

```text
  P-LOW   (1,1)                 the DOMAIN control point.  This is where the
                                record's six `cross` refutations live and where
                                `cross` and `self` actually differ — board #1223
                                is w-mixed reporting that it put its controls
                                where the main axis needed them instead.
  P-IN    (2,3) (3,4)           cu = ru+1.  Every rule says `prod`; a positive
                                control — a family that cannot say `prod` here
                                is broken, not informative.
  P-DEC   (1,3) (2,4) (3,5)     cu = ru+2.  THE deciding band: d=0 says `const`,
                                d=1 says `prod`.
  P-HI    (1,4) (2,5) (3,6)     cu = ru+3.  Every rule says `const`; the check
                                that the bonus is +1 and not +2 or unbounded.
```

9 points × 6 families = **54 in-domain cells**, plus 12 `-far` cells at P-DEC and
9 out-of-domain controls at `(1,1)`, `(2,4)`, `(3,5)`. **75 total.**

---

## 4. Protocol

1. `gridz.py --freeze` writes every source, computes **every rule's prediction
   from the CELL SPEC ALONE**, and writes `pred.tsv` + `GRIDZ.sha256`. It
   compiles no obj, captures no IL and takes no disassembly. **Committed before
   `--grade` is run** — `w-mixed`'s protocol, which committed its predictions at
   `efdcf6e6` before one cell existed.
2. A moved `sha256` at `--grade` is a **hard error**, never a re-freeze.
3. One directory per cell with its own `.cpp`, `ref.obj` and `dis.txt` (#1045).
4. Every cell at the workload's own `/GR /O1 /Oi /EHsc`, read from
   `work/dc3-workload/flags.txt` by `work/w-frame/refobj.sh` rather than
   transcribed (#1112).
5. Every cell compiled at **one shared path**, so no directory or file name can
   land in the obj or the IL (w-ilx PREREG §1.1).
6. The producer's register is read off **its own store's displacement**; no regex
   ever names a source register (w-refbind's OOR bug), and the observer returns
   an **OOR counter** rather than a verdict when it matches nothing (w-ilx's
   first grade run came back `OOR` on all 45 and was not published as a result).
7. Real `c2.dll` under wibo + the project's byte compare is the sole judge.
8. Both sweep fragments (`88-store-run-call`, `89-store-run-live-arg`) and the
   full gate at **both ends** (#1205).

---

## 5. Registered predictions

| # | prediction |
|---|---|
| **P0** | **`Z5` (MIRROR) and `Z6` (TWOBIND) compile, are in regime, and are the first cells of their classes anywhere.** If either comes back OOR or compile-failed at every point, this lane has no experiment and says so. |
| **P1** | **H-2X misses at least one cell of GRID Z.** Registered as the expected outcome, on the base rate: **eight for eight**. If it survives at 0 wrong, that is the surprise, and §7 says what happens then. |
| **P2** | **`Z2` (bind→bind) follows `cu <= ru+1`**, i.e. `const` at `(1,3) (2,4) (3,5)`. This re-kills `H-MIX` on fresh names and is the cell class `w-mixed`'s own generator confounded. |
| **P3** | **The registered direction of loss: `Z5` comes back `const` at P-DEC** (`cu = ru+1` behaviour), i.e. the bonus is asymmetric and attaches only when the **stores** are rooted at a bind. That refutes H-2X and leaves H-VADD, which F-2 forbids shipping. |
| **P4** | **`Z3` and `Z4` agree at every one of the 9 points.** The value's path depth and whether its tail agrees with the store's are FREE. If they disagree, the `SELF-2B` class is itself two classes and #1221's count of 22 is wrong. |
| **P5** | **`-far` is free**: moving the bound object from displacement 48 to 256 changes no answer, 12 of 12. `w-mixed` measured four structural axes free at 16/16 and the bind's displacement is the one it could not vary. |
| **P6** | **The `(1,1)` controls DISCRIMINATE**: `X1`/`X2`/`X3` come back `const` at `(1,1)` where every in-domain family comes back `prod`. This is the measurement board **#1223** says w-mixed owed and did not take — its own controls sat where `cross` and `self` agree. If they agree here too, the PREFIX domain is unsupported by this grid as well, and the row is repeated rather than closed. |
| **P7** | **`P-HI` (`cu = ru+3`) is `const` in all six families**, 18 of 18. The bonus is bounded at +1. If any family says `prod` there, `d`'s magnitude is not 1 and every fit in §2.2 is a coincidence. |
| **P8** | **TU match stays 10, `mismatch` stays 0, both sweep fragments stay `97/1,479` and `302/968`, and every `gap-metric` line is byte-identical at both ends.** This lane changes no reader and no emitter. |
| **P9** | **Nothing in `crates/` gains an allocation statement.** Whatever GRID Z says, F-3 stands: the gate cannot fire on today's emitter, so the deliverable is the measurement. |

---

## 6. What would make this lane WRONG rather than merely negative

* Publishing `cu <= ru+2` under a `SELF-2B` predicate. F-2.
* Adding an axis to the grid after seeing an answer. The `sha256` manifest is
  the mechanical guard; a moved hash is a hard error.
* Reading the grade table and then editing §2.1. The rule is frozen as written
  at this commit.
* Quoting a gate exit code instead of graded counts.
* Claiming the `.ex` finding is new. It is **not** — `w-mixed` §4.4 and w-ilx §6
  both record that the difference is in the `.ex`. What this lane claims is
  narrower and checkable: that the fact is a **relation between two root
  tokens**, and that it makes a prediction on a class nobody has compiled.

---

## 7. The decision rule, written before the answer

```text
  Z5 = ru+2  and  Z6 = ru+2  and  0 wrong overall
      -> H-2X SURVIVES a frozen holdout.  It still does not ship (F-3): the
         gate cannot fire.  Publish the rule, the grid, the carrier spec
         (`c2-il` must record the root token of BOTH the value and the store
         designator per producer), and the larger grid the next lane owes.
         This is tier 2 of the brief: "shipped as nothing, with the larger
         grid named".

  Z5 = ru+1                      -> H-2X REFUTED.  Ninth graveyard entry.  The
                                    surviving reading is H-VADD and F-2 forbids
                                    it.  Say which cell killed it.

  Z6 = ru+1  and  Z5 = ru+2      -> H-2X REFUTED on Z6.  Ninth graveyard entry.

  any OOR / compile-failed cell  -> reported as a counter, never as a verdict.
```

In every branch the lane's deliverable is the same: **the IL fact that
distinguishes the killing pair, stated as a relation between two root tokens,
and the field `alloc::Producer` needs to carry it.** That is tier 3 of the
brief and it does not depend on the grade.
