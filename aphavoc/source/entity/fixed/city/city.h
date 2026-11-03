

































































struct CITY
{
	fixed
		fix;

	list_root
		city_building_root;

	vec3d
		bounding_box_min,
		bounding_box_max;

	unsigned int
		approximation_object_3d_shape		: NUM_OBJECT_3D_SHAPE_BITS;
};

typedef struct CITY city;





#include "ct_creat.h"

#include "ct_draw.h"

#include "ct_dstry.h"

#include "ct_float.h"

#include "ct_funcs.h"

#include "ct_int.h" 

#include "ct_list.h"

#include "ct_msgs.h"

#include "ct_pack.h"

#include "ct_vec3d.h"




