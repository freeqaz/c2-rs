# w-emitp — PRE-REGISTRATION

    Lane:    w-emitp, 2026-08-04, worktree `wt-w-emitp` off master `b6fa935`
    Ships:   NOTHING under `crates/`.  Analysis lane.
    Owns:    work/w-emitp/, one rung doc, docs/ prose (not PRIOR_ART.md).

**Committed BEFORE any corpus-wide measurement.**  §9 discloses the 3-TU pilot in
full, including every number I have already seen.

---

## 0. The mechanism, and why it is not one of the four already eliminated

Six lanes modelled the emit set as a closure over **`U` = the gate-clean
tag-0x0E `.gl` records**.  Four hypothesis classes are eliminated: the edge
hypothesis (w-refs), the owner skips (w-skip), the joint DATA+CODE fixpoint
(w-joint — the defined-data set is absorbing, 50 897/50 897), and the `db`
sub-stream (w-db — it is DEBUG and `/Zi`-gated, c2 never reads it here).

All four are hypotheses about **which edges to follow between nodes of `U`**.
This lane's hypothesis is about a **node class that is not in `U` at all**.

> ### **H-ALIAS.  The `.gl` stream carries tag-0x10 ALIAS records.  An alias has no `.ex` body, so no lane's `U` contains it; and in the same word a tag-0x0E record uses for its emit flags it carries a TOKEN naming another symbol.  A vftable's initializer names the ALIAS; the symbol c2 emits is the alias's TARGET.  Every model to date drops that edge on the floor, and the class it drops is `??_G`/`??_E` — 58.95 % of the ORACLE ceiling's residual (w-joint U-i).**

Read from `c2.dll` 16.00.11886.00 (image base 0x10b00000), not fitted.  The tag
dispatch at `0x10b9b91f` sends tags 0x04 / 0x0E / 0x10 to one shared KIND-4
handler at `0x10b9bdcf`, which splits on the tag only at the end:

    10b9bf46  cmp  DWORD PTR [ebp-0x78],0xe      ; tag == 0x0E ?
    10b9bf4a  jne  0x10b9c01e
    10b9bf50  or   DWORD PTR [esi+0x37],0x200000 ; "is in U"
    10b9bf57  call 0x10c1f9e9                    ; i32c -> +0x54  (the .ex offset)
    10b9bf70  call 0x10c1f91b                    ; varU -> +0x4c  (flags4c: the MARK word)
    10b9bf99  test DWORD PTR [esi+0x4c],0x1000   ; ... then w-refs' reference list
  ---------------------------------------------------------------------------
    10b9c01e  cmp  DWORD PTR [ebp-0x78],0x10     ; tag == 0x10 ?
    10b9c022  jne  0x10b9c033
    10b9c024  or   DWORD PTR [esi+0x37],0x400000 ; THE ALIAS BIT
    10b9c02b  call 0x10c1f91b                    ; varU
    10b9c030  mov  DWORD PTR [esi+0x4c],eax      ; THE ALIAS TARGET TOKEN

So on a tag-0x10 record `[sym+0x4c]` is **not** `flags4c` — it is a symbol
token, at exactly the byte offset `refs.head` already locates.  An `<imm32>`
scan of the whole `.text` finds `0x400000` against `+0x37` at **three** sites,
one of which is the write above; the two readers are

    10b8ac60  test [eax+0x37],0x400000  ->  or [eax+0x32],1
    10b99621  test [esi+0x37],0x400000  ->  ecx = [esi+0x4c] ; resolve (0x10b9860d)
              10b99635  or [eax+0x20],0x2000     ... on the TARGET's flag word

`+0x20 & 0x2000` is the same bit w-joint enumerated as the static rule
`F20_2000` and graded at precision 0.81639 — **statically**.  This lane's claim
is that c2 *sets* it, from the alias records, and that the static reading was
therefore measuring the residue of a channel rather than the channel.

**What this lane does NOT claim.**  It does not claim to have identified the
instruction that turns `+0x20 & 0x2000` into `+0x4c & 0x20` (the Mark bit).
`0x10b28ca3` — the COFF writer's Mark, gated on `[edi+0x37] & 0x200000`, reading
a token from `[esi+0x3f]` — is a candidate and is **not decoded here**.  The
claim graded below is **extensional**: an initializer node naming an alias
contributes the alias's *target*.  §X puts that to the sole judge.

---

## 1. The frozen model list.  Nothing is added after truth is read.

`resolve(t) = alias(t) if t names a bound tag-0x10 record else t`, applied at
the named edge class, then the incumbent closure unchanged.

