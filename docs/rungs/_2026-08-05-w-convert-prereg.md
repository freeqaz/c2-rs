# w-convert — PREREG: the `2C` cross-class reinterpret, the one key the one-away screen ranks first

    Tag:       w-convert
    Slug:      w-convert-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg. It admits no shape and ships no
               lowering, so there is nothing an obj-graded fixture could grade.
               The findings record names whatever fixtures the build earns.
    Census:    not measured yet by design. The baseline this lane registers is
               in §1 and is the brief's, to be reproduced before any change.
    Lane:      w-convert, worktree `wt-w-convert` off master **`c5dd2f6`**.
    Record:    this file. The findings record is
               `docs/rungs/_2026-08-05-w-convert.md`.

---

## 0. What this lane is for

Nine successive rankings over blocker keys have been refuted, the ninth
(`w-build`, #683) from the *definition* of the ranked object. What replaced it is
the **one-away screen** (#684): put one opcode in `C2RS_SINK_CHAIN`, count
`expr-chain-sink-poison`, and the count is the number of functions that operator
is the **only** expression-layer blocker for.

`convert` is the screen's highest figure at near-perfect purity — **8,181
one-away of 8,222 total mass, "nothing above +19" as a substitution**. It is
the only key on that board where closing it is measured to be close to
*terminal* rather than a fall-through. This lane builds it.

The construct is `2C <TYPE target> 00` in `parse_expr_classed`. The arm admits a
conversion **only when the target is the value's own `ValueClass`**; anything
else refuses under `expr-convert-target-<tag><kind>`. The arm's own doc comment
names the open rung in so many words:

> **Cross-class refuses**, and that is a conservatism with a measured price
> rather than an oversight: `int f(S* p){ return (int)p; }` and
> `S* f(int a){ return (S*)a; }` are both a bare `blr` on this target (probed),
> so admitting them is a rung — but a reinterpret between the two classes has
> never been graded across the widths, cv-spellings and argument positions this
> parser reaches, and `expr-convert-target` makes what it costs a number instead
> of an argument.

**This lane grades it.** The hypothesis under test is narrow and stated as a
falsifiable claim, not as a direction:

> **H1.** A `2C` whose target is a width-4 value class (`Int4` or `Ptr4`) applied
> to a value that is already a width-4 value class emits **no instruction**, in
> every signedness, cv-spelling, pointee type and operand position this parser
> reaches — so the *cross-class* pair `Int4→Ptr4` / `Ptr4→Int4` is free exactly
> as the two same-class pairs already shipped are.

**H1 is falsifiable per cell.** The complete matrix over the reachable target
types is affordable (§3), and every cell is graded by the sole judge — real
`c2.dll` under wibo, byte-exact obj compare — not by an argument about what a
reinterpret "should" cost.

---

## 1. Baseline, reproduced before any change

Reproduced on `wt-w-convert` at `c5dd2f6` with a linked `compilers/` and the
shared capture cache, **before** this file was written:

```
match 9 · mismatch 0 · codegen-gap 0 · vocab-gap 862 · capture-fail 7
A/B/C/D/E = 28 (LO 27)/338/169/9/2 · A∧B∧C 27 · FRONTIER 18
FBM 0.16654 · fnbyte-exact 29,801 · fnbyte-partial 9,375 · fnbyte-differs 0
emitted census 39,176/178,975 in class (21.89%)
```

Every figure matches the brief's exactly. `work/w-convert/gap-baseline.txt`.

**Registered before the run**, per the bar: `scripts/gate.sh --jobs 6` PASS
**18/18, 4,608 verdicts**, and it grows by exactly 18 per fixture — so if this
lane lands *N* fixtures the number must be `4,608 + 18N` and nothing else.
Sweep **16,466** / 16,370 graded / **96 ungraded**; cross 77,125 of 77,513 /
**388 ungraded**; 0 mismatch. `cargo test --workspace --release` **855 passed /
0 FAILED / 27 targets** — and the *target count* is registered too, because a
run that stops at the first failing target reports fewer passes **and** fewer
targets.

---

## 2. Registered predictions

Written down before the corresponding measurement. **Misses stay on the page.**

| # | quantity | registered | basis |
|---|---|---|---|
| P1 | `convert` one-away, re-derived by me | **8,181 exactly** | same binary logic, unmodified master, `C2RS_SINK_CHAIN=convert`. A different number means the screen is not reproducible and that is the finding |
| P2 | share of the 8,222 whose target is a **pointer** type (`x6 43`) | **40–80 %** | the workload's convert population is dominated by `this`/getter plumbing; the doc's `T*`→`void*` row is the shape that recurs |
| P3 | functions actually converted by the build | **+300 to +3,000** | P1 is a *ceiling with a named leak* (#687): the poison precedes the pointer-arithmetic guard, and my rule is the one that **sets** `saw_ptr`, so the leak bites this lane harder than any prior one. I register a realized fraction of 4–37 % of the ceiling |
| P4 | `fnbyte-exact` | **29,801 → 30,101…32,801** | P3, one-to-one: an accepted body that the emitter completes is exactly one exact or one partial |
| P5 | FBM | **0.16654 → 0.16822…0.18215** | P4 over the fixed denominator 178,975 |
| P6 | **`fnbyte-differs`** | **0, unchanged** | the alarm. Any movement is a wrong emit and the build reverts |
| P7 | emitted census in class | **39,176 → 39,476…42,176** | P3 |
| P8 | **TU match** | **9 — no movement** | seven of the last nine lanes correctly registered no TU movement. Being one-away is per-**function**; a TU converts only when *every* blocked function in it converts, and the 18 frontier TUs' blockers are the call/branch/off-add families, not `2C`. If this moves it is the first codegen conversion since `xboxmem` |
| P9 | FRONTIER | **18, unchanged** | P8 |
| P10 | cells of the full matrix (§3) that turn out **not** free | **1 to 4** | conversions are where sign/width interactions hide, and the single-cell trap has fired five times. Registering *zero* exceptions would be registering that the matrix was not worth running |

**P8 and P9 are the ones the models are trusted for.** A lane that registers
movement and gets none has learned nothing; a lane that registers none and gets
none has confirmed the model that says the frontier is not gated here.

---

## 3. The matrix, registered before it is run

Not the workload's cell — the **full cross product**. The single-cell trap has
fired five times (`!=`→`>` at exactly 63 burners; a 32768 bound; unsigned `k=0`
emitting a bare `blr` with no relocation; a mask collapse that changed **block
layout**; `C = 0x249b0000` wrong on 29 of 32 columns of its own row), so the
grid is declared here and run whole.

**Axes**, and what each is for:

1. **direction** — `int→ptr`, `ptr→int`, `ptr→ptr`, `int→int` (the last two are
   the already-shipped controls and must stay free);
2. **integer signedness / spelling** — `int`, `unsigned`, `long`,
   `unsigned long`, `const int`, an **enum**, a **typedef**. Signedness is the
   axis this target could plausibly break on: the registers are 64-bit and the
   pointers are 32-bit, so a *signed* `int`→pointer is where an `extsw`/`rldicl`
   would appear if one appears anywhere;
3. **pointee** — `void*`, `int*`, `char*`, `S*`, `const char*`, `S**`, a
   **function pointer**;
4. **position** — leaf return value; **call argument**; interior of an add
   chain; trailing after the operator. The last two exist because
   `CONV_INTERIOR`/`CONV_TRAILING` are already the shipped shapes and a new
   class must be free in the same three places;
5. **width and direction of change** — the **boundary values** of the axis the
   rule must *refuse*: `char`, `short`, `unsigned char`, `long long`, `float`,
   `double`, `bool`. These are the negative controls and they must keep
   refusing.

**What the probes silently hold fixed, stated (#644):** every cell is a *formal
parameter* source (nothing folds — §2.3 of `docs/IL_CAST_CONVERT.md` says c1xx
folds a cast of a literal, so a literal source cannot reach the arm at all), one
conversion per function, and one optimization profile. The profile is the axis I
am explicitly *not* holding fixed: the grid runs at the workload's `/O1 /Oi
/EHsc` **and** at `/Ox`, because `/Ox` listings are not COMDAT (#643) and a
grid that only ever saw one profile is a grid that cannot contain its own
counterexample.

---

## 4. What ships, and the shape of the guard

**Positive guards, additive-refusal by construction where possible.** The `2C`
arm is a positive gate — it accepts only what a predicate says `true` for — so a
widening here is additive-**accept** by construction and cannot claim the
additive-refusal property. **That is said here rather than claimed away**, and
it is why the negative controls in §3.5 are part of the ship and not a nicety.

Two guards are load-bearing and are registered now so that forgetting one is a
visible miss rather than a silent wrong emit:

* **A convert that produces `Ptr4` must set `saw_ptr`.** `(S*)a + 1` is
  `addi r3,r3,sizeof(S)` — c2 **scales** pointer arithmetic — and a chain that
  added 1 unscaled is a wrong emit, not a gap. The existing pointer-arithmetic
  guard already refuses `saw_ptr && arithmetic`; it only works if the *converted*
  pointer indicts the value the same way a loaded one does.
* **`Int1u` stays out.** `bool`→`int` is `rlwinm r3,r3,0,24,31`, a real
  instruction. The one free cell the doc records in that class (`bool`→`char`)
  produces a value of a class this parser does not model, so there is nothing to
  track it as; it refuses.

**When a rule fits some cells but not enough, this lane ships a REFUSAL** with
the count, exactly as #688 shipped 23 witnesses as a refusal rather than a rule.

---

## 5. What would break it, and whether the corpus can express that

Asked before the build, per the standing instruction, because #688 is a live
wrong emit that survived because *the corpus could not express the failure* —
`sweep.d/10-int-chains.py` enumerated three leaves and one intermediate, and at
one intermediate every candidate rule gives the same register.

The shapes that would break H1 and the question of whether anything in the tree
today reaches them:

| shape | breaks H1 by | can the corpus express it today? |
|---|---|---|
| `(void*)a` where `a` is **signed** `int` | an `extsw`/`rldicl` appearing at the 32→64 boundary | **no** — every tracked convert fixture is same-class |
| `(S*)a + 1` | unscaled add against c2's scaled `addi` | **no** — `saw_ptr` is only ever set by a LOAD today, so no fixture can even build the shape |
| `(int)p` in a **call-argument** position with a `55` type that restates the *pointer* class | class desync between the `2C` and the arg type | partly — `CONV_PTR_ARG` is same-class |
| a **function** pointer source | a different `kind` nibble than the data-pointer one | **no** |
| a convert at `/Ox` where allocation differs | #688's neighbourhood | **no** — no convert fixture is graded at `/Ox` |

**Four of five are `no`.** So this lane owes the tree a fragment and a fixture
that can express them, and the fragment's own negative control must be checked
to actually reach the shape it names — `w-build`'s fragment first reported 0
mismatches against the very binary it was written to catch, because unbracketed
C++ precedence reassociated every case away from the shape.

---

## 6. Board items taken

Starting at **#700** per the brief. Reported in the findings record.

---

## 7. Retirement clause

If the matrix refutes H1 — if the cross-class reinterpret costs an instruction in
any reachable cell — this lane ships the **refusal** with the cell that killed
it, corrects the arm's doc comment (which currently asserts both directions are
"a bare `blr` on this target (probed)"), and registers the corrected claim beside
the wrong one. **Retire a prediction when its framing changes; correct your own
claims on the page.**
