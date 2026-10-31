/*
 * The campaign's half of the host-owned weather and wind field (EEDCS-51, D3): see ss_hostenv.h.  This file holds no weather or wind arithmetic, no ring
 * model, no gust table and no lightning timer arithmetic.  It carries each question of session.c and ss_updt.c to the host, checks the answer, and keeps the
 * field the host answered in the session record, which is a read-only MIRROR: the campaign never evolves it.  The draws of the shared random word that the
 * original made inside the physical code are made here, in the original order and at the original site, and handed to the host as values.
 */
#include "project.h"

#include <math.h>
#include <string.h>

#include "entity/special/session/ss_hostenv.h"

#include "entity/special/session/ss_envfx.h"

#ifdef EECH_CAMPAIGN_HOST_ENV

static campaign_env_host
	current_host;

static int
	host_installed;

void campaign_env_set_host (const campaign_env_host *host)
{
	if (host && host->environment)
	{
		current_host = *host;

		host_installed = TRUE;
	}
}

// one request, one reply.  A missing provider, a refusal and a malformed answer all stop the engine with an error that names the environment provider:
// there is no campaign solver to fall back to, and nothing the campaign holds is changed before the answer has been checked
static void ask (campaign_env_request *request, campaign_env_reply *reply, const char *what)
{
	char
		error[CAMPAIGN_ENV_ERROR_MAX + 1];

	if (!host_installed)
	{
		debug_fatal ("host environment: [env-provider:missing] no environment provider is installed (%s)", what);
	}

	// the season is read where the original read it, at each question
	request->winter = (get_global_season () == SESSION_SEASON_WINTER);

	memset (reply, 0, sizeof (*reply));

	memset (error, 0, sizeof (error));

	if (current_host.environment (request, reply, error, CAMPAIGN_ENV_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host environment: %s", error[0] ? error : "[env-provider:callback] the host environment provider failed without a message");
	}
}

static int all_finite (const float *values, int count)
{
	int
		i;

	for (i = 0; i < count; i++)
	{
		if (!isfinite (values[i]))
		{
			return FALSE;
		}
	}

	return TRUE;
}

static void check_state (const campaign_env_state *state, const char *what)
{
	int
		i;

	if (!all_finite (state->f, CAMPAIGN_ENV_FLOATS))
	{
		debug_fatal ("host environment: [env-provider:%s.state] the host's field has a number that is not finite", what);
	}

	for (i = CAMPAIGN_ENV_N_WEATHER_MODE; i <= CAMPAIGN_ENV_N_TARGET_WEATHER_MODE; i++)
	{
		if ((state->n[i] < 0) || (state->n[i] >= WEATHERMODE_LAST))
		{
			debug_fatal ("host environment: [env-provider:%s.state] the host's weather mode %d is not one of the modes", what, state->n[i]);
		}
	}

	for (i = CAMPAIGN_ENV_N_LOCAL_WEATHER_MODEL; i < CAMPAIGN_ENV_INTS; i++)
	{
		if ((state->n[i] != FALSE) && (state->n[i] != TRUE))
		{
			debug_fatal ("host environment: [env-provider:%s.state] the host's flag %d is neither 0 nor 1", what, state->n[i]);
		}
	}
}

////////////////////////////////////////
//
// the mirror: the session record's physical fields and the host's field
//
////////////////////////////////////////

static void state_from_raw (campaign_env_state *state, const session *raw)
{
	memset (state, 0, sizeof (*state));

	state->f[CAMPAIGN_ENV_F_WEATHER_RADIUS] = raw->weather_radius;
	state->f[CAMPAIGN_ENV_F_TRANSITIONAL_PERIOD] = raw->weather_mode_transitional_period;
	state->f[CAMPAIGN_ENV_F_TRANSITIONAL_STATUS] = raw->weather_mode_transitional_status;
	state->f[CAMPAIGN_ENV_F_WEATHER_POSITION_X] = raw->weather_position.x;
	state->f[CAMPAIGN_ENV_F_WEATHER_POSITION_Y] = raw->weather_position.y;
	state->f[CAMPAIGN_ENV_F_WEATHER_POSITION_Z] = raw->weather_position.z;
	state->f[CAMPAIGN_ENV_F_WEATHER_VELOCITY_X] = raw->weather_velocity.x;
	state->f[CAMPAIGN_ENV_F_WEATHER_VELOCITY_Y] = raw->weather_velocity.y;
	state->f[CAMPAIGN_ENV_F_WEATHER_VELOCITY_Z] = raw->weather_velocity.z;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_RADIUS] = raw->wind_effect_radius;
	state->f[CAMPAIGN_ENV_F_WIND_GUSTING_VALUE] = raw->wind_gusting_value;
	state->f[CAMPAIGN_ENV_F_WIND_MINIMUM_SPEED] = raw->wind_minimum_speed;
	state->f[CAMPAIGN_ENV_F_WIND_MAXIMUM_SPEED] = raw->wind_maximum_speed;
	state->f[CAMPAIGN_ENV_F_WIND_DIRECTION_X] = raw->wind_direction_vector.x;
	state->f[CAMPAIGN_ENV_F_WIND_DIRECTION_Y] = raw->wind_direction_vector.y;
	state->f[CAMPAIGN_ENV_F_WIND_DIRECTION_Z] = raw->wind_direction_vector.z;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_X] = raw->wind_effect_position.x;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_Y] = raw->wind_effect_position.y;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_Z] = raw->wind_effect_position.z;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_X] = raw->wind_effect_velocity.x;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_Y] = raw->wind_effect_velocity.y;
	state->f[CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_Z] = raw->wind_effect_velocity.z;
	state->f[CAMPAIGN_ENV_F_LIGHTNING_TIMER] = raw->lightning_timer;

	state->n[CAMPAIGN_ENV_N_WEATHER_MODE] = (int) raw->weather_mode;
	state->n[CAMPAIGN_ENV_N_TARGET_WEATHER_MODE] = (int) raw->target_weather_mode;
	state->n[CAMPAIGN_ENV_N_LOCAL_WEATHER_MODEL] = (int) raw->local_weather_model;
	state->n[CAMPAIGN_ENV_N_WEATHER_INCREASING] = (int) raw->weather_increasing;
	state->n[CAMPAIGN_ENV_N_WIND_INCREASING] = (int) raw->wind_increasing;
}

