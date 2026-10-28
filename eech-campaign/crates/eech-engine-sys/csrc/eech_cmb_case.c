/*
 * TEST-ONLY controlled combat cases for the ORIGINAL campaign (EEDCS-46). Compiled only with the `weapon-fixture` feature; never part of the shipping module. NOT yet built or run
 * (design under review); it moves no physics anywhere and adds no campaign behaviour.
 *
 * The natural war reaches only some of the physical combat paths (the census of csrc/eech_cmb_fixture.c says which). These cases put the REST of them to the campaign's own
 * code, so the observer can measure what the original does there: each case sets a PRECONDITION through the campaign's own functions and then calls the campaign's own entry
 * point; everything that follows (the collision handler of the receiver's kind, damage_client_server_entity, weapon_damage_capability and its draw, the health write, the fatal
 * decision, the credit, the explosion effect, the blast's walk over the sectors) is the original's and is recorded by the observer's hooks, never by this file.
 *
 *   census  how many living entities of each of the thirteen health-bearing kinds the world holds (the receivers a case can choose among)
 *   collide the n-th living entity of a kind is delivered ENTITY_MESSAGE_COLLISION by a REAL weapon (launched through the Task45 case launch from a host-identified
 *           launcher) with a chosen modifier (0.0 is a direct hit; above 0 is a blast at that range fraction), or by no weapon at all (the path a debug event takes)
 *   blast   a REAL weapon launched the same way is placed at a chosen offset from the n-th living entity of a kind, given a chosen kill code (and, for a hit on target, linked
 *           to that entity as its target) and put through create_client_server_weapon_killed_explosion_effect, the call the weapon's own update makes when its kill code is set,
 *           then destroyed as the update would
 *   health  the n-th living entity of a kind is set to a percentage of its initial health through the campaign's own setter, so that the campaign's own per-frame fire and
 *           bleeding rolls can run on it in the frames that follow
 *
 * PRECONDITIONS are the test's choice and are declared as such: the receiver (kind and ordinal in the entity list's order), the weapon type and launcher, the modifier, the
 * offset, the kill code, the starting health. The weapon is a real launched weapon, but WHERE it ends and WITH WHAT kill code in a blast case is the test's choice, standing in
 * for the flight's decision (which the physical weapon-execution task owns). Campaign consequences (credit, speech, force notifications, losses) happen as the original does them.
 * Only kinds, counts and host ids cross; no pointer, index or seed leaves this file. A receiver is never read after a case: the observer's event log is the record.
 */

#ifdef EECH_WEAPON_FIXTURE

#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

extern int eech_wpn_case_launch_run (const char *launcher_id, const char *target_id, int weapon, int salvo_count, char *text, int capacity);

#define NEW_MAX 16

/* the thirteen kinds that hold a damage level and read the collision modifier */
static const entity_types case_kinds[] =
{
	ENTITY_TYPE_ANTI_AIRCRAFT,
	ENTITY_TYPE_CITY_BUILDING,
	ENTITY_TYPE_FIXED_WING,
	ENTITY_TYPE_HELICOPTER,
	ENTITY_TYPE_OBJECT,
	ENTITY_TYPE_PERSON,
	ENTITY_TYPE_PYLON,
	ENTITY_TYPE_ROUTED_VEHICLE,
	ENTITY_TYPE_SCENIC,
	ENTITY_TYPE_SEGMENT,
	ENTITY_TYPE_SHIP_VEHICLE,
	ENTITY_TYPE_SITE,
	ENTITY_TYPE_SITE_UPDATABLE
};

#define CASE_KINDS ((int) (sizeof (case_kinds) / sizeof (case_kinds[0])))

static int case_error (char *text, int capacity, const char *reason, const char *detail)
{
	snprintf (text, (size_t) capacity, "cmb-case-error reason=%s detail=%s\n", reason, detail ? detail : "");

	return -1;
}

static int entity_in_array (entity *en)
{
	return entities && en >= entities && en < entities + number_of_entities;
}

/* the kind named by the caller, with or without the ENTITY_TYPE_ prefix, among the thirteen; -1 when it is none of them */
static int kind_by_name (const char *name)
{
	int i;
	char full[96];

	snprintf (full, sizeof (full), "ENTITY_TYPE_%s", name);

	for (i = 0; i < CASE_KINDS; i++)
	{
		const char *candidate = get_entity_type_name (case_kinds[i]);

		if (strcmp (candidate, name) == 0 || strcmp (candidate, full) == 0)
		{
			return (int) case_kinds[i];
		}
	}

	return -1;
}

