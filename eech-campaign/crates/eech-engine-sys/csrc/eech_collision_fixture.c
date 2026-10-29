/*
 * TEST-ONLY object and tree collision characterisation (EEDCS-49, D3 G3a). Compiled only with the `collision-fixture` feature; never
 * part of the shipping module. It changes no production source: it calls the ORIGINAL get_line_of_sight_collision_entity and
 * get_line_of_sight_collision_tree (en_cllsn.c), the functions the compiled consumers call (mb_hostlos.c and mb_tgt.c object rays;
 * the wn_move.c COLLISION and TREES lookups), with the same call shape those consumers use (the intercept point zeroed first; the
 * face normal left unwritten; the tree query flags FALSE, TRUE), against the sector lists, 3D object database and tree data the
 * campaign's own boot loaded. Nothing here is a second implementation: every answer is the original function's, and every
 * expectation is therefore a measurement, never a table written by hand.
 *
 * What it observes, per query:
 *   - hit or miss, the kind of the entity hit (and whether it is a hosted roster member), the hit entity's position, the intercept
 *     point and the face normal, all as float bit patterns;
 *   - whether the query wrote the intercept point or the face normal on a MISS (the contract calls both invalid then; the consumers
 *     report the point to the host even for a miss, so what a miss leaves in it is a physical fact);
 *   - whether the query touched the shared terrain point record (terrelev.c system_point_data: both functions can: the tree query reads
 *     the terrain elevation at both ends, an entity hit on a terrain-aligned fixed object asks the terrain face normal). Before each
 *     COLD query the record is set to a known sentinel; the query is then repeated WARM on exactly the record it left, and a warm
 *     answer that differs from the cold one is a cache-state fact, counted, never hidden;
 *   - for a hit, whether a NEARER hit exists: the same query is repeated with the ray cut 0.1 percent short of the reported point, and a
 *     hit there proves the first answer was not the nearest. This is the original function asked a second question, not a model of it.
 *
 * The cases are two groups:
 *   STRUCTURED: a finite set that is a pure function of the loaded world (no random draw). Targets are real entities taken from the real
 *   sector lists in scan order, the first PER_CLASS of each class (moving mobile, stationary mobile, fixed object, city building, each by
 *   whether the object model has collision geometry triangles or only a bounding box), and real trees taken from the real tree sectors
 *   in scan order, the first PER_CLASS of each class (quad-plane tree, geometry tree). Rays are placed relative to the real target
 *   (through its centre both ways, a lateral miss, over it) and the same ray is repeated with the source and target exclusion slots
 *   set to the target, a decoy, the city of a building, and the hits already found (which enumerates the objects on a ray in the
 *   original's scan order). Trees get through, reverse, over, under, lateral, slant, high (above the tree-height gate), vertical and
 *   sub-0.1 m rays, and the through ray under all four combinations of the two tree flags.
 *   RANDOM: `entity_cases` and `tree_cases` seeded cases around entities and trees drawn from a private reservoir sample of the real
 *   lists. The generator is a private xorshift; the game's random stream is read before and after and must be unchanged.
 *
 * Three kinds of fact are kept apart in the report, as in the ground fixture (docs/m5-collision-characterization.md):
 *   - PHYSICAL (`col-physical`): the inputs, the exclusion role, the hit flag, the identity of the hit (kind, hosted id text, position),
 *     the intercept point (always, consumer shape) and the face normal (hits only). Entity heap indices and pointers are never in it.
 *   - CACHE and SHORT-CIRCUIT counters: record touched, warm agreement, miss-time writes, nearest checks. Counters only.
 *   - LEGACY (`col-legacy`): the shared record's private bookkeeping and the hit entity's heap slot. They exist only because the
 *     original structures exist, and only compare between two builds that keep them.
 *
 * Preconditions (a probe that does not hold them is refused with its reason in the report, before anything is altered):
 *   - the FPU is in round-toward-zero mode, as inside every engine entry;
 *   - all six floating-point exceptions are MASKED in BOTH the x87 control word (bits 0 to 5) and the SSE MXCSR (bits 7 to 12), read with
 *     fnstcw and stmxcsr and never written; only those mask bits are inspected, not the sticky status flags, the rounding mode or the
 *     precision. The original tree query divides by the segment's x and z extent (en_cllsn.c 2021 and 2024), so a zero extent is an
 *     IEEE division by zero that traps unless it is masked. The probe REQUIRES the masks; it never sets them. On an architecture whose
 *     control registers this file cannot read the probe is refused;
 *   - a terrain is loaded and its extents equal the world map's;
 *   - the entity sector map exists, and every collision candidate in every sector list has a valid 3D shape and bounds (the original
 *     asserts and then reads them; a world with an invalid one is refused here instead of being run into the engine);
 *   - the shared terrain point record can be located through the public accessors.
 * Never probed (declared, not silently skipped): NaN or infinite coordinates; a PRIMARY random tree ray whose f32 x or z extent is
 * exactly zero (counted as skipped_axis; this keeps the random case set what it was); anything while draw_collision_boxes is set.
 * Zero-extent rays that ARE queried, counted and not screened: the deliberate vertical and sub-0.1 m structured tree rays (the 2D range
 * gate returns before the division, and at large coordinates the 0.05 m offset can round to zero extent), and the DERIVED shortened
 * nearer-check ray of a tree hit, whose f32 end can have zero x or z extent yet a range above 0.1 m and so reach the division;
 * `derived_axis` counts them. With the masks required above, that division is IEEE-defined (inf or NaN, no trap). Source checks of the
 * axis extents alone are not a statement of the original's full physics.
 *
 * State the probe changes, and what is checked about it (this is a bounded set of observations, not a proof of purity):
 *   - the shared terrain record: snapshot before, byte-for-byte restore after, both asserted (record_restored);
 *   - the game random stream: its seed word is read before and after (rng_unchanged);
 *   - the three bounded UPDATE-FLAG bit arrays (up_update.c: updated_entities, moved_entities, rotated_entities, MAX_NUM_ENTITIES / 32 words
 *     each). The ORIGINAL getter get_local_entity_int_value (INT_TYPE_UPDATED, INT_TYPE_MOVED or INT_TYPE_ROTATED) is not a read: mb_int.c does
 *     `value = updated_entities [index] &= mask`, clearing every OTHER bit of the entity's 32-entity word. The original moving-target
 *     collision calls it (en_cllsn.c), so the original queries really do change these arrays, and those effects stay in the measured query
 *     sequence unchanged. This file therefore NEVER calls that getter: its own observation reads the raw bit
 *     `(array [index >> 5] >> (index & 31)) & 1`. The three arrays are copied in full before anything is observed or queried, the
 *     effect of the original queries on them is measured against that copy and reported as LEGACY diagnostics (col-flags), and the copy is
 *     written back before the final fingerprint (the arrays then equal the copy, asserted as flags_restored). This restoration is for the
 *     extra observer only; it is not a production semantic and no actor state other than these arrays is restored;
 *   - a SECTOR-LIST FINGERPRINT (world_before/world_after), taken with raw reads only and the final one AFTER the flag restoration: in
 *     sector order, per sector a marker, then per sector-list entity in list order its type and collision-candidate flags, and for a
 *     candidate its position and, for a mobile, its motion vector and the RAW updated bit, plus the frame delta time. It does NOT
 *     traverse the buildings of a city, hash any per-entity identity, or cover any other field; equal fingerprints are not a proof that no
 *     state changed. A fingerprint difference is a finding and is never suppressed by restoring more state.
 * The queries call no host. No entity is created, moved or destroyed by this file.
 *
 * Counters: warm_same and warm_different compare only the PRIMARY cold query with its warm repeat; the warm repeat of the shortened
 * nearer-check ray is made but not compared.
 */

#ifdef EECH_COLLISION_FIXTURE

#include <fenv.h>
#include <math.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

#define FNV_OFFSET 14695981039346656037ull
#define FNV_PRIME 1099511628211ull

#define SENTINEL_BITS 0x5a5a5a5au
#define SENTINEL_SECTOR (-1)

#define PER_CLASS 3
#define POOL_MAX 1024
#define SAMPLES_HIT 16
#define SAMPLES_MISS 8

enum { G_ENTITY_STRUCT = 0, G_ENTITY_RANDOM = 1, G_TREE_STRUCT = 2, G_TREE_RANDOM = 3, G_COUNT = 4 };

/* entity target classes */
enum
{
	CLS_MOVING_GEO, CLS_MOVING_BOX, CLS_STATIONARY_GEO, CLS_STATIONARY_BOX, CLS_FIXED_GEO, CLS_FIXED_BOX, CLS_BUILDING_GEO, CLS_BUILDING_BOX, CLS_COUNT
};

/* entity ray kinds */
/* EK_EXCLUSION is every follow-up query on a through ray (exclusion roles and the hit chain), kept apart so EK_THROUGH counts only the one primary ray per target */
enum { EK_THROUGH, EK_REVERSE, EK_LATERAL, EK_ABOVE, EK_EXCLUSION, EK_EDGE, EK_RANDOM, EK_COUNT };
static const char *const entity_kind_name[EK_COUNT] = { "through", "reverse", "lateral", "above", "exclusion", "edge", "random" };

/* tree ray kinds */
enum { TK_THROUGH, TK_REVERSE, TK_OVER, TK_UNDER, TK_LATERAL, TK_SLANT, TK_HIGH, TK_VERTICAL, TK_SHORT, TK_EDGE, TK_RANDOM, TK_COUNT };
static const char *const tree_kind_name[TK_COUNT] = { "through", "reverse", "over", "under", "lateral", "slant", "high", "vertical", "short", "edge", "random" };

