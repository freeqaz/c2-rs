#!/usr/bin/env python3
"""provenance_census.py — the derived-vs-fitted census over `crates/`.

Board **#3630**. Lane `w-provenance`, decision 15
(`docs/DECISIONS_2026-08-22.md`). Convention:
`docs/whitebox/DISCLOSURE.md` § "The in-code provenance markers".

WHAT IT COUNTS AND WHY THAT IS THE HARD PART
--------------------------------------------
The project's primary goal is understanding MSVC's internals. The ratio that
most directly tracks it is *how much of the port is READ from the binary
versus FITTED to observations* — and before this script that ratio was
invisible: at tree `6c753ead0` the whole of `crates/` carried **5** provenance
marks, all three of them prose sentences in module doc comments, none attached
to a constant, against a 247-line `DISCLOSURE.md`.

A numerator alone would be worthless (`#3470`, `#1002`: only a denominator
catches an absence). So this script counts a **population**, not a set of
hits, and prints the untagged remainder per module as a **named residue**.

  population  = every `const` / `static` item in non-test code under the
                scanned roots. This is the mechanical proxy for
                "load-bearing constant" defined in DISCLOSURE.md; a proxy
                member that is genuinely not load-bearing is expected to say
                so with `PROV[N]`, which is why exclusions are visible rather
                than silent.
  tagged      = a population member whose attached comment block carries a
                `PROV[X]` marker.
  untagged    = population minus tagged. THE POINT OF THE SCRIPT.
  rule marks  = `PROV[X]` markers that are NOT attached to a population
                member (they annotate a function, a match arm, a module).
                Counted, **never given a denominator** — the set of
                load-bearing *rules* is not mechanically enumerable and this
                script does not pretend otherwise.

WHAT IT MUST NOT BECOME
-----------------------
A ranking instrument. `#3505`'s family: four of four lanes dispatched off a
size ranking found the ranking was measuring itself. Modules print in **path
order**, never sorted by count, and the tracked signal is the **change** in a
module's row between two trees — never its distance from 100 % `[R]`. A high
`[R]` count licenses **no emit** (`docs/FUNCTION_BYTE_MATCH.md` §0).

NOT TO BE CONFUSED WITH `#207`
------------------------------
Board `#202`/`#207` ("census provenance", lane `w-prov`, `.prov` sidecars) is
a *different* instrument with a colliding name: it records **which dc3 corpus
a measurement was graded against**. This script records **where a constant in
the source came from**. Corpus provenance vs derivation provenance.

THE LEVEL IS NOT THE SIGNAL — `--since` (board **#3648**, lane `w-provext`)
--------------------------------------------------------------------------
The rule above says the tracked signal is the **change** in a module's row
between two trees. Until `--since` existed the tool could only print a
**level**, so the signal the doctrine names had to be reconstructed by hand
from two transcripts — which is how a level gets quoted as a result.

`--since <sha>` extracts the scanned roots at `<sha>` (`git archive` into a
temp dir; the working tree is never touched), runs the same census over it,
and reports the delta per module. Three properties it is required to have,
each of them a repo failure it is designed not to repeat:

* **Every rate change is printed beside its population change.** `#3045` is
  the canonical local failure: an accuracy moved 0.0035 while its denominator
  moved 10.8 %, and *the two numbers together are the finding* — either alone
  misleads. A coverage delta with no population delta beside it is a defect
  of the instrument, not a terse output.
* **"A constant was TAGGED" and "a constant APPEARED" are never summed.**
  They have opposite meanings for the goal: one is provenance work, the other
  is new unprovenanced surface. They are reported in separate columns, and
  the per-class delta is published with its **decomposition identity**
  (`Δ[X] = appeared − vanished + retagged-in − retagged-out`) checked at
  runtime, so a differ that quietly conflates them cannot print `OK`.
* **A base that predates the convention is a LABELLED OUTCOME, not a
  triumph.** Against an unmarked base every marker reads as "added" and the
  coverage delta equals the tip level, carrying no information the level did
  not already carry. The tool says so in those words and withholds the
  delta-as-progress reading.

USAGE
  scripts/provenance_census.py                     census over crates/
  scripts/provenance_census.py --by-file           per-file rows
  scripts/provenance_census.py --list-untagged M   name the residue of module M
  scripts/provenance_census.py --since <sha>       two-tree diff: what CHANGED
                                                   per module between <sha> and
                                                   the working tree
  scripts/provenance_census.py --since <sha> --by-file      … per file
  scripts/provenance_census.py --self-test         planted fixture, exact
                                                   counts, plus a marker
                                                   removal that MUST move the
                                                   count (a control never seen
                                                   red is decoration, #3336)

EXIT CODES
  0  census printed (or self-test passed)
  1  self-test failed
  2  could not run (bad root, no files found, unknown sha)
  3  a marker carries no citation — a defect, reported by name
"""

import io
import os
import re
import subprocess
import sys
import tarfile
import tempfile

MARKS = ("R", "O", "F", "S", "N")

# The marker token. Prefixed deliberately: a bare `[R]` grep cannot tell a
# marker from an array index — at tree `6c753ead0` the brief's sixth "marker"
# was `params[src]` in `c2-il/src/func/mod.rs:2013` — and three prose `[R]`s
# in `codegen/` module docs would otherwise be counted as tags they are not.
#
# ── THE MENTION RULE (board **#3669**, lane `w-provaudit`, from **#3641**) ──
#
# **A marker written inside backticks is a MENTION and is NOT counted.** The
# prefix above solves *"is this token a marker or an array index"*. It does not
# solve *"is this token a marker or a discussion OF a marker"*, and that is the
# defect `#3641` measured on the neighbouring instrument: writing prose about
# mark letters moved a subsystem's own agreement census **9/28 → 13/34**, from
# four prose sites, **one of which was the sentence explicitly warning against
# the hazard**. Nothing could see it, because a counter cannot tell an
# annotation from a citation of one.
#
# So the delimiter carries the distinction: `` `PROV[R]` `` in prose is a
# mention; a bare `PROV[R]` is a mark. Anything wanting to *discuss* a marker —
# this comment, `DISCLOSURE.md`'s legend, a rung — backticks it and is silent.
#
# **THE COST WAS MEASURED BEFORE THE RULE WAS ADOPTED, in both directions.** At
# tree `0dcfca959`, `crates/` carries **649** `PROV`/`PROV-BLOCK` tokens
# producing **777** tagged constants and **6** rule marks, and **0 of the 649
# are backticked** — so the rule is adoptable at exactly zero cost and every
# one of the 777 counts identically after it. Proved with `--since` rather than
# asserted (`work/w-provaudit/census_since_markre.txt`: →tag 0, →untag 0,
# reclass 0, +new 0, -gone 0, on every module), and the `--self-test` carries a
# planted backticked marker that MUST NOT be counted plus a bare one beside it
# that must.
#
# **AND IT IS NOT TRANSFERABLE TO THE OTHER COUNTED SURFACE — measured, and
# that is the finding.** `subsys.rs::count_marks` counts literal `[R]`/`[O]`/
# `[I]` on the ten `ref/P_*.md` pages, and those pages write **481 of their 488
# evidence marks in backticks**. Adopting this rule there would zero the
# census, not clean it; position does not separate them either (`P_ENCODE.md`,
# the page `#3641` was found on, has **0** marks in table rows and **28** in
# prose). See `scripts/prose_audit.py`'s C5 surface-2 table.
MARK_RE = re.compile(r"(?<!`)PROV\[([RSOFN])\](?!`)")

