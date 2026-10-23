/*
 * The host's physical step for an alive fixed-wing aircraft.
 *
 * The arithmetic is EECH's own, kept expression for expression so that the
 * host and the original reference produce the same bits: fw_move.c
 * (basic_fixed_wing_movement and the functions it calls), with the shared
 * helpers in eech_arith.h.  The campaign decides everything that is guidance:
 * whether the aircraft moves at all, which sub-mode applies (taxi, flight,
 * landing, or the absolute turn-around/aiming flight), the waypoint, the guide's
 * speed, the navigator's acceleration vector (collision avoidance, pursuit,
 * waypoint following) and the cruise/turn parameters.  This file performs the
 * physical step those decisions ask for: velocity integration with its limits,
 * position, attitude, the motion vector, the map bound and the terrain clamp.
 * The engine's step delta (`delta_seconds`) and the frame delta (`frame_delta`,
 * a reciprocal used for the motion vector) are different values and stay
 * separate.
 *
 * `motion_vector` is both an input (the previous step's, which the flight
 * attitude and the navigator follow) and the output.  The legacy absolute
 * flight's ground branch is unreachable (it is only entered airborne) and is
 * not carried over.
 */
#include "eech_arith.h"

#define PHASE_TAXI 0
#define PHASE_FLIGHT 1
#define PHASE_LAND 2
#define PHASE_ABSOLUTE 3

#define GROUND_ACCELERATION 3.0f
#define AIR_ACCELERATION 30.0f
#define PITCH_MIN -30.0f
#define PITCH_MAX 30.0f
#define AIRCRAFT_MAX_FLARE_ANGLE rad(15.0f)
#define AIRCRAFT_MAX_LANDING_FLARE_ANGLE rad(4.0f)

static void bound_vector_scale(vec3d *vector, float min_scale, float max_scale)
{
	float distance;

	distance = get_3d_vector_magnitude(vector);

	if (distance > max_scale)
	{
		normalise_3d_vector_given_magnitude(vector, distance);

		vector->x *= max_scale;
		vector->y *= max_scale;
		vector->z *= max_scale;
	}
	else if (distance < min_scale)
	{
		normalise_3d_vector_given_magnitude(vector, distance);

		vector->x *= min_scale;
		vector->y *= min_scale;
		vector->z *= min_scale;
	}
}

