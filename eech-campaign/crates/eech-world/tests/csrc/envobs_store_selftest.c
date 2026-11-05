/*
 * SELF-TEST of the sound initialisation probe's bookkeeping (csrc/eech_envobs_store.h), TEST-ONLY, run by tests/env_sound_init_store.rs of eech-world with the host C compiler. It is a small program, not a campaign and
 * not a fixture case: it builds a 4-key store with a CANARY one past the end of every array, and drives it, with near-limit states PRESET instead of read four billion times, through the paths the independent review of
 * aa1bf9364 found unsafe (ENV-SOUND-INIT-CAPACITY-1): a use count at its maximum must stop and latch, never wrap back to "unseen"; the first-seen order must never be written past its end; and the normal, finite
 * counts, first-seen order and digest must be exactly what an independent computation says.
 */
#include <stdio.h>

#include "eech_envobs_store.h"

#define KEYS 4
#define CANARY 0x5a5a5a5a

static int checks, failures;

#define CHECK(condition) \
	do \
	{ \
		checks++; \
		if (!(condition)) \
		{ \
			failures++; \
			printf ("FAILED line %d: %s\n", __LINE__, #condition); \
		} \
	} while (0)

struct fixture
{
	uint32_t uses[KEYS + 1];
	uint8_t seen[KEYS + 1];
	int rate[KEYS + 1], size[KEYS + 1], orate[KEYS + 1], order[KEYS + 1];
	uint32_t saturated;
	struct enva_sample_store store;
};

static void start (struct fixture *f)
{
	int i;

	for (i = 0; i <= KEYS; i++)
	{
		f->uses[i] = 0;
		f->seen[i] = 0;
		f->rate[i] = f->size[i] = f->orate[i] = f->order[i] = CANARY;
	}

	f->uses[KEYS] = CANARY;
	f->seen[KEYS] = 0x5a;
	f->saturated = 0;
	f->store.keys = KEYS;
	f->store.uses = f->uses;
	f->store.seen = f->seen;
	f->store.rate = f->rate;
	f->store.size = f->size;
	f->store.orate = f->orate;
	f->store.order = f->order;
	f->store.saturated = &f->saturated;
	f->store.reads = f->store.distinct = f->store.out_of_domain = 0;
	f->store.digest = ENVA_FNV_OFFSET;
}

static int canaries_intact (const struct fixture *f)
{
	return f->uses[KEYS] == CANARY && f->seen[KEYS] == 0x5a && f->rate[KEYS] == CANARY && f->size[KEYS] == CANARY && f->orate[KEYS] == CANARY && f->order[KEYS] == CANARY;
}

/* an independent FNV-1a 64 over the four words of each read, written out in full (not the header's function) */
static uint64_t independent_digest (const int (*reads)[4], int count)
{
	uint64_t h = 14695981039346656037ULL;
	int n, w, b;

	for (n = 0; n < count; n++)
	{
		for (w = 0; w < 4; w++)
		{
			uint32_t word = (uint32_t) reads[n][w];

			for (b = 0; b < 4; b++)
			{
				h ^= (uint64_t) ((word >> (8 * b)) & 255u);
				h *= 1099511628211ULL;
			}
		}
	}

	return h;
}

int main (void)
{
	struct fixture f;
	int n;

	/* 1. the normal, finite case: counts, first-seen order, first descriptor numbers and the digest are exact */
	{
		static const int reads[6][4] = { { 2, 100, 10, 1000 }, { 0, 101, 11, 1001 }, { 2, 200, 20, 2000 }, { 3, 103, 13, 1003 }, { 2, 300, 30, 3000 }, { 1, 104, 14, 1004 } };

		start (&f);

		for (n = 0; n < 6; n++)
		{
			enva_sample_read (&f.store, reads[n][0], reads[n][1], reads[n][2], reads[n][3]);
		}

		CHECK (f.store.reads == 6 && f.store.distinct == 4 && f.store.out_of_domain == 0 && f.saturated == 0);
		CHECK (f.order[0] == 2 && f.order[1] == 0 && f.order[2] == 3 && f.order[3] == 1);
		CHECK (f.uses[0] == 1 && f.uses[1] == 1 && f.uses[2] == 3 && f.uses[3] == 1);
		CHECK (f.rate[2] == 100 && f.size[2] == 10 && f.orate[2] == 1000);
		CHECK (f.store.digest == independent_digest (reads, 6));
		CHECK (canaries_intact (&f));
	}

	/* 2. a key outside the catalogue is counted and in the digest, and stores nothing */
	{
		static const int reads[4][4] = { { -1, 1, 2, 3 }, { KEYS, 4, 5, 6 }, { 1000000, 7, 8, 9 }, { 1, 10, 11, 12 } };

		start (&f);

		for (n = 0; n < 4; n++)
		{
			enva_sample_read (&f.store, reads[n][0], reads[n][1], reads[n][2], reads[n][3]);
		}

		CHECK (f.store.reads == 4 && f.store.out_of_domain == 3 && f.store.distinct == 1 && f.order[0] == 1 && f.uses[1] == 1 && f.saturated == 0);
		CHECK (f.store.digest == independent_digest (reads, 4));
		CHECK (canaries_intact (&f));
	}

	/* 3. THE WRAP: the whole catalogue is seen, and key 2 has been read 2^32 - 2 times (preset). The two next reads reach and pass the maximum: the count STOPS at the maximum and the latch is raised; it never returns to
	      the "unseen" value, so the key is never appended to the first-seen order again and nothing is written past the end of any array. A third read does the same. */
	{
		start (&f);

		for (n = 0; n < KEYS; n++)
		{
			enva_sample_read (&f.store, n, 1000 + n, 2000 + n, 3000 + n);
		}

		f.uses[2] = UINT32_MAX - 1;

		enva_sample_read (&f.store, 2, 1, 1, 1);
		CHECK (f.uses[2] == UINT32_MAX && f.saturated == 0);

		enva_sample_read (&f.store, 2, 1, 1, 1);
		CHECK (f.uses[2] == UINT32_MAX && f.saturated == 1);

		enva_sample_read (&f.store, 2, 1, 1, 1);
		CHECK (f.uses[2] == UINT32_MAX && f.saturated == 2);

		CHECK (f.store.distinct == KEYS && f.order[0] == 0 && f.order[1] == 1 && f.order[2] == 2 && f.order[3] == 3);
		CHECK (f.rate[2] == 1002 && f.size[2] == 2002 && f.orate[2] == 3002);
		CHECK (f.store.reads == 7 && f.store.out_of_domain == 0);
		CHECK (canaries_intact (&f));
	}

	/* 4. the total read counter has the same rule */
	{
		start (&f);
		enva_sample_read (&f.store, 1, 1, 1, 1);
		f.store.reads = UINT32_MAX;
		enva_sample_read (&f.store, 1, 1, 1, 1);
		CHECK (f.store.reads == UINT32_MAX && f.saturated == 1 && f.uses[1] == 2 && f.store.distinct == 1);
		CHECK (canaries_intact (&f));
	}

	/* 5. the order has its own bound: if the distinct count is already at the catalogue size and a key is nevertheless unseen (a corrupted store), nothing is written and the latch is raised */
	{
		start (&f);

		for (n = 0; n < KEYS; n++)
		{
			enva_sample_read (&f.store, n, 1, 2, 3);
		}

		f.seen[1] = 0;
		f.uses[1] = 0;
		enva_sample_read (&f.store, 1, 9, 9, 9);
		CHECK (f.store.distinct == KEYS && f.saturated == 1 && f.seen[1] == 0 && f.uses[1] == 0 && f.order[0] == 0 && f.order[1] == 1 && f.order[2] == 2 && f.order[3] == 3);
		CHECK (f.rate[1] == 1 && f.size[1] == 2 && f.orate[1] == 3);
		CHECK (canaries_intact (&f));
	}

	/* 6. the latch itself stops at its maximum, and a plain counter counts to its maximum exactly */
	{
		uint32_t counter = UINT32_MAX - 2, latch = UINT32_MAX;

		enva_bump (&counter, &latch);
		CHECK (counter == UINT32_MAX - 1 && latch == UINT32_MAX);
		enva_bump (&counter, &latch);
		CHECK (counter == UINT32_MAX && latch == UINT32_MAX);
		enva_bump (&counter, &latch);
		CHECK (counter == UINT32_MAX && latch == UINT32_MAX);
	}

	if (failures)
	{
		printf ("ENVOBS_STORE_SELFTEST FAILED checks=%d failures=%d\n", checks, failures);

		return 1;
	}

	printf ("ENVOBS_STORE_SELFTEST OK checks=%d\n", checks);

	return 0;
}
