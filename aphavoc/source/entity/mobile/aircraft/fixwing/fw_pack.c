

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	switch (mode)
	{

		case PACK_MODE_SERVER_SESSION:
		case PACK_MODE_CLIENT_SESSION:

		{
			fixed_wing
				*raw;

			raw = (fixed_wing *) get_local_entity_data (en);

			pack_entity_type (get_local_entity_type (en));

			pack_entity_safe_ptr (en);

			//
			// pack aircraft data
			//

			pack_aircraft_data (en, &raw->ac, mode);



			if (raw->ac.mob.velocity == 0.0)
			{

				pack_int_value (en, INT_TYPE_VALID, TRUE);
			}
			else
			{

				pack_int_value (en, INT_TYPE_VALID, FALSE);

				pack_float_value (en, FLOAT_TYPE_HIGH_VELOCITY, raw->ac.mob.velocity);

				pack_float_value (en, FLOAT_TYPE_AFTERBURNER_STATE, raw->afterburner_state);
			}



			//
			// pack fixwing data
			//

			//
			// spin through special effects link and pack up local only smoke trails
			//

//			pack_aircraft_meta_smoke_lists (en, mode);

			pack_mobile_local_sound_effects (en, mode);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			/*
			fixed_wing
				*raw;

			raw = get_local_entity_data (en);

			pack_vec3d (en, VEC3D_TYPE_POSITION, &raw->ac.mob.position);

			pack_attitude_matrix (en, raw->ac.mob.attitude);

			pack_vec3d (en, VEC3D_TYPE_MOTION_VECTOR, &raw->ac.mob.motion_vector);

			*/
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

			fixed_wing
				*raw;

			//
			// create entity
			//

			index = unpack_entity_safe_index ();

			en = get_free_entity (index);

			set_local_entity_type (en, type);

			raw = (fixed_wing *) malloc_fast_mem (sizeof (fixed_wing));

			set_local_entity_data (en, raw);

			memset (raw, 0, sizeof (fixed_wing));

			//
			// unpack data (in exactly the same order as the data was packed)
			//

			unpack_aircraft_data (en, &raw->ac, mode);



			if (unpack_int_value (en, INT_TYPE_VALID))
			{

				raw->ac.mob.velocity = 0.0;

				raw->afterburner_state = 0.0;
			}
			else
			{

				raw->ac.mob.velocity = unpack_float_value (en, FLOAT_TYPE_HIGH_VELOCITY);

				raw->afterburner_state = unpack_float_value (en, FLOAT_TYPE_AFTERBURNER_STATE);
			}



			set_fixed_wing_id_number (en);

			initialise_fixed_wing_propellors (en);

			//
			// link into system
			//

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_VIEW, get_camera_entity (), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, get_local_sector_entity (&raw->ac.mob.position), NULL);

			insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), NULL);

			add_to_force_info (get_local_force_entity ((entity_sides) raw->ac.mob.side), en);

			//
			// attached smoke lists must be unpacked after the entity is linked into the world
			//

//			unpack_aircraft_meta_smoke_lists (en, mode);
//			switch_aircraft_damaged_smoke(en);

			unpack_mobile_local_sound_effects (en, mode);

			break;
		}

		case PACK_MODE_BROWSE_SESSION:

		{
			break;
		}

		case PACK_MODE_UPDATE_ENTITY:

		{
			/*
			//
			// always use access functions to set the data
			//

			vec3d
				position,
				motion_vector;

			matrix3x3
				attitude;

			float
				velocity;

			velocity = 0;

			//
			// unpack all data (even if the echoed data is to be ignored)
			//

			unpack_vec3d (en, VEC3D_TYPE_POSITION, &position);

			unpack_attitude_matrix (en, attitude);

			unpack_vec3d (en, VEC3D_TYPE_MOTION_VECTOR, &motion_vector);

			if (!check_zero_3d_vector (&motion_vector))
			{

				vec3d
					normalised_motion_vector;

				flat
					mag;

				normalised_motion_vector = motion_vector;

				mag = normalise_3d_vector (&normalised_motion_vector);

				velocity = mag *
								(normalised_motion_vector.x * attitude [2][0] +
								normalised_motion_vector.y * attitude [2][1] +
								normalised_motion_vector.z * attitude [2][2]);
			}

			set_local_entity_vec3d (en, VEC3D_TYPE_POSITION, &position);

			set_local_entity_attitude_matrix (en, attitude);

			set_local_entity_vec3d (en, VEC3D_TYPE_MOTION_VECTOR, &motion_vector);

			set_local_entity_float_value (en, FLOAT_TYPE_VELOCITY, velocity);

			*/
			break;
		}
	}
}





void overload_fixed_wing_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_FIXED_WING]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_FIXED_WING]	= unpack_local_data;
}




