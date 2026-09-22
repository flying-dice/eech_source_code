//
// C provenance: aphavoc/source/comms/comms.c :: get_comms_model / set_comms_model
//
// The campaign core is the authoritative simulation, i.e. EECH's
// COMMS_MODEL_SERVER. The client model exists in the enum and dispatch tables
// but no client overload is ported (see docs/port-manifest.md).
//

import { CommsModelType } from "../../generated/c-enums";

export type CommsModel = CommsModelType.COMMS_MODEL_SERVER | CommsModelType.COMMS_MODEL_CLIENT;

let commsModel: CommsModel = CommsModelType.COMMS_MODEL_SERVER;

export function getCommsModel(): CommsModel {
	return commsModel;
}

export function setCommsModel(model: CommsModel): void {
	commsModel = model;
}
