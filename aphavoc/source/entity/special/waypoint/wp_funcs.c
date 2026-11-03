

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_waypoint_functions (void)
{
	overload_waypoint_char_value_functions ();

	overload_waypoint_create_functions ();

	overload_waypoint_draw_functions ();

	overload_waypoint_destroy_functions ();

	overload_waypoint_float_value_functions ();

	overload_waypoint_int_value_functions ();

	overload_waypoint_list_functions ();

	overload_waypoint_pack_functions ();

	overload_waypoint_ptr_value_functions ();

	overload_waypoint_string_functions ();

	overload_waypoint_vec3d_functions ();
}




