/*
 * TEST-ONLY (EEDCS-50, D3; feature model-host-fixture): diagnostics of the host-owned model data (aphavoc/.../mb_hostmodel.h). Never part of the shipping module.
 *
 * The host-owned accessors are the SHIPPING code; this file adds three things to a build that has them, so a run can show where its answers came from. The entries that
 * call them are in eech_engine.c (they run inside the engine entry, as the campaign's own calls do: round-toward-zero, a fatal error unwinding to the entry).
 *
 *   counters   how many catalogue, scene and triangle questions the host was asked and how it answered them, and how often each bounds / radius table was read. A counter
 *              alone proves nothing about authority (it only counts calls); it is read together with
 *   poison     which overwrites every campaign-side array the ORIGINAL accessors read (the scene database's radius, both bounds and collision object index) with NaN and an
 *              out-of-table index. A campaign that goes on to give the same observations as an unpoisoned run has taken them from the host; a read of any of those arrays,
 *              by an accessor or by code that bypasses them, would change them.
 *   probe      one scene's radius and bounds, and one step of its collision-triangle cursor, through the REAL public accessors, in the format of the original fixture's
 *              rows (eech_model_fixture.c): float bit patterns, the return value, the number of the nine buffer words the call wrote (0 for a FALSE: the buffer is left
 *              untouched), the pointer-stability check of the bounds, and the rounding mode (fegetround) before and after the accessor call INSIDE the entry. That is the
 *              rounding mode only: it says nothing of the exception flags, the masks or MXCSR, and nothing about what the entry hands back to its CALLER.
 *   fpu        the COMPLETE outer floating-point state (x87 control and status words, MXCSR) around the REAL public entries, from the caller's side (eech_engine_model_host_fpu,
 *              below): the proof of what a caller sees, which no fegetround check is.
 *
 * None of them draws a random number, advances time or touches an entity.
 */

#ifdef EECH_MODEL_HOST_FIXTURE

#include <fenv.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "entity/special/guide/gd_attck.h"
#include "eech_engine.h"
#include "eech_headless.h"

#ifdef EECH_CAMPAIGN_HOST_MODEL

struct campaign_model_counters
{
	long catalogue, scene, triangle, found, end, absent, bounds_reads, bounds_without_lines_reads, radius_reads;
	long presence_checked, presence_mismatches;
	long target_offset;
	long target_chain, target_world, target_fixed;
};

extern struct campaign_model_counters campaign_model_counts;
extern void campaign_model_poison_arrays (void);
extern void campaign_model_core_residency (int *rows, int *metadata_rows, int *points, int *faces, int *point_refs, int *surface_points);
extern void campaign_model_core_horizon (int *cached, int *uncached);
extern void campaign_model_core_metadata (long *values);
extern int campaign_object_is_structure (int entity_type);
extern int campaign_model_target_point_chain (int scene, int *offsets);

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void bounds_text (char *buffer, size_t capacity, const struct OBJECT_3D_BOUNDS *b)
{
	snprintf (buffer, capacity, "0x%08x,0x%08x,0x%08x,0x%08x,0x%08x,0x%08x", float_bits (b->xmin), float_bits (b->xmax), float_bits (b->ymin), float_bits (b->ymax),
		float_bits (b->zmin), float_bits (b->zmax));
}

int eech_model_host_counters_run (char *text, int capacity)
{
	int rows, metadata_rows, points, faces, point_refs, surface_points, cached, uncached;
	int n, m, k, j;
	long metadata[9];

	n = snprintf (text, (size_t) capacity,
		"model-host catalogue=%ld scene=%ld triangle=%ld found=%ld end=%ld absent=%ld bounds_reads=%ld bounds_without_lines_reads=%ld radius_reads=%ld"
		" presence_checked=%ld presence_mismatches=%ld\n",
		campaign_model_counts.catalogue, campaign_model_counts.scene, campaign_model_counts.triangle, campaign_model_counts.found, campaign_model_counts.end,
		campaign_model_counts.absent, campaign_model_counts.bounds_reads, campaign_model_counts.bounds_without_lines_reads, campaign_model_counts.radius_reads,
		campaign_model_counts.presence_checked, campaign_model_counts.presence_mismatches);

	if ((n < 0) || (n >= capacity))
	{
		return 1;
	}

	/* the loader detachment (EEDCS-50, D3): how many rows of the campaign's own object table still hold each physical buffer, as integer counts over the actual rows (no address),
	   next to the rows that still carry the file's true point count. */
	campaign_model_core_residency (&rows, &metadata_rows, &points, &faces, &point_refs, &surface_points);
	m = snprintf (text + n, (size_t) (capacity - n), "model-core-residency rows=%d metadata_rows=%d points=%d faces=%d point_refs=%d surface_points=%d\n", rows, metadata_rows, points, faces,
		point_refs, surface_points);

	if ((m < 0) || (m >= capacity - n))
	{
		return 1;
	}

	/* the horizon images the boot took: the pre-processed branch (point colours and fog, no point read) and the uncached one (which the hosted engine refuses by name) */
	campaign_model_core_horizon (&cached, &uncached);
	k = snprintf (text + n + m, (size_t) (capacity - n - m), "model-core-horizon cached=%d uncached=%d\n", cached, uncached);

	if ((k < 0) || (k >= capacity - n - m))
	{
		return 1;
	}

	/* the model-file metadata exchange (EEDCS-50, D3, L2): the questions the campaign asked of the host while it read its scenes, how they were answered, and how many private rows the
	   answers filled (rows_filled) or confirmed again (rows_confirmed). The campaign has no model-file parser to compare with: these count the host's answers the campaign took. */
	campaign_model_core_metadata (metadata);
	j = snprintf (text + n + m + k, (size_t) (capacity - n - m - k),
		"model-core-metadata objects_asked=%ld objects_found=%ld objects_absent=%ld rows_filled=%ld rows_confirmed=%ld rows_replaced=%ld animation_asked=%ld animation_names=%ld animation_ends=%ld\n",
		metadata[0], metadata[1], metadata[2], metadata[3], metadata[4], metadata[5], metadata[6], metadata[7], metadata[8]);

	return ((j < 0) || (j >= capacity - n - m - k)) ? 1 : 0;
}

