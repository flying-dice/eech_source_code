/* A focused C-level check of the campaign's half of the DAMAGE of a hit (EEDCS-46, real weapon): it compiles the real aphavoc/source/entity/mobile/mb_hostaim.c against a stand-in
   for the engine's project.h and drives campaign_aim_damage with a scripted host.  It asserts what the campaign SENDS (the hit by value), that the campaign takes the ONE sample of
   its random stream the host asks for, at that point and not otherwise, and hands it back; that the host's answer is returned whole (a changed host changes the outcome); and what
   the campaign REFUSES before it applies anything: a second sample, a sample for a hit the invulnerability stops, an answer that contradicts the request (health written under the
   cheat guard or outside [0, the old level], changed without being written, classes the target's model cannot have, a stop for a target that is not invulnerable), an echo that is
   not exactly one agreeing event, and a missing, failing or malformed host.  It checks the campaign's bridge; it is NOT a native run and says nothing about the arithmetic (that is
   the host's: eech-world src/cmb.rs).  Run by run-damage-checks.sh. */
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
static int next_sample = 12345;

int rand16 (void) { draws_taken++; return next_sample; }
float get_delta_time (void) { return 0.1f; }
unsigned int get_system_time (void) { return 0; }
const char *campaign_roster_host_id (entity *en) { return NULL; }
int get_local_entity_index (entity *en) { return 0; }

static campaign_aim_result scripted, last_result;
static campaign_aim_event scripted_echo[4];
static int scripted_echo_count;
static campaign_aim_step seen_first, seen;
static int host_calls, host_fails, asks_for_draw, asks_twice;

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	host_calls++;

	if (host_fails)
	{
		snprintf (error, error_size, "the host has no model for this hit");

		return 1;
	}

	if (host_calls == 1)
	{
		seen_first = *step;
	}

	seen = *step;

	if (asks_twice && step->has_draw)
	{
		memset (result, 0, sizeof (*result));
		result->state = CAMPAIGN_AIM_NEED_DRAW;

		return 0;
	}

	if (asks_for_draw && !step->has_draw)
	{
		memset (result, 0, sizeof (*result));
		result->state = CAMPAIGN_AIM_NEED_DRAW;

		return 0;
	}

	*result = scripted;

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

static campaign_damage_request request;

/* a vehicle hit by a high-explosive weapon, a player, with a health of 80 of 100 */
static void a_hit (void)
{
	memset (&request, 0, sizeof (request));
	request.figures.weapon_warhead = 5;
	request.figures.warhead = 5;
	request.figures.damage_capability = 100;
	request.figures.armor_penetration_capability = 7;
	request.figures.burst_duration = 2.5f;
	request.weapon_speed = 300.0f;
	request.weapon_cruise_velocity = 250.0f;
	request.damage_modifier = 0.5f;
	request.armour_level = 4;
	request.armour_model = CAMPAIGN_DAMAGE_ARMOUR_VEHICLE;
	request.armour_front = 1.0f;
	request.armour_rear = 0.5f;
	request.armour_side = 0.75f;
	request.weapon_position[0] = 1.0f;
	request.weapon_position[1] = 2.0f;
	request.weapon_position[2] = 3.0f;
	request.target_attitude[0] = request.target_attitude[4] = request.target_attitude[8] = 1.0f;
	request.target_not_ai = 1;
	request.player_modifier_valid = 1;
	request.player_modifier = 0.66f;
	request.has_launcher = 1;
	request.multiplier = 2;
	request.damage_level = 80;
	request.initial_damage_level = 100;
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_VEHICLE;
}

/* the host answers: capability, written and the new level, the classes */
static void host_answers (int capability, int written, int new_level, int before, int after)
{
	campaign_aim_host host;

	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	memset (&scripted, 0, sizeof (scripted));
	memset (&seen_first, 0, sizeof (seen_first));
	memset (&seen, 0, sizeof (seen));
	scripted.state = CAMPAIGN_AIM_DONE;
	scripted.damage.capability = capability;
	scripted.damage.written = written;
	scripted.damage.new_level = new_level;
	scripted.damage.critical_before = before;
	scripted.damage.critical_after = after;
	scripted_echo_count = 0;
	memset (scripted_echo, 0, sizeof (scripted_echo));
	host_calls = host_fails = asks_for_draw = asks_twice = 0;
	draws_taken = 0;
	harness_message[0] = '\0';
}

static int ask (int echo)
{
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return 0;
	}

	last_result = campaign_aim_damage (54, &request, echo);

	return 1;
}

