#!/usr/bin/env python3
"""patch_reloc.py — lane w-inl0 scratch: add `fnbyte-elided-ref-reloc`.

w-drop3 (boards #984–#986) measured that a `/Gy` call word carries the same
placeholder displacement whatever it calls, so byte equality on a call word says
nothing about whom it calls. An **elided** body cannot be affected — its whole
output is one `4e800020`, which is not a call word and takes no relocation — and
this counter is the positive count that says so instead of the argument.
"""
p = 'crates/c2-harness/src/gap/fnbytes.rs'
s = open(p).read()
old = '''                *res.emit.entry("fnbyte-elided".into()).or_insert(0) += 1;
                if v == FnByte::Exact {
                    *res.emit.entry("fnbyte-elided-exact".into()).or_insert(0) += 1;
                }'''
new = '''                *res.emit.entry("fnbyte-elided".into()).or_insert(0) += 1;
                if v == FnByte::Exact {
                    *res.emit.entry("fnbyte-elided-exact".into()).or_insert(0) += 1;
                    // **The relocation-target caveat, answered rather than
                    // argued** (lane `w-drop3`, boards #984-#986): a `/Gy` call
                    // word carries the same placeholder displacement whatever it
                    // calls, so byte equality on a call word says nothing about
                    // WHOM it calls, and 861 bodies FBM credits as exact
                    // relocate against the wrong symbol.
                    //
                    // An elided body cannot be one of them, and this is the
                    // positive count that says so: mechanism E's whole output is
                    // the single word `4e800020`, which is not a call word and
                    // takes no relocation. **Known answer 0** - a nonzero here
                    // means the port credited an elision for a c2 body that
                    // still relocates, which would be a wrong emit of exactly
                    // the kind `-calltarget-disagree` was built to see.
                    if relocs.get(name.as_str()).copied().unwrap_or(0) > 0 {
                        *res.emit
                            .entry("fnbyte-elided-ref-reloc".into())
                            .or_insert(0) += 1;
                    }
                }'''
assert old in s
open(p, 'w').write(s.replace(old, new))

p = 'crates/c2-harness/src/gap/factors.rs'
s = open(p).read()
old = '''            for k in ["fnbyte-elided", "fnbyte-elided-exact", "fnbyte-name-disagree"] {'''
new = '''            // `fnbyte-elided-ref-reloc` is w-drop3's caveat closed for this
            // population: known answer **0**, because an elided body is one
            // `4e800020` and carries no relocation for a symbol to disagree
            // about. Printed, not inferred.
            for k in [
                "fnbyte-elided",
                "fnbyte-elided-exact",
                "fnbyte-elided-ref-reloc",
                "fnbyte-name-disagree",
            ] {'''
assert old in s
open(p, 'w').write(s.replace(old, new))
print("patched")
