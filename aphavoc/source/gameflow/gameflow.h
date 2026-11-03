

































































enum GAME_FLOW_TYPES
{
	GAME_FLOW_UNKNOWN,
	GAME_FLOW_INSTANT_ACTION,
	GAME_FLOW_COMBAT_MISSIONS,
	NUM_GAME_FLOW_TYPES
};

typedef enum GAME_FLOW_TYPES game_flow_types;





enum GAME_INITIALISATION_PHASES
{

	GAME_INITIALISATION_PHASE_NONE,
	GAME_INITIALISATION_PHASE_GAME_TYPE,
	GAME_INITIALISATION_PHASE_SCENARIO,
	GAME_INITIALISATION_PHASE_SETUP,
	GAME_INITIALISATION_PHASE_GUNSHIP_TYPE,
	GAME_INITIALISATION_PHASE_TRANSFERRING,
	NUM_GAME_INITIALISATION_PHASES
};

typedef enum GAME_INITIALISATION_PHASES game_initialisation_phases;

extern game_initialisation_phases
	game_initialisation_phase;





#define set_game_initialisation_phase(PHASE) (game_initialisation_phase = (PHASE))

#define get_game_initialisation_phase() (game_initialisation_phase)





extern void set_game_flow (game_flow_types flow);

extern game_flow_types get_game_flow (void);

extern void initialise_game_initialisation_phases (void);

extern void process_game_initialisation_phases (void);





