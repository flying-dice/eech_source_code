

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));
	
	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:

		{
			
			vec3d
				position;
			
			waypoint
				*raw;
			
			raw = (waypoint *) get_local_entity_data (en);
			
			ASSERT (raw->waypoint_link.parent);
				
			ASSERT (get_local_entity_type (raw->waypoint_link.parent) == ENTITY_TYPE_TASK);
				
			//
			// do send task's waypoint list (not landing ones)
			//
					
			switch (get_local_entity_int_value (raw->waypoint_link.parent, INT_TYPE_ENTITY_SUB_TYPE))
			{
						
				case ENTITY_SUB_TYPE_TASK_LANDING:
				case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
				case ENTITY_SUB_TYPE_TASK_TAKEOFF:
				case ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING:
				{
							
					return;
				}
			}
			
			pack_entity_type (get_local_entity_type (en));
			
			pack_entity_safe_ptr (en);
			
			//
			// pack data (in exactly the same order as the data is unpacked)
			//
			
			pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);
			


			pack_list_link (en, LIST_TYPE_WAYPOINT, &raw->waypoint_link);


			

			if (raw->current_waypoint_root.first_child)
			{
				
				pack_int_value (en, INT_TYPE_VALID, TRUE);
				
				pack_list_root (en, LIST_TYPE_CURRENT_WAYPOINT, &raw->current_waypoint_root);
			}
			else
			{
				
				pack_int_value (en, INT_TYPE_VALID, FALSE);
			}

			

			if (raw->task_dependent_link.parent)
			{
				
				pack_int_value (en, INT_TYPE_VALID, TRUE);
				
				pack_list_link (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_link);
			}
			else
			{
				
				pack_int_value (en, INT_TYPE_VALID, FALSE);
			}

			
			pack_float_value (en, FLOAT_TYPE_ALTITUDE, raw->altitude);
			
			pack_float_value (en, FLOAT_TYPE_HEADING, raw->heading);
			
			pack_int_value (en, INT_TYPE_POSITION_TYPE, raw->position_type);
			
			pack_int_value (en, INT_TYPE_ROUTE_NODE, raw->route_node);
			
			pack_char_value (en, CHAR_TYPE_TAG, (char) raw->tag);
			
			pack_int_value (en, INT_TYPE_WAYPOINT_FORMATION, raw->waypoint_formation);
			
			switch (raw->position_type)
			{
				
				case POSITION_TYPE_RELATIVE:
				{
					
					debug_fatal ("WP_PACK: internal waypoint with relative position");
					
					break;
				}
				
				case POSITION_TYPE_ABSOLUTE:
				case POSITION_TYPE_VIRTUAL:
				{
					
					position = raw->position;
					
					pack_vec3d (en, VEC3D_TYPE_POSITION, &position);
					
					pack_entity_safe_ptr (raw->position_entity);
					
					break;
				}
			}
			
			break;
		}

		case PACK_MODE_CLIENT_SESSION:

		{
			
			vec3d
				position;
			
			waypoint
				*raw;
			
			raw = (waypoint *) get_local_entity_data (en);
			
			ASSERT (raw->waypoint_link.parent);
				
			ASSERT (get_local_entity_type (raw->waypoint_link.parent) == ENTITY_TYPE_TASK);
				
			//
			// do send task's waypoint list (not landing ones)
			//
					
			switch (get_local_entity_int_value (raw->waypoint_link.parent, INT_TYPE_ENTITY_SUB_TYPE))
			{
						
				case ENTITY_SUB_TYPE_TASK_LANDING:
				case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
				case ENTITY_SUB_TYPE_TASK_TAKEOFF:
				case ENTITY_SUB_TYPE_TASK_TAKEOFF_HOLDING:
				{
							
					return;
				}
			}
			
			pack_entity_type (get_local_entity_type (en));
			
			pack_entity_safe_ptr (en);
			
			//
			// pack data (in exactly the same order as the data is unpacked)
			//
			
			pack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, raw->sub_type);


			
			pack_list_link (en, LIST_TYPE_WAYPOINT, &raw->waypoint_link);
			
			pack_float_value (en, FLOAT_TYPE_ALTITUDE, raw->altitude);
			
			pack_int_value (en, INT_TYPE_POSITION_TYPE, raw->position_type);
			
			pack_int_value (en, INT_TYPE_ROUTE_NODE, raw->route_node);
			
			pack_char_value (en, CHAR_TYPE_TAG, (char) raw->tag);
			
			pack_int_value (en, INT_TYPE_WAYPOINT_FORMATION, raw->waypoint_formation);
			
			switch (raw->position_type)
			{
				
				case POSITION_TYPE_RELATIVE:
				{
					
					debug_fatal ("WP_PACK: internal waypoint with relative position");
					
					break;
				}
				
				case POSITION_TYPE_ABSOLUTE:
				case POSITION_TYPE_VIRTUAL:
				{
					
					position = raw->position;
					
					pack_vec3d (en, VEC3D_TYPE_POSITION, &position);
					
					pack_entity_safe_ptr (raw->position_entity);
					
					break;
				}
			}
			
			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			break;
		}
	}
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));
	
	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:

		{
			int
			index;
			
			waypoint
			*raw;
			
			//
			// create entity
			//
			
			index = unpack_entity_safe_index ();
			
			en = get_free_entity (index);
			
			set_local_entity_type (en, type);
			
			raw = (waypoint *) malloc_fast_mem (sizeof (waypoint));
			
			set_local_entity_data (en, raw);
			
			memset (raw, 0, sizeof (waypoint));
			
			//
			// unpack data (in exactly the same order as the data was packed)
			//
			
			raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			unpack_list_link (en, LIST_TYPE_WAYPOINT, &raw->waypoint_link);
			

			if (unpack_int_value (en, INT_TYPE_VALID))
			{
				
				unpack_list_root (en, LIST_TYPE_CURRENT_WAYPOINT, &raw->current_waypoint_root);
			}

			

			if (unpack_int_value (en, INT_TYPE_VALID))
			{
				
				unpack_list_link (en, LIST_TYPE_TASK_DEPENDENT, &raw->task_dependent_link);
			}

			
			raw->altitude = unpack_float_value (en, FLOAT_TYPE_ALTITUDE);
			
			raw->heading = unpack_float_value (en, FLOAT_TYPE_HEADING);
			
			raw->position_type = unpack_int_value (en, INT_TYPE_POSITION_TYPE);
			
			raw->route_node = unpack_int_value (en, INT_TYPE_ROUTE_NODE);
			
			raw->tag = unpack_char_value (en, CHAR_TYPE_TAG);
			
			raw->waypoint_formation = unpack_int_value (en, INT_TYPE_WAYPOINT_FORMATION);
			
			//
			//
			//
			
			switch (raw->position_type)
			{
				
				case POSITION_TYPE_RELATIVE:
				{
					
					debug_fatal ("WP_PACK: internal waypoint with relative position");
					
					break;
				}
				
				case POSITION_TYPE_ABSOLUTE:
				case POSITION_TYPE_VIRTUAL:
				{
					
					unpack_vec3d (en, VEC3D_TYPE_POSITION, &raw->position);
					
					index = unpack_entity_safe_index ();
					
					raw->position_entity = get_local_entity_safe_ptr (index);
					
					break;
				}
			}
			
			break;
		}

		case PACK_MODE_CLIENT_SESSION:

		{
			int
				index;
			
			waypoint
				*raw;
			
			//
			// create entity
			//
			
			index = unpack_entity_safe_index ();
			
			en = get_free_entity (index);
			
			set_local_entity_type (en, type);
			
			raw = (waypoint *) malloc_fast_mem (sizeof (waypoint));
			
			set_local_entity_data (en, raw);
			
			memset (raw, 0, sizeof (waypoint));
			
			//
			// unpack data (in exactly the same order as the data was packed)
			//
			
			raw->sub_type = unpack_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

			unpack_list_link (en, LIST_TYPE_WAYPOINT, &raw->waypoint_link);
			
			raw->altitude = unpack_float_value (en, FLOAT_TYPE_ALTITUDE);
			
			raw->position_type = unpack_int_value (en, INT_TYPE_POSITION_TYPE);
			
			raw->route_node = unpack_int_value (en, INT_TYPE_ROUTE_NODE);
			
			raw->tag = unpack_char_value (en, CHAR_TYPE_TAG);
			
			raw->waypoint_formation = unpack_int_value (en, INT_TYPE_WAYPOINT_FORMATION);
			
			//
			//
			//
			
			switch (raw->position_type)
			{
				
				case POSITION_TYPE_RELATIVE:
				{
					
					debug_fatal ("WP_PACK: internal waypoint with relative position");
					
					break;
				}
				
				case POSITION_TYPE_ABSOLUTE:
				case POSITION_TYPE_VIRTUAL:
				{
					
					unpack_vec3d (en, VEC3D_TYPE_POSITION, &raw->position);
					
					index = unpack_entity_safe_index ();
					
					raw->position_entity = get_local_entity_safe_ptr (index);
					
					break;
				}
			}
			
			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			break;
		}
	}
}





void overload_waypoint_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_WAYPOINT]         = pack_local_data;
	
	fn_unpack_local_entity_data[ENTITY_TYPE_WAYPOINT]       = unpack_local_data;
}




