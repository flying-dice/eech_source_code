

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_bridge_functions (void)
{
	overload_bridge_create_functions ();

	overload_bridge_destroy_functions ();

	overload_bridge_float_value_functions ();

	overload_bridge_int_value_functions ();

	overload_bridge_list_functions ();

	overload_bridge_pack_functions ();

	overload_bridge_string_functions ();

	overload_bridge_vec3d_functions ();
}




