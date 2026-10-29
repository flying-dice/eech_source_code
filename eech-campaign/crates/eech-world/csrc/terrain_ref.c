/*
 * The reference host's terrain kernel (EEDCS-47, D3).
 *
 * The host owns and loads the physical terrain independently of the campaign DLL, and answers the campaign's point-surface and
 * fast-elevation questions from it. This file is EECH's own terrain solver kept expression for expression, as the other host
 * kernels in this directory are, and nothing else:
 *
 *   - the triangle tests and the query body are copied verbatim from modules/3d/terrain/terrelev.c at integration
 *     6bbaa8197a4843181d03cd355d562af998c75321 (file sha256 ef897d423c2d1a30bfec9cc9205c492a759020b5e0d85ee1d2a68d3df7311c9e), the `point_inside_terrain_*` functions from lines 145-1196 and
 *     `get_3d_terrain_point_data` from lines 1208-2005, with the public names prefixed `host_` so that this archive cannot interpose
 *     on the engine DLL's symbols;
 *   - the build-time patches that the characterised engine applies on top of that source (eech-engine-sys/build/patches.rs N1, a NaN
 *     position is moved to the map corner; N3, a degenerate face takes the up vector; X1, float-to-int is a C truncation) are
 *     applied here too, so this is the characterised post-patch behaviour and not the historical text;
 *   - the sector data are the sectors `load_3d_terrain` (terrdata.c) reads from default.sec and terrain.ffp, but taken from memory
 *     buffers the caller owns and checked against their length first, instead of trusted pointers into a memory map;
 *   - the fast grid is `get_3d_terrain_fast_elevation` (terrsel.c) over the terrain.sim file the original reads.
 *
 * What differs from the original, and why: the legacy find-fast record (pointers into the sector memory) is carried between calls
 * as a plain value, the host_terrain_hint, which the query turns back into the record the original solver expects and out of the
 * record it leaves; the campaign never sees the layout. The solver's ASSERT and debug_log are not compiled (a map the loader accepts
 * cannot trip the ones that matter), and the original's off-map ASSERT, whose effect is not characterised, is reported as the clamp
 * it falls through to. Not thread safe: a query copies the map's facts into file statics, as the original's solver reads them.
 *
 * Rounding: the original sets round-toward-zero on every query (get_terrain_3d_sector) and the engine runs under it. A query here
 * does the same and restores the caller's floating-point environment on return.
 */

#include <fenv.h>
#include <math.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "eech_arith.h"

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif

#define ASSERT(E) ((void) 0)
#define debug_log(...) ((void) 0)
/* unreachable for a map the loader accepted and a hint the query validated: every polygon type a face can reach is a fan or a strip */
#define debug_fatal(...) abort ()
#define normalise_3d_vector normalise_any_3d_vector
#define X_AND_Z_ARE_FLOATS 0

#define TERRAIN_3D_SECTOR_SIDE_LENGTH 2048
#define TERRAIN_3D_SECTOR_SIDE_LENGTH_ADJUSTMENT 2048
#define TERRAIN_3D_XZ_SCALE ( ( ( float ) ( TERRAIN_3D_SECTOR_SIDE_LENGTH + TERRAIN_3D_SECTOR_SIDE_LENGTH_ADJUSTMENT ) ) / 65536.0f )
#define TERRAIN_3D_Y_SCALE ( 1.0f / 32767.0f )

/* terrtype.h: the terrain type ids are the campaign's vocabulary and are kept as they are */
enum host_terrain_types
{
TERRAIN_TYPE_INVALID,
TERRAIN_TYPE_RIVER,
TERRAIN_TYPE_RESERVOIR,
TERRAIN_TYPE_TRENCH,
TERRAIN_TYPE_RIVER_BANK,
TERRAIN_TYPE_TRENCH_SIDE_X,
TERRAIN_TYPE_TRENCH_SIDE_Z,
TERRAIN_TYPE_SEA,
TERRAIN_TYPE_BEACH,
TERRAIN_TYPE_LAND,
TERRAIN_TYPE_FIELD1,
TERRAIN_TYPE_FIELD2,
TERRAIN_TYPE_FIELD3,
TERRAIN_TYPE_FIELD4,
TERRAIN_TYPE_FIELD5,
TERRAIN_TYPE_FIELD6,
TERRAIN_TYPE_FIELD7,
TERRAIN_TYPE_FIELD8,
TERRAIN_TYPE_FIELD9,
TERRAIN_TYPE_FIELD10,
TERRAIN_TYPE_FIELD11,
TERRAIN_TYPE_ALTERED_LAND1,
TERRAIN_TYPE_ALTERED_LAND2,
TERRAIN_TYPE_ALTERED_LAND3,
TERRAIN_TYPE_FOREST_FLOOR,
TERRAIN_TYPE_ROAD_BANK,
TERRAIN_TYPE_RAIL_BANK,
TERRAIN_TYPE_HEDGE_SIDE_X,
TERRAIN_TYPE_HEDGE_SIDE_Z,
TERRAIN_TYPE_WALL_SIDE_X,
TERRAIN_TYPE_WALL_SIDE_Z,
TERRAIN_TYPE_FOREST_SIDE_BOTTOM_X,
TERRAIN_TYPE_FOREST_SIDE_BOTTOM_Z,
TERRAIN_TYPE_FOREST_SIDE_MID_X,
TERRAIN_TYPE_FOREST_SIDE_MID_Z,
TERRAIN_TYPE_FOREST_SIDE_TOP_X,
TERRAIN_TYPE_FOREST_SIDE_TOP_Z,
TERRAIN_TYPE_BUILT_UP_AREA1,
TERRAIN_TYPE_BUILT_UP_AREA2,
TERRAIN_TYPE_BUILT_UP_AREA3,
TERRAIN_TYPE_BUILT_UP_AREA4,
TERRAIN_TYPE_ROAD,
TERRAIN_TYPE_TRACK,
TERRAIN_TYPE_RAIL,
TERRAIN_TYPE_WALL_TOP,
TERRAIN_TYPE_HEDGE_TOP,
TERRAIN_TYPE_FOREST_TOP,
TEMP_TERRAIN_TYPE_FRINGE,
TEMP_TERRAIN_TYPE_POWER_LINE,
TEMP_TERRAIN_TYPE_TELEPHONE_LINE,
TEMP_TERRAIN_TYPE_SEA_BANK,
TEMP_TERRAIN_TYPE_COASTAL_RIVER,
TEMP_TERRAIN_TYPE_BEACH_MID,
TEMP_TERRAIN_TYPE_STEPPED_RIVER_BANK3,
TEMP_TERRAIN_TYPE_STEPPED_RIVER_BANK5,
TEMP_TERRAIN_TYPE_STEPPED_RIVER_BANK7,
TEMP_TERRAIN_TYPE_LAKE,
TEMP_TERRAIN_TYPE_FOREST_SIDE_X,
TEMP_TERRAIN_TYPE_FOREST_SIDE_Z,
TEMP_TERRAIN_TYPE_OFFROAD,
TEMP_TERRAIN_TYPE_POLITICAL_BOUNDARY,
TEMP_TERRAIN_TYPE_BUILT_UP_AREA_BANK,
TERRAIN_TYPE_LAST,
};
typedef int terrain_types;

enum { POLYGON_TYPE_FAN, POLYGON_TYPE_STRIP, POLYGON_TYPE_UNKNOWN };

/* terrtype.c, initialise_3d_terrain_rendering_routines: the polygon type of each terrain type (every assignment, in both rendering modes, agrees; the temporary generation types are explicitly unknown, so a face of one is never found; INVALID is never assigned and is 0, a fan) */
static struct { int polygon_type; } terrain_type_information[TERRAIN_TYPE_LAST] = {
	[TEMP_TERRAIN_TYPE_FRINGE] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_POWER_LINE] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_TELEPHONE_LINE] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_SEA_BANK] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_COASTAL_RIVER] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_BEACH_MID] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_STEPPED_RIVER_BANK3] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_STEPPED_RIVER_BANK5] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_STEPPED_RIVER_BANK7] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_LAKE] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_FOREST_SIDE_X] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_FOREST_SIDE_Z] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_OFFROAD] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_POLITICAL_BOUNDARY] = POLYGON_TYPE_UNKNOWN,
	[TEMP_TERRAIN_TYPE_BUILT_UP_AREA_BANK] = POLYGON_TYPE_UNKNOWN,
	[TERRAIN_TYPE_ALTERED_LAND1] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_ALTERED_LAND2] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_ALTERED_LAND3] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_BEACH] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_BUILT_UP_AREA1] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_BUILT_UP_AREA2] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_BUILT_UP_AREA3] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_BUILT_UP_AREA4] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD1] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD10] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD11] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD2] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD3] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD4] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD5] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD6] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD7] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD8] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FIELD9] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FOREST_FLOOR] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_FOREST_SIDE_BOTTOM_X] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_FOREST_SIDE_BOTTOM_Z] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_FOREST_SIDE_MID_X] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_FOREST_SIDE_MID_Z] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_FOREST_SIDE_TOP_X] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_FOREST_SIDE_TOP_Z] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_FOREST_TOP] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_HEDGE_SIDE_X] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_HEDGE_SIDE_Z] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_HEDGE_TOP] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_LAND] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_RAIL] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_RAIL_BANK] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_RESERVOIR] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_RIVER] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_RIVER_BANK] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_ROAD] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_ROAD_BANK] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_SEA] = POLYGON_TYPE_FAN,
	[TERRAIN_TYPE_TRACK] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_TRENCH] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_TRENCH_SIDE_X] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_TRENCH_SIDE_Z] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_WALL_SIDE_X] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_WALL_SIDE_Z] = POLYGON_TYPE_STRIP,
	[TERRAIN_TYPE_WALL_TOP] = POLYGON_TYPE_STRIP,
};

/* terrdata.h: the sector geometry, as it lies in terrain.ffp */
struct TERRAIN_3D_POINT_XZ
{
	short int
		x,
		z;
};

typedef struct TERRAIN_3D_POINT_XZ terrain_3d_point_xz;

#pragma pack(push, 1)

struct TERRAIN_3D_POINT_Y
{
	union
	{
		struct
		{
			unsigned short int
				y:15,
				normal_change:1;
		};

		unsigned short int
			whole_value;

		short int
			whole_signed_value;
	};
};

typedef struct TERRAIN_3D_POINT_Y terrain_3d_point_y;

#pragma pack(pop)

struct TERRAIN_3D_POINT_WORD_REFERENCE
{
	unsigned short int
		point;
};

typedef struct TERRAIN_3D_POINT_WORD_REFERENCE terrain_3d_point_word_reference;

struct TERRAIN_3D_POINT_BYTE_REFERENCE
{
	unsigned char
		point;
};

typedef struct TERRAIN_3D_POINT_BYTE_REFERENCE terrain_3d_point_byte_reference;

struct TERRAIN_3D_FACE
{
	unsigned char
		number_of_points:7,
		surface_change:1;
};

