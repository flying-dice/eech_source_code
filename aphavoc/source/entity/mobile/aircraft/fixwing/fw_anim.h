

































































extern void set_client_server_fixed_wing_afterburner_state (entity *en, int state);

extern void set_local_fixed_wing_afterburner_state (entity *en, int state);

extern void animate_fixed_wing_afterburners (entity *en);

extern void update_fixed_wing_afterburners (entity *en);





#define FIXED_WING_AIRBRAKES_CLOSED_FLOAT_VALUE 						0.0

#define FIXED_WING_AIRBRAKES_OPEN_FLOAT_VALUE							2.0

extern void animate_fixed_wing_airbrakes( entity *en );

extern void update_fixed_wing_airbrakes( entity *en );





#define FIXED_WING_FLAPS_CLOSED_FLOAT_VALUE 						0.0

#define FIXED_WING_FLAPS_OPEN_FLOAT_VALUE							2.0

extern void animate_fixed_wing_flaps( entity *en );

extern void update_fixed_wing_flaps( entity *en );



extern void animate_fixed_wing_propellors( entity *en );

extern void initialise_fixed_wing_propellors (entity *en);

extern void damage_fixed_wing_3d_object (entity *en);

extern void set_fixed_wing_id_number (entity *en);




