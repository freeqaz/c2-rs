#!/usr/bin/env python3
"""probe — the one-command measurement loop: source -> real toolchain -> one
consolidated COFF report, with the captured IL beside it.

The loop every session in this repo runs by hand is: write a tiny C++ probe,
compile it with the real toolchain, then read the obj's sections, symbols and
relocations and cross-reference the three by eye. `coffdump.py` prints those
three tables *separately* and prints relocations by raw symbol index, so the
cross-referencing is manual and gets redone every time. This does it once:

  * section table, in section order;
  * under each section, every symbol that section DEFINES (Value, inferred
    size, storage class) -- so "which of the six `.text$yc` sections is this
    symbol in" is answered by where it is printed;
  * under each section, its relocations rendered **by target symbol name**;
  * then the leftovers a section cannot own: undefined externals, absolutes,
    and debug/section-less symbols.

It is NOT the correctness judge -- that is `c2rs diff` / `crates/c2-obj`, the
byte-exact compare with `TimeDateStamp` zeroed. This is an instrument.

Usage:
  probe.py <file.cpp> [options]
  probe.py - [options]                     read the source from stdin
  probe.py --source 'int f(){return 1;}'   inline source text
  probe.py --selftest

Options:
  --flags '/O1 /Oi /c'   compile flags (default: /O1 /Oi /EHsc /GS- /c)
  --flags-file FILE      read flags from a file instead (one or many per line)
  --cwd DIR              working directory for the compile (project includes)
  --work DIR             scratch dir (default <repo>/work/probe)
  --no-il                skip the IL capture
  --keep                 do not delete the scratch obj/IL afterwards (default:
                         kept -- this flag exists for symmetry; see --clean)
  --clean                delete the scratch dir for this probe when done
  --aux                  also show COMDAT selection / section aux records
  --hex SYM              hexdump one symbol's bytes as well

Toolchain absent -> prints `SKIP: toolchain absent` and exits 0, like `c2rs`.

# A trap this tool reports rather than hides

`c2rs compile` and `c2rs capture` do **not** use the same default flags:
`compile` uses `/O1 /Oi /EHsc /GS- /c` (and honours `--flags-file`), while
`capture` is hardcoded to `/Ox /GS- /c` and accepts no flags at all. `/Ox` does
not imply `/GF` where `/O1` and `/O2` do, so a TU with a string literal captures
IL with **no `??_C@` record at all** while its obj carries a `.rdata` COMDAT for
one (see `crates/c2-il/src/func/gl.rs::gl_string_comdat_names`). Whenever the
requested compile flags are not exactly capture's, this prints a FLAG SKEW
banner and, if the obj/IL actually disagree about string COMDATs, says so
concretely. Do not correlate `.gl` against an obj across a skew without reading
that banner.
"""
import argparse
import os
import subprocess
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
if _HERE not in sys.path:
    sys.path.insert(0, _HERE)

import coffdump  # noqa: E402  (same directory, deliberate)

DEFAULT_FLAGS = "/O1 /Oi /EHsc /GS- /c"
# What `Toolchain::capture_il` hardcodes; see the module docstring.
CAPTURE_FLAGS = "/Ox /GS- /c"
SKIP_LINE = "SKIP: toolchain absent"

STORAGE_CLASS = {
    0: "NULL", 1: "AUTOMATIC", 2: "EXTERNAL", 3: "STATIC", 4: "REGISTER",
    5: "EXTERNAL_DEF", 6: "LABEL", 7: "UNDEF_LABEL", 8: "MEM_OF_STRUCT",
    9: "ARGUMENT", 10: "STRUCT_TAG", 11: "MEM_OF_UNION", 12: "UNION_TAG",
    13: "TYPEDEF", 14: "UNDEF_STATIC", 15: "ENUM_TAG", 16: "MEM_OF_ENUM",
    17: "REG_PARAM", 18: "BIT_FIELD", 100: "BLOCK", 101: "FUNCTION",
    102: "END_OF_STRUCT", 103: "FILE", 104: "SECTION", 105: "WEAK_EXTERNAL",
    107: "CLR_TOKEN",
}

