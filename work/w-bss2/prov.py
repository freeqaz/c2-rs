#!/usr/bin/env python3
"""Lane w-prov: the PROVENANCE STAMP every workload census must carry.

Why this exists
---------------
Lane `w-repro` spent a lane finding out that two `.gl` censuses taken thirty
minutes apart disagreed because **the corpus moved under the instrument** —
`../dc3-decomp` is a live repo other agents merge into, and it took 40+ commits
between the two runs.  The measurements survived that drift.  That was luck, not
a property: **nothing in the pipeline recorded which corpus a number was graded
against**, so the next drift would have been silent too.

This module makes it loud.  Every census writes a sidecar; every consumer that
joins two censuses **refuses** unless their stamps agree.

Design decisions, and why
-------------------------

**Sidecar, not a first record.**  `sections.jsonl` is read by a dozen scripts in
`work/` that all do `json.loads(line)` per line and would have to learn to skip a
header.  A sidecar breaks nothing.  The failure mode a sidecar invites — it goes
missing, or describes a different file — is closed by recording
`data_sha256`: a stamp that does not hash to its data file is **rejected**, so
the binding is checked rather than assumed.

**The absolute corpus path is recorded, but never in a committed file.**
`CLAUDE.md` forbids absolute machine paths in the repo, and
`work/w-bss/census/sections.jsonl` is force-added (committed) while
`work/w-bss2/glcensus.jsonl` is not.  So:

  * `write(..., committed=True)`  — strips every absolute path and **raises** if
    one survives.  What is kept is `path_rel` (relative to the repo root, and
    `null` if that would escape more than one level, which is exactly the case
    where it would leak a machine layout) and `path_sha256`.
  * `write(..., committed=False)` — additionally keeps `path_abs` in cleartext,
    because an untracked sidecar beside an untracked data file leaks nothing and
    a human debugging a mismatch wants the real path.

**`path_sha256` is the pin**, not `path_abs` — it is present on both sides of the
tracked/untracked boundary, so the cross-check works uniformly, and it is opaque.
MSVC's `?A0x<hash>` anonymous-namespace mangling is *path-derived*: two censuses
taken at different directories cannot be joined on symbol names, and w-repro
measured the cost at **20 % of the graded population**, silently, with the
printed rates intact.  That is `docs/STATUS.md` trap 5.  The pin is what turns it
into an error message.

**HEAD is captured before the run and re-checked after.**  A corpus that moved
*during* a run is the case that produced this lane, and it is a hard error, not a
note.  A whole `glcensus` takes 24 s and a whole `sections` census takes tens of
minutes; the second is straddled by a merge as a matter of course.

stdlib only, per the project rule.  Nothing here imports from `crates/`.
"""
import hashlib
import json
import os
import subprocess
import sys
import time

SCHEMA = "c2rs-census-prov/1"

# The join fields: two censuses may only be joined when all of these agree.
# `sections_sha256` is deliberately NOT here — it is a one-directional link
# (glcensus reads sections.jsonl) and is checked separately, against the file
# actually on disk.
JOIN_FIELDS = ("head", "path_sha256")


class ProvError(Exception):
    """Any provenance failure.  Consumers turn this into a banner and exit 2."""


# --------------------------------------------------------------------- hashing

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(s):
    return hashlib.sha256(s.encode("utf-8")).hexdigest()


# ------------------------------------------------------------------------ git

def _git(tree, *args):
    try:
        r = subprocess.run(("git", "-C", tree) + args,
                           capture_output=True, timeout=60)
    except Exception:
        return None
    if r.returncode != 0:
        return None
    return r.stdout.decode("utf-8", "replace").strip()


def corpus_state(corpus):
    """(head, dirty) for a source tree.  `head` is None when it is not a git
    checkout — recorded as None rather than guessed, because a corpus with no
    version is a fact a consumer needs to see, not one to paper over."""
    head = _git(corpus, "rev-parse", "HEAD")
    if head is None:
        return None, None
    st = _git(corpus, "status", "--porcelain")
    return head, bool(st)


# ----------------------------------------------------------------------- paths

def path_sha256(path):
    """The pin.  Hash of the RESOLVED path, so a symlinked or `..`-laden
    spelling of one directory pins to one value."""
    return sha256_text(os.path.realpath(path))


def path_rel(path, root):
    """`path` relative to the repo root, or None when that would encode a
    machine layout.

    The sibling default (`<repo>/../dc3-decomp`) yields `../dc3-decomp`, which is
    repo-relative and commit-safe.  Anything needing two or more levels up is
    somewhere else on this box, so it is dropped rather than committed."""
    try:
        r = os.path.relpath(os.path.realpath(path), os.path.realpath(root))
    except ValueError:
        return None
    if r.startswith(".." + os.sep + ".."):
        return None
    if os.path.isabs(r):
        return None
    return r


