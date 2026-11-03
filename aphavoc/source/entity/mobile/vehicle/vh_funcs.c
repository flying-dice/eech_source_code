

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_vehicle_functions (entity_types type)
{
	overload_mobile_functions (type);

	overload_vehicle_float_value_functions (type);

	overload_vehicle_int_value_functions (type);

	overload_vehicle_list_functions (type);

	overload_vehicle_ptr_value_functions (type);

	overload_vehicle_string_functions (type);

	overload_vehicle_vec3d_functions (type);
}




