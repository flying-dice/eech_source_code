

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
}





void overload_update_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_UPDATE] 	= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_UPDATE]	= unpack_local_data;
}




