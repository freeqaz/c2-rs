#!/usr/bin/env python3
"""xcheck — decode Xbox 360 PPC COFF objs with LLVM *and* with this repo's own
readers, and report every field where they disagree.

Why this exists: `crates/c2-obj`, `scripts/gt_dump.py` and `tools/coffdump.py`
are all *ours*. Until now every question about what an obj contains was settled
by code written by the same project that is under test. `llvm-readobj` is an
outside implementation of the same COFF parse, so it can disagree with us — and
a disagreement is a bug in exactly one of the two.

    tools/llvm/xcheck.py <obj>...            # human-readable
    tools/llvm/xcheck.py --tsv out.tsv <obj>...

Output ends with a POSITIVE claim -- "compared N objs / M field instances / K
disagreements". A run that compared nothing exits non-zero and says so; this
repo's most common recorded defect is a green report produced by grading
nothing (docs/BOARD.md, 14 recorded instances).

Fields compared, per obj:
  header    machine, section count, timestamp, symbol-table pointer,
            symbol count, optional-header size, characteristics       (7)
  section   name, raw 8-byte name field, vsize, vaddr, rawsize, rawptr,
            relptr, nreloc, characteristics                           (9 x nsec)
  symbol    name, value, section number, type word, storage class,
            aux count                                                 (6 x nsym)
  aux       Length, RelocationCount, LineNumberCount, Checksum,
            Number, Selection  (section-definition records only)      (6 x naux)
  reloc     offset, type word, symbol index                           (3 x nrel)

LLVM is compared against `tools/coffdump.py`'s reader on all of the above, and
against `crates/c2-obj` (via tools/llvm/c2objdump, when cargo is available) on
the two lists that the port's metrics actually consume: the section-name
sequence and the `.text` COMDAT leader sequence. `scripts/gt_dump.py` is
compared as a third opinion on sections, symbols and relocations.
"""

import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "tools"))
sys.path.insert(0, os.path.join(REPO, "scripts"))

import llvmpath  # noqa: E402
import readobj_parse as R  # noqa: E402

try:
    import coffdump  # tools/coffdump.py -- this lane's seam
except ImportError:
    coffdump = None
try:
    import gt_dump  # scripts/gt_dump.py -- read-only import, never modified here
except ImportError:
    gt_dump = None

IMAGE_SCN_LNK_COMDAT = 0x00001000
IMAGE_SYM_CLASS_STATIC = 3

# --selftest: perturb OUR side only, after decode, and require the comparison to
# notice. Corrupting the obj itself proves nothing here -- both readers would
# read the same corrupted bytes and still agree, which is exactly the trap this
# switch exists to close. See docs/rungs/_2026-08-04-w-llvm.md.
SELFTEST = False


class Tally:
    """Counts comparisons and disagreements. Cannot be satisfied by silence."""

    def __init__(self):
        self.compared = 0
        self.diffs = []
        self.per_field = {}

    def eq(self, obj, where, field, ours, theirs, ours_name="c2-rs", theirs_name="llvm"):
        self.compared += 1
        self.per_field[field] = self.per_field.get(field, 0) + 1
        if ours != theirs:
            self.diffs.append((obj, where, field, ours_name, ours, theirs_name, theirs))
            return False
        return True


