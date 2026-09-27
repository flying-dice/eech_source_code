//! Map coordinates: EECH world metres (x east, z north) from a map origin at
//! (latitude, longitude). The conversion is EECH's own Tacview geodesy
//! (`aphavoc/source/entity/tacview/tacview.c`, and eech-world's recorder), so
//! a position on the map and its Tacview position agree exactly.

/// tacview.c: metres per degree of latitude (series in the latitude)
const M1: f64 = 111_132.92;
const M2: f64 = -559.82;
const M3: f64 = 1.175;
/// tacview.c: metres per degree of longitude (series in the latitude)
const P1: f64 = 111_412.84;
const P2: f64 = -93.5;

#[derive(Clone, Copy, Debug)]
pub struct Geo {
    pub latitude: f64,
    pub longitude: f64,
    metres_per_degree_latitude: f64,
}

impl Geo {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        let rad = latitude.to_radians();
        Geo {
            latitude,
            longitude,
            metres_per_degree_latitude: M1 + M2 * (2.0 * rad).cos() + M3 * (4.0 * rad).cos(),
        }
    }

    fn metres_per_degree_longitude(latitude: f64) -> f64 {
        let lat = latitude.to_radians().abs();
        P1 * lat.cos() + P2 * (3.0 * lat).cos()
    }

    /// (x, z) metres from the origin
    pub fn to_map(&self, latitude: f64, longitude: f64) -> (f64, f64) {
        let z = (latitude - self.latitude) * self.metres_per_degree_latitude;
        let x = (longitude - self.longitude) * Self::metres_per_degree_longitude(latitude);
        (x, z)
    }

    /// (latitude, longitude)
    pub fn to_geo(&self, x: f64, z: f64) -> (f64, f64) {
        let latitude = self.latitude + z / self.metres_per_degree_latitude;
        (latitude, self.longitude + x / Self::metres_per_degree_longitude(latitude))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let g = Geo::new(49.4, 5.7);
        let (lat, lon) = g.to_geo(30_000.0, 40_000.0);
        let (x, z) = g.to_map(lat, lon);
        assert!((x - 30_000.0).abs() < 1e-6 && (z - 40_000.0).abs() < 1e-6);
    }
}
