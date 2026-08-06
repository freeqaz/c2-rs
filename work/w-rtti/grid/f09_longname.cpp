// FRESH: a long template name, to re-test that ??_R0 is UNPADDED.
template <class A1, class A2, class A3>
struct AVeryLongTemplateNameForPaddingProbe {
    AVeryLongTemplateNameForPaddingProbe();
    virtual void f();
    A1 a; A2 b; A3 c;
};
template <class A1, class A2, class A3>
AVeryLongTemplateNameForPaddingProbe<A1,A2,A3>::AVeryLongTemplateNameForPaddingProbe(){}
template struct AVeryLongTemplateNameForPaddingProbe<int, char, short>;
