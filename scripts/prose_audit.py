#!/usr/bin/env python3
"""prose_audit.py — is the CLAIM inside a provenance marker or a provenance
document TRUE?

Board **#3667**. Lane `w-provaudit`, decision 17
(`docs/DECISIONS_2026-08-22.md`). Convention:
`docs/whitebox/DISCLOSURE.md` § "The in-code provenance markers".

THE DEFECT CLASS, AND WHY NO EXISTING CONTROL REACHES IT
--------------------------------------------------------
Every control this repo owns catches a **fabricated number**: the byte judge
catches a wrong emitted byte, `gate_identity_diff.sh` catches a moved gate
count, `provenance_census.py` catches an untagged constant, `board_audit.sh`
catches a cited row that does not exist, `doc_cite_audit.sh` catches a citation
that does not resolve.

**This defect class fabricates no number at all.** Board **#3643**: a
`PROV[R]` marker in `crates/c2-core/src/codegen/mop.rs` said the port emits
*"**71** distinct opcodes and the other **589** are not transcribed"*, over
*"**24** of c2's **109** forms"*. The truth is **85 / 575 / 34 of 104**. The
marker was present, well-formed, cited, and counted by the census as a tag —
and its prose was false. It had been false since the file's first commit, the
same file had **575** right one comment away, and the wrong figure was quoted
forward into a board row and into `DISCLOSURE.md`. Nothing moved. Nothing could
move: every one of those numbers lives in a doc comment, so the identity diff
is 0 over 21 rows by construction.

Board **#3641** is the same family from the other side: writing prose *about*
mark letters moved a subsystem's own agreement census **9/28 → 13/34**, because
the counter cannot tell an evidence mark from a mention of one.

So the census can say **whether** a constant is tagged. It cannot say whether
the tag is **TRUE**. That is the difference between a provenance record and a
provenance *claim*, and the project's primary goal rests on the former.

WHAT THIS TOOL CHECKS — the shape list, and it is deliberately short
-------------------------------------------------------------------
"Every claim in every comment" is not achievable and is not attempted. What is
achievable is a small set of **shapes** where a claim is mechanically
recheckable. Six of them can go red:

  C1 ROW-REF      a `W-<NAME>-<N>` token cited in the tree must name a row in
                  DISCLOSURE.md's adopted-findings table.
  C2 ABSENCE      prose asserting that NO ledger row exists for an address,
                  checked against the ledger's own address column. An absence
                  claim is the easiest kind to falsify mechanically and the
                  likeliest to rot, because absences get FILLED.
  C3 SELF-COUNT   a document stating the size of a table it itself contains.
  C4 BOUND-COUNT  a prose number bound by an explicit `COUNT[<recipe>]`
                  annotation to a population this tool can recount.
  C5 MENTION-RISK a counted mark token sitting in discussion context rather
                  than annotating anything — #3641's class, as a detector.
  C6 ADOPTED-PATH every path in a ledger row's `Adopted into` column resolves.

And one inventory that cannot go red, and is the most important line of output:

  I7 UNBOUND      every numeric claim on a provenance surface that NONE of the
                  above can reach, counted and printed on every run.

WHAT THIS TOOL CANNOT SEE — printed on every run, for one reason
----------------------------------------------------------------
**An audit whose coverage is unstated will be read as total.** That is this
repo's trap 5 (*absence reads as success unless something forbids it*) pointed
at an auditor instead of at a gate. So the residue is a NUMBER, not a sentence:
I7 counts the claims C1–C6 cannot reach, and a run where I7 dwarfs the checked
population is the honest report of a narrow tool, not a clean bill of health.

Specifically out of scope, and each of these is a different reason:

  * **Any claim of fact about c2's behaviour.** *"c2 reads the field through
    the VI32 reader"* is graded by the image and by the byte judge, never here.
  * **Whether a cited ADDRESS contains the instruction claimed.** C1/C6 grade
    citation TARGETS. `work/w-inlmetric/addrcheck.py` asks the other question
    (is address A inside the function the page names) and is not folded in.
  * **Whether a bound count is bound to the RIGHT population.** C4 checks the
    arithmetic of a binding a human wrote. A recipe aimed at the wrong array is
    green and wrong, and no tool fixes that.
  * **`file.md:NNN` line-citation staleness** — `doc_cite_audit.sh`'s own
    stated LIMIT, not duplicated here.
  * **Numbers with no binding.** The default state of a prose number in this
    tree is UNREACHABLE. That is what I7 measures.

DATED RECORDS ARE NOT LIVE SURFACES, AND THE TOOL SAYS WHICH IS WHICH
---------------------------------------------------------------------
`docs/rungs/**`, `docs/BOARD.md`, `docs/ROADMAP.md` and the
`docs/whitebox/WB_*_FINDINGS.md` grade pages are **dated records**: this repo's
standing rule is that they stay as written and corrections live in rows that
CITE them. A dead `W-EXT-1` in a rung from 2026-08-08 is a record of what that
lane drafted, not a defect. A dead `W-EXT-1` in `crates/` is a defect.

So C1 splits its unresolved tokens into **LIVE** (red) and **DATED**
(reported, counted, never red). The split is a property of the path, it is
printed, and `--strict` collapses it if a caller wants the harsher reading.

USAGE
  scripts/prose_audit.py                 audit the tree, print findings
  scripts/prose_audit.py --verbose       also print every candidate considered
  scripts/prose_audit.py --strict        DATED unresolved refs count as findings
  scripts/prose_audit.py --self-test     planted fixtures; RED on a false claim
                                         and GREEN on a true one, both directions

EXIT CODES
  0  audited, no findings (or self-test passed)
  1  findings, or self-test failed
  2  could not run (missing ledger, bad root)
  3  nothing was checked — a checker with no subject is decoration, and this
     is #1002/#3470's rule: only a denominator catches an absence
"""

import os
import re
import sys
import tempfile

# ---------------------------------------------------------------------------
# scope
# ---------------------------------------------------------------------------

LEDGER = "docs/whitebox/DISCLOSURE.md"

# Roots scanned for CLAIMS. Reading is unfenced; this tool writes nothing.
SCAN_ROOTS = ("crates", "docs", "scripts", "c2host")
SCAN_FILES = ("README.md", "CLAUDE.md")
SCAN_EXTS = (".md", ".rs", ".c", ".h", ".py", ".sh")

# Paths whose content is a DATED RECORD of the day it was written. An
# unresolved reference from one of these is reported, never red — see the
# module doc.
DATED_PREFIXES = (
    "docs/rungs/",
    "docs/BOARD.md",
    "docs/ROADMAP.md",
    "docs/DECISIONS_",
    "docs/whitebox/WB_",
    "docs/whitebox/READ_PLAN_",
    "docs/STRATEGY_REVIEW_",
    "docs/ARCH_REVIEW_",
    "docs/ARCHITECTURE_PROPOSAL_",
    "docs/GOAL_DECISION_",
    "docs/ROADMAP_SLICING_",
    "docs/STEP5_PRICING_",
    "docs/WHITEBOX_LEVERAGE_",
    "docs/SHIPPING_ROADMAP_",
    "work/",
)

# The surfaces I7 inventories: where a provenance CLAIM lives.
PROV_DOC_SURFACES = (
    "docs/whitebox/DISCLOSURE.md",
    "docs/whitebox/ref/README.md",
)

# The `P_*.md` pages `crates/c2-harness/src/subsys.rs::count_marks` counts.
# Listed here so C5 grades the SAME surface the live counter grades; the list is
# read off `subsys.rs`'s own `page:` fields and is asserted against them at run
# time so it cannot drift silently.
SUBSYS_PAGES = (
    "P_COFF.md", "P_SECTION.md", "P_REGALLOC.md", "P_GLOBREGS.md", "P_DAG.md",
    "P_INLINE.md", "P_ENCODE.md", "P_EH.md", "P_LABEL.md", "P_SYMBOL.md",
)
SUBSYS_RS = "crates/c2-harness/src/subsys.rs"