void eech_fw_step(
	int phase,
	float *position,
	float *velocity_io,
	float *attitude,
	float delta_seconds,
	float frame_delta,
	float wp_vel,
	float cruise_velocity,
	float g_max,
	float fuselage_angle,
	float cg_to_ground,
	float max_roll,
	float turn_speed_multi,
	const float *waypoint,
	const float *acceleration,
	float *motion_vector,
	int navigating,
	float elevation,
	const float *map_min,
	const float *map_max,
	int tacview_logging,
	int *rotated)
{
	vec3d current_pos, new_pos;
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	float velocity = *velocity_io;
	float old_roll = 0.0, old_pitch = 0.0, old_heading = 0.0;
	float terrain_elevation = elevation;

	current_pos.x = position[0];
	current_pos.y = position[1];
	current_pos.z = position[2];

	*rotated = 0;

	if (tacview_logging)
	{
		old_roll = get_roll_from_attitude_matrix(*matrix);
		old_pitch = get_pitch_from_attitude_matrix(*matrix);
		old_heading = get_heading_from_attitude_matrix(*matrix);
	}

	if (phase == PHASE_TAXI)
	{
		vec3d wp_pos, wp_vec, dist;
		float desired_velocity, current_heading, desired_heading, delta_heading, new_heading, current_pitch, new_pitch;

		wp_pos.x = waypoint[0];
		wp_pos.y = waypoint[1];
		wp_pos.z = waypoint[2];

		wp_vec.x = wp_pos.x - current_pos.x;
		wp_vec.y = wp_pos.y - (current_pos.y - cg_to_ground);
		wp_vec.z = wp_pos.z - current_pos.z;

		desired_velocity = wp_vel;

		velocity += min((desired_velocity - velocity), GROUND_ACCELERATION) * delta_seconds;

		current_heading = get_heading_from_attitude_matrix(*matrix);

		desired_heading = atan2(wp_vec.x, wp_vec.z);

		delta_heading = desired_heading - current_heading;

		if (delta_heading <= -PI)
		{
			delta_heading += PI2;
		}

		if (delta_heading >= PI)
		{
			delta_heading -= PI2;
		}

		delta_heading = bound(delta_heading, rad(-30.0), rad(30.0));

		new_heading = current_heading + (delta_heading * delta_seconds);

		current_pitch = get_pitch_from_attitude_matrix(*matrix);

		if (current_pitch > fuselage_angle)
		{
			new_pitch = current_pitch + ((fuselage_angle - current_pitch) * delta_seconds);
		}
		else
		{
			new_pitch = fuselage_angle;
		}

		get_3d_transformation_matrix(*matrix, new_heading, new_pitch, 0.0);

		dist.x = min(velocity * (*matrix)[2][0] * delta_seconds, fabs(wp_vec.x));
		dist.y = 0.0;
		dist.z = min(velocity * (*matrix)[2][2] * delta_seconds, fabs(wp_vec.z));

		new_pos.x = current_pos.x + dist.x;
		new_pos.y = current_pos.y + dist.y;
		new_pos.z = current_pos.z + dist.z;

		bound_position_to_map_volume(&new_pos, map_min, map_max);

		new_pos.y = terrain_elevation + cg_to_ground;

		motion_vector[0] = velocity * (*matrix)[2][0];
		motion_vector[1] = 0.0;
		motion_vector[2] = velocity * (*matrix)[2][2];
	}
	else if (phase == PHASE_FLIGHT)
	{
		vec3d acc, vel, norm_mv;
		float min_speed, max_speed, acc_avail, d;
		float pitch, calc_old_heading, heading, change_in_heading, roll, factor;

		acc.x = acceleration[0];
		acc.y = acceleration[1];
		acc.z = acceleration[2];

		vel.x = motion_vector[0];
		vel.y = motion_vector[1];
		vel.z = motion_vector[2];

		/* fixed_wing_set_velocity */
		min_speed = max_speed = cruise_velocity;

		min_speed *= 0.7;

		max_speed *= 2.0;

		acc_avail = 1.0;

		d = get_3d_vector_magnitude(&acc);

		if (d > g_max / 10)
		{
			normalise_3d_vector_given_magnitude(&acc, d / (g_max / 10));
		}

		vel.x += acc.x * acc_avail * delta_seconds;
		vel.y += acc.y * acc_avail * delta_seconds;
		vel.z += acc.z * acc_avail * delta_seconds;

		bound_vector_scale(&vel, min_speed, max_speed);

		velocity = get_3d_vector_magnitude(&vel);

		new_pos.x = current_pos.x + vel.x * delta_seconds;
		new_pos.y = current_pos.y + vel.y * delta_seconds;
		new_pos.z = current_pos.z + vel.z * delta_seconds;

		/* fixed_wing_calc_attitude: follows the previous step's motion vector */
		calc_old_heading = get_heading_from_attitude_matrix(*matrix);

		roll = get_roll_from_attitude_matrix(*matrix);

		norm_mv.x = motion_vector[0];
		norm_mv.y = motion_vector[1];
		norm_mv.z = motion_vector[2];

		normalise_any_3d_vector(&norm_mv);

		heading = atan2(norm_mv.x, norm_mv.z);

		pitch = asin(norm_mv.y);

		change_in_heading = heading - calc_old_heading;

		if (change_in_heading > PI)
		{
			change_in_heading -= PI2;
		}
		else if (change_in_heading < (-PI))
		{
			change_in_heading += PI2;
		}

		factor = max_roll * 0.01;

		if (fabs(change_in_heading) >= factor)
		{
			roll += change_in_heading;
		}
		else
		{
			factor = (factor - fabs(change_in_heading)) / factor;

			roll -= (roll * factor * delta_seconds);

			roll += (change_in_heading * (1.0 - factor));
		}

		if (max_roll >= PI)
		{
			if (roll > PI)
			{
				roll -= PI2;
			}
			else if (roll < (-PI))
			{
				roll += PI2;
			}
		}
		else
		{
			roll = bound(roll, -max_roll, max_roll);
		}

		get_3d_transformation_matrix(*matrix, heading, pitch, roll);

		/* the motion vector is calculated BEFORE the bound, otherwise it may be zeroed */
		motion_vector[0] = (new_pos.x - current_pos.x) * frame_delta;
		motion_vector[1] = (new_pos.y - current_pos.y) * frame_delta;
		motion_vector[2] = (new_pos.z - current_pos.z) * frame_delta;
	}
	else if (phase == PHASE_LAND)
	{
		vec3d wp_pos, wp_vec;
		float factor, range, delta_vel, heading, pitch, roll, desired_heading, desired_pitch, delta_heading, delta_pitch, delta_roll;

		wp_pos.x = waypoint[0];
		wp_pos.y = waypoint[1];
		wp_pos.z = waypoint[2];

		desired_pitch = 0.0;

		wp_vec.x = wp_pos.x - current_pos.x;
		wp_vec.y = wp_pos.y - current_pos.y;
		wp_vec.z = wp_pos.z - current_pos.z;

		range = get_3d_vector_magnitude(&wp_vec);

		normalise_any_3d_vector(&wp_vec);

		heading = get_heading_from_attitude_matrix(*matrix);

		pitch = get_pitch_from_attitude_matrix(*matrix);

		roll = get_roll_from_attitude_matrix(*matrix);

		desired_heading = atan2(wp_vec.x, wp_vec.z);

		if (range < 100.0)
		{
			desired_pitch = AIRCRAFT_MAX_LANDING_FLARE_ANGLE;
		}
		else if (range < 1000.0)
		{
			factor = (1000.0 - range) * 0.001;

			desired_pitch = AIRCRAFT_MAX_FLARE_ANGLE * factor;
		}
		else
		{
			desired_pitch = asin(wp_vec.y);
		}

		delta_heading = (desired_heading - heading) * delta_seconds;

		delta_pitch = (desired_pitch - pitch) * delta_seconds;

		delta_roll = -roll * delta_seconds;

		heading += delta_heading;

		pitch += delta_pitch;

		roll += delta_roll;

		get_3d_transformation_matrix(*matrix, heading, pitch, roll);

		delta_vel = (wp_vel - velocity);

		velocity += delta_vel;

		wp_vec.x *= velocity;
		wp_vec.y *= velocity;
		wp_vec.z *= velocity;

		new_pos.x = current_pos.x + wp_vec.x * delta_seconds;
		new_pos.y = current_pos.y + wp_vec.y * delta_seconds;
		new_pos.z = current_pos.z + wp_vec.z * delta_seconds;

		motion_vector[0] = (new_pos.x - current_pos.x) * frame_delta;
		motion_vector[1] = (new_pos.y - current_pos.y) * frame_delta;
		motion_vector[2] = (new_pos.z - current_pos.z) * frame_delta;
	}
	else
	{
		/* basic_fixed_wing_movement_absolute, airborne */
		vec3d wp_pos, wp_vec, normalised_wp_vec, dist;
		float required_heading, required_pitch, required_roll, delta_heading, delta_pitch, heading, pitch, roll, max_turn_rate;

		wp_pos.x = waypoint[0];
		wp_pos.y = waypoint[1];
		wp_pos.z = waypoint[2];

		wp_vec.x = wp_pos.x - current_pos.x;
		wp_vec.y = wp_pos.y - (current_pos.y - cg_to_ground);
		wp_vec.z = wp_pos.z - current_pos.z;

		velocity += min(((wp_vel / turn_speed_multi) - velocity), AIR_ACCELERATION) * delta_seconds;

		normalised_wp_vec = wp_vec;

		normalise_any_3d_vector(&normalised_wp_vec);

		/* get_heading_and_pitch_from_3d_unit_vector */
		required_heading = atan2(normalised_wp_vec.x, normalised_wp_vec.z);
		required_pitch = asin(normalised_wp_vec.y);

		heading = get_heading_from_attitude_matrix(*matrix);

		pitch = get_pitch_from_attitude_matrix(*matrix);

		roll = get_roll_from_attitude_matrix(*matrix);

		if (velocity)
		{
			max_turn_rate = g_max / velocity;
		}
		else
		{
			max_turn_rate = g_max;
		}

		delta_heading = (required_heading - heading);

		if (delta_heading <= -PI)
		{
			delta_heading += PI2;
		}

		if (delta_heading >= PI)
		{
			delta_heading -= PI2;
		}

		delta_heading = bound(delta_heading, -max_turn_rate, max_turn_rate);

		/* the position follows the attitude's z axis from BEFORE this step's turn */
		dist.x = velocity * (*matrix)[2][0] * delta_seconds;
		dist.y = velocity * (*matrix)[2][1] * delta_seconds;
		dist.z = velocity * (*matrix)[2][2] * delta_seconds;

		new_pos.x = current_pos.x + dist.x;
		new_pos.y = current_pos.y + dist.y;
		new_pos.z = current_pos.z + dist.z;

		heading += delta_heading * delta_seconds;

		delta_pitch = (required_pitch - pitch);

		delta_pitch = bound(delta_pitch, PITCH_MIN, PITCH_MAX);

		pitch += delta_pitch * delta_seconds;

		required_roll = (delta_heading / max_turn_rate) * rad(70);

		roll += (required_roll - roll) * delta_seconds;

		get_3d_transformation_matrix(*matrix, heading, pitch, roll);

		motion_vector[0] = (new_pos.x - current_pos.x) * frame_delta;
		motion_vector[1] = (new_pos.y - current_pos.y) * frame_delta;
		motion_vector[2] = (new_pos.z - current_pos.z) * frame_delta;
	}

	bound_position_to_map_volume(&new_pos, map_min, map_max);

	if (navigating)
	{
		if (new_pos.y < (terrain_elevation + 100.0))
		{
			new_pos.y = terrain_elevation + 100.0;
		}
	}
	else
	{
		if (new_pos.y < terrain_elevation)
		{
			new_pos.y = terrain_elevation;
		}
	}

	/* the absolute flight never reported a rotation */
	if (tacview_logging && phase != PHASE_ABSOLUTE)
	{
		if (fabs(get_roll_from_attitude_matrix(*matrix) - old_roll) > rad(0.001)
			|| fabs(get_pitch_from_attitude_matrix(*matrix) - old_pitch) > rad(0.001)
			|| fabs(get_heading_from_attitude_matrix(*matrix) - old_heading) > rad(0.001))
		{
			*rotated = 1;
		}
	}

	position[0] = new_pos.x;
	position[1] = new_pos.y;
	position[2] = new_pos.z;
	*velocity_io = velocity;
}

