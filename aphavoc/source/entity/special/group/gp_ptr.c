

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_ptr_value (entity *en, ptr_types type, void *ptr)
{
	group
		*raw;







	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_GROUP_LEADER:

		{
			entity
				*member;
				
			//
			// make an entity group leader by removing it from the member list, and inserting it at the front
			//

			member = (entity *) ptr;

			ASSERT (member);

			ASSERT (get_local_entity_parent (member, LIST_TYPE_MEMBER) == en);
	
			delete_local_entity_from_parents_child_list (member, LIST_TYPE_MEMBER);

			insert_local_entity_into_parents_child_list (member, LIST_TYPE_MEMBER, en, NULL);

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
	group
		*raw;

	void
		*ptr;

	raw = (group *) get_local_entity_data (en);

	switch (type)
	{

		case PTR_TYPE_GROUP_LEADER:

		{
			ptr = get_local_entity_first_child (en, LIST_TYPE_MEMBER);

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





// void overload_group_ptr_value_functions (entity_types type)










