

































































#define AIRCRAFT_LOADING_DOORS_CLOSED_FLOAT_VALUE 							0.0

#define AIRCRAFT_LOADING_DOORS_OPEN_FLOAT_VALUE								2.0



extern void animate_aircraft_loading_doors( entity *en );

extern void update_aircraft_loading_doors( entity *en );





#define AIRCRAFT_CARGO_DOORS_CLOSED_FLOAT_VALUE 							0.0

#define AIRCRAFT_CARGO_DOORS_OPEN_FLOAT_VALUE								2.0



extern void animate_aircraft_cargo_doors( entity *en );

extern void update_aircraft_cargo_doors( entity *en );





extern void animate_aircraft_rudder( entity *en );

extern void update_aircraft_rudder( entity *en );





#define AIRCRAFT_UNDERCARRIAGE_UP			0
#define AIRCRAFT_UNDERCARRIAGE_LOWERING	1
#define AIRCRAFT_UNDERCARRIAGE_DOWN			2
#define AIRCRAFT_UNDERCARRIAGE_RAISING		3



#define AIRCRAFT_UNDERCARRIAGE_UP_FLOAT_VALUE 							0.0

#define AIRCRAFT_UNDERCARRIAGE_DOWN_FLOAT_VALUE							2.0



extern void animate_aircraft_undercarriage( entity *en );

extern void update_aircraft_undercarriage( entity *en );





#define AIRCRAFT_WEAPON_SYSTEM_READY_CLOSED_FLOAT_VALUE 				0.0

#define AIRCRAFT_WEAPON_SYSTEM_READY_OPEN_FLOAT_VALUE					2.0



extern void animate_aircraft_weapon_system_ready( entity *en );

extern void update_aircraft_weapon_system_ready( entity *en );





extern void animate_aircraft_shadow (entity *en);





extern void initiate_aircraft_crew_ejection (entity *en);

extern void uneject_aircraft_crew (entity *en);




