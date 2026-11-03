















































































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






















extern void process_game_initialisation_phases (void);





