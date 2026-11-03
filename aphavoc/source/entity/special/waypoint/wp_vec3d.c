

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_raw_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	waypoint
		*raw;

	ASSERT (v);







	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{

			raw->position = *v;

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
	waypoint
		*raw;

	ASSERT (v);







	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{

			raw->position = *v;

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
	waypoint
		*raw;

	ASSERT (v);

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{

			switch (raw->position_type)
			{

				case POSITION_TYPE_VIRTUAL:
				{
					entity
						*position_entity;

					vec3d
						*pos;

					position_entity = get_local_entity_parent (en, LIST_TYPE_TASK_DEPENDENT);

					if (position_entity)
					{
						pos = get_local_entity_vec3d_ptr (position_entity, VEC3D_TYPE_POSITION);

						v->x = pos->x;
						v->y = pos->y;
						v->z = pos->z;
					}
					else
					{
						v->x = MID_MAP_X;
						v->y = MID_MAP_Y;
						v->z = MID_MAP_Z;
					}
					
				  	break;
				}
				default:
				{
			
					*v = raw->position;
					
				  	break;
				}
			}

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
	waypoint
		*raw;

	vec3d
		*v;

	raw = (waypoint *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_POSITION:

		{

			switch (raw->position_type)
			{

				case POSITION_TYPE_VIRTUAL:
				{

					entity
						*position_entity;

					position_entity = get_local_entity_parent (en, LIST_TYPE_TASK_DEPENDENT);

					if (position_entity)
					{
	
						v = get_local_entity_vec3d_ptr (position_entity, VEC3D_TYPE_POSITION);
					}
					else
					{

						v = NULL;
					}
					
				  	break;
				}
				default:
				{

					v = &raw->position;
					
				  	break;
				}
			}

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





























