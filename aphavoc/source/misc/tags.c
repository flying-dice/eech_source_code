

































































#include "project.h"





#define DEBUG_MODULE 0





//file_tag_variable_type
	//*file_tag_variables;





const char
   *application_tag_strings [] =
   {
		"AIRBASE_SITE",
		"AIR_NODE_ROUTE_DATA",
		"AMMO_DUMP_SITE",
		"AMMO_SUPPLIES",
		"ANCHORAGE_SITE",
		"ASSIGN_GROUP_TASK",
		"ATTITUDE",
		"AUTO_ASSIGN_GUNSHIP",
		"BUILDING",
		"BRIDGE",
		"BRIEFING_ALLIED_CAUSE_TEXT",
		"BRIEFING_ENEMY_CAUSE_TEXT",
		"BRIEFING_EFFECT_TEXT",
		"CAMPAIGN_DATA",
		"CAMPAIGN_REQUIRES_APACHE_HAVOC",
		"CAMPAIGN_TRIGGER",
		"CAMPAIGN_TRIGGER_RESULT",
		"COLOUR",
		"COUNT",
		"CREATE_GROUP",
		"CREATE_MEMBERS",
		"CREATE_MOBILE",
		"CREATE_TASK",
		"CREATE_WAYPOINT",
		"CURRENT_PLAYER",
		"DAYS",
		"DEBRIEFING_TEXT",
		"DELTA_TIME",
		"DESTROY_ENTITY",
		"DIVISION_ID_LIST",
		"DURATION",
		"END_CAMPAIGN",
		"EXPERIENCE_POINTS",
		"FACTION",
		"FACTORY_SITE",
		"FAILURE",
		"FARP_SITE",
		"FLAG_PYLONS",
		"FLIGHT_DYNAMICS",
		"FORCE_NAME",
		"FORMATION",
		"FORMATION_COMPONENT",
		"FRONTLINE_FORCES",
		"FUEL_SUPPLIES",
		"GET_ALTITUDE",
		"GET_ENTITY_GROUP",
		"GET_ENTITY_PLAYER",
		"GET_ENTITY_SIDE",
		"GET_ENTITY_TARGET",
		"GET_ENTITY_TASK",
		"GET_ENTITY_TYPE",
		"GET_ENTITY_X_POSITION",
		"GET_ENTITY_Y_POSITION",
		"GET_ENTITY_Z_POSITION",
		"GET_INFINITE_AMMO",
		"GET_INFINITE_FUEL",
		"GET_INFINITE_WEAPONS",
		"GET_INVULNERABLE",
		"GET_NEAREST_SIDE_AIRCRAFT",
		"GET_NEAREST_SIDE_KEYSITE",
		"GET_NEAREST_SIDE_VEHICLE",
		"GET_POSITION_TERRAIN_TYPE",
		"GET_RANGE",
		"GET_SIDE_BALANCE_OF_POWER",
		"GET_SIDE_SECTOR_COUNT",
		"GET_TIME",
		"GET_TRIGGER_ENTITY",
		"GET_USER_ENTITY",
		"GET_VELOCITY",
		"GET_WEATHER",
		"GOAL",
		"GROUP",
		"HARDWARE_RESERVES",
		"HEADING",
		"HOURS",
		"KEYSITE",
		"KEYSITE_END",
		"LANGUAGE_ENGLISH",
		"LANGUAGE_FRENCH",
		"LANGUAGE_GERMAN",
		"LANGUAGE_ITALIAN",
		"LANGUAGE_POLISH",
		"LANGUAGE_SPANISH",
		"LANGUAGE_TEXT_END",
		"LANGUAGE_TEXT_START",
		"LANGUAGE_TEXT_STOP",
		"LENGTH",
		"LIST",
		"LONG_TEXT_START",
		"MAP_SECTOR_SIZE",
		"MAP_X_MAX",
		"MAP_X_MIN",
		"MAP_X_SIZE",
		"MAP_Z_MAX",
		"MAP_Z_MIN",
		"MAP_Z_SIZE",
		"MAX_PLAYERS",
		"MEDAL",
		"MEMBER",
		"MILITARY_SITE",
		"MINUTES",
		"MOBILE_KEYSITE",
		"MOVEMENT",
		"NAME",
		"OBJECTIVES",
		"OBJECT_NUMBER",
		"OBJECT_SCALING",
		"OIL_RIG_SITE",
		"OPTION_INFINITE_FUEL",
		"OPTION_INFINITE_WEAPONS",
		"OPTION_INVULNERABLE",
		"OPTION_SUPPRESS_AI_FIRE",
		"PARTIAL",
		"PILOT",
		"PILOT_EXPERIENCE",
		"PILOT_LEVEL",
		"PILOT_RANK",
		"PLANNER_DATA",
		"PLAY_SOUND_EFFECT",
		"PLAYER",
		"POPULATION_DATA",
		"POPULATION_TYPE",
		"POSITION",
		"POWER_STATION_SITE",
		"PROMOTION",
		"RADIUS",
		"RANK",
		"RANK_POINTS",
		"REGEN",
		"REGEN_FREQUENCY",
		"RESULT",
		"ROAD_NODE_ROUTE_DATA",
		"ROUTE",
		"SAM_AAA_SITE",
		"SAVED_CAMPAIGN",
		"SCENIC_OBJECT",
		"SEA_NODE_ROUTE_DATA",
		"SECONDS",
		"SET_DATE",
		"SET_ENTITY_POSITION",
		"SET_EVENT_TRIGGERED",
		"SET_INFINITE_AMMO",
		"SET_INFINITE_FUEL",
		"SET_INFINITE_WEAPONS",
		"SET_INVULNERABLE",
		"SET_TIME",
		"SET_WEATHER",
		"SHORT_TEXT_START",
		"SIDE",
		"SIZE",
		"START_BASE",
		"SUCCESS",
		"SUPPRESS_POPULATION_KEYSITES",
		"TASK_GENERATION",
		"TERRAIN_OBJECT",
		"TEXT1",
		"TEXT2",
		"TEXT3",
		"TEXT4",
		"TEXT_END",
		"TITLE",
		"TOUR_OF_DUTY",
		"TRAFFIC_ROUTE",
		"TRANSLATION",
		"TRANSLATION_PARAGRAPH",
		"VELOCITY",
		"VERSION_NUMBER",
		"WARZONE_NAME",
		"WARZONE_SIZE",
		"WEATHER_RAIN",
		"WEATHER_WIND",
		"X_SECTOR",
		"Z_SECTOR",
		"ZOOM",
	//VJ 051202 add season file tag for winter/summer camo		
		"SEASON",
		"APPLICATION_LAST_TAG"
   };