def llvm_decode(llvm, obj, scratch):
    """Everything llvm-readobj will tell us about one obj, as plain dicts."""
    path = llvm.readable(obj, scratch)
    out, err, rc = llvm.run(
        "llvm-readobj",
        ["--file-headers", "--sections", "--section-relocations", "--expand-relocs",
         "--symbols", path],
    )
    if rc != 0 or "ImageFileHeader" not in out:
        raise RuntimeError("llvm-readobj failed on %s: rc=%d %s" % (obj, rc, err.strip()[:200]))
    root = R.parse(out)

    hdr_node = R.find_all(root, "ImageFileHeader")[0]
    chars = None
    for k, v in hdr_node.pairs:
        if k == "Characteristics" and isinstance(v, R.Node):
            chars = v.raw
    hdr = {
        "machine": hdr_node.num("Machine"),
        "nsec": hdr_node.num("SectionCount"),
        "timestamp": hdr_node.num("TimeDateStamp"),
        "symptr": hdr_node.num("PointerToSymbolTable"),
        "nsym": hdr_node.num("SymbolCount"),
        "optsz": hdr_node.num("OptionalHeaderSize"),
        "chars": chars,
    }

    secs = []
    for s in R.find_all(root, "Section"):
        c = None
        rels = []
        for k, v in s.pairs:
            if k == "Characteristics" and isinstance(v, R.Node):
                c = v.raw
            if k == "Relocations" and isinstance(v, R.Node):
                for r in v.all("Relocation"):
                    rels.append(
                        {
                            "off": r.num("Offset"),
                            "type": r.num("Type"),
                            "sym": r.num("SymbolIndex"),
                            "symname": r.name("Symbol"),
                        }
                    )
        secs.append(
            {
                "number": s.num("Number"),
                "name": s.name("Name"),
                "rawname": s.rawname("Name"),
                "vsize": s.num("VirtualSize"),
                "vaddr": s.num("VirtualAddress"),
                "rawsize": s.num("RawDataSize"),
                "rawptr": s.num("PointerToRawData"),
                "relptr": s.num("PointerToRelocations"),
                "nrel": s.num("RelocationCount"),
                "chars": c,
                "relocs": rels,
            }
        )

    syms = []
    for y in R.find_all(root, "Symbol"):
        base = y.num("BaseType")
        cplx = y.num("ComplexType")
        aux = []
        for a in y.all("AuxSectionDef"):
            aux.append(
                {
                    "length": a.num("Length"),
                    "nrel": a.num("RelocationCount"),
                    "nln": a.num("LineNumberCount"),
                    "cksum": a.num("Checksum"),
                    "number": a.num("Number"),
                    "sel": a.num("Selection"),
                }
            )
        syms.append(
            {
                "name": y.name("Name"),
                "value": y.num("Value"),
                "sec": y.num("Section"),
                "type": (cplx << 4) | base if base is not None and cplx is not None else None,
                "sc": y.num("StorageClass"),
                "naux": y.num("AuxSymbolCount"),
                "aux": aux,
            }
        )
    return hdr, secs, syms, path


def coffdump_decode(obj):
    data = open(obj, "rb").read()
    secs, syms = coffdump.read_coff(data)
    if secs is None:
        raise RuntimeError("coffdump.read_coff refused %s" % obj)
    import struct

    mach, nsec, tds, symoff, nsym, optsz, ch = struct.unpack_from("<HHIIIHH", data, 0)
    hdr = {
        "machine": mach, "nsec": nsec, "timestamp": tds, "symptr": symoff,
        "nsym": nsym, "optsz": optsz, "chars": ch,
    }
    return hdr, secs, syms


def aux_secdef(data, symoff, sym_index):
    """Decode the section-definition aux record following symbol `sym_index`."""
    import struct

    o = symoff + 18 * (sym_index + 1)
    return {
        "length": struct.unpack_from("<I", data, o)[0],
        "nrel": struct.unpack_from("<H", data, o + 4)[0],
        "nln": struct.unpack_from("<H", data, o + 6)[0],
        "cksum": struct.unpack_from("<I", data, o + 8)[0],
        "number": struct.unpack_from("<H", data, o + 12)[0],
        "sel": data[o + 14],
    }


