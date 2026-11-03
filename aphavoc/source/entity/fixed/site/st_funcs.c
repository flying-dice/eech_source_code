

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_site_functions (void)
{
	overload_fixed_functions (ENTITY_TYPE_SITE);

	overload_site_create_functions ();

	overload_site_destroy_functions ();

	overload_site_draw_functions ();

	overload_site_float_value_functions ();

	overload_site_int_value_functions ();

	overload_site_list_functions ();

	overload_site_pack_functions ();
}




