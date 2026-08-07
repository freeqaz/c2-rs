#!/usr/bin/env python3
"""diffcheck.py — is every added `crates/` line a doc comment or inside `mod tests`?

"By construction" is the reasoning that let board #232 run 255 commits, so this
walks the UNIFIED DIFF's new-file line numbers against the `#[cfg(test)]` marker
rather than trusting the eye.  Prints the added lines that are NEITHER, and
fails if there are any.

    diffcheck.py <base-rev>
"""

import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def main():
    base = sys.argv[1] if len(sys.argv) > 1 else "master"
    d = subprocess.run(["git", "diff", "-U0", base, "--", "crates/"],
                       capture_output=True, text=True, cwd=ROOT).stdout
    files, cur, newno = {}, None, 0
    bad = []
    for line in d.splitlines():
        if line.startswith("+++ b/"):
            cur = line[6:]
            files.setdefault(cur, [])
            continue
        m = re.match(r"^@@ -\S+ \+(\d+)(?:,(\d+))? @@", line)
        if m:
            newno = int(m.group(1))
            continue
        if line.startswith("+") and not line.startswith("+++"):
            files[cur].append((newno, line[1:]))
            newno += 1
        elif not line.startswith("-"):
            newno += 1

    total = 0
    for f, adds in sorted(files.items()):
        p = os.path.join(ROOT, f)
        marker = None
        if os.path.exists(p):
            for i, l in enumerate(open(p), 1):
                if l.startswith("#[cfg(test)]"):
                    marker = i
                    break
        print("  %-46s +%d lines, #[cfg(test)] at %s"
              % (f, len(adds), marker))
        total += len(adds)
        for no, text in adds:
            s = text.strip()
            is_doc = s.startswith("//!") or s.startswith("///") or s == ""
            in_tests = marker is not None and no >= marker
            if not (is_doc or in_tests):
                bad.append((f, no, text))

    print("\n  added `crates/` lines: %d" % total)
    print("  NEITHER a doc comment NOR inside `mod tests`: %d" % len(bad))
    for f, no, t in bad:
        print("      %s:%d  %s" % (f, no, t))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
