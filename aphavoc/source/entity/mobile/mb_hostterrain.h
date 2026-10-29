/*
 * Host-owned physical terrain surface (EEDCS-47, D3).  The host owns the physical terrain: the elevation at a point, the face normal and the surface
 * type there, the fast gridpost elevation, and the facts about the map (extents, heights).  Campaign code keeps what it does with them: terrain class and
 * suitability, geography, route planning, task generation and every consequence.  The module-level queries of 3d/terrain (get_3d_terrain_point_data,
 * get_3d_terrain_elevation, get_3d_terrain_fast_elevation and the record getters) are implemented in mb_hostterrain.c as requests to the host; the
 * original solver (3d/terrain/terrelev.c, terrsel.c) is not compiled into the headless campaign module.
 *
 * The exchange is one synchronous host callback with three operations, each a single request and a single reply:
 *
 *   MAP    the map facts the core reads once at boot (campaign_terrain_install_map_facts): the sector counts, the extents, the lowest and highest
 *          heights and the scaled height difference, and the size of the fast grid (0 when the host has none).
 *   POINT  the query of get_3d_terrain_point_data.  The request is the position and the record's hint; the reply is what the query returns and
 *          what it stores in the record: the elevation it returns, the elevation it stores (`last_elevation`: it differs from the returned one only
 *          where the host says so, e.g. the original's Cuba flaw area), the face normal, the surface type (an EECH terrain_types id) and the
 *          record's next hint.
 *   FAST   the fast gridpost elevation at a position.
 *
 * The HINT is the host's opaque state for one find-fast record (the original kept pointers into its own terrain geometry there).  The campaign stores
 * it in the record and returns it with the next query on that record, exactly as the records are used (a shared record for queries that pass no record
 * of their own, private ones in entities and locals); it never parses, builds or compares one, and it is a bounded string, never a pointer.  A host that
 * keeps no such state returns no hint (empty) and ignores the one it is given.
 *
 * There is no campaign fallback: a hosted engine with no terrain provider stops with an error at boot, and a malformed or failed answer stops it with an
 * error that names the terrain provider.  No EECH pointer, entity index or terrain layout crosses the boundary.
 */
#ifndef MB_HOSTTERRAIN_H
#define MB_HOSTTERRAIN_H

#include <stddef.h>

#define CAMPAIGN_TERRAIN_ERROR_MAX 512

#ifndef CAMPAIGN_TERRAIN_HINT_MAX
#define CAMPAIGN_TERRAIN_HINT_MAX 127
#endif

enum campaign_terrain_op
{
	CAMPAIGN_TERRAIN_MAP = 1,
	CAMPAIGN_TERRAIN_POINT,
	CAMPAIGN_TERRAIN_FAST
};

typedef struct campaign_terrain_request
{
	int op;
	/* POINT and FAST */
	float x, z;
	/* POINT: the record's hint; empty when it has none */
	char hint[CAMPAIGN_TERRAIN_HINT_MAX + 1];
} campaign_terrain_request;

typedef struct campaign_terrain_reply
{
	/* MAP */
	int sector_x_max, sector_z_max;
	int fast_width, fast_height;
	float min_x, min_z, max_x, max_z;
	float min_height, max_height, scaled_height_difference;
	/* POINT and FAST: the elevation the query returns */
	float elevation;
	/* POINT */
	float last_elevation;
	float normal[3];
	int type;
	char hint[CAMPAIGN_TERRAIN_HINT_MAX + 1];
} campaign_terrain_reply;

typedef int (*campaign_terrain_fn)
(
	const campaign_terrain_request *request,
	campaign_terrain_reply *reply,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_terrain_host
{
	campaign_terrain_fn terrain;
	void *user;
} campaign_terrain_host;

/* Installs the host for the rest of the process: unlike the dialogue hosts it is kept across engine entries, since the terrain is asked about from every
   entry (a frame, a probe, a trace) and its provider is fixed at boot.  A NULL host or one with no callback leaves the installed one. */
void campaign_terrain_set_host (const campaign_terrain_host *host);

/* Asks the host for the map facts, checks them against the terrain the campaign loaded for the sector-level users that are still its own (the sector
   counts must agree) and takes the extents and heights from the host.  Called once, straight after the terrain is loaded and before anything reads it. */
void campaign_terrain_install_map_facts (void);

#endif
