//
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
//
// 	 Please see the document LICENSE.TXT for the full licence agreement
//
// 2. LICENCE
//  2.1
//  	Subject to the provisions of this Agreement we now grant to you the
//  	following rights in respect of the Source Code:
//   2.1.1
//   	the non-exclusive right to Exploit  the Source Code and Executable
//   	Code on any medium; and
//   2.1.2
//   	the non-exclusive right to create and distribute Derivative Works.
//  2.2
//  	Subject to the provisions of this Agreement we now grant you the
// 	following rights in respect of the Object Code:
//   2.2.1
// 	the non-exclusive right to Exploit the Object Code on the same
// 	terms and conditions set out in clause 3, provided that any
// 	distribution is done so on the terms of this Agreement and is
// 	accompanied by the Source Code and Executable Code (as
// 	applicable).
//
// 3. GENERAL OBLIGATIONS
//  3.1
//  	In consideration of the licence granted in clause 2.1 you now agree:
//   3.1.1
// 	that when you distribute the Source Code or Executable Code or
// 	any Derivative Works to Recipients you will also include the
// 	terms of this Agreement;
//   3.1.2
// 	that when you make the Source Code, Executable Code or any
// 	Derivative Works ("Materials") available to download, you will
// 	ensure that Recipients must accept the terms of this Agreement
// 	before being allowed to download such Materials;
//   3.1.3
// 	that by Exploiting the Source Code or Executable Code you may
// 	not impose any further restrictions on a Recipient's subsequent
// 	Exploitation of the Source Code or Executable Code other than
// 	those contained in the terms and conditions of this Agreement;
//   3.1.4
// 	not (and not to allow any third party) to profit or make any
// 	charge for the Source Code, or Executable Code, any
// 	Exploitation of the Source Code or Executable Code, or for any
// 	Derivative Works;
//   3.1.5
// 	not to place any restrictions on the operability of the Source
// 	Code;
//   3.1.6
// 	to attach prominent notices to any Derivative Works stating
// 	that you have changed the Source Code or Executable Code and to
// 	include the details anddate of such change; and
//   3.1.7
//   	not to Exploit the Source Code or Executable Code otherwise than
// 	as expressly permitted by  this Agreement.
//



/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#include "project.h"

#include "entity/special/session/ss_envfx.h"

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define DEBUG_MODULE 0

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float
	weather_rain_radius_min = MIN_WEATHER_RADIUS,
	weather_rain_radius_max = MAX_WEATHER_RADIUS,
	weather_rain_speed = WEATHER_EFFECT_SPEED,
	weather_wind_radius_min = MIN_WIND_RADIUS,
	weather_wind_radius_max = MAX_WIND_RADIUS,
	weather_wind_speed = WIND_EFFECT_SPEED;


/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

entity
	*session_entity = NULL;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifndef EECH_CAMPAIGN_HOST_ENV

#define WIND_GUSTING_SAMPLE_POINTS	6

static float
	wind_gust_modulation_amounts [WIND_GUSTING_SAMPLE_POINTS] =
	{
		 0.25,
		-0.05,
		-0.25,
		 0.05,
		-0.01,
		 0.15
	};