def _has_abs(obj):
    """Any string anywhere in the structure that looks like an absolute path."""
    if isinstance(obj, str):
        return obj.startswith("/") or (len(obj) > 2 and obj[1] == ":")
    if isinstance(obj, dict):
        return any(_has_abs(v) for v in obj.values())
    if isinstance(obj, (list, tuple)):
        return any(_has_abs(v) for v in obj)
    return False


# ------------------------------------------------------------- begin / finish

def begin(corpus):
    """Snapshot the corpus BEFORE the run.  Cheap; call it first."""
    head, dirty = corpus_state(corpus)
    return dict(corpus=os.path.realpath(corpus), head=head, dirty=dirty,
                started_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()))


def begin_write(path, b):
    with open(path, "w") as f:
        json.dump(b, f, indent=1, sort_keys=True)


def begin_read(path):
    with open(path) as f:
        return json.load(f)


def finish(b, allow_dirty=False, allow_move=False):
    """Re-check the corpus AFTER the run.  Raises unless told not to.

    Two distinct failures, and they are NOT the same thing:
      * dirty  — the corpus had uncommitted edits, so `head` does not describe
                 the bytes that were compiled.
      * moved  — the corpus advanced DURING the run, so no single commit
                 describes it.  This is the case that produced lane w-prov.
    """
    head_after, dirty_after = corpus_state(b["corpus"])
    moved = (b["head"] != head_after)
    dirty = bool(b["dirty"]) or bool(dirty_after)
    if moved and not allow_move:
        raise ProvError(
            "CORPUS MOVED DURING THE RUN: %s\n"
            "  HEAD before %s\n  HEAD after  %s\n"
            "No single commit describes what was measured. Re-run against a\n"
            "still tree, or freeze one with `git archive <sha> src`.\n"
            "(`--allow-move` records it instead, and every consumer will\n"
            "refuse the resulting census.)"
            % (b["corpus"], b["head"], head_after))
    if dirty and not allow_dirty:
        raise ProvError(
            "CORPUS IS DIRTY: %s\n"
            "  HEAD %s has uncommitted changes, so it does not describe the\n"
            "  bytes that were compiled. Commit or stash, or pass --allow-dirty\n"
            "  to record the census as unpinnable." % (b["corpus"], b["head"]))
    return head_after, dirty_after, moved


# ------------------------------------------------------------------- the stamp

def stamp(tool, data_path, b, repo_root, inputs=None,
          allow_dirty=False, allow_move=False, begin_scope="run", records=None):
    """Build the sidecar dict.  Re-checks the corpus; raises on move/dirty.

    `begin_scope` is an honesty field:
      "run"       the begin snapshot covers the whole run (compile + aggregate)
      "aggregate" it covers only the aggregation step, so drift during the
                  earlier compile phase is INVISIBLE to this stamp
    """
    head_after, dirty_after, moved = finish(b, allow_dirty, allow_move)
    p = dict(
        schema=SCHEMA,
        tool=tool,
        generated_utc=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        data_file=os.path.basename(data_path),
        data_sha256=sha256_file(data_path),
        data_records=records,
        begin_scope=begin_scope,
        corpus=dict(
            head=b["head"],
            head_after=head_after,
            moved_during_run=moved,
            dirty=bool(b["dirty"]) or bool(dirty_after),
            path_abs=b["corpus"],
            path_rel=path_rel(b["corpus"], repo_root),
            path_sha256=path_sha256(b["corpus"]),
        ),
        inputs=dict(inputs or {}),
    )
    return p


def write(data_path, p, committed):
    """Write `<data_path>.prov`.

    `committed=True` means the DATA file is tracked by git, so the sidecar will
    be too — it is stripped of every absolute path and the result is re-checked.
    The check is the point: a future edit that adds an absolute field to the
    stamp fails here rather than leaking `/home/<user>/…` into the history.
    """
    p = json.loads(json.dumps(p))          # deep copy; do not mutate the caller's
    if committed:
        p["corpus"].pop("path_abs", None)
        p["committed_safe"] = True
        if _has_abs(p):
            raise ProvError(
                "refusing to write a COMMITTED provenance sidecar containing an "
                "absolute path (CLAUDE.md: never commit absolute machine paths).\n"
                "Offending stamp: %s" % json.dumps(p, sort_keys=True))
    else:
        p["committed_safe"] = False
    out = data_path + ".prov"
    with open(out, "w") as f:
        json.dump(p, f, indent=1, sort_keys=True)
        f.write("\n")
    return out


