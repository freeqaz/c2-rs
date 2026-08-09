// x_deref_merged — store form `*buffer = 0`, error arms `merged`.
typedef unsigned int size_t_;
typedef int (*output_fn_t)(void);
extern "C" int *_errno(void);
int vsnprintf_helper_long(output_fn_t, char *, size_t_, const char *, void *, void *);
int output_s_l_long(void);
extern "C" void _invalid_parameter_noinfo(void);

int shape_x_deref_merged(char *buffer, size_t_ sizeInBytes, const char *format, void *locale, void *argptr) {
    int result; int *err_ptr; int err_val;
    if (format == 0 || buffer == 0 || sizeInBytes == 0) {
        err_ptr = _errno(); err_val = 22;
    } else {
        result = vsnprintf_helper_long(output_s_l_long, buffer, sizeInBytes, format, locale, argptr);
        if (result < 0) { *buffer = 0; }
        if (result != -2) { return result; }
        err_ptr = _errno(); err_val = 34;
    }
    *err_ptr = err_val;
    _invalid_parameter_noinfo();
    return -1;
}
