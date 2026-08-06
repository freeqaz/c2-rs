// FRESH: two template parameters, one of them a pointer type.
template <class T, class U> struct Tw { Tw(); virtual void g(); T t; U* u; };
template <class T, class U> Tw<T,U>::Tw(){}
template struct Tw<char, long>;
