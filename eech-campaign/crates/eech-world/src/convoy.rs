//! The host's physical record of its ground convoy vehicles, keyed by the host
//! identity it issued at `bind_mobile`. The campaign asks for a step; the host
//! owns the position, speed and attitude it keeps, and the answer is what the
//! campaign applies.
//!
//! The campaign still holds a copy of each pose (combat, perception, arrival and
//! Tacview read it). On a step the copy is sent only to be compared with the
//! record: the host never adopts it, so a write the host did not make is an
//! error, not a silent overwrite. The campaign says so explicitly (`Place`) when
//! it puts a vehicle somewhere.

use std::collections::HashMap;
use std::os::raw::c_int;

extern "C" {
    fn eech_convoy_step(
        position: *mut f32,
        velocity: *mut f32,
        attitude: *mut f32,
        delta_seconds: f32,
        desired_velocity: f32,
        max_acceleration: f32,
        waypoint: *const f32,
        tacview_logging: c_int,
        terrain_present: c_int,
        elevation: f32,
        forest: c_int,
        face_normal: *const f32,
        map_min: *const f32,
        map_max: *const f32,
        rotated: *mut c_int,
    );
}

/// Position in EECH world metres, speed in m/s, a row-major 3x3 attitude.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: [f32; 3],
    pub velocity: f32,
    pub attitude: [f32; 9],
}

impl Pose {
    /// Bit-for-bit equality (the campaign's copy must be exactly the host's record):
    /// the first component that differs, with both values.
    fn first_difference(&self, other: &Pose) -> Option<String> {
        let parts: [(&str, &[f32], &[f32]); 3] = [
            ("position", &self.position, &other.position),
            ("velocity", std::slice::from_ref(&self.velocity), std::slice::from_ref(&other.velocity)),
            ("attitude", &self.attitude, &other.attitude),
        ];
        parts.iter().find_map(|(what, ours, theirs)| {
            let index = ours.iter().zip(*theirs).position(|(a, b)| a.to_bits() != b.to_bits())?;
            Some(format!("{what}[{index}] ({} against {})", ours[index], theirs[index]))
        })
    }
}

