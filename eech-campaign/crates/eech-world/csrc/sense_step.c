/*
 * The physical sensing of the AI target scans (EEDCS-43, D2): what a source's sensor perceives of a candidate target. It is EECH's own
 * arithmetic, moved here expression for expression from aphavoc/source/entity/mobile/aircraft/ac_tgt.c (air-to-air and air-to-surface scans)
 * and vehicle/vh_tgt.c (surface-to-air and surface-to-surface scans), and from the radar-stealth factor of helicopter/hc_float.c, so the host
 * and the original reference produce the same bits. Neither a rounding mode nor a type is chosen here: the engine runs a host callback under
 * round-toward-zero and every float and double expression below is the engine's own, in its own order.
 *
 * What is kept exactly:
 *   - the range: planar squared (x and z) for the aircraft scans, `max + ((med + min) / 4)` approximate 3-D for the vehicle scans;
 *   - the aircraft scans compare against the squared scan range (squared here, in float, from the raw figure) and the stealth-scaled squared range;
 *   - the surface-to-air floor is lowered in place by `bound (range / scan_range, 0.1, 1.0)` for EVERY candidate the campaign asks about, and by
 *     the human pilot's difficulty factor, and never restored within a scan: the next candidate meets the lowered floor;
 *   - the ceiling is the group's raw ceiling plus the source's own altitude, compared with the target's radar altitude;
 *   - the altitude test comes before the random tolerance, whose limit is `floor * (1.0 + (0.5 * draw))` in double, the draw being the campaign's;
 *   - a switched-off emitter of a human-flown target multiplies the compared range by 1.1 (a double constant) in the order radar, radar jammer,
 *     infra-red jammer;
 *   - the radar-stealth factor: 1 for anything but a helicopter, and for a helicopter 1 less 0.4 when its weapon bay is closed, less 0.35 when its
 *     stub wings are not attached, less 0.1 when its undercarriage is up, each subtraction a float narrowed from a double.
 *
 * What is not here: which targets are candidates, in which order they are asked, the random generator and its seed, the ranking, line of sight
 * and engagement. The unit database figures (scan range, ceiling, floor), the positions, the radar altitude and the helicopter configuration
 * facts are the campaign's mirror of the world, supplied as facts: declared transitional coupling.
 *
 * The state is one open scan. A new `eech_sense_begin` replaces whatever was open; a message that does not fit the state is refused.
 */
#include <stddef.h>

#include "eech_arith.h"

#define NO_RADAR_STEALTH_FACTOR ((float) 1.0)

#define SENSE_AIR_TO_AIR 1
#define SENSE_AIR_TO_SURFACE 2
#define SENSE_SURFACE_TO_AIR 3
#define SENSE_SURFACE_TO_SURFACE 4

/* the states of eech_sense_result: the numbers are the campaign's campaign_sense_state values */
#define SENSE_OPEN 1
#define SENSE_ACCEPT 2
#define SENSE_REJECT 3
#define SENSE_NEED_DRAW 4
#define SENSE_NEED_SIGNATURE 5
#define SENSE_CLOSED 6

#define SENSE_REASON_RANGE 1
#define SENSE_REASON_RANGE_ADJUSTED 2
#define SENSE_REASON_ALTITUDE 3
#define SENSE_REASON_TOLERANCE 4

enum
{
	AWAIT_CANDIDATE,
	AWAIT_DRAW,
	AWAIT_SIGNATURE
};

typedef struct eech_sense_answer
{
	int state;
	int reason;
	int has_range;
	float target_range;
	int has_echo;
	float echo_ceiling;
	float echo_floor;
	float echo_adjusted_range;
	float echo_stealth_factor;
	float echo_stealth_limit;
	double echo_tolerance_limit;
} eech_sense_answer;

typedef struct eech_sense_scan
{
	int open;
	int scan;
	int awaiting;
	vec3d source_position;
	float scan_range;
	float squared_range;
	float ceiling;
	float floor;
	/* the candidate being asked about */
	float target_range;
	float radar_altitude;
} eech_sense_scan;

size_t eech_sense_scan_size(void)
{
	return sizeof(eech_sense_scan);
}

/* maths/range.c */
static float approx_3d_range(const vec3d *v1, const vec3d *v2)
{
	float dx, dy, dz, tmp, range;

	dx = fabs(v1->x - v2->x);
	dy = fabs(v1->y - v2->y);
	dz = fabs(v1->z - v2->z);

	if (dx < dy)
	{
		tmp = dx;
		dx = dy;
		dy = tmp;
	}

	if (dx < dz)
	{
		tmp = dx;
		dx = dz;
		dz = tmp;
	}

	range = dx + ((dy + dz) * (1.0 / 4.0));

	return range;
}

