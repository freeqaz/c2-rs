#!/usr/bin/env python3
"""attrib.py — how often is the folding rule's owner attribution WRONG?

`model.named_bodies` binds a name to only ~70 % of `.ex` body segments; the
pipeline's `ref_graph` folds each unnamed segment onto the nearest *preceding*
named one.  That is the dominant uncertainty in this measurement, so it gets
measured rather than assumed.

Independent owner channel: a function-local static's `.gl` name carries its
owner's decorated name inside it —

    ??_B?4??SetType@RndTex@@UAAXVSymbol@@@Z@54        (the init guard)
    ?types@?4??SetType@RndTex@@UAAXVSymbol@@@Z@4PAV…  (the variable)

so any segment that references such a name declares whose body it is.  On the
subset of segments where that channel fires we can grade the folding rule
directly:  folded-owner == local-static owner ?

    usage: attrib.py <ilroot> <tulist> [N]
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "..", "pipeline"))
import il      # noqa: E402
import model   # noqa: E402
import detect  # noqa: E402

_LOCAL = re.compile(r"^\?(?:\?_B)?\?*[^@]*@?\?\d\?\?(.+)@\d+[A-Za-z0-9_@$?]*$")


def owner_from_local(n):
    """`??_B?4??OWNER@54` / `?v@?1??OWNER@4V3@A` -> `?OWNER`, else None."""
    i = n.find("??")
    if i < 0:
        return None
    j = n.find("?", 0)
    m = re.match(r"^\?\?_B\?\d\?\?(.*)$", n)
    if not m:
        m = re.match(r"^\?[^@]*@\?\d\?\?(.*)$", n)
    if not m:
        return None
    rest = m.group(1)
    # strip the trailing `@<digits>...` discriminator/type suffix
    k = rest.rfind("@")
    while k > 0:
        if rest[k + 1:k + 2].isdigit():
            return "?" + rest[:k]
        k = rest.rfind("@", 0, k)
    return None


def main():
    ilroot, tulist = sys.argv[1:3]
    lim = int(sys.argv[3]) if len(sys.argv) > 3 else 10**9
    srcs = [l.strip() for l in open(tulist) if l.strip()][:lim]
    tot = agree = 0
    seg_named = seg_all = 0
    examples = []
    for k, src in enumerate(srcs):
        d = os.path.join(ilroot, detect.slug(src))
        if not os.path.exists(os.path.join(d, "gl")):
            continue
        glb = open(os.path.join(d, "gl"), "rb").read()
        exb = open(os.path.join(d, "ex"), "rb").read()
        Nf = model.named_bodies(glb, exb)
        idx = il.gl_symbol_index(glb)
        loc = {t: owner_from_local(n) for t, n in idx.items()}
        loc = {t: v for t, v in loc.items() if v}
        n = len(exb)
        owner = None
        for (s, e) in il.segments(exb):
            seg_all += 1
            nm = Nf.get(s)
            if nm is not None:
                owner = nm
                seg_named += 1
                continue
            if owner is None:
                continue
            found = set()
            for p in range(s, min(e, n - 1)):
                b1 = exb[p + 1]
                if b1 & 0x80:
                    if p + 3 >= n:
                        continue
                    tok = (exb[p] << 24) | (b1 << 16) | (exb[p + 2] << 8) | exb[p + 3]
                else:
                    tok = (exb[p] << 8) | b1
                v = loc.get(tok)
                if v:
                    found.add(v)
            if len(found) != 1:
                continue
            real = found.pop()
            tot += 1
            if real == owner:
                agree += 1
            elif len(examples) < 12:
                examples.append((src, owner, real))
        if (k + 1) % 100 == 0:
            print("... %d/%d  graded=%d agree=%d" % (k + 1, len(srcs), tot, agree), flush=True)
    print("\nsegments: %d total, %d named (%.1f%%)" % (seg_all, seg_named, 100.0 * seg_named / max(seg_all, 1)))
    print("UNNAMED segments gradeable by the local-static channel: %d" % tot)
    print("folding rule correct on those: %d  (%.1f%%)" % (agree, 100.0 * agree / max(tot, 1)))
    for e in examples:
        print("   MISATTRIB %s\n      folded->%s\n      real   ->%s" % e)


if __name__ == "__main__":
    main()
