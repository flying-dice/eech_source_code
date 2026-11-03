

































































#include "project.h"





#define DEBUG_MODULE 									0

#define DEBUG_MODULE_AIRCRAFT_POSITION_MESSAGE	0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
   aircraft
      *raw;

   ASSERT (v);







   raw = (aircraft *) get_local_entity_data (en);

   switch (type)
   {

      case VEC3D_TYPE_POSITION:

      {
         raw->mob.position = *v;

         break;
      }

      default:

      {
         debug_fatal_invalid_vec3d_type (en, type);

         break;
      }
   }
}





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
   aircraft
      *raw;

   ASSERT (v);







   raw = (aircraft *) get_local_entity_data (en);

   switch (type)
   {

      case VEC3D_TYPE_MOTION_VECTOR:

      {
         raw->mob.motion_vector = *v;

         break;
      }

      case VEC3D_TYPE_POSITION:

      {

         float
            heading;

         entity
				*dependent,
				*special_effect,
				*old_sector,
				*new_sector;

			//
			// notify dependents of move, must be done before set_vec3d so children can work out delta positions
			//

         heading = 0.0;

			//if (get_local_entity_type (en) == ENTITY_TYPE_HELICOPTER)
			{

				heading = get_heading_from_attitude_matrix (raw->mob.attitude);
	
				dependent = get_local_entity_first_child (en, LIST_TYPE_MOVEMENT_DEPENDENT);
	
				while (dependent)
				{





	
					notify_local_entity (ENTITY_MESSAGE_SET_ENTITY_POSITION, dependent, en, v, (double) heading, (double) raw->mob.velocity);
	
					dependent = get_local_entity_child_succ (dependent, LIST_TYPE_MOVEMENT_DEPENDENT);
				}
			}

			//
			// Actual set_vec3d
			//

         raw->mob.position = *v;

			//
			// check if entered new sector
			//

			old_sector = get_local_entity_parent (en, LIST_TYPE_SECTOR);

			new_sector = get_local_sector_entity (v);

         if (old_sector != new_sector)
         {
            delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

            insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, new_sector, NULL);

				//
				// play speech for flown across "front-line"
				//

				if (get_comms_model () == COMMS_MODEL_SERVER)
				{
					if (get_local_entity_int_value (new_sector, INT_TYPE_SIDE) != get_local_entity_int_value (old_sector, INT_TYPE_SIDE))
					{
						play_aircraft_flown_into_new_sector_speech (en, old_sector, new_sector);
					}
				}
         }

			//
			// notify special effects of move
			//

			special_effect = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);

			while (special_effect)
			{






				notify_local_entity (ENTITY_MESSAGE_SET_ENTITY_POSITION, special_effect, en, &raw->mob.position, (double) heading, (double) raw->mob.velocity);

				special_effect = get_local_entity_child_succ (special_effect, LIST_TYPE_SPECIAL_EFFECT);
			}

         break;
      }

      case VEC3D_TYPE_WEAPON_TO_TARGET_VECTOR:

      {
         raw->weapon_to_target_vector = *v;

         break;
      }

      case VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR:

      {
         raw->weapon_to_intercept_point_vector = *v;

         break;
      }

      case VEC3D_TYPE_WEAPON_VECTOR:

      {
         raw->weapon_vector = *v;

         break;
      }

      default:

      {
         debug_fatal_invalid_vec3d_type (en, type);

         break;
      }
   }
}





























































static void get_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
   aircraft
      *raw;

   ASSERT (v);

   raw = (aircraft *) get_local_entity_data (en);

   switch (type)
   {

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			*v = raw->mob.motion_vector;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			*v = raw->mob.position;

			break;
		}

		case VEC3D_TYPE_WEAPON_TO_TARGET_VECTOR:

		{
			*v = raw->weapon_to_target_vector;

			break;
		}

		case VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR:

		{
			*v = raw->weapon_to_intercept_point_vector;

			break;
		}

		case VEC3D_TYPE_WEAPON_VECTOR:

		{
			*v = raw->weapon_vector;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}
}





static vec3d *get_local_vec3d_ptr (entity *en, vec3d_types type)
{
	aircraft
		*raw;

	vec3d
		*v;

	raw = (aircraft *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_MOTION_VECTOR:

		{
			v = &raw->mob.motion_vector;

			break;
		}

		case VEC3D_TYPE_POSITION:

		{
			v = &raw->mob.position;

			break;
		}

		case VEC3D_TYPE_WEAPON_TO_TARGET_VECTOR:

		{
			v = &raw->weapon_to_target_vector;

			break;
		}

		case VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR:

		{
			v = &raw->weapon_to_intercept_point_vector;

			break;
		}

		case VEC3D_TYPE_WEAPON_VECTOR:

		{
			v = &raw->weapon_vector;

			break;
		}

		default:

		{
			debug_fatal_invalid_vec3d_type (en, type);

			break;
		}
	}

	return (v);
}








































