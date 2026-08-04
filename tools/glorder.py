#!/usr/bin/env python3
"""glorder — the `.gl` record-order reader, and the `.bss` order relation.

`docs/OBJ_DATA_BSS_SHAPE.md` §5.2 establishes **Rule A1**: partition a TU's
namespace-scope objects into *eager* (no dynamic initializer) and *deferred*
(has one); the eager objects are laid out in **`.gl` symbol-record order** and
the deferred ones in the **exact reverse** of it, with every eager object below
every deferred one. That correlation is the load-bearing input to the `.bss`
allocator -- the permutation is not a hash, it is an IL order -- and every
session so far has re-derived it by hand out of a `python3 - <<EOF` heredoc.

  glorder.py <bundle.gl>                    the record order
  glorder.py <file.cpp>                     capture the IL first, then the same
  glorder.py <bundle.gl> --obj <file.obj>   the three orders + the verdict
  glorder.py <file.cpp>  --obj <file.obj>
  glorder.py --selftest

Options:
  --obj FILE     also read the obj and check Rule A1 against it
  --all          list EVERY separator-introduced run, not only data records
  --raw          show byte offset, separator and record fields per run
  --work DIR     scratch dir when capturing from a .cpp
  --section NAME which obj section to relate (default .bss; try .data)

# What a "record" is here

A `.gl` symbol record is `<kind> <operand token> <SEP> <name> 00 <TYPE> ...`.
The separator is `00`, `26` (COMDAT-ish: deleting dtors, vftables, RTTI,
header-inline members) or `24` (**internal linkage**, and the name that follows
it is the COFF name *undecorated* -- `$sL` is the symbol `sL`). `25` introduces
a string literal and is deliberately not a data record.

This is a Python port of `crates/c2-il/src/func/gl.rs::gl_data_objects` and its
`data_object_at` frame check -- `<tag> [wide] <kind> 00 02 <linkage> <size
varint> <attr>` -- which is what separates a *data* record from a function or a
type-table entry structurally, rather than by guessing from the name. Keep the
two in step; gl.rs is the reference, this is the instrument.

# Two traps

* `c2rs capture` is pinned to `/Ox /GS- /c` (see tools/probe.py). If you pass a
  `.cpp` here, that is the profile the `.gl` comes from -- and it is NOT the
  profile `c2rs compile` uses for the obj you are relating it to. Capture the
  `.gl` and the obj through `tools/probe.py` if you need them to agree, and
  read its FLAG SKEW banner.
* An `--obj` whose `.bss` names do not intersect the `.gl` names at all is
  reported as an ERROR, not as a vacuously-holding rule. "Zero objects, no
  contradiction found" is exactly the shape of an instrument that grades
  nothing.
"""
import argparse
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
if _HERE not in sys.path:
    sys.path.insert(0, _HERE)

import coffdump  # noqa: E402
from probe import Fail, Skip, SKIP_LINE, find_c2rs, repo_root, run_capture  # noqa: E402

# gl.rs constants, kept spelled the same way so a diff between the two is easy.
SEP_NUL = 0x00
SEP_COMDAT = 0x26
SEP_UNDECORATED = 0x24
SEP_STRING_LITERAL = 0x25
NAME_SEPARATORS = (SEP_NUL, SEP_COMDAT, SEP_UNDECORATED)
SYMBOL_RECORD_KINDS = (0x00, 0x04, 0x0E, 0x10)
LINKAGE_DEFINED_EXTERN = 0x01
LINKAGE_UNDEF_EXTERN = 0x02
LINKAGE_STATIC = 0x04
DATA_ATTR_UNINITIALIZED = 0x00
DATA_ATTR_INITIALIZED = 0x80
TAG_WIDE = 0x40
WIDE_MARK = 0x80
ALIGN_OF_TAG = {0x82: 1, 0x84: 2, 0x86: 4, 0x88: 8}


def is_symbol_char(b):
    return (0x30 <= b <= 0x39 or 0x41 <= b <= 0x5A or 0x61 <= b <= 0x7A
            or b in (0x5F, 0x24, 0x3F, 0x40))


