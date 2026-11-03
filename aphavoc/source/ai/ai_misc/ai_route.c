

































































#include "project.h"

#include "ai_route.h"





// 3 levels of debug

#define DEBUG_MODULE 0

#define SAFE_RADIUS 1

#define CHECK_ROUTE_NODE_HEIGHT 0

#define AI_ROUTE_ROAD_SEARCH 0





#define ALLIED_SECTOR         0

#define ENEMY_SECTOR          2

#define FRONT_LINE_SECTOR     1





static int read_road_route_node_data (char *name);

static int read_road_route_node_positions (char *name);

static int read_road_route_node_link_positions (char *name);





node_data
   *road_nodes;

vec3d
   *road_node_positions;

node_link_data
   *road_node_link_positions;

int
   duplicate_count = 0,
	road_nodes_loaded;





int
   total_road_node_link_count,
   total_number_of_road_nodes,
   best_recurse_level = INT_MAX,
   current_route [MAX_ROUTE_LENGTH],
   best_route [MAX_ROUTE_LENGTH];

entity
	**ground_landing_entities;





void initialise_route_data (void)
{

	deinitialise_route_data ();







	road_nodes_loaded = FALSE;

   road_node_link_positions = NULL;
   road_node_positions = NULL;
   road_nodes = NULL;

   total_road_node_link_count = 0;

   total_number_of_road_nodes = 0;
}





void load_route_data (void)
{
	deinitialise_road_route_data ();

	read_road_route_data ();
}





void deinitialise_route_data (void)
{







	deinitialise_road_route_data ();
}





void deinitialise_road_route_data (void)
{

   int
      loop;

   //
   // Free road_nodes
   //

   if (road_nodes)
   {
   
      for (loop = 0; loop < total_number_of_road_nodes; loop ++)
      {

         if (road_nodes [loop].links)
         {
   
            free_mem (road_nodes [loop].links);
      
            #if AI_DEBUG
         
            ai_memory_counter --;
         
            #endif
         }
      }
   
      free_mem (road_nodes);

      #if AI_DEBUG
   
		ai_memory_counter --;
	
		#endif
	}

	//
	// Free node road_node_link_positions
	//

	if (road_node_positions)
	{

		free_mem (road_node_positions);

		#if AI_DEBUG
	
		ai_memory_counter --;
	
		#endif
	}

	//
	// Free all road_node_link_positions for node links
	//

	if (road_node_link_positions)
	{
	
		for (loop = 0; loop < total_road_node_link_count; loop ++)
		{

			if (road_node_link_positions [loop].number_of_links > 0)
			{

				ASSERT (road_node_link_positions [loop].link_positions);
	
				free_mem (road_node_link_positions [loop].link_positions);
		
				#if AI_DEBUG
			
				ai_memory_counter --;
			
				#endif
			}
		}
	
		free_mem (road_node_link_positions);

		#if AI_DEBUG
	
		ai_memory_counter --;
	
		#endif
	}

   road_node_link_positions = NULL;
   road_node_positions = NULL;
   road_nodes = NULL;

   total_road_node_link_count = 0;
   total_number_of_road_nodes = 0;
}





void read_road_route_data (void)
{







   if (read_road_route_node_data ("ROADS"))
	{
	   read_road_route_node_positions ("ROADS");

   	read_road_route_node_link_positions ("ROADS");
	}
	else if (read_road_route_node_data ("ROADDATA"))
	{
	   read_road_route_node_positions ("ROADDATA");

   	read_road_route_node_link_positions ("ROADDATA");
	}
}





//
// Data format:
//
// Number of nodes
// loop 	{
//				node_id,
//				number_of_links
//				list of links
//			}
//






















































































































































































































//
// Data format
//
// Number of nodes
// list of vec3d positions
//





















































































//
// Data format
//
// Source
// Destination
// Number of links
// List of node_link_data for each position and cost.
//

static int read_road_route_node_link_positions (char *name)
{




































































































































	road_nodes_loaded = TRUE;


}





void clear_road_route_data (void)
{

	int
		loop;

	if (!get_road_nodes_loaded ())
	{

		return;
	}

	best_recurse_level = 0;

	//
	// initialise node visited data
	//

	if (road_nodes)
	{
	
		for (loop = 0; loop < total_number_of_road_nodes; loop ++)
		{
	
			road_nodes [loop].visited = MAX_ROUTE_LENGTH;
		}
	}
}




// road routing system




