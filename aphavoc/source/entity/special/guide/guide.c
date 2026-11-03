

































































#include "project.h"
#include "ai/taskgen/taskgen.h"





#define DEBUG_MODULE 0





static void initialise_guide_criteria (entity *en);





const char
	*guide_criteria_type_names [NUM_GUIDE_CRITERIA_TYPES] =
	{
		"RADIUS",
		"HEADING",
		"ALTITUDE",
		"TRANSMIT_DATA",
		"LAST_TO_REACH",	
		"WEAPON_VECTOR"
	};





const int
	guide_position_type_virtual [NUM_GUIDE_POSITION_TYPES] =
	{
		FALSE,					// ABSOLUTE
		FALSE,					// WAYPOINT
		TRUE,						// RELATIVE_WORLD_SPACE
		TRUE,						// RELATIVE_OBJECT_SPACE,
		TRUE,						// RELATIVE_CIRCULAR,
	};





entity *create_client_server_guide_entity (entity *task_en, entity *first_waypoint, unsigned int valid_members)
{
	entity
		*en,
		*wp;

	vec3d
		*pos;

	int
		guide_type,
		create_stack_attributes;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (task_en);










	if (first_waypoint)
	{
		//
		// first waypoint has been specified - but check it actually belongs to the task...
		//
		
		wp = first_waypoint;

		ASSERT (get_local_entity_parent (wp, LIST_TYPE_WAYPOINT) == task_en);
	}
	else
	{
		//
		// get first waypoint of the task
		//

		wp = get_local_entity_first_child (task_en, LIST_TYPE_WAYPOINT);
	}

	ASSERT (wp);

	//
	// set sub type (must be done before set position)
	//

	guide_type = get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_GUIDE_TYPE);

	//
	// get waypoint position 
	//

	pos = get_local_entity_vec3d_ptr (wp, VEC3D_TYPE_POSITION);

	ASSERT (pos);
	
	//
	// create guide entity
	//

	create_stack_attributes = force_local_entity_create_stack_attributes;

	if (get_comms_data_flow () == COMMS_DATA_FLOW_RX)
	{
		set_force_local_entity_create_stack_attributes (TRUE);
	}

	en = create_client_server_entity
	(
		ENTITY_TYPE_GUIDE,
		ENTITY_INDEX_DONT_CARE,
		ENTITY_ATTR_PARENT (LIST_TYPE_GUIDE, task_en),
		ENTITY_ATTR_PARENT (LIST_TYPE_CURRENT_WAYPOINT, wp),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, guide_type),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_VALID_GUIDE_MEMBERS, valid_members),
		ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, pos->x, pos->y, pos->z),
		ENTITY_ATTR_END
	);

	set_force_local_entity_create_stack_attributes (create_stack_attributes);

	return en;
}





int validate_local_guide_entity (entity *en)
{
	entity
		*task;

	//
	// guide should always have a task and a group
	//

	task = get_local_entity_parent (en, LIST_TYPE_GUIDE);

	ASSERT (task);

	ASSERT (get_local_entity_parent (en, LIST_TYPE_GUIDE_STACK));

	//
	// Guide cannot be terrain following if waypoint has altitude criteria
	//

	if (get_local_entity_int_value (en, INT_TYPE_TERRAIN_FOLLOW_MODE) != GUIDE_TERRAIN_FOLLOW_NONE)
	{
		ASSERT (!get_guide_criteria_valid (en, GUIDE_CRITERIA_ALTITUDE));
	}

	//
	// Guide shouldn't have both heading AND weapon_vector criteria set
	//

	if (get_guide_criteria_valid (en, GUIDE_CRITERIA_HEADING))
	{
		ASSERT (!get_guide_criteria_valid (en, GUIDE_CRITERIA_WEAPON_VECTOR));
	}

	//
	// Engage tasks with valid members should never be on the stack unless they are being done 
	//

	if (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_ENGAGE)
	{
		if (get_local_entity_int_value (en, INT_TYPE_VALID_GUIDE_MEMBERS) != TASK_ASSIGN_NO_MEMBERS)
		{
			ASSERT (get_local_entity_first_child (en, LIST_TYPE_FOLLOWER));
		}
	}

	return TRUE;
}





int check_guide_reached_waypoint (entity *en)
{
	entity
		*leader,
		*member,
		*group;

	int
		member_number,
		valid_members;

	//
	// find task leader
	//

	leader = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	if (!leader)
	{
		return FALSE;
	}

	if (get_guide_criteria_valid (en, GUIDE_CRITERIA_LAST_TO_REACH))
	{
		//
		// check all valid members reached
		//

		group = get_local_entity_parent (en, LIST_TYPE_GUIDE_STACK);

		ASSERT (group);

		valid_members = get_local_entity_int_value (en, INT_TYPE_VALID_GUIDE_MEMBERS);

		member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);
		
		while (member)
		{
			member_number = (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER));

			if (valid_members & member_number)
			{
				if (!notify_local_entity (ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE, member, en))
				{






					return FALSE;
				}
			}

			member = get_local_entity_child_succ (member, LIST_TYPE_MEMBER);
		}
	}
	else
	{
		//
		// check task leader reached
		//

		if (!notify_local_entity (ENTITY_MESSAGE_CHECK_MOBILE_REACHED_GUIDE, leader, en))
		{






			return FALSE;
		}
	}

	return TRUE;
}





