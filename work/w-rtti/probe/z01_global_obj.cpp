// Is there a `.rdata$r` obj with NO plain `.text` COMDAT? A namespace-scope
// object of a polymorphic class: the vfptr write lands in a `??__E` dynamic
// initializer (`.text$yc`), which is a section the port ALREADY emits.
struct A { virtual void f(); int a; };
A g;
