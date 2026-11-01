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

static int forked_type (int type)
{
	return (type >= LIGHTNING_TYPE_FORKED_1) && (type <= LIGHTNING_TYPE_FORKED_4);
}

// the frame count of the strike animation: a positive count, or a refusal that names what is missing (see 3dstrike.c) and stops the engine; the model data is never invented
static int model_frames (int (*frames_of) (void))
{
	int
		frames;

	frames = frames_of ();

	if (frames < 1)
	{
		debug_fatal ("host environment: [env-provider:lightning_update.model] the strike animation's frame count is not available (reason %d: -1 no texture animation table is loaded, -2 the strike animation is not one of its entries, -3 it has no texture index table, -4 it has no frames)", frames);
	}

	return frames;
}

void campaign_lightning_update (float time_of_day, int (*frames_of) (void), campaign_lightning_strike *mirror)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	int
		frames = 0,
		have_model = FALSE,
		live_forked = FALSE,
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

		// the request is honoured only for a forked strike the mirror really holds: with none, nothing in the queue needs a frame count and the model is not touched
		for (i = 0; i < CAMPAIGN_LIGHTNING_SLOTS; i++)
		{
			if (mirror[i].in_use && forked_type (mirror[i].type))
			{
				live_forked = TRUE;
			}
		}

		if (!live_forked)
		{
			debug_fatal ("host environment: [env-provider:lightning_update.answer] the host asked for the strike animation's frame count although the queue holds no forked strike");
		}

		frames = model_frames (frames_of);

		have_model = TRUE;

		request.lightning_have_frames = TRUE;
		request.lightning_frames = frames;

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

	// the renderer indexes the strike animation's textures by a live forked strike's frame number (3dstrike.c draw_3d_lightning_strike): that number must lie inside the model's frames BEFORE the mirror
	// holds it, however the host got there (a host that never asked for the count included). The model is consulted only when a forked strike is live in the answer, and its absence is a named refusal.
	// No frame is calculated here: the host's number is only checked against the physical domain.
	for (i = 0; i < CAMPAIGN_LIGHTNING_SLOTS; i++)
	{
		if (reply.lightning_queue[i].in_use && forked_type (reply.lightning_queue[i].type))
		{
			if (!have_model)
			{
				frames = model_frames (frames_of);

				have_model = TRUE;
			}

			if ((reply.lightning_queue[i].current_frame_number < 0) || (reply.lightning_queue[i].current_frame_number >= frames))
			{
				debug_fatal ("host environment: [env-provider:lightning_update.answer] the frame %d of the forked strike in slot %d is outside the strike animation's %d frames", reply.lightning_queue[i].current_frame_number, i, frames);
			}
		}
	}

	memcpy (mirror, reply.lightning_queue, sizeof (reply.lightning_queue));
}


////////////////////////////////////////
//
// the physical lifetime of a sound effect (soundeff.c and se_updt.c, EEDCS-51 D3): see ss_hostenv.h SOUND_LIFETIME_INIT and SOUND_LIFETIME_UPDATE.  The stored fields stay in the effect's record (the pause and
// resume functions, the network pack and unpack and the observer read and write them); they are handed over by value and the answer is applied by the caller.  This file holds no duration arithmetic, no
// cap, no aging, no countdown and no expiry rule.
//
////////////////////////////////////////

static int positive_zero (const float *value)
{
	static const float
		zero = 0.0f;

	return memcmp (value, &zero, sizeof (float)) == 0;
}

float campaign_sound_lifetime_init (int looping, int music, int count, const int *rate, const int *size)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	int
		i;

	// the host's sequence holds the sound manager's largest sequence (and one more); a longer one is refused here, never truncated
	if ((count < 1) || (count > CAMPAIGN_SOUND_SAMPLES))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_init.request] the sound effect has %d samples where the host's sequence holds 1 to %d", count, CAMPAIGN_SOUND_SAMPLES);
	}

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_SOUND_LIFETIME_INIT;

	request.sound_looping = (looping != 0);
	request.sound_music = (music != 0);
	request.sound_count = count;

	for (i = 0; i < count; i++)
	{
		request.sound_rate[i] = rate[i];
		request.sound_size[i] = size[i];
	}

	ask (&request, &reply, "sound_lifetime_init");

	if (!isfinite (reply.sound_lifetime) || (reply.sound_lifetime < 0.0f))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_init.answer] the host's lifetime is not a finite number from 0 up");
	}

	// a looping effect is not timed: the original gave it +0
	if (looping && !positive_zero (&reply.sound_lifetime))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_init.answer] the host gave a looping effect a lifetime other than 0");
	}

	return reply.sound_lifetime;
}

