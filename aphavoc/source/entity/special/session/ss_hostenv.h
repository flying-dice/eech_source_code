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
 *   AMBIENT      the physical RAIN and WIND levels of the session's ambient sound children (ss_updt.c update_session_sound_effects, EEDCS-51): the point the sound is heard at (the camera,
 *                or the gunship in the cockpit: x, z), the campaign's observations of it (its height, its radar altitude, the cloud base height) and whether the cockpit is the point.  The
 *                answer is the four levels by value (rain light, rain heavy, wind light, wind heavy), computed by the host from its own weather and wind with the original's expressions
 *                (equal outdoor rain modes alias one entry and are attenuated twice, as in the original).  The campaign writes them into its children and computes nothing.  The terrain
 *                children's levels are the TERRAIN provider's question (mb_hostterrain.h AMBIENT).
 *   LIGHTNING_ADD / LIGHTNING_UPDATE
 *                the PHYSICAL QUEUE of lightning strikes (modules/3d/3dstrike.c, EEDCS-51 D3): the host owns the 32 slots, the first-free choice, the drop of a strike when every slot is in use, the
 *                time stamp, the aging and the expiry.  ADD hands over what the original add_3d_lightning_strike was given and read (the type, the length, the position, whether an active
 *                environment exists and, when it does, its time of day: the campaign still switches the 3D environment as the original did and still decides WHEN a strike is added); the host answers
 *                the slot it filled (-1: none, with no active environment or with every slot in use) and the whole queue.  UPDATE hands over the time of day update_3d_lightning_strikes read; the host ages
 *                every strike in use and frees the finished ones, and answers the whole queue.  A forked strike's frame number needs the frame count of the strike animation, which only the
 *                campaign's texture table holds: the host answers NEED_FRAMES (nothing changed) and the campaign asks again with the count, read where the original read it and only because a forked
 *                strike was to be aged.  The campaign keeps a read-only MIRROR of the queue (the renderer's draw routines and the observer read it), refreshed from every answer and never evolved by the
 *                campaign; the answer is checked against the mirror and the request before anything is written.  No EECH pointer, slot layout or entity index crosses the boundary.
 *   SOUND_LIFETIME_INIT / SOUND_LIFETIME_UPDATE
 *                the PHYSICAL LIFETIME of a sound effect (soundeff.c and se_updt.c, EEDCS-51 D3): INIT answers the lifetime of a new effect (the sum of the samples' durations, the 40 s cap or the 300 s music cap, 0 for a
 *                looping effect) from the sequence's sample descriptors handed over by value; UPDATE answers, for one call of the server or the client routine, the aged lifetime and valid lifetime, whether a valid effect's
 *                lifetime went below zero, and the decision (none; kill the entity; unlink it from the update list and release its handle).  The stored fields stay in the effect's record, which the pause and
 *                resume functions, the network pack and unpack and the observer read and write, so the request hands them over by value and the campaign applies the answer; the sound database (the descriptors), the
 *                playback handle and the device stay the campaign's, and the host may ask once whether a handle is still playing (NEED_PLAYING, nothing changed).  No entity pointer, id or list layout crosses.
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

/* One slot of the physical lightning queue, by value (modules/3d/3dstrike.c: the in_use flag, the type, the position, the start time and the length, the current frame) */
#define CAMPAIGN_LIGHTNING_SLOTS 32

typedef struct campaign_lightning_strike
{
	int in_use;
	int type;
	float x, y, z;
	float time_started;
	float time_length;
	int current_frame_number;
} campaign_lightning_strike;


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
	CAMPAIGN_ENV_STEP,
	CAMPAIGN_ENV_AMBIENT,
	CAMPAIGN_ENV_LIGHTNING_ADD,
	CAMPAIGN_ENV_LIGHTNING_UPDATE,
	CAMPAIGN_ENV_SOUND_LIFETIME_INIT,
	CAMPAIGN_ENV_SOUND_LIFETIME_UPDATE
};

#define CAMPAIGN_ENV_DRAWS 10

/* The samples of one sound effect's sequence that cross the boundary by value (the sound manager's MAX_SEQUENCED_SAMPLES is 15; a longer sequence is refused, not truncated), and the decisions the host
   returns for a sound effect's update */
