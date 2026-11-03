

































































































enum GAME_TYPES
{
	GAME_TYPE_INVALID,
	GAME_TYPE_FREE_FLIGHT,
	GAME_TYPE_CAMPAIGN,
	GAME_TYPE_SKIRMISH,
	GAME_TYPE_DEMO,
	NUM_GAME_TYPES
};

typedef enum GAME_TYPES game_types;




////Moje 030519 Added GUNSHIP_TYPE_BLACKHAWK, below
////Moje 030612 Added GUNSHIP_TYPE_HIND, below
////Moje 030816 Added GUNSHIP_TYPE_AH64A, and GUNSHIP_TYPE_KA50, below

enum GUNSHIP_TYPES
{
	GUNSHIP_TYPE_APACHE,
	GUNSHIP_TYPE_HAVOC,
	GUNSHIP_TYPE_COMANCHE,
	GUNSHIP_TYPE_HOKUM,
	GUNSHIP_TYPE_BLACKHAWK,
	GUNSHIP_TYPE_HIND,
	GUNSHIP_TYPE_AH64A,
 	GUNSHIP_TYPE_KA50,
 	GUNSHIP_TYPE_VIPER,
 	GUNSHIP_TYPE_KIOWA,
	NUM_GUNSHIP_TYPES
};

typedef enum GUNSHIP_TYPES gunship_types;

extern gunship_types alphabetical_gunship_types[];





extern const char
	*game_status_names [],
	*gunship_type_names [],
	*long_gunship_names[],
	new_gunship[];





#include "entity/system/en_types/en_sbtyp.h"
#include "entity/system/en_types/en_types.h"

extern entity_sub_types
	gunship_sub_types [];
extern entity_sides
	gunship_sides [];

























































































































































enum SESSION_TIME_OF_DAY_SETTINGS
{

	SESSION_TIME_INVALID,
	SESSION_TIME_RANDOM,
	SESSION_TIME_DAWN,
	SESSION_TIME_MIDDAY,
	SESSION_TIME_AFTERNOON,
	SESSION_TIME_DUSK,
	SESSION_TIME_MIDNIGHT,
};

typedef enum SESSION_TIME_OF_DAY_SETTINGS session_time_of_day_settings;





enum SESSION_WEATHER_SETTINGS
{

	SESSION_WEATHER_INVALID,
	SESSION_WEATHER_RANDOM,
	SESSION_WEATHER_GOOD,
	SESSION_WEATHER_FAIR,
	SESSION_WEATHER_POOR,
};

typedef enum SESSION_WEATHER_SETTINGS session_weather_settings;




//VJ 051011 add season summer/winter/desert button
enum SESSION_SEASON_SETTINGS
{

	SESSION_SEASON_INVALID,
	SESSION_SEASON_DEFAULT,
	SESSION_SEASON_SUMMER,
	SESSION_SEASON_WINTER,
	SESSION_SEASON_DESERT,
};

typedef enum SESSION_SEASON_SETTINGS session_season_settings;





enum GAME_DIFFICULTY_SETTINGS
{

	GAME_DIFFICULTY_INVALID,
	GAME_DIFFICULTY_HARD,
	GAME_DIFFICULTY_MEDIUM,
	GAME_DIFFICULTY_EASY,
};

typedef enum GAME_DIFFICULTY_SETTINGS game_difficulty_settings;






























































































struct GLOBAL_OPTIONS_DATA
{




















	int
		gunship_side;

	gunship_types
		gunship_type;
























































































	//
	// VERSION 1.1
	//


	enum SESSION_TIME_OF_DAY_SETTINGS
		session_free_flight_time_of_day,
		session_campaign_time_of_day;

	enum SESSION_WEATHER_SETTINGS
		session_free_flight_weather,
		session_campaign_weather;

	enum GAME_DIFFICULTY_SETTINGS
		difficulty_level;

	unsigned int
		session_free_flight_realism_infinite_weapons					:1,
		session_free_flight_realism_infinite_fuel						:1,
		session_free_flight_realism_invulnerable_from_weapons		:1,
		session_free_flight_realism_invulnerable_from_collisions	:1,
		session_free_flight_realism_suppress_ai_fire					:1,

		session_campaign_realism_infinite_weapons						:1,
		session_campaign_realism_infinite_fuel							:1,
		session_campaign_realism_invulnerable_from_weapons			:1,
		session_campaign_realism_invulnerable_from_collisions		:1,
		session_campaign_realism_suppress_ai_fire						:1;
















































};

typedef struct GLOBAL_OPTIONS_DATA global_options_data;





extern global_options_data
	global_options;













#define get_global_gunship_side() ((entity_sides) global_options.gunship_side)

#define set_global_gunship_side(SIDE) (global_options.gunship_side = (SIDE))



#define get_global_gunship_type() (global_options.gunship_type)














































































































































































































































































































#define set_global_session_campaign_time_of_day(VALUE) (global_options.session_campaign_time_of_day = (VALUE))
#define get_global_session_campaign_time_of_day() (global_options.session_campaign_time_of_day)





#define set_global_session_campaign_weather(VALUE) (global_options.session_campaign_weather = (VALUE))
#define get_global_session_campaign_weather() (global_options.session_campaign_weather)


#define set_global_difficulty_level(VALUE) (global_options.difficulty_level = (VALUE))
#define get_global_difficulty_level() (global_options.difficulty_level)


















#define set_global_session_campaign_realism_infinite_weapons(VALUE) (global_options.session_campaign_realism_infinite_weapons = (VALUE))
#define get_global_session_campaign_realism_infinite_weapons() (global_options.session_campaign_realism_infinite_weapons)

#define set_global_session_campaign_realism_infinite_fuel(VALUE) (global_options.session_campaign_realism_infinite_fuel = (VALUE))
#define get_global_session_campaign_realism_infinite_fuel() (global_options.session_campaign_realism_infinite_fuel)

#define set_global_session_campaign_realism_invulnerable_from_weapons(VALUE) (global_options.session_campaign_realism_invulnerable_from_weapons = (VALUE))
#define get_global_session_campaign_realism_invulnerable_from_weapons() (global_options.session_campaign_realism_invulnerable_from_weapons)

#define set_global_session_campaign_realism_invulnerable_from_collisions(VALUE) (global_options.session_campaign_realism_invulnerable_from_collisions = (VALUE))
#define get_global_session_campaign_realism_invulnerable_from_collisions() (global_options.session_campaign_realism_invulnerable_from_collisions)

#define set_global_session_campaign_realism_suppress_ai_fire(VALUE) (global_options.session_campaign_realism_suppress_ai_fire = (VALUE))
#define get_global_session_campaign_realism_suppress_ai_fire() (global_options.session_campaign_realism_suppress_ai_fire)







































































