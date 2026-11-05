/*
 * Plain-value input and results of the host's helicopter physical guidance (hc_step.c), the physical block of
 * helicopter_movement (helicop/hc_move.c): the LOS adjustment of the fly-to position, the altitude follow with the
 * structure avoidance and terrain look-ahead, the velocity shaping, the world acceleration and the lift, and the
 * heading, pitch and roll demands.
 *
 * Every field is a fact the legacy code read through an entity pointer, resolved by the caller before the call: no
 * entity pointers cross this boundary.  Vectors are x, y, z in world space; `attitude` is the 3x3 attitude matrix in
 * row-major order, the layout eech_hc_end takes.  The campaign still owns which waypoint, formation slot, hold and
 * target apply (`waypoint`, `wp_vel`, `distance`, the waypoint kind, the hold flag, the required heading, the weapon
 * vectors); the thresholds, limits, following, obstacle selection and arithmetic are here.
 *
 * Three calls, in the engine's phases:
 *   1. eech_hc_guidance_plan   (pre-begin) what the guidance has to look up: the structure scan and the terrain samples.
 *   2. eech_hc_guidance        (pre-begin) with those answers: the acceleration, the climb demand and the lift.  The
 *                              caller then runs eech_hc_rotor_alive, the human-wait gates and eech_hc_begin as before.
 *   3. eech_hc_demand          (post-begin) the heading, pitch and roll demands eech_hc_end takes.
 */
#ifndef HC_GUIDANCE_H
#define HC_GUIDANCE_H

/* the current waypoint's sub type, by the groups hc_move.c tells apart */
enum
{
	EECH_HC_WP_OTHER = 0,       /* any other sub type, or no current waypoint */
	EECH_HC_WP_ROUTE = 1,       /* NAVIGATION, CAP_LOOP, CAP_START */
	EECH_HC_WP_HOLDING = 2,     /* HOLDING, HOLDING_LOOP, TAXI */
	EECH_HC_WP_LANDED = 3,      /* LANDED */
	EECH_HC_WP_INSERTION = 4    /* PREPARE_FOR_INSERTION, TROOP_INSERT */
};

/* HELICOPTER_TERRAIN_AVOIDANCE_LOOKAHEAD_SAMPLE_NUMBER + 1 */
#define EECH_HC_TERRAIN_SAMPLES 2

/* a structure in the aircraft's sector, as helicopter_movement_structure_avoidance saw it (sector list order) */
typedef struct eech_hc_structure
{
	float position[3];       /* VEC3D_TYPE_POSITION */
	float avoidance_radius;  /* FLOAT_TYPE_AVOIDANCE_RADIUS */
	float height;            /* FLOAT_TYPE_FIXED_STRUCTURE_HEIGHT (read by the engine only when it is inside the radius) */
} eech_hc_structure;

/* another member of this aircraft's group (this aircraft excluded) in the group list's order */
typedef struct eech_hc_other
{
	float position[3];
} eech_hc_other;

typedef struct eech_hc_guidance_input
{
	/* campaign facts */
	int group_leader;          /* INT_TYPE_GROUP_LEADER: the leader does not avoid wingmen */
	int task_leader;           /* INT_TYPE_TASK_LEADER */
	int task_leader_is_self;   /* the guide's PTR_TYPE_TASK_LEADER is this aircraft */
	int position_hold;         /* INT_TYPE_POSITION_HOLD (the raw flag altitude follow reads) */
	int terrain_follow;        /* the guide's INT_TYPE_TERRAIN_FOLLOW_MODE != GUIDE_TERRAIN_FOLLOW_NONE */
	int terrain_follow_attack_altitude; /* ... == GUIDE_TERRAIN_FOLLOW_ATTACK_ALTITUDE */
	int waypoint_kind;         /* EECH_HC_WP_* of the current waypoint */
	int guide_radius_valid;    /* get_guide_criteria_valid (guide, GUIDE_CRITERIA_RADIUS) */
	float guide_radius;        /* get_guide_criteria_value (guide, GUIDE_CRITERIA_RADIUS), read when valid */
	int guide_altitude_valid;  /* get_guide_criteria_valid (guide, GUIDE_CRITERIA_ALTITUDE) */

	/* the waypoint, as helicopter_movement_get_waypoint_position_and_velocity resolved it */
	float waypoint[3];         /* wp_pos */
	float waypoint_velocity[3]; /* wp_vel (x and z are used) */
	float distance;            /* FLOAT_TYPE_DISTANCE it just stored (2d range to wp_pos) */

	/* this aircraft, pre-begin */
	float position[3];
	float motion_vector[3];    /* raw->ac.mob.motion_vector, the previous step's */
	float attitude[9];
	float velocity;            /* raw->ac.mob.velocity, the previous step's */
	float terrain_elevation;   /* get_3d_terrain_point_data_elevation (&raw->ac.terrain_info) before any sample */
	float fuel_supply_level;   /* raw->fuel_supply_level */
	float cruise_velocity;     /* aircraft_database[sub_type].cruise_velocity */
	float power_output;        /* aircraft_database[sub_type].power_output */
	float cruise_altitude;     /* FLOAT_TYPE_CRUISE_ALTITUDE */
	float attack_altitude;     /* FLOAT_TYPE_ATTACK_ALTITUDE */
	float leader_radar_altitude; /* the task leader's FLOAT_TYPE_RADAR_ALTITUDE (transitional terrain data, not host-owned); read for a follower in terrain follow */

	/* the map bounds the look-ahead positions are held inside (x and z are used) */
	float map_min[3];
	float map_max[3];

	float delta_time;          /* get_entity_movement_delta_time() */

	/* the group's other helicopters, in list order */
	int other_count;
	const eech_hc_other *others;
} eech_hc_guidance_input;