static float get_wind_gusting_modulation_value (float wind_gusting_value);

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void set_session_entity (entity *en)
{
	session_entity = en;

	if (en)
	{
		debug_log ("Setting session_entity (type = %s, index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));
	}
	else
	{
		debug_log ("Setting session_entity to NULL");
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float calculate_session_time_of_day (entity *en, int *day)
{
	float
		start_time,
		time_of_day,
		elapsed_time;

	int
		count;

	ASSERT (en);

	elapsed_time = get_local_entity_float_value (en, FLOAT_TYPE_ELAPSED_TIME);

	start_time = get_local_entity_float_value (en, FLOAT_TYPE_START_TIME);

	time_of_day = start_time + elapsed_time;

	count = 0;

	while (time_of_day >= ONE_DAY)
	{
		time_of_day -= ONE_DAY;

		count ++;
	}

	time_of_day = max (time_of_day, 0.0f);

	if (day)
	{
		*day = count;
	}

	return time_of_day;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

day_segment_types get_day_segment_type (float time_of_day)
{
	day_segment_types
		type;

	if (time_of_day < (ONE_HOUR * 6.0))
	{
		type = DAY_SEGMENT_TYPE_NIGHT;
	}
	else if (time_of_day < (ONE_HOUR * 8.0))
	{
		type = DAY_SEGMENT_TYPE_DAWN;
	}
	else if (time_of_day < (ONE_HOUR * 18.0))
	{
		type = DAY_SEGMENT_TYPE_DAY;
	}
	else if (time_of_day < (ONE_HOUR * 20.0))
	{
		type = DAY_SEGMENT_TYPE_DUSK;
	}
	else
	{
		type = DAY_SEGMENT_TYPE_NIGHT;
	}

	return (type);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float get_segment_time_of_day (day_segment_types segment)
{
	float
		time_of_day;

	switch (segment)
	{
		case DAY_SEGMENT_TYPE_DAWN:
		{
			time_of_day = (ONE_HOUR * 6.0);

			break;
		}
		case DAY_SEGMENT_TYPE_DAY:
		{
			time_of_day = (ONE_HOUR * 8.0);

			break;
		}
		case DAY_SEGMENT_TYPE_DUSK:
		{
			time_of_day = (ONE_HOUR * 18.0);

			break;
		}
		case DAY_SEGMENT_TYPE_NIGHT:
		{
			time_of_day = (ONE_HOUR * 20.0);

			break;
		}
	}

	return (time_of_day);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#define NUM_RANDOM_SESSION_START_TIMES	6

static float
	random_session_start_times [NUM_RANDOM_SESSION_START_TIMES] =
	{
		( 0.5 * ONE_HOUR),
		( 5.5 * ONE_HOUR),
		( 9.0 * ONE_HOUR),
		(12.0 * ONE_HOUR),
		(15.0 * ONE_HOUR),
		(17.5 * ONE_HOUR)
	};

float get_session_random_start_time_of_day (void)
{
	int
		index;

	float
		start_time;

	initialise_random_number_generator ();

	ENV_FX (ENV_FX_START_RESEED, 0, 0, 0, 0, 0, 0, 0, 0)

	index = rand16 () % NUM_RANDOM_SESSION_START_TIMES;

	ENV_FX (ENV_FX_START_INDEX, index, 0, 0, 0, 0, 0, 0, 0)

	start_time = random_session_start_times [index];

	start_time += floor (sfrand1 () * ONE_MINUTE * 15.0);

	ENV_FX (ENV_FX_START_TIME, 0, 0, ENV_FXF (start_time), 0, 0, 0, 0, 0)

	if (start_time < 0.0)
	{
		start_time += (24.0 * ONE_HOUR);
	}
	else if (start_time >= 24.0 * ONE_HOUR)
	{
		start_time -= (24.0 * ONE_HOUR);
	}

	ENV_FX (ENV_FX_START_RESULT, 0, 0, ENV_FXF (start_time), 0, 0, 0, 0, 0)

	return start_time;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

static struct SESSION_PRESET_TIME_OF_DAY_TABLE
{
	session_time_of_day_settings
		preset_enum;

	float
		time;

} session_preset_time_of_day_table [] =
	{
		SESSION_TIME_DAWN,			(05.0 * ONE_HOUR) + (45 * ONE_MINUTE),
		SESSION_TIME_MIDDAY,			(11.0 * ONE_HOUR) + (50 * ONE_MINUTE),
		SESSION_TIME_AFTERNOON,		(15.0 * ONE_HOUR) + (20 * ONE_MINUTE),
		SESSION_TIME_DUSK,			(18.0 * ONE_HOUR) + (00 * ONE_MINUTE),
		SESSION_TIME_MIDNIGHT,		(23.0 * ONE_HOUR) + (45 * ONE_MINUTE),
		(session_time_of_day_settings) -1
	};

float get_session_preset_time_of_day (session_time_of_day_settings preset_enum)
{
	int
		loop;

	loop = 0;

	while (session_preset_time_of_day_table [loop].preset_enum >= 0)
	{
		if (session_preset_time_of_day_table [loop].preset_enum == preset_enum)
		{
			return session_preset_time_of_day_table [loop].time;
		}

		loop ++;
	}

	debug_fatal ("SESSION: No time of day specified for preset %d", preset_enum);

	return 0.0;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

// EECH_CAMPAIGN_HOST_ENV (EEDCS-51, D3): the weather at a point is the host's (ss_hostenv.h); the original solver is below, for the original game's build only
void get_session_weather_at_point (vec3d *pos, weathermodes *current_weather_mode, weathermodes *target_weather_mode, float *transitional_state)
{
	int
		cwm,
		twm,
		simple;

	float
		ts;

	ASSERT (get_session_entity ());

	ASSERT (pos);

	campaign_env_weather_at (pos->x, pos->z, &cwm, &twm, &ts, &simple);

	if (current_weather_mode)
	{
		*current_weather_mode = (weathermodes) cwm;
	}

	if (target_weather_mode)
	{
		*target_weather_mode = (weathermodes) twm;
	}

	if (transitional_state)
	{
		*transitional_state = ts;
	}
}

#else

void get_session_weather_at_point (vec3d *pos, weathermodes *current_weather_mode, weathermodes *target_weather_mode, float *transitional_state)
{
	float
		range;

	session
		*raw;

	weathermodes
		cwm,
		twm;

	float
		ts;

	ASSERT (get_session_entity ());

	ASSERT (pos);

	raw = (session *) get_local_entity_data (get_session_entity ());

	if (raw->local_weather_model)
	{
		//
		// local weather model
		//

		if (raw->weather_radius == 0.0)
		{
			//
			// weather is fine all across the map
			//

			cwm = WEATHERMODE_DRY;
			twm = WEATHERMODE_DRY;

			ts = 1.0;
		}
		else
		{
			//
			// there is some bad weather somewhere on the map
			//

			range = get_approx_2d_range (pos, &raw->weather_position);

			if (range >= raw->weather_radius)
			{
				//
				// camera is in the dry zone
				//

				cwm = WEATHERMODE_DRY;
				twm = WEATHERMODE_DRY;

				ts = 0.0;
			}
			else if (range >= (raw->weather_radius * 0.666))
			{
				//
				// camera is in the dry / light rain zone
				//

				cwm = WEATHERMODE_DRY;
				//VJ 051014 link winter/summer to snow/rain
				if (get_global_season() == SESSION_SEASON_WINTER)
					twm = WEATHERMODE_SNOW;
				else
					twm = WEATHERMODE_LIGHT_RAIN;

				ts = (raw->weather_radius - range) / (raw->weather_radius * 0.333);
			}
			else if (range >= (raw->weather_radius * 0.333))
			{
				//
				// camera is in the light rain / heavy rain zone
				//

				//VJ 051014 link winter/summer to snow/rain
				if (get_global_season() == SESSION_SEASON_WINTER)
				{
					cwm = WEATHERMODE_SNOW;
					twm = WEATHERMODE_SNOW;
				}
				else
				{
					cwm = WEATHERMODE_LIGHT_RAIN;
					twm = WEATHERMODE_HEAVY_RAIN;
				}

				ts = ((raw->weather_radius * 0.666) - range) / (raw->weather_radius * 0.333);
			}
			else
			{
				//
				// camera is in the heavy rain zone
				//

				//VJ 051014 link winter/summer to snow/rain
				if (get_global_season() == SESSION_SEASON_WINTER)
				{
					cwm = WEATHERMODE_SNOW;
					twm = WEATHERMODE_SNOW;
				}
				else
				{
					cwm = WEATHERMODE_HEAVY_RAIN;
					twm = WEATHERMODE_HEAVY_RAIN;
				}
				ts = 0.0;
			}

			ts = bound (ts, 0.0, 1.0);
		}
	}
	else
	{
		//
		// global weather model
		//

		cwm = raw->weather_mode;
		twm = raw->target_weather_mode;

		ts = raw->weather_mode_transitional_status;
	}

	//
	// return required values
	//

	if (current_weather_mode)
	{
		*current_weather_mode = cwm;
	}

	if (target_weather_mode)
	{
		*target_weather_mode = twm;
	}

	if (transitional_state)
	{
		*transitional_state = ts;
	}
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

weathermodes get_simple_session_weather_at_point (vec3d *pos)
{
	int
		cwm,
		twm,
		simple;

	float
		ts;

	ASSERT (get_session_entity ());

	ASSERT (pos);

	campaign_env_weather_at (pos->x, pos->z, &cwm, &twm, &ts, &simple);

	return (weathermodes) simple;
}

#else

weathermodes get_simple_session_weather_at_point (vec3d *pos)
{
	weathermodes
		cwm,
		twm;

	float
		transitional_status;

	get_session_weather_at_point (pos, &cwm, &twm, &transitional_status);

	if (transitional_status < 0.5)
	{
		return cwm;
	}
	else
	{
		return twm;
	}
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

void set_session_fixed_weather_mode (entity *en, weathermodes weather_mode)
{
	session
		*raw;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (en);

	raw = (session *) get_local_entity_data (en);

	ENV_FX (ENV_FX_FIXED_WEATHER, (int) weather_mode, 0, 0, 0, 0, 0, 0, 0)

	campaign_env_init_fixed ((struct SESSION *) raw, (int) weather_mode);
}

#else

void set_session_fixed_weather_mode (entity *en, weathermodes weather_mode)
{
	session
		*raw;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (en);

	raw = (session *) get_local_entity_data (en);

	ENV_FX (ENV_FX_FIXED_WEATHER, (int) weather_mode, 0, 0, 0, 0, 0, 0, 0)

	raw->local_weather_model = FALSE;

	raw->weather_mode = weather_mode;
	raw->target_weather_mode = weather_mode;

	raw->weather_mode_transitional_status = 0.0;
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

// The ten draws of the shared random word are made HERE, in the original's order (rain: radius, direction flag, x, z, heading; wind: the same five), each
// where the original made it, and are the only ones; the host applies the original's expressions to them (ss_hostenv.h INIT_RANDOM).
void set_session_random_weather_mode (entity *en)
{
	session
		*raw;

	double
		draw[CAMPAIGN_ENV_DRAWS];

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (en);

	raw = (session *) get_local_entity_data (en);

	ENV_FX (ENV_FX_WEATHER_BEGIN, 0, 0, 0, 0, 0, 0, 0, 0)

	//
	// Rain
	//

	draw[0] = frand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 1, 0, ENV_FXF ((float) draw[0]), 0, 0, 0, 0, 0)

	draw[1] = (double) rand16 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 2, ((int) draw[1]) & 1, 0, 0, 0, 0, 0, 0)

	draw[2] = frand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 3, 0, ENV_FXF ((float) draw[2]), 0, 0, 0, 0, 0)

	draw[3] = frand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 4, 0, ENV_FXF ((float) draw[3]), 0, 0, 0, 0, 0)

	draw[4] = sfrand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 5, 0, ENV_FXF ((float) draw[4]), 0, 0, 0, 0, 0)

	//
	// Wind
	//

	draw[5] = frand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 6, 0, ENV_FXF ((float) draw[5]), 0, 0, 0, 0, 0)

	draw[6] = (double) rand16 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 7, ((int) draw[6]) & 1, 0, 0, 0, 0, 0, 0)

	draw[7] = frand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 8, 0, ENV_FXF ((float) draw[7]), 0, 0, 0, 0, 0)

	draw[8] = frand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 9, 0, ENV_FXF ((float) draw[8]), 0, 0, 0, 0, 0)

	draw[9] = sfrand1 ();

	ENV_FX (ENV_FX_WEATHER_DRAW, 10, 0, ENV_FXF ((float) draw[9]), 0, 0, 0, 0, 0)

	campaign_env_init_random ((struct SESSION *) raw, draw);

	ENV_FX (ENV_FX_WEATHER_END, 0, 0, 0, 0, 0, 0, 0, 0)
}

#else

void set_session_random_weather_mode (entity *en)
{
	session
		*raw;

	float
		theta;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (en);

	raw = (session *) get_local_entity_data (en);

	ENV_FX (ENV_FX_WEATHER_BEGIN, 0, 0, 0, 0, 0, 0, 0, 0)

	raw->local_weather_model = TRUE;

	//
	// Rain
	//

	raw->weather_radius = weather_rain_radius_min + ((weather_rain_radius_max - weather_rain_radius_min) * frand1 ());

	ENV_FX (ENV_FX_WEATHER_DRAW, 1, 0, ENV_FXF (raw->weather_radius), 0, 0, 0, 0, 0)

	raw->weather_increasing = (rand16 () & 1);

	ENV_FX (ENV_FX_WEATHER_DRAW, 2, raw->weather_increasing, 0, 0, 0, 0, 0, 0)

	raw->weather_position.x = MIN_MAP_X + ((MAX_MAP_X - MIN_MAP_X) * frand1 ());
	raw->weather_position.y = 0.0;
	ENV_FX (ENV_FX_WEATHER_DRAW, 3, 0, ENV_FXF (raw->weather_position.x), 0, 0, 0, 0, 0)
	raw->weather_position.z = MIN_MAP_Z + ((MAX_MAP_Z - MIN_MAP_Z) * frand1 ());

	ENV_FX (ENV_FX_WEATHER_DRAW, 4, 0, ENV_FXF (raw->weather_position.z), 0, 0, 0, 0, 0)

	theta = PI * sfrand1 ();
	ENV_FX (ENV_FX_WEATHER_DRAW, 5, 0, ENV_FXF (theta), 0, 0, 0, 0, 0)

	raw->weather_velocity.x = sin (theta) * weather_rain_speed;
	raw->weather_velocity.y = 0.0;
	raw->weather_velocity.z = cos (theta) * weather_rain_speed;

	//
	// Wind
	//

	raw->wind_effect_radius = weather_wind_radius_min + ((weather_wind_radius_max - weather_wind_radius_min) * frand1 ());

	ENV_FX (ENV_FX_WEATHER_DRAW, 6, 0, ENV_FXF (raw->wind_effect_radius), 0, 0, 0, 0, 0)

	raw->wind_increasing = (rand16 () & 1);

	ENV_FX (ENV_FX_WEATHER_DRAW, 7, raw->wind_increasing, 0, 0, 0, 0, 0, 0)

	raw->wind_effect_position.x = MIN_MAP_X + ((MAX_MAP_X - MIN_MAP_X) * frand1 ());
	raw->wind_effect_position.y = 0.0;
	ENV_FX (ENV_FX_WEATHER_DRAW, 8, 0, ENV_FXF (raw->wind_effect_position.x), 0, 0, 0, 0, 0)
	raw->wind_effect_position.z = MIN_MAP_Z + ((MAX_MAP_Z - MIN_MAP_Z) * frand1 ());

	ENV_FX (ENV_FX_WEATHER_DRAW, 9, 0, ENV_FXF (raw->wind_effect_position.z), 0, 0, 0, 0, 0)

	theta = PI * sfrand1 ();
	ENV_FX (ENV_FX_WEATHER_DRAW, 10, 0, ENV_FXF (theta), 0, 0, 0, 0, 0)

	raw->wind_effect_velocity.x = sin (theta) * weather_wind_speed;
	raw->wind_effect_velocity.y = 0.0;
	raw->wind_effect_velocity.z = cos (theta) * weather_wind_speed;

	ENV_FX (ENV_FX_WEATHER_END, 0, 0, 0, 0, 0, 0, 0, 0)
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

float get_session_wind_velocity_at_point (vec3d *pos, vec3d *wind_velocity)
{
	float
		wind_speed,
		vector[3];

	ASSERT (get_session_entity ());

	ASSERT (pos);

	wind_speed = campaign_env_wind_at (pos->x, pos->z, wind_velocity ? vector : NULL);

	if (wind_velocity)
	{
		wind_velocity->x = vector[0];
		wind_velocity->y = vector[1];
		wind_velocity->z = vector[2];
	}

	return wind_speed;
}

#else

float get_session_wind_velocity_at_point (vec3d *pos, vec3d *wind_velocity)
{
	float
		r,
		range,
		wind_speed;

	session
		*raw;

	ASSERT (get_session_entity ());

	ASSERT (pos);

	raw = (session *) get_local_entity_data (get_session_entity ());

	if (raw->wind_effect_radius == 0.0)
	{
		//
		// wind is calm all across the map
		//

		wind_speed = raw->wind_minimum_speed;
	}
	else
	{
		//
		// there is some high wind somewhere on the map
		//

		range = get_approx_2d_range (pos, &raw->wind_effect_position);

		if (range >= raw->wind_effect_radius)
		{
			//
			// camera is in the calm zone
			//

			wind_speed = raw->wind_minimum_speed;
		}
		else
		{
			r = ((raw->wind_effect_radius - range) / raw->wind_effect_radius);

			wind_speed = raw->wind_minimum_speed + ((raw->wind_maximum_speed - raw->wind_minimum_speed) * r);
		}
	}

	wind_speed += (wind_speed * get_wind_gusting_modulation_value (raw->wind_gusting_value));

	if (wind_velocity)
	{
		wind_velocity->x = raw->wind_direction_vector.x * wind_speed;
		wind_velocity->y = 0.0;
		wind_velocity->z = raw->wind_direction_vector.z * wind_speed;
	}

	return wind_speed;
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void attach_session_sound_effects (entity *en)
{
	sound_sample_indices
		sound_effect_index;

	//
	// light rain
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMBIENT_LIGHT_RAIN;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_LIGHT_RAIN,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							// sample index list
	);

	//
	// heavy rain
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMBIENT_HEAVY_RAIN;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_HEAVY_RAIN,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							// sample index list
	);

	//
	// light wind
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMBIENT_LIGHT_WIND;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_LIGHT_WIND,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							// sample index list
	);

	//
	// heavy wind
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMBIENT_HEAVY_WIND;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_HEAVY_WIND,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							// sample index list
	);

	//
	// sea
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMBIENT_SEA;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_SEA,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							// sample index list
	);

	//
	// jungle
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMBIENT_JUNGLE;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_JUNGLE,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							       // sample index list

	);

	//
	// Maverick's Ambiente sound mod
	//

	sound_effect_index = SOUND_SAMPLE_INDEX_AMB1;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMB1,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							        // sample index list
	);

    sound_effect_index = SOUND_SAMPLE_INDEX_AMB2;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMB2,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							        // sample index list
	);

    sound_effect_index = SOUND_SAMPLE_INDEX_AMB3;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMB3,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							        // sample index list
    );

    sound_effect_index = SOUND_SAMPLE_INDEX_AMB4;

	create_local_sound_effect_entity
	(
		ENTITY_INDEX_DONT_CARE,
		en,
		ENTITY_SIDE_NEUTRAL,
		ENTITY_SUB_TYPE_EFFECT_SOUND_AMB4,
		SOUND_CHANNEL_SOUND_EFFECT,
		SOUND_LOCALITY_ALL,
		NULL,												// position
		0.0,												// amplification
		1.0, //Werewolf pitch
		TRUE,												// valid sound effect
		TRUE,												// looping
		1,													// sample count
		&sound_effect_index							        // sample index list
	);
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_CAMPAIGN_HOST_ENV

