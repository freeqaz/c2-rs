// w-decouple `_neg` — the MUST-FAIL cell for the whole varargs conjunction, and
// the one place this lane could have shipped a wrong obj instead of a refusal.
//
// `mangled_is_varargs` is `name.ends_with("ZZ")`. Its own doc names the coupling
// it rested on: *"An `extern "C"` variadic function has an undecorated name and
// is invisible here. That is covered, for a different reason that must not be
// quietly relied upon: `gl_defined_names` accepts only `?…@@…` forms… **If that
// ever loosens, this gate stops covering C variadics**"*. `NameFit`'s wide walk
// is that loosening.
//
// The parser cannot see the difference. `cva.cpp`'s `.ex` and `cnv.cpp`'s are
// **byte-identical** — 2,751 bytes, `cmp`-checked (`work/w-decouple/probe/`) —
// and the objs are not:
//
//     cnv   .text 8 B     addi 3,3,1 ; blr                    5 sections
//     cva   .text 36 B    7 register-home `std`s, then the    6 sections
//                         same two instructions               (+ .pdata)
//
// So an emit here is 28 bytes and a whole section wrong. What sees it is the
// `.gl` record's own flags byte at `name_nul + 5`: `0x40`, measured on 3 of 3
// variadic records against 7 of 7 others in `work/w-decouple/probe/vgrid.cpp`.
//
// **Delete the `record_is_varargs` clause and this cell goes `Port=Mismatch`,
// not `NotImplemented`** — the mutation deletes the whole conjunction, which is
// what #2698/#2699 require of a merged clause's must-fail cell.

extern "C" int v_s(int a, ...) { return a + 1; }