static int refused_with (const char *text)
{
	return strstr (harness_message, text) != NULL;
}

/* the raw bits of echo float `index` (0 the damage modifier ... 3 the random modifier) as two exact halves in f[4..11], and back */
static void set_bits (campaign_aim_event *event, int index, unsigned int bits)
{
	event->f[4 + 2 * index] = (float) (bits >> 16);
	event->f[5 + 2 * index] = (float) (bits & 0xFFFFu);
}

static unsigned int echo_bits (const campaign_aim_event *event, int index)
{
	float value = campaign_aim_damage_echo_float (event, index);
	unsigned int bits;

	memcpy (&bits, &value, sizeof (bits));

	return bits;
}

int main (void)
{
	int accepted;

	/* ---- what the campaign sends, the one sample, and what comes back */
	a_hit ();
	host_answers (61, 1, 19, 0, 1);
	asks_for_draw = 1;
	accepted = ask (0);
	check (accepted && host_calls == 2 && draws_taken == 1, "the host asks for its sample once: the campaign takes ONE sample of its stream and asks again");
	check (seen_first.mode == CAMPAIGN_AIM_DAMAGE && !seen_first.has_draw && seen.has_draw && seen.draw_sample == 12345, "the first question carries no sample and the second carries the one taken");
	check (seen.weapon == 54 && seen.damage.figures.weapon_warhead == 5 && seen.damage.figures.damage_capability == 100 && seen.damage.figures.burst_duration == 2.5f && seen.damage.weapon_speed == 300.0f &&
		seen.damage.weapon_cruise_velocity == 250.0f && seen.damage.damage_modifier == 0.5f, "the weapon, its figures, its speed and the blast fraction cross by value");
	check (seen.damage.armour_level == 4 && seen.damage.armour_model == CAMPAIGN_DAMAGE_ARMOUR_VEHICLE && seen.damage.armour_front == 1.0f && seen.damage.armour_rear == 0.5f &&
		seen.damage.armour_side == 0.75f && seen.damage.weapon_position[2] == 3.0f && seen.damage.target_attitude[8] == 1.0f, "the armour data and the geometry cross by value");
	check (seen.damage.target_not_ai && seen.damage.player_modifier_valid && seen.damage.player_modifier == 0.66f && seen.damage.has_launcher && seen.damage.multiplier == 2 &&
		!seen.damage.invulnerable && !seen.damage.cheat_guard && seen.damage.damage_level == 80 && seen.damage.initial_damage_level == 100 && seen.damage.critical_model == CAMPAIGN_DAMAGE_CRITICAL_VEHICLE,
		"the settings and the health cross by value");
	check (last_result.damage.capability == 61 && last_result.damage.written && last_result.damage.new_level == 19 && last_result.damage.critical_before == 0 && last_result.damage.critical_after == 1,
		"the host's answer is what the campaign gets, whole");

	/* a host that needs no sample takes none; a changed host changes the outcome */
	host_answers (7, 1, 73, 0, 0);
	accepted = ask (0);
	check (accepted && host_calls == 1 && draws_taken == 0 && last_result.damage.capability == 7 && last_result.damage.new_level == 73, "a host with a model of its own that asks for none: none is taken, its outcome is used");
	host_answers (0, 0, 80, 0, 0);
	accepted = ask (0);
	check (accepted && last_result.damage.capability == 0 && !last_result.damage.written, "a hit that does nothing is a legal answer");
	host_answers (-2147483647 - 1, 0, 80, 0, 0);
	accepted = ask (0);
	check (accepted, "and so is a capability of zero or less, which the original's arithmetic can give");

	/* ---- the invulnerable player: stopped before the sample */
	a_hit ();
	request.invulnerable = 1;
	host_answers (0, 0, 80, 0, 0);
	scripted.damage.exited = 1;
	accepted = ask (0);
	check (accepted && draws_taken == 0 && last_result.damage.exited, "an invulnerable player's hit is stopped by the host and takes no sample");
	host_answers (0, 0, 80, 0, 0);
	scripted.damage.exited = 1;
	asks_for_draw = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("against an invulnerable target") && draws_taken == 0, "a host that asks for a sample for such a hit is refused and none is taken");
	host_answers (5, 0, 80, 0, 0);
	accepted = ask (0);
	check (!accepted && refused_with ("for a target that is invulnerable"), "a host that hits an invulnerable player is refused");
	host_answers (0, 0, 80, 0, 0);
	scripted.damage.exited = 1;
	scripted.damage.capability = 3;
	accepted = ask (0);
	check (!accepted && refused_with ("invulnerability stopped and still changed something"), "a stopped hit that still changes something is refused");
	a_hit ();
	host_answers (0, 0, 80, 0, 0);
	scripted.damage.exited = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("for a target that is not"), "a stop for a target that is not invulnerable is refused");

	/* ---- the second sample, lookups and a failing host */
	a_hit ();
	host_answers (5, 1, 75, 0, 0);
	asks_for_draw = 1;
	asks_twice = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("already carried one") && draws_taken == 1, "a second request for a sample is refused");
	host_answers (5, 1, 75, 0, 0);
	scripted.state = CAMPAIGN_AIM_NEED_LOOKUP;
	scripted.lookup_count = 1;
	scripted.lookups[0].kind = CAMPAIGN_STEP_LOOKUP_WIND;
	accepted = ask (0);
	check (!accepted && refused_with ("asked for lookups in answer to a damage request") && draws_taken == 0, "a request for a lookup is refused");
	host_answers (5, 1, 75, 0, 0);
	host_fails = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("failed for a damage request") && refused_with ("no model for this hit"), "a failing host stops the engine with its reason");
	campaign_aim_set_host (NULL);
	accepted = ask (0);
	check (!accepted && refused_with ("damage request needs a host aim callback"), "no host: the engine stops with a clear message");

	/* ---- an answer that contradicts the request */
	host_answers (9, 1, 71, 0, 0);
	request.cheat_guard = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("under the cheat guard"), "health written under the cheat guard is refused");
	a_hit ();
	host_answers (9, 1, -1, 0, 0);
	accepted = ask (0);
	check (!accepted && refused_with ("writing health -1 over 80"), "health written below zero is refused");
	host_answers (9, 1, 81, 0, 0);
	accepted = ask (0);
	check (!accepted && refused_with ("writing health 81 over 80"), "health raised is refused");
	host_answers (9, 1, 0, 0, 1);
	accepted = ask (0);
	check (accepted && last_result.damage.new_level == 0, "health taken to exactly zero is legal");
	/* a hit of zero or less capability does nothing (the original writes health only for a capability above zero): a write for one is refused, whatever level it writes */
	host_answers (0, 1, 0, 0, 1);
	accepted = ask (0);
	check (!accepted && refused_with ("which does nothing"), "a write of health 0 for a hit of capability 0 (it would kill a healthy target) is refused");
	host_answers (0, 1, 70, 0, 0);
	accepted = ask (0);
	check (!accepted && refused_with ("which does nothing"), "so is any other write for a hit of capability 0");
	host_answers (-5, 1, 0, 0, 1);
	accepted = ask (0);
	check (!accepted && refused_with ("which does nothing"), "and for a negative capability");
	host_answers (-2147483647 - 1, 1, 79, 0, 0);
	accepted = ask (0);
	check (!accepted && refused_with ("which does nothing"), "and for the most negative one");
	host_answers (-5, 0, 80, 0, 0);
	accepted = ask (0);
	check (accepted && !last_result.damage.written && last_result.damage.new_level == 80, "a negative capability that writes nothing stays a legal no-op answer");
	host_answers (0, 0, 80, 1, 1);
	accepted = ask (0);
	check (accepted, "and so does a zero one, whatever class the unchanged health has");
	host_answers (1, 1, 79, 0, 0);
	accepted = ask (0);
	check (accepted && last_result.damage.written && last_result.damage.new_level == 79, "a capability of 1 may be written");
	host_answers (9, 0, 70, 0, 0);
	accepted = ask (0);
	check (!accepted && refused_with ("changing health or its class without writing it"), "health changed without being written is refused");
	host_answers (9, 0, 80, 0, 1);
	accepted = ask (0);
	check (!accepted && refused_with ("changing health or its class without writing it"), "a class changed without the health is refused");
	a_hit ();
	request.damage_level = -5;
	host_answers (9, 1, 0, 1, 1);
	accepted = ask (0);
	check (accepted && last_result.damage.new_level == 0, "health below zero (never written by the original) is clamped to zero by a legal write");
	a_hit ();
	host_answers (9, 1, 71, 0, 2);
	accepted = ask (0);
	check (!accepted && refused_with ("the target's classes are 0 to 1"), "a vehicle given an aircraft's class is refused");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_AIRCRAFT;
	host_answers (9, 1, 71, 1, 2);
	accepted = ask (0);
	check (accepted && last_result.damage.critical_after == 2, "an aircraft may be critical (2)");
	host_answers (9, 1, 71, 0, 3);
	accepted = ask (0);
	check (!accepted, "and not 3");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_PERSON;
	host_answers (9, 1, 71, 0, 1);
	accepted = ask (0);
	check (!accepted, "a person is never critical");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_NONE;
	host_answers (9, 1, 71, 1, 1);
	accepted = ask (0);
	check (!accepted, "and a fixed structure has no class");

	/* ---- a player with no damage setting */
	a_hit ();
	request.player_modifier_valid = 0;
	host_answers (0, 0, 80, 0, 0);
	scripted.damage.bad_modifier = 1;
	accepted = ask (0);
	check (accepted && last_result.damage.bad_modifier, "a host that needs a damage setting the campaign could not resolve says so (the campaign then stops with the original's message)");
	request.player_modifier_valid = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("missing damage setting that the request carried or did not need"), "it may not say so when the request carried one");
	a_hit ();
	request.target_not_ai = 0;
	request.player_modifier_valid = 0;
	accepted = ask (0);
	check (!accepted, "nor for an AI, which needs none");
	a_hit ();
	request.player_modifier_valid = 0;
	host_answers (9, 1, 71, 0, 0);
	accepted = ask (0);
	check (accepted, "a host that does not use the setting at all is not made to");

	/* ---- the fixture build's echo */
	a_hit ();
	host_answers (31, 1, 49, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].a = 2;
	scripted_echo[0].b = 31;
	scripted_echo[0].c = 5;
	scripted_echo[0].d = 31;
	scripted_echo[0].f[3] = 0.75f;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (1);
	check (accepted && seen.echo == 1 && last_result.has_echo && last_result.echo_count == 1 && last_result.echo[0].kind == CAMPAIGN_AIM_EVENT_DAMAGE && last_result.echo[0].f[3] == 0.75f,
		"one damage event that agrees with the answer is accepted and handed on whole");
	host_answers (31, 1, 49, 0, 0);
	accepted = ask (1);
	check (accepted && !last_result.has_echo, "an echo the host does not give is not an error");
	host_answers (31, 1, 49, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].d = 31;
	scripted_echo[1] = scripted_echo[0];
	scripted_echo_count = 2;
	scripted.has_echo = 1;
	scripted.echo_count = 2;
	accepted = ask (1);
	check (!accepted && refused_with ("not one event agreeing with its answer"), "two damage events are refused");
	host_answers (31, 1, 49, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_FORECAST;
	scripted_echo[0].d = 31;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (1);
	check (!accepted, "an event of another kind is refused");
	host_answers (31, 1, 49, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].d = 30;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (1);
	check (!accepted && refused_with ("not one event agreeing with its answer"), "an echo whose capability disagrees with the answer is refused");
	host_answers (31, 1, 49, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].d = 31;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (0);
	check (!accepted && refused_with ("echo nobody asked for"), "an echo nobody asked for is refused");
	/* the raw-bit transport of the four intermediate floats: a centred hit has a not-a-number aspect armour and does nothing; the echo is finite floats and gives the bits back */
	a_hit ();
	host_answers (0, 0, 80, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].a = 0;
	scripted_echo[0].c = 3;
	scripted_echo[0].d = 0;
	set_bits (&scripted_echo[0], 0, 0x00000000u);	/* the damage modifier 0.0 */
	set_bits (&scripted_echo[0], 1, 0x41700000u);	/* the penetration 15.0 */
	set_bits (&scripted_echo[0], 2, 0xFFC00000u);	/* the armour: the not-a-number x86 gives 0 / 0, sign set */
	set_bits (&scripted_echo[0], 3, 0x3F400000u);	/* the random modifier 0.75 */
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (1);
	check (accepted && last_result.has_echo && last_result.damage.capability == 0 && !last_result.damage.written, "a centred hit's echo (a not-a-number armour as raw bits) is accepted: the hit does nothing and writes nothing");
	check (accepted && echo_bits (&last_result.echo[0], 2) == 0xFFC00000u && echo_bits (&last_result.echo[0], 1) == 0x41700000u && echo_bits (&last_result.echo[0], 3) == 0x3F400000u && echo_bits (&last_result.echo[0], 0) == 0u,
		"and the four values come back bit for bit, the not-a-number with its sign");
	check (accepted && isnan (campaign_aim_damage_echo_float (&last_result.echo[0], 2)) && campaign_aim_damage_echo_float (&last_result.echo[0], 3) == 0.75f, "as the value the observer saw");
	{
		/* every pattern that matters, through the same two halves */
		static const unsigned int patterns[] = { 0x7FC12345u, 0xFFC00000u, 0xFFFFFFFFu, 0x80000000u, 0x7F800000u, 0xFF800000u, 0x00000001u, 0x7F7FFFFFu, 0x3F400000u, 0x00000000u };
		campaign_aim_event event;
		int k, index, same = 1;

		for (k = 0; k < (int) (sizeof (patterns) / sizeof (patterns[0])); k++)
		{
			memset (&event, 0, sizeof (event));

			for (index = 0; index < 4; index++)
			{
				set_bits (&event, index, patterns[(k + index) % (int) (sizeof (patterns) / sizeof (patterns[0]))]);
			}

			for (index = 0; index < 4; index++)
			{
				same = same && (echo_bits (&event, index) == patterns[(k + index) % (int) (sizeof (patterns) / sizeof (patterns[0]))]);
			}
		}

		check (same, "every bit pattern that matters (payloads, signs, zero, infinities, a denormal, the largest finite) survives the halves");
	}
	host_answers (0, 0, 80, 0, 0);
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].d = 0;
	scripted_echo[0].f[8] = 65536.0f;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (1);
	check (!accepted && refused_with ("raw-bit half 8"), "a raw-bit half above 65535 is refused");
	scripted_echo[0].f[8] = -1.0f;
	accepted = ask (1);
	check (!accepted && refused_with ("raw-bit half 8"), "or below zero");
	scripted_echo[0].f[8] = 1.5f;
	accepted = ask (1);
	check (!accepted && refused_with ("raw-bit half 8"), "or not an integer");
	scripted_echo[0].f[8] = 0.0f;
	scripted_echo[0].f[2] = NAN;
	accepted = ask (1);
	check (!accepted && refused_with ("malformed echo event"), "a not-a-number in the readable floats is still refused: finite validation is not weakened");
	scripted_echo[0].f[2] = 0.0f;
	scripted_echo[0].f[5] = INFINITY;
	accepted = ask (1);
	check (!accepted, "and an infinity in a half");
	a_hit ();
	request.invulnerable = 1;
	host_answers (0, 0, 80, 0, 0);
	scripted.damage.exited = 1;
	scripted_echo[0].kind = CAMPAIGN_AIM_EVENT_DAMAGE;
	scripted_echo[0].d = 0;
	scripted_echo_count = 1;
	scripted.has_echo = 1;
	scripted.echo_count = 1;
	accepted = ask (1);
	check (!accepted && refused_with ("not one event agreeing with its answer"), "a stopped hit echoes nothing");

	/* ---- a malformed request never reaches the host */
	a_hit ();
	host_answers (9, 1, 71, 0, 0);
	request.armour_model = 5;
	accepted = ask (0);
	check (!accepted && refused_with ("inconsistent figures for weapon 54") && host_calls == 0, "an armour model that does not exist is refused before the host is asked");
	a_hit ();
	request.critical_model = 9;
	accepted = ask (0);
	check (!accepted && host_calls == 0, "so is a critical model");
	a_hit ();
	request.invulnerable = 2;
	accepted = ask (0);
	check (!accepted && host_calls == 0, "a flag that is not a boolean");
	a_hit ();
	request.target_position[1] = NAN;
	accepted = ask (0);
	check (!accepted && host_calls == 0, "a non-finite position");
	a_hit ();
	request.target_attitude[4] = INFINITY;
	accepted = ask (0);
	check (!accepted && host_calls == 0, "a non-finite attitude");
	a_hit ();
	request.weapon_speed = NAN;
	accepted = ask (0);
	check (!accepted && host_calls == 0, "a non-finite speed");
	a_hit ();
	request.initial_damage_level = 0;
	accepted = ask (0);
	check (!accepted && host_calls == 0, "a classified health with no initial level to divide by");
	request.critical_model = CAMPAIGN_DAMAGE_CRITICAL_PERSON;
	accepted = ask (0);
	check (accepted, "but a person, who is not classified, has none to need");
	a_hit ();
	request.armour_level = -1;
	accepted = ask (0);
	check (accepted, "an armour level of any sign is a legal question (the original only logged a zero)");

	printf ("DAMAGE_CHECKS=%s: %d checks, %d failures\n", failures ? "FAILED" : "ok", checks, failures);

	return failures ? 1 : 0;
}
