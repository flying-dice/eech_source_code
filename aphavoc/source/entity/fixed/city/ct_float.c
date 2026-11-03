

































































#include "project.h"





#define DEBUG_MODULE 0





#define SET_LOCAL_REQUIRED				1
#define SET_CLIENT_SERVER_REQUIRED	1
#define GET_LOCAL_REQUIRED				1





#if SET_LOCAL_REQUIRED

static void set_local_float_value (entity *en, float_types type, float value)
{
	city
		*raw;







	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

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
	city
		*raw;

	float
		value;

	raw = (city *) get_local_entity_data (en);

	switch (type)
	{

		case FLOAT_TYPE_MAX_3D_OBJECT_VISUAL_RANGE:

		{
			env_3d
				*env;

			float
				yonder,
				hither;

			env = get_3d_active_environment ();

			ASSERT (env);

			get_3d_view_distances (env, &yonder, &hither);

			value = yonder;

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

















