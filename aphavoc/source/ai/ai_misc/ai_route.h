




























































#ifndef AI_ROUTE_H
#define AI_ROUTE_H





#define MAX_ROUTE_LENGTH 100





#define ROUTE_NODE_POSITIONS_EXTENSION   				".nde"

#define ROUTE_NODE_DATA_EXTENSION       				".dat"

#define ROUTE_NODE_LINK_POSITION_DATA_EXTENSION		".wp"





extern int
	road_nodes_loaded;





#define get_road_nodes_loaded() (road_nodes_loaded)





struct LINK_DATA
{

	int
		cost;

	unsigned short int
		node			: 14,
		breaks		: 5;
};

typedef struct LINK_DATA link_data;





struct NODE_DATA
{

	unsigned int
		node						: 14,
		safe_radius				: 14,
		number_of_links		: 7,
		visited 					: 7,
		side_occupying 		: 2,
		side_aware				: 2,
		side_exploring			: 2;

	link_data
		*links;
};

typedef struct NODE_DATA node_data;





struct NODE_LINK_DATA
{

	unsigned short int
		source,
		destination,
		path_type,
		number_of_links;

	int
		padding;

	vec3d
		*link_positions;
};

typedef struct NODE_LINK_DATA node_link_data;





extern node_data
	*road_nodes;

extern vec3d
	*road_node_positions;

extern node_link_data
	*road_node_link_positions;





extern int
	total_road_node_link_count,
	total_number_of_road_nodes,
	best_cost,
	best_recurse_level,
	current_route [MAX_ROUTE_LENGTH],
	best_route [MAX_ROUTE_LENGTH];





extern void initialise_route_data (void);

extern void load_route_data (void);

extern void read_road_route_data (void);

extern void clear_road_route_data (void);

extern void deinitialise_route_data (void);

extern void deinitialise_road_route_data (void);

extern node_link_data *get_road_link_data (int node1, int node2);

extern node_link_data *get_road_sub_route (int node1, int node2, int *count, node_link_data *start_route);

extern int set_road_link_breaks (int node1, int node2, int count);

extern int get_road_link_breaks (int node1, int node2);

extern void set_route_node_data_filename (char **route_node_data_filename, const char *filename);





#endif // AI_ROUTE_H