typedef struct TERRAIN_3D_FACE terrain_3d_face;

struct TERRAIN_3D_SURFACE
{
	unsigned char
		surface_id;
};

typedef struct TERRAIN_3D_SURFACE terrain_3d_surface;

/* the sector as the solver reads it (the rendering, colour, normal-index and object members are not the solver's); the last three are the host's own */
struct TERRAIN_3D_SECTOR
{
	unsigned short int
		number_of_points,
		number_of_polygons,
		radius;

	short int
		minimum_height,
		maximum_height,
		average_height;

	struct TERRAIN_3D_POINT_XZ
		*points_xz;

	struct TERRAIN_3D_POINT_Y
		*points_y;

	struct TERRAIN_3D_FACE
		*polygons;

	union
	{
		struct TERRAIN_3D_POINT_WORD_REFERENCE
			*point_word_references;

		struct TERRAIN_3D_POINT_BYTE_REFERENCE
			*point_byte_references;
	};

	struct TERRAIN_3D_SURFACE
		*surface_changes;

	uint32_t
		*face_reference_offset;

	unsigned char
		*face_type;

	/* the one allocation that holds this sector's copy of the geometry above (and the two host arrays), 4-aligned */
	void
		*block;

	uint32_t
		total_references;
};

typedef struct TERRAIN_3D_SECTOR terrain_3d_sector;

/* terrdata.h: the find-fast record the solver reads and leaves */
struct TERRAIN_3D_POINT_DATA
{
	int
		x_sector,
		z_sector;

	terrain_3d_sector
		*terrain_sector;

	float
		last_elevation;

	terrain_3d_face
		*terrain_face;

	union
	{
		terrain_3d_point_word_reference
			*terrain_elevation_point_word_references;

		terrain_3d_point_byte_reference
			*terrain_elevation_point_byte_references;
	};

	int
		terrain_triangle_offset;

	terrain_types
		terrain_type;

	vec3d
		face_normal;
};

typedef struct TERRAIN_3D_POINT_DATA terrain_3d_point_data;

/* the solver's globals (terrdata.c), for the map being queried; set by host_terrain_enter */
static float
	terrain_3d_min_map_x,
	terrain_3d_min_map_z,
	terrain_3d_max_map_x,
	terrain_3d_max_map_z,
	terrain_3d_map_minimum_height,
	terrain_3d_map_scaled_height_difference;

static terrain_3d_sector
	**terrain_sectors;

static struct { int map_number; } current_terrain_version;

/* what the query noted about itself (read by the wrapper) */
static int
	host_note_clamped,
	host_note_fast_attempted,
	host_note_fast_hit,
	host_note_found,
	host_note_cuba,
	host_note_nan;

/* terrelev.c: the shared record of a query without a record of its own (the wrapper always supplies one) and the reference pointers the triangle tests read */
static terrain_3d_point_data
	system_point_data;

static terrain_3d_point_word_reference
	*terrain_elevation_point_word_references;

static terrain_3d_point_byte_reference
	*terrain_elevation_point_byte_references;

/* terrdata.c get_terrain_3d_sector, with X1 (float to int is a C truncation) and set_fpu_rounding_mode_zero (a round-toward-zero environment) */
static void get_terrain_3d_sector ( float x, float z, int *sectorx, int *sectorz )
{
	fesetround ( FE_TOWARDZERO );

	*sectorx = ( int ) ( x / TERRAIN_3D_SECTOR_SIDE_LENGTH );

	*sectorz = ( int ) ( z / TERRAIN_3D_SECTOR_SIDE_LENGTH );
}

/* ---- terrelev.c, verbatim: the triangle tests (host_ prefix on the names only) ---- */

static int host_point_inside_terrain_word_fan ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int number_of_points, int *triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	int
		internal_triangle_offset;

	terrain_3d_point_word_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_word_references;

	ASSERT ( number_of_points > 2 );

	point = points + point_ref->point;

	x0 = point->x;
	z0 = point->z;

	point_ref++;

	point = points + point_ref->point;

	x2 = point->x;
	z2 = point->z;

	point_ref++;

	internal_triangle_offset = 0;

	number_of_points -= 2;

	while ( number_of_points-- )
	{

		x1 = x2;
		z1 = z2;

		point = points + point_ref->point;

		x2 = point->x;
		z2 = point->z;

		point_ref++;

		//
		// Check against the first side
		//

		dx1 = x0 - x1;
		dz1 = z0 - z1;
		dx2 = x1 - x_offset;
		dz2 = z1 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the third side
			//

			dx1 = x2 - x0;
			dz1 = z2 - z0;
			dx2 = x0 - x_offset;
			dz2 = z0 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				//
				// Check against the second side
				//

				dx1 = x1 - x2;
				dz1 = z1 - z2;
				dx2 = x2 - x_offset;
				dz2 = z2 - z_offset;

				cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

				if ( cyclic >= 0.0 )
				{

					*triangle_offset = internal_triangle_offset;

					return ( TRUE );
				}
			}
		}

		internal_triangle_offset++;
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int host_point_inside_terrain_byte_fan ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int number_of_points, int *triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	int
		internal_triangle_offset;

	terrain_3d_point_byte_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_byte_references;

	ASSERT ( number_of_points > 2 );

	point = points + point_ref->point;

	x0 = point->x;
	z0 = point->z;

	point_ref++;

	point = points + point_ref->point;

	x2 = point->x;
	z2 = point->z;

	point_ref++;

	internal_triangle_offset = 0;

	number_of_points -= 2;

	while ( number_of_points-- )
	{

		x1 = x2;
		z1 = z2;

		point = points + point_ref->point;

		x2 = point->x;
		z2 = point->z;

		point_ref++;

		//
		// Check against the first side
		//

		dx1 = x0 - x1;
		dz1 = z0 - z1;
		dx2 = x1 - x_offset;
		dz2 = z1 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the third side
			//

			dx1 = x2 - x0;
			dz1 = z2 - z0;
			dx2 = x0 - x_offset;
			dz2 = z0 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				//
				// Check against the second side
				//

				dx1 = x1 - x2;
				dz1 = z1 - z2;
				dx2 = x2 - x_offset;
				dz2 = z2 - z_offset;

				cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

				if ( cyclic >= 0.0 )
				{

					*triangle_offset = internal_triangle_offset;

					return ( TRUE );
				}
			}
		}

		internal_triangle_offset++;
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int host_point_inside_terrain_word_strip ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int number_of_points, int *triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	int
		even_triangle,
		internal_triangle_offset;

	terrain_3d_point_word_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_word_references;

	ASSERT ( number_of_points > 2 );

	point = points + point_ref[0].point;

	x0 = point->x;
	z0 = point->z;

	point = points + point_ref[1].point;

	x1 = point->x;
	z1 = point->z;

	point = points + point_ref[2].point;

	x2 = point->x;
	z2 = point->z;

	point_ref++;

	even_triangle = FALSE;

	internal_triangle_offset = 0;

	number_of_points -= 2;

	while ( number_of_points-- )
	{

		//
		// Check against the first side
		//

		dx1 = x0 - x1;
		dz1 = z0 - z1;
		dx2 = x1 - x_offset;
		dz2 = z1 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the third side
			//

			dx1 = x2 - x0;
			dz1 = z2 - z0;
			dx2 = x0 - x_offset;
			dz2 = z0 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				//
				// Check against the second side
				//

				dx1 = x1 - x2;
				dz1 = z1 - z2;
				dx2 = x2 - x_offset;
				dz2 = z2 - z_offset;

				cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

				if ( cyclic >= 0.0 )
				{

					*triangle_offset = internal_triangle_offset;

					return ( TRUE );
				}
			}
		}

		if ( number_of_points )
		{

			if ( even_triangle )
			{

				point = points + point_ref[0].point;

				x0 = point->x;
				z0 = point->z;

				point = points + point_ref[1].point;

				x1 = point->x;
				z1 = point->z;

				point = points + point_ref[2].point;

				x2 = point->x;
				z2 = point->z;

				even_triangle = FALSE;
			}
			else
			{

				point = points + point_ref[0].point;

				x0 = point->x;
				z0 = point->z;

				point = points + point_ref[2].point;

				x1 = point->x;
				z1 = point->z;

				point = points + point_ref[1].point;

				x2 = point->x;
				z2 = point->z;

				even_triangle = TRUE;
			}
		}

		point_ref++;

		internal_triangle_offset++;
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static int host_point_inside_terrain_byte_strip ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int number_of_points, int *triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	int
		even_triangle,
		internal_triangle_offset;

	terrain_3d_point_byte_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_byte_references;

	ASSERT ( number_of_points > 2 );

	point = points + point_ref[0].point;

	x0 = point->x;
	z0 = point->z;

	point = points + point_ref[1].point;

	x1 = point->x;
	z1 = point->z;

	point = points + point_ref[2].point;

	x2 = point->x;
	z2 = point->z;

	point_ref++;

	even_triangle = FALSE;

	internal_triangle_offset = 0;

	number_of_points -= 2;

	while ( number_of_points-- )
	{

		//
		// Check against the first side
		//

		dx1 = x0 - x1;
		dz1 = z0 - z1;
		dx2 = x1 - x_offset;
		dz2 = z1 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the third side
			//

			dx1 = x2 - x0;
			dz1 = z2 - z0;
			dx2 = x0 - x_offset;
			dz2 = z0 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				//
				// Check against the second side
				//

				dx1 = x1 - x2;
				dz1 = z1 - z2;
				dx2 = x2 - x_offset;
				dz2 = z2 - z_offset;

				cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

				if ( cyclic >= 0.0 )
				{

					*triangle_offset = internal_triangle_offset;

					return ( TRUE );
				}
			}
		}

		if ( even_triangle )
		{

			point = points + point_ref[0].point;

			x0 = point->x;
			z0 = point->z;

			point = points + point_ref[1].point;

			x1 = point->x;
			z1 = point->z;

			point = points + point_ref[2].point;

			x2 = point->x;
			z2 = point->z;

			even_triangle = FALSE;
		}
		else
		{

			point = points + point_ref[0].point;

			x0 = point->x;
			z0 = point->z;

			point = points + point_ref[2].point;

			x1 = point->x;
			z1 = point->z;

			point = points + point_ref[1].point;

			x2 = point->x;
			z2 = point->z;

			even_triangle = TRUE;
		}

		point_ref++;

		internal_triangle_offset++;
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int host_point_inside_terrain_fan_word_triangle ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	terrain_3d_point_word_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_word_references;

	point = points + point_ref->point;

	x0 = point->x;
	z0 = point->z;

	point_ref++;

	point_ref += triangle_offset;

	point = points + point_ref->point;

	x1 = point->x;
	z1 = point->z;

	point_ref++;

	point = points + point_ref->point;

	x2 = point->x;
	z2 = point->z;

	point_ref++;

	//
	// Check against the first side
	//

	dx1 = x0 - x1;
	dz1 = z0 - z1;
	dx2 = x1 - x_offset;
	dz2 = z1 - z_offset;

	cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

	if ( cyclic >= 0.0 )
	{

		//
		// Check against the third side
		//

		dx1 = x2 - x0;
		dz1 = z2 - z0;
		dx2 = x0 - x_offset;
		dz2 = z0 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the second side
			//

			dx1 = x1 - x2;
			dz1 = z1 - z2;
			dx2 = x2 - x_offset;
			dz2 = z2 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				return ( TRUE );
			}
		}
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int host_point_inside_terrain_fan_byte_triangle ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	terrain_3d_point_byte_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_byte_references;

	point = points + point_ref->point;

	x0 = point->x;
	z0 = point->z;

	point_ref++;

	point_ref += triangle_offset;

	point = points + point_ref->point;

	x1 = point->x;
	z1 = point->z;

	point_ref++;

	point = points + point_ref->point;

	x2 = point->x;
	z2 = point->z;

	point_ref++;

	//
	// Check against the first side
	//

	dx1 = x0 - x1;
	dz1 = z0 - z1;
	dx2 = x1 - x_offset;
	dz2 = z1 - z_offset;

	cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

	if ( cyclic >= 0.0 )
	{

		//
		// Check against the third side
		//

		dx1 = x2 - x0;
		dz1 = z2 - z0;
		dx2 = x0 - x_offset;
		dz2 = z0 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the second side
			//

			dx1 = x1 - x2;
			dz1 = z1 - z2;
			dx2 = x2 - x_offset;
			dz2 = z2 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				return ( TRUE );
			}
		}
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int host_point_inside_terrain_strip_word_triangle ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	terrain_3d_point_word_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_word_references;

	point_ref += triangle_offset;

	if ( triangle_offset & 1 )
	{

		//
		// Checking an odd triangle
		//

		point = points + point_ref->point;
		point_ref++;
		x0 = point->x;
		z0 = point->z;

		point = points + point_ref->point;
		point_ref++;
		x2 = point->x;
		z2 = point->z;

		point = points + point_ref->point;
		x1 = point->x;
		z1 = point->z;
	}
	else
	{

		//
		// Checking an even triangle
		//

		point = points + point_ref->point;
		point_ref++;
		x0 = point->x;
		z0 = point->z;

		point = points + point_ref->point;
		point_ref++;
		x1 = point->x;
		z1 = point->z;

		point = points + point_ref->point;
		x2 = point->x;
		z2 = point->z;
	}

	//
	// Check against the first side
	//

	dx1 = x0 - x1;
	dz1 = z0 - z1;
	dx2 = x1 - x_offset;
	dz2 = z1 - z_offset;

	cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

	if ( cyclic >= 0.0 )
	{

		//
		// Check against the third side
		//

		dx1 = x2 - x0;
		dz1 = z2 - z0;
		dx2 = x0 - x_offset;
		dz2 = z0 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the second side
			//

			dx1 = x1 - x2;
			dz1 = z1 - z2;
			dx2 = x2 - x_offset;
			dz2 = z2 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				return ( TRUE );
			}
		}
	}

	return ( FALSE );
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int host_point_inside_terrain_strip_byte_triangle ( const float x_offset, const float z_offset, terrain_3d_point_xz *points, int triangle_offset )
{

	float
		x0,
		z0,
		x1,
		z1,
		x2,
		z2,
		dx1,
		dz1,
		dx2,
		dz2,
		cyclic;

	terrain_3d_point_byte_reference
		*point_ref;

	terrain_3d_point_xz
		*point;

#if DEBUG_MODULE

	num_in_poly_tests++;

#endif

	point_ref = terrain_elevation_point_byte_references;

	point_ref += triangle_offset;

	if ( triangle_offset & 1 )
	{

		//
		// Checking an odd triangle
		//

		point = points + point_ref->point;
		point_ref++;
		x0 = point->x;
		z0 = point->z;

		point = points + point_ref->point;
		point_ref++;
		x2 = point->x;
		z2 = point->z;

		point = points + point_ref->point;
		x1 = point->x;
		z1 = point->z;
	}
	else
	{

		//
		// Checking an even triangle
		//

		point = points + point_ref->point;
		point_ref++;
		x0 = point->x;
		z0 = point->z;

		point = points + point_ref->point;
		point_ref++;
		x1 = point->x;
		z1 = point->z;

		point = points + point_ref->point;
		x2 = point->x;
		z2 = point->z;
	}

	//
	// Check against the first side
	//

	dx1 = x0 - x1;
	dz1 = z0 - z1;
	dx2 = x1 - x_offset;
	dz2 = z1 - z_offset;

	cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

	if ( cyclic >= 0.0 )
	{

		//
		// Check against the third side
		//

		dx1 = x2 - x0;
		dz1 = z2 - z0;
		dx2 = x0 - x_offset;
		dz2 = z0 - z_offset;

		cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

		if ( cyclic >= 0.0 )
		{

			//
			// Check against the second side
			//

			dx1 = x1 - x2;
			dz1 = z1 - z2;
			dx2 = x2 - x_offset;
			dz2 = z2 - z_offset;

			cyclic = ( dx2 * dz1 ) - ( dx1 * dz2 );

			if ( cyclic >= 0.0 )
			{

				return ( TRUE );
			}
		}
	}

	return ( FALSE );
}

