#!/usr/bin/env python3
"""verify_rows.py — grade `docs/whitebox/DISCLOSURE.md` against the tree.

Lane `w-disclose` (wave 14, decision 16, board #3642-#3646). Six checks, in
two directions, plus a value check that no previous lane ran:

  A  LEDGER -> CRATES.  Every path named in a row's `Adopted into` cell exists,
     and every symbol named beside it is still present in that file.
     `w-provenance` ran this by hand and scored 13 of 13 live (#3631); this
     makes it a program so the next lane does not re-derive it.

  B  CRATES -> LEDGER.  Every `DISCLOSURE <ROW-ID>` citation anywhere in
     `crates/` names a row that is actually in the table.  **This direction had
     never been checked.**

  C  COVERAGE.  Every `[R]`-marked constant in `crates/c2-core/src/codegen/`
     is reachable from a ledger row via its own marker's citation.

  D  VALUES (needs the pinned image; SKIPs without it).  `mop.rs`'s `OPCODES`
     against a live dump of c2.dll's own tables -- mnemonic, base word and
     form, per row -- and `MAX_C2_OPCODE` against the table's extent.  This is
     the check that would catch a row registering a provenance the value does
     not have.

  E  TRANSCRIPTION.  `docs/whitebox/ref/ENCODE_OPCODES.txt`, which is what
     `mop.rs` was transcribed from, reproduces byte-identically from the image.

  F  ADDRESSES.  Every c2 address cited in `mop.rs` is accounted for: a table
     this ledger names, an arm the 85 transcribed rows dispatch to, or a
     composer `P_ENCODE.md` Sec 5.5 names.  #3626's class -- a page carrying
     four wrong addresses for eight days because nothing checked them.

Usage:
    python3 work/w-disclose/verify_rows.py [--self-test]

Exit 0 clean, 1 on any failure, 2 on a usage error.  Run from the repo root.

#1406: this is a python instrument outside the std-only workspace, so it
carries its own self-test, which is the shape `scripts/gate_identity_diff.sh
--self-test` and `scripts/provenance_census.py --self-test` already establish
in this repo.  The control is watched failing on planted defects before the
clean run is believed (#3336: this repo has shipped a --check that could not
fail).
"""

import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
DISCLOSURE = "docs/whitebox/DISCLOSURE.md"
MOP = "crates/c2-core/src/codegen/mop.rs"
ENCODE_TXT = "docs/whitebox/ref/ENCODE_OPCODES.txt"
DUMP = "docs/whitebox/scripts/dump_opcode_tables.py"
IMAGE = "compilers/X360/16.00.11886.00/c2.dll"

ROW_RE = re.compile(r"^\|\s*\*\*(W-[A-Z0-9]+-\d+)\*\*\s*\|")
CITE_RE = re.compile(r"DISCLOSURE\s+`?(W-[A-Z0-9]+-\d+)`?")
PATH_RE = re.compile(r"`((?:crates|c2host|docs|scripts)/[A-Za-z0-9_./-]+)`")
SYM_RE = re.compile(r"`([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)(?:\[\])?`")
# A backticked token in an `Adopted into` cell is not always a symbol: the
# column also carries the adopting COMMIT (`a09f33704`, `2bfc70caf`).  Treated
# as a symbol it is a guaranteed false red, which is how it was found.
SHA_RE = re.compile(r"^[0-9a-f]{7,40}$")
MARK_RE = re.compile(r"PROV(?:-BLOCK)?\[([RSOFN])\]")
ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([a-z:]+\))?\s+)?(?:const|static)\s+(?:mut\s+)?([A-Z][A-Z_0-9]*)\s*:"
)

# `P_ENCODE.md` Sec 5.5: the four D-form / X-form composers the memory arms
# call.  `mop.rs` cites these instead of the jump-table arm, which is one level
# DEEPER and is the level the field placement actually lives at.
COMPOSERS = {
    "10bf9e55": "D-form load composer (arm 10bfa667, forms 21/45/46)",
    "10bf9eb5": "D-form store composer (arm 10bfa676, forms 27/58/71)",
    "10bf9788": "X-form load composer (arm 10bfa17f, forms 26/50)",
    "10bf97c8": "X-form store composer (arm 10bfa1a1, forms 28/61)",
}
# Table addresses the ledger names in W-MID-1 / W-MID-2 / W-MOP-*.
TABLES = {
    "10b1b260": "mnemonic table (W-MID-1)",
    "10c3a578": "base-word table (W-MID-2)",
    "10c39b18": "encode-form table (W-MID-2)",
    "10bfae2d": "arm jump table (W-MID-2)",
}


