/*
 * TEST-ONLY characterisation of the ORIGINAL session environment (EEDCS-51, first increment of D3). Compiled only with the `env-fixture` feature; never part of the shipping module.
 * SOURCE PREPARED, UNBUILT, UNRUN, UNREVIEWED: nothing in this file has been compiled or executed. It moves no physics and adds no campaign behaviour.
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
 * NOT witnessed here, and why (printed in the census as `env-unsupported`):
 *   get_session_random_start_time_of_day  reseeds the shared AND the libc generators from the clock; the libc state cannot be saved or restored from here.
 *   update_lightning_effect               creates strike and sound entities and draws the shared generator; the controlled update cases keep lightning_timer far from zero so it
 *                                         never fires, and its natural firing is seen only through the real frames' `lightning_timer` and `rng` in the retail snapshots.
 *   update_session_sound_effects          runs in every headless frame (a camera entity exists) and only sets sound amplitudes; it also writes the shared terrain_types_in_sector.
 *   update_time_of_day_resync             runs in every headless server frame (comms transmit every minute); observed only as the `time_of_day_resync` counter.
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
	ENV_GROUP_LAST = 9
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
	memcpy (&s->raw, env_session_raw (), sizeof (session));
	s->season = (int) get_global_season ();
	s->rng = random_number_seed;
	s->delta = get_delta_time ();
}

static void env_restore_state (const struct env_saved_state *s)
{
	memcpy (env_session_raw (), &s->raw, sizeof (session));
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
