

































































#include "project.h"





#define DEBUG_MODULE 0

#define DEBUG_UPDATE_LIST_VALIDATION 0





void (*fn_destroy_local_entity[NUM_ENTITY_TYPES]) (entity *en);

void (*fn_destroy_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);

void (*fn_destroy_local_entity_family[NUM_ENTITY_TYPES]) (entity *en);

void (*fn_destroy_client_server_entity_family[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);

void (*fn_kill_local_entity[NUM_ENTITY_TYPES]) (entity *en);

void (*fn_kill_client_server_entity[NUM_ENTITY_TYPES][NUM_COMMS_MODEL_TYPES]) (entity *en);





static void default_destroy_entity (entity *en)
{
//Werewolf 14 May 2003
//	debug_fatal ("default_destroy_entity invoked: %s (index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));
}





static void default_destroy_entity_family (entity *en)
{
////Moje 030929 According to an e-mail Werewolf should have done this:
//   debug_fatal ("default_destroy_entity_family invoked: %s (index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));
}





static void default_kill_entity (entity *en)
{
     debug_fatal ("default_kill_entity invoked: %s (index = %d)", get_local_entity_type_name (en), get_local_entity_index (en));
}





void initialise_entity_destroy_default_functions (void)
{
   int
      i;

   for (i = 0; i < NUM_ENTITY_TYPES; i++)
   {
      fn_destroy_local_entity[i]													= default_destroy_entity;

      fn_destroy_client_server_entity[i][COMMS_MODEL_SERVER]			= default_destroy_entity;

      fn_destroy_client_server_entity[i][COMMS_MODEL_CLIENT]			= default_destroy_entity;

      fn_destroy_local_entity_family[i]										= default_destroy_entity_family;

      fn_destroy_client_server_entity_family[i][COMMS_MODEL_SERVER]	= default_destroy_entity_family;

      fn_destroy_client_server_entity_family[i][COMMS_MODEL_CLIENT]	= default_destroy_entity_family;

      fn_kill_local_entity[i]														= default_kill_entity;

      fn_kill_client_server_entity[i][COMMS_MODEL_SERVER]				= default_kill_entity;

      fn_kill_client_server_entity[i][COMMS_MODEL_CLIENT]				= default_kill_entity;
   }
}





void destroy_local_entity (entity *en)
{
   update_destroy_entity_statistics (en);

	#if DEBUG_MODULE
	{

		static int
			reported_segment = FALSE,
			reported_city_building = FALSE,
			reported_sector = FALSE;

		#if DEBUG_UPDATE_LIST_VALIDATION

		entity
			*succ,
			*pred,
			*updt;

		if (get_update_entity ())
		{
		
			updt = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);
	
			while (updt)
			{
	
				ASSERT (updt->type != ENTITY_TYPE_UNKNOWN);

				succ = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);

				if (succ)
				{

					ASSERT (get_local_entity_child_pred (succ, LIST_TYPE_UPDATE) == updt);
				}

				pred = get_local_entity_child_pred (updt, LIST_TYPE_UPDATE);

				if (pred)
				{

					ASSERT (get_local_entity_child_succ (pred, LIST_TYPE_UPDATE) == updt);
				}
	
				updt = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);
			}
		}

		#endif
	
		switch (get_local_entity_type (en))
		{
	
			case ENTITY_TYPE_SEGMENT:
			case ENTITY_TYPE_SECTOR:
			case ENTITY_TYPE_CITY_BUILDING:
			{
	
				if (get_local_entity_type (en) == ENTITY_TYPE_SEGMENT)
				{
		
					if (reported_segment)
					{
		
						break;
					}
		
					reported_segment = TRUE;
				}
	
				if (get_local_entity_type (en) == ENTITY_TYPE_SECTOR)
				{
		
					if (reported_sector)
					{
		
						break;
					}
		
					reported_sector = TRUE;
				}
	
				if (get_local_entity_type (en) == ENTITY_TYPE_CITY_BUILDING)
				{
		
					if (reported_city_building)
					{
		
						break;
					}
		
					reported_city_building = TRUE;
				}
			}
			default:
			{
	
				debug_log ("EN_DSTRY: destroyed %s, index %d, sub_type %d", get_local_entity_type_name (en), get_local_entity_index (en), get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE));
	
				break;
			} 
		}
	}
	#endif

   fn_destroy_local_entity[get_local_entity_type(en)] (en);
}





