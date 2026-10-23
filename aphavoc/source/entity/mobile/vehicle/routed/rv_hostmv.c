#include "project.h"

#include <math.h>
#include <string.h>

#include "rv_hostmv.h"

static campaign_movement_host current_host;
static int applying_host_result;
static void apply_speed (entity *en, float speed);

void campaign_movement_set_host (const campaign_movement_host *host)
{
	if (host)
	{
		current_host = *host;
	}
	else
	{
		memset (&current_host, 0, sizeof (current_host));
	}
	applying_host_result = FALSE;
}

void campaign_movement_apply_begin (void)
{
	applying_host_result = TRUE;
}

void campaign_movement_apply_end (void)
{
	applying_host_result = FALSE;
}

int campaign_movement_is_hosted (entity *vehicle)
{
	return (current_host.move_vehicle != NULL) && (campaign_roster_host_id (vehicle) != NULL);
}

void campaign_movement_require_hosted (entity *vehicle)
{
	if (!campaign_movement_is_hosted (vehicle))
	{
		debug_fatal ("host movement: live movement of entity %d needs a host move_vehicle callback and a host identity", get_local_entity_index (vehicle));
	}
}

static const char *require_host_id (entity *vehicle)
{
	const char *host_id;

	if (!current_host.move_vehicle)
	{
		debug_fatal ("host movement: host move_vehicle callback is unavailable");
	}
	host_id = campaign_roster_host_id (vehicle);
	if (!host_id || !host_id[0] || (strlen (host_id) > CAMPAIGN_MOVEMENT_HOST_ID_MAX))
	{
		debug_fatal ("host movement: entity %d has no host identity", get_local_entity_index (vehicle));
	}
	return host_id;
}

// the mobile part of a hosted entity: an aircraft carries it inside its aircraft part, a vehicle directly

static const mobile *hosted_mobile (entity *en)
{
	if (get_local_entity_type (en) == ENTITY_TYPE_FIXED_WING)
	{
		return &((const fixed_wing *) get_local_entity_data (en))->ac.mob;
	}

	if (get_local_entity_type (en) == ENTITY_TYPE_HELICOPTER)
	{
		return &((const helicopter *) get_local_entity_data (en))->ac.mob;
	}

	return &((const vehicle *) get_local_entity_data (en))->mob;
}

static void fill_pose (campaign_movement_step *step, const mobile *mob)
{
	step->position[0] = mob->position.x;
	step->position[1] = mob->position.y;
	step->position[2] = mob->position.z;
	step->velocity = mob->velocity;
	memcpy (step->attitude, mob->attitude, sizeof (step->attitude));
}

static void fill_map_bounds (campaign_movement_step *step)
{
	step->map_min[0] = MIN_MAP_X;
	step->map_min[1] = MIN_MAP_Y;
	step->map_min[2] = MIN_MAP_Z;
	step->map_max[0] = MAX_MAP_X;
	step->map_max[1] = MAX_MAP_Y;
	step->map_max[2] = MAX_MAP_Z;
}

static void begin_step (campaign_movement_step *step, entity *en, int mode)
{
	const char *host_id = require_host_id (en);

	memset (step, 0, sizeof (*step));
	memcpy (step->host_id, host_id, strlen (host_id) + 1);
	step->mode = mode;
	fill_pose (step, hosted_mobile (en));
}

static int all_finite (const float *values, int count)
{
	int i;

	for (i = 0; i < count; i++)
	{
		if (!isfinite (values[i]))
		{
			return FALSE;
		}
	}
	return TRUE;
}

static campaign_movement_result ask_host (const campaign_movement_step *step)
{
	campaign_movement_result result;
	char error[CAMPAIGN_MOVEMENT_ERROR_MAX + 1];

	memset (error, 0, sizeof (error));
	memset (&result, 0, sizeof (result));
	result.rotated = -1;
	result.pending = -1;
	result.moved = -1;
	if (current_host.move_vehicle (step, &result, error, CAMPAIGN_MOVEMENT_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host movement: host move_vehicle failed for %s: %s", step->host_id, error[0] ? error : "unspecified error");
	}
	if (!all_finite (result.position, 3) || !all_finite (&result.velocity, 1) || !all_finite (result.attitude, 9) ||
		!all_finite (result.motion_vector, 3) || !all_finite (result.rotor, 3) || !all_finite (&result.altitude, 1) ||
		!all_finite (result.angles, 3) ||
		((result.rotated != FALSE) && (result.rotated != TRUE)) ||
		((result.pending != FALSE) && (result.pending != TRUE)) ||
		((result.moved != FALSE) && (result.moved != TRUE)) ||
		!all_finite (result.hc_query.structure_position, 3) || !all_finite (&result.hc_query.terrain_sample[0][0], 4) ||
		((result.hc_query.structure_scan != FALSE) && (result.hc_query.structure_scan != TRUE)) ||
		(result.hc_query.terrain_sample_count < 0) || (result.hc_query.terrain_sample_count > CAMPAIGN_HC_TERRAIN_SAMPLES) ||
		!all_finite (&result.hc_guidance.lift, 1) || !all_finite (result.hc_guidance.waypoint, 3) ||
		!all_finite (&result.hc_guidance.required_height, 1) || !all_finite (result.hc_guidance.world_acceleration, 3) ||
		!all_finite (result.hc_guidance.wp_vec, 3))
	{
		debug_fatal ("host movement: host move_vehicle returned an invalid result for %s", step->host_id);
	}
	return result;
}

void campaign_movement_place (entity *vehicle)
{
	campaign_movement_step step;
	entity_types type;

	if (applying_host_result || !current_host.move_vehicle)
	{
		return;
	}
	type = get_local_entity_type (vehicle);
	if ((type != ENTITY_TYPE_ROUTED_VEHICLE) && (type != ENTITY_TYPE_ANTI_AIRCRAFT) && (type != ENTITY_TYPE_SHIP_VEHICLE) && (type != ENTITY_TYPE_PERSON) && (type != ENTITY_TYPE_FIXED_WING) && (type != ENTITY_TYPE_HELICOPTER))
	{
		return;
	}
	if (!campaign_roster_host_id (vehicle))
	{
		return;
	}
	begin_step (&step, vehicle, CAMPAIGN_MOVEMENT_PLACE);
	if (type == ENTITY_TYPE_FIXED_WING)
	{
		/* the campaign's own write of an aircraft's motion vector is an explicit placement of that physical state */
		const mobile *mob = hosted_mobile (vehicle);

		step.fw_motion_vector[0] = mob->motion_vector.x;
		step.fw_motion_vector[1] = mob->motion_vector.y;
		step.fw_motion_vector[2] = mob->motion_vector.z;
	}
	if (type == ENTITY_TYPE_HELICOPTER)
	{
		/* likewise a helicopter's: any campaign write of its pose or vector is a placement the host adopts */
		const mobile *mob = hosted_mobile (vehicle);

		step.fw_motion_vector[0] = mob->motion_vector.x;
		step.fw_motion_vector[1] = mob->motion_vector.y;
		step.fw_motion_vector[2] = mob->motion_vector.z;
	}
	ask_host (&step);
}

int campaign_movement_create_place (entity *mobile, const vec3d *position, float cg_to_ground, float heading, float fuselage_angle, const vec3d *face_normal, const vec3d *aircraft_vector, vec3d *placed, float *angles)
{
	campaign_movement_step step;
	campaign_movement_result result;

	if (!campaign_movement_is_hosted (mobile))
	{
		return FALSE;
	}
	begin_step (&step, mobile, CAMPAIGN_MOVEMENT_CREATE_PLACE);
	// the campaign's formation point on the terrain (x, z and the sampled ground height) and the entity's physical offset above
	// it are inputs; the host answers the physical height
	step.position[0] = position->x;
	step.position[1] = position->y;
	step.position[2] = position->z;
	step.fw_cg_to_ground = cg_to_ground;
	step.hc_desired_heading = heading;
	step.fw_fuselage_angle = fuselage_angle;
	step.face_normal[0] = face_normal->x;
	step.face_normal[1] = face_normal->y;
	step.face_normal[2] = face_normal->z;
	if (aircraft_vector)
	{
		step.fw_motion_vector[0] = aircraft_vector->x;
		step.fw_motion_vector[1] = aircraft_vector->y;
		step.fw_motion_vector[2] = aircraft_vector->z;
	}
	result = ask_host (&step);
	placed->x = result.position[0];
	placed->y = result.position[1];
	placed->z = result.position[2];
	angles[0] = result.angles[0];
	angles[1] = result.angles[1];
	angles[2] = result.angles[2];
	return TRUE;
}

void campaign_movement_retire (const char *host_id)
{
	campaign_movement_step step;

	if (!current_host.move_vehicle || !host_id || !host_id[0])
	{
		return;
	}
	memset (&step, 0, sizeof (step));
	memcpy (step.host_id, host_id, strlen (host_id) + 1);
	step.mode = CAMPAIGN_MOVEMENT_RETIRE;
	ask_host (&step);
}

float campaign_movement_stop (entity *vehicle)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, vehicle, CAMPAIGN_MOVEMENT_STOP);
	result = ask_host (&step);
	return result.velocity;
}

