

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_anti_aircraft_functions (void)
{
	overload_vehicle_functions (ENTITY_TYPE_ANTI_AIRCRAFT);

	overload_anti_aircraft_create_functions ();

	overload_anti_aircraft_destroy_functions ();

	overload_anti_aircraft_draw_functions ();

	overload_anti_aircraft_int_value_functions ();

	overload_anti_aircraft_pack_functions ();

//	overload_anti_aircraft_sound_functions ();

	overload_anti_aircraft_update_functions ();
}




