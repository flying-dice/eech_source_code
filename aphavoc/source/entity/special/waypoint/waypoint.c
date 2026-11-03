

































































#include "project.h"





#define DEBUG_MODULE 0





entity *get_local_entity_first_waypoint (entity *en)
{
   entity
		*cwp,
		*task,
		*group,
      *first_waypoint;

   ASSERT (en);

	first_waypoint = NULL;

	//
	// get tasks waypoint route first
	//

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (group)
	{

		task = get_local_group_primary_task (group);

		if (task)
		{

			first_waypoint = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT);

			return first_waypoint;
		}
	}

	//
	// If not get current waypoint tasks route (might be the landing routes)
	//

	cwp = get_local_entity_current_waypoint (en);

	if (cwp)
	{
	
		task = get_local_entity_parent (cwp, LIST_TYPE_WAYPOINT);
	
		if (task)
		{
	
			first_waypoint = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT);
		}
	}

   return (first_waypoint);
}





entity *get_local_entity_current_waypoint (entity *en)
{
	entity
		*guide,
		*cwp;
		
	ASSERT (en);

	guide = get_local_entity_parent (en, LIST_TYPE_FOLLOWER);

	if (!guide)
	{
		return NULL;
	}

	cwp = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);

	return cwp;
}





entity *get_local_entity_last_waypoint (entity *en)
{
   entity
		*cwp,
		*task,
      *group,
      *last_waypoint;

   ASSERT (en);

	last_waypoint = NULL;

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (!group)
	{

		return NULL;
	}

	task = get_local_group_primary_task (group);

	if (!task)
	{

		cwp = get_local_entity_current_waypoint (en);

		if (cwp)
		{

			task = get_local_entity_parent (cwp, LIST_TYPE_WAYPOINT);
		}
	}

	if (task)
	{

		last_waypoint = get_local_entity_first_child (task, LIST_TYPE_WAYPOINT);

		while (get_local_entity_child_succ (last_waypoint, LIST_TYPE_WAYPOINT))
		{

			last_waypoint = get_local_entity_child_succ (last_waypoint, LIST_TYPE_WAYPOINT);
		}
	}

   return (last_waypoint);
}





entity *get_local_entity_next_waypoint (entity *en)
{

   entity
      *current_waypoint,
      *next_waypoint;

   ASSERT (en);

	next_waypoint = NULL;

   current_waypoint = get_local_entity_current_waypoint (en);

   if (current_waypoint)
   {

      next_waypoint = get_local_entity_child_succ (current_waypoint, LIST_TYPE_WAYPOINT);

      if (!next_waypoint)
      {

         next_waypoint = get_local_entity_first_waypoint (en);
      }
   }

   return (next_waypoint);
}





entity *get_local_entity_prev_waypoint (entity *en)
{

   entity
      *current_waypoint,
      *prev_waypoint;

   ASSERT (en);

	prev_waypoint = NULL;

   current_waypoint = get_local_entity_current_waypoint (en);

   if (current_waypoint)
   {

      prev_waypoint = get_local_entity_child_pred (current_waypoint, LIST_TYPE_WAYPOINT);

      if (!prev_waypoint)
      {

         prev_waypoint = get_local_entity_last_waypoint (en);
      }
   }

   return (prev_waypoint);
}

























































































































































































































































































