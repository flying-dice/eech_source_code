/*
 * The physical half of a carried child's repositioning (the MOVEMENT_DEPENDENT list: faction.c attaches a force's actual
 * members to the vehicle or aircraft that carries them; vh_vec3d.c and ac_vec3d.c notify each dependent, before the parent's
 * position is written, with the parent's NEW position, heading and velocity).  The campaign owns the attachment, the order of
 * the notifications and which children exist; the host owns the pose they are carried to.
 *
 * Only the helicopter's handler (hc_msgs.c: response_to_set_entity_position) does arithmetic: it keeps the child's offset from
 * the parent's OLD position, turns it by the parent's heading change, and adopts the parent's heading and velocity.  The other
 * dependents (routed, anti-air, person, cargo) copy the parent's new position and are handled by the host without arithmetic.
 * Every expression is kept from the handler, with its types, so the host and the original produce the same bits.
 *
 * `position` and `attitude` are the child's pose before the carry; `parent_old` the parent's position the notification
 * still sees stored; `parent_new` the position being written.  The heading setter rebuilds the attitude from the child's old
 * pitch and roll and the new heading (mb_float.c FLOAT_TYPE_HEADING); the velocity is adopted by the caller.
 */
#include "eech_arith.h"

void eech_dependent_helicopter(const float *parent_old, const float *parent_new, float heading, const float *position, const float *attitude, float *out_position, float *out_attitude)
{
	const vec3d *old_parent = (const vec3d *)parent_old;
	const vec3d *new_parent = (const vec3d *)parent_new;
	const vec3d *child = (const vec3d *)position;
	const matrix3x3 *child_attitude = (const matrix3x3 *)attitude;
	matrix3x3 *result = (matrix3x3 *)out_attitude;
	vec3d delta_position, new_position;
	float en_heading, pitch, roll;

	en_heading = get_heading_from_attitude_matrix(*child_attitude);

	delta_position.x = child->x - old_parent->x;
	delta_position.y = child->y - old_parent->y;
	delta_position.z = child->z - old_parent->z;

	new_position.x = new_parent->x + ((cos(heading - en_heading) * delta_position.x) + (sin(heading - en_heading) * delta_position.z));
	new_position.y = new_parent->y + delta_position.y;
	new_position.z = new_parent->z + ((cos(heading - en_heading) * delta_position.z) - (sin(heading - en_heading) * delta_position.x));

	out_position[0] = new_position.x;
	out_position[1] = new_position.y;
	out_position[2] = new_position.z;

	pitch = get_pitch_from_attitude_matrix(*child_attitude);
	roll = get_roll_from_attitude_matrix(*child_attitude);

	get_3d_transformation_matrix(*result, heading, pitch, roll);
}
