//
// Port: the frame clock.
//
// C provenance: modules/system/time.c :: set_delta_time, lock_frame_rate / unlock_frame_rate
//
// EECH measures each frame with timeGetTime (never less than 0.001 s while
// unlocked) or uses the locked frame rate's fixed delta, and stores the result
// in system_delta_time, which the campaign reads through get_delta_time ().
// Measuring time is the environment's job. The campaign core only needs the
// result, and whether the frame rate is locked, because a locked frame rate
// changes how the entity update loop subdivides the frame
// (up_update.c, set_manual_delta_time).
//
// Values are narrowed to C `float` by the core, not by the adapter. The core
// narrows toward zero (EECH's FPU rounding); an adapter that measures in
// double and wants EECH's own measurement arithmetic must reproduce it.
//

export interface Clock {
	// The frame's delta time in seconds (C: system_delta_time after set_delta_time ()).
	getDeltaTime(): number;

	// C: locked_frame_rate
	isFrameRateLocked(): boolean;
}
