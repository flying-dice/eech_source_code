




























































#ifndef _AI_MISC_H
#define _AI_MISC_H





#include "ai_route.h"

#include "ai_dbase.h"





extern entity *get_closest_waypoint (entity *task, vec3d *pos, float min_range, float *actual_range);

extern entity *get_closest_halfway_waypoint (entity *task, vec3d *pos, float min_range, float *actual_range);

extern entity *get_closest_free_landing_site (entity_sub_types sub_type, entity *mission, vec3d *pos, float min_range, float *actual_range, int sites_required);

extern entity *get_closest_mobile_entity (vec3d *pos);

extern int get_closest_road_node (vec3d *pos, float error);

extern int get_closest_side_road_node (entity_sides side, vec3d *pos, float error);

extern int validate_landing_route (entity *route_task);





#endif // _AI_MISC_H
