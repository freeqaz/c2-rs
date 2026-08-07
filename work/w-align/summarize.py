#!/usr/bin/env python3
"""summarize.py — one row per cell from `glread.py`'s output. Read-only."""
import re
import sys

rows, cell, nsec, cur = [], None, None, None
for line in open(sys.argv[1]):
    if line.startswith("== "):
        cell = line[3:].split()[0]
        nsec = line.split("sections[")[1].split("]")[0]
        cur = None
    elif line.startswith("   ") and not line.startswith("      "):
        cur = {"cell": cell, "nsec": nsec, "name": line.strip()}
    elif "gl: tag=" in line and cur is not None:
        for k, v in re.findall(r"(\w+)=(\S+)", line):
            cur[k] = v
    elif "c2: section=" in line and cur is not None:
        for k, v in re.findall(r"(\w+)=(\S+)", line):
            cur["c2_" + k] = v
        rows.append(cur)
        cur = None
    elif "c2: (symbol" in line and cur is not None:
        rows.append(cur)
        cur = None

hdr = f"{'cell':27s} {'symbol':16s} {'#s':>3s} {'tag':>4s} {'mk':>3s} {'kd':>3s} " \
      f"{'lk':>3s} {'size':>5s} {'at':>3s} {'fl':>3s} {'refused':10s} " \
      f"{'c2-section':12s} {'c2align':>7s} {'c2size':>6s}"
print(hdr)
print("-" * len(hdr))
for r in rows:
    if r["name"].startswith(("__C1", "__C2", "??_C")):
        continue
    print(f"{r['cell']:27s} {r['name'][:16]:16s} {r['nsec']:>3s} "
          f"{r.get('tag','--'):>4s} {r.get('mark','--'):>3s} {r.get('kind','--'):>3s} "
          f"{r.get('link','--'):>3s} {r.get('size','--'):>5s} {r.get('attr','--'):>3s} "
          f"{r.get('flags','--'):>3s} {str(r.get('refused')):10s} "
          f"{r.get('c2_section','-'):12s} {str(r.get('c2_align','-')):>7s} "
          f"{str(r.get('c2_secsize','-')):>6s}")
