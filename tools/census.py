#!/usr/bin/env python3
"""census — query the committed section census of the 871-object workload.

`work/w-bss/census/sections.jsonl` is one JSON record per workload object:

    {"src": "src/App.cpp", "nsec": 494,
     "order": [".drectve", ".debug$S", ".XBLD$W:C2", ".bss", ".XBLD$W:C1", ...],
     "data": [ {...per-.data-section header + symbols...} ],
     "bss":  [ {...per-.bss-section header + symbols...} ]}

`order` is the **full ordered section-name list**, with the two `.XBLD$W`
watermarks distinguished as `:C2` / `:C1` (they are not interchangeable -- which
side of them a `.bss` falls on is a different obj, see OBJ_DATA_BSS_SHAPE.md
2.2). Every session so far has hand-written a `python3 - <<EOF` heredoc to ask
"how many objects have a `.bss` before the C1 watermark", "which objects have
more than one `.data`", "what is the section multiset of App.cpp". This is that
heredoc, once.

The file is TRACKED even though `/work` is gitignored, so this needs no
toolchain and no capture -- it answers offline.

  census.py count                       how many objects match the filter
  census.py list                        src + nsec, one per line
  census.py order <src> [<src>...]      the full ordered section list
  census.py multiset [<src>...]         section-name counts (per object, or
                                        aggregated over the filter)
  census.py names                       every distinct section name + how many
                                        objects carry it
  census.py sections <src>...           the per-.data/.bss header + symbol detail
  census.py --selftest

Filters (accepted by every subcommand, and combined with AND):
  --has NAME            object's order contains NAME            (repeatable)
  --not-has NAME        ...does not                             (repeatable)
  --count NAME=LO..HI   occurrences of NAME within [LO,HI]      (repeatable)
                        `--count .bss=1..1`, `--count .data=2..` , `..0`
  --nsec LO..HI         section count within [LO,HI]
  --src-re REGEX        source path matches (Python re, searched not anchored)
  --before A=B          every A in the order comes before every B
  --after  A=B          every A comes after every B
  --straddles A=B       A occurs BOTH before and after B (a third case, not
                        the negation of either -- 113 objects have a .bss
                        between the watermarks AND .bss COMDATs after C1)

Output control:
  --sort nsec|src|-nsec|-src     (default: file order)
  --limit N
  --census PATH         override (else $C2RS_CENSUS, else the tracked file)

A filter that matches nothing exits **1** and says so on stderr. "0 objects" is
a legitimate answer to a question and a silent lie when the census failed to
load, so the two are kept distinguishable.
"""
import argparse
import json
import os
import re
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
if _HERE not in sys.path:
    sys.path.insert(0, _HERE)

from probe import Fail, repo_root  # noqa: E402

DEFAULT_CENSUS = os.path.join("work", "w-bss", "census", "sections.jsonl")
REQUIRED_KEYS = ("src", "nsec", "order")


def census_path(override=None):
    if override:
        return override
    env = os.environ.get("C2RS_CENSUS")
    if env:
        return env
    return os.path.join(repo_root(), DEFAULT_CENSUS)


def load_census(path):
    """Parse the JSONL. Refuses an absent, empty, or malformed file rather than
    returning an empty list -- an empty corpus and a missing corpus must not
    produce the same answer."""
    if not os.path.exists(path):
        raise Fail("census: no such census file: %s\n"
                   "  (it lives at %s in the repo; override with --census or "
                   "$C2RS_CENSUS)" % (path, DEFAULT_CENSUS))
    if os.path.getsize(path) == 0:
        raise Fail("census: census file is empty: %s" % path)
    recs = []
    with open(path) as f:
        for lineno, line in enumerate(f, 1):
            if not line.strip():
                continue
            try:
                r = json.loads(line)
            except ValueError as e:
                raise Fail("census: %s line %d is not JSON: %s" % (path, lineno, e))
            if not isinstance(r, dict):
                raise Fail("census: %s line %d is not a JSON object" % (path, lineno))
            for k in REQUIRED_KEYS:
                if k not in r:
                    raise Fail("census: %s line %d has no %r key -- this is not a "
                               "section census" % (path, lineno, k))
            if not isinstance(r["order"], list):
                raise Fail("census: %s line %d: 'order' is not a list" % (path, lineno))
            # The census's own internal consistency, checked once at load: nsec
            # must equal len(order). A record where they disagree means the
            # generator and the reader disagree about what a section is, and
            # every count below would be quietly wrong.
            if r["nsec"] != len(r["order"]):
                raise Fail("census: %s line %d (%s): nsec=%d but order has %d "
                           "entries -- the census is internally inconsistent"
                           % (path, lineno, r.get("src"), r["nsec"], len(r["order"])))
            recs.append(r)
    if not recs:
        raise Fail("census: %s contains no records" % path)
    return recs