node_link_data *get_road_link_data (int node1, int node2)
{

	int
		loop;

	//
	// check database loaded
	//

	ASSERT (road_nodes);
	ASSERT (road_node_positions);
	ASSERT (road_node_link_positions);

	if (!get_road_nodes_loaded ())
	{

		debug_fatal ("AI_ROUTE: data not loaded");
	}

	//
	// Easy case
	//

	if (node1 == node2)
	{

		return NULL;
	}

	//
	// correct order
	//

	if (node1 > node2)
	{

		int
			temp;

		temp = node1;

		node1 = node2;

		node2 = temp;
	}

	//
	// Search for route
	//

	for (loop = (total_road_node_link_count - 1); loop >= 0; loop --)
	{

		//
		// setup index pointers for forward wp list
		//

		if ((road_node_link_positions [loop].source == node1) && (road_node_link_positions [loop].destination == node2))
		{

			return &road_node_link_positions [loop];
		}
	}

	debug_log ("AI_ROUTE: Warning! Link between %d and %d not found", node1, node2);

	return NULL;
}





node_link_data *get_road_sub_route (int node1, int node2, int *count, node_link_data *start_route)
{

	int
		loop;

	//
	// check database loaded
	//

	ASSERT (road_nodes);
	ASSERT (road_node_positions);
	ASSERT (road_node_link_positions);

	if (!get_road_nodes_loaded ())
	{

		debug_fatal ("AI_ROUTE: data not loaded");
	}

	//
	// Easy case
	//

	if (node1 == node2)
	{

		return NULL;
	}

	//
	// correct order
	//

	if (node1 > node2)
	{

		int
			temp;

		temp = node1;

		node1 = node2;

		node2 = temp;
	}

	//
	// zip to start_route position
	//

	while (TRUE)
	{
	
		loop = total_road_node_link_count - 1;
	
		if (start_route)
		{
		
			for (; loop >= 0; loop --)
			{
		
				//
				// setup index pointers for forward wp list
				//
		
				if (&road_node_link_positions [loop] == start_route)
				{

					loop --;
		
					break;
				}
			}
		}
	
		//
		// Search for route
		//
	
		for (; loop >= 0; loop --)
		{
	
			//
			// setup index pointers for forward wp list
			//
	
			if ((road_node_link_positions [loop].source == node1) && (road_node_link_positions [loop].destination == node2))
			{
	
				*count = road_node_link_positions [loop].number_of_links;
	
				return &road_node_link_positions [loop];
			}
		}

		if (start_route)
		{

			start_route = NULL;
		}
		else
		{

			break;
		}
	}

	*count = 0;























































	return NULL;
}





int set_road_link_breaks (int node1, int node2, int count)
{

	unsigned int
		loop;

	ASSERT (road_nodes);
	ASSERT (road_node_positions);

	//
	// Set valid node1 -> node2 link
	//

	if (count > 32)
	{

		debug_fatal ("AI_ROUTE: too many bridges destroyed on link between %d and %d (max 32)", node1, node2);
	}

	for (loop = 0; loop < road_nodes [node1].number_of_links; loop ++)
	{

		if (road_nodes [node1].links [loop].node == node2)
		{

			road_nodes [node1].links [loop].breaks = count;

			//
			// Set valid node2 -> node1 link
			//

			for (loop = 0; loop < road_nodes [node2].number_of_links; loop ++)
			{

				if (road_nodes [node2].links [loop].node == node1)
				{
	
					road_nodes [node2].links [loop].breaks = count;

					return TRUE;
				}
			}

			debug_fatal ("AI_ROUTE: only found 1 way link between %d and %d", node1, node2);
		}
	}

	return FALSE;
}





int get_road_link_breaks (int node1, int node2)
{

	unsigned int
		loop2,
		loop;

	ASSERT (road_nodes);
	ASSERT (road_node_positions);

	//
	// find node 1
	//

	for (loop = 0; loop < road_nodes [node1].number_of_links; loop ++)
	{

		if (road_nodes [node1].links [loop].node == node2)
		{

			//
			// Check that node2 has same valid flag
			//
	
			for (loop2 = 0; loop2 < road_nodes [node2].number_of_links; loop2 ++)
			{
	
				if (road_nodes [node2].links [loop2].node == node1)
				{

					ASSERT (road_nodes [node1].links [loop].breaks == road_nodes [node2].links [loop2].breaks);
	
					return road_nodes [node1].links [loop].breaks;
				}
			}
		}
	}

	return -1;
}





#define MAX_SAFE_RADIUS_CHECK (50 * KILOMETRE)





































































void set_route_node_data_filename (char **route_node_data_filename, const char *filename)
{

	if (*route_node_data_filename)
	{

		free_mem (*route_node_data_filename);

		*route_node_data_filename = NULL;
	}

	*route_node_data_filename = (char *) malloc_heap_mem (sizeof (char) * (strlen (filename) + 1));

	strcpy (*route_node_data_filename, filename);






}