/* exclusion roles of an entity query (folded as a number; a pointer never is) */
enum
{
	ROLE_NONE = 0, ROLE_SRC_TARGET = 1, ROLE_TGT_TARGET = 2, ROLE_BOTH_TARGET = 3, ROLE_SRC_DECOY = 4, ROLE_TGT_DECOY = 5, ROLE_SRC_CITY = 6, ROLE_TGT_CITY = 7,
	ROLE_CHAIN_SRC = 8, ROLE_CHAIN_BOTH = 9, ROLE_NEARER = 32
};

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* hashing, report text, private generator */

static uint64_t fold64 (uint64_t hash, uint32_t word)
{
	int byte;
	for (byte = 0; byte < 4; byte++)
	{
		hash ^= (uint64_t) ((word >> (8 * byte)) & 0xffu);
		hash *= FNV_PRIME;
	}
	return hash;
}

static uint32_t float_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static float bits_float (uint32_t bits)
{
	float value;
	memcpy (&value, &bits, sizeof (value));
	return value;
}

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

static uint32_t next_random (uint32_t *state)
{
	uint32_t x = *state;
	x ^= x << 13;
	x ^= x >> 17;
	x ^= x << 5;
	*state = x;
	return x;
}

static float unit (uint32_t *state)
{
	return (float) (next_random (state) & 0xffffffu) / 16777216.0f;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the shared terrain point record, reached as in the ground fixture */

static terrain_3d_point_data *shared_record (void)
{
	return (terrain_3d_point_data *) ((char *) get_3d_terrain_point_data_normal (NULL) - offsetof (terrain_3d_point_data, face_normal));
}

static void make_sentinel (terrain_3d_point_data *sentinel)
{
	memset (sentinel, 0, sizeof (*sentinel));
	sentinel->x_sector = SENTINEL_SECTOR;
	sentinel->z_sector = SENTINEL_SECTOR;
	sentinel->last_elevation = bits_float (SENTINEL_BITS);
	sentinel->terrain_triangle_offset = SENTINEL_SECTOR;
	sentinel->terrain_type = TERRAIN_TYPE_INVALID;
}

struct col_record_view
{
	int touched;
	uint32_t elevation_bits, normal_bits[3];
	int type, x_sector, z_sector, triangle_offset, sector_held, face_held;
};

static void view_record (const terrain_3d_point_data *record, const terrain_3d_point_data *sentinel, struct col_record_view *view)
{
	view->touched = memcmp (record, sentinel, sizeof (*record)) != 0;
	view->elevation_bits = float_bits (record->last_elevation);
	view->normal_bits[0] = float_bits (record->face_normal.x);
	view->normal_bits[1] = float_bits (record->face_normal.y);
	view->normal_bits[2] = float_bits (record->face_normal.z);
	view->type = (int) record->terrain_type;
	view->x_sector = record->x_sector;
	view->z_sector = record->z_sector;
	view->triangle_offset = record->terrain_triangle_offset;
	view->sector_held = record->terrain_sector != NULL;
	view->face_held = record->terrain_face != NULL;
}

static uint64_t record_words_hash (const terrain_3d_point_data *record)
{
	struct col_record_view view;
	uint64_t hash = FNV_OFFSET;
	view_record (record, record, &view);
	hash = fold64 (hash, view.elevation_bits);
	hash = fold64 (hash, view.normal_bits[0]);
	hash = fold64 (hash, view.normal_bits[1]);
	hash = fold64 (hash, view.normal_bits[2]);
	hash = fold64 (hash, (uint32_t) view.type);
	hash = fold64 (hash, (uint32_t) view.x_sector);
	hash = fold64 (hash, (uint32_t) view.z_sector);
	hash = fold64 (hash, (uint32_t) view.triangle_offset);
	hash = fold64 (hash, (uint32_t) (view.sector_held | (view.face_held << 1)));
	return hash;
}

/* terrain height used to PLACE edge rays, from a private record, never the shared one */
static float terrain_height (float x, float z)
{
	terrain_3d_point_data point;
	memset (&point, 0, sizeof (point));
	return get_3d_terrain_point_data (x, z, &point);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* probe state */

struct col_target
{
	entity *en;
	entity *city; /* the owning city of a building, else NULL */
	int klass;
};

struct col_treeref
{
	int sx, sz, index;
};

struct col_census
{
	int list_entities, mobile_candidates, mobile_moving, mobile_stationary, fixed_plain, cities, cities_with_buildings, buildings, listed_buildings, geometry_path, box_path,
		align_terrain, invalid_shape;
	int tree_data, trees_type0, trees_type1, sectors_with_trees, forest_geometry;
	long trees_total;
};

struct col_common
{
	int cases, hit, miss, record_touched, warm_same, warm_different, miss_point_written, miss_normal_written, input_mutated, nearer_checked, nearer_exists, skipped_axis, derived_axis;
};

struct col_kinds
{
	int cases, hit;
};

struct col_exclusion
{
	int through_cases, through_hit_target, through_hit_other, base_cases, src_removed, tgt_removed, both_removed, decoy_cases, decoy_unchanged, building_base, building_src_removed,
		building_city_src_removed, building_city_tgt_removed, chain_cases, chain_depth1, chain_depth2, chain_depth3, chain_stalled, chain_order_violations;
};

/* the bounded size of each update-flag array, as up_update.c defines it (the header declares them unsized, so sizeof is not available here) */
#define COL_FLAG_WORDS ((int) (MAX_NUM_ENTITIES / (sizeof (unsigned int) * 8)))
#define COL_FLAG_ARRAYS 3

struct col_flag_effect
{
	int changed_words[COL_FLAG_ARRAYS], cleared_bits[COL_FLAG_ARRAYS], added_bits;
	uint64_t before, after_queries, after_restore;
	int restored;
};

struct col_probe
{
	uint32_t rng, pool_rng;
	unsigned int flags_saved[COL_FLAG_ARRAYS][COL_FLAG_WORDS]; /* [0] updated, [1] moved, [2] rotated: the copy taken before anything is observed */
	struct col_flag_effect flags;
	terrain_3d_point_data *record, sentinel;
	uint64_t phys[G_COUNT], legacy[G_COUNT];
	struct out *out;
	struct col_census census;

	struct col_target sel[CLS_COUNT][PER_CLASS];
	int sel_n[CLS_COUNT];
	int class_seen[CLS_COUNT]; /* every candidate of the class in the sector lists, not only the first PER_CLASS */
	struct col_target pool[POOL_MAX];
	int pool_n;
	long pool_seen;

	struct col_treeref tsel[2][PER_CLASS]; /* [0] quad-plane trees, [1] geometry trees */
	int tsel_n[2];
	int tclass_seen[2]; /* every tree of the class in the tree sectors, not only the first PER_CLASS */
	struct col_treeref tpool[POOL_MAX];
	int tpool_n;
	long tpool_seen;

	struct col_common ecommon[2]; /* [0] structured, [1] random */
	struct col_common tcommon[2];
	struct col_kinds ekinds[2][EK_COUNT];
	struct col_kinds tkinds[2][TK_COUNT];
	struct col_kinds tflags[2][2][2]; /* [class][ignore_edge][skip_top] */
	struct col_exclusion excl;
	int type_hits[2][NUM_ENTITY_TYPES];

	int samples_hit[2], samples_miss[2]; /* [0] entity, [1] tree */
};

static struct col_probe probe_storage;

static void pfold (struct col_probe *p, int group, uint32_t word)
{
	p->phys[group] = fold64 (p->phys[group], word);
}

static void pfold_vec (struct col_probe *p, int group, const vec3d *v)
{
	pfold (p, group, float_bits (v->x));
	pfold (p, group, float_bits (v->y));
	pfold (p, group, float_bits (v->z));
}

static void lfold (struct col_probe *p, int group, uint32_t word)
{
	p->legacy[group] = fold64 (p->legacy[group], word);
}

static void lfold_view (struct col_probe *p, int group, const struct col_record_view *view)
{
	lfold (p, group, (uint32_t) view->touched);
	lfold (p, group, view->elevation_bits);
	lfold (p, group, view->normal_bits[0]);
	lfold (p, group, view->normal_bits[1]);
	lfold (p, group, view->normal_bits[2]);
	lfold (p, group, (uint32_t) view->type);
	lfold (p, group, (uint32_t) view->x_sector);
	lfold (p, group, (uint32_t) view->z_sector);
	lfold (p, group, (uint32_t) view->triangle_offset);
	lfold (p, group, (uint32_t) (view->sector_held | (view->face_held << 1)));
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the model facts the original reads, asked the way it asks them */

static int has_collision_geometry (int object_index)
{
	vec3d points[8];
	return get_object_3d_collision_object_geometry_triangle ((object_3d_index_numbers) object_index, points, 0) ? 1 : 0;
}

static int forest_tree_has_geometry (void)
{
	vec3d points[8];
	return get_object_3d_collision_object_geometry_triangle (OBJECT_3D_FOREST_TREE_OBJECT, points, 0) ? 1 : 0;
}

/* a collision candidate's shape must be valid before the original is run: it asserts, then reads it */
static int shape_valid (entity *en)
{
	int index = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
	if (index == OBJECT_3D_INVALID_OBJECT_INDEX || index <= 0 || index >= OBJECT_3D_LAST)
	{
		return 0;
	}
	return get_object_3d_bounding_box_without_lines ((object_3d_index_numbers) index) != NULL;
}

static void add_pool (struct col_probe *p, entity *en, entity *city, int klass)
{
	p->pool_seen++;
	if (p->pool_n < POOL_MAX)
	{
		p->pool[p->pool_n].en = en;
		p->pool[p->pool_n].city = city;
		p->pool[p->pool_n].klass = klass;
		p->pool_n++;
	}
	else
	{
		uint32_t j = next_random (&p->pool_rng) % (uint32_t) p->pool_seen;
		if (j < POOL_MAX)
		{
			p->pool[j].en = en;
			p->pool[j].city = city;
			p->pool[j].klass = klass;
		}
	}
}

static void add_selection (struct col_probe *p, entity *en, entity *city, int klass)
{
	add_pool (p, en, city, klass);
	p->class_seen[klass]++;
	if (p->sel_n[klass] < PER_CLASS)
	{
		p->sel[klass][p->sel_n[klass]].en = en;
		p->sel[klass][p->sel_n[klass]].city = city;
		p->sel[klass][p->sel_n[klass]].klass = klass;
		p->sel_n[klass]++;
	}
}

static void count_path (struct col_census *c, int object_index)
{
	if (has_collision_geometry (object_index))
	{
		c->geometry_path++;
	}
	else
	{
		c->box_path++;
	}
}

/*
 * One pass over the real sector lists, read-only: the census, the structured selection (the first PER_CLASS of each class in scan order:
 * sector x outer, z inner, list head to tail) and the reservoir pool for the random cases. Invalid shapes are only counted here; the caller
 * refuses the probe if there are any.
 */
static void scan_entities (struct col_probe *p)
{
	struct col_census *c = &p->census;
	int x, z;

	for (x = 0; x < NUM_MAP_X_SECTORS; x++)
	{
		for (z = 0; z < NUM_MAP_Z_SECTORS; z++)
		{
			entity *sector = entity_sector_map[x + (z * NUM_MAP_X_SECTORS)];
			entity *en;

			if (!sector)
			{
				continue;
			}
			for (en = get_local_entity_first_child (sector, LIST_TYPE_SECTOR); en; en = get_local_entity_child_succ (en, LIST_TYPE_SECTOR))
			{
				c->list_entities++;
				if (get_local_entity_int_value (en, INT_TYPE_COLLISION_TEST_MOBILE))
				{
					vec3d *motion;
					float sqr_velocity;
					int index, moving, geometry;

					c->mobile_candidates++;
					if (!shape_valid (en))
					{
						c->invalid_shape++;
						continue;
					}
					index = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
					motion = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_MOTION_VECTOR);
					sqr_velocity = (motion->x * motion->x) + (motion->y * motion->y) + (motion->z * motion->z);
					moving = sqr_velocity > 1.0;
					geometry = has_collision_geometry (index);
					count_path (c, index);
					if (moving)
					{
						c->mobile_moving++;
					}
					else
					{
						c->mobile_stationary++;
					}
					add_selection (p, en, NULL, moving ? (geometry ? CLS_MOVING_GEO : CLS_MOVING_BOX) : (geometry ? CLS_STATIONARY_GEO : CLS_STATIONARY_BOX));
				}
				else if (get_local_entity_int_value (en, INT_TYPE_COLLISION_TEST_FIXED))
				{
					if (get_local_entity_type (en) == ENTITY_TYPE_CITY)
					{
						entity *building = get_local_entity_first_child (en, LIST_TYPE_CITY_BUILDING);

						c->cities++;
						if (building)
						{
							c->cities_with_buildings++;
						}
						for (; building; building = get_local_entity_child_succ (building, LIST_TYPE_CITY_BUILDING))
						{
							int index, geometry;

							c->buildings++;
							if (!shape_valid (building))
							{
								c->invalid_shape++;
								continue;
							}
							index = get_local_entity_int_value (building, INT_TYPE_OBJECT_3D_SHAPE);
							geometry = has_collision_geometry (index);
							count_path (c, index);
							add_selection (p, building, en, geometry ? CLS_BUILDING_GEO : CLS_BUILDING_BOX);
						}
					}
					else if (get_local_entity_type (en) == ENTITY_TYPE_CITY_BUILDING)
					{
						/* a building listed in a sector directly: the original skips it (its city carries it) */
						c->listed_buildings++;
					}
					else
					{
						int index, geometry;

						c->fixed_plain++;
						if (!shape_valid (en))
						{
							c->invalid_shape++;
							continue;
						}
						index = get_local_entity_int_value (en, INT_TYPE_OBJECT_3D_SHAPE);
						geometry = has_collision_geometry (index);
						count_path (c, index);
						if (get_local_entity_int_value (en, INT_TYPE_ALIGN_WITH_TERRAIN))
						{
							c->align_terrain++;
						}
						add_selection (p, en, NULL, geometry ? CLS_FIXED_GEO : CLS_FIXED_BOX);
					}
				}
			}
		}
	}
}

/* the real tree sectors, read-only: counts, the structured selection (first PER_CLASS of each class in scan order) and the reservoir pool */
static void scan_trees (struct col_probe *p)
{
	struct col_census *c = &p->census;
	int geometry = forest_tree_has_geometry ();
	int x, z, i;

	c->forest_geometry = geometry;
	for (x = 0; x < terrain_3d_sector_x_max; x++)
	{
		for (z = 0; z < terrain_3d_sector_z_max; z++)
		{
			terrain_3d_tree_data *trees = NULL;
			int count = get_terrain_3d_tree_sector_data (x, z, &trees);

			if (count <= 0 || !trees)
			{
				continue;
			}
			c->sectors_with_trees++;
			for (i = 0; i < count; i++)
			{
				int klass = (trees[i].type == 1 && geometry) ? 1 : 0;

				p->tclass_seen[klass]++;
				c->trees_total++;
				if (trees[i].type == 1)
				{
					c->trees_type1++;
				}
				else
				{
					c->trees_type0++;
				}
				if (p->tsel_n[klass] < PER_CLASS)
				{
					p->tsel[klass][p->tsel_n[klass]].sx = x;
					p->tsel[klass][p->tsel_n[klass]].sz = z;
					p->tsel[klass][p->tsel_n[klass]].index = i;
					p->tsel_n[klass]++;
				}
				p->tpool_seen++;
				if (p->tpool_n < POOL_MAX)
				{
					p->tpool[p->tpool_n].sx = x;
					p->tpool[p->tpool_n].sz = z;
					p->tpool[p->tpool_n].index = i;
					p->tpool_n++;
				}
				else
				{
					uint32_t j = next_random (&p->pool_rng) % (uint32_t) p->tpool_seen;
					if (j < POOL_MAX)
					{
						p->tpool[j].sx = x;
						p->tpool[j].sz = z;
						p->tpool[j].index = i;
					}
				}
			}
		}
	}
	c->tree_data = c->trees_total > 0;
}

/*
 * The RAW updated bit of an entity. The original getter for INT_TYPE_UPDATED clears the other bits of the 32-entity word (mb_int.c: `&= mask`), so the observer
 * must not call it; this reads the bit with the original's own index and bit arithmetic (index >> 5, 1 << (index & 0x1f)) and writes nothing.
 */
static uint32_t raw_updated_bit (entity *en)
{
	int index = get_local_entity_index (en);

	return (uint32_t) ((updated_entities[index >> 5] >> (index & 0x1f)) & 1u);
}

/*
 * A SECTOR-LIST FINGERPRINT: in sector order a marker per sector, then per sector-list entity in list order its type and collision-candidate flags and,
 * for a candidate, its position and (a mobile) motion vector and RAW updated bit, plus the frame delta time. It reads with no destructive getter. It does not
 * traverse the buildings of a city and hashes no per-entity identity, so equal fingerprints before and after are a bounded observation, not a proof that
 * nothing changed.
 */
static uint64_t world_fold (void)
{
	uint64_t hash = FNV_OFFSET;
	int x, z;

	hash = fold64 (hash, float_bits ((float) get_delta_time ()));
	for (x = 0; x < NUM_MAP_X_SECTORS; x++)
	{
		for (z = 0; z < NUM_MAP_Z_SECTORS; z++)
		{
			entity *sector = entity_sector_map[x + (z * NUM_MAP_X_SECTORS)];
			entity *en;

			if (!sector)
			{
				continue;
			}
			hash = fold64 (hash, 0x53454300u);
			for (en = get_local_entity_first_child (sector, LIST_TYPE_SECTOR); en; en = get_local_entity_child_succ (en, LIST_TYPE_SECTOR))
			{
				int mobile = get_local_entity_int_value (en, INT_TYPE_COLLISION_TEST_MOBILE) != 0;
				int fixed = !mobile && get_local_entity_int_value (en, INT_TYPE_COLLISION_TEST_FIXED) != 0;

				hash = fold64 (hash, (uint32_t) get_local_entity_type (en));
				hash = fold64 (hash, (uint32_t) (mobile | (fixed << 1)));
				/* position and motion are read only for the collision candidates, the entities whose getters the original itself calls */
				if (mobile || fixed)
				{
					vec3d *position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

					hash = fold64 (hash, float_bits (position->x));
					hash = fold64 (hash, float_bits (position->y));
					hash = fold64 (hash, float_bits (position->z));
				}
				if (mobile)
				{
					vec3d *motion = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_MOTION_VECTOR);

					hash = fold64 (hash, float_bits (motion->x));
					hash = fold64 (hash, float_bits (motion->y));
					hash = fold64 (hash, float_bits (motion->z));
					hash = fold64 (hash, raw_updated_bit (en));
				}
			}
		}
	}
	return hash;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* one query: cold on a known record, then warm on the record it left */

struct col_qr
{
	entity *en;
	int hit;
	vec3d point, normal;
	int touched, point_written, normal_written, mutated;
	struct col_record_view view;
};

static int vec_is_zero (const vec3d *v)
{
	return float_bits (v->x) == 0u && float_bits (v->y) == 0u && float_bits (v->z) == 0u;
}

static int normal_is_sentinel (const vec3d *v)
{
	return float_bits (v->x) == SENTINEL_BITS && float_bits (v->y) == SENTINEL_BITS && float_bits (v->z) == SENTINEL_BITS;
}

static void begin_outputs (struct col_qr *r)
{
	memset (r, 0, sizeof (*r));
	/* the consumers' shape: the intercept point zeroed first, the face normal left unwritten (here a sentinel, so a write is seen) */
	r->normal.x = bits_float (SENTINEL_BITS);
	r->normal.y = bits_float (SENTINEL_BITS);
	r->normal.z = bits_float (SENTINEL_BITS);
}

static void end_outputs (struct col_probe *p, struct col_qr *r, const vec3d *a, const vec3d *b, const vec3d *a0, const vec3d *b0)
{
	r->touched = memcmp (p->record, &p->sentinel, sizeof (*p->record)) != 0;
	view_record (p->record, &p->sentinel, &r->view);
	r->mutated = memcmp (a, a0, sizeof (*a)) != 0 || memcmp (b, b0, sizeof (*b)) != 0;
	if (!r->hit)
	{
		r->point_written = !vec_is_zero (&r->point);
		r->normal_written = !normal_is_sentinel (&r->normal);
	}
}

static void query_entity (struct col_probe *p, entity *src, entity *tgt, const vec3d *a0, const vec3d *b0, int cold, struct col_qr *r)
{
	vec3d a = *a0, b = *b0;

	begin_outputs (r);
	if (cold)
	{
		memcpy (p->record, &p->sentinel, sizeof (*p->record));
	}
	r->en = get_line_of_sight_collision_entity (src, tgt, &a, &b, &r->point, &r->normal);
	r->hit = r->en != NULL;
	end_outputs (p, r, &a, &b, a0, b0);
}

static void query_tree (struct col_probe *p, const vec3d *a0, const vec3d *b0, int ignore_edge, int skip_top, int cold, struct col_qr *r)
{
	vec3d a = *a0, b = *b0;

	begin_outputs (r);
	if (cold)
	{
		memcpy (p->record, &p->sentinel, sizeof (*p->record));
	}
	r->hit = get_line_of_sight_collision_tree (&a, &b, &r->point, &r->normal, ignore_edge, skip_top) ? 1 : 0;
	r->en = NULL;
	end_outputs (p, r, &a, &b, a0, b0);
}

static int same_answer (const struct col_qr *x, const struct col_qr *y)
{
	return x->hit == y->hit && x->en == y->en && memcmp (&x->point, &y->point, sizeof (vec3d)) == 0 && memcmp (&x->normal, &y->normal, sizeof (vec3d)) == 0;
}

/* the hit's identity as stable semantic facts: kind, hosted id text, position */
static void fold_identity (struct col_probe *p, int group, entity *en)
{
	const char *id = campaign_roster_host_id (en);
	vec3d *position = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

	pfold (p, group, (uint32_t) get_local_entity_type (en));
	if (id)
	{
		while (*id)
		{
			pfold (p, group, (uint32_t) (unsigned char) *id++);
		}
		pfold (p, group, 0u);
	}
	else
	{
		pfold (p, group, 0xffffffffu);
	}
	pfold_vec (p, group, position);
}

static void fold_query (struct col_probe *p, int group, int role, const vec3d *a, const vec3d *b, const struct col_qr *r)
{
	pfold (p, group, (uint32_t) role);
	pfold_vec (p, group, a);
	pfold_vec (p, group, b);
	pfold (p, group, (uint32_t) r->hit);
	if (r->hit && r->en)
	{
		fold_identity (p, group, r->en);
	}
	pfold_vec (p, group, &r->point);
	if (r->hit)
	{
		pfold_vec (p, group, &r->normal);
	}
	lfold_view (p, group, &r->view);
	lfold (p, group, r->en ? (uint32_t) get_local_entity_index (r->en) : 0xffffffffu);
}

static void note_common (struct col_common *c, const struct col_qr *cold, const struct col_qr *warm)
{
	c->cases++;
	if (cold->hit)
	{
		c->hit++;
	}
	else
	{
		c->miss++;
		c->miss_point_written += cold->point_written;
		c->miss_normal_written += cold->normal_written;
	}
	c->record_touched += cold->touched;
	c->input_mutated += cold->mutated;
	if (same_answer (cold, warm))
	{
		c->warm_same++;
	}
	else
	{
		c->warm_different++;
	}
}

static void sample_entity (struct col_probe *p, int role, const vec3d *a, const vec3d *b, const struct col_qr *r)
{
	int *count = r->hit ? &p->samples_hit[0] : &p->samples_miss[0];
	int limit = r->hit ? SAMPLES_HIT : SAMPLES_MISS;

	if (*count >= limit)
	{
		return;
	}
	(*count)++;
	emit (p->out, "col-sample kind=entity role=%d hit=%d type=%d hosted=%d a=%08x,%08x,%08x b=%08x,%08x,%08x point=%08x,%08x,%08x normal=%08x,%08x,%08x\n", role, r->hit,
		r->en ? (int) get_local_entity_type (r->en) : -1, (r->en && campaign_roster_host_id (r->en)) ? 1 : 0, float_bits (a->x), float_bits (a->y), float_bits (a->z),
		float_bits (b->x), float_bits (b->y), float_bits (b->z), float_bits (r->point.x), float_bits (r->point.y), float_bits (r->point.z), float_bits (r->normal.x),
		float_bits (r->normal.y), float_bits (r->normal.z));
}

static void sample_tree (struct col_probe *p, int role, const vec3d *a, const vec3d *b, const struct col_qr *r)
{
	int *count = r->hit ? &p->samples_hit[1] : &p->samples_miss[1];
	int limit = r->hit ? SAMPLES_HIT : SAMPLES_MISS;

	if (*count >= limit)
	{
		return;
	}
	(*count)++;
	emit (p->out, "col-sample kind=tree role=%d hit=%d type=-1 hosted=0 a=%08x,%08x,%08x b=%08x,%08x,%08x point=%08x,%08x,%08x normal=%08x,%08x,%08x\n", role, r->hit,
		float_bits (a->x), float_bits (a->y), float_bits (a->z), float_bits (b->x), float_bits (b->y), float_bits (b->z), float_bits (r->point.x), float_bits (r->point.y),
		float_bits (r->point.z), float_bits (r->normal.x), float_bits (r->normal.y), float_bits (r->normal.z));
}

/* the end of the ray cut 0.1 percent short of the reported point: a hit on the shortened ray proves a nearer one existed */
static void shorten (const vec3d *a, const vec3d *point, vec3d *end)
{
	end->x = a->x + ((point->x - a->x) * 0.999f);
	end->y = a->y + ((point->y - a->y) * 0.999f);
	end->z = a->z + ((point->z - a->z) * 0.999f);
}

/* an entity query with everything recorded; returns the cold result in *out_cold */
static void run_entity (struct col_probe *p, int group, int kind, int role, entity *src, entity *tgt, const vec3d *a, const vec3d *b, int check_nearer, struct col_qr *out_cold)
{
	struct col_common *common = &p->ecommon[group == G_ENTITY_RANDOM ? 1 : 0];
	struct col_kinds *k = &p->ekinds[group == G_ENTITY_RANDOM ? 1 : 0][kind];
	struct col_qr cold, warm;

	query_entity (p, src, tgt, a, b, 1, &cold);
	query_entity (p, src, tgt, a, b, 0, &warm);
	fold_query (p, group, role, a, b, &cold);
	note_common (common, &cold, &warm);
	k->cases++;
	if (cold.hit)
	{
		k->hit++;
		if (cold.en)
		{
			int type = (int) get_local_entity_type (cold.en);

			if (type >= 0 && type < NUM_ENTITY_TYPES)
			{
				p->type_hits[group == G_ENTITY_RANDOM ? 1 : 0][type]++;
			}
		}
	}
	sample_entity (p, role, a, b, &cold);
	if (check_nearer && cold.hit)
	{
		vec3d end;
		struct col_qr near_cold, near_warm;

		shorten (a, &cold.point, &end);
		query_entity (p, src, tgt, a, &end, 1, &near_cold);
		query_entity (p, src, tgt, a, &end, 0, &near_warm);
		fold_query (p, group, role + ROLE_NEARER, a, &end, &near_cold);
		common->nearer_checked++;
		if (near_cold.hit)
		{
			common->nearer_exists++;
		}
	}
	if (out_cold)
	{
		*out_cold = cold;
	}
}

static void run_tree (struct col_probe *p, int group, int kind, int role, const vec3d *a, const vec3d *b, int ignore_edge, int skip_top, int check_nearer, struct col_qr *out_cold)
{
	struct col_common *common = &p->tcommon[group == G_TREE_RANDOM ? 1 : 0];
	struct col_kinds *k = &p->tkinds[group == G_TREE_RANDOM ? 1 : 0][kind];
	struct col_qr cold, warm;

	query_tree (p, a, b, ignore_edge, skip_top, 1, &cold);
	query_tree (p, a, b, ignore_edge, skip_top, 0, &warm);
	fold_query (p, group, role | (ignore_edge << 8) | (skip_top << 9), a, b, &cold);
	note_common (common, &cold, &warm);
	k->cases++;
	if (cold.hit)
	{
		k->hit++;
	}
	sample_tree (p, role, a, b, &cold);
	if (check_nearer && cold.hit)
	{
		vec3d end;
		struct col_qr near_cold, near_warm;

		shorten (a, &cold.point, &end);
		/* the derived ray is queried as it is, never screened; one whose f32 x or z extent is exactly zero reaches the original's division and is only counted */
		if (end.x - a->x == 0.0f || end.z - a->z == 0.0f)
		{
			common->derived_axis++;
		}
		query_tree (p, a, &end, ignore_edge, skip_top, 1, &near_cold);
		query_tree (p, a, &end, ignore_edge, skip_top, 0, &near_warm);
		fold_query (p, group, role + ROLE_NEARER, a, &end, &near_cold);
		common->nearer_checked++;
		if (near_cold.hit)
		{
			common->nearer_exists++;
		}
	}
	if (out_cold)
	{
		*out_cold = cold;
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* structured entity cases */

static float range_between (const vec3d *a, const vec3d *b)
{
	float dx = b->x - a->x, dy = b->y - a->y, dz = b->z - a->z;
	return (float) sqrt ((dx * dx) + (dy * dy) + (dz * dz));
}

/* bearings off the axes, by position in the class */
static const float bearings[4] = { 0.5235988f, 2.0943951f, 3.6651914f, 5.2359878f };

static void entity_struct_target (struct col_probe *p, const struct col_target *t, int n)
{
	vec3d *position = get_local_entity_vec3d_ptr (t->en, VEC3D_TYPE_POSITION);
	int index = get_local_entity_int_value (t->en, INT_TYPE_OBJECT_3D_SHAPE);
	float radius = get_object_3d_radius ((object_3d_index_numbers) index);
	object_3d_bounds *bounds = get_object_3d_bounding_box_without_lines ((object_3d_index_numbers) index);
	float bearing = bearings[n % 4];
	float sx = (float) sin (bearing), cz = (float) cos (bearing);
	float reach = radius + 20.0f;
	float shift = radius + 30.0f;
	float ymid = position->y + ((bounds->ymin + bounds->ymax) * 0.5f);
	vec3d a, b, a2, b2, a3, b3;
	struct col_qr base, r;
	entity *decoy = NULL;
	entity *exclusion_subject = t->en;
	int k;

	a.x = position->x - (sx * reach);
	a.y = ymid;
	a.z = position->z - (cz * reach);
	b.x = position->x + (sx * reach);
	b.y = ymid;
	b.z = position->z + (cz * reach);

	/* a real decoy: the first pool entity that is neither the target, its city, nor another building of that city */
	for (k = 0; k < p->pool_n; k++)
	{
		if (p->pool[k].en != t->en && p->pool[k].en != t->city && (t->city == NULL || p->pool[k].city != t->city))
		{
			decoy = p->pool[k].en;
			break;
		}
	}

	/* through the centre, then the same segment both ways round */
	run_entity (p, G_ENTITY_STRUCT, EK_THROUGH, ROLE_NONE, NULL, NULL, &a, &b, 1, &base);
	p->excl.through_cases++;
	if (base.hit && base.en == t->en)
	{
		p->excl.through_hit_target++;
		p->excl.base_cases++;
		run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_SRC_TARGET, exclusion_subject, NULL, &a, &b, 0, &r);
		if (!(r.hit && r.en == t->en))
		{
			p->excl.src_removed++;
		}
		run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_TGT_TARGET, NULL, exclusion_subject, &a, &b, 0, &r);
		if (!(r.hit && r.en == t->en))
		{
			p->excl.tgt_removed++;
		}
		run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_BOTH_TARGET, exclusion_subject, exclusion_subject, &a, &b, 0, &r);
		if (!(r.hit && r.en == t->en))
		{
			p->excl.both_removed++;
		}
		if (t->city)
		{
			/* a building is carried by its city: the original's exclusion test is on the sector-list entity, the city */
			p->excl.building_base++;
			/* excluding the building itself: the test is not on the building, so this is expected to leave it hit (measured, not assumed) */
			run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_SRC_TARGET, t->en, NULL, &a, &b, 0, &r);
			if (!(r.hit && r.en == t->en))
			{
				p->excl.building_src_removed++;
			}
			run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_SRC_CITY, t->city, NULL, &a, &b, 0, &r);
			if (!(r.hit && r.en == t->en))
			{
				p->excl.building_city_src_removed++;
			}
			run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_TGT_CITY, NULL, t->city, &a, &b, 0, &r);
			if (!(r.hit && r.en == t->en))
			{
				p->excl.building_city_tgt_removed++;
			}
		}
		if (decoy)
		{
			p->excl.decoy_cases++;
			run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_SRC_DECOY, decoy, NULL, &a, &b, 0, &r);
			if (r.hit && r.en == t->en)
			{
				p->excl.decoy_unchanged++;
			}
			run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_TGT_DECOY, NULL, decoy, &a, &b, 0, &r);
		}
	}
	else if (base.hit)
	{
		p->excl.through_hit_other++;
	}

	/* the objects the ray meets, in the original's scan order, by excluding each hit in turn (one slot each for source and target) */
	if (base.hit)
	{
		struct col_qr second, third;
		float d1, d2, d3;

		p->excl.chain_cases++;
		p->excl.chain_depth1++;
		d1 = range_between (&a, &base.point);
		run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_CHAIN_SRC, base.en, NULL, &a, &b, 0, &second);
		if (!second.hit)
		{
			/* nothing else on the ray */
		}
		else if (second.en == base.en)
		{
			p->excl.chain_stalled++;
		}
		else
		{
			p->excl.chain_depth2++;
			d2 = range_between (&a, &second.point);
			if (d2 < d1)
			{
				p->excl.chain_order_violations++;
			}
			run_entity (p, G_ENTITY_STRUCT, EK_EXCLUSION, ROLE_CHAIN_BOTH, base.en, second.en, &a, &b, 0, &third);
			if (third.hit && third.en != base.en && third.en != second.en)
			{
				p->excl.chain_depth3++;
				d3 = range_between (&a, &third.point);
				if (d3 < d2)
				{
					p->excl.chain_order_violations++;
				}
			}
			else if (third.hit)
			{
				p->excl.chain_stalled++;
			}
		}
	}

	run_entity (p, G_ENTITY_STRUCT, EK_REVERSE, ROLE_NONE, NULL, NULL, &b, &a, 1, &r);

	/* a lateral miss: the segment moved sideways past the broad-phase box of the target (radius plus 10) */
	a2 = a;
	b2 = b;
	a2.x += cz * shift;
	b2.x += cz * shift;
	a2.z -= sx * shift;
	b2.z -= sx * shift;
	run_entity (p, G_ENTITY_STRUCT, EK_LATERAL, ROLE_NONE, NULL, NULL, &a2, &b2, 1, &r);

	/* over it: above the target's box, where a fixed object is ignored by the sector's tallest-structure test */
	a3 = a;
	b3 = b;
	a3.y = position->y + radius + 50.0f;
	b3.y = a3.y;
	run_entity (p, G_ENTITY_STRUCT, EK_ABOVE, ROLE_NONE, NULL, NULL, &a3, &b3, 1, &r);
}

