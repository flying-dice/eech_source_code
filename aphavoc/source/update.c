

































































#include "project.h"





// 2 levels

#define UPDATE_FUNCTION_DEBUG 0





update_function_data_type
	*update_function_list;

int
	update_counter;





static void reset_update_function_list (void);





void initialise_update_function_list (void)
{

	#if UPDATE_FUNCTION_DEBUG

	debug_log ("UPDATE FUNCTION: initialise");

	#endif

	update_counter = 0;

	reset_update_function_list ();
}





void deinitialise_update_function_list (void)
{

	#if UPDATE_FUNCTION_DEBUG

	debug_log ("UPDATE FUNCTION: deinitialise");

	#endif

	reset_update_function_list ();
}





void reset_update_function_list (void)
{

	#if UPDATE_FUNCTION_DEBUG

	debug_log ("UPDATE FUNCTION: reset update function list");

	#endif

	while (update_function_list)
	{

		remove_update_function (update_function_list->function);
	}

	debug_assert (update_counter == 0);

	update_function_list = NULL;
}





int add_update_function (void ((*function) (void)), float sleep_time, float offset_time)
{

	update_function_data_type
		*list_ptr,
		*new_update_function;

	int
		unique_in_list_flag = TRUE;

	if (!function)
	{

		return FALSE;
	}

	#if UPDATE_FUNCTION_DEBUG

	update_counter ++;

	debug_log ("UPDATE FUNCTION: add function %d count = %d", function, update_counter);

	#endif

	list_ptr = update_function_list;

	//
	// Check for duplications, BUT only warn on them
	//

	while (list_ptr)
	{

		if (list_ptr->function == function)
		{

			debug_log ("UPDATE: WARNING FUNCTION ALREADY ON UPDATE LIST");

			unique_in_list_flag = FALSE;
		}

		list_ptr = list_ptr->next;
	}

	//
	//
	//

	new_update_function = (update_function_data_type *) malloc_heap_mem (sizeof (update_function_data_type));

	new_update_function->function = function;

	new_update_function->sleep_time = sleep_time;

	new_update_function->last_update_timer = offset_time;

	new_update_function->next = update_function_list;

	update_function_list = new_update_function;

	return unique_in_list_flag;
}





int remove_update_function (void ((*function) (void)))
{

	update_function_data_type
		*prev_update_function,
		*update_function;

	if (!function)
	{

		return FALSE;
	}

	update_function = update_function_list;

	prev_update_function = NULL;

	while (update_function)
	{

		if (update_function->function == function)
		{

			if (prev_update_function)
			{

				prev_update_function->next = update_function->next;
			}
			else
			{

				update_function_list = update_function->next;
			}

			#if UPDATE_FUNCTION_DEBUG

			update_counter --;
		
			debug_log ("UPDATE FUNCTION: remove function %d, counter = %d", update_function, update_counter);
		
			#endif

			free_mem (update_function);

			return TRUE;
		}

		prev_update_function = update_function;

		update_function = update_function->next;
	}

	return FALSE;
}





void update_update_functions (void)
{

	update_function_data_type
		*update_function;

	update_function = update_function_list;

	while (update_function)
	{

		update_function->last_update_timer -= get_delta_time ();

		if (update_function->last_update_timer <= 0.0)
		{

			#if UPDATE_FUNCTION_DEBUG >= 2

			debug_log ("UPDATE FUNCTION: updating function %d, last updated %d, sleep time %d", update_function->function, update_function->last_update_timer, update_function->sleep_time);

			#endif

			update_function->function ();

			update_function->last_update_timer = update_function->sleep_time;
		}

		update_function = update_function->next;
	}
}