/* ---- terrelev.c get_3d_terrain_point_data, verbatim but for the marked host changes (name, off-map note, N1, N3) ---- */

static float host_get_3d_terrain_point_data ( float x, float z, terrain_3d_point_data *point_data )
{

	int
		x_sector,
		z_sector,
		number_of_faces,
		number_of_points,
		triangle_offset,
		found,
		found_fast;

	float
		x_offset,
		z_offset,
		scaled_x_offset,
		scaled_z_offset,
		y,
		dx,
		dy,
		dz,
		terrain_elevation;

	terrain_3d_sector
		*terrain_sector;

	terrain_3d_face
		*terrain_face;

	terrain_3d_point_xz
		*point;

	terrain_3d_point_y
		*point_y;

	terrain_3d_surface
		*surface;

	terrain_types
		terrain_type;

	terrain_3d_point_xz
		*points_xz;

	terrain_3d_point_y
		*points_y;

#if DEBUG_MODULE
	num_elevation_tests++;
#endif

	////////////////////////////////////////
	//
	// check point is on the map
	//
	////////////////////////////////////////

	/* host (N1, patches.rs): a NaN position passes every range test below; the characterised engine reports it and uses the map corner */
	if ( ( x != x ) || ( z != z ) )
	{
		host_note_nan = TRUE;

		x = terrain_3d_min_map_x + 0.0001f;
		z = terrain_3d_min_map_z + 0.0001f;
	}

	if ( ( x < terrain_3d_min_map_x ) || ( x > terrain_3d_max_map_x ) || ( z < terrain_3d_min_map_z ) || ( z > terrain_3d_max_map_z ) )
	{
		ASSERT (!"Terrain elevation off map");
		/* host: the original asserts here and falls through to the clamp below; the clamp is what is reproduced, and it is reported */
		host_note_clamped = TRUE;

		x = bound(x, terrain_3d_min_map_x + 0.0001f, terrain_3d_max_map_x - 0.0001f);
		z = bound(z, terrain_3d_min_map_z + 0.0001f, terrain_3d_max_map_z - 0.0001f);

	}

	////////////////////////////////////////
	//
	// get terrain sector and offsets within sector
	//
	////////////////////////////////////////

	get_terrain_3d_sector (x, z, &x_sector, &z_sector);

	//
	// Setup the point_data ptr
	///

	if ( !point_data )
	{

		point_data = &system_point_data;
	}

	//
	// Detect Cuba terrain flaw area
	//

	if ( current_terrain_version.map_number == 2 )
	{

		if (	( x_sector >= 71 ) && ( x_sector <= 72 ) &&
				( z_sector >= 58 ) && ( z_sector <= 60 ) )
		{

			//
			// Fudge the terrain height stuff.
			//

			point_data->terrain_sector = NULL;

			point_data->face_normal.x = 0.0;
			point_data->face_normal.y = 1.0;
			point_data->face_normal.z = 0.0;

			point_data->terrain_type = TERRAIN_TYPE_INVALID;

			terrain_sector = &terrain_sectors[z_sector][x_sector];

			point_data->last_elevation = terrain_sector->maximum_height;

			host_note_cuba = TRUE;

			return ( terrain_sector->minimum_height );
		}
	}

	x_offset = x - ( ( float ) x_sector * TERRAIN_3D_SECTOR_SIDE_LENGTH );
	z_offset = z - ( ( float ) z_sector * TERRAIN_3D_SECTOR_SIDE_LENGTH );

	x_offset -= ( TERRAIN_3D_SECTOR_SIDE_LENGTH / 2 );
	z_offset -= ( TERRAIN_3D_SECTOR_SIDE_LENGTH / 2 );

#if X_AND_Z_ARE_FLOATS

	scaled_x_offset = x_offset;
	scaled_z_offset = z_offset;

#else

	scaled_x_offset = ( x_offset / TERRAIN_3D_XZ_SCALE );
	scaled_z_offset = ( z_offset / TERRAIN_3D_XZ_SCALE );

#endif

	////////////////////////////////////////
	//
	// attempt find fast
	//
	////////////////////////////////////////

	found = found_fast = FALSE;

	if ( point_data->terrain_sector )
	{

		ASSERT ( point_data->terrain_face );

		if ( ( x_sector == point_data->x_sector ) && ( z_sector == point_data->z_sector ) )
		{

			host_note_fast_attempted = TRUE;

			terrain_sector = point_data->terrain_sector;

			points_y = terrain_sector->points_y;

			points_xz = terrain_sector->points_xz;

			terrain_face = point_data->terrain_face;

			triangle_offset = point_data->terrain_triangle_offset;

			terrain_type = point_data->terrain_type;

			if ( terrain_sector->number_of_points < 257 )
			{

				terrain_elevation_point_byte_references = point_data->terrain_elevation_point_byte_references;

				switch ( terrain_type_information[terrain_type].polygon_type )
				{

					case POLYGON_TYPE_FAN:
					{

						if ( host_point_inside_terrain_fan_byte_triangle ( scaled_x_offset, scaled_z_offset, points_xz, triangle_offset ) )
						{

							found = TRUE;

							found_fast = TRUE;
						}

						break;
					}

					case POLYGON_TYPE_STRIP:
					{

						if ( host_point_inside_terrain_strip_byte_triangle ( scaled_x_offset, scaled_z_offset, points_xz, triangle_offset ) )
						{

							found = TRUE;

							found_fast = TRUE;
						}

						break;
					}

					default:
					{

						debug_fatal ( "Unknown polygon type in terrain elevation, sector: %d, %d", x_sector, z_sector );
					}
				}
			}
			else
			{

				terrain_elevation_point_word_references = point_data->terrain_elevation_point_word_references;

				switch ( terrain_type_information[terrain_type].polygon_type )
				{

					case POLYGON_TYPE_FAN:
					{

						if ( host_point_inside_terrain_fan_word_triangle ( scaled_x_offset, scaled_z_offset, points_xz, triangle_offset ) )
						{

							found = TRUE;

							found_fast = TRUE;
						}

						break;
					}

					case POLYGON_TYPE_STRIP:
					{

						if ( host_point_inside_terrain_strip_word_triangle ( scaled_x_offset, scaled_z_offset, points_xz, triangle_offset ) )
						{

							found = TRUE;

							found_fast = TRUE;
						}

						break;
					}

					default:
					{

						debug_fatal ( "Unknown polygon type in terrain elevation, sector: %d, %d", x_sector, z_sector );
					}
				}
			}

#if DEBUG_MODULE

			if ( found )
			{

				num_find_fast_successes++;
			}

#endif
		}
	}

	////////////////////////////////////////
	//
	// scan sector
	//
	////////////////////////////////////////

	if ( !found_fast )
	{

		terrain_sector = &terrain_sectors[z_sector][x_sector];

		points_y = terrain_sector->points_y;

		points_xz = terrain_sector->points_xz;

		if ( terrain_sector->number_of_points < 257 )
		{

			//
			// Search through all the polygon fans first of all
			//

			number_of_faces = terrain_sector->number_of_polygons;

			terrain_face = terrain_sector->polygons;

			terrain_elevation_point_byte_references = terrain_sector->point_byte_references;

			surface = terrain_sector->surface_changes;

			while ( number_of_faces-- )
			{

				surface += terrain_face->surface_change;

				number_of_points = terrain_face->number_of_points;

				terrain_type = surface->surface_id;

				if ( terrain_type_information[terrain_type].polygon_type == POLYGON_TYPE_FAN )
				{

					if ( host_point_inside_terrain_byte_fan ( scaled_x_offset, scaled_z_offset, points_xz, number_of_points, &triangle_offset ) )
					{

						found = TRUE;

						break;
					}
				}

				terrain_elevation_point_byte_references += number_of_points;

				terrain_face++;
			}

			//
			// If the point isn't over a polygon fan - search the polygon strips
			//

			if ( !found )
			{

				number_of_faces = terrain_sector->number_of_polygons;

				terrain_face = terrain_sector->polygons;

				terrain_elevation_point_byte_references = terrain_sector->point_byte_references;

				surface = terrain_sector->surface_changes;

				while ( number_of_faces-- )
				{

					surface += terrain_face->surface_change;

					number_of_points = terrain_face->number_of_points;

					terrain_type = surface->surface_id;

					switch ( terrain_type )
					{

						case TERRAIN_TYPE_HEDGE_SIDE_X:
						case TERRAIN_TYPE_HEDGE_SIDE_Z:
						case TERRAIN_TYPE_WALL_SIDE_X:
						case TERRAIN_TYPE_WALL_SIDE_Z:
						case TERRAIN_TYPE_FOREST_SIDE_BOTTOM_X:
						case TERRAIN_TYPE_FOREST_SIDE_BOTTOM_Z:
						case TERRAIN_TYPE_FOREST_SIDE_MID_X:
						case TERRAIN_TYPE_FOREST_SIDE_MID_Z:
						{

							break;
						}

						default:
						{

							if ( terrain_type_information[terrain_type].polygon_type == POLYGON_TYPE_STRIP )
							{

								if ( host_point_inside_terrain_byte_strip ( scaled_x_offset, scaled_z_offset, points_xz, number_of_points, &triangle_offset ) )
								{

									found = TRUE;

									break;
								}
							}

							break;
						}
					}

					if ( found )
					{

						break;
					}

					terrain_elevation_point_byte_references += number_of_points;

					terrain_face++;
				}
			}
		}
		else
		{

			//
			// Search through all the polygon fans first
			//

			number_of_faces = terrain_sector->number_of_polygons;

			terrain_face = terrain_sector->polygons;

			terrain_elevation_point_word_references = terrain_sector->point_word_references;

			surface = terrain_sector->surface_changes;

			while ( number_of_faces-- )
			{

				surface += terrain_face->surface_change;

				number_of_points = terrain_face->number_of_points;

				terrain_type = surface->surface_id;

				if ( terrain_type_information[terrain_type].polygon_type == POLYGON_TYPE_FAN )
				{

					if ( host_point_inside_terrain_word_fan ( scaled_x_offset, scaled_z_offset, points_xz, number_of_points, &triangle_offset ) )
					{

						found = TRUE;

						break;
					}
				}

				terrain_elevation_point_word_references += number_of_points;

				terrain_face++;
			}

			//
			// If the point isn't over a polygon fan - search the polygon strips
			//

			if ( !found )
			{

				number_of_faces = terrain_sector->number_of_polygons;

				terrain_face = terrain_sector->polygons;

				terrain_elevation_point_word_references = terrain_sector->point_word_references;

				surface = terrain_sector->surface_changes;

				while ( number_of_faces-- )
				{

					surface += terrain_face->surface_change;

					number_of_points = terrain_face->number_of_points;

					terrain_type = surface->surface_id;

					if ( terrain_type_information[terrain_type].polygon_type == POLYGON_TYPE_STRIP )
					{

						switch ( terrain_type )
						{

							case TERRAIN_TYPE_HEDGE_SIDE_X:
							case TERRAIN_TYPE_HEDGE_SIDE_Z:
							case TERRAIN_TYPE_WALL_SIDE_X:
							case TERRAIN_TYPE_WALL_SIDE_Z:
							case TERRAIN_TYPE_FOREST_SIDE_BOTTOM_X:
							case TERRAIN_TYPE_FOREST_SIDE_BOTTOM_Z:
							case TERRAIN_TYPE_FOREST_SIDE_MID_X:
							case TERRAIN_TYPE_FOREST_SIDE_MID_Z:
							{

								break;
							}

							default:
							{

								if ( host_point_inside_terrain_word_strip ( scaled_x_offset, scaled_z_offset, points_xz, number_of_points, &triangle_offset ) )
								{

									found = TRUE;

									break;
								}

								break;
							}
						}
					}

					if ( found )
					{

						break;
					}

					terrain_elevation_point_word_references += number_of_points;

					terrain_face++;
				}
			}
		}
	}

	////////////////////////////////////////
	//
	// get terrain elevation and face normal
	//
	////////////////////////////////////////

	if ( found )
	{

		if ( !found_fast )
		{

#if DEBUG_TERRAIN
			debug_log ( "Elevation: %s, %d: SLOW", file, line );
#endif

			point_data->x_sector = x_sector;

			point_data->z_sector = z_sector;

			point_data->terrain_sector = terrain_sector;

			point_data->terrain_face = terrain_face;

			if ( terrain_sector->number_of_points < 257 )
			{

				point_data->terrain_elevation_point_byte_references = terrain_elevation_point_byte_references;
			}
			else
			{

				point_data->terrain_elevation_point_word_references = terrain_elevation_point_word_references;
			}

			point_data->terrain_triangle_offset = triangle_offset;

			point_data->terrain_type = terrain_type;

			{

				vec3d
					vector1,
					vector2,
					normal;

				terrain_3d_point_xz
					*point0xz,
					*point1xz,
					*point2xz;

				terrain_3d_point_y
					*point0y,
					*point1y,
					*point2y;

				float
					y0,
					y1,
					y2;

				switch ( terrain_type_information[terrain_type].polygon_type )
				{

					case POLYGON_TYPE_FAN:
					{

						if ( terrain_sector->number_of_points < 257 )
						{

							point0xz = points_xz + terrain_elevation_point_byte_references[0].point;
							point0y = points_y + terrain_elevation_point_byte_references[0].point;

							point1xz = points_xz + terrain_elevation_point_byte_references[triangle_offset + 1].point;
							point1y = points_y + terrain_elevation_point_byte_references[triangle_offset + 1].point;

							point2xz = points_xz + terrain_elevation_point_byte_references[triangle_offset + 2].point;
							point2y = points_y + terrain_elevation_point_byte_references[triangle_offset + 2].point;
						}
						else
						{

							point0xz = points_xz + terrain_elevation_point_word_references[0].point;
							point0y = points_y + terrain_elevation_point_word_references[0].point;

							point1xz = points_xz + terrain_elevation_point_word_references[triangle_offset + 1].point;
							point1y = points_y + terrain_elevation_point_word_references[triangle_offset + 1].point;

							point2xz = points_xz + terrain_elevation_point_word_references[triangle_offset + 2].point;
							point2y = points_y + terrain_elevation_point_word_references[triangle_offset + 2].point;
						}

						break;
					}

					case POLYGON_TYPE_STRIP:
					{

						if ( terrain_sector->number_of_points < 257 )
						{

							if ( triangle_offset & 1 )
							{

								point0xz = points_xz + terrain_elevation_point_byte_references[triangle_offset].point;
								point0y = points_y + terrain_elevation_point_byte_references[triangle_offset].point;

								point1xz = points_xz + terrain_elevation_point_byte_references[triangle_offset + 2].point;
								point1y = points_y + terrain_elevation_point_byte_references[triangle_offset + 2].point;

								point2xz = points_xz + terrain_elevation_point_byte_references[triangle_offset + 1].point;
								point2y = points_y + terrain_elevation_point_byte_references[triangle_offset + 1].point;
							}
							else
							{

								point0xz = points_xz + terrain_elevation_point_byte_references[triangle_offset].point;
								point0y = points_y + terrain_elevation_point_byte_references[triangle_offset].point;

								point1xz = points_xz + terrain_elevation_point_byte_references[triangle_offset + 1].point;
								point1y = points_y + terrain_elevation_point_byte_references[triangle_offset + 1].point;

								point2xz = points_xz + terrain_elevation_point_byte_references[triangle_offset + 2].point;
								point2y = points_y + terrain_elevation_point_byte_references[triangle_offset + 2].point;
							}
						}
						else
						{

							if ( triangle_offset & 1 )
							{

								point0xz = points_xz + terrain_elevation_point_word_references[triangle_offset].point;
								point0y = points_y + terrain_elevation_point_word_references[triangle_offset].point;

								point1xz = points_xz + terrain_elevation_point_word_references[triangle_offset + 2].point;
								point1y = points_y + terrain_elevation_point_word_references[triangle_offset + 2].point;

								point2xz = points_xz + terrain_elevation_point_word_references[triangle_offset + 1].point;
								point2y = points_y + terrain_elevation_point_word_references[triangle_offset + 1].point;
							}
							else
							{

								point0xz = points_xz + terrain_elevation_point_word_references[triangle_offset].point;
								point0y = points_y + terrain_elevation_point_word_references[triangle_offset].point;

								point1xz = points_xz + terrain_elevation_point_word_references[triangle_offset + 1].point;
								point1y = points_y + terrain_elevation_point_word_references[triangle_offset + 1].point;

								point2xz = points_xz + terrain_elevation_point_word_references[triangle_offset + 2].point;
								point2y = points_y + terrain_elevation_point_word_references[triangle_offset + 2].point;
							}
						}

						break;
					}
				}

				//
				// Get the y components in world coordinates:
				//

				y0 = terrain_3d_map_minimum_height + ( point0y->y * terrain_3d_map_scaled_height_difference );
				y1 = terrain_3d_map_minimum_height + ( point1y->y * terrain_3d_map_scaled_height_difference );
				y2 = terrain_3d_map_minimum_height + ( point2y->y * terrain_3d_map_scaled_height_difference );

				vector2.x = ( float ) ( point2xz->x - point0xz->x );
				vector2.y = y2 - y0;
				vector2.z = ( float ) ( point2xz->z - point0xz->z );

				vector1.x = ( float ) ( point1xz->x - point0xz->x );
				vector1.y = y1 - y0;
				vector1.z = ( float ) ( point1xz->z - point0xz->z );

				//
				// The x & z parts of the vectors have been scaled by ( 65535 / 2048 ), so do the same to the y vectors.
				//

				vector1.y /= TERRAIN_3D_XZ_SCALE;	//( 65535.0 / TERRAIN_3D_SECTOR_SIDE_LENGTH );
				vector2.y /= TERRAIN_3D_XZ_SCALE;	//( 65535.0 / TERRAIN_3D_SECTOR_SIDE_LENGTH );

				normal.x = ( vector1.y * vector2.z ) - ( vector1.z * vector2.y );
				normal.y = ( vector1.z * vector2.x ) - ( vector1.x * vector2.z );
				normal.z = ( vector1.x * vector2.y ) - ( vector1.y * vector2.x );

				normalise_3d_vector (&normal);

				/* host (N3, patches.rs): a degenerate face has no normal; take the up vector, as the original's commented-out fallback does */
				if ( !( normal.y > 0.00001f ) )
				{
					normal.x = 0.0;
					normal.y = 1.0;
					normal.z = 0.0;
				}

				point_data->face_normal = normal;
			}
		}
#if DEBUG_TERRAIN
		else
		{

			debug_log ( "Elevation: %s, %d: FAST", file, line );
		}
#endif


		//
		// Here the find_fast structure is guaranteed to have the correct information in it.
		//

		if ( terrain_sector->number_of_points < 257 )
		{

			point = points_xz + terrain_elevation_point_byte_references[triangle_offset + 1].point;
			point_y = points_y + terrain_elevation_point_byte_references[triangle_offset + 1].point;
		}
		else
		{

			point = points_xz + terrain_elevation_point_word_references[triangle_offset + 1].point;
			point_y = points_y + terrain_elevation_point_word_references[triangle_offset + 1].point;
		}

#if X_AND_Z_ARE_FLOATS

		dx = x_offset - point->x;
		dz = z_offset - point->z;

#else

		dx = x_offset - ( ( ( float ) point->x ) * TERRAIN_3D_XZ_SCALE );
		dz = z_offset - ( ( ( float ) point->z ) * TERRAIN_3D_XZ_SCALE );

#endif

		dy = ( ( point_data->face_normal.x * dx ) + ( point_data->face_normal.z * dz ) ) / point_data->face_normal.y;

		y = terrain_3d_map_minimum_height + ( ( ( float ) point_y->y ) * terrain_3d_map_scaled_height_difference );

		terrain_elevation = y - dy;

/*
		//DEBUG//
		if (point_data->face_normal.y < 0.00001)
		{
			if (point_data->face_normal.y < 0.0)
			{
				debug_log ("Face normal is undercut %f", point_data->face_normal.y);
			}
			else
			{
				debug_log ("Face normal y component is %f", point_data->face_normal.y);
			}

			point_data->face_normal.x = 0.0;
			point_data->face_normal.y = 1.0;
			point_data->face_normal.z = 0.0;

			terrain_elevation = terrain_sector->maximum_height;
		}
		//DEBUG//
		*/
	}
	else
	{

		debug_log ( "Failed to locate a terrain face (x = %.4f, z = %.4f, x_sector = %d, z_sector = %d, x_offset = %f, z_offset = %f",
																				x, z, x_sector, z_sector, x_offset + 1024, z_offset + 1024 );

#if DEBUG_MODULE

		num_elevation_test_failures++;

#endif

		point_data->terrain_sector = NULL;

		point_data->face_normal.x = 0.0;
		point_data->face_normal.y = 1.0;
		point_data->face_normal.z = 0.0;

		point_data->terrain_type = TERRAIN_TYPE_INVALID;

		terrain_elevation = terrain_sector->maximum_height;
	}

	////////////////////////////////////////
	//
	// return values
	//
	////////////////////////////////////////

	if ( ( terrain_elevation < -32000 ) || ( terrain_elevation > 32000 ) )
	{

		terrain_elevation = 0;
	}

	point_data->last_elevation = terrain_elevation;

	host_note_found = found;
	host_note_fast_hit = found_fast;

	return ( terrain_elevation );
}

