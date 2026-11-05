/*
 * Plain-value input to the host's fixed-wing navigator (eech_fw_navigate in fw_step.c).
 *
 * Every field is a fact the legacy fixed_wing_navigator read through an entity
 * pointer, resolved by the caller before the call: no entity pointers and no
 * campaign state cross this boundary.  Vectors are x, y, z in world space;
 * `attitude` is the 3x3 attitude matrix in row-major order, the layout eech_fw_step
 * takes.  Where the engine read one stored value through two accessors
 * (FLOAT_TYPE_VELOCITY and raw->ac.mob.velocity) there is one field, `velocity`.
 *
 * The result is the steering acceleration the legacy navigator left in `acc`.
 */
#ifndef FW_NAVIGATOR_H
#define FW_NAVIGATOR_H

/* another member of this aircraft's group, as fixed_wing_collision_avoidance saw it */
typedef struct eech_fw_nav_other
{
	float position[3];
	float motion_vector[3];
	float avoidance_radius; /* aircraft_database[sub_type].avoidance_radius */
} eech_fw_nav_other;

typedef struct eech_fw_nav_input
{
	/* INT_TYPE_GROUP_LEADER and INT_TYPE_TASK_LEADER: different flags, both read */
	int group_leader;
	int task_leader;

	float delta_time; /* get_entity_movement_delta_time() */

	/* this aircraft */
	float position[3];
	float motion_vector[3]; /* raw->ac.mob.motion_vector, used by collision avoidance */
	float attitude[9];      /* raw->ac.mob.attitude */
	float velocity;         /* raw->ac.mob.velocity == FLOAT_TYPE_VELOCITY */
	float navigation_velocity[3]; /* the `vel` argument the legacy navigator was given */
	float cruise_velocity;  /* FLOAT_TYPE_CRUISE_VELOCITY */
	float max_vertical_turn_allowance; /* FLOAT_TYPE_MAX_VERTICAL_TURN_ALLOWANCE */
	float g_max;            /* aircraft_database[sub_type].g_max */
	float avoidance_radius; /* aircraft_database[sub_type].avoidance_radius */

	/* resolved by the caller: fixed_wing_movement_get_waypoint_position and the guide */
	float waypoint[3];
	float guide_velocity;              /* FLOAT_TYPE_VELOCITY of the guide (0 means use cruise) */
	float task_leader_motion_vector[3]; /* the guide's task leader; read when !task_leader */

	/* the group's other members (this aircraft excluded) in the group list's order */
	int other_count;
	const eech_fw_nav_other *others;
} eech_fw_nav_input;

void eech_fw_navigate(const eech_fw_nav_input *input, float *acceleration);

/*
 * Facts for the physical turn-around decision of an airborne fixed-wing step
 * (basic_fixed_wing_movement, the facing-away and cannon-aim part).  The caller
 * supplies observations and the campaign's gun-aim intent; whether the aircraft
 * is facing away, and so whether the absolute turn-around flight applies, is
 * decided here from the retained pose and motion vector.
 */
typedef struct eech_fw_turn_input
{
	int task_leader;     /* INT_TYPE_TASK_LEADER */
	int aiming_cannon;   /* campaign policy: a cannon engagement is in range and being aimed */
	float elapsed_time;  /* FLOAT_TYPE_ELAPSED_TIME */
	float position[3];
	float motion_vector[3];  /* raw->ac.mob.motion_vector, the previous step's */
	float waypoint[3];
	float leader_position[3]; /* the task leader's position; read only when facing away and !task_leader */
} eech_fw_turn_input;

/* 1 when the waypoint lies behind the retained motion vector (the legacy dot-product test) */
int eech_fw_facing_away(const float *position, const float *motion_vector, const float *waypoint);

/*
 * Update the retained turn multiplier and hold-until time and say whether this
 * step is the absolute turn-around flight (hold-until still ahead of elapsed_time).
 * Both retained values persist across steps in the caller.
 */
int eech_fw_turn(const eech_fw_turn_input *input, float *turn_speed_multi, float *fixed_wing_toggle);

/*
 * The terrain-follow look-ahead of mobile_terrain_adjust_waypoint_height: the x and z of the point in front of the
 * aircraft (its speed plus ten metres along the attitude's forward axis) at which the campaign's terrain service is
 * sampled.  Computed from the host's retained pose, never taken from the campaign.
 */
void eech_fw_lookahead(const float *position, float velocity, const float *attitude, float *point_xz);

/*
 * The rest of mobile_terrain_adjust_waypoint_height for a fixed-wing aircraft, given the terrain elevation the
 * campaign's terrain service answered at the look-ahead point: the waypoint height that climbs (or descends) the
 * aircraft to `min_height` above the terrain.  `guide_position` is the guide's position as the campaign selected it
 * (the horizontal range is measured to it and its height is what stays when the original left the height alone).
 * Returns the height.
 */
float eech_fw_terrain_height(
	const float *position,
	float velocity,
	const float *attitude,
	const float *guide_position,
	float terrain_elevation,
	float min_height);

#endif
