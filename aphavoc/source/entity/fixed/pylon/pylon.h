

































































struct PYLON
{
	fixed
		fix;

	int
		damage_level;

	entity
		*succ;
};

typedef struct PYLON pylon;





#define NUMBER_OF_PYLON_CABLE_POINTS	6

#define MAX_PYLON_CABLE_SAG				14.0

#define PYLON_DAMAGE_LEVEL					1000





#include "py_creat.h"

#include "py_dbase.h"

#include "py_draw.h"

#include "py_dstry.h"

#include "py_funcs.h"

#include "py_float.h"

#include "py_int.h"

#include "py_msgs.h"

#include "py_pack.h"

#include "py_ptr.h"





extern void calculate_pylon_heading( entity *pred, entity *en, entity *succ, vec3d *start, vec3d *end );

extern vec3d	*get_pylon_cable_lines_in_sector (entity *sector_en, int *number_of_lines);




