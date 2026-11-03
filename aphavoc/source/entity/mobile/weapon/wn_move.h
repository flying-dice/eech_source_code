

































































extern int
	draw_weapon_terrain_los_markers;





extern void weapon_movement (entity *en);

extern void generate_ballistics_tables(void);
extern void delete_ballistics_tables(void);
extern void generate_guided_missiles_tables(void);
extern float get_projectile_flight_time_fast(entity_sub_types wpn_type, float range, float height_difference);

/** 
 * @author arneh
 *  Finds ballistics drop for projectile
 *  Can be used either if you have a target and want to know where to aim, or you fire 
 *  in a certain direction and want to know how much it has dropped by range.
 * 
 * @param wpn_type   The projectile type to get drop ballistics value for
 * @param range      Range to calculate ballistics for
 * @height_difference_or_pitch   If firing at a specific point give the height difference
 *                               between shooter and target in this variable (positive values means shoorter is above target)
 *                               If firing at a fixed pitch provide the pitch in this var.  Which is used will be determined 
 *                               by the fixed_pitch parameter.
 * @aiming_pitch     Resulting pitch to aim for (or invert to get drop angle)
 * @time_of_flight   Result for how long the projectile will take to reach range
 * @simplified       If true will use simpler calculations.  Less accurate, but faster.  Mostly for AI use
 * @use_fixed_pitch  Use fixed pitch, not fixed target.
 * @return           TRUE if able to calculate ballistics.  FALSE otherwise (usually because target is outside maximum range)
 */
extern int get_ballistic_pitch_deflection(entity_sub_types wpn_type, float range, float height_diff_or_pitch, float* aiming_pitch, float* time_of_flight, int simplified, int fixed_pitch, float parent_velocity);




