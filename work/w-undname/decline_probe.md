# The decline probe — reading each `_neg` cell's OWN clause

`c2rs census` reports only the **fall-through** blocker for a function, so a
`_neg` file's own verdict cannot tell a cell that hit its intended clause from
one that hit an earlier one. Board **#1704**'s defect; w-cfgclass §6.2's method,
paying a fourth time.

The probe: `crates/c2-il/src/func/body/mod.rs`'s dispatch arm

```rust
    if let Ok(shape) = try_parse_alloc_init_or_fail(seg, p, lo) {
        disp("disp-alloc-init-or-fail");
        return Ok(shape);
    }
```

temporarily rewritten to **propagate** the `Err` instead of falling through:

```rust
    match try_parse_alloc_init_or_fail(seg, p, lo) {
        Ok(shape) => { disp("disp-alloc-init-or-fail"); return Ok(shape); }
        Err(e) => return Err(e),
    }
```

**Applied, run, and reverted** — `git diff` on that file is empty at the tip.

## The result: eight cells, eight distinct clauses

```text
  [0] n1  aiof-test-is-signed-so-c2-emits-cmpwi
  [1] n2  aiof-object-designator
  [2] n3  aiof-call-arglist-close
  [3] n4  aiof-status-store-is-a-word-not-a-byte
  [4] n5  aiof-link-store-is-a-different-member
  [5] n6  aiof-vtable-designator
  [6] n7  aiof-not-one-formal-member-fn
  [7] n8  aiof-test-literal-not-zero
```

Two of the eight land on a clause other than the one the cell's comment names,
and both are recorded rather than reworded:

* **n2** (a FREE allocation function) was expected on
  `aiof-object-not-a-this-push`; it stops one production earlier, on
  `aiof-object-designator`, because a free call pushes **two** designators where
  a member call on a global pushes three — the reader runs out of `26`s before it
  reaches the `99`. The clause it does hit is still the one the cell is about,
  and it is still a clause no other cell reaches.
* **n6** (the third store writes a literal) hits `aiof-vtable-designator` at the
  `33`, which is the same reading one byte earlier.

**The pair most at risk of collapsing into one clause is n1 and n8** — both
change the entry test — and they do not: n1 stops on the operand's SIGN before
the literal is read, n8 on the literal after the sign has passed.
