/*
 * The campaign's half of host-owned physical sensing (EEDCS-43): see mb_hostsense.h.  This file holds no sensing arithmetic: no range,
 * no floor or ceiling, no tolerance limit, no radar signature and no emitter adjustment.  It reads the campaign facts the host needs at
 * the points where the original scans read them, carries each candidate to the host in the order the scan enumerated it, and acts on the
 * host's verdict.  It validates every host answer.
 *
 * Two things stay here because they belong to the campaign: the random draw (taken from the source-derived generator at the point the
 * original scan took it, after the altitude test, and passed to the host as a measured fact; the generator's seed is the source's slot in
 * the entity heap and never leaves this file), and the configuration facts of a helicopter target (state of its weapon bay, stub wings
 * and undercarriage), which the reference host turns into its radar-stealth factor.  The positions, radar altitude and unit figures this file
 * reads are transitional mirrors of what a world-owning host already knows, supplied so the reference host can measure with them; what comes
 * back is the host's verdict, including a NOT_PERCEIVED one from a model of its own.
 */

#include "project.h"

#include <math.h>
#include <string.h>

#include "mb_hostsense.h"
#include "entity/mobile/mb_sensorfx.h"

// the surface-to-air scan lowers the floor with every candidate; a host cannot need more than a few messages for one, so this is far above
// any real dialogue
#define CAMPAIGN_SENSE_MAX_REQUESTS 16

static campaign_sense_host current_host;
static int scan_open;
static int scan_kind;
static float scan_range_raw;

