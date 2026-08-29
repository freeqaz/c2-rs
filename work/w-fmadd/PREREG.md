# PREREG — lane `w-fmadd`, wave 19 (2026-08-29)

**Committed before `c2.dll` was disassembled and before any FP probe was
compiled.** Board `#3790`–`#3795`. Charter:
`docs/ADOPTION_BRIEF_2026-08-29.md` §L1; `docs/rungs/2026-08-28-w-encarms.md`
§10 item 3.

Base: master `12d3c0558`, branch `wt-w-fmadd`.

---

## 0. What is ALREADY KNOWN before this lane starts

Recorded first so no re-read can later be sold as a read (`w-encarms`'s §0
pattern). Every line below was established by reading files already in the
tree, **not** by opening the image.

1. **`10bfa49a` serves form 24, 18 opcodes** — `docs/whitebox/ref/ENCODE_ARMS.txt:12`
   and `P_ENCODE.md:772`. The 18 opcodes are `fmadd(.)`, `fmadds(.)`,
   `fmsub(.)`, `fmsubs(.)`, `fnmadd(.)`, `fnmadds(.)`, `fnmsub(.)`,
   `fnmsubs(.)` and two more — `ENCODE_OPCODES.txt` rows `0x0077`–`0x008e`.
2. **`P_ENCODE.md` §5 does NOT carry form 24's placement rule.** §5.1's table
   runs forms 49/22/39/47/38/36/25/51/43/33/41/42/56/40/31; form 24 appears
   nowhere in §5. So the arm read this lane owes **is a new read**, unlike
   `w-encarms`'s forms 7 and 54 which §5.7/§5.3 had already bounded.
3. **Neither half of the encoder is in the port.** `mop::op` has no
   `fmadd`-family constant (the FP constants stop at `FMUL`/`FMULS`/`FMR`/
   `FRSP`, `mop.rs:229–240`) and `mop::plan`'s match has **no arm for form
   24** (`mop.rs:866–1005`; form 23 is at `mop.rs:888`, form 25 at 890).
4. **The refusal that blocks the emit is explicit and is in the LOWERING**,
   `crates/c2-core/src/codegen/leaf/float.rs:109–113`:
   > `"FP expression mixes `*` with `+`/`-`: c2 contracts these to
   > fmadds/fmsubs/fnmsubs, which is not modeled; out of class"`
5. **The observed words are already published** —
   `docs/CODEGEN_W13_FLOAT.md` §3.3, an `[O]` reading off real objs:
   `a*b+c` → `ec2118ba` (`fmadds f1,f1,f2,f3`), `c+a*b` → the *same* word,
   `a*b-c` → `ec2118b8` (`fmsubs`), `c-a*b` → `ec2118bc` (`fnmsubs`),
   `double a*b+c*d` → `fc2100ba`. §3.3 is a **measurement of the output**;
   it says nothing about which c2 slot feeds which field, which is what the
   arm read is for.
6. **The negatives already exist as source**: `fixtures/cpp/w13_fneg.cpp`
   §N1 is seven functions and §N2 one more, written expressly as the
   contraction class.

## 1. What this lane claims is genuinely open

* **R1 — form 24's field plan, read at `0x10bfa49a`.** Which c2 operand slot
  lands in the PPC **B** field (shift 11) and which in the **C** field
  (shift 6). §3.3's words cannot answer this: in every published site the
  order is unambiguous only if you already assume the mnemonic order.
* **R2 — the contraction rule** the lowering would need: which multiply
  becomes the `A×C` pair, which operand becomes `B`, and what happens when
  both sides of the `+` are products.
* **R3 — whether the byte judge can grade any of it**, i.e. whether a
  fixture exists (or can be added) whose obj a `c2.dll` run will compare
  byte-exact against a port emit.

## 2. Registered predictions

Scored in the rung, hit or miss.

