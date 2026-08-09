#!/usr/bin/env python3
"""w-readpx — the whole-workload reader-refusal ranking on the EMITTED column,
with the three replication tests this board now requires and the BYTE verdict
beside every row.

Columns, and why each is here:

  emitted   the symbol count. #2020: the body column and the emitted column
            disagree by 13x, so nothing is ranked on bodies here.
  dTU       distinct TUs. #2000's test: `emitted == dTU` is TU replication.
  dname     distinct mangled names. #2022: an emitted column CONCENTRATES,
            it does not discount.
  dstem     distinct DEMANGLED stems -- template arguments and the parameter
            list removed. #2246: a mangled name embeds its template
            arguments, so `dname` is structurally blind to a template and
            `emitted == dTU` fails open on one. This is the column that
            catches the eighth artifact.
  exact/differs/refused   the ORACLE's own per-function byte verdict on the
            same rows. #2081/#2095: a conversion count is not a result unless
            it is crossed with the byte judge.

Demangling is `llvm-undname` (present on this box); a name it cannot demangle
falls back to itself, and the count of those is printed rather than hidden.
"""
import collections
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
STEM = sys.argv[1] if len(sys.argv) > 1 else "hex"
TOPN = int(sys.argv[2]) if len(sys.argv) > 2 else 25


def rows(path):
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("READPX\t"):
            continue
        f = line.rstrip("\n").split("\t")
        if len(f) < 10:
            continue
        yield {
            "tu": f[1], "name": f[2], "fnb": f[3], "key": f[4],
            "cflow": f[5], "off": f[6], "cls": f[7], "bytes": int(f[8]),
            "byte": f[9], "win": f[10] if len(f) > 10 else "",
        }


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


def stems(names):
    """mangled -> demangled STEM (qualified name, no template args, no params)."""
    proc = subprocess.run(["llvm-undname"], input="\n".join(names),
                          capture_output=True, text=True)
    lines = [l for l in proc.stdout.split("\n")]
    out, i, failed = {}, 0, 0
    # llvm-undname echoes the input line then prints the demangling, then a
    # blank line. Walk that triple rather than assuming a 1:1 mapping.
    j = 0
    while j < len(lines) and i < len(names):
        if lines[j].strip() == names[i]:
            dem = lines[j + 1] if j + 1 < len(lines) else ""
            out[names[i]] = dem
            i += 1
            j += 2
            while j < len(lines) and lines[j].strip() == "":
                j += 1
        else:
            j += 1
    res = {}
    for n in names:
        d = out.get(n, "")
        if not d or d == n:
            failed += 1
            res[n] = n
            continue
        d = strip_templates(d)
        # cut the parameter list at the LAST top-level '(' -- a function
        # pointer parameter can contain one, so scan from the right.
        depth = 0
        cut = len(d)
        for k in range(len(d) - 1, -1, -1):
            if d[k] == ")":
                depth += 1
            elif d[k] == "(":
                depth -= 1
                if depth == 0:
                    cut = k
                    break
        d = d[:cut].strip()
        # drop leading storage/return words: keep the last whitespace-separated
        # token, which is the qualified name.
        d = d.split(" ")[-1] if " " in d else d
        res[n] = d or n
    return res, failed


def main():
    all_rows = list(rows(os.path.join(HERE, STEM + ".err")))
    print("READPX rows: %d" % len(all_rows))
    blk = [r for r in all_rows if r["cls"] == "blk"]
    inc = [r for r in all_rows if r["cls"] == "in"]
    unb = [r for r in all_rows if r["cls"] == "?"]
    print("emitted denominator %d = in-class %d + blocked %d + unbound %d"
          % (len(all_rows), len(inc), len(blk), len(unb)))
    assert len(inc) + len(blk) + len(unb) == len(all_rows)
    print("COLUMN-SUM ASSERT OK")
    fb = collections.Counter(r["fnb"] for r in all_rows)
    print("byte verdict over the WHOLE emitted denominator:")
    for k, v in sorted(fb.items(), key=lambda kv: -kv[1]):
        print("   %-26s %6d" % (k, v))
    fbb = collections.Counter(r["fnb"] for r in blk)
    print("byte verdict over the BLOCKED emitted rows:")
    for k, v in sorted(fbb.items(), key=lambda kv: -kv[1]):
        print("   %-26s %6d" % (k, v))

    by = collections.defaultdict(list)
    for r in blk:
        by[r["key"]].append(r)
    ranked = sorted(by.items(), key=lambda kv: -len(kv[1]))[:TOPN]

    want = set()
    for _, rs in ranked:
        want |= {r["name"] for r in rs}
    smap, failed = stems(sorted(want))
    print("\ndemangled %d distinct names for the top %d keys; %d not demangleable "
          "(fall back to the mangled name)" % (len(want), TOPN, failed))

    print("\n| rank | key | emitted | dTU | dname | dstem | e/TU | exact | differs | refused |")
    print("|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    for i, (k, rs) in enumerate(ranked, 1):
        dtu = len({r["tu"] for r in rs})
        dn = len({r["name"] for r in rs})
        ds = len({smap[r["name"]] for r in rs})
        c = collections.Counter(r["fnb"] for r in rs)
        print("| %d | `%s` | %d | %d | %d | **%d** | %.2f | %d | %d | %d |"
              % (i, k, len(rs), dtu, dn, ds, len(rs) / dtu,
                 c.get("fnbyte-exact", 0), c.get("fnbyte-differs", 0)
                 + c.get("fnbyte-reloc-differs", 0), c.get("fnbyte-refused", 0)))

    print("\n--- top keys: blocking byte histogram, cflow_off, and 3 bodies each ---")
    for i, (k, rs) in enumerate(ranked[:10], 1):
        bh = collections.Counter(r["byte"] for r in rs)
        oh = collections.Counter(r["off"] or "-" for r in rs)
        ch = collections.Counter(r["cflow"] for r in rs)
        print("\n### %d. `%s` — %d emitted, %d TUs, %d names, %d stems"
              % (i, k, len(rs), len({r['tu'] for r in rs}),
                 len({r['name'] for r in rs}), len({smap[r['name']] for r in rs})))
        print("  blocking byte : " + " ".join("%s*%d" % (b, n)
                                              for b, n in bh.most_common(6)))
        print("  cflow_off     : " + " ".join("%s*%d" % (b, n)
                                              for b, n in oh.most_common(6)))
        print("  cflow         : " + " ".join("%s*%d" % (b, n)
                                              for b, n in ch.most_common(6)))
        seen, shown = set(), 0
        for r in sorted(rs, key=lambda r: (r["tu"], r["name"])):
            st = smap[r["name"]]
            if st in seen:
                continue
            seen.add(st)
            shown += 1
            print("  READ %d: %s" % (shown, r["name"]))
            print("        stem=%s" % st)
            print("        tu=%s  cflow=%s off=%s  fnbyte=%s  %dB"
                  % (r["tu"], r["cflow"], r["off"] or "-",
                     r["fnb"].replace("fnbyte-", ""), r["bytes"]))
            print("        hex=%s" % r["win"])
            if shown == 3:
                break


main()
