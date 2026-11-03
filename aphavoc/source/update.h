

































































struct UPDATE_FUNCTION_DATA_TYPE
{

	void
		((*function) (void));

	float
		sleep_time,
		last_update_timer;

	struct UPDATE_FUNCTION_DATA_TYPE
		*next;
};

typedef struct UPDATE_FUNCTION_DATA_TYPE update_function_data_type;





extern void initialise_update_function_list (void);

extern void deinitialise_update_function_list (void);

extern int add_update_function (void ((*function) (void)), float sleep_time, float offset_time);

extern int remove_update_function (void ((*function) (void)));

extern void update_update_functions (void);





