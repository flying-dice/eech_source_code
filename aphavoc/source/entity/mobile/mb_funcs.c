

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_mobile_functions (entity_types type)
{
	overload_mobile_attitude_functions (type);

	overload_mobile_float_value_functions (type);

	overload_mobile_int_value_functions (type);

	overload_mobile_list_functions (type);

	overload_mobile_vec3d_functions (type);
}




