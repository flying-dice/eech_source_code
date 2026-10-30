/* Toolchain-only reproduction of the unchanged production entry's fenv sequence.
 * No EECH code, data, Lua, model callback or campaign. Prints observations.
 */
#include <fenv.h>
#include <stdint.h>
#include <stdio.h>

static uint32_t mxcsr(void) {
    uint32_t word;
    __asm__ __volatile__("stmxcsr %0" : "=m"(word));
    return word;
}
static void put_mxcsr(uint32_t word) {
    __asm__ __volatile__("ldmxcsr %0" : : "m"(word));
}
static unsigned short x87_control(void) {
    unsigned short word;
    __asm__ __volatile__("fnstcw %0" : "=m"(word));
    return word;
}
int main(void) {
    const int modes[] = {FE_TONEAREST, FE_DOWNWARD, FE_UPWARD, FE_TOWARDZERO};
    fenv_t original;
    uint32_t original_mxcsr = mxcsr();
    if (fegetenv(&original)) return 2;
    for (int i = 0; i < 4; ++i) {
        fenv_t caller;
        if (fesetround(modes[i])) return 3;
        put_mxcsr(mxcsr() | 0x0034u);
        uint32_t before_mxcsr = mxcsr();
        unsigned short before_x87_control = x87_control();
        /* Exactly the entry/leave sequence under investigation. */
        if (fegetenv(&caller) || fesetround(FE_TOWARDZERO) || fesetenv(&caller)) return 4;
        uint32_t after_mxcsr = mxcsr();
        unsigned short after_x87_control = x87_control();
        printf("rounding=%d before_x87_control=0x%04x after_x87_control=0x%04x before_mxcsr=0x%08x after_mxcsr=0x%08x same_mxcsr=%d\n",
               i, before_x87_control, after_x87_control, before_mxcsr, after_mxcsr, before_mxcsr == after_mxcsr);
        if (fesetenv(&original)) return 5;
        put_mxcsr(original_mxcsr);
    }
    return 0;
}
