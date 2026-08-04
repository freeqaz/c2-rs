#!/usr/bin/env python3
"""mutate_db.py — MUT-DB: does the `db` sub-stream reach anything but debug?

`db` is sub-stream ordinal 4, read at `0x10be7f41` under `[module+0xcd8] &
0x2000` and fed to `0x10be997b` / `0x10be9892`.  The prereg's registered reading
is that it is the DEBUG (CodeView type) stream and does **not** determine `D`.

That is a null, so it is graded the way w-skip graded its null: the instrument
must be shown to reach c2 before an inert result means anything.

    P0  rewrite `db` byte-for-byte      -> the obj must be BYTE-IDENTICAL
                                          (the replay is deterministic)
    P1  replace `db` with an EMPTY stream (its 2-byte header only)
    P2  replace `db` with ANOTHER TU's `db` (a wild, well-formed value)

    M19 positive check: P1/P2 must CHANGE the obj at all, else the write never
        reached c2 and the null is about the instrument.
    M20 prediction:     everything except `.debug$*` is byte-identical and the
        DEFINED-SYMBOL set of the non-debug sections is unchanged.

M20 going red means `db` reaches the emit set, and prereg clause 3 then puts
that result above the headline.

    usage: mutate_db.py <src.cpp> [donor.cpp]
"""
import os
import shutil
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(REPO, "work", "w-roots"))
import mutate      # noqa: E402

mutate.WORK = os.path.join(HERE, "mut")
HELDOUT = os.path.join(REPO, "work", "emitpred", "magnitude", "heldout.txt")


def check_quarantine(*srcs):
    q = set(l.strip() for l in open(HELDOUT) if l.strip())
    for s in srcs:
        if s in q:
            raise SystemExit("QUARANTINED TU refused: " + s)
    print("quarantine check: %d held-out TUs, none of %s is among them"
          % (len(q), list(srcs)))


def zts(b):
    """The project's own correctness rule: zero the COFF TimeDateStamp (file
    offset 4..8) before any byte compare.  Two replays a second apart differ
    there and nowhere else, which made the P0 control red on the first run."""
    return b[:4] + b"\0\0\0\0" + b[8:]


def sections(b):
    """{name: bytes} plus the defined-symbol set restricted to non-debug."""
    nsec = struct.unpack_from("<H", b, 2)[0]
    psym = struct.unpack_from("<I", b, 8)[0]
    nsym = struct.unpack_from("<I", b, 12)[0]
    strtab = b[psym + nsym * 18:]

    def sname(raw):
        s = raw.rstrip(b"\0").decode("latin1")
        if s.startswith("/"):
            i = int(s[1:])
            e = strtab.find(b"\0", i)
            s = strtab[i:e].decode("latin1")
        return s

    secs, content = [], {}
    for i in range(nsec):
        o = 20 + i * 40
        nm = sname(b[o:o + 8])
        sz, ptr = struct.unpack_from("<II", b, o + 16)
        secs.append(nm)
        content.setdefault(nm, []).append(b[ptr:ptr + sz] if ptr else b"")

    def str_at(i):
        e = strtab.find(b"\0", i)
        return strtab[i:e].decode("latin1") if e >= 0 else ""

    defined = set()
    i = 0
    while i < nsym:
        o = psym + i * 18
        naux = b[o + 17]
        sec = struct.unpack_from("<h", b, o + 12)[0]
        if 1 <= sec <= nsec and not secs[sec - 1].startswith(".debug"):
            nm = (str_at(struct.unpack_from("<I", b, o + 4)[0])
                  if b[o:o + 4] == b"\0\0\0\0"
                  else b[o:o + 8].rstrip(b"\0").decode("latin1"))
            if nm:
                defined.add(nm)
        i += 1 + naux
    return content, defined, secs


