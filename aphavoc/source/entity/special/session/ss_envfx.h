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
// Without EECH_ENV_OBSERVER every macro below expands to nothing and no hook argument is evaluated, and the kind and counter enumerations are not declared, so the executable bodies of
// the sources that carry the hooks have the original tokens. Their preprocessed text is NOT literally identical to the original: the added hook lines shift __LINE__ (and the
// diagnostics, such as ASSERT messages, that print it) and the hook macros are defined; the feature-off check classifies exactly those and compares the body tokens, it does not claim whole-file equality.
// With it (cargo feature `env-observer`, never shipping) each hook only READS values the original has already computed or globals it already holds (the shared seed word
// `random_number_seed`, the entity heap's next free index, the headless clock) and appends them as integers or IEEE bit patterns to a bounded buffer in
// csrc/eech_env_observer.c. No hook draws a random number, calls the engine, allocates an entity, writes a game variable or changes the order of anything the original does.
// The observer records nothing until it is armed (before boot, by the test-only module function), and a full buffer drops events and says so.
//
// The word recorded with every event (`rng`) is the shared seed word AT that point: a change between two events with no draw hook between them is a draw (or a reseed) by code
// that has no hook, and the report says so by what it shows, never by inference from a later snapshot. For ENV_FX_SEED_SET the hook runs BEFORE the original assigns the word, so `rng`
// (and w0) is the word as it stands and `a` is the seed about to be set; the next event shows the assigned word.
//

#ifndef SS_ENVFX_H
#define SS_ENVFX_H

#ifdef EECH_ENV_OBSERVER