static void raw_from_state (session *raw, const campaign_env_state *state)
{
	raw->weather_radius = state->f[CAMPAIGN_ENV_F_WEATHER_RADIUS];
	raw->weather_mode_transitional_period = state->f[CAMPAIGN_ENV_F_TRANSITIONAL_PERIOD];
	raw->weather_mode_transitional_status = state->f[CAMPAIGN_ENV_F_TRANSITIONAL_STATUS];
	raw->weather_position.x = state->f[CAMPAIGN_ENV_F_WEATHER_POSITION_X];
	raw->weather_position.y = state->f[CAMPAIGN_ENV_F_WEATHER_POSITION_Y];
	raw->weather_position.z = state->f[CAMPAIGN_ENV_F_WEATHER_POSITION_Z];
	raw->weather_velocity.x = state->f[CAMPAIGN_ENV_F_WEATHER_VELOCITY_X];
	raw->weather_velocity.y = state->f[CAMPAIGN_ENV_F_WEATHER_VELOCITY_Y];
	raw->weather_velocity.z = state->f[CAMPAIGN_ENV_F_WEATHER_VELOCITY_Z];
	raw->wind_effect_radius = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_RADIUS];
	raw->wind_gusting_value = state->f[CAMPAIGN_ENV_F_WIND_GUSTING_VALUE];
	raw->wind_minimum_speed = state->f[CAMPAIGN_ENV_F_WIND_MINIMUM_SPEED];
	raw->wind_maximum_speed = state->f[CAMPAIGN_ENV_F_WIND_MAXIMUM_SPEED];
	raw->wind_direction_vector.x = state->f[CAMPAIGN_ENV_F_WIND_DIRECTION_X];
	raw->wind_direction_vector.y = state->f[CAMPAIGN_ENV_F_WIND_DIRECTION_Y];
	raw->wind_direction_vector.z = state->f[CAMPAIGN_ENV_F_WIND_DIRECTION_Z];
	raw->wind_effect_position.x = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_X];
	raw->wind_effect_position.y = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_Y];
	raw->wind_effect_position.z = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_Z];
	raw->wind_effect_velocity.x = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_X];
	raw->wind_effect_velocity.y = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_Y];
	raw->wind_effect_velocity.z = state->f[CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_Z];
	raw->lightning_timer = state->f[CAMPAIGN_ENV_F_LIGHTNING_TIMER];

	raw->weather_mode = (weathermodes) state->n[CAMPAIGN_ENV_N_WEATHER_MODE];
	raw->target_weather_mode = (weathermodes) state->n[CAMPAIGN_ENV_N_TARGET_WEATHER_MODE];
	raw->local_weather_model = state->n[CAMPAIGN_ENV_N_LOCAL_WEATHER_MODEL];
	raw->weather_increasing = state->n[CAMPAIGN_ENV_N_WEATHER_INCREASING];
	raw->wind_increasing = state->n[CAMPAIGN_ENV_N_WIND_INCREASING];
}

