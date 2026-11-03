

































































#include "project.h"





#define DEBUG_MODULE 0





static void pack_local_data (entity *en, pack_modes mode)
{
}





static void unpack_local_data (entity *en, entity_types type, pack_modes mode)
{
}





void overload_segment_pack_functions (void)
{
	fn_pack_local_entity_data[ENTITY_TYPE_SEGMENT]	= pack_local_data;

	fn_unpack_local_entity_data[ENTITY_TYPE_SEGMENT]	= unpack_local_data;
}





void pack_local_segment_data (pack_modes mode)
{
   entity
      *en;

   segment
      *raw;

	#if DEBUG_MODULE

	static int
		count = 0;

	#endif

	int
		segment_alive,
		segment_count;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode == PACK_MODE_BROWSE_SESSION)
   {
      return;
   }

	//
	// RLE segments, only alive ones.
	//

	segment_count = 0;

	segment_alive = TRUE;

	debug_log ("RLE SEGMENT");

	en = get_local_entity_list ();

	while (en)
	{

		if (get_local_entity_type (en) == ENTITY_TYPE_SEGMENT)
		{

			#if DEBUG_MODULE

			count ++;

			#endif

			raw = (segment *) get_local_entity_data (en);

			if (segment_alive == raw->fix.alive)
			{

				segment_count ++;
			}
			else
			{

				pack_int_value (NULL, INT_TYPE_LENGTH, segment_count);

				pack_int_value (NULL, INT_TYPE_ALIVE, segment_alive);

				segment_alive = raw->fix.alive;

				segment_count = 1;
			}
		}
	
		en = get_local_entity_succ (en);
	}

	pack_int_value (NULL, INT_TYPE_LENGTH, segment_count);

	pack_int_value (NULL, INT_TYPE_ALIVE, segment_alive);

	#if DEBUG_MODULE

	debug_log ("SG_PACK: packing segment data (count = %d)", count);

	#endif
}





void unpack_local_segment_data (pack_modes mode)
{
   entity
      *en,
		*pred,
		*succ;

   segment
      *raw;

	#if DEBUG_MODULE

	static int
		count = 0;

	#endif

	int
		segment_alive,
		segment_count;

   ASSERT (mode != PACK_MODE_UPDATE_ENTITY);

   if (mode == PACK_MODE_BROWSE_SESSION)
   {
      return;
   }

	//
	// unpack alive status for each segment
	//

	debug_log ("RLE SEGMENT");

	segment_count = 0;

	segment_count = unpack_int_value (NULL, INT_TYPE_LENGTH);

	segment_alive = unpack_int_value (NULL, INT_TYPE_ALIVE);

	en = get_local_entity_list ();

	while (en)
	{

		if (en->type == ENTITY_TYPE_SEGMENT)
		{

			#if DEBUG_MODULE

			count ++;

			#endif

			if (segment_count == 0)
			{

				segment_count = unpack_int_value (NULL, INT_TYPE_LENGTH);

				segment_alive = unpack_int_value (NULL, INT_TYPE_ALIVE);
			}

			segment_count --;

			raw = (segment *) get_local_entity_data (en);

			raw->fix.alive = segment_alive;

			if ((!raw->fix.alive) && (raw->fix.sub_type == ENTITY_SUB_TYPE_FIXED_BRIDGE_UNSUPPORTED_MID_SECTION))
			{
				//
				// position the segment on the ground
				//

				vec3d
					*pos;

				terrain_3d_point_data
					terrain_info;

				pos = get_local_entity_vec3d_ptr (en, VEC3D_TYPE_POSITION);

				memset (&terrain_info, 0, sizeof (terrain_3d_point_data));

				ASSERT(point_inside_map_area(pos));

				pos->y = get_3d_terrain_point_data (pos->x, pos->z, &terrain_info);

				if (get_terrain_type_class (terrain_info.terrain_type) == TERRAIN_CLASS_WATER)
				{
					pos->y -= 1.0;
				}
			}
		}

		en = get_local_entity_succ (en);
	}

	#if DEBUG_MODULE

	debug_log ("SG_PACK: unpacking segment data (count = %d)", count);

	#endif

	//
	// determine 3d object for each segment ( some might be damaged / destroyed )
	//

	en = get_local_entity_list ();

	while (en)
	{

		if (en->type == ENTITY_TYPE_SEGMENT)
		{

			raw = (segment *) get_local_entity_data (en);

			pred = get_local_entity_child_pred (en, LIST_TYPE_SEGMENT);

			succ = get_local_entity_child_succ (en, LIST_TYPE_SEGMENT);

			raw->fix.object_3d_shape = get_bridge_segment_damaged_object (en, pred, succ);
		}

		en = get_local_entity_succ (en);
	}
}




