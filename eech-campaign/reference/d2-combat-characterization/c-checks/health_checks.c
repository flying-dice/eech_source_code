/* A focused C-level check of the campaign's half of the HEALTH questions (EEDCS-46: the critical class, what a change of health crosses, the six timed health rolls and the helicopter's
   own-hit eligibility): it compiles the real aphavoc/source/entity/mobile/mb_hostaim.c against a stand-in for the engine's project.h and drives campaign_aim_health with a scripted host.
   It asserts what the campaign SENDS (by value), that the host's answer is returned whole (a changed host changes the outcome), that the campaign takes the ONE sample of its random
   stream a timed roll asks for only when the host asks for it, once, and for no target the original did not draw for (a person, an aircraft in no class, a vehicle with no health or
   that cannot burn), and what the campaign REFUSES before it applies anything: an answer that contradicts the request (an ineligible target rolled or changed, a hit without
   eligibility, a roll without the sample, a hit that leaves the wrong health, a fatal that does not match, a class the model cannot have, a pure question answered with more) and a
   missing, failing or malformed host.  It checks the campaign's bridge; it is NOT a native run and says nothing about the arithmetic (the host's: eech-world src/cmb.rs).  Run by
   run-health-checks.sh. */
#include "project.h"

#include "mb_hostaim.h"

jmp_buf harness_fatal;
char harness_message[512];

void debug_fatal (const char *format, ...)
{
	va_list arguments;

	va_start (arguments, format);
	vsnprintf (harness_message, sizeof (harness_message), format, arguments);
	va_end (arguments);
	longjmp (harness_fatal, 1);
}

static int draws_taken;
static int next_sample = 4321;

int rand16 (void) { draws_taken++; return next_sample; }
float get_delta_time (void) { return 0.1f; }
unsigned int get_system_time (void) { return 0; }
const char *campaign_roster_host_id (entity *en) { return NULL; }
int get_local_entity_index (entity *en) { return 0; }

static campaign_aim_result scripted, last_result;
static campaign_aim_step seen_first, seen;
static int host_calls, host_fails, asks_for_draw, asks_twice;

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	host_calls++;

	if (host_fails)
	{
		snprintf (error, error_size, "the host has no model for this health");

		return 1;
	}

	if (host_calls == 1)
	{
		seen_first = *step;
	}

	seen = *step;

	if ((asks_twice && step->has_draw) || (asks_for_draw && !step->has_draw))
	{
		memset (result, 0, sizeof (*result));
		result->state = CAMPAIGN_AIM_NEED_DRAW;

		return 0;
	}

	*result = scripted;

	return 0;
}

static int failures, checks;

static void check (int condition, const char *what)
{
	checks++;

	if (!condition)
	{
		failures++;
		printf ("FAIL %s\n", what);
	}
	else
	{
		printf ("ok   %s\n", what);
	}
}

static campaign_health_request request;

static void host_answers (void)
{
	campaign_aim_host host;

	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	memset (&scripted, 0, sizeof (scripted));
	scripted.state = CAMPAIGN_AIM_DONE;
	memset (&seen_first, 0, sizeof (seen_first));
	memset (&seen, 0, sizeof (seen));
	host_calls = host_fails = asks_for_draw = asks_twice = 0;
	draws_taken = 0;
	harness_message[0] = '\0';
}

static int ask (void)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	last_result = campaign_aim_health (&request);

	return 1;
}

static int refused_with (const char *text)
{
	return strstr (harness_message, text) != NULL;
}

/* a request of `op` about a vehicle with health 14 of 100 that can burn, 0.1 of a second long */
static void a_request (int op, int model, int level, int initial)
{
	memset (&request, 0, sizeof (request));
	request.op = op;
	request.critical_model = model;
	request.damage_level = level;
	request.old_damage_level = level;
	request.initial_damage_level = initial;
	request.explosive = 1;
	request.delta_time = 0.1f;
}