void campaign_env_import (const struct SESSION *raw_in)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_IMPORT;

	state_from_raw (&request.state, (const session *) raw_in);

	ask (&request, &reply, "import");
}

void campaign_env_refresh (struct SESSION *raw_in)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_EXPORT;

	ask (&request, &reply, "export");

	check_state (&reply.state, "export");

	raw_from_state ((session *) raw_in, &reply.state);
}

////////////////////////////////////////
//
// initialisation
//
////////////////////////////////////////

void campaign_env_init_random (struct SESSION *raw_in, const double *draw)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	int
		i;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_INIT_RANDOM;

	for (i = 0; i < CAMPAIGN_ENV_DRAWS; i++)
	{
		request.draw[i] = draw[i];
	}

	request.init_config[0] = weather_rain_radius_min;
	request.init_config[1] = weather_rain_radius_max;
	request.init_config[2] = weather_rain_speed;
	request.init_config[3] = weather_wind_radius_min;
	request.init_config[4] = weather_wind_radius_max;
	request.init_config[5] = weather_wind_speed;

	request.map[0] = MIN_MAP_X;
	request.map[1] = MAX_MAP_X;
	request.map[2] = MIN_MAP_Z;
	request.map[3] = MAX_MAP_Z;
	request.map[4] = MID_MAP_X;
	request.map[5] = MID_MAP_Z;

	ask (&request, &reply, "init_random");

	check_state (&reply.state, "init_random");

	raw_from_state ((session *) raw_in, &reply.state);
}

void campaign_env_init_fixed (struct SESSION *raw_in, int mode)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_INIT_FIXED;

	request.fixed_mode = mode;

	ask (&request, &reply, "init_fixed");

	check_state (&reply.state, "init_fixed");

	raw_from_state ((session *) raw_in, &reply.state);
}

////////////////////////////////////////
//
// the questions
//
////////////////////////////////////////

void campaign_env_weather_at (float x, float z, int *current_mode, int *target_mode, float *transitional_state, int *simple_mode)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_WEATHER;

	request.x = x;
	request.z = z;

	ask (&request, &reply, "weather");

	if ((reply.current_mode < 0) || (reply.current_mode >= WEATHERMODE_LAST) || (reply.target_mode < 0) || (reply.target_mode >= WEATHERMODE_LAST) ||
		(reply.simple_mode < 0) || (reply.simple_mode >= WEATHERMODE_LAST) || !isfinite (reply.transitional_state))
	{
		debug_fatal ("host environment: [env-provider:weather.answer] the host's weather answer is malformed");
	}

	*current_mode = reply.current_mode;
	*target_mode = reply.target_mode;
	*transitional_state = reply.transitional_state;
	*simple_mode = reply.simple_mode;
}

float campaign_env_wind_at (float x, float z, float *vector)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_WIND;

	request.x = x;
	request.z = z;
	request.want_vector = (vector != NULL);

	ask (&request, &reply, "wind");

	// the speed is always required; the vector only when the campaign asked for it (an unrequested vector is not read, as at the Rust boundary, which does not read it either)
	if (!isfinite (reply.speed) || (request.want_vector && !all_finite (reply.vector, 3)))
	{
		debug_fatal ("host environment: [env-provider:wind.answer] the host's wind answer is malformed");
	}

	if (vector)
	{
		vector[0] = reply.vector[0];
		vector[1] = reply.vector[1];
		vector[2] = reply.vector[2];
	}

	return reply.speed;
}