/* The scenes' FINAL texture animation lists as the campaign holds them after the host-fed load (EEDCS-50, D3, L2), in the row grammar of the original observer (eech_model_fixture.c,
   eech_model_animation_run), so the same driver reads them and a literal comparison with the original's rows is possible: scenes first .. first + count - 1.

   WHAT IS MEASURED. Not inferred: the hooks below (patches.rs HOST_ANIM_FIXTURE_PATCHES, HA1..HA7) are ADDITIVE calls at the real stages of read_scene and add_animation, exactly the
   stages the original observer hooks (MA0..MA4): the scene's declared list count was read; each entry of a declared list was read (what fread returned is recorded and handed back
   unchanged); add_animation allocated its grown array (address and size) and appended an entry; read_scene is finishing (the list pointer and count it leaves). None changes a statement,
   an argument, an order or a value, and none allocates or writes a model byte. A row is read from the scene database only where the recorded backing shows the list is the array the
   campaign allocated for it, still the scene's list with the same count, within its allocation and filled in full: every other case is a named outcome and nothing is read. The
   capacity and filled figures are those records (the allocation add_animation or read_scene made, the entries known to be filled), never derived from the count. A scene whose read the
   hooks never saw is absent. The source is built for a scene a custom scene file defined (the hook is told sceneid), file for a database record. The ids are the campaign's own private
   animation numbers (diagnostic data, never the host protocol). The page digest covers the recorded state and the verified projection; it is this profile's own, not the original's. */
#define HOST_ANIM_IDS_MAX 4096
#define HOST_ANIM_PAGE_SCENES 96

enum { HF_ANIM_NONE = 0, HF_ANIM_BEGUN = 1, HF_ANIM_DONE = 2 };

struct hf_anim
{
	int state;
	int custom;                  /* read_scene was given a sceneid: the list is built by add_animation */
	int declared;                /* the count the file declared */
	uintptr_t file_ptr;          /* the array read_scene allocated for a declared list */
	int file_capacity, file_filled, file_bad;
	uintptr_t alloc_ptr;         /* the array add_animation allocated last */
	int alloc_capacity, allocs, appends, overflow;
	int extent;                  /* leading entries known to be filled */
	uintptr_t final_ptr;
	int final_count, fail;
};

static struct hf_anim hf_anim[OBJECT_3D_LAST];
static int hf_anim_unknown_scene;
static unsigned hf_generation;

static struct hf_anim *hf_anim_of (const void *scene)
{
	const struct OBJECT_3D_SCENE_DATABASE_ENTRY *e = (const struct OBJECT_3D_SCENE_DATABASE_ENTRY *) scene;
	ptrdiff_t index;

	if (!objects_3d_scene_database || e < objects_3d_scene_database)
	{
		hf_anim_unknown_scene++;
		return NULL;
	}
	index = e - objects_3d_scene_database;
	if (index <= OBJECT_3D_INVALID_OBJECT_INDEX || index >= OBJECT_3D_LAST)
	{
		hf_anim_unknown_scene++;
		return NULL;
	}
	return &hf_anim[index];
}

static int hf_anim_capacity (const struct hf_anim *a)
{
	return a->allocs ? a->alloc_capacity : a->file_capacity;
}

/* read_scene has read the scene's declared list count */
void eech_hf_anim_begin (int scene, int sceneid, int declared)
{
	struct hf_anim *a;

	if (scene <= OBJECT_3D_INVALID_OBJECT_INDEX || scene >= OBJECT_3D_LAST)
	{
		hf_anim_unknown_scene++;
		return;
	}
	a = &hf_anim[scene];
	memset (a, 0, sizeof (*a));
	a->state = HF_ANIM_BEGUN;
	a->custom = sceneid != 0;
	a->declared = declared;
}

/* one fread of one entry of a declared list: `returned` is what fread returned (the original ignores it) */
void eech_hf_anim_file_read (int scene, int index, int total, const void *list, size_t returned)
{
	struct hf_anim *a;

	if (scene <= OBJECT_3D_INVALID_OBJECT_INDEX || scene >= OBJECT_3D_LAST)
	{
		hf_anim_unknown_scene++;
		return;
	}
	a = &hf_anim[scene];
	if (a->state != HF_ANIM_BEGUN || total != a->declared)
	{
		a->file_bad = 1;
		return;
	}
	if (index == 0)
	{
		a->file_ptr = (uintptr_t) list;
		a->file_capacity = total;
		a->file_filled = 0;
	}
	if (returned == 1 && index == a->file_filled && (uintptr_t) list == a->file_ptr)
	{
		a->file_filled++;
		a->extent = a->file_filled;
	}
	else
	{
		a->file_bad = 1;
	}
}

/* add_animation has allocated `capacity` ints for the scene's list (the entries already in it were copied) */
void eech_hf_anim_alloc (const void *scene, const void *list, int capacity)
{
	struct hf_anim *a = hf_anim_of (scene);

	if (!a)
	{
		return;
	}
	a->allocs++;
	a->alloc_ptr = (uintptr_t) list;
	a->alloc_capacity = capacity;
}

/* add_animation has appended one entry: the scene's count is now the position after it */
void eech_hf_anim_append (const void *scene)
{
	struct hf_anim *a = hf_anim_of (scene);
	int count;

	if (!a)
	{
		return;
	}
	count = ((const struct OBJECT_3D_SCENE_DATABASE_ENTRY *) scene)->number_of_texture_animations;
	a->appends++;
	if (count > hf_anim_capacity (a))
	{
		a->overflow = 1;
	}
	else if (a->extent == count - 1)
	{
		a->extent = count;
	}
}

/* read_scene is finishing the scene: the list and count it leaves */
void eech_hf_anim_done (int scene, int fail)
{
	struct hf_anim *a;
	const struct OBJECT_3D_SCENE_DATABASE_ENTRY *e;

	if (scene <= OBJECT_3D_INVALID_OBJECT_INDEX || scene >= OBJECT_3D_LAST)
	{
		return;
	}
	a = &hf_anim[scene];
	e = &objects_3d_scene_database[scene];
	if (a->state != HF_ANIM_BEGUN)
	{
		return;
	}
	a->state = HF_ANIM_DONE;
	a->final_ptr = (uintptr_t) e->texture_animations;
	a->final_count = e->number_of_texture_animations;
	a->fail = fail != 0;
}

/* the objects are freed: every record is dropped */
void eech_hf_anim_reset (void)
{
	memset (hf_anim, 0, sizeof (hf_anim));
	hf_anim_unknown_scene = 0;
	hf_generation++;
}

static void host_anim_hash (uint64_t *hash, uint64_t value)
{
	int i;

	for (i = 0; i < 8; i++)
	{
		*hash ^= (value >> (8 * i)) & 0xffu;
		*hash *= 0x100000001b3ULL;
	}
}

struct host_anim_view
{
	const char *status, *source;
	int succeeded, count, capacity, filled;
	const int *ids;
};

/* One scene's final list as the hooks recorded it. A list is read (`ids` set) only when it is the array the campaign allocated for it, still the scene's list with the same count,
   within its allocation and its filled entries and within the fixed bound; every other case is a named outcome and nothing is read. The same decision table as the original observer's. */
