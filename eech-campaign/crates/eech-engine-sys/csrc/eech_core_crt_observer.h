/* TEST-ONLY core-terrain-crt-observer. Include libc declarations BEFORE macros;
 * wrappers forward the original call exactly once. No Host executable CRT. */
#ifndef EECH_CORE_CRT_OBSERVER_H
#define EECH_CORE_CRT_OBSERVER_H
/* eech_win32.c requests GNU cookie I/O before its own system includes. A force
 * include must preserve that feature namespace before libc's feature header. */
#if defined(__linux__) && !defined(_GNU_SOURCE)
#define _GNU_SOURCE
#endif
#include <stdlib.h>
int eech_core_crt_rand (const char *file, int line);
void eech_core_crt_srand (unsigned int seed, const char *file, int line);
void eech_core_crt_load_begin (int clouds, int season);
void eech_core_crt_load_end (int map, int width, int height);
void eech_core_crt_boundary (const char *name);
void eech_core_crt_ctor (int kind, int scene);
void eech_core_crt_skin_begin (int scene, int animation, int season, int normal, int desert, int winter);
void eech_core_crt_skin_frames (int frames);
void eech_core_crt_skin_end (void);
int eech_core_crt_report (char *text, int capacity);
#define rand() eech_core_crt_rand (__FILE__, __LINE__)
#define srand(SEED) eech_core_crt_srand ((SEED), __FILE__, __LINE__)
#endif
