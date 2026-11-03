

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	site_updatable
		*raw;







	raw = (site_updatable *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_LOADING_DOOR_STATE:

		{
			raw->loading_door_state = value;

			break;
		}

		case FLOAT_TYPE_RADAR_ROTATION_STATE:

		{
			raw->radar_rotation_state = value;

			break;
		}

		case FLOAT_TYPE_LOADING_DOOR_TIMER:

		{
			raw->loading_door_timer = value;

			break;
		}

		case FLOAT_TYPE_GROUND_CREW_ANIMATION:

		{
			raw->ground_crew_animation = value;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}
}

#endif





#if SET_CLIENT_SERVER_REQUIRED












#endif





#if SET_CLIENT_SERVER_REQUIRED












#endif





#if SET_CLIENT_SERVER_REQUIRED

















#endif





#if GET_LOCAL_REQUIRED

static float get_local_float_value (entity *en, float_types type)
{
	site_updatable
		*raw;

	float
		value;

	raw = (site_updatable *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_AVOIDANCE_RADIUS:

		{
			object_3d_bounds
				*bounding_box;

			bounding_box = get_object_3d_bounding_box_without_lines (raw->fix.object_3d_shape);

			value = max (50.0, (2.0 * bounding_box->ymax));

			break;
		}

		case FLOAT_TYPE_LOADING_DOOR_STATE:

		{
			value = raw->loading_door_state;

			break;
		}

		case FLOAT_TYPE_RADAR_ROTATION_STATE:

		{
			value = raw->radar_rotation_state;

			break;
		}

		case FLOAT_TYPE_LOADING_DOOR_TIMER:

		{
			value = raw->loading_door_timer;

			break;
		}

		case FLOAT_TYPE_GROUND_CREW_ANIMATION:

		{
			value = raw->ground_crew_animation;

			break;
		}

		default:

		{
			debug_fatal_invalid_float_type (en, type);

			break;
		}
	}

	return (value);
}

#endif





