void campaign_sense_set_host (const campaign_sense_host *host)
{
	if (host)
	{
		current_host = *host;
	}
	else
	{
		memset (&current_host, 0, sizeof (current_host));
	}

	scan_open = FALSE;
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

static void fill_host_id (char *out, entity *en)
{
	const char *host_id = campaign_roster_host_id (en);

	out[0] = '\0';

	if (host_id && host_id[0])
	{
		if (strlen (host_id) > CAMPAIGN_SENSE_HOST_ID_MAX)
		{
			debug_fatal ("host sense: entity %d has a host identity longer than %d bytes", get_local_entity_index (en), CAMPAIGN_SENSE_HOST_ID_MAX);
		}

		memcpy (out, host_id, strlen (host_id) + 1);
	}
}

static int reason_allowed (int scan, int reason)
{
	// the host's own verdict from its own model names no predicate of the original scans, so every scan accepts it
	if (reason == CAMPAIGN_SENSE_REASON_NOT_PERCEIVED)
	{
		return (scan >= CAMPAIGN_SENSE_AIR_TO_AIR) && (scan <= CAMPAIGN_SENSE_SURFACE_TO_SURFACE);
	}

	switch (scan)
	{
		case CAMPAIGN_SENSE_AIR_TO_AIR:
		{
			return reason == CAMPAIGN_SENSE_REASON_RANGE_ADJUSTED;
		}
		case CAMPAIGN_SENSE_AIR_TO_SURFACE:
		case CAMPAIGN_SENSE_SURFACE_TO_SURFACE:
		{
			return reason == CAMPAIGN_SENSE_REASON_RANGE;
		}
		case CAMPAIGN_SENSE_SURFACE_TO_AIR:
		{
			return (reason == CAMPAIGN_SENSE_REASON_ALTITUDE) || (reason == CAMPAIGN_SENSE_REASON_TOLERANCE) || (reason == CAMPAIGN_SENSE_REASON_RANGE_ADJUSTED);
		}
	}

	return FALSE;
}

static const char *mode_name (int mode)
{
	switch (mode)
	{
		case CAMPAIGN_SENSE_BEGIN: return "begin";
		case CAMPAIGN_SENSE_CANDIDATE: return "candidate";
		case CAMPAIGN_SENSE_DRAW: return "draw";
		case CAMPAIGN_SENSE_SIGNATURE: return "signature";
		case CAMPAIGN_SENSE_END: return "end";
	}

	return "unknown";
}

// which answers a message may receive: the state, and whether a measured range must or may accompany it
static campaign_sense_result ask_host (const campaign_sense_step *step)
{
	campaign_sense_result
		result;

	char
		error[CAMPAIGN_SENSE_ERROR_MAX + 1];

	int
		legal;

	memset (error, 0, sizeof (error));

	memset (&result, 0, sizeof (result));

	if (current_host.sense (step, &result, error, CAMPAIGN_SENSE_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host sense: host sense failed: %s", error[0] ? error : "unspecified error");
	}

	switch (step->mode)
	{
		case CAMPAIGN_SENSE_BEGIN:
		{
			legal = (result.state == CAMPAIGN_SENSE_OPEN);

			break;
		}
		case CAMPAIGN_SENSE_CANDIDATE:
		{
			legal = (result.state == CAMPAIGN_SENSE_ACCEPT) || (result.state == CAMPAIGN_SENSE_REJECT) || (result.state == CAMPAIGN_SENSE_NEED_SIGNATURE) ||
					((result.state == CAMPAIGN_SENSE_NEED_DRAW) && (step->scan == CAMPAIGN_SENSE_SURFACE_TO_AIR));

			// the scans that need a signature ask for it; the others have nothing further to ask
			if ((result.state == CAMPAIGN_SENSE_NEED_SIGNATURE) && (step->scan != CAMPAIGN_SENSE_AIR_TO_AIR) && (step->scan != CAMPAIGN_SENSE_SURFACE_TO_AIR))
			{
				legal = FALSE;
			}

			break;
		}
		case CAMPAIGN_SENSE_DRAW:
		{
			legal = (result.state == CAMPAIGN_SENSE_REJECT) || (result.state == CAMPAIGN_SENSE_NEED_SIGNATURE);

			break;
		}
		case CAMPAIGN_SENSE_SIGNATURE:
		{
			legal = (result.state == CAMPAIGN_SENSE_ACCEPT) || (result.state == CAMPAIGN_SENSE_REJECT);

			break;
		}
		case CAMPAIGN_SENSE_END:
		{
			legal = (result.state == CAMPAIGN_SENSE_CLOSED);

			break;
		}
		default:
		{
			legal = FALSE;
		}
	}

	if (!legal)
	{
		debug_fatal ("host sense: host sense answered state %d to a %s message of scan %d", result.state, mode_name (step->mode), step->scan);
	}

	if (result.state == CAMPAIGN_SENSE_REJECT)
	{
		if (!reason_allowed (step->scan, result.reason))
		{
			debug_fatal ("host sense: host sense rejected a candidate of scan %d with reason %d", step->scan, result.reason);
		}
	}

	if (result.has_range && (!isfinite (result.target_range) || (result.target_range < 0.0)))
	{
		debug_fatal ("host sense: host sense reported a range that is not a finite non-negative number");
	}

	if ((result.state == CAMPAIGN_SENSE_ACCEPT) && !result.has_range)
	{
		debug_fatal ("host sense: host sense accepted a candidate without a measured range");
	}

	if (result.has_echo)
	{
		if (!all_finite (&result.echo_ceiling, 5) || !isfinite (result.echo_tolerance_limit))
		{
			debug_fatal ("host sense: host sense trace echo has a non-finite value");
		}
	}

	return result;
}

void campaign_sense_begin (int scan, entity *source, const vec3d *source_position, float scan_range, float scan_ceiling, float scan_floor)
{
	campaign_sense_step
		step;

	campaign_sense_result
		result;

	ASSERT (source);

	ASSERT (source_position);

	if (!current_host.sense)
	{
		debug_fatal ("host sense: a scan needs a host sense callback");
	}

	if (scan_open)
	{
		debug_fatal ("host sense: a scan was started inside another");
	}

	if ((scan < CAMPAIGN_SENSE_AIR_TO_AIR) || (scan > CAMPAIGN_SENSE_SURFACE_TO_SURFACE))
	{
		debug_fatal ("host sense: unknown scan %d", scan);
	}

	if (!isfinite (source_position->x) || !isfinite (source_position->y) || !isfinite (source_position->z) || !isfinite (scan_range) || !isfinite (scan_ceiling) ||
		!isfinite (scan_floor))
	{
		debug_fatal ("host sense: a scan with a non-finite source position or sensor figure");
	}

	scan_open = TRUE;

	scan_kind = scan;

	scan_range_raw = scan_range;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_SENSE_BEGIN;
	step.scan = scan;
	fill_host_id (step.source_host_id, source);
	step.source_position[0] = source_position->x;
	step.source_position[1] = source_position->y;
	step.source_position[2] = source_position->z;
	step.scan_range = scan_range;
	step.scan_ceiling = scan_ceiling;
	step.scan_floor = scan_floor;

	result = ask_host (&step);

	#ifdef EECH_SENSOR_FIXTURE

	if (scan == CAMPAIGN_SENSE_SURFACE_TO_AIR)
	{
		// the observer's record of the scan's start: the host's envelope ceiling is the group's ceiling plus the source's altitude
		SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_X, source_position->x)
		SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_Y, source_position->y)
		SENSOR_FX_F (EECH_SENSOR_TAG_SOURCE_Z, source_position->z)
		SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_RANGE, scan_range)
		SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_FLOOR_INITIAL, scan_floor)
		SENSOR_FX_F (EECH_SENSOR_TAG_SCAN_CEILING, result.has_echo ? result.echo_ceiling : 0.0f)
	}

	#endif
}