void destroy_local_entities (void)
{
	entity
		*en;

	en = get_local_entity_list ();

	if (get_gunship_entity())
		set_gunship_entity(NULL);

	while (en)
   {

		#if DEBUG_MODULE
		{

			static int
				reported_segment = FALSE,
				reported_city_building = FALSE,
				reported_sector = FALSE;
	
			#if DEBUG_UPDATE_LIST_VALIDATION

			entity
				*succ,
				*pred,
				*updt;
	
			if (get_update_entity ())
			{
			
				updt = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);
		
				while (updt)
				{
		
					ASSERT (updt->type != ENTITY_TYPE_UNKNOWN);
	
					succ = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);
	
					if (succ)
					{
	
						ASSERT (get_local_entity_child_pred (succ, LIST_TYPE_UPDATE) == updt);
					}
	
					pred = get_local_entity_child_pred (updt, LIST_TYPE_UPDATE);
	
					if (pred)
					{
	
						ASSERT (get_local_entity_child_succ (pred, LIST_TYPE_UPDATE) == updt);
					}
		
					updt = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);
				}
			}

			#endif

			switch (get_local_entity_type (en))
			{
		
				case ENTITY_TYPE_SEGMENT:
				case ENTITY_TYPE_SECTOR:
				case ENTITY_TYPE_CITY_BUILDING:
				{
		
					if (get_local_entity_type (en) == ENTITY_TYPE_SEGMENT)
					{
			
						if (reported_segment)
						{
			
							break;
						}
			
						reported_segment = TRUE;
					}
		
					if (get_local_entity_type (en) == ENTITY_TYPE_SECTOR)
					{
			
						if (reported_sector)
						{
			
							break;
						}
			
						reported_sector = TRUE;
					}
		
					if (get_local_entity_type (en) == ENTITY_TYPE_CITY_BUILDING)
					{
			
						if (reported_city_building)
						{
			
							break;
						}
			
						reported_city_building = TRUE;
					}
				}
				default:
				{
		
					debug_log ("EN_DSTRY: destroying %s, index %d, sub_type %d", get_local_entity_type_name (en), get_local_entity_index (en), get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE));
		
					break;
				} 
			}
		}
		#endif

		destroy_local_entity_family(en);

		en = get_local_entity_list ();
   }
}





