/*
 * Host-owned physical weather and wind field (EEDCS-51, D3).  The host owns the physical environment: the weather zone (its centre, radius, drift, the
 * ring model and the global/fixed model), the wind zone (its centre, radius, drift, the gusts and the direction), the lightning timer and their
 * evolution, and the answers to every question about them.  Campaign code keeps what it does with them: the campaign clock (the elapsed time that
 * update_time_of_day accumulates, skip-night, the resync), every timer and schedule, the draws of the shared random word, the entity effects of a
 * lightning strike (the strikes, the sound entities) and every consequence.  The original solvers (session.c get_session_weather_at_point,
 * get_simple_session_weather_at_point, get_session_wind_velocity_at_point, the gust modulation, set_session_random_weather_mode,
 * set_session_fixed_weather_mode, and the physical part of ss_updt.c update_transitional_weather with the lightning timer of update_lightning_effect) are
 * not compiled into the headless campaign module (EECH_CAMPAIGN_HOST_ENV); the reference host runs the same original C as its kernels
 * (eech-world/csrc/env_ref.c).
 *
 * The exchange is one synchronous host callback, each operation a single request and a single reply, all by value:
 *
 *   IMPORT       the host adopts the physical field of the session record (the session's defaults at its creation, the setters of the pack/unpack and
 *                debug paths, a test's synthetic state).  The session record is a read-only MIRROR of the host's field: it is refreshed from every
 *                reply that carries the state and never evolved by the campaign.
 *   EXPORT       the host's field, to refresh the mirror (before the session is packed).
 *   INIT_RANDOM  set_session_random_weather_mode: the TEN draws the original makes, in the original order and at the original call site, taken by the
 *                campaign from the shared random word (frand1 / rand16 / sfrand1) and handed over as values, with the configured radii and speeds
 *                and the map extents; the host applies the original's expressions.  No draw is made ahead of need and the host draws nothing.
 *   INIT_FIXED   set_session_fixed_weather_mode.
 *   WEATHER      get_session_weather_at_point (and the simple choice): the position and the season, answered with the current and target mode, the
 *                transitional state and the simple choice.
 *   WIND         get_session_wind_velocity_at_point: the position; answered with the speed and the velocity vector.
 *   GUST         the gust modulation at a phase (a test-only question: the campaign never asks it).
 *   STEP         the physical part of update_transitional_weather for one update: the delta of the frame (the previous frame's, as the original reads it),
 *                the camera position, the comms model, the map extents.  The lightning timer's expiry needs one draw of the shared word for the next
 *                timer: the first request carries none and the host answers NEED_DRAW without changing anything; the campaign then makes that one
 *                draw at the original site and asks again with it.  The reply carries the field after the step and, on an expiry, the position the
 *                original struck (the weather centre BEFORE it moved) and the new timer; the campaign runs the strikes and the sound entities
 *                (their draws, their allocation) in the original order.
 *
 * There is no campaign fallback: a hosted engine with no environment provider stops at boot with an error, and a malformed, failed or withheld answer
 * stops it with an error naming the provider.  No EECH pointer, entity index or session layout crosses the boundary.
 */
#ifndef SS_HOSTENV_H
#define SS_HOSTENV_H

#include <stddef.h>

#define CAMPAIGN_ENV_ERROR_MAX 512

/* the floats and the integers of the physical field, in the order of the arrays of campaign_env_state */
enum campaign_env_float
{
	CAMPAIGN_ENV_F_WEATHER_RADIUS = 0,
	CAMPAIGN_ENV_F_TRANSITIONAL_PERIOD,
	CAMPAIGN_ENV_F_TRANSITIONAL_STATUS,
	CAMPAIGN_ENV_F_WEATHER_POSITION_X,
	CAMPAIGN_ENV_F_WEATHER_POSITION_Y,
	CAMPAIGN_ENV_F_WEATHER_POSITION_Z,
	CAMPAIGN_ENV_F_WEATHER_VELOCITY_X,
	CAMPAIGN_ENV_F_WEATHER_VELOCITY_Y,
	CAMPAIGN_ENV_F_WEATHER_VELOCITY_Z,
	CAMPAIGN_ENV_F_WIND_EFFECT_RADIUS,
	CAMPAIGN_ENV_F_WIND_GUSTING_VALUE,
	CAMPAIGN_ENV_F_WIND_MINIMUM_SPEED,
	CAMPAIGN_ENV_F_WIND_MAXIMUM_SPEED,
	CAMPAIGN_ENV_F_WIND_DIRECTION_X,
	CAMPAIGN_ENV_F_WIND_DIRECTION_Y,
	CAMPAIGN_ENV_F_WIND_DIRECTION_Z,
	CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_X,
	CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_Y,
	CAMPAIGN_ENV_F_WIND_EFFECT_POSITION_Z,
	CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_X,
	CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_Y,
	CAMPAIGN_ENV_F_WIND_EFFECT_VELOCITY_Z,
	CAMPAIGN_ENV_F_LIGHTNING_TIMER,
	CAMPAIGN_ENV_FLOATS
};