/*




void initialise_file_tag_system (void)
{

	file_tag_variable_type
		*destroy_file_tag_variable;

	while (file_tag_variables)
	{

		destroy_file_tag_variable = file_tag_variables;

		file_tag_variables = file_tag_variables->next;

		if (destroy_file_tag_variable->name)
		{

			free (destroy_file_tag_variable->name);
		}

		free (destroy_file_tag_variable);
	}

	file_tag_variables = NULL;
}





void deinitialise_file_tag_system (void)
{

	initialise_file_tag_system ();
}




// SET FUNCTIONS




void set_file_tag (FILE *file_ptr, file_tags tag)
{

   //debug_assert (file_ptr);

   fprintf (file_ptr, "\n:%s ", tag_strings [tag]);
}





void set_file_int (FILE *file_ptr, int value)
{

   //debug_assert (file_ptr);

   fprintf (file_ptr, "%d ", value);
}





void set_file_float (FILE *file_ptr, float value)
{

   //debug_assert (file_ptr);

   fprintf (file_ptr, "%.02f ", value);
}





void set_file_string (FILE *file_ptr, char *string)
{

   //debug_assert (file_ptr);

   fprintf (file_ptr, "%s\0 ", string);
}





void set_file_enum (FILE *file_ptr, char *enum_strings [], int enum_value)
{

   //debug_assert (file_ptr);

   fprintf (file_ptr, "%s ", enum_strings [enum_value]);
}





void set_file_new_line (FILE *file_ptr, int count)
{

   //debug_assert (file_ptr);

	while (count)
	{

		fprintf (file_ptr, "\n");

		count --;
	}
}





void set_file_comment (FILE *file_ptr, char *string)
{

   //debug_assert (file_ptr);

	fprintf (file_ptr, "\n//%s", string);
}




// GET FUNCTIONS




file_tags get_file_tag (char *line)
{

   file_tags
      current_tag = FILE_TAG_UNKNOWN;

   if (*line == ':')
   {

      line ++;

      current_tag = FILE_TAG_INVALID;

      while (current_tag != FILE_TAG_UNKNOWN)
      {

         if (strcmpi (line, tag_strings [current_tag]) == 0)
         {

            break;
         }

         current_tag ++;
      }
   }








   return current_tag;
}





file_tags get_next_file_tag (FILE *file_ptr)
{

   int
		c_count,
      last_c,
      c;

   char
      line [260];

   file_tags
      current_tag = FILE_TAG_UNKNOWN;

   memset (line, 0, 260);

   //
   // Search for next :
   //

   while ((c = fgetc (file_ptr)) != EOF)
   {

		//
		// skip lines starting with comments '//'
		//

      if ((c == '/') && (last_c == '/'))
      {

         fgets (line, 256, file_ptr);

			memset (line, 0, 260);
      }
      else if (c == ':')
      {

         // read off :

         c = fgetc (file_ptr);

         break;
      }

      last_c = c;
   }

   //
   // Read in next tag
   //

	c_count = 0;

   while ((c != EOF) && (c != '\n') && (c != ' ') && (c != '='))
   {

		line [c_count] = c;

		c_count ++;

		if (c_count > sizeof (line))
		{

			debug_fatal ("FILE_TAG: tag too long %s", line);
		}

      c = fgetc (file_ptr);
   }

   //
   // Compare tag with enums
   //







   if (c != EOF)
   {

      current_tag = FILE_TAG_INVALID;

      while (current_tag != FILE_TAG_UNKNOWN)
      {

         if (strcmpi (line, tag_strings [current_tag]) == 0)
         {

            break;
         }

         current_tag ++;
      }
   }
	else
	{

		current_tag = FILE_TAG_END;
	}








   return current_tag;
}





file_tags check_next_file_tag (FILE *file_ptr)
{

   int
		c_count,
      last_c,
      c;

   char
      line [260];

   file_tags
      current_tag = FILE_TAG_UNKNOWN;

   memset (line, 0, 260);

   //
   // Search for next :
   //

   while ((c = fgetc (file_ptr)) != EOF)
   {

		//
		// skip lines starting with comments '//'
		//

      if ((c == '/') && (last_c == '/'))
      {

         fgets (line, 256, file_ptr);

			memset (line, 0, 260);
      }
      else if (c == ':')
      {

         // read off :

         c = fgetc (file_ptr);

         break;
      }

      last_c = c;
   }

   //
   // Read in next tag
   //

	c_count = 0;

   while ((c != EOF) && (c != '\n') && (c != ' ') && (c != '='))
   {

		line [c_count] = c;

		c_count ++;

		if (c_count > sizeof (line))
		{

			debug_fatal ("FILE_TAG: check tag too long");
		}

      c = fgetc (file_ptr);
   }

   //
   // Compare tag with enums
   //

   if (c != EOF)
   {

      current_tag = FILE_TAG_INVALID;

      while (current_tag != FILE_TAG_UNKNOWN)
      {

         if (strcmpi (line, tag_strings [current_tag]) == 0)
         {

            break;
         }

         current_tag ++;
      }

      fseek (file_ptr, -(strlen (line) + 3), SEEK_CUR);
   }








   return current_tag;
}





int get_next_file_int (FILE *file_ptr)
{

	int
		c,
		c_count,
		number;

	char
		text [256];

	memset (text, 0, sizeof (text));

	c_count = 0;

	c = fgetc (file_ptr);

	while ((c != '.') && (c != ',') && (c != '\n') && (c != ' '))
	{

		text [c_count] = c;

		c_count ++;

		if (c_count > sizeof (text))
		{

			debug_fatal ("FILE_TAG: get next int too long");
		}

		c = fgetc (file_ptr);
	}

	number = atoi (text);







	return number;
}





float get_next_file_float (FILE *file_ptr)
{

	double
		digit,
		result;

	int
		c,
		sign = 1,
		level;

	//
	// skim off crap
	//

	c = fgetc (file_ptr);

	while ((!isdigit (c)) && (c != '-'))
	{

		c = fgetc (file_ptr);
	}

	//
	// interger part
	//

	if (c == '-')
	{

		sign = -1;

		c = fgetc (file_ptr);
	}

	result = 0.0;

	while ((c != '.') && (c != ',') && (c != '\n') && (c != ' '))
	{

		result *= 10.0;

		result += (float) (c - '0');

		c = fgetc (file_ptr);
	}

	//
	// factional part
	//

	if (c == '.')
	{

		level = 1;

		c = fgetc (file_ptr);
	
		while ((c != ',') && (c != '\n') && (c != ' '))
		{

			level *= 10;
	
			digit = (float) (c - '0');

			digit /= level;

			result += digit;
	
			c = fgetc (file_ptr);
		}
	}







	return (result * sign);
}





int get_next_file_string (FILE *file_ptr, char *buffer, int size)
{

	char
		*ptr;

	memset (buffer, 0, size);

	if (fgets (buffer, size - 1, file_ptr))
	{
	
		ptr = strchr (buffer, '\n');
	
		*ptr = '\0';






	
		return TRUE;
	}
	else
	{

		return FALSE;
	}
}





int get_next_file_word (FILE *file_ptr, char *buffer, int size)
{

	int
		character,
		count;

	memset (buffer, 0, size);

	count = 0;

	while ((character = fgetc (file_ptr)) != EOF)
	{

		if ((character == ' ') || (character == ',') || (character == '\n'))
		{

			break;
		}

		buffer[count++] = character;
	}







	return TRUE;
}





int get_next_file_enum (FILE *file_ptr, char *enum_strings [], int last_enum)
{

	int
		c,
		c_count,
		tag = 0;

	char
		buffer [256];

	memset (buffer, 0, 256);

   //
   // Read in line
   //

	c = fgetc (file_ptr);

	c_count = 0;

   while ((c != EOF) && (c != '\n') && (c != ' ') && (c != ','))
   {

		buffer [c_count] = c;

		c_count ++;

		if (c_count > sizeof (buffer))
		{

			debug_fatal ("FILE_TAG: get next enum too long");
		}

      c = fgetc (file_ptr);
   }

	if (c_count > 0)
	{
	
  		while (tag <= last_enum)
		{

         if (strcmpi (buffer, enum_strings [tag]) == 0)
         {

            break;
         }

			tag ++;
		}
	}

	if (tag > last_enum)
	{

		debug_fatal ("FILE_TAG: unrecognised tag %s", buffer);
	}







	return tag;
}





void register_file_tag_variable (char *variable_name, int *variable)
{

	int
		next_id;

	file_tag_variable_type
		*new_variable;

	//
	// check if variable already registered
	//

	next_id = 0;

	new_variable = file_tag_variables;

	while (new_variable)
	{

		if (strcmpi (new_variable->name, variable_name) == 0)
		{

			return;
		}

		next_id ++;

		new_variable = new_variable->next;
	}

	//
	// Register new variable
	//

	new_variable = (file_tag_variable_type *) malloc (sizeof (file_tag_variable_type));

	memset (new_variable, 0, sizeof (file_tag_variable_type));

	new_variable->name = (char *) malloc ((strlen (variable_name) + 1));

	sprintf (new_variable->name, "%s", variable_name);

	new_variable->variable = variable;

	new_variable->id = next_id;

	new_variable->next = file_tag_variables;







	file_tag_variables = new_variable;
}





void unregister_file_tag_variable (char *variable_name)
{

	file_tag_variable_type
		*variable,
		*last_variable;

	//
	// check if variable already registered
	//

	last_variable = NULL;

	variable = file_tag_variables;

	while (variable)
	{

		if (strcmpi (variable->name, variable_name) == 0)
		{

			break;
		}

		last_variable = variable;

		variable = variable->next;
	}

	if (variable)
	{
	
		if (last_variable)
		{
	
			last_variable->next = variable->next;
		}
		else
		{
	
			file_tag_variables = variable->next;
		}






	
		if (variable->name)
		{
	
			free (variable->name);
		}

		free (variable);
	}
}





int *get_file_tag_variable (char *variable_name)
{

	file_tag_variable_type
		*variable;

	variable = file_tag_variables;

	while (variable)
	{

		if (strcmpi (variable_name, variable->name) == 0)
		{







			return variable->variable;
		}

		variable = variable->next;
	}







	return NULL;
}





int if_file_tag_variable (char *variable_name, char *operator, int value)
{

	int
		return_value,
		*variable;







	return_value = FALSE;

	switch (*operator)
	{

		case '=':
		{

			variable = get_file_tag_variable (variable_name);

			if (variable)
			{
	
				if (*variable == value)
				{
	
					return_value = TRUE;
				}
			}

			break;
		}

		case '<':
		{

			variable = get_file_tag_variable (variable_name);

			if (variable)
			{
	
				if (*variable < value)
				{
	
					return_value = TRUE;
				}
			}

			break;
		}

		case '>':
		{

			variable = get_file_tag_variable (variable_name);

			if (variable)
			{
	
				if (*variable > value)
				{
	
					return_value = TRUE;
				}
			}

			break;
		}
	}

	return return_value;
}





int if_file_tag_operator (int value1, char *operator, int value2)
{

	int
		return_value;
		//*variable;







	return_value = FALSE;

	switch (*operator)
	{

		case '=':
		{

			if (value1 == value2)
			{

				return_value = TRUE;
			}

			break;
		}

		case '<':
		{

			if (value1 < value2)
			{

				return_value = TRUE;
			}

			break;
		}

		case '>':
		{

			if (value1 > value2)
			{

				return_value = TRUE;
			}

			break;
		}

		default:
		{

			char
				log_string [256];

			sprintf (log_string, "FILE_TAG: unknown operator %c", operator [0]);
			debug_fatal (log_string);
		}
	}

	return return_value;
}





int set_file_tag_variable (char *variable_name, int value)
{

	int
		*variable;

	variable = get_file_tag_variable (variable_name);

	if (variable)
	{







		*variable = value;

		return TRUE;
	}

	return FALSE;
}





int get_variable_id (char *variable_name)
{

	file_tag_variable_type
		*variable;

	variable = file_tag_variables;

	while (variable)
	{

		if (strcmpi (variable_name, variable->name) == 0)
		{







			return variable->id;
		}

		variable = variable->next;
	}

	return -1;
}





char *get_variable_name (int id)
{

	file_tag_variable_type
		*variable;

	variable = file_tag_variables;

	while (variable)
	{

		if (variable->id == id)
		{







			return variable->name;
		}

		variable = variable->next;
	}

	return NULL;
}





int get_operator_id (char *name)
{







	switch (*name)
	{

		case '=':
		{

			return FILE_TAG_EQUALS;
		}

		case '<':
		{
			return FILE_TAG_LESS_THAN;
		}

		case '>':
		{

			return FILE_TAG_GREATER_THAN;
		}
	}

	return -1;
}





char get_operator_name (int id)
{







	switch (id)
	{

		case FILE_TAG_EQUALS:
		{

			return '=';
		}

		case FILE_TAG_LESS_THAN:
		{

			return '<';
		}

		case FILE_TAG_GREATER_THAN:
		{

			return '>';
		}
	}

	return NULL;
}




*/