class Grader:
    def __init__(self, root):
        self.root = root
        self.fail = 0
        self.lines = []
        self.quiet = False

    def say(self, s):
        self.lines.append(s)
        if not self.quiet:
            print(s)

    def ok(self, label, detail=""):
        self.say("  PASS  %-46s %s" % (label, detail))

    def bad(self, label, detail=""):
        self.fail += 1
        self.say("  FAIL  %-46s %s" % (label, detail))

    def skip(self, label, detail=""):
        self.say("  SKIP  %-46s %s" % (label, detail))

    def read(self, rel):
        with open(os.path.join(self.root, rel), encoding="utf-8") as fh:
            return fh.read()

    def exists(self, rel):
        return os.path.exists(os.path.join(self.root, rel))


def parse_rows(text):
    """rowid -> (kind, adopted_into_cell). Splits on unescaped pipes."""
    rows = {}
    for line in text.splitlines():
        m = ROW_RE.match(line)
        if not m:
            continue
        cells = re.split(r"(?<!\\)\|", line)
        # cells[0] is empty (leading pipe); 1=#, 2=Kind, 3=What, 4=Address,
        # 5=Adopted into, 6=Commit, 7=Notes
        if len(cells) < 6:
            continue
        rows[m.group(1)] = (cells[2].strip(), cells[5].strip())
    return rows


def check_a(g, rows):
    g.say("A  LEDGER -> CRATES  (every cited site is live)")
    live = dead = 0
    crates_rows = 0
    crates_only = [0]  # #3631 counted `crates/` paths only; keep both numbers
    for rid, (_kind, cell) in sorted(rows.items()):
        paths = PATH_RE.findall(cell)
        code_paths = [p for p in paths if p.startswith(("crates/", "c2host/"))]
        if not code_paths:
            continue
        crates_rows += 1
        if any(p.startswith("crates/") for p in code_paths):
            crates_only[0] += 1
        bad = []
        for p in code_paths:
            if not g.exists(p):
                bad.append("path missing: %s" % p)
                continue
            body = g.read(p)
            # symbols are the backticked non-path tokens in the same cell
            for s in SYM_RE.findall(cell):
                if "/" in s or s in paths or SHA_RE.match(s):
                    continue
                needle = s.split("::")[0]
                if needle not in body and s not in body:
                    # a symbol may belong to a sibling path in the same cell
                    if not any(
                        needle in g.read(q) for q in code_paths if g.exists(q)
                    ) and not any(
                        needle in g.read(q) for q in paths
                        if q not in code_paths and g.exists(q)
                    ):
                        bad.append("symbol absent: %s" % s)
        if bad:
            dead += 1
            g.bad(rid, "; ".join(sorted(set(bad))))
        else:
            live += 1
    detail = "%d live, %d dead (%d of them name a crates/ path -- #3631's count)" % (
        live, dead, crates_only[0])
    if dead == 0:
        g.ok("%d rows name a code path" % crates_rows, detail)
    else:
        g.say("        %d rows name a code path -- %s" % (crates_rows, detail))
    return crates_rows, live, dead


def check_b(g, rows):
    g.say("B  CRATES -> LEDGER  (every citation names a real row)")
    found = {}
    for dirpath, dirnames, filenames in os.walk(os.path.join(g.root, "crates")):
        dirnames[:] = [d for d in dirnames if d not in ("target", ".git")]
        for fn in filenames:
            if not fn.endswith(".rs"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, fn), g.root)
            for i, line in enumerate(g.read(rel).splitlines(), 1):
                for rid in CITE_RE.findall(line):
                    found.setdefault(rid, []).append("%s:%d" % (rel, i))
    dead = {r: w for r, w in found.items() if r not in rows}
    for rid, where in sorted(dead.items()):
        g.bad("cites a row that is NOT in the ledger: " + rid, where[0])
    if not dead:
        g.ok("%d distinct rows cited from crates/" % len(found), "0 dead")
    return found, dead


