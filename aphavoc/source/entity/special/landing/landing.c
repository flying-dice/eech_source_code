

































































#include "project.h"





#define DEBUG_MODULE 0

#define DEBUG_LANDING_TYPE (1 << ENTITY_SUB_TYPE_LANDING_FIXED_WING)

#define DEBUG_LANDING_SIDE ((1 << ENTITY_SIDE_RED_FORCE) & (1 << ENTITY_SIDE_BLUE_FORCE))

#define FATAL	1





entity *get_local_entity_landing_entity (entity *en, entity_sub_types landing_type)
{

	entity
		*landing_entity;

	ASSERT (get_local_entity_type (en) == ENTITY_TYPE_KEYSITE);

	landing_entity = get_local_entity_first_child (en, LIST_TYPE_LANDING_SITE);

	while (landing_entity)
	{

		if (get_local_entity_int_value (landing_entity, INT_TYPE_ENTITY_SUB_TYPE) == landing_type)
		{

			return landing_entity;
		}

		landing_entity = get_local_entity_child_succ (landing_entity, LIST_TYPE_LANDING_SITE);
	}

	return NULL;
}





entity *get_local_landing_entity_route (entity *landing_en, entity_sub_types type)
{

	entity
		*task,
		*wp;

	if (landing_en->type != ENTITY_TYPE_LANDING)
	{

		// not even a landing entity

		return NULL;
	}

	task = get_local_entity_first_child (landing_en, LIST_TYPE_ASSIGNED_TASK);

	//

	while (task)
	{

		if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == type)
		{

			wp = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT);

			return wp;
		}

		task = get_local_entity_child_succ (task, LIST_TYPE_ASSIGNED_TASK);
	}

	return NULL;
}





entity *get_local_landing_entity_task (entity *landing_en, entity_sub_types type)
{

	entity
		*task;

	ASSERT (landing_en->type == ENTITY_TYPE_LANDING);

	task = get_local_entity_first_child (landing_en, LIST_TYPE_ASSIGNED_TASK);

	//

	while (task)
	{

		if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == type)
		{

			return task;
		}

		task = get_local_entity_child_succ (task, LIST_TYPE_ASSIGNED_TASK);
	}

	return NULL;
}





entity *get_local_group_member_landing_entity_from_keysite (entity *en)
{
	entity
		*group,
		*keysite,
		*landing;

	entity_sub_types
		group_type;

	landing = NULL;

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (group)
	{
		keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);

		if ((keysite) && (get_local_entity_type (keysite) == ENTITY_TYPE_KEYSITE))
		{
			group_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

			landing = get_local_entity_landing_entity (keysite, group_database [group_type].default_landing_type);
		}
	}

	return landing;
}





entity *get_local_group_member_landing_entity_from_task (entity *en)
{
	entity
		*task,
		*guide,
		*landing;

	landing = NULL;

	guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

	if (guide)
	{

		task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

		if (task)
		{

			landing = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);
		}
	}

	return landing;
}



























































































































































































































































































































































































































































































































int check_available_landing_route_lock (entity *landing_en)
{

	formation_type
		*formation_data;

	entity
		*wp;

	landing
		*raw;

	unsigned int
		mask,
		new_mask;

	raw = (landing *) get_local_entity_data (landing_en);

	wp = get_local_landing_entity_route (landing_en, ENTITY_SUB_TYPE_TASK_LANDING);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	mask = (1 << formation_data->number_in_formation) - 1;

	new_mask = (~raw->landing_lock);// & (~raw->landed_lock);

	return (mask & new_mask);
}





int check_landing_route_lock_clear (entity *landing_en, entity *en)
{

	formation_type
		*formation_data;

	entity
		*wp;

	landing
		*raw;

	unsigned int
		formation_landing;

	raw = (landing *) get_local_entity_data (landing_en);

	formation_landing = get_local_entity_int_value (en, INT_TYPE_FORMATION_POSITION);

	wp = get_local_landing_entity_route (landing_en, ENTITY_SUB_TYPE_TASK_LANDING);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	formation_landing %= formation_data->number_in_formation;

	return (!(raw->landing_lock & (1 << formation_landing)));
}