def check_obj(llvm, obj, scratch, t, tg):
    """Compare one obj. Returns (nsec, nsym, nrel) actually compared."""
    import struct

    lh, ls, ly, used_path = llvm_decode(llvm, obj, scratch)
    ch, cs, cy = coffdump_decode(obj)
    if SELFTEST:
        # One field, one reader, off by one -- the comparison must report
        # exactly `len(cs)` disagreements and no more.
        for c in cs:
            c.rawsize += 1
    data = open(obj, "rb").read()
    name = os.path.basename(obj)

    # --- header -------------------------------------------------------------
    # The machine word is the one field the scratch copy deliberately changes,
    # so it is compared against what the *scratch* should hold, not the source.
    expect_machine = ch["machine"] if llvm.native_ppcbe else llvmpath.MACHINE_POWERPC
    t.eq(name, "header", "machine", expect_machine, lh["machine"])
    for f in ("nsec", "timestamp", "symptr", "nsym", "optsz", "chars"):
        t.eq(name, "header", f, ch[f], lh[f])

    # --- sections -----------------------------------------------------------
    t.eq(name, "sections", "count", len(cs), len(ls))
    for i, (c, l) in enumerate(zip(cs, ls)):
        w = "sec[%d]" % (i + 1)
        t.eq(name, w, "sec.name", c.name, l["name"])
        raw = data[20 + i * 40 : 20 + i * 40 + 8]
        if l["rawname"] is not None:
            t.eq(name, w, "sec.rawname", raw.hex(), l["rawname"].hex())
        t.eq(name, w, "sec.vsize", c.vsize, l["vsize"])
        t.eq(name, w, "sec.vaddr", c.vaddr, l["vaddr"])
        t.eq(name, w, "sec.rawsize", c.rawsize, l["rawsize"])
        t.eq(name, w, "sec.rawptr", c.rawptr, l["rawptr"])
        t.eq(name, w, "sec.relptr", c.relptr, l["relptr"])
        t.eq(name, w, "sec.nrel", c.nrel, l["nrel"])
        t.eq(name, w, "sec.chars", c.chars, l["chars"])

    # --- symbols ------------------------------------------------------------
    t.eq(name, "symbols", "count", len(cy), len(ly))
    for i, (c, l) in enumerate(zip(cy, ly)):
        w = "sym[%d]" % c.index
        t.eq(name, w, "sym.name", c.name, l["name"])
        t.eq(name, w, "sym.value", c.value, l["value"])
        t.eq(name, w, "sym.sec", c.sec, l["sec"])
        t.eq(name, w, "sym.type", c.typ, l["type"])
        t.eq(name, w, "sym.sc", c.cls, l["sc"])
        t.eq(name, w, "sym.naux", c.naux, l["naux"])
        # Aux section-definition records: llvm prints one AuxSectionDef per
        # aux entry it recognises as such.
        if l["aux"] and c.naux >= 1:
            ours = aux_secdef(data, ch["symptr"], c.index)
            theirs = l["aux"][0]
            for f in ("length", "nrel", "nln", "cksum", "number", "sel"):
                t.eq(name, w, "aux." + f, ours[f], theirs[f])

    # --- relocations --------------------------------------------------------
    nrel_total = 0
    for i, (c, l) in enumerate(zip(cs, ls)):
        w = "sec[%d]" % (i + 1)
        t.eq(name, w, "rel.count", len(c.relocs), len(l["relocs"]))
        for j, (cr, lr) in enumerate(zip(c.relocs, l["relocs"])):
            t.eq(name, "%s.rel[%d]" % (w, j), "rel.off", cr[0], lr["off"])
            t.eq(name, "%s.rel[%d]" % (w, j), "rel.sym", cr[1], lr["sym"])
            t.eq(name, "%s.rel[%d]" % (w, j), "rel.type", cr[2], lr["type"])
            nrel_total += 1

    # --- third opinion: scripts/gt_dump.py ----------------------------------
    if gt_dump is not None:
        g = gt_dump.Obj(data)
        tg.eq(name, "header", "nsec", g.nsec, lh["nsec"], "gt_dump")
        tg.eq(name, "header", "nsym", g.nsym, lh["nsym"], "gt_dump")
        tg.eq(name, "header", "timestamp", g.timestamp, lh["timestamp"], "gt_dump")
        for i, (gs, l) in enumerate(zip(g.sections, ls)):
            w = "sec[%d]" % (i + 1)
            tg.eq(name, w, "sec.name", gs["name"], l["name"], "gt_dump")
            tg.eq(name, w, "sec.rawsize", gs["rawsize"], l["rawsize"], "gt_dump")
            tg.eq(name, w, "sec.rawptr", gs["rawptr"], l["rawptr"], "gt_dump")
            tg.eq(name, w, "sec.nrel", gs["nrel"], l["nrel"], "gt_dump")
            tg.eq(name, w, "sec.chars", gs["chars"], l["chars"], "gt_dump")
            for j, (gr, lr) in enumerate(zip(g.relocs(gs), l["relocs"])):
                tg.eq(name, "%s.rel[%d]" % (w, j), "rel.off", gr[0], lr["off"], "gt_dump")
                tg.eq(name, "%s.rel[%d]" % (w, j), "rel.sym", gr[1], lr["sym"], "gt_dump")
                tg.eq(name, "%s.rel[%d]" % (w, j), "rel.type", gr[2], lr["type"], "gt_dump")
        for i, (gy, l) in enumerate(zip(g.symbols, ly)):
            w = "sym[%d]" % gy["idx"]
            tg.eq(name, w, "sym.name", gy["name"], l["name"], "gt_dump")
            tg.eq(name, w, "sym.value", gy["value"], l["value"], "gt_dump")
            tg.eq(name, w, "sym.sec", gy["sec"], l["sec"], "gt_dump")
            tg.eq(name, w, "sym.type", gy["type"], l["type"], "gt_dump")
            tg.eq(name, w, "sym.sc", gy["sc"], l["sc"], "gt_dump")
            tg.eq(name, w, "sym.naux", gy["naux"], l["naux"], "gt_dump")

    return len(cs), len(cy), nrel_total, ls, ly


