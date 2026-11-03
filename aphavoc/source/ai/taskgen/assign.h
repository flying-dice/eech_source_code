

































































extern void assign_keysite_tasks (entity *keysite, task_category_types category);

extern entity *get_suitable_registered_group (entity *task, int *idle_group_count);

extern int assign_primary_task_to_group (entity *group_en, entity *task_en);

extern entity *push_task_onto_group_task_stack (entity *group, entity *task, unsigned int valid_members);

extern int assign_task_to_group (entity *group_en, entity *task_en, unsigned int valid_members);

extern int assign_task_to_group_members (entity *group, entity *guide, unsigned int valid_members);

extern int assign_new_task_to_group_member (entity *group, entity *member, entity *task, entity *guide);

extern int reassign_group_members_to_valid_tasks (entity *group, entity *last_task, unsigned int members_to_reassign, int engage_enemy);

extern void respond_to_player_task_assign_request (entity *pilot, entity *task, entity *mobile);

extern int check_group_members_awake (entity *group);

extern void prepare_helicopter_for_task(entity *en, entity *task, entity *group);

extern void set_helicopter_fuel_level(entity *en, entity *task);




