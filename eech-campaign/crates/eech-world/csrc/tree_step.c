/*
 * The physical calculation of whether a line segment hits a forest tree (EEDCS-49, D3): EECH's own get_line_of_sight_collision_tree
 * (aphavoc/source/entity/en_misc/en_maths/en_cllsn.c) with the line-quad and line-triangle intersections it calls
 * (modules/maths/vector.c get_3d_line_quad_intersection and get_3d_line_triangle_intersection), kept expression for expression so the host
 * and the original reference produce the same bits, and turned into a resumable state machine because the host asks the campaign for the
 * two terrain elevations the original reads (D3 terrain heights are not the host's yet; that is declared transitional coupling).
 *
 * What the host owns here: the tree records and the per-sector tables (loaded from the map's own tree_pos.sec and tree_pos.dat by
 * src/trees.rs), the map metadata (extents, terrain sector counts, the terrain height scale the tree heights use) and, when the original has
 * one, the forest tree's collision triangles. All of it arrives as a borrowed, read-only scene descriptor on each call; this file keeps no
 * pointer between calls, so nothing here can outlive the storage that Rust owns.
 *
 * What is kept exactly (the reasons are the original's, not choices made here):
 *   - the lookup order: the terrain elevation at the old end point, then at the new one, each asked only if that end point is inside the map
 *     (x and z only, bounds inclusive, y ignored); an end point outside the map has radial altitude 0 (old) or the old one's (new);
 *   - the early FALSE when the lower radial altitude is above the 35 m maximum tree height, and when the 2D range is under 0.1;
 *   - float and double types: `sqrt` returns a double narrowed to float, `TREE_RADIUS`, `0.5` and `25.0` are doubles, the tree position and
 *     height scaling promote through double exactly as the original expressions do;
 *   - the sector scan x outer, z inner, each sector's trees in file order, with the ignore_forest_edge_trees filter;
 *   - the similar-triangle divisions by dx and dz (no guard: an IEEE infinity or NaN falls out of the comparisons as it does there);
 *   - the nearest-by-2D-range choice with its strict `<`, the `continue` after a nearer hit, the top quad only when skip_tree_top_test is 0;
 *   - the intersection helpers write the intercept point and normal BEFORE the hit decision, so a miss leaves partial writes; the point and
 *     normal the caller passes in are inputs, and the final values are returned whether or not there was a hit;
 *   - the geometry branch: a tree of type 1 is tested against the forest tree's collision triangles when the original has any, and the first
 *     triangle hit returns TRUE at once, without the nearest choice.
 *
 * What is not here: the debug drawing, the ASSERTs, and the original's side effect of setting the x87 rounding mode to zero in
 * get_terrain_3d_sector (the conversion to a sector number is a C truncation, as in the 64-bit patched engine). The decoding of the model's
 * collision faces into triangles (3dobjdb.c get_object_3d_collision_object_geometry_triangle) is not the host's yet: the scene says either that
 * there are no triangles (the retail facts) or supplies them already decoded, and no other state is accepted.
 *
 * STATUS: PREPARATION. This is the host-private kernel of the first host-owned physical component (static tree data and the tree segment query),
 * written and checked against the reviewed original captures before anything depends on it. The campaign DLL's tree queries still run the
 * original solver; no production dispatch has moved here, and nothing in this file claims that the responsibility has moved. Still open for D3:
 * the decoding of the forest tree's collision model into the triangles the geometry branch reads (the branch below is the original's text and has
 * not been run against a native witness; the retail facts have no triangles), the terrain heights the two lookups ask for, and the entity
 * collision that shares the original's surrounding code.
 *
 * The state is one open query. A new eech_tree_begin replaces whatever was open; an answer with nothing pending is refused.
 */
#include <stddef.h>
#include <stdint.h>

#include "eech_arith.h"

#define TREE_SECTOR_SIDE_LENGTH 2048
#define TREE_XZ_SCALE (((float) (TREE_SECTOR_SIDE_LENGTH + TREE_SECTOR_SIDE_LENGTH)) / 65536.0f)
#define TREE_MAX_TREE_HEIGHT (35)
#define TREE_RADIUS (5.0)

#ifndef FALSE
#define FALSE 0
#endif
#ifndef TRUE
#define TRUE 1
#endif

/* the states of eech_tree_request */
#define TREE_DONE 1
#define TREE_NEED_TERRAIN 2

/* where the query is waiting */
enum
{
	AWAIT_NONE,
	AWAIT_OLD_ELEVATION,
	AWAIT_NEW_ELEVATION
};

