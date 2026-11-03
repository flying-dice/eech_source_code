

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	segment
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

		raw = (segment *) malloc_fast_mem (sizeof (segment));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (segment));

		//
		// fixed
		//

		raw->fix.position.x = MID_MAP_X;
		raw->fix.position.y = MID_MAP_Y;
		raw->fix.position.z = MID_MAP_Z;

		raw->fix.sub_type = ENTITY_SUB_TYPE_FIXED_UNKNOWN;

		raw->fix.object_3d_shape = OBJECT_3D_INVALID_OBJECT_INDEX;

		raw->fix.alive = TRUE;

		raw->fix.side = ENTITY_SIDE_NEUTRAL;

		//
		// segment
		//


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->segment_link.parent);


		//
		// RESOLVE DEFAULT VALUES
		//


		raw->altitude = raw->fix.position.y;		// used when repairing segment

		if (raw->fix.object_3d_shape == OBJECT_3D_INVALID_OBJECT_INDEX)
		{
			raw->fix.object_3d_shape = fixed_database[raw->fix.sub_type].default_3d_shape;
		}


		//
		// BUILD COMPONENTS
		//



		//
		// LINK INTO SYSTEM
		//


		insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->fix.position), NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_SEGMENT, raw->segment_link.parent, raw->segment_link.child_pred);

	}

	return (en);
}




















































































