#include "project.h"

#include <stdlib.h>
#include <string.h>

#include "gd_supply_arrival.h"

static uint32_t *entity_generations;
static int generation_count;
static campaign_supply_arrival_host current_host;

static void validate_scope_database (void)
{
	static const entity_sub_types waypoints[] =
	{
		ENTITY_SUB_TYPE_WAYPOINT_PICK_UP,
		ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF,
		ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF,
		ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF
	};
	size_t index;
	for (index = 0; index < sizeof (waypoints) / sizeof (waypoints[0]); ++index)
	{
		entity_sub_types waypoint = waypoints[index];
		if (get_waypoint_database_last_to_reach_flag (waypoint, ENTITY_TYPE_FIXED_WING) ||
			get_waypoint_database_last_to_reach_flag (waypoint, ENTITY_TYPE_HELICOPTER))
		{
			debug_fatal ("supply arrival: scoped waypoint %d unexpectedly requires LAST_TO_REACH", waypoint);
		}
		if (waypoint_database[waypoint].check_waypoint_action ||
			(waypoint_database[waypoint].waypoint_action_message != NUM_ENTITY_MESSAGES) ||
			waypoint_database[waypoint].player_skip_waypoint)
		{
			debug_fatal ("supply arrival: scoped waypoint %d unexpectedly has an action path", waypoint);
		}
	}
}

static int entity_index (entity *en)
{
	int index = get_local_entity_index (en);
	if ((index < 0) || (index >= generation_count))
	{
		debug_fatal ("supply arrival: entity index %d is outside the heap", index);
	}
	return index;
}

static uint64_t entity_key (entity *en)
{
	int index = entity_index (en);
	return ((uint64_t) entity_generations[index] << 32) | (uint32_t) (index + 1);
}

static int waypoint_kind (entity *waypoint)
{
	switch (get_local_entity_int_value (waypoint, INT_TYPE_ENTITY_SUB_TYPE))
	{
		case ENTITY_SUB_TYPE_WAYPOINT_PICK_UP:
			return CAMPAIGN_SUPPLY_WAYPOINT_PICK_UP;
		case ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF:
			return CAMPAIGN_SUPPLY_WAYPOINT_PREPARE_FOR_DROP_OFF;
		case ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF:
			return CAMPAIGN_SUPPLY_WAYPOINT_DROP_OFF;
		case ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF:
			return CAMPAIGN_SUPPLY_WAYPOINT_FINISH_DROP_OFF;
		default:
			return CAMPAIGN_SUPPLY_WAYPOINT_NONE;
	}
}

static entity *current_waypoint (entity *guide)
{
	entity *waypoint = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);
	if (!waypoint)
	{
		debug_fatal ("supply arrival: guide has no current waypoint");
	}
	return waypoint;
}

void campaign_supply_arrival_reset (void)
{
	int index;
	free (entity_generations);
	generation_count = number_of_entities;
	entity_generations = (uint32_t *) malloc ((size_t) generation_count * sizeof (*entity_generations));
	if (!entity_generations)
	{
		debug_fatal ("supply arrival: cannot allocate generations for %d entities", generation_count);
	}
	for (index = 0; index < generation_count; ++index)
	{
		entity_generations[index] = 1;
	}
	validate_scope_database ();
}

void campaign_supply_arrival_set_host (const campaign_supply_arrival_host *host)
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

void campaign_supply_arrival_release_entity (entity *en)
{
	int index;
	if (!entity_generations || !entities)
	{
		return;
	}
	index = get_local_entity_index (en);
	if ((index < 0) || (index >= generation_count))
	{
		return;
	}
	entity_generations[index]++;
	if (!entity_generations[index])
	{
		debug_fatal ("supply arrival: entity generation overflow at index %d", index);
	}
}

int campaign_supply_arrival_is_target (entity *guide)
{
	entity *waypoint = current_waypoint (guide);
	entity *task = get_local_entity_parent (waypoint, LIST_TYPE_WAYPOINT);
	return task &&
		(get_local_entity_type (task) == ENTITY_TYPE_TASK) &&
		(get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_SUPPLY) &&
		(waypoint_kind (waypoint) != CAMPAIGN_SUPPLY_WAYPOINT_NONE);
}