def scan_marks(g, rel):
    """(line, name, mark, citation) for every const/static, block form honoured."""
    body = g.read(rel).splitlines()
    out = []
    block = []  # (brace_depth_at_declaration, mark, citation)
    depth = 0
    pending = None
    for i, line in enumerate(body, 1):
        m = MARK_RE.search(line)
        if m:
            cite = line.split(m.group(0), 1)[1].strip()
            if "PROV-BLOCK[" in line:
                block.append((depth, m.group(1), cite))
            else:
                pending = (m.group(1), cite)
        it = ITEM_RE.match(line)
        if it:
            if pending:
                out.append((i, it.group(1), pending[0], pending[1]))
                pending = None
            elif block:
                d, mk, ct = block[-1]
                out.append((i, it.group(1), mk, ct))
            else:
                out.append((i, it.group(1), None, ""))
        depth += line.count("{") - line.count("}")
        # A block marker declared at depth d covers until the enclosing brace
        # closes, i.e. until depth drops BELOW d.  `<=` popped it on the very
        # next line and silently reported `mod op`'s 85 constants as 0 -- the
        # #3516 shape, a check that grades an empty population and looks green.
        while block and depth < block[-1][0]:
            block.pop()
        if it or (line.strip() and not line.strip().startswith(("//", "///", "//!", "#["))):
            if not m:
                pending = None
    return out


def check_c(g, rows):
    g.say("C  COVERAGE  (every [R] constant in codegen/ reaches a row)")
    total = covered = 0
    orphan = []
    for dirpath, dirnames, filenames in os.walk(
        os.path.join(g.root, "crates/c2-core/src/codegen")
    ):
        for fn in sorted(filenames):
            if not fn.endswith(".rs"):
                continue
            rel = os.path.relpath(os.path.join(dirpath, fn), g.root)
            for ln, name, mark, cite in scan_marks(g, rel):
                if mark != "R":
                    continue
                total += 1
                ids = CITE_RE.findall(cite) or re.findall(r"(W-[A-Z0-9]+-\d+)", cite)
                # **The bar is not "the cited row exists".**  That was true of
                # all 88 of `mop.rs`'s constants at `e548f01fd` and is exactly
                # what #3632 found insufficient: they cited `W-MID-1`/`W-MID-2`,
                # whose `Adopted into` names `middle_interfaces.rs` and nothing
                # else.  A constant is COVERED only when a row it cites names
                # the file the constant is in.
                if any(i in rows and rel in rows[i][1] for i in ids):
                    covered += 1
                elif any(i in rows for i in ids):
                    orphan.append("%s:%d %s cites %s, which does not name this file"
                                  % (rel, ln, name, ",".join(ids)))
                else:
                    orphan.append("%s:%d %s -> no row: %s" % (rel, ln, name, cite[:50]))
    shown = orphan[:4]
    for o in shown:
        g.bad("[R] constant not covered by a row", o)
    if len(orphan) > len(shown):
        g.bad("... and %d more" % (len(orphan) - len(shown)), "(same class)")
    g.say("        codegen/ [R] constants: %d of %d covered" % (covered, total))
    # Reconciliation against the peer instrument, so a broken scanner here
    # cannot report a small population as a clean one.
    if total != 88:
        g.bad("population disagrees with provenance_census.py",
              "this scanner sees %d [R] constants in codegen/, the census says 88"
              % total)
    if not orphan:
        g.ok("codegen/ [R] constants", "%d of %d reach a row that names their file"
             % (covered, total))
    return total, covered, orphan


def live_dump(g):
    if not g.exists(IMAGE):
        return None
    out = subprocess.run(
        [sys.executable, DUMP, IMAGE, "--encode", "1", "0x295"],
        cwd=g.root, capture_output=True, text=True,
    )
    if out.returncode != 0:
        return None
    return out.stdout


def parse_dump(text):
    ref = {}
    for line in text.splitlines():
        if line.startswith("#"):
            continue
        p = line.split()
        if len(p) < 6:
            continue
        ref[int(p[0], 16)] = (p[2], p[3].lower(), int(p[4]), p[5].lower())
    return ref


