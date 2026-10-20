/*
 * EEDCS-11 helicopter discriminator, TEST ONLY: one controlled physical input,
 * the position of an AI helicopter wingman. It is never part of a product
 * build: tools/build-observed-windows.sh controlled copies this file and
 * eech_arrival_observe.c into csrc/ and adds the hooks of patches.json and
 * patches-control.json, in a temporary worktree.
 *
 * Dormant unless EECH_ARRIVAL_CONTROL is set (and EECH_ARRIVAL_TRACE, which
 * records every trial):
 *
 *   EECH_ARRIVAL_CONTROL=hold:<four|navigation>:<offset m>:<trigger m>:<hold s>
 *
 * A trial starts when a SUPPLY task's guide, whose task leader is an AI
 * helicopter with another valid AI helicopter member, has a current waypoint
 * of the chosen kind ("four": PICK_UP, PREPARE_FOR_DROP_OFF, DROP_OFF,
 * FINISH_DROP_OFF; "navigation": NAVIGATION) and the leader is within
 * <trigger> m of its guide position. From then until the guide reaches that
 * waypoint, or <hold> simulated seconds pass, each such wingman is held
 * <offset> m from the leader, horizontally, on the side away from the guide
 * position, at the leader's height above the terrain. It is moved through
 * EECH's own aircraft position setter (set_local_entity_vec3d, the path the
 * flight model uses), just before the guide's arrival check, so the check
 * sees the world as it then is.
 *
 * It controls nothing else. It never sets a verdict, a criterion, a guide, a
 * task, a group or a member choice; the campaign's own check, leader
 * selection and handlers run unchanged, and the observer records what they
 * do. Every trial is a changed scenario, never windows-retail v1.
 */

#include "project.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "eech_headless.h"

#define MAX_TRIALS 4096
#define MAX_INDEX (1 << 20)

static int control_state; /* 0 not yet read, 1 on, -1 off */
static int kind_four;
static float offset_m, trigger_m, hold_s;

static struct
{
	int id;
	int wp_index;
	unsigned wp_gen_plus_one;
	double started, released;
	float dir_x, dir_z;
	int holding, holds;
} trial_of[MAX_INDEX]; /* by guide index */
static int trials;

extern unsigned eech_observe_generation (entity *en);
extern void eech_observe_trial_event (const char *what, int trial, entity *guide, entity *wingman, const vec3d *from, const vec3d *to, double held_for);
extern int (*eech_observe_trial_query) (entity *guide);

int eech_control_trial (entity *guide);

static int on (void)
{
	if (control_state == 0)
	{
		const char *spec = getenv ("EECH_ARRIVAL_CONTROL");
		char kind[32];
		control_state = -1;
		if (spec && getenv ("EECH_ARRIVAL_TRACE") && sscanf (spec, "hold:%31[a-z]:%f:%f:%f", kind, &offset_m, &trigger_m, &hold_s) == 4)
		{
			if (!strcmp (kind, "four") || !strcmp (kind, "navigation"))
			{
				kind_four = !strcmp (kind, "four");
				control_state = 1;
				eech_observe_trial_query = eech_control_trial;
				eech_log (1, "EEDCS-11 TEST-ONLY CONTROL: holding AI helicopter wingmen %.0f m from the leader at %s waypoints (trigger %.0f m, hold %.0f s)",
					offset_m, kind, trigger_m, hold_s);
			}
		}
		else if (spec)
		{
			eech_log (0, "EECH_ARRIVAL_CONTROL ignored: '%s' (needs hold:<four|navigation>:<offset>:<trigger>:<hold> and EECH_ARRIVAL_TRACE)", spec);
		}
	}
	return control_state == 1;
}

static int wanted (int wp_type)
{
	if (kind_four)
	{
		return wp_type == ENTITY_SUB_TYPE_WAYPOINT_PICK_UP || wp_type == ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF
			|| wp_type == ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF || wp_type == ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF;
	}
	return wp_type == ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION;
}

static double now (void)
{
	entity *session = get_session_entity ();
	return session ? get_local_entity_float_value (session, FLOAT_TYPE_ELAPSED_TIME) : -1.0;
}

