/*
 * The campaign's half of host-owned physical weapon aiming and readiness (EEDCS-44): see mb_hostaim.h.  This file holds no aiming arithmetic: no
 * intercept, lead or ballistic prediction, no table interpolation, no device offset or step, no keyframe state.  It carries the questions the aiming
 * code asks to the host, answers the host's one request for a random sample from the campaign's own stream, and validates every answer.
 *
 * The random sample is the only thing the campaign contributes besides facts: the lead prediction jitters its final range by `5 * sfrand1 ()`, a
 * draw from the shared stream, and the draw is taken here at the point the original took it (the host reaches it, asks, and is asked again with the
 * sample).  The stream and its seed never leave the campaign; the host receives a raw integer and forms the double with the macro's own expression.
 */

#include "project.h"

#include <math.h>
#include <string.h>

#include "mb_hostaim.h"

static campaign_aim_host current_host;

/* the room the host writes its echo in, when the fixture build asks for one (requests are synchronous and never nest) */
static campaign_aim_event echo_buffer[CAMPAIGN_AIM_ECHO_MAX];

void campaign_aim_set_host (const campaign_aim_host *host)
{
	if (host)
	{
		current_host = *host;
	}
	else
	{
		memset (&current_host, 0, sizeof (current_host));
	}
}

int campaign_aim_available (void)
{
	return current_host.aim != NULL;
}

static void fill_host_id (char *out, entity *en)
{
	const char *host_id;

	out[0] = '\0';

	if (!en)
	{
		return;
	}

	host_id = campaign_roster_host_id (en);

	if (host_id && host_id[0])
	{
		if (strlen (host_id) > CAMPAIGN_AIM_HOST_ID_MAX)
		{
			debug_fatal ("host aim: entity %d has a host identity longer than %d bytes", get_local_entity_index (en), CAMPAIGN_AIM_HOST_ID_MAX);
		}

		memcpy (out, host_id, strlen (host_id) + 1);
	}
}

void campaign_aim_identify (campaign_aim_step *step, entity *source, entity *target)
{
	fill_host_id (step->source_host_id, source);

	fill_host_id (step->target_host_id, target);
}

