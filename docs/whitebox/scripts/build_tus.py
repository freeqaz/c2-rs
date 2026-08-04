#!/usr/bin/env python3
"""build_tus.py — recover c2.dll's original translation units from its ICE sites.

PROVENANCE — TWO TIERS, and they are not the same (see docs/whitebox/DISCLOSURE.md):

  TIER 1 (black-box, already blessed by ROADMAP.md 9.8): the *list of file names*.
    c2's internal-compiler-error path prints "compiler file '%s', line %d"
    (C1001), so the file names are literals recoverable with `strings` alone and
    are an observable output of the binary, alongside the obj, the /FAsc listing
    and the diagnostic text.

  TIER 2 (white-box): the *addresses*. Which code loads which file pointer, and
    therefore which address range belongs to which file, comes from reading the
    disassembly. Everything with an address in the output is tier 2.

Why it works: MSVC lays .text out in object-file order, so the call sites that
report file F cluster into one contiguous range. Recovering the line number
alongside the file pointer orders sites *within* a file, taking the map from
module-level to nearly function-level.

The x86 shapes emitted by the ICE macro in this build:

    mov edx, <line>          ; ba <imm32>
    mov ecx, <file-ptr>      ; b9 <imm32>
    jmp/call <reporter>      ; fastcall(ecx=file, edx=line)

    push <line>
    push <code>
    mov edx, <file-ptr>
    pop ecx
    call [<reporter-iat>]

Usage: build_tus.py <objdump.asm> <strings.tsv> <functions.tsv> <out-sites.tsv> <out-tus.tsv>
"""
import sys, re, bisect, collections

LINE_MAX = 200000       # a plausible source line number; rejects pointers/flags
WINDOW = 12             # instructions to search around the file-pointer load