# ---------------------------------------------------------------------------
# Filters
# ---------------------------------------------------------------------------

def parse_range(spec, what):
    """`LO..HI`, `LO..`, `..HI`, or a bare `N`."""
    s = spec.strip()
    if ".." not in s:
        try:
            n = int(s)
        except ValueError:
            raise Fail("census: %s: %r is not a number or LO..HI range" % (what, spec))
        return (n, n)
    lo, _, hi = s.partition("..")
    try:
        lo_v = int(lo) if lo.strip() else 0
        hi_v = int(hi) if hi.strip() else float("inf")
    except ValueError:
        raise Fail("census: %s: %r is not a LO..HI range" % (what, spec))
    if lo_v > hi_v:
        raise Fail("census: %s: range %r is empty (lo > hi)" % (what, spec))
    return (lo_v, hi_v)


def parse_pair(spec, what):
    if "=" not in spec:
        raise Fail("census: %s expects A=B, got %r" % (what, spec))
    a, _, b = spec.partition("=")
    if not a or not b:
        raise Fail("census: %s expects A=B with both sides non-empty, got %r"
                   % (what, spec))
    return a, b


def build_filter(args, known_names):
    """A list of (description, predicate). Every name mentioned must actually
    occur somewhere in the census -- a typo'd `--has .txet` would otherwise
    return 0 objects and read as a finding."""
    preds = []

    def require_known(name, flag):
        if name not in known_names:
            near = sorted(n for n in known_names if name.lstrip(".") in n)[:6]
            raise Fail("census: %s %r never appears in this census.%s"
                       % (flag, name,
                          ("  Did you mean: " + ", ".join(near)) if near else ""))

    for n in args.has or []:
        require_known(n, "--has")
        preds.append(("has %s" % n, lambda r, n=n: n in r["order"]))
    for n in getattr(args, "not_has") or []:
        require_known(n, "--not-has")
        preds.append(("not-has %s" % n, lambda r, n=n: n not in r["order"]))
    for spec in args.count or []:
        name, rng = parse_pair(spec, "--count")
        # A `--count X=0..0` is a legitimate way to ask "objects with no X", and
        # X may then be a name that DOES occur elsewhere -- so still require it.
        require_known(name, "--count")
        lo, hi = parse_range(rng, "--count %s" % name)
        preds.append(("count %s in [%s,%s]" % (name, lo, hi),
                      lambda r, n=name, lo=lo, hi=hi: lo <= r["order"].count(n) <= hi))
    if args.nsec:
        lo, hi = parse_range(args.nsec, "--nsec")
        preds.append(("nsec in [%s,%s]" % (lo, hi),
                      lambda r, lo=lo, hi=hi: lo <= r["nsec"] <= hi))
    if args.src_re:
        try:
            rx = re.compile(args.src_re)
        except re.error as e:
            raise Fail("census: --src-re %r is not a valid regex: %s" % (args.src_re, e))
        preds.append(("src matches /%s/" % args.src_re,
                      lambda r, rx=rx: rx.search(r["src"]) is not None))
    for spec in args.before or []:
        a, b = parse_pair(spec, "--before")
        require_known(a, "--before")
        require_known(b, "--before")
        preds.append(("every %s before every %s" % (a, b),
                      lambda r, a=a, b=b: _ordered(r["order"], a, b)))
    for spec in args.after or []:
        a, b = parse_pair(spec, "--after")
        require_known(a, "--after")
        require_known(b, "--after")
        preds.append(("every %s after every %s" % (a, b),
                      lambda r, a=a, b=b: _ordered(r["order"], b, a)))
    for spec in args.straddles or []:
        a, b = parse_pair(spec, "--straddles")
        require_known(a, "--straddles")
        require_known(b, "--straddles")
        preds.append(("%s occurs both before and after %s" % (a, b),
                      lambda r, a=a, b=b: _straddles(r["order"], a, b)))
    return preds