void campaign_sound_lifetime_update (int server, float delta, int valid, int looping, float *lifetime, float *valid_lifetime, void *handle, int (*playing_of) (void *handle), int *expired, int *action)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_SOUND_LIFETIME_UPDATE;

	request.sound_server = (server != 0);
	request.sound_valid = (valid != 0);
	request.sound_looping = (looping != 0);
	request.sound_has_handle = (handle != NULL);
	request.sound_delta = delta;
	request.sound_lifetime = *lifetime;
	request.sound_valid_lifetime = *valid_lifetime;

	ask (&request, &reply, "sound_lifetime_update");

	if (reply.sound_need_playing)
	{
		// the host decides a server's expired effect by whether its handle is still playing, which only the campaign's sound system knows: asked once, nothing was changed by the host, and only for
		// a handle the campaign really holds in the server routine of a valid effect (the original called get_system_sound_effect_playing in that branch and no other)
		if ((reply.sound_need_playing != TRUE) || !server || !valid || !handle || !playing_of)
		{
			debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host asked whether a playback handle is playing where none may be asked about (only the server routine of a valid effect that holds a handle)");
		}

		request.sound_have_playing = TRUE;
		request.sound_playing = (playing_of (handle) != 0);

		ask (&request, &reply, "sound_lifetime_update");

		if (reply.sound_need_playing)
		{
			debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host asked whether the handle is playing a second time");
		}
	}

	// the whole answer is checked against the request BEFORE anything is written: the numbers are finite and the lifetime of a valid, timed effect is the only one that may change, the valid lifetime of an
	// invalid effect the only other; an expiry belongs to a valid effect, a kill to the server's expired one and an unlink to an invalid one.  No lifetime, countdown or decision is worked out here.
	if (!isfinite (reply.sound_lifetime) || !isfinite (reply.sound_valid_lifetime))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] a lifetime in the host's answer is not finite");
	}

	if ((reply.sound_expired != FALSE) && (reply.sound_expired != TRUE))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] expired is neither 0 nor 1");
	}

	if ((reply.sound_action < CAMPAIGN_SOUND_ACTION_NONE) || (reply.sound_action > CAMPAIGN_SOUND_ACTION_UNLINK))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the action %d is not none, kill or unlink", reply.sound_action);
	}

	if ((!valid || looping) && !same_bits (&reply.sound_lifetime, lifetime))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host changed the lifetime of an effect that is invalid or looping");
	}

	if (valid && !same_bits (&reply.sound_valid_lifetime, valid_lifetime))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host changed the valid lifetime of a valid effect");
	}

	if (!valid && (reply.sound_valid_lifetime < 0.0f) && !same_bits (&reply.sound_valid_lifetime, valid_lifetime))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host's valid lifetime is below zero");
	}

	if (reply.sound_expired && !valid)
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host reports an expiry of an invalid effect");
	}

	if ((reply.sound_action == CAMPAIGN_SOUND_ACTION_KILL) && (!server || !reply.sound_expired))
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host kills an entity that is not an expired effect in the server routine");
	}

	if ((reply.sound_action == CAMPAIGN_SOUND_ACTION_UNLINK) && valid)
	{
		debug_fatal ("host environment: [env-provider:sound_lifetime_update.answer] the host unlinks a valid effect");
	}

	*lifetime = reply.sound_lifetime;
	*valid_lifetime = reply.sound_valid_lifetime;
	*expired = reply.sound_expired;
	*action = reply.sound_action;
}

