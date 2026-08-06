# w-bytes — PREREGISTRATION

Lane: **w-bytes**. Unit: the **byte interior** of a differing function body.
Board rows reserved: **#976–#985**.

Written and committed **before** any signature was computed. Nothing below is a
result; everything below is a prediction or a falsifier.

## 0. What this lane is measuring

The population is the **3,195 `fnbyte-differs`** rows of the dc3 workload scan —
per-(TU, symbol) bodies where `comdat_body_from_selected` produced a complete
`/Gy` body whose bytes are not real c2's. Today the scan prints, per differ:

    fnbyte-differs-fn|<shape>|w<pw>/<rw>/eq<eq>|first@<i>:port=…,ref=…|<name>

That is a first-divergence word and two counts. It does not say whether the two
bodies are the *same instructions in a different order*, whether the divergence
is an insertion/deletion or a substitution, or **which field** of a substituted
instruction moved.

The lane adds a **diff signature** per differing symbol and clusters the 3,195
by it. Measurement and tooling only — **no emitter change ships from this lane.**

## 1. Method, fixed in advance

* PPC is fixed-width, so the unit is a **4-byte big-endian word**.
* Alignment: common prefix, common suffix, and an **LCS over the interior** at
  word granularity, yielding per-word `equal` / `substitute` / `insert` (port has
  a word c2 does not) / `delete` (c2 has a word the port does not).
* Each substituted pair is classified by **decoded field**. Discipline from
  `docs/CODEGEN_W6_COMPARE.md`: a word is decoded only if its form's field
  partition **covers all 32 bits and reassembles bit-exactly**; anything else is
  `undecoded`, never guessed.
* `same-multiset` bit: sort both word lists; equal ⇒ the port emitted exactly
  c2's instructions in a different order (a pure schedule/order defect).
* Relocation sites: the reference COMDAT's relocation `VirtualAddress`es,
  so a mismatched word can be marked as sitting under a relocation (whose
  displacement/immediate field the linker owns and c2 may leave at a different
  filler value).
* Keyed on **`FnCensus::emit_name`** (#918). Never `IlFunction::mangled_name`.

## 2. What I expect to find (falsifiable predictions)

P1. **The differs are structured, not scattered.** ≥ 60 % of the 3,195 fall into
    ≤ 10 distinct signature clusters (signature = shape + edit-op multiset +
    field-class multiset).

P2. **`tail` dominates and is length-driven.** The ~1,531 `tail` differs are
    mostly `port_words != ref_words` — the port emits a branch (or a body) where
    c2 emitted something shorter/longer — rather than same-length substitutions.
    Predicted: ≥ 70 % of `tail` differs have `pw != rw`.

P3. **Same-length differs are dominated by ONE field class.** Among differs with
    `pw == rw`, the modal per-word class is a **register field** or a **branch
    displacement**, not an opcode. Predicted: ≥ 50 % of same-length differing
    words have identical primary+extended opcode.

P4. **A first-divergence spike at word 0.** A non-trivial cluster diverges at
    the very first word (prologue/first instruction), which would be a
    lower-complexity target than an interior schedule difference.

P5. **`same-multiset` is small but nonzero** — pure scheduling permutations exist
    but are not the bulk. Predicted 0 < same-multiset < 10 % of 3,195.

P6. **Decode coverage is high.** ≥ 90 % of mismatched words decode cleanly under
    the round-trip rule (c2's `/O1` output for these shapes is a narrow
    instruction vocabulary).

## 3. What would FALSIFY "the differs are structured"

* **F1.** The top 10 clusters cover < 25 % of the 3,195 and the cluster-size
  distribution has no head (i.e. essentially every body has its own signature).
* **F2.** First-divergence index is ~uniform over body length with no mass at
  small indices — a diff that starts anywhere is a diff with no single mechanism.
* **F3.** > 30 % of mismatched words fail the round-trip decode — meaning we do
  **not** understand the layout well enough and the honest answer to the user's
  question is "no".
* **F4.** The modal substitution class is `opcode` — different instructions
  entirely, i.e. a different lowering rather than a field-level defect. That is
  still a finding, but it refutes "lower immediate complexity".

Any of F1–F4 will be reported **as measured**, not explained away.

## 4. Controls (known answers, checked on the same run)

| control | known answer |
|---|---|
| `fnbyte-differs` | **3,195** — must not grow; this lane widens no instrument that grades bytes |
| `fnbyte-exact` | **35,982** — must not shrink |
| `fnbyte-match-tu-differs` | 0 |
| scan `mismatch` | 0 |
| every differ has a signature | `fndiff-rows == fnbyte-differs` |
| edit accounting | `equal + substitute + delete == ref_words` and `equal + substitute + insert == port_words`, per row |
| decode round-trip | every word classified as decoded re-encodes to itself, asserted in a unit test over the real workload words |

The **edit accounting** identity is the one that matters: a signature computed by
a broken alignment would still produce a tidy cluster table. Trap 0 — a green
control is a statement about the population it ran over — so the identity is
checked **per row**, positively, and a break is counted and printed rather than
inferred from an absence.

## 5. What this lane will NOT do

* Ship any change to `crates/c2-core` codegen. If a cluster looks
  one-rule-shippable it is **spec'd in the report and stopped there**.
* Add partial credit anywhere. The signature is forensic; no numerator moves.
* Key anything on `IlFunction::mangled_name`.
* Duplicate lane `w-seq`'s family/mechanism taxonomy — its axis is *which C++
  idiom*, this lane's is *which bytes*.
