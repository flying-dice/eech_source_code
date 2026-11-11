void get_identity_matrix3x3 (matrix3x3 m)
{
	m[0][0] = 1.0;
	m[0][1] = 0.0;
	m[0][2] = 0.0;

	m[1][0] = 0.0;
	m[1][1] = 1.0;
	m[1][2] = 0.0;

	m[2][0] = 0.0;
	m[2][1] = 0.0;
	m[2][2] = 1.0;
}
float normalise_any_3d_vector ( vec3d *vector )
{

	float
		x,
		y,
		z,
		length;

	ASSERT ( vector );

	x = ( ( vector->x ) * ( vector->x ) );
	y = ( ( vector->y ) * ( vector->y ) );
	z = ( ( vector->z ) * ( vector->z ) );

	length = sqrt ( x + y + z );

	if ( length > 0 )
	{

		float
			one_over_length;

		one_over_length = 1.0 / length;

		vector->x = ( vector->x * one_over_length );
		vector->y = ( vector->y * one_over_length );
		vector->z = ( vector->z * one_over_length );
	}
	else
	{

		vector->x = 0;
		vector->y = 0;
		vector->z = 0;
	}

	return ( length );
}
float normalise_3d_vector(vec3d *vector) { return normalise_any_3d_vector(vector); }
void get_3d_vector_cross_product (vec3d *result, const vec3d *v1, const vec3d *v2)
{
	ASSERT (result);

	ASSERT (v1);

	ASSERT (v2);

	result->x = (v1->y * v2->z) - (v2->y * v1->z);
	result->y = (v1->z * v2->x) - (v2->z * v1->x);
	result->z = (v1->x * v2->y) - (v2->x * v1->y);
}
void get_3d_transformation_matrix_from_face_normal_and_heading (matrix3x3 result, const vec3d *normal, float heading)
{
	//
	// guard against possible divide by zero
	//

	if (fabs (normal->y) > 0.0001)
	{
		//
		// set y vector to face normal
		//

		memcpy (&result[1][0], normal, sizeof (vec3d));

		// set z vector perpendicular to face normal in the heading direction
		//
		// use 'planes through the origin' to get y
		//
		// r.n = 0, where r = (x y z) and n = (a b c), then ax + by + cz = 0, therefore y = - (ax + cz) / b

		result[2][0] = sin (heading);
		result[2][2] = cos (heading);
		result[2][1] = - ((normal->x * result[2][0]) + (normal->z * result[2][2])) / normal->y;

		normalise_3d_vector ((vec3d *) &result[2][0]);

		//
		// get x vector (y X z)
		//

		get_3d_vector_cross_product ((vec3d *) &result[0][0], (vec3d *) &result[1][0], (vec3d *) &result[2][0]);

		//
		// (no need to normalise x vector because y and z vectors are perpendicular unit vectors)
		//
	}
	else
	{
		get_identity_matrix3x3 (result);
	}
}
