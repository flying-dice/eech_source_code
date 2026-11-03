

































































struct COMPRESSED_WAYPOINT_TYPE
{

	int
		waypoint_type,
		formation_type,
		criteria,
		radius,
		velocity;

	vec3d
		position;
};

typedef struct COMPRESSED_WAYPOINT_TYPE compressed_waypoint_type;





struct PRE_DESIGNED_ROUTES
{

	int
		waypoint_count;

	compressed_waypoint_type
		route [];
};

typedef struct PRE_DESIGNED_ROUTES pre_designed_routes;