# ---------------------------------------------------------------------------
# tokens
# ---------------------------------------------------------------------------

ROW_TOKEN_RE = re.compile(r"\bW-[A-Z][A-Z0-9]*-\d+\b")

# `W-FAKE-*` is RESERVED for planted fixtures. Without a reserved namespace an
# auditor reports its own controls as findings — the same shape as a `pgrep -f`
# wait-loop matching its own argv, and it happened on this tool's first real
# run (9 hits from its own self-test strings, 6 more from the census's).
FIXTURE_TOKEN_RE = re.compile(r"^W-FAKE-\d+$")

# Files whose CONTENT is planted fixtures for these instruments. A tool cannot
# audit its own controls: the bindings inside `prose_audit.py`'s fixture strings
# point at `crates/planted/real.rs`, which exists only inside a temp dir during
# `--self-test`. Named rather than pattern-matched, and printed on every run.
SELF_FIXTURE_FILES = ("scripts/prose_audit.py", "scripts/provenance_census.py")
LEDGER_ROW_RE = re.compile(r"^\|\s*\*\*(W-[A-Z][A-Z0-9]*-\d+)\*\*\s*\|")
ADDR_RE = re.compile(r"\b0x1[0-9a-fA-F]{7}\b")

# A `PROV[X]` / `PROV-BLOCK[X]` marker, spelled exactly as
# `scripts/provenance_census.py` spells it. Kept in sync by the self-test,
# which imports that module and compares the two patterns' behaviour on a
# planted string rather than comparing the patterns themselves.
PROV_RE = re.compile(r"PROV(?:-BLOCK)?\[([RSOFN])\]")

# The evidence marks `subsys.rs::count_marks` counts, verbatim.
SUBSYS_MARK_RE = re.compile(r"\[([ROI])\]")

# The BINDING. `COUNT[<recipe>] = <N>` written beside a prose number.
BIND_RE = re.compile(r"COUNT\[([^\]]+)\]\s*=\s*(\d+)")

NUMBER_WORDS = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
    "seven": 7, "eight": 8, "nine": 9, "ten": 10, "eleven": 11, "twelve": 12,
    "thirteen": 13, "fourteen": 14, "fifteen": 15, "sixteen": 16,
    "seventeen": 17, "eighteen": 18, "nineteen": 19, "twenty": 20,
    "twenty-one": 21, "twenty-two": 22, "twenty-three": 23, "thirty": 30,
    "forty": 40, "fifty": 50,
}

# C1 suppression: a line that DECLARES its own token to be unresolved is
# honest, not defective. Every phrase here is printed with its hit count.
#
# **The list is deliberately NARROW, and the self-test is why.** The first
# draft carried `"does not exist"` and `"no row"`, and section [2] of the
# self-test went green-when-it-should-have-been-red: the planted fixture's own
# sentence *"W-FAKE-9 — a row that does not exist"* suppressed the very finding
# it existed to provoke. A suppression class wide enough to swallow the control
# is wide enough to swallow the defect. Every phrase below names a DRAFT
# STATUS; none of them is a general statement of absence.
PREDRAFT_PHRASES = (
    "pre-draft", "pre-drafted", "predraft", "predrafted",
    "not carried", "never been carried", "no lane ever carried",
    "not adopted", "no such row", "not a row",
    "no adopted row", "drafted, not", "is not carried",
)

# C2: the shapes an absence claim takes in this tree.
ABSENCE_PHRASES = (
    "no disclosure row exists",
    "no row names this address",
    "no row in this ledger",
    "no such row exists",
    "has no row",
    "no row exists for this address",
    "the ledger has no row",
    "no row in disclosure",
)

# C3: the self-counting populations this tool knows about. One entry today.
# A population is registered here, never guessed: the tool states the size of a
# table only where somebody has said which table.
SELF_COUNTS = (
    # (file, noun, self-reference qualifiers, counter)
    (LEDGER, "rows",
     ("exhaustively", "this ledger", "this file", "the table above",
      "rows in this ledger", "adopted findings", "adopted rows"),
     "ledger_rows"),
)

# How close a self-reference qualifier must sit to the "<N> rows" it qualifies.
# **Measured, not chosen**: on the first real run the qualifier list contained
# the bare phrase "the ledger", and `W-MOP-2`'s row — which opens *"THE
# LEDGER'S FIRST `Adopted into` …"* and separately says *"85 of c2's 660
# rows"* about a c2 table 400 characters away — produced two false C3 findings.
# A qualifier at the far end of a 2,000-character table row qualifies nothing.
SELF_COUNT_PROXIMITY = 80

# C5: the vocabulary that turns a mark token into a MENTION of a mark.
META_WORDS = (
    "legend", "mark letter", "marker convention", "the marker", "a marker",
    "mentions", "mention", "misuse", "counts every", "the census counts",
    "not a marker", "vocabulary", "respell", "count_marks", "prov[",
    "the mark", "marks are", "a mark ", "quoted verbatim",
)


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def split_md_row(line):
    """Split a markdown table row on UNESCAPED pipes. The ledger's cells
    contain `\\|` (a literal pipe inside a bit-OR expression) in several rows,
    and a naive `split('|')` shears them into the wrong column — which would
    make C6 read half a path."""
    cells = []
    cur = []
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if c == "\\" and i + 1 < n and line[i + 1] == "|":
            cur.append("|")
            i += 2
            continue
        if c == "|":
            cells.append("".join(cur))
            cur = []
            i += 1
            continue
        cur.append(c)
        i += 1
    cells.append("".join(cur))
    return cells


