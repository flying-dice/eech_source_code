

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	guide
		*raw;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_POSITION_ENTITY:

		{
			raw->position_entity = (entity *) ptr;

			break;
		}

		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}
}





static void *get_local_ptr_value (entity *en, ptr_types type)
{
	guide
		*raw;

	void
		*ptr;

	#if DEBUG_MODULE

	debug_log_entity_args (ENTITY_DEBUG_LOCAL, ENTITY_DEBUG_PTR_VALUE, en, type, ptr);

	#endif

	raw = (guide *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_POSITION_ENTITY:

		{
			ptr = raw->position_entity;

			break;
		}

		case PTR_TYPE_TASK_LEADER:

		{
			entity
				*group,
				*group_leader;

			//
			// Task Leader is the first member in the follower list UNLESS the group leader is one of the followers
			//

			group = get_local_entity_parent (en, LIST_TYPE_GUIDE_STACK);

			ASSERT (group);

			group_leader = (entity *) get_local_entity_ptr_value (group, PTR_TYPE_GROUP_LEADER);

			if (group_leader)
			{
				if (get_local_entity_parent (group_leader, LIST_TYPE_FOLLOWER) == en)
				{
					ptr = group_leader;
				}
				else
				{
					ptr = get_local_entity_first_child (en, LIST_TYPE_FOLLOWER);
				}
			}
			else
			{
				ptr = NULL;
			}

			break;
		}

		default:

		{
			debug_fatal_invalid_ptr_type (en, type);

			break;
		}
	}

	return (ptr);
}





// void overload_guide_ptr_value_functions (entity_types type)
void overload_guide_ptr_value_functions (void)
{

	fn_set_local_entity_ptr_value				[ENTITY_TYPE_GUIDE][PTR_TYPE_POSITION_ENTITY]	= set_local_ptr_value;
	fn_get_local_entity_ptr_value				[ENTITY_TYPE_GUIDE][PTR_TYPE_POSITION_ENTITY]	= get_local_ptr_value;

	fn_get_local_entity_ptr_value				[ENTITY_TYPE_GUIDE][PTR_TYPE_TASK_LEADER]			= get_local_ptr_value;
}