def is_object_name(b):
    return (len(b) > 0
            and (b[0] == 0x3F or (0x41 <= b[0] <= 0x5A) or (0x61 <= b[0] <= 0x7A)
                 or b[0] == 0x5F)
            and all(is_symbol_char(c) for c in b))


def read_token_var(gl, p):
    """`readers::read_token_var` -- 2-byte form unless the second byte has 0x80."""
    if p + 1 >= len(gl):
        return None
    b0, b1 = gl[p], gl[p + 1]
    if b1 & 0x80 == 0:
        return ((b0 << 8) | b1, 2)
    if p + 3 >= len(gl):
        return None
    return ((b0 << 24) | (b1 << 16) | (gl[p + 2] << 8) | gl[p + 3], 4)


def read_varint(gl, p):
    """`readers::read_varint` -> (value, next_p). Short form is SIGNED."""
    if p >= len(gl):
        return None
    if gl[p] == 0x80:
        if p + 4 >= len(gl):
            return None
        v = int.from_bytes(gl[p + 1:p + 5], "little", signed=True)
        return (v, p + 5)
    v = gl[p]
    return (v - 256 if v > 127 else v, p + 1)


def data_object_at(gl, name_nul, name):
    """`gl.rs::data_object_at`. None if the bytes after the name are not the
    ordinary-data frame -- which is what keeps a function or a type-table entry
    out, structurally."""
    if name_nul + 1 >= len(gl):
        return None
    tag = gl[name_nul + 1]
    if tag & 0x80 == 0:
        return None
    i = name_nul + 2
    if tag & TAG_WIDE:
        if i >= len(gl) or gl[i] & WIDE_MARK == 0:
            return None
        i += 1
    i += 1  # the kind byte
    if i + 2 >= len(gl) or gl[i] != 0x00 or gl[i + 1] != 0x02:
        return None
    linkage = gl[i + 2]
    if linkage == LINKAGE_DEFINED_EXTERN:
        external = True
    elif linkage == LINKAGE_STATIC:
        external = False
    else:
        return None
    got = read_varint(gl, i + 3)
    if got is None:
        return None
    size, p = got
    if size <= 0 or p >= len(gl):
        return None
    attr = gl[p]
    if attr == DATA_ATTR_UNINITIALIZED:
        initialized = False
    elif attr == DATA_ATTR_INITIALIZED:
        initialized = True
    else:
        return None
    if tag not in ALIGN_OF_TAG:
        return None
    return {
        "name": name.decode("ascii", "replace"),
        "size": size,
        "align": ALIGN_OF_TAG[tag],
        "external": external,
        "initialized": initialized,
        "linkage": linkage,
    }


def graphic_runs(gl):
    """Every maximal run of printable bytes, as (start, end). `end` indexes the
    non-printable byte that terminates it, which for a record name is the NUL
    where its TYPE begins."""
    out = []
    i, n = 0, len(gl)
    while i < n:
        if not (0x21 <= gl[i] <= 0x7E):
            i += 1
            continue
        start = i
        while i < n and 0x21 <= gl[i] <= 0x7E:
            i += 1
        if i >= n:
            break
        out.append((start, i))
    return out


def scan_records(gl):
    """Every separator-introduced name in `.gl` **file order**, with whatever
    the frame check says about it.

    Mirrors `gl_data_objects`' candidate walk: rightmost separator first, each
    candidate validated whole (token width, record-kind byte, and for a data
    record the ordinary-data frame) so a rejected candidate is rejected on
    structure and never on a guess about which `$` was meant.
    """
    out = []
    for (start, end) in graphic_runs(gl):
        q = end
        with_token = None   # rightmost candidate that also has a valid record head
        fallback = None     # rightmost separator-preceded run, token or not
        while q > start:
            q -= 1
            if q == 0:
                break
            sep = gl[q - 1]
            if sep not in NAME_SEPARATORS:
                continue
            if not is_object_name(gl[q:end]):
                continue
            cand = {
                "off": q, "name_end": end, "sep": sep, "token": None,
                "name": gl[q:end].decode("ascii", "replace"), "data": None,
            }
            if fallback is None:
                fallback = cand
            # The operand token immediately precedes the separator, and the
            # record-kind byte immediately precedes THAT. This pair is what
            # disambiguates `$sL$initializer$`: at its rightmost `$` the byte
            # before the token is another `$` (0x24), not a record kind, so the
            # candidate `initializer$` is rejected on structure and the walk
            # continues left to `sL$initializer$`. Dropping this check names
            # every initializer slot `initializer$`.
            tok = None
            for w in (4, 2):
                if q < w + 2:
                    continue
                p = q - 1 - w
                got = read_token_var(gl, p)
                if got is None or got[1] != w:
                    continue
                if gl[p - 1] not in SYMBOL_RECORD_KINDS:
                    continue
                tok = got[0]
                break
            if tok is None:
                continue
            cand["token"] = tok
            cand["data"] = data_object_at(gl, end, gl[q:end])
            if cand["data"] is not None:
                with_token = cand
                break
            if with_token is None:
                with_token = cand
        chosen = with_token or fallback
        if chosen is not None:
            out.append(chosen)
    return out


