#!/usr/bin/env python3
"""sets.py — write the predicted sets themselves, in plain text, for committing.

    usage: sets.py <predictions.jsonl> <model> [<model> ...]

One block per TU:  `== <src>  <model>  n=<count>  sha256=<digest>` then the
sorted names, one per line.  The digest is over exactly the lines that follow,
LF-terminated, so the file is self-checking.
"""
import hashlib
import json
import sys


def main():
    rows = sorted([json.loads(l) for l in open(sys.argv[1]) if l.strip()],
                  key=lambda r: r["src"])
    models = sys.argv[2:]
    for m in models:
        for r in rows:
            names = sorted(r["P"][m])
            body = "\n".join(names) + "\n"
            sys.stdout.write("== %s  %s  n=%d  sha256=%s\n"
                             % (r["src"], m, len(names),
                                hashlib.sha256(body.encode()).hexdigest()))
            sys.stdout.write(body)


if __name__ == "__main__":
    main()