/* ======================================================================================================================== */
/* the host's own code: the loader, the hint, the query wrapper, the fast grid                                              */
/* ======================================================================================================================== */

#include "terrain_ref.h"

#define HOST_TERRAIN_MAX_SECTORS 1048576

struct host_terrain
{
	host_terrain_facts facts;
	terrain_3d_sector *sector_memory;
	terrain_3d_sector **rows;
	const uint8_t *grid;
	size_t grid_length;
	uint64_t key; /* this map instance's identity, folded into every hint it issues */
	float *generated; /* the fast grid built here for a map that has no terrain.sim (grid then points at it) */
};

static int fail (char *error, size_t error_size, const char *format, ...) __attribute__ ((format (printf, 3, 4)));

static int fail (char *error, size_t error_size, const char *format, ...)
{
	va_list args;

	if (error && error_size)
	{
		va_start (args, format);
		vsnprintf (error, error_size, format, args);
		va_end (args);
		error[error_size - 1] = '\0';
	}

	return -1;
}

/* a cursor over the sector file; every read is bounds-checked and a short read marks the cursor bad */
struct cursor
{
	const uint8_t *p, *end;
	int bad;
};

static uint32_t read_u (struct cursor *c, int bytes)
{
	uint32_t value = 0;
	int i;

	if ((size_t) (c->end - c->p) < (size_t) bytes)
	{
		c->bad = TRUE;
		c->p = c->end;
		return 0;
	}

	for (i = 0; i < bytes; i++)
	{
		value |= (uint32_t) c->p[i] << (8 * i);
	}

	c->p += bytes;

	return value;
}

