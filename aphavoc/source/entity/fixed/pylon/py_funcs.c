

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_pylon_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_PYLON);

	overload_pylon_create_functions ();

	overload_pylon_destroy_functions ();

	overload_pylon_draw_functions ();

	overload_pylon_float_value_functions ();

	overload_pylon_int_value_functions ();

	overload_pylon_pack_functions ();

	overload_pylon_ptr_value_functions ();

	initialise_pylon_database();
}