static void copy_vec3d (float target[3], const vec3d *source)
{
	ASSERT (source);
	target[0] = source->x;
	target[1] = source->y;
	target[2] = source->z;
}

static int mobile_kind (int type)
{
	switch (type)
	{
		case ENTITY_TYPE_FIXED_WING:
			return CAMPAIGN_SUPPLY_MOBILE_FIXED_WING;
		case ENTITY_TYPE_HELICOPTER:
			return CAMPAIGN_SUPPLY_MOBILE_HELICOPTER;
		case ENTITY_TYPE_ROUTED_VEHICLE:
			return CAMPAIGN_SUPPLY_MOBILE_ROUTED_VEHICLE;
		case ENTITY_TYPE_SHIP_VEHICLE:
			return CAMPAIGN_SUPPLY_MOBILE_SHIP_VEHICLE;
		case ENTITY_TYPE_ANTI_AIRCRAFT:
			return CAMPAIGN_SUPPLY_MOBILE_ANTI_AIRCRAFT;
		case ENTITY_TYPE_PERSON:
			return CAMPAIGN_SUPPLY_MOBILE_PERSON;
		default:
			return 0;
	}
}

static int aircraft_kind (int kind)
{
	return (kind == CAMPAIGN_SUPPLY_MOBILE_FIXED_WING) || (kind == CAMPAIGN_SUPPLY_MOBILE_HELICOPTER);
}

/* Identity and mobile kind common to every query.  A member without a host identity is a boundary failure. */
static void begin_descriptor (campaign_supply_reach_descriptor *descriptor, int query, entity *member, entity *task, entity *waypoint, int kind)
{
	const char *host_id = campaign_roster_host_id (member);
	int sub_type = get_local_entity_int_value (waypoint, INT_TYPE_ENTITY_SUB_TYPE);

	if (!host_id)
	{
		debug_fatal ("supply arrival: member has no host identity");
	}
	if (strlen (host_id) > CAMPAIGN_SUPPLY_HOST_ID_MAX)
	{
		debug_fatal ("supply arrival: member host identity is too long");
	}
	memset (descriptor, 0, sizeof (*descriptor));
	memcpy (descriptor->host_id, host_id, strlen (host_id) + 1);
	descriptor->member_key = campaign_roster_member_key (member);
	descriptor->task_key = task ? entity_key (task) : 0;
	descriptor->waypoint_key = entity_key (waypoint);
	descriptor->query = query;
	descriptor->waypoint_kind = kind;
	descriptor->waypoint_sub_type = sub_type;
	descriptor->player_skip_waypoint = waypoint_database[sub_type].player_skip_waypoint ? TRUE : FALSE;
	descriptor->landing_waypoint = ((sub_type == ENTITY_SUB_TYPE_WAYPOINT_LANDED) || (sub_type == ENTITY_SUB_TYPE_WAYPOINT_TROOP_INSERT)) ? TRUE : FALSE;
	descriptor->mobile_kind = mobile_kind (get_local_entity_type (member));
	copy_vec3d (descriptor->position, get_local_entity_vec3d_ptr (member, VEC3D_TYPE_POSITION));
}

static campaign_arrival_result ask_host (const campaign_supply_reach_descriptor *descriptor)
{
	campaign_arrival_result result;
	char error[CAMPAIGN_SUPPLY_ERROR_MAX + 1];

	if (!current_host.observe_reach)
	{
		debug_fatal ("supply arrival: host supply_reached callback is unavailable");
	}
	memset (error, 0, sizeof (error));
	result.gates_passed = -1;
	result.angle_present = -1;
	result.launch_angle_error = 0.0f;
	if (current_host.observe_reach (descriptor, &result, error, CAMPAIGN_SUPPLY_ERROR_MAX, current_host.user) != 0)
	{
		debug_fatal ("supply arrival: host supply_reached failed: %s", error[0] ? error : "unspecified error");
	}
	if (((result.gates_passed != FALSE) && (result.gates_passed != TRUE)) ||
		((result.angle_present != FALSE) && (result.angle_present != TRUE)))
	{
		debug_fatal ("supply arrival: host supply_reached returned an invalid result");
	}
	return result;
}

