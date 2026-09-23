//
// The game status: the campaign's lifecycle phase.
//
// C provenance: aphavoc/source/global.c :: game_status, set_game_status
//               aphavoc/source/global.h :: #define get_game_status() (game_status)
//
// The host's game flow moves it through its phases (init.c: UNINITIALISED;
// gameflow.c: INITIALISING while a session is built, created or joined;
// flight.c: INITIALISED when the flight loop starts, UNINITIALISING when it
// ends). Campaign code reads it to tell a session under construction from a
// running one: keysite.c :: update_keysite_cargo does not derive crates while
// INITIALISING, because a restored session brings its packed cargo with it.
//
// It is campaign state the host drives, like the comms model, not a query
// about the environment, so it is core state with a setter rather than a
// port (docs/slices/keysite-cargo.md, Investigation 3). game_status_string
// is not ported: no campaign code reads it.
//

import { GameStatusType } from "../generated/c-enums";

// C: a zero-initialised global
let game_status: GameStatusType = GameStatusType.GAME_STATUS_UNINITIALISED;

export function resetGameStatus(): void {
	game_status = GameStatusType.GAME_STATUS_UNINITIALISED;
}

// C provenance: global.c :: set_game_status
export function setGameStatus(status: GameStatusType): void {
	game_status = status;
}

// C provenance: global.h :: #define get_game_status() (game_status)
export function getGameStatus(): GameStatusType {
	return game_status;
}
