

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_ship_vehicle_functions (void)
{
	overload_vehicle_functions (ENTITY_TYPE_SHIP_VEHICLE);

	overload_ship_vehicle_create_functions ();

	overload_ship_vehicle_destroy_functions ();

	overload_ship_vehicle_draw_functions ();

	overload_ship_vehicle_float_value_functions ();

	overload_ship_vehicle_int_value_functions ();

	overload_ship_vehicle_pack_functions ();

	overload_ship_vehicle_update_functions ();
}




