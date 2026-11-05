/*
 * Host-owned dying/wreck/fall/sink arithmetic for routed, anti-air and ship
 * vehicles (EEDCS-37). Pure arithmetic only: terrain refresh, effects, RNG
 * draws, kill/DEAD/destroy and smoke clears stay with the campaign DLL and are
 * interleaved at the original call boundaries (see CHECKPOINT.md).
 *
 * Source anchors: rv_move.c routed_vehicle_death_movement /
 * routed_vehicle_falling_movement / routed_vehicle_impact_movement,
 * aa_move.c anti_aircraft_death_movement (identical but for the speed literal
 * and the DEAD gate), sh_move.c ship_vehicle_death_movement.
 *
 * Literal precision is deliberate: routed `speed -= 4.0f * dt` (float) vs
 * anti-air `speed -= (4.0 * dt)` (double, stored back to float);
 * `heading -= PI * 0.1 * dt` (PI is a float macro, 0.1 a double).
 */

#include <math.h>

#include "eech_arith.h"

#define SHIP_SINK_RATE 0.06 /* double literal, as sh_move.c */

/* routed/AA death candidate: x/z only, no map bound (original has an ASSERT). */
void eech_gv_death_candidate(const float *pos, const float *mv, float dt, float *out_xz)
{
	out_xz[0] = pos[0] + mv[0] * dt;
	out_xz[1] = pos[2] + mv[2] * dt;
}

/*
 * routed/AA death decay after the DLL terrain refresh at the candidate.
 * `mv` is the pre-decay vector (speed != 0, checked by the DLL); `wide`
 * selects the anti-air double literal. Writes the decayed vector, the new
 * speed, the new heading and the final position (y = elevation + cg). The
 * heading, pitch and roll come from the host-retained `attitude` (9 floats),
 * never from the caller.
 */
void eech_gv_death_decay(
	const float *candidate_xz, float elevation, float cg, const float *mv,
	const float *attitude, float dt, int wide,
	float *out_pos, float *out_mv, float *out_speed, float *out_attitude)
{
	vec3d v;
	matrix3x3 m;
	float speed;
	float heading;
	float pitch;
	float roll;
	const float (*held)[3] = (const float (*)[3]) attitude;

	/* the host's own retained attitude, as the original reads raw->mob.attitude */
	heading = get_heading_from_attitude_matrix(held);
	pitch = get_pitch_from_attitude_matrix(held);
	roll = get_roll_from_attitude_matrix(held);

	v.x = mv[0];
	v.y = mv[1];
	v.z = mv[2];

	speed = get_3d_vector_magnitude(&v);

	normalise_3d_vector_given_magnitude(&v, speed);

	if (wide)
	{
		speed -= (4.0 * dt);
	}
	else
	{
		speed -= 4.0f * dt;
	}

	speed = (0.0f > speed) ? 0.0f : speed;

	v.x *= speed;
	v.y *= speed;
	v.z *= speed;

	heading -= PI * 0.1 * dt;

	out_pos[0] = candidate_xz[0];
	out_pos[1] = elevation + cg;
	out_pos[2] = candidate_xz[1];
	out_mv[0] = v.x;
	out_mv[1] = v.y;
	out_mv[2] = v.z;
	*out_speed = speed;

	get_3d_transformation_matrix(m, heading, pitch, roll);
	memcpy(out_attitude, m, sizeof(float) * 9);
}

/* routed/AA falling: full candidate pos + mv * dt, all axes, no map bound. */
void eech_gv_fall_candidate(const float *pos, const float *mv, float dt, float *out_pos)
{
	out_pos[0] = pos[0] + mv[0] * dt;
	out_pos[1] = pos[1] + mv[1] * dt;
	out_pos[2] = pos[2] + mv[2] * dt;
}