/*
 * The host's fixed-wing navigator: fw_move.c's fixed_wing_navigator and the
 * functions it calls (collision avoidance, pursue guide, follow waypoint,
 * attain waypoint, aim correction, accum), kept expression for expression,
 * including float/double promotions, vector mutation and evaluation order.
 * Inputs are the plain values in fw_navigator.h.  Preserved as the engine has
 * them: the task leader's follow_waypoint overwrites the acceleration collision
 * avoidance accumulated, and the second accum reuses the acc_req the first accum
 * normalised.
 */
#include "fw_navigator.h"

#define check_zero_3d_vector(VECTOR) (get_3d_vector_magnitude(VECTOR) == 0.0)

static void load_vec3d(vec3d *vector, const float *values)
{
	vector->x = values[0];
	vector->y = values[1];
	vector->z = values[2];
}

static void fixed_wing_collision_avoidance(const eech_fw_nav_input *in, vec3d *des_acc)
{
	matrix3x3 wp_attitude;
	vec3d curr_pos, jet_pos, jet_to_jet, nearest_jet_to_jet, local_jet_to_jet, wp_pos, wp_vec, motion_vector, jet_motion_vector;
	float test_distance, shortest_distance, jet_to_jet_2d_distance, avoid_distance, max_speed, wp_distance;
	float delta_time = in->delta_time;
	int nearest = 0;
	int index;

	/* only read when a nearest jet was found, which needs the non-leader branch below */
	memset(wp_attitude, 0, sizeof(wp_attitude));
	nearest_jet_to_jet.x = nearest_jet_to_jet.y = nearest_jet_to_jet.z = 0.0;

	if (!in->group_leader)
	{
		load_vec3d(&curr_pos, in->position);
		load_vec3d(&motion_vector, in->motion_vector);

		shortest_distance = in->velocity;

		load_vec3d(&wp_pos, in->waypoint);

		wp_vec.x = wp_pos.x - curr_pos.x;
		wp_vec.y = wp_pos.y - curr_pos.y;
		wp_vec.z = wp_pos.z - curr_pos.z;

		wp_distance = get_3d_vector_magnitude(&wp_vec);

		normalise_any_3d_vector(&wp_vec);

		get_matrix3x3_from_unit_vec3d(wp_attitude, &wp_vec);

		/* find nearest jet to ours that is in the way of our objective */
		for (index = 0; index < in->other_count; index++)
		{
			const eech_fw_nav_other *other = &in->others[index];

			load_vec3d(&jet_pos, other->position);
			load_vec3d(&jet_motion_vector, other->motion_vector);

			jet_to_jet.x = (jet_pos.x + jet_motion_vector.x * delta_time) - (curr_pos.x + motion_vector.x * delta_time);
			jet_to_jet.y = (jet_pos.y + jet_motion_vector.y * delta_time) - (curr_pos.y + motion_vector.y * delta_time);
			jet_to_jet.z = (jet_pos.z + jet_motion_vector.z * delta_time) - (curr_pos.z + motion_vector.z * delta_time);

			multiply_transpose_matrix3x3_vec3d(&local_jet_to_jet, wp_attitude, &jet_to_jet);

			test_distance = get_3d_vector_magnitude(&jet_to_jet);

			/* is the jet between us and our waypoint? */
			if ((local_jet_to_jet.z > 0.0) && (test_distance < wp_distance))
			{
				/* get 2d distance & collision avoidance radius */
				jet_to_jet_2d_distance = (local_jet_to_jet.x * local_jet_to_jet.x + local_jet_to_jet.y * local_jet_to_jet.y);

				avoid_distance = in->avoidance_radius + other->avoidance_radius;

				/* does the jet get in our way? */
				if (jet_to_jet_2d_distance < (avoid_distance * avoid_distance))
				{
					/* is it the nearest jet in our way? */
					if (test_distance < shortest_distance)
					{
						nearest = 1;

						nearest_jet_to_jet = local_jet_to_jet;

						shortest_distance = test_distance;
					}
				}
			}
		}
	}

	/* if a jet is in the way, modify our direction */
	if (nearest)
	{
		max_speed = in->cruise_velocity * 2.0;

		/* get local acc. request */
		nearest_jet_to_jet.x = -nearest_jet_to_jet.x;
		nearest_jet_to_jet.y = -nearest_jet_to_jet.y;
		nearest_jet_to_jet.z = 0.0;

		normalise_any_3d_vector(&nearest_jet_to_jet);

		/* rotate back into world coord sys */
		multiply_matrix3x3_vec3d(des_acc, wp_attitude, &nearest_jet_to_jet);

		des_acc->x *= max_speed;
		des_acc->y *= max_speed;
		des_acc->z *= max_speed;
	}
	else
	{
		/* set desired acceleration to nothing */
		des_acc->z = des_acc->y = des_acc->x = 0.0;
	}
}

