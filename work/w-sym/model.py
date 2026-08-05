#!/usr/bin/env python3
"""model.py — lane w-sym. The rule battery, scored as a COMPARISON.

Two questions, kept apart on purpose:

  * the **PRODUCER emission order** — board #582, scored *conditional on the
    observed store order* so a wrong store order cannot contaminate it;
  * the **STORE order** — `w-parse`'s SYMORDER half, at 91.9 % on multi-symbol
    cells and not shipped.

Producer-order rules
    RANK      ORDER #561: the global rank (use count desc, first-use asc).
    FC        `w-alloc`'s: first consumption in the final store order.
    SYMRANK   this lane's prereg §2: (grank asc, first-use asc). REFUTED.
    QUEUE-G   each symbol GROUP holds its producers in a queue ordered by the
              GLOBAL rank restricted to that group; walking the final store
              order, a store's group is drained up to and including that
              store's producer.
    QUEUE-L   the same with the queue ordered by the group's LOCAL use counts.

Store-order rules
    SRC       source order.
    IGNORE    ORDER as shipped, symbols disregarded.
    SYMORDER  `w-parse`'s: global `u`, global position, group-restricted
              GLOBAL rank, cross-symbol pin.
    PSYM-G    each group scheduled INDEPENDENTLY by ORDER (its own `u`, its own
              positions) with the group-restricted GLOBAL rank; merged so that
              the emitted symbol pattern equals the source one.
    PSYM-L    the same with the group's LOCAL use counts.

RAISES on any path containing `holdout`.
"""
import os
import sys

W = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, W)
import symlib as S  # noqa: E402

BLOCK = 2


# ------------------------------------------------------------ the scheduler --
def order_sched(items, prod, rank, u):
    """ORDER's walk over `items` (source indices): the earliest ALLOWED store.

    `prod(k)` -> producer id or None; `rank[j]` -> the producer's rank; a store
    of rank `j` may not occupy position `< u + j`.
    """
    left, out = list(items), []
    while left:
        q = len(out)
        pick = 0
        for i, k in enumerate(left):
            j = prod(k)
            if j is None or q >= u + rank[j]:
                pick = i
                break
        out.append(left.pop(pick))
    return out


def groups_of(syms):
    return sorted(set(syms))


def local_rank(specs, syms, g):
    """The group's producers by (count WITHIN g desc, first use in g asc)."""
    ks = [k for k in range(len(specs)) if syms[k] == g and specs[k][0] == "V"]
    pos = {}
    for k in ks:
        pos.setdefault(int(specs[k][1:]), []).append(k)
    return sorted(pos, key=lambda j: (-len(pos[j]), pos[j][0]))


def restricted_rank(specs, syms, g):
    """The group's producers in the order of the GLOBAL rank."""
    have = {int(specs[k][1:]) for k in range(len(specs))
            if syms[k] == g and specs[k][0] == "V"}
    return [j for j in S.global_rank(specs) if j in have]