////////////////////////////////////////
//
// the physical acoustics of ONE helicopter (helicop.c update_local_helicopter_rotor_sounds, EEDCS-51 D3): see ss_hostenv.h ROTOR_SOUND.  The host computes every level, pitch and the ORDERED requests; this function
// holds no threshold, no rate and no decision, and only checks the answer against the request and the protocol before the caller changes anything.
//
////////////////////////////////////////

void campaign_rotor_sound_ask (int player, float rpm, int have_flight_model, const float *acceleration, float left_rpm, float right_rpm, float apu_rpm, campaign_rotor_sound *result)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	int
		local_engines,
		expected_actions,
		i;

	float
		zero;

	// the channels the original asks for, in its order: the local role with a flight model asks turbine 1, turbine 2 and the APU first
	static const int
		engine_channels[3] = { CAMPAIGN_ROTOR_CHANNEL_TURBINE1, CAMPAIGN_ROTOR_CHANNEL_TURBINE2, CAMPAIGN_ROTOR_CHANNEL_APU };

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_ROTOR_SOUND;

	request.rotor_player = player;
	request.rotor_have_flight_model = (have_flight_model != 0);
	request.rotor_rpm = rpm;

	// read only where the original read them: the acceleration with a flight model (every role), the engines for the local role with one
	local_engines = (player == CAMPAIGN_ROTOR_PLAYER_LOCAL) && have_flight_model;

	if (have_flight_model)
	{
		request.rotor_acceleration[0] = acceleration[0];
		request.rotor_acceleration[1] = acceleration[1];
		request.rotor_acceleration[2] = acceleration[2];
	}

	if (local_engines)
	{
		request.rotor_left_rpm = left_rpm;
		request.rotor_right_rpm = right_rpm;
		request.rotor_apu_rpm = apu_rpm;
	}

	if ((player != CAMPAIGN_ROTOR_PLAYER_AI) && (player != CAMPAIGN_ROTOR_PLAYER_LOCAL) && (player != CAMPAIGN_ROTOR_PLAYER_REMOTE))
	{
		debug_fatal ("host environment: [env-provider:rotor_sound.request] the player role %d is not one of the protocol's roles", player);
	}

	ask (&request, &reply, "rotor_sound");

	// the WHOLE answer is checked before anything is written: the write masks, the levels, the requests
	for (i = 0; i < CAMPAIGN_ROTOR_CHANNELS; i++)
	{
		const int
			wanted = (i == CAMPAIGN_ROTOR_CHANNEL_WINDING) ? CAMPAIGN_ROTOR_WRITE_AMPLIFICATION : (CAMPAIGN_ROTOR_WRITE_AMPLIFICATION | CAMPAIGN_ROTOR_WRITE_PITCH);

		if (reply.rotor_write[i] != wanted)
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] the write mask %d of channel %d is not the protocol's %d", reply.rotor_write[i], i, wanted);
		}

		if (!isfinite (reply.rotor_amplification[i]))
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] the amplification of channel %d is not finite", i);
		}

		if ((wanted & CAMPAIGN_ROTOR_WRITE_PITCH) && !isfinite (reply.rotor_pitch[i]))
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] the pitch of channel %d is not finite", i);
		}
	}

	// a channel with no pitch write carries no pitch: the winding sounds keep the pitch they had
	zero = 0.0f;

	if (!same_bits (&reply.rotor_pitch[CAMPAIGN_ROTOR_CHANNEL_WINDING], &zero))
	{
		debug_fatal ("host environment: [env-provider:rotor_sound.answer] the winding channel has a pitch where it may write none");
	}

	expected_actions = local_engines ? 5 : 2;

	if (reply.rotor_action_count != expected_actions)
	{
		debug_fatal ("host environment: [env-provider:rotor_sound.answer] %d pause and resume requests where the original makes %d for this role", reply.rotor_action_count, expected_actions);
	}

	for (i = 0; i < expected_actions; i++)
	{
		const int
			wanted_channel = local_engines ? ((i < 3) ? engine_channels[i] : (i == 3 ? CAMPAIGN_ROTOR_CHANNEL_LOOPING : CAMPAIGN_ROTOR_CHANNEL_SLAP)) : (i == 0 ? CAMPAIGN_ROTOR_CHANNEL_LOOPING : CAMPAIGN_ROTOR_CHANNEL_SLAP);

		if (reply.rotor_action_channel[i] != wanted_channel)
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] request %d is for channel %d where the original asks channel %d", i, reply.rotor_action_channel[i], wanted_channel);
		}

		if ((reply.rotor_action_resume[i] != FALSE) && (reply.rotor_action_resume[i] != TRUE))
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] request %d has a resume flag that is neither 0 nor 1", i);
		}

		if (!isfinite (reply.rotor_action_delay[i]))
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] the delay of request %d is not finite", i);
		}

		if (reply.rotor_action_resume[i] && !same_bits (&reply.rotor_action_delay[i], &zero))
		{
			debug_fatal ("host environment: [env-provider:rotor_sound.answer] request %d resumes with a delay, which a resume does not carry", i);
		}
	}

	// the looping and slap requests are one decision: both pauses or both resumes
	if (reply.rotor_action_resume[expected_actions - 2] != reply.rotor_action_resume[expected_actions - 1])
	{
		debug_fatal ("host environment: [env-provider:rotor_sound.answer] the looping and slap requests are not both pauses or both resumes");
	}

	memset (result, 0, sizeof (*result));

	memcpy (result->write, reply.rotor_write, sizeof (result->write));
	memcpy (result->amplification, reply.rotor_amplification, sizeof (result->amplification));
	memcpy (result->pitch, reply.rotor_pitch, sizeof (result->pitch));

	result->action_count = reply.rotor_action_count;

	memcpy (result->action_resume, reply.rotor_action_resume, sizeof (result->action_resume));
	memcpy (result->action_channel, reply.rotor_action_channel, sizeof (result->action_channel));
	memcpy (result->action_delay, reply.rotor_action_delay, sizeof (result->action_delay));
}