def _ordered(order, a, b):
    """True iff `a` and `b` both occur and every `a` precedes every `b`."""
    ia = [i for i, n in enumerate(order) if n == a]
    ib = [i for i, n in enumerate(order) if n == b]
    if not ia or not ib:
        return False
    return max(ia) < min(ib)


def _straddles(order, a, b):
    """True iff `a` occurs on BOTH sides of `b`.

    This is the OBJ_DATA_BSS_SHAPE.md 2.2 question and it is a third case, not
    the negation of either --before or --after: 113 of the 690 `.bss`-carrying
    workload objects have the shared `.bss` between the two watermarks AND
    per-symbol `.bss` COMDATs after `.XBLD$W:C1`, so subtracting before+after
    from the total is the only way to see them without this.
    """
    ia = [i for i, n in enumerate(order) if n == a]
    ib = [i for i, n in enumerate(order) if n == b]
    if not ia or not ib:
        return False
    return min(ia) < max(ib) and max(ia) > min(ib)


def apply_filter(recs, preds):
    out = recs
    for _desc, p in preds:
        out = [r for r in out if p(r)]
    return out


def sort_recs(recs, spec):
    if not spec:
        return recs
    rev = spec.startswith("-")
    key = spec.lstrip("-")
    if key not in ("nsec", "src"):
        raise Fail("census: --sort takes nsec|src|-nsec|-src, got %r" % spec)
    return sorted(recs, key=lambda r: r[key], reverse=rev)


def pick(recs, srcs):
    """Records for the named sources; an unmatched name is an error, not a
    silent omission."""
    by = {r["src"]: r for r in recs}
    out = []
    for s in srcs:
        if s in by:
            out.append(by[s])
            continue
        hits = [r for r in recs if s in r["src"]]
        if len(hits) == 1:
            out.append(hits[0])
        elif not hits:
            raise Fail("census: no object matches %r" % s)
        else:
            raise Fail("census: %r is ambiguous (%d matches): %s"
                       % (s, len(hits), ", ".join(r["src"] for r in hits[:8])))
    return out


# ---------------------------------------------------------------------------
# Subcommands
# ---------------------------------------------------------------------------

def cmd_count(recs, args, out):
    out.write("%d\n" % len(recs))


def cmd_list(recs, args, out):
    out.write("%6s  %s\n" % ("nsec", "src"))
    for r in recs:
        out.write("%6d  %s\n" % (r["nsec"], r["src"]))


def cmd_order(recs, args, out):
    for r in pick(recs, args.srcs):
        out.write("== %s (%d sections) ==\n" % (r["src"], r["nsec"]))
        for i, n in enumerate(r["order"]):
            out.write("  %4d  %s\n" % (i, n))


def cmd_multiset(recs, args, out):
    targets = pick(recs, args.srcs) if args.srcs else None
    if targets is None:
        counts, objs = {}, {}
        for r in recs:
            seen = set()
            for n in r["order"]:
                counts[n] = counts.get(n, 0) + 1
                seen.add(n)
            for n in seen:
                objs[n] = objs.get(n, 0) + 1
        out.write("== section multiset over %d object(s) ==\n" % len(recs))
        out.write("  %8s %8s  %s\n" % ("total", "objects", "section"))
        for n, c in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
            out.write("  %8d %8d  %s\n" % (c, objs[n], n))
        return
    for r in targets:
        counts = {}
        for n in r["order"]:
            counts[n] = counts.get(n, 0) + 1
        out.write("== %s (%d sections) ==\n" % (r["src"], r["nsec"]))
        for n, c in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
            out.write("  %6d  %s\n" % (c, n))


def cmd_names(recs, args, out):
    objs = {}
    for r in recs:
        for n in set(r["order"]):
            objs[n] = objs.get(n, 0) + 1
    out.write("== %d distinct section names over %d object(s) ==\n"
              % (len(objs), len(recs)))
    for n, c in sorted(objs.items(), key=lambda kv: (-kv[1], kv[0])):
        out.write("  %6d objects  %s\n" % (c, n))


