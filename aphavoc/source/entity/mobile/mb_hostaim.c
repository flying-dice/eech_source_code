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
		case CAMPAIGN_AIM_WEAPON_BIRTH: return "weapon_birth";
		case CAMPAIGN_AIM_RELEASE: return "release";
		case CAMPAIGN_AIM_ANIMATE: return "animate";
		case CAMPAIGN_AIM_WEAPON_DECOY: return "weapon_decoy";
		case CAMPAIGN_AIM_WEAPON_SEDUCE: return "weapon_seduce";
		case CAMPAIGN_AIM_WEAPON_REACQUIRE: return "weapon_reacquire";
		case CAMPAIGN_AIM_FORECAST: return "forecast";
		case CAMPAIGN_AIM_DAMAGE: return "damage";
		case CAMPAIGN_AIM_HEALTH: return "health";
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

// the modes of the step protocol (lookups, samples of the shared stream, ordered actions)
static int step_mode (int mode)
{
	return (mode == CAMPAIGN_AIM_WEAPON_STEP) || (mode == CAMPAIGN_AIM_WEAPON_BIRTH) || (mode == CAMPAIGN_AIM_WEAPON_DECOY) || (mode == CAMPAIGN_AIM_WEAPON_SEDUCE) || (mode == CAMPAIGN_AIM_WEAPON_REACQUIRE);
}

