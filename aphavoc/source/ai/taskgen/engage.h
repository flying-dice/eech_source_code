

































































extern entity *create_engage_task (entity *group, entity *objective, entity *originator, int expire);

extern int engage_targets_in_sector (entity *group, int sx, int sz, unsigned int task_target_type, int expire);

extern int engage_targets_in_area (entity *group, vec3d *target_point, float radius, unsigned int task_target_type, int expire);



extern int engage_targets_in_group (entity *group, entity *target_group, int expire);

extern int engage_specific_target (entity *group, entity *target, unsigned int valid_members, int expire);

extern void engage_specific_targets(entity *wingman, entity *targets[]);

extern unsigned int assign_engage_tasks_to_group (entity *group, unsigned int valid_members);

extern void terminate_all_engage_tasks (entity *group);

extern void terminate_entity_current_engage_task (entity *en);