def cmd_sections(recs, args, out):
    for r in pick(recs, args.srcs):
        out.write("== %s ==\n" % r["src"])
        for key in ("data", "bss"):
            entries = r.get(key) or []
            out.write("  .%s: %d section(s)\n" % (key, len(entries)))
            for e in entries:
                out.write("    idx %-5d size %-8s vsz %-6s nrel %-4s comdat %-5s %s\n"
                          % (e.get("idx", -1), e.get("size"), e.get("vsz"),
                             e.get("nrel"), e.get("comdat"), e.get("chdec", "")))
                for s in e.get("syms") or []:
                    out.write("        v=%-8s sc=%-3s %s\n"
                              % (s.get("v"), s.get("sc"), s.get("n")))


COMMANDS = {
    "count": (cmd_count, 0), "list": (cmd_list, 0), "names": (cmd_names, 0),
    "order": (cmd_order, 1), "multiset": (cmd_multiset, -1),
    "sections": (cmd_sections, 1),
}


# ---------------------------------------------------------------------------

def selftest():
    """Prove the query layer refuses bad input instead of answering 0.

    The failure mode this guards is specific: a filter that silently matches
    nothing looks exactly like a real finding ("no object has two .bss!"), and a
    census that failed to load looks exactly like an empty one.
    """
    import io
    import tempfile
    ok = []

    def check(name, fn):
        try:
            fn()
        except AssertionError as e:
            print("FAIL  %s: %s" % (name, e))
            ok.append(False)
        else:
            print("ok    %s" % name)
            ok.append(True)

    def expect_fail(fn, what):
        try:
            fn()
        except Fail:
            return
        except BaseException as e:
            raise AssertionError("%s raised %r, wanted Fail" % (what, e))
        raise AssertionError("%s returned normally; wanted Fail -- answering "
                             "'0 objects' here is indistinguishable from a "
                             "real finding" % what)

    d = tempfile.mkdtemp(prefix="census-selftest-")

    def write(name, text):
        p = os.path.join(d, name)
        with open(p, "w") as f:
            f.write(text)
        return p

    GOOD = "\n".join([
        json.dumps({"src": "a.cpp", "nsec": 3,
                    "order": [".drectve", ".XBLD$W:C2", ".XBLD$W:C1"],
                    "data": [], "bss": []}),
        json.dumps({"src": "b.cpp", "nsec": 5,
                    "order": [".drectve", ".XBLD$W:C2", ".bss", ".XBLD$W:C1", ".text"],
                    "data": [], "bss": [{"idx": 2, "size": 4, "syms": [{"n": "g"}]}]}),
        json.dumps({"src": "c.cpp", "nsec": 6,
                    "order": [".drectve", ".XBLD$W:C2", ".XBLD$W:C1", ".text",
                              ".bss", ".bss"],
                    "data": [], "bss": []}),
    ]) + "\n"

    def missing():
        expect_fail(lambda: load_census(os.path.join(d, "nope.jsonl")),
                    "load_census(missing)")
    check("a missing census is an error", missing)

    def empty():
        expect_fail(lambda: load_census(write("empty.jsonl", "")),
                    "load_census(empty)")
    check("an empty census is an error, not zero objects", empty)

    def blank_only():
        expect_fail(lambda: load_census(write("blank.jsonl", "\n\n\n")),
                    "load_census(only blank lines)")
    check("a census of only blank lines is an error", blank_only)

    def not_json():
        expect_fail(lambda: load_census(write("bad.jsonl", "{not json}\n")),
                    "load_census(malformed JSON)")
    check("malformed JSON is refused, not skipped", not_json)

    def wrong_schema():
        expect_fail(lambda: load_census(write("sch.jsonl",
                                              json.dumps({"src": "x", "nsec": 1}) + "\n")),
                    "load_census(record with no 'order')")
        expect_fail(lambda: load_census(write("sch2.jsonl",
                                              json.dumps({"nsec": 1, "order": []}) + "\n")),
                    "load_census(record with no 'src')")
        expect_fail(lambda: load_census(write("sch3.jsonl", "[1,2,3]\n")),
                    "load_census(JSON array, not object)")
    check("a record missing a required key is refused", wrong_schema)

    def inconsistent():
        bad = json.dumps({"src": "x", "nsec": 99, "order": [".text"]}) + "\n"
        expect_fail(lambda: load_census(write("inc.jsonl", bad)),
                    "load_census(nsec != len(order))")
    check("nsec disagreeing with len(order) is refused", inconsistent)

    def bad_ranges():
        expect_fail(lambda: parse_range("banana", "t"), "parse_range('banana')")
        expect_fail(lambda: parse_range("5..1", "t"), "parse_range('5..1')")
        expect_fail(lambda: parse_pair("noequals", "t"), "parse_pair('noequals')")
        expect_fail(lambda: parse_pair("=b", "t"), "parse_pair('=b')")
        assert parse_range("2", "t") == (2, 2), "bare N range wrong"
        assert parse_range("1..3", "t") == (1, 3), "LO..HI range wrong"
        assert parse_range("2..", "t")[0] == 2, "open-ended range wrong"
        assert parse_range("..4", "t") == (0, 4), "open-start range wrong"
    check("range/pair specs are validated, and the valid ones parse", bad_ranges)

    p = write("good.jsonl", GOOD)
    recs = load_census(p)
    known = {n for r in recs for n in r["order"]}

    def unknown_section():
        class A:
            has, not_has, count, nsec, src_re, before, after = \
                [".txet"], None, None, None, None, None, None
        expect_fail(lambda: build_filter(A(), known), "--has with a typo'd name")
    check("a section name that never occurs is a typo, not a 0-result", unknown_section)

    def bad_regex():
        class A:
            has, not_has, count, nsec, src_re, before, after = \
                None, None, None, None, "([", None, None
        expect_fail(lambda: build_filter(A(), known), "--src-re with a bad regex")
    check("an invalid --src-re is refused", bad_regex)

    def unknown_src():
        expect_fail(lambda: pick(recs, ["zzz.cpp"]), "pick(unknown src)")
    check("naming an object that is not in the census is an error", unknown_src)

    def positives():
        class A:
            has, not_has, count, nsec, src_re, before, after, straddles = \
                None, None, None, None, None, None, None, None
        a = A()
        a.has = [".bss"]
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["b.cpp", "c.cpp"], "--has .bss wrong: %r" % got

        a = A(); a.count = [".bss=2.."]
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["c.cpp"], "--count .bss=2.. wrong: %r" % got

        a = A(); a.count = [".bss=0..0"]
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["a.cpp"], "--count .bss=0..0 wrong: %r" % got

        a = A(); a.not_has = [".bss"]
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["a.cpp"], "--not-has .bss wrong: %r" % got

        a = A(); a.nsec = "5..6"
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["b.cpp", "c.cpp"], "--nsec 5..6 wrong: %r" % got

        # The watermark question this census exists for: which side of the C1
        # watermark does the .bss fall on? b.cpp is between them, c.cpp after.
        a = A(); a.before = [".bss=.XBLD$W:C1"]
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["b.cpp"], "--before .bss=.XBLD$W:C1 wrong: %r" % got

        a = A(); a.after = [".bss=.XBLD$W:C1"]
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["c.cpp"], "--after .bss=.XBLD$W:C1 wrong: %r" % got

        # `--straddles` is a THIRD case, not the negation of before/after:
        # d.cpp has a .bss on each side of the C1 watermark and must be in
        # neither --before nor --after, but in --straddles.
        recs2 = recs + [{"src": "d.cpp", "nsec": 5,
                         "order": [".drectve", ".XBLD$W:C2", ".bss",
                                   ".XBLD$W:C1", ".bss"],
                         "data": [], "bss": []}]
        known2 = {n for r in recs2 for n in r["order"]}
        a = A(); a.straddles = [".bss=.XBLD$W:C1"]
        got = [r["src"] for r in apply_filter(recs2, build_filter(a, known2))]
        assert got == ["d.cpp"], "--straddles wrong: %r" % got
        for flag in ("before", "after"):
            a = A(); setattr(a, flag, [".bss=.XBLD$W:C1"])
            got = [r["src"] for r in apply_filter(recs2, build_filter(a, known2))]
            assert "d.cpp" not in got, \
                "--%s counted a straddling object: %r" % (flag, got)

        a = A(); a.src_re = r"^[bc]\.cpp$"
        got = [r["src"] for r in apply_filter(recs, build_filter(a, known))]
        assert got == ["b.cpp", "c.cpp"], "--src-re wrong: %r" % got

        assert [r["src"] for r in sort_recs(recs, "-nsec")] == ["c.cpp", "b.cpp", "a.cpp"], \
            "--sort -nsec wrong"
    check("the filters select the right objects (true positives)", positives)

    def rendering():
        buf = io.StringIO()

        class A:
            srcs = ["c.cpp"]
        cmd_order(recs, A(), buf)
        t = buf.getvalue()
        assert ".XBLD$W:C1" in t and t.count(".bss") == 2, \
            "order did not render the full ordered list: %r" % t

        buf = io.StringIO()
        cmd_multiset(recs, A(), buf)
        assert "2  .bss" in buf.getvalue(), "multiset count wrong: %r" % buf.getvalue()

        buf = io.StringIO()

        class B:
            srcs = None
        cmd_multiset(recs, B(), buf)
        assert "over 3 object(s)" in buf.getvalue(), "aggregate multiset header wrong"

        buf = io.StringIO()

        class C:
            srcs = ["b.cpp"]
        cmd_sections(recs, C(), buf)
        assert "g" in buf.getvalue(), "sections did not print the .bss symbol"
    check("order/multiset/sections render the real content", rendering)

    def ambiguous():
        expect_fail(lambda: pick(recs, ["cpp"]), "pick(ambiguous substring)")
    check("an ambiguous source substring is an error, not the first match",
          ambiguous)

    print("\n%d/%d checks passed" % (sum(ok), len(ok)))
    return 0 if all(ok) else 1