def main():
    asm, strp, funp, outsites, outtus = sys.argv[1:6]

    # --- tier 1: the file names ---
    fileaddr = {}
    for l in open(strp):
        if l.startswith("#") or l.startswith("addr"):
            continue
        p = l.rstrip("\n").split("\t")
        if len(p) < 7 or "vctools" not in p[6]:
            continue
        nm = p[6].split("\\\\")[-1]
        if nm.endswith(".pdb"):
            continue
        fileaddr[int(p[0], 16)] = (nm, p[6])

    # --- functions, for mapping a site to its containing function ---
    fstart, fname = [], {}
    for i, l in enumerate(open(funp)):
        if i == 0:
            continue
        p = l.rstrip("\n").split("\t")
        if len(p) < 2:
            continue
        a = int(p[0], 16)
        fstart.append(a)
        fname[a] = int(p[1])
    fstart.sort()

    def containing(a):
        i = bisect.bisect_right(fstart, a) - 1
        if i < 0:
            return None
        s = fstart[i]
        return s if a < s + fname[s] else None

    # --- scan the disassembly ---
    ln_re = re.compile(r"^([0-9a-f]{8}):\t[0-9a-f ]+\t\s*(\w+)\s*(.*)$")
    imm_re = re.compile(r"0x([0-9a-f]+)")
    insns = []
    for line in open(asm):
        m = ln_re.match(line)
        if m:
            insns.append((int(m.group(1), 16), m.group(2), m.group(3)))

    sites = []
    for i, (a, op, args) in enumerate(insns):
        ms = imm_re.findall(args)
        hit = None
        for v in ms:
            iv = int(v, 16)
            if iv in fileaddr:
                hit = iv
                break
        if hit is None:
            continue
        # Recover the line number. Two shapes are emitted by the ICE macro in
        # this build, and they put the line in DIFFERENT places -- a naive
        # "nearest immediate" rule silently returns the error *code* for the
        # second shape, which is how this was first got wrong:
        #
        #   (a) fastcall:  mov edx,<line> ; mov ecx,<file> ; jmp <reporter>
        #   (b) stdcall:   push <line> ; push <code> ; mov edx,<file> ; pop ecx
        #
        # In (b) the line is pushed FIRST, so it is the *farther* of the two,
        # and the nearer immediate is the diagnostic code. So: walk back over the
        # contiguous run of immediate-producing instructions and take the
        # earliest one in that run.
        def imm_of(j):
            aj, opj, argsj = insns[j]
            if "[" in argsj:
                return None
            if opj == "mov":
                parts = argsj.split(",")
                if len(parts) != 2:
                    return None
                dst, src = parts[0].strip(), parts[1].strip()
                if dst not in ("edx", "ecx", "eax", "esi", "edi", "ebx"):
                    return None
                cand = src
            elif opj == "push":
                cand = argsj.strip()
            else:
                return None
            mm = re.fullmatch(r"0x([0-9a-f]+)", cand)
            if not mm:
                return None
            return int(mm.group(1), 16)

        run = []
        j = i - 1
        while j >= 0 and i - j <= WINDOW:
            v = imm_of(j)
            if v is None:
                # tolerate the one interleaved non-immediate the macro emits
                if insns[j][1] in ("pop", "nop", "lea") and run:
                    j -= 1
                    continue
                break
            if v in fileaddr:
                break
            run.append(v)
            j -= 1
        cands = [v for v in run if 0 < v <= LINE_MAX]
        line_no = cands[-1] if cands else None      # earliest in program order
        nm = fileaddr[hit][0]
        sites.append((a, nm, line_no, containing(a)))

    sites.sort()
    with open(outsites, "w") as f:
        f.write("# DISASSEMBLY-DERIVED (tier 2: addresses). File names are tier 1.\n")
        f.write("# Generated by docs/whitebox/scripts/build_tus.py — do not hand-edit.\n")
        f.write("site_addr\tfile\tline\tfunc\n")
        for a, nm, ln, fn in sites:
            f.write("%08x\t%s\t%s\t%s\n" % (a, nm, ln if ln is not None else "",
                                            "%08x" % fn if fn else ""))

    # --- per-file ranges, over *function entries* not raw sites ---
    byfile = collections.defaultdict(set)
    lines_by_file = collections.defaultdict(list)
    for a, nm, ln, fn in sites:
        if fn:
            byfile[nm].add(fn)
        if ln is not None:
            lines_by_file[nm].append((fn if fn else a, ln))

    rows = []
    for nm, fns in byfile.items():
        s = sorted(fns)
        lns = [l for _, l in lines_by_file[nm]]
        # monotonicity of line number vs address, within the file
        pairs = sorted(set(lines_by_file[nm]))
        mono = inv = 0
        for i in range(1, len(pairs)):
            if pairs[i][1] >= pairs[i - 1][1]:
                mono += 1
            else:
                inv += 1
        rows.append({
            "file": nm, "start": s[0], "end": s[-1], "nfunc": len(s),
            "nsite": sum(1 for a, n, l, fn in sites if n == nm),
            "minline": min(lns) if lns else None, "maxline": max(lns) if lns else None,
            "mono": mono, "inv": inv,
        })
    rows.sort(key=lambda r: r["start"])

    # structural check: overlap and coverage
    overlaps = []
    for i in range(len(rows) - 1):
        if rows[i + 1]["start"] <= rows[i]["end"]:
            overlaps.append((rows[i]["file"], rows[i + 1]["file"],
                             rows[i]["end"] - rows[i + 1]["start"]))

    with open(outtus, "w") as f:
        f.write("# DISASSEMBLY-DERIVED (tier 2: addresses; tier 1: file names).\n")
        f.write("# c2.dll's original translation units, recovered from its C1001 ICE sites.\n")
        f.write("# anchor_start/anchor_end bracket the functions containing an ICE site for\n")
        f.write("# that file; code BETWEEN one file's anchor_end and the next file's\n")
        f.write("# anchor_start is unattributed - a file with no ICE site is invisible here.\n")
        f.write("# Generated by docs/whitebox/scripts/build_tus.py - do not hand-edit.\n")
        f.write("file\tanchor_start\tanchor_end\tnfunc_anchored\tnsites\tmin_line\tmax_line\tline_monotone\tline_inversions\n")
        for r in rows:
            f.write("%s\t%08x\t%08x\t%d\t%d\t%s\t%s\t%d\t%d\n" % (
                r["file"], r["start"], r["end"], r["nfunc"], r["nsite"],
                r["minline"] if r["minline"] is not None else "",
                r["maxline"] if r["maxline"] is not None else "",
                r["mono"], r["inv"]))

    # --- report ---
    e = sys.stderr.write
    e("ICE sites: %d  (with line number: %d, %.1f%%)\n" % (
        len(sites), sum(1 for s in sites if s[2] is not None),
        100.0 * sum(1 for s in sites if s[2] is not None) / max(1, len(sites))))
    e("files with at least one anchored function: %d\n" % len(rows))
    e("overlaps: %d\n" % len(overlaps))
    for a, b, n in overlaps:
        e("  %s <-> %s  by %d bytes\n" % (a, b, n))
    span = rows[-1]["end"] - rows[0]["start"]
    cov = sum(r["end"] - r["start"] for r in rows)
    e("anchored span %08x..%08x = %d bytes; in-file spans sum = %d (%.1f%% coverage)\n"
      % (rows[0]["start"], rows[-1]["end"], span, cov, 100.0 * cov / span))
    tm = sum(r["mono"] for r in rows)
    ti = sum(r["inv"] for r in rows)
    e("line-vs-address monotonicity: %d in order, %d inversions (%.1f%% monotone)\n"
      % (tm, ti, 100.0 * tm / max(1, tm + ti)))


if __name__ == "__main__":
    main()
