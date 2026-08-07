#!/usr/bin/env python3
"""revcmp.py — do the listed workload sources have identical blobs at two revs?

The 850-TU model population is indexed at dc3 `940d07dc`; the tree's HEAD has
moved since.  A fresh capture compiles HEAD.  If a source's blob is identical at
both revs the two compiles are the same compile; if it is not, the fresh capture
is a different corpus and must be said so.

    usage: revcmp.py <dc3-repo> <revA> <revB> <tulist>
"""
import subprocess
import sys


def blob(dc3, rev, path):
    try:
        return subprocess.check_output(
            ["git", "-C", dc3, "rev-parse", "%s:%s" % (rev, path)],
            stderr=subprocess.DEVNULL).decode().strip()
    except subprocess.CalledProcessError:
        return None


def main():
    dc3, a, b, tulist = sys.argv[1:5]
    same = diff = missing = 0
    for ln in open(tulist):
        p = ln.strip()
        if not p:
            continue
        ba, bb = blob(dc3, a, p), blob(dc3, b, p)
        if ba is None or bb is None:
            missing += 1
            print("  MISSING %s (%s / %s)" % (p, ba, bb))
        elif ba == bb:
            same += 1
        else:
            diff += 1
            print("  DIFFER  %s  %s -> %s" % (p, ba[:12], bb[:12]))
    print("identical %d ; differ %d ; missing %d" % (same, diff, missing))


if __name__ == "__main__":
    main()
