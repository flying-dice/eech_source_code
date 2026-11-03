

































































#include "project.h"





#define DEBUG_MODULE				0

#define DEBUG_MODULE_PACK_ALL	(PACK_ENTIRE_SESSION || 0)





struct COMMS_ENTITY_SMOKE_LIST_CREATION_TYPE
{

	entity
		*en;

	meta_smoke_list_types
		meta_smoke_type;

	object_3d_sub_object_index_numbers
		attachment_point;

	int
		count,
		*entity_index_list;

	struct COMMS_ENTITY_SMOKE_LIST_CREATION_TYPE
		*next;
};

typedef struct COMMS_ENTITY_SMOKE_LIST_CREATION_TYPE comms_entity_smoke_list_creation_type;





struct COMMS_ENTITY_SOUND_EFFECT_CREATION_TYPE
{

	entity
		*en;

	entity_sub_types
		sub_type;

	int
		side,
		valid,
		entity_index,
		sample_index;

	struct COMMS_ENTITY_SOUND_EFFECT_CREATION_TYPE
		*next;
};

typedef struct COMMS_ENTITY_SOUND_EFFECT_CREATION_TYPE comms_entity_sound_effect_creation_type;





struct COMMS_ENTITY_LIST_VALIDATION_TYPE
{

	entity
		*en;

	list_types
		list_type;

	struct COMMS_ENTITY_LIST_VALIDATION_TYPE
		*next;
};

typedef struct COMMS_ENTITY_LIST_VALIDATION_TYPE comms_entity_list_validation_type;





comms_entity_smoke_list_creation_type
	*entity_smoke_list_creation;

comms_entity_sound_effect_creation_type
	*entity_sound_effect_creation;

comms_entity_list_validation_type
	*entity_list_link_validation,
	*entity_list_root_validation;





struct ENTITY_PACK_SIZE_TYPE
{
	int
		count,
		size;
};

typedef struct ENTITY_PACK_SIZE_TYPE entity_pack_size_type;

entity_pack_size_type
	entity_pack_size_array [NUM_ENTITY_TYPES];





static void repair_landing_entity_locks (pack_modes mode);

static void process_entity_list_link_validation (void);

static void process_entity_list_root_validation (void);

static void process_entity_smoke_list_creation (void);

static void process_entity_sound_effect_creation (void);

//static void kill_entities_awaiting_update (void);





int pack_session (unsigned char *buffer, int *buffer_size, pack_modes mode)
{
	entity
		*en;

	int
		packed_data_size,
		pack_buffer_overflow;









	ASSERT (buffer);

	ASSERT (*buffer_size > 0);

	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	#if !DEBUG_MODULE_PACK_ALL

	disable_debug_log_entity_args ();

	#endif









	open_pack_buffer (buffer, *buffer_size);

	//
	// kill any entities that are marked 'kill next update'
	//

	//kill_entities_awaiting_update ();

	//
	// Unlink all killed "local_only" entities from lists. Just
	//









	//
	// pack sector entities first
	//

	pack_local_sector_data (mode);







	//
	// pack pylon entities
	//

	pack_local_pylon_data (mode);







	//
	// pack segment entities
	//

	pack_local_segment_data (mode);

	//
	// pack bridge entities
	//

	pack_local_bridge_data (mode);

	//
	// pack Session and Force entities first
	//

	pack_local_session_data (mode);






	
	en = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (en)
	{
		pack_local_force_data (en, mode);

		en = get_local_entity_child_succ (en, LIST_TYPE_FORCE);
	}






	
	//
	// pack remaining entities
	//

	en = get_local_entity_list ();









	while (en)
	{
		pack_local_entity_data (en, mode);















		en = get_local_entity_succ (en);
	}

	//
	// mark end of session data
	//

	pack_entity_type (ENTITY_TYPE_UNKNOWN);







	//
	// pack other entity data
	//

	pack_local_keysite_data (mode);







	pack_local_regen_data (mode);







	pack_local_city_building_data (mode);







	pack_local_scenic_data (mode);







	pack_local_site_data (mode);







	pack_local_site_updatable_data (mode);







	pack_local_anti_aircraft_data (mode);







	pack_local_group_data (mode);







	pack_local_division_database (mode);

	pack_local_message_log (mode);

	pack_group_callsign_database (mode);

	#if LANDING_ROUTE_CHECK

	pack_debug_entity_landing_locks (mode);

	#endif

	pack_buffer_overflow = get_pack_buffer_overflow ();

	packed_data_size = close_pack_buffer ();










	*buffer_size = packed_data_size;

	#if !DEBUG_MODULE_PACK_ALL

	enable_debug_log_entity_args ();

	#endif






















	//
	//
	//

	return (pack_buffer_overflow);
}