static float read_f (struct cursor *c)
{
	uint32_t bits = read_u (c, 4);
	float value;

	memcpy (&value, &bits, sizeof (value));

	return value;
}

/* a span of `length` bytes at `offset` lies inside a file of `total` bytes (64-bit, so 32-bit offsets and lengths cannot wrap) */
static int span_inside (uint64_t offset, uint64_t length, uint64_t total)
{
	return offset <= total && length <= total - offset;
}

static uint64_t align4 (uint64_t value)
{
	return (value + 3u) & ~(uint64_t) 3u;
}

static void host_terrain_free_sectors (host_terrain *map)
{
	if (map->sector_memory)
	{
		size_t count = (size_t) map->facts.sector_x_max * (size_t) map->facts.sector_z_max, i;

		for (i = 0; i < count; i++)
		{
			free (map->sector_memory[i].block);
		}

		free (map->sector_memory);
	}

	free (map->rows);
	map->sector_memory = NULL;
	map->rows = NULL;
}

void host_terrain_close (host_terrain *map)
{
	if (map)
	{
		host_terrain_free_sectors (map);
		free (map->generated);
		free (map);
	}
}
static int host_terrain_generate_grid (host_terrain *map, char *error, size_t error_size);


/* terrdata.c load_3d_terrain, for the facts and sectors the solver reads, from memory. The file offsets are into `geometry` (terrain.ffp). */
host_terrain *host_terrain_open (const uint8_t *sector, size_t sector_length, const uint8_t *geometry, size_t geometry_length, const uint8_t *grid, size_t grid_length,
	int32_t map_number, uint64_t map_key, char *error, size_t error_size)
{
	host_terrain *map;
	struct cursor c;
	fenv_t saved;
	int width, height, x, z;
	uint32_t version;
	uint64_t geometry_total = geometry_length;

	if (!sector || !geometry)
	{
		fail (error, error_size, "terrain: no sector or geometry data");
		return NULL;
	}

	c.p = sector;
	c.end = sector + sector_length;
	c.bad = FALSE;

	version = read_u (&c, 4);
	width = (int) read_u (&c, 2);
	height = (int) read_u (&c, 2);

	if (c.bad || width < 1 || height < 1 || (size_t) width * (size_t) height > HOST_TERRAIN_MAX_SECTORS)
	{
		fail (error, error_size, "terrain: the sector file header is malformed (%d by %d sectors)", width, height);
		return NULL;
	}

	map = (host_terrain *) calloc (1, sizeof (*map));

	if (!map)
	{
		fail (error, error_size, "terrain: out of memory");
		return NULL;
	}

	/* the derived float facts are computed as the engine computes them, under round-toward-zero */
	fegetenv (&saved);
	fesetround (FE_TOWARDZERO);

	map->facts.sector_x_max = width;
	map->facts.sector_z_max = height;
	map->facts.map_number = map_number;
	map->key = map_key;
	map->facts.version = version;
	map->facts.min_x = 0.0;
	map->facts.min_z = 0.0;
	map->facts.max_x = ( ( float ) ( width * TERRAIN_3D_SECTOR_SIDE_LENGTH ) - 1 );
	map->facts.max_z = ( ( float ) ( height * TERRAIN_3D_SECTOR_SIDE_LENGTH ) - 1 );
	map->facts.min_height = read_f (&c);
	map->facts.max_height = read_f (&c);
	map->facts.scaled_height_difference = ( map->facts.max_height - map->facts.min_height ) * TERRAIN_3D_Y_SCALE;

	fesetenv (&saved);

	if (c.bad || !isfinite (map->facts.min_height) || !isfinite (map->facts.max_height) || !isfinite (map->facts.scaled_height_difference))
	{
		fail (error, error_size, "terrain: the sector file header heights are malformed");
		host_terrain_close (map);
		return NULL;
	}

	map->sector_memory = (terrain_3d_sector *) calloc ((size_t) width * (size_t) height, sizeof (terrain_3d_sector));
	map->rows = (terrain_3d_sector **) calloc ((size_t) height, sizeof (terrain_3d_sector *));

	if (!map->sector_memory || !map->rows)
	{
		fail (error, error_size, "terrain: out of memory");
		host_terrain_close (map);
		return NULL;
	}

	for (z = 0; z < height; z++)
	{
		map->rows[z] = map->sector_memory + (size_t) z * (size_t) width;

		for (x = 0; x < width; x++)
		{
			terrain_3d_sector *s = &map->rows[z][x];
			uint32_t off_y, off_xz, off_ref, off_poly, off_surface;
			uint64_t refs = 0, bits = 0, ref_bytes, offset_y, offset_ref, offset_poly, offset_surface, offset_face_reference, offset_face_type, block_size;
			size_t index = 0, faces, points, f, r;
			const terrain_3d_face *face_source;
			unsigned char *block;

			s->number_of_points = (unsigned short) read_u (&c, 2);
			(void) read_u (&c, 2);
			s->number_of_polygons = (unsigned short) read_u (&c, 2);
			s->radius = (unsigned short) read_u (&c, 2);
			s->minimum_height = (short) read_u (&c, 2);
			s->maximum_height = (short) read_u (&c, 2);
			s->average_height = (short) read_u (&c, 2);
			(void) read_u (&c, 2);
			off_y = read_u (&c, 4);
			off_xz = read_u (&c, 4);
			off_ref = read_u (&c, 4);
			off_poly = read_u (&c, 4);
			off_surface = read_u (&c, 4);
			(void) read_u (&c, 4);
			(void) read_u (&c, 4);
			(void) read_u (&c, 4);

			if (c.bad)
			{
				fail (error, error_size, "terrain: the sector file ends inside sector %d,%d", x, z);
				host_terrain_close (map);
				return NULL;
			}

			faces = s->number_of_polygons;
			points = s->number_of_points;
			ref_bytes = (points < 257) ? 1 : 2;

			/* every span is checked in 64-bit arithmetic against the file length before any pointer into the file is formed (offsets are 32-bit) */
			if (!span_inside (off_y, 2u * points, geometry_total) || !span_inside (off_xz, 4u * points, geometry_total) || !span_inside (off_poly, faces, geometry_total))
			{
				fail (error, error_size, "terrain: sector %d,%d points or faces lie outside the geometry file", x, z);
				host_terrain_close (map);
				return NULL;
			}

			/* a face is one byte (alignment 1), so reading the faces in the file is safe once the span is inside */
			face_source = (const terrain_3d_face *) (geometry + off_poly);

			for (f = 0; f < faces; f++)
			{
				if (face_source[f].number_of_points < 3)
				{
					fail (error, error_size, "terrain: sector %d,%d face %u has fewer than three points", x, z, (unsigned) f);
					host_terrain_close (map);
					return NULL;
				}

				refs += face_source[f].number_of_points;
				bits += face_source[f].surface_change;
			}

			if (!span_inside (off_ref, refs * ref_bytes, geometry_total) || !span_inside (off_surface, bits + 1, geometry_total))
			{
				fail (error, error_size, "terrain: sector %d,%d references or surfaces lie outside the geometry file", x, z);
				host_terrain_close (map);
				return NULL;
			}

			/*
			 * The sector's geometry is copied into one block of the host's own, 4-aligned, so that the typed reads of the solver (16-bit coordinates and
			 * references, 32-bit offsets) are never made through a misaligned pointer into the file, whatever offsets the file holds. The solver
			 * reads the same bytes, so its arithmetic is unchanged.
			 */
			offset_y = 4u * (uint64_t) points;
			offset_ref = offset_y + align4 (2u * (uint64_t) points);
			offset_poly = offset_ref + align4 (refs * ref_bytes);
			offset_surface = offset_poly + align4 (faces);
			offset_face_reference = offset_surface + align4 (bits + 1);
			offset_face_type = offset_face_reference + 4u * (uint64_t) faces;
			block_size = offset_face_type + (uint64_t) faces;

			block = (unsigned char *) malloc ((size_t) (block_size ? block_size : 1));

			if (!block)
			{
				fail (error, error_size, "terrain: out of memory");
				host_terrain_close (map);
				return NULL;
			}

			s->block = block;

			memcpy (block, geometry + off_xz, 4u * (size_t) points);
			memcpy (block + offset_y, geometry + off_y, 2u * (size_t) points);
			memcpy (block + offset_ref, geometry + off_ref, (size_t) (refs * ref_bytes));
			memcpy (block + offset_poly, geometry + off_poly, faces);
			memcpy (block + offset_surface, geometry + off_surface, (size_t) (bits + 1));

			s->points_xz = (terrain_3d_point_xz *) block;
			s->points_y = (terrain_3d_point_y *) (block + offset_y);
			s->point_byte_references = (terrain_3d_point_byte_reference *) (block + offset_ref);
			s->polygons = (terrain_3d_face *) (block + offset_poly);
			s->surface_changes = (terrain_3d_surface *) (block + offset_surface);
			s->face_reference_offset = (uint32_t *) (block + offset_face_reference);
			s->face_type = block + offset_face_type;
			s->total_references = (uint32_t) refs;

			for (r = 0; r < refs; r++)
			{
				unsigned point = (ref_bytes == 1) ? s->point_byte_references[r].point : s->point_word_references[r].point;

				if (point >= points)
				{
					fail (error, error_size, "terrain: sector %d,%d reference %u names point %u of %u", x, z, (unsigned) r, point, (unsigned) points);
					host_terrain_close (map);
					return NULL;
				}
			}

			refs = 0;

			for (f = 0; f < faces; f++)
			{
				/* terrelev.c: `surface += face->surface_change` before the face's type is read */
				index += s->polygons[f].surface_change;
				s->face_reference_offset[f] = (uint32_t) refs;
				s->face_type[f] = s->surface_changes[index].surface_id;
				refs += s->polygons[f].number_of_points;

				if (s->face_type[f] >= TERRAIN_TYPE_LAST)
				{
					fail (error, error_size, "terrain: sector %d,%d face %u has surface type %u", x, z, (unsigned) f, (unsigned) s->face_type[f]);
					host_terrain_close (map);
					return NULL;
				}
			}
		}
	}

	if (grid)
	{
		struct cursor g;
		int gw, gh;
		float gx, gz;

		g.p = grid;
		g.end = grid + grid_length;
		g.bad = FALSE;
		gw = (int) read_u (&g, 4);
		gh = (int) read_u (&g, 4);
		gx = read_f (&g);
		gz = read_f (&g);

		if (g.bad || gw < 1 || gh < 1 || !isfinite (gx) || !isfinite (gz) || !(gx > 0.0f) || !(gz > 0.0f) || (uint64_t) gw * (uint64_t) gh * 4u > (uint64_t) (g.end - g.p))
		{
			fail (error, error_size, "terrain: the fast elevation grid is malformed (%d by %d, grid sizes %g, %g, %zu bytes)", gw, gh, (double) gx, (double) gz, grid_length);
			host_terrain_close (map);
			return NULL;
		}

		map->grid = g.p;
		map->grid_length = (size_t) (g.end - g.p);
		map->facts.fast_width = gw;
		map->facts.fast_height = gh;
		map->facts.fast_x_grid = gx;
		map->facts.fast_z_grid = gz;
	}
	else if (host_terrain_generate_grid (map, error, error_size))
	{
		/* a map with no terrain.sim: the original generates the grid at load from the point query (and saves it); so does the host, in memory */
		host_terrain_close (map);
		return NULL;
	}

	return map;
}

