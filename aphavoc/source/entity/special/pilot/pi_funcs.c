

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_pilot_functions (void)
{
	overload_pilot_create_functions ();

	overload_pilot_destroy_functions ();

	overload_pilot_int_value_functions ();

	overload_pilot_list_functions ();

	overload_pilot_pack_functions ();

	overload_pilot_string_functions ();
}




