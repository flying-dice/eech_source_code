

































































#include "project.h"
#include "ai/highlevl/imaps.h"







#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{

	char
		 name [STRING_TYPE_KEYSITE_NAME_MAX_LENGTH];

	entity
		*group,
		*force,
		*sector,
		*en;

	keysite
		*raw;


  	//
  	// VALIDATE
  	//


	validate_local_create_entity_index (index);







	en = get_free_entity (index);

	if (en)
	{

   	//
   	// MALLOC ENTITY DATA
   	//


		set_local_entity_type (en, type);

		raw = (keysite *) malloc_fast_mem (sizeof (keysite));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (keysite));

		sprintf (name, "KEYSITE %d", (int) en % 100);

		strncpy (raw->keysite_name, name, STRING_TYPE_KEYSITE_NAME_MAX_LENGTH);

		//
		// fixed
		//

		raw->position.x = MID_MAP_X;
		raw->position.y = MID_MAP_Y;
		raw->position.z = MID_MAP_Z;

		raw->alive = TRUE;

		raw->keysite_usable_state = KEYSITE_STATE_USABLE;

		raw->ndb_frequency = 0;

		raw->in_use = FALSE;
		raw->object_index = OBJECT_3D_INVALID_OBJECT_INDEX;

		raw->side = ENTITY_SIDE_NEUTRAL;

		raw->supplies.ammo_supply_level = 0.0;

		raw->supplies.fuel_supply_level = 0.0;

		raw->assign_timer = frand1 () * KEYSITE_TASK_ASSIGN_TIMER;		// SERVER ONLY - OK TO USE RANDOM

		raw->sleep = frand1 () * KEYSITE_UPDATE_SLEEP_TIMER;				// SERVER ONLY - OK TO USE RANDOM


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->side != ENTITY_SIDE_NEUTRAL);

		ASSERT (entity_sub_type_keysite_valid (raw->sub_type));

		ASSERT (keysite_database [raw->sub_type].minimum_efficiency < 1.0);

		// the following is currently required for the campaign to progress properly...
		ASSERT (keysite_database [raw->sub_type].repairable == keysite_database [raw->sub_type].troop_insertion_target);


		//
		// RESOLVE DEFAULT VALUES
		//


		update_keysite_cargo (en, raw->supplies.ammo_supply_level, ENTITY_SUB_TYPE_CARGO_AMMO, CARGO_AMMO_SIZE);

		update_keysite_cargo (en, raw->supplies.fuel_supply_level, ENTITY_SUB_TYPE_CARGO_FUEL, CARGO_FUEL_SIZE);






		//
		// BUILD COMPONENTS
		//



		//
		// LINK INTO SYSTEM
		//


		force = get_local_entity_parent (en, LIST_TYPE_KEYSITE_FORCE);

		debug_assert (get_local_entity_type (force) == ENTITY_TYPE_FORCE);

		ASSERT (force);

		sector = get_local_sector_entity (&raw->position);

		ASSERT (sector);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_KEYSITE_FORCE, force, NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, sector, NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

		set_local_entity_int_value (sector, INT_TYPE_KEYSITE_COUNT, get_local_entity_int_value (sector, INT_TYPE_KEYSITE_COUNT) + 1);

		if (raw->in_use)
		{
			update_imap_sector_side (en, TRUE);

			update_imap_importance_level (en, TRUE);

			update_keysite_distance_to_friendly_base (en, (entity_sides) raw->side);
		}


		//
		//	CREATE SUB ENTITIES
		//


		// for site buildings

		group = create_local_entity
		(
			ENTITY_TYPE_GROUP,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_BUILDING_GROUP, en),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, raw->position.x, raw->position.y, raw->position.z),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_GROUP_BUILDINGS),
			ENTITY_ATTR_END
		);













	}

	return (en);
}




















































