def llvm_section_names(ls):
    return [s["name"] for s in ls]


def llvm_text_comdat_leaders(ls, ly):
    """Reproduce crates/c2-obj's rule from LLVM's decode: for each COMDAT
    `.text*` section, the first symbol in that section that is not the
    section-definition symbol (STATIC with exactly one aux)."""
    is_text = {}
    for s in ls:
        if s["name"].startswith(".text") and (s["chars"] or 0) & IMAGE_SCN_LNK_COMDAT:
            is_text[s["number"]] = True
    claimed = set()
    out = []
    for y in ly:
        sec = y["sec"]
        if sec in is_text and sec not in claimed:
            if y["sc"] == IMAGE_SYM_CLASS_STATIC and y["naux"] == 1:
                continue
            claimed.add(sec)
            out.append((sec, y["name"]))
    out.sort()
    return [n for _, n in out]


def c2obj_decode(objs):
    """crates/c2-obj's own answers, via the out-of-workspace helper bin.

    Returns {obj: {"sections": [...], "text_comdats": [...]}} or None when
    cargo/rustc is unavailable -- absence is a SKIP, never a pass.
    """
    manifest = os.path.join(HERE, "c2objdump", "Cargo.toml")
    if not os.path.isfile(manifest):
        return None
    env = dict(os.environ)
    # Keep the helper's build products out of the workspace's /target and out of
    # git: /target is the one gitignored build path at the repo root.
    env.setdefault("CARGO_TARGET_DIR", os.path.join(REPO, "target", "w-llvm-c2objdump"))
    try:
        p = subprocess.run(
            ["cargo", "run", "--quiet", "--release", "--manifest-path", manifest, "--"] + objs,
            capture_output=True, text=True, cwd=REPO, env=env,
        )
    except OSError:
        return None
    if p.returncode != 0:
        tail = (p.stderr.strip() or p.stdout.strip()).splitlines()[-3:]
        sys.stderr.write("SKIP: crates/c2-obj cross-check unavailable (rc=%d): %s\n"
                         % (p.returncode, " | ".join(tail)))
        return None
    res, cur = {}, None
    for line in p.stdout.splitlines():
        if line.startswith("OBJ\t"):
            cur = line.split("\t", 1)[1]
            res[cur] = {"sections": [], "text_comdats": []}
        elif line.startswith("SEC\t") and cur:
            res[cur]["sections"].append(line.split("\t", 1)[1])
        elif line.startswith("FN\t") and cur:
            res[cur]["text_comdats"].append(line.split("\t", 1)[1])
        elif line.startswith("REFUSED\t"):
            cur = line.split("\t", 1)[1]
            res[cur] = None
    return res


