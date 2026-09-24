/*
 * EECH headless engine: observing the simulation. Reports the objects a
 * viewer (Tacview) or a host needs, classified the way EECH's own Tacview
 * logger classifies them (aphavoc/source/entity/tacview/tacview.c): aircraft,
 * ground vehicles, ships, air defence, weapons in flight and keysites, with
 * their names from EECH's databases.
 */

#include "project.h"

#include "eech_engine.h"
#include "eech_headless.h"

extern jmp_buf *eech_fatal_target;

static void fill_attitude (entity *en, struct eech_object *o)
{
	matrix3x3 m;
	get_local_entity_attitude_matrix (en, m);
	o->heading = (float) get_heading_from_attitude_matrix (m);
	o->pitch = (float) get_pitch_from_attitude_matrix (m);
	o->roll = (float) get_roll_from_attitude_matrix (m);
}

static void fill_group (entity *en, struct eech_object *o)
{
	entity *group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	if (group)
	{
		entity *task;
		o->group_id = get_local_entity_index (group);
		o->name = get_local_entity_string (group, STRING_TYPE_GROUP_CALLSIGN);
		task = get_local_group_primary_task (group);
		if (task)
		{
			o->task = entity_sub_type_task_names[get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE)];
		}
	}
}

static int observe (entity *en, eech_object_callback callback, void *user)
{
	struct eech_object o;
	vec3d *p;
	int sub_type;
	memset (&o, 0, sizeof (o));
	o.group_id = -1;
	o.id = get_local_entity_index (en);
	sub_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);
	o.sub_type = sub_type;
	switch (get_local_entity_type (en))
	{
		case ENTITY_TYPE_HELICOPTER:
			o.kind = EECH_OBJECT_HELICOPTER;
			o.type_name = aircraft_database[sub_type].full_name;
			break;
		case ENTITY_TYPE_FIXED_WING:
			o.kind = EECH_OBJECT_FIXED_WING;
			o.type_name = aircraft_database[sub_type].full_name;
			break;
		case ENTITY_TYPE_ROUTED_VEHICLE:
			o.kind = EECH_OBJECT_GROUND_VEHICLE;
			o.type_name = vehicle_database[sub_type].full_name;
			break;
		case ENTITY_TYPE_ANTI_AIRCRAFT:
			o.kind = EECH_OBJECT_AIR_DEFENCE;
			o.type_name = vehicle_database[sub_type].full_name;
			break;
		case ENTITY_TYPE_SHIP_VEHICLE:
			o.kind = EECH_OBJECT_SHIP;
			o.type_name = vehicle_database[sub_type].full_name;
			break;
		case ENTITY_TYPE_PERSON:
			o.kind = EECH_OBJECT_INFANTRY;
			o.type_name = vehicle_database[sub_type].full_name;
			break;
		case ENTITY_TYPE_WEAPON:
			/*
			 * missiles, rockets and bombs are reported; gun and artillery
			 * rounds, decoys, cargo and debris are not (tacview.c: sub-types up
			 * to 2A65_152MM_ROUND are rounds, except the MLRS rockets)
			 */
			if (sub_type <= ENTITY_SUB_TYPE_WEAPON_NO_WEAPON
				|| (sub_type <= ENTITY_SUB_TYPE_WEAPON_2A65_152MM_ROUND && sub_type != ENTITY_SUB_TYPE_WEAPON_BM21_122MM_ROCKET
					&& sub_type != ENTITY_SUB_TYPE_WEAPON_M26A1_227MM_ROCKET)
				|| (weapon_database[sub_type].weapon_class & (WEAPON_CLASS_DECOY | WEAPON_CLASS_CARGO | WEAPON_CLASS_DEBRIS)))
			{
				return 0;
			}
			o.kind = EECH_OBJECT_WEAPON;
			o.type_name = weapon_database[sub_type].full_name;
			{
				entity *launcher = get_local_entity_parent (en, LIST_TYPE_LAUNCHED_WEAPON);
				o.group_id = launcher ? get_local_entity_index (launcher) : -1;
			}
			break;
		case ENTITY_TYPE_KEYSITE:
			o.kind = EECH_OBJECT_KEYSITE;
			o.type_name = entity_sub_type_keysite_names[sub_type];
			o.name = get_local_entity_string (en, STRING_TYPE_KEYSITE_NAME);
			break;
		default:
			return 0;
	}
	o.side = get_local_entity_int_value (en, INT_TYPE_SIDE);
	p = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);
	o.x = p->x;
	o.y = p->y;
	o.z = p->z;
	if (o.kind == EECH_OBJECT_KEYSITE)
	{
		o.alive = TRUE;
		o.efficiency = get_local_entity_float_value (en, FLOAT_TYPE_EFFICIENCY);
	}
	else
	{
		fill_attitude (en, &o);
		if (o.kind != EECH_OBJECT_WEAPON)
		{
			o.alive = get_local_entity_int_value (en, INT_TYPE_ALIVE);
			fill_group (en, &o);
		}
		else
		{
			o.alive = TRUE;
		}
	}
	callback (&o, user);
	return 1;
}

int eech_observe_objects (eech_object_callback callback, void *user)
{
	int count = 0;
	entity *en, *force;
	if (!get_session_entity ())
	{
		return 0;
	}
	for (en = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE); en; en = get_local_entity_child_succ (en, LIST_TYPE_UPDATE))
	{
		count += observe (en, callback, user);
	}
	for (force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE); force; force = get_local_entity_child_succ (force, LIST_TYPE_FORCE))
	{
		for (en = get_local_entity_first_child (force, LIST_TYPE_KEYSITE_FORCE); en; en = get_local_entity_child_succ (en, LIST_TYPE_KEYSITE_FORCE))
		{
			count += observe (en, callback, user);
		}
	}
	return count;
}

int eech_observe_clock (struct eech_clock *clock)
{
	entity *session = get_session_entity ();
	if (!session)
	{
		return EECH_ENGINE_STATE;
	}
	clock->elapsed_seconds = get_local_entity_float_value (session, FLOAT_TYPE_ELAPSED_TIME);
	clock->time_of_day_seconds = get_local_entity_float_value (session, FLOAT_TYPE_TIME_OF_DAY);
	clock->day = get_local_entity_int_value (session, INT_TYPE_DAY);
	return EECH_ENGINE_OK;
}
