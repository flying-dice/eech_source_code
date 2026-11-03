

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_regen_functions (void)
{
	overload_regen_create_functions ();

	overload_regen_destroy_functions ();

	overload_regen_float_value_functions ();

	overload_regen_int_value_functions ();

	overload_regen_list_functions ();

	overload_regen_pack_functions ();

	overload_regen_update_functions ();

	overload_regen_vec3d_functions ();
}




