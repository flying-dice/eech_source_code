



























































#ifndef _AI_DBASE_H
#define _AI_DBASE_H




//
// TOWN / KEYSITE NAMES
//




enum POPULATION_NAME_TYPES
{
	POPULATION_TYPE_KEYSITE,
	POPULATION_TYPE_TOWN,

	NUM_POPULATION_NAME_TYPES
};

typedef enum POPULATION_NAME_TYPES population_name_types;





struct POPULATION_NAME_DATABASE_TYPE
{

	char
		*name;

	population_name_types
		type;

	float
		x,
		z,
		zoom;

	struct POPULATION_NAME_DATABASE_TYPE
		*next;
};

typedef struct POPULATION_NAME_DATABASE_TYPE population_name_database_type;

extern population_name_database_type
	*population_name_database;





extern void initialise_ai_database (void);

extern void deinitialise_ai_database (void);

extern void load_ai_sector_data (const char *filename);

extern void initialise_population_name_database (void);

extern void deinitialise_population_name_database (void);





#endif // _AI_DBASE_H
