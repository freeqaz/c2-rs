// **w-tu3 / board #503** — the INTER-FUNCTION LABEL STRIDE cell nothing in the
// corpus had: leaf functions **between** two framed ones.
//
// `wunw_framed_pair.cpp` pins framed→framed (stride 5 under `/Gy`, 4 packed) and
// `wunw_two_leaves_framed.cpp` pins leaf,leaf→framed. Neither has a leaf *in the
// middle*, so neither can tell a stride rule that accumulates per intervening
// function from one that simply doubles for the second frame. This TU can.
//
// Lane `w-tu2` measured the rule through **real c2** at the workload's own
// `/O1 /Oi /EHsc /GR` over a 36-cell cross product with six shapes held out
// before any fit and **no free parameter** (board #481):
//
//     inter-function stride = 5 + 1*(leaf/tail fns between) + 5*(framed between)
//
// and it predicted `src/xdk/nuispeech/mmio.cpp`'s own two gaps out of sample —
// 5 and 10 predicted, 5 and 10 observed. **This fixture is that second gap**:
// `mmioSetInfo` → `mmioClose` across five leaf stubs. Predicted `?f2`'s `$M` at
// `?f1`'s `$M` + **10** under `/Gy` (`/O1`, `/O2`), + **9** packed (`/Ox`).
//
// The rule is not new code — `coff::label::plan_labels` has charged 5-per-framed
// and 1-per-leaf all along, and w-tu2 measured that loop from outside without
// reading it. What was missing was a cell where the two candidate readings
// differ, and the machine-checked pin
// (`the_inter_function_label_stride_is_a_constant_and_the_source_lead_cancels`).
//
// **It does NOT pin the intra-function charge.** Every function here has label
// lead 0. Where a *control-flow-bearing* function's own labels start is board
// #286/#482 and is still open — `ifelse` at +3 with one `if` against `if3_ret`
// at +3 with three is the cell that kills the obvious rule.
int g(int);
int f1(int a) { return g(a) + 1; }
int s1(int a) { return a + 1; }
int s2(int a) { return a + 2; }
int s3(int a) { return a + 3; }
int s4(int a) { return a + 4; }
int s5(int a) { return a + 5; }
int f2(int a) { return g(a) + 2; }
