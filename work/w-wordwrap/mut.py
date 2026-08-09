#!/usr/bin/env python3
"""Apply ONE named must-fail mutation to `global_store_leaf.rs` in place.

Each is a whole-CONJUNCTION deletion, not a single clause: `w-xtea2` #2665
measured that a cell fenced by several clauses grades NONE of them, because
deleting one leaves the others refusing it. `M2` below is that lesson applied —
the `2C` check and the type-restatement check both refuse `conv_neg`, so both
go together or the mutation comes back green for the wrong reason.

    work/w-wordwrap/mut.py M1
"""
import sys

REC = "crates/c2-il/src/func/body/shapes/global_store_leaf.rs"
EMIT = "crates/c2-core/src/codegen/global_store_leaf.rs"

MUTS = {
    # The value must be the FORMAL. Cell `gg_neg` — a global source is spelled
    # `B9` exactly as a formal is, so this one clause is the whole fence.
    "M1": (REC, """    if val_tok != params[0] {
        return Err(blk(seg, p, "gstore-value-is-not-the-formal"));
    }
""", ""),
    # The no-conversion clause AND the type restatement, TOGETHER. Cell
    # `conv_neg` is refused by either alone (#2665).
    "M2": (REC, """    if seg.get(p) == Some(&0x2C) {
        return Err(blk(seg, p, "gstore-value-carries-a-conversion"));
    }""", """    if false && seg.get(p) == Some(&0x2C) {
        return Err(blk(seg, p, "gstore-value-carries-a-conversion"));
    }
    let mut _skip = p;
    if seg.get(_skip) == Some(&0x2C) {
        _skip += 1;
        if let Some((_, _, _, cw)) = read_type(seg, _skip) {
            _skip += cw;
        }
        if crate::func::readers::read_varint(seg, &mut _skip).is_some() {
            p = _skip;
        }
    }"""),
    # ...and its other half: the store TYPE no longer has to restate the value's.
    "M2b": (REC, """    if seg.get(p..p + tw) != Some(&ty[..]) {
        return Err(blk(seg, p, "gstore-store-type-does-not-restate-the-value-type"));
    }
    p += tw;""", """    let (_, _, _, tw2) = read_type(seg, p).ok_or(blk(seg, p, "gstore-store-type"))?;
    let _ = &ty;
    p += tw2;"""),
    # Exactly one formal in r3, TOGETHER WITH the value-token comparison. Cell
    # `second_neg` is refused by either alone, and the first run of this grid
    # proved it: deleting the arity fence on its own came back GREEN, because
    # `val_tok != params[0]` still refused the body whose value is params[1].
    # #2665's shape, found here rather than quoted — the repair is merging the
    # clauses into one mutation, not adding a cell.
    "M3": (REC, """    if params.len() != 1 || formals.len() != 1 || params[0] != formals[0] {
        return Err(blk(seg, start, "gstore-not-exactly-one-formal-in-r3"));
    }""", """    if params.is_empty() || formals.is_empty() || params[0] != formals[0] {
        return Err(blk(seg, start, "gstore-not-exactly-one-formal-in-r3"));
    }"""),
    "M3b": (REC, """    if val_tok != params[0] {
        return Err(blk(seg, p, "gstore-value-is-not-the-formal"));
    }
""", """    if !params.contains(&val_tok) {
        return Err(blk(seg, p, "gstore-value-is-not-the-formal"));
    }
"""),
    # The FP rows' absence from the width table. Cell `float_neg`.
    "M4": (REC, "    (0x88, 0x82, 8), // unsigned long long",
           "    (0x88, 0x82, 8), // unsigned long long\n    (0x86, 0x45, 4), // MUTATION M4: float, which c2 stores with `stfs f1`"),
    # The mode gate. Graded at `/Od`, where c2 emits five words.
    "M5": (REC, """    if opt_word_mode(opt_word_at(seg)).is_none() {
        return Err(blk(seg, start, "gstore-mode-not-modelled"));
    }""", "    let _ = opt_word_mode(opt_word_at(seg));"),
}


def main(argv):
    name = argv[0]
    if name not in MUTS:
        print("unknown mutation %r; have %s" % (name, ", ".join(sorted(MUTS))))
        return 2
    path, old, new = MUTS[name]
    s = open(path).read()
    if old not in s:
        print("%s: ANCHOR NOT FOUND — the mutation is void, not green" % name)
        return 3
    open(path, "w").write(s.replace(old, new, 1))
    print("%s applied to %s" % (name, path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
