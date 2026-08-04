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
import cap, glparse, paths, prov

DC3 = paths.DC3
FLAGS = paths.flags()
CENSUS = paths.SECTIONS


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


def _check_upstream():
    """`sections.jsonl` is this census's INPUT.  If it was built against another
    corpus or at another directory, everything joined downstream is cross-corpus
    and the join loses ~20 % of its population silently (w-repro section 5).
    Catch it here, before spending the capture, not in `grade.py` afterwards."""
    try:
        up = prov.read(CENSUS)
    except prov.ProvError as e:
        print(prov.banner(e), file=sys.stderr)
        print("WARNING: continuing against an UNSTAMPED sections.jsonl. The\n"
              "         resulting glcensus records `sections_prov: null` and\n"
              "         grade.py will refuse to join it.", file=sys.stderr)
        return None
    head, dirty = prov.corpus_state(DC3)
    skew = (up["corpus"]["head"] != head
            or up["corpus"]["path_sha256"] != prov.path_sha256(DC3))
    if skew and os.environ.get("C2RS_PROV_ALLOW_SKEW") != "1":
        print(prov.banner(prov.ProvError(
            "sections.jsonl WAS BUILT AGAINST A DIFFERENT CORPUS\n"
            "  sections.jsonl : head %s  path_sha256 %s\n"
            "  this run       : head %s  path_sha256 %s\n"
            "Capturing .gl here would produce a census that cannot be joined\n"
            "against it. Regenerate sections.jsonl (scripts/regen_census.sh\n"
            "--sections), or set C2RS_PROV_ALLOW_SKEW=1 to record the skew and\n"
            "let grade.py refuse it later."
            % (up["corpus"]["head"], up["corpus"]["path_sha256"][:16],
               head, prov.path_sha256(DC3)[:16]))), file=sys.stderr)
        sys.exit(2)
    return up


if __name__ == "__main__":
    out_path = sys.argv[1]
    jobs = int(sys.argv[2]) if len(sys.argv) > 2 else 12
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 0
    # BEFORE the captures: this census does its own compiling, so its snapshot
    # covers the whole run.
    begin = prov.begin(DC3)
    up = _check_upstream()
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

    # glcensus.jsonl is NOT committed, so its sidecar is not either, and it may
    # carry the absolute corpus path in cleartext -- nothing about an untracked
    # file beside an untracked file reaches the history, and a human debugging a
    # provenance mismatch wants the real path.
    p = prov.stamp("glcensus.py", out_path, begin, paths.MAIN,
                   inputs=dict(flags_sha256=prov.sha256_file(
                                   os.path.join(paths.WORKLOAD, "flags.txt")),
                               files_sha256=prov.sha256_file(
                                   os.path.join(paths.WORKLOAD, "files.txt")),
                               sections_sha256=prov.sha256_file(CENSUS),
                               sections_head=(up or {}).get("corpus", {}).get("head")),
                   allow_dirty=os.environ.get("C2RS_PROV_ALLOW_DIRTY") == "1",
                   allow_move=os.environ.get("C2RS_PROV_ALLOW_MOVE") == "1",
                   begin_scope="run", records=done)
    print("provenance ->", prov.write(out_path, p, committed=False))
    print(" ", prov.describe(p))
