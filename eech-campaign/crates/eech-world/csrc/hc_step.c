/*
 * The physical half of an alive AI helicopter's step (helicopter_movement, helicop/hc_move.c), in two phases around
 * the campaign's guidance.  Every expression is kept from the engine so the host and the original reference produce the
 * same bits.  The DLL supplies guidance inputs (the world acceleration and the climb demand, the desired heading, pitch
 * and roll, the turn-rate limit, the terrain elevation under the aircraft); the heading wrap, the turn-rate bound, the
 * attitude blend, the position integration and the floor clamp are the host's physical integration.
 *
 * Also here: the rotor and spin-up integration, and the physical arithmetic of the dying, crash and impact movement
 * (helicopter_movement_calculate_rotor_rpm, helicopter_death_movement, helicopter_crash_movement and
 * helicopter_impact_movement in hc_move.c).  The DLL keeps the order of the calls, the terrain samples (which refresh
 * its cache), the effects, sounds, craters, the random draws, the death state and every campaign decision.
 *
 * Not here, and still campaign-side coupling: terrain look-ahead, the desired pitch/roll/heading guidance, formation and
 * avoidance, pilot control, terrain sampling, effects and targeting.
 */
#include "eech_arith.h"

/*
 * Phase one: the motion vector, in world coordinates, then the speed it implies.
 * `acceleration` is the world acceleration's x and z (the campaign's decision), `wp_vec_y` the climb demand.
 */
void eech_hc_begin(float *motion_vector, float acceleration_x, float acceleration_z, float wp_vec_y, float delta_seconds, float *speed, int *moved)
{
	vec3d *mv = (vec3d *)motion_vector;
	float s;

	mv->x += acceleration_x * delta_seconds;
	mv->y = wp_vec_y;
	mv->z += acceleration_z * delta_seconds;

	s = get_3d_vector_magnitude(mv);

	*speed = s;
	*moved = (fabs(s) > 0.1) ? 1 : 0;
}

/*
 * Phase two: the attitude, then the position integrated with the motion vector, the map bound and the floor clamp.
 * `desired_heading` is the guidance heading, wrapped here to the shortest turn from the attitude's heading and bound to
 * `max_turn_rate`, the aircraft's physical turn limit (the engine's order: wrap, then bound, then integrate).  The
 * attitude is the one the campaign's guidance read, so the heading is the same one the DLL passed `desired_heading`
 * against.  The floor clamp may change the motion vector's y after the speed was computed; the speed is not recomputed.
 */
void eech_hc_end(float *position, float *motion_vector, float *attitude, float desired_heading, float max_turn_rate, float desired_pitch, float desired_roll, float delta_seconds, float elevation, float cg_to_ground, int terrain_follow, const float *map_min, const float *map_max, int *rotated)
{
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	vec3d *pos = (vec3d *)position;
	vec3d *mv = (vec3d *)motion_vector;
	float current_heading, current_pitch, current_roll;
	float delta_heading;
	float new_heading, new_pitch, new_roll;
	float min_height;

	current_heading = get_heading_from_attitude_matrix(*matrix);
	current_pitch = get_pitch_from_attitude_matrix(*matrix);
	current_roll = get_roll_from_attitude_matrix(*matrix);

	delta_heading = (desired_heading - current_heading);

	if (delta_heading < -PI)
	{
		delta_heading += PI2;
	}
	else if (delta_heading > PI)
	{
		delta_heading -= PI2;
	}

	delta_heading = bound(delta_heading, -max_turn_rate, max_turn_rate);

	new_heading = current_heading + (delta_heading * delta_seconds);

	new_pitch = current_pitch + (desired_pitch - current_pitch) * 0.5 * delta_seconds;
	new_roll = current_roll + (desired_roll - current_roll) * 0.5 * delta_seconds;

	*rotated = (fabs(new_pitch - current_pitch) > rad(0.1) || fabs(new_heading - current_heading) > rad(0.1) || fabs(new_roll - current_roll) > rad(0.1)) ? 1 : 0;

	get_3d_transformation_matrix(*matrix, new_heading, new_pitch, new_roll);

	pos->x += (mv->x * delta_seconds);
	pos->y += (mv->y * delta_seconds);
	pos->z += (mv->z * delta_seconds);

	bound_position_to_map_volume(pos, map_min, map_max);

	min_height = elevation + cg_to_ground;

	if (terrain_follow)
	{
		min_height += 2.0;
	}

	/* bound to floor.  Allow for the pitch of the aircraft (the new attitude) ... assume 5m to nose.  Stop motion vector. */
	if (pos->y <= min_height)
	{
		float pitch;

		pitch = get_pitch_from_attitude_matrix(*matrix);

		min_height += 5.0 * sin(fabs(pitch));

		mv->y = (mv->y > 0.0f) ? mv->y : 0.0f;

		pos->y = (pos->y > min_height) ? pos->y : min_height;
	}
}