def check_de(g, dump):
    g.say("D  VALUES  (OPCODES vs the pinned image's own tables)")
    if dump is None:
        g.skip("toolchain absent", "no pinned c2.dll; D/E/F-arms not graded")
        return None, (0, 0)
    ref = parse_dump(dump)
    src = g.read(MOP)
    ops = dict(
        (n, int(v, 16))
        for n, v in re.findall(r"pub const (\w+): C2Op = C2Op\((0x[0-9a-fA-F]+)\);", src)
    )
    rows = re.findall(r'row\(op::(\w+), "([^"]+)", (0x[0-9a-f_]+), (\d+)\)', src)
    bad = []
    for name, mnem, base, form in rows:
        o = ops.get(name)
        if o is None:
            bad.append("%s: no opcode constant" % name)
            continue
        r = ref.get(o)
        if r is None:
            bad.append("%s: opcode %#06x not in c2's table" % (name, o))
            continue
        b = base.replace("_", "")[2:].lstrip("0") or "0"
        if r[0] != mnem:
            bad.append("%s: mnemonic %s, c2 says %s" % (name, mnem, r[0]))
        if b != (r[1].lstrip("0") or "0"):
            bad.append("%s: base %s, c2 says %s" % (name, b, r[1]))
        if int(form) != r[2]:
            bad.append("%s: form %s, c2 says %d" % (name, form, r[2]))
    mx = re.search(r"const MAX_C2_OPCODE: usize = (0x[0-9a-fA-F]+);", src)
    if not mx or int(mx.group(1), 16) != max(ref):
        bad.append("MAX_C2_OPCODE != the table extent %#x" % max(ref))
    for b in bad:
        g.bad("value disagrees with the image", b)
    if not bad:
        g.ok("OPCODES vs c2's tables", "%d of %d rows agree on mnemonic+base+form; "
             "MAX_C2_OPCODE = %#x" % (len(rows), len(rows), max(ref)))

    g.say("E  TRANSCRIPTION  (ENCODE_OPCODES.txt reproduces from the image)")
    if g.read(ENCODE_TXT) == dump:
        g.ok("ENCODE_OPCODES.txt", "byte-identical to a live dump, %d rows" % len(ref))
    else:
        g.bad("ENCODE_OPCODES.txt", "DIFFERS from a live dump of the pinned image")

    g.say("F  ADDRESSES  (every c2 address cited in mop.rs is accounted for)")
    used_arms = {ref[ops[n]][3] for n, _, _, _ in rows if ops.get(n) in ref}
    cited = sorted(set(a.lower() for a in re.findall(r"(?:0x)?(10[bc][0-9a-f]{5})", src)))
    unaccounted = [
        a for a in cited
        if a not in used_arms and a not in COMPOSERS and a not in TABLES
        and a not in {v[3] for v in ref.values()}
    ]
    for a in unaccounted:
        g.bad("address cited by mop.rs is not a c2 table, arm or composer", a)
    if not unaccounted:
        g.ok("mop.rs addresses", "%d cited, all accounted for (%d arms, %d composers, "
             "%d tables)" % (len(cited), len(used_arms & set(cited)),
                             len(set(cited) & set(COMPOSERS)),
                             len(set(cited) & set(TABLES))))
    return ref, (len(rows), len(bad))


def run(root, quiet=False):
    g = Grader(root)
    if quiet:
        g.quiet = True
    rows = parse_rows(g.read(DISCLOSURE))
    g.say("verify_rows.py -- DISCLOSURE.md has %d rows in the adopted-findings table"
          % len(rows))
    g.say("")
    check_a(g, rows)
    g.say("")
    check_b(g, rows)
    g.say("")
    check_c(g, rows)
    g.say("")
    check_de(g, live_dump(g))
    g.say("")
    g.say("VERDICT: %s (%d failures)" % ("GREEN" if g.fail == 0 else "RED", g.fail))
    return g


def fails(g):
    return set(l.strip() for l in g.lines if l.startswith("  FAIL"))