# The BLOCK form. One citation covering every population member lexically
# inside the block it is declared in, so a 91-entry transcribed opcode table
# does not need 91 identical lines. It is still greppable, still cited, and
# still counted PER CONSTANT — the count is what the census publishes, so the
# saving is in the source and not in the number. An item's own marker always
# wins over the block it sits in.
BLOCK_RE = re.compile(r"(?<!`)PROV-BLOCK\[([RSOFN])\](?!`)")

# A `const`/`static` item declaration. Anchored at line start with optional
# indentation so a `const` in expression position is not matched; requires a
# SCREAMING_SNAKE name, which is Rust's own convention for these items and is
# what every one of the 410 items in `c2-core` uses.
ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([a-z:]+\))?\s+)?(?:const|static)\s+(?:mut\s+)?([A-Z][A-Z_0-9]*)\s*:"
)

COMMENT_RE = re.compile(r"^\s*(?://|#\[|#!\[)")

TEST_FILE_NAMES = ("tests.rs", "testutil.rs")


def strip_for_braces(line):
    """Crude but adequate: remove line comments, string and char literals so
    brace counting is not thrown by `"{"` or `'{'`. Block comments are handled
    by the caller's in_block flag."""
    out = []
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break
        if c == '"':
            i += 1
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if c == "'":
            # char literal or lifetime; only skip if it looks like a literal
            m = re.match(r"'(?:\\.|[^\\'])'", line[i:])
            if m:
                i += m.end()
                continue
        out.append(c)
        i += 1
    return "".join(out)


def scan_file(path):
    """Return (items, rule_marks, citation_defects).

    items:  list of (line_no, name, mark_or_None)
    rule_marks: list of (line_no, mark)
    citation_defects: list of (line_no, mark) for markers with no citation
    """
    with open(path, "r", encoding="utf-8", errors="replace") as fh:
        lines = fh.read().split("\n")

    is_test_file = os.path.basename(path) in TEST_FILE_NAMES

    # --- pass 1: mark the line spans that live inside a `#[cfg(test)]` block
    # or a `#[test]` fn, so their consts are out of the population.
    in_test = [False] * len(lines)
    depth_after = [0] * len(lines)
    pending_test_attr = False
    test_depth = None
    depth = 0
    in_block_comment = False
    for i, raw in enumerate(lines):
        line = raw
        if in_block_comment:
            if "*/" in line:
                line = line.split("*/", 1)[1]
                in_block_comment = False
            else:
                in_test[i] = test_depth is not None or is_test_file
                depth_after[i] = depth
                continue
        # A block comment can only OPEN in code — never inside a `//` line
        # comment and never inside a string literal. `strip_for_braces` removes
        # both, so this must run on its output.
        #
        # **Board #3649, and it is not a nicety.** Before this line ran on
        # `clean`, a glob like `coff/*.rs` written inside a `///` doc comment
        # opened a block comment that never closed, and from that line to the end
        # of the file the brace depth stopped tracking. The visible consequence
        # is that every `#[cfg(test)]` region BELOW such a glob failed to be
        # excluded, so test-only constants were counted as production population.
        # Eighteen files in `crates/` contain one.
        clean = strip_for_braces(line)
        if "/*" in clean and "*/" not in clean.split("/*", 1)[1]:
            clean = clean.split("/*", 1)[0]
            in_block_comment = True

        stripped = line.strip()
        if stripped.startswith("#[cfg(test)]") or stripped.startswith("#[test]"):
            pending_test_attr = True

        in_test[i] = test_depth is not None or is_test_file

        opens = clean.count("{")
        closes = clean.count("}")
        if pending_test_attr and opens > 0:
            if test_depth is None:
                test_depth = depth
                in_test[i] = True
            pending_test_attr = False
        depth += opens - closes
        depth_after[i] = depth
        if test_depth is not None and depth <= test_depth:
            test_depth = None

    # --- pass 1b: block markers. A `PROV-BLOCK[X]` declared on line b inside a
    # block whose depth is d covers every later line until the depth drops
    # below d — i.e. until that block closes.
    block_at = [None] * len(lines)
    for b, raw in enumerate(lines):
        mb = BLOCK_RE.search(raw)
        if not mb or in_test[b]:
            continue
        d = depth_after[b]
        for k in range(b + 1, len(lines)):
            if depth_after[k] < d:
                break
            block_at[k] = mb.group(1)

    # --- pass 2: items, markers
    items = []
    rule_marks = []
    defects = []

    marked_lines = {}
    for i, raw in enumerate(lines):
        m = MARK_RE.search(raw)
        if not m:
            continue
        if in_test[i]:
            # a marker inside a test region annotates nothing the judge
            # grades; it is neither a tag nor a rule mark.
            continue
        mark = m.group(1)
        marked_lines[i] = mark
        tail = raw[m.end():].strip()
        # a citation is anything substantive after the marker; an em-dash is
        # the convention but is not required, an empty tail is the defect.
        cite = tail.lstrip("—-: ").strip()
        if len(cite) < 3:
            defects.append((i + 1, mark))

    for b, raw in enumerate(lines):
        mb = BLOCK_RE.search(raw)
        if not mb or in_test[b]:
            continue
        cite = raw[mb.end():].strip().lstrip("—-: ").strip()
        if len(cite) < 3:
            defects.append((b + 1, mb.group(1)))

    consumed = set()
    for i, raw in enumerate(lines):
        m = ITEM_RE.match(raw)
        if not m:
            continue
        if in_test[i]:
            continue
        name = m.group(1)
        mark = None
        # same-line trailing marker
        if i in marked_lines:
            mark = marked_lines[i]
            consumed.add(i)
        else:
            # walk backwards over the contiguous attached comment/attribute
            # block; stop at the first line that is neither
            j = i - 1
            while j >= 0 and COMMENT_RE.match(lines[j]):
                if j in marked_lines:
                    mark = marked_lines[j]
                    consumed.add(j)
                    break
                j -= 1
            if mark is None:
                mark = block_at[i]
        items.append((i + 1, name, mark))

    for i, mark in marked_lines.items():
        if i not in consumed:
            rule_marks.append((i + 1, mark))

    return items, rule_marks, defects


def walk(roots):
    files = []
    for root in roots:
        if os.path.isfile(root) and root.endswith(".rs"):
            files.append(root)
            continue
        for dirpath, dirnames, filenames in os.walk(root):
            dirnames[:] = [d for d in sorted(dirnames) if d not in ("target", ".git")]
            for fn in sorted(filenames):
                if fn.endswith(".rs"):
                    files.append(os.path.join(dirpath, fn))
    return sorted(files)