def store_order(row, rule):
    specs, syms = row["specs"], S.sched_syms(row)
    n = len(specs)

    def prod(k):
        return int(specs[k][1:]) if specs[k][0] == "V" else None

    if rule == "SRC":
        return list(range(n))
    if rule == "IGNORE":
        rk = {j: i for i, j in enumerate(S.global_rank(specs))}
        u = min(BLOCK, sum(1 for s in specs if s[0] != "V"))
        return order_sched(range(n), prod, rk, u)
    if rule in ("SYMORDER", "SYMORDER-U"):
        rk = {}
        for g in groups_of(syms):
            for i, j in enumerate(restricted_rank(specs, syms, g)):
                rk[(g, j)] = i

        def walk(u):
            """-> (order, whether the FALLBACK had to fire)."""
            left, out, relaxed = list(range(n)), [], False
            while left:
                q = len(out)
                pick, ok = 0, False
                for i, k in enumerate(left):
                    j = prod(k)
                    if j is not None and q < u + rk[(syms[k], j)]:
                        continue
                    if any(syms[k2] != syms[k] for k2 in left[:i]):
                        continue    # the cross-symbol pin
                    pick, ok = i, True
                    break
                relaxed = relaxed or not ok
                out.append(left.pop(pick))
            return out, relaxed

        umax = min(BLOCK, sum(1 for s in specs if s[0] != "V"))
        if rule == "SYMORDER":
            return walk(umax)[0]
        # SYMORDER-U: the largest `u` the run can actually afford. `w-sched`
        # rule 1's fallback ("if every remaining store is blocked, source order
        # wins") is DELETED, exactly as `w-order2` deleted it: instead of
        # relaxing a floor, lower `u` until no floor has to be relaxed.
        #
        # On ONE symbol this is a no-op — `order.rs`'s own enumerating test
        # shows the fallback never fires there — so it reduces to ORDER #561
        # verbatim, which is the property any multi-symbol rule must have.
        for u in range(umax, -1, -1):
            out, relaxed = walk(u)
            if not relaxed:
                return out
        return list(range(n))
    if rule in ("PSYM-G", "PSYM-L"):
        per = {}
        for g in groups_of(syms):
            ks = [k for k in range(n) if syms[k] == g]
            seq = (restricted_rank(specs, syms, g) if rule == "PSYM-G"
                   else local_rank(specs, syms, g))
            rk = {j: i for i, j in enumerate(seq)}
            u = min(BLOCK, sum(1 for k in ks if specs[k][0] != "V"))
            per[g] = order_sched(ks, prod, rk, u)
        it = {g: iter(v) for g, v in per.items()}
        return [next(it[g]) for g in syms]      # the pin: source symbol pattern
    raise ValueError(rule)


# --------------------------------------------------------- producer orders --
def producer_order(row, rule, stores):
    specs, syms = row["specs"], S.sched_syms(row)
    pr = S.producers(specs)
    if rule == "RANK":
        return S.global_rank(specs)
    slot = {k: q for q, k in enumerate(stores)}
    if rule == "FC":
        return sorted(pr, key=lambda j: min(slot[k] for k in pr[j]))
    if rule == "SYMPROD":
        # The CASE SPLIT the corpus supports, and it is not a unification:
        #   one symbol      -> the RANK order (ORDER #561), exact
        #   two or more     -> FIRST CONSUMPTION in the final store order
        # `xboxheap`'s word emits the count-2 producer first through ONE symbol
        # and the count-1 producer first through TWO, with the same statements,
        # the same registers and the same producers. Board #582.
        if len(set(syms)) == 1:
            return S.global_rank(specs)
        return sorted(pr, key=lambda j: min(slot[k] for k in pr[j]))
    if rule == "SYMRANK":
        gt = S.grank_table(specs, syms)
        return sorted(pr, key=lambda j: (
            min(gt[(syms[k], j)] for k in pr[j]), pr[j][0]))
    if rule in ("SYMMERGE", "SYMMERGE-L"):
        # Each producer belongs to the symbol group of its FIRST CONSUMPTION in
        # the final store order. Within a group the producers must come out in
        # the global rank order restricted to that group; subject to that
        # precedence, the earliest first consumption goes first.
        #
        # One group -> the constraint is total -> the global rank order, which
        # is ORDER #561 verbatim. This is a MERGE, not a sort, which is why the
        # 8,420-configuration sort search of the prereg §3 could not contain it.
        fc = {j: min(slot[k] for k in pr[j]) for j in pr}
        home = {}
        for j in pr:
            home[j] = syms[min(pr[j], key=lambda k: slot[k])]
        queues = {}
        for g in sorted(set(home.values())):
            mem = [j for j in pr if home[j] == g]
            seq = (S.global_rank(specs) if rule == "SYMMERGE"
                   else sorted(pr, key=lambda j: (-len(pr[j]), pr[j][0])))
            queues[g] = [j for j in seq if j in mem]
        head = {g: 0 for g in queues}
        out = []
        while len(out) < len(pr):
            avail = [queues[g][head[g]] for g in queues
                     if head[g] < len(queues[g])]
            j = min(avail, key=lambda x: (fc[x], x))
            out.append(j)
            head[home[j]] += 1
        return out
    if rule in ("QUEUE-G", "QUEUE-L"):
        q = {}
        for g in groups_of(syms):
            q[g] = (restricted_rank(specs, syms, g) if rule == "QUEUE-G"
                    else local_rank(specs, syms, g))
        out = []
        for k in stores:
            if specs[k][0] != "V":
                continue
            j = int(specs[k][1:])
            if j in out:
                continue
            for x in q[syms[k]]:
                if x not in out:
                    out.append(x)
                if x == j:
                    break
        for j in sorted(pr, key=lambda j: pr[j][0]):
            if j not in out:
                out.append(j)
        return out
    raise ValueError(rule)


