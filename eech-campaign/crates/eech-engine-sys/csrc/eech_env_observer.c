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
#include "eech_envobs_store.h"
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

/* the sound entity probe (below): its state is reset with the observer, and its open window and frame count follow the observer's own events */
static void enve_reset (void);
static int enve_open;
static uint32_t enve_frames;

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
		enve_reset ();
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

	/* the sound entity probe follows the observer's own events: a frame began, a lightning sound creation opened (a is 1 or 2) or closed */
	if (kind == ENV_FX_FRAME_BEGIN)
	{
		enve_frames++;
	}
	else if (kind == ENV_FX_SOUND_BEGIN)
	{
		enve_open = a;
	}
	else if (kind == ENV_FX_SOUND_END)
	{
		enve_open = 0;
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
	/* the stored word, copied as bits straight from the original object: the value is never passed through a floating-point parameter or load. Absent when no environment is active. */
	r->time_bits = 0;

	if (active != NULL)
	{
		memcpy (&r->time_bits, &active->time_of_day, sizeof (r->time_bits));
	}

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


/* ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
   THE SOUND ENTITY PROBE (ss_envfx.h ENV_FXE). A separate finite store of what the ORIGINAL sound effect entities of the lightning routine look like when the real constructor has built them and
   how the real update routines (se_updt.c, the server's and the client's) treat them. Everything is READ: stored fields are copied as bits (never loaded by value as floats), the playback handle is
   a present/absent flag, lists are read as present/absent and whether the parent is the session. A creation is recorded only while a lightning sound creation is open (the observer's own
   ENV_FX_SOUND_BEGIN ... ENV_FX_SOUND_END) or while a fixture control has switched tracking on, and the entity is then followed through its updates (at most ENVE_TRACKED entities; a creation beyond them is recorded and COUNTED as untracked, never followed). Records beyond the
   capacity are counted, never stored, and the report says so. */

extern int sound_system_initialised;
extern int sound_system_paused;

#define ENVE_CAPACITY 80
#define ENVE_TRACKED 8
#define ENVE_ENTERS_STORED 3	/* per entity: the first updates are stored whole, the rest are counted (upd on the later records says how many ran) */

struct enve_record
{
	uint32_t seq, point, n, frame, upd;
	int tracked, entity, etype, sub, channel, locality, valid, looping, panning, seqcount, handle, session_parent, update_member, sample, rate, size, orate;
	uint32_t life, vlife, amp, pitch, px, py, pz, delta;
};

static struct enve_record enve_records[ENVE_CAPACITY];
static const void *enve_tracked[ENVE_TRACKED];
static int enve_tracked_n[ENVE_TRACKED], enve_tracked_dead[ENVE_TRACKED];
static uint32_t enve_tracked_updates[ENVE_TRACKED];
static int enve_tracked_count, enve_open, enve_track_all;
static uint32_t enve_stored, enve_dropped, enve_next_seq, enve_frames, enve_creations, enve_updates, enve_kills;
static uint32_t enve_untracked;	/* creations seen after the tracking table was full: recorded as creations, never followed */

static void enve_reset (void)
{
	memset (enve_records, 0, sizeof (enve_records));
	memset (enve_tracked, 0, sizeof (enve_tracked));
	memset (enve_tracked_n, 0, sizeof (enve_tracked_n));
	memset (enve_tracked_dead, 0, sizeof (enve_tracked_dead));
	memset (enve_tracked_updates, 0, sizeof (enve_tracked_updates));
	enve_tracked_count = enve_open = enve_track_all = 0;
	enve_stored = enve_dropped = enve_next_seq = enve_frames = enve_creations = enve_updates = enve_kills = 0;
	enve_untracked = 0;
}

/* a fixture control switches tracking on: the entities it creates are then followed like the lightning routine's */
void eech_envobs_sound_track (int on)
{
	enve_track_all = on ? 1 : 0;
}

static uint32_t enve_bits (const float *address)
{
	uint32_t bits;

	memcpy (&bits, address, sizeof (bits));

	return bits;
}

void eech_envobs_sound (int point, const void *p)
{
	entity *en = (entity *) p;
	sound_effect *raw;
	struct enve_record *r;
	int tracked = -1, i;

	if (!envobs_armed || point < 0 || point >= ENV_FXE_POINT_COUNT)
	{
		return;
	}

	if (point == ENV_FXE_CREATED)
	{
		if (!enve_open && !enve_track_all)
		{
			return;
		}

		enve_creations++;

		if (enve_tracked_count < ENVE_TRACKED)
		{
			tracked = enve_tracked_count++;
			enve_tracked[tracked] = p;
			enve_tracked_n[tracked] = enve_open;
		}
		else
		{
			enve_untracked++;
		}
	}
	else
	{
		for (i = 0; i < enve_tracked_count; i++)
		{
			if (enve_tracked[i] == p && !enve_tracked_dead[i])
			{
				tracked = i;
			}
		}

		if (tracked < 0)
		{
			return;
		}

		if (point == ENV_FXE_SERVER_ENTER || point == ENV_FXE_CLIENT_ENTER)
		{
			enve_updates++;
			enve_tracked_updates[tracked]++;
		}

		if ((point == ENV_FXE_SERVER_ENTER || point == ENV_FXE_CLIENT_ENTER || point == ENV_FXE_CLIENT_EXIT) && enve_tracked_updates[tracked] > ENVE_ENTERS_STORED)
		{
			return;
		}

		if (point == ENV_FXE_SERVER_KILLED)
		{
			enve_kills++;
			enve_tracked_dead[tracked] = 1;
		}
	}

	if (enve_stored >= ENVE_CAPACITY)
	{
		enve_dropped++;

		return;
	}

	r = &enve_records[enve_stored];
	memset (r, 0, sizeof (*r));
	r->seq = enve_next_seq++;
	enve_stored++;
	r->point = (uint32_t) point;
	r->n = (uint32_t) ((tracked >= 0) ? enve_tracked_n[tracked] : enve_open);
	r->frame = enve_frames;
	r->upd = (tracked >= 0) ? enve_tracked_updates[tracked] : 0;
	r->tracked = tracked;
	r->entity = get_local_entity_index (en);
	r->etype = (int) get_local_entity_type (en);
	r->sample = -1;
	r->delta = enve_bits (&system_delta_time);

	/* after the kill only the entity slot is read: the sound data was freed with the entity */
	if (point == ENV_FXE_SERVER_KILLED)
	{
		return;
	}

	raw = (sound_effect *) get_local_entity_data (en);

	r->sub = (int) raw->eff.sub_type;
	r->channel = (int) raw->sound_channel;
	r->locality = (int) raw->sound_locality;
	r->valid = (int) raw->valid_sound_effect;
	r->looping = (int) raw->looping;
	r->panning = (int) raw->panning;
	r->seqcount = (int) raw->sound_effect_sequence_count;
	r->handle = (raw->sound_effect_data != NULL) ? 1 : 0;
	r->session_parent = (raw->eff.special_effect_link.parent != NULL && raw->eff.special_effect_link.parent == get_session_entity ()) ? 1 : 0;
	r->update_member = (raw->eff.update_link.parent != NULL) ? 1 : 0;
	r->life = enve_bits (&raw->effect_lifetime);
	r->vlife = enve_bits (&raw->valid_effect_lifetime);
	r->amp = enve_bits (&raw->amplification);
	r->pitch = enve_bits (&raw->pitch);
	r->px = enve_bits (&raw->eff.position.x);
	r->py = enve_bits (&raw->eff.position.y);
	r->pz = enve_bits (&raw->eff.position.z);

	if (raw->effect_index)
	{
		r->sample = (int) raw->effect_index[0];

		if (r->sample >= 0 && r->sample < NUM_SOUND_SAMPLE_INDICES)
		{
			r->rate = application_sound_effects[r->sample].rate;
			r->size = application_sound_effects[r->sample].size;
			r->orate = application_sound_effects[r->sample].original_rate;
		}
	}
}

/* appended at *used in the caller's buffer (the fixture's group 14): the probe's state and its records in the order they were stored. A report that does not fit is refused (-1), never truncated. */
int eech_envobs_sound_report (char *text, int capacity, int *used)
{
	uint32_t n;

	if (append (text, capacity, used, "env-snd-report armed=%d capacity=%d stored=%u dropped=%u overflow=%d creations=%u updates=%u kills=%u frames=%u tracked=%d open=%d track_all=%d untracked=%u\n", envobs_armed, ENVE_CAPACITY,
		enve_stored, enve_dropped, enve_dropped ? 1 : 0, enve_creations, enve_updates, enve_kills, enve_frames, enve_tracked_count, enve_open, enve_track_all, enve_untracked) != 0)
	{
		return -1;
	}

	for (n = 0; n < enve_stored; n++)
	{
		const struct enve_record *r = &enve_records[n];

		if (append (text, capacity, used, "env-snd seq=%u point=%u n=%u tracked=%d frame=%u upd=%u entity=%d etype=%d sub=%d channel=%d locality=%d valid=%d looping=%d panning=%d seqcount=%d handle=%d session_parent=%d update_member=%d"
			" sample=%d rate=%d size=%d orate=%d life_w=%08x vlife_w=%08x amp_w=%08x pitch_w=%08x px_w=%08x py_w=%08x pz_w=%08x delta_w=%08x\n", r->seq, r->point, r->n, r->tracked, r->frame, r->upd, r->entity, r->etype, r->sub,
			r->channel, r->locality, r->valid, r->looping, r->panning, r->seqcount, r->handle, r->session_parent, r->update_member, r->sample, r->rate, r->size, r->orate, r->life, r->vlife, r->amp, r->pitch, r->px,
			r->py, r->pz, r->delta) != 0)
		{
			return -1;
		}
	}

	return 0;
}

/*
 * THE SOUND INITIALISATION PROBE (ss_envfx.h ENV_FXA). A third separate finite store: fixed tables, integers only, filled by the hooks of the original initialisation of the application sound system (appsound/snd_init.c,
 * snd_data.c), by the descriptor reads of the sound effect constructor (soundeff.c, hosted block), and, in the headless platform layer, by the open attempts for the sound header and data files (eech_win32.c
 * eech_fopen) and the calls of the null DirectSound entry points (eech_com_null.c). It is NOT reset by arming: the initialisation happens during boot, before or without any arming, and the store counts from
 * process start. A point reads values the original already holds; nothing here calls the engine, allocates, draws, opens a file, formats at the point or writes a game variable. A record that does not fit is counted
 * in `dropped` and the report says so (the driver refuses incomplete evidence). Nothing in it is a pointer, an address, a path or a coordinate.
 */

extern int sound_system_initialised;
extern int number_of_blocked_sound_samples;

#define ENVA_BOOTS 4
#define ENVA_ATTEMPTS 8
#define ENVA_SIDES 8
#define ENVA_COM 4

struct enva_catalogue
{
	uint32_t named, nonzero_size, nonzero_rate, nonzero_orate;
	uint64_t digest;
};

struct enva_boot
{
	int no_sound, dedicated, enabled_before, initialised_before;
	int effects_begin, filled, effects_end;
	int open_true, open_false, header_count;
	int matched, missed;
	/* the HOST's validated sample metadata answer (ENV_FXA_HOST_ANSWER): a MIRROR of what the host said, not an observation of a file operation of the campaign; open_true/open_false/matched/missed above stay the campaign's own events */
	int host_answers, host_present, host_header_count, host_matched, host_missed;
	/* the HOST's validated backend availability answer (ENV_FXA_HOST_BACKEND): a MIRROR of what the host said; the campaign makes no DirectSound call in the hosted initialisation, so the platform counters below stay what they are (0) */
	int host_backend_answers, host_backend_available;
	int backend, enabled_backend, initialised_backend;
	int load, skipped, exited, enabled_exit;
	struct enva_catalogue filled_catalogue, end_catalogue;
};

struct enva_side
{
	int side, enabled, no_sound, dedicated;
};

struct enva_attempt
{
	int kind, ok, fallback, read;
};

/*
 * THE SAMPLE KEY DOMAIN. A sample key is an index into the descriptor catalogue, which has exactly NUM_SOUND_SAMPLE_INDICES entries (a compile-time constant of the original, 2069 in the measured native), so the set of keys the
 * constructor can read is FINITE and known: the store is a table indexed by the key, not a guess at how many distinct keys a run reads (a first version kept 32 and a natural 120 s boot overflowed it with 1210 reads). Per key: the
 * number of reads, and the rate, size and original rate of its FIRST read; plus the keys in the order they were first read. A key outside 0 .. NUM_SOUND_SAMPLE_INDICES - 1 cannot be stored: it is counted in `out_of_domain` and the
 * driver refuses the capture. All of this is bounded test-only static storage (about 41 KiB), filled by integer operations at the hook: no heap, no float, no draw, no extra read of the game.
 *
 * THE REPORT'S WORST CASE. The report of group 14 index 15 is written into a buffer of ENVA_REPORT_CAPACITY bytes (csrc/eech_env_fixture.c passes whatever the caller gave it; the Rust wrapper gives this size for this case only, and the
 * other cases keep their own). Its largest part is one sample row per kept key:
 *   "env-sndinit-sample n=%u index=%d rate=%d size=%d orate=%d uses=%u\n" is 54 literal characters plus n (at most 10 digits), index, rate, size and orate (at most 11 characters each, a sign and 10 digits) and uses (at most 10 digits)
 *   = 118 characters at the very most, taken as ENVA_SAMPLE_ROW_MAX 128; NUM_SOUND_SAMPLE_INDICES rows of that is 2069 * 128 = 264,832 bytes.
 * Every other row is bounded by a fixed table (4 initialisations, 8 side calls, 8 file attempts, 4 DirectSound entries) of rows of at most about 520 characters, under 8 KiB in all; ENVA_REPORT_FIXED_MAX takes 16 KiB.
 * The sum, 281,216, is covered by ENVA_REPORT_CAPACITY = 320 KiB = 327,680, which the compile-time check below enforces. `append` still refuses, never truncates, a report that does not fit.
 */
#define ENVA_SAMPLE_ROW_MAX 128
#define ENVA_REPORT_FIXED_MAX 16384
#define ENVA_REPORT_CAPACITY 327680

typedef char enva_report_capacity_covers_the_whole_catalogue_domain[((NUM_SOUND_SAMPLE_INDICES * ENVA_SAMPLE_ROW_MAX + ENVA_REPORT_FIXED_MAX) <= ENVA_REPORT_CAPACITY) ? 1 : -1];

static struct enva_boot enva_boots_table[ENVA_BOOTS];
static uint32_t enva_boots, enva_dropped, enva_orphans, enva_opens_outside;
static int enva_current = -1;
static int enva_in_effects;

static struct enva_side enva_sides[ENVA_SIDES];
static uint32_t enva_side_calls;

static uint32_t enva_file_attempts[2], enva_file_ok[2], enva_attempt_count;
static struct enva_attempt enva_attempt_table[ENVA_ATTEMPTS];

static uint32_t enva_com_calls[ENVA_COM], enva_com_last[ENVA_COM];

/* the latch of every counter of this store that reached its maximum (eech_envobs_store.h enva_bump): it is reported and the driver refuses a capture in which it is not 0 */
static uint32_t enva_saturated;

static uint32_t enva_ctor_calls, enva_ctor_looping, enva_ctor_max_count;
static uint32_t enva_sample_uses[NUM_SOUND_SAMPLE_INDICES];
static uint8_t enva_sample_seen[NUM_SOUND_SAMPLE_INDICES];
static int enva_sample_rate[NUM_SOUND_SAMPLE_INDICES], enva_sample_size[NUM_SOUND_SAMPLE_INDICES], enva_sample_orate[NUM_SOUND_SAMPLE_INDICES];
static int enva_sample_order[NUM_SOUND_SAMPLE_INDICES];
static struct enva_sample_store enva_samples = { NUM_SOUND_SAMPLE_INDICES, enva_sample_uses, enva_sample_seen, enva_sample_rate, enva_sample_size, enva_sample_orate, enva_sample_order, &enva_saturated, 0, 0, 0, ENVA_FNV_OFFSET };

/* the descriptor table, read with integer operations only: how many entries are named, have a size, a rate and an original rate, and an FNV-1a digest of (index, rate, size, original rate) of every entry in index order */
static void enva_take_catalogue (struct enva_catalogue *c)
{
	int i;

	memset (c, 0, sizeof (*c));

	c->digest = 0xcbf29ce484222325ULL;

	for (i = 0; i < NUM_SOUND_SAMPLE_INDICES; i++)
	{
		const sound_effect_information *item = &application_sound_effects[i];

		if (item->name != NULL)
		{
			c->named++;
		}

		if (item->size != 0)
		{
			c->nonzero_size++;
		}

		if (item->rate != 0)
		{
			c->nonzero_rate++;
		}

		if (item->original_rate != 0)
		{
			c->nonzero_orate++;
		}

		c->digest = enva_fold (c->digest, (uint32_t) i);
		c->digest = enva_fold (c->digest, (uint32_t) item->rate);
		c->digest = enva_fold (c->digest, (uint32_t) item->size);
		c->digest = enva_fold (c->digest, (uint32_t) item->original_rate);
	}
}

void eech_envobs_sndinit (int point, int a, int b, int c, int d)
{
	struct enva_boot *boot = (enva_current >= 0) ? &enva_boots_table[enva_current] : NULL;

	(void) c;

	switch (point)
	{
		case ENV_FXA_INIT_ENTER:
			if (enva_boots < ENVA_BOOTS)
			{
				enva_current = (int) enva_boots;
				boot = &enva_boots_table[enva_current];
				memset (boot, 0, sizeof (*boot));
				boot->no_sound = command_line_no_sound;
				boot->dedicated = command_line_comms_dedicated_server;
				boot->enabled_before = application_sound_system_enabled;
				boot->initialised_before = sound_system_initialised;
			}
			else
			{
				enva_current = -1;
				enva_bump (&enva_dropped, &enva_saturated);
			}

			enva_bump (&enva_boots, &enva_saturated);

			return;

		case ENV_FXA_LOAD_SIDE:
			if (enva_side_calls < ENVA_SIDES)
			{
				enva_sides[enva_side_calls].side = a;
				enva_sides[enva_side_calls].enabled = application_sound_system_enabled;
				enva_sides[enva_side_calls].no_sound = command_line_no_sound;
				enva_sides[enva_side_calls].dedicated = command_line_comms_dedicated_server;
			}

			enva_bump (&enva_side_calls, &enva_saturated);

			return;

		case ENV_FXA_CTOR:
			enva_bump (&enva_ctor_calls, &enva_saturated);

			if (b != 0)
			{
				enva_bump (&enva_ctor_looping, &enva_saturated);
			}

			if ((uint32_t) a > enva_ctor_max_count)
			{
				enva_ctor_max_count = (uint32_t) a;
			}

			return;

		case ENV_FXA_CTOR_SAMPLE:
			/* the bookkeeping is eech_envobs_store.h enva_sample_read: every read in the digest, a key outside the catalogue counted, the first read of a key kept in order, no counter allowed to wrap */
			enva_sample_read (&enva_samples, a, b, c, d);

			return;

		default:
			break;
	}

	/* the remaining points belong to the boot whose initialisation is running */
	if (boot == NULL)
	{
		enva_bump (&enva_orphans, &enva_saturated);

		return;
	}

	switch (point)
	{
		case ENV_FXA_EFFECTS_BEGIN:
			boot->effects_begin++;
			enva_in_effects = 1;
			break;
		case ENV_FXA_DESCRIPTORS_FILLED:
			boot->filled++;
			enva_take_catalogue (&boot->filled_catalogue);
			break;
		case ENV_FXA_OPEN_TRUE:
			if (enva_in_effects)
			{
				boot->open_true++;
				boot->header_count = a;
			}
			else
			{
				enva_bump (&enva_opens_outside, &enva_saturated);
			}
			break;
		case ENV_FXA_OPEN_FALSE:
			if (enva_in_effects)
			{
				boot->open_false++;
			}
			else
			{
				enva_bump (&enva_opens_outside, &enva_saturated);
			}
			break;
		case ENV_FXA_HEADER_MATCH:
			boot->matched++;
			break;
		case ENV_FXA_HEADER_MISS:
			boot->missed++;
			break;
		case ENV_FXA_HOST_ANSWER:
			if (enva_in_effects)
			{
				boot->host_answers++;
				boot->host_present = a;
				boot->host_header_count = b;
				boot->host_matched = c;
				boot->host_missed = d;
			}
			else
			{
				/* a host answer outside the initialisation is counted with the opens outside it */
				enva_bump (&enva_opens_outside, &enva_saturated);
			}
			break;
		case ENV_FXA_EFFECTS_END:
			boot->effects_end++;
			enva_in_effects = 0;
			enva_take_catalogue (&boot->end_catalogue);
			break;
		case ENV_FXA_HOST_BACKEND:
			boot->host_backend_answers++;
			boot->host_backend_available = a;
			break;
		case ENV_FXA_BACKEND:
			boot->backend++;
			boot->enabled_backend = application_sound_system_enabled;
			boot->initialised_backend = sound_system_initialised;
			break;
		case ENV_FXA_LOAD:
			boot->load++;
			break;
		case ENV_FXA_SKIPPED:
			boot->skipped++;
			break;
		case ENV_FXA_INIT_EXIT:
			boot->exited++;
			boot->enabled_exit = application_sound_system_enabled;
			break;
		default:
			enva_bump (&enva_dropped, &enva_saturated);
			break;
	}
}

/* eech_win32.c eech_fopen: every open attempt by the engine reports its native path here; only the sound header and data files are recorded (kind 0 sound.hdr, 1 sound.dat, by the file name alone, case-insensitively),
   as a count, the result and whether the path was the installation-root retry (it contains a cohokum directory). No path is stored or exported. The caller made the attempt; this adds none. */
static int enva_lower (int ch)
{
	return (ch >= 'A' && ch <= 'Z') ? ch + ('a' - 'A') : ch;
}

static int enva_ends_with (const char *text, const char *tail)
{
	size_t length = strlen (text), tail_length = strlen (tail), n;

	if (length < tail_length)
	{
		return 0;
	}

	for (n = 0; n < tail_length; n++)
	{
		if (enva_lower ((unsigned char) text[length - tail_length + n]) != tail[n])
		{
			return 0;
		}
	}

	return (length == tail_length) || text[length - tail_length - 1] == '/' || text[length - tail_length - 1] == '\\';
}

static int enva_contains_cohokum (const char *text)
{
	static const char needle[] = "cohokum";
	size_t n, m;

	for (n = 0; text[n] != '\0'; n++)
	{
		for (m = 0; needle[m] != '\0' && text[n + m] != '\0'; m++)
		{
			if (enva_lower ((unsigned char) text[n + m]) != needle[m])
			{
				break;
			}
		}

		if (needle[m] == '\0')
		{
			return 1;
		}
	}

	return 0;
}

void eech_envobs_file (const char *native, const char *mode, int ok)
{
	int kind;

	if (native == NULL)
	{
		return;
	}

	if (enva_ends_with (native, "sound.hdr"))
	{
		kind = 0;
	}
	else if (enva_ends_with (native, "sound.dat"))
	{
		kind = 1;
	}
	else
	{
		return;
	}

	enva_bump (&enva_file_attempts[kind], &enva_saturated);

	if (ok)
	{
		enva_bump (&enva_file_ok[kind], &enva_saturated);
	}

	if (enva_attempt_count < ENVA_ATTEMPTS)
	{
		enva_attempt_table[enva_attempt_count].kind = kind;
		enva_attempt_table[enva_attempt_count].ok = ok ? 1 : 0;
		enva_attempt_table[enva_attempt_count].fallback = enva_contains_cohokum (native);
		enva_attempt_table[enva_attempt_count].read = (mode != NULL && mode[0] == 'r') ? 1 : 0;
	}

	enva_bump (&enva_attempt_count, &enva_saturated);
}

/* eech_com_null.c: a call of one of the null DirectSound entry points (0 DirectSoundCreate, 1 IDirectSound_SetCooperativeLevel, 2 IDirectSound_CreateSoundBuffer, 3 IDirectSound_Release) and the low 32 bits of what it returned */
void eech_envobs_com (int which, long long returned)
{
	if (which < 0 || which >= ENVA_COM)
	{
		return;
	}

	enva_bump (&enva_com_calls[which], &enva_saturated);
	enva_com_last[which] = (uint32_t) (uint64_t) returned;
}

/* appended at *used in the caller's buffer (the fixture's group 14 index 15): READ-ONLY. A report that does not fit is refused (-1), never truncated. */
int eech_envobs_sndinit_report (char *text, int capacity, int *used)
{
	static const char *const com_names[ENVA_COM] = { "DirectSoundCreate", "IDirectSound_SetCooperativeLevel", "IDirectSound_CreateSoundBuffer", "IDirectSound_Release" };
	static const char *const kind_names[2] = { "header", "data" };
	struct enva_catalogue now;
	uint32_t n, stored = (enva_boots < ENVA_BOOTS) ? enva_boots : ENVA_BOOTS;

	enva_take_catalogue (&now);

	if (append (text, capacity, used, "env-sndinit-store version=1 boots=%u stored=%u capacity=%d dropped=%u orphans=%u opens_outside_effects=%u catalogue_entries=%d saturated=%u\n", enva_boots, stored, ENVA_BOOTS, enva_dropped, enva_orphans,
		enva_opens_outside, NUM_SOUND_SAMPLE_INDICES, enva_saturated) != 0)
	{
		return -1;
	}

	for (n = 0; n < stored; n++)
	{
		const struct enva_boot *b = &enva_boots_table[n];

		if (append (text, capacity, used, "env-sndinit-boot n=%u no_sound=%d dedicated=%d enabled_before=%d initialised_before=%d effects_begin=%d filled=%d effects_end=%d open_true=%d open_false=%d header_count=%d matched=%d missed=%d"
			" backend=%d enabled_backend=%d initialised_backend=%d load=%d skipped=%d exited=%d enabled_exit=%d\n", n, b->no_sound, b->dedicated, b->enabled_before, b->initialised_before, b->effects_begin, b->filled,
			b->effects_end, b->open_true, b->open_false, b->header_count, b->matched, b->missed, b->backend, b->enabled_backend, b->initialised_backend, b->load, b->skipped, b->exited, b->enabled_exit) != 0)
		{
			return -1;
		}

		/* the host's answer, labelled as a mirror: the campaign's own events above (open_true, open_false, matched, missed) and its file attempts below are NOT these numbers */
		if (append (text, capacity, used, "env-sndinit-host-answer n=%u source=host_reply_mirror answers=%d present=%d header_count=%d matched=%d missed=%d\n", n, b->host_answers, b->host_present, b->host_header_count, b->host_matched,
			b->host_missed) != 0)
		{
			return -1;
		}

		/* the host's backend answer, labelled as a mirror: the campaign's own events (backend, enabled_backend above) and the DirectSound entry counters below are NOT these numbers */
		if (append (text, capacity, used, "env-sndinit-host-backend n=%u source=host_reply_mirror answers=%d available=%d\n", n, b->host_backend_answers, b->host_backend_available) != 0)
		{
			return -1;
		}

		if (append (text, capacity, used, "env-sndinit-catalogue n=%u when=filled named=%u nonzero_size=%u nonzero_rate=%u nonzero_orate=%u digest_w=%016llx\n", n, b->filled_catalogue.named, b->filled_catalogue.nonzero_size,
			b->filled_catalogue.nonzero_rate, b->filled_catalogue.nonzero_orate, (unsigned long long) b->filled_catalogue.digest) != 0)
		{
			return -1;
		}

		if (append (text, capacity, used, "env-sndinit-catalogue n=%u when=end named=%u nonzero_size=%u nonzero_rate=%u nonzero_orate=%u digest_w=%016llx\n", n, b->end_catalogue.named, b->end_catalogue.nonzero_size,
			b->end_catalogue.nonzero_rate, b->end_catalogue.nonzero_orate, (unsigned long long) b->end_catalogue.digest) != 0)
		{
			return -1;
		}
	}

	if (append (text, capacity, used, "env-sndinit-catalogue n=now when=report named=%u nonzero_size=%u nonzero_rate=%u nonzero_orate=%u digest_w=%016llx\n", now.named, now.nonzero_size, now.nonzero_rate, now.nonzero_orate,
		(unsigned long long) now.digest) != 0)
	{
		return -1;
	}

	if (append (text, capacity, used, "env-sndinit-now no_sound=%d dedicated=%d enabled=%d initialised=%d blocked_count=%d application_samples=%d\n", command_line_no_sound, command_line_comms_dedicated_server,
		application_sound_system_enabled, sound_system_initialised, number_of_blocked_sound_samples, number_of_application_sound_samples) != 0)
	{
		return -1;
	}

	if (append (text, capacity, used, "env-sndinit-samples lightning_rate=%d lightning_size=%d lightning_orate=%d thunder_rate=%d thunder_size=%d thunder_orate=%d\n",
		application_sound_effects[SOUND_SAMPLE_INDEX_AMBIENT_LIGHTNING].rate, application_sound_effects[SOUND_SAMPLE_INDEX_AMBIENT_LIGHTNING].size, application_sound_effects[SOUND_SAMPLE_INDEX_AMBIENT_LIGHTNING].original_rate,
		application_sound_effects[SOUND_SAMPLE_INDEX_AMBIENT_THUNDER].rate, application_sound_effects[SOUND_SAMPLE_INDEX_AMBIENT_THUNDER].size, application_sound_effects[SOUND_SAMPLE_INDEX_AMBIENT_THUNDER].original_rate) != 0)
	{
		return -1;
	}

	if (append (text, capacity, used, "env-sndinit-load-side calls=%u recorded=%u\n", enva_side_calls, (enva_side_calls < ENVA_SIDES) ? enva_side_calls : ENVA_SIDES) != 0)
	{
		return -1;
	}

	for (n = 0; n < enva_side_calls && n < ENVA_SIDES; n++)
	{
		if (append (text, capacity, used, "env-sndinit-load-side-call n=%u side=%d enabled=%d no_sound=%d dedicated=%d\n", n, enva_sides[n].side, enva_sides[n].enabled, enva_sides[n].no_sound, enva_sides[n].dedicated) != 0)
		{
			return -1;
		}
	}

	for (n = 0; n < 2; n++)
	{
		if (append (text, capacity, used, "env-sndinit-file kind=%s attempts=%u ok=%u\n", kind_names[n], enva_file_attempts[n], enva_file_ok[n]) != 0)
		{
			return -1;
		}
	}

	if (append (text, capacity, used, "env-sndinit-file-log attempts=%u recorded=%u\n", enva_attempt_count, (enva_attempt_count < ENVA_ATTEMPTS) ? enva_attempt_count : ENVA_ATTEMPTS) != 0)
	{
		return -1;
	}

	for (n = 0; n < enva_attempt_count && n < ENVA_ATTEMPTS; n++)
	{
		if (append (text, capacity, used, "env-sndinit-file-attempt n=%u kind=%s ok=%d installation_root_retry=%d read_mode=%d\n", n, kind_names[enva_attempt_table[n].kind], enva_attempt_table[n].ok,
			enva_attempt_table[n].fallback, enva_attempt_table[n].read) != 0)
		{
			return -1;
		}
	}

	for (n = 0; n < ENVA_COM; n++)
	{
		if (append (text, capacity, used, "env-sndinit-com function=%s calls=%u last_return_w=%08x\n", com_names[n], enva_com_calls[n], enva_com_last[n]) != 0)
		{
			return -1;
		}
	}

	if (append (text, capacity, used, "env-sndinit-ctor calls=%u looping=%u max_count=%u sample_reads=%u distinct=%u out_of_domain=%u digest_w=%016llx\n", enva_ctor_calls, enva_ctor_looping, enva_ctor_max_count,
		enva_samples.reads, enva_samples.distinct, enva_samples.out_of_domain, (unsigned long long) enva_samples.digest) != 0)
	{
		return -1;
	}

	for (n = 0; n < enva_samples.distinct && n < NUM_SOUND_SAMPLE_INDICES; n++)
	{
		int key = enva_sample_order[n];

		if (append (text, capacity, used, "env-sndinit-sample n=%u index=%d rate=%d size=%d orate=%d uses=%u\n", n, key, enva_sample_rate[key], enva_sample_size[key], enva_sample_orate[key],
			enva_sample_uses[key]) != 0)
		{
			return -1;
		}
	}

	return 0;
}

/* ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
   THE HELICOPTER ROTOR SOUND PROBE (ss_envfx.h ENV_FXR; EEDCS-51, D3). One record for each call of the original update_local_helicopter_rotor_sounds (helicopter/helicop.c), complete enough for a later independent
   bit-for-bit replay of the function from the record alone: the inputs it reads (the helicopter's stored main rotor rpm and player kind, whether a current flight model exists and, when it does, the stored words of its
   acceleration vector and the left, right and APU engine rpm), the ORDERED pause and resume requests with the sound sub-type and the delay (as bits), every special-effect child it walks in order (entity type; for a sound
   effect its sub-type and its stored amplification and pitch BEFORE and AFTER the original's switch), and the shared seed word on entry and on exit (so any draw inside the call, by the pause and resume helpers
   included, is visible; this probe itself draws nothing). Everything is READ with memcpy of the already stored words or with integer reads; no float is loaded by value, computed or written, and no sound, entity or game function
   other than the plain accessors is called.

   It is SEPARATE from the event stream, the sound entity probe, the lightning probe and the sound initialisation probe, and is armed ONLY by eech_envobs_rotor_arm (before boot): until then every point returns at once, so the
   observation-off stream and every older report are unchanged. It is a PAGED store: the driver drains it after every frame (eech_envobs_rotor_events), and once everything stored has been read the page is empty again. The domain
   limits are stated in the report, never assumed: ENVR_CALLS calls pending at once, ENVR_ACTIONS requests and ENVR_CHILDREN special-effect children per call. A call that does not fit the page is counted in `dropped`; a request or child
   beyond the per-call limits is counted in `over_actions` / `over_children`; a call that begins inside another, a point with no call open and a point of another number are counted too. Every such counter is reported and the
   driver refuses the capture if one is not zero: nothing is truncated into a result. `phase` is set by the controlled fixture (eech_env_fixture.c) so a world-input variation is never mistaken for natural coverage. */

#define ENVR_CALLS 1024
#define ENVR_ACTIONS 8
#define ENVR_CHILDREN 32

struct envr_action
{
	uint32_t kind, sub, delay;
};

struct envr_child
{
	uint32_t type, sub, before_amp, before_pitch, after_amp, after_pitch, after_seen;
};

struct envr_call
{
	uint32_t seq, time_ms, phase, entity;
	int32_t player, rng_enter, rng_exit;
	uint32_t fd_present, rpm, acc[3], lrpm, rrpm, apu;
	uint32_t actions, visited, sound;
	struct envr_action action[ENVR_ACTIONS];
	struct envr_child child[ENVR_CHILDREN];
};

static struct envr_call envr_pages[ENVR_CALLS];
static struct envr_call envr_open;
static const void *envr_child_pointer[ENVR_CHILDREN];
static int envr_armed, envr_is_open;
static uint32_t envr_phase, envr_stored, envr_read, envr_next_seq, envr_total, envr_drained;
static uint32_t envr_dropped, envr_over_actions, envr_over_children, envr_nested, envr_orphans, envr_bad_point;

void eech_envobs_rotor_arm (int on)
{
	if (on)
	{
		memset (envr_pages, 0, sizeof (envr_pages));
		memset (&envr_open, 0, sizeof (envr_open));
		memset (envr_child_pointer, 0, sizeof (envr_child_pointer));
		envr_is_open = 0;
		envr_phase = envr_stored = envr_read = envr_next_seq = envr_total = envr_drained = 0;
		envr_dropped = envr_over_actions = envr_over_children = envr_nested = envr_orphans = envr_bad_point = 0;
	}

	envr_armed = on ? 1 : 0;
}

int eech_envobs_rotor_is_armed (void)
{
	return envr_armed;
}

void eech_envobs_rotor_phase (int phase)
{
	envr_phase = (uint32_t) phase;
}

static void envr_sound_words (const void *p, uint32_t *sub, uint32_t *amp, uint32_t *pitch)
{
	const sound_effect *raw = (const sound_effect *) get_local_entity_data ((entity *) p);

	*sub = (uint32_t) raw->eff.sub_type;
	memcpy (amp, &raw->amplification, sizeof (*amp));
	memcpy (pitch, &raw->pitch, sizeof (*pitch));
}

void eech_envobs_rotor (int point, const void *p, int a, int b, unsigned int w)
{
	struct envr_call *c = &envr_open;
	entity *en = (entity *) p;
	uint32_t slot;

	if (!envr_armed)
	{
		return;
	}

	if (point < 0 || point >= ENV_FXR_POINT_COUNT)
	{
		envr_bad_point++;

		return;
	}

	if (point == ENV_FXR_ENTER)
	{
		const helicopter *raw = (const helicopter *) get_local_entity_data (en);
		const dynamics_type *fd = current_flight_dynamics;

		if (envr_is_open)
		{
			envr_nested++;
		}

		memset (c, 0, sizeof (*c));
		memset (envr_child_pointer, 0, sizeof (envr_child_pointer));
		envr_is_open = 1;
		c->time_ms = eech_headless_time_ms ();
		c->phase = envr_phase;
		c->entity = (uint32_t) get_local_entity_index (en);
		c->player = (int32_t) raw->player;
		memcpy (&c->rpm, &raw->main_rotor_rpm, sizeof (c->rpm));
		c->fd_present = (fd != NULL) ? 1 : 0;

		if (fd != NULL)
		{
			memcpy (&c->acc[0], &fd->model_acceleration_vector.x, sizeof (c->acc[0]));
			memcpy (&c->acc[1], &fd->model_acceleration_vector.y, sizeof (c->acc[1]));
			memcpy (&c->acc[2], &fd->model_acceleration_vector.z, sizeof (c->acc[2]));
			memcpy (&c->lrpm, &fd->left_engine_n1_rpm.value, sizeof (c->lrpm));
			memcpy (&c->rrpm, &fd->right_engine_n1_rpm.value, sizeof (c->rrpm));
			memcpy (&c->apu, &fd->apu_rpm.value, sizeof (c->apu));
		}

		c->rng_enter = (int32_t) random_number_seed;

		return;
	}

	if (!envr_is_open)
	{
		envr_orphans++;

		return;
	}

	switch (point)
	{
		case ENV_FXR_ACTION:
			if (c->actions < ENVR_ACTIONS)
			{
				c->action[c->actions].kind = (uint32_t) a;
				c->action[c->actions].sub = (uint32_t) b;
				c->action[c->actions].delay = w;
			}
			else
			{
				envr_over_actions++;
			}

			c->actions++;
			break;
		case ENV_FXR_CHILD:
			slot = c->visited;
			c->visited++;

			if (slot >= ENVR_CHILDREN)
			{
				envr_over_children++;

				break;
			}

			envr_child_pointer[slot] = p;
			c->child[slot].type = (uint32_t) en->type;

			if (en->type == ENTITY_TYPE_SOUND_EFFECT)
			{
				c->sound++;
				envr_sound_words (p, &c->child[slot].sub, &c->child[slot].before_amp, &c->child[slot].before_pitch);
			}

			break;
		case ENV_FXR_CHILD_AFTER:
			/* the same sound effect child, straight after its CHILD point */
			slot = (c->visited > 0) ? c->visited - 1 : 0;

			if (c->visited == 0 || slot >= ENVR_CHILDREN)
			{
				break;
			}

			if (envr_child_pointer[slot] != p)
			{
				envr_orphans++;

				break;
			}

			{
				uint32_t sub;

				envr_sound_words (p, &sub, &c->child[slot].after_amp, &c->child[slot].after_pitch);
			}

			c->child[slot].after_seen = 1;
			break;
		default:
			/* ENV_FXR_EXIT */
			c->rng_exit = (int32_t) random_number_seed;
			c->seq = envr_next_seq++;
			envr_total++;
			envr_is_open = 0;

			if (envr_stored >= ENVR_CALLS)
			{
				envr_dropped++;
			}
			else
			{
				envr_pages[envr_stored++] = *c;
			}

			break;
	}
}

/* one formatted call: every field as an integer or a word, the lists as colon-separated tuples (`-` when empty) */
static int envr_put (char *line, size_t size, size_t *n, const char *format, ...) __attribute__ ((format (printf, 4, 5)));

static int envr_put (char *line, size_t size, size_t *n, const char *format, ...)
{
	va_list args;
	int w;

	va_start (args, format);
	w = vsnprintf (line + *n, size - *n, format, args);
	va_end (args);

	if (w < 0 || (size_t) w >= size - *n)
	{
		return -1;
	}

	*n += (size_t) w;

	return 0;
}

static int envr_format (char *line, size_t size, const struct envr_call *c)
{
	size_t n = 0;
	uint32_t i, shown;

	if (envr_put (line, size, &n, "env-rotor-call seq=%u t=%u phase=%u ent=%u player=%d fd=%u rng_in=%d rng_out=%d rpm_w=%08x acc_x_w=%08x acc_y_w=%08x acc_z_w=%08x lrpm_w=%08x rrpm_w=%08x apu_w=%08x actions=%u visited=%u sound=%u act=",
		c->seq, c->time_ms, c->phase, c->entity, (int) c->player, c->fd_present, (int) c->rng_enter, (int) c->rng_exit, c->rpm, c->acc[0], c->acc[1], c->acc[2], c->lrpm, c->rrpm, c->apu, c->actions, c->visited, c->sound) != 0)
	{
		return -1;
	}

	shown = (c->actions < ENVR_ACTIONS) ? c->actions : ENVR_ACTIONS;

	if (shown == 0 && envr_put (line, size, &n, "-") != 0)
	{
		return -1;
	}

	for (i = 0; i < shown; i++)
	{
		if (envr_put (line, size, &n, "%s%u:%u:%08x", (i == 0) ? "" : ",", c->action[i].kind, c->action[i].sub, c->action[i].delay) != 0)
		{
			return -1;
		}
	}

	if (envr_put (line, size, &n, " kids=") != 0)
	{
		return -1;
	}

	shown = (c->visited < ENVR_CHILDREN) ? c->visited : ENVR_CHILDREN;

	if (shown == 0 && envr_put (line, size, &n, "-") != 0)
	{
		return -1;
	}

	for (i = 0; i < shown; i++)
	{
		if (envr_put (line, size, &n, "%s%u:%u:%08x:%08x:%08x:%08x:%u", (i == 0) ? "" : ",", c->child[i].type, c->child[i].sub, c->child[i].before_amp, c->child[i].before_pitch, c->child[i].after_amp,
			c->child[i].after_pitch, c->child[i].after_seen) != 0)
		{
			return -1;
		}
	}

	return envr_put (line, size, &n, "\n");
}

int eech_envobs_rotor_report (char *text, int capacity)
{
	int used = 0;

	text[0] = '\0';

	return append (text, capacity, &used,
		"env-rotor-report version=1 armed=%d open=%d phase=%u capacity=%d max_actions=%d max_children=%d calls_total=%u drained=%u pending=%u dropped=%u over_actions=%u over_children=%u nested=%u orphans=%u bad_points=%u\n",
		envr_armed, envr_is_open, envr_phase, ENVR_CALLS, ENVR_ACTIONS, ENVR_CHILDREN, envr_total, envr_drained, envr_stored - envr_read, envr_dropped, envr_over_actions, envr_over_children, envr_nested,
		envr_orphans, envr_bad_point);
}

/* drains complete call lines while they fit `capacity`; `more` is 1 while stored calls remain unread. When everything stored has been read the page is empty again. */
int eech_envobs_rotor_events (char *text, int capacity, int *more)
{
	int used = 0;
	char line[4096];

	text[0] = '\0';
	*more = 0;

	while (envr_read < envr_stored)
	{
		int before = used;

		if (envr_format (line, sizeof (line), &envr_pages[envr_read]) != 0)
		{
			return -1;
		}

		if (append (text, capacity, &used, "%s", line) != 0)
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

		envr_read++;
		envr_drained++;
	}

	envr_stored = envr_read = 0;

	return 0;
}

#endif