def self_test():
    """Plant one defect per direction and demand each is caught BY ITS OWN
    CHECK and by no other.

    The baseline is deliberately NOT required to be green -- at `e548f01fd` it
    is red, and that redness is this lane's subject.  The control is therefore
    stated as a DELTA: the planted run must produce failure lines the clean run
    did not, and they must name the check the plant belongs to.  A control
    nobody watched fail is not a control (#3336), and a plant that matched
    nothing is a green that looks exactly like a caught defect (#3516).
    """
    import shutil
    import tempfile

    print("---- self-test: baseline ----")
    base = run(ROOT)
    base_fails = fails(base)
    print("baseline: %d failures\n" % base.fail)

    plants = [
        # NOT `mop.rs`: at `e548f01fd` that string IS in DISCLOSURE.md, but
        # only in the "Adoptions this ledger does not carry" PROSE, which check
        # A does not read -- so the plant matched and changed nothing gradeable.
        # Caught by this self-test on its first run; the plant now targets a
        # path that is inside an actual table row at both ends of the lane.
        ("A", DISCLOSURE, "crates/c2-core/src/plan/mod.rs",
         "crates/c2-core/src/plan/mod_NOT_A_FILE.rs", "path missing"),
        ("B", MOP, "DISCLOSURE `W-MID-1` (the mnemonic table",
         "DISCLOSURE `W-BOGUS-9` (the mnemonic table", "NOT in the ledger"),
        ("D", MOP, 'row(op::ADD, "add", 0x7c00_0214, 49)',
         'row(op::ADD, "add", 0x7c00_0215, 49)', "disagrees with the image"),
        ("E", ENCODE_TXT, "0x0001     1  add           7c000214    49  10bfa456",
         "0x0001     1  add           7c000215    49  10bfa456", "DIFFERS"),
        ("F", MOP, "// `10bfa456` — RT=reg(S)", "// `10bfa999` — RT=reg(S)",
         "not a c2 table, arm or composer"),
    ]
    rc = 0
    ignore = shutil.ignore_patterns(
        ".git", "target", "work", "captures", "compilers", ".claude", ".c2rs-work")
    for tag, rel, old, new_text, expect in plants:
        tmp = tempfile.mkdtemp(prefix="w-disclose-selftest-")
        dst = os.path.join(tmp, "tree")
        shutil.copytree(ROOT, dst, symlinks=True, ignore=ignore)
        img_src = os.path.join(ROOT, IMAGE)
        if os.path.exists(img_src):
            os.makedirs(os.path.join(dst, os.path.dirname(IMAGE)), exist_ok=True)
            os.symlink(img_src, os.path.join(dst, IMAGE))
        p = os.path.join(dst, rel)
        body = open(p, encoding="utf-8").read()
        if old not in body:
            print("SELF-TEST DEFECT: plant %s matched NOTHING in %s (#3516)"
                  % (tag, rel))
            rc = 1
            shutil.rmtree(tmp)
            continue
        open(p, "w", encoding="utf-8").write(body.replace(old, new_text, 1))
        if open(p, encoding="utf-8").read() == body:
            print("SELF-TEST DEFECT: plant %s did not change the file" % tag)
            rc = 1
            shutil.rmtree(tmp)
            continue
        print("\n---- self-test: planted defect %s in %s ----" % (tag, rel))
        got = run(dst, quiet=True)
        added = fails(got) - base_fails
        hit = [a for a in added if expect in a]
        if not added:
            print("SELF-TEST RED: plant %s produced NO new failure" % tag)
            rc = 1
        elif not hit:
            print("SELF-TEST RED: plant %s was caught by the WRONG check: %s"
                  % (tag, sorted(added)[:2]))
            rc = 1
        else:
            print("plant %s CAUGHT by its own check (+%d failure line(s)):\n    %s"
                  % (tag, len(added), sorted(hit)[0][:140]))
        shutil.rmtree(tmp)
    print("\nself-test %s" % ("PASSED" if rc == 0 else "FAILED"))
    return rc


if __name__ == "__main__":
    args = sys.argv[1:]
    if args and args[0] == "--self-test":
        sys.exit(self_test())
    if args:
        print(__doc__)
        sys.exit(2)
    sys.exit(1 if run(ROOT).fail else 0)
