#!/usr/bin/env python3
"""check_table.py -- grade the conformance table MECHANICALLY.

PREREG SS6 (w-inlmetric). FIVE checks, each of which has caught a real defect
in this repo:

 1. ADDRESS  every `addr` must lie inside the function `owner` names, per
    FUNCS.tsv's entry+size. `P_INLINE.md` SS2.1's CORRECTION block is exactly
    this check done by hand, once, after four addresses had been published in
    the wrong function. Here it runs on every row, every time.

 2. ALIGN    every `addr` must START an instruction, per the INDEPENDENT
    objdump listing. Check 1 CANNOT FAIL on a mid-instruction address -- an
    address 0x11b bytes past the instruction the clause describes is still
    inside the same function, so containment is green and the citation is
    wrong. `w-inlfit` (board #3721) found EIGHT of the 24 in that state.

 3. DECODE   the instruction at `addr` must be the one the `asm` column
    records. ALIGN is NECESSARY AND NOT SUFFICIENT: `w-clausefix` found TWO
    rows (C10, C15) that were aligned, inside the right function, and pointed
    at a different instruction entirely. Neither check 1 nor check 2 can see
    that class; this one is the only thing that can.

 4. WITNESS  a row whose state is `R-derived` or `fitted` must cite
    `path:token` and that token must be PRESENT at that path.

 5. ABSENCE  a row whose state is `absent` or `unexercisable` must cite
    `none:<token>` and that token must be ABSENT from `crates/`. An `absent`
    verdict that is merely unchecked is the failure mode this exists for.

ADDRESS / WITNESS / ABSENCE need only the repo. ALIGN and DECODE need the
objdump listing, which is REGENERATED AND NEVER COMMITTED -- so an absent
listing is a **SKIP**, never a failure, and the SKIP is printed loudly with the
path it looked at and the number of rows it therefore did not grade (#3470: a
clean report over zero rows is not clean).

The boundary set comes from `objdump -d -M intel`, PE32 read as pei-i386 at
true VAs (`docs/whitebox/C2_MAP_METHOD.md`) -- deliberately NOT the Ghidra
database the addresses were transcribed out of. Two disassemblers agreeing that
an address is mid-instruction is a stronger claim than one of them saying so.

Exit 0 = GREEN (or GREEN-with-SKIP). Non-zero = RED. Read the verdict line,
never the exit code.

Usage: check_table.py [CLAUSES.tsv] [--plant ID=ADDR ...]

  --plant  overwrite row ID's `addr` with ADDR before grading, so the RED path
           can be WATCHED rather than assumed. #3336: a control nobody has seen
           fail is decoration. Repeatable. `--plant C2=10b62704` shifts one byte
           (reddens ALIGN); `--plant C2=10b62708` moves to a different real
           boundary (ALIGN stays green, DECODE reddens).

Provenance: checks 1/4/5 are `w-inlmetric`'s. Check 2 is `w-inlfit`'s
`work/w-inlfit/addr_align.py`, FOLDED IN HERE by `w-clausefix` on 2026-08-28
under `work/w-clausefix/PREREG.md` SS4 -- `w-inlfit` kept it separate because
this file was another lane's frozen instrument, a governance reason that its
prereg dissolved. That path survives as a shim which delegates here. Check 3 is
`w-clausefix`'s.
"""
import bisect, csv, os, re, subprocess, sys
from collections import Counter

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# The uncommitted, regenerable objdump listing. Overridable so a caller (or a
# control) can point the check somewhere else.
LISTING = os.environ.get(
    'C2RS_OBJDUMP_ASM',
    os.path.expanduser('~/ghidra-projects/export/c2/objdump_intel.asm'))

# `10b62703:\t a3 cc f5 c3 10 \tmov    ds:0x10c3f5cc,eax`
ASM_LINE = re.compile(r'^([0-9a-f]{8}):\t([^\t]*)\t(.*)$')


def norm(text):
    """Collapse runs of blanks -- objdump pads mnemonics to a column."""
    return re.sub(r'\s+', ' ', text.strip())


def funcs():
    p = os.path.join(REPO, 'docs/whitebox/ref/FUNCS.tsv')
    out = []
    for x in csv.DictReader([l for l in open(p) if not l.startswith('#')], delimiter='\t'):
        try:
            out.append((int(x['addr'], 16), int(x['size'])))
        except (ValueError, TypeError):
            pass
    out.sort()
    return out


def owner_of(fns, a):
    starts = [f[0] for f in fns]
    i = bisect.bisect_right(starts, a) - 1
    if i < 0:
        return None
    s, n = fns[i]
    return s if a < s + n else None


def listing():
    """(starts, {addr: disasm-text}) from the objdump listing, or None if absent."""
    if not os.path.exists(LISTING):
        return None
    starts, text = [], {}
    with open(LISTING, errors='replace') as fh:
        for line in fh:
            m = ASM_LINE.match(line)
            if m:
                a = int(m.group(1), 16)
                starts.append(a)
                text[a] = norm(m.group(3))
    starts.sort()
    return starts, text


def containing(starts, a):
    """The instruction start at or below `a`, or None if `a` precedes them all."""
    i = bisect.bisect_right(starts, a) - 1
    return starts[i] if i >= 0 else None


