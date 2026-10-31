/*
 * The reference host's weather and wind field (EEDCS-51, D3): see env_ref.h.
 *
 * Each kernel below is the original C, transcribed with its types and its expression order: the places in aphavoc/source/entity/special/session/ are named at each.  The
 * float and double mix of the original (a float field times a double constant, a double expression stored in a float field) is kept as it is: with the engine's round-toward-zero
 * environment, set here for every call, it gives the same bits.  Nothing here is a different formulation of the original's numerics.  Where the original asserted, the kernel
 * refuses with a message and changes nothing: every operation works on a copy of the field and commits it only when it has finished.
 */
#include <fenv.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "env_ref.h"

#define TRUE 1
#define FALSE 0

/* modules/3d/3denv.h WEATHERMODES */
#define WEATHERMODE_DRY 1
#define WEATHERMODE_LIGHT_RAIN 2
#define WEATHERMODE_HEAVY_RAIN 3
#define WEATHERMODE_SNOW 4
#define WEATHERMODE_LAST 5

/* modules/maths/constant.h */
#define PI (3.14159265359f)
#define ONE_SECOND (1)
#define SECONDS_IN_A_MINUTE (60)
#define ONE_MINUTE (SECONDS_IN_A_MINUTE)

/* session.h */
#define MIN_WEATHER_RADIUS 5000.0
#define MAX_WEATHER_RADIUS 100000.0
#define WEATHER_EXPANSION_RATE 100.0
#define MIN_WIND_RADIUS 8000.0
#define MAX_WIND_RADIUS 128000.0
#define WIND_EXPANSION_RATE 200.0
#define WIND_GUST_FREQUENCY 0.1
#define WIND_GUSTING_SAMPLE_POINTS 6

/* ss_updt.c (unparenthesised, as the original: LIGHTNING_EFFECT_MINIMUM_TIMER + ((LIGHTNING_EFFECT_MAXIMUM_TIMER - LIGHTNING_EFFECT_MINIMUM_TIMER) * frand1 ())) */
#define LIGHTNING_EFFECT_MINIMUM_TIMER 20.0 * ONE_SECOND
#define LIGHTNING_EFFECT_MAXIMUM_TIMER 4.0 * ONE_MINUTE

/* modules/maths/miscmath.h */
#define bound(VALUE,LOWER,UPPER) ( ( VALUE ) < ( LOWER ) ? ( LOWER ) : ( ( VALUE ) > ( UPPER ) ? ( UPPER ) : ( VALUE ) ) )

/* the field: the order of the floats and the integers of ss_hostenv.h */
#define F_WEATHER_RADIUS 0
#define F_TRANSITIONAL_PERIOD 1
#define F_TRANSITIONAL_STATUS 2
#define F_WEATHER_POSITION_X 3
#define F_WEATHER_POSITION_Y 4
#define F_WEATHER_POSITION_Z 5
#define F_WEATHER_VELOCITY_X 6
#define F_WEATHER_VELOCITY_Y 7
#define F_WEATHER_VELOCITY_Z 8
#define F_WIND_EFFECT_RADIUS 9
#define F_WIND_GUSTING_VALUE 10
#define F_WIND_MINIMUM_SPEED 11
#define F_WIND_MAXIMUM_SPEED 12
#define F_WIND_DIRECTION_X 13
#define F_WIND_DIRECTION_Y 14
#define F_WIND_DIRECTION_Z 15
#define F_WIND_EFFECT_POSITION_X 16
#define F_WIND_EFFECT_POSITION_Y 17
#define F_WIND_EFFECT_POSITION_Z 18
#define F_WIND_EFFECT_VELOCITY_X 19
#define F_WIND_EFFECT_VELOCITY_Y 20
#define F_WIND_EFFECT_VELOCITY_Z 21
#define F_LIGHTNING_TIMER 22
#define N_WEATHER_MODE 0
#define N_TARGET_WEATHER_MODE 1
#define N_LOCAL_WEATHER_MODEL 2
#define N_WEATHER_INCREASING 3
#define N_WIND_INCREASING 4