def read(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read()
    except OSError:
        return None


def is_dated(rel):
    return any(rel.startswith(p) for p in DATED_PREFIXES)


def walk_scan(root):
    """Every file in scope, as repo-relative paths, in path order."""
    out = []
    for r in SCAN_FILES:
        if os.path.isfile(os.path.join(root, r)):
            out.append(r)
    for d in SCAN_ROOTS:
        base = os.path.join(root, d)
        if not os.path.isdir(base):
            continue
        for dirpath, dirnames, filenames in os.walk(base):
            dirnames[:] = [x for x in sorted(dirnames)
                           if x not in ("target", ".git", "__pycache__",
                                        "node_modules")]
            for fn in sorted(filenames):
                if fn.endswith(SCAN_EXTS):
                    out.append(os.path.relpath(
                        os.path.join(dirpath, fn), root))
    return sorted(set(out))


# ---------------------------------------------------------------------------
# the ledger
# ---------------------------------------------------------------------------

class Ledger:
    """DISCLOSURE.md's adopted-findings table, parsed.

    Nothing here guesses: a row is a line that starts `| **W-…-N** |`, which is
    the file's own format and the same shape `grep -c '^| \\*\\*W-'` counts.
    """

    def __init__(self, text):
        self.rows = {}          # id -> list of cells
        self.addresses = set()  # every 0x1……… cited by any row
        self.order = []
        for line in text.split("\n"):
            m = LEDGER_ROW_RE.match(line)
            if not m:
                continue
            rid = m.group(1)
            cells = [c.strip() for c in split_md_row(line)]
            # leading and trailing empties from the outer pipes
            if cells and cells[0] == "":
                cells = cells[1:]
            if cells and cells[-1] == "":
                cells = cells[:-1]
            self.rows[rid] = cells
            self.order.append(rid)
            for a in ADDR_RE.findall(line):
                self.addresses.add(a.lower())

    def __len__(self):
        return len(self.rows)

    def adopted_into_paths(self):
        """(row_id, path) for every tree path named in the `Adopted into`
        column. Column 4 (0-based) of the seven-column table."""
        out = []
        for rid in self.order:
            cells = self.rows[rid]
            if len(cells) < 5:
                continue
            cell = cells[4]
            for p in re.findall(
                    r"(?:crates|c2host|docs|scripts|fixtures)/[A-Za-z0-9_./-]+",
                    cell):
                out.append((rid, p.rstrip(".,;`")))
        return out


# ---------------------------------------------------------------------------
# recount recipes — C4's vocabulary
# ---------------------------------------------------------------------------

def rs_items(root, relpath):
    """Non-test `const`/`static` items, using provenance_census's own scanner
    so the two instruments cannot disagree about what a population member is."""
    sys.path.insert(0, os.path.join(root, "scripts"))
    try:
        import provenance_census as pc
    except ImportError:
        return None
    finally:
        sys.path.pop(0)
    p = os.path.join(root, relpath)
    if not os.path.isfile(p):
        return None
    items, _rules, _defects = pc.scan_file(p)
    return items


def array_entries(text, ident):
    """Top-level entries of the array/slice literal assigned to `ident`.

    Counts commas at bracket depth 1 relative to the opening `[`, which is what
    "one entry" means for a table of struct literals. Returns None if the
    identifier is not found — a recipe that cannot resolve is a FINDING, never
    a silent zero."""
    m = re.search(r"\b" + re.escape(ident) + r"\b\s*:[^=]*=\s*[&]?\[", text)
    if not m:
        return None
    i = text.index("[", m.end() - 1)
    depth = 0
    n = 0
    j = i
    in_str = False
    in_chr = False
    in_line_comment = False
    while j < len(text):
        c = text[j]
        if in_line_comment:
            if c == "\n":
                in_line_comment = False
            j += 1
            continue
        if in_str:
            if c == "\\":
                j += 2
                continue
            if c == '"':
                in_str = False
            j += 1
            continue
        if in_chr:
            if c == "\\":
                j += 2
                continue
            if c == "'":
                in_chr = False
            j += 1
            continue
        if c == "/" and j + 1 < len(text) and text[j + 1] == "/":
            in_line_comment = True
            j += 2
            continue
        if c == '"':
            in_str = True
            j += 1
            continue
        if c in "[({":
            depth += 1
        elif c in "])}":
            depth -= 1
            if depth == 0:
                # a trailing comma leaves n == entries; no trailing comma
                # leaves n == entries - 1. Both are handled by counting the
                # non-empty text between separators instead.
                break
        elif c == "," and depth == 1:
            n += 1
        j += 1
    # entries = separators + 1, unless the literal ends with a trailing comma,
    # in which case separators == entries. Decide by looking backwards from the
    # closing bracket.
    k = j - 1
    while k > i and text[k] in " \t\r\n":
        k -= 1
    return n if text[k] == "," else n + 1


def run_recipe(root, recipe):
    """(value, error). Recipes are deliberately few and each one is a COUNT of
    something the tool can enumerate. A recipe that cannot resolve returns an
    error and the binding is a FINDING — never a zero that reads as agreement.
    """
    parts = recipe.split(":")
    kind = parts[0]

    if kind == "ledger-rows" and len(parts) == 1:
        text = read(os.path.join(root, LEDGER))
        if text is None:
            return None, f"cannot read {LEDGER}"
        return len(Ledger(text)), None

    if kind == "md-rows" and len(parts) >= 3:
        path = parts[1]
        pat = ":".join(parts[2:])
        text = read(os.path.join(root, path))
        if text is None:
            return None, f"cannot read {path}"
        try:
            rx = re.compile(pat)
        except re.error as e:
            return None, f"bad regex: {e}"
        return sum(1 for l in text.split("\n") if rx.search(l)), None

    if kind == "grep" and len(parts) >= 3:
        path = parts[1]
        pat = ":".join(parts[2:])
        text = read(os.path.join(root, path))
        if text is None:
            return None, f"cannot read {path}"
        try:
            rx = re.compile(pat)
        except re.error as e:
            return None, f"bad regex: {e}"
        return sum(1 for l in text.split("\n") if rx.search(l)), None

    if kind == "rs-consts" and len(parts) >= 2:
        items = rs_items(root, parts[1])
        if items is None:
            return None, f"cannot scan {parts[1]}"
        return len(items), None

    if kind == "rs-marks" and len(parts) == 3:
        letter = parts[2]
        text = read(os.path.join(root, parts[1]))
        if text is None:
            return None, f"cannot read {parts[1]}"
        return sum(1 for m in PROV_RE.finditer(text)
                   if m.group(1) == letter), None

    if kind == "rs-array" and len(parts) == 3:
        text = read(os.path.join(root, parts[1]))
        if text is None:
            return None, f"cannot read {parts[1]}"
        v = array_entries(text, parts[2])
        if v is None:
            return None, f"no array literal named {parts[2]} in {parts[1]}"
        return v, None

    return None, f"unknown recipe: {recipe}"


# ---------------------------------------------------------------------------
# the checks
# ---------------------------------------------------------------------------

class Finding:
    def __init__(self, check, path, line, msg):
        self.check = check
        self.path = path
        self.line = line
        self.msg = msg

    def __str__(self):
        return f"  [{self.check}] {self.path}:{self.line}: {self.msg}"


ATTRIB_WINDOW = 24


def _attributes_to_ledger(line, tok):
    """Does this line say the token is a DISCLOSURE row?

    Adjacency matters. `DISCLOSURE W-EXT-1` attributes; a line that names
    `DISCLOSURE.md` at one end and an unrelated `W-LOOP-2` at the other does
    not. The window is characters, not words, because the tree writes the
    attribution as `DISCLOSURE W-EXT-1`, `DISCLOSURE draft **W-EXT-1**` and
    `DISCLOSURE row W-EH-1` — all inside 24 characters.
    """
    low = line.lower()
    for m in re.finditer(re.escape(tok), line):
        lo = max(0, m.start() - ATTRIB_WINDOW)
        if "disclosure" in low[lo:m.start()]:
            return True
    return False


def predraft_index(root, files):
    """{token: {file, …}} — every `W-<NAME>-<N>` DEFINED as a table row
    somewhere in `docs/` that is not the ledger. These are the pre-drafts:
    `WB_EH_FINDINGS.md` §5 defines `W-EH-1`, `WB_READER_FINDINGS.md` §5.3
    defines `W-EXT-1`, and so on. A citation that NAMES its home document is
    honest and resolvable; a citation that says `DISCLOSURE W-EXT-1` is not."""
    idx = {}
    for rel in files:
        if rel == LEDGER or not rel.endswith(".md"):
            continue
        text = read(os.path.join(root, rel))
        if text is None:
            continue
        for line in text.split("\n"):
            m = LEDGER_ROW_RE.match(line.lstrip("> "))
            if m:
                idx.setdefault(m.group(1), set()).add(os.path.basename(rel))
    return idx


def check_rowref(root, files, ledger, strict, predrafts):
    """C1 — a citation that ATTRIBUTES a `W-<NAME>-<N>` token to this ledger
    must name a row that is in it.

    **The row-id grammar is not a reserved namespace, and that is itself a
    finding.** `W-UNW-1` is used 32 times in this tree as a FIXTURE-FAMILY
    label (`sweep.d/70-framed.py`, `differential.rs`, `GAPS.md` beside `W13b`
    / `W14`) and is defined as a DISCLOSURE row nowhere. A checker that treats
    every `W-*-N` as a ledger citation reports 32 false positives and is
    thrown away on first use. So the token is graded by its ATTRIBUTION:

      * in the ledger                       -> resolved
      * line declares it a pre-draft        -> suppressed, counted
      * line names the doc that defines it  -> suppressed, counted
      * line says DISCLOSURE                -> **FINDING** (the sharp case)
      * defined as a pre-draft elsewhere,
        cited without saying so             -> FINDING (LIVE) / reported (DATED)
      * attributed to nothing at all        -> UNATTRIBUTED, its own class,
                                               reported and never red
    """
    findings = []
    dated = []
    unattributed = []
    suppressed_predraft = 0
    suppressed_home = 0
    suppressed_fixture = 0
    checked = 0
    for rel in files:
        if rel in SELF_FIXTURE_FILES:
            continue
        text = read(os.path.join(root, rel))
        if text is None:
            continue
        lines = text.split("\n")
        for i, line in enumerate(lines, 1):
            toks = set(ROW_TOKEN_RE.findall(line))
            if not toks:
                continue
            # a declaring phrase may sit on the next line of a bullet, so the
            # window is [i-1, i+2] — measured against the ledger's own
            # amend-beside boxes, which put the token and its disposition on
            # separate lines.
            window = "\n".join(lines[max(0, i - 2):i + 2]).lower()
            declares = any(p in window for p in PREDRAFT_PHRASES)
            row_line = LEDGER_ROW_RE.match(line) is not None
            quoted_box = rel == LEDGER and line.lstrip().startswith(">")
            for tok in sorted(toks):
                if rel == LEDGER and row_line:
                    continue  # the definition itself
                if FIXTURE_TOKEN_RE.match(tok):
                    suppressed_fixture += 1
                    continue
                checked += 1
                if tok in ledger.rows:
                    continue
                homes = predrafts.get(tok, set())
                # **ATTRIBUTION OUTRANKS EVERY SUPPRESSION, and #3645 is why.**
                # `middle_interfaces.rs:634` reads
                # `(WB_READER_FINDINGS.md §3.2 / DISCLOSURE W-EXT-1)`. It names
                # the home document AND makes a false attribution to the ledger
                # in the same breath. Checked home-first, that line is
                # suppressed and the one dead citation this lane was dispatched
                # to find disappears — which is what the first draft did.
                attributes = _attributes_to_ledger(line, tok)
                if declares and not attributes:
                    suppressed_predraft += 1
                    continue
                if homes and not attributes and any(h in line for h in homes):
                    suppressed_home += 1
                    continue
                if attributes:
                    msg = (f"attributes `{tok}` to {LEDGER}, which has no such "
                           f"row"
                           + (f" (it is a pre-draft in {', '.join(sorted(homes))})"
                              if homes else ""))
                elif homes:
                    msg = (f"cites `{tok}` bare; it is a PRE-DRAFT in "
                           f"{', '.join(sorted(homes))} and not a ledger row, "
                           f"and this line does not say so")
                else:
                    unattributed.append(Finding(
                        "C1", rel, i,
                        f"`{tok}` matches the ledger's row-id grammar but is "
                        f"defined nowhere and attributed to nothing"))
                    continue
                f = Finding("C1", rel, i, msg)
                if (is_dated(rel) or quoted_box) and not strict:
                    dated.append(f)
                else:
                    findings.append(f)
    return (findings, dated, unattributed,
            (suppressed_predraft, suppressed_home, suppressed_fixture), checked)


QUOTE_CHARS = '"“”'


def _inside_quotes(line, pos):
    """Is `pos` inside a quotation on this line?

    **A QUOTED claim is not a claim.** `DISCLOSURE.md`'s own `W-EXCLASS-1` row
    quotes the stale marker — *"the marker on the constant said "NO DISCLOSURE
    ROW EXISTS FOR THIS ADDRESS" in those words … That sentence is now false"* —
    and reporting the ledger for repeating the sentence it is correcting is the
    same category error as counting a mention of a mark as a mark (**#3641**).
    So the quote is the disambiguator here, and it is the same convention C5
    proposes for marks: a token inside delimiters is a MENTION.
    """
    return sum(1 for c in line[:pos] if c in QUOTE_CHARS) % 2 == 1


def check_absence(root, files, ledger, strict):
    """C2 — an absence claim the ledger falsifies."""
    findings = []
    dated = []
    quoted = 0
    checked = 0
    for rel in files:
        if rel in SELF_FIXTURE_FILES:
            continue
        text = read(os.path.join(root, rel))
        if text is None:
            continue
        lines = text.split("\n")
        for i, line in enumerate(lines, 1):
            low = line.lower()
            hit = None
            for p in ABSENCE_PHRASES:
                j = low.find(p)
                if j >= 0:
                    hit = (p, j)
                    break
            if hit is None:
                continue
            phrase, at = hit
            if _inside_quotes(line, at):
                quoted += 1
                continue
            # the address the claim is about: on the claim's own line, or in
            # the two lines either side (a marker's citation often precedes it)
            window = "\n".join(lines[max(0, i - 3):i + 2])
            addrs = {a.lower() for a in ADDR_RE.findall(window)}
            if not addrs:
                continue
            checked += 1
            live = sorted(addrs & ledger.addresses)
            if live:
                rows = sorted({rid for rid in ledger.order
                               if any(a in " ".join(ledger.rows[rid]).lower()
                                      for a in live)})
                f = Finding(
                    "C2", rel, i,
                    f'claims "{phrase}" but the ledger cites '
                    f"{', '.join(live[:6])}{' …' if len(live) > 6 else ''} "
                    f"in row(s) {', '.join(rows) or '?'}")
                if is_dated(rel) and not strict:
                    dated.append(f)
                else:
                    findings.append(f)
    return findings, dated, quoted, checked


def ledger_rows_count(root):
    text = read(os.path.join(root, LEDGER))
    return None if text is None else len(Ledger(text))


SELF_COUNTERS = {"ledger_rows": ledger_rows_count}


def check_selfcount(root, verbose):
    """C3 — a document stating the size of a table it itself contains.

    Heuristic by necessity, and the heuristic is PRINTED: every candidate the
    scanner considered is available under --verbose, so a false negative from a
    missing qualifier is visible rather than silent."""
    findings = []
    candidates = []
    checked = 0
    for path, noun, quals, counter in SELF_COUNTS:
        text = read(os.path.join(root, path))
        if text is None:
            continue
        actual = SELF_COUNTERS[counter](root)
        if actual is None:
            continue
        num = r"(\d+|" + "|".join(sorted(NUMBER_WORDS, key=len, reverse=True)) + r")"
        rx = re.compile(num + r"\s+(?:\w+\s+)?" + re.escape(noun) + r"\b",
                        re.IGNORECASE)
        for i, line in enumerate(text.split("\n"), 1):
            for m in rx.finditer(line):
                raw = m.group(1).lower()
                val = NUMBER_WORDS.get(raw)
                if val is None:
                    try:
                        val = int(raw)
                    except ValueError:
                        continue
                low = line.lower()
                lo = max(0, m.start() - SELF_COUNT_PROXIMITY)
                hi = min(len(line), m.end() + SELF_COUNT_PROXIMITY)
                near = low[lo:hi]
                qualified = any(q in near for q in quals)
                candidates.append((path, i, raw, val, qualified,
                                   line[lo:hi].strip()[:110]))
                if not qualified:
                    continue
                checked += 1
                if val != actual:
                    findings.append(Finding(
                        "C3", path, i,
                        f'says "{raw} {noun}" about its own table, which has '
                        f"{actual}"))
    return findings, checked, candidates


def check_bindings(root, files):
    """C4 — `COUNT[<recipe>] = <N>` must recount to N, and N must appear in the
    prose it annotates."""
    findings = []
    checked = 0
    oks = []
    for rel in files:
        if rel.startswith("work/") or rel in SELF_FIXTURE_FILES:
            continue
        text = read(os.path.join(root, rel))
        if text is None or "COUNT[" not in text:
            continue
        lines = text.split("\n")
        for i, line in enumerate(lines, 1):
            for m in BIND_RE.finditer(line):
                recipe, claimed = m.group(1), int(m.group(2))
                checked += 1
                got, err = run_recipe(root, recipe)
                if err:
                    findings.append(Finding(
                        "C4", rel, i,
                        f"binding `COUNT[{recipe}]` cannot be recounted: {err}"))
                    continue
                if got != claimed:
                    findings.append(Finding(
                        "C4", rel, i,
                        f"binding `COUNT[{recipe}]` claims {claimed}, "
                        f"recount says {got}"))
                    continue
                # C4b — the binding must be attached to prose that states the
                # same number, or it is a machine-readable claim floating free
                # of the human-readable one it is supposed to grade.
                window = "\n".join(lines[max(0, i - 6):i + 1])
                window = BIND_RE.sub("", window)
                if not re.search(r"(?<![\d.])" + str(claimed) + r"(?![\d.])",
                                 window) and not any(
                        w for w, v in NUMBER_WORDS.items()
                        if v == claimed and w in window.lower()):
                    findings.append(Finding(
                        "C4", rel, i,
                        f"binding `COUNT[{recipe}] = {claimed}` is DETACHED: "
                        f"the number {claimed} appears nowhere in the six "
                        f"lines of prose above it"))
                    continue
                oks.append((rel, i, recipe, claimed))
    return findings, checked, oks


def check_mentions(root, files):
    """C5 — a counted mark token in discussion context.

    Two counted surfaces, and they are counted by two different programs:
    `PROV[X]` in `crates/` by `provenance_census.py`, and `[R]`/`[O]`/`[I]` on
    the ten `P_*.md` subsystem pages by `subsys.rs::count_marks` (which counts
    LITERAL substrings after the page's first `---`, backticks and all).

    Reported as RISK, not as a defect: this tool does not own either counter,
    and #3641's own repair was to respell the prose, not to change the rule.
    """
    risks = []
    checked = 0
    for rel in files:
        if not rel.startswith("crates/") or rel in SELF_FIXTURE_FILES:
            continue
        text = read(os.path.join(root, rel))
        if text is None or "PROV" not in text:
            continue
        for i, line in enumerate(text.split("\n"), 1):
            for m in PROV_RE.finditer(line):
                checked += 1
                tok = m.group(0)
                low = line.lower()
                backticked = (
                    m.start() > 0 and line[m.start() - 1] == "`"
                    and m.end() < len(line) and line[m.end()] == "`")
                meta = [w for w in META_WORDS
                        if w in low and w != "prov["]
                if backticked or (meta and "PROV-BLOCK" not in tok
                                  and _looks_like_discussion(line)):
                    risks.append(Finding(
                        "C5", rel, i,
                        f"`{tok}` reads as a MENTION, not a mark "
                        f"({'backticked' if backticked else 'meta: ' + meta[0]})"
                        f" — provenance_census.py counts it either way"))
    # the P_*.md surface
    for page in SUBSYS_PAGES:
        rel = os.path.join("docs/whitebox/ref", page)
        text = read(os.path.join(root, rel))
        if text is None:
            continue
        body = _after_first_rule(text)
        if body is None:
            continue
        for i, line in enumerate(body.split("\n"), 1):
            for m in SUBSYS_MARK_RE.finditer(line):
                checked += 1
                backticked = (
                    m.start() > 0 and line[m.start() - 1] == "`"
                    and m.end() < len(line) and line[m.end()] == "`")
                if backticked:
                    risks.append(Finding(
                        "C5", rel, "?",
                        f"`{m.group(0)}` is backticked — a MENTION by the "
                        f"convention this tool proposes, and counted as an "
                        f"evidence mark by subsys.rs::count_marks today"))
    return risks, checked


def _looks_like_discussion(line):
    """A line that talks ABOUT markers rather than carrying one. Deliberately
    conservative: a real marker's own citation names a row, a doc or a rung,
    and reads as a statement about a value."""
    low = line.lower()
    return ("legend" in low or "misuse" in low or "mention" in low
            or "not a marker" in low or "count_marks" in low
            or "the census counts" in low)


def _after_first_rule(text):
    lines = text.split("\n")
    for i, l in enumerate(lines):
        if l.rstrip() == "---":
            return "\n".join(lines[i + 1:])
    return None


def check_adopted_paths(root, ledger):
    """C6 — every path in the `Adopted into` column resolves."""
    findings = []
    checked = 0
    for rid, p in ledger.adopted_into_paths():
        checked += 1
        if not os.path.exists(os.path.join(root, p)):
            findings.append(Finding(
                "C6", LEDGER, "-",
                f"row {rid} adopts into `{p}`, which does not exist"))
    return findings, checked


# ---------------------------------------------------------------------------
# I7 — the inventory of what nothing above can reach
# ---------------------------------------------------------------------------

NUM_IN_PROSE_RE = re.compile(r"(?<![\w.$/])(\d{1,6})(?![\w.])")


def inventory_unbound(root, files):
    """Count the numeric claims on provenance surfaces that no check reaches.

    A provenance surface is (a) the ledger, (b) `ref/README.md`, (c) any line
    in `crates/` carrying a `PROV[X]` marker. Hex addresses, board numbers,
    dates, section numbers and 0/1 are excluded — those are citations, and
    `doc_cite_audit.sh` / `board_audit.sh` already own them.
    """
    per_surface = {}
    samples = []
    bound_lines = set()

    for rel in files:
        text = read(os.path.join(root, rel))
        if text is None or "COUNT[" not in text:
            continue
        for i, line in enumerate(text.split("\n"), 1):
            if BIND_RE.search(line):
                bound_lines.add((rel, i))

    def scan_line(rel, i, line):
        if (rel, i) in bound_lines:
            return 0
        stripped = ADDR_RE.sub(" ", line)
        stripped = re.sub(r"#\d+", " ", stripped)          # board rows
        stripped = re.sub(r"§\s*[\d.]+", " ", stripped)    # section cites
        stripped = re.sub(r"\b20\d\d-\d\d-\d\d\b", " ", stripped)
        stripped = re.sub(r"`[0-9a-f]{7,}`", " ", stripped)  # shas
        stripped = re.sub(r"0x[0-9a-fA-F]+", " ", stripped)
        n = 0
        for m in NUM_IN_PROSE_RE.finditer(stripped):
            v = int(m.group(1))
            if v <= 1:
                continue
            n += 1
            if len(samples) < 12:
                samples.append((rel, i, v, line.strip()[:96]))
        return n

    for rel in PROV_DOC_SURFACES:
        text = read(os.path.join(root, rel))
        if text is None:
            continue
        tot = 0
        for i, line in enumerate(text.split("\n"), 1):
            tot += scan_line(rel, i, line)
        per_surface[rel] = tot

    marker_total = 0
    marker_lines = 0
    for rel in files:
        if not rel.startswith("crates/"):
            continue
        text = read(os.path.join(root, rel))
        if text is None or "PROV" not in text:
            continue
        for i, line in enumerate(text.split("\n"), 1):
            if not PROV_RE.search(line):
                continue
            marker_lines += 1
            marker_total += scan_line(rel, i, line)
    per_surface[f"crates/** PROV marker lines ({marker_lines})"] = marker_total
    return per_surface, samples


# ---------------------------------------------------------------------------
# driver
# ---------------------------------------------------------------------------

def audit(root, verbose=False, strict=False, quiet=False):
    text = read(os.path.join(root, LEDGER))
    if text is None:
        print(f"prose_audit: cannot read {LEDGER} under {root}", file=sys.stderr)
        return 2
    ledger = Ledger(text)
    files = walk_scan(root)

    out = []
    say = out.append

    say("prose audit — is the CLAIM inside a marker or a provenance doc TRUE?")
    say(f"root: {root}")
    say(f"ledger: {LEDGER} — {len(ledger)} adopted rows, "
        f"{len(ledger.addresses)} distinct addresses cited")
    say(f"scope: {len(files)} files under {', '.join(SCAN_ROOTS)} + "
        f"{', '.join(SCAN_FILES)}")
    say("")

    predrafts = predraft_index(root, files)
    f1, dated1, unattr, sup1, n1 = check_rowref(
        root, files, ledger, strict, predrafts)
    f2, dated2, quoted2, n2 = check_absence(root, files, ledger, strict)
    f3, n3, cands = check_selfcount(root, verbose)
    f4, n4, oks = check_bindings(root, files)
    f5, n5 = check_mentions(root, files)
    f6, n6 = check_adopted_paths(root, ledger)

    dated = dated1 + dated2
    findings = f1 + f2 + f3 + f4 + f6
    checked = n1 + n2 + n3 + n4 + n6

    say("CHECKS THAT CAN GO RED")
    hdr = f"{'id':<4} {'shape':<14} {'checked':>8} {'findings':>9}"
    say(hdr)
    say("-" * len(hdr))
    for cid, name, nn, ff in (("C1", "ROW-REF", n1, len(f1)),
                              ("C2", "ABSENCE", n2, len(f2)),
                              ("C3", "SELF-COUNT", n3, len(f3)),
                              ("C4", "BOUND-COUNT", n4, len(f4)),
                              ("C6", "ADOPTED-PATH", n6, len(f6))):
        say(f"{cid:<4} {name:<14} {nn:>8} {ff:>9}")
    say("-" * len(hdr))
    say(f"{'':<4} {'TOTAL':<14} {checked:>8} {len(findings):>9}")
    say("")
    say(f"C5   MENTION-RISK  {n5:>8} {len(f5):>9}   "
        f"(RISK, never red — this tool owns neither counter)")
    say("")

    if findings:
        say("FINDINGS — a claim that is FALSE, not a number that moved")
        for f in sorted(findings, key=lambda x: (x.check, x.path, str(x.line))):
            say(str(f))
        say("")

    if dated:
        say(f"IN A DATED RECORD — {len(dated)}, reported and NOT red")
        say("  A rung, a board row, a `WB_*_FINDINGS.md` grade page, or one of")
        say("  the ledger's own `>` amend-beside boxes is a record of the day it")
        say("  was written. This repo's standing rule is that they stay as")
        say("  written and corrections live in rows that CITE them")
        say("  (`ref/README.md` §2.1). `--strict` counts these as findings.")
        for f in sorted(dated, key=lambda x: (x.path, str(x.line)))[:30]:
            say(str(f))
        if len(dated) > 30:
            say(f"  … and {len(dated) - 30} more")
        say("")

    if unattr:
        by_tok = {}
        for f in unattr:
            tok = re.search(r"`(W-[A-Z0-9-]+)`", f.msg).group(1)
            by_tok.setdefault(tok, []).append(f)
        say(f"UNATTRIBUTED `W-*-N` TOKENS — {len(unattr)} citation(s), "
            f"{len(by_tok)} distinct token(s), reported and NOT red")
        say("  **The ledger's row-id grammar is not a reserved namespace.**")
        say("  These tokens match `W-<NAME>-<N>`, are defined as a row in no")
        say("  document, and are attributed to no ledger. They are a DIFFERENT")
        say("  namespace wearing the same spelling — and a grep-based reader")
        say("  cannot tell them apart, which is #3641's shape one level up.")
        for tok in sorted(by_tok):
            paths = sorted({f.path for f in by_tok[tok]})
            say(f"  {tok}: {len(by_tok[tok])} citation(s) in {len(paths)} file(s)"
                f" — e.g. {paths[0]}")
        say("")

    if f5:
        say(f"C5 MENTION-RISK — {len(f5)} site(s)")
        say("  #3641: a counter cannot tell an evidence mark from a mention of")
        say("  one, and writing prose ABOUT mark letters moved a subsystem's own")
        say("  agreement census 9/28 -> 13/34.")
        for f in f5[:40]:
            say(str(f))
        if len(f5) > 40:
            say(f"  … and {len(f5) - 40} more")
        say("")

    sp, sh, sf = sup1
    say("SUPPRESSION CLASSES — printed, never silent (doc_cite_audit.sh's rule)")
    say(f"  C1  declared a pre-draft on or beside its own line   {sp:>6}")
    say(f"  C1  qualified by the document that defines it        {sh:>6}")
    say(f"  C1  `W-FAKE-*`, the reserved planted-fixture space    {sf:>6}")
    say(f"  C2  the absence phrase is inside QUOTES (a mention)  {quoted2:>6}")
    say(f"  all self-fixture files excluded whole: "
        f"{', '.join(SELF_FIXTURE_FILES)}")
    if oks:
        say(f"C4 BINDINGS THAT AGREE: {len(oks)}")
        for rel, i, recipe, claimed in oks:
            say(f"  {rel}:{i}: COUNT[{recipe}] = {claimed}  OK")
    say("")

    if verbose and cands:
        say("C3 CANDIDATES CONSIDERED (qualified = the sentence names the table)")
        for path, i, raw, val, q, snippet in cands:
            say(f"  {path}:{i}: '{raw}' -> {val} "
                f"{'QUALIFIED' if q else 'skipped'}  {snippet}")
        say("")

    per_surface, samples = inventory_unbound(root, files)
    unbound = sum(per_surface.values())
    say("=" * 72)
    say("I7 — WHAT THIS AUDIT CANNOT SEE, AS A NUMBER")
    say("=" * 72)
    say("Numeric claims on provenance surfaces that NO check above can reach.")
    say("This is not a footnote. An audit whose coverage is unstated will be")
    say("read as total, and the default state of a prose number in this tree is")
    say("UNREACHABLE — it becomes checkable only when a human binds it with a")
    say("COUNT[...] recipe or it happens to fall into C1/C2/C3/C6's shapes.")
    say("")
    for k in sorted(per_surface):
        say(f"  {k:<52} {per_surface[k]:>6}")
    say(f"  {'TOTAL UNBOUND':<52} {unbound:>6}")
    say(f"  {'checkable (C1+C2+C3+C4+C6 above)':<52} {checked:>6}")
    if checked:
        say(f"  {'ratio unbound : checkable':<52} {unbound / checked:>5.1f}:1")
    say("")
    say("  Also outside scope, by kind rather than by count:")
    say("   * any claim of fact about c2's behaviour — the image and the byte")
    say("     judge grade that, never this tool;")
    say("   * whether a cited ADDRESS holds the instruction claimed (that is")
    say("     addrcheck.py's question, deliberately not folded in);")
    say("   * whether a bound count is bound to the RIGHT population — C4")
    say("     checks the arithmetic of a binding a human wrote;")
    say("   * `file.md:NNN` line-citation staleness — doc_cite_audit.sh's own")
    say("     stated LIMIT, not duplicated here.")
    if samples:
        say("")
        say("  A sample of the unbound residue, so it is not an abstraction:")
        for rel, i, v, snippet in samples:
            say(f"    {rel}:{i}: {v} — {snippet}")
    say("")

    if checked == 0:
        say("NOTHING WAS CHECKED — a checker with no subject is decoration.")
        if not quiet:
            print("\n".join(out))
        return 3

    say(f"VERDICT: {'FINDINGS ' + str(len(findings)) if findings else 'CLEAN'} "
        f"over {checked} checked claims "
        f"({len(dated)} in dated records, {len(unattr)} unattributed, "
        f"{len(f5)} mention-risks)")
    if not quiet:
        print("\n".join(out))
    return 1 if findings else 0


# ---------------------------------------------------------------------------
# self-test — planted fixtures, and it must be watched RED and GREEN
# ---------------------------------------------------------------------------

TRUE_LEDGER = """\
# DISCLOSURE — planted fixture

| # | Kind | What was adopted | Address in `c2.dll` | Adopted into | Commit | Notes |
|---|---|---|---|---|---|---|
| **W-PLANT-1** | **adoption** | a made-up value, `a \\| b` | **`0x10b00001`** | `crates/planted/real.rs` — `A` | x | n |
| **W-PLANT-2** | **route** | a second made-up value | **`0x10b00002`** | `crates/planted/real.rs` — `B` | x | n |
| **W-PLANT-3** | **adoption** | a third | **`0x10b00003`** | `crates/planted/real.rs` — `C` | x | n |

The table above is exhaustive at three rows.
"""

# The FALSE variant: same three rows, and the prose says four.
FALSE_LEDGER = TRUE_LEDGER.replace(
    "The table above is exhaustive at three rows.",
    "The table above is exhaustive at four rows.")

TRUE_SRC = """\
//! planted source, every claim TRUE.

/// PROV[R] DISCLOSURE W-PLANT-1 — the first made-up value, `0x10b00001`.
pub const A: u32 = 1;

/// PROV[R] DISCLOSURE W-PLANT-2 — the second, `0x10b00002`.
pub const B: u32 = 2;

/// PROV[R] DISCLOSURE W-PLANT-3 — the third, `0x10b00003`.
pub const C: u32 = 3;

/// This module declares three constants.  COUNT[rs-consts:crates/planted/real.rs] = 3
/// and the ledger it cites has three rows.  COUNT[ledger-rows] = 3
pub fn note() {}
"""

FALSE_SRC = """\
//! planted source, four claims FALSE and one honest.

/// PROV[R] DISCLOSURE W-PLANT-9 — C1 must fire on this citation.
pub const A: u32 = 1;

/// PROV[R] `0x10b00002` — NO DISCLOSURE ROW EXISTS FOR THIS ADDRESS. C2 must
/// fire, because W-PLANT-2 cites exactly this address.
pub const B: u32 = 2;

/// PROV[R] DISCLOSURE W-PLANT-3 — honest, and C1 must NOT fire on it.
pub const C: u32 = 3;

/// This module declares nine constants.  COUNT[rs-consts:crates/planted/real.rs] = 9
pub const D: u32 = 4;

/// A pre-draft cited honestly: W-PLANT-77 is a pre-draft and is not carried.
pub const E: u32 = 5;
"""

# The suppression fixtures. **Every suppression class gets a control**, because
# a suppression class is a hole by construction and the only question is whether
# it is the hole you meant. Each block below must be suppressed, and the
# NEIGHBOURING line in the same block must still fire — so a suppression that
# swallowed its whole file would be caught.
#
# NOTE ON THE WORDING, and it is the second time this bit me: the fixture's own
# EXPLANATORY prose must not contain a suppressing phrase, or the control
# suppresses itself and goes green. Section [2]'s first draft did exactly that
# with *"a row that does not exist"*, and this block's first draft did it again
# with *"the token is a real pre-draft"* sitting one line under the citation it
# was supposed to leave alone. Explanations live in the section text below, not
# in the fixture.
SUPPRESSION_SRC = """\
//! Four suppressions, each beside a live finding it must NOT swallow.

/// PROV[R] OTHER_FINDINGS.md W-OTHER-1 — qualified by its home document.
pub const B: u32 = 2;

/// PROV[R] DISCLOSURE W-OTHER-1 — attributed to the ledger.
pub const C: u32 = 3;

/// The stale marker was quoted as saying "NO DISCLOSURE ROW EXISTS FOR THIS
/// ADDRESS", about `0x10b00001`, and that sentence is corrected below.
pub const D: u32 = 4;

/// PROV[R] `0x10b00003` — NO DISCLOSURE ROW EXISTS FOR THIS ADDRESS.
pub const E: u32 = 5;

/// W-NAMESPACE-7 names a fixture family, and belongs to no ledger.
pub const F: u32 = 6;
"""

OTHER_FINDINGS = """\
# OTHER_FINDINGS — a planted pre-draft table

| # | Kind | What | Address | Notes |
|---|---|---|---|---|
| **W-OTHER-1** | **adoption** | a pre-drafted value | `0x10b00009` | not carried |
"""

DETACHED_SRC = """\
//! the binding is arithmetically right and attached to nothing.

/// Some prose that states no number whatsoever.
/// COUNT[ledger-rows] = 3
pub const A: u32 = 1;
"""


def _plant(base, ledger_text, src_text, extra=None):
    os.makedirs(os.path.join(base, "docs", "whitebox"), exist_ok=True)
    os.makedirs(os.path.join(base, "crates", "planted"), exist_ok=True)
    with open(os.path.join(base, LEDGER), "w", encoding="utf-8") as fh:
        fh.write(ledger_text)
    with open(os.path.join(base, "crates", "planted", "real.rs"),
              "w", encoding="utf-8") as fh:
        fh.write(src_text)
    if extra:
        for rel, text in extra.items():
            p = os.path.join(base, rel)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, "w", encoding="utf-8") as fh:
                fh.write(text)


