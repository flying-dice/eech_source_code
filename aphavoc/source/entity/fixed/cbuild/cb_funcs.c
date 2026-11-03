

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_city_building_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_CITY_BUILDING);

	overload_city_building_create_functions ();

	overload_city_building_destroy_functions ();

	overload_city_building_draw_functions ();

	overload_city_building_float_value_functions ();

	overload_city_building_int_value_functions ();

	overload_city_building_list_functions ();

	overload_city_building_pack_functions ();
}




