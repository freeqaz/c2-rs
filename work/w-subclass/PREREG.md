# PREREG — lane `w-subclass`, board #778

Written **before any measurement on this tree**. Nothing below was read off a
run; the baseline figures are deliberately left blank and filled from the
pre-change scan in §3.

Assignment: **#778** — `PORT_CFG_CLASSES` is a flat `&[&str]` matched against a
bare census class string, so a lane that has measured *partial* coverage of a
CFG class has no way to record it. Two lanes in a row (`w-rotate` §7,
`w-sched2` §8) refused the wholesale claim and filed the gap. Give the screen a
**sub-class predicate**.

**Not in scope, declared before starting:** `cflow-loop` does **not** enter the
list in this lane, restricted or otherwise. This lane builds the mechanism; the
evidence that would fill it belongs to a loop lane.

---

## 1. The design, registered before it is written

`PORT_CFG_CLASSES` becomes a list of entries, not strings:

```rust
struct CfgClass { class: &'static str, sub: CfgSub }
enum CfgSub {
    Whole,                          // what a bare &str meant
    Keys(&'static [&'static str]),  // ONLY these census keys, EXACT equality
}
```

`admits(class, key) = self.class == class && match sub { Whole => true,
Keys(ks) => ks.contains(&key) }`.

**Narrowing is meant to be a theorem, not a hope**: `Keys` only ever *conjoins*
a term onto the predicate the bare string already had, so
`admits_Keys ⟹ admits_Whole` for the same class. The measurements below exist
to check that the implementation actually has the property the algebra claims,
on the real workload, at both ends.

**Matching is EXACT string equality on the census key, never prefix and never
substring.** A sub-class is an enumeration. An enumeration cannot silently grow
when the census mints a new key; a prefix can, and that is the wrongly-permissive
failure mode the brief names.

### 1.1 The three bounds the screen will print every scan

Computed over the same `results`, so they cost nothing and cannot go stale:

| bound | list used | meaning |
|---|---|---|
| `⊥` | every shipped entry rewritten to `Keys(&[])` | admits nothing |
| `shipped` | the list as it stands | today's answer |
| `⊤` | every class observed in the frontier cross-tab, `Whole` | admits every class present |

Registered invariant: **the reachable TU sets nest, `⊥ ⊆ shipped ⊆ ⊤`**, checked
as sets by name and not as counts.

### 1.2 The enumeration control

Each shipped `Whole` entry re-expressed as `Keys(<exactly the census keys
observed for that class in this scan>)` must reproduce the shipped verdict **TU
for TU**. This is the live exercise of the `Keys` path: without it `Keys` would
be a code path no run reaches, which this project rates worse than an absent
one (`w-rotate` §7.2, `w-frame` row F-c).

---

## 2. Registered predictions

Each is scored in the rung. **R1, R3 and R5 can lose.**

| # | prediction | why it can lose |
|---|---|---|
| **R1** | **IDENTITY.** With every entry `Whole`, the reachable frontier TU set is **identical by name** to the pre-change flat-list answer, and every `gap-metric` value printed by the scan is byte-identical | If the rewrite changes the verdict on even one TU, the mechanism is not narrower-or-equal at the identity end — it is *different*, and the whole change is unsound |
| **R2** | `reach(⊥) == 0` | A `Keys(&[])` that admits anything means the matcher ignores its key argument |
| **R3** | `reach(⊤) − reach(shipped) ≥ 8` | If it is **0**, the `⊤` bound is inert on this workload and demonstrates nothing — the narrowing would be proved only by algebra, which is what the brief forbids |
| **R4** | The enumeration control reports **0** TUs differing between `shipped` and `shipped_as_keys` | A non-zero count means `Keys` and `Whole` disagree where they must agree |
| **R5** | **≥ 5 distinct census keys** are crossed with `cflow-loop` in the 878-TU scan | If it is 1 or 2, the `(class, key)` pair is too coarse to express *"the sentinel walk at `/O1` with a one-op body"*, and the honest price is a **census-side key mint** before any screen-side restriction can bite. That is a finding this lane must report rather than paper over |
| **R6** | **MUTATION M1** — matcher changed to `Keys(_) => true`: `reach(⊥)` jumps from 0 to `reach(shipped)` and the nesting control prints **FAIL** with a count | If no control moves, the instrument cannot catch a wrongly-permissive matcher and the mechanism is not gradeable |
| **R7** | **MUTATION M2** — matcher changed from exact to `starts_with`: the prefix-witness unit test fails, naming the key it wrongly admitted | Same, for the specific mistake a hand-written allow-list invites |
| **R8** | `fnbyte-differs` stays **0**; `cargo test --workspace --release` 0 failed; `gate.sh` all PASS; `status.sh --check` PASS; `board_audit.sh` clean | Any move in `fnbyte-differs` means an emit path was touched, which this lane must not do |

## 2.1 The claim this lane does NOT make

That the mechanism makes `cflow-loop` claimable. It makes a restricted claim
**expressible and auditable**. Whether the census key is a fine enough handle to
express any *particular* loop lane's restriction is R5's question, and R5 is
registered so that a "no" is recorded as a price rather than discovered later.

---

## 3. Baseline, filled from the pre-change run (numbers not yet seen)

| figure | value |
|---|---|
| frontier size | _(pending)_ |
| `reach(shipped)` pre-change | _(pending)_ |
| reachable TU names | _(pending)_ |
| distinct `cflow-loop|*` keys | _(pending)_ |
