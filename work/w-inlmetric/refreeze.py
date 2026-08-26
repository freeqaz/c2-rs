#!/usr/bin/env python3
"""refreeze.py -- re-freeze INLINE-P's SAMPLE-B hold-out BY CONTENT HASH.

Board #3045: `work/w-inline/sample_b.txt` is frozen by TU NAME. dc3 is a live
repo, so the sources moved underneath a byte-identical list and the graded
population fell 10.8% with no line changing. This writes the missing half: the
sha256 of every listed TU's source AS OF the recorded dc3 stamp, so a later
re-run can say WHICH files moved rather than only that the number did.

It does not change which TUs are in the sample. Re-freezing by content is not
re-selecting: the list stays byte-identical and gains a hash column.

Usage: refreeze.py <dc3-root> <sample.txt> <out.tsv>
"""
import hashlib, os, subprocess, sys

def main(argv):
    dc3, lst, out = argv[0], argv[1], argv[2]
    stamp = subprocess.run(["git", "-C", dc3, "rev-parse", "HEAD"],
                           capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "-C", dc3, "status", "--porcelain"],
                           capture_output=True, text=True).stdout.splitlines()
    rows, missing = [], 0
    for line in open(lst):
        rel = line.rstrip("\n")
        if not rel:
            continue
        p = os.path.join(dc3, rel)
        if not os.path.exists(p):
            rows.append((rel, "MISSING", "-"))
            missing += 1
            continue
        b = open(p, "rb").read()
        rows.append((rel, hashlib.sha256(b).hexdigest(), str(len(b))))
    with open(out, "w") as f:
        f.write(f"# SAMPLE-B re-frozen BY CONTENT -- board #3045's named fix.\n")
        f.write(f"# dc3 stamp: {stamp}  dirty: {len(dirty)}\n")
        f.write(f"# source list: {lst} (byte-identical; this file only adds hashes)\n")
        f.write(f"# rows: {len(rows)}  missing: {missing}\n")
        f.write("tu\tsha256\tbytes\n")
        for r in rows:
            f.write("\t".join(r) + "\n")
    print(f"dc3 {stamp} dirty={len(dirty)}  rows={len(rows)}  missing={missing}")
    listhash = hashlib.sha256(open(lst, "rb").read()).hexdigest()
    cat = hashlib.sha256("".join(r[1] for r in rows).encode()).hexdigest()
    print(f"sample_b.txt sha256      = {listhash}")
    print(f"CONTENT-FREEZE sha256    = {cat}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