/* the results of the calls */
#define TREE_OK 0
#define TREE_OUT_OF_SEQUENCE (-1)
#define TREE_FAULT (-2)

/* terrain_3d_tree_data, decoded explicitly by src/trees.rs from the 8 bytes a retail record occupies; src/trees.rs asserts the same size */
typedef struct eech_tree
{
	int16_t x;
	int16_t z;
	uint16_t y;
	uint8_t colour;
	uint8_t height;
	uint8_t type;
	uint8_t reserved;
} eech_tree;

/* terrain_3d_tree_sector: the number of trees and the index of the first one in the tree array (the file's offset is in records) */
typedef struct eech_tree_sector
{
	int32_t count;
	uint32_t first;
} eech_tree_sector;

typedef char eech_tree_size_check[(sizeof(eech_tree) == 10) ? 1 : -1];
typedef char eech_tree_sector_size_check[(sizeof(eech_tree_sector) == 8) ? 1 : -1];

/* borrowed for one call: nothing here is kept */
typedef struct eech_tree_scene
{
	float map_min[3];
	float map_max[3];
	int32_t sectors_x;
	int32_t sectors_z;
	float minimum_height;
	float scaled_height_difference;
	const eech_tree_sector *sectors; /* NULL: the map has no trees loaded; else sectors_x * sectors_z entries, indexed [z * sectors_x + x] */
	const eech_tree *trees;
	size_t tree_count;
	int32_t geometry_present; /* 0: the original has no forest tree collision triangles; 1: geometry holds geometry_triangles of them */
	int32_t reserved;
	const float *geometry; /* 9 floats per triangle: the three points, in the order the original's triangle function returns them */
	size_t geometry_triangles;
} eech_tree_scene;

typedef struct eech_tree_request
{
	int32_t state;
	int32_t endpoint; /* NEED_TERRAIN: 0 for the old end point, 1 for the new one */
	float x;
	float z;
	int32_t hit;
	float point[3];
	float normal[3];
} eech_tree_request;

typedef struct eech_tree_query
{
	int awaiting;
	int ignore_forest_edge_trees;
	int skip_tree_top_test;
	float old_rad_alt;
	vec3d old_position;
	vec3d new_position;
	vec3d intercept_point;
	vec3d face_normal;
} eech_tree_query;

size_t eech_tree_query_size(void)
{
	return sizeof(eech_tree_query);
}

size_t eech_tree_record_size(void)
{
	return sizeof(eech_tree);
}

size_t eech_tree_sector_size(void)
{
	return sizeof(eech_tree_sector);
}

size_t eech_tree_scene_size(void)
{
	return sizeof(eech_tree_scene);
}

/*
 * The floating-point exception masks of this thread, read and never written (the fixture's guard reads the same two registers): the low six
 * bits of the x87 control word and bits 7 to 12 of MXCSR, each 63 when every exception is masked, as the original's callers have them. The
 * SSE rounding control is reported too, and is not judged here.  Returns 0, or -1 where the registers cannot be read.
 */
int eech_tree_fp_state(int *x87_masks, int *sse_masks, int *sse_rounding)
{
#if defined(__x86_64__) || defined(__i386__)
	uint16_t control_word = 0;
	uint32_t mxcsr = 0;

	__asm__ __volatile__("fnstcw %0" : "=m"(control_word));
	__asm__ __volatile__("stmxcsr %0" : "=m"(mxcsr));
	*x87_masks = (int) (control_word & 0x3fu);
	*sse_masks = (int) ((mxcsr >> 7) & 0x3fu);
	*sse_rounding = (int) ((mxcsr >> 13) & 0x3u);
	return 0;
#else
	*x87_masks = 0;
	*sse_masks = 0;
	*sse_rounding = 0;
	return -1;
#endif
}

/* ------------------------------------------------------------------------------------------------------------------------------------ */
/* maths/vector.c: the intersections, without the ASSERTs or debug logging; the float, double and narrowing steps are the original's. */
/* ------------------------------------------------------------------------------------------------------------------------------------ */