int unpack_session (unsigned char *buffer, int buffer_size, pack_modes mode)
{
	entity
		*en;

	entity_types
		type;

	int
		unpack_buffer_overflow;

	ASSERT (buffer);

	ASSERT (buffer_size > 0);

	ASSERT ((mode >= 0) && (mode < NUM_PACK_MODES));

	#if !DEBUG_MODULE_PACK_ALL

	disable_debug_log_entity_args ();

	#endif

	set_comms_data_flow (COMMS_DATA_FLOW_RX);










	set_gunship_screen_progress_message (GUNSHIPS_SCREEN_GENERATING_CAMPAIGN_TEXT);
	initialise_entity_list_validation ();


	open_unpack_buffer (buffer, buffer_size);

	//
	// unpack sector entities first
	//


	update_gunships_screen_progress_indicator ();


	unpack_local_sector_data (mode);

	//
	// unpack pylon entities
	//


	update_gunships_screen_progress_indicator ();


	unpack_local_pylon_data (mode);

	//
	// unpack segment entities
	//


	update_gunships_screen_progress_indicator ();


	unpack_local_segment_data (mode);

	//
	// unpack bridge entities
	//


	update_gunships_screen_progress_indicator ();


	unpack_local_bridge_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_session_data (mode);


	update_gunships_screen_progress_indicator ();


	en = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE);

	while (en)
	{
		unpack_local_force_data (en, mode);

		en = get_local_entity_child_succ (en, LIST_TYPE_FORCE);
	}


	if (mode == PACK_MODE_CLIENT_SESSION)
	{
		reinitialise_client_ai_system ();
	}


	//
	// unpack remaining entities
	//

	type = unpack_entity_type ();

	while (type != ENTITY_TYPE_UNKNOWN && type < NUM_ENTITY_TYPES)
	{


		update_gunships_screen_progress_indicator ();








		unpack_local_entity_data (NULL, type, mode);

		type = unpack_entity_type ();
	}

	//
	// unpack other entity data
	//


	update_gunships_screen_progress_indicator ();

	
	unpack_local_keysite_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_regen_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_city_building_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_scenic_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_site_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_site_updatable_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_anti_aircraft_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_group_data (mode);


	update_gunships_screen_progress_indicator ();


	unpack_local_division_database (mode);

	unpack_local_message_log (mode);

	unpack_group_callsign_database (mode);


	update_gunships_screen_progress_indicator ();


	#if LANDING_ROUTE_CHECK

	unpack_debug_entity_landing_locks (mode);

	#endif

	unpack_buffer_overflow = get_unpack_buffer_overflow ();

	close_unpack_buffer ();



	process_entity_list_link_validation ();

	process_entity_list_root_validation ();

	#if 1
	repair_landing_entity_locks (mode);
	#endif

	process_entity_smoke_list_creation ();

	process_entity_sound_effect_creation ();


	initialise_entity_list_validation ();


	//
	// Any processes which can only be completed when ALL entities have been unpacked
	//

	validate_local_group_data ();









	set_comms_data_flow (COMMS_DATA_FLOW_TX);

	#if !DEBUG_MODULE_PACK_ALL

	enable_debug_log_entity_args ();

	#endif

	return (unpack_buffer_overflow);
}





void initialise_entity_list_validation (void)
{

	comms_entity_smoke_list_creation_type
		*destroy_smoke_list;

	comms_entity_sound_effect_creation_type
		*destroy_sound_effect;

	comms_entity_list_validation_type
		*destroy_entity_list;

	//
	// list links
	//
	
	while (entity_list_link_validation)
	{

		destroy_entity_list = entity_list_link_validation;

		entity_list_link_validation = entity_list_link_validation->next;

		free_mem (destroy_entity_list);
	}

	entity_list_link_validation = NULL;

	//
	// list roots
	//
	
	while (entity_list_root_validation)
	{

		destroy_entity_list = entity_list_root_validation;

		entity_list_root_validation = entity_list_root_validation->next;

		free_mem (destroy_entity_list);
	}

	entity_list_root_validation = NULL;

	//
	// smoke lists
	//
	
	while (entity_smoke_list_creation)
	{

		destroy_smoke_list = entity_smoke_list_creation;

		entity_smoke_list_creation = entity_smoke_list_creation->next;

		free_mem (destroy_smoke_list);
	}

	entity_smoke_list_creation = NULL;

	//
	// sound effects
	//
	
	while (entity_sound_effect_creation)
	{

		destroy_sound_effect = entity_sound_effect_creation;

		entity_sound_effect_creation = entity_sound_effect_creation->next;

		free_mem (destroy_sound_effect);
	}

	entity_sound_effect_creation = NULL;
}





