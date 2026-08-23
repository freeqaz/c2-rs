// w4f01 - the `4F 01` source-line record's VI32 width, WIDE half of a twin.
// Board #3443; lane w-4f01; docs/rungs/2026-08-23-w-4f01.md.
//
// Every one of this file's fifteen `4F 01` records is the escaped seven-byte
// form (source lines 100003..100018).
//
// WHY `100000` AND NOT `128`. The straddle cell already proves the boundary.
// What this one adds is a value wide by a wide margin: 100003 is 0x000186A3, so
// its four payload bytes are `a3 86 01 00` - three non-zero. A value just over
// the boundary (128 = `80 00 00 00`) leaves three zero bytes, which several
// plausible wrong rules consume by accident. This one does not let them.
//
// It also exercises the two NESTED records that `w-read-r9`'s defect table did
// not name and that lane w-4f01 found: the `4F 01` inside the block-start
// marker `4F 02 20 00 4F 01 <VI32>` and inside the module end `... 4D`.
// `4F 02 20 00` is itself exactly a 4-byte record (`docs/whitebox/ref/P_SUB4F.md`
// section 4: sub `0x02`, format `73`, VARU payload), so the `4F 01` after it is
// a separate record with its own VI32 - 7 bytes narrow and 11 wide for the block
// start, 8 and 12 for the module end. The old module-end decoder looked for its
// trailing `4D` at a fixed `p+7`, did not find it, and refused the whole record.
//
// Those nested payloads also settled the field's MEANING: they hold the module's
// first and last source lines, not the "statement/block index" the codec's doc
// comments called them. No previous fixture could separate those two readings,
// because every one is a one-line function below line 128 where a line number
// and a small per-function index are the same single byte.
//
// THE THREE `w4f01_line_*.cpp` FILES ARE A TWIN SET. Their bodies below the
// `#line` directive are BYTE-IDENTICAL; the directive is the only difference.
// That is the design: it is `docs/whitebox/scripts/sub4f_probe.py --grid`'s
// method moved into the tracked corpus, where the grading rule is internal
// consistency between cells rather than a guess about where c2 puts a marker.
//
//   w4f01_line_narrow.cpp    #line 2       15 records, ALL 3 bytes
//   w4f01_line_straddle.cpp  #line 122     15 records, 3 narrow + 12 WIDE
//   w4f01_line_wide.cpp      #line 100000  15 records, ALL 7 bytes
//
// (Counts measured on real captures at this lane's tip, not predicted.)
//
// THE RULE. `4F 01`'s payload is a VI32 varint (`c2.dll` `0x10c1f9e9`): one byte
// when the value is < 0x80, else the escape `0x80` followed by four LE bytes.
// The record is therefore 3 bytes below source line 128 and 7 at or above it.
// Until 2026-08-23 four sites in this tree read a fixed three — and **every
// fixture in `fixtures/cpp` sat below line 128**, where a fixed-byte read and a
// VI32 read consume the same three bytes and agree. The defect was green on the
// entire corpus and no gate had ever seen it. Board #3443.
//
// WHY THIS BODY, AND NOT A SIMPLER ONE. The shape is `w9_rel_signed.cpp`'s
// conditional tail call, cut to two relations. That is not decoration — it was
// chosen by measurement after a simpler candidate FAILED to fence. A file of
// one-line `int f(int,int){return a+b;}` functions is also `Port=Match` and also
// carries wide markers, and it passes **just as green with the width rule
// deliberately re-broken**: for that class the port never reaches the varint
// line reader at all. A fixture that cannot fail is not a fence.
//
// This body does reach it. With `eat_opt_stmt_marker` re-broken to the old
// fixed-3 rule, the straddle and wide cells drop `Port=Match` ->
// `Port=NotImplemented`, and restoring the VI32 read brings them back. That
// control is in the rung, and it is what these three files rest on.
//
// WHAT THE BYTE JUDGE CAN AND CANNOT SEE HERE — read this before quoting a
// green verdict. **The source line does not reach the obj at the fixture
// profile.** Measured, not assumed: compiled from an identical path, the
// narrow / straddle / wide objs differ only in bytes lying inside the harness's
// own embedded `...\out.obj` scratch path. So the judge cannot catch a wrong
// line *value*. What it catches is desynchronization: a decoder that
// mis-measures the record walks off, the parse blocks, and the port refuses —
// the verdict moves Match -> NotImplemented. These fixtures fence the width
// through refusal-vs-emit, not through differing bytes.

#line 100000
void g2(void *, unsigned long);
void h3(void *, unsigned long, void *);

void s_eq(void *v1, unsigned long ul, int a) {
    if (a == 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}

void s_lt(void *v1, unsigned long ul, int a) {
    if (a < 0) {
        g2(v1, ul);
        return;
    }
    h3(v1, 0, 0);
}
