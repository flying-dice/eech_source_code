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
} host_terrain_hint;

typedef struct host_terrain_answer
{
	float elevation;      /* the value the query returns */
	float last_elevation; /* the value it stores in the record: differs from `elevation` only in the Cuba flaw area */
	float normal[3];
	int32_t type;
	int32_t has_hint;     /* 0 when the query cleared the record's hint (no face found, or the Cuba flaw area) */
	host_terrain_hint hint;
	int32_t fast_attempted, fast_hit, found, clamped, cuba, nan_position; /* what the query did, for the conformance record */
} host_terrain_answer;

/* The buffers belong to the caller and must outlive the map. `grid` may be NULL (no fast grid). Returns NULL and a message on a malformed input. */
host_terrain *host_terrain_open (const uint8_t *sector, size_t sector_length, const uint8_t *geometry, size_t geometry_length, const uint8_t *grid, size_t grid_length,
	int32_t map_number, char *error, size_t error_size);
void host_terrain_close (host_terrain *map);
void host_terrain_get_facts (const host_terrain *map, host_terrain_facts *out);

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

/* the first `emit` samples (at most 256) are written to `samples` */
int host_terrain_conformance_sweep (const host_terrain *map, uint32_t seed, int count, int emit, host_terrain_sweep *out, host_terrain_sweep_sample *samples,
	char *error, size_t error_size);

#endif
