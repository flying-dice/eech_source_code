/*
 * Host-owned physical terrain surface (EEDCS-47, D3).  The host owns the physical terrain: the elevation at a point, the face normal and the surface
 * type there, the fast gridpost elevation, and the facts about the map (extents, heights).  Campaign code keeps what it does with them: terrain class and
 * suitability, geography, route planning, task generation and every consequence.  The module-level queries of 3d/terrain (get_3d_terrain_point_data,
 * get_3d_terrain_elevation, get_3d_terrain_fast_elevation and the record getters) are implemented in mb_hostterrain.c as requests to the host; the
 * original solver (3d/terrain/terrelev.c, terrsel.c) is not compiled into the headless campaign module.
 *
 * The exchange is one synchronous host callback with five terrain operations (MAP, POINT, FAST, SECTOR, AMBIENT) and three ground operations (GROUND_ABOVE, GROUND_BELOW, GROUND_IMPACT, further down), each a single request and a single reply:
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
 *   AMBIENT the physical AMPLITUDE of the session's six terrain ambient sound children (EEDCS-51, D3; ss_updt.c update_session_sound_effects): sea, jungle (forest top), amb1 (river), amb2
 *          (reservoir), amb3 / amb4 (altered land 1 / 2).  The request is the position (x, z: the sound routine's camera clamped to the map, the same the SECTOR question is asked at), the campaign's
 *          observations (the radar altitude, the previous frame's delta, whether the cockpit is the point) and, for each such child in the session's child order, its ambient class and its
 *          PREVIOUS amplification (campaign child state: the smoothing compounds, the stored value is the scaled one).  The reply is the new amplification of each child, by value, in the same
 *          order, computed by the host from ITS sector surfaces with the original's arithmetic (previous plus or minus half the delta by whether the class's surface is in the sector, bound to
 *          0 to 0.3, times the altitude factor, times 0.75 in the cockpit, and 0 at or above 280 m).  The campaign keeps its child identities and lifecycle and writes the amplitudes; it
 *          computes none.  This is separate from SECTOR, which still fills the legacy scratch mirror for the renderer and the observer.
 *
 * Three more operations carry the ground-contact questions of the weapon flight, the avionics and the particles (EEDCS-48, D3 G2): whether a position is above the
 * ground (en_world.h point_above_ground), whether it is below it (point_below_ground) and where the segment between an above and a below position meets it
 * (en_cllsn.c get_ground_impact_point).  They are PHYSICAL questions, so the host answers them from its own terrain, with the original's expressions, order and
 * short circuits; the DLL has no predicate, sector maximum or bisection of its own behind them.  What stays the campaign's: the world-map area the below-ground test
 * compares against (campaign geography: `world`, supplied by value with every ground request and never assumed equal to the terrain's extents), the shared point
 * record and what is written into it, the aliasing of the caller's buffers, and the terrain type -> class mapping.
 *
 *   GROUND_ABOVE, GROUND_BELOW  the position (x, y, z), the world area and the shared record's hint.  The reply is the predicate (`answer`) and `queried`: whether the answer
 *          read the terrain.  When it did, the reply carries the complete record the last read left (last_elevation, normal, type, `stored`, hint) and the campaign
 *          stores all of it in the shared record; when it did not (the sector maximum or the world area decided), the record in the reply is not looked at and the
 *          campaign's record is left exactly as it is.
 *   GROUND_IMPACT  the above endpoint (x, y, z), the below endpoint (`below`), the world area and the shared record's hint.  The reply is the above and below endpoints after
 *          the original's correction, the impact point, and the complete record the final read left; the surface type of that record is what the campaign maps to a class.
 *
 * Two more operations carry the physical TREE segment query (EEDCS-49, D3; en_cllsn.c get_line_of_sight_collision_tree).  The host owns the tree data (its own tree_pos.sec and
 * tree_pos.dat), the forest tree's collision triangles, the sector scan, the early exits, the two tie rules and the segment/triangle and segment/quad arithmetic; the DLL keeps no
 * tree record, scan or intersection behind the query.  What stays the campaign's: the caller's buffers (the initial intercept point and normal words the caller passed, and what it
 * is left holding), the debug visualisation line, and the TERRAIN ELEVATION the host asks for, which the campaign answers from the same host terrain through the same call, in the same
 * order, and with the same shared-record side effect as the original (get_3d_terrain_elevation).  The exchange is a short dialogue on this callback, with at most one query open:
 *
 *   TREE_BEGIN    the old and new position, the caller's initial intercept point and face normal, and the two flags.  Every float travels as its 32-bit PATTERN (`tree_*` words):
 *                 the caller's own buffers may hold anything (signed zero, a NaN), and a miss leaves them as the original left them, so no value is ever converted.  The reply is
 *                 DONE (hit, the final point and normal words) or NEED_TERRAIN (which end, x and z).
 *   TREE_TERRAIN  the campaign's answer to NEED_TERRAIN: the elevation pattern.  The reply is again DONE or NEED_TERRAIN.
 *
 * A host asks for none, the old end only, the new end only (a segment that enters the map) or the old end and then the new end, each end at most once and only when it is inside the map.  A reply is checked completely before the caller's buffers are written, and a
 * missing provider, a refusal, a malformed reply or a third lookup stops the engine at the tree boundary: the campaign has NO tree solver to fall back to.  Equivalent absence (a map with no
 * tree files, or no forest triangles) is the host's to state and is an answer, not a refusal.
 *
 * A ground reply is checked completely before any caller buffer or the shared record is written: a malformed reply stops the engine at the ground boundary and leaves
 * both untouched.
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
#include <stdint.h>

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
	CAMPAIGN_TERRAIN_GROUND_ABOVE,
	CAMPAIGN_TERRAIN_GROUND_BELOW,
	CAMPAIGN_TERRAIN_GROUND_IMPACT,
	CAMPAIGN_TERRAIN_SECTOR,
	CAMPAIGN_TERRAIN_AMBIENT,
	CAMPAIGN_TERRAIN_TREE_BEGIN,
	CAMPAIGN_TERRAIN_TREE_TERRAIN
};

/* TREE reply states and ends */
enum campaign_terrain_tree_state
{
	CAMPAIGN_TERRAIN_TREE_DONE = 1,
	CAMPAIGN_TERRAIN_TREE_NEED_TERRAIN
};