static void fixed_wing_attain_waypoint(const eech_fw_nav_input *in, vec3d *wp_vec, const vec3d *vel)
{
	float distance_to_wp, max_turn_radius;

	distance_to_wp = get_3d_vector_magnitude(wp_vec);

	max_turn_radius = (in->velocity * in->velocity) / in->g_max;

	/* test to see if waypoint is attainable */
	if (distance_to_wp < max_turn_radius)
	{
		/* continue along current motion vector until waypoint can be reached */
		*wp_vec = *vel;
	}
}

static void fixed_wing_aim_correction(const eech_fw_nav_input *in, vec3d *wp_vec, const vec3d *vel)
{
	matrix3x3 attitude;
	vec3d dir_vec;

	memcpy(attitude, in->attitude, sizeof(attitude));

	if (!check_zero_3d_vector(vel))
	{
		if ((get_3d_vector_dot_product(wp_vec, vel) < 0.0))
		{
			multiply_transpose_matrix3x3_vec3d(&dir_vec, attitude, wp_vec);

			if (fabs(dir_vec.y) > fabs(dir_vec.z))
			{
				dir_vec.y = fabs(dir_vec.y);
			}
			else
			{
				dir_vec.y = fabs(dir_vec.z);
			}

			dir_vec.y *= in->max_vertical_turn_allowance;

			dir_vec.z = 0.0;

			normalise_any_3d_vector(&dir_vec);

			multiply_matrix3x3_vec3d(wp_vec, attitude, &dir_vec);
		}
	}
}

