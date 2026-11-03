

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
}





void overload_regen_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_REGEN]		= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_REGEN]	= unpack_local_data;
}





void pack_local_regen_data (pack_modes mode)
{
   entity
      *en;

 	regen
      *raw;

	int
		i,
		j,
		k;
		
   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode != PACK_MODE_SERVER_SESSION)
   {
      return;
   }

	en = get_local_entity_list ();

	while (en)
	{
		if (get_local_entity_type (en) == ENTITY_TYPE_REGEN)
		{
			//
			// pack regen data
			//

			pack_entity_safe_ptr (en);

			raw = (regen *) get_local_entity_data (en);

			pack_float_value (en, FLOAT_TYPE_SLEEP, raw->sleep);

			pack_int_value (en, INT_TYPE_ALIVE, raw->alive);

			pack_int_value (en, INT_TYPE_SIDE, raw->side);
		}

		en = get_local_entity_succ (en);
	}

	for (i = 0; i < NUM_ENTITY_SIDES; i++)
	{
		for (j = 0; j < NUM_ENTITY_SUB_TYPE_REGENS; j++)
		{
			pack_int_value (en, INT_TYPE_VALUE, regen_manager[i][j].size);
			pack_int_value (en, INT_TYPE_VALUE, regen_manager[i][j].count);
			pack_int_value (en, INT_TYPE_VALUE, regen_manager[i][j].front);

			for (k = 0; k < regen_manager[i][j].size; k++)
			{
				pack_int_value (en, INT_TYPE_VALUE, regen_queue [i][j][k].type);
				pack_int_value (en, INT_TYPE_VALUE, regen_queue [i][j][k].sub_type);
				pack_int_value (en, INT_TYPE_VALUE, regen_queue [i][j][k].group);
			}
		}
	}

}





void unpack_local_regen_data (pack_modes mode)
{
   entity
      *en;

   regen
      *raw;

	int
		i,
		j,
		k,
		index;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode != PACK_MODE_SERVER_SESSION)
   {
      return;
   }

	en = get_local_entity_list ();

	while (en)
	{
		if (get_local_entity_type (en) == ENTITY_TYPE_REGEN)
		{
			index = unpack_entity_safe_index ();

			raw = (regen *) get_local_entity_data (en);

			raw->sleep = unpack_float_value (en, FLOAT_TYPE_SLEEP);

			raw->alive = unpack_int_value (en, INT_TYPE_ALIVE);

			raw->side = unpack_int_value (en, INT_TYPE_SIDE);
		}

		en = get_local_entity_succ (en);
	}

	for (i = 0; i < NUM_ENTITY_SIDES; i++)
	{
		for (j = 0; j < NUM_ENTITY_SUB_TYPE_REGENS; j++)
		{
			regen_manager [i][j].size = unpack_int_value (en, INT_TYPE_VALUE);
			regen_manager [i][j].count = unpack_int_value (en, INT_TYPE_VALUE);
			regen_manager [i][j].front = unpack_int_value (en, INT_TYPE_VALUE);

			regen_queue [i][j] = (regen_list_element *) safe_malloc (sizeof (regen_list_element) * regen_manager [i][j].size);

			for (k = 0; k < regen_manager [i][j].size; k++)
			{
				regen_queue [i][j][k].type = unpack_int_value (en, INT_TYPE_VALUE);
				regen_queue [i][j][k].sub_type = unpack_int_value (en, INT_TYPE_VALUE);
				regen_queue [i][j][k].group = unpack_int_value (en, INT_TYPE_VALUE);
			}
		}
	}
}




