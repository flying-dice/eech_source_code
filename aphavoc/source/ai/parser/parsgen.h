

































































extern entity
	*parser_keysite,
	*parser_force,
	*parser_landing,
	*parser_route,
	*parser_waypoint,
	*parser_member,
	*parser_object;





extern char
	*campaign_population_filename,
	*population_placement_filename,
	*side_data_filename;





extern int
	parser_flight_dynamics_options;





#define set_parser_keysite(KEYSITE) (parser_keysite = (KEYSITE))

#define set_parser_force(FORCE) (parser_force = (FORCE))

#define set_parser_landing(LANDING) (parser_landing = (LANDING))

#define set_parser_route(ROUTE) (parser_route = (ROUTE))

#define set_parser_waypoint(WAYPOINT) (parser_waypoint = (WAYPOINT))

#define set_parser_member(MEMBER) (parser_member = (MEMBER))

#define set_parser_object(OBJECT) (parser_object = (OBJECT))

#define get_parser_keysite() (parser_keysite)

#define get_parser_force() (parser_force)

#define get_parser_landing() (parser_landing)

#define get_parser_route() (parser_route)

#define get_parser_waypoint() (parser_waypoint)

#define get_parser_member() (parser_member)

#define get_parser_object() (parser_object)





extern void initialise_parser (void);

extern void deinitialise_parser (void);

extern void update_campaign_triggers (void);

extern int parser_campaign_file (const char *filename, int *offset);

extern int update_script_triggers (campaign_trigger trigger);

extern void generate_key_bound_triggers (void);