enum campaign_terrain_tree_end
{
	CAMPAIGN_TERRAIN_TREE_END_OLD = 0,
	CAMPAIGN_TERRAIN_TREE_END_NEW
};

/* AMBIENT: the most children one question carries, and the classes of the six terrain ambient sound children */
#define CAMPAIGN_TERRAIN_AMBIENT_MAX 16

enum campaign_terrain_ambient_class
{
	CAMPAIGN_TERRAIN_AMBIENT_SEA = 0,
	CAMPAIGN_TERRAIN_AMBIENT_JUNGLE,
	CAMPAIGN_TERRAIN_AMBIENT_AMB1,
	CAMPAIGN_TERRAIN_AMBIENT_AMB2,
	CAMPAIGN_TERRAIN_AMBIENT_AMB3,
	CAMPAIGN_TERRAIN_AMBIENT_AMB4,
	CAMPAIGN_TERRAIN_AMBIENT_CLASSES
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
	/* AMBIENT (with x, z and winter): whether the cockpit is the point, the number of children (0 to CAMPAIGN_TERRAIN_AMBIENT_MAX), the point's radar altitude, the frame delta, and for each child its
	   class and previous amplification, in the session's child order */
	int ambient_in_cockpit, ambient_count;
	float ambient_alt, ambient_delta;
	int ambient_class[CAMPAIGN_TERRAIN_AMBIENT_MAX];
	float ambient_previous[CAMPAIGN_TERRAIN_AMBIENT_MAX];
	/* GROUND_ABOVE and GROUND_BELOW: the position is (x, y, z); GROUND_IMPACT: the above endpoint is (x, y, z) and `below` is the below endpoint */
	float y;
	float below[3];
	/* GROUND_*: the campaign's world-map area (min_x, min_z, max_x, max_z): campaign geography the host compares against, not the terrain's extents */
	float world[4];
	/* TREE_BEGIN: the old and new position and the caller's initial intercept point and face normal, each as the three 32-bit patterns x, y, z of its floats; the two flags (0 or 1) */
	uint32_t tree_old[3], tree_new[3], tree_point[3], tree_normal[3];
	int tree_ignore_forest_edge_trees, tree_skip_tree_top_test;
	/* TREE_TERRAIN: the elevation pattern the campaign read at the point it was asked about */
	uint32_t tree_elevation;
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
	/* AMBIENT: the new amplification of each child of the request, in its order */
	float ambient_amplitude[CAMPAIGN_TERRAIN_AMBIENT_MAX];
	/* GROUND_ABOVE and GROUND_BELOW: the predicate (0 or 1), and (0 or 1) whether the answer read the terrain; the record above (last_elevation, normal, type, stored, hint)
	   is meaningful only when it did */
	int answer, queried;
	/* GROUND_IMPACT: the endpoints after the original's correction and the impact point; the record above is the one the final read left */
	float above[3], below[3], impact[3];
	/* TREE_BEGIN and TREE_TERRAIN: the state (campaign_terrain_tree_state); NEED_TERRAIN: the end the elevation is wanted for and the patterns of x and z; DONE: whether a tree was hit and
	   the patterns of the final intercept point and face normal, as the original leaves them (the caller's own on a miss, or what a failed attempt partly overwrote) */
	int tree_state, tree_end, tree_hit;
	uint32_t tree_x, tree_z;
	uint32_t tree_result_point[3], tree_result_normal[3];
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

/* The AMBIENT operation (called by update_session_sound_effects in the headless campaign module): the host's new amplification of each of `count` terrain ambient sound children, by value.  `count` is
   the bound of ONE question (CAMPAIGN_TERRAIN_AMBIENT_MAX), a property of the transport and NOT of the session: the session may hold any number of such children, and the caller asks about them in as many
   bounded questions as its child list needs, in list order, dropping, clamping or deduplicating none.  The
   request is validated before it is sent (count within the limit, each class one of the six, nothing else); the answer is checked in full (finite amplitudes) BEFORE anything is written to
   `amplitude`.  A missing provider, a refusal, a malformed answer or a position the host cannot answer stops the engine with an error that names the terrain provider; nothing is solved from the
   campaign's own sectors. */
void campaign_terrain_ambient (float x, float z, int in_cockpit, float alt, float delta, int count, const int *classes, const float *previous, float *amplitude);

struct VEC3D;

/* The TREE operations (called by get_line_of_sight_collision_tree in the headless campaign module): the host decides whether the segment from `old_position` to `new_position` meets a forest tree and
   where, asking the campaign for the terrain elevation it needs, one at a time, from the campaign's own (host-backed) terrain.  `intercept_point` and `face_normal` are the caller's buffers:
   their initial words are sent and the host's final words are written back, only after the whole answer is checked.  The flags are the original's `ignore_forest_edge_trees` and
   `skip_tree_top_test`.  Returns the hit, as the original does.  A missing provider, a refusal, a malformed reply, a lookup beyond the two the original makes, or a query started inside another stops the
   engine with an error that names the terrain provider; nothing is solved from the campaign's own tree data. */
int campaign_terrain_tree_collision
(
	const struct VEC3D *old_position,
	const struct VEC3D *new_position,
	struct VEC3D *intercept_point,
	struct VEC3D *face_normal,
	int ignore_forest_edge_trees,
	int skip_tree_top_test
);

#endif
