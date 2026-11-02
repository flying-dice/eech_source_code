/*
 * The pure bookkeeping of the sound initialisation probe (EEDCS-51, D3; csrc/eech_env_observer.c), kept free of every game header so that a small C program can exercise it near its limits
 * (eech-world/tests/csrc/envobs_store_selftest.c, run by eech-world/tests/env_sound_init_store.rs: NOT in this crate's csrc, whose every top-level .c file the build compiles into the engine). TEST-ONLY: included only by the env-observer collector and the self-test. SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED.
 *
 * Two rules, from the independent source review of aa1bf9364 (ENV-SOUND-INIT-CAPACITY-1):
 *   - A COUNTER NEVER WRAPS. enva_bump stops a counter at its maximum and LATCHES the shared `saturated` count, which the report prints and the driver refuses: a capture whose counter reached its limit is incomplete,
 *     never plausible data. (A wrapped use count used to return to the "unseen" value and append a seen key to the first-seen order again, one past the end of the array once the catalogue was full.)
 *   - SEEN IS NOT A COUNT. Whether a sample key was read before is its own flag (`seen`), independent of how many times it was read, and the first-seen order is appended only under a bound check of its own: the order
 *     index must be below the number of keys, and if it ever is not, nothing is written and the latch is raised.
 * Nothing here allocates, draws, calls the game or uses a float. Storage is supplied by the caller and is bounded by the key count.
 */
#ifndef EECH_ENVOBS_STORE_H
#define EECH_ENVOBS_STORE_H

#include <stdint.h>

#define ENVA_FNV_OFFSET 0xcbf29ce484222325ULL

static uint64_t enva_fold (uint64_t digest, uint32_t word)
{
	int byte;

	for (byte = 0; byte < 4; byte++)
	{
		digest ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		digest *= 0x100000001b3ULL;
	}

	return digest;
}

/* counts one event; at the counter's maximum it does not count and raises the latch (which itself stops at its maximum) */
static void enva_bump (uint32_t *counter, uint32_t *saturated)
{
	if (*counter == UINT32_MAX)
	{
		if (*saturated != UINT32_MAX)
		{
			(*saturated)++;
		}

		return;
	}

	(*counter)++;
}

/* The constructor's descriptor reads, by sample key. `keys` is the size of the catalogue; every array has `keys` entries (the first-seen order too). */
struct enva_sample_store
{
	int keys;
	uint32_t *uses;
	uint8_t *seen;
	int *rate, *size, *orate, *order;
	uint32_t *saturated;
	uint32_t reads, distinct, out_of_domain;
	uint64_t digest;
};

/* one read of a descriptor: every read is in the digest, in order; a key outside the catalogue is counted (and refused by the driver); the first read of a key keeps its descriptor numbers and its place in the order */
static void enva_sample_read (struct enva_sample_store *s, int key, int rate, int size, int orate)
{
	enva_bump (&s->reads, s->saturated);
	s->digest = enva_fold (s->digest, (uint32_t) key);
	s->digest = enva_fold (s->digest, (uint32_t) rate);
	s->digest = enva_fold (s->digest, (uint32_t) size);
	s->digest = enva_fold (s->digest, (uint32_t) orate);

	if (key < 0 || key >= s->keys)
	{
		enva_bump (&s->out_of_domain, s->saturated);

		return;
	}

	if (!s->seen[key])
	{
		/* the order has room for exactly `keys` entries and a key is first seen once: a count that is not below `keys` here is a corrupted store, and nothing is written */
		if (s->distinct >= (uint32_t) s->keys)
		{
			if (*s->saturated != UINT32_MAX)
			{
				(*s->saturated)++;
			}

			return;
		}

		s->seen[key] = 1;
		s->rate[key] = rate;
		s->size[key] = size;
		s->orate[key] = orate;
		s->order[s->distinct] = key;
		s->distinct++;
	}

	enva_bump (&s->uses[key], s->saturated);
}

#endif
