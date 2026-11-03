

































































#include "project.h"





#define DEBUG_MODULE				0

#define DEBUG_MODULE_PACK_ALL	(PACK_ENTIRE_SESSION || 0)





int
	number_of_entities = 0;

entity
	*entities = NULL,
	*first_free_entity = NULL,
	*first_used_entity = NULL;





//Xhit: These variables are for the create local only heap (030428)
int 
	start_of_local_entity_heap;

entity
	*first_free_local_entity = NULL,
	*first_used_local_entity = NULL;





static void reset_entity_heap (void)
{
	int
		i;

	ASSERT (entities);

	ASSERT ((number_of_entities > 0) && (number_of_entities < MAX_NUM_ENTITIES));

	memset (entities, 0, number_of_entities * sizeof (entity));

	//VJ 030508 if downwash num entities is 135000 else 125000
	if (command_line_downwash)
	{
		for (i = 0; i < number_of_entities; i++)
		{
			//Xhit: split the heap into two parts, normal entity heap and local created only heap. (030428)
			if ( (i < start_of_local_entity_heap - 1) || ( (i >= start_of_local_entity_heap) && (i < number_of_entities - 1) ) )
			{
				entities[i].succ = &entities[i + 1];
			}
			
			//Xhit: split the heap into two parts, normal entity heap and local created only heap. (030428)
			if ( ( (i > 0) && (i < start_of_local_entity_heap) ) || (i > start_of_local_entity_heap) )
			{
				entities[i].pred = &entities[i - 1];
			}
		}
	
		first_free_entity = entities;
	
		first_used_entity = NULL;

		//Xhit: Mark the start of the local only entity heap. (030428)
		first_free_local_entity = &entities[start_of_local_entity_heap];
		first_used_local_entity = NULL;
	}
	else
	{
		for (i = 0; i < number_of_entities; i++)
		{
			if (i < number_of_entities - 1)
			{
				entities[i].succ = &entities[i + 1];
			}
	
			if (i > 0)
			{
				entities[i].pred = &entities[i - 1];
			}
		}	
		
		first_free_entity = entities;
	
		first_used_entity = NULL;

	}	
}





