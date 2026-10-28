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
 *   health  the n-th living entity of a vehicle kind with a timed roll is set to a percentage of its initial health through the campaign's own setter, and the ORIGINAL critical
 *           predicate is asked whether it is now critical (the health is put back and the case reported not eligible if not); a person is never critical in the source and is
 *           reported, not changed; a fire roll also needs an explosive or flammable vehicle type
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
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

extern int eech_wpn_case_launch_run (const char *launcher_id, const char *target_id, int weapon, int salvo_count, char *text, int capacity);
extern int eech_wpn_case_query_run (const char *host_id, int weapon, char *text, int capacity);

/* the most launched-weapon children a launcher may already have, and the most weapons one launch may create, that this driver will account for exactly: beyond either the case
   is refused (before the launch for the first; after it, with every provably new weapon retired and the counts reported, for the second), never truncated */
#define CHILD_MAX 256

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

/* the launched-weapon children of a launcher, counted in full */
static int count_children (entity *launcher)
{
	entity *w;
	int n = 0;

	for (w = get_local_entity_first_child (launcher, LIST_TYPE_LAUNCHED_WEAPON); w; w = get_local_entity_child_succ (w, LIST_TYPE_LAUNCHED_WEAPON))
	{
		n++;
	}

	return n;
}

static int in_snapshot (entity *const *snapshot, int count, entity *w)
{
	int i;

	for (i = 0; i < count; i++)
	{
		if (snapshot[i] == w)
		{
			return 1;
		}
	}

	return 0;
}

/* destroys, by the campaign's own call, every launched-weapon child of the launcher that is NOT in the snapshot (it is complete, so those are exactly the weapons the launch
   created); one at a time, rescanning, because destroying a child changes the list being walked */
static int retire_all_new (entity *launcher, entity *const *snapshot, int count)
{
	int retired = 0;

	for (;;)
	{
		entity *w, *fresh = NULL;

		for (w = get_local_entity_first_child (launcher, LIST_TYPE_LAUNCHED_WEAPON); w; w = get_local_entity_child_succ (w, LIST_TYPE_LAUNCHED_WEAPON))
		{
			if (!in_snapshot (snapshot, count, w))
			{
				fresh = w;

				break;
			}
		}

		if (!fresh)
		{
			return retired;
		}

		destroy_client_server_entity_family (fresh);

		retired++;
	}
}

/* the text of a launch result with every run of white space turned into a bar, so that it is one word in a key=value report */
static void one_word (const char *from, char *to, int capacity)
{
	int i = 0;

	for (; *from && i < capacity - 1; from++)
	{
		to[i++] = (*from == ' ' || *from == '\n' || *from == '\t') ? '|' : *from;
	}

	while (i > 0 && to[i - 1] == '|')
	{
		i--;
	}

	to[i] = 0;
}

/* the integer after `key` in a Task45 state report, or `fallback` when the key is not there */
static int report_int (const char *report, const char *key, int fallback)
{
	const char *at = strstr (report, key);

	return at ? atoi (at + strlen (key)) : fallback;
}

/* launches one weapon from the host-identified launcher through the Task45 case launch (the campaign's own launch call, which asks the host for the birth) and returns the
   weapons it created, found as the launcher's launched-weapon children that were not there before.  FAIL CLOSED: a launcher that already has more children than the snapshot
   holds is refused BEFORE the launch (the new weapons could not be told from the old); a launch that creates more weapons than the table holds retires every one of them (all
   provably new) and is reported as an error with the counts.
   The launcher and the weapon are validated BEFORE the launch, by the Task45 read-only query (the helper's own checks of the host id, the launcher kind and the weapon sub type),
   because the launch helper answers an unsupported launcher kind, an invalid weapon sub type and a dead launcher with the SAME nonzero status: an invalid argument is an error and
   a dead launcher is a valid campaign outcome (the campaign does not launch from a dead launcher), and only the validation can tell them apart.  Nothing is changed by it.
   RETURNS: n > 0 the weapons created; 0 when the campaign REFUSED the launch or created nothing (a launcher that is not alive, no ammunition, the release gate's rate of fire):
   that is a result of the campaign, not an argument error, so it is DATA, and `detail` holds the launch result (the Task45 text with the counts before and after, or the reason)
   as one word; -1 for an error (an unusable launcher, an invalid weapon sub type, a launch the helper rejects after both were validated, or the case's own limits; `text` holds
   it).  `detail` is also filled when weapons were created. */
