

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_routed_vehicle_functions (void)
{
	overload_vehicle_functions (ENTITY_TYPE_ROUTED_VEHICLE);

	overload_routed_vehicle_create_functions ();

	overload_routed_vehicle_destroy_functions ();

	overload_routed_vehicle_draw_functions ();

	overload_routed_vehicle_float_value_functions ();

	overload_routed_vehicle_int_value_functions ();

	overload_routed_vehicle_pack_functions ();

	//overload_routed_vehicle_list_functions ();

//	overload_routed_vehicle_sound_functions ();

	overload_routed_vehicle_update_functions ();
}




