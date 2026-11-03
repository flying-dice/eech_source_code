

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_fixed_wing_functions (void)
{
	overload_aircraft_functions (ENTITY_TYPE_FIXED_WING);

	overload_fixed_wing_create_functions ();

	overload_fixed_wing_destroy_functions ();

	overload_fixed_wing_draw_functions ();

	overload_fixed_wing_float_value_functions ();

	overload_fixed_wing_int_value_functions ();

	overload_fixed_wing_list_functions ();

	overload_fixed_wing_pack_functions ();

//	overload_fixed_wing_sound_functions ();

	overload_fixed_wing_update_functions ();
}