def main(argv):
    global SELFTEST
    args = [a for a in argv[1:] if not a.startswith("--")]
    SELFTEST = "--selftest" in argv
    tsv = None
    if "--tsv" in argv:
        tsv = argv[argv.index("--tsv") + 1]
        args = [a for a in args if a != tsv]
    if not args:
        print(__doc__)
        return 2

    llvm = llvmpath.require_llvm("tools/llvm/xcheck.py")
    scratch = os.environ.get("C2RS_LLVM_SCRATCH", os.path.join(REPO, "work", "w-llvm", "scratch"))
    print("llvm: %s" % llvm.describe())
    if coffdump is None:
        print("SKIP: tools/coffdump.py not importable")
        return 0

    t = Tally()      # llvm vs tools/coffdump.py
    tg = Tally()     # llvm vs scripts/gt_dump.py
    tc = Tally()     # llvm vs crates/c2-obj
    nobj = nsec = nsym = nrel = 0
    llvm_cache = {}
    for obj in args:
        try:
            s, y, r, ls, ly = check_obj(llvm, obj, scratch, t, tg)
        except Exception as e:  # a refusal is a loud failure, not a skipped obj
            print("ERROR %s: %s" % (obj, e))
            return 1
        llvm_cache[obj] = (ls, ly)
        nobj += 1
        nsec += s
        nsym += y
        nrel += r

    c2 = c2obj_decode([os.path.abspath(a) for a in args])
    if c2 is None:
        print("SKIP: crates/c2-obj cross-check (cargo unavailable)")
    else:
        for obj in args:
            key = os.path.abspath(obj)
            entry = c2.get(key, c2.get(obj))
            base = os.path.basename(obj)
            ls, ly = llvm_cache[obj]
            if entry is None:
                tc.eq(base, "c2-obj", "decode", "REFUSED-or-missing", "decoded", "c2-obj")
                continue
            tc.eq(base, "c2-obj", "section_names", entry["sections"],
                  llvm_section_names(ls), "c2-obj")
            tc.eq(base, "c2-obj", "text_comdat_functions", entry["text_comdats"],
                  llvm_text_comdat_leaders(ls, ly), "c2-obj")

    rows = []
    for label, tt in (("coffdump.py", t), ("gt_dump.py", tg), ("c2-obj", tc)):
        rows.append((label, tt))

    print("")
    print("%-14s %12s %8s" % ("our reader", "comparisons", "diffs"))
    for label, tt in rows:
        print("%-14s %12d %8d" % (label, tt.compared, len(tt.diffs)))
    print("")
    print("objs %d   sections %d   symbols %d   relocations %d" % (nobj, nsec, nsym, nrel))

    total = sum(tt.compared for _, tt in rows)
    ndiff = sum(len(tt.diffs) for _, tt in rows)
    if total == 0 or nobj == 0:
        print("FAIL: compared 0 field instances — this run graded nothing")
        return 1

    if ndiff:
        print("")
        print("DISAGREEMENTS (%d):" % ndiff)
        seen = {}
        for label, tt in rows:
            for obj, where, field, on, ov, tn, tv in tt.diffs:
                k = (label, field)
                seen.setdefault(k, []).append((obj, where, on, ov, tn, tv))
        for (label, field), items in sorted(seen.items()):
            obj, where, on, ov, tn, tv = items[0]
            print("  [%s] %-22s x%-6d e.g. %s %s: %s=%r  llvm=%r"
                  % (label, field, len(items), obj, where, on, ov, tv))
    if tsv:
        with open(tsv, "w") as f:
            f.write("reader\tobj\twhere\tfield\tours\tllvm\n")
            for label, tt in rows:
                for obj, where, field, on, ov, tn, tv in tt.diffs:
                    f.write("%s\t%s\t%s\t%s\t%r\t%r\n" % (label, obj, where, field, ov, tv))
        print("wrote %s" % tsv)

    print("")
    print("compared %d objs across %d distinct fields, %d field instances; %d disagreements"
          % (nobj, len(set(f for _, tt in rows for f in tt.per_field)), total, ndiff))

    if SELFTEST:
        # The instrument check: perturbing sec.rawsize on our side must produce
        # exactly one disagreement per section, all of them on sec.rawsize.
        want = nsec
        got = [d for d in t.diffs if d[2] == "sec.rawsize"]
        ok = len(got) == want and len(t.diffs) == want
        print("SELFTEST: perturbed sec.rawsize on the c2-rs side of %d sections; "
              "comparison reported %d sec.rawsize diffs and %d diffs total -> %s"
              % (want, len(got), len(t.diffs), "PASS" if ok else "FAIL"))
        return 0 if ok else 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
