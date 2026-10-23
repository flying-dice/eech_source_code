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
void eech_hc_tumble(float *attitude, double heading_draw, double roll_draw, float delta_seconds)
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

/*
 * Fixed-wing dying and crash arithmetic (EEDCS-37).  These share hc_face_normal_matrix and the type-agnostic fall,
 * contact and impact helpers above (eech_hc_fall_position, eech_hc_fall_gravity, eech_hc_contact, eech_hc_impact); the
 * settle is eech_fw_settle below because the fixed-wing river depth is the double 3.0 and there is no rotor.  Everything
 * here is the fw_move.c expression for expression; the terrain refresh, effects, RNG draws, kill, DEAD and smoke clears stay
 * in the DLL.
 */

/*
 * fixed_wing_crash_movement, phase one: the candidate x and z are the held position plus the motion vector, bound to the
 * map area (x and z only).  The candidate's y is the held position's until phase two.
 */
void eech_fw_crash_candidate(const float *position, const float *motion_vector, float delta_seconds, const float *map_min, const float *map_max, float *candidate)
{
	const vec3d *pos = (const vec3d *)position;
	const vec3d *velocity = (const vec3d *)motion_vector;
	vec3d *new_pos = (vec3d *)candidate;

	new_pos->x = pos->x + (velocity->x * delta_seconds);
	new_pos->z = pos->z + (velocity->z * delta_seconds);

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
 * fixed_wing_crash_movement, phase two (CRASH_VELOCITY), after the DLL's terrain sample at the candidate: the candidate's
 * height, then friction and the slope push on the motion vector using the NEW sample's face normal (unlike the
 * helicopter, which eases to the old normal).  The pre-friction speed (the `speed > 0` test) is the magnitude of the held
 * vector.  `speed_out` is the post-slope magnitude and `stopped` is speed < 1; the DLL then clears the velocity
 * (stopped) or sets FLOAT_TYPE_VELOCITY to the speed, which is BEFORE the attitude rebuild and the position setter.
 */
void eech_fw_crash_velocity(const float *position, float *candidate, float *motion_vector, const float *face_normal, int forest, float elevation, float cg_to_ground, float delta_seconds, float *speed_out, int *stopped)
{
	const vec3d *pos = (const vec3d *)position;
	vec3d *new_pos = (vec3d *)candidate;
	vec3d *velocity = (vec3d *)motion_vector;
	const vec3d *normal = (const vec3d *)face_normal;
	float speed, horizontal_speed;

	speed = get_3d_vector_magnitude(velocity);

	if (forest)
	{
		new_pos->y = pos->y;
	}
	else
	{
		new_pos->y = elevation + cg_to_ground;
	}

	horizontal_speed = sqrt((velocity->x * velocity->x) + (velocity->z * velocity->z));

	if (speed > 0.0)
	{
		normalise_3d_vector_given_magnitude(velocity, speed);

		horizontal_speed -= (((8.0f * horizontal_speed > 10.0f) ? 8.0f * horizontal_speed : 10.0f) * delta_seconds);

		horizontal_speed = (horizontal_speed > 0.0f) ? horizontal_speed : 0.0f;

		velocity->x *= horizontal_speed;
		velocity->y = 0.0;
		velocity->z *= horizontal_speed;
	}
	else
	{
		velocity->x = 0.0;
		velocity->y = 0.0;
		velocity->z = 0.0;
	}

	if (normal->y < 0.3)
	{
		velocity->x += (EECH_HC_G * normal->x * delta_seconds);
		velocity->z += (EECH_HC_G * normal->z * delta_seconds);
	}

	speed = get_3d_vector_magnitude(velocity);

	*speed_out = speed;
	*stopped = (speed < 1.0) ? 1 : 0;
}

/*
 * fixed_wing_crash_movement, phase three (CRASH_ATTITUDE), after the DLL has cleared the velocity or set the speed: the
 * pitch and roll ease toward the NEW face normal's attitude plus the destroyed offsets and the attitude is rebuilt (the
 * caller writes it raw), then the DLL's position setter runs.  `at_rest` is the DLL's stop test without its `!alive` term:
 * speed <= 0.01 && delta_pitch <= 0.01 && delta_roll <= 0.01 (signed deltas, as the engine compares them); `speed` is
 * the phase-two post-slope magnitude.
 */