int check_guide_waypoint_action_reached (entity *en)
{
	float
		range;

	entity
		*wp,
		*leader;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	//
	// find task leader
	//

	leader = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	if (!leader)
	{
		return FALSE;
	}

	//
	// Check if landed
	//

	if (get_local_entity_int_value (leader, INT_TYPE_LANDED))
	{
		return FALSE;
	}

	//
	// check task leader reached action radius
	//

	wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (wp);

	range = get_local_entity_float_value (leader, FLOAT_TYPE_DISTANCE);
	
	if (range > get_local_entity_waypoint_database_action_radius_value (wp, leader))
	{






		return FALSE;
	}
		
	return TRUE;
}





int set_guide_new_waypoint (entity *en, entity *wp)
{
	entity
		*task,
		*group,
		*leader,
		*member,
		*objective,
		*current_wp;

	vec3d
		new_pos,
		*leader_pos;

	float
		range,
		velocity;

	entity_sub_types
		guide_type,
		group_type,
		waypoint_type;
		
	entity_types
		member_type;

	if (get_comms_model () != COMMS_MODEL_SERVER)
	{
		transmit_entity_comms_message (ENTITY_COMMS_SET_WAYPOINT, en, wp);

		return TRUE;
	}

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	ASSERT (wp);

	current_wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (current_wp);

	group = get_local_entity_parent (en, LIST_TYPE_GUIDE_STACK);

	ASSERT (group);

	group_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

	member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

	ASSERT (member);

	member_type = group_database [group_type].default_entity_type;

	waypoint_type = get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE);

	//
	// set sub type (must be done before set position)
	//

	guide_type = get_local_entity_int_value (wp, INT_TYPE_WAYPOINT_GUIDE_TYPE);

	set_client_server_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE, guide_type);

	//
	// set positions
	//

	if (guide_position_type_virtual [guide_database [guide_type].position_type])
	{
		task = get_local_entity_parent (wp, LIST_TYPE_WAYPOINT);

		ASSERT (task);

		objective = get_local_entity_parent (task, LIST_TYPE_TASK_DEPENDENT);

		ASSERT (objective);

		new_pos.x = 0.0;
		new_pos.y = 0.0;
		new_pos.z = get_waypoint_database_reached_radius_value (waypoint_type, member_type);

		set_client_server_guide_entity_new_position (en, &new_pos, objective);
	}
	else
	{
		get_local_entity_vec3d (wp, VEC3D_TYPE_POSITION, &new_pos);

////Moje 030923 Todd's suggestion to solve the FARP bug Start

////Moje 030924 -didn't help                 new_pos.y = get_3d_terrain_elevation (new_pos.x, new_pos.z);

////Moje 030923 Todd's suggestion to solve the FARP bug End

		set_client_server_guide_entity_new_position (en, &new_pos, NULL);
	}

	//
	// Set distance to task leader
	//

	leader = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	if (leader)
	{
		leader_pos = get_local_entity_vec3d_ptr (leader, VEC3D_TYPE_POSITION);

		range = get_2d_range (leader_pos, &new_pos);

		set_local_entity_float_value (leader, FLOAT_TYPE_DISTANCE, range);
	}

	//
	// set velocity
	//

	velocity = get_waypoint_database_velocity_value (waypoint_type, member_type);

	velocity *= get_local_entity_float_value (member, FLOAT_TYPE_CRUISE_VELOCITY);

	set_client_server_entity_float_value (en, FLOAT_TYPE_VELOCITY, velocity);

	//
	// seperate check for waypoint action ?
	//

	set_local_entity_int_value (en, INT_TYPE_CHECK_WAYPOINT_ACTION, get_local_entity_int_value (wp, INT_TYPE_CHECK_WAYPOINT_ACTION));

	//
	// set new waypoint
	//

	set_client_server_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT, wp);

	//
	// set new guide reached criteria
	//

	initialise_guide_criteria (en);

	return TRUE;
}





int set_guide_next_waypoint (entity *en)
{
	entity
		*current_wp,
		*next_wp;

	current_wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (current_wp);

	next_wp = get_local_entity_child_succ (current_wp, LIST_TYPE_WAYPOINT);

	if (next_wp)
	{
		set_guide_new_waypoint (en, next_wp);
	}

	return TRUE;
}





