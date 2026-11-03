

































































#include "project.h"







#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	fixed_wing
		*raw;

	entity_sub_types
		group_sub_type;


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

		raw = (fixed_wing *) malloc_fast_mem (sizeof (fixed_wing));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (fixed_wing));

		//
		// mobile
		//

		raw->ac.mob.sub_type = ENTITY_SUB_TYPE_UNINITIALISED;

		raw->ac.mob.position.x = MID_MAP_X;
		raw->ac.mob.position.y = MID_MAP_Y;
		raw->ac.mob.position.z = MID_MAP_Z;

		get_identity_matrix3x3 (raw->ac.mob.attitude);

		raw->ac.mob.alive = TRUE;

		raw->ac.mob.side = ENTITY_SIDE_UNINITIALISED;

		raw->ac.weapon_launch_delay = 0;

		raw->ac.operational_state = OPERATIONAL_STATE_UNKNOWN;

		//
		// aircraft
		//

		raw->ac.object_3d_shape = OBJECT_3D_INVALID_OBJECT_INDEX;

		raw->ac.weapon_config_type = WEAPON_CONFIG_TYPE_UNINITIALISED;

		raw->ac.selected_weapon = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;

		raw->ac.weapon_vector.x = 0.0;
		raw->ac.weapon_vector.y = 0.0;
		raw->ac.weapon_vector.z = 1.0;

		raw->ac.weapon_to_target_vector.x = 0.0;
		raw->ac.weapon_to_target_vector.y = 0.0;
		raw->ac.weapon_to_target_vector.z = -1.0;

		raw->ac.weapon_to_intercept_point_vector.x = 0.0;
		raw->ac.weapon_to_intercept_point_vector.y = 0.0;
		raw->ac.weapon_to_intercept_point_vector.z = -1.0;

		raw->ac.weapon_to_intercept_point_range = 0.0;
		
		raw->ac.loading_door_state = AIRCRAFT_LOADING_DOORS_OPEN_FLOAT_VALUE;
		raw->ac.undercarriage_state = AIRCRAFT_UNDERCARRIAGE_DOWN_FLOAT_VALUE;

		raw->ac.air_radar_contact_timeout = AIR_RADAR_CONTACT_TIMEOUT_INVALID;

		//magitek navigation
		raw->fixed_wing_toggle = 0.0;
		raw->turn_speed_multi = 2.0;
		
		//
		// fixed_wing
		//


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->ac.member_link.parent);

		ASSERT (get_local_entity_type (raw->ac.member_link.parent) == ENTITY_TYPE_GROUP);


		//
		// RESOLVE DEFAULT VALUES
		//


		//
		// side
		//

		if (raw->ac.mob.side == ENTITY_SIDE_UNINITIALISED)
		{
			raw->ac.mob.side = get_local_entity_int_value (raw->ac.member_link.parent, INT_TYPE_SIDE);
		}

		ASSERT (raw->ac.mob.side != ENTITY_SIDE_NEUTRAL);

		//
		// sub_type
		//

		if (raw->ac.mob.sub_type == ENTITY_SUB_TYPE_UNINITIALISED)
		{
			group_sub_type = get_local_entity_int_value (raw->ac.member_link.parent, INT_TYPE_ENTITY_SUB_TYPE);

			if (raw->ac.mob.side == ENTITY_SIDE_BLUE_FORCE)
			{
				raw->ac.mob.sub_type = group_database[group_sub_type].default_blue_force_sub_type;
			}
			else
			{
				raw->ac.mob.sub_type = group_database[group_sub_type].default_red_force_sub_type;
			}
		}

		ASSERT (entity_sub_type_aircraft_valid (raw->ac.mob.sub_type));

		//
		// 3D shape
		//

		if (raw->ac.object_3d_shape == OBJECT_3D_INVALID_OBJECT_INDEX)
		{
			raw->ac.object_3d_shape = aircraft_database[raw->ac.mob.sub_type].default_3d_shape;
		}

		//
		// weapon config
		//

		if (raw->ac.weapon_config_type == WEAPON_CONFIG_TYPE_UNINITIALISED)
		{
			raw->ac.weapon_config_type = aircraft_database[raw->ac.mob.sub_type].default_weapon_config_type;
		}

		ASSERT (weapon_config_type_valid (raw->ac.weapon_config_type));

		//
		// damage levels
		//

		raw->ac.damage_level = aircraft_database[raw->ac.mob.sub_type].initial_damage_level;


		//
		// BUILD COMPONENTS
		//


















		//
		// weapon config
		//

		raw->ac.weapon_package_status_array = (weapon_package_status *) malloc_fast_mem (SIZE_WEAPON_PACKAGE_STATUS_ARRAY);

		memset (raw->ac.weapon_package_status_array, 0, SIZE_WEAPON_PACKAGE_STATUS_ARRAY);

		load_local_entity_weapon_config (en);

		//
		// update force info
		//

		add_to_force_info (get_local_force_entity ((entity_sides) raw->ac.mob.side), en);


		//
		// LINK INTO SYSTEM
		//


		insert_local_entity_into_parents_child_list (en, LIST_TYPE_MEMBER, raw->ac.member_link.parent, raw->ac.member_link.child_pred);







		insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->ac.mob.position), NULL);

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);



	}

	return (en);
}





















































































entity *create_client_server_entity_fixed_wing (int index, entity_sub_types sub_type, entity *group, vec3d *position)
{
	entity
		*new_entity;

	fixed_wing
		*raw;

	ASSERT (get_comms_model() == COMMS_MODEL_SERVER);

	//
	// create fixed wing entity
	//

	new_entity = create_client_server_entity
	(
		ENTITY_TYPE_FIXED_WING,
		index,
		ENTITY_ATTR_PARENT (LIST_TYPE_MEMBER, group),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, sub_type),
		ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, position->x, position->y, position->z),
		ENTITY_ATTR_END
	);


































































	return new_entity;
}







