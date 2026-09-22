//
// Per entity type function tables.
//
// C provenance: the fn_* arrays of entity/system/en_funcs (e.g.
//   fn_get_local_entity_int_value [NUM_ENTITY_TYPES][NUM_INT_TYPES]),
// filled with defaults and then overloaded by each entity type's
// overload_*_functions ().
//
// The port starts every entry as unported. Only overloads that have been
// ported are installed, so reaching an unported entry fails loudly instead of
// silently running an EECH default that the real overload would have replaced.
//

import { UnportedBehaviourError } from "../../core/assert";
import { EntityType } from "../../generated/c-enums";

export class EntityFunctionTable<F> {
	private readonly entries: Record<number, Record<number, F>> = {};

	public constructor(private readonly tableName: string) {}

	public overload(entityType: EntityType, key: number, fn: F): void {
		let row = this.entries[entityType];

		if (row === undefined) {
			row = {};
			this.entries[entityType] = row;
		}

		row[key] = fn;
	}

	public lookup(entityType: EntityType, key: number, keyName: string): F {
		const row = this.entries[entityType];

		const fn = row === undefined ? undefined : row[key];

		if (fn === undefined) {
			throw new UnportedBehaviourError(`${this.tableName} [${EntityType[entityType]}] [${keyName}]`);
		}

		return fn;
	}
}
