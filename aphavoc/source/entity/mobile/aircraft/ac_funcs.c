

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_aircraft_functions (entity_types type)
{
	overload_mobile_functions (type);

	overload_aircraft_float_value_functions (type);

	overload_aircraft_int_value_functions (type);

	overload_aircraft_list_functions (type);

	overload_aircraft_ptr_value_functions (type);

	overload_aircraft_string_functions (type);

	overload_aircraft_vec3d_functions (type);
}




