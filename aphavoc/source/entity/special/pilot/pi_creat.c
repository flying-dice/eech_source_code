

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	pilot
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

		raw = (pilot *) malloc_fast_mem (sizeof (pilot));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (pilot));

		strcpy (raw->pilots_name, "UNKNOWN");

		raw->sub_type = ENTITY_SUB_TYPE_PILOT_PILOT;

		raw->crew_role = CREW_ROLE_PILOT;

		raw->difficulty_level = GAME_DIFFICULTY_INVALID;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->pilot_link.parent);


		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//


		if (raw->aircrew_link.parent)
		{

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_AIRCREW, raw->aircrew_link.parent, raw->aircrew_link.child_pred);
		}

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_PILOT, raw->pilot_link.parent, raw->pilot_link.child_pred);











	}

	return (en);
}




















































































