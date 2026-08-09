// GRID-S cell s_uchar — an unsigned char.
// SIX formals (the only arity the shipped class admits), C++ linkage, long
// names. The ONLY axis is the store target's pointee type.
typedef int (*outfn_pointer_t)(void);
extern "C" int *_errno(void);
int helper_function_long(outfn_pointer_t, unsigned char *, unsigned , unsigned , void *, void *, void *);
int outfn_function_long(void);
extern "C" void _invalid_parameter_noinfo(void);

int store_type_cell_uchar(unsigned char *param_number_0, unsigned param_number_1, unsigned param_number_2, void *param_number_3, void *param_number_4, void *param_number_5) {
    int result;
    if (param_number_2 == 0 || param_number_0 == 0 || param_number_1 == 0) {
        *_errno() = 0x16;
        _invalid_parameter_noinfo();
        return -1;
    }
    result = helper_function_long(outfn_function_long, param_number_0, param_number_1, param_number_2, param_number_3, param_number_4, param_number_5);
    if (result < 0) { *param_number_0 = 0; }
    if (result != -2) { return result; }
    *_errno() = 0x22;
    _invalid_parameter_noinfo();
    return -1;
}