static void host_anim_view (int scene, struct host_anim_view *v)
{
	const struct hf_anim *a = &hf_anim[scene];
	const struct OBJECT_3D_SCENE_DATABASE_ENTRY *e = &objects_3d_scene_database[scene];
	uintptr_t allocated;

	memset (v, 0, sizeof (*v));
	v->status = "absent";
	v->source = "none";
	if (a->state != HF_ANIM_DONE)
	{
		if (e->number_of_texture_animations != 0 || e->texture_animations)
		{
			v->status = "unobserved"; /* a list the loader made that the hooks did not see */
		}
		return;
	}
	v->succeeded = e->succeeded;
	v->source = a->custom ? "built" : "file";
	v->count = a->final_count;
	v->capacity = hf_anim_capacity (a);
	v->filled = a->extent;
	allocated = a->allocs ? a->alloc_ptr : a->file_ptr;
	if (e->number_of_texture_animations != a->final_count || (uintptr_t) e->texture_animations != a->final_ptr)
	{
		v->status = "stale"; /* the scene's list is no longer the one recorded when its read ended */
	}
	else if (a->final_count < 0 || a->overflow)
	{
		v->status = a->overflow ? "overflow" : "unbacked";
	}
	else if (a->final_count == 0)
	{
		v->status = a->final_ptr ? "unbacked" : "empty";
	}
	else if (a->final_count > HOST_ANIM_IDS_MAX)
	{
		v->status = "capacity";
	}
	else if (!a->final_ptr || a->final_ptr != allocated || a->final_count > v->capacity)
	{
		v->status = "unbacked";
	}
	else if (a->file_bad || a->extent < a->final_count)
	{
		v->status = "short_read";
	}
	else
	{
		v->status = "listed";
		v->ids = (const int *) a->final_ptr;
	}
}

int eech_model_host_animation_run (int first, int count, char *text, int capacity)
{
	uint64_t content = 0xcbf29ce484222325ULL;
	int scene, rows = 0, done = 0, n, m;

	if (!objects_3d_scene_database || !text || capacity <= 0 || first < 1 || count < 1 || count > HOST_ANIM_PAGE_SCENES || first > (int) OBJECT_3D_LAST - count)
	{
		return 1;
	}

	for (scene = OBJECT_3D_INVALID_OBJECT_INDEX + 1; scene < OBJECT_3D_LAST; scene++)
	{
		struct host_anim_view v;
		int i;

		host_anim_view (scene, &v);
		done += hf_anim[scene].state == HF_ANIM_DONE;
		host_anim_hash (&content, (uint64_t) scene);
		host_anim_hash (&content, (uint64_t) hf_anim[scene].state);
		host_anim_hash (&content, (uint64_t) (unsigned) v.count);
		host_anim_hash (&content, (uint64_t) (unsigned) v.filled);
		for (i = 0; v.status[i]; i++)
		{
			host_anim_hash (&content, (uint64_t) (unsigned char) v.status[i]);
		}
		for (i = 0; v.ids && i < v.count; i++)
		{
			host_anim_hash (&content, (uint64_t) (unsigned) v.ids[i]);
		}
	}

	n = snprintf (text, (size_t) capacity, "model-anim-page first=%d count=%d scene_last=%d generation=%u scenes_read=%d unknown_scene=%d content=%016llx\n", first, count, (int) OBJECT_3D_LAST,
		hf_generation, done, hf_anim_unknown_scene, (unsigned long long) content);

	if ((n < 0) || (n >= capacity))
	{
		return 1;
	}

	for (scene = first; scene < first + count; scene++)
	{
		struct host_anim_view v;
		int i;

		host_anim_view (scene, &v);

		m = snprintf (text + n, (size_t) (capacity - n), "model-anim scene=%d name=%s status=%s source=%s succeeded=%d count=%d capacity=%d filled=%d ids=", scene,
			object_3d_scene_names[scene] ? object_3d_scene_names[scene] : "", v.status, v.source, v.succeeded, v.count, v.capacity, v.filled);

		if ((m < 0) || (m >= capacity - n))
		{
			return 1;
		}

		n += m;

		if (v.ids)
		{
			for (i = 0; i < v.count; i++)
			{
				m = snprintf (text + n, (size_t) (capacity - n), "%s%d", i ? "," : "", v.ids[i]);

				if ((m < 0) || (m >= capacity - n))
				{
					return 1;
				}

				n += m;
			}
		}
		else
		{
			m = snprintf (text + n, (size_t) (capacity - n), "-");

			if ((m < 0) || (m >= capacity - n))
			{
				return 1;
			}

			n += m;
		}

		m = snprintf (text + n, (size_t) (capacity - n), "\n");

		if ((m < 0) || (m >= capacity - n))
		{
			return 1;
		}

		n += m;
		rows++;
	}

	m = snprintf (text + n, (size_t) (capacity - n), "model-anim-end first=%d count=%d rows=%d\n", first, count, rows);

	return ((m < 0) || (m >= capacity - n)) ? 1 : 0;
}

/* An UNSUPPORTED reader, called for real: add_3d_horizon_image on the campaign's own horizon environment (main_3d_env, made at boot) with a basename no installation has. It has
   no pre-processed `.bin`, so the real reader takes its uncached branch, where the hosted engine stops by name BEFORE any image I/O or geometry read (patches.rs LU1). The stop
   unwinds to the engine entry as a fatal error and the engine is then poisoned: nothing is asked after it. A return from this function means the reader did NOT stop. */
int eech_model_host_horizon_refusal_run (char *text, int capacity)
{
	extern env_3d *main_3d_env;
	int n;

	if (!main_3d_env)
	{
		return 1;
	}
	add_3d_horizon_image (main_3d_env, "eech-fixture-no-such-horizon", 0.0f, WEATHERMODE_DRY);
	n = snprintf (text, (size_t) capacity, "model-core-horizon-refusal returned=1\n");

	return ((n < 0) || (n >= capacity)) ? 1 : 0;
}

/* The host-authored target-point offsets as the campaign HOLDS them (EEDCS-52, D3): `initialise_target_point_offsets` copied each scene's three LOCAL coordinates from the host's
   TARGET_POINT_OFFSET answers into the private `target_point_offsets` table the direct guidance consumer reads. This reports how many such questions the host was asked since boot
   (the real boot asks once per scene per initialiser invocation, and invokes it twice) and, for scenes `first ..= first + count - 1`, the held words. A held value that equals what
   the host answered is the authority; one the campaign could have computed itself would show only if the host's answer were changed. Read-only. */