def main():
    p = argparse.ArgumentParser(
        prog="census", description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("cmd", nargs="?", choices=sorted(COMMANDS))
    p.add_argument("srcs", nargs="*")
    p.add_argument("--has", action="append")
    p.add_argument("--not-has", action="append", dest="not_has")
    p.add_argument("--count", action="append")
    p.add_argument("--nsec")
    p.add_argument("--src-re", dest="src_re")
    p.add_argument("--before", action="append")
    p.add_argument("--after", action="append")
    p.add_argument("--straddles", action="append")
    p.add_argument("--sort")
    p.add_argument("--limit", type=int)
    p.add_argument("--census")
    p.add_argument("--quiet", action="store_true",
                   help="suppress the filter banner on stderr")
    p.add_argument("--selftest", action="store_true")
    args = p.parse_args()

    if args.selftest:
        return selftest()
    if not args.cmd:
        p.print_usage()
        print("census: expected a subcommand (%s)" % ", ".join(sorted(COMMANDS)),
              file=sys.stderr)
        return 2

    try:
        path = census_path(args.census)
        recs = load_census(path)
        total = len(recs)
        known = {n for r in recs for n in r["order"]}
        preds = build_filter(args, known)
        recs = apply_filter(recs, preds)
        fn, needs_srcs = COMMANDS[args.cmd]
        if needs_srcs == 1 and not args.srcs:
            raise Fail("census: `%s` needs at least one source path" % args.cmd)
        recs = sort_recs(recs, args.sort)
        if args.limit is not None:
            recs = recs[:args.limit]
        if not args.quiet:
            desc = "; ".join(d for d, _ in preds) or "no filter"
            print("# %s: %d/%d objects (%s)" % (path, len(recs), total, desc),
                  file=sys.stderr)
        if not recs:
            # Distinguishable from a load failure: this is a real, empty answer,
            # but it exits nonzero so a shell pipeline cannot mistake it for
            # data.
            print("census: 0 objects match (%s)"
                  % ("; ".join(d for d, _ in preds) or "no filter"), file=sys.stderr)
            return 1
        fn(recs, args, sys.stdout)
        return 0
    except Fail as e:
        print(str(e), file=sys.stderr)
        return 1
    except BrokenPipeError:
        return 0


if __name__ == "__main__":
    sys.exit(main())