def coff_name(rec):
    """The COFF symbol name a record spells: a `24`-introduced name is the run
    itself, UNDECORATED (`$sL` -> `sL`); `00`/`26` names carry their own
    decoration and are used as found."""
    return rec["name"]


# ---------------------------------------------------------------------------

def load_gl(path):
    if not os.path.exists(path):
        raise Fail("glorder: no such .gl file: %s" % path)
    with open(path, "rb") as f:
        gl = f.read()
    if not gl:
        raise Fail("glorder: .gl file is empty: %s" % path)
    # The header is the cheapest structural check there is, and a `.gl` that
    # does not start with it is not one -- refuse rather than scan garbage and
    # report "0 records", which reads as a legitimate answer.
    if gl[:7] != b"\x11\x02\x061j2\x01":
        raise Fail("glorder: %s does not begin with the .gl header prefix "
                   "(11 02 06 '1j2' 01) -- not a captured .gl stream" % path)
    return gl


def resolve_gl(arg, work):
    """A `.gl` path, or a `.cpp` we capture first."""
    if arg.endswith(".gl"):
        return load_gl(arg), arg, None
    if not os.path.exists(arg):
        raise Fail("glorder: no such file: %s" % arg)
    c2rs = find_c2rs()
    if c2rs is None:
        raise Fail("glorder: no c2rs binary -- build it with\n"
                   "  cargo build --release -p c2-harness\n"
                   "or point $C2RS_BIN at one, or pass a .gl directly.")
    base = run_capture(c2rs, arg, os.path.join(work, "il"))
    p = base + ".gl"
    return load_gl(p), p, arg


def bss_objects(obj_path, section):
    """(name, value, size, symbol-table index) for every non-section symbol the
    named section defines, plus the set of ALL symbol names in the obj."""
    secs, syms = coffdump.load(obj_path)
    hits = [s for s in secs if s.name == section]
    if not hits:
        raise Fail("glorder: %s has no %s section (sections: %s)"
                   % (obj_path, section, ", ".join(sorted({s.name for s in secs}))))
    idxs = {s.index for s in hits}
    out = []
    for s in syms:
        if 0 < s.sec and (s.sec - 1) in idxs and s.kind != coffdump.K_SEC:
            out.append({"name": s.name, "value": s.value, "size": s.size,
                        "index": s.index})
    allnames = {s.name for s in syms}
    return out, allnames


def is_deferred(name, allnames, gl_names):
    """An object is *deferred* iff it has a dynamic initializer. Two witnesses,
    both structural: c2 emits a `<name>$initializer$` slot in `.CRT$XCU` and a
    `??__E<name>@@YAXXZ` thunk. Either one in the obj (or the `$initializer$`
    record in `.gl`) says deferred."""
    init = name + "$initializer$"
    if init in allnames or init in gl_names:
        return True
    return ("??__E" + name + "@@YAXXZ") in allnames