void eech_fw_crash_attitude(float *attitude, const float *face_normal, float speed, float destroyed_pitch_offset, float destroyed_bank_offset, float delta_seconds, int *at_rest)
{
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	const vec3d *normal = (const vec3d *)face_normal;
	matrix3x3 m;
	float heading, pitch, roll, required_angle, max_angle, delta_pitch, delta_roll;

	heading = get_heading_from_attitude_matrix(*matrix);
	pitch = get_pitch_from_attitude_matrix(*matrix);
	roll = get_roll_from_attitude_matrix(*matrix);

	hc_face_normal_matrix(m, normal, heading);

	required_angle = get_pitch_from_attitude_matrix(m) + destroyed_pitch_offset;
	max_angle = 2 * PI * delta_seconds;
	delta_pitch = bound(required_angle - pitch, -max_angle, max_angle);
	pitch += delta_pitch;

	required_angle = get_roll_from_attitude_matrix(m) + destroyed_bank_offset;
	max_angle = 2 * PI * delta_seconds;
	delta_roll = bound(required_angle - roll, -max_angle, max_angle);
	roll += delta_roll;

	get_3d_transformation_matrix(*matrix, heading, pitch, roll);

	*at_rest = ((speed <= 0.01) && (delta_pitch <= 0.01) && (delta_roll <= 0.01)) ? 1 : 0;
}

/*
 * basic_fixed_wing_death_movement settle, after the DLL's effects and the fresh post-kill CG query.  Same classes and
 * bury tests as eech_hc_settle, but the river depth is the double RIVER_DEPTH 3.0 with the original expression shape
 * (`y = elevation + cg; y -= 3.0`, test `altitude < -3.0`) and there is no rotor: a bury only zeroes the motion vector (the
 * DLL then runs clear_fixed_wing_velocity at the original point).  `contact_altitude` is the candidate's altitude measured
 * with the PRE-kill CG; `cg_to_ground` is the FRESH post-kill CG and sets only the final solid and river heights.
 */
void eech_fw_settle(float *candidate, const float *held_position, float *motion_vector, int terrain_class, float elevation, float cg_to_ground, float contact_altitude, float bury_limit, int *buried)
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
		if (altitude < -3.0)
		{
			new_pos->y = elevation + cg_to_ground;
			new_pos->y -= 3.0;
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
		mv->x = 0.0;
		mv->y = 0.0;
		mv->z = 0.0;
	}
}

/*
 * basic_fixed_wing_death_movement, attitude block (after the settle/bury, before the position setter).  `speed` is the
 * magnitude the engine last computed (after gravity, before any bury clear); `altitude` is the candidate altitude
 * measured against the new terrain with the live CG.  The block runs only while the motion vector's x or z is non-zero,
 * and eases the roll by the DLL's frand1() draw only when altitude > 12.  The DLL draws only in exactly that case and
 * passes `draw_made`; the function returns -1 when the host's gate and the DLL's draw disagree, otherwise bit 0 =
 * attitude rebuilt, bit 1 = draw consumed.
 */
int eech_fw_death_attitude(float *attitude, const float *motion_vector, float speed, float altitude, int draw_made, double roll_draw, float delta_seconds)
{
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	vec3d direction_vector;
	float heading, pitch, roll, required_heading, required_pitch, dh, dp;
	int needs_draw;

	if (!((motion_vector[0] != 0.0) || (motion_vector[2] != 0.0)))
	{
		return draw_made ? -1 : 0;
	}

	needs_draw = (altitude > 12.0) ? 1 : 0;

	if (needs_draw != (draw_made ? 1 : 0))
	{
		return -1;
	}

	heading = get_heading_from_attitude_matrix(*matrix);
	pitch = get_pitch_from_attitude_matrix(*matrix);
	roll = get_roll_from_attitude_matrix(*matrix);

	memcpy(&direction_vector, motion_vector, sizeof(vec3d));

	normalise_3d_vector_given_magnitude(&direction_vector, speed);

	/* get_heading_and_pitch_from_3d_unit_vector */
	required_heading = atan2(direction_vector.x, direction_vector.z);
	required_pitch = asin(direction_vector.y);

	dh = bound(required_heading - heading, -(PI * 0.25), (PI * 0.25));

	heading += (dh * delta_seconds);

	if (needs_draw)
	{
		roll += (PI * 0.2 * roll_draw * delta_seconds);

		dp = bound(required_pitch - pitch, -(PI * 0.25), (PI * 0.25));

		pitch += (dp * delta_seconds);
	}

	get_3d_transformation_matrix(*matrix, heading, pitch, roll);

	return 1 | (needs_draw ? 2 : 0);
}

