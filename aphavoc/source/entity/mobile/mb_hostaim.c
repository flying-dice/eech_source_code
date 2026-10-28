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
		case CAMPAIGN_AIM_RANGE: return "range";
		case CAMPAIGN_AIM_FIGURES: return "figures";
		case CAMPAIGN_AIM_WEAPON_STEP: return "weapon_step";
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

/* the floats of the table a weapon's figures give: velocity tests by pitch indices by `max_range / 20 + 2` ranges, two floats a cell */
static size_t table_cell_count (const campaign_weapon_figures *figures)
{
	return (size_t) CAMPAIGN_TABLE_VELOCITY_TESTS * CAMPAIGN_TABLE_PITCH_INDICES * (size_t) ((int) (figures->max_range / CAMPAIGN_TABLE_RANGE_STEP) + 2) * 2;
}

static void validate_request (const campaign_aim_step *step)
{
	switch (step->mode)
	{
		case CAMPAIGN_AIM_TABLE:
		case CAMPAIGN_AIM_RANGE:
		{
			const campaign_weapon_figures *figures = &step->figures;

			if ((step->weapon <= 0) || !all_finite (&figures->start_weight, 16) || !all_finite (&figures->cruise_time_max_error, 4) || (figures->max_range <= 0.0) || (figures->max_range > CAMPAIGN_TABLE_MAX_RANGE) || (figures->name[CAMPAIGN_WEAPON_NAME_MAX] != '\0') ||
				((step->pass_first != FALSE) && (step->pass_first != TRUE)) || ((step->pass_last != FALSE) && (step->pass_last != TRUE)) ||
				(step->echo && (step->mode == CAMPAIGN_AIM_TABLE) && (!step->echo_cells || (step->echo_cell_capacity != table_cell_count (figures)))))
			{
				debug_fatal ("host aim: a %s request with inconsistent figures for weapon %d", mode_name (step->mode), step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_FIGURES:
		{
			const campaign_weapon_figures *figures = &step->figures;

			if ((step->weapon <= 0) || !all_finite (&figures->start_weight, 16) || !all_finite (&figures->cruise_time_max_error, 4) || (figures->name[CAMPAIGN_WEAPON_NAME_MAX] != '\0'))
			{
				debug_fatal ("host aim: a figures request with inconsistent figures for weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_WEAPON_STEP:
		{
			const campaign_weapon_state *state = &step->weapon_state;
			int i;

			if ((step->weapon <= 0) || !isfinite (step->delta_time) || !all_finite (step->map_min, 3) || !all_finite (step->map_max, 3) || !all_finite (state->position, 3) ||
				!all_finite (state->attitude, 9) || !all_finite (state->motion_vector, 3) || !all_finite (&state->velocity, 3) ||
				(state->loal_mode & ~1) || (state->weapon_on_ground & ~1) || (state->has_launcher & ~1) || (state->has_target & ~1) || (state->server & ~1) ||
				(step->answer_count < 0) || (step->answer_count > CAMPAIGN_STEP_ANSWERS_MAX) || (step->draw_count < 0) || (step->draw_count > CAMPAIGN_STEP_DRAWS_MAX))
			{
				debug_fatal ("host aim: a weapon_step request with an inconsistent weapon %d", step->weapon);
			}

			for (i = 0; i < 4; i++)
			{
				if ((state->entity_samples[i] < 0) || (state->entity_samples[i] > 32767))
				{
					debug_fatal ("host aim: a weapon_step request with an entity sample %d outside [0, 32767]", state->entity_samples[i]);
				}
			}

			for (i = 0; i < step->draw_count; i++)
			{
				if ((step->draws[i] < 0) || (step->draws[i] > 32767))
				{
					debug_fatal ("host aim: a weapon_step request with a draw %d outside [0, 32767]", step->draws[i]);
				}
			}

			for (i = 0; i < step->answer_count; i++)
			{
				if (!all_finite (step->answers[i].f, 12))
				{
					debug_fatal ("host aim: a weapon_step request with a non-finite lookup answer");
				}
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
				!all_finite (&step->current_heading, 4) || !isfinite (step->delta_time) || !isfinite (step->track_pitch) || !all_finite (step->device_position, 3) ||
				!all_finite (step->tracking_point, 3))
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

	if ((result->state != CAMPAIGN_AIM_DONE) && (result->state != CAMPAIGN_AIM_NEED_DRAW) && (result->state != CAMPAIGN_AIM_NEED_LOOKUP))
	{
		debug_fatal ("host aim: host aim answered state %d to a %s request", result->state, mode_name (step->mode));
	}

	if (result->state == CAMPAIGN_AIM_NEED_LOOKUP)
	{
		// only a weapon step asks for lookups, and each must be a lookup the campaign can answer
		if (step->mode != CAMPAIGN_AIM_WEAPON_STEP)
		{
			debug_fatal ("host aim: host aim asked for lookups in answer to a %s request", mode_name (step->mode));
		}

		if ((result->lookup_count < 1) || (result->lookup_count > CAMPAIGN_STEP_LOOKUPS_MAX))
		{
			debug_fatal ("host aim: host aim asked for %d lookups in answer to a weapon_step request", result->lookup_count);
		}

		for (i = 0; i < result->lookup_count; i++)
		{
			if ((result->lookups[i].kind < 1) || (result->lookups[i].kind >= CAMPAIGN_STEP_LOOKUPS) || !all_finite (result->lookups[i].f, 12))
			{
				debug_fatal ("host aim: host aim asked a weapon_step request for a malformed lookup (kind %d)", result->lookups[i].kind);
			}
		}

		return;
	}

	if (result->state == CAMPAIGN_AIM_NEED_DRAW)
	{
		// only the lead prediction has a random jitter, and a request that already carries the sample has nothing further to ask; a weapon step takes its draws one
		// at a time, up to the room it has
		if (step->mode == CAMPAIGN_AIM_WEAPON_STEP)
		{
			if (step->draw_count >= CAMPAIGN_STEP_DRAWS_MAX)
			{
				debug_fatal ("host aim: host aim asked a weapon_step request for more than %d draws", CAMPAIGN_STEP_DRAWS_MAX);
			}
		}
		else if (((step->mode != CAMPAIGN_AIM_VECTORS) && (step->mode != CAMPAIGN_AIM_LEAD)) || step->has_draw)
		{
			debug_fatal ("host aim: host aim asked for a draw in answer to a %s request%s", mode_name (step->mode), step->has_draw ? " that already carried one" : "");
		}

		return;
	}

	// the time of flight is a partial output: its presence is a fact of its own, only a lead or a ballistic answer may carry it, a ballistic failure never does, and an
	// absent one is zero (the value of an absent output is not read anywhere)
	if ((result->has_time_of_flight != FALSE) && (result->has_time_of_flight != TRUE))
	{
		debug_fatal ("host aim: host aim answered a %s request with a non-boolean time-of-flight presence %d", mode_name (step->mode), result->has_time_of_flight);
	}

	if (result->has_time_of_flight)
	{
		if (((step->mode != CAMPAIGN_AIM_LEAD) && (step->mode != CAMPAIGN_AIM_BALLISTIC)) || ((step->mode == CAMPAIGN_AIM_BALLISTIC) && !result->result))
		{
			debug_fatal ("host aim: host aim answered a %s request with a time of flight it cannot carry", mode_name (step->mode));
		}

		require_finite (step, "time of flight", &result->time_of_flight, 1);
	}
	else if (result->time_of_flight != 0.0)
	{
		debug_fatal ("host aim: host aim answered a %s request with a time-of-flight value but no presence", mode_name (step->mode));
	}

	switch (step->mode)
	{
		case CAMPAIGN_AIM_TABLE:
		{
			// the host's table is its own: the answer carries nothing, except the diagnostic echo the fixture build ASKED for. The echo is optional (a host with a model of its
			// own has no EECH table to echo and is healthy without it), but one that is given must be exactly the table the figures give
			if (step->echo && (result->echo_cell_count != 0) && (result->echo_cell_count != table_cell_count (&step->figures)))
			{
				debug_fatal ("host aim: host aim echoed %lu table cells for weapon %d, not the %lu its figures give", (unsigned long) result->echo_cell_count, step->weapon,
					(unsigned long) table_cell_count (&step->figures));
			}

			break;
		}
		case CAMPAIGN_AIM_RANGE:
		{
			// the diagnostic echo is optional here too; one that is given is the two range runs
			if (step->echo && result->has_echo && (result->echo_count != 2))
			{
				debug_fatal ("host aim: host aim echoed %d range runs for weapon %d, not 2", result->echo_count, step->weapon);
			}

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

				if (!result->has_time_of_flight)
				{
					debug_fatal ("host aim: host aim answered a successful ballistic request without a time of flight");
				}
			}

			break;
		}
		case CAMPAIGN_AIM_INTERCEPT:
		{
			require_finite (step, "intercept point", result->intercept_point, 3);

			break;
		}
		case CAMPAIGN_AIM_FIGURES:
		{
			break;
		}
		case CAMPAIGN_AIM_WEAPON_STEP:
		{
			// the actions, each of a kind the campaign applies, with figures it can apply; their meaning is checked as each is applied
			if ((result->action_count < 0) || (result->action_count > CAMPAIGN_STEP_ACTIONS_MAX))
			{
				debug_fatal ("host aim: host aim answered a weapon_step request with %d actions", result->action_count);
			}

			for (i = 0; i < result->action_count; i++)
			{
				if ((result->actions[i].kind < 1) || (result->actions[i].kind >= CAMPAIGN_STEP_ACTIONS) || !all_finite (result->actions[i].f, 12))
				{
					debug_fatal ("host aim: host aim answered a weapon_step request with a malformed action (kind %d)", result->actions[i].kind);
				}

				if ((result->actions[i].d < 0) || (result->actions[i].d > (step->echo && result->has_echo ? result->echo_count : 0)))
				{
					debug_fatal ("host aim: host aim answered a weapon_step request with an action that follows %d echo events", result->actions[i].d);
				}
			}

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
			if (!(((result->echo[i].kind >= CAMPAIGN_AIM_EVENT_INTERCEPT) && (result->echo[i].kind <= CAMPAIGN_AIM_EVENT_RANGE_RESULT)) ||
				((step->mode == CAMPAIGN_AIM_WEAPON_STEP) && (result->echo[i].kind >= CAMPAIGN_AIM_EVENT_STEP_PATH) && (result->echo[i].kind <= CAMPAIGN_AIM_EVENT_STEP_CONTACT_TREES))) ||
				!all_finite (result->echo[i].f, 12))
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

campaign_aim_result campaign_aim_generate (int mode, int weapon, const campaign_weapon_figures *figures, int pass_first, int pass_last, float *echo_cells, size_t echo_cell_capacity)
{
	campaign_aim_step step;

	campaign_aim_result result;

	memset (&step, 0, sizeof (step));

	step.mode = mode;

	step.weapon = weapon;

	step.figures = *figures;

	step.pass_first = pass_first;

	step.pass_last = pass_last;

	if (echo_cells)
	{
		step.echo = TRUE;

		step.echo_out = echo_buffer;

		step.echo_capacity = CAMPAIGN_AIM_ECHO_MAX;

		step.echo_cells = echo_cells;

		step.echo_cell_capacity = echo_cell_capacity;
	}

	result = ask_once (&step);

	return result;
}

void campaign_aim_supply_figures (int weapon, const campaign_weapon_figures *figures)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_FIGURES;

	step.weapon = weapon;

	step.figures = *figures;

	(void) ask_once (&step);
}

campaign_aim_result campaign_aim_weapon_step (campaign_aim_step *step, campaign_step_answer_fn answer, void *context)
{
	campaign_aim_result result;

	int round, i, j;

	step->mode = CAMPAIGN_AIM_WEAPON_STEP;

	step->echo_out = step->echo ? echo_buffer : NULL;

	step->echo_capacity = step->echo ? CAMPAIGN_AIM_ECHO_MAX : 0;

	step->answer_count = 0;

	step->draw_count = 0;

	for (round = 0; round < CAMPAIGN_STEP_ROUNDS_MAX; round++)
	{
		result = ask_once (step);

		if (result.state == CAMPAIGN_AIM_DONE)
		{
			return result;
		}

		if (result.state == CAMPAIGN_AIM_NEED_DRAW)
		{
			// one sample of the campaign's shared stream, at the point the original took it: the raw integer of frand1 ()
			step->draws[step->draw_count++] = rand16 ();
		}
		else
		{
			for (i = 0; i < result.lookup_count; i++)
			{
				for (j = 0; j < step->answer_count; j++)
				{
					if ((step->answers[j].kind == result.lookups[i].kind) && (step->answers[j].a == result.lookups[i].a) && (step->answers[j].b == result.lookups[i].b) &&
						(step->answers[j].c == result.lookups[i].c) && (step->answers[j].d == result.lookups[i].d) && !memcmp (step->answers[j].f, result.lookups[i].f, 6 * sizeof (float)))
					{
						debug_fatal ("host aim: host aim asked a weapon_step request again for a lookup it was already given (kind %d)", result.lookups[i].kind);
					}
				}

				if (step->answer_count >= CAMPAIGN_STEP_ANSWERS_MAX)
				{
					debug_fatal ("host aim: a weapon_step request needed more than %d lookup answers", CAMPAIGN_STEP_ANSWERS_MAX);
				}

				answer (&result.lookups[i], &step->answers[step->answer_count], context);

				step->answer_count++;
			}
		}
	}

	debug_fatal ("host aim: a weapon step did not finish in %d rounds", CAMPAIGN_STEP_ROUNDS_MAX);

	return result;
}