def module_of(path):
    """Module = the file's parent directory. Coarse enough that a row is
    stable across file splits, fine enough that `codegen` and `coff` are
    separate rows — which is the granularity decision 15's per-subsystem
    scoreboard is written at."""
    return os.path.dirname(path)


def census(roots):
    rows = {}
    files = walk(roots)
    if not files:
        return None, None, None
    all_defects = []
    per_file = {}
    for path in files:
        items, rule_marks, defects = scan_file(path)
        mod = module_of(path)
        r = rows.setdefault(mod, {"pop": 0, "rules": 0, **{k: 0 for k in MARKS}, "untagged": 0})
        f = {"pop": 0, "rules": 0, **{k: 0 for k in MARKS}, "untagged": 0}
        for _, _, mark in items:
            r["pop"] += 1
            f["pop"] += 1
            if mark:
                r[mark] += 1
                f[mark] += 1
            else:
                r["untagged"] += 1
                f["untagged"] += 1
        r["rules"] += len(rule_marks)
        f["rules"] += len(rule_marks)
        per_file[path] = (f, items)
        for ln, mk in defects:
            all_defects.append((path, ln, mk))
    return rows, per_file, all_defects


def tree_stamp(repo_root):
    def run(*args):
        try:
            return subprocess.run(
                args, cwd=repo_root, capture_output=True, text=True, check=True
            ).stdout.strip()
        except Exception:
            return "?"

    sha = run("git", "rev-parse", "HEAD")
    dirty = run("git", "status", "--porcelain")
    return sha, ("DIRTY" if dirty else "clean")


def print_table(rows, label, repo_root):
    sha, state = tree_stamp(repo_root)
    print(f"provenance census — tree {sha} ({state})")
    print(f"scope: {label}")
    print(
        "rows are in PATH ORDER, never sorted by count (#3505: an instrument "
        "sorted by size measures itself)"
    )
    print()
    hdr = f"{'module':<44} {'pop':>5} {'[R]':>5} {'[O]':>5} {'[F]':>5} {'[S]':>5} {'[N]':>5} {'untag':>6} {'rule':>5}"
    print(hdr)
    print("-" * len(hdr))
    tot = {"pop": 0, "rules": 0, "untagged": 0, **{k: 0 for k in MARKS}}
    for mod in sorted(rows):
        r = rows[mod]
        if r["pop"] == 0 and r["rules"] == 0:
            continue
        print(
            f"{mod:<44} {r['pop']:>5} {r['R']:>5} {r['O']:>5} {r['F']:>5} "
            f"{r['S']:>5} {r['N']:>5} {r['untagged']:>6} {r['rules']:>5}"
        )
        for k in tot:
            tot[k] += r[k]
    print("-" * len(hdr))
    print(
        f"{'TOTAL':<44} {tot['pop']:>5} {tot['R']:>5} {tot['O']:>5} {tot['F']:>5} "
        f"{tot['S']:>5} {tot['N']:>5} {tot['untagged']:>6} {tot['rules']:>5}"
    )
    print()
    print(
        f"denominator: {tot['pop']} const/static items in non-test code. "
        f"tagged {tot['pop'] - tot['untagged']}, untagged {tot['untagged']}."
    )
    print(
        f"rule markers: {tot['rules']} — NO DENOMINATOR. The set of load-bearing "
        "rules is not mechanically enumerable; this number is a numerator and "
        "is never printed as a ratio."
    )
    return tot


# ---------------------------------------------------------------------------
# `--since <sha>` — the two-tree diff. Board #3648.
#
# Decision 15's doctrine is that the tracked signal is the CHANGE per module.
# Everything below exists to print that change WITHOUT the two ways this repo
# has previously turned a delta into a wrong statement:
#   #3045 — a rate delta with no population delta beside it;
#   and the conflation of "tagged" with "appeared", which have opposite
#   meanings for the goal and are therefore never added together here.
# ---------------------------------------------------------------------------

# The class label used for a population member carrying no marker. Not one of
# MARKS: the residue is a state, not a provenance claim, and giving it a letter
# would let it be read as one.
UNTAGGED = "-"
CLASSES = MARKS + (UNTAGGED,)


def item_index(per_file):
    """`per_file` -> {(path, name, nth): mark_or_UNTAGGED}.

    Keyed by NAME and its ordinal within the file, never by LINE — a comment-only
    tagging pass moves every line number in the file it touches, so a line-keyed
    identity would report the whole file as vanished-and-reappeared. The ordinal
    is load-bearing and not defensive: `c2-il/src/func/body/expr.rs` declares
    five distinct function-local `const ON` items, so (path, name) alone is not
    unique in this tree.
    """
    out = {}
    for path, (_f, items) in per_file.items():
        seen = {}
        for _ln, name, mark in items:
            n = seen.get(name, 0)
            seen[name] = n + 1
            out[(path, name, n)] = mark or UNTAGGED
    return out


def git_show_subject(repo_root, sha):
    try:
        return subprocess.run(
            ["git", "log", "-1", "--format=%h %s", sha],
            cwd=repo_root, capture_output=True, text=True, check=True,
        ).stdout.strip()
    except Exception:
        return sha


def extract_at(repo_root, sha, roots, dest):
    """Extract `roots` as they stood at `sha` into `dest`.

    Returns (present_roots, error_or_None). The working tree is never touched:
    this is `git archive` into a temp directory, so the mode is safe to run on a
    dirty tree and safe to run concurrently with a peer session.
    """
    present = []
    for r in roots:
        cp = subprocess.run(
            ["git", "ls-tree", "-d", "--name-only", sha, "--", r],
            cwd=repo_root, capture_output=True, text=True,
        )
        if cp.returncode != 0:
            return [], f"unknown revision or bad root: {sha} -- {r}"
        if cp.stdout.strip():
            present.append(r)
            continue
        # not a directory at that sha; it may still be a single file
        cp2 = subprocess.run(
            ["git", "ls-tree", "--name-only", sha, "--", r],
            cwd=repo_root, capture_output=True, text=True,
        )
        if cp2.returncode == 0 and cp2.stdout.strip():
            present.append(r)
    if not present:
        return [], None
    cp = subprocess.run(
        ["git", "archive", "--format=tar", sha, "--"] + present,
        cwd=repo_root, capture_output=True,
    )
    if cp.returncode != 0:
        return [], cp.stderr.decode("utf-8", "replace").strip() or "git archive failed"
    with tarfile.open(fileobj=io.BytesIO(cp.stdout)) as tf:
        try:
            tf.extractall(dest, filter="data")
        except TypeError:  # python < 3.12 has no `filter`
            tf.extractall(dest)
    return present, None


