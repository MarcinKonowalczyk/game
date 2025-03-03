#include <stdio.h>
#include <stdarg.h>

#ifdef __cplusplus
extern "C" {
#endif

void _vprintf(const char *format, va_list args) {
    vprintf(format, args);
    fflush(stdout);
}

void _vsnprintf(char *str, size_t size, const char *format, va_list args) {
    vsnprintf(str, size, format, args);
}

#ifdef __cplusplus
} // extern "C"
#endif
