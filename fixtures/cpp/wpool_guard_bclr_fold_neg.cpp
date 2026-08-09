// ### **THIS CELL CONVERTED. IT IS NO LONGER A NEGATIVE** — lane `w-pool2`
// ### (#2591), 2026-08-09. The `_neg` in the filename is `w-pool`'s and is kept
// ### so its rung still names the file it shipped; everything below this banner
// ### is `w-pool`'s text, left as written.
//
// `shapes::pool_free_list` admits this body and `codegen::pool_free_list` emits
// its six words byte-exact against real `c2.dll` at `/O1` — it *is*
// `?Free@Pool@@QAAXPAX@Z`, and `src/system/utl/Pool.cpp` converted on it (TU
// match 21 -> 22). **w-pool's fence fired exactly as designed**: the assertion
// in `crates/c2-harness/tests/pool_cells.rs` failed the build, and the cell is
// re-stated there rather than quietly relaxed.
//
// The reading below is right about the bytes and wrong about one sentence.
// `w-pool` §3.1 closes with *"adding a construct never makes a body more
// acceptable"*, and **this cell refutes it**: cell B — this body with the guard
// deleted — is STILL blocked at `expr-op-0x27`, because without a guard it
// reaches `leaf_store::collect_store_run` and dies on the `value_is_load`
// clause. A longer body is acceptable where the shorter one it contains is not,
// because acceptance runs through whole-body productions rather than through an
// incrementally widening expression grammar. That is `w-biquad` #2531's point,
// arriving from the other side.
//
// What is untouched: `leaf_store`'s value clause. `w-pool2` declined to widen it
// (403,879 workload bodies carry that key), so cell B's fence is still live and
// still the one that matters.
//
// ---
//
// **MUST REFUSE — lane `w-pool` (#2564).** `wpool_store_run_member_value_neg.cpp`
// with **one thing added**: a null guard whose arm is a bare `return`.
//
// This body **is** `?Free@Pool@@QAAXPAX@Z`, the second of
// `src/system/utl/Pool.cpp`'s three functions, verbatim — 24 bytes:
//
//     2b040000  cmplwi cr6,r4,0
//     4d9a0020  bclr   12,26      <- FOLD BAND 2: the guard is a conditional RETURN
//     81630000  lwz    r11,0(r3)
//     91640000  stw    r11,0(r4)
//     90830000  stw    r4,0(r3)
//     4e800020  blr
//
// ## It carries a SECOND blocker, and that is the whole reason it is a separate
// ## cell from the one above it
//
// The run's value clause (`expr-op-0x27`) is still owed here — adding a guard
// does not repair it. What this cell adds is the `bclr` fold, and the port
// declines it **by name**: `codegen::cond_tail`'s own doc comment ends
//
//     A shape where one arm falls through to the epilogue is fold band 2
//     (a `bclr`) and is out of class.
//
// so band 2 is a declared exclusion rather than an unrecorded gap. Two of
// `Pool.cpp`'s three functions are band-2 folds (`?Alloc@Pool@@QAAPAXXZ` is the
// other, 28 bytes, the same `bclr 12,26` after a `mr`/`lwz` park), and neither
// has any branch **target** at all — which is why the whole obj carries **zero**
// `$M`/`$T` label symbols and the label channel is owed nothing on this TU.
//
// ## Why the pair matters more than either cell
//
// With only this file in the corpus, `Pool.cpp` reads as one repair from a
// match: close the fold and the TU converts. It does not. Its first two
// functions each owe **both** rungs, and its constructor owes four more that
// neither cell here touches — a signed divide with its two branchless `twi`
// guards, interleaved with unrelated stores; an entry guard `bf 25` over the
// loop; and an `mtctr`/`bdnz` counted loop whose body is four words. **Eight
// reader rungs from an empty chain sink, terminal `expr-jump`** (this lane's
// `work/w-pool/ladder.sh`), against `w-conv`'s inherited **7**.
//
// A single refusal fixture would have said none of that, which is the same
// shape #2506 warns about one level up: `{body-out-of-class}` being a TU's only
// gate cause is not a price, because the reader gate stands in front of every
// emitter question and answers none of them.

struct WPoolP {
    char *mFree;
};

void wpool_guard_bclr_fold(WPoolP *p, void *v) {
    if (!v) {
        return;
    }
    *(void **)v = p->mFree;
    p->mFree = (char *)v;
}