static int get_3d_line_triangle_intersection(const vec3d *p1, const vec3d *p2, const vec3d *pa, const vec3d *pb, const vec3d *pc, vec3d *pi, vec3d *ni)
{
	float length, d, denom, mu, angle_aib, angle_aic, angle_bic, acos_angle_aib, acos_angle_aic, acos_angle_bic, total;

	vec3d vec_ab, vec_ac, vec_12, vec_ia, vec_ib, vec_ic;

	vec_ab.x = pb->x - pa->x;
	vec_ab.y = pb->y - pa->y;
	vec_ab.z = pb->z - pa->z;

	vec_ac.x = pc->x - pa->x;
	vec_ac.y = pc->y - pa->y;
	vec_ac.z = pc->z - pa->z;

	ni->x = (vec_ab.y * vec_ac.z) - (vec_ab.z * vec_ac.y);
	ni->y = (vec_ab.z * vec_ac.x) - (vec_ab.x * vec_ac.z);
	ni->z = (vec_ab.x * vec_ac.y) - (vec_ab.y * vec_ac.x);

	length = sqrt((ni->x * ni->x) + (ni->y * ni->y) + (ni->z * ni->z));

	if (length == 0.0)
	{
		return (FALSE);
	}

	normalise_3d_vector_given_magnitude(ni, length);

	d = -(ni->x * pa->x) - (ni->y * pa->y) - (ni->z * pa->z);

	vec_12.x = p2->x - p1->x;
	vec_12.y = p2->y - p1->y;
	vec_12.z = p2->z - p1->z;

	denom = (ni->x * vec_12.x) + (ni->y * vec_12.y) + (ni->z * vec_12.z);

	if (fabs(denom) == 0.0)
	{
		return (FALSE);
	}

	mu = -(d + (ni->x * p1->x) + (ni->y * p1->y) + (ni->z * p1->z)) / denom;

	if ((mu < 0.0) || (mu > 1.0))
	{
		return (FALSE);
	}

	pi->x = p1->x + mu * (vec_12.x);
	pi->y = p1->y + mu * (vec_12.y);
	pi->z = p1->z + mu * (vec_12.z);

	vec_ia.x = pa->x - pi->x;
	vec_ia.y = pa->y - pi->y;
	vec_ia.z = pa->z - pi->z;

	length = sqrt((vec_ia.x * vec_ia.x) + (vec_ia.y * vec_ia.y) + (vec_ia.z * vec_ia.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ia, length);

	vec_ib.x = pb->x - pi->x;
	vec_ib.y = pb->y - pi->y;
	vec_ib.z = pb->z - pi->z;

	length = sqrt((vec_ib.x * vec_ib.x) + (vec_ib.y * vec_ib.y) + (vec_ib.z * vec_ib.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ib, length);

	vec_ic.x = pc->x - pi->x;
	vec_ic.y = pc->y - pi->y;
	vec_ic.z = pc->z - pi->z;

	length = sqrt((vec_ic.x * vec_ic.x) + (vec_ic.y * vec_ic.y) + (vec_ic.z * vec_ic.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ic, length);

	angle_aib = (vec_ia.x * vec_ib.x) + (vec_ia.y * vec_ib.y) + (vec_ia.z * vec_ib.z);
	angle_bic = (vec_ib.x * vec_ic.x) + (vec_ib.y * vec_ic.y) + (vec_ib.z * vec_ic.z);
	angle_aic = (vec_ic.x * vec_ia.x) + (vec_ic.y * vec_ia.y) + (vec_ic.z * vec_ia.z);

	acos_angle_aib = acos(angle_aib);
	acos_angle_bic = acos(angle_bic);
	acos_angle_aic = acos(angle_aic);

	total = acos_angle_aib + acos_angle_bic + acos_angle_aic;

	if ((total < (-PI2 + 0.01)) || (total > (PI2 - 0.01)))
	{
		return (TRUE);
	}

	return (FALSE);
}

static int get_3d_line_quad_intersection(const vec3d *p1, const vec3d *p2, const vec3d *pa, const vec3d *pb, const vec3d *pc, const vec3d *pd, vec3d *pi, vec3d *ni)
{
	float length, d, denom, mu, angle_aib, angle_aic, angle_aid, angle_bic, angle_cid, acos_angle_aib, acos_angle_aic, acos_angle_aid, acos_angle_bic,
		acos_angle_cid, total;

	vec3d vec_ab, vec_ac, vec_12, vec_ia, vec_ib, vec_ic, vec_id;

	vec_ab.x = pb->x - pa->x;
	vec_ab.y = pb->y - pa->y;
	vec_ab.z = pb->z - pa->z;

	vec_ac.x = pc->x - pa->x;
	vec_ac.y = pc->y - pa->y;
	vec_ac.z = pc->z - pa->z;

	ni->x = (vec_ab.y * vec_ac.z) - (vec_ab.z * vec_ac.y);
	ni->y = (vec_ab.z * vec_ac.x) - (vec_ab.x * vec_ac.z);
	ni->z = (vec_ab.x * vec_ac.y) - (vec_ab.y * vec_ac.x);

	length = sqrt((ni->x * ni->x) + (ni->y * ni->y) + (ni->z * ni->z));

	if (length == 0.0)
	{
		return (FALSE);
	}

	normalise_3d_vector_given_magnitude(ni, length);

	d = -(ni->x * pa->x) - (ni->y * pa->y) - (ni->z * pa->z);

	vec_12.x = p2->x - p1->x;
	vec_12.y = p2->y - p1->y;
	vec_12.z = p2->z - p1->z;

	denom = (ni->x * vec_12.x) + (ni->y * vec_12.y) + (ni->z * vec_12.z);

	if (fabs(denom) == 0.0)
	{
		return (FALSE);
	}

	mu = -(d + (ni->x * p1->x) + (ni->y * p1->y) + (ni->z * p1->z)) / denom;

	if ((mu < 0.0) || (mu > 1.0))
	{
		return (FALSE);
	}

	pi->x = p1->x + mu * (vec_12.x);
	pi->y = p1->y + mu * (vec_12.y);
	pi->z = p1->z + mu * (vec_12.z);

	vec_ia.x = pa->x - pi->x;
	vec_ia.y = pa->y - pi->y;
	vec_ia.z = pa->z - pi->z;

	length = sqrt((vec_ia.x * vec_ia.x) + (vec_ia.y * vec_ia.y) + (vec_ia.z * vec_ia.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ia, length);

	vec_ib.x = pb->x - pi->x;
	vec_ib.y = pb->y - pi->y;
	vec_ib.z = pb->z - pi->z;

	length = sqrt((vec_ib.x * vec_ib.x) + (vec_ib.y * vec_ib.y) + (vec_ib.z * vec_ib.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ib, length);

	vec_ic.x = pc->x - pi->x;
	vec_ic.y = pc->y - pi->y;
	vec_ic.z = pc->z - pi->z;

	length = sqrt((vec_ic.x * vec_ic.x) + (vec_ic.y * vec_ic.y) + (vec_ic.z * vec_ic.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_ic, length);

	angle_aib = (vec_ia.x * vec_ib.x) + (vec_ia.y * vec_ib.y) + (vec_ia.z * vec_ib.z);
	angle_bic = (vec_ib.x * vec_ic.x) + (vec_ib.y * vec_ic.y) + (vec_ib.z * vec_ic.z);
	angle_aic = (vec_ic.x * vec_ia.x) + (vec_ic.y * vec_ia.y) + (vec_ic.z * vec_ia.z);

	acos_angle_aib = acos(angle_aib);
	acos_angle_bic = acos(angle_bic);
	acos_angle_aic = acos(angle_aic);

	total = acos_angle_aib + acos_angle_bic + acos_angle_aic;

	if ((total < (-PI2 + 0.01)) || (total > (PI2 - 0.01)))
	{
		return (TRUE);
	}

	vec_id.x = pd->x - pi->x;
	vec_id.y = pd->y - pi->y;
	vec_id.z = pd->z - pi->z;

	length = sqrt((vec_id.x * vec_id.x) + (vec_id.y * vec_id.y) + (vec_id.z * vec_id.z));

	if (length == 0.0)
	{
		return (TRUE);
	}

	normalise_3d_vector_given_magnitude(&vec_id, length);

	angle_cid = (vec_ic.x * vec_id.x) + (vec_ic.y * vec_id.y) + (vec_ic.z * vec_id.z);
	angle_aid = (vec_id.x * vec_ia.x) + (vec_id.y * vec_ia.y) + (vec_id.z * vec_ia.z);

	acos_angle_cid = acos(angle_cid);
	acos_angle_aid = acos(angle_aid);

	total = acos_angle_aic + acos_angle_cid + acos_angle_aid;

	if ((total < (-PI2 + 0.01)) || (total > (PI2 - 0.01)))
	{
		return (TRUE);
	}

	return (FALSE);
}

/* en_cllsn.c: the two face wrappers, minus the debug drawing */
#define get_3d_line_triangle_face_intersection(OLD, NEW, P1, P2, P3, INTERCEPT, NORMAL) get_3d_line_triangle_intersection(OLD, NEW, P1, P2, P3, INTERCEPT, NORMAL)
#define get_3d_line_quad_face_intersection(OLD, NEW, P1, P2, P3, P4, INTERCEPT, NORMAL) get_3d_line_quad_intersection(OLD, NEW, P1, P2, P3, P4, INTERCEPT, NORMAL)

/* maths/range.c */
static float get_2d_range(const vec3d *v1, const vec3d *v2)
{
	float dx, dz, range;

	dx = v1->x - v2->x;
	dz = v1->z - v2->z;

	range = sqrt((dx * dx) + (dz * dz));

	return (range);
}

/* maths/matrix.c */
static void get_identity_matrix3x3(matrix3x3 m)
{
	m[0][0] = 1.0;
	m[0][1] = 0.0;
	m[0][2] = 0.0;

	m[1][0] = 0.0;
	m[1][1] = 1.0;
	m[1][2] = 0.0;

	m[2][0] = 0.0;
	m[2][1] = 0.0;
	m[2][2] = 1.0;
}

/* en_world.h point_inside_map_area: x and z only, the bounds inclusive */
static int point_inside_map_area(const eech_tree_scene *scene, const vec3d *position)
{
	return (position->x >= scene->map_min[0]) && (position->x <= scene->map_max[0]) && (position->z >= scene->map_min[2]) && (position->z <= scene->map_max[2]);
}

/* terrdata.c get_terrain_3d_sector, with the engine's patched C conversion; the clamped position is never negative, so truncation is the floor */
static void get_terrain_3d_sector(float x, float z, int *sectorx, int *sectorz)
{
	*sectorx = (int) (x / TREE_SECTOR_SIDE_LENGTH);
	*sectorz = (int) (z / TREE_SECTOR_SIDE_LENGTH);
}

/* terrdata.c get_terrain_3d_tree_scale */
static float get_terrain_3d_tree_scale(const eech_tree *tree)
{
	float scale;

	scale = tree->height;
	scale *= ((25 - 6) / 127.0f);
	scale += 6;
	scale *= 1.10f;

	return (scale);
}

/* en_maths: get_object_3d_collision_object_geometry_triangle, for a scene that already holds the decoded triangles; FALSE past the last (or when there are none) */
static int get_forest_tree_collision_triangle(const eech_tree_scene *scene, vec3d *return_points, int depth)
{
	const float *triangle;

	if (!scene->geometry_present || depth < 0 || (size_t) depth >= scene->geometry_triangles)
	{
		return (FALSE);
	}

	triangle = scene->geometry + ((size_t) depth * 9);

	return_points[0].x = triangle[0];
	return_points[0].y = triangle[1];
	return_points[0].z = triangle[2];

	return_points[1].x = triangle[3];
	return_points[1].y = triangle[4];
	return_points[1].z = triangle[5];

	return_points[2].x = triangle[6];
	return_points[2].y = triangle[7];
	return_points[2].z = triangle[8];

	return (TRUE);
}

/* en_world.c: the tree sector the scene has for these sector numbers; -1 where the scene's table does not cover them (the original reads out of bounds there) */
static int get_terrain_3d_tree_sector_data(const eech_tree_scene *scene, int x, int z, const eech_tree **ptr)
{
	const eech_tree_sector *sector;

	if (!scene->sectors)
	{
		return (0);
	}

	if (x < 0 || z < 0 || x >= scene->sectors_x || z >= scene->sectors_z)
	{
		return (-1);
	}

	sector = scene->sectors + ((size_t) z * (size_t) scene->sectors_x + (size_t) x);

	if (sector->count > 0)
	{
		/* the loader has checked that first + count lies inside the tree array; the scene is refused here too if it does not */
		if ((size_t) sector->first > scene->tree_count || (size_t) sector->count > scene->tree_count - (size_t) sector->first)
		{
			return (-1);
		}

		*ptr = scene->trees + sector->first;
	}

	return (sector->count);
}

/*
 * The rest of get_line_of_sight_collision_tree once the radial altitudes of the two end points are known: the original's text from the
 * "check if source is above highest tree" comparison onward. Returns the original's result (TRUE or FALSE), or TREE_FAULT when the scene's tree
 * sectors do not cover a sector the scan reaches.
 */
static int scan(eech_tree_query *q, const eech_tree_scene *scene, float source_old_position_rad_alt, float source_new_position_rad_alt)
{
	int x_sector_min, z_sector_min, x_sector_max, z_sector_max, x_sector, z_sector, hit_tree, num_trees, depth;

	float source_range, source_dy, source_ratio, height, tree_top, tree_range, nearest_tree_range, tree_intercept_y, x_sector_mid_point,
		z_sector_mid_point, dx, dz, dx1, dz1, dx2, dz2, ddx, ddz;

	vec3d nearest_tree_intercept_point, nearest_tree_face_normal, source_min_position, source_max_position, tree_position, p1, p2, p3, p4, p[8];

	const eech_tree *tree, *first_tree;
	int tree_index;

	matrix3x3 attitude;

	vec3d *source_old_position = &q->old_position;
	vec3d *source_new_position = &q->new_position;
	vec3d *source_intercept_point = &q->intercept_point;
	vec3d *face_normal = &q->face_normal;
	int ignore_forest_edge_trees = q->ignore_forest_edge_trees;
	int skip_tree_top_test = q->skip_tree_top_test;
	float terrain_3d_map_minimum_height = scene->minimum_height;
	float terrain_3d_map_scaled_height_difference = scene->scaled_height_difference;

	if (min(source_old_position_rad_alt, source_new_position_rad_alt) > TREE_MAX_TREE_HEIGHT)
	{
		return (FALSE);
	}

	source_range = get_2d_range(source_old_position, source_new_position);

	if (source_range < 0.1)
	{
		return (FALSE);
	}

	source_dy = source_new_position->y - source_old_position->y;

	source_ratio = source_dy / source_range;

	if (source_old_position->x <= source_new_position->x)
	{
		source_min_position.x = source_old_position->x;
		source_max_position.x = source_new_position->x;
	}
	else
	{
		source_min_position.x = source_new_position->x;
		source_max_position.x = source_old_position->x;
	}

	if (source_old_position->y <= source_new_position->y)
	{
		source_min_position.y = source_old_position->y;
		source_max_position.y = source_new_position->y;
	}
	else
	{
		source_min_position.y = source_new_position->y;
		source_max_position.y = source_old_position->y;
	}

	if (source_old_position->z <= source_new_position->z)
	{
		source_min_position.z = source_old_position->z;
		source_max_position.z = source_new_position->z;
	}
	else
	{
		source_min_position.z = source_new_position->z;
		source_max_position.z = source_old_position->z;
	}

	bound_position_to_map_area(&source_min_position, scene->map_min, scene->map_max);

	bound_position_to_map_area(&source_max_position, scene->map_min, scene->map_max);

	get_terrain_3d_sector(source_min_position.x, source_min_position.z, &x_sector_min, &z_sector_min);

	get_terrain_3d_sector(source_max_position.x, source_max_position.z, &x_sector_max, &z_sector_max);

	hit_tree = FALSE;

	nearest_tree_range = FLT_MAX;

	for (x_sector = x_sector_min; x_sector <= x_sector_max; x_sector++)
	{
		for (z_sector = z_sector_min; z_sector <= z_sector_max; z_sector++)
		{
			num_trees = get_terrain_3d_tree_sector_data(scene, x_sector, z_sector, &tree);

			if (num_trees < 0)
			{
				return (TREE_FAULT);
			}

			if (num_trees > 0)
			{
				x_sector_mid_point = (x_sector * TREE_SECTOR_SIDE_LENGTH) + (TREE_SECTOR_SIDE_LENGTH * 0.5);
				z_sector_mid_point = (z_sector * TREE_SECTOR_SIDE_LENGTH) + (TREE_SECTOR_SIDE_LENGTH * 0.5);

				/* the original steps a pointer from one before the first tree; an index from the first tree visits the same trees in the same order */
				first_tree = tree;
				tree_index = -1;

				while (num_trees--)
				{
					tree_index++;
					tree = first_tree + tree_index;

					if (!(ignore_forest_edge_trees && (tree->type == 0)))
					{
						tree_position.y = (tree->y * terrain_3d_map_scaled_height_difference * 0.5) + terrain_3d_map_minimum_height;

						if (source_max_position.y >= tree_position.y)
						{
							height = get_terrain_3d_tree_scale(tree);

							tree_top = tree_position.y + height;

							if (source_min_position.y <= tree_top)
							{
								tree_position.x = x_sector_mid_point + (tree->x * TREE_XZ_SCALE * 0.5);

								tree_position.z = z_sector_mid_point + (tree->z * TREE_XZ_SCALE * 0.5);

								depth = 0;

								if ((tree_position.x + 25.0 >= source_min_position.x) && (tree_position.x - 25.0 <= source_max_position.x) &&
									(tree_position.z + 25.0 >= source_min_position.z) && (tree_position.z - 25.0 <= source_max_position.z))
								{
									if (tree->type == 1 && get_forest_tree_collision_triangle(scene, p, depth++))
									{
										get_identity_matrix3x3(attitude);

										do
										{
											multiply_matrix3x3_vec3d(&p[0], attitude, &p[0]);
											multiply_matrix3x3_vec3d(&p[1], attitude, &p[1]);
											multiply_matrix3x3_vec3d(&p[2], attitude, &p[2]);

											p[0].x = p[0].x * height + tree_position.x;
											p[0].y = p[0].y * height + tree_position.y;
											p[0].z = p[0].z * height + tree_position.z;

											p[1].x = p[1].x * height + tree_position.x;
											p[1].y = p[1].y * height + tree_position.y;
											p[1].z = p[1].z * height + tree_position.z;

											p[2].x = p[2].x * height + tree_position.x;
											p[2].y = p[2].y * height + tree_position.y;
											p[2].z = p[2].z * height + tree_position.z;

											if (get_3d_line_triangle_face_intersection(source_old_position, source_new_position, &p[0], &p[1], &p[2], source_intercept_point, face_normal))
											{
												return TRUE;
											}
										}
										while (get_forest_tree_collision_triangle(scene, p, depth++));
									}
									else
									{
										dx = source_new_position->x - source_old_position->x;
										dz = source_new_position->z - source_old_position->z;

										dx1 = tree_position.x - source_old_position->x;
										dz1 = dx1 * dz / dx;

										dz2 = tree_position.z - source_old_position->z;
										dx2 = dz2 * dx / dz;

										ddx = dx1 - dx2;
										ddz = dz1 - dz2;

										if (((ddx > -TREE_RADIUS) && (ddx < TREE_RADIUS)) || ((ddz > -TREE_RADIUS) && (ddz < TREE_RADIUS)))
										{
											tree_range = get_2d_range(source_old_position, &tree_position);

											tree_intercept_y = (tree_range * source_ratio) + source_old_position->y;

											if ((tree_position.y <= tree_intercept_y) && (tree_top >= tree_intercept_y))
											{
												p1.x = tree_position.x - (TREE_RADIUS * 0.5);
												p1.y = tree_position.y;
												p1.z = tree_position.z;

												p4.x = tree_position.x + (TREE_RADIUS * 0.5);
												p4.y = tree_position.y;
												p4.z = tree_position.z;

												p2.x = p1.x;
												p2.y = tree_top;
												p2.z = p1.z;

												p3.x = p4.x;
												p3.y = tree_top;
												p3.z = p4.z;

												if (get_3d_line_quad_face_intersection(source_old_position, source_new_position, &p1, &p2, &p3, &p4, source_intercept_point, face_normal))
												{
													if (tree_range < nearest_tree_range)
													{
														hit_tree = TRUE;

														nearest_tree_range = tree_range;

														nearest_tree_intercept_point = *source_intercept_point;

														nearest_tree_face_normal = *face_normal;

														continue;
													}
												}

												p1.x = tree_position.x;
												p1.y = tree_position.y;
												p1.z = tree_position.z - (TREE_RADIUS * 0.5);

												p4.x = tree_position.x;
												p4.y = tree_position.y;
												p4.z = tree_position.z + (TREE_RADIUS * 0.5);

												p2.x = p1.x;
												p2.y = tree_top;
												p2.z = p1.z;

												p3.x = p4.x;
												p3.y = tree_top;
												p3.z = p4.z;

												if (get_3d_line_quad_face_intersection(source_old_position, source_new_position, &p1, &p2, &p3, &p4, source_intercept_point, face_normal))
												{
													if (tree_range < nearest_tree_range)
													{
														hit_tree = TRUE;

														nearest_tree_range = tree_range;

														nearest_tree_intercept_point = *source_intercept_point;

														nearest_tree_face_normal = *face_normal;

														continue;
													}
												}

												if (!skip_tree_top_test)
												{
													p1.x = tree_position.x - (TREE_RADIUS * 0.5);
													p1.y = tree_top;
													p1.z = tree_position.z - (TREE_RADIUS * 0.5);

													p2.x = tree_position.x - (TREE_RADIUS * 0.5);
													p2.y = tree_top;
													p2.z = tree_position.z + (TREE_RADIUS * 0.5);

													p3.x = tree_position.x + (TREE_RADIUS * 0.5);
													p3.y = tree_top;
													p3.z = tree_position.z + (TREE_RADIUS * 0.5);

													p4.x = tree_position.x + (TREE_RADIUS * 0.5);
													p4.y = tree_top;
													p4.z = tree_position.z - (TREE_RADIUS * 0.5);

													if (get_3d_line_quad_face_intersection(source_old_position, source_new_position, &p1, &p2, &p3, &p4, source_intercept_point, face_normal))
													{
														if (tree_range < nearest_tree_range)
														{
															hit_tree = TRUE;

															nearest_tree_range = tree_range;

															nearest_tree_intercept_point = *source_intercept_point;

															nearest_tree_face_normal = *face_normal;

															continue;
														}
													}
												}
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}

	if (hit_tree)
	{
		*source_intercept_point = nearest_tree_intercept_point;

		*face_normal = nearest_tree_face_normal;
	}

	return (hit_tree);
}

/* the answer: the original's result with the point and normal as they stand (the caller's own on a miss that wrote nothing) */
static int finish(eech_tree_query *q, int result, eech_tree_request *out)
{
	q->awaiting = AWAIT_NONE;

	if (result == TREE_FAULT)
	{
		return (TREE_FAULT);
	}

	out->state = TREE_DONE;
	out->hit = result ? 1 : 0;
	out->point[0] = q->intercept_point.x;
	out->point[1] = q->intercept_point.y;
	out->point[2] = q->intercept_point.z;
	out->normal[0] = q->face_normal.x;
	out->normal[1] = q->face_normal.y;
	out->normal[2] = q->face_normal.z;

	return (TREE_OK);
}

static int after_old_elevation(eech_tree_query *q, const eech_tree_scene *scene, eech_tree_request *out)
{
	if (point_inside_map_area(scene, &q->new_position))
	{
		q->awaiting = AWAIT_NEW_ELEVATION;
		out->state = TREE_NEED_TERRAIN;
		out->endpoint = 1;
		out->x = q->new_position.x;
		out->z = q->new_position.z;
		return (TREE_OK);
	}

	return (finish(q, scan(q, scene, q->old_rad_alt, q->old_rad_alt), out));
}

/*
 * Starts a query: the segment from `old_position` to `new_position`, the two flags, and the intercept point and normal the caller passes in
 * (which a miss leaves, or partly overwrites, exactly as the original's out parameters). `out` says what is needed next. Replaces any open query.
 */
int eech_tree_begin(eech_tree_query *q, const eech_tree_scene *scene, const float *old_position, const float *new_position, const float *intercept_point,
	const float *face_normal, int ignore_forest_edge_trees, int skip_tree_top_test, eech_tree_request *out)
{
	memset(out, 0, sizeof(*out));
	memset(q, 0, sizeof(*q));

	q->old_position.x = old_position[0];
	q->old_position.y = old_position[1];
	q->old_position.z = old_position[2];
	q->new_position.x = new_position[0];
	q->new_position.y = new_position[1];
	q->new_position.z = new_position[2];
	q->intercept_point.x = intercept_point[0];
	q->intercept_point.y = intercept_point[1];
	q->intercept_point.z = intercept_point[2];
	q->face_normal.x = face_normal[0];
	q->face_normal.y = face_normal[1];
	q->face_normal.z = face_normal[2];
	q->ignore_forest_edge_trees = ignore_forest_edge_trees;
	q->skip_tree_top_test = skip_tree_top_test;

	if (point_inside_map_area(scene, &q->old_position))
	{
		q->awaiting = AWAIT_OLD_ELEVATION;
		out->state = TREE_NEED_TERRAIN;
		out->endpoint = 0;
		out->x = q->old_position.x;
		out->z = q->old_position.z;
		return (TREE_OK);
	}

	q->old_rad_alt = 0.0;

	return (after_old_elevation(q, scene, out));
}

/*
 * The campaign's elevation for the lookup just asked about. Returns TREE_OK, TREE_OUT_OF_SEQUENCE when no lookup is pending (nothing changes), or
 * TREE_FAULT when the scene does not cover a sector the scan reaches (the query is closed).
 */
int eech_tree_terrain(eech_tree_query *q, const eech_tree_scene *scene, float elevation, eech_tree_request *out)
{
	memset(out, 0, sizeof(*out));

	if (q->awaiting == AWAIT_OLD_ELEVATION)
	{
		q->old_rad_alt = q->old_position.y - elevation;

		return (after_old_elevation(q, scene, out));
	}

	if (q->awaiting == AWAIT_NEW_ELEVATION)
	{
		float source_new_position_rad_alt;

		source_new_position_rad_alt = q->new_position.y - elevation;

		return (finish(q, scan(q, scene, q->old_rad_alt, source_new_position_rad_alt), out));
	}

	return (TREE_OUT_OF_SEQUENCE);
}