/* the gust modulation is the host's (ss_hostenv.h GUST); the original is below, for the original game's build only */

#else

float get_wind_gusting_modulation_value (float wind_gusting_value)
{
	float
		a,
		b,
		d,
		low,
		high,
		val;

	int
		low_index,
		high_index,
		num_segments;

	num_segments = WIND_GUSTING_SAMPLE_POINTS + 1;

	val = wind_gusting_value * num_segments;

	low = floor (val);

	high = ceil (val);

	d = val - low;

	convert_float_to_int (low, &low_index);
	convert_float_to_int (high, &high_index);

	ASSERT ((low_index >= 0) && (low_index < num_segments));
	ASSERT ((high_index >= 0) && (high_index <= num_segments));

	if (low_index == 0)
	{
		a = 0.0;
	}
	else
	{
		a = wind_gust_modulation_amounts [low_index - 1];
	}

	if ((high_index == 0) || (high_index == num_segments))
	{
		b = 0.0;
	}
	else
	{
		b = wind_gust_modulation_amounts [high_index - 1];
	}

	return (a + (d * (b - a)));
}

#endif

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

int check_point_inside_population_area (vec3d *point)
{

	session
		*raw;

	debug_assert (get_session_entity ());

	raw = (session *) get_local_entity_data (get_session_entity ());

	if ((raw->population_x_min < point->x) && (raw->population_x_max > point->x))
	{

		if ((raw->population_z_min < point->z) && (raw->population_z_max > point->z))
		{

			return TRUE;
		}
	}

	return FALSE;
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#ifdef EECH_ENV_FIXTURE

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// TEST-ONLY (EEDCS-51, feature env-fixture): the static gust modulation, called unchanged. Not compiled in the shipping build.
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

float eech_env_fixture_gust_modulation (float wind_gusting_value)
{
#ifdef EECH_CAMPAIGN_HOST_ENV
	return campaign_env_gust_modulation (wind_gusting_value);
#else
	return get_wind_gusting_modulation_value (wind_gusting_value);
#endif
}

#endif