int eech_model_host_target_offsets_run (int first, int count, char *text, int capacity)
{
	int i, n, used;

	if ((first < 1) || (count < 1) || (first > (int) OBJECT_3D_LAST - count))
	{
		return 1;
	}

	used = snprintf (text, (size_t) capacity, "model-host-offsets asked=%ld\n", campaign_model_counts.target_offset);

	if ((used < 0) || (used >= capacity))
	{
		return 1;
	}

	for (i = first; i < first + count; i++)
	{
		n = snprintf (text + used, (size_t) (capacity - used), "model-host-offset scene=%d cache=0x%08x,0x%08x,0x%08x\n", i, float_bits (target_point_offsets[i].x),
			float_bits (target_point_offsets[i].y), float_bits (target_point_offsets[i].z));

		if ((n < 0) || (n >= capacity - used))
		{
			return 1;
		}

		used += n;
	}

	return 0;
}

/* A used-entity list node is trusted only if it is an aligned slot of the entity heap this run allocated (compared as integers); the same check the original target-point
   observer makes before it dereferences a node. */
static int host_heap_slot (const entity *en)
{
	uintptr_t base = (uintptr_t) entities;
	uintptr_t at = (uintptr_t) en;

	if (!entities || number_of_entities <= 0)
	{
		return 0;
	}

	if ((at < base) || (at >= base + ((uintptr_t) number_of_entities * sizeof (entity))))
	{
		return 0;
	}

	return ((at - base) % sizeof (entity)) == 0;
}

/* The DIRECT guidance consumer, called for real on a real entity (EEDCS-52, D3): `calculate_attack_guide_intercept_point` (gd_attck.c) reads all three LOCAL offset coordinates of
   the target's shape from `target_point_offsets` and returns them as its intercept point when the target is not moving and the weapon is not a ballistic one. This finds, among the
   live mobiles, the first whose shape is a valid scene and whose velocity is exactly zero, picks the first weapon with a guidance type (so the ballistic drop branch is not taken),
   calls the consumer once and reports the held offset, the consumer's three coordinates and their equality. No candidate is UNAVAILABLE with its reason, never a pass. The entity is
   not changed; the function draws no random number and advances no time. Not an engine target selection: the entity is chosen by this diagnostic. */
int eech_model_host_attack_consumer_run (char *text, int capacity)
{
	entity *en, *target = NULL;
	int visited = 0, shape = 0, weapon = -1, w, equal;
	vec3d ip;
	float velocity;
	int n;

	for (en = get_local_entity_list (); en;)
	{
		int type;

		if ((visited >= number_of_entities) || !host_heap_slot (en))
		{
			n = snprintf (text, (size_t) capacity, "model-host-consumer status=unavailable reason=entity_list_not_trusted visited=%d\n", visited);
			return ((n < 0) || (n >= capacity)) ? 1 : 0;
		}

		visited++;
		type = (int) en->type;

		if (((type == ENTITY_TYPE_ROUTED_VEHICLE) || (type == ENTITY_TYPE_ANTI_AIRCRAFT) || (type == ENTITY_TYPE_SHIP_VEHICLE)) && get_local_entity_data (en))
		{
			int candidate = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);

			if ((candidate > OBJECT_3D_INVALID_OBJECT_INDEX) && (candidate < OBJECT_3D_LAST))
			{
				velocity = get_local_entity_float_value (en, FLOAT_TYPE_VELOCITY);

				if ((float_bits (velocity) & 0x7fffffffu) == 0)
				{
					target = en;
					shape = candidate;
					break;
				}
			}
		}

		en = get_local_entity_succ (en);
	}

	for (w = 0; w < NUM_ENTITY_SUB_TYPE_WEAPONS; w++)
	{
		if (weapon_database[w].guidance_type != WEAPON_GUIDANCE_TYPE_NONE)
		{
			weapon = w;
			break;
		}
	}

	if (!target || (weapon < 0))
	{
		n = snprintf (text, (size_t) capacity, "model-host-consumer status=unavailable reason=%s visited=%d\n", !target ? "no_stationary_mobile_with_a_valid_shape" : "no_guided_weapon", visited);
		return ((n < 0) || (n >= capacity)) ? 1 : 0;
	}

	memset (&ip, 0, sizeof (ip));
	calculate_attack_guide_intercept_point (target, target, weapon, &ip);

	equal = (float_bits (ip.x) == float_bits (target_point_offsets[shape].x)) && (float_bits (ip.y) == float_bits (target_point_offsets[shape].y)) &&
		(float_bits (ip.z) == float_bits (target_point_offsets[shape].z));

	n = snprintf (text, (size_t) capacity,
		"model-host-consumer status=called scene=%d entity_type=%d weapon=%d cache=0x%08x,0x%08x,0x%08x consumer=0x%08x,0x%08x,0x%08x equal=%d\n", shape, (int) target->type, weapon,
		float_bits (target_point_offsets[shape].x), float_bits (target_point_offsets[shape].y), float_bits (target_point_offsets[shape].z), float_bits (ip.x), float_bits (ip.y),
		float_bits (ip.z), equal);

	return ((n < 0) || (n >= capacity)) ? 1 : 0;
}

/* The target-point QUERY, called for real on real entities (EEDCS-52, D3): `get_local_entity_target_point` (en_maths.c) on the live entities this diagnostic picks, with the host questions
   it asked counted around each call. Cases, each one row `model-host-tp case=... status=called|unavailable`:
     instance_a, instance_b   the first two live vehicles with an instance (any outcome: the counters say whether the host named a chain and was asked for a position). The instance
                              viewpoint is first set to a sentinel and read after the call, so `vp_written` shows whether the getter wrote it (it writes only when the host named a chain). When `mutate`
                              is set and the host named a chain, the leaf sub-instance's live relative position, heading and scale are changed for the call and restored after it, and `pose` is the
                              placement as this diagnostic itself read it from the live tree (the root pose, then each level of the chain), for the script to compare with what the host was sent.
     instance_none            a vehicle whose instance pointer is set to NULL for the call (not fixed): the original's no-instance, non-fixed fallback, restored after.
     fixed_a, fixed_b         the first fixed structures with a valid shape: one whose cached host offset has a horizontal part and one that has none, when both exist.
     fixed_invalid            a fixed structure whose shape is set to the invalid index for the call, restored after: the bit-for-bit position copy.
   A host-named chain that does not lie inside the instance (a negative or too-large sibling position, a missing child array) is a named REFUSAL of the row (`status=refused
   reason=chain_path_outside_instance`), made by the ONE bounded walk below before the sentinel, any change, the getter or any pose read; the row returns normally, nothing was touched, and
   the production world-chain guard is never relied on to catch what this fixture would dereference first. Restoration is READ BACK on normal return (`restored=1` measured equal,
   `0` measured different, `-1` nothing was changed); nothing here claims cleanup after a fatal unwind.
   `base` is the entity position as the getter reads it, `heading` the entity heading, `cache` the cached host offset of the shape, `result` the getter's answer, `chain`, `world` and
   `fixed` the questions asked of the host DURING the call. Nothing else is changed, no random number is drawn and no time advances. A case with no candidate is UNAVAILABLE with its
   reason, never a pass. */