int register_entity_list_link_for_post_unpack_validation (struct ENTITY *en, int list_type)
{

	comms_entity_list_validation_type
		*new_entity_list;

	new_entity_list = (comms_entity_list_validation_type *) malloc_heap_mem (sizeof (comms_entity_list_validation_type));

	new_entity_list->en = en;

	new_entity_list->list_type = (list_types) list_type;

	new_entity_list->next = entity_list_link_validation;

	entity_list_link_validation = new_entity_list;














	return TRUE;
}





int register_entity_list_root_for_post_unpack_validation (struct ENTITY *en, int list_type)
{

	comms_entity_list_validation_type
		*new_entity_list;

	new_entity_list = (comms_entity_list_validation_type *) malloc_heap_mem (sizeof (comms_entity_list_validation_type));

	new_entity_list->en = en;

	new_entity_list->list_type = (list_types) list_type;

	new_entity_list->next = entity_list_root_validation;

	entity_list_root_validation = new_entity_list;













	return TRUE;
}





//
// Nasty function to restitch the entity lists after unpacking.
// 	problem occurs because some entities are created locally (parent)
//		and some (which are linked into them as children) are created client/server
//		When the children are unpacked the parent has not been created so their
//		links are setup properly but the parent never knows to set its first_child
//		pointer to them. Enter the re-stitch code king...
//

void process_entity_list_link_validation (void)
{

	entity
		*en,
		*succ,
		*pred,
		*parent,
		*last_en;

	comms_entity_list_validation_type
		*this_entity_list;

	dump_entities ();

	while (entity_list_link_validation)
	{

		this_entity_list = entity_list_link_validation;

		entity_list_link_validation = entity_list_link_validation->next;

		en = this_entity_list->en;

		last_en = en;

		while (en)
		{

			parent = get_local_entity_parent (en, this_entity_list->list_type);


			// if NULL its been removed from list, skip to next
			if (!parent)
			{

				en = get_local_entity_child_pred (en, this_entity_list->list_type);

				continue;
			}


			ASSERT (parent->type != ENTITY_TYPE_UNKNOWN);


			// check valid list type for parent
			get_local_entity_first_child (parent, this_entity_list->list_type);



			// check succ
			succ = get_local_entity_child_succ (en, this_entity_list->list_type);

			if (succ)
			{

				ASSERT (succ->type != ENTITY_TYPE_UNKNOWN);

				if (get_local_entity_child_pred (succ, this_entity_list->list_type) != en)
				{

					ASSERT (get_local_entity_child_pred (succ, this_entity_list->list_type) == NULL);







					set_local_entity_child_pred (succ, this_entity_list->list_type, en);

					if (get_local_entity_parent (succ, this_entity_list->list_type) == NULL)
					{

						set_local_entity_parent (succ, this_entity_list->list_type, parent);






					}
				}
			}



			// check pred
			pred = get_local_entity_child_pred (en, this_entity_list->list_type);

			if (pred)
			{

				ASSERT (pred->type != ENTITY_TYPE_UNKNOWN);

				if (get_local_entity_child_succ (pred, this_entity_list->list_type) != en)
				{

					ASSERT (get_local_entity_child_succ (pred, this_entity_list->list_type) == NULL);

					set_local_entity_child_succ (pred, this_entity_list->list_type, en);







					if (get_local_entity_parent (pred, this_entity_list->list_type) == NULL)
					{

						set_local_entity_parent (pred, this_entity_list->list_type, parent);






					}
				}
			}


			last_en = en;

			en = get_local_entity_child_pred (en, this_entity_list->list_type);
		}


		// check first child links to parent
		parent = get_local_entity_parent (last_en, this_entity_list->list_type);

		if (get_local_entity_first_child (parent, this_entity_list->list_type) != last_en)
		{

			//ASSERT (get_local_entity_first_child (parent, this_entity_list->list_type) == NULL);

			set_local_entity_first_child (parent, this_entity_list->list_type, last_en);






		}


		free_mem (this_entity_list);
	}
}