////////////////////////////////////////
//
// the physical validity of ONE sound effect under a pause or a resume (soundeff.c, EEDCS-51 D3): see ss_hostenv.h SOUND_PAUSE_RESUME.  The legacy walk of the parent's children, the type and sub type match, the
// playback of the parent's sound and the update-list relink stay the caller's (private pointers and device-bound plumbing); this file holds no rule of when an effect is valid, no no-op decision, no delay copy and
// no count.  A stored valid lifetime the host leaves alone is never written (its exact word stays in the record).
//
////////////////////////////////////////

int campaign_sound_pause_resume (int resume, float delay, int member, int *valid, float *valid_lifetime, int *play, int *relink)
{
	campaign_env_request
		request;

	campaign_env_reply
		reply;

	memset (&request, 0, sizeof (request));

	request.op = CAMPAIGN_ENV_SOUND_PAUSE_RESUME;

	request.sound_resume = (resume != 0);
	request.sound_valid = (*valid != 0);
	request.sound_member = (member != 0);
	request.sound_valid_lifetime = *valid_lifetime;

	// only a pause carries a delay; a resume's request holds none
	if (!resume)
	{
		request.sound_delay = delay;
	}

	ask (&request, &reply, "sound_pause_resume");

	// the whole answer is checked against the request BEFORE anything is written.  Nothing is worked out here: an answer is only held to the fields the call may touch.
	if ((reply.sound_changed != FALSE) && (reply.sound_changed != TRUE))
	{
		debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] changed is neither 0 nor 1");
	}

	if ((reply.sound_valid != FALSE) && (reply.sound_valid != TRUE))
	{
		debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] valid is neither 0 nor 1");
	}

	if ((reply.sound_play != FALSE) && (reply.sound_play != TRUE))
	{
		debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] play is neither 0 nor 1");
	}

	if ((reply.sound_relink != FALSE) && (reply.sound_relink != TRUE))
	{
		debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] relink is neither 0 nor 1");
	}

	if (!isfinite (reply.sound_valid_lifetime))
	{
		debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the valid lifetime in the host's answer is not finite");
	}

	if (!reply.sound_changed)
	{
		if ((reply.sound_valid != request.sound_valid) || !same_bits (&reply.sound_valid_lifetime, valid_lifetime))
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host changed the validity or the valid lifetime of an effect it reports unchanged");
		}

		if (reply.sound_play || reply.sound_relink)
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host asks for playback or a relink of an effect it reports unchanged");
		}
	}
	else if (!resume)
	{
		if (!request.sound_valid || reply.sound_valid)
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host changed a pause other than a valid effect to an invalid one");
		}

		if (reply.sound_play || reply.sound_relink)
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host asks for playback or a relink on a pause");
		}
	}
	else
	{
		if (request.sound_valid || !reply.sound_valid)
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host changed a resume other than an invalid effect to a valid one");
		}

		if (!same_bits (&reply.sound_valid_lifetime, valid_lifetime))
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host changed the valid lifetime on a resume");
		}

		if (reply.sound_relink && member)
		{
			debug_fatal ("host environment: [env-provider:sound_pause_resume.answer] the host relinks an effect that is already in the update list");
		}
	}

	*valid = reply.sound_valid;

	if (reply.sound_changed && !resume)
	{
		*valid_lifetime = reply.sound_valid_lifetime;
	}

	*play = reply.sound_play;
	*relink = reply.sound_relink;

	return reply.sound_changed;
}

