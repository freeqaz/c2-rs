#!/usr/bin/env python3
"""tagcensus.py — WHICH `.gl` DATA-record alignment tags does the real workload
actually contain? Lane w-align16. Read-only w.r.t. the crate.

Board #1120 asks whether anything above 16 exists. The 20 frozen cells say the
ENCODING goes to 64 (`CE`), but a cell is a thing this lane wrote; the workload
is the thing the port is graded on. This walks all 878 dc3 TUs at the workload's
own flags, captures the IL, and tallies the tag byte of every `.gl` symbol run
that frames as an ORDINARY-DATA record.

ONE DIRECTORY PER TU (board #1045) and each one removed after it is read, so the
whole pass costs one TU's IL at a time and never populates a shared temp dir.

    tagcensus.py <out.txt> [jobs]
"""
import collections
import concurrent.futures as cf
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(ROOT, "work", "w-align16"))
from glread import graphic_runs, data_record  # noqa: E402

DC3 = os.environ.get("C2RS_DC3", os.path.join(ROOT, "..", "..", "..", "..", "dc3-decomp"))
C2RS = os.path.join(ROOT, "target", "release", "c2rs")
FLAGS = os.path.join(ROOT, "work", "dc3-workload", "flags.txt")
SCRATCH = os.path.join(ROOT, "work", "w-align16", "tagcensus.d")


def one(idx_src):
    idx, src = idx_src
    d = os.path.join(SCRATCH, "%04d" % idx)
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d, exist_ok=True)
    tags = collections.Counter()
    status = "ok"
    try:
        r = subprocess.run([C2RS, "capture", src, "--keep-il", d,
                            "--flags-file", FLAGS, "--cwd", DC3],
                           capture_output=True, timeout=300)
        if r.returncode != 0:
            status = "capture-fail"
        gls = [f for f in os.listdir(d) if f.endswith(".gl")]
        if not gls:
            status = "no-gl" if status == "ok" else status
        else:
            gl = open(os.path.join(d, gls[0]), "rb").read()
            for _s, nul, txt in graphic_runs(gl):
                if txt.startswith("$"):
                    txt = txt[1:]
                if not txt or not (txt[0] == "?" or txt[0] == "_" or txt[0].isalpha()):
                    continue
                rec = data_record(gl, nul)
                t = rec["tag"]
                if t is None or not (t & 0x80):
                    continue
                # Only records that FRAME as ordinary data — the population
                # `data_object_at` walks. `attr` is allowed to be anything here
                # (a0 included) so a refusal downstream is still counted.
                if rec["refused_at"] in ("frame", "linkage", "size", "wide-mark"):
                    continue
                tags[t] += 1
    except Exception as e:  # noqa: BLE001
        status = "error:%s" % type(e).__name__
    finally:
        shutil.rmtree(d, ignore_errors=True)
    return src, status, tags


def main():
    out = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 12
    files = [l.strip() for l in open(os.path.join(ROOT, "work", "dc3-workload", "files.txt"))
             if l.strip()]
    os.makedirs(SCRATCH, exist_ok=True)
    total = collections.Counter()
    status = collections.Counter()
    tus_with = collections.defaultdict(set)
    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        for src, st, tags in ex.map(one, list(enumerate(files))):
            status[st] += 1
            total.update(tags)
            for t in tags:
                tus_with[t].add(src)
    shutil.rmtree(SCRATCH, ignore_errors=True)
    with open(out, "w") as f:
        f.write("TUs %d  status %s\n\n" % (len(files), dict(status)))
        f.write("%-6s %-8s %10s %8s   %s\n" % ("tag", "masked", "records", "TUs", "width the encoding gives"))
        W = {0x82: 1, 0x84: 2, 0x86: 4, 0x88: 8, 0x8A: 16, 0x8C: 32, 0x8E: 64}
        for t in sorted(total):
            m = t & ~0x40
            f.write("%-6s %-8s %10d %8d   %s\n"
                    % ("%02x" % t, "%02x" % m, total[t], len(tus_with[t]),
                       W.get(m, "UNKNOWN")))
        f.write("\ntotal records %d\n" % sum(total.values()))
        above8 = sum(v for k, v in total.items() if (k & ~0x40) in (0x8A, 0x8C, 0x8E))
        f.write("records whose masked tag is 8A/8C/8E (align >= 16): %d\n" % above8)
        for t in (0x8A, 0x8C, 0x8E, 0xCA, 0xCC, 0xCE):
            f.write("  tag %02x: %d records in %d TUs\n" % (t, total.get(t, 0), len(tus_with.get(t, ()))))
            for s in sorted(tus_with.get(t, ()))[:20]:
                f.write("      %s\n" % s)
    print(open(out).read())


main()
