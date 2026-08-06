// FRESH: an explicit template instantiation. The mangled middle carries `?$`.
template <class T> struct Tm { Tm(); virtual void f(); T t; };
template <class T> Tm<T>::Tm(){}
template struct Tm<int>;
