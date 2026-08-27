#!/usr/bin/env python3
"""verify_scrub.py — grade the scrub against GIT'S OWN stored blobs.

Deliberately NOT the scrubber marking its own homework. `scrub.py` verified
its invariants against the bytes it had already read into memory; this reads
the **pre** side back out of the object database (`git show <rev>:<path>`) and
the **post** side off disk, and re-derives every invariant independently.

The question it answers is the one the charter asks: *did every count, verdict
line, hash and measurement in these transcripts survive byte-identical?* The
answer is a **sha256 shown equal on both sides**, per file and in aggregate,
over the text with path tokens masked — plus, for the 63 files that carry
one, the `GATE:` verdict block hashed on its own.

    verify_scrub.py <pre-rev>        default: the commit before HEAD's scrub
"""

import hashlib
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from scrub import RULES, PATHTOK, canon, INTENTIONAL      # noqa: E402


def blob(root, rev, path):
    r = subprocess.run(["git", "-C", root, "show", "%s:%s" % (rev, path)],
                       capture_output=True)
    return r.stdout if r.returncode == 0 else None


def sha(b):
    return hashlib.sha256(b).hexdigest()


def main(argv):
    root = os.path.realpath(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                         "..", ".."))
    rev = argv[1] if len(argv) > 1 else "HEAD"
    changed = subprocess.run(
        ["git", "-C", root, "diff", "--name-only", "-z", rev, "--", "work"],
        capture_output=True, check=True).stdout
    files = [p.decode() for p in changed.split(b"\0") if p]
    print("pre-scrub revision : %s" % subprocess.run(
        ["git", "-C", root, "rev-parse", "--short", rev],
        capture_output=True, text=True).stdout.strip())
    print("files differing under work/ : %d" % len(files))
    if not files:
        sys.stderr.write("ERROR: 0 files to verify. A verification over "
                         "nothing is not a verification (#3470, #1002).\n")
        return 2

    agg_pre, agg_post = hashlib.sha256(), hashlib.sha256()
    gate_rows, fails, still_abs = [], [], []
    n_lines_ok = 0
    for f in files:
        pre = blob(root, rev, f)
        if pre is None:
            fails.append(("no blob at %s" % rev, f))
            continue
        post = open(os.path.join(root, f), "rb").read()

        if canon(pre) != canon(post):
            fails.append(("canon differs", f))
        if pre.count(b"\n") != post.count(b"\n"):
            fails.append(("line count moved", f))
        else:
            n_lines_ok += 1
        for s in INTENTIONAL:
            if pre.count(s) != post.count(s):
                fails.append(("intentional string %s moved" % s.decode(), f))
        if b"\x00" in post:
            fails.append(("NUL in post", f))
        if re.search(rb"/home/free|\\home\\free", post):
            still_abs.append(f)

        gp = b"\n".join(l for l in pre.split(b"\n") if l.startswith(b"GATE:"))
        gq = b"\n".join(l for l in post.split(b"\n") if l.startswith(b"GATE:"))
        if gp:
            gate_rows.append((f, sha(gp), sha(gq)))
            if gp != gq:
                fails.append(("GATE block changed", f))

        agg_pre.update(canon(pre))
        agg_post.update(canon(post))

    print("line count preserved      : %d / %d" % (n_lines_ok, len(files)))
    print("still carrying /home/free : %d %s" % (len(still_abs), still_abs or ""))
    print()
    print("THE GATE: VERDICT BLOCKS — sha256 of the GATE: lines, both sides")
    print("%-58s %-16s %-16s %s" % ("file", "pre", "post", "equal"))
    for f, a, b in sorted(gate_rows):
        print("%-58s %-16s %-16s %s" % (f[:58], a[:16], b[:16],
                                        "yes" if a == b else "*** NO ***"))
    print("gate blocks compared: %d, identical: %d"
          % (len(gate_rows), sum(1 for _, a, b in gate_rows if a == b)))
    print()
    print("AGGREGATE, path tokens masked, over all %d files:" % len(files))
    print("  pre  sha256 %s" % agg_pre.hexdigest())
    print("  post sha256 %s" % agg_post.hexdigest())
    print("  EQUAL: %s" % ("yes" if agg_pre.hexdigest() == agg_post.hexdigest()
                           else "NO"))
    if fails:
        for k, f in fails:
            sys.stderr.write("FAIL [%s] %s\n" % (k, f))
        return 1
    print("VERIFY PASS: %d files, 0 invariant failures" % len(files))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