def read(data_path, verify_hash=True):
    """Load and validate `<data_path>.prov`.  Raises `ProvError` on anything."""
    sp = data_path + ".prov"
    if not os.path.exists(sp):
        raise ProvError(
            "NO PROVENANCE for %s\n"
            "  expected %s\n"
            "  Every census must record the corpus it was graded against.\n"
            "  Regenerate with scripts/regen_census.sh, which writes it."
            % (data_path, sp))
    with open(sp) as f:
        p = json.load(f)
    if p.get("schema") != SCHEMA:
        raise ProvError("provenance schema %r, expected %r (%s)"
                        % (p.get("schema"), SCHEMA, sp))
    if p["corpus"].get("moved_during_run"):
        raise ProvError(
            "CENSUS TAKEN ACROSS A MOVING CORPUS: %s\n"
            "  HEAD %s -> %s during the run. No single commit describes it."
            % (data_path, p["corpus"]["head"], p["corpus"]["head_after"]))
    if verify_hash:
        actual = sha256_file(data_path)
        if actual != p["data_sha256"]:
            raise ProvError(
                "PROVENANCE DOES NOT DESCRIBE THIS FILE: %s\n"
                "  stamp says sha256 %s\n"
                "  file is         %s\n"
                "  The sidecar is stale, or the data file was regenerated\n"
                "  without it. Neither can be graded."
                % (data_path, p["data_sha256"], actual))
    return p


def require_join(a_path, a, b_path, b):
    """Refuse to join two censuses whose stamps disagree.  This is the PIN.

    Raises with the full disagreement, never a boolean — a checker that returns
    False is one `if` away from being ignored.
    """
    bad = []
    for f in JOIN_FIELDS:
        if a["corpus"].get(f) != b["corpus"].get(f):
            bad.append((f, a["corpus"].get(f), b["corpus"].get(f)))
    if a["inputs"].get("flags_sha256") != b["inputs"].get("flags_sha256"):
        bad.append(("flags_sha256", a["inputs"].get("flags_sha256"),
                    b["inputs"].get("flags_sha256")))
    if not bad:
        return
    lines = ["REFUSING TO JOIN TWO CENSUSES WITH DIFFERENT PROVENANCE",
             "  A: %s" % a_path, "  B: %s" % b_path, ""]
    for f, x, y in bad:
        lines.append("  %-14s A=%s" % (f, x))
        lines.append("  %-14s B=%s" % ("", y))
    if any(f == "path_sha256" for f, _, _ in bad):
        lines += [
            "",
            "  path_sha256 differs: these censuses were taken at DIFFERENT",
            "  directories. MSVC's `?A0x<hash>` anonymous-namespace mangling is",
            "  path-derived, so the symbol-name join silently loses ~20 % of the",
            "  graded population while the printed rates stay healthy",
            "  (measured: .bss 117 -> 93, .data 68 -> 53, rates 94.0 % -> 93.5 %",
            "  and 100 % -> 100 %). docs/STATUS.md trap 5. This is why the join",
            "  is pinned rather than repaired.",
        ]
    raise ProvError("\n".join(lines))


def require_input(p, key, path):
    """The one-directional link: glcensus.jsonl was built by reading
    sections.jsonl, so the file on disk now must be the one it read."""
    want = p["inputs"].get(key)
    if want is None:
        raise ProvError("stamp for %s records no %s" % (p["data_file"], key))
    got = sha256_file(path)
    if got != want:
        raise ProvError(
            "INPUT CHANGED UNDER A CENSUS: %s\n"
            "  %s was built against %s = %s\n"
            "  the file on disk is                        %s\n"
            "  Regenerate the downstream census (scripts/regen_census.sh --gl)."
            % (path, p["data_file"], key, want, got))


def describe(p):
    c = p["corpus"]
    return ("corpus %s%s  path %s  flags %s"
            % ((c["head"] or "UNVERSIONED")[:12],
               " DIRTY" if c.get("dirty") else "",
               c.get("path_rel") or c.get("path_abs") or c["path_sha256"][:12],
               (p["inputs"].get("flags_sha256") or "?")[:12]))


def banner(e):
    """Render a ProvError the way a consumer should: unmissable."""
    bar = "=" * 72
    return "\n".join((bar, "PROVENANCE CHECK FAILED", bar, str(e), bar))


# ------------------------------------------------------------------ self-check

