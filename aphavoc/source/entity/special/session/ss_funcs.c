

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_session_functions (void)
{
	overload_session_create_functions ();

	overload_session_destroy_functions ();

	overload_session_float_value_functions ();

	overload_session_int_value_functions ();

	overload_session_pack_functions ();

	overload_session_list_functions ();

	overload_session_update_functions ();

	overload_session_vec3d_functions ();

	set_session_entity (NULL);
}