void campaign_movement_move (entity *vehicle, entity *guide, const vec3d *waypoint)
{
	routed_vehicle *raw = (routed_vehicle *) get_local_entity_data (vehicle);
	campaign_movement_step step;
	campaign_movement_result result;
	vec3d wp_vec;
	vec3d new_pos;

	begin_step (&step, vehicle, CAMPAIGN_MOVEMENT_MOVE);
	step.delta_seconds = get_entity_movement_delta_time ();
	step.waypoint[0] = waypoint->x;
	step.waypoint[1] = waypoint->y;
	step.waypoint[2] = waypoint->z;
	step.tacview_logging = raw->vh.mob.tacview_logging ? TRUE : FALSE;
	fill_map_bounds (&step);

	wp_vec.x = waypoint->x - raw->vh.mob.position.x;
	wp_vec.z = waypoint->z - raw->vh.mob.position.z;

	/* not on top of the waypoint (sqr check within 0.1m): the host steps, and needs the speed inputs and terrain */
	if (((wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z)) > 1.0)
	{
		raw->desired_velocity = get_local_entity_float_value (guide, FLOAT_TYPE_VELOCITY);
		step.desired_velocity = raw->desired_velocity;
		step.max_acceleration = get_local_entity_float_value (vehicle, FLOAT_TYPE_MAX_ACCELERATION);

		if (get_local_entity_int_value (guide, INT_TYPE_TERRAIN_FOLLOW_MODE) != GUIDE_TERRAIN_FOLLOW_NONE)
		{
			vec3d *face_normal;

			new_pos = raw->vh.mob.position;

			ASSERT (point_inside_map_area (&new_pos));

			get_3d_terrain_point_data (new_pos.x, new_pos.z, &raw->vh.terrain_info);

			face_normal = get_3d_terrain_point_data_normal (&raw->vh.terrain_info);

			step.terrain_follow = TRUE;
			step.terrain_present = TRUE;
			step.elevation = get_3d_terrain_point_data_elevation (&raw->vh.terrain_info);
			step.forest = (get_terrain_type_class (get_3d_terrain_point_data_type (&raw->vh.terrain_info)) == TERRAIN_CLASS_FOREST) ? TRUE : FALSE;
			step.face_normal[0] = face_normal->x;
			step.face_normal[1] = face_normal->y;
			step.face_normal[2] = face_normal->z;
		}
	}

	result = ask_host (&step);

	//
	// apply: the speed and attitude as the legacy step wrote them, then the position through the setter
	//

	raw->vh.mob.velocity = result.velocity;
	memcpy (raw->vh.mob.attitude, result.attitude, sizeof (result.attitude));

	if (raw->vh.mob.tacview_logging && result.rotated)
	{
		set_local_entity_int_value (vehicle, INT_TYPE_ROTATED, TRUE);
	}

	new_pos.x = result.position[0];
	new_pos.y = result.position[1];
	new_pos.z = result.position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (vehicle, VEC3D_TYPE_POSITION, &new_pos);
	campaign_movement_apply_end ();

	//
	// Calculate motion vector for view system
	//

	raw->vh.mob.motion_vector.x = raw->vh.mob.attitude [2][0] * raw->vh.mob.velocity;
	raw->vh.mob.motion_vector.y = raw->vh.mob.attitude [2][1] * raw->vh.mob.velocity;
	raw->vh.mob.motion_vector.z = raw->vh.mob.attitude [2][2] * raw->vh.mob.velocity;
}

void campaign_movement_ship_move (entity *vehicle, entity *guide, const vec3d *waypoint)
{
	ship_vehicle *raw = (ship_vehicle *) get_local_entity_data (vehicle);
	campaign_movement_step step;
	campaign_movement_result result;
	vec3d new_pos;

	begin_step (&step, vehicle, CAMPAIGN_MOVEMENT_SHIP_MOVE);
	step.delta_seconds = get_entity_movement_delta_time ();
	step.frame_delta = get_one_over_delta_time ();
	step.desired_velocity = get_local_entity_float_value (guide, FLOAT_TYPE_VELOCITY);
	step.max_acceleration = get_local_entity_float_value (vehicle, FLOAT_TYPE_MAX_ACCELERATION);
	step.g_max = vehicle_database [raw->vh.mob.sub_type].g_max;
	step.waypoint[0] = waypoint->x;
	step.waypoint[1] = waypoint->y;
	step.waypoint[2] = waypoint->z;
	fill_map_bounds (&step);

	result = ask_host (&step);

	//
	// apply: the attitude, speed and motion vector as the legacy step wrote them, then the position through the setter
	//

	memcpy (raw->vh.mob.attitude, result.attitude, sizeof (result.attitude));
	raw->vh.mob.velocity = result.velocity;
	raw->vh.mob.motion_vector.x = result.motion_vector[0];
	raw->vh.mob.motion_vector.y = result.motion_vector[1];
	raw->vh.mob.motion_vector.z = result.motion_vector[2];

	new_pos.x = result.position[0];
	new_pos.y = result.position[1];
	new_pos.z = result.position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (vehicle, VEC3D_TYPE_POSITION, &new_pos);
	campaign_movement_apply_end ();
}

// a navigation identity the campaign resolved from its own roster: the host resolves the aircraft behind it.  An identity
// the roster does not have is passed on as unresolved (NULL or empty): the host decides whether the step reads it.  One
// that is supplied but cannot be carried is a fault here

static void check_navigation_id (const char *host_id, const char *role)
{
	if (host_id && (strlen (host_id) > CAMPAIGN_MOVEMENT_HOST_ID_MAX))
	{
		debug_fatal ("host movement: the host identity of the %s of a navigating aircraft is too long", role);
	}
}

