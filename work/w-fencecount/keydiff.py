#!/usr/bin/env python3
"""keydiff — neutrality check for a gap-scan pair (BASE vs TIP).

Answers exactly three questions, and answers them positively (counts, not
absences):

  1. Is every `gap-metric <key> <value>` line that BASE printed present in TIP
     with a byte-identical value?  (Any violation is an ALARM.)
  2. Which keys are new in TIP?  (For an additive change these should be the
     whole of the new family and nothing else.)
  3. Is the per-TU verdict map {src: class} identical?  (only-in-base,
     only-in-tip, changed-by-name should all be 0.)

Plus a side-by-side of the six headline class counts.

Usage:
  keydiff.py BASE.out TIP.out BASE.jsonl TIP.jsonl

stdlib only; paths come from argv so nothing machine-specific is baked in.
"""

import json
import re
import sys

# A metric line is EXACTLY three whitespace-separated fields after stripping:
# the literal `gap-metric`, a key, and a value. The scan's prose also mentions
# `gap-metric fnbyte-census-disagree-*` inside a sentence; requiring the whole
# stripped line to match keeps that out.
METRIC = re.compile(r"^gap-metric (\S+) (\S+)$")

HEADLINE = [
    "match",
    "mismatch",
    "codegen-gap",
    "vocab-gap",
    "port-error",
    "capture-fail",
]


def read_metrics(path):
    """{key: value} for every well-formed `gap-metric` line in a scan .out."""
    out = {}
    dupes = []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = METRIC.match(line.strip())
            if not m:
                continue
            key, val = m.group(1), m.group(2)
            if key in out and out[key] != val:
                dupes.append((key, out[key], val))
            out[key] = val
    return out, dupes


def read_verdicts(path):
    """{src: class} for every TU record in a scan .jsonl (provenance skipped)."""
    out = {}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            rec = json.loads(line)
            if "class" not in rec:  # provenance / any non-TU record
                continue
            out[rec["src"]] = rec["class"]
    return out


def main(argv):
    if len(argv) != 5:
        print(__doc__.strip())
        return 2
    base_out, tip_out, base_jsonl, tip_jsonl = argv[1:5]

    base_m, base_dupes = read_metrics(base_out)
    tip_m, tip_dupes = read_metrics(tip_out)

    alarms = 0
    print("=" * 72)
    print("KEYDIFF — base vs tip gap scan")
    print("=" * 72)
    print()
    for label, dupes in (("base", base_dupes), ("tip", tip_dupes)):
        for key, a, b in dupes:
            print("NOTE: %s prints key %r twice with differing values %s / %s"
                  % (label, key, a, b))
    print("metric keys: %d in base, %d in tip" % (len(base_m), len(tip_m)))
    print()

    # --- (b) every pre-existing key, identical value --------------------------
    shared = sorted(base_m)
    differ = [k for k in shared if k not in tip_m or tip_m[k] != base_m[k]]
    print("-- PRE-EXISTING KEYS ---------------------------------------------")
    print("%d keys compared (every key BASE printed), %d differ"
          % (len(shared), len(differ)))
    if differ:
        alarms += len(differ)
        print()
        print("*** ALARM: pre-existing keys changed or vanished ***")
        for k in differ:
            print("    %-52s base=%-12s tip=%s"
                  % (k, base_m[k], tip_m.get(k, "<ABSENT IN TIP>")))
    print()

    # --- (c) keys only in tip -------------------------------------------------
    new = sorted(k for k in tip_m if k not in base_m)
    print("-- KEYS ONLY IN TIP (additive) -----------------------------------")
    print("%d new keys" % len(new))
    fam = {}
    for k in new:
        fam.setdefault(k.split("-", 1)[0], []).append(k)
    print("families: %s" % ", ".join(
        "%s-* (%d)" % (f, len(v)) for f, v in sorted(fam.items())) or "(none)")
    print()
    nonzero = [k for k in new if tip_m[k] not in ("0", "0.0")]
    print("of the new keys, %d are nonzero, %d are zero"
          % (len(nonzero), len(new) - len(nonzero)))
    print()
    for k in new:
        print("    %-56s %s" % (k, tip_m[k]))
    print()

    # --- (d) per-TU verdicts --------------------------------------------------
    bv = read_verdicts(base_jsonl)
    tv = read_verdicts(tip_jsonl)
    only_base = sorted(set(bv) - set(tv))
    only_tip = sorted(set(tv) - set(bv))
    changed = sorted(s for s in set(bv) & set(tv) if bv[s] != tv[s])
    print("-- PER-TU VERDICTS -----------------------------------------------")
    print("%d TU records in base, %d in tip; %d names in common, %d verdicts "
          "compared, %d differ"
          % (len(bv), len(tv), len(set(bv) & set(tv)), len(set(bv) & set(tv)),
             len(changed)))
    print("only-in-base %d · only-in-tip %d · changed-by-name %d"
          % (len(only_base), len(only_tip), len(changed)))
    if only_base or only_tip or changed:
        alarms += len(only_base) + len(only_tip) + len(changed)
        print()
        print("*** ALARM: the TU verdict map is not identical ***")
        for s in only_base:
            print("    only-in-base  %-56s %s" % (s, bv[s]))
        for s in only_tip:
            print("    only-in-tip   %-56s %s" % (s, tv[s]))
        for s in changed:
            print("    changed       %-56s %s -> %s" % (s, bv[s], tv[s]))
    print()

    # --- (e) headline class counts -------------------------------------------
    print("-- HEADLINE CLASS COUNTS (from the .out metric block) -------------")
    print("    %-16s %10s %10s   %s" % ("class", "base", "tip", ""))
    for k in HEADLINE:
        b, t = base_m.get(k, "<absent>"), tip_m.get(k, "<absent>")
        flag = "OK" if b == t else "*** DIFFERS ***"
        if b != t:
            alarms += 1
        print("    %-16s %10s %10s   %s" % (k, b, t, flag))
    print()
    # And the same six recomputed straight from the jsonl, as a cross-check on
    # the metric block itself.
    print("    recomputed from the .jsonl verdict maps:")
    for k in HEADLINE:
        b = sum(1 for v in bv.values() if v == k)
        t = sum(1 for v in tv.values() if v == k)
        print("    %-16s %10d %10d   %s"
              % (k, b, t, "OK" if b == t else "*** DIFFERS ***"))
        if b != t:
            alarms += 1
    print()

    print("=" * 72)
    if alarms:
        print("VERDICT: %d ALARM(S) — the change is NOT neutral" % alarms)
    else:
        print("VERDICT: NEUTRAL — %d pre-existing keys identical, %d TU "
              "verdicts identical, %d new keys added"
              % (len(shared), len(set(bv) & set(tv)), len(new)))
    print("=" * 72)
    return 1 if alarms else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
