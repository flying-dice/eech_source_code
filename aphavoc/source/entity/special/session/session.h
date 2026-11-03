

































































struct SESSION
{

	//
	// ALWAYS KEEP THIS FIRST
	//

	int
		version_number;

	//


	list_root
		force_root,
		special_effect_root;

	list_link
		update_link;

	float
		start_time,
		elapsed_time,
		lightning_timer,
		time_of_day_resync,
		time_of_day_acceleration,
		fog_of_war_maximum_value;

	//
	// rain effect
	//

	float
		weather_radius,								// used for local weather model only
		weather_mode_transitional_period,		// used for global weather model only
		weather_mode_transitional_status;

	weathermodes
		weather_mode,
		target_weather_mode;

	vec3d
		weather_position,								// used for local weather model only
		weather_velocity;								// used for local weather model only

	//
	// wind effect
	//

	float
		wind_effect_radius,							// used for local weather model only
		wind_gusting_value,
		wind_minimum_speed,
		wind_maximum_speed;

	vec3d
		wind_direction_vector,
		wind_effect_position,						// used for local weather model only
		wind_effect_velocity;						// used for local weather model only

	//

	day_segment_types
		day_segment_type;

	float
		population_x_min, 							// population size of map (for Reduced games like Skirmishs)
		population_x_max,
		population_z_min,
		population_z_max;

	unsigned int
		campaign_medal								: NUM_CAMPAIGN_MEDAL_BITS,
		campaign_requires_apache_havoc		: NUM_CAMPAIGN_REQUIRES_APACHE_HAVOC_BITS,
		auto_assign_gunship						: NUM_AUTO_ASSIGN_GUNSHIP_BITS,
		session_complete							: NUM_SESSION_COMPLETE_BITS,
		infinite_fuel								: NUM_INFINITE_FUEL_BITS,
		infinite_weapons							: NUM_INFINITE_WEAPONS_BITS,
		suppress_ai_fire							: NUM_SUPPRESS_AI_FIRE_BITS,
		invulnerable_from_collisions			: NUM_INVULNERABLE_BITS,
		invulnerable_from_weapons				: NUM_INVULNERABLE_BITS,
		cheats_enabled								: NUM_CHEATS_ENABLED_BITS,
		skip_night_time							: NUM_SKIP_NIGHT_TIME_BITS,
		weather_increasing						: NUM_WEATHER_INCREASING_BITS,
		wind_increasing							: NUM_WEATHER_INCREASING_BITS,
		local_weather_model						: NUM_LOCAL_WEATHER_MODEL_BITS;
};

typedef struct SESSION session;





extern entity
	*session_entity;

#define get_session_entity() (session_entity)





#define MIN_WEATHER_RADIUS	5000.0
#define MAX_WEATHER_RADIUS	100000.0
#define WEATHER_EXPANSION_RATE	100.0

#define WEATHER_EFFECT_SPEED			knots_to_metres_per_second (360.0)

#define MIN_WIND_RADIUS	8000.0
#define MAX_WIND_RADIUS	128000.0
#define WIND_EXPANSION_RATE	200.0

#define WIND_EFFECT_SPEED				knots_to_metres_per_second (400.0)

#define WIND_GUST_FREQUENCY	0.1

// magnitude of 0 to 15 knots works well with smoke effects
#define DEFAULT_MIN_WIND_SPEED		knots_to_metres_per_second (5.0)
#define DEFAULT_MAX_WIND_SPEED		knots_to_metres_per_second (25.0)

#define SESSION_RESYNC_FREQUENCY		ONE_MINUTE





extern float
	weather_rain_radius_min,
	weather_rain_radius_max,
	weather_rain_speed,
	weather_wind_radius_min,
	weather_wind_radius_max,
	weather_wind_speed;





extern void set_session_entity (entity *en);

extern float calculate_session_time_of_day (entity *en, int *day);

extern day_segment_types get_day_segment_type (float time_of_day);

extern float get_segment_time_of_day (day_segment_types segment);

extern float get_session_random_start_time_of_day (void);

extern float get_session_preset_time_of_day (session_time_of_day_settings preset_enum);





extern void set_session_fixed_weather_mode (entity *en, weathermodes weather_mode);

extern void set_session_random_weather_mode (entity *en);





extern int check_point_inside_population_area (vec3d *point);




