static void reset_answer(eech_sense_answer *out)
{
	memset(out, 0, sizeof(*out));
}

static void echo(const eech_sense_scan *s, eech_sense_answer *out, float adjusted, float stealth, float limit, double tolerance)
{
	out->has_echo = 1;
	out->echo_ceiling = s->ceiling;
	out->echo_floor = s->floor;
	out->echo_adjusted_range = adjusted;
	out->echo_stealth_factor = stealth;
	out->echo_stealth_limit = limit;
	out->echo_tolerance_limit = tolerance;
}

static void reject(eech_sense_scan *s, int reason, eech_sense_answer *out)
{
	s->awaiting = AWAIT_CANDIDATE;
	out->state = SENSE_REJECT;
	out->reason = reason;
}

/* opens a scan; returns 0, or 1 for a scan kind that does not exist */
int eech_sense_begin(void *scan, int kind, const float *source, float scan_range, float scan_ceiling, float scan_floor, eech_sense_answer *out)
{
	eech_sense_scan *s = (eech_sense_scan *) scan;

	reset_answer(out);
	if (kind < SENSE_AIR_TO_AIR || kind > SENSE_SURFACE_TO_SURFACE)
	{
		return 1;
	}
	memset(s, 0, sizeof(*s));
	s->open = 1;
	s->scan = kind;
	s->awaiting = AWAIT_CANDIDATE;
	s->source_position.x = source[0];
	s->source_position.y = source[1];
	s->source_position.z = source[2];
	s->scan_range = scan_range;
	if (kind == SENSE_AIR_TO_AIR || kind == SENSE_AIR_TO_SURFACE)
	{
		/* ac_tgt.c: source_*_scan_range *= source_*_scan_range */
		s->squared_range = scan_range;
		s->squared_range *= s->squared_range;
	}
	if (kind == SENSE_SURFACE_TO_AIR)
	{
		/* vh_tgt.c: source_air_scan_ceiling += source_position->y */
		s->ceiling = scan_ceiling;
		s->ceiling += s->source_position.y;
		s->floor = scan_floor;
	}
	out->state = SENSE_OPEN;
	if (kind == SENSE_SURFACE_TO_AIR)
	{
		echo(s, out, 0.0f, 0.0f, 0.0f, 0.0);
	}
	return 0;
}

/* one candidate; returns 0, 1 when no scan is open or the scan is not between candidates, 2 for a scan kind that does not match */
int eech_sense_candidate(void *scan, int kind, const float *target, int has_difficulty, float difficulty, float radar_altitude, eech_sense_answer *out)
{
	eech_sense_scan *s = (eech_sense_scan *) scan;
	vec3d target_position;
	float target_range;

	reset_answer(out);
	if (!s->open || s->awaiting != AWAIT_CANDIDATE)
	{
		return 1;
	}
	if (s->scan != kind)
	{
		return 2;
	}
	target_position.x = target[0];
	target_position.y = target[1];
	target_position.z = target[2];

	switch (s->scan)
	{
	case SENSE_AIR_TO_SURFACE:
		/* ac_tgt.c get_aircraft_air_to_surface_scan_group_targets */
		target_range = get_sqr_2d_range(&s->source_position, &target_position);
		out->has_range = 1;
		out->target_range = target_range;
		if (target_range > s->squared_range)
		{
			reject(s, SENSE_REASON_RANGE, out);
		}
		else
		{
			out->state = SENSE_ACCEPT;
		}
		return 0;

	case SENSE_SURFACE_TO_SURFACE:
		/* vh_tgt.c get_vehicle_surface_to_surface_scan_group_targets */
		target_range = approx_3d_range(&s->source_position, &target_position);
		out->has_range = 1;
		out->target_range = target_range;
		if (target_range > s->scan_range)
		{
			reject(s, SENSE_REASON_RANGE, out);
		}
		else
		{
			out->state = SENSE_ACCEPT;
		}
		return 0;

	case SENSE_AIR_TO_AIR:
		/* ac_tgt.c get_aircraft_air_to_air_scan_group_targets: the range, then the emitter and stealth adjustment (the signature) */
		target_range = get_sqr_2d_range(&s->source_position, &target_position);
		s->target_range = target_range;
		out->has_range = 1;
		out->target_range = target_range;
		s->awaiting = AWAIT_SIGNATURE;
		out->state = SENSE_NEED_SIGNATURE;
		return 0;

	default:
		/* vh_tgt.c get_vehicle_surface_to_air_scan_group_targets */
		target_range = approx_3d_range(&s->source_position, &target_position);
		s->target_range = target_range;
		s->radar_altitude = radar_altitude;
		out->has_range = 1;
		out->target_range = target_range;

		/* DECREASE SCAN FLOOR FOR CLOSE TARGETS */
		s->floor *= bound(target_range / s->scan_range, 0.1, 1.0);

		if (has_difficulty)
		{
			s->floor *= difficulty;
		}

		echo(s, out, 0.0f, 0.0f, 0.0f, 0.0);

		if ((radar_altitude < s->floor) || (radar_altitude > s->ceiling))
		{
			reject(s, SENSE_REASON_ALTITUDE, out);
		}
		else
		{
			s->awaiting = AWAIT_DRAW;
			out->state = SENSE_NEED_DRAW;
		}
		return 0;
	}
}

