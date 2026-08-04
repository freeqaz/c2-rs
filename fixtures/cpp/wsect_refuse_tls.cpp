// w-sect / board #174 — MUST REFUSE. A thread-local lands in `.tls$`, and its
// `.gl` data record is byte-identical to an ordinary uninitialized object in
// every field the reader had before this lane; the discriminator is the byte
// AFTER the attribute (0x10). Rule T1 (§5.8) is fitted on ten probe cells, has
// never been seen on a real TU, and `.tls$` is not one of the workload's 13
// section names — so it is worth +0 to factor C.
__declspec(thread) int t1;
