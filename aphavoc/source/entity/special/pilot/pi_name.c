

































































#include "project.h"





#define DEBUG_MODULE 0





//
// temporary implementation for now...
// will probably be read in from a file eventually
//

static const char
	*blue_side_first_name_list [] =
	{
		"Alex",
		"Andrew",
		"Ben",
		"Cameron",
		"Douglas",
		"Frank",
		"Harry",
		"James",
		"John",
		"Leonard",
		"Peter",
		"Robert",
		"Warren",
	};

#define NUM_BLUE_SIDE_FIRST_NAMES 13

static const char
	*blue_side_last_name_list [] =
	{
		"Cole",
		"Connor",
		"Dyson",
		"Fredrickson",
		"Fry",
		"Harper",
		"Hartman",
		"Lawrence",
		"Lewis",
		"Matrix",
		"McClane",
		"Morten",
		"Murphy",
		"Quaid",
		"Reid",
		"Richards",
		"Ripley",
		"Schaefer",
		"Tasker",
		"Venkman",
	};

#define NUM_BLUE_SIDE_LAST_NAMES 20



static const char
	*red_side_first_name_list [] =
	{
		"Anthony",
		"Carter",
		"Clarence",
		"Damon",
		"Hans",
		"Ivan",
		"Joe",
		"Leon",
		"Marcellus",
		"Richard",
		"Ronald",
		"Vincent",
	};

#define NUM_RED_SIDE_FIRST_NAMES 12

static const char
	*red_side_last_name_list [] =
	{
		"Bodicker",
		"Burke",
		"Cox",
		"Drago",
		"Emile",
		"Gruber",
		"Jones",
		"Killian",
		"Mihn",
		"Miller",
		"Nash",
		"Rooney",
		"Vega",
		"Wallace",
	};

#define NUM_RED_SIDE_LAST_NAMES 14





void create_new_pilot_name( entity_sides side, const char **firstname, const char **lastname )
{

	switch( side )
	{
		case ENTITY_SIDE_NEUTRAL:
		case ENTITY_SIDE_BLUE_FORCE:
		{

			*firstname = blue_side_first_name_list[ rand16() % NUM_BLUE_SIDE_FIRST_NAMES ];

			*lastname = blue_side_last_name_list[ rand16() % NUM_BLUE_SIDE_LAST_NAMES ];

			break;
		}
		case ENTITY_SIDE_RED_FORCE:
		{

			*firstname = red_side_first_name_list[ rand16() % NUM_RED_SIDE_FIRST_NAMES ];

			*lastname = red_side_last_name_list[ rand16() % NUM_RED_SIDE_LAST_NAMES ];

			break;
		}
		default:
		{

			debug_fatal( "Trying to create pilot for side %d", side );
		}
	}
}






