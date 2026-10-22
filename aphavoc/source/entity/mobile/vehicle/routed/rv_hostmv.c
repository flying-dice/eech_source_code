#include "project.h"

#include <math.h>
#include <string.h>

#include "rv_hostmv.h"

static campaign_movement_host current_host;
static int applying_host_result;

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
	if (current_host.move_vehicle (step, &result, error, CAMPAIGN_MOVEMENT_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host movement: host move_vehicle failed for %s: %s", step->host_id, error[0] ? error : "unspecified error");
	}
	if (!all_finite (result.position, 3) || !all_finite (&result.velocity, 1) || !all_finite (result.attitude, 9) ||
		!all_finite (result.motion_vector, 3) ||
		((result.rotated != FALSE) && (result.rotated != TRUE)) ||
		((result.pending != FALSE) && (result.pending != TRUE)))
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
	if ((type != ENTITY_TYPE_ROUTED_VEHICLE) && (type != ENTITY_TYPE_SHIP_VEHICLE) && (type != ENTITY_TYPE_PERSON) && (type != ENTITY_TYPE_FIXED_WING))
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
	memcpy (step.fw_acceleration, guidance->acceleration, sizeof (step.fw_acceleration));
	memcpy (step.fw_motion_vector, guidance->motion_vector, sizeof (step.fw_motion_vector));
	step.fw_cruise_velocity = guidance->cruise_velocity;
	step.fw_fuselage_angle = guidance->fuselage_angle;
	step.fw_cg_to_ground = guidance->cg_to_ground;
	step.fw_max_roll = guidance->max_roll;
	step.fw_turn_speed_multi = guidance->turn_speed_multi;
	step.fw_navigating = guidance->navigating ? TRUE : FALSE;

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
