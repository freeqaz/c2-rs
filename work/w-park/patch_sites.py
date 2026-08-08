#!/usr/bin/env python3
"""One-shot: thread `ArgSite` through `tail_call_shape`'s ten call sites."""
edits = {
    'crates/c2-il/src/func/body/shapes/mcall_tail.rs': [
        (544, 'callee_tok, p)', 'callee_tok, p, ArgSite::Tail)'),
        (620, 'callee_tok, p)', 'callee_tok, p, ArgSite::Tail)'),
    ],
    'crates/c2-il/src/func/body/shapes/mcall_chain.rs': [
        (277, 'methods.len() - 1], p)', 'methods.len() - 1], p, ArgSite::Tail)'),
    ],
    'crates/c2-il/src/func/body/shapes/calls.rs': [
        (1329, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Tail)'),
        (1382, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Tail)'),
        (1390, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Tail)'),
        (1445, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Tail)'),
        (1457, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Tail)'),
        (1738, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Tail)'),
        (1753, 'callee_tok, *p)', 'callee_tok, *p, ArgSite::Sequence)'),
    ],
}
for path, es in edits.items():
    lines = open(path).read().split('\n')
    for ln, old, new in es:
        i = ln - 1
        assert old in lines[i], (path, ln, lines[i])
        lines[i] = lines[i].replace(old, new)
    open(path, 'w').write('\n'.join(lines))
    print("patched", path)