def _selfcheck():
    """KAC-2 of `docs/rungs/_2026-08-04-w-prov-prereg.md`.

    A stamp checker that cannot fail is trap 5 wearing a lab coat, so this
    exercises **every** rejection path, not just the happy one.  Needs no
    toolchain and no corpus.
    """
    import tempfile, shutil
    tmp = tempfile.mkdtemp(prefix="prov-selfcheck-")
    ok = fail = 0

    def check(label, cond):
        nonlocal ok, fail
        if cond:
            ok += 1
            print("  PASS  %s" % label)
        else:
            fail += 1
            print("  FAIL  %s" % label)

    def raises(label, fn):
        try:
            fn()
        except ProvError:
            check(label, True)
            return
        except Exception as e:                       # noqa: BLE001
            check(label + " (wrong exception %r)" % e, False)
            return
        check(label + " (did NOT raise)", False)

    try:
        root = os.path.join(tmp, "repo")
        corpus = os.path.join(tmp, "corpus")
        os.makedirs(root)
        os.makedirs(corpus)
        data = os.path.join(root, "census.jsonl")
        open(data, "w").write('{"src":"a"}\n{"src":"b"}\n')
        flags = os.path.join(root, "flags.txt")
        open(flags, "w").write("/O1 /Oi\n")

        b = begin(corpus)                            # not a git tree -> head None
        check("unversioned corpus records head=None", b["head"] is None)

        p = stamp("selfcheck", data, b, root,
                  inputs=dict(flags_sha256=sha256_file(flags)), records=2)
        check("path_sha256 is the realpath hash",
              p["corpus"]["path_sha256"] == path_sha256(corpus))

        # committed sidecars must not carry an absolute path
        raises("committed write refuses an injected absolute path",
               lambda: write(data, dict(p, corpus=dict(p["corpus"],
                                                       path_rel="/home/x/leak")),
                             committed=True))
        sp = write(data, p, committed=True)
        blob = open(sp).read()
        check("committed sidecar has no path_abs", '"path_abs"' not in blob)
        check("committed sidecar leaks no absolute path",
              not _has_abs(json.loads(blob)))
        sp2 = write(data, p, committed=False)
        check("untracked sidecar keeps path_abs",
              '"path_abs"' in open(sp2).read())

        got = read(data)
        check("round-trips", got["data_sha256"] == p["data_sha256"])

        # every rejection path
        open(data, "a").write('{"src":"c"}\n')
        raises("rejects a stamp that does not hash to its file",
               lambda: read(data))
        write(data, stamp("selfcheck", data, b, root,
                          inputs=dict(flags_sha256=sha256_file(flags)),
                          records=3), committed=False)
        read(data)                                   # repaired

        missing = os.path.join(root, "absent.jsonl")
        open(missing, "w").write("{}\n")
        raises("rejects a missing sidecar", lambda: read(missing))

        a_p = read(data)
        b_p = json.loads(json.dumps(a_p))
        b_p["corpus"]["path_sha256"] = "0" * 64
        raises("refuses a join across different paths",
               lambda: require_join("A", a_p, "B", b_p))
        b_p = json.loads(json.dumps(a_p))
        b_p["corpus"]["head"] = "deadbeef"
        raises("refuses a join across different corpus HEADs",
               lambda: require_join("A", a_p, "B", b_p))
        b_p = json.loads(json.dumps(a_p))
        b_p["inputs"]["flags_sha256"] = "0" * 64
        raises("refuses a join across different flags",
               lambda: require_join("A", a_p, "B", b_p))
        require_join("A", a_p, "B", json.loads(json.dumps(a_p)))
        check("accepts a matched pair", True)

        moved = json.loads(json.dumps(a_p))
        moved["corpus"]["moved_during_run"] = True
        mp = os.path.join(root, "moved.jsonl")
        open(mp, "w").write("{}\n")
        moved["data_sha256"] = sha256_file(mp)
        open(mp + ".prov", "w").write(json.dumps(moved))
        raises("rejects a census taken across a moving corpus",
               lambda: read(mp))

        raises("rejects a changed one-directional input",
               lambda: require_input(a_p, "flags_sha256", data))
        require_input(dict(a_p, inputs=dict(flags_sha256=sha256_file(flags))),
                      "flags_sha256", flags)
        check("accepts an unchanged one-directional input", True)

        # a corpus that moves DURING the run
        b2 = dict(b, head="aaaa")
        raises("finish() rejects a moved corpus",
               lambda: finish(dict(b2, corpus=corpus)))
        b3 = dict(b, dirty=True)
        raises("finish() rejects a dirty corpus", lambda: finish(b3))
        finish(b3, allow_dirty=True)
        check("finish() accepts a dirty corpus under --allow-dirty", True)

        check("path_rel drops a two-level escape",
              path_rel("/opt/elsewhere/x", "/home/u/a/b/c") is None)
        check("path_rel keeps the sibling default",
              path_rel(corpus, root) == os.path.join("..", "corpus"))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    print("\nprov selfcheck: %d PASS, %d FAIL" % (ok, fail))
    return 0 if fail == 0 else 1


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "selfcheck":
        sys.exit(_selfcheck())
    if len(sys.argv) > 1:
        try:
            print(describe(read(sys.argv[1])))
        except ProvError as e:
            print(banner(e), file=sys.stderr)
            sys.exit(2)
        sys.exit(0)
    print(__doc__)
