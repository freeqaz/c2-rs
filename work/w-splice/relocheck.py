#!/usr/bin/env python3
"""relocheck.py — the per-symbol relocation verdict for every spliced function.

Lane w-splice measurement tooling. **Read-only with respect to `crates/`.**

    relocheck.py <scan.jsonl>

FUNCTION BYTE MATCH compares a `.text` COMDAT's raw data, and a `.text`
section's raw data does not contain its relocations. So two bodies that are both
the word `48000000` against two DIFFERENT targets compare `exact` — board
**#882**, `fnbyte-exact-relocated` = 4,664 credited functions, and `w-seq`'s
`s12` is the compiled reproducer.

SPLICE-0-PORT replaces a caller's relocations with its **callee's**, whose
targets were resolved in the callee's context. That makes the one thing FBM
cannot see exactly the thing this mechanism changes, for every function it
moves. `crates/c2-harness/src/gap/fnbytes.rs::reloc_verdict` therefore compares,
per spliced symbol:

    port side  the spliced ComdatBody's `calls` + `data_refs`
               — the relocation sites PortC2::build would register
    ref side   the reference obj's own relocation records for the SAME COMDAT,
               by target NAME and in-section OFFSET, PAIR records excluded

and publishes `fnbyte-spliced-reloc|<verdict>`. This reads them with their
denominator. `no-relocs` is printed apart from `ok` on purpose: "both sides are
empty" is a much weaker statement than "both sides name the same targets at the
same offsets", and folding them would let an empty answer read as a pass.
"""

import collections
import json
import sys


def main():
    v = collections.Counter()
    fired = 0
    unreadable = 0
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, n in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-spliced-reloc|"):
                v[k.split("|", 1)[1]] += n
            elif k == "fnbyte-spliced":
                fired += n
            elif k == "fnbyte-reloc-records-unreadable":
                unreadable += n

    den = sum(v.values())
    print("=== SPLICED FUNCTIONS, RELOCATION SET vs THE REFERENCE OBJ ===")
    print("  spliced functions            : %d" % fired)
    print("  with a relocation verdict    : %d" % den)
    print("  objs whose reloc table failed: %d" % unreadable)
    if den != fired:
        print("  !!!! %d spliced functions produced NO verdict" % (fired - den))
    for k, n in v.most_common():
        print("  %6d  %5.1f%%  %s" % (n, 100.0 * n / den if den else 0.0, k))
    if not v:
        print("  (none)")

    bad = sum(n for k, n in v.items() if not (k.startswith("ok|") or k == "no-relocs"))
    print("\n  DISAGREEMENTS: %d" % bad)
    print("  (PREREG §3 item 4: any disagreement is a decline-floor failure)")


if __name__ == "__main__":
    main()