# PPC relocation types (winnt.h IMAGE_REL_PPC_*). The set c2 actually emits is
# small -- REFHI/REFLO/PAIR for a 32-bit address split across two instructions,
# REL24 for a call, ADDR32/ADDR32NB for a data word, SECREL/SECTION for debug.
RELOC_TYPE = {
    0x0000: "ABSOLUTE", 0x0001: "ADDR64", 0x0002: "ADDR32", 0x0003: "ADDR24",
    0x0004: "ADDR16", 0x0005: "ADDR14", 0x0006: "REL24", 0x0007: "REL14",
    0x000A: "ADDR32NB", 0x000B: "SECREL", 0x000C: "SECTION",
    0x000F: "SECREL16", 0x0010: "REFHI", 0x0011: "REFLO", 0x0012: "PAIR",
    0x0013: "SECRELLO", 0x0015: "GPREL", 0x0016: "TOKEN",
}
RELOC_PAIR = 0x0012

SCN_FLAGS = [
    (0x00000020, "CNT_CODE"), (0x00000040, "CNT_INITIALIZED_DATA"),
    (0x00000080, "CNT_UNINITIALIZED_DATA"), (0x00000200, "LNK_INFO"),
    (0x00000800, "LNK_REMOVE"), (0x00001000, "LNK_COMDAT"),
    (0x02000000, "MEM_DISCARDABLE"), (0x04000000, "MEM_NOT_CACHED"),
    (0x08000000, "MEM_NOT_PAGED"), (0x10000000, "MEM_SHARED"),
    (0x20000000, "MEM_EXECUTE"), (0x40000000, "MEM_READ"),
    (0x80000000, "MEM_WRITE"),
]
COMDAT_SELECT = {
    1: "NODUPLICATES", 2: "ANY", 3: "SAME_SIZE", 4: "EXACT_MATCH",
    5: "ASSOCIATIVE", 6: "LARGEST",
}


def decode_chars(ch):
    """Human-readable IMAGE_SCN_* set plus the alignment nibble."""
    out = [name for bit, name in SCN_FLAGS if ch & bit]
    align = (ch >> 20) & 0xF
    if align:
        out.append("ALIGN_%d" % (1 << (align - 1)))
    return "|".join(out) if out else "0"


# ---------------------------------------------------------------------------
# Repo / toolchain plumbing. Shared with glorder.py -- keep importable.
# ---------------------------------------------------------------------------

def repo_root():
    """The repo root, resolved from THIS FILE's location (tools/..).

    Never an absolute machine path in source; `C2RS_ROOT` overrides for the
    case where the tools are copied elsewhere.
    """
    env = os.environ.get("C2RS_ROOT")
    if env:
        return os.path.abspath(env)
    return os.path.dirname(_HERE)


def find_c2rs():
    """The `c2rs` binary: $C2RS_BIN, else target/release, else target/debug."""
    env = os.environ.get("C2RS_BIN")
    if env:
        return env if os.path.exists(env) else None
    for rel in ("target/release/c2rs", "target/debug/c2rs"):
        p = os.path.join(repo_root(), rel)
        if os.path.exists(p):
            return p
    return None


class Skip(Exception):
    """The toolchain is absent. Callers print SKIP and exit 0."""


class Fail(Exception):
    """A real failure -- nonzero exit, never a silent empty success."""


def _run(argv, cwd=None):
    try:
        p = subprocess.run(argv, cwd=cwd, capture_output=True, text=True)
    except OSError as e:
        raise Fail("cannot execute %s: %s" % (argv[0], e))
    blob = (p.stdout or "") + (p.stderr or "")
    if SKIP_LINE in blob:
        raise Skip(SKIP_LINE)
    return p, blob


def run_compile(c2rs, src, obj, flags_file=None, cwd=None):
    """`c2rs compile` -> obj at `obj`. Raises Skip / Fail; never returns a
    missing obj as success (the absence-reads-as-zero failure mode)."""
    argv = [c2rs, "compile", src]
    if flags_file:
        argv += ["--flags-file", flags_file]
    if cwd:
        argv += ["--cwd", cwd]
    argv += ["--keep-obj", obj]
    p, blob = _run(argv, cwd=None)
    if p.returncode != 0:
        raise Fail("c2rs compile failed (exit %d):\n%s" % (p.returncode, blob.strip()))
    if not os.path.exists(obj) or os.path.getsize(obj) == 0:
        raise Fail("c2rs compile reported success but produced no obj at %s\n%s"
                   % (obj, blob.strip()))
    return blob