static void fixed_wing_follow_waypoint(const eech_fw_nav_input *in, const vec3d *vel, vec3d *des_acc)
{
	vec3d curr_pos, wp_pos, wp_vec;
	float wp_vel;

	load_vec3d(&wp_pos, in->waypoint);

	load_vec3d(&curr_pos, in->position);

	wp_vel = in->guide_velocity;

	if (wp_vel == 0)
	{
		wp_vel = in->cruise_velocity;
	}

	/* get waypoint vector - no lookahead */
	wp_vec.x = wp_pos.x - curr_pos.x;
	wp_vec.y = wp_pos.y - curr_pos.y;
	wp_vec.z = wp_pos.z - curr_pos.z;

	fixed_wing_attain_waypoint(in, &wp_vec, vel);

	/* scale to correct length - that of waypoint velocity */
	normalise_any_3d_vector(&wp_vec);

	fixed_wing_aim_correction(in, &wp_vec, vel);

	wp_vec.x *= wp_vel;
	wp_vec.y *= wp_vel;
	wp_vec.z *= wp_vel;

	/* calculate desired change in velocity */
	des_acc->x = wp_vec.x - vel->x;
	des_acc->y = wp_vec.y - vel->y;
	des_acc->z = wp_vec.z - vel->z;
}

static void fixed_wing_pursue_guide(const eech_fw_nav_input *in, const vec3d *vel, vec3d *des_acc)
{
	vec3d curr_pos, wp_pos, wp_vec, vel_shift;
	float slowdown_distance, distance, wp_speed, max_speed, sd_shift, ramped_speed;
	float delta_time = in->delta_time;

	wp_speed = in->guide_velocity;

	if (wp_speed == 0)
	{
		wp_speed = in->cruise_velocity;
	}

	load_vec3d(&wp_pos, in->waypoint);

	load_vec3d(&curr_pos, in->position);

	max_speed = in->cruise_velocity * 2.0;

	slowdown_distance = max_speed * 1.5;

	/* shift slowdown radius forwards so point of velocity equilibrium is coincident with formation position required */
	sd_shift = (wp_speed * 1.5);

	load_vec3d(&vel_shift, in->task_leader_motion_vector);

	normalise_any_3d_vector(&vel_shift);

	/* get vector to the offset waypoint */
	wp_vec.x = (wp_pos.x + vel_shift.x * sd_shift) - (curr_pos.x + vel->x * delta_time);
	wp_vec.y = (wp_pos.y + vel_shift.y * sd_shift) - (curr_pos.y + vel->y * delta_time);
	wp_vec.z = (wp_pos.z + vel_shift.z * sd_shift) - (curr_pos.z + vel->z * delta_time);

	distance = get_3d_vector_magnitude(&wp_vec);

	if (distance > 0.0)
	{
		/* ramp the speed linearly to zero from slowdown distance to formation point */
		ramped_speed = max_speed * (distance / slowdown_distance);

		ramped_speed = bound(ramped_speed, -max_speed, max_speed);

		normalise_3d_vector_given_magnitude(&wp_vec, distance);

		fixed_wing_aim_correction(in, &wp_vec, vel);

		wp_vec.x *= ramped_speed;
		wp_vec.y *= ramped_speed;
		wp_vec.z *= ramped_speed;

		/* calculate desired change in velocity */
		des_acc->x = wp_vec.x - vel->x;
		des_acc->y = wp_vec.y - vel->y;
		des_acc->z = wp_vec.z - vel->z;
	}
	else
	{
		des_acc->x = -vel->x;
		des_acc->y = -vel->y;
		des_acc->z = -vel->z;
	}
}