static const char *mode_name (int mode)
{
	switch (mode)
	{
		case CAMPAIGN_AIM_TABLE: return "table";
		case CAMPAIGN_AIM_READY_UPDATE: return "ready_update";
		case CAMPAIGN_AIM_READY_COMMAND: return "ready_command";
		case CAMPAIGN_AIM_REQUIRED: return "required";
		case CAMPAIGN_AIM_STEP: return "step";
		case CAMPAIGN_AIM_VECTORS: return "vectors";
		case CAMPAIGN_AIM_LEAD: return "lead";
		case CAMPAIGN_AIM_BALLISTIC: return "ballistic";
		case CAMPAIGN_AIM_INTERCEPT: return "intercept";
	}

	return "unknown";
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

static void require_finite (const campaign_aim_step *step, const char *what, const float *values, int count)
{
	if (!all_finite (values, count))
	{
		debug_fatal ("host aim: host aim answered a %s request with a non-finite %s", mode_name (step->mode), what);
	}
}

static void validate_request (const campaign_aim_step *step)
{
	switch (step->mode)
	{
		case CAMPAIGN_AIM_TABLE:
		{
			if (!step->table_cells || (step->table_velocity_tests <= 0) || (step->table_pitch_indices <= 0) || (step->table_range_values <= 0) ||
				(step->table_cell_count != ((size_t) step->table_velocity_tests * step->table_pitch_indices * step->table_range_values * 2)) || !isfinite (step->table_max_range))
			{
				debug_fatal ("host aim: a ballistic table with an inconsistent shape for weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_READY_UPDATE:
		case CAMPAIGN_AIM_READY_COMMAND:
		{
			if (!isfinite (step->ready_state) || !isfinite (step->ready_rate) || !isfinite (step->delta_time))
			{
				debug_fatal ("host aim: a %s request with a non-finite figure", mode_name (step->mode));
			}

			break;
		}
		case CAMPAIGN_AIM_REQUIRED:
		case CAMPAIGN_AIM_STEP:
		{
			if (!all_finite (step->device_attitude, 9) || !all_finite (step->intercept_vector, 3) || !all_finite (&step->heading_min, 6) ||
				!all_finite (&step->current_heading, 4) || !isfinite (step->delta_time))
			{
				debug_fatal ("host aim: a %s request with a non-finite figure", mode_name (step->mode));
			}

			break;
		}
		case CAMPAIGN_AIM_VECTORS:
		case CAMPAIGN_AIM_LEAD:
		case CAMPAIGN_AIM_BALLISTIC:
		case CAMPAIGN_AIM_INTERCEPT:
		{
			if (!all_finite (step->device_position, 3) || !all_finite (step->target_position, 3) || !all_finite (step->target_point, 3) ||
				!all_finite (step->target_motion_vector, 3) || !all_finite (step->tracking_point, 3) || !all_finite (step->device_attitude, 9) ||
				!all_finite (step->map_min, 3) || !all_finite (step->map_max, 3) || !isfinite (step->cruise_velocity) || !isfinite (step->source_velocity) ||
				!isfinite (step->parent_velocity) || !isfinite (step->comms_weapon_lag) || !isfinite (step->range) || !isfinite (step->height_diff_or_pitch))
			{
				debug_fatal ("host aim: a %s request with a non-finite figure", mode_name (step->mode));
			}

			break;
		}
		default:
		{
			debug_fatal ("host aim: unknown request mode %d", step->mode);
		}
	}
}

static void validate_result (const campaign_aim_step *step, const campaign_aim_result *result)
{
	int i;

	if ((result->state != CAMPAIGN_AIM_DONE) && (result->state != CAMPAIGN_AIM_NEED_DRAW))
	{
		debug_fatal ("host aim: host aim answered state %d to a %s request", result->state, mode_name (step->mode));
	}

	if (result->state == CAMPAIGN_AIM_NEED_DRAW)
	{
		// only the lead prediction has a random jitter, and a request that already carries the sample has nothing further to ask
		if (((step->mode != CAMPAIGN_AIM_VECTORS) && (step->mode != CAMPAIGN_AIM_LEAD)) || step->has_draw)
		{
			debug_fatal ("host aim: host aim asked for a draw in answer to a %s request%s", mode_name (step->mode), step->has_draw ? " that already carried one" : "");
		}

		return;
	}

	switch (step->mode)
	{
		case CAMPAIGN_AIM_TABLE:
		{
			break;
		}
		case CAMPAIGN_AIM_READY_UPDATE:
		case CAMPAIGN_AIM_READY_COMMAND:
		{
			if (!isfinite (result->ready_state) || (result->ready_state < 0.0) || (result->ready_state > 4.0))
			{
				debug_fatal ("host aim: host aim answered a %s request with a state outside [0, 4]", mode_name (step->mode));
			}

			if ((step->mode == CAMPAIGN_AIM_READY_UPDATE) && ((result->phase < 0) || (result->phase > 3)))
			{
				debug_fatal ("host aim: host aim answered a ready_update request with phase %d", result->phase);
			}

			break;
		}
		case CAMPAIGN_AIM_REQUIRED:
		{
			require_finite (step, "required offset", &result->required_heading, 2);

			break;
		}
		case CAMPAIGN_AIM_STEP:
		{
			require_finite (step, "step", &result->delta_heading, 4);

			break;
		}
		case CAMPAIGN_AIM_VECTORS:
		{
			if ((result->result != FALSE) && (result->result != TRUE))
			{
				debug_fatal ("host aim: host aim answered a vectors request with a non-boolean validity %d", result->result);
			}

			require_finite (step, "weapon vector", result->weapon_vector, 3);
			require_finite (step, "target vector", result->target_vector, 3);
			require_finite (step, "intercept vector", result->intercept_vector, 3);
			require_finite (step, "range", &result->intercept_range, 1);

			if (result->intercept_range < 0.0)
			{
				debug_fatal ("host aim: host aim answered a vectors request with a negative range");
			}

			break;
		}
		case CAMPAIGN_AIM_LEAD:
		{
			if ((result->result != FALSE) && (result->result != TRUE))
			{
				debug_fatal ("host aim: host aim answered a lead request with a non-boolean result %d", result->result);
			}

			require_finite (step, "intercept point", result->intercept_point, 3);
			require_finite (step, "angle", &result->angle, 1);
			require_finite (step, "time of flight", &result->time_of_flight, 1);

			break;
		}
		case CAMPAIGN_AIM_BALLISTIC:
		{
			if ((result->result != FALSE) && (result->result != TRUE))
			{
				debug_fatal ("host aim: host aim answered a ballistic request with a non-boolean result %d", result->result);
			}

			if (result->result)
			{
				require_finite (step, "aiming angle", &result->angle, 1);
				require_finite (step, "time of flight", &result->time_of_flight, 1);
			}

			break;
		}
		case CAMPAIGN_AIM_INTERCEPT:
		{
			require_finite (step, "intercept point", result->intercept_point, 3);

			break;
		}
	}

	if (result->has_echo)
	{
		if (!step->echo)
		{
			debug_fatal ("host aim: host aim sent an echo nobody asked for");
		}

		if ((result->echo_count < 0) || (result->echo_count > CAMPAIGN_AIM_ECHO_MAX))
		{
			debug_fatal ("host aim: host aim sent %d echo events", result->echo_count);
		}

		for (i = 0; i < result->echo_count; i++)
		{
			if ((result->echo[i].kind < CAMPAIGN_AIM_EVENT_INTERCEPT) || (result->echo[i].kind > CAMPAIGN_AIM_EVENT_LEAD_RESULT) || !all_finite (result->echo[i].f, 12))
			{
				debug_fatal ("host aim: host aim sent a malformed echo event");
			}
		}
	}
}

static campaign_aim_result ask_once (const campaign_aim_step *step)
{
	campaign_aim_result result;

	char error[CAMPAIGN_AIM_ERROR_MAX + 1];

	if (!current_host.aim)
	{
		debug_fatal ("host aim: a %s request needs a host aim callback", mode_name (step->mode));
	}

	validate_request (step);

	memset (error, 0, sizeof (error));

	memset (&result, 0, sizeof (result));

	if (current_host.aim (step, &result, error, CAMPAIGN_AIM_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("host aim: host aim failed for a %s request: %s", mode_name (step->mode), error[0] ? error : "unspecified error");
	}

	result.echo = step->echo_out;

	validate_result (step, &result);

	return result;
}

campaign_aim_result campaign_aim_ask (campaign_aim_step *step, int allow_draw)
{
	campaign_aim_result result;

	step->echo_out = step->echo ? echo_buffer : NULL;

	step->echo_capacity = step->echo ? CAMPAIGN_AIM_ECHO_MAX : 0;

	result = ask_once (step);

	if (result.state == CAMPAIGN_AIM_NEED_DRAW)
	{
		if (!allow_draw)
		{
			debug_fatal ("host aim: host aim asked for a draw in answer to a %s request that cannot take one", mode_name (step->mode));
		}

		// one sample of the campaign's own stream, at the point the original lead prediction took it: the raw integer of sfrand1 ()
		step->has_draw = TRUE;

		step->draw_sample = rand16 ();

		result = ask_once (step);
	}

	return result;
}

void campaign_aim_supply_table (int weapon, float max_range, int velocity_tests, int pitch_indices, int range_values, const float *cells, size_t cell_count)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_TABLE;

	step.weapon = weapon;

	step.table_max_range = max_range;

	step.table_velocity_tests = velocity_tests;

	step.table_pitch_indices = pitch_indices;

	step.table_range_values = range_values;

	step.table_cells = cells;

	step.table_cell_count = cell_count;

	(void) ask_once (&step);
}