def run_capture(c2rs, src, il_dir):
    """`c2rs capture --keep-il` -> the bundle base path (no suffix).

    NOTE the flag skew documented at the top of this file: capture is pinned to
    `/Ox /GS- /c`.
    """
    os.makedirs(il_dir, exist_ok=True)
    p, blob = _run([c2rs, "capture", src, "--keep-il", il_dir])
    if p.returncode != 0:
        raise Fail("c2rs capture failed (exit %d):\n%s" % (p.returncode, blob.strip()))
    bases = sorted({os.path.splitext(f)[0] for f in os.listdir(il_dir)
                    if f.startswith("_CL_")})
    if not bases:
        raise Fail("c2rs capture reported success but wrote no _CL_* bundle in %s\n%s"
                   % (il_dir, blob.strip()))
    # Newest bundle wins if the dir was reused.
    bases.sort(key=lambda b: os.path.getmtime(os.path.join(il_dir, b + ".gl")))
    return os.path.join(il_dir, bases[-1])


def read_flags_file(path):
    """Same rule `c2rs compile --flags-file` uses: blank lines and `#` comment
    lines dropped, everything else split on whitespace."""
    out = []
    with open(path) as f:
        for line in f:
            t = line.strip()
            if not t or t.startswith("#"):
                continue
            out.extend(t.split())
    return out


def resolve_source(args, work):
    """Turn (path | '-' | --source TEXT) into a concrete file path."""
    if args.source is not None:
        os.makedirs(work, exist_ok=True)
        path = os.path.join(work, "probe.cpp")
        text = args.source
        if not text.endswith("\n"):
            text += "\n"
        with open(path, "w") as f:
            f.write(text)
        return path
    if args.file is None:
        raise Fail("probe: expected a <file.cpp>, '-', or --source TEXT")
    if args.file == "-":
        os.makedirs(work, exist_ok=True)
        path = os.path.join(work, "probe.cpp")
        text = sys.stdin.read()
        if not text.strip():
            raise Fail("probe: empty source on stdin")
        with open(path, "w") as f:
            f.write(text)
        return path
    if not os.path.exists(args.file):
        raise Fail("probe: no such source file: %s" % args.file)
    if os.path.getsize(args.file) == 0:
        raise Fail("probe: source file is empty: %s" % args.file)
    return args.file


# ---------------------------------------------------------------------------
# COFF aux records -- coffdump's reader drops them, and COMDAT selection is
# exactly what distinguishes six same-named `.text$yc` sections.
# ---------------------------------------------------------------------------

def section_aux(path):
    """symbol index -> dict of the IMAGE_AUX_SYMBOL_SECTION fields."""
    import struct
    with open(path, "rb") as f:
        data = f.read()
    if len(data) < 20:
        return {}
    _m, _nsec, _tds, symoff, nsym, _opt, _ch = struct.unpack_from("<HHIIIHH", data, 0)
    out = {}
    i = 0
    while i < nsym:
        off = symoff + i * 18
        if off + 18 > len(data):
            break
        cls = data[off + 16]
        naux = data[off + 17]
        if cls == 3 and naux >= 1 and off + 36 <= len(data):
            a = off + 18
            length, nrel, nlin, cks, num, sel = struct.unpack_from("<IHHIHB", data, a)
            out[i] = {"length": length, "nrel": nrel, "nlin": nlin,
                      "checksum": cks, "number": num, "sel": sel}
        i += 1 + naux
    return out


# ---------------------------------------------------------------------------
# The report
# ---------------------------------------------------------------------------

