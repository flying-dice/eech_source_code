/*
 * The reference host's terrain kernel (EEDCS-47): see terrain_ref.c. Plain C values only; the Rust side (src/terrain.rs) mirrors these
 * layouts. Nothing here exposes a pointer into the map: the find-fast record the original solver keeps between queries crosses as a
 * host_terrain_hint value, which only the host that issued it can interpret.
 */
#ifndef EECH_TERRAIN_REF_H
#define EECH_TERRAIN_REF_H

#include <stddef.h>
#include <stdint.h>

typedef struct host_terrain host_terrain;

/* the facts the campaign reads once about the map (terrdata.c globals, terrsel.c grid header); fast_* are 0 when there is no fast grid */
typedef struct host_terrain_facts
{
	int32_t sector_x_max, sector_z_max, map_number;
	uint32_t version;
	float min_x, min_z, max_x, max_z, min_height, max_height, scaled_height_difference;
	int32_t fast_width, fast_height;
	float fast_x_grid, fast_z_grid;
} host_terrain_facts;

/* what a query leaves in the legacy record, by value: the sector, the face (its index in the sector), the offset of the face's point references,
   the triangle in it, the surface type and the stored normal. A query given no hint behaves as a query on a freshly cleared record. */
typedef struct host_terrain_hint
{
	int32_t x_sector, z_sector;
	uint32_t face_index, reference_offset;
	int32_t triangle_offset, type;
	float normal[3];
	/* binds the hint to the map instance that issued it and to every field above: a corrupted, edited or foreign hint is refused (an integrity check against accident and staleness, not a secret) */
	uint64_t tag;
} host_terrain_hint;

typedef struct host_terrain_answer
{
	float elevation;      /* the value the query returns */
	float last_elevation; /* the value it stores in the record: differs from `elevation` only in the Cuba flaw area */
	float normal[3];
	int32_t type;
	int32_t has_hint;     /* 0 when the query cleared the record's hint (no face found, or the Cuba flaw area) */
	int32_t stored;       /* 1 when the record holds a face after the query (the legacy "terrain_face != NULL": it stays so after a later no-face or Cuba answer) */
	host_terrain_hint hint;
	int32_t fast_attempted, fast_hit, found, clamped, cuba, nan_position; /* what the query did, for the conformance record */
} host_terrain_answer;

/* The buffers belong to the caller and must outlive the map. `grid` may be NULL: the original generates the fast grid when terrain.sim is missing, and so does the host
   (in memory, from the point query, then the coast pass and the sea normalisation). The generation reads the map's coastal rivers (route/rivers.nde and rivers.wp, or
   rivdata.nde and rivdata.wp): pass both river buffers, or NULL for both when the map has none; they are read only while generating and may be released after the call.
   Returns NULL and a message on a malformed input. */
host_terrain *host_terrain_open (const uint8_t *sector, size_t sector_length, const uint8_t *geometry, size_t geometry_length, const uint8_t *grid, size_t grid_length,
	const uint8_t *river_nodes, size_t river_node_length, const uint8_t *river_paths, size_t river_path_length,
	int32_t map_number, uint64_t map_key, char *error, size_t error_size);
void host_terrain_close (host_terrain *map);
void host_terrain_get_facts (const host_terrain *map, host_terrain_facts *out);

/* DIAGNOSTIC. The fast grid as terrain.sim holds it (i32 width, i32 height, f32 x size, f32 z size, then the f32 posts, little-endian), for comparing a grid the
   host generated with the original's file. Size is 0 when the map has no grid; the copy fails when `capacity` is smaller than the size. */
size_t host_terrain_grid_size (const host_terrain *map);
int host_terrain_copy_grid (const host_terrain *map, uint8_t *out, size_t capacity, char *error, size_t error_size);

/* 0 on success; otherwise a message (a malformed hint, a position that is not finite for the fast grid, no fast grid). The hint and loader checks are meant to keep every read inside the map; that is by inspection, not yet tested or reviewed. */
int host_terrain_query (const host_terrain *map, float x, float z, const host_terrain_hint *hint, host_terrain_answer *out, char *error, size_t error_size);
int host_terrain_fast_elevation (const host_terrain *map, float x, float z, float *out, char *error, size_t error_size);

