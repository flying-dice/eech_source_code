

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{

	entity
		*en;

	guide
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

		raw = (guide *) malloc_fast_mem (sizeof (guide));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (guide));

		raw->sub_type = ENTITY_SUB_TYPE_GUIDE_NAVIGATION_DIRECT;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->guide_link.parent);

		ASSERT (raw->current_waypoint_link.parent);


		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//


		insert_local_entity_into_parents_child_list (en, LIST_TYPE_GUIDE, raw->guide_link.parent, NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_CURRENT_WAYPOINT, raw->current_waypoint_link.parent, NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
	}

	return (en);
}




















































