#define CAMPAIGN_SOUND_SAMPLES 16
#define CAMPAIGN_SOUND_ACTION_NONE 0
#define CAMPAIGN_SOUND_ACTION_KILL 1
#define CAMPAIGN_SOUND_ACTION_UNLINK 2

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
	/* AMBIENT (x, z and in_cockpit above): the point's height, its radar altitude and the cloud base height */
	float ambient_y, ambient_alt, ambient_cloud_base;
	/* LIGHTNING_ADD (x, z above): the strike type, its length, whether an active 3D environment exists and, if so, its time of day;
	   LIGHTNING_UPDATE: the time of day, and the strike animation's frame count when the campaign has been asked for it */
	int lightning_type;
	float lightning_length;
	int lightning_environment_active;
	float lightning_time_of_day;
	int lightning_have_frames;
	int lightning_frames;
	/* SOUND_LIFETIME_INIT (soundeff.c set_local_sound_effect_sample_indices): whether the effect loops, whether it is a MUSIC sound, and the sequence's samples by value (the sample descriptor's rate and size, as
	   the ints the sound database holds); SOUND_LIFETIME_UPDATE (se_updt.c update_server / update_client): the routine, the frame delta, the effect's validity, whether it loops, its lifetime and valid lifetime as stored, whether
	   it holds a playback handle and, only once the host has asked for it, whether that handle is still playing */
	int sound_looping;
	int sound_music;
	int sound_count;
	int sound_rate[CAMPAIGN_SOUND_SAMPLES];
	int sound_size[CAMPAIGN_SOUND_SAMPLES];
	int sound_server;
	int sound_valid;
	int sound_has_handle;
	int sound_have_playing;
	int sound_playing;
	float sound_delta;
	float sound_lifetime;
	float sound_valid_lifetime;
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
	/* AMBIENT: rain light, rain heavy, wind light, wind heavy */
	float ambient_levels[4];
	/* LIGHTNING_ADD: the slot filled (-1: none); LIGHTNING_UPDATE: the host needs the strike animation's frame count (nothing was changed);
	   both: the whole queue after the operation */
	int lightning_slot;
	int lightning_need_frames;
	campaign_lightning_strike lightning_queue[CAMPAIGN_LIGHTNING_SLOTS];
	/* SOUND_LIFETIME_INIT: the effect's lifetime; SOUND_LIFETIME_UPDATE: the lifetime and valid lifetime after the update, whether a valid effect's lifetime went below zero, the decision (none, kill the entity,
	   unlink it from the update list and release its handle) or that the host needs to know whether the handle is still playing (nothing was changed) */
	float sound_lifetime;
	float sound_valid_lifetime;
	int sound_expired;
	int sound_action;
	int sound_need_playing;
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

/* The ambient sound levels (rain light, rain heavy, wind light, wind heavy) at a point, from the host's own weather and wind.  The answer is checked in full before anything is written to the
   outputs: the four numbers must be finite and the wind levels within [0, 1] (the original's piecewise function never leaves it).  A missing provider, a refusal or a malformed answer stops
   the engine with an error that names the environment provider; there is no campaign solver to fall back to. */
void campaign_env_ambient (int in_cockpit, float x, float y, float z, float alt, float cloud_base, float *rain_light, float *rain_heavy, float *wind_light, float *wind_heavy);

/* The physical update of one session update.  Returns nonzero when the lightning timer expired, with the position the original struck and the new timer; the mirror holds the field
   after the step.  camera is NULL when the campaign has no camera position to give (the host refuses only if it needs one). */
int campaign_env_step (struct SESSION *raw, int in_cockpit, const float *camera, float *strike, float *new_timer);


/* The physical lightning queue (modules/3d/3dstrike.c hands its add and its update here).  Both write the campaign's mirror of the queue (CAMPAIGN_LIGHTNING_SLOTS slots) from the host's answer,
   after checking the whole answer against the request and the mirror as it stood; a missing provider, a refusal or a malformed answer stops the engine with an error that names the environment
   provider, and the mirror is then unchanged.  There is no campaign queue to fall back to.  campaign_lightning_add returns the slot filled or -1.  campaign_lightning_update asks for the frame count
   of the strike animation through frames_of (called only when the host needs it, where the original read it). */
int campaign_lightning_add (int type, float length, float x, float z, int environment_active, float time_of_day, campaign_lightning_strike *mirror);
void campaign_lightning_update (float time_of_day, int (*frames_of) (void), campaign_lightning_strike *mirror);

/* The physical lifetime of a sound effect (soundeff.c and se_updt.c).  INIT: the sum of the samples' durations, the cap (40 s; 300 s for MUSIC) and the looping case, from the sample descriptors the campaign hands over by value.
   UPDATE: the aging of a valid effect by the frame delta, the server's expiry decision (kill when there is no handle or the handle is not playing), the client's clamp at zero, and the countdown and
   unlink of an invalid effect.  The campaign keeps the stored fields (the pause and resume functions, the network pack and unpack and the observer read and write them), hands them over by value, checks the
   answer against the request and applies it: writes both lifetimes, then kills the entity or unlinks it from the update list and releases its handle.  The sound database (the descriptors), the playback handle and the
   device stay the campaign's: playing_of is called only when the host asks whether a handle is still playing.  A missing provider, a refusal or a malformed answer stops the engine with an error that names
   the environment provider; there is no campaign arithmetic to fall back to. */
float campaign_sound_lifetime_init (int looping, int music, int count, const int *rate, const int *size);
void campaign_sound_lifetime_update (int server, float delta, int valid, int looping, float *lifetime, float *valid_lifetime, void *handle, int (*playing_of) (void *handle), int *expired, int *action);

#endif