def census_at(repo_root, sha, roots, tmp):
    """(rows, per_file, defects, present_roots, error). Paths come back RELATIVE
    to the extracted root, so they are directly comparable with the tip's."""
    present, err = extract_at(repo_root, sha, roots, tmp)
    if err:
        return None, None, None, [], err
    if not present:
        return {}, {}, [], [], None
    cwd = os.getcwd()
    os.chdir(tmp)
    try:
        rows, per_file, defects = census(present)
    finally:
        os.chdir(cwd)
    if rows is None:
        return {}, {}, [], present, None
    return rows, per_file, defects, present, None


def blank_row():
    return {"pop": 0, "rules": 0, "untagged": 0, **{k: 0 for k in MARKS}}


def diff_rows(base_rows, tip_rows, base_items, tip_items, key_of):
    """Build the per-key delta record. `key_of(path)` selects the row a file
    belongs to (its module, or the file itself under --by-file).

    Every returned counter is either a POPULATION fact or a PROVENANCE fact and
    the two are never merged into one field. That separation is the deliverable.
    """
    keys = set(base_rows) | set(tip_rows)
    for (path, _n, _i) in list(base_items) + list(tip_items):
        keys.add(key_of(path))

    d = {}
    for k in keys:
        d[k] = {
            "base": base_rows.get(k, blank_row()),
            "tip": tip_rows.get(k, blank_row()),
            # POPULATION facts — a constant that appeared or vanished
            "appeared": {c: 0 for c in CLASSES},
            "vanished": {c: 0 for c in CLASSES},
            # PROVENANCE facts — a constant that was already there and whose
            # marker changed. Split three ways because they mean three things.
            "retag_in": {c: 0 for c in CLASSES},
            "retag_out": {c: 0 for c in CLASSES},
            "n_tagged": 0,      # untagged -> tagged   (the provenance work)
            "n_untagged": 0,    # tagged   -> untagged (a marker was LOST)
            "n_reclass": 0,     # tagged X -> tagged Y (a judgement changed)
        }

    for key, mark in tip_items.items():
        if key in base_items:
            continue
        r = d[key_of(key[0])]
        r["appeared"][mark] += 1
    for key, mark in base_items.items():
        if key in tip_items:
            continue
        r = d[key_of(key[0])]
        r["vanished"][mark] += 1
    for key, tmark in tip_items.items():
        bmark = base_items.get(key)
        if bmark is None or bmark == tmark:
            continue
        r = d[key_of(key[0])]
        r["retag_out"][bmark] += 1
        r["retag_in"][tmark] += 1
        if bmark == UNTAGGED:
            r["n_tagged"] += 1
        elif tmark == UNTAGGED:
            r["n_untagged"] += 1
        else:
            r["n_reclass"] += 1
    return d


def class_count(row, c):
    return row["untagged"] if c == UNTAGGED else row[c]


def check_decomposition(rec):
    """Δ[X] must equal appeared − vanished + retag_in − retag_out, for every
    class including the untagged residue. Returns a list of (class, got, want).

    This is not belt-and-braces. It is the only thing standing between this
    mode and the failure it was built to prevent: a differ that folds an
    appeared-already-tagged constant into the retag column still prints a
    plausible Δ, and the identity is what refuses it."""
    bad = []
    for c in CLASSES:
        got = class_count(rec["tip"], c) - class_count(rec["base"], c)
        want = (rec["appeared"][c] - rec["vanished"][c]
                + rec["retag_in"][c] - rec["retag_out"][c])
        if got != want:
            bad.append((c, got, want))
    return bad


def _rate(tagged, pop):
    return (100.0 * tagged / pop) if pop else 0.0