/*
 * The rotor state is {main rpm, tail rpm, spin-up timer}.
 *
 * Alive: helicopter_movement_calculate_rotor_rpm, then the spin-up gate.  `gate` is the campaign's decision (an AI or
 * auto-piloted helicopter); while it is set and the rotor is still spinning up the timer counts down and the DLL returns
 * before the rest of the step (`gated`).
 */
void eech_hc_rotor_alive(float *rotor, float lift, float power_output, float delta_seconds, int gate, int *gated)
{
	float desired_rpm, delta_rpm, spin_up_amount;

	desired_rpm = (lift / power_output) * 150.0;
	desired_rpm = min(desired_rpm, 100.0f);

	delta_rpm = desired_rpm - rotor[0];

	spin_up_amount = ((1.0 / 3600) * rotor[0] * rotor[0]);
	spin_up_amount = bound(spin_up_amount, 0.01, 1.0);

	delta_rpm *= spin_up_amount;

	rotor[0] += delta_rpm * delta_seconds;
	rotor[0] = bound(rotor[0], 0.0, 100.0);
	rotor[1] = rotor[0];

	*gated = 0;

	if ((rotor[2] > 0.0 || rotor[0] < 50) && gate)
	{
		rotor[2] -= delta_seconds;
		rotor[2] = (rotor[2] > 0.0f) ? rotor[2] : 0.0f;
		*gated = 1;
	}
}

/* Dying: the rotor winds down. */
void eech_hc_rotor_down(float *rotor, float delta_seconds)
{
	float delta_rpm, spin_up_amount;

	delta_rpm = -rotor[0];

	spin_up_amount = ((1.0 / 3600) * rotor[0] * rotor[0]);
	spin_up_amount = bound(spin_up_amount, 0.3, 1.0);

	delta_rpm *= spin_up_amount;

	rotor[0] += delta_rpm * delta_seconds;

	if (rotor[0] < 2.0)
	{
		rotor[0] = 0.0;
	}

	rotor[1] = rotor[0];
}

/* Idle (no follower parent): the linear wind-down by the frame delta, a different law from the dying wind-down. */
void eech_hc_rotor_idle(float *rotor, float delta_seconds)
{
	rotor[0] -= 5.0 * delta_seconds;
	rotor[0] = bound(rotor[0], 0.0, 100.0);
	rotor[1] = rotor[0];
}

/*
 * Dying, airborne: the candidate position (the motion vector integrated and bound to the map volume) and, after the
 * DLL's terrain sample at it, gravity on the motion vector and the speed it implies.  The two are separate calls
 * because the engine integrates the position before the sample and applies gravity after it.
 */
void eech_hc_fall_position(const float *position, const float *motion_vector, float delta_seconds, const float *map_min, const float *map_max, float *new_position)
{
	const vec3d *pos = (const vec3d *)position;
	const vec3d *mv = (const vec3d *)motion_vector;
	vec3d *new_pos = (vec3d *)new_position;

	new_pos->x = pos->x + mv->x * delta_seconds;
	new_pos->y = pos->y + mv->y * delta_seconds;
	new_pos->z = pos->z + mv->z * delta_seconds;

	bound_position_to_map_volume(new_pos, map_min, map_max);
}

void eech_hc_fall_gravity(float *motion_vector, float gravity, float delta_seconds, float *speed)
{
	vec3d *mv = (vec3d *)motion_vector;

	mv->y -= gravity * delta_seconds;

	*speed = get_3d_vector_magnitude(mv);
}

/* Dying, on contact: the heading, pitch and roll tumble.  The two random draws were made by the DLL, heading's first. */
void eech_hc_tumble(float *attitude, float heading_draw, float roll_draw, float delta_seconds)
{
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	float heading, pitch, roll;

	heading = get_heading_from_attitude_matrix(*matrix);
	pitch = get_pitch_from_attitude_matrix(*matrix);
	roll = get_roll_from_attitude_matrix(*matrix);

	heading += PI * 0.2 * heading_draw * delta_seconds;
	pitch -= pitch * 0.25 * delta_seconds;
	roll -= PI * 0.2 * roll_draw * delta_seconds;

	get_3d_transformation_matrix(*matrix, heading, pitch, roll);
}