def report(obj_path, il_base, flags, show_aux, hex_sym, out=sys.stdout):
    secs, syms = coffdump.load(obj_path)
    aux = section_aux(obj_path) if show_aux else {}
    by_idx = {s.index: s for s in syms}
    w = out.write

    w("== obj %s (%d bytes) ==\n" % (obj_path, os.path.getsize(obj_path)))
    w("   flags: %s\n" % flags)
    w("   %d sections, %d symbol records\n\n" % (len(secs), len(syms)))

    w("== section table ==\n")
    w("%3s  %-14s %8s %8s %6s  %s\n"
      % ("idx", "name", "rawsize", "vsize", "nrel", "characteristics"))
    for s in secs:
        w("%3d  %-14s %8d %8d %6d  %s\n"
          % (s.index, s.name, s.rawsize, s.vsize, s.nrel, decode_chars(s.chars)))
    w("\n")

    # Group defined symbols by their section, in section-table order.
    defined = {}
    for s in syms:
        if 0 < s.sec <= len(secs):
            defined.setdefault(s.sec - 1, []).append(s)

    for s in secs:
        members = defined.get(s.index, [])
        w("== [%d] %s  --  %d B, %d sym, %d reloc ==\n"
          % (s.index, s.name, s.rawsize, len(members), s.nrel))
        if members:
            w("  symbols (by Value, then symbol index):\n")
            w("  %6s %5s %8s %8s  %-13s %s\n"
              % ("symidx", "kind", "value", "size", "storage", "name"))
            for m in sorted(members, key=lambda x: (x.value, x.index)):
                sc = STORAGE_CLASS.get(m.cls, str(m.cls))
                extra = ""
                if show_aux and m.index in aux:
                    a = aux[m.index]
                    sel = COMDAT_SELECT.get(a["sel"], str(a["sel"]))
                    extra = ("   [aux len=%d nrel=%d cks=0x%08x num=%d sel=%s]"
                             % (a["length"], a["nrel"], a["checksum"], a["number"], sel))
                w("  %6d %5s %8d %8d  %-13s %s%s\n"
                  % (m.index, m.kind, m.value, m.size, sc, m.name, extra))
        if s.relocs:
            w("  relocations (by target symbol NAME):\n")
            w("  %8s  %-10s %6s  %s\n" % ("va", "type", "symidx", "target"))
            for (va, symidx, typ) in s.relocs:
                t = by_idx.get(symidx)
                if typ == RELOC_PAIR:
                    # IMAGE_REL_PPC_PAIR's SymbolTableIndex field is NOT a
                    # symbol index -- it carries the other half's displacement
                    # for the preceding REFHI/REFLO. Rendering it by name reads
                    # it as `@comp.id` (symbol 0) on every single c2 obj, which
                    # is a lie the eye will start believing.
                    w("  %8d  %-10s %6s  (not a symbol: displacement %d)\n"
                      % (va, "PAIR", "-", symidx))
                    continue
                if t is None:
                    tname = "?? (symidx %d out of range)" % symidx
                elif 0 < t.sec <= len(secs):
                    tname = "%s  [in %s +%d]" % (t.name, secs[t.sec - 1].name, t.value)
                elif t.sec == 0:
                    tname = "%s  [UNDEFINED]" % t.name
                else:
                    tname = "%s  [sec %d]" % (t.name, t.sec)
                w("  %8d  %-10s %6d  %s\n"
                  % (va, RELOC_TYPE.get(typ, "0x%04x" % typ), symidx, tname))
        if not members and not s.relocs:
            w("  (no symbols, no relocations)\n")
        w("\n")

    undef = [s for s in syms if s.sec == 0 and s.name]
    w("== undefined externals (%d) ==\n" % len(undef))
    for s in undef:
        w("  %6d  %-13s %s\n" % (s.index, STORAGE_CLASS.get(s.cls, str(s.cls)), s.name))
    if not undef:
        w("  (none)\n")
    w("\n")

    other = [s for s in syms if s.sec < 0]
    w("== absolute / debug symbols (%d) ==\n" % len(other))
    for s in other:
        w("  %6d  sec=%-4d %-13s %s = %d (0x%x)\n"
          % (s.index, s.sec, STORAGE_CLASS.get(s.cls, str(s.cls)),
             s.name, s.value, s.value))
    if not other:
        w("  (none)\n")
    w("\n")

    if hex_sym:
        sym = coffdump.find_symbol(syms, hex_sym)
        w("== bytes of %s ==\n" % sym.name)
        if 0 < sym.sec <= len(secs):
            b = coffdump.funclet_signature(secs[sym.sec - 1], sym, False)
            w((coffdump.hexdump(b) if b else "  (zero size)") + "\n\n")
        else:
            w("  (no section-relative body)\n\n")

    if il_base:
        w("== captured IL (%s) ==\n" % CAPTURE_FLAGS)
        for suf in ("ex", "gl", "sy", "in", "db"):
            p = il_base + "." + suf
            w("  .%-2s %8d B  %s\n"
              % (suf, os.path.getsize(p) if os.path.exists(p) else 0, p))
        w("  (record order: tools/glorder.py %s.gl --obj %s)\n\n"
          % (il_base, obj_path))
        skew_note(flags, il_base + ".gl", syms, out)