void campaign_movement_fixed_wing_move (entity *aircraft, const campaign_fixed_wing_guidance *guidance)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (aircraft);
	campaign_movement_step step;
	campaign_movement_result result;
	vec3d new_pos;

	begin_step (&step, aircraft, CAMPAIGN_MOVEMENT_FIXED_WING_MOVE);
	step.delta_seconds = guidance->delta_seconds;
	step.frame_delta = guidance->frame_delta;
	step.desired_velocity = guidance->wp_vel;
	step.g_max = guidance->g_max;
	step.elevation = guidance->elevation;
	step.waypoint[0] = guidance->waypoint[0];
	step.waypoint[1] = guidance->waypoint[1];
	step.waypoint[2] = guidance->waypoint[2];
	step.tacview_logging = raw->ac.mob.tacview_logging ? TRUE : FALSE;
	fill_map_bounds (&step);
	step.fw_phase = guidance->phase;
	memcpy (step.fw_motion_vector, guidance->motion_vector, sizeof (step.fw_motion_vector));
	step.fw_cruise_velocity = guidance->cruise_velocity;
	step.fw_fuselage_angle = guidance->fuselage_angle;
	step.fw_cg_to_ground = guidance->cg_to_ground;
	step.fw_max_roll = guidance->max_roll;
	step.fw_navigating = guidance->navigating ? TRUE : FALSE;

	if (guidance->terrain.follow)
	{
		step.fw_terrain = TRUE;
		step.fw_terrain_min_height = guidance->terrain.min_height;
		memcpy (step.fw_terrain_guide_position, guidance->terrain.guide_position, sizeof (step.fw_terrain_guide_position));
		memcpy (step.fw_terrain_sample_point, guidance->terrain.sample_point, sizeof (step.fw_terrain_sample_point));
		step.fw_terrain_sample_elevation = guidance->terrain.sample_elevation;
		step.fw_terrain_site_y = guidance->terrain.site_y;
		step.fw_terrain_cg = guidance->terrain.cg;
	}

	if ((guidance->phase == CAMPAIGN_FIXED_WING_FLIGHT) || (guidance->phase == CAMPAIGN_FIXED_WING_LAND))
	{
		int member;

		if (guidance->member_count < 0)
		{
			debug_fatal ("host movement: a navigating aircraft has %d group members", guidance->member_count);
		}

		step.fw_aiming_cannon = guidance->aiming_cannon ? TRUE : FALSE;
		step.fw_elapsed_time = guidance->elapsed_time;
		step.fw_group_leader = guidance->group_leader ? TRUE : FALSE;
		step.fw_task_leader = guidance->task_leader ? TRUE : FALSE;
		step.fw_max_vertical_turn_allowance = guidance->max_vertical_turn_allowance;
		step.fw_avoidance_radius = guidance->avoidance_radius;

		if (!guidance->task_leader)
		{
			check_navigation_id (guidance->leader_host_id, "task leader");

			if (guidance->leader_host_id)
			{
				memcpy (step.fw_leader_host_id, guidance->leader_host_id, strlen (guidance->leader_host_id) + 1);
			}
		}

		// the group leader never tests for a collision, so it sends no members; the members are checked here, before the
		// host sees the step, and travel as the caller's own lists, whatever their length
		if (!guidance->group_leader)
		{
			if ((guidance->member_count > 0) && (!guidance->member_host_id || !guidance->member_radius))
			{
				debug_fatal ("host movement: a navigating aircraft has %d group members without their lists", guidance->member_count);
			}

			step.fw_member_count = guidance->member_count;
			step.fw_member_host_id = guidance->member_host_id;
			step.fw_member_radius = guidance->member_radius;

			for (member = 0; member < guidance->member_count; member++)
			{
				check_navigation_id (guidance->member_host_id [member], "group member");
			}
		}
	}

	result = ask_host (&step);

	//
	// apply: the attitude, speed and motion vector as the legacy step wrote them, then the position through the setter
	//

	memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));
	raw->ac.mob.velocity = result.velocity;
	raw->ac.mob.motion_vector.x = result.motion_vector[0];
	raw->ac.mob.motion_vector.y = result.motion_vector[1];
	raw->ac.mob.motion_vector.z = result.motion_vector[2];

	if (raw->ac.mob.tacview_logging && result.rotated)
	{
		set_local_entity_int_value (aircraft, INT_TYPE_ROTATED, TRUE);
	}

	new_pos.x = result.position[0];
	new_pos.y = result.position[1];
	new_pos.z = result.position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (aircraft, VEC3D_TYPE_POSITION, &new_pos);
	campaign_movement_apply_end ();
}

// an alive AI helicopter, phase one: the motion vector and the speed are applied at once, as the legacy step wrote them,
// because the campaign's attitude guidance reads them before phase two

void campaign_movement_helicopter_begin (entity *en, float acceleration_x, float acceleration_z, float wp_vec_y)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_BEGIN);
	step.delta_seconds = get_entity_movement_delta_time ();
	step.hc_acceleration_x = acceleration_x;
	step.hc_acceleration_z = acceleration_z;
	step.hc_wp_vec_y = wp_vec_y;
	step.hc_motion_vector[0] = raw->ac.mob.motion_vector.x;
	step.hc_motion_vector[1] = raw->ac.mob.motion_vector.y;
	step.hc_motion_vector[2] = raw->ac.mob.motion_vector.z;

	result = ask_host (&step);

	if (result.pending != TRUE)
	{
		debug_fatal ("host movement: host move_vehicle did not hold the helicopter step of %s for its second phase", step.host_id);
	}

	raw->ac.mob.motion_vector.x = result.motion_vector[0];
	raw->ac.mob.motion_vector.y = result.motion_vector[1];
	raw->ac.mob.motion_vector.z = result.motion_vector[2];

	campaign_movement_apply_begin ();
	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, result.velocity);
	campaign_movement_apply_end ();

	if (result.moved)
	{
		set_local_entity_int_value (en, INT_TYPE_MOVED, TRUE);
	}
}

// phase two: the attitude, the position and the floor-clamped vector; the position goes through the setter with the
// aircraft's own stored position as the argument, as the legacy step passed it

void campaign_movement_helicopter_end (entity *en, float desired_heading, float max_turn_rate, float desired_pitch, float desired_roll, float elevation, float cg_to_ground, int terrain_follow)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_END);
	step.delta_seconds = get_entity_movement_delta_time ();
	step.hc_desired_heading = desired_heading;
	step.hc_max_turn_rate = max_turn_rate;
	step.hc_desired_pitch = desired_pitch;
	step.hc_desired_roll = desired_roll;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;
	step.terrain_follow = terrain_follow ? TRUE : FALSE;
	step.hc_motion_vector[0] = raw->ac.mob.motion_vector.x;
	step.hc_motion_vector[1] = raw->ac.mob.motion_vector.y;
	step.hc_motion_vector[2] = raw->ac.mob.motion_vector.z;
	fill_map_bounds (&step);

	result = ask_host (&step);

	if (result.pending)
	{
		debug_fatal ("host movement: host move_vehicle left the helicopter step of %s pending after its second phase", step.host_id);
	}

	memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));

	if (result.rotated)
	{
		set_local_entity_int_value (en, INT_TYPE_ROTATED, TRUE);
	}

	raw->ac.mob.motion_vector.x = result.motion_vector[0];
	raw->ac.mob.motion_vector.y = result.motion_vector[1];
	raw->ac.mob.motion_vector.z = result.motion_vector[2];

	raw->ac.mob.position.x = result.position[0];
	raw->ac.mob.position.y = result.position[1];
	raw->ac.mob.position.z = result.position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &raw->ac.mob.position);
	campaign_movement_apply_end ();
}

