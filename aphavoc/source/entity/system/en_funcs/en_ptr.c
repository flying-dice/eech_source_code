

































































#include "project.h"





#define DEBUG_MODULE 0





ptr_type_data
	ptr_type_database[NUM_PTR_TYPES] =
	{
		{
			"PTR_TYPE_EXTERNAL_VIEW_ENTITY",				// name
		},
		{
			"PTR_TYPE_GROUP_LEADER",						// name
		},
		{
			"PTR_TYPE_INSTANCE_3D_OBJECT",  				// name
		},
		{
			"PTR_TYPE_POSITION_ENTITY",					// name
		},
		{
			"PTR_TYPE_PROTOTYPE_1",				  			// name
		},
		{
			"PTR_TYPE_PROTOTYPE_2",			 				// name
		},
		{
			"PTR_TYPE_PROTOTYPE_3",							// name
		},
		{
			"PTR_TYPE_PROTOTYPE_4",							// name
		},
		{
			"PTR_TYPE_RETURN_KEYSITE",						// name
		},
		{
			"PTR_TYPE_ROUTE_DEPENDENTS",					// name
		},
		{
			"PTR_TYPE_ROUTE_FORMATION_TYPES",			// name
		},
		{
			"PTR_TYPE_ROUTE_NODE",							// name
		},
		{
			"PTR_TYPE_ROUTE_WAYPOINT_TYPES",				// name
		},
		{
			"PTR_TYPE_SUCC",									// name
		},
		{
			"PTR_TYPE_SOUND_EFFECT_DATA",					// name
		},
		{
			"PTR_TYPE_TASK_LEADER",							// name
		},
		{
			"PTR_TYPE_TASK_OBJECTIVE",						// name
		},
		{
			"PTR_TYPE_TERRAIN_FACE_NORMAL",				// name
		},
		{
			"PTR_TYPE_TERRAIN_INFO",						// name
		},
		{
			"PTR_TYPE_WEAPON_PACKAGE_STATUS_ARRAY",	// name
		},
	};





const char
	*overload_invalid_ptr_type_message = "Overloaded entity function invoked with invalid ptr type",
	*debug_fatal_invalid_ptr_type_message = "Invalid ptr type (entity type = %s, index = %d, ptr type = %s, file = %s, line = %d)";





void (*fn_set_local_entity_ptr_value[NUM_ENTITY_TYPES][NUM_PTR_TYPES]) (entity *en, ptr_types type, void *ptr);

void *(*fn_get_local_entity_ptr_value[NUM_ENTITY_TYPES][NUM_PTR_TYPES]) (entity *en, ptr_types type);





static void default_set_entity_ptr_value (entity *en, ptr_types type, void *ptr)
{
}





static void *default_get_entity_ptr_value (entity *en, ptr_types type)
{
	entity
		*ptr;

	switch (type)
	{

		default:

		{
			ptr = NULL;

			break;
		}
	}

	return (ptr);
}





void initialise_entity_ptr_value_default_functions (void)
{
	int
		i,
		j;

	for (i = 0; i < NUM_ENTITY_TYPES; i++)
	{
		for (j = 0; j < NUM_PTR_TYPES; j++)
		{
			fn_set_local_entity_ptr_value[i][j] = default_set_entity_ptr_value;

			fn_get_local_entity_ptr_value[i][j] = default_get_entity_ptr_value;
		}
	}
}




