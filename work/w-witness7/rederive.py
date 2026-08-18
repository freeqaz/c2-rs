#!/usr/bin/env python3
"""`w-witness7` — DERIVE the results table from the raw suite logs.

`docs/rungs/README.md` probe rule 2: *derive the results table from the logs;
never accumulate it.* Every number in the rung's tables comes out of here.

    rederive.py logs/*.suite.log
"""

import re
import sys

RES = re.compile(r"^test result: (\w+)\. (\d+) passed; (\d+) failed;")
FAIL = re.compile(r"^\s{4}(\S+)$")


def one(path):
    txt = open(path, encoding="utf8", errors="replace").read()
    lines = txt.split("\n")
    passed = failed = targets = 0
    for m in RES.finditer(txt):
        pass
    for l in lines:
        m = RES.match(l)
        if m:
            targets += 1
            passed += int(m.group(2))
            failed += int(m.group(3))
    # The named failing set, from the `failures:` block cargo prints per target.
    names = []
    infail = False
    for l in lines:
        if l.strip() == "failures:":
            infail = True
            continue
        if infail:
            m = FAIL.match(l)
            if m and not m.group(1).startswith("---"):
                names.append(m.group(1))
            elif l.strip() == "" or l.startswith("test result"):
                infail = False
    # `census_gate`'s duration — the differential that actually grades. An
    # ungraded run reads 0.00s and every colour taken in it is VOID.
    dur = None
    for l in lines:
        if "census_gate" in l and "finished in" in l:
            dur = l.strip()
    # Fall back: the per-target duration line that follows the census_gate
    # target header.
    if dur is None:
        cur = None
        for l in lines:
            if l.startswith("     Running tests/census_gate.rs"):
                cur = True
            elif cur and l.startswith("test result"):
                dur = l.strip()
                cur = None
    exit_code = None
    for l in lines:
        if l.startswith("EXIT="):
            exit_code = l.strip()
    return dict(path=path, passed=passed, failed=failed, targets=targets,
                names=sorted(set(names)), census_gate=dur, exit=exit_code)


def main(argv):
    for p in argv:
        r = one(p)
        print(f"{r['path']}")
        print(f"   {r['passed']} passed / {r['failed']} failed / "
              f"{r['targets']} targets   {r['exit']}")
        print(f"   census_gate: {r['census_gate']}")
        if r["names"]:
            for n in r["names"]:
                print(f"   FAIL  {n}")
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