//
// EEDCS-39 host-planned helicopter control.  The campaign supplies facts and the DLL's lookups; the host owns the guidance
// and the physical demands.  Each helper builds a zeroed step, so a field a mode does not name is zero.
//

static void copy_helicopter_motion_vector (campaign_movement_step *step, const helicopter *raw)
{
	step->hc_motion_vector[0] = raw->ac.mob.motion_vector.x;
	step->hc_motion_vector[1] = raw->ac.mob.motion_vector.y;
	step->hc_motion_vector[2] = raw->ac.mob.motion_vector.z;
}

// step one: facts and members in, the lookups the DLL must make out.  Nothing is applied and no hold is required of the host
// beyond what the plan names

void campaign_movement_helicopter_plan (entity *en, const campaign_hc_plan *plan, campaign_hc_query *query)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;
	int member;

	if (plan->member_count < 0)
	{
		debug_fatal ("host movement: a navigating helicopter has %d group members", plan->member_count);
	}

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_PLAN);
	step.delta_seconds = plan->delta_seconds;
	step.terrain_follow = plan->terrain_follow ? TRUE : FALSE;
	step.elevation = plan->terrain_elevation;
	step.waypoint[0] = plan->waypoint[0];
	step.waypoint[1] = plan->waypoint[1];
	step.waypoint[2] = plan->waypoint[2];
	fill_map_bounds (&step);
	copy_helicopter_motion_vector (&step, raw);
	step.hc_power_output = plan->power_output;

	step.hc_group_leader = plan->group_leader ? TRUE : FALSE;
	step.hc_task_leader = plan->task_leader ? TRUE : FALSE;
	step.hc_task_leader_is_self = plan->task_leader_is_self ? TRUE : FALSE;
	step.hc_position_hold = plan->position_hold ? TRUE : FALSE;
	step.hc_terrain_follow_attack_altitude = plan->terrain_follow_attack_altitude ? TRUE : FALSE;
	step.hc_waypoint_kind = plan->waypoint_kind;
	step.hc_guide_radius_valid = plan->guide_radius_valid ? TRUE : FALSE;
	step.hc_guide_radius = plan->guide_radius;
	step.hc_guide_altitude_valid = plan->guide_altitude_valid ? TRUE : FALSE;
	memcpy (step.hc_waypoint_velocity, plan->waypoint_velocity, sizeof (step.hc_waypoint_velocity));
	step.hc_distance = plan->distance;
	step.hc_fuel_supply_level = plan->fuel_supply_level;
	step.hc_cruise_velocity = plan->cruise_velocity;
	step.hc_cruise_altitude = plan->cruise_altitude;
	step.hc_attack_altitude = plan->attack_altitude;
	step.hc_leader_radar_altitude = plan->leader_radar_altitude;

	// the group leader never tests for a collision, so it sends no members; the others travel as the caller's own list,
	// whatever its length
	if (!plan->group_leader)
	{
		if ((plan->member_count > 0) && !plan->member_host_id)
		{
			debug_fatal ("host movement: a navigating helicopter has %d group members without their list", plan->member_count);
		}

		step.hc_member_count = plan->member_count;
		step.hc_member_host_id = plan->member_host_id;

		for (member = 0; member < plan->member_count; member++)
		{
			check_navigation_id (plan->member_host_id [member], "group member");
		}
	}

	result = ask_host (&step);

	*query = result.hc_query;
}

// step two: the lookups' results in (own-sector structures in sector order; each terrain sample's getter returns in call
// order), the host's guidance out.  The lists are the caller's own and are read only during the call

void campaign_movement_helicopter_guide (entity *en, int structure_count, const campaign_hc_structure *structures, int terrain_sample_count, const campaign_hc_terrain_sample *terrain_samples, campaign_hc_guidance_answer *guidance)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	if ((structure_count < 0) || (terrain_sample_count < 0))
	{
		debug_fatal ("host movement: a navigating helicopter has %d structures and %d terrain samples", structure_count, terrain_sample_count);
	}

	if (((structure_count > 0) && !structures) || ((terrain_sample_count > 0) && !terrain_samples))
	{
		debug_fatal ("host movement: a navigating helicopter has %d structures and %d terrain samples without their lists", structure_count, terrain_sample_count);
	}

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_GUIDE);
	copy_helicopter_motion_vector (&step, raw);
	step.hc_structure_count = structure_count;
	step.hc_structures = structures;
	step.hc_terrain_sample_count = terrain_sample_count;
	step.hc_terrain_samples = terrain_samples;

	result = ask_host (&step);

	*guidance = result.hc_guidance;
}

// step three: as the legacy phase one, the motion vector and speed applied at once, but the host's own retained guidance
// supplies the acceleration.  The position is not touched

void campaign_movement_helicopter_guided_begin (entity *en)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_BEGIN);
	step.delta_seconds = get_entity_movement_delta_time ();
	copy_helicopter_motion_vector (&step, raw);

	result = ask_host (&step);

	if (result.pending != TRUE)
	{
		debug_fatal ("host movement: host move_vehicle did not hold the helicopter step of %s for its second phase", step.host_id);
	}

	raw->ac.mob.motion_vector.x = result.motion_vector[0];
	raw->ac.mob.motion_vector.y = result.motion_vector[1];
	raw->ac.mob.motion_vector.z = result.motion_vector[2];

	campaign_movement_apply_begin ();
	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, result.velocity);
	campaign_movement_apply_end ();

	if (result.moved)
	{
		set_local_entity_int_value (en, INT_TYPE_MOVED, TRUE);
	}
}

// step four: the steering facts in (the elevation is the altitude follow's cached sample; no terrain is queried here), the
// host's attitude, rotated flag, motion vector and position applied as the legacy phase two did, its heading, pitch and roll
// out.  The leader is named by host id, and only when this aircraft is not its own task leader

void campaign_movement_helicopter_guided_end (entity *en, const campaign_hc_steer *steer, float angles[3])
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_GUIDED_END);
	step.delta_seconds = steer->delta_seconds;
	step.hc_max_turn_rate = steer->max_turn_rate;
	step.elevation = steer->elevation;
	step.hc_cg_to_ground = steer->cg_to_ground;
	step.g_max = steer->g_max;
	step.terrain_follow = steer->terrain_follow ? TRUE : FALSE;
	copy_helicopter_motion_vector (&step, raw);
	fill_map_bounds (&step);

	step.hc_waypoint_kind = steer->waypoint_kind;
	step.hc_task_leader_is_self = steer->task_leader_is_self ? TRUE : FALSE;
	step.hc_guide_radius_valid = steer->guide_radius_valid ? TRUE : FALSE;
	step.hc_guide_radius = steer->guide_radius;
	step.hc_required_heading_valid = steer->required_heading_valid ? TRUE : FALSE;
	step.hc_required_heading = steer->required_heading;
	step.hc_targeting_active = steer->targeting_active ? TRUE : FALSE;
	memcpy (step.hc_weapon_vector, steer->weapon_vector, sizeof (step.hc_weapon_vector));
	memcpy (step.hc_weapon_to_intercept_vector, steer->weapon_to_intercept_vector, sizeof (step.hc_weapon_to_intercept_vector));
	step.hc_main_rotor_shaft_angle = steer->main_rotor_shaft_angle;

	if (!steer->task_leader_is_self)
	{
		check_navigation_id (steer->leader_host_id, "task leader");

		if (steer->leader_host_id)
		{
			memcpy (step.hc_leader_host_id, steer->leader_host_id, strlen (steer->leader_host_id) + 1);
		}
	}

	result = ask_host (&step);

	if (result.pending)
	{
		debug_fatal ("host movement: host move_vehicle left the helicopter step of %s pending after its second phase", step.host_id);
	}

	memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));

	if (result.rotated)
	{
		set_local_entity_int_value (en, INT_TYPE_ROTATED, TRUE);
	}

	raw->ac.mob.motion_vector.x = result.motion_vector[0];
	raw->ac.mob.motion_vector.y = result.motion_vector[1];
	raw->ac.mob.motion_vector.z = result.motion_vector[2];

	raw->ac.mob.position.x = result.position[0];
	raw->ac.mob.position.y = result.position[1];
	raw->ac.mob.position.z = result.position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &raw->ac.mob.position);
	campaign_movement_apply_end ();

	memcpy (angles, result.angles, sizeof (result.angles));
}