/*
 * CONFORMANCE ONLY. The deterministic sweep of eech-engine-sys/csrc/eech_terrain_fixture.c (the probe the original solver was characterised
 * with), driven against this kernel instead of the DLL, with the same point generator, the same hints carried between queries and the same
 * hashes. Its float arithmetic runs under round-toward-zero as the engine's does. It starts with a cleared shared record; the original probe
 * started with whatever the campaign had left in its shared record, which can differ only in the first iteration's discarded primer.
 */
typedef struct host_terrain_sweep_sample
{
	float x, z, result, last, nx, ny, nz;
	int32_t type;
	float fast; /* the fast grid's answer at the point (0 when the map has no grid) */
} host_terrain_sweep_sample;

typedef struct host_terrain_sweep
{
	uint64_t sample_hash, primed_hash;
	uint64_t repeat, private_primed, shared_primed, private_primed_fast, shared_primed_fast, elevation_macro, elevation_and_normal, face_normal, result_ne_last;
	uint64_t snapped, primer_skipped;
	uint64_t fast_hash, fast_count, fast_available; /* the fast elevation at the same points: hash over (x, z, answer) bits, how many, and whether the map has a grid */
} host_terrain_sweep;

/* `count` is 0..200000; `emit` is capped at 256 and at count + 6; `samples` must hold the capped number of entries and only those are written */
int host_terrain_conformance_sweep (const host_terrain *map, uint32_t seed, int count, int emit, host_terrain_sweep *out, host_terrain_sweep_sample *samples,
	char *error, size_t error_size);

/*
 * GROUND CONTACT (EEDCS-48, the host side of D3 group G2; the campaign DLL no longer computes these: mb_hostterrain.c asks the host for all three).
 *
 * The two physical expressions the campaign's weapon flight asks about the ground, kept expression for expression from the original (en_world.h point_above_ground and
 * point_below_ground, en_cllsn.c get_ground_impact_point), over this map's own terrain: the sector maximum is read from the sectors the host already holds, and every
 * elevation is the point query above, with the find-fast record carried through the whole sequence exactly as the original's one shared record is. Nothing here exposes an
 * index, a pointer or a layout: the record a sequence leaves crosses as the host_terrain_answer a point query already returns (values, `stored`, and the opaque hint).
 *
 * The CAMPAIGN keeps the terrain-class vocabulary (the impact returns the surface TYPE the last query left in the record, not a class) and its geography: the world-map
 * extents the below-ground test compares against are supplied by the caller (they are campaign data, and are not assumed equal to the terrain's), and so is the aliasing of
 * the impact output with the below endpoint (the caller composes it from the corrected endpoints and the impact point).
 *
 * Floating point: each public ground entry saves the caller's COMPLETE state (x87 environment and MXCSR) before its first statement and restores it on every return path, refusals
 * included; the kernels' own round-toward-zero arithmetic is unchanged (terrain_ref.c, struct host_fp_state).
 *
 * Unsupported, refused with a message and nothing computed: a coordinate that is not finite; a position off the terrain's own extents for any read of the sector maximum
 * (the original stops the engine in debug_fatal there); a hint the map did not issue (refused when it is first USED, so an answer that never reads the terrain never
 * looks at it).
 */
typedef struct host_ground_world
{
	float min_x, min_z, max_x, max_z; /* the campaign's world-map area (en_world.h point_inside_map_area): x and z only */
} host_ground_world;

typedef struct host_ground_below_answer
{
	int32_t below; /* what point_below_ground answers */
	/* 1 when the answer read the terrain (the sector maximum did not decide it, and the point is inside the world area). 0: no terrain read happened, the caller's record
	   is to be left exactly as it is, and `record` is zeroed. When 1, `record` is the record the last read left (the complete value state and the optional hint). */
	int32_t queried;
	host_terrain_answer record;
} host_ground_below_answer;

typedef struct host_ground_above_answer
{
	int32_t above; /* what point_above_ground answers */
	/* as host_ground_below_answer: 1 when the answer read the terrain (the sector maximum decided it otherwise: no read, the record untouched, `record` zeroed) */
	int32_t queried;
	host_terrain_answer record;
} host_ground_above_answer;

typedef struct host_ground_impact_answer
{
	float above[3];  /* the above endpoint after the original's correction (unchanged when it already was above the ground) */
	float below[3];  /* the below endpoint after the original's correction */
	float impact[3]; /* the impact point: x and z of the final below endpoint of the bisection, y the terrain elevation there */
	int32_t type;    /* the EECH terrain_types id the final query left in the record: the campaign maps it to a class */
	int32_t own_assert_failures; /* how many of the original function's ASSERT conditions were false (they are inert in the characterised build: counted, never fatal) */
	host_terrain_answer record;  /* the record the sequence left (there is always at least one read) */
} host_ground_impact_answer;

