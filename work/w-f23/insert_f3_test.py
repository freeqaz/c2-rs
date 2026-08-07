#!/usr/bin/env python3
"""Insert the F3 unit test, whose body constant is the WHOLE captured segment of
`work/w-f23/grid/f2_xboxheap_direct/` at the workload's own flags."""
import pathlib

seg = pathlib.Path('work/w-f23/seg_const.txt').read_text().rstrip()

test = '''
    /// **F3 — `xboxheap`'s own constructor in the DIRECT spelling**, the WHOLE
    /// captured segment from `work/w-f23/grid/f2_xboxheap_direct/` at the
    /// workload's own `/GR /O1 /Oi /EHsc` (board #1112). Six stores through
    /// `this` — two of them F2 addresses, one a literal — then
    /// `AllocatePageBlock(initSize)`, a member call on `this` whose two actuals
    /// are already in their slots, then the constructor's `return this`.
    ///
    /// It is the **direct** spelling on purpose: the shipped source binds a
    /// reference first (`auto& listHead = mListHead;`), which c1xx spells as a
    /// store into a LOCAL whose token then stands in two later stores' base
    /// position — board #839's obligation, refusal 5 of `w-heap`'s five, and
    /// **not** this rung's. The two spellings emit different bodies (`w-heap`
    /// §4.2), so a reader that collapsed them would emit the other body's words.
    const F3_XBOXHEAP_DIRECT: &[u8] = &[
{SEG}
    ];

    #[test]
    fn f3_reads_the_store_run_then_call_composition_and_the_bundle_refuses_it() {
        // The reader admits it: six store groups (two carrying `AddrOf`, one a
        // `Lit`) and the callee token, with the regime gate satisfied because
        // slot 0 is `this` and slot 1 is `initSize` — both already in place, so
        // the call's argument setup is EMPTY (board #1129).
        let shape = parse_segment(F3_XBOXHEAP_DIRECT, NO_LOCALS)
            .expect("the composition parses to the end of the segment");
        let BodyShape::StoreRunCall { params, ops, callee_tok } = shape else {
            panic!("expected StoreRunCall");
        };
        assert_eq!(params, vec![0x09FF, 0x09FD, 0x09FC]);
        assert_eq!(callee_tok, 0x09F6);
        // The two F2 values are the interior address `this + 8`, twice, and the
        // one literal is `mCount = 0`. That mix — an `addi`-interior producer
        // beside an `li` one — is the MIXED-KIND run `alloc::allocate` refuses
        // (#836) and `w-seam` declined to lift (#868); it is admitted here and
        // refused downstream, never modelled.
        assert_eq!(
            ops.iter().filter(|o| matches!(o, IlOp::AddrOf { off: 8 })).count(),
            2,
            "the two &mListHead values"
        );
        assert_eq!(ops.iter().filter(|o| matches!(o, IlOp::Lit(0))).count(), 1);
        assert_eq!(
            ops.iter().filter(|o| matches!(o, IlOp::StoreInd { .. })).count(),
            6,
            "six stores"
        );
        // …and the BUNDLE refuses it, which is the whole point. `IlFunction` has
        // no carrier for a composition — `ops` and the call fields are
        // alternatives `codegen::select` tries in a fixed order, with
        // `store_leaf_text` first — so a function built from these ops plus any
        // call field would emit the run and DROP THE `bl`. That is board #232's
        // exact shape, and #844's seam is what closes it.
        assert!(
            crate::func::bundle::shape_to_function(
                BodyShape::StoreRunCall {
                    params: vec![0x09FF],
                    ops: vec![
                        IlOp::Load(0x09FF),
                        IlOp::Lit(0),
                        IlOp::StoreInd { off: 0, width: 4 },
                    ],
                    callee_tok: 0x09F6,
                },
                "?ctor@@QAA@XZ",
                &None,
                &|_| Some("?callee@@AAAXXZ".to_string()),
                &|_| None,
            )
            .is_none(),
            "the composition must not reach codegen while the model cannot spell it"
        );
    }
'''.replace('{SEG}', seg)

p = pathlib.Path('crates/c2-il/src/func/body/shapes/leaf_store.rs')
s = p.read_text()
anchor = "    /// W25: the store leaf, from whole captured segments"
assert anchor in s, "anchor missing"
assert "F3_XBOXHEAP_DIRECT" not in s, "already inserted"
s = s.replace(anchor, test.strip('\n') + "\n\n" + anchor, 1)
p.write_text(s)
print("inserted")
