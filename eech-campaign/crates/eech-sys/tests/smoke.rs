use eech_sys::*;
use std::collections::HashMap;

#[derive(Default)]
struct Replay {
    out: String,
    positions: HashMap<Ref, [f32; 3]>,
    bounds: HashMap<i32, [f32; 6]>,
}

impl Host for Replay {
    fn mobile_position(&mut self, m: Ref) -> Result<[f32; 3], HostError> {
        self.positions.get(&m).copied().ok_or(HostError(format!("no position for {m:?}")))
    }
    fn object_bounds(&mut self, o: i32) -> Result<[f32; 6], HostError> {
        self.bounds.get(&o).copied().ok_or(HostError(format!("no bounds for {o}")))
    }
    fn event(&mut self, _e: RawEvent) -> Result<(), HostError> {
        Ok(())
    }
    fn output(&mut self, t: &str) -> Result<(), HostError> {
        self.out.push_str(t);
        Ok(())
    }
    fn declare(&mut self, d: Declaration) -> Result<(), HostError> {
        match d {
            Declaration::MobilePosition { mobile, position } => self.positions.insert(mobile, position).map(|_| ()).unwrap_or(()),
            Declaration::ObjectBounds { object, bounds } => self.bounds.insert(object, bounds).map(|_| ()).unwrap_or(()),
        }
        Ok(())
    }
}

#[test]
fn replay_slice1() {
    let mut h = Replay::default();
    legacy_replay(
        &mut h,
        "session 1\nforce 1\nkeysite 1 0 1 1000 1000 50 50\ngroup 6 1 10 10 1 0 0 1 1000 1000\nop assess\n",
    )
    .unwrap();
    println!("{}", h.out);
}

#[test]
fn probe() {
    println!("{:?} {:?}", probe::va_list_reinterpretation(), probe::marshalled());
}