static int ai_helicopter (entity *en)
{
	return en && get_local_entity_type (en) == ENTITY_TYPE_HELICOPTER && get_local_entity_int_value (en, INT_TYPE_ALIVE)
		&& get_local_entity_int_value (en, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI;
}

/* the trial under way for this guide, or 0 (read by the observer) */
int eech_control_trial (entity *guide)
{
	int gi;
	if (control_state != 1)
	{
		return 0;
	}
	gi = get_local_entity_index (guide);
	return (gi >= 0 && gi < MAX_INDEX) ? trial_of[gi].id : 0;
}

/* A13: gd_updt.c, before the guide's arrival check */
void eech_control_before_check (entity *guide)
{
	entity *wp, *task, *group, *leader, *member;
	vec3d *leader_pos, guide_pos, target;
	int gi, valid, wp_type;
	double t;

	if (!on ())
	{
		return;
	}
	gi = get_local_entity_index (guide);
	if (gi < 0 || gi >= MAX_INDEX)
	{
		return;
	}
	wp = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);
	t = now ();

	/* a trial ends when its waypoint is no longer current; its record stays open until then */
	if (trial_of[gi].id)
	{
		if (!wp || get_local_entity_index (wp) != trial_of[gi].wp_index || eech_observe_generation (wp) + 1 != trial_of[gi].wp_gen_plus_one)
		{
			eech_observe_trial_event ("trial_end", trial_of[gi].id, guide, NULL, NULL, NULL, t - trial_of[gi].started);
			memset (&trial_of[gi], 0, sizeof (trial_of[gi]));
		}
	}
	if (!wp)
	{
		return;
	}
	wp_type = get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE);
	task = get_local_entity_parent (wp, LIST_TYPE_WAYPOINT);
	if (!wanted (wp_type) || !task || get_local_entity_type (task) != ENTITY_TYPE_TASK
		|| get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) != ENTITY_SUB_TYPE_TASK_SUPPLY)
	{
		return;
	}
	leader = (entity *) get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER);
	group = get_local_entity_parent (guide, LIST_TYPE_GUIDE_STACK);
	if (!ai_helicopter (leader) || !group || get_local_entity_int_value (leader, INT_TYPE_LANDED))
	{
		return;
	}
	leader_pos = get_local_entity_vec3d_ptr (leader, VEC3D_TYPE_POSITION);
	valid = get_local_entity_int_value (guide, INT_TYPE_VALID_GUIDE_MEMBERS);
	get_local_entity_vec3d (guide, VEC3D_TYPE_POSITION, &guide_pos);

	if (!trial_of[gi].id)
	{
		int wingmen = 0;
		float dx, dz, len;
		if (get_local_entity_float_value (leader, FLOAT_TYPE_DISTANCE) > trigger_m || trials >= MAX_TRIALS)
		{
			return;
		}
		for (member = get_local_entity_first_child (group, LIST_TYPE_MEMBER); member; member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER))
		{
			if (member != leader && ai_helicopter (member) && !get_local_entity_int_value (member, INT_TYPE_LANDED)
				&& (valid & (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER))))
			{
				wingmen++;
			}
		}
		if (!wingmen)
		{
			return;
		}
		dx = leader_pos->x - guide_pos.x;
		dz = leader_pos->z - guide_pos.z;
		len = sqrt (dx * dx + dz * dz);
		trial_of[gi].id = ++trials;
		trial_of[gi].wp_index = get_local_entity_index (wp);
		trial_of[gi].wp_gen_plus_one = eech_observe_generation (wp) + 1;
		trial_of[gi].started = t;
		trial_of[gi].dir_x = len > 1.0f ? dx / len : 1.0f;
		trial_of[gi].dir_z = len > 1.0f ? dz / len : 0.0f;
		trial_of[gi].holding = 1;
		eech_observe_trial_event ("trial_start", trial_of[gi].id, guide, NULL, NULL, NULL, 0.0);
	}

	if (!trial_of[gi].holding)
	{
		return;
	}
	if (t - trial_of[gi].started > hold_s)
	{
		trial_of[gi].holding = 0;
		trial_of[gi].released = t;
		eech_observe_trial_event ("trial_release", trial_of[gi].id, guide, NULL, NULL, NULL, t - trial_of[gi].started);
		return;
	}

	/* hold each valid AI wingman at the offset: the physical input under control */
	for (member = get_local_entity_first_child (group, LIST_TYPE_MEMBER); member; member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER))
	{
		vec3d from;
		float agl;
		if (member == leader || !ai_helicopter (member) || get_local_entity_int_value (member, INT_TYPE_LANDED)
			|| !(valid & (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER))))
		{
			continue;
		}
		from = *get_local_entity_vec3d_ptr (member, VEC3D_TYPE_POSITION);
		agl = leader_pos->y - get_3d_terrain_elevation (leader_pos->x, leader_pos->z);
		target.x = leader_pos->x + trial_of[gi].dir_x * offset_m;
		target.z = leader_pos->z + trial_of[gi].dir_z * offset_m;
		target.y = 0.0f;
		bound_position_to_adjusted_map_area (&target);
		target.y = get_3d_terrain_elevation (target.x, target.z) + agl;
		set_local_entity_vec3d (member, VEC3D_TYPE_POSITION, &target);
		/* the first hold of each trial, then every 50th frame; trial_check events carry every frame's distances */
		if (trial_of[gi].holds++ % 50 == 0)
		{
			eech_observe_trial_event ("hold", trial_of[gi].id, guide, member, &from, &target, t - trial_of[gi].started);
		}
	}
}
