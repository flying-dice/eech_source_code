

































































#include "project.h"





#define DEBUG_MODULE 			0





static void update_server (entity *en)
{
	task
		*raw;

	raw = (task *) get_local_entity_data (en);







	//
	// Unassigned tasks terminated if expire time
	//	Assigned tasks terminated if stop time reached
	//

	switch (raw->task_state)
	{
		case TASK_STATE_UNASSIGNED:
		{
			//
			// Only decrement the expire timer if the task requires it (i.e. non-zero expire timer)
			//
			
			if (raw->expire_timer > 0.0)
			{
				raw->expire_timer -= get_delta_time ();
		
				if (raw->expire_timer <= 0.0)
				{
					raw->expire_timer = 0.0;
		






					notify_local_entity (ENTITY_MESSAGE_TASK_TERMINATED, en, en, TASK_TERMINATED_EXPIRE_TIME_REACHED);
				}
			}

			break;
		}

		case TASK_STATE_ASSIGNED:
		{
			//
			// Only decrement the stop timer if the task requires it (i.e. non-zero stop timer)
			//













			
			if (raw->stop_timer > 0.0)
			{
				raw->stop_timer -= get_delta_time ();
		
				if (raw->stop_timer <= 0.0)
				{
					raw->stop_timer = 0.0;







					notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, en, en, TASK_TERMINATED_STOP_TIME_REACHED);
				}
			}

			break;
		}

		case TASK_STATE_COMPLETED:
		{
			//
			// Expire Timer is used to destroy completed tasks after a certain time 
			//

			ASSERT (task_database [raw->sub_type].primary_task);
			
			if (raw->expire_timer > 0.0)
			{
				raw->expire_timer -= get_delta_time ();
		
				if (raw->expire_timer <= 0.0)
				{
					raw->expire_timer = 0.0;
		





	
					//
					// Destroy task
					//
				
					destroy_local_entity_family (en);
		
					transmit_entity_comms_message (ENTITY_COMMS_DESTROY_LOCAL_FAMILY, en);
				}
			}
			
			break;
		}
	}
}





static void update_client (entity *en)
{
	task
		*raw;

	raw = (task *) get_local_entity_data (en);

	switch (raw->task_state)
	{
		case TASK_STATE_UNASSIGNED:
		{
			//
			// Only decrement the expire timer if the task requires it (i.e. non-zero expire timer)
			//
			
			if (raw->expire_timer > 0.0)
			{
				raw->expire_timer -= get_delta_time ();
		
				raw->expire_timer = max (raw->expire_timer, 0.0f);
			}

			break;
		}

		case TASK_STATE_ASSIGNED:
		{
			//
			// Only decrement the stop timer if the task requires it (i.e. non-zero stop timer)
			//
			
			if (raw->stop_timer > 0.0)
			{
				raw->stop_timer -= get_delta_time ();
		
				raw->stop_timer = max (raw->stop_timer, 0.0f);
			}

			break;
		}
	}
}