////////////////////////////////////////

// The host-owned physical sample metadata (ss_hostenv.h campaign_sound_catalogue_request): one exchange at initialisation, checked in full before the caller applies anything.

static campaign_sound_catalogue_host
	catalogue_host;

static int
	catalogue_host_installed;

void campaign_sound_catalogue_set_host (const campaign_sound_catalogue_host *host)
{
	if (host && host->catalogue)
	{
		catalogue_host = *host;

		catalogue_host_installed = TRUE;
	}
}

void campaign_sound_catalogue (int slots, int count, const int *index, const char *const *name, int *header_present, int *header_count, int *rate, int *original_rate, int *matched, int *size)
{
	campaign_sound_catalogue_request
		request;

	campaign_sound_catalogue_reply
		reply;

	char
		error[CAMPAIGN_ENV_ERROR_MAX + 1];

	int
		i,
		j,
		distinct_matches;

	if (!catalogue_host_installed)
	{
		debug_fatal ("host sound data: [sound-data-provider:missing] no sample metadata provider is installed");
	}

	// what the campaign asks about is checked before the host is asked: a slot outside the domain, a missing name or an empty request is the campaign's error, not the host's
	if ((slots < 1) || (count < 0) || !index || !name || !header_present || !header_count || !rate || !original_rate || !matched || !size)
	{
		debug_fatal ("host sound data: [sound-data-provider:request] the campaign's sample metadata request is not well formed (slots %d, count %d)", slots, count);
	}

	for (i = 0; i < count; i++)
	{
		if ((index[i] < 0) || (index[i] >= slots) || !name[i] || !name[i][0])
		{
			debug_fatal ("host sound data: [sound-data-provider:request] descriptor %d has slot %d (the domain is 0 to %d) or no asset name", i, index[i], slots - 1);
		}
	}

	// the host's scratch starts as values no valid answer contains, so an entry the host did not write is refused, never read as a rate of zero or a header entry of length zero
	for (i = 0; i < count; i++)
	{
		rate[i] = -1;
		original_rate[i] = -1;
		matched[i] = -1;
		size[i] = -1;
	}

	memset (&request, 0, sizeof (request));

	request.slots = slots;
	request.count = count;
	request.index = index;
	request.name = name;

	memset (&reply, 0, sizeof (reply));

	reply.header_present = -1;
	reply.header_count = -1;
	reply.rate = rate;
	reply.original_rate = original_rate;
	reply.matched = matched;
	reply.size = size;

	memset (error, 0, sizeof (error));

	if (catalogue_host.catalogue (&request, &reply, error, CAMPAIGN_ENV_ERROR_MAX, catalogue_host.user) != 0)
	{
		debug_fatal ("host sound data: %s", error[0] ? error : "[sound-data-provider:callback] the host sample metadata provider failed without a message");
	}

	if (((reply.header_present != 0) && (reply.header_present != 1)) || (reply.header_count < 0) || (!reply.header_present && (reply.header_count != 0)))
	{
		debug_fatal ("host sound data: [sound-data-provider:answer] the header answer is malformed (present %d, entries %d)", reply.header_present, reply.header_count);
	}

	for (i = 0; i < count; i++)
	{
		if ((rate[i] < 1) || (original_rate[i] < 1) || ((matched[i] != 0) && (matched[i] != 1)) || (size[i] < 0) || (!matched[i] && (size[i] != 0)))
		{
			debug_fatal ("host sound data: [sound-data-provider:answer] the answer for descriptor %d (slot %d) is malformed (rate %d, original rate %d, matched %d, length %d)", i, index[i], rate[i], original_rate[i], matched[i], size[i]);
		}
	}

	// the answer must be one a header could give: a match needs a header that holds entries, and one header entry matches one (slot, name) pair, so the distinct pairs matched cannot outnumber the entries.
	// The same question asked twice is one pair; a present header with no entry, and a matched length of 0, are both valid.
	distinct_matches = 0;

	for (i = 0; i < count; i++)
	{
		if (matched[i])
		{
			int
				repeated;

			repeated = FALSE;

			for (j = 0; j < i; j++)
			{
				if (matched[j] && (index[j] == index[i]) && (strcmp (name[j], name[i]) == 0))
				{
					repeated = TRUE;

					break;
				}
			}

			if (!repeated)
			{
				distinct_matches++;
			}
		}
	}

	if ((distinct_matches > 0) && (!reply.header_present || (distinct_matches > reply.header_count)))
	{
		debug_fatal ("host sound data: [sound-data-provider:answer] the answer matches %d distinct descriptors in a header that is %s and holds %d entries", distinct_matches, reply.header_present ? "present" : "absent", reply.header_count);
	}

	*header_present = reply.header_present;

	*header_count = reply.header_count;
}

