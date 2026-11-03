

































































#include "project.h"





#define DEBUG_MODULE 0





void initialise_entity_system (int num_entities)
{
	initialise_entity_heap (num_entities);

	initialise_entity_databases ();

	initialise_entity_type_names ();

	initialise_entity_functions ();

	initialise_entity_message_responses ();

	initialise_entity_system_statistics ();

	initialise_debug_3d_objects ();

	initialise_debug_3d_lines ();

	enable_entity_comms_messages ();
}





void reinitialise_entity_system (void)
{
	destroy_local_entities ();

	reinitialise_entity_heap ();

	display_entity_system_statistics ();

	initialise_entity_functions ();

	initialise_entity_message_responses ();

	initialise_entity_system_statistics ();

	enable_entity_comms_messages ();
}





void deinitialise_entity_system (void)
{
	display_entity_system_statistics ();

	deinitialise_entity_type_names ();

	deinitialise_entity_heap ();

	deinitialise_entity_databases ();

	deinitialise_debug_3d_objects ();

	deinitialise_debug_3d_lines ();
}




