

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_helicopter_functions (void)
{
	overload_aircraft_functions (ENTITY_TYPE_HELICOPTER);

	overload_helicopter_create_functions ();

	overload_helicopter_destroy_functions ();

	overload_helicopter_draw_functions ();

	overload_helicopter_float_value_functions ();

	overload_helicopter_int_value_functions ();

	overload_helicopter_list_functions ();

	overload_helicopter_pack_functions ();

	overload_helicopter_update_functions ();

	overload_helicopter_vec3d_functions ();
}




