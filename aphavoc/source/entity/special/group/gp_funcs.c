

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_group_functions (void)
{
	overload_group_create_functions ();

	overload_group_destroy_functions ();

	overload_group_int_value_functions ();

	overload_group_float_value_functions ();

	overload_group_list_functions ();

	overload_group_pack_functions ();

	overload_group_ptr_value_functions ();

	overload_group_string_functions ();

	overload_group_update_functions ();

	overload_group_vec3d_functions ();
}