//Xhit: Function for the local only entity heap. (030428)
static void check_no_local_entities_in_use (void)
{
	if (first_used_local_entity)
	{
		debug_colour_log (DEBUG_COLOUR_RED, "WARNING! Entities still in use");

		#if DEBUG_MODULE
		{
			entity
				*en;

			en = first_used_local_entity;

			while (en)
			{
				debug_log ("->ENTITY: %s (index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));

				en = get_local_entity_succ (en);
			}
		}
		#endif
	}
}





static void check_no_entities_in_use (void)
{
	if (first_used_entity)
	{
		debug_colour_log (DEBUG_COLOUR_RED, "WARNING! Entities still in use");

		#if DEBUG_MODULE
		{
			entity
				*en;

			en = first_used_entity;

			while (en)
			{
				debug_log ("->ENTITY: %s (index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));

				en = get_local_entity_succ (en);
			}
		}
		#endif
	}
}





void initialise_entity_heap (int num_entities)
{
	ASSERT ((num_entities > 0) && (num_entities < MAX_NUM_ENTITIES));

	number_of_entities = num_entities;

	//Xhit: The last 10000 entities in the heap is local only. (030428)
	//VJ 030508 if downwash 
	if (command_line_downwash)
		start_of_local_entity_heap = number_of_entities - 10000;

	entities = (entity *) malloc_heap_mem (number_of_entities * sizeof (entity));
	//VJ 050308 if not downwash than number_of_entities = 125000 (original)

	reset_entity_heap ();
}





void reinitialise_entity_heap (void)
{
	check_no_entities_in_use ();

	//Xhit: Check the local entity heap space also. (030428)
	//VJ 030508 if downwash 
	if (command_line_downwash)
		check_no_local_entities_in_use ();

	reset_entity_heap ();
}





void deinitialise_entity_heap (void)
{
	check_no_entities_in_use ();

	//Xhit: Check the local entity heap space also. (030428)
	//VJ 030508 if downwash 
	if (command_line_downwash)
		check_no_local_entities_in_use ();

	free_mem (entities);

	number_of_entities = 0;

	entities = NULL;

	first_free_entity = NULL;

	first_used_entity = NULL;

	//Xhit: Set the local entity heap pointers to NULL. (030428)
	//VJ 030508 if downwash 
	if (command_line_downwash)
	{
		first_free_local_entity = NULL;
		first_used_local_entity = NULL;
	}	

}





//Xhit: Same type of behaviour as get_free_entity. Here the local entity pointers are set instead. (030428)
entity *get_free_local_entity (int index)
{
	entity
		*en;

	if (index == ENTITY_INDEX_CREATE_LOCAL)
	{
		en = first_free_local_entity;

		if (en)
		{
			//
			// unlink local_entity from start of free list
			//

			first_free_local_entity = en->succ;

			if (en->succ)
			{
				en->succ->pred = NULL;
			}

			//
			// insert local_entity into start of used list (en->pred already set to NULL)
			//

			en->succ = first_used_local_entity;

			if (en->succ)
			{
				en->succ->pred = en;
			}

			first_used_local_entity = en;
		}
		else
		{
			debug_colour_log (DEBUG_COLOUR_RED, "WARNING! Failed to get a free local_entity");
		}
	}
	else
	{
		debug_fatal ("Entity already in use: %s (index = %d)", entity_type_names[index], index);
	}

	return (en);
}





//
// If index == ENTITY_INDEX_DONT_CARE then get next free entity, otherwise get free entity at given index.
//

entity *get_free_entity (int index)
{
	entity
		*en;

	if (index == ENTITY_INDEX_DONT_CARE)
	{
		en = first_free_entity;

		if (en)
		{
			//
			// unlink entity from start of free list
			//

			first_free_entity = en->succ;

			if (en->succ)
			{
				en->succ->pred = NULL;
			}

			//
			// insert entity into start of used list (en->pred already set to NULL)
			//

			en->succ = first_used_entity;

			if (en->succ)
			{
				en->succ->pred = en;
			}

			first_used_entity = en;
		}
		else
		{
			debug_colour_log (DEBUG_COLOUR_RED, "WARNING! Failed to get a free entity");
		}
	}
	else
	{
		ASSERT ((index >= 0) && (index < number_of_entities));

		en = get_local_entity_ptr (index);

		if (get_local_entity_type (en) != ENTITY_TYPE_UNKNOWN)
		{
			debug_fatal ("Entity already in use: %s (index = %d)", get_local_entity_type_name (en), index);
		}

		//
		// unlink entity from free list
		//

		if (en->pred)
		{
			en->pred->succ = en->succ;
		}
		else
		{
			first_free_entity = en->succ;
		}

		if (en->succ)
		{
			en->succ->pred = en->pred;
		}

		//
		// insert entity into start of used list
		//

		en->succ = first_used_entity;

		if (en->succ)
		{
			en->succ->pred = en;
		}

		en->pred = NULL;

		first_used_entity = en;
	}

	//debug_log ("EN_HEAP: index %d", get_local_entity_index (en));

	return (en);
}





//Xhit: Same type of behaviour as set_free_entity. Here the local entity pointers are set instead. (030428)
void set_free_local_entity (entity *en)
{
	ASSERT (en);

	en->type = ENTITY_TYPE_UNKNOWN;

	en->data = NULL;

	//
	// unlink entity from used list
	//

	if (en->pred)
	{
		en->pred->succ = en->succ;
	}
	else
	{
		first_used_local_entity = en->succ;
	}

	if (en->succ)
	{
		en->succ->pred = en->pred;
	}

	//
	// link entity into start of free list
	//

	en->succ = first_free_local_entity;

	if (en->succ)
	{
		en->succ->pred = en;
	}

	en->pred = NULL;

	first_free_local_entity = en;
}





void set_free_entity (entity *en)
{
	ASSERT (en);

	en->type = ENTITY_TYPE_UNKNOWN;

	en->data = NULL;

	//
	// unlink entity from used list
	//

	if (en->pred)
	{
		en->pred->succ = en->succ;
	}
	else
	{
		first_used_entity = en->succ;
	}

	if (en->succ)
	{
		en->succ->pred = en->pred;
	}

	//
	// link entity into start of free list
	//

	en->succ = first_free_entity;

	if (en->succ)
	{
		en->succ->pred = en;
	}

	en->pred = NULL;

	first_free_entity = en;
}





void pack_entity_safe_index (int index)
{
	ASSERT ((index == ENTITY_INDEX_DONT_CARE) || ((index >= 0) && (index < number_of_entities)));

	#if (DEBUG_MODULE_PACK_ALL)

	debug_log_entity_args (ENTITY_DEBUG_PACK, ENTITY_DEBUG_ENTITY_INDEX, NULL, index);

	#endif

	pack_signed_data (index, NUM_ENTITY_INDEX_PACK_BITS);
}





int unpack_entity_safe_index (void)
{
	int
		index;

	index = unpack_signed_data (NUM_ENTITY_INDEX_PACK_BITS);

	ASSERT ((index == ENTITY_INDEX_DONT_CARE) || ((index >= 0) && (index < number_of_entities)));

	#if (DEBUG_MODULE_PACK_ALL)

	debug_log_entity_args (ENTITY_DEBUG_UNPACK, ENTITY_DEBUG_ENTITY_INDEX, NULL, index);

	#endif

	//debug_log ("EN_HEAP: index %d", index);

	return (index);
}





void pack_entity_safe_ptr (entity *en)
{
	int
		index;

	index = get_local_entity_safe_index (en);

	pack_entity_safe_index (index);
}





entity *unpack_entity_safe_ptr (void)
{
	int
		index;

	entity
		*en;

	index = unpack_entity_safe_index ();

	en = get_local_entity_safe_ptr (index);

	return (en);
}




