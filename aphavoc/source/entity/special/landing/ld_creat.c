

































































#include "project.h"

#include "ai/ai_misc/ai_misc.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{

	entity
		*task,
		*en;

	landing
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

		raw = (landing *) malloc_fast_mem (sizeof (landing));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (landing));

		raw->position.x = MID_MAP_X;
		raw->position.y = MID_MAP_Y;
		raw->position.z = MID_MAP_Z;

		raw->free_landing_sites = 0;

		raw->total_landing_sites = 0;

		raw->reserved_landing_sites = 0;


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

		
		raw->free_landing_sites = raw->total_landing_sites;

		switch (raw->sub_type)
		{

			case ENTITY_SUB_TYPE_LANDING_FIXED_WING:
			case ENTITY_SUB_TYPE_LANDING_FIXED_WING_TRANSPORT:
			case ENTITY_SUB_TYPE_LANDING_HELICOPTER:
			{

				//raw->route_node = get_closest_air_node (&raw->position, 10.0);

				break;
			}

			case ENTITY_SUB_TYPE_LANDING_GROUND:
			{

				raw->route_node = get_closest_road_node (&raw->position, 10.0);

				break;
			}
			case ENTITY_SUB_TYPE_LANDING_SEA:
			{

				//raw->route_node = get_closest_sea_node (&raw->position, 10.0);

				break;
			}
		}


		//
		// LINK INTO SYSTEM
		//


		if (raw->landing_site_link.parent)
		{

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_LANDING_SITE, raw->landing_site_link.parent, NULL);
		}

















		//
		// CREATE SUB ENTITIES
		//


		task = create_local_entity
		(
			ENTITY_TYPE_TASK,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_ASSIGNED_TASK, en),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_TASK_STATE, TASK_STATE_ASSIGNED),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, raw->position.x, raw->position.y, raw->position.z),
			ENTITY_ATTR_END
		);

		task = create_local_entity
		(
			ENTITY_TYPE_TASK,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_ASSIGNED_TASK, en),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_TASK_LANDING_HOLDING),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_TASK_STATE, TASK_STATE_ASSIGNED),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, raw->position.x, raw->position.y, raw->position.z),
			ENTITY_ATTR_END
		);

		task = create_local_entity
		(
			ENTITY_TYPE_TASK,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_ASSIGNED_TASK, en),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_TASK_TAKEOFF),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_TASK_STATE, TASK_STATE_ASSIGNED),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, raw->position.x, raw->position.y, raw->position.z),
			ENTITY_ATTR_END
		);

		task = create_local_entity
		(
			ENTITY_TYPE_TASK,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_PARENT (LIST_TYPE_ASSIGNED_TASK, en),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_TASK_LANDING),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_TASK_STATE, TASK_STATE_ASSIGNED),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, raw->position.x, raw->position.y, raw->position.z),
			ENTITY_ATTR_END
		);
	}

	return (en);
}




















































































