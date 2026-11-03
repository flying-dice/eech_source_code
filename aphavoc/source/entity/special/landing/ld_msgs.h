

































































extern void overload_landing_message_responses (void);

#if LANDING_ROUTE_CHECK

extern void destroy_debug_landing_route_check (void);

extern void destroy_debug_entity_landing_route_check (entity *en);

extern void pack_debug_entity_landing_locks (pack_modes mode);

extern void unpack_debug_entity_landing_locks (pack_modes mode);





enum DEBUG_LANDING_LOCK
{

	DEBUG_LANDING_LOCK_RESERVE_LOCK,
	DEBUG_LANDING_LOCK_LANDING_SITE_LOCK,
	DEBUG_LANDING_LOCK_LANDING_ROUTE_LOCK,
	DEBUG_LANDING_LOCK_TAKEOFF_ROUTE_LOCK,
	NUM_DEBUG_LANDING_LOCKS
};

typedef enum DEBUG_LANDING_LOCK debug_landing_lock;





struct DEBUG_LANDING_LOCK_TYPE
{

	entity
		*en,
		*landing_en;

	int
		locks [NUM_DEBUG_LANDING_LOCKS];

	struct DEBUG_LANDING_LOCK_TYPE
		*next;
};

typedef struct DEBUG_LANDING_LOCK_TYPE debug_landing_lock_type;





extern debug_landing_lock_type
	*debug_landing_lock_list;

#endif