def skew_note(flags, gl_path, syms, out):
    """State the compile/capture flag skew, and check the one consequence that
    is decidable from the two files we already have."""
    if set(flags.split()) == set(CAPTURE_FLAGS.split()):
        return
    w = out.write
    w("!! FLAG SKEW: obj compiled at `%s`, IL captured at `%s`.\n" % (flags, CAPTURE_FLAGS))
    w("   `c2rs capture` takes no flags (Toolchain::capture_il is pinned).\n")
    try:
        with open(gl_path, "rb") as f:
            gl = f.read()
    except OSError:
        return
    gl_has = b"??_C@" in gl
    obj_has = any(s.name.startswith("??_C@") for s in syms)
    if obj_has and not gl_has:
        w("   CONFIRMED consequence: the obj carries ??_C@ string COMDATs and the\n")
        w("   captured .gl carries NONE (/GF is implied by /O1 and /O2, not /Ox).\n")
        w("   Do NOT correlate this .gl against this obj for string literals.\n")
    elif obj_has == gl_has:
        w("   (string-COMDAT presence agrees between the two: %s)\n"
          % ("both present" if obj_has else "both absent"))


# ---------------------------------------------------------------------------

def selftest():
    """Prove the tool FAILS when it should. Runs with no toolchain.

    Every check below is a case where an earlier hand-rolled reader would have
    reported an empty success -- a missing file read as "no symbols", a
    truncated obj read as "no sections". An instrument that is green on an
    absence grades nothing.
    """
    import io
    import struct
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
        except BaseException as e:  # SystemExit included, deliberately
            raise AssertionError("%s raised %r, wanted %s" % (what, e, exc))
        raise AssertionError("%s returned normally; wanted %s -- an empty "
                             "success here is exactly the failure mode this "
                             "selftest exists for" % (what, exc))

    d = tempfile.mkdtemp(prefix="probe-selftest-")

    def missing_file():
        # OSError from the open, or SystemExit from the COFF check -- either is
        # an error. What must NOT happen is `([], [])`, an empty report.
        p = os.path.join(d, "nope.obj")
        expect_raises((SystemExit, OSError), lambda: coffdump.load(p),
                      "load(missing obj)")
    check("missing obj is an error, not an empty report", missing_file)

    def empty_file():
        p = os.path.join(d, "empty.obj")
        open(p, "wb").close()
        expect_raises(SystemExit, lambda: coffdump.load(p), "load(empty obj)")
    check("empty obj is an error, not zero sections", empty_file)

    def truncated():
        p = os.path.join(d, "trunc.obj")
        # A well-formed 20-byte header claiming 3 sections, with no section
        # table behind it. A reader that trusted the count would report three
        # empty sections; this must refuse.
        with open(p, "wb") as f:
            f.write(struct.pack("<HHIIIHH", 0x1F2, 3, 0, 200, 5, 0, 0))
        expect_raises(SystemExit, lambda: coffdump.load(p), "load(truncated obj)")
    check("truncated section table is refused", truncated)

    def no_symtab():
        p = os.path.join(d, "nosym.obj")
        with open(p, "wb") as f:
            f.write(struct.pack("<HHIIIHH", 0x1F2, 0, 0, 0, 0, 0, 0))
        expect_raises(SystemExit, lambda: coffdump.load(p), "load(obj with no symtab)")
    check("obj with no symbol table is refused", no_symtab)

    def bad_source():
        class A:
            source, file = None, os.path.join(d, "no-such.cpp")
        expect_raises(Fail, lambda: resolve_source(A(), d), "resolve_source(missing)")

        class B:
            source, file = None, None
        expect_raises(Fail, lambda: resolve_source(B(), d), "resolve_source(nothing)")

        class C:
            source, file = None, os.path.join(d, "zero.cpp")
        open(C.file, "w").close()
        expect_raises(Fail, lambda: resolve_source(C(), d), "resolve_source(empty file)")
    check("missing/absent/empty source is an error", bad_source)

    def bogus_c2rs():
        expect_raises(Fail, lambda: _run([os.path.join(d, "not-a-binary")]),
                      "_run(nonexistent binary)")
    check("un-executable c2rs is an error", bogus_c2rs)

    def skip_is_skip():
        sh = os.path.join(d, "fake-c2rs")
        with open(sh, "w") as f:
            f.write("#!/bin/sh\necho '%s'\n" % SKIP_LINE)
        os.chmod(sh, 0o755)
        expect_raises(Skip, lambda: _run([sh]), "_run(toolchain-absent stub)")
    check("`SKIP: toolchain absent` becomes a Skip, not a Fail", skip_is_skip)

    def compile_lying():
        # The exact absence-reads-as-success shape: exit 0, cheerful message,
        # no obj on disk.
        sh = os.path.join(d, "liar-c2rs")
        with open(sh, "w") as f:
            f.write("#!/bin/sh\necho 'compiled x -> 2140 bytes'\nexit 0\n")
        os.chmod(sh, 0o755)
        expect_raises(Fail,
                      lambda: run_compile(sh, "x.cpp", os.path.join(d, "never.obj")),
                      "run_compile(binary that writes no obj)")
    check("exit-0-with-no-obj is a failure, not a pass", compile_lying)

    def capture_lying():
        sh = os.path.join(d, "liar2-c2rs")
        with open(sh, "w") as f:
            f.write("#!/bin/sh\necho 'captured IL bundle _CL_dead'\nexit 0\n")
        os.chmod(sh, 0o755)
        empty = os.path.join(d, "il-empty")
        expect_raises(Fail, lambda: run_capture(sh, "x.cpp", empty),
                      "run_capture(binary that writes no bundle)")
    check("capture with no _CL_* bundle on disk is a failure", capture_lying)

    def positive():
        # A hand-built minimal COFF must parse AND report its one symbol under
        # its section -- the tool has to be capable of a true positive, or the
        # failure checks above prove only that it never works.
        p = os.path.join(d, "good.obj")
        nsec, nsym = 1, 1
        symoff = 20 + 40 + 4
        hdr = struct.pack("<HHIIIHH", 0x1F2, nsec, 0, symoff, nsym, 0, 0)
        sec = (b".text\0\0\0" + struct.pack("<IIIIIIHH", 4, 0, 4, 20 + 40, 0, 0, 0, 0)
               + struct.pack("<I", 0x60000020))
        body = b"\x38\x60\x00\x00"
        sym = b"f\0\0\0\0\0\0\0" + struct.pack("<IhHBB", 0, 1, 0x20, 2, 0)
        blob = hdr + sec + body + sym + struct.pack("<I", 4)
        with open(p, "wb") as f:
            f.write(blob)
        secs, syms = coffdump.load(p)
        assert len(secs) == 1 and secs[0].name == ".text", "section table wrong"
        assert len(syms) == 1 and syms[0].name == "f", "symbol table wrong"
        buf = io.StringIO()
        report(p, None, DEFAULT_FLAGS, True, None, out=buf)
        text = buf.getvalue()
        assert "== [0] .text" in text, "section heading missing"
        assert "f" in text and "EXTERNAL" in text, "symbol not reported under section"
        assert "== undefined externals (0) ==" in text, "leftover block missing"
    check("a valid minimal obj parses and reports (true positive)", positive)

    def reloc_named():
        # A relocation must render by NAME. A reader that silently printed the
        # raw index would pass every check above.
        p = os.path.join(d, "reloc.obj")
        nsec, nsym = 1, 1
        secdata_off = 20 + 40
        relptr = secdata_off + 4
        symoff = relptr + 10
        hdr = struct.pack("<HHIIIHH", 0x1F2, nsec, 0, symoff, nsym, 0, 0)
        sec = (b".text\0\0\0" + struct.pack("<IIIIIIHH", 4, 0, 4, secdata_off,
                                            relptr, 0, 1, 0)
               + struct.pack("<I", 0x60000020))
        body = b"\x48\x00\x00\x01"
        rel = struct.pack("<IIH", 0, 0, 6)
        sym = b"tgt\0\0\0\0\0" + struct.pack("<IhHBB", 0, 1, 0x20, 2, 0)
        with open(p, "wb") as f:
            f.write(hdr + sec + body + rel + sym + struct.pack("<I", 4))
        buf = io.StringIO()
        report(p, None, DEFAULT_FLAGS, False, None, out=buf)
        text = buf.getvalue()
        assert "REL24" in text, "relocation type not decoded"
        assert "tgt" in text.split("relocations")[1], "relocation not named"
    check("relocations render by target name, with a decoded type", reloc_named)

    def skew_detected():
        class S:
            def __init__(self, n):
                self.name = n
        gl = os.path.join(d, "nostr.gl")
        with open(gl, "wb") as f:
            f.write(b"\x11\x02\x061j2\x01" + b"\0" * 32)
        buf = io.StringIO()
        skew_note(DEFAULT_FLAGS, gl, [S("??_C@_02ABCD@a1?$AA@")], buf)
        assert "CONFIRMED consequence" in buf.getvalue(), \
            "the /GF skew was not detected when obj has ??_C@ and .gl does not"
        buf2 = io.StringIO()
        skew_note(CAPTURE_FLAGS, gl, [S("??_C@_02ABCD@a1?$AA@")], buf2)
        assert buf2.getvalue() == "", "reported a skew when the flags agree"
    check("the /GF flag skew is detected from the two files", skew_detected)

    print("\n%d/%d checks passed" % (sum(ok), len(ok)))
    return 0 if all(ok) else 1