void destroy_local_entity_children (entity *en, list_types list)
{
   entity
      *child;

   child = get_local_entity_first_child (en, list);

   while (child)
   {

		#if DEBUG_MODULE
		{

			static int
				reported_segment = FALSE,
				reported_city_building = FALSE,
				reported_sector = FALSE;
	
			#if DEBUG_UPDATE_LIST_VALIDATION

			entity
				*succ,
				*pred,
				*updt;
		
			if (get_update_entity ())
			{
			
				updt = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);
		
				while (updt)
				{
		
					ASSERT (updt->type != ENTITY_TYPE_UNKNOWN);
	
					succ = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);
	
					if (succ)
					{
	
						ASSERT (get_local_entity_child_pred (succ, LIST_TYPE_UPDATE) == updt);
					}
	
					pred = get_local_entity_child_pred (updt, LIST_TYPE_UPDATE);
	
					if (pred)
					{
	
						ASSERT (get_local_entity_child_succ (pred, LIST_TYPE_UPDATE) == updt);
					}
		
					updt = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);
				}
			}

			#endif
			
			switch (get_local_entity_type (en))
			{
		
				case ENTITY_TYPE_SEGMENT:
				case ENTITY_TYPE_SECTOR:
				case ENTITY_TYPE_CITY_BUILDING:
				{
		
					if (get_local_entity_type (en) == ENTITY_TYPE_SEGMENT)
					{
			
						if (reported_segment)
						{
			
							break;
						}
			
						reported_segment = TRUE;
					}
		
					if (get_local_entity_type (en) == ENTITY_TYPE_SECTOR)
					{
			
						if (reported_sector)
						{
			
							break;
						}
			
						reported_sector = TRUE;
					}
		
					if (get_local_entity_type (en) == ENTITY_TYPE_CITY_BUILDING)
					{
			
						if (reported_city_building)
						{
			
							break;
						}
			
						reported_city_building = TRUE;
					}
				}
				default:
				{
	
					debug_log ("EN_DSTRY: destroying %s, index %d, sub_type %d", get_local_entity_type_name (child), get_local_entity_index (child), get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE));
		
					break;
				} 
			}
		}
		#endif

      destroy_local_entity_family (child);

      child = get_local_entity_first_child (en, list);
   }
}





void destroy_client_server_entity_children (entity *en, list_types list)
{
   entity
      *child;

   child = get_local_entity_first_child (en, list);

   while (child)
   {

		#if DEBUG_MODULE
		{

			static int
				reported_segment = FALSE,
				reported_city_building = FALSE,
				reported_sector = FALSE;
	
			#if DEBUG_UPDATE_LIST_VALIDATION

			entity
				*succ,
				*pred,
				*updt;
		
			if (get_update_entity ())
			{
			
				updt = get_local_entity_first_child (get_update_entity (), LIST_TYPE_UPDATE);
		
				while (updt)
				{
		
					ASSERT (updt->type != ENTITY_TYPE_UNKNOWN);

				succ = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);

				if (succ)
				{

					ASSERT (get_local_entity_child_pred (succ, LIST_TYPE_UPDATE) == updt);
				}

				pred = get_local_entity_child_pred (updt, LIST_TYPE_UPDATE);

				if (pred)
				{

					ASSERT (get_local_entity_child_succ (pred, LIST_TYPE_UPDATE) == updt);
				}
	
				updt = get_local_entity_child_succ (updt, LIST_TYPE_UPDATE);
				}
			}

			#endif

			switch (get_local_entity_type (en))
			{
		
				case ENTITY_TYPE_SEGMENT:
				case ENTITY_TYPE_SECTOR:
				case ENTITY_TYPE_CITY_BUILDING:
				{
		
					if (get_local_entity_type (en) == ENTITY_TYPE_SEGMENT)
					{
			
						if (reported_segment)
						{
			
							break;
						}
			
						reported_segment = TRUE;
					}
		
					if (get_local_entity_type (en) == ENTITY_TYPE_SECTOR)
					{
			
						if (reported_sector)
						{
			
							break;
						}
			
						reported_sector = TRUE;
					}
		
					if (get_local_entity_type (en) == ENTITY_TYPE_CITY_BUILDING)
					{
			
						if (reported_city_building)
						{
			
							break;
						}
			
						reported_city_building = TRUE;
					}
				}
				default:
				{
	
					debug_log ("EN_DSTRY: destroying %s, index %d, sub_type %d", get_local_entity_type_name (child), get_local_entity_index (child), get_local_entity_int_value (en, INT_TYPE_ENTITY_SUB_TYPE));
		
					break;
				} 
			}
		}
		#endif

      destroy_client_server_entity_family (child);

      child = get_local_entity_first_child (en, list);
   }
}





void destroy_local_only_entities (void)
{

	destroy_local_sector_entities ();

	//
	// These functions just assert that the entities have been destroyed by the destroy_local_entity_family system
	//

	destroy_local_pylon_entities ();

	destroy_local_update_entity ();

	destroy_local_camera_entity ();
}




