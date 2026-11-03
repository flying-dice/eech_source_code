

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_sector_functions (void)
{
	overload_sector_create_functions ();

	overload_sector_destroy_functions ();

	overload_sector_float_value_functions ();

	overload_sector_int_value_functions ();

	overload_sector_list_functions ();

	overload_sector_pack_functions ();
}