#ifdef EECH_SENSOR_FIXTURE

static int reason_to_verdict (int reason)
{
	switch (reason)
	{
		case CAMPAIGN_SENSE_REASON_RANGE: return EECH_SENSOR_VERDICT_RANGE;
		case CAMPAIGN_SENSE_REASON_RANGE_ADJUSTED: return EECH_SENSOR_VERDICT_RANGE_ADJUSTED;
		case CAMPAIGN_SENSE_REASON_ALTITUDE: return EECH_SENSOR_VERDICT_ALTITUDE;
		case CAMPAIGN_SENSE_REASON_TOLERANCE: return EECH_SENSOR_VERDICT_TOLERANCE;
	}

	// only the original's predicates reach here: a host's own NOT_PERCEIVED is counted apart (host_verdict) and never relabelled as one of them
	debug_fatal ("host sense: the sensor fixture has no predicate verdict for host reason %d", reason);

	return EECH_SENSOR_VERDICT_TOLERANCE;
}

// a host's own nondetection verdict names none of the original's predicates: the fixture counts it separately and skips the reference-only
// hooks of that exchange, so a run whose host never answers it records exactly what it recorded before
static int host_verdict (const campaign_sense_result *result)
{
	return (result->state == CAMPAIGN_SENSE_REJECT) && (result->reason == CAMPAIGN_SENSE_REASON_NOT_PERCEIVED);
}

#endif

