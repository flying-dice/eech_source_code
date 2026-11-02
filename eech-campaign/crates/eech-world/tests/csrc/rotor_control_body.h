/*
 * rotor_control_body.h -- the BODY of the original update_local_helicopter_rotor_sounds, for the TEST-ONLY numeric control (EEDCS-51, D3; ENV-HOST-ROTOR-CI-1). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED.
 *
 * WHAT IS IDENTICAL AND WHAT IS NOT. The statements are the original's from aphavoc/source/entity/mobile/aircraft/helicop/helicop.c (the #else body of the hosted build) at d2e67a2e2 / dd84bd748: the
 * declarations (with the unassigned locals, which the build's -ftrivial-auto-var-init=zero makes zero as for the original), the expressions with their C types and grouping, every assignment boundary and
 * every collaborator call. Against the original text this file differs ONLY by (1) the function name (CONTROL_NAME), (2) the removal of the ENV_FXR probe lines and (3) the added STAGE / STAGE_EXTRA marker lines;
 * with the markers empty (the UNMODIFIED variant, U) the arithmetic tokens are the original's. That is a statement about SOURCE TOKENS, not about generated code: the compilation context is a SURROGATE of the
 * engine's. The collaborators are the engine's macros in shape (get_local_entity_data is the same direct ((EN)->data); the int, child-list and float accessors are typed function-pointer-table dispatch macros
 * through a table indexed by the entity type, as in en_int.h, en_list.h and en_float.h; pause_local_entity_sound_type, resume_local_entity_sound_type and the vector magnitude are plain external functions of
 * another translation unit), but their bodies are stubs, the entity and dynamics layouts are reduced to the fields the routine reads, the validate_* range checks are omitted and the tables are filled at run time.
 * The assembly of U is therefore the compiler's treatment of the original arithmetic in a surrogate, never the engine's own assembly (the hosted build does not compile the original body at all).
 *
 * STAGED variant (S). STAGE (tag, variable) stores the variable into an addressable volatile float, reloads it into the variable and records the stored value, so the arithmetic that follows continues from the value
 * narrowed to single precision at that point; STAGE_EXTRA (tag, expression) records one EXTRA evaluation of an expression (the vector magnitude, tag 0) and is a second call of the function, not an observation of
 * the first. S is DIAGNOSTIC CHANGED CODE: it is never a replacement for the original or for the oracle.
 *
 * Included twice by rotor_control.c.

 */
void CONTROL_NAME (entity *en)
{
	helicopter
		*raw;

	float
		looping_amp,
		looping_pitch, //Werewolf pitch
		slap_amp,
		left_turbine_amp, right_turbine_amp, apu_amp,
		left_turbine_pitch = 1.0, right_turbine_pitch = 1.0, apu_pitch = 1.0,
		winding_amp,
//		main_rotor_pitch,
//		main_rotor_roll,
		acc_force;

	entity
		*spec;

	ASSERT (en);

	raw = (helicopter *) get_local_entity_data (en);


	if (raw->main_rotor_rpm >= HELICOPTER_ROTOR_SOUNDS_LOOPING_MAX_RPM_VALUE)
	{
		looping_amp = 1.0;
	}
	else if (raw->main_rotor_rpm <= HELICOPTER_ROTOR_SOUNDS_LOOPING_MIN_RPM_VALUE)
	{
		looping_amp = 0.0;
	}
	else
	{
		looping_amp = (raw->main_rotor_rpm - HELICOPTER_ROTOR_SOUNDS_LOOPING_MIN_RPM_VALUE) /
							(HELICOPTER_ROTOR_SOUNDS_LOOPING_MAX_RPM_VALUE - HELICOPTER_ROTOR_SOUNDS_LOOPING_MIN_RPM_VALUE);
	}

	STAGE (4, looping_amp)

	//Werewolf pitch
	looping_pitch = raw->main_rotor_rpm / 100.0;
	STAGE (5, looping_pitch)

	//For AI choppers we need to invert and tweak the RPM value changes, because AI choppers base their rotor RPM on the produced lift
	//(For the AI, more lift => more RPM, which is the exact opposite of what it should be)
	if (get_local_entity_int_value (en, INT_TYPE_PLAYER) == ENTITY_PLAYER_AI)
	{
		looping_pitch = 1.0; //Disabled since it simply doesn't sound right. Someday, someone should figure out a nice way of doing it.

//		looping_pitch = (1.0 - looping_pitch); //this would go from 0.1 to 0.3, so we need further tweaks
//		looping_pitch = looping_pitch / 12.0; //make the pitch changes less drastic
//		looping_pitch = 0.95 + looping_pitch;
	}

	//Slapping sound depends on how harsh the direction changes are: Do a tight turn and the rotor blades will slap
	if (get_current_flight_dynamics ())
	{
		acc_force = (get_3d_vector_magnitude (&current_flight_dynamics->model_acceleration_vector) / 15.0); //force vector can get as high as 30.0, but 15.0 should suffice for the maximum slap sound
		STAGE_EXTRA (0, get_3d_vector_magnitude (&current_flight_dynamics->model_acceleration_vector))
		STAGE (1, acc_force)
		slap_amp = (acc_force) * looping_amp;
		STAGE (2, slap_amp)
		slap_amp = bound (slap_amp, 0.0, 1.0);
		STAGE (3, slap_amp)
	}
	else
	{
		acc_force = 0.0;
		slap_amp = 0.0;
	}



	// arneh - get the engine rpm statistics so we can control the turbine sounds accordingly
	if (get_local_entity_int_value (en, INT_TYPE_PLAYER) == ENTITY_PLAYER_LOCAL
		&& get_current_flight_dynamics ())
	{
		float lrpm = current_flight_dynamics->left_engine_n1_rpm.value;
		float rrpm = current_flight_dynamics->right_engine_n1_rpm.value;
		float apu_rpm = current_flight_dynamics->apu_rpm.value;
		short play_both_sounds = fabs(lrpm - rrpm) > 1.0;    // only play one sound if both engines have same RPM

		// sound for left turbine:
		left_turbine_pitch = lrpm / 100.0;
		left_turbine_pitch = bound (left_turbine_pitch, 0.0, 1.6)+0.6;
		STAGE (8, left_turbine_pitch)
		left_turbine_amp = 0.2 + bound((lrpm - 20.0) * 0.01, 0.0, 0.4);
		STAGE (9, left_turbine_amp)

		if (lrpm > 1)
		{
			resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE1);
		}
		else
		{
			pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE1, 0.0);
		}

		if (play_both_sounds)
		{
			// sound for right turbine:
			right_turbine_pitch = rrpm / 100.0;
			right_turbine_pitch = bound (right_turbine_pitch, 0.0, 1.6)+0.6;
			STAGE (11, right_turbine_pitch)
			right_turbine_amp = 0.2 + bound((rrpm - 40.0) * 0.02, 0.0, 0.4);
			STAGE (12, right_turbine_amp)

			if (rrpm > 1)
			{
				resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE2);
			}
			else
			{
				pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE2, 0.0);
			}
		}
		else
		{
			pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE2, 0.0);
			left_turbine_amp *= 1.2;  // if only one sound is used for both engines, compensate by making it louder
			STAGE (10, left_turbine_amp)
		}

		// sound for APU:
		if (apu_rpm > 0.0)
		{
			apu_pitch = bound(apu_rpm * 0.02, 0.0, 2.0) + 0.2;
			STAGE (6, apu_pitch)
			apu_amp = bound(apu_rpm * 0.01, 0.0, 0.25);
			STAGE (7, apu_amp)
			resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_APU_TURBINE);
		}
		else
		{
			pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_APU_TURBINE, 0.0);
		}
	}
	else
	{
		left_turbine_pitch = right_turbine_pitch = 1.3;
		left_turbine_amp = right_turbine_amp = 1.0;
		apu_pitch = 1.0;
		apu_amp = 0.0;
	}