int set_guide_prev_waypoint (entity *en)
{
	entity
		*current_wp,
		*prev_wp;

	current_wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (current_wp);

	prev_wp = get_local_entity_child_pred (current_wp, LIST_TYPE_WAYPOINT);

	if (prev_wp)
	{
		set_guide_new_waypoint (en, prev_wp);
	}

	return TRUE;
}
















































































































































































void attach_group_member_to_guide_entity (entity *member, entity *guide)
{
	ASSERT (member);
	ASSERT (guide);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);















	//
	// follower link
	//

	delete_local_entity_from_parents_child_list (member, LIST_TYPE_FOLLOWER);

	insert_local_entity_into_parents_child_list (member, LIST_TYPE_FOLLOWER, guide, NULL);

	transmit_entity_comms_message (ENTITY_COMMS_SWITCH_LIST, member, LIST_TYPE_FOLLOWER, guide, LIST_TYPE_FOLLOWER);

	//
	// set mobile range to maximum (okay, so I've just picked an arbitrary big number coz it may need to be packed up)
	//

	set_local_entity_float_value (member, FLOAT_TYPE_DISTANCE, 32000.0);
}





void attach_group_to_guide_entity (entity *group, entity *guide)
{
	ASSERT (group);
	ASSERT (guide);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	//
	// guide stack
	//

	ASSERT (get_local_entity_parent (guide, LIST_TYPE_GUIDE_STACK) == NULL);

	insert_local_entity_into_parents_child_list (guide, LIST_TYPE_GUIDE_STACK, group, NULL);

	transmit_entity_comms_message (ENTITY_COMMS_SWITCH_LIST, guide, LIST_TYPE_GUIDE_STACK, group, LIST_TYPE_GUIDE_STACK);

	//
	// set-up guide criteria relevant to the group type
	//

	initialise_guide_criteria (guide);

	//
	// Set guide velocity with respect to the group type
	//
	{
		entity
			*wp,
			*member;

		float
			velocity;

		entity_sub_types
			group_type,
			waypoint_type;

		wp = get_local_entity_parent (guide, LIST_TYPE_CURRENT_WAYPOINT);

		ASSERT (wp);

		member = get_local_entity_first_child (group, LIST_TYPE_MEMBER);

		ASSERT (member);
		
		group_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

		waypoint_type = get_local_entity_int_value (wp, INT_TYPE_ENTITY_SUB_TYPE);

		velocity = get_waypoint_database_velocity_value (waypoint_type, group_database [group_type].default_entity_type);

		velocity *= get_local_entity_float_value (member, FLOAT_TYPE_CRUISE_VELOCITY);

		set_client_server_entity_float_value (guide, FLOAT_TYPE_VELOCITY, velocity);
	}
}





void delete_group_member_from_guide_entity (entity *member, entity *guide)
{
	unsigned int
		valid_members,
		member_number;
			
	ASSERT (member);
	ASSERT (guide);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	valid_members = get_local_entity_int_value (guide, INT_TYPE_VALID_GUIDE_MEMBERS);

	ASSERT (valid_members != TASK_ASSIGN_ALL_MEMBERS);

	member_number = (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER));
	
	ASSERT (valid_members & member_number);

	//
	// adjust guide valid members
	//

	valid_members &= (~member_number);

	//
	// if valid members empty - terminate the task (which will also destroy the guide)
	//

	if (!valid_members)
	{
		entity
			*task;

		task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

		ASSERT (task);
		











		notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, task, NULL, TASK_TERMINATED_GROUP_DESTROYED);
	}
	else
	{
		set_client_server_entity_int_value (guide, INT_TYPE_VALID_GUIDE_MEMBERS, valid_members);
	}
}





void delete_group_member_from_engage_guide (entity *member, entity *guide, int engage_enemy)
{
	entity
		*task,
		*group;

	unsigned int
		valid_members,
		member_number;
			
	ASSERT (member);
	ASSERT (guide);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	valid_members = get_local_entity_int_value (guide, INT_TYPE_VALID_GUIDE_MEMBERS);

	member_number = (1 << get_local_entity_int_value (member, INT_TYPE_GROUP_MEMBER_NUMBER));
	
	ASSERT (valid_members == member_number);

	group = get_local_entity_parent (guide, LIST_TYPE_GUIDE_STACK);

	ASSERT (group);

	task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

	ASSERT (task);

	ASSERT (get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_TASK_ENGAGE);
		
	destroy_client_server_entity (guide);

	reassign_group_members_to_valid_tasks (group, task, valid_members, engage_enemy);
}



































































