

































































#include "project.h"





#define DEBUG_MODULE 0





static void set_local_int_value (entity *en, int_types type, int value)
{
   task
      *raw;







   raw = (task *) get_local_entity_data (en);

   switch (type)
   {

      case INT_TYPE_AWARDED_MEDALS:

      {
         raw->awarded_medals = value;

         break;
      }

      case INT_TYPE_AWARDED_PROMOTION:

      {
         raw->awarded_promotion = value;

         break;
      }

      case INT_TYPE_CRITICAL_TASK:

      {
         raw->critical_task = value;

         break;
      }

      case INT_TYPE_ENTITY_SUB_TYPE:

      {
         raw->sub_type = value;

         break;
      }

      case INT_TYPE_KILLS:

      {
         raw->kills = value;

         break;
      }

      case INT_TYPE_LOSSES:

      {
         raw->losses = value;

         break;
      }

      case INT_TYPE_MOVEMENT_TYPE:

      {
         raw->movement_type = value;

         break;
      }

      case INT_TYPE_PLAYER_TASK:

      {
         raw->player_task = value;

         break;
      }

      case INT_TYPE_ROUTE_CHECK_SUM:

      {
         raw->route_check_sum = value;

         break;
      }

      case INT_TYPE_ROUTE_LENGTH:

      {
         raw->route_length = value;

         break;
      }

      case INT_TYPE_SIDE:

      {
         raw->side = value;

         break;
      }

      case INT_TYPE_TASK_COMPLETED:

      {
         raw->task_completed = value;

         break;
      }

      case INT_TYPE_TASK_DIFFICULTY:

      {
         raw->difficulty = value;

         break;
      }

      case INT_TYPE_TASK_ID:

      {
         raw->task_id = value;

         break;
      }

      case INT_TYPE_TASK_SCORE:

      {
         raw->task_score = value;

         break;
      }

      case INT_TYPE_TASK_STATE:

      {
         raw->task_state = (task_state_types) value;

         break;
      }

      case INT_TYPE_TASK_TERMINATED:

      {
         raw->task_terminated = (task_terminated_types) value;

         break;
      }

      default:

      {
         debug_fatal_invalid_int_type (en, type);

         break;
      }
   }
}























































static int get_local_int_value (entity *en, int_types type)
{
   task
      *raw;

   int
      value;

   raw = (task *) get_local_entity_data (en);

   switch (type)
   {

      case INT_TYPE_ASSESS_LANDING:

      {
         value = task_database [raw->sub_type].assess_landing;

         break;
      }

      case INT_TYPE_AWARDED_MEDALS:

      {
         value = raw->awarded_medals;

         break;
      }

      case INT_TYPE_AWARDED_PROMOTION:

      {
         value = raw->awarded_promotion;

         break;
      }

      case INT_TYPE_CRITICAL_TASK:

      {
         value = raw->critical_task;

         break;
      }

      case INT_TYPE_DEFAULT_ENGAGE_ENEMY:

      {
         value = task_database [raw->sub_type].engage_enemy;

         break;
      }

      case INT_TYPE_DELAY_TASK_ASSIGNMENT:

      {
         value = task_database [raw->sub_type].delay_task_assignment;

         break;
      }

      case INT_TYPE_ENTITY_SUB_TYPE:

      {
         value = raw->sub_type;

         break;
      }

      case INT_TYPE_KILLS:

      {
         value = raw->kills;

         break;
      }

      case INT_TYPE_LOSSES:

      {
         value = raw->losses;

         break;
      }

      case INT_TYPE_MAP_ICON:

      {
         value = MAP_ICON_MISSION;

         break;
      }

      case INT_TYPE_MINIMUM_MEMBER_COUNT:

      {
         value = task_database [raw->sub_type].minimum_member_count;

         break;
      }

      case INT_TYPE_MOVEMENT_TYPE:

      {
         value = raw->movement_type;

         break;
      }

      case INT_TYPE_PERFORM_DEBRIEF:

      {
         value = task_database [raw->sub_type].perform_debrief;

         break;
      }

      case INT_TYPE_PLAYER_TASK:

      {
         value = raw->player_task;

         break;
      }

      case INT_TYPE_POINTS_VALUE:

      {
         value = task_database [raw->sub_type].task_completed_score;

         break;
      }

      case INT_TYPE_PRIMARY_TASK:

      {
         value = task_database [raw->sub_type].primary_task;

         break;
      }

      case INT_TYPE_ROUTE_CHECK_SUM:

      {
         value = raw->route_check_sum;

         break;
      }

      case INT_TYPE_ROUTE_LENGTH:

      {
         value = raw->route_length;

         break;
      }

      case INT_TYPE_RULES_OF_ENGAGEMENT:

      {
         value = task_database [raw->sub_type].rules_of_engagement;

         break;
      }

      case INT_TYPE_SIDE:

      {
         value = raw->side;

         break;
      }

      case INT_TYPE_SOLO_TASK:

      {
         value = task_database [raw->sub_type].solo_task;

         break;
      }

      case INT_TYPE_TASK_CATEGORY:

      {
         value = task_database [raw->sub_type].task_category;

         break;
      }

      case INT_TYPE_TASK_COMPLETED:

      {
         value = raw->task_completed;

         break;
      }

      case INT_TYPE_TASK_DIFFICULTY:

      {
         value = raw->difficulty;

         break;
      }

      case INT_TYPE_TASK_ID:

      {
         value = raw->task_id;

         break;
      }

      case INT_TYPE_TASK_OBJECTIVE_PREVIEW_TYPE:

      {
         value = task_database [raw->sub_type].task_objective_preview;

         break;
      }

      case INT_TYPE_TASK_PASS_PERCENTAGE_PARTIAL:

      {
         value = task_database [raw->sub_type].task_pass_percentage_partial;

         break;
      }

      case INT_TYPE_TASK_PASS_PERCENTAGE_SUCCESS:

      {
         value = task_database [raw->sub_type].task_pass_percentage_success;

         break;
      }

      case INT_TYPE_TASK_ROUTE_SEARCH:

      {
         value = task_database [raw->sub_type].task_route_search;

         break;
      }

      case INT_TYPE_TASK_SCORE:

      {
         value = raw->task_score;

         break;
      }

      case INT_TYPE_TASK_STATE:

      {
         value = raw->task_state;

         break;
      }

      case INT_TYPE_TASK_TARGET_CLASS:

      {
         value = task_database [raw->sub_type].task_default_target_class;

         break;
      }

      case INT_TYPE_TASK_TARGET_SOURCE:

      {
         value = task_database [raw->sub_type].task_default_target_source;

         break;
      }

      case INT_TYPE_TASK_TARGET_TYPE:

      {
         value = task_database [raw->sub_type].task_default_target_type;

         break;
      }

      case INT_TYPE_TASK_TERMINATED:

      {
         value = raw->task_terminated;

         break;
      }

      case INT_TYPE_VISIBLE_TASK:

      {
         value = task_database [raw->sub_type].visible_task;

         break;
      }

      case INT_TYPE_WAIT_FOR_END_TASK:

      {
         value = task_database [raw->sub_type].wait_for_end_task;

         break;
      }

      default:

      {
         debug_fatal_invalid_int_type (en, type);

         break;
      }
   }

   return (value);
}

























































































































