typedef struct env_state
{
	float f[ENV_REF_FLOATS];
	int n[ENV_REF_INTS];
} env_state;

struct env_ref
{
	env_state state;
	int has_state;
};

#define FP_ENTER fenv_t saved_environment; fegetenv (&saved_environment); fesetround (FE_TOWARDZERO)
#define FP_LEAVE fesetenv (&saved_environment)

static int fail (char *error, size_t error_size, const char *format, ...) __attribute__ ((format (printf, 3, 4)));

static int fail (char *error, size_t error_size, const char *format, ...)
{
	va_list args;

	if (error && error_size)
	{
		va_start (args, format);
		vsnprintf (error, error_size, format, args);
		va_end (args);
	}

	return -1;
}

static int all_finite (const float *values, int count)
{
	int i;

	for (i = 0; i < count; i++)
	{
		if (!isfinite (values[i]))
		{
			return FALSE;
		}
	}

	return TRUE;
}

static int check_state (const env_state *s, char *error, size_t error_size)
{
	int i;

	if (!all_finite (s->f, ENV_REF_FLOATS))
	{
		return fail (error, error_size, "the field has a number that is not finite");
	}

	for (i = N_WEATHER_MODE; i <= N_TARGET_WEATHER_MODE; i++)
	{
		if ((s->n[i] < 0) || (s->n[i] >= WEATHERMODE_LAST))
		{
			return fail (error, error_size, "the weather mode %d is not one of the modes", s->n[i]);
		}
	}

	for (i = N_LOCAL_WEATHER_MODEL; i < ENV_REF_INTS; i++)
	{
		if ((s->n[i] != FALSE) && (s->n[i] != TRUE))
		{
			return fail (error, error_size, "the flag %d is neither 0 nor 1", s->n[i]);
		}
	}

	return 0;
}

env_ref *env_ref_new (void)
{
	return (env_ref *) calloc (1, sizeof (env_ref));
}

void env_ref_free (env_ref *field)
{
	free (field);
}

////////////////////////////////////////
//
// the original arithmetic
//
////////////////////////////////////////

/* modules/maths/range.c get_approx_2d_range */
static float get_approx_2d_range (float ax, float az, float bx, float bz)
{
	float
		dx,
		dz,
		range;

	dx = fabs (ax - bx);
	dz = fabs (az - bz);

	if (dx > dz)
	{
		range = ((dx * 4.0) + dz) * (1.0 / 4.0);
	}
	else
	{
		range = ((dz * 4.0) + dx) * (1.0 / 4.0);
	}

	return (range);
}

/* modules/maths/vector.c normalise_any_3d_vector */
static void normalise_any_3d_vector (float *vector)
{
	float
		x,
		y,
		z,
		length;

	x = ((vector[0]) * (vector[0]));
	y = ((vector[1]) * (vector[1]));
	z = ((vector[2]) * (vector[2]));

	length = sqrt (x + y + z);

	if (length > 0)
	{
		float
			one_over_length;

		one_over_length = 1.0 / length;

		vector[0] = (vector[0] * one_over_length);
		vector[1] = (vector[1] * one_over_length);
		vector[2] = (vector[2] * one_over_length);
	}
	else
	{
		vector[0] = 0;
		vector[1] = 0;
		vector[2] = 0;
	}
}