// drops an open plan host-side; nothing is applied

void campaign_movement_helicopter_abandon (entity *en)
{
	campaign_movement_step step;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_HELICOPTER_ABANDON);
	ask_host (&step);
}

// the look-ahead point of a terrain-following fixed-wing waypoint: the host's own pose decides it, nothing is applied

void campaign_movement_fixed_wing_lookahead (entity *aircraft, float *point_x, float *point_z)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, aircraft, CAMPAIGN_MOVEMENT_FIXED_WING_LOOKAHEAD);
	result = ask_host (&step);

	if (result.hc_query.terrain_sample_count != 1)
	{
		debug_fatal ("host movement: the host answered %d look-ahead points for %s", result.hc_query.terrain_sample_count, step.host_id);
	}

	*point_x = result.hc_query.terrain_sample[0][0];
	*point_z = result.hc_query.terrain_sample[0][1];
}

// helicopter rotor and dying/crash physics.  Every mode carries the DLL's copy of the motion vector and the rotor state,
// which the host refuses a difference from (except ROTOR_SET, the explicit seed/reset it adopts).  The host's results are
// written back as raw fields or inside apply_begin/end, never as placements.

static void fill_helicopter_state (campaign_movement_step *step, const helicopter *raw)
{
	step->hc_motion_vector[0] = raw->ac.mob.motion_vector.x;
	step->hc_motion_vector[1] = raw->ac.mob.motion_vector.y;
	step->hc_motion_vector[2] = raw->ac.mob.motion_vector.z;
	step->hc_rotor[0] = raw->main_rotor_rpm;
	step->hc_rotor[1] = raw->tail_rotor_rpm;
	step->hc_rotor[2] = raw->main_rotor_spin_up_timer;
}

static void apply_rotor (helicopter *raw, const float *rotor)
{
	raw->main_rotor_rpm = rotor[0];
	raw->tail_rotor_rpm = rotor[1];
	raw->main_rotor_spin_up_timer = rotor[2];
}

static void apply_motion_vector (helicopter *raw, const float *motion_vector)
{
	raw->ac.mob.motion_vector.x = motion_vector[0];
	raw->ac.mob.motion_vector.y = motion_vector[1];
	raw->ac.mob.motion_vector.z = motion_vector[2];
}

static campaign_movement_result ask_lifecycle (campaign_movement_step *step, int expect_pending)
{
	campaign_movement_result result = ask_host (step);

	if (result.pending != expect_pending)
	{
		debug_fatal ("host movement: host move_vehicle answered pending %d for helicopter mode %d of %s", result.pending, step->mode, step->host_id);
	}
	return result;
}

// dying and crashing helicopters hand the POSITION setter a separate local position: the raw position still holds the old
// pose while dependents are notified, as the legacy step did (only alive integration aliases the raw position)

static void apply_position (entity *en, const float *position)
{
	vec3d new_pos;

	new_pos.x = position[0];
	new_pos.y = position[1];
	new_pos.z = position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &new_pos);
	campaign_movement_apply_end ();
}

// an explicit seed/reset of the rotor state (creation, unpack, setter, wind-down): the host adopts it

void campaign_movement_rotor_set (entity *en)
{
	campaign_movement_step step;

	if (applying_host_result || !current_host.move_vehicle)
	{
		return;
	}
	if ((get_local_entity_type (en) != ENTITY_TYPE_HELICOPTER) || !campaign_roster_host_id (en))
	{
		return;
	}
	begin_step (&step, en, CAMPAIGN_MOVEMENT_ROTOR_SET);
	fill_helicopter_state (&step, (const helicopter *) get_local_entity_data (en));
	ask_lifecycle (&step, FALSE);
}

// alive: the rotor integration and the timer; the result is whether the DLL's spin-up gate held (the DLL then returns)

int campaign_movement_rotor_alive (entity *en, float lift, float power_output, float delta_seconds, int gate)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_ROTOR_ALIVE);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.hc_lift = lift;
	step.hc_power_output = power_output;
	step.hc_flag = gate ? TRUE : FALSE;

	result = ask_lifecycle (&step, FALSE);

	apply_rotor (raw, result.rotor);
	return result.moved;
}

void campaign_movement_rotor_down (entity *en, float delta_seconds)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_ROTOR_DOWN);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;

	result = ask_lifecycle (&step, FALSE);

	apply_rotor (raw, result.rotor);
}

// idle (no follower parent): the linear wind-down by the FRAME delta, a different law from the dying wind-down above

void campaign_movement_rotor_idle (entity *en, float delta_seconds)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	campaign_movement_require_hosted (en);

	begin_step (&step, en, CAMPAIGN_MOVEMENT_ROTOR_IDLE);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;

	result = ask_lifecycle (&step, FALSE);

	apply_rotor (raw, result.rotor);
}

// dying, phase one: the host holds the candidate position (map-clamped); nothing else changes, so the DLL samples the
// terrain at the candidate before the gravity step, as the legacy step did

void campaign_movement_fall (entity *en, float delta_seconds, vec3d *candidate)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FALL);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;
	fill_map_bounds (&step);

	result = ask_lifecycle (&step, FALSE);

	candidate->x = result.position[0];
	candidate->y = result.position[1];
	candidate->z = result.position[2];
}

// dying, phase two (after the terrain sample): gravity, the vector written raw and the speed through the setter, as the
// legacy step wrote them

void campaign_movement_fall_gravity (entity *en, float gravity, float delta_seconds)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FALL_GRAVITY);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.hc_gravity = gravity;

	result = ask_lifecycle (&step, FALSE);

	apply_motion_vector (raw, result.motion_vector);

	campaign_movement_apply_begin ();
	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, result.velocity);
	campaign_movement_apply_end ();
}

// the altitude of the held position (use_candidate FALSE) or the FALL candidate over the campaign's terrain sample, with the
// CG height the caller read BEFORE the kill; the host retains the candidate altitude for SETTLE

int campaign_movement_contact (entity *en, int use_candidate, float elevation, float cg_to_ground, float *altitude)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_CONTACT);
	fill_helicopter_state (&step, raw);
	step.hc_flag = use_candidate ? TRUE : FALSE;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;

	result = ask_lifecycle (&step, FALSE);

	*altitude = result.altitude;
	return result.moved;
}

