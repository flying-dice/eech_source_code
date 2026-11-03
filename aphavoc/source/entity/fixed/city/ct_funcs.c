

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_city_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_CITY);

	overload_city_create_functions ();

	overload_city_destroy_functions ();

	overload_city_draw_functions ();

	overload_city_float_value_functions ();

	overload_city_int_value_functions ();

	overload_city_list_functions ();

	overload_city_pack_functions ();

	overload_city_vec3d_functions ();
}




