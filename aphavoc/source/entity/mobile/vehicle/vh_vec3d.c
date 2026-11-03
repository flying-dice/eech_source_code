

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
   vehicle
      *raw;

   ASSERT (v);







   raw = (vehicle *) get_local_entity_data (en);

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
   vehicle
      *raw;

   ASSERT (v);







   raw = (vehicle *) get_local_entity_data (en);

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

         int
            old_x_sec,
            old_z_sec,
            new_x_sec,
            new_z_sec;

         entity
				*dependent,
				*special_effect,
            *parent;

			//
			// notify dependents of move, must be done before set_vec3d so children can work out delta positions
			//

         heading = get_heading_from_attitude_matrix (raw->mob.attitude);

			dependent = get_local_entity_first_child (en, LIST_TYPE_MOVEMENT_DEPENDENT);

			while (dependent)
			{







				notify_local_entity (ENTITY_MESSAGE_SET_ENTITY_POSITION, dependent, en, v, (double) heading, (double) raw->mob.velocity);

				dependent = get_local_entity_child_succ (dependent, LIST_TYPE_MOVEMENT_DEPENDENT);
			}

			//
			// Actual set_vec3d
			//

         get_x_sector (old_x_sec, raw->mob.position.x);
         get_z_sector (old_z_sec, raw->mob.position.z);

         get_x_sector (new_x_sec, v->x);
         get_z_sector (new_z_sec, v->z);

         if ((old_x_sec != new_x_sec) || (old_z_sec != new_z_sec))
         {
            delete_local_entity_from_parents_child_list (en, LIST_TYPE_SECTOR);

            raw->mob.position = *v;

            parent = get_local_raw_sector_entity (new_x_sec, new_z_sec);

            insert_local_entity_into_parents_child_list (en, LIST_TYPE_SECTOR, parent, NULL);
         }
         else
         {
            raw->mob.position = *v;
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
   vehicle
      *raw;

   ASSERT (v);

   raw = (vehicle *) get_local_entity_data (en);

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
	vehicle
		*raw;

	vec3d
		*v;

	raw = (vehicle *) get_local_entity_data (en);

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








































