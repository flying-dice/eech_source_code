

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	division
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

		raw = (division *) malloc_fast_mem (sizeof (division));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (division));

		raw->sub_type = ENTITY_SUB_TYPE_UNINITIALISED;

		strcpy (raw->division_name, "UNKNOWN");


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (entity_sub_type_division_valid (raw->sub_type));


		//
		// RESOLVE DEFAULT VALUES
		//



		//
		// LINK INTO SYSTEM
		//


		ASSERT (raw->division_link.parent);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_DIVISION, raw->division_link.parent, NULL);

		if (raw->division_headquarters_link.parent)
		{
			insert_local_entity_into_parents_child_list (en, LIST_TYPE_DIVISION_HEADQUARTERS, raw->division_headquarters_link.parent, NULL);
		}

		set_local_division_name (en, raw->division_name);
	}

	return (en);
}




















































































