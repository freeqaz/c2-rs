#!/usr/bin/env python3
"""w-fltret — apply the two SCRATCH instruments, so the `_neg` cells' own
clauses can be read. Reverted with `git checkout -- crates/`; in no commit.

1. `census.rs` prints `prod=` beside every per-function line.
2. `mcall_tail.rs` CARRIES the value arm's `msc-value-*` tag past the two
   later `prod_tag("tail-void-body-does-not-end-at-the-call")` writes. The
   route re-arms that tag after a failed sequence attempt on purpose
   (w-mcall PREREG §2.2), and `eat_return_plumbing`'s own `map_err` writes it a
   second time — so a tag written inside `eat_member_value_call` is invisible
   whenever the body's FIRST statement is the member call, which is every cell
   in `wfltret_value_neg.cpp`.
"""
import sys

P1 = "crates/c2-harness/src/cli/census.rs"
s = open(P1).read()
old = """                f.seg_len,
                f.name.as_deref().unwrap_or("(unnamed)")
            );"""
new = """                f.seg_len,
                format!("{} prod={}", f.name.as_deref().unwrap_or("(unnamed)"), f.prod)
            );"""
if old in s:
    open(P1, "w").write(s.replace(old, new, 1))
elif new not in s:
    sys.exit(f"{P1}: neither the base nor the patched form is present")

P2 = "crates/c2-il/src/func/body/shapes/mcall_tail.rs"
s = open(P2).read()
pairs = [
    (
        "    if eat_byte(seg, &mut p, 0x4B) {\n",
        "    let mut wfr_seq: Option<&'static str> = None;\n    if eat_byte(seg, &mut p, 0x4B) {\n",
    ),
    (
        '            // Re-arm: the failed attempt above may have written a tag of its own.\n'
        '            prod_tag("tail-void-body-does-not-end-at-the-call");',
        '            let t = crate::func::body::prod_site();\n'
        '            if t.starts_with("msc-value-") {\n'
        '                wfr_seq = Some(t);\n'
        '            }\n'
        '            prod_tag("tail-void-body-does-not-end-at-the-call");',
    ),
    (
        "        eat_return_plumbing(seg, &mut p, false, depth)\n"
        '            .map_err(|_| prod_tag("tail-void-body-does-not-end-at-the-call"))?;',
        "        eat_return_plumbing(seg, &mut p, false, depth)\n"
        '            .map_err(|_| prod_tag(wfr_seq.unwrap_or("tail-void-body-does-not-end-at-the-call")))?;',
    ),
]
for old, new in pairs:
    if old not in s:
        sys.exit(f"{P2}: anchor not found:\n{old[:80]}")
    s = s.replace(old, new, 1)
open(P2, "w").write(s)
print("scratch applied to census.rs and mcall_tail.rs")