/*
 * Helicopter physical guidance (EEDCS-39), from helicopter_movement and its helpers in hc_move.c: the LOS adjustment of the
 * fly-to position (helicopter_adjust_waypoint_position_with_los), the altitude follow with the structure avoidance and the
 * terrain look-ahead (helicopter_movement_altitude_follow, helicopter_movement_structure_avoidance), the velocity shaping,
 * the world acceleration and the lift, and the heading, pitch and roll demands (helicopter_movement_get_desired_heading,
 * _pitch and _roll).  Expression for expression: the double literals, the float stores, the macros' repeated operands and the
 * engine's quirks are kept.  Quirks kept on purpose:
 *   - the holding waypoints compute a ramped max_vel and then overwrite it with 15.0;
 *   - the desired roll is computed from the lateral velocity and then overwritten by the acceleration term;
 *   - the absolute-velocity limit divides by (mag / (1.1 * max_vel)).
 * The campaign keeps which waypoint, formation slot, hold, target and required heading apply; none of that is decided here.
 * The sector's structures and the terrain samples are the DLL's transitional services: the physical positions to look up
 * are computed here (eech_hc_guidance_plan) and the obstacle selection and avoidance arithmetic on the answers too.
 */
#include "hc_guidance.h"

#define EECH_HC_METRE (1.0f)
#define EECH_HC_VELOCITY_RAMP_DISTANCE (500.0f * EECH_HC_METRE)
#define EECH_HC_MAX_PITCH (rad(20.0f))
#define EECH_HC_MAX_ROLL (rad(30.0f))
#define EECH_HC_LATERAL_TO_FORWARD_VELOCITY_SCALING 5.0f
#define EECH_HC_AVOIDANCE_DISTANCE 20.0f
#define EECH_HC_STRUCTURE_AVOIDANCE_LOOKAHEAD_TIME 1.0f
#define EECH_HC_TERRAIN_AVOIDANCE_LOOKAHEAD_TIME 2.0f
#define EECH_HC_TERRAIN_AVOIDANCE_LOOKAHEAD_SAMPLE_NUMBER 1

static void hc_load_vec3d(vec3d *vector, const float *values)
{
	vector->x = values[0];
	vector->y = values[1];
	vector->z = values[2];
}

/* helicopter_adjust_waypoint_position_with_los: steer the fly-to position off the nearest group member on the line to it */
static void hc_adjust_waypoint_with_los(const eech_hc_guidance_input *input, vec3d *wp_pos)
{
	float closest_range, range, distance;
	vec3d closest_collision_point, collision_point, en_pos, hc_pos;
	const eech_hc_other *closest_hc;
	int index;

	if (input->group_leader)
	{
		/* leader doesn't avoid wingmen */
		return;
	}

	hc_load_vec3d(&en_pos, input->position);

	memset(&closest_collision_point, 0, sizeof(closest_collision_point));
	memset(&collision_point, 0, sizeof(collision_point));

	closest_range = ((500.0 * EECH_HC_METRE) * (500.0 * EECH_HC_METRE));

	closest_hc = NULL;

	for (index = 0; index < input->other_count; index++)
	{
		hc_load_vec3d(&hc_pos, input->others[index].position);

		range = get_sqr_3d_range(&hc_pos, &en_pos);

		if (range < closest_range)
		{
			distance = get_3d_perp_dist_of_point_from_line(&en_pos, wp_pos, &hc_pos, &collision_point);

			if (distance <= EECH_HC_AVOIDANCE_DISTANCE)
			{
				closest_range = range;

				closest_hc = &input->others[index];

				closest_collision_point = collision_point;
			}
		}
	}

	if (closest_hc)
	{
		vec3d new_pos;

		hc_load_vec3d(&hc_pos, closest_hc->position);

		new_pos.x = closest_collision_point.x - hc_pos.x;
		new_pos.y = closest_collision_point.y - hc_pos.y;
		new_pos.z = closest_collision_point.z - hc_pos.z;

		normalise_any_3d_vector(&new_pos);

		new_pos.x = hc_pos.x + (2.0 * EECH_HC_AVOIDANCE_DISTANCE * new_pos.x);
		new_pos.y = hc_pos.y + (2.0 * EECH_HC_AVOIDANCE_DISTANCE * new_pos.y);
		new_pos.z = hc_pos.z + (2.0 * EECH_HC_AVOIDANCE_DISTANCE * new_pos.z);

		wp_pos->x = new_pos.x;
		wp_pos->y = new_pos.y;
		wp_pos->z = new_pos.z;
	}
}

