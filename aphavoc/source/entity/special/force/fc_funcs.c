

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_force_functions (void)
{
	overload_force_create_functions ();

	overload_force_destroy_functions ();

	overload_force_int_value_functions ();

	overload_force_float_value_functions ();

	overload_force_list_functions ();

	overload_force_pack_functions ();

	overload_force_string_functions ();

	overload_force_update_functions ();
}




