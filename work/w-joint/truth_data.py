#!/usr/bin/env python3
"""truth_data.py — build the extended truth (DEFINED DATA SYMBOLS) and grade it.

Reads the obj the harness's capture cache already holds for every workload TU
at the pinned dc3 rev (`cacheindex.py`), classifies its whole symbol table
(`objsyms.py`), writes one JSON per TU, and prints the four invariants that
grade a *correspondence* instrument, which the compiler cannot grade for us:

  INJ    injectivity  — a defined name defines one entity, per obj
  TOT    totality     — every entity in exactly one bucket, RESIDUE NAMED
  AR     arity        — A1 records, A2 aux, A3 long-name bytes
  AGREE  agreement    — recomputed code COMDAT leaders == w-emit's truth,
                        the one place the oracle already graded the table
  DUP    a control that can go red — the cache holds several entries per TU at
         the same rev (one per lane that ever scanned it); their symbol
         classification must be IDENTICAL.  If the entries disagreed, reading
         the cache instead of re-running `cl` would be unsound and this control
         is what says so.

    usage: truth_data.py <cacheidx.tsv> <outroot> <w-emit-truth-dir> [jobs]

Every failure mode is printed with a COUNT and with NAMES.  A silent pass is
not a pass here: `KA-POS` prints how many TUs actually carried a data symbol,
so a run that graded nothing cannot read as success.
"""
import json
import os
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import objsyms  # noqa: E402


def slug(src):
    return src.replace("/", "__").replace("\\", "__")


def one(row, outroot, wetruth):
    src, entry, ndup = row
    r = {"src": src, "entry": os.path.basename(entry), "n_entries": ndup}
    try:
        b = open(os.path.join(entry, "out.obj"), "rb").read()
    except OSError as ex:
        r["status"] = "NOOBJ: %r" % (ex,)
        return r
    o = objsyms.ObjSyms(b)
    if not o.ok:
        r["status"] = "COFF-REJECT: %s" % o.err
        return r
    s = objsyms.sets(o)
    nm = objsyms.name_rule_E(o)

    # ---- AGREE: against w-emit's independently captured truth -------
    tf = os.path.join(wetruth, slug(src) + ".txt")
    if os.path.exists(tf):
        E_we = sorted(set(x for x in open(tf).read().split() if x))
        r["agree"] = 1 if E_we == s["E"] else 0
        r["agree_only_mine"] = sorted(set(s["E"]) - set(E_we))[:12]
        r["agree_only_theirs"] = sorted(set(E_we) - set(s["E"]))[:12]
        r["n_E_we"] = len(E_we)
    else:
        r["agree"] = -1
        r["n_E_we"] = -1

    r["status"] = "ok"
    r["n_E"] = len(s["E"])
    r["n_D_all"] = len(s["D_all"])
    r["n_D_data"] = len(s["D_data"])
    r["n_D_lead"] = len(s["D_lead"])
    r["n_undef"] = len(s["U_undef"])
    r["buckets"] = s["buckets"]
    r["arity"] = s["arity"]
    r["residue"] = s["residue"][:20]
    r["n_residue"] = len(s["residue"])
    r["conflicts"] = [c[0] for c in s["conflicts"]][:20]
    r["n_conflicts"] = len(s["conflicts"])
    r["unclaimed_comdat"] = s["unclaimed_comdat"][:20]
    r["n_unclaimed_comdat"] = len(s["unclaimed_comdat"])
    r["name_rule_ok"] = 1 if nm == s["E"] else 0
    r["secnames"] = s["secnames"]

    json.dump({"src": src, "E": s["E"], "D_all": s["D_all"],
               "D_data": s["D_data"], "D_lead": s["D_lead"],
               "undef": s["U_undef"]},
              open(os.path.join(outroot, slug(src) + ".json"), "w"))
    return r


def dup_control(rows, cacheidx_all, limit):
    """KA-DUP — a second entry for the same TU must classify identically."""
    ok = bad = 0
    detail = []
    for src, entries in list(cacheidx_all.items())[:limit]:
        if len(entries) < 2:
            continue
        sig = []
        for e in entries[:2]:
            b = open(os.path.join(e, "out.obj"), "rb").read()
            o = objsyms.ObjSyms(b)
            if not o.ok:
                sig.append(None)
                continue
            s = objsyms.sets(o)
            sig.append((s["E"], s["D_all"], s["D_data"], s["buckets"]))
        if sig[0] is not None and sig[0] == sig[1]:
            ok += 1
        else:
            bad += 1
            detail.append(src)
    return ok, bad, detail


