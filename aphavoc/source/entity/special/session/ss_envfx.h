//
// 	 Enemy Engaged RAH-66 Comanche Versus KA-52 Hokum
// 	 Copyright (C) 2000 Empire Interactive (Europe) Ltd,
// 	 677 High Road, North Finchley, London N12 0DA
//
// 	 Please see the document LICENSE.TXT for the full licence agreement
//

//
// TEST-ONLY session environment observer hooks (EEDCS-51).
//
// Without EECH_ENV_OBSERVER every macro below expands to nothing, so the sources that carry them preprocess to the original text and no hook argument is evaluated.
// With it (cargo feature `env-observer`, never shipping) each hook only READS values the original has already computed or globals it already holds (the shared seed word
// `random_number_seed`, the entity heap's next free index, the headless clock) and appends them as integers or IEEE bit patterns to a bounded buffer in
// csrc/eech_env_observer.c. No hook draws a random number, calls the engine, allocates an entity, writes a game variable or changes the order of anything the original does.
// The observer records nothing until it is armed (before boot, by the test-only module function), and a full buffer drops events and says so.
//
// The word recorded with every event is the shared seed word AT that point: a change between two events with no draw hook between them is a draw (or a reseed) by code
// that has no hook, and the report says so by what it shows, never by inference from a later snapshot.
//

#ifndef SS_ENVFX_H
#define SS_ENVFX_H

enum ENV_FX_KINDS
{
	ENV_FX_BOOT_BEGIN = 1,			// eech_engine.c boot: a: the boot seed, w0: the headless clock (ms)
	ENV_FX_BOOT_END,				// eech_engine.c boot returned: w0: the headless clock (ms)
	ENV_FX_SEED_SET,				// random.c set_random_number_seed: a: the seed argument, w0: the shared word before (the libc generator is reseeded with the same argument)
	ENV_FX_SEED_SOURCE,				// random.c initialise_random_number_generator: a: get_system_time () as returned, before the zero test
	ENV_FX_START_RESEED,			// session.c get_session_random_start_time_of_day: initialise_random_number_generator returned
	ENV_FX_START_INDEX,				// ... the table index drawn (a)
	ENV_FX_START_TIME,				// ... the start time after the minute jitter (w0), before the day wrap
	ENV_FX_START_RESULT,			// ... the start time returned (w0)
	ENV_FX_SESSION_CREATED,			// ss_creat.c create_local: defaults set (a: local weather model, b: day segment; w0: start time, w1: lightning timer, w2: weather radius, w3: wind radius, w4: acceleration)
	ENV_FX_PARSER_SESSION_BEGIN,	// parsgen.c: the campaign file is about to create the session (nothing exists yet)
	ENV_FX_PARSER_SESSION_END,		// parsgen.c: set_session_random_weather_mode returned
	ENV_FX_RAIN_TAG,				// parsgen.c FILE_TAG_WEATHER_RAIN (a: the value read)
	ENV_FX_WIND_TAG,				// parsgen.c FILE_TAG_WEATHER_WIND (w0: the value read, only logged by the original)
	ENV_FX_WEATHER_BEGIN,			// session.c set_session_random_weather_mode begins
	ENV_FX_WEATHER_DRAW,			// ... after draw a of ten (w0: the field or local the draw produced, as bits; b: the flag for the two flag draws)
	ENV_FX_WEATHER_END,				// ... returned
	ENV_FX_FIXED_WEATHER,			// session.c set_session_fixed_weather_mode (a: the mode)
	ENV_FX_FRAME_BEGIN,				// eech_engine.c frame (a: the host's milliseconds, b: the time acceleration count, w0: the headless clock, w1: system_delta_time)
	ENV_FX_FRAME_END,				// ... after set_delta_time (w0: the clock, w1: system_delta_time, w2: system_one_over_delta_time)
	ENV_FX_TOD_BEGIN,				// ss_updt.c update_time_of_day begins (w0: elapsed time, w1: get_delta_time (), w2: the acceleration)
	ENV_FX_TOD_END,					// ... ends (a: skip night time, b: day segment, w0: elapsed time, w1: time of day)
	ENV_FX_LIGHTNING_EXPIRED,		// ss_updt.c update_lightning_effect: the timer reached zero (a: server, w0: the timer after the decrement, w1: the delta)
	ENV_FX_LIGHTNING_NEW_TIMER,		// ... the next timer drawn (w0)
	ENV_FX_LIGHTNING_TIMER_SET,		// ... after set_client_server_entity_float_value (w0: raw->lightning_timer)
	ENV_FX_LIGHTNING_ENV,			// ... a: whether the active 3D environment was the main one
	ENV_FX_STRIKE_BEGIN,			// ... before add_3d_lightning_strike a (w0, w1: the strike position)
	ENV_FX_STRIKE_END,				// ... after it
	ENV_FX_SOUND_BEGIN,				// ... before create_client_server_sound_effect_entity a (1 lightning, 2 thunder); w0, w1: the next free entity index and the next free local entity index
	ENV_FX_SOUND_END,				// ... after it (w0, w1: the next free indices now)
	ENV_FX_SOUND_FACTS,				// ss_updt.c update_session_sound_effects: recorded when the terrain scratch or the camera changes (see csrc/eech_env_observer.c)
	ENV_FX_COUNT					// (not an event kind)
};

enum ENV_FX_COUNTERS
{
	ENV_FX_C_UPDATE_SERVER = 0,		// ss_updt.c update_server calls
	ENV_FX_C_LIGHTNING_CALLS,		// update_lightning_effect calls
	ENV_FX_C_SOUND_CALLS,			// update_session_sound_effects calls
	ENV_FX_C_RESYNC_CALLS,			// update_time_of_day_resync calls
	ENV_FX_C_WEATHER_CALLS,			// update_transitional_weather calls
	ENV_FX_C_COUNT
};

#ifdef EECH_ENV_OBSERVER

extern int random_number_seed;

extern void eech_envobs_event (int kind, int a, int b, unsigned int w0, unsigned int w1, unsigned int w2, unsigned int w3, unsigned int w4, unsigned int w5, int rng);
extern unsigned int eech_envobs_bits (float value);
extern void eech_envobs_count (int counter);
extern void eech_envobs_sound_facts (int in_cockpit, float x, float y, float z, float alt, float light_wind, float heavy_wind, float sector_x, float sector_z, int rng);

#define ENV_FX(kind, a, b, w0, w1, w2, w3, w4, w5) eech_envobs_event (kind, a, b, w0, w1, w2, w3, w4, w5, random_number_seed);
#define ENV_FXF(value) eech_envobs_bits (value)
#define ENV_FXC(counter) eech_envobs_count (counter);
#define ENV_FXS(in_cockpit, x, y, z, alt, light, heavy, sx, sz) eech_envobs_sound_facts (in_cockpit, x, y, z, alt, light, heavy, sx, sz, random_number_seed);

#else

#define ENV_FX(kind, a, b, w0, w1, w2, w3, w4, w5)
#define ENV_FXF(value) 0
#define ENV_FXC(counter)
#define ENV_FXS(in_cockpit, x, y, z, alt, light, heavy, sx, sz)

#endif

#endif
