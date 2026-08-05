#!/usr/bin/env python3
"""llvmmc.py -- ask `llvm-mc` what it thinks a big-endian PowerPC word is.

Pure stdlib. Tooling, outside the std-only Rust workspace.

Locates LLVM the way `tools/llvm/llvmpath.py` does (`C2RS_LLVM_BIN` ->
`C2RS_LLVM_PREFIX/bin` -> `PATH`) and returns `None` when there is none, so
every caller can print `SKIP: llvm-mc absent` and exit 0. That is the project
rule for anything touching an external toolchain.

The batching trick, and why it is exact: `llvm-mc --disassemble` writes one
line to stdout per *successfully* decoded word and writes `<stdin>:N:1:` to
stderr for each word it refuses. Refused words produce no stdout line. So
walking 1..N and consuming a stdout line for every input line NOT named in
stderr reconstructs the per-word answer exactly. `disassemble()` asserts that
the two counts add up and falls back to one-process-per-word if they do not --
a silent misalignment here would be the same bug class this whole lane exists
to catch.
"""
import os
import re
import shutil
import subprocess
import sys

_ERRLINE = re.compile(r"^<stdin>:(\d+):\d+: (?:error|warning): (.*)$")


def find_llvm_mc():
    for env in ("C2RS_LLVM_BIN",):
        d = os.environ.get(env)
        if d and os.path.isfile(os.path.join(d, "llvm-mc")):
            return os.path.join(d, "llvm-mc")
    p = os.environ.get("C2RS_LLVM_PREFIX")
    if p and os.path.isfile(os.path.join(p, "bin", "llvm-mc")):
        return os.path.join(p, "bin", "llvm-mc")
    return shutil.which("llvm-mc")


def version(mc=None):
    mc = mc or find_llvm_mc()
    if not mc:
        return None
    try:
        out = subprocess.run([mc, "--version"], capture_output=True, text=True,
                             timeout=30).stdout
    except Exception:
        return None
    for line in out.splitlines():
        if "version" in line:
            return line.strip()
    return out.strip().splitlines()[0] if out.strip() else None


def _run(mc, words, triple, mcpu):
    stdin = "".join("0x%02x 0x%02x 0x%02x 0x%02x\n" % (
        (w >> 24) & 0xFF, (w >> 16) & 0xFF, (w >> 8) & 0xFF, w & 0xFF)
        for w in words)
    cmd = [mc, "--disassemble", "-triple=" + triple]
    if mcpu:
        cmd.append("-mcpu=" + mcpu)
    r = subprocess.run(cmd, input=stdin, capture_output=True, text=True,
                       timeout=300)
    bad = {}
    for line in r.stderr.splitlines():
        m = _ERRLINE.match(line)
        if m:
            bad[int(m.group(1))] = m.group(2)
    good = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    return bad, good


def disassemble(words, triple="powerpc", mcpu=None, mc=None, batch=256):
    """[(ok, text_or_error)] parallel to `words`. `ok` False means llvm-mc
    REFUSED the word -- which is the *safe* outcome. `ok` True with wrong text
    is the dangerous one, and is what this lane measures."""
    mc = mc or find_llvm_mc()
    if not mc:
        return None
    out = []
    for i in range(0, len(words), batch):
        chunk = words[i:i + batch]
        bad, good = _run(mc, chunk, triple, mcpu)
        if len(good) + len(bad) != len(chunk):
            # Misalignment: refuse to guess. One process per word.
            for w in chunk:
                b, g = _run(mc, [w], triple, mcpu)
                out.append((False, b.get(1, "refused")) if b
                           else (True, g[0] if g else ""))
            continue
        gi = 0
        for k in range(1, len(chunk) + 1):
            if k in bad:
                out.append((False, bad[k]))
            else:
                out.append((True, good[gi]))
                gi += 1
    return out


def normalize(text):
    """`vmr\t30, 10` -> `vmr 30,10`, so textual comparison is about content."""
    t = text.replace("\t", " ")
    t = re.sub(r"\s*,\s*", ",", t)
    t = re.sub(r"\s+", " ", t).strip()
    return t


if __name__ == "__main__":
    mc = find_llvm_mc()
    if not mc:
        print("SKIP: llvm-mc absent")
        sys.exit(0)
    print("# %s -- %s" % (mc, version(mc)))
    ws = [int(a, 16) for a in sys.argv[1:]]
    for w, (ok, txt) in zip(ws, disassemble(ws)):
        print("%08x  %-6s %s" % (w, "OK" if ok else "REFUSED", normalize(txt)))
