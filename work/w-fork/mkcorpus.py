#!/usr/bin/env python3
"""Extract N capture-cache entries into work/w-fork/corpus/<NNNNN>/.

Each case dir gets the _CL_* bundle files plus argv.txt: the captured c2 argv
with -il and -Fo re-pointed at this dir.  Nothing is mutated in the cache.

usage: mkcorpus.py <out_dir> <n> [--skip K] [--min-ex BYTES] [--source DIR]
"""
import os, sys, shutil

def main():
    out_dir = os.path.abspath(sys.argv[1])
    n = int(sys.argv[2])
    skip = 0
    min_ex = 0
    src = os.path.abspath("work/capture-cache")
    a = sys.argv[3:]
    i = 0
    while i < len(a):
        if a[i] == "--skip": skip = int(a[i+1]); i += 2
        elif a[i] == "--min-ex": min_ex = int(a[i+1]); i += 2
        elif a[i] == "--source": src = os.path.abspath(a[i+1]); i += 2
        else: raise SystemExit("bad arg " + a[i])

    if os.path.exists(out_dir):
        shutil.rmtree(out_dir)
    os.makedirs(out_dir)

    made = 0
    seen = 0
    with os.scandir(src) as it:
        for ent in it:
            if made >= n:
                break
            if not ent.is_dir():
                continue
            seen += 1
            if seen <= skip:
                continue
            meta = os.path.join(ent.path, "meta.txt")
            if not os.path.exists(meta):
                continue
            base = None
            args = []
            with open(meta) as f:
                for line in f:
                    line = line.rstrip("\n")
                    if line.startswith("base "):
                        base = line[5:]
                    elif line.startswith("arg "):
                        args.append(line[4:])
            if not base or not args:
                continue
            exf = os.path.join(ent.path, base + "ex")
            if not os.path.exists(exf):
                continue
            if min_ex and os.path.getsize(exf) < min_ex:
                continue

            case = os.path.join(out_dir, "%05d" % made)
            os.makedirs(case)
            for suf in ("ex", "gl", "sy", "in", "db"):
                p = os.path.join(ent.path, base + suf)
                if os.path.exists(p):
                    shutil.copy2(p, os.path.join(case, base + suf))
            # golden obj from the cache, for provenance only (never the gate)
            gold = os.path.join(ent.path, "out.obj")
            if os.path.exists(gold):
                shutil.copy2(gold, os.path.join(case, "cache.obj"))

            # re-point -il and -Fo at this case dir
            new = []
            j = 0
            while j < len(args):
                t = args[j]
                if t == "-il":
                    new.append("-il")
                    new.append("Z:%s/%s" % (case, base))
                    j += 2
                    continue
                if t.startswith("-Fo"):
                    new.append("-FoZ:%s/out.obj" % case)
                    j += 1
                    continue
                new.append(t)
                j += 1
            with open(os.path.join(case, "argv.txt"), "w") as f:
                f.write("\n".join(new) + "\n")
            with open(os.path.join(case, "base.txt"), "w") as f:
                f.write(base + "\n")
            with open(os.path.join(case, "origin.txt"), "w") as f:
                f.write(ent.name + "\n")
            made += 1

    print("scanned %d cache dirs, wrote %d cases to %s" % (seen, made, out_dir))
    if made < n:
        print("WARNING: requested %d, produced %d" % (n, made), file=sys.stderr)
        sys.exit(1)

main()
