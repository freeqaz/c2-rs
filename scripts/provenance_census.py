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

USAGE
  scripts/provenance_census.py                     census over crates/
  scripts/provenance_census.py --by-file           per-file rows
  scripts/provenance_census.py --list-untagged M   name the residue of module M
  scripts/provenance_census.py --self-test         planted fixture, exact
                                                   counts, plus a marker
                                                   removal that MUST move the
                                                   count (a control never seen
                                                   red is decoration, #3336)

EXIT CODES
  0  census printed (or self-test passed)
  1  self-test failed
  2  could not run (bad root, no files found)
  3  a marker carries no citation — a defect, reported by name
"""

import os
import re
import subprocess
import sys
import tempfile

MARKS = ("R", "O", "F", "S", "N")

# The marker token. Prefixed deliberately: a bare `[R]` grep cannot tell a
# marker from an array index — at tree `6c753ead0` the brief's sixth "marker"
# was `params[src]` in `c2-il/src/func/mod.rs:2013` — and three prose `[R]`s
# in `codegen/` module docs would otherwise be counted as tags they are not.
MARK_RE = re.compile(r"PROV\[([RSOFN])\]")

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
                continue
        if "/*" in line and "*/" not in line.split("/*", 1)[1]:
            line = line.split("/*", 1)[0]
            in_block_comment = True

        stripped = line.strip()
        if stripped.startswith("#[cfg(test)]") or stripped.startswith("#[test]"):
            pending_test_attr = True

        in_test[i] = test_depth is not None or is_test_file

        clean = strip_for_braces(line)
        opens = clean.count("{")
        closes = clean.count("}")
        if pending_test_attr and opens > 0:
            if test_depth is None:
                test_depth = depth
                in_test[i] = True
            pending_test_attr = False
        depth += opens - closes
        if test_depth is not None and depth <= test_depth:
            test_depth = None

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
# UNTAGGED_TWO, PROSE_TRAP = 8; the residue is the last three; `a_rule`'s
# marker is the one rule mark; everything inside `mod tests` is invisible.
EXPECT = {"pop": 8, "R": 1, "O": 1, "F": 1, "S": 1, "N": 1, "untagged": 3, "rules": 1}


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