/* the task leader inside its guide radius, with an altitude criterion, takes no structure clearance */
static int hc_structure_avoidance_skipped(const eech_hc_guidance_input *input)
{
	if (input->task_leader)
	{
		if (input->guide_radius_valid)
		{
			if (input->guide_altitude_valid)
			{
				if (input->distance < input->guide_radius)
				{
					return 1;
				}
			}
		}
	}

	return 0;
}

/* the look-ahead point of helicopter_movement_structure_avoidance (y is 0) */
static void hc_structure_look_ahead(const eech_hc_guidance_input *input, vec3d *pos)
{
	const matrix3x3 *attitude = (const matrix3x3 *)input->attitude;

	hc_load_vec3d(pos, input->position);

	pos->x += ((*attitude)[2][0] * input->cruise_velocity * EECH_HC_STRUCTURE_AVOIDANCE_LOOKAHEAD_TIME);
	pos->y = 0.0;
	pos->z += ((*attitude)[2][2] * input->cruise_velocity * EECH_HC_STRUCTURE_AVOIDANCE_LOOKAHEAD_TIME);
}

/* helicopter_movement_structure_avoidance: the tallest structure whose radius reaches the look-ahead point */
static float hc_structure_avoidance(const eech_hc_guidance_input *input, const eech_hc_structure *structures, int structure_count)
{
	vec3d pos, test_pos;
	float d, radius, max_height;
	int index;

	if (hc_structure_avoidance_skipped(input))
	{
		return 0.0;
	}

	hc_structure_look_ahead(input, &pos);

	max_height = 0;

	for (index = 0; index < structure_count; index++)
	{
		radius = structures[index].avoidance_radius;

		if (radius > 0.0)
		{
			radius *= radius;

			hc_load_vec3d(&test_pos, structures[index].position);

			d = get_sqr_2d_range(&pos, &test_pos);

			if (d < radius)
			{
				max_height = max(max_height, structures[index].height);
			}
		}
	}

	return max_height;
}

/* the terrain sample positions of helicopter_movement_altitude_follow, loop from the far sample down to the aircraft */
static void hc_terrain_sample_position(const eech_hc_guidance_input *input, int loop, vec3d *test_pos)
{
	const matrix3x3 *attitude = (const matrix3x3 *)input->attitude;
	vec3d pos;

	hc_load_vec3d(&pos, input->position);

	test_pos->x = pos.x + ((*attitude)[2][0] * loop * input->cruise_velocity * EECH_HC_TERRAIN_AVOIDANCE_LOOKAHEAD_TIME);
	test_pos->y = pos.y + 0.0;
	test_pos->z = pos.z + ((*attitude)[2][2] * loop * input->cruise_velocity * EECH_HC_TERRAIN_AVOIDANCE_LOOKAHEAD_TIME);

	bound_position_to_map_area(test_pos, input->map_min, input->map_max);
}

void eech_hc_guidance_plan(const eech_hc_guidance_input *input, eech_hc_query_plan *plan)
{
	int loop, sample;
	vec3d pos;

	memset(plan, 0, sizeof(*plan));

	if (!input->terrain_follow)
	{
		return;
	}

	if (!hc_structure_avoidance_skipped(input))
	{
		hc_structure_look_ahead(input, &pos);

		plan->structure_scan = 1;
		plan->structure_position[0] = pos.x;
		plan->structure_position[1] = pos.y;
		plan->structure_position[2] = pos.z;
	}

	sample = 0;

	for (loop = EECH_HC_TERRAIN_AVOIDANCE_LOOKAHEAD_SAMPLE_NUMBER; loop >= 0; loop--)
	{
		hc_terrain_sample_position(input, loop, &pos);

		plan->terrain_sample[sample][0] = pos.x;
		plan->terrain_sample[sample][1] = pos.z;

		sample++;
	}

	plan->terrain_sample_count = sample;
}

