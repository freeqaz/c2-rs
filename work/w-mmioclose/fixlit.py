#!/usr/bin/env python3
"""One-shot: add `inlinable: None` to the eleven test-only `IlFunction` literals
that do not go through `IlFunction::base`. Kept so the edit is reproducible."""
sites = {
    'crates/c2-core/src/codegen/leaf/addr.rs': [90],
    'crates/c2-core/src/codegen/leaf/load.rs': [111, 168],
    'crates/c2-core/src/codegen/leaf/store.rs': [1228, 1347, 1510, 1768],
    'crates/c2-core/src/codegen/straightline.rs': [711, 1090, 1143],
    'crates/c2-core/src/codegen/testutil.rs': [15],
}
for p, lines in sites.items():
    src = open(p).read().split('\n')
    for ln in sorted(lines, reverse=True):
        i = ln - 1
        indent = ' ' * (len(src[i]) - len(src[i].lstrip()) + 4)
        src.insert(i + 1, indent + "inlinable: None,")
    open(p, 'w').write('\n'.join(src))
print("done")