def print_diff(base_sha, base_label, roots, d, base_marked, tip_state, tip_sha):
    print("provenance census — TWO-TREE DIFF")
    print(f"  base  {base_label}")
    print(f"  tip   {tip_sha} ({tip_state})")
    print(f"  scope {', '.join(roots)}")
    print()
    print("rows are in PATH ORDER, never sorted by count or by delta (#3505).")
    print("The tracked signal is the CHANGE per module — never the distance")
    print("from 0 or from 100 % [R] (DISCLOSURE.md § What these numbers do NOT")
    print("license, item 3).")
    print()

    if not base_marked:
        print("=" * 78)
        print("BASE PREDATES THE MARKER CONVENTION — 0 markers found at the base tree.")
        print("Every marker at the tip therefore reads as 'added' against an unmarked")
        print("base, and the coverage delta below is EQUAL TO THE TIP LEVEL. It carries")
        print("no information the level does not already carry, and it is NOT progress")
        print("measured against a comparable base. Labelled outcome, not a result.")
        print("=" * 78)
        print()

    hdr = (f"{'module':<44} {'popB':>5} {'popT':>5} {'Δpop':>6} | "
           f"{'tagB':>5} {'tagT':>5} {'Δtag':>6} | {'covB':>6} {'covT':>6} {'Δpp':>7}")
    print(hdr)
    print("-" * len(hdr))
    tot = {"popB": 0, "popT": 0, "tagB": 0, "tagT": 0}
    for k in sorted(d):
        rec = d[k]
        b, t = rec["base"], rec["tip"]
        if b["pop"] == 0 and t["pop"] == 0:
            continue
        tb = b["pop"] - b["untagged"]
        tt = t["pop"] - t["untagged"]
        cb, ct = _rate(tb, b["pop"]), _rate(tt, t["pop"])
        print(f"{k:<44} {b['pop']:>5} {t['pop']:>5} {t['pop'] - b['pop']:>+6} | "
              f"{tb:>5} {tt:>5} {tt - tb:>+6} | {cb:>5.1f}% {ct:>5.1f}% {ct - cb:>+6.1f}")
        tot["popB"] += b["pop"]
        tot["popT"] += t["pop"]
        tot["tagB"] += tb
        tot["tagT"] += tt
    print("-" * len(hdr))
    cb = _rate(tot["tagB"], tot["popB"])
    ct = _rate(tot["tagT"], tot["popT"])
    print(f"{'TOTAL':<44} {tot['popB']:>5} {tot['popT']:>5} {tot['popT'] - tot['popB']:>+6} | "
          f"{tot['tagB']:>5} {tot['tagT']:>5} {tot['tagT'] - tot['tagB']:>+6} | "
          f"{cb:>5.1f}% {ct:>5.1f}% {ct - cb:>+6.1f}")
    print()
    print("Δpop is printed BESIDE Δcov and is never omitted: #3045 moved an")
    print("accuracy by 0.0035 while its denominator moved 10.8 %, and the two")
    print("numbers together were the finding. A rate delta alone is a defect.")
    print()

    print("WHY THE TAGGED COUNT MOVED — provenance work vs. new surface")
    print("These two halves are NEVER summed. 'a constant was tagged' and 'a")
    print("constant appeared' have opposite meanings for the goal: the first is")
    print("understanding recorded, the second is unprovenanced surface added.")
    print()
    h2 = (f"{'module':<44} {'→tag':>6} {'→untag':>7} {'reclass':>8} | "
          f"{'+new':>5} {'+new(t)':>8} {'-gone':>6} {'-gone(t)':>9}")
    print(h2)
    print("-" * len(h2))
    t2 = {"tg": 0, "ut": 0, "rc": 0, "ap": 0, "apt": 0, "va": 0, "vat": 0}
    for k in sorted(d):
        rec = d[k]
        ap = sum(rec["appeared"].values())
        apt = ap - rec["appeared"][UNTAGGED]
        va = sum(rec["vanished"].values())
        vat = va - rec["vanished"][UNTAGGED]
        if (rec["n_tagged"] or rec["n_untagged"] or rec["n_reclass"] or ap or va):
            print(f"{k:<44} {rec['n_tagged']:>6} {rec['n_untagged']:>7} "
                  f"{rec['n_reclass']:>8} | {ap:>5} {apt:>8} {va:>6} {vat:>9}")
        t2["tg"] += rec["n_tagged"]
        t2["ut"] += rec["n_untagged"]
        t2["rc"] += rec["n_reclass"]
        t2["ap"] += ap
        t2["apt"] += apt
        t2["va"] += va
        t2["vat"] += vat
    print("-" * len(h2))
    print(f"{'TOTAL':<44} {t2['tg']:>6} {t2['ut']:>7} {t2['rc']:>8} | "
          f"{t2['ap']:>5} {t2['apt']:>8} {t2['va']:>6} {t2['vat']:>9}")
    print()
    print("  →tag      an EXISTING constant gained a marker      (provenance work)")
    print("  →untag    an EXISTING constant LOST its marker      (a regression)")
    print("  reclass   an EXISTING marker changed letter         (a judgement moved)")
    print("  +new      constants that did not exist at the base  (new surface)")
    print("  +new(t)   … of which arrived already tagged")
    print("  -gone     constants deleted since the base;  -gone(t) … were tagged")
    print()

    print("PER-CLASS DELTA, with the decomposition identity checked")
    h3 = (f"{'class':<8} {'base':>6} {'tip':>6} {'Δ':>6}   "
          f"{'appeared':>9} {'vanished':>9} {'retag-in':>9} {'retag-out':>10}   check")
    print(h3)
    print("-" * len(h3))
    agg = {c: {"b": 0, "t": 0, "ap": 0, "va": 0, "ri": 0, "ro": 0} for c in CLASSES}
    for k in d:
        rec = d[k]
        for c in CLASSES:
            agg[c]["b"] += class_count(rec["base"], c)
            agg[c]["t"] += class_count(rec["tip"], c)
            agg[c]["ap"] += rec["appeared"][c]
            agg[c]["va"] += rec["vanished"][c]
            agg[c]["ri"] += rec["retag_in"][c]
            agg[c]["ro"] += rec["retag_out"][c]
    ok = 0
    for c in CLASSES:
        a = agg[c]
        got = a["t"] - a["b"]
        want = a["ap"] - a["va"] + a["ri"] - a["ro"]
        good = got == want
        ok += 1 if good else 0
        label = f"[{c}]" if c in MARKS else "untag"
        print(f"{label:<8} {a['b']:>6} {a['t']:>6} {got:>+6}   "
              f"{a['ap']:>9} {a['va']:>9} {a['ri']:>9} {a['ro']:>10}   "
              f"{'OK' if good else 'BROKEN'}")
    print("-" * len(h3))
    n = len(CLASSES)
    # Worded so the number cannot be read the wrong way round: "BROKEN for 5 of
    # 6" would name the classes that HOLD as if they were the ones that failed.
    if ok == n:
        print(f"DECOMPOSITION: HOLDS for all {n} classes "
              f"(Δ[X] = appeared − vanished + retag-in − retag-out)")
    else:
        print(f"DECOMPOSITION: BROKEN on {n - ok} of {n} classes "
              f"({ok} hold) — Δ[X] ≠ appeared − vanished + retag-in − retag-out")
    return ok == n


def since_records(repo_root, sha, roots, by_file=False):
    """(d, base_marked, missing_roots, tip_defects, error). The computation half
    of `--since`, with no printing, so the self-test can assert on the numbers
    themselves rather than on a parsed transcript."""
    tip_rows, tip_per_file, tip_defects = census(roots)
    if tip_rows is None:
        return None, False, [], [], "no .rs files under: " + ", ".join(roots)

    with tempfile.TemporaryDirectory() as td:
        base_rows, base_per_file, _bd, present, err = census_at(
            repo_root, sha, roots, td)
        if err:
            return None, False, [], [], err
        base_items = item_index(base_per_file)
        base_marked = any(m != UNTAGGED for m in base_items.values())
        missing = [r for r in roots if r not in present]
        if by_file:
            base_keyed = {p: f for p, (f, _) in base_per_file.items()}
        else:
            base_keyed = base_rows

    tip_items = item_index(tip_per_file)
    key_of = (lambda p: p) if by_file else module_of
    tip_keyed = ({p: f for p, (f, _) in tip_per_file.items()}
                 if by_file else tip_rows)

    d = diff_rows(base_keyed, tip_keyed, base_items, tip_items, key_of)
    return d, base_marked, missing, tip_defects, None


def run_since(repo_root, sha, roots, by_file):
    d, base_marked, missing, tip_defects, err = since_records(
        repo_root, sha, roots, by_file)
    if err:
        print(f"--since: {err}", file=sys.stderr)
        return 2
    tip_sha, tip_state = tree_stamp(repo_root)
    holds = print_diff(sha, git_show_subject(repo_root, sha), roots, d,
                       base_marked, tip_state, tip_sha)

    if missing:
        print()
        print("LABELLED OUTCOME — roots absent from the base tree: "
              + ", ".join(missing))
        print("  Their whole tip population reads as 'appeared'. That is a")
        print("  property of the base, not a measurement of this tree.")

    if not holds:
        print()
        print("DECOMPOSITION BROKEN — the differ disagrees with the census it")
        print("is differencing. Every number above is untrustworthy.", file=sys.stderr)
        return 1

    if tip_defects:
        print()
        print(f"CITATION DEFECTS AT TIP: {len(tip_defects)} marker(s) carry no citation")
        for path, ln, mk in tip_defects:
            print(f"  {path}:{ln}: PROV[{mk}] with no citation")
        return 3
    return 0


# ---------------------------------------------------------------------------
# self-test — a planted fixture with exact expected counts, and a demonstrated
# red. #3336: a control never watched failing is decoration, and this repo has
# shipped a `--check` flag that could not fail (`rustfmt --check` on stdin).
# ---------------------------------------------------------------------------