////////////////////////////////////////

// The host-owned physical sound backend availability (ss_hostenv.h campaign_sound_backend_request): one exchange at initialisation, checked before the caller stores anything.

static campaign_sound_backend_host
	backend_host;

static int
	backend_host_installed;

void campaign_sound_backend_set_host (const campaign_sound_backend_host *host)
{
	if (host && host->backend)
	{
		backend_host = *host;

		backend_host_installed = TRUE;
	}
}

int campaign_sound_backend (int no_sound, int dedicated)
{
	campaign_sound_backend_request
		request;

	campaign_sound_backend_reply
		reply;

	char
		error[CAMPAIGN_ENV_ERROR_MAX + 1];

	if (!backend_host_installed)
	{
		debug_fatal ("host sound backend: [sound-backend-provider:missing] no sound backend provider is installed");
	}

	// what the campaign asks is checked before the host is asked: a flag that is not a flag is the campaign's error, not the host's
	if (((no_sound != 0) && (no_sound != 1)) || ((dedicated != 0) && (dedicated != 1)))
	{
		debug_fatal ("host sound backend: [sound-backend-provider:request] the campaign's flags are not flags (no_sound %d, dedicated %d)", no_sound, dedicated);
	}

	memset (&request, 0, sizeof (request));

	request.phase = CAMPAIGN_SOUND_BACKEND_PHASE_INITIALISE;
	request.no_sound = no_sound;
	request.dedicated = dedicated;
	request.load_samples = TRUE;

	// the host's answer starts as a value no valid answer contains, so a host that wrote nothing is refused, never read as "not available"
	memset (&reply, 0, sizeof (reply));

	reply.available = -1;

	memset (error, 0, sizeof (error));

	if (backend_host.backend (&request, &reply, error, CAMPAIGN_ENV_ERROR_MAX, backend_host.user) != 0)
	{
		debug_fatal ("host sound backend: %s", error[0] ? error : "[sound-backend-provider:callback] the host sound backend provider failed without a message");
	}

	if ((reply.available != 0) && (reply.available != 1))
	{
		debug_fatal ("host sound backend: [sound-backend-provider:answer] the availability answer is not 0 or 1 (%d)", reply.available);
	}

	if (reply.available)
	{
		debug_fatal ("host sound backend: [sound-backend-provider:unsupported] the host reports a physical sound backend, which is not supported until the sample loader bodies and the playback routes are the host's: the campaign has no handles for it and does not fake one");
	}

	return reply.available;
}

#endif
