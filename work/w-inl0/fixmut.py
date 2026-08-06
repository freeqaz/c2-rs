#!/usr/bin/env python3
"""fixmut.py — lane w-inl0 scratch: retarget mutation M2 at `elide.rs`.

`IlFunction::base` is private, so M2's first form (a fabricated seed row in the
harness) did not compile. The guard it is supposed to remove lives one crate
down anyway: `Reduction::NoEffectCall` contributes `(seeds=false, link)`, and
making it `(seeds=true, link)` is exactly "admit the body without asking about
its callee".
"""
p = 'work/w-inl0/mutate.sh'
s = open(p).read()
s = s.replace(
    'NOEFF=crates/c2-il/src/func/body/shapes/no_effect.rs\nFNB=crates/c2-harness/src/gap/fnbytes.rs\n\nrestore() { git checkout -- "$NOEFF" "$FNB"; }',
    'NOEFF=crates/c2-il/src/func/body/shapes/no_effect.rs\nELIDE=crates/c2-core/src/elide.rs\n\nrestore() { git checkout -- "$NOEFF" "$ELIDE"; }')
start = s.index('M2)\n')
end = s.index('M3)\n')
new = '''M2)
    file="$ELIDE"
    python3 - <<'PY'
p = 'crates/c2-core/src/elide.rs'
s = open(p).read()
old = "                    Reduction::NoEffectCall(callee) => (false, Some(callee)),"
new = "                    Reduction::NoEffectCall(callee) => (true, Some(callee)), // M2"
assert old in s, 'M2 did not find its guard'
open(p, 'w').write(s.replace(old, new))
PY
    test -n "$(git diff --stat -- "$file")" || { echo "M2 DID NOT MUTATE $file"; exit 1; }
    echo "-- M2 mutated $file:"; git diff --stat -- "$file"
    cargo build --release -p c2-harness 2>&1 | grep -E '^error' && exit 1
    work/w-inl0/scan.sh work/w-inl0/mut_M2_scan
    grep -E "fnbyte-(differs|exact|noeffect-ref-other|noeffect-ref-blr|noeffect-admitted) " work/w-inl0/mut_M2_scan.txt
    cargo test --release -p c2-harness --test dead_temp_elision 2>&1 | grep -E "^test |test result"
    ;;
'''
s = s[:start] + new + s[end:]
open(p, 'w').write(s)
print("patched")
