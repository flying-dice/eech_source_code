

































































#define MAX_NUMBER_OF_MISSION_TEXTS 	30





struct TASK_BRIEFING_TYPE
{
	entity_sub_types
		task_type;

	int
		briefing_text1_count,
		briefing_text2_count,
		briefing_text3_count,
		debriefing_text_success_count,
		debriefing_text_partial_count,
		debriefing_text_failure_count;

	char
		*briefing_text1 [MAX_NUMBER_OF_MISSION_TEXTS],
		*briefing_text2 [MAX_NUMBER_OF_MISSION_TEXTS],
		*briefing_text3 [MAX_NUMBER_OF_MISSION_TEXTS],
		*debriefing_text_success [MAX_NUMBER_OF_MISSION_TEXTS],
		*debriefing_text_partial [MAX_NUMBER_OF_MISSION_TEXTS],
		*debriefing_text_failure [MAX_NUMBER_OF_MISSION_TEXTS];
};

typedef struct TASK_BRIEFING_TYPE task_briefing_type;





extern task_briefing_type
	*task_briefing_database;





enum EXTRA_BRIEFING_TYPES
{

	EXTRA_BRIEFING_MEDAL,
	EXTRA_BRIEFING_PROMOTION,
	EXTRA_BRIEFING_OBJECTIVES,
	NUM_EXTRA_BRIEFING_TYPES
};

typedef enum EXTRA_BRIEFING_TYPES extra_briefing_types;





struct EXTRA_BRIEFING_TYPE
{

	extra_briefing_types
		type;

	int
		sub_type;

	int
		text_count;

	char
		*briefing_text [MAX_NUMBER_OF_MISSION_TEXTS];

	struct EXTRA_BRIEFING_TYPE
		*next;
};

typedef struct EXTRA_BRIEFING_TYPE extra_briefing_type;





extern extra_briefing_type
	*extra_briefing_database;





extern void initialise_briefing_parser (void);

extern void deinitialise_briefing_parser (void);

extern int get_briefing_text (entity *task, char *text1, char *text2, char *text3);

extern int get_debriefing_text (entity *task, char **debrief, char **ff_debrief);

extern int get_medal_briefing_text (entity *task, int medal_type, char *medal_text);

extern int get_promotion_briefing_text (entity *task, int rank_type, char *rank_text);

extern int get_objective_briefing_text (entity *task, int objective_type, char *objective_text);