/* accumulates all acceleration vectors, works on a first come first served basis */
static float fixed_wing_accum(const eech_fw_nav_input *in, vec3d *accumulator, vec3d *new_acc)
{
	float acc_avail, vm, new_mag, max_acc;

	max_acc = in->cruise_velocity * 2.0;

	acc_avail = max_acc - get_3d_vector_magnitude(accumulator);

	vm = get_3d_vector_magnitude(new_acc);

	new_mag = vm < acc_avail ? vm : acc_avail;

	/* scale newacc by new_mag */
	normalise_any_3d_vector(new_acc);

	accumulator->x += new_acc->x * new_mag;
	accumulator->y += new_acc->y * new_mag;
	accumulator->z += new_acc->z * new_mag;

	return new_mag;
}

void eech_fw_navigate(const eech_fw_nav_input *input, float *acceleration)
{
	vec3d acc_req, acc, vel;

	load_vec3d(&vel, input->navigation_velocity);

	acc_req.x = acc_req.y = acc_req.z = 0.0;

	acc.x = acc.y = acc.z = 0.0;

	if (!input->task_leader)
	{
		fixed_wing_collision_avoidance(input, &acc_req);

		fixed_wing_accum(input, &acc, &acc_req);

		fixed_wing_pursue_guide(input, &vel, &acc_req);

		fixed_wing_accum(input, &acc, &acc_req);
	}
	else
	{
		fixed_wing_collision_avoidance(input, &acc_req);

		fixed_wing_accum(input, &acc, &acc_req);

		fixed_wing_follow_waypoint(input, &vel, &acc);

		fixed_wing_accum(input, &acc, &acc_req);
	}

	acceleration[0] = acc.x;
	acceleration[1] = acc.y;
	acceleration[2] = acc.z;
}