| variant | what changes against its incumbent |
|---|---|
| `RGL` `INIT` `SKIP` `JFP` `ORACLE` | the five incumbents, recomputed in the same pass — KA-A |
| **`ALIAS_IN`** | `ORACLE` with `resolve` applied to `in` `02`-node targets |
| **`ALIAS_REF`** | `RGL` with `resolve` applied to `.gl` reference-list targets |
| **`ALIAS_BOTH`** | `ORACLE` with `resolve` applied to both |
| **`JFP_ALIAS`** | `JFP` with `resolve` applied to both |
| `RGL_ALIAS_IN` | `RGL` with `resolve` on `in` targets — isolates the alias from the ORACLE's data oracle |
| `ALIAS_SHIFT1` | `ALIAS_BOTH` computed from the **shift +1** alias table — the null |

---

## 2. Registered points and intervals — the instrument

| # | quantity | **point** | interval |
|---|---|---:|---|
| T1 | bound fraction of tag-0x10 records, shift 0 | **1.000** | [0.95, 1.00] |
| T2 | fraction of bound aliases of the shape `??_E<X>` -> `??_G<X>` | **0.99** | [0.90, 1.00] |
| T3 | **SHIFT null** — bound fraction at shift −1 / +1, mean | **0.02** | [0.00, 0.20] |
| T4 | corpus tag-0x10 record count | **300 000** | [50 000, 1 500 000] |
| T5 | fraction of alias targets that are in `U` | **0.99** | [0.85, 1.00] |
| T6 | `\|dom(alias) ∩ U\|` — an alias must not itself have a body | **0** | [0, 2 000] |
| T7 | KA-A: the five incumbents reproduce w-joint/w-db to the digit | **5/5** | 5/5 |

## 3. Registered points — the CEILING DECOMPOSITION

This is the measurement `STATUS.md` trap 8 says nobody has made: the per-TU
exact metric **decomposed by residual class**.  w-joint published micro-F1
stratified on `#152` and did **not** publish per-TU exact stratified.  Computed
on the incumbent `ORACLE`, no new model involved.

| # | quantity | **point** | interval |
|---|---|---:|---|
| C1 | `ORACLE` per-TU exact with `#152` removed from **both** `E` and `P` | **420** of 850 | [200, 700] |
| C2 | `RGL` per-TU exact, same removal | **200** of 850 | [132, 500] |
| C3 | TUs whose entire `ORACLE` residual is `#152` | **260** | [50, 550] |
| C4 | TUs whose `ORACLE` residual is non-empty and contains no `#152` | **200** | [50, 500] |

## 4. Registered points — the MODEL

Incumbents, from w-joint §2.2 and w-db §2.2, over 850 TUs and `\|E\|` 174 417:
`RGL` 1.00000 / 0.74307 / 0.85260 / **132**; `JFP` 0.99899 / 0.86391 / 0.92655 /
**132**; `ORACLE` 0.99997 / 0.95867 / 0.97888 / **151**.

| # | quantity | **point** | interval |
|---|---|---:|---|
| M1 | `ALIAS_IN` recall | **0.978** | [0.955, 0.995] |
| M2 | `ALIAS_IN` precision | **0.9998** | [0.980, 1.000] |
| M3 | `ALIAS_IN` F1 | **0.989** | [0.965, 0.9975] |
| **M4** | **`ALIAS_IN` per-TU exact** | **330** of 850 | [151, 700] |
| M5 | `ALIAS_REF` F1 − `RGL` F1 — predicted **INERT**, because `0x10b27f3c` keeps an edge only for a tag-0x0E target (w-db V-d) | **+0.000** | [−0.001, +0.020] |
| M6 | `JFP_ALIAS` per-TU exact | **230** of 850 | [132, 600] |
| M7 | `#152` share of `ALIAS_IN`'s residual (0.5895 for `ORACLE`) | **0.10** | [0.00, 0.40] |
| M8 | `ALIAS_SHIFT1` F1 − `ORACLE` F1 — the null | **0.000** | [−0.010, +0.005] |

## 5. Registered — the SOLE JUDGE (real `c2.dll` under wibo)

Byte-length-preserving `varU` retarget of one `02` node in the `in` stream,
w-joint's technique unchanged, on **three** non-quarantined TUs, 5 draws each.

| # | arm | prediction |
|---|---|---|
| X1 | **H+** — retarget to a token naming a tag-0x10 alias whose target is in `U` and is NOT in the baseline leader set | the **TARGET's** COMDAT APPEARS, ≥ 4/5 per TU |
| X2 | **H−** — retarget to a token that is in neither `U` nor `dom(alias)` | ≤ 1/5 APPEAR |
| X3 | **H0** — rewrite the same token over itself | obj byte-identical (TimeDateStamp zeroed), 3/3 |
| X4 | in the H+ arm, the **ALIAS's own name** appears as a COMDAT | ≤ 1/15 — the target is emitted, not the alias |

X2 is the arm that makes X1 mean something: it is this lane's version of w-db
§10a's FUNC-vs-DATA parity control, and it can go green-as-APPEARS, in which
case the write is perturbing the obj by something other than the alias.

## 6. Decline clauses

