#!/usr/bin/env python3
"""Fixture verdicts compared BY NAME between two binaries at one mode."""
import json, sys
def rows(p):
    d={}
    for line in open(p):
        r=json.loads(line)
        if "src" in r: d[r["src"]]=r["class"]
    return d
a,b=rows(sys.argv[1]),rows(sys.argv[2])
ch=[(s,a[s],b[s]) for s in a if s in b and a[s]!=b[s]]
print("%s vs %s: %d/%d fixtures, changed by name %d, only-in-a %d, only-in-b %d"
      % (sys.argv[1].split('/')[-1], sys.argv[2].split('/')[-1], len(a), len(b),
         len(ch), len(set(a)-set(b)), len(set(b)-set(a))))
for s,x,y in ch: print("   ",s,x,"->",y)
