/*
 * TEST-ONLY original collision-model characterisation (EEDCS-50). Compiled only with the `model-fixture` feature; never part of the
 * shipping module.
 *
 * It asks the ORIGINAL compiled accessors of modules/3d about the models the retail data loaded, and reports what they answer:
 *
 *   get_object_3d_collision_object_geometry_triangle   the collision triangles, one call per depth, ascending, one buffer each
 *   get_object_3d_bounding_box_without_lines           the bounds the collision tests use
 *   get_object_3d_bounding_box, get_object_3d_radius   the other scene accessors the entity queries read
 *
 * The answers are the original's. Nothing here computes a triangle, a bound or a count the original did not return. Floats are
 * reported as their bit patterns (0x%08x), so no decimal round trip can lose anything.
 *
 * What this file DOES compute is a read-only SAFETY verdict: whether the original decoder can be called at all, and to which depth,
 * without reading memory the loader did not back. The decoder is not defensive. It reads obj->surfaces[0] before it looks at the
 * face count (a NULL surface list is a crash, the cause of the C1 patch for object index 0); at the end of the walk it reads
 * obj->surfaces[number_of_surfaces], ONE SURFACE PAST the object's own; and it trusts every face/point index. The verdict replays
 * the decoder's integer path exactly (csrc/eech_model_walk.h) over the BACKING the real loader reported, before any call, and
 * reports why it stops if it does. It is not an expected answer: a model the verdict refuses is reported refused (with its
 * metadata), never passed and never "fixed", and the verdict never computes a triangle.
 *
 * The backing is OBSERVED, not inferred (no next-pointer guess):
 *   - the shared face and point-list blocks: the loader's own allocation sizes, the cursor it advanced through them, and how many
 *     items each fread actually returned (a short read leaves a hole: the bytes after it are allocated but not initialised);
 *     reported by the feature-only loader hooks (csrc/eech_model_hooks.h, installed by build/patches.rs) before any override file
 *     mutates an object;
 *   - the two file-mapped shared arrays (points, surface points): the exact length of the live view whose base the loader holds
 *     (eech_view_extent in eech_win32.c);
 *   - the consolidated surface block: one allocation of exactly the sum of every object's surfaces;
 *   - each custom (override) object: the exact allocation identities and filled extents read_object made, matched to the object's
 *     CURRENT pointers, one slot per object index (bounded by MF_OBJECT_MAX, well above the object table of the supported data); a repeat, a stale
 *     pointer, an object that is not an element of the table or an index past the bound is a refusal of that object alone, never a guess.
 * An array whose backing was not observed is refused as unobservable; a read inside an allocation but past the initialised bytes
 * is refused as uninitialised; a read outside is refused as unbacked. A mixed-layout model (number_of_faces counting faces the
 * object's own arrays do not hold) is walked into the following objects' shared data when, and only when, those reads are backed.
 *
 * Pure observer: the entries below write no table, scene, object, pointer or model byte, draw no random number, make no
 * physical query beyond the accessors named, and touch no entity (the use count only reads each entity's shape through the
 * ordinary getter, whose default for a type without a shape is the invalid index). The loader hooks only record. `eech_model_state_run`
 * hashes every table these entries read, and the observer's own records, so a driver can show before and after that nothing moved.
 */

#ifdef EECH_MODEL_FIXTURE

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"
#include "eech_model_hooks.h"
#include "eech_model_walk.h"

#define PAGE_SCENES 96
#define PAGE_TRIANGLES 256
#define USE_TYPE_SLOTS 4

/* One record slot per OBJECT INDEX, so a record is found in constant time and no override is ever skipped for arriving late. The loader sizes the object table at
   (objects in 3dobjs.bin) + custom_number_of_objects (2000) + 1 entries; the supported retail data has 36 hex-named override files and 811 named objects (847 custom loads)
   with named ids reaching 10252, so the table is a little over twelve thousand entries. 32768 is a fixed bound with better than twofold margin over that (the slots are static,
   not a model allocation). An object whose index does not fit is refused on its own (`record_capacity`, counted); no other record is affected. */
#define MF_OBJECT_MAX 32768
/* the most elementary steps one verdict may take */
#define MF_STEP_BUDGET 4000000ull

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

typedef char vec3d_is_three_floats[(sizeof (vec3d) == 3 * sizeof (float)) ? 1 : -1];

/* the element sizes read_object and the walk both rely on */
typedef char point_is_six_bytes[(sizeof (object_short_3d_point) == 6) ? 1 : -1];
typedef char face_is_one_byte[(sizeof (object_3d_face) == 1) ? 1 : -1];
typedef char point_list_entry_is_one_byte[(sizeof (point_3d_short_reference) == 1) ? 1 : -1];
typedef char surface_point_is_two_bytes[(sizeof (point_3d_plain_reference) == 2) ? 1 : -1];
typedef char surface_is_sixteen_bytes[(sizeof (face_surface_description) == MW_SURFACE_BYTES) ? 1 : -1];

/* defined in eech_win32.c under this feature: the length of the live view whose base is exactly the pointer */
extern int eech_view_extent (const void *base, unsigned long long *size);

/* ---------------------------------------------------------------------------------------------------------------------------- */

struct out
{
	char *text;
	int capacity, used, overflow;
};

static void emit (struct out *out, const char *format, ...) __attribute__ ((format (printf, 2, 3)));

static void emit (struct out *out, const char *format, ...)
{
	va_list args;
	int n;
	if (out->overflow)
	{
		return;
	}
	va_start (args, format);
	n = vsnprintf (out->text + out->used, (size_t) (out->capacity - out->used), format, args);
	va_end (args);
	if (n < 0 || n >= out->capacity - out->used)
	{
		out->overflow = 1;
		return;
	}
	out->used += n;
}

