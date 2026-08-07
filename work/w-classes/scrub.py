#!/usr/bin/env python3
"""Scrub absolute machine paths out of this lane's committed logs.

Two standing rules meet here:

  * **never commit an absolute machine path**, and **do not hard-code one in
    the scrubber** — three artifacts needed hand-fixing when a scrubber knew
    only the author's own `$HOME`. So the pattern is DERIVED:
    `/home/<user>/…` for any plausible user name, plus the repo root read from
    this file's own location, plus the run-scratch roots the tools print.

  * **never rewrite a file another process still holds open** (#1135): a scrub
    that raced a backgrounded `gate.sh` punched a NUL hole into a PASSING
    gate's log, `grep` returned nothing, and a waiter reported TIMEOUT — on a
    FAILING gate the same corruption makes `grep -q FAIL` read clean. So this
    asserts NUL-free **before and after**, writes through a temp file and
    renames, and refuses any file that is not already clean.

    work/w-classes/scrub.py <file> [<file>...]
"""

import os
import re
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

SUBS = [
    # the repo root first — it is a prefix of everything below it
    (re.compile(re.escape(REPO)), "<repo>"),
    (re.compile(re.escape(REPO.replace("/", "\\\\"))), "<repo>"),
    # …then any home directory, derived rather than named
    (re.compile(r"/home/[a-z_][a-z0-9_-]*"), "<home>"),
    (re.compile(r"\\home\\[a-z_][a-z0-9_-]*"), "<home>"),
    # run-scratch roots the harness prints
    (re.compile(r"/tmp/[A-Za-z0-9._-]+"), "<tmp>"),
]


def main():
    bad = 0
    for p in sys.argv[1:]:
        with open(p, "rb") as fh:
            raw = fh.read()
        if b"\0" in raw:
            print("REFUSING %s: it already contains NUL bytes — repair by a "
                  "clean re-run, never by a rewrite" % p)
            bad = 1
            continue
        text = raw.decode("utf-8", "replace")
        for rx, rep in SUBS:
            text = rx.sub(rep, text)
        out = text.encode()
        if b"\0" in out:
            print("REFUSING %s: the scrub introduced NUL bytes" % p)
            bad = 1
            continue
        tmp = p + ".scrub.tmp"
        with open(tmp, "wb") as fh:
            fh.write(out)
        os.replace(tmp, p)
        left = len(re.findall(r"/home/[a-z_]", text)) + len(re.findall(r"\\home\\[a-z_]", text))
        print("%-46s %7d B -> %7d B   residual absolute paths: %d"
              % (p, len(raw), len(out), left))
        bad |= 1 if left else 0
    return bad


sys.exit(main())
