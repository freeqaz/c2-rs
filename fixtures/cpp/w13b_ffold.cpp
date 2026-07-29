// W13b **negative** — the algebraic identities c2's constant evaluator removes.
// Four of these five must keep refusing; the fifth must keep emitting.
//
// This fixture exists because W13b briefly mis-emitted all four. Once the port
// could lower `a + <k>` it lowered `a + 0.0f` the same way — three instructions
// against a pooled `__real@00000000` — where the live capture is a **bare
// `blr`**, no constant pooled at all. Captured behaviour:
//
//   q1  a + 0.0f  ->  4e800020                     (nothing at all; a is in f1)
//   q2  a * 1.0f  ->  4e800020
//   q4  a - 0.0f  ->  4e800020
//   q3  a / 2.0f  ->  addis/lfs __real@3f000000 ; fmuls — a reciprocal multiply,
//                     NOT fdivs, and 1/k need not be exact (a/3.0f/7.0f pools
//                     __real@3d430c31, i.e. 1/21)
//   q5  a * 0.0f  ->  addis/lfs __real@00000000 ; fmuls — NOT folded to a
//                     constant zero (signed zero and NaN make that unsafe)
//
// q5 is the load-bearing one. The over-broad gate — "refuse any constant that is
// 0.0 or 1.0" — would refuse it too; the over-eager fold — "anything times zero
// is zero" — would emit a wrong `blr`. The rule is per (operator, value) pair,
// which only a fixture holding both halves can separate.
//
// All of this is the backend's doing: c1xx hands c2 the literals verbatim, so
// none of these folds are visible in the IL.

float q1(float a) { return a + 0.0f; }
float q2(float a) { return a * 1.0f; }
float q3(float a) { return a / 2.0f; }
float q4(float a) { return a - 0.0f; }
float q5(float a) { return a * 0.0f; }
