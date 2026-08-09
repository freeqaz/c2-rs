// v3 — the leaf forwards to an EXTERNAL, isolating the INTRA-TU call edge from vsnprnc.cpp's TWO functions with C++ linkage and no other change.
// Isolates `extern "C"` / the va_list typedefs from the two shapes.
typedef unsigned int size_t_;
typedef int (*output_fn_t)(void);
extern "C" int *_errno(void);
int vsnprintf_helper_long(output_fn_t, char *, size_t_, const char *, void *, void *);
int output_s_l_long(void);
extern "C" void _invalid_parameter_noinfo(void);

int vsprintf_s_l_long(char *buffer, size_t_ sizeInBytes, const char *format, void *locale, void *argptr) {
    int result;
    int *err_ptr;
    int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = _errno();
        err_val = 22;
    } else {
        result = vsnprintf_helper_long(output_s_l_long, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) { buffer[0] = '\0'; }
        if (result != -2) { return result; }
        err_ptr = _errno();
        err_val = 34;
    }
    *err_ptr = err_val;
    _invalid_parameter_noinfo();
    return -1;
}

int other_target_long(char *, size_t_, const char *, void *, void *);
int vsprintf_s_long(char *buffer, size_t_ sizeInBytes, const char *format, void *argptr) {
    return other_target_long(buffer, sizeInBytes, format, 0, argptr);
}
