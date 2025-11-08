/**
 * Aurora C Runtime Stub
 *
 * Provides minimal C runtime support for Aurora programs,
 * including println implementation and program startup.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * Print string to stdout with newline
 * Called from Aurora code
 */
void aurora_println(const char* str) {
    if (str) {
        printf("%s\n", str);
    }
}

/**
 * Print string to stdout without newline
 * Called from Aurora code
 */
void aurora_print(const char* str) {
    if (str) {
        printf("%s", str);
    }
}

/**
 * Allocate memory
 * Simple wrapper around malloc for Aurora
 */
void* aurora_alloc(size_t size) {
    return malloc(size);
}

/**
 * Free memory
 * Simple wrapper around free for Aurora
 */
void aurora_free(void* ptr) {
    free(ptr);
}

/**
 * Reallocate memory
 */
void* aurora_realloc(void* ptr, size_t size) {
    return realloc(ptr, size);
}

/**
 * Aurora panic handler
 * Called when Aurora code panics
 */
void aurora_panic(const char* msg, const char* file, int line) {
    fprintf(stderr, "Aurora panic at %s:%d: %s\n", file, line, msg);
    abort();
}

/**
 * Aurora main function signature
 * The Aurora compiler generates a main() function
 * We don't redefine main here - let Aurora's main be the real main
 */
