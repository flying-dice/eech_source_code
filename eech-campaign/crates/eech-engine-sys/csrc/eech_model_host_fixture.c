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
 *              untouched), the pointer-stability check of the bounds, and the caller's rounding mode before and after.
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
};

extern struct campaign_model_counters campaign_model_counts;
extern void campaign_model_poison_arrays (void);

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
	int n = snprintf (text, (size_t) capacity,
		"model-host catalogue=%ld scene=%ld triangle=%ld found=%ld end=%ld absent=%ld bounds_reads=%ld bounds_without_lines_reads=%ld radius_reads=%ld\n",
		campaign_model_counts.catalogue, campaign_model_counts.scene, campaign_model_counts.triangle, campaign_model_counts.found, campaign_model_counts.end,
		campaign_model_counts.absent, campaign_model_counts.bounds_reads, campaign_model_counts.bounds_without_lines_reads, campaign_model_counts.radius_reads);

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
	return ((n < 0) || (n >= capacity - used)) ? 1 : 0;
}

#endif

#endif