static int launch_new (const char *launcher_id, int weapon, entity **created, char *detail, int detail_capacity, char *text, int capacity)
{
	entity *launcher = find_by_host_id (launcher_id);
	entity *before[CHILD_MAX];
	entity *w;
	int before_count = 0, existing, made = 0, total_new = 0;
	char launch_text[512];

	detail[0] = 0;

	if (!launcher)
	{
		return case_error (text, capacity, "no_such_launcher_host_id", launcher_id);
	}

	if (eech_wpn_case_query_run (launcher_id, weapon, launch_text, (int) sizeof (launch_text)) != 0)
	{
		one_word (launch_text, detail, detail_capacity);

		return case_error (text, capacity, "launcher_cannot_be_used_for_a_case_launch", detail);
	}

	if (report_int (launch_text, " warhead=", -1) < 0)
	{
		snprintf (detail, (size_t) detail_capacity, "weapon=%d", weapon);

		return case_error (text, capacity, "weapon_sub_type_not_valid", detail);
	}

	if (report_int (launch_text, " alive=", 0) == 0)
	{
		/* a valid campaign outcome: the campaign does not launch from a dead launcher, and nothing was launched or changed */
		snprintf (detail, (size_t) detail_capacity, "case-refused|reason=launcher_not_alive");

		return 0;
	}

	existing = count_children (launcher);

	if (existing > CHILD_MAX)
	{
		snprintf (launch_text, sizeof (launch_text), "existing=%d limit=%d: nothing was launched", existing, CHILD_MAX);

		return case_error (text, capacity, "launcher_has_too_many_children_to_tell_new_from_old", launch_text);
	}

	for (w = get_local_entity_first_child (launcher, LIST_TYPE_LAUNCHED_WEAPON); w; w = get_local_entity_child_succ (w, LIST_TYPE_LAUNCHED_WEAPON))
	{
		before[before_count++] = w;
	}

	if (eech_wpn_case_launch_run (launcher_id, "", weapon, 1, launch_text, (int) sizeof (launch_text)) != 0)
	{
		/* the launcher and the weapon were validated above, so this is not a refusal of the campaign: it is an error, with the helper's own reason */
		one_word (launch_text, detail, detail_capacity);

		return case_error (text, capacity, "launch_rejected_after_validation", detail);
	}

	one_word (launch_text, detail, detail_capacity);

	for (w = get_local_entity_first_child (launcher, LIST_TYPE_LAUNCHED_WEAPON); w; w = get_local_entity_child_succ (w, LIST_TYPE_LAUNCHED_WEAPON))
	{
		if (!in_snapshot (before, before_count, w))
		{
			total_new++;

			if (made < CHILD_MAX)
			{
				created[made++] = w;
			}
		}
	}

	if (total_new > CHILD_MAX)
	{
		int retired = retire_all_new (launcher, before, before_count);

		snprintf (launch_text, sizeof (launch_text), "created=%d limit=%d retired_new=%d unrelated_retired=0", total_new, CHILD_MAX, retired);

		return case_error (text, capacity, "launch_created_more_weapons_than_the_table_holds", launch_text);
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

/* the vehicle kinds whose per-frame update has a timed health roll gated by vehicle_critically_damaged (vehicle.c): a person's bleeding, and the fire of an anti-aircraft, routed or
   ship vehicle.  The original predicate is `100 * damage_level / initial_damage_level < VEHICLE_CRITICAL_DAMAGE_LEVEL (15)` in integer arithmetic and is FALSE for a person whatever
   its health, so a person's bleeding roll cannot be reached from any health: this driver reports that and changes nothing.  A fire roll also needs an explosive or flammable
   vehicle type.  The driver sets the health through the campaign's own setter, asks the ORIGINAL predicate whether the vehicle is now critical, and puts the health back (and says
   so) if it is not: it never edits the predicate and never claims a roll that cannot run. */
int eech_cmb_case_health_run (const char *kind, int ordinal, int percent, char *text, int capacity)
{
	int type = kind_by_name (kind), live, initial, before, level, quality, critical;
	entity *receiver;

	text[0] = 0;

	if (type < 0)
	{
		return case_error (text, capacity, "kind_not_one_of_the_thirteen", kind);
	}

	if (type != ENTITY_TYPE_ANTI_AIRCRAFT && type != ENTITY_TYPE_ROUTED_VEHICLE && type != ENTITY_TYPE_SHIP_VEHICLE && type != ENTITY_TYPE_PERSON)
	{
		return case_error (text, capacity, "kind_has_no_vehicle_critical_predicate", kind);
	}

	if (ordinal < 0 || percent < 1 || percent > 100)
	{
		return case_error (text, capacity, "ordinal_or_percent_out_of_range", kind);
	}

	receiver = nth_live (type, ordinal, &live);

	if (!receiver)
	{
		snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=0 eligible=0\n", kind, ordinal, live);

		return 0;
	}

	initial = get_local_entity_int_value (receiver, INT_TYPE_INITIAL_DAMAGE_LEVEL);
	before = get_local_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL);
	quality = get_local_entity_int_value (receiver, INT_TYPE_EXPLOSIVE_QUALITY);

	if (type == ENTITY_TYPE_PERSON)
	{
		/* the original predicate returns FALSE for a person before it looks at the health: asked once, as it is asked in the update, and nothing is changed */
		critical = vehicle_critically_damaged (receiver);

		snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=1 eligible=0 reason=person_never_critical_in_source predicate=%d initial=%d before=%d set=-1\n", kind,
			ordinal, live, critical ? 1 : 0, initial, before);

		return 0;
	}

	if (!(quality & (EXPLOSIVE_QUALITY_EXPLOSIVE | EXPLOSIVE_QUALITY_FLAMMABLE)))
	{
		snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=1 eligible=0 reason=not_explosive_or_flammable quality=%d initial=%d before=%d set=-1\n", kind, ordinal, live,
			quality, initial, before);

		return 0;
	}

	level = (initial * percent) / 100;

	if (level < 1)
	{
		level = 1;
	}

	set_client_server_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL, level);

	critical = vehicle_critically_damaged (receiver);

	if (!critical)
	{
		set_client_server_entity_int_value (receiver, INT_TYPE_DAMAGE_LEVEL, before);

		snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=1 eligible=0 reason=level_not_critical quality=%d initial=%d before=%d set=%d restored=%d\n", kind, ordinal,
			live, quality, initial, before, level, before);

		return 0;
	}

	snprintf (text, (size_t) capacity, "cmb-case-health kind=%s ordinal=%d live=%d found=1 eligible=1 quality=%d initial=%d before=%d set=%d critical=1\n", kind, ordinal, live, quality, initial, before, level);

	return 0;
}

