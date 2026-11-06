/*
 * ENTITY_SCAN: the ENUMERATION of EECH's own get_line_of_sight_collision_entity (aphavoc/source/entity/en_misc/en_maths/en_cllsn.c) that precedes every candidate decision
 * (EEDCS-49, D3, fourth slice). The per-candidate decisions are entity_candidate_step.c and the narrow phase entity_step.c.
 *
 * What the host decides here, expression for expression from the original: the segment's min and max, the sector window (the box widened by LARGEST_3D_OBJECT_RADIUS and clamped to the map's
 * extents, converted to sector indices), the traversal (x outer, z inner), that each sector's list is read from its TAIL backwards, that the source and the target are skipped, that a candidate
 * is OFFERED for its decisions, and that the FIRST hit ends the scan. A farther candidate in an earlier sector or list position wins over a nearer one: the order is the result.
 *
 * What the host keeps (src/roster.rs, served by src/model_host.rs): WHICH entities stand in each sector and in what order. The campaign reports each insertion into and removal from a sector list as an event
 * of the same question (phases 4 to 6, which this kernel is never asked about) and the host answers with a key it issued; this kernel decides the window, the traversal and the ends, and the entry a
 * sector's list serves (its oldest first, then what follows the entry the campaign is standing at) is the host's roster's. The campaign's own lists are only the conformance mirror of that order.
 *
 * What the campaign keeps, named and REQUIRED to move: the tallest structure of a sector, a city's buildings, the entities' live state and static pose, the semantic identity of a structure, and the
 * identity of the source and the target (a comparison only the entity heap can make, reported as a fact).
 *
 * It is a pure function of its facts, like entity_candidate_step.c: the host keeps nothing between questions and the cursor travels in the facts and the answer. The caller
 * (src/entity_host.rs) runs it under round-toward-zero with the caller's floating-point state saved and restored; this file does not touch the environment.
 *
 * FACTS (floats):
 *   [0..2] segment old end, [3..5] segment new end,
 *   [6] the sector side length (a whole number), [7] min map x, [8] min map z, [9] max map x, [10] max map z (the campaign's world map, copied; the 150 m widening is this file's),
 *   [11] PHASE: 0 BEGIN, 1 ENTRY (the cursor's entry is served), 2 END (the cursor's list has no more entries), 3 RESOLVED (the offered entry's decisions are finished),
 *   [12] cursor sector x, [13] cursor sector z (answers of the host, echoed),
 *   [14] the served entry is the source or the target (0 or 1), [15] the offered entry hit (0 or 1).
 * RESULT: a state and, for FIRST, the cursor (two sector indices as whole numbers).
 */
#include <math.h>
#include <stddef.h>

#define SCAN_FIRST (1)
#define SCAN_NEXT (2)
#define SCAN_OFFER (3)
#define SCAN_DONE_NONE (4)
#define SCAN_DONE_HIT (5)
#define SCAN_INVALID_CURSOR (-1)

#define SCAN_BEGIN (0)
#define SCAN_ENTRY (1)
#define SCAN_END (2)
#define SCAN_RESOLVED (3)

#define LARGEST_3D_OBJECT_RADIUS (150.0f)

#define max(a, b) (((a) > (b)) ? (a) : (b))
#define min(a, b) (((a) < (b)) ? (a) : (b))

/* get_x_sector / get_z_sector: convert_float_to_int (a truncation under the original's round-toward-zero), then the integer division by the side length */
static int sector_of(float value, int side)
{
	int sector;

	sector = (int)value;
	sector /= side;

	return sector;
}

int eech_entity_scan_step(const float *facts, float *cursor)
{
	float old_x, old_y, old_z, new_x, new_y, new_z, source_min_x, source_min_z, source_max_x, source_max_z, min_map_x, min_map_z, max_map_x, max_map_z;
	int side, phase, x_sector_min, z_sector_min, x_sector_max, z_sector_max, x_sector, z_sector;

	cursor[0] = 0.0f;
	cursor[1] = 0.0f;

	old_x = facts[0];
	old_y = facts[1];
	old_z = facts[2];
	new_x = facts[3];
	new_y = facts[4];
	new_z = facts[5];
	(void)old_y;
	(void)new_y;

	side = (int)facts[6];
	min_map_x = facts[7];
	min_map_z = facts[8];
	max_map_x = facts[9];
	max_map_z = facts[10];
	phase = (int)facts[11];
	x_sector = (int)facts[12];
	z_sector = (int)facts[13];

	/* get source min and max positions (the original's per-axis tests; y does not enter the window) */
	if (old_x <= new_x)
	{
		source_min_x = old_x;
		source_max_x = new_x;
	}
	else
	{
		source_min_x = new_x;
		source_max_x = old_x;
	}

	if (old_z <= new_z)
	{
		source_min_z = old_z;
		source_max_z = new_z;
	}
	else
	{
		source_min_z = new_z;
		source_max_z = old_z;
	}

	/* calc sector scan (widened to include entities which are overlapping sector edges) */
	x_sector_min = sector_of(max(source_min_x - LARGEST_3D_OBJECT_RADIUS, min_map_x), side);
	z_sector_min = sector_of(max(source_min_z - LARGEST_3D_OBJECT_RADIUS, min_map_z), side);
	x_sector_max = sector_of(min(source_max_x + LARGEST_3D_OBJECT_RADIUS, max_map_x), side);
	z_sector_max = sector_of(min(source_max_z + LARGEST_3D_OBJECT_RADIUS, max_map_z), side);

	if (phase == SCAN_BEGIN)
	{
		/* for (x_sector = x_sector_min; x_sector <= x_sector_max; x_sector++) for (z_sector = z_sector_min; z_sector <= z_sector_max; z_sector++) */
		if ((x_sector_min > x_sector_max) || (z_sector_min > z_sector_max))
		{
			return SCAN_DONE_NONE;
		}

		cursor[0] = (float)x_sector_min;
		cursor[1] = (float)z_sector_min;

		return SCAN_FIRST;
	}

	/* a cursor the campaign echoes must lie in the window this question computes: anything else is not a cursor this function gave */
	if ((x_sector < x_sector_min) || (x_sector > x_sector_max) || (z_sector < z_sector_min) || (z_sector > z_sector_max))
	{
		return SCAN_INVALID_CURSOR;
	}

	if (phase == SCAN_ENTRY)
	{
		/* if ((target != los_source) && (target != los_target)) the candidate is offered, otherwise the next entry is taken */
		if (facts[14] != 0.0f)
		{
			return SCAN_NEXT;
		}

		return SCAN_OFFER;
	}

	if (phase == SCAN_RESOLVED)
	{
		/* the first candidate that hits ends the scan */
		if (facts[15] != 0.0f)
		{
			return SCAN_DONE_HIT;
		}

		return SCAN_NEXT;
	}

	/* SCAN_END: the list of the cursor's sector has no more entries; the next sector, z inner then x outer */
	z_sector++;

	if (z_sector > z_sector_max)
	{
		x_sector++;
		z_sector = z_sector_min;

		if (x_sector > x_sector_max)
		{
			return SCAN_DONE_NONE;
		}
	}

	cursor[0] = (float)x_sector;
	cursor[1] = (float)z_sector;

	return SCAN_FIRST;
}
