//! DCS points ⇄ EECH positions.
//!
//! * DCS (`Unit.getPoint`, `Airbase.getPoint`): metres, **x north**, y up,
//!   **z east**, origin at the theatre's origin (negative coordinates are
//!   common).
//! * EECH (`Position`): metres, **x east**, y up, **z north**, origin at the
//!   south-west corner of the campaign map, which is a grid of sectors from
//!   (0, 0).
//!
//! The adapter places the campaign map on the theatre with an origin: the DCS
//! point (`origin.x` north, `origin.z` east) that is EECH (0, 0). Then
//! `eech.x = dcs.z − origin.z`, `eech.z = dcs.x − origin.x`, `eech.y = dcs.y`.
//! (Swapping the horizontal axes is a reflection; it maps points, which is all
//! the campaign exchanges. Headings would need their own conversion.)

use eech_campaign::Position;

/// Where the campaign map's (0, 0) lies in the theatre, in DCS coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Origin {
    /// DCS x (north) of EECH (0, 0)
    pub x: f64,
    /// DCS z (east) of EECH (0, 0)
    pub z: f64,
}

/// A DCS point as DCS gives it.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DcsPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[allow(clippy::cast_possible_truncation)] // EECH positions are C floats
pub fn to_eech(point: DcsPoint, origin: Origin) -> Position {
    Position::new((point.z - origin.z) as f32, point.y as f32, (point.x - origin.x) as f32)
}

pub fn to_dcs(position: Position, origin: Origin) -> DcsPoint {
    DcsPoint {
        x: f64::from(position.z) + origin.x,
        y: f64::from(position.y),
        z: f64::from(position.x) + origin.z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_is_eech_z_and_east_is_eech_x() {
        let origin = Origin { x: -300_000.0, z: 600_000.0 };
        // 10 km north and 3 km east of the origin
        let p = to_eech(
            DcsPoint {
                x: -290_000.0,
                y: 25.0,
                z: 603_000.0,
            },
            origin,
        );
        assert_eq!(p, Position::new(3000.0, 25.0, 10_000.0));
        assert_eq!(
            to_dcs(p, origin),
            DcsPoint {
                x: -290_000.0,
                y: 25.0,
                z: 603_000.0
            }
        );
    }
}