void host_terrain_get_facts (const host_terrain *map, host_terrain_facts *out)
{
	*out = map->facts;
}

/* the solver reads its map facts from file statics: point them at this map for the duration of a query */
static void host_terrain_enter (const host_terrain *map)
{
	terrain_3d_min_map_x = map->facts.min_x;
	terrain_3d_min_map_z = map->facts.min_z;
	terrain_3d_max_map_x = map->facts.max_x;
	terrain_3d_max_map_z = map->facts.max_z;
	terrain_3d_map_minimum_height = map->facts.min_height;
	terrain_3d_map_scaled_height_difference = map->facts.scaled_height_difference;
	terrain_sectors = map->rows;
	current_terrain_version.map_number = map->facts.map_number;
}

/*
 * terrsel.c generate_terrain_simple_elevation (256.0), for a map that has no terrain.sim.  The original builds the fast grid at load by asking the point
 * query for every gridpost, on the shared record, in this order (x fastest), and then saves the file; the host does the same with its own record (cleared
 * at the start, as the original's shared record is cleared when a map is loaded) and keeps the grid in memory instead of writing it into the installation.
 * The expressions, the power-of-two rounding of the dimensions and the sea rule (a sea gridpost is -5000, any other is its elevation plus 2) are the
 * original's, kept in order; the sampling runs under round-toward-zero as the engine's does.
 */
static int host_terrain_generate_grid (host_terrain *map, char *error, size_t error_size)
{
	const float elevation_grid_size = 256.0f;
	int x_size, z_size, x_grid, z_grid, power, x_rounded, z_rounded;
	float x_elevation_grid_size, z_elevation_grid_size;
	terrain_3d_point_data record;
	fenv_t saved;
	float *grid;

	x_size = ( int ) ( ( map->facts.sector_x_max * TERRAIN_3D_SECTOR_SIDE_LENGTH ) / elevation_grid_size );
	z_size = ( int ) ( ( map->facts.sector_z_max * TERRAIN_3D_SECTOR_SIDE_LENGTH ) / elevation_grid_size );

	x_rounded = z_rounded = FALSE;

	for ( power = 2; power < 32768; power *= 2 )
	{
		if ( !x_rounded )
		{
			if ( ( x_size == power ) || ( x_size < power ) )
			{
				x_size = power;
				x_rounded = TRUE;
			}
		}

		if ( !z_rounded )
		{
			if ( ( z_size == power ) || ( z_size < power ) )
			{
				z_size = power;
				z_rounded = TRUE;
			}
		}
	}

	x_elevation_grid_size = ( float ) ( ( map->facts.sector_x_max * TERRAIN_3D_SECTOR_SIDE_LENGTH ) / x_size );
	z_elevation_grid_size = ( float ) ( ( map->facts.sector_z_max * TERRAIN_3D_SECTOR_SIDE_LENGTH ) / z_size );

	x_size++;
	z_size++;

	if ( ( x_size < 3 ) || ( z_size < 3 ) || ( ( int64_t ) x_size * ( int64_t ) z_size > 16777216 ) )
	{
		return fail ( error, error_size, "terrain: a generated fast grid of %d by %d posts is outside the supported size", x_size, z_size );
	}

	grid = ( float * ) malloc ( ( size_t ) x_size * ( size_t ) z_size * sizeof ( float ) );

	if ( !grid )
	{
		return fail ( error, error_size, "terrain: out of memory" );
	}

	memset ( &record, 0, sizeof ( record ) );

	fegetenv ( &saved );
	fesetround ( FE_TOWARDZERO );

	host_terrain_enter ( map );

	for ( z_grid = 0; z_grid < z_size; z_grid++ )
	{
		float
			*elevations;

		elevations = grid + ( ( size_t ) z_grid * ( size_t ) x_size );

		for ( x_grid = 0; x_grid < x_size; x_grid++ )
		{
			float
				x,
				z,
				height;

			terrain_types
				type;

			x = x_grid * x_elevation_grid_size;
			z = z_grid * z_elevation_grid_size;

			x = bound ( x, 0, terrain_3d_max_map_x );
			z = bound ( z, 0, terrain_3d_max_map_z );

			height = host_get_3d_terrain_point_data ( x, z, &record );

			//
			// Find if sea or not
			//

			type = record.terrain_type;

			switch ( type )
			{
				case TERRAIN_TYPE_SEA:
				{
					height = -5000;

					break;
				}
				default:
				{
					height += 2;
				}
			}

			if ( type == TERRAIN_TYPE_SEA )
			{
				height = -5000;
			}

			*elevations++ = height;
		}
	}

	fesetenv ( &saved );

	map->generated = grid;
	map->grid = ( const uint8_t * ) grid;
	map->grid_length = ( size_t ) x_size * ( size_t ) z_size * sizeof ( float );
	map->facts.fast_width = x_size;
	map->facts.fast_height = z_size;
	map->facts.fast_x_grid = x_elevation_grid_size;
	map->facts.fast_z_grid = z_elevation_grid_size;

	return 0;
}