def main():
    idxp, outroot, wetruth = sys.argv[1], sys.argv[2], sys.argv[3]
    jobs = int(sys.argv[4]) if len(sys.argv) > 4 else 12
    dupn = int(sys.argv[5]) if len(sys.argv) > 5 else 40
    os.makedirs(outroot, exist_ok=True)
    rows = []
    allent = {}
    for ln in open(idxp):
        p = ln.rstrip("\n").split("\t")
        if len(p) >= 3:
            rows.append((p[0], p[1], int(p[2])))
    print("TUs to build: %d" % len(rows), flush=True)

    res = []
    with cf.ThreadPoolExecutor(jobs) as ex:
        for i, r in enumerate(ex.map(lambda x: one(x, outroot, wetruth), rows)):
            res.append(r)
            if (i + 1) % 200 == 0:
                print("... %d/%d" % (i + 1, len(rows)), flush=True)

    good = [r for r in res if r["status"] == "ok"]
    bad = [r for r in res if r["status"] != "ok"]
    print("\n==== EXTENDED TRUTH — %d TUs built, %d failed" % (len(good), len(bad)))
    for r in bad:
        print("  FAIL %s  %s" % (r["src"], r["status"]))

    # ---- the four invariants ---------------------------------------
    n_res = sum(r["n_residue"] for r in good)
    n_con = sum(r["n_conflicts"] for r in good)
    n_unc = sum(r["n_unclaimed_comdat"] for r in good)
    a1 = sum(1 for r in good
             if r["arity"]["records_consumed"] != r["arity"]["nsym_header"])
    a2 = sum(1 for r in good if r["arity"]["aux"] != r["arity"]["aux_check"])
    a3 = sum(1 for r in good if r["arity"]["long_name_unresolved"])
    agree = sum(1 for r in good if r["agree"] == 1)
    agree_no = [r for r in good if r["agree"] == 0]
    agree_na = sum(1 for r in good if r["agree"] == -1)
    namerule = sum(1 for r in good if r["name_rule_ok"])

    tot_rec = sum(r["arity"]["records_consumed"] for r in good)
    tot_ent = sum(r["arity"]["entities"] for r in good)
    tot_aux = sum(r["arity"]["aux"] for r in good)
    tot_str = sum(r["arity"]["long_name_bytes"] for r in good)

    print("\n-- INJ  injectivity: %d TUs with a duplicate defined name "
          "(%d names total)"
          % (sum(1 for r in good if r["n_conflicts"]), n_con))
    for r in good:
        if r["n_conflicts"]:
            print("     %s  %s" % (r["src"], r["conflicts"][:6]))
    print("-- TOT  totality: residue %d entities over %d TUs ; "
          "unclaimed COMDAT sections %d" % (n_res, len(good), n_unc))
    for r in good:
        if r["n_residue"]:
            print("     RESIDUE %s  %s" % (r["src"], r["residue"][:6]))
        if r["n_unclaimed_comdat"]:
            print("     UNCLAIMED %s  %s" % (r["src"], r["unclaimed_comdat"][:6]))
    print("-- AR   arity: A1 records!=nsym on %d TUs ; A2 aux mismatch on %d ; "
          "A3 unresolved long name on %d" % (a1, a2, a3))
    print("        totals: records %d = entities %d + aux %d ; "
          "long-name bytes %d" % (tot_rec, tot_ent, tot_aux, tot_str))
    print("-- AGREE code COMDAT leaders vs w-emit truth: %d/%d agree, "
          "%d disagree, %d without truth"
          % (agree, len(good), len(agree_no), agree_na))
    for r in agree_no[:20]:
        print("     DISAGREE %s  mine-only %s  theirs-only %s"
              % (r["src"], r["agree_only_mine"], r["agree_only_theirs"]))
    print("        (.text-prefix rule agrees with the characteristic rule on "
          "%d/%d)" % (namerule, len(good)))

    # ---- KA-DUP -----------------------------------------------------
    for ln in open(idxp):
        p = ln.rstrip("\n").split("\t")
        if len(p) >= 3:
            allent.setdefault(p[0], []).append(p[1])
    print("\n-- KA-DUP is run separately by dupcheck.py (it needs every entry, "
          "not the chosen one)")

    # ---- KA-POS: this run GRADED something --------------------------
    withdata = sum(1 for r in good if r["n_D_data"])
    tot_D = sum(r["n_D_all"] for r in good)
    tot_Dd = sum(r["n_D_data"] for r in good)
    tot_E = sum(r["n_E"] for r in good)
    print("\n-- KA-POS  TUs carrying at least one defined DATA symbol: %d/%d"
          % (withdata, len(good)))
    print("           defined symbols  |D_all| %d   |D_data| %d   "
          "|D_lead| %d   |E| %d"
          % (tot_D, tot_Dd, sum(r["n_D_lead"] for r in good), tot_E))
    secs = {}
    for r in good:
        for s in r["secnames"]:
            secs[s] = secs.get(s, 0) + 1
    print("           section names over the corpus: %s"
          % sorted(secs.items(), key=lambda kv: -kv[1]))
    print("\nDONE", flush=True)


if __name__ == "__main__":
    main()