int eech_cmb_case_collide_run (const char *kind, int ordinal, const char *launcher_id, int weapon, double modifier, int weaponless, char *text, int capacity)
{
	int type = kind_by_name (kind), live, made = 0, i, level_before, alive_before;
	entity *receiver, *created[CHILD_MAX], *sender = NULL;
	char detail[256];

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

	detail[0] = 0;

	if (!weaponless)
	{
		made = launch_new (launcher_id, weapon, created, detail, (int) sizeof (detail), text, capacity);

		if (made < 0)
		{
			return -1;
		}

		if (made == 0)
		{
			/* the campaign refused the launch or created nothing: the case is NOT done, and the reason is the result */
			snprintf (text, (size_t) capacity, "cmb-case-collide kind=%s ordinal=%d live=%d found=1 weapon=%d weaponless=0 launched=0 refused=1 modifier=%.9g launch=%s\n", kind, ordinal, live, weapon,
				modifier, detail);

			return 0;
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

	snprintf (text, (size_t) capacity, "cmb-case-collide kind=%s ordinal=%d live=%d found=1 weapon=%d weaponless=%d launched=%d refused=0 modifier=%.9g level_before=%d alive_before=%d launch=%s\n", kind,
		ordinal, live, weapon, weaponless ? 1 : 0, made, modifier, level_before, alive_before, detail[0] ? detail : "none");

	return 0;
}

int eech_cmb_case_blast_run (const char *kind, int ordinal, double offset, int kill_code, const char *launcher_id, int weapon, char *text, int capacity)
{
	int type = kind_by_name (kind), live, made, i, capability;
	entity *receiver, *created[CHILD_MAX], *exploder;
	vec3d position;
	char detail[256];

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

	/* the resulting position is validated BEFORE anything is launched or mutated, with the game's own map test (the one the weapon position setter's sector lookup applies, and
	   which it answers by stopping the engine); a non-finite or off-map placement, including the default offset near the map edge, is reported as not done */
	get_local_entity_vec3d (receiver, VEC3D_TYPE_POSITION, &position);

	position.x += (float) offset;

	if (!isfinite (position.x) || !isfinite (position.y) || !isfinite (position.z) || !point_inside_map_area (&position))
	{
		snprintf (text, (size_t) capacity, "cmb-case-blast kind=%s ordinal=%d live=%d found=1 placed=0 reason=placement_off_map offset=%.9g x=%.9g z=%.9g\n", kind, ordinal, live, offset, (double) position.x,
			(double) position.z);

		return 0;
	}

	made = launch_new (launcher_id, weapon, created, detail, (int) sizeof (detail), text, capacity);

	if (made < 0)
	{
		return -1;
	}

	if (made == 0)
	{
		/* the campaign refused the launch or created nothing: the case is NOT done (the placement was valid), and the reason is the result */
		snprintf (text, (size_t) capacity, "cmb-case-blast kind=%s ordinal=%d live=%d found=1 placed=1 weapon=%d kill_code=%d offset=%.9g launched=0 refused=1 launch=%s\n", kind, ordinal, live, weapon,
			kill_code, offset, detail);

		return 0;
	}

	exploder = created[0];

	for (i = 1; i < made; i++)
	{
		retire (created[i]);
	}

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

	snprintf (text, (size_t) capacity, "cmb-case-blast kind=%s ordinal=%d live=%d found=1 placed=1 weapon=%d kill_code=%d offset=%.9g capability=%d launched=%d refused=0 launch=%s\n", kind, ordinal, live,
		weapon, kill_code, offset, capability, made, detail[0] ? detail : "none");

	return 0;
}

#endif
