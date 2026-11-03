

































































typedef enum CRATER_TYPES
{
	CRATER_TYPE_NONE,

	CRATER_TYPE_SMALL_AP_SHELL,
	CRATER_TYPE_LARGE_AP_SHELL,

	CRATER_TYPE_SMALL_EXPLOSION,
	CRATER_TYPE_MEDIUM_EXPLOSION,
	CRATER_TYPE_LARGE_EXPLOSION,

	NUM_CRATER_TYPES

} crater_types;





struct CRATER
{
	fixed
		fix;

	float
		crater_lifetime,
		scale;

	list_link
		update_link;
};

typedef struct CRATER crater;





#include "cr_creat.h"

#include "cr_dbase.h"

#include "cr_draw.h"

#include "cr_dstry.h"

#include "cr_float.h"

#include "cr_funcs.h"

#include "cr_list.h"

#include "cr_msgs.h"

#include "cr_pack.h"

#include "cr_updt.h"




