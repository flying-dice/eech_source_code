/*
 * Host-owned physical terrain surface (EEDCS-47, D3).  The host owns the physical terrain: the elevation at a point, the face normal and the surface
 * type there, the fast gridpost elevation, and the facts about the map (extents, heights).  Campaign code keeps what it does with them: terrain class and
 * suitability, geography, route planning, task generation and every consequence.  The module-level queries of 3d/terrain (get_3d_terrain_point_data,
 * get_3d_terrain_elevation, get_3d_terrain_fast_elevation and the record getters) are implemented in mb_hostterrain.c as requests to the host; the
 * original solver (3d/terrain/terrelev.c, terrsel.c) is not compiled into the headless campaign module.
 *
 * The exchange is one synchronous host callback with four operations, each a single request and a single reply:
 *
 *   Every request carries the season the terrain was loaded in (`winter`), captured before the first question: the host serves the map of that season.
 *
 *   MAP    the map facts the core reads once at boot (campaign_terrain_install_map_facts): the sector counts, the extents, the lowest and highest
 *          heights and the scaled height difference, and the size of the fast grid (0 when the host has none).
 *   POINT  the query of get_3d_terrain_point_data.  The request is the position and the record's hint; the reply is what the query returns and
 *          what it stores in the record: the elevation it returns, the elevation it stores (`last_elevation`: it differs from the returned one only
 *          where the host says so, e.g. the original's Cuba flaw area), the face normal, the surface type (an EECH terrain_types id) and the
 *          record's next hint, and (`stored`, by value) whether the record holds a surface afterwards: the legacy "a face is stored" test of the fixed-wing movement
 *          asks that, never the hint, so a host that returns no hint still answers it.
 *   FAST   the fast gridpost elevation at a position.
 *   SECTOR the surface enumeration of the sector that holds a position (EEDCS-51, D3): get_terrain_3d_types_in_sector, which the session sound routine calls every update.  The request is
 *          the position (x, z; the sound routine passes the camera clamped to the map).  The reply is `sector_returned`, the original's return value (1 PLUS THE NUMBER OF SURFACE
 *          TRANSITIONS of the sector, not the number of distinct surfaces), and `sector_flags`, the COMPLETE scratch by value: one character per terrain type, '1' where the type is a surface
 *          of the sector (its first surface and the surface of every polygon that changes surface), '0' elsewhere.  The campaign keeps the legacy global scratch
 *          terrain_types_in_sector as a by-value MIRROR of the last reply, written only after the reply has been checked, so its readers (the sound routine, the renderer) are unchanged;
 *          it computes nothing for this operation and has no fallback to its own loaded sectors.  The sector choice, its domain (a position whose sector is not in the map is refused,
 *          where the original indexed outside its table) and the reset of the scratch are the host's.  The other sector readers (the per-type triangle counts and lists of the guide
 *          cover code) are not part of this operation and remain the campaign's own.
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

/* the room of a SECTOR reply's flags: at least TERRAIN_TYPE_LAST (62) characters and a terminator; the reply carries exactly TERRAIN_TYPE_LAST of them */
#define CAMPAIGN_TERRAIN_FLAGS_MAX 64

enum campaign_terrain_op
{
	CAMPAIGN_TERRAIN_MAP = 1,
	CAMPAIGN_TERRAIN_POINT,
	CAMPAIGN_TERRAIN_FAST,
	CAMPAIGN_TERRAIN_SECTOR
};

typedef struct campaign_terrain_request
{
	int op;
	/* every operation: the season the campaign was loaded in, as the original chose its terrain files (terrdata.c: winter uses default-W.sec and default-W.rgb when both exist, else the default pair); the host answers for the map of that season */
	int winter;
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
	/* POINT: nonzero when the record holds a surface after this reply (the legacy "a face is stored"); independent of the hint, which a host may never return */
	int stored;
	char hint[CAMPAIGN_TERRAIN_HINT_MAX + 1];
	/* SECTOR: the original's return value (1 + the surface transitions) and the complete scratch, one '0' or '1' per terrain type, NUL-terminated after TERRAIN_TYPE_LAST characters */
	int sector_returned;
	char sector_flags[CAMPAIGN_TERRAIN_FLAGS_MAX];
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

/* The SECTOR operation, called by get_terrain_3d_types_in_sector (modules/3d/terrain/terrdata.c) in the headless campaign module: the host's answer is checked in full, then written into
   terrain_types_in_sector by value, and the original's return value is returned.  A missing provider, a refusal, a malformed answer or a position the host cannot answer stops the engine
   with an error that names the terrain provider; nothing is solved from the campaign's own sectors. */
int campaign_terrain_types_in_sector (float x, float z);

#endif
