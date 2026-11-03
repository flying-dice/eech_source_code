

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_keysite_functions (void)
{
	overload_keysite_create_functions ();

	overload_keysite_destroy_functions ();

	overload_keysite_float_value_functions ();

	overload_keysite_int_value_functions ();

	overload_keysite_list_functions ();

	overload_keysite_pack_functions ();

	overload_keysite_string_functions ();

	overload_keysite_update_functions ();

	overload_keysite_vec3d_functions ();
}