/* the ordinal-th (from 0) living entity of the kind, in the entity list's order; `live` is how many there are */
static entity *nth_live (int type, int ordinal, int *live)
{
	entity *en, *found = NULL;
	int n = 0;

	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		if (!entity_in_array (en))
		{
			break;
		}

		if (((int) en->type == type) && get_local_entity_int_value (en, INT_TYPE_ALIVE))
		{
			if (n == ordinal)
			{
				found = en;
			}

			n++;
		}
	}

	*live = n;

	return found;
}

static entity *find_by_host_id (const char *host_id)
{
	entity *en;

	for (en = get_local_entity_list (); en; en = get_local_entity_succ (en))
	{
		const char *id;

		if (!entity_in_array (en))
		{
			break;
		}

		id = campaign_roster_host_id (en);

		if (id && strcmp (id, host_id) == 0)
		{
			return en;
		}
	}

	return NULL;
}

/* launches one weapon from the host-identified launcher through the Task45 case launch (the campaign's own launch call, which asks the host for the birth) and returns the
   weapons it created, found as the launcher's launched-weapon children that were not there before */
static int launch_new (const char *launcher_id, int weapon, entity **created, char *text, int capacity)
{
	entity *launcher = find_by_host_id (launcher_id);
	entity *before[NEW_MAX * 4];
	entity *w;
	int before_count = 0, made = 0, i;
	char launch_text[512];

	if (!launcher)
	{
		return case_error (text, capacity, "no_such_launcher_host_id", launcher_id);
	}

	for (w = get_local_entity_first_child (launcher, LIST_TYPE_LAUNCHED_WEAPON); w && before_count < NEW_MAX * 4; w = get_local_entity_child_succ (w, LIST_TYPE_LAUNCHED_WEAPON))
	{
		before[before_count++] = w;
	}

	if (eech_wpn_case_launch_run (launcher_id, "", weapon, 1, launch_text, (int) sizeof (launch_text)) != 0)
	{
		return case_error (text, capacity, "launch_refused", launch_text);
	}

	for (w = get_local_entity_first_child (launcher, LIST_TYPE_LAUNCHED_WEAPON); w; w = get_local_entity_child_succ (w, LIST_TYPE_LAUNCHED_WEAPON))
	{
		int known = 0;

		for (i = 0; i < before_count; i++)
		{
			if (before[i] == w)
			{
				known = 1;
			}
		}

		if (!known && made < NEW_MAX)
		{
			created[made++] = w;
		}
	}

	if (!made)
	{
		return case_error (text, capacity, "launch_created_no_weapon", launch_text);
	}

	return made;
}

/* a launched weapon that is not part of the case is destroyed by the campaign's own call, with no explosion effect (so that it is neither an explosion invocation nor a blast
   in the observer's counts) */
static void retire (entity *weapon)
{
	destroy_client_server_entity_family (weapon);
}

int eech_cmb_case_census_run (char *text, int capacity)
{
	int i, live, used = 0;

	text[0] = 0;

	for (i = 0; i < CASE_KINDS; i++)
	{
		int n;

		(void) nth_live ((int) case_kinds[i], -1, &live);

		n = snprintf (text + used, (size_t) (capacity - used), "cmb-case-census kind=%s live=%d\n", get_entity_type_name (case_kinds[i]), live);

		if (n < 0 || n >= capacity - used)
		{
			return -1;
		}

		used += n;
	}

	return 0;
}

int eech_cmb_case_health_run (const char *kind, int ordinal, int percent, char *text, int capacity)
{
	int type = kind_by_name (kind), live, initial, before, level;
	entity *receiver;

	text[0] = 0;

	if (type < 0)
	{
		return case_error (text, capacity, "kind_not_one_of_the_thirteen", kind);
	}

	if (ordinal < 0 || percent < 1 || percent > 100)
	{
		return case_error (text, capacity, "ordinal_or_percent_out_of_range", kind);
	}

	receiver = nth_live (type, ordinal, &live);

	if (!receiver)
	{
		snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=0\n", kind, ordinal, live);

		return 0;
	}

	initial = get_local_entity_int_value (receiver, INT_TYPE_INITIAL_DAMAGE_LEVEL);
	before = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	level = (initial * percent) / 100;

	if (level < 1)
	{
		level = 1;
	}

	set_client_server_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL, level);

	snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=1 initial=%d before=%d set=%d\n", kind, ordinal, live, initial, before, level);

	return 0;
}