static void tp_bits3 (char *out, size_t capacity, const vec3d *v)
{
	snprintf (out, capacity, "0x%08x,0x%08x,0x%08x", float_bits (v->x), float_bits (v->y), float_bits (v->z));
}

static int tp_vehicle_instance_field (entity *en, object_3d_instance ***field)
{
	void *data = get_local_entity_data (en);

	switch ((int) en->type)
	{
		case ENTITY_TYPE_ANTI_AIRCRAFT:
		case ENTITY_TYPE_ROUTED_VEHICLE:
		case ENTITY_TYPE_SHIP_VEHICLE:
		{
			*field = &((vehicle *) data)->inst3d;
			return 1;
		}
		default:
		{
			return 0;
		}
	}
}

/* the live placement as this diagnostic reads it: the root, then each level of the host's chain by walking the instance's own sub-instance arrays */
static int tp_pose_text (object_3d_instance *inst, object_3d_sub_instance *const *path, int levels, char *out, int capacity)
{
	int used = 0, i, level, n;
	float words[15];

	words[0] = inst->vp.position.x;
	words[1] = inst->vp.position.y;
	words[2] = inst->vp.position.z;
	memcpy (&words[3], inst->vp.attitude, 9 * sizeof (float));
	words[12] = inst->relative_scale.x;
	words[13] = inst->relative_scale.y;
	words[14] = inst->relative_scale.z;

	for (i = 0; i < 15; i++)
	{
		n = snprintf (out + used, (size_t) (capacity - used), "%s0x%08x", i ? "," : "", float_bits (words[i]));
		if ((n < 0) || (n >= capacity - used)) return 0;
		used += n;
	}

	for (level = 0; level < levels; level++)
	{
		object_3d_sub_instance *node = path[level];
		float level_words[9];

		level_words[0] = node->relative_position.x;
		level_words[1] = node->relative_position.y;
		level_words[2] = node->relative_position.z;
		level_words[3] = node->relative_heading;
		level_words[4] = node->relative_pitch;
		level_words[5] = node->relative_roll;
		level_words[6] = node->relative_scale.x;
		level_words[7] = node->relative_scale.y;
		level_words[8] = node->relative_scale.z;

		for (i = 0; i < 9; i++)
		{
			n = snprintf (out + used, (size_t) (capacity - used), ",0x%08x", float_bits (level_words[i]));
			if ((n < 0) || (n >= capacity - used)) return 0;
			used += n;
		}
	}

	return 1;
}

