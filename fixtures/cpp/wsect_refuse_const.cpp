// w-sect / board #174 — MUST REFUSE. `extern const` lands in a non-COMDAT
// `.rdata`, not in `.data`, and is KEPT even when unreferenced because the
// linkage forces emission (§4.4).
// Its `.gl` frame is `00 04` (read-only) where an ordinary object is `00 02`,
// so `data_object_at` refuses it — and the recognizer's exhaustive accounting
// is what turns that refusal into a TU-level refusal instead of a silently
// missing section.
extern const int ce = 9;
