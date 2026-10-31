/*
 * TEST-ONLY read-only observer of the ORIGINAL session environment's initialisation, shared-entropy order, lightning/sound effects and frame time phase (EEDCS-51). Compiled only with
 * the `env-observer` feature; never part of the shipping module. SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED.
 *
 * The hooks are the ENV_FX macros of aphavoc/source/entity/special/session/ss_envfx.h, placed in the original routines. They only READ: the shared seed word (never a draw), the
 * entity heap's next free indices, the headless clock, floats the original already computed (as IEEE bit patterns). Nothing here draws a random number, reseeds, allocates, calls
 * the engine or writes a game variable. Until it is armed (by eech_engine_env_observer_arm, before boot) every hook returns at once. Storage is a fixed array: an event that does
 * not fit is counted in `dropped`, `overflow` is set, and the report says so; the driver refuses incomplete evidence.
 *
 * Output (`key=value` lines, integers and words only):
 *   env-obs-report  the state of the observer: armed, how many events were stored / read / dropped, and the counters of the per-frame routines
 *   env-obs         one event: its sequence number, kind, the two integers, the shared seed word at that point (`rng`, signed, and `rng_w`) and six words
 */

#ifdef EECH_ENV_OBSERVER

#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "entity/special/session/ss_envfx.h"

#include "eech_engine.h"
#include "eech_headless.h"

#define ENVOBS_CAPACITY 16384

struct envobs_event
{
	uint32_t seq;
	int kind, a, b, rng;
	uint32_t w[6];
};

static struct envobs_event envobs_events[ENVOBS_CAPACITY];
static int envobs_armed;
static uint32_t envobs_stored, envobs_read_cursor, envobs_dropped, envobs_next_seq;
static uint64_t envobs_counters[ENV_FX_C_COUNT];
static uint32_t envobs_sound_calls, envobs_sound_records;
static uint32_t envobs_last_scratch_hash, envobs_last_camera[3];
static int envobs_sound_seen;

/* THE LIGHTNING QUEUE PROBE (ss_envfx.h ENV_FXQ): a separate finite store, never part of the event stream above. The queue is read through the accessor that modules/3d/3dstrike.c exports in this build. */
extern int eech_lightning_queue_view (int slot, unsigned int *words);
extern unsigned int eech_lightning_update_calls;

#define ENVQ_CAPACITY 96

struct envq_record
{
	uint32_t seq, point, lightning, active, main_present, time_present, time_bits, mask, occupancy;
	int slot;
	uint32_t w[8];
};

static struct envq_record envq_records[ENVQ_CAPACITY];
static uint32_t envq_stored, envq_dropped, envq_next_seq, envq_lightning, envq_before_mask;

void eech_envobs_arm (int on)
{
	if (on)
	{
		memset (envobs_events, 0, sizeof (envobs_events));
		memset (envobs_counters, 0, sizeof (envobs_counters));
		envobs_stored = envobs_read_cursor = envobs_dropped = envobs_next_seq = 0;
		envobs_sound_calls = envobs_sound_records = 0;
		envobs_sound_seen = 0;
		memset (envq_records, 0, sizeof (envq_records));
		envq_stored = envq_dropped = envq_next_seq = envq_lightning = envq_before_mask = 0;
		eech_lightning_update_calls = 0;
	}

	envobs_armed = on ? 1 : 0;
}

unsigned int eech_envobs_bits (float value)
{
	uint32_t bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

void eech_envobs_event (int kind, int a, int b, unsigned int w0, unsigned int w1, unsigned int w2, unsigned int w3, unsigned int w4, unsigned int w5, int rng)
{
	struct envobs_event *e;

	if (!envobs_armed)
	{
		return;
	}

	if (envobs_stored >= ENVOBS_CAPACITY)
	{
		envobs_dropped++;

		return;
	}

	e = &envobs_events[envobs_stored++];
	e->seq = envobs_next_seq++;
	e->kind = kind;
	e->a = a;
	e->b = b;
	e->rng = rng;
	e->w[0] = w0;
	e->w[1] = w1;
	e->w[2] = w2;
	e->w[3] = w3;
	e->w[4] = w4;
	e->w[5] = w5;
}

void eech_envobs_count (int counter)
{
	if (envobs_armed && counter >= 0 && counter < ENV_FX_C_COUNT)
	{
		envobs_counters[counter]++;
	}
}

/* the per-frame sound routine: counted every call, and recorded when its inputs change (the terrain scratch it fills, or the camera it reads), so a long run is not a long log. The scratch is
   read, never refilled: terrain_types_in_sector is the array the routine itself has just written. */
void eech_envobs_sound_facts (int in_cockpit, float x, float y, float z, float alt, float light_wind, float heavy_wind, float sector_x, float sector_z, int rng)
{
	uint32_t hash = 2166136261u, camera[3];
	int i, nonzero = 0;

	if (!envobs_armed)
	{
		return;
	}

	envobs_sound_calls++;

	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		hash ^= (uint32_t) terrain_types_in_sector[i];
		hash *= 16777619u;

		if (terrain_types_in_sector[i])
		{
			nonzero++;
		}
	}

	camera[0] = eech_envobs_bits (x);
	camera[1] = eech_envobs_bits (y);
	camera[2] = eech_envobs_bits (z);

	if (envobs_sound_seen && hash == envobs_last_scratch_hash && camera[0] == envobs_last_camera[0] && camera[1] == envobs_last_camera[1] && camera[2] == envobs_last_camera[2])
	{
		return;
	}

	envobs_sound_seen = 1;
	envobs_last_scratch_hash = hash;
	memcpy (envobs_last_camera, camera, sizeof (camera));
	envobs_sound_records++;

	/* a: in_cockpit, b: the scratch hash (as a signed int), w0..w2: the camera, w3: its radar altitude, w4/w5: the light and heavy wind sound levels; the clamped sector x/z and the count of
	   non-zero scratch entries follow in the next event of the same kind */
	eech_envobs_event (ENV_FX_SOUND_FACTS, in_cockpit, (int) hash, camera[0], camera[1], camera[2], eech_envobs_bits (alt), eech_envobs_bits (light_wind), eech_envobs_bits (heavy_wind), rng);
	eech_envobs_event (ENV_FX_SOUND_FACTS, -1, nonzero, eech_envobs_bits (sector_x), eech_envobs_bits (sector_z), 0, 0, 0, 0, rng);
}

