//
// Delta time as the campaign sees it.
//
// C provenance: modules/system/time.c, modules/system/time.h
//
//   float system_delta_time = 0.1, system_one_over_delta_time = 10.0;
//   int locked_frame_rate = FALSE;
//   #define get_delta_time() (system_delta_time)
//   void set_manual_delta_time (float delta_time);
//
// set_delta_time () measures the frame and belongs to the environment (the
// Clock port). The entity update loop reads and overrides the delta, so that
// part is core state.
//
// Not ported: system_one_over_delta_time and the delta history (and hence
// get_delta_time_average), which no ported code reads.
//

import { toFloat32 } from "./float32";

export interface FrameTime {
	getDeltaTime(): number;
	isFrameRateLocked(): boolean;
}

let system_delta_time = toFloat32(0.1);

let locked_frame_rate = false;

// C: static initialisers of time.c
export function resetDeltaTime(): void {
	system_delta_time = toFloat32(0.1);
	locked_frame_rate = false;
}

// C provenance: time.h :: #define get_delta_time() (system_delta_time)
export function getDeltaTime(): number {
	return system_delta_time;
}

// C provenance: time.h :: extern int locked_frame_rate
export function isFrameRateLocked(): boolean {
	return locked_frame_rate;
}

// C provenance: time.c :: set_manual_delta_time (the float parameter narrows)
export function setManualDeltaTime(delta_time: number): void {
	if (!locked_frame_rate) {
		system_delta_time = toFloat32(delta_time);
	}

	// else: debug_log ("TIME: cannot set locked delta time")
}

// C provenance: time.c :: set_delta_time. The measurement itself is the
// environment's: the Clock port supplies its result.
export function setDeltaTimeFrom(clock: FrameTime): void {
	locked_frame_rate = clock.isFrameRateLocked();

	system_delta_time = toFloat32(clock.getDeltaTime());
}