/* helicopter_movement_altitude_follow: the required height above sea level */
static float hc_altitude_follow(const eech_hc_guidance_input *input, const eech_hc_structure *structures, int structure_count, float sampled_terrain_elevation, const vec3d *wp_pos)
{
	float min_height, required_height;

	if (input->terrain_follow)
	{
		if (input->terrain_follow_attack_altitude || input->position_hold)
		{
			min_height = input->attack_altitude + hc_structure_avoidance(input, structures, structure_count);
		}
		else
		{
			if (input->task_leader_is_self)
			{
				min_height = input->cruise_altitude + hc_structure_avoidance(input, structures, structure_count);
			}
			else
			{
				min_height = max(input->leader_radar_altitude, (input->cruise_altitude + hc_structure_avoidance(input, structures, structure_count)));
			}
		}

		/*
		 * The original folds the samples with terrain_elevation = max (terrain_elevation, get_3d_terrain_point_data (...)),
		 * starting from 0.0.  The max macro calls the getter once when the running value exceeds the first return and
		 * otherwise a second time, and then selects that second return.  Which return is selected is the host's fold (it
		 * keeps every return in call order); this is its result.
		 */
		required_height = min_height + sampled_terrain_elevation;
	}
	else
	{
		float tmp;

		tmp = pow(input->velocity, 4.0f) - 0.1f;
		required_height = wp_pos->y + bound(tmp, 0.0f, 2.0f);
	}

	return required_height;
}

void eech_hc_guidance(const eech_hc_guidance_input *input, const eech_hc_structure *structures, int structure_count, float sampled_terrain_elevation, eech_hc_guidance_result *result)
{
	float lift, height, range, max_vel, range_scaler, max_x_vel, max_y_vel, max_z_vel;
	vec3d wp_vec, wp_pos, wp_vel, hc_pos, normalised_wp_vec, motion_vector, world_acceleration_vector;
	matrix3x3 attitude;

	memcpy(attitude, input->attitude, sizeof(attitude));

	hc_load_vec3d(&hc_pos, input->position);
	hc_load_vec3d(&wp_pos, input->waypoint);
	hc_load_vec3d(&wp_vel, input->waypoint_velocity);
	hc_load_vec3d(&motion_vector, input->motion_vector);

	wp_pos.y = max(wp_pos.y, input->terrain_elevation);

	hc_adjust_waypoint_with_los(input, &wp_pos);

	range = input->distance;

	height = hc_altitude_follow(input, structures, structure_count, sampled_terrain_elevation, &wp_pos);

	// ramp vel from 0.01 to cruise_vel between 0m -> HELICOPTER_VELOCITY_RAMP_DISTANCE
	range_scaler = range / EECH_HC_VELOCITY_RAMP_DISTANCE;
	range_scaler = bound(range_scaler, 0.01f, 1.0f);

	wp_vec.x = wp_pos.x - hc_pos.x;
	wp_vec.y = wp_pos.y - hc_pos.y;
	wp_vec.z = wp_pos.z - hc_pos.z;

	// calcualte max_vel
	max_vel = input->cruise_velocity;

	// OUT OF FUEL HANDICAP
	if (input->fuel_supply_level <= 10.0)
	{
		max_vel /= 5.0;
	}

	switch (input->waypoint_kind)
	{
	case EECH_HC_WP_ROUTE:
		break;
	case EECH_HC_WP_HOLDING:
		max_vel = min(max_vel * ((2.0 * range_scaler) - (range_scaler * range_scaler)), 15.0);
		max_vel = 15.0;
		break;
	default:
		max_vel *= ((2.0 * range_scaler) - (range_scaler * range_scaler));
		break;
	}

	// bound speeds to something realistic
	max_x_vel = 0.2 * input->cruise_velocity;
	max_y_vel = 0.1 * input->cruise_velocity;
	max_z_vel = 1.0 * input->cruise_velocity;

	normalise_any_3d_vector(&wp_vec);

	// rotate into model space for speed bounding
	multiply_transpose_matrix3x3_vec3d(&wp_vec, attitude, &wp_vec);

	normalised_wp_vec = wp_vec;

	// get velocity vector to wp
	wp_vec.x *= max_vel;
	wp_vec.y *= max_vel;
	wp_vec.z *= max_vel;

	// limit max_velocity depending on required climb rate
	max_z_vel -= (max_z_vel * min((max(height - hc_pos.y, 0.0f) / max_y_vel), 1.0f));
	max_x_vel -= (max_x_vel * min((max(height - hc_pos.y, 0.0f) / max_y_vel), 1.0f));

	// limit forward acceleration depending on climb demand

	wp_vec.x = bound(wp_vec.x, -max_x_vel, max_x_vel);
	wp_vec.y = bound(wp_vec.y, -max_y_vel, max_y_vel);
	wp_vec.z = bound(wp_vec.z, -max_z_vel, max_z_vel);

	// rotate back into world space
	multiply_matrix3x3_vec3d(&wp_vec, attitude, &wp_vec);

	// add on required height for terrain avoidance
	wp_vec.y = (height - hc_pos.y);
	wp_vec.y = bound(wp_vec.y, -max_y_vel, max_y_vel);

	// add on the velocity of the wp (i.e. the speed it is moving)
	// don't add in Y-component as it causes the wingmen to crash into the floor if the player decends quickly
	wp_vec.x += wp_vel.x;
	wp_vec.z += wp_vel.z;

	if (get_3d_vector_magnitude(&wp_vec) > 1.1 * max_vel) // limit absolute velocity value
		normalise_3d_vector_given_magnitude(&wp_vec, get_3d_vector_magnitude(&wp_vec) / (1.1 * max_vel));

	world_acceleration_vector.x = (wp_vec.x - motion_vector.x) * 0.3;
	world_acceleration_vector.y = (wp_vec.y - motion_vector.y) * 0.3;
	world_acceleration_vector.z = (wp_vec.z - motion_vector.z) * 0.3;

	// bodge for rotor rpm
	lift = (0.7 * input->power_output) + (fabs(world_acceleration_vector.y) + fabs(world_acceleration_vector.z));

	result->world_acceleration[0] = world_acceleration_vector.x;
	result->world_acceleration[1] = world_acceleration_vector.y;
	result->world_acceleration[2] = world_acceleration_vector.z;
	result->wp_vec[0] = wp_vec.x;
	result->wp_vec[1] = wp_vec.y;
	result->wp_vec[2] = wp_vec.z;
	result->lift = lift;
	result->waypoint[0] = wp_pos.x;
	result->waypoint[1] = wp_pos.y;
	result->waypoint[2] = wp_pos.z;
	result->required_height = height;
}