float campaign_env_gust_modulation (float phase)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_GUST;

	request.phase = phase;

	ask (&request, &reply, "gust");

	if (!isfinite (reply.modulation))
	{
		debug_fatal ("host environment: [env-provider:gust.answer] the host's gust answer is malformed");
	}

	return reply.modulation;
}

// ss_updt.c update_session_sound_effects: the rain and wind levels of the ambient sound children.  The campaign hands over what it observed (the point, its height, its radar
// altitude, the cloud base height, whether the cockpit is the point) and takes four levels back; it computes none of them.  The answer is checked whole BEFORE any output is written.
void campaign_env_ambient (int in_cockpit, float x, float y, float z, float alt, float cloud_base, float *rain_light, float *rain_heavy, float *wind_light, float *wind_heavy)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_AMBIENT;

	request.x = x;
	request.z = z;
	request.in_cockpit = (in_cockpit != 0);
	request.ambient_y = y;
	request.ambient_alt = alt;
	request.ambient_cloud_base = cloud_base;

	ask (&request, &reply, "ambient");

	if (!all_finite (reply.ambient_levels, 4) || (reply.ambient_levels[2] < 0.0) || (reply.ambient_levels[2] > 1.0) || (reply.ambient_levels[3] < 0.0) || (reply.ambient_levels[3] > 1.0))
	{
		debug_fatal ("host environment: [env-provider:ambient.answer] the host's ambient sound answer is malformed (four finite levels, the wind levels within 0 to 1)");
	}

	*rain_light = reply.ambient_levels[0];
	*rain_heavy = reply.ambient_levels[1];
	*wind_light = reply.ambient_levels[2];
	*wind_heavy = reply.ambient_levels[3];
}

////////////////////////////////////////
//
// the physical update
//
////////////////////////////////////////

int campaign_env_step (struct SESSION *raw_in, int in_cockpit, const float *camera, float *strike, float *new_timer)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_STEP;

	// the delta is read where the original read it: the delta of the previous frame (the clock advances after the updates)
	request.delta = get_delta_time ();
	request.server = (get_comms_model () == COMMS_MODEL_SERVER);
	request.in_cockpit = (in_cockpit != 0);

	if (camera)
	{
		request.have_camera = TRUE;
		request.camera[0] = camera[0];
		request.camera[1] = camera[1];
		request.camera[2] = camera[2];
	}

	request.map[0] = MIN_MAP_X;
	request.map[1] = MAX_MAP_X;
	request.map[2] = MIN_MAP_Z;
	request.map[3] = MAX_MAP_Z;
	request.map[4] = MID_MAP_X;
	request.map[5] = MID_MAP_Z;

	ask (&request, &reply, "step");

	// the protocol of the lightning timer is checked here against the request the campaign built, BEFORE the shared random word is drawn from and before the mirror is written: only a server
	// whose timer expired is ever asked for a draw (a client's next timer is set by the server), and a server's expiry is always negotiated through that question
	if (reply.need_draw && ((reply.need_draw != TRUE) || !request.server))
	{
		debug_fatal ("host environment: [env-provider:step.answer] the host asked for the lightning draw where none may be asked for");
	}

	if (!reply.need_draw && request.server && reply.expired)
	{
		debug_fatal ("host environment: [env-provider:step.answer] a server's lightning timer expired without the draw the new timer needs");
	}

	if (reply.need_draw)
	{
		// the lightning timer expired and the next timer needs the one draw of the shared word the original made at this point (update_lightning_effect, straight
		// after the decrement): nothing was changed by the host; the draw is made now, once, and the question asked again with it
		ENV_FX (ENV_FX_LIGHTNING_EXPIRED, request.server, 0, ENV_FXF (reply.new_timer), ENV_FXF (request.delta), 0, 0, 0, 0)

		request.have_draw = TRUE;
		request.lightning_draw = frand1 ();

		ask (&request, &reply, "step");

		if (reply.need_draw)
		{
			debug_fatal ("host environment: [env-provider:step.answer] the host asked for a second draw of the lightning timer");
		}

		// the draw was made for an expiry: the final answer must report that expiry
		if (reply.expired != TRUE)
		{
			debug_fatal ("host environment: [env-provider:step.answer] the lightning draw was made for an expiry the host's final answer does not report");
		}
	}

	check_state (&reply.state, "step");

	if (((reply.expired != FALSE) && (reply.expired != TRUE)) || (reply.expired && (!all_finite (reply.strike, 3) || (request.server && !isfinite (reply.new_timer)))))
	{
		debug_fatal ("host environment: [env-provider:step.answer] the host's step answer is malformed");
	}

	raw_from_state ((session *) raw_in, &reply.state);

	if (reply.expired)
	{
		strike[0] = reply.strike[0];
		strike[1] = reply.strike[1];
		strike[2] = reply.strike[2];

		*new_timer = request.server ? reply.new_timer : 0.0f;

		// only the server draws and sets a next timer (a client's is set by the server): the original recorded nothing for a client's expiry
		if (request.server)
		{
			ENV_FX (ENV_FX_LIGHTNING_NEW_TIMER, 0, 0, ENV_FXF (reply.new_timer), 0, 0, 0, 0, 0)
		}
	}

	return reply.expired;
}