static int tp_row (char *text, int capacity, int *used, const char *name, entity *en, int mutate, int what)
{
	/* what: 0 a vehicle's own instance, 1 the instance pointer set to NULL, 2 a fixed structure as it is, 3 a fixed structure with the invalid shape */
	object_3d_instance **field = NULL, *inst = NULL, *saved_instance = NULL;
	vec3d base, result, vp_position_saved;
	matrix3x3 vp_attitude_saved;
	long chain0 = campaign_model_counts.target_chain, world0 = campaign_model_counts.target_world, fixed0 = campaign_model_counts.target_fixed;
	char base_text[80], result_text[80], cache_text[80], pose_text[2048], mutation[96];
	int scene = 0, vp_written = -1, restored = -1, shape_saved = 0, pose_ok = 0, levels = 0, legacy_same = -1, offsets[CAMPAIGN_MODEL_CHAIN_MAX], n;
	float heading_value = 0.0f;
	uint32_t heading_bits = 0;
	object_3d_sub_instance *path[CAMPAIGN_MODEL_CHAIN_MAX], *leaf = NULL;
	matrix3x3 entity_attitude;
	vec3d leaf_position;
	float leaf_heading;
	vec3d leaf_scale;

	pose_text[0] = '-';
	pose_text[1] = '\0';
	mutation[0] = '-';
	mutation[1] = '\0';
	memset (&result, 0, sizeof (result));

	if (what <= 1)
	{
		tp_vehicle_instance_field (en, &field);
		saved_instance = *field;
		inst = saved_instance;
		scene = inst->object_number;
	}
	else
	{
		scene = (int) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
	}

	get_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &base);
	heading_value = get_local_entity_float_value (en, FLOAT_TYPE_HEADING);
	heading_bits = float_bits (heading_value);

	if (what == 0)
	{
		/* the host's chain, asked by this diagnostic itself (not counted in the deltas below), to place the mutation and to read the live pose independently */
		levels = campaign_model_target_point_chain (scene, offsets);
		/* the diagnostic's own ask raised the counter: the deltas are taken from the value after it */
		chain0 = campaign_model_counts.target_chain;

		/* THE ONE BOUNDED WALK, a mandatory preflight. It follows the positions the host named down the instance's own sub-instance arrays, each checked against the scene database's counts,
		   and records every node it reaches: the legacy comparison, the mutation and the pose read below use these nodes and never walk the offsets again. A path that is not inside the
		   instance is a named refusal of the row, returned before the sentinel, any change, the getter or any pose read. */
		{
			object_3d_sub_object_search_data search;
			object_3d_sub_instance *walk = inst->sub_objects, *reached = NULL;
			object_3d_database_entry *entries = objects_3d_scene_database[scene].sub_objects;
			int count = objects_3d_scene_database[scene].number_of_sub_objects, step, found;

			search.search_depth = 0;
			search.search_object = inst;
			search.sub_object_index = OBJECT_3D_SUB_OBJECT_TARGET_POINT;
			found = (find_object_3d_sub_object (&search) == SUB_OBJECT_SEARCH_RESULT_OBJECT_FOUND);

			for (step = 0; step < levels; step++)
			{
				if ((offsets[step] < 0) || (offsets[step] >= count) || (!walk) || (!entries))
				{
					n = snprintf (text + *used, (size_t) (capacity - *used),
						"model-host-tp case=%s status=refused reason=chain_path_outside_instance scene=%d level=%d offset=%d levels=%d chain=%ld world=%ld fixed=%ld getter_called=0 vp_touched=0\n", name,
						scene, step, offsets[step], levels, campaign_model_counts.target_chain - chain0, campaign_model_counts.target_world - world0,
						campaign_model_counts.target_fixed - fixed0);

					if ((n < 0) || (n >= capacity - *used))
					{
						return 1;
					}

					*used += n;
					return 0;
				}

				reached = &walk[offsets[step]];
				path[step] = reached;
				count = entries[offsets[step]].number_of_sub_objects;
				entries = entries[offsets[step]].sub_objects;
				walk = reached->sub_objects;
			}

			/* the ORIGINAL's own selection as the oracle of the host's: the legacy depth-0 search (compiled, not called by the campaign) reaches the same sub-instance as the walk, or both find none */
			legacy_same = found ? ((levels > 0) && (reached == search.result_sub_object)) : (levels == 0);
			leaf = reached;
		}

		vp_position_saved = inst->vp.position;
		memcpy (vp_attitude_saved, inst->vp.attitude, sizeof (vp_attitude_saved));
		inst->vp.position.x = 12345.0f;
		inst->vp.position.y = 23456.0f;
		inst->vp.position.z = 34567.0f;
		{
			int row_index, column;

			for (row_index = 0; row_index < 3; row_index++)
			{
				for (column = 0; column < 3; column++)
				{
					inst->vp.attitude[row_index][column] = 7777.0f;
				}
			}
		}

		if (mutate && (levels > 0))
		{
			leaf_position = leaf->relative_position;
			leaf_heading = leaf->relative_heading;
			leaf_scale = leaf->relative_scale;
			leaf->relative_position.x += 1.0f;
			leaf->relative_heading += 0.25f;
			leaf->relative_scale.x *= 1.5f;
			snprintf (mutation, sizeof (mutation), "x+1,heading+0.25,scale_x*1.5");
		}
		else
		{
			leaf = NULL;
		}
	}
	else if (what == 1)
	{
		*field = NULL;
	}
	else if (what == 3)
	{
		shape_saved = scene;
		set_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE, OBJECT_3D_INVALID_OBJECT_INDEX);
	}

	memset (&result, 0, sizeof (result));
	get_local_entity_target_point (en, &result);

	if (what == 0)
	{
		/* the FULL meaning of "the getter wrote the instance viewpoint": position AND attitude are the entity's (written), both are still the sentinels (untouched), or one of each (partial) */
		int position_written, position_untouched, attitude_written, attitude_untouched, row_index, column;

		get_local_entity_attitude_matrix (en, entity_attitude);
		position_written = (float_bits (inst->vp.position.x) == float_bits (base.x)) && (float_bits (inst->vp.position.y) == float_bits (base.y)) &&
			(float_bits (inst->vp.position.z) == float_bits (base.z));
		position_untouched = (float_bits (inst->vp.position.x) == float_bits (12345.0f)) && (float_bits (inst->vp.position.y) == float_bits (23456.0f)) &&
			(float_bits (inst->vp.position.z) == float_bits (34567.0f));
		attitude_written = (memcmp (inst->vp.attitude, entity_attitude, sizeof (entity_attitude)) == 0);
		attitude_untouched = 1;

		for (row_index = 0; row_index < 3; row_index++)
		{
			for (column = 0; column < 3; column++)
			{
				attitude_untouched = attitude_untouched && (float_bits (inst->vp.attitude[row_index][column]) == float_bits (7777.0f));
			}
		}

		vp_written = (position_written && attitude_written) ? 1 : ((position_untouched && attitude_untouched) ? 0 : 2);

		if (levels > 0)
		{
			pose_ok = tp_pose_text (inst, path, levels, pose_text, (int) sizeof (pose_text));

			if (!pose_ok)
			{
				pose_text[0] = '-';
				pose_text[1] = '\0';
			}
		}

		if (leaf)
		{
			leaf->relative_position = leaf_position;
			leaf->relative_heading = leaf_heading;
			leaf->relative_scale = leaf_scale;
		}

		inst->vp.position = vp_position_saved;
		memcpy (inst->vp.attitude, vp_attitude_saved, sizeof (vp_attitude_saved));

		/* READ BACK: the touched viewpoint and (when it was changed) the leaf hold exactly what they held before */
		restored = (memcmp (&inst->vp.position, &vp_position_saved, sizeof (vp_position_saved)) == 0) && (memcmp (inst->vp.attitude, vp_attitude_saved, sizeof (vp_attitude_saved)) == 0);

		if (leaf)
		{
			restored = restored && (float_bits (leaf->relative_position.x) == float_bits (leaf_position.x)) && (float_bits (leaf->relative_position.y) == float_bits (leaf_position.y)) &&
				(float_bits (leaf->relative_position.z) == float_bits (leaf_position.z)) && (float_bits (leaf->relative_heading) == float_bits (leaf_heading)) &&
				(float_bits (leaf->relative_scale.x) == float_bits (leaf_scale.x)) && (float_bits (leaf->relative_scale.y) == float_bits (leaf_scale.y)) &&
				(float_bits (leaf->relative_scale.z) == float_bits (leaf_scale.z));
		}
	}
	else if (what == 1)
	{
		*field = saved_instance;
		restored = (*field == saved_instance);
	}
	else if (what == 3)
	{
		set_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE, shape_saved);
		restored = ((int) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE) == shape_saved);
	}

	tp_bits3 (base_text, sizeof (base_text), &base);
	tp_bits3 (result_text, sizeof (result_text), &result);

	if ((what == 2) && (scene > OBJECT_3D_INVALID_OBJECT_INDEX) && (scene < OBJECT_3D_LAST))
	{
		tp_bits3 (cache_text, sizeof (cache_text), &target_point_offsets[scene]);
	}
	else
	{
		snprintf (cache_text, sizeof (cache_text), "-");
	}

	n = snprintf (text + *used, (size_t) (capacity - *used),
		"model-host-tp case=%s status=called type=%d scene=%d base=%s heading=0x%08x cache=%s result=%s chain=%ld world=%ld fixed=%ld vp_written=%d levels=%d legacy_same=%d restored=%d mutation=%s pose=%s\n", name,
		(int) en->type, (what == 3) ? shape_saved : scene, base_text, heading_bits, cache_text, result_text, campaign_model_counts.target_chain - chain0,
		campaign_model_counts.target_world - world0, campaign_model_counts.target_fixed - fixed0, vp_written, levels, legacy_same, restored, mutation, pose_text);

	if ((n < 0) || (n >= capacity - *used))
	{
		return 1;
	}

	*used += n;
	return 0;
}

int eech_model_host_target_queries_run (int mutate, char *text, int capacity)
{
	entity *en;
	entity *vehicle_a = NULL, *vehicle_b = NULL, *fixed_horizontal = NULL, *fixed_flat = NULL;
	int visited = 0, used = 0, n;

	for (en = get_local_entity_list (); en;)
	{
		int type;
		object_3d_instance **field;

		if ((visited >= number_of_entities) || !host_heap_slot (en))
		{
			n = snprintf (text, (size_t) capacity, "model-host-tp case=scan status=unavailable reason=entity_list_not_trusted visited=%d\n", visited);
			return ((n < 0) || (n >= capacity)) ? 1 : 0;
		}

		visited++;
		type = (int) en->type;

		if (get_local_entity_data (en))
		{
			if (tp_vehicle_instance_field (en, &field))
			{
				if (*field)
				{
					if (!vehicle_a) vehicle_a = en;
					else if (!vehicle_b) vehicle_b = en;
				}
			}
			else if (campaign_object_is_structure (type) && get_local_entity_int_value (en, INT_TYPE_IDENTIFY_FIXED) && !get_local_entity_ptr_value (en, PTR_TYPE_INSTANCE_3D_OBJECT))
			{
				int shape = (int) get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);

				if ((shape > OBJECT_3D_INVALID_OBJECT_INDEX) && (shape < OBJECT_3D_LAST))
				{
					int horizontal = ((float_bits (target_point_offsets[shape].x) & 0x7fffffffu) != 0) || ((float_bits (target_point_offsets[shape].z) & 0x7fffffffu) != 0);

					if (horizontal && !fixed_horizontal) fixed_horizontal = en;
					else if (!horizontal && !fixed_flat) fixed_flat = en;
				}
			}
		}

		en = get_local_entity_succ (en);
	}

	used = 0;
	text[0] = '\0';

