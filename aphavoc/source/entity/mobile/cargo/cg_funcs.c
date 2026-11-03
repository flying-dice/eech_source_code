

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_cargo_functions (void)
{
	overload_mobile_functions (ENTITY_TYPE_CARGO);

	overload_cargo_create_functions ();

	overload_cargo_destroy_functions ();

	overload_cargo_draw_functions ();

	overload_cargo_int_value_functions ();

	overload_cargo_list_functions ();

	overload_cargo_pack_functions ();

	overload_cargo_ptr_value_functions ();

	overload_cargo_update_functions ();
}