/* gravity after the DLL terrain refresh; returns the new speed. */
float eech_gv_fall_gravity(float *mv, float gravity, float dt)
{
	vec3d v;

	mv[1] -= gravity * dt;

	v.x = mv[0];
	v.y = mv[1];
	v.z = mv[2];

	return get_3d_vector_magnitude(&v);
}

/*
 * routed/AA falling settle. terrain_class: 1 water-like (original
 * TERRAIN_CLASS_WATER), anything else solid ground. Returns bit 0 = contact,
 * bit 1 = splash (water contact with pos.y > old_elevation, server effect
 * stays with the DLL), bit 2 = sunk (water, altitude < -|bbox diagonal|, mv
 * zeroed and the DLL sets DEAD unconditionally), bit 3 = ground impact effect
 * (solid contact; server effect stays with the DLL). `out_speed` is the magnitude
 * of the settled vector, the scalar the commit ends with.
 */
int eech_gv_fall_settle(
	const float *pos, const float *candidate, float *mv, int is_water,
	float old_elevation, float elevation, float cg, float bbox_diagonal,
	float *out_pos, float *out_speed)
{
	vec3d v;
	float altitude;
	int flags = 0;

	altitude = candidate[1] - (elevation + cg);

	out_pos[0] = candidate[0];
	out_pos[1] = candidate[1];
	out_pos[2] = candidate[2];

	if (altitude <= 0.0f)
	{
		flags |= 1;

		out_pos[0] = pos[0];
		out_pos[2] = pos[2];

		if (is_water)
		{
			if (pos[1] > old_elevation)
			{
				flags |= 2;
			}

			if (altitude < -bbox_diagonal)
			{
				mv[0] = 0.0f;
				mv[1] = 0.0f;
				mv[2] = 0.0f;

				flags |= 4;
			}
		}
		else
		{
			out_pos[1] = elevation + cg;

			mv[1] = 0.0f;

			flags |= 8;
		}
	}

	v.x = mv[0];
	v.y = mv[1];
	v.z = mv[2];

	*out_speed = get_3d_vector_magnitude(&v);

	return flags;
}

/*
 * routed/AA impact: the DLL draws `4.0 * frand1x(&seed)` into a float `speed`
 * (the original stores the double to float before the multiply), and sends that
 * float: float * float, as original.
 */
void eech_gv_impact(float *mv, float speed, const float *xv)
{
	mv[0] += (speed * xv[0]);
	mv[2] += (speed * xv[2]);
}

/*
 * ship death sink (no terrain refresh, no gate). Order as original:
 * candidate pos + mv * dt (all axes, no map bound), then horizontal drag
 * 0.2, sink rate, speed, then pitch/roll drift (heading unchanged).
 */
void eech_ship_sink(
	const float *pos, const float *mv, const float *attitude, float dt,
	float *out_pos, float *out_mv, float *out_speed, float *out_attitude)
{
	vec3d v;
	matrix3x3 m;
	float heading;
	float pitch;
	float roll;
	const float (*held)[3] = (const float (*)[3]) attitude;

	heading = get_heading_from_attitude_matrix(held);
	pitch = get_pitch_from_attitude_matrix(held);
	roll = get_roll_from_attitude_matrix(held);

	out_pos[0] = pos[0] + mv[0] * dt;
	out_pos[1] = pos[1] + mv[1] * dt;
	out_pos[2] = pos[2] + mv[2] * dt;

	v.x = mv[0];
	v.y = mv[1];
	v.z = mv[2];

	v.x -= (v.x * 0.2 * dt);
	v.z -= (v.z * 0.2 * dt);
	v.y -= (SHIP_SINK_RATE * dt);

	out_mv[0] = v.x;
	out_mv[1] = v.y;
	out_mv[2] = v.z;

	*out_speed = get_3d_vector_magnitude(&v);

	pitch += rad(0.1) * dt;
	roll += rad(0.5) * dt;

	get_3d_transformation_matrix(m, heading, pitch, roll);
	memcpy(out_attitude, m, sizeof(float) * 9);
}
