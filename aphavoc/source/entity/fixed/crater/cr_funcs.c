

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_crater_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_CRATER);

	overload_crater_create_functions ();

	overload_crater_destroy_functions ();

	overload_crater_draw_functions ();

	overload_crater_float_value_functions ();

	overload_crater_list_functions ();

	overload_crater_pack_functions ();

	overload_crater_update_functions ();
}




