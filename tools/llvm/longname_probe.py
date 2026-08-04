#!/usr/bin/env python3
"""longname_probe — exercise the `/NNN` long-section-name path that real dc3
objs never take, and see which readers survive it.

`crates/c2-obj` documents `/NNN` (a section name that is a decimal offset into
the string table) as one of *"three chances for a second reader to disagree with
this one"*, and ROADMAP §10.14 records a session lost to exactly that. But no
obj `c2.dll` emits at the workload's flags carries one: every section name in
the sample is <= 8 characters, so the branch is dead code in production and a
reader that gets it wrong will never be caught by a corpus sweep.

This builds a SYNTHETIC obj -- a real one with one section's 8-byte name field
rewritten to `/NNN` and the long name appended to the string table -- and asks
every reader what that section is called. It is a probe of the decoders, not a
claim about c2's output: c2 does not produce this, and nothing here should be
read as saying it does.

    tools/llvm/longname_probe.py <real.obj> [out.obj]
"""

import os
import struct
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "tools"))
sys.path.insert(0, os.path.join(REPO, "scripts"))

import llvmpath  # noqa: E402
import readobj_parse as R  # noqa: E402

LONG_NAME = ".averylongsectionname$probe"


def build(src, dst, sec_index=0):
    """Rewrite section `sec_index`'s name to a `/NNN` string-table reference."""
    d = bytearray(open(src, "rb").read())
    nsec, symptr, nsym = (
        struct.unpack_from("<H", d, 2)[0],
        struct.unpack_from("<I", d, 8)[0],
        struct.unpack_from("<I", d, 12)[0],
    )
    strtab = symptr + 18 * nsym
    size = struct.unpack_from("<I", d, strtab)[0]
    # Append the long name at the current end of the string table.
    off = size
    d[strtab + size : strtab + size] = LONG_NAME.encode() + b"\0"
    struct.pack_into("<I", d, strtab, size + len(LONG_NAME) + 1)
    ref = ("/%d" % off).encode()
    o = 20 + sec_index * 40
    d[o : o + 8] = ref.ljust(8, b"\0")
    open(dst, "wb").write(bytes(d))
    return nsec, off


def readers(obj):
    """{reader name: what it calls section 1}."""
    out = {}
    data = open(obj, "rb").read()

    import coffdump

    secs, _ = coffdump.read_coff(data)
    out["tools/coffdump.py"] = secs[0].name if secs else "<refused>"

    try:
        import gt_dump

        out["scripts/gt_dump.py"] = gt_dump.Obj(data).sections[0]["name"]
    except Exception as e:
        out["scripts/gt_dump.py"] = "<error %s>" % e

    manifest = os.path.join(HERE, "c2objdump", "Cargo.toml")
    env = dict(os.environ)
    env.setdefault("CARGO_TARGET_DIR", os.path.join(REPO, "target", "w-llvm-c2objdump"))
    try:
        p = subprocess.run(
            ["cargo", "run", "--quiet", "--release", "--manifest-path", manifest, "--",
             os.path.abspath(obj)],
            capture_output=True, text=True, cwd=REPO, env=env,
        )
        secs = [l.split("\t", 1)[1] for l in p.stdout.splitlines() if l.startswith("SEC\t")]
        if secs:
            out["crates/c2-obj"] = secs[0]
        elif any(l.startswith("REFUSED") for l in p.stdout.splitlines()):
            out["crates/c2-obj"] = "<ObjImage refused the obj>"
        else:
            # cargo/rustc could not run at all. An unavailable reader is a SKIP,
            # never a "did not resolve" -- scoring it as a failure would let an
            # absent toolchain masquerade as a decoder bug.
            out["crates/c2-obj"] = None
    except OSError:
        out["crates/c2-obj"] = None

    llvm = llvmpath.find_llvm()
    if llvm is None:
        out["llvm-readobj"] = None
    else:
        scratch = os.environ.get(
            "C2RS_LLVM_SCRATCH", os.path.join(REPO, "work", "w-llvm", "scratch")
        )
        path = llvm.readable(obj, scratch)
        text, _, rc = llvm.run("llvm-readobj", ["--sections", path])
        secs = R.find_all(R.parse(text), "Section")
        out["llvm-readobj"] = secs[0].name("Name") if secs else "<refused rc=%d>" % rc
    return out


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    src = argv[1]
    dst = argv[2] if len(argv) > 2 else os.path.join(
        os.environ.get("C2RS_LLVM_SCRATCH", os.path.join(REPO, "work", "w-llvm", "scratch")),
        "longname.obj",
    )
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    nsec, off = build(src, dst)
    print("synthetic obj: %s" % dst)
    print("section 1 name field rewritten to /%d -> %r (appended to string table)"
          % (off, LONG_NAME))
    print("")
    res = readers(dst)
    width = max(len(k) for k in res)
    right = asked = 0
    for k, v in res.items():
        if v is None:
            print("  %-*s  SKIP: reader unavailable" % (width, k))
            continue
        asked += 1
        ok = v == LONG_NAME
        right += ok
        print("  %-*s  %-30r %s" % (width, k, v, "resolved" if ok else "*** DID NOT RESOLVE"))
    print("")
    if asked == 0:
        print("SKIP: no reader was available — nothing was probed")
        return 0
    print("%d of %d AVAILABLE readers resolved the long name "
          "(%d of %d readers were unavailable)"
          % (right, asked, len(res) - asked, len(res)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