static int finite3 (const float *v)
{
	return isfinite (v[0]) && isfinite (v[1]) && isfinite (v[2]);
}

/*
 * The tag of a hint: a keyed 64-bit FNV-1a over the issuing map instance's key and every field of the hint. It binds a hint to the map that issued it
 * (a hint from another map, or from an earlier instance of the same files, is refused even when its indices happen to be valid) and to the normal and
 * triangle it carries, so an edited hint is refused. It needs no table of issued hints. It is an integrity check against accident and staleness; it is
 * not a secret, and the structural checks below still run on a hint whose tag matches.
 */
static uint64_t hint_tag (const host_terrain *map, const host_terrain_hint *hint)
{
	uint64_t hash = 14695981039346656037ull;
	uint32_t words[9];
	int w, byte;

	words[0] = (uint32_t) hint->x_sector;
	words[1] = (uint32_t) hint->z_sector;
	words[2] = hint->face_index;
	words[3] = hint->reference_offset;
	words[4] = (uint32_t) hint->triangle_offset;
	words[5] = (uint32_t) hint->type;
	memcpy (&words[6], &hint->normal[0], sizeof (uint32_t));
	memcpy (&words[7], &hint->normal[1], sizeof (uint32_t));
	memcpy (&words[8], &hint->normal[2], sizeof (uint32_t));

	for (byte = 0; byte < 8; byte++)
	{
		hash ^= (uint64_t) ((map->key >> (8 * byte)) & 0xffu);
		hash *= 1099511628211ull;
	}

	for (w = 0; w < 9; w++)
	{
		for (byte = 0; byte < 4; byte++)
		{
			hash ^= (uint64_t) ((words[w] >> (8 * byte)) & 0xffu);
			hash *= 1099511628211ull;
		}
	}

	return hash;
}

/* turns a hint back into the record the original solver expects, checking everything it will read */
static int record_from_hint (const host_terrain *map, const host_terrain_hint *hint, terrain_3d_point_data *record, char *error, size_t error_size)
{
	terrain_3d_sector *s;
	const terrain_3d_face *face;

	memset (record, 0, sizeof (*record));

	if (!hint)
	{
		return 0;
	}

	if (hint->tag != hint_tag (map, hint))
	{
		return fail (error, error_size, "terrain hint: not issued by this map, or altered since it was issued");
	}

	if (hint->x_sector < 0 || hint->x_sector >= map->facts.sector_x_max || hint->z_sector < 0 || hint->z_sector >= map->facts.sector_z_max)
	{
		return fail (error, error_size, "terrain hint: sector %d,%d is not on the map", (int) hint->x_sector, (int) hint->z_sector);
	}

	s = &map->rows[hint->z_sector][hint->x_sector];

	if (hint->face_index >= s->number_of_polygons)
	{
		return fail (error, error_size, "terrain hint: face %u is not in the sector", (unsigned) hint->face_index);
	}

	face = &s->polygons[hint->face_index];

	if (hint->reference_offset != s->face_reference_offset[hint->face_index] || hint->type < 0 || hint->type >= TERRAIN_TYPE_LAST
		|| hint->type != s->face_type[hint->face_index] || terrain_type_information[hint->type].polygon_type == POLYGON_TYPE_UNKNOWN)
	{
		return fail (error, error_size, "terrain hint: the face, its reference offset and its surface type do not agree");
	}

	/* a normal the solver stored is finite with a positive y component (N3 guarantees it); a zero normal would make a fast hit compute 0/0 */
	if (hint->triangle_offset < 0 || hint->triangle_offset > (int) face->number_of_points - 3 || !finite3 (hint->normal) || !(hint->normal[1] > 0.00001f))
	{
		return fail (error, error_size, "terrain hint: the triangle or the stored normal is not valid");
	}

	record->x_sector = hint->x_sector;
	record->z_sector = hint->z_sector;
	record->terrain_sector = s;
	record->terrain_face = s->polygons + hint->face_index;

	if (s->number_of_points < 257)
	{
		record->terrain_elevation_point_byte_references = s->point_byte_references + hint->reference_offset;
	}
	else
	{
		record->terrain_elevation_point_word_references = s->point_word_references + hint->reference_offset;
	}

	record->terrain_triangle_offset = hint->triangle_offset;
	record->terrain_type = hint->type;
	record->face_normal.x = hint->normal[0];
	record->face_normal.y = hint->normal[1];
	record->face_normal.z = hint->normal[2];

	return 0;
}

static void hint_from_record (const host_terrain *map, const terrain_3d_point_data *record, host_terrain_answer *answer)
{
	const terrain_3d_sector *s = record->terrain_sector;

	answer->has_hint = FALSE;

	if (!s)
	{
		return;
	}

	answer->has_hint = TRUE;
	answer->hint.x_sector = record->x_sector;
	answer->hint.z_sector = record->z_sector;
	answer->hint.face_index = (uint32_t) (record->terrain_face - s->polygons);
	answer->hint.reference_offset = (s->number_of_points < 257) ? (uint32_t) (record->terrain_elevation_point_byte_references - s->point_byte_references)
		: (uint32_t) (record->terrain_elevation_point_word_references - s->point_word_references);
	answer->hint.triangle_offset = record->terrain_triangle_offset;
	answer->hint.type = record->terrain_type;
	answer->hint.normal[0] = record->face_normal.x;
	answer->hint.normal[1] = record->face_normal.y;
	answer->hint.normal[2] = record->face_normal.z;
	answer->hint.tag = hint_tag (map, &answer->hint);
}

int host_terrain_query (const host_terrain *map, float x, float z, const host_terrain_hint *hint, host_terrain_answer *out, char *error, size_t error_size)
{
	terrain_3d_point_data record;
	fenv_t saved;

	memset (out, 0, sizeof (*out));

	if (record_from_hint (map, hint, &record, error, error_size))
	{
		return -1;
	}

	fegetenv (&saved);
	fesetround (FE_TOWARDZERO);

	host_terrain_enter (map);

	host_note_clamped = FALSE;
	host_note_nan = FALSE;
	host_note_fast_attempted = FALSE;
	host_note_fast_hit = FALSE;
	host_note_found = FALSE;
	host_note_cuba = FALSE;

	out->elevation = host_get_3d_terrain_point_data (x, z, &record);

	fesetenv (&saved);

	out->last_elevation = record.last_elevation;
	out->normal[0] = record.face_normal.x;
	out->normal[1] = record.face_normal.y;
	out->normal[2] = record.face_normal.z;
	out->type = record.terrain_type;
	out->fast_attempted = host_note_fast_attempted;
	out->fast_hit = host_note_fast_hit;
	out->found = host_note_cuba ? TRUE : host_note_found;
	out->clamped = host_note_clamped;
	out->cuba = host_note_cuba;
	out->nan_position = host_note_nan;

	hint_from_record (map, &record, out);

	return 0;
}

/* terrsel.c get_3d_terrain_fast_elevation, over the grid of terrain.sim, expression for expression */
int host_terrain_fast_elevation (const host_terrain *map, float x, float z, float *out, char *error, size_t error_size)
{
	int
		grid_x1,
		grid_x2,
		grid_z1,
		grid_z2,
		width = map->facts.fast_width,
		height = map->facts.fast_height;

	float
		x_offset,
		z_offset,
		height_x1z1,
		height_x2z1,
		height_x1z2,
		height_x2z2,
		height_zx1,
		height_zx2,
		result,
		x_grid_size = map->facts.fast_x_grid,
		z_grid_size = map->facts.fast_z_grid,
		x_grid_size_reciprocal,
		z_grid_size_reciprocal;

	fenv_t saved;

	if (!map->grid)
	{
		return fail (error, error_size, "terrain: this map has no fast elevation grid");
	}

	if (!isfinite (x) || !isfinite (z))
	{
		return fail (error, error_size, "terrain fast elevation: the position is not finite");
	}

	fegetenv (&saved);
	fesetround (FE_TOWARDZERO);

	/* terrsel.c load_terrain_simple_elevation: the reciprocals are derived at load, under the engine's rounding */
	x_grid_size_reciprocal = 1.0f / x_grid_size;
	z_grid_size_reciprocal = 1.0f / z_grid_size;

	if (!isfinite (x * x_grid_size_reciprocal) || !isfinite (z * z_grid_size_reciprocal) || fabsf (x * x_grid_size_reciprocal) >= 2147483520.0f
		|| fabsf (z * z_grid_size_reciprocal) >= 2147483520.0f)
	{
		fesetenv (&saved);

		return fail (error, error_size, "terrain fast elevation: the position is outside any grid cell index");
	}

#define GRID_AT(Z, X) ( { float v_; memcpy ( &v_, map->grid + ( ( size_t ) ( Z ) * ( size_t ) width + ( size_t ) ( X ) ) * 4, 4 ); v_; } )

	grid_x1 = ( int ) ( x * x_grid_size_reciprocal );
	grid_z1 = ( int ) ( z * z_grid_size_reciprocal );

	x_offset = x - ( grid_x1 * x_grid_size );
	z_offset = z - ( grid_z1 * z_grid_size );

	x_offset *= x_grid_size_reciprocal;
	z_offset *= z_grid_size_reciprocal;

	grid_x2 = grid_x1 + 1;
	grid_z2 = grid_z1 + 1;

	grid_x1 = bound ( grid_x1, 0, ( width - 1 ) );
	grid_x2 = bound ( grid_x2, 0, ( width - 1 ) );
	grid_z1 = bound ( grid_z1, 0, ( height - 1 ) );
	grid_z2 = bound ( grid_z2, 0, ( height - 1 ) );

	height_x1z1 = GRID_AT ( grid_z1, grid_x1 );
	height_x2z1 = GRID_AT ( grid_z1, grid_x2 );
	height_x1z2 = GRID_AT ( grid_z2, grid_x1 );
	height_x2z2 = GRID_AT ( grid_z2, grid_x2 );

#undef GRID_AT

	height_zx1 = ( ( height_x1z1 * ( 1 - z_offset ) ) + ( height_x1z2 * z_offset ) );
	height_zx2 = ( ( height_x2z1 * ( 1 - z_offset ) ) + ( height_x2z2 * z_offset ) );

	result = ( ( height_zx1 * ( 1 - x_offset ) ) + ( height_zx2 * x_offset ) );

	fesetenv (&saved);

	*out = result;

	return 0;
}