def self_test():
    ok = True

    def check(label, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'PASS' if good else 'FAIL'}  {label}: got {got}, want {want}")

    print("[1] THE GREEN — every planted claim is TRUE, the audit must exit 0")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, TRUE_LEDGER, TRUE_SRC)
        rc = audit(td, quiet=True)
        check("exit code", rc, 0)
        # and it must have actually checked something — a green from a scanner
        # that matched nothing is the failure mode #1002 names.
        text = read(os.path.join(td, LEDGER))
        led = Ledger(text)
        check("the fixture ledger parsed to three rows", len(led), 3)
        check("and its three addresses were collected",
              len(led.addresses), 3)

    print()
    print("[2] THE RED — C1, a citation to a row that does not exist")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, TRUE_LEDGER, FALSE_SRC)
        files = walk_scan(td)
        led = Ledger(read(os.path.join(td, LEDGER)))
        pre = predraft_index(td, files)
        f1, dated, unattr, sup, n1 = check_rowref(td, files, led, False, pre)
        check("one C1 finding", len(f1), 1)
        check("and it names the fake row",
              "W-PLANT-9" in (f1[0].msg if f1 else ""), True)
        check("the honest pre-draft line is SUPPRESSED, not reported", sup[0], 1)
        check("and the suppression is counted, never silent", sup[0] > 0, True)

        print()
        print("[3] THE RED — C2, an absence claim the ledger falsifies")
        f2, dated2, quoted2, n2 = check_absence(td, files, led, False)
        check("one C2 finding", len(f2), 1)
        check("and it names the row that falsifies it",
              "W-PLANT-2" in (f2[0].msg if f2 else ""), True)

        print()
        print("[4] THE RED — C4, a binding whose recount disagrees")
        f4, n4, oks = check_bindings(td, files)
        check("one C4 finding", len(f4), 1)
        check("recount is reported, not just the disagreement",
              "recount says" in (f4[0].msg if f4 else ""), True)

        print()
        print("[5] the whole audit exits 1 on this tree")
        rc = audit(td, quiet=True)
        check("exit code", rc, 1)

    print()
    print("[6] THE RED — C3, a document that miscounts its OWN table")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, FALSE_LEDGER, TRUE_SRC)
        f3, n3, cands = check_selfcount(td, False)
        check("one C3 finding", len(f3), 1)
        check("it says what the table actually has",
              "which has 3" in (f3[0].msg if f3 else ""), True)
        check("candidates were considered, so a miss is visible",
              len(cands) >= 1, True)

    print()
    print("[7] THE RED — C4b, a binding that is arithmetically RIGHT and")
    print("    attached to no prose. This is the hole a count-checker has:")
    print("    a machine-readable claim floating free of the human one.")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, TRUE_LEDGER, DETACHED_SRC)
        files = walk_scan(td)
        f4, n4, oks = check_bindings(td, files)
        check("one C4 finding", len(f4), 1)
        check("and it is the DETACHED kind, not an arithmetic one",
              "DETACHED" in (f4[0].msg if f4 else ""), True)

    print()
    print("[8] THE RED — C6, a ledger row adopting into a path that is gone")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, TRUE_LEDGER.replace("real.rs` — `C`", "vanished.rs` — `C`"),
               TRUE_SRC)
        led = Ledger(read(os.path.join(td, LEDGER)))
        f6, n6 = check_adopted_paths(td, led)
        check("three paths were checked", n6, 3)
        check("one C6 finding", len(f6), 1)
        check("and it names the row", "W-PLANT-3" in (f6[0].msg if f6 else ""),
              True)

    print()
    print("[9] THE ESCAPED PIPE — the ledger's own cells contain `\\|`, and a")
    print("    naive split shears the `Adopted into` column. C6 would then")
    print("    check half a path and pass.")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, TRUE_LEDGER, TRUE_SRC)
        led = Ledger(read(os.path.join(td, LEDGER)))
        paths = dict(led.adopted_into_paths())
        check("W-PLANT-1's path survived the escaped pipe in its own cell",
              paths.get("W-PLANT-1"), "crates/planted/real.rs")
        naive = [c.strip() for c in
                 TRUE_LEDGER.split("\n")[4].split("|")]
        check("and a naive split really would have got it wrong",
              naive[5].startswith("crates/"), False)

    print()
    print("[10] NOTHING TO CHECK is exit 3, not exit 0 — absence is not success")
    with tempfile.TemporaryDirectory() as td:
        os.makedirs(os.path.join(td, "docs", "whitebox"))
        with open(os.path.join(td, LEDGER), "w", encoding="utf-8") as fh:
            fh.write("# DISCLOSURE — no rows at all\n")
        rc = audit(td, quiet=True)
        check("exit code", rc, 3)

    print()
    print("[11] A MISSING LEDGER is exit 2, never a crash and never a green")
    with tempfile.TemporaryDirectory() as td:
        rc = audit(td, quiet=True)
        check("exit code", rc, 2)

    print()
    print("[12] THE MARKER GRAMMAR IS THE CENSUS'S — proved by BEHAVIOUR on a")
    print("     planted string, not by comparing two regex sources")
    probe = ("PROV[R] a real marker\n"
             "params[src] is an array index, not a marker\n"
             "a bare [R] in prose is not a marker either\n"
             "PROV-BLOCK[O] is a block marker\n")
    mine = sorted(m.group(1) for m in PROV_RE.finditer(probe))
    check("this tool sees exactly the two markers", mine, ["O", "R"])
    here = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, here)
    try:
        import provenance_census as pc
        theirs = sorted(
            [m.group(1) for m in pc.MARK_RE.finditer(probe)]
            + [m.group(1) for m in pc.BLOCK_RE.finditer(probe)])
        check("and the census sees the same two", theirs, ["O", "R"])
    except ImportError:
        print("  SKIP  provenance_census.py not importable from here")
    finally:
        sys.path.pop(0)

    print()
    print("[13] EVERY SUPPRESSION CLASS GETS A CONTROL — a suppression is a")
    print("     hole by construction, and the only question is whether it is")
    print("     the hole you meant. Each must fire on its neighbour.")
    with tempfile.TemporaryDirectory() as td:
        _plant(td, TRUE_LEDGER, SUPPRESSION_SRC,
               extra={"docs/whitebox/OTHER_FINDINGS.md": OTHER_FINDINGS})
        files = walk_scan(td)
        led = Ledger(read(os.path.join(td, LEDGER)))
        pre = predraft_index(td, files)
        check("the pre-draft index found W-OTHER-1",
              sorted(pre.get("W-OTHER-1", ())), ["OTHER_FINDINGS.md"])
        f1, d1, unattr, sup, n1 = check_rowref(td, files, led, False, pre)
        sp, sh, sf = sup
        check("the home-document citation is suppressed", sh, 1)
        check("W-NAMESPACE-7 lands in UNATTRIBUTED, not findings",
              [f.msg.split("`")[1] for f in unattr], ["W-NAMESPACE-7"])
        live = [f for f in f1 if "attributes" in f.msg]
        check("the DISCLOSURE-attributed pre-draft still FIRES", len(live), 1)
        check("and the finding says where the pre-draft actually lives",
              "OTHER_FINDINGS.md" in (live[0].msg if live else ""), True)

        f2, d2, quoted, n2 = check_absence(td, files, led, False)
        check("the QUOTED absence claim is suppressed", quoted, 1)
        check("the unquoted one beside it still FIRES", len(f2), 1)
        check("and it names the row that falsifies it",
              "W-PLANT-3" in (f2[0].msg if f2 else ""), True)

    print()
    print("[14] THE SUPPRESSION MUST NOT SWALLOW THE FILE — take the two")
    print("     suppressing devices away and the SAME file goes red twice as")
    print("     often. A suppression that survives this is over-wide.")
    with tempfile.TemporaryDirectory() as td:
        stripped = (SUPPRESSION_SRC
                    .replace("OTHER_FINDINGS.md W-OTHER-1",
                             "DISCLOSURE W-OTHER-1")
                    .replace('quoted as saying "NO DISCLOSURE ROW EXISTS FOR '
                             'THIS',
                             "recorded as NO DISCLOSURE ROW EXISTS FOR THIS"))
        assert stripped != SUPPRESSION_SRC, "the mutation did not apply"
        _plant(td, TRUE_LEDGER, stripped,
               extra={"docs/whitebox/OTHER_FINDINGS.md": OTHER_FINDINGS})
        files = walk_scan(td)
        led = Ledger(read(os.path.join(td, LEDGER)))
        pre = predraft_index(td, files)
        f1, _d, _u, sup, _n = check_rowref(td, files, led, False, pre)
        check("the home-document suppression is now empty", sup[1], 0)
        check("and both attributed citations fire", len(f1), 2)
        f2, _d2, quoted, _n2 = check_absence(td, files, led, False)
        check("nothing is suppressed as quoted any more", quoted, 0)
        check("and both absence claims fire", len(f2), 2)

    print()
    print("[15] ATTRIBUTION OUTRANKS SUPPRESSION — the exact shape of #3645.")
    print("     One line names the home document AND falsely attributes the")
    print("     token to the ledger. Suppression-first loses the finding.")
    with tempfile.TemporaryDirectory() as td:
        src = ("//! the #3645 line, reproduced\n\n"
               "/// TYPE word (OTHER_FINDINGS.md §3.2 / DISCLOSURE W-OTHER-1)\n"
               "pub const A: u32 = 1;\n")
        _plant(td, TRUE_LEDGER, src,
               extra={"docs/whitebox/OTHER_FINDINGS.md": OTHER_FINDINGS})
        files = walk_scan(td)
        led = Ledger(read(os.path.join(td, LEDGER)))
        pre = predraft_index(td, files)
        f1, _d, _u, sup, _n = check_rowref(td, files, led, False, pre)
        check("the false attribution FIRES despite the home-doc mention",
              len(f1), 1)
        check("and nothing was suppressed as home-qualified", sup[1], 0)
        # the adjacency half: a DISCLOSURE far from the token must NOT attribute
        far = ("//! DISCLOSURE.md is discussed here at length, and separately "
               "this line ends by naming W-OTHER-1 in OTHER_FINDINGS.md\n"
               "pub const B: u32 = 2;\n")
        check("a DISCLOSURE far from the token does not attribute",
              _attributes_to_ledger(far, "W-OTHER-1"), False)
        check("and an adjacent one does",
              _attributes_to_ledger("see DISCLOSURE W-OTHER-1 for this",
                                    "W-OTHER-1"), True)

    print()
    print("SELF-TEST:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def main(argv):
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(here)
    args = argv[1:]
    if "--self-test" in args:
        return self_test()
    verbose = "--verbose" in args
    strict = "--strict" in args
    rest = [a for a in args if a not in ("--verbose", "--strict")]
    if rest:
        root = os.path.abspath(rest[0])
    return audit(root, verbose=verbose, strict=strict)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
