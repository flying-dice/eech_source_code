

































































#include "project.h"





#define DEBUG_MODULE				0

#define DEBUG_MODULE_PACK_ONE	0

#define DEBUG_MODULE_PACK_ALL	(PACK_ENTIRE_SESSION || 0)





const char
	*null_entity_pointer_message = "Overloaded entity function invoked with NULL entity pointer";





const char
	**entity_type_names;





entity_type_data
	entity_type_database[NUM_ENTITY_TYPES] =
	{
		{
			"ENTITY_TYPE_UNKNOWN",						// name
			0,													// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_ANTI_AIRCRAFT",				// name
			sizeof (anti_aircraft),						// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_BRIDGE",						// name
			sizeof (bridge),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_CAMERA",						// name
			sizeof (camera),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_CARGO",							// name
			sizeof (cargo),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_CITY",							// name
			sizeof (city),									// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_CITY_BUILDING",				// name
			sizeof (city_building),						// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_CRATER",						// name
			sizeof (crater),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_DIVISION",						// name
			sizeof (division),							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_EXPLOSION",					// name
			sizeof (explosion),							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_FIXED_WING",					// name
			sizeof (fixed_wing),							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_FORCE",							// name
			sizeof (force), 								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_GROUP",							// name
			sizeof (group),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_GUIDE",							// name
			sizeof (guide),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_HELICOPTER",					// name
			sizeof (helicopter),							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_KEYSITE",						// name
			sizeof (keysite),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_LANDING",						// name
			sizeof (landing),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_OBJECT",						// name
			sizeof (object), 								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_PARTICLE",						// name
			sizeof (particle), 							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_PERSON",						// name
			sizeof (person), 								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_PILOT",							// name
			sizeof (pilot),  								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_PYLON",							// name
			sizeof (pylon), 								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_REGEN",							// name
			sizeof (regen), 								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_ROUTED_VEHICLE",				// name
			sizeof (routed_vehicle), 					// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SCENIC",						// name
			sizeof (scenic), 								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SECTOR",						// name
			sizeof (sector),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SEGMENT",						// name
			sizeof (segment),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SESSION",						// name
			sizeof (session),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SHIP_VEHICLE",				// name
			sizeof (ship_vehicle),						// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SITE",							// name
			sizeof (site),									// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SITE_UPDATABLE",				// name
			sizeof (site_updatable),					// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SMOKE_LIST",					// name
			sizeof (smoke_list),							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SOUND_EFFECT",				// name
			sizeof (sound_effect),						// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_SPRITE",						// name
			sizeof (sprite),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_TASK",							// name
			sizeof (task),									// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_UPDATE",						// name
			sizeof (update),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_WAYPOINT",						// name
			sizeof (waypoint),							// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
		{
			"ENTITY_TYPE_WEAPON",						// name
			sizeof (weapon),								// size_of_data
			(DEBUG_MODULE_PACK_ALL || 0),				// debug_pack
		},
	};





void initialise_entity_type_names (void)
{

	int
		loop;

	entity_type_names = (const char **) malloc_heap_mem (sizeof (const char *) * NUM_ENTITY_TYPES + 1);

	for (loop = 0; loop < NUM_ENTITY_TYPES; loop ++)
	{

		entity_type_names [loop] = get_entity_type_name (loop);
	}
}





void deinitialise_entity_type_names (void)
{

	free_mem ((void*)entity_type_names);
}





void pack_entity_type (entity_types type)
{
	ASSERT ((type >= 0) && (type < NUM_ENTITY_TYPES));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (entity_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_PACK, ENTITY_DEBUG_ENTITY_TYPE, NULL, type);
	}

	#endif

	ASSERT(!(type >> NUM_ENTITY_TYPE_PACK_BITS));

	pack_unsigned_data (type, NUM_ENTITY_TYPE_PACK_BITS);
}





entity_types unpack_entity_type (void)
{
	entity_types
		type;

	type = (entity_types) unpack_unsigned_data (NUM_ENTITY_TYPE_PACK_BITS);

	ASSERT ((type >= 0) && (type < NUM_ENTITY_TYPES));

	#if (DEBUG_MODULE_PACK_ONE || DEBUG_MODULE_PACK_ALL)

	if (entity_type_database[type].debug_pack)
	{
		debug_log_entity_args (ENTITY_DEBUG_UNPACK, ENTITY_DEBUG_ENTITY_TYPE, NULL, type);
	}

	#endif

	return (type);
}




