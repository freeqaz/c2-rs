#!/usr/bin/env python3
"""w-readpx — name the constructs behind the keys whose `dstem` collapsed, and
size `param-width-undetermined:mid` (wb-eh R1) properly.

`dname` and `emitted == dTU` both pass on a template, because a mangled name
embeds its template arguments (#2246). This prints the STEMS themselves, so a
collapsed row is named rather than merely counted.
"""
import collections
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "hex"

KEYS = [
    "expr-call-in-expr-recv-object-then-call-recv-object-more",
    "expr-call-in-expr-recv-load-then-bit-and-and-branch-more",
    "expr-call-in-expr-recv-load-then-call-recv-load-and-deref-load-more",
    "return-scope-close-cflow-label",
    "expr-call-in-expr-recv-load-whole",
    "expr-call-in-expr-recv-object-then-deref-load-more",
    "expr-intrinsic-memset",
    "param-width-undetermined:mid",
    "param-multi-reg:mid",
    "expr-op-0x28",
]


def strip_templates(s):
    out, depth = [], 0
    for ch in s:
        if ch == "<":
            depth += 1
        elif ch == ">":
            depth = max(0, depth - 1)
        elif depth == 0:
            out.append(ch)
    return "".join(out)


def demangle(names):
    p = subprocess.run(["llvm-undname"], input="\n".join(names),
                       capture_output=True, text=True)
    lines = p.stdout.split("\n")
    out, i, j = {}, 0, 0
    while j < len(lines) and i < len(names):
        if lines[j].strip() == names[i]:
            out[names[i]] = lines[j + 1] if j + 1 < len(lines) else ""
            i += 1
            j += 2
            while j < len(lines) and lines[j].strip() == "":
                j += 1
        else:
            j += 1
    return out


def stem_of(dem, mangled):
    if not dem or dem == mangled:
        return mangled
    d = strip_templates(dem)
    depth, cut = 0, len(d)
    for k in range(len(d) - 1, -1, -1):
        if d[k] == ")":
            depth += 1
        elif d[k] == "(":
            depth -= 1
            if depth == 0:
                cut = k
                break
    d = d[:cut].strip()
    return (d.split(" ")[-1] if " " in d else d) or mangled


def main():
    rows = []
    for line in open(os.path.join(HERE, STEM + ".err"),
                     encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 10 or f[7] != "blk":
            continue
        rows.append(f)
    by = collections.defaultdict(list)
    for f in rows:
        by[f[4]].append(f)

    want = sorted({f[2] for k in KEYS for f in by.get(k, [])})
    dem = demangle(want)

    for k in KEYS:
        rs = by.get(k, [])
        if not rs:
            print("\n## `%s` — NOT PRESENT on the emitted column\n" % k)
            continue
        st = collections.Counter(stem_of(dem.get(f[2], ""), f[2]) for f in rs)
        tus = collections.Counter(f[1] for f in rs)
        print("\n## `%s` — %d emitted, %d TUs, %d names, **%d stems**"
              % (k, len(rs), len(tus), len({f[2] for f in rs}), len(st)))
        print("   stems: " + " · ".join("`%s` ×%d" % (s, n)
                                        for s, n in st.most_common(8)))
        if len(st) > 8:
            print("   (+%d more stems, %d emitted)"
                  % (len(st) - 8, sum(n for _, n in st.most_common()[8:])))
        print("   top TUs: " + " · ".join("%s ×%d" % (t.split('/')[-1], n)
                                          for t, n in tus.most_common(6)))
        # three bodies, one per distinct stem where possible
        seen, shown = set(), 0
        for f in sorted(rs, key=lambda f: (f[1], f[2])):
            s = stem_of(dem.get(f[2], ""), f[2])
            if s in seen:
                continue
            seen.add(s)
            shown += 1
            print("   READ %d  %s" % (shown, f[2]))
            print("           demangled = %s" % (dem.get(f[2], "?")))
            print("           tu=%s cflow=%s off=%s %sB fnbyte=%s"
                  % (f[1], f[5], f[6] or "-", f[8], f[3].replace("fnbyte-", "")))
            print("           hex=%s" % (f[10] if len(f) > 10 else ""))
            if shown == 3:
                break


main()
