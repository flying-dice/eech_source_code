

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_object_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_OBJECT);

	overload_object_create_functions ();

	overload_object_destroy_functions ();

	overload_object_draw_functions ();

	//overload_object_int_value_functions ();

	overload_object_pack_functions ();

	overload_object_vec3d_functions ();
}




