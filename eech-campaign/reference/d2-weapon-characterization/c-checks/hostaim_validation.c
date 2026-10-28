/* A focused C-level check of what the campaign accepts from the host in the step protocol (EEDCS-45): it compiles the real aphavoc/source/entity/mobile/mb_hostaim.c against
   a stand-in for the engine's project.h and drives campaign_aim_weapon_step with a scripted host.  It asserts the strict boolean treatment of the decisions the campaign acts on
   (a decoy's restart, a seduction's two decisions, a reacquisition's one) and that nothing else about a step's actions is narrowed by it.  Run by run-hostaim-validation.sh. */
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

int rand16 (void) { return 0; }
float get_delta_time (void) { return 0.1f; }
unsigned int get_system_time (void) { return 0; }
const char *campaign_roster_host_id (entity *en) { return NULL; }
int get_local_entity_index (entity *en) { return 0; }

static campaign_aim_result scripted;

static int host_aim (const campaign_aim_step *step, campaign_aim_result *result, char *error, size_t error_size, void *user)
{
	*result = scripted;
	return 0;
}

static void no_lookups (const campaign_aim_event *lookup, campaign_aim_event *answer, void *context)
{
	debug_fatal ("the scripted host asked a lookup");
}

static int failures;

/* asks the step of `mode` with a host that answers one action of `kind` with decisions a, b; returns whether the campaign accepted the answer (and the stopping message when not) */
static int ask (int mode, int kind, int a, int b, int actions)
{
	campaign_aim_step step;
	campaign_aim_host host;

	memset (&step, 0, sizeof (step));
	memset (&scripted, 0, sizeof (scripted));
	memset (&host, 0, sizeof (host));
	host.aim = host_aim;
	campaign_aim_set_host (&host);
	step.mode = mode;
	step.weapon = 10;
	step.weapon_state.other_weapon = 20;
	scripted.state = CAMPAIGN_AIM_DONE;
	scripted.action_count = actions;
	scripted.actions[0].kind = kind;
	scripted.actions[0].a = a;
	scripted.actions[0].b = b;
	harness_message[0] = '\0';

	if (setjmp (harness_fatal))
	{
		return FALSE;
	}

	(void) campaign_aim_weapon_step (&step, no_lookups, NULL);

	return TRUE;
}

static void expect (const char *what, int accepted, int wanted)
{
	if (accepted != wanted)
	{
		printf ("FAIL %s: %s (%s)\n", what, accepted ? "accepted" : "refused", harness_message);
		failures++;
	}
	else
	{
		printf ("ok   %s: %s%s%s\n", what, accepted ? "accepted" : "refused", accepted ? "" : " - ", accepted ? "" : harness_message);
	}
}

int main (void)
{
	int a, b;
	const int flags[] = { -2, -1, 0, 1, 2, 3, 256 };
	char name[96];

	for (a = 0; a < 7; a++)
	{
		sprintf (name, "decoy decision %d", flags[a]);
		expect (name, ask (CAMPAIGN_AIM_WEAPON_DECOY, CAMPAIGN_STEP_ACTION_DECOY, flags[a], 0, 1), (flags[a] == 0) || (flags[a] == 1));
		sprintf (name, "reacquisition decision %d", flags[a]);
		expect (name, ask (CAMPAIGN_AIM_WEAPON_REACQUIRE, CAMPAIGN_STEP_ACTION_REACQUIRE, flags[a], 0, 1), (flags[a] == 0) || (flags[a] == 1));

		for (b = 0; b < 7; b++)
		{
			sprintf (name, "seduction decisions %d and %d", flags[a], flags[b]);
			expect (name, ask (CAMPAIGN_AIM_WEAPON_SEDUCE, CAMPAIGN_STEP_ACTION_SEDUCE, flags[a], flags[b], 1), ((flags[a] == 0) || (flags[a] == 1)) && ((flags[b] == 0) || (flags[b] == 1)));
		}
	}

	/* what is not a decision is not narrowed: a state action's phase is not a boolean, and a reacquisition with no action is a matter for the handler that applies it */
	expect ("a step's state action with a missile phase of 5", ask (CAMPAIGN_AIM_WEAPON_STEP, CAMPAIGN_STEP_ACTION_STATE, 5, 1, 1), TRUE);
	expect ("a reacquisition with no action (refused when it is applied, not here)", ask (CAMPAIGN_AIM_WEAPON_REACQUIRE, CAMPAIGN_STEP_ACTION_REACQUIRE, 7, 7, 0), TRUE);
	expect ("an action of a kind the campaign does not know", ask (CAMPAIGN_AIM_WEAPON_REACQUIRE, CAMPAIGN_STEP_ACTIONS, 0, 0, 1), FALSE);

	printf ("%s: %d failures\n", failures ? "HOSTAIM_VALIDATION=FAILED" : "HOSTAIM_VALIDATION=ok", failures);

	return failures ? 1 : 0;
}