static void out_begin (struct out *out, char *text, int capacity)
{
	out->text = text;
	out->capacity = capacity;
	out->used = 0;
	out->overflow = 0;
	text[0] = 0;
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static void bounds_text (char *buffer, size_t capacity, const struct OBJECT_3D_BOUNDS *b)
{
	snprintf (buffer, capacity, "0x%08x,0x%08x,0x%08x,0x%08x,0x%08x,0x%08x", float_bits (b->xmin), float_bits (b->xmax), float_bits (b->ymin),
		float_bits (b->ymax), float_bits (b->zmin), float_bits (b->zmax));
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the loader's backing, as the hooks report it (single-threaded: the loader runs on the engine thread that also serves these entries) */

enum { MF_BAD_NONE = 0, MF_BAD_ORDER, MF_BAD_RANGE, MF_BAD_BASE, MF_BAD_CURSOR };

static const char *const mf_bad_names[] = { "-", "order", "range", "base", "cursor" };

struct mf_common
{
	int phase; /* 0: nothing observed since the last reset; 1: the object loop is being read; 2: the loop is over (the backing is usable) */
	int bad;   /* the first inconsistency between what the hooks reported and what the loader's own bookkeeping must satisfy (0: none) */
	uintptr_t faces_lo, list_lo;                  /* the shared blocks' bases, as the first fread reported them */
	uint64_t faces_allocated, list_allocated;     /* bytes the loader allocated */
	uint64_t faces_requested, list_requested;     /* items the loop asked fread for, summed */
	uint64_t faces_initialised, list_initialised; /* leading bytes fread filled (up to the first short read) */
	int faces_short, list_short;                  /* a fread returned fewer items than asked: nothing after it is initialised */
	uintptr_t points_lo, sp_lo;                   /* the file-mapped arrays' bases */
	uint64_t points_consumed, sp_consumed;        /* bytes the loader's cursors advanced through them */
};

static struct mf_common mf_common;
static unsigned mf_generation;

enum { MF_RECORD_OK = 1, MF_RECORD_AMBIGUOUS = 2 };

struct mf_custom
{
	unsigned generation; /* the load this record belongs to; a record of any other generation does not exist */
	const void *object;
	int state;
	uintptr_t points, faces, list, sp;                           /* the pointers read_object stored */
	uint64_t points_allocated, faces_allocated, list_allocated, sp_allocated; /* bytes allocated */
	uint64_t points_initialised, faces_initialised, list_initialised, sp_initialised; /* bytes fread filled */
};

/* indexed by object index (slot 0 is the null object and is never used) */
static struct mf_custom mf_custom_table[MF_OBJECT_MAX];
/* this load's records: usable, ambiguous (repeat or inconsistent), refused for index capacity, not an element of the object table, and read outside the observed window */
static int mf_custom_recorded, mf_custom_ambiguous, mf_custom_dropped, mf_custom_unknown, mf_custom_late;
static uint64_t mf_pending[4];
static int mf_pending_valid[4];

void eech_mf_reset (void)
{
	memset (&mf_common, 0, sizeof (mf_common));
	memset (mf_pending, 0, sizeof (mf_pending));
	memset (mf_pending_valid, 0, sizeof (mf_pending_valid));
	mf_custom_recorded = mf_custom_ambiguous = mf_custom_dropped = mf_custom_unknown = mf_custom_late = 0;
	/* the table is not cleared: a record is only believed in the generation that made it */
	mf_generation++;
}

void eech_mf_common_read (int which, const void *base, const void *cursor, long long requested, long long returned)
{
	struct mf_common *m = &mf_common;
	uintptr_t b = (uintptr_t) base, c = (uintptr_t) cursor;
	uintptr_t *lo;
	uint64_t *total, *initialised, elem;
	int *shorted;

	if (m->bad)
	{
		return;
	}
	if (m->phase == 0)
	{
		m->phase = 1;
	}
	else if (m->phase != 1)
	{
		m->bad = MF_BAD_ORDER;
		return;
	}
	if (which == 0)
	{
		lo = &m->faces_lo;
		total = &m->faces_requested;
		initialised = &m->faces_initialised;
		shorted = &m->faces_short;
		elem = sizeof (object_3d_face);
	}
	else if (which == 1)
	{
		lo = &m->list_lo;
		total = &m->list_requested;
		initialised = &m->list_initialised;
		shorted = &m->list_short;
		elem = sizeof (point_3d_short_reference);
	}
	else
	{
		m->bad = MF_BAD_RANGE;
		return;
	}
	if (!b || requested < 0 || returned < 0 || returned > requested)
	{
		m->bad = MF_BAD_RANGE;
		return;
	}
	if (!*lo)
	{
		*lo = b;
	}
	else if (*lo != b)
	{
		m->bad = MF_BAD_BASE;
		return;
	}
	/* the loop's own cursor is the block base plus everything asked for so far */
	if (c < b || (uint64_t) (c - b) != *total * elem)
	{
		m->bad = MF_BAD_CURSOR;
		return;
	}
	if (!*shorted)
	{
		if (returned == requested)
		{
			*initialised = (uint64_t) (c - b) + (uint64_t) requested * elem;
		}
		else
		{
			*shorted = 1;
			*initialised = (uint64_t) (c - b) + (uint64_t) returned * elem;
		}
	}
	*total += (uint64_t) requested;
}

/* 0, or why the loader's reported cursor and allocation are not consistent with what the hooks saw */
static int mf_block_check (uintptr_t seen, const void *base, const void *cursor, long long allocated, uint64_t requested, uint64_t elem)
{
	uintptr_t b = (uintptr_t) base, c = (uintptr_t) cursor;

	if (allocated < 0 || requested > (uint64_t) allocated)
	{
		return MF_BAD_RANGE;
	}
	if (!b)
	{
		return (allocated == 0 && requested == 0 && !c) ? MF_BAD_NONE : MF_BAD_BASE;
	}
	if (seen && seen != b)
	{
		return MF_BAD_BASE;
	}
	if (c < b || (uint64_t) (c - b) != requested * elem)
	{
		return MF_BAD_CURSOR;
	}
	return MF_BAD_NONE;
}

void eech_mf_common_done (const void *faces, long long faces_allocated, const void *faces_cursor, const void *list, long long list_allocated,
	const void *list_cursor, const void *points, const void *points_cursor, const void *surface_points, const void *surface_points_cursor)
{
	struct mf_common *m = &mf_common;
	int why;

	if (m->phase == 2)
	{
		m->bad = MF_BAD_ORDER;
		return;
	}
	m->phase = 2;
	if (m->bad)
	{
		return;
	}
	why = mf_block_check (m->faces_lo, faces, faces_cursor, faces_allocated, m->faces_requested, sizeof (object_3d_face));
	if (!why)
	{
		why = mf_block_check (m->list_lo, list, list_cursor, list_allocated, m->list_requested, sizeof (point_3d_short_reference));
	}
	if (!why && (!points || !surface_points || (uintptr_t) points_cursor < (uintptr_t) points || (uintptr_t) surface_points_cursor < (uintptr_t) surface_points))
	{
		why = MF_BAD_CURSOR;
	}
	if (why)
	{
		m->bad = why;
		return;
	}
	m->faces_lo = (uintptr_t) faces;
	m->list_lo = (uintptr_t) list;
	m->faces_allocated = (uint64_t) faces_allocated * sizeof (object_3d_face);
	m->list_allocated = (uint64_t) list_allocated * sizeof (point_3d_short_reference);
	m->points_lo = (uintptr_t) points;
	m->sp_lo = (uintptr_t) surface_points;
	m->points_consumed = (uint64_t) ((uintptr_t) points_cursor - (uintptr_t) points);
	m->sp_consumed = (uint64_t) ((uintptr_t) surface_points_cursor - (uintptr_t) surface_points);
}

size_t eech_mf_filled (int which, size_t returned)
{
	if (which >= 0 && which < 4)
	{
		mf_pending[which] = (uint64_t) returned;
		mf_pending_valid[which] = 1;
	}
	return returned;
}

enum { MF_SLOT_OK = 0, MF_SLOT_UNKNOWN, MF_SLOT_CAPACITY };

/* the slot of an object: its index in the loader's object table, computed on integers (the pointer must be exactly an element 1 .. raw of that table) */
static struct mf_custom *mf_custom_slot (const void *object, int *why)
{
	uintptr_t base = (uintptr_t) objects_3d_data, o = (uintptr_t) object;
	uint64_t index;

	*why = MF_SLOT_UNKNOWN;
	if (!base || o < base || total_number_of_raw_3d_objects < 1 || ((uint64_t) (o - base)) % sizeof (object_3d))
	{
		return NULL;
	}
	index = ((uint64_t) (o - base)) / sizeof (object_3d);
	if (index < 1 || index > (uint64_t) total_number_of_raw_3d_objects)
	{
		return NULL;
	}
	if (index >= MF_OBJECT_MAX)
	{
		*why = MF_SLOT_CAPACITY;
		return NULL;
	}
	*why = MF_SLOT_OK;
	return &mf_custom_table[index];
}

/* the record of an object made in this generation, or NULL */
static struct mf_custom *mf_custom_find (const void *object)
{
	int why;
	struct mf_custom *r = mf_custom_slot (object, &why);
	if (!r || r->generation != mf_generation || r->state == 0 || r->object != object)
	{
		return NULL;
	}
	return r;
}

void eech_mf_custom (const void *object, long long polygoned_faces, long long point_references, long long surface_point_references)
{
	const object_3d *obj = (const object_3d *) object;
	struct mf_custom *r;
	long long counts[4];
	uint64_t sizes[4];
	const void *pointers[4];
	uint64_t *allocated[4], *initialised[4];
	int i, ok = 1, why;

	if (mf_common.phase != 2)
	{
		/* read_object outside the window the common backing was observed in: nothing can be matched, the object stays unrecorded */
		mf_custom_late++;
		memset (mf_pending_valid, 0, sizeof (mf_pending_valid));
		return;
	}
	r = mf_custom_slot (object, &why);
	if (!r)
	{
		/* not an element of the object table (unknown), or an index past the fixed bound (capacity): that object alone stays unrecorded and is refused as such */
		if (why == MF_SLOT_CAPACITY)
		{
			mf_custom_dropped++;
		}
		else
		{
			mf_custom_unknown++;
		}
		memset (mf_pending_valid, 0, sizeof (mf_pending_valid));
		return;
	}
	if (r->generation == mf_generation && r->state != 0)
	{
		/* a second read_object into the same object: the first allocations are no longer the object's, and the later ones may not be either */
		if (r->state == MF_RECORD_OK)
		{
			mf_custom_recorded--;
			mf_custom_ambiguous++;
		}
		r->state = MF_RECORD_AMBIGUOUS;
		memset (mf_pending_valid, 0, sizeof (mf_pending_valid));
		return;
	}
	memset (r, 0, sizeof (*r));
	r->generation = mf_generation;
	r->object = object;
	r->state = MF_RECORD_OK;

	counts[0] = obj->number_of_points;
	counts[1] = polygoned_faces;
	counts[2] = point_references;
	counts[3] = surface_point_references;
	sizes[0] = sizeof (object_short_3d_point);
	sizes[1] = sizeof (object_3d_face);
	sizes[2] = sizeof (point_3d_short_reference);
	sizes[3] = sizeof (point_3d_plain_reference);
	pointers[0] = obj->points;
	pointers[1] = obj->faces;
	pointers[2] = obj->object_faces_point_plain_list;
	pointers[3] = obj->surface_points;
	allocated[0] = &r->points_allocated;
	allocated[1] = &r->faces_allocated;
	allocated[2] = &r->list_allocated;
	allocated[3] = &r->sp_allocated;
	initialised[0] = &r->points_initialised;
	initialised[1] = &r->faces_initialised;
	initialised[2] = &r->list_initialised;
	initialised[3] = &r->sp_initialised;
	r->points = (uintptr_t) pointers[0];
	r->faces = (uintptr_t) pointers[1];
	r->list = (uintptr_t) pointers[2];
	r->sp = (uintptr_t) pointers[3];

	for (i = 0; i < 4; i++)
	{
		if (counts[i] < 0)
		{
			ok = 0;
		}
		else if (counts[i] == 0)
		{
			/* read_object stores NULL and reads nothing */
			if (pointers[i] || mf_pending_valid[i])
			{
				ok = 0;
			}
		}
		else if (!pointers[i] || !mf_pending_valid[i] || mf_pending[i] > (uint64_t) counts[i])
		{
			ok = 0;
		}
		else
		{
			*allocated[i] = (uint64_t) counts[i] * sizes[i];
			*initialised[i] = mf_pending[i] * sizes[i];
		}
	}
	if (!ok)
	{
		r->state = MF_RECORD_AMBIGUOUS;
		mf_custom_ambiguous++;
	}
	else
	{
		mf_custom_recorded++;
	}
	memset (mf_pending_valid, 0, sizeof (mf_pending_valid));
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the loaded tables and the regions the verdict reads against */

static const char *scene_name (int scene)
{
	const char *name = object_3d_scene_names[scene];
	return name ? name : "-";
}

static int scene_in_range (int scene)
{
	return scene > OBJECT_3D_INVALID_OBJECT_INDEX && scene < OBJECT_3D_LAST;
}

/* the one reallocated surface block: its length is the sum of every object's surfaces, as the loader sized it */
static long surface_block_length (void)
{
	long sum = 0;
	int i;
	for (i = 1; i <= total_number_of_raw_3d_objects; i++)
	{
		sum += objects_3d_data[i].number_of_surfaces;
	}
	return sum;
}

/* a whole live file view (every byte is readable: the view is a zero-filled private copy of the file) */
static void region_view (struct mw_region *r, uintptr_t base)
{
	unsigned long long size = 0;

	memset (r, 0, sizeof (*r));
	if (base && eech_view_extent ((const void *) base, &size))
	{
		r->known = 1;
		r->lo = base;
		r->allocated = size;
		r->initialised = size;
	}
}

static void region_common (struct mw_region *r, uintptr_t lo, uint64_t allocated, uint64_t initialised)
{
	memset (r, 0, sizeof (*r));
	if (mf_common.phase == 2 && !mf_common.bad)
	{
		r->known = 1;
		r->lo = lo;
		r->allocated = allocated;
		r->initialised = initialised;
	}
}

static void region_custom (struct mw_region *r, uintptr_t lo, uint64_t allocated, uint64_t initialised)
{
	r->known = 1;
	r->lo = lo;
	r->allocated = allocated;
	r->initialised = initialised;
}

/* the usable record of a custom object, or NULL and why */
static const struct mf_custom *custom_record (const object_3d *o, const char **why)
{
	const struct mf_custom *r;

	if (mf_common.phase != 2 || mf_common.bad)
	{
		*why = "load_unobserved";
		return NULL;
	}
	{
		int slot_why;
		(void) mf_custom_slot (o, &slot_why);
		if (slot_why == MF_SLOT_CAPACITY)
		{
			/* this object's index does not fit the fixed bound: it alone is refused */
			*why = "record_capacity";
			return NULL;
		}
		if (slot_why == MF_SLOT_UNKNOWN)
		{
			*why = "unknown_object";
			return NULL;
		}
	}
	r = mf_custom_find (o);
	if (!r)
	{
		*why = "unrecorded";
		return NULL;
	}
	if (r->state != MF_RECORD_OK)
	{
		*why = "ambiguous_record";
		return NULL;
	}
	/* the object must still hold exactly the allocations the record names (a stale pointer is never trusted) */
	if (r->points != (uintptr_t) o->points || r->faces != (uintptr_t) o->faces || r->list != (uintptr_t) o->object_faces_point_plain_list ||
		r->sp != (uintptr_t) o->surface_points)
	{
		*why = "stale_record";
		return NULL;
	}
	return r;
}

/* elements from the object's own pointer to the end of its observed allocation; -1 when unobserved or the pointer is outside it */
static long cap_elements (const struct mw_region *r, uintptr_t ptr, uint64_t element)
{
	if (!r->known || !ptr || ptr < r->lo || (uint64_t) (ptr - r->lo) > r->allocated)
	{
		return -1;
	}
	return (long) ((r->allocated - (uint64_t) (ptr - r->lo)) / element);
}

struct analysis
{
	const char *status; /* safe, empty, or the refusal / absence class */
	const char *detail; /* which table the refusal concerns, or "-" */
	int collision_index;
	int number_of_points, number_of_faces, number_of_surfaces, custom;
	long surface_faces, polygon_surface_faces, nonpolygon_surfaces, zero_face_surfaces, surface_points_sum;
	long face_points_sum, fan_triangles; /* fan_triangles: the walk's safe depth count (the whole fan when the walk is complete) */
	int face_points_min, face_points_max;
	long cap_faces, cap_point_list, cap_surface_points;
	int end_read_backed;
	int callable;
	const char *backing; /* common, custom, or - */
	const char *walk;    /* complete, end_unbacked, partial, refused, or - */
	const char *stop;    /* why the walk stopped, or - */
	const char *stop_array;
	long faces_read, list_read, sp_read, points_read; /* elements of the object's own pointers the walk read, all of them backed */
};

static int analysis_probeable (const struct analysis *a)
{
	return a->callable;
}

/* the deepest depth the original may be asked: the end probe (depth == fan_triangles) only when the one-past surface read is backed */
static long analysis_max_depth (const struct analysis *a)
{
	return a->end_read_backed ? a->fan_triangles : a->fan_triangles - 1;
}

static const char *reason_name (int reason)
{
	switch (reason)
	{
		case MW_OK: return "-";
		case MW_NULL_SURFACES: return "null_surfaces";
		case MW_SURFACES_OUTSIDE: return "surfaces_outside";
		case MW_NULL_ARRAY: return "null_array";
		case MW_UNOBSERVED: return "unobserved";
		case MW_UNBACKED: return "unbacked";
		case MW_UNINITIALISED: return "uninitialised";
		case MW_MISALIGNED: return "misaligned";
		case MW_OVERFLOW: return "overflow";
		case MW_BUDGET: return "budget";
		default: return "unknown";
	}
}

/* the refusal class of a walk that leaves no callable depth */
static void refuse (struct analysis *a, const struct mw_result *w)
{
	a->detail = w->array;
	switch (w->reason)
	{
		case MW_NULL_SURFACES: a->status = "null_surfaces"; break;
		case MW_SURFACES_OUTSIDE: a->status = "surfaces_outside_block"; break;
		case MW_NULL_ARRAY: a->status = "null_arrays"; break;
		case MW_UNOBSERVED: a->status = "capacity_unobservable"; break;
		case MW_BUDGET:
			a->status = "capacity_unobservable";
			a->detail = "walk_budget";
			break;
		default: a->status = "capacity_short"; break;
	}
}

static void analyse (struct analysis *a, int scene)
{
	const object_3d *obj;
	struct mw_input in;
	struct mw_result w;
	const struct mf_custom *rec = NULL;
	const char *why = "-";
	long block_length;
	uintptr_t block_lo, block_hi, surfaces;
	int s;
	int c = objects_3d_scene_database[scene].collision_object_index;

	memset (a, 0, sizeof (*a));
	a->status = "safe";
	a->detail = "-";
	a->collision_index = c;
	a->face_points_min = -1;
	a->cap_faces = a->cap_point_list = a->cap_surface_points = -3;
	a->backing = "-";
	a->walk = "-";
	a->stop = "-";
	a->stop_array = "-";

	if (c == -1)
	{
		a->status = "absent";
		return;
	}
	if (c == 0 && objects_3d_scene_database[scene].index == 0)
	{
		a->status = "no_scene";
		return;
	}
	if (c < 1 || c > total_number_of_raw_3d_objects)
	{
		a->status = "invalid_index";
		return;
	}

	obj = &objects_3d_data[c];
	a->number_of_points = obj->number_of_points;
	a->number_of_faces = obj->number_of_faces;
	a->number_of_surfaces = obj->number_of_surfaces;
	a->custom = obj->custom;

	/* the surfaces: the decoder reads surfaces[0] unconditionally, from the consolidated block */
	block_length = surface_block_length ();
	if (!object_database_surfaces || obj->number_of_surfaces < 1 || !obj->surfaces)
	{
		a->status = "null_surfaces";
		return;
	}
	block_lo = (uintptr_t) object_database_surfaces;
	block_hi = block_lo + (uintptr_t) block_length * sizeof (face_surface_description);
	surfaces = (uintptr_t) obj->surfaces;
	if (surfaces < block_lo || surfaces >= block_hi || (uint64_t) (block_hi - surfaces) < (uint64_t) obj->number_of_surfaces * sizeof (face_surface_description))
	{
		a->status = "surfaces_outside_block";
		return;
	}

	for (s = 0; s < obj->number_of_surfaces; s++)
	{
		const face_surface_description *surface = &obj->surfaces[s];
		a->surface_faces += surface->number_of_faces;
		if (surface->polygons)
		{
			a->polygon_surface_faces += surface->number_of_faces;
		}
		else
		{
			a->nonpolygon_surfaces++;
		}
		if (surface->number_of_faces == 0)
		{
			a->zero_face_surfaces++;
		}
		a->surface_points_sum += surface->number_of_points ? surface->number_of_points : 256;
	}

	/* the backing of the four arrays the decoder walks */
	memset (&in, 0, sizeof (in));
	in.surfaces_ptr = surfaces;
	in.surfaces_lo = block_lo;
	in.surfaces_hi = block_hi;
	in.faces_ptr = (uintptr_t) obj->faces;
	in.list_ptr = (uintptr_t) obj->object_faces_point_plain_list;
	in.sp_ptr = (uintptr_t) obj->surface_points;
	in.points_ptr = (uintptr_t) obj->points;
	in.number_of_faces = obj->number_of_faces > 0 ? (uint64_t) obj->number_of_faces : 0;
	in.step_budget = MF_STEP_BUDGET;
	if (obj->custom)
	{
		rec = custom_record (obj, &why);
		if (!rec)
		{
			a->status = "custom_object";
			a->detail = why;
			return;
		}
		a->backing = "custom";
		region_custom (&in.faces, rec->faces, rec->faces_allocated, rec->faces_initialised);
		region_custom (&in.list, rec->list, rec->list_allocated, rec->list_initialised);
		region_custom (&in.sp, rec->sp, rec->sp_allocated, rec->sp_initialised);
		region_custom (&in.points, rec->points, rec->points_allocated, rec->points_initialised);
	}
	else
	{
		if (mf_custom_find (obj))
		{
			/* recorded as a custom object but no longer flagged: the record cannot be trusted either way */
			a->status = "custom_object";
			a->detail = "stale_record";
			return;
		}
		a->backing = "common";
		region_common (&in.faces, mf_common.faces_lo, mf_common.faces_allocated, mf_common.faces_initialised);
		region_common (&in.list, mf_common.list_lo, mf_common.list_allocated, mf_common.list_initialised);
		region_view (&in.sp, mf_common.sp_lo);
		region_view (&in.points, mf_common.points_lo);
	}
	a->cap_faces = cap_elements (&in.faces, in.faces_ptr, sizeof (object_3d_face));
	a->cap_point_list = cap_elements (&in.list, in.list_ptr, sizeof (point_3d_short_reference));
	a->cap_surface_points = cap_elements (&in.sp, in.sp_ptr, sizeof (point_3d_plain_reference));

	mw_walk (&in, &w);

	a->face_points_sum = (long) w.face_points_sum;
	a->face_points_min = w.face_points_min;
	a->face_points_max = w.face_points_max;
	a->fan_triangles = (long) w.safe_depths;
	a->end_read_backed = w.end_backed;
	a->stop = reason_name (w.reason);
	a->stop_array = w.array;
	a->faces_read = (long) w.faces_read;
	a->list_read = (long) w.list_read;
	a->sp_read = (long) w.sp_read;
	a->points_read = (long) w.points_read;
	a->callable = w.safe_depths > 0 || w.end_backed;
	if (a->callable)
	{
		a->status = obj->number_of_faces == 0 ? "empty" : "safe";
		a->walk = w.reason == MW_OK ? "complete" : (w.complete ? "end_unbacked" : "partial");
	}
	else
	{
		a->walk = "refused";
		refuse (a, &w);
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* summary */

int eech_model_summary_run (char *text, int capacity)
{
	struct out out;
	unsigned long long points_view = 0, sp_view = 0;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data)
	{
		return 1;
	}
	if (mf_common.points_lo)
	{
		(void) eech_view_extent ((const void *) mf_common.points_lo, &points_view);
	}
	if (mf_common.sp_lo)
	{
		(void) eech_view_extent ((const void *) mf_common.sp_lo, &sp_view);
	}
	emit (&out, "model-summary scene_last=%d raw_objects=%d surface_block=%ld scene_entry_bytes=%d object_bytes=%d surface_bytes=%d page_scenes=%d page_triangles=%d names=compiled_enum",
		(int) OBJECT_3D_LAST, total_number_of_raw_3d_objects, surface_block_length (), (int) sizeof (struct OBJECT_3D_SCENE_DATABASE_ENTRY), (int) sizeof (object_3d),
		(int) sizeof (face_surface_description), PAGE_SCENES, PAGE_TRIANGLES);
	/* the backing the loader hooks reported (optional fields; the closed driver tolerates them) */
	emit (&out,
		" backing=hooked generation=%u common_phase=%d common_bad=%s faces_allocated=%llu faces_consumed=%llu faces_initialised=%llu list_allocated=%llu list_consumed=%llu"
		" list_initialised=%llu points_view=%llu points_consumed=%llu sp_view=%llu sp_consumed=%llu custom_slots=%d custom_recorded=%d custom_ambiguous=%d custom_record_overflow=%d custom_unknown=%d custom_late=%d\n",
		mf_generation, mf_common.phase, mf_bad_names[mf_common.bad], (unsigned long long) mf_common.faces_allocated,
		(unsigned long long) (mf_common.faces_requested * sizeof (object_3d_face)), (unsigned long long) mf_common.faces_initialised,
		(unsigned long long) mf_common.list_allocated, (unsigned long long) (mf_common.list_requested * sizeof (point_3d_short_reference)),
		(unsigned long long) mf_common.list_initialised, points_view, (unsigned long long) mf_common.points_consumed, sp_view,
		(unsigned long long) mf_common.sp_consumed, MF_OBJECT_MAX, mf_custom_recorded, mf_custom_ambiguous, mf_custom_dropped, mf_custom_unknown, mf_custom_late);
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* catalogue */

static int entity_live (entity *en)
{
	return entities && en >= entities && en < entities + number_of_entities;
}

struct uses
{
	int count;
	int type[USE_TYPE_SLOTS];
	int type_count[USE_TYPE_SLOTS];
	int other;
};

int eech_model_catalogue_run (int first, int count, char *text, int capacity)
{
	struct out out;
	static struct uses uses[PAGE_SCENES];
	entity *en;
	long scanned = 0, with_shape = 0;
	int i;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data || first < 1 || count < 1 || count > PAGE_SCENES || first >= OBJECT_3D_LAST)
	{
		return 1;
	}
	if (first + count > OBJECT_3D_LAST)
	{
		count = OBJECT_3D_LAST - first;
	}

	/* the use count: the entities of the loaded campaign, by the shape the ordinary getter reports for each */
	memset (uses, 0, sizeof (uses));
	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		int shape, slot;
		if (!entity_live (en))
		{
			break;
		}
		scanned++;
		shape = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
		if (shape < 1 || shape >= OBJECT_3D_LAST)
		{
			continue;
		}
		with_shape++;
		if (shape >= first && shape < first + count)
		{
			struct uses *u = &uses[shape - first];
			u->count++;
			for (slot = 0; slot < USE_TYPE_SLOTS; slot++)
			{
				if (u->type_count[slot] && u->type[slot] == (int) en->type)
				{
					u->type_count[slot]++;
					break;
				}
				if (!u->type_count[slot])
				{
					u->type[slot] = (int) en->type;
					u->type_count[slot] = 1;
					break;
				}
			}
			if (slot == USE_TYPE_SLOTS)
			{
				u->other++;
			}
		}
	}

	emit (&out, "model-page first=%d count=%d scene_last=%d entities_scanned=%ld entities_with_shape=%ld\n", first, count, (int) OBJECT_3D_LAST, scanned, with_shape);
	for (i = first; i < first + count; i++)
	{
		struct analysis a;
		const struct OBJECT_3D_SCENE_DATABASE_ENTRY *scene = &objects_3d_scene_database[i];
		const char *name = scene_name (i);
		char bb[160], bb2[160], cbb[160];
		struct uses *u = &uses[i - first];
		int slot;
		float radius;
		object_3d_index_numbers index = (object_3d_index_numbers) i;

		analyse (&a, i);
		/* the original scene accessors, once each */
		bounds_text (bb, sizeof (bb), get_object_3d_bounding_box (index));
		bounds_text (bb2, sizeof (bb2), get_object_3d_bounding_box_without_lines (index));
		radius = get_object_3d_radius (index);
		emit (&out, "model-scene scene=%d name=%s main=%d succeeded=%d collision=%d status=%s detail=%s radius=0x%08x bb2=%s bb=%s uses=%d", i, name, scene->index,
			scene->succeeded, a.collision_index, a.status, a.detail, float_bits (radius), bb2, bb, u->count);
		emit (&out, " use_types=");
		if (!u->count)
		{
			emit (&out, "-");
		}
		for (slot = 0; slot < USE_TYPE_SLOTS && u->type_count[slot]; slot++)
		{
			emit (&out, "%s%s:%d", slot ? "," : "", get_entity_type_name ((entity_types) u->type[slot]), u->type_count[slot]);
		}
		emit (&out, " use_other=%d", u->other);
		if (a.collision_index > 0 && a.collision_index <= total_number_of_raw_3d_objects && strcmp (a.status, "null_surfaces"))
		{
			const object_3d *obj = &objects_3d_data[a.collision_index];
			bounds_text (cbb, sizeof (cbb), &obj->bounding_box);
			emit (&out,
				" c_points=%d c_faces=%d c_surfaces=%d c_custom=%d c_radius=0x%08x c_bb=%s surface_faces=%ld polygon_faces=%ld nonpolygon_surfaces=%ld zero_face_surfaces=%ld surface_points=%ld face_points=%ld"
				" face_points_min=%d face_points_max=%d fan_bound=%ld end_read_backed=%d cap_faces=%ld cap_point_list=%ld cap_surface_points=%ld max_depth=%ld",
				a.number_of_points, a.number_of_faces, a.number_of_surfaces, a.custom, float_bits (obj->radius), cbb, a.surface_faces, a.polygon_surface_faces,
				a.nonpolygon_surfaces, a.zero_face_surfaces, a.surface_points_sum, a.face_points_sum, a.face_points_min, a.face_points_max, a.fan_triangles,
				a.end_read_backed, a.cap_faces, a.cap_point_list, a.cap_surface_points, analysis_probeable (&a) ? analysis_max_depth (&a) : -1L);
			emit (&out, " backing=%s walk=%s stop=%s stop_array=%s faces_read=%ld list_read=%ld sp_read=%ld points_read=%ld", a.backing, a.walk, a.stop, a.stop_array,
				a.faces_read, a.list_read, a.sp_read, a.points_read);
		}
		emit (&out, "\n");
	}
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* geometry: the original accessor, depth by depth */

int eech_model_geometry_run (int scene, int first_depth, int count, char *text, int capacity)
{
	struct out out;
	struct analysis a;
	long max_depth;
	int depth, called = 0;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data || !scene_in_range (scene) || first_depth < 0 || count < 1 || count > PAGE_TRIANGLES ||
		first_depth > 0x7fffffff - PAGE_TRIANGLES)
	{
		return 1;
	}
	analyse (&a, scene);
	emit (&out, "model-geometry scene=%d name=%s collision=%d status=%s detail=%s first=%d count=%d fan_bound=%ld end_read_backed=%d backing=%s walk=%s stop=%s stop_array=%s\n",
		scene, scene_name (scene), a.collision_index, a.status, a.detail, first_depth, count, a.fan_triangles, a.end_read_backed, a.backing, a.walk, a.stop, a.stop_array);

	if (!strcmp (a.status, "absent"))
	{
		/* the original answers the absence itself, before any object is read */
		vec3d buffer[3];
		uint32_t words[9];
		int i, writes = 0, returned;
		for (i = 0; i < 9; i++)
		{
			words[i] = 0x7fc0a5a5u;
		}
		memcpy (buffer, words, sizeof (buffer));
		returned = get_object_3d_collision_object_geometry_triangle ((object_3d_index_numbers) scene, buffer, first_depth);
		memcpy (words, buffer, sizeof (words));
		for (i = 0; i < 9; i++)
		{
			writes += words[i] != 0x7fc0a5a5u;
		}
		emit (&out, "model-tri depth=%d return=%d writes=%d\n", first_depth, returned, writes);
		return out.overflow ? 1 : 0;
	}
	if (!analysis_probeable (&a))
	{
		emit (&out, "model-refused scene=%d status=%s detail=%s\n", scene, a.status, a.detail);
		return out.overflow ? 1 : 0;
	}

	max_depth = analysis_max_depth (&a);
	for (depth = first_depth; depth < first_depth + count; depth++)
	{
		vec3d buffer[3];
		uint32_t words[9];
		int i, writes = 0, returned;
		if (depth > max_depth)
		{
			emit (&out, "model-end scene=%d depth=%d reason=%s\n", scene, depth, (depth == a.fan_triangles) ? "end_probe_unbacked" : "beyond_safe_bound");
			break;
		}
		for (i = 0; i < 9; i++)
		{
			words[i] = 0x7fc0a5a5u;
		}
		memcpy (buffer, words, sizeof (buffer));
		returned = get_object_3d_collision_object_geometry_triangle ((object_3d_index_numbers) scene, buffer, depth);
		called++;
		memcpy (words, buffer, sizeof (words));
		for (i = 0; i < 9; i++)
		{
			writes += words[i] != 0x7fc0a5a5u;
		}
		emit (&out, "model-tri depth=%d return=%d writes=%d", depth, returned, writes);
		if (writes)
		{
			emit (&out, " p=");
			for (i = 0; i < 9; i++)
			{
				emit (&out, "%s0x%08x", i ? "," : "", words[i]);
			}
		}
		emit (&out, "\n");
		if (!returned)
		{
			emit (&out, "model-end scene=%d depth=%d reason=original_false\n", scene, depth);
			break;
		}
	}
	emit (&out, "model-geometry-end scene=%d calls=%d\n", scene, called);
	return out.overflow ? 1 : 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* state digest: every table the entries above read, hashed whole, and the observer's own records */

static void hash_bytes (uint64_t *hash, const void *data, size_t length)
{
	const unsigned char *p = (const unsigned char *) data;
	size_t i;
	for (i = 0; i < length; i++)
	{
		*hash ^= p[i];
		*hash *= FNV_PRIME;
	}
}

static void hash_u64 (uint64_t *hash, uint64_t value)
{
	hash_bytes (hash, &value, sizeof (value));
}

int eech_model_state_run (char *text, int capacity)
{
	struct out out;
	uint64_t tables = FNV_OFFSET, pointers = FNV_OFFSET, content = FNV_OFFSET, backing = FNV_OFFSET;
	long callable_objects = 0;
	int i;

	out_begin (&out, text, capacity);
	if (!objects_3d_scene_database || !objects_3d_data || !object_database_surfaces)
	{
		return 1;
	}
	/* the scene table and the object table, every byte; the surface block, every byte */
	hash_bytes (&tables, objects_3d_scene_database, sizeof (struct OBJECT_3D_SCENE_DATABASE_ENTRY) * (size_t) OBJECT_3D_LAST);
	hash_bytes (&tables, objects_3d_data, sizeof (object_3d) * (size_t) (total_number_of_raw_3d_objects + 1));
	hash_bytes (&tables, object_database_surfaces, sizeof (face_surface_description) * (size_t) surface_block_length ());
	/* the table and block addresses, and the bounds */
	{
		uintptr_t addresses[3];
		addresses[0] = (uintptr_t) objects_3d_scene_database;
		addresses[1] = (uintptr_t) objects_3d_data;
		addresses[2] = (uintptr_t) object_database_surfaces;
		hash_bytes (&pointers, addresses, sizeof (addresses));
		hash_bytes (&pointers, &total_number_of_raw_3d_objects, sizeof (total_number_of_raw_3d_objects));
	}
	/* the arrays the original decoder reads, for every scene whose collision model the verdict allows to be called: exactly the extents the walk read, all backed */
	for (i = 1; i < OBJECT_3D_LAST; i++)
	{
		struct analysis a;
		const object_3d *obj;
		analyse (&a, i);
		if (!a.callable || a.collision_index < 1)
		{
			continue;
		}
		obj = &objects_3d_data[a.collision_index];
		hash_bytes (&content, &i, sizeof (i));
		if (a.points_read > 0)
		{
			hash_bytes (&content, obj->points, sizeof (object_short_3d_point) * (size_t) a.points_read);
		}
		if (a.faces_read > 0)
		{
			hash_bytes (&content, obj->faces, sizeof (object_3d_face) * (size_t) a.faces_read);
		}
		if (a.list_read > 0)
		{
			hash_bytes (&content, obj->object_faces_point_plain_list, sizeof (point_3d_short_reference) * (size_t) a.list_read);
		}
		if (a.sp_read > 0)
		{
			hash_bytes (&content, obj->surface_points, sizeof (point_3d_plain_reference) * (size_t) a.sp_read);
		}
		callable_objects++;
	}
	/* the observer's records, member by member (no padding is hashed) */
	hash_u64 (&backing, mf_generation);
	hash_u64 (&backing, (uint64_t) mf_common.phase);
	hash_u64 (&backing, (uint64_t) mf_common.bad);
	hash_u64 (&backing, (uint64_t) mf_common.faces_lo);
	hash_u64 (&backing, (uint64_t) mf_common.list_lo);
	hash_u64 (&backing, mf_common.faces_allocated);
	hash_u64 (&backing, mf_common.list_allocated);
	hash_u64 (&backing, mf_common.faces_requested);
	hash_u64 (&backing, mf_common.list_requested);
	hash_u64 (&backing, mf_common.faces_initialised);
	hash_u64 (&backing, mf_common.list_initialised);
	hash_u64 (&backing, (uint64_t) mf_common.faces_short);
	hash_u64 (&backing, (uint64_t) mf_common.list_short);
	hash_u64 (&backing, (uint64_t) mf_common.points_lo);
	hash_u64 (&backing, (uint64_t) mf_common.sp_lo);
	hash_u64 (&backing, mf_common.points_consumed);
	hash_u64 (&backing, mf_common.sp_consumed);
	hash_u64 (&backing, (uint64_t) mf_custom_recorded);
	hash_u64 (&backing, (uint64_t) mf_custom_ambiguous);
	hash_u64 (&backing, (uint64_t) mf_custom_dropped);
	hash_u64 (&backing, (uint64_t) mf_custom_unknown);
	hash_u64 (&backing, (uint64_t) mf_custom_late);
	for (i = 1; i <= total_number_of_raw_3d_objects && i < MF_OBJECT_MAX; i++)
	{
		const struct mf_custom *r = &mf_custom_table[i];
		if (r->generation != mf_generation || r->state == 0)
		{
			continue;
		}
		hash_u64 (&backing, (uint64_t) i);
		hash_u64 (&backing, (uint64_t) (uintptr_t) r->object);
		hash_u64 (&backing, (uint64_t) r->state);
		hash_u64 (&backing, (uint64_t) r->points);
		hash_u64 (&backing, (uint64_t) r->faces);
		hash_u64 (&backing, (uint64_t) r->list);
		hash_u64 (&backing, (uint64_t) r->sp);
		hash_u64 (&backing, r->points_allocated);
		hash_u64 (&backing, r->faces_allocated);
		hash_u64 (&backing, r->list_allocated);
		hash_u64 (&backing, r->sp_allocated);
		hash_u64 (&backing, r->points_initialised);
		hash_u64 (&backing, r->faces_initialised);
		hash_u64 (&backing, r->list_initialised);
		hash_u64 (&backing, r->sp_initialised);
	}
	emit (&out, "model-state tables=%016llx pointers=%016llx content=%016llx content_objects=%ld backing=0x%016llx\n", (unsigned long long) tables,
		(unsigned long long) pointers, (unsigned long long) content, callable_objects, (unsigned long long) backing);
	return out.overflow ? 1 : 0;
}

#endif