// settle on or in the terrain class.  The bury tests use the altitude the host retained from CONTACT(candidate), which was
// measured with the pre-kill CG; cg_to_ground here is the CG queried afresh AFTER the kill (the destroyed CG) and sets only
// the final heights.  A bury zeroes the rotor (written back here) and the host's vector (the caller's
// clear_helicopter_velocity announces the zero at the original point)

int campaign_movement_settle (entity *en, int terrain_class, float elevation, float cg_to_ground, float bury_limit, vec3d *candidate)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_SETTLE);
	fill_helicopter_state (&step, raw);
	step.hc_terrain_class = terrain_class;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;
	step.hc_bury_limit = bury_limit;

	result = ask_lifecycle (&step, FALSE);

	candidate->x = result.position[0];
	candidate->y = result.position[1];
	candidate->z = result.position[2];

	apply_rotor (raw, result.rotor);
	return result.moved;
}

// the tumble attitude and the host's final vector and position (the settled or fall candidate), the position through the setter

void campaign_movement_tumble (entity *en, double heading_draw, double roll_draw)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_TUMBLE);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = get_entity_movement_delta_time ();
	step.hc_draws[0] = heading_draw;
	step.hc_draws[1] = roll_draw;

	result = ask_lifecycle (&step, FALSE);

	memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));
	apply_motion_vector (raw, result.motion_vector);
	apply_position (en, result.position);
}

// crash slide, phase one: the attitude from the OLD terrain normal (the caller passes the terrain info's normal before the
// refresh) and the candidate position; the step awaits CRASH_END with nothing else in between

void campaign_movement_crash_begin (entity *en, const vec3d *old_normal, float pitch_offset, float bank_offset, float delta_seconds, vec3d *candidate)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_CRASH_BEGIN);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.face_normal[0] = old_normal->x;
	step.face_normal[1] = old_normal->y;
	step.face_normal[2] = old_normal->z;
	step.hc_pitch_offset = pitch_offset;
	step.hc_bank_offset = bank_offset;
	fill_map_bounds (&step);

	result = ask_lifecycle (&step, TRUE);

	memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));

	candidate->x = result.position[0];
	candidate->y = result.position[1];
	candidate->z = result.position[2];
}

// phase two: the NEW (refreshed) terrain normal gives the slope push.  The pre-clear vector is written, then the position
// through the setter; a stopped slide is returned for the caller to clear AFTER that setter, otherwise the speed is set here

int campaign_movement_crash_end (entity *en, const vec3d *new_normal, int forest, float elevation, float cg_to_ground, float delta_seconds)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_CRASH_END);
	fill_helicopter_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.face_normal[0] = new_normal->x;
	step.face_normal[1] = new_normal->y;
	step.face_normal[2] = new_normal->z;
	step.forest = forest ? TRUE : FALSE;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;

	result = ask_lifecycle (&step, FALSE);

	apply_motion_vector (raw, result.motion_vector);
	apply_position (en, result.position);

	if (!result.moved)
	{
		campaign_movement_apply_begin ();
		set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, result.velocity);
		campaign_movement_apply_end ();
	}
	return result.moved;
}

// the shove along the attitude x axis, decided (with its random draw) by the DLL

void campaign_movement_impact (entity *en, float signed_speed)
{
	helicopter *raw = (helicopter *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_IMPACT);
	fill_helicopter_state (&step, raw);
	step.hc_signed_speed = signed_speed;

	result = ask_lifecycle (&step, FALSE);

	apply_motion_vector (raw, result.motion_vector);
}

// EEDCS-37: fixed-wing dying and crash.  Same host protocol as the helicopter above, on the fixed_wing part and with no
// rotor: the fall, gravity, contact and impact use the helicopter modes (the host dispatches by member kind), hc_rotor is
// zero and ignored, and hc_motion_vector is the DLL copy of the vector the host retains for the member.

static void fill_fw_state (campaign_movement_step *step, const fixed_wing *raw)
{
	step->hc_motion_vector[0] = raw->ac.mob.motion_vector.x;
	step->hc_motion_vector[1] = raw->ac.mob.motion_vector.y;
	step->hc_motion_vector[2] = raw->ac.mob.motion_vector.z;
}

static void apply_fw_vector (fixed_wing *raw, const float *motion_vector)
{
	raw->ac.mob.motion_vector.x = motion_vector[0];
	raw->ac.mob.motion_vector.y = motion_vector[1];
	raw->ac.mob.motion_vector.z = motion_vector[2];
}

static void take_candidate (vec3d *candidate, const campaign_movement_result *result)
{
	candidate->x = result->position[0];
	candidate->y = result->position[1];
	candidate->z = result->position[2];
}

void campaign_movement_fw_fall (entity *en, float delta_seconds, vec3d *candidate)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FALL);
	fill_fw_state (&step, raw);
	step.delta_seconds = delta_seconds;
	fill_map_bounds (&step);

	result = ask_lifecycle (&step, FALSE);

	take_candidate (candidate, &result);
}

// the vector raw and the speed through the setter at once, as the legacy death step wrote them (the attitude block reads the
// post-gravity speed the DLL keeps for it)

float campaign_movement_fw_fall_gravity (entity *en, float gravity, float delta_seconds)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FALL_GRAVITY);
	fill_fw_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.hc_gravity = gravity;

	result = ask_lifecycle (&step, FALSE);

	apply_fw_vector (raw, result.motion_vector);
	apply_speed (en, result.velocity);

	return result.velocity;
}

int campaign_movement_fw_contact (entity *en, int use_candidate, float elevation, float cg_to_ground, float *altitude)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_CONTACT);
	fill_fw_state (&step, raw);
	step.hc_flag = use_candidate ? TRUE : FALSE;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;

	result = ask_lifecycle (&step, FALSE);

	*altitude = result.altitude;
	return result.moved;
}

// the bury test uses the altitude retained from CONTACT(candidate) (pre-kill CG); cg_to_ground is the fresh post-kill CG and
// sets only the final heights.  A bury zeroes the host vector; the caller's clear_fixed_wing_velocity announces the zero at the
// original point.  The vector is written back raw, the candidate updated.

int campaign_movement_fw_settle (entity *en, int terrain_class, float elevation, float cg_to_ground, float bury_limit, vec3d *candidate)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FW_SETTLE);
	fill_fw_state (&step, raw);
	step.hc_terrain_class = terrain_class;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;
	step.hc_bury_limit = bury_limit;

	result = ask_lifecycle (&step, FALSE);

	take_candidate (candidate, &result);
	apply_fw_vector (raw, result.motion_vector);

	return result.moved;
}

// the dying attitude block then the final position through the setter, which is last as in the legacy step.  `draw_made` is
// whether the DLL drew the roll (only when altitude > 12 and the vector has x or z); the host recomputes the gate and a
// mismatch is fatal.  `speed` is the post-gravity, pre-bury magnitude.

void campaign_movement_fw_death_attitude (entity *en, float altitude, float speed, int draw_made, double roll_draw, float delta_seconds)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FW_DEATH_ATTITUDE);
	fill_fw_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.elevation = altitude;
	step.hc_signed_speed = speed;
	step.hc_flag = draw_made ? TRUE : FALSE;
	step.hc_draws[1] = roll_draw;

	result = ask_lifecycle (&step, FALSE);

	if (result.flags < 0)
	{
		debug_fatal ("host movement: fixed-wing dying attitude gate disagrees with the campaign's roll draw (%d) for %s", draw_made, step.host_id);
	}
	if (result.flags & 1)
	{
		memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));
	}

	apply_position (en, result.position);
}