/* Reads the required heading only when get_guide_required_heading cannot assert on its inputs. */
static int gather_required_heading (entity *guide, entity *member, entity *waypoint, float *heading)
{
	if (get_local_entity_int_value (waypoint, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_WAYPOINT_TARGET)
	{
		entity *task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);
		if (!task || !get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT))
		{
			return FALSE;
		}
	}
	return get_guide_required_heading (guide, member, heading);
}

/*
 * Copies the primitive facts of one member's arrival at the guide's current
 * waypoint and asks the host for the physical gates.  Only the facts each
 * mobile kind's legacy predicate reads are gathered, and the weapon-vector
 * tolerance (frand1, exponent, weapon limit) stays here as campaign policy.
 */
static int observe_reach (entity *member, entity *guide, entity *task, entity *waypoint, entity *leader, int kind, int is_task_leader)
{
	campaign_supply_reach_descriptor descriptor;
	campaign_arrival_result result;
	vec3d guide_position;
	int mobile = mobile_kind (get_local_entity_type (member));
	int selected_weapon = ENTITY_SUB_TYPE_WEAPON_NO_WEAPON;
	int vectors_flag = FALSE;
	int weapon_exponent = 0;

	begin_descriptor (&descriptor, CAMPAIGN_ARRIVAL_QUERY_REACHED, member, task, waypoint, kind);
	descriptor.is_task_leader = is_task_leader;
	descriptor.landed = get_local_entity_int_value (member, INT_TYPE_LANDED);
	descriptor.radius_valid = get_guide_criteria_valid (guide, GUIDE_CRITERIA_RADIUS) ? TRUE : FALSE;
	descriptor.altitude_valid = get_guide_criteria_valid (guide, GUIDE_CRITERIA_ALTITUDE) ? TRUE : FALSE;
	descriptor.heading_valid = get_guide_criteria_valid (guide, GUIDE_CRITERIA_HEADING) ? TRUE : FALSE;
	descriptor.last_to_reach_valid = get_guide_criteria_valid (guide, GUIDE_CRITERIA_LAST_TO_REACH) ? TRUE : FALSE;

	if (aircraft_kind (mobile))
	{
		descriptor.is_ai = get_local_entity_int_value (member, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI;
	}
	if (descriptor.radius_valid)
	{
		descriptor.radius = get_guide_criteria_value (guide, GUIDE_CRITERIA_RADIUS);
		descriptor.distance = get_local_entity_float_value (member, FLOAT_TYPE_DISTANCE);
	}

	if (mobile == CAMPAIGN_SUPPLY_MOBILE_FIXED_WING)
	{
		descriptor.updated = get_local_entity_int_value (member, INT_TYPE_UPDATED);
		descriptor.velocity = get_local_entity_float_value (member, FLOAT_TYPE_VELOCITY);
		descriptor.delta_seconds = get_delta_time ();
		copy_vec3d (descriptor.motion, get_local_entity_vec3d_ptr (member, VEC3D_TYPE_MOTION_VECTOR));
		get_local_entity_vec3d (member, VEC3D_TYPE_GUIDE_POSITION, &guide_position);
		copy_vec3d (descriptor.guide_position, &guide_position);
	}
	else if (mobile == CAMPAIGN_SUPPLY_MOBILE_HELICOPTER)
	{
		descriptor.rotor_spin_up_seconds = get_local_entity_float_value (member, FLOAT_TYPE_MAIN_ROTOR_SPIN_UP_TIMER);
		if (!is_task_leader)
		{
			descriptor.leader_is_ai = get_local_entity_int_value (leader, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI;
			copy_vec3d (descriptor.leader_position, get_local_entity_vec3d_ptr (leader, VEC3D_TYPE_POSITION));
		}
		if (descriptor.altitude_valid && !descriptor.is_ai)
		{
			descriptor.airborne = get_local_entity_int_value (member, INT_TYPE_AIRBORNE_AIRCRAFT);
			descriptor.auto_pilot = get_local_entity_int_value (member, INT_TYPE_AUTO_PILOT);
		}
		if (descriptor.heading_valid && descriptor.is_ai && is_task_leader)
		{
			descriptor.heading_criterion = get_guide_criteria_value (guide, GUIDE_CRITERIA_HEADING);
			descriptor.heading = get_local_entity_float_value (member, FLOAT_TYPE_HEADING);
			descriptor.required_heading_present = gather_required_heading (guide, member, waypoint, &descriptor.required_heading);
		}
	}

	/* Fixed-wing ignores ALTITUDE; the other kinds compare against the guide height. */
	if (descriptor.altitude_valid && (mobile != CAMPAIGN_SUPPLY_MOBILE_FIXED_WING))
	{
		descriptor.altitude_criterion = get_guide_criteria_value (guide, GUIDE_CRITERIA_ALTITUDE);
		descriptor.altitude = get_local_entity_float_value (member, FLOAT_TYPE_ALTITUDE);
		get_local_entity_vec3d (guide, VEC3D_TYPE_POSITION, &guide_position);
		descriptor.guide_altitude = guide_position.y;
		if (mobile == CAMPAIGN_SUPPLY_MOBILE_HELICOPTER)
		{
			descriptor.centre_of_gravity_to_ground = get_local_entity_float_value (member, FLOAT_TYPE_CENTRE_OF_GRAVITY_TO_GROUND_DISTANCE);
		}
	}

	if (get_guide_criteria_valid (guide, GUIDE_CRITERIA_WEAPON_VECTOR))
	{
		if (mobile == CAMPAIGN_SUPPLY_MOBILE_FIXED_WING)
		{
			descriptor.weapon_gate = TRUE;
			weapon_exponent = 4;
		}
		else if ((mobile == CAMPAIGN_SUPPLY_MOBILE_HELICOPTER) && descriptor.is_ai && is_task_leader)
		{
			descriptor.weapon_gate = TRUE;
			weapon_exponent = 6;
		}
	}
	if (descriptor.weapon_gate)
	{
		selected_weapon = get_local_entity_int_value (member, INT_TYPE_SELECTED_WEAPON);
		if (selected_weapon != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			vectors_flag = get_local_entity_int_value (member, INT_TYPE_WEAPON_AND_TARGET_VECTORS_VALID) ? TRUE : FALSE;
		}
		if (vectors_flag)
		{
			const vec3d *weapon_vector = get_local_entity_vec3d_ptr (member, VEC3D_TYPE_WEAPON_VECTOR);
			const vec3d *intercept_vector = get_local_entity_vec3d_ptr (member, VEC3D_TYPE_WEAPON_TO_INTERCEPT_POINT_VECTOR);
			if (weapon_vector && intercept_vector)
			{
				copy_vec3d (descriptor.weapon_vector, weapon_vector);
				copy_vec3d (descriptor.intercept_vector, intercept_vector);
				descriptor.vectors_valid = TRUE;
			}
		}
	}

	result = ask_host (&descriptor);
	if (!result.gates_passed)
	{
		return FALSE;
	}
	if (descriptor.weapon_gate)
	{
		if (selected_weapon == ENTITY_SUB_TYPE_WEAPON_NO_WEAPON)
		{
			if (mobile == CAMPAIGN_SUPPLY_MOBILE_HELICOPTER)
			{
				debug_log ("GUIDE: no selectedweapon");
				return FALSE;
			}
			ASSERT (selected_weapon != ENTITY_SUB_TYPE_WEAPON_NO_WEAPON);
		}
		if (vectors_flag)
		{
			ASSERT (descriptor.vectors_valid);
			if (!result.angle_present)
			{
				debug_fatal ("supply arrival: host supply_reached omitted the launch angle error the weapon vector requires");
			}
			if (fabs (result.launch_angle_error) > pow (frand1 (), weapon_exponent) * weapon_database[selected_weapon].max_launch_angle_error)
			{
				return FALSE;
			}
		}
	}
	return TRUE;
}

int campaign_supply_arrival_check (entity *member, entity *guide)
{
	entity *waypoint = current_waypoint (guide);
	entity *task = get_local_entity_parent (waypoint, LIST_TYPE_WAYPOINT);
	entity *leader = (entity *) get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER);

	if (!campaign_supply_arrival_is_target (guide) || !task || !leader)
	{
		debug_fatal ("supply arrival: invalid campaign selection");
	}
	if (!get_guide_criteria_valid (guide, GUIDE_CRITERIA_RADIUS) ||
		get_guide_criteria_valid (guide, GUIDE_CRITERIA_ALTITUDE) ||
		get_guide_criteria_valid (guide, GUIDE_CRITERIA_HEADING) ||
		get_guide_criteria_valid (guide, GUIDE_CRITERIA_WEAPON_VECTOR))
	{
		debug_fatal ("supply arrival: unsupported guide criteria for waypoint kind %d", waypoint_kind (waypoint));
	}
	if (!aircraft_kind (mobile_kind (get_local_entity_type (member))))
	{
		debug_fatal ("supply arrival: unsupported mobile entity type %d", get_local_entity_type (member));
	}
	return observe_reach (member, guide, task, waypoint, leader, waypoint_kind (waypoint), member == leader);
}

/*
 * Every other waypoint arrival of a mobile the campaign groups (fixed-wing,
 * helicopter, routed vehicle, ship, anti-aircraft, person).  Each kind keeps
 * its own criteria and early-out order in the host; the DLL keeps frand1.
 */
int campaign_radius_arrival_is_target (entity *member, entity *guide)
{
	return get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER) &&
		(mobile_kind (get_local_entity_type (member)) != 0);
}

