

































































#include "project.h"

#include "ai/ai_misc/ai_route.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			routed_vehicle
				*raw;

			raw = (routed_vehicle *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			//
			// pack vehicle data
			//

			pack_vehicle_data (en, &raw->vh, mode);



			if ((raw->vh.mob.velocity == 0.0) && (raw->desired_velocity == 0.0))
			{

				pack_int_value (en, INT_TYPE_VALID, TRUE);
			}
			else
			{
				
				pack_int_value (en, INT_TYPE_VALID, FALSE);

				pack_float_value (en, FLOAT_TYPE_LOW_VELOCITY, raw->vh.mob.velocity);

				pack_float_value (en, FLOAT_TYPE_DESIRED_VELOCITY, raw->desired_velocity);
			}



			//
			// pack routed data
			//


			if ((raw->sub_waypoint_count == 0) && (raw->waypoint_next_index == 0) && (raw->waypoint_this_index == 0))
			{

				pack_int_value (en, INT_TYPE_VALID, FALSE);
			}
			else
			{

				pack_int_value (en, INT_TYPE_VALID, TRUE);
	
				pack_int_value (en, INT_TYPE_SUB_WAYPOINT_COUNT, raw->sub_waypoint_count);
	
				pack_int_value (en, INT_TYPE_WAYPOINT_NEXT_INDEX, raw->waypoint_next_index);
	
				pack_int_value (en, INT_TYPE_WAYPOINT_THIS_INDEX, raw->waypoint_this_index);
			}


			// sub_waypoint_list reconstructed locally

			//
			// spin through special effects link and pack up local only smoke trails
			//

			pack_routed_vehicle_meta_smoke_lists (en, mode);

			pack_mobile_local_sound_effects (en, mode);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{

			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{

			routed_vehicle
				*raw;

			raw = (routed_vehicle *) get_local_entity_data (en);

			pack_vec3d (en, VEC3D_TYPE_POSITION, &raw->vh.mob.position);

			pack_attitude_matrix (en, raw->vh.mob.attitude);

			pack_int_value (en, INT_TYPE_SUB_WAYPOINT_COUNT, raw->sub_waypoint_count);

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
		case PACK_MODE_CLIENT_SESSION:

		{

			int
				index;

			routed_vehicle
				*raw;

			node_link_data
				*route_data;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (routed_vehicle *) malloc_fast_mem (sizeof (routed_vehicle));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (routed_vehicle));

			//
			// unpack vehicle data (in exactly the same order as the data was packed)
			//

			unpack_vehicle_data (en, &raw->vh, mode);

			if (unpack_int_value (en, INT_TYPE_VALID))
			{

				raw->vh.mob.velocity = 0.0;

				raw->desired_velocity = 0.0;
			}
			else
			{

				raw->vh.mob.velocity = unpack_float_value (en, FLOAT_TYPE_LOW_VELOCITY);

				raw->desired_velocity = unpack_float_value (en, FLOAT_TYPE_DESIRED_VELOCITY);
			}

			//
			// unpack routed data
			//

			if (unpack_int_value (en, INT_TYPE_VALID))
			{
	
				raw->sub_waypoint_count = unpack_int_value (en, INT_TYPE_SUB_WAYPOINT_COUNT);
	
				raw->waypoint_next_index = unpack_int_value (en, INT_TYPE_WAYPOINT_NEXT_INDEX);
	
				raw->waypoint_this_index = unpack_int_value (en, INT_TYPE_WAYPOINT_THIS_INDEX);
			}

			//
			// turn lights on if necessary
			//

			set_vehicle_headlight_state (en, raw->vh.lights_on);

			//
			// setup waypoint route pointer
			//

			#if DEBUG_MODULE

			debug_log ("ROUTED ENTITY PACK: client setting up waypoint list pointer");

			#endif

			route_data = get_road_sub_route (raw->waypoint_this_index, raw->waypoint_next_index, &index, NULL);

			if ((route_data) && (route_data->link_positions))
			{

				raw->sub_route = route_data;
			}
			else
			{

				if ((raw->waypoint_this_index + raw->waypoint_next_index) != 0)
				{

					debug_log ("RV_PACK: WARNING NO SUB ROUTE FOUND BETWEEN %d AND %d", raw->waypoint_this_index, raw->waypoint_next_index);
				}
			}

			//
			// radar dish rotation (ok to use a random number as this is for visual effect only)
			//

			raw->vh.radar_rotation_state = frand1 ();

			set_routed_vehicle_id_number (en);

			set_initial_rotation_angle_of_routed_vehicle_wheels (raw->vh.inst3d);

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_VIEW, get_camera_entity (), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->vh.mob.position), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

			add_to_force_info (get_local_force_entity ((entity_sides) raw->vh.mob.side), en);

			//
			// attached smoke lists must be unpacked after the entity is linked into the world
			//

			unpack_routed_vehicle_meta_smoke_lists (en, mode);
	
			unpack_mobile_local_sound_effects (en, mode);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{

			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			//
			// always use access functions to set the data
			//

			vec3d
				position;

			matrix3x3
				attitude;

			int
				sub_waypoint_count;

			//
			// unpack all data (even if the echoed data is to be ignored)
			//

			unpack_vec3d (en, VEC3D_TYPE_POSITION, &position);

			unpack_attitude_matrix (en, attitude);

			sub_waypoint_count = unpack_int_value (en, INT_TYPE_SUB_WAYPOINT_COUNT);

			set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &position);

			set_local_entity_attitude_matrix (en, attitude);

			set_local_entity_int_value (en, INT_TYPE_SUB_WAYPOINT_COUNT, sub_waypoint_count);

			// Vehicle is moving so sleep must equal 0
			set_local_entity_float_value (en, FLOAT_TYPE_SLEEP, 0.0);

			break;
		}
	}
}





void overload_routed_vehicle_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_ROUTED_VEHICLE]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_ROUTED_VEHICLE]	= unpack_local_data;
}