def relate(gl_recs, objs, allnames, section, out=sys.stdout):
    """Print the three orders side by side and state which relation holds."""
    w = out.write
    gl_names = [coff_name(r) for r in gl_recs]
    gl_all = set(gl_names)

    obj_names = {o["name"] for o in objs}
    common = [n for n in gl_names if n in obj_names]
    # Dedupe, first occurrence wins -- a name may appear in more than one record.
    seen, gl_order = set(), []
    for n in common:
        if n not in seen:
            seen.add(n)
            gl_order.append(n)

    asc = [o["name"] for o in sorted(objs, key=lambda o: (o["value"], o["index"]))]
    symtab = [o["name"] for o in sorted(objs, key=lambda o: o["index"])]

    w("\n== %s vs .gl ==\n" % section)
    w("  .gl record order   : %s\n" % (" ".join(gl_order) or "(none)"))
    w("  %s ascending%s: %s\n" % (section, " " * max(0, 9 - len(section)),
                                  " ".join(asc) or "(none)"))
    w("  %s symtab order%s: %s\n" % (section, " " * max(0, 6 - len(section)),
                                     " ".join(symtab) or "(none)"))

    # The other direction is normal and must not read as an anomaly: an
    # unreferenced internal-linkage object keeps its `.gl` record and is simply
    # not emitted, and an initializer slot lives in `.CRT$XCU`, not here.
    data_names = [coff_name(r) for r in gl_recs if r["data"]]
    unemitted = [n for n in data_names if n not in obj_names]
    if unemitted:
        w("\n  (%d .gl data record(s) have no %s symbol -- eliminated, or in "
          "another section: %s)\n" % (len(unemitted), section, " ".join(unemitted)))

    missing = [n for n in asc if n not in gl_all]
    if missing:
        w("\n  !! %d %s symbol(s) have NO .gl record: %s\n"
          % (len(missing), section, " ".join(missing)))
        w("     (a flag skew between the capture and the compile does this --\n")
        w("      see tools/probe.py. The verdict below is on the rest only.)\n")

    if not gl_order:
        raise Fail(
            "glorder: the .gl record names and the %s symbol names do not "
            "intersect at all (%d records, %d %s symbols). There is nothing to "
            "relate, so this is an ERROR and not a rule that vacuously holds."
            % (section, len(gl_recs), len(objs), section))

    def verdict(label, gl_seq, obj_seq):
        if gl_seq == obj_seq:
            return "%s: = .gl order" % label
        if list(reversed(gl_seq)) == obj_seq:
            return "%s: = REVERSE of .gl order" % label
        return "%s: NEITHER (.gl %s vs obj %s)" % (label, gl_seq, obj_seq)

    asc_common = [n for n in asc if n in gl_all]
    w("\n  whole-list  %s\n" % verdict(section + " ascending", gl_order, asc_common))
    w("  symtab      %s\n" % verdict(section + " symtab", gl_order, [n for n in symtab if n in gl_all]))

    # Rule A1 proper: partition, then check each block independently.
    defer = {n: is_deferred(n, allnames, gl_all) for n in gl_order}
    eager_gl = [n for n in gl_order if not defer[n]]
    defer_gl = [n for n in gl_order if defer[n]]
    eager_obj = [n for n in asc_common if not defer[n]]
    defer_obj = [n for n in asc_common if defer[n]]

    w("\n  Rule A1 (OBJ_DATA_BSS_SHAPE.md 5.2) -- eager = .gl order, "
      "deferred = reverse(.gl order),\n  every eager address below every deferred one:\n")
    w("    eager   (%d): .gl %s | %s %s\n"
      % (len(eager_gl), eager_gl or "-", section, eager_obj or "-"))
    w("    deferred(%d): .gl %s | %s %s\n"
      % (len(defer_gl), defer_gl or "-", section, defer_obj or "-"))

    fails = []
    if eager_gl != eager_obj:
        fails.append("eager block is not in .gl order")
    if list(reversed(defer_gl)) != defer_obj:
        fails.append("deferred block is not in reverse .gl order")
    addr = {o["name"]: o["value"] for o in objs}
    if eager_obj and defer_obj:
        if max(addr[n] for n in eager_obj) >= min(addr[n] for n in defer_obj):
            fails.append("an eager object is at or above a deferred one")
    if fails:
        w("    VERDICT: Rule A1 DOES NOT HOLD -- %s\n" % "; ".join(fails))
        return False
    w("    VERDICT: Rule A1 HOLDS (%d eager, %d deferred)\n"
      % (len(eager_gl), len(defer_gl)))
    return True