def main():
    src = sys.argv[1]
    donor = sys.argv[2] if len(sys.argv) > 2 else None
    check_quarantine(*( [src] + ([donor] if donor else []) ))

    bdir, base, argv, pipeline_obj = mutate.capture(src)
    dbp = os.path.join(bdir, base + "db")
    db0 = open(dbp, "rb").read()

    def replay(tag):
        # ONE fixed output path for every arm: the obj embeds its own `-Fo`
        # path in `S_OBJNAME` (w-joint U-j), so a per-arm path makes the
        # instrument control P0 red for a reason that has nothing to do with
        # `db`.  This was observed on the first run and is recorded here.
        od = os.path.join(bdir, "o_fixed")
        shutil.rmtree(od, ignore_errors=True)
        return mutate.replay(bdir, base, argv, os.path.join(od, "out.obj"))

    b0, err = replay("base")
    if b0 is None:
        raise SystemExit("baseline replay failed: " + err[-1500:])
    c0, d0, _ = sections(b0)
    print("bundle %s ; db %d bytes ; baseline defined(non-debug) %d ; "
          "leader set == pipeline obj %s"
          % (base, len(db0), len(d0), mutate.leaders(pipeline_obj) == mutate.leaders(b0)))

    donor_db = None
    if donor:
        dd, dbase, _a, _o = None, None, None, None
        # capture the donor in a separate scratch dir, then restore ours
        save = mutate.WORK
        mutate.WORK = os.path.join(HERE, "mut_donor")
        dd, dbase, _a, _o = mutate.capture(donor)
        donor_db = open(os.path.join(dd, dbase + "db"), "rb").read()
        mutate.WORK = save
        print("donor %s db %d bytes" % (donor, len(donor_db)))

    arms = [("P0", "db", db0), ("P1", "db", db0[:2]), ("P3", "db", None)]
    if donor_db is not None:
        arms.insert(2, ("P2", "db", donor_db))
    # P4 is the POSITIVE CONTROL, added AFTER P1/P2 came back inert (prereg
    # clause 5 disclosure).  `in` is a stream w-mark proved c2 reads, so the
    # same substitution on it MUST change the obj -- otherwise the inert `db`
    # result is about the harness and not about `db`.
    inp = os.path.join(bdir, base + "in")
    in0 = open(inp, "rb").read()
    arms.append(("P4", "in", in0[:2]))

    rows = []
    for tag, stream, payload in arms:
        path = dbp if stream == "db" else inp
        orig = db0 if stream == "db" else in0
        if payload is None:
            os.rename(path, path + ".away")
        else:
            open(path, "wb").write(payload)
        b1, e1 = replay(tag)
        if payload is None:
            os.rename(path + ".away", path)
        else:
            open(path, "wb").write(orig)
        if b1 is None:
            print("  [%s] REPLAY FAIL: %s" % (tag, e1[-300:]))
            rows.append((tag, None, None, None, None))
            continue
        c1, d1, _ = sections(b1)
        ident = (zts(b1) == zts(b0))
        names = set(c0) | set(c1)
        nondebug_same = all(c0.get(k) == c1.get(k)
                            for k in names if not k.startswith(".debug"))
        dbg_changed = any(c0.get(k) != c1.get(k)
                          for k in names if k.startswith(".debug"))
        print("  [%s] %s=%-9s obj identical=%-5s  non-debug sections identical=%-5s"
              "  .debug changed=%-5s  defined(non-debug) gained=%d lost=%d"
              % (tag, stream,
                 "REMOVED" if payload is None else len(payload),
                 ident, nondebug_same, dbg_changed,
                 len(d1 - d0), len(d0 - d1)))
        if d1 - d0 or d0 - d1:
            print("       gained:", sorted(d1 - d0)[:6])
            print("       lost:  ", sorted(d0 - d1)[:6])
        rows.append((tag, ident, nondebug_same, dbg_changed, d1 == d0))

    print("\n---- %s" % src)
    for tag, ident, nds, dbg, dsame in rows:
        if ident is None:
            print("  %s REPLAY FAIL / c2 refused -- for P4 that IS the positive"
                  " control: the substitution reached c2" % tag)
        elif tag == "P4":
            print("  P4 POSITIVE CONTROL, `in` emptied -> obj changed: %-5s"
                  " (if False, the substitution never reaches c2)" % (not ident))
        elif tag == "P0":
            print("  P0 no-op rewrite -> obj byte-identical: %s   (instrument control)"
                  % ident)
        else:
            print("  %s M19 obj changed at all: %-5s | M20 non-debug identical: %-5s"
                  " AND defined set unchanged: %s" % (tag, not ident, nds, dsame))


if __name__ == "__main__":
    main()
