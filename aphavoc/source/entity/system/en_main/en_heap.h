

































































extern int 
	start_of_local_entity_heap;

extern entity
	*first_free_local_entity,
	*first_used_local_entity;





extern int
	number_of_entities;

extern entity
	*entities,
	*first_free_entity,
	*first_used_entity;





#define ENTITY_INDEX_DONT_CARE 		(-1)

//Xhit: To mark that a entity should be created localy only without communication
#define ENTITY_INDEX_CREATE_LOCAL 	(-2)

#define MAX_NUM_ENTITIES				(131072 * 4)

//
// index is signed therefore 1 extra bit required
//

#define NUM_ENTITY_INDEX_PACK_BITS	(20)





#define get_local_entity_list() (first_used_entity)


//
// THESE FUNCTIONS ARE NOT PROTECTED AGAINST (EN) == NULL
//


#define get_local_entity_index(EN) ((EN) - entities)

#define get_local_entity_ptr(INDEX) (&entities[(INDEX)])

#define get_client_server_entity_random_number_seed(EN) ((EN) - entities)


//
// THESE FUNCTIONS ARE PROTECTED AGAINST (EN) == NULL
//


#define get_local_entity_safe_index(EN) ((EN) ? ((EN) - entities) : (ENTITY_INDEX_DONT_CARE))

//
// WARNING! If (INDEX) is a function it will get called twice which may cause problems.
//

#define get_local_entity_safe_ptr(INDEX) ((INDEX != ENTITY_INDEX_DONT_CARE) ? (&entities[(INDEX)]) : (NULL))

//
// use this function to get the index number of the next available entity
//

#define get_next_free_entity_index() (get_local_entity_safe_index (first_free_entity))





//Xhit: returns the next available local entity .
#define get_next_free_local_entity_index() (get_local_entity_safe_index (first_free_local_entity))





//Xhit: Functions to get and set local entitys
extern entity *get_free_local_entity (int index);

extern void set_free_local_entity (entity *en);





extern void initialise_entity_heap (int num_entities);

extern void reinitialise_entity_heap (void);

extern void deinitialise_entity_heap (void);

extern entity *get_free_entity (int index);

extern void set_free_entity (entity *en);

extern void pack_entity_safe_index (int index);

extern int unpack_entity_safe_index (void);

extern void pack_entity_safe_ptr (entity *en);

extern entity *unpack_entity_safe_ptr (void);