/* helicopter_movement_get_desired_heading */
static float hc_desired_heading(const eech_hc_demand_input *input, const vec3d *fly_to_pos, float current_heading, int *needs_leader)
{
	float heading;

	*needs_leader = 0;

	switch (input->waypoint_kind)
	{
	case EECH_HC_WP_LANDED: // save heading angle so helicopter will land properly
		if (input->velocity < 10.0)
			return current_heading;
		/* fall through */
	case EECH_HC_WP_INSERTION: // do not change leader's heading value before troops insertion
		if (input->task_leader_is_self && input->velocity < 5.0)
			return current_heading;
	}

	if (input->guide_radius_valid)
	{
		if (input->distance < input->guide_radius)
		{
			if (((input->motion_vector[0] * input->motion_vector[0]) + (input->motion_vector[2] * input->motion_vector[2])) < (5.0 * 5.0))
			{
				if (input->required_heading_valid)
				{
					return input->required_heading;
				}
				else
				{
					if (!input->task_leader_is_self)
					{
						/* get_local_entity_float_value (leader, FLOAT_TYPE_HEADING): the heading of the leader's attitude */
						if (!input->leader_attitude_valid)
						{
							*needs_leader = 1;

							return current_heading;
						}

						return get_heading_from_attitude_matrix(((const float (*)[3]) input->leader_attitude));
					}
				}
			}
		}
	}

	// Default case :- face the direction the helicopter is flying if more than 5m away,
	// otherwise just face the way its facing already.
	if (((fly_to_pos->x * fly_to_pos->x) + (fly_to_pos->z * fly_to_pos->z)) < (5.0 * 5.0))
	{
		heading = current_heading;
	}
	else
	{
		heading = atan2(fly_to_pos->x, fly_to_pos->z);
	}

	return heading;
}

