

































































struct UPDATE
{
	list_root
		update_root;
};

typedef struct UPDATE update;





extern unsigned int
	moved_entities[],
	rotated_entities[],
	updated_entities[];

extern entity
	*update_entity,
	*update_succ;

#define get_update_entity() (update_entity)

#define set_update_succ(EN) (update_succ = (EN))

#define get_update_succ() (update_succ)





extern void set_update_entity (entity *en);

extern void update_client_server_entities (void);

extern int set_entity_update_frame_rate (int frame_rate);





#include "up_creat.h"

#include "up_dstry.h"

#include "up_funcs.h"

#include "up_list.h"

#include "up_msgs.h"

#include "up_pack.h"