int main (void)
{
	int accepted;

	/* ---- CLASSIFY: a pure question */
	a_request (CAMPAIGN_HEALTH_CLASSIFY, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	host_answers ();
	scripted.health.critical_class = 1;
	accepted = ask ();
	check (accepted && host_calls == 1 && draws_taken == 0 && last_result.health.critical_class == 1, "a classification is one question, takes no sample and returns the host's class");
	check (seen.mode == CAMPAIGN_AIM_HEALTH && seen.health.op == CAMPAIGN_HEALTH_CLASSIFY && seen.health.critical_model == CAMPAIGN_DAMAGE_CRITICAL_VEHICLE && seen.health.damage_level == 14 &&
		seen.health.initial_damage_level == 100, "the model, the level and the initial level cross by value");
	host_answers ();
	scripted.health.critical_class = 0;
	accepted = ask ();
	check (accepted && last_result.health.critical_class == 0, "a changed host gives another class, used as given");
	host_answers ();
	scripted.health.critical_class = 2;
	accepted = ask ();
	check (!accepted && refused_with ("outside 0 to 1"), "a vehicle given an aircraft's class is refused");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT;
	accepted = ask ();
	check (accepted && last_result.health.critical_class == 2, "an aircraft may be critical (2)");
	scripted.health.critical_class = 3;
	accepted = ask ();
	check (!accepted, "and not 3");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_PERSON;
	scripted.health.critical_class = 1;
	accepted = ask ();
	check (!accepted, "a person is never critical");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_NONE;
	request.initial_damage_level = 0;
	scripted.health.critical_class = 0;
	accepted = ask ();
	check (accepted, "a fixed structure with no initial level is a legal question with class 0");
	a_request (CAMPAIGN_HEALTH_CLASSIFY, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	scripted.health.critical_class = 0;
	scripted.health.hit = 1;
	accepted = ask ();
	check (!accepted && refused_with ("more than a class"), "a pure question answered with more than a class is refused");

	/* ---- ASSESS */
	a_request (CAMPAIGN_HEALTH_ASSESS, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	request.old_damage_level = 16;
	host_answers ();
	scripted.health.crossed = 1;
	accepted = ask ();
	check (accepted && last_result.health.crossed && seen.health.old_damage_level == 16 && seen.health.explosive, "a vehicle that can burn crosses the threshold, and the change crosses by value");
	request.explosive = 0;
	accepted = ask ();
	check (!accepted && refused_with ("health assessment"), "a vehicle that cannot burn crosses nothing");
	a_request (CAMPAIGN_HEALTH_ASSESS, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	scripted.health.recovered = 1;
	accepted = ask ();
	check (!accepted, "only an aircraft recovers");
	a_request (CAMPAIGN_HEALTH_ASSESS, CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT, 290, 1000);
	request.old_damage_level = 400;
	host_answers ();
	scripted.health.crossed = 1;
	scripted.health.eject = 0;
	accepted = ask ();
	check (accepted && last_result.health.crossed, "an aircraft crosses a threshold");
	host_answers ();
	scripted.health.recovered = 1;
	scripted.health.eject = 1;
	accepted = ask ();
	check (accepted && last_result.health.recovered && last_result.health.eject, "and may recover and be in the health that ejects a crew");
	scripted.health.crossed = 1;
	accepted = ask ();
	check (!accepted, "but not cross a threshold and recover in one change");
	a_request (CAMPAIGN_HEALTH_ASSESS, CAMPAIGN_DAMAGE_CRITICAL_PERSON, 14, 100);
	request.old_damage_level = 16;
	host_answers ();
	scripted.health.crossed = 1;
	accepted = ask ();
	check (accepted, "a person that can burn is judged by the threshold arithmetic like any vehicle");

	/* ---- TIMED: the sample */
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	host_answers ();
	asks_for_draw = 1;
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.hit = 1;
	scripted.health.new_level = 13;
	scripted.health.critical_class = 1;
	accepted = ask ();
	check (accepted && host_calls == 2 && draws_taken == 1 && seen_first.has_draw == 0 && seen.has_draw && seen.draw_sample == 4321, "an eligible target asks for its sample once: the campaign takes ONE and asks again");
	check (last_result.health.hit && last_result.health.new_level == 13 && !last_result.health.fatal && last_result.health.critical_class == 1, "the host's roll, the health it leaves and the class after are what the campaign gets");
	check (seen.health.op == CAMPAIGN_HEALTH_TIMED && seen.health.explosive && !seen.health.flammable && seen.health.delta_time == 0.1f && seen.health.damage_level == 14, "the frame time and the burning flags cross by value");
	host_answers ();
	scripted.health.eligible = 1;
	scripted.health.rolled = 0;
	scripted.health.hit = 1;
	scripted.health.new_level = 13;
	accepted = ask ();
	check (accepted && draws_taken == 0, "a host with a roll of its own that asks for no sample takes none");
	host_answers ();
	asks_for_draw = 1;
	asks_twice = 1;
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.new_level = 14;
	accepted = ask ();
	check (!accepted && refused_with ("already carried one") && draws_taken == 1, "a second request for a sample is refused");
	host_answers ();
	scripted.state = CAMPAIGN_AIM_NEED_LOOKUP;
	scripted.lookup_count = 1;
	scripted.lookups[0].kind = CAMPAIGN_STEP_LOOKUP_WIND;
	accepted = ask ();
	check (!accepted && refused_with ("asked for lookups in answer to a health request"), "a lookup is refused");

	/* the eligibility to roll is the HOST's: the campaign does not veto a sample by the reference host's rules.  A host with a model of its own may roll for a person with health (a person
	   is never critical in the reference, which takes no sample), for an aircraft in no class, for a vehicle that cannot burn and for one with no health; it takes its one sample and its
	   answer is applied like any other.  These are the controls the reference rules once stood in front of */
	for (accepted = 0; accepted < 4; accepted++)
	{
		if (accepted == 0)
		{
			a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_PERSON, 5, 100);
		}
		else if (accepted == 1)
		{
			a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT, 5, 100);
			request.aircraft_class = 0;
		}
		else if (accepted == 2)
		{
			a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 5, 100);
			request.explosive = 0;
		}
		else
		{
			a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 0, 100);
		}

		host_answers ();
		asks_for_draw = 1;
		scripted.health.eligible = 1;
		scripted.health.rolled = 1;
		scripted.health.hit = 1;
		scripted.health.new_level = (request.damage_level > 0) ? request.damage_level - 1 : 0;
		scripted.health.fatal = (scripted.health.new_level <= 0);

		check (ask () && draws_taken == 1 && host_calls == 2 && seen.has_draw && seen.draw_sample == 4321 && last_result.health.hit && last_result.health.new_level == scripted.health.new_level,
			accepted == 0 ? "a host's own stochastic bleeding for a person with health is accepted: it takes its ONE sample and its hit is returned" :
			accepted == 1 ? "a host's own degradation of an aircraft in class 0 is accepted the same way" :
			accepted == 2 ? "a host's own timed physics for a vehicle that cannot burn is accepted the same way" : "and for a vehicle with no health (a hit on none leaves none)");
	}

	/* ... and the same hosts that answer without a sample are accepted too (the reference host does for all of them) */
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_PERSON, 5, 100);
	host_answers ();
	scripted.health.new_level = 5;
	accepted = ask ();
	check (accepted && draws_taken == 0 && !last_result.health.eligible, "the reference host's answer for a person (ineligible, no sample, nothing changes) is accepted");

	/* the structure of a timed question is the campaign's: one sample at most, an answer that is consistent and safe to apply, and no such question for a fixed structure */
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_PERSON, 5, 100);
	host_answers ();
	asks_for_draw = 1;
	asks_twice = 1;
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.new_level = 5;
	accepted = ask ();
	check (!accepted && refused_with ("already carried one") && draws_taken == 1, "a second sample is refused for a person as for anything");

	/* a pure question may not draw: a classification, an assessment and a hit eligibility */
	for (accepted = 0; accepted < 3; accepted++)
	{
		a_request (accepted == 0 ? CAMPAIGN_HEALTH_CLASSIFY : accepted == 1 ? CAMPAIGN_HEALTH_ASSESS : CAMPAIGN_HEALTH_HIT_ELIGIBLE, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
		request.self_hit = 1;
		host_answers ();
		asks_for_draw = 1;
		check (!ask () && refused_with ("which is a pure question") && draws_taken == 0,
			accepted == 0 ? "a classification may not ask for a sample" : accepted == 1 ? "an assessment may not" : "a hit eligibility may not");
	}

	/* ineligible targets: no sample, nothing changes */
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 50, 100);
	host_answers ();
	scripted.health.new_level = 50;
	accepted = ask ();
	check (accepted && !last_result.health.eligible && !last_result.health.hit && draws_taken == 0, "an ineligible target (here not critical) takes no sample and is not changed");
	scripted.health.new_level = 49;
	accepted = ask ();
	check (!accepted && refused_with ("timed health request"), "an ineligible target whose health changed is refused");
	scripted.health.new_level = 50;
	scripted.health.rolled = 1;
	accepted = ask ();
	check (!accepted, "an ineligible target that rolled is refused");
	scripted.health.rolled = 0;
	scripted.health.hit = 1;
	accepted = ask ();
	check (!accepted, "a hit on an ineligible target is refused");

	/* hits */
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 1, 100);
	host_answers ();
	asks_for_draw = 1;
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.hit = 1;
	scripted.health.new_level = 0;
	scripted.health.fatal = 1;
	scripted.health.critical_class = 1;
	accepted = ask ();
	check (accepted && last_result.health.fatal && last_result.health.new_level == 0, "the last point of health is fatal");
	scripted.health.fatal = 0;
	accepted = ask ();
	check (!accepted, "a hit that leaves no health and is not fatal is refused");
	scripted.health.fatal = 1;
	scripted.health.new_level = 1;
	accepted = ask ();
	check (!accepted, "a hit that leaves health no lower is refused");
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	host_answers ();
	asks_for_draw = 1;
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.hit = 1;
	scripted.health.new_level = 13;
	scripted.health.fatal = 1;
	accepted = ask ();
	check (!accepted, "a fatal hit that leaves health is refused");
	scripted.health.fatal = 0;
	scripted.health.new_level = -1;
	accepted = ask ();
	check (!accepted, "health below zero is refused");
	scripted.health.new_level = 14;
	scripted.health.hit = 0;
	accepted = ask ();
	check (accepted && !last_result.health.hit && last_result.health.rolled, "a roll that misses leaves the health as it was");
	scripted.health.rolled = 0;
	accepted = ask ();
	check (!accepted && refused_with ("timed health request"), "a host that was given the sample and did not roll is refused");
	host_answers ();
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.new_level = 14;
	accepted = ask ();
	check (!accepted, "a roll that took no sample is refused");

	/* aircraft: the class crosses by value and the hit is one point */
	a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT, 5, 100);
	request.aircraft_class = 2;
	host_answers ();
	asks_for_draw = 1;
	scripted.health.eligible = 1;
	scripted.health.rolled = 1;
	scripted.health.hit = 1;
	scripted.health.new_level = 4;
	scripted.health.critical_class = 2;
	accepted = ask ();
	check (accepted && seen.health.aircraft_class == 2 && last_result.health.new_level == 4, "an aircraft in a class asks for its sample and degrades by one point");

	/* ---- HIT_ELIGIBLE */
	a_request (CAMPAIGN_HEALTH_HIT_ELIGIBLE, CAMPAIGN_DAMAGE_CRITICAL_NONE, 0, 0);
	request.self_hit = 1;
	host_answers ();
	scripted.health.eligible = 0;
	accepted = ask ();
	check (accepted && !last_result.health.eligible && seen.health.self_hit && !seen.health.target_not_ai && draws_taken == 0, "a self hit by an AI is refused by the host, with no sample");
	scripted.health.eligible = 1;
	request.target_not_ai = 1;
	accepted = ask ();
	check (accepted && last_result.health.eligible && seen.health.target_not_ai, "and allowed for a player");
	request.self_hit = 0;
	scripted.health.eligible = 0;
	accepted = ask ();
	check (!accepted && refused_with ("not the target's own"), "a host may not refuse a hit that is not the target's own");
	request.self_hit = 1;
	scripted.health.eligible = 0;
	scripted.health.hit = 1;
	accepted = ask ();
	check (!accepted, "nor answer more than yes or no");

	/* ---- hosts */
	a_request (CAMPAIGN_HEALTH_CLASSIFY, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
	host_answers ();
	host_fails = 1;
	accepted = ask ();
	check (!accepted && refused_with ("failed for a health request") && refused_with ("no model for this health"), "a failing host stops the engine with its reason");
	campaign_aim_set_host (NULL);
	accepted = ask ();
	check (!accepted && refused_with ("health request needs a host aim callback"), "no host: the engine stops with a clear message");

	/* ---- a malformed request never reaches the host */
	for (accepted = 0; accepted < 8; accepted++)
	{
		a_request (CAMPAIGN_HEALTH_TIMED, CAMPAIGN_DAMAGE_CRITICAL_VEHICLE, 14, 100);
		host_answers ();

		switch (accepted)
		{
			case 0: request.op = 0; break;
			case 1: request.op = 5; break;
			case 2: request.critical_model = 9; break;
			case 3: request.explosive = 2; break;
			case 4: request.delta_time = NAN; break;
			case 5: request.aircraft_class = 3; break;
			case 6: request.initial_damage_level = 0; break;
			default: request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_NONE; break;
		}

		check (!ask () && refused_with ("inconsistent figures") && host_calls == 0,
			accepted == 0 ? "op 0 never reaches the host" : accepted == 1 ? "an op that does not exist" : accepted == 2 ? "a model that does not exist" : accepted == 3 ? "a flag that is not a boolean" :
			accepted == 4 ? "a non-finite frame time" : accepted == 5 ? "an aircraft class that does not exist" : accepted == 6 ? "a classified health with no initial level" : "a timed roll for a fixed structure");
	}

	a_request (CAMPAIGN_HEALTH_ASSESS, CAMPAIGN_DAMAGE_CRITICAL_NONE, 5, 0);
	host_answers ();
	accepted = ask ();
	check (!accepted && host_calls == 0, "an assessment divides by the initial level: none is refused");

	printf ("HEALTH_CHECKS=%s: %d checks, %d failures\n", failures ? "FAILED" : "ok", checks, failures);

	return failures ? 1 : 0;
}