int campaign_radius_arrival_check (entity *member, entity *guide)
{
	entity *waypoint = current_waypoint (guide);
	entity *task = get_local_entity_parent (waypoint, LIST_TYPE_WAYPOINT);
	entity *leader = (entity *) get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER);
	int is_task_leader;

	if (!campaign_radius_arrival_is_target (member, guide))
	{
		debug_fatal ("mobile arrival: invalid campaign selection");
	}
	is_task_leader = (get_local_entity_type (member) == ENTITY_TYPE_HELICOPTER) ?
		get_local_entity_int_value (member, INT_TYPE_TASK_LEADER) : (member == leader);
	return observe_reach (member, guide, task, waypoint, leader, CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT, is_task_leader);
}

/* The leader (already known not to be landed) is beyond the current waypoint's action radius: physical. */
int campaign_waypoint_action_reached (entity *guide, entity *leader)
{
	campaign_supply_reach_descriptor descriptor;
	entity *waypoint = current_waypoint (guide);
	entity *task = get_local_entity_parent (waypoint, LIST_TYPE_WAYPOINT);

	if (mobile_kind (get_local_entity_type (leader)) == 0)
	{
		debug_fatal ("waypoint action: unsupported mobile entity type %d", get_local_entity_type (leader));
	}
	begin_descriptor (&descriptor, CAMPAIGN_ARRIVAL_QUERY_ACTION, leader, task, waypoint, CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT);
	/* the caller has already returned FALSE for a landed leader */
	descriptor.distance = get_local_entity_float_value (leader, FLOAT_TYPE_DISTANCE);
	descriptor.action_radius = get_local_entity_waypoint_database_action_radius_value (waypoint, leader);
	return ask_host (&descriptor).gates_passed;
}