/* random.c: every reseed of the shared generator (the libc generator is reseeded with the same argument by the original in the same call). The hook runs BEFORE the original assigns the word,
   so the event's rng and w0 are the word as it stands (`before`) and `a` is the target; the next event's rng shows the assigned word. */
void eech_envobs_seed_set (int seed, int before)
{
	eech_envobs_event (ENV_FX_SEED_SET, seed, 0, (uint32_t) before, 0, 0, 0, 0, 0, before);
}

void eech_envobs_seed_source (int source)
{
	eech_envobs_event (ENV_FX_SEED_SOURCE, source, 0, 0, 0, 0, 0, 0, 0, random_number_seed);
}

static int append (char *text, int capacity, int *used, const char *format, ...) __attribute__ ((format (printf, 4, 5)));

static int append (char *text, int capacity, int *used, const char *format, ...)
{
	va_list args;
	int n;

	va_start (args, format);
	n = vsnprintf (text + *used, (size_t) (capacity - *used), format, args);
	va_end (args);

	if (n < 0 || n >= capacity - *used)
	{
		return -1;
	}

	*used += n;

	return 0;
}

int eech_envobs_report (char *text, int capacity)
{
	int used = 0;

	text[0] = '\0';

	if (append (text, capacity, &used, "env-obs-report armed=%d capacity=%d stored=%u read=%u dropped=%u overflow=%d next_seq=%u update_server=%llu lightning_calls=%llu sound_calls=%llu"
		" resync_calls=%llu weather_calls=%llu sound_records=%u\n", envobs_armed, ENVOBS_CAPACITY, envobs_stored, envobs_read_cursor, envobs_dropped, envobs_dropped ? 1 : 0, envobs_next_seq,
		(unsigned long long) envobs_counters[ENV_FX_C_UPDATE_SERVER], (unsigned long long) envobs_counters[ENV_FX_C_LIGHTNING_CALLS], (unsigned long long) envobs_counters[ENV_FX_C_SOUND_CALLS],
		(unsigned long long) envobs_counters[ENV_FX_C_RESYNC_CALLS], (unsigned long long) envobs_counters[ENV_FX_C_WEATHER_CALLS], envobs_sound_records) != 0)
	{
		return -1;
	}

	return 0;
}

/* the events not yet read, as many as fit; `more` is set when events remain. A capacity too small for one event is refused. */
int eech_envobs_events (char *text, int capacity, int *more)
{
	int used = 0;

	text[0] = '\0';
	*more = 0;

	while (envobs_read_cursor < envobs_stored)
	{
		const struct envobs_event *e = &envobs_events[envobs_read_cursor];
		int before = used;

		if (append (text, capacity, &used, "env-obs seq=%u kind=%d a=%d b=%d rng=%d rng_w=%08x w0=%08x w1=%08x w2=%08x w3=%08x w4=%08x w5=%08x\n", e->seq, e->kind, e->a, e->b, e->rng, (unsigned) e->rng,
			e->w[0], e->w[1], e->w[2], e->w[3], e->w[4], e->w[5]) != 0)
		{
			text[before] = '\0';
			used = before;

			if (before == 0)
			{
				return -1;
			}

			*more = 1;

			return 0;
		}

		envobs_read_cursor++;
	}

	return 0;
}


/* ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
   THE LIGHTNING QUEUE PROBE. Each ENV_FXQ point of the original lightning routine (hosted create_lightning_effects and legacy update_lightning_effect alike) stores one record: the active and the main 3D environment (absent, the main one, another), the active one's time of
   day as a bit pattern, the strike queue's occupancy mask and, after an add, the slot that became occupied with its words. Everything is READ; the queue is read through the accessor. Records beyond
   the capacity are counted, never stored, and the report says so. */

/* the occupancy mask of the queue (bit n: slot n in use) and its count; returns the number of slots of the queue, or -1 when the accessor does not answer */
static int envq_mask (uint32_t *mask, uint32_t *occupancy)
{
	unsigned int words[8];
	int slot, slots = eech_lightning_queue_view (0, words);

	*mask = 0;
	*occupancy = 0;

	if (slots < 1 || slots > 32)
	{
		return -1;
	}

	for (slot = 0; slot < slots; slot++)
	{
		eech_lightning_queue_view (slot, words);

		if (words[0])
		{
			*mask |= ((uint32_t) 1) << slot;
			(*occupancy)++;
		}
	}

	return slots;
}

void eech_envobs_queue (int point, const void *main_env)
{
	const env_3d *main_3d = (const env_3d *) main_env;
	const env_3d *active = active_3d_environment;
	struct envq_record *r;
	uint32_t mask, occupancy, changed;
	unsigned int words[8];
	int slot = -1, i;

	if (!envobs_armed || point < 0 || point >= ENV_FXQ_POINT_COUNT)
	{
		return;
	}

	if (point == ENV_FXQ_ENTRY)
	{
		envq_lightning++;
	}

	envq_mask (&mask, &occupancy);

	if (point == ENV_FXQ_BEFORE_1 || point == ENV_FXQ_BEFORE_2)
	{
		envq_before_mask = mask;
	}

	memset (words, 0, sizeof (words));

	if (point == ENV_FXQ_AFTER_1 || point == ENV_FXQ_AFTER_2)
	{
		changed = mask & ~envq_before_mask;

		for (i = 0; i < 32; i++)
		{
			if (changed & (((uint32_t) 1) << i))
			{
				slot = i;

				break;
			}
		}

		if (slot >= 0)
		{
			eech_lightning_queue_view (slot, words);
		}
	}

	if (envq_stored >= ENVQ_CAPACITY)
	{
		envq_dropped++;

		return;
	}

	r = &envq_records[envq_stored];
	r->seq = envq_next_seq++;
	envq_stored++;
	r->point = (uint32_t) point;
	r->lightning = envq_lightning;
	r->active = (active == NULL) ? 0 : ((active == main_3d) ? 1 : 2);
	r->main_present = (main_3d != NULL) ? 1 : 0;
	r->time_present = (active != NULL) ? 1 : 0;
	r->time_bits = (active != NULL) ? eech_envobs_bits (active->time_of_day) : 0;
	r->mask = mask;
	r->occupancy = occupancy;
	r->slot = slot;
	memcpy (r->w, words, sizeof (r->w));
}

/* appended at *used in the caller's buffer (the fixture's group 13 case 0): the probe's state, its records in the order they were stored, and the queue as it stands now. A report that does not fit
   is refused (-1), never truncated. */
int eech_envobs_queue_report (char *text, int capacity, int *used)
{
	unsigned int words[8];
	uint32_t mask, occupancy, n;
	int slot, slots = envq_mask (&mask, &occupancy);

	if (append (text, capacity, used, "env-lq-report armed=%d capacity=%d stored=%u dropped=%u overflow=%d lightning_events=%u update_calls=%u queue_capacity=%d final_mask_w=%08x final_occupancy=%u\n",
		envobs_armed, ENVQ_CAPACITY, envq_stored, envq_dropped, envq_dropped ? 1 : 0, envq_lightning, eech_lightning_update_calls, slots, mask, occupancy) != 0)
	{
		return -1;
	}

	for (n = 0; n < envq_stored; n++)
	{
		const struct envq_record *r = &envq_records[n];

		if (append (text, capacity, used, "env-lq seq=%u point=%u n=%u active=%u main=%u time_present=%u time_w=%08x mask_w=%08x occupancy=%u slot=%d w0=%08x w1=%08x w2=%08x w3=%08x w4=%08x w5=%08x w6=%08x w7=%08x\n",
			r->seq, r->point, r->lightning, r->active, r->main_present, r->time_present, r->time_bits, r->mask, r->occupancy, r->slot, r->w[0], r->w[1], r->w[2], r->w[3], r->w[4], r->w[5], r->w[6], r->w[7]) != 0)
		{
			return -1;
		}
	}

	for (slot = 0; slot < slots; slot++)
	{
		eech_lightning_queue_view (slot, words);

		if (words[0] && append (text, capacity, used, "env-lq-slot slot=%d w0=%08x w1=%08x w2=%08x w3=%08x w4=%08x w5=%08x w6=%08x w7=%08x\n", slot, words[0], words[1], words[2], words[3], words[4], words[5], words[6], words[7]) != 0)
		{
			return -1;
		}
	}

	return 0;
}

#endif