#define TP_CASE(NAME, ENTITY, WHAT, REASON) \
	if (ENTITY) \
	{ \
		if (tp_row (text, capacity, &used, NAME, ENTITY, mutate, WHAT)) return 1; \
	} \
	else \
	{ \
		n = snprintf (text + used, (size_t) (capacity - used), "model-host-tp case=%s status=unavailable reason=%s visited=%d\n", NAME, REASON, visited); \
		if ((n < 0) || (n >= capacity - used)) return 1; \
		used += n; \
	}

	TP_CASE ("instance_a", vehicle_a, 0, "no_vehicle_with_an_instance")
	TP_CASE ("instance_b", vehicle_b, 0, "no_second_vehicle_with_an_instance")
	TP_CASE ("instance_none", vehicle_a, 1, "no_vehicle_with_an_instance")
	TP_CASE ("fixed_horizontal", fixed_horizontal, 2, "no_fixed_structure_with_a_horizontal_offset")
	TP_CASE ("fixed_flat", fixed_flat, 2, "no_fixed_structure_without_a_horizontal_offset")
	TP_CASE ("fixed_invalid", fixed_flat ? fixed_flat : fixed_horizontal, 3, "no_fixed_structure")
#undef TP_CASE

	return 0;
}

int eech_model_host_poison_run (void)
{
	if (!objects_3d_scene_database)
	{
		return 1;
	}

	campaign_model_poison_arrays ();

	return 0;
}

int eech_model_host_probe_run (int scene, int depth, char *text, int capacity)
{
	object_3d_index_numbers index;
	const struct OBJECT_3D_BOUNDS *bb, *bb2, *bb_again, *bb2_again;
	float radius, radius_again;
	vec3d buffer[3];
	uint32_t words[9];
	char bb_text[160], bb2_text[160];
	int i, writes = 0, returned, round_before, round_after, used = 0, n;

	if ((scene < 1) || (scene >= OBJECT_3D_LAST) || (depth < 0))
	{
		return 1;
	}

	index = (object_3d_index_numbers) scene;

	/* the bounds and radius, twice: a caller keeps the bounds pointer across loops, so a second ask must give the same address and the same bits */
	bb = get_object_3d_bounding_box (index);
	bb2 = get_object_3d_bounding_box_without_lines (index);
	radius = get_object_3d_radius (index);
	bb_again = get_object_3d_bounding_box (index);
	bb2_again = get_object_3d_bounding_box_without_lines (index);
	radius_again = get_object_3d_radius (index);
	bounds_text (bb_text, sizeof (bb_text), bb);
	bounds_text (bb2_text, sizeof (bb2_text), bb2);

	n = snprintf (text + used, (size_t) (capacity - used), "model-host-scene scene=%d radius=0x%08x bb2=%s bb=%s stable=%d\n", scene, float_bits (radius), bb2_text, bb_text,
		(bb == bb_again) && (bb2 == bb2_again) && (float_bits (radius) == float_bits (radius_again)) && !memcmp (bb, bb_again, sizeof (*bb)) && !memcmp (bb2, bb2_again, sizeof (*bb2)));
	if ((n < 0) || (n >= capacity - used))
	{
		return 1;
	}
	used += n;

	/* the triangle step, into a buffer pre-filled with the original fixture's quiet-NaN sentinel */
	for (i = 0; i < 9; i++)
	{
		words[i] = 0x7fc0a5a5u;
	}
	memcpy (buffer, words, sizeof (buffer));
	round_before = fegetround ();
	returned = get_object_3d_collision_object_geometry_triangle (index, buffer, depth);
	round_after = fegetround ();
	memcpy (words, buffer, sizeof (words));
	for (i = 0; i < 9; i++)
	{
		writes += words[i] != 0x7fc0a5a5u;
	}
	n = snprintf (text + used, (size_t) (capacity - used), "model-host-tri scene=%d depth=%d return=%d writes=%d round_before=%d round_after=%d", scene, depth, returned, writes,
		round_before, round_after);
	if ((n < 0) || (n >= capacity - used))
	{
		return 1;
	}
	used += n;
	if (writes)
	{
		n = snprintf (text + used, (size_t) (capacity - used), " p=");
		if ((n < 0) || (n >= capacity - used))
		{
			return 1;
		}
		used += n;
		for (i = 0; i < 9; i++)
		{
			n = snprintf (text + used, (size_t) (capacity - used), "%s0x%08x", i ? "," : "", words[i]);
			if ((n < 0) || (n >= capacity - used))
			{
				return 1;
			}
			used += n;
		}
	}
	n = snprintf (text + used, (size_t) (capacity - used), "\n");
	if ((n < 0) || (n >= capacity - used))
	{
		return 1;
	}
	used += n;

	/* the route cursor, node and line face, of the airfield landing route of the scene at the same depth: the same callback and Lua provider the campaign's parser uses, under the
	   same entry, so the outer floating-point harness measures the route questions with the triangle ones. A scene with no such route answers `absent` (result 3). */
	{
		extern int campaign_model_route_probe (int scene, int kind, int cursor, int *node_result, float *node, int *face_result, int *face);
		int node_result = 0, face_result = 0, face[6];
		float node[3];

		round_before = fegetround ();
		campaign_model_route_probe (scene, OBJECT_3D_SUB_OBJECT_FIXED_WING_LANDING_ROUTE, depth, &node_result, node, &face_result, face);
		round_after = fegetround ();
		n = snprintf (text + used, (size_t) (capacity - used),
			"model-host-route scene=%d kind=%d cursor=%d node_result=%d node=0x%08x,0x%08x,0x%08x face_result=%d face=%d,%d,%d,%d,%d,%d round_before=%d round_after=%d\n", scene,
			(int) OBJECT_3D_SUB_OBJECT_FIXED_WING_LANDING_ROUTE, depth, node_result, float_bits (node[0]), float_bits (node[1]), float_bits (node[2]), face_result, face[0], face[1],
			face[2], face[3], face[4], face[5], round_before, round_after);
		return ((n < 0) || (n >= capacity - used)) ? 1 : 0;
	}
}


