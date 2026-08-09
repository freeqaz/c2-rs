// w6char — SIX formals with a BYTE store: the reader admits any non-word store, the emitter always writes sth. If the port accepts this, it is a live wrong emit.
// C++ linkage and long names throughout, so the 8-byte inline-name fence
// (GRID-T) cannot be what decides this cell.
typedef int (*outfn_pointer_t)(void);
extern "C" int *_errno(void);
int helper_function_long(outfn_pointer_t, char *, unsigned , unsigned , void *, void *, void *);
int outfn_function_long(void);
extern "C" void _invalid_parameter_noinfo(void);

int shape_under_test_w6char(char *param_number_0, unsigned param_number_1, unsigned param_number_2, void *param_number_3, void *param_number_4, void *param_number_5) {
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
