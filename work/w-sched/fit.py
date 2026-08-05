#!/usr/bin/env python3
"""fit.py — lane w-sched. Search the declared list-scheduler family against the
FIT partition only.

This script must never open holdout.tsv. It asserts that.

Family (prereg §3, widened in §3.1 of the findings and counted there):
  direction  forward | backward
  latency    L in 1..6            (producer -> consumer, in issue slots)
  priority   a lexicographic key built from up to 3 signed features drawn from
             {natural index, source statement index, is_producer, is_consumer,
              depth, height}
  memory     two stores are ordered iff their base SYMBOLS differ; same-symbol
             stores at distinct constant offsets are independent
"""
import itertools
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
FIT = os.path.join(W, "fit.tsv")
assert "holdout" not in FIT


# ---------------------------------------------------------------- the DAG ---
class Node:
    __slots__ = ("kind", "stmt", "nat", "deps", "base", "off")

    def __init__(self, kind, stmt, nat, deps, base=None, off=None):
        self.kind = kind      # 'P' or 'S'
        self.stmt = stmt      # source statement index (producer: first consumer)
        self.nat = nat        # index in the natural (unscheduled) order
        self.deps = deps      # list of (node_index, latency_kind)
        self.base = base
        self.off = off


def build(specs, base):
    """specs/base -> node list. Producers are shared when the spec repeats."""
    nodes = []
    prod_of = {}                      # spec-string -> producer node index
    order = []                        # natural order
    for k, s in enumerate(specs):
        if s in ("F", "T"):
            continue
        if s not in prod_of:
            prod_of[s] = len(nodes)
            nodes.append(Node("P", k, 0, []))
            order.append(prod_of[s])
    # rebuild in natural order: producer just before its first consumer
    nodes = []
    prod_of = {}
    for k, s in enumerate(specs):
        if s not in ("F", "T") and s not in prod_of:
            prod_of[s] = len(nodes)
            nodes.append(Node("P", k, len(nodes), []))
        st = Node("S", k, len(nodes), [], base[k], k)
        if s not in ("F", "T"):
            st.deps = [prod_of[s]]
        nodes.append(st)
    # memory order: a store is also dependent (latency 1) on every earlier
    # store with a DIFFERENT base symbol
    stores = [i for i, n in enumerate(nodes) if n.kind == "S"]
    for a_i in range(len(stores)):
        for b_i in range(a_i):
            a, b = nodes[stores[a_i]], nodes[stores[b_i]]
            if a.base != b.base:
                a.deps = a.deps + [("mem", stores[b_i])]
    return nodes


def depths(nodes, L):
    d = [0] * len(nodes)
    for i, n in enumerate(nodes):
        for dep in n.deps:
            if isinstance(dep, tuple):
                d[i] = max(d[i], d[dep[1]] + 1)
            else:
                d[i] = max(d[i], d[dep] + L)
    return d


def heights(nodes, L):
    h = [0] * len(nodes)
    for i in range(len(nodes) - 1, -1, -1):
        for dep in nodes[i].deps:
            if isinstance(dep, tuple):
                h[dep[1]] = max(h[dep[1]], h[i] + 1)
            else:
                h[dep] = max(h[dep], h[i] + L)
    return h


FEATS = ("nat", "stmt", "isP", "isS", "dep", "hgt")


def feature(nodes, i, f, dep, hgt):
    n = nodes[i]
    if f == "nat":
        return n.nat
    if f == "stmt":
        return n.stmt
    if f == "isP":
        return 1 if n.kind == "P" else 0
    if f == "isS":
        return 1 if n.kind == "S" else 0
    if f == "dep":
        return dep[i]
    return hgt[i]


def schedule(nodes, L, key, backward):
    """Greedy list schedule. `key` is a tuple of (feature, sign)."""
    n = len(nodes)
    dep, hgt = depths(nodes, L), heights(nodes, L)
    if backward:
        # reverse the DAG: successors become predecessors
        succ = [[] for _ in range(n)]
        for i, nd in enumerate(nodes):
            for d in nd.deps:
                j, lat = (d[1], 1) if isinstance(d, tuple) else (d, L)
                succ[j].append((i, lat))
        pending = [len(s) for s in succ]
        placed = [None] * n
        slot_of = {}
        out = []
        for t in range(n):
            ready = []
            for i in range(n):
                if placed[i] is not None or pending[i]:
                    continue
                if all(slot_of[j] <= t - lat for j, lat in succ[i]):
                    ready.append(i)
            if not ready:
                return None
            pick = min(ready, key=lambda i: tuple(
                sg * feature(nodes, i, f, dep, hgt) for f, sg in key))
            placed[pick] = t
            slot_of[pick] = t
            out.append(pick)
            for d in nodes[pick].deps:
                j = d[1] if isinstance(d, tuple) else d
                pending[j] -= 1
        return list(reversed(out))
    # forward
    placed = [None] * n
    out = []
    for t in range(n):
        ready = []
        for i in range(n):
            if placed[i] is not None:
                continue
            ok = True
            for d in nodes[i].deps:
                j, lat = (d[1], 1) if isinstance(d, tuple) else (d, L)
                if placed[j] is None or placed[j] + lat > t:
                    ok = False
                    break
            if ok:
                ready.append(i)
        if not ready:
            return None
        pick = min(ready, key=lambda i: tuple(
            sg * feature(nodes, i, f, dep, hgt) for f, sg in key))
        placed[pick] = t
        out.append(pick)
    return out


def render(nodes, order):
    return " ".join(("P%d" if nodes[i].kind == "P" else "S%d") % nodes[i].stmt
                    for i in order)


# ---------------------------------------------------------------- search ---
def load(path):
    rows = []
    for line in open(path).read().splitlines()[1:]:
        if not line.strip():
            continue
        cid, tier, specs, base, emitted, _ = line.split("\t")
        rows.append((cid, int(tier), specs.split(","), list(base), emitted))
    return rows


def keys(maxlen=3):
    for r in range(1, maxlen + 1):
        for fs in itertools.permutations(FEATS, r):
            for sgs in itertools.product((1, -1), repeat=r):
                yield tuple(zip(fs, sgs))


def main():
    rows = load(FIT)
    print("fit cells: %d" % len(rows))
    best = []
    for backward in (False, True):
        for L in range(1, 7):
            for key in keys(3):
                hit = 0
                for cid, tier, specs, base, emitted in rows:
                    nodes = build(specs, base)
                    o = schedule(nodes, L, key, backward)
                    if o and render(nodes, o) == emitted:
                        hit += 1
                best.append((hit, backward, L, key))
    best.sort(key=lambda x: -x[0])
    print("configs searched: %d" % len(best))
    for hit, bw, L, key in best[:8]:
        print("  %4d/%d  %-8s L=%d  %s" % (hit, len(rows),
                                           "backward" if bw else "forward", L,
                                           key))
    # per-tier breakdown of the winner
    hit, bw, L, key = best[0]
    per = {}
    for cid, tier, specs, base, emitted in rows:
        nodes = build(specs, base)
        o = schedule(nodes, L, key, bw)
        ok = bool(o) and render(nodes, o) == emitted
        a, b = per.get(tier, (0, 0))
        per[tier] = (a + (1 if ok else 0), b + 1)
    print("winner per tier: " + "  ".join("t%d %d/%d" % (t, v[0], v[1])
                                          for t, v in sorted(per.items())))
    return best


if __name__ == "__main__":
    main()
