

































































#include "project.h"





#define DEBUG_MODULE 0





void overload_person_functions (void)
{
	overload_vehicle_functions (ENTITY_TYPE_PERSON);

	overload_person_create_functions ();

	overload_person_destroy_functions ();

	overload_person_draw_functions ();

	overload_person_float_value_functions ();

	overload_person_int_value_functions ();

	overload_person_pack_functions ();

	overload_person_update_functions ();
}




