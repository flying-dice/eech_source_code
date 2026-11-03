

































































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




