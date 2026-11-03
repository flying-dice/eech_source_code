

































































#include "project.h"

#include "ai/highlevl/highlevl.h"





#define DEBUG_MODULE 0





static entity *create_local (entity_types type, int index, char *pargs)
{
	entity
		*en;

	session
		*raw;


  	//
  	// VALIDATE
  	//


	validate_local_create_entity_index (index);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_CREATE, NULL, type, index);

	#endif

	en = get_free_entity (index);

	ASSERT (!get_session_entity ());

	if (en)
	{

   	//
   	// MALLOC ENTITY DATA
   	//


		set_local_entity_type (en, type);

		raw = (session *) malloc_fast_mem (sizeof (session));

		set_local_entity_data (en, raw);


   	//
   	// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES
		//
		// DO NOT USE ACCESS FUNCTIONS
		//
		// DO NOT USE RANDOM VALUES
		//


		memset (raw, 0, sizeof (session));

		raw->version_number = get_global_version_number ();

		raw->session_complete = FALSE;

		raw->cheats_enabled = FALSE;

		raw->start_time = get_session_random_start_time_of_day ();

		raw->time_of_day_acceleration = 1.0;

		raw->local_weather_model = TRUE;

		raw->weather_mode = WEATHERMODE_DRY;

		raw->target_weather_mode = WEATHERMODE_DRY;

		raw->weather_radius = 40.0 * KILOMETRE;

		raw->wind_effect_radius = 40.0 * KILOMETRE;

		raw->weather_mode_transitional_period = 60.0;

		raw->weather_mode_transitional_status = 0.0;

		raw->weather_position.x = MID_MAP_X;
		raw->weather_position.y = 0.0;
		raw->weather_position.z = MID_MAP_Z;

		raw->weather_velocity.x = knots_to_metres_per_second (300.0);
		raw->weather_velocity.y = 0.0;
		raw->weather_velocity.z = -knots_to_metres_per_second (100.0);

		raw->wind_effect_position.x = 0.0;
		raw->wind_effect_position.y = 0.0;
		raw->wind_effect_position.z = 0.0;

		raw->wind_effect_velocity.x = knots_to_metres_per_second (240.0);
		raw->wind_effect_velocity.y = 0.0;
		raw->wind_effect_velocity.z = knots_to_metres_per_second (350.0);

		raw->lightning_timer = ONE_MINUTE;

		raw->wind_minimum_speed = DEFAULT_MIN_WIND_SPEED;
		raw->wind_maximum_speed = DEFAULT_MAX_WIND_SPEED;

		raw->wind_direction_vector.x = raw->wind_effect_position.x - MID_MAP_X;
		raw->wind_direction_vector.y = 0.0;
		raw->wind_direction_vector.z = raw->wind_effect_position.z - MID_MAP_Z;

		normalise_any_3d_vector (&raw->wind_direction_vector);

		raw->day_segment_type = get_day_segment_type (raw->start_time);

		raw->population_x_min = MIN_MAP_X;
		raw->population_x_max = MAX_MAP_X;
		raw->population_z_min = MIN_MAP_Z;
		raw->population_z_max = MAX_MAP_Z;

		raw->fog_of_war_maximum_value = command_line_fog_of_war_maximum_value;


		//
		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		//


		set_local_entity_attributes (en, pargs);


		//
		// CHECK MANDATORY ATTRIBUTES HAVE BEEN GIVEN
		//



		//
		// RESOLVE DEFAULT VALUES
		//


		raw->fog_of_war_maximum_value = bound (raw->fog_of_war_maximum_value, (8.0 * FOG_OF_WAR_DECAY_RATE), (8.0 * ONE_HOUR));


		//
		// LINK INTO SYSTEM
		//


		//
		// need to insert session into update list after camera entity
		//

		ASSERT (get_camera_entity ());

		insert_local_entity_into_parents_child_list (en, LIST_TYPE_UPDATE, get_update_entity (), get_camera_entity ());

		set_session_entity (en);

		//
		// for special (i.e. deathmatch games), initialise the high score table (Not used in C-H yet...)
		//

		initialise_pilot_high_score_table ();
	}

	return (en);
}





static entity *create_remote (entity_types type, int index, char *pargs)
{
	validate_remote_create_entity_index (index);

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_REMOTE, ENTITY_DEBUG_CREATE, NULL, type, index);

	#endif

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





void overload_session_create_functions (void)
{
	fn_create_local_entity[ENTITY_TYPE_SESSION]										= create_local;

	fn_create_client_server_entity[ENTITY_TYPE_SESSION][COMMS_MODEL_SERVER]	= create_server;

	fn_create_client_server_entity[ENTITY_TYPE_SESSION][COMMS_MODEL_CLIENT]	= create_client;
}




