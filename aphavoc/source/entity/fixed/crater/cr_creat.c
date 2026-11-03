

































































#include "project.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	crater
		*raw;

	int
		seed;


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

		raw = (crater *) malloc_fast_mem (sizeof (crater));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (crater));

		//
		// fixed
		//

		raw->fix.position.x = MID_MAP_X;
		raw->fix.position.y = MID_MAP_Y;
		raw->fix.position.z = MID_MAP_Z;

		raw->fix.sub_type = ENTITY_SUB_TYPE_FIXED_CRATER;

		raw->fix.object_3d_shape = OBJECT_3D_INVALID_OBJECT_INDEX;

		raw->fix.alive = FALSE;

		raw->fix.side = ENTITY_SIDE_NEUTRAL;

		seed = get_client_server_entity_random_number_seed (en);

		raw->fix.heading = PI * sfrand1x (&seed);

		//
		// crater
		//


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//


		ASSERT (raw->crater_lifetime > 0.0);

		ASSERT (raw->scale > 0.0);


		//
		// RESOLVE DEFAULT VALUES
		//


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

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);
	}

	return (en);
}





static entity *create_remote (entity_types type, int index, char *pargs)
{
	validate_remote_create_entity_index (index);







	transmit_entity_comms_message (ENTITY_COMMS_CREATE, NULL, type, index, pargs);

	return (NULL);
}





static entity *create_server (entity_types type, int index, char *pargs)
{
	entity
		*en;

	validate_client_server_local_fn ();

	en = create_local (type, index, pargs);

	if (en)
	{
		validate_client_server_remote_fn ();

		create_remote (type, get_local_entity_index (en), pargs);
	}

	return (en);
}





static entity *create_client (entity_types type, int index, char *pargs)
{
	entity
		*en;

	if (get_comms_data_flow () == COMMS_DATA_FLOW_TX)
	{
		validate_client_server_remote_fn ();

		en = create_remote (type, index, pargs);
	}
	else
	{
		validate_client_server_local_fn ();

		en = create_local (type, index, pargs);
	}

	return (en);
}


















void create_client_server_weapon_hit_ground_crater (entity *weapon, vec3d *position)
{
	int
		damage_capability;

	weapon_warhead_types
		warhead_type;

	crater_types
		type;

	entity_sub_types
			weapon_type = get_local_entity_int_value (weapon, INT_TYPE_ENTITY_SUB_TYPE);

	switch (weapon_database[weapon_type].warhead_type)
	{
		case WEAPON_WARHEAD_TYPE_CONVENTIONAL_MUNITIONS:
			warhead_type = weapon_database[weapon_type + 1].warhead_type;
			damage_capability = weapon_database[weapon_type + 1].damage_capability * weapon_database[weapon_type + 1].burst_duration / 3;
			break;
		default:
			warhead_type = weapon_database[weapon_type].warhead_type;
			damage_capability = weapon_database[weapon_type].damage_capability;
	}

	switch (warhead_type)
	{
		case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_TANK:
		case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_DUAL_PURPOSE:
		case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE:
		case WEAPON_WARHEAD_TYPE_HIGH_EXPLOSIVE_ANTI_AIRCRAFT:
		{
			if (damage_capability > 2000)
			{
				type = CRATER_TYPE_LARGE_EXPLOSION;
			}
			else if (damage_capability > 1000)
			{
				type = CRATER_TYPE_MEDIUM_EXPLOSION;
			}
			else if (damage_capability > 100)
			{
				type = CRATER_TYPE_SMALL_EXPLOSION;
			}
			else
				return;

			break;
		}
		default:
		{
			return;
		}
	}

	create_client_server_crater (type, position);
}





void create_client_server_crater (crater_types type, vec3d *position)
{
	float
		scale,
		min_scale,
		max_scale;

	min_scale = crater_database[type].min_scale;
	max_scale = crater_database[type].max_scale;

	scale = min_scale + ((max_scale - min_scale) * frand1());

	create_client_server_entity
	(
		ENTITY_TYPE_CRATER,
		ENTITY_INDEX_DONT_CARE,
		ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, position->x, position->y, position->z),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_OBJECT_3D_SHAPE, crater_database[type].default_3d_shape),
		ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_CRATER_LIFETIME, crater_database[type].lifetime),
		ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_SCALE, scale),
		ENTITY_ATTR_END
	);
}