//	resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE);
//	winding_amp = 1.0 - (looping_amp * looping_amp);
	winding_amp = 0.75;

	if (looping_amp == 0.0)
	{
		pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING, 0.0);
		pause_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_SLAP, 0.0);
	}
	else
	{
		resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING);
		resume_local_entity_sound_type (en, ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_SLAP);
	}

	//
	// set sound effect amplification
	//

	spec = get_local_entity_first_child (en, LIST_TYPE_SPECIAL_EFFECT);

	while (spec)
	{

		if (spec->type == ENTITY_TYPE_SOUND_EFFECT)
		{
			switch (get_local_entity_int_value (spec, INT_TYPE_ENTITY_SUB_TYPE))
			{
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING:
//			if (get_local_entity_int_value (spec, INT_TYPE_ENTITY_SUB_TYPE) == ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_LOOPING)
				set_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION, (looping_amp-slap_amp)); //Werewolf: regular rotor sound cuts out when slapping sets in
				set_local_entity_float_value (spec, FLOAT_TYPE_SOUNDPITCH, looping_pitch);
				break;
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE1:
				set_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION, left_turbine_amp); //Werewolf: turbine sound
				set_local_entity_float_value (spec, FLOAT_TYPE_SOUNDPITCH, left_turbine_pitch);
				break;
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_TURBINE2:
				set_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION, right_turbine_amp);
				set_local_entity_float_value (spec, FLOAT_TYPE_SOUNDPITCH, right_turbine_pitch);
				break;
			case ENTITY_SUB_TYPE_EFFECT_SOUND_APU_TURBINE:
				set_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION, apu_amp);
				set_local_entity_float_value (spec, FLOAT_TYPE_SOUNDPITCH, apu_pitch);
				break;
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_UP1:
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_UP2:
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_DOWN1:
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_WIND_DOWN2:
				set_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION, winding_amp);
				break;
			//Werewolf TODO: Implement pitch and volume curves for turbine and slap sounds
			case ENTITY_SUB_TYPE_EFFECT_SOUND_ROTOR_SLAP:
				set_local_entity_float_value (spec, FLOAT_TYPE_AMPLIFICATION, (slap_amp)); //Werewolf: slapping sound
				set_local_entity_float_value (spec, FLOAT_TYPE_SOUNDPITCH, looping_pitch);
				break;
			}

		}

		spec = get_local_entity_child_succ (spec, LIST_TYPE_SPECIAL_EFFECT);
	}

}