void process_entity_list_root_validation (void)
{

	entity
		*en,
		*first_child;

	comms_entity_list_validation_type
		*this_entity_list;

	while (entity_list_root_validation)
	{

		this_entity_list = entity_list_root_validation;

		entity_list_root_validation = entity_list_root_validation->next;

		en = this_entity_list->en;

		first_child = get_local_entity_first_child (en, this_entity_list->list_type);


		// if NULL its been removed from list, skip to next
		if (!first_child)
		{







			continue;
		}


		ASSERT (first_child->type != ENTITY_TYPE_UNKNOWN);


		// check valid list type for first_child
		get_local_entity_parent (first_child, this_entity_list->list_type);


		if (get_local_entity_parent (first_child, this_entity_list->list_type) != en)
		{

			set_local_entity_parent (first_child, this_entity_list->list_type, en);






		}
		else
		{






		}


		free_mem (this_entity_list);
	}
}




#if 1
void repair_landing_entity_locks (pack_modes mode)
{

	entity
		*en,
		*guide,
		*group,
		*keysite,
		*landing,
		*task;

	int
		landing_flag;

	if (mode != PACK_MODE_SERVER_SESSION)
	{
		return;
	}

	en = get_local_entity_list ();

	while (en)
	{

		landing_flag = FALSE;

		// debug
		if (get_local_entity_type (en) != ENTITY_TYPE_GROUP)
		// debug
		{
	
			if ((get_local_entity_int_value (en, INT_TYPE_IDENTIFY_AIRCRAFT)) || (get_local_entity_int_value (en, INT_TYPE_IDENTIFY_VEHICLE)))
			{
	
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
	
								//
								// LANDING
								//
						
								landing = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);
		
								ASSERT (landing);
		










		
								notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_ROUTE, landing, en);
		
								landing_flag = TRUE;
	
								break;
							}
	
							case ENTITY_SUB_TYPE_TASK_LANDING_HOLDING:
							{
	
								//
								// LANDING HOLDING
								//
						
								landing_flag = TRUE;
	
								break;
							}
	
							case ENTITY_SUB_TYPE_TASK_TAKEOFF:
							{
			
								landing = get_local_entity_parent (task, LIST_TYPE_ASSIGNED_TASK);
		
								keysite = get_local_entity_parent (landing, LIST_TYPE_LANDING_SITE);
	










			
								notify_local_entity (ENTITY_MESSAGE_LOCK_TAKEOFF_ROUTE, landing, en);
	
								break;
							}
						}
					}
				}
	
				//
				// LANDED
				//
	
				if ((get_local_entity_int_value (en, INT_TYPE_LANDED)) || (landing_flag))
				{
	
					group = get_local_entity_parent (en, LIST_TYPE_MEMBER);
	
					keysite = get_local_entity_parent (group, LIST_TYPE_KEYSITE_GROUP);

					if (get_local_entity_type (keysite) != ENTITY_TYPE_KEYSITE)
					{
			
						ASSERT (get_local_entity_type (en) == ENTITY_TYPE_SHIP_VEHICLE);
			
						ASSERT (get_local_entity_type (keysite) == ENTITY_TYPE_FORCE);
			






					}
					else
					{
		
						landing = get_local_entity_landing_entity (keysite, group_database [get_local_entity_int_value (group, INT_TYPE_ENTITY_SUB_TYPE)].default_landing_type);
		










			
						notify_local_entity (ENTITY_MESSAGE_LOCK_LANDING_SITE, landing, en);
					}
				}
			}
		}

		en = get_local_entity_succ (en);
	}
}
#endif




/* FIXME: 2nd arg should be of type meta_smoke_list_types, but its declaration is not available yet, which breaks in gcc (yem) */
void register_attach_meta_smoke_list_to_object (entity *en, int type, object_3d_sub_object_index_numbers attachment_point, int *entity_index_list, int count)
{

	comms_entity_smoke_list_creation_type
		*new_smoke_list_creation;

	new_smoke_list_creation = (comms_entity_smoke_list_creation_type *) malloc_heap_mem (sizeof (comms_entity_smoke_list_creation_type));

	new_smoke_list_creation->en = en;

	new_smoke_list_creation->meta_smoke_type = (meta_smoke_list_types) type;

	new_smoke_list_creation->attachment_point = attachment_point;

	new_smoke_list_creation->count = count;

	new_smoke_list_creation->entity_index_list = (int *) malloc_heap_mem (sizeof (int) * count);

	memcpy (new_smoke_list_creation->entity_index_list, entity_index_list, sizeof (int) * count);

	new_smoke_list_creation->next = entity_smoke_list_creation;

	entity_smoke_list_creation = new_smoke_list_creation;
}