/* helicopter_movement_get_desired_pitch */
static float hc_desired_pitch(const eech_hc_demand_input *input, const vec3d *model_motion_vector, const vec3d *model_acceleration_vector)
{
	const matrix3x3 *attitude = (const matrix3x3 *)input->attitude;
	float pitch, targeting_pitch = 0.0, current_pitch;

	if (input->targeting_active)
	{
		float weapon_to_target_pitch, weapon_pitch;
		vec3d weapon_vector, weapon_to_intercept_point_vector;

		hc_load_vec3d(&weapon_vector, input->weapon_vector);
		hc_load_vec3d(&weapon_to_intercept_point_vector, input->weapon_to_intercept_vector);

		/* get_heading_and_pitch_from_3d_unit_vector: only the pitch is used */
		weapon_to_target_pitch = asin(weapon_to_intercept_point_vector.y);
		weapon_pitch = asin(weapon_vector.y);

		targeting_pitch = weapon_to_target_pitch - weapon_pitch;
	}

	// calculate pitch for helicopter at given velocity
	pitch = -(EECH_HC_MAX_PITCH * min((model_motion_vector->z / input->cruise_velocity), 1.0f));

	// reduce pitch if climbing, upto 0 pitch
	pitch = min(pitch + EECH_HC_MAX_PITCH * bound(((fabs(model_motion_vector->y) * EECH_HC_G) / input->g_max), -1.0, 1.0), 0.0);

	// calculate pitch for helicopter at given acceleration
	pitch -= EECH_HC_MAX_PITCH * bound(((model_acceleration_vector->z * EECH_HC_G) / input->g_max), -1.0, 1.0);

	// add in rotor angle
	pitch += input->main_rotor_shaft_angle;

	/* FLOAT_TYPE_PITCH */
	current_pitch = get_pitch_from_attitude_matrix(*attitude);

	return bound(targeting_pitch ? current_pitch + targeting_pitch : pitch, - EECH_HC_MAX_PITCH, EECH_HC_MAX_PITCH);
}

/* helicopter_movement_get_desired_roll */
static float hc_desired_roll(const eech_hc_demand_input *input, float lateral_velocity, float lateral_acceleration)
{
	float roll;

	// calculate roll for helicopter at given velocity
	roll = (EECH_HC_MAX_ROLL * min(((lateral_velocity * EECH_HC_LATERAL_TO_FORWARD_VELOCITY_SCALING) / input->cruise_velocity), 1.0f));

	// calculate roll for helicopter at given acceleration (the engine overwrites the velocity term)
	roll = EECH_HC_MAX_ROLL * min(((lateral_acceleration * EECH_HC_G) / input->g_max), 1.0f);

	return roll;
}

/*
 * The post-begin demands: the heading from the fly-to velocity, the pitch and roll from the model-space motion vector (the
 * one eech_hc_begin just built) and the model-space world acceleration (the pre-begin one).
 */
int eech_hc_demand(const eech_hc_demand_input *input, eech_hc_demand_result *result)
{
	matrix3x3 attitude;
	vec3d wp_vec, motion_vector, world_acceleration_vector, model_motion_vector, model_acceleration_vector;
	float current_heading;
	int needs_leader;

	memcpy(attitude, input->attitude, sizeof(attitude));

	hc_load_vec3d(&wp_vec, input->wp_vec);
	hc_load_vec3d(&motion_vector, input->motion_vector);
	hc_load_vec3d(&world_acceleration_vector, input->world_acceleration);

	current_heading = get_heading_from_attitude_matrix(attitude);

	result->desired_heading = hc_desired_heading(input, &wp_vec, current_heading, &needs_leader);

	if (needs_leader)
	{
		return EECH_HC_DEMAND_NEEDS_LEADER;
	}

	multiply_transpose_matrix3x3_vec3d(&model_motion_vector, attitude, &motion_vector);
	multiply_transpose_matrix3x3_vec3d(&model_acceleration_vector, attitude, &world_acceleration_vector);

	result->desired_pitch = hc_desired_pitch(input, &model_motion_vector, &model_acceleration_vector);
	result->desired_roll = hc_desired_roll(input, model_motion_vector.x, model_acceleration_vector.x);

	return 0;
}