FIXTURE = '''\
//! planted fixture for provenance_census.py --self-test
//!
//! Deliberately contains every case the scanner has to get right.

/// PROV[R] W-FAKE-1 — read from a made-up address.
pub const READ_ONE: u32 = 1;

/// PROV[O] docs/FAKE.md §1 — obj-confirmed on a made-up grid.
pub(crate) const OBJ_ONE: u8 = 2;

// PROV[F] rung/fake — fitted to a made-up grid.
const FIT_ONE: usize = 3;

/// PROV[S] PE/COFF spec — a published external constant.
pub const SPEC_ONE: u16 = 4;

/// PROV[N] a debug-print width, reaches no emitted byte.
const NOT_LOAD_BEARING: usize = 5;

/// No marker at all. This is the residue the census exists to name.
pub const UNTAGGED_ONE: u32 = 6;

const UNTAGGED_TWO: u32 = 7;   // bare, no comment block at all

/// A marker attached to a FUNCTION is a rule mark, not a population member.
/// PROV[R] W-FAKE-2 — the rule, not a constant.
pub fn a_rule() -> u32 { 0 }

/// This comment mentions params[src] and a bare [R] in prose. Neither is a
/// marker and the census must not count either.
pub const PROSE_TRAP: u32 = 8;

/// The MENTION trap (#3641, #3669). This sentence discusses `PROV[R]` and
/// `PROV-BLOCK[F]` by name, in backticks. Both are mentions, neither is a
/// mark, and this constant must land in the untagged residue.
pub const MENTION_TRAP: u32 = 88;

/// A block marker: one citation covering every const in this module.
pub mod table {
    //! PROV-BLOCK[R] W-FAKE-3 — transcribed from a made-up table dump.

    pub const T0: u32 = 0;
    pub const T1: u32 = 1;

    /// PROV[F] rung/fake — an item marker BEATS the block it sits in.
    pub const T2: u32 = 2;
}

/// Outside the block again — the block must not leak past its closing brace.
pub const AFTER_BLOCK: u32 = 99;

#[cfg(test)]
mod tests {
    /// PROV[R] this must not be counted — it is in a test region.
    const IN_TEST: u32 = 9;

    #[test]
    fn t() {
        const ALSO_IN_TEST: [u8; 4] = [0, 0, 0, 1];
        let _ = (IN_TEST, ALSO_IN_TEST);
    }
}
'''

# Expected, by construction from FIXTURE above. Enumerated by hand so the
# fixture and the expectation are two independent statements: the population
# is READ_ONE, OBJ_ONE, FIT_ONE, SPEC_ONE, NOT_LOAD_BEARING, UNTAGGED_ONE,
# UNTAGGED_TWO, PROSE_TRAP, MENTION_TRAP, T0, T1, T2, AFTER_BLOCK = 13; the
# residue is UNTAGGED_ONE, UNTAGGED_TWO, PROSE_TRAP, MENTION_TRAP and
# AFTER_BLOCK = 5 (the block must NOT leak past its closing brace);
# `a_rule`'s marker is the one rule mark; everything inside `mod tests` is
# invisible. [R] = READ_ONE + T0 + T1 = 3, because T2 carries its own [F] and
# an item marker beats its block.
#
# MENTION_TRAP is the **#3669** control: its doc comment contains a backticked
# `PROV[R]` AND a backticked `PROV-BLOCK[F]`, so a census without the mention
# rule reads it as `[R]` = 4 and, worse, treats the block form as opening a
# block at that point. With the rule it is untagged, which is what a sentence
# that merely talks about markers deserves.
EXPECT = {"pop": 13, "R": 3, "O": 1, "F": 2, "S": 1, "N": 1, "untagged": 5, "rules": 1}


# ---------------------------------------------------------------------------
# The `--since` planted fixture. Two trees in a throw-away git repo, so the
# self-test exercises the REAL extraction path (`git archive`) and not a
# stubbed one.
#
# `SINCE_BASE`/`SINCE_TIP` are built so that SIX distinct provenance events
# happen while the LEVEL is completely silent: population 6 -> 6, tagged 4 -> 4,
# coverage 66.7 % -> 66.7 %. That is the whole argument for the mode in one
# fixture — a census that prints only levels reports "nothing happened" on a
# tree where a marker was lost, a judgement changed, two constants were deleted
# and two appeared.
# ---------------------------------------------------------------------------

SINCE_BASE = '''\
//! planted base tree for provenance_census.py --self-test --since

/// PROV[R] W-FAKE-1 — a read, unchanged between the two trees.
pub const KEEP_TAGGED: u32 = 1;

/// No marker at the base; the tip gives it one. This is PROVENANCE WORK.
pub const WILL_BE_TAGGED: u32 = 2;

/// PROV[O] docs/FAKE.md §1 — the tip changes this judgement to [R].
pub const WILL_BE_RECLASSED: u32 = 3;

/// PROV[F] rung/fake — the tip LOSES this marker. A regression, and the level
/// cannot see it.
pub const WILL_LOSE_MARKER: u32 = 4;

/// PROV[S] PE/COFF — this constant is DELETED in the tip.
pub const WILL_VANISH_TAGGED: u32 = 5;

/// Deleted in the tip, and it was never tagged.
pub const WILL_VANISH_UNTAGGED: u32 = 6;
'''

SINCE_TIP = '''\
//! planted tip tree for provenance_census.py --self-test --since

/// PROV[R] W-FAKE-1 — a read, unchanged between the two trees.
pub const KEEP_TAGGED: u32 = 1;

/// PROV[F] rung/fake — fitted, and newly said so. PROVENANCE WORK.
pub const WILL_BE_TAGGED: u32 = 2;

/// PROV[R] W-FAKE-9 — reclassified from [O]. A JUDGEMENT MOVED.
pub const WILL_BE_RECLASSED: u32 = 3;

/// The marker is gone. A REGRESSION.
pub const WILL_LOSE_MARKER: u32 = 4;

/// PROV[N] a debug width — NEW SURFACE that arrived already tagged.
pub const APPEARED_TAGGED: u32 = 7;

/// NEW SURFACE, unprovenanced. Never to be summed with the tagged column.
pub const APPEARED_UNTAGGED: u32 = 8;
'''

# The ordinal / line-shift fixture, in its own module row so the assertions
# above stay clean. `DUP` is declared twice in one file, which is not a
# contrivance: `c2-il/src/func/body/expr.rs` declares five distinct `const ON`.
SINCE_DUP_BASE = '''\
//! ordinal + line-shift fixture

pub fn f() -> u32 {
    const DUP: u32 = 1;
    DUP
}

pub fn g() -> u32 {
    const DUP: u32 = 2;
    DUP
}
'''

SINCE_DUP_TIP = '''\
//! ordinal + line-shift fixture
//!
//! Every line below has MOVED relative to the base. A line-keyed identity
//! would report this whole file as vanished-and-reappeared; the ordinal key
//! must report zero population churn and exactly one tagging.
//!
//! (padding)
//! (padding)

pub fn f() -> u32 {
    const DUP: u32 = 1;
    DUP
}

pub fn g() -> u32 {
    // PROV[N] a fixture ordinal — only the SECOND `DUP` is tagged.
    const DUP: u32 = 2;
    DUP
}
'''