static void validate_request (const campaign_aim_step *step)
{
	switch (step->mode)
	{
		case CAMPAIGN_AIM_TABLE:
		case CAMPAIGN_AIM_RANGE:
		{
			const campaign_weapon_figures *figures = &step->figures;

			if ((step->weapon <= 0) || !all_finite (&figures->start_weight, 16) || !all_finite (&figures->cruise_time_max_error, 4) || !all_finite (&figures->muzzle_velocity_max_error, 4) || (figures->max_range <= 0.0) || (figures->max_range > CAMPAIGN_TABLE_MAX_RANGE) || (figures->name[CAMPAIGN_WEAPON_NAME_MAX] != '\0') ||
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

			if ((step->weapon <= 0) || !all_finite (&figures->start_weight, 16) || !all_finite (&figures->cruise_time_max_error, 4) || !all_finite (&figures->muzzle_velocity_max_error, 4) || (figures->name[CAMPAIGN_WEAPON_NAME_MAX] != '\0'))
			{
				debug_fatal ("host aim: a figures request with inconsistent figures for weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_RELEASE:
		{
			if ((step->weapon <= 0) || !isfinite (step->delta_time) || (step->release_has_timer & ~1) || (step->release_not_ai & ~1))
			{
				debug_fatal ("host aim: a release request with an inconsistent weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_FORECAST:
		{
			// the armour level is a table value (never negative: the original takes a fractional power of it), the flags are booleans, and every figure is finite
			if ((step->weapon <= 0) || (step->forecast_armour_level < 0) || (step->forecast_target_not_ai & ~1) || !isfinite (step->forecast_figures.burst_duration) ||
				!isfinite (step->forecast_player_modifier) || !isfinite (step->forecast_damage_modifier) || !isfinite (step->forecast_velocity))
			{
				debug_fatal ("host aim: a forecast request with inconsistent figures for weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_DAMAGE:
		{
			const campaign_damage_request *damage = &step->damage;

			// every figure is finite, the flags are booleans, the models are known, and a health that is classified has an initial level to divide by.  A hit of any capability is
			// legal (a capability of zero or less does nothing), as is an armour level of any sign (the original only logged a zero), so neither is narrowed here
			if ((step->weapon <= 0) || !isfinite (damage->figures.burst_duration) || !isfinite (damage->weapon_speed) || !isfinite (damage->weapon_cruise_velocity) ||
				!isfinite (damage->damage_modifier) || !isfinite (damage->armour_front) || !isfinite (damage->armour_rear) || !isfinite (damage->armour_side) ||
				!all_finite (damage->weapon_position, 3) || !all_finite (damage->target_position, 3) || !all_finite (damage->target_attitude, 9) || !isfinite (damage->player_modifier) ||
				(damage->armour_easy & ~1) || (damage->target_not_ai & ~1) || (damage->player_modifier_valid & ~1) || (damage->has_launcher & ~1) || (damage->invulnerable & ~1) ||
				(damage->cheat_guard & ~1) || (damage->armour_model < CAMPAIGN_DAMAGE_ARMOUR_PLAIN) || (damage->armour_model > CAMPAIGN_DAMAGE_ARMOUR_AIRCRAFT) ||
				(damage->critical_model < CAMPAIGN_DAMAGE_CRITICAL_NONE) || (damage->critical_model > CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT) ||
				(((damage->critical_model == CAMPAIGN_DAMAGE_CRITICAL_VEHICLE) || (damage->critical_model == CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT)) && (damage->initial_damage_level <= 0)))
			{
				debug_fatal ("host aim: a damage request with inconsistent figures for weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_HEALTH:
		{
			const campaign_health_request *health = &step->health;
			int classified = (health->critical_model == CAMPAIGN_DAMAGE_CRITICAL_VEHICLE) || (health->critical_model == CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT);

			// the op and the model are known, the flags are booleans, and a health that is classified or assessed has an initial level to divide by.  A level of any value is legal
			// (the original never narrowed it), and so is a delta time of zero
			if ((health->op < CAMPAIGN_HEALTH_CLASSIFY) || (health->op > CAMPAIGN_HEALTH_HIT_ELIGIBLE) || (health->critical_model < CAMPAIGN_DAMAGE_CRITICAL_NONE) ||
				(health->critical_model > CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT) || (health->explosive & ~1) || (health->flammable & ~1) || (health->self_hit & ~1) || (health->target_not_ai & ~1) ||
				!isfinite (health->delta_time) || (health->aircraft_class < 0) || (health->aircraft_class > 2) ||
				(((health->op == CAMPAIGN_HEALTH_CLASSIFY) || (health->op == CAMPAIGN_HEALTH_ASSESS) || (health->op == CAMPAIGN_HEALTH_TIMED)) && classified && (health->initial_damage_level <= 0)) ||
				((health->op == CAMPAIGN_HEALTH_ASSESS) && (health->initial_damage_level <= 0)) ||
				((health->op == CAMPAIGN_HEALTH_TIMED) && (health->critical_model == CAMPAIGN_DAMAGE_CRITICAL_NONE)))
			{
				debug_fatal ("host aim: a health request (op %d) with inconsistent figures", health->op);
			}

			break;
		}
		case CAMPAIGN_AIM_ANIMATE:
		{
			if ((step->weapon <= 0) || !isfinite (step->animate_lifetime) || !isfinite (step->animate_motion_y) || (step->animate_on_ground & ~1) || (step->animate_status < 0) ||
				(step->animate_status > 4))
			{
				debug_fatal ("host aim: an animate request with an inconsistent weapon %d", step->weapon);
			}

			break;
		}
		case CAMPAIGN_AIM_WEAPON_STEP:
		case CAMPAIGN_AIM_WEAPON_BIRTH:
		case CAMPAIGN_AIM_WEAPON_DECOY:
		case CAMPAIGN_AIM_WEAPON_SEDUCE:
		case CAMPAIGN_AIM_WEAPON_REACQUIRE:
		{
			const campaign_weapon_state *state = &step->weapon_state;
			int i;

			if (!isfinite (state->coefficient) || !all_finite (state->effectiveness, 3) || (((step->mode == CAMPAIGN_AIM_WEAPON_SEDUCE) || (step->mode == CAMPAIGN_AIM_WEAPON_REACQUIRE)) && (state->other_weapon <= 0)))
			{
				debug_fatal ("host aim: a %s request with an inconsistent decoy or setting (decoy weapon %d)", mode_name (step->mode), state->other_weapon);
			}

			if ((step->weapon <= 0) || !isfinite (step->delta_time) || !all_finite (step->map_min, 3) || !all_finite (step->map_max, 3) || !all_finite (state->position, 3) ||
				!all_finite (state->attitude, 9) || !all_finite (state->motion_vector, 3) || !all_finite (&state->velocity, 3) ||
				(state->loal_mode & ~1) || (state->weapon_on_ground & ~1) || (state->has_launcher & ~1) || (state->has_target & ~1) || (state->server & ~1) || (state->submunition & ~1) ||
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
		// only a weapon step or birth asks for lookups, and each must be a lookup the campaign can answer
		if (!step_mode (step->mode))
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
		if (step_mode (step->mode))
		{
			if (step->draw_count >= CAMPAIGN_STEP_DRAWS_MAX)
			{
				debug_fatal ("host aim: host aim asked a weapon_step request for more than %d draws", CAMPAIGN_STEP_DRAWS_MAX);
			}
		}
		else if (((step->mode != CAMPAIGN_AIM_VECTORS) && (step->mode != CAMPAIGN_AIM_LEAD) && (step->mode != CAMPAIGN_AIM_DAMAGE) && (step->mode != CAMPAIGN_AIM_HEALTH)) || step->has_draw)
		{
			debug_fatal ("host aim: host aim asked for a draw in answer to a %s request%s", mode_name (step->mode), step->has_draw ? " that already carried one" : "");
		}

		// only a timed roll may ask for the campaign's sample: a classification, an assessment and a hit eligibility are pure questions.  WHICH targets roll (the reference host
		// draws for an eligible one only: a vehicle with health that can burn in the critical class, an aircraft in a class above none, never a person) is the host's physical
		// eligibility, not the campaign's: a host with a model of its own may roll for a person, for an aircraft in no class or for a vehicle that does not burn, and takes its one sample
		if ((step->mode == CAMPAIGN_AIM_HEALTH) && (step->health.op != CAMPAIGN_HEALTH_TIMED))
		{
			debug_fatal ("host aim: host aim asked for a draw in answer to a health request (op %d), which is a pure question", step->health.op);
		}

		// the player's invulnerability stops a hit before its arithmetic: a hit that is stopped takes no sample of the shared stream
		if ((step->mode == CAMPAIGN_AIM_DAMAGE) && step->damage.target_not_ai && step->damage.invulnerable)
		{
			debug_fatal ("host aim: host aim asked for a draw in answer to a damage request for weapon %d against an invulnerable target", step->weapon);
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
		case CAMPAIGN_AIM_FORECAST:
		{
			// the capability is a count of damage points: never negative.  The echo is optional (a host with a model of its own has none), but one that is given is exactly one
			// forecast event, and agrees with the answer
			if (result->forecast_capability < 0)
			{
				debug_fatal ("host aim: host aim answered a forecast request for weapon %d with a negative capability %d", step->weapon, result->forecast_capability);
			}

			if (step->echo && result->has_echo && ((result->echo_count != 1) || (result->echo[0].kind != CAMPAIGN_AIM_EVENT_FORECAST) || (result->echo[0].d != result->forecast_capability)))
			{
				debug_fatal ("host aim: host aim echoed a forecast that is not one event agreeing with its answer");
			}

			break;
		}
		case CAMPAIGN_AIM_DAMAGE:
		{
			const campaign_damage_request *request = &step->damage;
			const campaign_damage_result *damage = &result->damage;
			int not_applicable = (request->critical_model == CAMPAIGN_DAMAGE_CRITICAL_NONE) || (request->critical_model == CAMPAIGN_DAMAGE_CRITICAL_PERSON);
			int highest = (request->critical_model == CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT) ? 2 : ((request->critical_model == CAMPAIGN_DAMAGE_CRITICAL_VEHICLE) ? 1 : 0);

			if ((damage->exited & ~1) || (damage->written & ~1) || (damage->bad_modifier & ~1) || (not_applicable && (damage->critical_before || damage->critical_after)) ||
				(damage->critical_before < 0) || (damage->critical_before > highest) || (damage->critical_after < 0) || (damage->critical_after > highest))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d with exit %d, written %d, classes %d and %d (the target's classes are 0 to %d)", step->weapon,
					damage->exited, damage->written, damage->critical_before, damage->critical_after, highest);
			}

			// the player's invulnerability is the campaign's setting, given as a figure: the hit is stopped by it, and only by it, and a stopped hit takes no sample and changes nothing
			if (damage->exited != (request->target_not_ai && request->invulnerable))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d with exit %d for a target %s", step->weapon, damage->exited,
					request->invulnerable && request->target_not_ai ? "that is invulnerable" : "that is not");
			}

			if (damage->exited && (step->has_draw || damage->capability || damage->written || damage->bad_modifier || (damage->new_level != request->damage_level) ||
				(damage->critical_before != damage->critical_after)))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d with a hit that the invulnerability stopped and still changed something", step->weapon);
			}

			// a player's damage setting that the campaign could not resolve is the host's to report, and only for a player that had none
			if (damage->bad_modifier && (request->player_modifier_valid || !request->target_not_ai))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d with a missing damage setting that the request carried or did not need", step->weapon);
			}

			if (damage->bad_modifier)
			{
				// the campaign stops with the original's message; nothing else of the answer is used
				break;
			}

			// health is written only for a hit that does damage (the original writes it only for a capability above zero: a hit of zero or less does nothing and may not kill) and
			// not under the cheat guard, to a level from zero up to what it was; when it is not written it is as it was, and the classes did not change.  A host is free to
			// decide the health a positive capability leaves, and to write none for it
			if (damage->written && (damage->capability <= 0))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d writing health %d over %d for a hit of capability %d, which does nothing", step->weapon,
					damage->new_level, request->damage_level, damage->capability);
			}

			if (damage->written && (request->cheat_guard || (damage->new_level < 0) || (damage->new_level > ((request->damage_level > 0) ? request->damage_level : 0))))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d writing health %d over %d%s", step->weapon, damage->new_level, request->damage_level,
					request->cheat_guard ? " under the cheat guard" : "");
			}

			if (!damage->written && ((damage->new_level != request->damage_level) || (damage->critical_before != damage->critical_after)))
			{
				debug_fatal ("host aim: host aim answered a damage request for weapon %d changing health or its class without writing it", step->weapon);
			}

			// the echo (the fixture build's) is exactly one damage event agreeing with the answer, or none; a stopped hit has none.  Its raw-bit halves (f[4..11]) are exact integers in
			// [0, 65535]: they carry the four intermediate floats of the original's observer, any of which can be a legitimate not-a-number (see mb_hostaim.h)
			if (step->echo && result->has_echo && (damage->exited || (result->echo_count != 1) || (result->echo[0].kind != CAMPAIGN_AIM_EVENT_DAMAGE) || (result->echo[0].d != damage->capability)))
			{
				debug_fatal ("host aim: host aim echoed a damage that is not one event agreeing with its answer");
			}

			if (step->echo && result->has_echo)
			{
				for (i = 4; i < 12; i++)
				{
					if ((result->echo[0].f[i] < 0.0f) || (result->echo[0].f[i] > 65535.0f) || (result->echo[0].f[i] != floorf (result->echo[0].f[i])))
					{
						debug_fatal ("host aim: host aim echoed a damage whose raw-bit half %d is %g, not an integer in [0, 65535]", i, (double) result->echo[0].f[i]);
					}
				}
			}

			break;
		}
		case CAMPAIGN_AIM_HEALTH:
		{
			const campaign_health_request *request = &step->health;
			const campaign_health_result *health = &result->health;
			int highest = (request->critical_model == CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT) ? 2 : ((request->critical_model == CAMPAIGN_DAMAGE_CRITICAL_VEHICLE) ? 1 : 0);

			if ((health->crossed & ~1) || (health->recovered & ~1) || (health->eject & ~1) || (health->eligible & ~1) || (health->rolled & ~1) || (health->hit & ~1) || (health->fatal & ~1) ||
				(health->critical_class < 0) || (health->critical_class > highest))
			{
				debug_fatal ("host aim: host aim answered a health request (op %d) with a flag that is not a boolean or a class %d outside 0 to %d", request->op, health->critical_class, highest);
			}

			switch (request->op)
			{
				case CAMPAIGN_HEALTH_CLASSIFY:
				{
					// a pure question: nothing else is answered
					if (health->crossed || health->recovered || health->eject || health->eligible || health->rolled || health->hit || health->fatal)
					{
						debug_fatal ("host aim: host aim answered a health classification with more than a class");
					}

					break;
				}
				case CAMPAIGN_HEALTH_ASSESS:
				{
					// only an aircraft recovers or ejects, a vehicle that cannot burn crosses nothing, and a change cannot both fall below a threshold and recover above one
					if (health->critical_class || health->eligible || health->rolled || health->hit || health->fatal ||
						((request->critical_model != CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT) && (health->recovered || health->eject)) ||
						((request->critical_model != CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT) && !request->explosive && !request->flammable && health->crossed) ||
						(health->crossed && health->recovered))
					{
						debug_fatal ("host aim: host aim answered a health assessment (%d to %d) with an answer its model cannot give", request->old_damage_level, request->damage_level);
					}

					break;
				}
				case CAMPAIGN_HEALTH_TIMED:
				{
					int level = (request->damage_level > 0) ? request->damage_level : 0;

					// an ineligible target is not rolled for and does not change; a roll took the campaign's sample; a hit needs eligibility; a hit leaves health from zero to one
					// less than it was, and is fatal exactly when it leaves none; the class after is the class of the health the target is left with
					if (health->crossed || health->recovered || health->eject || (!health->eligible && (health->rolled || health->hit || health->fatal || (health->new_level != request->damage_level))) ||
						(health->rolled && !step->has_draw) || (health->hit && !health->eligible) || (!health->hit && (health->fatal || (health->new_level != request->damage_level))) ||
						(health->hit && ((health->new_level < 0) || (health->new_level > level) || ((request->damage_level > 0) && (health->new_level >= request->damage_level)))) ||
						(health->fatal != (health->hit && (health->new_level <= 0))) || (step->has_draw && !health->rolled))
					{
						debug_fatal ("host aim: host aim answered a timed health request (level %d) with eligible %d, rolled %d, hit %d, level %d, fatal %d", request->damage_level, health->eligible,
							health->rolled, health->hit, health->new_level, health->fatal);
					}

					break;
				}
				case CAMPAIGN_HEALTH_HIT_ELIGIBLE:
				{
					if (health->critical_class || health->crossed || health->recovered || health->eject || health->rolled || health->hit || health->fatal || (!request->self_hit && !health->eligible))
					{
						debug_fatal ("host aim: host aim answered a hit eligibility with more than a yes or no, or refused a hit that is not the target's own");
					}

					break;
				}
			}

			break;
		}
		case CAMPAIGN_AIM_RELEASE:
		{
			if ((result->release_refuse & ~1) || (result->release_write_last_shot & ~1) || (result->release_burst_size < 0) || (result->release_burst_size > 100000) ||
				(!result->release_refuse && (result->release_burst_size < 1)) || (result->release_write_last_shot && !step->release_has_timer))
			{
				debug_fatal ("host aim: host aim answered a release request with refuse %d, %d rounds and%s a last-shot clock for a launcher that %s", result->release_refuse,
					result->release_burst_size, result->release_write_last_shot ? "" : " no", step->release_has_timer ? "has one" : "has none");
			}

			break;
		}
		case CAMPAIGN_AIM_ANIMATE:
		{
			if ((result->animate_status < 0) || (result->animate_status > 4) || (result->animate_write_lifetime & ~1) || !isfinite (result->animate_lifetime) ||
				(!result->animate_write_lifetime && (result->animate_lifetime != 0.0)))
			{
				debug_fatal ("host aim: host aim answered an animate request with parachute state %d", result->animate_status);
			}

			break;
		}
		case CAMPAIGN_AIM_WEAPON_STEP:
		case CAMPAIGN_AIM_WEAPON_BIRTH:
		case CAMPAIGN_AIM_WEAPON_DECOY:
		case CAMPAIGN_AIM_WEAPON_SEDUCE:
		case CAMPAIGN_AIM_WEAPON_REACQUIRE:
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

				// the decisions the campaign acts on are booleans: 0 or 1, nothing else, whichever is meant; checked before any is applied
				if (((result->actions[i].kind == CAMPAIGN_STEP_ACTION_DECOY) || (result->actions[i].kind == CAMPAIGN_STEP_ACTION_REACQUIRE)) && (result->actions[i].a & ~1))
				{
					debug_fatal ("host aim: host aim answered a %s request with a non-boolean decision %d (action kind %d)", mode_name (step->mode), result->actions[i].a, result->actions[i].kind);
				}

				if ((result->actions[i].kind == CAMPAIGN_STEP_ACTION_SEDUCE) && ((result->actions[i].a & ~1) || (result->actions[i].b & ~1)))
				{
					debug_fatal ("host aim: host aim answered a %s request with non-boolean decisions %d and %d (action kind %d)", mode_name (step->mode), result->actions[i].a, result->actions[i].b,
						result->actions[i].kind);
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
				((step->mode == CAMPAIGN_AIM_FORECAST) && (result->echo[i].kind == CAMPAIGN_AIM_EVENT_FORECAST)) || ((step->mode == CAMPAIGN_AIM_DAMAGE) && (result->echo[i].kind == CAMPAIGN_AIM_EVENT_DAMAGE)) ||
				(step_mode (step->mode) && (result->echo[i].kind >= CAMPAIGN_AIM_EVENT_STEP_PATH) && (result->echo[i].kind <= CAMPAIGN_AIM_EVENT_STEP_BIRTH_DISPERSION))) ||
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

	if (!step_mode (step->mode))
	{
		debug_fatal ("host aim: a weapon step was asked in mode %d", step->mode);
	}

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

campaign_aim_result campaign_aim_release (int weapon, int has_timer, unsigned int last_shot, int not_ai)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_RELEASE;

	step.weapon = weapon;

	step.delta_time = get_delta_time ();

	step.system_time = get_system_time ();

	step.release_has_timer = has_timer;

	step.release_last_shot = last_shot;

	step.release_not_ai = not_ai;

	return ask_once (&step);
}

void campaign_step_hits_clear (campaign_step_hits *hits)
{
	memset (hits, 0, sizeof (*hits));
}

void campaign_step_hits_add (campaign_step_hits *hits, int which, const float *segment, struct ENTITY *entity)
{
	int i;

	for (i = 0; i < hits->count; i++)
	{
		if ((hits->hit[i].which == which) && !memcmp (hits->hit[i].segment, segment, 6 * sizeof (float)))
		{
			debug_fatal ("host aim: a weapon step's collision question was answered twice");
		}
	}

	if (hits->count >= CAMPAIGN_STEP_HITS_MAX)
	{
		debug_fatal ("host aim: a weapon step asked more than %d collision questions", CAMPAIGN_STEP_HITS_MAX);
	}

	hits->hit[hits->count].which = which;

	memcpy (hits->hit[hits->count].segment, segment, 6 * sizeof (float));

	hits->hit[hits->count].entity = entity;

	hits->count++;
}

int campaign_step_hits_find (const campaign_step_hits *hits, int which, const float *segment, struct ENTITY **entity)
{
	int i;

	for (i = 0; i < hits->count; i++)
	{
		if ((hits->hit[i].which == which) && !memcmp (hits->hit[i].segment, segment, 6 * sizeof (float)))
		{
			*entity = hits->hit[i].entity;

			return TRUE;
		}
	}

	return FALSE;
}

campaign_aim_result campaign_aim_forecast (int weapon, const campaign_damage_figures *figures, int armour_level, int target_not_ai, float player_modifier, float damage_modifier, float velocity, int echo)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_FORECAST;

	step.weapon = weapon;

	step.forecast_figures = *figures;

	step.forecast_armour_level = armour_level;

	step.forecast_target_not_ai = target_not_ai;

	step.forecast_player_modifier = player_modifier;

	step.forecast_damage_modifier = damage_modifier;

	step.forecast_velocity = velocity;

	if (echo)
	{
		step.echo = TRUE;

		step.echo_out = echo_buffer;

		step.echo_capacity = CAMPAIGN_AIM_ECHO_MAX;
	}

	// a pure question: ask_once refuses a NEED_DRAW (no sample of the shared stream is taken for a forecast) and a NEED_LOOKUP
	return ask_once (&step);
}

campaign_aim_result campaign_aim_damage (int weapon, const campaign_damage_request *request, int echo)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_DAMAGE;

	step.weapon = weapon;

	step.damage = *request;

	step.echo = echo ? TRUE : FALSE;

	// the hit takes at most one sample of the campaign's stream: campaign_aim_ask takes it once, after the host asks, and asks again
	return campaign_aim_ask (&step, TRUE);
}

// the raw bits of a float are carried in an unsigned int
typedef char campaign_float_is_unsigned_int_sized[(sizeof (float) == sizeof (unsigned int)) ? 1 : -1];

float campaign_aim_damage_echo_float (const campaign_aim_event *event, int index)
{
	unsigned int
		bits;

	float
		value;

	bits = (((unsigned int) event->f[4 + 2 * index]) << 16) | ((unsigned int) event->f[5 + 2 * index]);

	memcpy (&value, &bits, sizeof (value));

	return value;
}

campaign_aim_result campaign_aim_health (const campaign_health_request *request)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_HEALTH;

	step.health = *request;

	// a timed roll takes at most one sample of the campaign's stream: campaign_aim_ask takes it once, after the host asks, and asks again
	return campaign_aim_ask (&step, TRUE);
}

campaign_aim_result campaign_aim_animate (int weapon, int status, int on_ground, float lifetime, float motion_y)
{
	campaign_aim_step step;

	memset (&step, 0, sizeof (step));

	step.mode = CAMPAIGN_AIM_ANIMATE;

	step.weapon = weapon;

	step.animate_status = status;

	step.animate_on_ground = on_ground;

	step.animate_lifetime = lifetime;

	step.animate_motion_y = motion_y;

	return ask_once (&step);
}
