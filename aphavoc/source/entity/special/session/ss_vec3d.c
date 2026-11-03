

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_vec3d (entity *en, vec3d_types type, vec3d *v)
{
	session
		*raw;

	ASSERT (v);







	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_WEATHER_POSITION:

		{
			raw->weather_position = *v;

			break;
		}

		case VEC3D_TYPE_WEATHER_VELOCITY:

		{
			raw->weather_velocity = *v;

			break;
		}

		case VEC3D_TYPE_WIND_DIRECTION_VECTOR:

		{
			raw->wind_direction_vector = *v;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_POSITION:

		{
			raw->wind_effect_position = *v;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_VELOCITY:

		{
			raw->wind_effect_velocity = *v;

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
	session
		*raw;

	ASSERT (v);

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_WEATHER_POSITION:

		{
			*v = raw->weather_position;

			break;
		}

		case VEC3D_TYPE_WEATHER_VELOCITY:

		{
			*v = raw->weather_velocity;

			break;
		}

		case VEC3D_TYPE_WIND_DIRECTION_VECTOR:

		{
			*v = raw->wind_direction_vector;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_POSITION:

		{
			*v = raw->wind_effect_position;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_VELOCITY:

		{
			*v = raw->wind_effect_velocity;

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
	session
		*raw;

	vec3d
		*v;

	raw = (session *) get_local_entity_data (en);

	switch (type)
	{

		case VEC3D_TYPE_WEATHER_POSITION:

		{
			v = &raw->weather_position;

			break;
		}

		case VEC3D_TYPE_WEATHER_VELOCITY:

		{
			v = &raw->weather_velocity;

			break;
		}

		case VEC3D_TYPE_WIND_DIRECTION_VECTOR:

		{
			v = &raw->wind_direction_vector;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_POSITION:

		{
			v = &raw->wind_effect_position;

			break;
		}

		case VEC3D_TYPE_WIND_EFFECT_VELOCITY:

		{
			v = &raw->wind_effect_velocity;

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









