////////////////////////////////////////
//
// the physical lightning queue (modules/3d/3dstrike.c, EEDCS-51 D3)
//
// The 32 slots, the first-free choice, the drop of a strike when every slot is in use, the time stamp, the aging and the expiry are the HOST's.  The campaign keeps what the original did around
// them (it switches the 3D environment, it decides when a strike is added and where) and a read-only MIRROR of the queue for the routines that read it (the draw routines and the observer).
// Every answer is checked whole -- against the request and against the mirror as it stood -- before anything is written to the mirror, so a malformed answer changes nothing and the engine stops
// with an error that names the environment provider.  The exact bit patterns are compared through their addresses (no float is loaded for the comparison).
//
////////////////////////////////////////

static int same_bits (const float *a, const float *b)
{
	return memcmp (a, b, sizeof (float)) == 0;
}

static int strike_finite (const campaign_lightning_strike *s)
{
	return isfinite (s->x) && isfinite (s->y) && isfinite (s->z) && isfinite (s->time_started) && isfinite (s->time_length);
}

// a strike that stays in use keeps everything it was given; only its frame number may change
static int strike_kept (const campaign_lightning_strike *was, const campaign_lightning_strike *is)
{
	return (was->type == is->type) && same_bits (&was->x, &is->x) && same_bits (&was->y, &is->y) && same_bits (&was->z, &is->z) && same_bits (&was->time_started, &is->time_started) &&
		same_bits (&was->time_length, &is->time_length);
}

static void check_lightning_queue (const campaign_lightning_strike *queue, const char *what)
{
	int
		i;

	for (i = 0; i < CAMPAIGN_LIGHTNING_SLOTS; i++)
	{
		if ((queue[i].in_use != FALSE) && (queue[i].in_use != TRUE))
		{
			debug_fatal ("host environment: [env-provider:%s.answer] slot %d of the host's lightning queue has an in_use flag that is neither 0 nor 1", what, i);
		}

		if (!strike_finite (&queue[i]))
		{
			debug_fatal ("host environment: [env-provider:%s.answer] slot %d of the host's lightning queue has a number that is not finite", what, i);
		}

		if ((queue[i].type < LIGHTNING_TYPE_INVALID) || (queue[i].type > LIGHTNING_TYPE_FORKED_4))
		{
			debug_fatal ("host environment: [env-provider:%s.answer] slot %d of the host's lightning queue has the type %d, which is not a lightning type", what, i, queue[i].type);
		}
	}
}