/* The helicopter is strictly inside the current waypoint's recon radius (squared 2D range). */
int campaign_recon_reached (entity *member, entity *waypoint)
{
	campaign_supply_reach_descriptor descriptor;
	entity *task = get_local_entity_parent (waypoint, LIST_TYPE_WAYPOINT);

	if (get_local_entity_type (member) != ENTITY_TYPE_HELICOPTER)
	{
		debug_fatal ("recon: unsupported mobile entity type %d", get_local_entity_type (member));
	}
	begin_descriptor (&descriptor, CAMPAIGN_ARRIVAL_QUERY_RECON, member, task, waypoint, CAMPAIGN_RADIUS_ARRIVAL_WAYPOINT);
	descriptor.action_radius = get_local_entity_waypoint_database_action_radius_value (waypoint, member);
	copy_vec3d (descriptor.waypoint_position, get_local_entity_vec3d_ptr (waypoint, VEC3D_TYPE_POSITION));
	return ask_host (&descriptor).gates_passed;
}


static entity *find_waypoint_dependent (entity *task, int wanted_kind)
{
	entity *waypoint;
	for (waypoint = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT); waypoint;
		waypoint = get_local_entity_child_succ (waypoint, LIST_TYPE_WAYPOINT))
	{
		if (waypoint_kind (waypoint) == wanted_kind)
		{
			return get_local_entity_parent (waypoint, LIST_TYPE_TASK_DEPENDENT);
		}
	}
	return NULL;
}

