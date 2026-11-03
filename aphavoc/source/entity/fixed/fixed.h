

































































struct FIXED_
{
	vec3d
		position;

	list_root
		task_dependent_root,
		target_root;

	list_link
		designated_target_link, // Jabberwock - Designated targets
		gunship_target_link,
		sector_link;

	unsigned int
		sub_type			 	  				:NUM_ENTITY_SUB_TYPE_FIXED_BITS,
		object_3d_shape  		  			:NUM_OBJECT_3D_SHAPE_BITS,
		alive					  				:NUM_ALIVE_BITS,
		side			 	  					:NUM_SIDE_BITS,
		gunship_radar_los_clear			:NUM_GUNSHIP_RADAR_LOS_CLEAR_BITS,
		gunship_radar_target_index		:NUM_GUNSHIP_RADAR_TARGET_INDEX_BITS,
		object_drawn_once_this_frame	:NUM_OBJECT_DRAWN_ONCE_THIS_FRAME_BITS;

	float
		heading;
};

typedef struct FIXED_ fixed;







extern void restore_local_fixed_entity (entity *en);





#include	"fx_dbase.h"

















#include	"fx_objdb.h"

















#include "segment/segment.h"

#include "site/site.h"

#include "siteupdt/siteupdt.h"