/* session.c get_session_weather_at_point */
static void weather_at (const env_state *s, float px, float pz, int winter, int *current_weather_mode, int *target_weather_mode, float *transitional_state)
{
	float
		range;

	int
		cwm,
		twm;

	float
		ts;

	if (s->n[N_LOCAL_WEATHER_MODEL])
	{
		if (s->f[F_WEATHER_RADIUS] == 0.0)
		{
			cwm = WEATHERMODE_DRY;
			twm = WEATHERMODE_DRY;

			ts = 1.0;
		}
		else
		{
			range = get_approx_2d_range (px, pz, s->f[F_WEATHER_POSITION_X], s->f[F_WEATHER_POSITION_Z]);

			if (range >= s->f[F_WEATHER_RADIUS])
			{
				cwm = WEATHERMODE_DRY;
				twm = WEATHERMODE_DRY;

				ts = 0.0;
			}
			else if (range >= (s->f[F_WEATHER_RADIUS] * 0.666))
			{
				cwm = WEATHERMODE_DRY;

				if (winter)
					twm = WEATHERMODE_SNOW;
				else
					twm = WEATHERMODE_LIGHT_RAIN;

				ts = (s->f[F_WEATHER_RADIUS] - range) / (s->f[F_WEATHER_RADIUS] * 0.333);
			}
			else if (range >= (s->f[F_WEATHER_RADIUS] * 0.333))
			{
				if (winter)
				{
					cwm = WEATHERMODE_SNOW;
					twm = WEATHERMODE_SNOW;
				}
				else
				{
					cwm = WEATHERMODE_LIGHT_RAIN;
					twm = WEATHERMODE_HEAVY_RAIN;
				}

				ts = ((s->f[F_WEATHER_RADIUS] * 0.666) - range) / (s->f[F_WEATHER_RADIUS] * 0.333);
			}
			else
			{
				if (winter)
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
		cwm = s->n[N_WEATHER_MODE];
		twm = s->n[N_TARGET_WEATHER_MODE];

		ts = s->f[F_TRANSITIONAL_STATUS];
	}

	*current_weather_mode = cwm;
	*target_weather_mode = twm;
	*transitional_state = ts;
}

/* session.c get_wind_gusting_modulation_value: the original asserts that the indices are in range; the phase must be in [0, 1) */
static int get_wind_gusting_modulation_value (float wind_gusting_value, float *result, char *error, size_t error_size)
{
	static const float wind_gust_modulation_amounts[WIND_GUSTING_SAMPLE_POINTS] =
	{
		 0.25,
		-0.05,
		-0.25,
		 0.05,
		-0.01,
		 0.15
	};

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

	if (!(val >= 0.0) || !(val < 8.0))
	{
		return fail (error, error_size, "the gust phase %.9g is outside the original's domain [0, 1)", (double) wind_gusting_value);
	}

	/* the original converts these integral values with the x87 store, which is exact for them; the C conversion is the same value */
	low_index = (int) low;
	high_index = (int) high;

	if (!((low_index >= 0) && (low_index < num_segments)) || !((high_index >= 0) && (high_index <= num_segments)))
	{
		return fail (error, error_size, "the gust phase %.9g is outside the original's domain [0, 1)", (double) wind_gusting_value);
	}

	if (low_index == 0)
	{
		a = 0.0;
	}
	else
	{
		a = wind_gust_modulation_amounts[low_index - 1];
	}

	if ((high_index == 0) || (high_index == num_segments))
	{
		b = 0.0;
	}
	else
	{
		b = wind_gust_modulation_amounts[high_index - 1];
	}

	*result = (a + (d * (b - a)));

	return 0;
}

/* session.c get_session_wind_velocity_at_point */
static int wind_at (const env_state *s, float px, float pz, float *speed_out, float *vector, char *error, size_t error_size)
{
	float
		r,
		range,
		wind_speed,
		modulation;

	if (s->f[F_WIND_EFFECT_RADIUS] == 0.0)
	{
		wind_speed = s->f[F_WIND_MINIMUM_SPEED];
	}
	else
	{
		range = get_approx_2d_range (px, pz, s->f[F_WIND_EFFECT_POSITION_X], s->f[F_WIND_EFFECT_POSITION_Z]);

		if (range >= s->f[F_WIND_EFFECT_RADIUS])
		{
			wind_speed = s->f[F_WIND_MINIMUM_SPEED];
		}
		else
		{
			r = ((s->f[F_WIND_EFFECT_RADIUS] - range) / s->f[F_WIND_EFFECT_RADIUS]);

			wind_speed = s->f[F_WIND_MINIMUM_SPEED] + ((s->f[F_WIND_MAXIMUM_SPEED] - s->f[F_WIND_MINIMUM_SPEED]) * r);
		}
	}

	if (get_wind_gusting_modulation_value (s->f[F_WIND_GUSTING_VALUE], &modulation, error, error_size) != 0)
	{
		return -1;
	}

	wind_speed += (wind_speed * modulation);

	if (vector)
	{
		vector[0] = s->f[F_WIND_DIRECTION_X] * wind_speed;
		vector[1] = 0.0;
		vector[2] = s->f[F_WIND_DIRECTION_Z] * wind_speed;
	}

	*speed_out = wind_speed;

	return 0;
}

////////////////////////////////////////
//
// the operations
//
////////////////////////////////////////

static int import_impl (env_ref *field, const float *f, const int *n, char *error, size_t error_size)
{
	env_state s;

	memcpy (s.f, f, sizeof (s.f));
	memcpy (s.n, n, sizeof (s.n));

	if (check_state (&s, error, error_size) != 0)
	{
		return -1;
	}

	field->state = s;
	field->has_state = TRUE;

	return 0;
}

static int require_state (const env_ref *field, char *error, size_t error_size)
{
	if (!field->has_state)
	{
		return fail (error, error_size, "the field has no state: the campaign imports the session's defaults when it creates the session");
	}

	return 0;
}

/* session.c set_session_random_weather_mode, with the draws the campaign made at its call site */
static int init_random_impl (env_ref *field, const double *draw, const float *config, const float *map, char *error, size_t error_size)
{
	env_state s;

	float
		theta,
		weather_rain_radius_min = config[0],
		weather_rain_radius_max = config[1],
		weather_rain_speed = config[2],
		weather_wind_radius_min = config[3],
		weather_wind_radius_max = config[4],
		weather_wind_speed = config[5],
		MIN_MAP_X = map[0],
		MAX_MAP_X = map[1],
		MIN_MAP_Z = map[2],
		MAX_MAP_Z = map[3];

	int i;

	if (require_state (field, error, error_size) != 0)
	{
		return -1;
	}

	if (!all_finite (config, 6) || !all_finite (map, 6))
	{
		return fail (error, error_size, "the configuration or the map extents have a number that is not finite");
	}

	for (i = 0; i < ENV_REF_DRAWS; i++)
	{
		int flag = (i == 1) || (i == 6);
		int signed_draw = (i == 4) || (i == 9);

		if (!isfinite (draw[i]) || (flag && ((draw[i] < 0.0) || (draw[i] > 32767.0) || (draw[i] != floor (draw[i])))) ||
			(!flag && !signed_draw && ((draw[i] < 0.0) || (draw[i] > 1.0))) || (signed_draw && ((draw[i] < -1.0) || (draw[i] > 1.0))))
		{
			return fail (error, error_size, "draw %d (%.17g) is not a value the original's draw macro gives", i + 1, draw[i]);
		}
	}

	s = field->state;

	s.n[N_LOCAL_WEATHER_MODEL] = TRUE;

	/* Rain */

	s.f[F_WEATHER_RADIUS] = weather_rain_radius_min + ((weather_rain_radius_max - weather_rain_radius_min) * draw[0]);

	s.n[N_WEATHER_INCREASING] = (((int) draw[1]) & 1);

	s.f[F_WEATHER_POSITION_X] = MIN_MAP_X + ((MAX_MAP_X - MIN_MAP_X) * draw[2]);
	s.f[F_WEATHER_POSITION_Y] = 0.0;
	s.f[F_WEATHER_POSITION_Z] = MIN_MAP_Z + ((MAX_MAP_Z - MIN_MAP_Z) * draw[3]);

	theta = PI * draw[4];

	s.f[F_WEATHER_VELOCITY_X] = sin (theta) * weather_rain_speed;
	s.f[F_WEATHER_VELOCITY_Y] = 0.0;
	s.f[F_WEATHER_VELOCITY_Z] = cos (theta) * weather_rain_speed;

	/* Wind */

	s.f[F_WIND_EFFECT_RADIUS] = weather_wind_radius_min + ((weather_wind_radius_max - weather_wind_radius_min) * draw[5]);

	s.n[N_WIND_INCREASING] = (((int) draw[6]) & 1);

	s.f[F_WIND_EFFECT_POSITION_X] = MIN_MAP_X + ((MAX_MAP_X - MIN_MAP_X) * draw[7]);
	s.f[F_WIND_EFFECT_POSITION_Y] = 0.0;
	s.f[F_WIND_EFFECT_POSITION_Z] = MIN_MAP_Z + ((MAX_MAP_Z - MIN_MAP_Z) * draw[8]);

	theta = PI * draw[9];

	s.f[F_WIND_EFFECT_VELOCITY_X] = sin (theta) * weather_wind_speed;
	s.f[F_WIND_EFFECT_VELOCITY_Y] = 0.0;
	s.f[F_WIND_EFFECT_VELOCITY_Z] = cos (theta) * weather_wind_speed;

	if (check_state (&s, error, error_size) != 0)
	{
		return -1;
	}

	field->state = s;

	return 0;
}

/* session.c set_session_fixed_weather_mode */
static int init_fixed_impl (env_ref *field, int weather_mode, char *error, size_t error_size)
{
	env_state s;

	if (require_state (field, error, error_size) != 0)
	{
		return -1;
	}

	if ((weather_mode < 0) || (weather_mode >= WEATHERMODE_LAST))
	{
		return fail (error, error_size, "the weather mode %d is not one of the modes", weather_mode);
	}

	s = field->state;

	s.n[N_LOCAL_WEATHER_MODEL] = FALSE;

	s.n[N_WEATHER_MODE] = weather_mode;
	s.n[N_TARGET_WEATHER_MODE] = weather_mode;

	s.f[F_TRANSITIONAL_STATUS] = 0.0;

	field->state = s;

	return 0;
}

/* the map's side of a bounce: ss_updt.c update_transitional_weather */
static void bounce (float *position, float *velocity, float minimum, float maximum)
{
	if (*position < minimum)
	{
		*position = minimum;

		*velocity = -*velocity;
	}
	else if (*position >= maximum)
	{
		*position = maximum - 1.0;

		*velocity = -*velocity;
	}
}

/* ss_updt.c update_transitional_weather, update_lightning_effect (the timer) */
static int step_impl (env_ref *field, const float *in_f, const int *in_i, double draw, int *out_i, float *out_f, char *error, size_t error_size)
{
	env_state s;

	float
		delta = in_f[0],
		MIN_MAP_X = in_f[4],
		MAX_MAP_X = in_f[5],
		MIN_MAP_Z = in_f[6],
		MAX_MAP_Z = in_f[7],
		MID_MAP_X = in_f[8],
		MID_MAP_Z = in_f[9],
		new_time;

	int
		winter = in_i[0],
		server = in_i[1],
		have_camera = in_i[3],
		have_draw = in_i[4],
		lightning_flag = FALSE;

	float strike[3] = { 0.0, 0.0, 0.0 };

	out_i[0] = FALSE;
	out_i[1] = FALSE;
	out_f[0] = out_f[1] = out_f[2] = out_f[3] = 0.0;

	if (require_state (field, error, error_size) != 0)
	{
		return -1;
	}

	if (!all_finite (in_f, 10) || !(delta >= 0.0))
	{
		return fail (error, error_size, "the delta or the camera or the map extents are not finite numbers (the delta must not be negative)");
	}

	s = field->state;

	/* the gust phase is wrapped by a loop that would not end for a huge value: refuse it, as no original run reaches it */
	if (!(s.f[F_WIND_GUSTING_VALUE] >= 0.0) || !(s.f[F_WIND_GUSTING_VALUE] < 1.0e6))
	{
		return fail (error, error_size, "the gust phase %.9g is outside the range the original's wrap handles", (double) s.f[F_WIND_GUSTING_VALUE]);
	}

	if (s.n[N_LOCAL_WEATHER_MODEL])
	{
		//
		// local weather model
		//

		if (s.f[F_WEATHER_RADIUS] == 0.0)
		{
			//
			// weather is fine all across the map
			//

			s.n[N_WEATHER_MODE] = WEATHERMODE_DRY;
			s.n[N_TARGET_WEATHER_MODE] = WEATHERMODE_DRY;

			s.f[F_TRANSITIONAL_STATUS] = 1.0;
		}
		else
		{
			int
				cwm,
				twm;

			float
				ts;

			//
			// there is some bad weather somewhere on the map
			//

			if (!have_camera)
			{
				return fail (error, error_size, "a weather zone's mode is taken at the camera and the campaign gave no camera position");
			}

			weather_at (&s, in_f[1], in_f[3], winter, &cwm, &twm, &ts);

			s.n[N_WEATHER_MODE] = cwm;
			s.n[N_TARGET_WEATHER_MODE] = twm;
			s.f[F_TRANSITIONAL_STATUS] = ts;

			//
			// lightning effect (update_lightning_effect): the strike is made at the weather centre BEFORE it moves
			//

			strike[0] = s.f[F_WEATHER_POSITION_X];
			strike[1] = s.f[F_WEATHER_POSITION_Y];
			strike[2] = s.f[F_WEATHER_POSITION_Z];

			if (server)
			{
				//
				// SERVER : decrease timer, if timer reaches zero create a strike, and reset timer (c/s)
				//

				s.f[F_LIGHTNING_TIMER] -= delta;

				if (s.f[F_LIGHTNING_TIMER] <= 0.0)
				{
					lightning_flag = TRUE;

					if (!have_draw)
					{
						/* the next timer needs the one draw of the shared random word: nothing has been changed */
						out_i[0] = TRUE;
						out_f[3] = s.f[F_LIGHTNING_TIMER];

						return 0;
					}

					if (!isfinite (draw) || (draw < 0.0) || (draw > 1.0))
					{
						return fail (error, error_size, "the lightning draw (%.17g) is not a value the original's draw macro gives", draw);
					}

					new_time = LIGHTNING_EFFECT_MINIMUM_TIMER + ((LIGHTNING_EFFECT_MAXIMUM_TIMER - LIGHTNING_EFFECT_MINIMUM_TIMER) * draw);

					/* the campaign's setter writes the same value into its mirror */
					s.f[F_LIGHTNING_TIMER] = new_time;

					out_f[3] = new_time;
				}
			}
			else
			{
				//
				// CLIENT : decrease timer only if non-zero and create strike if zero reached ( new timer will be set by server )
				//

				if (s.f[F_LIGHTNING_TIMER] > 0.0)
				{
					s.f[F_LIGHTNING_TIMER] -= delta;

					if (s.f[F_LIGHTNING_TIMER] <= 0.0)
					{
						lightning_flag = TRUE;

						s.f[F_LIGHTNING_TIMER] = 0.0;
					}
				}
			}
		}

		//
		// move weather
		//

		s.f[F_WEATHER_POSITION_X] += (s.f[F_WEATHER_VELOCITY_X] * delta);
		s.f[F_WEATHER_POSITION_Z] += (s.f[F_WEATHER_VELOCITY_Z] * delta);

		//
		// "bounce" weather of sides of map
		//

		bounce (&s.f[F_WEATHER_POSITION_X], &s.f[F_WEATHER_VELOCITY_X], MIN_MAP_X, MAX_MAP_X);
		bounce (&s.f[F_WEATHER_POSITION_Z], &s.f[F_WEATHER_VELOCITY_Z], MIN_MAP_Z, MAX_MAP_Z);

		//
		// expand / contract weather radius
		//

		if (s.n[N_WEATHER_INCREASING])
		{
			s.f[F_WEATHER_RADIUS] += (WEATHER_EXPANSION_RATE * delta);

			if (s.f[F_WEATHER_RADIUS] > MAX_WEATHER_RADIUS)
			{
				s.n[N_WEATHER_INCREASING] = FALSE;

				s.f[F_WEATHER_RADIUS] = MAX_WEATHER_RADIUS;
			}
		}
		else
		{
			s.f[F_WEATHER_RADIUS] -= (WEATHER_EXPANSION_RATE * delta);

			if (s.f[F_WEATHER_RADIUS] < MIN_WEATHER_RADIUS)
			{
				s.n[N_WEATHER_INCREASING] = TRUE;

				s.f[F_WEATHER_RADIUS] = MIN_WEATHER_RADIUS;
			}
		}
	}
	else
	{
		//
		// global weather model
		//

		if (s.n[N_WEATHER_MODE] != s.n[N_TARGET_WEATHER_MODE])
		{
			s.f[F_TRANSITIONAL_STATUS] += delta / s.f[F_TRANSITIONAL_PERIOD];

			if (s.f[F_TRANSITIONAL_STATUS] >= 1.0)
			{
				s.n[N_WEATHER_MODE] = s.n[N_TARGET_WEATHER_MODE];

				s.f[F_TRANSITIONAL_STATUS] = 0.0;
			}
		}
		else
		{
			s.f[F_TRANSITIONAL_STATUS] = 0.0;
		}
	}

	//
	// move wind
	//

	s.f[F_WIND_EFFECT_POSITION_X] += (s.f[F_WIND_EFFECT_VELOCITY_X] * delta);
	s.f[F_WIND_EFFECT_POSITION_Z] += (s.f[F_WIND_EFFECT_VELOCITY_Z] * delta);

	//
	// "bounce" wind_effect of sides of map
	//

	bounce (&s.f[F_WIND_EFFECT_POSITION_X], &s.f[F_WIND_EFFECT_VELOCITY_X], MIN_MAP_X, MAX_MAP_X);
	bounce (&s.f[F_WIND_EFFECT_POSITION_Z], &s.f[F_WIND_EFFECT_VELOCITY_Z], MIN_MAP_Z, MAX_MAP_Z);

	//
	// increase gusting value
	//

	s.f[F_WIND_GUSTING_VALUE] += (delta * WIND_GUST_FREQUENCY);

	while (s.f[F_WIND_GUSTING_VALUE] >= 1.0)
	{
		s.f[F_WIND_GUSTING_VALUE] -= 1.0;
	}

	//
	// expand / contract wind_effect radius
	//

	if (s.n[N_WIND_INCREASING])
	{
		s.f[F_WIND_EFFECT_RADIUS] += (WIND_EXPANSION_RATE * delta);

		if (s.f[F_WIND_EFFECT_RADIUS] > MAX_WIND_RADIUS)
		{
			s.n[N_WIND_INCREASING] = FALSE;

			s.f[F_WIND_EFFECT_RADIUS] = MAX_WIND_RADIUS;
		}
	}
	else
	{
		s.f[F_WIND_EFFECT_RADIUS] -= (WIND_EXPANSION_RATE * delta);

		if (s.f[F_WIND_EFFECT_RADIUS] < MIN_WIND_RADIUS)
		{
			s.n[N_WIND_INCREASING] = TRUE;

			s.f[F_WIND_EFFECT_RADIUS] = MIN_WIND_RADIUS;
		}
	}

	//
	// set wind direction
	//

	s.f[F_WIND_DIRECTION_X] = s.f[F_WIND_EFFECT_POSITION_X] - MID_MAP_X;
	s.f[F_WIND_DIRECTION_Y] = 0.0;
	s.f[F_WIND_DIRECTION_Z] = s.f[F_WIND_EFFECT_POSITION_Z] - MID_MAP_Z;

	normalise_any_3d_vector (&s.f[F_WIND_DIRECTION_X]);

	if (check_state (&s, error, error_size) != 0)
	{
		return -1;
	}

	field->state = s;

	if (lightning_flag)
	{
		out_i[1] = TRUE;

		out_f[0] = strike[0];
		out_f[1] = strike[1];
		out_f[2] = strike[2];

		/* a client's expiry carries no new timer: it was zeroed above */
		if (!server)
		{
			out_f[3] = 0.0;
		}
	}

	return 0;
}

////////////////////////////////////////
//
// the exported operations: each under the complete floating-point environment of its own
//
////////////////////////////////////////

int env_ref_import (env_ref *field, const float *f, const int *n, char *error, size_t error_size)
{
	int result;

	FP_ENTER;
	result = import_impl (field, f, n, error, error_size);
	FP_LEAVE;

	return result;
}

int env_ref_export (const env_ref *field, float *f, int *n, char *error, size_t error_size)
{
	int result = 0;

	FP_ENTER;
	if (require_state (field, error, error_size) != 0)
	{
		result = -1;
	}
	else
	{
		memcpy (f, field->state.f, sizeof (field->state.f));
		memcpy (n, field->state.n, sizeof (field->state.n));
	}
	FP_LEAVE;

	return result;
}

int env_ref_init_random (env_ref *field, const double *draw, const float *config, const float *map, char *error, size_t error_size)
{
	int result;

	FP_ENTER;
	result = init_random_impl (field, draw, config, map, error, error_size);
	FP_LEAVE;

	return result;
}

int env_ref_init_fixed (env_ref *field, int mode, char *error, size_t error_size)
{
	int result;

	FP_ENTER;
	result = init_fixed_impl (field, mode, error, error_size);
	FP_LEAVE;

	return result;
}

int env_ref_weather (const env_ref *field, float x, float z, int winter, int *modes, float *transitional_state, char *error, size_t error_size)
{
	int result = 0;

	FP_ENTER;
	if (require_state (field, error, error_size) != 0)
	{
		result = -1;
	}
	else
	{
		int cwm, twm;
		float ts;

		weather_at (&field->state, x, z, winter, &cwm, &twm, &ts);

		modes[0] = cwm;
		modes[1] = twm;

		/* get_simple_session_weather_at_point */
		modes[2] = (ts < 0.5) ? cwm : twm;

		*transitional_state = ts;
	}
	FP_LEAVE;

	return result;
}

int env_ref_wind (const env_ref *field, float x, float z, int want_vector, float *speed, float *vector, char *error, size_t error_size)
{
	int result = 0;

	FP_ENTER;
	if (require_state (field, error, error_size) != 0)
	{
		result = -1;
	}
	else
	{
		result = wind_at (&field->state, x, z, speed, want_vector ? vector : NULL, error, error_size);
	}
	FP_LEAVE;

	return result;
}

int env_ref_gust (float phase, float *modulation, char *error, size_t error_size)
{
	int result;

	FP_ENTER;
	result = get_wind_gusting_modulation_value (phase, modulation, error, error_size);
	FP_LEAVE;

	return result;
}

int env_ref_step (env_ref *field, const float *in_f, const int *in_i, double draw, int *out_i, float *out_f, char *error, size_t error_size)
{
	int result;

	FP_ENTER;
	result = step_impl (field, in_f, in_i, draw, out_i, out_f, error, error_size);
	FP_LEAVE;

	return result;
}