void register_attach_sound_effect_to_object (entity *en, int index, entity_sides side, entity_sub_types sub_type, int valid, int sample_index)
{

	comms_entity_sound_effect_creation_type
		*new_sound_effect_creation;

	new_sound_effect_creation = (comms_entity_sound_effect_creation_type *) malloc_heap_mem (sizeof (comms_entity_sound_effect_creation_type));

	new_sound_effect_creation->en = en;

	new_sound_effect_creation->side = side;

	new_sound_effect_creation->sub_type = sub_type;

	new_sound_effect_creation->entity_index = index;

	new_sound_effect_creation->sample_index = sample_index;

	new_sound_effect_creation->valid = valid;

	new_sound_effect_creation->next = entity_sound_effect_creation;

	entity_sound_effect_creation = new_sound_effect_creation;
}





void process_entity_smoke_list_creation (void)
{
	
	comms_entity_smoke_list_creation_type
		*this_smoke_list_creation;

	while (entity_smoke_list_creation)
	{

		this_smoke_list_creation = entity_smoke_list_creation;

		ASSERT (this_smoke_list_creation->en);

		attach_local_meta_smoke_lists_to_object (this_smoke_list_creation->en,
																this_smoke_list_creation->meta_smoke_type,
																this_smoke_list_creation->attachment_point,
																this_smoke_list_creation->entity_index_list);

		if (get_local_entity_int_value (this_smoke_list_creation->en, INT_TYPE_MOBILE_MOVING))
		{

			set_infinite_smoke_list_generating_state (this_smoke_list_creation->en, ENTITY_SUB_TYPE_EFFECT_SMOKE_LIST_VEHICLE_WAKE, INFINITE_SMOKE_ON);
		}
		else
		{
			set_infinite_smoke_list_generating_state (this_smoke_list_creation->en, ENTITY_SUB_TYPE_EFFECT_SMOKE_LIST_VEHICLE_WAKE, INFINITE_SMOKE_OFF);
		}

		// JB 030311 From Gotcha (Victor Jetten)
		// Bug makes it impossible to load saved campaigns with builds made with VC/gcc or anything that is not watcom.
		// Swap the followting two lines to fix exception

		entity_smoke_list_creation = entity_smoke_list_creation->next;

		free_mem (this_smoke_list_creation);
	}
}





void process_entity_sound_effect_creation (void)
{
	
	comms_entity_sound_effect_creation_type
		*this_sound_effect_creation;

	int
		sample_index;

	while (entity_sound_effect_creation)
	{

		this_sound_effect_creation = entity_sound_effect_creation;

		ASSERT (this_sound_effect_creation->en);

		//
		// create sound effect
		//
		
		sample_index = this_sound_effect_creation->sample_index;

		create_local_sound_effect_entity
		(
			this_sound_effect_creation->entity_index,
			this_sound_effect_creation->en,			// parent
			(entity_sides) this_sound_effect_creation->side,
			this_sound_effect_creation->sub_type,
			SOUND_CHANNEL_SOUND_EFFECT,				// only sound effects are done this way
			SOUND_LOCALITY_ALL,
			NULL,												// position
			1.0,												// amplification
			1.0, //Werewolf pitch
			this_sound_effect_creation->valid,		// valid sound effect
			TRUE,												// looping
			1,													// sample count
			&sample_index									// sample index list
		);

		// JB 030311 From Gotcha (Victor Jetten)
		// Bug makes it impossible to load saved campaigns with builds made with VC/gcc or anything that is not watcom.
		// Swap the followting two lines to fix exception

		entity_sound_effect_creation = entity_sound_effect_creation->next;

		free_mem (this_sound_effect_creation);
	}
}





void dump_entities (void)
{
/*
	int
		loop;

	for (loop = 0; loop < 60000; loop ++)
	{

		if (get_local_entity_type (&entities [loop]) != ENTITY_TYPE_UNKNOWN)
		{

			debug_log ("EN_SESSN: entity %d = %s sub_type = %d", loop, get_local_entity_type_name (&entities [loop]), get_local_entity_int_value (&entities [loop], INT_TYPE_ENTITY_SUB_TYPE));
		}
		else
		{

			debug_log ("EN_SESSN: entity %d = %s", loop, get_local_entity_type_name (&entities [loop]));
		}
	}
	*/
}





void kill_entities_awaiting_update (void)
{

	entity
		*destroy_en,
		*en;

	en = get_local_entity_list ();

	while (en)
	{

		destroy_en = en;

		en = get_local_entity_succ (en);

		if (get_local_entity_int_value (destroy_en, INT_TYPE_KILL_NEXT_UPDATE))
		{

			destroy_client_server_entity_family (destroy_en);
		}
	}
}




