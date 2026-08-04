// W8 (the CFG step, fold band 3): a two-arm conditional tail call — the port's
// FIRST conditional branch.
//
// This is `?MemFree@NUISPEECH@@YAXPAX0K@Z` from the dc3 workload's
// `src/xdk/nuispeech/xboxmem.cpp`, reduced to its externals. The reference
// emits one `.text` COMDAT of 0x24 bytes, nrel 2, no `.pdata`:
//
//     mr     r11,r4        <- entry: v2 is wanted by BOTH arms, at DIFFERENT
//     cmplwi cr6,r3,0         registers, so it is parked in the scratch
//     bne    cr6,+16       <- the edge to the ELSE block: the NEGATION of the
//     mr     r4,r5            IL relation, because `38` is brFALSE
//     mr     r3,r11
//     b      g2            <- REL24; the THEN block is the fall-through
//     mr     r5,r11
//     li     r4,0
//     b      h3            <- REL24
//
// Three things this fixture pins that nothing else in the corpus does:
//
//   * the `bc` carries its TRUE self-relative displacement and takes NO
//     relocation, while the two `b`s carry a section-start-relative placeholder
//     and take one each — the same opcode, two encodings (docs/CFG_SHAPE.md
//     §3.3, board #191);
//   * a POINTER null-check is an UNSIGNED compare, so `cmplwi` and not `cmpwi`
//     (§3.2). The relational opcodes are sign-agnostic; only the operand TYPE
//     triple says which;
//   * the epilogue is never materialized. Both arms leave through a tail call,
//     which is what puts this body in fold band 3 — see w8_cond_tail_neg.cpp
//     for the band-2 shape that must stay refused.
void g2(void *, unsigned long);
void h3(void *, unsigned long, void *);

void f(void *v1, void *v2, unsigned long ul) {
    if (v1 == 0) {
        g2(v2, ul);
        return;
    }
    h3(v1, 0, v2);
}
