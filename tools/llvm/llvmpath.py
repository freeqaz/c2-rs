#!/usr/bin/env python3
"""llvmpath — locate an LLVM that can read Xbox 360 PPC COFF, or say SKIP.

Shared by every script in tools/llvm/. Three resolution steps, in order:

  1. ``$C2RS_LLVM_BIN`` — a directory holding ``llvm-readobj`` etc. Set this to
     the ``bin/`` of a tree built with ``tools/llvm/ppcbe.patch`` applied.
  2. ``$C2RS_LLVM_PREFIX/bin`` — same, one level up.
  3. ``PATH`` — the distro LLVM.

Then it asks the binary it found whether it accepts ``IMAGE_FILE_MACHINE_POWERPCBE
= 0x01F2`` directly. A *patched* LLVM does; a stock one does not, and for a stock
one every script works on a **scratch copy of the obj with the machine word
rewritten to 0x01F0**. The scratch copy is a diagnostic convenience only — it is
never an input to the port and never compared as bytes.

Nothing here fails when LLVM is absent: `find_llvm()` returns None and callers
print ``SKIP: llvm-readobj absent ...`` and exit 0. That is the project rule for
anything that touches an external toolchain.
"""

import os
import shutil
import struct
import subprocess
import sys

TOOLS = ("llvm-readobj", "llvm-objdump", "llvm-mc")

# Xbox 360 PPC COFF. Defined by Microsoft in exactly one published artifact
# (microsoft-pdb/cvdump/cvdump.cpp); in no PE/COFF spec revision and in no
# winnt.h. See docs/PRIOR_ART.md §5.5.
MACHINE_POWERPCBE = 0x01F2
# IMAGE_FILE_MACHINE_POWERPC, which stock LLVM's identify_magic does accept.
MACHINE_POWERPC = 0x01F0


class Llvm:
    """A located LLVM, plus whether it needs the obj machine word rewritten."""

    def __init__(self, bindir, version, native_ppcbe):
        self.bindir = bindir
        self.version = version
        self.native_ppcbe = native_ppcbe

    def tool(self, name):
        return os.path.join(self.bindir, name) if self.bindir else name

    def describe(self):
        where = self.bindir or "PATH"
        how = "accepts 0x01F2 natively" if self.native_ppcbe else "needs 0x01F0 scratch copy"
        return "%s (%s) — %s" % (where, self.version, how)

    def readable(self, obj_path, scratch_dir):
        """Return a path llvm-readobj will accept for `obj_path`.

        Patched LLVM: the obj itself. Stock LLVM: a scratch copy under
        `scratch_dir` whose first two bytes are 0x01F0.
        """
        if self.native_ppcbe:
            return obj_path
        os.makedirs(scratch_dir, exist_ok=True)
        out = os.path.join(scratch_dir, os.path.basename(obj_path) + ".le")
        data = bytearray(open(obj_path, "rb").read())
        if len(data) < 2:
            return obj_path
        if struct.unpack_from("<H", data, 0)[0] == MACHINE_POWERPCBE:
            struct.pack_into("<H", data, 0, MACHINE_POWERPC)
        with open(out, "wb") as f:
            f.write(data)
        return out

    def run(self, name, args, scratch_dir=None):
        cmd = [self.tool(name)] + list(args)
        p = subprocess.run(cmd, capture_output=True, text=True)
        return p.stdout, p.stderr, p.returncode


def _probe_dir(d):
    if not d:
        return None
    exe = os.path.join(d, "llvm-readobj")
    return d if os.path.isfile(exe) and os.access(exe, os.X_OK) else None


def _version(bindir):
    exe = os.path.join(bindir, "llvm-readobj") if bindir else "llvm-readobj"
    try:
        out = subprocess.run([exe, "--version"], capture_output=True, text=True).stdout
    except OSError:
        return "?"
    for line in out.splitlines():
        line = line.strip()
        if line.startswith("LLVM version"):
            return line.split()[-1]
    return "?"


def _accepts_ppcbe(bindir):
    """Build a 20-byte header-only COFF at 0x01F2 and see if llvm-readobj takes it.

    A positive test, not an absence test: the same probe at 0x01F0 must SUCCEED,
    otherwise the probe itself is broken and we say so rather than reporting
    'not native'.
    """
    exe = os.path.join(bindir, "llvm-readobj") if bindir else "llvm-readobj"

    def try_machine(m):
        import tempfile

        hdr = struct.pack("<HHIIIHH", m, 0, 0, 20, 0, 0, 0x0180) + b"\0\0\0\0"
        with tempfile.NamedTemporaryFile(suffix=".obj", delete=False) as f:
            f.write(hdr)
            path = f.name
        try:
            p = subprocess.run([exe, "--file-headers", path], capture_output=True, text=True)
            return p.returncode == 0
        finally:
            os.unlink(path)

    control = try_machine(MACHINE_POWERPC)
    if not control:
        # The control failed: this llvm-readobj rejects even 0x01F0, so the
        # answer for 0x01F2 is not interpretable. Report "not native" but the
        # caller's own parse will fail loudly rather than silently.
        return False
    return try_machine(MACHINE_POWERPCBE)


def find_llvm():
    """Locate an LLVM, or None. Never raises, never exits."""
    for cand in (
        os.environ.get("C2RS_LLVM_BIN"),
        os.path.join(os.environ["C2RS_LLVM_PREFIX"], "bin")
        if os.environ.get("C2RS_LLVM_PREFIX")
        else None,
    ):
        d = _probe_dir(cand)
        if d:
            return Llvm(d, _version(d), _accepts_ppcbe(d))
    if shutil.which("llvm-readobj"):
        return Llvm(None, _version(None), _accepts_ppcbe(None))
    return None


def require_llvm(what="tools/llvm"):
    """find_llvm(), or print the project's SKIP line and exit 0."""
    llvm = find_llvm()
    if llvm is None:
        print(
            "SKIP: llvm-readobj absent — %s needs LLVM on PATH or $C2RS_LLVM_BIN "
            "(see tools/llvm/README.md)" % what
        )
        sys.exit(0)
    return llvm


if __name__ == "__main__":
    l = find_llvm()
    if l is None:
        print("SKIP: llvm-readobj absent")
    else:
        print(l.describe())
