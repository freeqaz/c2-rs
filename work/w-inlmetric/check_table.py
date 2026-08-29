#!/usr/bin/env python3
"""check_table.py -- grade the conformance table MECHANICALLY.

PREREG SS6 (w-inlmetric). SIX checks, each of which has caught a real defect
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

 6. CITES    the set of files under `crates/` citing `0x<addr>` must equal the
    row's frozen `cites` cell. ADDED 2026-08-29 by `w-clausegen` under
    `work/w-clausegen/PREREG.md` SS2, board `#3817`. **This closes check 5's
    FALSE-NEGATIVE half**, and see the long note on `cites_in_crates` for why
    an address is the handle and a name is not.

ADDRESS / WITNESS / ABSENCE / CITES need only the repo. ALIGN and DECODE need the
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

Usage: check_table.py [CLAUSES.tsv] [--plant ID=ADDR ...] [--set ID.COL=VAL ...]
                      [--rev REV]

  --plant  overwrite row ID's `addr` with ADDR before grading, so the RED path
           can be WATCHED rather than assumed. #3336: a control nobody has seen
           fail is decoration. Repeatable. `--plant C2=10b62704` shifts one byte
           (reddens ALIGN); `--plant C2=10b62708` moves to a different real
           boundary (ALIGN stays green, DECODE reddens).

  --set    the same idea for any cell: `--set C1.cites=crates/c2-core/src/x.rs`
           reddens CITES on C1 without touching the tracked table. Repeatable.

  --rev    run checks 4/5/6 against `crates/` AS OF a git revision instead of
           the working tree. This exists for ONE reason and it is the strongest
           evidence this file carries: at `72caf2586`, C14 and C18 read `absent`
           with tokens that were genuinely absent -- check 5 GREEN -- while
           `splice.rs` cited each row's OWN address twice. Point this at that
           commit's table and watch check 6 catch a false negative that really
           happened, rather than one that was planted.

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


def grep_l(pat, rev, word=False):
    """`git grep -l` for a FIXED string under `crates/`, at `rev` or the worktree.

    `--untracked --exclude-standard` is meaningless (and rejected) with a rev,
    so the worktree path keeps them and the rev path drops them.
    """
    cmd = ['git', '-C', REPO, 'grep', '-l']
    cmd += ['--untracked', '--exclude-standard'] if rev is None else []
    cmd += ['-F'] + (['-w'] if word else [])
    cmd += ['--', pat] + ([rev] if rev else []) + ['--', 'crates/']
    r = subprocess.run(cmd, capture_output=True, text=True)
    # With a rev, git prefixes each path `REV:`; strip it so the two modes
    # produce comparable path sets.
    out = []
    for p in r.stdout.strip().split('\n'):
        if not p:
            continue
        out.append(p[len(rev) + 1:] if rev and p.startswith(rev + ':') else p)
    return out


def token_in_file(path, tok, rev=None):
    if rev:
        r = subprocess.run(['git', '-C', REPO, 'show', f'{rev}:{path}'],
                           capture_output=True, text=True)
        return r.returncode == 0 and tok in r.stdout
    p = os.path.join(REPO, path)
    if not os.path.exists(p):
        return False
    return tok in open(p, encoding='utf-8', errors='replace').read()


def cites_in_crates(addr, rev=None):
    """Which files under `crates/` cite `0x<addr>`? Sorted, repo-relative.

    ---- CHECK 6, and WHY AN ADDRESS ------------------------------------------

    Check 5 asks *"is this one spelling absent from `crates/`?"*. A counterpart
    adopted under a DIFFERENT NAME answers yes, and the row stays `absent`
    forever with nothing counting the miss. `#3641` and `token_in_crates`'s
    docstring both describe the OTHER direction -- a mention read as a
    counterpart, a false positive, which is noisy and therefore gets found.
    This is the false NEGATIVE, which is silent. C14 and C18 sat in it for a
    full wave; C3 and C19 converted in wave 18 only because the adopting lane
    happened to choose colliding tokens.

    **A name is a lane's free choice. An address is not.** `CLAUDE.md`
    SS"Whitebox" requires a `DISCLOSURE` row naming the address in the same
    commit that adopts a disassembly-derived constant into `crates/`, and
    `PROV[R]` citation is what every adopting lane in this subsystem has in
    fact done. So an adoption leaves an address fingerprint whatever it calls
    itself, and that is the handle this check grabs.

    **It is a FROZEN-SET DIFFER, not a judge.** It does not decide whether a
    citation is a counterpart or a mention -- it decides whether the citation
    footprint has CHANGED since a human last read it. Any difference from the
    row's `cites` cell is RED, and the remedy is a reviewed one-cell edit by
    the table's owner.

    **Three blindnesses, declared here rather than discovered (`#3684`):**

    1. It inherits `#3641` transposed: a mention cites an address too.
       `clause_table.rs`'s own doc comment cites two of these addresses and
       `subsys.rs` cites a third as a band boundary. Those are mentions and
       this check reports them, by design -- a frozen mention is silent, a NEW
       one is not.
    2. It is blind to an adoption that cites no address at all. C24 is the
       standing example: its counterpart is real and its address appears
       nowhere in `crates/`. Sensitivity was MEASURED, not asserted, at
       **6 of 9** rows that already have a counterpart
       (`work/w-clausegen/RESULT.md`); it is not 9 of 9 and must never be
       quoted as if it were.
    3. Self-reference: editing `clause_table.rs`'s doc comment moves the set
       for the rows it discusses. That is the instrument working on the file
       most likely to talk about these clauses, not a defect to suppress.
    """
    return sorted(grep_l('0x' + addr, rev))


def token_in_crates(tok, rev=None):
    """Where under `crates/` does `tok` appear as an IDENTIFIER? A list of paths.

    `--untracked --exclude-standard` is LOAD-BEARING and was added by
    `w-clausefix` after a bare `git grep` reported GREEN over a file that
    existed on disk and had not been `git add`ed yet. The verdict of an
    ABSENCE check must depend on what is IN crates/, not on what has been
    staged: otherwise a lane's controls run green, the lane commits, and the
    check changes its mind with no edit in between. `--exclude-standard` keeps
    `target/` and other ignored output out of it.

    ---- `-w` ADDED 2026-08-29, coordinator, board **#3788** ----------------

    This screened with a bare `-F` — a SUBSTRING match — until now, and on its
    first real firing that produced a false positive. `w-inlbudget` landed
    `forceinline_charged`, and C19's token `inline_charge` is a substring of
    it, so the row went red for a coincidence of spelling. (C19's `absent`
    verdict was stale anyway, for an unrelated and real reason, so the check
    was right by accident — which is the least useful way for a check to be
    right.)

    Measured before changing, over all 18 `none:` tokens in the table:
    **0 rows change verdict** under `-w` on this tree, so this is a strict
    improvement and not a silent weakening. And it is confirmed against the
    actual defect: `inline_charge` is substring-present and word-ABSENT, while
    `caller_instrs` is word-PRESENT — separating the two failure classes of
    2026-08-28 exactly.

    STILL KNOWN AND STILL NOT FIXED HERE, because it is not a bug: this is a
    NAME screen over the whole subtree, so it cannot tell a counterpart in the
    port from a MENTION in a comment, a test, a GENERATED artifact, or — the
    2026-08-28 case — a function PARAMETER name (`#3641`). `caller_instrs`
    matches `splice.rs` and `surface/DOMAIN.txt`, and neither is a counterpart.
    Narrowing to `crates/*/src/` would redefine what `absent` means, which is
    `w-inlmetric` PREREG SS5's to define, not this function's. What IS done
    here is to return the paths, so the reader can classify a hit in seconds
    instead of reading the module — the verdict is unchanged, the diagnosis is
    not.

    ---- THE OTHER HALF IS NOW CHECK 6, AND IT IS NOT THIS FUNCTION -----------

    Everything above is about a **false positive**: a mention read as a
    counterpart. The block above says that is KNOWN AND NOT FIXED, and it still
    is. What was not written down anywhere until 2026-08-29 is the **false
    negative** — a counterpart adopted under a DIFFERENT NAME, which this
    function answers `absent` to and which nothing counts. That one is closed,
    by `cites_in_crates` (check 6), and it is closed with an ADDRESS rather
    than by narrowing this grep, because narrowing to `crates/*/src/` would
    redefine what `absent` MEANS and that is still `w-inlmetric` PREREG SS5's
    to define and not this function's. The two checks are independent and
    neither subsumes the other: this one catches an adoption that reuses the
    table's spelling, check 6 catches one that cites the table's address, and
    an adoption that does neither is invisible to both. See `w-clausegen`.
    """
    return grep_l(tok, rev, word=True)


def main(argv):
    plants, sets, rev, args, i = {}, {}, None, [], 0
    while i < len(argv):
        if argv[i] == '--plant':
            rid, _, addr = argv[i + 1].partition('=')
            plants[rid] = addr
            i += 2
            continue
        if argv[i] == '--set':
            key, _, val = argv[i + 1].partition('=')
            rid, _, col = key.partition('.')
            sets.setdefault(rid, {})[col] = val
            i += 2
            continue
        if argv[i] == '--rev':
            rev = argv[i + 1]
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
        if r['id'] in sets:
            for col, val in sets[r['id']].items():
                r[col] = val
            r['id'] += '(PLANTED)'

    lst = listing()
    # Displayed home-relative: this output is COMMITTED as lane evidence, and an
    # absolute machine path in a tracked file is a class-3 violation of
    # scripts/tracked_artifact_audit.sh.
    listing_shown = LISTING.replace(os.path.expanduser('~'), '~', 1)

    fails = []
    cited_rows = 0
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
                if not token_in_file(p, tok, rev):
                    fails.append(f"{rid}: WITNESS {tok!r} NOT FOUND in {p}")
        elif st in ('absent', 'unexercisable'):
            if not w.startswith('none:'):
                fails.append(f"{rid}: state {st} must cite none:<token>, got {w!r}")
            else:
                tok = w[len('none:'):]
                where = token_in_crates(tok, rev)
                if where:
                    # The paths are the whole point of printing this (#3788).
                    # A hit in a `src/` module is probably a counterpart; a hit
                    # only in a test, a generated file, or a doc comment is
                    # probably a MENTION, and this screen cannot tell them
                    # apart. Naming the files lets the reader do in seconds
                    # what cost a merge a full read of splice.rs.
                    # NOTE: this used to be called `shown`, which is also the
                    # name of the LISTING path printed after this loop -- so a
                    # single ABSENCE failure silently rewrote the `listing :`
                    # line to a list of source files. Found and fixed by
                    # `w-clausegen`; a report that misnames its own inputs is
                    # the `#3470` family, one level down.
                    hits = ', '.join(where[:4]) + (' …' if len(where) > 4 else '')
                    fails.append(
                        f"{rid}: ABSENCE state {st} but token {tok!r} IS PRESENT in crates/ "
                        f"as an identifier, in: {hits}. "
                        f"If you are a lane that just added it, this is NOT a defect in your "
                        f"code -- the table's `absent` verdict has gone stale and the remedy "
                        f"is a one-cell `state` edit by CLAUSES.tsv's owner. CHECK THE PATHS "
                        f"FIRST: a hit only in a test, a doc comment or a GENERATED file is a "
                        f"mention, not a counterpart, and this screen cannot tell the "
                        f"difference (#3641).")
        else:
            fails.append(f"{rid}: unknown state {st!r}")

        # 6. CITES -- the address fingerprint, frozen. See `cites_in_crates`.
        if 'cites' not in r:
            fails.append(f"{rid}: no `cites` column -- check 6 cannot run, and a "
                         f"check that cannot run is not a SKIP here: the column is "
                         f"committed (`w-clausegen`, #3817)")
            continue
        frozen = [] if r['cites'].strip() in ('-', '') else \
            sorted(x.strip() for x in r['cites'].split(',') if x.strip())
        measured = cites_in_crates(r['addr'], rev)
        if frozen:
            cited_rows += 1
        if frozen != measured:
            added = [p for p in measured if p not in frozen]
            gone = [p for p in frozen if p not in measured]
            why = []
            if added:
                why.append(
                    "A NEW citation of this clause's own address is how a counterpart "
                    "adopted under a DIFFERENT NAME becomes visible -- check 5 cannot "
                    "see one and stayed GREEN on C14/C18 for a full wave. READ THE "
                    "FILE, then either move `state` or record the citation as a "
                    "mention by updating the `cites` cell.")
            if gone:
                why.append(
                    "A citation that DISAPPEARED means the code that cited this clause "
                    "was deleted or renamed away from the address. If the row is not "
                    "`absent`, its witness may now be the only thing holding it up.")
            fails.append(
                f"{rid}: CITES footprint for 0x{r['addr']} MOVED -- "
                + (f"NEW: {', '.join(added)}. " if added else "")
                + (f"GONE: {', '.join(gone)}. " if gone else "")
                + f"State is {st!r}. " + ' '.join(why)
                + " Do not regenerate the cell without reading it (#3641: this check "
                  "cannot tell a mark from a mention either).")

    c = Counter(r['state'] for r in rows)
    e = Counter(r['exercised'] for r in rows)
    n = len(rows)
    print(f"table    : {os.path.relpath(path, REPO)}")
    print(f"rows     : {n}")
    print("  state    :", dict(c))
    print("  exercised:", dict(e))
    print(f"  CITES    : {cited_rows} of {n} rows have a non-empty frozen crates/ "
          f"citation footprint, {n} of {n} compared"
          + (f"  [at rev {rev}]" if rev else ""))
    print(f"listing  : {listing_shown}")
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
    if sets:
        print(f"set      : {sets}")
    for f in fails:
        print("  FAIL " + f)
    skip = " (ALIGN+DECODE SKIPPED)" if lst is None else ""
    print(f"\nCONFORMANCE-CHECK: {'RED' if fails else 'GREEN'}{skip}  "
          f"({len(fails)} failure(s) over {n} rows)")
    return 1 if fails else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
