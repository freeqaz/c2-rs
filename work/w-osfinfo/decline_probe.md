# The `_neg` cells' clauses, read with a probe patch

Lane `w-osfinfo`. Board **#1704**'s defect and w-cfgclass §6.2's method, paying a
fifth time: **`c2rs census` reports only the fall-through blocker**, so a `_neg`
file's own verdict cannot tell a cell that hit its clause from one that hit an
earlier one. Every cell of `fixtures/cpp/wosf_handle_guard_neg.cpp` reads
`expr-cmp-ge` on an unpatched tree — the same key, ten times, saying nothing.

## The patch

Applied, run, and **reverted**. In `crates/c2-il/src/func/body/mod.rs`, the
dispatch arm

```rust
if let Ok(shape) = try_parse_osf_handle_guard(seg, p, lo) {
    disp("disp-osf-handle-guard");
    return Ok(shape);
}
```

becomes, for the duration of the run only,

```rust
{ disp("disp-osf-handle-guard"); return try_parse_osf_handle_guard(seg, p, lo); }
```

so the production's own `Block` surfaces instead of the ladder falling through to
the generic expression layer. `git checkout crates/c2-il/src/func/body/mod.rs`
reverts it; the tree this lane ships has the committing form.

## The result — TEN cells, TEN DISTINCT clauses

```text
  n1   osf-low-guard-convert-0x33
  n2   osf-high-guard-limit-0x2C
  n3   osf-flag-member-is-not-a-byte-0x33
  n4   osf-handle-member-is-not-at-offset-zero-0x30
  n5   osf-element-size-is-a-power-of-two-0x04
  n6   osf-flag-mask-is-not-2n-minus-1-0x0B
  n7   osf-success-value-is-not-the-sentinel-0x32
  n8   osf-live-member-0x53
  n9   osf-not-one-formal-free-fn-0xB9
  n10  osf-call-takes-arguments-0x33
```

No two collapse. That is the property the file exists to have, and it is checked
here rather than assumed.

## What the probe found that the cells' own comments did not

**One cell lands on a clause other than the one its comment names, and it is
recorded rather than reworded** — the same outcome w-undname §6 reported for two
of its eight.

* **n2** — "the range guard is SIGNED, so c2 emits `cmpw`" — stops on
  `osf-high-guard-limit`, **one production before**
  `osf-high-guard-is-signed-so-c2-emits-cmpw`. Writing `(int) fh < nhandle`
  puts the `2C` conversion on the LEFT operand, so the reader's `eat_load` for
  the limit meets a `2C` where it wants a `B9` and declines there. The cell is
  still a correct negative for the fact it names — the body really does emit
  `cmpw` — but the clause that catches it is the operand ORDER, not the
  signedness test. There is no cell in this file that reaches
  `osf-high-guard-is-signed-so-c2-emits-cmpw`, and saying so is better than
  implying there is.

**And one key was doing two jobs, which only the probe could show.** n6's mask
is `6` — not `2^n − 1` — and the clause reported
`osf-flag-mask-reaches-the-sign-bit`, which is a different fact about a
different mask. The key is **split** in the shipped tree
(`osf-flag-mask-is-not-2n-minus-1` and `osf-flag-mask-reaches-the-sign-bit`), so
the diagnostic now names the fact that fired. A `_neg` file that had only been
counted, rather than read per cell, would have left that key lying.

## The pair most at risk of collapsing, and it does not

**n3 and n4** both change `ioinfo`'s layout and both are read through the same
`eat_member` → `eat_deref` pair. They separate cleanly: n3 stops on the flag
read's TYPE (`osf-flag-member-is-not-a-byte`, at the `33` after the deref) and n4
on the handle member's OFFSET (`osf-handle-member-is-not-at-offset-zero`, at the
`30` before the deref). Different productions, different bytes, twenty tokens
apart.