/* two rays across the map's own edge: the original clamps the scan to the map, so these are supported inputs */
static void entity_edge_cases (struct col_probe *p)
{
	vec3d a, b;
	struct col_qr r;
	float ha, hb;

	ha = terrain_height (MAX_MAP_X - 50.0f, MAX_MAP_Z - 50.0f);
	a.x = MAX_MAP_X - 50.0f;
	a.y = ha + 5.0f;
	a.z = MAX_MAP_Z - 50.0f;
	b.x = MAX_MAP_X + 400.0f;
	b.y = ha + 5.0f;
	b.z = MAX_MAP_Z + 400.0f;
	run_entity (p, G_ENTITY_STRUCT, EK_EDGE, ROLE_NONE, NULL, NULL, &a, &b, 1, &r);

	hb = terrain_height (MIN_MAP_X + 10.0f, MIN_MAP_Z + 10.0f);
	a.x = MIN_MAP_X - 400.0f;
	a.y = hb + 5.0f;
	a.z = MIN_MAP_Z - 300.0f;
	b.x = MIN_MAP_X + 60.0f;
	b.y = hb + 5.0f;
	b.z = MIN_MAP_Z + 50.0f;
	run_entity (p, G_ENTITY_STRUCT, EK_EDGE, ROLE_NONE, NULL, NULL, &a, &b, 1, &r);
}