int eech_cmb_case_collide_run (const char *kind, int ordinal, const char *launcher_id, int weapon, double modifier, int weaponless, char *text, int capacity)
{
	int type = kind_by_name (kind), live, made = 0, i, level_before, alive_before;
	entity *receiver, *created[NEW_MAX], *sender = NULL;

	text[0] = 0;

	if (type < 0)
	{
		return case_error (text, capacity, "kind_not_one_of_the_thirteen", kind);
	}

	if (ordinal < 0 || !(modifier >= 0.0 && modifier <= 1.0))
	{
		return case_error (text, capacity, "ordinal_or_modifier_out_of_range", kind);
	}

	receiver = nth_live (type, ordinal, &live);

	if (!receiver)
	{
		snprintf (text, (size_t) capacity, "cmb-case-collide kind=%s ordinal=%d live=%d found=0\n", kind, ordinal, live);

		return 0;
	}

	if (!weaponless)
	{
		made = launch_new (launcher_id, weapon, created, text, capacity);

		if (made < 0)
		{
			return -1;
		}

		sender = created[0];

		/* a weapon cannot be its own launcher's victim in this case: the credit path would be the receiver crediting itself */
		if (get_local_entity_parent (sender, LIST_TYPE_LAUNCHED_WEAPON) == receiver)
		{
			for (i = 0; i < made; i++)
			{
				retire (created[i]);
			}

			return case_error (text, capacity, "receiver_is_the_launcher", launcher_id);
		}
	}

	level_before = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	alive_before = get_local_entity_int_value (receiver, INT_TYPE_ALIVE);

	/* the campaign's own notification, as the explosion's direct hit and blast send it; the receiver is not read again */
	notify_local_entity (ENTITY_MESSAGE_COLLISION, receiver, sender, modifier);

	for (i = 0; i < made; i++)
	{
		retire (created[i]);
	}

	snprintf (text, (size_t) capacity, "cmb-case-collide kind=%s ordinal=%d live=%d found=1 weapon=%d weaponless=%d launched=%d modifier=%.9g level_before=%d alive_before=%d\n", kind, ordinal, live,
		weapon, weaponless ? 1 : 0, made, modifier, level_before, alive_before);

	return 0;
}

int eech_cmb_case_blast_run (const char *kind, int ordinal, double offset, int kill_code, const char *launcher_id, int weapon, char *text, int capacity)
{
	int type = kind_by_name (kind), live, made, i, capability;
	entity *receiver, *created[NEW_MAX], *exploder;
	vec3d position;

	text[0] = 0;

	if (type < 0)
	{
		return case_error (text, capacity, "kind_not_one_of_the_thirteen", kind);
	}

	/* only the kill codes that blast: land, target, overshot, self destruct (the others do nothing, which a collide case with an exhausted weapon already is) */
	if (ordinal < 0 || !(fabs (offset) < 100000.0) || !(kill_code == WEAPON_KILL_CODE_HIT_LAND || kill_code == WEAPON_KILL_CODE_HIT_TARGET || kill_code == WEAPON_KILL_CODE_OVERSHOT_TARGET || kill_code == WEAPON_KILL_CODE_SELF_DESTRUCT))
	{
		return case_error (text, capacity, "ordinal_offset_or_kill_code_out_of_range", kind);
	}

	receiver = nth_live (type, ordinal, &live);

	if (!receiver)
	{
		snprintf (text, (size_t) capacity, "cmb-case-blast kind=%s ordinal=%d live=%d found=0\n", kind, ordinal, live);

		return 0;
	}

	made = launch_new (launcher_id, weapon, created, text, capacity);

	if (made < 0)
	{
		return -1;
	}

	exploder = created[0];

	for (i = 1; i < made; i++)
	{
		retire (created[i]);
	}

	get_local_entity_vec3d (receiver, VEC3D_TYPE_POSITION, &position);

	position.x += (float) offset;

	/* the test's precondition: where this weapon ends and with what kill code, standing in for the flight's decision */
	set_client_server_entity_vec3d (exploder, VEC3D_TYPE_POSITION, &position);

	if (kill_code == WEAPON_KILL_CODE_HIT_TARGET)
	{
		set_client_server_entity_parent (exploder, LIST_TYPE_TARGET, receiver);
	}

	set_local_entity_int_value (exploder, INT_TYPE_WEAPON_KILL_CODE, kill_code);

	capability = get_local_entity_int_value (exploder, INT_TYPE_DAMAGE_CAPABILITY);

	/* the call the weapon's own update makes when its kill code is set, then the destruction its next update makes */
	create_client_server_weapon_killed_explosion_effect (exploder);

	destroy_client_server_entity_family (exploder);

	snprintf (text, (size_t) capacity, "cmb-case-blast kind=%s ordinal=%d live=%d found=1 weapon=%d kill_code=%d offset=%.9g capability=%d launched=%d\n", kind, ordinal, live, weapon, kill_code,
		offset, capability, made);

	return 0;
}

#endif