// the crash slide, phase one: the map-clamped candidate x and z with the held y; the DLL then refreshes the terrain there

void campaign_movement_fw_crash_candidate (entity *en, float delta_seconds, vec3d *candidate)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FW_CRASH_CANDIDATE);
	fill_fw_state (&step, raw);
	step.delta_seconds = delta_seconds;
	fill_map_bounds (&step);

	result = ask_lifecycle (&step, FALSE);

	take_candidate (candidate, &result);
}

// phase two, with the NEW normal: friction and the slope push.  The vector is written raw; the speed is set here unless the
// slide stopped (speed < 1), in which case the caller runs clear_fixed_wing_velocity.  Both are before the attitude and the
// position, as in the legacy step.  Returns whether the slide stopped.

int campaign_movement_fw_crash_velocity (entity *en, const vec3d *new_normal, int forest, float elevation, float cg_to_ground, float delta_seconds)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FW_CRASH_VELOCITY);
	fill_fw_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.face_normal[0] = new_normal->x;
	step.face_normal[1] = new_normal->y;
	step.face_normal[2] = new_normal->z;
	step.forest = forest ? TRUE : FALSE;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;

	result = ask_lifecycle (&step, FALSE);

	apply_fw_vector (raw, result.motion_vector);

	if (!(result.flags & 1))
	{
		apply_speed (en, result.velocity);
	}
	return (result.flags & 1);
}

// phase three: the attitude eases to the NEW normal plus the destroyed offsets (raw), then the position setter.  Returns the
// at-rest test without the DLL's !alive term (the speed is the host's retained post-slope magnitude).

int campaign_movement_fw_crash_attitude (entity *en, const vec3d *new_normal, float pitch_offset, float bank_offset, float delta_seconds)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_FW_CRASH_ATTITUDE);
	fill_fw_state (&step, raw);
	step.delta_seconds = delta_seconds;
	step.face_normal[0] = new_normal->x;
	step.face_normal[1] = new_normal->y;
	step.face_normal[2] = new_normal->z;
	step.hc_pitch_offset = pitch_offset;
	step.hc_bank_offset = bank_offset;

	result = ask_lifecycle (&step, FALSE);

	memcpy (raw->ac.mob.attitude, result.attitude, sizeof (result.attitude));
	apply_position (en, result.position);

	return (result.flags & 1);
}

void campaign_movement_fw_impact (entity *en, float signed_speed)
{
	fixed_wing *raw = (fixed_wing *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_IMPACT);
	fill_fw_state (&step, raw);
	step.hc_signed_speed = signed_speed;

	result = ask_lifecycle (&step, FALSE);

	apply_fw_vector (raw, result.motion_vector);
}

// EEDCS-37: routed, anti-air and ship wreck/fall/sink.  The host claims the vehicle vector at the first VEHICLE_VECTOR_SEED
// (adopted only while it retains none) and every later mode sends the DLL's copy in hc_motion_vector, which the host validates
// bit-identically against the held vector and refuses on a difference.  The host's result is written
// back raw or through the setters inside apply_begin/end, in the order the legacy step wrote it.

static void fill_vehicle_vector (campaign_movement_step *step, const vehicle *raw)
{
	step->hc_motion_vector[0] = raw->mob.motion_vector.x;
	step->hc_motion_vector[1] = raw->mob.motion_vector.y;
	step->hc_motion_vector[2] = raw->mob.motion_vector.z;
}

static void apply_vehicle_vector (vehicle *raw, const float *motion_vector)
{
	raw->mob.motion_vector.x = motion_vector[0];
	raw->mob.motion_vector.y = motion_vector[1];
	raw->mob.motion_vector.z = motion_vector[2];
}

static void apply_speed (entity *en, float speed)
{
	campaign_movement_apply_begin ();
	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, speed);
	campaign_movement_apply_end ();
}

void campaign_movement_vehicle_vector_seed (entity *vehicle_en)
{
	campaign_movement_step step;

	if (applying_host_result || !current_host.move_vehicle)
	{
		return;
	}
	if (!campaign_roster_host_id (vehicle_en))
	{
		return;
	}
	begin_step (&step, vehicle_en, CAMPAIGN_MOVEMENT_VEHICLE_VECTOR_SEED);
	fill_vehicle_vector (&step, (const vehicle *) get_local_entity_data (vehicle_en));
	ask_lifecycle (&step, FALSE);
}

// wreck slide, phase one: the candidate x/z (no map bound, as the legacy ASSERT) before the DLL refreshes the terrain there

void campaign_movement_gv_death_candidate (entity *en, float delta_seconds, float *candidate_xz)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_DEATH_CANDIDATE);
	fill_vehicle_vector (&step, (const vehicle *) get_local_entity_data (en));
	step.delta_seconds = delta_seconds;

	result = ask_lifecycle (&step, FALSE);

	candidate_xz[0] = result.position[0];
	candidate_xz[1] = result.position[2];
}

// phase two (after the terrain refresh at the candidate): the speed through the setter, the decayed vector and attitude raw,
// then the position through the setter, as the legacy step wrote them.  candidate_xz travels in waypoint[0] and waypoint[2].
// Returns the new speed (the routed DEAD gate on speed == 0 stays with the caller, which decided before the candidate)

float campaign_movement_gv_death_decay (entity *en, const float *candidate_xz, float elevation, float cg_to_ground, float delta_seconds, int anti_air)
{
	vehicle *raw = (vehicle *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_DEATH_DECAY);
	fill_vehicle_vector (&step, raw);
	step.delta_seconds = delta_seconds;
	step.waypoint[0] = candidate_xz[0];
	step.waypoint[2] = candidate_xz[1];
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;
	step.gv_wide = anti_air ? TRUE : FALSE;

	result = ask_lifecycle (&step, FALSE);

	apply_speed (en, result.velocity);
	apply_vehicle_vector (raw, result.motion_vector);
	memcpy (raw->mob.attitude, result.attitude, sizeof (result.attitude));
	apply_position (en, result.position);

	return result.velocity;
}

void campaign_movement_gv_fall_candidate (entity *en, float delta_seconds, vec3d *candidate)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_FALL_CANDIDATE);
	fill_vehicle_vector (&step, (const vehicle *) get_local_entity_data (en));
	step.delta_seconds = delta_seconds;

	result = ask_lifecycle (&step, FALSE);

	candidate->x = result.position[0];
	candidate->y = result.position[1];
	candidate->z = result.position[2];
}

// gravity on the vector after the refresh, written raw in place (the legacy `velocity->y -= G * dt`); the speed is set at the
// end of the legacy step (after the settle), so it is not applied here

void campaign_movement_gv_fall_gravity (entity *en, float gravity, float delta_seconds)
{
	vehicle *raw = (vehicle *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_FALL_GRAVITY);
	fill_vehicle_vector (&step, raw);
	step.delta_seconds = delta_seconds;
	step.hc_gravity = gravity;

	result = ask_lifecycle (&step, FALSE);

	apply_vehicle_vector (raw, result.motion_vector);
}

// contact / splash / sunk / ground impact over the terrain at the candidate: returns the flags (bit0 contact, bit1 splash,
// bit2 sunk, bit3 ground impact), the settled position and the final speed for the DLL's effects.  The host STAGES the
// result (vector zeroed when sunk, y zeroed on ground, settled position, scalar speed); nothing is written here

