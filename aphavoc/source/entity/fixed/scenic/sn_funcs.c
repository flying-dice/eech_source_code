

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_scenic_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_SCENIC);

	overload_scenic_create_functions ();

	overload_scenic_destroy_functions ();

	overload_scenic_draw_functions ();

	overload_scenic_float_value_functions ();

	overload_scenic_int_value_functions ();

	overload_scenic_pack_functions ();
}




