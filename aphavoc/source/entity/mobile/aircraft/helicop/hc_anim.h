

































































extern void animate_helicopter_controls ( entity *en );

extern void animate_helicopter_main_rotors (entity *en, int ignore_drawn_once);

extern void animate_helicopter_virtual_cockpit_main_rotors (entity *en, object_3d_instance *cockpit_main_rotor_inst3d, object_3d_instance *inst3d);

extern void animate_helicopter_tail_rotor (entity *en);

extern void animate_helicopter_wipers (entity *en);

extern void animate_helicopter_eo (entity *en);

extern void animate_helicopter_wheels (entity *en);

extern void damage_helicopter_3d_object (entity *en);

extern int damage_helicopter_main_rotors (entity *en, int blade_number);

extern void damage_helicopter_tail_rotors (entity *en);

extern int restore_helicopter_main_rotors (entity *en);

extern int restore_helicopter_main_rotor_inst3d (object_3d_instance *inst3d);

extern int restore_helicopter_tail_rotors (entity *en);

extern int get_helicopter_main_rotors_blurred (entity *en);

extern void set_rotation_angle_of_helicopter_main_rotors (object_3d_instance *inst3d, float theta);

extern float get_rotation_angle_of_helicopter_main_rotors (object_3d_instance *inst3d);

extern void set_rotation_angle_of_helicopter_tail_rotor (object_3d_instance *inst3d, float theta);

extern void set_helicopter_id_number (entity *en);