int campaign_lightning_add (int type, float length, float x, float z, int environment_active, float time_of_day, campaign_lightning_strike *mirror)
{
	static const float
		strike_height = 6000.0f;

	campaign_env_request
		request;

	campaign_env_reply
		reply;

	int
		slot,
		i;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_LIGHTNING_ADD;

	request.lightning_type = type;
	request.lightning_length = length;
	request.x = x;
	request.z = z;
	request.lightning_environment_active = (environment_active != 0);

	// the time of day is what the original read from the active environment; with none there is nothing to read and nothing is given
	if (environment_active)
	{
		request.lightning_time_of_day = time_of_day;
	}

	ask (&request, &reply, "lightning_add");

	check_lightning_queue (reply.lightning_queue, "lightning_add");

	slot = reply.lightning_slot;

	if (reply.lightning_need_frames || (slot < -1) || (slot >= CAMPAIGN_LIGHTNING_SLOTS))
	{
		debug_fatal ("host environment: [env-provider:lightning_add.answer] the host's add answer is malformed (a slot from -1 to %d, and no request for a frame count)", CAMPAIGN_LIGHTNING_SLOTS - 1);
	}

	if ((slot >= 0) && !environment_active)
	{
		debug_fatal ("host environment: [env-provider:lightning_add.answer] the host queued a strike although no active environment was given");
	}

	for (i = 0; i < CAMPAIGN_LIGHTNING_SLOTS; i++)
	{
		if (i == slot)
		{
			// the slot the host says it filled was free, and holds exactly what this request gave, the position's height as the original set it, and frame 0
			if (mirror[i].in_use || !reply.lightning_queue[i].in_use || (reply.lightning_queue[i].type != type) || !same_bits (&reply.lightning_queue[i].x, &request.x) ||
				!same_bits (&reply.lightning_queue[i].z, &request.z) || !same_bits (&reply.lightning_queue[i].y, &strike_height) ||
				!same_bits (&reply.lightning_queue[i].time_started, &request.lightning_time_of_day) || !same_bits (&reply.lightning_queue[i].time_length, &request.lightning_length) ||
				(reply.lightning_queue[i].current_frame_number != 0))
			{
				debug_fatal ("host environment: [env-provider:lightning_add.answer] slot %d is not a free slot filled with the strike that was given", i);
			}
		}
		else if (mirror[i].in_use)
		{
			// a strike already in the queue is untouched by an add (a frame number included)
			if (!reply.lightning_queue[i].in_use || !strike_kept (&mirror[i], &reply.lightning_queue[i]) || (mirror[i].current_frame_number != reply.lightning_queue[i].current_frame_number))
			{
				debug_fatal ("host environment: [env-provider:lightning_add.answer] the add changed slot %d, which it did not fill", i);
			}
		}
		else if (reply.lightning_queue[i].in_use)
		{
			debug_fatal ("host environment: [env-provider:lightning_add.answer] the add filled slot %d but answered slot %d", i, slot);
		}
	}

	memcpy (mirror, reply.lightning_queue, sizeof (reply.lightning_queue));

	return slot;
}

void campaign_lightning_update (float time_of_day, int (*frames_of) (void), campaign_lightning_strike *mirror)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	int
		i;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_LIGHTNING_UPDATE;

	request.lightning_time_of_day = time_of_day;

	ask (&request, &reply, "lightning_update");

	if (reply.lightning_need_frames)
	{
		// a forked strike is to be aged: the frame count of the strike animation is read now, where the original read it, and only because the host needs it; nothing was changed by the host
		if (reply.lightning_need_frames != TRUE)
		{
			debug_fatal ("host environment: [env-provider:lightning_update.answer] the host's request for the frame count is neither 0 nor 1");
		}

		request.lightning_have_frames = TRUE;
		request.lightning_frames = frames_of ();

		ask (&request, &reply, "lightning_update");

		if (reply.lightning_need_frames)
		{
			debug_fatal ("host environment: [env-provider:lightning_update.answer] the host asked for the frame count a second time");
		}
	}

	check_lightning_queue (reply.lightning_queue, "lightning_update");

	if (reply.lightning_slot != -1)
	{
		debug_fatal ("host environment: [env-provider:lightning_update.answer] the host's update answer names a slot (an update fills none)");
	}

	for (i = 0; i < CAMPAIGN_LIGHTNING_SLOTS; i++)
	{
		if (!mirror[i].in_use)
		{
			if (reply.lightning_queue[i].in_use)
			{
				debug_fatal ("host environment: [env-provider:lightning_update.answer] the update filled slot %d, which was free", i);
			}
		}
		else if (reply.lightning_queue[i].in_use && !strike_kept (&mirror[i], &reply.lightning_queue[i]))
		{
			// aged: only the frame number may change; finished: freed
			debug_fatal ("host environment: [env-provider:lightning_update.answer] the update changed more than the frame number of slot %d", i);
		}
	}

	memcpy (mirror, reply.lightning_queue, sizeof (reply.lightning_queue));
}

#endif
