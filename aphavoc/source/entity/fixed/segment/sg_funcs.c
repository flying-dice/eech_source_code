

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_segment_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_SEGMENT);

	overload_segment_create_functions ();

	overload_segment_destroy_functions ();

	overload_segment_draw_functions ();

	overload_segment_float_value_functions ();

	overload_segment_int_value_functions ();

	overload_segment_list_functions ();

	overload_segment_pack_functions ();

	overload_segment_update_functions ();
}