def dump(gl_recs, show_all, raw, out=sys.stdout):
    w = out.write
    recs = gl_recs if show_all else [r for r in gl_recs if r["data"]]
    kind = "all separator-introduced runs" if show_all else "data records"
    w("== .gl record order (%d %s) ==\n" % (len(recs), kind))
    if not recs:
        w("  (none)\n")
        return
    if raw:
        w("  %6s %4s %10s  %-6s %6s %5s %-9s %s\n"
          % ("off", "sep", "token", "size", "align", "init", "linkage", "name"))
    else:
        w("  %3s  %-6s %6s %5s %-9s %s\n"
          % ("#", "size", "align", "init", "linkage", "name"))
    for i, r in enumerate(recs):
        d = r["data"]
        if d:
            size, align = str(d["size"]), str(d["align"])
            init = "data" if d["initialized"] else "bss"
            link = "extern" if d["external"] else "static"
        else:
            size = align = init = link = "-"
        if raw:
            tok = "0x%08x" % r["token"] if r["token"] is not None else "-"
            w("  %6d 0x%02x %10s  %-6s %6s %5s %-9s %s\n"
              % (r["off"], r["sep"], tok, size, align, init, link, r["name"]))
        else:
            w("  %3d  %-6s %6s %5s %-9s %s\n"
              % (i, size, align, init, link, r["name"]))


# ---------------------------------------------------------------------------

