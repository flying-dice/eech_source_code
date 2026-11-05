use crate::{Bounds, EntityId, ObjectModel, Position};

/// The physical world the campaign runs in: what the campaign asks of its
/// environment while it runs.
///
/// Derived from the original C call sites the implemented slice reaches (see
/// docs/ports.md):
///
/// * [`World::position`] — the flight model's position of a campaign aircraft
///   (EECH reads `VEC3D_TYPE_POSITION` of a helicopter or fixed-wing member,
///   e.g. for a group's position, its supply assessment and a mission's ETA);
/// * [`World::object_bounds`] — the 3D object database (EECH
///   `get_object_3d_bounding_box`, e.g. to lay out supply crates at a keysite).
///
/// The campaign asks synchronously, in the middle of its own update: EECH is
/// not an observe → step → command machine, it pulls physical state when its
/// logic needs it. Answers must be deterministic for a deterministic
/// campaign. `None` means the world does not know; the campaign call then
/// fails with [`crate::CampaignError::World`] rather than inventing a value.
///
/// A world implementation must not call back into the campaign (the campaign
/// is mutably borrowed by the step that is asking).
pub trait World {
    fn position(&self, entity: EntityId) -> Option<Position>;

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds>;
}

impl<W: World + ?Sized> World for &W {
    fn position(&self, entity: EntityId) -> Option<Position> {
        (**self).position(entity)
    }

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds> {
        (**self).object_bounds(model)
    }
}

impl<W: World + ?Sized> World for &mut W {
    fn position(&self, entity: EntityId) -> Option<Position> {
        (**self).position(entity)
    }

    fn object_bounds(&self, model: ObjectModel) -> Option<Bounds> {
        (**self).object_bounds(model)
    }
}