/* ======================================================================================================================== */
/* CONFORMANCE ONLY: the characterisation probe of eech_terrain_fixture.c, driven against this kernel                      */
/* ======================================================================================================================== */

static uint32_t sweep_next_random (uint32_t *state)
{
	uint32_t x = *state;
	x ^= x << 13;
	x ^= x >> 17;
	x ^= x << 5;
	*state = x;
	return x;
}

static float sweep_unit (uint32_t *state)
{
	return (float) (sweep_next_random (state) & 0xffffffu) / 16777216.0f;
}

static float sweep_inside (float value, float low, float high)
{
	return value < low ? low : (value > high ? high : value);
}

static uint32_t sweep_bits (float value)
{
	uint32_t bits;
	memcpy (&bits, &value, sizeof (bits));
	return bits;
}

static int sweep_same (const host_terrain_answer *a, const host_terrain_answer *b)
{
	return sweep_bits (a->elevation) == sweep_bits (b->elevation) && sweep_bits (a->last_elevation) == sweep_bits (b->last_elevation)
		&& sweep_bits (a->normal[0]) == sweep_bits (b->normal[0]) && sweep_bits (a->normal[1]) == sweep_bits (b->normal[1])
		&& sweep_bits (a->normal[2]) == sweep_bits (b->normal[2]) && a->type == b->type;
}

static void sweep_fold (uint64_t *hash, const host_terrain_answer *a)
{
	uint32_t words[7];
	int w, byte;

	words[0] = sweep_bits (a->elevation);
	words[1] = sweep_bits (a->last_elevation);
	words[2] = sweep_bits (a->normal[0]);
	words[3] = sweep_bits (a->normal[1]);
	words[4] = sweep_bits (a->normal[2]);
	words[5] = (uint32_t) a->type;
	words[6] = (uint32_t) a->fast_hit;

	for (w = 0; w < 7; w++)
	{
		for (byte = 0; byte < 4; byte++)
		{
			*hash ^= (uint64_t) ((words[w] >> (8 * byte)) & 0xffu);
			*hash *= 1099511628211ull;
		}
	}
}

/* a record held by value: the hint a query left (valid or cleared) */
struct sweep_record
{
	host_terrain_hint hint;
	int valid;
};

static void sweep_fold_fast (uint64_t *hash, float x, float z, float fast)
{
	uint32_t words[3];
	int w, byte;

	words[0] = sweep_bits (x);
	words[1] = sweep_bits (z);
	words[2] = sweep_bits (fast);

	for (w = 0; w < 3; w++)
	{
		for (byte = 0; byte < 4; byte++)
		{
			*hash ^= (uint64_t) ((words[w] >> (8 * byte)) & 0xffu);
			*hash *= 1099511628211ull;
		}
	}
}

static int sweep_ask (const host_terrain *map, float x, float z, struct sweep_record *record, host_terrain_answer *answer, char *error, size_t error_size)
{
	if (host_terrain_query (map, x, z, record->valid ? &record->hint : NULL, answer, error, error_size))
	{
		return -1;
	}

	record->valid = answer->has_hint;
	record->hint = answer->hint;

	return 0;
}

int host_terrain_conformance_sweep (const host_terrain *map, uint32_t seed, int count, int emit, host_terrain_sweep *out, host_terrain_sweep_sample *samples,
	char *error, size_t error_size)
{
	static const float primer_offsets[5] = { 0.05f, 0.5f, 5.0f, 50.0f, 500.0f };
	struct sweep_record shared;
	uint32_t state = seed ? seed : 1u;
	const float margin = 0.01f;
	const float side = (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
	fenv_t saved;
	float min_x, max_x, min_z, max_z;
	int total, i, rc = 0;

	/* the original probe's bounds (eech_engine_terrain_probe): a count of 0 to 200000, a non-negative emit that is capped at 256; checked before any arithmetic on them */
	if (count < 0 || count > 200000 || emit < 0 || !out)
	{
		return fail (error, error_size, "terrain sweep: count %d or emit %d is out of range", count, emit);
	}

	total = 6 + count;

	if (emit > 256)
	{
		emit = 256;
	}

	if (emit > total)
	{
		emit = total;
	}

	if (emit && !samples)
	{
		return fail (error, error_size, "terrain sweep: no sample buffer");
	}

	memset (out, 0, sizeof (*out));
	memset (&shared, 0, sizeof (shared));
	out->sample_hash = 14695981039346656037ull;
	out->primed_hash = 14695981039346656037ull;
	out->fast_hash = 14695981039346656037ull;
	out->fast_available = map->grid != NULL;

	fegetenv (&saved);
	fesetround (FE_TOWARDZERO);

	min_x = map->facts.min_x + margin;
	max_x = map->facts.max_x - margin;
	min_z = map->facts.min_z + margin;
	max_z = map->facts.max_z - margin;

	for (i = 0; i < total && !rc; i++)
	{
		float x, z, primer_x, primer_z, distance;
		host_terrain_answer fresh, repeat, shared_primed, private_primed, scratch, wrapper;
		struct sweep_record record, primer_record;
		int sign_x, sign_z;

		if (i < 4)
		{
			x = (i & 1) ? max_x : min_x;
			z = (i & 2) ? max_z : min_z;
		}
		else if (i == 4)
		{
			x = 0.5f * (min_x + max_x);
			z = 0.5f * (min_z + max_z);
		}
		else if (i == 5)
		{
			x = side * 4.0f + 0.001f;
			z = side * 4.0f - 0.001f;
		}
		else
		{
			x = min_x + sweep_unit (&state) * (max_x - min_x);
			z = min_z + sweep_unit (&state) * (max_z - min_z);

			if ((sweep_next_random (&state) & 7u) == 0)
			{
				float base_x = (float) ((int) (x / side)) * side + ((sweep_next_random (&state) & 1u) ? 0.5f * side : 0.0f);
				float base_z = (float) ((int) (z / side)) * side + ((sweep_next_random (&state) & 1u) ? 0.5f * side : 0.0f);

				x = base_x + ((int) (sweep_next_random (&state) % 3u) - 1) * 0.001f;
				z = base_z + ((int) (sweep_next_random (&state) % 3u) - 1) * 0.001f;
				out->snapped++;
			}
		}

		x = sweep_inside (x, min_x, max_x);
		z = sweep_inside (z, min_z, max_z);

		distance = primer_offsets[sweep_next_random (&state) % 5];
		sign_x = (sweep_next_random (&state) & 1u) ? 1 : -1;
		sign_z = (sweep_next_random (&state) & 1u) ? 1 : -1;
		primer_x = x + sign_x * distance;
		primer_z = z + sign_z * distance * 0.5f;

		if (primer_x < min_x || primer_x > max_x || primer_z < min_z || primer_z > max_z)
		{
			out->primer_skipped++;
			primer_x = x;
			primer_z = z;
		}

		/* reference: a cleared record holds no hint */
		memset (&record, 0, sizeof (record));
		rc = sweep_ask (map, x, z, &record, &fresh, error, error_size);

		if (rc)
		{
			break;
		}

		sweep_fold (&out->sample_hash, &fresh);

		if (sweep_bits (fresh.elevation) != sweep_bits (fresh.last_elevation))
		{
			out->result_ne_last++;
		}

		/* the same record straight after */
		rc = sweep_ask (map, x, z, &record, &repeat, error, error_size);

		if (!rc && !sweep_same (&fresh, &repeat))
		{
			out->repeat++;
		}

		/* a private record primed elsewhere */
		memset (&primer_record, 0, sizeof (primer_record));
		rc = rc || sweep_ask (map, primer_x, primer_z, &primer_record, &scratch, error, error_size);
		rc = rc || sweep_ask (map, x, z, &primer_record, &private_primed, error, error_size);

		if (!rc)
		{
			out->private_primed_fast += private_primed.fast_hit;
			out->private_primed += !sweep_same (&fresh, &private_primed);
			sweep_fold (&out->primed_hash, &private_primed);
		}

		/* the shared record primed elsewhere, then the wrappers the campaign calls, all on the shared record */
		rc = rc || sweep_ask (map, primer_x, primer_z, &shared, &scratch, error, error_size);
		rc = rc || sweep_ask (map, x, z, &shared, &shared_primed, error, error_size);

		if (!rc)
		{
			out->shared_primed_fast += shared_primed.fast_hit;
			out->shared_primed += !sweep_same (&fresh, &shared_primed);
			sweep_fold (&out->primed_hash, &shared_primed);
		}

		rc = rc || sweep_ask (map, x, z, &shared, &wrapper, error, error_size);

		if (!rc)
		{
			out->elevation_macro += sweep_bits (wrapper.elevation) != sweep_bits (fresh.elevation);
		}

		rc = rc || sweep_ask (map, x, z, &shared, &wrapper, error, error_size);

		if (!rc)
		{
			out->elevation_and_normal += sweep_bits (wrapper.elevation) != sweep_bits (fresh.elevation) || sweep_bits (wrapper.normal[0]) != sweep_bits (fresh.normal[0])
				|| sweep_bits (wrapper.normal[1]) != sweep_bits (fresh.normal[1]) || sweep_bits (wrapper.normal[2]) != sweep_bits (fresh.normal[2]);
		}

		rc = rc || sweep_ask (map, x, z, &shared, &wrapper, error, error_size);

		if (!rc)
		{
			out->face_normal += sweep_bits (wrapper.normal[0]) != sweep_bits (fresh.normal[0]) || sweep_bits (wrapper.normal[1]) != sweep_bits (fresh.normal[1])
				|| sweep_bits (wrapper.normal[2]) != sweep_bits (fresh.normal[2]);
		}

		if (!rc && i < emit)
		{
			samples[i].x = x;
			samples[i].z = z;
			samples[i].result = fresh.elevation;
			samples[i].last = fresh.last_elevation;
			samples[i].nx = fresh.normal[0];
			samples[i].ny = fresh.normal[1];
			samples[i].nz = fresh.normal[2];
			samples[i].type = fresh.type;
		}

		/* the fast grid at the same point, as the original fixture samples it (a read of the grid only, so its place in the order is immaterial) */
		if (!rc && map->grid)
		{
			float fast = 0.0f;

			rc = host_terrain_fast_elevation (map, x, z, &fast, error, error_size);

			if (!rc)
			{
				sweep_fold_fast (&out->fast_hash, x, z, fast);
				out->fast_count++;

				if (i < emit)
				{
					samples[i].fast = fast;
				}
			}
		}
	}

	fesetenv (&saved);

	return rc ? -1 : 0;
}