/// Terrain at the pre-move position, sampled by the campaign.
#[derive(Clone, Copy, Debug)]
pub struct Terrain {
    pub elevation: f32,
    pub forest: bool,
    pub face_normal: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct Advance {
    pub delta_seconds: f32,
    pub desired_velocity: f32,
    pub max_acceleration: f32,
    pub waypoint: [f32; 3],
    pub tacview_logging: bool,
    pub terrain: Option<Terrain>,
    pub map_min: [f32; 3],
    pub map_max: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub enum Request {
    /// step towards the waypoint
    Advance(Advance),
    /// the campaign zeroes the speed
    Stop,
    /// the campaign placed the vehicle: adopt its pose
    Place,
}

#[derive(Default)]
pub struct Convoys {
    vehicles: HashMap<String, Pose>,
}

impl Convoys {
    /// Start the record of a vehicle the host just issued `host_id` to.
    pub fn bind(&mut self, host_id: &str, pose: Pose) {
        self.vehicles.insert(host_id.to_owned(), pose);
    }

    /// Run one request. `campaign` is the campaign's copy of the pose; the
    /// result is the host's record afterwards, and whether the attitude turned.
    pub fn step(&mut self, host_id: &str, campaign: Pose, request: Request) -> Result<(Pose, bool), String> {
        let record = self
            .vehicles
            .get_mut(host_id)
            .ok_or_else(|| format!("host.move_vehicle: unknown host id {host_id}"))?;
        if matches!(request, Request::Place) {
            *record = campaign;
            return Ok((*record, false));
        }
        if let Some(what) = campaign.first_difference(record) {
            return Err(format!(
                "host.move_vehicle: the campaign's {what} for {host_id} differs from the host's record (a write the host did not make)"
            ));
        }
        let mut rotated = false;
        match request {
            Request::Advance(advance) => rotated = advance_pose(record, &advance),
            Request::Stop => record.velocity = 0.0,
            Request::Place => unreachable!("placed above"),
        }
        Ok((*record, rotated))
    }
}

fn advance_pose(record: &mut Pose, advance: &Advance) -> bool {
    let terrain = advance.terrain.unwrap_or(Terrain {
        elevation: 0.0,
        forest: false,
        face_normal: [0.0; 3],
    });
    let mut rotated: c_int = 0;
    // SAFETY: every pointer is to a live array of the length the C step reads (3, 1, 9),
    // and the step keeps none of them.
    unsafe {
        eech_convoy_step(
            record.position.as_mut_ptr(),
            &mut record.velocity,
            record.attitude.as_mut_ptr(),
            advance.delta_seconds,
            advance.desired_velocity,
            advance.max_acceleration,
            advance.waypoint.as_ptr(),
            c_int::from(advance.tacview_logging),
            c_int::from(advance.terrain.is_some()),
            terrain.elevation,
            c_int::from(terrain.forest),
            terrain.face_normal.as_ptr(),
            advance.map_min.as_ptr(),
            advance.map_max.as_ptr(),
            &mut rotated,
        );
    }
    rotated != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL: [f32; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

    fn pose(x: f32, z: f32) -> Pose {
        Pose {
            position: [x, 0.0, z],
            velocity: 0.0,
            attitude: LEVEL,
        }
    }

    fn advance(waypoint: [f32; 3]) -> Advance {
        Advance {
            delta_seconds: 1.0,
            desired_velocity: 10.0,
            max_acceleration: 4.0,
            waypoint,
            tacview_logging: true,
            terrain: None,
            map_min: [-1.0e6; 3],
            map_max: [1.0e6; 3],
        }
    }

    fn assert_near(actual: [f32; 3], expected: [f32; 3]) {
        for (actual, expected) in actual.iter().zip(expected) {
            assert!((actual - expected).abs() < 1.0e-5, "{actual:?} vs {expected:?}");
        }
    }

    fn convoys(host_id: &str, start: Pose) -> Convoys {
        let mut convoys = Convoys::default();
        convoys.bind(host_id, start);
        convoys
    }

    #[test]
    fn accelerates_towards_the_waypoint_and_keeps_its_own_record() {
        let mut convoys = convoys("v", pose(0.0, 0.0));
        let (first, rotated) = convoys.step("v", pose(0.0, 0.0), Request::Advance(advance([0.0, 0.0, 100.0]))).unwrap();
        // 4 m/s^2 for 1 s, then one second at that speed, due +z
        assert_eq!(first.velocity, 4.0);
        assert_near(first.position, [0.0, 0.0, 4.0]);
        assert!(!rotated, "already facing +z");
        // the next step compares the campaign's copy with the record, so it must be what was returned
        let (second, _) = convoys.step("v", first, Request::Advance(advance([0.0, 0.0, 100.0]))).unwrap();
        assert_eq!(second.velocity, 8.0);
        assert_near(second.position, [0.0, 0.0, 12.0]);
    }

    #[test]
    fn turning_towards_a_waypoint_reports_the_rotation() {
        let mut convoys = convoys("v", pose(0.0, 0.0));
        let (turned, rotated) = convoys.step("v", pose(0.0, 0.0), Request::Advance(advance([100.0, 0.0, 0.0]))).unwrap();
        assert!(rotated);
        // facing +x: heading row [sin, 0, cos]
        assert!((turned.attitude[6] - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn a_vehicle_on_its_waypoint_is_not_moved() {
        let mut convoys = convoys("v", pose(5.0, 5.0));
        let (same, rotated) = convoys.step("v", pose(5.0, 5.0), Request::Advance(advance([5.0, 0.0, 5.5]))).unwrap();
        assert_eq!(same, pose(5.0, 5.0));
        assert!(!rotated);
    }

    #[test]
    fn the_map_volume_bounds_the_position() {
        let mut convoys = convoys("v", pose(0.0, 0.0));
        let mut towards_the_edge = advance([0.0, 0.0, 100.0]);
        towards_the_edge.map_max = [1.0e6, 1.0e6, 2.0];
        let (bounded, _) = convoys.step("v", pose(0.0, 0.0), Request::Advance(towards_the_edge)).unwrap();
        assert_eq!(bounded.position[2], 2.0);
    }

    #[test]
    fn a_campaign_copy_the_host_did_not_make_is_refused() {
        let mut convoys = convoys("v", pose(0.0, 0.0));
        let error = convoys.step("v", pose(1.0, 0.0), Request::Advance(advance([0.0, 0.0, 100.0]))).unwrap_err();
        assert!(error.contains("position[0]") && error.contains("differs from the host's record"), "{error}");
        let error = convoys.step("v", pose(1.0, 0.0), Request::Stop).unwrap_err();
        assert!(error.contains("position"), "{error}");
    }

    #[test]
    fn placing_adopts_the_campaign_pose_and_stopping_zeroes_the_speed() {
        let mut convoys = convoys("v", pose(0.0, 0.0));
        let mut moving = pose(50.0, 60.0);
        moving.velocity = 7.0;
        let (adopted, _) = convoys.step("v", moving, Request::Place).unwrap();
        assert_eq!(adopted, moving);
        let (stopped, _) = convoys.step("v", moving, Request::Stop).unwrap();
        assert_eq!(stopped.velocity, 0.0);
        assert_eq!(stopped.position, moving.position);
    }

    #[test]
    fn an_unknown_host_id_is_an_error() {
        let mut convoys = Convoys::default();
        for request in [Request::Stop, Request::Place, Request::Advance(advance([0.0; 3]))] {
            let error = convoys.step("eech-world:9", pose(0.0, 0.0), request).unwrap_err();
            assert!(error.contains("unknown host id eech-world:9"), "{error}");
        }
    }
}
