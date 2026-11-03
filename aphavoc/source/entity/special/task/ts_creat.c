

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{

	entity
		*en;

	task
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

		raw = (task *) malloc_fast_mem (sizeof (task));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (task));

		raw->side = ENTITY_SIDE_UNINITIALISED;

		raw->task_state = TASK_STATE_UNASSIGNED;

		raw->task_terminated = TASK_TERMINATED_IN_PROGRESS;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//



		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//


		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

		if (raw->task_dependent_link.parent)
		{
			insert_local_entity_into_parents_child_list (en, LIST_TYPE_TASK_DEPENDENT, raw->task_dependent_link.parent, NULL);
		}

		if (raw->task_link.parent)
		{
			insert_local_entity_into_parents_child_list (en, get_local_task_list_type (en), raw->task_link.parent, NULL);
		}
	}

	return (en);
}




















































