/* point_above_ground: the sector maximum first and, when it does not decide, the elevation. The world area is not consulted by the predicate itself but is validated like the other operations' */
int host_terrain_ground_above (const host_terrain *map, const host_ground_world *world, float x, float y, float z, const host_terrain_hint *hint,
	host_ground_above_answer *out, char *error, size_t error_size);
int host_terrain_ground_below (const host_terrain *map, const host_ground_world *world, float x, float y, float z, const host_terrain_hint *hint,
	host_ground_below_answer *out, char *error, size_t error_size);
int host_terrain_ground_impact (const host_terrain *map, const host_ground_world *world, const float above[3], const float below[3], const host_terrain_hint *hint,
	host_ground_impact_answer *out, char *error, size_t error_size);

/*
 * CONFORMANCE ONLY. The deterministic ground probe of eech-engine-sys/csrc/eech_ground_fixture.c (the one the ORIGINAL queries were characterised with, c119a7370), driven against
 * these two operations instead of the DLL: the same private generator, the same cases in the same draw order, the same cold and warm calls, the same physical fold over the same
 * words, the same counters and the same samples. Its float arithmetic runs under round-toward-zero as the engine's does. The campaign's data it needs are given, not invented:
 * the world area is the map's own extents (the fixture refuses a map whose world and terrain extents differ, and the three retail roots it ran on are equal), and the
 * terrain-type -> class table (the campaign's vocabulary, aphavoc/source/misc/terr_db.c) is the caller's. The "record touched" facts of the fixture are `queried` here.
 */
#define HOST_GROUND_POINT_SAMPLES 12  /* 2 per point tag, 6 tags */
#define HOST_GROUND_IMPACT_SAMPLES 32 /* 4 per impact type, 8 types */

typedef struct host_ground_point_sample
{
	int32_t n, tag;
	uint32_t x, y, z; /* bit patterns */
	int32_t inside, above, above_queried, below, below_queried; /* above is -1 when it was not asked (outside the world area) */
	uint32_t above_elevation, below_elevation;                   /* last_elevation of the record the call left; the fixture's sentinel word 0x5a5a5a5a when it was not touched, 0 for an unasked above */
	int32_t above_type, below_type;
} host_ground_point_sample;

typedef struct host_ground_impact_sample
{
	int32_t n, type;
	uint32_t a0[3], b0[3];
	int32_t class_alias;
	uint32_t pa[3], qa[3];
	int32_t class_separate;
	uint32_t pb[3], qb[3], rb[3];
	int32_t alias_equal, warm_same, queried;
	uint32_t elevation;
	int32_t type_word;
} host_ground_impact_sample;

typedef struct host_ground_sweep
{
	uint64_t physical_combined, physical_above, physical_below, physical_impact;
	int32_t points, in_map, off_map, input_mutated;
	int32_t above_cases, above_true, above_queried, above_true_unqueried, above_warm_same;
	int32_t below_cases, below_true, below_queried, below_true_unqueried, below_warm_same, below_off_map_true, below_off_map_queried;
	int32_t in_map_both, complement, not_complement, both_true, both_false;
	int32_t impacts, a_corrected, b_corrected, xz_changed, alias_equal, alias_unequal, warm_same, warm_different, impact_queried, impact_unqueried, impact_equals_corrected_below;
	int32_t class_count[4], class_other;
	int32_t by_type[8];
	int32_t original_below_endpoint_still_above, extra_above_endpoint_not_above, own_assert_failures;
	int32_t point_sample_count, impact_sample_count;
	host_ground_point_sample point_samples[HOST_GROUND_POINT_SAMPLES];
	host_ground_impact_sample impact_samples[HOST_GROUND_IMPACT_SAMPLES];
} host_ground_sweep;

/* `points` and `impacts` are 0..100000 and not both 0; `class_of_type` has TERRAIN_TYPE_LAST (62) entries, each a terrain class id 0..3 */
int host_terrain_ground_conformance (const host_terrain *map, uint32_t seed, int points, int impacts, const int32_t *class_of_type, size_t class_count, host_ground_sweep *out,
	char *error, size_t error_size);

#endif
