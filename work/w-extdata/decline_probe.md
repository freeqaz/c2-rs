# The decline probe — how the eight negative cells were checked

`c2rs census` reports the **fall-through** blocker (`expr-cmp-eq` for every one
of these), not the recognizer's own decline clause, so a `_neg` file's verdict
cannot tell "this cell hit the clause it was written for" from "this cell hit an
earlier clause and the one under test was never reached". That is w-cfgclass
§6.2's method and board **#1704**'s defect — two cells sharing one clause is a
residue nobody can size.

The check was made by patching the dispatch hook to print the recognizer's own
`Block::ctx` on decline, running `c2rs census` at `/O1` over the `_neg` file, and
**reverting the patch**. The patch, verbatim:

```rust
// crates/c2-il/src/func/body/mod.rs — applied, run, reverted
match try_parse_guard_chain_shared_tail(seg, p, lo) {
    Ok(shape) => { disp("disp-guard-chain-shared-tail"); return Ok(shape); }
    Err(b) => { if std::env::var("GCST_PROBE").is_ok() {
        eprintln!("GCST-DECLINE {}", b.ctx); } }
}
```

Result — **eight cells, eight distinct clauses, none repeated**:

```text
  n0  gcst-formals-not-6
  n1  gcst-guard-is-signed-so-c2-emits-cmpwi
  n2  gcst-arms-return-different-values
  n3  gcst-arms-call-different-reporter
  n4  gcst-store-is-a-word-not-a-halfword
  n5  gcst-fnaddr-no-decay
  n6  gcst-arms-call-different-errno
  n7  gcst-arms-share-their-literal
```

`n3` and `n6` are the pair most at risk of collapsing into one clause — both move
a callee between the arms — and they do not: `n3` moves the reporter that is in
the **shared tail** and `n6` moves the call each arm makes **before** the merge.