entity *get_local_group_primary_guide (entity *en)
{
	entity
		*guide,
		*task,
		*primary_guide;

	entity_sub_types
		sub_type;

	int
		count;

	ASSERT (en);

	count = 0;

	primary_guide = NULL;

	guide = get_local_entity_first_child (en, LIST_TYPE_GUIDE_STACK);

	while (guide)
	{
		task = get_local_entity_parent (guide, LIST_TYPE_GUIDE);

		sub_type = get_local_entity_int_value (task, INT_TYPE_ENTITY_SUB_TYPE);

		if (task_database [sub_type].primary_task)
		{
			count ++;

			primary_guide = guide;

			//
			// could break out / return here, but keep going to check that there is only one primary task
			//								
		}

		guide = get_local_entity_child_succ (guide, LIST_TYPE_GUIDE_STACK);
	}

	ASSERT (count <= 1);

	return primary_guide;
}





entity *get_local_entity_primary_guide (entity *en)
{
	entity
		*group;

	ASSERT (en);

	group = get_local_entity_parent (en, LIST_TYPE_MEMBER);

	if (group)
	{
		return get_local_group_primary_guide (group);
	}
	else
	{
		return NULL;
	}
}





void get_local_guide_entity_pointers (entity *en, entity **aggressor, entity **waypoint, entity **task, entity **objective)
{
	entity
		*agg,
		*wp,
		*ts,
		*obj;

	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	agg = (entity *) get_local_entity_ptr_value (en, PTR_TYPE_TASK_LEADER);

	if (aggressor)
	{
		ASSERT (agg);
	}

	wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (wp);

	ts = get_local_entity_parent (en, LIST_TYPE_GUIDE);

	ASSERT (ts);

	obj = get_local_entity_parent (ts, LIST_TYPE_TASK_DEPENDENT);

	if (aggressor) *aggressor = agg;

	if (waypoint) *waypoint = wp;

	if (task) *task = ts;

	if (objective) *objective = obj;
}




































































































































































































































int get_guide_criteria_valid (entity *en, guide_criteria_types type)
{
	guide
		*raw;

	ASSERT (en);

	ASSERT (type < NUM_GUIDE_CRITERIA_TYPES);

	raw = (guide *) get_local_entity_data (en);

	return raw->criteria [type].valid;
}





float get_guide_criteria_value (entity *en, guide_criteria_types type)
{
	guide
		*raw;

	ASSERT (en);

	ASSERT (type < NUM_GUIDE_CRITERIA_TYPES);

	raw = (guide *) get_local_entity_data (en);

	return raw->criteria [type].value;
}
































void set_local_guide_criteria_valid (entity *en, guide_criteria_types type, int valid, float value)
{
	guide
		*raw;

	ASSERT (en);

	ASSERT (type < NUM_GUIDE_CRITERIA_TYPES);

	raw = (guide *) get_local_entity_data (en);

	raw->criteria [type].valid = valid;
	raw->criteria [type].value = value;
}





void initialise_guide_criteria (entity *en)
{
	entity
		*current_wp,
		*group;

	entity_sub_types
		waypoint_type,
		group_type,
		guide_type;

	entity_types
		mobile_type;

	int
		loop_;

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	guide_type = get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE);

	//
	// get current waypoint
	//

	current_wp = get_local_entity_parent (en, LIST_TYPE_CURRENT_WAYPOINT);

	ASSERT (current_wp);

	waypoint_type = get_local_entity_int_value (current_wp, INT_TYPE_ENTITY_SUB_TYPE);

	//
	// get follower type
	//

	group = get_local_entity_parent (en, LIST_TYPE_GUIDE_STACK);

	ASSERT (group);

	group_type = get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE);

	mobile_type = group_database [group_type].default_entity_type;

	//
	// get values from GUIDE DATABASE
	//

	for (loop_ = 0; loop_ < NUM_GUIDE_CRITERIA_TYPES; loop_++)
	{
		guide_criteria_types
			loop;

		loop = (guide_criteria_types) loop_;

		if (guide_database [guide_type].criteria [loop].valid)
		{
			set_client_server_guide_criteria_valid (en, loop, TRUE, guide_database [guide_type].criteria [loop].value);
		}
		else
		{
			if (get_guide_criteria_valid (en, loop))
			{
				set_client_server_guide_criteria_valid (en, loop, FALSE, 0.0);
			}
		}
	}
	 
	//
	// get values from WAYPOINT DATABASE
	//

	set_client_server_guide_criteria_valid (en, GUIDE_CRITERIA_RADIUS, TRUE, get_waypoint_database_reached_radius_value (waypoint_type, mobile_type));

	set_client_server_guide_criteria_valid (en, GUIDE_CRITERIA_TRANSMIT_DATA, get_waypoint_database_transmit_recon_flag (waypoint_type, mobile_type), 0.0);

	set_client_server_guide_criteria_valid (en, GUIDE_CRITERIA_LAST_TO_REACH, get_waypoint_database_last_to_reach_flag (waypoint_type, mobile_type), 0.0);
}








