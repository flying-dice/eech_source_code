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

#include "eech_engine.h"
#include "eech_headless.h"

#ifdef EECH_CAMPAIGN_HOST_MODEL

struct campaign_model_counters
{
	long catalogue, scene, triangle, found, end, absent, bounds_reads, bounds_without_lines_reads, radius_reads;
	long presence_checked, presence_mismatches;
};

extern struct campaign_model_counters campaign_model_counts;
extern void campaign_model_poison_arrays (void);
extern void campaign_model_core_residency (int *rows, int *metadata_rows, int *points, int *faces, int *point_refs, int *surface_points);
extern void campaign_model_core_horizon (int *cached, int *uncached);
extern void campaign_model_core_metadata (long *values);
extern void campaign_model_core_scene_state (int scene, int *read, int *custom, int *listed);

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
	long metadata[8];

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
		"model-core-metadata objects_asked=%ld objects_found=%ld objects_absent=%ld rows_filled=%ld rows_confirmed=%ld animation_asked=%ld animation_names=%ld animation_ends=%ld\n",
		metadata[0], metadata[1], metadata[2], metadata[3], metadata[4], metadata[5], metadata[6], metadata[7]);

	return ((j < 0) || (j >= capacity - n - m - k)) ? 1 : 0;
}

/* The scenes' FINAL texture animation lists as the campaign holds them after the host-fed load (EEDCS-50, D3, L2), in the row grammar of the original observer (eech_model_fixture.c,
   eech_model_animation_run) so the same driver reads them and a literal comparison with the original's rows is possible: scenes first .. first + count - 1; a scene the object loader
   never read is `absent`; a list of no entries is `empty`; otherwise `listed` with the entries as the campaign's own table holds them (its private animation numbers: diagnostic data,
   never the host protocol). The source is `built` for a scene a custom scene file defined, `file` for a database record; the allocation of a built list is what add_animation makes (8, then
   doubling) and of a file list its count. Nothing is recomputed: the entries are read from the scene database as they stand. The page digest covers the same projection (status, count,
   entries) but is this profile's own: it is not the original's. */
#define HOST_ANIM_IDS_MAX 4096
#define HOST_ANIM_PAGE_SCENES 96

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
	int succeeded, count, capacity, filled, read;
	const int *ids;
};

static void host_anim_view (int scene, struct host_anim_view *v)
{
	const struct OBJECT_3D_SCENE_DATABASE_ENTRY *e = &objects_3d_scene_database[scene];
	int custom, listed, cap;

	memset (v, 0, sizeof (*v));
	v->status = "absent";
	v->source = "none";
	campaign_model_core_scene_state (scene, &v->read, &custom, &listed);

	if (!v->read)
	{
		return;
	}

	v->succeeded = e->succeeded;
	v->source = custom ? "built" : "file";
	v->count = e->number_of_texture_animations;

	if (v->count < 0)
	{
		v->status = "unbacked";
		return;
	}

	if (v->count == 0)
	{
		v->status = e->texture_animations ? "unbacked" : "empty";
		return;
	}

	if (v->count > HOST_ANIM_IDS_MAX)
	{
		v->status = "capacity";
		return;
	}

	if (!e->texture_animations)
	{
		v->status = "unbacked";
		return;
	}

	cap = 8;
	while (cap < v->count)
	{
		cap *= 2;
	}
	v->capacity = custom ? cap : v->count;
	v->filled = v->count;
	v->status = "listed";
	v->ids = (const int *) e->texture_animations;
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
		done += v.read != 0;
		host_anim_hash (&content, (uint64_t) scene);
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
		1u, done, 0, (unsigned long long) content);

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
