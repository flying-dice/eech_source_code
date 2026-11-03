

































































#include "project.h"





static vec3d
	*pylon_sector_line_points;





void calculate_pylon_heading( entity *pred, entity *en, entity *succ, vec3d *start, vec3d *end )
{
	vec3d
		succ_pos,
		pred_pos,
		en_pos,
		v,
		ds,
		dp;

	float
		heading;

	//
	// determine the heading of the pylon based on the location of its neighbours
	//

	if ( !en ) return;

	get_local_entity_vec3d( en, VEC3D_TYPE_POSITION, &en_pos );


	if ( !pred )
	{

		pred_pos = *start;
	}		
	else
	{
		get_local_entity_vec3d( pred, VEC3D_TYPE_POSITION, &pred_pos );
	}		

	if ( !succ )
	{

		succ_pos = *end;
	}		
	else
	{
		get_local_entity_vec3d( succ, VEC3D_TYPE_POSITION, &succ_pos );
	}
		
	ds.x = succ_pos.x - en_pos.x;
	ds.z = succ_pos.z - en_pos.z;
	ds.y = 0;

	dp.x = en_pos.x - pred_pos.x;
	dp.z = en_pos.z - pred_pos.z;
	dp.y = 0;

	//
	// find the average of the two unit vectors
	//

	normalise_3d_vector( &ds );
	normalise_3d_vector( &dp );

	v.x = ds.x + dp.x;
	v.z = ds.z + dp.z;
	v.y = 0;
	
	normalise_3d_vector( &v );

	heading = atan2( v.x, v.z );

	if (!succ)
	{
		//
		// rotate last pylon in the link by 180deg ( so the terminator object is oriented correctly )
		//

		heading = wrap_angle( heading + PI );
	}

	if (heading < 0.0)
	{
		heading += PI2;
	}

	set_local_entity_float_value( en, FLOAT_TYPE_HEADING, heading );
}





vec3d	*get_pylon_cable_lines_in_sector (entity *sector_en, int *number_of_lines)
{
	entity
		*en;

	pylon
		*raw,
		*succ;

	float
		this_pylon_heading,
		next_pylon_heading;

	vec3d
		this_pylon_offset,
		next_pylon_offset;

	int
		cable_loop,
		point_loop,
		this_pylon_type,
		next_pylon_type,
		number_of_points,
		max_number_of_lines,
		number_of_pylons_in_sector;

	vec3d
		d,
		pos [NUMBER_OF_PYLON_CABLE_POINTS];

	ASSERT (sector_en);

	*number_of_lines = 0;

	number_of_points = 0;

	number_of_pylons_in_sector = 0;

	//
	// count number of pylons in sector
	//

	en = get_local_entity_first_child (sector_en, LIST_TYPE_SECTOR);

	while (en)
	{
		if (en->type == ENTITY_TYPE_PYLON)
		{
			number_of_pylons_in_sector ++;
		}

		en = get_local_entity_child_succ (en, LIST_TYPE_SECTOR);
	}

	if (number_of_pylons_in_sector == 0)
	{
		return NULL;
	}

	max_number_of_lines = number_of_pylons_in_sector * NUM_PYLON_ATTACHMENT_POINTS * (NUMBER_OF_PYLON_CABLE_POINTS - 1);

	pylon_sector_line_points = (vec3d *) malloc_fast_mem (sizeof (vec3d) * max_number_of_lines * 2);

	//
	// process each pylon pair
	//

	en = get_local_entity_first_child (sector_en, LIST_TYPE_SECTOR);

	while (en)
	{
		if (en->type == ENTITY_TYPE_PYLON)
		{
			raw = (pylon *) get_local_entity_data (en);

			if (raw->fix.alive)
			{

				this_pylon_heading = get_local_entity_float_value (en, FLOAT_TYPE_HEADING);

				this_pylon_type = get_pylon_sub_type (raw->fix.sub_type);

				if (raw->succ)
				{
					succ = (pylon *) get_local_entity_data (raw->succ);

					if (succ->fix.alive)
					{	
						next_pylon_type = get_pylon_sub_type (succ->fix.sub_type);

						next_pylon_heading = get_local_entity_float_value (raw->succ, FLOAT_TYPE_HEADING);
		
						for (cable_loop = NUM_PYLON_ATTACHMENT_POINTS - 1; cable_loop >= 0; cable_loop --)
						{
		
							this_pylon_offset = pylon_attachment_offset (this_pylon_type, cable_loop);
							next_pylon_offset = pylon_attachment_offset (next_pylon_type, cable_loop);
		
							//
							// get first point
							//
							pos [0].x = raw->fix.position.x + (this_pylon_offset.x * cos (this_pylon_heading));
							pos [0].y = raw->fix.position.y + this_pylon_offset.y;
							pos [0].z = raw->fix.position.z - (this_pylon_offset.x * sin (this_pylon_heading));
		
							//
							// get last point
							//
							pos [NUMBER_OF_PYLON_CABLE_POINTS - 1].x = succ->fix.position.x + (next_pylon_offset.x * cos (next_pylon_heading));
							pos [NUMBER_OF_PYLON_CABLE_POINTS - 1].y = succ->fix.position.y + next_pylon_offset.y;
							pos [NUMBER_OF_PYLON_CABLE_POINTS - 1].z = succ->fix.position.z - (next_pylon_offset.x * sin (next_pylon_heading));
		
							//
							// calculate intermediate points
							//
		
							d.x = (pos [NUMBER_OF_PYLON_CABLE_POINTS - 1].x - pos [0].x) / NUMBER_OF_PYLON_CABLE_POINTS;
							d.y = (pos [NUMBER_OF_PYLON_CABLE_POINTS - 1].y - pos [0].y) / NUMBER_OF_PYLON_CABLE_POINTS;
							d.z = (pos [NUMBER_OF_PYLON_CABLE_POINTS - 1].z - pos [0].z) / NUMBER_OF_PYLON_CABLE_POINTS;
		
							for (point_loop = 1 ; point_loop < NUMBER_OF_PYLON_CABLE_POINTS - 1 ; point_loop++)
							{
								pos [point_loop].x = pos [point_loop - 1].x + d.x;
								pos [point_loop].y = pos [point_loop - 1].y + d.y;
								pos [point_loop].z = pos [point_loop - 1].z + d.z;
							}
		
							//
							// add "sag"
							//
		
							for (point_loop = 1 ; point_loop < NUMBER_OF_PYLON_CABLE_POINTS - 1 ; point_loop++)
							{
								pos [point_loop].y -= (get_pylon_sag_value (point_loop) * MAX_PYLON_CABLE_SAG);
							}
						}

						//
						// copy points into return list
						//

						for (point_loop = 0 ; point_loop < NUMBER_OF_PYLON_CABLE_POINTS - 1 ; point_loop++)
						{
							pylon_sector_line_points [number_of_points] = pos [point_loop];
							pylon_sector_line_points [number_of_points + 1] = pos [point_loop + 1];

							number_of_points += 2;
						}
					}
				}
			}
		}

		en = get_local_entity_child_succ (en, LIST_TYPE_SECTOR);
	}

	if (number_of_points == 0)
	{
		free_mem (pylon_sector_line_points);

		return NULL;
	}
	else
	{
	
		(*number_of_lines) = (number_of_points >> 1);

		return pylon_sector_line_points;
	}
}
		



