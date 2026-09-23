//
// The game type: free flight, campaign, skirmish or demo.
//
// C provenance: aphavoc/source/ui_menu/gametype/gametype.c :: game_type,
//               gametype.h :: #define get_game_type() (game_type),
//               global.h :: enum GAME_TYPES
//
// The front end assigns it when the player chooses a kind of game. Campaign code reads
// it to decide whether the campaign screen hears about campaign changes
// (ca_msgs.c :: notify_campaign_screen). Like the game status it is campaign
// state the host drives, so it is core state with a setter.
//

import { GameType } from "../generated/c-enums";

// C: a zero-initialised global (GAME_TYPE_INVALID)
let game_type: GameType = GameType.GAME_TYPE_INVALID;

export function resetGameType(): void {
	game_type = GameType.GAME_TYPE_INVALID;
}

// C: the front end's `game_type = ...` assignments
export function setGameType(type: GameType): void {
	game_type = type;
}

// C provenance: gametype.h :: #define get_game_type() (game_type)
export function getGameType(): GameType {
	return game_type;
}
