# w-cache — the verbatim reds

Every block below is copied from a real run. Absolute paths are scrubbed by
`work/w-self2b/scrub.py`; nothing else is edited.

## 1. Board #1388, reproduced on the BASE binary at `119af05f`

Same binary, same entry, same flags, same cwd. One HIT, zero misses, either way.
Only the spelling of `--cache` differs. The FIRST run at the relative spelling —
the one that filled the entry — reported `match`; this is the second.

    $ ./work/w-cache/c2rs-base gap --list work/w-cache/tu.txt \
        --flags-file work/dc3-workload/flags.txt --cwd ../../../../dc3-decomp \
        --cache work/w-cache/cache --jobs 1

      capture cache: work/w-cache/cache
      [1/1] mismatch     src/xdk/nuispeech/xboxheap.cpp  (bytes diverge)
      match             0    0.0%
      mismatch          1  100.0%
      codegen-gap       0    0.0%
      vocab-gap         0    0.0%
      capture-fail      0    0.0%
      capture cache: 1 hit, 0 miss, 0 uncacheable  |  validator: 0 re-captured
        and agreed (0 of them only after zeroing the COFF TimeDateStamp), 0 POISONED
      mismatches:
            1 x bytes diverge

    CORRECTNESS SIGNAL: 1 mismatching TU(s), 0 replay divergence(s), 0 poisoned cache entr(ies)

...and the absolute spelling, same binary, same entry:

      capture cache: <REPO>/work/w-cache/cache
      [1/1] match        src/xdk/nuispeech/xboxheap.cpp
      match             1  100.0%
      mismatch          0    0.0%
      capture cache: 1 hit, 0 miss, 0 uncacheable  |  ...

The cached obj is **1,213 B** and contains exactly one `out.obj` string:

    Z:\<REPO>\work\w-cache\cache\38ae869280b316944bde743b6c7b59be\out.obj

## 2. The repaired behaviour — TIP binary, RELATIVE spelling, on the entry the BASE wrote

      capture cache: work/w-cache/cache
      [1/1] match        src/xdk/nuispeech/xboxheap.cpp
      match             1  100.0%
      mismatch          0    0.0%
      capture cache: 1 hit, 0 miss, 0 uncacheable  |  ...
      cache entries REFUSED on provenance: 0 (expected 0 — an entry whose recorded
        capture path is not where it is being served from is re-captured, never served)

A HIT, on a pre-existing entry: the fix invalidates nothing.

## 3. The provenance guard, fired on purpose

One entry's `objpath` line rewritten to a path it was not captured at:

      capture cache: 0 hit, 1 miss, 0 uncacheable  |  validator: 0 re-captured and agreed
        (0 of them only after zeroing the COFF TimeDateStamp), 0 POISONED
      cache entries REFUSED on provenance: 1 (expected 0 — an entry whose recorded
        capture path is not where it is being served from is re-captured, never served)
        REFUSED src/xdk/nuispeech/xboxheap.cpp: entry records its capture at
        /some/other/cache/2093bcd668a7bfbe0a27195bb1a88a0c/out.obj but is being
        served from <REPO>/work/w-cache/cache2/2093bcd668a7bfbe0a27195bb1a88a0c/out.obj
        — the obj embeds its own -Fo path, so those are not the bytes c2 would emit here
              0 |    1/1    | src/xdk/nuispeech/xboxheap.cpp [match]

A MISS plus a named refusal, and the verdict stayed `match`.

## 4. The three cache mutations

    ### undo the absolutisation
    test capture_cache::tests::a_relative_root_and_an_absolute_root_agree_on_key_and_on_served_path ... FAILED
    assertion `left == right` failed: two spellings of one cache directory must serve from one path
    test result: FAILED. 15 passed; 1 failed;

    ### drop the read-side provenance check
    test capture_cache::tests::an_entry_recorded_at_another_path_is_refused_not_served ... FAILED
    a foreign entry read as Miss — it must be REFUSED
    test result: FAILED. 15 passed; 1 failed;

    ### drop the written provenance line
    test capture_cache::tests::a_written_entry_records_its_own_capture_path ... FAILED
    write_entry did not record the capture path; meta.txt was:
    test result: FAILED. 15 passed; 1 failed;

## 5. The seven gate.sh mutations (board #1406's row)

    ############ M1 order: ARMS-FAILED read before HATCH-STALE
      FAIL  hatchred-stale-hatch-is-not-a-broken-guard wanted REFUSED/HATCH-STALE,
            got FAIL/ARMS-FAILED — ARMS-FAILED FAILED: R2 DIRTY+HATCH, C1 HATCH-ONLY
      FAIL  hatchred-every-refusal-leads-with-its-own-word 6 distinct leading words
            across 7 refusal shapes
    gate.sh --selftest: FAIL — 2 of 120 checks did not behave as required.

    ############ M2 drop the VACUOUS arm (0 red arms reads as a pass)
      FAIL  hatchred-no-red-arms-is-vacuous  wanted FAIL/VACUOUS, got PASS/none —
    gate.sh --selftest: FAIL — 1 of 120 checks did not behave as required.

    ############ M3 REFUSED no longer forfeits the unqualified headline
      FAIL  hatchred-refused-exits-zero      GATE: PASS — 2/2 lanes ran and every
            one of them graded a corpus,
      FAIL  also: a REFUSED hatch row never prints an unqualified PASS
    gate.sh --selftest: FAIL — 2 of 120 checks did not behave as required.

    ############ M4 drop the TRUNCATED arm (9 of 11 arms reads as a pass)
      FAIL  hatchred-short-run-is-not-a-pass wanted FAIL/TRUNCATED, got PASS/none —
    gate.sh --selftest: FAIL — 1 of 120 checks did not behave as required.

    ############ M5 an absent hatch-red tuple stops being a failure
      FAIL  hatchred-absent-tuple-fails-the-gate GATE: FAIL — hatch-red reported an
            unrecognized verdict ''.
    gate.sh --selftest: FAIL — 1 of 120 checks did not behave as required.

    ############ M6 fold the 11 arms into the --require-graded unit sum
      FAIL  require-graded-all-skip-fails    wanted FAIL, got PASS — GATE: SKIPPED —
            all 2 lanes, the sweep and the cross skipped, NOTHING WAS GRADED.
      FAIL  also: the demand turns an all-skip run RED
      FAIL  also: and it fails on a COUNT of graded units, never on a status string
      FAIL  also: the lanes-that-graded count is printed beside the sum
      FAIL  also: a demanded run that graded nothing never also says SKIPPED
      FAIL  hatchred-does-not-satisfy-require-graded GATE: SKIPPED — ...
      FAIL  also: an all-skip run with 11 green arms still graded nothing
    gate.sh --selftest: FAIL — 7 of 120 checks did not behave as required.

    ############ M7 a FAIL verdict no longer fails the gate
      FAIL  hatchred-fail-fails-the-gate     GATE: NOTE — hatch-red: ARMS-FAILED
            FAILED: R1 DIRTY-NOHATCH
      FAIL  hatchred-residue-fails-the-gate  GATE: NOTE — hatch-red: RESIDUE the
            arms left crates/ modified
    gate.sh --selftest: FAIL — 2 of 120 checks did not behave as required.

Restored after every one; `gate.sh --selftest: PASS — 120 cases`.
