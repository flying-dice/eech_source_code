/*
 * The physical half of a mobile's initial placement (faction.c: create_landed_faction_group, create_landing_faction_members,
 * create_faction_members).  The campaign decides where a force is placed and which way it faces (the heading, the formation
 * position, the keysite); the host owns the physical state the placement produces.  This is the attitude the engine took
 * from the terrain face under the mobile: the face-normal frame at the heading, its pitch lifted by the airframe's fuselage
 * angle, and its roll, rebuilt as the mobile's attitude matrix.  Every expression is kept from the engine so the host and
 * the original reference produce the same bits.
 *
 * Not here, and still campaign-side coupling: the terrain sample itself (the elevation and the face normal under the
 * mobile, the shared terrain service), the formation arithmetic and the heading choice.
 */
#include "eech_arith.h"

static void place_face_normal_matrix(matrix3x3 result, const vec3d *normal, float heading)
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
 * The placement attitude: `angles` receives the heading, pitch and roll the engine's angle setter takes (so the campaign
 * keeps the original setter, and with it the multiplayer message), and `attitude` the matrix those angles build.
 */
void eech_place_attitude(const float *face_normal, float heading, float fuselage_angle, float *angles, float *attitude)
{
	matrix3x3 frame;
	matrix3x3 *matrix = (matrix3x3 *)attitude;
	float pitch, roll;

	place_face_normal_matrix(frame, (const vec3d *)face_normal, heading);

	pitch = get_pitch_from_attitude_matrix(frame);

	pitch += fuselage_angle;

	roll = get_roll_from_attitude_matrix(frame);

	angles[0] = heading;
	angles[1] = pitch;
	angles[2] = roll;

	get_3d_transformation_matrix(*matrix, heading, pitch, roll);
}
