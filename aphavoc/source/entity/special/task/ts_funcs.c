

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_task_functions (void)
{
	overload_task_create_functions ();

	overload_task_destroy_functions ();

	overload_task_float_value_functions ();

	overload_task_int_value_functions ();

	overload_task_list_functions ();

	overload_task_pack_functions ();

	overload_task_ptr_value_functions ();

	overload_task_string_functions ();

	overload_task_update_functions ();

	overload_task_vec3d_functions ();
}




