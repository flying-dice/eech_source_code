/*
 * TEST-ONLY (EEDCS-52, D3; feature `target-point-fixture`): force-included into every translation unit of that build (build/main.rs) and into no other. It renames every rand () of the
 * build to eech_tp_counted_rand (csrc/eech_tp_fixture.c), which counts the draw and then calls the real rand () once, in the same place with the same result: the CRT stream, its
 * order and its values are the original's. It does not seed, reseed or reorder anything, and it does not touch the campaign's own generator (random.c random_number_seed, rand16 and
 * friends), which is a different generator.
 */
#ifndef EECH_TP_RAND_H
#define EECH_TP_RAND_H

extern int eech_tp_counted_rand (void);

#define rand eech_tp_counted_rand

#endif
