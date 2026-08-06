// FRESH: internal linkage. Does the RTTI stay a COMDAT with Selection 2?
namespace { struct An { An(); virtual void f(); int a; }; }
An::An(){}
void* keep(){ static An a; return &a; }