/*
 * What guidance needs looked up before it can finish.  The positions are computed from the physical state; the lookups
 * themselves (the sector's structures, the terrain samples) are the campaign's transitional services.
 */
typedef struct eech_hc_query_plan
{
	/*
	 * 1: pass the structures of the AIRCRAFT'S OWN sector (sector list order, uncapped) to eech_hc_guidance, as the
	 * original scans them.  The position is only the look-ahead point those structures are tested against (the engine's y
	 * is 0); it is not the sector to enumerate.  0: not asked for (no terrain follow, or the task leader is inside its
	 * radius with an altitude criterion); the structure list is ignored.
	 */
	int structure_scan;
	float structure_position[3];

	/*
	 * The terrain samples, far to near, to be taken in this order with the original statement
	 * terrain_elevation = max (terrain_elevation, get_3d_terrain_point_data (x, z, terrain_info)), the running value
	 * starting at 0.0.  Every getter call writes terrain_info; the last call is at the aircraft and leaves terrain_info
	 * there, which eech_hc_end's elevation is read from.  0 samples when terrain follow is off; then terrain_info is left
	 * as it was.
	 */
	int terrain_sample_count;
	float terrain_sample[EECH_HC_TERRAIN_SAMPLES][2]; /* x, z, already bound to the map area */
} eech_hc_query_plan;

typedef struct eech_hc_guidance_result
{
	float world_acceleration[3]; /* world_acceleration_vector: x and z go to eech_hc_begin, y is only in the lift */
	float wp_vec[3];             /* the velocity demand: y is the climb demand eech_hc_begin takes, x and z feed eech_hc_demand */
	float lift;                  /* goes to eech_hc_rotor_alive */
	float waypoint[3];           /* the fly-to position after the LOS adjustment (observational) */
	float required_height;       /* altitude follow's height above sea level (observational) */
} eech_hc_guidance_result;

void eech_hc_guidance_plan(const eech_hc_guidance_input *input, eech_hc_query_plan *plan);

/*
 * `structures`/`structure_count`: the sector's structures when the plan asked for the scan.
 * `sampled_terrain_elevation`: the original's running maximum after the plan's samples (terrain_elevation in
 * helicopter_movement_altitude_follow, which starts at 0.0).  It is the HOST's fold of the DLL's recorded getter returns
 * (see the host's HelicopterTerrainSample): `terrain_elevation = max (terrain_elevation, get_3d_terrain_point_data (...))`
 * calls the getter once when the running value exceeds the first return and otherwise a second time, selecting the SECOND
 * return.  Ignored when terrain follow is off.
 */
void eech_hc_guidance(const eech_hc_guidance_input *input, const eech_hc_structure *structures, int structure_count, float sampled_terrain_elevation, eech_hc_guidance_result *result);

typedef struct eech_hc_demand_input
{
	/* campaign facts */
	int waypoint_kind;         /* EECH_HC_WP_* of get_local_entity_current_waypoint */
	int task_leader_is_self;   /* the guide's PTR_TYPE_TASK_LEADER is this aircraft */
	int guide_radius_valid;
	float guide_radius;
	float distance;            /* FLOAT_TYPE_DISTANCE */
	int required_heading_valid; /* get_guide_required_heading's return, a pure read */
	float required_heading;    /* ... and the heading it gave */
	/*
	 * The task leader's attitude, the host's own record of it (its FLOAT_TYPE_HEADING is the heading of this attitude).
	 * Read only when the heading demand follows another aircraft; until then leader_attitude_valid is 0 and the demand
	 * reports EECH_HC_DEMAND_NEEDS_LEADER when it reaches that read, so the caller resolves the leader only then.
	 */
	int leader_attitude_valid;
	float leader_attitude[9];

	/* D2 weapon inputs: the campaign's targeting decision and the vectors it aims with */
	int targeting_active;      /* a target, and a weapon-vector criterion, launch delay or burst timer */
	float weapon_vector[3];    /* VEC3D_TYPE_WEAPON_VECTOR */
	float weapon_to_intercept_vector[3]; /* VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR */

	/* this aircraft, post-begin */
	float attitude[9];
	float motion_vector[3];    /* after eech_hc_begin */
	float velocity;            /* FLOAT_TYPE_VELOCITY eech_hc_begin stored */
	float cruise_velocity;     /* FLOAT_TYPE_CRUISE_VELOCITY */
	float g_max;               /* aircraft_database[sub_type].g_max */
	float main_rotor_shaft_angle; /* FLOAT_TYPE_MAIN_ROTOR_SHAFT_ANGLE */

	/* from eech_hc_guidance */
	float world_acceleration[3];
	float wp_vec[3];
} eech_hc_demand_input;

typedef struct eech_hc_demand_result
{
	float desired_heading;
	float desired_pitch;
	float desired_roll;
} eech_hc_demand_result;

#define EECH_HC_DEMAND_NEEDS_LEADER 1

/* 0: the demands are in `result`.  EECH_HC_DEMAND_NEEDS_LEADER: the heading follows the leader and leader_attitude_valid is 0; `result` is not valid, call again with the leader's attitude. */
int eech_hc_demand(const eech_hc_demand_input *input, eech_hc_demand_result *result);

#endif
