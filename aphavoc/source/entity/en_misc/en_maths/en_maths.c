

































































#include "project.h"





#define DEBUG_MODULE 0





float get_local_entity_target_bearing (entity *source, entity *target, float *range)
{
	vec3d
		*source_pos,
		*target_pos;

	float
		dx,
		dz,
		bearing;

	ASSERT (source);

	ASSERT (target);

	source_pos = get_local_entity_vec3d_ptr (source, VEC3D_TYPE_POSITION);

	target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

	dx = target_pos->x - source_pos->x;
	dz = target_pos->z - source_pos->z;

	if (range)
	{
		*range = sqrt ((dx * dx) + (dz * dz));
	}

	bearing = atan2 (dx, dz);

	if (bearing < 0.0)
	{
		bearing += rad (360.0);
	}

	return (bearing);
}





float get_local_entity_target_relative_bearing (entity *source, entity *target)
{
	vec3d
		*source_pos,
		*target_pos;

	float
		dx,
		dz,
		bearing,
		heading,
		relative_bearing;

	ASSERT (source);

	ASSERT (target);

	source_pos = get_local_entity_vec3d_ptr (source, VEC3D_TYPE_POSITION);

	target_pos = get_local_entity_vec3d_ptr (target, VEC3D_TYPE_POSITION);

	dx = target_pos->x - source_pos->x;
	dz = target_pos->z - source_pos->z;

	bearing = atan2 (dx, dz);

	heading = get_local_entity_float_value (source, FLOAT_TYPE_HEADING);

	relative_bearing = bearing - heading;

	if (relative_bearing > rad (180.0))
	{
		relative_bearing -= rad (360.0);
	}
	else if (relative_bearing < rad (-180.0))
	{
		relative_bearing += rad (360.0);
	}

	return (relative_bearing);
}





























































































