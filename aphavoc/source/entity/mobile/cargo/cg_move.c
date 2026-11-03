

































































#include "project.h"





#define DEBUG_MODULE 			0



///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////// /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void cargo_crash_movement (entity *en);

void clear_cargo_velocity (entity *en);





void basic_cargo_movement (entity *en)
{

	cargo
		*raw;

	raw = (cargo *) get_local_entity_data (en);
}





#define RIVER_DEPTH 3.0

void basic_cargo_death_movement (entity *en)
{

}





void cargo_crash_movement (entity *en)
{

}





void clear_cargo_velocity (entity *en)
{
	vec3d
		velocity;

	velocity.x = 0.0;
	velocity.y = 0.0;
	velocity.z = 0.0;

	set_local_entity_vec3d (en, VEC3D_TYPE_MOTION_VECTOR, &velocity);

	set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, 0.0);
}