enum campaign_env_int
{
	CAMPAIGN_ENV_N_WEATHER_MODE = 0,
	CAMPAIGN_ENV_N_TARGET_WEATHER_MODE,
	CAMPAIGN_ENV_N_LOCAL_WEATHER_MODEL,
	CAMPAIGN_ENV_N_WEATHER_INCREASING,
	CAMPAIGN_ENV_N_WIND_INCREASING,
	CAMPAIGN_ENV_INTS
};

enum campaign_env_op
{
	CAMPAIGN_ENV_IMPORT = 1,
	CAMPAIGN_ENV_EXPORT,
	CAMPAIGN_ENV_INIT_RANDOM,
	CAMPAIGN_ENV_INIT_FIXED,
	CAMPAIGN_ENV_WEATHER,
	CAMPAIGN_ENV_WIND,
	CAMPAIGN_ENV_GUST,
	CAMPAIGN_ENV_STEP
};

#define CAMPAIGN_ENV_DRAWS 10

typedef struct campaign_env_state
{
	float f[CAMPAIGN_ENV_FLOATS];
	int n[CAMPAIGN_ENV_INTS];
} campaign_env_state;

typedef struct campaign_env_request
{
	int op;
	/* every operation: the global season the campaign was loaded in is winter (read where the original read it, at each question) */
	int winter;
	/* IMPORT */
	campaign_env_state state;
	/* INIT_RANDOM: the ten draws in the original's order (frand1, rand16, frand1, frand1, sfrand1, then the same five for the wind; the rand16 draws as their integer value), the
	   configured radii and speeds (rain minimum, maximum, speed; wind minimum, maximum, speed) and the map extents (minimum x, maximum x, minimum z, maximum z, middle x, middle z) */
	double draw[CAMPAIGN_ENV_DRAWS];
	float init_config[6];
	float map[6];
	/* INIT_FIXED */
	int fixed_mode;
	/* WEATHER, WIND (the position; vector: the velocity vector is wanted) */
	float x, z;
	int want_vector;
	/* GUST */
	float phase;
	/* STEP: the delta, the comms model (server), whether the player's cockpit is the camera and the camera position when there is one, and the one draw of the lightning timer */
	float delta;
	int server;
	int in_cockpit;
	int have_camera;
	float camera[3];
	int have_draw;
	double lightning_draw;
} campaign_env_request;

typedef struct campaign_env_reply
{
	/* EXPORT, INIT_RANDOM, INIT_FIXED, STEP: the field after the operation */
	campaign_env_state state;
	/* WEATHER */
	int current_mode, target_mode, simple_mode;
	float transitional_state;
	/* WIND: the speed and the velocity vector */
	float speed;
	float vector[3];
	/* GUST */
	float modulation;
	/* STEP: the host needs the one draw (nothing was changed; new_timer then carries the timer after the decrement, for the record); or the lightning timer expired: the position the original struck and the next timer */
	int need_draw;
	int expired;
	float strike[3];
	float new_timer;
} campaign_env_reply;

typedef int (*campaign_env_fn)
(
	const campaign_env_request *request,
	campaign_env_reply *reply,
	char *error,
	size_t error_size,
	void *user
);

typedef struct campaign_env_host
{
	campaign_env_fn environment;
	void *user;
} campaign_env_host;

struct SESSION;

/* Installs the host for the rest of the process (like the terrain provider it is kept across engine entries: the field is asked about from every entry).  A NULL host or
   one with no callback leaves the installed one. */
void campaign_env_set_host (const campaign_env_host *host);

/* The session record's physical fields are a read-only mirror of the host's field.  IMPORT pushes the mirror to the host (the session's defaults when it is created, and the
   record's setters afterwards); REFRESH makes the mirror the host's current field (EXPORT). */
void campaign_env_import (const struct SESSION *raw);
void campaign_env_refresh (struct SESSION *raw);

/* the ten draws are taken by the caller (session.c), in the original order and at the original site */
void campaign_env_init_random (struct SESSION *raw, const double *draw);
void campaign_env_init_fixed (struct SESSION *raw, int mode);
void campaign_env_weather_at (float x, float z, int *current_mode, int *target_mode, float *transitional_state, int *simple_mode);
float campaign_env_wind_at (float x, float z, float *vector);
float campaign_env_gust_modulation (float phase);

/* The physical update of one session update.  Returns nonzero when the lightning timer expired, with the position the original struck and the new timer; the mirror holds the field
   after the step.  camera is NULL when the campaign has no camera position to give (the host refuses only if it needs one). */
int campaign_env_step (struct SESSION *raw, int in_cockpit, const float *camera, float *strike, float *new_timer);

#endif
