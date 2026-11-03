

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_fixed_functions (entity_types type)
{
	overload_fixed_float_value_functions (type);

	overload_fixed_int_value_functions (type);

	overload_fixed_list_functions (type);

	overload_fixed_string_functions (type);

	overload_fixed_vec3d_functions (type);
}