1. **If T3 ≥ 0.5 × T1** the field is not identified; publish the decode as
   UNPROVEN, run no model, and stop.
2. **If M4 ≤ 151** — no per-TU exact gain over the ORACLE ceiling — publish it
   plainly as a micro-F1-only result in `STATUS.md` trap 8's exact shape, say so
   in the headline, and do **not** request the one-shot gate.
3. **If X1 < 3/5 pooled** the mechanism is REFUTED; the refutation goes above
   the fold and the model claim is withdrawn.
4. **Nothing ships under `crates/`.**  No fixture, no codegen, no
   `DISCLOSURE.md` row.  If the model is implementation-ready, the deliverable
   is a spec.
5. **Do not spend the one-shot Part-1 gate.**  The 21-TU quarantine is checked
   by the mutation script by name before anything is written.  If the model
   looks finished, ASK the coordinator; do not run it.
6. **No instrument tuning after truth is read.**  `alias.py`'s field position,
   its three gates and the variant list of §1 are frozen at this commit.
7. **Report per-TU exact and micro-F1 SEPARATELY**, every table, both numbers.

## 7. What would make me decline the whole lane

If the shift null binds (clause 1), or if the corpus-wide `??_E`->`??_G` shape
fraction falls below 0.90, the decode is not a decode and everything below it is
uninterpretable.  I would publish the pilot, the disassembly and the null, and
stop.

## 8. The declared bias

**The numbers I most expect to be wrong about are C1 and M4** — the per-TU exact
figures.  Trap 8 exists because micro-F1 has moved three times without moving
per-TU exact, and I am registering a large jump in a metric that has resisted
three previous large jumps in its leading indicator.  I have deliberately
registered M4's interval **open at the bottom to exactly the incumbent (151)**,
so "no gain at all" is inside the interval and costs me nothing to report.

Second: **M5.**  I predict the reference-list channel is inert because w-db
showed `0x10b27f3c` drops non-tag-0x0E targets.  If `ALIAS_REF` moves, w-db's
V-d needs qualifying and I will say so.

## 9. DISCLOSURE — the 3-TU pilot, in full

Run before this commit, on `src/App.cpp`, `src/system/rnddx9/Movie.cpp`,
`src/system/synth/StreamNull.cpp` (all three are w-joint's KA-MUT TUs and none
is quarantined).  **Every number I have already seen:**

* `??_7FilePath@@6B@`'s `in` record names `??_R4FilePath@@6B@`,
  **`??_EFilePath@@UAAPAXI@Z`** and `?Print@String@@UAAXPBD@Z`; the obj emits
  **`??_GFilePath@@UAAPAXI@Z`**.  Same for `??_7ObjRef@@6B@` and
  `??_7Message@@6B@`.
* `??_EFilePath@@UAAPAXI@Z` is a **tag-0x10** record, `f20 = 0x404`, no `.ex`
  body, token `0xd76a`; `??_GFilePath@@UAAPAXI@Z` is **tag-0x0E**, `f20 = 0x405`,
  `.ex` at 212 350, token `0xd56a`, `flags4c = 0x18c8`.  The bytes `d5 6a` occur
  in the alias record at the offset the disassembly predicts.
* Every `??_G`/`??_E` in those three TUs' truth is **in `U`** and (except the
  `$4…` adjustor thunks) is named by **no** `02` node and, mostly, by no
  reference list.
* Gated decode, **shift 0**: bound **419/419**, **78/78**, **43/43**; shape
  `??_E<X>`->`??_G<X>` **419/419, 78/78, 43/43**.
* **shift −1**: bound 6, 0, 1.  **shift +1**: bound 8, 1, 1.  None paired.
* Alias targets present in `E`: **5** of App.cpp's 6 `#152` names, 4 of
  Movie.cpp's 7, 2 of StreamNull.cpp's 3.  The uncovered ones are
  `??_GDataArray@@AAAPAXI@Z` (already a reference-list target),
  `??_GFaderGroup@@QAAPAXI@Z` (same) and the two `??_E…$4PPPPPPPM@A@…` adjustor
  thunks, which **are** named by `02` nodes under their own name.
* An earlier version of `alias.py` used w-refs' terminus gate and passed only
  99/419 on App.cpp.  **All 320 rejects still decoded to a correct
  `??_E`->`??_G` pair**; they were rejected because the *following* record is a
  tag-0x0B undecorated-name record.  The gate was replaced with RT + BIND +
  SHIFT **before** this commit and the replacement is disclosed here rather than
  presented as the original design.

Nothing above was measured against corpus truth, and no per-TU exact or F1
number for any variant of §1 has been computed at the time of this commit.

## 10. Registered before the numbers exist

* **TU match stays 8.**  An analysis lane does not move it.
* **`census/gate disagreement` stays 0.**
* **Order is untouched.**  A right set in the wrong order is still a mismatch.
* A high F1 is not a shippable predicate, and the ORACLE variants remain
  ceilings, never models.