| # | prediction |
|---|---|
| **P1** | **The brief's own sentence is FALSE as written.** §L1 says *"a lane that treats this as an encoder row will find the arm already reachable and nothing to do."* Form 24 has **no field plan and no opcode row** (§0.3), so an encoder-only lane finds real table work. The brief's *conclusion* (codegen lane, not encoder lane) is right; its stated reason is not. |
| **P2** | Arm `0x10bfa49a` places four register fields, `RT<<21 \| RA<<16 \| RB<<11 \| RC<<6`, all 5-bit, all through `reg()`. |
| **P3** | The slot order follows the **mnemonic** (`fmadd fD,fA,fC,fB`), i.e. c2's `D0`→A(16), `D1`→**C(6)**, `D2`→**B(11)** — the non-monotone one. Registered because form 23 (`fmul`) already puts `D1` at shift 6 (`mop.rs:888`), so this is the continuation of that pattern rather than of the bit layout. **Confidence ~65 %; the whole point of the read.** |
| **P4** | The contraction is **c2's, not c1xx's**: the captured IL for `a*b+c` carries a separate multiply node and add node, and no fused node. |
| **P5** | `a*b+c` and `c+a*b` emit **identical** bytes and the port can reproduce both from one rule; the addend is the `B` field regardless of which source side it was written on. |
| **P6** | At least **4 of the 8** contraction functions in `w13_fneg.cpp` (§N1's seven plus §N2's `n_rank`) can be emitted byte-exact by a rule that adds no register allocator. `n_rank` (§N2) and `n_fma_tree` are the two predicted to stay refused. |
| **P7** | Adoption moves `[encode] ported` **29/79 → 30/79**, and nothing else on the subsys board. |
| **P8** | **A mutation of form 24's C-field shift (6 → 7) WILL move at least one byte test**, unlike `w-encarms`'s C-C2. Registered as a falsifiable claim about `#3723`: if it moves **zero**, the surface row is again the only grader and that is the lane's finding, not a footnote. |
| **P9** | The **required-zero byte delta is not available to this lane** as its grading instrument, because the adoption converts refusals into emits. The grade is byte tests against real `c2.dll`, plus a `c2_core::surface` row whose domain runs past the corpus. |

## 3. The fail axis, registered before the first edit to `crates/`

**The operand-to-field assignment of form 24, and the contraction rule's
choice of which operand is the addend.** FP multiply-add is fused: `fA*fC+fB`
with `A`/`C` swapped is still a legal, disassemblable, *numerically identical*
word (multiplication commutes), so **a B↔C swap is the defect the byte judge
must catch and a fuzz-matcher cannot** — the same shape as form 39's
destination field, which `P_ENCODE.md` §5.1 calls the most safety-critical
fact on the page. Concretely, every one of these must be watched RED before
any verdict is quoted:

* **C-1** — swap form 24's B and C shifts (11 ↔ 6). Predicted RED on byte
  tests **and** on the surface baseline.
* **C-2** — the `#3723` control: perturb only the field the corpus cannot
  reach, and require the surface row to catch it when the byte tests do not.
* **C-3** — drop the new `OPCODE_ROWS` entries; predicted RED on the surface
  domain rendering.
* **C-4** — plant a contraction that picks the wrong addend (`fB` ← the
  multiplicand rather than the summand). Predicted RED on byte tests.

Controls restore with `cp` **and then `touch`** — `w-encarms` §5.2 and the
brief §5 both record that the un-touched restore silently runs the mutated
binary.

## 4. Refusal conditions, registered so a decline is not retro-fitted

This lane says **declined** (or `FAILED`, if it produces no deliverable at
all) rather than adopting, if any of:

* the arm read is ambiguous about the B/C assignment and no obj can
  disambiguate it;
* no fixture can be constructed whose port emit is graded byte-exact by
  real `c2.dll`, so that `[encode] ported` would move by construction alone
  (`#3505`);
* the contraction rule cannot be stated without a register allocator or a
  scheduler the port does not have.

**Adding the table row and the field plan with no emit path behind them is
explicitly OUT of scope**: `subsys.rs`'s `ported` predicate would count the
arm on a plan nothing reaches, which is the fabricated-numerator shape
`#3505` is five for five on.

## 5. Out of scope

* Any register allocator or FP scheduler (`float.rs`'s two-constant gate and
  `n_spill` stay refused).
* The 14 non-`s` / non-`madd` opcodes of the arm beyond what an emit reaches.
* `docs/STATUS.md`, `docs/rungs/INDEX.md` (generated), `splice.rs`,
  `CLAUSES.tsv`, `P_INLINE.md`, `P_GLOBREGS.md`.
* A 22nd count-bearing `scripts/gate.sh` row (`#3691`).
