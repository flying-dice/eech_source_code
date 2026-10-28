/* A focused C-level check of the campaign's half of the damage FORECAST (EEDCS-46): it compiles the real aphavoc/source/entity/mobile/mb_hostaim.c against a stand-in for the
   engine's project.h and drives campaign_aim_forecast with a scripted host.  It asserts what the campaign SENDS (the figures and flags by value, the two arguments of the
   original function), that the host's answer is what comes back (a changed host changes the capability), that a forecast takes NO sample of the shared random stream and refuses
   a host that asks for one or for a lookup, and that a missing, failing or malformed host stops the engine with a clear message before the answer is used.  It checks the
   campaign's bridge; it is NOT a native run and says nothing about the arithmetic (that is the host's: eech-world src/cmb.rs).  Run by run-forecast-checks.sh. */
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

int rand16 (void) { draws_taken++; return 0; }
float get_delta_time (void) { return 0.1f; }
unsigned int get_system_time (void) { return 0; }
const char *campaign_roster_host_id (entity *en) { return NULL; }
const char *campaign_object_identify (entity *en) { return campaign_roster_host_id (en); }
int get_local_entity_index (entity *en) { return 0; }

static campaign_aim_result scripted, last_result;
static campaign_aim_event scripted_echo[4];
static int scripted_echo_count;
static campaign_aim_step seen;
static int host_calls, host_fails;

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	host_calls++;

	if (host_fails)
	{
		snprintf (error, error_size, "the host has no model for this weapon");

		return 1;
	}

	seen = *step;

	*result = scripted;

	/* the campaign gives the host room for its echo at echo_out (only when it asked for one); the scripted host writes its events there */
	if (scripted_echo_count && step->echo_out)
	{
		memcpy (step->echo_out, scripted_echo, (size_t) scripted_echo_count * sizeof (campaign_aim_event));
	}

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

static void host_answers (int capability)
{
	campaign_aim_host host;

	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	memset (&scripted, 0, sizeof (scripted));
	memset (&seen, 0, sizeof (seen));
	scripted.state = CAMPAIGN_AIM_DONE;
	scripted.forecast_capability = capability;
	scripted_echo_count = 0;
	memset (scripted_echo, 0, sizeof (scripted_echo));
	host_calls = 0;
	host_fails = 0;
	draws_taken = 0;
	harness_message[0] = '\0';
}

static campaign_damage_figures figures (void)
{
	campaign_damage_figures f;

	f.weapon_warhead = 8;
	f.warhead = 5;
	f.damage_capability = 30;
	f.armor_penetration_capability = 7;
	f.burst_duration = 2.5f;

	return f;
}

/* asks one forecast of weapon 54; returns whether the campaign accepted the exchange (0 when the engine stopped, with the message), and the capability in *capability */
static int ask (int armour, int not_ai, float player, float modifier, float velocity, int echo, int *capability)
{
	campaign_damage_figures f = figures ();
	campaign_aim_result result;

	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	result = campaign_aim_forecast (54, &f, armour, not_ai, player, modifier, velocity, echo);

	last_result = result;

	if (capability)
	{
		*capability = result.forecast_capability;
	}

	return 1;
}

static int refused_with (const char *text)
{
	return strstr (harness_message, text) != NULL;
}