# The #3649 fixture: a glob inside a doc comment, then a `#[cfg(test)]` region.
# Before the fix, `coff/*.rs` opened a block comment that never closed, brace
# depth froze for the rest of the file, and the two test-only constants below
# were counted as production population. The trailing production constant is the
# other half of the control: the fix must not make the scanner blind past the
# test module either.
GLOB_FIXTURE = '''\
//! planted fixture for the #3649 line-comment/glob defect.

/// This doc comment mentions `coff/*.rs` and `fixtures/cpp/*.cpp`. Neither is a
/// block comment, and a scanner that thinks otherwise stops tracking braces
/// here and never starts again.
pub const BEFORE_THE_GLOB: u32 = 1;

#[cfg(test)]
mod tests {
    const IN_TEST_ONE: u32 = 2;
    const IN_TEST_TWO: u32 = 3;
}

/// Production again, BELOW the test module.
pub const AFTER_THE_TEST_MOD: u32 = 4;
'''


def _git(repo, *args, **kw):
    return subprocess.run(
        ["git", "-c", "user.email=selftest@example.invalid",
         "-c", "user.name=selftest", "-C", repo] + list(args),
        capture_output=True, text=True, **kw)


def since_self_test(check):
    """Sections [5]-[8]: the two-tree diff, on a real throw-away git repo."""
    with tempfile.TemporaryDirectory() as td:
        repo = os.path.join(td, "repo")
        os.makedirs(os.path.join(repo, "src", "sub"))
        planted = os.path.join(repo, "src", "planted.rs")
        dup = os.path.join(repo, "src", "sub", "dup.rs")

        def write(path, text):
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(text)

        # commit 1: a tree with NO markers at all — the pre-convention base.
        write(planted, SINCE_BASE.replace("PROV[", "NOTAMARKER["))
        write(dup, SINCE_DUP_BASE)
        subprocess.run(["git", "init", "-q", repo], capture_output=True)
        _git(repo, "add", "-A")
        _git(repo, "commit", "-q", "-m", "pre-convention")
        sha_pre = _git(repo, "rev-parse", "HEAD").stdout.strip()

        # commit 2: the base tree.
        write(planted, SINCE_BASE)
        write(dup, SINCE_DUP_BASE)
        _git(repo, "add", "-A")
        _git(repo, "commit", "-q", "-m", "base")
        sha_base = _git(repo, "rev-parse", "HEAD").stdout.strip()

        # working tree: the tip.
        write(planted, SINCE_TIP)
        write(dup, SINCE_DUP_TIP)

        cwd = os.getcwd()
        os.chdir(repo)
        try:
            d, base_marked, missing, defects, err = since_records(
                repo, sha_base, ["src"])
        finally:
            os.chdir(cwd)

        print()
        print("[5] --since, planted two-tree fixture: SIX events, and the LEVEL")
        print("    is silent — pop 6→6, tagged 4→4, coverage 66.7%→66.7%")
        if err:
            print(f"  FAIL  --since could not run: {err}")
            return False
        check("no citation defects at tip", len(defects), 0)
        check("no missing roots", len(missing), 0)
        check("base carries markers", base_marked, True)

        m = d["src"]
        b, t = m["base"], m["tip"]
        check("LEVEL: population base", b["pop"], 6)
        check("LEVEL: population tip", t["pop"], 6)
        check("LEVEL: tagged base", b["pop"] - b["untagged"], 4)
        check("LEVEL: tagged tip", t["pop"] - t["untagged"], 4)
        check("LEVEL: Δtagged (the silence this mode exists for)",
              (t["pop"] - t["untagged"]) - (b["pop"] - b["untagged"]), 0)

        print("    …and the DIFF is not:")
        check("untagged→tagged", m["n_tagged"], 1)
        check("tagged→untagged (a marker LOST)", m["n_untagged"], 1)
        check("reclassified", m["n_reclass"], 1)
        check("appeared", sum(m["appeared"].values()), 2)
        check("appeared already tagged", sum(m["appeared"].values())
              - m["appeared"][UNTAGGED], 1)
        check("vanished", sum(m["vanished"].values()), 2)
        check("vanished while tagged", sum(m["vanished"].values())
              - m["vanished"][UNTAGGED], 1)
        check("decomposition identity holds", check_decomposition(m), [])

        print()
        print("[6] the ORDINAL key: two `DUP` in one file, every line shifted")
        s = d[os.path.join("src", "sub")]
        check("population unchanged", s["tip"]["pop"] - s["base"]["pop"], 0)
        check("nothing APPEARED (a line shift is not churn)",
              sum(s["appeared"].values()), 0)
        check("nothing VANISHED", sum(s["vanished"].values()), 0)
        check("exactly one of the two `DUP` got tagged", s["n_tagged"], 1)
        check("decomposition identity holds", check_decomposition(s), [])

        print()
        print("[7] a base that PREDATES the convention is a labelled outcome")
        os.chdir(repo)
        try:
            d0, base_marked0, _m0, _df0, err0 = since_records(
                repo, sha_pre, ["src"])
        finally:
            os.chdir(cwd)
        if err0:
            print(f"  FAIL  --since could not run against the pre-convention base: {err0}")
            return False
        check("base_marked is False (the banner fires)", base_marked0, False)
        check("no crash: the module row exists", "src" in d0, True)
        m0 = d0["src"]
        # KEEP_TAGGED, WILL_BE_TAGGED and WILL_BE_RECLASSED exist in both trees
        # and gain a marker; APPEARED_TAGGED is genuinely new surface and is
        # NOT counted here. That distinction surviving against a pre-convention
        # base is the point of the section.
        check("existing constants that gained a marker", m0["n_tagged"], 3)
        check("new surface is still reported separately",
              sum(m0["appeared"].values()), 2)
        check("decomposition still holds", check_decomposition(m0), [])

        print()
        print("[8] THE RED — a differ that CONFLATES 'appeared' with 'retagged'")
        print("    must be refused by the decomposition identity, not printed OK")
        conflated = {
            "base": {**blank_row(), "pop": 1, "untagged": 1},
            "tip": {**blank_row(), "pop": 2, "N": 1, "untagged": 1},
            "appeared": {c: 0 for c in CLASSES},
            "vanished": {c: 0 for c in CLASSES},
            # the bug: the appeared-already-tagged constant booked as a retag
            "retag_in": {**{c: 0 for c in CLASSES}, "N": 1},
            "retag_out": {**{c: 0 for c in CLASSES}, UNTAGGED: 1},
            "n_tagged": 1, "n_untagged": 0, "n_reclass": 0,
        }
        bad = check_decomposition(conflated)
        check("the conflation is DETECTED", len(bad) > 0, True)
        check("and it is detected on the untagged residue",
              any(c == UNTAGGED for c, _g, _w in bad), True)

        print()
        print("[9] a bogus sha is a labelled failure (exit 2), never a crash")
        os.chdir(repo)
        try:
            rc = run_since(repo, "0000000000000000000000000000000000000000",
                           ["src"], False)
        finally:
            os.chdir(cwd)
        check("exit code", rc, 2)

    return True