int check_takeoff_route_lock_clear (entity *landing_en, entity *en)
{

	formation_type
		*formation_data;

	entity
		*wp;

	landing
		*raw;

	unsigned int
		formation_takeoff;

	raw = (landing *) get_local_entity_data (landing_en);

	formation_takeoff = get_local_entity_int_value (en, INT_TYPE_FORMATION_POSITION);

	wp = get_local_landing_entity_route (landing_en, ENTITY_SUB_TYPE_TASK_TAKEOFF);

	formation_data = get_formation ((formation_types) get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION));

	formation_takeoff %= formation_data->number_in_formation;

	return (!(raw->takeoff_lock & (1 << formation_takeoff)));
}





int check_landing_site_lock_clear (entity *landing_en, entity *en)
{

	landing
		*raw;

	unsigned int
		formation_landing;

	raw = (landing *) get_local_entity_data (landing_en);

	formation_landing = get_local_entity_int_value (en, INT_TYPE_FORMATION_POSITION);

	return (!(raw->landed_lock & (1 << formation_landing)));
}





void release_mobile_entity_landing_locks (entity *en)
{
	entity
		*task,
		*group,
		*guide,
		*keysite,
		*landing_en;

	int
		landing_flag;







	landing_flag = FALSE;

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	ASSERT (group);

	guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

	if (guide)
	{
		task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

		if (task)
		{
			switch (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE))
			{
				case ENTITY_SUB_TYPE_TASK_LANDING:
				{






					landing_en = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);

					notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_ROUTE, landing_en, en);

					landing_flag = TRUE;

					break;
				}

				case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
				{







					landing_flag = TRUE;

					break;
				}

				case ENTITY_SUB_TYPE_TASK_TAKEOFF:
				{






					landing_en = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);

					notify_local_entity (ENTITY_MESSAGE_UNLOCK_TAKEOFF_ROUTE, landing_en, en);

					//break;
				}
				default:
				{
			
					keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);
			
					if ((keysite) && (get_local_entity_type (keysite) == ENTITY_TYPE_KEYSITE))
					{
						landing_en = get_local_entity_landing_entity (keysite, group_database [get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].default_landing_type);
	









										
						// unreserve landing lock
				
						if (landing_en)
						{
	
							entity
								*primary_task;
	
							primary_task = get_local_group_primary_task (group);
	
							if ((primary_task) && (get_local_entity_int_value (primary_task, INT_TYPE_ASSESS_LANDING)))
							{
	
								notify_local_entity (ENTITY_MESSAGE_UNRESERVE_LANDING_SITE, landing_en, en, 1);
							}
						}
					}
					
					break;
				}
			}
		}
	}

	if ((get_local_entity_int_value (en, INT_TYPE_LANDED)) || (landing_flag))
	{

		keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);

		ASSERT (keysite);

		if (get_local_entity_type (keysite) != ENTITY_TYPE_KEYSITE)
		{

			ASSERT (get_local_entity_type (en) == ENTITY_TYPE_SHIP_VEHICLE);

			ASSERT (get_local_entity_type (keysite) == ENTITY_TYPE_FORCE);







			return;
		}

		landing_en = get_local_entity_landing_entity (keysite, group_database [get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].default_landing_type);

		ASSERT (landing_en);







		notify_local_entity (ENTITY_MESSAGE_UNLOCK_LANDING_SITE, landing_en, en);
	}
	/*
	else
	{
						
		// unreserve landing lock

		keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);

		landing_en = get_local_entity_landing_entity (keysite, group_database [get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].default_landing_type);

		if (landing_en)
		{
			notify_local_entity (ENTITY_MESSAGE_UNRESERVE_LANDING_SITE, landing_en, en, 1);
		}
	}
	*/
}















































