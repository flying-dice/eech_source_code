/*
 * TEST-ONLY characterisation of the ORIGINAL session environment (EEDCS-51, first increment of D3). Compiled only with the `env-fixture` feature; never part of the shipping module.
 * SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED: nothing in this file has been compiled or executed. It moves no physics and adds no campaign behaviour.
 *
 * HOST-OWNED FIELD (EECH_CAMPAIGN_HOST_ENV, EEDCS-51 production stage): when the module is built with the host-owned weather and wind field (ss_hostenv.h), the routines named below are the campaign's
 * synchronous requests to the HOST, which runs the original arithmetic as its kernels, and the session record's physical fields are a read-only mirror of the host's field. The witnesses are unchanged
 * in what they observe and print; they now reach the host's field through the campaign's own import (a witness pushes its synthetic input, a restore pushes the saved record back) and read it back
 * through the campaign's own export (a snapshot or fingerprint re-reads the host first), so every number a case prints is still the original arithmetic's, now computed by the host. Against the original
 * session code (no host field) the push and the re-read are no-ops and this file is the retained characterisation of the original.
 *
 * What it observes is the session entity's environment: the clock (start time, elapsed time, time of day, day, day segment), the weather (the local ring model or the global/fixed
 * model, the simple half-transition choice) and the wind (the zone around a drifting centre, the gusts, the direction). Every number a case prints comes from the ORIGINAL routines
 * (session.c, ss_updt.c, ss_int.c, ss_float.c, range.c, vector.c); this file only chooses inputs, calls them and prints the words they return. It never recomputes a result.
 *
 * Two kinds of line, never mixed:
 *   kind=retail   the unmodified session as the booted campaign holds it (op 1, read-only). The queries in it are the original's, asked at fixed points of the live state.
 *   kind=witness  a CONTROLLED synthetic case: the session fields the case names are set, the original routine is called, its answer is printed, and EVERY state the case touched
 *                 is restored before return. The case ends with `env-witness-end restored=0|1 fp_before fp_after`, the fingerprint of the whole rendered environment state
 *                 (session fields, season, the shared random seed, the frame delta, the camera position) before and after: restored=0 would be a defect of this fixture, never a result.
 *
 * Restored on every path that returns: the whole session record (memcpy), the map season, the shared generator's seed word (random_number_seed; the libc generator behind
 * set_random_number_seed is never touched), and the frame delta. A path that leaves through the engine's fatal handler (longjmp) cannot restore anything and poisons the engine; every
 * case is therefore domain-guarded so that no original ASSERT can fire (the wind gust must be in [0,1): the original asserts outside it, and the cases that would go there are
 * REFUSED and printed as refused, never called).
 *
 * Float values are printed as their IEEE-754 words (%08x), so a signed zero, a neighbouring float or a rounding-mode effect is visible; decimal text is never the comparison key.
 * The calls run inside the engine's entry discipline (round-toward-zero), exactly as a frame does; the case inputs derived here (ring thresholds and their neighbours) are computed in
 * that same mode and printed as words, so a later run is compared on the identical input words.
 *
 * GROUP 10 (explicit only, never in the default run, the census or the 164-case oracle): the ORIGINAL terrain-sector surface enumeration get_terrain_3d_types_in_sector, which the sound routine
 * calls every frame and which writes the shared scratch terrain_types_in_sector. It reports the original's return value (1 + the number of surface transitions, NOT the distinct flags), the complete
 * scratch flags, the sector the original selected, the reset between calls and the interleave behaviour, over a finite sweep of real sector centres and boundary samples, and restores the scratch. A
 * sample the original cannot be asked is rejected with its reason. See the comment before env_sector_case. AFTER THE HOST CUTOVER OF THIS QUERY (EEDCS-51; build define EECH_CAMPAIGN_HOST_TERRAIN) the
 * function this group calls is the campaign's client of the HOST's sector enumeration (terrdata.c delegates to mb_hostterrain.c), so the same group, command and rows are the like-for-like comparison of the
 * host's answers with the three-root original collection; the sample domain below still reads the campaign's own loaded sectors, only to reject what the original could not be asked.
 *
 * NOT witnessed here, and why (printed in the census as `env-unsupported`):
 *   get_session_random_start_time_of_day  reseeds the shared AND the libc generators from the clock; the libc state cannot be saved or restored from here.
 *   update_lightning_effect               creates strike and sound entities and draws the shared generator; the controlled update cases keep lightning_timer far from zero so it
 *                                         never fires, and its natural firing is seen only through the real frames' `lightning_timer` and `rng` in the retail snapshots.
 *   update_session_sound_effects          runs in every headless frame (a camera entity exists) and only sets sound amplitudes; it also writes the shared terrain_types_in_sector. (The census line
 *                                         above is the default oracle and stays as it is; GROUP 11, explicit only, now characterises this routine: see env_sound_case.)
 *   update_time_of_day_resync             runs in every headless server frame (comms transmit every minute); observed only as the `time_of_day_resync` counter.
 *
 * GROUP 11 (explicit only, never in the default run, the census or the 164-case oracle): the ORIGINAL ambient sound routine update_session_sound_effects (ss_updt.c, unchanged) on the real session,
 * camera, terrain and the ten real ambient sound-effect children; per case it sets the controlled inputs, calls the routine, prints every child's amplification as IEEE words and restores
 * everything. See the comment before env_sound_case. The only change to the default behaviour of this file is the group range check (the last group is now 12).
 *
 * GROUP 12 (explicit only, PROCESS-ISOLATED like group 9): a WIDE session child list -- additional terrain ambient sound children created with the real constructor, the real routine called on them. See
 * the comment before env_wide_case.
 */

#ifdef EECH_ENV_FIXTURE

#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

/* exposed under this feature only, by the thin wrappers appended to session.c and ss_updt.c (the originals stay static there) */
extern float eech_env_fixture_gust_modulation (float wind_gusting_value);
extern void eech_env_fixture_update_time_of_day (entity *en);
/* ss_updt.c, non-static there */
extern void update_transitional_weather (entity *en);

enum
{
	ENV_GROUP_CENSUS = 0,
	ENV_GROUP_SNAPSHOT = 1,
	ENV_GROUP_WEATHER = 2,
	ENV_GROUP_WIND = 3,
	ENV_GROUP_GUST = 4,
	ENV_GROUP_TIME = 5,
	ENV_GROUP_INIT = 6,
	ENV_GROUP_UPDATE_TIME = 7,
	ENV_GROUP_UPDATE_WEATHER = 8,
	ENV_GROUP_CONTROLLED = 9,
	ENV_GROUP_SECTOR = 10,
	ENV_GROUP_SOUND = 11,
	ENV_GROUP_WIDE = 12,
	ENV_GROUP_LIGHTNING = 13,
	ENV_GROUP_LAST = 13
};

#define ENV_WEATHER_MAX 96
#define ENV_WIND_MAX 48
#define ENV_GUST_MAX 48
#define ENV_TIME_MAX 48
#define ENV_INIT_COUNT 7
#define ENV_UPDATE_TIME_COUNT 10
#define ENV_UPDATE_WEATHER_COUNT 16
#define ENV_CONTROLLED_COUNT 2
#define ENV_SCRATCH 12288

#define ENV_FNV_OFFSET 14695981039346656037ull
#define ENV_FNV_PRIME 1099511628211ull

#define ENV_WORD(value) ((unsigned) float_bits ((float) (value)))
#define ENV_EMIT(...) do { if (append (text, capacity, used, __VA_ARGS__) != 0) { return -1; } } while (0)

