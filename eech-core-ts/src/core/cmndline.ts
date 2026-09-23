//
// Campaign configuration that EECH reads from the command line / EECH.INI.
//
// C provenance: aphavoc/source/cmndline.c, eechini.c
//

// C: command_line_entity_update_frame_rate = 2.0 (cmndline.c); EECH.INI "entity update frame rate"
export const DEFAULT_ENTITY_UPDATE_FRAME_RATE = 2;

let command_line_entity_update_frame_rate = DEFAULT_ENTITY_UPDATE_FRAME_RATE;

export function getCommandLineEntityUpdateFrameRate(): number {
	return command_line_entity_update_frame_rate;
}

export function setCommandLineEntityUpdateFrameRate(frame_rate: number): void {
	command_line_entity_update_frame_rate = frame_rate;
}
