

































































#include "project.h"





#define DEBUG_MODULE 0




//
// The update entity is local only.
//
// It must be created before any client/server entities to ensure that the index number corresponds on each machine.
//




void create_local_update_entity (void)
{
	create_local_entity (ENTITY_TYPE_UPDATE, ENTITY_INDEX_DONT_CARE, ENTITY_ATTR_END);
}





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	update
		*raw;


  	//
  	// VALIDATE
  	//


	validate_local_create_entity_index (index);







	en = get_free_entity (index);

	ASSERT (!get_update_entity ());

	if (en)
	{

   	//
   	// MALLOC ENTITY DATA
   	//


		set_local_entity_type (en, type);

		raw = (update *) malloc_fast_mem (sizeof (update));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (update));


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


		set_update_entity (en);

		set_update_succ (NULL);
	}

	return (en);
}













