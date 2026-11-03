

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	force
		*raw;

	int
		loop;


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

		raw = (force *) malloc_fast_mem (sizeof (force));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (force));

		raw->side = ENTITY_SIDE_UNINITIALISED;

		raw->force_attitude = ENTITY_FORCE_ATTITUDE_NORMAL;

		for (loop = 0; loop < NUM_ENTITY_SUB_TYPE_TASKS; loop ++)
		{
			raw->task_generation [loop].valid = TRUE;
		}


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->force_link.parent);

		ASSERT (get_local_entity_type (raw->force_link.parent) == ENTITY_TYPE_SESSION);

		ASSERT (raw->side != ENTITY_SIDE_UNINITIALISED);


		//
		// RESOLVE DEFAULT VALUES
		//


		strcpy (raw->force_name, entity_side_short_names [raw->side]);

		set_force_hardware_update (TRUE);

		for (loop = 0; loop < NUM_FORCE_INFO_CATAGORIES; loop ++)
		{

			raw->force_info_reserve_hardware [loop] = 0;
		}


		//
		// LINK INTO SYSTEM
		//


		insert_local_entity_into_parents_child_list (en, LIST_TYPE_FORCE, raw->force_link.parent, raw->force_link.child_pred);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
	}

	return (en);
}




















































