int main (void)
{
	int capability = -1, accepted;

	/* ---- what the campaign sends, by value, and what comes back */
	host_answers (41);
	accepted = ask (9, 1, 0.66f, 0.0f, 1.0f, 0, &capability);
	check (accepted && capability == 41 && host_calls == 1, "the host's own capability is what the campaign gets back");
	check (seen.mode == CAMPAIGN_AIM_FORECAST && seen.weapon == 54 && seen.forecast_figures.weapon_warhead == 8 && seen.forecast_figures.warhead == 5 && seen.forecast_figures.damage_capability == 30 &&
		seen.forecast_figures.armor_penetration_capability == 7 && seen.forecast_figures.burst_duration == 2.5f, "the weapon and its damage figures cross by value");
	check (seen.forecast_armour_level == 9 && seen.forecast_target_not_ai == 1 && seen.forecast_player_modifier == 0.66f && seen.forecast_damage_modifier == 0.0f && seen.forecast_velocity == 1.0f,
		"the armour level, the player flag and setting and the two arguments of the original cross by value");
	check (seen.echo == 0 && seen.echo_out == NULL, "no echo is asked for outside the fixture build");

	/* a changed host changes the capability: the campaign uses the answer, it computes none */
	host_answers (7);
	accepted = ask (9, 0, 1.0f, 0.0f, 1.0f, 0, &capability);
	check (accepted && capability == 7, "a host with another model gives another capability, used as it is");
	host_answers (0);
	accepted = ask (9, 0, 1.0f, 0.0f, 1.0f, 0, &capability);
	check (accepted && capability == 0, "and a capability of zero is a legal answer");

	/* ---- a forecast takes no sample of the shared random stream */
	host_answers (5);
	(void) ask (4, 0, 1.0f, 0.5f, 1.0f, 0, &capability);
	(void) ask (4, 1, 0.33f, 0.5f, 1.0f, 0, &capability);
	check (draws_taken == 0, "accepted forecasts took no sample of the shared stream");
	host_answers (5);
	scripted.state = CAMPAIGN_AIM_NEED_DRAW;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("asked for a draw in answer to a forecast request") && draws_taken == 0, "a host that asks for a sample is refused and no sample is taken");
	host_answers (5);
	scripted.state = CAMPAIGN_AIM_NEED_LOOKUP;
	scripted.lookup_count = 1;
	scripted.lookups[0].kind = CAMPAIGN_STEP_LOOKUP_WIND;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("asked for lookups in answer to a forecast request") && draws_taken == 0, "a host that asks for a lookup is refused");

	/* ---- a missing, failing or malformed host stops the engine before the answer is used */
	campaign_aim_set_host (NULL);
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("forecast request needs a host aim callback"), "no host: the engine stops with a clear message");
	host_answers (5);
	host_fails = 1;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("failed for a forecast request") && refused_with ("no model for this weapon"), "a host that fails: the engine stops with the host's reason");
	host_answers (-1);
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("negative capability"), "a negative capability is refused");
	host_answers (5);
	scripted.state = 99;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("answered state 99"), "an unknown state is refused");
	host_answers (5);
	scripted.has_time_of_flight = 1;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("time of flight it cannot carry"), "an answer that carries a time of flight, which a forecast cannot, is refused");

	/* ---- a malformed request never reaches the host */
	host_answers (5);
	accepted = ask (-1, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("inconsistent figures for weapon 54") && host_calls == 0, "an armour level below zero is refused before the host is asked");
	accepted = ask (4, 2, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && host_calls == 0, "a player flag that is not a boolean is refused");
	accepted = ask (4, 0, NAN, 0.5f, 1.0f, 0, NULL);
	check (!accepted && host_calls == 0, "a non-finite player setting is refused");
	accepted = ask (4, 0, 1.0f, INFINITY, 1.0f, 0, NULL);
	check (!accepted && host_calls == 0, "a non-finite damage modifier is refused");
	accepted = ask (4, 0, 1.0f, 0.5f, NAN, 0, NULL);
	check (!accepted && host_calls == 0, "a non-finite velocity is refused");

	/* ---- the fixture build's echo: one forecast event that agrees with the answer, or none */
	host_answers (31);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_FORECAST;
	scripted_echo[0].a = 2;
	scripted_echo[0].b = 31;
	scripted_echo[0].c = 5;
	scripted_echo[0].d = 31;
	scripted_echo[0].f[0] = 0.31429072f;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 1, &capability);
	check (accepted && capability == 31 && seen.echo == 1 && last_result.has_echo && last_result.echo_count == 1 && last_result.echo[0].kind == CAMPAIGN_AIM_EVENT_FORECAST &&
		last_result.echo[0].a == 2 && last_result.echo[0].b == 31 && last_result.echo[0].c == 5 && last_result.echo[0].f[0] == 0.31429072f, "one forecast event that agrees with the answer is accepted and handed on whole");

	host_answers (31);
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 1, &capability);
	check (accepted && capability == 31 && !last_result.has_echo, "an echo the host does not give is not an error (a host with a model of its own has none)");

	host_answers (31);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_FORECAST;
	scripted_echo[0].d = 31;
	scripted_echo[1] = scripted_echo[0];
	scripted_echo_count = 2;
	scripted.has_echo = 1;
	scripted.echo_count = 2;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 1, NULL);
	check (!accepted && refused_with ("not one event agreeing with its answer"), "two forecast events are refused");

	host_answers (31);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_INTERCEPT;
	scripted_echo[0].d = 31;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 1, NULL);
	check (!accepted && refused_with ("not one event agreeing with its answer"), "an event of another kind is refused");

	host_answers (31);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_FORECAST;
	scripted_echo[0].d = 30;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 1, NULL);
	check (!accepted && refused_with ("not one event agreeing with its answer"), "an echo whose capability disagrees with the answer is refused");

	host_answers (31);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_FORECAST;
	scripted_echo[0].d = 31;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (4, 0, 1.0f, 0.5f, 1.0f, 0, NULL);
	check (!accepted && refused_with ("echo nobody asked for"), "an echo nobody asked for is refused");

	printf ("FORECAST_CHECKS=%s: %d checks, %d failures\n", failures ? "FAILED" : "ok", checks, failures);

	return failures ? 1 : 0;
}