SO_RULES = ("SRC", "IGNORE", "SYMORDER", "SYMORDER-U", "PSYM-G", "PSYM-L")
PO_RULES = ("RANK", "FC", "SYMRANK", "QUEUE-G", "QUEUE-L", "SYMMERGE",
            "SYMPROD")


def score(rows, label):
    so = {r: [0, 0] for r in SO_RULES}
    po = {r: [0, 0] for r in PO_RULES}
    pon = 0
    for row in rows:
        multi = len(set(S.sched_syms(row))) > 1
        for r in SO_RULES:
            ok = store_order(row, r) == row["stores"]
            so[r][0] += ok
            so[r][1] += ok and multi
        if len(S.producers(row["specs"])) >= 2:
            pon += 1
            for r in PO_RULES:
                ok = producer_order(row, r, row["stores"]) == row["prods"]
                po[r][0] += ok
                po[r][1] += ok and multi
    n = len(rows)
    nm = sum(1 for row in rows if len(set(S.sched_syms(row))) > 1)
    npm = sum(1 for row in rows if len(S.producers(row["specs"])) >= 2
              and len(set(S.sched_syms(row))) > 1)
    print("== %s ==   %d cells (%d multi-symbol)" % (label, n, nm))
    print("  STORE order")
    for r in SO_RULES:
        print("    %-9s %5d / %5d (%5.1f%%)   multi %5d / %5d (%5.1f%%)"
              % (r, so[r][0], n, 100.0 * so[r][0] / n, so[r][1], nm,
                 100.0 * so[r][1] / max(nm, 1)))
    print("  PRODUCER order (conditional on the OBSERVED store order), "
          "%d cells with >= 2 producers" % pon)
    for r in PO_RULES:
        print("    %-9s %5d / %5d (%5.1f%%)   multi %5d / %5d (%5.1f%%)"
              % (r, po[r][0], pon, 100.0 * po[r][0] / max(pon, 1), po[r][1],
                 npm, 100.0 * po[r][1] / max(npm, 1)))
    return so, po


def main():
    argv = sys.argv[1:]
    if "--holdout" in argv:
        rows = S.read_rows_unchecked(os.path.join(W, "holdout.tsv"))
        label = "HOLDOUT"
    elif "--external" in argv:
        rows = S.read_rows_unchecked(os.path.join(W, "external.tsv"))
        label = "EXTERNAL (xboxheap's word)"
    else:
        rows = S.read_rows(os.path.join(W, "fit.tsv"))
        label = "FIT"
    if "--tier" in argv:
        t = argv[argv.index("--tier") + 1]
        rows = [r for r in rows if r["tier"] == t]
        label += " tier %s" % t
    if not rows:
        raise SystemExit("FAIL: 0 rows")
    score(rows, label)
    return 0


if __name__ == "__main__":
    sys.exit(main())