static int count_cargo (entity *leader, int cargo_type)
{
	int count = 0;
	entity *cargo;
	if (!leader)
	{
		return 0;
	}
	for (cargo = get_local_entity_first_child (leader, LIST_TYPE_MOVEMENT_DEPENDENT); cargo;
		cargo = get_local_entity_child_succ (cargo, LIST_TYPE_MOVEMENT_DEPENDENT))
	{
		if ((get_local_entity_type (cargo) == ENTITY_TYPE_CARGO) &&
			(get_local_entity_int_value (cargo, INT_TYPE_ENTITY_SUB_TYPE) == cargo_type))
		{
			count++;
		}
	}
	return count;
}

int campaign_supply_task_snapshots (campaign_supply_task_snapshot_fn callback, void *user)
{
	int count = 0;
	int index;
	for (index = 0; index < number_of_entities; ++index)
	{
		entity *task = &entities[index];
		if ((get_local_entity_type (task) == ENTITY_TYPE_TASK) &&
			(get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_SUPPLY))
		{
			campaign_supply_task_snapshot snapshot;
			entity *guide = get_local_entity_first_child (task, LIST_TYPE_GUIDE);
			entity *waypoint = guide ? get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT) : NULL;
			entity *leader = guide ? (entity *) get_local_entity_ptr_value (guide, PTR_TYPE_TASK_LEADER) : NULL;
			entity *supplier = find_waypoint_dependent (task, CAMPAIGN_SUPPLY_WAYPOINT_PICK_UP);
			entity *receiver = find_waypoint_dependent (task, CAMPAIGN_SUPPLY_WAYPOINT_DROP_OFF);
			memset (&snapshot, 0, sizeof (snapshot));
			snapshot.task_key = entity_key (task);
			snapshot.waypoint_key = waypoint ? entity_key (waypoint) : 0;
			if (leader)
			{
				const char *leader_host_id = campaign_roster_host_id (leader);
				if (leader_host_id)
				{
					memcpy (snapshot.leader_host_id, leader_host_id, strlen (leader_host_id) + 1);
				}
			}
			snapshot.task_entity_index = index;
			snapshot.waypoint_entity_index = waypoint ? get_local_entity_index (waypoint) : -1;
			snapshot.waypoint_kind = waypoint ? waypoint_kind (waypoint) : CAMPAIGN_SUPPLY_WAYPOINT_NONE;
			snapshot.task_completed = get_local_entity_int_value (task, INT_TYPE_TASK_COMPLETED);
			snapshot.supplier_entity_index = supplier ? get_local_entity_index (supplier) : -1;
			snapshot.receiver_entity_index = receiver ? get_local_entity_index (receiver) : -1;
			snapshot.cargo_type = (int) get_local_entity_float_value (task, FLOAT_TYPE_TASK_USER_DATA);
			snapshot.cargo_on_leader = count_cargo (leader, snapshot.cargo_type);
			if (supplier)
			{
				snapshot.supplier_ammo = get_local_entity_float_value (supplier, FLOAT_TYPE_AMMO_SUPPLY_LEVEL);
				snapshot.supplier_fuel = get_local_entity_float_value (supplier, FLOAT_TYPE_FUEL_SUPPLY_LEVEL);
			}
			if (receiver)
			{
				snapshot.receiver_ammo = get_local_entity_float_value (receiver, FLOAT_TYPE_AMMO_SUPPLY_LEVEL);
				snapshot.receiver_fuel = get_local_entity_float_value (receiver, FLOAT_TYPE_FUEL_SUPPLY_LEVEL);
			}
			callback (&snapshot, user);
			count++;
		}
	}
	return count;
}