def token_in_file(path, tok):
    p = os.path.join(REPO, path)
    if not os.path.exists(p):
        return False
    return tok in open(p, encoding='utf-8', errors='replace').read()


def token_in_crates(tok):
    r = subprocess.run(['git', '-C', REPO, 'grep', '-l', '-F', '--', tok, '--', 'crates/'],
                       capture_output=True, text=True)
    return bool(r.stdout.strip())


def main(argv):
    plants, args, i = {}, [], 0
    while i < len(argv):
        if argv[i] == '--plant':
            rid, _, addr = argv[i + 1].partition('=')
            plants[rid] = addr
            i += 2
            continue
        args.append(argv[i])
        i += 1

    path = args[0] if args else os.path.join(REPO, 'work/w-inlmetric/CLAUSES.tsv')
    fns = funcs()
    rows = list(csv.DictReader([l for l in open(path) if not l.startswith('#')], delimiter='\t'))
    for r in rows:
        if r['id'] in plants:
            r['addr'] = plants[r['id']]
            r['id'] += '(PLANTED)'

    lst = listing()
    # Displayed home-relative: this output is COMMITTED as lane evidence, and an
    # absolute machine path in a tracked file is a class-3 violation of
    # scripts/tracked_artifact_audit.sh.
    shown = LISTING.replace(os.path.expanduser('~'), '~', 1)

    fails = []
    for r in rows:
        rid = r['id']
        # 1. ADDRESS
        try:
            a = int(r['addr'], 16)
            claimed = int(r['owner'], 16)
        except ValueError:
            fails.append(f"{rid}: addr/owner not hex")
            continue
        real = owner_of(fns, a)
        if real is None:
            fails.append(f"{rid}: ADDRESS 0x{a:08x} is inside NO FUNCS.tsv function (orphan)")
        elif real != claimed:
            fails.append(f"{rid}: ADDRESS 0x{a:08x} is in FUN_{real:08x}, "
                         f"table claims FUN_{claimed:08x}")

        # 2/3. ALIGN + DECODE -- only when the listing is on disk.
        if lst is not None:
            starts, text = lst
            b = containing(starts, a)
            if b is not None and b != a:
                fails.append(f"{rid}: ALIGN 0x{a:08x} is +{a - b} INTO the instruction at "
                             f"0x{b:08x} -- {r['clause'][:52]}")
            elif r.get('asm'):
                got = text.get(a, '(no such instruction)')
                if norm(r['asm']) != got:
                    fails.append(f"{rid}: DECODE 0x{a:08x} is {got!r}, "
                                 f"table records {norm(r['asm'])!r}")

        # 4/5. WITNESS / ABSENCE
        st, w = r['state'], r['witness']
        if st in ('R-derived', 'fitted'):
            if w.startswith('none:') or ':' not in w:
                fails.append(f"{rid}: state {st} must cite path:token, got {w!r}")
            else:
                p, tok = w.rsplit(':', 1)
                if not token_in_file(p, tok):
                    fails.append(f"{rid}: WITNESS {tok!r} NOT FOUND in {p}")
        elif st in ('absent', 'unexercisable'):
            if not w.startswith('none:'):
                fails.append(f"{rid}: state {st} must cite none:<token>, got {w!r}")
            else:
                tok = w[len('none:'):]
                if token_in_crates(tok):
                    fails.append(
                        f"{rid}: ABSENCE state {st} but token {tok!r} IS PRESENT in crates/. "
                        f"If you are a lane that just added it, this is NOT a defect in your "
                        f"code -- the table's `absent` verdict has gone stale and the remedy "
                        f"is a one-cell `state` edit by CLAUSES.tsv's owner.")
        else:
            fails.append(f"{rid}: unknown state {st!r}")

    c = Counter(r['state'] for r in rows)
    e = Counter(r['exercised'] for r in rows)
    n = len(rows)
    print(f"table    : {os.path.relpath(path, REPO)}")
    print(f"rows     : {n}")
    print("  state    :", dict(c))
    print("  exercised:", dict(e))
    print(f"listing  : {shown}")
    if lst is None:
        print(f"  ALIGN  : SKIP -- listing absent, so 0 of {n} rows were checked for "
              f"instruction alignment")
        print(f"  DECODE : SKIP -- listing absent, so 0 of {n} rows were checked against "
              f"their `asm` cell")
        print("           regenerate per docs/whitebox/C2_MAP_METHOD.md, or set "
              "C2RS_OBJDUMP_ASM")
    else:
        withasm = sum(1 for r in rows if r.get('asm'))
        print(f"  ALIGN  : {len(lst[0]):,} instruction starts, {n} of {n} rows graded")
        print(f"  DECODE : {withasm} of {n} rows carry an `asm` cell and were graded")
    if plants:
        print(f"planted  : {plants}")
    for f in fails:
        print("  FAIL " + f)
    skip = " (ALIGN+DECODE SKIPPED)" if lst is None else ""
    print(f"\nCONFORMANCE-CHECK: {'RED' if fails else 'GREEN'}{skip}  "
          f"({len(fails)} failure(s) over {n} rows)")
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