static uint32_t float_bits (float value)
{
	uint32_t bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

static int append (char *text, int capacity, int *used, const char *format, ...) __attribute__ ((format (printf, 4, 5)));

static int append (char *text, int capacity, int *used, const char *format, ...)
{
	va_list args;
	int n;

	va_start (args, format);
	n = vsnprintf (text + *used, (size_t) (capacity - *used), format, args);
	va_end (args);

	if (n < 0 || n >= capacity - *used)
	{
		return -1;
	}

	*used += n;

	return 0;
}

static uint64_t env_hash_bytes (const char *text, int length)
{
	uint64_t h = ENV_FNV_OFFSET;
	int i;

	for (i = 0; i < length; i++)
	{
		h ^= (uint64_t) (unsigned char) text[i];
		h *= ENV_FNV_PRIME;
	}

	return h;
}

static session *env_session_raw (void)
{
	return (session *) get_local_entity_data (get_session_entity ());
}

/* With the host-owned field (EECH_CAMPAIGN_HOST_ENV, ss_hostenv.h) the session record's physical fields are a read-only MIRROR of the host's field.  A witness that sets synthetic physical state
   therefore PUSHES it to the host through the campaign's own import, so that the queries and the update that follow run against it; a restore pushes the saved record back; and a state read
   (a snapshot, a fingerprint) first re-reads the host's field, so what is rendered and compared is the host's, not a stale copy.  Against the original session code (no host field) all three are
   no-ops: the record IS the field. */
#ifdef EECH_CAMPAIGN_HOST_ENV
#include "entity/special/session/ss_hostenv.h"

static void env_push (void)
{
	campaign_env_import ((struct SESSION *) env_session_raw ());
}

static void env_pull (void)
{
	campaign_env_refresh ((struct SESSION *) env_session_raw ());
}
#else
static void env_push (void)
{
}

static void env_pull (void)
{
}
#endif

/* the camera's position, or NULL when the headless build holds no camera entity (it does hold one at present: create_local of the session asserts it) */
static vec3d *env_camera_position (void)
{
	entity *camera = get_camera_entity ();

	return camera ? get_local_entity_vec3d_ptr (camera, VEC3D_TYPE_POSITION) : NULL;
}

/* the original asserts a gust outside [0,1); NaN fails both comparisons */
static int env_gust_in_domain (float gust)
{
	return (gust >= 0.0f) && (gust < 1.0f);
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the rendered environment state: its text is both the report and the input of the fingerprint */

static int env_append_vec (char *text, int capacity, int *used, const char *name, const vec3d *v)
{
	ENV_EMIT (" %s_x_w=%08x %s_y_w=%08x %s_z_w=%08x", name, ENV_WORD (v->x), name, ENV_WORD (v->y), name, ENV_WORD (v->z));

	return 0;
}

static int env_render_state (char *text, int capacity, int *used, const char *event, const char *tag)
{
	session *raw = env_session_raw ();
	vec3d *cp = env_camera_position ();
	int day = 0;
	float tod = calculate_session_time_of_day (get_session_entity (), &day);

	ENV_EMIT ("%s-time tag=%s start_time_w=%08x elapsed_time_w=%08x time_of_day_w=%08x day=%d day_segment_type=%d time_of_day_acceleration_w=%08x skip_night_time=%d time_of_day_resync_w=%08x\n",
		event, tag, ENV_WORD (raw->start_time), ENV_WORD (raw->elapsed_time), ENV_WORD (tod), day + 1, (int) raw->day_segment_type, ENV_WORD (raw->time_of_day_acceleration), (int) raw->skip_night_time, ENV_WORD (raw->time_of_day_resync));
	ENV_EMIT ("%s-weather tag=%s local_weather_model=%d weather_increasing=%d weather_radius_w=%08x weather_mode=%d target_weather_mode=%d transitional_status_w=%08x transitional_period_w=%08x lightning_timer_w=%08x",
		event, tag, (int) raw->local_weather_model, (int) raw->weather_increasing, ENV_WORD (raw->weather_radius), (int) raw->weather_mode, (int) raw->target_weather_mode,
		ENV_WORD (raw->weather_mode_transitional_status), ENV_WORD (raw->weather_mode_transitional_period), ENV_WORD (raw->lightning_timer));
	if (env_append_vec (text, capacity, used, "weather_pos", &raw->weather_position) || env_append_vec (text, capacity, used, "weather_vel", &raw->weather_velocity))
	{
		return -1;
	}
	ENV_EMIT ("\n");
	ENV_EMIT ("%s-wind tag=%s wind_increasing=%d wind_effect_radius_w=%08x gusting_value_w=%08x minimum_speed_w=%08x maximum_speed_w=%08x", event, tag, (int) raw->wind_increasing,
		ENV_WORD (raw->wind_effect_radius), ENV_WORD (raw->wind_gusting_value), ENV_WORD (raw->wind_minimum_speed), ENV_WORD (raw->wind_maximum_speed));
	if (env_append_vec (text, capacity, used, "direction", &raw->wind_direction_vector) || env_append_vec (text, capacity, used, "effect_pos", &raw->wind_effect_position)
		|| env_append_vec (text, capacity, used, "effect_vel", &raw->wind_effect_velocity))
	{
		return -1;
	}
	ENV_EMIT ("\n");
	ENV_EMIT ("%s-globals tag=%s season=%d rng=%d rng_w=%08x delta_time_w=%08x comms_model=%d in_cockpit=%d camera=%d", event, tag, (int) get_global_season (), random_number_seed, (unsigned) random_number_seed,
		ENV_WORD (get_delta_time ()), (int) get_comms_model (), (int) in_cockpit, cp ? 1 : 0);
	if (cp && env_append_vec (text, capacity, used, "camera", cp))
	{
		return -1;
	}
	ENV_EMIT ("\n");

	return 0;
}

/* the fingerprint of the whole rendered state: what a witness must leave exactly as it found it */
static uint64_t env_state_fingerprint (void)
{
	char scratch[ENV_SCRATCH];
	int used = 0;

	scratch[0] = '\0';

	env_pull ();

	if (env_render_state (scratch, ENV_SCRATCH, &used, "fp", "fp") != 0)
	{
		return 0;
	}

	return env_hash_bytes (scratch, used);
}

/* one original query at one point: the weather (all three outputs and the simple choice) and the wind (speed and vector), asked of the CURRENT session state */
static int env_render_queries (char *text, int capacity, int *used, const char *event, const char *tag, const char *label, vec3d *pos)
{
	session *raw = env_session_raw ();
	weathermodes cwm, twm, simple;
	float ts;

	get_session_weather_at_point (pos, &cwm, &twm, &ts);
	simple = get_simple_session_weather_at_point (pos);

	ENV_EMIT ("%s tag=%s point=%s x_w=%08x y_w=%08x z_w=%08x cwm=%d twm=%d ts_w=%08x simple=%d", event, tag, label, ENV_WORD (pos->x), ENV_WORD (pos->y), ENV_WORD (pos->z), (int) cwm, (int) twm, ENV_WORD (ts), (int) simple);

	if (env_gust_in_domain (raw->wind_gusting_value))
	{
		vec3d wind;
		float speed = get_session_wind_velocity_at_point (pos, &wind);

		ENV_EMIT (" wind_speed_w=%08x wind_x_w=%08x wind_y_w=%08x wind_z_w=%08x\n", ENV_WORD (speed), ENV_WORD (wind.x), ENV_WORD (wind.y), ENV_WORD (wind.z));
	}
	else
	{
		ENV_EMIT (" wind=unavailable_gust_outside_0_1\n");
	}

	return 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the finite case tables (built once, deterministically, under the engine's rounding mode) */

struct env_weather_case
{
	int local, season, cwm, twm;
	float radius, px, pz, ts;
};

struct env_wind_case
{
	float radius, wpx, wpz, minimum, maximum, gust, dx, dz, px, pz;
};

struct env_time_case
{
	int kind; /* 0 calculate (a = start, b = elapsed), 1 day segment of a, 2 the constants */
	float a, b;
};

static struct env_weather_case env_weather_cases[ENV_WEATHER_MAX];
static struct env_wind_case env_wind_cases[ENV_WIND_MAX];
static float env_gust_cases[ENV_GUST_MAX];
static int env_gust_refused[ENV_GUST_MAX];
static struct env_time_case env_time_cases[ENV_TIME_MAX];
static int env_weather_count, env_wind_count, env_gust_count, env_time_count;
static int env_tables_built;

static void env_add_ring (int season, float radius, float range)
{
	struct env_weather_case *c = &env_weather_cases[env_weather_count++];

	memset (c, 0, sizeof (*c));
	c->local = 1;
	c->season = season;
	c->radius = radius;
	c->px = range;
}

static void env_add_global (int cwm, int twm, float ts)
{
	struct env_weather_case *c = &env_weather_cases[env_weather_count++];

	memset (c, 0, sizeof (*c));
	c->local = 0;
	c->season = SESSION_SEASON_SUMMER;
	c->cwm = cwm;
	c->twm = twm;
	c->ts = ts;
	c->px = 123.0f;
	c->pz = -456.0f;
}

static void env_add_wind (float radius, float wpx, float wpz, float minimum, float maximum, float gust, float dx, float dz, float px, float pz)
{
	struct env_wind_case *c = &env_wind_cases[env_wind_count++];

	c->radius = radius;
	c->wpx = wpx;
	c->wpz = wpz;
	c->minimum = minimum;
	c->maximum = maximum;
	c->gust = gust;
	c->dx = dx;
	c->dz = dz;
	c->px = px;
	c->pz = pz;
}

static void env_add_gust (float gust, int refused)
{
	env_gust_cases[env_gust_count] = gust;
	env_gust_refused[env_gust_count] = refused;
	env_gust_count++;
}

static void env_add_time (int kind, float a, float b)
{
	struct env_time_case *c = &env_time_cases[env_time_count++];

	c->kind = kind;
	c->a = a;
	c->b = b;
}

static void env_build_tables (void)
{
	const float ring_radius = 30000.0f;
	const float wind_zone_radius = 50000.0f;
	float thresholds[3], ranges[16], wind_ranges[6];
	int k, n = 0, season, i;
	const int seasons[2] = { SESSION_SEASON_SUMMER, SESSION_SEASON_WINTER };
	const int from[3] = { WEATHERMODE_DRY, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_SNOW };
	const int to[3] = { WEATHERMODE_HEAVY_RAIN, WEATHERMODE_SNOW, WEATHERMODE_DRY };
	float steps[6];

	if (env_tables_built)
	{
		return;
	}

	/* weather, local model: the three ring edges of the original (0.333, 0.666 and 1.0 of the radius), the float below, at and above each, the centre, one point in each zone and one far outside */
	thresholds[0] = (float) ((double) ring_radius * 0.333);
	thresholds[1] = (float) ((double) ring_radius * 0.666);
	thresholds[2] = ring_radius;
	ranges[n++] = 0.0f;
	for (k = 0; k < 3; k++)
	{
		ranges[n++] = nextafterf (thresholds[k], 0.0f);
		ranges[n++] = thresholds[k];
		ranges[n++] = nextafterf (thresholds[k], ring_radius * 4.0f);
	}
	ranges[n++] = ring_radius * 0.15f;
	ranges[n++] = ring_radius * 0.5f;
	ranges[n++] = ring_radius * 0.83f;
	ranges[n++] = ring_radius * 2.0f;
	for (season = 0; season < 2; season++)
	{
		for (i = 0; i < n; i++)
		{
			env_add_ring (seasons[season], ring_radius, ranges[i]);
		}
	}
	/* a season that is neither summer nor winter must behave as summer (rain, not snow) */
	env_add_ring (SESSION_SEASON_DESERT, ring_radius, ring_radius * 0.15f);
	env_add_ring (SESSION_SEASON_DESERT, ring_radius, ring_radius * 0.5f);
	env_add_ring (SESSION_SEASON_DESERT, ring_radius, ring_radius * 0.83f);
	env_add_ring (SESSION_SEASON_DESERT, ring_radius, ring_radius * 2.0f);
	/* radius zero: the original returns dry with transitional state 1.0 everywhere */
	env_add_ring (SESSION_SEASON_SUMMER, 0.0f, 100.0f);
	/* the approximate range is not the Euclidean one: a point with both offsets */
	env_add_ring (SESSION_SEASON_SUMMER, ring_radius, 0.0f);
	env_weather_cases[env_weather_count - 1].px = 3000.0f;
	env_weather_cases[env_weather_count - 1].pz = 4000.0f;
	env_add_ring (SESSION_SEASON_WINTER, ring_radius, 0.0f);
	env_weather_cases[env_weather_count - 1].px = 3000.0f;
	env_weather_cases[env_weather_count - 1].pz = 4000.0f;

	/* weather, global/fixed model: the half-transition choice around 0.5 for mode pairs, and equal modes */
	steps[0] = 0.0f;
	steps[1] = 0.25f;
	steps[2] = nextafterf (0.5f, 0.0f);
	steps[3] = 0.5f;
	steps[4] = 0.75f;
	steps[5] = 1.0f;
	for (k = 0; k < 3; k++)
	{
		for (i = 0; i < 6; i++)
		{
			env_add_global (from[k], to[k], steps[i]);
		}
	}
	env_add_global (WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f);
	env_add_global (WEATHERMODE_SNOW, WEATHERMODE_SNOW, 0.5f);

	/* wind: inside, at and outside the zone radius (the float below and above the edge), zero radius, a zero minimum (signed zero), equal limits, reversed and zero directions, gusts */
	wind_ranges[0] = 0.0f;
	wind_ranges[1] = wind_zone_radius * 0.5f;
	wind_ranges[2] = nextafterf (wind_zone_radius, 0.0f);
	wind_ranges[3] = wind_zone_radius;
	wind_ranges[4] = nextafterf (wind_zone_radius, wind_zone_radius * 4.0f);
	wind_ranges[5] = wind_zone_radius * 2.0f;
	for (i = 0; i < 6; i++)
	{
		env_add_wind (wind_zone_radius, 0.0f, 0.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.0f, 0.6f, 0.8f, wind_ranges[i], 0.0f);
	}
	for (i = 0; i < 6; i++)
	{
		env_add_wind (wind_zone_radius, 0.0f, 0.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.3f, 0.6f, 0.8f, wind_ranges[i], 0.0f);
	}
	env_add_wind (0.0f, 0.0f, 0.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.0f, 0.6f, 0.8f, 5000.0f, 0.0f);
	env_add_wind (0.0f, 0.0f, 0.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.3f, 0.6f, 0.8f, 5000.0f, 0.0f);
	env_add_wind (wind_zone_radius, 0.0f, 0.0f, 0.0f, DEFAULT_MAX_WIND_SPEED, 0.0f, -1.0f, 0.0f, wind_zone_radius * 2.0f, 0.0f);
	env_add_wind (wind_zone_radius, 0.0f, 0.0f, 0.0f, DEFAULT_MAX_WIND_SPEED, 0.3f, -1.0f, 0.0f, wind_zone_radius * 2.0f, 0.0f);
	env_add_wind (wind_zone_radius, 0.0f, 0.0f, 0.0f, DEFAULT_MAX_WIND_SPEED, 0.0f, -1.0f, 0.0f, wind_zone_radius * 0.5f, 0.0f);
	env_add_wind (wind_zone_radius, 0.0f, 0.0f, 10.0f, 10.0f, 0.0f, 0.6f, 0.8f, wind_zone_radius * 0.5f, 0.0f);
	env_add_wind (wind_zone_radius, 0.0f, 0.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.0f, -0.6f, -0.8f, wind_zone_radius * 0.5f, 0.0f);
	env_add_wind (wind_zone_radius, 1000.0f, -2000.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.0f, 0.6f, 0.8f, 4000.0f, 2000.0f);
	env_add_wind (wind_zone_radius, 0.0f, 0.0f, DEFAULT_MIN_WIND_SPEED, DEFAULT_MAX_WIND_SPEED, 0.0f, 0.0f, 0.0f, wind_zone_radius * 0.5f, 0.0f);

	/* gust modulation: every sample boundary k/7 (the original has seven segments over [0,1)) with the float below and above, mid-segment points, signed zero and the float below 1 */
	env_add_gust (0.0f, 0);
	env_add_gust (nextafterf (0.0f, 1.0f), 0);
	env_add_gust (-0.0f, 0);
	for (k = 1; k < 7; k++)
	{
		float boundary = (float) ((double) k / 7.0);

		env_add_gust (nextafterf (boundary, 0.0f), 0);
		env_add_gust (boundary, 0);
		env_add_gust (nextafterf (boundary, 1.0f), 0);
	}
	for (k = 0; k < 7; k++)
	{
		env_add_gust ((float) (((double) k + 0.5) / 7.0), 0);
	}
	env_add_gust (0.25f, 0);
	env_add_gust (0.5f, 0);
	env_add_gust (nextafterf (1.0f, 0.0f), 0);
	/* outside [0,1) the original asserts: refused, never called */
	env_add_gust (1.0f, 1);
	env_add_gust (-0.5f, 1);

	/* clock: day roll-over and the clamp of calculate_session_time_of_day, the day segment edges (6, 8, 18, 20 hours), and the constants */
	env_add_time (0, 0.0f, 0.0f);
	env_add_time (0, (float) (6.0 * ONE_HOUR), 0.0f);
	env_add_time (0, (float) (ONE_DAY - 1.0), 0.0f);
	env_add_time (0, (float) (ONE_DAY - 1.0), 1.0f);
	env_add_time (0, (float) ONE_DAY, 0.0f);
	env_add_time (0, (float) (2.0 * ONE_DAY + 5.0), 0.0f);
	env_add_time (0, 50000.0f, 100000.0f);
	env_add_time (0, 0.0f, -5.0f);
	env_add_time (0, -0.0f, 0.0f);
	env_add_time (0, 43200.0f, 43200.0f);
	env_add_time (0, 0.0f, (float) (3.0 * ONE_DAY));
	env_add_time (1, 0.0f, 0.0f);
	for (k = 0; k < 4; k++)
	{
		static const double edges[4] = { 6.0, 8.0, 18.0, 20.0 };
		float edge = (float) (edges[k] * ONE_HOUR);

		env_add_time (1, nextafterf (edge, 0.0f), 0.0f);
		env_add_time (1, edge, 0.0f);
	}
	env_add_time (1, nextafterf ((float) ONE_DAY, 0.0f), 0.0f);
	env_add_time (2, 0.0f, 0.0f);

	env_tables_built = 1;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* saved state: restored on every path that returns */

struct env_saved_state
{
	session raw;
	int season;
	int rng;
	float delta;
};

static void env_save_state (struct env_saved_state *s)
{
	env_pull ();
	memcpy (&s->raw, env_session_raw (), sizeof (session));
	s->season = (int) get_global_season ();
	s->rng = random_number_seed;
	s->delta = get_delta_time ();
}

static void env_restore_state (const struct env_saved_state *s)
{
	memcpy (env_session_raw (), &s->raw, sizeof (session));
	env_push ();
	set_global_season (s->season);
	random_number_seed = s->rng;
	system_delta_time = s->delta;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* the cases */

static int env_census (char *text, int capacity, int *used)
{
	ENV_EMIT ("env-census version=1 session_bytes=%d weather_cases=%d wind_cases=%d gust_cases=%d time_cases=%d init_cases=%d update_time_cases=%d update_weather_cases=%d\n", (int) sizeof (session), env_weather_count,
		env_wind_count, env_gust_count, env_time_count, ENV_INIT_COUNT, ENV_UPDATE_TIME_COUNT, ENV_UPDATE_WEATHER_COUNT);
	ENV_EMIT ("env-census-enums dry=%d light_rain=%d heavy_rain=%d snow=%d season_summer=%d season_winter=%d season_desert=%d segment_dawn=%d segment_day=%d segment_dusk=%d segment_night=%d comms_server=%d\n",
		(int) WEATHERMODE_DRY, (int) WEATHERMODE_LIGHT_RAIN, (int) WEATHERMODE_HEAVY_RAIN, (int) WEATHERMODE_SNOW, (int) SESSION_SEASON_SUMMER, (int) SESSION_SEASON_WINTER, (int) SESSION_SEASON_DESERT,
		(int) DAY_SEGMENT_TYPE_DAWN, (int) DAY_SEGMENT_TYPE_DAY, (int) DAY_SEGMENT_TYPE_DUSK, (int) DAY_SEGMENT_TYPE_NIGHT, (int) COMMS_MODEL_SERVER);
	ENV_EMIT ("env-census-constants min_weather_radius_w=%08x max_weather_radius_w=%08x weather_expansion_rate_w=%08x min_wind_radius_w=%08x max_wind_radius_w=%08x wind_expansion_rate_w=%08x wind_gust_frequency_w=%08x"
		" session_resync_frequency_w=%08x default_min_wind_speed_w=%08x default_max_wind_speed_w=%08x one_hour_w=%08x one_day_w=%08x\n",
		ENV_WORD (MIN_WEATHER_RADIUS), ENV_WORD (MAX_WEATHER_RADIUS), ENV_WORD (WEATHER_EXPANSION_RATE), ENV_WORD (MIN_WIND_RADIUS), ENV_WORD (MAX_WIND_RADIUS), ENV_WORD (WIND_EXPANSION_RATE), ENV_WORD (WIND_GUST_FREQUENCY), ENV_WORD (SESSION_RESYNC_FREQUENCY),
		ENV_WORD (DEFAULT_MIN_WIND_SPEED), ENV_WORD (DEFAULT_MAX_WIND_SPEED), ENV_WORD (ONE_HOUR), ENV_WORD (ONE_DAY));
	ENV_EMIT ("env-census-configured weather_rain_radius_min_w=%08x weather_rain_radius_max_w=%08x weather_rain_speed_w=%08x weather_wind_radius_min_w=%08x weather_wind_radius_max_w=%08x weather_wind_speed_w=%08x\n",
		ENV_WORD (weather_rain_radius_min), ENV_WORD (weather_rain_radius_max), ENV_WORD (weather_rain_speed), ENV_WORD (weather_wind_radius_min), ENV_WORD (weather_wind_radius_max), ENV_WORD (weather_wind_speed));
	ENV_EMIT ("env-census-map min_map_x_w=%08x max_map_x_w=%08x min_map_z_w=%08x max_map_z_w=%08x mid_map_x_w=%08x mid_map_z_w=%08x\n", ENV_WORD (MIN_MAP_X), ENV_WORD (MAX_MAP_X), ENV_WORD (MIN_MAP_Z), ENV_WORD (MAX_MAP_Z),
		ENV_WORD (MID_MAP_X), ENV_WORD (MID_MAP_Z));
	ENV_EMIT ("env-unsupported item=get_session_random_start_time_of_day reason=reseeds_the_shared_and_the_libc_generators_from_the_clock_and_the_libc_state_cannot_be_restored\n");
	ENV_EMIT ("env-unsupported item=update_lightning_effect reason=creates_strike_and_sound_entities_and_draws_the_shared_generator_witnessed_only_through_real_frames_lightning_timer_and_rng\n");
	ENV_EMIT ("env-unsupported item=update_session_sound_effects reason=runs_in_every_headless_frame_sets_sound_amplitudes_and_writes_terrain_types_in_sector_not_witnessed_apart_from_real_frames\n");
	ENV_EMIT ("env-unsupported item=update_time_of_day_resync reason=runs_in_every_headless_server_frame_comms_transmit_each_minute_seen_only_as_the_time_of_day_resync_counter\n");
	ENV_EMIT ("env-unsupported item=wind_gust_outside_0_1 reason=the_original_asserts_so_the_cases_there_are_refused_not_called\n");
	ENV_EMIT ("env-unsupported item=session_state_loading_and_serialisation reason=ss_pack_and_ss_msgs_unpack_the_session_for_clients_and_saves_not_exercised_by_the_dedicated_server_run\n");

	return 0;
}

static int env_snapshot (char *text, int capacity, int *used)
{
	session *raw = env_session_raw ();
	vec3d *cp = env_camera_position ();
	vec3d centre;
	int start = *used;

	env_pull ();

	if (env_render_state (text, capacity, used, "env-state", "retail") != 0)
	{
		return -1;
	}
	ENV_EMIT ("env-state-fingerprint tag=retail fp=%016llx\n", (unsigned long long) env_hash_bytes (text + start, *used - start));
	/* the same original queries, asked of the unmodified state at the points a consumer would ask: the camera (what the unmodified update uses), the map centre and the two drifting centres */
	centre.x = MID_MAP_X;
	centre.y = 0.0f;
	centre.z = MID_MAP_Z;
	if (cp && env_render_queries (text, capacity, used, "env-query", "retail", "camera", cp) != 0)
	{
		return -1;
	}
	if (env_render_queries (text, capacity, used, "env-query", "retail", "map_centre", &centre) != 0 || env_render_queries (text, capacity, used, "env-query", "retail", "weather_centre", &raw->weather_position) != 0
		|| env_render_queries (text, capacity, used, "env-query", "retail", "wind_centre", &raw->wind_effect_position) != 0)
	{
		return -1;
	}

	return 0;
}

static void env_set_vec (vec3d *v, float x, float y, float z)
{
	v->x = x;
	v->y = y;
	v->z = z;
}

static int env_weather_case (int i, char *text, int capacity, int *used)
{
	const struct env_weather_case *c = &env_weather_cases[i];
	session *raw = env_session_raw ();
	vec3d pos;
	float range;

	set_global_season (c->season);
	raw->local_weather_model = c->local;
	raw->weather_radius = c->radius;
	raw->weather_mode = (weathermodes) c->cwm;
	raw->target_weather_mode = (weathermodes) c->twm;
	raw->weather_mode_transitional_status = c->ts;
	env_set_vec (&raw->weather_position, 0.0f, 0.0f, 0.0f);
	env_push ();
	env_set_vec (&pos, c->px, 0.0f, c->pz);
	range = get_approx_2d_range (&pos, &raw->weather_position);

	ENV_EMIT ("env-weather kind=witness i=%d model=%s season=%d radius_w=%08x in_cwm=%d in_twm=%d in_ts_w=%08x range_w=%08x\n", i, c->local ? "local" : "global", c->season, ENV_WORD (c->radius), c->cwm, c->twm, ENV_WORD (c->ts), ENV_WORD (range));

	return env_render_queries (text, capacity, used, "env-weather-answer", "witness", "case", &pos);
}

static int env_wind_case (int i, char *text, int capacity, int *used)
{
	const struct env_wind_case *c = &env_wind_cases[i];
	session *raw = env_session_raw ();
	vec3d pos, wind, centre;
	float speed, speed_null, modulation, range;

	raw->wind_effect_radius = c->radius;
	env_set_vec (&centre, c->wpx, 0.0f, c->wpz);
	raw->wind_effect_position = centre;
	raw->wind_minimum_speed = c->minimum;
	raw->wind_maximum_speed = c->maximum;
	raw->wind_gusting_value = c->gust;
	env_set_vec (&raw->wind_direction_vector, c->dx, 0.0f, c->dz);
	env_set_vec (&pos, c->px, 0.0f, c->pz);
	env_set_vec (&wind, 7.0f, 7.0f, 7.0f);

	if (!env_gust_in_domain (c->gust))
	{
		ENV_EMIT ("env-wind kind=witness i=%d refused=gust_outside_0_1\n", i);

		return 0;
	}

	env_push ();

	range = get_approx_2d_range (&pos, &centre);
	speed = get_session_wind_velocity_at_point (&pos, &wind);
	speed_null = get_session_wind_velocity_at_point (&pos, NULL);
	modulation = eech_env_fixture_gust_modulation (c->gust);

	ENV_EMIT ("env-wind kind=witness i=%d radius_w=%08x centre_x_w=%08x centre_z_w=%08x minimum_w=%08x maximum_w=%08x gust_w=%08x dir_x_w=%08x dir_z_w=%08x x_w=%08x z_w=%08x range_w=%08x modulation_w=%08x"
		" speed_w=%08x vx_w=%08x vy_w=%08x vz_w=%08x null_speed_w=%08x null_same=%d\n", i, ENV_WORD (c->radius), ENV_WORD (c->wpx), ENV_WORD (c->wpz), ENV_WORD (c->minimum), ENV_WORD (c->maximum), ENV_WORD (c->gust), ENV_WORD (c->dx), ENV_WORD (c->dz), ENV_WORD (c->px),
		ENV_WORD (c->pz), ENV_WORD (range), ENV_WORD (modulation), ENV_WORD (speed), ENV_WORD (wind.x), ENV_WORD (wind.y), ENV_WORD (wind.z), ENV_WORD (speed_null), float_bits (speed) == float_bits (speed_null));

	return 0;
}

/* the gust modulation alone, and the speed it gives in a calm field (radius zero, limits 10 and 20) */
static int env_gust_case (int i, char *text, int capacity, int *used)
{
	float gust = env_gust_cases[i];
	session *raw = env_session_raw ();
	vec3d wind;

	if (env_gust_refused[i] || !env_gust_in_domain (gust))
	{
		ENV_EMIT ("env-gust kind=witness i=%d gust_w=%08x refused=original_asserts_outside_0_1\n", i, ENV_WORD (gust));

		return 0;
	}

	raw->wind_effect_radius = 0.0f;
	raw->wind_minimum_speed = 10.0f;
	raw->wind_maximum_speed = 20.0f;
	raw->wind_gusting_value = gust;
	env_set_vec (&raw->wind_direction_vector, 1.0f, 0.0f, 0.0f);
	env_push ();
	env_set_vec (&wind, 0.0f, 0.0f, 0.0f);
	{
		vec3d pos;
		float modulation, speed;

		env_set_vec (&pos, 0.0f, 0.0f, 0.0f);
		modulation = eech_env_fixture_gust_modulation (gust);
		speed = get_session_wind_velocity_at_point (&pos, &wind);
		ENV_EMIT ("env-gust kind=witness i=%d gust_w=%08x modulation_w=%08x calm_speed_w=%08x calm_vx_w=%08x\n", i, ENV_WORD (gust), ENV_WORD (modulation), ENV_WORD (speed), ENV_WORD (wind.x));
	}

	return 0;
}

static int env_time_case (int i, char *text, int capacity, int *used)
{
	const struct env_time_case *c = &env_time_cases[i];
	session *raw = env_session_raw ();

	if (c->kind == 0)
	{
		int day = -1, accessor_day;
		float tod, accessor_tod;

		raw->start_time = c->a;
		raw->elapsed_time = c->b;
		tod = calculate_session_time_of_day (get_session_entity (), &day);
		accessor_tod = get_local_entity_float_value (get_session_entity (), FLOAT_TYPE_TIME_OF_DAY);
		accessor_day = get_local_entity_int_value (get_session_entity (), INT_TYPE_DAY);
		ENV_EMIT ("env-time kind=witness i=%d what=calculate start_w=%08x elapsed_w=%08x tod_w=%08x rolled_days=%d accessor_tod_w=%08x accessor_day=%d segment=%d\n", i, ENV_WORD (c->a), ENV_WORD (c->b), ENV_WORD (tod), day,
			ENV_WORD (accessor_tod), accessor_day, (int) get_day_segment_type (tod));
	}
	else if (c->kind == 1)
	{
		ENV_EMIT ("env-time kind=witness i=%d what=segment tod_w=%08x segment=%d\n", i, ENV_WORD (c->a), (int) get_day_segment_type (c->a));
	}
	else
	{
		ENV_EMIT ("env-time kind=witness i=%d what=constants segment_time_dawn_w=%08x segment_time_day_w=%08x segment_time_dusk_w=%08x segment_time_night_w=%08x preset_dawn_w=%08x preset_midday_w=%08x"
			" preset_afternoon_w=%08x preset_dusk_w=%08x preset_midnight_w=%08x\n", i, ENV_WORD (get_segment_time_of_day (DAY_SEGMENT_TYPE_DAWN)), ENV_WORD (get_segment_time_of_day (DAY_SEGMENT_TYPE_DAY)),
			ENV_WORD (get_segment_time_of_day (DAY_SEGMENT_TYPE_DUSK)), ENV_WORD (get_segment_time_of_day (DAY_SEGMENT_TYPE_NIGHT)), ENV_WORD (get_session_preset_time_of_day (SESSION_TIME_DAWN)),
			ENV_WORD (get_session_preset_time_of_day (SESSION_TIME_MIDDAY)), ENV_WORD (get_session_preset_time_of_day (SESSION_TIME_AFTERNOON)), ENV_WORD (get_session_preset_time_of_day (SESSION_TIME_DUSK)),
			ENV_WORD (get_session_preset_time_of_day (SESSION_TIME_MIDNIGHT)));
	}

	return 0;
}

/* the ORIGINAL initialisers on the live session: the four fixed modes, and the random mode from three controlled generator seeds (the shared seed word is restored) */
static int env_init_case (int i, char *text, int capacity, int *used)
{
	static const int modes[4] = { WEATHERMODE_DRY, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, WEATHERMODE_SNOW };
	static const int seeds[3] = { 1, 1234567, 20261030 };
	entity *en = get_session_entity ();
	vec3d a, b;

	if (get_comms_model () != COMMS_MODEL_SERVER)
	{
		ENV_EMIT ("env-init kind=witness i=%d refused=the_original_initialisers_assert_the_server_comms_model comms_model=%d\n", i, (int) get_comms_model ());

		return 0;
	}

	if (i < 4)
	{
		set_session_fixed_weather_mode (en, (weathermodes) modes[i]);
		ENV_EMIT ("env-init kind=witness i=%d what=fixed mode=%d rng=%d\n", i, modes[i], random_number_seed);
	}
	else
	{
		random_number_seed = seeds[i - 4];
		set_session_random_weather_mode (en);
		ENV_EMIT ("env-init kind=witness i=%d what=random seed_in=%d rng=%d rng_w=%08x\n", i, seeds[i - 4], random_number_seed, (unsigned) random_number_seed);
	}

	if (env_render_state (text, capacity, used, "env-init-state", "witness") != 0)
	{
		return -1;
	}
	env_set_vec (&a, 0.0f, 0.0f, 0.0f);
	env_set_vec (&b, MID_MAP_X, 0.0f, MID_MAP_Z);

	if (env_render_queries (text, capacity, used, "env-init-query", "witness", "origin", &a) != 0)
	{
		return -1;
	}

	return env_render_queries (text, capacity, used, "env-init-query", "witness", "map_centre", &b);
}

/* the original update of the time of day on synthetic state (update_time_of_day, exposed under this feature) */
static int env_update_time_case (int i, char *text, int capacity, int *used)
{
	static const char *const labels[ENV_UPDATE_TIME_COUNT] = { "midday_real_time", "midday_accelerated_60", "night_to_dawn_edge", "dawn_to_day_edge", "day_to_dusk_edge", "dusk_to_night_edge", "day_wrap",
		"skip_night_after_dusk", "skip_night_before_dawn", "zero_delta" };
	session *raw = env_session_raw ();
	float start = 43200.0f, accel = 1.0f, delta = 0.1f;
	int skip = 0, day_before = 0, day_after = 0;
	float tod_before, tod_after;
	char tag[48];

	switch (i)
	{
		case 1: accel = 60.0f; break;
		case 2: start = nextafterf ((float) (6.0 * ONE_HOUR), 0.0f); break;
		case 3: start = nextafterf ((float) (8.0 * ONE_HOUR), 0.0f); break;
		case 4: start = nextafterf ((float) (18.0 * ONE_HOUR), 0.0f); break;
		case 5: start = nextafterf ((float) (20.0 * ONE_HOUR), 0.0f); break;
		case 6: start = nextafterf ((float) ONE_DAY, 0.0f); break;
		case 7: start = (float) (21.0 * ONE_HOUR); skip = 1; break;
		case 8: start = (float) (1.0 * ONE_HOUR); skip = 1; break;
		case 9: delta = 0.0f; break;
		default: break;
	}

	raw->start_time = start;
	raw->elapsed_time = 0.0f;
	raw->time_of_day_acceleration = accel;
	raw->skip_night_time = skip;
	system_delta_time = delta;
	tod_before = calculate_session_time_of_day (get_session_entity (), &day_before);
	snprintf (tag, sizeof (tag), "before_%d", i);
	ENV_EMIT ("env-update-time kind=witness i=%d label=%s delta_w=%08x accel_w=%08x skip_night=%d tod_before_w=%08x day_before=%d\n", i, labels[i], ENV_WORD (delta), ENV_WORD (accel), skip, ENV_WORD (tod_before), day_before + 1);
	if (env_render_state (text, capacity, used, "env-update-time-state", tag) != 0)
	{
		return -1;
	}
	eech_env_fixture_update_time_of_day (get_session_entity ());
	tod_after = calculate_session_time_of_day (get_session_entity (), &day_after);
	snprintf (tag, sizeof (tag), "after_%d", i);
	ENV_EMIT ("env-update-time-result kind=witness i=%d tod_after_w=%08x day_after=%d\n", i, ENV_WORD (tod_after), day_after + 1);

	return env_render_state (text, capacity, used, "env-update-time-state", tag);
}

/* the original update of the weather and the wind on synthetic state (update_transitional_weather, non-static in ss_updt.c). Lightning is kept from firing (its timer is far from zero). */
static int env_update_weather_case (int i, char *text, int capacity, int *used)
{
	static const char *const labels[ENV_UPDATE_WEATHER_COUNT] = { "local_interior", "local_radius_clamps_at_max", "local_radius_clamps_at_min", "local_bounce_min_x", "local_bounce_max_x", "local_bounce_min_z",
		"local_bounce_max_z", "local_radius_zero", "global_transition_progress", "global_transition_completes", "global_steady", "gust_phase_wraps", "gust_phase_at_one", "wind_radius_clamps_at_max",
		"wind_radius_clamps_at_min", "wind_bounce_min_x_and_max_z" };
	session *raw = env_session_raw ();
	vec3d *cp = env_camera_position ();
	float span_x = MAX_MAP_X - MIN_MAP_X, span_z = MAX_MAP_Z - MIN_MAP_Z;
	char tag[48];

	if (in_cockpit || !cp || get_comms_model () != COMMS_MODEL_SERVER)
	{
		ENV_EMIT ("env-update-weather kind=witness i=%d refused=unavailable_in_cockpit_or_no_camera_or_not_server in_cockpit=%d camera=%d comms_model=%d\n", i, (int) in_cockpit, cp ? 1 : 0, (int) get_comms_model ());

		return 0;
	}

	/* the base state: local model, interior centres, ordinary drift, lightning far away */
	set_global_season (SESSION_SEASON_SUMMER);
	raw->local_weather_model = 1;
	raw->weather_radius = 30000.0f;
	raw->weather_increasing = 1;
	env_set_vec (&raw->weather_position, MIN_MAP_X + 0.4f * span_x, 0.0f, MIN_MAP_Z + 0.4f * span_z);
	env_set_vec (&raw->weather_velocity, 100.0f, 0.0f, -50.0f);
	raw->weather_mode = WEATHERMODE_DRY;
	raw->target_weather_mode = WEATHERMODE_DRY;
	raw->weather_mode_transitional_status = 0.0f;
	raw->weather_mode_transitional_period = 60.0f;
	raw->lightning_timer = 1.0e6f;
	raw->wind_effect_radius = 40000.0f;
	raw->wind_increasing = 1;
	raw->wind_gusting_value = 0.5f;
	env_set_vec (&raw->wind_effect_position, MIN_MAP_X + 0.7f * span_x, 0.0f, MIN_MAP_Z + 0.3f * span_z);
	env_set_vec (&raw->wind_effect_velocity, -80.0f, 0.0f, 60.0f);
	env_set_vec (&raw->wind_direction_vector, 0.0f, 0.0f, 1.0f);
	system_delta_time = 0.1f;

	switch (i)
	{
		case 1: raw->weather_radius = MAX_WEATHER_RADIUS - 5.0f; break;
		case 2: raw->weather_radius = MIN_WEATHER_RADIUS + 5.0f; raw->weather_increasing = 0; break;
		case 3: raw->weather_position.x = MIN_MAP_X + 1.0f; raw->weather_velocity.x = -100.0f; break;
		case 4: raw->weather_position.x = MAX_MAP_X - 1.0f; raw->weather_velocity.x = 100.0f; break;
		case 5: raw->weather_position.z = MIN_MAP_Z + 1.0f; raw->weather_velocity.z = -100.0f; break;
		case 6: raw->weather_position.z = MAX_MAP_Z - 1.0f; raw->weather_velocity.z = 100.0f; break;
		case 7: raw->weather_radius = 0.0f; raw->weather_mode = WEATHERMODE_HEAVY_RAIN; raw->target_weather_mode = WEATHERMODE_SNOW; break;
		case 8: raw->local_weather_model = 0; raw->weather_mode = WEATHERMODE_DRY; raw->target_weather_mode = WEATHERMODE_HEAVY_RAIN; raw->weather_mode_transitional_status = 0.5f; break;
		case 9: raw->local_weather_model = 0; raw->weather_mode = WEATHERMODE_DRY; raw->target_weather_mode = WEATHERMODE_HEAVY_RAIN; raw->weather_mode_transitional_status = 0.99999f; system_delta_time = 1.0f; break;
		case 10: raw->local_weather_model = 0; raw->weather_mode = WEATHERMODE_LIGHT_RAIN; raw->target_weather_mode = WEATHERMODE_LIGHT_RAIN; raw->weather_mode_transitional_status = 0.7f; break;
		case 11: raw->wind_gusting_value = 0.95f; system_delta_time = 1.0f; break;
		case 12: raw->wind_gusting_value = 0.9f; system_delta_time = 1.0f; break;
		case 13: raw->wind_effect_radius = MAX_WIND_RADIUS - 5.0f; break;
		case 14: raw->wind_effect_radius = MIN_WIND_RADIUS + 5.0f; raw->wind_increasing = 0; break;
		case 15: raw->wind_effect_position.x = MIN_MAP_X + 1.0f; raw->wind_effect_velocity.x = -100.0f; break;
		default: break;
	}
	if (i == 15)
	{
		/* one case for the second wind edge as well: the z edge from the base's interior x */
		raw->wind_effect_position.z = MAX_MAP_Z - 1.0f;
		raw->wind_effect_velocity.z = 100.0f;
	}

	env_push ();

	snprintf (tag, sizeof (tag), "before_%d", i);
	ENV_EMIT ("env-update-weather kind=witness i=%d label=%s delta_w=%08x camera_dependent_fields=weather_mode,target_weather_mode,transitional_status_when_local_and_radius_nonzero camera=%d\n", i, labels[i],
		ENV_WORD (get_delta_time ()), 1);
	if (env_render_state (text, capacity, used, "env-update-weather-state", tag) != 0)
	{
		return -1;
	}
	update_transitional_weather (get_session_entity ());
	snprintf (tag, sizeof (tag), "after_%d", i);

	return env_render_state (text, capacity, used, "env-update-weather-state", tag);
}

/* group 9, EXPLICITLY NOT A WITNESS: a controlled-weather case of the ORIGINAL setters, for a fresh process. It changes the live session (the original's own setters, with physical inputs) and
   restores NOTHING: no random draw, entity or libc state it causes could be put back, so the process is the isolation boundary and the report says `changed=1` and `process_isolated=required`. It
   is never part of the default run, the census or the witness set. i=0 brings the next lightning strike forward (only if the weather is already the local model with a non-zero radius); i=1
   first makes the weather the local model with the radius the original's own session defaults use (40 km), then does the same. */
static int env_controlled_case (int i, char *text, int capacity, int *used)
{
	static const char *const what[ENV_CONTROLLED_COUNT] = { "lightning_due", "local_weather_on" };
	entity *en = get_session_entity ();
	session *raw = env_session_raw ();
	int local_before = (int) raw->local_weather_model;
	unsigned radius_before = ENV_WORD (raw->weather_radius), timer_before = ENV_WORD (raw->lightning_timer);

	if (get_comms_model () != COMMS_MODEL_SERVER)
	{
		ENV_EMIT ("env-controlled kind=controlled i=%d what=%s refused=not_the_server comms_model=%d\n", i, what[i], (int) get_comms_model ());
		ENV_EMIT ("env-controlled-end group=9 index=%d changed=0 process_isolated=required\n", i);

		return 0;
	}

	if (i == 0 && (!local_before || raw->weather_radius == 0.0f))
	{
		ENV_EMIT ("env-controlled kind=controlled i=%d what=%s refused=weather_not_local_or_radius_zero local_weather_model=%d weather_radius_w=%08x\n", i, what[i], local_before, radius_before);
		ENV_EMIT ("env-controlled-end group=9 index=%d changed=0 process_isolated=required\n", i);

		return 0;
	}

	if (i == 1)
	{
		set_local_entity_int_value (en, INT_TYPE_LOCAL_WEATHER_MODEL, TRUE);
		set_local_entity_float_value (en, FLOAT_TYPE_WEATHER_RADIUS, 40.0 * KILOMETRE);
	}

	set_local_entity_float_value (en, FLOAT_TYPE_LIGHTNING_TIMER, 0.5);

	ENV_EMIT ("env-controlled kind=controlled i=%d what=%s local_before=%d radius_before_w=%08x timer_before_w=%08x local_after=%d radius_after_w=%08x timer_after_w=%08x\n", i, what[i], local_before,
		radius_before, timer_before, (int) raw->local_weather_model, ENV_WORD (raw->weather_radius), ENV_WORD (raw->lightning_timer));
	ENV_EMIT ("env-controlled-end group=9 index=%d changed=1 process_isolated=required\n", i);

	return 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* group 10, READ-ONLY apart from a scratch it restores: the ORIGINAL terrain-sector surface enumeration that the sound routine calls every frame.

   update_session_sound_effects (ss_updt.c) clamps the camera to the map and calls get_terrain_3d_types_in_sector (terrdata.c), which selects a sector, CLEARS the shared int array
   terrain_types_in_sector, flags the sector's first surface and the surface of EVERY surface_change polygon, and returns 1 plus the NUMBER OF TRANSITIONS (the count of
   surface_change flags walked) -- not the number of distinct flags. This group calls that original on real loaded sectors and reports what it returned and what it left. The observer's
   existing hash of the same scratch inside a real frame is REUSED as it is; this group adds only the missing surface-enumeration semantics (the return value against the flags,
   the reset between calls, the boundary samples).

   Nothing here recomputes an answer: the return value and the flags are the original's. Per case the original is called four times (each after the scratch was filled with a different
   sentinel, so a flag that survived would show; the second call repeats the first; an interleaved call on the reference sector sits between a call and its repeat) and the scratch is
   put back byte for byte. A sample the original cannot be asked -- a sector outside the loaded grid (its index would address memory that is not a sector), a sector with no surface
   table or no polygon table, a surface id that is not a terrain type (the original would index its scratch out of range) -- is REJECTED at the boundary with the reason, not called and not
   repaired. The sector the original selects is the original's own (get_terrain_3d_sector, which truncates toward zero and sets the FPU rounding mode to zero, the engine's own mode).

   Counts are kept apart on purpose: `returned` is the original's return value, `distinct_flags` is the number of scratch entries left set, `polygons` is the sector's raw polygon count.
   The cases are a finite sweep of real sector centres, the corners and the middle of the loaded grid, boundary samples (the float below, at and above a sector's lower and upper edge in x
   and in z) around three sectors, and the clamp limits the sound routine itself uses. Index 0 is the plan. */

#define ENV_SECTOR_MAX 192
#define ENV_SECTOR_SWEEP 48

struct env_sector_case
{
	const char *label;
	float x, z;
};

static struct env_sector_case env_sector_cases[ENV_SECTOR_MAX];
static int env_sector_count;

static void env_sector_add (const char *label, float x, float z)
{
	if (env_sector_count < ENV_SECTOR_MAX)
	{
		env_sector_cases[env_sector_count].label = label;
		env_sector_cases[env_sector_count].x = x;
		env_sector_cases[env_sector_count].z = z;
		env_sector_count++;
	}
}

static float env_sector_centre (int index)
{
	return ((float) index + 0.5f) * (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
}

static void env_sector_edges (int sx, int sz)
{
	float side = (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
	float x0 = (float) sx * side, z0 = (float) sz * side;
	float xc = env_sector_centre (sx), zc = env_sector_centre (sz);

	env_sector_add ("x_below_lower_edge", nextafterf (x0, -HUGE_VALF), zc);
	env_sector_add ("x_at_lower_edge", x0, zc);
	env_sector_add ("x_above_lower_edge", nextafterf (x0, HUGE_VALF), zc);
	env_sector_add ("x_below_upper_edge", nextafterf (x0 + side, -HUGE_VALF), zc);
	env_sector_add ("x_at_upper_edge", x0 + side, zc);
	env_sector_add ("z_below_lower_edge", xc, nextafterf (z0, -HUGE_VALF));
	env_sector_add ("z_at_lower_edge", xc, z0);
	env_sector_add ("z_above_lower_edge", xc, nextafterf (z0, HUGE_VALF));
	env_sector_add ("z_below_upper_edge", xc, nextafterf (z0 + side, -HUGE_VALF));
	env_sector_add ("z_at_upper_edge", xc, z0 + side);
}

static void env_sector_build (void)
{
	int xmax = terrain_3d_sector_x_max, zmax = terrain_3d_sector_z_max;
	float side = (float) TERRAIN_3D_SECTOR_SIDE_LENGTH;
	int total, step, k;

	env_sector_count = 0;

	if (!terrain_sectors || xmax < 1 || zmax < 1)
	{
		return;
	}

	total = xmax * zmax;
	step = total / ENV_SECTOR_SWEEP + 1;

	/* the first case is the interleave reference: the centre of the first sector */
	env_sector_add ("sector_first", env_sector_centre (0), env_sector_centre (0));
	env_sector_add ("sector_x_last", env_sector_centre (xmax - 1), env_sector_centre (0));
	env_sector_add ("sector_z_last", env_sector_centre (0), env_sector_centre (zmax - 1));
	env_sector_add ("sector_last", env_sector_centre (xmax - 1), env_sector_centre (zmax - 1));
	env_sector_add ("sector_middle", env_sector_centre (xmax / 2), env_sector_centre (zmax / 2));

	for (k = 0; k < ENV_SECTOR_SWEEP; k++)
	{
		int index = (k * step) % total;

		env_sector_add ("sweep", env_sector_centre (index % xmax), env_sector_centre (index / xmax));
	}

	env_sector_edges (0, 0);
	env_sector_edges (xmax / 2, zmax / 2);
	env_sector_edges (xmax - 1, zmax - 1);

	env_sector_add ("zero", 0.0f, 0.0f);
	env_sector_add ("negative_zero", -0.0f, -0.0f);
	env_sector_add ("minus_half_metre_truncates_toward_zero", -0.5f, -0.5f);
	env_sector_add ("minus_one_sector", -side, env_sector_centre (0));
	env_sector_add ("terrain_end_minus_1mm", (float) xmax * side - 0.001f, (float) zmax * side - 0.001f);
	env_sector_add ("terrain_end", (float) xmax * side, (float) zmax * side);
	env_sector_add ("sound_clamp_minimum", MIN_MAP_X, MIN_MAP_Z);
	env_sector_add ("sound_clamp_middle", MID_MAP_X, MID_MAP_Z);
	env_sector_add ("sound_clamp_maximum", MAX_MAP_X, MAX_MAP_Z);
}

/* NULL when the original can be asked about this point, else the reason it cannot (it would address memory that is not a sector, or index its scratch out of range) */
static const char *env_sector_domain (float x, float z, int *sx, int *sz, int *polygons)
{
	const terrain_3d_sector *sector;
	const terrain_3d_surface *surface;
	const terrain_3d_face *polygon;
	int count;

	*sx = -1;
	*sz = -1;
	*polygons = -1;

	if (!isfinite (x) || !isfinite (z))
	{
		return "not_finite";
	}

	/* the original's own sector selection (it truncates toward zero) */
	get_terrain_3d_sector (x, z, sx, sz);

	if (*sx < 0 || *sx >= terrain_3d_sector_x_max || *sz < 0 || *sz >= terrain_3d_sector_z_max)
	{
		return "sector_outside_the_loaded_grid";
	}

	sector = &terrain_sectors[*sz][*sx];
	*polygons = (int) sector->number_of_polygons;

	if (!sector->surface_changes)
	{
		return "no_surface_table";
	}

	if (sector->number_of_polygons > 0 && !sector->polygons)
	{
		return "no_polygon_table";
	}

	/* the same pointers the original walks, read only for the one thing that would be undefined behaviour in the original: a surface id outside its scratch */
	surface = sector->surface_changes;
	polygon = sector->polygons;

	if ((int) surface->surface_id >= TERRAIN_TYPE_LAST)
	{
		return "surface_id_outside_the_terrain_types";
	}

	for (count = (int) sector->number_of_polygons; count > 0; count--)
	{
		if (polygon->surface_change)
		{
			surface++;

			if ((int) surface->surface_id >= TERRAIN_TYPE_LAST)
			{
				return "surface_id_outside_the_terrain_types";
			}
		}

		polygon++;
	}

	return NULL;
}

static void env_sector_fill (int value)
{
	int i;

	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		terrain_types_in_sector[i] = value;
	}
}

static int env_sector_nonbinary (const int *flags)
{
	int i, n = 0;

	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		if (flags[i] != 0 && flags[i] != 1)
		{
			n++;
		}
	}

	return n;
}

static int env_sector_distinct (const int *flags)
{
	int i, n = 0;

	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		if (flags[i] != 0)
		{
			n++;
		}
	}

	return n;
}

static void env_sector_text (const int *flags, char *out)
{
	int i;

	for (i = 0; i < TERRAIN_TYPE_LAST; i++)
	{
		out[i] = (flags[i] == 0) ? '0' : ((flags[i] == 1) ? '1' : '?');
	}

	out[TERRAIN_TYPE_LAST] = 0;
}

static int env_sector_case (int index, char *text, int capacity, int *used)
{
	int saved[TERRAIN_TYPE_LAST], first[TERRAIN_TYPE_LAST], second[TERRAIN_TYPE_LAST], third[TERRAIN_TYPE_LAST];
	char flags[TERRAIN_TYPE_LAST + 1];
	const struct env_sector_case *c;
	const char *refused;
	int sx, sz, polygons, rsx, rsz, rpolygons;
	int returned, repeated, other = -1, again = -1, interleave_known;

	env_sector_build ();

	if (index == 0)
	{
		ENV_EMIT ("env-sector-plan version=1 terrain_loaded=%d sector_x_max=%d sector_z_max=%d side=%d terrain_types=%d cases=%d scratch_ints=%d map_min_x_w=%08x map_max_x_w=%08x map_min_z_w=%08x map_max_z_w=%08x"
			" map_mid_x_w=%08x map_mid_z_w=%08x\n", terrain_sectors ? 1 : 0, terrain_3d_sector_x_max, terrain_3d_sector_z_max, (int) TERRAIN_3D_SECTOR_SIDE_LENGTH, (int) TERRAIN_TYPE_LAST, env_sector_count,
			(int) (sizeof (terrain_types_in_sector) / sizeof (terrain_types_in_sector[0])), ENV_WORD (MIN_MAP_X), ENV_WORD (MAX_MAP_X), ENV_WORD (MIN_MAP_Z), ENV_WORD (MAX_MAP_Z), ENV_WORD (MID_MAP_X),
			ENV_WORD (MID_MAP_Z));

		return 0;
	}

	if (index < 1 || index > env_sector_count)
	{
		return -1;
	}

	c = &env_sector_cases[index - 1];

	refused = env_sector_domain (c->x, c->z, &sx, &sz, &polygons);

	if (refused)
	{
		ENV_EMIT ("env-sector kind=original i=%d label=%s x_w=%08x z_w=%08x sector_x=%d sector_z=%d polygons=%d refused=%s\n", index, c->label, ENV_WORD (c->x), ENV_WORD (c->z), sx, sz, polygons, refused);

		return 0;
	}

	memcpy (saved, terrain_types_in_sector, sizeof (saved));

	env_sector_fill (7);
	returned = get_terrain_3d_types_in_sector (c->x, c->z);
	memcpy (first, terrain_types_in_sector, sizeof (first));

	env_sector_fill (9);
	repeated = get_terrain_3d_types_in_sector (c->x, c->z);
	memcpy (second, terrain_types_in_sector, sizeof (second));

	/* an interleaved call on the first case's sector (when the original can be asked about it) between a call and its repeat */
	interleave_known = env_sector_domain (env_sector_cases[0].x, env_sector_cases[0].z, &rsx, &rsz, &rpolygons) == NULL;

	if (interleave_known)
	{
		other = get_terrain_3d_types_in_sector (env_sector_cases[0].x, env_sector_cases[0].z);
		again = get_terrain_3d_types_in_sector (c->x, c->z);
	}

	memcpy (third, terrain_types_in_sector, sizeof (third));

	memcpy (terrain_types_in_sector, saved, sizeof (saved));

	env_sector_text (first, flags);

	ENV_EMIT ("env-sector kind=original i=%d label=%s x_w=%08x z_w=%08x sector_x=%d sector_z=%d polygons=%d returned=%d distinct_flags=%d nonbinary=%d flags=%s repeated=%d repeat_equal=%d"
		" sentinel_cleared=%d interleave=%s interleave_other_returned=%d interleave_returned=%d interleave_equal=%d restored=%d\n", index, c->label, ENV_WORD (c->x), ENV_WORD (c->z), sx, sz, polygons, returned,
		env_sector_distinct (first), env_sector_nonbinary (first), flags, repeated, (repeated == returned) && (memcmp (first, second, sizeof (first)) == 0),
		env_sector_nonbinary (first) == 0 && env_sector_nonbinary (second) == 0, interleave_known ? "done" : "na", other, again,
		interleave_known ? ((again == returned) && (memcmp (first, third, sizeof (first)) == 0)) : -1, memcmp (terrain_types_in_sector, saved, sizeof (saved)) == 0);

	return 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* group 11 (EEDCS-51, D3, the ORIGINAL ambient sound routine): explicit only, never part of the default run, the census or the 164-case oracle.

   update_session_sound_effects (ss_updt.c, UNCHANGED in this change) runs in every server update. For the camera (or, with in_cockpit, the gunship) it computes
     rain    the session weather at the point (current mode, target mode, transitional state) as per-mode levels, switched off above the cloud base and, outdoors, above 180 m of radar
             altitude, attenuated outdoors by 1 - alt / 180;
     wind    the session wind speed at the point against the session's minimum and maximum speed, as a light level and a heavy level (a piecewise function of r in [0,1]);
     terrain the shared scratch terrain_types_in_sector of the sector under the camera clamped to the map (get_terrain_3d_types_in_sector, characterised by group 10 and already host-owned);
   and then walks the session's LIST_TYPE_SPECIAL_EFFECT children and sets FLOAT_TYPE_AMPLIFICATION on the ten ambient sound children the campaign attaches at boot
   (attach_session_sound_effects): rain and wind from the levels above, and for the six terrain children (sea, jungle = forest top, amb1 = river, amb2 = reservoir, amb3 / amb4 = altered
   land 1 / 2) a smoothed value: the child's PREVIOUS amplification plus or minus 0.5 * the frame delta (up when its terrain type is flagged in the sector, down when not), bounded to
   [0, 0.3], times bound (1 - alt / 280, 0, 1), times 0.75 in the cockpit; zero at or above 280 m. The value written is the scaled one and is read back next call, so consecutive calls compound.

   WHAT THIS GROUP IS. The REAL unchanged routine is called on the REAL collaborating state of the booted campaign: the session entity and its ten real ambient sound-effect children, the real
   camera entity, the real loaded terrain (and, in the host builds, the real host providers). Per case the group sets the controlled inputs the routine reads (named below), prints the original
   queries the routine consumes at its point as an input echo, calls the routine once or several times and prints what the children hold afterwards, as IEEE words. It recomputes NOTHING:
   every amplification word is the original's. The inputs it controls:
     camera       position (x, y, z) -- the camera's cached terrain elevation is forced to 0 so the radar altitude is exactly y -- and the cloud base height (the shared global
                  cloud_3d_base_height, which the full game sets to 6000 at render start-up: the headless run has no renderer, so it is set here and PRINTED, and put back);
     session      the weather (global model: current / target mode and transitional state; or the local ring around a point at a fraction of its radius) and the wind (limits, zone radius
                  and centre, gust phase), pushed to the host field with the campaign's own import in the host builds;
     the frame    delta time; the children's initial amplification (the terrain children a controlled value, the rain and wind children a sentinel to show they are overwritten);
     in_cockpit   only when the headless campaign holds a gunship entity (the routine then reads the gunship's live position and radar altitude, which are NOT moved).
   The sector under the camera is chosen from the real loaded terrain by a bounded probe (at most ENV_SOUND_PROBES sectors, stride over the grid) with the ORIGINAL sector query: the first
   sector whose scratch has each of the six sound-relevant surfaces set, and the first with none of them set. A surface the probe does not find makes its cases `refused`, with the reason.

   WHAT IT DOES NOT SHOW (printed in the plan as `env-sound-unsupported`): the speaker output or any render of these children (the sound-effect update that plays them, se_snd.c, needs the
   sound system; the headless build has none), the ALTERED_LAND3 reader of 3drain.c (render, dormant), the routine called through a real update_server (the observer's sound_facts, group 8's
   frames and the sector driver's observer already establish that call, its place in the update order and the shared-RNG draw count of zero), and any non-ambient special-effect child of the
   session (none exists at boot: the lightning strike sounds appear only after a strike, which is the lightning effect's area). Untouched children are checked where they exist: the whole child
   list, in its order, is printed before and after each call.

   Restored before every return: the session record (and the host field), the season, the shared random seed, the frame delta, the camera position and cached elevation, the cloud base,
   in_cockpit, the terrain scratch and the amplification of every sound child. `env-sound-end` says restored=0|1 for the whole and for each part; 0 would be a defect of this fixture, never a
   result. Index 0 is the plan. */

extern void update_session_sound_effects (entity *en);
extern float cloud_3d_base_height;

#define ENV_SOUND_MAX 64
#define ENV_SOUND_CHILDREN 32
#define ENV_SOUND_PROBES 1024
#define ENV_SOUND_CLOUD_BASE 6000.0f
#define ENV_SOUND_SENTINEL 0.625f
#define ENV_SOUND_RING_RADIUS 30000.0f
#define ENV_SOUND_WIND_RADIUS 8192.0f
#define ENV_SOUND_CLASSES 10

enum
{
	ENV_SITE_ABSENT,
	ENV_SITE_SEA,
	ENV_SITE_FOREST_TOP,
	ENV_SITE_RIVER,
	ENV_SITE_RESERVOIR,
	ENV_SITE_ALTERED_LAND1,
	ENV_SITE_ALTERED_LAND2,
	ENV_SITE_PROBED = 7, /* the number of sites the probe looks for */
	ENV_SITE_OUTSIDE_HIGH = 7,
	ENV_SITE_OUTSIDE_LOW = 8,
	ENV_SITE_GUNSHIP = 9
};

struct env_sound_site
{
	int found, sx, sz;
	float x, z;
};

struct env_sound_case
{
	const char *label;
	int site, cloud, model, cwm, twm, cockpit, calls;
	float alt, ts, ring, zone, gust, wmin, wmax, amp0, dt;
};

struct env_sound_child
{
	entity *en;
	int sub_type, klass, is_sound;
	float amp;
};

static struct env_sound_site env_sound_sites[ENV_SITE_PROBED];
static struct env_sound_case env_sound_cases[ENV_SOUND_MAX];
static int env_sound_count, env_sound_built;
static int env_sound_probed;

/* the ten ambient sound children, in the order of attach_session_sound_effects; 0 for any other child */
static int env_sound_class (int sub_type)
{
	switch (sub_type)
	{
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_LIGHT_RAIN: return 1;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_HEAVY_RAIN: return 2;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_LIGHT_WIND: return 3;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_HEAVY_WIND: return 4;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_SEA: return 5;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_JUNGLE: return 6;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMB1: return 7;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMB2: return 8;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMB3: return 9;
		case ENTITY_SUB_TYPE_EFFECT_SOUND_AMB4: return 10;
		default: return 0;
	}
}

static const char *env_sound_class_name (int klass)
{
	static const char *const names[ENV_SOUND_CLASSES + 1] = { "other", "light_rain", "heavy_rain", "light_wind", "heavy_wind", "sea", "jungle", "amb1", "amb2", "amb3", "amb4" };

	return names[(klass >= 0 && klass <= ENV_SOUND_CLASSES) ? klass : 0];
}

/* the session's special-effect children in list order; -1 when there are more than the table holds */
static int env_sound_children (struct env_sound_child *kids, int max)
{
	entity *spec = get_local_entity_first_child (get_session_entity (), LIST_TYPE_SPECIAL_EFFECT);
	int n = 0;

	while (spec)
	{
		if (n >= max)
		{
			return -1;
		}

		kids[n].en = spec;
		kids[n].sub_type = get_local_entity_int_value (spec, INT_TYPE_ENTITY_SUB_TYPE);
		kids[n].klass = env_sound_class (kids[n].sub_type);
		kids[n].is_sound = get_local_entity_type (spec) == ENTITY_TYPE_SOUND_EFFECT;
		kids[n].amp = kids[n].is_sound ? get_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION) : 0.0f;
		n++;

		spec = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);
	}

	return n;
}

/* every one of the ten ambient classes exactly once */
static int env_sound_complete (const struct env_sound_child *kids, int n)
{
	int seen[ENV_SOUND_CLASSES + 1], i;

	memset (seen, 0, sizeof (seen));

	for (i = 0; i < n; i++)
	{
		seen[kids[i].klass]++;
	}

	for (i = 1; i <= ENV_SOUND_CLASSES; i++)
	{
		if (seen[i] != 1)
		{
			return 0;
		}
	}

	return 1;
}

static int env_sound_print_children (char *text, int capacity, int *used, const char *key, const struct env_sound_child *kids, int n)
{
	int i;

	ENV_EMIT (" %s=", key);

	for (i = 0; i < n; i++)
	{
		if (kids[i].is_sound)
		{
			ENV_EMIT ("%s%d:%d:%s:%08x", i ? "," : "", i, kids[i].sub_type, env_sound_class_name (kids[i].klass), ENV_WORD (kids[i].amp));
		}
		else
		{
			ENV_EMIT ("%s%d:%d:%s:na", i ? "," : "", i, kids[i].sub_type, env_sound_class_name (kids[i].klass));
		}
	}

	if (n == 0)
	{
		ENV_EMIT ("none");
	}

	return 0;
}

static int env_sound_flags_set (int terrain_type)
{
	return terrain_types_in_sector[terrain_type] != 0;
}

/* the first sector (over a bounded stride of the loaded grid) whose scratch has each sound-relevant surface set, and the first with none of them: asked of the ORIGINAL sector query at the
   position the routine itself would pass it (the sector centre bounded to the map), with the scratch put back */
static void env_sound_probe_sites (void)
{
	static const int types[ENV_SITE_PROBED] = { -1, TERRAIN_TYPE_SEA, TERRAIN_TYPE_FOREST_TOP, TERRAIN_TYPE_RIVER, TERRAIN_TYPE_RESERVOIR, TERRAIN_TYPE_ALTERED_LAND1, TERRAIN_TYPE_ALTERED_LAND2 };
	int saved[TERRAIN_TYPE_LAST];
	int xmax = terrain_3d_sector_x_max, zmax = terrain_3d_sector_z_max;
	int total, step, k, i;

	memset (env_sound_sites, 0, sizeof (env_sound_sites));
	env_sound_probed = 0;

	if (!terrain_sectors || xmax < 1 || zmax < 1)
	{
		return;
	}

	total = xmax * zmax;
	step = total / ENV_SOUND_PROBES + 1;

	memcpy (saved, terrain_types_in_sector, sizeof (saved));

	for (k = 0; k < total; k += step)
	{
		float x = bound (env_sector_centre (k % xmax), MIN_MAP_X, MAX_MAP_X);
		float z = bound (env_sector_centre (k / xmax), MIN_MAP_Z, MAX_MAP_Z);
		int sx, sz, polygons, any = 0;

		if (env_sector_domain (x, z, &sx, &sz, &polygons) != NULL)
		{
			continue;
		}

		get_terrain_3d_types_in_sector (x, z);
		env_sound_probed++;

		for (i = 1; i < ENV_SITE_PROBED; i++)
		{
			if (env_sound_flags_set (types[i]))
			{
				any = 1;

				if (!env_sound_sites[i].found)
				{
					env_sound_sites[i].found = 1;
					env_sound_sites[i].x = x;
					env_sound_sites[i].z = z;
					env_sound_sites[i].sx = sx;
					env_sound_sites[i].sz = sz;
				}
			}
		}

		if (!any && !env_sound_sites[ENV_SITE_ABSENT].found)
		{
			env_sound_sites[ENV_SITE_ABSENT].found = 1;
			env_sound_sites[ENV_SITE_ABSENT].x = x;
			env_sound_sites[ENV_SITE_ABSENT].z = z;
			env_sound_sites[ENV_SITE_ABSENT].sx = sx;
			env_sound_sites[ENV_SITE_ABSENT].sz = sz;
		}
	}

	memcpy (terrain_types_in_sector, saved, sizeof (saved));
}

/* one case: label, site, camera height, cloud base mode (0 6000, 1 equal to y, 2 just above y, 3 just below y), weather (model 0 global: current, target, transitional state; 1 local ring at
   `ring` of its radius), wind (zone: fraction of the wind radius the camera is inside; negative: radius zero), gust, wind limits, the terrain children's initial amplification, frame delta,
   number of calls, in_cockpit */
static void env_sound_add (const char *label, int site, float alt, int cloud, int model, int cwm, int twm, float ts, float ring, float zone, float gust, float wmin, float wmax, float amp0, float dt,
	int calls, int cockpit)
{
	struct env_sound_case *c;

	if (env_sound_count >= ENV_SOUND_MAX)
	{
		return;
	}

	c = &env_sound_cases[env_sound_count++];
	c->label = label;
	c->site = site;
	c->alt = alt;
	c->cloud = cloud;
	c->model = model;
	c->cwm = cwm;
	c->twm = twm;
	c->ts = ts;
	c->ring = ring;
	c->zone = zone;
	c->gust = gust;
	c->wmin = wmin;
	c->wmax = wmax;
	c->amp0 = amp0;
	c->dt = dt;
	c->calls = calls;
	c->cockpit = cockpit;
}

static void env_sound_build (void)
{
	const float below_180 = nextafterf (180.0f, 0.0f), below_280 = nextafterf (280.0f, 0.0f);
	const float mn = DEFAULT_MIN_WIND_SPEED, mx = DEFAULT_MAX_WIND_SPEED;

	if (env_sound_built)
	{
		return;
	}

	env_sound_built = 1;
	env_sound_count = 0;
	env_sound_probe_sites ();

	/* rain: the weather at the point as per-mode levels (global model: the modes and the transitional state are the inputs; the wind is calm, the terrain absent) */
	env_sound_add ("rain_dry_dry", ENV_SITE_ABSENT, 50.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_light_light_equal_modes", ENV_SITE_ABSENT, 0.0f, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_LIGHT_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_light_to_heavy_quarter", ENV_SITE_ABSENT, 0.0f, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.25f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_heavy_to_light_three_quarters", ENV_SITE_ABSENT, 0.0f, 0, 0, WEATHERMODE_HEAVY_RAIN, WEATHERMODE_LIGHT_RAIN, 0.75f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_snow_to_light", ENV_SITE_ABSENT, 0.0f, 0, 0, WEATHERMODE_SNOW, WEATHERMODE_LIGHT_RAIN, 0.4f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_outdoor_altitude_90", ENV_SITE_ABSENT, 90.0f, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_altitude_just_below_180", ENV_SITE_ABSENT, below_180, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_altitude_exactly_180", ENV_SITE_ABSENT, 180.0f, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_altitude_250", ENV_SITE_ABSENT, 250.0f, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_cloud_base_equal_to_y", ENV_SITE_ABSENT, 50.0f, 1, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_cloud_base_just_above_y", ENV_SITE_ABSENT, 50.0f, 2, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_cloud_base_just_below_y", ENV_SITE_ABSENT, 50.0f, 3, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_local_ring_inner", ENV_SITE_ABSENT, 10.0f, 0, 1, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.0f, 0.15f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_local_ring_middle", ENV_SITE_ABSENT, 10.0f, 0, 1, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.0f, 0.5f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_local_ring_outer", ENV_SITE_ABSENT, 10.0f, 0, 1, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.0f, 0.83f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_local_ring_outside", ENV_SITE_ABSENT, 10.0f, 0, 1, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.0f, 2.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("rain_cockpit_gunship", ENV_SITE_GUNSHIP, 0.0f, 0, 0, WEATHERMODE_LIGHT_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 1);

	/* wind: the speed at the point against the session limits (dry, absent terrain); the camera is `zone` of the way into a wind zone of radius 8192 */
	env_sound_add ("wind_calm_radius_zero", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_at_zone_edge", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 0.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_quarter_into_zone", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 0.25f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_half_into_zone", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 0.5f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_three_quarters_into_zone", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 0.75f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_zone_centre", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_gust_pushes_speed_past_maximum", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 0.75f, 0.125f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_equal_limits", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 1.0f, 0.0f, 10.0f, 10.0f, 0.0f, 0.1f, 1, 0);
	env_sound_add ("wind_zero_minimum_half_into_zone", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, 0.5f, 0.0f, 0.0f, mx, 0.0f, 0.1f, 1, 0);

	/* terrain: the smoothed children (previous value, plus or minus half the frame delta, bound to [0, 0.3], altitude factor, cockpit factor); dry weather, calm wind */
	env_sound_add ("terrain_sea_rising_from_zero", ENV_SITE_SEA, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 1, 0);
	env_sound_add ("terrain_sea_saturating_at_the_upper_bound", ENV_SITE_SEA, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.29f, 0.1f, 1, 0);
	env_sound_add ("terrain_absent_falling", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.2f, 0.1f, 1, 0);
	env_sound_add ("terrain_absent_falling_below_the_lower_bound", ENV_SITE_ABSENT, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.02f, 0.1f, 1, 0);
	env_sound_add ("terrain_forest_top", ENV_SITE_FOREST_TOP, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_river", ENV_SITE_RIVER, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_reservoir", ENV_SITE_RESERVOIR, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_altered_land1", ENV_SITE_ALTERED_LAND1, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_altered_land2", ENV_SITE_ALTERED_LAND2, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_altitude_just_below_280", ENV_SITE_SEA, below_280, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_altitude_exactly_280", ENV_SITE_SEA, 280.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_altitude_400", ENV_SITE_SEA, 400.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("terrain_altitude_140_half_factor", ENV_SITE_SEA, 140.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.2f, 0.1f, 1, 0);
	env_sound_add ("terrain_three_calls_the_scaled_value_is_fed_back", ENV_SITE_SEA, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.0f, 0.1f, 3, 0);
	env_sound_add ("terrain_zero_frame_delta", ENV_SITE_SEA, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.2f, 0.0f, 1, 0);
	env_sound_add ("terrain_one_second_frame_delta", ENV_SITE_SEA, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 1.0f, 1, 0);
	env_sound_add ("terrain_cockpit_gunship", ENV_SITE_GUNSHIP, 0.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.2f, 0.1f, 1, 1);
	env_sound_add ("camera_east_of_the_map_clamped", ENV_SITE_OUTSIDE_HIGH, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);
	env_sound_add ("camera_west_of_the_map_clamped", ENV_SITE_OUTSIDE_LOW, 10.0f, 0, 0, WEATHERMODE_DRY, WEATHERMODE_DRY, 0.5f, 0.0f, -1.0f, 0.0f, mn, mx, 0.1f, 0.1f, 1, 0);

	/* everything at once: heavy rain, wind half into the zone, the sea, two calls */
	env_sound_add ("combined_heavy_rain_wind_sea_two_calls", ENV_SITE_SEA, 20.0f, 0, 0, WEATHERMODE_HEAVY_RAIN, WEATHERMODE_HEAVY_RAIN, 0.5f, 0.0f, 0.5f, 0.0f, mn, mx, 0.1f, 0.1f, 2, 0);
}

/* the original queries the routine consumes at its point, as an input echo (read-only) */
static int env_sound_echo (char *text, int capacity, int *used, int cockpit, vec3d *pos, entity *source)
{
	session *raw = env_session_raw ();
	weathermodes cwm, twm;
	float ts, alt = get_local_entity_float_value (source, FLOAT_TYPE_RADAR_ALTITUDE);

	get_session_weather_at_point (pos, &cwm, &twm, &ts);

	ENV_EMIT (" source=%s x_w=%08x y_w=%08x z_w=%08x alt_w=%08x cwm=%d twm=%d ts_w=%08x", cockpit ? "gunship" : "camera", ENV_WORD (pos->x), ENV_WORD (pos->y), ENV_WORD (pos->z), ENV_WORD (alt), (int) cwm, (int) twm,
		ENV_WORD (ts));

	if (env_gust_in_domain (raw->wind_gusting_value))
	{
		float speed = get_session_wind_velocity_at_point (pos, NULL);
		float range = get_approx_2d_range (pos, &raw->wind_effect_position);

		ENV_EMIT (" wind_speed_w=%08x wind_range_w=%08x", ENV_WORD (speed), ENV_WORD (range));
	}
	else
	{
		ENV_EMIT (" wind_speed=unavailable_gust_outside_0_1");
	}

	return 0;
}

/* the mutation and the report of one case; the caller has saved everything and restores it whatever this returns */
static int env_sound_body (const struct env_sound_case *c, int index, struct env_sound_child *kids, int n, char *text, int capacity, int *used)
{
	struct CAMERA *craw = (struct CAMERA *) get_local_entity_data (get_camera_entity ());
	session *raw = env_session_raw ();
	session before_raw;
	vec3d *cp = env_camera_position ();
	vec3d *source_pos;
	entity *source;
	float y, base;
	int i, call, rng_first, scratch_flags_before[TERRAIN_TYPE_LAST];
	char flags[TERRAIN_TYPE_LAST + 1];

	set_global_season (SESSION_SEASON_SUMMER);
	in_cockpit = c->cockpit;

	if (c->site != ENV_SITE_GUNSHIP)
	{
		if (c->site == ENV_SITE_OUTSIDE_HIGH)
		{
			cp->x = MAX_MAP_X + 5000.0f;
			cp->z = MID_MAP_Z;
		}
		else if (c->site == ENV_SITE_OUTSIDE_LOW)
		{
			cp->x = MIN_MAP_X - 5000.0f;
			cp->z = MID_MAP_Z;
		}
		else
		{
			cp->x = env_sound_sites[c->site].x;
			cp->z = env_sound_sites[c->site].z;
		}

		y = c->alt;
		cp->y = y;
		craw->terrain_info.last_elevation = 0.0f;
		base = (c->cloud == 0) ? ENV_SOUND_CLOUD_BASE : ((c->cloud == 1) ? y : ((c->cloud == 2) ? nextafterf (y, HUGE_VALF) : nextafterf (y, -HUGE_VALF)));
		cloud_3d_base_height = base;
		source = get_camera_entity ();
		source_pos = cp;
	}
	else
	{
		cloud_3d_base_height = ENV_SOUND_CLOUD_BASE;
		source = get_gunship_entity ();
		source_pos = get_local_entity_vec3d_ptr (source, VEC3D_TYPE_POSITION);
	}

	/* the session: the weather and the wind this case names */
	if (c->model == 0)
	{
		raw->local_weather_model = 0;
		raw->weather_radius = 0.0f;
		env_set_vec (&raw->weather_position, 0.0f, 0.0f, 0.0f);
	}
	else
	{
		raw->local_weather_model = 1;
		raw->weather_radius = ENV_SOUND_RING_RADIUS;
		env_set_vec (&raw->weather_position, source_pos->x + c->ring * ENV_SOUND_RING_RADIUS, 0.0f, source_pos->z);
	}

	raw->weather_mode = (weathermodes) c->cwm;
	raw->target_weather_mode = (weathermodes) c->twm;
	raw->weather_mode_transitional_status = c->ts;
	raw->wind_minimum_speed = c->wmin;
	raw->wind_maximum_speed = c->wmax;
	raw->wind_gusting_value = c->gust;
	env_set_vec (&raw->wind_direction_vector, 0.0f, 0.0f, 1.0f);

	if (c->zone < 0.0f)
	{
		raw->wind_effect_radius = 0.0f;
		env_set_vec (&raw->wind_effect_position, 0.0f, 0.0f, 0.0f);
	}
	else
	{
		raw->wind_effect_radius = ENV_SOUND_WIND_RADIUS;
		env_set_vec (&raw->wind_effect_position, source_pos->x + ENV_SOUND_WIND_RADIUS * (1.0f - c->zone), 0.0f, source_pos->z);
	}

	system_delta_time = c->dt;

	/* the children: the terrain children start at the case's value, the rain and wind children at a sentinel (to show that they are overwritten); any other child is left alone */
	for (i = 0; i < n; i++)
	{
		if (kids[i].is_sound && kids[i].klass >= 1)
		{
			set_local_entity_float_value (kids[i].en, FLOAT_TYPE_AMPLIFICATION, (kids[i].klass >= 5) ? c->amp0 : ENV_SOUND_SENTINEL);
		}
	}

	env_push ();
	memcpy (&before_raw, raw, sizeof (before_raw));

	if (env_sound_children (kids, ENV_SOUND_CHILDREN) != n)
	{
		return -1;
	}

	ENV_EMIT ("env-sound kind=original i=%d label=%s site=%d site_found=%d site_sector_x=%d site_sector_z=%d cockpit=%d calls=%d want_alt_w=%08x want_zone_w=%08x gust_w=%08x min_w=%08x max_w=%08x amp0_w=%08x delta_w=%08x"
		" cloud_mode=%d cloud_base_w=%08x model=%s ring_w=%08x in_cwm=%d in_twm=%d in_ts_w=%08x rng=%d", index, c->label, c->site, (c->site < ENV_SITE_PROBED) ? env_sound_sites[c->site].found : -1,
		(c->site < ENV_SITE_PROBED) ? env_sound_sites[c->site].sx : -1, (c->site < ENV_SITE_PROBED) ? env_sound_sites[c->site].sz : -1, c->cockpit, c->calls, ENV_WORD (c->alt), ENV_WORD (c->zone), ENV_WORD (c->gust),
		ENV_WORD (c->wmin), ENV_WORD (c->wmax), ENV_WORD (c->amp0), ENV_WORD (c->dt), c->cloud, ENV_WORD (cloud_3d_base_height), c->model ? "local" : "global", ENV_WORD (c->ring), c->cwm, c->twm, ENV_WORD (c->ts),
		random_number_seed);

	if (env_sound_echo (text, capacity, used, c->cockpit, source_pos, source) != 0)
	{
		return -1;
	}

	if (env_sound_print_children (text, capacity, used, "children_before", kids, n) != 0)
	{
		return -1;
	}

	ENV_EMIT ("\n");

	rng_first = random_number_seed;

	for (call = 1; call <= c->calls; call++)
	{
		update_session_sound_effects (get_session_entity ());

		memcpy (scratch_flags_before, terrain_types_in_sector, sizeof (scratch_flags_before));
		env_sector_text (scratch_flags_before, flags);

		if (env_sound_children (kids, ENV_SOUND_CHILDREN) != n)
		{
			return -1;
		}

		ENV_EMIT ("env-sound-call i=%d call=%d rng=%d rng_unchanged=%d scratch_flags=%s", index, call, random_number_seed, random_number_seed == rng_first, flags);

		if (env_sound_print_children (text, capacity, used, "children", kids, n) != 0)
		{
			return -1;
		}

		ENV_EMIT ("\n");
	}

	env_pull ();

	ENV_EMIT ("env-sound-session i=%d session_unchanged_by_the_routine=%d\n", index, memcmp (&before_raw, raw, sizeof (before_raw)) == 0);

	return 0;
}

static int env_sound_case (int index, char *text, int capacity, int *used)
{
	const struct env_sound_case *c;
	struct env_saved_state saved;
	struct env_sound_child kids[ENV_SOUND_CHILDREN], original[ENV_SOUND_CHILDREN], after[ENV_SOUND_CHILDREN];
	struct CAMERA *craw;
	int scratch_saved[TERRAIN_TYPE_LAST];
	vec3d camera_saved;
	float elevation_saved, cloud_saved;
	int cockpit_saved, n, m, i, rc, children_restored, scratch_restored, camera_restored, cloud_restored, globals_restored;
	uint64_t before_fp, after_fp;

	env_sound_build ();

	n = get_camera_entity () ? env_sound_children (original, ENV_SOUND_CHILDREN) : -2;

	if (index == 0)
	{
		int s;

		ENV_EMIT ("env-sound-plan version=1 cases=%d camera=%d gunship=%d in_cockpit=%d comms_model=%d children=%d complete=%d cloud_base_w=%08x probes=%d probe_limit=%d terrain_loaded=%d sector_x_max=%d sector_z_max=%d"
			" map_min_x_w=%08x map_max_x_w=%08x map_min_z_w=%08x map_max_z_w=%08x delta_w=%08x terrain_types=%d rain_zero_altitude=180 terrain_zero_altitude=280\n", env_sound_count, get_camera_entity () ? 1 : 0, get_gunship_entity () ? 1 : 0,
			(int) in_cockpit, (int) get_comms_model (), n, (n > 0) ? env_sound_complete (original, n) : 0, ENV_WORD (cloud_3d_base_height), env_sound_probed, ENV_SOUND_PROBES, terrain_sectors ? 1 : 0, terrain_3d_sector_x_max,
			terrain_3d_sector_z_max, ENV_WORD (MIN_MAP_X), ENV_WORD (MAX_MAP_X), ENV_WORD (MIN_MAP_Z), ENV_WORD (MAX_MAP_Z), ENV_WORD (get_delta_time ()), (int) TERRAIN_TYPE_LAST);

		for (s = 0; s < ENV_SITE_PROBED; s++)
		{
			ENV_EMIT ("env-sound-site site=%d found=%d x_w=%08x z_w=%08x sector_x=%d sector_z=%d\n", s, env_sound_sites[s].found, ENV_WORD (env_sound_sites[s].x), ENV_WORD (env_sound_sites[s].z), env_sound_sites[s].sx,
				env_sound_sites[s].sz);
		}

		ENV_EMIT ("env-sound-children count=%d", n);

		if (n >= 0 && env_sound_print_children (text, capacity, used, "order", original, n) != 0)
		{
			return -1;
		}

		ENV_EMIT ("\n");
		ENV_EMIT ("env-sound-unsupported item=speaker_output_and_the_sound_effect_update reason=the_headless_build_has_no_sound_system_the_children_amplification_is_the_observable\n");
		ENV_EMIT ("env-sound-unsupported item=terrain_altered_land3_reader_3drain reason=render_code_dormant_in_the_headless_campaign_not_inferred\n");
		ENV_EMIT ("env-sound-unsupported item=non_ambient_special_effect_children reason=none_exists_at_boot_lightning_strike_sounds_appear_only_after_a_strike\n");
		ENV_EMIT ("env-sound-unsupported item=cockpit_branch_without_a_gunship reason=the_routine_reads_the_gunship_entity_when_in_cockpit_and_the_dedicated_server_may_hold_none\n");

		return 0;
	}

	if (index < 1 || index > env_sound_count)
	{
		return -1;
	}

	c = &env_sound_cases[index - 1];

	if (n < 0 || !get_camera_entity ())
	{
		ENV_EMIT ("env-sound kind=original i=%d label=%s refused=%s\n", index, c->label, n == -2 ? "no_camera_entity" : "more_special_effect_children_than_the_fixture_table_holds");
		ENV_EMIT ("env-sound-end i=%d restored=1 nothing_changed=1\n", index);

		return 0;
	}

	if (!env_sound_complete (original, n))
	{
		ENV_EMIT ("env-sound kind=original i=%d label=%s refused=the_session_does_not_hold_each_of_the_ten_ambient_sound_children_exactly_once children=%d\n", index, c->label, n);
		ENV_EMIT ("env-sound-end i=%d restored=1 nothing_changed=1\n", index);

		return 0;
	}

	if (c->site < ENV_SITE_PROBED && !env_sound_sites[c->site].found)
	{
		ENV_EMIT ("env-sound kind=original i=%d label=%s site=%d refused=no_sector_with_that_surface_found_by_the_bounded_probe probed=%d\n", index, c->label, c->site, env_sound_probed);
		ENV_EMIT ("env-sound-end i=%d restored=1 nothing_changed=1\n", index);

		return 0;
	}

	if (c->cockpit && !get_gunship_entity ())
	{
		ENV_EMIT ("env-sound kind=original i=%d label=%s site=%d refused=no_gunship_entity_for_the_cockpit_branch\n", index, c->label, c->site);
		ENV_EMIT ("env-sound-end i=%d restored=1 nothing_changed=1\n", index);

		return 0;
	}

	if (!env_gust_in_domain (c->gust))
	{
		ENV_EMIT ("env-sound kind=original i=%d label=%s refused=gust_outside_0_1\n", index, c->label);
		ENV_EMIT ("env-sound-end i=%d restored=1 nothing_changed=1\n", index);

		return 0;
	}

	/* everything the case may touch is saved here and restored below, whatever the body returns */
	craw = (struct CAMERA *) get_local_entity_data (get_camera_entity ());
	before_fp = env_state_fingerprint ();
	env_save_state (&saved);
	memcpy (scratch_saved, terrain_types_in_sector, sizeof (scratch_saved));
	camera_saved = *env_camera_position ();
	elevation_saved = craw->terrain_info.last_elevation;
	cloud_saved = cloud_3d_base_height;
	cockpit_saved = in_cockpit;
	memcpy (kids, original, sizeof (kids));

	rc = env_sound_body (c, index, kids, n, text, capacity, used);

	for (i = 0; i < n; i++)
	{
		if (original[i].is_sound)
		{
			set_local_entity_float_value (original[i].en, FLOAT_TYPE_AMPLIFICATION, original[i].amp);
		}
	}

	memcpy (terrain_types_in_sector, scratch_saved, sizeof (scratch_saved));
	*env_camera_position () = camera_saved;
	craw->terrain_info.last_elevation = elevation_saved;
	cloud_3d_base_height = cloud_saved;
	in_cockpit = cockpit_saved;
	env_restore_state (&saved);

	after_fp = env_state_fingerprint ();
	m = env_sound_children (after, ENV_SOUND_CHILDREN);
	children_restored = (m == n);

	for (i = 0; children_restored && i < n; i++)
	{
		children_restored = after[i].en == original[i].en && after[i].sub_type == original[i].sub_type && float_bits (after[i].amp) == float_bits (original[i].amp);
	}

	scratch_restored = memcmp (terrain_types_in_sector, scratch_saved, sizeof (scratch_saved)) == 0;
	camera_restored = memcmp (env_camera_position (), &camera_saved, sizeof (camera_saved)) == 0 && float_bits (craw->terrain_info.last_elevation) == float_bits (elevation_saved);
	cloud_restored = float_bits (cloud_3d_base_height) == float_bits (cloud_saved);
	globals_restored = in_cockpit == cockpit_saved;

	if (rc != 0)
	{
		return -1;
	}

	ENV_EMIT ("env-sound-end i=%d restored=%d fp_before=%016llx fp_after=%016llx children_restored=%d scratch_restored=%d camera_restored=%d cloud_restored=%d in_cockpit_restored=%d\n", index,
		before_fp == after_fp && children_restored && scratch_restored && camera_restored && cloud_restored && globals_restored, (unsigned long long) before_fp, (unsigned long long) after_fp, children_restored,
		scratch_restored, camera_restored, cloud_restored, globals_restored);

	return 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* group 12 (EEDCS-51, D3; EXPLICIT ONLY, never part of the default run, the census, the 164-case oracle or group 11): a WIDE session child list.

   update_session_sound_effects walks the session's WHOLE special-effect child list. The campaign attaches ten ambient sound children at boot, but nothing in the game limits how many terrain
   ambient sound children a session may hold: create_local_sound_effect_entity attaches the supplied subtype to the supplied parent with no count check, and the compiled comms receiver
   (se_msgs.c) creates a received child through the same path. This group makes that real: it creates ADDITIONAL terrain ambient sound children with the REAL production constructor, exactly as
   attach_session_sound_effects does (the same arguments, the same sample indices), cycling the six terrain classes so that duplicates of every class exist, gives every terrain child its own DISTINCT
   previous amplitude (j + 1) / 128 in list order, and calls the REAL unchanged routine once with the camera at radar altitude 140 (real camera height 140 over its cached elevation 0: the altitude factor 1 - 140 / 280 is EXACTLY 0.5) outdoors and a frame delta of 0, so that a PROCESSED child holds exactly half its previous amplitude and a skipped one does not.

   Case i creates children until the session holds the case's number of terrain ambient children (16, 17 and 20 -- the boundary of a transport bound a host protocol may have, one past it, and well
   past it; the natural six count), so the cases are to be run IN ORDER in ONE fresh process. It prints the child list (position in the list, entity index, subtype, class, amplification word) before and
   after the call. It computes and judges nothing: the driver compares what came back with what went in.

   NOT A WITNESS, NOT RESTORED: creating entities changes the entity heap and the session's child list, and they cannot be put back from here (like group 9, the process is the isolation boundary: the report says
   `changed=1 process_isolated=required heap_restored=0`). What IS put back: the session record and the host field EXCEPT the list roots and links the constructor changed (special_effect_root, force_root, update_link: restoring those would detach the new children and corrupt the lists, so they stay as the creation left them and the end row checks the list link by link and its identity with the list the routine ran on), the season, the shared seed, the frame delta, the camera position and cached elevation, the
   cloud base, in_cockpit and the scratch. The children's amplifications are NOT restored. A routine that stops the engine (a fatal error) leaves nothing restored and poisons the process: the case then
   reports no rows at all, the engine's error is the result. Speaker, network delivery, the player and the renderer are not exercised and not claimed. */

#define ENV_WIDE_CASES 3

static const int env_wide_targets[ENV_WIDE_CASES] = { 16, 17, 20 };

/* the six terrain classes of group 11 (sea, jungle, amb1 to amb4): their entity sub types and sample indices, as attach_session_sound_effects uses them */
static const int env_wide_sub_types[6] = { ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_SEA, ENTITY_SUB_TYPE_EFFECT_SOUND_AMBIENT_JUNGLE, ENTITY_SUB_TYPE_EFFECT_SOUND_AMB1, ENTITY_SUB_TYPE_EFFECT_SOUND_AMB2,
	ENTITY_SUB_TYPE_EFFECT_SOUND_AMB3, ENTITY_SUB_TYPE_EFFECT_SOUND_AMB4 };

static const int env_wide_samples[6] = { SOUND_SAMPLE_INDEX_AMBIENT_SEA, SOUND_SAMPLE_INDEX_AMBIENT_JUNGLE, SOUND_SAMPLE_INDEX_AMB1, SOUND_SAMPLE_INDEX_AMB2, SOUND_SAMPLE_INDEX_AMB3,
	SOUND_SAMPLE_INDEX_AMB4 };

static int env_wide_terrain_count (const struct env_sound_child *kids, int n)
{
	int i, count = 0;

	for (i = 0; i < n; i++)
	{
		if (kids[i].klass >= 5)
		{
			count++;
		}
	}

	return count;
}

/* each of the ten ambient classes at least once (a wide list holds duplicates) */
static int env_wide_natural_present (const struct env_sound_child *kids, int n)
{
	int seen[ENV_SOUND_CLASSES + 1], i;

	memset (seen, 0, sizeof (seen));

	for (i = 0; i < n; i++)
	{
		seen[kids[i].klass]++;
	}

	for (i = 1; i <= ENV_SOUND_CLASSES; i++)
	{
		if (seen[i] < 1)
		{
			return 0;
		}
	}

	return 1;
}

static int env_wide_print (char *text, int capacity, int *used, const char *key, const struct env_sound_child *kids, int n)
{
	int i;

	ENV_EMIT (" %s=", key);

	for (i = 0; i < n; i++)
	{
		ENV_EMIT ("%s%d:%d:%d:%s:%08x", i ? "," : "", i, (int) get_local_entity_index (kids[i].en), kids[i].sub_type, env_sound_class_name (kids[i].klass), ENV_WORD (kids[i].amp));
	}

	return 0;
}

/* the mutation and the report of one case; the caller restores what it can whatever this returns */
/* The topology of the session's special-effect child list, checked link by link: every child's parent is the session and its predecessor is the child walked before it (the first has none); NULL when it
   holds, else the reason. The real constructor inserts each new child at the HEAD of the list and re-links the old first child, so this is what a cleanup that restored a stale list root would break. */
static const char *env_wide_topology (void)
{
	entity *session_en = get_session_entity (), *spec = get_local_entity_first_child (session_en, LIST_TYPE_SPECIAL_EFFECT), *previous = NULL;
	int walked = 0;

	while (spec)
	{
		if (get_local_entity_parent (spec, LIST_TYPE_SPECIAL_EFFECT) != session_en)
		{
			return "a_child_of_the_list_has_another_parent";
		}

		if (get_local_entity_child_pred (spec, LIST_TYPE_SPECIAL_EFFECT) != previous)
		{
			return "a_child_of_the_list_has_a_predecessor_that_is_not_the_child_walked_before_it";
		}

		if (++walked > 4096)
		{
			return "the_list_does_not_end";
		}

		previous = spec;
		spec = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);
	}

	return NULL;
}

static int env_wide_body (int index, int target, struct env_sound_child *kids, int n, char *text, int capacity, int *used, int *n_post)
{
	struct CAMERA *craw = (struct CAMERA *) get_local_entity_data (get_camera_entity ());
	vec3d *cp = env_camera_position ();
	char ids[ENV_SOUND_CHILDREN * 8];
	const char *topology;
	int created = 0, ids_used = 0, terrain, terrain_before, terrain_index, i, rng_before;

	ids[0] = '\0';
	*n_post = -1;

	terrain = env_wide_terrain_count (kids, n);
	terrain_before = terrain;

	/* the REAL constructor, as attach_session_sound_effects calls it: the session as parent, neutral side, the sound channel, all localities, no position, amplification 0, pitch 1, valid, looping, one sample */
	while (terrain < target)
	{
		int klass = created % 6;
		sound_sample_indices sample = (sound_sample_indices) env_wide_samples[klass];
		entity *en = create_local_sound_effect_entity (ENTITY_INDEX_DONT_CARE, get_session_entity (), ENTITY_SIDE_NEUTRAL, (entity_sub_types) env_wide_sub_types[klass], SOUND_CHANNEL_SOUND_EFFECT,
			SOUND_LOCALITY_ALL, NULL, 0.0, 1.0, TRUE, TRUE, 1, &sample);

		if (!en)
		{
			ENV_EMIT ("env-wide kind=control i=%d refused=the_constructor_returned_no_entity created=%d\n", index, created);

			return 0;
		}

		if (ids_used + 8 < (int) sizeof (ids))
		{
			ids_used += snprintf (ids + ids_used, sizeof (ids) - (size_t) ids_used, "%s%d", created ? "," : "", (int) get_local_entity_index (en));
		}

		created++;
		terrain++;
	}

	n = env_sound_children (kids, ENV_SOUND_CHILDREN);

	if (n < 0 || env_wide_terrain_count (kids, n) != target)
	{
		ENV_EMIT ("env-wide kind=control i=%d refused=the_list_does_not_hold_the_target_after_creating_the_children children=%d created=%d\n", index, n, created);

		return 0;
	}

	topology = env_wide_topology ();

	if (topology)
	{
		ENV_EMIT ("env-wide kind=control i=%d refused=the_list_topology_is_inconsistent_after_creating_the_children reason=%s children=%d created=%d\n", index, topology, n, created);

		return 0;
	}

	*n_post = n;

	/* every terrain child its own previous amplification, in list order; the rain and wind children the sentinel */
	terrain_index = 0;

	for (i = 0; i < n; i++)
	{
		if (kids[i].klass >= 5)
		{
			set_local_entity_float_value (kids[i].en, FLOAT_TYPE_AMPLIFICATION, (float) (terrain_index + 1) / 128.0f);
			terrain_index++;
		}
		else if (kids[i].is_sound)
		{
			set_local_entity_float_value (kids[i].en, FLOAT_TYPE_AMPLIFICATION, ENV_SOUND_SENTINEL);
		}
	}

	/* the controlled inputs: the camera at height 140 over its cached elevation 0, i.e. radar altitude 140 (read back and printed, 430c0000), outdoors, a frame delta of 0 */
	set_global_season (SESSION_SEASON_SUMMER);
	in_cockpit = 0;
	cp->y = 140.0f;
	craw->terrain_info.last_elevation = 0.0f;
	cloud_3d_base_height = ENV_SOUND_CLOUD_BASE;
	system_delta_time = 0.0f;

	if (env_sound_children (kids, ENV_SOUND_CHILDREN) != n)
	{
		return -1;
	}

	ENV_EMIT ("env-wide kind=control i=%d target_terrain=%d terrain_before=%d created=%d children=%d terrain=%d created_ids=%s delta_w=%08x alt_w=%08x cockpit=0", index, target, terrain_before, created, n,
		env_wide_terrain_count (kids, n), created ? ids : "none", ENV_WORD (get_delta_time ()), ENV_WORD (get_local_entity_float_value (get_camera_entity (), FLOAT_TYPE_RADAR_ALTITUDE)));

	if (env_wide_print (text, capacity, used, "before", kids, n) != 0)
	{
		return -1;
	}

	ENV_EMIT ("\n");

	rng_before = random_number_seed;

	update_session_sound_effects (get_session_entity ());

	if (env_sound_children (kids, ENV_SOUND_CHILDREN) != n)
	{
		return -1;
	}

	topology = env_wide_topology ();

	ENV_EMIT ("env-wide-after i=%d rng_unchanged=%d topology=%s", index, random_number_seed == rng_before, topology ? topology : "ok");

	if (env_wide_print (text, capacity, used, "children", kids, n) != 0)
	{
		return -1;
	}

	ENV_EMIT ("\n");

	return 0;
}

static int env_wide_case (int index, char *text, int capacity, int *used)
{
	struct env_saved_state saved;
	struct env_sound_child kids[ENV_SOUND_CHILDREN], after[ENV_SOUND_CHILDREN];
	struct CAMERA *craw;
	int scratch_saved[TERRAIN_TYPE_LAST];
	vec3d camera_saved;
	list_root keep_force, keep_effects;
	list_link keep_update;
	const char *topology;
	float elevation_saved, cloud_saved;
	int cockpit_saved, n, n_post = -1, after_n, preserved, terrain, rc, i, target;

	n = get_camera_entity () ? env_sound_children (kids, ENV_SOUND_CHILDREN) : -2;

	if (index == 0)
	{
		ENV_EMIT ("env-wide-plan version=1 cases=%d camera=%d children=%d natural_present=%d terrain_children=%d targets=", ENV_WIDE_CASES, get_camera_entity () ? 1 : 0, n, (n >= 0) ? env_wide_natural_present (kids, n) : 0,
			(n >= 0) ? env_wide_terrain_count (kids, n) : -1);

		for (i = 0; i < ENV_WIDE_CASES; i++)
		{
			ENV_EMIT ("%s%d", i ? "," : "", env_wide_targets[i]);
		}

		ENV_EMIT (" process_isolated=required\n");
		ENV_EMIT ("env-wide-unsupported item=speaker_network_delivery_player_and_renderer reason=not_exercised_headless_not_claimed\n");

		return 0;
	}

	if (index < 1 || index > ENV_WIDE_CASES)
	{
		return -1;
	}

	target = env_wide_targets[index - 1];

	if (n < 0 || !env_wide_natural_present (kids, n))
	{
		ENV_EMIT ("env-wide kind=control i=%d refused=%s children=%d\n", index, n == -2 ? "no_camera_entity" : "the_session_does_not_hold_the_ambient_children_or_more_than_the_fixture_table_holds", n);
		ENV_EMIT ("env-wide-end i=%d changed=0 process_isolated=required heap_restored=1 created=0\n", index);

		return 0;
	}

	terrain = env_wide_terrain_count (kids, n);

	if (terrain > target)
	{
		ENV_EMIT ("env-wide kind=control i=%d refused=the_session_already_holds_more_terrain_children_than_the_target_and_none_can_be_removed terrain=%d target=%d\n", index, terrain, target);
		ENV_EMIT ("env-wide-end i=%d changed=0 process_isolated=required heap_restored=1 created=0\n", index);

		return 0;
	}

	craw = (struct CAMERA *) get_local_entity_data (get_camera_entity ());
	env_save_state (&saved);
	memcpy (scratch_saved, terrain_types_in_sector, sizeof (scratch_saved));
	camera_saved = *env_camera_position ();
	elevation_saved = craw->terrain_info.last_elevation;
	cloud_saved = cloud_3d_base_height;
	cockpit_saved = in_cockpit;

	rc = env_wide_body (index, target, kids, n, text, capacity, used, &n_post);

	memcpy (terrain_types_in_sector, scratch_saved, sizeof (scratch_saved));
	*env_camera_position () = camera_saved;
	craw->terrain_info.last_elevation = elevation_saved;
	cloud_3d_base_height = cloud_saved;
	in_cockpit = cockpit_saved;

	/* env_restore_state puts back the WHOLE session record as it was saved BEFORE the children were created. Most of that record is scalar, field and global state, which is what is to be restored; but the
	   record also holds the roots and links of the lists the real constructor changed: it inserted each new sound child at the HEAD of the session's special-effect list (special_effect_root) and into the
	   update list (update_link, and force_root is kept with them as list topology, not state). Restoring those would detach the new children and leave the entity links pointing at a list that no longer
	   reaches them. They are therefore kept as the creation left them -- the children, the list and the heap are NOT restored, and the report says so -- and everything else is put back. */
	keep_force = env_session_raw ()->force_root;
	keep_effects = env_session_raw ()->special_effect_root;
	keep_update = env_session_raw ()->update_link;
	env_restore_state (&saved);
	env_session_raw ()->force_root = keep_force;
	env_session_raw ()->special_effect_root = keep_effects;
	env_session_raw ()->update_link = keep_update;

	if (rc != 0)
	{
		return -1;
	}

	/* after the cleanup: the list is the list the routine ran on (the same children, in the same places, checked link by link), so the next case starts from what this one left */
	after_n = env_sound_children (after, ENV_SOUND_CHILDREN);
	preserved = (n_post >= 0) && (after_n == n_post);

	for (i = 0; preserved && i < n_post; i++)
	{
		preserved = after[i].en == kids[i].en;
	}

	topology = env_wide_topology ();

	ENV_EMIT ("env-wide-end i=%d changed=1 process_isolated=required heap_restored=0 amplitudes_restored=0 session_camera_cloud_cockpit_scratch_restored=1 list_topology_after_cleanup=%s list_preserved=%d\n", index,
		topology ? topology : "ok", preserved);

	return 0;
}

#ifdef EECH_ENV_OBSERVER

/*
 * GROUP 13 (explicit only, never in the default run, the census or the witness set; needs the env-observer build, which holds the read-only accessor to the strike queue): the lightning strike QUEUE
 * (modules/3d/3dstrike.c) that add_3d_lightning_strike writes and update_3d_lightning_strikes ages. TEST-ONLY, SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED.
 *
 *   index 0     the plan and the natural report: the state of the queue now, and the records of the lightning queue probe (ss_envfx.h ENV_FXQ, csrc/eech_env_observer.c) stored while the ORIGINAL
 *               lightning routine ran in the frames. Read-only.
 *   index 1..4  PRIMITIVE CONTROLS of add_3d_lightning_strike and update_3d_lightning_strikes themselves, PROCESS-ISOLATED (one case per process, from a queue that has not been written): the queue
 *               cannot be put back, so a case refuses if the queue is not empty and says changed=1. They call the two original primitives with the active environment set by plain assignment
 *               (restored afterwards); they are NOT the lightning routine, NOT a missing-host proof and do not exercise the routine's own switch to the main environment (the natural run does that):
 *                 1 add_without_active_environment      active environment NULL: does the add queue anything?
 *                 2 add_with_the_main_environment       active environment the main one: the slot, the position, the type, the start time (the main environment's time of day) and the length
 *                 3 fill_the_queue_and_drop_the_next    one add more than the queue has slots for: what the last add does
 *                 4 age_a_cloud_burst_by_the_time_of_day a CLOUD_BURST aged by update_3d_lightning_strikes at three times of day (the forked types read texture animations the headless build may not
 *                                                       initialise and are NOT exercised, said below)
 * A prerequisite that is absent (no main environment, a queue that is not empty) is REFUSED and printed; nothing is forced.
 * In the hosted build (EECH_CAMPAIGN_HOST_ENV) the two primitives are the campaign's requests to the HOST's queue (ss_hostenv.h LIGHTNING_ADD / LIGHTNING_UPDATE) and the queue the rows show is the campaign's read-only MIRROR
 * of the answer; the rows, their order and their grammar are the same as the original's.
 */

extern int eech_lightning_queue_view (int slot, unsigned int *words);
extern unsigned int eech_lightning_update_calls;
extern int eech_envobs_queue_report (char *text, int capacity, int *used);

#define ENV_LQ_CASES 4

static int env_lq_mask (unsigned int *mask, unsigned int *occupancy)
{
	unsigned int words[8];
	int slot, slots = eech_lightning_queue_view (0, words);

	*mask = 0;
	*occupancy = 0;

	if (slots < 1 || slots > 32)
	{
		return -1;
	}

	for (slot = 0; slot < slots; slot++)
	{
		eech_lightning_queue_view (slot, words);

		if (words[0])
		{
			*mask |= ((unsigned int) 1) << slot;
			(*occupancy)++;
		}
	}

	return slots;
}

static int env_lq_active_state (void)
{
	return (active_3d_environment == NULL) ? 0 : ((active_3d_environment == main_3d_env) ? 1 : 2);
}

static int env_lq_slot (char *text, int capacity, int *used, const char *tag, int slot)
{
	unsigned int w[8];

	if (eech_lightning_queue_view (slot, w) < 0)
	{
		return -1;
	}

	ENV_EMIT ("env-lightning-slot tag=%s slot=%d w0=%08x w1=%08x w2=%08x w3=%08x w4=%08x w5=%08x w6=%08x w7=%08x\n", tag, slot, w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]);

	return 0;
}

static int env_lq_slots_in_use (char *text, int capacity, int *used, const char *tag, int slots)
{
	unsigned int w[8];
	int slot;

	for (slot = 0; slot < slots; slot++)
	{
		eech_lightning_queue_view (slot, w);

		if (w[0] && env_lq_slot (text, capacity, used, tag, slot) != 0)
		{
			return -1;
		}
	}

	return 0;
}

static int env_lightning_case (int index, char *text, int capacity, int *used)
{
	static const char *const names[ENV_LQ_CASES] = { "add_without_active_environment", "add_with_the_main_environment", "fill_the_queue_and_drop_the_next", "age_a_cloud_burst_by_the_time_of_day" };
	unsigned int mask_before, occupancy_before, mask_after, occupancy_after;
	env_3d *active_saved = active_3d_environment;
	const char *refusal = NULL;
	float tod_saved = 0.0f;
	int slots, i, first = -1;

	slots = env_lq_mask (&mask_before, &occupancy_before);

	if (index == 0)
	{
		ENV_EMIT ("env-lightning-plan version=1 cases=%d main_present=%d active=%d queue_capacity=%d mask_w=%08x update_calls=%u process_isolated=required\n", ENV_LQ_CASES, main_3d_env != NULL, env_lq_active_state (), slots,
			mask_before, eech_lightning_update_calls);
		ENV_EMIT ("env-lightning-unsupported item=forked_strike_update reason=update_3d_lightning_strikes_reads_texture_animations_the_headless_build_may_not_initialise_not_exercised\n");
		ENV_EMIT ("env-lightning-unsupported item=drawing_and_the_application_scene_update reason=not_exercised_headless_not_claimed\n");

		return eech_envobs_queue_report (text, capacity, used);
	}

	if (index < 1 || index > ENV_LQ_CASES)
	{
		return -1;
	}

	ENV_EMIT ("env-lightning kind=control i=%d label=%s main_present=%d active_before=%d queue_capacity=%d mask_before_w=%08x occupancy_before=%u update_calls_before=%u\n", index, names[index - 1], main_3d_env != NULL, env_lq_active_state (), slots,
		mask_before, occupancy_before, eech_lightning_update_calls);

	if (slots < 1)
	{
		refusal = "the_queue_accessor_does_not_answer";
	}
	else if (mask_before != 0)
	{
		refusal = "the_queue_is_not_empty_a_control_needs_a_process_that_has_not_queued_a_strike";
	}
	else if (index >= 2 && main_3d_env == NULL)
	{
		refusal = "there_is_no_main_3d_environment_nothing_is_forced";
	}

	if (refusal)
	{
		ENV_EMIT ("env-lightning-end i=%d refused=%s changed=0 process_isolated=required\n", index, refusal);

		return 0;
	}

	switch (index)
	{
		case 1:
		{
			active_3d_environment = NULL;
			add_3d_lightning_strike (LIGHTNING_TYPE_CLOUD_BURST, 0.3f, 1500.0f, 2500.0f);
			break;
		}
		case 2:
		{
			active_3d_environment = main_3d_env;
			ENV_EMIT ("env-lightning-input tag=add main_time_w=%08x type=%d length_w=%08x x_w=%08x z_w=%08x\n", ENV_WORD (main_3d_env->time_of_day), (int) LIGHTNING_TYPE_CLOUD_BURST, ENV_WORD (0.3f), ENV_WORD (1500.0f),
				ENV_WORD (2500.0f));
			add_3d_lightning_strike (LIGHTNING_TYPE_CLOUD_BURST, 0.3f, 1500.0f, 2500.0f);
			break;
		}
		case 3:
		{
			active_3d_environment = main_3d_env;

			/* one add more than the queue has slots for; the step rows are the last add that fits and the one that cannot */
			for (i = 0; i <= slots; i++)
			{
				add_3d_lightning_strike (LIGHTNING_TYPE_CLOUD_BURST, 0.3f, (float) (i + 1), 2500.0f);

				if (i >= slots - 1)
				{
					env_lq_mask (&mask_after, &occupancy_after);
					ENV_EMIT ("env-lightning-step i=%d add=%d mask_w=%08x occupancy=%u\n", index, i + 1, mask_after, occupancy_after);
				}
			}

			break;
		}
		default:
		{
			tod_saved = main_3d_env->time_of_day;
			active_3d_environment = main_3d_env;

			set_3d_time_of_day (main_3d_env, 100.0f);
			add_3d_lightning_strike (LIGHTNING_TYPE_CLOUD_BURST, 0.3f, 1500.0f, 2500.0f);
			env_lq_mask (&mask_after, &occupancy_after);

			for (i = 0; i < slots; i++)
			{
				if (mask_after & (((unsigned int) 1) << i))
				{
					first = i;

					break;
				}
			}

			ENV_EMIT ("env-lightning-input tag=age added_at_w=%08x length_w=%08x inside_w=%08x after_the_end_w=%08x first_slot=%d\n", ENV_WORD (100.0f), ENV_WORD (0.3f), ENV_WORD (100.1f), ENV_WORD (100.5f), first);

			if (first >= 0 && env_lq_slot (text, capacity, used, "added", first) != 0)
			{
				return -1;
			}

			set_3d_time_of_day (main_3d_env, 100.1f);
			update_3d_lightning_strikes (main_3d_env);

			if (first >= 0 && env_lq_slot (text, capacity, used, "aged_inside_the_length", first) != 0)
			{
				return -1;
			}

			set_3d_time_of_day (main_3d_env, 100.5f);
			update_3d_lightning_strikes (main_3d_env);

			if (first >= 0 && env_lq_slot (text, capacity, used, "aged_after_the_end", first) != 0)
			{
				return -1;
			}

			set_3d_time_of_day (main_3d_env, tod_saved);
			break;
		}
	}

	active_3d_environment = active_saved;
	env_lq_mask (&mask_after, &occupancy_after);

	if (env_lq_slots_in_use (text, capacity, used, "after", slots) != 0)
	{
		return -1;
	}

	ENV_EMIT ("env-lightning-end i=%d changed=1 mask_after_w=%08x occupancy_after=%u update_calls_after=%u active_restored=%d queue_restored=0 process_isolated=required\n", index, mask_after, occupancy_after,
		eech_lightning_update_calls, active_3d_environment == active_saved);

	return 0;
}

#endif

static int env_case_count (int group)
{
	switch (group)
	{
		case ENV_GROUP_WEATHER: return env_weather_count;
		case ENV_GROUP_WIND: return env_wind_count;
		case ENV_GROUP_GUST: return env_gust_count;
		case ENV_GROUP_TIME: return env_time_count;
		case ENV_GROUP_INIT: return ENV_INIT_COUNT;
		case ENV_GROUP_UPDATE_TIME: return ENV_UPDATE_TIME_COUNT;
		case ENV_GROUP_UPDATE_WEATHER: return ENV_UPDATE_WEATHER_COUNT;
		default: return 1;
	}
}

/* the entry: group 0 census, 1 retail snapshot (both read-only), 2.. the controlled witnesses (one case per call). Returns 0, or non-zero with a reason line in `text` */
int eech_env_case_run (int group, int index, char *text, int capacity)
{
	int used_value = 0, *used = &used_value;
	struct env_saved_state saved;
	uint64_t before, after;
	int rc;

	text[0] = '\0';
	env_build_tables ();

	if (!get_session_entity ())
	{
		append (text, capacity, used, "env-case-error reason=no_session_entity\n");

		return -1;
	}
	if (group < ENV_GROUP_CENSUS || group > ENV_GROUP_LAST)
	{
		append (text, capacity, used, "env-case-error reason=group_out_of_range group=%d\n", group);

		return -1;
	}
	if (group == ENV_GROUP_CENSUS || group == ENV_GROUP_SNAPSHOT)
	{
		/* read-only: nothing to restore */
		if ((group == ENV_GROUP_CENSUS ? env_census (text, capacity, used) : env_snapshot (text, capacity, used)) != 0)
		{
			text[0] = 0;
			used_value = 0;
			append (text, capacity, used, "env-case-error reason=report_did_not_fit group=%d\n", group);

			return -1;
		}

		return 0;
	}
	if (group == ENV_GROUP_CONTROLLED)
	{
		if (index < 0 || index >= ENV_CONTROLLED_COUNT || env_controlled_case (index, text, capacity, used) != 0)
		{
			text[0] = 0;
			used_value = 0;
			append (text, capacity, used, "env-case-error reason=controlled_case_refused group=9 index=%d\n", index);

			return -1;
		}

		return 0;
	}

	if (group == ENV_GROUP_SECTOR)
	{
		/* explicit only (never part of the default run, the census or the witness set): it reads the loaded sectors and restores the one scratch it writes */
		if (env_sector_case (index, text, capacity, used) != 0)
		{
			text[0] = 0;
			used_value = 0;
			append (text, capacity, used, "env-case-error reason=sector_case_refused group=10 index=%d\n", index);

			return -1;
		}

		return 0;
	}

	if (group == ENV_GROUP_SOUND)
	{
		/* explicit only (never part of the default run, the census or the witness set): the case brackets its own save and restore (env_sound_case), whatever its body returns */
		if (env_sound_case (index, text, capacity, used) != 0)
		{
			text[0] = 0;
			used_value = 0;
			append (text, capacity, used, "env-case-error reason=sound_case_refused_or_report_did_not_fit group=11 index=%d\n", index);

			return -1;
		}

		return 0;
	}

	if (group == ENV_GROUP_WIDE)
	{
		/* explicit only; PROCESS-ISOLATED like group 9: it creates entities, which cannot be put back (see env_wide_case) */
		if (env_wide_case (index, text, capacity, used) != 0)
		{
			text[0] = 0;
			used_value = 0;
			append (text, capacity, used, "env-case-error reason=wide_case_refused_or_report_did_not_fit group=12 index=%d\n", index);

			return -1;
		}

		return 0;
	}

	if (group == ENV_GROUP_LIGHTNING)
	{
		/* explicit only; the cases 1 to 4 are PROCESS-ISOLATED (the strike queue cannot be put back, see env_lightning_case); needs the env-observer build */
#ifdef EECH_ENV_OBSERVER
		if (env_lightning_case (index, text, capacity, used) != 0)
		{
			text[0] = 0;
			used_value = 0;
			append (text, capacity, used, "env-case-error reason=lightning_case_refused_or_report_did_not_fit group=13 index=%d\n", index);

			return -1;
		}

		return 0;
#else
		append (text, capacity, used, "env-case-error reason=lightning_queue_needs_the_env_observer_build group=13 index=%d\n", index);

		return -1;
#endif
	}

	if (index < 0 || index >= env_case_count (group))
	{
		append (text, capacity, used, "env-case-error reason=index_out_of_range group=%d index=%d count=%d\n", group, index, env_case_count (group));

		return -1;
	}

	before = env_state_fingerprint ();
	env_save_state (&saved);
	switch (group)
	{
		case ENV_GROUP_WEATHER: rc = env_weather_case (index, text, capacity, used); break;
		case ENV_GROUP_WIND: rc = env_wind_case (index, text, capacity, used); break;
		case ENV_GROUP_GUST: rc = env_gust_case (index, text, capacity, used); break;
		case ENV_GROUP_TIME: rc = env_time_case (index, text, capacity, used); break;
		case ENV_GROUP_INIT: rc = env_init_case (index, text, capacity, used); break;
		case ENV_GROUP_UPDATE_TIME: rc = env_update_time_case (index, text, capacity, used); break;
		default: rc = env_update_weather_case (index, text, capacity, used); break;
	}
	env_restore_state (&saved);
	after = env_state_fingerprint ();

	if (rc != 0)
	{
		/* the report did not fit: the state is restored and the case is refused, never truncated */
		text[0] = '\0';
		used_value = 0;
		append (text, capacity, used, "env-case-error reason=report_did_not_fit group=%d index=%d restored=%d\n", group, index, before == after);

		return -1;
	}

	if (append (text, capacity, used, "env-witness-end group=%d index=%d restored=%d fp_before=%016llx fp_after=%016llx\n", group, index, before == after, (unsigned long long) before, (unsigned long long) after) != 0)
	{
		return -1;
	}

	return 0;
}

#endif
