

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_site_updatable_functions (void)
{
   overload_fixed_functions (ENTITY_TYPE_SITE_UPDATABLE);

   overload_site_updatable_create_functions ();

   overload_site_updatable_destroy_functions ();

   overload_site_updatable_draw_functions ();

   overload_site_updatable_float_value_functions ();

   overload_site_updatable_int_value_functions ();

   overload_site_updatable_list_functions ();

	overload_site_updatable_pack_functions ();

   overload_site_updatable_update_functions ();
}