/* the engine's gravity (maths/units: G) and the depth of a river or lake that swallows a wreck */
#define EECH_HC_G (9.81f)
#define EECH_HC_RIVER_DEPTH (3.0f)

static void hc_face_normal_matrix(matrix3x3 result, const vec3d *normal, float heading)
{
	/* get_3d_transformation_matrix_from_face_normal_and_heading, expression for expression */
	if (fabs(normal->y) > 0.0001)
	{
		vec3d *y_vector = (vec3d *)&result[1][0];
		vec3d *z_vector = (vec3d *)&result[2][0];
		vec3d *x_vector = (vec3d *)&result[0][0];

		memcpy(&result[1][0], normal, sizeof(vec3d));

		result[2][0] = sin(heading);
		result[2][2] = cos(heading);
		result[2][1] = -((normal->x * result[2][0]) + (normal->z * result[2][2])) / normal->y;

		normalise_any_3d_vector(z_vector);

		x_vector->x = (y_vector->y * z_vector->z) - (z_vector->y * y_vector->z);
		x_vector->y = (y_vector->z * z_vector->x) - (z_vector->z * y_vector->x);
		x_vector->z = (y_vector->x * z_vector->y) - (z_vector->x * y_vector->y);
	}
	else
	{
		result[0][0] = 1.0;
		result[0][1] = 0.0;
		result[0][2] = 0.0;
		result[1][0] = 0.0;
		result[1][1] = 1.0;
		result[1][2] = 0.0;
		result[2][0] = 0.0;
		result[2][1] = 0.0;
		result[2][2] = 1.0;
	}
}

/*
 * The contact test: the point's height above the terrain sample the DLL took under it (elevation plus the class's
 * centre-of-gravity height).  Contact is altitude <= 0, which is what the dying movement acts on.
 */
void eech_hc_contact(const float *point, float elevation, float cg_to_ground, float *altitude, int *contact)
{
	const vec3d *p = (const vec3d *)point;

	*altitude = p->y - (elevation + cg_to_ground);
	*contact = (*altitude <= 0.0) ? 1 : 0;
}

/*
 * Dying, on contact, after the DLL's effects and its splash placement: the candidate position settles by the terrain's
 * class (`terrain_class`: 0 solid ground, 1 sea, 2 river or lake, 3 forest).  Water keeps the held position's x and z (the
 * position after the splash, which is what the engine's pointer then reads); the candidate's height is bound to the
 * ground on solid ground and to the river bed.  `buried` is the aircraft sinking past `bury_limit` (the DLL's bounding
 * box, a negative altitude): the rotor stops and the motion vector is cleared; the DLL clears the scalar speed with it.
 *
 * Two centre-of-gravity heights are in play, as in the engine.  `contact_altitude` is the candidate's altitude from
 * eech_hc_contact, measured with the pre-kill (live) CG, and is what the bury tests use; it is not recomputed.
 * `cg_to_ground` is the CG queried afresh after the kill (the destroyed CG) and sets only the final solid and river heights.
 */
void eech_hc_settle(float *candidate, const float *held_position, float *motion_vector, float *rotor, int terrain_class, float elevation, float cg_to_ground, float contact_altitude, float bury_limit, int *buried)
{
	vec3d *new_pos = (vec3d *)candidate;
	const vec3d *pos = (const vec3d *)held_position;
	vec3d *mv = (vec3d *)motion_vector;
	float altitude;

	altitude = contact_altitude;

	*buried = 0;

	switch (terrain_class)
	{
	case 1:
		new_pos->x = pos->x;
		new_pos->z = pos->z;
		if (altitude < bury_limit)
		{
			*buried = 1;
		}
		break;
	case 2:
		new_pos->x = pos->x;
		new_pos->z = pos->z;
		if (altitude < -EECH_HC_RIVER_DEPTH)
		{
			new_pos->y = elevation + cg_to_ground - EECH_HC_RIVER_DEPTH;
			*buried = 1;
		}
		break;
	case 3:
		if (altitude < bury_limit)
		{
			*buried = 1;
		}
		break;
	default:
		new_pos->y = elevation + cg_to_ground;
		break;
	}

	if (*buried)
	{
		rotor[0] = 0.0;
		rotor[1] = 0.0;
		mv->x = 0.0;
		mv->y = 0.0;
		mv->z = 0.0;
	}
}

