//! Tacview ACMI 2.2 text recording of the world.
//!
//! Positions are EECH world coordinates (metres, x east, y up, z north from
//! the map origin). They are placed on the globe the way EECH's own Tacview
//! writer places them (`aphavoc/source/entity/tacview/tacview.c`): the map's
//! origin at a reference latitude/longitude, latitude from z with EECH's
//! meridian-length series (M1..M3), longitude from x with its parallel-length
//! series (P1, P2).

use std::fmt::Write as _;
use std::io::Write;

/// tacview.c: metres per degree of latitude (series in the latitude)
const M1: f64 = 111_132.92;
const M2: f64 = -559.82;
const M3: f64 = 1.175;
/// tacview.c: metres per degree of longitude (series in the latitude)
const P1: f64 = 111_412.84;
const P2: f64 = -93.5;

pub struct Recorder<W: Write> {
    out: W,
    latitude: f64,
    latitude_scale: f64,
    last_frame: Option<f64>,
    text: String,
}

/// What an object is (Tacview `Type` tags) and its declared properties.
pub struct ObjectInfo<'a> {
    pub kind: &'a str,
    pub name: &'a str,
    pub coalition: &'a str,
    pub color: &'a str,
    pub group: Option<&'a str>,
    pub callsign: Option<&'a str>,
}

impl<W: Write> Recorder<W> {
    /// `latitude`, `longitude`: degrees of the EECH map origin
    pub fn new(mut out: W, title: &str, reference_time: &str, latitude: f64, longitude: f64) -> std::io::Result<Self> {
        let rad = latitude.to_radians();
        let latitude_scale = 1.0 / (M1 + M2 * (2.0 * rad).cos() + M3 * (4.0 * rad).cos());
        writeln!(out, "FileType=text/acmi/tacview")?;
        writeln!(out, "FileVersion=2.2")?;
        writeln!(out, "0,ReferenceTime={reference_time}")?;
        writeln!(out, "0,ReferenceLatitude={latitude}")?;
        writeln!(out, "0,ReferenceLongitude={longitude}")?;
        writeln!(out, "0,Title={title}")?;
        writeln!(out, "0,DataSource=EECH headless harness (eech-world)")?;
        writeln!(out, "0,DataRecorder=eech-world")?;
        Ok(Recorder {
            out,
            latitude,
            latitude_scale,
            last_frame: None,
            text: String::new(),
        })
    }

    /// degrees of (longitude, latitude) offsets from the reference for an EECH position
    fn offsets(&self, x: f64, z: f64) -> (f64, f64) {
        let dlat = z * self.latitude_scale;
        let lat = (self.latitude + dlat).to_radians().abs();
        let longitude_length = P1 * lat.cos() + P2 * (3.0 * lat).cos();
        (x / longitude_length, dlat)
    }

    pub fn frame(&mut self, time: f64) -> std::io::Result<()> {
        if self.last_frame != Some(time) {
            writeln!(self.out, "#{time:.2}")?;
            self.last_frame = Some(time);
        }
        Ok(())
    }

    pub fn declare(&mut self, id: u64, info: &ObjectInfo) -> std::io::Result<()> {
        self.text.clear();
        let _ = write!(
            self.text,
            "{id:x},Type={},Name={},Coalition={},Color={}",
            info.kind,
            escape(info.name),
            info.coalition,
            info.color
        );
        if let Some(g) = info.group {
            let _ = write!(self.text, ",Group={}", escape(g));
        }
        if let Some(c) = info.callsign {
            let _ = write!(self.text, ",CallSign={}", escape(c));
        }
        writeln!(self.out, "{}", self.text)
    }

    /// position (EECH metres) and heading (degrees, 0 north, clockwise)
    pub fn update(&mut self, id: u64, x: f64, y: f64, z: f64, heading: Option<f64>) -> std::io::Result<()> {
        let (dlon, dlat) = self.offsets(x, z);
        match heading {
            Some(h) => writeln!(self.out, "{id:x},T={dlon:.7}|{dlat:.7}|{y:.1}|||{h:.1}"),
            None => writeln!(self.out, "{id:x},T={dlon:.7}|{dlat:.7}|{y:.1}"),
        }
    }

    pub fn property(&mut self, id: u64, key: &str, value: &str) -> std::io::Result<()> {
        writeln!(self.out, "{id:x},{key}={}", escape(value))
    }

    pub fn remove(&mut self, id: u64) -> std::io::Result<()> {
        writeln!(self.out, "-{id:x}")
    }

    /// Tacview events: Message, Bookmark, TakenOff, Landed, Destroyed, ...
    pub fn event(&mut self, kind: &str, ids: &[u64], text: &str) -> std::io::Result<()> {
        let mut line = format!("0,Event={kind}");
        for id in ids {
            let _ = write!(line, "|{id:x}");
        }
        let _ = write!(line, "|{}", escape(text));
        writeln!(self.out, "{line}")
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }

    pub fn into_inner(self) -> W {
        self.out
    }
}

/// ACMI text values escape commas (and newlines) with a backslash
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace(',', "\\,").replace('\n', "\\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_well_formed_acmi() {
        let mut r = Recorder::new(Vec::new(), "t", "2026-09-24T06:00:00Z", 42.0, 41.0).unwrap();
        r.frame(0.0).unwrap();
        r.declare(
            0x10,
            &ObjectInfo {
                kind: "Air+Rotorcraft",
                name: "UH-60A",
                coalition: "Blue",
                color: "Blue",
                group: Some("lift, 1"),
                callsign: None,
            },
        )
        .unwrap();
        r.update(0x10, 0.0, 10.0, 11_100.0, Some(90.0)).unwrap();
        r.frame(1.0).unwrap();
        r.remove(0x10).unwrap();
        let text = String::from_utf8(r.into_inner()).unwrap();
        assert!(text.starts_with("FileType=text/acmi/tacview\nFileVersion=2.2\n"));
        assert!(text.contains("Group=lift\\, 1"));
        // 11.1 km north is about 0.1 degree of latitude at 42 N
        let t = text.lines().find(|l| l.starts_with("10,T=")).unwrap();
        let lat: f64 = t.split('|').nth(1).unwrap().parse().unwrap();
        assert!((lat - 0.0999).abs() < 0.001, "{t}");
        assert!(text.contains("#1.00\n-10\n"));
    }
}