/* the campaign's random draw; returns 0, or 1 when none was asked for */
int eech_sense_draw(void *scan, double draw, eech_sense_answer *out)
{
	eech_sense_scan *s = (eech_sense_scan *) scan;
	double limit;

	reset_answer(out);
	if (!s->open || s->awaiting != AWAIT_DRAW)
	{
		return 1;
	}
	out->has_range = 1;
	out->target_range = s->target_range;

	/* allow for random tolerance: target_radar_altitude < (source_air_scan_floor * (1.0 + (0.5 * frand1x (&seed)))) */
	limit = s->floor * (1.0 + (0.5 * draw));
	echo(s, out, 0.0f, 0.0f, 0.0f, limit);
	if (s->radar_altitude < limit)
	{
		reject(s, SENSE_REASON_TOLERANCE, out);
	}
	else
	{
		s->awaiting = AWAIT_SIGNATURE;
		out->state = SENSE_NEED_SIGNATURE;
	}
	return 0;
}

/* the emitters and the helicopter's configuration; returns 0, 1 when none was asked for, 2 for a stealth factor that is not positive */
int eech_sense_signature(void *scan, int has_emitters, int radar_on, int radar_jammer_on, int infra_red_jammer_on, int helicopter, int weapon_bay_closed,
	int stub_wings_attached, int undercarriage_up, eech_sense_answer *out)
{
	eech_sense_scan *s = (eech_sense_scan *) scan;
	float adj_range;
	float value;
	float limit;

	reset_answer(out);
	if (!s->open || s->awaiting != AWAIT_SIGNATURE)
	{
		return 1;
	}

	/* adjust range if target is player - each jammer / radar turned off allows player to fly 10% closer */
	adj_range = s->target_range;

	if (has_emitters)
	{
		if (!radar_on)
		{
			adj_range *= 1.1;
		}
		if (!radar_jammer_on)
		{
			adj_range *= 1.1;
		}
		if (!infra_red_jammer_on)
		{
			adj_range *= 1.1;
		}
	}

	/* hc_float.c FLOAT_TYPE_RADAR_STEALTH_FACTOR (en_float.c's default, NO_RADAR_STEALTH_FACTOR, for anything that is not a helicopter) */
	value = NO_RADAR_STEALTH_FACTOR;

	if (helicopter)
	{
		if (weapon_bay_closed)
		{
			value -= NO_RADAR_STEALTH_FACTOR * 0.4;
		}

		if (!stub_wings_attached)
		{
			value -= NO_RADAR_STEALTH_FACTOR * 0.35;
		}

		if (undercarriage_up)
		{
			value -= NO_RADAR_STEALTH_FACTOR * 0.1;
		}

		if (!(value > 0.0))
		{
			return 2;
		}
	}

	out->has_range = 1;
	out->target_range = s->target_range;

	if (s->scan == SENSE_AIR_TO_AIR)
	{
		limit = s->squared_range * value;
	}
	else
	{
		limit = s->scan_range * value;
	}
	echo(s, out, adj_range, value, limit, 0.0);

	if (adj_range > limit)
	{
		reject(s, SENSE_REASON_RANGE_ADJUSTED, out);
	}
	else
	{
		s->awaiting = AWAIT_CANDIDATE;
		out->state = SENSE_ACCEPT;
	}
	return 0;
}

/* closes the scan; returns 0, or 1 when none was open */
int eech_sense_end(void *scan, eech_sense_answer *out)
{
	eech_sense_scan *s = (eech_sense_scan *) scan;

	reset_answer(out);
	if (!s->open)
	{
		return 1;
	}
	s->open = 0;
	out->state = SENSE_CLOSED;
	return 0;
}