static void entity_random_case (struct col_probe *p, int i)
{
	struct col_target *t;
	struct col_target *decoy;
	vec3d *position;
	vec3d a, b;
	float radius, bearing, sx, cz, length, offset, ya, yb;
	int index, role;
	entity *src = NULL, *tgt = NULL;
	static const float lengths[4] = { 40.0f, 150.0f, 600.0f, 2500.0f };
	struct col_qr r;

	(void) i;
	if (p->pool_n == 0)
	{
		return;
	}
	t = &p->pool[next_random (&p->rng) % (uint32_t) p->pool_n];
	decoy = &p->pool[next_random (&p->rng) % (uint32_t) p->pool_n];
	position = get_local_entity_vec3d_ptr (t->en, VEC3D_TYPE_POSITION);
	index = get_local_entity_int_value (t->en, INT_TYPE_OBJECT_3D_SHAPE);
	radius = get_object_3d_radius ((object_3d_index_numbers) index);
	bearing = unit (&p->rng) * 6.2831853f;
	sx = (float) sin (bearing);
	cz = (float) cos (bearing);
	length = lengths[next_random (&p->rng) % 4u];
	offset = (unit (&p->rng) - 0.5f) * 2.0f * (radius + 40.0f);
	ya = position->y + ((unit (&p->rng) - 0.25f) * (radius + 30.0f));
	yb = position->y + ((unit (&p->rng) - 0.25f) * (radius + 30.0f));
	a.x = position->x + (cz * offset) - (sx * length * 0.5f);
	a.y = ya;
	a.z = position->z - (sx * offset) - (cz * length * 0.5f);
	b.x = position->x + (cz * offset) + (sx * length * 0.5f);
	b.y = yb;
	b.z = position->z - (sx * offset) + (cz * length * 0.5f);
	role = (int) (next_random (&p->rng) % 5u);
	switch (role)
	{
	case 1:
		src = t->en;
		break;
	case 2:
		tgt = t->en;
		break;
	case 3:
		src = t->en;
		tgt = t->en;
		break;
	case 4:
		src = decoy->en;
		break;
	default:
		role = ROLE_NONE;
		break;
	}
	run_entity (p, G_ENTITY_RANDOM, EK_RANDOM, role == 4 ? ROLE_SRC_DECOY : role, src, tgt, &a, &b, 1, &r);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* tree cases */

static terrain_3d_tree_data *tree_at (const struct col_treeref *ref)
{
	terrain_3d_tree_data *trees = NULL;
	int count = get_terrain_3d_tree_sector_data (ref->sx, ref->sz, &trees);

	if (count <= ref->index || !trees)
	{
		return NULL;
	}
	return &trees[ref->index];
}

/* the tree's placement as the original computes it (the same expressions), used only to place the rays */
static void place_tree (const struct col_treeref *ref, const terrain_3d_tree_data *tree, vec3d *position, float *height)
{
	float x_mid = (ref->sx * TERRAIN_3D_SECTOR_SIDE_LENGTH) + (TERRAIN_3D_SECTOR_SIDE_LENGTH * 0.5);
	float z_mid = (ref->sz * TERRAIN_3D_SECTOR_SIDE_LENGTH) + (TERRAIN_3D_SECTOR_SIDE_LENGTH * 0.5);

	position->y = (tree->y * terrain_3d_map_scaled_height_difference * 0.5) + terrain_3d_map_minimum_height;
	position->x = x_mid + (tree->x * TERRAIN_3D_XZ_SCALE * 0.5);
	position->z = z_mid + (tree->z * TERRAIN_3D_XZ_SCALE * 0.5);
	*height = get_terrain_3d_tree_scale ((terrain_3d_tree_data *) tree);
}

static void tree_struct_target (struct col_probe *p, const struct col_treeref *ref, int klass, int n)
{
	terrain_3d_tree_data *tree = tree_at (ref);
	vec3d tp, a, b, a2, b2;
	float height, top, ymid, bearing, sx, cz;
	struct col_qr r;
	int ignore_edge, skip_top;

	if (!tree)
	{
		return;
	}
	place_tree (ref, tree, &tp, &height);
	top = tp.y + height;
	ymid = tp.y + (height * 0.5f);
	bearing = bearings[n % 4];
	sx = (float) sin (bearing);
	cz = (float) cos (bearing);

	a.x = tp.x - (sx * 30.0f);
	a.y = ymid;
	a.z = tp.z - (cz * 30.0f);
	b.x = tp.x + (sx * 30.0f);
	b.y = ymid;
	b.z = tp.z + (cz * 30.0f);

	/* the through ray under all four combinations of the two flags; (FALSE, TRUE) is the one every compiled consumer passes */
	for (ignore_edge = 0; ignore_edge < 2; ignore_edge++)
	{
		for (skip_top = 0; skip_top < 2; skip_top++)
		{
			int consumer = (ignore_edge == 0 && skip_top == 1);

			run_tree (p, G_TREE_STRUCT, TK_THROUGH, TK_THROUGH, &a, &b, ignore_edge, skip_top, consumer, &r);
			p->tflags[klass][ignore_edge][skip_top].cases++;
			if (r.hit)
			{
				p->tflags[klass][ignore_edge][skip_top].hit++;
			}
		}
	}
	run_tree (p, G_TREE_STRUCT, TK_REVERSE, TK_REVERSE, &b, &a, 0, 1, 1, &r);

	a2 = a;
	b2 = b;
	a2.y = top + 3.0f;
	b2.y = top + 3.0f;
	run_tree (p, G_TREE_STRUCT, TK_OVER, TK_OVER, &a2, &b2, 0, 1, 1, &r);

	a2.y = tp.y - 3.0f;
	b2.y = tp.y - 3.0f;
	run_tree (p, G_TREE_STRUCT, TK_UNDER, TK_UNDER, &a2, &b2, 0, 1, 1, &r);

	/* shifted sideways past the 25 m window around the tree */
	a2 = a;
	b2 = b;
	a2.x += cz * 40.0f;
	b2.x += cz * 40.0f;
	a2.z -= sx * 40.0f;
	b2.z -= sx * 40.0f;
	run_tree (p, G_TREE_STRUCT, TK_LATERAL, TK_LATERAL, &a2, &b2, 0, 1, 1, &r);

	/* rising across the tree */
	a2 = a;
	b2 = b;
	a2.y = tp.y - 1.0f;
	b2.y = top + 1.0f;
	run_tree (p, G_TREE_STRUCT, TK_SLANT, TK_SLANT, &a2, &b2, 0, 1, 1, &r);

	/* both ends far above the terrain: the first gate returns FALSE without a sector scan */
	a2 = a;
	b2 = b;
	a2.y = tp.y + 80.0f;
	b2.y = tp.y + 80.0f;
	run_tree (p, G_TREE_STRUCT, TK_HIGH, TK_HIGH, &a2, &b2, 0, 1, 1, &r);

	/* a vertical segment and a segment under 0.1 m in the plane: the 2D range gate returns FALSE before any division */
	a2.x = tp.x;
	a2.z = tp.z;
	a2.y = ymid + 10.0f;
	b2 = a2;
	b2.y = ymid - 10.0f;
	run_tree (p, G_TREE_STRUCT, TK_VERTICAL, TK_VERTICAL, &a2, &b2, 0, 1, 1, &r);

	a2.x = tp.x;
	a2.z = tp.z;
	a2.y = ymid;
	b2 = a2;
	b2.x += sx * 0.05f;
	b2.z += cz * 0.05f;
	run_tree (p, G_TREE_STRUCT, TK_SHORT, TK_SHORT, &a2, &b2, 0, 1, 1, &r);
}

static void tree_edge_cases (struct col_probe *p)
{
	vec3d a, b;
	struct col_qr r;
	float h;

	h = terrain_height (MAX_MAP_X - 50.0f, MAX_MAP_Z - 50.0f);
	a.x = MAX_MAP_X - 50.0f;
	a.y = h + 5.0f;
	a.z = MAX_MAP_Z - 50.0f;
	b.x = MAX_MAP_X + 400.0f;
	b.y = h + 5.0f;
	b.z = MAX_MAP_Z + 300.0f;
	run_tree (p, G_TREE_STRUCT, TK_EDGE, TK_EDGE, &a, &b, 0, 1, 1, &r);

	/* both ends off the map: the 'new end' branch copies the old end's height above ground (0) */
	a.x = MIN_MAP_X - 400.0f;
	a.y = 20.0f;
	a.z = MIN_MAP_Z - 300.0f;
	b.x = MIN_MAP_X - 100.0f;
	b.y = 25.0f;
	b.z = MIN_MAP_Z - 50.0f;
	run_tree (p, G_TREE_STRUCT, TK_EDGE, TK_EDGE, &a, &b, 0, 1, 1, &r);
}

static void tree_random_case (struct col_probe *p)
{
	struct col_treeref *ref;
	terrain_3d_tree_data *tree;
	vec3d tp, a, b;
	float height, bearing, sx, cz, length, offset, ya, yb;
	static const float lengths[3] = { 40.0f, 200.0f, 800.0f };
	struct col_qr r;
	int consumer_flags;

	if (p->tpool_n == 0)
	{
		return;
	}
	ref = &p->tpool[next_random (&p->rng) % (uint32_t) p->tpool_n];
	tree = tree_at (ref);
	if (!tree)
	{
		return;
	}
	place_tree (ref, tree, &tp, &height);
	bearing = unit (&p->rng) * 6.2831853f;
	sx = (float) sin (bearing);
	cz = (float) cos (bearing);
	length = lengths[next_random (&p->rng) % 3u];
	offset = (unit (&p->rng) - 0.5f) * 60.0f;
	ya = tp.y + ((unit (&p->rng) * (height + 10.0f)) - 5.0f);
	yb = tp.y + ((unit (&p->rng) * (height + 10.0f)) - 5.0f);
	a.x = tp.x + (cz * offset) - (sx * length * 0.5f);
	a.y = ya;
	a.z = tp.z - (sx * offset) - (cz * length * 0.5f);
	b.x = tp.x + (cz * offset) + (sx * length * 0.5f);
	b.y = yb;
	b.z = tp.z - (sx * offset) + (cz * length * 0.5f);
	consumer_flags = (next_random (&p->rng) % 4u) != 0u;
	if (b.x - a.x == 0.0f || b.z - a.z == 0.0f)
	{
		/* never probed: the similar-triangle test divides by these extents */
		p->tcommon[1].skipped_axis++;
		return;
	}
	if (consumer_flags)
	{
		run_tree (p, G_TREE_RANDOM, TK_RANDOM, TK_RANDOM, &a, &b, 0, 1, 1, &r);
	}
	else
	{
		/* one draw per statement: the order in which a call's arguments are evaluated is unspecified, and the random case set must be the same for the same seed */
		int ignore_edge = (int) (next_random (&p->rng) & 1u);
		int skip_top = (int) (next_random (&p->rng) & 1u);

		run_tree (p, G_TREE_RANDOM, TK_RANDOM, TK_RANDOM, &a, &b, ignore_edge, skip_top, 1, &r);
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the report */

/*
 * The floating-point exception MASKS, read and never written. x87 control word bits 0 to 5 (invalid, denormal, zero-divide, overflow, underflow,
 * precision) and SSE MXCSR bits 7 to 12 are the masks; a set bit means the exception is masked. Only those bits are returned: the sticky status flags,
 * the rounding mode and the precision control are not looked at. Returns 0 where this file cannot read the registers.
 */
#if defined(__x86_64__) || defined(__i386__)
#ifdef __x86_64__
#define COL_FP_ARCH_NAME "x86_64"
#else
#define COL_FP_ARCH_NAME "i386"
#endif
static int read_fp_masks (int *x87_masks, int *sse_masks)
{
	uint16_t control_word = 0;
	uint32_t mxcsr = 0;

	__asm__ __volatile__ ("fnstcw %0" : "=m" (control_word));
	__asm__ __volatile__ ("stmxcsr %0" : "=m" (mxcsr));
	*x87_masks = (int) (control_word & 0x3fu);
	*sse_masks = (int) ((mxcsr >> 7) & 0x3fu);
	return 1;
}
#else
#define COL_FP_ARCH_NAME "unsupported"
static int read_fp_masks (int *x87_masks, int *sse_masks)
{
	*x87_masks = 0;
	*sse_masks = 0;
	return 0;
}
#endif

static void refuse (char *text, int capacity, const char *reason)
{
	snprintf (text, (size_t) capacity, "col-refused reason=%s\n", reason);
}

static void sanitise (const char *name, char *to, size_t size)
{
	size_t i = 0;

	if (!name)
	{
		name = "unknown";
	}
	for (; *name && i + 1 < size; name++)
	{
		unsigned char c = (unsigned char) *name;

		to[i++] = ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9')) ? (char) c : '_';
	}
	if (i == 0)
	{
		to[i++] = '_';
	}
	to[i] = '\0';
}

static void emit_common (struct out *o, const char *event, const char *group, const struct col_common *c)
{
	emit (o, "%s group=%s cases=%d hit=%d miss=%d record_touched=%d warm_same=%d warm_different=%d miss_point_written=%d miss_normal_written=%d input_mutated=%d nearer_checked=%d "
		"nearer_exists=%d skipped_axis=%d derived_axis=%d\n", event, group, c->cases, c->hit, c->miss, c->record_touched, c->warm_same, c->warm_different, c->miss_point_written,
		c->miss_normal_written, c->input_mutated, c->nearer_checked, c->nearer_exists, c->skipped_axis, c->derived_axis);
}

/* the live update-flag array: 0 updated, 1 moved, 2 rotated (the same three arrays the original getter reads and clears) */
static unsigned int *flag_array (int which)
{
	return which == 0 ? updated_entities : (which == 1 ? moved_entities : rotated_entities);
}

static int popcount32 (unsigned int value)
{
	int n = 0;

	while (value)
	{
		n += (int) (value & 1u);
		value >>= 1;
	}
	return n;
}

static uint64_t flags_hash (const unsigned int *updated, const unsigned int *moved, const unsigned int *rotated)
{
	uint64_t hash = FNV_OFFSET;
	const unsigned int *arrays[COL_FLAG_ARRAYS];
	int a, w;

	arrays[0] = updated;
	arrays[1] = moved;
	arrays[2] = rotated;
	for (a = 0; a < COL_FLAG_ARRAYS; a++)
	{
		for (w = 0; w < COL_FLAG_WORDS; w++)
		{
			hash = fold64 (hash, (uint32_t) arrays[a][w]);
		}
	}
	return hash;
}

/* copy the three live arrays in full, before anything is observed */
static void flags_save (struct col_probe *p)
{
	int a;

	for (a = 0; a < COL_FLAG_ARRAYS; a++)
	{
		memcpy (p->flags_saved[a], flag_array (a), (size_t) COL_FLAG_WORDS * sizeof (unsigned int));
	}
	p->flags.before = flags_hash (p->flags_saved[0], p->flags_saved[1], p->flags_saved[2]);
}

/* what the ORIGINAL queries did to the live arrays, against the copy: words changed, bits cleared, bits added (the original only clears, so added is a finding if not 0) */
static void flags_measure (struct col_probe *p)
{
	int a, w;

	p->flags.after_queries = flags_hash (updated_entities, moved_entities, rotated_entities);
	for (a = 0; a < COL_FLAG_ARRAYS; a++)
	{
		const unsigned int *live = flag_array (a);

		for (w = 0; w < COL_FLAG_WORDS; w++)
		{
			unsigned int was = p->flags_saved[a][w], now = live[w];

			if (was != now)
			{
				p->flags.changed_words[a]++;
				p->flags.cleared_bits[a] += popcount32 (was & ~now);
				p->flags.added_bits += popcount32 (now & ~was);
			}
		}
	}
}

/* write the copy back and check it; idempotent, so every exit after the copy can call it */
static void flags_restore (struct col_probe *p)
{
	int a;

	p->flags.restored = 1;
	for (a = 0; a < COL_FLAG_ARRAYS; a++)
	{
		memcpy (flag_array (a), p->flags_saved[a], (size_t) COL_FLAG_WORDS * sizeof (unsigned int));
		if (memcmp (flag_array (a), p->flags_saved[a], (size_t) COL_FLAG_WORDS * sizeof (unsigned int)) != 0)
		{
			p->flags.restored = 0;
		}
	}
	p->flags.after_restore = flags_hash (updated_entities, moved_entities, rotated_entities);
}

/*
 * Runs the structured and random cases and writes the report. Returns 0, or -1 with `col-refused reason=...` in the report (a precondition
 * failed before anything was altered) or with nothing useful when the report does not fit.
 */
int eech_collision_probe_run (uint32_t seed, int entity_cases, int tree_cases, char *text, int capacity)
{
	struct col_probe *p = &probe_storage;
	struct out out;
	terrain_3d_point_data snapshot;
	uint64_t record_before, record_after, world_before, world_after;
	int rng_before, rng_after, restored, anchored, i, k, c;
	int fp_x87_masks = 0, fp_sse_masks = 0;

	text[0] = '\0';
	if (fegetround () != FE_TOWARDZERO)
	{
		refuse (text, capacity, "fpu_not_round_toward_zero");
		return -1;
	}
	if (!read_fp_masks (&fp_x87_masks, &fp_sse_masks))
	{
		refuse (text, capacity, "fp_masks_unreadable_on_this_architecture");
		return -1;
	}
	if (fp_x87_masks != 63)
	{
		refuse (text, capacity, "x87_exceptions_not_all_masked");
		return -1;
	}
	if (fp_sse_masks != 63)
	{
		refuse (text, capacity, "sse_exceptions_not_all_masked");
		return -1;
	}
	if (!terrain_sectors || terrain_3d_sector_x_max <= 0 || terrain_3d_sector_z_max <= 0)
	{
		refuse (text, capacity, "no_terrain_loaded");
		return -1;
	}
	if (float_bits (MIN_MAP_X) != float_bits (terrain_3d_min_map_x) || float_bits (MIN_MAP_Z) != float_bits (terrain_3d_min_map_z)
		|| float_bits (MAX_MAP_X) != float_bits (terrain_3d_max_map_x) || float_bits (MAX_MAP_Z) != float_bits (terrain_3d_max_map_z))
	{
		refuse (text, capacity, "world_map_is_not_the_terrain_extents");
		return -1;
	}
	if (!(MAX_MAP_X > 0.0f) || !(MAX_MAP_Z > 0.0f) || NUM_MAP_X_SECTORS < 1 || NUM_MAP_Z_SECTORS < 1 || SECTOR_SIDE_LENGTH < 1)
	{
		refuse (text, capacity, "empty_map");
		return -1;
	}
	if (!entity_sector_map)
	{
		refuse (text, capacity, "no_entity_sector_map");
		return -1;
	}
	if (draw_collision_boxes)
	{
		refuse (text, capacity, "draw_collision_boxes_is_set");
		return -1;
	}

	memset (p, 0, sizeof (*p));
	for (k = 0; k < G_COUNT; k++)
	{
		p->phys[k] = FNV_OFFSET;
		p->legacy[k] = FNV_OFFSET;
	}
	p->rng = seed ? seed : 1u;
	p->pool_rng = (seed ? seed : 1u) ^ 0x9e3779b9u;
	if (p->pool_rng == 0)
	{
		p->pool_rng = 1u;
	}
	p->record = shared_record ();
	make_sentinel (&p->sentinel);
	/* the update-flag arrays are copied in full before anything is observed (see the header); every exit below restores them */
	flags_save (p);

	/* read-only census and selection, before the shared record is touched */
	scan_entities (p);
	if (p->census.invalid_shape > 0)
	{
		flags_restore (p);
		refuse (text, capacity, "a_collision_candidate_has_no_valid_3d_shape_or_bounds");
		return -1;
	}
	scan_trees (p);

	memcpy (&snapshot, p->record, sizeof (snapshot));
	record_before = record_words_hash (&snapshot);
	p->record->last_elevation = bits_float (SENTINEL_BITS);
	p->record->terrain_type = (terrain_types) 1;
	anchored = (float_bits (get_3d_terrain_point_data_elevation (NULL)) == SENTINEL_BITS) && ((int) get_3d_terrain_point_data_type (NULL) == 1)
		&& (get_3d_terrain_point_data_normal (NULL) == &p->record->face_normal);
	memcpy (p->record, &snapshot, sizeof (snapshot));
	if (!anchored)
	{
		flags_restore (p);
		refuse (text, capacity, "shared_record_not_reachable_through_accessors");
		return -1;
	}

	out.text = text;
	out.capacity = capacity;
	out.used = 0;
	out.overflow = 0;
	p->out = &out;

	rng_before = get_random_number_seed ();
	world_before = world_fold ();

	/* structured entity cases, in class order */
	for (c = 0; c < CLS_COUNT; c++)
	{
		for (i = 0; i < p->sel_n[c]; i++)
		{
			entity_struct_target (p, &p->sel[c][i], i);
		}
	}
	entity_edge_cases (p);
	for (i = 0; i < entity_cases; i++)
	{
		entity_random_case (p, i);
	}

	/* structured tree cases: quad-plane trees, then geometry trees */
	for (c = 0; c < 2; c++)
	{
		for (i = 0; i < p->tsel_n[c]; i++)
		{
			tree_struct_target (p, &p->tsel[c][i], c, i);
		}
	}
	tree_edge_cases (p);
	for (i = 0; i < tree_cases; i++)
	{
		tree_random_case (p);
	}

	/* what the original queries did to the update flags is measured first, then the copy is written back; only then is the fingerprint taken again */
	flags_measure (p);
	flags_restore (p);

	/* put the terrain record back and check it */
	memcpy (p->record, &snapshot, sizeof (snapshot));
	restored = memcmp (p->record, &snapshot, sizeof (snapshot)) == 0;
	record_after = record_words_hash (p->record);
	rng_after = get_random_number_seed ();
	world_after = world_fold ();

	emit (&out, "col-probe seed=%u entity_random=%d tree_random=%d per_class=%d samples_hit=%d samples_miss=%d\n", (unsigned) seed, entity_cases, tree_cases, PER_CLASS, SAMPLES_HIT,
		SAMPLES_MISS);
	emit (&out, "col-map world_sectors_x=%d world_sectors_z=%d world_sector_side=%d terrain_sectors_x=%d terrain_sectors_z=%d terrain_sector_side=%d delta_time=%08x\n", NUM_MAP_X_SECTORS,
		NUM_MAP_Z_SECTORS, SECTOR_SIDE_LENGTH, terrain_3d_sector_x_max, terrain_3d_sector_z_max, (int) TERRAIN_3D_SECTOR_SIDE_LENGTH, float_bits ((float) get_delta_time ()));
	emit (&out, "col-guard record_restored=%d record_words_before=%016llx record_words_after=%016llx rng_unchanged=%d world_unchanged=%d world_before=%016llx world_after=%016llx\n",
		restored, (unsigned long long) record_before, (unsigned long long) record_after, rng_before == rng_after, world_before == world_after, (unsigned long long) world_before,
		(unsigned long long) world_after);
	emit (&out, "col-flags restored=%d words=%d before=%016llx after_queries=%016llx after_restore=%016llx updated_changed_words=%d updated_cleared_bits=%d moved_changed_words=%d "
		"moved_cleared_bits=%d rotated_changed_words=%d rotated_cleared_bits=%d added_bits=%d\n", p->flags.restored, COL_FLAG_WORDS, (unsigned long long) p->flags.before,
		(unsigned long long) p->flags.after_queries, (unsigned long long) p->flags.after_restore, p->flags.changed_words[0], p->flags.cleared_bits[0], p->flags.changed_words[1],
		p->flags.cleared_bits[1], p->flags.changed_words[2], p->flags.cleared_bits[2], p->flags.added_bits);
	emit (&out, "col-census-entity list_entities=%d mobile_candidates=%d mobile_moving=%d mobile_stationary=%d fixed_plain=%d cities=%d cities_with_buildings=%d buildings=%d "
		"listed_buildings=%d geometry_path=%d box_path=%d align_terrain=%d pool=%d\n", p->census.list_entities, p->census.mobile_candidates, p->census.mobile_moving,
		p->census.mobile_stationary, p->census.fixed_plain, p->census.cities, p->census.cities_with_buildings, p->census.buildings, p->census.listed_buildings,
		p->census.geometry_path, p->census.box_path, p->census.align_terrain, p->pool_n);
	emit (&out, "col-census-tree tree_data=%d trees_total=%ld sectors_with_trees=%d trees_type0=%d trees_type1=%d forest_geometry=%d pool=%d\n", p->census.tree_data,
		p->census.trees_total, p->census.sectors_with_trees, p->census.trees_type0, p->census.trees_type1, p->census.forest_geometry, p->tpool_n);
	emit (&out, "col-targets moving_geometry=%d moving_box=%d stationary_geometry=%d stationary_box=%d fixed_geometry=%d fixed_box=%d building_geometry=%d building_box=%d tree_quad=%d "
		"tree_geometry=%d\n", p->sel_n[CLS_MOVING_GEO], p->sel_n[CLS_MOVING_BOX], p->sel_n[CLS_STATIONARY_GEO], p->sel_n[CLS_STATIONARY_BOX], p->sel_n[CLS_FIXED_GEO],
		p->sel_n[CLS_FIXED_BOX], p->sel_n[CLS_BUILDING_GEO], p->sel_n[CLS_BUILDING_BOX], p->tsel_n[0], p->tsel_n[1]);

	emit (&out, "col-census-class moving_geometry=%d moving_box=%d stationary_geometry=%d stationary_box=%d fixed_geometry=%d fixed_box=%d building_geometry=%d building_box=%d "
		"tree_quad=%d tree_geometry=%d\n", p->class_seen[CLS_MOVING_GEO], p->class_seen[CLS_MOVING_BOX], p->class_seen[CLS_STATIONARY_GEO], p->class_seen[CLS_STATIONARY_BOX],
		p->class_seen[CLS_FIXED_GEO], p->class_seen[CLS_FIXED_BOX], p->class_seen[CLS_BUILDING_GEO], p->class_seen[CLS_BUILDING_BOX], p->tclass_seen[0], p->tclass_seen[1]);
	emit (&out, "col-fp arch=%s x87_masks=%d sse_masks=%d\n", COL_FP_ARCH_NAME, fp_x87_masks, fp_sse_masks);

	for (k = 0; k < EK_COUNT; k++)
	{
		const struct col_kinds *s = &p->ekinds[0][k];
		const struct col_kinds *r = &p->ekinds[1][k];

		if (k != EK_RANDOM)
		{
			emit (&out, "col-entity-kind group=struct kind=%s cases=%d hit=%d\n", entity_kind_name[k], s->cases, s->hit);
		}
		else
		{
			emit (&out, "col-entity-kind group=random kind=%s cases=%d hit=%d\n", entity_kind_name[k], r->cases, r->hit);
		}
	}
	emit (&out, "col-entity-exclusion through_cases=%d through_hit_target=%d through_hit_other=%d base_cases=%d src_removed=%d tgt_removed=%d both_removed=%d decoy_cases=%d "
		"decoy_unchanged=%d building_base=%d building_src_removed=%d building_city_src_removed=%d building_city_tgt_removed=%d chain_cases=%d chain_depth1=%d chain_depth2=%d "
		"chain_depth3=%d chain_stalled=%d chain_order_violations=%d\n", p->excl.through_cases, p->excl.through_hit_target, p->excl.through_hit_other, p->excl.base_cases,
		p->excl.src_removed, p->excl.tgt_removed, p->excl.both_removed, p->excl.decoy_cases, p->excl.decoy_unchanged, p->excl.building_base, p->excl.building_src_removed,
		p->excl.building_city_src_removed, p->excl.building_city_tgt_removed, p->excl.chain_cases, p->excl.chain_depth1, p->excl.chain_depth2, p->excl.chain_depth3,
		p->excl.chain_stalled, p->excl.chain_order_violations);
	emit_common (&out, "col-entity-common", "struct", &p->ecommon[0]);
	emit_common (&out, "col-entity-common", "random", &p->ecommon[1]);
	for (c = 0; c < 2; c++)
	{
		for (k = 0; k < NUM_ENTITY_TYPES; k++)
		{
			if (p->type_hits[c][k] > 0)
			{
				char name[96];

				sanitise (get_entity_type_name ((entity_types) k), name, sizeof (name));
				emit (&out, "col-entity-type group=%s type=%d name=%s hits=%d\n", c == 0 ? "struct" : "random", k, name, p->type_hits[c][k]);
			}
		}
	}

	for (k = 0; k < TK_COUNT; k++)
	{
		if (k == TK_RANDOM)
		{
			emit (&out, "col-tree-kind group=random kind=%s cases=%d hit=%d\n", tree_kind_name[k], p->tkinds[1][k].cases, p->tkinds[1][k].hit);
		}
		else
		{
			emit (&out, "col-tree-kind group=struct kind=%s cases=%d hit=%d\n", tree_kind_name[k], p->tkinds[0][k].cases, p->tkinds[0][k].hit);
		}
	}
	for (c = 0; c < 2; c++)
	{
		for (i = 0; i < 2; i++)
		{
			for (k = 0; k < 2; k++)
			{
				emit (&out, "col-tree-flags class=%s ignore_edge=%d skip_top=%d cases=%d hit=%d\n", c == 0 ? "quad" : "geometry", i, k, p->tflags[c][i][k].cases,
					p->tflags[c][i][k].hit);
			}
		}
	}
	emit_common (&out, "col-tree-common", "struct", &p->tcommon[0]);
	emit_common (&out, "col-tree-common", "random", &p->tcommon[1]);

	emit (&out, "col-physical entity_struct=%016llx entity_random=%016llx tree_struct=%016llx tree_random=%016llx\n", (unsigned long long) p->phys[G_ENTITY_STRUCT],
		(unsigned long long) p->phys[G_ENTITY_RANDOM], (unsigned long long) p->phys[G_TREE_STRUCT], (unsigned long long) p->phys[G_TREE_RANDOM]);
	emit (&out, "col-legacy entity_struct=%016llx entity_random=%016llx tree_struct=%016llx tree_random=%016llx\n", (unsigned long long) p->legacy[G_ENTITY_STRUCT],
		(unsigned long long) p->legacy[G_ENTITY_RANDOM], (unsigned long long) p->legacy[G_TREE_STRUCT], (unsigned long long) p->legacy[G_TREE_RANDOM]);

	if (out.overflow)
	{
		refuse (text, capacity, "report_does_not_fit");
		return -1;
	}
	return 0;
}

#endif