/*
 * basic_fixed_wing_movement, airborne: the cannon-aim and facing-away decisions
 * that choose the absolute turn-around flight, in the legacy order and with the
 * legacy float/double promotions.  The turn multiplier and the hold-until time
 * are retained by the caller between steps.
 */
int eech_fw_facing_away(const float *position, const float *motion_vector, const float *waypoint)
{
	vec3d wpvector, mvvector;

	wpvector.x = waypoint[0] - position[0];
	wpvector.y = waypoint[1] - position[1];
	wpvector.z = waypoint[2] - position[2];

	load_vec3d(&mvvector, motion_vector);

	return get_3d_vector_dot_product(&wpvector, &mvvector) < 0.0;
}

int eech_fw_turn(const eech_fw_turn_input *in, float *turn_speed_multi, float *fixed_wing_toggle)
{
	if (in->aiming_cannon)
	{
		*turn_speed_multi = 1.0;
		*fixed_wing_toggle = in->elapsed_time + 0.250;
	}

	if (eech_fw_facing_away(in->position, in->motion_vector, in->waypoint))
	{
		if (!in->task_leader)
		{
			vec3d leader_dist;

			leader_dist.x = in->leader_position[0] - in->position[0];
			leader_dist.y = in->leader_position[1] - in->position[1];
			leader_dist.z = in->leader_position[2] - in->position[2];

			if (fabs(get_3d_vector_magnitude(&leader_dist)) > 1500.0)
			{
				*fixed_wing_toggle = in->elapsed_time + 3.0;
				*turn_speed_multi = 2.0;
			}
		}
		else
		{
			*fixed_wing_toggle = in->elapsed_time + 3.0;
			*turn_speed_multi = 2.0;
		}
	}

	return *fixed_wing_toggle > in->elapsed_time;
}

void eech_fw_lookahead(const float *position, float velocity, const float *attitude, float *point_xz)
{
	/* the attitude's third row is the aircraft's zv */
	point_xz[0] = position[0] + ((velocity * 1.0 + 10.0) * attitude[6]);
	point_xz[1] = position[2] + ((velocity * 1.0 + 10.0) * attitude[8]);
}

/*
 * mobile_terrain_adjust_waypoint_height, after its terrain sample, for an aircraft that is not a helicopter.  The
 * original wrote into the waypoint it was handed; the height it leaves is returned, and the guide's own is returned
 * where the original left the waypoint alone.
 */
float eech_fw_terrain_height(
	const float *position,
	float velocity,
	const float *attitude,
	const float *guide_position,
	float terrain_elevation,
	float min_height)
{
	float range, height, xz[2];
	vec3d test_point, wp_vec, vector;

	height = guide_position[1];

	eech_fw_lookahead(position, velocity, attitude, xz);
	test_point.x = xz[0];
	test_point.z = xz[1];

	/* check waypoint SHOULD be adjusted */
	if (min_height == -1.0)
	{
		return height;
	}

	wp_vec.x = guide_position[0] - position[0];
	wp_vec.y = 0.0;
	wp_vec.z = guide_position[2] - position[2];

	range = sqrt((wp_vec.x * wp_vec.x) + (wp_vec.z * wp_vec.z));

	if (range <= 1.0)
	{
		return terrain_elevation + min_height;
	}

	test_point.y = terrain_elevation + min_height;

	vector.x = test_point.x - position[0];
	vector.y = test_point.y - position[1];
	vector.z = test_point.z - position[2];

	if (normalise_any_3d_vector(&vector) != 0.0)
	{
		float d, h;

		h = vector.y;

		d = sqrt((vector.x * vector.x) + (vector.z * vector.z));

		/* too little lateral velocity (jets suspended themselves pointing down into the ground) */
		if (d > fabs(h))
		{
			height = position[1] + (range * h / d);
		}
		else
		{
			height = terrain_elevation + min_height;
		}
	}

	return height;
}