/* ---------------------------------------------------------------------------------------------------------------------------- */
/* The COMPLETE outer floating-point state around a PUBLIC engine entry (EEDCS-50, D3).
 *
 * eech_engine_model_host_fpu is an OUTER harness, not an engine entry: it is the caller. It chooses the caller's floating-point state, calls a real public entry, and reads the
 * state back the moment the entry returns, so what it reports is what a caller of the shipped entries sees. The state is the raw words, not a summary:
 *
 *   x87 control word   precision, rounding and the six exception masks
 *   x87 status word    the six exception FLAGS, the stack top, the condition codes
 *   MXCSR              the SSE exception flags, the six masks, the rounding, flush-to-zero and denormals-are-zero
 *
 * Before the entry the caller state is set to: the requested rounding mode in BOTH units (0 nearest, 1 down, 2 up, 3 toward zero), every exception masked, and NONZERO
 * exception flags (divide-by-zero, underflow, inexact) in both the x87 status word and MXCSR, so that an entry that clears or overwrites flags is seen. The entries are the real ones:
 *
 *   kind 0   eech_engine_model_host_probe: the engine entry (ENTER/LEAVE), the campaign's public accessors, the host's model callback, the Lua provider and its bridge, the
 *            parsing and narrowing in Rust and back. A host that cannot answer takes the entry's fatal path (setjmp), which is the second path a caller must see restored.
 *   kind 1   eech_engine_model_host_counters: the engine entry (ENTER/LEAVE) alone, with no callback.
 *
 * The report gives the words before and after and whether each is identical. It does NOT decide what is acceptable and it repairs nothing: a difference is a measurement of the
 * shipped ENTER/LEAVE macros (which restore with fesetenv; whether that restores the SSE word on a given toolchain is exactly what this measures). Afterwards the harness puts the
 * process back into the environment it found, so the diagnostic itself leaves no mark. Rounding modes alone (fegetround) are NOT a proof of the complete state, and this report
 * does not rest on them.
 */
#if defined(__x86_64__) || defined(__i386__)

struct fpu_words
{
	uint32_t x87_control, x87_status, mxcsr;
};

static void fpu_capture (struct fpu_words *w)
{
	uint16_t control = 0, status = 0;
	uint32_t mxcsr = 0;

	__asm__ __volatile__ ("fnstcw %0" : "=m" (control));
	__asm__ __volatile__ ("fnstsw %0" : "=m" (status));
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));
	w->x87_control = control;
	w->x87_status = status;
	w->mxcsr = mxcsr;
}

/* the rounding mode in both units, all exceptions masked, and the divide-by-zero (0x04), underflow (0x10) and inexact (0x20) flags raised in both */
static void fpu_prepare (int rounding)
{
	uint16_t control = 0;
	uint32_t mxcsr = 0;
	unsigned char environment[28];
	uint16_t status;

	__asm__ __volatile__ ("fnstcw %0" : "=m" (control));
	control = (uint16_t) ((control & ~0x0C00u) | ((unsigned) rounding << 10) | 0x3Fu);
	__asm__ __volatile__ ("fldcw %0" : : "m" (control));

	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));
	mxcsr = (mxcsr & ~0x6000u) | ((unsigned) rounding << 13) | 0x1F80u | 0x34u;
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (mxcsr));

	/* the x87 flags are written through the 28-byte protected-mode environment image (status word at offset 4) and loaded back; nothing is computed */
	__asm__ __volatile__ ("fnstenv %0" : "=m" (environment));
	memcpy (&status, environment + 4, sizeof (status));
	status = (uint16_t) (status | 0x34u);
	memcpy (environment + 4, &status, sizeof (status));
	__asm__ __volatile__ ("fldenv %0" : : "m" (environment));
}

static void fpu_write_mxcsr (uint32_t mxcsr)
{
	__asm__ __volatile__ ("ldmxcsr %0" : : "m" (mxcsr));
}

int eech_engine_model_host_fpu (int kind, int scene, int depth, int rounding, char *text, int capacity)
{
	struct fpu_words before, after;
	fenv_t original;
	uint32_t original_mxcsr = 0;
	char scratch[2048];
	int result, n, used;

	if (!text || (capacity <= 0) || (kind < 0) || (kind > 1) || (rounding < 0) || (rounding > 3) || (scene < 1) || (depth < 0))
	{
		return EECH_ENGINE_BAD_ARGUMENT;
	}

	/* the process's own environment, to be handed back after the diagnostic */
	fegetenv (&original);
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (original_mxcsr));

	fpu_prepare (rounding);
	fpu_capture (&before);
	if (kind == 0)
	{
		result = eech_engine_model_host_probe (scene, depth, scratch, (int) sizeof (scratch));
	}
	else
	{
		result = eech_engine_model_host_counters (scratch, (int) sizeof (scratch));
	}
	fpu_capture (&after);

	fesetenv (&original);
	fpu_write_mxcsr (original_mxcsr);

	n = snprintf (text, (size_t) capacity,
		"model-host-fpu kind=%d rounding=%d scene=%d depth=%d result=%d before_x87_control=0x%04x before_x87_status=0x%04x before_mxcsr=0x%08x"
		" after_x87_control=0x%04x after_x87_status=0x%04x after_mxcsr=0x%08x same_x87_control=%d same_x87_status=%d same_mxcsr=%d\n",
		kind, rounding, scene, depth, result, before.x87_control, before.x87_status, before.mxcsr, after.x87_control, after.x87_status, after.mxcsr,
		before.x87_control == after.x87_control, before.x87_status == after.x87_status, before.mxcsr == after.mxcsr);
	if ((n < 0) || (n >= capacity))
	{
		return EECH_ENGINE_BAD_ARGUMENT;
	}
	used = n;
	if (result == EECH_ENGINE_FATAL)
	{
		/* the engine's stop, first line only: the host's refusal that took the entry's fatal path */
		const char *message = eech_engine_fatal_message ();
		char line[200];
		size_t length = 0;

		while (message && message[length] && (message[length] != '\n') && (length < sizeof (line) - 1))
		{
			line[length] = message[length];
			length++;
		}
		line[length] = 0;
		n = snprintf (text + used, (size_t) (capacity - used), "model-host-fpu-fatal %s\n", line);
		if ((n < 0) || (n >= capacity - used))
		{
			return EECH_ENGINE_BAD_ARGUMENT;
		}
	}

	return EECH_ENGINE_OK;
}

#else

int eech_engine_model_host_fpu (int kind, int scene, int depth, int rounding, char *text, int capacity)
{
	(void) kind;
	(void) scene;
	(void) depth;
	(void) rounding;
	(void) text;
	(void) capacity;

	return EECH_ENGINE_BAD_ARGUMENT;
}

#endif

#endif

#endif