int campaign_sense_candidate (entity *source, entity *target, const vec3d *target_position, float *target_range)
{
	campaign_sense_step
		step;

	campaign_sense_result
		result;

	int
		seed,
		player,
		requests;

	float
		radar_altitude;

	double
		draw;

	ASSERT (source);

	ASSERT (target);

	ASSERT (target_position);

	ASSERT (target_range);

	if (!scan_open)
	{
		debug_fatal ("host sense: a candidate was asked about with no scan open");
	}

	if (!isfinite (target_position->x) || !isfinite (target_position->y) || !isfinite (target_position->z))
	{
		debug_fatal ("host sense: a candidate with a non-finite position");
	}

	player = FALSE;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_SENSE_CANDIDATE;
	step.scan = scan_kind;
	fill_host_id (step.target_host_id, target);
	step.target_position[0] = target_position->x;
	step.target_position[1] = target_position->y;
	step.target_position[2] = target_position->z;

	if ((scan_kind == CAMPAIGN_SENSE_AIR_TO_AIR) || (scan_kind == CAMPAIGN_SENSE_SURFACE_TO_AIR))
	{
		player = (get_local_entity_int_value (target, INT_TYPE_PLAYER) != ENTITY_PLAYER_AI);

		step.target_player = player;
	}

	if (scan_kind == CAMPAIGN_SENSE_SURFACE_TO_AIR)
	{
		if (player)
		{
			step.has_difficulty = TRUE;
			step.difficulty_factor = get_scan_floor_difficulty_factor (target);
		}

		step.radar_altitude = get_local_entity_float_value (target, FLOAT_TYPE_RADAR_ALTITUDE);

		if (!isfinite (step.radar_altitude) || !isfinite (step.difficulty_factor))
		{
			debug_fatal ("host sense: a candidate with a non-finite radar altitude or difficulty factor");
		}
	}

	radar_altitude = step.radar_altitude;

	result = ask_host (&step);

	*target_range = result.has_range ? result.target_range : 0.0;

	#ifdef EECH_SENSOR_FIXTURE

	if (!host_verdict (&result))
	{
		SENSOR_FX_CANDIDATE ()
		SENSOR_FX_F (EECH_SENSOR_TAG_TARGET_RANGE, *target_range)

		if ((scan_kind == CAMPAIGN_SENSE_AIR_TO_AIR) || (scan_kind == CAMPAIGN_SENSE_SURFACE_TO_AIR))
		{
			SENSOR_FX_I (EECH_SENSOR_TAG_TARGET_PLAYER, player)
		}

		if (scan_kind == CAMPAIGN_SENSE_SURFACE_TO_AIR)
		{
			if (player)
			{
				SENSOR_FX_FLOOR_FACTOR (step.difficulty_factor)
			}

			SENSOR_FX_FLOOR (result.has_echo ? result.echo_floor : 0.0f, scan_range_raw, *target_range)
			SENSOR_FX_F (EECH_SENSOR_TAG_RADAR_ALTITUDE, step.radar_altitude)
		}
	}

	#endif

	requests = 0;

	while (result.state != CAMPAIGN_SENSE_ACCEPT)
	{
		if (result.state == CAMPAIGN_SENSE_REJECT)
		{
			#ifdef EECH_SENSOR_FIXTURE

			if (host_verdict (&result))
			{
				SENSOR_FX_NOT_PERCEIVED ()
			}
			else
			{
				SENSOR_FX_VERDICT (reason_to_verdict (result.reason))
			}

			#endif

			return FALSE;
		}

		requests++;

		if (requests > CAMPAIGN_SENSE_MAX_REQUESTS)
		{
			debug_fatal ("host sense: host sense asked for more than %d facts about one candidate", CAMPAIGN_SENSE_MAX_REQUESTS);
		}

		if (result.state == CAMPAIGN_SENSE_NEED_DRAW)
		{
			//
			// the random tolerance: drawn here, after the altitude test, from the source-derived local seed, exactly where the original
			// scan drew it.  Only the draw leaves; the seed is the source's heap slot and stays private.
			//

			seed = get_client_server_entity_random_number_seed (source);

			draw = frand1x (&seed);

			memset (&step, 0, sizeof (step));

			step.mode = CAMPAIGN_SENSE_DRAW;
			step.scan = scan_kind;
			step.draw = draw;

			result = ask_host (&step);

			#ifdef EECH_SENSOR_FIXTURE

			if (!host_verdict (&result))
			{
				SENSOR_FX_TOLERANCE (draw, result.has_echo ? result.echo_tolerance_limit : 0.0, radar_altitude)
			}

			#endif
		}
		else
		{
			//
			// the radar signature: the emitters of a human-flown target and, for a helicopter, the configuration facts of its stealth
			// factor, read in the order the original read them
			//

			memset (&step, 0, sizeof (step));

			step.mode = CAMPAIGN_SENSE_SIGNATURE;
			step.scan = scan_kind;

			if (player)
			{
				step.has_emitters = TRUE;
				step.radar_on = (get_local_entity_int_value (target, INT_TYPE_RADAR_ON) != 0);
				step.radar_jammer_on = (get_local_entity_int_value (target, INT_TYPE_RADAR_JAMMER_ON) != 0);
				step.infra_red_jammer_on = (get_local_entity_int_value (target, INT_TYPE_INFRA_RED_JAMMER_ON) != 0);
			}

			if (get_local_entity_type (target) == ENTITY_TYPE_HELICOPTER)
			{
				step.target_helicopter = TRUE;

				if (get_local_entity_int_value (target, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_AIRCRAFT_RAH66_COMANCHE)
				{
					step.weapon_bay_closed = (get_local_entity_weapon_system_ready_state (target) == WEAPON_SYSTEM_READY_CLOSED);
				}

				step.stub_wings_attached = (get_comanche_stub_wings_attached (target) != 0);

				step.undercarriage_up = (get_local_entity_undercarriage_state (target) == AIRCRAFT_UNDERCARRIAGE_UP);
			}

			result = ask_host (&step);

			#ifdef EECH_SENSOR_FIXTURE

			if (!host_verdict (&result))
			{
				if (player)
				{
					SENSOR_FX_I (EECH_SENSOR_TAG_RADAR_ON, step.radar_on)
					SENSOR_FX_I (EECH_SENSOR_TAG_RADAR_JAMMER_ON, step.radar_jammer_on)
					SENSOR_FX_I (EECH_SENSOR_TAG_INFRA_RED_JAMMER_ON, step.infra_red_jammer_on)
				}

				SENSOR_FX_F (EECH_SENSOR_TAG_ADJUSTED_RANGE, result.has_echo ? result.echo_adjusted_range : 0.0f)
				SENSOR_FX_F (EECH_SENSOR_TAG_STEALTH_FACTOR, result.has_echo ? result.echo_stealth_factor : 0.0f)
				SENSOR_FX_F (EECH_SENSOR_TAG_STEALTH_LIMIT, result.has_echo ? result.echo_stealth_limit : 0.0f)
			}

			#endif

			if (result.has_range)
			{
				*target_range = result.target_range;
			}
		}
	}

	#ifdef EECH_SENSOR_FIXTURE

	// the surface-to-air scan completes its record after line of sight, in the scan function
	if (scan_kind != CAMPAIGN_SENSE_SURFACE_TO_AIR)
	{
		SENSOR_FX_VERDICT (EECH_SENSOR_VERDICT_ACCEPT)
	}

	#endif

	return TRUE;
}

void campaign_sense_end (void)
{
	campaign_sense_step
		step;

	if (!scan_open)
	{
		debug_fatal ("host sense: a scan was ended with none open");
	}

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_SENSE_END;
	step.scan = scan_kind;

	scan_open = FALSE;

	ask_host (&step);
}
