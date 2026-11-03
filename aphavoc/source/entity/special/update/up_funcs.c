

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_update_functions (void)
{
	overload_update_create_functions ();

	overload_update_destroy_functions ();

	overload_update_pack_functions ();

	overload_update_list_functions ();

	set_update_entity (NULL);

	set_update_succ (NULL);
}




