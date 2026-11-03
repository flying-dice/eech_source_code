

































































#include "project.h"





#define DEBUG_MODULE 0




//
// Create Entity Calling Conventions
// =================================
//
// There are two ways of invoking the create entity functions, depending on the comms data flow.
//
// If transmitting then the variable argument list is on the stack, as normal.
//
// If receiving then the variable argument list is in a char buffer, passed as the first and only variable argument.
//
// The create entity functions convert both types into a char buffer for the overloaded create functions.
//
// Entity Index Rules
// ==================
//
// Logic table for local entity index:
//
// 	SERVER/TX: index == ENTITY_INDEX_DONT_CARE
// 	SERVER/RX: index == ENTITY_INDEX_DONT_CARE
// 	CLIENT/TX: ILLEGAL
// 	CLIENT/RX: index != ENTITY_INDEX_DONT_CARE
//
// Logic table for remote entity index:
//
// 	SERVER/TX: index != ENTITY_INDEX_DONT_CARE
// 	SERVER/RX: index != ENTITY_INDEX_DONT_CARE
// 	CLIENT/TX: index == ENTITY_INDEX_DONT_CARE
// 	CLIENT/RX: ILLEGAL
//




entity *(*fn_create_local_entity[NUM_ENTITY_TYPES]) (entity_types type, int index, char *pargs);

entity *(*fn_create_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity_types type, int index, char *pargs);





//
// force local create function to accept stack based attributes during receive - use with caution!!!
//

int
	force_local_entity_create_stack_attributes = FALSE;





static entity *default_create_entity (entity_types type, int index, char *pargs)
{
	debug_fatal ("default_create_entity invoked: %s (index = %d)", get_entity_type_name (type), index);

	return (NULL);
}





void initialise_entity_create_default_functions (void)
{
	int
		i;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		fn_create_local_entity[i]										= default_create_entity;

		fn_create_client_server_entity[i][COMMS_MODEL_SERVER]	= default_create_entity;

		fn_create_client_server_entity[i][COMMS_MODEL_CLIENT]	= default_create_entity;
	}

	force_local_entity_create_stack_attributes = FALSE;
}





entity *create_local_entity (entity_types type, int index, ...)
{
	va_list
		pargs;

	char
		*pargs_buffer;

	entity
		*en;

	ASSERT ((type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES));

	//
	// safe guard - should never need to force stack attributes during transmit
	//

	ASSERT (!((get_comms_data_flow () == COMMS_DATA_FLOW_TX) && force_local_entity_create_stack_attributes));

	//
	// update statistics
	//

	update_create_entity_statistics (type);

	//
	// sort argument list
	//

	va_start (pargs, index);

	if ((get_comms_data_flow () == COMMS_DATA_FLOW_TX) || force_local_entity_create_stack_attributes)
	{
		#ifdef __WATCOMC__

		pargs_buffer = (char *) *pargs;

		#else

		pargs_buffer = (char *) pargs;

		#endif
	}
	else
	{
		pargs_buffer = va_arg (pargs, char *);
	}

	va_end (pargs);

	//
	// create
	//

	en = NULL;

	en = fn_create_local_entity[type] (type, index, pargs_buffer);

	if (!en)
	{

		debug_fatal ("EN_CREATE: CREATE_LOCAL_ENTITY : unable to create entity %s. Limit of %d reached", get_entity_type_name (type), number_of_entities);
	}


































































	return (en);
}





entity *create_client_server_entity (entity_types type, int index, ...)
{
	va_list
		pargs;

	char
		*pargs_buffer;

	entity
		*en;

	ASSERT ((type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES));

	//
	// safe guard - should never need to force stack attributes during transmit
	//

	ASSERT (!((get_comms_data_flow () == COMMS_DATA_FLOW_TX) && force_local_entity_create_stack_attributes));

	//
	// update statistics
	//

	update_create_entity_statistics (type);

	//
	// sort argument list
	//

	va_start (pargs, index);

	if ((get_comms_data_flow () == COMMS_DATA_FLOW_TX) || force_local_entity_create_stack_attributes)
	{
		#ifdef __WATCOMC__

		pargs_buffer = (char *) *pargs;

		#else

		pargs_buffer = (char *) pargs;

		#endif
	}
	else
	{
		pargs_buffer = va_arg (pargs, char *);
	}

	va_end (pargs);

	//
	// create
	//

	en = NULL;

	en = fn_create_client_server_entity[type][get_comms_model ()] (type, index, pargs_buffer);

	if ((get_comms_model () == COMMS_MODEL_SERVER) && (!en))
	{

		debug_fatal ("EN_CREATE: CREATE_CLIENT_SERVER_ENTITY : unable to create entity %s. Limit of %d reached", get_entity_type_name (type), number_of_entities);
	}
























	return (en);
}





void create_local_only_entities (pack_modes pack_mode)
{
	comms_model_types
		temp_comms_model;

	comms_data_flow_types
		temp_comms_data_flow;

	//
	// store comms model and data flow
	//

	temp_comms_model = get_comms_model ();

	temp_comms_data_flow = get_comms_data_flow ();

	//
	// set comms model and data flow to SERVER/TX
	//

	set_comms_model (COMMS_MODEL_SERVER);

	set_comms_data_flow (COMMS_DATA_FLOW_TX);

	disable_entity_comms_messages ();

	//
	// create local only entities
	//

	create_local_sector_entities ();

	create_local_pylon_entities (pack_mode);

	create_local_bridge_entities (pack_mode);

	create_local_update_entity ();

	// create camera entity now so that it is forced to the end of the update list (updated last)

	create_local_camera_entity ();

	//
	// restore comms model and data flow
	//

	set_comms_model (temp_comms_model);

	set_comms_data_flow (temp_comms_data_flow);

	enable_entity_comms_messages ();
}