def self_test():
    ok = True

    def check(label, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'PASS' if good else 'FAIL'}  {label}: got {got}, want {want}")

    with tempfile.TemporaryDirectory() as td:
        src = os.path.join(td, "planted.rs")
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(FIXTURE)

        print("[1] planted fixture, exact counts")
        rows, _, defects = census([src])
        r = list(rows.values())[0]
        for k, v in EXPECT.items():
            check(k, r[k], v)
        check("citation defects", len(defects), 0)

        print()
        print("[2] THE RED — remove one marker; the count MUST move")
        with open(src, "r", encoding="utf-8") as fh:
            text = fh.read()
        mutated = text.replace("/// PROV[R] W-FAKE-1 — read from a made-up address.\n", "")
        if mutated == text:
            print("  FAIL  mutation did not apply — the control cannot be trusted")
            return 1
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(mutated)
        rows2, _, _ = census([src])
        r2 = list(rows2.values())[0]
        print(f"  before: [R]={r['R']} untagged={r['untagged']}")
        print(f"  after:  [R]={r2['R']} untagged={r2['untagged']}")
        check("[R] fell by one", r["R"] - r2["R"], 1)
        check("untagged rose by one", r2["untagged"] - r["untagged"], 1)
        check("population unchanged", r2["pop"], r["pop"])

        print()
        print("[3] THE OTHER RED — an uncited marker is a reported defect")
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(text.replace(
                "/// PROV[O] docs/FAKE.md §1 — obj-confirmed on a made-up grid.",
                "/// PROV[O]",
            ))
        _, _, defects3 = census([src])
        check("one citation defect found", len(defects3), 1)

        print()
        print("[4] THE THIRD RED — remove the BLOCK marker; two tags MUST move")
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(text.replace(
                "    //! PROV-BLOCK[R] W-FAKE-3 — transcribed from a made-up table dump.\n",
                "",
            ))
        rows4, _, _ = census([src])
        r4 = list(rows4.values())[0]
        print(f"  before: [R]={r['R']} untagged={r['untagged']}")
        print(f"  after:  [R]={r4['R']} untagged={r4['untagged']}")
        check("[R] fell by two", r["R"] - r4["R"], 2)
        check("untagged rose by two", r4["untagged"] - r["untagged"], 2)

    print()
    print("[10] #3649 — a glob in a doc comment must not open a block comment,")
    print("     and must not disable `#[cfg(test)]` exclusion below it")
    with tempfile.TemporaryDirectory() as td:
        src = os.path.join(td, "globbed.rs")
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(GLOB_FIXTURE)
        rowsg, _, _ = census([src])
        rg = list(rowsg.values())[0]
        check("population is the TWO production consts, not four", rg["pop"], 2)
        check("untagged residue", rg["untagged"], 2)
        # The red: with the fix reverted, the glob swallows the file and the two
        # test constants are counted. Demonstrated by feeding the scanner the
        # same file with the glob removed — the count must NOT move, which is
        # what "the glob is irrelevant" means and what was false before.
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(GLOB_FIXTURE.replace("`coff/*.rs` and `fixtures/cpp/*.cpp`",
                                          "two paths"))
        rowsg2, _, _ = census([src])
        rg2 = list(rowsg2.values())[0]
        check("removing the glob moves NOTHING (it never should have)",
              (rg2["pop"], rg2["untagged"]), (rg["pop"], rg["untagged"]))

    print()
    print("[11] #3669 — THE MENTION RULE, watched in BOTH directions. A")
    print("     backticked marker is a MENTION and must not be counted; the")
    print("     bare one beside it must.")
    with tempfile.TemporaryDirectory() as td:
        src = os.path.join(td, "mention.rs")
        both = (
            "/// PROV[R] W-FAKE-1 — a real mark, bare.\n"
            "pub const BARE: u32 = 1;\n"
            "\n"
            "/// A sentence about `PROV[R]` and `PROV-BLOCK[R]`, in backticks.\n"
            "pub const MENTIONED: u32 = 2;\n")
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(both)
        rowsm, _, _ = census([src])
        rm = list(rowsm.values())[0]
        check("population is two", rm["pop"], 2)
        check("exactly ONE is tagged [R]", rm["R"], 1)
        check("and the mention is in the residue", rm["untagged"], 1)
        # THE RED: strip the backticks and the same file counts differently.
        # A rule that cannot be made to change the answer is not a rule.
        with open(src, "w", encoding="utf-8") as fh:
            fh.write(both.replace("`PROV[R]`", "PROV[R]"))
        rowsm2, _, _ = census([src])
        rm2 = list(rowsm2.values())[0]
        check("un-backticking the mention MOVES the count", rm2["R"], 2)
        check("and empties the residue", rm2["untagged"], 0)

    if not since_self_test(check):
        ok = False

    print()
    print("SELF-TEST:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def main(argv):
    here = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(here)

    args = argv[1:]
    if "--self-test" in args:
        return self_test()

    by_file = "--by-file" in args
    args = [a for a in args if a != "--by-file"]

    since = None
    if "--since" in args:
        i = args.index("--since")
        if i + 1 >= len(args):
            print("--since needs a base revision (a sha, tag or ref)", file=sys.stderr)
            return 2
        since = args[i + 1]
        args = args[:i] + args[i + 2:]

    list_untagged = None
    if "--list-untagged" in args:
        i = args.index("--list-untagged")
        if i + 1 >= len(args):
            print("--list-untagged needs a module path", file=sys.stderr)
            return 2
        list_untagged = args[i + 1]
        args = args[:i] + args[i + 2:]

    roots = args or [os.path.join(repo_root, "crates")]
    roots = [os.path.relpath(os.path.abspath(r), repo_root) for r in roots]
    cwd = os.getcwd()
    os.chdir(repo_root)
    try:
        if since is not None:
            if list_untagged is not None:
                print("--since and --list-untagged are different questions; "
                      "pick one", file=sys.stderr)
                return 2
            return run_since(repo_root, since, roots, by_file)

        rows, per_file, defects = census(roots)
        if rows is None:
            print("no .rs files under: " + ", ".join(roots), file=sys.stderr)
            return 2

        if list_untagged is not None:
            target = os.path.relpath(os.path.abspath(list_untagged), repo_root)
            n = 0
            for path, (f, items) in sorted(per_file.items()):
                if not path.startswith(target):
                    continue
                for ln, name, mark in items:
                    if mark is None:
                        print(f"{path}:{ln}: {name}")
                        n += 1
            print(f"-- {n} untagged load-bearing candidates under {target}")
            return 0

        if by_file:
            frows = {p: f for p, (f, _) in per_file.items()}
            print_table(frows, ", ".join(roots) + " (per file)", repo_root)
        else:
            print_table(rows, ", ".join(roots), repo_root)

        if defects:
            print()
            print(f"CITATION DEFECTS: {len(defects)} marker(s) carry no citation")
            for path, ln, mk in defects:
                print(f"  {path}:{ln}: PROV[{mk}] with no citation")
            return 3
        return 0
    finally:
        os.chdir(cwd)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