enum ENV_FX_KINDS
{
	ENV_FX_BOOT_BEGIN = 1,			// eech_engine.c boot: a: the boot seed, w0: the headless clock (ms)
	ENV_FX_BOOT_END,				// eech_engine.c boot returned: w0: the headless clock (ms)
	ENV_FX_SEED_SET,				// random.c set_random_number_seed, BEFORE it assigns: a: the seed argument (the target), rng and w0: the shared word as it stands (the libc generator is then reseeded with the same argument)
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

//
// THE LIGHTNING QUEUE PROBE (ENV_FXQ). A SEPARATE, finite record store (csrc/eech_env_observer.c) that is NOT part of the event stream above: the stream, its kinds, its sequence numbers and the
// transcript grammar that checks it are untouched, and the probe adds no event to it. Each probe point READS the active 3D environment, the main 3D environment and the lightning strike queue
// (modules/3d/3dstrike.c, through a read-only accessor that exists only in this build) and stores integers and IEEE bit patterns. It draws nothing, allocates nothing, formats nothing and writes no
// game variable. The point names the place in the original lightning routine: ENTRY (before the active environment is read), BEFORE/AFTER_n (around the n-th add_3d_lightning_strike), RESTORED (after
// the original's own environment restore, before the sound entities). The same six points are in BOTH compiled entry points of the lightning routine, which compile exclusively: the hosted
// create_lightning_effects (EECH_CAMPAIGN_HOST_ENV, the active path of the current baseline) and the legacy update_lightning_effect (the #else build).
//

enum ENV_FXQ_POINTS
{
	ENV_FXQ_ENTRY = 0,
	ENV_FXQ_BEFORE_1,
	ENV_FXQ_AFTER_1,
	ENV_FXQ_BEFORE_2,
	ENV_FXQ_AFTER_2,
	ENV_FXQ_RESTORED,
	ENV_FXQ_POINT_COUNT				// (not a point)
};

extern void eech_envobs_queue (int point, const void *main_env);

#define ENV_FXQ(point) eech_envobs_queue (point, main_3d_env);

extern int random_number_seed;

extern void eech_envobs_event (int kind, int a, int b, unsigned int w0, unsigned int w1, unsigned int w2, unsigned int w3, unsigned int w4, unsigned int w5, int rng);
extern unsigned int eech_envobs_bits (float value);
extern void eech_envobs_count (int counter);
extern void eech_envobs_sound_facts (int in_cockpit, float x, float y, float z, float alt, float light_wind, float heavy_wind, float sector_x, float sector_z, int rng);

#define ENV_FX(kind, a, b, w0, w1, w2, w3, w4, w5) eech_envobs_event (kind, a, b, w0, w1, w2, w3, w4, w5, random_number_seed);
#define ENV_FXF(value) eech_envobs_bits (value)
#define ENV_FXC(counter) eech_envobs_count (counter);
#define ENV_FXS(in_cockpit, x, y, z, alt, light, heavy, sx, sz) eech_envobs_sound_facts (in_cockpit, x, y, z, alt, light, heavy, sx, sz, random_number_seed);

//
// THE SOUND ENTITY PROBE (ENV_FXE). Another SEPARATE, finite record store (csrc/eech_env_observer.c), not the event stream: it adds no event and changes none. A point READS the sound effect entity the original
// code has just created or is updating (its stored fields copied as bits -- no float is loaded by value --, its list links, its playback handle as a present/absent flag, the sample table's rate and size for
// its first sample) and the process's frame counter. It draws nothing, allocates nothing, formats nothing at the point and writes no game variable. The creation point sits at the end of
// soundeff.c create_local_sound_effect_entity (the real constructor every creator of a sound effect entity passes through, the lightning routine's two included, on the hosted and the legacy path alike); the
// update points sit in se_updt.c update_server and update_client. The observer records a creation only while a lightning sound creation is open (ENV_FX_SOUND_BEGIN to ENV_FX_SOUND_END) or a fixture
// control has switched tracking on, and then follows that entity through its updates.
//

enum ENV_FXE_POINTS
{
	ENV_FXE_CREATED = 0,			// create_local_sound_effect_entity: the entity as created, linked and given its lifetime
	ENV_FXE_SERVER_ENTER,			// se_updt.c update_server: before anything is changed
	ENV_FXE_SERVER_EXPIRED_NO_HANDLE,	// ... the lifetime is below zero and there is no playback handle: the entity is killed next
	ENV_FXE_SERVER_EXPIRED_IDLE,		// ... below zero, a handle exists and is not playing: killed next
	ENV_FXE_SERVER_EXPIRED_PLAYING,		// ... below zero, a handle exists and is still playing: not killed
	ENV_FXE_SERVER_KILLED,			// ... straight after kill_client_server_entity (only the entity slot is read: the sound data is gone)
	ENV_FXE_CLIENT_ENTER,			// se_updt.c update_client: before anything is changed
	ENV_FXE_CLIENT_EXIT,			// ... at the end (the client never kills here)
	ENV_FXE_POINT_COUNT			// (not a point)
};

extern void eech_envobs_sound (int point, const void *en);

#define ENV_FXE(point, en) eech_envobs_sound (point, (const void *) (en));

//
// THE SOUND INITIALISATION PROBE (ENV_FXA). A third SEPARATE, finite store (csrc/eech_env_observer.c): it is NOT the event stream and NOT the ENV_FXE record store, it adds no event and no record to either, and it is
// NEVER reset by arming (the application sound system is initialised during boot, once, and the observer may be armed before it or not at all: the store counts from process start and reports `boots`). A point READS
// integers the original code already holds (the command line flags, the backend result, the counts of the sound database header, a sample index and its descriptor's rate, size and original rate) and stores them in
// fixed tables; it calls nothing, allocates nothing, draws nothing, opens nothing, formats nothing and writes no game variable. The descriptor catalogue digest is made from the table
// `application_sound_effects` with integer operations only. Points sit in appsound/snd_init.c initialise_application_sound_system, appsound/snd_data.c initialise_application_sound_effects,
// open_application_sound_file_system and load_side_dependant_application_sound_samples, and in soundeff.c set_local_sound_effect_sample_indices (the hosted block that reads the descriptors for the host's duration question).
// The attempts to open the sound header and data files and the calls of the null DirectSound entry points are counted where they happen, in csrc/eech_win32.c and csrc/eech_com_null.c (the headless platform layer),
// so no extra attempt is made to learn them. The report is the explicit fixture case group 14 index 15 (csrc/eech_env_fixture.c env_snd_init_case).
//

enum ENV_FXA_POINTS
{
	ENV_FXA_INIT_ENTER = 0,		// snd_init.c initialise_application_sound_system: first statement (the flags and the backend state as they stand)
	ENV_FXA_EFFECTS_BEGIN,		// snd_data.c initialise_application_sound_effects: first statement
	ENV_FXA_DESCRIPTORS_FILLED,	// ... the compiled-in descriptors are in the table (the digest is taken here), before the header is read
	ENV_FXA_OPEN_TRUE,		// snd_data.c open_application_sound_file_system returns TRUE (a: the header's sample count)
	ENV_FXA_OPEN_FALSE,		// ... returns FALSE
	ENV_FXA_HEADER_MATCH,		// ... a descriptor found its entry in the header (its size is set)
	ENV_FXA_HEADER_MISS,		// ... a descriptor found no entry
	ENV_FXA_EFFECTS_END,		// initialise_application_sound_effects: last statement (the digest is taken again)
	ENV_FXA_BACKEND,		// snd_init.c: initialise_sound_system returned and its result was stored
	ENV_FXA_LOAD,			// snd_init.c: load_application_sound_samples is about to be called
	ENV_FXA_SKIPPED,		// snd_init.c: the sound system is not initialised (no-sound or dedicated flag)
	ENV_FXA_INIT_EXIT,		// initialise_application_sound_system: last statement
	ENV_FXA_LOAD_SIDE,		// snd_data.c load_side_dependant_application_sound_samples: first statement (a: the side)
	ENV_FXA_CTOR,			// soundeff.c set_local_sound_effect_sample_indices, hosted block: before the descriptors are read (a: the sample count, b: the looping flag)
	ENV_FXA_CTOR_SAMPLE,		// ... one descriptor read (a: the sample index, b: rate, c: size, d: original rate)
	ENV_FXA_POINT_COUNT		// (not a point)
};

extern void eech_envobs_sndinit (int point, int a, int b, int c, int d);

#define ENV_FXA(point, a, b, c, d) eech_envobs_sndinit (point, a, b, c, d);

#else

#define ENV_FX(kind, a, b, w0, w1, w2, w3, w4, w5)
#define ENV_FXF(value) 0
#define ENV_FXC(counter)
#define ENV_FXS(in_cockpit, x, y, z, alt, light, heavy, sx, sz)
#define ENV_FXQ(point)
#define ENV_FXE(point, en)
#define ENV_FXA(point, a, b, c, d)

#endif

#endif