/*
 * Crash, phase one: the attitude eases towards the terrain's face normal as it was before the new sample (the DLL's
 * cached terrain, `face_normal`) plus the class's destroyed offsets; the candidate's x and z are the held position plus the
 * motion vector, bound to the map area.  The candidate's y is the held position's until phase two.
 */
void eech_hc_crash_begin(float *attitude, const float *position, const float *motion_vector, const float *face_normal, float destroyed_pitch_offset, float destroyed_bank_offset, float delta_seconds, const float *map_min, const float *map_max, float *candidate)
{
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	const vec3d *pos = (const vec3d *)position;
	const vec3d *velocity = (const vec3d *)motion_vector;
	vec3d *new_pos = (vec3d *)candidate;
	matrix3x3 m;
	float heading, pitch, roll, required_angle, delta_angle, max_angle;

	heading = get_heading_from_attitude_matrix(*matrix);
	pitch = get_pitch_from_attitude_matrix(*matrix);
	roll = get_roll_from_attitude_matrix(*matrix);

	hc_face_normal_matrix(m, (const vec3d *)face_normal, heading);

	required_angle = get_pitch_from_attitude_matrix(m) + destroyed_pitch_offset;
	max_angle = 2 * PI * delta_seconds;
	delta_angle = bound(required_angle - pitch, -max_angle, max_angle);
	pitch += delta_angle;

	required_angle = get_roll_from_attitude_matrix(m) + destroyed_bank_offset;
	max_angle = 2 * PI * 0.25 * delta_seconds;
	delta_angle = bound(required_angle - roll, -max_angle, max_angle);
	roll += delta_angle;

	get_3d_transformation_matrix(*matrix, heading, pitch, roll);

	new_pos->x = pos->x + (velocity->x * delta_seconds);
	new_pos->z = pos->z + (velocity->z * delta_seconds);

	/* bound_position_to_map_area: x and z only */
	if (new_pos->x < map_min[0])
	{
		new_pos->x = map_min[0];
	}
	else if (new_pos->x > map_max[0])
	{
		new_pos->x = map_max[0];
	}

	if (new_pos->z < map_min[2])
	{
		new_pos->z = map_min[2];
	}
	else if (new_pos->z > map_max[2])
	{
		new_pos->z = map_max[2];
	}

	new_pos->y = pos->y;
}

/*
 * Crash, phase two, after the DLL sampled terrain at the candidate: the candidate's height (a forest keeps the held
 * position's, any other terrain the ground plus the centre-of-gravity height), the friction on the motion vector, the
 * slope's push from the NEW sample's face normal (`face_normal_x`, `face_normal_z`) and the vertical speed that
 * leaves the candidate.  `stopped` is the speed (before the slope push) falling under 1.  The motion vector is left as the
 * friction and slope made it: the engine sets the position with it still present and only then clears it
 * (clear_helicopter_velocity), so the DLL does the clearing after the position setter.
 */
void eech_hc_crash_end(const float *position, float *candidate, float *motion_vector, int forest, float elevation, float cg_to_ground, float face_normal_x, float face_normal_z, float delta_seconds, float *speed_out, int *stopped)
{
	const vec3d *pos = (const vec3d *)position;
	vec3d *new_pos = (vec3d *)candidate;
	vec3d *velocity = (vec3d *)motion_vector;
	float speed, acc, dx, dy, dz;

	if (forest)
	{
		new_pos->y = pos->y;
	}
	else
	{
		new_pos->y = elevation + cg_to_ground;
	}

	speed = normalise_any_3d_vector(velocity);

	acc = (8 * speed > 10.0f) ? 8 * speed : 10.0f;

	speed -= (acc * delta_seconds);

	speed = (speed > 0.0f) ? speed : 0.0f;

	velocity->x *= speed;
	velocity->y *= speed;
	velocity->z *= speed;

	dx = (EECH_HC_G * face_normal_x * delta_seconds);
	dz = (EECH_HC_G * face_normal_z * delta_seconds);

	dy = (new_pos->y - pos->y) / delta_seconds;

	velocity->x += dx;
	velocity->z += dz;

	velocity->y = dy;

	*speed_out = speed;
	*stopped = (speed < 1.0) ? 1 : 0;
}

/* Impact: the DLL's signed random speed pushes the motion vector along the attitude's x axis. */
void eech_hc_impact(float *motion_vector, const float *attitude, float signed_speed)
{
	vec3d *velocity = (vec3d *)motion_vector;
	const vec3d *xv = (const vec3d *)attitude;

	velocity->x += (signed_speed * xv->x);

	velocity->z += (signed_speed * xv->z);
}
