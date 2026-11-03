



























































#include "project.h"





#define DEBUG_MODULE 0





#include "ai/ai_misc/ai_route.h"





char
	valid_warzone_bridges [NUM_ENTITY_SUB_TYPE_BRIDGES];





static float get_actual_bridge_length (vec3d *end1, vec3d *end2, entity_sub_types type, vec3d *actual_start, vec3d *actual_end, int *return_num_mid_sections);





void initialise_valid_warzone_bridge_database (void)
{

	int
		result;

	char
		temp_filename [128];

	session_list_data_type
		*current_session;

	FILE
		*file_ptr;

	file_tags
		tag;

	memset (valid_warzone_bridges, 0, sizeof (valid_warzone_bridges));

	//
	// read in valid_bridge_warzone file
	//

	ASSERT (get_valid_current_game_session ());

	current_session = get_current_game_session ();

	sprintf (temp_filename, "%s\\%s", current_session->data_path, VALID_BRIDGE_WARZONE_FILE);

	file_ptr = safe_fopen (temp_filename, "r");

	if (file_ptr)
	{

		while (TRUE)
		{

			tag = (file_tags) get_next_file_tag (file_ptr, application_tag_strings, FILE_TAG_APPLICATION_LAST_TAG);

			switch (tag)
			{

				case FILE_TAG_START:
				{

					break;
				}

				case FILE_TAG_BRIDGE:
				{

					entity_sub_types
						bridge_type;

					while (check_next_file_tag (file_ptr, application_tag_strings, FILE_TAG_APPLICATION_LAST_TAG) != FILE_TAG_END)
					{

						result = get_next_file_tag (file_ptr, application_tag_strings, FILE_TAG_APPLICATION_LAST_TAG);

						ASSERT (result == FILE_TAG_TYPE);

						bridge_type = get_next_file_enum (file_ptr, entity_sub_type_bridge_names, NUM_ENTITY_SUB_TYPE_BRIDGES);

						valid_warzone_bridges [bridge_type] = TRUE;
					}

					break;
				}

				case FILE_TAG_END:
				{

					fclose (file_ptr);

					return;
				}
			}
		}
	}
}





entity *create_bridge( vec3d *end1, vec3d *end2, int start_node, int end_node, entity_sub_types type, float max_leg_height )
{

	const bridge_data
		*bridge_info;

	entity
		*segment_entity,
		*bridge_entity;

	float
		pitch,
		heading,
		start_length,
		actual_length;

	int
		element_loop,
		section_loop,
		num_mid_sections;

	vec3d
		start,
		end,
		pos,
		raw_vector;

	bridge_segment_types
		segment_type;

	ASSERT( ( type >= 0 ) && ( type < NUM_ENTITY_SUB_TYPE_BRIDGES ) );

	bridge_info = &bridge_database[ type ];







	//
	// get adjusted start and end points
	//

	actual_length = get_actual_bridge_length (end1, end2, type, &start, &end, &num_mid_sections);

	raw_vector.x = end2->x - end1->x;
	raw_vector.y = end2->y - end1->y;
	raw_vector.z = end2->z - end1->z;

	normalise_3d_vector( &raw_vector );

	//
	// calculate heading and pitch for the segments
	//

	get_heading_and_pitch_from_3d_unit_vector( &raw_vector, &heading, &pitch );

	//
	// create bridge entity
	//

	bridge_entity = create_local_entity
	(
		ENTITY_TYPE_BRIDGE,
		ENTITY_INDEX_DONT_CARE,
		ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, type),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_START_NODE, start_node),
		ENTITY_ATTR_INT_VALUE (INT_TYPE_END_NODE, end_node),
		ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_PITCH, pitch),
		ENTITY_ATTR_END
	);

	//
	// create segments
	//

	//
	// start segment
	//

	if ( bridge_info->start_section_type != BRIDGE_SEGMENT_TYPE_NONE )
	{
		segment_entity = create_local_entity
		(
			ENTITY_TYPE_SEGMENT,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_BRIDGE_SEGMENT_TYPE, bridge_info->start_section_type),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_OBJECT_3D_SHAPE, bridge_segment_normal_object( bridge_info->start_section_type )),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_DAMAGE_LEVEL, bridge_segment_initial_damage_level( bridge_info->start_section_type )),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, start.x, start.y, start.z),
			ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_HEADING, heading),
			ENTITY_ATTR_PARENT (LIST_TYPE_SEGMENT, bridge_entity),
			ENTITY_ATTR_END
		);
	}

	//
	// mid segment(s)
	//

	start_length = bridge_segment_length( bridge_info->start_section_type );

	pos.x = start.x + ( raw_vector.x * start_length * 0.5 );
	pos.y = start.y + ( raw_vector.y * start_length * 0.5 );
	pos.z = start.z + ( raw_vector.z * start_length * 0.5 );

	for ( section_loop = 0 ; section_loop < num_mid_sections ; section_loop ++ )
	{

		for ( element_loop = 0 ; element_loop < bridge_info->num_elements_in_mid_section_pattern ; element_loop ++ )
		{

			segment_type = bridge_info->mid_section_type[ element_loop ];

			pos.x += ( raw_vector.x * bridge_segment_length( segment_type ) * 0.5 );
			pos.y += ( raw_vector.y * bridge_segment_length( segment_type ) * 0.5 );
			pos.z += ( raw_vector.z * bridge_segment_length( segment_type ) * 0.5 );

			segment_entity = create_local_entity
			(
				ENTITY_TYPE_SEGMENT,
				ENTITY_INDEX_DONT_CARE,
				ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, bridge_segment_sub_type( segment_type )),
				ENTITY_ATTR_INT_VALUE (INT_TYPE_BRIDGE_SEGMENT_TYPE, segment_type),
				ENTITY_ATTR_INT_VALUE (INT_TYPE_OBJECT_3D_SHAPE, bridge_segment_normal_object( segment_type )),
				ENTITY_ATTR_INT_VALUE (INT_TYPE_DAMAGE_LEVEL, bridge_segment_initial_damage_level( segment_type )),
				ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, pos.x, pos.y, pos.z),
				ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_HEADING, heading),
				ENTITY_ATTR_PARENT (LIST_TYPE_SEGMENT, bridge_entity),
				ENTITY_ATTR_END
			);

			pos.x += ( raw_vector.x * bridge_segment_length( segment_type ) * 0.5 );
			pos.y += ( raw_vector.y * bridge_segment_length( segment_type ) * 0.5 );
			pos.z += ( raw_vector.z * bridge_segment_length( segment_type ) * 0.5 );
		}
	}

	//
	// end segment
	//

	if ( bridge_info->end_section_type != BRIDGE_SEGMENT_TYPE_NONE )
	{
		segment_entity = create_local_entity
		(
			ENTITY_TYPE_SEGMENT,
			ENTITY_INDEX_DONT_CARE,
			ENTITY_ATTR_INT_VALUE (INT_TYPE_ENTITY_SUB_TYPE, ENTITY_SUB_TYPE_FIXED_BRIDGE_END_SECTION),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_BRIDGE_SEGMENT_TYPE, bridge_info->end_section_type),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_OBJECT_3D_SHAPE, bridge_segment_normal_object( bridge_info->end_section_type )),
			ENTITY_ATTR_INT_VALUE (INT_TYPE_DAMAGE_LEVEL, bridge_segment_initial_damage_level( bridge_info->end_section_type )),
			ENTITY_ATTR_VEC3D (VEC3D_TYPE_POSITION, end.x, end.y, end.z),
			ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE_HEADING, heading),
			ENTITY_ATTR_PARENT (LIST_TYPE_SEGMENT, bridge_entity),
			ENTITY_ATTR_END
		);
	}

















	return bridge_entity;
}





