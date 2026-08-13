# DRIFT — `decode_causes` does not agree with `decodes()` on a TU the NARROWED gate exempts

Lane w-fencecount. **Measured, not argued** — a throwaway probe (a temporary
integration test, run and deleted; nothing under `crates/` ships from it)
captured `fixtures/cpp/wfence2_kept_local_callee.cpp` at the `/O1` profile
`tests/gate_cause.rs` uses (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`) and
called both predicates on the same bundle.

```text
PROBE decodes=true  functions_is_some=true  whole_tu=false
      first=Some("unclaimed-gl-symbol")
      causes=["unclaimed-gl-symbol", "locally-defined-callee"]
      invariant_holds=false
```

## What that means

`crates/c2-il/src/func/diag.rs`'s module docs state the anti-drift contract in
as many words: *"the struct carries `DecodeCauses::decodes` … and the invariant
`causes.is_empty() == decodes` is checkable per TU by any caller and is asserted
by this crate's tests."* On this bundle it **does not hold**: the gate accepts
(`IlBundle::functions()` is `Some`, and the TU is a whole-obj `match` on the
scan) while the diagnostic reports two firing causes.

The `locally-defined-callee` half is the one this lane predicted (PREREG D11)
and its mechanism is in the tree:

| | predicate asked | site |
|---|---|---|
| the **gate** | `bind::callee_defined_here_unmodelled(f, defined, exempt)` — the **w-fence2 narrowing**, with `exempt = gl::plain_external_names_among(...)` at `/O1` | `crates/c2-il/src/func/bundle.rs` ~2302–2339 |
| the **diagnostic** | `bind::callee_defined_here(f, defined)` — the **broad** pre-w-fence2 form, no exemption | `crates/c2-il/src/func/diag.rs` ~488–493 |

So every TU the narrowing was built to admit reports a cause it is not held by.
`w-fence2`'s own doc says the exemption is what converted `vsnprnc.cpp`; the
diagnostic was not re-pointed with it.

The `unclaimed-gl-symbol` half was **not** predicted and is a second,
independent divergence in the same direction (diagnostic stricter than gate) on
the same bundle.

## Why it does not corrupt the `fence-blocks-exact` counter

`gap/scan.rs` populates `gate_cause`/`gate_causes` **only** inside the
`if !captured.bundle.decodes()` arm, so a decoding TU carries an empty cause
list into the report no matter what `decode_causes` would have said. The
counter's `class_disagree` and `on_match_tu` controls exist for exactly this
question and both read **0** on the 878-TU workload and on the control fixture.
The divergence is therefore **latent**: it can be reached by any caller that
asks `decode_causes` directly (`tests/gate_cause.rs` does), and not by the scan.

## Deliberately NOT repaired here

`decode_causes` is a shared surface with callers in three test targets and one
scan field; narrowing or re-pointing it is a change to what every existing
reader of the cause vocabulary sees — the shared-predicate erasure this repo has
recorded three times. This lane changes zero `c2-il` behaviour by design. What
it does instead is **carry the caveat where the reading is made**: the
`locally-defined-callee` row of the FENCE-BLOCKS-EXACT block and
`GapReport::fence_blocks`' doc both say the row is the diagnostic's broad re-ask
and can fire where the narrowed gate exempts.

No test is added pinning either behaviour, because pinning the current one would
make the divergence look intended.
