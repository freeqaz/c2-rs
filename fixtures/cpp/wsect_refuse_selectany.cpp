// w-sect / board #174 — MUST REFUSE. `__declspec(selectany)` makes the section
// itself a COMDAT (Selection 2 ANY, characteristics `...1040`), which is a
// different section entirely.
// It refuses on the `.gl` ATTRIBUTE byte: `E0` here where `int sa = 3;` spells
// `80`. So board #172's "COMDAT-ness is not in tag 9" is about the SECTION
// record and does not carry over to the DATA record.
__declspec(selectany) int sa = 3;