def selftest():
    """Prove the reader refuses garbage instead of reporting "0 records".

    A `.gl` scanner that returns an empty list for a truncated or wrong file is
    indistinguishable from one reading a TU with no data objects -- the same
    absence-reads-as-success shape the `sed`-that-read-a-missing-number-as-0
    incident had. Every check below is a refusal, plus true positives so the
    refusals are not green from the scanner never working at all.
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

    def expect_raises(exc, fn, what):
        try:
            fn()
        except exc:
            return
        except BaseException as e:
            raise AssertionError("%s raised %r, wanted %s" % (what, e, exc))
        raise AssertionError("%s returned normally; wanted %s -- an empty "
                             "success here grades nothing" % (what, exc))

    d = tempfile.mkdtemp(prefix="glorder-selftest-")
    HDR = b"\x11\x02\x061j2\x01" + b"\x00" * 4

    def missing():
        expect_raises(Fail, lambda: load_gl(os.path.join(d, "nope.gl")),
                      "load_gl(missing)")
    check("missing .gl is an error", missing)

    def empty():
        p = os.path.join(d, "empty.gl")
        open(p, "wb").close()
        expect_raises(Fail, lambda: load_gl(p), "load_gl(empty)")
    check("empty .gl is an error, not zero records", empty)

    def not_a_gl():
        p = os.path.join(d, "wrong.gl")
        with open(p, "wb") as f:
            f.write(b"MZ\x90\x00" + b"\x00" * 200)
        expect_raises(Fail, lambda: load_gl(p), "load_gl(not a .gl)")
    check("a file without the .gl header prefix is refused", not_a_gl)

    def obj_missing_section():
        # Build a one-section obj and ask for .bss.
        import struct
        p = os.path.join(d, "nobss.obj")
        symoff = 20 + 40 + 4
        hdr = struct.pack("<HHIIIHH", 0x1F2, 1, 0, symoff, 1, 0, 0)
        sec = (b".text\0\0\0" + struct.pack("<IIIIIIHH", 4, 0, 4, 60, 0, 0, 0, 0)
               + struct.pack("<I", 0x60000020))
        sym = b"f\0\0\0\0\0\0\0" + struct.pack("<IhHBB", 0, 1, 0x20, 2, 0)
        with open(p, "wb") as f:
            f.write(hdr + sec + b"\x00" * 4 + sym + struct.pack("<I", 4))
        expect_raises(Fail, lambda: bss_objects(p, ".bss"),
                      "bss_objects(obj with no .bss)")
    check("an obj with no such section is an error", obj_missing_section)

    def frame_check_rejects():
        # The ordinary-data frame is `<tag> [wide] <kind> 00 02 <linkage> <size>
        # <attr>`. Corrupt each field in turn; every one must refuse.
        # `<name> 00 <tag> <kind> 00 02 <linkage> <size varint> <attr>`; the
        # `name_nul` argument indexes that first 00.
        good = b"\x86\x01\x00\x02\x04\x01\x00"
        gl = b"$sL\x00" + good
        assert data_object_at(gl, 3, b"sL") is not None, \
            "the reference frame does not parse -- the negatives below are vacuous"
        bad = [
            (b"\x06\x01\x00\x02\x04\x01\x00", "tag without 0x80"),
            (b"\x86\x01\x00\x03\x04\x01\x00", "frame 00 03 instead of 00 02"),
            (b"\x86\x01\x05\x04\x04\x01\x00", "a FUNCTION record (86 01 05 04)"),
            (b"\x86\x01\x00\x02\x02\x01\x00", "linkage 02 (undefined extern)"),
            (b"\x86\x01\x00\x02\x07\x01\x00", "linkage 07 (unseen)"),
            (b"\x86\x01\x00\x02\x04\x00\x00", "size 0"),
            (b"\x86\x01\x00\x02\x04\x01\x7f", "attr 7f (neither 00 nor 80)"),
            (b"\x81\x01\x00\x02\x04\x01\x00", "tag 81 (no known alignment)"),
        ]
        for payload, why in bad:
            g = b"$sL\x00" + payload
            assert data_object_at(g, 3, b"sL") is None, \
                "frame check accepted a record with %s" % why
    check("the data-record frame check rejects each corrupted field",
          frame_check_rejects)

    def positive_records():
        # A hand-built .gl with three records: a static uninitialized object, an
        # external initialized one, and a FUNCTION record (which must not be
        # read as data). Order is the thing under test.
        def rec(sep, name, payload):
            # <kind> <operand token> <SEP> <name> 00 <TYPE...>
            return b"\x04" + b"\x30\x01" + bytes([sep]) + name + b"\x00" + payload
        gl = (HDR
              + rec(SEP_UNDECORATED, b"sB", b"\x86\x01\x00\x02\x04\x04\x00")
              + rec(SEP_NUL, b"?gD@@3HA", b"\x86\x01\x00\x02\x01\x04\x80")
              + rec(SEP_NUL, b"?f@@YAHH@Z", b"\x86\x01\x05\x04\x00\x00\x00"))
        recs = scan_records(gl)
        names = [r["name"] for r in recs if r["data"]]
        assert names == ["sB", "?gD@@3HA"], \
            "data-record order wrong: %r (a function record leaked in?)" % names
        by = {r["name"]: r["data"] for r in recs if r["data"]}
        assert by["sB"]["initialized"] is False and by["sB"]["external"] is False, \
            "static uninitialized object misread: %r" % (by["sB"],)
        assert by["?gD@@3HA"]["initialized"] is True and by["?gD@@3HA"]["external"], \
            "external initialized object misread: %r" % (by["?gD@@3HA"],)
        assert by["sB"]["size"] == 4 and by["sB"]["align"] == 4, \
            "size/align misread: %r" % (by["sB"],)
        buf = io.StringIO()
        dump(recs, False, True, out=buf)
        assert "sB" in buf.getvalue() and "2 data records" in buf.getvalue(), \
            "dump did not report the two data records"
    check("a synthetic .gl yields the right records, in order (true positive)",
          positive_records)

    def order_reverses():
        # Rule A1's deferred case, end to end, on synthetic inputs: .gl order
        # s2 s1 s3 with all three deferred must relate to .bss ascending
        # s3 s1 s2.
        recs = [{"off": 0, "name_end": 0, "sep": SEP_UNDECORATED, "token": 1,
                 "name": n, "data": {"name": n, "size": 1, "align": 1,
                                     "external": False, "initialized": False,
                                     "linkage": 4}}
                for n in ("s2", "s1", "s3")]
        objs = [{"name": "s3", "value": 0, "size": 1, "index": 34},
                {"name": "s1", "value": 1, "size": 1, "index": 33},
                {"name": "s2", "value": 2, "size": 1, "index": 32}]
        allnames = {"s1$initializer$", "s2$initializer$", "s3$initializer$"}
        buf = io.StringIO()
        held = relate(recs, objs, allnames | {"s1", "s2", "s3"}, ".bss", out=buf)
        t = buf.getvalue()
        assert held, "Rule A1 reported as failing on the canonical deferred case:\n" + t
        assert "REVERSE of .gl order" in t, "the reverse relation was not named:\n" + t
        assert "deferred(3)" in t, "the partition is wrong:\n" + t

        # ...and it must FAIL when the obj order is wrong. A verdict that
        # cannot say "does not hold" is not a verdict.
        bad = [{"name": "s2", "value": 0, "size": 1, "index": 32},
               {"name": "s1", "value": 1, "size": 1, "index": 33},
               {"name": "s3", "value": 2, "size": 1, "index": 34}]
        buf2 = io.StringIO()
        held2 = relate(recs, bad, allnames | {"s1", "s2", "s3"}, ".bss", out=buf2)
        assert not held2, "Rule A1 reported as HOLDING on a permuted .bss:\n" + buf2.getvalue()
    check("Rule A1 holds on the canonical case and FAILS on a permuted one",
          order_reverses)

    def eager_case():
        recs = [{"off": 0, "name_end": 0, "sep": SEP_UNDECORATED, "token": 1,
                 "name": n, "data": {"name": n, "size": 1, "align": 1,
                                     "external": False, "initialized": False,
                                     "linkage": 4}}
                for n in ("b", "a", "c")]
        objs = [{"name": "b", "value": 0, "size": 1, "index": 10},
                {"name": "a", "value": 1, "size": 1, "index": 11},
                {"name": "c", "value": 2, "size": 1, "index": 12}]
        buf = io.StringIO()
        held = relate(recs, objs, {"a", "b", "c"}, ".bss", out=buf)
        assert held, "eager (no initializer) case should be = .gl order:\n" + buf.getvalue()
        assert "eager   (3)" in buf.getvalue(), "partition put eager objects in deferred"
    check("an eager-only TU relates as = .gl order", eager_case)

    def no_intersection():
        recs = [{"off": 0, "name_end": 0, "sep": SEP_NUL, "token": 1,
                 "name": "zzz", "data": None}]
        objs = [{"name": "qqq", "value": 0, "size": 1, "index": 1}]
        expect_raises(Fail, lambda: relate(recs, objs, {"qqq"}, ".bss",
                                           out=io.StringIO()),
                      "relate(.gl and .bss share no names)")
    check("a zero-name intersection is an ERROR, not a vacuous pass",
          no_intersection)

    print("\n%d/%d checks passed" % (sum(ok), len(ok)))
    return 0 if all(ok) else 1


def main():
    p = argparse.ArgumentParser(
        prog="glorder", description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("input", nargs="?", help="a .gl bundle file, or a .cpp to capture")
    p.add_argument("--obj")
    p.add_argument("--all", action="store_true")
    p.add_argument("--raw", action="store_true")
    p.add_argument("--work")
    p.add_argument("--section", default=".bss")
    p.add_argument("--selftest", action="store_true")
    args = p.parse_args()

    if args.selftest:
        return selftest()

    work = args.work or os.path.join(repo_root(), "work", "glorder")
    try:
        if args.input is None:
            raise Fail("glorder: expected a .gl or a .cpp (see --help)")
        gl, gl_path, from_cpp = resolve_gl(args.input, work)
        print("== %s (%d bytes)%s ==" % (gl_path, len(gl),
                                         " captured from " + from_cpp if from_cpp else ""))
        recs = scan_records(gl)
        dump(recs, args.all, args.raw)
        if args.obj:
            if not os.path.exists(args.obj):
                raise Fail("glorder: no such obj: %s" % args.obj)
            objs, allnames = bss_objects(args.obj, args.section)
            held = relate(recs, objs, allnames, args.section)
            return 0 if held else 1
        return 0
    except Skip:
        print(SKIP_LINE)
        return 0
    except Fail as e:
        print(str(e), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
