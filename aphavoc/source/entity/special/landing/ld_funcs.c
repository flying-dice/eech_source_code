

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_landing_functions (void)
{
	overload_landing_create_functions ();

	overload_landing_destroy_functions ();

	overload_landing_float_value_functions ();

	overload_landing_int_value_functions ();

	overload_landing_list_functions ();

	overload_landing_pack_functions ();

	overload_landing_string_functions ();

	overload_landing_update_functions ();

	overload_landing_vec3d_functions ();
}