int campaign_movement_gv_fall_settle (entity *en, const vec3d *candidate, int water, float old_elevation, float elevation, float cg_to_ground, float bbox_diagonal, vec3d *settled, float *speed)
{
	vehicle *raw = (vehicle *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_FALL_SETTLE);
	fill_vehicle_vector (&step, raw);
	step.waypoint[0] = candidate->x;
	step.waypoint[1] = candidate->y;
	step.waypoint[2] = candidate->z;
	step.gv_water = water ? TRUE : FALSE;
	step.gv_old_elevation = old_elevation;
	step.elevation = elevation;
	step.hc_cg_to_ground = cg_to_ground;
	step.gv_bbox_diagonal = bbox_diagonal;

	result = ask_lifecycle (&step, FALSE);

	/* nothing is applied: the host stages the settled position, vector and speed; the DLL's effects run first */

	settled->x = result.position[0];
	settled->y = result.position[1];
	settled->z = result.position[2];
	*speed = result.velocity;

	return result.flags;
}

// where the legacy step wrote the vector (sunk: before DEAD; ground: after the effect): the staged vector, raw.  The host
// compares the DLL copy with the retained vector first, so an effect that wrote the vector itself is refused

void campaign_movement_gv_fall_vector (entity *en)
{
	vehicle *raw = (vehicle *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_FALL_VECTOR);
	fill_vehicle_vector (&step, raw);

	result = ask_lifecycle (&step, FALSE);

	apply_vehicle_vector (raw, result.motion_vector);
}

// the end of the legacy falling step, after the effects and the DEAD: the host finalises its staged state (retained vector,
// scalar speed, settled pose) and the DLL then writes the speed through the setter and the position through the setter,
// in that order.  An intermediate collision POSITION (an explicit PLACE) has not lost the staged candidate

void campaign_movement_gv_commit (entity *en)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_FALL_COMMIT);
	fill_vehicle_vector (&step, (const vehicle *) get_local_entity_data (en));

	result = ask_lifecycle (&step, FALSE);

	apply_speed (en, result.velocity);
	apply_position (en, result.position);
}

// the shove along the attitude x axis (attitude row 0 is xv); the DLL drew the speed from the entity's seed and narrowed it to a float, as the original stored it

void campaign_movement_gv_impact (entity *en, float speed)
{
	vehicle *raw = (vehicle *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_GV_IMPACT);
	fill_vehicle_vector (&step, raw);
	step.hc_signed_speed = speed;

	result = ask_lifecycle (&step, FALSE);

	apply_vehicle_vector (raw, result.motion_vector);
}

// ship sinking: the candidate position, drag, sink rate, pitch/roll drift and attitude, written as the legacy step did:
// vector raw, speed through the setter, attitude raw, then the position through the setter.  Returns the new height for the
// DLL's destroy test (the DLL keeps clear_ship_fires and the destroy)

float campaign_movement_ship_sink (entity *en, float delta_seconds)
{
	vehicle *raw = (vehicle *) get_local_entity_data (en);
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_SHIP_SINK);
	fill_vehicle_vector (&step, raw);
	step.delta_seconds = delta_seconds;

	result = ask_lifecycle (&step, FALSE);

	apply_vehicle_vector (raw, result.motion_vector);
	apply_speed (en, result.velocity);
	memcpy (raw->mob.attitude, result.attitude, sizeof (result.attitude));
	apply_position (en, result.position);

	return result.position[1];
}

// EEDCS-38: the lifetime of a dead mobile.  The campaign passes its mirrored timer and its policy; the host accumulates the
// timer, applies the gate and the range-to-lifetime curve, and says whether the wreck has expired.  The host's timer is written
// back to the mirror.  The squared range to the nearest keysite is a full keysite traversal, which the original made only on a
// gated frame: the host answers 2 when the gate is open and it needs the range, and the campaign then samples it once and
// asks again with the range supplied.  The explicit campaign removal request stays with the caller

int campaign_movement_expire (entity *en, float *death_timer, float delta_seconds, int policy)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_EXPIRE);
	step.death_timer = *death_timer;
	step.death_policy = policy;
	step.delta_seconds = delta_seconds;

	result = ask_lifecycle (&step, FALSE);

	if (result.flags == CAMPAIGN_MOVEMENT_EXPIRE_NEEDS_RANGE)
	{
		step.death_range = calculate_mobile_nearest_keysite_sqr_range (en);
		step.death_policy = policy | CAMPAIGN_MOVEMENT_DEATH_RANGE_SUPPLIED;

		result = ask_lifecycle (&step, FALSE);
	}

	*death_timer = result.death_timer;

	return (result.flags & 1) != 0;
}

// a ship's sink test: the host's height (after campaign_movement_ship_sink) against the sight of the ship's shape

int campaign_movement_ship_sunk (entity *en, float bbox_diagonal)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, en, CAMPAIGN_MOVEMENT_EXPIRE);
	step.gv_bbox_diagonal = bbox_diagonal;

	result = ask_lifecycle (&step, FALSE);

	return (result.flags & 1) != 0;
}

// the final pose of an infantry step, as the legacy step wrote it: attitude, position through the setter, speed, motion vector

static void apply_person_pose (entity *vehicle, const campaign_movement_result *result)
{
	person *raw = (person *) get_local_entity_data (vehicle);
	vec3d new_pos;

	memcpy (raw->vh.mob.attitude, result->attitude, sizeof (result->attitude));

	new_pos.x = result->position[0];
	new_pos.y = result->position[1];
	new_pos.z = result->position[2];

	campaign_movement_apply_begin ();
	set_local_entity_vec3d (vehicle, VEC3D_TYPE_POSITION, &new_pos);
	campaign_movement_apply_end ();

	raw->vh.mob.velocity = result->velocity;
	raw->vh.mob.motion_vector.x = result->motion_vector[0];
	raw->vh.mob.motion_vector.y = result->motion_vector[1];
	raw->vh.mob.motion_vector.z = result->motion_vector[2];
}

int campaign_movement_person_begin (entity *vehicle, float ps_vel, const vec3d *waypoint, int terrain_follow, vec3d *advanced)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, vehicle, CAMPAIGN_MOVEMENT_PERSON_BEGIN);
	step.delta_seconds = get_entity_movement_delta_time ();
	step.ps_vel = ps_vel;
	step.waypoint[0] = waypoint->x;
	step.waypoint[1] = waypoint->y;
	step.waypoint[2] = waypoint->z;
	step.terrain_follow = terrain_follow ? TRUE : FALSE;

	result = ask_host (&step);

	if (result.pending != step.terrain_follow)
	{
		debug_fatal ("host movement: host move_vehicle answered pending %d for an infantry step with terrain follow %d (%s)", result.pending, step.terrain_follow, step.host_id);
	}

	advanced->x = result.position[0];
	advanced->y = result.position[1];
	advanced->z = result.position[2];

	if (!result.pending)
	{
		apply_person_pose (vehicle, &result);
	}

	return result.pending;
}

void campaign_movement_person_end (entity *vehicle, float elevation)
{
	campaign_movement_step step;
	campaign_movement_result result;

	begin_step (&step, vehicle, CAMPAIGN_MOVEMENT_PERSON_END);
	step.elevation = elevation;

	result = ask_host (&step);

	if (result.pending)
	{
		debug_fatal ("host movement: host move_vehicle left the infantry step of %s pending after its terrain sample", step.host_id);
	}

	apply_person_pose (vehicle, &result);
}
