#!/usr/bin/env python3
"""Lane w-bss2: capture the IL `.gl` for every workload TU and keep, per TU,
only the records that name a symbol defined in one of that TU's `.data`/`.bss`
sections.

The obj side already exists — `work/w-bss/census/sections.jsonl` carries every
section header, every defined symbol and its offset.  What it does not carry is
the allocator's *input*: each object's size, alignment, linkage and declaration
order.  That is in the IL, and this fetches it.

Front-end only (`/Bd /d2nop`): c2 never runs, no obj is produced, nothing large
is written.  Output is one JSON line per TU, a few KB each.

  usage: glcensus.py <out.jsonl> [jobs] [limit]

NEVER glob work/capture-cache or .claude/worktrees — this iterates the explicit
resolved list in work/dc3-workload/files.txt.
"""
import json, os, sys, concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import cap, glparse

MAIN = "/home/free/code/milohax/c2-rs"
DC3 = os.environ.get("C2RS_DC3_SRC", "/home/free/code/milohax/dc3-decomp")
FLAGS = open(os.path.join(MAIN, "work/dc3-workload/flags.txt")).read().split()
CENSUS = os.path.join(MAIN, "work/w-bss/census/sections.jsonl")


def wanted_names(rec):
    """Every symbol defined in a .data/.bss of this TU, plus its `$` form."""
    w = set()
    for e in rec["data"] + rec["bss"]:
        for sy in e["syms"]:
            w.add(sy["n"])
            w.add("$" + sy["n"])
    return w


def one(rec):
    src = rec["src"]
    try:
        b = cap.capture_il(src, FLAGS, cwd=DC3)
    except Exception as e:
        return dict(src=src, err=str(e)[:200])
    g = glparse.globals_in_order(b["gl"])
    w = wanted_names(rec)
    keep = [dict(i=i, n=r["name"], sz=r["size"], al=r["align"],
                 sc=r["sc"], gid=r["gid"])
            for i, r in enumerate(g) if r["name"] in w]
    # The deferred (dynamic-initializer) set.  Two markers, one per linkage:
    #   internal: a `$<name>$initializer$` data record
    #   external: a `??__E<qualified-path>@@YAXXZ` FUNCTION record — c1xx's
    #             per-object dynamic-initializer thunk.  It is not a data
    #             record, so it has to be read off the raw name list.
    # `??__E` embeds the object's *path* for a namespace-scope object
    # (`??__ETheRockCentral@@YAXXZ`) but its whole DECORATED name for a class
    # static member (`??__E?kServerVer@RockCentral@@0VString@@B@@YAXXZ`), so
    # both spellings are kept and matched against both forms downstream.
    init = set()
    for nm in glparse.all_names(b["gl"]):
        if nm.startswith("$") and "$initializer$" in nm:
            init.add(nm[1:].split("$initializer$")[0])
        elif nm.startswith("??__E") and nm.endswith("@@YAXXZ"):
            init.add(nm[len("??__E"):-len("@@YAXXZ")])
    return dict(src=src, ngl=len(g), keep=keep, init=sorted(init))


if __name__ == "__main__":
    out_path = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 12
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    recs = [json.loads(l) for l in open(CENSUS)]
    if limit:
        recs = recs[:limit]
    done = 0
    with open(out_path, "w") as out, cf.ThreadPoolExecutor(jobs) as ex:
        for r in ex.map(one, recs):
            out.write(json.dumps(r) + "\n")
            done += 1
            if done % 50 == 0:
                print("  %d/%d" % (done, len(recs)), flush=True)
    print("wrote", done, "records to", out_path)
