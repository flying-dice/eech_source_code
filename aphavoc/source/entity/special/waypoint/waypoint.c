

































































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





void get_local_waypoint_formation_position_offset (int formation_position, entity *wp, vec3d *offset)
{
   int
      type,
      index,
		formation_count;

   entity
      *wp_pred,
      *wp_succ;

   vec3d
      *wp_pos,
      pred_pos,
      succ_pos,
      first,
      second,
      resultant,
      perpendicular;

	float
		cos_heading,
		sin_heading;

	formation_type
		*formation;

	waypoint
		*raw;

	ASSERT (wp);

	ASSERT (offset);

	raw = (waypoint *) get_local_entity_data (wp);

	//
	// default values
	//

	perpendicular.x = 1.0;
	perpendicular.y = 0.0;
	perpendicular.z = 0.0;

	//
	// get current waypoint position
	//	
	
	wp_pos = get_local_entity_vec3d_ptr (wp, VEC3D_TYPE_POSITION);
	
	//
	// find perpendicular vector (but only for non-relative waypoints)
	//

	if (raw->position_type != POSITION_TYPE_RELATIVE)
	{
		wp_pred = get_local_entity_child_pred (wp, LIST_TYPE_WAYPOINT);
	
		wp_succ = get_local_entity_child_succ (wp, LIST_TYPE_WAYPOINT);
	
		//
		// get previous waypoint position
		//	
	
		if (wp_pred)
		{
			get_local_entity_vec3d (wp_pred, VEC3D_TYPE_POSITION, &pred_pos);
		}
		else
		{
			pred_pos.x = wp_pos->x;
			pred_pos.y = wp_pos->y;
			pred_pos.z = wp_pos->z;
		}
	
		//
		// get next waypoint position
		//	
	
		if (wp_succ)
		{
			get_local_entity_vec3d (wp_succ, VEC3D_TYPE_POSITION, &succ_pos);
		}
		else
		{
			succ_pos.x = wp_pos->x;
			succ_pos.y = wp_pos->y;
			succ_pos.z = wp_pos->z;
		}
	
		//
		// Calculate tangent vector at waypoint 
		//
		
		if ((wp_pred) || (wp_succ))
		{
			first.x = wp_pos->x - pred_pos.x;
			first.y = 0.0;
			first.z = wp_pos->z - pred_pos.z;
		
			normalise_any_3d_vector (&first);
		
			second.x = succ_pos.x - wp_pos->x;
			second.y = 0.0;
			second.z = succ_pos.z - wp_pos->z;
		
			normalise_any_3d_vector (&second);
		
			resultant.x = second.x + first.x;
			resultant.y = 0.0;
			resultant.z = second.z + first.z;
	
			cos_heading = cos (raw->heading);
			sin_heading = sin (raw->heading);
		
			perpendicular.x = (cos_heading * -resultant.z) + (sin_heading * resultant.x);
			perpendicular.y = 0.0;
			perpendicular.z = (cos_heading * resultant.x) - (sin_heading * -resultant.z);

			if ((perpendicular.x == 0) && (perpendicular.z == 0))
			{

				perpendicular.x = 1.0;
				perpendicular.y = 0.0;
				perpendicular.z = 0.0;
			}
			else
			{
		
				normalise_any_3d_vector (&perpendicular);
			}
		}
	}

	//
	// get formation data
	//
		
	type = get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_FORMATION);
	
	formation = get_formation ((formation_types) type);
	
	formation_count = formation->number_in_formation;
	
	if (formation_count)
	{
		index = formation_position % formation_count;			// needed ? DL
	}
	else
	{
		index = 0;		// can we have a formation with 0 positions ??? DL
	}
	
	//
	// Calculate position offset in world coordinates
	//

	offset->x = (perpendicular.x * formation->sites [index].x) -		
					(perpendicular.z * formation->sites [index].z);
	
	offset->y = formation->sites [index].y;
	
	offset->z = (perpendicular.z * formation->sites [index].x) +
					(perpendicular.x * formation->sites [index].z);
}





void get_local_entity_formation_position_offset (entity *en, entity *wp, vec3d *offset)
{
   int
		formation_position;

	ASSERT (en);

	ASSERT (wp);

	ASSERT (offset);

	formation_position = get_local_entity_int_value (en, INT_TYPE_FORMATION_POSITION);

	get_local_waypoint_formation_position_offset (formation_position, wp, offset);
}





void get_local_waypoint_formation_position (int formation_position, entity *wp, vec3d *pos)
{
	vec3d
		offset;

	ASSERT (wp);

	ASSERT (pos);

	get_local_entity_vec3d (wp, VEC3D_TYPE_POSITION, pos);

	get_local_waypoint_formation_position_offset (formation_position, wp, &offset);

	pos->x += offset.x;
	pos->y += offset.y;
	pos->z += offset.z;
}





void get_local_entity_formation_position (entity *en, entity *wp, vec3d *pos)
{
	vec3d
		offset;

	ASSERT (en);

	ASSERT (wp);

	ASSERT (pos);

	get_local_entity_vec3d (wp, VEC3D_TYPE_POSITION, pos);

	get_local_entity_formation_position_offset (en, wp, &offset);

	pos->x += offset.x;
	pos->y += offset.y;
	pos->z += offset.z;
}





void get_local_waypoint_simple_formation_position (int formation_position, vec3d *origin, int type, vec3d *pos)
{
   int
      index;

   vec3d
      perpendicular;

	formation_type
		*formation;

	perpendicular.x = 1.0;
	perpendicular.y = 0.0;
	perpendicular.z = 0.0;

	formation = get_formation ((formation_types) type);

	ASSERT (formation);
	
   index = formation->number_in_formation;

   if (index)
   {
      index = formation_position % index;
   }

   pos->x = origin->x +
            perpendicular.x * formation->sites [index].x -	
            perpendicular.z * formation->sites [index].z;

   pos->y = origin->y + formation->sites [index].y;

   pos->z = origin->z +
            perpendicular.z * formation->sites [index].x +
            perpendicular.x * formation->sites [index].z;
}




