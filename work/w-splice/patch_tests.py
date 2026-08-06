#!/usr/bin/env python3
"""patch_tests.py — replace the one-level test with the fixpoint tests.

Lane w-splice scratch. Applied once; kept so the edit is in the history rather
than only in a diff.
"""
import sys

p = "crates/c2-core/src/splice.rs"
s = open(p).read()
old = '''    /// **THE ONE-LEVEL RESTRICTION.** `h` is lowerable, `g` splices `h`, and
    /// `f` splices **`g`'s own lowering** — a branch to `?g`… no: `g`'s own
    /// lowering is a branch to `?h`, so `f` gets that. What `f` must NOT get is
    /// `h`'s body, because nothing in this rung measured whether c2 closes the
    /// chain (`t11`).
    #[test]
    fn the_splice_takes_exactly_one_level() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g@@YAHH@Z"),
        ];
        let h = spliced(&funcs, 1).expect("?g splices ?h");
        let f = spliced(&funcs, 2).expect("?f splices ?g's own lowering");
        assert_eq!(h.len(), 8, "?h is `addi r3,r3,1 ; blr`");
        assert_eq!(
            f,
            vec![0x48, 0x00, 0x00, 0x00],
            "THE SPLICE CLOSED A CHAIN IT WAS NOT GRADED ON: ?f must get ?g's \\
             OWN lowering — one branch word — and never ?h's body. Whether c2 \\
             closes it is work/w-splice/'s t11 and is a separate rung"
        );
    }'''
new = '''    /// **THE FIXPOINT** — `t11`, and the 150 relocation witnesses that forced
    /// it. `h` is lowerable, `g` splices `h`, and `f` must get **`h`'s body**
    /// and not `g`'s one branch word.
    ///
    /// c2 closes the chain: `t11` compiles this exact source and c2 emits `?h`'s
    /// two words for all three functions. The one-level rule that shipped first
    /// named the intermediate in **150 of 945** spliced functions' relocations
    /// where c2 named the end.
    #[test]
    fn the_splice_closes_the_chain() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g@@YAHH@Z"),
        ];
        let h = spliced(&funcs, 1).expect("?g splices ?h");
        let f = spliced(&funcs, 2).expect("?f splices through ?g to ?h");
        assert_eq!(h.len(), 8, "?h is `addi r3,r3,1 ; blr`");
        assert_eq!(
            f, h,
            "THE CHAIN WAS NOT CLOSED: ?f must get ?h's BODY, not ?g's branch \\
             word. GRID-T t11 grades c2 emitting ?h's two words for all three, \\
             and the workload's relocation check found the one-level rule \\
             naming the intermediate 150 times"
        );
    }

    /// A chain of four, and every member below the top gets the same body. The
    /// depth is not special-cased anywhere, which is the property this pins.
    #[test]
    fn the_chain_closes_at_every_depth() {
        let funcs = vec![
            leaf("?h@@YAHH@Z"),
            tail("?g3@@YAHH@Z", "?h@@YAHH@Z"),
            tail("?g2@@YAHH@Z", "?g3@@YAHH@Z"),
            tail("?g1@@YAHH@Z", "?g2@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?g1@@YAHH@Z"),
        ];
        let want = spliced(&funcs, 1).expect("?g3 splices ?h");
        for i in 2..funcs.len() {
            assert_eq!(
                spliced(&funcs, i).as_ref(),
                Some(&want),
                "{} did not reach the chain's end",
                funcs[i].mangled_name
            );
        }
    }

    /// **A CYCLE TERMINATES AND REFUSES.** `?a` splices `?b` splices `?a`: the
    /// walk repeats a name, and a repeated name is the refusal rather than a
    /// deeper step. `elide.rs`'s least fixpoint never seeds a cycle and never
    /// admits one; this reaches the same answer from the other direction.
    #[test]
    fn a_chain_cycle_terminates_and_refuses() {
        let funcs = vec![
            tail("?a@@YAHH@Z", "?b@@YAHH@Z"),
            tail("?b@@YAHH@Z", "?a@@YAHH@Z"),
            tail("?f@@YAHH@Z", "?a@@YAHH@Z"),
        ];
        for i in 0..funcs.len() {
            assert!(
                spliced(&funcs, i).is_none(),
                "A CYCLE WAS SPLICED: the walk must refuse, not recurse — {}",
                funcs[i].mangled_name
            );
        }
    }'''
if old not in s:
    sys.exit("the one-level test is not present — already patched?")
open(p, "w").write(s.replace(old, new))
print("patched")