def main():
    p = argparse.ArgumentParser(
        prog="probe", description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("file", nargs="?")
    p.add_argument("--source")
    p.add_argument("--flags")
    p.add_argument("--flags-file")
    p.add_argument("--cwd")
    p.add_argument("--work")
    p.add_argument("--no-il", action="store_true")
    p.add_argument("--keep", action="store_true")
    p.add_argument("--clean", action="store_true")
    p.add_argument("--aux", action="store_true")
    p.add_argument("--hex")
    p.add_argument("--selftest", action="store_true")
    args = p.parse_args()

    if args.selftest:
        return selftest()

    work = args.work or os.path.join(repo_root(), "work", "probe")
    try:
        src = resolve_source(args, work)
        c2rs = find_c2rs()
        if c2rs is None:
            raise Fail("probe: no c2rs binary -- build it with\n"
                       "  cargo build --release -p c2-harness\n"
                       "or point $C2RS_BIN at one.")

        # Flags: --flags-file wins, then --flags, then the default. When
        # --flags is given we materialize a flags file, because that is the
        # only path `c2rs compile` has for a non-default profile.
        os.makedirs(work, exist_ok=True)
        flags_file = args.flags_file
        if flags_file:
            if not os.path.exists(flags_file):
                raise Fail("probe: no such --flags-file: %s" % flags_file)
            flags = " ".join(read_flags_file(flags_file))
            if not flags:
                raise Fail("probe: --flags-file %s contains no flags" % flags_file)
        else:
            flags = args.flags or DEFAULT_FLAGS
            flags_file = os.path.join(work, "flags.txt")
            with open(flags_file, "w") as f:
                f.write(flags + "\n")

        obj = os.path.join(work, os.path.splitext(os.path.basename(src))[0] + ".obj")
        run_compile(c2rs, src, obj, flags_file=flags_file, cwd=args.cwd)

        il_base = None
        if not args.no_il:
            il_base = run_capture(c2rs, src, os.path.join(work, "il"))

        report(obj, il_base, flags, args.aux, args.hex)

        if args.clean:
            import shutil
            shutil.rmtree(work, ignore_errors=True)
        return 0
    except Skip:
        print(SKIP_LINE)
        return 0
    except Fail as e:
        print(str(e), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
