
    /// **#839 — `xboxheap.cpp`'s constructor in its SHIPPED spelling**, the
    /// WHOLE captured segment from `work/w-bind/grid/b_target_bind/` at the
    /// workload's own `/GR /O1 /Oi /EHsc` (board #1112).
    ///
    /// The one difference from [`F3_XBOXHEAP_DIRECT`] beside it is source line
    /// 8, `auto& listHead = mListHead;`, which `c1xx` writes as
    ///
    /// ```text
    ///   26 01 0a  b9 ff 09 a6 43 81 20  33 86 41 74 08 27 a6 43 8d 20  32 86 43 90 20 4b
    /// ```
    ///
    /// after which the two remaining stores carry `b9 01 0a` — the LOCAL's token
    /// — in their base position instead of `this`.
    ///
    /// **The two cells' reference objs differ in four `.text` words** and both
    /// were captured by this lane (`b_target_bind/dis.txt`,
    /// `b_target_direct/dis.txt`, crossed by `work/w-bind/twins.sh`):
    ///
    /// ```text
    ///   BIND    li 10,0 ; stw 5,16(3) ; addi 11,3,8 ; stw 3,0(3) ; stw 10,20(3) ;
    ///           mr 31,3 ; stw 3,4(3) ; stw 11,8(3) ; stw 11,12(3)
    ///   DIRECT  addi 11,3,8 ; stw 5,16(3) ; li 10,0 ; stw 3,0(3) ; stw 3,4(3) ;
    ///           mr 31,3 ; stw 10,20(3) ; stw 11,8(3) ; stw 11,12(3)
    /// ```
    ///
    /// so a reader that gave both the same op stream would hand the emitter the
    /// other body's words. Board #1128, and board #232's direction.
    const BIND_XBOXHEAP_SHIPPED: &[u8] = &[
    ];

    /// **The bound reference at its SMALLEST** — `void fn(H* h){ BE& l =
    /// h->mListHead; l.mNext = &l; }`, the whole captured segment from
    /// `work/w-bind/grid/b_leaf_bind/`. One bind, one store, the plain run tail
    /// and no call, so it separates the base-position obligation from everything
    /// the composition adds.
    const BIND_LEAF: &[u8] = &[
    ];

    /// A `.sy` view that declares `tok` the way a captured one does: a width-4
    /// data-pointer automatic. The membership test is **positive** on purpose
    /// (`assign.rs`'s header records what absence-based reasoning cost), so a
    /// test that passed `NO_LOCALS` would be testing the refusal and not the
    /// production.
    fn ptr_local(tok: &[u32]) -> SyView<'_> {
        SyView {
            locals: &[],
            ptr_locals: tok,
            formals: Formals::AllOneRegisterByConstruction,
        }
    }

    #[test]
    fn bind_reads_the_shipped_xboxheap_spelling_and_keeps_it_apart_from_the_direct_one() {
        // The BIND spelling. The binding is carried, NOT discharged: the two
        // stores that follow it keep the local's own token in their base
        // position and their offsets are the ones INSIDE the bound object (0 and
        // 4), never the sums (8 and 12). That is the whole of board #839's
        // second obligation, and collapsing it is what would emit the DIRECT
        // body's words.
        let sy = ptr_local(&[0x010A]);
        let shape = parse_segment(BIND_XBOXHEAP_SHIPPED, sy)
            .expect("the shipped spelling parses to the end of the segment");
        let BodyShape::StoreRunBind { params, binds, ops, callee_tok, live_args } = shape else {
            panic!("expected StoreRunBind, got {shape:?}");
        };
        assert_eq!(params, vec![0xFF09, 0xFC09, 0xFD09]);
        assert_eq!(binds, vec![RefBind { tok: 0x010A, base_tok: 0xFF09, off: 8 }]);
        assert_eq!(callee_tok, Some(0xF609));
        assert_eq!(live_args, 2);
        // The last two groups — `listHead.mNext = &listHead;` and
        // `listHead.mPrev = &listHead;`. Both the BASE and the VALUE are the
        // bound local, and neither is `this`.
        assert_eq!(
            &ops[ops.len() - 6..],
            &[
                IlOp::Load(0x010A),
                IlOp::Load(0x010A),
                IlOp::StoreInd { off: 0, width: 4 },
                IlOp::Load(0x010A),
                IlOp::Load(0x010A),
                IlOp::StoreInd { off: 4, width: 4 },
            ]
        );
        // …and the DIRECT spelling of the same constructor still reads as the
        // composition it is, with `this` in those base positions and the address
        // materialised as an `AddrOf` VALUE. **The two readings are different
        // shapes**, which is the acceptance criterion of the lane that wrote
        // this test, stated as an assertion rather than as prose.
        let direct = parse_segment(F3_XBOXHEAP_DIRECT, NO_LOCALS)
            .expect("the direct spelling still parses");
        assert!(
            matches!(direct, BodyShape::StoreRunCall { .. }),
            "the direct spelling must NOT become a StoreRunBind: it has no bind, \
             and its emitted body is four words away from the bound one"
        );
    }

    #[test]
    fn bind_at_displacement_zero_stays_refused_because_the_two_spellings_are_one_body() {
        // MEASURED, and the measurement is why this is a rule and not caution:
        // `work/w-bind/grid/b_off0` and `b_off0_ctrl` — the same body with and
        // without the bind, at displacement 0 — have **byte-identical `.text`**,
        // where the target pair differs by four words. So at offset 0 the bind
        // makes no second store-base value (boards #856/#865) and admitting it
        // with its own base symbol would be a wrong reading of the schedule.
        //
        // The mutation is one byte, exactly as the two captured cells differ by
        // one byte — `w-refbind`'s #856 reproduced in the reader.
        let mut seg = BIND_LEAF.to_vec();
        let at = seg
            .windows(6)
            .position(|w| w[..5] == [0x26, 0xFB, 0x09, 0xB9, 0xF8] || false)
            .expect("the bind statement");
        let disp = seg[at..]
            .windows(5)
            .position(|w| w[..4] == [0x33, 0x86, 0x41, 0x74])
            .expect("the bind's offset add")
            + at
            + 4;
        assert_eq!(seg[disp], 0x08, "the captured bind binds at +8");
        seg[disp] = 0x00;
        assert_eq!(parse_segment(&seg, ptr_local(&[0xFB09])), None);
    }

    #[test]
    fn bind_refuses_a_destination_the_sy_layer_does_not_call_a_local() {
        // The bound token must be POSITIVELY a `.sy` automatic. Absence from
        // `.gl` proves nothing — `assign.rs`'s header records a file-scope
        // `static int sv` appearing there as `$sv`, being taken for a local, and
        // its store being silently dropped — so the same segment with an empty
        // `ptr_locals` must refuse.
        assert!(parse_segment(BIND_LEAF, ptr_local(&[0xFB09])).is_some());
        assert_eq!(parse_segment(BIND_LEAF, NO_LOCALS), None);
    }

    #[test]
    fn bind_reads_the_plain_run_tail_as_well_as_the_call_one() {
        // `b_leaf_bind` has no call: one bind, one store, the void tail. Both
        // tails are admitted because the bind is orthogonal to what ends the
        // run, and a production that took only the call tail would be fitted to
        // `xboxheap`.
        let shape = parse_segment(BIND_LEAF, ptr_local(&[0xFB09])).expect("parses");
        let BodyShape::StoreRunBind { binds, ops, callee_tok, live_args, .. } = shape else {
            panic!("expected StoreRunBind, got {shape:?}");
        };
        assert_eq!(binds, vec![RefBind { tok: 0xFB09, base_tok: 0xF809, off: 8 }]);
        assert_eq!(callee_tok, None);
        assert_eq!(live_args, 0);
        assert_eq!(
            ops,
            vec![
                IlOp::Load(0xFB09),
                IlOp::Load(0xFB09),
                IlOp::StoreInd { off: 0, width: 4 },
            ]
        );
    }