void create_local_bridge_entities (pack_modes pack_mode)
{
	entity_sub_types
		type;

	char
		filename [512];

	session_list_data_type
		*current_session;

	FILE
		*fp;

	int
		loop,
		number_of_bridges,
		start_node,
		end_node;

	vec3d
		end1,
		end2;

	float
		max_leg_height;

	entity
		*first_bridge_entity;

	if (pack_mode == PACK_MODE_BROWSE_SESSION)
	{

		return;
	}

	first_bridge_entity = NULL;

	if (command_line_run_mode == RUN_MODE_AITOOL)
	{
		//return;
	}

	ASSERT (get_valid_current_game_session ());

	current_session = get_current_game_session ();

	sprintf (filename, "%s\\%s", current_session->data_path, BRIDGE_DATA_FILENAME);

	if (!file_exist (filename))
	{
		return;
	}

	fp = safe_fopen (filename, "rb");

	fread (&number_of_bridges, sizeof (int), 1, fp);

	for (loop = 0; loop < number_of_bridges; loop ++)
	{
		fread (&end1, sizeof (vec3d), 1, fp);

		fread (&end2, sizeof (vec3d), 1, fp);

		fread (&start_node, sizeof (int), 1, fp);

		fread (&end_node, sizeof (int), 1, fp);

		fread (&type, sizeof (int), 1, fp);

		fread (&max_leg_height, sizeof (float), 1, fp);

		type = get_most_suitable_bridge_type (&end1, &end2, max_leg_height);

		if (!first_bridge_entity)
		{

			first_bridge_entity = create_bridge (&end1, &end2, start_node, end_node, type, max_leg_height);
		}
		else
		{

			create_bridge (&end1, &end2, start_node, end_node, type, max_leg_height);
		}
	}

	safe_fclose (fp);






}





































































































































































































































































































































































































































void repair_client_server_bridge_entity (entity *en)
{
	ASSERT (en);

	ASSERT (get_comms_model () == COMMS_MODEL_SERVER);

	repair_local_bridge_entity (en);


}





void repair_local_bridge_entity (entity *en)
{

	int
		start_node,
		end_node,
		count;

	entity
		*segment;

	ASSERT (en);

	//
	// repair segments
	//

	segment = get_local_entity_first_child (en, LIST_TYPE_SEGMENT);

	while (segment)
	{
		repair_local_segment_entity (segment);

		segment = get_local_entity_child_succ (segment, LIST_TYPE_SEGMENT);
	}

	//
	// set bridge alive flag
	//

	set_local_entity_int_value (en, INT_TYPE_ALIVE, TRUE);

	//
	// repair the link between nodes
	//

	start_node = get_local_entity_int_value (en, INT_TYPE_START_NODE);

	end_node = get_local_entity_int_value (en, INT_TYPE_END_NODE);

	count = get_road_link_breaks (start_node, end_node);

	count --;

	set_road_link_breaks (start_node, end_node, count);

	debug_log ("BRIDGE: Repairin bridge and route between %d and %d... broken bridge count = %d", count);
}








